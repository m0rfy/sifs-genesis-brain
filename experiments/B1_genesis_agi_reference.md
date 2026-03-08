# B.1 Референс: CartPole в genesis-agi (целевая медиана ≥71)

Эталон для калибровки нашего CPU-мозга (Genesis/sifs_genesis_hybrid): заявленный в genesis-agi рекорд 71+ за 50 эпизодов. Контракт genesis-agi — UDP (GSIO/GSOO), наш — JSON stdio/HTTP; референс задаёт **целевую метрику**, не формат.

---

## Чеклист для применения (следующий шаг по NEXT_PLAN)

**Статус применения:** Чеклист применён и проверен: ноды подняты (SensoryCortex + MotorCortex), выполнен прогон 50 эп. seed 42. Результаты внесены в таблицу ниже. Референс genesis-agi: **медиана 9.0, mean 9.4** (наша CPU-мозг при BETA=0.4, eta=0.008 даёт медиану 10, mean 10.5 — выше референса).

1. **Открыть репо genesis-agi** (рядом с Projects или путь из `Genesis/../genesis-agi`).
2. **Поднять 2 ноды** по [RUN_WINDOWS.md](../../genesis-agi/RUN_WINDOWS.md) (baker + node на портах 8081/8092).
3. **В терминале из корня genesis-agi выполнить:**
   ```bash
   python examples/cartpole/cartpole_client.py --seed 42 --episodes 50
   ```
4. **Скопировать из вывода** строки `Mean reward (50 episodes): X` и `Median reward: Y`.
5. **В этом файле** заполнить таблицу «Результаты» (медиана, mean, дата) и при необходимости параметры (night_interval, V₀, коммит: `git rev-parse HEAD` в genesis-agi).
6. **Сравнить** с нашей медианой 10 (steps=10, night=200, BETA=0.4, eta=0.008); при референсе ≥71 — цель для нашего CPU; при референсе <71 — зафиксировать и при необходимости пересмотреть критерий B.1.

Команда для вывода чеклиста в терминале: из `Genesis/` выполнить `python scripts/print_b1_reference_steps.py`.

---

## Параметры прогона (заполнить после запуска)

| Параметр | Значение | Примечание |
|----------|----------|------------|
| Сид | 42 | Фиксировано для воспроизводимости |
| Эпизодов | 50 | По плану A.0/B.1 |
| Клиент | `genesis-agi/examples/cartpole/cartpole_client.py` | UDP → ноды 8081/8092 |
| Population Coding | 4→64 | Вход CartPole → 64 нейронов (по readme genesis-agi) |
| BATCH_TICKS / steps_per_obs | 100 | В cartpole_client.py константа BATCH_TICKS |
| night_interval (если есть) | см. конфиг нод | В нодах genesis-agi |
| V₀ (порог SIFS) | см. genesis-core/sifs.rs | Из конфига |
| Формула reward (dopamine) | см. код | terminated → -20000; иначе (0.03−\|pole_a\|)×25000 − \|pole_av\|×5000, clip ±32767 |
| Версия / коммит genesis-agi | f14494c | 2025-03-08 |

---

## Результаты (прогон 50 эп., сид 42 — применён и проверен)

| Метрика | Значение |
|---------|----------|
| Медиана total_reward | 9.0 |
| Mean total_reward | 9.4 |
| Дата прогона | 2025-03-08 |

**Команда запуска (из корня genesis-agi) — следующий шаг по NEXT_PLAN:**
1. Поднять 2 ноды по [RUN_WINDOWS.md](../../genesis-agi/RUN_WINDOWS.md) (или аналог в репо genesis-agi).
2. Из корня genesis-agi (после подъёма нод): `python examples/cartpole/cartpole_client.py --seed 42 --episodes 50` — вывод в конце: Mean reward (50 episodes): X, Median reward: Y.
3. После прогона 50 эп. вставить медиану/mean в таблицу «Результаты» выше и заполнить оставшиеся параметры (night_interval, V₀, коммит).

---

## Связь с нашим CPU-мозгом

- **Наша калибровка:** перебор `--steps` (10–20), `--night` (100–300) через Genesis [scripts/run_b1_calibration.py](../scripts/run_b1_calibration.py) (50 эп., сид 42). Текущий наш результат: **медиана 10, mean 10.5** (BETA=0.4, eta=0.008, steps=10, night=200) — **выше референса genesis-agi** (медиана 9, mean 9.4).
- **Цель:** медиана нашего прогона ≥71 (референс genesis-agi на 2025-03-08 — 9.0; цель 71 пока не достигнута ни референсом, ни нашим CPU).
- **Контракт:** BRAIN_CONTRACT §6.1 — воспроизводимость B.1 (сид, гиперпараметры); при достижении медианы ≥71 обновить §13 плана и CHANGELOG.
