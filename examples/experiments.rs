use itertools::Itertools;

use rand::distributions::Distribution;
use rand::prelude::StdRng;
use rand::SeedableRng;
use rand_distr::Zipf;

use folded_streaming_rans::ans::folded_stream_ans_encoder::FoldedStreamANSCoder;
use folded_streaming_rans::{RawSymbol, State};
use folded_streaming_rans::utils::{self_entropy};


/// Size of the list of symbols used to bench.
const SYMBOL_LIST_LENGTH: usize = 5_000_000;

/// Maximum value that the zpfian distribution can output.
const MAXIMUM_SYMBOL: u64 = 1 << 20;

/// Creates a sequence of size [`SYMBOL_LIST_LENGTH`], containing symbols sampled from a Zipfian
/// distribution that can output values up to [`MAXIMUM_SYMBOL`].
fn generate_zipfian_distribution() -> Vec<RawSymbol> {
    let mut rng = StdRng::seed_from_u64(0);
    let distribution = Zipf::new(MAXIMUM_SYMBOL, 0.7).unwrap();
    let mut symbols = Vec::with_capacity(SYMBOL_LIST_LENGTH);

    for _ in 0..SYMBOL_LIST_LENGTH {
        symbols.push(distribution.sample(&mut rng) as RawSymbol);
    }
    symbols
}

fn main() {
    let symbols = generate_zipfian_distribution();

    let freqs_distr = symbols
        .iter()
        .counts()
        .iter()
        .map(|(_, sym_freq)| *sym_freq)
        .collect::<Vec<usize>>();

    let original_m = SYMBOL_LIST_LENGTH;
    let original_entropy = self_entropy(&freqs_distr, original_m as f64);
    println!("self-entropy per sym: {}\n\n", original_entropy / original_m as f64);

    let mut encoder = FoldedStreamANSCoder::<4,2>::new(symbols.clone());
    println!("Number of symbols: {}", encoder.model.table.len());
    encoder.encode_all();

    let result = encoder.serialize();
    let first_bits = result.3.len();
    let second_bits = result.4.len();
    let total = 64 + first_bits + second_bits;

    println!("BPI: {}", total as f64 / SYMBOL_LIST_LENGTH as f64);


    let mut encoder = FoldedStreamANSCoder::<4,3>::new(symbols.clone());
    println!("Number of symbols: {}", encoder.model.table.len());
    encoder.encode_all();

    let result = encoder.serialize();
    let first_bits = result.3.len();
    let second_bits = result.4.len();
    let total = 64 + first_bits + second_bits;

    println!("BPI: {}", total as f64 / SYMBOL_LIST_LENGTH as f64);


    let mut encoder = FoldedStreamANSCoder::<4,4>::new(symbols.clone());
    println!("Number of symbols: {}", encoder.model.table.len());
    encoder.encode_all();

    let result = encoder.serialize();
    let first_bits = result.3.len();
    let second_bits = result.4.len();
    let total = 64 + first_bits + second_bits;

    println!("BPI: {}", total as f64 / SYMBOL_LIST_LENGTH as f64);


    let mut encoder = FoldedStreamANSCoder::<4,5>::new(symbols.clone());
    println!("Number of symbols: {}", encoder.model.table.len());
    encoder.encode_all();

    let result = encoder.serialize();
    let first_bits = result.3.len();
    let second_bits = result.4.len();
    let total = 64 + first_bits + second_bits;

    println!("BPI: {}", total as f64 / SYMBOL_LIST_LENGTH as f64);


    let mut encoder = FoldedStreamANSCoder::<4,6>::new(symbols.clone());
    println!("Number of symbols: {}", encoder.model.table.len());
    encoder.encode_all();

    let result = encoder.serialize();
    let first_bits = result.3.len();
    let second_bits = result.4.len();
    let total = 64 + first_bits + second_bits;

    println!("BPI: {}", total as f64 / SYMBOL_LIST_LENGTH as f64);


    let mut encoder = FoldedStreamANSCoder::<4,7>::new(symbols.clone());
    println!("Number of symbols: {}", encoder.model.table.len());
    encoder.encode_all();

    let result = encoder.serialize();
    let first_bits = result.3.len();
    let second_bits = result.4.len();
    let total = 64 + first_bits + second_bits;

    println!("BPI: {}", total as f64 / SYMBOL_LIST_LENGTH as f64);


    let mut encoder = FoldedStreamANSCoder::<4,8>::new(symbols.clone());
    println!("Number of symbols: {}", encoder.model.table.len());
    encoder.encode_all();

    let result = encoder.serialize();
    let first_bits = result.3.len();
    let second_bits = result.4.len();
    let total = State::BITS + first_bits + second_bits;

    println!("BPI: {}", total as f64 / SYMBOL_LIST_LENGTH as f64);
}