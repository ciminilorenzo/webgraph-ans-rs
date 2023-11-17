use rand::prelude::Distribution;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand_distr::Zipf;

use folded_streaming_rans::ans::dec_model::VecFrame;
use folded_streaming_rans::ans::decoder::FoldedStreamANSDecoder;
use folded_streaming_rans::ans::encoder::FoldedStreamANSCoder;
use folded_streaming_rans::RawSymbol;


/// Size of the list of symbols used to test.
const SYMBOL_LIST_LENGTH: usize = 1_000_000;

/// Maximum value that the zpfian distribution can output.
const MAXIMUM_SYMBOL: u64 = 10_000_000_000;

const RADIX: u8 = 4;

const FIDELITY: u8 = 2;


// !!! if shared, use it as a fixture !!!
/// Creates a sequence of size [`SYMBOL_LIST_LENGTH`], containing symbols sampled from a Zipfian
///
/// distribution that can output values up to [`MAXIMUM_SYMBOL`].
fn get_symbols() -> Vec<RawSymbol> {
    let mut rng = SmallRng::seed_from_u64(0);
    let distribution = Zipf::new(MAXIMUM_SYMBOL, 1.0).unwrap();
    let mut symbols = Vec::with_capacity(SYMBOL_LIST_LENGTH);

    for _ in 0.. SYMBOL_LIST_LENGTH {
        symbols.push(distribution.sample(&mut rng) as RawSymbol);
    }
    symbols
}


// let's test that the decoder is able to retrieve the original sequence of symbols encoded by the
// encoder
#[test]
fn test_decodes_correctly() {
    let symbols = get_symbols();
    let mut coder = FoldedStreamANSCoder::<RADIX, FIDELITY>::new(symbols.to_vec());
    coder.encode_all();

    let data = coder.serialize();

    let mut decoder = FoldedStreamANSDecoder::<RADIX, FIDELITY, VecFrame>::new(
        &data.1,
        data.3,
        data.2,
        data.4,
        data.5,
        data.0,
    );

    assert_eq!(decoder.decode_all(), symbols);
}