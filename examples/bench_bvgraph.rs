use epserde::prelude::{Deserialize, Serialize};

use webgraph::prelude::{BVComp, BVGraph, EmptyDict, RandomAccessLabelling, SequentialLabelling};

use folded_streaming_rans::bvgraph::reader::ANSBVGraphReaderBuilder;
use folded_streaming_rans::bvgraph::writer::{BVGraphModelBuilder, BVGraphWriter};
use folded_streaming_rans::multi_model_ans::Prelude;

use anyhow::Result;
use folded_streaming_rans::bvgraph::mock_writers::{EntropyMockWriter, Log2MockWriter, MockWriter};
use folded_streaming_rans::multi_model_ans::encoder::ANSCompressorPhase;
use folded_streaming_rans::utils::ans_utilities::get_symbol_costs_table;

const NODES: usize = 325557;
const ARCS: usize = 3216152;
const FIDELITY: usize = 2;
const RADIX: usize = 8;


//    This bench checks how many nanoseconds are needed to successfully decode 10k arcs from the
//    cnr-2000 graph.

fn encode_graph() -> Result<()> {
    let graph = webgraph::graph::bvgraph::load("tests/data/cnr-2000/cnr-2000")?;

    let model_builder = BVGraphModelBuilder::<FIDELITY, RADIX, Log2MockWriter>::new(Vec::new());
    let mut bvcomp = BVComp::<BVGraphModelBuilder<FIDELITY, RADIX, Log2MockWriter>>::new(
        model_builder,
        7,
        2,
        3,
        0
    );

    // second iteration: build the model with log2mock
    bvcomp.extend(graph.iter())?;
    let encoder = bvcomp.flush()?.build();

    // Salvare un riferimento del mocker all'interno del bv comp o del model builder?

    let symbol_costs_table = get_symbol_costs_table(&encoder.tables, &encoder.frame_sizes, FIDELITY, RADIX);
    let model_builder = BVGraphModelBuilder::<FIDELITY, RADIX, EntropyMockWriter>::new(symbol_costs_table);
    let mut bvcomp = BVComp::<BVGraphModelBuilder<FIDELITY, RADIX, EntropyMockWriter>>::new(
        model_builder,
        7,
        2,
        3,
        0
    );

    // second iteration: build the model with entropy mock
    bvcomp.extend(graph.iter())?;
    let encoder = bvcomp.flush()?.build();


    let mut bvcomp = BVComp::<BVGraphWriter<FIDELITY, RADIX>>::new(
        BVGraphWriter::new(encoder),
        7,
        2,
        3,
        0
    );

    // third iteration: encode with the entropy mock
    bvcomp.extend(graph.iter())?;

    // get phases and the encoder from the bvcomp
    let (mut encoder, phases) = bvcomp.flush()?.into_inner();
    let prelude = encoder.serialize();

    phases.store("cnr-2000-phases")?;
    prelude.store("cnr-2000-prelude")?;
    Ok(())
}

// !!!! REMEMBER TO DELETE THE FILES BEFORE RUNNING THIS BENCH IF YOU HAVE CHANGED MOCK WRITER!!!!
fn main() -> Result<()> {
    let (prelude, phases) = match (
        <Prelude<RADIX>>::load_full("cnr-2000-prelude"),
        <Vec<ANSCompressorPhase>>::load_full("cnr-2000-phases"))
    {
        // if the files are present, load them
        (Ok(prelude), Ok(phases)) => (prelude, phases),
        // otherwise, encode the graph and load the files
        _ => {
            encode_graph()?;
            (
                <Prelude<8>>::load_full("cnr-2000-prelude")?,
                <Vec<ANSCompressorPhase>>::load_full("cnr-2000-phases")?,
            )
        }
    };

    let code_reader_builder = ANSBVGraphReaderBuilder::<FIDELITY, RADIX>::new(prelude, phases);

    let second_graph= BVGraph::<ANSBVGraphReaderBuilder<FIDELITY, RADIX>, EmptyDict<usize, usize>>::new(
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