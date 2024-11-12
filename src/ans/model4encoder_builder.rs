use anyhow::{bail, Result};

use itertools::Itertools;

use log::info;

use std::cmp::max;
use std::collections::HashMap;

use crate::ans::models::component_model4encoder::{ANSComponentModel4Encoder, EncoderModelEntry};
use crate::ans::models::model4encoder::ANSModel4Encoder;
use crate::bvgraph::BVGraphComponent;
use crate::utils::ans_utils::fold_without_streaming_out;
use crate::utils::data_utils::scale_freqs;
use crate::{RawSymbol, Symbol, MAX_RAW_SYMBOL};

/// Multiplicative constant used to fix a maximum increase, in terms of cost, that we can accept
/// when scaling a folded distribution.
///
/// Example: with THETA = 1.0001, if the original cost of the graph is 1000 bytes, we will accept
/// approximated distributions that lead to a cost of at most 1000,1 bytes.
const THETA: f64 = 1.0001;

/// All possible combinations of radix and fidelity which sum is at least 4 and at most 11.
/// These values are chosen since we want to explicitly represent at least the numbers in the
/// interval [0; 8) and at most the numbers in the interval [0; 1024).
const PARAMS_COMBINATIONS: Vec<(usize, usize)> = vec![
    (1, 3), (2, 2), (3, 1),
    (1, 4), (2, 3), (3, 2), (4, 1),
    (1, 5), (2, 4), (3, 3), (4, 2), (5, 1),
    (1, 6), (2, 5), (3, 4), (4, 3), (5, 2), (6, 1),
    (1, 7), (2, 6), (3, 5), (4, 4), (5, 3), (6, 2), (7, 1),
    (1, 8), (2, 7), (3, 6), (4, 5), (5, 4), (6, 3), (7, 2), (8, 1),
    (1, 9), (2, 8), (3, 7), (4, 6), (5, 5), (6, 4), (7, 3), (8, 2), (9, 1),
    (1, 10), (2, 9), (3, 8), (4, 7), (5, 6), (6, 5), (7, 4), (8, 3), (9, 2), (10, 1)
];

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
    const MAXIMUM_FRAME_SIZE: usize = 1 << 16;

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

            // the final folded distribution for this component, scaled to sum up to a power of two.
            let mut scaled_distribution = Vec::new();
            let (mut fidelity, mut radix) = (0usize, 0usize);
            let mut frame_size = usize::MAX;
            let mut lowest_cost = f64::MAX;

            for (fid, rad) in PARAMS_COMBINATIONS.iter() {
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

                // We need the list of symbols indexes sorted by the frequency of the related
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
                        // if we have reached the maximum frame size, we can go further with
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

            assert_ne!(frame_size, usize::MAX, "\
            It's not been possible to approximate the folded distribution for the component {} with \
            any of the available radix and fidelity and a frame size <= 2^16."
            , BVGraphComponent::from(component));

            components_final_cost.push(lowest_cost);

            let mut table = Vec::with_capacity(scaled_distribution.len());
            let log_m = frame_size.ilog2() as usize;
            let k = if log_m > 0 { 16 - log_m } else { 15 };

            let mut last_covered_freq = 0;

            for freq in scaled_distribution.iter() {
                table.push(EncoderModelEntry::new(*freq as u16, k, last_covered_freq));
                last_covered_freq = last_covered_freq.checked_add(*freq as u16).unwrap_or(0);
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
            "Final graph entropy before ANS encoding is {:?} B (+{:.2}% w.r.t the original cost)\n",
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
}
