//! Minimal runnable: SIFS vs simple threshold + benchmark; API mode: --input JSON --steps N → readout JSON.

use sifs_genesis_core::{
    BrainConfig, BrainReadout, BrainRunner,
    calculate_sifs_current, fixed_to_f32, f32_to_fixed, run_benchmark, sifs_threshold, sifs_w,
    SIFS_LEVELS,
};
use std::env;

fn sifs_vs_simple_test() {
    println!("\nSIFS vs Simple Threshold");
    println!("{}", "=".repeat(40));

    let test_voltages = [
        f32_to_fixed(0.005),
        f32_to_fixed(0.010),
        f32_to_fixed(0.015),
        f32_to_fixed(0.020),
        f32_to_fixed(0.025),
        f32_to_fixed(0.030),
    ];

    let mut sifs_weights = [0; SIFS_LEVELS];
    for n in 0..SIFS_LEVELS {
        sifs_weights[n] = sifs_w(n);
    }
    let mut sifs_thresholds = [0; SIFS_LEVELS];
    for n in 0..SIFS_LEVELS {
        sifs_thresholds[n] = sifs_threshold(n, f32_to_fixed(0.02));
    }

    println!("Input V (mV) | Simple | SIFS | Ratio");
    println!("{}", "-".repeat(40));

    for &v in &test_voltages {
        let simple_current = if v > f32_to_fixed(0.02) {
            f32_to_fixed(1.0)
        } else {
            0
        };
        let sifs_current =
            calculate_sifs_current(v, &sifs_weights, &sifs_thresholds, f32_to_fixed(40.0));
        let ratio = if simple_current > 0 {
            fixed_to_f32(sifs_current) / fixed_to_f32(simple_current)
        } else if sifs_current > 0 {
            999.9
        } else {
            1.0
        };
        println!(
            "{:8.1} | {:6.3} | {:6.3} | {:5.2}",
            fixed_to_f32(v) * 1000.0,
            fixed_to_f32(simple_current),
            fixed_to_f32(sifs_current),
            ratio
        );
    }
}

fn compare_architectures() {
    println!("\nArchitecture Comparison");
    println!("{}", "=".repeat(60));

    for &size in &[1000_usize, 5000, 10000] {
        println!("\nTesting {} neurons:", size);
        let metrics = run_benchmark(size, 5000, 500);
        println!("  Day phase: {:.3} ms/step", metrics.day_phase_ms);
        println!("  Night phase: {:.3} ms/cycle", metrics.night_phase_ms);
        println!("  Memory: {:.2} MB", metrics.memory_mb);
        println!("  Spikes/sec: {:.0}", metrics.spikes_per_second);
        println!("  Shard distribution: {:?}", metrics.neurons_per_shard);
    }
}

fn run_api_mode(input_json: &str, steps: u32, n_neurons: usize) -> Result<(), Box<dyn std::error::Error>> {
    let input: Vec<f32> = serde_json::from_str(input_json)?;
    let config = BrainConfig {
        n_neurons,
        ..BrainConfig::default()
    };
    let mut runner = BrainRunner::new(config);
    let mut last_readout = BrainReadout {
        spike_count_total: 0,
        spike_count_per_shard: vec![],
        score: 0.0,
        time_step: 0,
    };
    for _ in 0..steps {
        last_readout = runner.run_step(&input);
    }
    println!("{}", serde_json::to_string(&last_readout)?);
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    let mut input_json: Option<String> = None;
    let mut steps: u32 = 1;
    let mut n_neurons: usize = 1000;
    while i < args.len() {
        if args[i] == "--input" && i + 1 < args.len() {
            input_json = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        if args[i] == "--steps" && i + 1 < args.len() {
            steps = args[i + 1].parse().unwrap_or(1);
            i += 2;
            continue;
        }
        if args[i] == "--neurons" && i + 1 < args.len() {
            n_neurons = args[i + 1].parse().unwrap_or(1000);
            i += 2;
            continue;
        }
        i += 1;
    }

    if let Some(ref json) = input_json {
        if let Err(e) = run_api_mode(json, steps, n_neurons) {
            eprintln!("brain api error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    println!("SIFS-Genesis Core (minimal runnable)");
    println!("Constants synced with core.py / BRAIN_CONTRACT\n");

    sifs_vs_simple_test();
    compare_architectures();

    println!("\nDone.");
}
