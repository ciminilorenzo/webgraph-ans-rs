use bitvec::prelude::*;

use crate::{K_LOG2, LOG2_B, RawSymbol, State};
use crate::ans::ans_util::fold_symbol;
use crate::ans::encoder_model::FoldedANSModel4Encoder;
use crate::ans::{EncoderModelEntry};


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

    state: State,

    /// The normalized bits during the encoding process.
    normalized_bits: BitVec,

    /// The folded bits during the encoding process for those symbols which are bucketed.
    folded_bits: BitVec,

    /// Original sequence of symbols.
    input_sequence: Vec<RawSymbol>
}

impl <const RADIX: u8, const FIDELITY: u8> FoldedStreamANSCoder<RADIX, FIDELITY> {

    /// Creates an Encoder from a sequence of [`RawSymbol`]s;
    ///
    /// # Panics
    /// If either the input sequence is empty or is not possibile to approximated the folded symbols'
    /// distribution with a common denominator lower or equal than 2^28.
    pub fn new(input: Vec<RawSymbol>) -> Self {
        assert!(! input.is_empty(), "A non-empty sequence must be provided!");

        Self {
            model: FoldedANSModel4Encoder::new(&input, RADIX, FIDELITY),
            state: 0,
            normalized_bits: BitVec::new(),
            folded_bits: BitVec::new(),
            input_sequence: input,
        }
    }

    /// Encodes the whole input sequence.
    ///
    /// # Note
    /// In order to give priority to the decoding process, this
    /// function will encode the sequence in reverse order.
    pub fn encode_all(&mut self) {
        self.state = 1 << (self.model.log2_frame_size + K_LOG2);

        for i in (0.. self.input_sequence.len()).rev() {
            self.state = self.encode_symbol(self.input_sequence[i], self.state);
        }
    }

    fn encode_symbol(&mut self, symbol: RawSymbol, mut state: State) -> State {
        let folded_symbol = fold_symbol(symbol, true, Some(&mut self.folded_bits), RADIX, FIDELITY);
        let sym_data = &self.model[folded_symbol];

        if state >= sym_data.upperbound {
            state = Self::shrink_state(state, self.normalized_bits.as_mut());
        }

        let block = state / sym_data.freq as u64;
        (block << self.model.log2_frame_size) + sym_data.cumul_freq as u64 + (state - (block * sym_data.freq as u64))
    }

    fn shrink_state(mut state: State, out: &mut BitVec) -> State {
        let lsb = (state & MASK) as u32;
        out.extend(lsb.view_bits::<Lsb0>());
        state >>= LOG2_B;
        state
    }

    // TODO: this will write in the output stream
    pub fn serialize(&mut self) -> (Vec<EncoderModelEntry>, State, u8, BitVec, BitVec) {
        (self.model.to_raw_parts(), self.state, self.model.log2_frame_size, self.normalized_bits.clone(), self.folded_bits.clone())
    }
}