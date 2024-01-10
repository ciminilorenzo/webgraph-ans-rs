use crate::ans::dec_model::VecFrame;
use crate::ans::traits::{Decode, Fold, Quasi};
use crate::ans::{Prelude, FASTER_RADIX, DecoderModelEntry};
use crate::{RawSymbol, State, LOG2_B};
use crate::ans::enc_model::SymbolLookup;

/// The streaming rangeANS decoder that uses the symbol folding technique.
#[derive(Clone)]
pub struct FoldedStreamANSDecoder<
    const FIDELITY: usize,
    const RADIX: usize = FASTER_RADIX,
    H = u64,
    M = VecFrame<RADIX, H>,
    F = Vec<u8>>
    where
        H: Quasi<RADIX>,
        M: Decode + SymbolLookup<State, Output = DecoderModelEntry<RADIX, H>>,
        F: Fold<RADIX>,
{
    model: M,

    /// The normalized bits during the encoding process.
    normalized_bits: Vec<u32>,

    /// The folded bits during the encoding process.
    folded_bits: F,

    state: State,

    last_unfolded_pos: usize,
}

impl<const FIDELITY: usize, const RADIX: usize, H, M, F> FoldedStreamANSDecoder<FIDELITY, RADIX, H, M, F>
where
    H: Quasi<RADIX>,
    M: Decode + SymbolLookup<State, Output = DecoderModelEntry<RADIX, H>>,
    F: Fold<RADIX>,
{
    /// The lower bound of the interval.
    const LOWER_BOUND: State = 1 << 32;

    /// Creates a personalized FoldedStreamANSDecoder with the current values of `FIDELITY` and `RADIX` and the
    /// given model.
    pub fn with_parameters(prelude: Prelude<RADIX, F>, model: M) -> Self {
        Self {
            last_unfolded_pos: prelude.folded_bits.len(),
            model,
            normalized_bits: prelude.normalized_bits,
            folded_bits: prelude.folded_bits,
            state: prelude.state,
        }
    }
}

impl<const FIDELITY: usize> FoldedStreamANSDecoder<FIDELITY, FASTER_RADIX, u64, VecFrame<FASTER_RADIX, u64>, Vec<u8>>
{
    /// Creates the standard FoldedStreamANSDecoder from the given parameters.
    ///
    /// The standard decoder uses fixed types for this struct's generics. This means that,
    /// by using this constructor, you're prevented from tuning any another parameter but fidelity.
    /// If you want to create a decoder with different components, you should use the [this](Self::with_parameters)
    pub fn new(prelude: Prelude<FASTER_RADIX, Vec<u8>>) -> Self {
        let folding_offset = (1 << (FIDELITY - 1)) * ((1 << FASTER_RADIX) - 1);
        let folding_threshold = 1 << (FIDELITY + FASTER_RADIX - 1);

        let vec_model = VecFrame::<FASTER_RADIX, u64>::new(
            prelude.tables.clone(),
            prelude.frame_sizes.clone(),
            folding_offset,
            folding_threshold,
        );

        Self::with_parameters(prelude, vec_model)
    }
}

/// Decoding functions.
impl<const FIDELITY: usize, const RADIX: usize, H, M, F> FoldedStreamANSDecoder<FIDELITY, RADIX, H, M, F>
where
    H: Quasi<RADIX>,
    M: Decode + SymbolLookup<State, Output = DecoderModelEntry<RADIX, H>>,
    F: Fold<RADIX>,
{

    pub fn decode(&mut self, model_index: usize) -> RawSymbol {
        let slot = self.state & self.model.get_frame_mask(model_index);
        let symbol_entry = self.model.symbol(slot, model_index);

        self.state = (self.state >> self.model.get_log2_frame_size(model_index))
            * (symbol_entry.freq as State) + slot as State
            - (symbol_entry.cumul_freq as State);

        if self.state < Self::LOWER_BOUND {
            let bits = self.normalized_bits.pop().unwrap();
            self.state = (self.state << LOG2_B) | bits as State;
        }

        self.folded_bits.unfold_symbol(symbol_entry.quasi_folded, &mut self.last_unfolded_pos)
    }
}
