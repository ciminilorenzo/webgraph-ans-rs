mod common;

use rand::prelude::SliceRandom;
use folded_streaming_rans::RawSymbol;
use folded_streaming_rans::multi_model_ans::decoder::ANSDecoder;
use folded_streaming_rans::multi_model_ans::encoder::ANSEncoder;
use folded_streaming_rans::multi_model_ans::model4encoder_builder::AnsModel4EncoderBuilder;

use crate::common::*;

#[test]
fn decoder_decodes_correctly_a_single_dummy_sequence() {
    let source = vec![1_u64, 1, 1, 2, 2, 2, 3, 3, 4, 5];
    let mut encoder_model_builder = AnsModel4EncoderBuilder::<FIDELITY, FASTER_RADIX>::new(1);

    for symbol in &source {
        encoder_model_builder.push_symbol(*symbol, 0).unwrap(); // first traversal to build the statistics
    }

    let encoder_model = encoder_model_builder.build();
    let mut encoder = ANSEncoder::<FIDELITY>::new(encoder_model);

    for symbol in &source {
        encoder.encode(*symbol, 0); // second traversal to encode the symbols
    }

    let prelude = encoder.serialize();
    let mut decoder = ANSDecoder::<FIDELITY>::new(prelude);
    let mut decoded_symbols: Vec<RawSymbol> = Vec::new();

    for _ in 0..source.len() {
        decoded_symbols.push(decoder.decode(0));
    }
    decoded_symbols.reverse(); // since encodes as a LIFO

    assert_eq!(decoded_symbols, source);
}

#[test]
fn decoder_decodes_correctly_dummy_sequences() {
    let first_source = vec![1_u64, 1, 1, 2, 2, 2, 3, 3, 4, 5];
    let second_source = vec![1_u64, 3, 3, 3, 2, 2, 3, 3, 4, 5];
    let mut encoder_model_builder = AnsModel4EncoderBuilder::<FIDELITY, FASTER_RADIX>::new(2);

    for index in 0..first_source.len() {
        encoder_model_builder.push_symbol(first_source[index], 0).unwrap();
        encoder_model_builder.push_symbol(second_source[index], 1).unwrap();
    }

    let encoder_model = encoder_model_builder.build();
    let mut encoder = ANSEncoder::<FIDELITY>::new(encoder_model);

    for index in 0..first_source.len() {
        encoder.encode(first_source[index], 0);
        encoder.encode(second_source[index], 1);
    }

    let prelude = encoder.serialize();
    let mut decoder = ANSDecoder::<FIDELITY>::new(prelude);

    let mut first_decoded_sequence: Vec<RawSymbol> = Vec::new();
    let mut second_decoded_sequence: Vec<RawSymbol> = Vec::new();

    for _ in 0..first_source.len() {
        second_decoded_sequence.push(decoder.decode(1)); // let's start from the last encoded
        first_decoded_sequence.push(decoder.decode(0));
    }

    first_decoded_sequence.reverse(); // since encodes as a LIFO
    second_decoded_sequence.reverse();

    assert_eq!(first_decoded_sequence, first_source);
    assert_eq!(second_decoded_sequence, second_source);
}

#[test]
fn decoder_decodes_correctly_dummy_interleaved_sequences() {
    // (model_index, symbol)
    let first_source = vec![(0, 1_u64),(0, 1),(0, 1),(0, 2),(0, 2),(0,2),(0, 3),(0, 3),(0, 4),(0,5)];
    let second_source = vec![(1, 1_u64),(1, 1),(1, 1),(1, 1),(1, 4),(1,3),(1, 3),(1, 3),(1, 4),(1,10)];
    let mut encoder_model_builder = AnsModel4EncoderBuilder::<FIDELITY, FASTER_RADIX>::new(2);

    for index in 0..first_source.len() {
        encoder_model_builder.push_symbol(first_source[index].1, 0).unwrap();
        encoder_model_builder.push_symbol(second_source[index].1, 1).unwrap();
    }

    let encoder_model = encoder_model_builder.build();
    let mut encoder = ANSEncoder::<FIDELITY>::new(encoder_model);

    // create a unique source of symbols and randomize it
    let mut random_unified_source = vec![first_source, second_source].concat();
    random_unified_source.shuffle(&mut rand::thread_rng());

    let expected_first_source = random_unified_source
        .iter()
        .filter(|(model_index, _)| *model_index == 0)
        .map(|(_, symbol)| *symbol).collect::<Vec<u64>>();

    let expected_second_source = random_unified_source
        .iter()
        .filter(|(model_index, _)| *model_index == 1)
        .map(|(_, symbol)| *symbol).collect::<Vec<u64>>();

    // now encode each symbol (in random order) with the corresponding model previously associated
    for (model_index, symbol) in random_unified_source.iter() {
        encoder.encode(*symbol, *model_index);
    }

    let prelude = encoder.serialize();
    let mut decoder = ANSDecoder::<FIDELITY>::new(prelude);
    let mut decoded: Vec<Vec<RawSymbol>> = vec![Vec::new(), Vec::new()];

    random_unified_source.reverse(); // now let's reverse the order of the model_index-symbol pairs to decode in reverse

    for (model_index, _symbol) in &random_unified_source {
        decoded[*model_index].push(decoder.decode(*model_index));
    }

    decoded[0].reverse(); // they have been decoded in reversed order
    decoded[1].reverse();

    assert_eq!(expected_first_source, decoded[0]);
    assert_eq!(expected_second_source, decoded[1]);
}


#[test]
fn decoder_decodes_correctly_real_interleaved_sequences() {
    // (model_index, symbol)
    let first_sequence = get_zipfian_distr(0, 1.0).iter().map(|symbol| (0, *symbol)).collect::<Vec<(usize, RawSymbol)>>();
    let second_sequence = get_zipfian_distr(1, 1.0).iter().map(|symbol| (1, *symbol)).collect::<Vec<(usize, RawSymbol)>>();
    let third_sequence = get_zipfian_distr(2, 1.0).iter().map(|symbol| (2, *symbol)).collect::<Vec<(usize, RawSymbol)>>();
    let fourth_sequence = get_zipfian_distr(1, 1.0).iter().map(|symbol| (3, *symbol)).collect::<Vec<(usize, RawSymbol)>>();

    let mut encoder_model_builder = AnsModel4EncoderBuilder::<FIDELITY, FASTER_RADIX>::new(4);

    for index in 0..first_sequence.len() {
        encoder_model_builder.push_symbol(first_sequence[index].1, 0).unwrap();
        encoder_model_builder.push_symbol(second_sequence[index].1, 1).unwrap();
        encoder_model_builder.push_symbol(third_sequence[index].1, 2).unwrap();
        encoder_model_builder.push_symbol(fourth_sequence[index].1, 3).unwrap();
    }
    let encoder_model = encoder_model_builder.build();
    let mut encoder = ANSEncoder::<FIDELITY>::new(encoder_model);
    let mut source = vec![first_sequence, second_sequence, third_sequence, fourth_sequence].concat();
    source.shuffle(&mut rand::thread_rng()); // randomize the order of the symbols to encode

    let mut expected = vec![Vec::new(), Vec::new(), Vec::new(), Vec::new()];

    for (model_index, symbol) in &source {
        expected[*model_index].push(*symbol);
    }

    // now encode each symbol with the corresponding model previously associated
    for (model_index, symbol) in source.iter() {
        encoder.encode(*symbol, *model_index);
    }

    let prelude = encoder.serialize();
    let mut decoder = ANSDecoder::<FIDELITY>::new(prelude);
    let mut decoded: Vec<Vec<RawSymbol>> = vec![Vec::new(), Vec::new(), Vec::new(), Vec::new()];

    source.reverse(); // now let's reverse the order of the model_index-symbol pairs to decode in reverse

    for (model_index, _symbol) in &source {
        decoded[*model_index].push(decoder.decode(*model_index));
    }

    decoded.iter_mut().for_each(|sequence| sequence.reverse()); // they have been decoded in reversed order

    assert_eq!(expected[0], decoded[0]);
    assert_eq!(expected[1], decoded[1]);
    assert_eq!(expected[2], decoded[2]);
    assert_eq!(expected[3], decoded[3]);
}

#[test]
// Frame sizes: [9, 14, 13, 10] (note that these actually are log_2 of the frame sizes)
fn decoder_decodes_correctly_real_interleaved_sequences_with_different_frame_sizes() {
    // let's get a random sequence of symbols to encode and map them to have this shape: (model_index, symbol)
    let first_sequence = get_zipfian_distr(0, 1.3)
        .iter()
        .map(|symbol| (0, *symbol)).collect::<Vec<(usize, RawSymbol)>>()[..SYMBOL_LIST_LENGTH/2000].to_vec();

    let second_sequence = get_zipfian_distr(1, 1.2)
        .iter()
        .map(|symbol| (1, *symbol)).collect::<Vec<(usize, RawSymbol)>>();

    let third_sequence = get_zipfian_distr(2, 1.0)
        .iter()
        .map(|symbol| (2, *symbol))
        .collect::<Vec<(usize, RawSymbol)>>();

    let fourth_sequence = get_zipfian_distr(3, 1.4)
        .iter()
        .map(|symbol| (3, *symbol))
        .collect::<Vec<(usize, RawSymbol)>>()[..SYMBOL_LIST_LENGTH/1000].to_vec();

    // now let's unify each source in a single one and randomize it
    let mut source = vec![first_sequence, second_sequence, third_sequence, fourth_sequence].concat();
    source.shuffle(&mut rand::thread_rng());

    let mut encoder_model_builder = AnsModel4EncoderBuilder::<FIDELITY, FASTER_RADIX>::new(4);

    for (model_index, symbol) in &source {
        encoder_model_builder.push_symbol(*symbol, *model_index).unwrap();
    }

    let encoder_model = encoder_model_builder.build();
    let mut encoder = ANSEncoder::<FIDELITY>::new(encoder_model);
    let mut expected = vec![Vec::new(), Vec::new(), Vec::new(), Vec::new()];

    for (model_index, symbol) in &source {
        expected[*model_index].push(*symbol);
    }

    // now encode each symbol with the corresponding model previously associated
    for (model_index, symbol) in source.iter() {
        encoder.encode(*symbol, *model_index);
    }

    let prelude = encoder.serialize();
    let mut decoder = ANSDecoder::<FIDELITY>::new(prelude);
    let mut decoded: Vec<Vec<RawSymbol>> = vec![Vec::new(), Vec::new(), Vec::new(), Vec::new()];

    source.reverse(); // now let's reverse the order of the model_index-symbol pairs to decode in reverse

    for (model_index, _symbol) in &source {
        decoded[*model_index].push(decoder.decode(*model_index));
    }

    decoded.iter_mut().for_each(|sequence| sequence.reverse()); // they have been decoded in reversed order

    assert_eq!(expected[0], decoded[0]);
    assert_eq!(expected[1], decoded[1]);
    assert_eq!(expected[2], decoded[2]);
    assert_eq!(expected[3], decoded[3]);
}
