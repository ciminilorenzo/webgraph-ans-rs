use criterion::{Criterion, criterion_group};

use pprof::criterion::{Output, PProfProfiler};

use folded_streaming_rans::ans::folded_stream_ans_encoder::FoldedStreamANSCoder;

use crate::benchmarks::get_symbols;

use crate::benchmarks::{RADIX, FIDELITY};

fn encoding_benchmark(c: &mut Criterion) {
    let symbols = get_symbols();

    let mut group = c.benchmark_group("coder benchmark");
    group.measurement_time(std::time::Duration::from_secs(20));
    group.bench_function("encoding", |b| {
        let mut encoder = FoldedStreamANSCoder::<RADIX, FIDELITY>::new(symbols.clone());
        b.iter(|| encoder.encode_all())
    });
}

criterion_group! {
    name = encoder_benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = encoding_benchmark
    }