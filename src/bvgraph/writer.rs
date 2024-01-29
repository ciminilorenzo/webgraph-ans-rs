use std::{convert::Infallible};
use std::marker::PhantomData;
use webgraph::graph::bvgraph::BVGraphCodesWriter;

use crate::bvgraph::BVGraphComponent;

use crate::bvgraph::mock_writers::{ANSymbolTable, EntropyMockWriter, MockWriter};
use crate::multi_model_ans::ANSCompressorPhase;
use crate::multi_model_ans::encoder::ANSEncoder;
use crate::multi_model_ans::model4encoder::ANSModel4Encoder;
use crate::multi_model_ans::model4encoder_builder::ANSModel4EncoderBuilder;
use crate::utils::ans_utilities::fold_without_streaming_out;


pub struct BVGraphModelBuilder<MW: BVGraphCodesWriter + MockWriter> {
    model_builder: ANSModel4EncoderBuilder,

    costs_table: ANSymbolTable,

    /// The type of the mock writer.
    _marker: PhantomData<MW>,
}

impl<MW: BVGraphCodesWriter + MockWriter> BVGraphModelBuilder<MW> {

    pub fn new(symbol_costs: ANSymbolTable, component_args: [(usize, usize); 9]) -> Self {
        Self {
            model_builder: ANSModel4EncoderBuilder::new(component_args),
            costs_table: symbol_costs,
            _marker: PhantomData,
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
        MW::build(self.costs_table.clone())
    }

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder.push_symbol(value, BVGraphComponent::Outdegree);

        if value < self.costs_table.get_component_threshold(BVGraphComponent::Outdegree) {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::Outdegree))
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::Outdegree),
            self.costs_table.get_component_fidelity(BVGraphComponent::Outdegree),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::Outdegree))
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder.push_symbol(value, BVGraphComponent::ReferenceOffset);

        if value < self.costs_table.get_component_threshold(BVGraphComponent::ReferenceOffset) {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::ReferenceOffset))
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::ReferenceOffset),
            self.costs_table.get_component_fidelity(BVGraphComponent::ReferenceOffset),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::ReferenceOffset))
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder.push_symbol(value, BVGraphComponent::BlockCount);

        if value < self.costs_table.get_component_threshold(BVGraphComponent::BlockCount) {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::BlockCount))
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::BlockCount),
            self.costs_table.get_component_fidelity(BVGraphComponent::BlockCount),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::BlockCount))
    }

    fn write_blocks(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder.push_symbol(value, BVGraphComponent::Blocks);

        if value < self.costs_table.get_component_threshold(BVGraphComponent::Blocks) {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::Blocks))
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::Blocks),
            self.costs_table.get_component_fidelity(BVGraphComponent::Blocks),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::Blocks))
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder.push_symbol(value, BVGraphComponent::IntervalCount);

        if value < self.costs_table.get_component_threshold(BVGraphComponent::IntervalCount) {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::IntervalCount))
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::IntervalCount),
            self.costs_table.get_component_fidelity(BVGraphComponent::IntervalCount),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::IntervalCount))
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder.push_symbol(value, BVGraphComponent::IntervalStart);

        if value < self.costs_table.get_component_threshold(BVGraphComponent::IntervalStart) {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::IntervalStart))
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::IntervalStart),
            self.costs_table.get_component_fidelity(BVGraphComponent::IntervalStart),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::IntervalStart))
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder.push_symbol(value, BVGraphComponent::IntervalLen);

        if value < self.costs_table.get_component_threshold(BVGraphComponent::IntervalLen) {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::IntervalLen))
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::IntervalLen),
            self.costs_table.get_component_fidelity(BVGraphComponent::IntervalLen),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::IntervalLen))
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder.push_symbol(value, BVGraphComponent::FirstResidual);

        if value < self.costs_table.get_component_threshold(BVGraphComponent::FirstResidual) {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::FirstResidual))
        }

        let folded_sym = fold_without_streaming_out(
            value,
            self.costs_table.get_component_radix(BVGraphComponent::FirstResidual),
            self.costs_table.get_component_fidelity(BVGraphComponent::FirstResidual),
        );
        Ok(self.costs_table.get_symbol_cost(folded_sym as usize, BVGraphComponent::FirstResidual))
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder.push_symbol(value, BVGraphComponent::Residual);

        if value < self.costs_table.get_component_threshold(BVGraphComponent::Residual) {
            return Ok(self.costs_table.get_symbol_cost(value as usize, BVGraphComponent::Residual))
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