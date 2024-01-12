pub mod decoder;
pub mod model4encoder_builder;
pub mod model4decoder;
pub mod encoder;
pub mod model4encoder;

use crate::traits::folding::Fold;
use crate::{EncoderModelEntry, State};


pub struct Prelude<const RADIX: usize, F: Fold<RADIX>> {
    /// Contains, for each index, the data associated to the symbol equal to that index.
    pub tables: Vec<Vec<EncoderModelEntry>>,

    /// The normalized bits during the encoding process.
    pub normalized_bits: Vec<u32>,

    /// The folded bits during the encoding process.
    pub folded_bits: F,

    /// Contains the log2 of the frame size for each model.
    pub frame_sizes: Vec<usize>,

    pub state: State,
}


// WIP: these could possibly be associated to every possible index of the model. At this point functions like
// decode would take a ModelIndex instead of a usize.
pub enum ModelIndex {
    ReferenceNumber,
    BlockCount,
    IntervalsCount,
    IntervalLeftExtreme,
    IntervalLength,
    Residual,
}

impl ModelIndex {
    pub fn index(&self) -> usize {
        match self {
            ModelIndex::ReferenceNumber => 0,
            ModelIndex::BlockCount => 1,
            ModelIndex::IntervalsCount => 2,
            ModelIndex::IntervalLeftExtreme => 3,
            ModelIndex::IntervalLength => 4,
            ModelIndex::Residual => 5,
        }
    }
}