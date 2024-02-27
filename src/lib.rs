#![allow(unused_must_use)]
#![allow(dead_code)]

pub mod ans;
pub mod bvgraph;
pub mod utils;

mod traits;

/// The parameter with the same name in Duda's paper. In this case we store the logarithm of the
/// parameter since, if b = 2^k, we extract/insert k bits from the state at once.
///
/// Having said that, in this project b is fixed to be 32 in order to extract/insert 32 bits from/to
/// the state at once.
pub const B: usize = 16;

/// The maximum symbol that can be encoded/decoded.
pub const MAX_RAW_SYMBOL: u64 = (1 << 48) - 1;

/// The lower end of the interval in which the state of the compressor can stay.
pub const INTERVAL_LOWER_BOUND: State = 1 << 16;

/// The type representing an encoded symbols, that could have been either folded or not.
///
/// # Note
/// This implementation assumes that the maximum symbol is u16::MAX. If more symbols are present,
/// RADIX and FIDELITY should be changed since ANS gets worse with a lot of symbols.
///
/// Moreover, since most of the DS used within the project are tables where symbols data is located
/// in the index equal to the symbol, this type can be interpreted as the maximum symbol index we can
/// have
pub type Symbol = u16;

/// The type representing the raw symbols, i.e. the symbols coming from the input.
pub type RawSymbol = u64;

/// The type representing the state of the encoder/decoder.
pub type State = u32;

/// The type representing the frequencies of the symbols. This type is bounded to be u16 since we
/// deliberately accept to have frequencies that can reach at most this value. This is done in order
/// to have entries for the decoder that have both the frequency and cumulated frequency of each
/// symbol as 16-bit unsigned.
pub type Freq = u16;

/// Used to extract the 16 LSB from a 32-bit state.
pub const NORMALIZATION_MASK: u32 = 0xFFFF;
