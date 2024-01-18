pub mod writer;
pub mod reader;
pub mod mock_writers;

/// An enumeration of the components getting a different model in the Rust
/// implementation of the BV format.
pub enum Component {
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
