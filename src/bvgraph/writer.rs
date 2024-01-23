use std::{convert::Infallible};
use std::marker::PhantomData;
use webgraph::graph::bvgraph::BVGraphCodesWriter;

use crate::bvgraph::BVGraphComponent;

use crate::bvgraph::mock_writers::{ANSymbolTable, EntropyMockWriter, MockWriter};
use crate::multi_model_ans::ANSCompressorPhase;
use crate::multi_model_ans::encoder::ANSEncoder;
use crate::multi_model_ans::model4encoder::ANSModel4Encoder;
use crate::multi_model_ans::model4encoder_builder::ANSModel4EncoderBuilder;
use crate::utils::ans_utilities::folding_without_streaming_out;


pub struct BVGraphModelBuilder<MW: BVGraphCodesWriter + MockWriter> {
    model_builder: ANSModel4EncoderBuilder,

    symbol_costs: ANSymbolTable,

    /// The type of the mock writer.
    _marker: PhantomData<MW>,

    fidelity: usize,

    radix: usize,

    folding_threshold: u64,
}

impl<MW: BVGraphCodesWriter + MockWriter> BVGraphModelBuilder<MW> {

    pub fn new(symbol_costs: ANSymbolTable, fidelity: usize, radix: usize) -> Self {
        Self {
            model_builder: ANSModel4EncoderBuilder::new(fidelity, radix),
            symbol_costs,
            _marker: PhantomData,
            fidelity,
            radix,
            folding_threshold: 1u64 << (fidelity + radix - 1),
        }
    }

    /// Build an [`ANSModel4Encoder`] from the symbols written to this
    /// [`BVGraphModelBuilder`].
    pub fn build(self) -> ANSModel4Encoder {
        self.model_builder.build()
    }
}

impl<MW: BVGraphCodesWriter + MockWriter> BVGraphCodesWriter for BVGraphModelBuilder<MW> {
    type Error = Infallible;

    type MockWriter = MW;

    fn mock(&self) -> Self::MockWriter {
        // !!!!! now it's a clone since it's &Self. Otherwise i would give ownership !!!!!
        MW::build(self.symbol_costs.clone(), self.fidelity, self.radix)
    }

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, BVGraphComponent::Outdegree);

        if value < self.folding_threshold {
            return Ok(self.symbol_costs.table[BVGraphComponent::Outdegree as usize][value as usize])
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.symbol_costs.table[BVGraphComponent::Outdegree as usize][folded_sym as usize])
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, BVGraphComponent::ReferenceOffset);

        if value < self.folding_threshold {
            return Ok(self.symbol_costs.table[BVGraphComponent::ReferenceOffset as usize][value as usize])
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.symbol_costs.table[BVGraphComponent::ReferenceOffset as usize][folded_sym as usize])
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, BVGraphComponent::BlockCount);

        if value < self.folding_threshold {
            return Ok(self.symbol_costs.table[BVGraphComponent::BlockCount as usize][value as usize])
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.symbol_costs.table[BVGraphComponent::BlockCount as usize][folded_sym as usize])
    }

    fn write_blocks(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, BVGraphComponent::Blocks);

        if value < self.folding_threshold {
            return Ok(self.symbol_costs.table[BVGraphComponent::Blocks as usize][value as usize])
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.symbol_costs.table[BVGraphComponent::Blocks as usize][folded_sym as usize])
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, BVGraphComponent::IntervalCount);

        if value < self.folding_threshold {
            return Ok(self.symbol_costs.table[BVGraphComponent::IntervalCount as usize][value as usize])
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.symbol_costs.table[BVGraphComponent::IntervalCount as usize][folded_sym as usize])
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, BVGraphComponent::IntervalStart);

        if value < self.folding_threshold {
            return Ok(self.symbol_costs.table[BVGraphComponent::IntervalStart as usize][value as usize])
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.symbol_costs.table[BVGraphComponent::IntervalStart as usize][folded_sym as usize])
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, BVGraphComponent::IntervalLen);

        if value < self.folding_threshold {
            return Ok(self.symbol_costs.table[BVGraphComponent::IntervalLen as usize][value as usize])
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.symbol_costs.table[BVGraphComponent::IntervalLen as usize][folded_sym as usize])
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, BVGraphComponent::FirstResidual);

        if value < self.folding_threshold {
            return Ok(self.symbol_costs.table[BVGraphComponent::FirstResidual as usize][value as usize])
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.symbol_costs.table[BVGraphComponent::FirstResidual as usize][folded_sym as usize])
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, BVGraphComponent::Residual);

        if value < self.folding_threshold {
            return Ok(self.symbol_costs.table[BVGraphComponent::Residual as usize][value as usize])
        }

        let folded_sym = folding_without_streaming_out(value, self.radix, self.fidelity);
        Ok(self.symbol_costs.table[BVGraphComponent::Residual as usize][folded_sym as usize])
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

    pub fn new(model: ANSModel4Encoder, costs_table: ANSymbolTable, fidelity: usize, radix: usize) -> Self {
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
            mock_writer: EntropyMockWriter::build(costs_table, fidelity, radix),
            encoder: ANSEncoder::new(model, fidelity, radix),
            phases: Vec::new(),
        }
    }

    /// Consume self and return the encoder.
    pub fn into_inner(self) -> (ANSEncoder, Vec<ANSCompressorPhase>) {
        (self.encoder, self.phases)
    }
}

impl BVGraphCodesWriter for BVGraphWriter {
    type Error = Infallible;

    type MockWriter = EntropyMockWriter;

    fn mock(&self) -> Self::MockWriter {
        self.mock_writer.clone() // i must return costs even below so i have to keep an instance of the mock
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
                    self.encoder
                        .encode(symbol as u64, BVGraphComponent::from(component + BVGraphComponent::FirstResidual as usize));
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
                    self.encoder.encode(symbol as u64, BVGraphComponent::from(component));
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

    fn write_blocks(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[BVGraphComponent::Blocks as usize].push(value as usize);
        self.mock_writer.write_blocks(value)
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
                self.encoder.encode(symbol as u64, BVGraphComponent::from(component));
            }
        }
        self.phases
            .push(self.encoder.get_current_compressor_phase());
        Ok(())
    }
}