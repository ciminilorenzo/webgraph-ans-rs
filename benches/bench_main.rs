use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::graph_encoding::encoder_benches,
    benchmarks::model4encoder_building::model4encoder_building_benches,
    benchmarks::fastdiv::div_benches,
}
