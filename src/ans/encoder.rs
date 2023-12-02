use crate::{K_LOG2, LOG2_B, RawSymbol, State, Symbol};
use crate::ans::enc_model::FoldedANSModel4Encoder;
use crate::ans::{FASTER_RADIX, Prelude};
use crate::ans::traits::Foldable;

/// Used to extract the 32 LSB from a 64-bit state.
const NORMALIZATION_MASK: u64 = 0xFFFFFFFF;


#[derive(Clone)]
pub struct FoldedStreamANSCoder<'a, const FIDELITY: u8, const RADIX: u8 = FASTER_RADIX, F = Vec<u8>>
    where
        F:  Foldable + Default + Clone
{
    model: FoldedANSModel4Encoder,

    states: [u64; 4],

    /// The normalized bits during the encoding process.
    normalized_bits: Vec<u32>,

    /// The folded bits during the encoding process for those symbols which are bucketed.
    folded_bits: F,

    /// Original sequence of symbols.
    input_sequence: &'a Vec<RawSymbol>,

    /// The biggest singleton symbol, i.e. the biggest symbol that doesn't need to be folded.
    folding_threshold: RawSymbol,
}

impl <'a, const FIDELITY: u8, const RADIX: u8, F> FoldedStreamANSCoder<'a, FIDELITY, RADIX, F>
    where
        F: Foldable + Default + Clone
{
    /// Creates a FoldedStreamANSEncoder with the current values of `FIDELITY` and `RADIX` and the
    /// given model. Please note that this constructor will return a decoder that uses a BitVec as
    /// folded bits, which is way slower than the one that uses a Vec of bytes.
    pub fn with_parameters(input: &'a Vec<RawSymbol>, folded_bits: F) -> Self {
        Self {
            states: [0; 4], // wasting 64 bits for each state
            model: FoldedANSModel4Encoder::new(input, RADIX, FIDELITY),
            normalized_bits: Vec::new(),
            folded_bits,
            input_sequence: input,
            folding_threshold: 1 << (RADIX + FIDELITY - 1),
        }
    }
}

impl <'a, const FIDELITY: u8> FoldedStreamANSCoder<'a, FIDELITY, FASTER_RADIX, Vec<u8>> {

    /// Creates the standard FoldedStreamANSEncoder from the given parameters.
    ///
    /// The standard decoder uses fixed radix of 8. This means that, by using this
    /// constructor, you're prevented from tuning any another parameter but fidelity.
    /// If you want to create a decoder with different components, you should use the [this](Self::with_parameters)
    pub fn new(input: &'a Vec<RawSymbol>) -> Self {
        Self::with_parameters(input, Vec::new())
    }
}

/// Encoding functions
impl <'a, const FIDELITY: u8, const RADIX: u8, F> FoldedStreamANSCoder<'a, FIDELITY, RADIX, F>
    where
        F: Foldable + Default + Clone
{
    /// Encodes the whole input sequence.
    ///
    /// # Note
    /// In order to give priority to the decoding process, this function will encode the sequence in
    /// reverse order.
    pub fn encode_all(&mut self) {
        let mut states = [1_u64 << (self.model.log2_frame_size + K_LOG2); 4];
        let mut folded_bits = F::default();
        let mut normalized_bits = Vec::with_capacity(self.input_sequence.len());

        let symbols_iter = self.input_sequence.chunks_exact(4);
        let symbols_left = symbols_iter.remainder();

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

    fn encode_symbol(&self, symbol: RawSymbol, mut state: State, normalized_bits: &mut Vec<u32>, folded_bits: &mut F) -> State {
        let symbol = if symbol < self.folding_threshold { symbol as Symbol } else {
            folded_bits.fold_symbol(symbol, RADIX, FIDELITY)
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

    fn shrink_state(mut state: State, out: &mut Vec<u32>) -> State {
        let lsb = (state & NORMALIZATION_MASK) as u32;
        out.push(lsb);
        state >>= LOG2_B;
        state
    }

    pub fn serialize(&mut self) -> Prelude<F> {
        Prelude {
            table: self.model.to_raw_parts(),
            sequence_length: self.input_sequence.len() as u64,
            normalized_bits: self.normalized_bits.clone(),
            folded_bits: self.folded_bits.clone(),
            log2_frame_size: self.model.log2_frame_size,
            states: self.states,
        }
    }
}

