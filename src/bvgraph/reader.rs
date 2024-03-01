use webgraph::graphs::{RandomAccessDecoderFactory, SequentialDecoderFactory};

use crate::ans::decoder::ANSDecoder;
use crate::ans::model4decoder::ANSModel4Decoder;
use crate::ans::{ANSCompressorPhase, Prelude};

pub struct ANSBVGraphDecoderFactory {
    /// The vec of ANSCompressorPhase, one for each node of the graph.
    phases: Vec<ANSCompressorPhase>,

    /// The prelude resulting from the encoding process of the graph.
    pub prelude: Prelude,

    model: ANSModel4Decoder,
}

impl ANSBVGraphDecoderFactory {
    pub fn new(prelude: Prelude, phases: Vec<ANSCompressorPhase>) -> Self {
        Self {
            phases,
            model: ANSModel4Decoder::new(&prelude.tables),
            prelude,
        }
    }
}

impl RandomAccessDecoderFactory for ANSBVGraphDecoderFactory {
    type Decoder<'b> = ANSDecoder<'b> where Self: 'b;

    fn new_decoder(&self, node: usize) -> anyhow::Result<Self::Decoder<'_>> {
        let phase = self
            .phases
            .get(node)
            .expect("The node must have a phase associated to it.");

        Ok(ANSDecoder::from_phase(
            &self.model,
            &self.prelude.stream,
            *phase,
        ))
    }
}

pub struct ANSBVGraphSeqDecoderFactory<'a> {
    /// The prelude resulting from the encoding process of the graph.
    prelude: &'a Prelude,

    model: ANSModel4Decoder,
}

impl<'a> ANSBVGraphSeqDecoderFactory<'a> {
    pub fn new(prelude: &'a Prelude) -> Self {
        Self {
            prelude,
            model: ANSModel4Decoder::new(&prelude.tables),
        }
    }
}

impl<'a> SequentialDecoderFactory for ANSBVGraphSeqDecoderFactory<'a> {
    type Decoder<'b> = ANSDecoder<'b> where Self: 'b;

    fn new_decoder(&self) -> anyhow::Result<Self::Decoder<'_>> {
        Ok(ANSDecoder::new(
            &self.model,
            &self.prelude.stream,
            self.prelude.state,
        ))
    }
}
