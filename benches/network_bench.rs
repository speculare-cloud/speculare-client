use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use metrics_rs::network::*;

pub fn network_benches(c: &mut Criterion) {
    
}

criterion_group!(benches, network_benches);
criterion_main!(benches);
