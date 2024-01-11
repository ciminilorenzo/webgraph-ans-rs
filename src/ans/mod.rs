use crate::{EncoderModelEntry, State};
use crate::traits::folding::Fold;

mod encoder;
mod decoder;
mod model4encoder;
mod model4decoder;

pub const K: usize = 16;
pub const K_LOG2: usize = 4;

/// How big M (the frame) can be. This constrained is imposed by the fact that B and K are fixed and
/// State is a u64.
pub const MAXIMUM_LOG2_M: usize = 28;

pub struct Prelude<const RADIX: usize, F: Fold<RADIX>> {
    /// Contains, for each index, the data associated to the symbol equal to that index.
    pub table: Vec<EncoderModelEntry>,

    /// The length of the sequence to decode.
    pub sequence_length: u64,

    /// The normalized bits during the encoding process.
    pub normalized_bits: Vec<u32>,

    /// The folded bits during the encoding process.
    pub folded_bits: F,

    pub log2_frame_size: usize,

    pub states: [State; 4],
}