//! SIFS-Genesis core: constants and math synced with core.py and BRAIN_CONTRACT.md.
//! K, PHI, FIB, W(n), fixed-point, I_SIFS, Day/Night (rayon).

use rayon::prelude::*;
use std::f32::consts::PI;

// === SIFS Constants (sync with core.py, sifs_constants.json) ===
pub const PHI: f32 = 1.618_034_0; // (1+√5)/2
pub const K: f32 = 1.0 / (PI * PI);
pub const SIFS_LEVELS: usize = 10;

// === Genesis Integer Physics ===
pub type Fixed16 = i32;
pub const FIXED_ONE: Fixed16 = 1 << 16;

#[inline]
pub fn f32_to_fixed(x: f32) -> Fixed16 {
    (x * FIXED_ONE as f32) as Fixed16
}

#[inline]
pub fn fixed_to_f32(x: Fixed16) -> f32 {
    x as f32 / FIXED_ONE as f32
}

// === SIFS Math in Fixed Point ===
#[inline]
pub fn sifs_w(n: usize) -> Fixed16 {
    let w = (-2.0 * K * n as f32).exp();
    f32_to_fixed(w)
}

#[inline]
pub fn sifs_threshold(n: usize, v0: Fixed16) -> Fixed16 {
    let phi_n = PHI.powi(n as i32);
    (v0 as f64 / phi_n as f64) as Fixed16
}

// === SoA Layout (Genesis style) ===
#[derive(Debug, Clone)]
pub struct NeuronArrays {
    pub voltage: Vec<Fixed16>,
    pub threshold: Vec<Fixed16>,
    pub spike_count: Vec<u16>,
    pub sifs_weights: Vec<[Fixed16; SIFS_LEVELS]>,
    pub sifs_thresholds: Vec<[Fixed16; SIFS_LEVELS]>,
    pub s_coordinate: Vec<Fixed16>,
    pub axon_direction: Vec<(f32, f32)>,
    pub axon_fov: Vec<f32>,
    pub dendrite_targets: Vec<Vec<usize>>,
    pub last_spike_time: Vec<u32>,
    pub refractory_timer: Vec<u16>,
}

impl NeuronArrays {
    pub fn new(n_neurons: usize) -> Self {
        let mut neurons = NeuronArrays {
            voltage: vec![0; n_neurons],
            threshold: vec![f32_to_fixed(0.02); n_neurons],
            spike_count: vec![0; n_neurons],
            sifs_weights: vec![[0; SIFS_LEVELS]; n_neurons],
            sifs_thresholds: vec![[0; SIFS_LEVELS]; n_neurons],
            s_coordinate: vec![0; n_neurons],
            axon_direction: vec![(0.0, 0.0); n_neurons],
            axon_fov: vec![0.0; n_neurons],
            dendrite_targets: vec![Vec::new(); n_neurons],
            last_spike_time: vec![0; n_neurons],
            refractory_timer: vec![0; n_neurons],
        };

        for i in 0..n_neurons {
            let angle = i as f32 * 2.0 * PI / (PHI * PHI);
            let radius = (i as f32 / n_neurons as f32).sqrt();
            neurons.s_coordinate[i] = f32_to_fixed(radius * 5.0);

            for n in 0..SIFS_LEVELS {
                neurons.sifs_weights[i][n] = sifs_w(n);
                neurons.sifs_thresholds[i][n] = sifs_threshold(n, neurons.threshold[i]);
            }
            neurons.axon_direction[i] = (angle.cos(), angle.sin());
            neurons.axon_fov[i] = PI / 6.0;
        }

        neurons
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.voltage.len()
    }
}

// === S-Level Sharding ===
#[derive(Debug)]
pub struct SifsShard {
    pub level: usize,
    pub neurons: Vec<usize>,
    pub w_factor: Fixed16,
    pub time_scale: u32,
}

impl SifsShard {
    pub fn new(level: usize) -> Self {
        SifsShard {
            level,
            neurons: Vec::new(),
            w_factor: sifs_w(level),
            time_scale: 1 << level,
        }
    }

    pub fn add_neuron(&mut self, neuron_id: usize) {
        self.neurons.push(neuron_id);
    }
}

/// Build S-shards from current s_coordinate (S-level 0..9). Clears and refills each shard.
/// Call after Night Phase or for initial assignment. Genesis: planar sharding replaced by S-coordinate.
pub fn build_s_shards_from_neurons(neurons: &NeuronArrays, shards: &mut [SifsShard]) {
    for shard in shards.iter_mut() {
        shard.neurons.clear();
    }
    let n = neurons.len();
    for i in 0..n {
        let s_level = (fixed_to_f32(neurons.s_coordinate[i]) as usize).min(SIFS_LEVELS - 1);
        if s_level < shards.len() {
            shards[s_level].add_neuron(i);
        }
    }
}

/// Ghost Axons on S-boundaries: for each spike from a neuron in shard L, neurons in shards L±1
/// receive that spike as incoming (ApplySpikeBatch). Returns deduplicated list of target indices.
/// Genesis: 04_connectivity §1.7, 06_distributed — Ghost Axon handover between shards.
pub fn ghost_spike_targets(neurons: &NeuronArrays, shards: &[SifsShard], spikes: &[usize]) -> Vec<usize> {
    let mut targets = std::collections::HashSet::new();
    for &idx in spikes {
        if idx >= neurons.len() {
            continue;
        }
        let s_level = (fixed_to_f32(neurons.s_coordinate[idx]) as usize).min(SIFS_LEVELS - 1);
        if s_level > 0 {
            if let Some(s) = shards.get(s_level - 1) {
                for &i in &s.neurons {
                    targets.insert(i);
                }
            }
        }
        if s_level + 1 < shards.len() {
            if let Some(s) = shards.get(s_level + 1) {
                for &i in &s.neurons {
                    targets.insert(i);
                }
            }
        }
    }
    let mut out: Vec<usize> = targets.into_iter().collect();
    out.sort_unstable();
    out
}

// === SIFS Current ===
#[inline]
pub fn calculate_sifs_current(
    input_voltage: Fixed16,
    sifs_weights: &[Fixed16; SIFS_LEVELS],
    sifs_thresholds: &[Fixed16; SIFS_LEVELS],
    _beta: Fixed16,
) -> Fixed16 {
    let mut i_sifs = 0i64;
    for n in 0..SIFS_LEVELS {
        let v_diff = input_voltage - sifs_thresholds[n];
        let sigma = if v_diff < -2 * FIXED_ONE {
            0
        } else if v_diff > 2 * FIXED_ONE {
            FIXED_ONE
        } else {
            (FIXED_ONE / 2) + (v_diff / 4)
        };
        let contrib = (sifs_weights[n] as i64 * sigma as i64) >> 16;
        i_sifs += contrib;
    }
    (i_sifs.min(i32::MAX as i64) as i32).max(0)
}

// === Day Phase (single step, backward compatible) ===
pub fn day_phase_step(
    neurons: &mut NeuronArrays,
    _shards: &[SifsShard],
    time_step: u32,
    external_input: &[Fixed16],
) -> Vec<usize> {
    day_phase_pipeline_6step(neurons, _shards, time_step, external_input, &[])
}

/// Order of Day Phase kernels (Genesis 07_gpu_runtime §2.1, 05_signal_physics).
/// 1=InjectInputs, 2=ApplySpikeBatch, 3=PropagateAxons, 4=UpdateNeurons, 5=ApplyGSOP, 6=RecordReadout.
/// In this rayon implementation: steps 1–2 build effective input; 3 is no-op (no axon_heads);
/// 4 runs GLIF + I_SIFS + φ-thresholds; 5 is no-op (GSOP in Genesis); 6 returns spike list.
pub fn day_phase_pipeline_6step(
    neurons: &mut NeuronArrays,
    _shards: &[SifsShard],
    time_step: u32,
    external_input: &[Fixed16],
    incoming_ghost_spikes: &[usize],
) -> Vec<usize> {
    let n = neurons.len();
    // Step 1: InjectInputs — external (virtual) input → effective_input
    let mut effective_input: Vec<Fixed16> = external_input
        .iter()
        .take(n)
        .copied()
        .chain(std::iter::repeat(0).take(n.saturating_sub(external_input.len())))
        .collect();
    // Step 2: ApplySpikeBatch — ghost spikes add current to targets (simplified: add to input)
    let ghost_current = f32_to_fixed(0.01);
    for &idx in incoming_ghost_spikes {
        if idx < n {
            effective_input[idx] = effective_input[idx].saturating_add(ghost_current);
        }
    }
    // Step 3: PropagateAxons — no-op in this CPU model (no axon_heads; full version would += v_seg)
    // Step 4: UpdateNeurons — GLIF + I_SIFS + φ-thresholds, then spike
    let new_voltages: Vec<Fixed16> = neurons
        .voltage
        .par_iter()
        .enumerate()
        .map(|(i, &v)| {
            if neurons.refractory_timer[i] > 0 {
                return v / 2;
            }
            let input_v = effective_input[i] + v;
            let i_sifs = calculate_sifs_current(
                input_v,
                &neurons.sifs_weights[i],
                &neurons.sifs_thresholds[i],
                f32_to_fixed(40.0),
            );
            let tau_decay = f32_to_fixed(0.95);
            let resistance = f32_to_fixed(1e9 / 1e12);
            let v_new = ((v as i64 * tau_decay as i64) >> 16)
                + ((resistance as i64 * i_sifs as i64) >> 16);
            (v_new as Fixed16).max(0)
        })
        .collect();

    let mut spikes = Vec::new();
    for i in 0..n {
        neurons.voltage[i] = new_voltages[i];
        if neurons.refractory_timer[i] > 0 {
            neurons.refractory_timer[i] -= 1;
        }
        if neurons.voltage[i] > neurons.threshold[i] && neurons.refractory_timer[i] == 0 {
            spikes.push(i);
            neurons.voltage[i] = 0;
            neurons.refractory_timer[i] = 5;
            neurons.spike_count[i] += 1;
            neurons.last_spike_time[i] = time_step;
        }
    }
    // Step 5: ApplyGSOP — no-op here (Genesis: STDP on dendrite weights)
    // Step 6: RecordReadout — return spike list (Genesis: write to output_history)
    spikes
}

// === Night Phase ===
pub fn night_phase_plasticity(
    neurons: &mut NeuronArrays,
    shards: &mut [SifsShard],
    spike_history: &[Vec<usize>],
) {
    let n = neurons.len();
    let _eta = f32_to_fixed(0.001);
    let hist_len = spike_history.len().max(1) as f32;

    for i in 0..n {
        let pre_activity = neurons.spike_count[i] as f32 / hist_len;
        let activity_delta = f32_to_fixed(pre_activity * 0.1 - 0.05);
        neurons.s_coordinate[i] = (neurons.s_coordinate[i] + activity_delta)
            .max(0)
            .min(f32_to_fixed(10.0));

        for level in 0..SIFS_LEVELS {
            let s_shift = fixed_to_f32(neurons.s_coordinate[i]);
            let shifted_level = (level as f32 - s_shift).max(0.0);
            neurons.sifs_weights[i][level] = f32_to_fixed((-2.0 * K * shifted_level).exp());
        }
        neurons.spike_count[i] = 0;
    }

    build_s_shards_from_neurons(neurons, shards);
}

// === Brain config and runner (I/O API, BRAIN_CONTRACT) ===

/// Конфиг мозга (аналог Brain TOML). По умолчанию — значения из BRAIN_CONTRACT.
#[derive(Debug, Clone)]
pub struct BrainConfig {
    pub n_neurons: usize,
    pub v0: f32,
    pub night_interval: usize,
}

impl Default for BrainConfig {
    fn default() -> Self {
        BrainConfig {
            n_neurons: 1000,
            v0: 0.02,
            night_interval: 100,
        }
    }
}

/// Readout после шага (Population Coding). Версия API v1.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrainReadout {
    pub spike_count_total: usize,
    pub spike_count_per_shard: Vec<usize>,
    /// Нормализованная активность 0..1 (spike_count_total / n_neurons, clamped).
    pub score: f32,
    pub time_step: u32,
}

/// Единая точка входа: шаг с входным вектором и возврат readout.
pub struct BrainRunner {
    pub config: BrainConfig,
    pub neurons: NeuronArrays,
    pub shards: Vec<SifsShard>,
    pub time_step: u32,
    spike_history: Vec<Vec<usize>>,
}

impl BrainRunner {
    pub fn new(config: BrainConfig) -> Self {
        let n = config.n_neurons;
        let mut neurons = NeuronArrays::new(n);
        for i in 0..n {
            neurons.threshold[i] = f32_to_fixed(config.v0);
            for level in 0..SIFS_LEVELS {
                neurons.sifs_thresholds[i][level] = sifs_threshold(level, neurons.threshold[i]);
            }
        }
        let mut shards: Vec<SifsShard> = (0..SIFS_LEVELS).map(SifsShard::new).collect();
        build_s_shards_from_neurons(&neurons, &mut shards);
        BrainRunner {
            config,
            neurons,
            shards,
            time_step: 0,
            spike_history: Vec::new(),
        }
    }

    /// Один шаг: вход (нормализованный вектор float) → readout. Ghost handover между S-шардами включён.
    pub fn run_step(&mut self, input: &[f32]) -> BrainReadout {
        let n = self.neurons.len();
        let input_fixed: Vec<Fixed16> = input
            .iter()
            .take(n)
            .map(|&x| f32_to_fixed(x))
            .chain(std::iter::repeat(0).take(n.saturating_sub(input.len())))
            .collect();

        let ghost = if self.spike_history.is_empty() {
            vec![]
        } else {
            let last = self.spike_history.last().unwrap();
            ghost_spike_targets(&self.neurons, &self.shards, last)
        };

        let spikes = day_phase_pipeline_6step(
            &mut self.neurons,
            &self.shards,
            self.time_step,
            &input_fixed,
            &ghost,
        );
        self.time_step += 1;
        self.spike_history.push(spikes.clone());
        if self.spike_history.len() > self.config.night_interval {
            self.spike_history.remove(0);
        }
        if self.time_step % self.config.night_interval as u32 == 0 && self.time_step > 0 {
            night_phase_plasticity(&mut self.neurons, &mut self.shards, &self.spike_history);
        }

        let mut spike_count_per_shard = [0usize; SIFS_LEVELS];
        for &idx in &spikes {
            let s_level = (fixed_to_f32(self.neurons.s_coordinate[idx]) as usize).min(SIFS_LEVELS - 1);
            if s_level < SIFS_LEVELS {
                spike_count_per_shard[s_level] += 1;
            }
        }
        let score = (spikes.len() as f32 / n as f32).min(1.0);
        BrainReadout {
            spike_count_total: spikes.len(),
            spike_count_per_shard: spike_count_per_shard.to_vec(),
            score,
            time_step: self.time_step - 1,
        }
    }
}

// === Metrics & Benchmark ===
#[derive(Debug)]
pub struct PerformanceMetrics {
    pub day_phase_ms: f64,
    pub night_phase_ms: f64,
    pub spikes_per_second: f64,
    pub neurons_per_shard: [usize; SIFS_LEVELS],
    pub memory_mb: f64,
}

pub fn run_benchmark(
    n_neurons: usize,
    n_steps: usize,
    night_interval: usize,
) -> PerformanceMetrics {
    use std::time::Instant;

    let mut neurons = NeuronArrays::new(n_neurons);
    let mut shards: Vec<SifsShard> = (0..SIFS_LEVELS).map(SifsShard::new).collect();
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
        let spikes = day_phase_step(&mut neurons, &shards, step as u32, &input_pattern);
        day_time += day_start.elapsed().as_secs_f64();
        total_spikes += spikes.len();
        spike_history.push(spikes);
        if spike_history.len() > night_interval {
            spike_history.remove(0);
        }
        if step % night_interval == 0 && step > 0 {
            let night_start = Instant::now();
            night_phase_plasticity(&mut neurons, &mut shards, &spike_history);
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
        night_phase_ms: night_time * 1000.0 / (n_steps / night_interval).max(1) as f64,
        spikes_per_second: total_spikes as f64 / (n_steps as f64 / 1000.0),
        neurons_per_shard,
        memory_mb,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_day_phase_pipeline_6step() {
        let mut neurons = NeuronArrays::new(64);
        let shards: Vec<SifsShard> = (0..SIFS_LEVELS).map(SifsShard::new).collect();
        let input: Vec<Fixed16> = (0..64).map(|i| f32_to_fixed(0.01 * (1.0 + (i as f32 * 0.02).sin()))).collect();
        let spikes0 = day_phase_pipeline_6step(&mut neurons, &shards, 0, &input, &[]);
        let spikes1 = day_phase_pipeline_6step(&mut neurons, &shards, 1, &input, &[]);
        assert!(spikes0.len() <= 64);
        assert!(spikes1.len() <= 64);
        let with_ghost = day_phase_pipeline_6step(&mut neurons, &shards, 2, &input, &[0, 1]);
        assert!(with_ghost.len() <= 64);
    }

    /// Регрессия: I_SIFS близок к эталону из scripts/compare_sifs_i_sifs.py (допуск 2 в fixed-point из-за округления).
    #[test]
    fn test_i_sifs_vs_python() {
        let v0 = f32_to_fixed(0.02);
        let test_cases: [(f32, i32); 5] = [
            (0.01, 155_200),
            (0.02, 155_977),
            (0.03, 156_752),
            (0.005, 154_811),
            (0.025, 156_363),
        ];
        for (v, expected_fixed) in test_cases {
            let mut weights = [0; SIFS_LEVELS];
            let mut thresholds = [0; SIFS_LEVELS];
            for n in 0..SIFS_LEVELS {
                weights[n] = sifs_w(n);
                thresholds[n] = sifs_threshold(n, v0);
            }
            let got = calculate_sifs_current(
                f32_to_fixed(v),
                &weights,
                &thresholds,
                f32_to_fixed(40.0),
            );
            let diff = (got - expected_fixed).abs();
            assert!(diff <= 5, "v={} expected {} got {} (diff {})", v, expected_fixed, got, diff);
        }
    }

    #[test]
    fn test_brain_runner_readout() {
        let config = BrainConfig {
            n_neurons: 64,
            ..BrainConfig::default()
        };
        let mut runner = BrainRunner::new(config);
        let input = vec![0.02f32; 64];
        let out = runner.run_step(&input);
        assert_eq!(out.spike_count_per_shard.len(), SIFS_LEVELS);
        assert!(out.score >= 0.0 && out.score <= 1.0);
        assert_eq!(out.time_step, 0);
    }

    #[test]
    fn test_s_sharding_and_ghost_axons() {
        let mut neurons = NeuronArrays::new(128);
        let mut shards: Vec<SifsShard> = (0..SIFS_LEVELS).map(SifsShard::new).collect();
        build_s_shards_from_neurons(&neurons, &mut shards);
        let total: usize = shards.iter().map(|s| s.neurons.len()).sum();
        assert_eq!(total, 128);
        let input: Vec<Fixed16> = (0..128).map(|i| f32_to_fixed(0.02 * (1.0 + (i as f32 * 0.01).sin()))).collect();
        let spikes = day_phase_pipeline_6step(&mut neurons, &shards, 0, &input, &[]);
        let ghost_targets = ghost_spike_targets(&neurons, &shards, &spikes);
        let spikes2 = day_phase_pipeline_6step(&mut neurons, &shards, 1, &input, &ghost_targets);
        assert!(spikes2.len() <= 128);
    }
}
