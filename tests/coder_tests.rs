use rand::prelude::Distribution;
use rand::rngs::SmallRng;
use rand::SeedableRng;

use folded_streaming_rans::ans::dec_model::Rank9SelFrame;
use rand_distr::Zipf;

use folded_streaming_rans::ans::decoder::FoldedStreamANSDecoder;
use folded_streaming_rans::ans::encoder::FoldedStreamANSCoder;
use folded_streaming_rans::ans::FASTER_RADIX;
use folded_streaming_rans::RawSymbol;

fn get_folding_offset(radix: usize, fidelity: usize) -> u64 {
    (1 << (fidelity - 1)) * ((1 << radix) - 1)
}

fn get_folding_threshold(radix: usize, fidelity: usize) -> u64 {
    1 << (fidelity + radix - 1)
}

/// Size of the list of symbols used to test.
const SYMBOL_LIST_LENGTH: usize = 1_000_000;

/// Maximum value that the zipfian distribution can output.
const MAXIMUM_SYMBOL: u64 = 10_000_000_000;

const FIDELITY: usize = 2;

/// Creates a sequence of size [`SYMBOL_LIST_LENGTH`], containing symbols sampled from a Zipfian
///
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

#[test]
fn create_with_right_parameters() {
    let symbols = get_symbols();
    let mut coder = FoldedStreamANSCoder::<FIDELITY>::new(&symbols);

    coder.encode_all();
    let prelude = coder.serialize();
    let folding_offset = get_folding_offset(FASTER_RADIX, FIDELITY);
    let folding_threshold = get_folding_threshold(FASTER_RADIX, FIDELITY);

    let frame = Rank9SelFrame::new(
        &prelude.table,
        prelude.log2_frame_size,
        folding_offset,
        folding_threshold,
        FASTER_RADIX,
    );

    let decoder = FoldedStreamANSDecoder::<
        FIDELITY,
        FASTER_RADIX,
        Rank9SelFrame<FASTER_RADIX>,
        Vec<u8>,
    >::with_parameters(prelude, frame);
    decoder.decode_all();
}

#[test]
fn test_decodes_correctly() {
    let symbols = get_symbols();
    let mut coder = FoldedStreamANSCoder::<FIDELITY>::new(&symbols);
    coder.encode_all();

    let prelude = coder.serialize();
    let decoder = FoldedStreamANSDecoder::<FIDELITY>::new(prelude);

    assert_eq!(symbols, decoder.decode_all());
}