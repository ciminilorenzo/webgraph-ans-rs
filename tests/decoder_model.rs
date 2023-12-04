use rstest::*;

use folded_streaming_rans::ans::dec_model::{EliasFanoFrame, Rank9SelFrame, VecFrame};
use folded_streaming_rans::ans::enc_model::FoldedANSModel4Encoder;
use folded_streaming_rans::{RawSymbol, State, Symbol};


const RADIX: usize = 4;
const FIDELITY: usize = 2;


#[rstest]
#[case(
        // symbols' segments: [0,10) -> 1 | [10,19) -> 2 | [20,26) -> 3 | [26,29) -> 4 | [29,32) -> 5
        vec![1,1,1,2,2,2,3,3,4,5], vec![1_usize,0,10,2,3,29,31,20, 9], vec![1 as Symbol, 1, 2, 1, 1 , 5, 5 , 3, 1]
    )]
#[case(
        // symbols' segments: [0,6) -> 2 | [6,16) -> 3 | [16,26) -> 7 | [26,29) -> 10 | [29,32) -> 12
        vec![3,3,3,7,7,7,2,2,10,12], vec![1_usize,0,10,2,3,29,31,20, 9], vec![2 as Symbol, 2, 3, 2, 2 , 12, 12, 7, 3]
    )]
fn probe_works_for_all_types_of_frames(#[case] symbols: Vec<RawSymbol>,
                                       #[case] slots: Vec<usize>,
                                       #[case] expected_symbols: Vec<Symbol>)
{
    let encoder_model = FoldedANSModel4Encoder::new(&symbols, RADIX, FIDELITY);
    let raw_frame = encoder_model.to_raw_parts();

    let folding_offset = ((1 << (FIDELITY - 1)) * ((1 << RADIX) - 1)) as RawSymbol;
    let folding_threshold = (1 << (FIDELITY + RADIX - 1)) as RawSymbol;

    let bitvec_frame = Rank9SelFrame::new(&raw_frame, encoder_model.log2_frame_size, folding_offset, folding_threshold, RADIX);
    let vec_frame = VecFrame::new(&raw_frame, encoder_model.log2_frame_size, folding_offset, folding_threshold, RADIX);
    let elias_frame = EliasFanoFrame::new(&raw_frame, encoder_model.log2_frame_size, folding_offset, folding_threshold, RADIX);

    for i in 0..slots.len() {
        let slot_to_probe = slots[i] as State;

        assert_eq!(expected_symbols[i], bitvec_frame[slot_to_probe].symbol);
        assert_eq!(expected_symbols[i], elias_frame[slot_to_probe].symbol);
        assert_eq!(expected_symbols[i], vec_frame[slot_to_probe].symbol);
    }
}