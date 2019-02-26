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
    fn modulate_into(&self, input: &[u8], output: &mut Iterator<Item = &mut cf32>) {
        input
            .chunks(Self::BITS_PER_SYMBOL)
            .map(Self::index)
            .map(|idx| self.symbol(idx))
            .zip(output)
            .for_each(|(s, out)| *out = s);
    }

    fn demod_naive(
        &self,
        symbols: &mut Iterator<Item = &cf32>,
        mut output: &mut Iterator<Item = &mut u8>,
    ) {
        for symbol in symbols {
            let (idx, _symb) = (0..Self::BITS_PER_SYMBOL * 2)
                .map(|idx| *symbol - self.symbol(idx))
                .map(|dist| dist.re * dist.re + dist.im * dist.im)
                .enumerate()
                .min_by(|(_i, d), (_j, e)| d.partial_cmp(e).unwrap_or(Ordering::Greater))
                .expect("Finding the minimum symbol distance failed");

            (0..Self::BITS_PER_SYMBOL)
                .map(|i| idx as u8 >> i & 1u8)
                .zip(&mut output)
                .for_each(|(b, out)| *out = b);
        }
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
            let bits = (0..100).map(|_| r.gen_range(0u8, 2u8)).collect::<Vec<_>>();
            let output = m.modulate(&bits);
            let mut demod_bits = vec![0u8; 100];
            m.demod_naive(&mut output.iter(), &mut demod_bits.iter_mut());
            assert_eq!(bits, demod_bits);
        }
    }
}
