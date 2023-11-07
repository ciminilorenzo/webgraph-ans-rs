use criterion::{Criterion, criterion_group, SamplingMode};

use pprof::criterion::{Output, PProfProfiler};

use folded_streaming_rans::ans::decoder_model::Rank9SelFrame;
use folded_streaming_rans::ans::folded_stream_ans_decoder::FoldedStreamANSDecoder;
use folded_streaming_rans::ans::folded_stream_ans_encoder::FoldedStreamANSCoder;

use crate::benchmarks::get_symbols;
use crate::benchmarks::{RADIX, FIDELITY};

fn decode_benchmark(c: &mut Criterion) {
    let symbols = get_symbols();
    let mut coder = FoldedStreamANSCoder::<RADIX, FIDELITY>::new(symbols.clone());

    coder.encode_all();

    let data = coder.serialize();
    let frame = Rank9SelFrame::new(&data.0, data.2);

    let mut group = c.benchmark_group("decoder benchmark");
    group.sampling_mode(SamplingMode::Flat);
    group.bench_function("decoding", |b| {
        b.iter_batched(
            || {
                let decoder = FoldedStreamANSDecoder::<RADIX, FIDELITY, Rank9SelFrame>::new(
                    data.1,
                    frame.clone(),
                    data.2,
                    data.3.clone(),
                    data.4.clone(),
                );
                decoder
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
