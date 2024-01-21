use std::{convert::Infallible};
use std::marker::PhantomData;
use webgraph::graph::bvgraph::BVGraphCodesWriter;

use crate::bvgraph::Component;
use crate::multi_model_ans::encoder::ANSCompressorPhase;
use crate::{
    multi_model_ans::{
        encoder::ANSEncoder, model4encoder::ANSModel4Encoder,
        model4encoder_builder::ANSModel4EncoderBuilder,
    },
};
use crate::bvgraph::mock_writers::{ANSymbolTable, EntropyMockWriter, MockWriter};
use crate::utils::ans_utilities::folding_without_streaming_out;


pub struct BVGraphModelBuilder<const FIDELITY: usize, const RADIX: usize, MW>
where
    MW: BVGraphCodesWriter + MockWriter<FIDELITY, RADIX>,
{
    model_builder: ANSModel4EncoderBuilder<FIDELITY, RADIX>,
    symbol_costs: ANSymbolTable<FIDELITY, RADIX>,
    _marker: PhantomData<MW>,
}

impl<const FIDELITY: usize, const RADIX: usize, MW> BVGraphModelBuilder<FIDELITY, RADIX, MW>
where
    MW: BVGraphCodesWriter + MockWriter<FIDELITY, RADIX>,
{
    const FOLDING_THRESHOLD: u64 = 1 << (FIDELITY + RADIX - 1);

    // symbol_costs should be ANSymbolTable<FIDELITY, RADIX>::initialize_with_binary_cost(9) here
    pub fn new(symbol_costs: ANSymbolTable<FIDELITY, RADIX>) -> Self {
        Self {
            model_builder: ANSModel4EncoderBuilder::<FIDELITY, RADIX>::new(9),
            symbol_costs,
            _marker: PhantomData,
        }
    }

    /// Build an [`ANSModel4Encoder`] from the symbols written to this
    /// [`BVGraphModelBuilder`].
    pub fn build(self) -> ANSModel4Encoder {
        self.model_builder.build()
    }
}

impl<const FIDELITY: usize, const RADIX: usize, MW> BVGraphCodesWriter for BVGraphModelBuilder<FIDELITY, RADIX, MW>
where
    MW: BVGraphCodesWriter + MockWriter<FIDELITY, RADIX>,
{
    type Error = Infallible;

    type MockWriter = MW;

    fn mock(&self) -> Self::MockWriter {
        // !!!!! now it's a clone since it's &Self. Otherwise i would give ownership !!!!!
        MW::build(self.symbol_costs.clone())
    }

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, Component::Outdegree as usize);

        if value < Self::FOLDING_THRESHOLD {
            return Ok(self.symbol_costs.table[Component::Outdegree as usize][value as usize])
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.symbol_costs.table[Component::Outdegree as usize][folded_sym as usize])
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, Component::ReferenceOffset as usize);

        if value < Self::FOLDING_THRESHOLD {
            return  Ok(self.symbol_costs.table[Component::ReferenceOffset as usize][value as usize])
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.symbol_costs.table[Component::ReferenceOffset as usize][folded_sym as usize])
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, Component::BlockCount as usize);

        if value < Self::FOLDING_THRESHOLD {
            return  Ok(self.symbol_costs.table[Component::BlockCount as usize][value as usize])
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.symbol_costs.table[Component::BlockCount as usize][folded_sym as usize])
    }

    fn write_blocks(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, Component::Blocks as usize);

        if value < Self::FOLDING_THRESHOLD {
            return  Ok(self.symbol_costs.table[Component::Blocks as usize][value as usize])
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.symbol_costs.table[Component::Blocks as usize][folded_sym as usize])
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, Component::IntervalCount as usize);

        if value < Self::FOLDING_THRESHOLD {
            return  Ok(self.symbol_costs.table[Component::IntervalCount as usize][value as usize])
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.symbol_costs.table[Component::IntervalCount as usize][folded_sym as usize])
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, Component::IntervalStart as usize);

        if value < Self::FOLDING_THRESHOLD {
            return  Ok(self.symbol_costs.table[Component::IntervalStart as usize][value as usize])
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.symbol_costs.table[Component::IntervalStart as usize][folded_sym as usize])
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, Component::IntervalLen as usize);

        if value < Self::FOLDING_THRESHOLD {
            return  Ok(self.symbol_costs.table[Component::IntervalLen as usize][value as usize])
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.symbol_costs.table[Component::IntervalLen as usize][folded_sym as usize])
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, Component::FirstResidual as usize);

        if value < Self::FOLDING_THRESHOLD {
            return  Ok(self.symbol_costs.table[Component::FirstResidual as usize][value as usize])
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.symbol_costs.table[Component::FirstResidual as usize][folded_sym as usize])
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.model_builder
            .push_symbol(value, Component::Residual as usize);

        if value < Self::FOLDING_THRESHOLD {
            return  Ok(self.symbol_costs.table[Component::Residual as usize][value as usize])
        }

        let folded_sym = folding_without_streaming_out(value, RADIX, FIDELITY);
        Ok(self.symbol_costs.table[Component::Residual as usize][folded_sym as usize])
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
pub struct BVGraphWriter<const FIDELITY: usize, const RADIX: usize> {
    /// The container containing the buffers (one for each [component](`Component`)) where symbols are collected.
    data: [Vec<usize>; 9],

    /// The index of the node the encoder is currently encoding.
    curr_node: usize,

    /// The encoder used by this writer to encode symbols.
    encoder: ANSEncoder<FIDELITY, RADIX>,

    /// A buffer containing a [`ANSCompressorPhase`], one for each node.
    phases: Vec<ANSCompressorPhase>,

    mock_writer: EntropyMockWriter<FIDELITY, RADIX>,
}

impl<const FIDELITY: usize, const RADIX: usize> BVGraphWriter<FIDELITY, RADIX> {

    // costs_table should be ANSymbolTable<FIDELITY, RADIX>::new by passing the table of entries of the encoder
    pub fn new(model: ANSModel4Encoder, costs_table: ANSymbolTable<FIDELITY, RADIX>) -> Self {
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
            encoder: ANSEncoder::<FIDELITY, RADIX>::new(model),
            phases: Vec::new(),
        }
    }

    /// Consume self and return the encoder.
    pub fn into_inner(self) -> (ANSEncoder<FIDELITY, RADIX>, Vec<ANSCompressorPhase>) {
        (self.encoder, self.phases)
    }
}

impl<const FIDELITY: usize, const RADIX: usize> BVGraphCodesWriter for BVGraphWriter<FIDELITY, RADIX> {
    type Error = Infallible;

    type MockWriter = EntropyMockWriter<FIDELITY, RADIX>;

    fn mock(&self) -> Self::MockWriter {
        self.mock_writer.clone() // i must return costs even below so i have to keep an instance of the mock
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
        self.mock_writer.write_outdegree(value)
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[Component::ReferenceOffset as usize].push(value as usize);
        self.mock_writer.write_reference_offset(value)
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[Component::BlockCount as usize].push(value as usize);
        self.mock_writer.write_block_count(value)
    }

    fn write_blocks(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[Component::Blocks as usize].push(value as usize);
        self.mock_writer.write_blocks(value)
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[Component::IntervalCount as usize].push(value as usize);
        self.mock_writer.write_interval_count(value)
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[Component::IntervalStart as usize].push(value as usize);
        self.mock_writer.write_interval_start(value)
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[Component::IntervalLen as usize].push(value as usize);
        self.mock_writer.write_interval_len(value)
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[Component::FirstResidual as usize].push(value as usize);
        self.mock_writer.write_first_residual(value)
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.data[Component::Residual as usize].push(value as usize);
        self.mock_writer.write_residual(value)
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