/*
    This bench prints the medium time (in terms of nanoseconds) need to decode each arc of the cnr-2000 graph.
 */

use epserde::prelude::{Deserialize, Serialize};
use anyhow::Result;
use webgraph::prelude::{BVComp, BVGraph, EmptyDict, RandomAccessLabelling, SequentialLabelling};

use folded_streaming_rans::bvgraph::reader::ANSBVGraphReaderBuilder;
use folded_streaming_rans::bvgraph::writer::{BVGraphModelBuilder, BVGraphWriter};
use folded_streaming_rans::multi_model_ans::{ANSCompressorPhase, Prelude};
use folded_streaming_rans::bvgraph::mock_writers::{ANSymbolTable, EntropyMockWriter, Log2MockWriter};

const NODES: usize = 325557;
const ARCS: usize = 3216152;
const FIDELITY: usize = 2;
const RADIX: usize = 8;


fn main() -> Result<()> {
    let graph = webgraph::graph::bvgraph::load("tests/data/cnr-2000/cnr-2000")?;

    let binary_costs_table = ANSymbolTable::initialize_with_binary_cost(FIDELITY, RADIX);
    let model_builder = BVGraphModelBuilder::<Log2MockWriter>::new(binary_costs_table, FIDELITY, RADIX);

    // first iteration: build the model with log2mock
    let mut bvcomp = BVComp::<BVGraphModelBuilder<Log2MockWriter>>::new(model_builder, 7, 2, 3, 0);

    bvcomp.extend(graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();
    let symbol_freqs = model4encoder.get_symbol_freqs();
    let entropy_costs = ANSymbolTable::new(symbol_freqs, model4encoder.frame_sizes, FIDELITY, RADIX);
    let model_builder = BVGraphModelBuilder::<EntropyMockWriter>::new(entropy_costs.clone(), FIDELITY, RADIX);

    let mut bvcomp = BVComp::<BVGraphModelBuilder<EntropyMockWriter>>::new(model_builder, 7, 2, 3, 0);

    bvcomp.extend(graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();

    let mut bvcomp = BVComp::<BVGraphWriter>::new(
        BVGraphWriter::new(model4encoder, entropy_costs, FIDELITY, RADIX),
        7,
        2,
        3,
        0
    );

    // third iteration: encode with the entropy mock
    bvcomp.extend(graph.iter())?;

    // get phases and the encoder from the bvcomp
    let (encoder, phases) = bvcomp.flush()?.into_inner();
    let prelude = encoder.serialize();

    phases.store("cnr-2000-phases")?;
    prelude.store("cnr-2000-prelude")?;

    let prelude = Prelude::load_full("cnr-2000-prelude")?;
    let phases = Vec::<ANSCompressorPhase>::load_full("cnr-2000-phases")?;
    let code_reader_builder = ANSBVGraphReaderBuilder::new(&prelude, phases, FIDELITY, RADIX);

    let second_graph= BVGraph::<ANSBVGraphReaderBuilder, EmptyDict<usize, usize>>::new(
        code_reader_builder,
        2,
        7,
        NODES,
        ARCS,
    );

    let now = std::time::Instant::now();
    let mut arcs = 0;
    for node_index in 0..NODES {
        let decoded_successors = second_graph.successors(node_index).collect::<Vec<_>>();
        arcs += decoded_successors.len();
    }

    dbg!(now.elapsed());
    dbg!(now.elapsed().as_nanos() / arcs as u128);

    Ok(())
}