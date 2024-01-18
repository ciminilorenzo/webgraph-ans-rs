use std::{convert::Infallible};
use std::marker::PhantomData;
use webgraph::graph::bvgraph::BVGraphCodesWriter;

use crate::bvgraph::Component;
use crate::multi_model_ans::encoder::ANSCompressorPhase;
use crate::traits::folding::FoldRead;
use crate::{
    multi_model_ans::{
        encoder::ANSEncoder, model4encoder::ANSModel4Encoder,
        model4encoder_builder::ANSModel4EncoderBuilder,
    },
    traits::folding::FoldWrite,
    FASTER_RADIX,
};
use crate::bvgraph::mock_writers::EntropyMockWriter;
use crate::utils::ans_utilities::get_mock_writer;

fn len(value: u64) -> Result<usize, Infallible> {
    Ok((value + 2).ilog2() as usize)
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

/// A [`BVGraphCodesWriter`] that builds an [`ANSModel4Encoder`] using the
/// symbols written to it.
///
/// Note that a [`BVGraphCodesWriter`] needs a mock writer to measure code
/// lengths. We use a [`Log2MockWriter`] that returns `⌊log₂(x)⌋` as the number
/// of bits written encoding `x`.
pub struct BVGraphModelBuilder<const FIDELITY: usize, const RADIX: usize> {
    model_builder: ANSModel4EncoderBuilder<FIDELITY, RADIX>,
}

impl<const FIDELITY: usize, const RADIX: usize> BVGraphModelBuilder<FIDELITY, RADIX> {
    pub fn new() -> Self {
        Self {
            model_builder: ANSModel4EncoderBuilder::<FIDELITY, RADIX>::new(9),
        }
    }

    /// Build an [`ANSModel4Encoder`] from the symbols written to this
    /// [`BVGraphModelBuilder`].
    pub fn build(self) -> ANSModel4Encoder {
        self.model_builder.build()
    }
}

impl<const FIDELITY: usize, const RADIX: usize> BVGraphCodesWriter for BVGraphModelBuilder<FIDELITY, RADIX> {
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
pub struct BVGraphWriter<const FIDELITY: usize, const RADIX: usize, F>
where
    F: FoldWrite<RADIX> + Default + Clone,
{
    /// The container containing the buffers (one for each [component](`Component`)) where symbols are collected.
    data: [Vec<usize>; 9],

    /// The index of the node the encoder is currently encoding.
    curr_node: usize,

    /// The encoder used by this writer to encode symbols.
    encoder: ANSEncoder<FIDELITY, RADIX, F>,

    /// A buffer containing a [`ANSCompressorPhase`], one for each node.
    phases: Vec<ANSCompressorPhase>,
}

impl<const FIDELITY: usize> BVGraphWriter<FIDELITY, FASTER_RADIX, Vec<u8>> {
    pub fn new(model: ANSModel4Encoder, mock_writer: EntropyMockWriter) -> Self {
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
            encoder: ANSEncoder::<FIDELITY, FASTER_RADIX, Vec<u8>>::with_parameters(
                model,
                Vec::<u8>::new(),
            ),
            phases: Vec::new(),
        }
    }

    /// Consume self and return the encoder.
    pub fn into_inner(self, ) -> (ANSEncoder<FIDELITY, FASTER_RADIX, Vec<u8>>, Vec<ANSCompressorPhase>) {
        (self.encoder, self.phases)
    }
}

impl<const FIDELITY: usize, const RADIX: usize, F> BVGraphCodesWriter for BVGraphWriter<FIDELITY, RADIX, F>
where
    F: FoldWrite<RADIX> + FoldRead<RADIX> + Default + Clone,
{
    type Error = Infallible;

    type MockWriter = EntropyMockWriter;

    fn mock(&self) -> Self::MockWriter {
        get_mock_writer(&self.encoder.model.tables, &self.encoder.model.frame_sizes)
    }

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        if self.curr_node != usize::MAX {
            for (component, symbols) in self.data
                [Component::FirstResidual as usize..=Component::Residual as usize]
                .iter()
                .enumerate()
                .rev()
            {
                for &symbol in symbols.iter().rev() {
                    self.encoder
                        .encode(symbol as u64, component + Component::FirstResidual as usize);
                }
            }

            debug_assert_eq!(
                self.data[Component::IntervalStart as usize].len(),
                self.data[Component::IntervalLen as usize].len()
            );

            for i in (0..self.data[Component::IntervalStart as usize].len()).rev() {
                self.encoder.encode(
                    self.data[Component::IntervalLen as usize][i] as u64,
                    Component::IntervalLen as usize,
                );
                self.encoder.encode(
                    self.data[Component::IntervalStart as usize][i] as u64,
                    Component::IntervalStart as usize,
                );
            }

            for (component, symbols) in self.data
                [Component::Outdegree as usize..=Component::IntervalCount as usize]
                .iter()
                .enumerate()
                .rev()
            {
                for &symbol in symbols.iter().rev() {
                    self.encoder.encode(symbol as u64, component);
                }
            }
            // save state of the encoder as soon as it finishes encoding the node
            self.phases
                .push(self.encoder.get_current_compressor_phase());
        }

        // Increase and cleanup
        self.curr_node = self.curr_node.wrapping_add(1);
        for symbols in &mut self.data {
            symbols.clear();
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

    // Dump last node
    fn flush(&mut self) -> Result<(), Self::Error> {
        for (component, symbols) in self.data.iter().enumerate().rev() {
            for &symbol in symbols.iter().rev() {
                self.encoder.encode(symbol as u64, component);
            }
        }
        self.phases
            .push(self.encoder.get_current_compressor_phase());
        Ok(())
    }
}
