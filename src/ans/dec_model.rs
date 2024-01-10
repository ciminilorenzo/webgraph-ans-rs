use sux::prelude::*;

use sucds::bit_vectors::{Rank, Rank9Sel};

use crate::ans::traits::{Decode, Quasi};
use crate::ans::{DecoderModelEntry, EncoderModelEntry};
use crate::{State, Symbol};
use crate::ans::enc_model::SymbolLookup;


pub struct EliasFanoFrame<const RADIX: usize, T>
    where
        T: Quasi<RADIX>
{
    /// Contains the log2 of the frame size for each model.
    frame_sizes: Vec<usize>,

    /// Contains a list of vector of entries for each model where, in each index, the data associated to the symbol equal to that index.
    symbols: Vec<Vec<DecoderModelEntry<RADIX, T>>>,

    /// The mapped frames as Elias-Fano structures.
    frames: Vec<EliasFano>,
}

impl <const RADIX: usize, T: Quasi<RADIX>> Decode for EliasFanoFrame<RADIX, T> {

    #[inline(always)]
    fn get_frame_mask(&self, model_index: usize) -> u64 {
        (1 << self.frame_sizes[model_index]) - 1
    }

    #[inline(always)]
    fn get_log2_frame_size(&self, model_index: usize) -> usize {
        self.frame_sizes[model_index]
    }
}

impl<const RADIX: usize, T: Quasi<RADIX>> EliasFanoFrame<RADIX, T> {

    pub fn new(
        tables: Vec<Vec<EncoderModelEntry>>,
        frame_sizes: Vec<usize>,
        folding_offset: u64,
        folding_threshold: u64,
    ) -> Self {
        let mut symbols_table = Vec::with_capacity(tables.len());
        let mut elias_table = Vec::with_capacity(tables.len());

        tables.iter().enumerate().for_each(|(model_index, table)| {
            let nonzero_symbols = tables[model_index].iter().filter(|sym| sym.freq > 0).count();
            let mut symbols = Vec::with_capacity(nonzero_symbols);
            let mut frame_builder = EliasFanoBuilder::new(nonzero_symbols + 1, (1 << frame_sizes[model_index]) + 1);

            for (sym, sym_data) in table.iter().enumerate() {
                if sym_data.freq == 0 { continue; }

                frame_builder.push(sym_data.cumul_freq as usize).unwrap();

                symbols.push(DecoderModelEntry {
                    freq: sym_data.freq,
                    cumul_freq: sym_data.cumul_freq,
                    quasi_folded: T::quasi_fold(sym as Symbol, folding_threshold, folding_offset),
                });
            }

            frame_builder.push(1 << frame_sizes[model_index]).unwrap();
            let frame: EliasFano = frame_builder.build().convert_to().unwrap();
            symbols_table.push(symbols);
            elias_table.push(frame);
        });

        Self {
            frame_sizes,
            symbols: symbols_table,
            frames: elias_table,
        }
    }
}

impl <const RADIX: usize, T: Quasi<RADIX>> SymbolLookup<State> for EliasFanoFrame<RADIX, T> {
    type Output = DecoderModelEntry<RADIX, T>;

    #[inline(always)]
    fn symbol(&self, slot: State, model_index: usize) -> &Self::Output {
        let symbol_index = unsafe { self.frames[model_index].pred_unchecked::<false>(&(slot as usize)).0 as Symbol };
        &self.symbols[model_index][symbol_index as usize]
    }
}


#[derive(Clone)]
pub struct Rank9SelFrame<const RADIX: usize, T: Quasi<RADIX>> {
    /// Contains the log2 of the frame size for each model.
    frame_sizes: Vec<usize>,

    /// Contains a list of vector of entries for each model where, in each index, the data associated to the symbol equal to that index.
    symbols: Vec<Vec<DecoderModelEntry<RADIX, T>>>,

    frames: Vec<Rank9Sel>,
}

impl<const RADIX: usize, T: Quasi<RADIX>> Rank9SelFrame<RADIX, T> {
    pub fn new(
        tables: Vec<Vec<EncoderModelEntry>>,
        frame_sizes: Vec<usize>,
        folding_offset: u64,
        folding_threshold: u64,
    ) -> Self {
        let mut symbols_table = Vec::with_capacity(tables.len());
        let mut rank9_table = Vec::with_capacity(tables.len());

        tables.iter().enumerate().for_each(|(model_index, table)| {
            let nonzero_symbols = tables[model_index].iter().filter(|sym| sym.freq > 0).count();
            let mut symbols = Vec::with_capacity(nonzero_symbols);
            let mut vec = vec![false; 1 << frame_sizes[model_index]];

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
                    freq: sym_data.freq,
                    cumul_freq: sym_data.cumul_freq,
                    quasi_folded: T::quasi_fold(sym as Symbol, folding_threshold, folding_offset),
                });
            }

            rank9_table.push(Rank9Sel::from_bits(vec));
            symbols_table.push(symbols);
        });

        Self {
            frame_sizes,
            symbols: symbols_table,
            frames: rank9_table,
        }
    }
}

impl <const RADIX: usize, T: Quasi<RADIX>> Decode for Rank9SelFrame<RADIX, T> {

    #[inline(always)]
    fn get_frame_mask(&self, model_index: usize) -> u64 {
        (1 << self.frame_sizes[model_index]) - 1
    }

    #[inline(always)]
    fn get_log2_frame_size(&self, model_index: usize) -> usize {
        self.frame_sizes[model_index]
    }
}

impl<const RADIX: usize, T: Quasi<RADIX>> SymbolLookup<State> for Rank9SelFrame<RADIX, T> {
    type Output = DecoderModelEntry<RADIX, T>;

    #[inline(always)]
    fn symbol(&self, slot: State, model_index: usize) -> &Self::Output {
        let symbol_index = self.frames[model_index].rank1((slot + 1) as usize).unwrap() as Symbol;
        &self.symbols[model_index][symbol_index as usize]
    }
}


#[derive(Clone)]
pub struct VecFrame<const RADIX: usize, T: Quasi<RADIX>> {
    /// Contains the log2 of the frame size for each model.
    frame_sizes: Vec<usize>,

    /// Contains a set of vectors, one each for model. Within each vector, each index contains the data associated to
    /// the symbol equal to that index.
    symbols: Vec<Vec<DecoderModelEntry<RADIX, T>>>,
}

impl<const RADIX: usize, T: Quasi<RADIX>> VecFrame<RADIX, T> {

    pub fn new(
        tables: Vec<Vec<EncoderModelEntry>>,
        frame_sizes: Vec<usize>,
        folding_offset: u64,
        folding_threshold: u64,
    ) -> Self {
        let mut vectors = Vec::with_capacity(tables.len());

        tables.iter().enumerate().for_each(|(model_index, table)| {
            let mut vec = vec![DecoderModelEntry::default(); 1 << frame_sizes[model_index]];
            let mut last_slot = 0; // the last slot of the frame we have actually filled with data

            for (sym, symbol_entry) in table.iter().enumerate() {
                if symbol_entry.freq == 0 {
                    continue; // let's skip symbols with frequency 0
                }

                for slot in last_slot..last_slot + symbol_entry.freq {
                    // fill the symbol's slots with the data
                    *vec.get_mut(slot as usize).unwrap() = DecoderModelEntry {
                        freq: symbol_entry.freq,
                        cumul_freq: symbol_entry.cumul_freq,
                        quasi_folded: T::quasi_fold(sym as Symbol, folding_threshold, folding_offset),
                    };
                }
                last_slot += symbol_entry.freq;
            }

            vectors.push(vec);
        });

        Self {
            frame_sizes,
            symbols: vectors,
        }
    }
}

impl <const RADIX: usize, T: Quasi<RADIX>> Decode for VecFrame<RADIX, T> {

    #[inline(always)]
    fn get_frame_mask(&self, model_index: usize) -> u64 {
        (1 << self.frame_sizes[model_index]) - 1
    }

    #[inline(always)]
    fn get_log2_frame_size(&self, model_index: usize) -> usize {
        self.frame_sizes[model_index]
    }
}

impl<const RADIX: usize, T: Quasi<RADIX>> SymbolLookup<State> for VecFrame<RADIX, T> {
    type Output = DecoderModelEntry<RADIX, T>;

    #[inline(always)]
    fn symbol(&self, slot: State, model_index: usize) -> &Self::Output {
        &self.symbols[model_index][slot as usize]
    }
}
