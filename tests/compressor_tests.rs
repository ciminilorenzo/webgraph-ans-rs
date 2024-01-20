mod utils;

use utils::*;

use rand::prelude::{IteratorRandom, SliceRandom};
use folded_streaming_rans::RawSymbol;
use folded_streaming_rans::multi_model_ans::decoder::ANSDecoder;
use folded_streaming_rans::multi_model_ans::encoder::ANSEncoder;
use folded_streaming_rans::multi_model_ans::model4decoder::VecFrame;
use folded_streaming_rans::multi_model_ans::model4encoder_builder::ANSModel4EncoderBuilder;

#[test]
fn decoder_decodes_correctly_single_dummy_sequence() {
    let source = vec![1_u64, 1, 1, 2, 2, 2, 3, 3, 4, 5];
    let mut model4encoder_builder = ANSModel4EncoderBuilder::<FIDELITY, FASTER_RADIX>::new(1);

    for symbol in &source {
        model4encoder_builder.push_symbol(*symbol, 0).unwrap();
    }

    let encoder_model = model4encoder_builder.build();
    let mut encoder = ANSEncoder::<FIDELITY>::new(encoder_model); // if not specified is always 8

    for symbol in &source {
        encoder.encode(*symbol, 0);
    }

    let prelude = encoder.serialize();
    let model = VecFrame::<FASTER_RADIX, u64>::new(
        &prelude.tables.clone(),
        &prelude.frame_sizes.clone(),
        get_folding_offset(FASTER_RADIX, FIDELITY),
        get_folding_threshold(FASTER_RADIX, FIDELITY),
    );

    let mut decoder = ANSDecoder::<FIDELITY>::new(&prelude, &model);
    let mut decoded_symbols: Vec<RawSymbol> = Vec::new();

    for _ in 0..source.len() {
        decoded_symbols.push(decoder.decode(0));
    }
    decoded_symbols.reverse(); // since encodes as a LIFO

    assert_eq!(decoded_symbols, source);
}

#[test]
fn decoder_decodes_correctly_dummy_sequence_with_folding() {
    let source = vec![1000, 1000, 2000];

    let mut model4encoder_builder = ANSModel4EncoderBuilder::<FIDELITY, 4>::new(1);

    for symbol in &source {
        model4encoder_builder.push_symbol(*symbol, 0).unwrap();
    }

    let encoder_model = model4encoder_builder.build();
    let mut encoder = ANSEncoder::<FIDELITY, 4>::new(encoder_model); // if not specified is always 8

    for symbol in &source {
        encoder.encode(*symbol, 0);
    }

    let prelude = encoder.serialize();
    let model = VecFrame::<4, u64>::new(
        &prelude.tables.clone(),
        &prelude.frame_sizes.clone(),
        get_folding_offset(4, FIDELITY),
        get_folding_threshold(4, FIDELITY),
    );

    let mut decoder = ANSDecoder::<FIDELITY, 4, u64, VecFrame<4, u64>>::new(&prelude, &model);
    let mut decoded_symbols: Vec<RawSymbol> = Vec::new();

    for _ in 0..source.len() {
        decoded_symbols.push(decoder.decode(0));
    }
    decoded_symbols.reverse(); // since encodes as a LIFO

    assert_eq!(decoded_symbols, source);
}

#[test]
fn decoder_decodes_correctly_real_sequence() {
    let source = get_zipfian_distr(0, 1.2).to_vec();

    let mut model4encoder_builder = ANSModel4EncoderBuilder::<FIDELITY, 6>::new(1);

    for symbol in &source {
        model4encoder_builder.push_symbol(*symbol, 0).unwrap();
    }

    let encoder_model = model4encoder_builder.build();
    let mut encoder = ANSEncoder::<FIDELITY, 6>::new(encoder_model);

    for symbol in &source {
        encoder.encode(*symbol, 0);
    }

    let prelude = encoder.serialize();
    let model = VecFrame::<6, u64>::new(
        &prelude.tables.clone(),
        &prelude.frame_sizes.clone(),
        get_folding_offset(6, FIDELITY),
        get_folding_threshold(6, FIDELITY),
    );

    let mut decoder = ANSDecoder::<FIDELITY, 6, u64, VecFrame<6, u64>>::new(&prelude, &model);
    let mut decoded_symbols: Vec<RawSymbol> = Vec::new();

    for _ in 0..source.len() {
        decoded_symbols.push(decoder.decode(0));
    }
    decoded_symbols.reverse(); // since encodes as a LIFO

    assert_eq!(decoded_symbols, source);
}

/*
#[test]
fn decoder_decodes_correctly_a_single_dummy_sequence() {
    let source = vec![1_u64, 1, 1, 2, 2, 2, 3, 3, 4, 5];
    let mut encoder_model_builder = ANSModel4EncoderBuilder::<FIDELITY, FASTER_RADIX>::new(1);

    for symbol in &source {
        encoder_model_builder.push_symbol(*symbol, 0).unwrap(); // first traversal to build the statistics
    }

    let encoder_model = encoder_model_builder.build();
    let mut encoder = ANSEncoder::<FIDELITY>::new(encoder_model);

    for symbol in &source {
        encoder.encode(*symbol, 0); // second traversal to encode the symbols
    }

    let prelude = encoder.serialize();
    let mut decoder = ANSDecoder::<FIDELITY>::new(&prelude);
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
    let mut encoder_model_builder = ANSModel4EncoderBuilder::<FIDELITY, FASTER_RADIX>::new(2);

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
    let mut decoder = ANSDecoder::<FIDELITY>::new(&prelude);

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
    let mut encoder_model_builder = ANSModel4EncoderBuilder::<FIDELITY, FASTER_RADIX>::new(2);

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
    let mut decoder = ANSDecoder::<FIDELITY>::new(&prelude);
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

    let mut encoder_model_builder = ANSModel4EncoderBuilder::<FIDELITY, FASTER_RADIX>::new(4);

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
    let mut decoder = ANSDecoder::<FIDELITY>::new(&prelude);
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
// Frame sizes: [9, 14, 13, 10] (note that these are actually log_2 of the frame sizes)
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

    let mut encoder_model_builder = ANSModel4EncoderBuilder::<FIDELITY, FASTER_RADIX>::new(4);

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
    let mut decoder = ANSDecoder::<FIDELITY>::new(&prelude);
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
fn test_random_access() {
    // let's get a random sequence of symbols to encode and map them to have this shape: (model_index, symbol)
    let first_sequence = get_zipfian_distr(0, 1.3)
        .iter()
        .map(|symbol| (0, *symbol)).collect::<Vec<(usize, RawSymbol)>>();

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
        .collect::<Vec<(usize, RawSymbol)>>();

    // now let's unify each source in a single one and randomize it
    let mut source = vec![first_sequence, second_sequence, third_sequence, fourth_sequence].concat();
    source.shuffle(&mut rand::thread_rng());

    let mut encoder_model_builder = ANSModel4EncoderBuilder::<FIDELITY, FASTER_RADIX>::new(4);

    for (model_index, symbol) in &source {
        encoder_model_builder.push_symbol(*symbol, *model_index).unwrap();
    }

    let encoder_model = encoder_model_builder.build();
    let mut encoder = ANSEncoder::<FIDELITY>::new(encoder_model);

    // let's take 100 random indexes of symbols that will lately want to decode
    let random_symbols_indexes = (0..source.len()).into_iter().choose_multiple(&mut rand::thread_rng(), 100);
    let mut phases = Vec::new();
    let mut expected = Vec::new();

    for (model_index, symbol) in &source {
        encoder.encode(*symbol, *model_index);

        if random_symbols_indexes.contains(&model_index) {
            phases.push(encoder.get_current_compressor_phase()); // save the phase of the symbol at index i
            expected.push(*symbol); // save the symbol at index i
        }
    }

    let prelude = encoder.serialize();
    let mut decoder = ANSDecoder::<FIDELITY>::new(&prelude);

    for phase_index in 0..phases.len() {
        let phase = phases[phase_index].clone();
        assert_eq!(decoder.decode_from_phase(phase, 0), expected[phase_index]);
    }
}

#[test]
fn test_random_access_with_bitvec() {
    let sequence = get_zipfian_distr(0, 1.2);
    let mut encoder_model_builder = ANSModel4EncoderBuilder::<FIDELITY, 5>::new(1);

    for symbol in &sequence {
        encoder_model_builder.push_symbol(*symbol, 0).unwrap();
    }

    let encoder_model = encoder_model_builder.build();
    let mut encoder = ANSEncoder::<FIDELITY, 5, BitVec<usize, Msb0>>::with_parameters(encoder_model, BitVec::new());

    // let's take 100 random indexes of symbols that will lately want to decode
    let random_symbols_indexes = (0..sequence.len()).into_iter().choose_multiple(&mut rand::thread_rng(), 100);

    let mut phases = Vec::new();
    let mut expected = Vec::new();

    for index in 0..sequence.len() {
        encoder.encode(sequence[index], 0);

        if random_symbols_indexes.contains(&index) {
            phases.push(encoder.get_current_compressor_phase()); // save the phase of the symbol at index i
            expected.push(sequence[index]); // save the symbol at index i
        }
    }

    let prelude = encoder.serialize();

    let frame = VecFrame::<5, u64>::new(
        &prelude.tables.clone(),
        &prelude.frame_sizes.clone(),
        get_folding_offset(5, FIDELITY),
        get_folding_threshold(5, FIDELITY),
    );

    let mut decoder = ANSDecoder::<
        FIDELITY,
        5,
        u64,
        VecFrame<5, u64>,
        BitVec<usize, Msb0>
    >::with_parameters(&prelude, frame);

    for phase_index in 0..phases.len() {
        let phase = phases[phase_index].clone();
        assert_eq!(decoder.decode_from_phase(phase, 0), expected[phase_index]);
    }
}
*/
