use bitvec::field::BitField;
use bitvec::prelude::{BitVec, Msb0};
use bitvec::view::BitView;

use crate::{RawSymbol, Symbol};
use crate::traits::quasi::Quasi;


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