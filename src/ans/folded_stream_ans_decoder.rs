use std::ops::Index;

use bitvec::field::BitField;
use bitvec::vec::BitVec;

use crate::{K_LOG2, LOG2_B, RawSymbol, State};
use crate::ans::ans_util::undo_fold_symbol;
use crate::ans::decoder_model::DecoderModelEntry;

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
pub struct FoldedStreamANSDecoder<const RADIX: u8, const FIDELITY: u8, T> where
    T: Index<State, Output = DecoderModelEntry>
{
    model: T,

    /// The normalized bits during the encoding process.
    normalized_bits: BitVec,

    /// The folded bits during the encoding process.
    folded_bits: BitVec,

    /// The lower bound of the interval.
    lower_bound: State,

    state: State,

    /// Mask used to extract, from the current state, the frame's slot in which the current state falls.
    frame_mask: usize,

    /// Logarithm (base 2) of the frame size.
    log2_frame_size: u8,
}

impl <const RADIX: u8, const FIDELITY: u8, T> FoldedStreamANSDecoder<RADIX, FIDELITY, T> where
    T: Index<State, Output = DecoderModelEntry>
{

    pub fn new (state: State, model: T, log2_frame_size: u8, normalized_bits: BitVec, folded_bits: BitVec, ) -> Self {
        Self {
            model,
            normalized_bits,
            folded_bits,
            lower_bound: 1 << (log2_frame_size + K_LOG2),
            state,
            frame_mask: (1 << log2_frame_size) - 1,
            log2_frame_size,
        }
    }

    /// Decodes the whole sequence given as input.
    pub fn decode_all(&mut self) -> Vec<RawSymbol> {
        let mut decoded = Vec::new(); // if we save the size of the encoded list, we can preallocate the right amount of space!

        loop {
            let slot = self.state & self.frame_mask as State;
            let symbol_entry: &DecoderModelEntry = &self.model[slot as State];

            decoded.push(undo_fold_symbol(symbol_entry.symbol, RADIX, FIDELITY, &mut self.folded_bits));

            self.state = (self.state >> self.log2_frame_size) * symbol_entry.freq as State + slot as State - symbol_entry.cumul_freq as State;

            if self.state <= self.lower_bound {
                if self.normalized_bits.is_empty() { break; }

                // TODO: try to bench with https://docs.rs/bitvec/1.0.1/bitvec/slice/struct.BitSlice.html#method.windows instead of this
                let bits = self.normalized_bits.drain(self.normalized_bits.len() - LOG2_B as usize..).collect::<BitVec>();
                self.state = ((self.state << LOG2_B) | bits.load::<State>()) as State;
            }
        }
        decoded
    }

    pub fn from_raw_parts() {
        // !!!  Creates a new instance by using data directly shaped as needed by an ad-hoc input reader !!!
        // !!!  that reads encoder output and feeds it to the decoder as needed.                         !!!
        todo!()
    }
}

