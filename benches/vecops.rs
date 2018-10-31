#[macro_use]
extern crate criterion;

use aether_primitives::cf32;
use aether_primitives::vecops::VecOps;
use criterion::Criterion;

fn make_vecs() -> (Vec<cf32>,Vec<cf32>){
    let v = vec![cf32::new(1.0, 1.0); 2048];
    let v2 = vec![cf32::new(1.0, 1.0); 2048];
    (v, v2)
}

fn mul(c: &mut Criterion) {
    c.bench_function("VecOps.vec_mul", |b| {
        b.iter_with_setup(
            || {
                make_vecs()
            },
            |(mut v, v2)| {
                v.vec_mul(v2);
            },
        );
    });
}

fn clone(c: &mut Criterion) {
    c.bench_function("VecOps.vec_clone", |b| {
        b.iter_with_setup(
            || {
                make_vecs()
            },
            |(mut v, v2)| {
                v.vec_clone(v2);
            },
        );
    });
}


criterion_group!(benches, mul, clone);
criterion_main!(benches);
