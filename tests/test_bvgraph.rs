use anyhow::Result;
use std::iter::Iterator;

use folded_streaming_rans::bvgraph::factories::bvgraph_decoder_factory::ANSBVGraphDecoderFactory;
use folded_streaming_rans::bvgraph::writers::bvgraph_encoder::ANSBVGraphEncodeAndEstimate;

use dsi_bitstream::prelude::BE;
use folded_streaming_rans::bvgraph::estimators::entropy_estimator::EntropyEstimator;
use folded_streaming_rans::bvgraph::estimators::log2_estimator::Log2Estimator;
use folded_streaming_rans::bvgraph::factories::bvgraphseq_decoder_factory::ANSBVGraphSeqDecoderFactory;
use folded_streaming_rans::bvgraph::random_access::ANSBVGraph;
use folded_streaming_rans::bvgraph::sequential::ANSBVGraphSeq;
use folded_streaming_rans::bvgraph::writers::bvgraph_model_builder::BVGraphModelBuilder;
use folded_streaming_rans::State;
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
    let mut model_builder = <BVGraphModelBuilder<Log2Estimator>>::new(log_mock);
    let mut bvcomp = BvComp::new(&mut model_builder, 7, 3, 2, 0);

    // first iteration -> build the model with log2 mock writer
    for node_index in 0..graph.num_nodes() {
        let successors = graph
            .successors(node_index)
            .map(|x| x.0)
            .collect::<Vec<_>>();
        bvcomp.push(successors)?;
    }

    bvcomp.flush()?;
    let model4encoder = model_builder.build();
    let folding_params = model4encoder.get_folding_params();
    let entropic_costs_table = EntropyEstimator::new(&model4encoder, folding_params);
    let mut model_builder =
        BVGraphModelBuilder::<EntropyEstimator>::new(entropic_costs_table.clone());
    let mut bvcomp = BvComp::new(&mut model_builder, 7, 3, 2, 0);

    // second iteration -> build the model with entropic mock writer
    for node_index in 0..graph.num_nodes() {
        let successors = graph
            .successors(node_index)
            .map(|x| x.0)
            .collect::<Vec<_>>();
        bvcomp.push(successors)?;
    }
    bvcomp.flush()?;
    let model4encoder = model_builder.build();
    let mut enc = ANSBVGraphEncodeAndEstimate::new(model4encoder, entropic_costs_table, 6, 4, 7, 2);
    let mut bvcomp = BvComp::new(&mut enc, 7, 3, 2, 0);

    // now encode the graph
    for node_index in 0..graph.num_nodes() {
        let successors = graph
            .successors(node_index)
            .map(|x| x.0)
            .collect::<Vec<_>>();

        bvcomp.push(successors)?;
    }
    bvcomp.flush()?;
    let (prelude, phases) = enc.into_prelude_phases();

    // (4) create elias fano
    let states: Box<[State]> = phases.iter().map(|phase| phase.state).collect();
    let ef = ANSBVGraph::build_ef(phases, graph.num_nodes())?;
    let code_reader_builder = ANSBVGraphDecoderFactory::new(prelude, ef, states);
    let decoded_graph = BvGraph::<ANSBVGraphDecoderFactory>::new(code_reader_builder, 6, 4, 7, 2);

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
    let graph = BvGraph::with_basename("tests/data/cnr-2000/cnr-2000")
        .endianness::<BE>()
        .load()?;
    let num_nodes = graph.num_nodes();
    let num_arcs = graph.num_arcs_hint().unwrap();

    let log2_mock = Log2Estimator::default();
    let mut model_builder = BVGraphModelBuilder::<Log2Estimator>::new(log2_mock);
    let mut bvcomp = BvComp::new(&mut model_builder, 7, 3, 2, 0);

    // First iteration with Log2MockWriter
    bvcomp.extend(graph.iter())?;
    bvcomp.flush()?;
    let model4encoder = model_builder.build();
    let folding_params = model4encoder.get_folding_params();
    let entropic_mock = EntropyEstimator::new(&model4encoder, folding_params);
    let mut model_builder = BVGraphModelBuilder::<EntropyEstimator>::new(entropic_mock.clone());
    let mut bvcomp = BvComp::new(&mut model_builder, 7, 3, 2, 0);

    // second iteration with EntropyMockWriter
    bvcomp.extend(graph.iter())?;
    bvcomp.flush()?;
    let model4encoder = model_builder.build();
    let mut enc =
        ANSBVGraphEncodeAndEstimate::new(model4encoder, entropic_mock, num_nodes, num_arcs, 7, 2);

    let mut bvcomp = BvComp::new(&mut enc, 7, 3, 2, 0);

    // Encoding the graph
    bvcomp.extend(graph.iter())?;
    bvcomp.flush()?;
    let (prelude, phases) = enc.into_prelude_phases();

    // (4) create elias fano
    let states: Box<[State]> = phases.iter().map(|phase| phase.state).collect();
    let ef = ANSBVGraph::build_ef(phases, num_nodes)?;
    let code_reader_builder = ANSBVGraphDecoderFactory::new(prelude, ef, states);

    let decoded_graph =
        BvGraph::<ANSBVGraphDecoderFactory>::new(code_reader_builder, num_nodes, num_arcs, 7, 2);

    for node_index in 0..graph.num_nodes() {
        let successors = graph.outdegree(node_index);
        let decoded_successors = decoded_graph.outdegree(node_index);

        assert_eq!(successors, decoded_successors);
    }

    Ok(())
}

#[test]
fn decodes_correctly_sequential_cnr_graph() -> Result<()> {
    let graph = BvGraphSeq::with_basename("tests/data/cnr-2000/cnr-2000")
        .endianness::<BE>()
        .load()?;
    let num_nodes = graph.num_nodes();
    let num_arcs = graph.num_arcs_hint().unwrap();

    let log2_mock = Log2Estimator::default();
    let mut model_builder = BVGraphModelBuilder::<Log2Estimator>::new(log2_mock);
    let mut bvcomp = BvComp::new(&mut model_builder, 7, 3, 2, 0);

    // First iteration with Log2MockWriter
    bvcomp.extend(graph.iter())?;
    bvcomp.flush()?;
    let model4encoder = model_builder.build();
    let folding_params = model4encoder.get_folding_params();
    let entropic_mock = EntropyEstimator::new(&model4encoder, folding_params);
    let mut model_builder = BVGraphModelBuilder::<EntropyEstimator>::new(entropic_mock.clone());
    let mut bvcomp = BvComp::new(&mut model_builder, 7, 3, 2, 0);

    // second iteration with EntropyMockWriter
    bvcomp.extend(graph.iter())?;
    bvcomp.flush()?;
    let model4encoder = model_builder.build();

    let mut enc =
        ANSBVGraphEncodeAndEstimate::new(model4encoder, entropic_mock, num_nodes, num_arcs, 7, 2);
    let mut bvcomp = BvComp::new(&mut enc, 7, 3, 2, 0);

    // Encoding the graph
    bvcomp.extend(graph.iter())?;
    bvcomp.flush()?;
    let (prelude, _phases) = enc.into_prelude_phases();

    let code_reader_builder = ANSBVGraphSeqDecoderFactory::new(prelude);

    let decoded_graph = BvGraphSeq::<ANSBVGraphSeqDecoderFactory>::new(
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
fn decodes_correctly_cnr_written_and_loaded_from_disk() -> Result<()> {
    let graph = BvGraph::with_basename("tests/data/cnr-2000/cnr-2000")
        .endianness::<BE>()
        .load()?;

    // (1) encode the BVGraph
    ANSBVGraph::store(
        "tests/data/cnr-2000/cnr-2000",
        "tests/data/cnr-2000/results",
        7,
        3,
        2,
    )?;

    // (3) load the compressed graph from disk
    let decoded_graph = ANSBVGraph::load("tests/data/cnr-2000/results")?;

    for node_index in 0..decoded_graph.num_nodes() {
        let successors = graph.successors(node_index).collect::<Vec<_>>();
        let decoded_successors = decoded_graph.successors(node_index).collect::<Vec<_>>();

        assert_eq!(successors, decoded_successors);
    }

    Ok(())
}

#[test]
fn decodes_correctly_sequential_cnr_written_and_loaded_from_disk() -> Result<()> {
    let graph = BvGraph::with_basename("tests/data/cnr-2000/cnr-2000")
        .endianness::<BE>()
        .load()?;

    // (1) encode the BVGraph
    ANSBVGraph::store(
        "tests/data/cnr-2000/cnr-2000",
        "tests/data/cnr-2000/results",
        7,
        3,
        2,
    )?;

    // (3) load the compressed graph from disk
    let decoded_graph = ANSBVGraphSeq::load("tests/data/cnr-2000/results")?;

    for_![ ((_, s), (_, t)) in lender::zip(graph.iter(), decoded_graph.iter()){
        assert!(itertools::equal(s, t));
    }];

    Ok(())
}
