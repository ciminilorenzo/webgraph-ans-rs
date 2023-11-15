use folded_streaming_rans::RawSymbol;
use rand::distributions::Distribution;
use rand::prelude::StdRng;
use rand::SeedableRng;
use rand_distr::Zipf;

pub mod decoder;
pub mod encoder;
pub mod model_for_decoder;

/// Size of the list of symbols used to bench.
const SYMBOL_LIST_LENGTH: usize = 500_000;

/// Maximum value that the zpfian distribution can output.
const MAXIMUM_SYMBOL: u64 = 10_000_000_000;

pub const RADIX: u8 = 4;

pub const FIDELITY: u8 = 2;

/// Creates a sequence of size [`SYMBOL_LIST_LENGTH`], containing symbols sampled from a Zipfian
/// distribution that can output values up to [`MAXIMUM_SYMBOL`].
fn get_symbols() -> Vec<RawSymbol> {
    let mut rng = StdRng::seed_from_u64(0);
    let distribution = Zipf::new(MAXIMUM_SYMBOL, 1.0).unwrap();
    let mut symbols = Vec::with_capacity(SYMBOL_LIST_LENGTH);

    for _ in 0..SYMBOL_LIST_LENGTH {
        symbols.push(distribution.sample(&mut rng) as RawSymbol);
    }
    symbols
}
