use std::sync::{Arc, Mutex};

use crate::{Decision, Limiter};

/// A cloneable, thread-safe handle to any [`Limiter`], guarded by a mutex.
///
/// All clones share the same limiter state, so one budget is enforced across threads.
#[derive(Debug)]
pub struct Shared<L> {
    inner: Arc<Mutex<L>>,
}

impl<L> Clone for Shared<L> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<L: Limiter> Shared<L> {
    pub fn new(limiter: L) -> Self {
        Self {
            inner: Arc::new(Mutex::new(limiter)),
        }
    }

    pub fn try_acquire_n(&self, cost: u32) -> Decision {
        // A panic while holding the lock cannot leave a limiter half-updated in a
        // way that matters more than refusing service, so recover from poisoning.
        let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        guard.try_acquire_n(cost)
    }

    pub fn try_acquire(&self) -> Decision {
        self.try_acquire_n(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ManualClock, TokenBucket};
    use std::thread;

    #[test]
    fn clones_share_one_budget() {
        let clock = ManualClock::new();
        let a = Shared::new(TokenBucket::new(clock, 3, 1.0));
        let b = a.clone();
        assert!(a.try_acquire().is_allowed());
        assert!(b.try_acquire().is_allowed());
        assert!(a.try_acquire().is_allowed());
        assert!(!b.try_acquire().is_allowed());
    }

    #[test]
    fn never_admits_more_than_burst_across_threads() {
        let clock = ManualClock::new();
        let limiter = Shared::new(TokenBucket::new(clock, 100, 1.0));
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let l = limiter.clone();
                thread::spawn(move || (0..50).filter(|_| l.try_acquire().is_allowed()).count())
            })
            .collect();
        let allowed: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
        assert_eq!(allowed, 100);
    }
}
