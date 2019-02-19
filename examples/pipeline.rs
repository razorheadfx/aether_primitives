extern crate aether_primitives as aeth;
use aeth::pipeline;

use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let (i, o) = pipeline::new("Take absolute", |v: Vec<f32>| {
        v.iter().map(|c| c.abs()).collect::<Vec<_>>()
    })
    .add_stage("Multiply by 20", |mut v| {
        v.iter_mut().for_each(|c| *c = *c * 20.0);
        v
    })
    .finish();

    let end = Instant::now() + Duration::from_secs(10);
    let mut rx_ed = 0;

    let handle = thread::spawn(move || {
        let mut tx_ed = 0;
        println!("Feeder up");
        while Instant::now() < end {
            // if you use larger buffers you may run out of memory because a backlog may bould up
            // mainly because the "Take absolute" stage allocates another buffer and put quite the
            // load on the allocator
            // In general: Input stages should be rate limited and if any of the downstream stages
            // shows 100% load you should consider splitting the load
            let v = (0..128).map(|x| -1.0 * x as f32).collect();
            match i.send(v) {
                Err(e) => {
                    println!("{:?}", e);
                    break;
                }
                Ok(_) => tx_ed += 1,
            };
            // thread::sleep(Duration::from_micros(500));
        }
        println!("Feeder down");
        tx_ed
    });

    // drain messages
    while let Ok(_ov) = o.recv() {
        rx_ed += 1;
    }

    let tx_ed = handle.join().unwrap();

    println!("Rxed :{} Txed: {}", rx_ed, tx_ed);
}
