use std::convert::Infallible;
use webgraph::graphs::{Encoder, MeasurableEncoder};

use crate::bvgraph::mock_writers::{ANSymbolTable, EntropyMockWriter, MockWriter};
use crate::bvgraph::BVGraphComponent;
use crate::multi_model_ans::encoder::ANSEncoder;
use crate::multi_model_ans::model4encoder::ANSModel4Encoder;
use crate::multi_model_ans::model4encoder_builder::ANSModel4EncoderBuilder;
use crate::multi_model_ans::ANSCompressorPhase;

pub struct BVGraphModelBuilder<MW: Encoder + MockWriter> {
    model_builder: ANSModel4EncoderBuilder,

    /// The type of the mock writer.
    mock: MW,
}

impl<MW: Encoder + MockWriter> BVGraphModelBuilder<MW> {
    pub fn new(mock: MW) -> Self {
        Self {
            model_builder: ANSModel4EncoderBuilder::new(),
            mock,
        }
    }

    /// Build an [`ANSModel4Encoder`] from the symbols written to this
    /// [`BVGraphModelBuilder`].
    pub fn build(self) -> ANSModel4Encoder {
        self.model_builder.build()
    }
}

impl<MW: Encoder + MockWriter> Encoder for BVGraphModelBuilder<MW> {
    type Error = Infallible;

    fn start_node(node: usize) -> Result<(), Self::Error> {
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

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn end_node(node: usize) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<MW: Encoder + MockWriter> MeasurableEncoder for BVGraphModelBuilder<MW> {
    type Estimator = MW;

    fn estimator(&self) -> Self::Estimator {
        self.mock.clone()
    }
}

/// A [`BVGraphCodesWriter`] that writes to an [`ANSEncoder`].
///
/// Data is gathered in a number of buffers, one for each [component](`Component`).
/// At the next node (i.e. when `write_outdegree` is called again), the buffers
/// are emptied in reverse order.
pub struct BVGraphWriter {
    /// The container containing the buffers (one for each [component](`Component`)) where symbols are collected.
    data: [Vec<usize>; 9],

    /// The index of the node the encoder is currently encoding.
    curr_node: usize,

    /// The encoder used by this writer to encode symbols.
    encoder: ANSEncoder,

    /// A buffer containing a [`ANSCompressorPhase`], one for each node.
    phases: Vec<ANSCompressorPhase>,

    mock_writer: EntropyMockWriter,
}

impl BVGraphWriter {
    pub fn new(model: ANSModel4Encoder, costs_table: ANSymbolTable) -> Self {
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
            mock_writer: EntropyMockWriter::build(costs_table),
            encoder: ANSEncoder::new(model),
            phases: Vec::new(),
        }
    }

    /// Consume self and return the encoder.
    pub fn into_inner(self) -> (ANSEncoder, Vec<ANSCompressorPhase>) {
        (self.encoder, self.phases)
    }
}

impl Encoder for BVGraphWriter {
    type Error = Infallible;

    fn start_node(node: usize) -> Result<(), Self::Error> {
        Ok(())
    }

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        if self.curr_node != usize::MAX {
            for (component, symbols) in self.data
                [BVGraphComponent::FirstResidual as usize..=BVGraphComponent::Residual as usize]
                .iter()
                .enumerate()
                .rev()
            {
                for &symbol in symbols.iter().rev() {
                    self.encoder.encode(
                        symbol as u64,
                        BVGraphComponent::from(
                            component + BVGraphComponent::FirstResidual as usize,
                        ),
                    );
                }
            }

            debug_assert_eq!(
                self.data[BVGraphComponent::IntervalStart as usize].len(),
                self.data[BVGraphComponent::IntervalLen as usize].len()
            );

            for i in (0..self.data[BVGraphComponent::IntervalStart as usize].len()).rev() {
                self.encoder.encode(
                    self.data[BVGraphComponent::IntervalLen as usize][i] as u64,
                    BVGraphComponent::IntervalLen,
                );
                self.encoder.encode(
                    self.data[BVGraphComponent::IntervalStart as usize][i] as u64,
                    BVGraphComponent::IntervalStart,
                );
            }

            for (component, symbols) in self.data
                [BVGraphComponent::Outdegree as usize..=BVGraphComponent::IntervalCount as usize]
                .iter()
                .enumerate()
                .rev()
            {
                for &symbol in symbols.iter().rev() {
                    self.encoder
                        .encode(symbol as u64, BVGraphComponent::from(component));
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

        self.data[BVGraphComponent::Outdegree as usize].push(value as usize);
        self.mock_writer.write_outdegree(value)
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[BVGraphComponent::ReferenceOffset as usize].push(value as usize);
        self.mock_writer.write_reference_offset(value)
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[BVGraphComponent::BlockCount as usize].push(value as usize);
        self.mock_writer.write_block_count(value)
    }

    fn write_block(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[BVGraphComponent::Blocks as usize].push(value as usize);
        self.mock_writer.write_block(value)
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[BVGraphComponent::IntervalCount as usize].push(value as usize);
        self.mock_writer.write_interval_count(value)
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[BVGraphComponent::IntervalStart as usize].push(value as usize);
        self.mock_writer.write_interval_start(value)
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[BVGraphComponent::IntervalLen as usize].push(value as usize);
        self.mock_writer.write_interval_len(value)
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[BVGraphComponent::FirstResidual as usize].push(value as usize);
        self.mock_writer.write_first_residual(value)
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[BVGraphComponent::Residual as usize].push(value as usize);
        self.mock_writer.write_residual(value)
    }

    // Dump last node
    fn flush(&mut self) -> Result<(), Self::Error> {
        for (component, symbols) in self.data.iter().enumerate().rev() {
            for &symbol in symbols.iter().rev() {
                self.encoder
                    .encode(symbol as u64, BVGraphComponent::from(component));
            }
        }
        self.phases
            .push(self.encoder.get_current_compressor_phase());
        Ok(())
    }

    fn end_node(node: usize) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl MeasurableEncoder for BVGraphWriter {
    type Estimator = EntropyMockWriter;

    fn estimator(&self) -> Self::Estimator {
        self.mock_writer.clone()
    }
}
