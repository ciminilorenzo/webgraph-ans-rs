use webgraph::prelude::BVGraphCodesReader;
use crate::multi_model_ans::encoder::ANSCompressorPhase;
use crate::multi_model_ans::model4decoder::VecFrame;
use crate::multi_model_ans::model4encoder::SymbolLookup;
use crate::multi_model_ans::Prelude;
use crate::traits::quasi::{Decode, Quasi};
use crate::{DecoderModelEntry, RawSymbol, State, FASTER_RADIX, LOG2_B};
use crate::bvgraph::Component;

#[derive(Clone)]
pub struct ANSDecoder<'a, const FIDELITY: usize, const RADIX: usize = FASTER_RADIX, H = u64, M = VecFrame<FASTER_RADIX, H>>
where
    H: Quasi<RADIX>,
    M: Decode + SymbolLookup<State, Output = DecoderModelEntry<RADIX, H>>,
{
    pub model: &'a M,

    /// The normalized bits during the encoding process.
    pub normalized_bits: &'a Vec<u32>,

    pub state: State,

    pub normalized_pointer: usize,
}

impl<'a, const FIDELITY: usize, const RADIX: usize, H, M> ANSDecoder<'a, FIDELITY, RADIX, H, M>
where
    H: Quasi<RADIX>,
    M: Decode + SymbolLookup<State, Output = DecoderModelEntry<RADIX, H>>,
{
    /// The lower bound of the interval.
    const LOWER_BOUND: State = 1 << 32;

    pub fn new(prelude: &'a Prelude<RADIX>, model: &'a M) -> Self {
        Self {
            normalized_pointer: prelude.normalized_bits.len(),
            model,
            normalized_bits: &prelude.normalized_bits,
            state: prelude.state,
        }
    }

    pub fn from_raw_parts (prelude: &'a Prelude<RADIX>, model: &'a M, phase: ANSCompressorPhase) -> Self {
        Self {
            model,
            normalized_bits: &prelude.normalized_bits,
            state: phase.state,
            normalized_pointer: phase.normalized,
        }
    }
}

impl<'a, const FIDELITY: usize, const RADIX: usize, H, M> ANSDecoder<'a, FIDELITY, RADIX, H, M>
where
    H: Quasi<RADIX>,
    M: Decode + SymbolLookup<State, Output = DecoderModelEntry<RADIX, H>>,
{
    pub fn decode(&mut self, model_index: usize) -> RawSymbol {
        let slot = self.state & self.model.get_frame_mask(model_index);
        let symbol_entry = self.model.symbol(slot, model_index);

        self.state = (self.state >> self.model.get_log2_frame_size(model_index))
            * (symbol_entry.freq as State)
            + slot as State
            - (symbol_entry.cumul_freq as State);

        if self.state < Self::LOWER_BOUND {
            self.extend_state();
        }

        let (quasi_unfolded, folds) = H::quasi_unfold(symbol_entry.quasi_folded);
        let mut fold = 0u64;

        for _ in 0..folds {
            if self.state < Self::LOWER_BOUND {
                self.extend_state();
            }
            fold = (fold << RADIX) | self.state & ((1 << RADIX) - 1);
            self.state >>= RADIX;

            if self.state < Self::LOWER_BOUND {
                self.extend_state();
            }
        }
        quasi_unfolded.into() | fold
    }

    fn extend_state(&mut self) {
        self.normalized_pointer -= 1;
        let bits = self.normalized_bits[self.normalized_pointer];
        self.state = (self.state << LOG2_B) | bits as State;
    }

    /*
    pub fn decode_from_phase(
        &mut self,
        phase: ANSCompressorPhase,
        model_index: usize,
    ) -> RawSymbol {
        self.state = phase.state;
        self.last_unfolded_pos = phase.folded;
        self.normalized_pointer = phase.normalized;

        Self::decode(self, model_index)
    }

    pub fn set_compressor_at_phase(&mut self, phase: &ANSCompressorPhase) {
        self.state = phase.state;
        self.last_unfolded_pos = phase.folded;
        self.normalized_pointer = phase.normalized;
    }
    */
}

impl<'a, const FIDELITY: usize, const RADIX: usize, H, M> BVGraphCodesReader for ANSDecoder<'a, FIDELITY, RADIX, H, M>
where
    H: Quasi<RADIX>,
    M: Decode + SymbolLookup<State, Output = DecoderModelEntry<RADIX, H>>,
{
    fn read_outdegree(&mut self) -> u64 {
        self.decode(Component::Outdegree as usize)
    }

    fn read_reference_offset(&mut self) -> u64 {
        self.decode(Component::ReferenceOffset as usize)
    }

    fn read_block_count(&mut self) -> u64 {
        self.decode(Component::BlockCount as usize)
    }

    fn read_blocks(&mut self) -> u64 {
        self.decode(Component::Blocks as usize)
    }

    fn read_interval_count(&mut self) -> u64 {
        self.decode(Component::IntervalCount as usize)
    }

    fn read_interval_start(&mut self) -> u64 {
        self.decode(Component::IntervalStart as usize)
    }

    fn read_interval_len(&mut self) -> u64 {
        self.decode(Component::IntervalLen as usize)
    }

    fn read_first_residual(&mut self) -> u64 {
        self.decode(Component::FirstResidual as usize)
    }

    fn read_residual(&mut self) -> u64 {
        self.decode(Component::Residual as usize)
    }
}
