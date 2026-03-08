# D.3 Следующие шаги: интеграция с sifs_ft и бэктест по §9

Сводка рекомендаций агентов (explore + generalPurpose) при продолжении плана. Актуально после готовности минимального цикла [trading_brain_loop.py](../scripts/trading_brain_loop.py).

---

## Приоритет (рекомендация generalPurpose)

| Порядок | Задача | Комментарий |
|--------|--------|-------------|
| **1** | **D.3** — интеграция sifs_ft + мозг + бэктест по §9 | Закрывает критерий фазы D; цикл уже есть в trading_brain_loop.py. |
| 2 | B.1 (по желанию) — референс genesis-agi (50 эп., сид 42), калибровка CPU | Риск отвлечения; медиана 71+ маловероятна без обучения весов дендритов. |
| 3 | D.1 (по желанию) — бенчмарк GPU 100K–1M в genesis-agi | После рабочего D.3. |

---

## Точки интеграции (по выводу explore)

### sifs_ft

| Файл | Назначение для D.3 |
|------|---------------------|
| `sifs_ft/strategies/modules/entry.py` | Логика входа (66–84): место для опционального «режима мозга» (наблюдение → brain_step → enter_long/enter_short). |
| `sifs_ft/strategies/modules/exit_logic.py` | Выход: при необходимости — сигнал мозга в custom_exit. |
| `sifs_ft/data/binance/` | Общий источник .feather с trading_brain_loop (имена: `XRP_USDT-15m-spot.feather` и т.д.). |
| `sifs_ft/BACKTEST_RESULTS.md` | Команды и конфиги бэктеста; §9 в смысле «как гонять». |

### Genesis

| Файл | Назначение для D.3 |
|------|---------------------|
| `Genesis/scripts/trading_brain_loop.py` | Готовый цикл: бар → obs → brain HTTP → действие; `bar_to_observation`, reward по §6. |
| `Genesis/scripts/brain_http_client.py` | `brain_step(url, observation, reward, timeout_sec)` — вызывать из sifs_ft или из скрипта бэктеста. |
| [TRADING_REWARD_AND_DATA.md](TRADING_REWARD_AND_DATA.md) | §6 reward, §9 данные (источник, train/val, no look-ahead). |

---

## 3 конкретных шага для D.3

1. **Конфиг и данные §9** ✅  
   В `trading_brain_loop.py`: путь к данным (--data, по умолчанию `sifs_ft/data/binance/`), `--pair`, `--timeframe` уже были; добавлены **`--start_date`** и **`--end_date`** (ISO) для обрезки по датам (train/val по §9). В сводке JSON выводятся `first_bar_date`, `last_bar_date`, `pair`, `timeframe` для воспроизводимости.

2. **Точка входа «мозг как политика»** ✅ (A и B)  
   **Вариант B:** `run_d3_backtest.py` + `--decisions_csv`. **Вариант A:** в sifs_ft [strategies/modules/genesis_brain.py](../../sifs_ft/strategies/modules/genesis_brain.py) (bar_to_observation, brain_step) и в [entry.py](../../sifs_ft/strategies/modules/entry.py) при `use_genesis_brain=True` решение по последней свече отдаётся мозгу; конфиг: strategy_settings `use_genesis_brain`, `genesis_brain_url`, `genesis_brain_timeout_sec`.

3. **Скрипт бэктеста по §9** ✅  
   [scripts/run_d3_backtest.py](../scripts/run_d3_backtest.py): те же .feather и логика (load_ohlcv, bar_to_observation, brain_step), без look-ahead; вывод одной строки JSON summary и отчёта в stderr (pair, timeframe, date range, bars, hold/long/short, cumulative_pnl_pct); опционально `--equity_csv` для кривой equity. Запуск: мозг `--serve`, затем `python scripts/run_d3_backtest.py [--start_date ...] [--end_date ...] [--equity_csv out.csv]`.

Единый формат наблюдения: в sifs_ft использовать ту же схему, что в `trading_brain_loop.py` (OBS_SIZE=10, порядок полей по BRAIN_CONTRACT), чтобы бэктест и цикл Genesis были на одной схеме.

---

## Ссылки

- План D.3 и §9: [GENESIS_SIFS_AI_PLAN.md](../GENESIS_SIFS_AI_PLAN.md)
- Данные и reward: [TRADING_REWARD_AND_DATA.md](TRADING_REWARD_AND_DATA.md)
- Клиент мозга: [CLIENT_INTEGRATION.md](CLIENT_INTEGRATION.md)
- Как продолжить: [HOW_TO_CONTINUE.md](../HOW_TO_CONTINUE.md)
