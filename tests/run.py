"""Runs every scenario of the XEstates catalogs on SQLite; exits with 1 when any of them fails.

    python3 tests/run.py                     every module
    python3 tests/run.py stanowisko          only the named modules

The catalogs are read from spec/ of this XBoard.

Every module in this directory with a list `SCENARIOS` runs, so a new catalog only adds its file.
"""

import importlib
import sys
import traceback
from pathlib import Path

NOT_SCENARIO_MODULES = {"room", "run"}


def scenario_modules(names):
    """The modules beside this file that hold scenarios, by file name; only `names` when given."""
    modules = []

    for path in sorted(Path(__file__).resolve().parent.glob("*.py")):
        if path.stem in NOT_SCENARIO_MODULES:
            continue
        if names and path.stem not in names:
            continue

        module = importlib.import_module(path.stem)
        if hasattr(module, "SCENARIOS"):
            modules.append(module)

    return modules


def main():
    names = sys.argv[1:]
    modules = scenario_modules(names)

    unknown_names = set(names) - {module.__name__ for module in modules}
    if unknown_names:
        print(f"no scenario module {', '.join(sorted(unknown_names))}")
        return 1

    failures = 0

    for module in modules:
        for title, scenario in module.SCENARIOS:
            try:
                scenario()
            # Any failure of one scenario is reported and the run goes on to the next one.
            except Exception:  # noqa: BLE001
                failures += 1
                print(f"FAIL  {title}")
                traceback.print_exc(limit=3)
                continue

            print(f"ok    {title}")

    total = sum(len(module.SCENARIOS) for module in modules)
    print(f"\n{total - failures} of {total} scenarios passed")

    return 1 if failures > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
