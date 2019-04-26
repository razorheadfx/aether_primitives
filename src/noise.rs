use crate::cf32;
use rand::distributions::Normal;
use rand::prelude::*;
use rand::SeedableRng;

const DEFAULT_RNG_SEED: u64 = 815;

/// Creates an AWGN generator with default seed and a noise power of 1
pub fn generator() -> Awgn {
    Awgn::new(1f32, DEFAULT_RNG_SEED)
}

/// Creates an AWGN generator with the given power and seed value
pub fn new(power: f32, seed: u64) -> Awgn {
    Awgn::new(power, seed)
}

/// An AWGN Sampler
#[derive(Debug)]
pub struct Awgn {
    pub power: f32,
    pub rng: StdRng,
    pub dist: Normal,
    scale: f32,
}

impl Awgn {
    /// Initalise an AWGN with given power (Standard Deviation) and RNG seed
    pub fn new(power: f32, seed: u64) -> Awgn {
        Awgn {
            power,
            rng: SeedableRng::seed_from_u64(seed),
            dist: Normal::new(0f64, 1f64),
            scale: power.sqrt(),
        }
    }

    #[inline(always)]
    fn next(&mut self) -> cf32 {
        cf32 {
            re: self.rng.sample(self.dist) as f32 * self.scale,
            im: self.rng.sample(self.dist) as f32 * self.scale,
        }
    }

    /// Change the noise power
    pub fn set_power(&mut self, power: f32) {
        self.power = power;
        self.scale = power.sqrt();
    }

    /// Overlay the given signal with noise from this generator
    pub fn apply(&mut self, signal: &mut [cf32]) {
        let sc = self.scale;
        signal
            .iter_mut()
            .zip(self.iter())
            .for_each(|(s, n)| *s += n.scale(sc));
    }

    /// Fill a vector up to capacity with noise from this generator
    pub fn fill(&mut self, target: &mut Vec<cf32>) {
        while target.len() < target.capacity() {
            target.push(self.next())
        }
    }

    pub fn iter(&mut self) -> NoiseIter {
        NoiseIter { noisegen: self }
    }
}

#[derive(Debug)]
pub struct NoiseIter<'a> {
    noisegen: &'a mut Awgn,
}

impl<'a> Iterator for NoiseIter<'a> {
    type Item = cf32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.noisegen.next())
    }
}
