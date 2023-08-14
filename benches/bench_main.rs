use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::model_for_decoder::decoder_benches,
    benchmarks::encoder::encoder_benches
}