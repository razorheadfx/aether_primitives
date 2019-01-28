
fn main(){
    #[cfg(not(feature = "gui"))]
    #[cfg(not(feature = "fft"))]
    {
        println!("This example requires the \"gui\" and \"fft\" features.");
        println!("Try it with: cargo run --example plotting --features=\"gui fft_chfft\"");
    }

    #[cfg(feature = "gui")]
    #[cfg(feature = "fft")]{
        use aether_primitives::cf32;
        use aether_primitives::channel::noise;
        use aether_primitives::gui;
        use aether_primitives::util::DB;
        use std::sync::mpsc;
        use std::time::{Duration, SystemTime};
        use std::thread;


        let bins = 1024;


        let (s,r) = mpsc::channel::<Vec<cf32>>();

        let w = gui::waterfall(bins,Some((-50.0,10.0)));
        let h = gui::launch(r, w);

        let mut noise = noise::new(DB(-0.0).ratio() as f32 ,815);

        let duration = 10;
        let start = SystemTime::now();
        while start.elapsed().unwrap().as_secs() < duration{
            let v = noise.iter().take(bins).collect();

            match s.send(v){
                Ok(_) => (),
                Err(_) => {
                    println!("Receiver dropped; Did you click the exit button? Or Press Esc?");
                    break;
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
        drop(s);
        h.join().expect("Failed to rejoin the thread");
    }
}