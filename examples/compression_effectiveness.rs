use itertools::Itertools;

use rand::distributions::Distribution;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand_distr::Zipf;

use folded_streaming_rans::ans::encoder::FoldedStreamANSCoder;
use folded_streaming_rans::{RawSymbol, State};
use folded_streaming_rans::utils::{self_entropy};


/// Size of the list of symbols used during the examples.
const SYMBOL_LIST_LENGTH: usize = 100_000_000;

/// Maximum value that the zpfian distribution can output.
const MAXIMUM_SYMBOL: u64 = 1_000_000_000;

/// Creates a sequence of size [`SYMBOL_LIST_LENGTH`], containing symbols sampled from a Zipfian
/// distribution that can output values up to [`MAXIMUM_SYMBOL`].
fn generate_zipfian_distribution() -> Vec<RawSymbol> {
    let mut rng = SmallRng::seed_from_u64(0);
    let distribution = Zipf::new(MAXIMUM_SYMBOL, 1.0).unwrap();
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

    // where M is frame size
    let original_m = SYMBOL_LIST_LENGTH;
    let original_entropy = self_entropy(&freqs_distr, original_m as f64);

    println!("original distribution BPI: {}", original_entropy / original_m as f64);

    // test compression effectiveness with different radix = 4 and fidelity 2
    let mut encoder = FoldedStreamANSCoder::<4,2>::new(symbols.clone());
    encoder.encode_all();
    let result = encoder.serialize();
    let first_bits = result.4.len();
    let second_bits = result.5.len();
    let total = State::BITS as usize + first_bits + second_bits;

    println!("\
        Encoded with radix = 4 and fidelity = 2 \
        Number of symbols: {} \
        BPI: {}", encoder.model.table.len(), total as f64 / SYMBOL_LIST_LENGTH as f64
    );

    // test compression effectiveness with different radix = 4 and fidelity 3
    let mut encoder = FoldedStreamANSCoder::<4,3>::new(symbols.clone());
    encoder.encode_all();
    let result = encoder.serialize();
    let first_bits = result.4.len();
    let second_bits = result.5.len();
    let total = State::BITS as usize + first_bits + second_bits;

    println!("\
        Encoded with radix = 4 and fidelity = 3 \
        Number of symbols: {} \
        BPI: {}", encoder.model.table.len(), total as f64 / SYMBOL_LIST_LENGTH as f64
    );

    // test compression effectiveness with different radix = 4 and fidelity 4
    let mut encoder = FoldedStreamANSCoder::<4,4>::new(symbols.clone());
    encoder.encode_all();
    let result = encoder.serialize();
    let first_bits = result.4.len();
    let second_bits = result.5.len();
    let total = State::BITS as usize + first_bits + second_bits;

    println!("\
        Encoded with radix = 4 and fidelity = 4 \
        Number of symbols: {} \
        BPI: {}", encoder.model.table.len(), total as f64 / SYMBOL_LIST_LENGTH as f64
    );

    // test compression effectiveness with different radix = 4 and fidelity 5
    let mut encoder = FoldedStreamANSCoder::<4,5>::new(symbols.clone());
    encoder.encode_all();
    let result = encoder.serialize();
    let first_bits = result.4.len();
    let second_bits = result.5.len();
    let total = State::BITS as usize + first_bits + second_bits;

    println!("\
        Encoded with radix = 4 and fidelity = 5 \
        Number of symbols: {} \
        BPI: {}", encoder.model.table.len(), total as f64 / SYMBOL_LIST_LENGTH as f64
    );

    // test compression effectiveness with different radix = 4 and fidelity 6
    let mut encoder = FoldedStreamANSCoder::<4,6>::new(symbols.clone());
    encoder.encode_all();
    let result = encoder.serialize();
    let first_bits = result.4.len();
    let second_bits = result.5.len();
    let total = State::BITS as usize + first_bits + second_bits;

    println!("\
        Encoded with radix = 4 and fidelity = 6 \
        Number of symbols: {} \
        BPI: {}", encoder.model.table.len(), total as f64 / SYMBOL_LIST_LENGTH as f64
    );

    // test compression effectiveness with different radix = 4 and fidelity 7
    let mut encoder = FoldedStreamANSCoder::<4,7>::new(symbols.clone());
    encoder.encode_all();
    let result = encoder.serialize();
    let first_bits = result.4.len();
    let second_bits = result.5.len();
    let total = State::BITS as usize + first_bits + second_bits;

    println!("\
        Encoded with radix = 4 and fidelity = 7 \
        Number of symbols: {} \
        BPI: {}", encoder.model.table.len(), total as f64 / SYMBOL_LIST_LENGTH as f64
    );

    // test compression effectiveness with different radix = 4 and fidelity 8
    let mut encoder = FoldedStreamANSCoder::<4,8>::new(symbols.clone());
    encoder.encode_all();
    let result = encoder.serialize();
    let first_bits = result.4.len();
    let second_bits = result.5.len();
    let total = State::BITS as usize + first_bits + second_bits;

    println!("\
        Encoded with radix = 4 and fidelity = 8 \
        Number of symbols: {} \
        BPI: {}", encoder.model.table.len(), total as f64 / SYMBOL_LIST_LENGTH as f64
    );
}