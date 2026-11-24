use std::time::Duration;

use crate::{Clock, Decision, Limiter};

/// Generic Cell Rate Algorithm. Equivalent to a token bucket in what it admits,
/// but stores a single timestamp (the "theoretical arrival time") per limiter,
/// which makes it a good fit for per-key limiting.
#[derive(Debug)]
pub struct Gcra<C: Clock> {
    clock: C,
    interval: Duration,
    burst: u32,
    tat: Duration,
}

impl<C: Clock> Gcra<C> {
    /// Sustains `rate_per_sec` requests per second and tolerates bursts of `burst`.
    pub fn new(clock: C, rate_per_sec: f64, burst: u32) -> Self {
        assert!(rate_per_sec > 0.0, "rate must be positive");
        assert!(burst > 0, "burst must be positive");
        let tat = clock.now();
        Self {
            clock,
            interval: Duration::from_secs_f64(1.0 / rate_per_sec),
            burst,
            tat,
        }
    }
}

impl<C: Clock> Limiter for Gcra<C> {
    fn try_acquire_n(&mut self, cost: u32) -> Decision {
        if cost > self.burst {
            return Decision::Denied {
                retry_after: Duration::MAX,
            };
        }
        let now = self.clock.now();
        let new_tat = self.tat.max(now) + self.interval * cost;
        let allow_at = new_tat.saturating_sub(self.interval * self.burst);
        if now >= allow_at {
            self.tat = new_tat;
            Decision::Allowed
        } else {
            Decision::Denied {
                retry_after: allow_at - now,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ManualClock, TokenBucket};

    #[test]
    fn allows_burst_then_denies() {
        let clock = ManualClock::new();
        let mut g = Gcra::new(clock.clone(), 10.0, 5);
        for _ in 0..5 {
            assert!(g.try_acquire().is_allowed());
        }
        assert_eq!(
            g.try_acquire(),
            Decision::Denied {
                retry_after: Duration::from_millis(100)
            }
        );
    }

    #[test]
    fn recovers_one_slot_per_interval() {
        let clock = ManualClock::new();
        let mut g = Gcra::new(clock.clone(), 10.0, 5);
        g.try_acquire_n(5);
        clock.advance(Duration::from_millis(100));
        assert!(g.try_acquire().is_allowed());
        assert!(!g.try_acquire().is_allowed());
    }

    #[test]
    fn idle_time_does_not_bank_more_than_the_burst() {
        let clock = ManualClock::new();
        let mut g = Gcra::new(clock.clone(), 10.0, 3);
        clock.advance(Duration::from_secs(3600));
        assert!(g.try_acquire_n(3).is_allowed());
        assert!(!g.try_acquire().is_allowed());
    }

    #[test]
    fn matches_token_bucket_decisions() {
        let c1 = ManualClock::new();
        let c2 = ManualClock::new();
        let mut g = Gcra::new(c1.clone(), 4.0, 6);
        let mut t = TokenBucket::new(c2.clone(), 6, 4.0);
        // Deterministic pseudo-random arrivals and costs.
        let mut x: u32 = 12345;
        for _ in 0..500 {
            x = x.wrapping_mul(1664525).wrapping_add(1013904223);
            let gap = Duration::from_millis(((x >> 8) % 600) as u64);
            let cost = 1 + (x >> 20) % 3;
            c1.advance(gap);
            c2.advance(gap);
            assert_eq!(
                g.try_acquire_n(cost).is_allowed(),
                t.try_acquire_n(cost).is_allowed()
            );
        }
    }

    #[test]
    fn oversized_request_is_never_allowed() {
        let clock = ManualClock::new();
        let mut g = Gcra::new(clock.clone(), 10.0, 5);
        assert_eq!(
            g.try_acquire_n(6),
            Decision::Denied {
                retry_after: Duration::MAX
            }
        );
    }
}
