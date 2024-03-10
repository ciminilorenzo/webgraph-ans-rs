use clap::Parser;
use folded_streaming_rans::bvgraph::random_access::ANSBVGraph;
use lender::Lender;
use webgraph::prelude::SequentialLabeling;

#[derive(Parser, Debug)]
#[command(about = "Tests the speed of an ANS graph", long_about = None)]
struct Args {
    /// The basename of the graph.
    basename: String,
}

pub fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let graph = ANSBVGraph::load(args.basename)?;

    for i in 0..10 {
        eprintln!("Iteration number {} out of 10", i);
        let mut c: u64 = 0;
        let start = std::time::Instant::now();
        let mut iter = graph.iter();
        while let Some((_, succ)) = iter.next() {
            c += succ.into_iter().count() as u64;
        }

        println!("{}", (start.elapsed().as_secs_f64() / c as f64) * 1e9);
    }
    Ok(())
}
