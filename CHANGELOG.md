# Лог изменений (Genesis + SIFS)

Все заметные изменения плана, контракта, конфигов и решений фиксируются здесь. Формат — [Keep a Changelog](https://keepachangelog.com/ru/1.1.0/); версионирование семантическое при релизах.

---

## [Unreleased]

### Добавлено
- **B.1 обучаемый readout и калибровка (src/):** Моторные пулы + обучаемый readout перенесены в src/ (neuron.rs: weighted_recurrent_sum_in_range, BrainConfig trainable_readout/readout_lr; runner.rs: пулы, веса left/right, trigger_night_phase; io.rs: episode_end; main.rs: CLI --trainable-readout/--readout-lr, тесты test_trainable_readout_*). Cargo test 11 passed, pytest 14 passed. Калибровка: без trainable median 16.0; с --trainable-readout --readout-lr 0.03 — median **17.0**, mean 20.4 (стабильно). run_cartpole_agent.py и run_b1_calibration.py поддерживают --trainable-readout и --readout-lr.
- **B.1 моторные пулы (п.6) и анализ разрыва 11 vs 71:** Добавлен [experiments/B1_ANALYSIS_GAP_71.md](experiments/B1_ANALYSIS_GAP_71.md) — почему медиана ~11, что значит цель 71, приоритеты (моторные пулы, частота Night). Реализованы моторные пулы в sifs_genesis_hybrid.rs: два readout (left/right половинки), action = argmax; weighted_recurrent_sum_in_range для пулов. Калибровка: пулы не дали прироста (11.0); night=50/30 ухудшают (10.0, 9.5). Вывод: потолок ~11 при фиксированном Hebbian+reward readout; для 71 — обучаемый readout или другая схема.
- **B.1 калибровка (продолжение):** Расширенная сетка 3×3 (steps 4,5,6 × night 175,200,225) с episode_end — только (5,200) даёт median 11.0. Проверка reward_scale 0.5→0.7 в night_phase (рекомендация 4): три прогона — без прироста, оставлен 0.5. Результаты в B1_calibration_results.
- **B.1 рекомендация 3 (night phase по концу эпизода):** В Rust: `BrainRunner::trigger_night_phase()`, в agent loop парсинг `episode_end: true` и вызов после step. В run_cartpole_agent.py при terminated/truncated отправляется запрос с `episode_end: true` и терминальным reward. BRAIN_CONTRACT §3.1 дополнен. Калибровка: сетка 2×2 (steps 5,10 × night 150,200) — лучшая 5/200 (median 11.0, mean 10.7); 3 повторных прогона — стабильно 11.0/10.7. Cargo test 9 passed, pytest 14 passed.
- **B.1 рекомендация 5 (200 эп., медиана по последним 50):** В run_cartpole_agent.py при episodes ≥ 50 выводится «Median reward (last 50)». Прогон 200 эп. (steps=5, night=200, dopamine-shaping, population-coding): median (all) 10.0, median (last 50) 9.0 — прироста от удлинения прогона нет. Результаты в B1_calibration_results.md и B1_status.md. Q.1: pytest 14 passed.
- **B.1 консультация MCP и inertia:** Рекомендации по шагам к медиане ≥71 (population coding, inertia, night по эпизоду, reward_scale, больше эпизодов, моторные пулы) зафиксированы в [experiments/B1_NEXT_RECOMMENDATIONS.md](experiments/B1_NEXT_RECOMMENDATIONS.md). Реализована inertia по рангу веса в night_phase_plasticity (16 уровней, сильные связи пластичнее не меняются). 50 эп. steps=5 night=200: median 12.0, mean 11.4.
- **Пластичность по SIFS (B.1):** В `night_phase_plasticity` (sifs_genesis_hybrid.rs) eta и дельта синапсов зависят от s_level и W(s_level): мелкий масштаб (низкий s_level) — пластичнее, крупный — стабильнее; вклад W(n) уменьшает изменение весов у крупномасштабных нейронов. Формулы ядра (K, φ, FIB, W(n)) не менялись. Cargo test 9 passed; 50 эп. с dopamine shaping: median 10.0, mean 10.8. B1_calibration_results, B1_status обновлены.
- **Dopamine shaping и сетка с ним:** В run_cartpole_agent.py — dopamine_shaped() по образцу genesis-agi, флаг --dopamine-shaping; run_b1_calibration.py поддерживает --dopamine-shaping. Сетка steps 8,10,12 × night 150,200,250 с shaping: лучшая ячейка 10/200 (mean 10.8). Результаты в B1_calibration_results.
- **NotebookLM: экспорт 21 источника:** Созданы [documents/NOTEBOOKLM_EXPORT_SOURCES.md](../documents/NOTEBOOKLM_EXPORT_SOURCES.md) (процедура экспорта через MCP) и [documents/notebooklm_sources/00_INDEX.md](../documents/notebooklm_sources/00_INDEX.md) (индекс). MCP вернул «Authentication expired» — нужен `nlm login`, затем повторить запрос на выгрузку.
- **Калибровка B.1 раунд 2:** Проверены BETA=0.45/0.5, eta=0.009/0.01 и activity_delta 0.15 (вместо 0.1). Во всех случаях медиана 10.0; mean 10.5 при (0.45,0.009) и (0.5,0.01), 10.4 при activity_delta 0.15. Оставлены BETA=0.4, eta=0.008, activity_delta 0.1. Результаты в B1_calibration_results (секция «Калибровка B.1 раунд 2»).
- **Повторный запуск B.1 (цель медиана ≥71):** Выполнен прогон 50 эп. seed 42 (steps=10, night=200, BETA=0.4, eta=0.008). Результат: медиана 10.0, mean 10.5. Цель ≥71 не достигнута. Зафиксировано в B1_calibration_results (секция «Повторный запуск B.1») и B1_status.
- **Референс B.1 проверен, результаты внесены:** Запущены 2 ноды genesis-agi (SensoryCortex, MotorCortex), выполнен `cartpole_client.py --seed 42 --episodes 50`. Результат: медиана **9.0**, mean **9.4** (коммит f14494c). Таблица в B1_genesis_agi_reference.md заполнена; наш CPU даёт медиану 10 (выше референса). Цель ≥71 не достигнута.
- **Чеклист B.1 применён:** В genesis-agi/examples/cartpole/cartpole_client.py убраны emoji и Unicode-стрелки для Windows (cp1251). Клиент запускается; референс-прогон не завершён (ноды не запущены). В B1_genesis_agi_reference.md зафиксирован статус: поднять ноды → выполнить команду → заполнить таблицу результатов.
- **Референс B.1 — чеклист для применения:** В B1_genesis_agi_reference.md добавлен блок «Чеклист для применения» (6 шагов). Скрипт scripts/print_b1_reference_steps.py выводит чеклист в терминал (`python scripts/print_b1_reference_steps.py` из Genesis/).
- **План: сетка steps×night при BETA=0.4, eta=0.008:** 3×3 (steps 10,15,20 × night 100,200,300), 50 эп. seed 42. Лучшая ячейка 10/200 (median 10.0, mean 10.5). Результаты в B1_calibration_results.md. NEXT_PLAN и B1_genesis_agi_reference обновлены: следующий шаг — референс genesis-agi (ручной прогон).
- **Продолжай B.1:** Калибровка reward_scale 0.5→0.8 — mean 10.3 (хуже 10.5), оставлен 0.5. Зафиксировано в sifs_genesis_hybrid.rs и B1_calibration_results.
- **eta: почему не 0.00008:** В Q16.16 малые eta (1e-4 и меньше) дают обнуление hebbian_delta после сдвига; в коде и B1_status зафиксировано обоснование диапазона 0.005–0.01.
- **eta 0.003 → 0.008:** Крупнее шаг Hebbian-пластичности, меньше потерь на округлении в fixed-point (Q16.16). 50 эп. seed 42 — медиана 10.0, mean 10.5 (без регрессии). B1_status, B1_calibration_results, HOW_TO_CONTINUE обновлены.
- **Оптимизация BETA и eta (backtester/сетка):** Сетка BETA∈{0.35, 0.4, 0.45}, eta∈{0.002, 0.003, 0.004} — 4 точки, 50 эп. seed 42. Во всех медиана 10.0; лучший mean 10.5 при (0.4, 0.003) и др. Оставлены BETA=0.4, eta=0.003. Результаты в B1_calibration_results.md (секция «Сетка оптимизации BETA × eta»).
- **Продолжай B.1:** Проверена калибровка BETA=0.5, eta=0.005 — медиана 10.0 (без прироста). Оставлены лучшие 0.4/0.003; эксперимент зафиксирован в B1_calibration_results.
- **B.1 калибровка гиперпараметров:** BETA_RECURRENT 0.2→0.4, eta 0.001→0.003 в sifs_genesis_hybrid.rs; медиана 9→10, mean 9.4→10.5 (50 эп., seed 42), воспроизводимо. B1_status, B1_calibration_results, HOW_TO_CONTINUE обновлены.
- **Продолжай:** Q.1 (pytest 14 passed). Калибровка после B.1.опц (steps 15/20, night 200) — медиана 9.0. HOW_TO_CONTINUE и B1_status обновлены: B.1.опц сделан, следующий шаг — референс genesis-agi или калибровка гиперпараметров.
- **B.1.опц реализовано (обучение весов дендритов):** В sifs_genesis_hybrid.rs — поле `synaptic_weights`, граф связей (кольцо, 5 исходящих на нейрон), обновление весов в Night Phase (Hebbian + reward_scale), учёт весов в readout (score = population + 0.2×recurrent). Cargo test 9 passed, pytest 14 passed, CartPole 50 эп. медиана 9.0. B1_status обновлён (раздел B.1.опц).
- **Продолжай / B.1.опц:** Q.1 (pytest 14 passed). В [experiments/B1_status.md](experiments/B1_status.md) добавлен подраздел «B.1.опц — следующие шаги»: указание на `night_phase_plasticity` в sifs_genesis_hybrid.rs и что нужно для обучения весов дендритов (массив весов по рёбрам, применение Hebbian-дельта, использование в readout).
- **B.1 базовая линия:** повторный прогон CartPole 50 эп. (сид 42, steps=10, night=200) — медиана 9.0, mean 9.4 (совпадает с базовой линией). HOW_TO_CONTINUE обновлён: текущий фокус — Q.1, B.1 (референс genesis-agi или B.1.опц), опции D.3/sifs_agents.
- **D.1 закрыт в §13:** в GENESIS_SIFS_AI_PLAN.md §13 добавлен пункт D.1 [x] (бенчмарк GPU 100K и 1M). HOW_TO_CONTINUE обновлён: текущий фокус — D.1 выполнен, открытый B.1 (медиана ≥71).
- **D.1.3 бенчмарк 1M (genesis-agi):** конфиг examples/bench_1m (зона Bench1M, ~1M нейронов); baker 1_002_016 нейронов; нода запускается, TPS ~333 (~3 ms/step). Результаты в B4_benchmarks.md. Скрипт run_bench_node_timed.ps1 исправлен ($pid → $procId).
- **D.1.2 бенчмарк 100K (genesis-agi):** добавлен конфиг examples/bench_100k (одна зона Bench100K, ~100K нейронов); baker успешно печёт 100960 нейронов; нода запускается, TPS ~1733 (~0.58 ms/step). Результаты в [experiments/B4_benchmarks.md](experiments/B4_benchmarks.md) (секция GPU). NEXT_PLAN текущий шаг — D.1.3.
- **D.1.1 сборка genesis-agi под GPU:** в репо genesis-agi выполнена сборка с CUDA (CUDA_ARCH=sm_89): бинарники genesis-node, genesis-baker собираются без ошибок; RUN_WINDOWS.md и README проверены.
- **Выполнение NEXT_PLAN (B.1.3–B.1.5):** в genesis-agi cartpole_client.py добавлены --seed, --episodes, вывод Mean/Median в конце; B1_genesis_agi_reference обновлён (команда запуска). Расширенная калибровка B.1.4: сетка 3×3 (50 эп.) выполнена агентом, результат в B1_calibration_results.md (медиана 9.0). B.1.5 A/B задокументирован в B1_status. NEXT_PLAN обновлён (текущий шаг B.1.6/D.1.1).
- **NEXT_PLAN.md:** полный пошаговый план «что дальше» (B.1.3–B.1.6, D.1.1–D.1.3, Q.1–Q.3); при «Продолжай по NEXT_PLAN» агент выполняет следующий шаг и идёт дальше без вопроса «продолжать?». HOW_TO_CONTINUE дополнен ссылкой и формулировкой.
- **B.1 тесты и референс:** добавлены tests/test_b1_calibration.py (парсинг медианы/mean, run_one с моком subprocess); в B1_genesis_agi_reference заполнены BATCH_TICKS=100 и формула dopamine из genesis-agi/cartpole_client.py; отмечено, что для 50 эп. с сидом 42 в genesis-agi при необходимости добавить в скрипт флаги.
- **B.1 калибровка выполнена и проверена:** run_cartpole_agent (2 эп.) и run_b1_calibration (2 эп., 50 эп., сетка 2×2) запущены; результат сетки записан в experiments/B1_calibration_results.md (медиана 9.0 при всех комбинациях). В скрипт калибровки добавлен --episodes для быстрых тестов.
- **B.1 возврат к калибровке:** в run_cartpole_agent.py добавлены `--night` и вывод медианы; скрипт [scripts/run_b1_calibration.py](scripts/run_b1_calibration.py) — 50 эп. сид 42, один прогон или сетка (--grid) steps×night с выводом таблицы; B1_status и B1_genesis_agi_reference обновлены (инструкции запуска, калибровка).
- **D.3 режим мозга в sifs_ft (вариант A):** в sifs_ft добавлены [strategies/modules/genesis_brain.py](../sifs_ft/strategies/modules/genesis_brain.py) (bar_to_observation, brain_step — контракт как в Genesis) и опциональный вызов в [entry.py](../sifs_ft/strategies/modules/entry.py): при use_genesis_brain=True решение по последней свече отдаётся мозгу по HTTP; fallback при ошибке — SIFS. В Sifs.py из strategy_settings читаются use_genesis_brain, genesis_brain_url, genesis_brain_timeout_sec. §13 плана: D.3 отмечен выполненным.
- **Тесты D.3:** [Genesis/tests/test_trading_brain_loop.py](tests/test_trading_brain_loop.py) — юнит-тесты для pnl_to_reward, bar_to_observation, load_ohlcv (§9 фильтр по датам), smoke run_backtest с моком brain_step. Запуск: `python -m pytest Genesis/tests/ -v` из корня проекта или `pytest tests/ -v` из Genesis.
- **D.3 таблица решений для сравнения с SIFS-only:** в run_d3_backtest.py добавлен `--decisions_csv` (bar_index, date, action, action_name). Шаг 2 (вариант B) в D3_NEXT_STEPS отмечен выполненным.
- **D.3 скрипт бэктеста по §9:** [scripts/run_d3_backtest.py](scripts/run_d3_backtest.py) — прогон по .feather, obs→мозг→действие, отчёт в stderr, JSON summary в stdout, опционально --equity_csv. D3_NEXT_STEPS шаг 3 отмечен выполненным.
- **D.3 конфиг §9 в trading_brain_loop:** добавлены `--start_date` и `--end_date` (ISO) для train/val по времени; в summary выводятся first_bar_date, last_bar_date, pair, timeframe. D3_NEXT_STEPS шаг 1 отмечен выполненным.
- **D.3 следующие шаги (агенты):** [docs/D3_NEXT_STEPS.md](docs/D3_NEXT_STEPS.md) — приоритет D.3 (интеграция sifs_ft + бэктест по §9), точки интеграции (entry.py, trading_brain_loop, brain_http_client), три шага (конфиг §9, точка входа «мозг как политика», скрипт бэктеста). HOW_TO_CONTINUE обновлён: текущий фокус — D.3 по D3_NEXT_STEPS.
- **D.3 сводка прогона:** в конце trading_brain_loop выводится сводка: число баров, счётчики hold/long/short, cumulative_pnl_pct (бумажный PnL в %); одна строка JSON с полем summary в stdout для пайпов.
- **D.3 reward в торговом цикле:** в trading_brain_loop.py добавлен расчёт бумажного PnL за шаг (long/short/hold) и передача нормированного reward в мозг: reward = clip(pnl_pct / reward_scale, -1, 1) по TRADING_REWARD_AND_DATA §6; параметр --reward_scale (по умолчанию 0.02); в лог добавлено поле reward_sent.
- **D.3 минимальный торговый цикл:** [Genesis/scripts/trading_brain_loop.py](scripts/trading_brain_loop.py) — чтение OHLCV из .feather (sifs_ft/data/binance), построение вектора наблюдения (OBS_SIZE=10), вызов мозга по HTTP (brain_http_client.brain_step), логирование (t, action, readout); fallback pause/sifs_only/fail при недоступности мозга; без look-ahead. В docs/CLIENT_INTEGRATION.md добавлен сценарий (3) D.3; HOW_TO_CONTINUE обновлён.
- **B.1 референс:** [Genesis/experiments/B1_genesis_agi_reference.md](experiments/B1_genesis_agi_reference.md) — шаблон для фиксации параметров и результатов CartPole 50 эп. (сид 42) в genesis-agi; в B1_status добавлена секция «Референс genesis-agi»; HOW_TO_CONTINUE обновлён (текущий фокус: заполнить референс, калибровка --steps/--night, либо фаза D.1/D.3). Продолжение плана с привлечением агентов (explore, generalPurpose).
- **B.1 отложен до после C:** рекомендация агентов (MCP) зафиксирована в experiments/B1_status.md; при возврате к B.1 — референс genesis-agi, --steps/--night, dopamine shaping, пластичность весов.
- **C.1:** [Genesis/docs/CLIENT_INTEGRATION.md](docs/CLIENT_INTEGRATION.md) — runbook: latency budget, fallback (pause/sifs_only/fail), пример конфига клиента (YAML/TOML), когда HTTP vs UDP. Ссылка из BRAIN_CONTRACT §3.4 и из плана §13.
- **C.2:** [Genesis/scripts/brain_http_client.py](scripts/brain_http_client.py) — HTTP-клиент мозга (POST /step, input + reward, timeout). В docs/CLIENT_INTEGRATION.md добавлен пример вызова (Python). Сценарии: stdio (run_cartpole_agent) + HTTP (brain_http_client).
- **C.1:** отмечен выполненным в §13 (документ + пример конфига готовы).
- **§5:** [Genesis/docs/SIFS_AGENTS_INTEGRATION.md](docs/SIFS_AGENTS_INTEGRATION.md) — зафиксирован вариант A (дополнение), места реализации, B/C на будущее.
- **§7:** [Genesis/docs/CHECKPOINTING_WEIGHTS.md](docs/CHECKPOINTING_WEIGHTS.md) — базовая схема: когда сохранять, формат, откат при деградации, A/B весов; связь с D.2.
- **D.2:** [Genesis/docs/CATASTROPHIC_FORGETTING.md](docs/CATASTROPHIC_FORGETTING.md) — выбран механизм (b) отдельные чекпоинты на задачу с переключением; критерий деградации; реализация переключения — при фазах C–D.
- **§6 и §9:** [Genesis/docs/TRADING_REWARD_AND_DATA.md](docs/TRADING_REWARD_AND_DATA.md)
- **Репозитории и эталон:** [Genesis/docs/REPOS_AND_REFERENCE.md](docs/REPOS_AND_REFERENCE.md). Репо m0rfy/sifs-genesis-brain создан и запушен; из документа убран раздел «Создание репо», таблица приведена в соответствие с текущим состоянием. — где лежат genesis-agi и sifs-genesis-brain, эталон констант (core.py), как сверять SIFS и обновлять клоны. — reward для торговой среды (PnL за шаг, нормировка, доставка в мозг); данные (источник feather, train/val по времени, no look-ahead, out-of-sample). BRAIN_CONTRACT §6.2 обновлён.
- **B.2 Доставка reward в мозг:** бинарник принимает опциональное поле `"reward"` в JSON (stdio: `{"input": [...], "reward": float}`, HTTP: то же). `last_reward` передаётся в Night Phase; пластичность s_coordinate масштабируется (1 + reward×0.5). run_cartpole_agent.py шлёт reward от предыдущего шага. BRAIN_CONTRACT §3.1 обновлён; B1_status обновлён (блокер ослаблен).
- **Типичные ошибки:** в README добавлена секция «Типичные ошибки (CartPole, протокол)»: пересборка `cargo build --release` после изменений в Rust; протокол 1:1 (всегда отвечать на запрос); поддержка обоих форматов stdin (массив и объект). Чтобы не повторять сбои `invalid type: map, expected a sequence` и OSError при записи в stdin.
- (Остальные будущие изменения — перед релизом переносятся в версию ниже.)

---

## 2025-03-07 (продолжение плана, фаза B)

### Эксперименты и бенчмарки
- **B.1:** [experiments/B1_status.md](experiments/B1_status.md) — базовая линия (медиана 10), блокер (нет доставки reward в мозг), следующие шаги для медианы ≥71.
- **B.4:** [experiments/B4_benchmarks.md](experiments/B4_benchmarks.md) — CPU: 1K, 10K, 100K нейронов (Day/Night ms, память, spikes/sec). В коде бенчмарк: размеры 1K, 10K, 100K.

### BRAIN_CONTRACT
- **§1.1 (B.3):** Night Phase перераспределяет нейроны по шардам по s_level (night_phase_plasticity).

### План
- §13: B.3, B.4 отмечены выполненными; B.1 — ссылка на B1_status.md.

---

## 2025-03-07

### BRAIN_CONTRACT
- **§3.4 Latency и сервис:** latency budget (&lt;500 ms для 15m, &lt;100 ms для CartPole), fallback при недоступности мозга (пауза / SIFS-only / отказ), когда использовать HTTP vs UDP.
- **§3.5 Роль мозга и агентов:** варианты A/B/C (дополнение, замена в узком слое, разделение зон); ссылка на §5 плана.
- **§6 Reward для сред:** CartPole — формула (+1 за шаг, терминал без отдельного штрафа), место в пайпе (сейчас reward не передаётся в мозг; при добавлении — GSIO/JSON), воспроизводимость B.1 (сид 42, гиперпараметры). Заглушка §6.2 для торговой среды (D.3).
- **Правило констант:** без зелёного `scripts/compare_sifs.py` не мержить изменения в core.py / sifs_genesis_hybrid / genesis-core sifs.
- **Порог V₀:** уточнено, что CPU-бинарник пока не читает brain.toml; эталон конфига — brain.toml, дефолты в коде должны совпадать.

### План (GENESIS_SIFS_AI_PLAN)
- Ссылка на CHANGELOG в §12 «Связь с другими документами».
- Чеклист §13: отмечены выполненные A.2, A.3, A.4 (если не были отмечены ранее).

### Консультации агентов (MCP)
- Зафиксированы рекомендации: порядок B.1→B.2→B.3/B.4; заложить latency/fallback/роль мозга в контракт; зафиксировать формулу reward и место доставки; один источник констант, один контракт, один формат конфига; правило compare_sifs; чтение brain.toml в hybrid или явная документация.

---

## Как вести лог

- **Добавлено** — новые разделы контракта, пункты плана, конфиги, правила.
- **Изменено** — правки формул, порогов, форматов, приоритетов.
- **Устарело** — снятые с актуальности решения (с указанием замены).
- Дату ставить в формате ГГГГ-ММ-ДД. Крупные релизы выносить в заголовок `## [X.Y.Z]` при необходимости.
