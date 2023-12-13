use crate::ans::traits::Fold;
use crate::{Freq, State};
use strength_reduce::StrengthReducedU64;

pub mod dec_model;
pub mod decoder;
pub mod enc_model;
pub mod encoder;
mod traits;

pub const FASTER_RADIX: usize = 8;

#[readonly::make]
#[derive(Clone, Debug)]
pub struct EncoderModelEntry {
    pub freq: Freq,
    pub upperbound: u64,
    pub cumul_freq: Freq,
    pub reciprocal: StrengthReducedU64,
}

impl From<(Freq, u64, Freq)> for EncoderModelEntry {
    fn from(tuple: (Freq, u64, Freq)) -> Self {
        let reciprocal = if tuple.0 > 0 {
            StrengthReducedU64::new(tuple.0 as u64)
        } else {
            StrengthReducedU64::new(1)
        };

        Self {
            freq: tuple.0,
            upperbound: tuple.1,
            cumul_freq: tuple.2,
            reciprocal,
        }
    }
}

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
