use anyhow::{bail, Result};
use itertools::Itertools;
use log::info;
use std::cmp::max;
use std::collections::HashMap;

use crate::a::model4encoder::ANSComponentModel4Encoder;
use crate::a::model4encoder::ANSModel4Encoder;
use crate::a::EncoderModelEntry;
use crate::bvgraph::BVGraphComponent;
use crate::utils::ans_utilities::fold_without_streaming_out;
use crate::utils::data_utilities::scale_freqs;
use crate::{RawSymbol, Symbol, MAX_RAW_SYMBOL};

/// Multiplicative constant used to fix a maximum increase, in terms of cost, that we can accept
/// when scaling a folded distribution.
///
/// Example: with THETA = 1.0001, if the original cost of the graph is 1000 bytes, we will accept
/// approximated distributions that lead to a cost of at most 1000,1 bytes.
const THETA: f64 = 1.0001;

pub struct ANSModel4EncoderBuilder {
    /// The frequencies of the raw symbols for each component.
    real_freqs: Vec<HashMap<RawSymbol, usize>>,

    /// The sum of all symbols' frequencies for each component.
    total_freqs: Vec<usize>,
}

impl Default for ANSModel4EncoderBuilder {
    /// Creates a new instance of the builder that will allow for the creation of a new
    /// [ANSModel4Encoder], one for each [component](BVGraphComponent).
    fn default() -> Self {
        Self {
            real_freqs: vec![HashMap::new(); BVGraphComponent::COMPONENTS],
            total_freqs: vec![0; BVGraphComponent::COMPONENTS],
        }
    }
}

impl ANSModel4EncoderBuilder {
    /// The maximum frame size allowed for any of the [models](ANSComponentModel4Encoder) used by
    /// the ANS encoder.
    const MAXIMUM_FRAME_SIZE: usize = 1 << 15;

    /// Pushes a new symbol for the given [component](BVGraphComponent) into the builder.
    ///
    /// Note: it returns an error if the pushed symbol is bigger than [MAX_RAW_SYMBOL].
    pub fn push_symbol(&mut self, symbol: RawSymbol, component: BVGraphComponent) -> Result<()> {
        if symbol > MAX_RAW_SYMBOL {
            bail!("Symbol can't be bigger than u48::MAX");
        }

        self.total_freqs[component as usize] += 1;
        *self.real_freqs[component as usize]
            .entry(symbol)
            .or_insert(0) += 1;

        Ok(())
    }

    pub fn build(self) -> ANSModel4Encoder {
        // the original cost of each component.
        let original_comp_costs = self.calculate_cost();
        // the cost of the folded graph, before the scaling process.
        let original_graph_cost = original_comp_costs.iter().sum::<f64>();
        // a vec of ANSComponentModel4Encoder, one for each component
        let mut models = Vec::with_capacity(BVGraphComponent::COMPONENTS);
        // the cost of each folded component, before the scaling process.
        let mut folded_comp_costs = Vec::with_capacity(BVGraphComponent::COMPONENTS);
        // the cost of each folded component, after the scaling process.
        let mut components_final_cost = Vec::with_capacity(BVGraphComponent::COMPONENTS);

        for component in 0..BVGraphComponent::COMPONENTS {
            // if the component has no symbols to encode, we can skip it by filling vars with dummy data
            // since we won't use them.
            if self.real_freqs[component].is_empty() {
                models.push(ANSComponentModel4Encoder::default());
                components_final_cost.push(0.0);
                continue;
            }

            // the cost of the folded distribution for this component, before the scaling process.
            let mut folded_cost = 0.0;
            // the final folded distribution for this component, scaled to sum up to a power of two.
            let mut scaled_distribution = Vec::new();
            let (mut fidelity, mut radix) = (0usize, 0usize);
            let mut frame_size = usize::MAX;
            let mut lowest_cost = f64::MAX;

            let params_combinations = Self::get_folding_params();

            for (fid, rad) in params_combinations.iter() {
                let max_bucket = fold_without_streaming_out(MAX_RAW_SYMBOL, *rad, *fid);
                let folding_threshold = 1u64 << (fid + rad - 1);
                let mut folded_sym_freqs = vec![0_usize; max_bucket as usize];
                let mut biggest_symbol = 0u16;

                // create the table containing, for each folded symbol, its frequency.
                for (raw_symbol, freq) in self.real_freqs[component].iter() {
                    let folded_sym = match *raw_symbol < folding_threshold {
                        true => *raw_symbol as Symbol,
                        false => fold_without_streaming_out(*raw_symbol, *rad, *fid),
                    };
                    // update the frequency of the folded symbol
                    folded_sym_freqs[folded_sym as usize] += freq;
                    biggest_symbol = max(biggest_symbol, folded_sym);
                }

                // let's count the number of actual symbols we have in the folded distribution
                let n = folded_sym_freqs.iter().filter(|freq| **freq > 0).count();

                let mut m = match n.is_power_of_two() {
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
                    if m > Self::MAXIMUM_FRAME_SIZE {
                        // if we have reached the maximum frame size, we can't go further with
                        // the next fidelity and radix combination.
                        break;
                    }

                    let scaled_distribution_attempt = scale_freqs(
                        &folded_sym_freqs,
                        &sorted_indexes,
                        n,
                        self.total_freqs[component],
                        m as isize,
                    );

                    match scaled_distribution_attempt {
                        Ok(mut new_distribution) => {
                            let new_cost = Self::calculate_approx_folded_distribution_cost(
                                &folded_sym_freqs,
                                &new_distribution,
                                m as f64,
                                *fid,
                                *rad,
                            );

                            let difference = new_cost - original_comp_costs[component];
                            let ratio = (original_graph_cost + difference) / original_graph_cost;

                            // if accepting this distribution would make the graph just THETA times
                            // bigger than the original graph and the frame size is smaller than the
                            // current best frame size, we can accept it.
                            if ratio <= THETA {
                                if m < frame_size {
                                    // the cost associated to this folded distribution with current fidelity and radix,
                                    // not approximated yet.
                                    folded_cost = Self::calculate_folded_distribution_cost(
                                        &folded_sym_freqs,
                                        self.total_freqs[component],
                                        *fid,
                                        *rad,
                                    );

                                    lowest_cost = new_cost;
                                    new_distribution.drain(biggest_symbol as usize + 1..);
                                    scaled_distribution = new_distribution;
                                    frame_size = m;
                                    fidelity = *fid;
                                    radix = *rad;
                                }
                            } else if m == Self::MAXIMUM_FRAME_SIZE {
                                // if we reach the maximum frame size and the cost is higher
                                // than the current lowest cost, it means that we previously
                                // found the best distribution with a smaller frame size.
                                if new_cost >= lowest_cost {
                                    break;
                                }

                                // we reach this point only when we have not been able to find a scaled
                                // distribution that we could have accepted. This should happen only
                                // with components with which the folding process don't work properly,
                                // such as Residuals.
                                folded_cost = Self::calculate_folded_distribution_cost(
                                    &folded_sym_freqs,
                                    self.total_freqs[component],
                                    *fid,
                                    *rad,
                                );

                                lowest_cost = new_cost;
                                new_distribution.drain(biggest_symbol as usize + 1..);
                                scaled_distribution = new_distribution;
                                frame_size = m;
                                fidelity = *fid;
                                radix = *rad;

                                break;
                            }
                            m *= 2;
                        }
                        Err(_) => {
                            m *= 2;
                        }
                    }
                }
            }

            components_final_cost.push(lowest_cost);
            folded_comp_costs.push(folded_cost);

            let mut table = Vec::with_capacity(scaled_distribution.len());
            let log_m = frame_size.ilog2() as usize;
            let k = if log_m > 0 { 32 - log_m } else { 31 };

            let mut last_covered_freq = 0;

            for freq in scaled_distribution.iter() {
                table.push(EncoderModelEntry::new(
                    *freq as u16,
                    k,
                    last_covered_freq,
                    log_m,
                ));
                last_covered_freq += *freq as u16;
            }

            models.push(ANSComponentModel4Encoder {
                table,
                fidelity,
                radix,
                folding_threshold: 1u64 << (fidelity + radix - 1),
                folding_offset: ((1 << radix) - 1) * (1 << (fidelity - 1)),
                frame_size: log_m,
            });
        }

        info!(
            "{:<15} | {:<5} | {} & {:<2} | {:<12} | {:<14}",
            "Component", "frame", "R", "F", "Of total(%)", "Cost(bytes)"
        );

        // the cost of the graph before the scaling process.
        let _folded_graph_cost = folded_comp_costs.iter().sum::<f64>();
        let final_graph_cost = components_final_cost.iter().sum::<f64>();

        for component in 0..BVGraphComponent::COMPONENTS {
            info!(
                "{} | {:<5} | {}   {:<2} | {:<12.2} | {:.3}(+{:.2}%)",
                BVGraphComponent::from(component),
                models[component].frame_size,
                models[component].radix,
                models[component].fidelity,
                // how much big this component is w.r.t the final folded graph
                (components_final_cost[component] / final_graph_cost) * 100.0,
                // the final cost of the component
                (components_final_cost[component] / 8f64).round() as usize,
                // the difference in percentage between the final cost and the original cost
                ((components_final_cost[component] - original_comp_costs[component])
                    / original_comp_costs[component])
                    * 100.0,
            );
        }

        info!(
            "Final graph's final self-information before ANS-based encoding is {:?} B (+{:.2}%)\n",
            (final_graph_cost / 8f64).round() as usize,
            ((final_graph_cost - original_graph_cost) / final_graph_cost) * 100.0
        );

        ANSModel4Encoder {
            component_models: models,
        }
    }

    /// Calculates the __original__ cost of every component, calculated as the sum, for every symbol
    /// in the sequence to encode, of its self-information times its frequency.
    fn calculate_cost(&self) -> Vec<f64> {
        self.real_freqs
            .iter()
            .enumerate()
            .map(|(component, freqs)| {
                freqs
                    .values()
                    .map(|freq| {
                        let prob = *freq as f64 / self.total_freqs[component] as f64;
                        -prob.log2() * *freq as f64
                    })
                    .sum::<f64>()
            })
            .collect::<Vec<f64>>()
    }

    /// Given the folded distribution (folded with the given radix & fidelity) and the new approximated
    /// one, calculates the minimum cost we have to pay to encode the sequence by using the new probabilities
    /// coming from the new distribution but the original frequencies from the original folded one.
    ///
    /// NB: for each folded symbol we consider as cost its self-information plus the bits we have to
    /// dump in the state to fold the symbol, times its frequency.
    fn calculate_approx_folded_distribution_cost(
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

    /// Given a folded distribution (with the given radix & fidelity), calculates minimum cost we have
    /// to pay to encode the sequence of symbols, that is the self-information of every symbol times
    /// its frequency, plus the bits we have to dump in the state to fold the symbol.
    fn calculate_folded_distribution_cost(
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
            let prob = freq / total_freq as f64;
            information_content += (-prob.log2() + (folds * radix as f64)) * freq;
        }
        information_content
    }

    /// Returns all possibile combinations of radix and fidelity which sum is at least 4 and at most
    /// 11.
    /// These values are chosen since we want to explicitly represent at least the numbers in the
    /// interval [0; 8) and at most the numbers in the interval [0; 1024).
    pub fn get_folding_params() -> Vec<(usize, usize)> {
        [1usize, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            .iter()
            .combinations_with_replacement(2)
            .map(|v| (*v[0], *v[1]))
            .filter(|(fidelity, radix)| fidelity + radix <= 11 && fidelity + radix >= 4)
            .collect()
    }
}
