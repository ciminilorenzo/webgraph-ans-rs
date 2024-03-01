use anyhow::Result;
use std::iter::Iterator;
use std::path::PathBuf;

use folded_streaming_rans::bvgraph::reader::{
    ANSBVGraphDecoderFactory, ANSBVGraphSeqDecoderFactory,
};
use folded_streaming_rans::bvgraph::writer::{ANSBVGraphMeasurableEncoder, BVGraphModelBuilder};

use dsi_bitstream::prelude::BE;
use epserde::prelude::Serialize;
use folded_streaming_rans::bvgraph::mock_writers::{EntropyEstimator, Log2Estimator};
use folded_streaming_rans::bvgraph::random_access::ANSBVGraph;
use folded_streaming_rans::bvgraph::sequential::ANSBVGraphSeq;
use lender::for_;
use webgraph::prelude::*;

#[test]
fn decodes_correctly_dummy_graph() -> Result<()> {
    let mut graph = VecGraph::new();

    for i in 0..=5 {
        graph.add_node(i);
    }

    graph.add_arc(4, 0); // 4 -> 0 -> 2
    graph.add_arc(0, 2); //       `-> 3
    graph.add_arc(0, 3); // 1 -> 5
    graph.add_arc(1, 5);

    let log_mock = Log2Estimator::default();
    let model_builder = BVGraphModelBuilder::new(log_mock);
    let mut bvcomp = BVComp::<BVGraphModelBuilder<Log2Estimator>>::new(model_builder, 7, 3, 2, 0);

    // first iteration -> build the model with log2 mock writer
    for node_index in 0..graph.num_nodes() {
        let successors = graph
            .successors(node_index)
            .map(|x| x.0)
            .collect::<Vec<_>>();
        bvcomp.push(successors)?;
    }

    let model4encoder = bvcomp.flush()?.build();
    let folding_params = model4encoder.get_folding_params();
    let entropic_costs_table = EntropyEstimator::new(&model4encoder, folding_params);
    let model_builder = BVGraphModelBuilder::<EntropyEstimator>::new(entropic_costs_table.clone());
    let mut bvcomp =
        BVComp::<BVGraphModelBuilder<EntropyEstimator>>::new(model_builder, 7, 3, 2, 0);

    // second iteration -> build the model with entropic mock writer
    for node_index in 0..graph.num_nodes() {
        let successors = graph
            .successors(node_index)
            .map(|x| x.0)
            .collect::<Vec<_>>();
        bvcomp.push(successors)?;
    }

    let model4encoder = bvcomp.flush()?.build();
    let mut bvcomp = BVComp::<ANSBVGraphMeasurableEncoder>::new(
        ANSBVGraphMeasurableEncoder::new(model4encoder, entropic_costs_table, 6, 4, 7, 2),
        7,
        3,
        2,
        0,
    );

    // now encode the graph
    for node_index in 0..graph.num_nodes() {
        let successors = graph
            .successors(node_index)
            .map(|x| x.0)
            .collect::<Vec<_>>();

        bvcomp.push(successors)?;
    }

    let (prelude, mut phases) = bvcomp.flush()?.into_inner();

    // let's reverse the phases so that the first phase is associated to the last node encoded
    phases.reverse();

    let code_reader_builder = ANSBVGraphDecoderFactory::new(prelude, phases);

    let decoded_graph = BVGraph::<ANSBVGraphDecoderFactory>::new(code_reader_builder, 6, 4, 7, 2);

    assert_eq!(
        decoded_graph.successors(0).collect::<Vec<_>>(),
        vec![2usize, 3]
    );
    assert_eq!(
        decoded_graph.successors(1).collect::<Vec<_>>(),
        vec![5usize]
    );
    assert_eq!(
        decoded_graph.successors(2).collect::<Vec<_>>(),
        Vec::<usize>::new()
    );
    Ok(())
}

#[test]
fn decodes_correctly_cnr_graph() -> Result<()> {
    let graph = BVGraph::with_basename("tests/data/cnr-2000/cnr-2000")
        .endianness::<BE>()
        .load()?;
    let num_nodes = graph.num_nodes();
    let num_arcs = graph.num_arcs_hint().unwrap();

    let log2_mock = Log2Estimator::default();
    let model_builder = BVGraphModelBuilder::<Log2Estimator>::new(log2_mock);
    let mut bvcomp = BVComp::<BVGraphModelBuilder<Log2Estimator>>::new(model_builder, 7, 3, 2, 0);

    // First iteration with Log2MockWriter
    bvcomp.extend(graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();
    let folding_params = model4encoder.get_folding_params();
    let entropic_mock = EntropyEstimator::new(&model4encoder, folding_params);
    let model_builder = BVGraphModelBuilder::<EntropyEstimator>::new(entropic_mock.clone());
    let mut bvcomp =
        BVComp::<BVGraphModelBuilder<EntropyEstimator>>::new(model_builder, 7, 3, 2, 0);

    // second iteration with EntropyMockWriter
    bvcomp.extend(graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();

    let mut bvcomp = BVComp::<ANSBVGraphMeasurableEncoder>::new(
        ANSBVGraphMeasurableEncoder::new(model4encoder, entropic_mock, num_nodes, num_arcs, 7, 2),
        7,
        3,
        2,
        0,
    );

    // Encoding the graph
    bvcomp.extend(graph.iter())?;

    let (prelude, mut phases) = bvcomp.flush()?.into_inner();

    // let's reverse the phases so that the first phase is associated to the last node encoded
    phases.reverse();

    let code_reader_builder = ANSBVGraphDecoderFactory::new(prelude, phases);

    let decoded_graph =
        BVGraph::<ANSBVGraphDecoderFactory>::new(code_reader_builder, num_nodes, num_arcs, 7, 2);

    for node_index in 0..graph.num_nodes() {
        let successors = graph.outdegree(node_index);
        let decoded_successors = decoded_graph.outdegree(node_index);

        assert_eq!(successors, decoded_successors);
    }

    Ok(())
}

#[test]
fn decodes_correctly_sequential_cnr_graph() -> Result<()> {
    let graph = BVGraphSeq::with_basename("tests/data/cnr-2000/cnr-2000")
        .endianness::<BE>()
        .load()?;
    let num_nodes = graph.num_nodes();
    let num_arcs = graph.num_arcs_hint().unwrap();

    let log2_mock = Log2Estimator::default();
    let model_builder = BVGraphModelBuilder::<Log2Estimator>::new(log2_mock);
    let mut bvcomp = BVComp::<BVGraphModelBuilder<Log2Estimator>>::new(model_builder, 7, 3, 2, 0);

    // First iteration with Log2MockWriter
    bvcomp.extend(graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();
    let folding_params = model4encoder.get_folding_params();
    let entropic_mock = EntropyEstimator::new(&model4encoder, folding_params);
    let model_builder = BVGraphModelBuilder::<EntropyEstimator>::new(entropic_mock.clone());
    let mut bvcomp =
        BVComp::<BVGraphModelBuilder<EntropyEstimator>>::new(model_builder, 7, 3, 2, 0);

    // second iteration with EntropyMockWriter
    bvcomp.extend(graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();

    let mut bvcomp = BVComp::<ANSBVGraphMeasurableEncoder>::new(
        ANSBVGraphMeasurableEncoder::new(model4encoder, entropic_mock, num_nodes, num_arcs, 7, 2),
        7,
        3,
        2,
        0,
    );

    // Encoding the graph
    bvcomp.extend(graph.iter())?;

    let (prelude, _phases) = bvcomp.flush()?.into_inner();

    let code_reader_builder = ANSBVGraphSeqDecoderFactory::new(prelude);

    let decoded_graph = BVGraphSeq::<ANSBVGraphSeqDecoderFactory>::new(
        code_reader_builder,
        num_nodes,
        Some(num_arcs),
        7,
        2,
    );

    for_![ ((_, s), (_, t)) in lender::zip(graph.iter(), decoded_graph.iter()){
        assert!(itertools::equal(s, t));
    }];

    Ok(())
}

#[test]
fn decodes_correctly_cnr_loaded_from_disk() -> Result<()> {
    // (1) encode the graph
    let graph = BVGraph::with_basename("tests/data/cnr-2000/cnr-2000")
        .endianness::<BE>()
        .load()?;
    let num_nodes = graph.num_nodes();
    let num_arcs = graph.num_arcs_hint().unwrap();

    let log2_mock = Log2Estimator::default();
    let model_builder = BVGraphModelBuilder::<Log2Estimator>::new(log2_mock);
    let mut bvcomp = BVComp::<BVGraphModelBuilder<Log2Estimator>>::new(model_builder, 7, 3, 2, 0);

    // First iteration with Log2MockWriter
    bvcomp.extend(graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();
    let folding_params = model4encoder.get_folding_params();
    let entropic_mock = EntropyEstimator::new(&model4encoder, folding_params);
    let model_builder = BVGraphModelBuilder::<EntropyEstimator>::new(entropic_mock.clone());
    let mut bvcomp =
        BVComp::<BVGraphModelBuilder<EntropyEstimator>>::new(model_builder, 7, 3, 2, 0);

    // second iteration with EntropyMockWriter
    bvcomp.extend(graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();

    let mut bvcomp = BVComp::<ANSBVGraphMeasurableEncoder>::new(
        ANSBVGraphMeasurableEncoder::new(model4encoder, entropic_mock, num_nodes, num_arcs, 7, 2),
        7,
        3,
        2,
        0,
    );

    // Encoding the graph
    bvcomp.extend(graph.iter())?;

    let (prelude, phases) = bvcomp.flush()?.into_inner();

    // (2) store the results of compression on disk on ./results.ans and ./results.phases
    let mut buf = PathBuf::from("results");
    buf.set_extension("ans");
    prelude.store(buf.as_path())?;
    buf.set_extension("phases");
    phases.store(buf.as_path())?;

    // (3) load the compressed graph from disk
    let decoded_graph = ANSBVGraph::load("results")?;

    for node_index in 0..graph.num_nodes() {
        let successors = graph.outdegree(node_index);
        let decoded_successors = decoded_graph.outdegree(node_index);

        assert_eq!(successors, decoded_successors);
    }

    Ok(())
}

#[test]
fn decodes_correctly_sequential_cnr_loaded_from_disk() -> Result<()> {
    let graph = BVGraphSeq::with_basename("tests/data/cnr-2000/cnr-2000")
        .endianness::<BE>()
        .load()?;
    let num_nodes = graph.num_nodes();
    let num_arcs = graph.num_arcs_hint().unwrap();

    let log2_mock = Log2Estimator::default();
    let model_builder = BVGraphModelBuilder::<Log2Estimator>::new(log2_mock);
    let mut bvcomp = BVComp::<BVGraphModelBuilder<Log2Estimator>>::new(model_builder, 7, 3, 2, 0);

    // First iteration with Log2MockWriter
    bvcomp.extend(graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();
    let folding_params = model4encoder.get_folding_params();
    let entropic_mock = EntropyEstimator::new(&model4encoder, folding_params);
    let model_builder = BVGraphModelBuilder::<EntropyEstimator>::new(entropic_mock.clone());
    let mut bvcomp =
        BVComp::<BVGraphModelBuilder<EntropyEstimator>>::new(model_builder, 7, 3, 2, 0);

    // second iteration with EntropyMockWriter
    bvcomp.extend(graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();

    let mut bvcomp = BVComp::<ANSBVGraphMeasurableEncoder>::new(
        ANSBVGraphMeasurableEncoder::new(model4encoder, entropic_mock, num_nodes, num_arcs, 7, 2),
        7,
        3,
        2,
        0,
    );

    // Encoding the graph
    bvcomp.extend(graph.iter())?;

    let (prelude, _phases) = bvcomp.flush()?.into_inner();

    // (2) store the results of compression on disk on ./results.ans
    let mut buf = PathBuf::from("results");
    buf.set_extension("ans");
    prelude.store(buf.as_path())?;

    // (3) load the compressed graph from disk
    let decoded_graph = ANSBVGraphSeq::load("results")?;

    for_![ ((_, s), (_, t)) in lender::zip(graph.iter(), decoded_graph.iter()){
        assert!(itertools::equal(s, t));
    }];

    Ok(())
}
