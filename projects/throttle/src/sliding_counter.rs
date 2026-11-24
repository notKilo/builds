use std::time::Duration;

use crate::{Clock, Decision, Limiter};

/// Approximate sliding window using two fixed-window counters.
///
/// The previous window's count is weighted by how much of it still overlaps
/// the sliding window. O(1) memory, and far smoother than a plain fixed window.
/// `retry_after` on denial is a lower bound: the earliest instant the estimate
/// could drop low enough, assuming no other traffic.
#[derive(Debug)]
pub struct SlidingCounter<C: Clock> {
    clock: C,
    limit: u32,
    window: Duration,
    index: u128,
    previous: u32,
    current: u32,
}

impl<C: Clock> SlidingCounter<C> {
    pub fn new(clock: C, limit: u32, window: Duration) -> Self {
        assert!(limit > 0, "limit must be positive");
        assert!(!window.is_zero(), "window must be non-zero");
        Self {
            clock,
            limit,
            window,
            index: 0,
            previous: 0,
            current: 0,
        }
    }

    fn roll(&mut self, index: u128) {
        if index == self.index {
            return;
        }
        self.previous = if index == self.index + 1 {
            self.current
        } else {
            0
        };
        self.current = 0;
        self.index = index;
    }
}

impl<C: Clock> Limiter for SlidingCounter<C> {
    fn try_acquire_n(&mut self, cost: u32) -> Decision {
        if cost > self.limit {
            return Decision::Denied {
                retry_after: Duration::MAX,
            };
        }
        let now = self.clock.now();
        let window_ns = self.window.as_nanos();
        self.roll(now.as_nanos() / window_ns);

        let elapsed = now.as_nanos() % window_ns;
        let frac = elapsed as f64 / window_ns as f64;
        let estimate = self.previous as f64 * (1.0 - frac) + self.current as f64;
        if estimate + cost as f64 <= self.limit as f64 {
            self.current += cost;
            return Decision::Allowed;
        }

        let to_rollover = Duration::from_nanos((window_ns - elapsed) as u64);
        let room = self.limit as f64 - self.current as f64 - cost as f64;
        let retry = if self.previous > 0 && room >= 0.0 {
            // Fraction of the window at which the previous count's weight is low enough.
            let needed_frac = 1.0 - room / self.previous as f64;
            let wait = Duration::from_secs_f64((needed_frac - frac) * self.window.as_secs_f64());
            wait.min(to_rollover)
        } else {
            to_rollover
        };
        Decision::Denied { retry_after: retry }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ManualClock;

    #[test]
    fn behaves_like_a_fixed_window_when_there_is_no_history() {
        let clock = ManualClock::new();
        let mut l = SlidingCounter::new(clock.clone(), 4, Duration::from_secs(10));
        assert!(l.try_acquire_n(4).is_allowed());
        assert!(!l.try_acquire().is_allowed());
    }

    #[test]
    fn previous_window_weight_decays_linearly() {
        let clock = ManualClock::new();
        let mut l = SlidingCounter::new(clock.clone(), 10, Duration::from_secs(10));
        assert!(l.try_acquire_n(10).is_allowed());
        // Halfway into the next window half of the old burst still counts: 5 of 10 used.
        clock.advance(Duration::from_secs(15));
        assert!(l.try_acquire_n(5).is_allowed());
        assert!(!l.try_acquire().is_allowed());
    }

    #[test]
    fn smooths_the_fixed_window_boundary_burst() {
        let clock = ManualClock::new();
        let mut l = SlidingCounter::new(clock.clone(), 10, Duration::from_secs(10));
        clock.advance(Duration::from_millis(9_999));
        assert!(l.try_acquire_n(10).is_allowed());
        clock.advance(Duration::from_millis(1));
        // A fixed window would admit another 10 here.
        assert!(!l.try_acquire_n(5).is_allowed());
    }

    #[test]
    fn retry_after_matches_the_decay_curve() {
        let clock = ManualClock::new();
        let mut l = SlidingCounter::new(clock.clone(), 10, Duration::from_secs(10));
        l.try_acquire_n(10);
        clock.advance(Duration::from_secs(10)); // start of window 1: previous=10, weight 1.0
                                                // Needs 1 unit: estimate must fall to 9, i.e. weight 0.9, after 1s.
        assert_eq!(
            l.try_acquire(),
            Decision::Denied {
                retry_after: Duration::from_secs(1)
            }
        );
        clock.advance(Duration::from_secs(1));
        assert!(l.try_acquire().is_allowed());
    }

    #[test]
    fn idle_windows_clear_history() {
        let clock = ManualClock::new();
        let mut l = SlidingCounter::new(clock.clone(), 3, Duration::from_secs(10));
        l.try_acquire_n(3);
        clock.advance(Duration::from_secs(30));
        assert!(l.try_acquire_n(3).is_allowed());
    }
}
