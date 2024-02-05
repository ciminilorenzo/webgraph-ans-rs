use std::fmt::Display;

pub mod mock_writers;
pub mod reader;
pub mod writer;

/// An enumeration of the components getting a different model in the Rust
/// implementation of the BV format.
#[derive(Clone, Copy, Debug)]
pub enum BVGraphComponent {
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

impl Display for BVGraphComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BVGraphComponent::Outdegree => write!(f, "{:<15}", "Outdegree"),
            BVGraphComponent::ReferenceOffset => write!(f, "{:<15}", "ReferenceOffset"),
            BVGraphComponent::BlockCount => write!(f, "{:<15}", "BlockCount"),
            BVGraphComponent::Blocks => write!(f, "{:<15}", "Blocks"),
            BVGraphComponent::IntervalCount => write!(f, "{:<15}", "IntervalCount"),
            BVGraphComponent::IntervalStart => write!(f, "{:<15}", "IntervalStart"),
            BVGraphComponent::IntervalLen => write!(f, "{:<15}", "IntervalLen"),
            BVGraphComponent::FirstResidual => write!(f, "{:<15}", "FirstResidual"),
            BVGraphComponent::Residual => write!(f, "{:<15}", "Residual"),
        }
    }
}

impl From<usize> for BVGraphComponent {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Outdegree,
            1 => Self::ReferenceOffset,
            2 => Self::BlockCount,
            3 => Self::Blocks,
            4 => Self::IntervalCount,
            5 => Self::IntervalStart,
            6 => Self::IntervalLen,
            7 => Self::FirstResidual,
            8 => Self::Residual,
            _ => panic!("Invalid component."),
        }
    }
}

impl BVGraphComponent {
    pub const COMPONENTS: usize = 9;
}
