use std::ops::Index;

use crate::{LOG2_B, K_LOG2, Symbol, RawSymbol};
use crate::ans::ans_util::*;
use crate::ans::EncoderModelEntry;

// TODO: to change
/// The maximum symbol we expect to see in the input.
const MAX_RAW_SYMBOL: RawSymbol = 100_000_000_000;


#[readonly::make]
#[derive(Debug, Clone)]
pub struct FoldedANSModel4Encoder {

    /// Contains, for each index, the data associated to the symbol equal to that index.
    #[readonly]
    pub table: Vec<EncoderModelEntry>,

    #[readonly]
    pub log2_frame_size: u8,
}

impl FoldedANSModel4Encoder {

    pub fn new(input: &[RawSymbol], radix: u8, fidelity: u8) -> Self {
        let presumed_max_bucket = fold_symbol(MAX_RAW_SYMBOL, false, None, radix, fidelity);
        let mut frequencies = vec![0; presumed_max_bucket as usize];
        let mut max_sym = 0;
        let mut n = 0;

        for sym in input {
            let folded_sym = fold_symbol(*sym, false, None, radix, fidelity);
            *frequencies.get_mut(folded_sym as usize).unwrap() += 1;
            max_sym = std::cmp::max(max_sym, folded_sym);
            n += 1;
        }

        let (approx_freqs, m) = approx_freqs(&frequencies, n, max_sym);
        let mut table: Vec<EncoderModelEntry> = Vec::with_capacity(max_sym as usize + 1);
        let mut last_covered_freq = 0;

        for freq in approx_freqs.iter() {
            table.push(EncoderModelEntry {
                freq: *freq as u32,
                upperbound: ((1 << (K_LOG2 + LOG2_B)) * *freq) as u64,
                cumul_freq: last_covered_freq,
            });
            last_covered_freq += *freq as u32;
        }

        Self {
            log2_frame_size: m.ilog2() as u8,
            table,
        }
    }

    pub fn to_raw_parts(&self) -> Vec<EncoderModelEntry> {
        self.table.clone()
    }
}

impl Index<Symbol> for FoldedANSModel4Encoder {
    type Output = EncoderModelEntry;

    fn index(&self, symbol: Symbol) -> &Self::Output {
        &self.table[symbol as usize]
    }
}