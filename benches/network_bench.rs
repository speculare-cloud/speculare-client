use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use metrics_rs::network::*;

pub fn network_benches(c: &mut Criterion) {
    c.bench_function("get_mac_address", |b| b.iter(|| get_mac_address()));
}

criterion_group!(benches, network_benches);
criterion_main!(benches);
