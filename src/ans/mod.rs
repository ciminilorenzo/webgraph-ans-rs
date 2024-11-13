//! Contains the whole implementation of the ANS-compressor.
//!

pub mod decoder;
pub mod encoder;
pub mod model4encoder_builder;
pub mod models;

use epserde::Epserde;

use mem_dbg::{MemDbg, MemSize};

use crate::ans::models::component_model4encoder::ANSComponentModel4Encoder;
use crate::State;

/// The same parameter described in Duda's [paper](https://arxiv.org/pdf/0902.0271). In this case
/// we store the logarithm (base 2) of this value.
pub const B: usize = 16;

/// The lower bound of the interval used by the compressor.
pub const INTERVAL_LOWER_BOUND: State = 1 << 16;

/// Used to extract the 16 LSB from a 32-bit state.
pub const NORMALIZATION_MASK: u32 = 0xFFFF;

/// The maximum frame size allowed for any of the [models](ANSComponentModel4Encoder) used by
/// the ANS encoder.
const MAXIMUM_FRAME_SIZE: usize = 1 << 16;

/// The main container for all the essential data needed by the decoder.
#[derive(Clone, Debug, Epserde, MemDbg, MemSize)]
pub struct Prelude {
    /// The list of [`ANSComponentModel4Encoder`], one for each component, used by
    /// the encoder.
    pub tables: Vec<ANSComponentModel4Encoder>,

    /// The normalized bits during the encoding process.
    pub stream: Vec<u16>,

    /// The final state of the encoder.
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

/// Represents the state of the compressor in a specific moment, that is: the current value of the
/// encoder state and the current index of the stream where inserting extracted bits.
#[derive(Debug, Clone, Copy, Epserde, MemDbg, MemSize)]
#[zero_copy]
#[repr(C)]
pub struct ANSCompressorPhase {
    pub state: State,
    pub stream_pointer: usize,
}