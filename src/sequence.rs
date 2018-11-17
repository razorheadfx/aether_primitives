/// Unpack an unsigned integer into a binary sequence of {0,1} u8
/// This function is meant to provide seed/initialisation values for M-sequence generators.
/// __Example__
/// ```
/// use aether_primitives::sequence;
/// // set bits 1,3,5
/// let seed = 1 + 4 + 16;
/// // take 32 values from that
/// let expanded = sequence::expand(seed, 32);
/// let mut expected = vec![0u8; 32];
/// expected.iter_mut().zip([1,0,1,0,1].iter())
///     .for_each(|(s,x)| *s = *x as u8);
///
/// assert_eq!(expanded, expected, "Seed expansion failed");
/// assert_eq!(expanded.len(), 32usize)
/// ```
pub fn expand(seed : u64, len: usize) -> Vec<u8>{
    // left shift seed value and use binary & to extract the bit we're interested in
    (0..len).map(|i| (seed >> i & 1) as u8).collect()
}

/// Generate a M-Sequence based on initial values and a number of taps
/// Such sequences are common building blocks for scrambling, synchronisation or modulation
/// (Direct-Sequence Spread-Spectrum) systems.
/// ```seed``` : 
/// ```generator```: a function that generates elements,
/// may rely on values currently within the sequence,
// is also provided the current position to be filled by the new value
/// ```len``` : length of the 
/// __Example__
/// ```
/// use aether_primitives::sequence;
/// // This example generates one half of the pseudo-random sequence
/// // used by LTE's physical layer as per 3GPP TS36.211 7.2
/// // It is provided in the form
/// // ``` x1(n+31) = (x1(n+3) + x1(n)) mod2 ```
/// // Since we use an array to generate that we need to indices
/// // Every step should yield one x(n) with n in [32..1600]
/// // ```x(n) = (x(n+3-31) + x(n-31)) mod2 ```
/// // Thus our generator is this
/// let gen = |n : usize, seq : &[u8] | (seq[n-28] + seq[n-31]) % 2;
/// // get an init vector
/// let mut init = sequence::expand(1,31);
/// let seq = sequence::generate(init, gen, 1600);
/// assert_eq!(seq.len(), 1600);
/// // not going to check these values here
pub fn generate(mut init : Vec<u8>, generator : impl Fn(usize,&[u8]) -> u8, len : usize) -> Vec<u8>{
    while init.len() < len {
        let next_elem = generator(init.len(), init.as_ref());
        init.push(next_elem);
    }
    init
}



#[cfg(test)]
mod test{
    use crate::sequence;

    #[test]
    /// A simple sequence
    fn simple_sequence(){
        let gen = |n : usize, s : &[u8]| (s[n-1] + s[n-2]) % 2;

        let seed = vec![1,0];
        let seq = sequence::generate(seed, gen, 6);

        assert_eq!(seq, vec![1,0,1,1,0,1]);
    }

}