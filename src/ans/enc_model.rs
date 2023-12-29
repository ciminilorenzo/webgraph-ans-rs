use std::cmp::max;
use std::ops::Index;

use crate::ans::EncoderModelEntry;
use crate::utils::{cross_entropy, entropy};
use crate::{RawSymbol, Symbol, K_LOG2, LOG2_B, MAX_RAW_SYMBOL};

use anyhow::{bail, Result};

use strength_reduce::StrengthReducedU64;


/// Multiplicative factor used to set the maximum cross entropy allowed for the new approximated
/// distribution of frequencies.
/// The bigger this factor is, the more approximated the new distribution will be. It means smaller frame
/// sizes and, consequently, less memory usage + faster encoding/decoding.
const TETA: f64 = 1.001;

#[readonly::make]
#[derive(Clone)]
pub struct FoldedANSModel4Encoder {
    /// Contains, for each index, the data associated to the symbol equal to that index.
    #[readonly]
    pub table: Vec<EncoderModelEntry>,

    #[readonly]
    pub log2_frame_size: usize,
}

impl FoldedANSModel4Encoder {
    pub fn new(input: &[RawSymbol], radix: usize, fidelity: usize) -> Self {
        let presumed_max_bucket = Self::folding_without_streaming_out(MAX_RAW_SYMBOL, radix, fidelity);
        let mut frequencies = vec![0; presumed_max_bucket as usize];
        let mut max_sym = 0;
        let folding_threshold = (1 << (fidelity + radix - 1)) as RawSymbol;

        for sym in input {
            let folded_sym = if *sym < folding_threshold {
                *sym as Symbol
            } else {
                Self::folding_without_streaming_out(*sym, radix, fidelity)
            };

            *frequencies.get_mut(folded_sym as usize).expect("Symbols from input must be at most 2^48 - 1") += 1;
            max_sym = max(max_sym, folded_sym);
        }

        let symbols_len = frequencies.iter().filter(|freq| **freq > 0).count();

        let (approx_freqs, m) = Self::approx_freqs(&frequencies, symbols_len, max_sym);
        let mut table: Vec<EncoderModelEntry> = Vec::with_capacity(max_sym as usize + 1);
        let mut last_covered_freq = 0;

        for freq in approx_freqs.iter() {
            let reciprocal = if *freq > 0 {
                StrengthReducedU64::new(*freq as u64)
            } else {
                StrengthReducedU64::new(1)
            };

            table.push(EncoderModelEntry {
                freq: *freq as u16,
                upperbound: ((1 << (K_LOG2 + LOG2_B)) * *freq) as u64,
                cumul_freq: last_covered_freq,
                reciprocal,
            });
            last_covered_freq += *freq as u16;
        }

        Self {
            log2_frame_size: m.ilog2() as _,
            table,
        }
    }

    fn folding_without_streaming_out(mut sym: RawSymbol, radix: usize, fidelity: usize) -> Symbol {
        let mut offset = 0;
        let cuts = (((u64::ilog2(sym) as usize) + 1) - fidelity) / radix;
        let bit_to_cut = cuts * radix;
        sym >>= bit_to_cut;
        offset += (((1 << radix) - 1) * (1 << (fidelity - 1))) * cuts as RawSymbol;

        u16::try_from(sym + offset).expect("Folded symbol is bigger than u16::MAX")
    }

    fn approx_freqs(freqs: &[usize], n: usize, max_sym: Symbol) -> (Vec<usize>, usize) {
        let mut total_freq = 0;
        let mut indexed_freqs: Vec<(usize, usize)> = Vec::with_capacity(freqs.len());

        for (index, freq) in freqs.iter().enumerate() {
            if *freq == 0 {
                continue;
            }

            total_freq += freq;
            indexed_freqs.push((*freq, index));
        }

        indexed_freqs.shrink_to_fit();
        let mut frame_size = if n.is_power_of_two() {
            n
        } else {
            n.next_power_of_two()
        };
        let mut approx_freqs: Vec<usize>;

        let entropy = entropy(
            &indexed_freqs
                .iter()
                .map(|(freq, _)| *freq)
                .collect::<Vec<usize>>(),
            total_freq as f64,
        );

        let sorted_indices = {
            let mut sorted_indexed_freqs = indexed_freqs.clone();
            sorted_indexed_freqs.sort_unstable_by(|a, b| a.0.cmp(&b.0));
            sorted_indexed_freqs
                .iter()
                .map(|(_, index)| *index)
                .collect::<Vec<usize>>()
        };

        loop {
            assert!(frame_size <= (1 << 28), "frame_size must be at most 2^28");

            let scaling_result = Self::try_scale_freqs(freqs, &sorted_indices, n, total_freq, frame_size as isize);

            match scaling_result {
                Ok(new_freqs) => {
                    let cross_entropy =
                        cross_entropy(freqs, total_freq as f64, &new_freqs, frame_size as f64);

                    // we are done if the cross entropy of the new distr is lower than the entropy of
                    // the original distribution times a multiplicative factor TETA.
                    if cross_entropy <= entropy * TETA {
                        approx_freqs = new_freqs;
                        break;
                    } else {
                        // else try with a bigger frame size
                        frame_size *= 2;
                    }
                }
                Err(_) => {
                    frame_size *= 2;
                }
            }
        }
        approx_freqs.drain(max_sym as usize + 1..);
        (approx_freqs, frame_size)
    }

    /// Tries to scale frequencies in `freqs` by using the new common denominator `new_frame`. This algorithm
    /// gives priority to low frequency symbols in order to be sure that the extra space the new frame size
    /// is supplying is firstly given to symbols with approximated frequency lower than 0.
    ///
    /// # Returns
    /// The approximated frequencies if is possibile to approximate with the given `new_frame` else, if too
    /// many symbols have frequency lower than 1 - meaning that M is not big enough to handle the whole
    /// set of symbols - an error is returned.
    pub fn try_scale_freqs(
        freqs: &[usize],
        sorted_indices: &[usize],
        n: usize,
        mut total_freq: usize,
        mut new_frame: isize,
    ) -> Result<Vec<usize>> {
        let mut approx_freqs = freqs.to_vec();
        let ratio = new_frame as f64 / total_freq as f64;

        let get_approx_freq = |scale: f64, sym_freq: f64| -> usize {
                let new_freq = max(1, (0.5 + scale * sym_freq).floor() as usize);

                if new_freq > ((1 << 16) - 1) {
                    panic!("Cannot have frequencies bigger than 2^16 - 1. Freq is {}", new_freq);
                }
                new_freq
            };

        for (index, sym_index) in sorted_indices.iter().enumerate() {
            let sym_freq = freqs[*sym_index];
            let second_ratio = new_frame as f64 / total_freq as f64;
            let scale =
                (n - index) as f64 * ratio / n as f64 + index as f64 * second_ratio / n as f64;

            approx_freqs[*sym_index] = get_approx_freq(scale, sym_freq as f64);
            new_frame -= approx_freqs[*sym_index] as isize;
            total_freq -= sym_freq;

            if new_frame < 0 {
                bail!("Cannot approximate frequencies with this new frame size!");
            }
        }
        Ok(approx_freqs)
    }

    pub fn to_raw_parts(&self) -> Vec<EncoderModelEntry> {
        self.table.clone()
    }
}

impl Index<Symbol> for FoldedANSModel4Encoder {
    type Output = EncoderModelEntry;

    #[inline(always)]
    fn index(&self, symbol: Symbol) -> &Self::Output {
        &self.table[symbol as usize]
    }
}
