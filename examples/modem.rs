extern crate aether_primitives as aeth;
extern crate rand;

#[cfg(not(feature = "plot"))]
fn main() {
    println!("This example requires the \"plot\" feature and an installation of gnuplot.");
    println!("Try it with: cargo run --example plotting --features=\"plot\"");
}

#[cfg(feature = "plot")]
fn main() {
    use aeth::{modulation, modulation::Modulation, noise, plot};
    use rand::prelude::*;

    let m = modulation::qpsk();

    let mut r = thread_rng();

    // generate some ones and zeroes
    let b = (0..100).map(|_| r.gen_range(0u8, 2u8)).collect::<Vec<_>>();
    println!("Input: {:#?}", b);
    // modulate them
    let mut output = m.modulate(&b);
    // add some noise
    let mut n = noise::new(0.01, 815);
    n.apply(&mut output);
    println!("Output: {:#?}", output);

    let mut bits = vec![0u8; 100];

    m.demod_naive(&mut output.iter(), &mut bits.iter_mut());
    assert_eq!(b, bits);

    plot::time(&output, "m", None);

    plot::constellation(&output, "Modulated bits", None);
}
