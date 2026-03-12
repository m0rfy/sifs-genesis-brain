# План и дорожная карта: SIFS Brain (Genesis)

Один видимый план: наша логика (ядро SIFS из core.py), проверки, путь от CPU до genesis-node/CUDA.

---

## Цель

**Полноценный мозг с ядром SIFS**, используемый локально или на сервере:

- Единый источник констант и формул — [sifs_ft/strategies/modules/core.py](../sifs_ft/strategies/modules/core.py); в Rust — строго наша логика (K, PHI, FIB, W(n), V_th(n)=V₀/φⁿ, I_SIFS).
- Режимы: **CPU-гибрид** (текущий бинарник в Genesis/) → затем **genesis-node + CUDA** с тем же ядром.
- Все тесты и проверки должны проходить; контракт входа/выхода — [BRAIN_CONTRACT.md](BRAIN_CONTRACT.md).

---

## Текущий статус (что уже сделано)

| Компонент | Статус | Где |
|-----------|--------|-----|
| Константы K, PHI, FIB, W(n) | ✅ синхронно с core.py | [sifs_genesis_hybrid.rs](sifs_genesis_hybrid.rs) |
| I_SIFS в fixed-point, пороги V_th(n) | ✅ наша формула | `calculate_sifs_current`, `sifs_threshold` |
| Day Phase (LIF + SIFS), Night Phase (ΔS, resharding) | ✅ | `day_phase_step`, `night_phase_plasticity` |
| BrainConfig, BrainRunner, readout | ✅ | `BrainConfig`, `BrainRunner::step`, `BrainReadout` |
| Режим `--agent` (stdin/stdout) | ✅ | `run_agent_loop` |
| Режим `--serve` (HTTP POST /step) | ✅ | `run_serve_loop` |
| Тесты (9 шт.): FIB, W, φ, I_SIFS, day/night, readout | ✅ | `cargo test` |
| Запуск cargo при отсутствии в PATH | ✅ | [run_cargo.ps1](run_cargo.ps1), [run_cargo.cmd](run_cargo.cmd) |
| CartPole-клиент (мозг как политика) | ✅ | [run_cartpole_agent.py](run_cartpole_agent.py) |
| Документация: README, BRAIN_CONTRACT, CUDA-путь | ✅ | [README.md](README.md), [BRAIN_CONTRACT.md](BRAIN_CONTRACT.md), [GENESIS_CUDA_SIFS.md](GENESIS_CUDA_SIFS.md) |
| Копия genesis-agi в проекте (наша, для SIFS) | ✅ | [genesis-agi](../genesis-agi) — клон H4V1K-dev/genesis-agi, см. [genesis-agi/SIFS_ORIGIN.md](../genesis-agi/SIFS_ORIGIN.md) |
| Репозитории и эталон для сверки SIFS | ✅ | [docs/REPOS_AND_REFERENCE.md](docs/REPOS_AND_REFERENCE.md) — где genesis-agi и sifs-genesis-brain, как сверять константы и обновлять клоны |

**Проверка работы (обязательно перед коммитом):**

```powershell
cd Genesis
.\run_cargo.ps1 test
.\run_cargo.ps1 run --release   # демо без флагов
.\run_cargo.ps1 run --release -- --agent   # режим агента (завершить вводом Ctrl+Z или пустой строкой при необходимости)
```

---

## Дорожная карта по фазам

### Фаза 1: CPU-мозг «довести до ума» (Genesis/)

- [x] Константы и формулы только из нашей логики (core.py ↔ Rust), без дублирования чисел без ссылки.
- [x] Конфиг мозга: V₀, n_neurons, night_interval, steps_per_observation (BrainConfig + CLI).
- [x] Полный набор тестов: константы, регрессия I_SIFS, day/night smoke, readout.
- [x] Режимы: демо, --agent, --serve.
- [ ] **Опционально:** TOML-конфиг мозга (файл `brain.toml`), загрузка BrainConfig из файла вместо только CLI.
- [ ] **Опционально:** регрессия с Python: скрипт [sifs-genesis-brain/scripts/compare_sifs_i_sifs.py](../sifs-genesis-brain/scripts/compare_sifs_i_sifs.py) запускать из CI или вручную, сверять I_SIFS с допуском.

**Критерий готовности фазы 1:** `.\run_cargo.ps1 test` — все тесты зелёные; `--agent` и `--serve` работают по контракту.

---

### Фаза 2: Встраивание в нашу копию genesis-agi (genesis-node + CUDA + SIFS)

- [x] Сборка нашей копии [genesis-agi](../genesis-agi): `cargo test -p genesis-core` проходит (в т.ч. тесты SIFS). Полная сборка с CUDA: `cargo build --release -p genesis-node -p genesis-compute -p genesis-baker` (или `mock-gpu` без GPU).
- [x] Константы SIFS в genesis-core: модуль [genesis-agi/genesis-core/src/sifs.rs](../genesis-agi/genesis-core/src/sifs.rs) — K, PHI, FIB, W(n), V₀; единый источник для genesis-compute.
- [x] **Заменить/дополнить логику в genesis-compute:** в CUDA-ядре обновления нейронов добавлен расчёт I_SIFS (K, W(n), V_th(n)=V₀/φⁿ), напряжение как proxy «price». Патч в [genesis-agi/genesis-compute/src/cuda/physics.cu](../genesis-agi/genesis-compute/src/cuda/physics.cu): константы SIFS + `sifs_i_sifs_fixed(voltage)`, ток добавляется к мембране в UpdateNeurons.
- [ ] S-Sharding по S-координате в конфиге/baker: уровни 0..9 (FIB).
- [x] Запуск полного стека: baker → node (SensoryCortex, MotorCortex) → Python-клиент CartPole. Baking проверен; инструкция для Windows: [genesis-agi/examples/cartpole/RUN_WINDOWS.md](../genesis-agi/examples/cartpole/RUN_WINDOWS.md).

**Критерий готовности фазы 2:** CartPole E2E идёт через genesis-node с нашими константами и формулой I_SIFS на GPU. ✅ Готово к запуску (3 терминала).

---

### Фаза 3: Сервис и интеграция

- [ ] Мозг как сервис: выбор между UDP (Fast Path genesis-agi) и HTTP (текущий `--serve`) для продакшена; документировать один основной способ.
- [ ] Python-клиент для удалённого мозга: вызов HTTP POST /step из [sifs_agents](../sifs_agents) (инструмент MCP `brain_query` уже есть; при --serve на другой машине — URL в конфиге).
- [ ] Конфиг мозга (TOML) для genesis-node: секция `[sifs]` (K, PHI, V₀, FIB или ссылка на общий контракт).

**Критерий готовности фазы 3:** агент/MCP может обращаться к мозгу по HTTP (локально или на сервере); конфиг описан.

---

### Фаза 4: Валидация и масштаб

- [ ] Бенчмарки: 1K, 10K, 100K нейронов (CPU); при наличии CUDA — сравнение с отчётом genesis-agi (время Day Phase, память).
- [ ] Числовая идентичность: регрессия I_SIFS Rust vs Python (скрипт compare_sifs_i_sifs.py) в CI или чеклисте релиза.
- [ ] CartPole (или простая торговая задача): воспроизводимость и, при возможности, улучшение результата с нашим ядром.
- [ ] Долгосрочно: масштаб 100K–1M нейронов, опция распределённого узла (см. [GENESIS_CUDA_SIFS.md](GENESIS_CUDA_SIFS.md)).

---

## Правила «нашей логики»

1. **Константы:** K, PHI, FIB, W(n) — только из [core.py](../sifs_ft/strategies/modules/core.py). В Rust — те же значения, с комментарием «синхронно с core.py и BRAIN_CONTRACT».
2. **Пороги:** V_th(n) = V₀/φⁿ; V₀ из конфига (по умолчанию 0.02).
3. **I_SIFS:** формула из hybrid (sigma piecewise, fixed-point); регрессия с Python — допуск в fixed-point (например ≤5).
4. **Никакого дублирования формул** без ссылки на контракт или core.py; при изменении в core.py — обновить Rust и BRAIN_CONTRACT.

---

## Ссылки

- **Стратегический план (Genesis + SIFS → полноценный ИИ):** [GENESIS_SIFS_AI_PLAN.md](GENESIS_SIFS_AI_PLAN.md).
- Контракт входа/выхода и константы: [BRAIN_CONTRACT.md](BRAIN_CONTRACT.md).
- Путь к CUDA и что менять в genesis-compute: [GENESIS_CUDA_SIFS.md](GENESIS_CUDA_SIFS.md).
- Расширенный план (репо sifs-genesis-brain, фазы 0–4): [.cursor/plans/реализация_мозга_sifs-genesis_2dafb180.plan.md](../.cursor/plans/реализация_мозга_sifs-genesis_2dafb180.plan.md).
- Эталон констант и формул: [sifs_ft/strategies/modules/core.py](../sifs_ft/strategies/modules/core.py), [documents/theory/SIFS_Theory_Documentation.md](../documents/theory/SIFS_Theory_Documentation.md).
