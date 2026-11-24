# throttle

Rate-limiting algorithms for Rust behind one `Limiter` trait, with an injectable
clock so limiter behaviour is testable without sleeping. No dependencies.

- `TokenBucket`, `Gcra`, `FixedWindow`, `SlidingLog`, `SlidingCounter`
- `Keyed`: one limiter per client id, with idle-key eviction
- `Shared<L>`: mutex-backed handle for sharing any limiter across threads
- `AtomicGcra`: lock-free GCRA taking `&self`
- `ManualClock` for deterministic tests, `SystemClock` for production
- `throttle-demo` CLI and a benchmark example

See [docs/strategies.md](docs/strategies.md) for how to choose between them, with benchmark numbers.

## Install

Not published to crates.io; use it as a path or git dependency:

```toml
[dependencies]
throttle = { git = "https://github.com/notKilo/builds", package = "throttle" }
```

## Usage

```rust
use throttle::{Decision, Limiter, SystemClock, TokenBucket};

let mut limiter = TokenBucket::new(SystemClock::new(), 5, 10.0); // burst 5, 10/s
match limiter.try_acquire() {
    Decision::Allowed => { /* do the work */ }
    Decision::Denied { retry_after } => println!("retry in {retry_after:?}"),
}
```

Per-client limiting (full program in [examples/per_client.rs](examples/per_client.rs)):

```rust
let mut limiter = Keyed::new(clock, Duration::from_secs(300), move || {
    Gcra::new(factory_clock.clone(), 5.0, 10)
});
limiter.try_acquire("203.0.113.7"); // each key gets its own Gcra
```

Wrap any limiter in `Shared::new(..)` to use it from several threads.

Compare strategies on the same traffic:

```console
$ cargo run -q --bin throttle-demo -- --strategy gcra --seconds 2
== gcra (rate 5/s, burst 5) ==
t=  0s    9/20  ######..#...#...#...
t=  1s    5/20  #...#...#...#...#...
admitted 14 of 40
```

`#` is an admitted request, `.` a denied one.

## Develop

```console
cargo test
cargo clippy --all-targets -- -D warnings
cargo run --release --example bench
```
