use criterion::{criterion_group, Criterion};

use pprof::criterion::{Output, PProfProfiler};

use bitvec::prelude::*;

use folded_streaming_rans::ans::dec_model::{Rank9SelFrame, VecFrame};
use folded_streaming_rans::ans::decoder::FoldedStreamANSDecoder;
use folded_streaming_rans::ans::encoder::FoldedStreamANSCoder;
use folded_streaming_rans::ans::FASTER_RADIX;

use crate::benchmarks::get_symbols;
use crate::benchmarks::FIDELITY;

const FOLDING_OFFSET: u64 = (1 << (FIDELITY - 1)) * ((1 << FASTER_RADIX) - 1);

const FOLDING_THRESHOLD: u64 = 1 << (FIDELITY + FASTER_RADIX - 1);


//                BENCH THE DIFFERENT WAYS OF IMPLEMENTING THE SYMBOL PRIMITIVE
// ---------------------- ---------------------- ---------------------- ---------------------- --- //
// Two strategies are benched:
// 1. by using a table as frame;
// 2. by using a rank9sel frame.
// Both decoders use the same radix (the fastest one: 8)  and fidelity values and a vec of bytes to handle folded bits.
// These benches aim to bench the the different implementations of the symbol primitive within the decoder and not, as
// done in model_for_decoder, in isolation.

fn decode_with_table_as_frame(c: &mut Criterion) {
    let symbols = get_symbols();
    let mut coder = FoldedStreamANSCoder::<FIDELITY>::new(&symbols);
    coder.encode_all();
    let prelude = coder.serialize();

    // by default it uses a table as frame and the fastest radix (radix = 8)
    let decoder = FoldedStreamANSDecoder::<FIDELITY>::new(prelude);

    let mut group = c.benchmark_group("Decoding with different frame");
    group.measurement_time(std::time::Duration::from_secs(25));
    group.sample_size(300);
    group.bench_function("with table as frame", |b| {
        b.iter(|| decoder.decode_all());
    });
}

fn decode_with_rank_as_frame(c: &mut Criterion) {
    let symbols = get_symbols();
    let mut group = c.benchmark_group("Decoding with different frame");
    group.measurement_time(std::time::Duration::from_secs(25));
    group.sample_size(300);

    let mut coder = FoldedStreamANSCoder::<FIDELITY>::new(&symbols);
    coder.encode_all();

    let prelude = coder.serialize();
    let table = Rank9SelFrame::<FASTER_RADIX, u64>::new(
        &prelude.table,
        prelude.log2_frame_size,
        FOLDING_OFFSET,
        FOLDING_THRESHOLD,
    );
    let decoder = FoldedStreamANSDecoder::<
        FIDELITY,
        FASTER_RADIX,
        u64,
        Rank9SelFrame<FASTER_RADIX, u64>
    >::with_parameters(prelude, table);

    group.bench_function("with rank9sel as frame", |b| {
        b.iter(|| decoder.decode_all());
    });
}

//                BENCH THE DIFFERENT STRUCTURES USED TO HANDLE THE FOLDED BITS
// ---------------------- ---------------------- ---------------------- ---------------------- --- //
// Two strategies are benched:
// 1. by using a vector of bytes;
// 2. by using a BitVec.
// both decoders use the fastest radix (radix = 8).

fn decode_with_byte_vector(c: &mut Criterion) {
    let symbols = get_symbols();
    let mut coder = FoldedStreamANSCoder::<FIDELITY>::new(&symbols);
    coder.encode_all();
    let prelude = coder.serialize();

    // this is the standard one that uses a vec of bytes to handle the folded bytes.
    let decoder = FoldedStreamANSDecoder::<FIDELITY>::new(prelude);

    let mut group = c.benchmark_group("Decoding with different DS for folded bits");
    group.measurement_time(std::time::Duration::from_secs(25));
    group.throughput(criterion::Throughput::Elements(symbols.len() as u64));
    group.sample_size(300);
    group.bench_function("with vec of bytes", |b| {
        b.iter(|| decoder.decode_all());
    });
}

// try to decode by using a BitVec instead of a Vec<u8> to handle the folded bits.
fn decode_with_bitvec(c: &mut Criterion) {
    let symbols = get_symbols();
    let mut coder = FoldedStreamANSCoder::<
        FIDELITY,
        FASTER_RADIX,
        BitVec<usize, Msb0>, // we need to use a bitvec even in the encoder
    >::with_parameters(&symbols, BitVec::<usize, Msb0>::new());

    coder.encode_all();
    let prelude = coder.serialize();

    let frame = VecFrame::<FASTER_RADIX, u64>::new(
        &prelude.table,
        prelude.log2_frame_size,
        FOLDING_OFFSET,
        FOLDING_THRESHOLD,
    );

    let decoder = FoldedStreamANSDecoder::<
        FIDELITY,
        FASTER_RADIX,
        u64,
        VecFrame<FASTER_RADIX, u64>,
        BitVec<usize, Msb0>, // <--- this time we use a bitvec
    >::with_parameters(prelude, frame);

    let mut group = c.benchmark_group("Decoding with different DS for folded bits");
    group.measurement_time(std::time::Duration::from_secs(25));
    group.throughput(criterion::Throughput::Elements(symbols.len() as u64));
    group.sample_size(300);
    group.bench_function("with BitVec", |b| {
        b.iter(|| decoder.decode_all());
    });
}

//                BENCH THE DIFFERENT WIDTH FOR THE `QUASI_FOLDED` FIELD
// ---------------------- ---------------------- ---------------------- ---------------------- --- //
// Two decoders are benched: the standard one of 64 bits and the one of 32 bits.

fn decode_with_64bit_quasi_folded(c: &mut Criterion) {
    let symbols = get_symbols();
    let mut group = c.benchmark_group("Decoding with different width for quasi folded");
    group.measurement_time(std::time::Duration::from_secs(25));
    group.throughput(criterion::Throughput::Elements(symbols.len() as u64));
    group.sample_size(300);

    let mut coder = FoldedStreamANSCoder::<FIDELITY>::new(&symbols);
    coder.encode_all();
    let prelude = coder.serialize();

    let decoder = FoldedStreamANSDecoder::<FIDELITY>::new(prelude); // the standard one uses 64 bits for the quasi folded field

    group.bench_function("with 64bit", |b| {
        b.iter(|| decoder.decode_all());
    });
}

fn decode_with_32bit_quasi_folded(c: &mut Criterion) {
    let symbols = get_symbols();
    let mut group = c.benchmark_group("Decoding with different width for quasi folded");
    group.measurement_time(std::time::Duration::from_secs(25));
    group.throughput(criterion::Throughput::Elements(symbols.len() as u64));
    group.sample_size(300);

    let mut coder = FoldedStreamANSCoder::<FIDELITY>::new(&symbols);
    coder.encode_all();
    let prelude = coder.serialize();

    let frame = VecFrame::<FASTER_RADIX, u32>::new(
        &prelude.table,
        prelude.log2_frame_size,
        FOLDING_OFFSET,
        FOLDING_THRESHOLD,
    );

    let decoder = FoldedStreamANSDecoder::<
        FIDELITY,
        FASTER_RADIX,
        u32,
        VecFrame<FASTER_RADIX, u32>,
        Vec<u8>
    >::with_parameters(prelude, frame);

    group.bench_function("with 32bit", |b| {
        b.iter(|| decoder.decode_all());
    });
}

criterion_group! {
    name = decoder_benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(150, Output::Flamegraph(None)));
    targets = decode_with_byte_vector,
        decode_with_bitvec,
        decode_with_table_as_frame,
        decode_with_rank_as_frame,
        decode_with_64bit_quasi_folded,
        decode_with_32bit_quasi_folded
}
