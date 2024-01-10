use std::cmp::max;
use anyhow::bail;

use strength_reduce::StrengthReducedU64;

use crate::{LOG2_B, MAX_RAW_SYMBOL, RawSymbol, Symbol};
use crate::ans::enc_model::AnsModel4Encoder;
use crate::ans::EncoderModelEntry;
use crate::utils::{cross_entropy, entropy};


/// Multiplicative factor used to set the maximum cross entropy allowed for the new approximated
/// distribution of frequencies.
/// The bigger this factor is, the more approximated the new distribution will be. It means smaller frame
/// sizes and, consequently, less memory usage + faster encoding/decoding.
const THETA: f64 = 1.001;


pub struct AnsModel4EncoderBuilder <const FIDELITY: usize, const RADIX: usize> {
    models: usize,
    frequencies: Vec<Vec<usize>>,
    max_sym: Vec<Symbol>,
}

impl <const FIDELITY: usize, const RADIX: usize> AnsModel4EncoderBuilder <FIDELITY, RADIX> {
    const FOLDING_THRESHOLD: RawSymbol = (1 << (FIDELITY + RADIX - 1)) as RawSymbol;

    /// Creates a new AnsModel4EncoderBuilder with the given number of models.
    pub fn new(models: usize) -> Self {
        let presumed_max_bucket: Symbol = Self::folding_without_streaming_out(MAX_RAW_SYMBOL, RADIX, FIDELITY);
        let frequencies = vec![ vec![0_usize; presumed_max_bucket as usize]; models];

        Self {
            models,
            frequencies,
            max_sym: vec![0; models],
        }
    }

    pub fn push_symbol(&mut self, symbol: RawSymbol, model_index: usize) -> anyhow::Result<()> {
        if symbol > MAX_RAW_SYMBOL {
            bail!("Symbol can't be bigger than u48::MAX");
        }

        let folded_sym = if symbol < Self::FOLDING_THRESHOLD {
            symbol as Symbol
        } else {
            Self::folding_without_streaming_out(symbol, RADIX, FIDELITY)
        };

        unsafe { // it can be unsafe since we are sure that there is a frequency in that position
            *self.frequencies[model_index].get_unchecked_mut(folded_sym as usize) += 1
        };
        self.max_sym[model_index] = max(self.max_sym[model_index], folded_sym);

        Ok(())
    }

    pub fn build(self) -> AnsModel4Encoder {
        let mut tables: Vec<Vec<EncoderModelEntry>> = Vec::with_capacity(self.models);
        let mut frame_sizes = Vec::with_capacity(self.models);

        for model_index in 0..self.models {
            let symbols_len = self.frequencies[model_index].iter().filter(|freq| **freq > 0).count();
            let (approx_freqs, m) = Self::approx_freqs(
                &self.frequencies[model_index],
                symbols_len,
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
                    fast_divisor,
                });
                last_covered_freq += *freq as u16;
            }

            tables.push(table);
            frame_sizes.push(log_m);
        }

        AnsModel4Encoder {
            tables,
            frame_sizes,
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
    ) -> anyhow::Result<Vec<usize>> {
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
}