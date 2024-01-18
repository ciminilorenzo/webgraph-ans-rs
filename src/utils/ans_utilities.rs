use crate::{RawSymbol, Symbol};
use crate::bvgraph::mock_writers::EntropyMockWriter;
use crate::multi_model_ans::EncoderModelEntry;

/// Folds a symbol without streaming out the bits.
pub fn folding_without_streaming_out(mut sym: RawSymbol, radix: usize, fidelity: usize) -> Symbol {
    let mut offset = 0;
    let cuts = (((u64::ilog2(sym) as usize) + 1) - fidelity) / radix;
    let bit_to_cut = cuts * radix;
    sym >>= bit_to_cut;
    offset += (((1 << radix) - 1) * (1 << (fidelity - 1))) * cuts as RawSymbol;

    u16::try_from(sym + offset).expect("Folded symbol is bigger than u16::MAX")
}

pub fn get_mock_writer(tables: &Vec<Vec<EncoderModelEntry>>, frame_sizes: &Vec<usize>) -> EntropyMockWriter {
    let mut table = vec![Vec::new(); 9];

    tables
        .iter()
        .enumerate()
        .for_each(|(model_index, current_table)| {
            current_table
                .iter()
                .for_each(|symbol| {
                    let freq = tables[model_index][symbol.freq as usize].freq;
                    let probability = freq as f64 / frame_sizes[model_index] as f64;
                    let inverse = 1.0 / probability;
                    let shifted = (inverse * ((1 << 16) as f64)).round() as usize;
                    table[model_index].push(shifted);
                })
        });

    EntropyMockWriter::new(table)
}