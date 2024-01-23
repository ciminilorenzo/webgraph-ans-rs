use crate::multi_model_ans::model4encoder::SymbolLookup;
use crate::{RawSymbol, State, Symbol};
use crate::bvgraph::BVGraphComponent;
use crate::multi_model_ans::EncoderModelEntry;
use crate::multi_model_ans::DecoderModelEntry;

#[derive(Clone)]
pub struct VecFrame {
    /// Contains the log2 of the frame size for each model.
    frame_sizes: Vec<usize>,

    /// Contains a set of vectors, one each for model. Within each vector, each index contains the data associated to
    /// the symbol equal to that index.
    symbols: Vec<Vec<DecoderModelEntry>>,
}

impl VecFrame {
    const BIT_RESERVED_FOR_SYMBOL: u64 = 48;

    pub fn new(
        tables: &Vec<Vec<EncoderModelEntry>>,
        frame_sizes: &Vec<usize>,
        fidelity: usize,
        radix: usize,
    ) -> Self
    {
        let folding_offset = ((1 << radix) - 1) * (1 << (fidelity - 1));
        let folding_threshold = (1 << (fidelity + radix - 1)) as u64;
        let mut vectors = Vec::with_capacity(tables.len());

        tables.iter().enumerate().for_each(|(model_index, table)| {
            let mut vec = vec![DecoderModelEntry::default(); 1 << frame_sizes[model_index]];
            let mut last_slot = 0; // the last slot of the frame we have actually filled with data

            for (sym, symbol_entry) in table.iter().enumerate() {
                if symbol_entry.freq == 0 {
                    continue; // let's skip symbols with frequency 0
                }

                for slot in last_slot..last_slot + symbol_entry.freq {
                    // fill the symbol's slots with the data
                    *vec.get_mut(slot as usize).unwrap() = DecoderModelEntry {
                        freq: symbol_entry.freq,
                        cumul_freq: symbol_entry.cumul_freq,
                        quasi_folded: Self::quasi_fold(
                            sym as Symbol,
                            folding_offset,
                            folding_threshold,
                            radix,
                        ),
                    };
                }
                last_slot += symbol_entry.freq;
            }
            vectors.push(vec);
        });

        Self {
            frame_sizes: frame_sizes.clone(),
            symbols: vectors,
        }
    }

    fn quasi_fold(sym: Symbol, folding_offset: u64, folding_threshold: u64, radix: usize) -> u64 {
        if sym < folding_threshold as Symbol {
            return sym as u64;
        }

        let mut symbol = sym as u64;
        let folds = (symbol - folding_threshold) / folding_offset + 1_u64;
        let folds_bits = folds << Self::BIT_RESERVED_FOR_SYMBOL;

        symbol -= folding_offset * folds as RawSymbol;
        symbol <<= folds * radix as u64;
        symbol | folds_bits
    }

    #[inline(always)]
    pub fn get_frame_mask(&self, component: BVGraphComponent) -> u64 {
        (1 << self.frame_sizes[component as usize]) - 1
    }

    #[inline(always)]
    pub fn get_log2_frame_size(&self, component: BVGraphComponent) -> usize {
        self.frame_sizes[component as usize]
    }
}

impl SymbolLookup<State> for VecFrame {
    type Output = DecoderModelEntry;

    #[inline(always)]
    fn symbol(&self, slot: State, component: BVGraphComponent) -> &Self::Output {
        &self.symbols[component as usize][slot as usize]
    }
}
