# Choosing a strategy

All five limiters implement `Limiter` and are built with the same two numbers: a
sustained rate and a burst size. Window strategies use a limit of `burst` per
window of `burst / rate` seconds.

| Strategy | State per limiter | Burst behaviour | Accuracy | Notes |
|---|---|---|---|---|
| `TokenBucket` | tokens + last refill time | up to `capacity` at once, then the refill rate | exact | The default choice. Easy to explain to API users. |
| `Gcra` | one timestamp | same as token bucket | exact | Admits exactly what a token bucket would, in a single `Duration`. Best for per-key limiting. |
| `FixedWindow` | counter + window start | up to 2x the limit across a window boundary | coarse | Cheapest, but a client can send `limit` at the end of one window and `limit` at the start of the next. |
| `SlidingLog` | timestamp per admitted request | never exceeds `limit` in any window | exact | Memory grows with the limit. Use for small limits where exactness matters. |
| `SlidingCounter` | two counters + window start | close to `limit` in any window | approximate | Weights the previous window by overlap. Slightly conservative; memory is constant. |

## Rules of thumb

- Need bursts and a steady rate: `TokenBucket` or `Gcra`.
- Limiting millions of keys: `Gcra` inside `Keyed` (one `Duration` each, idle keys evicted).
- Need a hard "no more than N in any 60 seconds" guarantee with small N: `SlidingLog`.
- Need that guarantee cheaply and can tolerate a few percent error: `SlidingCounter`.
- Quota that resets on the clock (daily/hourly billing quotas): `FixedWindow`.

Try them side by side with `cargo run --bin throttle-demo -- --rate 5 --burst 5 --traffic 20`.

## Thread safety

Limiters take `&mut self`. To share one across threads:

- `Shared<L>` wraps any limiter in an `Arc<Mutex<_>>`. Use it by default.
- `AtomicGcra` has the same admission rule as `Gcra` but is lock-free: its state is one
  atomic timestamp updated with compare-and-swap, and it takes `&self`.

## Benchmarks

`cargo run --release --example bench`. Numbers from a 4-core cloud VM, so treat them as
relative, not absolute.

Single thread, per decision (manual clock, so clock reads are nearly free):

| Strategy | ns/op |
|---|---|
| token-bucket | 37 |
| fixed-window | 20 |
| sliding-log | 21 |
| sliding-counter | 45 |
| gcra | 22 |

Contended, real clock, million decisions per second in total:

| Threads | `Shared<Gcra>` | `AtomicGcra` |
|---|---|---|
| 1 | 19.5 | 17.8 |
| 2 | 6.2 | 5.8 |
| 4 | 4.1 | 3.4 |
| 8 | 4.3 | 3.6 |

The lock-free version did not win here. Every thread hammers one cache line either way,
so the cost is cache-line bouncing rather than lock waiting, and the CAS retry loop adds a
little on top. `AtomicGcra` is still useful when you cannot hold a lock (for example
inside a signal-free async hot path) or want `&self` access, but do not expect it to be
faster than a mutex for a single hot limiter. Spreading load over many keys helps far
more than either choice.
