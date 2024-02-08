use anyhow::{bail, Result};
use itertools::Itertools;
use log::info;
use std::collections::HashMap;

use crate::bvgraph::BVGraphComponent;
use crate::multi_model_ans::model4encoder::ANSComponentModel4Encoder;
use crate::multi_model_ans::model4encoder::ANSModel4Encoder;
use crate::multi_model_ans::EncoderModelEntry;
use crate::utils::ans_utilities::fold_without_streaming_out;
use crate::utils::data_utilities::scale_freqs;
use crate::{RawSymbol, Symbol, B, MAX_RAW_SYMBOL};

pub struct ANSModel4EncoderBuilder {
    /// The frequencies of the raw symbols for each component.
    real_freqs: Vec<HashMap<RawSymbol, usize>>,

    /// The sum of all symbols' frequencies for each component.
    total_freqs: Vec<usize>,

    /// The cost of encoding the __original__ distribution of each component, expressed as the expected number of bits
    /// we have to spend.
    component_costs: Vec<f64>,

    /// The cost of the whole graph, calculated by summing the cost of each component.
    graph_cost: f64,

    /// The cost of the whole folded graph, that is the sum of the costs we have to spend to encode each folded
    /// distribution.
    folded_graph_cost: f64,
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
            component_costs: vec![0.0; BVGraphComponent::COMPONENTS],
            graph_cost: 0.0,
            folded_graph_cost: 0.0,
        }
    }

    /// Pushes a new symbol for the given [component](BVGraphComponent) into the builder.
    ///
    /// Note: it panics if the symbol is bigger than [MAX_RAW_SYMBOL].
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

    pub fn build(mut self) -> ANSModel4Encoder {
        info!(
            "{:<15} | {:<10} | {:<10} | {:<10} | {:<12} | {:<10}",
            "Component", "log(frame)", "radix", "fidelity", "Cost(bytes)", "Cost difference(%)",
        );

        let mut models = Vec::with_capacity(BVGraphComponent::COMPONENTS);

        let original_comp_costs = self.calculate_cost();
        let graph_cost: f64 = original_comp_costs.iter().sum(); // todo: do we want to store it inside the model?

        let mut folded_graph_cost = 0.0;
        let mut folded_comp_costs = Vec::with_capacity(BVGraphComponent::COMPONENTS);
        let mut folded_distributions = Vec::with_capacity(BVGraphComponent::COMPONENTS);
        let mut params = Vec::with_capacity(BVGraphComponent::COMPONENTS);

        let params_combinations = Self::get_folding_params();

        for component in 0..BVGraphComponent::COMPONENTS {
            // if the component has no symbols to encode, we can skip it by filling vars with dummy data.
            if self.total_freqs[component] == 0 {
                folded_comp_costs.push(0.0);
                folded_distributions.push(vec![]);
                params.push((0, 0));
                continue;
            }

            let mut lower_cost = f64::MAX;
            let mut best_params = (0, 0);
            let mut best_distr = vec![];

            // figures out which is the best combination of parameters to start from for this component
            for (fidelity, radix) in params_combinations.iter() {
                let max_bucket = fold_without_streaming_out(MAX_RAW_SYMBOL, *radix, *fidelity);
                let folding_threshold = 1u64 << (fidelity + radix - 1);
                let mut folded_sym_freqs = vec![0_usize; max_bucket as usize];

                // create the table containing, for each folded symbol, its frequency.
                for (raw_symbol, freq) in self.real_freqs[component].iter() {
                    let folded_sym = match *raw_symbol < folding_threshold {
                        true => *raw_symbol as Symbol,
                        false => fold_without_streaming_out(*raw_symbol, *radix, *fidelity),
                    };
                    folded_sym_freqs[folded_sym as usize] += freq;
                }

                let cost = Self::calculate_folded_distribution_cost(
                    &folded_sym_freqs,
                    self.total_freqs[component],
                    *fidelity,
                    *radix,
                );

                if cost < lower_cost {
                    lower_cost = cost;
                    best_params = (*fidelity, *radix);
                    best_distr = folded_sym_freqs;
                }
            }
            folded_comp_costs.push(lower_cost); // mi sto salvando costo componente foldato NON APPROSSIMATO
            folded_distributions.push(best_distr);
            params.push(best_params);
        }
        folded_graph_cost = folded_comp_costs.iter().sum::<f64>();

        let mut folded_approx_graph_cost = 0.0;
        let mut folded_approx_comp_cost = Vec::with_capacity(BVGraphComponent::COMPONENTS);

        for (component, folded_distribution) in folded_distributions.iter().enumerate() {
            if self.total_freqs[component] == 0 {
                // this component has no symbols to encode, thus fill with dummy data.
                models.push(ANSComponentModel4Encoder::default());
                folded_approx_comp_cost.push(0.0);
                continue;
            }

            let (fidelity, radix) = params[component];
            let folding_threshold = 1u64 << (fidelity + radix - 1);
            let folding_offset = ((1 << radix) - 1) * (1 << (fidelity - 1));

            // let's count the number of actual symbols we have in the folded distribution
            let n = folded_distribution.iter().filter(|freq| **freq > 0).count();

            let max_sym = folded_distribution
                .iter()
                .enumerate()
                .map(|(symbol, _)| symbol)
                .max()
                .unwrap();

            let mut m = match n.is_power_of_two() {
                true => n,
                false => n.next_power_of_two(),
            };

            // We need the list of symbols' indexes sorted by the frequency of the related
            // symbol, in ascending order.
            let sorted_indexes = folded_distribution
                .iter()
                .enumerate()
                .filter(|(_, freq)| **freq > 0)
                .sorted_unstable_by(|(_, freq_1), (_, freq_2)| freq_1.cmp(freq_2))
                .map(|(symbol_index, _)| symbol_index)
                .collect::<Vec<usize>>();

            // data related to the final approximated folded distribution
            let mut approx_distribution = vec![];
            let mut frame_size = 0;
            let mut cost = 0.0;

            loop {
                let scaling_attempt = scale_freqs(
                    folded_distribution,
                    &sorted_indexes,
                    n,
                    self.total_freqs[component],
                    m as isize,
                );

                match scaling_attempt {
                    Ok(mut new_distribution) => {
                        let new_cost = Self::get_approx_folded_distribution_cost(
                            folded_distribution,
                            &new_distribution,
                            m as f64,
                            params[component].0,
                            params[component].1,
                        );

                        let difference = new_cost - folded_comp_costs[component];
                        let ratio = (difference / folded_graph_cost) * 100.0;

                        if m == Self::MAXIMUM_FRAME_SIZE {
                            folded_approx_comp_cost.push(new_cost);
                            new_distribution.drain(max_sym + 1..);
                            approx_distribution = new_distribution;
                            frame_size = m;
                            cost = new_cost;
                            break;
                        }

                        if ratio < 0.01 {
                            // todo: is this threshold what we want?
                            folded_approx_comp_cost.push(new_cost);
                            new_distribution.drain(max_sym + 1..);
                            approx_distribution = new_distribution;
                            frame_size = m;
                            cost = new_cost;
                            break;
                        }
                        m *= 2;
                    }
                    Err(_) => {
                        m *= 2;
                    }
                }
            }

            folded_approx_graph_cost += cost;
            let log_m = frame_size.ilog2() as usize;
            let k = if log_m > 0 { 32 - log_m } else { 31 };
            let mut table = Vec::with_capacity(approx_distribution.len());
            let mut last_covered_freq = 0;

            for freq in approx_distribution.iter() {
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

            info!(
                "{} | {:<10} | {:<10} | {:<10} | {:<12} | {:<18.2}",
                BVGraphComponent::from(component),
                log_m,
                radix,
                fidelity,
                // cost in bytes of the folded approximated distribution for this component
                (cost / 8f64).round() as usize,
                // how much is increased the cost of the approximated folded distr w.r.t the original cost
                (cost - original_comp_costs[component]) / original_comp_costs[component] * 100.0,
            );
        }

        info!(
            "Original graph cost: {:?} B | Folded graph cost: {} B (+{:.2}%)\n",
            (graph_cost / 8f64).round() as usize,
            (folded_graph_cost / 8f64).round() as usize,
            ((folded_graph_cost - graph_cost) / graph_cost) * 100.0
        );

        ANSModel4Encoder {
            component_models: models,
        }
    }

    /// Calculates the __original__ cost of every component.
    fn calculate_cost(&mut self) -> Vec<f64> {
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

    /// Calculate the information content of an approximated folded distribution, performed with the given fidelity
    /// and radix, by using the original frequencies of the original folded distribution before the approximation.
    fn get_approx_folded_distribution_cost(
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

    /// Calculate the cost of a folded distribution that uses the given fidelity and
    /// radix.
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
