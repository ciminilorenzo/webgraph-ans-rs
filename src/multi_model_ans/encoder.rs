use epserde::Epserde;
use mem_dbg::{MemDbg, MemSize};
use crate::multi_model_ans::model4encoder::{ANSModel4Encoder, SymbolLookup};
use crate::multi_model_ans::Prelude;
use crate::traits::folding::{FoldRead, FoldWrite};
use crate::traits::quasi::Decode;
use crate::{RawSymbol, State, Symbol, FASTER_RADIX, LOG2_B};

/// Used to extract the 32 LSB from a 64-bit state.
const NORMALIZATION_MASK: u64 = 0xFFFFFFFF;

#[derive(Clone)]
pub struct ANSEncoder<
    const FIDELITY: usize,
    const RADIX: usize = FASTER_RADIX,
    F: FoldWrite<RADIX> + Default + Clone = Vec<u8>,
> {
    model: ANSModel4Encoder,

    pub state: State,

    /// The normalized bits during the encoding process.
    pub normalized_bits: Vec<u32>,

    /// The folded bits during the encoding process for those symbols which are bucketed.
    pub folded_bits: F,
}

impl<const FIDELITY: usize, const RADIX: usize, F> ANSEncoder<FIDELITY, RADIX, F>
where
    F: FoldWrite<RADIX> + Default + Clone,
{
    /// The biggest singleton symbol, i.e. the biggest symbol that doesn't need to be folded.
    const FOLDING_THRESHOLD: RawSymbol = (1 << (FIDELITY + RADIX - 1)) as RawSymbol;

    /// Creates a FoldedStreamANSEncoder with the current values of `FIDELITY` and `RADIX` and the
    /// given model. Please note that this constructor will return a decoder that uses a BitVec as
    /// folded bits, which is way slower than the one that uses a Vec of bytes.
    pub fn with_parameters(model: ANSModel4Encoder, folded_bits: F) -> Self {
        Self {
            state: 1_u64 << 32,
            model,
            normalized_bits: Vec::new(),
            folded_bits,
        }
    }
}

impl<const FIDELITY: usize> ANSEncoder<FIDELITY, FASTER_RADIX, Vec<u8>> {
    /// Creates the standard FoldedStreamANSEncoder from the given parameters.
    ///
    /// The standard decoder uses fixed radix of 8. This means that, by using this
    /// constructor, you're prevented from tuning any another parameter but fidelity.
    /// If you want to create a decoder with different components, you should use the [this](Self::with_parameters)
    pub fn new(model: ANSModel4Encoder) -> Self {
        Self::with_parameters(model, Vec::new())
    }
}

/// Encoding functions
impl<const FIDELITY: usize, const RADIX: usize, F> ANSEncoder<FIDELITY, RADIX, F>
where
    F: FoldWrite<RADIX> + FoldRead<RADIX> + Default + Clone,
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

        let block = self.state / sym_data.freq as u64;

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

    /// Returns the current phase of the compressor, that is: the current state, the index of the last chunk of 32 bits
    /// that have been normalized, and the index of the last chunk of [`RADIX`] bits that have been folded.
    ///
    /// An [`ANSCompressorPhase`] can be utilized to restore the state of the compressor at a given point in time. In the
    /// specific, if the compressor actual phase is `phase`, then the next decode symbol will be the same as the one
    /// that led the compressor to the phase `phase`.
    pub fn get_current_compressor_phase(&self) -> ANSCompressorPhase {
        ANSCompressorPhase {
            state: self.state,
            normalized: self.normalized_bits.len(),
            folded: self.folded_bits.len(),
        }
    }
}

#[derive(Debug, Clone, Copy, Epserde, MemDbg, MemSize)]
#[zero_copy]
#[repr(C)]
pub struct ANSCompressorPhase {
    pub state: State,
    pub normalized: usize,
    pub folded: usize,
}
