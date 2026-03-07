# S-Sharding и Ghost Axons на границах S

Шардирование по S-координате (уровни 0..9) и передача спайков между соседними S-уровнями (Ghost Axons).

## S-Sharding

- **Источник:** план мозга п.1.4; Genesis — planar XY sharding заменён на **S-coordinate sharding**.
- **Назначение:** каждый нейрон принадлежит ровно одному шарду по дискретизированной S-координате: `s_level = min(9, floor(s_coordinate))`.
- **Функция:** `build_s_shards_from_neurons(neurons, shards)` — очищает шарды и заполняет их по текущему `neurons.s_coordinate[]`. Вызывается после Night Phase (resharding по Hebbian ΔS) или при инициализации.

## Ghost Axons на S-boundaries

- **Источник:** [genesis-agi 04_connectivity §1.7](https://github.com/H4V1K-dev/genesis-agi/blob/master/docs/specs/04_connectivity.md), 06_distributed — Ghost Axon handover между шардами.
- **Правило:** если нейрон в шарде **L** выдал спайк, то нейроны в шардах **L−1** и **L+1** получают этот спайк как входящий (ApplySpikeBatch).
- **Функция:** `ghost_spike_targets(neurons, shards, spikes)` возвращает дедуплицированный список индексов нейронов-получателей. Этот список передаётся в `day_phase_pipeline_6step(..., incoming_ghost_spikes)` на **следующем** тике.

## Цикл с Ghost handover

```
1. build_s_shards_from_neurons(&neurons, &mut shards)
2. spikes = day_phase_pipeline_6step(..., &[])
3. ghost = ghost_spike_targets(&neurons, &shards, &spikes)
4. spikes_next = day_phase_pipeline_6step(..., &ghost)
5. (при необходимости) night_phase_plasticity → снова build_s_shards_from_neurons
```

В бенчмарме и в тесте `test_s_sharding_and_ghost_axons` используется тот же порядок.

## Дальнейшая интеграция

- **Genesis:** полный Ghost Axon = слот в `axon_heads[]` принимающего шарда, маппинг `soma_id_источника → ghost_axon_id`. В нашей CPU-модели — упрощённо: все нейроны соседнего S-уровня получают токен.
- **Дендриты:** при появлении явной топологии (dendrite_targets по S-границам) можно сузить ghost_targets до нейронов, у которых есть связь с источником спайка.
