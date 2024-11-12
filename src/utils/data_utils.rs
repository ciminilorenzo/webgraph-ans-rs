use anyhow::{bail, Result};

use std::cmp::max;


/// Tries to scale frequencies in `freqs` by using the new common denominator `new_frame`. This algorithm
/// gives priority to low frequency symbols in order to be sure that the space available is firstly given to symbols
/// with approximated frequency lower than 0. This happens when we are trying to approximate a distribution with a
/// frame size that is smaller compared to the original one.
///
/// # Returns
/// The approximated frequencies if is possible to approximate with the given `new_frame` else, if too
/// many symbols have frequency lower than 1 - meaning that M is not big enough to handle the whole
/// set of symbols - an error is returned.
pub fn scale_freqs(
    freqs: &[usize],
    sorted_indices: &[usize],
    n: usize,
    mut m: usize,
    mut new_m: isize,
) -> Result<Vec<usize>> {
    let mut approx_freqs = freqs.to_vec();
    let ratio = new_m as f64 / m as f64;

    for (index, sym_index) in sorted_indices.iter().enumerate() {
        let sym_freq = freqs[*sym_index];
        let second_ratio = new_m as f64 / m as f64;
        let scale = (n - index) as f64 * ratio / n as f64 + index as f64 * second_ratio / n as f64;
        approx_freqs[*sym_index] = max(1, (0.5 + scale * sym_freq as f64).floor() as usize);

        new_m -= approx_freqs[*sym_index] as isize;
        m -= sym_freq;

        if new_m < 0 {
            bail!("Too many symbols have frequency lower than 1! Need a bigger frame size");
        }
    }
    Ok(approx_freqs)
}