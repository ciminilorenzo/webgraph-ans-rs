use std::error::Error;

use webgraph::prelude::{BVGraphCodesReaderBuilder};

use crate::multi_model_ans::decoder::ANSDecoder;
use crate::multi_model_ans::{ANSCompressorPhase, Prelude};
use crate::multi_model_ans::model4decoder::ANSModel4Decoder;


pub struct ANSBVGraphReaderBuilder<'a> {
    /// The vec of ANSCompressorPhase, one for each node of the graph.
    phases: Vec<ANSCompressorPhase>,

    /// The prelude resulting from the encoding process of the graph.
    prelude: &'a Prelude,

    model: ANSModel4Decoder,
}

impl <'a> ANSBVGraphReaderBuilder<'a> {
    pub fn new(prelude: &'a Prelude, phases: Vec<ANSCompressorPhase>) -> Self {
        Self {
            prelude,
            phases,
            model: ANSModel4Decoder::new(&prelude.tables),
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
            self.prelude,
            &self.model,
            *phase,
        ))
    }
}