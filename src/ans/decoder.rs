use std::ops::Index;
use std::slice::Iter;

use bitvec::prelude::*;

use crate::ans::dec_model::{DecoderModelEntry, Rank9SelFrame};
use crate::{RawSymbol, State, LOG2_B, K_LOG2};
use crate::ans::{FASTER_RADIX, Prelude};


/// Mask used to extract the 48 LSB from `mapped_num`. This number will be the quasi-unfolded symbol.
const SYMBOL_MASK: u64 = 0x_FFFFFFFFFFFF;

/// How many bits are reserved to represent the quasi-unfolded symbol in `mapped_num`
const RESERVED_TO_SYMBOL: u8 = 48;

#[allow(clippy::len_without_is_empty)]
pub trait FoldedData {

    fn len(&self) -> usize;

    /// Unfolds a symbol from the given `mapped_num` and returns it.
    fn unfold_symbol(&self, mapped_num: u64, last_unfolded: &mut usize, radix: u8) -> RawSymbol;
}

impl FoldedData for BitVec<usize, Msb0> {

    fn len(&self) -> usize {
        self.len()
    }

    fn unfold_symbol(&self, mapped_num: u64, last_unfolded: &mut usize, radix: u8) -> RawSymbol {
        let folds = (mapped_num >> RESERVED_TO_SYMBOL) as usize;
        let quasi_unfolded = mapped_num & SYMBOL_MASK;
        let bits = self
            .as_bitslice()
            .get(*last_unfolded - folds * radix as usize..*last_unfolded)
            .unwrap();

        *last_unfolded -= folds * radix as usize;
        quasi_unfolded | bits.load_be::<RawSymbol>()
    }
}

impl FoldedData for Vec<u8> {

    fn len(&self) -> usize {
        self.len()
    }

    fn unfold_symbol(&self, mapped_num: u64, last_unfolded: &mut usize, _radix: u8) -> RawSymbol {
        let quasi_unfolded = mapped_num & SYMBOL_MASK;
        let folds = mapped_num >> RESERVED_TO_SYMBOL;
        let mut bytes = [0_u8; 8];

        bytes[8 - folds as usize..].copy_from_slice(&self[*last_unfolded - folds as usize..*last_unfolded]);
        *last_unfolded -= folds as usize;

        quasi_unfolded | u64::from_be_bytes(bytes)
    }
}


#[derive(Clone)]
pub struct FoldedStreamANSDecoder<const FIDELITY: u8, const RADIX: u8 = FASTER_RADIX,  M = Rank9SelFrame, F = Vec<u8>>
    where
        M: Index<State, Output = DecoderModelEntry>,
        F: FoldedData
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
    log2_frame_size: u8,

    /// The length of the sequence to decode.
    sequence_length: u64,

    /// The biggest singleton symbol, i.e. the biggest symbol that doesn't need to be folded.
    folding_threshold: u64,
}

impl <const FIDELITY: u8, const RADIX: u8, M, F> FoldedStreamANSDecoder<FIDELITY, RADIX, M, F>
    where
        M: Index<State, Output = DecoderModelEntry>,
        F: FoldedData
{
    /// Creates a FoldedStreamANSDecoder with the current values of `FIDELITY` and `RADIX` and the
    /// given model. Please note that this constructor will return a decoder that uses a BitVec as
    /// folded bits, which is way slower than the one that uses a Vec of bytes.
    pub fn with_parameters(mut prelude: Prelude<F>, model: M) -> Self {
        prelude.normalized_bits.reverse();

        Self {
            model,
            normalized_bits: prelude.normalized_bits,
            folded_bits: prelude.folded_bits,
            folding_threshold: (1 << (FIDELITY + RADIX - 1)) as RawSymbol,
            lower_bound: 1 << (prelude.log2_frame_size + K_LOG2),
            states: prelude.states,
            frame_mask: (1 << prelude.log2_frame_size) - 1,
            log2_frame_size: prelude.log2_frame_size,
            sequence_length: prelude.sequence_length,
        }
    }
}

impl <const FIDELITY: u8> FoldedStreamANSDecoder <FIDELITY, FASTER_RADIX, Rank9SelFrame, Vec<u8>> {

    /// Creates the standard FoldedStreamANSDecoder from the given parameters.
    ///
    /// The standard decoder uses fixed radix of 8 and a [`Rank9SelFrame`] as a frame. This means that,
    /// by using this constructor, you're prevented from tuning any another parameter but fidelity.
    /// If you want to create a decoder with different components, you should use the [this](Self::with_parameters)
    pub fn new(prelude: Prelude<Vec<u8>>) -> Self {
        let folding_offset = ((1 << (FIDELITY - 1)) * ((1 << FASTER_RADIX) - 1)) as RawSymbol;
        let folding_threshold = (1 << (FIDELITY + FASTER_RADIX - 1)) as RawSymbol;

        let frame = Rank9SelFrame::new(
            &prelude.table,
            prelude.log2_frame_size,
            folding_offset,
            folding_threshold,
            FASTER_RADIX
        );

        Self::with_parameters(
            prelude,
            frame,
        )
    }
}

/// Decoding functions.
impl <const FIDELITY: u8, const RADIX: u8, M, F> FoldedStreamANSDecoder<FIDELITY, RADIX, M, F>
    where
        M: Index<State, Output = DecoderModelEntry>,
        F: FoldedData
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
            decoded.push(self.decode_sym(&mut states[3], &mut norm_bits, &self.folded_bits, &mut last_unfolded_pos));
            decoded.push(self.decode_sym(&mut states[2], &mut norm_bits, &self.folded_bits, &mut last_unfolded_pos));
            decoded.push(self.decode_sym(&mut states[1], &mut norm_bits, &self.folded_bits, &mut last_unfolded_pos));
            decoded.push(self.decode_sym(&mut states[0], &mut norm_bits, &self.folded_bits, &mut last_unfolded_pos));
            current_symbol_index += 4;
        }

        while current_symbol_index < self.sequence_length {
            decoded.push(self.decode_sym(&mut states[0], &mut norm_bits, &self.folded_bits, &mut last_unfolded_pos));
            current_symbol_index += 1;
        }
        decoded
    }

    fn decode_sym(&self, state: &mut State, norm_bits_iter: &mut Iter<u32>, folded_bits: &F, last_unfolded_pos: &mut usize) -> RawSymbol {
        let slot = *state & self.frame_mask;
        let symbol_entry: &DecoderModelEntry = &self.model[slot as State];

        let decoded_sym = if (symbol_entry.symbol as RawSymbol) < self.folding_threshold {
            symbol_entry.symbol as RawSymbol
        } else {
            folded_bits.unfold_symbol(symbol_entry.mapped_num, last_unfolded_pos, RADIX)
        };

        *state = (*state >> self.log2_frame_size) * symbol_entry.freq as State
            + slot as State
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
