use crate::traits::quasi::Quasi;
use crate::{State, Symbol};
use epserde::prelude::*;
use std::ops::Index;
use sucds::bit_vectors::{Rank, Rank9Sel};
use sux::prelude::*;
use crate::ans::{DecoderModelEntry, EncoderModelEntry};

pub struct EliasFanoFrame<const RADIX: usize, T>
where
    T: Quasi<RADIX> + ZeroCopy,
{
    /// Contains, in each position, the data associated to the symbol in the same position within the EliasFano structure.
    symbols: Vec<DecoderModelEntry<RADIX, T>>,

    /// The mapped frame as an Elias-Fano structure.
    frame: EliasFano,
}

impl<const RADIX: usize, T: Quasi<RADIX>> EliasFanoFrame<RADIX, T> {
    pub fn new(
        table: &[EncoderModelEntry],
        log2_frame: usize,
        folding_offset: u64,
        folding_threshold: u64,
    ) -> Self {
        let nonzero_symbols = table.iter().filter(|sym| sym.freq > 0).count();
        let mut symbols = Vec::with_capacity(nonzero_symbols);
        let mut frame_builder = EliasFanoBuilder::new(nonzero_symbols + 1, (1 << log2_frame) + 1);

        table
            .iter()
            .enumerate()
            .filter(|(_, symbol_entry)| symbol_entry.freq > 0)
            .for_each(|(sym, symbol_entry)| {
                frame_builder
                    .push(symbol_entry.cumul_freq as usize)
                    .unwrap();
                symbols.push(DecoderModelEntry {
                    freq: symbol_entry.freq,
                    cumul_freq: symbol_entry.cumul_freq,
                    quasi_folded: T::quasi_fold(sym as Symbol, folding_threshold, folding_offset),
                });
            });

        frame_builder.push(1 << log2_frame).unwrap();
        let frame: EliasFano = frame_builder.build().convert_to().unwrap();

        Self {
            symbols,
            frame: frame.convert_to().unwrap(),
        }
    }
}

impl<const RADIX: usize, T: Quasi<RADIX>> Index<State> for EliasFanoFrame<RADIX, T> {
    type Output = DecoderModelEntry<RADIX, T>;

    #[inline(always)]
    fn index(&self, slot: State) -> &Self::Output {
        let symbol_index =
            unsafe { self.frame.pred_unchecked::<false>(&(slot as usize)).0 as Symbol };
        &self.symbols[symbol_index as usize]
    }
}

#[derive(Clone)]
pub struct Rank9SelFrame<const RADIX: usize, T: Quasi<RADIX>> {
    /// Contains, in each position, the data associated to the symbol in the same position within the Rank9Sel structure.
    symbols: Vec<DecoderModelEntry<RADIX, T>>,

    frame: Rank9Sel,
}

impl<const RADIX: usize, T: Quasi<RADIX>> Rank9SelFrame<RADIX, T> {
    pub fn new(
        table: &[EncoderModelEntry],
        log2_frame: usize,
        folding_offset: u64,
        folding_threshold: u64,
    ) -> Self {
        let nonzero_symbols = table.iter().filter(|sym| sym.freq > 0).count();
        let mut symbols = Vec::with_capacity(nonzero_symbols);
        let mut vec = vec![false; 1 << log2_frame];

        table
            .iter()
            .enumerate()
            .filter(|(_, symbol_entry)| symbol_entry.freq > 0)
            .for_each(|(sym, symbol_entry)| {
                match symbol_entry.cumul_freq {
                    0 => (),
                    _ => *vec.get_mut(symbol_entry.cumul_freq as usize).unwrap() = true,
                }

                symbols.push(DecoderModelEntry {
                    freq: symbol_entry.freq,
                    cumul_freq: symbol_entry.cumul_freq,
                    quasi_folded: T::quasi_fold(sym as Symbol, folding_threshold, folding_offset),
                });
            });

        Self {
            symbols,
            frame: Rank9Sel::from_bits(vec),
        }
    }
}

impl<const RADIX: usize, T: Quasi<RADIX>> Index<State> for Rank9SelFrame<RADIX, T> {
    type Output = DecoderModelEntry<RADIX, T>;

    #[inline(always)]
    fn index(&self, slot: State) -> &Self::Output {
        let symbol_index = self.frame.rank1((slot + 1) as usize).unwrap() as Symbol;
        &self.symbols[symbol_index as usize]
    }
}

#[derive(Clone)]
pub struct VecFrame<const RADIX: usize, T: Quasi<RADIX>>(Vec<DecoderModelEntry<RADIX, T>>);

impl<const RADIX: usize, T: Quasi<RADIX>> VecFrame<RADIX, T> {
    pub fn new(
        table: &[EncoderModelEntry],
        log2_frame_size: usize,
        folding_offset: u64,
        folding_threshold: u64,
    ) -> Self {
        let mut vec = vec![DecoderModelEntry::default(); 1 << log2_frame_size];
        let mut last_slot = 0; // the last slot of the frame we have actually filled with data

        table
            .iter()
            .enumerate()
            .filter(|(_, symbol_entry)| symbol_entry.freq > 0)
            .for_each(|(sym, symbol_entry)| {
                for slot in last_slot..last_slot + symbol_entry.freq {
                    // fill the symbol's slots with the data
                    *vec.get_mut(slot as usize).unwrap() = DecoderModelEntry {
                        freq: symbol_entry.freq,
                        cumul_freq: symbol_entry.cumul_freq,
                        quasi_folded: T::quasi_fold(
                            sym as Symbol,
                            folding_threshold,
                            folding_offset,
                        ),
                    };
                }
                last_slot += symbol_entry.freq;
            });
        Self(vec)
    }
}

impl<const RADIX: usize, T: Quasi<RADIX>> Index<State> for VecFrame<RADIX, T> {
    type Output = DecoderModelEntry<RADIX, T>;

    #[inline(always)]
    fn index(&self, slot: State) -> &Self::Output {
        &self.0[slot as usize]
    }
}
