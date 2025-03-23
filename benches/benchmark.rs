// SPDX-FileCopyrightText: 2025 Severen Redwood <sev@severen.dev>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::hint::black_box;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use rkn::natural::Natural;

// TODO: Come up with less terrible benchmarks.
fn benchmark_addition(c: &mut Criterion) {
  c.bench_function("add two small natural numbers without overflow", |b| {
    b.iter(|| Natural::from(black_box(10)) + Natural::from(black_box(5)))
  });

  c.bench_function("add two small natural numbers with overflow", |b| {
    b.iter(|| Natural::from(u64::MAX) + Natural::ONE)
  });

  let small_max = Natural::from(u64::MAX);

  let m = small_max.clone() + Natural::ONE;
  let n = small_max.clone() + small_max.clone();
  c.bench_function("add two large natural numbers without overflow", |b| {
    b.iter_batched(
      || (m.clone(), n.clone()),
      |(m, n)| m + n,
      BatchSize::SmallInput,
    );
  });
}

criterion_group!(benches, benchmark_addition);
criterion_main!(benches);
