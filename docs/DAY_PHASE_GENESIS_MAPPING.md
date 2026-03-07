# Day Phase: соответствие пайплайну Genesis

Порядок шагов в `day_phase_pipeline_6step` совпадает с порядком ядер Day Phase в [genesis-agi 07_gpu_runtime §2.1](https://github.com/H4V1K-dev/genesis-agi/blob/master/docs/specs/07_gpu_runtime.md) и [05_signal_physics](https://github.com/H4V1K-dev/genesis-agi/blob/master/docs/specs/05_signal_physics.md).

| # | Kernel (Genesis) | В sifs-genesis-core (rayon) |
|---|------------------|-----------------------------|
| 1 | **InjectInputs** | Внешний вектор `external_input` копируется в `effective_input` (виртуальные аксоны). |
| 2 | **ApplySpikeBatch** | `incoming_ghost_spikes` добавляют токен к целевым индексам в `effective_input` (Ghost Axon birth). |
| 3 | **PropagateAxons** | Пока no-op: в CPU-модели нет `axon_heads`; в CUDA-версии будет `+= v_seg` для всех аксонов. |
| 4 | **UpdateNeurons** | GLIF-подобный шаг: утечка, **I_SIFS** (по весам W(n) и порогам V₀/φⁿ), порог, сброс при спайке, рефрактерность. |
| 5 | **ApplyGSOP** | Пока no-op: в Genesis — STDP на дендритных весах; при интеграции с genesis-compute — вызов их ядра. |
| 6 | **RecordReadout** | Возврат списка индексов спайков (аналог записи в `output_history`). |

## I_SIFS и φ-пороги (шаг 4)

- **Ток I_SIFS:** `calculate_sifs_current(voltage, sifs_weights, sifs_thresholds, beta)` — сумма по уровням n=0..9: W(n)·σ(V − V_th(n)), фиксированная точка, без float в hot path.
- **Пороги:** V_th(n) = V₀/φⁿ, V₀ из конфига (по умолчанию 0.02), синхронно с [BRAIN_CONTRACT.md](BRAIN_CONTRACT.md) и core.py.

## Дальнейшая интеграция

- **CUDA:** замена шагов 3–5 на вызовы ядер genesis-compute при сборке с `genesis` feature и линковке с genesis-compute.
- **S-Sharding:** распределение нейронов по S-уровням (SifsShard) уже есть; Ghost Axons на границах S — следующий шаг (п.4 плана).
