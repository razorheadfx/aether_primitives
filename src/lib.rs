extern crate assert_approx_eq;
extern crate csv;
extern crate num_complex;

/// Shorthand for Complex<f32>
/// Default sample type
/// This type is repr(C), thus 2 f32s back-to-back equivalent to [f32;2]
#[allow(non_camel_case_types)]
pub type cf32 = num_complex::Complex32;

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

#[macro_export]
macro_rules! vec_align {
    [$init:expr; $len:expr] => {
        unimplemented!()
    }
}

/// Neat operations on vectors and slices
pub mod vecops;

/// Fourier Transform-related
pub mod fft;

/// Resampling (up/down), Interpolation
pub mod sampling;

/// Pseudo-Random Sequence Generation
pub mod sequence;

/// Operations on files of samples
pub mod file;

/// Helpers for dealing with channels and noise
pub mod channel;

/// Helpers to instantiate thread-based processing pipelines
/// built atop of std::syn::mpsc channels
pub mod pipeline;

/// A minimal, OpenGl accelerated UI based on [piston](https://github.com/pistondevelopers/piston)  
/// Supports waterfall, time domain and eye diagrams
#[cfg(feature = "gui")]
pub mod gui;

/// Uses ```gnuplot``` to fork off threads to plot given data.  
/// If no filename is given to plot functions gnuplot will open
/// a window to display the plot.
#[cfg(feature = "plot")]
pub mod plot;

/// Miscelaneous Helpers
pub mod util;

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

    // TODO: impl
    fn _vec_align() {
        let _v = vec_align![cf32::default(); 2048];
    }

}
