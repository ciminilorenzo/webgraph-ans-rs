use std::any::type_name;
use std::ops::Index;
use std::slice::Iter;

use crate::ans::dec_model::{DecoderModelEntry, Rank9SelFrame};
use crate::ans::traits::Foldable;
use crate::ans::{Prelude, FASTER_RADIX};
use crate::{RawSymbol, State, K_LOG2, LOG2_B};

#[derive(Clone)]
pub struct FoldedStreamANSDecoder<
    const FIDELITY: usize,
    const RADIX: usize = FASTER_RADIX,
    M = Rank9SelFrame,
    F = Vec<u8>,
> where
    M: Index<State, Output = DecoderModelEntry>,
    F: Foldable<RADIX>,
{
    model: M,

    /// The normalized bits during the encoding process.
    normalized_bits: Vec<u32>,

    /// The folded bits during the encoding process.
    folded_bits: F,

    /// The lower bound of the interval.
    lower_bound: State,

    states: [State; 4],

    /// Mask used to extract, from the current state, the frame's slot in which the current state falls.
    frame_mask: u64,

    /// Logarithm (base 2) of the frame size.
    log2_frame_size: usize,

    /// The length of the sequence to decode.
    sequence_length: u64,

    /// The biggest singleton symbol, i.e. the biggest symbol that doesn't need to be folded.
    folding_threshold: u16,
}

impl<const FIDELITY: usize, const RADIX: usize, M, F> FoldedStreamANSDecoder<FIDELITY, RADIX, M, F>
where
    M: Index<State, Output = DecoderModelEntry>,
    F: Foldable<RADIX>,
{
    /// Creates a FoldedStreamANSDecoder with the current values of `FIDELITY` and `RADIX` and the
    /// given model. Please note that this constructor will return a decoder that uses a BitVec as
    /// folded bits, which is way slower than the one that uses a Vec of bytes.
    pub fn with_parameters(mut prelude: Prelude<RADIX, F>, model: M) -> Self {
        if RADIX != 8 {
            assert_eq!(
                type_name::<F>(),
                "bitvec::vec::BitVec<usize, bitvec::order::Msb0>"
            )
        }

        prelude.normalized_bits.reverse();

        Self {
            model,
            normalized_bits: prelude.normalized_bits,
            folded_bits: prelude.folded_bits,
            folding_threshold: (1 << (FIDELITY + RADIX - 1)) as u16,
            lower_bound: 1 << (prelude.log2_frame_size + K_LOG2),
            states: prelude.states,
            frame_mask: (1 << prelude.log2_frame_size) - 1,
            log2_frame_size: prelude.log2_frame_size,
            sequence_length: prelude.sequence_length,
        }
    }
}

impl<const FIDELITY: usize> FoldedStreamANSDecoder<FIDELITY, FASTER_RADIX, Rank9SelFrame, Vec<u8>> {
    /// Creates the standard FoldedStreamANSDecoder from the given parameters.
    ///
    /// The standard decoder uses fixed radix of 8 and a [`Rank9SelFrame`] as a frame. This means that,
    /// by using this constructor, you're prevented from tuning any another parameter but fidelity.
    /// If you want to create a decoder with different components, you should use the [this](Self::with_parameters)
    pub fn new(prelude: Prelude<8, Vec<u8>>) -> Self {
        let folding_offset = ((1 << (FIDELITY - 1)) * ((1 << FASTER_RADIX) - 1)) as RawSymbol;
        let folding_threshold = (1 << (FIDELITY + FASTER_RADIX - 1)) as RawSymbol;

        let frame = Rank9SelFrame::new(
            &prelude.table,
            prelude.log2_frame_size,
            folding_offset,
            folding_threshold,
            FASTER_RADIX,
        );

        Self::with_parameters(prelude, frame)
    }
}

/// Decoding functions.
impl<const FIDELITY: usize, const RADIX: usize, M, F> FoldedStreamANSDecoder<FIDELITY, RADIX, M, F>
where
    M: Index<State, Output = DecoderModelEntry>,
    F: Foldable<RADIX>,
{
    /// Decodes the whole sequence given as input.
    pub fn decode_all(&self) -> Vec<RawSymbol> {
        let mut decoded = Vec::with_capacity(self.sequence_length as usize);
        let mut norm_bits = self.normalized_bits.iter();
        let mut last_unfolded_pos = self.folded_bits.len();
        let threshold = self.sequence_length - (self.sequence_length % 4);
        let mut states = self.states;

        let mut current_symbol_index = 0;

        while current_symbol_index < threshold {
            decoded.push(self.decode_sym(&mut states[3], &mut norm_bits, &mut last_unfolded_pos));
            decoded.push(self.decode_sym(&mut states[2], &mut norm_bits, &mut last_unfolded_pos));
            decoded.push(self.decode_sym(&mut states[1], &mut norm_bits, &mut last_unfolded_pos));
            decoded.push(self.decode_sym(&mut states[0], &mut norm_bits, &mut last_unfolded_pos));
            current_symbol_index += 4;
        }

        while current_symbol_index < self.sequence_length {
            decoded.push(self.decode_sym(&mut states[0], &mut norm_bits, &mut last_unfolded_pos));
            current_symbol_index += 1;
        }
        decoded
    }

    fn decode_sym(
        &self,
        state: &mut State,
        norm_bits_iter: &mut Iter<u32>,
        last_unfolded_pos: &mut usize,
    ) -> RawSymbol {
        let slot = *state & self.frame_mask;
        let symbol_entry: &DecoderModelEntry = &self.model[slot as State];

        let decoded_sym = if symbol_entry.symbol < self.folding_threshold {
            symbol_entry.symbol as RawSymbol
        } else {
            self.folded_bits
                .unfold_symbol(symbol_entry.mapped_num, last_unfolded_pos)
        };

        *state = (*state >> self.log2_frame_size) * symbol_entry.freq as State + slot as State
            - symbol_entry.cumul_freq as State;

        if *state < self.lower_bound {
            *state = Self::expand_state(*state, norm_bits_iter);
        }

        decoded_sym
    }

    fn expand_state(state: State, norm_bits: &mut Iter<u32>) -> State {
        let bits = norm_bits.next().unwrap();
        (state << LOG2_B) | *bits as State
    }
}
