use std::convert::Infallible;
use std::ops::Neg;
use webgraph::graphs::Encoder;

use crate::bvgraph::BVGraphComponent;
use crate::multi_model_ans::model4encoder::ANSModel4Encoder;
use crate::utils::ans_utilities::fold_without_streaming_out;
use crate::{Freq, Symbol, MAX_RAW_SYMBOL};

#[derive(Clone, Default)]
pub struct ANSymbolTable {
    /// A table containing the cost of each symbol contained in every [`BVGraphComponent`](component).
    table: Vec<Vec<usize>>,

    /// The fidelity and radix values used by each [component](BVGraphComponent).
    component_args: Vec<(usize, usize)>,

    /// Represent the threshold starting from which a symbol has to be folded, one for each [component](BVGraphComponent).
    folding_thresholds: Vec<u64>,

    /// The offset used to fold the symbols, one for each [component](BVGraphComponent).
    folding_offsets: Vec<u64>,
}

impl ANSymbolTable {
    /// Returns a new ANSymbolTable, that is a table containing for each component, the cost of every symbol calculated
    /// as follows:
    /// ```text
    /// C(x) = (-log(p) * 2^16) + ((bytes_to_unfold * radix) * 2^16)
    /// ```
    pub fn new(model: &ANSModel4Encoder, component_args: Vec<(usize, usize)>) -> Self {
        let mut table = Self::initialize(component_args);

        table
            .table
            .iter_mut()
            .enumerate()
            .for_each(|(component, current_table)| {
                (0..current_table.len()).for_each(|symbol| {
                    let symbol_freq = match model.models[component].table.get(symbol) {
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
        folding_threshold: u16,
        radix: usize,
    ) -> usize {
        // we shouldn't have a symbol with frequency 0 since we want to have the cost for each symbol
        debug_assert!(freq != 0);

        let bytes_to_unfold = match symbol < folding_threshold {
            true => 0_u16,
            false => (symbol - folding_threshold) / folding_offset + 1u16,
        };

        let probability = freq as f64 / (1u64 << frame_size) as f64;
        let shifted = (probability.log2().neg() * ((1 << 16) as f64)).round() as usize;

        shifted + ((bytes_to_unfold as usize * radix) * (1 << 16))
    }

    /// Creates a table of [`BVGraphComponent::COMPONENTS`] lists, each containing, at the index i, the cost of the
    /// symbol i calculate as follows:
    /// ```text
    ///  C(x) = (floor(log2(x)) + 1) + (bytes_to_unfold * radix)
    /// ```
    pub fn initialize(component_args: Vec<(usize, usize)>) -> Self {
        let mut folding_thresholds = Vec::new();
        let mut folding_offsets = Vec::new();

        let table = component_args
            .iter()
            .map(|(fidelity, radix)| {
                let max_folded_sym = fold_without_streaming_out(MAX_RAW_SYMBOL, *radix, *fidelity);
                let folding_threshold = (1 << (*fidelity + *radix - 1)) as u64;
                let folding_offset = ((1u64 << *radix) - 1) * (1 << (*fidelity - 1));
                folding_thresholds.push(folding_threshold);
                folding_offsets.push(folding_offset);

                (0..max_folded_sym + 1).map(|_symbol| 1).collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        Self {
            table,
            component_args,
            folding_thresholds,
            folding_offsets,
        }
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
pub trait MockWriter: Clone {
    /// Builds a mock writer from a given costs table.
    fn build(costs_table: ANSymbolTable) -> Self;

    fn get_symbol_cost(&self, value: u64, component: BVGraphComponent) -> usize;
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

    fn get_symbol_cost(&self, value: u64, component: BVGraphComponent) -> usize {
        let symbol = match value < self.costs_table.get_component_threshold(component) {
            true => value as usize,
            false => fold_without_streaming_out(
                value,
                self.costs_table.get_component_radix(component),
                self.costs_table.get_component_fidelity(component),
            ) as usize,
        };
        self.costs_table.get_symbol_cost(symbol, component)
    }
}

impl Encoder for EntropyMockWriter {
    type Error = Infallible;

    fn start_node(_node: usize) -> Result<(), Self::Error> {
        Ok(())
    }

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::Outdegree))
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::ReferenceOffset))
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::BlockCount))
    }

    fn write_block(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::Blocks))
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::IntervalCount))
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::IntervalStart))
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::IntervalLen))
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::FirstResidual))
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::Residual))
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn end_node(_node: usize) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct Log2MockWriter {}

impl MockWriter for Log2MockWriter {
    fn build(_costs_table: ANSymbolTable) -> Self {
        Self {}
    }

    fn get_symbol_cost(&self, value: u64, _component: BVGraphComponent) -> usize {
        u64::checked_ilog(value, 2).unwrap_or(0) as usize + 1
    }
}

impl Encoder for Log2MockWriter {
    type Error = Infallible;

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::Outdegree))
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::ReferenceOffset))
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::BlockCount))
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::IntervalCount))
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::IntervalStart))
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::IntervalLen))
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::FirstResidual))
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::Residual))
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn start_node(_node: usize) -> Result<(), Self::Error> {
        Ok(())
    }

    fn write_block(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::Blocks))
    }

    fn end_node(_node: usize) -> Result<(), Self::Error> {
        Ok(())
    }
}
