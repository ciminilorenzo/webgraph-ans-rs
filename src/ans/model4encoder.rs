use std::cmp::max;
use std::ops::Index;

use crate::ans::{EncoderModelEntry, K_LOG2};
use crate::{RawSymbol, Symbol, B, MAX_RAW_SYMBOL};
use crate::utils::data_utilities::{cross_entropy, entropy, scale_freqs};

use strength_reduce::StrengthReducedU64;
use crate::utils::ans_utilities::fold_without_streaming_out;


/// Multiplicative factor used to set the maximum cross entropy allowed for the new approximated
/// distribution of frequencies.
/// The bigger this factor is, the more approximated the new distribution will be. It means smaller frame
/// sizes and, consequently, less memory usage + faster encoding/decoding.
const TETA: f64 = 1.001;


#[derive(Clone)]
pub struct SingleANSModel4Encoder {
    /// Contains, for each index, the data associated to the symbol equal to that index.
    pub table: Vec<EncoderModelEntry>,

    pub log2_frame_size: usize,
}

impl SingleANSModel4Encoder {
    pub fn new(input: &[RawSymbol], radix: usize, fidelity: usize) -> Self {
        let presumed_max_bucket = fold_without_streaming_out(MAX_RAW_SYMBOL, radix, fidelity);
        let mut frequencies = vec![0; presumed_max_bucket as usize];
        let mut max_sym = 0;
        let folding_threshold = (1 << (fidelity + radix - 1)) as RawSymbol;

        for sym in input {
            let folded_sym = if *sym < folding_threshold {
                *sym as Symbol
            } else {
                fold_without_streaming_out(*sym, radix, fidelity)
            };

            *frequencies.get_mut(folded_sym as usize).expect("Symbols from input must be at most 2^48 - 1") += 1;
            max_sym = max(max_sym, folded_sym);
        }

        let symbols_len = frequencies.iter().filter(|freq| **freq > 0).count();

        let (approx_freqs, m) = Self::approx_freqs(&frequencies, symbols_len, max_sym);
        let mut table: Vec<EncoderModelEntry> = Vec::with_capacity(max_sym as usize + 1);
        let mut last_covered_freq = 0;

        for freq in approx_freqs.iter() {
            table.push(EncoderModelEntry {
                freq: *freq as u16,
                upperbound: ((1 << (K_LOG2 + B)) * *freq) as u64,
                cumul_freq: last_covered_freq,
                fast_divisor: match *freq > 0 {
                    true => StrengthReducedU64::new(*freq as u64),
                    false => StrengthReducedU64::new(1),
                },
            });
            last_covered_freq += *freq as u16;
        }

        Self {
            log2_frame_size: m.ilog2() as _,
            table,
        }
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

            let scaling_result = scale_freqs(freqs, &sorted_indices, n, total_freq, frame_size as isize);

            match scaling_result {
                Ok(new_freqs) => {
                    let cross_entropy = cross_entropy(freqs, total_freq as f64, &new_freqs, frame_size as f64);

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

    pub fn to_raw_parts(&self) -> Vec<EncoderModelEntry> {
        self.table.clone()
    }
}

impl Index<Symbol> for SingleANSModel4Encoder {
    type Output = EncoderModelEntry;

    #[inline(always)]
    fn index(&self, symbol: Symbol) -> &Self::Output {
        &self.table[symbol as usize]
    }
}