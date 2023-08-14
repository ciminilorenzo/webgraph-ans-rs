use rand::prelude::{Distribution, StdRng};
use rand::SeedableRng;
use rand_distr::Zipf;

use folded_streaming_rans::ans::decoder_model::Rank9SelFrame;
use folded_streaming_rans::ans::folded_stream_ans_decoder::FoldedStreamANSDecoder;
use folded_streaming_rans::ans::folded_stream_ans_encoder::FoldedStreamANSCoder;
use folded_streaming_rans::RawSymbol;


/// Size of the list of symbols used to bench.
const SYMBOL_LIST_LENGTH: usize = 1_000_000;
/// Maximum value that the zpfian distribution can output.
const MAXIMUM_SYMBOL: u64 = 1_000_000_000;

const RADIX: u8 = 4;

const FIDELITY: u8 = 2;


// !!! if shared, use it as a fixture !!!
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


#[test]
fn test_decodes_correctly() {
    let symbols = get_symbols();
    let mut coder = FoldedStreamANSCoder::<RADIX, FIDELITY>::new(symbols.clone());
    coder.encode_all();

    let data = coder.serialize();

    let frame = Rank9SelFrame::new(&data.0, data.2);
    let mut decoder = FoldedStreamANSDecoder::<RADIX, FIDELITY, Rank9SelFrame>::new(
        data.1,
        frame,
        data.2,
        data.3,
        data.4,
    );

    assert_eq!(decoder.decode_all(), symbols);
}
