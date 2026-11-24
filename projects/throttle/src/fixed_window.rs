use std::time::Duration;

use crate::{Clock, Decision, Limiter};

/// Allows `limit` units per aligned window. Cheap, but permits up to twice the
/// limit across a window boundary.
#[derive(Debug)]
pub struct FixedWindow<C: Clock> {
    clock: C,
    limit: u32,
    window: Duration,
    current: u128,
    used: u32,
}

impl<C: Clock> FixedWindow<C> {
    pub fn new(clock: C, limit: u32, window: Duration) -> Self {
        assert!(limit > 0, "limit must be positive");
        assert!(!window.is_zero(), "window must be non-zero");
        Self { clock, limit, window, current: 0, used: 0 }
    }
}

impl<C: Clock> Limiter for FixedWindow<C> {
    fn try_acquire_n(&mut self, cost: u32) -> Decision {
        if cost > self.limit {
            return Decision::Denied { retry_after: Duration::MAX };
        }
        let now = self.clock.now();
        let index = now.as_nanos() / self.window.as_nanos();
        if index != self.current {
            self.current = index;
            self.used = 0;
        }
        if self.used + cost <= self.limit {
            self.used += cost;
            Decision::Allowed
        } else {
            let window_end = (index + 1) * self.window.as_nanos();
            let retry = Duration::from_nanos((window_end - now.as_nanos()) as u64);
            Decision::Denied { retry_after: retry }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ManualClock;

    #[test]
    fn enforces_limit_within_a_window() {
        let clock = ManualClock::new();
        let mut w = FixedWindow::new(clock.clone(), 3, Duration::from_secs(10));
        assert!(w.try_acquire_n(3).is_allowed());
        assert_eq!(w.try_acquire(), Decision::Denied { retry_after: Duration::from_secs(10) });
    }

    #[test]
    fn resets_at_the_window_boundary() {
        let clock = ManualClock::new();
        let mut w = FixedWindow::new(clock.clone(), 2, Duration::from_secs(10));
        clock.advance(Duration::from_secs(9));
        assert!(w.try_acquire_n(2).is_allowed());
        assert_eq!(w.try_acquire(), Decision::Denied { retry_after: Duration::from_secs(1) });
        clock.advance(Duration::from_secs(1));
        assert!(w.try_acquire().is_allowed());
    }

    #[test]
    fn boundary_burst_allows_double_the_limit() {
        // The known weakness of fixed windows, documented as a test.
        let clock = ManualClock::new();
        let mut w = FixedWindow::new(clock.clone(), 5, Duration::from_secs(10));
        clock.advance(Duration::from_millis(9_999));
        assert!(w.try_acquire_n(5).is_allowed());
        clock.advance(Duration::from_millis(1));
        assert!(w.try_acquire_n(5).is_allowed());
    }
}
