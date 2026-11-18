"""Scenarios of the rental head's memory room (spec/positions/najem/kierownik_najmu).

The room has the head's own kind of areas, three kinds of the agent memory and all seven kinds of
the department memory; the rules of the shared kinds are checked in stanowisko.py and kierownik_dzialu.py. Each
scenario starts from an empty room and checks one rule the database keeps: a write the rule allows
and a write it refuses. The first scenario follows an area from the head's proposal through a
human's approval to its operations coordinator and its flats.
"""

from pathlib import Path

from stanowisko import first_row_copy
from kierownik_dzialu import appoint_agent
from room import Room

KIEROWNIK_NAJMU = (
    Path(__file__).resolve().parents[1] / "spec" / "positions" / "najem" / "kierownik_najmu"
)

SCENARIOS = []

REPLACEMENT = "wiersza pamięci nie zastępuje się nowym wpisem"
DELETION = "wierszy pamięci się nie usuwa (R9)"


def scenario(title):
    def register(function):
        SCENARIOS.append((f"kierownik_najmu: {title}", function))

        return function

    return register


def new_room():
    return Room(KIEROWNIK_NAJMU)


# Building blocks ---------------------------------------------------------------------------------


def propose_area(room, **fields):
    room.run(
        "zaproponuj_obszar",
        **{
            "nazwa": "Stare Miasto",
            "granice": "Wrocław, w granicach fosy miejskiej",
            "zaproponowano": "2026-11-02T09:00:00Z",
            **fields,
        },
    )


def approve_area(room, area_id=1, **fields):
    room.run(
        "zatwierdz_obszar",
        **{
            "id": area_id,
            "zatwierdzil": "tozsamosc-wlasciciela",
            "podpis": "podpis-kluczem-wlasciciela",
            "zatwierdzono": "2026-11-02T12:00:00Z",
            **fields,
        },
    )


def appoint_coordinator(room, agent_id, **fields):
    appoint_agent(
        room,
        agent_id,
        **{
            "stanowisko": "koordynator_operacji",
            "wlasciciel": "tozsamosc-wlasciciela",
            "teczka_odnosnik": None,
            **fields,
        },
    )


def assign_flat(room, **fields):
    room.run(
        "przypisz_lokal_do_obszaru",
        **{
            "obszar_id": 1,
            "teczka_odnosnik": "pokoj-lokalu-1",
            "od": "2026-11-03",
            **fields,
        },
    )


def full_memory(room):
    """One row in every table of the head's own kind, for the checks of R9 and replacement."""
    propose_area(room)
    approve_area(room)
    assign_flat(room)


# A whole day ---------------------------------------------------------------------------------------


@scenario(
    "kierownik proponuje obszar, człowiek go zatwierdza, a obszar dostaje Koordynatora i lokale"
)
def whole_day():
    room = new_room()

    propose_area(room)
    room.run(
        "zmien_propozycje_obszaru",
        id=1,
        nazwa="Stare Miasto i Ostrów Tumski",
        granice="Wrocław, w granicach fosy miejskiej i Ostrów Tumski",
    )
    approve_area(room)
    appoint_coordinator(room, "koordynator-1")
    room.run("wskaz_koordynatora_obszaru", id=1, koordynator_id="koordynator-1")
    assign_flat(room)
    assign_flat(room, teczka_odnosnik="pokoj-lokalu-2")

    appoint_coordinator(room, "koordynator-2", powolano="2026-11-10T09:00:00Z")
    room.run(
        "wymien_agenta",
        id="koordynator-1",
        wymieniono="2026-11-10T09:00:00Z",
        powod_wymiany="Zlecał pracę wykonawcy spoza rejestru bez zagrożenia",
        nastepca_id="koordynator-2",
    )
    room.run("wskaz_koordynatora_obszaru", id=1, koordynator_id="koordynator-2")

    assert [row["teczka_odnosnik"] for row in room.run("lokale_obszaru", obszar_id=1)] == [
        "pokoj-lokalu-1",
        "pokoj-lokalu-2",
    ]
    assert room.run("obszar_lokalu", teczka_odnosnik="pokoj-lokalu-2")[0]["koordynator_id"] == (
        "koordynator-2"
    )
    assert room.run("lokale_bez_koordynatora") == []
    assert len(room.run("obszary_koordynatora", koordynator_id="koordynator-2")) == 1


# Areas ---------------------------------------------------------------------------------------------


@scenario("obszar powstaje jako propozycja, a zatwierdzenie jest kompletne i nie przed propozycją")
def area_starts_as_proposal():
    room = new_room()
    propose_area(room)

    room.refused_sql(
        "obszar powstaje jako propozycja kierownika",
        """
        INSERT INTO kierownik_najmu_obszary_wpisy (nazwa, granice, zaproponowano, zatwierdzil,
          podpis, zatwierdzono)
        VALUES ('Krzyki', 'g', '2026-11-02', 'w', 'p', '2026-11-02')
        """,
    )
    room.refused("zatwierdzenie_w_komplecie", "zatwierdz_obszar", id=1, zatwierdzil="w", podpis="p")
    room.refused(
        "zatwierdzenie_nie_przed_propozycja",
        "zatwierdz_obszar",
        id=1,
        zatwierdzil="w",
        podpis="p",
        zatwierdzono="2026-11-01T00:00:00Z",
    )
    approve_area(room)


@scenario("zatwierdzonych granic się nie zmienia, a zniesionego obszaru wcale")
def approved_boundaries_do_not_change():
    room = new_room()
    propose_area(room)
    room.run("zmien_propozycje_obszaru", id=1, nazwa="Stare Miasto", granice="Nowe granice")
    approve_area(room)

    rule = "zatwierdzonych granic obszaru się nie zmienia"
    room.refused(
        rule, "zmien_propozycje_obszaru", id=1, nazwa="Stare Miasto", granice="Inne granice"
    )
    room.refused(
        rule,
        "zatwierdz_obszar",
        id=1,
        zatwierdzil="ktos-inny",
        podpis="p",
        zatwierdzono="2026-11-03T00:00:00Z",
    )

    room.refused(
        "zniesienie_nie_przed_propozycja", "znies_obszar", id=1, zniesiono="2026-11-01T00:00:00Z"
    )
    room.run("znies_obszar", id=1, zniesiono="2026-12-01T00:00:00Z")
    room.refused(rule, "znies_obszar", id=1, zniesiono="2026-12-02T00:00:00Z")
    assert room.run("obszary")[0]["zniesiono"] == "2026-12-01T00:00:00Z"


@scenario("Koordynatorem jest niewymieniony Koordynator operacji, a obszar jest zatwierdzony")
def coordinator_is_an_operations_coordinator():
    room = new_room()
    propose_area(room)
    appoint_coordinator(room, "koordynator-1")
    appoint_coordinator(room, "koordynator-2")
    appoint_agent(
        room, "zarzadca-1", stanowisko="zarzadca_obiektu", teczka_odnosnik="pokoj-lokalu-1"
    )

    room.refused(
        "koordynator_obszaru_zatwierdzonego",
        "wskaz_koordynatora_obszaru",
        id=1,
        koordynator_id="koordynator-1",
    )
    approve_area(room)

    rule = "koordynatorem obszaru jest niewymieniony agent na stanowisku Koordynatora operacji"
    room.refused(rule, "wskaz_koordynatora_obszaru", id=1, koordynator_id="zarzadca-1")
    room.run(
        "wymien_agenta",
        id="koordynator-1",
        wymieniono="2026-11-05T09:00:00Z",
        powod_wymiany="Zawiódł",
        nastepca_id="koordynator-2",
    )
    room.refused(rule, "wskaz_koordynatora_obszaru", id=1, koordynator_id="koordynator-1")
    room.run("wskaz_koordynatora_obszaru", id=1, koordynator_id="koordynator-2")


@scenario("obszar znosi się dopiero, gdy nie ma w nim lokali")
def area_is_closed_without_flats():
    room = new_room()
    propose_area(room)
    approve_area(room)
    assign_flat(room)

    rule = "obszar znosi się dopiero, gdy nie ma w nim lokali"
    room.refused(rule, "znies_obszar", id=1, zniesiono="2026-12-01T00:00:00Z")
    room.run("zakoncz_przynaleznosc_lokalu", id=1, do_dnia="2026-11-30")
    room.run("znies_obszar", id=1, zniesiono="2026-12-01T00:00:00Z")


# Flats ---------------------------------------------------------------------------------------------


@scenario("lokal należy do zatwierdzonego, niezniesionego obszaru i naraz do jednego")
def flat_in_one_approved_area():
    room = new_room()
    propose_area(room)
    propose_area(room, nazwa="Krzyki", granice="Wrocław, Krzyki")
    propose_area(room, nazwa="Psie Pole", granice="Wrocław, Psie Pole")
    approve_area(room, area_id=1)
    approve_area(room, area_id=3)
    room.run("znies_obszar", id=3, zniesiono="2026-11-03T00:00:00Z")
    assign_flat(room)

    rule = "lokal należy tylko do obszaru zatwierdzonego i niezniesionego"
    room.refused(
        rule,
        "przypisz_lokal_do_obszaru",
        obszar_id=2,
        teczka_odnosnik="pokoj-lokalu-2",
        od="2026-11-03",
    )
    room.refused(
        rule,
        "przypisz_lokal_do_obszaru",
        obszar_id=3,
        teczka_odnosnik="pokoj-lokalu-2",
        od="2026-11-03",
    )

    approve_area(room, area_id=2)
    room.refused(
        "lokal należy naraz do jednego obszaru",
        "przypisz_lokal_do_obszaru",
        obszar_id=2,
        teczka_odnosnik="pokoj-lokalu-1",
        od="2026-11-10",
    )
    room.run("zakoncz_przynaleznosc_lokalu", id=1, do_dnia="2026-11-10")
    room.run(
        "przypisz_lokal_do_obszaru", obszar_id=2, teczka_odnosnik="pokoj-lokalu-1", od="2026-11-10"
    )
    assert room.run("obszar_lokalu", teczka_odnosnik="pokoj-lokalu-1")[0]["nazwa"] == "Krzyki"
    assert [row["teczka_odnosnik"] for row in room.run("lokale_bez_koordynatora")] == [
        "pokoj-lokalu-1"
    ]


@scenario("przynależność lokalu powstaje otwarta, kończy się raz i nie przed początkiem")
def flat_membership_ends_once():
    room = new_room()
    propose_area(room)
    approve_area(room)
    assign_flat(room)

    room.refused_sql(
        "lokal powstaje w obszarze",
        """
        INSERT INTO kierownik_najmu_obszary_lokale (obszar_id, teczka_odnosnik, od, do_dnia)
        VALUES (1, 'pokoj-lokalu-2', '2026-11-03', '2026-11-04')
        """,
    )
    room.refused(
        "koniec_nie_przed_poczatkiem", "zakoncz_przynaleznosc_lokalu", id=1, do_dnia="2026-11-01"
    )

    rule = "przynależności lokalu do obszaru się nie zmienia"
    room.refused_sql(
        rule, "UPDATE kierownik_najmu_obszary_lokale SET obszar_id = 1, od = '2026-11-01'"
    )
    room.run("zakoncz_przynaleznosc_lokalu", id=1, do_dnia="2026-11-30")
    room.refused(rule, "zakoncz_przynaleznosc_lokalu", id=1, do_dnia="2026-12-01")


# R9 and replacement --------------------------------------------------------------------------------


def own_tables_of(room):
    return [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' "
            "AND name LIKE 'kierownik\\_najmu\\_%' ESCAPE '\\'"
        )
    ]


@scenario("R9: wierszy pamięci się nie usuwa ani nie zastępuje przez REPLACE albo upsert")
def no_deletion_or_replacement():
    room = new_room()
    full_memory(room)

    tables = own_tables_of(room)
    assert len(tables) == 2, tables

    for table in tables:
        assert room.value(f"SELECT count(*) FROM {table}") > 0, f"{table} has no row"

        room.refused_sql(REPLACEMENT, f"INSERT OR REPLACE {first_row_copy(room, table)}")
        room.refused_sql(
            REPLACEMENT,
            f"INSERT {first_row_copy(room, table)} ON CONFLICT (id) DO UPDATE SET id = excluded.id",
        )
        room.refused_sql(
            DELETION, f"DELETE FROM {table} WHERE rowid = (SELECT min(rowid) FROM {table})"
        )


@scenario("REPLACE nie usuwa otwartej przynależności lokalu przez klucz UNIQUE")
def no_replacement_through_open_membership():
    room = new_room()
    full_memory(room)

    room.refused_sql(
        "lokal należy naraz do jednego obszaru",
        "INSERT OR REPLACE INTO kierownik_najmu_obszary_lokale (id, obszar_id, teczka_odnosnik, od) "
        "SELECT 99, obszar_id, teczka_odnosnik, od FROM kierownik_najmu_obszary_lokale",
    )
