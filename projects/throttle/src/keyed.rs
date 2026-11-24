use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;

use crate::{Clock, Decision, Limiter};

const SWEEP_EVERY: u32 = 1024;

/// One independent limiter per key (client id, API token, IP address...).
///
/// Limiters are created on first use by `factory`. Keys that have not been seen
/// for `idle_ttl` are dropped, either by calling [`Keyed::evict_idle`] or
/// automatically every 1024 calls, so the map cannot grow without bound.
pub struct Keyed<K, L, C, F> {
    clock: C,
    factory: F,
    idle_ttl: Duration,
    entries: HashMap<K, (L, Duration)>,
    since_sweep: u32,
}

impl<K, L, C, F> Keyed<K, L, C, F>
where
    K: Hash + Eq,
    L: Limiter,
    C: Clock,
    F: FnMut() -> L,
{
    pub fn new(clock: C, idle_ttl: Duration, factory: F) -> Self {
        Self {
            clock,
            factory,
            idle_ttl,
            entries: HashMap::new(),
            since_sweep: 0,
        }
    }

    pub fn try_acquire_n(&mut self, key: K, cost: u32) -> Decision {
        self.since_sweep += 1;
        if self.since_sweep >= SWEEP_EVERY {
            self.evict_idle();
        }
        let now = self.clock.now();
        let factory = &mut self.factory;
        let (limiter, last_seen) = self.entries.entry(key).or_insert_with(|| (factory(), now));
        *last_seen = now;
        limiter.try_acquire_n(cost)
    }

    pub fn try_acquire(&mut self, key: K) -> Decision {
        self.try_acquire_n(key, 1)
    }

    /// Drops limiters not used within the idle TTL. Returns how many were removed.
    pub fn evict_idle(&mut self) -> usize {
        self.since_sweep = 0;
        let now = self.clock.now();
        let ttl = self.idle_ttl;
        let before = self.entries.len();
        self.entries
            .retain(|_, (_, seen)| now.saturating_sub(*seen) < ttl);
        before - self.entries.len()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ManualClock, TokenBucket};

    fn keyed(
        clock: &ManualClock,
    ) -> Keyed<
        &'static str,
        TokenBucket<ManualClock>,
        ManualClock,
        impl FnMut() -> TokenBucket<ManualClock>,
    > {
        let c = clock.clone();
        Keyed::new(clock.clone(), Duration::from_secs(60), move || {
            TokenBucket::new(c.clone(), 2, 1.0)
        })
    }

    #[test]
    fn keys_are_limited_independently() {
        let clock = ManualClock::new();
        let mut k = keyed(&clock);
        assert!(k.try_acquire_n("a", 2).is_allowed());
        assert!(!k.try_acquire("a").is_allowed());
        assert!(k.try_acquire("b").is_allowed());
    }

    #[test]
    fn idle_keys_are_evicted() {
        let clock = ManualClock::new();
        let mut k = keyed(&clock);
        k.try_acquire("a");
        clock.advance(Duration::from_secs(30));
        k.try_acquire("b");
        clock.advance(Duration::from_secs(40)); // a idle 70s, b idle 40s
        assert_eq!(k.evict_idle(), 1);
        assert_eq!(k.len(), 1);
    }

    #[test]
    fn evicted_key_starts_fresh() {
        let clock = ManualClock::new();
        let mut k = keyed(&clock);
        k.try_acquire_n("a", 2);
        clock.advance(Duration::from_secs(120));
        k.evict_idle();
        assert!(k.try_acquire_n("a", 2).is_allowed());
    }

    #[test]
    fn sweeps_automatically_under_load() {
        let clock = ManualClock::new();
        let mut k = keyed(&clock);
        k.try_acquire("old");
        clock.advance(Duration::from_secs(120));
        for _ in 0..SWEEP_EVERY {
            k.try_acquire("busy");
        }
        assert_eq!(k.len(), 1);
    }
}
