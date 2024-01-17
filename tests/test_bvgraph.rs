use folded_streaming_rans::bvgraph::writer::{BVGraphModelBuilder, BVGraphWriter};
use anyhow::Result;
use epserde::prelude::*;
use lender::Lender;

use webgraph::{graph::bvgraph::BVComp, traits::SequentialLabelling};
use webgraph::prelude::{BVGraph, EmptyDict, RandomAccessLabelling};
use folded_streaming_rans::bvgraph::reader::ANSBVGraphReaderBuilder;


#[test]
fn decoder_decodes_correctly_dummy_graph() -> Result<()> {
    let mut graph = webgraph::graph::vec_graph::VecGraph::new();

    for i in 0..=5 {
        graph.add_node(i);
    }

    graph.add_arc(4, 0);
    graph.add_arc(0, 2);
    graph.add_arc(0, 3);
    graph.add_arc(1, 5);

    // 4 -> 0 -> 2
    //       `-> 3
    // 1 -> 5

    let dir = tempfile::tempdir()?;
    let model_builder = BVGraphModelBuilder::<2, 8>::new(dir.path().join("model"));
    let mut bvcomp = BVComp::<BVGraphModelBuilder<2, 8>>::new(
        model_builder,
        7,
        2,
        3,
        0
    );

    // first iteration
    for node_index in 0..graph.num_nodes() {
        let successors = graph
            .successors(node_index)
            .map(|x| x.0)
            .collect::<Vec<_>>();

        bvcomp.push(successors)?;
    }

    let encoder = bvcomp.flush()?.build();

    let mut bvcomp = BVComp::<BVGraphWriter<2, 8, Vec<u8>>>::new(
        BVGraphWriter::new(encoder),
        7,
        2,
        3,
        0
    );

    // second iteration: encodes the graph
    for node_index in 0..graph.num_nodes() {
        let successors = graph
            .successors(node_index)
            .map(|x| x.0)
            .collect::<Vec<_>>();

        bvcomp.push(successors)?;
    }

    let (mut encoder, phases) = bvcomp.flush()?.into_inner();
    let prelude = encoder.serialize();

    let code_reader_builder = ANSBVGraphReaderBuilder::<2>::new(prelude, phases);

    let second_graph= BVGraph::<ANSBVGraphReaderBuilder<2>, EmptyDict<usize, usize>>::new(
        code_reader_builder,
        2,
        7,
        6,
        4,
    );

    assert_eq!(second_graph.successors(0).collect::<Vec<_>>(), vec![2, 3]);
    assert_eq!(second_graph.successors(1).collect::<Vec<_>>(), vec![5]);
    assert_eq!(second_graph.successors(2).collect::<Vec<_>>(), vec![]);
    Ok(())
}

#[test]
fn decoder_decodes_correctly_cnr_graph() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let graph = webgraph::graph::bvgraph::load("tests/data/cnr-2000")?;
    let num_nodes = graph.num_nodes();
    let num_arcs = graph.num_arcs_hint().unwrap();
    let model_builder = BVGraphModelBuilder::<2, 8>::new(dir.path().join("model"));
    let mut bvcomp = BVComp::<BVGraphModelBuilder<2, 8>>::new(
        model_builder,
        7,
        2,
        3,
        0
    );

    bvcomp.extend(graph.iter())?;
    let encoder = bvcomp.flush()?.build();

    let mut bvcomp = BVComp::<BVGraphWriter<2, 8, Vec<u8>>>::new(
        BVGraphWriter::new(encoder),
        7,
        2,
        3,
        0
    );

    bvcomp.extend(graph.iter())?;

    let (mut encoder, phases) = bvcomp.flush()?.into_inner();
    let prelude = encoder.serialize();

    let code_reader_builder = ANSBVGraphReaderBuilder::<2>::new(prelude, phases);

    let decoded_graph = BVGraph::<ANSBVGraphReaderBuilder<2>, EmptyDict<usize, usize>>::new(
        code_reader_builder,
        2,
        7,
        num_nodes,
        num_arcs,
    );

    for node_index in 0..graph.num_nodes() {
        let successors = graph.outdegree(node_index);
        let decoded_successors = decoded_graph.outdegree(node_index);

        assert_eq!(successors, decoded_successors);
    }

    Ok(())
}



