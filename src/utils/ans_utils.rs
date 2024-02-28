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

pub fn get_reciprocal_data(divisor: u16) -> (u32, u8) {
    let m = divisor.ilog2();

    match divisor.is_power_of_two() {
        true => (u32::MAX, ((m as u8) << 1) + 1u8),

        false => {
            let t = ((1u64 << (m + 32)) / divisor as u64) as u32;
            let r = (divisor as u64 * (t as u64 + 1) - (1u64 << (m + 32))) as u32;

            match r <= (1u32 << m) {
                true => (t + 1, (m as u8) << 1),
                false => (t, ((m as u8) << 1) + 1_u8),
            }
        }
    }
}
