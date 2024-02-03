/*
   This bench prints the medium time (in terms of nanoseconds) needed to decode each arc of the cnr-2000 graph.
*/

use anyhow::Result;
use epserde::prelude::{Deserialize, Serialize};
use webgraph::prelude::{BVComp, BVGraph, EmptyDict, RandomAccessLabelling, SequentialLabelling};

use folded_streaming_rans::bvgraph::mock_writers::{
    ANSymbolTable, EntropyMockWriter, Log2MockWriter, MockWriter,
};
use folded_streaming_rans::bvgraph::reader::ANSBVGraphReaderBuilder;
use folded_streaming_rans::bvgraph::writer::{BVGraphModelBuilder, BVGraphWriter};
use folded_streaming_rans::multi_model_ans::{ANSCompressorPhase, Prelude};

const NODES: usize = 325557;
const ARCS: usize = 3216152;

fn main() -> Result<()> {
    let graph = webgraph::graph::bvgraph::load("tests/data/cnr-2000/cnr-2000")?;

    let log2_mock = Log2MockWriter::build(ANSymbolTable::default());
    let model_builder = BVGraphModelBuilder::<Log2MockWriter>::new(log2_mock);
    let mut bvcomp = BVComp::<BVGraphModelBuilder<Log2MockWriter>>::new(model_builder, 7, 2, 3, 0);

    bvcomp.extend(graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();
    let folding_params = model4encoder.get_folding_params();
    let entropic_costs_table = ANSymbolTable::new(&model4encoder, folding_params);
    let entropic_mock = EntropyMockWriter::build(entropic_costs_table.clone());
    let model_builder = BVGraphModelBuilder::<EntropyMockWriter>::new(entropic_mock);
    let mut bvcomp =
        BVComp::<BVGraphModelBuilder<EntropyMockWriter>>::new(model_builder, 7, 2, 3, 0);

    bvcomp.extend(graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();

    let mut bvcomp = BVComp::<BVGraphWriter>::new(
        BVGraphWriter::new(model4encoder, entropic_costs_table),
        7,
        2,
        3,
        0,
    );

    // third iteration: encode with the entropy mock
    bvcomp.extend(graph.iter())?;

    // get phases and the encoder from the bvcomp
    let (encoder, phases) = bvcomp.flush()?.into_inner();
    let prelude = encoder.into_prelude();

    phases.store("cnr-2000-phases")?;
    prelude.store("cnr-2000-prelude")?;

    let prelude = Prelude::load_full("cnr-2000-prelude")?;
    let phases = Vec::<ANSCompressorPhase>::load_full("cnr-2000-phases")?;
    let code_reader_builder = ANSBVGraphReaderBuilder::new(&prelude, phases);
    let decoded_graph = BVGraph::<ANSBVGraphReaderBuilder, EmptyDict<usize, usize>>::new(
        code_reader_builder,
        2,
        7,
        NODES,
        ARCS,
    );

    let now = std::time::Instant::now();
    let mut arcs = 0;
    for node_index in 0..NODES {
        let decoded_successors = decoded_graph.successors(node_index).collect::<Vec<_>>();
        arcs += decoded_successors.len();
    }

    dbg!(now.elapsed());
    dbg!(now.elapsed().as_nanos() / arcs as u128);
    Ok(())
}
