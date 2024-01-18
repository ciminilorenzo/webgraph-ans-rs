use std::path::{PathBuf};

use anyhow::Result;
use clap::Parser;
use epserde::prelude::*;
use folded_streaming_rans::bvgraph::writer::{BVGraphModelBuilder, BVGraphWriter};
use mem_dbg::{DbgFlags, MemDbg};
use webgraph::prelude::*;
use folded_streaming_rans::utils::ans_utilities::get_mock_writer;

#[derive(Parser, Debug)]
#[command(about = "Recompress a BVGraph", long_about = None)]
struct Args {
    /// The basename of the graph.
    basename: String,
    /// The basename for the newly compressed graph.
    new_basename: String,

    #[clap(flatten)]
    num_cpus: NumCpusArg,

    #[clap(flatten)]
    pa: PermutationArgs,

    #[clap(flatten)]
    ca: CompressArgs,
}

pub fn main() -> Result<()> {
    let args = Args::parse();

    stderrlog::new()
        .verbosity(2)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    let seq_graph = load_seq(&args.basename)?;

    let model_builder = BVGraphModelBuilder::<2, 8>::new();
    let mut bvcomp = BVComp::<BVGraphModelBuilder<2, 8>>::new(model_builder, 7, 2, 3, 0);

    bvcomp.extend(seq_graph.iter())?;
    let encoder = bvcomp.flush()?.build();
    let mock_writer = get_mock_writer(&encoder.tables, &encoder.frame_sizes);

    let mut bvcomp =
        BVComp::<BVGraphWriter<2, 8, Vec<u8>>>::new(BVGraphWriter::new(encoder, mock_writer), 7, 2, 3, 0);

    bvcomp.extend(seq_graph.iter())?;

    let (mut encoder, phases) = bvcomp.flush()?.into_inner();
    let prelude = encoder.serialize();

    prelude.mem_dbg(DbgFlags::default() | DbgFlags::PERCENTAGE)?;
    let mut buf = PathBuf::from(&args.new_basename);
    buf.set_extension("ans");
    prelude.store(buf.as_path())?;
    buf.set_extension("phases");
    phases.store(buf.as_path())?;
    Ok(())
}
