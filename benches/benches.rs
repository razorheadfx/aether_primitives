#[macro_use]
extern crate criterion;

use aether_primitives::{cf32, sampling};
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

fn interpolate_1024_4(c: &mut Criterion) {
    c.bench_function("sampling::interpolate", |b| {
        b.iter_with_setup(
            || {
                let src = (0..1024).map(|i|cf32::new(i as f32, 0.0)).collect::<Vec<_>>();
                let dst = vec![cf32::default(); 2048];
                (src, dst)
            },
            |(src, mut dst)| {
                sampling::interpolate(&src, &mut dst, 4);
            },
        );
    });
}


criterion_group!(vecops, mul, clone);
criterion_group!(sampling, interpolate_1024_4);
criterion_main!(vecops, sampling);
