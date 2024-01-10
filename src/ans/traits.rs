use bitvec::prelude::*;
use crate::{RawSymbol, Symbol};


/// This Fold trait allows for folding and unfolding symbols from a source.
///
/// The generic constant `RADIX` is used to specify the value of the radix parameter used to fold
/// symbols.
#[allow(clippy::len_without_is_empty)]
pub trait Fold<const RADIX: usize> {
    /// The constant RADIX value used to fold symbols.
    const RADIX: usize = RADIX;

    /// How many blocks of `radix` bits have to be extracted from the symbol in order to fold it.
    fn get_folds_number(symbol: RawSymbol, fidelity: usize) -> usize {
        ((u64::ilog2(symbol) + 1) as usize - fidelity) / Self::RADIX
    }

    /// Performs the so called 'symbol folding'.
    fn fold_symbol(&mut self, symbol: RawSymbol, fidelity: usize) -> Symbol;

    fn len(&self) -> usize;

    /// Unfolds a symbol from the given `mapped_num` and returns it.
    fn unfold_symbol<T: Quasi<RADIX>> (&self, mapped_num: T, last_read: &mut usize) -> RawSymbol {
        let (quasi_unfolded, folds) = T::quasi_unfold(mapped_num);
        let folded_bits= self.read_folds(folds as usize, last_read);

        quasi_unfolded.into() | folded_bits
    }

    /// Reads the exact number of folded bits from the source.
    fn read_folds(&self, folds: usize, last_read: &mut usize) -> RawSymbol;
}


impl Fold<8> for Vec<u8> { // the fastest implementation since uses a vec of bytes

    fn fold_symbol(&mut self, mut symbol: RawSymbol, fidelity: usize) -> Symbol {
        let folds = Self::get_folds_number(symbol, fidelity);
        let offset = (((1 << Self::RADIX) - 1) * (1 << (fidelity - 1))) * folds as RawSymbol;
        let bytes = symbol.to_be_bytes();

        self.extend_from_slice(bytes[8 - folds..].as_ref());

        symbol >>= folds * Self::RADIX;
        (symbol + offset) as u16
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn read_folds (&self, folds: usize, last_read: &mut usize) -> RawSymbol {
        let mut folded_bytes: u64 = 0;

        for index in 0..folds {
            *last_read -= 1;
            folded_bytes |= (self[*last_read] as u64) << ((index) * 8);
        }

        folded_bytes
    }
}

// This is a general implementation that folds symbols given any reasonable radix.
// This generality makes this implementation slower since it doesn't allow relevant optimizations
// used with radix equal to 8.
impl<const RADIX: usize> Fold<RADIX> for BitVec<usize, Msb0> {

    fn fold_symbol(&mut self, mut symbol: RawSymbol, fidelity: usize) -> Symbol {
        let cuts = <Self as Fold<RADIX>>::get_folds_number(symbol, fidelity);
        let offset = (((1 << RADIX) - 1) * (1 << (fidelity - 1))) * cuts as RawSymbol;
        let bit_to_cut = cuts * RADIX;

        self.extend_from_bitslice(
            symbol
                .view_bits::<Msb0>()
                .split_at(RawSymbol::BITS as usize - bit_to_cut)
                .1,
        );

        symbol >>= bit_to_cut;

        (symbol + offset) as u16
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn read_folds(&self, folds: usize, last_read: &mut usize) -> RawSymbol {
        if folds == 0 {
            return 0; // since load_be panics if we try to load from a slice of length 0
        }

        let bits = self
            .as_bitslice()
            .get(*last_read - folds * RADIX..*last_read)
            .unwrap();

        *last_read -= folds * RADIX;
        bits.load_be::<RawSymbol>()
    }
}


/// A trait for those types which are quasi-foldable and quasi-unfoldable (according to the
/// value of the current `RADIX` value).
///
/// Currently, since the biggest symbol can be at most 2^48 - 1, the biggest type on
/// which this trait can be implemented is `u64`.
pub trait Quasi<const RADIX: usize>:
    Into<u64>
    + Clone
    + Default
    + Copy
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

// Regarding the trait's implementation for this type, the following arrangement is used:
// - the 16 MSB are used to store the number of folds
// - the 48 LSB are used to store the quasi-folded symbol
impl <const RADIX: usize> Quasi<RADIX> for u64 {

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
        // checked that symbols are not bigger than (2^48 - 1) in the enc_model:44.
        symbol | folds_bits
    }

    fn quasi_unfold(quasi_folded: Self) -> (Self, u32) {
        let symbol = quasi_folded & ((1 << <Self as Quasi<RADIX>>::BIT_RESERVED_FOR_SYMBOL) - 1);
        let folds = quasi_folded >> <Self as Quasi<RADIX>>::BIT_RESERVED_FOR_SYMBOL;
        (symbol, folds as u32)
    }
}


pub trait Decode {
    fn get_frame_mask(&self, model_index: usize) -> u64;

    fn get_log2_frame_size(&self, model_index: usize) -> usize;
}