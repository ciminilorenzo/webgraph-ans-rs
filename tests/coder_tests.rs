mod common;

use folded_streaming_rans::ans::dec_model::VecFrame;
use folded_streaming_rans::ans::decoder::FoldedStreamANSDecoder;
use folded_streaming_rans::ans::encoder::FoldedStreamANSCoder;
use crate::common::*;

#[test]
fn decoder_decodes_correctly() {
    let symbols = get_symbols();
    let mut coder = FoldedStreamANSCoder::<FIDELITY>::new(&symbols);
    coder.encode_all();

    let prelude = coder.serialize();
    let decoder = FoldedStreamANSDecoder::<FIDELITY>::new(prelude);

    assert_eq!(symbols, decoder.decode_all());
}

#[test]
fn decoder_32bits_decodes_correctly() {
    let symbols = get_symbols();
    let mut coder = FoldedStreamANSCoder::<FIDELITY>::new(&symbols);

    coder.encode_all();

    let prelude = coder.serialize();

    let frame = VecFrame::<FASTER_RADIX, u32>::new(
        &prelude.table,
        prelude.log2_frame_size,
        get_folding_offset(FASTER_RADIX, FIDELITY),
        get_folding_threshold(FASTER_RADIX, FIDELITY)
    );

    let decoder = FoldedStreamANSDecoder::<
        FIDELITY,
        FASTER_RADIX,
        u32,
        VecFrame<FASTER_RADIX, u32>,
        Vec<u8>
    >::with_parameters(prelude, frame);

    assert_eq!(symbols, decoder.decode_all());
}

#[test]
#[should_panic]
fn decoder_model_panics_when_input_out_of_bounds() {
    let input = vec![(1 << 50) - 1]; // in any case we can handle raw symbols up to 2^48-1

    let mut encoder = FoldedStreamANSCoder::<FIDELITY>::new(&input);
    encoder.encode_all();
    let prelude = encoder.serialize();
    let decoder = FoldedStreamANSDecoder::<FIDELITY>::new(prelude);

    assert_eq!(input, decoder.decode_all());

}

#[test]
#[should_panic]
fn decoder32bit_panics_when_input_out_of_bounds() {
    let input = vec![(1 << 32) - 1]; // in this case we can handle raw symbols up to 2^30 - 1
    let mut encoder = FoldedStreamANSCoder::<FIDELITY>::new(&input);
    encoder.encode_all();
    let prelude = encoder.serialize();

    let frame = VecFrame::<FASTER_RADIX, u32>::new(
        &prelude.table,
        prelude.log2_frame_size,
        get_folding_offset(FASTER_RADIX, FIDELITY),
        get_folding_threshold(FASTER_RADIX, FIDELITY)
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
