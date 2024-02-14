use crate::bvgraph::BVGraphComponent;
use crate::multi_model_ans::model4encoder::ANSModel4Encoder;
use crate::multi_model_ans::{ANSCompressorPhase, Prelude};
use crate::{RawSymbol, State, Symbol, B, INTERVAL_LOWER_BOUND, NORMALIZATION_MASK};

#[derive(Clone)]
pub struct ANSEncoder {
    /// The model used by the ANS encoder to encode symbols coming from every [component](BVGraphComponent).
    pub model: ANSModel4Encoder,

    /// The normalized bits during the encoding process.
    pub stream: Vec<u32>,

    pub state: State,
}

impl ANSEncoder {
    pub fn new(model: ANSModel4Encoder) -> Self {
        Self {
            state: INTERVAL_LOWER_BOUND,
            model,
            stream: Vec::new(),
        }
    }

    fn get_folds_number(&self, symbol: RawSymbol, component: BVGraphComponent) -> usize {
        ((u64::ilog2(symbol) + 1) as usize - self.model.get_fidelity(component))
            / self.model.get_radix(component)
    }
}

impl ANSEncoder {
    /// Encodes a single symbol of a specific [`Component`](BVGraphComponent).
    ///
    /// Note that the ANS decodes the sequence in reverse order.
    pub fn encode(&mut self, mut symbol: RawSymbol, component: BVGraphComponent) {
        // if symbol has to be folded, dump the bytes we have to fold
        if symbol >= self.model.get_folding_threshold(component) {
            let folds = self.get_folds_number(symbol, component);

            for _ in 0..folds {
                let bits_to_push = symbol & ((1 << self.model.get_radix(component)) - 1);

                // dump in the state if there is enough space
                if self.state.leading_zeros() >= self.model.get_radix(component) as u32 {
                    self.state <<= self.model.get_radix(component);
                    self.state += bits_to_push;
                } else {
                    // otherwise, normalize the state and push the bits in the normalized bits
                    self.state = Self::shrink_state(self.state, &mut self.stream);
                    self.state <<= self.model.get_radix(component);
                    self.state += bits_to_push;
                }
                symbol >>= self.model.get_radix(component);
            }
            symbol += self.model.get_folding_offset(component) * folds as RawSymbol;
        }
        let symbol = symbol as Symbol;
        let sym_data = self.model.symbol(symbol, component);

        if self.state >= sym_data.upperbound {
            self.state = Self::shrink_state(self.state, &mut self.stream);
        }

        let block = self.state / sym_data.freq as u64;

        self.state = block * sym_data.comp_freq as u64 + sym_data.cumul_freq as u64 + self.state;
    }

    fn shrink_state(mut state: State, out: &mut Vec<u32>) -> State {
        let lsb = (state & NORMALIZATION_MASK) as u32;
        out.push(lsb);
        state >>= B;
        state
    }

    pub fn into_prelude(self) -> Prelude {
        Prelude {
            tables: self.model.component_models,
            stream: self.stream,
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
