/*
 * Utility functions and consts used by the tests.
 *
 */

use rand::prelude::{Distribution, SmallRng};
use rand::SeedableRng;
use rand_distr::Zipf;
use folded_streaming_rans::RawSymbol;

pub const FIDELITY: usize = 2;
pub const FASTER_RADIX: usize = 8;
/// Size of the list of symbols used to test.
pub const SYMBOL_LIST_LENGTH: usize = 1_000_000;
/// Maximum value that the zipfian distribution can output.
const MAXIMUM_SYMBOL: u64 = 1 << 20;


pub fn get_folding_offset(radix: usize, fidelity: usize) -> u64 {
    (1 << (fidelity - 1)) * ((1 << radix) - 1)
}

pub fn get_folding_threshold(radix: usize, fidelity: usize) -> u64 {
    1 << (fidelity + radix - 1)
}

/// Creates a sequence of size [`SYMBOL_LIST_LENGTH`], containing symbols sampled from a Zipfian
/// distribution that can output values up to [`MAXIMUM_SYMBOL`].
pub fn get_symbols(seed: u64) -> Vec<RawSymbol> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let distribution = Zipf::new(MAXIMUM_SYMBOL, 1.0).unwrap();
    let mut symbols = Vec::with_capacity(SYMBOL_LIST_LENGTH);

    for _ in 0..SYMBOL_LIST_LENGTH {
        symbols.push(distribution.sample(&mut rng) as RawSymbol);
    }
    symbols
}