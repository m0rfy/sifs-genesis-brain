# План «что дальше» (Genesis + SIFS) — краткая версия для репо

Краткая сводка текущего шага и закрытых блоков. Все ссылки — только внутри этого репо (docs/, experiments/).

**Полный план с командами и всеми шагами:** см. `Genesis/NEXT_PLAN.md` в workspace (вне репо).

---

## Текущий шаг

**Сейчас:** **Поток 2** (интерфейс Фаза 2/3) или опционально верификация run_l5_loop / K.4. Блоки L и K.4 закрыты.

**Следующий:** Поток 2 Фаза 2 или Фаза 3 (фронт: Чат | SIFS | Brain | Настройки | Логи); опционально — прогон run_l5_loop для верификации r > 0.72.

---

## Закрытые блоки

| Блок | Кратко |
|------|--------|
| **B.1** | CartPole медиана ≥71: Python CMA-ES + SIFS median 229; Rust median 18.5. [experiments/B1_status.md](experiments/B1_status.md) |
| **B.5** | Перенос SIFS-политики в Rust: --load-weights, CMA-ES, A/B. [experiments/B5_AB_RESULTS.md](experiments/B5_AB_RESULTS.md) |
| **D.1** | Бенчмарк GPU 100K/1M. [experiments/B4_benchmarks.md](experiments/B4_benchmarks.md) |
| **E.1** | Архитектура SIFS-мозга. [docs/E_SIFS_BRAIN_ARCHITECTURE.md](docs/E_SIFS_BRAIN_ARCHITECTURE.md) |
| **E.2–E.4** | CartPole в Rust (124), торговля, мультитаск — в работе или частично. [experiments/E2_RUN_RESULTS.md](experiments/E2_RUN_RESULTS.md) |
| **F** | brain_query, genesis_agi start → Python Brain Server, run_f5_verify. [docs/WORKING_BRAIN_DEMO.md](docs/WORKING_BRAIN_DEMO.md) |
| **G, H** | Верификация конвейера, OHLCV, run_e3_compare. |
| **I** | «Думание»: state_in/state_out, think_steps (BRAIN_CONTRACT §3.6), sifs_codec. [docs/I_THINKING_SPEC.md](docs/I_THINKING_SPEC.md), [docs/I_THINKING_CODEC.md](docs/I_THINKING_CODEC.md) |
| **J** | Чат-CLI с агентами (chat_cli). |
| **K** | Самообучение: OWN_AI_SELFLEARNING, K.1–K.4 (контракт §3.7, GPU+CPU, петля, «голос»). [docs/K2_GPU_CPU_INTEGRATION.md](docs/K2_GPU_CPU_INTEGRATION.md), [docs/K3_SELFLEARNING_LOOP.md](docs/K3_SELFLEARNING_LOOP.md) |
| **L** | OFI-сигналы → Freqtrade: L.0–L.5 (формат, стратегия, пайплайн, бэктест, цикл). [docs/OFI_FREQTRADE_SIGNALS.md](docs/OFI_FREQTRADE_SIGNALS.md) |

---

## Ссылки

- Чеклист §13: [GENESIS_SIFS_AI_PLAN.md](GENESIS_SIFS_AI_PLAN.md)
- Лог изменений: [CHANGELOG.md](CHANGELOG.md)
- Как продолжить в новом чате: [HOW_TO_CONTINUE.md](HOW_TO_CONTINUE.md)
