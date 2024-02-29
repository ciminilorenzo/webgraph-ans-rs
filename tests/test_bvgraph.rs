use anyhow::Result;
use std::iter::Iterator;

use folded_streaming_rans::bvgraph::reader::ANSBVGraphDecoderFactory;
use folded_streaming_rans::bvgraph::writer::{ANSBVGraphMeasurableEncoder, BVGraphModelBuilder};

use dsi_bitstream::prelude::BE;
use folded_streaming_rans::bvgraph::mock_writers::{EntropyEstimator, Log2Estimator};
use lender::for_;
use webgraph::prelude::*;

#[test]
fn decoder_decodes_correctly_dummy_graph() -> Result<()> {
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
    let mut bvcomp = BVComp::<BVGraphModelBuilder<Log2Estimator>>::new(model_builder, 7, 2, 3, 0);

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
        BVComp::<BVGraphModelBuilder<EntropyEstimator>>::new(model_builder, 7, 2, 3, 0);

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
        ANSBVGraphMeasurableEncoder::new(model4encoder, entropic_costs_table),
        7,
        2,
        3,
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

    let (encoder, phases) = bvcomp.flush()?.into_inner();
    let prelude = encoder.into_prelude();

    let code_reader_builder = ANSBVGraphDecoderFactory::new(&prelude, phases);

    let decoded_graph = BVGraph::<ANSBVGraphDecoderFactory>::new(code_reader_builder, 2, 7, 6, 4);

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
fn decoder_decodes_correctly_cnr_graph() -> Result<()> {
    stderrlog::new()
        .verbosity(2)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    let graph = BVGraph::with_basename("tests/data/cnr-2000/cnr-2000")
        .endianness::<BE>()
        .load()?;
    let num_nodes = graph.num_nodes();
    let num_arcs = graph.num_arcs_hint().unwrap();

    let log2_mock = Log2Estimator::default();
    let model_builder = BVGraphModelBuilder::<Log2Estimator>::new(log2_mock);
    let mut bvcomp = BVComp::<BVGraphModelBuilder<Log2Estimator>>::new(model_builder, 7, 2, 3, 0);

    // First iteration with Log2MockWriter
    bvcomp.extend(graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();
    let folding_params = model4encoder.get_folding_params();
    let entropic_mock = EntropyEstimator::new(&model4encoder, folding_params);
    let model_builder = BVGraphModelBuilder::<EntropyEstimator>::new(entropic_mock.clone());
    let mut bvcomp =
        BVComp::<BVGraphModelBuilder<EntropyEstimator>>::new(model_builder, 7, 2, 3, 0);

    // second iteration with EntropyMockWriter
    bvcomp.extend(graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();

    let mut bvcomp = BVComp::<ANSBVGraphMeasurableEncoder>::new(
        ANSBVGraphMeasurableEncoder::new(model4encoder, entropic_mock),
        7,
        2,
        3,
        0,
    );

    // Encoding the graph
    bvcomp.extend(graph.iter())?;

    let (encoder, phases) = bvcomp.flush()?.into_inner();
    let prelude = encoder.into_prelude();

    let code_reader_builder = ANSBVGraphDecoderFactory::new(&prelude, phases);

    let decoded_graph =
        BVGraph::<ANSBVGraphDecoderFactory>::new(code_reader_builder, 2, 7, num_nodes, num_arcs);

    for node_index in 0..graph.num_nodes() {
        let successors = graph.outdegree(node_index);
        let decoded_successors = decoded_graph.outdegree(node_index);

        assert_eq!(successors, decoded_successors);
    }

    Ok(())
}

#[test]
fn decoder_decodes_correctly_sequential_cnr_graph() -> Result<()> {
    stderrlog::new()
        .verbosity(2)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    let graph = BVGraphSeq::with_basename("tests/data/cnr-2000/cnr-2000")
        .endianness::<BE>()
        .load()?;
    let num_nodes = graph.num_nodes();
    let num_arcs = graph.num_arcs_hint().unwrap();

    let log2_mock = Log2Estimator::default();
    let model_builder = BVGraphModelBuilder::<Log2Estimator>::new(log2_mock);
    let mut bvcomp = BVComp::<BVGraphModelBuilder<Log2Estimator>>::new(model_builder, 7, 2, 3, 0);

    // First iteration with Log2MockWriter
    bvcomp.extend(graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();
    let folding_params = model4encoder.get_folding_params();
    let entropic_mock = EntropyEstimator::new(&model4encoder, folding_params);
    let model_builder = BVGraphModelBuilder::<EntropyEstimator>::new(entropic_mock.clone());
    let mut bvcomp =
        BVComp::<BVGraphModelBuilder<EntropyEstimator>>::new(model_builder, 7, 2, 3, 0);

    // second iteration with EntropyMockWriter
    bvcomp.extend(graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();

    let mut bvcomp = BVComp::<ANSBVGraphMeasurableEncoder>::new(
        ANSBVGraphMeasurableEncoder::new(model4encoder, entropic_mock),
        7,
        2,
        3,
        0,
    );

    // Encoding the graph
    bvcomp.extend(graph.iter())?;

    let (encoder, phases) = bvcomp.flush()?.into_inner();
    let prelude = encoder.into_prelude();

    // todo -> i can't create it without phases.
    let code_reader_builder = ANSBVGraphDecoderFactory::new(&prelude, phases);

    let decoded_graph = BVGraphSeq::<ANSBVGraphDecoderFactory>::new(
        code_reader_builder,
        7,
        2,
        num_nodes,
        Some(num_arcs),
    );

    for_![ ((_, s), (_, t)) in lender::zip(graph.iter(), decoded_graph.iter()){
        assert!(itertools::equal(s, t));
    }];

    Ok(())
}
