pub mod decoder;
pub mod encoder;
pub mod model4decoder;
pub mod model4encoder;
pub mod model4encoder_builder;

use crate::traits::folding::Fold;
use crate::{Freq, State};

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

#[derive(Clone, Debug)]
pub struct EncoderModelEntry {
    pub freq: Freq,
    pub upperbound: u64,
    pub cumul_freq: Freq,
}

impl PartialEq for EncoderModelEntry {
    fn eq(&self, other: &Self) -> bool {
        self.freq == other.freq && self.upperbound == other.upperbound && self.cumul_freq == other.cumul_freq
    }
}

impl From<(Freq, u64, Freq)> for EncoderModelEntry {
    fn from(tuple: (Freq, u64, Freq)) -> Self {
        Self {
            freq: tuple.0,
            upperbound: tuple.1,
            cumul_freq: tuple.2,
        }
    }
}
