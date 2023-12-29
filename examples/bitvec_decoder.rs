/*
 * All the examples but this use a vec of bytes to handle the folded bits (which is the fastest option)
 * during the encoding process. This is something doable only when using a radix equal to 8. To give
 * the user the chance to use all other values, we implement the `Fold` trait for `BitVec<usize, Msb0>`
 * as well.
 *
 * In this example we will show how it's possibile to use a decoder with `RADIX`equal to 10.
 *
 * Note that this example can be made only when the decoder is a standard 64-bits decoder. This is
 * because a 32-bits decoder is usable only when `RADIX` is equal to 8 (since the trait Quasi for u32
 * is currently implemented only for that value of `RADIX`).
 *
 * to create the binary to profile, run: cargo build --release --example bitvec_decoder
 */

use bitvec::order::Msb0;
use bitvec::vec::BitVec;
use rand::prelude::{Distribution, SmallRng};
use rand::SeedableRng;
use rand_distr::Zipf;

use folded_streaming_rans::ans::dec_model::VecFrame;
use folded_streaming_rans::ans::decoder::FoldedStreamANSDecoder;
use folded_streaming_rans::ans::encoder::FoldedStreamANSCoder;
use folded_streaming_rans::RawSymbol;

/// Size of the list of symbols used during the examples.
const SYMBOL_LIST_LENGTH: usize = 1_000_000;

/// Maximum value that the zipfian distribution can output.
const MAXIMUM_SYMBOL: u64 = (1 << 30) - 1;

const RADIX: usize = 10; // <--- This time is not 8!

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
    let mut encoder = FoldedStreamANSCoder::<
        FIDELITY,
        RADIX,
        BitVec<usize, Msb0>
    >::with_parameters(&input, BitVec::new());

    encoder.encode_all();
    let prelude = encoder.serialize();

    let frame = VecFrame::<RADIX, u64>::new(
        &prelude.table,
        prelude.log2_frame_size,
        get_folding_offset(RADIX, FIDELITY),
        get_folding_threshold(RADIX, FIDELITY),
    );

    let decoder = FoldedStreamANSDecoder::<
        FIDELITY,
        RADIX,
        u64,
        VecFrame<RADIX, u64>,
        BitVec<usize, Msb0>
        >::with_parameters(prelude, frame);

    assert_eq!(input, decoder.decode_all());
}