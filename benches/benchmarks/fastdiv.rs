use std::num::{NonZeroU16, NonZeroU32};

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
                let t = ((1u64 << m + 32) / divisor as u64) as u32;
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

    #[inline(always)]
    pub fn div(&self, dividend: u32) -> u32 {
        ((self.a as u64 * dividend as u64 + self.b as u64) >> 32) as u32 >> self.shift
    }
}

#[derive(Clone, Debug)]
struct MulShiftTwoFields {
    a: u32,
    mul: u8,
    shift: u8,
}

impl MulShiftTwoFields {
    fn new(divisor: u16) -> MulShiftTwoFields {
        let m = divisor.ilog2();

        match divisor.is_power_of_two() {
            true => MulShiftTwoFields {
                a: u32::MAX,
                shift: m as u8,
                mul: 1,
            },
            false => {
                let t = ((1u64 << m + 32) / divisor as u64) as u32;
                let r = (divisor as u64 * (t as u64 + 1) - (1u64 << (m + 32))) as u32;

                match r <= (1u32 << m) {
                    true => MulShiftTwoFields {
                        a: t + 1,
                        shift: m as u8,
                        mul: 0,
                    },
                    false => MulShiftTwoFields {
                        a: t,
                        shift: m as u8,
                        mul: 1,
                    },
                }
            }
        }
    }

    #[inline(always)]
    pub fn div_test(&self, dividend: u32) -> u32 {
        if self.mul == 0 {
            (self.a as u64 * dividend as u64 >> 32) as u32 >> self.shift
        } else {
            (self.a as u64 * (dividend as u64 + 1) >> 32) as u32 >> self.shift
        }
    }

    #[inline(always)]
    pub fn div_arith(&self, dividend: u32) -> u32 {
        (self.a as u64 * (dividend as u64 + self.mul as u64) >> 32) as u32 >> self.shift
    }
}

#[derive(Clone, Debug)]
struct MulShiftOneField {
    a: u32,
    magic: u8, // WRT MulShiftTwoFields, shift << 1 | mul
}

impl MulShiftOneField {
    fn new(divisor: u16) -> MulShiftOneField {
        let m = MulShiftTwoFields::new(divisor);
        MulShiftOneField {
            a: m.a,
            magic: m.shift << 1 | m.mul,
        }
    }

    #[inline(always)]
    pub fn div_test(&self, dividend: u32) -> u32 {
        if self.magic & 1 == 0 {
            (self.a as u64 * dividend as u64 >> 32) as u32 >> (self.magic >> 1)
        } else {
            (self.a as u64 * (dividend as u64 + 1) >> 32) as u32 >> (self.magic >> 1)
        }
    }

    #[inline(always)]
    pub fn div_arith(&self, dividend: u32) -> u32 {
        (self.a as u64 * (dividend as u64 + (self.magic & 1) as u64) >> 32) as u32
            >> (self.magic >> 1)
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
    dividends.extend((0..n).map(|_| r.next_u32()));

    let mut divisors = vec![];
    divisors.extend((0..n).map(|_| (r.next_u32() as u16).max(1)));

    let double_add = divisors
        .iter()
        .map(|&x| DoubleAdd::new(x))
        .collect::<Vec<_>>();

    group.bench_function("no-op", |b| {
        let mut i = 0;
        b.iter(|| {
            black_box(unsafe { double_add.get_unchecked(i) }.a);
            black_box(unsafe { *dividends.get_unchecked(i) });
            i = (i + 1) % n;
        });
    });

    group.bench_function("double_add", |b| {
        let mut i = 0;
        b.iter(|| {
            black_box(
                unsafe { double_add.get_unchecked(i) }.div(unsafe { *dividends.get_unchecked(i) }),
            );
            i = (i + 1) % n;
        });
    });

    let mul_shift_two_fields = divisors
        .iter()
        .map(|&x| MulShiftTwoFields::new(x))
        .collect::<Vec<_>>();

    group.bench_function("mul_shift (two fields, test)", |b| {
        let mut i = 0;
        b.iter(|| {
            black_box(
                unsafe { mul_shift_two_fields.get_unchecked(i) }
                    .div_test(unsafe { *dividends.get_unchecked(i) }),
            );
            i = (i + 1) % n;
        });
    });

    group.bench_function("mul_shift (two fields, arithmetized)", |b| {
        let mut i = 0;
        b.iter(|| {
            black_box(
                unsafe { mul_shift_two_fields.get_unchecked(i) }
                    .div_arith(unsafe { *dividends.get_unchecked(i) }),
            );
            i = (i + 1) % n;
        });
    });

    let mul_shift_one_field = divisors
        .iter()
        .map(|&x| MulShiftOneField::new(x))
        .collect::<Vec<_>>();

    group.bench_function("mul_shift (one field, test)", |b| {
        let mut i = 0;
        b.iter(|| {
            black_box(
                unsafe { mul_shift_one_field.get_unchecked(i) }
                    .div_test(unsafe { *dividends.get_unchecked(i) }),
            );
            i = (i + 1) % n;
        });
    });

    group.bench_function("mul_shift (one field, arithmetized)", |b| {
        let mut i = 0;
        b.iter(|| {
            black_box(
                unsafe { mul_shift_one_field.get_unchecked(i) }
                    .div_arith(unsafe { *dividends.get_unchecked(i) }),
            );
            i = (i + 1) % n;
        });
    });

    // These are all nonzero by construction.
    let non_zero = divisors
        .iter()
        .map(|&x| NonZeroU16::try_from(x).unwrap())
        .collect::<Vec<_>>();

    group.bench_function("hardware", |b| {
        let mut i = 0;
        b.iter(|| {
            // For LLVM, division by zero is undefined behavior, so Rust inserts
            // a check for zero at each division. To avoid this, we must
            // guarantee that the divisor is not zero by wrapping it in a
            // NonZeroU16, which we cast to a NonZeroU32.
            black_box(
                black_box(unsafe { *dividends.get_unchecked(i) })
                    / black_box(NonZeroU32::from(unsafe { *non_zero.get_unchecked(i) })),
            );
            i = (i + 1) % n;
        });
    });
}

criterion_group! {
    name = div_benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_benchmark
}
