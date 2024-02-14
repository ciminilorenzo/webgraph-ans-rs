use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use std::time::Duration;

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

struct Reciprocal2 {
    a: u64,
    shift: u8,
    mul: u8,
}

/// This function returns the parameters a, b and m, that is the value used to get the same result
/// we would get by dividing any number by the divisor if a multiplication is performed.
fn get_multiplication_parameters2(divisor: u16) -> Reciprocal2 {
    let m = divisor.ilog2();

    match divisor.is_power_of_two() {
        // If the divisor (d) is power of two, then:
        // a := 2^n - 1 = 2^64 - 1;
        // b := 2^n - 1 = 2^64 - 1;
        // m := log_{2}(d)
        // finally:
        // floor(x / d)  = floor((ax + b)) / 2^n) / 2^m
        true => Reciprocal2 {
            a: u64::MAX,
            shift: m as u8,
            mul: 1,
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
                true => Reciprocal2 {
                    a: t + 1,
                    shift: m as u8,
                    mul: 0,
                },
                false => Reciprocal2 {
                    a: t,
                    shift: m as u8,
                    mul: 1,
                },
            }
        }
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let dividend = 0xdeadbeefdeadf00d_u64;

    let reciprocal = get_multiplication_parameters(4242);

    c.bench_function("double_add", |b| {
        b.iter(|| {
            black_box(
                ((black_box(reciprocal.a) as u128 * black_box(dividend) as u128
                    + reciprocal.b as u128)
                    >> 64) as u64
                    >> black_box(reciprocal.shift),
            )
        });
    });

    let reciprocal = get_multiplication_parameters2(4242);
    c.bench_function("arithmetized", |b| {
        b.iter(|| {
            black_box(
                (black_box(reciprocal.a) as u128
                    * (black_box(dividend) as u128 + black_box(reciprocal.mul) as u128)
                    >> 64) as u64
                    >> black_box(reciprocal.shift),
            )
        });
    });

    c.bench_function("test", |b| {
        b.iter(|| {
            black_box(if black_box(reciprocal.mul) == 0 {
                (black_box(reciprocal.a) as u128 * black_box(dividend) as u128 >> 64) as u64
                    >> black_box(reciprocal.shift)
            } else {
                (black_box(reciprocal.a) as u128 * (black_box(dividend) as u128 + 1) >> 64) as u64
                    >> black_box(reciprocal.shift)
            })
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(3));
    targets = criterion_benchmark
}
criterion_main!(benches);
