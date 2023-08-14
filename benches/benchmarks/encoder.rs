use criterion::{Criterion, criterion_group};
use pprof::criterion::{Output, PProfProfiler};
use rand::prelude::{Distribution, StdRng};
use rand::SeedableRng;
use rand_distr::Zipf;
use folded_streaming_rans::ans::folded_stream_ans_encoder::FoldedStreamANSCoder;
use crate::benchmarks::{MAXIMUM_SYMBOL, SYMBOL_LIST_LENGTH};

#[ignore]
fn encoding_benchmark(c: &mut Criterion) {
    /*
    let mut rng = StdRng::seed_from_u64(0);
    let distribution = Zipf::new(MAXIMUM_SYMBOL, 1.0).unwrap();
    let mut symbols = Vec::with_capacity(SYMBOL_LIST_LENGTH);

    for _ in 0..SYMBOL_LIST_LENGTH {
        symbols.push(distribution.sample(&mut rng) as usize);
    }

    let mut encoder = FoldedStreamANSCoder::<4,2>::new(symbols).unwrap();
    let mut group = c.benchmark_group("coder benchmark");

    // this bench takes a lot of time. Thus, reduce the sample size.
    group.sample_size(10);
    group.bench_function("encoding", |b| b.iter(|| encoder.encode_all()));
    group.finish();
     */
}

criterion_group! {
    name = encoder_benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = encoding_benchmark
    }