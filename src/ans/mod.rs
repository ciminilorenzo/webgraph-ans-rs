pub mod decoder;
pub mod encoder;
pub mod model4decoder;
pub mod model4encoder;
pub mod model4encoder_builder;

use crate::ans::model4encoder::ANSComponentModel4Encoder;
use crate::{Freq, State, B};
use epserde::Epserde;
use mem_dbg::{MemDbg, MemSize};

#[derive(Clone, Debug, Epserde, MemDbg, MemSize)]
pub struct Prelude {
    /// Contains, for each index, the data associated to the symbol equal to that index.
    pub tables: Vec<ANSComponentModel4Encoder>,

    /// The normalized bits during the encoding process.
    pub stream: Vec<u16>,

    /// The state of the encoder after having encoded the last symbol of the input.
    pub state: State,

    /// The number of nodes in the graph.
    pub number_of_nodes: usize,

    /// the maximum distance between two nodes that ∂∂reference each other.
    pub compression_window: usize,

    /// The minimum size of the intervals we are going to decode.
    pub min_interval_length: usize,

    /// The number of arcs in the graph.
    pub number_of_arcs: u64,
}

impl Prelude {
    pub fn new(
        tables: Vec<ANSComponentModel4Encoder>,
        stream: Vec<u16>,
        state: State,
        number_of_nodes: usize,
        number_of_arcs: u64,
        compression_window: usize,
        min_interval_length: usize,
    ) -> Self {
        Self {
            tables,
            stream,
            state,
            number_of_nodes,
            number_of_arcs,
            compression_window,
            min_interval_length,
        }
    }
}

#[derive(Debug, Clone, Copy, Epserde, MemDbg, MemSize)]
#[zero_copy]
#[repr(C)]
pub struct ANSCompressorPhase {
    pub state: State,
    pub stream_pointer: usize,
}

#[derive(Clone, Copy, Debug, Epserde, MemDbg, MemSize)]
#[repr(C)]
#[zero_copy]
pub struct EncoderModelEntry {
    /// The upperbound of the symbol, that is the maximum value starting from which we can safely encode this specific
    /// symbol without overflowing the interval in which the state of the compressor can be.
    pub upperbound: u32,

    /// The cumulative frequency of the symbol.
    pub cumul_freq: Freq,

    /// The frequency of the symbol.
    pub freq: Freq,
}

impl EncoderModelEntry {
    pub fn new(freq: u16, k: usize, cumul: Freq) -> Self {
        Self {
            freq,
            upperbound: (1_u32 << (k + B)) * freq as State,
            cumul_freq: cumul,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Epserde)]
#[repr(C)]
#[zero_copy]
pub struct DecoderModelEntry {
    /// The frequency of the symbol.
    pub freq: Freq,

    /// The cumulative frequency of the symbol.
    pub cumul_freq: Freq,

    /// A 64-bit integer, containing the number of folds that we need to successively unfold to get the raw symbol back
    /// in the most significant 16 bits, and the quasi-folded symbol in the least significant 48 bits.
    pub quasi_folded: u64,
}
