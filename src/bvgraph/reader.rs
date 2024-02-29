use webgraph::graphs::{RandomAccessDecoderFactory, SequentialDecoderFactory};

use crate::ans::decoder::ANSDecoder;
use crate::ans::model4decoder::ANSModel4Decoder;
use crate::ans::{ANSCompressorPhase, Prelude};

pub struct ANSBVGraphDecoderFactory<'a> {
    /// The vec of ANSCompressorPhase, one for each node of the graph.
    phases: Vec<ANSCompressorPhase>,

    /// The prelude resulting from the encoding process of the graph.
    prelude: &'a Prelude,

    model: ANSModel4Decoder,
}

impl<'a> ANSBVGraphDecoderFactory<'a> {
    pub fn new(prelude: &'a Prelude, phases: Vec<ANSCompressorPhase>) -> Self {
        Self {
            prelude,
            phases,
            model: ANSModel4Decoder::new(&prelude.tables),
        }
    }
}

impl<'a> RandomAccessDecoderFactory for ANSBVGraphDecoderFactory<'a> {
    type Decoder<'b> = ANSDecoder<'b> where Self: 'b;

    fn new_decoder(&self, node: usize) -> anyhow::Result<Self::Decoder<'_>> {
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

impl<'a> SequentialDecoderFactory for ANSBVGraphDecoderFactory<'a> {
    type Decoder<'b> = ANSDecoder<'b> where Self: 'b;

    fn new_decoder(&self) -> anyhow::Result<Self::Decoder<'_>> {
        Ok(ANSDecoder::new(self.prelude, &self.model))
    }
}

/*

pub struct ANSSeqBVGraphDecoderFactory<'a> {
    /// The vec of ANSCompressorPhase, one for each node of the graph.
    phases: Vec<ANSCompressorPhase>,

    /// The prelude resulting from the encoding process of the graph.
    prelude: &'a Prelude,

    model: ANSModel4Decoder,
}

impl<'a> ANSSeqBVGraphDecoderFactory<'a> {
    pub fn new(prelude: &'a Prelude, phases: Vec<ANSCompressorPhase>) -> Self {
        Self {
            prelude,
            phases,
            model: ANSModel4Decoder::new(&prelude.tables),
        }
    }
}

impl<'a> SequentialDecoderFactory for ANSBVGraphDecoderFactory<'a> {
    type Decoder<'b> = ANSDecoder<'b> where Self: 'b;

    fn new_decoder(&self) -> anyhow::Result<Self::Decoder<'_>> {
        Ok(ANSDecoder::new(self.prelude, &self.model))
    }
}

*/
