use crate::{RawSymbol, Symbol};
use epserde::traits::ZeroCopy;

/// A trait for those types which are quasi-foldable and quasi-unfoldable (according to the
/// value of the current `RADIX` value).
///
/// Currently, since the biggest symbol can be at most 2^48 - 1, the biggest type on
/// which this trait can be implemented is `u64`.
pub trait Quasi<const RADIX: usize>:
    Into<u64> + Clone + Default + Copy + ZeroCopy + 'static
{
    /// Given the width of the current type, how many bits are reserved to represent the symbol.
    ///
    /// Note that the concrete value of this constant will determine the maximum symbol that can be correctly
    /// encoded/decoded.
    const BIT_RESERVED_FOR_SYMBOL: u64;

    /// Given a symbol, returns a number (the quasi-folded) that contains both information about the correspondent raw
    /// symbol and the number of folds needed to fully unfold the symbol.
    fn quasi_fold(sym: Symbol, folding_threshold: u64, folding_offset: u64) -> Self;

    /// Given a quasi-folded symbol, extracts, and then returns, the data included during the `quasi-folding` process.
    fn quasi_unfold(quasi_folded: Self) -> (Self, u32);
}

// For the u32 type, we currently provide only the implementation for the radix equal to 8.
// This means that we can make at most 3 folds (representable by two bits). This is the reason
// why we reserve 30 bits to represent the symbol.
impl Quasi<8> for u32 {
    const BIT_RESERVED_FOR_SYMBOL: u64 = 30;

    fn quasi_fold(sym: Symbol, folding_threshold: u64, folding_offset: u64) -> Self {
        if sym < folding_threshold as Symbol {
            return sym as u32;
        }

        let mut symbol = sym as u32;
        let folds = (symbol - folding_threshold as u32) / folding_offset as u32 + 1_u32;
        let folds_bits = folds << Self::BIT_RESERVED_FOR_SYMBOL;

        symbol -= folding_offset as u32 * folds;
        symbol <<= folds * 8; // radix is fixed to be 8 in this case

        if symbol >= (1_u32 << Self::BIT_RESERVED_FOR_SYMBOL) {
            // We can handle symbols that consume at most the number of bits reserved for it.
            panic!("Symbol is too big to be quasi-unfolded");
        }

        symbol | folds_bits
    }

    fn quasi_unfold(quasi_folded: Self) -> (Self, u32) {
        let symbol = quasi_folded & ((1 << Self::BIT_RESERVED_FOR_SYMBOL) - 1);
        let folds = quasi_folded >> Self::BIT_RESERVED_FOR_SYMBOL;
        (symbol, folds)
    }
}

impl<const RADIX: usize> Quasi<RADIX> for u64 {
    // We reserve 48 bits to represent the symbol and the remaining 16 bits to represent the number of folds.
    const BIT_RESERVED_FOR_SYMBOL: u64 = 48;

    fn quasi_fold(sym: Symbol, folding_threshold: u64, folding_offset: u64) -> Self {
        if sym < folding_threshold as Symbol {
            return sym as u64;
        }

        let mut symbol = sym as u64;
        let folds = (symbol - folding_threshold) / folding_offset + 1_u64;
        let folds_bits = folds << <Self as Quasi<RADIX>>::BIT_RESERVED_FOR_SYMBOL;

        symbol -= folding_offset * folds as RawSymbol;
        symbol <<= folds * RADIX as u64;

        // in this case we can avoid checking that the symbol is too big to be quasi-unfolded since it's already been
        // checked.
        symbol | folds_bits
    }

    fn quasi_unfold(quasi_folded: Self) -> (Self, u32) {
        let symbol = quasi_folded & ((1 << <Self as Quasi<RADIX>>::BIT_RESERVED_FOR_SYMBOL) - 1);
        let folds = quasi_folded >> <Self as Quasi<RADIX>>::BIT_RESERVED_FOR_SYMBOL;
        (symbol, folds as u32)
    }
}

// TODO: here
pub trait Decode {
    fn get_frame_mask(&self, model_index: usize) -> u64;

    fn get_log2_frame_size(&self, model_index: usize) -> usize;
}
