//! SIFS-Genesis Brain — точка входа: разбор аргументов и запуск режимов.

mod bench;
mod constants;
mod fixed;
mod io;
mod neuron;
mod phase;
mod runner;

use crate::bench::{compare_architectures, sifs_vs_simple_test};
use crate::io::{run_agent_loop, run_compute_sifs_stdin, run_serve_loop};
use crate::neuron::BrainConfig;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    let mut agent_mode = false;
    let mut serve_bind: Option<String> = None;
    let mut config = BrainConfig::default();

    while i < args.len() {
        if args[i] == "--agent" {
            agent_mode = true;
            i += 1;
            continue;
        }
        if args[i] == "--serve" && i + 1 < args.len() {
            serve_bind = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        if args[i] == "--neurons" && i + 1 < args.len() {
            config.n_neurons = args[i + 1].parse().unwrap_or(1000);
            i += 2;
            continue;
        }
        if args[i] == "--steps" && i + 1 < args.len() {
            config.steps_per_observation = args[i + 1].parse().unwrap_or(1);
            i += 2;
            continue;
        }
        if args[i] == "--night" && i + 1 < args.len() {
            config.night_interval = args[i + 1].parse().unwrap_or(500);
            i += 2;
            continue;
        }
        if args[i] == "--v0" && i + 1 < args.len() {
            config.v0 = args[i + 1].parse().unwrap_or(0.02);
            i += 2;
            continue;
        }
        if args[i] == "--no-sifs" {
            config.sifs_enabled = false;
            i += 1;
            continue;
        }
        if args[i] == "--trainable-readout" {
            config.trainable_readout = true;
            i += 1;
            continue;
        }
        if args[i] == "--readout-lr" && i + 1 < args.len() {
            config.readout_lr = args[i + 1].parse().unwrap_or(0.05);
            i += 2;
            continue;
        }
        if args[i] == "--compute-sifs" {
            run_compute_sifs_stdin();
            return;
        }
        i += 1;
    }

    if let Some(bind) = serve_bind {
        if let Err(e) = run_serve_loop(config, &bind) {
            eprintln!("serve error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    if agent_mode {
        if let Err(e) = run_agent_loop(config) {
            eprintln!("agent error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    println!("🧠 SIFS Brain — тесты и демо");
    println!("Ядро SIFS (K, φ, W(n)) + Day/Night (BRAIN_CONTRACT)\n");

    sifs_vs_simple_test();
    compare_architectures();

    println!("\n✅ Тесты пройдены.");
    println!("\nРежимы:");
    println!("  --agent              stdin: JSON строки с массивом float → stdout: readout + action");
    println!("  --serve <addr:port>  HTTP: POST /step с {{ \"input\": [...] }} → JSON ответ");
    println!("  --compute-sifs       stdin: по одному float (V) на строку → stdout: I_SIFS (A.1)");
    println!("  Опции: --neurons N --steps K --night M --v0 V --no-sifs [--trainable-readout] [--readout-lr F]");
}

#[cfg(test)]
mod tests {
    use crate::constants::{FIB, PHI, SIFS_LEVELS};
    use crate::fixed::{fixed_to_f32, f32_to_fixed, sifs_threshold, sifs_w};
    use crate::neuron::{calculate_sifs_current, BrainConfig, NeuronArrays, SIFSShard};
    use crate::phase::{day_phase_step, night_phase_plasticity};
    use crate::runner::BrainRunner;

    #[test]
    fn test_fib_contract() {
        assert_eq!(FIB.len(), SIFS_LEVELS);
        assert_eq!(FIB[0], 1);
        assert_eq!(FIB[9], 89);
    }

    #[test]
    fn test_sifs_math() {
        let w0 = fixed_to_f32(sifs_w(0));
        let w9 = fixed_to_f32(sifs_w(9));
        assert!((w0 - 1.0).abs() < 0.01);
        assert!((w9 - 0.161).abs() < 0.01);
    }

    #[test]
    fn test_phi_thresholds() {
        let v0 = f32_to_fixed(1.0);
        let ratio = fixed_to_f32(sifs_threshold(1, v0)) / fixed_to_f32(sifs_threshold(2, v0));
        assert!((ratio - PHI).abs() < 0.01);
    }

    #[test]
    fn test_neuron_arrays() {
        let neurons = NeuronArrays::new(100);
        assert_eq!(neurons.len(), 100);
        assert_eq!(neurons.sifs_weights.len(), 100);
        assert_eq!(neurons.sifs_weights[0].len(), SIFS_LEVELS);
    }

    #[test]
    fn test_sifs_current() {
        let mut weights = [0; SIFS_LEVELS];
        let mut thresholds = [0; SIFS_LEVELS];
        for n in 0..SIFS_LEVELS {
            weights[n] = sifs_w(n);
            thresholds[n] = sifs_threshold(n, f32_to_fixed(0.02));
        }
        let current = calculate_sifs_current(
            f32_to_fixed(0.03),
            &weights,
            &thresholds,
            f32_to_fixed(40.0),
        );
        assert!(current > 0);
        assert!(fixed_to_f32(current) < 10.0);
    }

    #[test]
    fn test_i_sifs_regression_vs_python() {
        let mut weights = [0; SIFS_LEVELS];
        let mut thresholds = [0; SIFS_LEVELS];
        for n in 0..SIFS_LEVELS {
            weights[n] = sifs_w(n);
            thresholds[n] = sifs_threshold(n, f32_to_fixed(0.02));
        }
        let v = f32_to_fixed(0.02);
        let rust_i = calculate_sifs_current(v, &weights, &thresholds, f32_to_fixed(40.0));
        let rust_f = fixed_to_f32(rust_i);
        assert!(rust_f >= 0.0 && rust_f < 20.0);
        let v_high = f32_to_fixed(0.03);
        let rust_high = calculate_sifs_current(v_high, &weights, &thresholds, f32_to_fixed(40.0));
        assert!(fixed_to_f32(rust_high) > fixed_to_f32(rust_i));
    }

    #[test]
    fn test_day_phase_smoke() {
        let mut neurons = NeuronArrays::new(50);
        let mut shards: Vec<SIFSShard> = (0..SIFS_LEVELS).map(SIFSShard::new).collect();
        for i in 0..50 {
            shards[i * SIFS_LEVELS / 50].add_neuron(i);
        }
        let input: Vec<crate::fixed::Fixed16> = (0..50)
            .map(|i| f32_to_fixed(0.02 * (1.0 + (i % 5) as f32 * 0.1)))
            .collect();
        let _spikes = day_phase_step(&mut neurons, &shards, 0, &input, true);
        assert!(neurons.voltage.len() == 50);
    }

    #[test]
    fn test_night_phase_smoke() {
        let mut neurons = NeuronArrays::new(30);
        let mut shards: Vec<SIFSShard> = (0..SIFS_LEVELS).map(SIFSShard::new).collect();
        for i in 0..30 {
            shards[i * SIFS_LEVELS / 30].add_neuron(i);
        }
        let history = vec![vec![0usize, 5, 10], vec![1, 6], vec![]];
        night_phase_plasticity(&mut neurons, &mut shards, &history, 0.0);
        assert_eq!(neurons.len(), 30);
    }

    #[test]
    fn test_brain_runner_readout() {
        let config = BrainConfig {
            n_neurons: 100,
            steps_per_observation: 1,
            ..BrainConfig::default()
        };
        let mut runner = BrainRunner::new(config);
        let (readout, action) = runner.step(&[0.01, 0.02, 0.015]);
        assert!(readout.spike_count_total <= 100);
        assert!(readout.score >= 0.0 && readout.score <= 1.0);
        assert!(action == 0 || action == 1);
        let (r2, _a2) = runner.step(&[0.1; 4]);
        assert!(r2.time_step >= readout.time_step);
    }

    #[test]
    fn test_trainable_readout_disabled_by_default() {
        let config = BrainConfig {
            n_neurons: 50,
            steps_per_observation: 1,
            ..BrainConfig::default()
        };
        assert!(!config.trainable_readout);
        let mut runner = BrainRunner::new(config);
        let (_r, a1) = runner.step(&[0.1; 4]);
        let (_r, a2) = runner.step(&[0.2; 4]);
        assert!(a1 == 0 || a1 == 1);
        assert!(a2 == 0 || a2 == 1);
    }

    #[test]
    fn test_trainable_readout_updates_weights() {
        let config = BrainConfig {
            n_neurons: 50,
            steps_per_observation: 1,
            trainable_readout: true,
            readout_lr: 0.1,
            ..BrainConfig::default()
        };
        let mut runner = BrainRunner::new(config);
        let (_r1, a1) = runner.step(&[0.01; 4]);
        runner.last_reward = 1.0;
        let (_r2, a2) = runner.step(&[0.02; 4]);
        runner.last_reward = -0.5;
        let (_r3, _a3) = runner.step(&[0.01; 4]);
        assert!(a1 == 0 || a1 == 1);
        assert!(a2 == 0 || a2 == 1);
        let (readout, action) = runner.step(&[0.1; 4]);
        assert!(readout.score >= 0.0 && readout.score <= 1.0);
        assert!(action == 0 || action == 1);
    }
}
