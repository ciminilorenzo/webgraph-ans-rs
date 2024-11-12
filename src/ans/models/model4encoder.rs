use crate::ans::models::component_model4encoder::{ANSComponentModel4Encoder, EncoderModelEntry};
use crate::bvgraph::BVGraphComponent;
use crate::RawSymbol;

/// The main and unique model used by the ANS encoder to encode symbols of every [component](BVGraphComponent). Every
/// [component](BVGraphComponent) has its own [model](ANSComponentModel4Encoder) that is used to encode its symbols.
#[derive(Clone)]
pub struct ANSModel4Encoder {
    /// The whole set of [models](ANSComponentModel4Encoder) used by the ANS encoder, one for each
    /// [component](BVGraphComponent).
    pub component_models: Vec<ANSComponentModel4Encoder>,
}

impl ANSModel4Encoder {
    /// Returns a list of tuples, each containing the fidelity and radix of each [component](BVGraphComponent).
    pub fn get_folding_params(&self) -> Vec<(usize, usize)> {
        self.component_models
            .iter()
            .map(|table| (table.fidelity, table.radix))
            .collect::<Vec<_>>()
    }

    /// Returns the frame mask for the given [component](BVGraphComponent).
    #[inline(always)]
    pub fn get_frame_mask(&self, component: BVGraphComponent) -> u64 {
        (1 << self.component_models[component as usize].frame_size) - 1
    }

    /// Returns the log2 of the frame size for the given [component](BVGraphComponent).
    #[inline(always)]
    pub fn get_log2_frame_size(&self, component: BVGraphComponent) -> usize {
        self.component_models[component as usize].frame_size
    }

    /// Returns the radix for the given [component](BVGraphComponent).
    #[inline(always)]
    pub fn get_radix(&self, component: BVGraphComponent) -> usize {
        self.component_models[component as usize].radix
    }

    /// Returns the fidelity for the given [component](BVGraphComponent).
    #[inline(always)]
    pub fn get_fidelity(&self, component: BVGraphComponent) -> usize {
        self.component_models[component as usize].fidelity
    }

    /// Returns a reference to the [entry](EncoderModelEntry) of the symbol of the
    /// given [component](BVGraphComponent).
    #[inline(always)]
    pub fn symbol(&self, symbol: RawSymbol, component: BVGraphComponent) -> &EncoderModelEntry {
        &self.component_models[component as usize][symbol]
    }

    /// Returns the folding offset for the given [component](BVGraphComponent).
    #[inline(always)]
    pub fn get_folding_offset(&self, component: BVGraphComponent) -> u64 {
        self.component_models[component as usize].folding_offset
    }

    /// Returns the folding threshold for the given [component](BVGraphComponent).
    #[inline(always)]
    pub fn get_folding_threshold(&self, component: BVGraphComponent) -> u64 {
        self.component_models[component as usize].folding_threshold
    }
}
