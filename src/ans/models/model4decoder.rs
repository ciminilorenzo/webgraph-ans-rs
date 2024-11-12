use crate::ans::models::component_model4decoder::{ANSComponentModel4Decoder, DecoderModelEntry};
use crate::ans::models::component_model4encoder::ANSComponentModel4Encoder;
use crate::bvgraph::BVGraphComponent;
use crate::{RawSymbol, Symbol};

/// The container for the whole set of models, one for each [component](BVGraphComponent) used by the ANS decoder to
/// decode symbols.
pub struct ANSModel4Decoder {

    /// A table containing the whole set of [models](ANSComponentModel4Decoder) used by the ANS decoder, one for each
    /// [component](BVGraphComponent).
    pub tables: Vec<ANSComponentModel4Decoder>,
}

impl ANSModel4Decoder {
    const BIT_RESERVED_FOR_SYMBOL: u64 = 48;

    pub fn new(tables: &[ANSComponentModel4Encoder]) -> Self {
        let mut vectors = Vec::with_capacity(tables.len());

        tables.iter().for_each(|table| {
            let mut vec = vec![DecoderModelEntry::default(); 1 << table.frame_size];
            let mut last_slot: u32 = 0; // the last slot of the frame we have actually filled with data

            for (sym, symbol_entry) in table.table.iter().enumerate() {
                if symbol_entry.freq == 0 {
                    continue; // let's skip symbols with frequency 0
                }

                for slot in last_slot..last_slot + symbol_entry.freq as u32 {
                    // fill the symbol's slots with the data
                    *vec.get_mut(slot as usize).unwrap() = DecoderModelEntry {
                        freq: symbol_entry.freq,
                        cumul_freq: symbol_entry.cumul_freq,
                        quasi_folded: Self::quasi_fold(
                            sym as Symbol,
                            table.folding_offset,
                            table.folding_threshold,
                            table.radix,
                        ),
                    };
                }
                last_slot += symbol_entry.freq as u32;
            }
            vectors.push(ANSComponentModel4Decoder::new(
                vec,
                table.frame_size,
                table.radix,
                table.fidelity,
            ))
        });

        Self { tables: vectors }
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
    pub fn symbol(&self, slot: Symbol, component: BVGraphComponent) -> &DecoderModelEntry {
        &self.tables[component as usize][slot]
    }

    #[inline(always)]
    pub fn get_frame_mask(&self, component: BVGraphComponent) -> u64 {
        (1 << self.tables[component as usize].frame_size) - 1
    }

    #[inline(always)]
    pub fn get_log2_frame_size(&self, component: BVGraphComponent) -> usize {
        self.tables[component as usize].frame_size
    }

    #[inline(always)]
    pub fn get_radix(&self, component: BVGraphComponent) -> usize {
        self.tables[component as usize].radix
    }

    #[inline(always)]
    pub fn get_fidelity(&self, component: BVGraphComponent) -> usize {
        self.tables[component as usize].fidelity
    }
}
