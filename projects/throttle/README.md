# throttle

Rate-limiting algorithms for Rust behind one `Limiter` trait, with an injectable
clock so limiter behaviour is testable without sleeping. No dependencies.

**Status: in progress.** Implemented so far: token bucket, fixed window,
sliding-window log, sliding-window counter, GCRA, and a keyed (per-client)
wrapper. Still to come: thread-safe wrappers, a demo CLI, benchmarks and a
strategy comparison.

```rust
use std::time::Duration;
use throttle::{Decision, Limiter, SystemClock, TokenBucket};

let mut limiter = TokenBucket::new(SystemClock::new(), 5, 10.0); // burst 5, 10/s
match limiter.try_acquire() {
    Decision::Allowed => { /* do the work */ }
    Decision::Denied { retry_after } => println!("retry in {retry_after:?}"),
}
```

Run the tests with `cargo test`.
