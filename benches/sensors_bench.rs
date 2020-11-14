use criterion::Criterion;
use criterion::{criterion_group, criterion_main};

pub fn sensors_benches(_c: &mut Criterion) {}

criterion_group!(benches, sensors_benches);
criterion_main!(benches);
