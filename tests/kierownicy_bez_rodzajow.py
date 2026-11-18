"""Scenarios of the memory rooms of positions without kinds of their own.

Such a position's room is only the shared kinds its board.toml imports by name: three or four kinds
of the agent memory and, for a department head, the kinds of the department memory it keeps. Their
rules are checked in stanowisko.py and kierownik_dzialu.py, so here each room is checked to build and to hold
exactly the declared kinds, no more (imports are not transitive) and no fewer. The transaction head
keeps no policy copies (D11), so its room has no kierownik-dzialu-polisy.
"""

from pathlib import Path

from kierownik_dzialu import (
    add_case_to_list,
    appoint_agent,
    appoint_controller,
    forward_request,
    let_in,
    receive_report,
)
from room import Room

POSITIONS_FOLDER = Path(__file__).resolve().parents[1] / "spec" / "positions"

# The kinds each room must hold, as its board.toml declares them.
DECLARED_KINDS = {
    "transakcje/kierownik_transakcji": [
        "stanowisko-polecenia",
        "stanowisko-ustalenia",
        "stanowisko-zrodla",
        "kierownik-dzialu-agenci",
        "kierownik-dzialu-raporty",
        "kierownik-dzialu-spis",
        "kierownik-dzialu-standardy",
        "kierownik-dzialu-zajrzenia",
        "kierownik-dzialu-zwyczaje",
    ],
}

# Every shared kind a room could import, to tell which of them a room holds.
SHARED_KINDS = [
    "stanowisko-kontakty",
    "stanowisko-polecenia",
    "stanowisko-ustalenia",
    "stanowisko-zrodla",
    "kierownik-dzialu-agenci",
    "kierownik-dzialu-polisy",
    "kierownik-dzialu-raporty",
    "kierownik-dzialu-spis",
    "kierownik-dzialu-standardy",
    "kierownik-dzialu-zajrzenia",
    "kierownik-dzialu-zwyczaje",
]

SCENARIOS = []


def scenario(title):
    def register(function):
        SCENARIOS.append((f"kierownicy_bez_rodzajow: {title}", function))

        return function

    return register


def room_of(position):
    return Room(POSITIONS_FOLDER / position)


def kinds_in(room):
    """The shared kinds whose tables the room has."""
    tables = [
        row["name"] for row in room.execute("SELECT name FROM sqlite_schema WHERE type = 'table'")
    ]

    return [
        kind
        for kind in SHARED_KINDS
        if any(table.startswith(kind.replace("-", "_") + "_") for table in tables)
    ]


@scenario("pokój każdego stanowiska bez rodzajów własnych ma dokładnie zadeklarowane rodzaje")
def rooms_hold_the_declared_kinds():
    for position, declared in DECLARED_KINDS.items():
        room = room_of(position)

        assert kinds_in(room) == declared, (position, kinds_in(room))
        assert not (POSITIONS_FOLDER / position / "meta").exists(), position


@scenario("Kierownik transakcji powołuje agenta, wpuszcza Kontrolera i przekazuje prośbę i spis")
def transaction_head_day():
    room = room_of("transakcje/kierownik_transakcji")

    appoint_agent(
        room,
        "koordynator-1",
        stanowisko="koordynator_transakcji",
        wlasciciel="tozsamosc-wlasciciela",
        teczka_odnosnik="pokoj-transakcji-1",
    )
    appoint_controller(room, "kontroler-sprzedajacej")
    let_in(
        room,
        tozsamosc="tozsamosc-kontroler-sprzedajacej",
        agent_id="kontroler-sprzedajacej",
        pokoj_odnosnik="pokoj-transakcji-1",
        wpuscil="tozsamosc-kierownika-transakcji",
    )
    receive_report(
        room,
        "raport-1",
        "koordynator-1",
        etap_albo_okres="umowa przedwstępna",
        decyzja_odnosnik="pokoj-transakcji-1/decyzja-1",
    )
    forward_request(room, decyzja_odnosnik="pokoj-transakcji-1/decyzja-1")
    add_case_to_list(
        room,
        teczka_odnosnik="pokoj-transakcji-1",
        rodzaj="transakcja",
        oznaczenie="Transakcja 2026/12, ul. Polna 7 m. 12",
        agent="tozsamosc-koordynator-1",
        kancelaria_xnotariusz=1,
    )

    assert [
        row["id"] for row in room.run("agenci_teczki", teczka_odnosnik="pokoj-transakcji-1")
    ] == ["koordynator-1"]
    assert len(room.run("wpuszczenia_otwarte")) == 1
    assert len(room.run("prosby_na_liscie")) == 1
    assert room.run("do_przekazania_recepcji")[0]["kancelaria_xnotariusz"] == 1
