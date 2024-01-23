pub mod decoder;
pub mod encoder;
pub mod model4decoder;
pub mod model4encoder;
pub mod model4encoder_builder;

use epserde::prelude::*;
use mem_dbg::*;

use crate::{Freq, State};

#[derive(Clone, Debug, Epserde, MemDbg, MemSize)]
pub struct Prelude {
    /// Contains, for each index, the data associated to the symbol equal to that index.
    pub tables: Vec<Vec<EncoderModelEntry>>,

    /// The normalized bits during the encoding process.
    pub normalized_bits: Vec<u32>,

    /// Contains the log2 of the frame size for each model.
    pub frame_sizes: Vec<usize>,

    pub state: State,
}


#[derive(Debug, Clone, Copy, Epserde, MemDbg, MemSize)]
#[zero_copy]
#[repr(C)]
pub struct ANSCompressorPhase {
    pub state: State,
    pub stream_pointer: usize,
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

#[derive(Clone, Copy, Debug, Default, Epserde)]
#[repr(C)]
#[zero_copy]
pub struct DecoderModelEntry {
    pub freq: Freq,
    pub cumul_freq: Freq,
    pub quasi_folded: u64,
}
