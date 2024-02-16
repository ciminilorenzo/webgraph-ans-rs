/*
   This bench prints the medium time (in terms of nanoseconds) needed to decode each arc of the cnr-2000 graph.
*/

use anyhow::Result;
use dsi_bitstream::prelude::BE;
use epserde::prelude::{Deserialize, Serialize};
use folded_streaming_rans::bvgraph::mock_writers::{EntropyEstimator, Log2Estimator};
use folded_streaming_rans::bvgraph::reader::ANSBVGraphDecoderFactory;
use webgraph::graphs::BVGraphSeq;
use webgraph::prelude::{BVComp, BVGraph, RandomAccessGraph, SequentialLabeling};

use folded_streaming_rans::bvgraph::writer::{BVGraphMeasurableEncoder, BVGraphModelBuilder};
use folded_streaming_rans::multi_model_ans::{ANSCompressorPhase, Prelude};

const NODES: usize = 325557;
const ARCS: u64 = 3216152;

fn main() -> Result<()> {
    let seq_graph = BVGraphSeq::with_basename("tests/data/cnr-2000/cnr-2000")
        .endianness::<BE>()
        .load()?;

    let log2_mock = Log2Estimator::default();
    let model_builder = BVGraphModelBuilder::<Log2Estimator>::new(log2_mock);
    let mut bvcomp = BVComp::<BVGraphModelBuilder<Log2Estimator>>::new(model_builder, 7, 2, 3, 0);

    bvcomp.extend(seq_graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();
    let folding_params = model4encoder.get_folding_params();
    let entropic_mock = EntropyEstimator::new(&model4encoder, folding_params);
    let model_builder = BVGraphModelBuilder::<EntropyEstimator>::new(entropic_mock.clone());
    let mut bvcomp =
        BVComp::<BVGraphModelBuilder<EntropyEstimator>>::new(model_builder, 7, 2, 3, 0);

    bvcomp.extend(seq_graph.iter())?;

    let model4encoder = bvcomp.flush()?.build();

    let mut bvcomp = BVComp::<BVGraphMeasurableEncoder>::new(
        BVGraphMeasurableEncoder::new(model4encoder, entropic_mock),
        7,
        2,
        3,
        0,
    );

    // third iteration: encode with the entropy mock
    bvcomp.extend(seq_graph.iter())?;

    // get phases and the encoder from the bvcomp
    let (encoder, phases) = bvcomp.flush()?.into_inner();
    let prelude = encoder.into_prelude();

    phases.store("cnr-2000-phases")?;
    prelude.store("cnr-2000-prelude")?;

    let prelude = Prelude::load_full("cnr-2000-prelude")?;
    let phases = Vec::<ANSCompressorPhase>::load_full("cnr-2000-phases")?;
    let code_reader_builder = ANSBVGraphDecoderFactory::new(&prelude, phases);
    let decoded_graph =
        BVGraph::<ANSBVGraphDecoderFactory>::new(code_reader_builder, 2, 7, NODES, ARCS);

    let now = std::time::Instant::now();
    let mut arcs = 0;
    for node_index in 0..NODES {
        let decoded_successors = decoded_graph.successors(node_index).collect::<Vec<_>>();
        arcs += decoded_successors.len();
    }

    dbg!(now.elapsed());
    dbg!(now.elapsed().as_nanos() / arcs as u128);
    Ok(())
}
