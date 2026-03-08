#!/usr/bin/env python3
"""
B.1 Калибровка CartPole: 50 эпизодов, сид 42.
Запускает run_cartpole_agent.py с заданными --steps и --night (или сетку),
парсит Median/Mean reward, выводит таблицу.
Из Genesis/: python scripts/run_b1_calibration.py [--steps N] [--night M]
Сетка: python scripts/run_b1_calibration.py --grid [--grid-steps 10,15,20] [--grid-night 100,200,300]
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

GENESIS_ROOT = Path(__file__).resolve().parents[1]
AGENT_SCRIPT = GENESIS_ROOT / "run_cartpole_agent.py"
B1_EXPERIMENTS = GENESIS_ROOT / "experiments"
DEFAULT_EPISODES = 50
SEED = 42


def run_one(
    steps: int,
    night: int,
    episodes: int = DEFAULT_EPISODES,
    no_sifs: bool = False,
    dopamine_shaping: bool = False,
    population_coding: bool = False,
    trainable_readout: bool = False,
    readout_lr: float = 0.05,
) -> tuple[float, float]:
    """Запуск episodes эп., сид 42; возвращает (median, mean)."""
    cmd = [
        sys.executable,
        str(AGENT_SCRIPT),
        "--episodes",
        str(episodes),
        "--seed",
        str(SEED),
        "--steps",
        str(steps),
        "--night",
        str(night),
    ]
    if no_sifs:
        cmd.append("--no-sifs")
    if dopamine_shaping:
        cmd.append("--dopamine-shaping")
    if population_coding:
        cmd.append("--population-coding")
    if trainable_readout:
        cmd.append("--trainable-readout")
        cmd.extend(["--readout-lr", str(readout_lr)])
    proc = subprocess.run(
        cmd,
        cwd=str(GENESIS_ROOT),
        capture_output=True,
        text=True,
        timeout=600,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"run_cartpole_agent failed: {proc.stderr[:500]}")
    median = mean = float("nan")
    for line in proc.stdout.splitlines():
        if m := re.search(r"Median reward:\s*([\d.]+)", line):
            median = float(m.group(1))
        if m := re.search(r"Mean reward \([\d]+ episodes\):\s*([\d.]+)", line):
            mean = float(m.group(1))
    return median, mean


def main() -> int:
    parser = argparse.ArgumentParser(description="B.1 CartPole calibration: 50 ep, seed 42")
    parser.add_argument("--steps", type=int, default=10, help="steps_per_observation (default 10)")
    parser.add_argument("--night", type=int, default=200, help="night_interval (default 200)")
    parser.add_argument("--no-sifs", action="store_true", help="Run with --no-sifs (A/B)")
    parser.add_argument(
        "--grid",
        action="store_true",
        help="Run grid: --grid-steps x --grid-night, output table",
    )
    parser.add_argument(
        "--grid-steps",
        type=str,
        default="10,15,20",
        help="Comma-separated steps for grid (default 10,15,20)",
    )
    parser.add_argument(
        "--grid-night",
        type=str,
        default="100,200,300",
        help="Comma-separated night for grid (default 100,200,300)",
    )
    parser.add_argument(
        "--episodes",
        type=int,
        default=DEFAULT_EPISODES,
        help=f"Episodes per run (default {DEFAULT_EPISODES} for B.1)",
    )
    parser.add_argument(
        "--out",
        type=Path,
        default=None,
        help="Write results table to file (e.g. experiments/B1_calibration_results.md)",
    )
    parser.add_argument(
        "--dopamine-shaping",
        action="store_true",
        help="Use dopamine shaping (genesis-agi style) for reward sent to brain",
    )
    parser.add_argument(
        "--population-coding",
        action="store_true",
        help="Encode observation as 4×16 population segments (alternating replenishment)",
    )
    parser.add_argument(
        "--trainable-readout",
        action="store_true",
        help="Use trainable readout weights (REINFORCE-style update by reward)",
    )
    parser.add_argument(
        "--readout-lr",
        type=float,
        default=0.05,
        help="Learning rate for trainable readout (default 0.05)",
    )
    args = parser.parse_args()

    if not AGENT_SCRIPT.exists():
        print(f"Not found: {AGENT_SCRIPT}", file=sys.stderr)
        return 1

    if args.grid:
        steps_list = [int(x) for x in args.grid_steps.split(",")]
        night_list = [int(x) for x in args.grid_night.split(",")]
        rows = [["steps", "night", "median", "mean"]]
        for steps in steps_list:
            for night in night_list:
                try:
                    median, mean = run_one(
                        steps,
                        night,
                        episodes=args.episodes,
                        no_sifs=args.no_sifs,
                        dopamine_shaping=args.dopamine_shaping,
                        population_coding=args.population_coding,
                        trainable_readout=args.trainable_readout,
                        readout_lr=args.readout_lr,
                    )
                    rows.append([str(steps), str(night), f"{median:.1f}", f"{mean:.1f}"])
                    print(f"  steps={steps} night={night} -> median={median:.1f} mean={mean:.1f}")
                except Exception as e:
                    rows.append([str(steps), str(night), "err", str(e)[:30]])
                    print(f"  steps={steps} night={night} -> error: {e}", file=sys.stderr)
        # Markdown table
        header = "| " + " | ".join(rows[0]) + " |"
        sep = "|" + "|".join("---" for _ in rows[0]) + "|"
        data_lines = ["| " + " | ".join(r) + " |" for r in rows[1:]]
        table = "\n".join([header, sep] + data_lines)
        print("\n" + table)
        if args.out:
            args.out.parent.mkdir(parents=True, exist_ok=True)
            header = f"# B.1 Калибровка: {args.episodes} эп., сид {SEED}" + (" (I_SIFS off)" if args.no_sifs else " (I_SIFS on)") + "\n\n"
            args.out.write_text(header + table + "\n", encoding="utf-8")
            print(f"Written: {args.out}", file=sys.stderr)
    else:
        median, mean = run_one(
            args.steps,
            args.night,
            episodes=args.episodes,
            no_sifs=args.no_sifs,
            dopamine_shaping=args.dopamine_shaping,
            population_coding=args.population_coding,
            trainable_readout=args.trainable_readout,
            readout_lr=args.readout_lr,
        )
        print(f"Median: {median:.1f}, Mean: {mean:.1f} ({args.episodes} ep, seed {SEED}, steps={args.steps}, night={args.night})")
        if args.out:
            args.out.parent.mkdir(parents=True, exist_ok=True)
            args.out.write_text(f"steps={args.steps} night={args.night} median={median:.1f} mean={mean:.1f}\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    sys.exit(main())
