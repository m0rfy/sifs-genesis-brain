#!/usr/bin/env python3
"""
Регрессионный тест: I_SIFS в Python по той же формуле, что и в Rust (fixed-point, sigma).
Используется для проверки числовой идентичности с sifs-genesis-core::calculate_sifs_current.
Константы синхронны с core.py и BRAIN_CONTRACT.
"""
import json
import math

PI = math.pi
K = 1.0 / (PI * PI)
PHI = (1 + math.sqrt(5)) / 2
SIFS_LEVELS = 10
FIXED_ONE = 1 << 16


def f32_to_fixed(x: float) -> int:
    return int(x * FIXED_ONE)


def fixed_to_f32(x: int) -> float:
    return x / FIXED_ONE


def sifs_w(n: int) -> int:
    w = math.exp(-2.0 * K * n)
    return f32_to_fixed(w)


def sifs_threshold(n: int, v0: int) -> int:
    phi_n = PHI**n
    return int(v0 / phi_n)


def calculate_sifs_current_python(
    input_voltage: int,
    sifs_weights: list[int],
    sifs_thresholds: list[int],
) -> int:
    """Точная копия логики Rust: sigma piecewise, contrib = (w * sigma) >> 16."""
    i_sifs = 0
    for n in range(SIFS_LEVELS):
        v_diff = input_voltage - sifs_thresholds[n]
        if v_diff < -2 * FIXED_ONE:
            sigma = 0
        elif v_diff > 2 * FIXED_ONE:
            sigma = FIXED_ONE
        else:
            sigma = (FIXED_ONE // 2) + (v_diff // 4)
        contrib = (sifs_weights[n] * sigma) >> 16
        i_sifs += contrib
    return max(0, min(i_sifs, 0x7FFFFFFF))


def main() -> None:
    v0 = f32_to_fixed(0.02)
    weights = [sifs_w(n) for n in range(SIFS_LEVELS)]
    thresholds = [sifs_threshold(n, v0) for n in range(SIFS_LEVELS)]

    test_voltages = [0.01, 0.02, 0.03, 0.005, 0.025]
    results = []
    for v in test_voltages:
        v_fixed = f32_to_fixed(v)
        i_sifs = calculate_sifs_current_python(v_fixed, weights, thresholds)
        results.append({"v": v, "i_sifs_fixed": i_sifs, "i_sifs_float": fixed_to_f32(i_sifs)})

    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
