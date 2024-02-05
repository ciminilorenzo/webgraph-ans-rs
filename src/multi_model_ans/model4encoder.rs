use epserde::Epserde;
use mem_dbg::{MemDbg, MemSize};
use std::ops::Index;

use crate::bvgraph::BVGraphComponent;
use crate::multi_model_ans::EncoderModelEntry;
use crate::{Freq, Symbol};

#[derive(Clone, MemDbg, MemSize, Epserde, Debug)]
/// The ANS model of a specific [component](BVGraphComponent) used by the encoder to encode its symbols.
pub struct ANSComponentModel4Encoder {
    /// A table containing, at each index, the data related to the symbol equal to that index.
    pub table: Vec<EncoderModelEntry>,

    /// The log2 of the frame size for this [`BVGraphComponent`](component).
    pub frame_size: usize,

    /// The radix used by the current model.
    pub radix: usize,

    /// The fidelity used by the current model.
    pub fidelity: usize,

    /// The threshold representing the symbol from which we have to start folding, based on the current fidelity and radix.
    pub folding_threshold: u64,

    pub folding_offset: u64,
}

impl ANSComponentModel4Encoder {
    /// Returns the frequencies of each symbol in the [model](ANSComponentModel4Encoder).
    pub fn get_freqs(&self) -> Vec<Freq> {
        self.table
            .iter()
            .map(|symbol| symbol.freq)
            .collect::<Vec<_>>()
    }
}

impl Default for ANSComponentModel4Encoder {
    fn default() -> Self {
        Self {
            table: Vec::new(),
            frame_size: 0,
            radix: 2,
            fidelity: 2,
            folding_threshold: 10,
            folding_offset: 10,
        }
    }
}

impl Index<Symbol> for ANSComponentModel4Encoder {
    type Output = EncoderModelEntry;

    #[inline(always)]
    fn index(&self, symbol: Symbol) -> &Self::Output {
        &self.table[symbol as usize]
    }
}

/// The main and unique model used by the ANS encoder to encode symbols of every [component](BVGraphComponent). Every
/// [component](BVGraphComponent) has its own [model](ANSComponentModel4Encoder) that is used to encode its symbols.
#[derive(Clone)]
pub struct ANSModel4Encoder {
    /// The whole set of [models](ANSComponentModel4Encoder) used by the ANS encoder, one for each
    /// [component](BVGraphComponent).
    pub models: Vec<ANSComponentModel4Encoder>,
}

impl ANSModel4Encoder {
    pub fn get_folding_params(&self) -> Vec<(usize, usize)> {
        self.models
            .iter()
            .map(|table| (table.fidelity, table.radix))
            .collect::<Vec<_>>()
    }

    /// Returns the frame mask for the given [component](BVGraphComponent).
    #[inline(always)]
    pub fn get_frame_mask(&self, component: BVGraphComponent) -> u64 {
        (1 << self.models[component as usize].frame_size) - 1
    }

    /// Returns the log2 of the frame size for the given [component](BVGraphComponent).
    #[inline(always)]
    pub fn get_log2_frame_size(&self, component: BVGraphComponent) -> usize {
        self.models[component as usize].frame_size
    }

    /// Returns the radix for the given [component](BVGraphComponent).
    #[inline(always)]
    pub fn get_radix(&self, component: BVGraphComponent) -> usize {
        self.models[component as usize].radix
    }

    /// Returns the fidelity for the given [component](BVGraphComponent).
    #[inline(always)]
    pub fn get_fidelity(&self, component: BVGraphComponent) -> usize {
        self.models[component as usize].fidelity
    }

    /// Returns a reference to the [entry](EncoderModelEntry) of the symbol
    #[inline(always)]
    pub fn symbol(&self, symbol: Symbol, component: BVGraphComponent) -> &EncoderModelEntry {
        &self.models[component as usize][symbol]
    }

    /// Returns the folding offset for the given [component](BVGraphComponent).
    #[inline(always)]
    pub fn get_folding_offset(&self, component: BVGraphComponent) -> u64 {
        self.models[component as usize].folding_offset
    }

    /// Returns the folding threshold for the given [component](BVGraphComponent).
    #[inline(always)]
    pub fn get_folding_threshold(&self, component: BVGraphComponent) -> u64 {
        self.models[component as usize].folding_threshold
    }

    /// Returns a list of tuples containing the fidelity and radix of each [component](BVGraphComponent).
    pub fn get_component_args(&self) -> Vec<(usize, usize)> {
        self.models
            .iter()
            .map(|table| (table.fidelity, table.radix))
            .collect::<Vec<_>>()
    }
}
