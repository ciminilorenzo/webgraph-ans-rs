#![allow(unused_must_use)]

#![doc = include_str!("../README.md")]

pub mod ans;
pub mod bvgraph;
mod utils;
mod traits;

/// The type representing encoded symbols, which could have been either folded or not.
pub type Symbol = u16;

/// The type representing the raw symbols, that is the original symbols to encode before the folding
/// is applied, if needed.
pub type RawSymbol = u64;

/// The biggest [`RawSymbol`] encodable by the compressor.
pub const MAX_RAW_SYMBOL: u64 = (1 << 48) - 1;

/// The type representing the state of the compressor.
pub type State = u32;

/// The type representing the frequencies of the symbols.
pub type Freq = u16;
