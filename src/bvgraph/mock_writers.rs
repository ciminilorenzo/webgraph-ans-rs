use std::convert::Infallible;
use webgraph::prelude::BVGraphCodesWriter;
use crate::bvgraph::Component;
use crate::bvgraph::writer::Log2MockWriter;
use crate::multi_model_ans::model4encoder::ANSModel4Encoder;

/*

fn symbol_len(value: u64, model: &ANSModel4Encoder, model_index: usize) -> Result<usize, Infallible> {
    let frequency = *model.tables[model_index][value as usize].freq;
    let total_freq = model.tables[model_index]
        .iter()
        .map(|entry| entry.freq)
        .sum::<u64>();

    let probability = frequency as f64 / total_freq as f64;
    let inverse = 1.0 / probability;
    let shifted = inverse * (1 << 16) as f64;
    let rounded = shifted.round() as u64;

    Ok(10)
}

pub struct EntropyMockWriter<'a> {
    model: &'a ANSModel4Encoder,
}

impl <'a> EntropyMockWriter<'a> {
    pub fn new(model: &'a ANSModel4Encoder) -> Self {
        Self {
            model,
        }
    }
}

impl BVGraphCodesWriter for EntropyMockWriter {
    type Error = Infallible;

    type MockWriter = Log2MockWriter;

    fn mock(&self) -> Self::MockWriter {
        Log2MockWriter::new()
    }

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        symbol_len(value, self.model, Component::Outdegree as usize)
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        symbol_len(value, self.model, Component::ReferenceOffset as usize)
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        symbol_len(value, self.model, Component::BlockCount as usize)
    }

    fn write_blocks(&mut self, value: u64) -> Result<usize, Self::Error> {
        symbol_len(value, self.model, Component::Blocks as usize)
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        symbol_len(value, self.model, Component::IntervalCount as usize)
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        symbol_len(value, self.model, Component::IntervalStart as usize)
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        symbol_len(value, self.model, Component::IntervalLen as usize)
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        symbol_len(value, self.model, Component::FirstResidual as usize)
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        symbol_len(value, self.model, Component::Residual as usize)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
*/