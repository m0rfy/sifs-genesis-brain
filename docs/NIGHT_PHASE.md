# Night Phase: текущая реализация и соответствие Genesis

## Что реализовано (rayon / CPU)

Функция **`night_phase_plasticity`** в `sifs-genesis-core`:

1. **Hebbian ΔS** — по активности нейрона (spike_count / history_len) корректируется S-координата: `s_coordinate += activity_delta`, clamp 0..10. Нейроны «переезжают» по S-уровням в зависимости от активности.
2. **Обновление W(n)** — веса `sifs_weights[i][level]` пересчитываются от сдвинутой координаты: `W(shifted_level) = exp(-2·K·shifted_level)`.
3. **Resharding** — вызов `build_s_shards_from_neurons`: шарды заново заполняются по текущему `s_coordinate`.

Вызывается из `BrainRunner::run_step` каждые `night_interval` шагов и из `run_benchmark`.

## Что даёт полный Genesis (07_gpu_runtime §2.2, 09_baking_pipeline)

| Шаг Genesis | Где | В sifs-genesis-core |
|-------------|-----|----------------------|
| 1. Sort & Prune | GPU | Нет — нет дендритных весов/слотов в нашей модели |
| 2. Download (VRAM → RAM) | PCIe | — |
| 3. Sprouting & Cone Tracing | CPU | Нет — топология связей пока не растёт (нет dendrite_targets) |
| 4. Baking | CPU | — |
| 5. Upload (RAM → VRAM) | PCIe | — |

При интеграции с **genesis-compute** и **genesis-baker** полный Night Phase будет в их пайплайне; наш `night_phase_plasticity` остаётся CPU-аналогом Hebbian ΔS и resharding по S для автономного режима (без CUDA).

## Ссылки

- [07_gpu_runtime §2.2](https://github.com/H4V1K-dev/genesis-agi/blob/master/docs/specs/07_gpu_runtime.md) — конвейер Maintenance (5 шагов)
- [09_baking_pipeline](https://github.com/H4V1K-dev/genesis-agi/blob/master/docs/specs/09_baking_pipeline.md) — Sort & Prune, Sprouting, Baking
