"""Scenarios of the Concierge's memory room (spec/positions/najem/konsjerz).

The room is the Concierge's own kinds and the shared agent kinds it imports. The shared kinds are
checked in stanowisko.py; here each scenario starts from an empty room and checks one rule of
konsjerz-pytania or konsjerz-reguly: a write the rule allows and a write it refuses. The first
scenario follows a season of a flat: guests ask about the garage, the answer helps, a better answer
replaces it, the Concierge proposes a change of the flat's description and assesses a kind of
report instead of a guest.
"""

from pathlib import Path

from stanowisko import DELETION, REPLACEMENT, add_source, add_source_event, first_row_copy
from room import Room

KONSJERZ = Path(__file__).resolve().parents[1] / "spec" / "positions" / "najem" / "konsjerz"

SCENARIOS = []


def scenario(title):
    def register(function):
        SCENARIOS.append((f"konsjerz: {title}", function))

        return function

    return register


def new_room():
    return Room(KONSJERZ)


# Building blocks ---------------------------------------------------------------------------------


def save_question(room, **fields):
    room.run(
        "zapisz_pytanie",
        **{
            "temat": "wjazd do garażu",
            "odpowiedz": "Wjazd jest od ul. Bocznej; pilot leży w szufladzie przy drzwiach",
            "zapisano": "2026-07-01T10:00:00Z",
            **fields,
        },
    )


def add_occurrence(room, pytanie_id=1, zadano="2026-07-02T18:00:00Z"):
    room.run("dopisz_wystapienie_pytania", pytanie_id=pytanie_id, zadano=zadano)


# A whole season ----------------------------------------------------------------------------------


@scenario("sezon lokalu: pytania gości, lepsza odpowiedź, propozycja opisu i ocena zgłoszeń")
def whole_season():
    room = new_room()

    save_question(room)
    add_occurrence(room)
    add_occurrence(room, zadano="2026-07-09T20:00:00Z")
    room.run("odnotuj_czy_odpowiedz_pomogla", id=1, pomogla=0)
    room.run("odnotuj_czy_odpowiedz_pomogla", id=2, pomogla=0)
    save_question(
        room,
        odpowiedz="Wjazd od ul. Bocznej, druga brama za sklepem; pilot przy drzwiach",
        zapisano="2026-07-10T09:00:00Z",
    )
    room.run("zastap_odpowiedz", id=1, zastapione_przez_id=2)
    add_occurrence(room, pytanie_id=2, zadano="2026-07-12T17:00:00Z")
    room.run("odnotuj_czy_odpowiedz_pomogla", id=3, pomogla=1)
    save_question(
        room,
        temat="ciepła woda",
        odpowiedz="Wspólnota wyłącza wodę we wtorki 8–10",
        zapisano="2026-07-12T09:00:00Z",
    )
    room.run(
        "odnotuj_propozycje_opisu",
        id=2,
        propozycja_przekazano="2026-07-15T10:00:00Z",
        notatka_odnosnik="pokoj-teczki-lokalu/notatka-12",
    )

    add_source(room, "brak-wody", nazwa="zgłoszenie braku ciepłej wody", rodzaj="rodzaj_zgloszenia")
    add_source_event(
        room,
        "brak-wody",
        "rozbieznosc",
        opis="Zgłoszony brak wody był wyłączeniem przez wspólnotę",
    )
    room.run("ocen_zrodlo", id="brak-wody", ocena="zastrzezenia", oceniono="2026-07-16T10:00:00Z")

    questions = {
        row["temat"]: (row["id"], row["wystapienia"], row["pomocne"])
        for row in room.run("pytania_gosci")
    }
    assert questions == {
        "wjazd do garażu": (2, 1, 1),
        "ciepła woda": (3, 0, 0),
    }


# Questions ---------------------------------------------------------------------------------------


@scenario("pytanie powstaje aktualne i bez propozycji")
def question_starts_current():
    room = new_room()
    save_question(room)

    rule = "pytanie powstaje aktualne i bez propozycji"
    for columns, values in [
        (
            "propozycja_przekazano, notatka_odnosnik",
            "'2026-07-02', 'pokoj-teczki-lokalu/notatka-1'",
        ),
        ("zastapione_przez_id", "1"),
    ]:
        room.refused_sql(
            rule,
            f"""
            INSERT INTO konsjerz_pytania_wpisy (temat, odpowiedz, zapisano, {columns})
            VALUES ('parking', 'Parking pod budynkiem', '2026-07-01', {values})
            """,
        )


@scenario("propozycja ma notatkę, nie poprzedza zapisu i odnotowuje się ją raz")
def proposal_has_note():
    room = new_room()
    save_question(room)
    save_question(room, temat="parking", odpowiedz="Parking pod budynkiem")

    room.refused(
        "propozycja_z_notatka",
        "odnotuj_propozycje_opisu",
        id=1,
        propozycja_przekazano="2026-07-03T10:00:00Z",
    )
    room.refused(
        "propozycja_nie_przed_zapisem",
        "odnotuj_propozycje_opisu",
        id=1,
        propozycja_przekazano="2026-06-30T10:00:00Z",
        notatka_odnosnik="pokoj-teczki-lokalu/notatka-1",
    )
    room.run(
        "odnotuj_propozycje_opisu",
        id=1,
        propozycja_przekazano="2026-07-03T10:00:00Z",
        notatka_odnosnik="pokoj-teczki-lokalu/notatka-1",
    )
    room.refused(
        "pytania ani odpowiedzi się nie zmienia",
        "odnotuj_propozycje_opisu",
        id=1,
        propozycja_przekazano="2026-07-04T10:00:00Z",
        notatka_odnosnik="pokoj-teczki-lokalu/notatka-2",
    )


@scenario("pytania ani odpowiedzi się nie zmienia; lepsza odpowiedź to nowy wiersz")
def answer_is_fixed():
    room = new_room()
    save_question(room)

    rule = "pytania ani odpowiedzi się nie zmienia"
    for column, value in [
        ("temat", "'parking'"),
        ("odpowiedz", "'Inna odpowiedź'"),
        ("zapisano", "'2026-06-01'"),
    ]:
        room.refused_sql(rule, f"UPDATE konsjerz_pytania_wpisy SET {column} = {value} WHERE id = 1")


@scenario("nowsza odpowiedź na ten sam temat zastępuje poprzednią raz")
def newer_answer_replaces():
    room = new_room()
    save_question(room)
    save_question(room, odpowiedz="Lepsza odpowiedź", zapisano="2026-07-10T09:00:00Z")
    save_question(room, odpowiedz="Jeszcze lepsza", zapisano="2026-07-20T09:00:00Z")
    save_question(room, odpowiedz="Dawna", zapisano="2026-06-01T09:00:00Z")
    save_question(room, temat="parking", odpowiedz="Parking", zapisano="2026-07-10T09:00:00Z")

    rule = "pytanie zastępuje nowsza odpowiedź na ten sam temat"
    room.refused(rule, "zastap_odpowiedz", id=1, zastapione_przez_id=5)
    room.refused(rule, "zastap_odpowiedz", id=1, zastapione_przez_id=4)
    room.refused("zastapione_nie_przez_siebie", "zastap_odpowiedz", id=1, zastapione_przez_id=1)

    room.run("zastap_odpowiedz", id=1, zastapione_przez_id=2)
    room.refused(
        "pytania ani odpowiedzi się nie zmienia", "zastap_odpowiedz", id=1, zastapione_przez_id=3
    )


# Occurrences -------------------------------------------------------------------------------------


@scenario("wystąpienie dotyczy aktualnej odpowiedzi i nie poprzedza jej zapisania")
def occurrence_of_current_answer():
    room = new_room()
    save_question(room)
    save_question(room, odpowiedz="Lepsza odpowiedź", zapisano="2026-07-10T09:00:00Z")
    add_occurrence(room)

    rule = "wystąpienie dotyczy aktualnej odpowiedzi"
    room.refused(rule, "dopisz_wystapienie_pytania", pytanie_id=2, zadano="2026-07-09T09:00:00Z")
    room.run("zastap_odpowiedz", id=1, zastapione_przez_id=2)
    room.refused(rule, "dopisz_wystapienie_pytania", pytanie_id=1, zadano="2026-07-11T09:00:00Z")
    add_occurrence(room, pytanie_id=2, zadano="2026-07-11T09:00:00Z")


@scenario("wystąpienia tylko się dopisuje; czy odpowiedź pomogła, odnotowuje się raz")
def occurrences_are_appended():
    room = new_room()
    save_question(room)
    add_occurrence(room)

    room.refused_sql(
        "wystąpienie powstaje bez oceny",
        """
        INSERT INTO konsjerz_pytania_wystapienia (pytanie_id, zadano, pomogla)
        VALUES (1, '2026-07-03T09:00:00Z', 1)
        """,
    )

    rule = "wystąpienia tylko się dopisuje"
    room.refused_sql(
        rule, "UPDATE konsjerz_pytania_wystapienia SET zadano = '2026-07-05T09:00:00Z' WHERE id = 1"
    )
    room.run("odnotuj_czy_odpowiedz_pomogla", id=1, pomogla=1)
    room.refused(rule, "odnotuj_czy_odpowiedz_pomogla", id=1, pomogla=0)


# Rules on the shared memory ----------------------------------------------------------------------


@scenario("D14: Konsjerż ocenia rodzaj zgłoszenia, a nie gościa")
def source_is_not_a_guest():
    room = new_room()
    add_source(room, "brak-wody", nazwa="zgłoszenie braku ciepłej wody", rodzaj="rodzaj_zgloszenia")
    add_source(room, "wspolnota", nazwa="Wspólnota Mieszkaniowa Polna 7", rodzaj="instytucja")

    room.refused(
        "Konsjerż ocenia rodzaj zgłoszenia, a nie gościa",
        "wpisz_zrodlo",
        id="gosc",
        nazwa="Gość z pobytu 12",
        rodzaj="osoba",
    )
    room.refused(
        "Konsjerż ocenia rodzaj zgłoszenia, a nie gościa",
        "wpisz_zrodlo",
        id="gosc-z-teczki",
        rodzaj="osoba",
        wpis_odnosnik="pokoj-teczki-lokalu/pobyt-12",
    )


# R9 and replacement --------------------------------------------------------------------------------


@scenario("R9: wierszy się nie usuwa ani nie zastępuje przez REPLACE albo upsert")
def no_deletion_or_replacement():
    room = new_room()
    save_question(room)
    add_occurrence(room)

    tables = [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' "
            "AND name LIKE 'konsjerz\\_%' ESCAPE '\\' ORDER BY name"
        )
    ]
    assert tables == ["konsjerz_pytania_wpisy", "konsjerz_pytania_wystapienia"], tables

    for table in tables:
        room.refused_sql(REPLACEMENT, f"INSERT OR REPLACE {first_row_copy(room, table)}")
        room.refused_sql(
            REPLACEMENT,
            f"INSERT {first_row_copy(room, table)} ON CONFLICT (id) DO UPDATE SET id = excluded.id",
        )
        room.refused_sql(
            DELETION, f"DELETE FROM {table} WHERE rowid = (SELECT min(rowid) FROM {table})"
        )
