/*
 * This is an example of how a standard 64-bits decoder can be used (which is the standard one).
 *
 * NB: standard 64-bits decoder means that the decoder it's gonna have a table containing as much DecoderModelEntry
 * as the size of the frame, each having the following data:
 *  - 16 bits for the frequency
 *  - 16 bits for cumulative frequency
 *  - 64 bits for the `quasi-folded`
 *
 * This means that we are going to spend 96 bits for each entry. This is a theoretical value since it
 * can be aligned to wider values.
 *
 * The 64 bits decoder can be used with any value of RADIX (if RADIX is 8 the frame can be a vector
 * otherwise it must be a BitVec).
 *
 * The 64-bits decoder follows the following convention:
 *  - the 16 MSB are used to store the number of folds
 *  - the 48 LSB are used to store the quasi-folded symbol
 * This implies that the 64-bits decoder can handle symbols up to 2^48 - 1.
 *
 * --- --- ... ---      --- --- ... ---
 *  1   2      16        17  18 ...  64
 *      folds          quasi-folded symbol
 *
 * Since the standard value for the `quasi-folded` type is u64. This means that we don't have to specify
 * each generic parameter when building the decoder.
 *
 * to create the binary to profile, run: cargo build --release --example decoder
 *
 */

/*
use rand::distributions::Distribution;
use rand::prelude::SmallRng;
use rand::SeedableRng;
use rand_distr::Zipf;

use folded_streaming_rans::ans::decoder::FoldedStreamANSDecoder;
use folded_streaming_rans::ans::encoder::FoldedStreamANSCoder;
use folded_streaming_rans::RawSymbol;


/// Size of the list of symbols used during the examples.
const SYMBOL_LIST_LENGTH: usize = 1_000_000;

/// Maximum value that the zipfian distribution can output.
const MAXIMUM_SYMBOL: u64 = (1 << 30) - 1;

const FIDELITY: usize = 2;


pub fn generate_zipfian_distribution() -> Vec<RawSymbol> {
    let mut rng = SmallRng::seed_from_u64(0);
    let distribution = Zipf::new(MAXIMUM_SYMBOL, 1.0).unwrap();
    let mut symbols = Vec::with_capacity(SYMBOL_LIST_LENGTH);

    for _ in 0..SYMBOL_LIST_LENGTH {
        symbols.push(distribution.sample(&mut rng) as RawSymbol);
    }
    symbols
}

fn main() {
    let input = generate_zipfian_distribution();
    let mut encoder = FoldedStreamANSCoder::<FIDELITY>::new(&input);
    encoder.encode_all();
    let prelude = encoder.serialize();
    let decoder = FoldedStreamANSDecoder::<FIDELITY>::new(prelude);

    assert_eq!(input, decoder.decode_all());
}
*/

fn main() {

}
