use criterion::{BatchSize, Criterion, criterion_group};

use pprof::criterion::{Output, PProfProfiler};

use folded_streaming_rans::ans::encoder::FoldedStreamANSCoder;

use crate::benchmarks::get_symbols;
use crate::benchmarks::{RADIX, FIDELITY};

fn encoding_benchmark(c: &mut Criterion) {
    let symbols = get_symbols();
    let mut group = c.benchmark_group("coder benchmark");

    group.measurement_time(std::time::Duration::from_secs(30));
    group.throughput(criterion::Throughput::Elements(symbols.len() as u64));
    group.sample_size(100);

    let encoder = FoldedStreamANSCoder::<RADIX, FIDELITY>::new(symbols);

    group.bench_function("encoding", |b| {
        b.iter_batched(|| encoder.clone(), |mut coder| coder.encode_all(), BatchSize::SmallInput)
    });
    group.finish()
}

criterion_group! {
    name = encoder_benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = encoding_benchmark
}