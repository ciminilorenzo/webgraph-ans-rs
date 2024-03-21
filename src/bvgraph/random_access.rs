use crate::ans::{ANSCompressorPhase, Prelude};
use crate::bvgraph::mock_writers::{EntropyEstimator, Log2Estimator};
use crate::bvgraph::reader::ANSBVGraphDecoderFactory;
use crate::bvgraph::writer::{ANSBVGraphMeasurableEncoder, BVGraphModelBuilder};
use crate::{State, EF};
use anyhow::{Context, Result};
use dsi_bitstream::prelude::BE;
use dsi_progress_logger::{ProgressLog, ProgressLogger};
use epserde::prelude::*;
use lender::for_;
use log::info;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use sux::dict::EliasFanoBuilder;
use sux::traits::ConvertTo;
use webgraph::graphs::{BVComp, BVGraph, BVGraphSeq};
use webgraph::prelude::SequentialLabeling;

/// An ANS-encoded BVGraph that can be accessed both randomly and sequentially.
pub struct ANSBVGraph();

impl ANSBVGraph {
    /// Loads a previously ANS-encoded BVGraph from disk.
    pub fn load(
        basename: impl AsRef<std::path::Path> + AsRef<std::ffi::OsStr>,
    ) -> Result<BVGraph<ANSBVGraphDecoderFactory>> {
        let mut buf = PathBuf::from(&basename);

        // load prelude
        buf.set_extension("ans");
        let prelude = Prelude::load_full(buf.as_path())?;

        // load pointers
        buf.set_extension("pointers");
        let phases = EF::load_full(buf.as_path())?;

        // load states
        buf.set_extension("states");
        let states = Box::<[State]>::load_full(buf.as_path())?;

        let nun_nodes = prelude.number_of_nodes;
        let num_arcs = prelude.number_of_arcs;
        let compression_window = prelude.compression_window;
        let min_interval_length = prelude.min_interval_length;
        let factory = ANSBVGraphDecoderFactory::new(prelude, phases, states);

        Ok(BVGraph::<ANSBVGraphDecoderFactory>::new(
            factory,
            nun_nodes,
            num_arcs,
            compression_window,
            min_interval_length,
        ))
    }

    /// Recompresses a BVGraph stored in `basename` and stores the result in `new_basename`.
    /// The function stores two files with the following structure:
    /// - `basename.ans`: contains the prelude of the ANS encoding.
    /// - `basename.phases`: contains the phases of the ANS encoding.
    pub fn store(
        basename: impl AsRef<std::path::Path> + AsRef<std::ffi::OsStr>,
        new_basename: impl AsRef<std::path::Path> + AsRef<std::ffi::OsStr>,
        compression_window: usize,
        max_ref_count: usize,
        min_interval_length: usize,
    ) -> Result<()> {
        let mut pl = ProgressLogger::default();

        info!("Loading BVGraph...");
        let seq_graph = BVGraphSeq::with_basename(&basename)
            .endianness::<BE>()
            .load()?;

        // (1) setup for the first iteration with Log2Estimator
        let log2_mock = Log2Estimator::default();

        let mut model_builder = BVGraphModelBuilder::<Log2Estimator>::new(log2_mock);
        let mut bvcomp = BVComp::new(
            &mut model_builder,
            compression_window,
            max_ref_count,
            min_interval_length,
            0,
        );

        pl.item_name("node")
            .expected_updates(Some(seq_graph.num_nodes()));
        pl.start("Pushing symbols into model builder with Log2Estimator...");

        // first iteration: build a model with Log2Estimator
        for_![ (_, succ) in seq_graph {
            bvcomp.push(succ)?;
            pl.update();
        }];
        pl.done();

        pl.start("Building the model with Log2Estimator...");
        // get the ANSModel4Encoder obtained from the first iteration
        bvcomp.flush()?;
        let model4encoder = model_builder.build();
        pl.done();

        // (2) setup for the second iteration with EntropyEstimator
        // get the folding parameters from the model
        let folding_params = model4encoder.get_folding_params();
        // create a new table of costs based on params obtained from the previous step
        let entropy_estimator = EntropyEstimator::new(&model4encoder, folding_params);
        let mut model_builder =
            BVGraphModelBuilder::<EntropyEstimator>::new(entropy_estimator.clone());
        let mut bvcomp = BVComp::new(
            &mut model_builder,
            compression_window,
            max_ref_count,
            min_interval_length,
            0,
        );

        pl.item_name("node")
            .expected_updates(Some(seq_graph.num_nodes()));
        pl.start("Pushing symbols into model builder with EntropyEstimator...");

        // second iteration: build a model with the entropy mock writer
        for_![ (_, succ) in seq_graph {
        bvcomp.push(succ)?;
        pl.update();
        }];
        pl.done();

        pl.start("Building the model with EntropyEstimator...");
        // get the final ANSModel4Encoder from the second iteration
        bvcomp.flush()?;
        let model4encoder = model_builder.build();
        pl.done();

        let mut enc = ANSBVGraphMeasurableEncoder::new(
            model4encoder,
            entropy_estimator,
            seq_graph.num_nodes(),
            seq_graph.num_arcs_hint().unwrap(),
            compression_window,
            min_interval_length,
        );
        // (3) setup for the compression of the BVGraph
        let mut bvcomp = BVComp::new(
            &mut enc,
            compression_window,
            max_ref_count,
            min_interval_length,
            0,
        );

        pl.item_name("node")
            .expected_updates(Some(seq_graph.num_nodes()));
        pl.start("Compressing graph...");

        // third iteration: encode with the encoder that uses the ANSModel4Encoder we just got
        for_![ (_, succ) in seq_graph {
        bvcomp.push(succ)?;
        pl.update();
        }];
        pl.done();

        // get phases and the prelude after the graph's compression
        bvcomp.flush()?;
        let (prelude, phases) = enc.into_prelude_phases();

        // (5) serialize
        let mut buf = PathBuf::from(&new_basename);

        // serialize states
        let states: Box<[State]> = phases.iter().map(|phase| phase.state).collect();
        buf.set_extension("states");
        states.store(buf.as_path())?;

        // serialize ef containing stream pointers
        buf.set_extension("pointers");
        let ef = Self::build_ef(phases, prelude.number_of_nodes)?;
        let mut ef_file = BufWriter::new(
            File::create(&buf)
                .with_context(|| format!("Could not create {}", buf.to_str().unwrap()))?,
        );
        ef.serialize(&mut ef_file).with_context(|| {
            format!("Could not serialize EliasFano to {}", buf.to_str().unwrap())
        })?;

        // serialize prelude
        let mut buf = PathBuf::from(&new_basename);
        buf.set_extension("ans");
        prelude.store(buf.as_path())?;
        Ok(())
    }

    /// Builds an EliasFano with the whole set of stream pointers contained in the phases.
    pub fn build_ef(phases: Vec<ANSCompressorPhase>, num_nodes: usize) -> Result<EF> {
        let upper_bound = phases.last().unwrap().stream_pointer;
        let mut efb = EliasFanoBuilder::new(num_nodes, upper_bound + 1);

        for phase in phases.iter() {
            efb.push(phase.stream_pointer)?;
        }
        efb.build().convert_to()
    }
}
