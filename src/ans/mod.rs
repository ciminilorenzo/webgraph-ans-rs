use crate::ans::traits::{Fold, Quasi};
use crate::{Freq, State};
use strength_reduce::StrengthReducedU64;

pub mod dec_model;
pub mod decoder;
pub mod enc_model;
pub mod encoder;
pub mod enc_model_builder;
mod traits;


/// The default value for RADIX used by both the encoder and the decoder.
pub const FASTER_RADIX: usize = 8;

#[readonly::make]
#[derive(Clone, Debug)]
pub struct EncoderModelEntry {
    pub freq: Freq,
    pub upperbound: u64,
    pub cumul_freq: Freq,
    pub fast_divisor: StrengthReducedU64,
}

impl PartialEq for EncoderModelEntry {
    fn eq(&self, other: &Self) -> bool {
        self.freq == other.freq &&
            self.upperbound == other.upperbound &&
            self.cumul_freq == other.cumul_freq &&
            self.fast_divisor.get() == other.fast_divisor.get()
    }
}

impl From<(Freq, u64, Freq)> for EncoderModelEntry {
    fn from(tuple: (Freq, u64, Freq)) -> Self {
        let fast_divisor = if tuple.0 > 0 {
            StrengthReducedU64::new(tuple.0 as u64)
        } else {
            StrengthReducedU64::new(1)
        };

        Self {
            freq: tuple.0,
            upperbound: tuple.1,
            cumul_freq: tuple.2,
            fast_divisor,
        }
    }
}


#[readonly::make]
#[derive(Clone, Debug, Default)]
pub struct DecoderModelEntry<const RADIX: usize, T>
    where T: Quasi<RADIX>
{
    pub freq: Freq,
    pub cumul_freq: Freq,
    pub quasi_folded: T,
}

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
