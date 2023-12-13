use std::ops::Index;

use sucds::bit_vectors::{Rank, Rank9Sel};

use crate::ans::traits::RESERVED_TO_SYMBOL;
use crate::ans::EncoderModelEntry;
use crate::{RawSymbol, State, Symbol};
use sux::prelude::*;

#[readonly::make]
#[derive(Clone, Debug, Default)]
pub struct DecoderModelEntry {
    pub symbol: Symbol,
    pub freq: u32,
    pub cumul_freq: u32,
    pub mapped_num: u64,
}

pub struct EliasFanoFrame<const RADIX: usize> {
    /// Contains, in each position, the data associated to the symbol in the same position within the EliasFano structure.
    symbols: Vec<DecoderModelEntry>,

    /// The mapped frame as an Elias-Fano structure.
    frame: EliasFano,
}

impl<const RADIX: usize> EliasFanoFrame<RADIX> {
    pub fn new(
        table: &[EncoderModelEntry],
        log2_frame_size: usize,
        folding_offset: RawSymbol,
        folding_threshold: RawSymbol,
        _radix: usize,
    ) -> Self {
        assert!(
            table.len() < 1 << Symbol::BITS,
            "Can't have more than u16::MAX symbols"
        );

        let nonzero_symbols = table.iter().filter(|sym| sym.freq > 0).count();

        let mut symbols = Vec::with_capacity(nonzero_symbols);
        let mut frame_builder =
            EliasFanoBuilder::new(nonzero_symbols + 1, (1 << log2_frame_size) + 1);

        for (sym, sym_data) in table.iter().enumerate() {
            if sym_data.freq == 0 {
                continue;
            }

            frame_builder.push(sym_data.cumul_freq as usize).unwrap();

            let mapped_num = if sym < folding_threshold as usize {
                0_u64
            } else {
                quasi_unfold::<RADIX>(sym as Symbol, folding_threshold, folding_offset)
            };

            symbols.push(DecoderModelEntry {
                symbol: sym as Symbol,
                freq: sym_data.freq,
                cumul_freq: sym_data.cumul_freq,
                mapped_num,
            });
        }
        frame_builder.push(1 << log2_frame_size).unwrap();

        let frame: EliasFano<QuantumIndex> = frame_builder.build().convert_to().unwrap();

        Self {
            symbols,
            frame: frame.convert_to().unwrap(),
        }
    }
}

impl<const RADIX: usize> Index<State> for EliasFanoFrame<RADIX> {
    type Output = DecoderModelEntry;

    #[inline(always)]
    fn index(&self, slot: State) -> &Self::Output {
        let symbol_index =
            unsafe { self.frame.pred_unchecked::<false>(&(slot as usize)).0 as Symbol };
        &self.symbols[symbol_index as usize]
    }
}

/*
pub struct EliasFanoFrame<const RADIX: usize> {
    /// Contains, in each position, the data associated to the symbol in the same position within the EliasFano structure.
    symbols: Vec<DecoderModelEntry>,

    /// The mapped frame as an Elias-Fano structure.
    frame: EliasFano,
}
*/


#[derive(Clone)]
pub struct VecFrame<const RADIX: usize> {

    /// Contains, in each position, the data associated to the symbol with the same value.
    symbols: Vec<DecoderModelEntry>,

    /// Contains, in each position (slots from 0 up to M-1), the identifier of the associated symbol.
    frame: Vec<Symbol>,
}

impl<const RADIX: usize> VecFrame<RADIX> {
    /// Creates a new VecFrame from the given table.
    pub fn new(table: &[EncoderModelEntry], log2_frame_size: usize, folding_offset: RawSymbol, folding_threshold: RawSymbol, _radix: usize, ) -> Self {
        let mut vec = vec![0; 1 << log2_frame_size];
        let mut symbols = vec![DecoderModelEntry::default(); table.len()];
        let mut last_slot = 0; // the last slot of the frame we have actually filled with data

        let mut fill_slots = |symbol: Symbol, freq: u32, last_slot: u32| {
            for slot in last_slot..last_slot + freq {
                // fill the slot with the symbol id
                *vec.get_mut(slot as usize).unwrap() = symbol as Symbol;
            }
        };

        for (symbol, symbol_entry) in table.iter().enumerate() {
            if symbol_entry.freq == 0 {
                continue; // let's skip symbols with frequency 0
            }

            fill_slots(symbol as Symbol, symbol_entry.freq, last_slot);
            last_slot += symbol_entry.freq;

            let mapped_num = if symbol < folding_threshold as usize {
                0_u64 // this symbol it's not folded
            } else {
                quasi_unfold::<RADIX>(symbol as Symbol, folding_threshold, folding_offset)
            };

            *symbols.get_mut(symbol).unwrap() = DecoderModelEntry {
                symbol: symbol as Symbol,
                freq: symbol_entry.freq,
                cumul_freq: symbol_entry.cumul_freq,
                mapped_num,
            };
        }

        symbols.shrink_to_fit();

        Self {
            symbols,
            frame: vec,
        }
    }
}

impl<const RADIX: usize> Index<State> for VecFrame<RADIX> {
    type Output = DecoderModelEntry;

    #[inline(always)]
    fn index(&self, slot: State) -> &Self::Output {
        let symbol_id = self.frame[slot as usize];
        &self.symbols[symbol_id as usize]
    }
}

#[derive(Clone)]
pub struct Rank9SelFrame<const RADIX: usize> {
    /// Contains, in each position, the data associated to the symbol in the same position within the Rank9Sel structure.
    symbols: Vec<DecoderModelEntry>,

    frame: Rank9Sel,
}

impl<const RADIX: usize> Rank9SelFrame<RADIX> {
    pub fn new(
        table: &[EncoderModelEntry],
        log2_frame_size: usize,
        folding_offset: RawSymbol,
        folding_threshold: RawSymbol,
        _radix: usize,
    ) -> Self {
        assert!(table.len() < 1 << Symbol::BITS, "Too many symbols");

        let nonzero_symbols = table.iter().filter(|sym| sym.freq > 0).count();
        let mut symbols = Vec::with_capacity(nonzero_symbols);
        let mut vec = vec![false; 1 << log2_frame_size];

        for (sym, sym_data) in table.iter().enumerate() {
            if sym_data.freq == 0 {
                continue;
            }

            match sym_data.cumul_freq {
                0 => (),
                _ => {
                    let bit = vec.get_mut(sym_data.cumul_freq as usize).unwrap();
                    *bit = true;
                }
            }

            let mapped_num = if sym < folding_threshold as usize {
                0_u64
            } else {
                quasi_unfold::<RADIX>(sym as Symbol, folding_threshold, folding_offset)
            };

            symbols.push(DecoderModelEntry {
                symbol: sym as Symbol,
                freq: sym_data.freq,
                cumul_freq: sym_data.cumul_freq,
                mapped_num,
            });
        }

        Self {
            symbols,
            frame: Rank9Sel::from_bits(vec),
        }
    }
}

impl<const RADIX: usize> Index<State> for Rank9SelFrame<RADIX> {
    type Output = DecoderModelEntry;

    #[inline(always)]
    fn index(&self, slot: State) -> &Self::Output {
        let symbol_index = self.frame.rank1((slot + 1) as usize).unwrap() as Symbol;
        &self.symbols[symbol_index as usize]
    }
}

/// Quasi-unfolds the given symbol.
///
/// Quasi unfolding means creating a u64 with the following features:
///
/// 1. the 16 MSB bits are used to represent the number of folds (of size radix) that have been
/// performed during the symbol folding.
///
/// 2. the remaining 48 LSB bits contain: the fidelity bits in common between all the symbols folded
/// within the same bucket plus all zeros.
///
/// ## Example
/// Given radix and fidelity equal to 4 and 2, if 1111101000 is the original symbol from the input,
/// then 0000000000000010 are the 16 MSB of the quasi-unfolded symbol since 2 folds have to be done
/// in order to unfold the symbol while, the remaining 48 LSB are 1100000000 (with the remaining 40 MSB
/// equal to 0) since all the symbols bucketed in the same bucket have the same 2 fidelity bits (11)
/// and need to be unfolded in the same way (with 2 * 4 -radix- bits).
pub fn quasi_unfold<const RADIX: usize>(
    symbol: Symbol,
    folding_threshold: RawSymbol,
    folding_offset: RawSymbol,
) -> RawSymbol {
    let mut symbol = symbol as u64;

    let folds = u16::try_from((symbol - folding_threshold) / folding_offset + 1)
        .expect("Can't handle more than (2^16 - 1) folds.");

    let folds_bits = (folds as u64) << RESERVED_TO_SYMBOL;
    symbol -= folding_offset * folds as RawSymbol;
    symbol <<= (folds * RADIX as u16) as u64;

    // we want to have the 16 MSB bits free
    assert!(
        u64::ilog2(symbol) <= RESERVED_TO_SYMBOL as u32,
        "Can't handle a number bigger than 2^48 - 1"
    );

    symbol | folds_bits
}
