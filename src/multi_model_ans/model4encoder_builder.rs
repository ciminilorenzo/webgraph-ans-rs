use anyhow::{bail, Result};
use itertools::Itertools;
use log::info;
use std::cmp::max;
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

    /// The cost, expressed as the expected number of bits we have to spend to encode the whole sequence, of the original
    /// distribution of each component.
    component_costs: Vec<f64>,

    /// The cost of the whole graph, calculated by summing the cost of each component.
    graph_cost: f64,
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
            "{:<15} | {:<10} | {:<10} | {:<10} | {:<12} | {:<10} | {:<10}",
            "Component",
            "log(frame)",
            "radix",
            "fidelity",
            "Cost(bytes)",
            "Cost difference(%)",
            "Of total(%)"
        );

        self.calculate_cost();
        let mut models = Vec::with_capacity(BVGraphComponent::COMPONENTS);
        let mut folded_component_costs = Vec::with_capacity(BVGraphComponent::COMPONENTS);

        for component in 0..BVGraphComponent::COMPONENTS {
            if self.real_freqs[component].is_empty() {
                // this component has no symbols to encode. it should happen only with dummy graphs
                models.push(ANSComponentModel4Encoder::default());
                folded_component_costs.push(0.0);
                continue;
            }

            let mut radix = 0;
            let mut fidelity = 0;
            let mut approximated_distr = Vec::new();
            let mut frame_size = 0;
            let mut best_cost = f64::MAX;

            'main_loop: for (fid, rad) in Self::get_folding_params().iter() {
                let max_bucket = fold_without_streaming_out(MAX_RAW_SYMBOL, *rad, *fid);
                let mut folded_sym_freqs = vec![0_usize; max_bucket as usize];
                let folding_threshold = 1u64 << (fid + rad - 1);
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

                let folded_component_cost = Self::get_folded_distribution_cost(
                    &folded_sym_freqs,
                    self.total_freqs[component],
                    *fid,
                    *rad,
                );

                // we would just spend even more
                if folded_component_cost > best_cost {
                    continue;
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
                        continue 'main_loop;
                    }

                    match scale_freqs(
                        &folded_sym_freqs,
                        &sorted_indexes,
                        n,
                        self.total_freqs[component],
                        m as isize,
                    ) {
                        Ok(mut new_distribution) => {
                            let new_cost = Self::get_approx_folded_distribution_cost(
                                &folded_sym_freqs,
                                &new_distribution,
                                m as f64,
                                *fid,
                                *rad,
                            );

                            // accept if either it has a lower cost or has an equal cost but a smaller frame size.
                            if new_cost < best_cost || (new_cost == best_cost && m < frame_size) {
                                best_cost = new_cost;
                                new_distribution.drain(biggest_symbol as usize + 1..);
                                approximated_distr = new_distribution;
                                frame_size = m;
                                radix = *rad;
                                fidelity = *fid;
                            }
                            m *= 2;
                        }
                        Err(_) => {
                            m *= 2;
                        }
                    }
                }
            }

            folded_component_costs.push(best_cost);
            let folding_threshold = 1u64 << (fidelity + radix - 1);
            let folding_offset = ((1 << radix) - 1) * (1 << (fidelity - 1));
            let mut table = Vec::with_capacity(approximated_distr.len());
            let mut last_covered_freq = 0;
            let log_m = frame_size.ilog2() as usize;
            let k = if log_m > 0 { 32 - log_m } else { 31 };

            for freq in approximated_distr.iter() {
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

        let folded_graph_cost = folded_component_costs.iter().sum::<f64>();
        for component in 0..BVGraphComponent::COMPONENTS {
            info!(
                "{} | {:<10} | {:<10} | {:<10} | {:<12} | {:<18.2} | {:<10.2}",
                BVGraphComponent::from(component),
                models[component].frame_size,
                models[component].radix,
                models[component].fidelity,
                (folded_component_costs[component] / 8f64).round() as usize,
                (folded_component_costs[component] - self.component_costs[component])
                    / self.component_costs[component]
                    * 100.0,
                (folded_component_costs[component] / folded_graph_cost) * 100.0
            );
        }

        info!(
            "Original graph cost: {:?} B | Folded graph cost: {} B (+{:.2}%)\n",
            (self.graph_cost / 8f64).round() as usize,
            (folded_graph_cost / 8f64).round() as usize,
            (folded_graph_cost - self.graph_cost) / self.graph_cost * 100.0
        );

        ANSModel4Encoder { models }
    }

    /// Stores in `Self` the cost of every component and the total cost of the graph.
    fn calculate_cost(&mut self) {
        self.component_costs = self
            .real_freqs
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
            .collect::<Vec<f64>>();

        self.graph_cost = self.component_costs.iter().sum();
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

    /// Calculate the information content of a folded distribution that uses the given fidelity and
    /// radix.
    fn get_folded_distribution_cost(
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
