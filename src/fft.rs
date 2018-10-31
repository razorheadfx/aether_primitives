use cf32;

/// Scaling Policy for Transforms
pub enum Scale{
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
    X(f32)
}

/// Wrapper to be implemented for different fft implementations
/// For example for use in VecOps
pub trait Fft{

    /// FFT (Forward) from ```input``` to ```output```
    fn fwd(&mut self, input: &[cf32], output : &mut [cf32], s : Scale);


    /// iFFT (Backward) from ```input``` to ```output```
    fn bwd(&mut self, input: &[cf32], output : &mut [cf32], s : Scale);

    /// In-place FFT (Forward) 
    /// Overwrites the ```input``` with the output of the transform
    fn ifwd(&mut self, input: &mut [cf32], s : Scale);

    /// In-place iFFT (Backward)
    /// Overwrites the input with the output of the transform
    fn ibwd(&mut self, input: &mut [cf32], s : Scale);


}