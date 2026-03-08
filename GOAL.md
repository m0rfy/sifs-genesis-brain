# Цель проекта sifs-genesis-brain

**Основная задача:** довести проект до ума — реализовать нейроморфное ядро «мозга» на основе двух источников:

1. **NotebookLM SIFS (21 источник)** — канон спецификации: формулы, константы (K, φ, FIB, W), пороги, обозначения. Все изменения в коде сверять с этой спецификацией. *Примечание: сам артефакт NotebookLM вне репо; в репо используются [SIFS_Theory_Documentation.md](../документация/SIFS_Theory_Documentation.md), [core.py](../sifs_ft/strategies/modules/core.py) и [BRAIN_CONTRACT.md](docs/BRAIN_CONTRACT.md) как синхронизированные с ней источники.*

2. **[genesis-agi](https://github.com/H4V1K-dev/genesis-agi)** — движок для воплощённого интеллекта: Integer Physics, Day/Night, GSOP, SoA, CUDA, Baker, Node, CartPole E2E. Подключён как submodule `deps/genesis-agi`. Слой SIFS (I_SIFS, φ-пороги, S-sharding) реализуется поверх типов и рантайма Genesis в этом репо.

## Режим работы

- **Делать всё автоматически:** использовать все доступные инструменты (сборка, тесты, линтеры, MCP, скрипты экспорта констант, документация).
- **Единый источник констант:** Python — [core.py](../sifs_ft/strategies/modules/core.py); Rust — синхронно с core.py и [docs/BRAIN_CONTRACT.md](docs/BRAIN_CONTRACT.md); при изменении формул — сверять с NotebookLM Spec / SIFS_Theory_Documentation.

## Критерии «доведения до ума»

| Веха | Описание |
|------|----------|
| Спецификация | Контракт мозга (I/O, K, PHI, FIB, W, V₀, Day/Night) зафиксирован и совпадает с core.py и Spec. |
| Крейт + submodule | sifs-genesis-core собирается; genesis-agi в deps; при необходимости — path dependency на genesis-core. |
| Day Phase + SIFS | I_SIFS и φ-пороги в горячем цикле (rayon или CUDA); порядок пайплайна согласован с Genesis. |
| S-Sharding | Шардирование по S-координате (уровни 0..9); Ghost Axons на границах S. |
| Night Phase | Пластичность (Hebbian ΔS, resharding), Sort & Prune, Cone Tracing по спецификации Genesis. |
| I/O и API | Входы/выходы по [08_io_matrix](https://github.com/H4V1K-dev/genesis-agi/blob/master/docs/specs/08_io_matrix.md); API (UDP/HTTP); Python-клиент; конфиг Brain TOML. |
| MCP и агенты | Инструмент brain_query/brain_context в sifs_agents; README обновлён. |
| Верификация | Юнит-тесты, бенчмарки (1K–100K нейронов), числовая идентичность с Python (I_SIFS), при необходимости CartPole E2E. |

## Ссылки

- План: см. внутреннюю документацию проекта (вне этого репо).
- Контракт: [docs/BRAIN_CONTRACT.md](docs/BRAIN_CONTRACT.md)
- Genesis-agi: [deps/genesis-agi](deps/genesis-agi), [документация Genesis](https://github.com/H4V1K-dev/genesis-agi/tree/master/docs/specs)
