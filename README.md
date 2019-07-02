# aether-primitives - a rusty software-defined radio toolbox
[![Latest Version](https://img.shields.io/crates/v/aether_primitives.svg)](https://crates.io/crates/aether_primitives)
[![Documentation](https://docs.rs/aether_primitives/badge.svg)](https://docs.rs/crate/aether_primitives)
![License](https://img.shields.io/crates/l/aether_primitives.svg)
[![Build Status](https://travis-ci.org/razorheadfx/aether_primitives.svg?branch=master)](https://travis-ci.org/razorheadfx/aether_primitives)
[![Dependency Status](https://deps.rs/repo/github/razorheadfx/aether_primitives/status.svg)](https://deps.rs/repo/github/razorheadfx/aether_primitives)
## What is aether?
Aether is designed to ease development of SDR applications by providing convenient (low-level) building blocks for common SDR signal processing operations.  

## Design Decisions
- Should come with batteries included, but should not get in the way
- Modular
    - Convenience Traits should be implementable for your own objects
    - Feature gating of non-essential components (i.e. swap out the FFT impl by implementing a trait)
- The base versions will be written in idiomatic rust  
- Optimisations and unsafe speedups will be hidden behind feature flags  

## Examples
Core operations are implemented in the form of the VecOps trait implemented for Vecs/Slices of the C compatible [num::Complex<f32>](https://docs.rs/num-complex/latest/num_complex/type.Complex32.html) (```cf32``` for short).  

```rust
// #[macro_use] // includes the assert_evm macro
// extern crate aether_primitives;
// use aether_primitives::{cf32, vecops::VecOps};
// The main sample type is cf32 which is a type alias for num::Complex<f32>
let mut v = vec![cf32::new(2.0, 2.0); 100];
let twos = v.clone();
let ones = vec![cf32::new(1.0, 1.0); 100];

let correct = vec![cf32::new(1.0, -1.0); 100];

v.vec_div(&twos)
    .vec_mul(&twos)
    .vec_zero() // zero the vector
    .vec_add(&ones)
    .vec_sub(&twos)
    .vec_clone(&ones)
    .vec_mutate(|c| c.im = -1.0) 
    .vec_conj()
    .vec_mirror(); // mirror swaps elements around the midpoint of the array

/// ensure each element's error vector magnitude vs the correct vector is below -80dB
assert_evm!(&v, &correct, -80.0); 
```

## Implemented functionality
- Macros:
    - assert_evm!: check if elements of both vectors have a certain error vector magnitude relative to each other (given in dBm)
- Vecops: Helpers for operations of vectors/slices of cf32
    - Element wise operations: add, subtract, divide, multiply, complex conjugate, mutate
    . Mirror: Swap elements around mid of vector (for even length vectros)
    - Zero entire vector, copy elements over from another vector
    - FEATURE: Perform (i)FFTs using new or existing fourier transform instance (enabled via ```fft_chfft```)
- Sequence: Helpers for binary pseudo-random sequence generation (esp. M-Sequences)
    - expand: Expand a seed value into an initialisation vector for a Pseudo-random sequence
    - generate: Generate a pseudo random sequence
- Sampling
    - linear interpolation
    - even downsampling
- Modulation
    - Generic BPSK and QPSK modulation
    - Hard Demodulator
- Pool
    - Generic, thread-safe object pool
- FFT: DEFAULT FEATURE ```fft_rustfft```
    - perform fast fourier transforms (forward/backward) on slices/vecs of cf32 with different scaling factors
    - Supported fft implementations: [chfft](https://github.com/chalharu/chfft)
- File
    - binary file writing and reading for arbitrary structs
    - csv file writing and reading for arbitrary structs
- Noise
    - AWGN generator
- Pipeline
    - Multithreaded processing pipelines
- Plot: FEATURE ```plot```; requires an installed version of ```gnuplot```
    - Constellation diagram
    - Time sequence plot
    - Comparison plot of two sequences
    - Waterfall plot with a given fft size (requires ```fft_chfft```)
- Utils
    - Conversion from and to dB

- Benches: benchmarks for most operations in aether using criterion.rs framework
    - downsampling, interpolation, fft

## TODO
- [ ] Add vec_align! macro to create vecs aligned for SIMD instructions
- [ ] Ungrowable Vecs
    - maybe derefs to slice for convenience
- [ ] Add VecStats (f32,cf32)
    - Min(index),Max(index),Mean(index),Power
- [ ] Add VecOps Features
    - [ ] Feature: use [faster](https://github.com/AdamNiederer/faster) once it works on stable again
    - Add tests to ensure generated code is correctly aligned - should be ensured since cf32 (2x4 bytes) is 8 bytes. VOLK [prefers](https://libvolk.org/doxygen/concepts_terms_and_techniques.html) 32byte alignment /libfftw [prefers](http://www.fftw.org/fftw3_doc/SIMD-alignment-and-fftw_005fmalloc.html) 16 byte alignment
- [ ] Add Correlation by Freq. Domain Convolution
- [ ] Add FIR

## License
[Mozilla Public License 2.0](LICENSE)