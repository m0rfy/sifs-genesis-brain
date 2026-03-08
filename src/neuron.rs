//! NeuronArrays, SIFSShard, BrainConfig, BrainReadout, calculate_sifs_current

use std::collections::HashSet;

use crate::constants::SIFS_LEVELS;
use crate::fixed::{f32_to_fixed, sifs_threshold, sifs_w, Fixed16, FIXED_ONE};

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
    pub synaptic_weights: Vec<Vec<Fixed16>>,
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
            synaptic_weights: vec![Vec::new(); n_neurons],
            last_spike_time: vec![0; n_neurons],
            refractory_timer: vec![0; n_neurons],
        };
        const OUT_DEGREE: usize = 5;
        for i in 0..n_neurons {
            for d in 1..=OUT_DEGREE {
                let j = (i + d) % n_neurons;
                neurons.dendrite_targets[i].push(j);
                neurons.synaptic_weights[i].push(FIXED_ONE);
            }
        }
        let pi = std::f32::consts::PI;
        for i in 0..n_neurons {
            let angle = i as f32 * 2.0 * pi / (crate::constants::PHI * crate::constants::PHI);
            let radius = (i as f32 / n_neurons as f32).sqrt();
            neurons.s_coordinate[i] = f32_to_fixed(radius * 5.0);
            for n in 0..SIFS_LEVELS {
                neurons.sifs_weights[i][n] = sifs_w(n);
                neurons.sifs_thresholds[i][n] = sifs_threshold(n, neurons.threshold[i]);
            }
            neurons.axon_direction[i] = (angle.cos(), angle.sin());
            neurons.axon_fov[i] = pi / 6.0;
        }
        neurons
    }

    pub fn len(&self) -> usize {
        self.voltage.len()
    }

    #[allow(dead_code)] // используется при едином readout; пулы используют weighted_recurrent_sum_in_range
    pub fn weighted_recurrent_sum(&self, spike_indices: &[usize]) -> Fixed16 {
        let set: HashSet<usize> = spike_indices.iter().copied().collect();
        let mut sum: i64 = 0;
        for &i in spike_indices {
            for (k, &j) in self.dendrite_targets[i].iter().enumerate() {
                if set.contains(&j) {
                    let w = self.synaptic_weights[i].get(k).copied().unwrap_or(0) as i64;
                    sum += w;
                }
            }
        }
        (sum.min(i32::MAX as i64).max(i32::MIN as i64)) as Fixed16
    }

    /// B.1 п.6 моторные пулы: рекуррентная сумма только по нейронам в [start, end)
    pub fn weighted_recurrent_sum_in_range(
        &self,
        spike_indices: &[usize],
        start: usize,
        end: usize,
    ) -> Fixed16 {
        let set: HashSet<usize> = spike_indices.iter().copied().collect();
        let mut sum: i64 = 0;
        for &i in spike_indices {
            if i < start || i >= end {
                continue;
            }
            for (k, &j) in self.dendrite_targets[i].iter().enumerate() {
                if j >= start && j < end && set.contains(&j) {
                    let w = self.synaptic_weights[i].get(k).copied().unwrap_or(0) as i64;
                    sum += w;
                }
            }
        }
        (sum.min(i32::MAX as i64).max(i32::MIN as i64)) as Fixed16
    }

    pub fn set_v0(&mut self, v0: f32) {
        let v0_fixed = f32_to_fixed(v0);
        for i in 0..self.voltage.len() {
            self.threshold[i] = v0_fixed;
            for n in 0..SIFS_LEVELS {
                self.sifs_thresholds[i][n] = sifs_threshold(n, v0_fixed);
            }
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)] // s_level, w_factor, time_scale — для будущего использования по S-уровням
pub struct SIFSShard {
    pub s_level: usize,
    pub neurons: Vec<usize>,
    w_factor: Fixed16,
    time_scale: u32,
}

impl SIFSShard {
    pub fn new(s_level: usize) -> Self {
        debug_assert!(s_level < SIFS_LEVELS, "s_level must be in 0..{}", SIFS_LEVELS);
        SIFSShard {
            s_level,
            neurons: Vec::new(),
            w_factor: sifs_w(s_level),
            time_scale: 1 << s_level,
        }
    }

    pub fn add_neuron(&mut self, neuron_id: usize) {
        self.neurons.push(neuron_id);
    }
}

#[derive(Debug, Clone)]
pub struct BrainConfig {
    pub n_neurons: usize,
    pub v0: f32,
    pub night_interval: usize,
    pub steps_per_observation: u32,
    pub sifs_enabled: bool,
    /// B.1 обучаемый readout: веса пулов обновляются по reward (REINFORCE-style)
    pub trainable_readout: bool,
    pub readout_lr: f32,
}

impl Default for BrainConfig {
    fn default() -> Self {
        BrainConfig {
            n_neurons: 1000,
            v0: 0.02,
            night_interval: 500,
            steps_per_observation: 1,
            sifs_enabled: true,
            trainable_readout: false,
            readout_lr: 0.05,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BrainReadout {
    pub spike_count_total: usize,
    pub score: f32,
    pub time_step: u32,
}

/// SIFS current calculation (used by phase and bench)
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
