#!/usr/bin/env python3
"""
Клиент мозга по HTTP (фаза C.2). Контракт: BRAIN_CONTRACT §3.
POST /step с {"input": [float, ...], "reward": float} → {"readout": {...}, "action": 0|1}.
Использование: как скрипт (печать action) или import brain_step().
"""
from __future__ import annotations

import json
import socket
import sys
import urllib.error
import urllib.request
from typing import Any


def brain_step(
    url: str,
    observation: list[float],
    reward: float = 0.0,
    timeout_sec: float = 5.0,
) -> dict[str, Any]:
    """
    Один шаг: отправить наблюдение (и опционально reward) в мозг, получить action и readout.
    При ошибке/таймауте выбрасывает исключение; вызывающий код может применить fallback.
    """
    payload = {"input": observation, "reward": reward}
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        f"{url.rstrip('/')}/step",
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout_sec) as resp:
            body = resp.read().decode("utf-8")
    except urllib.error.URLError as e:
        reason = getattr(e, "reason", None)
        if isinstance(reason, (TimeoutError, socket.timeout)):
            raise RuntimeError(f"Таймаут {timeout_sec} с") from e
        raise RuntimeError(f"Мозг недоступен: {e}") from e
    except TimeoutError as e:
        raise RuntimeError(f"Таймаут {timeout_sec} с") from e
    out = json.loads(body)
    if "action" not in out:
        raise RuntimeError(f"Некорректный ответ мозга: {out}")
    return out


def main() -> int:
    url = "http://127.0.0.1:3030"
    obs = [0.0, 0.0, 0.0, 0.0]
    if len(sys.argv) > 1:
        url = sys.argv[1]
    if len(sys.argv) > 2:
        obs = [float(x) for x in sys.argv[2].split(",")]
    try:
        result = brain_step(url, obs, timeout_sec=2.0)
        print(json.dumps(result))
        return 0
    except Exception as e:
        print(json.dumps({"error": str(e), "action": None}), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
