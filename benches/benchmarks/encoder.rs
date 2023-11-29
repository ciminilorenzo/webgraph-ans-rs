use bitvec::order::Msb0;
use bitvec::prelude::BitVec;
use criterion::{BatchSize, Criterion, criterion_group};

use pprof::criterion::{Output, PProfProfiler};

use folded_streaming_rans::ans::encoder::FoldedStreamANSCoder;

use crate::benchmarks::get_symbols;
use crate::benchmarks::{RADIX, FIDELITY};


fn fast_encoding_bench(c: &mut Criterion) {
    let symbols = get_symbols();
    let mut group = c.benchmark_group("encoder");
    group.measurement_time(std::time::Duration::from_secs(30));
    group.throughput(criterion::Throughput::Elements(symbols.len() as u64));
    group.sample_size(50);

    let coder = FoldedStreamANSCoder::<FIDELITY>::new(&symbols);

    group.bench_function("faster encoder", |b| {
        b.iter_batched(|| coder.clone(), |mut coder| coder.encode_all(), BatchSize::SmallInput)
    });
    group.finish()
}


fn encoding_bench(c: &mut Criterion) {
    let symbols = get_symbols();
    let mut group = c.benchmark_group("encoder");
    group.measurement_time(std::time::Duration::from_secs(30));
    group.throughput(criterion::Throughput::Elements(symbols.len() as u64));
    group.sample_size(50);

    let coder = FoldedStreamANSCoder::<
        FIDELITY,
        RADIX,
        BitVec<usize, Msb0>
    >::with_parameters(&symbols, BitVec::<usize, Msb0>::new());

    group.bench_function("encoder", |b| {
        b.iter_batched(|| coder.clone(), |mut coder| coder.encode_all(), BatchSize::SmallInput)
    });
    group.finish()
}

criterion_group! {
    name = encoder_benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = fast_encoding_bench, encoding_bench
}