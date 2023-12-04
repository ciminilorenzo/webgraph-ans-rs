use fastdivide::DividerU64;
use crate::{Freq, State};
use crate::ans::traits::Foldable;

pub mod encoder;
pub mod decoder;
pub mod enc_model;
pub mod dec_model;
mod traits;

pub const FASTER_RADIX: usize = 8;

#[readonly::make]
#[derive(Clone, PartialEq, Debug)]
pub struct EncoderModelEntry {
    pub freq: Freq,
    pub upperbound: u64,
    pub cumul_freq: Freq,
    pub reciprocal: DividerU64,
}

impl From<(Freq, u64, Freq)> for EncoderModelEntry {
    fn from(tuple: (Freq, u64, Freq)) -> Self {
        let reciprocal = if tuple.0 > 0 {
            DividerU64::divide_by(tuple.0 as u64)
        } else {
            DividerU64::divide_by(1)
        };

        Self {
            freq: tuple.0,
            upperbound: tuple.1,
            cumul_freq: tuple.2,
            reciprocal,
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