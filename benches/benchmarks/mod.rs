pub mod decoder;
pub mod encoder;
pub mod model_for_encoder;
pub mod model_for_decoder;

/// Size of the list of symbols used to bench.
const SYMBOL_LIST_LENGTH: usize = 500_000;

/// Maximum value that the zpfian distribution can output.
const MAXIMUM_SYMBOL: u64 = 900_000_000;