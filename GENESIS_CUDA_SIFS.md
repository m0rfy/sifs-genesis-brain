# Genesis-Node / CUDA + SIFS — путь к полноценному стеку

Сейчас в папке **Genesis/** работает **CPU-гибрид** (SIFS ядро + Day/Night на rayon). Полноценный стек — это **genesis-agi** (genesis-node + genesis-compute на CUDA) с встроенным ядром SIFS.

---

## Текущее состояние

| Компонент | Где | Описание |
|-----------|-----|----------|
| **SIFS Brain (CPU)** | `Genesis/sifs_genesis_hybrid.rs` | Один бинарник: тесты, `--agent`, `--serve`. Константы K, φ, FIB, W(n), I_SIFS, Day/Night. |
| **Наша копия genesis-agi** | [genesis-agi](../genesis-agi) | Полная копия H4V1K-dev/genesis-agi для интеграции SIFS. Константы SIFS в [genesis-core/src/sifs.rs](../genesis-agi/genesis-core/src/sifs.rs). |

CPU-мозг не использует genesis-node и не использует CUDA — только наша математика в стиле Genesis (integer physics, Day/Night).

---

## Что нужно для полноценного genesis-node + CUDA + SIFS

1. **Сборка нашей копии**  
   Нужны: Rust, CUDA Toolkit (nvcc), подходящий драйвер, MSVC (Build Tools). Из корня копии [genesis-agi](../genesis-agi):
   ```powershell
   cd c:\Users\m0rfy\Projects\genesis-agi
   $env:CUDA_ARCH = "sm_89"   # для RTX 4090
   cargo build --release -p genesis-node -p genesis-compute -p genesis-baker
   ```
   Для CUDA 13.x + VS Build Tools 18 в [genesis-compute/build.rs](../genesis-agi/genesis-compute/build.rs) добавлен флаг `-allow-unsupported-compiler` (Windows). Без CUDA: `cargo build --release -p genesis-compute --features mock-gpu`. Константы SIFS в [genesis-core/src/sifs.rs](../genesis-agi/genesis-core/src/sifs.rs).
   **Предупреждения при сборке:** сообщения `Compiler family detection failed` от крейта `cc` (nvcc -E на Windows) можно игнорировать — компиляция CUDA завершается успешно, бинарники рабочие.

2. **Встраивание SIFS в genesis-compute** ✅  
   Реализовано в [genesis-compute/src/cuda/physics.cu](../genesis-agi/genesis-compute/src/cuda/physics.cu): константы K, φ, W(n), V_th(n)=V₀/φⁿ; в ядре `cu_update_neurons_kernel` добавлен I_SIFS = Σ W(n)·(V − V_th(n)) (напряжение как proxy «price»), ток входит в обновление мембраны.

3. **Конфиг и контракт**  
   - Порог V₀ и, при необходимости, число уровней S — в TOML конфиге зоны (или в genesis-core), чтобы baker и node использовали те же значения.
   - Формат входа/выхода оставить по спецификации genesis-agi; readout по-прежнему даёт спайки/популяцию — интерпретация «score»/действие может остаться на стороне клиента (как в нашем CPU-агенте).

4. **Лимиты GPU/ПК**  
   Длительные прогоны (бенчмарк 100K, нода): ограничивать TPS и/или мощность карты, чтобы не перегревать. См. [genesis-agi/RESOURCE_LIMITS.md](../genesis-agi/RESOURCE_LIMITS.md) (`--max-tps`, `nvidia-smi -pl`, температура).

5. **Запуск полного стека**  
   По [Quick Start](https://github.com/H4V1K-dev/genesis-agi):
   - `cargo run -p genesis-baker -- --brain config/brains/CartPole/brain.toml`
   - Запуск двух узлов (SensoryCortex, MotorCortex) и Python-клиента.

После внедрения SIFS в compute те же шаги дадут CartPole уже с ядром SIFS на GPU.

---

## Как запускать cargo, если он не в PATH

В PowerShell из папки **Genesis**:

```powershell
.\run_cargo.ps1 test
.\run_cargo.ps1 build --release
.\run_cargo.ps1 run --release
```

Или из cmd:

```cmd
run_cargo.cmd test
run_cargo.cmd build --release
```

Скрипты подставляют в PATH `%USERPROFILE%\.cargo\bin`. Если Rust установлен в другое место — отредактируйте `$cargoBin` в `run_cargo.ps1` или `CARGO_BIN` в `run_cargo.cmd`.

---

## Кратко

- **Сейчас:** полноценный SIFS-мозг на CPU в **Genesis/** (тесты, агент, HTTP). Cargo удобно вызывать через `run_cargo.ps1` / `run_cargo.cmd`.
- **Полный стек:** собрать genesis-agi из `sifs-genesis-brain/deps/genesis-agi`, добавить в genesis-compute (CUDA) расчёт I_SIFS и встроить его в UpdateNeurons, затем запускать node + baker + клиент как в документации genesis-agi.
