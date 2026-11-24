//! Time sources. Limiters read time through [`Clock`] so tests can drive it by hand.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// A monotonic time source. `now` is the time elapsed since an arbitrary, fixed origin.
pub trait Clock {
    fn now(&self) -> Duration;
}

/// Real time, measured from the moment the clock was created.
#[derive(Debug, Clone)]
pub struct SystemClock {
    origin: Instant,
}

impl SystemClock {
    pub fn new() -> Self {
        Self { origin: Instant::now() }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn now(&self) -> Duration {
        self.origin.elapsed()
    }
}

/// A clock that only moves when told to. Clones share the same time.
#[derive(Debug, Clone, Default)]
pub struct ManualClock {
    nanos: Arc<AtomicU64>,
}

impl ManualClock {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn advance(&self, by: Duration) {
        self.nanos.fetch_add(by.as_nanos() as u64, Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Duration {
        Duration::from_nanos(self.nanos.load(Ordering::SeqCst))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_clock_only_moves_when_advanced() {
        let c = ManualClock::new();
        assert_eq!(c.now(), Duration::ZERO);
        c.advance(Duration::from_millis(1500));
        assert_eq!(c.now(), Duration::from_millis(1500));
    }

    #[test]
    fn manual_clock_clones_share_time() {
        let a = ManualClock::new();
        let b = a.clone();
        a.advance(Duration::from_secs(2));
        assert_eq!(b.now(), Duration::from_secs(2));
    }

    #[test]
    fn system_clock_is_monotonic() {
        let c = SystemClock::new();
        let first = c.now();
        assert!(c.now() >= first);
    }
}
