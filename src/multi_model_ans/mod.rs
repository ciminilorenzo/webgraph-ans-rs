pub mod decoder;
pub mod encoder;
pub mod model4decoder;
pub mod model4encoder;
pub mod model4encoder_builder;

use crate::multi_model_ans::model4encoder::ANSComponentModel4Encoder;
use crate::{Freq, State, B};
use epserde::Epserde;
use mem_dbg::{MemDbg, MemSize};

#[cfg(not(feature = "arm"))]
use crate::utils::ans_utilities::get_reciprocal_data;

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
    /// The upperbound of the symbol, that is the maximum value starting from which we can safely encode this specific
    /// symbol without overflowing the interval in which the state of the compressor can be.
    pub upperbound: u64,

    #[cfg(not(feature = "arm"))]
    pub reciprocal: u64,

    #[cfg(not(feature = "arm"))]
    pub magic: u8,

    // todo: do we need it? it seems like not since we can calculate it from freqs in the decoder side
    /// The frequency of the symbol.
    pub freq: Freq,

    /// The cumulative frequency of the symbol.
    pub cumul_freq: Freq,
}

impl EncoderModelEntry {
    pub fn new(freq: u16, k: usize, cumul: Freq) -> Self {
        #[cfg(not(feature = "arm"))]
        let (reciprocal, magic) = if freq > 0 {
            get_reciprocal_data(freq)
        } else {
            (0, 0)
        };

        Self {
            #[cfg(not(feature = "arm"))]
            reciprocal,
            #[cfg(not(feature = "arm"))]
            magic,
            freq,
            upperbound: (1_u64 << (k + B)) * freq as u64,
            cumul_freq: cumul,
        }
    }
}

impl PartialEq for EncoderModelEntry {
    fn eq(&self, other: &Self) -> bool {
        self.freq == other.freq
            && self.upperbound == other.upperbound
            && self.cumul_freq == other.cumul_freq
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
