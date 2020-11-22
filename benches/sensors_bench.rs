use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use metrics::sensors::get_sensors_data;

pub fn sensors_benches(c: &mut Criterion) {
    c.bench_function("get_sensors_data", |b| b.iter(|| get_sensors_data()));
}

criterion_group!(benches, sensors_benches);
criterion_main!(benches);
