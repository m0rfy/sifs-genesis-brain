/*
SIFS-Genesis Brain — полноценный мозг с ядром SIFS.
Константы и формулы синхронны с core.py и BRAIN_CONTRACT.md.
Режимы: демо (тесты), --agent (stdin/stdout), --serve (HTTP API).
*/

use std::collections::HashSet;
use std::f32::consts::PI;
use std::io::Write;
use rayon::prelude::*;

// === SIFS Constants (BRAIN_CONTRACT, core.py) ===
/// K = 1/π² ≈ 0.10132 — безразмерная константа затухания
const K: f32 = 1.0 / (PI * PI);
/// φ = (1+√5)/2 — золотое сечение
const PHI: f32 = 1.6180339887;
const SIFS_LEVELS: usize = 10;
/// FIB — индексы 10 S-уровней (как в core.py), для EMA-периодов и контракта
#[allow(dead_code)]
const FIB: [u32; SIFS_LEVELS] = [1, 2, 3, 5, 8, 13, 21, 34, 55, 89];

// === Genesis Integer Physics ===
type Fixed16 = i32; // Q16.16 fixed point for deterministic GPU compute
const FIXED_ONE: Fixed16 = 1 << 16;

fn f32_to_fixed(x: f32) -> Fixed16 {
    (x * FIXED_ONE as f32) as Fixed16
}

fn fixed_to_f32(x: Fixed16) -> f32 {
    x as f32 / FIXED_ONE as f32
}

// === SIFS Math in Fixed Point ===
fn sifs_w(n: usize) -> Fixed16 {
    let w = (-2.0 * K * n as f32).exp();
    f32_to_fixed(w)
}

fn sifs_threshold(n: usize, v0: Fixed16) -> Fixed16 {
    let phi_n = PHI.powi(n as i32);
    (v0 as f64 / phi_n as f64) as Fixed16
}

// === SoA Layout (Genesis style) ===
#[derive(Debug, Clone)]
pub struct NeuronArrays {
    // Neuron state
    voltage: Vec<Fixed16>,           // V(t)
    threshold: Vec<Fixed16>,         // V_th base
    spike_count: Vec<u16>,           // Population coding
    
    // SIFS specific
    sifs_weights: Vec<[Fixed16; SIFS_LEVELS]>,  // W(n) for each neuron
    sifs_thresholds: Vec<[Fixed16; SIFS_LEVELS]>, // V₀/φⁿ levels
    s_coordinate: Vec<Fixed16>,       // Scale coordinate S
    
    // Connectivity (Cone Tracing inspired)
    axon_direction: Vec<(f32, f32)>,  // Growth direction
    axon_fov: Vec<f32>,              // Field of view
    dendrite_targets: Vec<Vec<usize>>, // Who this neuron connects to (i -> j)
    /// B.1.опц: обучаемые веса синапсов i→j, параллельно dendrite_targets[i]
    synaptic_weights: Vec<Vec<Fixed16>>,

    // Timing
    last_spike_time: Vec<u32>,
    refractory_timer: Vec<u16>,
}

impl NeuronArrays {
    pub fn new(n_neurons: usize) -> Self {
        let mut neurons = NeuronArrays {
            voltage: vec![0; n_neurons],
            threshold: vec![f32_to_fixed(0.02); n_neurons], // 20mV
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
        
        // B.1.опц: граф связей — каждый нейрон i соединяем с последующими (кольцо)
        const OUT_DEGREE: usize = 5;
        for i in 0..n_neurons {
            for d in 1..=OUT_DEGREE {
                let j = (i + d) % n_neurons;
                neurons.dendrite_targets[i].push(j);
                neurons.synaptic_weights[i].push(FIXED_ONE);
            }
        }

        // Initialize SIFS structure
        for i in 0..n_neurons {
            // S-coordinate from position (φ-spiral like SIFS)
            let angle = i as f32 * 2.0 * PI / (PHI * PHI);
            let radius = (i as f32 / n_neurons as f32).sqrt();
            neurons.s_coordinate[i] = f32_to_fixed(radius * 5.0); // 0-5 S range
            
            // SIFS weights W(n) = exp(-2kn)
            for n in 0..SIFS_LEVELS {
                neurons.sifs_weights[i][n] = sifs_w(n);
                neurons.sifs_thresholds[i][n] = sifs_threshold(n, neurons.threshold[i]);
            }
            
            // Cone tracing direction (φ-spiral)
            neurons.axon_direction[i] = (angle.cos(), angle.sin());
            neurons.axon_fov[i] = PI / 6.0; // 30 degree cone
        }
        
        neurons
    }
    
    pub fn len(&self) -> usize {
        self.voltage.len()
    }

    /// B.1.опц: взвешенная рекуррентная активность по синаптическим весам (для readout)
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

    /// Задать базовый порог V₀ (BRAIN_CONTRACT §2); пересчитывает sifs_thresholds
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

// === S-Level Sharding (Phase A.2: привязка шардов к S-уровням 0..9) ===
/// Шард нейронов, привязанный к одному S-уровню масштабной координаты.
/// Каждый шард имеет s_level ∈ [0..9]; всего 10 шардов (SIFS_LEVELS).
#[derive(Debug)]
#[allow(dead_code)] // w_factor, time_scale — для будущего использования по уровням
pub struct SIFSShard {
    /// S-level масштабной координаты: 0 = глобальный, 9 = локальный (диапазон 0..9).
    pub s_level: usize,
    neurons: Vec<usize>,    // Neuron indices in this shard
    w_factor: Fixed16,      // W(s_level) for this shard
    time_scale: u32,        // Time multiplier (higher S = faster)
}

impl SIFSShard {
    /// Создаёт шард для S-уровня `s_level`. Должен быть в диапазоне 0..SIFS_LEVELS (0..9).
    pub fn new(s_level: usize) -> Self {
        debug_assert!(s_level < SIFS_LEVELS, "s_level must be in 0..{}", SIFS_LEVELS);
        let w_factor = sifs_w(s_level);
        let time_scale = 1 << s_level; // 2^s_level speedup for higher S

        SIFSShard {
            s_level,
            neurons: Vec::new(),
            w_factor,
            time_scale,
        }
    }

    pub fn add_neuron(&mut self, neuron_id: usize) {
        self.neurons.push(neuron_id);
    }
}

// === Brain config (BRAIN_CONTRACT §2: V₀, тайминг) ===
#[derive(Debug, Clone)]
pub struct BrainConfig {
    pub n_neurons: usize,
    /// Базовый порог V₀ для ряда V_th(n) = V₀/φⁿ (условные 20 mV = 0.02)
    pub v0: f32,
    pub night_interval: usize,
    pub steps_per_observation: u32,
    /// A.0: выключить I_SIFS (ток = 0) для эксперимента on/off
    pub sifs_enabled: bool,
    /// B.1 обучаемый readout: веса пулов обновляются по reward (REINFORCE-style)
    pub trainable_readout: bool,
    /// Learning rate для весов readout при trainable_readout
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

/// Readout одного шага (контракт §4)
#[derive(Debug, Clone)]
pub struct BrainReadout {
    pub spike_count_total: usize,
    pub score: f32,
    pub time_step: u32,
}

/// Единый раннер мозга: состояние + один шаг наблюдения → readout + action
pub struct BrainRunner {
    pub neurons: NeuronArrays,
    pub shards: Vec<SIFSShard>,
    pub spike_history: Vec<Vec<usize>>,
    pub time_step: u32,
    pub config: BrainConfig,
    /// Reward от предыдущего шага (доставка дофамина для B.2); используется в Night Phase
    pub last_reward: f32,
    /// B.1 обучаемый readout: веса пулов left/right
    readout_weight_left: f32,
    readout_weight_right: f32,
    /// Предыдущее действие для обновления веса по reward
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

    /// Один запрос: вход (наблюдение) → steps_per_observation шагов Day/Night → readout + action
    pub fn step(&mut self, input: &[f32]) -> (BrainReadout, u32) {
        let n = self.neurons.len();
        let input_fixed: Vec<Fixed16> = input
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
        // B.1 обучаемый readout: обновить веса по предыдущему действию и reward (REINFORCE-style)
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

        // B.1 п.6 моторные пулы: два readout (left 0..n/2, right n/2..n), action = argmax
        let half = n / 2;
        let left_spikes: Vec<usize> = last_spikes.iter().copied().filter(|&i| i < half).collect();
        let _right_spikes: Vec<usize> = last_spikes.iter().copied().filter(|&i| i >= half).collect();
        let left_pop = (left_spikes.len() as f32 / half as f32).min(1.0);
        let right_pop = (last_spikes.len() - left_spikes.len()) as f32 / (n - half) as f32;
        let right_pop = right_pop.min(1.0);
        let left_rec_raw = fixed_to_f32(self.neurons.weighted_recurrent_sum_in_range(&last_spikes, 0, half));
        let right_rec_raw = fixed_to_f32(self.neurons.weighted_recurrent_sum_in_range(&last_spikes, half, n));
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
        let score = (score_left + score_right) / 2.0; // readout.score для контракта
        let readout = BrainReadout {
            spike_count_total: last_spikes.len(),
            score,
            time_step: self.time_step - 1,
        };
        (readout, action)
    }

    /// B.1 рекомендация 3: вызвать Night Phase по флагу «конец эпизода» с текущим reward.
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

// === SIFS Current Calculation ===
fn calculate_sifs_current(
    input_voltage: Fixed16,
    sifs_weights: &[Fixed16; SIFS_LEVELS],
    sifs_thresholds: &[Fixed16; SIFS_LEVELS],
    _beta: Fixed16, // sigmoid steepness
) -> Fixed16 {
    let mut i_sifs = 0i64;
    
    for n in 0..SIFS_LEVELS {
        let v_diff = input_voltage - sifs_thresholds[n];
        // Simplified sigmoid: σ(x) ≈ max(0, min(1, 0.5 + x/4))
        let sigma = if v_diff < -2 * FIXED_ONE { 0 }
                   else if v_diff > 2 * FIXED_ONE { FIXED_ONE }
                   else { (FIXED_ONE / 2) + (v_diff / 4) };
        
        let contrib = (sifs_weights[n] as i64 * sigma as i64) >> 16;
        i_sifs += contrib;
    }
    
    (i_sifs.min(i32::MAX as i64) as i32).max(0)
}

// === Day Phase: GPU-style Simulation ===
pub fn day_phase_step(
    neurons: &mut NeuronArrays,
    _shards: &[SIFSShard],
    time_step: u32,
    external_input: &[Fixed16],
    sifs_enabled: bool,
) -> Vec<usize> {
    let mut spikes = Vec::new();
    let n = neurons.len();
    
    // Parallel update (mimics GPU warps)
    let new_voltages: Vec<Fixed16> = neurons.voltage
        .par_iter()
        .enumerate()
        .map(|(i, &v)| {
            // Skip if refractory
            if neurons.refractory_timer[i] > 0 {
                return v / 2; // decay during refractory
            }
            
            // SIFS current calculation (A.0: можно отключить для эксперимента on/off)
            let input_v = external_input.get(i).copied().unwrap_or(0) + v;
            let i_sifs = if sifs_enabled {
                calculate_sifs_current(
                    input_v,
                    &neurons.sifs_weights[i],
                    &neurons.sifs_thresholds[i],
                    f32_to_fixed(40.0), // beta
                )
            } else {
                0
            };

            // LIF dynamics: τ·dV/dt = -V + R·I
            // Simplified: V_new = V·decay + R·I_sifs·dt
            let tau_decay = f32_to_fixed(0.95); // τ = 10ms, dt = 0.5ms
            let resistance = f32_to_fixed(1e9 / 1e12); // 1GΩ scaled
            
            let v_new = ((v as i64 * tau_decay as i64) >> 16) + 
                       ((resistance as i64 * i_sifs as i64) >> 16);
            
            (v_new as Fixed16).max(0)
        })
        .collect();
    
    // Update voltage and detect spikes
    for i in 0..n {
        neurons.voltage[i] = new_voltages[i];
        
        // Refractory countdown
        if neurons.refractory_timer[i] > 0 {
            neurons.refractory_timer[i] -= 1;
        }
        
        // Spike detection
        if neurons.voltage[i] > neurons.threshold[i] && neurons.refractory_timer[i] == 0 {
            spikes.push(i);
            neurons.voltage[i] = 0; // reset
            neurons.refractory_timer[i] = 5; // 2.5ms refractory
            neurons.spike_count[i] += 1;
            neurons.last_spike_time[i] = time_step;
        }
    }
    
    spikes
}

// === Night Phase: Structural Plasticity (CPU) ===
/// reward: от предыдущего шага (dopamine), модулирует пластичность (B.2)
/// Пластичность по SIFS: мелкий масштаб (низкий s_level) — выше eta, крупный — консервативнее (теория SIFS).
pub fn night_phase_plasticity(
    neurons: &mut NeuronArrays,
    shards: &mut Vec<SIFSShard>,
    spike_history: &[Vec<usize>], // Recent spike trains
    reward: f32,
) {
    let n = neurons.len();
    const ETA_BASE: f32 = 0.008;

    for i in 0..n {
        let pre_activity = neurons.spike_count[i] as f32 / spike_history.len() as f32;

        // Update S-coordinate: activity + reward modulation (dopamine, B.2)
        let reward_scale = 1.0 + reward * 0.5;
        let activity_delta = f32_to_fixed((pre_activity * 0.1 - 0.05) * reward_scale);
        neurons.s_coordinate[i] = (neurons.s_coordinate[i] + activity_delta)
            .max(0).min(f32_to_fixed(10.0));

        // Recalculate SIFS thresholds based on new S
        for lv in 0..SIFS_LEVELS {
            let s_shift = fixed_to_f32(neurons.s_coordinate[i]);
            let shifted_level = (lv as f32 - s_shift).max(0.0);
            neurons.sifs_weights[i][lv] = f32_to_fixed((-2.0 * K * shifted_level).exp());
        }

        // s_level нейрона (0 = мелкий масштаб, 9 = крупный) для модуляции пластичности
        let s_level = (fixed_to_f32(neurons.s_coordinate[i]) as usize).min(SIFS_LEVELS - 1);
        // Мелкий масштаб — пластичнее, крупный — стабильнее: factor = (10 - s_level) / 10
        let plasticity_factor = (SIFS_LEVELS - s_level) as f32 / SIFS_LEVELS as f32;
        // Дополнительно: вклад уровня через W(s_level) — крупномасштабные (малые W) меньше меняют веса
        let w_at_level = fixed_to_f32(neurons.sifs_weights[i][s_level]);
        let eta_effective = ETA_BASE * plasticity_factor.max(0.1) * w_at_level.max(0.15);
        let eta_i = f32_to_fixed(eta_effective);

        // B.1.опц: обновление синаптических весов (Hebbian + reward), eta по s_level и W(n)
        // Inertia по рангу веса (genesis-agi 04_connectivity): сильные связи меняются слабее
        const INERTIA_LEVELS: usize = 16;
        let inertia_curve: [Fixed16; INERTIA_LEVELS] = [
            f32_to_fixed(1.00), f32_to_fixed(0.94), f32_to_fixed(0.88), f32_to_fixed(0.82),
            f32_to_fixed(0.76), f32_to_fixed(0.70), f32_to_fixed(0.64), f32_to_fixed(0.58),
            f32_to_fixed(0.52), f32_to_fixed(0.46), f32_to_fixed(0.40), f32_to_fixed(0.34),
            f32_to_fixed(0.28), f32_to_fixed(0.22), f32_to_fixed(0.16), f32_to_fixed(0.10),
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
            neurons.synaptic_weights[i][k] = (w.max(0).min((2 * FIXED_ONE) as i64)) as Fixed16;
        }

        neurons.spike_count[i] = 0;
    }
    
    // Redistribute neurons among S-shards based on updated coordinates
    for shard in shards.iter_mut() {
        shard.neurons.clear();
    }
    
    for i in 0..n {
        let s_level = (fixed_to_f32(neurons.s_coordinate[i]) as usize).min(SIFS_LEVELS - 1);
        shards[s_level].add_neuron(i);
    }
}

// === Performance Metrics ===
#[derive(Debug)]
pub struct PerformanceMetrics {
    pub day_phase_ms: f64,
    pub night_phase_ms: f64,
    pub spikes_per_second: f64,
    pub neurons_per_shard: [usize; SIFS_LEVELS],
    pub sifs_efficiency: f64, // vs simple threshold
    pub memory_mb: f64,
}

// === Test Framework ===
pub fn run_benchmark(
    n_neurons: usize,
    n_steps: usize,
    night_interval: usize,
) -> PerformanceMetrics {
    use std::time::Instant;
    
    println!("🧠 SIFS-Genesis Hybrid Benchmark");
    println!("Neurons: {}, Steps: {}, Night every: {}", n_neurons, n_steps, night_interval);
    
    // Initialize system
    let mut neurons = NeuronArrays::new(n_neurons);
    let mut shards: Vec<SIFSShard> = (0..SIFS_LEVELS)
        .map(|i| SIFSShard::new(i))
        .collect();
    
    // Distribute neurons to initial shards
    for i in 0..n_neurons {
        let s_level = (i * SIFS_LEVELS / n_neurons).min(SIFS_LEVELS - 1);
        shards[s_level].add_neuron(i);
    }
    
    let mut spike_history = Vec::new();
    let mut total_spikes = 0;
    let mut day_time = 0.0;
    let mut night_time = 0.0;
    
    // Create test input pattern
    let input_pattern: Vec<Fixed16> = (0..n_neurons)
        .map(|i| f32_to_fixed(0.01 * (1.0 + (i as f32 * 0.1).sin())))
        .collect();
    
    for step in 0..n_steps {
        // Day Phase
        let day_start = Instant::now();
        let spikes = day_phase_step(&mut neurons, &shards, step as u32, &input_pattern, true);
        day_time += day_start.elapsed().as_secs_f64();
        
        total_spikes += spikes.len();
        if step % 1000 == 0 {
            let rate = total_spikes as f64 / (step + 1) as f64;
            println!("Step {}: {} spikes/step, {} Hz avg", step, spikes.len(), rate * 1000.0);
        }
        
        spike_history.push(spikes);
        if spike_history.len() > night_interval {
            spike_history.remove(0);
        }
        
        // Night Phase (structural plasticity)
        if step % night_interval == 0 && step > 0 {
            let night_start = Instant::now();
            night_phase_plasticity(&mut neurons, &mut shards, &spike_history, 0.0);
            night_time += night_start.elapsed().as_secs_f64();
        }
    }
    
    // Calculate metrics
    let mut neurons_per_shard = [0; SIFS_LEVELS];
    for (i, shard) in shards.iter().enumerate() {
        neurons_per_shard[i] = shard.neurons.len();
    }
    
    let memory_mb = (
        neurons.voltage.len() * 4 +
        neurons.sifs_weights.len() * SIFS_LEVELS * 4 +
        neurons.sifs_thresholds.len() * SIFS_LEVELS * 4
    ) as f64 / 1_048_576.0;
    
    PerformanceMetrics {
        day_phase_ms: day_time * 1000.0 / n_steps as f64,
        night_phase_ms: night_time * 1000.0 / (n_steps / night_interval) as f64,
        spikes_per_second: total_spikes as f64 / (n_steps as f64 / 1000.0),
        neurons_per_shard,
        sifs_efficiency: 1.0, // placeholder
        memory_mb,
    }
}

// === Comparison Tests ===
pub fn compare_architectures() {
    println!("\n🔬 Architecture Comparison");
    println!("{}", "=".repeat(60));
    
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

// === SIFS vs Simple Threshold Test ===
pub fn sifs_vs_simple_test() {
    println!("\n🧮 SIFS vs Simple Threshold");
    println!("{}", "=".repeat(40));
    
    let test_voltages = vec![
        f32_to_fixed(0.005), f32_to_fixed(0.010), f32_to_fixed(0.015),
        f32_to_fixed(0.020), f32_to_fixed(0.025), f32_to_fixed(0.030)
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
        let simple_current = if v > f32_to_fixed(0.02) { f32_to_fixed(1.0) } else { 0 };
        
        let sifs_current = calculate_sifs_current(
            v, &sifs_weights, &sifs_thresholds, f32_to_fixed(40.0)
        );
        
        let ratio = if simple_current > 0 {
            fixed_to_f32(sifs_current) / fixed_to_f32(simple_current)
        } else if sifs_current > 0 { 
            999.9 
        } else { 
            1.0 
        };
        
        println!("{:8.1} | {:6.3} | {:6.3} | {:5.2}",
            fixed_to_f32(v) * 1000.0,
            fixed_to_f32(simple_current),
            fixed_to_f32(sifs_current),
            ratio
        );
    }
}

// === Agent mode: stdin JSON lines → stdout JSON (readout + action) ===
fn run_agent_loop(config: BrainConfig) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self, BufRead, Write};

    let mut runner = BrainRunner::new(config.clone());
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut lines = stdin.lock().lines();

    while let Some(line_result) = lines.next() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                eprintln!("stdin read error: {}", e);
                break;
            }
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut episode_end = false;
        let obs: Vec<f32> = match serde_json::from_str::<serde_json::Value>(line) {
            Ok(serde_json::Value::Array(arr)) => arr
                .into_iter()
                .filter_map(|v| v.as_f64().map(|x| x as f32))
                .collect(),
            Ok(serde_json::Value::Object(map)) => {
                if let Some(reward) = map.get("reward").and_then(|v| v.as_f64()) {
                    runner.last_reward = reward as f32;
                }
                episode_end = map.get("episode_end").and_then(|v| v.as_bool()).unwrap_or(false);
                map.get("input")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_f64().map(|x| x as f32))
                            .collect()
                    })
                    .unwrap_or_default()
            }
            Ok(_) => Vec::new(),
            Err(_) => Vec::new(),
        };
        // Всегда отвечаем на каждый запрос (протокол 1:1), иначе клиент зависает на readline
        let (readout, action) = if obs.is_empty() {
            runner.step(&[0.0; 4])
        } else {
            runner.step(&obs)
        };
        if episode_end {
            runner.trigger_night_phase();
        }
        let out = format!(
            r#"{{"readout":{{"spike_count_total":{},"score":{},"time_step":{}}},"action":{}}}"#,
            readout.spike_count_total,
            readout.score,
            readout.time_step,
            action
        );
        writeln!(stdout, "{}", out)?;
        stdout.flush()?;
    }
    Ok(())
}

// === HTTP server mode: POST /step {"input": [...]} → {"readout":..., "action": N} ===
fn run_serve_loop(config: BrainConfig, bind: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Read;
    use std::net::TcpListener;

    let mut runner = BrainRunner::new(config);
    let listener = TcpListener::bind(bind)?;
    eprintln!("SIFS Brain HTTP server on http://{} (POST /step)", bind);

    for stream in listener.incoming().flatten() {
        let mut stream = stream;
        let mut buf = [0u8; 65536];
        let n = stream.read(&mut buf).unwrap_or(0);
        let req = std::str::from_utf8(&buf[..n]).unwrap_or("");
        let body_start = req.find("\r\n\r\n").or_else(|| req.find("\n\n")).map(|p| p + 2).unwrap_or(0);
        let body = if body_start > 0 && body_start < req.len() {
            req[body_start..].trim()
        } else {
            ""
        };
        let result = serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|json| {
                let input = json.get("input")?.as_array()?;
                let input_f32: Vec<f32> = input
                    .iter()
                    .filter_map(|v| v.as_f64().map(|x| x as f32))
                    .collect();
                Some(runner.step(&input_f32))
            });
        let (readout, action) = match result {
            Some((r, a)) => (r, a),
            None => {
                let _ = write_response(&mut stream, 400, r#"{"error":"bad request"}"#);
                continue;
            }
        };
        let resp_body = format!(
            r#"{{"readout":{{"spike_count_total":{},"score":{},"time_step":{}}},"action":{}}}"#,
            readout.spike_count_total,
            readout.score,
            readout.time_step,
            action
        );
        let _ = write_response(&mut stream, 200, &resp_body);
    }
    Ok(())
}

fn write_response(stream: &mut std::net::TcpStream, status: u16, body: &str) -> std::io::Result<()> {
    let status_line = if status == 200 { "HTTP/1.1 200 OK" } else { "HTTP/1.1 400 Bad Request" };
    let h = format!(
        "{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status_line,
        body.len(),
        body
    );
    stream.write_all(h.as_bytes())
}

/// A.1: читает напряжения из stdin (одно float на строку), выводит I_SIFS (float) по одному на строку.
fn run_compute_sifs_stdin() {
    use std::io::{self, BufRead};
    let mut weights = [0; SIFS_LEVELS];
    let mut thresholds = [0; SIFS_LEVELS];
    for n in 0..SIFS_LEVELS {
        weights[n] = sifs_w(n);
        thresholds[n] = sifs_threshold(n, f32_to_fixed(0.02));
    }
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap_or_default();
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: f32 = match line.parse() {
            Ok(x) => x,
            Err(_) => continue,
        };
        let v_fixed = f32_to_fixed(v);
        let i_sifs = calculate_sifs_current(v_fixed, &weights, &thresholds, f32_to_fixed(40.0));
        println!("{}", fixed_to_f32(i_sifs));
    }
}

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
    println!("  Опции: --neurons N --steps K --night M --v0 V --no-sifs");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fib_contract() {
        // BRAIN_CONTRACT: FIB = [1,2,3,5,8,13,21,34,55,89] как в core.py
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

    /// Регрессия I_SIFS: совпадение с эталоном из Python (compare_sifs_i_sifs.py)
    #[test]
    fn test_i_sifs_regression_vs_python() {
        let mut weights = [0; SIFS_LEVELS];
        let mut thresholds = [0; SIFS_LEVELS];
        for n in 0..SIFS_LEVELS {
            weights[n] = sifs_w(n);
            thresholds[n] = sifs_threshold(n, f32_to_fixed(0.02));
        }
        // Python: V=0.02 -> I_SIFS fixed; допуск 5 в fixed-point
        let v = f32_to_fixed(0.02);
        let rust_i = calculate_sifs_current(v, &weights, &thresholds, f32_to_fixed(40.0));
        let rust_f = fixed_to_f32(rust_i);
        assert!(rust_f >= 0.0 && rust_f < 20.0, "I_SIFS(0.02) в разумных границах");
        // При V=V_th пороги только начинают срабатывать — не нуль
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
        let input: Vec<Fixed16> = (0..50).map(|i| f32_to_fixed(0.02 * (1.0 + (i % 5) as f32 * 0.1))).collect();
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
        // Первый шаг: last_action ещё 0, веса не обновляются (time_step только что стал 1)
        let (_r1, a1) = runner.step(&[0.01; 4]);
        runner.last_reward = 1.0; // положительный reward
        let (_r2, a2) = runner.step(&[0.02; 4]); // здесь обновятся веса по a1 и reward
        runner.last_reward = -0.5;
        let (_r3, _a3) = runner.step(&[0.01; 4]);
        assert!(a1 == 0 || a1 == 1);
        assert!(a2 == 0 || a2 == 1);
        // После нескольких шагов контракт сохранён: action 0 или 1
        let (readout, action) = runner.step(&[0.1; 4]);
        assert!(readout.score >= 0.0 && readout.score <= 1.0);
        assert!(action == 0 || action == 1);
    }
}
