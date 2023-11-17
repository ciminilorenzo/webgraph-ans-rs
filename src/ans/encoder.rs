use bitvec::prelude::*;

use crate::{K_LOG2, LOG2_B, RawSymbol, State, Symbol};
use crate::ans::enc_model::FoldedANSModel4Encoder;
use crate::ans::{EncoderModelEntry};
use crate::ans::ans_util::fold_symbol;


// Used to extract the least significant 32 bits from a 64-bit state.
const MASK: u64 = 0xFFFFFFFF;


/// # Folded Streaming ANS-based Encoder
/// A streaming ANS-based encoder which uses the folded symbols technique in order to reduce the size
/// of the alphabet by using the technique called "symbol folding" (from Moffat and Petri's
/// [paper](https://dl.acm.org/doi/10.1145/3397175)).
///
/// ### STRUCT'S CONSTANTS
/// User of this struct can tune the parameters `RADIX` and `FIDELITY` in order to change how the
/// symbols folding is performed.
///
/// #### RADIX
/// For convenience, this value is intended to be the log_2 of the radix parameter as introduced in
/// the [paper](https://dl.acm.org/doi/10.1145/3397175). In other word, radix is 2^RADIX.
///
/// #### FIDELITY
/// to write
#[readonly::make]
pub struct FoldedStreamANSCoder<const RADIX: u8, const FIDELITY: u8> {

    pub model: FoldedANSModel4Encoder,

    state: [u64; 4],

    /// The normalized bits during the encoding process.
    normalized_bits: BitVec,

    /// The folded bits during the encoding process for those symbols which are bucketed.
    folded_bits: BitVec::<usize, Msb0>,

    /// Original sequence of symbols.
    input_sequence: Vec<RawSymbol>,

    /// The biggest singleton symbol, i.e. the biggest unfolded symbol.
    folding_threshold: RawSymbol,
}

impl <const RADIX: u8, const FIDELITY: u8> FoldedStreamANSCoder<RADIX, FIDELITY> {

    /// Creates an Encoder from a sequence of [`RawSymbol`]s;
    ///
    /// # Panics
    /// If either the input sequence is empty or is not possibile to approximated the folded symbols'
    /// distribution with a common denominator lower or equal than 2^28.
    pub fn new(input: Vec<RawSymbol>) -> Self {
        assert!(! input.is_empty(), "A non-empty sequence must be provided!");

        let model = FoldedANSModel4Encoder::new(&input, RADIX, FIDELITY);

        Self {
            state: [0; 4], // wasting 64 bits for each state
            model,
            normalized_bits: BitVec::new(),
            folded_bits: BitVec::new(),
            input_sequence: input,
            folding_threshold: 1 << (RADIX + FIDELITY - 1),
        }
    }

    /// Encodes the whole input sequence.
    ///
    /// # Note
    /// In order to give priority to the decoding process, this function will encode the sequence in
    /// reverse order.
    pub fn encode_all(&mut self) {
        let symbols_iter = self.input_sequence.chunks_exact(4);
        let symbols_left = symbols_iter.remainder();
        let mut normalized_bits = BitVec::new();
        let mut folded_bits = BitVec::<usize, Msb0>::new();
        let mut states = [1_u64 << (self.model.log2_frame_size + K_LOG2); 4];

        for symbol in symbols_left.iter().rev() {
            states[0] = self.encode_symbol(*symbol, states[0], &mut normalized_bits, &mut folded_bits);
        }

        symbols_iter.rev().for_each(|chunk| {
            states[0] = self.encode_symbol(chunk[3], states[0], &mut normalized_bits, &mut folded_bits);
            states[1] = self.encode_symbol(chunk[2], states[1], &mut normalized_bits, &mut folded_bits);
            states[2] = self.encode_symbol(chunk[1], states[2], &mut normalized_bits, &mut folded_bits);
            states[3] = self.encode_symbol(chunk[0], states[3], &mut normalized_bits, &mut folded_bits);
        });

        self.state = states;
        self.normalized_bits = normalized_bits;
        self.folded_bits = folded_bits;
    }

    fn encode_symbol(&self, symbol: RawSymbol, mut state: State, normalized_bits: &mut BitVec, folded_bits: &mut BitVec::<usize, Msb0>) -> State {
        let symbol = if symbol < self.folding_threshold { symbol as Symbol } else {
            fold_symbol(symbol, true, Some(folded_bits), RADIX, FIDELITY)
        };

        let sym_data = &self.model[symbol];

        if state >= sym_data.upperbound {
            state = Self::shrink_state(state, normalized_bits);
        }

        let block = state / sym_data.freq as u64;

        (block << self.model.log2_frame_size)
            + sym_data.cumul_freq as u64
            + (state - (block * sym_data.freq as u64))
    }

    fn shrink_state(mut state: State, out: &mut BitVec) -> State {
        let lsb = (state & MASK) as u32;
        out.extend(lsb.view_bits::<Lsb0>());
        state >>= LOG2_B;
        state
    }

    pub fn serialize(&mut self) -> (u64, Vec<EncoderModelEntry>, [State; 4], u8, BitVec, BitVec::<usize, Msb0>) {
        (
            self.input_sequence.len() as u64,
            self.model.to_raw_parts(),
            self.state,
            self.model.log2_frame_size,
            self.normalized_bits.clone(),
            self.folded_bits.clone()
        )
    }
}