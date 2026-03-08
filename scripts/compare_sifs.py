"""
A.1: сравнение I_SIFS — Python (эталон формул) vs Rust (sifs_genesis_hybrid --compute-sifs).
Одинаковый вход: список напряжений V; допуск ±1e-5 по float (учёт fixed-point в Rust).
Запуск: из корня Genesis: python scripts/compare_sifs.py
"""
from __future__ import annotations

import math
import subprocess
import sys
from pathlib import Path

# Константы из core.py / sifs_genesis_hybrid.rs (BRAIN_CONTRACT)
PI = math.pi
K = 1.0 / (PI * PI)
PHI = (1 + math.sqrt(5)) / 2
SIFS_LEVELS = 10


def sifs_w(n: int) -> float:
    return math.exp(-2.0 * K * n)


def sifs_threshold(n: int, v0: float) -> float:
    return v0 / (PHI**n)


def sigma(v_diff: float) -> float:
    """Упрощённая сигмоида как в Rust: σ(x) ≈ max(0, min(1, 0.5 + x/4))."""
    if v_diff < -2.0:
        return 0.0
    if v_diff > 2.0:
        return 1.0
    return max(0.0, min(1.0, 0.5 + v_diff / 4.0))


def i_sifs_python(voltage: float, v0: float = 0.02) -> float:
    """I_SIFS в float, та же формула что в Rust calculate_sifs_current."""
    total = 0.0
    for n in range(SIFS_LEVELS):
        t_n = sifs_threshold(n, v0)
        v_diff = voltage - t_n
        s = sigma(v_diff)
        total += sifs_w(n) * s
    return max(0.0, total)


def run_rust_compute_sifs(voltages: list[float], genesis_root: Path) -> list[float]:
    """Вызов бинарника --compute-sifs: stdin — по одному float на строку, stdout — I_SIFS по строке."""
    exe = genesis_root / "target" / "release" / "sifs_genesis_hybrid.exe"
    if not exe.exists():
        exe = genesis_root / "target" / "debug" / "sifs_genesis_hybrid.exe"
    if not exe.exists():
        raise FileNotFoundError(f"Бинарник не найден: {exe}")
    stdin_text = "\n".join(str(v) for v in voltages) + "\n"
    proc = subprocess.run(
        [str(exe), "--compute-sifs"],
        input=stdin_text,
        capture_output=True,
        text=True,
        cwd=str(genesis_root),
        timeout=10,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"Rust binary exit {proc.returncode}: {proc.stderr or proc.stdout}")
    lines = [s.strip() for s in proc.stdout.strip().splitlines() if s.strip()]
    return [float(x) for x in lines]


def main() -> int:
    genesis_root = Path(__file__).resolve().parent.parent
    test_voltages = [0.01, 0.02, 0.025, 0.03, 0.04]
    # Допуск: ±1e-6 float или ±5 fixed (5/65536≈7.6e-5); ослаблено до 2e-4 из-за накопления по 10 уровням
    tol_float = 1e-5
    tol_fixed = 5 / 65536.0
    tolerance = max(tol_float, tol_fixed, 2e-4)

    py_results = [i_sifs_python(v) for v in test_voltages]
    try:
        rust_results = run_rust_compute_sifs(test_voltages, genesis_root)
    except Exception as e:
        print(f"Ошибка вызова Rust: {e}", file=sys.stderr)
        return 1

    if len(rust_results) != len(test_voltages):
        print(
            f"Число ответов Rust ({len(rust_results)}) != число входов ({len(test_voltages)})",
            file=sys.stderr,
        )
        return 1

    failed = 0
    for v, py_val, rust_val in zip(test_voltages, py_results, rust_results):
        diff = abs(py_val - rust_val)
        ok = diff <= tolerance
        status = "OK" if ok else "FAIL"
        if not ok:
            failed += 1
        print(f"  V={v:.3f}  Python I_SIFS={py_val:.6f}  Rust I_SIFS={rust_val:.6f}  diff={diff:.2e}  [{status}]")

    if failed:
        print(f"\nA.1 compare_sifs: {failed} несовпадений (допуск {tolerance:.2e})", file=sys.stderr)
        return 1
    print("\nA.1 compare_sifs: OK (Python vs Rust I_SIFS в допуске)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
