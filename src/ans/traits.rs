use bitvec::prelude::*;

use crate::{RawSymbol, Symbol};

/// How many bits are reserved to represent the quasi-unfolded symbol in `mapped_num`
const RESERVED_TO_SYMBOL: usize = 48;

/// Mask used to extract the 48 LSB from `mapped_num`. This number will be the quasi-unfolded symbol.
const SYMBOL_MASK: u64 = (1 << RESERVED_TO_SYMBOL) - 1;

#[allow(clippy::len_without_is_empty)]
pub trait Foldable<const RADIX: usize> {
    const RADIX: usize = RADIX;
    /// How many blocks of `radix` bits have to be extracted from the symbol in order to fold it.
    fn get_folds_number(symbol: RawSymbol, fidelity: usize) -> usize {
        ((u64::ilog2(symbol) + 1) as usize - fidelity) / Self::RADIX
    }

    /// Performs the so called 'symbol folding'.
    fn fold_symbol(&mut self, symbol: RawSymbol, fidelity: usize) -> Symbol;

    fn len(&self) -> usize;

    /// Unfolds a symbol from the given `mapped_num` and returns it.
    fn unfold_symbol(&self, mapped_num: u64, last_unfolded: &mut usize) -> RawSymbol;
}

impl Foldable<8> for Vec<u8> {
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

    fn unfold_symbol(&self, mapped_num: u64, last_unfolded: &mut usize) -> RawSymbol {
        let quasi_unfolded = mapped_num & SYMBOL_MASK;
        let folds = mapped_num >> RESERVED_TO_SYMBOL;

        let mut unfolded_two: u64 = 0;

        for index in 0..folds {
            *last_unfolded -= 1;
            unfolded_two |= (self[*last_unfolded] as u64) << ((index) * 8);
        }

        quasi_unfolded | unfolded_two
    }
}

impl<const RADIX: usize> Foldable<RADIX> for BitVec<usize, Msb0> {
    /// This is a general implementation that folds symbols given any reasonable radix and fidelity.
    /// This generality makes this implementation slower since it doesn't allow relevant optimizations
    /// used with radix equal to 8.
    fn fold_symbol(&mut self, mut symbol: RawSymbol, fidelity: usize) -> Symbol {
        let cuts = <Self as Foldable<RADIX>>::get_folds_number(symbol, fidelity);
        let offset = (((1 << <Self as Foldable<RADIX>>::RADIX) - 1) * (1 << (fidelity - 1)))
            * cuts as RawSymbol;
        let bit_to_cut = cuts * <Self as Foldable<RADIX>>::RADIX;

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

    fn unfold_symbol(&self, mapped_num: u64, last_unfolded: &mut usize) -> RawSymbol {
        let folds = (mapped_num >> RESERVED_TO_SYMBOL) as usize;
        let quasi_unfolded = mapped_num & SYMBOL_MASK;
        let bits = self
            .as_bitslice()
            .get(*last_unfolded - folds * <Self as Foldable<RADIX>>::RADIX..*last_unfolded)
            .unwrap();

        *last_unfolded -= folds * <Self as Foldable<RADIX>>::RADIX;
        quasi_unfolded | bits.load_be::<RawSymbol>()
    }
}
