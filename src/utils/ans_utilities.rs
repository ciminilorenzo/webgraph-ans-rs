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

#[cfg(test)]
mod tests {
    // http://acsel-lab.com/arithmetic/arith17/papers/ARITH17_Robison.pdf

    use rand::{thread_rng, Rng};

    struct Reciprocal {
        a: u64,
        b: u64,
        shift: u16,
    }

    /// This function returns the parameters a, b and m, that is the value used to get the same result
    /// we would get by dividing any number by the divisor if a multiplication is performed.
    fn get_multiplication_parameters(divisor: u16) -> Reciprocal {
        let m = u16::ilog2(divisor);

        match divisor.is_power_of_two() {
            // If the divisor (d) is power of two, then:
            // a := 2^n - 1 = 2^64 - 1;
            // b := 2^n - 1 = 2^64 - 1;
            // m := log_{2}(d)
            // finally:
            // floor(x / d)  = floor((ax + b)) / 2^n) / 2^m
            true => Reciprocal {
                a: u64::MAX,
                b: u64::MAX,
                shift: m as u16,
            },
            // else:
            // t := floor(2^m+n / d);
            // r := (d * (t - 1)) - 2^m+n;
            // if r <= 2^m then:
            // a := t + 1; -> rounding up the reciprocal
            // b := 0;
            // else:
            // a := t; -> rounding down the reciprocal
            // b := t;
            false => {
                let t = ((1u128 << m + 64) / divisor as u128) as u64;
                let r = (divisor as u128 * (t as u128 + 1) - (1u128 << (m + 64))) as u64;

                match r <= (1u64 << m) {
                    true => Reciprocal {
                        a: t + 1,
                        b: 0,
                        shift: m as u16,
                    },
                    false => Reciprocal {
                        a: t,
                        b: t,
                        shift: m as u16,
                    },
                }
            }
        }
    }

    #[test]
    fn test_division() {
        for _ in 0..1000000 {
            let dividend = thread_rng().gen_range(1..u64::MAX);
            let divisor = thread_rng().gen_range(1..u16::MAX);
            let reciprocal = get_multiplication_parameters(divisor);

            // floor(x / d) = ((ax + b) / 2^n) / 2^m
            let result = ((reciprocal.a as u128 * dividend as u128 + reciprocal.b as u128) >> 64)
                as u64
                >> reciprocal.shift;

            assert_eq!(dividend / divisor as u64, result,)
        }
    }
}
