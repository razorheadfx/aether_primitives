use crate::cf32;
use crate::vecops::VecOps;

/// Scaling Policy for Transforms
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Scale {
    /// No scaling
    None,
    /// Multiplies with 1/sqrt(N)
    /// with ```N```: transform length
    /// Commonly used for symmetric spectra
    SN,
    /// Multiplies with 1/N
    /// with ```N```: transform length
    N,
    /// Multiplies with a user-provided scaling factor X
    X(f32),
}

impl Scale {
    /// scale all elements of the given slice using this scaler
    pub fn scale(self, data: &mut [cf32]) {
        match self {
            Scale::None => (),
            Scale::SN => {
                let s = (data.len() as f32).sqrt().recip();
                data.vec_scale(s);
            }
            Scale::N => {
                let s = (data.len() as f32).recip();
                data.vec_scale(s);
            }
            Scale::X(s) => {
                data.vec_scale(s);
            }
        }
    }
}

/// Wrapper to be implemented for different fft implementations
/// For use in VecOps or using the Cfft standalone struct.
/// FFT and input must be the same length.
/// Currently there are two fft implementations which are supported:
/// [chfft](https://github.com/chalharu/chfft) (activated by activating
/// the ```fft_chfft``` feature) and [RustFFT](https://github.com/awelkie/RustFFT)
/// (activated via ```fft_rustfft```).
#[allow(clippy::len_without_is_empty)]
pub trait Fft {
    /// FFT (Forward) from ```input``` to ```output```  
    /// Does not modify contents of ```input```
    fn fwd(&mut self, input: &[cf32], output: &mut [cf32], s: Scale);

    /// iFFT (Backward) from ```input``` to ```output```  
    /// Does not modify contents of ```input```
    fn bwd(&mut self, input: &[cf32], output: &mut [cf32], s: Scale);

    /// In-place FFT (Forward)  
    /// Overwrites the ```input``` with the output of the transform
    fn ifwd(&mut self, input: &mut [cf32], s: Scale);

    /// In-place iFFT (Backward)  
    /// Overwrites the input with the output of the transform
    fn ibwd(&mut self, input: &mut [cf32], s: Scale);

    /// temporary FFT (Forward) from ```input```
    /// Does not modify contents of input and then grants read access to
    /// the internal temp buffer.
    fn tfwd(&mut self, input: &[cf32], s: Scale) -> &[cf32];

    /// temporary iFFT (Backward) from ```input```
    /// Does not modify contents of input and then grants read access to
    /// the internal temp buffer.
    fn tbwd(&mut self, input: &[cf32], s: Scale) -> &[cf32];

    /// Retrieve the (fixed) size (number of bins) this is generated for
    fn len(&self) -> usize;
}

/// Complex fft using [Allen Welkie's Rustfft](https://github.com/awelkie/RustFFT)
/// This implementation will always perform an additional copy step in the
/// service of preseverving input. In addition its internal buffer is twice
/// in order to support tfwd/tbwd.
/// Rustffts planner is used to select the appropriate algorithm for the given fft size
/// # Example
/// ```
/// use aether_primitives::{cf32, assert_evm};
/// use aether_primitives::fft::{Fft,Cfft, Scale};
/// use aether_primitives::vecops::VecOps;
///
/// // no scaling of the result (other scalers are N=> 1/n, SN => 1/sqrt(N), X(your number) with N the number of bins)
/// let scale = Scale::None;
/// let mut data = vec![cf32::new(1.0,0.0);128];
/// // the vecops version
/// // forward transform convenience function (creates and destroys a Cfft instance on the fly)
/// // useful for rapid prototyping
/// data.vec_fft(scale);
/// let mut right = vec![cf32::default();128];
/// right[0] = cf32::new(128.0,0.0);
/// assert_evm!(&data, &right, -10.0);
///
///
/// ```
#[cfg(feature = "fft_rustfft")]
pub use self::ru::Cfft;

#[cfg(feature = "fft_rustfft")]
mod ru {
    extern crate rustfft;
    use super::{Fft, Scale};
    use std::sync::Arc;

    use crate::cf32;
    use crate::vecops::VecOps;
    use rustfft::{FFTplanner, FFT};

    pub struct Cfft {
        // unfortunately we need to use a smart pointer here
        fwd: Arc<FFT<f32>>,
        // unfortunately we need to use a smart pointer here
        bwd: Arc<FFT<f32>>,
        /// this this is used as an internal buffer in order to preserve input
        /// vector is twice the length so we can support temp transform variants tfwd/tbwd.
        tmp: Vec<cf32>,
        len: usize,
    }

    impl Cfft {
        /// Setup a RustFFT for forward and backward operation with the given length
        pub fn with_len(len: usize) -> Cfft {
            let fwd = FFTplanner::new(true).plan_fft(len);

            let bwd = FFTplanner::new(false).plan_fft(len);

            Cfft {
                fwd,
                bwd,
                tmp: vec![cf32::default(); 2 * len],
                len,
            }
        }
    }

    impl Fft for Cfft {
        fn fwd(&mut self, input: &[cf32], output: &mut [cf32], s: Scale) {
            assert_eq!(
                self.len,
                input.len(),
                "Input and FFT must be the same length"
            );
            self.tmp[..self.len].vec_clone(&input);
            self.fwd.process(&mut self.tmp[..self.len], output);
            s.scale(output);
        }

        fn bwd(&mut self, input: &[cf32], output: &mut [cf32], s: Scale) {
            assert_eq!(
                self.len,
                input.len(),
                "Input and FFT must be the same length"
            );
            self.tmp[..self.len].vec_clone(&input);
            self.bwd.process(&mut self.tmp[..self.len], output);
            s.scale(output);
        }

        fn ifwd(&mut self, input: &mut [cf32], s: Scale) {
            assert_eq!(
                self.len,
                input.len(),
                "Input and FFT must be the same length"
            );
            self.tmp[..self.len].vec_clone(&input);
            self.fwd.process(&mut self.tmp[..self.len], input);
            s.scale(input);
        }

        fn ibwd(&mut self, input: &mut [cf32], s: Scale) {
            assert_eq!(
                self.len,
                input.len(),
                "Input and FFT must be the same length"
            );
            self.tmp[..self.len].vec_clone(&input);
            self.bwd.process(&mut self.tmp[..self.len], input);
            s.scale(input);
        }

        fn tfwd(&mut self, input: &[cf32], s: Scale) -> &[cf32] {
            assert_eq!(
                self.len,
                input.len(),
                "Input and FFT must be the same length"
            );
            self.tmp[..self.len].vec_clone(&input);
            let (input, output) = self.tmp.split_at_mut(self.len);
            self.fwd.process(input, output);
            s.scale(output);
            output
        }

        fn tbwd(&mut self, input: &[cf32], s: Scale) -> &[cf32] {
            assert_eq!(
                self.len,
                input.len(),
                "Input and FFT must be the same length"
            );
            self.tmp[..self.len].vec_clone(&input);
            let (input, output) = self.tmp.split_at_mut(self.len);
            self.bwd.process(input, output);
            s.scale(output);
            output
        }

        fn len(&self) -> usize {
            self.len
        }
    }

}

#[cfg(test)]
mod test {
    use crate::cf32;
    use crate::fft::Scale;

    #[test]
    fn scale() {
        let input = vec![cf32::new(4.0, 0.0); 4];

        let no = Scale::None;
        let mut nos = input.clone();
        no.scale(&mut nos);
        assert_evm!(nos, &input, -80.0);

        let sn = Scale::SN;
        let snc = vec![cf32::new(2.0, 0.0); 4];
        let mut sns = input.clone();
        sn.scale(&mut sns);
        assert_evm!(sns, snc, -80.0);

        let n = Scale::N;
        let nc = vec![cf32::new(1.0, 0.0); 4];
        let mut ns = input.clone();
        n.scale(&mut ns);
        assert_evm!(ns, nc, -80.0);

        let x = Scale::X(2.0);
        let xc = vec![cf32::new(8.0, 0.0); 4];
        let mut xs = input.clone();
        x.scale(&mut xs);
        assert_evm!(xs, xc, -80.0);
    }
}
