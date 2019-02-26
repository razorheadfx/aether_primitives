fn main() {
    #[cfg(any(not(feature = "fft"), not(feature = "gui")))]
    {
        println!("This example requires the \"gui\" and \"fft\" features.");
        println!("Try again with: cargo run --example plotting --features=\"gui fft_rustfft\"");
    }

    #[cfg(all(feature = "gui", feature = "fft"))]
    {
        use aether_primitives::cf32;
        use aether_primitives::channel::noise;
        use aether_primitives::gui;
        use aether_primitives::util::DB;
        use std::f32::consts::PI;
        use std::sync::mpsc;
        use std::thread;
        use std::time::{Duration, SystemTime};

        let bins = 1024;

        let (s, r) = mpsc::channel::<Vec<cf32>>();

        let w = gui::waterfall(bins, Some((-50.0, 10.0)));
        let h = gui::launch(r, w);

        let mut noise = noise::new(DB(-20.0).ratio() as f32, 815);

        let duration = 10;
        let start = SystemTime::now();
        let mut f = 10e3;
        while start.elapsed().unwrap().as_secs() < duration {
            let v = (0..bins)
                .map(|x| x as f32)
                .map(|t| 2.0 * PI * f * t)
                .map(|arg| cf32::from_polar(&1.0, &arg))
                .zip(noise.iter())
                .map(|(a, b)| a + b)
                .collect::<Vec<_>>();

            match s.send(v) {
                Ok(_) => (),
                Err(_) => {
                    println!("Receiver dropped; Did you click the exit button? Or Press Esc?");
                    break;
                }
            }
            thread::sleep(Duration::from_millis(100));
            f += 100.0;
        }
        drop(s); // drop the sender so the gui thread dies
        h.join().expect("Failed to rejoin the thread");
    }
}
