"""Scenarios of the Legal status analyst's memory room
(spec/positions/transakcje/analityk_stanu_prawnego).

The room is the analyst's two own kinds and the shared agent kinds it imports. The shared kinds are
checked in stanowisko.py; here each scenario starts from an empty room and checks one rule of
analityk-stanu-prawnego-ksiega or analityk-stanu-prawnego-sprawdzenia: a write the rule allows and
a write it refuses. The first scenario follows a transaction: a land register extract, checks of
its sections and of the municipality's pre-emption right, a fresh viewing of the register the day
before the deed and the changes between the two states.
"""

from pathlib import Path

from stanowisko import DELETION, REPLACEMENT, first_row_copy
from room import Room

ANALITYK = (
    Path(__file__).resolve().parents[1]
    / "spec"
    / "positions"
    / "transakcje"
    / "analityk_stanu_prawnego"
)

SCENARIOS = []

NUMER_KW = "WA1M/00012345/6"


def scenario(title):
    def register(function):
        SCENARIOS.append((f"analityk_stanu_prawnego: {title}", function))

        return function

    return register


def new_room():
    return Room(ANALITYK)


# Building blocks ---------------------------------------------------------------------------------


def save_state(room, **fields):
    room.run(
        "zapisz_stan_ksiegi",
        **{
            "numer_kw": NUMER_KW,
            "chwila": "2026-11-02T08:00:00Z",
            "sposob": "odpis",
            "dokument_odnosnik": "pokoj-teczki-transakcji/odpis-1",
            **fields,
        },
    )


def save_register_entry(room, stan_id=1, **fields):
    room.run(
        "zapisz_wpis_ksiegi",
        **{
            "stan_id": stan_id,
            "dzial": "iv",
            "numer": "1",
            "rodzaj": "wpis",
            "przedmiot": "hipoteka umowna na rzecz banku",
            **fields,
        },
    )


def check_fields(check_id, **fields):
    return {
        "id": check_id,
        "przedmiot": "dzial_iv",
        "dokument_odnosnik": "pokoj-teczki-transakcji/odpis-1",
        "dokument_data": "2026-11-02",
        "przepis": "art. 65 ust. 1 ukwh",
        "wynik": "ryzyko",
        "opis": "Hipoteka do spłaty z ceny; projekt umowy musi to opisać",
        "ksiega_stan_id": 1,
        "sprawdzono": "2026-11-03T10:00:00Z",
        **fields,
    }


def save_check(room, check_id, **fields):
    room.run("zapisz_sprawdzenie_stanu_prawnego", **check_fields(check_id, **fields))


# A whole transaction -------------------------------------------------------------------------------


@scenario("transakcja: odpis, sprawdzenia działów i pierwokupu, przeglądanie przed aktem i zmiany")
def whole_transaction():
    room = new_room()

    save_state(room)
    save_register_entry(room, dzial="iii", przedmiot="dożywocie na rzecz matki zbywcy")
    save_register_entry(room)
    save_check(
        room,
        "dzial-iii",
        przedmiot="dzial_iii",
        przepis="art. 7 ukwh",
        opis="Dożywocie przechodzi na nabywcę",
    )
    save_check(room, "dzial-iv")
    save_check(
        room,
        "pierwokup",
        przedmiot="pierwokup",
        przepis="art. 109 ust. 1 ugn",
        wynik="bez_uwag",
        opis=None,
        ksiega_stan_id=None,
    )
    for row in room.run("sprawdzenia_bez_analizy"):
        room.run(
            "odnotuj_analize_sprawdzenia",
            id=row["id"],
            analiza_odnosnik="pokoj-teczki-transakcji/analiza-ksiegi",
        )

    assert room.run("zmiany_ksiegi", numer_kw=NUMER_KW) == []

    save_state(room, chwila="2026-11-19T07:30:00Z", sposob="przegladanie", dokument_odnosnik=None)
    save_register_entry(room, stan_id=2, dzial="iii", przedmiot="dożywocie na rzecz matki zbywcy")
    save_register_entry(
        room,
        stan_id=2,
        rodzaj="wzmianka",
        numer="Dz.Kw./4567/26",
        przedmiot="wniosek o wykreślenie hipoteki",
    )
    save_check(
        room,
        "dzial-iv-2",
        dokument_odnosnik="pokoj-teczki-transakcji/wydruk-2",
        dokument_data="2026-11-19",
        ksiega_stan_id=2,
        opis="Wzmianka o wniosku wyłącza rękojmię do jego rozpoznania",
        sprawdzono="2026-11-19T08:00:00Z",
    )
    room.run("zastap_sprawdzenie_stanu_prawnego", id="dzial-iv", zastapione_przez_id="dzial-iv-2")

    changes = {
        (row["zmiana"], row["dzial"], row["numer"], row["rodzaj"])
        for row in room.run("zmiany_ksiegi", numer_kw=NUMER_KW)
    }
    assert changes == {
        ("usuniety", "iv", "1", "wpis"),
        ("dodany", "iv", "Dz.Kw./4567/26", "wzmianka"),
    }
    assert [row["id"] for row in room.run("stany_ksiegi", numer_kw=NUMER_KW)] == [2, 1]
    assert {row["id"] for row in room.run("sprawdzenia_stanu_prawnego")} == {
        "dzial-iii",
        "dzial-iv-2",
        "pierwokup",
    }
    assert [row["id"] for row in room.run("sprawdzenia_bez_analizy")] == ["dzial-iv-2"]
    assert [
        row["id"]
        for row in room.run(
            "sprawdzenia_dokumentu", dokument_odnosnik="pokoj-teczki-transakcji/odpis-1"
        )
    ] == ["dzial-iii", "pierwokup"]


# The land register -------------------------------------------------------------------------------


@scenario("odpis wskazuje dokument w teczce, a przeglądanie nie musi")
def extract_points_document():
    room = new_room()
    save_state(room)
    save_state(room, chwila="2026-11-05T08:00:00Z", sposob="przegladanie", dokument_odnosnik=None)

    room.refused(
        "odpis_z_dokumentem",
        "zapisz_stan_ksiegi",
        numer_kw=NUMER_KW,
        chwila="2026-11-06T08:00:00Z",
        sposob="odpis",
    )
    room.refused(
        "CHECK constraint failed: numer_kw",
        "zapisz_stan_ksiegi",
        numer_kw="WA1M/123/6",
        chwila="2026-11-06T08:00:00Z",
        sposob="przegladanie",
    )


@scenario("nowy stan księgi pokazuje ją później niż poprzedni")
def new_state_is_later():
    room = new_room()
    save_state(room)
    save_state(room, numer_kw="KR1P/00000001/1", chwila="2026-11-01T08:00:00Z")

    rule = "nowy stan księgi pokazuje ją później niż poprzedni"
    room.refused(rule, "zapisz_stan_ksiegi", **_state(chwila="2026-11-01T08:00:00Z"))
    room.refused(REPLACEMENT, "zapisz_stan_ksiegi", **_state(chwila="2026-11-02T08:00:00Z"))


@scenario("wpis księgi dopisuje się tylko do ostatniego stanu i tylko raz")
def entries_go_to_latest_state():
    room = new_room()
    save_state(room)
    save_register_entry(room)
    save_register_entry(room, rodzaj="wzmianka", numer="Dz.Kw./1/26", przedmiot="wniosek")

    room.refused(REPLACEMENT, "zapisz_wpis_ksiegi", **_entry())

    save_state(room, chwila="2026-11-05T08:00:00Z")
    room.refused(
        "wpis księgi dopisuje się tylko do ostatniego stanu księgi",
        "zapisz_wpis_ksiegi",
        **_entry(numer="2"),
    )
    save_register_entry(room, stan_id=2, numer="2")


@scenario("stanu księgi ani jego wpisów się nie zmienia")
def state_is_fixed():
    room = new_room()
    save_state(room)
    save_register_entry(room)

    rule = "stanu księgi ani jego wpisów się nie zmienia"
    room.refused_sql(
        rule, "UPDATE analityk_stanu_prawnego_ksiega_stany SET chwila = '2026-11-03T08:00:00Z'"
    )
    room.refused_sql(
        rule, "UPDATE analityk_stanu_prawnego_ksiega_wpisy SET przedmiot = 'hipoteka wykreślona'"
    )


# Checks ------------------------------------------------------------------------------------------


@scenario("ryzyko i brak ustalenia mają opis")
def risk_has_description():
    room = new_room()
    save_state(room)
    save_check(room, "dzial-iv")
    save_check(room, "dzial-ii", przedmiot="dzial_ii", wynik="bez_uwag", opis=None)

    room.refused(
        "ryzyko_z_opisem",
        "zapisz_sprawdzenie_stanu_prawnego",
        **check_fields("dzial-iii", przedmiot="dzial_iii", wynik="nie_mozna_ustalic", opis=None),
    )


@scenario("sprawdzenie działu księgi stoi na stanie księgi")
def register_section_needs_state():
    room = new_room()
    save_state(room)
    save_check(room, "projekt", przedmiot="projekt_umowy", ksiega_stan_id=None)

    room.refused(
        "dzial_ksiegi_ze_stanem",
        "zapisz_sprawdzenie_stanu_prawnego",
        **check_fields("dzial-i", przedmiot="dzial_i", ksiega_stan_id=None),
    )
    room.refused(
        "FOREIGN KEY constraint failed",
        "zapisz_sprawdzenie_stanu_prawnego",
        **check_fields("dzial-iv", ksiega_stan_id=7),
    )


@scenario("dokument nie jest późniejszy od sprawdzenia")
def document_not_after_check():
    room = new_room()
    save_state(room)
    save_check(room, "dzial-iv", dokument_data="2026-11-03")

    room.refused(
        "dokument_nie_po_sprawdzeniu",
        "zapisz_sprawdzenie_stanu_prawnego",
        **check_fields("dzial-iv-2", dokument_data="2026-11-04"),
    )


@scenario("sprawdzenie powstaje aktualne i bez analizy")
def check_starts_current():
    room = new_room()
    save_state(room)
    save_check(room, "dzial-iv")

    rule = "sprawdzenie powstaje aktualne i bez analizy"
    for column, value in [
        ("analiza_odnosnik", "'pokoj-teczki-transakcji/analiza-1'"),
        ("zastapione_przez_id", "'dzial-iv'"),
    ]:
        room.refused_sql(
            rule,
            f"""
            INSERT INTO analityk_stanu_prawnego_sprawdzenia_wpisy (id, przedmiot,
              dokument_odnosnik, dokument_data, przepis, wynik, sprawdzono, {column})
            VALUES ('projekt', 'projekt_umowy', 'pokoj-teczki-transakcji/projekt-1', '2026-11-02',
              'art. 389 KC', 'bez_uwag', '2026-11-03T10:00:00Z', {value})
            """,
        )


@scenario("sprawdzenia się nie zmienia; analizę odnotowuje się raz")
def check_is_fixed():
    room = new_room()
    save_state(room)
    save_check(room, "dzial-iv")

    rule = "sprawdzenia się nie zmienia"
    for column, value in [
        ("wynik", "'bez_uwag'"),
        ("opis", "'Inny opis'"),
        ("przepis", "'art. 66 ukwh'"),
        ("dokument_data", "'2026-11-01'"),
    ]:
        room.refused_sql(
            rule,
            f"UPDATE analityk_stanu_prawnego_sprawdzenia_wpisy SET {column} = {value} "
            "WHERE id = 'dzial-iv'",
        )

    room.run(
        "odnotuj_analize_sprawdzenia",
        id="dzial-iv",
        analiza_odnosnik="pokoj-teczki-transakcji/analiza-1",
    )
    room.refused(
        rule,
        "odnotuj_analize_sprawdzenia",
        id="dzial-iv",
        analiza_odnosnik="pokoj-teczki-transakcji/analiza-2",
    )


@scenario("ponowne sprawdzenie tego samego przedmiotu, nie starsze, zastępuje poprzednie raz")
def new_check_replaces_old():
    room = new_room()
    save_state(room)
    save_check(room, "dzial-iv")
    save_check(room, "dzial-iv-2", sprawdzono="2026-11-04T10:00:00Z")
    save_check(room, "dzial-iv-3", sprawdzono="2026-11-05T10:00:00Z")
    save_check(room, "dzial-iv-dawny", sprawdzono="2026-11-02T10:00:00Z")
    save_check(room, "dzial-iii", przedmiot="dzial_iii", sprawdzono="2026-11-04T10:00:00Z")

    rule = "sprawdzenie zastępuje ponowne sprawdzenie tego samego przedmiotu"
    replace = "zastap_sprawdzenie_stanu_prawnego"
    room.refused(rule, replace, id="dzial-iv", zastapione_przez_id="dzial-iii")
    room.refused(rule, replace, id="dzial-iv", zastapione_przez_id="dzial-iv-dawny")
    room.refused(
        "zastapione_nie_przez_siebie", replace, id="dzial-iv", zastapione_przez_id="dzial-iv"
    )

    room.run(replace, id="dzial-iv", zastapione_przez_id="dzial-iv-2")
    room.refused(
        "sprawdzenia się nie zmienia", replace, id="dzial-iv", zastapione_przez_id="dzial-iv-3"
    )


def _state(**fields):
    return {
        "numer_kw": NUMER_KW,
        "chwila": "2026-11-02T08:00:00Z",
        "sposob": "przegladanie",
        **fields,
    }


def _entry(**fields):
    return {
        "stan_id": 1,
        "dzial": "iv",
        "numer": "1",
        "rodzaj": "wpis",
        "przedmiot": "hipoteka umowna na rzecz banku",
        **fields,
    }


# R9 and replacement --------------------------------------------------------------------------------


def own_tables(room):
    return [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' "
            "AND name LIKE 'analityk\\_stanu\\_prawnego\\_%' ESCAPE '\\' ORDER BY name"
        )
    ]


@scenario("R9: wierszy się nie usuwa ani nie zastępuje przez REPLACE albo upsert")
def no_deletion_or_replacement():
    room = new_room()
    save_state(room)
    save_register_entry(room)
    save_check(room, "dzial-iv")

    tables = own_tables(room)
    assert tables == [
        "analityk_stanu_prawnego_ksiega_stany",
        "analityk_stanu_prawnego_ksiega_wpisy",
        "analityk_stanu_prawnego_sprawdzenia_wpisy",
    ], tables

    for table in tables:
        room.refused_sql(REPLACEMENT, f"INSERT OR REPLACE {first_row_copy(room, table)}")
        room.refused_sql(
            REPLACEMENT,
            f"INSERT {first_row_copy(room, table)} ON CONFLICT (id) DO UPDATE SET id = excluded.id",
        )
        room.refused_sql(
            DELETION, f"DELETE FROM {table} WHERE rowid = (SELECT min(rowid) FROM {table})"
        )


@scenario("REPLACE i upsert nie zastępują stanu ani wpisu księgi przez klucz jednoznaczny")
def no_replacement_through_unique_keys():
    room = new_room()
    save_state(room)
    save_register_entry(room)

    room.refused_sql(
        REPLACEMENT,
        f"""
        INSERT OR REPLACE INTO analityk_stanu_prawnego_ksiega_stany (numer_kw, chwila, sposob)
        VALUES ('{NUMER_KW}', '2026-11-02T08:00:00Z', 'przegladanie')
        """,
    )
    room.refused_sql(
        REPLACEMENT,
        """
        INSERT INTO analityk_stanu_prawnego_ksiega_wpisy (stan_id, dzial, numer, rodzaj, przedmiot)
        VALUES (1, 'iv', '1', 'wpis', 'hipoteka wykreślona')
        ON CONFLICT (stan_id, dzial, rodzaj, numer) DO UPDATE SET przedmiot = excluded.przedmiot
        """,
    )
