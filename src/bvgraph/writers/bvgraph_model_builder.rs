use webgraph::prelude::{Encode, EncodeAndEstimate};
use std::convert::Infallible;
use crate::ans::model4encoder::ANSModel4Encoder;
use crate::ans::model4encoder_builder::ANSModel4EncoderBuilder;
use crate::bvgraph::BVGraphComponent;

/// An [`Encoder`] that writes to an [`ANSModel4EncoderBuilder`]. to collect data for each
///
/// Data for each [component](BVGraphComponent) is pushed into the [`ANSModel4EncoderBuilder`]. The [`ANSModel4Encoder`]
/// is then built from the collected data.
pub struct BVGraphModelBuilder<MW: Encode> {
    model_builder: ANSModel4EncoderBuilder,

    /// The type of the mock writer used by this builder. It may either be a `Log2Estimator` or an `EntropyEstimator`.
    mock: MW,
}

impl<MW: Encode> BVGraphModelBuilder<MW> {
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

impl<MW: Encode> EncodeAndEstimate for BVGraphModelBuilder<MW> {
    type Estimator<'a> = &'a mut MW where Self: 'a;

    fn estimator(&mut self) -> Self::Estimator<'_> {
        &mut self.mock
    }
}

impl<MW: Encode> Encode for BVGraphModelBuilder<MW> {
    type Error = Infallible;

    fn start_node(&mut self, _node: usize) -> Result<usize, Self::Error> {
        Ok(0)
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

    fn end_node(&mut self, _node: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }
}