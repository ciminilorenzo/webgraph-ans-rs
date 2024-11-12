pub mod decoder;
pub mod encoder;
pub mod model4encoder_builder;
pub mod models;

use epserde::Epserde;

use mem_dbg::{MemDbg, MemSize};

use crate::ans::models::component_model4encoder::ANSComponentModel4Encoder;
use crate::State;

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

    /// the maximum distance between two nodes that reference each other.
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

/// Represents the state of the compressor in a specific moment, that is: the current value of the
/// encoder state and the current index of the stream where inserting extracted bits.
#[derive(Debug, Clone, Copy, Epserde, MemDbg, MemSize)]
#[zero_copy]
#[repr(C)]
pub struct ANSCompressorPhase {
    pub state: State,
    pub stream_pointer: usize,
}
