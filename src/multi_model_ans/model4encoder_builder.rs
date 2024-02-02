use anyhow::{bail, Result};
use itertools::Itertools;
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
const THETA: [f64; 6] = [1.001, 1.003, 1.005, 1.01, 1.02, 1.05];

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
        println!("\n\nStarting building the model");
        let mut models: Vec<ANSComponentModel4Encoder> =
            Vec::with_capacity(BVGraphComponent::COMPONENTS);

        // todo: delete this stuff
        // let's calculate the self-information of each component
        let self_information = self
            .real_freqs
            .iter()
            .enumerate()
            .map(|(component, freqs)| {
                let mut self_information = 0.0;

                for freq in freqs.values() {
                    self_information +=
                        *freq as f64 * f64::log2(self.total_freqs[component] as f64 / *freq as f64);
                }
                self_information
            })
            .collect::<Vec<f64>>();

        let original_entropy = self
            .real_freqs
            .iter()
            .enumerate()
            .map(|(component_index, freqs)| {
                freqs
                    .values()
                    .map(|freq| {
                        let prob = *freq as f64 / self.total_freqs[component_index] as f64;
                        f64::log2(prob).neg() * *freq as f64
                    })
                    .sum::<f64>()
            })
            .collect::<Vec<f64>>();

        println!(
            "Original entropy cost of every component: {:?}",
            original_entropy
        );
        println!(
            "Sum of all entropy costs: {:?}",
            original_entropy.iter().sum::<f64>()
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
            let mut lower_divergence = f64::MAX;
            let mut best_entropy_cost = 0.0;

            for theta in THETA.iter() {
                'main_loop: for (fidelity, radix) in Self::get_folding_params().iter() {
                    // the biggest bucket that we can see given the current fidelity and radix.
                    let max_bucket = fold_without_streaming_out(MAX_RAW_SYMBOL, *radix, *fidelity);
                    let mut folded_sym_freqs = vec![0_usize; max_bucket as usize];
                    let folding_threshold = 1u64 << (fidelity + radix - 1);
                    let folding_offset = ((1 << radix) - 1) * (1 << (fidelity - 1));
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

                    /*
                    println!("Approximated with fidelity {} & radix {}. Entropy cost of original folded distribution is: {}",
                        fidelity,
                        radix,
                        Self::calculate_entropic_cost(
                            &folded_sym_freqs,
                            self.total_freqs[component],
                            *fidelity,
                            *radix,
                        )
                    );
                    */

                    let divergence = self.calculate_divergence(
                        self.total_freqs[component] as f64,
                        &folded_sym_freqs,
                        self.total_freqs[component] as f64,
                        component,
                        folding_threshold,
                        *fidelity,
                        *radix,
                    );

                    // stop if the divergence of the not-approximated distribution is already higher
                    // than the one we got from one of the previous combination of fidelity and radix.
                    if divergence > lower_divergence {
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
                                let difference = Self::calculate_difference(
                                    &folded_sym_freqs,
                                    &new_distribution,
                                    frame_size as f64,
                                    *fidelity,
                                    *radix,
                                );

                                /*
                                println!("Folded distribution approximated with frame size: {} | entropy is now: {}",
                                    frame_size,
                                    difference
                                );
                                */

                                let divergence = self.calculate_divergence(
                                    self.total_freqs[component] as f64,
                                    &new_distribution,
                                    frame_size as f64,
                                    component,
                                    folding_threshold,
                                    *fidelity,
                                    *radix,
                                );

                                if divergence <= self_information[component] * theta {
                                    // let's accept the current distribution if either it has a lower
                                    // divergence or has an equal divergence but a smaller frame size.
                                    if divergence < lower_divergence
                                        || (divergence == lower_divergence
                                            && frame_size < best_distribution_frame_size.unwrap())
                                    {
                                        lower_divergence = divergence;
                                        new_distribution.drain(biggest_symbol as usize + 1..);
                                        best_distribution = Some(new_distribution);
                                        best_distribution_frame_size = Some(frame_size);
                                        best_radix = Some(*radix);
                                        best_fidelity = Some(*fidelity);
                                        best_entropy_cost = Self::calculate_entropic_cost(
                                            &folded_sym_freqs,
                                            self.total_freqs[component],
                                            *fidelity,
                                            *radix,
                                        );
                                    }
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

            let new_distribution = best_distribution.unwrap();
            let frame_size = best_distribution_frame_size.unwrap();
            let radix = best_radix.unwrap();
            let fidelity = best_fidelity.unwrap();
            let folding_threshold = 1u64 << (fidelity + radix - 1);
            let folding_offset = ((1 << radix) - 1) * (1 << (fidelity - 1));
            let mut table: Vec<EncoderModelEntry> = Vec::with_capacity(new_distribution.len());
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
        ANSModel4Encoder { tables: models }
    }

    fn calculate_difference(
        folded_distr: &[usize], // the folded distribution BEFORE approximation
        folded_approximated_distr: &[usize], // the folded distribution AFTER approximation
        frame_size: f64,        // the new frame size AFTEER approximation
        fidelity: usize,
        radix: usize,
    ) -> f64 {
        let folding_threshold = 1u64 << (fidelity + radix - 1);
        let folding_offset = ((1 << radix) - 1) * (1 << (fidelity - 1));
        let mut entropy = 0.0;

        folded_approximated_distr
            .iter()
            .enumerate()
            .for_each(|(symbol, freq)| {
                if *freq > 0 {
                    let bytes_to_unfold = match symbol < folding_threshold as usize {
                        true => 0_usize,
                        false => {
                            (symbol - folding_threshold as usize) / folding_offset as usize + 1usize
                        }
                    };

                    let p = *freq as f64 / frame_size;
                    entropy += (-p.log2() + (bytes_to_unfold as f64 * radix as f64))
                        * folded_distr[symbol] as f64;
                }
            });
        entropy
    }

    fn calculate_entropic_cost(
        freqs: &[usize],
        total_freq: usize,
        fidelity: usize,
        radix: usize,
    ) -> f64 {
        let mut entropic_cost = 0.0;
        let folding_threshold = 1u64 << (fidelity + radix - 1);
        let folding_offset = ((1 << radix) - 1) * (1 << (fidelity - 1));

        for (symbol, freq) in freqs.iter().enumerate() {
            if *freq > 0 {
                let bytes_to_unfold = match symbol < folding_threshold as usize {
                    true => 0_usize,
                    false => {
                        (symbol - folding_threshold as usize) / folding_offset as usize + 1usize
                    }
                };

                let p = *freq as f64 / total_freq as f64;
                entropic_cost +=
                    (f64::log2(p).neg() + (bytes_to_unfold as f64 * radix as f64)) * *freq as f64;
            }
        }
        entropic_cost
    }

    fn calculate_divergence(
        &self,
        _total_freq: f64,
        other_distr: &[usize],
        new_total_freq: f64,
        component_index: usize,
        folding_threshold: u64,
        fidelity: usize,
        radix: usize,
    ) -> f64 {
        let mut divergence = 0.0;

        for (raw_symbol, freq) in self.real_freqs[component_index].iter() {
            let folded_sym: Symbol;
            let mut folds = 0;

            match *raw_symbol < folding_threshold {
                true => folded_sym = *raw_symbol as Symbol,
                false => {
                    folded_sym = fold_without_streaming_out(*raw_symbol, radix, fidelity);
                    folds = (((u64::ilog2(*raw_symbol) as usize) + 1) - fidelity) / radix;
                }
            }

            divergence += *freq as f64
                * f64::log2(new_total_freq / other_distr[folded_sym as usize] as f64)
                + folds as f64 * radix as f64;
        }
        divergence
    }

    /// Returns all possibile combinations of radix and fidelity which sum is at least 4 and at most 11.
    ///
    /// Note: the pairs are returns in descending order of the sum of radix and fidelity.
    pub fn get_folding_params() -> Vec<(usize, usize)> {
        [1usize, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            .iter()
            .combinations_with_replacement(2)
            .map(|v| (*v[0], *v[1]))
            .filter(|(fidelity, radix)|
                // we want to represent explicitly at least 0..7 and at most up to 2^10 - 1
                fidelity + radix <= 11 && fidelity + radix >= 4)
            .sorted_by(|(fid_1, rad_1), (fid_2, rad_2)| (fid_1 + rad_1).cmp(&(fid_2 + rad_2)))
            .rev()
            .collect()
    }
}
