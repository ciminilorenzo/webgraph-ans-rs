use std::convert::Infallible;
use webgraph::prelude::BVGraphCodesWriter;
use crate::bvgraph::Component;
use crate::{Freq, MAX_RAW_SYMBOL, Symbol};
use crate::utils::ans_utilities::{folding_without_streaming_out};


/// All mock writer have to be buildable.
pub trait MockWriter<const FIDELITY: usize, const RADIX: usize> {
    fn build(costs_table: ANSymbolTable<FIDELITY, RADIX>) -> Self;
}


#[derive(Clone)]
pub struct ANSymbolTable<const FIDELITY: usize, const RADIX: usize> {

    /// A table containing a list of costs for each model. Each list containing, at index i, the cost of encoding the
    /// symbol i.
    pub table: Vec<Vec<usize>>,

    /// A table containing the frame size for each model, intended as log(M).
    frame_sizes: Vec<usize>,
}

impl <const FIDELITY: usize, const RADIX: usize> ANSymbolTable<FIDELITY, RADIX> {

    const FOLDING_THRESHOLD: u16 = 1 << (FIDELITY + RADIX - 1);
    const FOLDING_OFFSET: u16 = (1 << (FIDELITY - 1)) * ((1 << RADIX) - 1);

    pub fn new(symbol_freqs: Vec<Vec<Freq>>, frame_sizes: Vec<usize>) -> Self {
        let mut table = Self::initialize_with_binary_cost(symbol_freqs.len()).table;

        table
            .iter_mut()
            .enumerate()
            .for_each(|(model_index, current_table)| {
                (0..current_table.len())
                    .into_iter()
                    .for_each(|symbol| {
                        let symbol_freq = match symbol_freqs[model_index].get(symbol) {
                            Some(freq) => match *freq {
                                0 => 1, // we can have 0 frequencies for symbols that exists due to bigger ones
                                _ => *freq,
                            },
                            None => 1,
                        };

                        current_table[symbol] = Self::calculate_symbol_cost(
                            symbol as Symbol,
                            symbol_freq,
                            frame_sizes[model_index]
                        );
                    });
            });

        Self {
            table,
            frame_sizes
        }
    }

    /// Creates a a table of `model_number` lists, each containing, at the index i, the cost of the symbol i calculated
    /// as follow: `cost = log2(freq) + bytes_to_unfold * RADIX`.
    pub fn initialize_with_binary_cost(model_number: usize) -> Self {
        let max_folded_sym = folding_without_streaming_out(MAX_RAW_SYMBOL, RADIX, FIDELITY);

        let table = (0..model_number)
            .into_iter()
            .map(|model_index| {
                (0..max_folded_sym + 1)
                    .into_iter()
                    .map(|symbol| {
                        Self::get_binary_cost(symbol)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        Self {
            table,
            frame_sizes: Vec::new(),
        }
    }

    fn calculate_symbol_cost(symbol: Symbol, symbol_freq: Freq, frame_size: usize) -> usize {
        // we shouldn't have a symbol with frequency 0 since we want to have the cost for each symbol
        debug_assert!(symbol_freq != 0);

        let bytes_to_unfold = match symbol < Self::FOLDING_THRESHOLD {
            true => 0_u16,
            false => (symbol - Self::FOLDING_THRESHOLD) / Self::FOLDING_OFFSET + 1u16,
        };

        let probability = symbol_freq as f64 / (1u64 << frame_size) as f64;
        let inverse = 1.0 / probability;
        let shifted = (inverse * ((1 << 16) as f64)).round() as usize;

        shifted + ((bytes_to_unfold as usize * RADIX) * (1 << 16))
    }

    fn get_binary_cost(symbol: Symbol) -> usize {
        let bytes_to_unfold = match symbol < Self::FOLDING_THRESHOLD {
            true => 0_usize,
            false => ((symbol - Self::FOLDING_THRESHOLD) / Self::FOLDING_OFFSET) as usize + 1_usize,
        };

        (symbol.checked_ilog2().unwrap_or(0) as usize + 1)  + bytes_to_unfold * RADIX
    }
}

#[derive(Clone)]
pub struct EntropyMockWriter<const FIDELITY: usize, const RADIX: usize> {
    costs_table: ANSymbolTable<FIDELITY, RADIX>,
}

impl <const FIDELITY: usize, const RADIX: usize> EntropyMockWriter<FIDELITY, RADIX> {
    const FOLDING_THRESHOLD: u64 = 1 << (FIDELITY + RADIX - 1);
}

impl <const FIDELITY: usize, const RADIX: usize> MockWriter<FIDELITY, RADIX> for EntropyMockWriter<FIDELITY, RADIX> {

    fn build(costs_table: ANSymbolTable<FIDELITY, RADIX>) -> Self {
        Self {
            costs_table,
        }
    }
}

impl <const FIDELITY: usize, const RADIX: usize> BVGraphCodesWriter for EntropyMockWriter<FIDELITY, RADIX> {
    type Error = Infallible;

    type MockWriter = Self; // it's essentially a marker

    fn mock(&self) -> Self::MockWriter {
         Self {costs_table: ANSymbolTable::initialize_with_binary_cost(0)} // thus we can return a fake one
    }

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < Self::FOLDING_THRESHOLD {
            return Ok(self.costs_table.table[Component::Outdegree as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.costs_table.table[Component::Outdegree as usize][folded_sym as usize])
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < Self::FOLDING_THRESHOLD {
            return Ok(self.costs_table.table[Component::ReferenceOffset as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.costs_table.table[Component::ReferenceOffset as usize][folded_sym as usize])
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < Self::FOLDING_THRESHOLD {
            return Ok(self.costs_table.table[Component::BlockCount as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.costs_table.table[Component::BlockCount as usize][folded_sym as usize])
    }

    fn write_blocks(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < Self::FOLDING_THRESHOLD {
            return Ok(self.costs_table.table[Component::Blocks as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.costs_table.table[Component::Blocks as usize][folded_sym as usize])
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < Self::FOLDING_THRESHOLD {
            return Ok(self.costs_table.table[Component::IntervalCount as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.costs_table.table[Component::IntervalCount as usize][folded_sym as usize])
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < Self::FOLDING_THRESHOLD {
            return Ok(self.costs_table.table[Component::IntervalStart as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.costs_table.table[Component::IntervalStart as usize][folded_sym as usize])
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < Self::FOLDING_THRESHOLD {
            return Ok(self.costs_table.table[Component::IntervalLen as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.costs_table.table[Component::IntervalLen as usize][folded_sym as usize])
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < Self::FOLDING_THRESHOLD {
            return Ok(self.costs_table.table[Component::FirstResidual as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.costs_table.table[Component::FirstResidual as usize][folded_sym as usize])
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < Self::FOLDING_THRESHOLD {
            return Ok(self.costs_table.table[Component::Residual as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.costs_table.table[Component::Residual as usize][folded_sym as usize])
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}


pub struct Log2MockWriter<const FIDELITY: usize, const RADIX: usize> {
    costs_table: ANSymbolTable<FIDELITY, RADIX>,
}

impl <const FIDELITY: usize, const RADIX: usize> Log2MockWriter<FIDELITY, RADIX> {
    const FOLDING_THRESHOLD: u64 = 1 << (FIDELITY + RADIX - 1);
}

impl <const FIDELITY: usize, const RADIX: usize> MockWriter<FIDELITY, RADIX> for Log2MockWriter<FIDELITY, RADIX> {

    fn build(_costs_table: ANSymbolTable<FIDELITY, RADIX>) -> Self {
        Self {
            costs_table: ANSymbolTable::initialize_with_binary_cost(9)
        }
    }
}

impl <const FIDELITY: usize, const RADIX: usize> BVGraphCodesWriter for Log2MockWriter<FIDELITY, RADIX> {
    type Error = Infallible;

    type MockWriter = Self; // it's essentially a marker

    fn mock(&self) -> Self::MockWriter {
        Log2MockWriter { costs_table: ANSymbolTable::initialize_with_binary_cost(0)} // thus we can return a fake one
    }

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < Self::FOLDING_THRESHOLD {
            return Ok(self.costs_table.table[Component::Outdegree as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.costs_table.table[Component::Outdegree as usize][folded_sym as usize])
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < Self::FOLDING_THRESHOLD {
            return Ok(self.costs_table.table[Component::ReferenceOffset as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.costs_table.table[Component::ReferenceOffset as usize][folded_sym as usize])
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < Self::FOLDING_THRESHOLD {
            return Ok(self.costs_table.table[Component::BlockCount as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.costs_table.table[Component::BlockCount as usize][folded_sym as usize])
    }

    fn write_blocks(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < Self::FOLDING_THRESHOLD {
            return Ok(self.costs_table.table[Component::Blocks as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.costs_table.table[Component::Blocks as usize][folded_sym as usize])
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < Self::FOLDING_THRESHOLD {
            return Ok(self.costs_table.table[Component::IntervalCount as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.costs_table.table[Component::IntervalCount as usize][folded_sym as usize])
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < Self::FOLDING_THRESHOLD {
            return Ok(self.costs_table.table[Component::IntervalStart as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.costs_table.table[Component::IntervalStart as usize][folded_sym as usize])
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < Self::FOLDING_THRESHOLD {
            return Ok(self.costs_table.table[Component::IntervalLen as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.costs_table.table[Component::IntervalLen as usize][folded_sym as usize])
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < Self::FOLDING_THRESHOLD {
            return Ok(self.costs_table.table[Component::FirstResidual as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.costs_table.table[Component::FirstResidual as usize][folded_sym as usize])
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < Self::FOLDING_THRESHOLD {
            return Ok(self.costs_table.table[Component::Residual as usize][value as usize]);
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.costs_table.table[Component::Residual as usize][folded_sym as usize])
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}