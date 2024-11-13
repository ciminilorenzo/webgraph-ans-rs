use crate::ans::encoder::ANSEncoder;
use crate::ans::{ANSCompressorPhase, Prelude};
use crate::bvgraph::estimators::entropy_estimator::EntropyEstimator;
use crate::bvgraph::BVGraphComponent;
use crate::utils::rev::RevBuffer;
use crate::ans::models::model4encoder::ANSModel4Encoder;

use std::convert::Infallible;

use tempfile::{Builder, NamedTempFile};

use webgraph::prelude::{Encode, EncodeAndEstimate};

/// An [encoder](`Encode`) that writes to an [`ANSEncoder`].
pub struct ANSBVGraphEncodeAndEstimate {
    /// A buffer containing a [`ANSCompressorPhase`], one for each node.
    phases: Vec<ANSCompressorPhase>,

    /// The estimator used by this writer to estimate cost of the written symbols.
    estimator: EntropyEstimator,

    /// The encoder used by this writer to encode symbols.
    encoder: ANSEncoder,

    /// The buffer where symbols are collected before encoding.
    symbols: RevBuffer<NamedTempFile>,

    /// The buffer where models associated to each collected symbol are collected before encoding.
    models: RevBuffer<NamedTempFile>,

    number_of_nodes: usize,
    number_of_arcs: u64,
    compression_window: usize,
    min_interval_length: usize,
}

impl ANSBVGraphEncodeAndEstimate {
    pub fn new(
        model: ANSModel4Encoder,
        estimator: EntropyEstimator,
        number_of_nodes: usize,
        number_of_arcs: u64,
        compression_window: usize,
        min_interval_length: usize,
    ) -> Self {
        let symbols_file = Builder::new().prefix("symbols").tempfile().unwrap();
        let models_file = Builder::new().prefix("models").tempfile().unwrap();

        Self {
            phases: Vec::new(),
            estimator,
            encoder: ANSEncoder::new(model),
            symbols: RevBuffer::new(symbols_file).unwrap(),
            models: RevBuffer::new(models_file).unwrap(),
            number_of_nodes,
            number_of_arcs,
            compression_window,
            min_interval_length,
        }
    }

    /// Returns the Prelude, containing the compression results of the encoded graph and all the
    /// complementary data needed to decode it, and the list of ANSCompressorPhase, one for each node.
    pub fn into_prelude_phases(self) -> (Prelude, Vec<ANSCompressorPhase>) {
        let compression_results = self.encoder.get_compression_results();

        (
            Prelude {
                tables: compression_results.0,
                stream: compression_results.1,
                state: compression_results.2,
                number_of_nodes: self.number_of_nodes,
                compression_window: self.compression_window,
                min_interval_length: self.min_interval_length,
                number_of_arcs: self.number_of_arcs,
            },
            self.phases,
        )
    }
}

impl EncodeAndEstimate for ANSBVGraphEncodeAndEstimate {
    type Estimator<'a>
        = &'a mut EntropyEstimator
    where
        Self: 'a;

    fn estimator(&mut self) -> Self::Estimator<'_> {
        &mut self.estimator
    }
}

/// Note that every Encoder's function write as model the component's index - 8 in order to have
/// the most frequent components encoded with the smallest number of bits. Reversing the order of
/// the components' indexes a good way to represent the expected frequency of the components.
impl Encode for ANSBVGraphEncodeAndEstimate {
    type Error = Infallible;

    fn start_node(&mut self, _node: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.symbols.push(value);
        self.models.push(8 - BVGraphComponent::Outdegree as u64);
        self.estimator.write_outdegree(value)
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.symbols.push(value);
        self.models
            .push(8 - BVGraphComponent::ReferenceOffset as u64);
        self.estimator.write_reference_offset(value)
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.symbols.push(value);
        self.models.push(8 - BVGraphComponent::BlockCount as u64);
        self.estimator.write_block_count(value)
    }

    fn write_block(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.symbols.push(value);
        self.models.push(8 - BVGraphComponent::Blocks as u64);
        self.estimator.write_block(value)
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.symbols.push(value);
        self.models.push(8 - BVGraphComponent::IntervalCount as u64);
        self.estimator.write_interval_count(value)
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.symbols.push(value);
        self.models.push(8 - BVGraphComponent::IntervalStart as u64);
        self.estimator.write_interval_start(value)
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.symbols.push(value);
        self.models.push(8 - BVGraphComponent::IntervalLen as u64);
        self.estimator.write_interval_len(value)
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.symbols.push(value);
        self.models.push(8 - BVGraphComponent::FirstResidual as u64);
        self.estimator.write_first_residual(value)
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.symbols.push(value);
        self.models.push(8 - BVGraphComponent::Residual as u64);
        self.estimator.write_residual(value)
    }

    // called after having encoded the last symbols of the last node
    fn flush(&mut self) -> Result<usize, Self::Error> {
        let symbols_iter = self.symbols.flush().unwrap();
        let models_iter = self.models.flush().unwrap();

        for (symbol, model) in symbols_iter.into_iter().zip(models_iter.into_iter()) {
            let model = 8 - model as usize;
            self.encoder.encode(symbol, BVGraphComponent::from(model));

            // let's save the phase if we have encoded the outdegree
            if model == BVGraphComponent::Outdegree as usize {
                self.phases
                    .push(self.encoder.get_current_compressor_phase());
            }
        }
        Ok(0)
    }

    fn end_node(&mut self, _node: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }
}
