extern crate aether_primitives as aeth;
use aeth::pipeline;
use aeth::pool::{self, Elem};
use std::env::args;
use std::mem::size_of;
use std::str::FromStr;
use std::thread;
use std::time::{Duration, Instant};
const POOLSIZE: usize = 150;
const BUFFSIZE: usize = 128;

fn main() {
    fn pooled() {
        println!("------------------------------------------");
        println!("Using pre-allocated pool and a synchronuous pipeline");

        let poolsize = args()
            .nth(1)
            .and_then(|x| usize::from_str(&x).ok())
            .unwrap_or(POOLSIZE);
        let buffsize = args()
            .nth(2)
            .and_then(|x| usize::from_str(&x).ok())
            .unwrap_or(BUFFSIZE);

        let buff_bytes = size_of::<f32>() * buffsize;
        let psize = poolsize.clone();
        println!("Using Poolsize of {}", poolsize);
        println!(
            "Using Buffsize of {} ({} kB/buffer, {} kB total)",
            buffsize,
            buff_bytes as f32 / 1e3,
            buff_bytes as f32 * poolsize as f32 / 1e3
        );
        let t = Instant::now();
        let p = pool::make::<Vec<f32>>(
            poolsize,
            Box::new(move || vec![1.0f32; psize]),
            Box::new(|v| v.clear()),
        );

        println!("Pool creation took {:#?}", t.elapsed());

        let (i, o) = pipeline::new("Abs", |mut v: Elem<Vec<f32>>| {
            v.iter_mut().for_each(|c| *c = c.abs());
            v
        })
        .add_stage("Mul 20", |mut v| {
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
                let mut v = loop {
                    if let Some(v) = p.take() {
                        break v;
                    }
                };

                v.extend((0..buffsize).map(|x| -1.0 * x as f32));

                match i.send(v) {
                    Err(e) => {
                        println!("{:?}", e);
                        break;
                    }
                    Ok(_) => tx_ed += 1,
                };
                // thread::sleep(Duration::from_micros(500));
            }
            println!("Fed Elements {}", tx_ed);
            tx_ed
        });

        while let Ok(_ov) = o.recv() {
            rx_ed += 1;
        }
        println!("RXed: {}", rx_ed);
        handle.join().unwrap();
    }

    fn pooled_or_make() {
        println!("------------------------------------------");
        println!("Using growing pool and a synchronuous pipeline");

        let poolsize = args()
            .nth(1)
            .and_then(|x| usize::from_str(&x).ok())
            .unwrap_or(0);
        let buffsize = args()
            .nth(2)
            .and_then(|x| usize::from_str(&x).ok())
            .unwrap_or(BUFFSIZE);

        let buff_bytes = size_of::<f32>() * buffsize;
        let psize = poolsize.clone();
        println!("Using Poolsize of {}", poolsize);
        println!(
            "Using Buffsize of {} ({} kB/buffer, {} kB total)",
            buffsize,
            buff_bytes as f32 / 1e3,
            buff_bytes as f32 * poolsize as f32 / 1e3
        );
        let t = Instant::now();
        let p = pool::make::<Vec<f32>>(
            poolsize,
            Box::new(move || vec![1.0f32; psize]),
            Box::new(|v| v.clear()),
        );

        println!("Pool created in {:#?}", t.elapsed());

        let (i, o) = pipeline::new("Abs", |mut v: Elem<Vec<f32>>| {
            v.iter_mut().for_each(|c| *c = c.abs());
            v
        })
        .add_stage("Mul 20", |mut v| {
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
                let mut v = p.take_or_make();

                v.extend((0..buffsize).map(|x| -1.0 * x as f32));

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
            println!("Fed: {}", tx_ed);
            tx_ed
        });

        while let Ok(_ov) = o.recv() {
            rx_ed += 1;
        }
        println!("RXed: {}", rx_ed);
        handle.join().unwrap();
    }

    fn unpooled() {
        let buffsize = args()
            .nth(2)
            .and_then(|x| usize::from_str(&x).ok())
            .unwrap_or(BUFFSIZE);

        let buff_bytes = size_of::<f32>() * buffsize;
        println!(
            "Using Buffsize of {} ({} kB/buffer",
            buffsize,
            buff_bytes as f32 / 1e3
        );

        println!("------------------------------------------");
        println!("Using no pool and synchronuous pipeline");

        let (i, o) = pipeline::new("Abs", |mut v: Vec<f32>| {
            v.iter_mut().for_each(|c| *c = c.abs());
            v
        })
        .add_stage("Mul 20", |mut v| {
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
                let mut v = vec![0.0f32; buffsize];
                v.clear();
                v.extend((0..buffsize).map(|x| -1.0 * x as f32));

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
            println!("Fed Elements: {}", tx_ed);
            tx_ed
        });
        while let Ok(_ov) = o.recv() {
            rx_ed += 1;
        }
        println!("RXed: {}", rx_ed);
        handle.join().unwrap();
    }

    thread::sleep(Duration::from_secs(2));
    pooled();
    thread::sleep(Duration::from_secs(2));
    pooled_or_make();
    thread::sleep(Duration::from_secs(2));
    unpooled();
}
