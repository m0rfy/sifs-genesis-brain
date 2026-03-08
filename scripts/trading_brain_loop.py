#!/usr/bin/env python3
"""
D.3: минимальный цикл «бар → наблюдение → мозг → действие» по историческим данным.
Читает OHLCV из .feather, строит вектор наблюдения, вызывает мозг по HTTP,
передаёт reward (PnL за предыдущий бар, нормированный в [-1,1] по §6 TRADING_REWARD_AND_DATA).
Логирует (t, reward_sent, action, readout). Контракт: BRAIN_CONTRACT §3; fallback по CLIENT_INTEGRATION.
Без look-ahead: на баре t только данные до t включительно.
"""
from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path

# brain_step из того же каталога
sys.path.insert(0, str(Path(__file__).resolve().parent))
from brain_http_client import brain_step

try:
    import pandas as pd
except ImportError:
    print("Требуется pandas: pip install pandas pyarrow", file=sys.stderr)
    sys.exit(1)


# Размер вектора наблюдения по контракту (недостающие — нули)
OBS_SIZE = 10

# Маппинг action мозга: 0 = hold, 1 = long, 2 = short (если мозг отдаёт только 0/1 — 1 трактуем как long)
ACTION_NAMES = ("hold", "long", "short")

# Нормировка reward (PnL за шаг) в [-1, 1]: pnl_pct / REWARD_SCALE обрезается до ±1 (TRADING_REWARD_AND_DATA §6)
REWARD_SCALE = 0.02  # 2% move -> reward ±1


def load_ohlcv(
    data_path: Path,
    pair: str,
    timeframe: str,
    start_date: str | None = None,
    end_date: str | None = None,
) -> pd.DataFrame:
    """Загружает OHLCV из файла или каталога sifs_ft/data/binance. §9: опционально обрезка по датам (train/val)."""
    if data_path.is_file():
        path = data_path
    else:
        # Freqtrade: XRP_USDT-15m-spot.feather
        fname = f"{pair}-{timeframe}-spot.feather"
        path = data_path / fname
    if not path.exists():
        raise FileNotFoundError(f"Данные не найдены: {path}")
    df = pd.read_feather(path)
    df = df.sort_values("date").reset_index(drop=True)
    if "date" not in df.columns:
        return df
    # §9: фильтр по датам (train/val split по времени)
    if start_date:
        try:
            t0 = pd.Timestamp(start_date)
            df = df[df["date"] >= t0]
        except Exception as e:
            raise ValueError(f"Неверный --start_date {start_date!r}: {e}") from e
    if end_date:
        try:
            t1 = pd.Timestamp(end_date)
            df = df[df["date"] <= t1]
        except Exception as e:
            raise ValueError(f"Неверный --end_date {end_date!r}: {e}") from e
    df = df.reset_index(drop=True)
    return df


def bar_to_observation(row: pd.Series, prev_close: float | None) -> list[float]:
    """
    Один бар → вектор float длины OBS_SIZE (нормализовано ~[0,1] или симметрично).
    Без look-ahead: только текущий бар и prev_close (предыдущий close).
    """
    o, h, l, c = float(row["open"]), float(row["high"]), float(row["low"]), float(row["close"])
    vol = float(row["volume"])
    obs = []
    # 1) return (close - open) / open, ограничим ±0.1 → нормируем в ~[0,1]
    if o > 0:
        ret = (c - o) / o
        obs.append(max(0.0, min(1.0, (ret + 0.1) / 0.2)))
    else:
        obs.append(0.5)
    # 2) (high - low) / close — волатильность бара
    if c > 0:
        obs.append(min(1.0, (h - l) / c * 10.0))
    else:
        obs.append(0.0)
    # 3) return по close к prev_close (если есть)
    if prev_close is not None and prev_close > 0:
        r = (c / prev_close) - 1.0
        obs.append(max(0.0, min(1.0, (r + 0.05) / 0.1)))
    else:
        obs.append(0.5)
    # 4) объём — лог-нормализация в [0,1]
    obs.append(min(1.0, math.log1p(vol) / 20.0))
    # Дополняем нулями до OBS_SIZE (контракт §3.1)
    while len(obs) < OBS_SIZE:
        obs.append(0.0)
    return obs[:OBS_SIZE]


def pnl_to_reward(pnl_pct: float, scale: float = REWARD_SCALE) -> float:
    """Нормировка PnL за шаг в [-1, 1] для контракта мозга (TRADING_REWARD_AND_DATA §6)."""
    if scale <= 0:
        return 0.0
    return max(-1.0, min(1.0, pnl_pct / scale))


def run_loop(
    df: pd.DataFrame,
    url: str,
    timeout_sec: float,
    fallback: str,
    max_bars: int,
    log_path: Path | None,
    reward_scale: float = REWARD_SCALE,
    pair: str = "",
    timeframe: str = "",
) -> None:
    """Проход по барам: наблюдение + reward (PnL за предыдущий шаг) → brain_step → логирование."""
    prev_close = None
    prev_position = 0  # 0=hold, 1=long, 2=short
    reward_to_send = 0.0  # на первом баре reward=0; далее — нормированный PnL за предыдущий бар
    cumulative_pnl_pct = 0.0
    n_bars = min(len(df), max_bars)
    counts = [0, 0, 0]  # hold, long, short
    log_lines = []
    for i in range(n_bars):
        row = df.iloc[i]
        obs = bar_to_observation(row, prev_close)
        current_close = float(row["close"])
        # PnL за предыдущий бар: при long +return, при short -return, при hold 0
        if prev_close is not None and prev_close > 0:
            bar_return = (current_close - prev_close) / prev_close
            if prev_position == 1:
                pnl_pct = bar_return
            elif prev_position == 2:
                pnl_pct = -bar_return
            else:
                pnl_pct = 0.0
            cumulative_pnl_pct += pnl_pct
            reward_to_send = pnl_to_reward(pnl_pct, reward_scale)
        try:
            result = brain_step(url, obs, reward=reward_to_send, timeout_sec=timeout_sec)
            action = result.get("action", 0)
            readout = result.get("readout", {})
        except Exception as e:
            if fallback == "pause":
                action = 0  # hold
                readout = {"error": str(e)}
            elif fallback == "fail":
                print(json.dumps({"error": str(e), "bar_index": i}), file=sys.stderr)
                raise
            else:  # sifs_only — тоже hold без мозга
                action = 0
                readout = {"error": str(e)}
        ts = row["date"].isoformat() if hasattr(row["date"], "isoformat") else str(row["date"])
        rec = {
            "t": ts,
            "bar_index": i,
            "reward_sent": reward_to_send,
            "action": action,
            "action_name": ACTION_NAMES[action] if action < len(ACTION_NAMES) else "?",
            "readout": readout,
        }
        log_lines.append(rec)
        print(json.dumps(rec))
        if action < len(counts):
            counts[action] += 1
        prev_close = current_close
        prev_position = action
    if log_path:
        log_path.write_text("\n".join(json.dumps(r) for r in log_lines), encoding="utf-8")
        print(f"\nLog written: {log_path}", file=sys.stderr)
    # Сводка по прогону: stderr для человека, одна строка JSON в stdout для пайпов (§9: даты для воспроизводимости)
    first_ts = df.iloc[0]["date"] if n_bars else None
    last_ts = df.iloc[n_bars - 1]["date"] if n_bars else None
    first_str = first_ts.isoformat() if hasattr(first_ts, "isoformat") else str(first_ts)
    last_str = last_ts.isoformat() if hasattr(last_ts, "isoformat") else str(last_ts)
    summary = {
        "summary": {
            "bars": n_bars,
            "pair": pair or None,
            "timeframe": timeframe or None,
            "hold": counts[0],
            "long": counts[1],
            "short": counts[2],
            "cumulative_pnl_pct": round(cumulative_pnl_pct, 6),
            "first_bar_date": first_str,
            "last_bar_date": last_str,
        }
    }
    print(json.dumps(summary))
    print("\n--- D.3 run summary ---", file=sys.stderr)
    print(f"  bars: {n_bars}", file=sys.stderr)
    print(f"  date range: {first_str} .. {last_str}", file=sys.stderr)
    print(f"  hold: {counts[0]}, long: {counts[1]}, short: {counts[2]}", file=sys.stderr)
    print(f"  cumulative_pnl_pct: {cumulative_pnl_pct:.4f}", file=sys.stderr)
    print("------------------------", file=sys.stderr)


def main() -> int:
    parser = argparse.ArgumentParser(description="D.3: bar -> brain -> action (historical OHLCV)")
    parser.add_argument("--data", type=Path, default=None, help="Path to .feather or sifs_ft/data/binance")
    parser.add_argument("--pair", type=str, default="XRP_USDT", help="Pair e.g. XRP_USDT")
    parser.add_argument("--timeframe", type=str, default="15m", help="Timeframe e.g. 15m")
    parser.add_argument("--url", type=str, default="http://127.0.0.1:3030", help="Brain URL (POST /step)")
    parser.add_argument("--timeout_ms", type=int, default=400, help="Request timeout ms")
    parser.add_argument("--fallback", type=str, default="pause", choices=("pause", "sifs_only", "fail"), help="If brain unavailable")
    parser.add_argument("--max_bars", type=int, default=1000, help="Max bars to run")
    parser.add_argument("--log", type=Path, default=None, help="Log file path (JSON lines)")
    parser.add_argument("--reward_scale", type=float, default=REWARD_SCALE, help="PnL norm scale for reward in [-1,1] (default 0.02)")
    parser.add_argument("--start_date", type=str, default=None, help="§9: начало диапазона дат (ISO, e.g. 2024-01-01)")
    parser.add_argument("--end_date", type=str, default=None, help="§9: конец диапазона дат (ISO, e.g. 2024-06-30)")
    args = parser.parse_args()
    if args.data is None:
        # По умолчанию: sifs_ft/data/binance относительно workspace (Projects)
        workspace = Path(__file__).resolve().parents[2]  # Genesis/scripts -> Genesis -> Projects
        args.data = workspace / "sifs_ft" / "data" / "binance"
    df = load_ohlcv(
        args.data,
        args.pair,
        args.timeframe,
        start_date=args.start_date,
        end_date=args.end_date,
    )
    print(f"Загружено баров: {len(df)}, пара={args.pair}, tf={args.timeframe}", file=sys.stderr)
    if args.start_date or args.end_date:
        print(f"  §9 диапазон: --start_date {args.start_date!r} --end_date {args.end_date!r}", file=sys.stderr)
    run_loop(
        df,
        args.url,
        timeout_sec=args.timeout_ms / 1000.0,
        fallback=args.fallback,
        max_bars=args.max_bars,
        log_path=args.log,
        reward_scale=args.reward_scale,
        pair=args.pair,
        timeframe=args.timeframe,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
