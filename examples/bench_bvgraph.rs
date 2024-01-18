use epserde::prelude::{Deserialize, Serialize};

use webgraph::prelude::{BVComp, BVGraph, EmptyDict, RandomAccessLabelling, SequentialLabelling};

use folded_streaming_rans::bvgraph::reader::ANSBVGraphReaderBuilder;
use folded_streaming_rans::bvgraph::writer::{BVGraphModelBuilder, BVGraphWriter};
use folded_streaming_rans::multi_model_ans::Prelude;

use anyhow::Result;
use folded_streaming_rans::multi_model_ans::encoder::ANSCompressorPhase;
use folded_streaming_rans::utils::ans_utilities::get_mock_writer;

const NODES: usize = 325557;
const ARCS: usize = 3216152;


//    This bench checks how many nanoseconds are needed to successfully decode 10k arcs from the
//    cnr-2000 graph.

fn encode_graph() -> Result<()> {
    let graph = webgraph::graph::bvgraph::load("tests/data/cnr-2000")?;

    let model_builder = BVGraphModelBuilder::<2, 8>::new();
    let mut bvcomp = BVComp::<BVGraphModelBuilder<2, 8>>::new(
        model_builder,
        7,
        2,
        3,
        0
    );

    // first iteration: builds the statistics for each model
    bvcomp.extend(graph.iter())?;
    let encoder = bvcomp.flush()?.build();
    let mock_writer = get_mock_writer(&encoder.tables, &encoder.frame_sizes);

    let mut bvcomp = BVComp::<BVGraphWriter<2, 8, Vec<u8>>>::new(
        BVGraphWriter::new(encoder, mock_writer),
        7,
        2,
        3,
        0
    );

    // second iteration: encodes the graph
    bvcomp.extend(graph.iter())?;

    // get phases and the encoder from the bvcomp
    let (mut encoder, phases) = bvcomp.flush()?.into_inner();
    let prelude = encoder.serialize();

    phases.store("cnr-2000-phases")?;
    prelude.store("cnr-2000-prelude")?;
    Ok(())
}

fn main() -> Result<()> {
    let (prelude, phases) = match (
        <Prelude<8, Vec<u8>>>::load_full("cnr-2000-prelude"),
        <Vec<ANSCompressorPhase>>::load_full("cnr-2000-phases"))
    {
        // if the files are present, load them
        (Ok(prelude), Ok(phases)) => (prelude, phases),
        // otherwise, encode the graph and load the files
        _ => {
            encode_graph()?;
            (
                <Prelude<8, Vec<u8>>>::load_full("cnr-2000-prelude")?,
                <Vec<ANSCompressorPhase>>::load_full("cnr-2000-phases")?,
            )
        }
    };

    let code_reader_builder = ANSBVGraphReaderBuilder::<2>::new(prelude, phases);

    let second_graph= BVGraph::<ANSBVGraphReaderBuilder<2>, EmptyDict<usize, usize>>::new(
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