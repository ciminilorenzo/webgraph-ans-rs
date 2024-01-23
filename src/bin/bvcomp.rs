/*
use std::path::PathBuf;
use anyhow::Result;
use clap::{Parser};
use epserde::prelude::Serialize;
use mem_dbg::{DbgFlags, MemDbg};
use webgraph::prelude::*;
use folded_streaming_rans::bvgraph::mock_writers::{ANSymbolTable, EntropyMockWriter, Log2MockWriter};
use folded_streaming_rans::bvgraph::writer::{BVGraphModelBuilder, BVGraphWriter};

const FIDELITY: usize = 2;
const RADIX: usize = 6;

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

    let costs_table = ANSymbolTable::<FIDELITY, RADIX>::initialize_with_binary_cost(9);
    let model_builder = BVGraphModelBuilder::<FIDELITY, RADIX, Log2MockWriter<FIDELITY, RADIX>>::new(costs_table);
    let mut bvcomp = BVComp::<BVGraphModelBuilder<
        FIDELITY,
        RADIX,
        Log2MockWriter<FIDELITY, RADIX>
    >>::new(model_builder, 16, 2, 2147483647, 0);

    bvcomp.extend(seq_graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();
    let symbol_freqs = model4encoder.get_symbol_freqs();
    let entropy_costs = ANSymbolTable::<FIDELITY, RADIX>::new(
        symbol_freqs,
        model4encoder.frame_sizes.clone()
    );

    let model_builder = BVGraphModelBuilder::<
        FIDELITY,
        RADIX,
        EntropyMockWriter<FIDELITY, RADIX>
    >::new(entropy_costs.clone());

    let mut bvcomp = BVComp::<BVGraphModelBuilder<
        FIDELITY,
        RADIX,
        EntropyMockWriter<FIDELITY, RADIX>
    >>::new(model_builder, 16, 2, 2147483647, 0);

    bvcomp.extend(seq_graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();

    let mut bvcomp = BVComp::<BVGraphWriter<FIDELITY, RADIX>>::new(
        BVGraphWriter::new(model4encoder, entropy_costs),
        16,
        2,
        2147483647,
        0
    );

    // third iteration: encode with the entropy mock
    bvcomp.extend(seq_graph.iter())?;

    // get phases and the encoder from the bvcomp
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
*/

fn main() {}