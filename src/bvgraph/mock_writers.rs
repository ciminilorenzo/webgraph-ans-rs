use std::convert::Infallible;
use webgraph::prelude::BVGraphCodesWriter;
use crate::bvgraph::Component;


pub trait MockWriter {
    fn build(symbol_costs_table: Vec<Vec<usize>>) -> Self;
}

pub struct EntropyMockWriter {
    symbol_costs_table: Vec<Vec<usize>>,
}

impl MockWriter for EntropyMockWriter {
    fn build(symbol_costs_table: Vec<Vec<usize>>) -> Self {
        Self {symbol_costs_table }
    }
}

impl BVGraphCodesWriter for EntropyMockWriter {
    type Error = Infallible;

    type MockWriter = Log2MockWriter; // it's essentially a marker

    fn mock(&self) -> Self::MockWriter {
        Log2MockWriter {} // thus we can return a fake one
    }

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.symbol_costs_table[Component::Outdegree as usize][value as usize])
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        // cerca di prendere dati per simbolo 1
        Ok(self.symbol_costs_table[Component::ReferenceOffset as usize][value as usize])
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.symbol_costs_table[Component::BlockCount as usize][value as usize])
    }

    fn write_blocks(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.symbol_costs_table[Component::Blocks as usize][value as usize])
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.symbol_costs_table[Component::IntervalCount as usize][value as usize])
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.symbol_costs_table[Component::IntervalStart as usize][value as usize])
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.symbol_costs_table[Component::IntervalLen as usize][value as usize])
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.symbol_costs_table[Component::FirstResidual as usize][value as usize])
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.symbol_costs_table[Component::Residual as usize][value as usize])
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn len(value: u64) -> Result<usize, Infallible> {
    Ok((value + 2).ilog2() as usize)
}

/// A mock writer that returns `⌊log₂(x)⌋` as the number of bits written
/// encoding `x`.
pub struct Log2MockWriter {}

impl MockWriter for Log2MockWriter {
    fn build(_symbol_costs_table: Vec<Vec<usize>>) -> Self {
        Self {}
    }
}

impl BVGraphCodesWriter for Log2MockWriter {
    type Error = Infallible;

    type MockWriter = Self; // it's essentially a marker

    fn mock(&self) -> Self::MockWriter {
        Log2MockWriter {} // thus we can return a fake one
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