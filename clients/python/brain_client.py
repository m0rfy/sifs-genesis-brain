"""
Python client for SIFS-Genesis brain.
Calls the sifs-genesis-core binary with --input (JSON array of floats) and --steps;
reads JSON readout from stdout.
Usage: python brain_client.py [path_to_binary] [--neurons N] [--steps K]
  stdin: one line = JSON array of input values (or empty for default 0.01 repeated).
  stdout: one line = JSON readout (spike_count_total, spike_count_per_shard, score, time_step).
"""

import json
import subprocess
import sys
from pathlib import Path
from typing import List, Optional, Union


def query_brain(
    input_vector: List[float],
    binary_path: Optional[Union[str, Path]] = None,
    steps: int = 1,
    n_neurons: int = 1000,
) -> dict:
    """
    Run brain for `steps` steps with given input, return readout dict.
    binary_path: path to sifs-genesis-core executable; if None, uses default relative path.
    """
    if binary_path is None:
        # From repo root: target/release/sifs-genesis-core[.exe] or target/debug/...
        root = Path(__file__).resolve().parents[2]
        suffix = ".exe" if sys.platform == "win32" else ""
        for name in ("target/release/sifs-genesis-core", "target/debug/sifs-genesis-core"):
            p = root / (name + suffix)
            if p.exists():
                binary_path = p
                break
        else:
            binary_path = "sifs-genesis-core" + suffix  # hope it's in PATH
    binary_path = str(binary_path)

    cmd = [
        binary_path,
        "--input",
        json.dumps(input_vector),
        "--steps",
        str(steps),
        "--neurons",
        str(n_neurons),
    ]
    result = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        timeout=60,
        cwd=str(Path(__file__).resolve().parents[2]),
    )
    if result.returncode != 0:
        raise RuntimeError(f"brain exited {result.returncode}: {result.stderr or result.stdout}")
    return json.loads(result.stdout.strip())


def main() -> None:
    binary = None
    steps = 1
    n_neurons = 1000
    args = sys.argv[1:]
    while args:
        if args[0] == "--neurons" and len(args) > 1:
            n_neurons = int(args[1])
            args = args[2:]
            continue
        if args[0] == "--steps" and len(args) > 1:
            steps = int(args[1])
            args = args[2:]
            continue
        if not args[0].startswith("-"):
            binary = args[0]
            args = args[1:]
            continue
        args = args[1:]

    try:
        line = sys.stdin.readline()
        if line.strip():
            input_vector = json.loads(line)
        else:
            input_vector = [0.01] * n_neurons
    except (json.JSONDecodeError, EOFError):
        input_vector = [0.01] * n_neurons

    out = query_brain(input_vector, binary_path=binary, steps=steps, n_neurons=n_neurons)
    print(json.dumps(out))


if __name__ == "__main__":
    main()
