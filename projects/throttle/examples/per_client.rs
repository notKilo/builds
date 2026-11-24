use std::time::Duration;

use throttle::{Gcra, Keyed, SystemClock};

fn main() {
    let clock = SystemClock::new();
    let factory_clock = clock.clone();
    // 5 requests/s with bursts of 10, tracked separately per client; idle clients are forgotten after 5 minutes.
    let mut limiter = Keyed::new(clock, Duration::from_secs(300), move || {
        Gcra::new(factory_clock.clone(), 5.0, 10)
    });
    for i in 0..12 {
        let decision = limiter.try_acquire("203.0.113.7");
        println!("request {i}: {decision:?}");
    }
    println!("other client: {:?}", limiter.try_acquire("198.51.100.2"));
}
