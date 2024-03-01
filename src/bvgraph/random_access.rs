use crate::ans::Prelude;
use crate::bvgraph::reader::ANSBVGraphDecoderFactory;
use anyhow::Result;
use epserde::prelude::*;
use std::path::PathBuf;
use webgraph::graphs::BVGraph;

/// An ANS-encoded BVGraph that can be accessed both randomly and sequentially.
pub struct ANSBVGraph(BVGraph<ANSBVGraphDecoderFactory>);

impl ANSBVGraph {
    /// Loads a previously ANS-encoded BVGraph from disk.
    pub fn load(
        basename: impl AsRef<std::path::Path> + AsRef<std::ffi::OsStr>,
    ) -> Result<BVGraph<ANSBVGraphDecoderFactory>> {
        let mut buf = PathBuf::from(&basename);

        // load prelude
        buf.set_extension("ans");
        let prelude = Prelude::load_full(buf.as_path())?;

        // load phases
        buf.set_extension("phases");
        let mut phases = Vec::<crate::ans::ANSCompressorPhase>::load_full(buf.as_path())?;
        // let's reverse the phases so that the first phase is associated to the last node encoded
        phases.reverse();

        let nun_nodes = prelude.number_of_nodes;
        let num_arcs = prelude.number_of_arcs;
        let compression_window = prelude.compression_window;
        let min_interval_length = prelude.min_interval_length;
        let factory = ANSBVGraphDecoderFactory::new(prelude, phases);

        Ok(BVGraph::<ANSBVGraphDecoderFactory>::new(
            factory,
            nun_nodes,
            num_arcs,
            compression_window,
            min_interval_length,
        ))
    }
}
