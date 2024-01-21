use std::error::Error;

use webgraph::prelude::{BVGraphCodesReaderBuilder};

use crate::multi_model_ans::decoder::ANSDecoder;
use crate::multi_model_ans::encoder::ANSCompressorPhase;
use crate::multi_model_ans::model4decoder::VecFrame;
use crate::multi_model_ans::Prelude;


pub struct ANSBVGraphReaderBuilder<const FIDELITY: usize, const RADIX: usize> {
    /// The vec of ANSCompressorPhase, one for each node of the graph.
    phases: Vec<ANSCompressorPhase>,

    /// The prelude resulting from the encoding process of the graph.
    prelude: Prelude<RADIX>,

    decoder_model: VecFrame<RADIX, u64>,
}

impl<const FIDELITY: usize, const RADIX: usize> ANSBVGraphReaderBuilder<FIDELITY, RADIX> {
    pub fn new(prelude: Prelude<RADIX>, phases: Vec<ANSCompressorPhase>) -> Self {
        let folding_offset = (1u64 << (FIDELITY - 1)) * ((1 << RADIX) - 1);
        let folding_threshold = 1u64 << (FIDELITY + RADIX - 1);

        let decoder_model = VecFrame::<RADIX, u64>::new(
            &prelude.tables,
            &prelude.frame_sizes,
            folding_offset,
            folding_threshold,
        );

        Self {
            prelude,
            phases,
            decoder_model,
        }
    }
}

impl<const FIDELITY: usize, const RADIX: usize> BVGraphCodesReaderBuilder for ANSBVGraphReaderBuilder<FIDELITY, RADIX> {
    type Reader<'a> = ANSDecoder<'a, FIDELITY, RADIX, u64, VecFrame<RADIX, u64>>
    where
        Self: 'a;

    fn get_reader(&self, node: usize) -> Result<Self::Reader<'_>, Box<dyn Error>> {
        let phase = self
            .phases
            .get(node)
            .expect("The node must have a phase associated to it.");

        Ok(ANSDecoder::<FIDELITY, RADIX, u64, VecFrame<RADIX, u64>>::from_raw_parts(
            &self.prelude,
            &self.decoder_model,
            *phase,
        ))
    }
}