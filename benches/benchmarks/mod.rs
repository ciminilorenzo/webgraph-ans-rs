use folded_streaming_rans::RawSymbol;

use rand::distributions::Distribution;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand_distr::Zipf;

pub mod graph_encoding;
pub mod model4encoder_building;

/// Size of the list of symbols used to bench.
const SYMBOL_LIST_LENGTH: usize = 1_000_000;

/// Maximum value that the zipfian distribution can output.
const MAXIMUM_SYMBOL: u64 = 1 << 20;

pub const RADIX: usize = 8;

pub const FIDELITY: usize = 1;

/// Creates a sequence of size [`SYMBOL_LIST_LENGTH`], containing symbols sampled from a Zipfian
/// distribution that can output values up to [`MAXIMUM_SYMBOL`].
fn get_symbols() -> Vec<RawSymbol> {
    let mut rng = SmallRng::seed_from_u64(0);
    let distribution = Zipf::new(MAXIMUM_SYMBOL, 1.0).unwrap();
    let mut symbols = Vec::with_capacity(SYMBOL_LIST_LENGTH);

    for _ in 0..SYMBOL_LIST_LENGTH {
        symbols.push(distribution.sample(&mut rng) as RawSymbol);
    }
    symbols
}
