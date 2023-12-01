use crate::{Freq, State};
use crate::ans::traits::Foldable;

pub mod encoder;
pub mod decoder;
pub mod enc_model;
pub mod dec_model;
mod traits;

pub const FASTER_RADIX: u8 = 8;

#[readonly::make]
#[derive(Clone, PartialEq, Debug)]
pub struct EncoderModelEntry {
    pub freq: Freq,
    pub upperbound: u64,
    pub cumul_freq: Freq,
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


pub struct Prelude <F: Foldable> {

    /// Contains, for each index, the data associated to the symbol equal to that index.
    pub table: Vec<EncoderModelEntry>,

    /// The length of the sequence to decode.
    pub sequence_length: u64,

    /// The normalized bits during the encoding process.
    pub normalized_bits: Vec<u32>,

    /// The folded bits during the encoding process.
    pub folded_bits: F,

    pub log2_frame_size: u8,

    pub states: [State; 4],
}