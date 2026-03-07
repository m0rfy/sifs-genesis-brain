//! SIFS-Genesis Agent: observation → brain → action.
//! Reads JSON observation lines from stdin, outputs JSON { readout, action } per line.
//! CartPole: action 0 = left, 1 = right (from brain score).

use sifs_genesis_core::{BrainConfig, BrainReadout, BrainRunner};
use std::io::{self, BufRead, Write};

fn observation_to_action(readout: &BrainReadout) -> u32 {
    // CartPole: binary action. score < 0.5 → left (0), else right (1).
    if readout.score < 0.5 {
        0
    } else {
        1
    }
}

fn run_agent_loop(n_neurons: usize, steps_per_obs: u32) -> Result<(), Box<dyn std::error::Error>> {
    let config = BrainConfig {
        n_neurons,
        ..BrainConfig::default()
    };
    let mut runner = BrainRunner::new(config);
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut lines = stdin.lock().lines();

    while let Some(Ok(line)) = lines.next() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let obs: Vec<f32> = serde_json::from_str(line)?;
        let mut last = BrainReadout {
            spike_count_total: 0,
            spike_count_per_shard: vec![],
            score: 0.0,
            time_step: 0,
        };
        for _ in 0..steps_per_obs.max(1) {
            last = runner.run_step(&obs);
        }
        let action = observation_to_action(&last);
        let out = serde_json::json!({
            "readout": {
                "spike_count_total": last.spike_count_total,
                "score": last.score,
                "time_step": last.time_step
            },
            "action": action
        });
        writeln!(stdout, "{}", out)?;
        stdout.flush()?;
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut n_neurons: usize = 1000;
    let mut steps_per_obs: u32 = 1;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--neurons" && i + 1 < args.len() {
            n_neurons = args[i + 1].parse().unwrap_or(1000);
            i += 2;
            continue;
        }
        if args[i] == "--steps" && i + 1 < args.len() {
            steps_per_obs = args[i + 1].parse().unwrap_or(1);
            i += 2;
            continue;
        }
        i += 1;
    }

    if let Err(e) = run_agent_loop(n_neurons, steps_per_obs) {
        eprintln!("agent error: {}", e);
        std::process::exit(1);
    }
}
