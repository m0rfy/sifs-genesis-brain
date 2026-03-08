"""
Тесты для D.3: trading_brain_loop (bar_to_observation, pnl_to_reward, load_ohlcv) и run_d3_backtest.
Без реального мозга: brain_step мокается.
"""
from __future__ import annotations

import tempfile
from pathlib import Path

import pytest

# после conftest scripts в path
import pandas as pd

from trading_brain_loop import (
    OBS_SIZE,
    REWARD_SCALE,
    bar_to_observation,
    load_ohlcv,
    pnl_to_reward,
)


class TestPnlToReward:
    def test_zero(self):
        assert pnl_to_reward(0.0) == 0.0

    def test_clip_positive(self):
        assert pnl_to_reward(REWARD_SCALE) == 1.0
        assert pnl_to_reward(REWARD_SCALE * 2) == 1.0

    def test_clip_negative(self):
        assert pnl_to_reward(-REWARD_SCALE) == -1.0
        assert pnl_to_reward(-REWARD_SCALE * 2) == -1.0

    def test_scale_zero(self):
        assert pnl_to_reward(0.1, scale=0) == 0.0

    def test_custom_scale(self):
        assert pnl_to_reward(0.01, scale=0.01) == 1.0


class TestBarToObservation:
    def test_length(self):
        row = pd.Series({
            "open": 100.0, "high": 101.0, "low": 99.0, "close": 100.5, "volume": 1000.0,
        })
        obs = bar_to_observation(row, None)
        assert len(obs) == OBS_SIZE

    def test_with_prev_close(self):
        row = pd.Series({
            "open": 100.0, "high": 101.0, "low": 99.0, "close": 100.5, "volume": 1000.0,
        })
        obs = bar_to_observation(row, 100.0)
        assert len(obs) == OBS_SIZE
        assert all(0 <= x <= 1 for x in obs[:4])

    def test_all_in_range(self):
        row = pd.Series({
            "open": 1.0, "high": 2.0, "low": 0.5, "close": 1.5, "volume": 1.0,
        })
        obs = bar_to_observation(row, 1.0)
        assert all(0 <= x <= 1 for x in obs)


class TestLoadOhlcv:
    def test_load_from_file(self):
        with tempfile.TemporaryDirectory() as td:
            path = Path(td) / "XRP_USDT-15m-spot.feather"
            df = pd.DataFrame({
                "date": pd.to_datetime(["2024-01-01 00:00:00", "2024-01-01 00:15:00"]),
                "open": [1.0, 1.01],
                "high": [1.02, 1.03],
                "low": [0.99, 1.0],
                "close": [1.01, 1.02],
                "volume": [100.0, 200.0],
            })
            df.to_feather(path)
            loaded = load_ohlcv(path, "XRP_USDT", "15m")
            assert len(loaded) == 2
            assert list(loaded.columns) == ["date", "open", "high", "low", "close", "volume"]

    def test_filter_by_start_end_date(self):
        with tempfile.TemporaryDirectory() as td:
            path = Path(td) / "pair-15m-spot.feather"
            df = pd.DataFrame({
                "date": pd.to_datetime([
                    "2024-01-01 00:00:00",
                    "2024-01-15 00:00:00",
                    "2024-02-01 00:00:00",
                ]),
                "open": [1.0, 1.0, 1.0],
                "high": [1.0, 1.0, 1.0],
                "low": [1.0, 1.0, 1.0],
                "close": [1.0, 1.0, 1.0],
                "volume": [0.0, 0.0, 0.0],
            })
            df.to_feather(path)
            loaded = load_ohlcv(path, "pair", "15m", start_date="2024-01-10", end_date="2024-01-20")
            assert len(loaded) == 1
            assert str(loaded.iloc[0]["date"]).startswith("2024-01-15")

    def test_file_not_found(self):
        with tempfile.TemporaryDirectory() as td:
            d = Path(td)
            with pytest.raises(FileNotFoundError, match="Данные не найдены"):
                load_ohlcv(d, "NOPAIR", "15m")


class TestRunD3BacktestSmoke:
    """Smoke: run_backtest с моком brain_step, без HTTP."""

    def test_run_backtest_returns_summary(self):
        from unittest.mock import patch
        import run_d3_backtest as d3

        df = pd.DataFrame({
            "date": pd.to_datetime(["2024-01-01 00:00:00", "2024-01-01 00:15:00"]),
            "open": [1.0, 1.01],
            "high": [1.02, 1.03],
            "low": [0.99, 1.0],
            "close": [1.01, 1.02],
            "volume": [100.0, 200.0],
        })
        with patch("run_d3_backtest.brain_step", return_value={"action": 0, "readout": {}}):
            summary = d3.run_backtest(
                df,
                url="http://127.0.0.1:3030",
                timeout_sec=0.4,
                fallback="pause",
                max_bars=2,
                reward_scale=REWARD_SCALE,
                pair="XRP_USDT",
                timeframe="15m",
                equity_csv_path=None,
                decisions_csv_path=None,
                verbose=False,
            )
        assert summary["bars"] == 2
        assert summary["pair"] == "XRP_USDT"
        assert summary["timeframe"] == "15m"
        assert "cumulative_pnl_pct" in summary
        assert summary["hold"] == 2
