#![allow(dead_code)]

use crate::traits::quasi::Quasi;
use epserde::prelude::*;
use epserde::traits::ZeroCopy;
use strength_reduce::StrengthReducedU64;
pub mod ans;
pub mod multi_model_ans;

pub mod bvgraph;

mod traits;
mod utils;

/// How many bits are extracted/added from/to the state during the encoding/decoding process.
pub const LOG2_B: usize = 32;

pub const MAX_RAW_SYMBOL: u64 = (1 << 48) - 1;

/// The type representing the folded symbols.
///
/// # Note
/// This implementation assumes that the maximum symbol is u16::MAX. If more symbols are present,
/// RADIX and FIDELITY should be changed since ANS gets worse with a lot of symbols.
///
/// Moreover, since most of the DS used within the project are tables where symbols data is located
/// in the index equal to the symbol, this type can be interpreted as the maximum symbol index we can
/// have
pub type Symbol = u16;

/// The type representing the raw symbols, i.e. the symbols coming from the input.
pub type RawSymbol = u64;

/// The type representing the state of the encoder/decoder.
pub type State = u64;

/// The type representing the frequencies of the symbols. This type is bounded to be u16 since we deliberately accept to
/// have frequencies that can reach at most this value. This is done in order to have entries for the decoder that have
/// both the frequency and cumulated frequency of each symbol as 16-bit unsigned.
pub type Freq = u16;

/// The default value for RADIX used by both the encoder and the decoder.
pub const FASTER_RADIX: usize = 8;

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

#[derive(Clone, Copy, Debug, Default, Epserde)]
#[repr(C)]
#[zero_copy]
pub struct DecoderModelEntry<const RADIX: usize, T: Quasi<RADIX> + ZeroCopy + 'static> {
    pub freq: Freq,
    pub cumul_freq: Freq,
    pub quasi_folded: T,
}
