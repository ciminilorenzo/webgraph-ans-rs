use criterion::{criterion_group, BatchSize, Criterion};
use dsi_bitstream::prelude::BE;
use folded_streaming_rans::bvgraph::estimators::entropy_estimator::EntropyEstimator;
use folded_streaming_rans::bvgraph::writers::bvgraph_model_builder::BVGraphModelBuilder;
use pprof::criterion::{Output, PProfProfiler};
use webgraph::graphs::{BVComp, BVGraph};
use webgraph::prelude::SequentialLabeling;
use folded_streaming_rans::bvgraph::estimators::log2_estimator::Log2Estimator;

fn model4encoder_building_bench(c: &mut Criterion) {
    let graph = BVGraph::with_basename("tests/data/cnr-2000/cnr-2000")
        .endianness::<BE>()
        .load()
        .unwrap();

    let log2_mock = Log2Estimator::default();
    let mut model_builder = BVGraphModelBuilder::<Log2Estimator>::new(log2_mock);
    let mut bvcomp = BVComp::new(&mut model_builder, 7, 3, 2, 0);

    // First iteration with Log2MockWriter
    bvcomp.extend(graph.iter()).unwrap();
    bvcomp.flush().unwrap();
    let model4encoder = model_builder.build();
    let folding_params = model4encoder.get_folding_params();
    let entropic_mock = EntropyEstimator::new(&model4encoder, folding_params);

    let mut group = c.benchmark_group("model building");
    group.measurement_time(std::time::Duration::from_secs(100));
    group.sample_size(20);

    group.bench_function("cnr-2000", |b| {
        b.iter_batched(
            || {
                let model_builder =
                    BVGraphModelBuilder::<EntropyEstimator>::new(entropic_mock.clone());
                BVComp::<BVGraphModelBuilder<EntropyEstimator>>::new(model_builder, 7, 3, 2, 0)
            },
            |mut bvcomp|
                // second iteration with EntropyMockWriter
                bvcomp.extend(graph.iter()).unwrap(),
            BatchSize::SmallInput,
        )
    });
    group.finish()
}

criterion_group! {
    name = model4encoder_building_benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = model4encoder_building_bench,
}
