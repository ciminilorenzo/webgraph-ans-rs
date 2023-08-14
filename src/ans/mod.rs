use bitvec::vec::BitVec;
use crate::{Freq, State};

pub mod folded_stream_ans_encoder;
pub mod folded_stream_ans_decoder;
pub mod encoder_model;
mod ans_util;
pub mod decoder_model;


#[readonly::make]
#[derive(Debug, Clone, PartialEq)]
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

#[readonly::make]
pub struct EncodingResult {
    pub state: State,
    pub normalized_bits: BitVec,
    pub folded_bits: BitVec,
}