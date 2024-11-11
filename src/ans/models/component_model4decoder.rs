use std::ops::Index;

use epserde::Epserde;

use crate::bvgraph::BVGraphComponent;
use crate::{Freq, Symbol};

/// The entry containing all the needed data to decode a specific [`Symbol`].
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

/// The model of a specific [component](BVGraphComponent) used by the ANS decoder to decode one of its [symbols](Symbol).
pub struct ANSComponentModel4Decoder {
    /// A table containing, at each index, an [entry](DecoderModelEntry) for the symbol equal to that index.
    pub table: Vec<DecoderModelEntry>,

    /// The log2 of the frame size for this [component](BVGraphComponent).
    frame_size: usize,

    /// The radix used by the current model.
    radix: usize,

    /// The fidelity used by the current model.
    fidelity: usize,
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
