#!/usr/bin/env python3
"""
Печатает чеклист для референс-прогона B.1 в genesis-agi (NEXT_PLAN).
Запуск: из Genesis/ — python scripts/print_b1_reference_steps.py
"""
from pathlib import Path

GENESIS_ROOT = Path(__file__).resolve().parents[1]
REFERENCE_DOC = GENESIS_ROOT / "experiments" / "B1_genesis_agi_reference.md"

def main() -> None:
    print("B.1 Референс genesis-agi — чеклист (NEXT_PLAN)\n" + "=" * 50)
    print("1. Открыть репо genesis-agi (рядом с Projects).")
    print("2. Поднять 2 ноды по RUN_WINDOWS.md (baker + node, 8081/8092).")
    print("3. Из корня genesis-agi:")
    print("   python examples/cartpole/cartpole_client.py --seed 42 --episodes 50")
    print("4. Скопировать Mean reward и Median reward из вывода.")
    print("5. Заполнить таблицу в:", REFERENCE_DOC.name)
    print("   Путь:", REFERENCE_DOC)
    print("6. Сравнить с нашей медианой 10 (steps=10, night=200).")
    print("\nПодробнее см.", REFERENCE_DOC)

if __name__ == "__main__":
    main()
