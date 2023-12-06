pub mod ans;
pub mod utils;

pub const K: usize = 16;
pub const K_LOG2: usize = 4;

/// How big M (the frame) can be. This constrained is imposed by the fact that B and K are fixed and
/// State is a u64.
pub const MAXIMUM_LOG2_M: usize = 28;

/// How many bits are extracted/added from/to the state during the encoding/decoding process.
pub const LOG2_B: usize = 32;

/// The type representing the folded symbols.
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
pub type State = u64;

/// Both `freq` and `cumul_freq` are u32 since M can be at most 2^28.
pub type Freq = u32;
