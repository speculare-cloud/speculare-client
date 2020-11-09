use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use metrics_rs::disks::*;

pub fn disks_benches(c: &mut Criterion) {
    c.bench_function("get_disks_data", |b| b.iter(|| get_disks_data()));
}

criterion_group!(benches, disks_benches);
criterion_main!(benches);
