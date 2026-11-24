use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::{Clock, Decision};

/// A lock-free GCRA limiter. The only state is one atomic "theoretical arrival
/// time" in nanoseconds, updated with compare-and-swap, so it can be shared by
/// reference (or `Arc`) across threads without a mutex.
#[derive(Debug)]
pub struct AtomicGcra<C: Clock> {
    clock: C,
    interval_ns: u64,
    burst: u32,
    tat_ns: AtomicU64,
}

impl<C: Clock> AtomicGcra<C> {
    /// Sustains `rate_per_sec` requests per second and tolerates bursts of `burst`.
    pub fn new(clock: C, rate_per_sec: f64, burst: u32) -> Self {
        assert!(rate_per_sec > 0.0, "rate must be positive");
        assert!(burst > 0, "burst must be positive");
        let interval_ns = ((1e9 / rate_per_sec) as u64).max(1);
        let tat_ns = AtomicU64::new(clock.now().as_nanos() as u64);
        Self {
            clock,
            interval_ns,
            burst,
            tat_ns,
        }
    }

    pub fn try_acquire_n(&self, cost: u32) -> Decision {
        if cost > self.burst {
            return Decision::Denied {
                retry_after: Duration::MAX,
            };
        }
        let cost_ns = self.interval_ns.saturating_mul(cost as u64);
        let window_ns = self.interval_ns.saturating_mul(self.burst as u64);
        let mut tat = self.tat_ns.load(Ordering::Acquire);
        loop {
            let now = self.clock.now().as_nanos() as u64;
            let new_tat = tat.max(now).saturating_add(cost_ns);
            let allow_at = new_tat.saturating_sub(window_ns);
            if now < allow_at {
                return Decision::Denied {
                    retry_after: Duration::from_nanos(allow_at - now),
                };
            }
            match self.tat_ns.compare_exchange_weak(
                tat,
                new_tat,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Decision::Allowed,
                Err(actual) => tat = actual,
            }
        }
    }

    pub fn try_acquire(&self) -> Decision {
        self.try_acquire_n(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ManualClock;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn allows_burst_then_denies_with_retry_hint() {
        let g = AtomicGcra::new(ManualClock::new(), 10.0, 5);
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
        let g = AtomicGcra::new(clock.clone(), 10.0, 5);
        g.try_acquire_n(5);
        clock.advance(Duration::from_millis(100));
        assert!(g.try_acquire().is_allowed());
        assert!(!g.try_acquire().is_allowed());
    }

    #[test]
    fn oversized_cost_is_never_allowed() {
        let g = AtomicGcra::new(ManualClock::new(), 10.0, 5);
        assert!(!g.try_acquire_n(6).is_allowed());
        assert!(g.try_acquire_n(5).is_allowed());
    }

    #[test]
    fn never_admits_more_than_burst_across_threads() {
        let g = Arc::new(AtomicGcra::new(ManualClock::new(), 1.0, 100));
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let g = Arc::clone(&g);
                thread::spawn(move || (0..50).filter(|_| g.try_acquire().is_allowed()).count())
            })
            .collect();
        let allowed: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
        assert_eq!(allowed, 100);
    }
}
