use crate::ans::enc_model::{AnsModel4Encoder, SymbolLookup};
use crate::ans::traits::Fold;
use crate::ans::{Prelude, FASTER_RADIX};
use crate::{RawSymbol, State, Symbol, LOG2_B};
use crate::ans::traits::Decode;


/// Used to extract the 32 LSB from a 64-bit state.
const NORMALIZATION_MASK: u64 = 0xFFFFFFFF;

#[derive(Clone)]
pub struct FoldedStreamANSCoder <const FIDELITY: usize, const RADIX: usize = FASTER_RADIX, F = Vec<u8>>
    where
        F: Fold<RADIX> + Default + Clone,
{
    model: AnsModel4Encoder,

    pub state: State,

    /// The normalized bits during the encoding process.
    pub normalized_bits: Vec<u32>,

    /// The folded bits during the encoding process for those symbols which are bucketed.
    pub folded_bits: F,
}

impl<const FIDELITY: usize, const RADIX: usize, F> FoldedStreamANSCoder<FIDELITY, RADIX, F>
    where
        F: Fold<RADIX> + Default + Clone,
{
    /// The biggest singleton symbol, i.e. the biggest symbol that doesn't need to be folded.
    const FOLDING_THRESHOLD: RawSymbol = (1 << (FIDELITY + RADIX - 1)) as RawSymbol;

    /// Creates a FoldedStreamANSEncoder with the current values of `FIDELITY` and `RADIX` and the
    /// given model. Please note that this constructor will return a decoder that uses a BitVec as
    /// folded bits, which is way slower than the one that uses a Vec of bytes.
    pub fn with_parameters(model: AnsModel4Encoder, folded_bits: F) -> Self {
        Self {
            state: 1_u64 << 32,
            model,
            normalized_bits: Vec::new(),
            folded_bits,
        }
    }
}

impl<const FIDELITY: usize> FoldedStreamANSCoder<FIDELITY, FASTER_RADIX, Vec<u8>> {

    /// Creates the standard FoldedStreamANSEncoder from the given parameters.
    ///
    /// The standard decoder uses fixed radix of 8. This means that, by using this
    /// constructor, you're prevented from tuning any another parameter but fidelity.
    /// If you want to create a decoder with different components, you should use the [this](Self::with_parameters)
    pub fn new(model: AnsModel4Encoder) -> Self {
        Self::with_parameters(model, Vec::new())
    }
}

/// Encoding functions
impl<const FIDELITY: usize, const RADIX: usize, F> FoldedStreamANSCoder<FIDELITY, RADIX, F>
    where
        F: Fold<RADIX> + Default + Clone,
{

    /// Encodes a single symbol by using the data in the model with the given index.
    ///
    /// Note that the ANS decodes the sequence in reverse order.
    pub fn encode(&mut self, symbol: RawSymbol, model_index: usize) {
        let symbol = if symbol < Self::FOLDING_THRESHOLD {
            symbol as Symbol
        } else {
            self.folded_bits.fold_symbol(symbol, FIDELITY)
        };

        let sym_data = self.model.symbol(symbol, model_index);

        if self.state >= sym_data.upperbound {
            self.state = Self::shrink_state(self.state, &mut self.normalized_bits);
        }

        let block = self.state / sym_data.fast_divisor;

        self.state = (block << self.model.get_log2_frame_size(model_index))
            + sym_data.cumul_freq as u64
            + (self.state - (block * sym_data.freq as u64))
    }

    fn shrink_state(mut state: State, out: &mut Vec<u32>) -> State {
        let lsb = (state & NORMALIZATION_MASK) as u32;
        out.push(lsb);
        state >>= LOG2_B;
        state
    }

    pub fn serialize(&mut self) -> Prelude<RADIX, F> {
        Prelude {
            tables: self.model.tables.clone(),
            normalized_bits: self.normalized_bits.clone(),
            folded_bits: self.folded_bits.clone(),
            frame_sizes: self.model.frame_sizes.clone(),
            state: self.state,
        }
    }
}
