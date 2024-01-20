pub mod decoder;
pub mod encoder;
pub mod model4decoder;
pub mod model4encoder;
pub mod model4encoder_builder;

use epserde::prelude::*;
use mem_dbg::*;

use crate::traits::folding::FoldRead;
use crate::{Freq, State};

#[derive(Clone, Debug, Epserde, MemDbg, MemSize)]
pub struct Prelude<const RADIX: usize> {
    /// Contains, for each index, the data associated to the symbol equal to that index.
    pub tables: Vec<Vec<EncoderModelEntry>>,

    /// The normalized bits during the encoding process.
    pub normalized_bits: Vec<u32>,

    /// Contains the log2 of the frame size for each model.
    pub frame_sizes: Vec<usize>,

    pub state: State,
}

#[derive(Clone, Copy, Debug, Epserde, MemDbg, MemSize)]
#[repr(C)]
#[zero_copy]
pub struct EncoderModelEntry {
    pub freq: Freq,
    pub cumul_freq: Freq,
    pub upperbound: u64,
}

impl PartialEq for EncoderModelEntry {
    fn eq(&self, other: &Self) -> bool {
        self.freq == other.freq
            && self.upperbound == other.upperbound
            && self.cumul_freq == other.cumul_freq
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
