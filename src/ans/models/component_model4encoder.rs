use std::ops::Index;

use mem_dbg::{MemDbg, MemSize};

use epserde::Epserde;

use crate::{Freq, State, Symbol};
use crate::ans::B;

/// The entry containing all the needed data to encode a specific [`Symbol`].
#[derive(Clone, Copy, Debug, Epserde, MemDbg, MemSize)]
#[repr(C)]
#[zero_copy]
pub struct EncoderModelEntry {
    /// The upperbound of the symbol, that is the maximum value starting from which we can safely
    /// encode this symbol without overflowing the interval in which the state of the compressor
    /// is allowed to be.
    pub upperbound: u32,

    /// The cumulative frequency of the symbol.
    pub cumul_freq: Freq,

    /// The frequency of the symbol.
    pub freq: Freq,
}

impl EncoderModelEntry {
    pub fn new(freq: u16, k: usize, cumul: Freq) -> Self {
        Self {
            freq,
            upperbound: (1_u32 << (k + B)) * freq as State,
            cumul_freq: cumul,
        }
    }
}

#[derive(Clone, MemDbg, MemSize, Epserde, Debug)]
/// The ANS model used by the encoder to encode [`Symbol`] of a specific component.
pub struct ANSComponentModel4Encoder {
    /// A table containing, at each index, an [entry](EncoderModelEntry) associated with the [`Symbol`]
    /// equal to that index.
    pub table: Vec<EncoderModelEntry>,

    /// The log2 of the frame size for this component.
    pub frame_size: usize,

    /// The radix used by the current model.
    pub radix: usize,

    /// The fidelity used by the current model.
    pub fidelity: usize,

    /// The threshold representing the symbol from which the folding starts.
    pub folding_threshold: u64,

    pub folding_offset: u64,
}

impl Default for ANSComponentModel4Encoder {
    fn default() -> Self {
        Self {
            table: Vec::new(),
            frame_size: 0,
            radix: 2,
            fidelity: 2,
            folding_threshold: 10,
            folding_offset: 10,
        }
    }
}

impl Index<Symbol> for ANSComponentModel4Encoder {
    type Output = EncoderModelEntry;

    #[inline(always)]
    fn index(&self, symbol: Symbol) -> &Self::Output {
        &self.table[symbol as usize]
    }
}
