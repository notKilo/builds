//! Deterministic traffic simulation used by the demo CLI.

use std::time::Duration;

use crate::{FixedWindow, Gcra, Limiter, ManualClock, SlidingCounter, SlidingLog, TokenBucket};

/// Names accepted by [`build`].
pub const STRATEGIES: [&str; 5] = [
    "token-bucket",
    "fixed-window",
    "sliding-log",
    "sliding-counter",
    "gcra",
];

/// Builds a limiter by name. `rate` is the sustained requests per second and
/// `burst` the most that may be admitted at once. Window strategies get a limit
/// of `burst` per window of `burst / rate` seconds so all five are comparable.
pub fn build(
    name: &str,
    clock: ManualClock,
    rate: f64,
    burst: u32,
) -> Result<Box<dyn Limiter>, String> {
    if rate <= 0.0 || burst == 0 {
        return Err("rate and burst must be positive".into());
    }
    let window = Duration::from_secs_f64(burst as f64 / rate);
    Ok(match name {
        "token-bucket" => Box::new(TokenBucket::new(clock, burst, rate)),
        "fixed-window" => Box::new(FixedWindow::new(clock, burst, window)),
        "sliding-log" => Box::new(SlidingLog::new(clock, burst, window)),
        "sliding-counter" => Box::new(SlidingCounter::new(clock, burst, window)),
        "gcra" => Box::new(Gcra::new(clock, rate, burst)),
        other => return Err(format!("unknown strategy {other:?}")),
    })
}

/// Sends `traffic_rps` evenly spaced requests per second for `seconds` seconds and
/// returns, for each simulated second, whether each request was admitted.
pub fn simulate(
    limiter: &mut dyn Limiter,
    clock: &ManualClock,
    traffic_rps: u32,
    seconds: u32,
) -> Vec<Vec<bool>> {
    let step = Duration::from_secs(1) / traffic_rps;
    (0..seconds)
        .map(|_| {
            (0..traffic_rps)
                .map(|_| {
                    let allowed = limiter.try_acquire().is_allowed();
                    clock.advance(step);
                    allowed
                })
                .collect()
        })
        .collect()
}

/// One line per second: `#` for an allowed request, `.` for a denied one.
pub fn render(timeline: &[Vec<bool>]) -> String {
    let mut out = String::new();
    for (sec, reqs) in timeline.iter().enumerate() {
        let allowed = reqs.iter().filter(|a| **a).count();
        let marks: String = reqs.iter().map(|a| if *a { '#' } else { '.' }).collect();
        out.push_str(&format!(
            "t={sec:>3}s  {allowed:>3}/{:<3} {marks}\n",
            reqs.len()
        ));
    }
    out
}

/// Total admitted requests.
pub fn total_allowed(timeline: &[Vec<bool>]) -> usize {
    timeline.iter().flatten().filter(|a| **a).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(name: &str, rate: f64, burst: u32, rps: u32, secs: u32) -> Vec<Vec<bool>> {
        let clock = ManualClock::new();
        let mut l = build(name, clock.clone(), rate, burst).unwrap();
        simulate(l.as_mut(), &clock, rps, secs)
    }

    #[test]
    fn every_strategy_roughly_holds_the_sustained_rate() {
        for name in STRATEGIES {
            let t = run(name, 5.0, 5, 20, 10);
            let allowed = total_allowed(&t);
            // 10s at 5/s plus one burst of slack; the sliding counter errs low.
            assert!((35..=60).contains(&allowed), "{name} admitted {allowed}");
        }
    }

    #[test]
    fn traffic_below_the_limit_is_never_denied() {
        for name in STRATEGIES {
            let t = run(name, 10.0, 10, 4, 5);
            assert_eq!(total_allowed(&t), 20, "{name}");
        }
    }

    #[test]
    fn rejects_bad_input() {
        let c = ManualClock::new();
        assert!(build("nope", c.clone(), 1.0, 1).is_err());
        assert!(build("gcra", c, 0.0, 1).is_err());
    }

    #[test]
    fn render_marks_allowed_and_denied() {
        let out = render(&[vec![true, false, true]]);
        assert_eq!(out, "t=  0s    2/3   #.#\n");
    }
}
