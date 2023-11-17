use criterion::{black_box, Criterion, criterion_group};

use pprof::criterion::{Output, PProfProfiler};

use folded_streaming_rans::ans::dec_model::VecFrame;
use folded_streaming_rans::ans::decoder::FoldedStreamANSDecoder;
use folded_streaming_rans::ans::encoder::FoldedStreamANSCoder;

use crate::benchmarks::get_symbols;
use crate::benchmarks::{RADIX, FIDELITY};

fn decode_benchmark(c: &mut Criterion) {
    let symbols = get_symbols();
    let mut coder = FoldedStreamANSCoder::<RADIX, FIDELITY>::new(symbols.clone());

    coder.encode_all();

    let data = coder.serialize();

    let mut group = c.benchmark_group("decoder benchmark");
    group.measurement_time(std::time::Duration::from_secs(10));
    group.throughput(criterion::Throughput::Elements(symbols.len() as u64));
    group.sample_size(10);
    group.bench_function("decoding", |b| {
        b.iter_batched(
            || {
                let decoder = FoldedStreamANSDecoder::<RADIX, FIDELITY, VecFrame>::new(
                    &data.1,
                    data.3,
                    data.2,
                    data.4.clone(),
                    data.5.clone(),
                    data.0,
                );
                black_box(decoder)
            },
            |mut decoder| decoder.decode_all(),
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group! {
    name = decoder_benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = decode_benchmark
    }
