use webgraph::prelude::Decode;

use crate::ans::models::model4decoder::ANSModel4Decoder;
use crate::bvgraph::BVGraphComponent;
use crate::{RawSymbol, State, Symbol, B};

#[derive(Clone)]
pub struct ANSDecoder<'a> {
    /// The model used to decode the sequence.
    pub model: &'a ANSModel4Decoder,

    /// The normalized bits during the encoding process.
    pub stream: &'a Vec<u16>,

    /// The current state of the decoder.
    pub state: State,

    /// The index of the next normalized 32-bits chunk to be read.
    pub stream_pointer: usize,
}

impl<'a> ANSDecoder<'a> {
    /// The lower bound of the interval.
    const LOWER_BOUND: State = 1 << 16;

    /// The number of bits reserved to represent the symbol in the quasi-folded value.
    const BIT_RESERVED_FOR_SYMBOL: u64 = 48;

    pub fn new(model: &'a ANSModel4Decoder, stream: &'a Vec<u16>, state: State) -> Self {
        Self {
            model,
            stream,
            state,
            stream_pointer: stream.len(),
        }
    }

    /// Initialize a new ANSDecoder from its raw parts.
    ///
    /// Note: the next decoded symbol will be the __last__ encoded in the given state. If the decoder
    /// will need to expand the state with bits in the stream, it will pull chunks starting from the
    /// `stream_pointer` index.
    pub fn from_raw_parts(
        model: &'a ANSModel4Decoder,
        stream: &'a Vec<u16>,
        stream_pointer: usize,
        state: State,
    ) -> Self {
        Self {
            model,
            stream,
            stream_pointer,
            state,
        }
    }
}

impl<'a> ANSDecoder<'a> {
    /// Decodes a single symbol of a specific [`Component`](BVGraphComponent).
    pub fn decode(&mut self, component: BVGraphComponent) -> RawSymbol {
        let slot = self.state & self.model.get_frame_mask(component) as State;
        let symbol_entry = self.model.symbol(slot as Symbol, component);

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
            fold = (fold << self.model.get_radix(component))
                | (self.state as u64) & ((1 << self.model.get_radix(component)) - 1);
            self.state >>= self.model.get_radix(component);

            if self.state < Self::LOWER_BOUND {
                self.extend_state();
            }
        }
        quasi_unfolded | fold
    }

    fn extend_state(&mut self) {
        self.stream_pointer -= 1;
        let bits = self.stream[self.stream_pointer];
        self.state = (self.state << B) | bits as State;
    }

    #[inline(always)]
    fn quasi_unfold(&self, quasi_folded: u64) -> (u64, u32) {
        let symbol = quasi_folded & ((1 << Self::BIT_RESERVED_FOR_SYMBOL) - 1);
        let folds = quasi_folded >> Self::BIT_RESERVED_FOR_SYMBOL;
        (symbol, folds as u32)
    }
}

impl<'a> Decode for ANSDecoder<'a> {
    fn read_outdegree(&mut self) -> u64 {
        self.decode(BVGraphComponent::Outdegree)
    }

    fn read_reference_offset(&mut self) -> u64 {
        self.decode(BVGraphComponent::ReferenceOffset)
    }

    fn read_block_count(&mut self) -> u64 {
        self.decode(BVGraphComponent::BlockCount)
    }

    fn read_block(&mut self) -> u64 {
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
