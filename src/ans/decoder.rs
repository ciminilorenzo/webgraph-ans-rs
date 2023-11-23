use std::mem;
use std::ops::Index;

use bitvec::prelude::*;
use bitvec::slice::{RChunks};

use crate::ans::dec_model::{DecoderModelEntry, VecFrame};
use crate::{RawSymbol, State, K_LOG2, LOG2_B};
use crate::ans::EncoderModelEntry;

/// Mask that extract the 16 MSB from `mapped_num`. This number will be the number of folds to unfold
/// the symbol with.
const FOLDS_MASK: u64 = 0x_FFFF000000000000;

/// Mask used the 48 LSB from `mapped_num`. This number will be the quasi-unfolded symbol.
const SYMBOL_MASK: u64 = 0x_FFFFFFFFFFFF;

/// How many bits are reserved to represent the quasi-unfolded symbol in `mapped_num`
const RESERVED_TO_SYMBOL: u8 = 48;


/// # Folded Streaming ANS-based Encoder
/// A streaming ANS-based decoder which uses the technique called "symbol folding" (from Moffat and Petri's
/// [paper](https://dl.acm.org/doi/10.1145/3397175)) in order to reduce the size of the alphabet.
///
/// ### STRUCT'S CONSTANTS
/// Users of this struct can tune the parameters `RADIX` and `FIDELITY` in order to change how the
/// symbols folding is performed.
///
/// #### RADIX
/// For convenience, this value is intended to be the log_2 of the radix parameter as described in
/// the paper.
///
/// #### FIDELITY
/// to write
#[derive(Clone)]
pub struct FoldedStreamANSDecoder<const RADIX: u8, const FIDELITY: u8, T>
    where
        T: Index<State, Output = DecoderModelEntry>
{
    model: T,

    /// The normalized bits during the encoding process.
    normalized_bits: BitVec,

    /// The folded bits during the encoding process.
    folded_bits: BitVec<usize, Msb0>,

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

impl<const RADIX: u8, const FIDELITY: u8> FoldedStreamANSDecoder<RADIX, FIDELITY, VecFrame> {

    /// Creates a new FoldedStreamANSDecoder from the given parameters.
    ///
    /// # Note
    /// By default, this constructor creates a new instance by using a [`VecFrame`] as a frame. This
    /// means that the frame is not space-efficient, but it's the fastest one. Thus, if you want to create
    /// the model with a more space-efficient frame, you should use the [this](Self::with_frame) constructor.
    pub fn new (table: &[EncoderModelEntry], log2_frame_size: u8, states: [State; 4], normalized_bits: BitVec, folded_bits: BitVec<usize, Msb0>, sequence_length: u64) -> Self {
        let folding_offset = ((1 << (FIDELITY - 1)) * ((1 << RADIX) - 1)) as RawSymbol;
        let folding_threshold = (1 << (FIDELITY + RADIX - 1)) as RawSymbol;
        let model_with_vec = VecFrame::new(table, log2_frame_size, folding_offset, folding_threshold, RADIX);

        Self {
            model: model_with_vec,
            normalized_bits,
            folded_bits,
            folding_threshold,
            lower_bound: 1 << (log2_frame_size + K_LOG2),
            states,
            frame_mask: (1 << log2_frame_size) - 1,
            log2_frame_size,
            sequence_length,
        }
    }
}

impl<const RADIX: u8, const FIDELITY: u8, T> FoldedStreamANSDecoder<RADIX, FIDELITY, T>
    where
        T: Index<State, Output = DecoderModelEntry>
{

    /// Creates a new FoldedStreamANSDecoder from the given parameters.
    pub fn with_frame(sequence_length: u64, states: [State; 4], model: T, log2_frame_size: u8, normalized_bits: BitVec, folded_bits: BitVec::<usize, Msb0>) -> Self {
        Self {
            model,
            normalized_bits,
            folded_bits,
            folding_threshold: (1 << (FIDELITY + RADIX - 1)) as RawSymbol,
            lower_bound: 1 << (log2_frame_size + K_LOG2),
            states,
            frame_mask: (1 << log2_frame_size) - 1,
            log2_frame_size,
            sequence_length,
        }
    }

    /*

    let slice = bits![0,0,1,1,0,0,1,1];
    println!("original slice: {}", slice);

    let mut last_unfolded_bit_pos = slice.len();

    println!("{}", slice.get(last_unfolded_bit_pos-2..last_unfolded_bit_pos).unwrap());
    last_unfolded_bit_pos -= 2;

    println!("{}", slice.get(last_unfolded_bit_pos-2..last_unfolded_bit_pos).unwrap());
    last_unfolded_bit_pos -= 2;
     */

    /// Decodes the whole sequence given as input.
    pub fn decode_all(&mut self) -> Vec<RawSymbol> {
        let mut decoded = Vec::with_capacity(self.sequence_length as usize);
        let norm_bits = mem::take(&mut self.normalized_bits);
        let mut norm_chunks = norm_bits.rchunks(LOG2_B as usize);
        let folded_bits = self.folded_bits.as_bitslice();
        let mut last_unfolded_pos = folded_bits.len();

        let threshold = self.sequence_length - (self.sequence_length % 4);
        let mut current_symbol_index = 0;

        while current_symbol_index < threshold {
            for state_index in (0..self.states.len()).rev() {
                let (sym, new_state) = self.decode_sym(self.states[state_index], &mut norm_chunks, folded_bits, &mut last_unfolded_pos);
                decoded.push(sym);
                self.states[state_index] = new_state;
                current_symbol_index += 1;
            }
        }

        while current_symbol_index < self.sequence_length {
            let (sym, new_state) = self.decode_sym(self.states[0], &mut norm_chunks, folded_bits, &mut last_unfolded_pos);
            decoded.push(sym);
            self.states[0] = new_state;
            current_symbol_index += 1;
        }
        decoded
    }

    fn decode_sym(
        &self,
        mut state: State,
        normalized_bits_iter: &mut RChunks<usize, Lsb0>,
        folded_bits: &BitSlice<usize, Msb0>,
        last_unfolded_bit_pos: &mut usize
    ) -> (RawSymbol, State)
    {
        let slot = state & self.frame_mask;
        let symbol_entry: &DecoderModelEntry = &self.model[slot as State];

        state = (state >> self.log2_frame_size) * symbol_entry.freq as State
            + slot as State
            - symbol_entry.cumul_freq as State;

        (state < self.lower_bound).then(|| state = Self::expand_state(state, normalized_bits_iter));

        let decoded_sym = if (symbol_entry.symbol as RawSymbol) < self.folding_threshold {
            symbol_entry.symbol as RawSymbol
        } else {
            let folds = (symbol_entry.mapped_num & FOLDS_MASK) >> RESERVED_TO_SYMBOL;
            let quasi_unfolded = symbol_entry.mapped_num & SYMBOL_MASK;

            let bits = folded_bits
                .get(*last_unfolded_bit_pos - folds as usize * RADIX as usize..*last_unfolded_bit_pos)
                .unwrap();

            *last_unfolded_bit_pos -= folds as usize * RADIX as usize;

            quasi_unfolded | bits.load_be::<RawSymbol>()
        };

        (decoded_sym, state)
    }

    #[inline]
    fn expand_state(state: State, normalized_bits: &mut RChunks<usize, Lsb0>) -> State {
        let bits = normalized_bits.next().unwrap();
        ((state << LOG2_B) | bits.load::<State>()) as State
    }
}
