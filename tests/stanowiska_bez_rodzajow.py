"""Scenarios of the memory rooms of the positions without kinds of their own.

Seven positions keep in their memory only what the shared agent memory offers; what else their
profiles name lies in the case file (D3). Their position has no `meta/`; its `board.toml` only
imports the shared memory. The rules of the shared kinds are checked in stanowisko.py; here each room is built
from its catalog and checked to hold exactly the kinds it declares, one row in every table.
"""

import tomllib
from pathlib import Path

from stanowisko import full_memory
from room import Room

POSITIONS_FOLDER = Path(__file__).resolve().parents[1] / "spec" / "positions"

SHARED_AGENT_MEMORY = "../../stanowisko/meta"

# Every position below needs contacts outside the case file, so it imports stanowisko-kontakty as well.
POSITIONS = [
    "posrednictwo/opiekun_oferty",
    "posrednictwo/opiekun_poszukujacego",
    "transakcje/analityk_kredytowy",
    "transakcje/koordynator_transakcji",
    "najem/opiekun_najmu",
    "najem/zarzadca_obiektu",
    "najem/koordynator_operacji",
]

DECLARED_KINDS = [
    f"{SHARED_AGENT_MEMORY}/stanowisko-kontakty",
    f"{SHARED_AGENT_MEMORY}/stanowisko-polecenia",
    f"{SHARED_AGENT_MEMORY}/stanowisko-ustalenia",
    f"{SHARED_AGENT_MEMORY}/stanowisko-zrodla",
]

TABLES = [
    "stanowisko_kontakty_wpisy",
    "stanowisko_polecenia_wpisy",
    "stanowisko_ustalenia_wpisy",
    "stanowisko_zrodla_wpisy",
    "stanowisko_zrodla_zdarzenia",
]

SCENARIOS = []


def scenario(title):
    def register(function):
        SCENARIOS.append((f"stanowiska_bez_rodzajow: {title}", function))

        return function

    return register


def room_has_declared_kinds(position):
    position_folder = POSITIONS_FOLDER / position

    declared = tomllib.loads((position_folder / "board.toml").read_text())["meta"]
    assert declared == DECLARED_KINDS, f"{position}: {declared}"

    has_own_kinds = (position_folder / "meta").exists()
    assert not has_own_kinds, f"{position} has a meta/ folder of its own"

    room = Room(position_folder)
    tables = [
        row["name"]
        for row in room.execute("SELECT name FROM sqlite_schema WHERE type = 'table' ORDER BY name")
    ]
    assert tables == TABLES, f"{position}: {tables}"

    full_memory(room)
    for table in tables:
        assert room.value(f"SELECT count(*) FROM {table}") == 1, f"{position}: {table}"


def register_position(position):
    @scenario(f"pokój {position} ma tylko zadeklarowane rodzaje pamięci wspólnej i się buduje")
    def check():
        room_has_declared_kinds(position)


for position in POSITIONS:
    register_position(position)
