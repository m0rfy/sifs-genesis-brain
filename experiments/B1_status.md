# B.1 Статус: CartPole медиана ≥71

**Критерий done:** медиана total_reward ≥71 за 50 эпизодов (сид 42), A/B с/без I_SIFS задокументирован.

---

## Текущая базовая линия (2025-03-07; повтор 2025-03-08; повторный запуск B.1 с целью ≥71)

| Метрика | Значение |
|---------|----------|
| Медиана (I_SIFS on) | **12.0** (steps=5, night=200, dopamine-shaping, пластичность по SIFS; mean 11.4). Ранее 10.0 при steps=10, night=200. |
| Медиана (I_SIFS off) | 9.0 |
| Цель B.1 | **≥71** (повторный запуск с настройками B.1 подтвердил медиану 10; цель не достигнута) |

**A/B с/без I_SIFS (B.1.5):** 50 эп. on → медиана 10, mean 9.9; 50 эп. off → медиана 9, mean 9.4. Таблица в A0_result.md и здесь.

**Повторный прогон с доставкой reward (B.2):** 50 эп. on → mean 9.9, медиана 10; 50 эп. off → mean 9.4, медиана 9. Без калибровки/референса до 71 не дотягиваем.

**Прогон калибровки (после добавления --night):** один прогон 50 эп. (steps=10, night=200) → медиана 9.0, mean 9.4. Сетка 2×2 (steps 10,15 × night 100,200) — все ячейки медиана 9.0, mean 9.4. Результат: [B1_calibration_results.md](B1_calibration_results.md).

Цикл: `run_cartpole_agent.py` → sifs_genesis_hybrid --agent; наблюдение + **reward** (B.2) → мозг → действие.

**Реализовано (B.2):** доставка reward в мозг: Python шлёт `{"input": [...], "reward": prev_reward}`; бинарник принимает, сохраняет в `last_reward`, в Night Phase модулирует пластичность (scale 1 + reward×0.5 для s_coordinate).

**Dopamine shaping (2025-03-08):** по образцу genesis-agi в `run_cartpole_agent.py` — сигнал от угла/угловой скорости и terminal; флаг `--dopamine-shaping`. Калибровка 50 эп.: median 10.0, mean 10.8 (без shaping mean 10.5).

**Пластичность по SIFS (2025-03-08):** в `night_phase_plasticity` eta и дельта синапсов зависят от s_level и W(s_level): мелкий масштаб — пластичнее, крупный — стабильнее; вклад W(n) уменьшает изменение весов у крупномасштабных нейронов. Тесты 9 passed; 50 эп. с shaping: median 10.0, mean 10.8.

**Inertia по весам (2025-03-08):** по рекомендации MCP (genesis-agi 04_connectivity) в night_phase добавлена кривая inertia_curve[16] по рангу |weight| — сильные связи меняются слабее. 50 эп. steps=5 night=200: median 12.0, mean 11.4 (без регрессии). Рекомендации по дальнейшим шагам: [B1_NEXT_RECOMMENDATIONS.md](B1_NEXT_RECOMMENDATIONS.md).

**Population coding / попеременное пополнение (2025-03-08):** применён в `run_cartpole_agent.py` — 4 переменные → 4 сегмента по 16 активаций (Gaussian receptive field). Флаг `--population-coding`; 50 эп. steps=5 night=200 с --dopamine-shaping: median 12.0, mean 11.4. Рекомендуемая команда B.1: добавить `--population-coding` к калибровке при необходимости.

**Night phase по концу эпизода (рекомендация 3, 2025-03-08):** в [sifs_genesis_hybrid.rs](../sifs_genesis_hybrid.rs) — `BrainRunner::trigger_night_phase()`, в agent loop парсинг `episode_end: true` и вызов после step. В [run_cartpole_agent.py](../run_cartpole_agent.py) при terminated/truncated отправляется запрос с `episode_end: true` и терминальным reward. Контракт BRAIN_CONTRACT §3.1 обновлён. Сетка 2×2 (steps 5,10 × night 150,200): лучшая **steps=5, night=200 — median 11.0, mean 10.7**; 3 повторных прогона — стабильно 11.0/10.7.

---

## Блокер (ослаблен)

Раньше мозг работал в режиме inference-only без reward. Сейчас:
1. **Доставка reward** — реализована (stdio и контракт §3.1).
2. **Использование в Night Phase** — reward модулирует изменение s_coordinate (положительный reward усиливает пластичность).
3. **Обучение весов дендритов** — по-прежнему не реализовано (только s_coordinate и resharding).

Чтобы выйти на медиану 71+, после прогона 50 эпизодов с reward проверить медиану; при необходимости — калибровка гиперпараметров (night_interval, reward_scale), референс из genesis-agi или расширение пластичности (веса).

### B.1.опц — обучение весов дендритов (реализовано)

- **Сделано (2025-03-08):** В [sifs_genesis_hybrid.rs](../sifs_genesis_hybrid.rs): (1) добавлено поле `synaptic_weights: Vec<Vec<Fixed16>>` (веса i→j по рёбрам `dendrite_targets`); (2) в `NeuronArrays::new()` заполняются `dendrite_targets` (кольцо, OUT_DEGREE=5) и начальные веса 1.0; (3) в `night_phase_plasticity` — Hebbian-дельта с `reward_scale`, кламп весов [0, 2×FIXED_ONE]; (4) в readout (`BrainRunner::step`) — score = population_score + 0.2×recurrent_score (взвешенная по синапсам активность через `weighted_recurrent_sum`). Исправлено затенение переменной `n` в цикле по уровням (lv).
- **Проверка:** cargo test 9 passed, pytest 14 passed. Калибровка: BETA_RECURRENT=0.4, eta=0.008 даёт **медиану 10.0, mean 10.5** (50 эп., seed 42). **Почему не 0.00008:** в Q16.16 при eta ~1e-4 и типичном hebbian_raw (0.0001–0.01) выражение `(eta*hebbian_raw)>>16` обнуляется — пластичность пропадает. Оставляем eta в диапазоне 0.005–0.01.

---

## Рекомендация агентов (консультация MCP)

- **B.1 отложить до после фазы C.** C (контракт, HTTP, fallback, sifs_agents) от B.1 не зависит; B.1 упирается в обучение весов дендритов или калибровку с неочевидной отдачей.
- **Когда вернёмся к B.1:** (1) референс genesis-agi CartPole (50 эп., сид 42) → зафиксировать в B1_genesis_agi_reference.md; (2) быстрые тесты: `--steps 10`–20, `--night 100`–300; (3) dopamine shaping и/или population coding по образцу genesis-agi; (4) обучение весов дендритов в Night Phase — отдельная доработка.

## Референс genesis-agi

- **Референс применён и проверен (2025-03-08):** [B1_genesis_agi_reference.md](B1_genesis_agi_reference.md) — прогон 50 эп. seed 42 в genesis-agi (ноды SensoryCortex + MotorCortex): **медиана 9.0, mean 9.4**, коммит f14494c. Наш CPU-мозг (BETA=0.4, eta=0.008): медиана 10, mean 10.5 — выше референса. Цель ≥71 не достигнута ни референсом, ни нами.

## Следующие шаги

1. ~~Прогнать 50 эпизодов с reward~~ — сделано (медианы 10 / 9).
2. ~~B.1 отложен до после C~~ — C выполнен, возврат к B.1.
3. ~~Референс genesis-agi~~ — выполнен, медиана 9.0 ([B1_genesis_agi_reference.md](B1_genesis_agi_reference.md)).
4. ~~Калибровка раунд 3~~ — сетка steps 8,10,12 × night 150,200,250; лучшая ячейка steps=10, night=200 (median 10.0, mean 10.5). [B1_calibration_results.md](B1_calibration_results.md).
5. ~~Калибровка раунд 4~~ — сетка steps 5,10,15 × night 100,200,300 с dopamine-shaping: **лучшая steps=5, night=200 (median 12.0, mean 11.4)**. Рекомендуемая команда: `--steps 5 --night 200 --dopamine-shaping`.
6. ~~Калибровка раунд 5~~ — окрестность (5,200): steps 4,5,6,7 × night 150,200,250 и steps=5 × night 175,200,225,300. Лучшая по-прежнему **steps=5, night=200** (median 12.0). [B1_calibration_results.md](B1_calibration_results.md).
7. **Night phase по концу эпизода (п.3):** реализован; сетка 2×2 + 3 повтора — median 11.0 стабильно. Расширенная сетка 3×3 (steps 4,5,6 × night 175,200,225) — только (5,200) даёт 11.0. Reward_scale 0.7 (п.4) проверен — без прироста, оставлен 0.5.
8. **Моторные пулы (п.6) и анализ 11 vs 71:** реализованы два readout (left/right половины нейронов), action = argmax. Калибровка: (5,200) по-прежнему 11.0; night=50/30 — хуже (10.0, 9.5). Анализ разрыва и выводы: [B1_ANALYSIS_GAP_71.md](B1_ANALYSIS_GAP_71.md). Потолок ~11 при текущей архитектуре; для 71 нужен обучаемый readout или иная схема обучения.
9. **Обучаемый readout (src/):** реализован в src/neuron.rs, runner.rs, io.rs (episode_end), main.rs (CLI). Веса пулов обновляются по reward (REINFORCE). Cargo test 11 passed (в т.ч. test_trainable_readout_*). Калибровка: пулы без trainable — median **16.0**; с --trainable-readout --readout-lr 0.03 — median **17.0**, mean 20.4 (стабильно). Команда: `python run_cartpole_agent.py --seed 42 --episodes 50 --steps 5 --night 200 --dopamine-shaping --population-coding [--trainable-readout] [--readout-lr 0.03]`.
11. **Прогон 200 эп., медиана по последним 50 (2025-03-08):** выполнено. В run_cartpole_agent при episodes ≥ 50 выводится `Median reward (last 50):`. Результат: 200 эп. steps=5 night=200 dopamine+population — median (all) 10.0, **median (last 50) 9.0**. Прирост от удлинения прогона не получен; следующий кандидат — night phase по флагу «конец эпизода» (рекомендация 3) или моторные пулы (6). Зафиксировано в [B1_calibration_results.md](B1_calibration_results.md).
12. **Калибровка нашего CPU (дальше):** бинарник поддерживает `--steps` и `--night`. Один прогон 50 эп., сид 42:
   ```bash
   cd Genesis
   python run_cartpole_agent.py --seed 42 --episodes 50 --steps 5 --night 200 --dopamine-shaping [--population-coding]
   ```
   Сетка (перебор steps × night) и вывод таблицы медиан:
   ```bash
   python scripts/run_b1_calibration.py --grid [--grid-steps 10,15,20] [--grid-night 100,200,300] [--out experiments/B1_calibration_results.md]
   ```
   Один набор параметров:
   ```bash
   python scripts/run_b1_calibration.py --steps 15 --night 200
   ```
13. При медиане ≥71 — отметить B.1 в §13 плана и CHANGELOG.
