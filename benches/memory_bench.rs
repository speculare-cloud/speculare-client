use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use metrics_rs::memory::*;

pub fn memory_benches(c: &mut Criterion) {
    c.bench_function("get_memory", |b| b.iter(|| get_memory()));
}

criterion_group!(benches, memory_benches);
criterion_main!(benches);
