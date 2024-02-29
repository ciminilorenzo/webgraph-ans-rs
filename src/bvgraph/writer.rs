use std::convert::Infallible;
use tempfile::{Builder, NamedTempFile};
use webgraph::graphs::{Encoder, MeasurableEncoder};

use crate::ans::encoder::ANSEncoder;
use crate::ans::model4encoder::ANSModel4Encoder;
use crate::ans::model4encoder_builder::ANSModel4EncoderBuilder;
use crate::ans::{ANSCompressorPhase, Prelude};
use crate::bvgraph::mock_writers::EntropyEstimator;
use crate::bvgraph::BVGraphComponent;
use crate::utils::rev::RevBuffer;

/// An [`Encoder`] that writes to an [`ANSModel4EncoderBuilder`]. to collect data for each
///
/// Data for each [component](BVGraphComponent) is pushed into the [`ANSModel4EncoderBuilder`]. The [`ANSModel4Encoder`]
/// is then built from the collected data.
pub struct BVGraphModelBuilder<MW: Encoder> {
    model_builder: ANSModel4EncoderBuilder,

    /// The type of the mock writer used by this builder. It may either be a `Log2Estimator` or an `EntropyEstimator`.
    mock: MW,
}

impl<MW: Encoder> BVGraphModelBuilder<MW> {
    pub fn new(mock: MW) -> Self {
        Self {
            model_builder: ANSModel4EncoderBuilder::default(),
            mock,
        }
    }

    /// Build an [`ANSModel4Encoder`] from the symbols written to this
    /// [`BVGraphModelBuilder`].
    pub fn build(self) -> ANSModel4Encoder {
        self.model_builder.build()
    }
}

impl<MW: Encoder> MeasurableEncoder for BVGraphModelBuilder<MW> {
    type Estimator<'a> = &'a mut MW where Self: 'a;

    fn estimator(&mut self) -> Self::Estimator<'_> {
        &mut self.mock
    }
}

impl<MW: Encoder> Encoder for BVGraphModelBuilder<MW> {
    type Error = Infallible;

    fn start_node(_node: usize) -> Result<(), Self::Error> {
        Ok(())
    }

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, BVGraphComponent::Outdegree);
        Ok(self.mock.write_outdegree(value).unwrap())
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, BVGraphComponent::ReferenceOffset);
        Ok(self.mock.write_reference_offset(value).unwrap())
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, BVGraphComponent::BlockCount);
        Ok(self.mock.write_block_count(value).unwrap())
    }

    fn write_block(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, BVGraphComponent::Blocks);
        Ok(self.mock.write_block(value).unwrap())
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, BVGraphComponent::IntervalCount);
        Ok(self.mock.write_interval_count(value).unwrap())
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, BVGraphComponent::IntervalStart);
        Ok(self.mock.write_interval_start(value).unwrap())
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, BVGraphComponent::IntervalLen);
        Ok(self.mock.write_interval_len(value).unwrap())
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, BVGraphComponent::FirstResidual);
        Ok(self.mock.write_first_residual(value).unwrap())
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, BVGraphComponent::Residual);
        Ok(self.mock.write_residual(value).unwrap())
    }

    fn flush(&mut self) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn end_node(_node: usize) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// An [`Encoder`] that writes to an [`ANSEncoder`].
pub struct ANSBVGraphMeasurableEncoder {
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

impl ANSBVGraphMeasurableEncoder {
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
    pub fn into_inner(self) -> (Prelude, Vec<ANSCompressorPhase>) {
        let compression_results = self.encoder.get_compression_results();

        (
            Prelude::new(
                compression_results.0,
                compression_results.1,
                compression_results.2,
                self.number_of_nodes,
                self.number_of_arcs,
                self.compression_window,
                self.min_interval_length,
            ),
            self.phases,
        )
    }
}

impl MeasurableEncoder for ANSBVGraphMeasurableEncoder {
    type Estimator<'a> = &'a mut EntropyEstimator where Self: 'a;

    fn estimator(&mut self) -> Self::Estimator<'_> {
        &mut self.estimator
    }
}

/// Note that every Encoder's function write as model the component's index - 8 in order to have
/// the most frequent components encoded with the smallest number of bits. Reversing the order of
/// the components' indexes a good way to represent the expected frequency of the components.
impl Encoder for ANSBVGraphMeasurableEncoder {
    type Error = Infallible;

    fn start_node(_node: usize) -> Result<(), Self::Error> {
        Ok(())
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
        // let's reverse the phases so that the first phase is associated to the last node encoded
        self.phases.reverse();
        Ok(0)
    }

    fn end_node(_node: usize) -> Result<(), Self::Error> {
        Ok(())
    }
}
