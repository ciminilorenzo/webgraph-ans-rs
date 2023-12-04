use folded_streaming_rans::ans::enc_model::FoldedANSModel4Encoder;
use folded_streaming_rans::ans::EncoderModelEntry;


const RADIX: usize = 4;
const FIDELITY: usize = 2;

#[test]
fn correct_distribution_is_created() {
    let symbols = [1,1,1,2,2,2,3,3,4,5];
    let mut expected = Vec::new();
    expected.push(EncoderModelEntry::from((0,  0, 0)));
    expected.push(EncoderModelEntry::from((10, 687194767360, 0)));  // symbol `1`
    expected.push(EncoderModelEntry::from((10, 687194767360, 10))); // symbol `2`
    expected.push(EncoderModelEntry::from((6,  412316860416, 20))); // symbol `3`
    expected.push(EncoderModelEntry::from((3,  206158430208, 26))); // symbol `4`
    expected.push(EncoderModelEntry::from((3,  206158430208, 29))); // symbol `5`

    let model = FoldedANSModel4Encoder::new(&symbols, RADIX, FIDELITY);
    assert_eq!(expected, model.to_raw_parts());
}

#[test]
fn correct_data_is_retrieved(){
    let symbols = [1,1,1,2,2,2,3,3,4,5];
    let model = FoldedANSModel4Encoder::new(&symbols, RADIX, FIDELITY);

    assert_eq!(
        EncoderModelEntry::from((10, 687194767360, 0)), // Precomputed EncoderModelEntry for symbol `1`
        model[1]
    )
}