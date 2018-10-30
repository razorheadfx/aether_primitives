use super::cf32;
use std::cmp;

#[cfg(feature = "fft")]
use super::fft::Fft;

/// This trait is designed to ease operations on complex slices/"vectors"
/// They are not necessarily the most performant way of doing things
/// especially the fft operations are not meant for time/allocation critical situations
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

    /// puts the contents of other into self
    /// (just shortcut for) slice1[a..b].copy_from_slice(slice[c..d])
    fn vec_clone(&mut self, other: impl AsRef<[cf32]>) -> &mut Self;

    /// zero the elements
    fn vec_zero(&mut self) -> &mut Self;

    /// mutate each of the elements with a function to apply
    fn vec_mutate(&mut self, f: impl FnMut(&mut cf32)) -> &mut Self;

    /// element-wise add the other slice to this one
    fn vec_add(&mut self, other: impl AsRef<[cf32]>) -> &mut Self;

    /// element-wise subtract the other slice to this one
    fn vec_sub(&mut self, other: impl AsRef<[cf32]>) -> &mut Self;

    /// perform fft and multiply the result with an optional scalar
    #[cfg(feature = "fft")]
    fn vec_fft(&mut self, scale: Option<f32>) -> &mut Self;

    /// perform ifft and multiply the result with an optional scalar
    #[cfg(feature = "fft")]
    fn vec_ifft(&mut self, scale: Option<f32>) -> &mut Self;

    /// perform fft and multiply the result with an optional scalar
    /// reuses a prebuilt fft instance
    #[cfg(feature = "fft")]
    fn vec_rfft(&mut self, fft: &mut impl Fft, scale: Option<f32>) -> &mut Self;

    /// perform fft and multiply the result with an optional scalar
    /// reuses a prebuilt fft instance
    #[cfg(feature = "fft")]
    fn vec_rifft(&mut self, fft: &mut impl Fft, scale: Option<f32>) -> &mut Self;
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
                    .for_each(|(a, b)| *a = *a * b);
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
                    .for_each(|(a, b)| *a = *a / b);
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
            fn vec_fft(&mut self, scale: Option<f32>) -> &mut Self {
                unimplemented!()
            }

            #[cfg(feature = "fft")]
            fn vec_ifft(&mut self, scale: Option<f32>) -> &mut Self {
                unimplemented!()
            }

            #[cfg(feature = "fft")]
            fn vec_rfft(&mut self, fft: &mut impl Fft, scale: Option<f32>) -> &mut Self {
                fft.fwdi(self.as_mut(), scale);
                self
            }
            #[cfg(feature = "fft")]
            fn vec_rifft(&mut self, fft: &mut impl Fft, scale: Option<f32>) -> &mut Self {
                fft.bwdi(self.as_mut(), scale);
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
                    .for_each(|(a, b)| *a = *a * b);
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
                    .for_each(|(a, b)| *a = *a / b);
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
            fn vec_fft(&mut self, scale: Option<f32>) -> &mut Self {
                unimplemented!()
            }

            #[cfg(feature = "fft")]
            fn vec_ifft(&mut self, scale: Option<f32>) -> &mut Self {
                unimplemented!()
            }

            #[cfg(feature = "fft")]
            fn vec_rfft(&mut self, fft: &mut impl Fft, scale: Option<f32>) -> &mut Self {
                fft.fwdi(self.as_mut(), scale);
                self
            }

            #[cfg(feature = "fft")]
            fn vec_rifft(&mut self, fft: &mut impl Fft, scale: Option<f32>) -> &mut Self {
                fft.bwdi(self.as_mut(), scale);
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
    use super::cf32;
    use super::VecOps;

    #[test]
    fn vec_ergonomics() {
        let mut v = vec![cf32::new(2.0, 2.0); 100];
        let v2 = v.clone();
        let ones = vec![cf32::new(1.0, 1.0); 100];

        v.vec_div(&v2)
            .vec_mul(&v2)
            .vec_scale(0.5)
            .vec_add(&ones)
            .vec_sub(&v2)
            .vec_mirror()
            .vec_conj();
    }

    #[test]
    fn vec_mul() {
        unimplemented!()
    }

    #[test]
    fn vec_div() {
        unimplemented!()
    }

    #[test]
    fn vec_conj() {
        unimplemented!()
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
        unimplemented!()
    }

    #[test]
    fn vec_clone() {
        unimplemented!()
    }

    #[test]
    fn vec_zero() {
        unimplemented!()
    }

    #[test]
    fn vec_mutate() {
        unimplemented!()
    }

    #[test]
    #[cfg(feature = "fft")]
    fn vec_fft() {
        unimplemented!()
    }

    #[test]
    #[cfg(feature = "fft")]
    fn vec_ifft() {
        unimplemented!()
    }

    #[test]
    #[cfg(feature = "fft")]
    fn vec_rfft() {
        unimplemented!()
    }

    #[test]
    #[cfg(feature = "fft")]
    fn vec_rifft() {
        unimplemented!()
    }

}
