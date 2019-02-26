use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::{Duration, SystemTime};

/// A thread-based object processing pipeline connected via mpsc channels used in
/// single-producer single-consumer fashion.
/// Creating and adding stages is not lazy, each stage spawns its thread when adding it.
/// Stages die when either Receiver or Sender dies.
/// Stages operate in blocking fashion, thus generate no CPU load if they do not run.
/// Stages will try to report load and number of processed objects every second.
/// It is very important to
pub struct Pipeline<I, O>
where
    I: Send,
    O: Send,
{
    input: Sender<I>,
    last_stage_output: Receiver<O>,
}

impl<I, O> Pipeline<I, O>
where
    I: Send,
    O: Send + 'static,
{
    /// add another stage to the pipeline
    pub fn add_stage<F: 'static + Send + FnMut(O) -> U, U: 'static + Send>(
        self,
        name: &str,
        op: F,
    ) -> Pipeline<I, U> {
        let input = self.input;
        let next_stage_input = self.last_stage_output;
        let next_stage_output = spawn_stage(name, next_stage_input, op);

        Pipeline {
            input,
            last_stage_output: next_stage_output,
        }
    }

    /// Consumes the pipeline builder and returns the sender used to
    /// feed the pipeline and the receiver used to take processed objects
    /// out of the pipeline
    pub fn finish(self) -> (Sender<I>, Receiver<O>) {
        let i = self.input;
        let o = self.last_stage_output;
        (i, o)
    }
}

/// This performs the actual setup and spawning for pipeline stages
fn spawn_stage<I, O, F>(name: &str, input: Receiver<I>, op: F) -> Receiver<O>
where
    I: Send + 'static,
    O: Send + 'static,
    F: Send + 'static + FnMut(I) -> O,
{
    let (o_tx, o) = channel();
    let name = name.to_string();
    let mut op = op;

    thread::spawn(move || {
        // OPT: here core pinning could happen
        println!("Stage: {:15} :up", name);
        let mut n = 0u64;
        let mut last_report = SystemTime::now();
        let mut time_active = Duration::from_secs(0);
        loop {
            let (i, s) = match input.recv() {
                Ok(i) => (i, SystemTime::now()),
                _ => break,
            };

            // perform the operation
            let v = op(i);

            match o_tx.send(v) {
                Ok(_) => (),
                Err(_) => break,
            };

            // log end time
            let e = SystemTime::now();
            // update the number of things processed
            n += 1;
            time_active += e.duration_since(s).unwrap_or(Duration::from_secs(0));

            // report every second
            let dur = e
                .duration_since(last_report)
                .unwrap_or(Duration::from_secs(0));
            if dur >= Duration::from_secs(1) {
                // ms precision is ok
                let dur_in_ms = (1000 * dur.as_secs()) as f64 + dur.subsec_millis() as f64;
                let active_in_ms =
                    (1000 * time_active.as_secs()) as f64 + time_active.subsec_millis() as f64;
                let ops_per_s = n as f64 / dur_in_ms * 1000.0;
                let utilisation = active_in_ms / dur_in_ms * 100.0;
                println!(
                    "Stage: {:15} : Processed {} in {:3.3}s ({:9.2}/s); Utilisation: {:3.2}%",
                    name,
                    n,
                    dur_in_ms / 1000.0,
                    ops_per_s,
                    utilisation
                );

                // reset our stats
                // assumes producing and printing the report requires no time
                last_report = e;
                n = 0u64;
                time_active = Duration::from_secs(0);
            }
        }
        println!("Stage: {:15} :down", name);
    });
    o
}

/// This creates a new thread-based processing pipeline
// OPT: add option to pin threads to cores
pub fn new<I, O, F>(name: &str, op: F) -> Pipeline<I, O>
where
    I: Send + 'static,
    O: Send + 'static,
    F: Send + 'static + FnMut(I) -> O,
{
    let (input, i_rx) = channel();

    let last_stage_output = spawn_stage(name, i_rx, op);

    Pipeline {
        input,
        last_stage_output,
    }
}
