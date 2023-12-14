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
// Both decoders use the same radix and fidelity values and a vec of bytes to handle folded bits.
// These benches aim to bench the the different implementations of the symbol primitive within the decoder
// and not, as done in model_for_decoder, in isolation.

fn decode_with_rank_frame(c: &mut Criterion) {
    let symbols = get_symbols();
    let mut group = c.benchmark_group("probing within decoder");
    group.measurement_time(std::time::Duration::from_secs(25));
    group.sample_size(300);

    let mut coder = FoldedStreamANSCoder::<FIDELITY>::new(&symbols);
    coder.encode_all();
    let prelude = coder.serialize();

    // by default it uses a the rank9sel as frame and the fastest radix (radix = 8)
    let decoder = FoldedStreamANSDecoder::<FIDELITY>::new(prelude);

    group.bench_function("with rank9sel as frame", |b| {
        b.iter(|| decoder.decode_all());
    });
}

fn decode_with_table_as_frame(c: &mut Criterion) {
    let symbols = get_symbols();
    let mut group = c.benchmark_group("probing within decoder");
    group.measurement_time(std::time::Duration::from_secs(25));
    group.sample_size(300);

    let mut coder = FoldedStreamANSCoder::<FIDELITY>::new(&symbols);
    coder.encode_all();

    let prelude = coder.serialize();
    let table = VecFrame::new(
        &prelude.table,
        prelude.log2_frame_size,
        FOLDING_OFFSET,
        FOLDING_THRESHOLD,
        FASTER_RADIX,
    );
    let decoder =
        FoldedStreamANSDecoder::<FIDELITY, FASTER_RADIX, VecFrame<FASTER_RADIX>>::with_parameters(
            prelude, table,
        );

    group.bench_function("with a table as a frame", |b| {
        b.iter(|| decoder.decode_all());
    });
}

//                BENCH THE DIFFERENT STRUCTURES USED TO HANDLE THE FOLDED BITS
// ---------------------- ---------------------- ---------------------- ---------------------- --- //
// Two strategies are benched:
// 1. by using a vector of bytes;
// 2. by using a BitVec.
// both decoders use the fastest radix (radix = 8) and a Rank9Sel as frame.

// try to decode by using a Vec<u8> to handle the folded bits.
fn decode_with_byte_vector(c: &mut Criterion) {
    let symbols = get_symbols();
    let mut group = c.benchmark_group("decoder benchmark");
    group.measurement_time(std::time::Duration::from_secs(25));
    group.throughput(criterion::Throughput::Elements(symbols.len() as u64));
    group.sample_size(300);

    let mut coder = FoldedStreamANSCoder::<FIDELITY>::new(&symbols);
    coder.encode_all();

    let prelude = coder.serialize();

    let frame = VecFrame::new(
        &prelude.table,
        prelude.log2_frame_size,
        FOLDING_OFFSET,
        FOLDING_THRESHOLD,
        FASTER_RADIX,
    );

    let decoder = FoldedStreamANSDecoder::<
        FIDELITY,
        FASTER_RADIX,
        VecFrame<8>,
        Vec<u8>
    >::with_parameters(prelude, frame);

    group.bench_function("with vec of bytes", |b| {
        b.iter(|| decoder.decode_all());
    });
}

// try to decode by using a BitVec instead of a Vec<u8> to handle the folded bits.
fn decode_with_bitvec(c: &mut Criterion) {
    let symbols = get_symbols();
    let mut group = c.benchmark_group("decoder benchmark");
    group.measurement_time(std::time::Duration::from_secs(25));
    group.throughput(criterion::Throughput::Elements(symbols.len() as u64));
    group.sample_size(300);

    let mut coder = FoldedStreamANSCoder::<
        FIDELITY,
        FASTER_RADIX,
        BitVec<usize, Msb0>, // we need to use a bitvec even in the encoder
    >::with_parameters(&symbols, BitVec::<usize, Msb0>::new());

    coder.encode_all();

    let prelude = coder.serialize();

    let model = Rank9SelFrame::new(
        &prelude.table,
        prelude.log2_frame_size,
        FOLDING_OFFSET,
        FOLDING_THRESHOLD,
        FASTER_RADIX,
    );

    let decoder = FoldedStreamANSDecoder::<
        FIDELITY,
        FASTER_RADIX,
        Rank9SelFrame<FASTER_RADIX>,
        BitVec<usize, Msb0>,
    >::with_parameters(prelude, model);

    group.bench_function("with BitVec", |b| {
        b.iter(|| decoder.decode_all());
    });
}

criterion_group! {
    name = decoder_benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(150, Output::Flamegraph(None)));
    targets = decode_with_rank_frame, decode_with_table_as_frame, decode_with_byte_vector, decode_with_bitvec
}
