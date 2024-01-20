/*
use std::path::PathBuf;
use anyhow::Result;
use clap::Parser;
use epserde::prelude::Serialize;
use mem_dbg::{DbgFlags, MemDbg};
use webgraph::prelude::*;
use folded_streaming_rans::bvgraph::mock_writers::{EntropyMockWriter, Log2MockWriter};
use folded_streaming_rans::bvgraph::writer::{BVGraphModelBuilder, BVGraphWriter};
use folded_streaming_rans::utils::ans_utilities::get_symbol_costs_table;

#[derive(Parser, Debug)]
#[command(about = "Recompress a BVGraph", long_about = None)]
struct Args {
    /// The basename of the graph.
    basename: String,

    /// The basename for the newly compressed graph.
    new_basename: String,

    radix: usize,

    fidelity: usize,

    #[clap(flatten)]
    num_cpus: NumCpusArg,

    #[clap(flatten)]
    pa: PermutationArgs,

    #[clap(flatten)]
    ca: CompressArgs,
}

pub fn main() -> Result<()> {
    let args = Args::parse();
    let radix = args.radix;
    let fidelity = args.fidelity;

    stderrlog::new()
        .verbosity(2)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    let seq_graph = load_seq(&args.basename)?;
    let model_builder = BVGraphModelBuilder::<{ radix }, { fidelity }, Log2MockWriter>::new(Vec::new());
    let mut bvcomp = BVComp::<BVGraphModelBuilder<{radix}, {fidelity}, Log2MockWriter>>::new(model_builder, 7, 2, 3, 0);

    bvcomp.extend(seq_graph.iter())?;

    let encoder = bvcomp.flush()?.build();
    let symbol_costs_table = get_symbol_costs_table(&encoder.tables, &encoder.frame_sizes, fidelity, radix);
    let model_builder = BVGraphModelBuilder::<{ radix }, { fidelity }, EntropyMockWriter>::new(symbol_costs_table);
    let mut bvcomp = BVComp::<BVGraphModelBuilder<{radix}, {fidelity}, EntropyMockWriter>>::new(model_builder, 7, 2, 3, 0);

    bvcomp.extend(seq_graph.iter())?;
    let encoder = bvcomp.flush()?.build();

    let mut bvcomp = BVComp::<BVGraphWriter<{fidelity}, {radix}>>::new(
        BVGraphWriter::new(encoder),
        7,
        2,
        3,
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

fn main() {

}