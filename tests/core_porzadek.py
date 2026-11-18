"""Scenarios of the core kinds that keep the case in order (spec/cases/sprawa):
deadlines (sprawa-terminy), hand-over notes (sprawa-notatki), links to other rooms
(sprawa-powiazania), and R9: rows of a case file are never deleted, except the corrections D14
allows.

Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
allows and a write it refuses.
"""

from core import (
    add_person,
    approve,
    approved_decision,
    ask_for_decision,
    full_case,
    new_room,
)

SCENARIOS = []


def scenario(title):
    def register(function):
        SCENARIOS.append((title, function))

        return function

    return register


# Deadlines ---------------------------------------------------------------------------------------


def deadline_fields(**fields):
    return {
        "id": "termin",
        "rodzaj": "ustawowy",
        "co": "Informacja o danych pozyskanych od kupującego",
        "zdarzenie_tabela": "sprawa_uczestnicy_wpisy",
        "zdarzenie_id": "sprzedajacy",
        "zdarzenie_chwila": "2026-11-02T10:00:00Z",
        "koniec": "2026-12-02",
        "podstawa": "art. 14 ust. 3 lit. a RODO",
        "pilnuje_stanowisko": "opiekun_poszukujacego",
        **fields,
    }


@scenario("Termin ustawowy ma podstawę i zdarzenie; termin powstaje biegnący")
def deadline_statutory():
    room = new_room()

    room.refused(
        "ustawowy_ma_podstawe_i_zdarzenie", "wpisz_termin", **deadline_fields(podstawa=None)
    )
    room.refused(
        "ustawowy_ma_podstawe_i_zdarzenie",
        "wpisz_termin",
        **deadline_fields(zdarzenie_tabela=None, zdarzenie_id=None),
    )
    room.refused(
        "zdarzenie_tabela_razem_z_id", "wpisz_termin", **deadline_fields(zdarzenie_id=None)
    )
    room.run("wpisz_termin", **deadline_fields())
    room.run(
        "wpisz_termin",
        **deadline_fields(
            id="odpowiedz",
            rodzaj="wewnetrzny",
            co="Odpowiedź na wiadomość klienta",
            zdarzenie_tabela=None,
            zdarzenie_id=None,
            zdarzenie_chwila=None,
            podstawa=None,
        ),
    )

    room.refused_sql(
        "termin powstaje jako biegnący i niepotwierdzony",
        """INSERT INTO sprawa_terminy_wpisy (id, rodzaj, co, koniec, pilnuje_stanowisko, stan)
        VALUES ('minal', 'wewnetrzny', 'Raport', '2026-11-01', 'opiekun_oferty', 'minal')""",
    )
    room.run("zmien_stan_terminu", id="odpowiedz", stan="minal")
    assert [row["id"] for row in room.run("terminy_biegnace")] == ["termin"]


@scenario("Termin potwierdza tylko zatwierdzona decyzja potwierdzenie_terminu, która go wskazuje")
def deadline_confirmed_by_decision():
    room = new_room()
    room.run(
        "wpisz_termin",
        **deadline_fields(
            id="akt",
            rodzaj="z_umowy",
            co="Akt notarialny",
            podstawa="§ 2 umowy przedwstępnej",
            pilnuje_stanowisko="koordynator_transakcji",
        ),
    )
    room.run(
        "wpisz_termin",
        **deadline_fields(id="inny", rodzaj="z_umowy", co="Przekazanie lokalu"),
    )
    refusal = (
        "termin potwierdza tylko zatwierdzona decyzja potwierdzenie_terminu, która go wskazuje"
    )

    room.refused_sql(
        "termin powstaje jako biegnący i niepotwierdzony",
        """INSERT INTO sprawa_terminy_wpisy (id, rodzaj, co, koniec, pilnuje_stanowisko, stan,
          potwierdzenie_decyzja_id)
        VALUES ('od-razu', 'z_umowy', 'Akt', '2026-12-01', 'koordynator_transakcji', 'biegnie', 'x')""",
    )

    approved_decision(room, "o-innym", "potwierdzenie_terminu", "sprawa_terminy_wpisy", "inny")
    room.refused(
        refusal, "odnotuj_potwierdzenie_terminu", id="akt", potwierdzenie_decyzja_id="o-innym"
    )

    ask_for_decision(room, "odmowa", "potwierdzenie_terminu", "sprawa_terminy_wpisy", "akt")
    approve(room, "odmowa", rozstrzygniecie="odmowa")
    room.refused(
        refusal, "odnotuj_potwierdzenie_terminu", id="akt", potwierdzenie_decyzja_id="odmowa"
    )

    approved_decision(room, "inna", "inna", "sprawa_terminy_wpisy", "akt")
    room.refused(
        refusal, "odnotuj_potwierdzenie_terminu", id="akt", potwierdzenie_decyzja_id="inna"
    )

    approved_decision(room, "zgoda", "potwierdzenie_terminu", "sprawa_terminy_wpisy", "akt")
    room.run("odnotuj_potwierdzenie_terminu", id="akt", potwierdzenie_decyzja_id="zgoda")


@scenario("Potwierdzony termin zmienia już tylko stan, a potwierdzenia się nie zeruje")
def confirmed_deadline_frozen():
    room = new_room()
    room.run(
        "wpisz_termin",
        **deadline_fields(id="akt", rodzaj="z_umowy", co="Akt notarialny", podstawa="§ 2"),
    )
    room.run(
        "przelicz_termin",
        id="akt",
        koniec="2026-12-02T10:00:00Z",
        kalendarz_odnosnik="kalendarz/akt",
    )
    approved_decision(room, "zgoda", "potwierdzenie_terminu", "sprawa_terminy_wpisy", "akt")
    room.run("odnotuj_potwierdzenie_terminu", id="akt", potwierdzenie_decyzja_id="zgoda")
    refusal = "potwierdzonego terminu się nie zmienia"

    # The workarounds from the audit: a new end and subject after the confirmation, and the
    # confirmation cleared.
    room.refused_sql(
        refusal,
        "UPDATE sprawa_terminy_wpisy SET koniec = '2027-01-01', co = 'Inny akt' WHERE id = 'akt'",
    )
    room.refused(refusal, "przelicz_termin", id="akt", koniec="2027-01-01")
    room.refused(refusal, "odnotuj_potwierdzenie_terminu", id="akt", potwierdzenie_decyzja_id=None)

    room.run("zmien_stan_terminu", id="akt", stan="zbedny")


# Hand-over notes ---------------------------------------------------------------------------------


def note_fields(**fields):
    return {
        "id": "notatka",
        "od": "tozsamosc-kontrolera",
        "od_stanowiska": "kontroler",
        "do_stanowiska": "koordynator_transakcji",
        "tresc": "Potrzebny odpis z KRS spółki z dnia aktu.",
        "utworzono": "2026-11-05T09:00:00Z",
        "do_kiedy": "2026-11-07",
        **fields,
    }


@scenario("Notatka powstaje otwarta, jej treść się nie zmienia, a stan zmienia się raz")
def note_rules():
    room = new_room()

    room.refused_sql(
        "notatka powstaje jako otwarta",
        """INSERT INTO sprawa_notatki_wpisy (id, od, od_stanowiska, do_stanowiska, tresc, utworzono,
          stan)
        VALUES ('od-razu', 'k', 'kontroler', 'opiekun_oferty', 'Treść', '2026-11-05T09:00:00Z',
          'wykonana')""",
    )

    room.run("dopisz_notatke", **note_fields())
    room.run(
        "wskaz_wiersz_notatki",
        notatka_id="notatka",
        dotyczy_tabela="sprawa_umowy_wpisy",
        dotyczy_id="u",
    )
    room.refused_sql(
        "treść notatki się nie zmienia",
        "UPDATE sprawa_notatki_wpisy SET tresc = 'Inna treść'",
    )
    room.refused_sql(
        "wiersza notatki się nie zmienia",
        "UPDATE sprawa_notatki_wiersze SET dotyczy_id = 'v'",
    )

    room.refused_sql(
        "odrzucona_ma_powod",
        "UPDATE sprawa_notatki_wpisy SET stan = 'odrzucona'",
    )
    room.run("odrzuc_notatke", id="notatka", powod="Odpis z KRS pobiera Analityk stanu prawnego.")
    room.refused(
        "stan notatki zmienia się raz",
        "oznacz_notatke_jako_wykonana",
        id="notatka",
    )

    room.run(
        "dopisz_notatke",
        **note_fields(
            id="odpowiedz",
            w_odpowiedzi_na="notatka",
            od_stanowiska="koordynator_transakcji",
            do_stanowiska="analityk_stanu_prawnego",
        ),
    )
    room.run("oznacz_notatke_jako_wykonana", id="odpowiedz")
    assert room.run("notatki_otwarte_do_stanowiska", do_stanowiska="analityk_stanu_prawnego") == []


@scenario("Notatka wskazuje klienta tylko wtedy, gdy idzie do Kontrolera (R1)")
def note_to_controller_of_client():
    room = new_room()
    add_person(room, "kupujacy")

    room.refused(
        "klient_tylko_przy_kontrolerze",
        "dopisz_notatke",
        **note_fields(
            od_stanowiska="koordynator_transakcji",
            do_stanowiska="opiekun_oferty",
            do_klienta_uczestnik_id="kupujacy",
        ),
    )
    room.run(
        "dopisz_notatke",
        **note_fields(
            od_stanowiska="koordynator_transakcji",
            do_stanowiska="kontroler",
            do_klienta_uczestnik_id="kupujacy",
        ),
    )
    assert room.run("notatki_otwarte_do_stanowiska", do_stanowiska="kontroler")


# Links -------------------------------------------------------------------------------------------


@scenario("Powiązania się nie zmienia; nowe powiązanie to nowy wiersz")
def link_rules():
    room = new_room()

    room.run(
        "wpisz_powiazanie",
        rodzaj="pokoj_wymiany",
        odnosnik="pokoj-wymiany-kancelaria-7",
        wpisal="tozsamosc-koordynatora",
        wpisano="2026-11-05T09:00:00Z",
    )
    room.refused(
        "typ_teczki",
        "wpisz_powiazanie",
        rodzaj="sprawa_zrodlowa",
        typ_teczki="Oferta",
        oznaczenie="Oferta 2026/041",
        odnosnik="pokoj-oferty-041",
        wpisal="tozsamosc-kierownika",
        wpisano="2026-11-05T09:00:00Z",
    )
    room.refused_sql(
        "powiązania się nie zmienia",
        "UPDATE sprawa_powiazania_wpisy SET odnosnik = 'inny-pokoj'",
    )


# R9: no deletion ---------------------------------------------------------------------------------

# D14: a mistaken address, citizenship or plot may be deleted, because the room's XSql journal keeps
# the previous state with its author.
CORRECTABLE_TABLES = {
    "sprawa_uczestnicy_adresy",
    "sprawa_uczestnicy_obywatelstwa",
    "sprawa_nieruchomosci_dzialki",
}


@scenario("R9: wierszy teczki się nie usuwa; wyjątkiem są adresy, obywatelstwa i działki (D14)")
def no_deletion():
    room = new_room()
    full_case(room)

    tables = [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'sprawa\\_%' ESCAPE '\\'"
        )
    ]
    assert len(tables) == 31, tables

    for table in tables:
        assert room.value(f"SELECT count(*) FROM {table}") > 0, f"{table} has no row to delete"
        if table in CORRECTABLE_TABLES:
            continue

        room.refused_sql(
            "wierszy teczki się nie usuwa (R9)",
            f"DELETE FROM {table} WHERE rowid = (SELECT min(rowid) FROM {table})",
        )

    for table in sorted(CORRECTABLE_TABLES):
        room.execute(f"DELETE FROM {table}")
        assert room.value(f"SELECT count(*) FROM {table}") == 0, table


def first_row_copy(room, table, replace_unique_key=False):
    """SQL that inserts again the first row of the table, as a careless upsert would.

    With `replace_unique_key` the copy leaves the key out, so only a UNIQUE key collides.
    """
    columns = [row["name"] for row in room.execute(f"PRAGMA table_info({table})")]
    if replace_unique_key:
        columns = [column for column in columns if column != "id"]
    column_list = ", ".join(columns)

    return (
        f"INTO {table} ({column_list}) SELECT {column_list} FROM {table} "
        f"WHERE rowid = (SELECT min(rowid) FROM {table})"
    )


def unique_keys_of(room, table):
    """Names of the UNIQUE indexes of a table other than its primary key."""
    return [
        row["name"]
        for row in room.execute(f"PRAGMA index_list({table})")
        if row["unique"] == 1 and row["origin"] == "u"
    ]


@scenario("Wiersza teczki nie zastępuje INSERT OR REPLACE ani upsert; wyjątkiem są poprawki D14")
def no_replacement():
    room = new_room()
    full_case(room)

    tables = [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'sprawa\\_%' ESCAPE '\\'"
        )
    ]
    refusal = "wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE"

    for table in tables:
        assert room.value(f"SELECT count(*) FROM {table}") > 0, f"{table} has no row to replace"
        if table in CORRECTABLE_TABLES:
            continue

        # The workaround from the audit: REPLACE deletes the old row without the triggers of
        # DELETE and UPDATE, and an upsert updates it without the triggers of UPDATE.
        room.refused_sql(refusal, f"INSERT OR REPLACE {first_row_copy(room, table)}")
        room.refused_sql(
            refusal,
            f"INSERT {first_row_copy(room, table)} ON CONFLICT (id) DO UPDATE SET id = excluded.id",
        )
        if unique_keys_of(room, table):
            room.refused_sql(
                refusal,
                f"INSERT OR REPLACE {first_row_copy(room, table, replace_unique_key=True)}",
            )

    # Corrections D14 may replace their rows.
    for table in sorted(CORRECTABLE_TABLES):
        rows_before = room.value(f"SELECT count(*) FROM {table}")
        room.execute(f"INSERT OR REPLACE {first_row_copy(room, table)}")
        room.execute(
            f"INSERT {first_row_copy(room, table)} ON CONFLICT (id) DO UPDATE SET id = excluded.id"
        )
        assert room.value(f"SELECT count(*) FROM {table}") == rows_before, table
