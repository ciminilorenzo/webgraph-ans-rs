use criterion::black_box;
use rand::prelude::{Distribution, SliceRandom, SmallRng};
use rand::SeedableRng;
use rand_distr::Zipf;
use folded_streaming_rans::ans::dec_model::VecFrame;
use folded_streaming_rans::ans::enc_model::FoldedANSModel4Encoder;
use folded_streaming_rans::{RawSymbol, State};

/// Size of the list of symbols used during the examples.
const SYMBOL_LIST_LENGTH: usize = 50_000_000;

/// Maximum value that the zipfian distribution can output.
const MAXIMUM_SYMBOL: u64 = 1_000_000_000;

const RADIX: usize = 8;

const FIDELITY: usize = 1;

fn get_slots_to_probe(log2_frame_size: u8) -> Vec<usize> {
    let mut slots = (0..(1 << log2_frame_size) - 1)
        .into_iter()
        .collect::<Vec<usize>>();

    slots.shuffle(&mut SmallRng::seed_from_u64(0));
    slots
}

fn get_symbols() -> Vec<RawSymbol> {
    let mut rng = SmallRng::seed_from_u64(0);
    let distribution = Zipf::new(MAXIMUM_SYMBOL, 1.0).unwrap();
    let mut symbols = Vec::with_capacity(SYMBOL_LIST_LENGTH);

    for _ in 0..SYMBOL_LIST_LENGTH {
        symbols.push(distribution.sample(&mut rng) as RawSymbol);
    }
    symbols
}


fn main() {
    let symbols = get_symbols();
    let encoder_model = FoldedANSModel4Encoder::new(&symbols, RADIX, FIDELITY);
    let table = encoder_model.to_raw_parts();
    let log_m = encoder_model.log2_frame_size;
    let slots_to_probe = get_slots_to_probe(log_m);

    let folding_offset = ((1 << (FIDELITY - 1)) * ((1 << RADIX) - 1)) as RawSymbol;
    let folding_threshold = (1 << (FIDELITY + RADIX - 1)) as RawSymbol;

    let vec_frame = VecFrame::new(&table, log_m, folding_offset, folding_threshold, RADIX);

    for slot in &slots_to_probe {
        black_box(&vec_frame[*slot as State]);
    }
}