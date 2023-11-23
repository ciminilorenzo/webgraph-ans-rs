use criterion::{Criterion, criterion_group};

use pprof::criterion::{Output, PProfProfiler};

use folded_streaming_rans::ans::dec_model::{Rank9SelFrame};
use folded_streaming_rans::ans::decoder::FoldedStreamANSDecoder;
use folded_streaming_rans::ans::encoder::FoldedStreamANSCoder;
use folded_streaming_rans::RawSymbol;

use crate::benchmarks::get_symbols;
use crate::benchmarks::{RADIX, FIDELITY};

// This bench needs iter_batched since each iteration of the bench modifies some inner values
// within the decoder (such as its multiple states).
fn decode_benchmark(c: &mut Criterion) {
    let symbols = get_symbols();
    let mut coder = FoldedStreamANSCoder::<RADIX, FIDELITY>::new(symbols.clone());

    coder.encode_all();
    let prelude = coder.serialize();

    let mut group = c.benchmark_group("decoder benchmark");
    group.measurement_time(std::time::Duration::from_secs(40));
    group.throughput(criterion::Throughput::Elements(symbols.len() as u64));
    group.sample_size(500);

    let folding_offset = ((1 << (FIDELITY - 1)) * ((1 << RADIX) - 1)) as RawSymbol;
    let folding_threshold = (1 << (FIDELITY + RADIX - 1)) as RawSymbol;

    let frame = Rank9SelFrame::new(&prelude.1, prelude.3, folding_offset, folding_threshold, RADIX);

    let decoder = FoldedStreamANSDecoder::<RADIX, FIDELITY, Rank9SelFrame>::with_frame(
        prelude.0,
        prelude.2,
        frame,
        prelude.3,
        prelude.4.clone(),
        prelude.5.clone(),
    );

    group.bench_function("decoding", |b| {
        b.iter_batched(|| decoder.clone(), |mut decoder| decoder.decode_all(), criterion::BatchSize::SmallInput)
    });
}

criterion_group! {
    name = decoder_benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(150, Output::Flamegraph(None)));
    targets = decode_benchmark
    }


/*

with this is 33% slower

let decoder = FoldedStreamANSDecoder::<RADIX, FIDELITY, VecFrame>::new(
        &prelude.1,
        prelude.3,
        prelude.2,
        prelude.4.clone(),
        prelude.5.clone(),
        prelude.0,
    );
 */