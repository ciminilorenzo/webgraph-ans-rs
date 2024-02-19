use folded_streaming_rans::RawSymbol;
use rand::prelude::{Distribution, SmallRng};
use rand::SeedableRng;
use rand_distr::Zipf;

#[allow(dead_code)]
pub const SYMBOL_LIST_LENGTH: usize = 1_000_000;

#[allow(dead_code)]
/// Maximum value that the zipfian distribution can output.
const MAXIMUM_SYMBOL: u64 = 1 << 30;

#[allow(dead_code)]
/// Creates a sequence of size [`SYMBOL_LIST_LENGTH`], containing symbols sampled from a Zipfian
/// distribution that can output values up to [`MAXIMUM_SYMBOL`].
pub fn get_zipfian_distr(seed: u64, exponent: f32) -> Vec<RawSymbol> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let distribution = Zipf::new(MAXIMUM_SYMBOL, exponent).unwrap();
    let mut symbols = Vec::with_capacity(SYMBOL_LIST_LENGTH);

    for _ in 0..SYMBOL_LIST_LENGTH {
        symbols.push(distribution.sample(&mut rng) as RawSymbol);
    }
    symbols
}
