#[macro_use]
extern crate criterion;

use aether_primitives::vecops::VecOps;
use aether_primitives::{cf32, sampling};
use criterion::{black_box, Criterion};

/////////////////--------------- core ops
fn make_vecs() -> (Vec<cf32>, Vec<cf32>) {
    let v = vec![cf32::new(1.0, 1.0); 2048];
    let v2 = vec![cf32::new(1.0, 1.0); 2048];
    (v, v2)
}

fn mul(c: &mut Criterion) {
    c.bench_function("VecOps.vec_mul", |b| {
        b.iter_with_setup(
            || make_vecs(),
            |(mut v, v2)| {
                v.vec_mul(v2);
            },
        );
    });
}

fn scale(c: &mut Criterion) {
    c.bench_function("VecOps.vec_scale", |b| {
        b.iter_with_setup(
            || vec![cf32::new(1.0, 1.0); 2048],
            |mut v| {
                v.vec_scale(2.0);
            },
        );
    });
}

fn clone(c: &mut Criterion) {
    c.bench_function("VecOps.vec_clone", |b| {
        b.iter_with_setup(
            || make_vecs(),
            |(mut v, v2)| {
                v.vec_clone(v2);
            },
        );
    });
}

fn interpolate(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "sampling::interpolate",
        |b, (len, between)| {
            b.iter_with_setup(
                || {
                    let src = (0..*len)
                        .map(|i| cf32::new(i as f32, 0.0))
                        .collect::<Vec<_>>();
                    let dst = vec![cf32::default(); *len * 2 / *between];
                    (src, dst)
                },
                |(src, mut dst)| {
                    sampling::interpolate(&src, &mut dst, 4);
                },
            );
        },
        vec![(1024, 4), (2048, 4), (400, 3)],
    );
}

/// downsample by 30
fn downsample_30720_to_1024(c: &mut Criterion) {
    c.bench_function("sampling::downsample 30720 to 1024", |b| {
        b.iter_with_setup(
            || {
                let src = vec![cf32::new(1.0, 1.0); 30720];
                let dst = vec![cf32::default(); 1024];
                (src, dst)
            },
            |(src, mut dst)| {
                sampling::downsample(&src, &mut dst);
            },
        );
    });
}

/// downsample by 30 using the step_by implementation
fn downsample_step_by_30720_to_1024(c: &mut Criterion) {
    c.bench_function("sampling::downsample_sb 30720 to 1024", |b| {
        b.iter_with_setup(
            || {
                let src = vec![cf32::new(1.0, 1.0); 30720];
                let dst = vec![cf32::default(); 1024];
                (src, dst)
            },
            |(src, mut dst)| {
                sampling::downsample_sb(&src, &mut dst);
            },
        );
    });
}

criterion_group!(vecops, mul, clone, scale);
criterion_group!(
    sampling,
    interpolate,
    downsample_30720_to_1024,
    downsample_step_by_30720_to_1024
);

//////////////---------------ffts
#[cfg(feature = "fft")]
use aether_primitives::fft::{Cfft, Fft, Scale};

fn inplace_ffts(_c: &mut Criterion) {
    #[cfg(feature = "fft")]
    {
        _c.bench_function_over_inputs(
            "fft inplace fwd",
            |b: &mut criterion::Bencher, len: &usize| {
                b.iter_with_setup(
                    || {
                        let len = *len;
                        let input = vec![cf32::new(1.0, 1.0); len];
                        let fft = Cfft::with_len(len);
                        (input, fft)
                    },
                    |(mut input, mut fft)| {
                        let s = Scale::SN;
                        fft.ifwd(&mut input, s);
                    },
                );
            },
            vec![512usize, 1024usize, 2048usize],
        );

        _c.bench_function_over_inputs(
            "fft inplace bwd",
            |b: &mut criterion::Bencher, len: &usize| {
                b.iter_with_setup(
                    || {
                        let len = *len;
                        let input = vec![cf32::new(1.0, 1.0); len];
                        let fft = Cfft::with_len(len);
                        (input, fft)
                    },
                    |(mut input, mut fft)| {
                        let s = Scale::SN;
                        fft.ibwd(&mut input, s);
                    },
                );
            },
            vec![512usize, 1024usize, 2048usize],
        );
    }
}

fn copy_ffts(_c: &mut Criterion) {
    #[cfg(feature = "fft")]
    {
        _c.bench_function_over_inputs(
            "fft copy fwd",
            |b: &mut criterion::Bencher, len: &usize| {
                b.iter_with_setup(
                    || {
                        let len = *len;
                        let input = vec![cf32::new(1.0, 1.0); len];
                        let output = input.clone();
                        let fft = Cfft::with_len(len);
                        (input, output, fft)
                    },
                    |(input, mut output, mut fft)| {
                        let s = Scale::SN;
                        fft.fwd(&input, &mut output, s);
                    },
                );
            },
            vec![512usize, 1024usize, 2048usize],
        );

        _c.bench_function_over_inputs(
            "fft copy bwd",
            |b: &mut criterion::Bencher, len: &usize| {
                b.iter_with_setup(
                    || {
                        let len = *len;
                        let input = vec![cf32::new(1.0, 1.0); len];
                        let output = input.clone();
                        let fft = Cfft::with_len(len);
                        (input, output, fft)
                    },
                    |(input, mut output, mut fft)| {
                        let s = Scale::SN;
                        fft.bwd(&input, &mut output, s);
                    },
                );
            },
            vec![512usize, 1024usize, 2048usize],
        );
    }
}

fn inplace_correlator(_c: &mut Criterion) {
    #[cfg(feature = "fft")]
    {
        _c.bench_function_over_inputs(
            "correlator inplace",
            |b: &mut criterion::Bencher, len: &usize| {
                b.iter_with_setup(
                    || {
                        let len = *len;
                        let mut sig = vec![cf32::new(-1.0,1.0), cf32::new( 0.0, 0.0), cf32::new( 1.0,-1.0), cf32::new( 1.0,-1.0)];
                        let input = (0..len).map(|i| sig[i % 4]).collect::<Vec<_>>();
                        
                        sig.vec_conj();
                        while sig.len() < len{
                            sig.push(cf32::default())
                        }

                        let fft = Cfft::with_len(len);
                        (input, sig, fft)
                    },
                    |(mut input, sig, mut fft)| {
                        let s = Scale::None;
                        input.vec_rfft(&mut fft, s)
                            .vec_mul(&sig)
                            .vec_rifft(&mut fft, s);
                        black_box(input)
                    },
                );
            },
            vec![512usize, 1024usize, 2048usize],
        );
    }

}



criterion_group!(fft, inplace_ffts, copy_ffts, inplace_correlator);

////////////////----------------------------
criterion_main!(vecops, sampling, fft);
