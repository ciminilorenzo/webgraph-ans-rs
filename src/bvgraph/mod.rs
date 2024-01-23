pub mod writer;
pub mod reader;
pub mod mock_writers;

/// An enumeration of the components getting a different model in the Rust
/// implementation of the BV format.
#[derive(Clone, Copy)]
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
