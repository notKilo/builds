//! Micro benchmarks: `cargo run --release --example bench`.
//!
//! Single-threaded cost per decision for each strategy (manual clock, so the
//! clock read is nearly free), then contended throughput for the two
//! thread-safe options.

use std::hint::black_box;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use throttle::sim::{build, STRATEGIES};
use throttle::{AtomicGcra, Gcra, ManualClock, Shared, SystemClock};

const SINGLE_OPS: u64 = 5_000_000;
const CONTENDED_OPS_PER_THREAD: u64 = 1_000_000;

fn main() {
    println!("single thread, {SINGLE_OPS} decisions each");
    for name in STRATEGIES {
        let clock = ManualClock::new();
        let mut limiter = build(name, clock.clone(), 1000.0, 100).unwrap();
        let start = Instant::now();
        for _ in 0..SINGLE_OPS {
            clock.advance(Duration::from_micros(500));
            black_box(limiter.try_acquire());
        }
        let ns = start.elapsed().as_nanos() as f64 / SINGLE_OPS as f64;
        println!("  {name:<16} {ns:>7.1} ns/op");
    }

    println!("\ncontended, {CONTENDED_OPS_PER_THREAD} decisions per thread (real clock)");
    for threads in [1, 2, 4, 8] {
        let mutex = Shared::new(Gcra::new(SystemClock::new(), 1e9, 1000));
        let m = run(threads, move || {
            let l = mutex.clone();
            Box::new(move || {
                black_box(l.try_acquire());
            })
        });
        let atomic = Arc::new(AtomicGcra::new(SystemClock::new(), 1e9, 1000));
        let a = run(threads, move || {
            let l = Arc::clone(&atomic);
            Box::new(move || {
                black_box(l.try_acquire());
            })
        });
        println!("  {threads} threads: Shared<Gcra> {m:>6.1} Mops/s   AtomicGcra {a:>6.1} Mops/s");
    }
}

/// Runs `threads` workers, each calling the closure from `make` repeatedly, and
/// returns total millions of operations per second.
fn run(threads: u32, make: impl Fn() -> Box<dyn FnMut() + Send>) -> f64 {
    let start = Instant::now();
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let mut op = make();
            thread::spawn(move || {
                for _ in 0..CONTENDED_OPS_PER_THREAD {
                    op();
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    let total = threads as f64 * CONTENDED_OPS_PER_THREAD as f64;
    total / start.elapsed().as_secs_f64() / 1e6
}
