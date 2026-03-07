#!/usr/bin/env python3
"""
Экспорт констант SIFS из core.py в JSON для Rust / тестов числовой идентичности.
Запуск: из корня Projects — python sifs-genesis-brain/scripts/export_sifs_constants.py
        или из sifs-genesis-brain — python scripts/export_sifs_constants.py
"""

import importlib.util
import json
import os
import sys

# Путь к core.py: либо рядом с Projects, либо через env
_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
_PROJECTS = os.path.abspath(os.path.join(_ROOT, ".."))
_CORE_PATH = os.path.join(_PROJECTS, "sifs_ft", "strategies", "modules", "core.py")

if not os.path.isfile(_CORE_PATH):
    _CORE_PATH = os.path.join(
        _ROOT, "..", "sifs_ft", "strategies", "modules", "core.py"
    )
if not os.path.isfile(_CORE_PATH):
    print(
        "Не найден core.py. Задайте CORE_PATH или запустите из корня Projects.",
        file=sys.stderr,
    )
    sys.exit(1)

sys.path.insert(0, os.path.dirname(_CORE_PATH))

spec = importlib.util.spec_from_file_location("core", _CORE_PATH)
core = importlib.util.module_from_spec(spec)
spec.loader.exec_module(core)

K = core.K
PHI = core.PHI
FIB = core.FIB
W = core.W

out = {
    "K": K,
    "PHI": PHI,
    "FIB": FIB,
    "W": W,
    "source": "core.py",
    "note": "Synchronize with docs/BRAIN_CONTRACT.md and Rust constants.",
}
out_path = os.path.join(_ROOT, "sifs_constants.json")
with open(out_path, "w", encoding="utf-8") as f:
    json.dump(out, f, indent=2, ensure_ascii=False)
print(f"Written {out_path}")
