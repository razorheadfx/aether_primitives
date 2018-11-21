use super::cf32;
use std::cmp;

#[cfg(feature = "fft")]
use crate::fft::{Cfft, Fft, Scale};

/// This trait is designed to ease operations on complex slices/"vectors"
/// They are not necessarily the most performant way of doing things but
/// they are written in idiomatic and hence safe Rust.
///
/// __Example__
///```
///#[macro_use]
/// extern crate aether_primitives;
/// use aether_primitives::{cf32, vecops::VecOps};
///# fn main(){
///# // the main(){} is need in order to get around "loading macro must be at the crate root error"
///
/// let mut v = vec![cf32::new(2.0, 2.0); 100];
/// let twos = v.clone();
/// let ones = vec![cf32::new(1.0, 1.0); 100];
///
/// let correct = vec![cf32::new(1.0, -1.0); 100];
///
/// v.vec_div(&twos)
///     .vec_mul(&twos)
///     .vec_zero() // zero the vector
///     .vec_add(&ones)
///     .vec_sub(&twos)
///     .vec_clone(&ones)
///     .vec_mutate(|c| c.im = -1.0)
///     .vec_conj()
///     .vec_mirror(); // mirror swaps elements around the midpoint of the array
///
/// // ensure each element's error vector magnitude in relation to the correct complex number is below -80dB
/// assert_evm!(&v, &correct, -80.0);
/// # }
///```
pub trait VecOps {
    /// scale this this vector with the given f32
    fn vec_scale(&mut self, scale: f32) -> &mut Self;

    /// element-wise multiply this vector with the other one
    fn vec_mul(&mut self, other: impl AsRef<[cf32]>) -> &mut Self;

    /// element-wise divide this vector by the other one
    fn vec_div(&mut self, other: impl AsRef<[cf32]>) -> &mut Self;

    /// conjugate this vector's elements
    fn vec_conj(&mut self) -> &mut Self;

    /// swap elements around the midpoint of this slice
    /// assumes an even number of elements
    fn vec_mirror(&mut self) -> &mut Self;

    /// copies the contents of other into self
    /// (just shortcut for) slice1[a..b].copy_from_slice(slice[c..d])
    fn vec_clone(&mut self, other: impl AsRef<[cf32]>) -> &mut Self;

    /// zero the elements
    fn vec_zero(&mut self) -> &mut Self;

    /// mutate each of the elements with a function to apply
    fn vec_mutate(&mut self, f: impl FnMut(&mut cf32)) -> &mut Self;

    /// element-wise add the other slice to this one
    fn vec_add(&mut self, other: impl AsRef<[cf32]>) -> &mut Self;

    /// element-wise subtract the other slice from this one
    fn vec_sub(&mut self, other: impl AsRef<[cf32]>) -> &mut Self;

    /// perform fft and multiply the result with an optional scalar
    #[cfg(feature = "fft")]
    fn vec_fft(&mut self, scale: Scale) -> &mut Self;

    /// perform ifft and multiply the result with an optional scalar
    #[cfg(feature = "fft")]
    fn vec_ifft(&mut self, scale: Scale) -> &mut Self;

    /// perform fft and multiply the result with an optional scalar
    /// reuses a prebuilt fft instance
    #[cfg(feature = "fft")]
    fn vec_rfft(&mut self, fft: &mut impl Fft, scale: Scale) -> &mut Self;

    /// perform fft and multiply the result with an optional scalar
    /// reuses a prebuilt fft instance
    #[cfg(feature = "fft")]
    fn vec_rifft(&mut self, fft: &mut impl Fft, scale: Scale) -> &mut Self;
}

macro_rules! impl_vec_ops {
    ($type:ty) => {
        impl<'a> VecOps for &'a mut $type {
            fn vec_scale(&mut self, scale: f32) -> &mut Self {
                self.iter_mut().for_each(|c| *c = c.scale(scale));
                self
            }

            fn vec_mul(&mut self, other: impl AsRef<[cf32]>) -> &mut Self {
                assert_eq!(
                    self.len(),
                    other.as_ref().len(),
                    "Vectors must have same length"
                );

                let min = cmp::min(self.len(), other.as_ref().len());
                self[..min]
                    .iter_mut()
                    .zip(other.as_ref()[..min].iter())
                    .for_each(|(a, b)| *a *= b);
                self
            }

            fn vec_div(&mut self, other: impl AsRef<[cf32]>) -> &mut Self {
                assert_eq!(
                    self.len(),
                    other.as_ref().len(),
                    "Vectors must have same length"
                );

                self.iter_mut()
                    .zip(other.as_ref().iter())
                    .for_each(|(a, b)| *a /= b);
                self
            }

            fn vec_conj(&mut self) -> &mut Self {
                self.iter_mut().for_each(|a| *a = a.conj());
                self
            }

            fn vec_add(&mut self, other: impl AsRef<[cf32]>) -> &mut Self {
                assert_eq!(
                    self.len(),
                    other.as_ref().len(),
                    "Vectors must have same length"
                );
                self.iter_mut()
                    .zip(other.as_ref().iter())
                    .for_each(|(a, b)| *a += b);
                self
            }

            fn vec_sub(&mut self, other: impl AsRef<[cf32]>) -> &mut Self {
                assert_eq!(
                    self.len(),
                    other.as_ref().len(),
                    "Vectors must have same length"
                );

                self.iter_mut()
                    .zip(other.as_ref().iter())
                    .for_each(|(a, b)| *a -= b);
                self
            }

            fn vec_mirror(&mut self) -> &mut Self {
                let mid = self.len() / 2;
                (0usize..mid).for_each(|x| self.swap(x, x + mid));
                self
            }

            fn vec_clone(&mut self, other: impl AsRef<[cf32]>) -> &mut Self {
                assert_eq!(
                    self.len(),
                    other.as_ref().len(),
                    "Vectors must have same length"
                );

                self.copy_from_slice(other.as_ref());
                self
            }

            fn vec_zero(&mut self) -> &mut Self {
                self.iter_mut().for_each(|c| *c = cf32 { re: 0.0, im: 0.0 });
                self
            }

            fn vec_mutate(&mut self, f: impl FnMut(&mut cf32)) -> &mut Self {
                self.iter_mut().for_each(f);
                self
            }

            #[cfg(feature = "fft")]
            fn vec_fft(&mut self, scale: Scale) -> &mut Self {
                let mut fft = Cfft::with_len(self.len());
                fft.ifwd(&mut self[..], scale);
                self

            }

            #[cfg(feature = "fft")]
            fn vec_ifft(&mut self, scale: Scale) -> &mut Self {
                let mut fft = Cfft::with_len(self.len());
                fft.ibwd(&mut self[..], scale);
                self
            }

            #[cfg(feature = "fft")]
            fn vec_rfft(&mut self, fft: &mut impl Fft, scale: Scale) -> &mut Self {
                fft.ifwd(self.as_mut(), scale);
                self
            }
            #[cfg(feature = "fft")]
            fn vec_rifft(&mut self, fft: &mut impl Fft, scale: Scale) -> &mut Self {
                fft.ibwd(self.as_mut(), scale);
                self
            }
        }

        impl<'a> VecOps for $type {
            fn vec_scale(&mut self, scale: f32) -> &mut Self {
                self.iter_mut().for_each(|c| *c = c.scale(scale));
                self
            }

            fn vec_mul(&mut self, other: impl AsRef<[cf32]>) -> &mut Self {
                assert_eq!(
                    self.len(),
                    other.as_ref().len(),
                    "Vectors must have same length"
                );

                let min = cmp::min(self.len(), other.as_ref().len());
                self[..min]
                    .iter_mut()
                    .zip(other.as_ref()[..min].iter())
                    .for_each(|(a, b)| *a *= b);
                self
            }

            fn vec_div(&mut self, other: impl AsRef<[cf32]>) -> &mut Self {
                assert_eq!(
                    self.len(),
                    other.as_ref().len(),
                    "Vectors must have same length"
                );

                self.iter_mut()
                    .zip(other.as_ref().iter())
                    .for_each(|(a, b)| *a /= b);
                self
            }

            fn vec_conj(&mut self) -> &mut Self {
                self.iter_mut().for_each(|a| *a = a.conj());
                self
            }

            fn vec_add(&mut self, other: impl AsRef<[cf32]>) -> &mut Self {
                assert_eq!(
                    self.len(),
                    other.as_ref().len(),
                    "Vectors must have same length"
                );
                self.iter_mut()
                    .zip(other.as_ref().iter())
                    .for_each(|(a, b)| *a += b);
                self
            }

            fn vec_sub(&mut self, other: impl AsRef<[cf32]>) -> &mut Self {
                assert_eq!(
                    self.len(),
                    other.as_ref().len(),
                    "Vectors must have same length"
                );

                self.iter_mut()
                    .zip(other.as_ref().iter())
                    .for_each(|(a, b)| *a -= b);
                self
            }

            fn vec_mirror(&mut self) -> &mut Self {
                let mid = self.len() / 2;
                (0usize..mid).for_each(|x| self.swap(x, x + mid));
                self
            }

            fn vec_clone(&mut self, other: impl AsRef<[cf32]>) -> &mut Self {
                assert_eq!(
                    self.len(),
                    other.as_ref().len(),
                    "Vectors must have same length"
                );

                self.copy_from_slice(other.as_ref());
                self
            }

            fn vec_zero(&mut self) -> &mut Self {
                self.iter_mut().for_each(|c| *c = cf32 { re: 0.0, im: 0.0 });
                self
            }

            fn vec_mutate(&mut self, f: impl FnMut(&mut cf32)) -> &mut Self {
                self.iter_mut().for_each(f);
                self
            }

            #[cfg(feature = "fft")]
            fn vec_fft(&mut self, scale: Scale) -> &mut Self {
                let mut fft = Cfft::with_len(self.len());
                fft.ifwd(&mut self[..], scale);
                self
            }

            #[cfg(feature = "fft")]
            fn vec_ifft(&mut self, scale: Scale) -> &mut Self {
                let mut fft = Cfft::with_len(self.len());
                fft.ibwd(&mut self[..], scale);
                self
            }

            #[cfg(feature = "fft")]
            fn vec_rfft(&mut self, fft: &mut impl Fft, scale: Scale) -> &mut Self {
                fft.ifwd(self.as_mut(), scale);
                self
            }

            #[cfg(feature = "fft")]
            fn vec_rifft(&mut self, fft: &mut impl Fft, scale: Scale) -> &mut Self {
                fft.ibwd(self.as_mut(), scale);
                self
            }
        }
    };
}

// derive using the macro above
impl_vec_ops!([cf32]);
impl_vec_ops!(Vec<cf32>);

#[cfg(test)]
mod test {
    use crate::cf32;
    use crate::vecops::VecOps;

    #[test]
    fn vec_scale() {
        let mut v = vec![cf32::new(0.5, 0.5); 100];
        let ones = vec![cf32::new(1.0, 1.0); 100];
        v.vec_scale(2.0);

        assert_evm!(&v, &ones);
    }

    #[test]
    fn vec_mul() {
        let mut ones = vec![cf32::new(1.0, 1.0); 100];
        let twos = vec![cf32::new(0.0, 2.0); 100];
        let minus_two_two = vec![cf32::new(-2.0, 2.0); 100];

        ones.vec_mul(&twos);

        assert_evm!(&ones, &minus_two_two);
    }

    #[test]
    fn vec_div() {
        let mut twos = vec![cf32::new(2.0, 2.0); 100];
        let two_re = vec![cf32::new(2.0, 0.0); 100];
        let ones = vec![cf32::new(1.0, 0.0); 100];
        twos.vec_div(&two_re);

        assert_evm!(&twos, ones);
    }

    #[test]
    fn vec_conj() {
        let mut ones = vec![cf32::new(1.0, 1.0); 100];
        let conj_ones = vec![cf32::new(1.0, -1.0); 100];
        ones.vec_conj();

        assert_evm!(ones, conj_ones, -80.0);
    }

    #[test]
    fn vec_add() {
        let ones = vec![cf32::new(1.0, 1.0); 100];
        let twos = vec![cf32::new(2.0, 2.0); 100];
        let mut v = vec![cf32::new(1.0, 1.0); 100];
        v.vec_add(&ones);
        assert_evm!(v, twos, -80.0);
    }

    #[test]
    fn vec_sub() {
        let mut v = vec![cf32::new(2.0, 2.0); 100];
        let ones = vec![cf32::new(1.0, 1.0); 100];
        v.vec_sub(&ones);
        assert_evm!(v, ones, -80.0);
    }

    #[test]
    fn vec_mirror() {
        let mut even = (0..4).map(|i| cf32::new(i as f32, 0.0)).collect::<Vec<_>>();
        let even_mirrored = [2, 3, 0, 1]
            .iter()
            .map(|i| cf32::new(*i as f32, 0.0))
            .collect::<Vec<_>>();
        even.vec_mirror();

        assert_evm!(&even, &even_mirrored);

        // This case is excluded as per documentation, but we check for it anyway
        let mut odd = (0..5).map(|i| cf32::new(i as f32, 0.0)).collect::<Vec<_>>();
        let odd_mirrored = [3, 4, 0, 1, 2]
            .iter()
            .map(|i| cf32::new(*i as f32, 0.0))
            .collect::<Vec<_>>();
        odd.vec_mirror();
        assert_evm!(&odd, &odd_mirrored);
    }

    #[test]
    fn vec_clone() {
        let mut v = vec![cf32::new(2.0, 2.0); 100];
        let ones = vec![cf32::new(1.0, 1.0); 100];
        v.vec_clone(&ones);

        assert_evm!(&v, &ones);
    }

    #[test]
    fn vec_zero() {
        let mut v = vec![cf32::new(2.0, 2.0); 100];
        let zeros = vec![cf32::new(0.0, 0.0); 100];

        v.vec_zero();

        assert_evm!(&v, &zeros);
    }

    #[test]
    fn vec_mutate() {
        let mut v = vec![cf32::new(1.0, 1.0); 100];
        let linear = (0..100)
            .map(|i| cf32::new(i as f32, i as f32))
            .collect::<Vec<_>>();

        let mut x = 0;
        let f = move |c: &mut cf32| {
            *c = c.scale(x as f32);
            x += 1;
        };
        v.vec_mutate(f);

        assert_evm!(&v, &linear);
    }

    #[test]
    #[cfg(feature = "fft")]
    fn vec_fft() {
        use crate::fft::Scale;
        let v = vec![cf32::new(1.0, 1.0); 100];
        let mut c = v.clone();
        c.vec_fft(Scale::None).vec_ifft(Scale::None);
        assert_evm!(c, v, -80.0);
    }

    #[test]
    #[cfg(feature = "fft")]
    fn vec_rfft() {
        use crate::fft::{Scale, Cfft};
        let v = vec![cf32::new(1.0, 1.0); 100];
        let mut c = v.clone();
        let mut fft = Cfft::with_len(100);
        c.vec_rfft(&mut fft, Scale::None).vec_rifft(&mut fft, Scale::None);
        assert_evm!(c, v, -80.0);
    }


}
