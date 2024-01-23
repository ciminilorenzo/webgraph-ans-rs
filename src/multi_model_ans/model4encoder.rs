use crate::multi_model_ans::{EncoderModelEntry};
use crate::{Freq, Symbol};
use crate::bvgraph::BVGraphComponent;

pub trait SymbolLookup<Idx> {
    type Output;

    fn symbol(&self, data: Idx, component: BVGraphComponent) -> &Self::Output;
}

#[derive(Clone)]
pub struct ANSModel4Encoder {
    /// Contains a vec of entries for each model where, inside each vec, each index contains the data
    /// related to the symbol equal to that index.
    pub tables: Vec<Vec<EncoderModelEntry>>,

    /// Contains the log2 of the frame size for each model.
    pub frame_sizes: Vec<usize>,
}

impl ANSModel4Encoder {
    pub fn get_symbol_freqs(&self) -> Vec<Vec<Freq>> {
        self
            .tables
            .iter()
            .map(|x| x.iter().map(|y| y.freq).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    }

    #[inline(always)]
    pub fn get_frame_mask(&self, component: BVGraphComponent) -> u64 {
        (1 << self.frame_sizes[component as usize]) - 1
    }

    #[inline(always)]
    pub fn get_log2_frame_size(&self, component: BVGraphComponent) -> usize {
        self.frame_sizes[component as usize]
    }
}

impl SymbolLookup<Symbol> for ANSModel4Encoder {
    type Output = EncoderModelEntry;

    #[inline(always)]
    fn symbol(&self, symbol: Symbol, component: BVGraphComponent) -> &Self::Output {
        &self.tables[component as usize][symbol as usize]
    }
}
