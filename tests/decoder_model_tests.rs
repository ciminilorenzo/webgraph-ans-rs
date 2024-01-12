mod utils;

use rstest::*;
use utils::*;

use folded_streaming_rans::{RawSymbol, State};
use folded_streaming_rans::multi_model_ans::model4decoder::{EliasFanoFrame, Rank9SelFrame, VecFrame};
use folded_streaming_rans::multi_model_ans::model4encoder::SymbolLookup;
use folded_streaming_rans::multi_model_ans::model4encoder_builder::AnsModel4EncoderBuilder;

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
    let mut encoder_model_builder = AnsModel4EncoderBuilder::<FIDELITY, RADIX>::new(1);

    for symbol in symbols {
        encoder_model_builder.push_symbol(symbol, 0).unwrap();
    }

    let encoder_model = encoder_model_builder.build();
    let tables = encoder_model.tables;
    let frame_sizes = encoder_model.frame_sizes;
    let folding_offset = get_folding_offset(RADIX, FIDELITY);
    let folding_threshold = get_folding_threshold(RADIX, FIDELITY);

    let bitvec_frame = Rank9SelFrame::<RADIX, u64>::new(
        tables.clone(),
        frame_sizes.clone(),
        folding_offset,
        folding_threshold,
    );
    let vec_frame = VecFrame::<RADIX, u64>::new(
        tables.clone(),
        frame_sizes.clone(),
        folding_offset,
        folding_threshold,
    );
    let elias_frame = EliasFanoFrame::<RADIX, u64>::new(
        tables.clone(),
        frame_sizes.clone(),
        folding_offset,
        folding_threshold,
    );

    for i in 0..slots.len() {
        let slot_to_probe = slots[i] as State;

        assert_eq!(expected_symbols[i], bitvec_frame.symbol(slot_to_probe, 0).quasi_folded);
        assert_eq!(expected_symbols[i], elias_frame.symbol(slot_to_probe, 0).quasi_folded);
        assert_eq!(expected_symbols[i], vec_frame.symbol(slot_to_probe, 0).quasi_folded);
    }
}