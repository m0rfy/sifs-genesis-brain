#!/usr/bin/env python3
"""
CartPole + SIFS-Genesis brain as policy (standalone agent).
Runs sifs-genesis-agent binary: each step sends observation (JSON line),
receives { "readout": {...}, "action": 0|1 }; action = left(0) / right(1).
Requires: gymnasium, built binary target/release/sifs-genesis-agent[.exe].
"""

import json
import subprocess
import sys
from pathlib import Path

# CartPole typical ranges for normalization to ~[0,1]
CART_POS_RANGE = (-2.4, 2.4)
CART_VEL_RANGE = (-3.0, 3.0)
POLE_ANGLE_RANGE = (-0.42, 0.42)
POLE_VEL_RANGE = (-2.0, 2.0)


def normalize_obs(obs: list[float]) -> list[float]:
    """Map 4D CartPole obs to [0,1] for brain input."""
    def norm(x: float, lo: float, hi: float) -> float:
        return max(0.0, min(1.0, (x - lo) / (hi - lo) if hi != lo else 0.5))

    return [
        norm(obs[0], *CART_POS_RANGE),
        norm(obs[1], *CART_VEL_RANGE),
        norm(obs[2], *POLE_ANGLE_RANGE),
        norm(obs[3], *POLE_VEL_RANGE),
    ]


def find_agent_binary() -> Path:
    root = Path(__file__).resolve().parents[2]
    suffix = ".exe" if sys.platform == "win32" else ""
    for name in (
        "target/release/sifs-genesis-agent",
        "target/debug/sifs-genesis-agent",
    ):
        p = root / (name + suffix)
        if p.exists():
            return p
    return root / ("sifs-genesis-agent" + suffix)


def run_episode(env, process, steps_per_obs: int = 1) -> float:
    obs, _ = env.reset()
    total_reward = 0.0
    while True:
        obs_norm = normalize_obs(obs.tolist())
        # Agent expects one JSON array per line
        line = json.dumps(obs_norm) + "\n"
        process.stdin.write(line)
        process.stdin.flush()
        out_line = process.stdout.readline()
        if not out_line:
            break
        data = json.loads(out_line.strip())
        action = data.get("action", 0)
        obs, reward, terminated, truncated, _ = env.step(action)
        total_reward += float(reward)
        if terminated or truncated:
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
            f"Build the agent first: cargo build --release (expected: {binary})",
            file=sys.stderr,
        )
        sys.exit(1)

    n_neurons = 1000
    steps_per_obs = 1
    episodes = 5
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
        if args[i] == "--episodes" and i + 1 < len(args):
            episodes = int(args[i + 1])
            i += 2
            continue
        i += 1

    cmd = [
        str(binary),
        "--neurons",
        str(n_neurons),
        "--steps",
        str(steps_per_obs),
    ]
    process = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,
        cwd=str(Path(__file__).resolve().parents[2]),
    )

    env = gym.make("CartPole-v2")
    rewards: list[float] = []
    try:
        for ep in range(episodes):
            r = run_episode(env, process, steps_per_obs)
            rewards.append(r)
            print(f"Episode {ep + 1}: reward = {r:.0f}")
    finally:
        process.terminate()
        env.close()

    if rewards:
        print(f"\nMean reward ({len(rewards)} episodes): {sum(rewards) / len(rewards):.1f}")
    print("Genesis + SIFS agent demo done.")


if __name__ == "__main__":
    main()
