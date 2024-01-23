use std::convert::Infallible;
use webgraph::prelude::BVGraphCodesWriter;
use crate::bvgraph::BVGraphComponent;
use crate::{Freq, MAX_RAW_SYMBOL, Symbol};
use crate::utils::ans_utilities::{folding_without_streaming_out};


/// A trait for those mock writers that can be buildable.
pub trait MockWriter {

    /// Builds a mock writer from a given costs table.
    fn build(costs_table: ANSymbolTable, fidelity: usize, radix: usize) -> Self;
}


#[derive(Clone)]
pub struct ANSymbolTable {

    /// A table containing a list of costs for each model. Each list containing, at index i, the cost of encoding the
    /// symbol i.
    pub table: Vec<Vec<usize>>,
}

impl ANSymbolTable {

    /// Returns a new ANSymbolTable, that is a table containing for each component, the cost of every symbol calculated
    /// as follow:
    /// ```text
    /// C(x) = ((1 / p) * 2^16) + ((bytes_to_unfold * radix) * 2^16)
    /// ```
    pub fn new(symbol_freqs: Vec<Vec<Freq>>, frame_sizes: Vec<usize>, fidelity: usize, radix: usize) -> Self {
        let mut table = Self::initialize_with_binary_cost(fidelity, radix).table;
        let folding_threshold = 1u16 << (fidelity + radix - 1);
        let folding_offset = ((1u16 << radix) - 1) * (1 << (fidelity - 1));

        table
            .iter_mut()
            .enumerate()
            .for_each(|(component, current_table)| {
                (0..current_table.len())
                    .into_iter()
                    .for_each(|symbol| {
                        let symbol_freq = match symbol_freqs[component].get(symbol) {
                            Some(freq) => match *freq {
                                0 => 1, // we can have 0 frequencies for symbols that exists due to bigger ones
                                _ => *freq,
                            },
                            None => 1,
                        };

                        current_table[symbol] = Self::calculate_symbol_cost(
                            symbol as Symbol,
                            symbol_freq,
                            frame_sizes[component],
                            folding_offset,
                            folding_threshold,
                            radix,
                        );
                    });
            });

        Self { table }
    }

    /// Creates a a table of [`BVGraphComponent::COMPONENTS`] lists, each containing, at the index i, the cost of the
    /// symbol i calculate as follow:
    /// ```text
    ///  C(x) = (floor(log2(x)) + 1) + (bytes_to_unfold * radix)
    /// ```
    pub fn initialize_with_binary_cost(fidelity: usize, radix: usize) -> Self {
        let max_folded_sym = folding_without_streaming_out(MAX_RAW_SYMBOL, radix, fidelity);
        let folding_threshold = 1u16 << (fidelity + radix - 1);
        let folding_offset = ((1u16 << radix) - 1) * (1 << (fidelity - 1));

        let table = (0..BVGraphComponent::COMPONENTS)
            .map(|_component| {
                (0..max_folded_sym + 1)
                    .map(|symbol| Self::get_binary_cost(symbol, folding_threshold, folding_offset, radix))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        Self { table }
    }

    fn calculate_symbol_cost (
        symbol: Symbol,
        freq: Freq,
        frame_size: usize,
        folding_offset: u16,
        folding_threshold:u16,
        radix: usize,
    ) -> usize
    {
        // we shouldn't have a symbol with frequency 0 since we want to have the cost for each symbol
        debug_assert!(freq != 0);

        let bytes_to_unfold = match symbol < folding_threshold {
            true => 0_u16,
            false => (symbol - folding_threshold) / folding_offset + 1u16,
        };

        let probability = freq as f64 / (1u64 << frame_size) as f64;
        let inverse = 1.0 / probability;
        let shifted = (inverse * ((1 << 16) as f64)).round() as usize;

        shifted + ((bytes_to_unfold as usize * radix) * (1 << 16))
    }

    fn get_binary_cost(symbol: Symbol, folding_threshold: u16, folding_offset: u16, radix: usize) -> usize {
        let bytes_to_unfold = match symbol < folding_threshold {
            true => 0_usize,
            false => ((symbol - folding_threshold) / folding_offset) as usize + 1_usize,
        };

        (symbol.checked_ilog2().unwrap_or(0) as usize + 1)  + bytes_to_unfold * radix
    }
}

#[derive(Clone)]
pub struct EntropyMockWriter {

    costs_table: ANSymbolTable,

    fidelity: usize,

    radix: usize,

    folding_threshold: u64,
}

impl MockWriter for EntropyMockWriter {

    fn build(costs_table: ANSymbolTable, fidelity: usize, radix: usize) -> Self {
        Self {
            costs_table,
            fidelity,
            radix,
            folding_threshold: 1u64 << (fidelity + radix - 1),
        }
    }
}

impl BVGraphCodesWriter for EntropyMockWriter {
    type Error = Infallible;

    type MockWriter = Self; // it's essentially a marker

    fn mock(&self) -> Self::MockWriter {
         Self {
             costs_table: ANSymbolTable::initialize_with_binary_cost(1, 1),
                fidelity: 0,
                radix: 0,
                folding_threshold: 0,
         } // thus we can return a fake one
    }

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.folding_threshold {
            return Ok(self.costs_table.table[BVGraphComponent::Outdegree as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.costs_table.table[BVGraphComponent::Outdegree as usize][folded_sym as usize])
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.folding_threshold {
            return Ok(self.costs_table.table[BVGraphComponent::ReferenceOffset as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.costs_table.table[BVGraphComponent::ReferenceOffset as usize][folded_sym as usize])
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.folding_threshold {
            return Ok(self.costs_table.table[BVGraphComponent::BlockCount as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.costs_table.table[BVGraphComponent::BlockCount as usize][folded_sym as usize])
    }

    fn write_blocks(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.folding_threshold {
            return Ok(self.costs_table.table[BVGraphComponent::Blocks as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.costs_table.table[BVGraphComponent::Blocks as usize][folded_sym as usize])
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.folding_threshold {
            return Ok(self.costs_table.table[BVGraphComponent::IntervalCount as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.costs_table.table[BVGraphComponent::IntervalCount as usize][folded_sym as usize])
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.folding_threshold {
            return Ok(self.costs_table.table[BVGraphComponent::IntervalStart as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.costs_table.table[BVGraphComponent::IntervalStart as usize][folded_sym as usize])
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.folding_threshold {
            return Ok(self.costs_table.table[BVGraphComponent::IntervalLen as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.costs_table.table[BVGraphComponent::IntervalLen as usize][folded_sym as usize])
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.folding_threshold {
            return Ok(self.costs_table.table[BVGraphComponent::FirstResidual as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.costs_table.table[BVGraphComponent::FirstResidual as usize][folded_sym as usize])
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.folding_threshold {
            return Ok(self.costs_table.table[BVGraphComponent::Residual as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.costs_table.table[BVGraphComponent::Residual as usize][folded_sym as usize])
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}


pub struct Log2MockWriter {
    costs_table: ANSymbolTable,

    fidelity: usize,

    radix: usize,

    folding_threshold: u64,
}

impl MockWriter for Log2MockWriter {

    fn build(_costs_table: ANSymbolTable, fidelity: usize, radix: usize) -> Self {
        Self {
            costs_table: ANSymbolTable::initialize_with_binary_cost(fidelity, radix),
            fidelity,
            radix,
            folding_threshold: 1 << (fidelity + radix - 1),
        }
    }
}

impl BVGraphCodesWriter for Log2MockWriter {
    type Error = Infallible;

    type MockWriter = Self; // it's essentially a marker

    fn mock(&self) -> Self::MockWriter {
        Log2MockWriter {
            costs_table: ANSymbolTable::initialize_with_binary_cost(1, 1),
            radix: 1,
            fidelity: 1,
            folding_threshold:1
        } // thus we can return a fake one
    }

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.folding_threshold {
            return Ok(self.costs_table.table[BVGraphComponent::Outdegree as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.costs_table.table[BVGraphComponent::Outdegree as usize][folded_sym as usize])
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.folding_threshold {
            return Ok(self.costs_table.table[BVGraphComponent::ReferenceOffset as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.costs_table.table[BVGraphComponent::ReferenceOffset as usize][folded_sym as usize])
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.folding_threshold {
            return Ok(self.costs_table.table[BVGraphComponent::BlockCount as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.costs_table.table[BVGraphComponent::BlockCount as usize][folded_sym as usize])
    }

    fn write_blocks(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.folding_threshold {
            return Ok(self.costs_table.table[BVGraphComponent::Blocks as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.costs_table.table[BVGraphComponent::Blocks as usize][folded_sym as usize])
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.folding_threshold {
            return Ok(self.costs_table.table[BVGraphComponent::IntervalCount as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.costs_table.table[BVGraphComponent::IntervalCount as usize][folded_sym as usize])
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.folding_threshold {
            return Ok(self.costs_table.table[BVGraphComponent::IntervalStart as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.costs_table.table[BVGraphComponent::IntervalStart as usize][folded_sym as usize])
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.folding_threshold {
            return Ok(self.costs_table.table[BVGraphComponent::IntervalLen as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.costs_table.table[BVGraphComponent::IntervalLen as usize][folded_sym as usize])
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.folding_threshold {
            return Ok(self.costs_table.table[BVGraphComponent::FirstResidual as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.costs_table.table[BVGraphComponent::FirstResidual as usize][folded_sym as usize])
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.folding_threshold {
            return Ok(self.costs_table.table[BVGraphComponent::Residual as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.costs_table.table[BVGraphComponent::Residual as usize][folded_sym as usize])
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}