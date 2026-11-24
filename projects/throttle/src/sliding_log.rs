use std::collections::VecDeque;
use std::time::Duration;

use crate::{Clock, Decision, Limiter};

/// Exact sliding window: remembers every admitted request for `window`.
/// Precise, but memory grows with the limit.
#[derive(Debug)]
pub struct SlidingLog<C: Clock> {
    clock: C,
    limit: u32,
    window: Duration,
    entries: VecDeque<(Duration, u32)>,
    used: u32,
}

impl<C: Clock> SlidingLog<C> {
    pub fn new(clock: C, limit: u32, window: Duration) -> Self {
        assert!(limit > 0, "limit must be positive");
        assert!(!window.is_zero(), "window must be non-zero");
        Self { clock, limit, window, entries: VecDeque::new(), used: 0 }
    }

    fn evict(&mut self, now: Duration) {
        while let Some(&(at, cost)) = self.entries.front() {
            if now.saturating_sub(at) >= self.window {
                self.used -= cost;
                self.entries.pop_front();
            } else {
                break;
            }
        }
    }
}

impl<C: Clock> Limiter for SlidingLog<C> {
    fn try_acquire_n(&mut self, cost: u32) -> Decision {
        if cost > self.limit {
            return Decision::Denied { retry_after: Duration::MAX };
        }
        let now = self.clock.now();
        self.evict(now);
        if self.used + cost <= self.limit {
            self.entries.push_back((now, cost));
            self.used += cost;
            return Decision::Allowed;
        }
        // Find the earliest moment enough old entries will have expired.
        let mut freed = 0;
        for &(at, c) in &self.entries {
            freed += c;
            if self.used - freed + cost <= self.limit {
                return Decision::Denied { retry_after: (at + self.window) - now };
            }
        }
        unreachable!("cost <= limit, so expiring every entry always makes room")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ManualClock;

    #[test]
    fn never_exceeds_limit_in_any_window() {
        let clock = ManualClock::new();
        let mut l = SlidingLog::new(clock.clone(), 5, Duration::from_secs(10));
        clock.advance(Duration::from_millis(9_999));
        assert!(l.try_acquire_n(5).is_allowed());
        clock.advance(Duration::from_millis(1));
        // Unlike a fixed window, the burst just before is still counted.
        assert!(!l.try_acquire().is_allowed());
    }

    #[test]
    fn retry_after_points_at_oldest_expiry() {
        let clock = ManualClock::new();
        let mut l = SlidingLog::new(clock.clone(), 2, Duration::from_secs(10));
        assert!(l.try_acquire().is_allowed());
        clock.advance(Duration::from_secs(4));
        assert!(l.try_acquire().is_allowed());
        assert_eq!(l.try_acquire(), Decision::Denied { retry_after: Duration::from_secs(6) });
    }

    #[test]
    fn capacity_returns_as_entries_expire() {
        let clock = ManualClock::new();
        let mut l = SlidingLog::new(clock.clone(), 2, Duration::from_secs(10));
        l.try_acquire();
        clock.advance(Duration::from_secs(5));
        l.try_acquire();
        clock.advance(Duration::from_secs(5)); // first entry expires
        assert!(l.try_acquire().is_allowed());
        assert!(!l.try_acquire().is_allowed());
    }

    #[test]
    fn multi_unit_request_waits_for_enough_room() {
        let clock = ManualClock::new();
        let mut l = SlidingLog::new(clock.clone(), 4, Duration::from_secs(10));
        l.try_acquire_n(1);
        clock.advance(Duration::from_secs(2));
        l.try_acquire_n(3);
        // Needs 2 free; the first entry (1 unit) expiring is not enough, so the
        // wait runs until the second entry (admitted at t=2s) expires at t=12s.
        assert_eq!(l.try_acquire_n(2), Decision::Denied { retry_after: Duration::from_secs(10) });
    }
}
