use std::ops::Index;

use sucds::bit_vectors::{Rank, Rank9Sel};

use sux::dict::{EliasFano, EliasFanoBuilder};
use sux::rank_sel::QuantumIndex;
use sux::traits::{ConvertTo, Pred};

use crate::ans::EncoderModelEntry;
use crate::{State, Symbol};

#[readonly::make]
#[derive(Clone, Debug, Default)]
pub struct DecoderModelEntry {
    pub symbol: Symbol,
    pub freq: u32,
    pub cumul_freq: u32,
}


pub struct EliasFanoFrame {

    /// Contains, in each position, the data associated to the symbol in the same position within the EliasFano structure.
    symbols: Vec<DecoderModelEntry>,

    /// The mapped frame as an Elias-Fano structure.
    frame: EliasFano
}

impl EliasFanoFrame {

    pub fn new(table: &[EncoderModelEntry], log2_frame_size: u8) -> Self {
        assert!(table.len() < 1 << Symbol::BITS, "Can't have more than u16::MAX symbols");

        let nonzero_symbols = table.iter().filter(|sym| sym.freq > 0).count();

        let mut symbols = Vec::with_capacity(nonzero_symbols);
        let mut frame_builder = EliasFanoBuilder::new(nonzero_symbols + 1, (1 << log2_frame_size) + 1);

        for (sym, sym_data) in table.iter().enumerate() {
            if sym_data.freq == 0 { continue; }

            frame_builder.push(sym_data.cumul_freq as usize).unwrap();
            symbols.push(DecoderModelEntry {
                symbol: sym as Symbol,
                freq: sym_data.freq,
                cumul_freq: sym_data.cumul_freq
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

impl Index<State> for EliasFanoFrame {
    type Output = DecoderModelEntry;

    fn index(&self, slot: State) -> &Self::Output {
        let symbol_index = unsafe { self.frame.pred_unchecked::<false>(&(slot as usize)).0 as Symbol };
        &self.symbols[symbol_index as usize]
    }
}


// Petri's solution. Inserts in each slot the data associated to the corresponding symbol. This
// solution is not space-efficient, but it's the fastest one.
#[derive(Clone)]
pub struct VecFrame(Vec<DecoderModelEntry>);

impl VecFrame {

    /// Creates a new VecFrame from the given table.
    pub fn new(table: &[EncoderModelEntry], log2_frame_size: u8) -> Self {
        assert!(table.len() < 1 << Symbol::BITS, "Can't have more than u16::MAX symbols");

        let mut vec = vec![DecoderModelEntry::default(); 1 << log2_frame_size];
        let mut last_slot = 0;
        let mut index = 0;

        for symbol in 0..table.len() {
            match table[index] {
                EncoderModelEntry {freq: 0, ..} => {
                    // let's skip symbols with frequency 0
                    index += 1;
                    continue;
                },
                EncoderModelEntry{ freq, upperbound: _ , cumul_freq} => {
                    for i in last_slot.. last_slot + freq {
                        unsafe {
                            let entry = vec.get_unchecked_mut(i as usize);
                            entry.symbol = symbol as Symbol;
                            entry.freq = freq;
                            entry.cumul_freq = cumul_freq;
                        }
                    }
                    index += 1;
                    last_slot += freq;
                }
            }
        }
        VecFrame(vec)
    }
}

impl Index<State> for VecFrame {
    type Output = DecoderModelEntry;

    fn index(&self, slot: State) -> &Self::Output {
        &self.0[slot as usize]
    }
}


#[derive(Clone)]
pub struct Rank9SelFrame {

    /// Contains, in each position, the data associated to the symbol in the same position within the Rank9Sel structure.
    symbols: Vec<DecoderModelEntry>,

    frame: Rank9Sel,
}

impl Rank9SelFrame {

    pub fn new(table: &[EncoderModelEntry], log2_frame_size: u8) -> Self {
        assert!(table.len() < 1 << Symbol::BITS, "Too many symbols");

        let nonzero_symbols = table.iter().filter(|sym| sym.freq > 0).count();
        let mut symbols = Vec::with_capacity(nonzero_symbols);
        let mut vec = vec![false; 1 << log2_frame_size];

        for (sym, sym_data) in table.iter().enumerate() {
            if sym_data.freq == 0 { continue; }

            match sym_data.cumul_freq {
                0 => (),
                _ => {
                    let bit = vec.get_mut(sym_data.cumul_freq as usize).unwrap();
                    *bit = true;
                }
            }
            symbols.push(DecoderModelEntry {symbol: sym as Symbol, freq: sym_data.freq, cumul_freq: sym_data.cumul_freq});
        }

        Self {
            symbols,
            frame: Rank9Sel::from_bits(vec),
        }
    }
}

impl Index<State> for Rank9SelFrame {
    type Output = DecoderModelEntry;

    fn index(&self, slot: State) -> &Self::Output {
        let symbol_index = self.frame.rank1((slot + 1) as usize).unwrap() as Symbol;
        &self.symbols[symbol_index as usize]
    }
}