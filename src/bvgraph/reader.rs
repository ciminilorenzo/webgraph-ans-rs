use std::error::Error;

use webgraph::prelude::{BVGraphCodesReader, BVGraphCodesReaderBuilder};

use crate::bvgraph::Component;
use crate::multi_model_ans::decoder::ANSDecoder;
use crate::multi_model_ans::encoder::ANSCompressorPhase;
use crate::multi_model_ans::model4decoder::VecFrame;
use crate::multi_model_ans::model4encoder::SymbolLookup;
use crate::multi_model_ans::Prelude;
use crate::traits::folding::FoldRead;
use crate::traits::quasi::{Decode, Quasi};
use crate::FASTER_RADIX;
use crate::{DecoderModelEntry, State};


/// A builder for [`ANSBVGraphReader`].
pub struct ANSBVGraphReaderBuilder<const FIDELITY: usize> {
    /// The vec of ANSCompressorPhase, one for each node of the graph.
    phases: Vec<ANSCompressorPhase>,

    /// The prelude resulting from the encoding process of the graph.
    prelude: Prelude<FASTER_RADIX, Vec<u8>>,
}

impl<const FIDELITY: usize> ANSBVGraphReaderBuilder<FIDELITY> {
    pub fn new(prelude: Prelude<FASTER_RADIX, Vec<u8>>, phases: Vec<ANSCompressorPhase>) -> Self {
        Self {
            prelude,
            phases
        }
    }
}

impl <const FIDELITY: usize> BVGraphCodesReaderBuilder for ANSBVGraphReaderBuilder<FIDELITY> {
    type Reader<'a> = ANSBVGraphReader<'a, FIDELITY> where Self: 'a;

    fn get_reader(&self, node: usize) -> Result<Self::Reader<'_>, Box<dyn Error>> {
        let mut code_reader = ANSBVGraphReader::<'_, FIDELITY>::new(&self.prelude);
        let phase = self.phases.get(node).unwrap();
        code_reader.decoder.set_compressor_at_phase(&phase);
        Ok(code_reader)
    }
}


/// An implementation of [`BVGraphCodesReader`] that reads from an ANS-encoded graph.
pub struct ANSBVGraphReader<
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
    pub decoder: ANSDecoder<'a, FIDELITY, RADIX, H, M, F>,
}

impl<'a, const FIDELITY: usize>
    ANSBVGraphReader<'a, FIDELITY, FASTER_RADIX, u64, VecFrame<FASTER_RADIX, u64>, Vec<u8>>
{
    pub fn new(prelude: &'a Prelude<FASTER_RADIX, Vec<u8>>) -> Self {
        Self {
            decoder: ANSDecoder::<FIDELITY>::new(prelude),
        }
    }
}

impl<'a, const FIDELITY: usize, const RADIX: usize, H, M, F> BVGraphCodesReader
    for ANSBVGraphReader<'a, FIDELITY, RADIX, H, M, F>
where
    H: Quasi<RADIX>,
    M: Decode + SymbolLookup<State, Output = DecoderModelEntry<RADIX, H>>,
    F: FoldRead<RADIX>,
{
    fn read_outdegree(&mut self) -> u64 {
        self.decoder.decode(Component::Outdegree as usize)
    }

    fn read_reference_offset(&mut self) -> u64 {
        self.decoder.decode(Component::ReferenceOffset as usize)
    }

    fn read_block_count(&mut self) -> u64 {
        self.decoder.decode(Component::BlockCount as usize)
    }

    fn read_blocks(&mut self) -> u64 {
        self.decoder.decode(Component::Blocks as usize)
    }

    fn read_interval_count(&mut self) -> u64 {
        self.decoder.decode(Component::IntervalCount as usize)
    }

    fn read_interval_start(&mut self) -> u64 {
        self.decoder.decode(Component::IntervalStart as usize)
    }

    fn read_interval_len(&mut self) -> u64 {
        self.decoder.decode(Component::IntervalLen as usize)
    }

    fn read_first_residual(&mut self) -> u64 {
        self.decoder.decode(Component::FirstResidual as usize)
    }

    fn read_residual(&mut self) -> u64 {
        self.decoder.decode(Component::Residual as usize)
    }
}
