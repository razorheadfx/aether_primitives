fn main() {
    #[cfg(not(feature = "plot"))]
    {
        println!("This example requires the plot feature and gnuplot must be installed!");
        println!("Try it with: cargo run --example plotting --features=\"plot\"");
    }

    #[cfg(feature = "plot")]
    {
        use aether_primitives::cf32;
        use aether_primitives::channel::noise;
        use aether_primitives::util::plot;

        println!("Generating noise and plotting constellation");
        let mut noise = noise::generator();
        let mut n = Vec::with_capacity(2048);
        noise.fill(&mut n);
        let use_db = false;
        let no_file_out = None;
        plot::constellation(&n, "2048 Noise Values", use_db, no_file_out);
    }
}
