use std::ops::Div;

use criterion::{black_box, criterion_group, Criterion};
use pprof::criterion::{Output, PProfProfiler};
use rand::{rngs::SmallRng, RngCore, SeedableRng};

#[derive(Clone, Debug)]
struct DoubleAdd {
    a: u32,
    b: u32,
    shift: u8,
}

impl DoubleAdd {
    fn new(divisor: u16) -> DoubleAdd {
        let m = u16::ilog2(divisor);

        match divisor.is_power_of_two() {
            // If the divisor (d) is power of two, then:
            // a := 2^n - 1 = 2^64 - 1;
            // b := 2^n - 1 = 2^64 - 1;
            // m := log_{2}(d)
            // finally:
            // floor(x / d)  = floor((ax + b)) / 2^n) / 2^m
            true => DoubleAdd {
                a: u32::MAX,
                b: u32::MAX,
                shift: m as u8,
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
                let t = ((1u64 << m + 64) / divisor as u64) as u32;
                let r = (divisor as u64 * (t as u64 + 1) - (1u64 << (m + 32))) as u32;

                match r <= (1u32 << m) {
                    true => DoubleAdd {
                        a: t + 1,
                        b: 0,
                        shift: m as u8,
                    },
                    false => DoubleAdd {
                        a: t,
                        b: t,
                        shift: m as u8,
                    },
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct MulShift {
    a: u32,
    mul: u8,
    shift: u8,
}

impl MulShift {
    fn new(divisor: u16) -> MulShift {
        let m = divisor.ilog2();

        match divisor.is_power_of_two() {
            // If the divisor (d) is power of two, then:
            // a := 2^n - 1 = 2^64 - 1;
            // b := 2^n - 1 = 2^64 - 1;
            // m := log_{2}(d)
            // finally:
            // floor(x / d)  = floor((ax + b)) / 2^n) / 2^m
            true => MulShift {
                a: u32::MAX,
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
                let t = ((1u64 << m + 32) / divisor as u64) as u32;
                let r = (divisor as u64 * (t as u64 + 1) - (1u64 << (m + 32))) as u32;

                match r <= (1u32 << m) {
                    true => MulShift {
                        a: t + 1,
                        shift: m as u8,
                        mul: 0,
                    },
                    false => MulShift {
                        a: t,
                        shift: m as u8,
                        mul: 1,
                    },
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct WithMagic {
    a: u32,
    magic: u8,
}

/// This function returns the parameters a, b and m, that is the value used to get the same result
/// we would get by dividing any number by the divisor if a multiplication is performed.

impl WithMagic {
    fn new(divisor: u16) -> WithMagic {
        let m = divisor.ilog2();

        match divisor.is_power_of_two() {
            // If the divisor (d) is power of two, then:
            // a := 2^n - 1 = 2^64 - 1;
            // b := 2^n - 1 = 2^64 - 1;
            // m := log_{2}(d)
            // finally:
            // floor(x / d)  = floor((ax + b)) / 2^n) / 2^m
            true => WithMagic {
                a: u32::MAX,
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
                let t = ((1u64 << m + 32) / divisor as u64) as u32;
                let r = (divisor as u64 * (t as u64 + 1) - (1u64 << (m + 32))) as u32;

                match r <= (1u32 << m) {
                    true => WithMagic {
                        a: t + 1,
                        magic: (m as u8) << 1,
                    },
                    false => WithMagic {
                        a: t,
                        magic: ((m as u8) << 1) + 1_u8,
                    },
                }
            }
        }
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("div");
    let mut r = SmallRng::seed_from_u64(0);
    let n = 1024;
    if let Some(core_ids) = core_affinity::get_core_ids() {
        // Not core 0. Anything goes.
        let core_id = core_ids[1];
        if !core_affinity::set_for_current(core_id) {
            eprintln!("Cannot pin thread to core {:?}", core_id);
        }
    } else {
        eprintln!("Cannot retrieve core ids");
    }

    let mut dividends = vec![];
    dividends.extend((0..n).map(|_| r.next_u64()));

    let mut divisors = vec![];
    divisors.extend((0..n).map(|_| (r.next_u32() + 1) as u16));

    let double_adds = divisors
        .iter()
        .map(|&x| DoubleAdd::new(x))
        .collect::<Vec<_>>();

    group.bench_function("Double Add", |b| {
        let mut i = 0;
        b.iter(|| {
            let dividend = dividends[i];
            let double_add = double_adds[i].clone();
            black_box(
                ((black_box(double_add.a) as u64 * black_box(dividend) as u64
                    + black_box(double_add.b) as u64)
                    >> 32) as u32
                    >> black_box(double_add.shift),
            );

            i = (i + 1) % n;
        });
    });

    let mul_shifts = divisors
        .iter()
        .map(|&x| MulShift::new(x))
        .collect::<Vec<_>>();

    group.bench_function("Mul + Shift (test)", |b| {
        let mut i = 0;
        b.iter(|| {
            let dividend = dividends[i];
            let mul_shift = mul_shifts[i].clone();
            black_box(if black_box(mul_shift.mul) == 0 {
                (black_box(mul_shift.a) as u64 * black_box(dividend) as u64 >> 32) as u32
                    >> black_box(mul_shift.shift)
            } else {
                (black_box(mul_shift.a) as u64 * (black_box(dividend) as u64 + 1) >> 32) as u32
                    >> black_box(mul_shift.shift)
            });
            i = (i + 1) % n;
        });
    });

    group.bench_function("Mul + Shift (arithmetized)", |b| {
        let mut i = 0;
        b.iter(|| {
            let dividend = dividends[i];
            let mul_shift = mul_shifts[i].clone();
            black_box(
                (black_box(mul_shift.a) as u64
                    * (black_box(dividend) as u64 + black_box(mul_shift.mul) as u64)
                    >> 32) as u32
                    >> black_box(mul_shift.shift),
            );
            i = (i + 1) % n;
        });
    });

    let with_magics = divisors
        .iter()
        .map(|&x| WithMagic::new(x))
        .collect::<Vec<_>>();

    group.bench_function("Mul + Shift (with magic)", |b| {
        let mut i = 0;
        b.iter(|| {
            let dividend = dividends[i];
            let with_magic = with_magics[i].clone();
            black_box(
                (black_box(with_magic.a) as u64
                    * (black_box(dividend) as u64
                        + black_box(black_box(with_magic.magic) & 1_u8) as u64)
                    >> 32) as u32
                    >> black_box(black_box(with_magic.magic) >> 1),
            );
            i = (i + 1) % n;
        });
    });

    group.bench_function("division", |b| {
        let mut i = 0;
        b.iter(|| {
            let dividend = dividends[i];
            let divisor = divisors[i];
            black_box(black_box(dividend) / black_box(divisor) as u64);
            i = (i + 1) % n;
        });
    });
}

criterion_group! {
    name = div_benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_benchmark
}
