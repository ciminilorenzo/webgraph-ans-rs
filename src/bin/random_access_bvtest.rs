use std::hint::black_box;
use std::iter::Iterator;

use anyhow::Result;
use clap::Parser;
use dsi_progress_logger::*;
use folded_streaming_rans::bvgraph::random_access::ANSBVGraph;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use webgraph::prelude::*;

const RANDOM_TEST_SAMPLES: u64 = 10_000_000;

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

    let mut pl = ProgressLogger::default();
    let graph = ANSBVGraph::load(args.basename)?;

    pl.item_name("node")
        .expected_updates(Some(RANDOM_TEST_SAMPLES as usize));
    pl.start("Starting random-access speed test...");

    // Random-access speed test
    let mut rng = SmallRng::seed_from_u64(0);
    let mut c: u64 = 0;
    let num_nodes = graph.num_nodes();
    let random_nodes = (0..RANDOM_TEST_SAMPLES).map(|_| rng.gen_range(0..num_nodes));
    let start = std::time::Instant::now();
    for node in random_nodes {
        c += black_box(graph.successors(node).count() as u64);
        pl.update();
    }
    pl.done_with_count(c as usize);

    println!("{:.2} ns/arc", start.elapsed().as_nanos() / c as u128);
    Ok(())
}