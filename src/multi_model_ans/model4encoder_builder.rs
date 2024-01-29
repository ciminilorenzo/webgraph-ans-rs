use std::cmp::max;
use anyhow::{bail, Result};

use crate::{B, MAX_RAW_SYMBOL, RawSymbol, Symbol};
use crate::bvgraph::BVGraphComponent;
use crate::multi_model_ans::EncoderModelEntry;
use crate::multi_model_ans::model4encoder::ANSModel4Encoder;
use crate::utils::ans_utilities::fold_without_streaming_out;
use crate::utils::data_utilities::{cross_entropy, entropy, try_scale_freqs};
use crate::multi_model_ans::model4encoder::ANSComponentModel4Encoder;


/// Multiplicative factor used to set the maximum cross entropy allowed for the new approximated distribution of
/// frequencies.
/// The bigger this factor is, the more approximated the new distribution will be. It means smaller frame
/// sizes and, consequently, less memory usage + faster encoding/decoding.
const THETA: f64 = 1.001;

pub struct ANSModel4EncoderBuilder {
    /// A temporary container used to store the frequencies of the symbols of each [component](BVGraphComponent).
    frequencies: Vec<Vec<usize>>,

    /// The fidelity and radix values used by each [component](BVGraphComponent).
    component_args: [(usize, usize); 9],

    /// Represent the threshold starting from which a symbol has to be folded, one for each [component](BVGraphComponent).
    folding_thresholds: Vec<RawSymbol>,

    folding_offsets: Vec<RawSymbol>,

    /// The maximum symbol seen for each [component](BVGraphComponent).
    max_sym: Vec<Symbol>,
}

impl ANSModel4EncoderBuilder {
    /// The maximum frame size allowed for any of the [models](ANSComponentModel4Encoder) used by the ANS encoder.
    const MAXIMUM_FRAME_SIZE : usize = 1 << 15;

    /// Creates a new instance of the builder that will allow for the creation of a new [ANSModel4Encoder], one for each
    /// [component](BVGraphComponent).
    pub fn new(component_args: [(usize, usize); 9]) -> Self {
        let mut max_folded_sym = Vec::with_capacity(BVGraphComponent::COMPONENTS);
        let mut frequencies = Vec::with_capacity(BVGraphComponent::COMPONENTS);
        let mut folding_thresholds = Vec::with_capacity(BVGraphComponent::COMPONENTS);
        let mut folding_offsets = Vec::with_capacity(BVGraphComponent::COMPONENTS);

        component_args
            .iter()
            .for_each(|(fidelity, radix)| {
                let max_folded_bucket = fold_without_streaming_out(MAX_RAW_SYMBOL, *radix, *fidelity);
                max_folded_sym.push(max_folded_bucket);
                frequencies.push(vec![0_usize; max_folded_bucket as usize]);
                folding_thresholds.push((1 << (fidelity + radix - 1)) as u64);
                folding_offsets.push(((1u64 << radix) - 1) * (1 << (fidelity - 1)));
            });

        Self {
            frequencies,
            component_args,
            folding_thresholds,
            folding_offsets,
            max_sym: vec![0; BVGraphComponent::COMPONENTS],
        }
    }

    /// Pushes a new symbol for the given [component](BVGraphComponent) into the builder.
    pub fn push_symbol(&mut self, symbol: RawSymbol, component: BVGraphComponent) -> Result<()> {
        if symbol > MAX_RAW_SYMBOL {
            bail!("Symbol can't be bigger than u48::MAX");
        }

        let folded_sym = match symbol < self.folding_thresholds[component as usize] {
            true => symbol as Symbol,
            false => fold_without_streaming_out(
                symbol,
                self.component_args[component as usize].1,
                self.component_args[component as usize].0,
            ),
        };

        self.frequencies[component as usize][folded_sym as usize] += 1;
        self.max_sym[component as usize] = max(self.max_sym[component as usize], folded_sym);

        Ok(())
    }

    pub fn build(self) -> ANSModel4Encoder {
        let mut tables: Vec<ANSComponentModel4Encoder> = Vec::with_capacity(BVGraphComponent::COMPONENTS);

        for model_index in 0..BVGraphComponent::COMPONENTS {
            let symbols = self.frequencies[model_index].iter().filter(|freq| **freq > 0).count();
            let (approx_freqs, m) = Self::approx_freqs(
                &self.frequencies[model_index],
                symbols,
                self.max_sym[model_index],
            );

            println!("Biggest symbol for component {:?}, is: {}",
                BVGraphComponent::from(model_index),
                self.max_sym[model_index]
            );

            let mut table: Vec<EncoderModelEntry> = Vec::with_capacity(self.max_sym[model_index] as usize + 1);
            let mut last_covered_freq = 0;
            let log_m = m.ilog2() as usize;

            // fixes when log_m is 0, that is when no symbols are present. Keeping 0 would panic during upperbound calculation
            let k =  if log_m > 0 {32 - log_m} else {31};

            for freq in approx_freqs.iter() {
                table.push(EncoderModelEntry {
                    freq: *freq as u16,
                    upperbound: (1_u64 << (k + B)) * *freq as u64,
                    cumul_freq: last_covered_freq,
                });
                last_covered_freq += *freq as u16;
            }

            tables.push(ANSComponentModel4Encoder {
                table,
                fidelity: self.component_args[model_index].0,
                radix: self.component_args[model_index].1,
                folding_threshold: self.folding_thresholds[model_index],
                folding_offset: self.folding_offsets[model_index],
                frame_size: log_m,
            });
        }
        ANSModel4Encoder { tables }
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
            if frame_size > Self::MAXIMUM_FRAME_SIZE {
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

                    // we are done if the cross entropy of the new distribution is lower than the entropy of
                    // the original distribution times theta.
                    if cross_entropy <= entropy * THETA {
                        approx_freqs = Some(new_distribution);
                        break;
                    } else {
                        // else let's try with a bigger frame but keep the current one for now.. it may be good for the future
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