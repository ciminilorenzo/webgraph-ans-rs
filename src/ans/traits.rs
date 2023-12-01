use bitvec::prelude::*;

use crate::{RawSymbol, Symbol};


/// Mask used to extract the 48 LSB from `mapped_num`. This number will be the quasi-unfolded symbol.
const SYMBOL_MASK: u64 = 0x_FFFFFFFFFFFF;

/// How many bits are reserved to represent the quasi-unfolded symbol in `mapped_num`
const RESERVED_TO_SYMBOL: u8 = 48;


#[allow(clippy::len_without_is_empty)]
pub trait Foldable {
    /// How many blocks of `radix` bits have to be extracted from the symbol in order to fold it.
    fn get_folds_number(symbol: RawSymbol, radix: u8, fidelity: u8) -> u8 {
        (((u64::ilog2(symbol) + 1) as u64 - fidelity as u64) / radix as u64) as u8
    }

    /// Performs the so called 'symbol folding'.
    fn fold_symbol(symbol: RawSymbol, radix: u8, fidelity: u8, out: &mut Self) -> Symbol;

    fn len(&self) -> usize;

    /// Unfolds a symbol from the given `mapped_num` and returns it.
    fn unfold_symbol(&self, mapped_num: u64, last_unfolded: &mut usize, radix: u8) -> RawSymbol;
}

impl Foldable for Vec<u8> {

    fn fold_symbol(mut symbol: RawSymbol, radix: u8, fidelity: u8, out: &mut Self) -> Symbol {
        let folds = Self::get_folds_number(symbol, radix, fidelity);
        let offset = (((1 << radix) - 1) * (1 << (fidelity - 1))) * folds as RawSymbol;

        out.extend_from_slice(symbol.to_be_bytes()[8 - folds as usize..].as_ref());

        symbol >>= folds * radix;
        (symbol + offset) as u16
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn unfold_symbol(&self, mapped_num: u64, last_unfolded: &mut usize, _radix: u8) -> RawSymbol {
        let quasi_unfolded = mapped_num & SYMBOL_MASK;
        let folds = mapped_num >> RESERVED_TO_SYMBOL;
        let mut bytes = [0_u8; 8];

        bytes[8 - folds as usize..].copy_from_slice(&self[*last_unfolded - folds as usize..*last_unfolded]);
        *last_unfolded -= folds as usize;

        quasi_unfolded | u64::from_be_bytes(bytes)
    }
}

impl Foldable for BitVec<usize, Msb0> {

    /// This is a general implementation that folds symbols given any reasonable radix and fidelity.
    /// This generality makes this implementation slower since it doesn't allow relevant optimizations
    /// used with radix equal to 8.
    fn fold_symbol(mut symbol: RawSymbol, radix: u8, fidelity: u8, out: &mut Self) -> Symbol {
        let cuts = Self::get_folds_number(symbol, radix, fidelity);
        let offset = (((1 << radix) - 1) * (1 << (fidelity - 1))) * cuts as RawSymbol;
        let bit_to_cut = cuts * radix;

        out.extend_from_bitslice(symbol
            .view_bits::<Msb0>()
            .split_at(RawSymbol::BITS as usize - bit_to_cut as usize).1
        );

        symbol >>= bit_to_cut;

        (symbol + offset) as u16
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn unfold_symbol(&self, mapped_num: u64, last_unfolded: &mut usize, radix: u8) -> RawSymbol {
        let folds = (mapped_num >> RESERVED_TO_SYMBOL) as usize;
        let quasi_unfolded = mapped_num & SYMBOL_MASK;
        let bits = self
            .as_bitslice()
            .get(*last_unfolded - folds * radix as usize..*last_unfolded)
            .unwrap();

        *last_unfolded -= folds * radix as usize;
        quasi_unfolded | bits.load_be::<RawSymbol>()
    }
}