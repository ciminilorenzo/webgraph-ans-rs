use crate::ans::model4encoder::ANSModel4Encoder;
use crate::ans::{ANSCompressorPhase, Prelude};
use crate::bvgraph::BVGraphComponent;
use crate::{RawSymbol, State, B, INTERVAL_LOWER_BOUND, NORMALIZATION_MASK};

#[cfg(feature = "arm")]
use crate::Freq;

#[derive(Clone)]
pub struct ANSEncoder {
    /// The model used by the ANS encoder to encode symbols coming from every [component](BVGraphComponent).
    pub model: ANSModel4Encoder,

    /// The normalized bits during the encoding process.
    pub stream: Vec<u32>,

    /// The state of the encoder.
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

    #[inline(always)]
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
        let sym_data = self.model.symbol(symbol, component);

        if self.state >= sym_data.upperbound {
            self.state = Self::shrink_state(self.state, &mut self.stream);
        }

        #[cfg(feature = "arm")]
        self.calculate_new_state(
            sym_data.freq,
            sym_data.cumul_freq,
            self.model.get_log2_frame_size(component),
        );

        #[cfg(not(feature = "arm"))]
        self.calculate_new_state(
            sym_data.cmpl_freq,
            sym_data.reciprocal,
            sym_data.magic,
            sym_data.cumul_freq,
        );
    }

    #[inline(always)]
    #[cfg(not(feature = "arm"))]
    fn calculate_new_state(&mut self, cmpl_freq: u16, rcp: u64, magic: u8, cumul: u16) {
        let block = ((rcp as u128 * (self.state as u128 + (magic & 1) as u128)) >> 64) as u64
            >> (magic >> 1);

        self.state += block * cmpl_freq as u64 + cumul as u64;
    }

    #[inline(always)]
    #[cfg(feature = "arm")]
    fn calculate_new_state(&mut self, freq: Freq, cumul: Freq, frame_size: usize) {
        let block = self.state / freq as u64;

        self.state = (block << frame_size) + cumul as u64 + (self.state - (block * freq as u64))
    }

    #[inline(always)]
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
