use crate::bvgraph::BVGraphComponent;
use crate::multi_model_ans::model4encoder::{ANSModel4Encoder, SymbolLookup};
use crate::multi_model_ans::{ANSCompressorPhase, Prelude};
use crate::{RawSymbol, State, Symbol, LOG2_B};

/// Used to extract the 32 LSB from a 64-bit state.
const NORMALIZATION_MASK: u64 = 0xFFFFFFFF;

#[derive(Clone)]
pub struct ANSEncoder {
    pub model: ANSModel4Encoder,

    pub state: State,

    /// The normalized bits during the encoding process.
    pub stream: Vec<u32>,

    /// Represent the threshold starting from which a symbol has to be folded.
    pub folding_threshold: u64,

    pub folding_offset: u64,

    pub radix: usize,

    pub fidelity: usize,
}

impl ANSEncoder {

    pub fn new(model: ANSModel4Encoder, fidelity: usize, radix: usize) -> Self {
        Self {
            state: 1_u64 << 32,
            model,
            stream: Vec::new(),
            folding_threshold: (1 << (fidelity + radix - 1)) as u64,
            folding_offset: ((1 << radix) - 1) * (1 << (fidelity - 1)),
            radix,
            fidelity,
        }
    }

    fn get_folds_number(&self, symbol: RawSymbol) -> usize {
        ((u64::ilog2(symbol) + 1) as usize - self.fidelity) / self.radix
    }
}

impl ANSEncoder {
    /// Encodes a single symbol of a specific [`Component`](BVGraphComponent).
    ///
    /// Note that the ANS decodes the sequence in reverse order.
    pub fn encode(&mut self, mut symbol: RawSymbol, component: BVGraphComponent) {
        // if symbol has to be folded, dump the bytes we have to fold
        if symbol >= self.folding_threshold {
            let folds = self.get_folds_number(symbol);

            for _ in 0..folds {
                let bits_to_push = symbol & ((1 << self.radix) - 1);

                // dump in the state if there is enough space
                if self.state.leading_zeros() >= self.radix as u32 {
                    self.state <<= self.radix;
                    self.state += bits_to_push;
                } else { // otherwise, normalize the state and push the bits in the normalized bits
                    self.state = Self::shrink_state(self.state, &mut self.stream);
                    self.state <<= self.radix;
                    self.state += bits_to_push;
                }
                symbol >>= self.radix;
            }
            symbol += self.folding_offset * folds as RawSymbol;
        }
        let symbol = symbol as Symbol;
        let sym_data = self.model.symbol(symbol, component);

        if self.state >= sym_data.upperbound {
            self.state = Self::shrink_state(self.state, &mut self.stream);
        }

        let block = self.state / sym_data.freq as u64;

        self.state = (block << self.model.get_log2_frame_size(component))
            + sym_data.cumul_freq as u64
            + (self.state - (block * sym_data.freq as u64));
    }

    fn shrink_state(mut state: State, out: &mut Vec<u32>) -> State {
        let lsb = (state & NORMALIZATION_MASK) as u32;
        out.push(lsb);
        state >>= LOG2_B;
        state
    }

    pub fn serialize(self) -> Prelude {
        Prelude {
            tables: self.model.tables,
            normalized_bits: self.stream,
            frame_sizes: self.model.frame_sizes,
            state: self.state,
        }
    }

    /// Returns the current phase of the compressor, that is: the current state and the index of the last chunk of 32 bits
    /// that have been normalized.
    ///
    /// An [`ANSCompressorPhase`] can be utilized to restore the state of the compressor at a given point in time. In the
    /// specific, if the compressor actual phase is `phase`, then the next decode symbol will be the same as the one
    /// that led the compressor to the phase `phase`.
    pub fn get_current_compressor_phase(&self) -> ANSCompressorPhase {
        ANSCompressorPhase {
            state: self.state,
            stream_pointer: self.stream.len(),
        }
    }
}