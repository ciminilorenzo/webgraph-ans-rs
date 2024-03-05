use std::hint::black_box;
use std::iter::Iterator;

use anyhow::Result;
use clap::Parser;
use dsi_progress_logger::*;
use folded_streaming_rans::bvgraph::random_access::ANSBVGraph;
use lender::for_;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use webgraph::prelude::*;

#[derive(Parser, Debug)]
#[command(about = "Tests the speed of an ANS graph", long_about = None)]
struct Args {
    /// The basename of the graph.
    basename: String,
}

pub fn main() -> Result<()> {
    let args = Args::parse();

    stderrlog::new()
        .verbosity(2)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    let graph = ANSBVGraph::load(&args.basename)?;
    let mut pl = ProgressLogger::default();
    let mut rng = SmallRng::seed_from_u64(0);

    let random_nodes = (0..graph.num_nodes())
        .into_iter()
        .map(|_| rng.gen_range(0..graph.num_nodes()))
        .collect::<Vec<_>>();

    pl.start("Testing random node's successors...");
    for random_mode in random_nodes.iter() {
        let mut d = 0;
        graph.successors(*random_mode).for_each(|x| {
            d += 1;
            black_box(x);
        });
        pl.update_with_count(d);
    }
    pl.done();

    pl.start("Testing successor of sequential nodes");
    for_![ (_, s) in graph.iter() {
        let mut d = 0;
        s.for_each(|x| {
            d += 1;
            black_box(x);
        });
        pl.update_with_count(d);
    }];
    pl.done();
    Ok(())
}
