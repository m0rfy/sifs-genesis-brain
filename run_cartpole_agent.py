#!/usr/bin/env python3
"""
CartPole + SIFS-Genesis Hybrid (Genesis/sifs_genesis_hybrid.rs) as policy.
Runs: sifs_genesis_hybrid --agent
  stdin: one JSON array of floats per line (observation)
  stdout: {"readout": {...}, "action": 0|1} per line; action = left(0) / right(1).
Requires: gymnasium. После изменений в Rust: cargo build --release (в Genesis/).
"""

import json
import subprocess
import sys
import threading
from pathlib import Path

# CartPole typical ranges for normalization to ~[0,1]
CART_POS_RANGE = (-2.4, 2.4)
CART_VEL_RANGE = (-3.0, 3.0)
POLE_ANGLE_RANGE = (-0.42, 0.42)
POLE_VEL_RANGE = (-2.0, 2.0)

# Population coding (по образцу genesis-agi): единиц на переменную, попеременное пополнение сегментов
UNITS_PER_VAR = 16
POPULATION_SIGMA = 2


def encode_population_float(
    value: float,
    min_val: float,
    max_val: float,
    n: int = UNITS_PER_VAR,
    sigma: int = POPULATION_SIGMA,
) -> list[float]:
    """
    Одна переменная → вектор из n активаций (Gaussian receptive field).
    sigma=2 → центр±2 получают 1.0, остальные 0.0.
    """
    norm = max(0.0, min(1.0, (value - min_val) / (max_val - min_val) if max_val != min_val else 0.5))
    center = (norm * (n - 1)) if n > 1 else 0
    center_i = int(round(center))
    return [
        1.0 if abs(i - center_i) <= sigma else 0.0
        for i in range(n)
    ]


def population_encode_obs(obs: list[float]) -> list[float]:
    """
    Попеременное пополнение: 4 переменные → 4 сегмента по UNITS_PER_VAR активаций.
    Порядок: cart_x, cart_v, pole_a, pole_av (как в genesis-agi).
    """
    cart_x, cart_v, pole_a, pole_av = obs
    return (
        encode_population_float(cart_x, *CART_POS_RANGE)
        + encode_population_float(cart_v, *CART_VEL_RANGE)
        + encode_population_float(pole_a, *POLE_ANGLE_RANGE)
        + encode_population_float(pole_av, *POLE_VEL_RANGE)
    )


def dopamine_shaped(
    pole_a: float,
    pole_av: float,
    terminated: bool,
    truncated: bool,
) -> float:
    """
    Dopamine signal по образцу genesis-agi (R-STDP steering).
    Upright pole -> positive; terminal -> negative; penalty за угловую скорость.
    Возвращает значение в ~[-1, 1] для передачи в мозг как reward.
    """
    if terminated or truncated:
        return -1.0
    # genesis-agi: (0.03 - abs(pole_a))*25000 - abs(pole_av)*5000, clamp [-32768, 32767]
    raw = (0.03 - abs(pole_a)) * 25000.0 - abs(pole_av) * 5000.0
    raw = max(-32768.0, min(32767.0, raw))
    return raw / 32768.0


def normalize_obs(obs: list[float]) -> list[float]:
    def norm(x: float, lo: float, hi: float) -> float:
        return max(0.0, min(1.0, (x - lo) / (hi - lo) if hi != lo else 0.5))

    return [
        norm(obs[0], *CART_POS_RANGE),
        norm(obs[1], *CART_VEL_RANGE),
        norm(obs[2], *POLE_ANGLE_RANGE),
        norm(obs[3], *POLE_VEL_RANGE),
    ]


def find_agent_binary() -> Path:
    """Genesis/ is the directory of this script."""
    root = Path(__file__).resolve().parent
    suffix = ".exe" if sys.platform == "win32" else ""
    for name in (
        "target/release/sifs_genesis_hybrid",
        "target/debug/sifs_genesis_hybrid",
    ):
        p = root / (name + suffix)
        if p.exists():
            return p
    return root / ("sifs_genesis_hybrid" + suffix)


def run_episode(
    env,
    process,
    steps_per_obs: int = 1,
    seed: int | None = None,
    dopamine_shaping: bool = False,
    population_coding: bool = False,
) -> float:
    if seed is not None:
        obs, _ = env.reset(seed=seed)
    else:
        obs, _ = env.reset()
    total_reward = 0.0
    prev_reward = 0.0  # B.2: доставка reward в мозг (dopamine)
    while True:
        if population_coding:
            obs_input = population_encode_obs(obs.tolist())  # попеременное пополнение, 4×16 = 64 float
        else:
            obs_input = normalize_obs(obs.tolist())
        payload = {"input": obs_input, "reward": prev_reward}
        line = json.dumps(payload) + "\n"
        process.stdin.write(line)
        process.stdin.flush()
        out_line = process.stdout.readline()
        if not out_line:
            break
        data = json.loads(out_line.strip())
        action = data.get("action", 0)
        obs, reward, terminated, truncated, _ = env.step(action)
        total_reward += float(reward)
        if dopamine_shaping:
            cart_x, cart_v, pole_a, pole_av = obs.tolist()
            prev_reward = dopamine_shaped(pole_a, pole_av, terminated, truncated)
        else:
            prev_reward = float(reward)
        if terminated or truncated:
            # B.1 рекомендация 3: night phase по концу эпизода с текущим reward
            if population_coding:
                obs_input_end = population_encode_obs(obs.tolist())
            else:
                obs_input_end = normalize_obs(obs.tolist())
            payload_end = {"input": obs_input_end, "reward": prev_reward, "episode_end": True}
            process.stdin.write(json.dumps(payload_end) + "\n")
            process.stdin.flush()
            process.stdout.readline()  # ответ мозга (1:1 протокол)
            break
    return total_reward


def main() -> None:
    try:
        import gymnasium as gym
    except ImportError:
        print("Install gymnasium: pip install gymnasium", file=sys.stderr)
        sys.exit(1)

    binary = find_agent_binary()
    if not binary.exists():
        print(
            f"Build first (in Genesis/): cargo build --release\nExpected: {binary}",
            file=sys.stderr,
        )
        sys.exit(1)

    n_neurons = 1000
    steps_per_obs = 1
    night_interval = 500
    episodes = 5
    seed: int | None = None
    no_sifs = False
    dopamine_shaping = False
    population_coding = False
    trainable_readout = False
    readout_lr = 0.05
    args = sys.argv[1:]
    i = 0
    while i < len(args):
        if args[i] == "--neurons" and i + 1 < len(args):
            n_neurons = int(args[i + 1])
            i += 2
            continue
        if args[i] == "--steps" and i + 1 < len(args):
            steps_per_obs = int(args[i + 1])
            i += 2
            continue
        if args[i] == "--night" and i + 1 < len(args):
            night_interval = int(args[i + 1])
            i += 2
            continue
        if args[i] == "--episodes" and i + 1 < len(args):
            episodes = int(args[i + 1])
            i += 2
            continue
        if args[i] == "--seed" and i + 1 < len(args):
            seed = int(args[i + 1])
            i += 2
            continue
        if args[i] == "--no-sifs":
            no_sifs = True
            i += 1
            continue
        if args[i] == "--dopamine-shaping":
            dopamine_shaping = True
            i += 1
            continue
        if args[i] == "--population-coding":
            population_coding = True
            i += 1
            continue
        if args[i] == "--trainable-readout":
            trainable_readout = True
            i += 1
            continue
        if args[i] == "--readout-lr" and i + 1 < len(args):
            readout_lr = float(args[i + 1])
            i += 2
            continue
        i += 1

    cmd = [
        str(binary),
        "--agent",
        "--neurons",
        str(n_neurons),
        "--steps",
        str(steps_per_obs),
        "--night",
        str(night_interval),
    ]
    if no_sifs:
        cmd.append("--no-sifs")
    if trainable_readout:
        cmd.append("--trainable-readout")
        cmd.extend(["--readout-lr", str(readout_lr)])
    process = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        bufsize=1,
        cwd=str(Path(__file__).resolve().parent),
    )

    def drain_stderr() -> None:
        if process.stderr:
            for line in process.stderr:
                print(line, end="", file=sys.stderr, flush=True)

    threading.Thread(target=drain_stderr, daemon=True).start()

    env = gym.make("CartPole-v1")
    rewards: list[float] = []
    try:
        for ep in range(episodes):
            ep_seed = (seed + ep) if seed is not None else None
            r = run_episode(
                env,
                process,
                steps_per_obs,
                seed=ep_seed,
                dopamine_shaping=dopamine_shaping,
                population_coding=population_coding,
            )
            rewards.append(r)
            print(f"Episode {ep + 1}: reward = {r:.0f}")
    finally:
        process.terminate()
        env.close()

    if rewards:
        mean_r = sum(rewards) / len(rewards)
        sorted_r = sorted(rewards)
        mid = len(sorted_r) // 2
        median_r = (sorted_r[mid] + sorted_r[mid - 1]) / 2 if len(sorted_r) % 2 == 0 else sorted_r[mid]
        print(f"\nMean reward ({len(rewards)} episodes): {mean_r:.1f}")
        print(f"Median reward: {median_r:.1f}")
        # B.1 рекомендация 5: медиана по последним 50 при длинных прогонах
        if len(rewards) >= 50:
            last50 = rewards[-50:]
            s50 = sorted(last50)
            m50 = len(s50) // 2
            median_last50 = (s50[m50] + s50[m50 - 1]) / 2 if len(s50) % 2 == 0 else s50[m50]
            print(f"Median reward (last 50): {median_last50:.1f}")
    print("Genesis + SIFS agent (Genesis/) done.")


if __name__ == "__main__":
    main()
