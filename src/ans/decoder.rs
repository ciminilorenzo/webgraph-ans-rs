use std::ops::Index;
use bitvec::field::BitField;
use bitvec::order::{Lsb0, Msb0};
use bitvec::slice::RChunks;

use bitvec::vec::BitVec;

use crate::ans::ans_util::undo_fold_symbol;
use crate::ans::dec_model::{DecoderModelEntry, VecFrame};
use crate::{RawSymbol, State, K_LOG2, LOG2_B};
use crate::ans::EncoderModelEntry;


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
pub struct FoldedStreamANSDecoder<const RADIX: u8, const FIDELITY: u8, T>
    where
        T: Index<State, Output = DecoderModelEntry>
{
    model: T,

    /// The normalized bits during the encoding process.
    normalized_bits: BitVec,

    /// The folded bits during the encoding process.
    folded_bits: BitVec::<usize, Msb0>,

    /// The lower bound of the interval.
    lower_bound: State,

    states: [State; 4],

    /// Mask used to extract, from the current state, the frame's slot in which the current state falls.
    frame_mask: usize,

    /// Logarithm (base 2) of the frame size.
    log2_frame_size: u8,

    /// The length of the sequence to decode.
    sequence_length: u64,
}

impl<const RADIX: u8, const FIDELITY: u8> FoldedStreamANSDecoder<RADIX, FIDELITY, VecFrame> {

    /// Creates a new FoldedStreamANSDecoder from the given parameters.
    ///
    /// # Note
    /// By default, this constructor creates a new instance by using a [`VecFrame`] as a frame. This
    /// means that the frame is not space-efficient, but it's the fastest one. Thus, if you want to create
    /// the model with a more space-efficient frame, you should use the [this](Self::with_frame) constructor.
    pub fn new (
        table: &[EncoderModelEntry],
        log2_frame_size: u8,
        states: [State; 4],
        normalized_bits: BitVec,
        folded_bits: BitVec::<usize, Msb0>,
        sequence_length: u64,
    ) -> Self
    {
        let model = VecFrame::new(table, log2_frame_size);

        Self {
            model,
            normalized_bits,
            folded_bits,
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
            lower_bound: 1 << (log2_frame_size + K_LOG2),
            states,
            frame_mask: (1 << log2_frame_size) - 1,
            log2_frame_size,
            sequence_length,
        }
    }

    /// Decodes the whole sequence given as input.
    pub fn decode_all(&mut self) -> Vec<RawSymbol> {
        let mut decoded_sym = Vec::with_capacity(self.sequence_length as usize);
        let binding = self.normalized_bits.clone(); // TODO: avoid clone?
        let mut iter= binding.rchunks(LOG2_B as usize);
        let threshold = self.sequence_length - (self.sequence_length % 4);
        let mut current_symbol_index = 0;

        while current_symbol_index < threshold {
            for state_index in (0..self.states.len()).rev() {
                let (sym, new_state) = self.decode_sym(self.states[state_index], &mut iter);
                decoded_sym.push(sym);
                self.states[state_index] = new_state;
                current_symbol_index += 1;
            }
        }

        while current_symbol_index < self.sequence_length {
            let (sym, new_state) = self.decode_sym(self.states[0], &mut iter);
            decoded_sym.push(sym);
            self.states[0] = new_state;
            current_symbol_index += 1;
        }
        decoded_sym
    }

    fn decode_sym(&mut self, mut state: State, normalized_bits_iter: &mut RChunks<usize, Lsb0>) -> (RawSymbol, State) {
        let slot = state & self.frame_mask as State;
        let symbol_entry: &DecoderModelEntry = &self.model[slot as State];

        state = (state >> self.log2_frame_size) * symbol_entry.freq as State
            + slot as State
            - symbol_entry.cumul_freq as State;

        if state < self.lower_bound {
            let bits = normalized_bits_iter.next().unwrap();
            state = ((state << LOG2_B) | bits.load::<State>()) as State;
        }

        (undo_fold_symbol(symbol_entry.symbol, RADIX, FIDELITY, &mut self.folded_bits), state)
    }

    pub fn from_raw_parts() {
        // !!!  Creates a new instance by using data directly shaped as needed by an ad-hoc input reader !!!
        // !!!  that reads encoder output and feeds it to the decoder as needed.                         !!!
        todo!()
    }
}
