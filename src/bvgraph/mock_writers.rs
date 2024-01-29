use std::convert::Infallible;
use webgraph::prelude::BVGraphCodesWriter;

use crate::bvgraph::BVGraphComponent;
use crate::{Freq, MAX_RAW_SYMBOL, Symbol};
use crate::multi_model_ans::model4encoder::ANSModel4Encoder;
use crate::utils::ans_utilities::{fold_without_streaming_out};


#[derive(Clone)]
pub struct ANSymbolTable {
    /// A table containing the cost of each symbol contained in every [`BVGraphComponent`](component).
    table: Vec<Vec<usize>>,

    /// The fidelity and radix values used by each [component](BVGraphComponent).
    component_args: [(usize, usize); 9],

    /// Represent the threshold starting from which a symbol has to be folded, one for each [component](BVGraphComponent).
    folding_thresholds: [u64; 9],

    /// The offset used to fold the symbols, one for each [component](BVGraphComponent).
    folding_offsets: [u64; 9],
}

impl ANSymbolTable {
    /// Returns a new ANSymbolTable, that is a table containing for each component, the cost of every symbol calculated
    /// as follow:
    /// ```text
    /// C(x) = ((1 / p) * 2^16) + ((bytes_to_unfold * radix) * 2^16)
    /// ```
    pub fn new(model: &ANSModel4Encoder, component_args: [(usize, usize); 9]) -> Self {
        let mut table = Self::initialize_with_binary_cost(component_args);

        table.table
            .iter_mut()
            .enumerate()
            .for_each(|(component, current_table)| {
                (0..current_table.len())
                    .for_each(|symbol| {
                        let symbol_freq = match model.tables[component].table.get(symbol) {
                            Some(entry) => match entry.freq {
                                0 => 1, // we can have 0 frequencies for symbols that exists due to bigger ones
                                _ => entry.freq,
                            },
                            None => 1,
                        };

                        current_table[symbol] = Self::calculate_symbol_cost(
                            symbol as Symbol,
                            symbol_freq,
                            model.get_log2_frame_size(BVGraphComponent::from(component)),
                            model.get_folding_offset(BVGraphComponent::from(component)) as u16,
                            model.get_folding_threshold(BVGraphComponent::from(component)) as u16,
                            model.get_radix(BVGraphComponent::from(component)),
                        );
                    });
            });
        table
    }

    fn calculate_symbol_cost(
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

    /// Creates a a table of [`BVGraphComponent::COMPONENTS`] lists, each containing, at the index i, the cost of the
    /// symbol i calculate as follow:
    /// ```text
    ///  C(x) = (floor(log2(x)) + 1) + (bytes_to_unfold * radix)
    /// ```
    pub fn initialize_with_binary_cost(component_args: [(usize, usize); 9]) -> Self {
        let mut folding_thresholds = [0u64; 9];
        let mut folding_offsets = [0u64; 9];

        let table = component_args
            .iter()
            .enumerate()
            .map(|(index, (fidelity, radix))| {
                let max_folded_sym = fold_without_streaming_out(MAX_RAW_SYMBOL, *radix, *fidelity);
                let folding_threshold = (1 << (*fidelity + *radix - 1)) as u64;
                let folding_offset = ((1u64 << *radix) - 1) * (1 << (*fidelity - 1));
                folding_thresholds[index] = folding_threshold;
                folding_offsets[index] = folding_offset;

                (0..max_folded_sym + 1)
                    .map(|symbol| Self::calculate_binary_cost(symbol, folding_threshold as u16, folding_offset as u16, *radix))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        Self {
            table,
            component_args,
            folding_thresholds,
            folding_offsets,
        }
    }

    fn calculate_binary_cost(symbol: Symbol, folding_threshold: u16, folding_offset: u16, radix: usize) -> usize {
        let bytes_to_unfold = match symbol < folding_threshold {
            true => 0_usize,
            false => ((symbol - folding_threshold) / folding_offset) as usize + 1_usize,
        };

        (symbol.checked_ilog2().unwrap_or(0) as usize + 1)  + bytes_to_unfold * radix
    }

    #[inline(always)]
    pub fn get_symbol_cost(&self, symbol: usize, component: BVGraphComponent) -> usize {
        self.table[component as usize][symbol]
    }

    #[inline(always)]
    pub fn get_component_threshold(&self, component: BVGraphComponent) -> u64 {
        self.folding_thresholds[component as usize]
    }

    #[inline(always)]
    pub fn get_component_radix(&self, component: BVGraphComponent) -> usize {
        self.component_args[component as usize].1
    }

    #[inline(always)]
    pub fn get_component_fidelity(&self, component: BVGraphComponent) -> usize {
        self.component_args[component as usize].0
    }
}


/// A trait for those mock writers that can be buildable.
pub trait MockWriter {
    /// Builds a mock writer from a given costs table.
    fn build(costs_table: ANSymbolTable) -> Self;
}


#[derive(Clone)]
pub struct EntropyMockWriter {
    /// The costs table used to encode the symbols.
    costs_table: ANSymbolTable,
}

impl MockWriter for EntropyMockWriter {

    fn build(costs_table: ANSymbolTable) -> Self {
        Self { costs_table }
    }
}

impl BVGraphCodesWriter for EntropyMockWriter {
    type Error = Infallible;

    type MockWriter = Self; // it's essentially a marker

    fn mock(&self) -> Self::MockWriter {
        Self {
            costs_table: ANSymbolTable::initialize_with_binary_cost( [(10, 10); 9]),
         } // thus we can return a fake one
    }

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.costs_table.folding_thresholds[BVGraphComponent::Outdegree as usize] {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::Outdegree));
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::Outdegree),
            self.costs_table.get_component_fidelity(BVGraphComponent::Outdegree),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::Outdegree))
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.costs_table.folding_thresholds[BVGraphComponent::ReferenceOffset as usize] {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::ReferenceOffset));
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::ReferenceOffset),
            self.costs_table.get_component_fidelity(BVGraphComponent::ReferenceOffset),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::ReferenceOffset))
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.costs_table.folding_thresholds[BVGraphComponent::BlockCount as usize] {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::BlockCount));
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::BlockCount),
            self.costs_table.get_component_fidelity(BVGraphComponent::BlockCount),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::BlockCount))
    }

    fn write_blocks(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.costs_table.folding_thresholds[BVGraphComponent::Blocks as usize] {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::Blocks));
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::Blocks),
            self.costs_table.get_component_fidelity(BVGraphComponent::Blocks),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::Blocks))
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.costs_table.folding_thresholds[BVGraphComponent::IntervalCount as usize] {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::IntervalCount));
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::IntervalCount),
            self.costs_table.get_component_fidelity(BVGraphComponent::IntervalCount),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::IntervalCount))
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.costs_table.folding_thresholds[BVGraphComponent::IntervalStart as usize] {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::IntervalStart));
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::IntervalStart),
            self.costs_table.get_component_fidelity(BVGraphComponent::IntervalStart),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::IntervalStart))
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.costs_table.folding_thresholds[BVGraphComponent::IntervalLen as usize] {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::IntervalLen));
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::IntervalLen),
            self.costs_table.get_component_fidelity(BVGraphComponent::IntervalLen),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::IntervalLen))
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.costs_table.folding_thresholds[BVGraphComponent::FirstResidual as usize] {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::FirstResidual));
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::FirstResidual),
            self.costs_table.get_component_fidelity(BVGraphComponent::FirstResidual),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::FirstResidual))
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.costs_table.folding_thresholds[BVGraphComponent::Residual as usize] {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::Residual));
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::Residual),
            self.costs_table.get_component_fidelity(BVGraphComponent::Residual),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::Residual))
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}


pub struct Log2MockWriter {
    /// The costs table used to encode the symbols.
    costs_table: ANSymbolTable,
}

impl MockWriter for Log2MockWriter {

    fn build(costs_table: ANSymbolTable) -> Self {
        Self { costs_table }
    }
}

impl BVGraphCodesWriter for Log2MockWriter {
    type Error = Infallible;

    type MockWriter = Self; // it's essentially a marker

    fn mock(&self) -> Self::MockWriter {
        Log2MockWriter {
            costs_table: ANSymbolTable::initialize_with_binary_cost( [(10, 10); 9]),
        } // thus we can return a fake one
    }

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.costs_table.get_component_threshold(BVGraphComponent::Outdegree) {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::Outdegree));
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::Outdegree),
            self.costs_table.get_component_fidelity(BVGraphComponent::Outdegree),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::Outdegree))
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.costs_table.get_component_threshold(BVGraphComponent::ReferenceOffset) {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::ReferenceOffset));
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::ReferenceOffset),
            self.costs_table.get_component_fidelity(BVGraphComponent::ReferenceOffset),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::ReferenceOffset))
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.costs_table.get_component_threshold(BVGraphComponent::BlockCount) {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::BlockCount));
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::BlockCount),
            self.costs_table.get_component_fidelity(BVGraphComponent::BlockCount),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::BlockCount))
    }

    fn write_blocks(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.costs_table.get_component_threshold(BVGraphComponent::Blocks) {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::Blocks));
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::Blocks),
            self.costs_table.get_component_fidelity(BVGraphComponent::Blocks),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::Blocks))
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.costs_table.get_component_threshold(BVGraphComponent::IntervalCount) {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::IntervalCount));
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::IntervalCount),
            self.costs_table.get_component_fidelity(BVGraphComponent::IntervalCount),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::IntervalCount))
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.costs_table.get_component_threshold(BVGraphComponent::IntervalStart) {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::IntervalStart));
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::IntervalStart),
            self.costs_table.get_component_fidelity(BVGraphComponent::IntervalStart),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::IntervalStart))
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.costs_table.get_component_threshold(BVGraphComponent::IntervalLen) {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::IntervalLen));
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::IntervalLen),
            self.costs_table.get_component_fidelity(BVGraphComponent::IntervalLen),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::IntervalLen))
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.costs_table.get_component_threshold(BVGraphComponent::FirstResidual) {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::FirstResidual));
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::FirstResidual),
            self.costs_table.get_component_fidelity(BVGraphComponent::FirstResidual),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::FirstResidual))
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        if value < self.costs_table.get_component_threshold(BVGraphComponent::Residual) {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::Residual));
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::Residual),
            self.costs_table.get_component_fidelity(BVGraphComponent::Residual),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::Residual))
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}