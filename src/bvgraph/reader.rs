use crate::ans::decoder::ANSDecoder;
use crate::ans::model4decoder::ANSModel4Decoder;
use crate::ans::Prelude;
use crate::EF;
use anyhow::Result;
use webgraph::prelude::{RandomAccessDecoderFactory, SequentialDecoderFactory};

pub struct ANSBVGraphDecoderFactory {
    /// The EliasFano containing the stream pointers for each of the nodes.
    phases: EF,

    /// A list of states, one for each of the nodes.
    states: Box<[u32]>,

    /// The prelude resulting from the encoding process of the graph.
    pub prelude: Prelude,

    /// The ANSModel4Decoder used by the decoder to decode the graph.
    model: ANSModel4Decoder,

    /// The number of nodes in the graph.
    num_nodes: usize,
}

impl ANSBVGraphDecoderFactory {
    pub fn new(prelude: Prelude, phases: EF, states: Box<[u32]>) -> Self {
        Self {
            phases,
            model: ANSModel4Decoder::new(&prelude.tables),
            num_nodes: prelude.number_of_nodes,
            prelude,
            states,
        }
    }
}

impl RandomAccessDecoderFactory for ANSBVGraphDecoderFactory {
    type Decoder<'b> = ANSDecoder<'b> where Self: 'b;

    fn new_decoder(&self, node: usize) -> Result<Self::Decoder<'_>> {
        // nodes' phases are stored in reversed order. Thus, for example, let's
        // take the last phase if we want the phase of the first node.
        let pointer = self.phases.get(self.num_nodes - node - 1);
        let state = self.states[self.num_nodes - node - 1];

        Ok(ANSDecoder::from_raw_parts(
            &self.model,
            &self.prelude.stream,
            pointer,
            state,
        ))
    }
}

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

    fn new_decoder(&self) -> Result<Self::Decoder<'_>> {
        Ok(ANSDecoder::new(
            &self.model,
            &self.prelude.stream,
            self.prelude.state,
        ))
    }
}
