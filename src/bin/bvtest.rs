use std::{
    hint::black_box,
    path::{PathBuf},
};

use anyhow::Result;
use clap::Parser;
use dsi_progress_logger::*;
use epserde::prelude::*;
use folded_streaming_rans::{
    bvgraph::{
        reader::ANSBVGraphReaderBuilder,
    },
    multi_model_ans::{encoder::ANSCompressorPhase, Prelude},
};
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use webgraph::prelude::*;

const FIDELITY: usize = 2;
const RADIX: usize = 4;

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

    let seq_graph = load_seq(&args.basename)?;
    let mut buf = PathBuf::from(&args.basename);
    buf.set_extension("ans");
    let prelude = Prelude::<RADIX>::load_full(buf.as_path())?;
    buf.set_extension("phases");
    let phases = Vec::<ANSCompressorPhase>::load_full(buf.as_path())?;
    let code_reader_builder = ANSBVGraphReaderBuilder::<FIDELITY, RADIX>::new(prelude, phases);

    let graph = BVGraph::<ANSBVGraphReaderBuilder<FIDELITY, RADIX>, EmptyDict<usize, usize>>::new(
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
