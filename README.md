# sifs-genesis-brain

Нейроморфное ядро **мозга SIFS-Genesis**: математика SIFS (K, φ, W(n), S-координата) поверх архитектуры [genesis-agi](https://github.com/H4V1K-dev/genesis-agi).

- **Репозиторий:** https://github.com/m0rfy/sifs-genesis-brain (доступы — m0rfy)
- **Константы SIFS:** единственный источник — [core.py](../sifs_ft/strategies/modules/core.py) в проекте SIFS; в Rust — синхронно с ним (см. [docs/BRAIN_CONTRACT.md](docs/BRAIN_CONTRACT.md)).
- **План реализации:** см. внутреннюю документацию проекта (вне этого репо).

## Текущий статус

- Фаза 0: контракт и экспорт констант — готовы (локально).
- П.2: крейт `sifs-genesis-core` (K, PHI, W(n), I_SIFS, Day/Night на rayon), бинарник, тесты. **genesis-agi** — submodule в `deps/genesis-agi`, опционально feature `genesis`.
- П.3: **Day Phase** — пайплайн из 6 шагов в порядке Genesis; см. [docs/DAY_PHASE_GENESIS_MAPPING.md](docs/DAY_PHASE_GENESIS_MAPPING.md).
- П.4: **S-Sharding и Ghost Axons** — см. [docs/S_SHARDING_AND_GHOST.md](docs/S_SHARDING_AND_GHOST.md).
- П.5: **I/O и API** — см. [docs/BRAIN_CONFIG.md](docs/BRAIN_CONFIG.md), [clients/python/brain_client.py](clients/python/brain_client.py).
- П.8: **Бенчмарки и регрессия** — [docs/BENCHMARKS.md](docs/BENCHMARKS.md).
- **Night Phase:** Hebbian ΔS и resharding по S в `night_phase_plasticity`; полный Sort & Prune / Cone Tracing — при интеграции с Genesis; [docs/NIGHT_PHASE.md](docs/NIGHT_PHASE.md).
- **Агент Genesis + SIFS:** бинарник `sifs-genesis-agent` (наблюдение → мозг → действие), демо CartPole: [clients/python/run_cartpole_sifs_agent.py](clients/python/run_cartpole_sifs_agent.py) — см. раздел «Агент и CartPole» ниже.
- **Следующий шаг:** создать репозиторий на GitHub [m0rfy/sifs-genesis-brain](https://github.com/m0rfy/sifs-genesis-brain), добавить `origin`, сделать первый push (и `git push --recurse-submodules=on-demand` при обновлении submodule).

## Submodule genesis-agi

- **Клонирование с submodule:**  
  `git clone --recurse-submodules https://github.com/m0rfy/sifs-genesis-brain`
- **Если репо уже склонирован без submodule:**  
  `git submodule update --init --recursive`
- **Сборка с типами Genesis:** `cargo build -p sifs-genesis-core --features genesis` (path dependency на `genesis-core`). В submodule `deps/genesis-agi/genesis-core/Cargo.toml` сделаны локальные правки для сборки как path dep (явные version, edition, deps); при обновлении submodule может потребоваться повторить правки.

## Структура

```
sifs-genesis-brain/
├── README.md
├── Cargo.toml              # workspace (sifs-genesis-core)
├── .gitmodules             # deps/genesis-agi
├── deps/
│   └── genesis-agi/        # submodule (H4V1K-dev/genesis-agi)
├── crates/
│   └── sifs-genesis-core/  # SIFS-логика, Day/Night (rayon); бинарники: core + agent
├── docs/
│   └── BRAIN_CONTRACT.md
├── scripts/
│   └── export_sifs_constants.py
└── sifs_constants.json
```

## Агент и CartPole (Genesis + SIFS как целый агент)

Один агент с «мозгом» SIFS: наблюдение → мозг (Day Phase + readout) → действие.

1. **Сборка агента**
   ```bash
   cd sifs-genesis-brain
   cargo build --release
   ```
   Появятся бинарники: `target/release/sifs-genesis-core`, `target/release/sifs-genesis-agent`.

2. **Запуск демо CartPole** (мозг как политика: score → left/right)
   ```bash
   pip install gymnasium
   python clients/python/run_cartpole_sifs_agent.py [--neurons 1000] [--steps 1] [--episodes 5]
   ```
   Скрипт запускает `sifs-genesis-agent`, передаёт нормализованное наблюдение (4 float) на каждый шаг среды, получает действие 0 или 1 и выполняет `env.step(action)`. В консоль выводится награда за эпизод.

3. **Режим агента вручную**  
   Запуск `sifs-genesis-agent --neurons N --steps K`: в stdin — по одной строке JSON (массив float), в stdout — JSON `{"readout": {...}, "action": 0|1}` на каждую строку.

## Экспорт констант для Rust

```bash
# из корня проекта Projects (или из sifs-genesis-brain с правильным путём к core.py)
python scripts/export_sifs_constants.py
```

Генерирует `sifs_constants.json`. Rust-код должен читать эти значения при сборке или использовать их как эталон для тестов числовой идентичности с Python.
