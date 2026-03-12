# Как продолжить в новом диалоге без потерь планов

Планы и прогресс **уже лежат в файлах** — новый чат может опираться на них.

---

## Что сказать в новом диалоге

Скопируй и вставь (или сократи):

```
Продолжай план Genesis + SIFS. Контекст:
- План и чеклист: Genesis/GENESIS_SIFS_AI_PLAN.md (§13)
- Лог изменений: Genesis/CHANGELOG.md
- Правила и контракт: .cursor/rules/, Genesis/BRAIN_CONTRACT.md, CLAUDE.md
```

Либо короче: **«Продолжай план Genesis, см. GENESIS_SIFS_AI_PLAN.md и CHANGELOG»**.

**«Продолжай по NEXT_PLAN»** или **«Выполняй NEXT_PLAN»** — агент берёт **первый незакрытый шаг** из [NEXT_PLAN.md](NEXT_PLAN.md) (B.1, D.1, …) и выполняет его. Q.1 (тесты) — только после правок кода или перед завершением сессии, не по умолчанию при «продолжай».

При необходимости приложи файлы: `@Genesis/GENESIS_SIFS_AI_PLAN.md`, `@Genesis/CHANGELOG.md`, `@Genesis/NEXT_PLAN.md`.

---

## Где что лежит (без потерь)

| Что | Файл |
|-----|------|
| План, фазы A–D, чеклист §13 | [GENESIS_SIFS_AI_PLAN.md](GENESIS_SIFS_AI_PLAN.md) |
| **Пошаговый план «что дальше»** | **[NEXT_PLAN.md](NEXT_PLAN.md)** — B.1, D.1, качество; продолжать без остановок по нему |
| Лог решений и правок | [CHANGELOG.md](CHANGELOG.md) |
| Контракт мозга, константы | [BRAIN_CONTRACT.md](BRAIN_CONTRACT.md) |
| Репозитории и эталон SIFS | [docs/REPOS_AND_REFERENCE.md](docs/REPOS_AND_REFERENCE.md) |
| Интеграция с sifs_agents | [docs/SIFS_AGENTS_INTEGRATION.md](docs/SIFS_AGENTS_INTEGRATION.md) |
| Чекпоинтинг, забывание, торговля | [docs/](docs/) (CHECKPOINTING_WEIGHTS, CATASTROPHIC_FORGETTING, TRADING_REWARD_AND_DATA) |
| Правила Cursor / Cowork | [.cursor/rules/](../.cursor/rules/), [CLAUDE.md](../CLAUDE.md), [AGENTS.md](../AGENTS.md) |

---

## Текущий фокус (обновлено при продолжении плана)

- **D.1 выполнен и отмечен в §13:** бенчмарки GPU 100K и 1M в genesis-agi (bench_100k, bench_1m), результаты в [experiments/B4_benchmarks.md](experiments/B4_benchmarks.md).
- **D.3 выполнен:** конфиг §9, скрипты trading_brain_loop и run_d3_backtest, тесты; опциональный режим мозга в sifs_ft. §13: D.3 отмечен.
- **Открытый пункт §13:** **B.1** — CartPole медиана ≥71. В src/ реализованы моторные пулы и обучаемый readout (REINFORCE по reward). Калибровка: (5,200) без trainable — median 16.0; с --trainable-readout --readout-lr 0.03 — **median 17.0**, mean 20.4. Цель ≥71 не достигнута. Статус: [experiments/B1_status.md](experiments/B1_status.md). Команда: `python run_cartpole_agent.py --seed 42 --episodes 50 --steps 5 --night 200 --dopamine-shaping --population-coding [--trainable-readout] [--readout-lr 0.03]`.
- Остальное по фазам A, C, D.2, §5–§7, §9 — отмечено в §13.


При продолжении работы обновляй CHANGELOG и при необходимости этот раздел «Текущий фокус».
