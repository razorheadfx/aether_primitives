/// Helpers for Average White Gaussian Noise (AWGN)
pub mod noise {
    use crate::cf32;
    use rand::distributions::Normal;
    use rand::prelude::*;
    use rand::SeedableRng;

    const DEFAULT_RNG_SEED: u64 = 0815;

    /// Creates an AWGN generator with default seed and a noise power of 1
    pub fn generator() -> Awgn {
        Awgn::new(1f32, DEFAULT_RNG_SEED)
    }

    /// Creates an AWGN generator with the given power and seed value
    pub fn with(power: f32, seed: u64) -> Awgn {
        Awgn::new(power, seed)
    }

    /// Convenience function which generates a vector of noise of given length and noise power
    pub fn make(len: usize, power: f32) -> std::vec::Vec<cf32> {
        let gen = with(power, DEFAULT_RNG_SEED);
        let mut noise = Vec::with_capacity(len);
        gen.take(len).for_each(|c| noise.push(c));
        noise
    }

    /// An AWGN Sampler
    pub struct Awgn {
        pub power: f32,
        pub rng: StdRng,
        pub dist: Normal,
        scale: f32,
    }

    impl Awgn {
        /// Initalise an AWGN with given power (Standard Deviation) and RNG seed
        fn new(power: f32, seed: u64) -> Awgn {
            Awgn {
                power: power,
                rng: SeedableRng::seed_from_u64(seed),
                dist: Normal::new(0f64, 1f64),
                scale: power.sqrt(),
            }
        }

        /// Change the noise power
        pub fn set_power(&mut self, power: f32) {
            self.power = power;
            self.scale = power.sqrt();
        }

        /// Overlay the given signal with noise from this generator
        pub fn apply(&mut self, signal: &mut [cf32]) {
            let p = self.power.sqrt();
            signal.iter_mut().zip(self).for_each(|(s, n)| *s += n * p);
        }

        /// Fill a vector up to capacity with noise from this generator
        pub fn fill(&mut self, target: &mut Vec<cf32>) {
            while target.len() < target.capacity() {
                target.push(self.next().unwrap())
            }
        }
    }

    impl Iterator for Awgn {
        type Item = cf32;

        fn next(&mut self) -> Option<Self::Item> {
            Some(cf32 {
                re: self.rng.sample(self.dist) as f32 * self.scale,
                im: self.rng.sample(self.dist) as f32 * self.scale,
            })
        }
    }
}
