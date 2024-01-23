use webgraph::prelude::BVGraphCodesReader;

use crate::multi_model_ans::model4decoder::VecFrame;
use crate::multi_model_ans::model4encoder::SymbolLookup;
use crate::multi_model_ans::{ANSCompressorPhase, Prelude};
use crate::{RawSymbol, State, LOG2_B};
use crate::bvgraph::BVGraphComponent;

#[derive(Clone)]
pub struct ANSDecoder<'a> {
    /// The model used to decode the sequence.
    pub model: &'a VecFrame,

    /// The normalized bits during the encoding process.
    pub stream: &'a Vec<u32>,

    /// The current state of the decoder.
    pub state: State,

    /// The pointer to the next normalized chunk of 32 bits to be read.
    pub stream_pointer: usize,

    /// The value of fidelity currently used by the decoder.
    pub fidelity: usize,

    /// The value of radix currently used by the decoder.
    pub radix: usize,
}

impl<'a> ANSDecoder<'a>
{
    /// The lower bound of the interval.
    const LOWER_BOUND: State = 1 << 32;

    /// The number of bits reserved to represent the symbol in the quasi-folded value.
    const BIT_RESERVED_FOR_SYMBOL: u64 = 48;

    pub fn new(prelude: &'a Prelude, model: &'a VecFrame, fidelity: usize, radix: usize) -> Self {
        Self {
            stream_pointer: prelude.normalized_bits.len(),
            model,
            stream: &prelude.normalized_bits,
            state: prelude.state,
            fidelity,
            radix,
        }
    }

    /// Initialize a new ANSDecoder from its raw parts.
    ///
    /// Note: the next decoded symbol will be the last one encoded in the given [`phase`](ANSCompressorPhase)
    pub fn from_raw_parts(
        prelude: &'a Prelude,
        model: &'a VecFrame,
        phase: ANSCompressorPhase,
        fidelity: usize,
        radix: usize,
    ) -> Self
    {
        Self {
            model,
            stream: &prelude.normalized_bits,
            state: phase.state,
            stream_pointer: phase.stream_pointer,
            fidelity,
            radix,
        }
    }
}

impl<'a> ANSDecoder<'a> {

    /// Decodes a single symbol of a specific [`Component`](BVGraphComponent).
    pub fn decode(&mut self, component: BVGraphComponent) -> RawSymbol {
        let slot = self.state & self.model.get_frame_mask(component);
        let symbol_entry = self.model.symbol(slot, component);

        self.state = (self.state >> self.model.get_log2_frame_size(component))
            * (symbol_entry.freq as State)
            + slot as State
            - (symbol_entry.cumul_freq as State);

        if self.state < Self::LOWER_BOUND {
            self.extend_state();
        }

        let (quasi_unfolded, folds) = self.quasi_unfold(symbol_entry.quasi_folded);
        let mut fold = 0u64;

        for _ in 0..folds {
            if self.state < Self::LOWER_BOUND {
                self.extend_state();
            }
            fold = (fold << self.radix) | self.state & ((1 << self.radix) - 1);
            self.state >>= self.radix;

            if self.state < Self::LOWER_BOUND {
                self.extend_state();
            }
        }
        quasi_unfolded | fold
    }

    fn extend_state(&mut self) {
        self.stream_pointer -= 1;
        let bits = self.stream[self.stream_pointer];
        self.state = (self.state << LOG2_B) | bits as State;
    }

    fn quasi_unfold(&self, quasi_folded: u64) -> (u64, u32) {
        let symbol = quasi_folded & ((1 << Self::BIT_RESERVED_FOR_SYMBOL) - 1);
        let folds = quasi_folded >> Self::BIT_RESERVED_FOR_SYMBOL;
        (symbol, folds as u32)
    }
}

impl<'a> BVGraphCodesReader for ANSDecoder<'a> {
    fn read_outdegree(&mut self) -> u64 {
        self.decode(BVGraphComponent::Outdegree)
    }

    fn read_reference_offset(&mut self) -> u64 {
        self.decode(BVGraphComponent::ReferenceOffset)
    }

    fn read_block_count(&mut self) -> u64 {
        self.decode(BVGraphComponent::BlockCount)
    }

    fn read_blocks(&mut self) -> u64 {
        self.decode(BVGraphComponent::Blocks)
    }

    fn read_interval_count(&mut self) -> u64 {
        self.decode(BVGraphComponent::IntervalCount)
    }

    fn read_interval_start(&mut self) -> u64 {
        self.decode(BVGraphComponent::IntervalStart)
    }

    fn read_interval_len(&mut self) -> u64 {
        self.decode(BVGraphComponent::IntervalLen)
    }

    fn read_first_residual(&mut self) -> u64 {
        self.decode(BVGraphComponent::FirstResidual)
    }

    fn read_residual(&mut self) -> u64 {
        self.decode(BVGraphComponent::Residual)
    }
}
