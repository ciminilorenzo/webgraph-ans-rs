use rand::distributions::Distribution;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand_distr::Zipf;

use folded_streaming_rans::ans::dec_model::VecFrame;
use folded_streaming_rans::ans::decoder::FoldedStreamANSDecoder;
use folded_streaming_rans::ans::encoder::FoldedStreamANSCoder;
use folded_streaming_rans::ans::FASTER_RADIX;
use folded_streaming_rans::RawSymbol;

// This is an example of how a generic decoder can be used. The generic decoder can have any reasonable
// radix and fidelity values and can use any type that implements the Foldable trait as folded bits.
// of 8 and a Vec<u8> as folded bits. The fastest decoder is even the one that uses a Rank9SelFrame
// as frame.

/// Size of the list of symbols used during the examples.
const SYMBOL_LIST_LENGTH: usize = 50_000_000;

/// Maximum value that the zipfian distribution can output.
const MAXIMUM_SYMBOL: u64 = 1_000_000_000;

/// Creates a sequence of size [`SYMBOL_LIST_LENGTH`], containing symbols sampled from a Zipfian
/// distribution that can output values up to [`MAXIMUM_SYMBOL`].
pub fn generate_zipfian_distribution() -> Vec<RawSymbol> {
    let mut rng = SmallRng::seed_from_u64(0);
    let distribution = Zipf::new(MAXIMUM_SYMBOL, 1.0).unwrap();
    let mut symbols = Vec::with_capacity(SYMBOL_LIST_LENGTH);

    for _ in 0..SYMBOL_LIST_LENGTH {
        symbols.push(distribution.sample(&mut rng) as RawSymbol);
    }
    symbols
}

const FIDELITY: usize = 1;

fn main() {
    let symbols = generate_zipfian_distribution();

    let mut encoder = FoldedStreamANSCoder::<1>::new(&symbols);
    encoder.encode_all();

    let prelude = encoder.serialize();

    let folding_offset = ((1 << (FIDELITY - 1)) * ((1 << FASTER_RADIX) - 1)) as RawSymbol;
    let folding_threshold = (1 << (FIDELITY + FASTER_RADIX - 1)) as RawSymbol;
    let vec_frame = VecFrame::new(
        &prelude.table,
        prelude.log2_frame_size,
        folding_offset,
        folding_threshold,
        FASTER_RADIX,
    );

    let decoder = FoldedStreamANSDecoder::<FIDELITY, FASTER_RADIX, VecFrame<FASTER_RADIX>, Vec<u8>>::with_parameters(prelude, vec_frame);
    let result = decoder.decode_all();

    assert_eq!(symbols, result)
}
