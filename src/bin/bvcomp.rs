use std::path::PathBuf;
use anyhow::Result;
use clap::{Parser};
use epserde::prelude::Serialize;
use mem_dbg::{DbgFlags, MemDbg};
use webgraph::prelude::*;
use folded_streaming_rans::bvgraph::mock_writers::{ANSymbolTable, EntropyMockWriter, Log2MockWriter, MockWriter};
use folded_streaming_rans::bvgraph::writer::{BVGraphModelBuilder, BVGraphWriter};

// for highly-compressed compressions are: [16, 2, 2147483647,0]
const BVGRAPH_COMPRESSION_PARAMS: [usize; 4] = [7, 2, 3, 0];

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
    let log2_mock = Log2MockWriter::build(ANSymbolTable::default());
    let model_builder = BVGraphModelBuilder::<Log2MockWriter>::new(log2_mock);
    let mut bvcomp = BVComp::<BVGraphModelBuilder<Log2MockWriter>>::new(
        model_builder,
        BVGRAPH_COMPRESSION_PARAMS[0],
        BVGRAPH_COMPRESSION_PARAMS[1],
        BVGRAPH_COMPRESSION_PARAMS[2],
        BVGRAPH_COMPRESSION_PARAMS[3]
    );

    bvcomp.extend(seq_graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();
    let folding_params = model4encoder.get_folding_params();
    let entropy_costs = ANSymbolTable::new(&model4encoder, folding_params);
    let entropy_mock = EntropyMockWriter::build(entropy_costs.clone());
    let model_builder = BVGraphModelBuilder::<EntropyMockWriter>::new(entropy_mock);
    let mut bvcomp = BVComp::<BVGraphModelBuilder<EntropyMockWriter>>::new(
        model_builder,
        BVGRAPH_COMPRESSION_PARAMS[0],
        BVGRAPH_COMPRESSION_PARAMS[1],
        BVGRAPH_COMPRESSION_PARAMS[2],
        BVGRAPH_COMPRESSION_PARAMS[3]
    );

    bvcomp.extend(seq_graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();

    let mut bvcomp = BVComp::<BVGraphWriter>::new(BVGraphWriter::new(model4encoder, entropy_costs),
        BVGRAPH_COMPRESSION_PARAMS[0],
        BVGRAPH_COMPRESSION_PARAMS[1],
        BVGRAPH_COMPRESSION_PARAMS[2],
        BVGRAPH_COMPRESSION_PARAMS[3]
    );

    // third iteration: encode with the entropy mock
    bvcomp.extend(seq_graph.iter())?;

    // get phases and the encoder from the bvcomp
    let (encoder, phases) = bvcomp.flush()?.into_inner();
    let prelude = encoder.into_prelude();

    prelude.mem_dbg(DbgFlags::default() | DbgFlags::PERCENTAGE)?;
    let mut buf = PathBuf::from(&args.new_basename);
    buf.set_extension("ans");
    prelude.store(buf.as_path())?;
    buf.set_extension("phases");
    phases.store(buf.as_path())?;

    Ok(())
}