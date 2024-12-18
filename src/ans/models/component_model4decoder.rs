use std::ops::Index;

use epserde::Epserde;

use crate::{Freq, Symbol};

/// The entry containing all the data needed to decode a specific [`Symbol`].
#[derive(Clone, Copy, Debug, Default, Epserde)]
#[repr(C)]
#[zero_copy]
pub struct DecoderModelEntry {
    /// The frequency of the symbol.
    pub freq: Freq,

    /// The cumulative frequency of the symbol.
    pub cumul_freq: Freq,

    /// A 64-bit integer, containing the number of folds that we need to retrieve to get the original
    /// symbol back in the most significant 16 bits, and the quasi-folded symbol in the least
    /// significant 48 bits.
    pub quasi_folded: u64,
}

/// The ANS model used by the decoder to decode symbols of a specific BvGraph component.
pub struct ANSComponentModel4Decoder {
    /// A table containing, at each index, an [entry](DecoderModelEntry) associated with the [`Symbol`]
    /// equal to that index.
    pub table: Vec<DecoderModelEntry>,

    /// The log2 of the frame size for this component.
    pub frame_size: usize,

    /// The radix used by the current model.
    pub radix: usize,

    /// The fidelity used by the current model.
    pub fidelity: usize,
}

impl ANSComponentModel4Decoder {
    pub fn new(
        table: Vec<DecoderModelEntry>,
        frame_size: usize,
        radix: usize,
        fidelity: usize,
    ) -> ANSComponentModel4Decoder {
        ANSComponentModel4Decoder {
            table,
            frame_size,
            radix,
            fidelity,
        }
    }
}

impl Index<Symbol> for ANSComponentModel4Decoder {
    type Output = DecoderModelEntry;

    #[inline(always)]
    fn index(&self, symbol: Symbol) -> &Self::Output {
        &self.table[symbol as usize]
    }
}
