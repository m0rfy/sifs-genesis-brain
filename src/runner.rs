//! BrainRunner: one step(input) -> (BrainReadout, action)

use crate::fixed::{fixed_to_f32, f32_to_fixed};
use crate::neuron::{BrainConfig, BrainReadout, NeuronArrays, SIFSShard};
use crate::phase::{day_phase_step, night_phase_plasticity};
use crate::constants::SIFS_LEVELS;

pub struct BrainRunner {
    pub neurons: NeuronArrays,
    pub shards: Vec<SIFSShard>,
    pub spike_history: Vec<Vec<usize>>,
    pub time_step: u32,
    pub config: BrainConfig,
    pub last_reward: f32,
    readout_weight_left: f32,
    readout_weight_right: f32,
    last_action: u32,
}

impl BrainRunner {
    pub fn new(config: BrainConfig) -> Self {
        let n = config.n_neurons;
        let mut neurons = NeuronArrays::new(n);
        neurons.set_v0(config.v0);
        let mut shards: Vec<SIFSShard> = (0..SIFS_LEVELS).map(SIFSShard::new).collect();
        for i in 0..n {
            let s_level = (i * SIFS_LEVELS / n).min(SIFS_LEVELS - 1);
            shards[s_level].add_neuron(i);
        }
        BrainRunner {
            neurons,
            shards,
            spike_history: Vec::new(),
            time_step: 0,
            config,
            last_reward: 0.0,
            readout_weight_left: 1.0,
            readout_weight_right: 1.0,
            last_action: 0,
        }
    }

    pub fn step(&mut self, input: &[f32]) -> (BrainReadout, u32) {
        let n = self.neurons.len();
        let input_fixed: Vec<crate::fixed::Fixed16> = input
            .iter()
            .map(|&x| f32_to_fixed(x))
            .chain(std::iter::repeat(0).take(n.saturating_sub(input.len())))
            .take(n)
            .collect();
        let steps = self.config.steps_per_observation.max(1);
        let mut last_spikes = Vec::new();
        for _ in 0..steps {
            last_spikes = day_phase_step(
                &mut self.neurons,
                &self.shards,
                self.time_step,
                &input_fixed,
                self.config.sifs_enabled,
            );
            self.time_step += 1;
            self.spike_history.push(last_spikes.clone());
            if self.spike_history.len() > self.config.night_interval {
                self.spike_history.remove(0);
            }
            if self.time_step as usize % self.config.night_interval == 0 && self.time_step > 0 {
                night_phase_plasticity(
                    &mut self.neurons,
                    &mut self.shards,
                    &self.spike_history,
                    self.last_reward,
                );
            }
        }
        if self.config.trainable_readout && self.time_step > 0 {
            let lr = self.config.readout_lr;
            let r = self.last_reward;
            if self.last_action == 0 {
                self.readout_weight_left += lr * r;
            } else {
                self.readout_weight_right += lr * r;
            }
            const W_MIN: f32 = 0.1;
            const W_MAX: f32 = 2.0;
            self.readout_weight_left = self.readout_weight_left.max(W_MIN).min(W_MAX);
            self.readout_weight_right = self.readout_weight_right.max(W_MIN).min(W_MAX);
        }

        let half = n / 2;
        let left_pop = last_spikes.iter().filter(|&&i| i < half).count() as f32 / half as f32;
        let right_pop = (last_spikes.len() - last_spikes.iter().filter(|&&i| i < half).count()) as f32
            / (n - half) as f32;
        let left_pop = left_pop.min(1.0);
        let right_pop = right_pop.min(1.0);
        let left_rec_raw =
            fixed_to_f32(self.neurons.weighted_recurrent_sum_in_range(&last_spikes, 0, half));
        let right_rec_raw =
            fixed_to_f32(self.neurons.weighted_recurrent_sum_in_range(&last_spikes, half, n));
        let norm = (half as f32 * 2.0).max(1.0);
        let left_rec = (left_rec_raw / norm).min(1.0).max(0.0);
        let right_rec = (right_rec_raw / norm).min(1.0).max(0.0);
        const BETA_RECURRENT: f32 = 0.4;
        let score_left = (left_pop + BETA_RECURRENT * left_rec).min(1.0);
        let score_right = (right_pop + BETA_RECURRENT * right_rec).min(1.0);
        let sl = score_left * self.readout_weight_left;
        let sr = score_right * self.readout_weight_right;
        let action = if sr > sl { 1u32 } else { 0 };
        self.last_action = action;
        let score = (score_left + score_right) / 2.0;
        let readout = BrainReadout {
            spike_count_total: last_spikes.len(),
            score,
            time_step: self.time_step - 1,
        };
        (readout, action)
    }

    pub fn trigger_night_phase(&mut self) {
        if !self.spike_history.is_empty() {
            night_phase_plasticity(
                &mut self.neurons,
                &mut self.shards,
                &self.spike_history,
                self.last_reward,
            );
        }
    }
}
