use folded_streaming_rans::bvgraph::writer::{BVGraphModelBuilder, BVGraphWriter};

use anyhow::Result;
use epserde::prelude::*;
use lender::Lender;
use mem_dbg::*;
use webgraph::{graph::bvgraph::BVComp, traits::SequentialLabelling};
use webgraph::prelude::{BVGraph, EmptyDict, RandomAccessLabelling};
use folded_streaming_rans::bvgraph::reader::ANSBVGraphReaderBuilder;
use folded_streaming_rans::multi_model_ans::Prelude;


#[test]
fn test_model_builder() -> Result<()> {
    env_logger::builder().is_test(true).try_init().unwrap();

    let dir = tempfile::tempdir()?;
    let graph = webgraph::graph::bvgraph::load_seq("tests/data/cnr-2000")?;
    let model_builder = BVGraphModelBuilder::<2, 8>::new(dir.path().join("model"));
    let mut bvcomp = BVComp::<BVGraphModelBuilder<2, 8>>::new(model_builder, 7, 2, 3, 0);

    // first iteration: builds the statistics for each model
    bvcomp.extend(graph.iter())?;
    let encoder = bvcomp.flush()?.build();

    let mut bvcomp = BVComp::<BVGraphWriter<2, 8, Vec<u8>>>::new(
        BVGraphWriter::new(encoder),
        7,
        2,
        3,
        0
    );

    // second iteration: encodes the graph
    bvcomp.extend(graph.iter())?;

    let mut encoder = bvcomp.flush()?.into_inner().0;
    let prelude = encoder.serialize();

    prelude.mem_dbg(DbgFlags::default())?;
    // prelude.store("pippo")?;
    // let prelude2 = <Prelude<8, Vec<u8>>>::load_mem("pippo")?;

    Ok(())
}

#[test]
fn decompress() -> Result<()> {
    env_logger::builder().is_test(true).try_init().unwrap();

    let dir = tempfile::tempdir()?;
    let graph = webgraph::graph::bvgraph::load("tests/data/cnr-2000")?;
    let model_builder = BVGraphModelBuilder::<2, 8>::new(dir.path().join("model"));
    let mut bvcomp = BVComp::<BVGraphModelBuilder<2, 8>>::new(
        model_builder,
        7,
        2,
        3,
        0
    );

    println!("original {:?}",graph.successors(10).collect::<Vec<_>>());
    println!("original {:?}",graph.successors(11).collect::<Vec<_>>());
    println!("original {:?}",graph.successors(12).collect::<Vec<_>>());

    let nodes = graph.num_nodes();
    let arcs = graph.num_arcs_hint().unwrap();

    // first iteration: builds the statistics for each model
    bvcomp.extend(graph.iter())?;

    let encoder = bvcomp.flush()?.build();

    let mut bvcomp = BVComp::<BVGraphWriter<2, 8, Vec<u8>>>::new(
        BVGraphWriter::new(encoder),
        7,
        2,
        3,
        0
    );

    // second iteration: encodes the graph
    bvcomp.extend(graph.iter())?;

    let (mut encoder, phases) = bvcomp.flush()?.into_inner();

    let prelude = encoder.serialize();

    let code_reader_builder = ANSBVGraphReaderBuilder::<2>::new(prelude, phases);

    let second_graph= BVGraph::<ANSBVGraphReaderBuilder<2>, EmptyDict<usize, usize>>::new(
        code_reader_builder,
        2,
        7,
        nodes,
        arcs,
    );

    println!("nodes: {}", nodes);

    println!("{:?}", second_graph.successors(10).collect::<Vec<_>>());
    println!("{:?}", second_graph.successors(11).collect::<Vec<_>>());
    println!("{:?}", second_graph.successors(12).collect::<Vec<_>>());

    Ok(())
}

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
fn decoder_decodes_correctly_graph() -> Result<()> {
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

    // first iteration: builds the statistics for each model
    bvcomp.extend(graph.iter())?;
    let encoder = bvcomp.flush()?.build();

    let mut bvcomp = BVComp::<BVGraphWriter<2, 8, Vec<u8>>>::new(
        BVGraphWriter::new(encoder),
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

    let code_reader_builder = ANSBVGraphReaderBuilder::<2>::new(prelude, phases);

    let second_graph= BVGraph::<ANSBVGraphReaderBuilder<2>, EmptyDict<usize, usize>>::new(
        code_reader_builder,
        2,
        7,
        num_nodes,
        num_arcs,
    );

    for node_index in 0..graph.num_nodes() {
        let successors = graph.outdegree(node_index);
        let decoded_successors = second_graph.outdegree(node_index);

        assert_eq!(successors, decoded_successors);
    }

    Ok(())
}

#[test]
fn decoder_decodes_correctly_deserialized_prelude() -> Result <()> {
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

    // first iteration: builds the statistics for each model
    bvcomp.extend(graph.iter())?;
    let encoder = bvcomp.flush()?.build();

    let mut bvcomp = BVComp::<BVGraphWriter<2, 8, Vec<u8>>>::new(
        BVGraphWriter::new(encoder),
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

    prelude.store("cnr-2000-prelude")?;
    let deserialized = <Prelude<8, Vec<u8>>>::load_full("cnr-2000-prelude")?;

    // prelude.mem_dbg(DbgFlags::default())?;

    let code_reader_builder = ANSBVGraphReaderBuilder::<2>::new(deserialized, phases);

    let second_graph= BVGraph::<ANSBVGraphReaderBuilder<2>, EmptyDict<usize, usize>>::new(
        code_reader_builder,
        2,
        7,
        num_nodes,
        num_arcs,
    );


    for node_index in 0..graph.num_nodes() {
        let successors = graph.successors(node_index).collect::<Vec<_>>();
        let decoded_successors = second_graph.successors(node_index).collect::<Vec<_>>();

        assert_eq!(successors, decoded_successors);
    }

    Ok(())
}



