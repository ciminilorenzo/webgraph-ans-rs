use anyhow::Result;

use std::iter::Iterator;

use dsi_bitstream::prelude::BE;

use lender::for_;

use webgraph::prelude::*;

use webgraph_ans::bvgraph::estimators::entropy_estimator::EntropyEstimator;
use webgraph_ans::bvgraph::estimators::log2_estimator::Log2Estimator;
use webgraph_ans::bvgraph::factories::bvgraph_decoder_factory::ANSBVGraphDecoderFactory;
use webgraph_ans::bvgraph::random_access::ANSBvGraph;
use webgraph_ans::bvgraph::sequential::ANSBvGraphSeq;
use webgraph_ans::bvgraph::writers::bvgraph_encoder::ANSBVGraphEncodeAndEstimate;
use webgraph_ans::bvgraph::writers::bvgraph_model_builder::BVGraphModelBuilder;
use webgraph_ans::State;

/// Check that we are correctly able to first encode a dummy graph and then decode what previously
/// encoded. Since this graph is not a BvGraph, we have to do the whole pipeline that allows us
/// to encode it as a ANSBvGraph here.
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
    let ef = ANSBvGraph::build_ef(phases, graph.num_nodes())?;
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

/// Checks that we are correctly able to encode on disk the graph, then decode it as a
/// ANSBVGraph and check that what we decode is the same we previously encoded.
#[test]
fn decodes_correctly_random_access_graph() -> Result<()> {
    let original_graph = BvGraph::with_basename("tests/data/cnr-2000/cnr-2000")
        .endianness::<BE>()
        .load()?;

    ANSBvGraph::store(
        "tests/data/cnr-2000/cnr-2000",
        "tests/data/cnr-2000/results",
        7,
        3,
        2,
    )?;

    let decoded_graph = ANSBvGraph::load("tests/data/cnr-2000/results")?;

    for node_index in 0..decoded_graph.num_nodes() {
        let original_successors = original_graph.successors(node_index).collect::<Vec<_>>();
        let decoded_successors = decoded_graph.successors(node_index).collect::<Vec<_>>();

        assert_eq!(original_successors, decoded_successors);
    }

    Ok(())
}

/// Checks that we are correctly able to encode on disk the graph, then decode it as a
/// ANSBVGraphSeq and check that what we decode is the same we previously encoded.
#[test]
fn decodes_correctly_sequential_graph() -> Result<()> {
    let original_graph = BvGraphSeq::with_basename("tests/data/cnr-2000/cnr-2000")
        .endianness::<BE>()
        .load()?;

    ANSBvGraph::store(
        "tests/data/cnr-2000/cnr-2000",
        "tests/data/cnr-2000/results",
        7,
        3,
        2,
    )?;

    let decoded_graph = ANSBvGraphSeq::load("tests/data/cnr-2000/results")?;

    for_![ ((_, original_successors), (_, decoded_successors)) in lender::zip(original_graph.iter(), decoded_graph.iter()){
        assert!(itertools::equal(original_successors, decoded_successors));
    }];

    Ok(())
}
