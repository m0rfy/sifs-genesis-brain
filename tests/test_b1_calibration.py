"""
Тесты B.1: run_b1_calibration — парсинг вывода run_cartpole_agent, один прогон с моком.
"""
from __future__ import annotations

import re
from unittest.mock import patch

import run_b1_calibration as b1


class TestRunB1CalibrationParse:
    """Парсинг stdout run_cartpole_agent."""

    def test_parse_median_mean(self):
        stdout = (
            "Episode 1: reward = 10\n"
            "Episode 2: reward = 8\n"
            "\n"
            "Mean reward (2 episodes): 9.0\n"
            "Median reward: 9.0\n"
        )
        median = mean = float("nan")
        for line in stdout.splitlines():
            if m := re.search(r"Median reward:\s*([\d.]+)", line):
                median = float(m.group(1))
            if m := re.search(r"Mean reward \([\d]+ episodes\):\s*([\d.]+)", line):
                mean = float(m.group(1))
        assert median == 9.0
        assert mean == 9.0

    def test_run_one_mocked(self):
        fake_stdout = (
            "Episode 1: reward = 12\n"
            "Episode 2: reward = 10\n"
            "\nMean reward (2 episodes): 11.0\n"
            "Median reward: 11.0\n"
        )
        import subprocess
        with patch("run_b1_calibration.subprocess.run") as m:
            m.return_value = subprocess.CompletedProcess(
                args=[],
                returncode=0,
                stdout=fake_stdout,
                stderr="",
            )
            median, mean = b1.run_one(10, 200, episodes=2)
        assert median == 11.0
        assert mean == 11.0
        assert m.called
        call_args = m.call_args[0][0]
        assert "--episodes" in call_args and "2" in call_args
        assert "--night" in call_args and "200" in call_args
