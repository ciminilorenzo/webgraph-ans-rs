use crate::ans::Prelude;
use crate::bvgraph::factories::bvgraphseq_decoder_factory::ANSBVGraphSeqDecoderFactory;

use epserde::prelude::*;

use std::path::PathBuf;

use webgraph::prelude::BvGraphSeq;

/// An ANS-encoded BVSeqGraph that can only be accessed sequentially.
pub struct ANSBvGraphSeq();

impl ANSBvGraphSeq {
    /// Loads a previously ANS-encoded BVSeqGraph from disk.
    ///
    /// To correctly reconstruct a previously encoded graph, the path specified in `basename`
    /// must lead to a directory containing the file `basename.ans`.
    pub fn load(
        basename: impl AsRef<std::path::Path> + AsRef<std::ffi::OsStr>,
    ) -> anyhow::Result<BvGraphSeq<ANSBVGraphSeqDecoderFactory>> {
        let mut buf = PathBuf::from(&basename);

        // load prelude
        buf.set_extension("ans");
        let prelude = Prelude::load_full(buf.as_path())?;

        let nun_nodes = prelude.number_of_nodes;
        let num_arcs = prelude.number_of_arcs;
        let compression_window = prelude.compression_window;
        let min_interval_length = prelude.min_interval_length;
        let factory = ANSBVGraphSeqDecoderFactory::new(prelude);

        Ok(BvGraphSeq::<ANSBVGraphSeqDecoderFactory>::new(
            factory,
            nun_nodes,
            Some(num_arcs),
            compression_window,
            min_interval_length,
        ))
    }
}
