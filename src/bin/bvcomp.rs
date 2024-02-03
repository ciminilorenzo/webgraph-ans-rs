use anyhow::Result;
use clap::Parser;
use dsi_progress_logger::*;
use epserde::prelude::Serialize;
use folded_streaming_rans::bvgraph::mock_writers::{
    ANSymbolTable, EntropyMockWriter, Log2MockWriter, MockWriter,
};
use folded_streaming_rans::bvgraph::writer::{BVGraphModelBuilder, BVGraphWriter};
use lender::*;
use log::info;
use mem_dbg::{DbgFlags, MemDbg};
use std::path::PathBuf;
use webgraph::prelude::*;

#[derive(Parser, Debug)]
#[command(about = "Recompress a BVGraph", long_about = None)]
struct Args {
    /// The basename of the graph.
    basename: String,

    /// The basename for the newly compressed graph.
    new_basename: String,

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

    // crete a log2 mock writer, where the cost of each symbol is the amount of bits needed to represent it
    let log2_mock = Log2MockWriter::build(ANSymbolTable::default());
    // create a builder that uses the log2 mock writer
    let model_builder = BVGraphModelBuilder::<Log2MockWriter>::new(log2_mock);
    let mut bvcomp = BVComp::<BVGraphModelBuilder<Log2MockWriter>>::new(
        model_builder,
        args.ca.compression_window,
        args.ca.min_interval_length,
        args.ca.max_ref_count,
        0,
    );

    let mut pl = ProgressLogger::default();
    pl.item_name("node")
        .expected_updates(Some(seq_graph.num_nodes()));
    pl.start("Computing data distribution...");

    // first iteration: build a model with the log2 mock writer
    for_![ (_, succ) in seq_graph {
        bvcomp.push(succ)?;
        pl.update();
    }];
    pl.done();

    info!("Building encoder...");
    // get the ANSModel4Encoder obtained from the first iteration
    let model4encoder = bvcomp.flush()?.build();
    // get the folding parameters from the model, that is the best combination of radix and fidelity
    let folding_params = model4encoder.get_folding_params();
    // create a new table of costs based on params obtained from the previous step
    let entropy_costs = ANSymbolTable::new(&model4encoder, folding_params);
    info!("Building entropy writer...");
    // create an entropy mock that uses the entropic costs table
    let entropy_mock = EntropyMockWriter::build(entropy_costs.clone());
    let model_builder = BVGraphModelBuilder::<EntropyMockWriter>::new(entropy_mock);
    let mut bvcomp = BVComp::<BVGraphModelBuilder<EntropyMockWriter>>::new(
        model_builder,
        args.ca.compression_window,
        args.ca.min_interval_length,
        args.ca.max_ref_count,
        0,
    );

    pl.item_name("node")
        .expected_updates(Some(seq_graph.num_nodes()));
    pl.start("Computing models...");

    // second iteration: build a model with the entropy mock writer
    for_![ (_, succ) in seq_graph {
        bvcomp.push(succ)?;
        pl.update();
    }];
    pl.done();

    // get the final ANSModel4Encoder from the second iteration
    let model4encoder = bvcomp.flush()?.build();
    let mut bvcomp = BVComp::<BVGraphWriter>::new(
        BVGraphWriter::new(model4encoder, entropy_costs),
        args.ca.compression_window,
        args.ca.min_interval_length,
        args.ca.max_ref_count,
        0,
    );

    pl.item_name("node")
        .expected_updates(Some(seq_graph.num_nodes()));
    pl.start("Compressing graph...");

    // third iteration: encode with the encoder that uses the ANSModel4Encoder we just got
    for_![ (_, succ) in seq_graph {
        bvcomp.push(succ)?;
        pl.update();
    }];
    pl.done();

    // get phases and the encoder from the bvcomp
    let (encoder, phases) = bvcomp.flush()?.into_inner();
    // get the prelude from the encoder
    let prelude = encoder.into_prelude();

    // let's store what we got
    prelude.mem_dbg(DbgFlags::default() | DbgFlags::PERCENTAGE)?;
    let mut buf = PathBuf::from(&args.new_basename);
    buf.set_extension("ans");
    prelude.store(buf.as_path())?;
    buf.set_extension("phases");
    phases.store(buf.as_path())?;

    Ok(())
}
