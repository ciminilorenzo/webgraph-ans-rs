use folded_streaming_rans::bvgraph::writer::{BVGraphModelBuilder, BVGraphWriter};
use mem_dbg::*;
use webgraph::{
    graph::bvgraph::{BVComp, BVGraphCodesWriter},
    traits::SequentialLabelling,
};

#[test]
fn test_model_builder() -> anyhow::Result<()> {
    env_logger::builder().is_test(true).try_init().unwrap();

    let dir = tempfile::tempdir()?;
    // load the graph
    let graph = webgraph::graph::bvgraph::load_seq("tests/data/cnr-2000")?;

    let model_builder = BVGraphModelBuilder::<2, 8>::new(dir.path().join("model"));
    let mut bvcomp = BVComp::<BVGraphModelBuilder<2, 8>>::new(model_builder, 7, 2, 3, 0);
    bvcomp.extend(graph.iter())?;
    let encoder = bvcomp.flush()?.build();

    let mut bvcomp =
        BVComp::<BVGraphWriter<2, 8, Vec<u8>>>::new(BVGraphWriter::new(encoder), 7, 2, 3, 0);
    bvcomp.extend(graph.iter())?;
    let mut encoder = bvcomp.flush()?.into_inner();
    let prelude = encoder.serialize();

    dbg!(prelude.normalized_bits.mem_size(SizeFlags::default()) * 8);
    dbg!(prelude.folded_bits.mem_size(SizeFlags::default()) * 8);
    dbg!(prelude.frame_sizes.mem_size(SizeFlags::default()) * 8);
    dbg!(prelude.state.mem_size(SizeFlags::default()) * 8);

    Ok(())
}
