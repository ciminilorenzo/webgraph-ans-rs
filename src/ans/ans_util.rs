use std::cmp::max;

use bitvec::prelude::{BitVec, Msb0};
use bitvec::view::BitView;
use bitvec::field::BitField;

use anyhow::{bail, Result};

use crate::{RawSymbol, Symbol};
use crate::utils::{entropy, cross_entropy};


/// Performs the so called 'symbol folding'. This optimized implementation is different
/// from the one described in the paper since here the while loop is avoided in favour of a single
/// block of operations that performs the same task.
///
/// # Panics
/// - if the caller wants to stream bits out even though no BitVec reference is provided;
/// - if the folded symbol is bigger than u16::MAX.
pub fn fold_symbol(mut symbol: RawSymbol, stream_bits: bool, out: Option<&mut BitVec>, radix: u8, fidelity: u8) -> Symbol {
    let mut offset = 0;
    let threshold = 1 << (radix + fidelity - 1);

    if symbol >= threshold {
        let cuts = ((f64::log2(symbol as f64).floor() + 1_f64) - fidelity as f64) / radix as f64;
        let bit_to_cut = cuts as u8 * radix;

        stream_bits.then(|| {
            let mask = ((1 << bit_to_cut) - 1) as RawSymbol;

            // Now let's push only the bits we actually extracted from the original symbol.
            // `bits` contains usize bits thus we would push this amount of bits instead of the
            // `bit_to_cut` number of bits we extracted.
            // -> We need to split the bitslice.
            //
            // Eventually, we are reversing the bit since in the decoder we use the drain method
            // which takes bits from [here --> to here]
            let mut bits = (symbol & mask)
                .view_bits::<Msb0>()
                .to_bitvec()
                .drain(RawSymbol::BITS as usize - bit_to_cut as usize..).collect::<BitVec>();
            bits.reverse();

            out
                .unwrap_or_else(|| panic!("Cannot stream bits out without a BitVec!"))
                .extend_from_bitslice(&bits);
        });

        symbol >>= bit_to_cut;
        offset += (((1 << radix) - 1) * (1 << (fidelity - 1))) * cuts as RawSymbol;
    }
    u16::try_from(symbol + offset).expect("Folded symbol is bigger than u16::MAX")
}

pub fn undo_fold_symbol(symbol: Symbol, radix: u8, fidelity: u8, folded_bits: &mut BitVec) -> RawSymbol {
    let offset = ((1 << (fidelity - 1)) * ((1 << radix) - 1)) as RawSymbol;
    let threshold = (1 << (fidelity + radix - 1)) as RawSymbol;
    let symbol = symbol as RawSymbol;

    if symbol < threshold {
        symbol // singleton bucket
    } else {
        let folds_numer = (symbol - threshold) / offset + 1;
        let mut original_sym = symbol - (offset * folds_numer);
        let bits = folded_bits
            .drain(folded_bits.len() - (radix as usize * folds_numer as usize)..)
            .collect::<BitVec>();

        original_sym = (original_sym << (folds_numer * (radix as RawSymbol))) | bits.load::<RawSymbol>();
        original_sym
    }
}

pub fn approx_freqs(freqs: &[usize], n: usize, max_sym: Symbol) -> (Vec<usize>, usize) {
    let mut total_freq = 0;
    let mut indexed_freqs: Vec<(usize, usize)> = Vec::with_capacity(freqs.len());

    for (index, freq) in freqs.iter().enumerate() {
        if *freq == 0 { continue; }

        total_freq += freq;
        indexed_freqs.push((*freq, index));
    }

    indexed_freqs.shrink_to_fit();
    let mut frame_size = if n.is_power_of_two() { n } else { n.next_power_of_two() };
    let mut approx_freqs: Vec<usize>;

    let entropy = entropy(
        &indexed_freqs.iter().map(|(freq,_)| *freq).collect::<Vec<usize>>(),
        total_freq as f64
    );

    let sorted_indices = {
        let mut sorted_indexed_freqs = indexed_freqs.clone();
        sorted_indexed_freqs.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        sorted_indexed_freqs.iter().map(|(_, index)| *index).collect::<Vec<usize>>()
    };

    loop {
        assert!(frame_size <= (1 << 28), "frame_size must be at most 2^28");

        let scaling_result = try_scale_freqs(freqs, &sorted_indices, n, total_freq, frame_size as isize);

        match scaling_result {
            Ok(new_freqs) => {
                let cross_entropy = cross_entropy(
                    freqs,
                    total_freq as f64,
                    &new_freqs,
                    frame_size as f64,
                );

                if cross_entropy <= entropy * 1.001 {
                    approx_freqs = new_freqs;
                    break;
                } else {
                    frame_size *= 2;
                }
            },
            Err(_) => { frame_size *= 2; }
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
pub fn try_scale_freqs(freqs: &[usize], sorted_indices: &[usize], n: usize, mut total_freq: usize, mut new_frame: isize) -> Result<Vec<usize>> {
    let mut approx_freqs = freqs.to_vec();
    let ratio = new_frame as f64 / total_freq as f64;

    let get_approx_freq = |scale: f64, sym_freq: f64| {
        max(
            1,
            (0.5 + scale * sym_freq).floor() as usize
        )
    };

    for (index, sym_index) in sorted_indices.iter().enumerate() {
        let sym_freq = freqs[*sym_index];
        let second_ratio = new_frame as f64 / total_freq as f64;
        let scale = (n - index) as f64 * ratio / n as f64 + index as f64 * second_ratio / n as f64;

        approx_freqs[*sym_index] = get_approx_freq(scale, sym_freq as f64);
        new_frame -= approx_freqs[*sym_index] as isize;
        total_freq -= sym_freq;

        if new_frame < 0 { bail!("Cannot approximate frequencies with this new frame size!"); }
    }
    Ok(approx_freqs)
}


#[cfg(test)]
mod tests {
    use crate::ans::ans_util::approx_freqs;

    #[test]
    fn test() {
        let freqs = vec![0, 3, 3, 2, 1, 1];
        let n = 5;
        let expected = vec![0,10,10,6,3,3];
        assert_eq!((expected, 32), approx_freqs(&freqs, n, 5));
    }
}