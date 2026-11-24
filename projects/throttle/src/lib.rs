//! Rate-limiting algorithms behind a single [`Limiter`] trait.
//!
//! Every limiter takes a [`Clock`], so behaviour can be tested without sleeping.

mod atomic_gcra;
pub mod clock;
mod fixed_window;
mod gcra;
mod keyed;
mod shared;
mod sliding_counter;
mod sliding_log;
mod token_bucket;

use std::time::Duration;

pub use atomic_gcra::AtomicGcra;
pub use clock::{Clock, ManualClock, SystemClock};
pub use fixed_window::FixedWindow;
pub use gcra::Gcra;
pub use keyed::Keyed;
pub use shared::Shared;
pub use sliding_counter::SlidingCounter;
pub use sliding_log::SlidingLog;
pub use token_bucket::TokenBucket;

/// The outcome of asking a limiter for permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allowed,
    /// Denied; the same request may succeed after `retry_after`.
    Denied {
        retry_after: Duration,
    },
}

impl Decision {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Decision::Allowed)
    }
}

/// A rate limiter that admits or rejects units of work.
pub trait Limiter {
    /// Try to take `cost` units. A denied request consumes nothing.
    fn try_acquire_n(&mut self, cost: u32) -> Decision;

    /// Try to take a single unit.
    fn try_acquire(&mut self) -> Decision {
        self.try_acquire_n(1)
    }
}
