use webgraph::prelude::Encode;
use std::convert::Infallible;
use crate::bvgraph::BVGraphComponent;

/// An estimator that simply returns the cost of each symbol calculated as the log2 of the value plus 2.
#[derive(Clone, Default)]
pub struct Log2Estimator {}

impl Log2Estimator {
    fn get_symbol_cost(&self, value: u64, _component: BVGraphComponent) -> usize {
        u64::ilog2(value + 2) as usize
    }
}

impl Encode for Log2Estimator {
    type Error = Infallible;

    fn start_node(&mut self, _node: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn write_outdegree(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::Outdegree))
    }

    fn write_reference_offset(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::ReferenceOffset))
    }

    fn write_block_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::BlockCount))
    }

    fn write_block(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::Blocks))
    }

    fn write_interval_count(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::IntervalCount))
    }

    fn write_interval_start(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::IntervalStart))
    }

    fn write_interval_len(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::IntervalLen))
    }

    fn write_first_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::FirstResidual))
    }

    fn write_residual(&mut self, value: u64) -> Result<usize, Self::Error> {
        Ok(self.get_symbol_cost(value, BVGraphComponent::Residual))
    }

    fn flush(&mut self) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn end_node(&mut self, _node: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }
}