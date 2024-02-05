use anyhow::{bail, Result};
use itertools::Itertools;
use log::info;
use std::cmp::max;
use std::collections::HashMap;
use std::ops::Neg;

use crate::bvgraph::BVGraphComponent;
use crate::multi_model_ans::model4encoder::ANSComponentModel4Encoder;
use crate::multi_model_ans::model4encoder::ANSModel4Encoder;
use crate::multi_model_ans::EncoderModelEntry;
use crate::utils::ans_utilities::fold_without_streaming_out;
use crate::utils::data_utilities::scale_freqs;
use crate::{RawSymbol, Symbol, B, MAX_RAW_SYMBOL};

/// Multiplicative factor used to define the maximum distance that a distribution can have from the
/// original one.
/// The bigger this factor is, the more approximated the new distribution can be. It means smaller
/// frame sizes and, consequently, less memory usage + faster encoding/decoding.
const THETA: [f64; 7] = [1.001, 1.003, 1.005, 1.01, 1.02, 1.05, 1.10];

pub struct ANSModel4EncoderBuilder {
    /// The frequencies of the raw symbols for each component.
    real_freqs: Vec<HashMap<RawSymbol, usize>>,

    /// The sum of all symbols' frequencies for each component.
    total_freqs: Vec<usize>,
}

impl ANSModel4EncoderBuilder {
    /// The maximum frame size allowed for any of the [models](ANSComponentModel4Encoder) used by
    /// the ANS encoder.
    const MAXIMUM_FRAME_SIZE: usize = 1 << 15;

    /// Creates a new instance of the builder that will allow for the creation of a new
    /// [ANSModel4Encoder], one for each [component](BVGraphComponent).
    pub fn new() -> Self {
        Self {
            real_freqs: vec![HashMap::new(); BVGraphComponent::COMPONENTS],
            total_freqs: vec![0; BVGraphComponent::COMPONENTS],
        }
    }

    /// Pushes a new symbol for the given [component](BVGraphComponent) into the builder.
    pub fn push_symbol(&mut self, symbol: RawSymbol, component: BVGraphComponent) -> Result<()> {
        if symbol > MAX_RAW_SYMBOL {
            bail!("Symbol can't be bigger than u48::MAX");
        }

        *self.real_freqs[component as usize]
            .entry(symbol)
            .or_insert(0) += 1;
        self.total_freqs[component as usize] += 1;
        Ok(())
    }

    pub fn build(self) -> ANSModel4Encoder {
        let mut models = Vec::with_capacity(BVGraphComponent::COMPONENTS);

        let original_entropy = self
            .real_freqs
            .iter()
            .map(|freqs| {
                Self::get_information_content(
                    freqs.values().collect::<Vec<_>>(),
                    self.total_freqs[0],
                )
            })
            .collect::<Vec<f64>>();

        info!(
            "Original cost of every component: {:?} B",
            original_entropy
                .iter()
                .map(|x| (x / 8f64).round() as usize)
                .collect::<Vec<usize>>()
        );
        info!(
            "Total cost of every component: {:?} B",
            original_entropy
                .iter()
                .map(|x| x / 8f64)
                .sum::<f64>()
                .round() as usize
        );

        for component in 0..BVGraphComponent::COMPONENTS {
            if self.real_freqs[component].is_empty() {
                // this component has no symbols to encode. it should happen only with dummy graphs
                models.push(ANSComponentModel4Encoder::default());
                continue;
            }

            let mut best_radix = None;
            let mut best_fidelity = None;
            let mut best_distribution = None;
            let mut best_distribution_frame_size = None;
            let mut best_cost = f64::MAX;

            for theta in THETA.iter() {
                let threshold = theta * original_entropy[component];

                'main_loop: for (fidelity, radix) in Self::get_folding_params().iter() {
                    let max_bucket = fold_without_streaming_out(MAX_RAW_SYMBOL, *radix, *fidelity);
                    let mut folded_sym_freqs = vec![0_usize; max_bucket as usize];
                    let folding_threshold = 1u64 << (fidelity + radix - 1);
                    let mut biggest_symbol = 0u16;

                    // create the table containing, for each folded symbol, its frequency.
                    for (raw_symbol, freq) in self.real_freqs[component].iter() {
                        let folded_sym = match *raw_symbol < folding_threshold {
                            true => *raw_symbol as Symbol,
                            false => fold_without_streaming_out(*raw_symbol, *radix, *fidelity),
                        };
                        // update the frequency of the folded symbol
                        folded_sym_freqs[folded_sym as usize] += freq;
                        biggest_symbol = max(biggest_symbol, folded_sym);
                    }

                    let folded_component_cost = Self::get_folded_distr_information_content(
                        &folded_sym_freqs,
                        self.total_freqs[component],
                        *fidelity,
                        *radix,
                    );

                    // stop if the cost of the not-approximated distribution is already higher
                    // than the one we got from one of the previous combination of fidelity and radix.
                    if folded_component_cost > best_cost {
                        continue;
                    }

                    // let's count the number of actual symbols we have in the folded distribution
                    let n = folded_sym_freqs.iter().filter(|freq| **freq > 0).count();

                    let mut frame_size = match n.is_power_of_two() {
                        true => n,
                        false => n.next_power_of_two(),
                    };

                    // We need the list of symbols' indexes sorted by the frequency of the related
                    // symbol, in ascending order.
                    let sorted_indexes = folded_sym_freqs
                        .iter()
                        .enumerate()
                        .filter(|(_, freq)| **freq > 0)
                        .sorted_unstable_by(|(_, freq_1), (_, freq_2)| freq_1.cmp(freq_2))
                        .map(|(symbol_index, _)| symbol_index)
                        .collect::<Vec<usize>>();

                    loop {
                        if frame_size > Self::MAXIMUM_FRAME_SIZE {
                            continue 'main_loop;
                        }

                        match scale_freqs(
                            &folded_sym_freqs,
                            &sorted_indexes,
                            n,
                            self.total_freqs[component],
                            frame_size as isize,
                        ) {
                            Ok(mut new_distribution) => {
                                let new_cost = Self::get_approx_folded_distr_information_content(
                                    &folded_sym_freqs,
                                    &new_distribution,
                                    frame_size as f64,
                                    *fidelity,
                                    *radix,
                                );

                                // accept if either it has a lower cost or has an equal cost but a smaller frame size.
                                if new_cost <= threshold && new_cost < best_cost
                                    || (new_cost == best_cost
                                        && frame_size < best_distribution_frame_size.unwrap())
                                {
                                    best_cost = new_cost;
                                    new_distribution.drain(biggest_symbol as usize + 1..);
                                    best_distribution = Some(new_distribution);
                                    best_distribution_frame_size = Some(frame_size);
                                    best_radix = Some(*radix);
                                    best_fidelity = Some(*fidelity);
                                }
                                frame_size *= 2;
                            }
                            Err(_) => {
                                frame_size *= 2;
                            }
                        }
                    }
                }
            }

            let new_distribution = best_distribution
                .expect("Given the available thresholds, no distribution has been found for one of the components");
            let frame_size = best_distribution_frame_size.unwrap();
            let radix = best_radix.unwrap();
            let fidelity = best_fidelity.unwrap();

            info!(
                "{:?} folded with frame: {}, radix: {} & fidelity: {}. Cost is {:?} B.",
                BVGraphComponent::from(component),
                frame_size,
                radix,
                fidelity,
                (best_cost / 8f64).round() as usize,
            );

            let folding_threshold = 1u64 << (fidelity + radix - 1);
            let folding_offset = ((1 << radix) - 1) * (1 << (fidelity - 1));
            let mut table = Vec::with_capacity(new_distribution.len());
            let mut last_covered_freq = 0;
            let log_m = frame_size.ilog2() as usize;
            let k = if log_m > 0 { 32 - log_m } else { 31 };

            for freq in new_distribution.iter() {
                table.push(EncoderModelEntry {
                    freq: *freq as u16,
                    upperbound: (1_u64 << (k + B)) * *freq as u64,
                    cumul_freq: last_covered_freq,
                });
                last_covered_freq += *freq as u16;
            }

            models.push(ANSComponentModel4Encoder {
                table,
                fidelity,
                radix,
                folding_threshold,
                folding_offset,
                frame_size: log_m,
            });
        }
        ANSModel4Encoder { models: models }
    }

    /// Calculate the information content of an approximated folded distribution, performed with the given fidelity
    /// and radix, by using the original frequencies of the original folded distribution before the approximation.
    fn get_approx_folded_distr_information_content(
        folded_distr: &[usize],
        folded_approximated_distr: &[usize],
        new_frame_size: f64,
        fidelity: usize,
        radix: usize,
    ) -> f64 {
        let folding_threshold = 1u64 << (fidelity + radix - 1);
        let folding_offset = ((1 << radix) - 1) * (1 << (fidelity - 1));
        let mut information_content = 0.0;

        for (symbol, approx_freq) in folded_approximated_distr.iter().enumerate() {
            if *approx_freq == 0 {
                continue;
            }

            let freq = folded_distr[symbol] as f64;
            let folds = match symbol < folding_threshold as usize {
                true => 0_f64,
                false => {
                    ((symbol - folding_threshold as usize) / folding_offset as usize + 1usize)
                        as f64
                }
            };

            let prob = *approx_freq as f64 / new_frame_size;
            information_content += (-prob.log2() + (folds * radix as f64)) * freq;
        }
        information_content
    }

    /// Calculate the information content of a folded distribution that uses the given fidelity and
    /// radix.
    fn get_folded_distr_information_content(
        freqs: &[usize],
        total_freq: usize,
        fidelity: usize,
        radix: usize,
    ) -> f64 {
        let mut information_content = 0.0;
        let folding_threshold = 1u64 << (fidelity + radix - 1);
        let folding_offset = ((1 << radix) - 1) * (1 << (fidelity - 1));

        for (symbol, freq) in freqs.iter().enumerate() {
            if *freq == 0 {
                continue;
            }

            let folds = match symbol < folding_threshold as usize {
                true => 0_f64,
                false => {
                    ((symbol - folding_threshold as usize) / folding_offset as usize + 1usize)
                        as f64
                }
            };

            let freq = *freq as f64;
            let prob = *freq as f64 / total_freq as f64;
            information_content += (-prob.log2() + (folds * radix as f64)) * freq;
        }
        information_content
    }

    fn get_information_content(freqs: &[usize], total_freq: usize) -> f64 {
        freqs
            .iter()
            .map(|freq| {
                let p = *freq as f64 / total_freq as f64;
                f64::log2(p).neg() * *freq as f64
            })
            .sum()
    }

    /// Returns all possibile combinations of radix and fidelity which sum is at least 4 and at most 11.
    pub fn get_folding_params() -> Vec<(usize, usize)> {
        [1usize, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            .iter()
            .combinations_with_replacement(2)
            .map(|v| (*v[0], *v[1]))
            .filter(|(fidelity, radix)|
                // we want to represent explicitly at least 0..7 and at most up to 2^10 - 1
                fidelity + radix <= 11 && fidelity + radix >= 4)
            .collect()
    }
}
