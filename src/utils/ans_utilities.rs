use crate::{RawSymbol, Symbol};

/// Folds a symbol without streaming out the bits.
pub fn fold_without_streaming_out(mut sym: RawSymbol, radix: usize, fidelity: usize) -> Symbol {
    let mut offset = 0;
    let cuts = (((u64::ilog2(sym) as usize) + 1) - fidelity) / radix;
    let bit_to_cut = cuts * radix;
    sym >>= bit_to_cut;
    offset += (((1 << radix) - 1) * (1 << (fidelity - 1))) * cuts as RawSymbol;

    u16::try_from(sym + offset).expect("Folded symbol is bigger than u16::MAX")
}

pub fn get_reciprocal_data(divisor: u16) -> (u64, u8) {
    let m = divisor.ilog2();

    match divisor.is_power_of_two() {
        true => (u64::MAX, ((m as u8) << 1) + 1u8),

        false => {
            let t = ((1u128 << (m + 64)) / divisor as u128) as u64;
            let r = (divisor as u128 * (t as u128 + 1) - (1u128 << (m + 64))) as u64;

            match r <= (1u64 << m) {
                true => (t + 1, (m as u8) << 1),
                false => (t, ((m as u8) << 1) + 1_u8),
            }
        }
    }
}
