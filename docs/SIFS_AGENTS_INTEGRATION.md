# Интеграция Genesis-мозга с sifs_agents (§5 плана)

Фиксация выбранного варианта A/B/C и мест реализации. Контракт ролей — [BRAIN_CONTRACT.md](../BRAIN_CONTRACT.md) §3.5.

---

## Выбранный вариант: **A (дополнение)**

**Роль мозга:** подсистема быстрых решений (реакция, торговый сигнал, низколатентный ответ). LLM-агенты остаются для рассуждений, планов, MCP-инструментов.

**Связь:** мозг вызывается из агента через `brain_query` (MCP) или HTTP (POST /step); агент передаёт наблюдение и получает действие/readout.

**Обоснование:** уже есть MCP-инструмент `brain_query` в sifs_agents; добавлен HTTP-клиент [brain_http_client.py](../scripts/brain_http_client.py) для Genesis --serve. Ни в одном пайпе мозг пока не заменяет LLM; замена (вариант B) или явное разделение зон (вариант C) — при появлении конкретных сценариев.

---

## Где реализовано

| Компонент | Реализация |
|-----------|-------------|
| **MCP brain_query** | sifs_agents/mcp_server.py → brain_client (репо sifs-genesis-brain). Для Genesis CPU: подставить URL --serve и вызывать HTTP через brain_http_client или requests. |
| **HTTP-клиент к Genesis** | [Genesis/scripts/brain_http_client.py](../scripts/brain_http_client.py); пример в [CLIENT_INTEGRATION.md](CLIENT_INTEGRATION.md). |
| **Конфиг fallback** | Пример в [CLIENT_INTEGRATION.md](CLIENT_INTEGRATION.md) (brain.url, timeout_ms, fallback_on_unavailable). Реализация чтения конфига — в конкретном клиенте при интеграции. |

---

## Варианты B и C (на будущее)

- **B (замена в узком слое):** когда появится пайп «только торговый сигнал» без LLM — зафиксировать в этом документе и в конфиге развёртывания.
- **C (разделение зон):** когда будут явно описаны зоны «мозг решает» / «агенты решают» — оформить в архитектурном доке и обновить этот раздел.

Контракт входа/выхода мозга (BRAIN_CONTRACT §3.1–3.2) не зависит от выбора A/B/C.
