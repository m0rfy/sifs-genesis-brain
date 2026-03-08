#!/usr/bin/env python3
"""
D.3 бэктест по §9: те же .feather и логика, что trading_brain_loop; без look-ahead.
Выводит краткий отчёт (train/val диапазон, пара, таймфрейм, cumulative_pnl_pct) и одну строку JSON summary.
Опционально: --equity_csv для кривой equity; --decisions_csv — таблица решений (bar_index, date, action, action_name) для сравнения с SIFS-only.
"""
from __future__ import annotations

import argparse
import csv
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from trading_brain_loop import (
    REWARD_SCALE,
    ACTION_NAMES,
    load_ohlcv,
    bar_to_observation,
    pnl_to_reward,
)
from brain_http_client import brain_step


def run_backtest(
    df,
    url: str,
    timeout_sec: float,
    fallback: str,
    max_bars: int,
    reward_scale: float,
    pair: str,
    timeframe: str,
    equity_csv_path: Path | None,
    decisions_csv_path: Path | None,
    verbose: bool,
) -> dict:
    """Прогон по барам без look-ahead; возвращает summary и при verbose — список записей по барам."""
    prev_close = None
    prev_position = 0
    reward_to_send = 0.0
    cumulative_pnl_pct = 0.0
    n_bars = min(len(df), max_bars)
    counts = [0, 0, 0]
    equity_rows = []
    decisions_rows = []

    for i in range(n_bars):
        row = df.iloc[i]
        obs = bar_to_observation(row, prev_close)
        current_close = float(row["close"])
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
        except Exception:
            if fallback == "fail":
                raise
            action = 0
        if action < len(counts):
            counts[action] += 1
        ts = row["date"].isoformat() if hasattr(row["date"], "isoformat") else str(row["date"])
        action_name = ACTION_NAMES[action] if action < len(ACTION_NAMES) else "?"
        if equity_csv_path is not None:
            equity_rows.append((i, ts, action, round(cumulative_pnl_pct, 6)))
        if decisions_csv_path is not None:
            decisions_rows.append((i, ts, action, action_name))
        prev_close = current_close
        prev_position = action

    first_ts = df.iloc[0]["date"] if n_bars else None
    last_ts = df.iloc[n_bars - 1]["date"] if n_bars else None
    first_str = first_ts.isoformat() if hasattr(first_ts, "isoformat") else str(first_ts)
    last_str = last_ts.isoformat() if hasattr(last_ts, "isoformat") else str(last_ts)
    summary = {
        "bars": n_bars,
        "pair": pair,
        "timeframe": timeframe,
        "first_bar_date": first_str,
        "last_bar_date": last_str,
        "hold": counts[0],
        "long": counts[1],
        "short": counts[2],
        "cumulative_pnl_pct": round(cumulative_pnl_pct, 6),
    }
    if equity_csv_path:
        with open(equity_csv_path, "w", newline="", encoding="utf-8") as f:
            w = csv.writer(f)
            w.writerow(["bar_index", "date", "position", "cumulative_pnl_pct"])
            w.writerows(equity_rows)
    if decisions_csv_path:
        with open(decisions_csv_path, "w", newline="", encoding="utf-8") as f:
            w = csv.writer(f)
            w.writerow(["bar_index", "date", "action", "action_name"])
            w.writerows(decisions_rows)
    return summary


def main() -> int:
    parser = argparse.ArgumentParser(description="D.3 backtest §9: bar -> brain -> action, report + optional equity CSV")
    parser.add_argument("--data", type=Path, default=None, help="Path to .feather or sifs_ft/data/binance")
    parser.add_argument("--pair", type=str, default="XRP_USDT")
    parser.add_argument("--timeframe", type=str, default="15m")
    parser.add_argument("--start_date", type=str, default=None, help="§9 train/val start (ISO)")
    parser.add_argument("--end_date", type=str, default=None, help="§9 train/val end (ISO)")
    parser.add_argument("--url", type=str, default="http://127.0.0.1:3030")
    parser.add_argument("--timeout_ms", type=int, default=400)
    parser.add_argument("--fallback", type=str, default="pause", choices=("pause", "sifs_only", "fail"))
    parser.add_argument("--max_bars", type=int, default=1000)
    parser.add_argument("--reward_scale", type=float, default=REWARD_SCALE)
    parser.add_argument("--equity_csv", type=Path, default=None, help="Write equity curve to CSV")
    parser.add_argument("--decisions_csv", type=Path, default=None, help="Write decisions table (bar_index, date, action, action_name) for comparison with SIFS-only")
    parser.add_argument("--verbose", action="store_true", help="Print per-bar lines to stderr")
    args = parser.parse_args()

    if args.data is None:
        workspace = Path(__file__).resolve().parents[2]
        args.data = workspace / "sifs_ft" / "data" / "binance"
    df = load_ohlcv(args.data, args.pair, args.timeframe, args.start_date, args.end_date)
    if args.verbose:
        print(f"Загружено баров: {len(df)}", file=sys.stderr)

    summary = run_backtest(
        df,
        args.url,
        timeout_sec=args.timeout_ms / 1000.0,
        fallback=args.fallback,
        max_bars=args.max_bars,
        reward_scale=args.reward_scale,
        pair=args.pair,
        timeframe=args.timeframe,
        equity_csv_path=args.equity_csv,
        decisions_csv_path=args.decisions_csv,
        verbose=args.verbose,
    )

    print(json.dumps({"summary": summary}))
    print("\n--- D.3 backtest report (§9) ---", file=sys.stderr)
    print(f"  pair: {summary['pair']}, timeframe: {summary['timeframe']}", file=sys.stderr)
    print(f"  date range: {summary['first_bar_date']} .. {summary['last_bar_date']}", file=sys.stderr)
    print(f"  bars: {summary['bars']}, hold: {summary['hold']}, long: {summary['long']}, short: {summary['short']}", file=sys.stderr)
    print(f"  cumulative_pnl_pct: {summary['cumulative_pnl_pct']}", file=sys.stderr)
    if args.equity_csv:
        print(f"  equity curve: {args.equity_csv}", file=sys.stderr)
    if args.decisions_csv:
        print(f"  decisions table: {args.decisions_csv}", file=sys.stderr)
    print("--------------------------------", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
