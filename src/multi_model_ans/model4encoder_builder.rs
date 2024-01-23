use std::cmp::max;
use anyhow::{bail, Result};


use crate::{LOG2_B, MAX_RAW_SYMBOL, RawSymbol, Symbol};
use crate::bvgraph::BVGraphComponent;
use crate::multi_model_ans::EncoderModelEntry;
use crate::multi_model_ans::model4encoder::ANSModel4Encoder;
use crate::utils::ans_utilities::folding_without_streaming_out;
use crate::utils::data_utilities::{cross_entropy, entropy, try_scale_freqs};


/// Multiplicative factor used to set the maximum cross entropy allowed for the new approximated
/// distribution of frequencies.
/// The bigger this factor is, the more approximated the new distribution will be. It means smaller frame
/// sizes and, consequently, less memory usage + faster encoding/decoding.
const THETA: f64 = 1.001;


pub struct ANSModel4EncoderBuilder {

    frequencies: Vec<Vec<usize>>,

    /// For each
    max_sym: Vec<Symbol>,

    /// The current fidelity used to fold the symbols.
    fidelity: usize,

    /// The current radix used to fold the symbols.
    radix: usize,

    /// Represent the threshold starting from which a symbol has to be folded.
    folding_threshold: RawSymbol,
}

impl ANSModel4EncoderBuilder {

    /// Creates a new AnsModel4EncoderBuilder with the given number of models.
    pub fn new(fidelity: usize, radix: usize) -> Self {
        // we can calculate the biggest folded sym that we can see. This is used to create a vec with already-allocated frequencies
        let presumed_max_bucket: Symbol = folding_without_streaming_out(MAX_RAW_SYMBOL, radix, fidelity);
        let frequencies = vec![ vec![0_usize; presumed_max_bucket as usize]; BVGraphComponent::COMPONENTS];

        Self {
            frequencies,
            max_sym: vec![0; BVGraphComponent::COMPONENTS],
            fidelity,
            radix,
            folding_threshold: (1 << (fidelity + radix - 1)) as u64,
        }
    }

    pub fn push_symbol(&mut self, symbol: RawSymbol, component: BVGraphComponent) -> Result<()> {
        if symbol > MAX_RAW_SYMBOL {
            bail!("Symbol can't be bigger than u48::MAX");
        }

        let folded_sym = match symbol < self.folding_threshold {
            true => symbol as Symbol,
            false => folding_without_streaming_out(symbol, self.radix, self.fidelity),
        };

        // this unwrap is safe since we have already filled the vec with all zeros
        *self.frequencies[component as usize].get_mut(folded_sym as usize).unwrap() += 1;
        self.max_sym[component as usize] = max(self.max_sym[component as usize], folded_sym);

        Ok(())
    }

    pub fn build(self) -> ANSModel4Encoder {
        let mut tables: Vec<Vec<EncoderModelEntry>> = Vec::with_capacity(BVGraphComponent::COMPONENTS);
        let mut frame_sizes = Vec::with_capacity(BVGraphComponent::COMPONENTS);

        for model_index in 0..BVGraphComponent::COMPONENTS {
            let symbols = self.frequencies[model_index].iter().filter(|freq| **freq > 0).count();
            let (approx_freqs, m) = Self::approx_freqs(
                &self.frequencies[model_index],
                symbols,
                self.max_sym[model_index],
            );
            let mut table: Vec<EncoderModelEntry> = Vec::with_capacity(self.max_sym[model_index] as usize + 1);
            let mut last_covered_freq = 0;
            let log_m = m.ilog2() as usize;
            let mut k = 32 - log_m;     // !!! K = 32 - log2(M) !!!

            for freq in approx_freqs.iter() {
                k = if log_m > 0 {k} else {31};

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

        let mut approx_freqs = None;
        let mut last_accepted_frame_size = 0;

        loop {
            if frame_size > (1 << 15) { // we can handle frame sizes bigger than 2^15 cause we want to use u16 for cumul_freqs
                match approx_freqs {
                    // if there is an approximation we didn't accept cause the cross entropy was too high, let's accept it
                    Some(approx_freqs) => {
                        return (approx_freqs, last_accepted_frame_size);
                    },
                    None => {
                        panic!("The distribution of the symbols cannot be satisfactorily approximated with a frame
                        size smaller than 2^16. You may want to change RADIX and/or FIDELITY to make the compressor work.
                        ");
                    }
                }
            }

            match try_scale_freqs(freqs, &sorted_indices, n, total_freq, frame_size as isize) {
                Ok(new_distribution) => {
                    let cross_entropy = cross_entropy(freqs, total_freq as f64, &new_distribution, frame_size as f64);

                    // we are done if the cross entropy of the new distr is lower than the entropy of
                    // the original distribution times a multiplicative factor TETA.
                    if cross_entropy <= entropy * THETA {
                        approx_freqs = Some(new_distribution);
                        break;
                    } else {
                        // try with a bigger frame but keep the current one for now
                        approx_freqs = Some(new_distribution);
                        last_accepted_frame_size = frame_size;
                        frame_size *= 2;
                    }
                },
                Err(_) => {
                    frame_size *= 2;
                }
            }
        }
        let mut approximated_frequencies = approx_freqs.unwrap();
        approximated_frequencies.drain(max_sym as usize + 1..);

        (approximated_frequencies, frame_size)
    }
}