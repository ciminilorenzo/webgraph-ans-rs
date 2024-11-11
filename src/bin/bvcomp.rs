use anyhow::Result;
use clap::Parser;
use folded_streaming_rans::bvgraph::random_access::ANSBVGraph;
use webgraph::cli::CompressArgs;

#[derive(Parser, Debug)]
#[command(about = "Recompress a BVGraph", long_about = None)]
struct Args {
    /// The basename of the graph.
    basename: String,

    /// The basename for the newly compressed graph.
    new_basename: String,

    /// Args for compressing the graph.
    #[clap(flatten)]
    compressions_args: CompressArgs,
}

pub fn main() -> Result<()> {
    stderrlog::new()
        .verbosity(2)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    let args = Args::parse();

    ANSBVGraph::store(
        &args.basename,
        &args.new_basename,
        args.compressions_args.compression_window,
        args.compressions_args.max_ref_count as usize,
        args.compressions_args.min_interval_length,
    )?;

    Ok(())
}
