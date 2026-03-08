# Интеграция клиентов с мозгом (фаза C.1)

Краткий runbook: как подключаться к мозгу и что делать при недоступности. Детали контракта — [BRAIN_CONTRACT.md](../BRAIN_CONTRACT.md) §3.

---

## Latency budget

| Сценарий | Допустимая задержка ответа мозга |
|----------|----------------------------------|
| Торговля, таймфрейм 15m | &lt;500 ms |
| CartPole, интерактивные симуляции | &lt;100 ms на шаг |

При превышении клиент может считать мозг недоступным и применять fallback.

---

## Fallback при недоступности мозга

При недоступности HTTP/UDP мозга (таймаут, ошибка соединения, 5xx) клиент **не** должен подставлять случайное действие. Варианты задаются в конфиге:

| Значение | Поведение |
|----------|-----------|
| `pause` | Не открывать новые позиции / не выполнять действия до восстановления связи |
| `sifs_only` | Использовать правило на основе SIFS-сигнала без мозга (если реализовано в клиенте) |
| `fail` | Возвращать ошибку вызывающему коду; не выполнять действие |

### Пример конфига клиента (YAML)

```yaml
brain:
  url: "http://127.0.0.1:3030"
  timeout_ms: 400
  fallback_on_unavailable: "pause"   # pause | sifs_only | fail
```

### Пример конфига (TOML)

```toml
[brain]
url = "http://127.0.0.1:3030"
timeout_ms = 400
fallback_on_unavailable = "pause"
```

Реализация: при таймауте или ошибке смотреть `fallback_on_unavailable` и выполнять соответствующее поведение; не вызывать мозг повторно до следующего тика или до явного retry.

---

## Когда HTTP, когда UDP

- **HTTP (--serve):** оркестрация, MCP, отладка, запросы не в hot path. Удобно для интеграции с sifs_agents, скриптами, Freqtrade (если latency в бюджете).
- **UDP (genesis-node):** hot path, низкая задержка, торговый тик. Тот же контракт §3.1–3.2; транспорт выбирается по сценарию.

---

## Пример вызова (Python HTTP)

Мозг должен быть запущен: `cargo run --release -- --serve 127.0.0.1:3030`.

```python
import urllib.request
import json

def brain_step(url: str, observation: list[float], reward: float = 0.0, timeout_sec: float = 5.0):
    payload = {"input": observation, "reward": reward}
    req = urllib.request.Request(
        f"{url.rstrip('/')}/step",
        data=json.dumps(payload).encode("utf-8"),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=timeout_sec) as resp:
        return json.loads(resp.read().decode("utf-8"))

# Пример
result = brain_step("http://127.0.0.1:3030", [0.1, 0.2, 0.0, 0.1])
action = result["action"]  # 0 или 1
readout = result.get("readout", {})
```

Готовый скрипт: [scripts/brain_http_client.py](../scripts/brain_http_client.py) — можно вызывать из командной строки или импортировать `brain_step()`.

**Сценарии C.2:** (1) CartPole через stdio — [run_cartpole_agent.py](../run_cartpole_agent.py); (2) любой клиент через HTTP — этот пример или brain_http_client.py; (3) **D.3 торговый цикл** — [scripts/trading_brain_loop.py](../scripts/trading_brain_loop.py): бар → наблюдение → мозг → действие по историческим .feather (sifs_ft/data/binance), fallback по конфигу.

---

## Ссылки

- Контракт входа/выхода, latency, роль мозга: [BRAIN_CONTRACT.md](../BRAIN_CONTRACT.md) §3.1–3.5.
- План: [GENESIS_SIFS_AI_PLAN.md](../GENESIS_SIFS_AI_PLAN.md) фаза C.
