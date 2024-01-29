mod utils;

use rstest::*;

use folded_streaming_rans::{RawSymbol, State};
use folded_streaming_rans::ans::model4decoder::*;
use folded_streaming_rans::ans::model4encoder::SingleANSModel4Encoder;

const RADIX: usize = 4;
const FIDELITY: usize = 2;

#[rstest]
#[case(
        // symbols' segments: [0,10) -> 1 | [10,19) -> 2 | [20,26) -> 3 | [26,29) -> 4 | [29,32) -> 5
        vec![1,1,1,2,2,2,3,3,4,5], vec![1_usize,0,10,2,3,29,31,20, 9], vec![1_u64, 1, 2, 1, 1 , 5, 5 , 3, 1]
    )]
#[case(
        // symbols' segments: [0,6) -> 2 | [6,16) -> 3 | [16,26) -> 7 | [26,29) -> 10 | [29,32) -> 12
        vec![3,3,3,7,7,7,2,2,10,12], vec![1_usize,0,10,2,3,29,31,20, 9], vec![2u64, 2, 3, 2, 2 , 12, 12, 7, 3]
    )]
fn probe_works_for_all_types_of_frames(
    #[case] symbols: Vec<RawSymbol>,
    #[case] slots: Vec<usize>,
    #[case] expected_symbols: Vec<u64>,
) {
    let model4encoder = SingleANSModel4Encoder::new(&symbols, FIDELITY, RADIX);
    let folding_threshold = (1 << (FIDELITY + RADIX - 1)) as u64;
    let folding_offset = ((1 << RADIX) - 1) * (1 << (FIDELITY - 1));

    let bitvec_frame = Rank9SelFrame::<RADIX, u64>::new(
        &model4encoder.table,
        model4encoder.log2_frame_size,
        folding_offset,
        folding_threshold,
    );
    let vec_frame = VecFrame::<RADIX, u64>::new(
        &model4encoder.table,
        model4encoder.log2_frame_size,
        folding_offset,
        folding_threshold,
    );
    let elias_frame = EliasFanoFrame::<RADIX, u64>::new(
        &model4encoder.table,
        model4encoder.log2_frame_size,
        folding_offset,
        folding_threshold,
    );

    for i in 0..slots.len() {
        let slot_to_probe = slots[i] as State;

        assert_eq!(expected_symbols[i], bitvec_frame[slot_to_probe].quasi_folded);
        assert_eq!(expected_symbols[i], elias_frame[slot_to_probe].quasi_folded);
        assert_eq!(expected_symbols[i], vec_frame[slot_to_probe].quasi_folded);
    }
}