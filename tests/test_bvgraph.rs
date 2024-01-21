
use anyhow::Result;
use folded_streaming_rans::bvgraph::writer::{BVGraphModelBuilder, BVGraphWriter};

use folded_streaming_rans::bvgraph::reader::ANSBVGraphReaderBuilder;
use webgraph::prelude::{BVGraph, EmptyDict, RandomAccessLabelling};
use webgraph::{graph::bvgraph::BVComp, traits::SequentialLabelling};
use folded_streaming_rans::bvgraph::mock_writers::{ANSymbolTable, EntropyMockWriter, Log2MockWriter};

const FIDELITY: usize = 2;
const RADIX: usize = 8;

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

    // let's pass a dummy table since the Log2MockWriter it's not going to use it
    let binary_costs_table = ANSymbolTable::<FIDELITY, RADIX>::initialize_with_binary_cost(9);
    let model_builder = BVGraphModelBuilder::<FIDELITY, RADIX, Log2MockWriter<FIDELITY, RADIX>>::new(binary_costs_table);

    let mut bvcomp = BVComp::<BVGraphModelBuilder<
        FIDELITY,
        RADIX,
        Log2MockWriter<FIDELITY, RADIX>
    >>::new(model_builder, 7, 2, 3, 0);

    // first iteration -> build the model with log2 mock writer
    for node_index in 0..graph.num_nodes() {
        let successors = graph
            .successors(node_index)
            .map(|x| x.0)
            .collect::<Vec<_>>();
        bvcomp.push(successors)?;
    }

    let mut model4encoder =  bvcomp.flush()?.build();
    let symbol_freqs = model4encoder.get_symbol_freqs();

    let entropic_costs_table = ANSymbolTable::<FIDELITY, RADIX>::new(
        symbol_freqs,
        model4encoder.frame_sizes.clone()
    );

    let model_builder = BVGraphModelBuilder::<FIDELITY, RADIX, EntropyMockWriter<FIDELITY, RADIX>>::new(
        entropic_costs_table.clone(),
    );

    let mut bvcomp = BVComp::<BVGraphModelBuilder<
        FIDELITY,
        RADIX,
        EntropyMockWriter<FIDELITY, RADIX>
    >>::new(model_builder, 7, 2, 3, 0);

    // second iteration -> build the model with entropic mock writer
    for node_index in 0..graph.num_nodes() {
        let successors = graph
            .successors(node_index)
            .map(|x| x.0)
            .collect::<Vec<_>>();
        bvcomp.push(successors)?;
    }

    let model4encoder = bvcomp.flush()?.build();

    let mut bvcomp = BVComp::<BVGraphWriter<FIDELITY, RADIX>>::new(
        BVGraphWriter::new(model4encoder, entropic_costs_table),
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

    let code_reader_builder = ANSBVGraphReaderBuilder::<FIDELITY, RADIX>::new(prelude, phases);

    let decoded_graph = BVGraph::<ANSBVGraphReaderBuilder<FIDELITY, RADIX>, EmptyDict<usize, usize>>::new(
        code_reader_builder,
        2,
        7,
        6,
        4,
    );

    assert_eq!(decoded_graph.successors(0).collect::<Vec<_>>(), vec![2, 3]);
    assert_eq!(decoded_graph.successors(1).collect::<Vec<_>>(), vec![5]);
    assert_eq!(decoded_graph.successors(2).collect::<Vec<_>>(), vec![]);
    Ok(())
}


#[test]
fn decoder_decodes_correctly_cnr_graph() -> Result<()> {
    let graph = webgraph::graph::bvgraph::load("tests/data/cnr-2000/cnr-2000")?;
    let num_nodes = graph.num_nodes();
    let num_arcs = graph.num_arcs_hint().unwrap();

    let binary_costs_table = ANSymbolTable::<FIDELITY, RADIX>::initialize_with_binary_cost(9);
    let model_builder = BVGraphModelBuilder::<FIDELITY, RADIX, Log2MockWriter<FIDELITY, RADIX>>::new(binary_costs_table);
    let mut bvcomp = BVComp::<BVGraphModelBuilder<
        FIDELITY,
        RADIX,
        Log2MockWriter<FIDELITY, RADIX>
    >>::new(model_builder, 7, 2, 3, 0);

    // First iteration with Log2MockWriter
    bvcomp.extend(graph.iter())?; // I SIMBOLI VANNO UNFOLDATI PRIMA DI CHIEDERE COSTO!!!!!

    let mut model4encoder =  bvcomp.flush()?.build();
    let symbol_freqs = model4encoder.get_symbol_freqs();

    let entropic_costs_table = ANSymbolTable::<FIDELITY, RADIX>::new(
        symbol_freqs,
        model4encoder.frame_sizes.clone()
    );

    let model_builder = BVGraphModelBuilder::<FIDELITY, RADIX, EntropyMockWriter<FIDELITY, RADIX>>::new(
        entropic_costs_table.clone(),
    );

    let mut bvcomp = BVComp::<BVGraphModelBuilder<
        FIDELITY,
        RADIX,
        EntropyMockWriter<FIDELITY, RADIX>
    >>::new(model_builder, 7, 2, 3, 0);

    // second iteration with EntropyMockWriter
    bvcomp.extend(graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();

    let mut bvcomp = BVComp::<BVGraphWriter<FIDELITY, RADIX>>::new(
        BVGraphWriter::new(model4encoder, entropic_costs_table),
        7,
        2,
        3,
        0
    );

    // Encoding the graph
    bvcomp.extend(graph.iter())?;

    let (mut encoder, phases) = bvcomp.flush()?.into_inner();
    let prelude = encoder.serialize();

    let code_reader_builder = ANSBVGraphReaderBuilder::<FIDELITY, RADIX>::new(prelude, phases);

    let decoded_graph = BVGraph::<ANSBVGraphReaderBuilder<FIDELITY, RADIX>, EmptyDict<usize, usize>>::new(
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
