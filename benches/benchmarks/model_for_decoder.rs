use criterion::{black_box, criterion_group, Criterion};

use pprof::criterion::{Output, PProfProfiler};

use rand::prelude::{SliceRandom, SmallRng};
use rand::SeedableRng;

use folded_streaming_rans::ans::dec_model::{EliasFanoFrame, Rank9SelFrame, VecFrame};
use folded_streaming_rans::ans::enc_model::FoldedANSModel4Encoder;
use folded_streaming_rans::{RawSymbol, State};

use crate::benchmarks::{get_symbols, FIDELITY, RADIX};

/// Creates a random permutation of the slots composing the frame.
fn get_slots_to_probe(log2_frame_size: u8) -> Vec<usize> {
    let mut slots = (0..(1 << log2_frame_size) - 1)
        .into_iter()
        .collect::<Vec<usize>>();

    slots.shuffle(&mut SmallRng::seed_from_u64(0));
    slots
}

fn probing_benchmark(c: &mut Criterion) {
    let symbols = get_symbols();
    let encoder_model = FoldedANSModel4Encoder::new(&symbols, RADIX, FIDELITY);
    let table = encoder_model.to_raw_parts();
    let log_m = encoder_model.log2_frame_size;
    let slots_to_probe = get_slots_to_probe(log_m);

    let folding_offset = ((1 << (FIDELITY - 1)) * ((1 << RADIX) - 1)) as RawSymbol;
    let folding_threshold = (1 << (FIDELITY + RADIX - 1)) as RawSymbol;

    let vec_frame = VecFrame::new(&table, log_m, folding_offset, folding_threshold, RADIX);
    let elias_frame = EliasFanoFrame::new(&table, log_m, folding_offset, folding_threshold, RADIX);
    let bitvec_frame = Rank9SelFrame::new(&table, log_m, folding_offset, folding_threshold, RADIX);

    let mut group = c.benchmark_group("Probing");
    group.measurement_time(std::time::Duration::from_secs(15));

    group.bench_with_input("with elias", &slots_to_probe, |b, slots_to_probe| b.iter(|| {
        for &s in slots_to_probe {
            black_box(&elias_frame[s as State]);
        }})
    );

    group.bench_with_input("with vec", &slots_to_probe,|b, slots_to_probe| b.iter(|| {
        for &s in slots_to_probe {
            black_box(&vec_frame[s as State]);
        } })
    );

    group.bench_with_input("with rank9", &slots_to_probe,|b, slots_to_probe| b.iter(|| {
        for &s in slots_to_probe {
            black_box(&bitvec_frame[s as State]);
        }})
    );
    group.finish();
}

criterion_group! {
    name = model_for_decoder_benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = probing_benchmark
}
