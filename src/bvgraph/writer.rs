use epserde::prelude::*;
use std::{convert::Infallible, path::Path};
use sucds::serial;
use webgraph::graph::bvgraph::BVGraphCodesWriter;

use crate::{
    multi_model_ans::{
        encoder::ANSEncoder, model4encoder::AnsModel4Encoder,
        model4encoder_builder::AnsModel4EncoderBuilder,
    },
    traits::folding::Fold,
};

/// An enumeration of the components getting a different model in the Rust
/// implementation of the BV format.
pub enum Component {
    Outdegree,
    ReferenceOffset,
    BlockCount,
    Blocks,
    IntervalCount,
    IntervalStart,
    IntervalLen,
    FirstResidual,
    Residual,
}

/// A mock writer that returns `⌊log₂(x)⌋` as the number of bits written
/// encoding `x`.
pub struct Log2MockWriter {}

impl BVGraphCodesWriter for Log2MockWriter {
    type Error = Infallible;

    type MockWriter = Self;

    fn mock(&self) -> Self::MockWriter {
        Log2MockWriter {}
    }

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        len(value)
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        len(value)
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        len(value)
    }

    fn write_blocks(&mut self, value: u64) -> Result<usize, Self::Error> {
        len(value)
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        len(value)
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        len(value)
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        len(value)
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        len(value)
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        len(value)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// A [`BVGraphCodesWriter`] that builds an [`AnsModel4Encoder`] using the
/// symbols written to it.
///
/// Note that a [`BVGraphCodesWriter`] needs a mock writer to measure code
/// lengths. We use a [`Log2MockWriter`] that returns `⌊log₂(x)⌋` as the number
/// of bits written encoding `x`.
pub struct BVGraphModelBuilder<const FIDELITY: usize, const RADIX: usize> {
    path: Box<Path>, // unused
    model_builder: AnsModel4EncoderBuilder<FIDELITY, RADIX>,
}

impl<const FIDELITY: usize, const RADIX: usize> BVGraphModelBuilder<FIDELITY, RADIX> {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_owned().into_boxed_path(),
            model_builder: AnsModel4EncoderBuilder::<FIDELITY, RADIX>::new(9),
        }
    }

    /// Build an [`AnsModel4Encoder`] from the symbols written to this
    /// [`BVGraphModelBuilder`].
    pub fn build(self) -> AnsModel4Encoder {
        self.model_builder.build()
    }
}

impl<const FIDELITY: usize, const RADIX: usize> BVGraphCodesWriter
    for BVGraphModelBuilder<FIDELITY, RADIX>
{
    type Error = Infallible;

    type MockWriter = Log2MockWriter;

    fn mock(&self) -> Self::MockWriter {
        Log2MockWriter {}
    }

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, Component::Outdegree as usize);
        len(value)
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, Component::ReferenceOffset as usize);
        len(value)
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, Component::BlockCount as usize);
        len(value)
    }

    fn write_blocks(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, Component::Blocks as usize);
        len(value)
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, Component::IntervalCount as usize);
        len(value)
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, Component::IntervalStart as usize);
        len(value)
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, Component::IntervalLen as usize);
        len(value)
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, Component::FirstResidual as usize);
        len(value)
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, Component::Residual as usize);
        len(value)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// A [`BVGraphCodesWriter`] that writes to an [`ANSEncoder`].
///
/// Data is gathered in a number of buffers, one for each [component](`Component`).
/// At the next node (i.e. when `write_outdegree` is called again), the buffers
/// are emptied in reverse order.
pub struct BVGraphWriter<
    const FIDELITY: usize,
    const RADIX: usize,
    F: Fold<RADIX> + Default + Clone,
> {
    data: [Vec<usize>; 9],
    curr_node: usize,
    encoder: ANSEncoder<FIDELITY, RADIX, F>,
}

fn len(value: u64) -> Result<usize, Infallible> {
    Ok((value + 2).ilog2() as usize)
}

impl<const FIDELITY: usize> BVGraphWriter<FIDELITY, 8, Vec<u8>> {
    pub fn new(model: AnsModel4Encoder) -> Self {
        Self {
            curr_node: usize::MAX,
            data: [
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ],
            encoder: ANSEncoder::<FIDELITY, 8, Vec<u8>>::with_parameters(model, Vec::<u8>::new()),
        }
    }

    /// Consume self and return the encoder.
    pub fn into_inner(self) -> ANSEncoder<FIDELITY, 8, Vec<u8>> {
        self.encoder
    }
}

impl<const FIDELITY: usize, const RADIX: usize, F: Fold<RADIX> + Default + Clone> BVGraphCodesWriter
    for BVGraphWriter<FIDELITY, RADIX, F>
{
    type Error = Infallible;

    type MockWriter = Log2MockWriter;

    fn mock(&self) -> Self::MockWriter {
        Log2MockWriter {}
    }

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        if self.curr_node != usize::MAX {
            for (i, v) in self.data.iter().enumerate().rev() {
                for &x in v.iter().rev() {
                    self.encoder.encode(x as u64, i);
                }
            }
        }

        // Increase and cleanup
        self.curr_node = self.curr_node.wrapping_add(1);
        for v in &mut self.data {
            v.clear();
        }

        self.data[Component::Outdegree as usize].push(value as usize);
        len(value)
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[Component::ReferenceOffset as usize].push(value as usize);
        len(value)
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[Component::BlockCount as usize].push(value as usize);
        len(value)
    }

    fn write_blocks(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[Component::Blocks as usize].push(value as usize);
        len(value)
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[Component::IntervalCount as usize].push(value as usize);
        len(value)
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[Component::IntervalStart as usize].push(value as usize);
        len(value)
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[Component::IntervalLen as usize].push(value as usize);
        len(value)
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[Component::FirstResidual as usize].push(value as usize);
        len(value)
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[Component::Residual as usize].push(value as usize);
        len(value)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // Dump last node
        for (i, v) in self.data.iter().enumerate().rev() {
            for &x in v.iter().rev() {
                self.encoder.encode(x as u64, i);
            }
        }
        Ok(())
    }
}
