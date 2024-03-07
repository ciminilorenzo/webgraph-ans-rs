use crate::ans::decoder::ANSDecoder;
use crate::ans::model4decoder::ANSModel4Decoder;
use crate::ans::Prelude;
use crate::{State, EF};
use anyhow::Result;
use sux::traits::indexed_dict::IndexedDict;
use webgraph::graphs::{RandomAccessDecoderFactory, SequentialDecoderFactory};

pub struct ANSBVGraphDecoderFactory {
    /// The EliasFano containing the phases of the ANS encoding, that is a stream pointer
    /// and state for each node. This data is merged into a single u64, one for each node.
    phases: EF,

    /// The prelude resulting from the encoding process of the graph.
    pub prelude: Prelude,

    /// The ANSModel4Decoder used by the decoder to decode the graph.
    model: ANSModel4Decoder,

    /// The number of nodes in the graph.
    num_nodes: usize,
}

impl ANSBVGraphDecoderFactory {
    pub fn new(prelude: Prelude, phases: EF) -> Self {
        Self {
            phases,
            model: ANSModel4Decoder::new(&prelude.tables),
            num_nodes: prelude.number_of_nodes,
            prelude,
        }
    }
}

impl RandomAccessDecoderFactory for ANSBVGraphDecoderFactory {
    type Decoder<'b> = ANSDecoder<'b> where Self: 'b;

    fn new_decoder(&self, node: usize) -> Result<Self::Decoder<'_>> {
        // nodes' phases are stored in reversed order. Thus, for example, let's
        // take the last phase if we want the phase of the first node.
        let ef_entry = self.phases.get(self.num_nodes - node - 1);

        Ok(ANSDecoder::from_raw_parts(
            &self.model,
            &self.prelude.stream,
            ef_entry >> 32,
            (ef_entry & u32::MAX as usize) as State,
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
