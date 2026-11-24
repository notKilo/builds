use std::time::Duration;

use crate::{Clock, Decision, Limiter};

/// Classic token bucket: holds up to `capacity` tokens and refills continuously.
/// Allows bursts up to `capacity`, then a steady `rate` per second.
#[derive(Debug)]
pub struct TokenBucket<C: Clock> {
    clock: C,
    capacity: f64,
    rate_per_sec: f64,
    tokens: f64,
    last: Duration,
}

impl<C: Clock> TokenBucket<C> {
    /// Starts full. Panics if `capacity` is zero or `rate_per_sec` is not positive.
    pub fn new(clock: C, capacity: u32, rate_per_sec: f64) -> Self {
        assert!(capacity > 0, "capacity must be positive");
        assert!(rate_per_sec > 0.0, "rate must be positive");
        let last = clock.now();
        Self {
            clock,
            capacity: capacity as f64,
            rate_per_sec,
            tokens: capacity as f64,
            last,
        }
    }

    fn refill(&mut self) {
        let now = self.clock.now();
        let elapsed = now.saturating_sub(self.last).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.rate_per_sec).min(self.capacity);
        self.last = now;
    }
}

impl<C: Clock> Limiter for TokenBucket<C> {
    fn try_acquire_n(&mut self, cost: u32) -> Decision {
        let cost = cost as f64;
        if cost > self.capacity {
            return Decision::Denied {
                retry_after: Duration::MAX,
            };
        }
        self.refill();
        if self.tokens >= cost {
            self.tokens -= cost;
            Decision::Allowed
        } else {
            let wait = (cost - self.tokens) / self.rate_per_sec;
            Decision::Denied {
                retry_after: Duration::from_secs_f64(wait),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ManualClock;

    fn bucket(clock: &ManualClock) -> TokenBucket<ManualClock> {
        TokenBucket::new(clock.clone(), 5, 10.0)
    }

    #[test]
    fn allows_a_burst_up_to_capacity() {
        let clock = ManualClock::new();
        let mut b = bucket(&clock);
        for _ in 0..5 {
            assert!(b.try_acquire().is_allowed());
        }
        assert!(!b.try_acquire().is_allowed());
    }

    #[test]
    fn denial_reports_time_until_next_token() {
        let clock = ManualClock::new();
        let mut b = bucket(&clock);
        b.try_acquire_n(5);
        assert_eq!(
            b.try_acquire(),
            Decision::Denied {
                retry_after: Duration::from_millis(100)
            }
        );
    }

    #[test]
    fn refills_over_time_but_not_past_capacity() {
        let clock = ManualClock::new();
        let mut b = bucket(&clock);
        b.try_acquire_n(5);
        clock.advance(Duration::from_millis(250)); // 2.5 tokens
        assert!(b.try_acquire_n(2).is_allowed());
        assert!(!b.try_acquire().is_allowed());
        clock.advance(Duration::from_secs(60));
        assert!(b.try_acquire_n(5).is_allowed());
        assert!(!b.try_acquire().is_allowed());
    }

    #[test]
    fn denied_request_consumes_nothing() {
        let clock = ManualClock::new();
        let mut b = bucket(&clock);
        b.try_acquire_n(3);
        assert!(!b.try_acquire_n(4).is_allowed());
        assert!(b.try_acquire_n(2).is_allowed());
    }

    #[test]
    fn cost_above_capacity_is_never_allowed() {
        let clock = ManualClock::new();
        let mut b = bucket(&clock);
        assert_eq!(
            b.try_acquire_n(6),
            Decision::Denied {
                retry_after: Duration::MAX
            }
        );
    }
}
