# Репозитории и эталон для сверки SIFS (Genesis)

Где лежат локальные копии «Genesis» с GitHub и как сверять константы и работу мозга.

**Примечание:** Genesis-AGI — отдельный независимый проект (H4V1K-dev). Мы не его авторы; мы взяли у них разработки и объединили с теорией SIFS.

---

## Что есть в проекте (локально)

| Путь | Что это | Git / источник |
|------|---------|----------------|
| **sifs-genesis-brain** (этот репо) | **Канонический репо SIFS-Genesis:** дерево Genesis — один крейт sifs-genesis-hybrid (src/), run_cartpole_agent.py, experiments, docs. Submodule deps/genesis-agi. | Клон `https://github.com/m0rfy/sifs-genesis-brain` |
| **Genesis/** (в Projects) | Локальная рабочая копия того же дерева (без своего .git). Синхронизировать с репо по необходимости. | Часть workspace Projects. |
| **genesis-agi/** | Полный стек Genesis (CUDA, node, baker). Копия для встраивания SIFS; в репо — как submodule deps/genesis-agi. | Клон `https://github.com/H4V1K-dev/genesis-agi.git` |

Без локальных клонов **genesis-agi** и **sifs-genesis-brain** нельзя сверять протокол и константы с «основным рабочим» кодом с GitHub. Рекомендация: держать оба клона актуальными (`git pull`); при клонировании sifs-genesis-brain — `git clone --recurse-submodules`.

---

## Эталон констант SIFS

**Единый эталон по числам:** [sifs_ft/strategies/modules/core.py](../../sifs_ft/strategies/modules/core.py) (K, PHI, FIB, W(n), V_th(n)=V₀/φⁿ).  

Сверка:
- **Genesis/** (наш CPU) и **core.py** — скрипт [scripts/compare_sifs.py](../scripts/compare_sifs.py) (A.1). Бинарник `sifs_genesis_hybrid --compute-sifs` vs Python I_SIFS при одинаковом входе; допуск в скрипте.
- **genesis-agi** — константы в [genesis-core/src/sifs.rs](../../genesis-agi/genesis-core/src/sifs.rs) и в [genesis-compute CUDA](../../genesis-agi/genesis-compute/src/cuda/physics.cu). При изменении core.py обновить эти файлы и проверить регрессию.
- **sifs-genesis-brain** (этот репо) — константы в src/ (constants.rs, fixed.rs и т.д.); по BRAIN_CONTRACT синхронны с core.py. При расхождении — привести к BRAIN_CONTRACT и core.py.

---

## Как настраивать SIFS корректнее

1. **Константы:** менять только в core.py; затем синхронизировать в этом репо (src/), при необходимости в genesis-agi (sifs.rs, physics.cu). Правило: без зелёного `compare_sifs.py` не мержить правки в константы (см. BRAIN_CONTRACT).
2. **Сверка работы:**  
   - CPU-цикл: [run_cartpole_agent.py](../run_cartpole_agent.py) → бинарник из корня репо (`cargo build --release` → `sifs_genesis_hybrid`).  
   - Референс «мозг + SIFS» — этот репо. При отличии поведения от локальной копии Genesis — сравнить протокол с [BRAIN_CONTRACT.md](../BRAIN_CONTRACT.md) и привести к одному контракту.
3. **Обновление клонов:**  
   - `genesis-agi`: `git pull origin main` (или текущая ветка); при конфликтах в наших правках (sifs.rs, physics.cu) — разрешить вручную, сохраняя константы из core.py.  
   - `sifs-genesis-brain`: `git pull` и при наличии submodule — `git submodule update --init --recursive`.

---

## Если клона sifs-genesis-brain или genesis-agi нет

- **genesis-agi:**  
  `git clone https://github.com/H4V1K-dev/genesis-agi.git` в каталог `Projects/genesis-agi`. Для нашей интеграции SIFS см. [GENESIS_CUDA_SIFS.md](../GENESIS_CUDA_SIFS.md) и [genesis-agi/SIFS_ORIGIN.md](../../genesis-agi/SIFS_ORIGIN.md) (если есть).
- **sifs-genesis-brain:**  
  `git clone --recurse-submodules https://github.com/m0rfy/sifs-genesis-brain.git` в каталог `Projects/sifs-genesis-brain`. Для MCP brain_query в sifs_agents задать `SIFS_GENESIS_BRAIN_ROOT=...\sifs-genesis-brain` (путь к корню репо).

---

## Ссылки

- Контракт и константы: [BRAIN_CONTRACT.md](../BRAIN_CONTRACT.md).  
- План: [GENESIS_SIFS_AI_PLAN.md](../GENESIS_SIFS_AI_PLAN.md).  
- CUDA и genesis-agi: [GENESIS_CUDA_SIFS.md](../GENESIS_CUDA_SIFS.md).  
- Интеграция sifs_agents: [SIFS_AGENTS_INTEGRATION.md](SIFS_AGENTS_INTEGRATION.md).
