use webgraph::prelude::BVGraphCodesReader;
use crate::multi_model_ans::encoder::ANSCompressorPhase;
use crate::multi_model_ans::model4decoder::VecFrame;
use crate::multi_model_ans::model4encoder::SymbolLookup;
use crate::multi_model_ans::Prelude;
use crate::traits::folding::FoldRead;
use crate::traits::quasi::{Decode, Quasi};
use crate::{DecoderModelEntry, RawSymbol, State, FASTER_RADIX, LOG2_B};
use crate::bvgraph::Component;

#[derive(Clone)]
pub struct ANSDecoder<
    'a,
    const FIDELITY: usize,
    const RADIX: usize = FASTER_RADIX,
    H = u64,
    M = VecFrame<RADIX, H>,
    F = Vec<u8>,
> where
    H: Quasi<RADIX>,
    M: Decode + SymbolLookup<State, Output = DecoderModelEntry<RADIX, H>>,
    F: FoldRead<RADIX>,
{
    pub model: &'a M,

    /// The normalized bits during the encoding process.
    pub normalized_bits: &'a Vec<u32>,

    /// The folded bits during the encoding process.
    pub folded_bits: &'a F,

    pub state: State,

    pub last_unfolded_pos: usize,

    pub last_normalized_pos: usize,
}

impl<'a, const FIDELITY: usize, const RADIX: usize, H, M, F>ANSDecoder<'a, FIDELITY, RADIX, H, M, F>
where
    H: Quasi<RADIX>,
    M: Decode + SymbolLookup<State, Output = DecoderModelEntry<RADIX, H>>,
    F: FoldRead<RADIX>,
{
    /// The lower bound of the interval.
    const LOWER_BOUND: State = 1 << 32;

    /// Creates a personalized FoldedStreamANSDecoder with the current values of `FIDELITY` and `RADIX`
    /// and the given model.
    pub fn with_parameters(prelude: &'a Prelude<RADIX, F>, model: &'a M) -> Self {
        Self {
            last_normalized_pos: prelude.normalized_bits.len(),
            last_unfolded_pos: prelude.folded_bits.len(),
            model,
            normalized_bits: &prelude.normalized_bits,
            folded_bits: &prelude.folded_bits,
            state: prelude.state,
        }
    }
}

impl<'a, const FIDELITY: usize> ANSDecoder <
    'a,
    FIDELITY,
    FASTER_RADIX,
    u64,
    VecFrame<FASTER_RADIX, u64>,
    Vec<u8>>
{
    /*
    /// Creates the standard FoldedStreamANSDecoder from the given parameters.
    ///
    /// The standard decoder uses fixed types for this struct's generics. This means that,
    /// by using this constructor, you're prevented from tuning any another parameter but fidelity.
    /// If you want to create a decoder with different components, you should use the [this](Self::with_parameters)
    pub fn new(prelude: &'a Prelude<FASTER_RADIX, Vec<u8>>) -> Self {
        let folding_offset = (1 << (FIDELITY - 1)) * ((1 << FASTER_RADIX) - 1);
        let folding_threshold = 1 << (FIDELITY + FASTER_RADIX - 1);

        let vec_model = VecFrame::<FASTER_RADIX, u64>::new(
            &prelude.tables,
            &prelude.frame_sizes,
            folding_offset,
            folding_threshold,
        );

        Self::with_parameters(&prelude, vec_model)
    }
    */

    pub fn from_raw_parts (
        prelude: &'a Prelude<FASTER_RADIX, Vec<u8>>,
        vec_model: &'a VecFrame<FASTER_RADIX, u64>,
        state: State,
        last_normalized_pos: usize,
        last_unfolded_pos: usize,
    )
        -> Self
    {
        Self {
            model: vec_model,
            normalized_bits: &prelude.normalized_bits,
            folded_bits: &prelude.folded_bits,
            state,
            last_normalized_pos,
            last_unfolded_pos,
        }
    }
}

/// Decoding functions.
impl<'a, const FIDELITY: usize, const RADIX: usize, H, M, F> ANSDecoder<'a, FIDELITY, RADIX, H, M, F>
where
    H: Quasi<RADIX>,
    M: Decode + SymbolLookup<State, Output = DecoderModelEntry<RADIX, H>>,
    F: FoldRead<RADIX>,
{
    pub fn decode(&mut self, model_index: usize) -> RawSymbol {
        let slot = self.state & self.model.get_frame_mask(model_index);
        let symbol_entry = self.model.symbol(slot, model_index);

        self.state = (self.state >> self.model.get_log2_frame_size(model_index))
            * (symbol_entry.freq as State)
            + slot as State
            - (symbol_entry.cumul_freq as State);

        if self.state < Self::LOWER_BOUND {
            self.last_normalized_pos -= 1;
            let bits = self.normalized_bits[self.last_normalized_pos];
            self.state = (self.state << LOG2_B) | bits as State;
        }

        self.folded_bits
            .unfold_symbol(symbol_entry.quasi_folded, &mut self.last_unfolded_pos)
    }

    pub fn decode_from_phase(
        &mut self,
        phase: ANSCompressorPhase,
        model_index: usize,
    ) -> RawSymbol {
        self.state = phase.state;
        self.last_unfolded_pos = phase.folded;
        self.last_normalized_pos = phase.normalized;

        Self::decode(self, model_index)
    }

    pub fn set_compressor_at_phase(&mut self, phase: &ANSCompressorPhase) {
        self.state = phase.state;
        self.last_unfolded_pos = phase.folded;
        self.last_normalized_pos = phase.normalized;
    }
}

impl <'a, const FIDELITY: usize> BVGraphCodesReader for ANSDecoder<
    'a,
    FIDELITY,
    FASTER_RADIX,
    u64,
    VecFrame<FASTER_RADIX, u64>,
    Vec<u8>>
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
