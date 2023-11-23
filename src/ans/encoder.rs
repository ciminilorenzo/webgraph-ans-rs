use std::usize;

use bitvec::prelude::*;

use crate::{K_LOG2, LOG2_B, RawSymbol, State, Symbol};
use crate::ans::enc_model::FoldedANSModel4Encoder;
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
#[derive(Clone)]
pub struct FoldedStreamANSCoder<const RADIX: u8, const FIDELITY: u8> {

    pub model: FoldedANSModel4Encoder,

    states: [u64; 4],

    /// The normalized bits during the encoding process.
    normalized_bits: BitVec,

    /// The folded bits during the encoding process for those symbols which are bucketed.
    folded_bits: BitVec<usize, Msb0>,

    /// Original sequence of symbols.
    input_sequence: Vec<RawSymbol>,

    /// The biggest singleton symbol, i.e. the biggest symbol that doesn't need to be folded.
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
            states: [0; 4], // wasting 64 bits for each state
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

        self.states = states;
        self.normalized_bits = normalized_bits;
        self.folded_bits = folded_bits;
    }

    fn encode_symbol(&self, symbol: RawSymbol, mut state: State, normalized_bits: &mut BitVec, folded_bits: &mut BitVec<usize, Msb0>) -> State {
        let symbol = if symbol < self.folding_threshold { symbol as Symbol } else {
            Self::fold_symbol(symbol, folded_bits, RADIX, FIDELITY)
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

    #[inline]
    fn shrink_state(mut state: State, out: &mut BitVec) -> State {
        let lsb = (state & MASK) as u32;
        out.extend(lsb.view_bits::<Lsb0>());
        state >>= LOG2_B;
        state
    }

    /// Performs the so called 'symbol folding'. This optimized implementation is different
    /// from the one described in the paper since here the while loop is avoided in favour of a single
    /// block of operations that performs the same task.
    ///
    /// # Panics
    /// - if the caller wants to stream bits out even though no BitVec reference is provided;
    /// - if the folded symbol is bigger than u16::MAX.
    fn fold_symbol(mut symbol: RawSymbol, out: &mut BitVec<usize, Msb0>, radix: u8, fidelity: u8) -> Symbol {
        let mut offset = 0;
        let cuts = ((f64::log2(symbol as f64).floor() + 1_f64) - fidelity as f64) / radix as f64;
        let bit_to_cut = cuts as u8 * radix;

        out.extend_from_bitslice(symbol
            .view_bits::<Msb0>()
            .split_at(RawSymbol::BITS as usize - bit_to_cut as usize).1
        );

        symbol >>= bit_to_cut;
        offset += (((1 << radix) - 1) * (1 << (fidelity - 1))) * cuts as RawSymbol;
        (symbol + offset) as u16
    }

    pub fn serialize(&mut self) -> (u64, Vec<EncoderModelEntry>, [State; 4], u8, BitVec, BitVec<usize, Msb0>) {
        (
            self.input_sequence.len() as u64,
            self.model.to_raw_parts(),
            self.states,
            self.model.log2_frame_size,
            self.normalized_bits.clone(),
            self.folded_bits.clone()
        )
    }
}