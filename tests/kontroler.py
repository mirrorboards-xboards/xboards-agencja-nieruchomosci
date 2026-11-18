"""Scenarios of the Controller's memory room (spec/positions/kontrola/kontroler).

The room is the Controller's own kind and the shared agent kinds it imports. The shared kinds are
checked in stanowisko.py; here each scenario starts from an empty room and checks one rule of
kontroler-sprawdzenia: a write the rule allows and a write it refuses. The first scenario follows a
day of the Controller: an identity verified in a public register, beneficial owners checked against
a statement that turns out wrong, a proposed risk level, the checks passed to the Head of control
and, once they are in the control register, their details cleared (D13).
"""

from pathlib import Path

from stanowisko import DELETION, REPLACEMENT, add_source, add_source_event, first_row_copy
from room import Room

KONTROLER = Path(__file__).resolve().parents[1] / "spec" / "positions" / "kontrola" / "kontroler"

SCENARIOS = []

HANDED_OVER = "przekazanego sprawdzenia się nie zmienia"


def scenario(title):
    def register(function):
        SCENARIOS.append((f"kontroler: {title}", function))

        return function

    return register


def new_room():
    return Room(KONTROLER)


# Building blocks ---------------------------------------------------------------------------------


def check_fields(check_id, **fields):
    return {
        "id": check_id,
        "rodzaj": "weryfikacja",
        "teczka_odnosnik": "pokoj-teczki-oferty",
        "dotyczy_tabela": "sprawa_uczestnicy_wpisy",
        "dotyczy_odnosnik": "pokoj-teczki-oferty/sprzedajaca",
        "zrodlo": "RDO",
        "zrodlo_data": "2026-11-04",
        "wynik": "zgodne",
        "powod_watpliwosci": 0,
        "sprawdzono": "2026-11-04T10:00:00Z",
        **fields,
    }


def save_check(room, check_id, **fields):
    room.run("zapisz_sprawdzenie", **check_fields(check_id, **fields))


def risk_assessment(check_id="ryzyko", **fields):
    return check_fields(
        check_id,
        **{
            "rodzaj": "ocena_ryzyka",
            "dotyczy_tabela": None,
            "dotyczy_odnosnik": None,
            "zrodlo": None,
            "zrodlo_data": None,
            "proponowany_poziom": "srednie",
            "czynniki": "klient spółka; wartość transakcji powyżej progu",
            **fields,
        },
    )


def hand_over(room, check_id, przekazano="2026-11-04T12:00:00Z"):
    room.run("odnotuj_przekazanie_sprawdzenia", id=check_id, przekazano=przekazano)


def enter_in_register(room, check_id, rejestr_odnosnik="pokoj-rejestru-kontroli/sprawdzenie-41"):
    room.run("odnotuj_wpis_sprawdzenia_w_rejestrze", id=check_id, rejestr_odnosnik=rejestr_odnosnik)


# A whole day ---------------------------------------------------------------------------------------


@scenario("dzień Kontrolera: tożsamość, beneficjenci, ocena ryzyka, przekazanie i wpis w rejestrze")
def whole_day():
    room = new_room()

    save_check(room, "tozsamosc")
    add_source(room, "pelnomocnik", nazwa="Pełnomocnik spółki kupującej", rodzaj="osoba")
    save_check(
        room,
        "beneficjenci-1",
        rodzaj="beneficjenci",
        teczka_odnosnik="pokoj-teczki-transakcji",
        dotyczy_odnosnik="pokoj-teczki-transakcji/spolka-kupujaca",
        zrodlo="CRBR",
        zrodlo_id="pelnomocnik",
        wynik="uwagi",
        opis="Oświadczenie pełnomocnika podaje innego beneficjenta niż CRBR",
        powod_watpliwosci=1,
    )
    add_source_event(
        room,
        "pelnomocnik",
        "rozbieznosc",
        dotyczy_tabela="sprawa_uczestnicy_wpisy",
        dotyczy_odnosnik="pokoj-teczki-transakcji/spolka-kupujaca",
    )
    room.run("ocen_zrodlo", id="pelnomocnik", ocena="zastrzezenia", oceniono="2026-11-04T11:00:00Z")
    save_check(
        room,
        "beneficjenci-2",
        rodzaj="beneficjenci",
        teczka_odnosnik="pokoj-teczki-transakcji",
        dotyczy_odnosnik="pokoj-teczki-transakcji/spolka-kupujaca",
        zrodlo="KRS i CRBR",
        zrodlo_data="2026-11-05",
        sprawdzono="2026-11-05T09:00:00Z",
    )
    room.run("zastap_sprawdzenie", id="beneficjenci-1", zastapione_przez_id="beneficjenci-2")
    room.run("zapisz_sprawdzenie", **risk_assessment(sprawdzono="2026-11-05T10:00:00Z"))

    for row in room.run("sprawdzenia_do_przekazania"):
        hand_over(room, row["id"], przekazano="2026-11-05T12:00:00Z")
    enter_in_register(room, "ryzyko", rejestr_odnosnik="pokoj-rejestru-kontroli/ocena-7")

    assert room.run("sprawdzenia_do_przekazania") == []
    assert {row["id"] for row in room.run("sprawdzenia")} == {
        "tozsamosc",
        "beneficjenci-2",
        "ryzyko",
    }
    assert [
        row["id"]
        for row in room.run("sprawdzenia_teczki", teczka_odnosnik="pokoj-teczki-transakcji")
    ] == ["beneficjenci-2"]

    entered = room.execute("SELECT * FROM kontroler_sprawdzenia_wpisy WHERE id = 'ryzyko'")[0]
    assert entered["opis"] is None
    assert entered["czynniki"] is None
    assert entered["proponowany_poziom"] is None
    assert entered["rodzaj"] == "ocena_ryzyka"
    assert entered["wynik"] == "zgodne"
    assert entered["rejestr_odnosnik"] == "pokoj-rejestru-kontroli/ocena-7"


# Writing a check ---------------------------------------------------------------------------------


@scenario("identyfikacja, weryfikacja i beneficjenci mają źródło z datą")
def identity_check_has_source():
    room = new_room()
    save_check(room, "tozsamosc")
    save_check(room, "sankcje", rodzaj="sankcje", zrodlo=None, zrodlo_data=None)

    room.refused(
        "identyfikacja_ze_zrodlem_i_data",
        "zapisz_sprawdzenie",
        **check_fields("beneficjenci", rodzaj="beneficjenci", zrodlo=None, zrodlo_data=None),
    )
    room.refused(
        "zrodlo_z_data",
        "zapisz_sprawdzenie",
        **check_fields("identyfikacja", rodzaj="identyfikacja", zrodlo_data=None),
    )


@scenario("sprawdzany wiersz ma tabelę razem z odnośnikiem")
def checked_row_has_table():
    room = new_room()
    save_check(room, "pep", rodzaj="pep", dotyczy_tabela=None, dotyczy_odnosnik=None)

    room.refused(
        "dotyczy_tabela_razem_z_odnosnikiem",
        "zapisz_sprawdzenie",
        **check_fields("pep-2", rodzaj="pep", dotyczy_tabela=None),
    )


@scenario("poziom i czynniki są tylko przy ocenie ryzyka, a ocena ryzyka je ma")
def level_and_factors_only_with_risk_assessment():
    room = new_room()
    room.run("zapisz_sprawdzenie", **risk_assessment())

    room.refused(
        "poziom_i_czynniki_tylko_przy_ocenie_ryzyka",
        "zapisz_sprawdzenie",
        **check_fields("sankcje", rodzaj="sankcje", proponowany_poziom="niskie"),
    )
    room.refused(
        "poziom_i_czynniki_tylko_przy_ocenie_ryzyka",
        "zapisz_sprawdzenie",
        **check_fields("pep", rodzaj="pep", czynniki="stanowisko polityczne"),
    )
    room.refused(
        "ocena_ryzyka_z_poziomem_i_czynnikami",
        "zapisz_sprawdzenie",
        **risk_assessment("ryzyko-2", czynniki=None),
    )
    room.refused(
        "ocena_ryzyka_z_poziomem_i_czynnikami",
        "zapisz_sprawdzenie",
        **risk_assessment("ryzyko-3", proponowany_poziom=None),
    )


@scenario("uwagi i brak ustalenia mają opis")
def remarks_have_description():
    room = new_room()
    save_check(room, "zrodlo-srodkow", rodzaj="zrodlo_srodkow", wynik="uwagi", opis="Brak wyciągu")

    room.refused(
        "uwagi_z_opisem",
        "zapisz_sprawdzenie",
        **check_fields("crbr", rodzaj="crbr", wynik="nie_mozna_ustalic"),
    )


@scenario("sprawdzenie powstaje robocze: bez przekazania, wpisu w rejestrze i zastąpienia")
def check_starts_as_draft():
    room = new_room()
    save_check(room, "tozsamosc")

    rule = "sprawdzenie powstaje robocze"
    for column, value in [
        ("przekazano", "'2026-11-04T12:00:00Z'"),
        ("zastapione_przez_id", "'tozsamosc'"),
    ]:
        room.refused_sql(
            rule,
            f"""
            INSERT INTO kontroler_sprawdzenia_wpisy (id, rodzaj, teczka_odnosnik, wynik,
              powod_watpliwosci, sprawdzono, {column})
            VALUES ('gotowe', 'sankcje', 'pokoj-teczki-oferty', 'zgodne', 0,
              '2026-11-04T10:00:00Z', {value})
            """,
        )


# Handover and the control register ---------------------------------------------------------------


@scenario("wpis w rejestrze jest tylko po przekazaniu, a przekazanie po sprawdzeniu")
def register_entry_after_handover():
    room = new_room()
    save_check(room, "tozsamosc")

    room.refused(
        "wpis_w_rejestrze_po_przekazaniu",
        "odnotuj_wpis_sprawdzenia_w_rejestrze",
        id="tozsamosc",
        rejestr_odnosnik="pokoj-rejestru-kontroli/sprawdzenie-41",
    )
    room.refused(
        "przekazanie_nie_przed_sprawdzeniem",
        "odnotuj_przekazanie_sprawdzenia",
        id="tozsamosc",
        przekazano="2026-11-01T00:00:00Z",
    )


@scenario("roboczy wpis się poprawia, a przekazanego się nie zmienia")
def handed_over_check_is_fixed():
    room = new_room()
    save_check(room, "sankcje", rodzaj="sankcje", wynik="uwagi", opis="Zbieżność nazwiska")
    room.execute(
        "UPDATE kontroler_sprawdzenia_wpisy SET opis = 'Zbieżność nazwiska; inna data urodzenia' "
        "WHERE id = 'sankcje'"
    )
    hand_over(room, "sankcje")

    for column, value in [
        ("wynik", "'zgodne'"),
        ("opis", "'Inny opis'"),
        ("zrodlo_data", "'2026-11-03'"),
        ("powod_watpliwosci", "1"),
        ("przekazano", "'2026-11-06T12:00:00Z'"),
    ]:
        room.refused_sql(
            HANDED_OVER,
            f"UPDATE kontroler_sprawdzenia_wpisy SET {column} = {value} WHERE id = 'sankcje'",
        )


@scenario("D13: po wpisie w rejestrze opis, czynniki i poziom znikają, a wpisu się nie zmienia")
def register_entry_clears_details():
    room = new_room()
    room.run("zapisz_sprawdzenie", **risk_assessment())
    hand_over(room, "ryzyko")

    room.refused_sql(
        "po_wpisie_w_rejestrze_bez_szczegolow",
        "UPDATE kontroler_sprawdzenia_wpisy SET rejestr_odnosnik = 'pokoj-rejestru-kontroli/ocena-7' "
        "WHERE id = 'ryzyko'",
    )
    enter_in_register(room, "ryzyko", rejestr_odnosnik="pokoj-rejestru-kontroli/ocena-7")

    room.refused_sql(
        "po_wpisie_w_rejestrze_bez_szczegolow",
        "UPDATE kontroler_sprawdzenia_wpisy SET opis = 'Szczegóły wracają' WHERE id = 'ryzyko'",
    )
    room.refused(
        HANDED_OVER,
        "odnotuj_wpis_sprawdzenia_w_rejestrze",
        id="ryzyko",
        rejestr_odnosnik="pokoj-rejestru-kontroli/ocena-8",
    )
    room.refused_sql(
        HANDED_OVER,
        "UPDATE kontroler_sprawdzenia_wpisy SET rejestr_odnosnik = NULL WHERE id = 'ryzyko'",
    )


# Replacement by a new check ----------------------------------------------------------------------


@scenario("ponowne sprawdzenie tego samego rodzaju, nie starsze, zastępuje poprzednie raz")
def new_check_replaces_old():
    room = new_room()
    save_check(room, "tozsamosc")
    hand_over(room, "tozsamosc")
    save_check(room, "tozsamosc-2", sprawdzono="2026-11-06T10:00:00Z")
    save_check(room, "tozsamosc-3", sprawdzono="2026-11-07T10:00:00Z")
    save_check(room, "dawne", sprawdzono="2026-11-01T10:00:00Z")
    save_check(room, "sankcje", rodzaj="sankcje", sprawdzono="2026-11-06T10:00:00Z")

    rule = "sprawdzenie zastępuje ponowne sprawdzenie tego samego rodzaju"
    room.refused(rule, "zastap_sprawdzenie", id="tozsamosc", zastapione_przez_id="sankcje")
    room.refused(rule, "zastap_sprawdzenie", id="tozsamosc", zastapione_przez_id="dawne")
    room.refused(
        "zastapione_nie_przez_siebie",
        "zastap_sprawdzenie",
        id="tozsamosc",
        zastapione_przez_id="tozsamosc",
    )

    room.run("zastap_sprawdzenie", id="tozsamosc", zastapione_przez_id="tozsamosc-2")
    room.refused(
        "zastąpionego sprawdzenia nie zastępuje się drugi raz",
        "zastap_sprawdzenie",
        id="tozsamosc",
        zastapione_przez_id="tozsamosc-3",
    )
    assert "tozsamosc" not in {row["id"] for row in room.run("sprawdzenia")}


# R9 and replacement --------------------------------------------------------------------------------


@scenario("R9: sprawdzeń się nie usuwa ani nie zastępuje przez REPLACE albo upsert")
def no_deletion_or_replacement():
    room = new_room()
    save_check(room, "tozsamosc")

    tables = [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'kontroler\\_%' ESCAPE '\\'"
        )
    ]
    assert tables == ["kontroler_sprawdzenia_wpisy"], tables

    for table in tables:
        room.refused_sql(REPLACEMENT, f"INSERT OR REPLACE {first_row_copy(room, table)}")
        room.refused_sql(
            REPLACEMENT,
            f"INSERT {first_row_copy(room, table)} ON CONFLICT (id) DO UPDATE SET id = excluded.id",
        )
        room.refused_sql(
            DELETION, f"DELETE FROM {table} WHERE rowid = (SELECT min(rowid) FROM {table})"
        )
