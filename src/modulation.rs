use crate::cf32;
use std::cmp::{Ordering, PartialOrd};

/// Blanket impl for cf32;2 array
impl Modulation for [cf32; 2] {
    const BITS_PER_SYMBOL: usize = 1;

    fn index(bits: &[u8]) -> usize {
        debug_assert!(bits.len() == Self::BITS_PER_SYMBOL);
        bits[0] as usize
    }

    fn symbol(&self, idx: usize) -> cf32 {
        self[idx]
    }
}

/// Blanket impl for cf32;4 array
impl Modulation for [cf32; 4] {
    const BITS_PER_SYMBOL: usize = 2;

    fn index(bits: &[u8]) -> usize {
        debug_assert!(bits.len() == Self::BITS_PER_SYMBOL);
        ((bits[1] << 1) + bits[0]) as usize
    }

    fn symbol(&self, idx: usize) -> cf32 {
        self[idx]
    }

    // this slightly optimized version cuts demod time by roughly 20%
    // as opposed to the non-vectorized version
    fn demod_naive<'a>(&self, symbols: &mut impl Iterator<Item = &'a cf32>, output: &mut Vec<u8>) {
        let dist = |s: &cf32, idx: usize| {
            // lets hope this gets vectorized
            (
                idx,
                (s.re - self[idx].re) * (s.re - self[idx].re)
                    + (s.im - self[idx].im) * (s.im - self[idx].im),
            )
        };

        let min_idx = |s| {
            [dist(s, 0), dist(s, 1), dist(s, 2), dist(s, 3)]
                .into_iter()
                .min_by(|d, e| d.1.partial_cmp(&e.1).unwrap_or(Ordering::Greater))
                .map(|(idx, _)| *idx)
                .unwrap()
        };

        for s in symbols {
            let idx = min_idx(s) as u8;
            output.push(idx & 1u8);
            output.push(idx & 1u8 << 1);
        }
    }
}

/// Get a generic BPSK modulator
/// Ensure the [modulation::Modulation] trait is in scope
pub fn bpsk() -> [cf32; 2] {
    GENERIC_BPSK_TABLE
}

/// Get a generic QPSK modulator
pub fn qpsk() -> [cf32; 4] {
    GENERIC_QPSK_TABLE
}

/// A Generic Binary Phase-Shift Keying Modulation
/// # Constellation Diagram:
/// ```text
///          | 0                     | 0
/// bits   ----- --> table index:  -----
///        1 |                     1 |
/// ```
pub const GENERIC_BPSK_TABLE: [cf32; 2] = [cf32 { re: 1.0, im: 1.0 }, cf32 { re: -1.0, im: -1.0 }];

/// A Generic Quadrature Phase-Shift Keying Modulation
/// Eqivalent to a 4-Quadrature Amplitude Modulation  
/// # Constellation Diagram:
/// ```text
///          01|00                   1 | 0
/// bits     ----- --> table index:  -----
///          11|10                   3 | 2
/// ```
pub const GENERIC_QPSK_TABLE: [cf32; 4] = [
    cf32 { re: 1.0, im: 1.0 },
    cf32 { re: -1.0, im: 1.0 },
    cf32 { re: 1.0, im: -1.0 },
    cf32 { re: -1.0, im: -1.0 },
];

pub trait Modulation {
    /// Number of bits modulated into one symbol
    const BITS_PER_SYMBOL: usize;

    /// Get the actual symbol value
    fn symbol(&self, idx: usize) -> cf32;

    /// Blanket impl
    /// Treats each bit as a single byte
    /// Expects each input byte in the range [0..=1]
    /// This is the blanket (slow) impl
    #[inline(always)]
    fn index(bits: &[u8]) -> usize {
        debug_assert!(bits.len() == Self::BITS_PER_SYMBOL);
        bits.iter()
            .enumerate()
            .map(|(i, bit)| (*bit as usize % 2) << i)
            .sum()
    }

    #[inline(always)]
    fn modulate(&self, input: &[u8]) -> Vec<cf32> {
        input
            .chunks(Self::BITS_PER_SYMBOL)
            .map(Self::index)
            .map(|idx| self.symbol(idx))
            .collect()
    }

    #[inline(always)]
    fn modulate_into<'a>(&self, input: &[u8], output: &mut impl Iterator<Item = &'a mut cf32>) {
        input
            .chunks(Self::BITS_PER_SYMBOL)
            .map(Self::index)
            .map(|idx| self.symbol(idx))
            .zip(output)
            .for_each(|(s, out)| *out = s);
    }

    fn demod_naive<'a>(&self, symbols: &mut impl Iterator<Item = &'a cf32>, output: &mut Vec<u8>) {
        for symbol in symbols {
            let (idx, _symb) = (0..Self::BITS_PER_SYMBOL * 2)
                .map(|idx| *symbol - self.symbol(idx))
                .map(|dist| dist.re * dist.re + dist.im * dist.im)
                .enumerate()
                .min_by(|(_i, d), (_j, e)| d.partial_cmp(e).unwrap_or(Ordering::Greater))
                .expect("Finding the minimum symbol distance failed");

            output.extend((0..Self::BITS_PER_SYMBOL).map(|i| idx as u8 >> i & 1u8))
        }
    }

    fn bits_per_symbol(&self) -> usize {
        Self::BITS_PER_SYMBOL
    }
}

#[cfg(test)]
mod test {
    use crate::cf32;
    use crate::modulation::{bpsk, qpsk, Modulation, GENERIC_QPSK_TABLE};
    use rand::{rngs::StdRng, Rng, SeedableRng};

    #[test]
    fn generic_bpsk() {
        let input: Vec<u8> = [0, 1, 0, 1].iter().map(|i| *i as u8).collect();

        let output = bpsk().modulate(&input);

        assert_eq!(
            output,
            vec![
                cf32 { re: 1.0, im: 1.0 },
                cf32 { re: -1.0, im: -1.0 },
                cf32 { re: 1.0, im: 1.0 },
                cf32 { re: -1.0, im: -1.0 },
            ]
        )
    }

    #[test]
    fn generic_qpsk() {
        let input: Vec<u8> = [0, 0, 1, 0, 0, 1, 1, 1].iter().map(|i| *i as u8).collect();

        let output = qpsk().modulate(&input);

        assert_eq!(output.as_slice(), &GENERIC_QPSK_TABLE);
    }

    #[test]
    fn naive_demod() {
        let m = qpsk();

        // generate some ones and zeroes
        for seed in &[815u64, 234354654543, 18324357] {
            let mut r = StdRng::seed_from_u64(*seed);
            let bits = (0..100).map(|_| r.gen_range(0u8, 1u8)).collect::<Vec<_>>();
            let output = m.modulate(&bits);
            let mut demod_bits = Vec::with_capacity(100);
            m.demod_naive(&mut output.iter(), &mut demod_bits);
            assert_eq!(bits, demod_bits);
        }
    }
}
