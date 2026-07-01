use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use smallvec::SmallVec as CrateSmallVec;
use smallvector_llvm_rs::SmallVec;

const N: usize = 16;

/// Sizes chosen to straddle the inline/spill boundary (N = 16):
///   4, 16  -> stay inline (no heap alloc; SmallVec should win)
///   64, 512 -> spill to the heap (should track Vec, minus spill overhead)
const SIZES: [usize; 4] = [4, 16, 64, 512];

fn bench_push(c: &mut Criterion) {
    let mut group = c.benchmark_group("push");
    for &n in &SIZES {
        group.throughput(Throughput::Elements(n as u64));

        group.bench_with_input(BenchmarkId::new("SmallVec", n), &n, |b, &n| {
            b.iter(|| {
                let mut sv = SmallVec::<u64, N>::new();
                for i in 0..n {
                    sv.push(black_box(i as u64));
                }
                black_box(sv.len())
            });
        });

        group.bench_with_input(BenchmarkId::new("Vec", n), &n, |b, &n| {
            b.iter(|| {
                let mut v: Vec<u64> = Vec::new();
                for i in 0..n {
                    v.push(black_box(i as u64));
                }
                black_box(v.len())
            });
        });

        group.bench_with_input(BenchmarkId::new("Vec_with_capacity", n), &n, |b, &n| {
            b.iter(|| {
                let mut v: Vec<u64> = Vec::with_capacity(n);
                for i in 0..n {
                    v.push(black_box(i as u64));
                }
                black_box(v.len())
            });
        });

        group.bench_with_input(BenchmarkId::new("smallvec_crate", n), &n, |b, &n| {
            b.iter(|| {
                let mut sv: CrateSmallVec<[u64; N]> = CrateSmallVec::new();
                for i in 0..n {
                    sv.push(black_box(i as u64));
                }
                black_box(sv.len())
            });
        });
    }
    group.finish();
}

fn bench_push_pop(c: &mut Criterion) {
    let mut group = c.benchmark_group("push_pop");
    for &n in &SIZES {
        group.throughput(Throughput::Elements(n as u64));

        group.bench_with_input(BenchmarkId::new("SmallVec", n), &n, |b, &n| {
            b.iter(|| {
                let mut sv = SmallVec::<u64, N>::new();
                for i in 0..n {
                    sv.push(black_box(i as u64));
                }
                let mut acc = 0u64;
                while let Some(x) = sv.pop() {
                    acc = acc.wrapping_add(x);
                }
                black_box(acc)
            });
        });

        group.bench_with_input(BenchmarkId::new("Vec", n), &n, |b, &n| {
            b.iter(|| {
                let mut v: Vec<u64> = Vec::new();
                for i in 0..n {
                    v.push(black_box(i as u64));
                }
                let mut acc = 0u64;
                while let Some(x) = v.pop() {
                    acc = acc.wrapping_add(x);
                }
                black_box(acc)
            });
        });

        group.bench_with_input(BenchmarkId::new("Vec_with_capacity", n), &n, |b, &n| {
            b.iter(|| {
                let mut v: Vec<u64> = Vec::with_capacity(n);
                for i in 0..n {
                    v.push(black_box(i as u64));
                }
                let mut acc = 0u64;
                while let Some(x) = v.pop() {
                    acc = acc.wrapping_add(x);
                }
                black_box(acc)
            });
        });

        group.bench_with_input(BenchmarkId::new("smallvec_crate", n), &n, |b, &n| {
            b.iter(|| {
                let mut sv: CrateSmallVec<[u64; N]> = CrateSmallVec::new();
                for i in 0..n {
                    sv.push(black_box(i as u64));
                }
                let mut acc = 0u64;
                while let Some(x) = sv.pop() {
                    acc = acc.wrapping_add(x);
                }
                black_box(acc)
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_push, bench_push_pop);
criterion_main!(benches);
