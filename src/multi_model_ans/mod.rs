pub mod decoder;
pub mod encoder;
pub mod model4decoder;
pub mod model4encoder;
pub mod model4encoder_builder;

use epserde::Epserde;
use mem_dbg::{MemDbg, MemSize};
use crate::{Freq, State};
use crate::multi_model_ans::model4encoder::ANSComponentModel4Encoder;

#[derive(Clone, Debug, Epserde, MemDbg, MemSize)]
pub struct Prelude {
    /// Contains, for each index, the data associated to the symbol equal to that index.
    pub tables: Vec<ANSComponentModel4Encoder>,

    /// The normalized bits during the encoding process.
    pub stream: Vec<u32>,

    /// The state of the encoder after having encoded the last symbol of the input.
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
    /// The frequency of the symbol.
    pub freq: Freq,

    /// The cumulative frequency of the symbol.
    pub cumul_freq: Freq,

    /// The upperbound of the symbol, that is the maximum value starting from which we can safely encode this specific
    /// symbol without overflowing the interval in which the state of the compressor can be.
    pub upperbound: u64,
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

#[derive(Clone, Copy, Debug, Default, Epserde)]
#[repr(C)]
#[zero_copy]
pub struct DecoderModelEntry {
    /// The frequency of the symbol.
    pub freq: Freq,

    /// The cumulative frequency of the symbol.
    pub cumul_freq: Freq,

    /// A 64-bit integer, containing the number of folds that we need to successively unfold to get the raw symbol back
    /// in the most significant 16 bits, and the quasi-folded symbol in the least significant 48 bits.
    pub quasi_folded: u64,
}
