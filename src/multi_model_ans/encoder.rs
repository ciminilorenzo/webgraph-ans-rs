use epserde::Epserde;
use mem_dbg::{MemDbg, MemSize};
use crate::multi_model_ans::model4encoder::{ANSModel4Encoder, SymbolLookup};
use crate::multi_model_ans::Prelude;
use crate::traits::quasi::Decode;
use crate::{RawSymbol, State, Symbol, FASTER_RADIX, LOG2_B};

/// Used to extract the 32 LSB from a 64-bit state.
const NORMALIZATION_MASK: u64 = 0xFFFFFFFF;

#[derive(Clone)]
pub struct ANSEncoder<const FIDELITY: usize, const RADIX: usize = FASTER_RADIX> {
    pub model: ANSModel4Encoder,

    pub state: State,

    /// The normalized bits during the encoding process.
    pub normalized_bits: Vec<u32>,
}

impl<const FIDELITY: usize, const RADIX: usize> ANSEncoder<FIDELITY, RADIX> {
    /// The biggest singleton symbol, i.e. the biggest symbol that doesn't need to be folded.
    const FOLDING_THRESHOLD: RawSymbol = (1 << (FIDELITY + RADIX - 1)) as RawSymbol;

    const FOLDING_OFFSET: RawSymbol = ((1 << RADIX) - 1) * (1 << (FIDELITY - 1));

    pub fn new(model: ANSModel4Encoder) -> Self {
        Self {
            state: 1_u64 << 32,
            model,
            normalized_bits: Vec::new(),
        }
    }

    fn get_folds_number(symbol: RawSymbol) -> usize {
        ((u64::ilog2(symbol) + 1) as usize - FIDELITY) / RADIX
    }
}

/// Encoding functions
impl<const FIDELITY: usize, const RADIX: usize> ANSEncoder<FIDELITY, RADIX> {
    /// Encodes a single symbol by using the data in the model with the given index.
    ///
    /// Note that the ANS decodes the sequence in reverse order.
    pub fn encode(&mut self, mut symbol: RawSymbol, model_index: usize) {
        // if symbol has to be folded, dump the bytes we have to fold
        if symbol >= Self::FOLDING_THRESHOLD {
            let folds = Self::get_folds_number(symbol);

            for _ in 0..folds {
                let bits_to_push = symbol & ((1 << RADIX) - 1);

                // dump in the space if there is enough space
                if self.state.leading_zeros() >= RADIX as u32 {
                    self.state <<= RADIX;
                    self.state += bits_to_push;
                }
                // otherwise, normalize the state and push the bits in the normalized bits
                else {
                    self.state = Self::shrink_state(self.state, &mut self.normalized_bits);
                    self.state <<= RADIX;
                    self.state += bits_to_push;
                }
                symbol >>= RADIX;
            }
            symbol += Self::FOLDING_OFFSET * folds as RawSymbol;
        }
        let symbol = symbol as Symbol;
        let sym_data = self.model.symbol(symbol, model_index);

        if self.state >= sym_data.upperbound {
            self.state = Self::shrink_state(self.state, &mut self.normalized_bits);
        }

        let block = self.state / sym_data.freq as u64;

        self.state = (block << self.model.get_log2_frame_size(model_index))
            + sym_data.cumul_freq as u64
            + (self.state - (block * sym_data.freq as u64));
    }

    fn shrink_state(mut state: State, out: &mut Vec<u32>) -> State {
        let lsb = (state & NORMALIZATION_MASK) as u32;
        out.push(lsb);
        state >>= LOG2_B;
        state
    }

    pub fn serialize(&mut self) -> Prelude<RADIX> {
        Prelude {
            tables: self.model.tables.clone(),
            normalized_bits: self.normalized_bits.clone(),
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
        }
    }
}

#[derive(Debug, Clone, Copy, Epserde, MemDbg, MemSize)]
#[zero_copy]
#[repr(C)]
pub struct ANSCompressorPhase {
    pub state: State,
    pub normalized: usize,
}
