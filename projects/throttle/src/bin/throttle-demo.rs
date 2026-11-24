use std::process::ExitCode;

use throttle::sim::{self, STRATEGIES};
use throttle::ManualClock;

const USAGE: &str = "\
usage: throttle-demo [--strategy NAME|all] [--rate N] [--burst N] [--traffic N] [--seconds N]

Simulates steady traffic against a limiter and prints which requests were admitted.
  --strategy  token-bucket | fixed-window | sliding-log | sliding-counter | gcra | all (default all)
  --rate      sustained requests per second the limiter allows (default 5)
  --burst     largest burst admitted at once (default 5)
  --traffic   requests sent per second (default 20)
  --seconds   simulated duration (default 5)";

struct Args {
    strategy: String,
    rate: f64,
    burst: u32,
    traffic: u32,
    seconds: u32,
}

fn parse(args: &[String]) -> Result<Args, String> {
    let mut a = Args {
        strategy: "all".into(),
        rate: 5.0,
        burst: 5,
        traffic: 20,
        seconds: 5,
    };
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        let mut value = || it.next().ok_or(format!("{flag} needs a value"));
        let bad = |f: &str| format!("invalid value for {f}");
        match flag.as_str() {
            "--strategy" => a.strategy = value()?.clone(),
            "--rate" => a.rate = value()?.parse().map_err(|_| bad(flag))?,
            "--burst" => a.burst = value()?.parse().map_err(|_| bad(flag))?,
            "--traffic" => a.traffic = value()?.parse().map_err(|_| bad(flag))?,
            "--seconds" => a.seconds = value()?.parse().map_err(|_| bad(flag))?,
            other => return Err(format!("unknown flag {other}")),
        }
    }
    if a.traffic == 0 || a.seconds == 0 {
        return Err("--traffic and --seconds must be positive".into());
    }
    Ok(a)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let a = match parse(&args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let names: Vec<&str> = if a.strategy == "all" {
        STRATEGIES.to_vec()
    } else {
        vec![a.strategy.as_str()]
    };
    for name in names {
        let clock = ManualClock::new();
        let mut limiter = match sim::build(name, clock.clone(), a.rate, a.burst) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::from(2);
            }
        };
        let timeline = sim::simulate(limiter.as_mut(), &clock, a.traffic, a.seconds);
        println!("== {name} (rate {}/s, burst {}) ==", a.rate, a.burst);
        print!("{}", sim::render(&timeline));
        println!(
            "admitted {} of {}\n",
            sim::total_allowed(&timeline),
            a.traffic * a.seconds
        );
    }
    ExitCode::SUCCESS
}
