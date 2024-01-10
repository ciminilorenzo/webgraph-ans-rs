/*
 * This is an example of how a standard 32-bits decoder can be used.
 *
 * NB: standard 32-bits decoder means that the decoder it's gonna have a table containing as much DecoderModelEntry
 * as the size of the frame, each having the following data:
 *  - 16 bits for the frequency
 *  - 16 bits for cumulative frequency
 *  - 32 bits for the `quasi-folded`
 *
 * This is particularly useful when the frame that the decoder is using is a vector (which is the standard
 * case) since it allows to spend for each entry 64 bits. This can be crucial when the goal is too have
 * the whole DS in the fastest cache.
 *
 * At the moment the 32-bits decoder is only implemented for only one RADIX value, that is 8.
 * In fact, having RADIX equal to 8 and `quasi-folded` as a u32 implies having a decoder that can handle
 * symbols up to (2^30 - 1) since we must reserve two bits (in the MSB position) to represent the number
 * of folds (that can be up to 3).
 *
 * --- ---      --- --- --- --- --- --- --- ... ---
 *  1   2        3   4   5   6   7   8   9  ...  32
 *  folds               quasi-folded symbol
 *
 * Since the standard value for the `quasi-folded` type is u64. This means that, if we want to use u32, we
 * must use the with_parameters builder in order to build the personalized decoder, which requires
 * to specify each generic parameter.
 *
 * to create the binary to profile, run: cargo build --release --example decoder_32bit
 */

use rand::distributions::Distribution;
use rand::prelude::SmallRng;
use rand::SeedableRng;
use rand_distr::Zipf;

use folded_streaming_rans::ans::dec_model::VecFrame;
use folded_streaming_rans::ans::decoder::FoldedStreamANSDecoder;
use folded_streaming_rans::ans::encoder::FoldedStreamANSCoder;
use folded_streaming_rans::RawSymbol;

/*
/// Size of the list of symbols used during the examples.
const SYMBOL_LIST_LENGTH: usize = 1_000_000;

/// Maximum value that the zipfian distribution can output.
const MAXIMUM_SYMBOL: u64 = (1 << 30) - 1;

const FASTER_RADIX: usize = 8;

const FIDELITY: usize = 2;


fn get_folding_offset(radix: usize, fidelity: usize) -> u64 {
    (1 << (fidelity - 1)) * ((1 << radix) - 1)
}

fn get_folding_threshold(radix: usize, fidelity: usize) -> u64 {
    1 << (fidelity + radix - 1)
}

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

    let frame = VecFrame::<FASTER_RADIX, u32>::new(
        &prelude.table,
        prelude.log2_frame_size,
        get_folding_offset(FASTER_RADIX, FIDELITY),
        get_folding_threshold(FASTER_RADIX, FIDELITY),
    );

    let decoder = FoldedStreamANSDecoder::<
        FIDELITY,
        FASTER_RADIX,
        u32,
        VecFrame<FASTER_RADIX, u32>,
        Vec<u8>
    >::with_parameters(prelude, frame);

    assert_eq!(input, decoder.decode_all());
}
*/

fn main() {

}