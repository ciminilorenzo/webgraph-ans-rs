use std::error::Error;

use webgraph::prelude::{BVGraphCodesReaderBuilder};

use crate::multi_model_ans::decoder::ANSDecoder;
use crate::multi_model_ans::model4decoder::VecFrame;
use crate::multi_model_ans::{ANSCompressorPhase, Prelude};


pub struct ANSBVGraphReaderBuilder<'a> {
    /// The vec of ANSCompressorPhase, one for each node of the graph.
    phases: Vec<ANSCompressorPhase>,

    /// The prelude resulting from the encoding process of the graph.
    prelude: &'a Prelude,

    decoder_model: VecFrame,

    fidelity: usize,

    radix: usize,
}

impl <'a> ANSBVGraphReaderBuilder<'a> {
    pub fn new(prelude: &'a Prelude, phases: Vec<ANSCompressorPhase>, fidelity: usize, radix: usize) -> Self {
        Self {
            prelude,
            phases,
            decoder_model: VecFrame::new(
                &prelude.tables,
                &prelude.frame_sizes,
                fidelity,
                radix,
            ),
            fidelity,
            radix,
        }
    }
}

impl <'a> BVGraphCodesReaderBuilder for ANSBVGraphReaderBuilder<'a> {
    type Reader<'b> = ANSDecoder<'b> where Self: 'b;

    fn get_reader(&self, node: usize) -> Result<Self::Reader<'_>, Box<dyn Error>> {
        let phase = self
            .phases
            .get(node)
            .expect("The node must have a phase associated to it.");

        Ok(ANSDecoder::from_raw_parts(
            &self.prelude,
            &self.decoder_model,
            *phase,
            self.fidelity,
            self.radix,
        ))
    }
}