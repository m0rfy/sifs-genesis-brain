# Бенчмарки и верификация

## Запуск бенчмарков

Из корня репо (или из `crates/sifs-genesis-core`):

```bash
cargo bench -p sifs-genesis-core
```

Измеряются три сценария:

| Бенчмарк | Нейроны | Шагов | Night interval |
|----------|---------|-------|----------------|
| brain_1k_1000steps | 1 000 | 1 000 | 100 |
| brain_10k_500steps | 10 000 | 500 | 100 |
| brain_100k_100steps | 100 000 | 100 | 50 |

Ориентир из [SIFS_Genesis_Hybrid_Report](../../Genesis/SIFS_Genesis_Hybrid_Report.md): ~0.38 ms/step для 10K нейронов (rayon). Релизная сборка: `cargo bench --release`.

## Числовая идентичность с Python

Константы K, PHI, W(n), V_th(n) синхронны с [core.py](../../sifs_ft/strategies/modules/core.py) и [BRAIN_CONTRACT](BRAIN_CONTRACT.md).

Регрессионный тест **I_SIFS** (фиксированная точка, sigma как в Rust):

1. Эталон: `python scripts/compare_sifs_i_sifs.py` — считает I_SIFS по той же формуле, что и `calculate_sifs_current` в Rust.
2. В крейте: тест `test_i_sifs_vs_python` сверяет результат `calculate_sifs_current` с эталонными значениями для v = 0.005, 0.01, 0.02, 0.025, 0.03.

После изменения формул в Rust перезапустите `compare_sifs_i_sifs.py` и при необходимости обновите константы в тесте.

## CartPole E2E

Полный цикл CartPole (Python Gymnasium → UDP → Genesis → Motor) планируется после интеграции с genesis-node и I/O матрицами. Текущий мозг (rayon, без CUDA) можно использовать для тестов readout и бенчмарков; для репродукции рекорда genesis-agi (71) потребуется запуск их стека (см. [genesis-agi Quick Start](https://github.com/H4V1K-dev/genesis-agi)).
