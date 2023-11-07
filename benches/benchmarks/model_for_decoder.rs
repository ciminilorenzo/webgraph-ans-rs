use criterion::{BenchmarkId, black_box, Criterion, criterion_group};

use pprof::criterion::{Output, PProfProfiler};

use rand::prelude::{SliceRandom, StdRng};
use rand::{SeedableRng};

use folded_streaming_rans::ans::decoder_model::{EliasFanoFrame, Rank9SelFrame, VecFrame};
use folded_streaming_rans::ans::encoder_model::FoldedANSModel4Encoder;
use folded_streaming_rans::{State};

use crate::benchmarks::{FIDELITY, get_symbols, RADIX};


/// Creates a random permutation of the slots composing the frame.
fn get_slots_to_probe(log2_frame_size: u8) -> Vec<usize> {
    let mut slots = (0.. (1 << log2_frame_size) - 1)
        .into_iter()
        .collect::<Vec<usize>>();

    slots.shuffle(&mut StdRng::seed_from_u64(0));

    slots[0..700].to_vec()
}

fn probing_benchmark(c: &mut Criterion) {
    let symbols = get_symbols();
    let encoder_model = FoldedANSModel4Encoder::new(&symbols, RADIX, FIDELITY);
    let table = encoder_model.to_raw_parts();
    let log_m = encoder_model.log2_frame_size;
    let slots_to_probe = get_slots_to_probe(log_m);

    let vec_frame = VecFrame::new(&table, log_m);
    let elias_frame = EliasFanoFrame::new(&table, log_m);
    let bitvec_frame = Rank9SelFrame::new(&table, log_m);

    let mut group = c.benchmark_group("Probing");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_nanos(1500));
    group.warm_up_time(std::time::Duration::from_millis(1));

    for slot_to_probe in slots_to_probe.iter() {
        group.bench_with_input(BenchmarkId::new("with elias", slot_to_probe), &slot_to_probe,
                               |b, i| b.iter(|| &elias_frame[black_box(**i as State)]));

        group.bench_with_input(BenchmarkId::new("with vec", slot_to_probe), &slot_to_probe,
                               |b, i| b.iter(|| &vec_frame[black_box(**i as State)]));

        group.bench_with_input(BenchmarkId::new("with Rank9Sel", slot_to_probe), &slot_to_probe,
                               |b, i| b.iter(|| &bitvec_frame[black_box(**i as State)]));

    }
    group.finish();
}

criterion_group! {
    name = decoder_benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = probing_benchmark
    }