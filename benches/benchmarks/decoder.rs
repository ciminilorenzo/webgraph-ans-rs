use bitvec::order::Msb0;
use criterion::{Criterion, criterion_group};

use pprof::criterion::{Output, PProfProfiler};
use bitvec::prelude::BitVec;
use folded_streaming_rans::ans::dec_model::{Rank9SelFrame, VecFrame};

use folded_streaming_rans::ans::decoder::FoldedStreamANSDecoder;
use folded_streaming_rans::ans::encoder::FoldedStreamANSCoder;
use folded_streaming_rans::RawSymbol;

use crate::benchmarks::get_symbols;
use crate::benchmarks::{FIDELITY};


fn decode_with_fastest_decoder(c: &mut Criterion) {
    let symbols = get_symbols();
    let mut coder = FoldedStreamANSCoder::<FIDELITY>::new(&symbols);
    coder.encode_all();

    let prelude = coder.serialize();

    let mut group = c.benchmark_group("decoder benchmark");
    group.measurement_time(std::time::Duration::from_secs(40));
    group.throughput(criterion::Throughput::Elements(symbols.len() as u64));
    group.sample_size(500);

    let decoder = FoldedStreamANSDecoder::<FIDELITY>::new(prelude);

    group.bench_function("faster decoding", |b| {
        b.iter(|| decoder.decode_all());
    });
}


fn decode(c: &mut Criterion) {
    let symbols = get_symbols();
    let mut coder = FoldedStreamANSCoder::<
        FIDELITY,
        4,
        BitVec<usize, Msb0>
    >::with_parameters(&symbols, BitVec::<usize, Msb0>::new());

    coder.encode_all();
    let prelude = coder.serialize();

    let mut group = c.benchmark_group("decoder benchmark");
    group.measurement_time(std::time::Duration::from_secs(40));
    group.throughput(criterion::Throughput::Elements(symbols.len() as u64));
    group.sample_size(500);

    let folding_offset = ((1 << (FIDELITY - 1)) * ((1 << 4) - 1)) as RawSymbol;
    let folding_threshold = (1 << (FIDELITY + 4 - 1)) as RawSymbol;

    let model = VecFrame::new(&prelude.table, prelude.log2_frame_size, folding_offset, folding_threshold, 4);

    let decoder = FoldedStreamANSDecoder::<
        FIDELITY,
        4,
        VecFrame,
        BitVec<usize, Msb0>
    >::with_parameters(prelude, model);

    group.bench_function("decoding", |b| {
        b.iter(|| decoder.decode_all());
    });
}

criterion_group! {
    name = decoder_benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(150, Output::Flamegraph(None)));
    targets = decode_with_fastest_decoder, decode
}