use std::ops::Index;

use sucds::bit_vectors::{Rank, Rank9Sel};

use crate::ans::traits::RESERVED_TO_SYMBOL;
use crate::ans::EncoderModelEntry;
use crate::{RawSymbol, State, Symbol};
use sux::prelude::*;

#[readonly::make]
#[derive(Clone, Debug, Default)]
pub struct DecoderModelEntry {
    pub freq: u16,
    pub cumul_freq: u16,
    pub mapped_num: u64,
}

pub struct EliasFanoFrame<const RADIX: usize> {

    /// Contains, in each position, the data associated to the symbol in the same position within the EliasFano structure.
    symbols: Vec<DecoderModelEntry>,

    /// The mapped frame as an Elias-Fano structure.
    frame: EliasFano,
}

impl<const RADIX: usize> EliasFanoFrame<RADIX> {

    pub fn new(table: &[EncoderModelEntry], log2_frame: usize, folding_offset: u64, folding_threshold: u64) -> Self {
        let nonzero_symbols = table.iter().filter(|sym| sym.freq > 0).count();

        let mut symbols = Vec::with_capacity(nonzero_symbols);
        let mut frame_builder =
            EliasFanoBuilder::new(nonzero_symbols + 1, (1 << log2_frame) + 1);

        for (sym, sym_data) in table.iter().enumerate() {
            if sym_data.freq == 0 { continue; }

            frame_builder.push(sym_data.cumul_freq as usize).unwrap();

            symbols.push(DecoderModelEntry {
                freq: sym_data.freq as u16,
                cumul_freq: sym_data.cumul_freq as u16,
                mapped_num: quasi_unfold::<RADIX>(sym as Symbol, folding_threshold, folding_offset),
            });
        }
        frame_builder.push(1 << log2_frame).unwrap();

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
        let symbol_index = unsafe { self.frame.pred_unchecked::<false>(&(slot as usize)).0 as Symbol };
        &self.symbols[symbol_index as usize]
    }
}


#[derive(Clone)]
pub struct VecFrame<const RADIX: usize>(Vec<DecoderModelEntry>);

impl<const RADIX: usize> VecFrame<RADIX> {

    /// Creates a new VecFrame from the given table.
    pub fn new(table: &[EncoderModelEntry], log2_frame_size: usize, folding_offset: RawSymbol, folding_threshold: RawSymbol) -> Self {
        let mut vec = vec![DecoderModelEntry::default(); 1 << log2_frame_size];
        let mut last_slot = 0; // the last slot of the frame we have actually filled with data

        for (symbol, symbol_entry) in table.iter().enumerate() {
            if symbol_entry.freq == 0 {
                continue; // let's skip symbols with frequency 0
            }

            for slot in last_slot..last_slot + symbol_entry.freq {
                // fill the symbol's slots with the data
                *vec.get_mut(slot as usize).unwrap() = DecoderModelEntry {
                    freq: symbol_entry.freq as u16,
                    cumul_freq: symbol_entry.cumul_freq as u16,
                    mapped_num: quasi_unfold::<RADIX>(symbol as Symbol, folding_threshold, folding_offset),
                };
            }
            last_slot += symbol_entry.freq;
        }
        Self(vec)
    }
}

impl<const RADIX: usize> Index<State> for VecFrame<RADIX> {
    type Output = DecoderModelEntry;

    #[inline(always)]
    fn index(&self, slot: State) -> &Self::Output {
        &self.0[slot as usize]
    }
}

#[derive(Clone)]
pub struct Rank9SelFrame<const RADIX: usize> {
    /// Contains, in each position, the data associated to the symbol in the same position within the Rank9Sel structure.
    symbols: Vec<DecoderModelEntry>,

    frame: Rank9Sel,
}

impl<const RADIX: usize> Rank9SelFrame<RADIX> {
    pub fn new(table: &[EncoderModelEntry], log2_frame: usize, folding_offset: u64, folding_threshold: u64) -> Self {
        let nonzero_symbols = table.iter().filter(|sym| sym.freq > 0).count();
        let mut symbols = Vec::with_capacity(nonzero_symbols);
        let mut vec = vec![false; 1 << log2_frame];

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

            symbols.push(DecoderModelEntry {
                freq: sym_data.freq as u16,
                cumul_freq: sym_data.cumul_freq as u16,
                mapped_num: quasi_unfold::<RADIX>(sym as Symbol, folding_threshold, folding_offset),
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
pub fn quasi_unfold<const RADIX: usize>(sym: Symbol, folding_threshold: u64, folding_offset: u64) -> u64 {
    if sym < folding_threshold as Symbol {
        return sym as u64;
    }

    let mut symbol = sym as u64;
    let folds = ((symbol - folding_threshold) / folding_offset + 1_u64) as u16;
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
