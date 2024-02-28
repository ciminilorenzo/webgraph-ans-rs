pub mod decoder;
pub mod encoder;
pub mod model4decoder;
pub mod model4encoder;
pub mod model4encoder_builder;

use crate::ans::model4encoder::ANSComponentModel4Encoder;
use crate::{Freq, State, B};
use epserde::Epserde;
use mem_dbg::{MemDbg, MemSize};

#[cfg(not(feature = "arm"))]
use crate::utils::ans_utils::get_reciprocal_data;

#[derive(Clone, Debug, Epserde, MemDbg, MemSize)]
pub struct Prelude {
    /// Contains, for each index, the data associated to the symbol equal to that index.
    pub tables: Vec<ANSComponentModel4Encoder>,

    /// The normalized bits during the encoding process.
    pub stream: Vec<u16>,

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
    pub upperbound: u32,

    #[cfg(not(feature = "arm"))]
    pub reciprocal: u32,

    #[cfg(not(feature = "arm"))]
    /// The complementary frequency of the symbol, that is: frame_size - freq.
    pub cmpl_freq: u16,

    /// The cumulative frequency of the symbol.
    pub cumul_freq: Freq,

    #[cfg(feature = "arm")]
    pub freq: Freq,

    #[cfg(not(feature = "arm"))]
    pub magic: u8,
}

impl EncoderModelEntry {
    #[allow(unused_variables)]
    pub fn new(freq: u16, k: usize, cumul: Freq, m: usize) -> Self {
        #[cfg(not(feature = "arm"))]
        let (reciprocal, magic) = if freq > 0 {
            get_reciprocal_data(freq)
        } else {
            // we may have entries for symbols that doesn't exist. fill with dummy data
            // since we won't be looking for it.
            (0, 0)
        };

        Self {
            #[cfg(not(feature = "arm"))]
            reciprocal,
            #[cfg(not(feature = "arm"))]
            magic,
            #[cfg(not(feature = "arm"))]
            cmpl_freq: (1 << m) - freq,
            #[cfg(feature = "arm")]
            freq,
            upperbound: (1_u32 << (k + B)) * freq as State,
            cumul_freq: cumul,
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
