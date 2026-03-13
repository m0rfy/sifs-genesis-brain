# SIFS Brain (Genesis)

Этот репозиторий — каноническое дерево **SIFS-Genesis** (CPU-мозг). **Genesis-AGI — отдельный независимый проект** ([H4V1K-dev/genesis-agi](https://github.com/H4V1K-dev/genesis-agi)); мы не его авторы. Мы взяли у них разработки Genesis и соединили с **нашей теорией SIFS** — так получился этот проект. Клонируйте и собирайте отсюда.

Полноценный мозг с ядром SIFS (K, φ, W(n), FIB) и двухфазным циклом Day/Night. Один бинарник для локального запуска или работы на сервере. **Один мозг — несколько сред:** CartPole и торговый мини (OHLCV → long/short/hold) — первые две среды для валидации и продакшена; цель фазы D — один агент для ≥2 сред без деградации.

- **План и дорожная карта:** [ROADMAP.md](ROADMAP.md) — фазы, чеклисты, наша логика, проверки.
- **Контракт:** [BRAIN_CONTRACT.md](BRAIN_CONTRACT.md), константы синхронны с [sifs_ft/strategies/modules/core.py](../sifs_ft/strategies/modules/core.py). Интеграция клиентов (fallback, latency): [docs/CLIENT_INTEGRATION.md](docs/CLIENT_INTEGRATION.md).
- **Ядро:** константы K, PHI, FIB, W(n), пороги V_th(n) = V₀/φⁿ, I_SIFS в fixed-point, Day Phase (LIF + SIFS), Night Phase (Hebbian ΔS, resharding). S-Sharding: каждый шард (SIFSShard) имеет явный **s_level ∈ [0..9]** — см. [BRAIN_CONTRACT.md](BRAIN_CONTRACT.md) §1.1.

## Основа (Foundations)

- **Теория SIFS:** математическая основа — [SIFS Theory (Spacetime)](https://github.com/m0rfy/SIFS-Theory-Core): константы K, φ, FIB и варпинг W(n) заданы теорией (Scale-Invariant Fractal System), а не подобраны под данные.
- **Движок Genesis:** Genesis-AGI — отдельный проект (H4V1K-dev). Мы взяли у них движок Genesis и объединили с нашей теорией SIFS (ядро K, φ, FIB, W, двухфазный цикл Day/Night). Данный репозиторий — CPU-гибрид и опционально CUDA через submodule [genesis-agi](https://github.com/H4V1K-dev/genesis-agi).

## Установка окружения (Rust, CUDA, VS Build Tools)

Для CPU-мозга достаточно **Rust**. Для genesis-agi с CUDA (наша копия) нужны ещё **CUDA Toolkit** и **Visual Studio Build Tools (C++)**. RTX 4090 — compute capability sm_89.

- Пошаговая установка: [SETUP_WINDOWS.md](SETUP_WINDOWS.md).
- Проверка и подсказки: `.\install_requirements.ps1`.

## Сборка и тесты

Если `cargo` не в PATH (например, в новой PowerShell), используйте скрипты:

```powershell
# в корне репо (sifs-genesis-brain)
.\run_cargo.ps1 build --release
.\run_cargo.ps1 test
```

Или из cmd: `run_cargo.cmd test`. Скрипты добавляют в PATH `%USERPROFILE%\.cargo\bin`.

Обычный запуск (если cargo в PATH):

```bash
cargo build --release
cargo test
```

Все тесты должны проходить: константы (FIB, W, φ), регрессия I_SIFS, Day/Night smoke, формат readout.

**Тесты Python:** D.3 (`trading_brain_loop`, `run_d3_backtest`) и B.1 (`run_b1_calibration`):

```bash
# в корне репо
python -m pytest tests/ -v
```

## Режимы запуска

### 1. Демо (по умолчанию)

Запуск без аргументов — прогон встроенных тестов и бенчмарков (SIFS vs simple threshold, сравнение архитектур).

```bash
cargo run --release
```

### 2. Агент (локально, stdin/stdout)

Чтение наблюдений из stdin (по одной JSON-строке с массивом float), вывод в stdout: `{"readout": {...}, "action": 0|1}`.

```bash
cargo run --release -- --agent [--neurons 1000] [--steps 1] [--night 500] [--v0 0.02]
# пример: echo '[0.1,0.2,0.15,0.0]' | cargo run --release -- --agent
```

Используется в [run_cartpole_agent.py](run_cartpole_agent.py): мозг как политика для CartPole.

### 3. HTTP-сервер (локально или на сервере)

Сервер принимает `POST /step` с телом `{"input": [float, ...]}` и возвращает JSON с readout и action. Удобно для удалённого вызова (другой хост, другой процесс).

```bash
# локально
cargo run --release -- --serve 127.0.0.1:3030

# на сервере (доступ с других машин)
cargo run --release -- --serve 0.0.0.0:3030 --neurons 2000
```

Пример запроса:

```bash
curl -X POST http://127.0.0.1:3030/step -H "Content-Type: application/json" -d '{"input":[0.01,0.02,0.015]}'
```

Ответ: `{"readout":{"spike_count_total":...,"score":...,"time_step":...},"action":0}` или `"action":1`.

Python-клиент (фаза C.2): [scripts/brain_http_client.py](scripts/brain_http_client.py), пример в [docs/CLIENT_INTEGRATION.md](docs/CLIENT_INTEGRATION.md).

## Опции конфигурации

| Опция | По умолчанию | Описание |
|-------|--------------|----------|
| `--neurons N` | 1000 | Число нейронов |
| `--steps K` | 1 | Шагов мозга на одно наблюдение |
| `--night M` | 500 | Период Night Phase (каждые M шагов) |
| `--v0 V` | 0.02 | Базовый порог V₀ (BRAIN_CONTRACT §2) |

## CartPole

```bash
pip install gymnasium
python run_cartpole_agent.py [--neurons 1000] [--episodes 5]
```

Скрипт запускает бинарник с `--agent` и передаёт нормализованные наблюдения CartPole; действие 0 = влево, 1 = вправо.

## Использование как «мозг» в своих проектах

- **Локально:** запуск с `--agent` и обмен по stdin/stdout (как в run_cartpole_agent.py) или вызов бинарника с `--input ...` не реализован — только построчный stdin.
- **По сети:** запуск с `--serve 0.0.0.0:PORT`, клиенты шлют POST /step с `{"input": [...]}` и получают readout + action.

Единый контракт входа/выхода описан в [BRAIN_CONTRACT.md](BRAIN_CONTRACT.md).

## Типичные ошибки (CartPole, протокол)

Чтобы не повторять сбои при запуске `run_cartpole_agent.py`:

1. **После любых изменений в Rust** обязательно пересобрать бинарник, который запускает Python: `cargo build --release` в каталоге Genesis. Скрипт ищет `target/release/sifs_genesis_hybrid.exe` (или debug). Если не пересобрать, возможны: `invalid type: map, expected a sequence` (старый бинарник ждёт только массив, а скрипт шлёт объект `{"input": [...], "reward": ...}`) или `OSError: [Errno 22] Invalid argument` при записи в stdin после первого эпизода (процесс уже завершился из‑за ошибки парсинга).
2. **Протокол 1:1:** на каждую строку со стороны клиента бинарник обязан ответить одной строкой (readout + action). Иначе клиент зависает на `readline()`. Бинарник не должен делать `continue` без вывода ответа.
3. **Формат stdin:** поддерживаются оба варианта: массив `[0.1, 0.2, ...]` (reward=0) и объект `{"input": [0.1, ...], "reward": 0.0}`. При невалидной строке бинарник отвечает действием по умолчанию (step с нулями), а не завершает работу.

Подробнее: [BRAIN_CONTRACT.md](BRAIN_CONTRACT.md) §3.1.

## Репозитории и сверка SIFS

- **Этот репозиторий:** [m0rfy/sifs-genesis-brain](https://github.com/m0rfy/sifs-genesis-brain) — канонический репозиторий дерева SIFS-Genesis (CPU-мозг). Genesis-AGI — отдельный проект; мы взяли у них исходники и соединили с нашей теорией SIFS.
- **Оригинальный движок:** [H4V1K-dev/genesis-agi](https://github.com/H4V1K-dev/genesis-agi) — полный стек (CUDA, node, baker); мы используем его как основу и submodule `deps/genesis-agi` для CUDA-пути.

Локальные клоны обоих репо — для сверки констант и протокола. Эталон констант — core.py; как обновлять клоны и настраивать SIFS: [docs/REPOS_AND_REFERENCE.md](docs/REPOS_AND_REFERENCE.md).

## Genesis-Node / CUDA + SIFS

Текущий бинарник — CPU-гибрид. Полноценный стек (genesis-node + genesis-compute на CUDA с ядром SIFS) и шаги по интеграции описаны в [GENESIS_CUDA_SIFS.md](GENESIS_CUDA_SIFS.md).
