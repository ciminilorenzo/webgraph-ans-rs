use crate::{FASTER_RADIX, RawSymbol, Symbol};
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

pub fn get_mock_writer(tables: &Vec<Vec<EncoderModelEntry>>, frame_sizes: &Vec<usize>, fidelity: usize)
                       -> EntropyMockWriter
{
    let mut table = vec![Vec::new(); 9];
    let folding_threshold = 1usize << (fidelity + FASTER_RADIX - 1);
    let folding_offset = (1usize << (fidelity - 1)) * ((1 << FASTER_RADIX) - 1);

    tables
        .iter()
        .enumerate()
        .for_each(|(model_index, current_table)| {
            current_table
                .iter()
                .enumerate()
                .for_each(|(symbol, symbol_entry)| {
                    let bytes_to_unfold = match symbol < folding_threshold {
                        true => 0_usize,
                        false => {
                            (symbol - folding_threshold) / folding_offset + 1_usize
                        }
                    };

                    let freq = tables[model_index][symbol].freq;
                    let probability = freq as f64 / frame_sizes[model_index] as f64;
                    let inverse = 1.0 / probability;
                    let shifted = (inverse * ((1 << 16) as f64)).round() as usize;

                    let final_prob = shifted + ((bytes_to_unfold * 8) * (1 << 16));
                    table[model_index].push(final_prob);
                })
        });

    EntropyMockWriter::new(table)
}