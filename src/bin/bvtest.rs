use std::iter::Iterator;
use std::{hint::black_box, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use dsi_progress_logger::*;
use epserde::prelude::*;
use folded_streaming_rans::ans::{ANSCompressorPhase, Prelude};
use folded_streaming_rans::bvgraph::reader::ANSBVGraphDecoderFactory;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use webgraph::prelude::*;

#[derive(Parser, Debug)]
#[command(about = "Tests the speed of an ANS graph", long_about = None)]
struct Args {
    /// The basename of the graph.
    basename: String,
    /// The number of nodes to test.
    #[clap(short, long, default_value_t = 1000)]
    n: usize,
}

pub fn main() -> Result<()> {
    let args = Args::parse();

    stderrlog::new()
        .verbosity(2)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    let seq_graph = BVGraph::with_basename(&args.basename).load()?;
    let mut buf = PathBuf::from(&args.basename);
    buf.set_extension("ans");
    let prelude = Prelude::load_full(buf.as_path())?;
    buf.set_extension("phases");
    let phases = Vec::<ANSCompressorPhase>::load_full(buf.as_path())?;
    let code_reader_builder = ANSBVGraphDecoderFactory::new(&prelude, phases);

    let graph = BVGraph::<ANSBVGraphDecoderFactory>::new(
        code_reader_builder,
        2,
        7,
        seq_graph.num_nodes(),
        seq_graph.num_arcs_hint().unwrap(),
    );

    let mut pl = ProgressLogger::default();
    let mut rng = SmallRng::seed_from_u64(0);

    pl.start("Testing successors...");
    for _ in 0..args.n {
        let mut d = 0;
        graph
            .successors(rng.gen_range(0..graph.num_nodes()))
            .for_each(|x| {
                d += 1;
                black_box(x);
            });
        pl.update_with_count(d);
    }
    pl.done();

    Ok(())
}
