use crate::traits::folding::FoldRead;
use crate::{Freq, State};
use strength_reduce::StrengthReducedU64;

mod decoder;
mod encoder;
mod model4decoder;
mod model4encoder;

pub const K: usize = 16;
pub const K_LOG2: usize = 4;

/// How big M (the frame) can be. This constrained is imposed by the fact that B and K are fixed and
/// State is a u64.
pub const MAXIMUM_LOG2_M: usize = 28;

pub struct Prelude<const RADIX: usize, F: FoldRead<RADIX>> {
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

#[derive(Clone, Debug)]
pub struct EncoderModelEntry {
    pub freq: Freq,
    pub upperbound: u64,
    pub cumul_freq: Freq,
    pub fast_divisor: StrengthReducedU64,
}

impl PartialEq for EncoderModelEntry {
    fn eq(&self, other: &Self) -> bool {
        self.freq == other.freq
            && self.upperbound == other.upperbound
            && self.cumul_freq == other.cumul_freq
            && self.fast_divisor.get() == other.fast_divisor.get()
    }
}

impl From<(Freq, u64, Freq)> for EncoderModelEntry {
    fn from(tuple: (Freq, u64, Freq)) -> Self {
        Self {
            freq: tuple.0,
            upperbound: tuple.1,
            cumul_freq: tuple.2,
            fast_divisor: match tuple.0 > 0 {
                true => StrengthReducedU64::new(tuple.0 as u64),
                false => StrengthReducedU64::new(1),
            },
        }
    }
}
