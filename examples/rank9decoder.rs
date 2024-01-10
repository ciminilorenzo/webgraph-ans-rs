/*
 * Internally, a decoder can even use a non-standard data structure to implement the symbol primitive.
 * In fact, in order to give flexibility to the user, three options are supplied:
 * - the table (which is the standard option)
 * - EliasFano encoding
 * - Rank9Sel bitvec
 * In this example we are going to use a 64-bit decoder that internally uses a Rank9Sel.
 *
 * Note that when using a data structures that is different from a table, the decoder's model has to
 * use an implicit link between each frame's slot and the corresponding symbol. This generally allows
 * for improvements in terms of memory but not in terms of speed.
 *
 * to create the binary to profile, run: cargo build --release --example rank9decoder
 *
 */
use rand::distributions::Distribution;
use rand::prelude::SmallRng;
use rand::SeedableRng;
use rand_distr::Zipf;
use folded_streaming_rans::ans::dec_model::Rank9SelFrame;

use folded_streaming_rans::ans::decoder::FoldedStreamANSDecoder;
use folded_streaming_rans::ans::encoder::FoldedStreamANSCoder;
use folded_streaming_rans::RawSymbol;

/*
/// Size of the list of symbols used during the examples.
const SYMBOL_LIST_LENGTH: usize = 1_000_000;

/// Maximum value that the zipfian distribution can output.
const MAXIMUM_SYMBOL: u64 = (1 << 30) - 1;

const FIDELITY: usize = 2;

const FASTER_RADIX: usize = 8;

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

    let frame = Rank9SelFrame::<FASTER_RADIX, u64>::new(
        &prelude.table,
        prelude.log2_frame_size,
        get_folding_offset(FASTER_RADIX, FIDELITY),
        get_folding_threshold(FASTER_RADIX, FIDELITY),
    );

    let decoder = FoldedStreamANSDecoder::<
        FIDELITY,
        FASTER_RADIX,
        u64,
        Rank9SelFrame<FASTER_RADIX, u64>,
        Vec<u8>
    >::with_parameters(prelude, frame);

    assert_eq!(input, decoder.decode_all());
}
*/

fn main() {

}