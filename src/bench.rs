//! Benchmarks and comparison tests

use crate::constants::SIFS_LEVELS;
use crate::fixed::{f32_to_fixed, fixed_to_f32, sifs_threshold, sifs_w, Fixed16};
use crate::neuron::{calculate_sifs_current, NeuronArrays, SIFSShard};
use crate::phase::{day_phase_step, night_phase_plasticity};

#[derive(Debug)]
#[allow(dead_code)] // sifs_efficiency — для будущего анализа
pub struct PerformanceMetrics {
    pub day_phase_ms: f64,
    pub night_phase_ms: f64,
    pub spikes_per_second: f64,
    pub neurons_per_shard: [usize; SIFS_LEVELS],
    pub sifs_efficiency: f64,
    pub memory_mb: f64,
}

pub fn run_benchmark(
    n_neurons: usize,
    n_steps: usize,
    night_interval: usize,
) -> PerformanceMetrics {
    use std::time::Instant;

    println!(
        "🧠 SIFS-Genesis Hybrid Benchmark\nNeurons: {}, Steps: {}, Night every: {}",
        n_neurons, n_steps, night_interval
    );

    let mut neurons = NeuronArrays::new(n_neurons);
    let mut shards: Vec<SIFSShard> = (0..SIFS_LEVELS).map(|i| SIFSShard::new(i)).collect();
    for i in 0..n_neurons {
        let s_level = (i * SIFS_LEVELS / n_neurons).min(SIFS_LEVELS - 1);
        shards[s_level].add_neuron(i);
    }

    let mut spike_history = Vec::new();
    let mut total_spikes = 0;
    let mut day_time = 0.0;
    let mut night_time = 0.0;

    let input_pattern: Vec<Fixed16> = (0..n_neurons)
        .map(|i| f32_to_fixed(0.01 * (1.0 + (i as f32 * 0.1).sin())))
        .collect();

    for step in 0..n_steps {
        let day_start = Instant::now();
        let spikes = day_phase_step(&mut neurons, &shards, step as u32, &input_pattern, true);
        day_time += day_start.elapsed().as_secs_f64();

        total_spikes += spikes.len();
        if step % 1000 == 0 {
            let rate = total_spikes as f64 / (step + 1) as f64;
            println!(
                "Step {}: {} spikes/step, {} Hz avg",
                step,
                spikes.len(),
                rate * 1000.0
            );
        }

        spike_history.push(spikes);
        if spike_history.len() > night_interval {
            spike_history.remove(0);
        }

        if step % night_interval == 0 && step > 0 {
            let night_start = Instant::now();
            night_phase_plasticity(&mut neurons, &mut shards, &spike_history, 0.0);
            night_time += night_start.elapsed().as_secs_f64();
        }
    }

    let mut neurons_per_shard = [0; SIFS_LEVELS];
    for (i, shard) in shards.iter().enumerate() {
        neurons_per_shard[i] = shard.neurons.len();
    }

    let memory_mb = (neurons.voltage.len() * 4
        + neurons.sifs_weights.len() * SIFS_LEVELS * 4
        + neurons.sifs_thresholds.len() * SIFS_LEVELS * 4) as f64
        / 1_048_576.0;

    PerformanceMetrics {
        day_phase_ms: day_time * 1000.0 / n_steps as f64,
        night_phase_ms: night_time * 1000.0 / (n_steps / night_interval) as f64,
        spikes_per_second: total_spikes as f64 / (n_steps as f64 / 1000.0),
        neurons_per_shard,
        sifs_efficiency: 1.0,
        memory_mb,
    }
}

pub fn compare_architectures() {
    println!("\n🔬 Architecture Comparison\n{}", "=".repeat(60));
    let test_sizes = vec![1000, 10000, 100_000];
    for &size in &test_sizes {
        println!("\n📊 Testing {} neurons:", size);
        let metrics = run_benchmark(size, 5000, 500);
        println!("  Day phase: {:.3} ms/step", metrics.day_phase_ms);
        println!("  Night phase: {:.3} ms/cycle", metrics.night_phase_ms);
        println!("  Memory: {:.2} MB", metrics.memory_mb);
        println!("  Spikes/sec: {:.0}", metrics.spikes_per_second);
        println!("  Shard distribution: {:?}", metrics.neurons_per_shard);
    }
}

pub fn sifs_vs_simple_test() {
    println!("\n🧮 SIFS vs Simple Threshold\n{}", "=".repeat(40));
    let test_voltages = vec![
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
    println!("Input V (mV) | Simple | SIFS | Ratio\n{}", "-".repeat(40));
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
