use std::cmp::max;
use anyhow::{bail, Result};

use strength_reduce::StrengthReducedU64;

use crate::{LOG2_B, MAX_RAW_SYMBOL, RawSymbol, Symbol};
use crate::multi_model_ans::EncoderModelEntry;
use crate::multi_model_ans::model4encoder::ANSModel4Encoder;
use crate::utils::ans_utilities::folding_without_streaming_out;
use crate::utils::data_utilities::{cross_entropy, entropy, try_scale_freqs};


/// Multiplicative factor used to set the maximum cross entropy allowed for the new approximated
/// distribution of frequencies.
/// The bigger this factor is, the more approximated the new distribution will be. It means smaller frame
/// sizes and, consequently, less memory usage + faster encoding/decoding.
const THETA: f64 = 1.001;


pub struct ANSModel4EncoderBuilder<const FIDELITY: usize, const RADIX: usize> {
    models: usize,
    frequencies: Vec<Vec<usize>>,
    max_sym: Vec<Symbol>,
}

impl <const FIDELITY: usize, const RADIX: usize> ANSModel4EncoderBuilder<FIDELITY, RADIX> {
    const FOLDING_THRESHOLD: RawSymbol = (1 << (FIDELITY + RADIX - 1)) as RawSymbol;

    /// Creates a new AnsModel4EncoderBuilder with the given number of models.
    pub fn new(models: usize) -> Self {
        // we can calculate the biggest folded sym that we can see. This is used to create a vec with already-allocated frequencies
        let presumed_max_bucket: Symbol = folding_without_streaming_out(MAX_RAW_SYMBOL, RADIX, FIDELITY);
        let frequencies = vec![ vec![0_usize; presumed_max_bucket as usize]; models];

        Self {
            models,
            frequencies,
            max_sym: vec![0; models],
        }
    }

    pub fn push_symbol(&mut self, symbol: RawSymbol, model_index: usize) -> Result<()> {
        if symbol > MAX_RAW_SYMBOL {
            bail!("Symbol can't be bigger than u48::MAX");
        }

        let folded_sym = match symbol < Self::FOLDING_THRESHOLD {
            true => symbol as Symbol,
            false => folding_without_streaming_out(symbol, RADIX, FIDELITY),
        };

        // this unwrap is safe since we have already filled the vec with all zeros
        *self.frequencies[model_index].get_mut(folded_sym as usize).unwrap() += 1;
        self.max_sym[model_index] = max(self.max_sym[model_index], folded_sym);

        Ok(())
    }

    pub fn build(self) -> ANSModel4Encoder {
        let mut tables: Vec<Vec<EncoderModelEntry>> = Vec::with_capacity(self.models);
        let mut frame_sizes = Vec::with_capacity(self.models);

        for model_index in 0..self.models {
            let symbols = self.frequencies[model_index].iter().filter(|freq| **freq > 0).count();
            let (approx_freqs, m) = Self::approx_freqs(
                &self.frequencies[model_index],
                symbols,
                self.max_sym[model_index],
            );
            let mut table: Vec<EncoderModelEntry> = Vec::with_capacity(self.max_sym[model_index] as usize + 1);
            let mut last_covered_freq = 0;
            let log_m = m.ilog2() as usize;
            let k = 32 - log_m;     // !!! K = 32 - log2(M) !!!

            for freq in approx_freqs.iter() {
                let fast_divisor = if *freq > 0 {
                    StrengthReducedU64::new(*freq as u64)
                } else {
                    StrengthReducedU64::new(1)
                };

                table.push(EncoderModelEntry {
                    freq: *freq as u16,
                    upperbound: (1_u64 << (k + LOG2_B)) * *freq as u64,
                    cumul_freq: last_covered_freq,
                });
                last_covered_freq += *freq as u16;
            }
            tables.push(table);
            frame_sizes.push(log_m);
        }

        ANSModel4Encoder {
            tables,
            frame_sizes,
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

        let mut frame_size = match n.is_power_of_two() {
            true => n,
            false => n.next_power_of_two(),
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
            assert!(frame_size <= (1 << 32), "The left extreme of the interval must be 2^32.");

            let scaling_result = try_scale_freqs(freqs, &sorted_indices, n, total_freq, frame_size as isize);

            match scaling_result {
                Ok(new_freqs) => {
                    let cross_entropy = cross_entropy(freqs, total_freq as f64, &new_freqs, frame_size as f64);

                    // we are done if the cross entropy of the new distr is lower than the entropy of
                    // the original distribution times a multiplicative factor TETA.
                    if cross_entropy <= entropy * THETA {
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
}