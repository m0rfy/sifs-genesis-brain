//! Day Phase and Night Phase (GPU-style sim + structural plasticity)

use rayon::prelude::*;

use crate::constants::{K, SIFS_LEVELS};
use crate::fixed::{f32_to_fixed, fixed_to_f32, Fixed16};
use crate::neuron::{calculate_sifs_current, NeuronArrays, SIFSShard};

pub fn day_phase_step(
    neurons: &mut NeuronArrays,
    _shards: &[SIFSShard],
    time_step: u32,
    external_input: &[Fixed16],
    sifs_enabled: bool,
) -> Vec<usize> {
    let mut spikes = Vec::new();
    let n = neurons.len();

    let new_voltages: Vec<Fixed16> = neurons
        .voltage
        .par_iter()
        .enumerate()
        .map(|(i, &v)| {
            if neurons.refractory_timer[i] > 0 {
                return v / 2;
            }
            let input_v = external_input.get(i).copied().unwrap_or(0) + v;
            let i_sifs = if sifs_enabled {
                calculate_sifs_current(
                    input_v,
                    &neurons.sifs_weights[i],
                    &neurons.sifs_thresholds[i],
                    f32_to_fixed(40.0),
                )
            } else {
                0
            };
            let tau_decay = f32_to_fixed(0.95);
            let resistance = f32_to_fixed(1e9 / 1e12);
            let v_new = ((v as i64 * tau_decay as i64) >> 16)
                + ((resistance as i64 * i_sifs as i64) >> 16);
            (v_new as Fixed16).max(0)
        })
        .collect();

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
    spikes
}

pub fn night_phase_plasticity(
    neurons: &mut NeuronArrays,
    shards: &mut Vec<SIFSShard>,
    spike_history: &[Vec<usize>],
    reward: f32,
) {
    let n = neurons.len();
    const ETA_BASE: f32 = 0.008;

    for i in 0..n {
        let pre_activity = neurons.spike_count[i] as f32 / spike_history.len() as f32;
        let reward_scale = 1.0 + reward * 0.5;
        let activity_delta = f32_to_fixed((pre_activity * 0.1 - 0.05) * reward_scale);
        neurons.s_coordinate[i] = (neurons.s_coordinate[i] + activity_delta)
            .max(0)
            .min(f32_to_fixed(10.0));

        for lv in 0..SIFS_LEVELS {
            let s_shift = fixed_to_f32(neurons.s_coordinate[i]);
            let shifted_level = (lv as f32 - s_shift).max(0.0);
            neurons.sifs_weights[i][lv] = f32_to_fixed((-2.0 * K * shifted_level).exp());
        }

        let s_level = (fixed_to_f32(neurons.s_coordinate[i]) as usize).min(SIFS_LEVELS - 1);
        let plasticity_factor = (SIFS_LEVELS - s_level) as f32 / SIFS_LEVELS as f32;
        let w_at_level = fixed_to_f32(neurons.sifs_weights[i][s_level]);
        let eta_effective = ETA_BASE * plasticity_factor.max(0.1) * w_at_level.max(0.15);
        let eta_i = f32_to_fixed(eta_effective);

        const INERTIA_LEVELS: usize = 16;
        let inertia_curve: [Fixed16; INERTIA_LEVELS] = [
            f32_to_fixed(1.00),
            f32_to_fixed(0.94),
            f32_to_fixed(0.88),
            f32_to_fixed(0.82),
            f32_to_fixed(0.76),
            f32_to_fixed(0.70),
            f32_to_fixed(0.64),
            f32_to_fixed(0.58),
            f32_to_fixed(0.52),
            f32_to_fixed(0.46),
            f32_to_fixed(0.40),
            f32_to_fixed(0.34),
            f32_to_fixed(0.28),
            f32_to_fixed(0.22),
            f32_to_fixed(0.16),
            f32_to_fixed(0.10),
        ];
        for (k, &j) in neurons.dendrite_targets[i].iter().enumerate() {
            if j >= n || k >= neurons.synaptic_weights[i].len() {
                continue;
            }
            let post_activity = neurons.spike_count[j] as f32 / spike_history.len() as f32;
            let hebbian_raw = pre_activity * post_activity * reward_scale;
            let hebbian_delta = (eta_i as i64 * f32_to_fixed(hebbian_raw) as i64) >> 16;
            let w_current = neurons.synaptic_weights[i][k] as i64;
            let rank = (w_current.unsigned_abs() as usize >> 13).min(INERTIA_LEVELS - 1);
            let inertia = inertia_curve[rank] as i64;
            let delta_scaled = (hebbian_delta * inertia) >> 16;
            let w = w_current + delta_scaled;
            neurons.synaptic_weights[i][k] =
                (w.max(0).min((2 * crate::fixed::FIXED_ONE) as i64)) as Fixed16;
        }

        neurons.spike_count[i] = 0;
    }

    for shard in shards.iter_mut() {
        shard.neurons.clear();
    }
    for i in 0..n {
        let s_level = (fixed_to_f32(neurons.s_coordinate[i]) as usize).min(SIFS_LEVELS - 1);
        shards[s_level].add_neuron(i);
    }
}
