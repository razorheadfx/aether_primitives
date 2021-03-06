// required by pool; enables destructuring of std::mem::ManuallyDrop containers
#![feature(manually_drop_take)]

extern crate assert_approx_eq;
extern crate csv;
extern crate num_complex;

/// Shorthand for Complex<f32>
/// Default sample type
/// This type is repr(C), thus 2 f32s back-to-back equivalent to [f32;2] on most platforms
#[allow(non_camel_case_types)]
pub type cf32 = num_complex::Complex32;

/// Shorthand for Complex<f64>
/// This type is repr(C), thus 2 f32s back-to-back equivalent to [f64;2] on most platforms
#[allow(non_camel_case_types)]
pub type cf64 = num_complex::Complex64;

/// Error Vector Magnitude assertion
/// Checks each element and panics if an element in the ```actual```
/// EVM = 10 log (P_error/P_ref) => Error vector in relation to the actually expected signal in dB.
/// The error vector is defined as the vector between the reference symbol and the actually received signal.
/// We achieve this by computing the norm of (actual-ref)
/// If no EVM threshold is provided -80dB = 1e-8 = 10nano is used
/// Due to IEEE 754 floating point representation there may be false positives depending on the range
#[macro_export]
macro_rules! assert_evm {

    ($actual:expr, $ref:expr) => {
        assert_evm!($actual,$ref, -80.0)
    };

    ($actual:expr, $ref:expr, $evm_limit_db:expr) => {
        assert_eq!($actual.len(),$ref.len(), "Input slices/vectors must be same length");
        assert!(($evm_limit_db as f64) < 0.0, "The EVM threshold must be negative");
        for (idx, (act, re)) in $actual.iter().zip($ref).enumerate() {
            let evm = (act - re).norm();
            let limit = re.norm() * 10f64.powf($evm_limit_db as f64 / 10f64) as f32;

            if evm > limit {
                let evm_db = evm.log10() * 10f32;
                panic!(
                    "EVM limit exceeded:  {}({}dB) > {}({}dB) for element {}. Actual {}, Expected {}",
                    evm, evm_db, limit, $evm_limit_db, idx, act, re
                );
            }
        }
    };
}

/// Fourier Transform-related
pub mod fft;

/// FIR: Finite Impulse Response Filters
pub mod fir;

/// Conversion of bits into to Q/I symbols and back
pub mod modulation;

/// Helpers for generating AWGN noise
pub mod noise;

/// Helpers to instantiate thread-based processing pipelines
/// built atop of std::syn::mpsc channels
pub mod pipeline;

/// Object pool for expensive objects which can be shared across threads
pub mod pool;

/// Resampling (up/down), Interpolation
pub mod sampling;

/// Pseudo-Random Sequence Generation
pub mod sequence;

/// Miscelaneous Helpers
pub mod util;

/// Neat operations on vectors and slices
pub mod vecops;

#[cfg(test)]
mod test {
    use super::cf32;

    #[test]
    fn evm_ok() {
        let refr = vec![cf32::new(1f32, 0f32), cf32::new(1f32, 0f32)];

        let act = vec![cf32::new(1f32, 0f32), cf32::new(1f32, 0f32)];
        assert_evm!(act, &refr, -80.0);

        let act = vec![cf32::new(1f32, 0f32), cf32::new(0.99f32, 0f32)];
        assert_evm!(act, &refr, -20);

        let act = vec![cf32::new(1f32, 0f32), cf32::new(1.01f32, 0f32)];
        assert_evm!(act, &refr, -20);
    }

    #[test]
    #[should_panic]
    fn evm_ieee754() {
        let refr = vec![cf32::new(1f32, 0f32), cf32::new(1f32, 0f32)];

        let act = vec![cf32::new(1f32, 0f32), cf32::new(0.9f32, 0f32)];

        assert_evm!(act, refr, -10);
    }

    #[test]
    #[should_panic]
    fn evm_exceeded() {
        let refr = vec![cf32::new(1f32, 0f32), cf32::new(1f32, 0f32)];

        let act = vec![cf32::new(1f32, 0f32), cf32::new(0.98f32, 0f32)];
        // error should be <= 0.0
        assert_evm!(act, refr, -20);
    }
}
