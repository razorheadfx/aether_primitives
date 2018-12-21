fn main() {
    #[cfg(not(feature = "plot"))]
    {
        println!("This example requires the \"plot\" feature and an installation of gnuplot.");
        println!("Try it with: cargo run --example plotting --features=\"plot\"");
    }

    #[cfg(feature = "plot")]
    {
        use aether_primitives::channel::noise;
        use aether_primitives::plot;

        {
            println!("Generating noise and plotting constellation");
            let noise = noise::make(2048, 1.0);
            let no_file_out = None;
            plot::constellation(&noise, "2048 Noise Values", no_file_out);
        }
        {
            println!("Generating noise and plotting time signal");
            let noise = noise::make(200, 1.0);
            let no_file_out = None;
            plot::time(&noise, "200 Noise Values", no_file_out);
        }
        {
            println!("Generating noise and plotting comparison");
            let noise = noise::make(400, 1.0);
            let no_file_out = None;
            plot::compare(
                &noise[..200],
                &noise[200..],
                "200 Noise Values",
                no_file_out,
            );
        }

        #[cfg(feature = "fft")]
        {
            println!("Generating noise and waterfall");
            let fft_len = 2048;
            let noise = noise::make(fft_len * 500, 1.0);
            let use_db = true;
            let no_file_out = None;
            plot::waterfall(
                &noise,
                fft_len,
                use_db,
                "500*2048 Noise Values",
                no_file_out,
            );
        }

        #[cfg(not(feature = "fft"))]
        {
            println!("Skipping waterfall plot; Enable by enabling feature fft_chfft");
        }
    }
}
