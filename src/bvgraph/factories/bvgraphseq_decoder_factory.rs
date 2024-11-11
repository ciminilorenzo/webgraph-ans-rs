use webgraph::prelude::SequentialDecoderFactory;
use crate::ans::decoder::ANSDecoder;
use crate::ans::model4decoder::ANSModel4Decoder;
use crate::ans::Prelude;

pub struct ANSBVGraphSeqDecoderFactory {
    /// The prelude resulting from the encoding process of the graph.
    prelude: Prelude,

    /// The ANSModel4Decoder used by the decoder to decode the graph.
    model: ANSModel4Decoder,
}

impl ANSBVGraphSeqDecoderFactory {
    pub fn new(prelude: Prelude) -> Self {
        Self {
            model: ANSModel4Decoder::new(&prelude.tables),
            prelude,
        }
    }
}

impl SequentialDecoderFactory for ANSBVGraphSeqDecoderFactory {
    type Decoder<'b> = ANSDecoder<'b> where Self: 'b;

    fn new_decoder(&self) -> anyhow::Result<Self::Decoder<'_>> {
        Ok(ANSDecoder::new(
            &self.model,
            &self.prelude.stream,
            self.prelude.state,
        ))
    }
}