use std::convert::Infallible;

use webgraph::graphs::Encoder;

use crate::ans::model4encoder::ANSModel4Encoder;
use crate::bvgraph::BVGraphComponent;
use crate::utils::ans_utilities::fold_without_streaming_out;
use crate::{Freq, Symbol, MAX_RAW_SYMBOL};

#[derive(Clone)]
pub struct EntropyEstimator {
    /// A table containing the cost of each symbol contained in every [`BVGraphComponent`](component).
    table: Vec<Vec<usize>>,

    /// The fidelity and radix values used by each [component](BVGraphComponent).
    component_args: Vec<(usize, usize)>,

    /// Represent the threshold starting from which a symbol has to be folded, one for each [component](BVGraphComponent).
    folding_thresholds: Vec<u64>,

    /// The offset used to fold the symbols, one for each [component](BVGraphComponent).
    folding_offsets: Vec<u64>,
}

impl EntropyEstimator {
    pub fn new(model: &ANSModel4Encoder, component_args: Vec<(usize, usize)>) -> Self {
        let mut folding_thresholds = Vec::new();
        let mut folding_offsets = Vec::new();

        let costs_table = component_args
            .iter()
            .enumerate()
            .map(|(component, (fidelity, radix))| {
                let max_folded_sym = fold_without_streaming_out(MAX_RAW_SYMBOL, *radix, *fidelity);
                let folding_threshold = (1 << (*fidelity + *radix - 1)) as u64;
                let folding_offset = ((1u64 << *radix) - 1) * (1 << (*fidelity - 1));
                folding_thresholds.push(folding_threshold);
                folding_offsets.push(folding_offset);

                (0_usize..(max_folded_sym as usize) + 1)
                    .map(|sym| {
                        #[cfg(feature = "arm")]
                        let sym_freq = match model.component_models[component].table.get(sym) {
                            Some(entry) => match entry.freq {
                                0 => 1, // we can have 0 frequencies for symbols that exists due to bigger ones
                                freq => freq,
                            },
                            None => 1,
                        };

                        #[cfg(not(feature = "arm"))]
                        let sym_freq = match model.component_models[component].table.get(sym) {
                            Some(entry) => {
                                let frame_size =
                                    model.get_log2_frame_size(BVGraphComponent::from(component));
                                let freq =
                                    (-(entry.cmpl_freq as i32) + (1i32 << frame_size)) as u16;
                                match freq {
                                    0 => 1, // we can have 0 frequencies for symbols that exists due to bigger ones
                                    freq => freq,
                                }
                            }
                            None => 1,
                        };

                        Self::calculate_symbol_cost(
                            sym as Symbol,
                            sym_freq,
                            model.get_log2_frame_size(BVGraphComponent::from(component)),
                            model.get_folding_offset(BVGraphComponent::from(component)) as u16,
                            model.get_folding_threshold(BVGraphComponent::from(component)) as u16,
                            model.get_radix(BVGraphComponent::from(component)),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        Self {
            table: costs_table,
            component_args,
            folding_thresholds,
            folding_offsets,
        }
    }

    /// Calculates, and then returns, the cost of the symbol folded with the given parameters, calculated as follows:
    /// ```text
    ///     cost = -log2(probability) * 2^16 + (folds * radix) * 2^16
    /// ```
    fn calculate_symbol_cost(
        sym: Symbol,
        freq: Freq,
        frame_size: usize,
        folding_offset: u16,
        folding_threshold: u16,
        radix: usize,
    ) -> usize {
        // we shouldn't have a symbol with frequency 0 since we want to have the cost for each symbol
        debug_assert!(freq != 0);

        let folds = match sym < folding_threshold {
            true => 0_u16,
            false => (sym - folding_threshold) / folding_offset + 1u16,
        };

        let probability = freq as f64 / (1u64 << frame_size) as f64;
        let shifted = (-probability.log2() * ((1 << 16) as f64)).round() as usize;
        shifted + ((folds as usize * radix) * (1 << 16))
    }

    /// Returns the cost of the symbol in the specific [component](BVGraphComponent).
    fn get_symbol_cost(&self, value: u64, component: BVGraphComponent) -> usize {
        let symbol = match value < self.folding_thresholds[component as usize] {
            true => value,
            false => fold_without_streaming_out(
                value,
                self.component_args[component as usize].1,
                self.component_args[component as usize].0,
            ) as u64,
        };
        self.table[component as usize][symbol as usize]
    }
}

impl Encoder for EntropyEstimator {
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

    fn flush(&mut self) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn end_node(_node: usize) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// An estimator that simply returns the cost of each symbol calculated as the log2 of the value plus 2.
#[derive(Clone, Default)]
pub struct Log2Estimator {}

impl Log2Estimator {
    fn get_symbol_cost(&self, value: u64, _component: BVGraphComponent) -> usize {
        u64::ilog2(value + 2) as usize
    }
}

impl Encoder for Log2Estimator {
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

    fn flush(&mut self) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn end_node(_node: usize) -> Result<(), Self::Error> {
        Ok(())
    }
}
