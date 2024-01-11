use crate::{EncoderModelEntry, Symbol};
use crate::traits::quasi::Decode;


pub trait SymbolLookup<Idx> {
    type Output;

    fn symbol(&self, data: Idx, model_index: usize) -> &Self::Output;
}

#[derive(Clone)]
pub struct AnsModel4Encoder {

    /// Contains a vec of entries for each model where, inside each vec, each index contains the data
    /// related to the symbol equal to that index.
    pub tables: Vec<Vec<EncoderModelEntry>>,

    /// Contains the log2 of the frame size for each model.
    pub frame_sizes: Vec<usize>,
}

impl Decode for AnsModel4Encoder {
    #[inline(always)]
    fn get_frame_mask(&self, model_index: usize) -> u64 {
        (1 << self.frame_sizes[model_index]) - 1
    }

    #[inline(always)]
    fn get_log2_frame_size(&self, model_index: usize) -> usize {
        self.frame_sizes[model_index]
    }
}

impl SymbolLookup<Symbol> for AnsModel4Encoder {
    type Output = EncoderModelEntry;

    #[inline(always)]
    fn symbol(&self, symbol: Symbol, model_index: usize) -> &Self::Output {
        &self.tables[model_index][symbol as usize]
    }
}