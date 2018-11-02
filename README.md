# aether-primitives - a software radio framework powered by rust

![aether logo](/aether_logo.svg)

## What is aether?
Aether is designed to ease development of SDR applications by providing convenient (low-level) building blocks for common operations.  

## Examples
Core operations are implemented in the form of the VecOps trait implemented for Vecs/Slices of the C compatible [num::Complex<f32>](https://docs.rs/num-complex/latest/num_complex/type.Complex32.html).  

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

## Design Decisions
* The base versions will be written in idiomatic rust  
* Optimisations and unsafe speedups will be hidden behind feature flags  
* The actual version of the num-traits and num-complex crates are not pinned by aether because multiple concurrent versions of the same trait are incompatible.  
This can cause type level incompatibility if there are dependencies which expose different versions of the same type to the user.
Hence the version is not pinned as cargo will usually try to build the same version of num-complex and num-traits for the biggest set of dependencies (within their version constraints), thus reducing the probability of this happening.

### TODO
- [ ] Pull out choice of FFT ([RustFFT](https://github.com/awelkie/RustFFT), [chfft](https://github.com/chalharu/chfft))
- [ ] Add vec_align! macro to create vecs aligned for SIMD instructions
- [ ] Add Fixed-size cf32 Vecs
    - maybe derefs to slice for convenience
- [ ] Add VecStats (f32,cf32)
    - Min(index),Max(index),Mean(index),Power
- [ ] Add VecOps Features
    - Unsafe Feature: use [VOLK](https://libvolk.org) for ops
        - Add tests to ensure generated code is correctly aligned - should be ensured since cf32 (2x4 bytes) is 8 bytes. VOLK [prefers](https://libvolk.org/doxygen/concepts_terms_and_techniques.html) 32byte alignment /libfftw [prefers](http://www.fftw.org/fftw3_doc/SIMD-alignment-and-fftw_005fmalloc.html) 16 byte alignment
        - must also hook into the vec_align macro
    - Feature: use [faster](https://github.com/AdamNiederer/faster) (currently broken)
    - Optional: vec_norm, vec_fft,vec_ifft, vec_rifft, vec_rfft, vec_rifft (depends on fft feature)
- [ ] Add Correlation by Freq. Domain Convolution
- [ ] Add FIR
- [ ] Add FFT benches

