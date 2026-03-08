# B.4 Бенчмарки CPU (Day Phase, Night Phase, память)

Запуск: `cargo run --release` (compare_architectures). Конфиг: 5000 шагов, Night каждые 500 шагов.

| Нейроны | Day phase (ms/step) | Night phase (ms/cycle) | Память (MB) | Spikes/sec |
|---------|---------------------|------------------------|-------------|------------|
| 1K      | 0.047               | 0.043                  | 0.08        | 81 435     |
| 10K     | 0.103               | 0.441                  | 0.80        | 814 461    |
| 100K    | 0.361               | 4.248                  | 8.01        | 8 144 920  |

*Дата: 2025-03-07. CPU: sifs_genesis_hybrid (release), Windows.*

---

## GPU (genesis-agi)

Конфиг 100K: `examples/bench_100k/config/brain.toml`. Baking: `cargo run --release -p genesis-baker --bin baker -- --brain examples/bench_100k/config/brain.toml`. Запуск ноды: `cargo run --release -p genesis-node -- --manifest baked/Bench100K/manifest.toml --batch-size 100`.

| Нейроны | Day phase (ms/step) | TPS (ticks/s) | Память GPU (MB) |
|---------|---------------------|---------------|-----------------|
| 100K    | ~0.58               | ~1733         | *(nvidia-smi при запущенной ноде)* |
| 1M      | ~3.0                | ~333          | *(nvidia-smi при запущенной ноде)* |

Конфиг 1M: `examples/bench_1m/config/brain.toml`. Baking: `--brain examples/bench_1m/config/brain.toml`. Запуск с таймаутом: `.\scripts\run_bench_node_timed.ps1 45 baked/Bench1M/manifest.toml`.

*Дата: 2026-03-08. genesis-agi (release), Windows, CUDA_ARCH=sm_89. Нода выводит TPS в консоль; память — замерить `nvidia-smi` во время работы ноды. Для длительных прогонов: `--max-tps 1000` и/или `nvidia-smi -pl` — см. genesis-agi/RESOURCE_LIMITS.md.*
