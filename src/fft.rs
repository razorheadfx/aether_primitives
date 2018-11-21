use crate::cf32;
use crate::vecops::VecOps;

/// Scaling Policy for Transforms
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Scale {
    /// Scale with 1 (no scaling)
    None,
    /// Multiplies with 1/sqrt(N)
    /// with ```N```: transform length
    /// Commonly used for symmetric spectra
    SN,
    /// Multiplies with 1/N
    /// with ```N```: transform length
    N,
    /// Multiplies with given scaling factor X
    X(f32),
}

impl Scale {
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
/// For example for use in VecOps
/// FFT and input must be the same length
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

    /// Retrieve the (fixed) size (number of bins) this is generated for
    fn len(&self) -> usize;
}

#[cfg(feature = "fft_chfft")]
/// Complex fft
pub use self::ch::Cfft;

#[cfg(feature = "fft_chfft")]
mod ch {
    extern crate chfft;
    use super::{Fft, Scale};

    use chfft::CFft1D;
    use crate::cf32;
    use crate::vecops::VecOps;

    pub struct Cfft {
        fft: CFft1D<f32>,
        tmp: Vec<cf32>,
        len: usize,
    }

    impl Cfft {
        pub fn with_len(len: usize) -> Cfft {
            Cfft {
                fft: CFft1D::<f32>::with_len(len),
                // TODO: use vec_align
                tmp: vec![cf32::default(); len],
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
            self.tmp.vec_clone(&input);
            self.fft.forward0i(&mut self.tmp);
            // OPT: optimize by scaling as its copied over so we do not have to read this stuff twice
            output.vec_clone(&self.tmp);
            s.scale(output);
        }

        fn bwd(&mut self, input: &[cf32], output: &mut [cf32], s: Scale) {
            assert_eq!(
                self.len,
                input.len(),
                "Input and FFT must be the same length"
            );
            self.tmp.vec_clone(&input);
            self.fft.backward0i(&mut self.tmp);
            output.vec_clone(&self.tmp);
            // OPT: optimize by scaling as its copied over so we do not have to read this stuff twice
            s.scale(output);
        }

        fn ifwd(&mut self, input: &mut [cf32], s: Scale) {
            assert_eq!(
                self.len,
                input.len(),
                "Input and FFT must be the same length"
            );
            self.fft.forward0i(input);
            s.scale(input);
        }

        fn ibwd(&mut self, input: &mut [cf32], s: Scale) {
            assert_eq!(
                self.len,
                input.len(),
                "Input and FFT must be the same length"
            );
            self.fft.backward0i(input);
            s.scale(input);
        }

        fn len(&self) -> usize {
            self.len
        }
    }

}

#[cfg(test)]
mod test {
    use super::Scale;
    use crate::cf32;

    #[test]
    fn scale() {
        let input = vec![cf32::new(4.0, 0.0); 4];

        let no = Scale::None;
        let mut nos = input.clone();
        no.scale(&mut nos);
        assert_evm!(nos, &input, 80.0);

        let sn = Scale::SN;
        let snc = vec![cf32::new(2.0, 0.0); 4];
        let mut sns = input.clone();
        sn.scale(&mut sns);
        assert_evm!(sns, snc, 80.0);

        let n = Scale::N;
        let nc = vec![cf32::new(1.0, 0.0); 4];
        let mut ns = input.clone();
        n.scale(&mut ns);
        assert_evm!(ns, nc, 80.0);

        let x = Scale::X(2.0);
        let xc = vec![cf32::new(8.0, 0.0); 4];
        let mut xs = input.clone();
        x.scale(&mut xs);
        assert_evm!(xs, xc, 80.0);
    }
}
