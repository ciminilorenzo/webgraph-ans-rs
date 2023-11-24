use std::mem;
use std::ops::Index;

use bitvec::prelude::*;
use bitvec::slice::{RChunks};

use crate::ans::dec_model::{DecoderModelEntry, Rank9SelFrame};
use crate::{RawSymbol, State, K_LOG2, LOG2_B};
use crate::ans::EncoderModelEntry;

/// Mask used to extract the 16 MSB from `mapped_num`. This number will be the number of folds to unfold
/// the symbol with.
const FOLDS_MASK: u64 = 0x_FFFF000000000000;

/// Mask used to extract the 48 LSB from `mapped_num`. This number will be the quasi-unfolded symbol.
const SYMBOL_MASK: u64 = 0x_FFFFFFFFFFFF;

/// How many bits are reserved to represent the quasi-unfolded symbol in `mapped_num`
const RESERVED_TO_SYMBOL: u8 = 48;


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

impl<const RADIX: u8, const FIDELITY: u8> FoldedStreamANSDecoder<RADIX, FIDELITY, Rank9SelFrame> {

    /// Creates a new FoldedStreamANSDecoder from the given parameters.
    ///
    /// # Note
    /// By default, this constructor creates a new instance by using a [`Rank9SelFrame`] as a frame.
    /// If you want to create the model with a different frame, you should use the [this](Self::with_frame)
    /// constructor.
    pub fn new (table: &[EncoderModelEntry], log2_frame_size: u8, states: [State; 4], normalized_bits: BitVec, folded_bits: BitVec<usize, Msb0>, sequence_length: u64) -> Self {
        let folding_offset = ((1 << (FIDELITY - 1)) * ((1 << RADIX) - 1)) as RawSymbol;
        let folding_threshold = (1 << (FIDELITY + RADIX - 1)) as RawSymbol;
        let model_with_vec = Rank9SelFrame::new(table, log2_frame_size, folding_offset, folding_threshold, RADIX);

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

    // here i'm using ::take since using self.normalize_bits to construct the iterator would mean
    // that the iterator would have a mutable reference to self. This would not allow me to call
    // decode_sym later on (since it takes another mutable reference to self).
    /// Decodes the whole sequence given as input.
    pub fn decode_all(&mut self) -> Vec<RawSymbol> {
        let mut decoded = Vec::with_capacity(self.sequence_length as usize);
        let norm_bits = mem::take(&mut self.normalized_bits);
        let mut norm_chunks = norm_bits.rchunks(LOG2_B as usize);
        let folded_bits_binding = mem::take(&mut self.folded_bits);
        let folded_bits = folded_bits_binding.as_bitslice();
        let mut last_unfolded_pos = folded_bits.len();
        let threshold = self.sequence_length - (self.sequence_length % 4);
        let mut current_symbol_index = 0;

        while current_symbol_index < threshold {
            decoded.push(self.decode_sym(3_usize, &mut norm_chunks, folded_bits, &mut last_unfolded_pos));
            decoded.push(self.decode_sym(2_usize, &mut norm_chunks, folded_bits, &mut last_unfolded_pos));
            decoded.push(self.decode_sym(1_usize, &mut norm_chunks, folded_bits, &mut last_unfolded_pos));
            decoded.push(self.decode_sym(0_usize, &mut norm_chunks, folded_bits, &mut last_unfolded_pos));
            current_symbol_index += 4;
        }

        while current_symbol_index < self.sequence_length {
            decoded.push(self.decode_sym(0_usize, &mut norm_chunks, folded_bits, &mut last_unfolded_pos));
            current_symbol_index += 1;
        }

        decoded
    }

    fn decode_sym(&mut self, state_index: usize, norm_chunks: &mut RChunks<usize, Lsb0>, folded_bits: &BitSlice<usize, Msb0>, last_unfolded_pos: &mut usize) -> RawSymbol {
        let slot = self.states[state_index] & self.frame_mask;
        let symbol_entry: &DecoderModelEntry = &self.model[slot as State];

        let decoded_sym = if (symbol_entry.symbol as RawSymbol) < self.folding_threshold {
            symbol_entry.symbol as RawSymbol
        } else {
            Self::unfold_symbol(symbol_entry.mapped_num, folded_bits, last_unfolded_pos)
        };

        self.states[state_index] = (self.states[state_index] >> self.log2_frame_size) * symbol_entry.freq as State
            + slot as State
            - symbol_entry.cumul_freq as State;

        if self.states[state_index] < self.lower_bound {
            self.states[state_index] = Self::shrink_state(self.states[state_index], norm_chunks);
        }

        decoded_sym
    }

    /// Divides the given u64 into two parts: the 16 MSB and the 48 LSB. The 16 MSB will be the number of
    /// folds of [`RADIX`] bits to retrieve from the folded bits to correctly unfold the symbol, while the
    /// 48 LSB will be the quasi-unfolded symbol.
    fn unfold_symbol(mapped_num: u64, folded_bits: &BitSlice<usize, Msb0>, last_unfolded_pos: &mut usize) -> RawSymbol {
        let folds = (mapped_num & FOLDS_MASK) >> RESERVED_TO_SYMBOL;
        let quasi_unfolded = mapped_num & SYMBOL_MASK;
        let bits = folded_bits
            .get(*last_unfolded_pos - folds as usize * RADIX as usize..*last_unfolded_pos)
            .unwrap();

        // let's keep an index that keeps track of the position of the last last unfolded bit. This
        // is needed since we don't know at priori how many bits we have to unfold (so we can't use
        // a method like chunks).
        *last_unfolded_pos -= folds as usize * RADIX as usize;
        quasi_unfolded | bits.load_be::<RawSymbol>()
    }

    #[must_use]
    fn shrink_state(state: State, norm_chunks: &mut RChunks<usize, Lsb0>) -> State {
        let bits = norm_chunks.next().unwrap();
        ((state << LOG2_B) | bits.load::<State>()) as State
    }
}
