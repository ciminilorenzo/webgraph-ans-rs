use criterion::{black_box, criterion_group, Criterion};
use pprof::criterion::{Output, PProfProfiler};

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

struct Reciprocal3 {
    a: u64,
    magic: u8,
}

/// This function returns the parameters a, b and m, that is the value used to get the same result
/// we would get by dividing any number by the divisor if a multiplication is performed.
fn get_multiplication_parameters3(divisor: u16) -> Reciprocal3 {
    let m = divisor.ilog2();

    match divisor.is_power_of_two() {
        // If the divisor (d) is power of two, then:
        // a := 2^n - 1 = 2^64 - 1;
        // b := 2^n - 1 = 2^64 - 1;
        // m := log_{2}(d)
        // finally:
        // floor(x / d)  = floor((ax + b)) / 2^n) / 2^m
        true => Reciprocal3 {
            a: u64::MAX,
            magic: ((m as u8) << 1) + 1_u8,
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
                true => Reciprocal3 {
                    a: t + 1,
                    magic: (m as u8) << 1,
                },
                false => Reciprocal3 {
                    a: t,
                    magic: ((m as u8) << 1) + 1_u8,
                },
            }
        }
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("div");
    let dividend = 0xdeadbeefdeadf00d_u64;

    let reciprocal = get_multiplication_parameters(4242);

    group.bench_function("double_add", |b| {
        b.iter(|| {
            black_box(
                ((black_box(reciprocal.a) as u128 * black_box(dividend) as u128
                    + black_box(reciprocal.b) as u128)
                    >> 64) as u64
                    >> black_box(reciprocal.shift),
            )
        });
    });

    let reciprocal = get_multiplication_parameters2(4242);
    group.bench_function("arithmetized", |b| {
        b.iter(|| {
            black_box(
                (black_box(reciprocal.a) as u128
                    * (black_box(dividend) as u128 + black_box(reciprocal.mul) as u128)
                    >> 64) as u64
                    >> black_box(reciprocal.shift),
            )
        });
    });

    group.bench_function("test", |b| {
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

    let reciprocal3 = get_multiplication_parameters3(4242);

    group.bench_function("with magic", |b| {
        b.iter(|| {
            black_box(
                (black_box(reciprocal3.a) as u128
                    * (black_box(dividend) as u128 + black_box(black_box(reciprocal3.magic) & 1_u8) as u128)
                    >> 64) as u64
                    >> black_box(black_box(reciprocal3.magic) >> 1),
            )
        });
    });

    group.bench_function("division", |b| {
        b.iter(|| black_box(black_box(dividend) / black_box(4242_u64)));
    });
}

criterion_group! {
    name = div_benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_benchmark
}
