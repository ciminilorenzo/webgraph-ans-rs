use folded_streaming_rans::multi_model_ans::encoder::ANSEncoder;
use folded_streaming_rans::multi_model_ans::model4encoder_builder::AnsModel4EncoderBuilder;

const RADIX: usize = 4;
const FIDELITY: usize = 2;

#[test]
fn builder_is_created_without_errors() {
    let first_symbols = vec![1,1,1,2,2,2,3,3,4,5];
    let second_symbols = vec![1,1,1,2,2,2,3,3,4,5];
    let third_symbols = vec![1,1,1,2,2,2,3,3,4,5];

    let mut builder = AnsModel4EncoderBuilder::<FIDELITY, RADIX>::new(3);

    for index in 0..first_symbols.len() {
        builder.push_symbol(first_symbols[index], 0).unwrap();
        builder.push_symbol(second_symbols[index], 1).unwrap();
        builder.push_symbol(third_symbols[index], 2).unwrap();
    }
}

#[test]
fn encoder_encodes_without_errors() {
    let first_symbols = vec![1, 1, 1, 2, 2, 2, 3, 3, 4, 5];
    let second_symbols = vec![1, 1, 1, 2, 2, 2, 3, 3, 4, 5];
    let third_symbols = vec![1, 1, 1, 2, 2, 2, 3, 3, 4, 5];

    let mut builder = AnsModel4EncoderBuilder::<FIDELITY, RADIX>::new(3);

    for index in 0..first_symbols.len() {
        builder.push_symbol(first_symbols[index], 0).unwrap();
        builder.push_symbol(second_symbols[index], 1).unwrap();
        builder.push_symbol(third_symbols[index], 2).unwrap();
    }
    let model = builder.build();
    let mut encoder = ANSEncoder::<FIDELITY>::new(model);

    for index in 0..first_symbols.len() {
        encoder.encode(first_symbols[index], 0);
    }
}

