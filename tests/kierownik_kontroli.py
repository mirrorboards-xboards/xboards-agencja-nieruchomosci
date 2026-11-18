"""Scenarios of the control head's memory room (spec/positions/kontrola/kierownik_kontroli).

The room has the head's own kinds, three kinds of the agent memory and five kinds of the department
memory (no case list for Reception and no policy copies, D11); the rules of the shared kinds are
checked in stanowisko.py and kierownik_dzialu.py. Each scenario starts from an empty room and checks one rule the
database keeps: a write the rule allows and a write it refuses. The first scenario follows the
office's risk assessment to the control register, a notification draft that the owner hands over
and a person's request about their data.
"""

from pathlib import Path

from stanowisko import first_row_copy
from kierownik_dzialu import appoint_controller, forward_request, receive_report
from room import Room

KIEROWNIK_KONTROLI = (
    Path(__file__).resolve().parents[1] / "spec" / "positions" / "kontrola" / "kierownik_kontroli"
)

SCENARIOS = []

REPLACEMENT = "wiersza pamięci nie zastępuje się nowym wpisem"
DELETION = "wierszy pamięci się nie usuwa (R9)"


def scenario(title):
    def register(function):
        SCENARIOS.append((f"kierownik_kontroli: {title}", function))

        return function

    return register


def new_room():
    return Room(KIEROWNIK_KONTROLI)


# Building blocks ---------------------------------------------------------------------------------


def start_assessment(room, **fields):
    room.run("rozpocznij_projekt_oceny", **{"rozpoczeto": "2026-11-02T09:00:00Z", **fields})


def add_factor(room, ocena_id=1, **fields):
    room.run(
        "dopisz_czynnik",
        **{
            "ocena_id": ocena_id,
            "obszar": "klienci",
            "czynnik": "Klienci spoza Europejskiego Obszaru Gospodarczego",
            "poziom": "srednie",
            "srodki": "Wzmożona weryfikacja źródła środków",
            "zrodlo": "Krajowa ocena ryzyka",
            **fields,
        },
    )


def create_draft(room, **fields):
    room.run(
        "utworz_projekt",
        **{
            "rodzaj": "art74",
            "teczka_odnosnik": "pokoj-transakcji-1",
            "termin": "2026-11-06",
            "utworzono": "2026-11-04T10:00:00Z",
            "sciezka": "projekty/art74-1.typ",
            "skrot": "sha256-projekt-1",
            **fields,
        },
    )


def owner_withdrawal():
    return {
        "odstapil": "tozsamosc-wlasciciela",
        "odstapienie_podpis": "podpis-kluczem-wlasciciela",
        "odstapiono": "2026-11-05T10:00:00Z",
    }


def add_data_request(room, **fields):
    room.run(
        "zapisz_sprawe_danych_osobowych",
        **{
            "rodzaj": "dostep_art15",
            "wplynelo": "2026-11-04T09:00:00Z",
            "wplyw_odnosnik": "wiadomosc-prosba-o-dane",
            "termin": "2026-12-04",
            **fields,
        },
    )


def full_memory(room):
    """One row in every table of the head's own kinds, for the checks of R9 and replacement."""
    start_assessment(room)
    add_factor(room)
    create_draft(room)
    room.run("dopisz_podstawe", projekt_id=1, rejestr_odnosnik="pokoj-rejestru/sprawdzenie-1")
    add_data_request(room)


# A whole day ---------------------------------------------------------------------------------------


@scenario("ocena ryzyka trafia do rejestru, właściciel przekazuje zawiadomienie i załatwia prośbę")
def whole_day():
    room = new_room()

    appoint_controller(room, "kontroler-1")
    receive_report(
        room,
        "raport-1",
        "kontroler-1",
        etap_albo_okres="weryfikacja",
        decyzja_odnosnik="pokoj-transakcji-1/decyzja-1",
    )
    forward_request(
        room, decyzja_odnosnik="pokoj-transakcji-1/decyzja-1", przekazano_do="wlasciciel"
    )

    start_assessment(room)
    add_factor(room)
    add_factor(
        room, obszar="kanaly_dostaw", czynnik="Umowy zawierane na odległość", poziom="wysokie"
    )
    room.run("odnotuj_przeglad_oceny", id=1, przejrzano="2026-11-03T09:00:00Z")
    room.run(
        "przekaz_ocene_do_zatwierdzenia", id=1, przekazano_do_zatwierdzenia="2026-11-03T10:00:00Z"
    )
    room.run("odnotuj_zatwierdzenie_oceny", id=1, rejestr_odnosnik="pokoj-rejestru/ocena-biura-1")

    create_draft(room)
    room.run("dopisz_podstawe", projekt_id=1, rejestr_odnosnik="pokoj-rejestru/sprawdzenie-1")
    room.run(
        "dopisz_podstawe", projekt_id=1, rejestr_odnosnik="pokoj-rejestru/decyzja-potwierdzenia"
    )
    room.run("daj_projekt_wlascicielowi", id=1, u_wlasciciela_od="2026-11-04T11:00:00Z")
    room.run(
        "odnotuj_przekazanie_zawiadomienia",
        id=1,
        przekazano="2026-11-05T09:00:00Z",
        rejestr_odnosnik="pokoj-rejestru/zawiadomienie-1",
    )

    add_data_request(room)
    room.run("daj_sprawe_wlascicielowi", id=1)
    room.run(
        "odnotuj_zalatwienie",
        id=1,
        odpowiedz_odnosnik="wiadomosc-odpowiedz-wlasciciela",
        zalatwiono="2026-11-20T09:00:00Z",
    )

    assert room.run("prosby_na_liscie")[0]["przekazano_do"] == "wlasciciel"
    assert len(room.run("czynniki_oceny", ocena_id=1)) == 2
    assert room.run("projekty_w_toku") == []
    draft = room.run("projekty")[0]
    assert (draft["stan"], draft["sciezka"], draft["skrot"]) == ("przekazany", None, None)
    assert len(room.run("podstawy_projektu", projekt_id=1)) == 2
    assert room.run("sprawy_danych_osobowych")[0]["stan"] == "zalatwiona"


# Risk assessment of the office ---------------------------------------------------------------------


@scenario("projekt oceny powstaje roboczy, a zatwierdzenie przychodzi po przekazaniu")
def assessment_is_approved_after_handover():
    room = new_room()
    start_assessment(room)

    room.refused_sql(
        "projekt oceny powstaje roboczy",
        """
        INSERT INTO kierownik_kontroli_oceny_wpisy (rozpoczeto, przekazano_do_zatwierdzenia)
        VALUES ('2026-11-02', '2026-11-03')
        """,
    )
    room.refused(
        "przeglad_nie_przed_rozpoczeciem",
        "odnotuj_przeglad_oceny",
        id=1,
        przejrzano="2026-11-01T09:00:00Z",
    )
    room.refused(
        "przekazanie_nie_przed_rozpoczeciem",
        "przekaz_ocene_do_zatwierdzenia",
        id=1,
        przekazano_do_zatwierdzenia="2026-11-01T09:00:00Z",
    )
    room.refused(
        "zatwierdzona_po_przekazaniu",
        "odnotuj_zatwierdzenie_oceny",
        id=1,
        rejestr_odnosnik="pokoj-rejestru/ocena-biura-1",
    )
    room.run(
        "przekaz_ocene_do_zatwierdzenia", id=1, przekazano_do_zatwierdzenia="2026-11-03T10:00:00Z"
    )
    room.run("odnotuj_zatwierdzenie_oceny", id=1, rejestr_odnosnik="pokoj-rejestru/ocena-biura-1")
    assert room.run("projekty_ocen")[0]["rejestr_odnosnik"] == "pokoj-rejestru/ocena-biura-1"


@scenario("zatwierdzonego projektu oceny ani jego czynników się nie zmienia")
def approved_assessment_does_not_change():
    room = new_room()
    start_assessment(room)
    start_assessment(room, rozpoczeto="2026-11-03T09:00:00Z")
    add_factor(room)
    room.run(
        "zmien_czynnik",
        id=1,
        obszar="klienci",
        czynnik="Klienci spoza EOG",
        poziom="wysokie",
        zrodlo="Krajowa ocena ryzyka",
    )
    room.refused_sql(
        "czynnik nie przechodzi do innego projektu",
        "UPDATE kierownik_kontroli_oceny_czynniki SET ocena_id = 2 WHERE id = 1",
    )

    room.run(
        "przekaz_ocene_do_zatwierdzenia", id=1, przekazano_do_zatwierdzenia="2026-11-03T10:00:00Z"
    )
    room.run("odnotuj_zatwierdzenie_oceny", id=1, rejestr_odnosnik="pokoj-rejestru/ocena-biura-1")

    rule = "projektu oceny zatwierdzonej w rejestrze kontroli się nie zmienia"
    room.refused(rule, "odnotuj_przeglad_oceny", id=1, przejrzano="2026-11-04T09:00:00Z")
    room.refused(rule, "dopisz_czynnik", **factor_fields())
    room.refused(rule, "zmien_czynnik", id=1, **factor_fields(ocena_id=None))
    room.run("dopisz_czynnik", **factor_fields(ocena_id=2))


def factor_fields(**fields):
    values = {
        "ocena_id": 1,
        "obszar": "uslugi",
        "czynnik": "Zarządzanie najmem krótkoterminowym",
        "poziom": "niskie",
        "zrodlo": "Raport Kontrolera",
        **fields,
    }

    return {name: value for name, value in values.items() if value is not None}


# Notification drafts -------------------------------------------------------------------------------


@scenario("projekt zawiadomienia powstaje jako projekt z plikiem, a u właściciela ma datę")
def draft_starts_with_file():
    room = new_room()
    create_draft(room)

    room.refused_sql(
        "projekt zawiadomienia powstaje w stanie projekt",
        """
        INSERT INTO kierownik_kontroli_projekty_wpisy (rodzaj, termin, utworzono, stan, przekazano,
          rejestr_odnosnik)
        VALUES ('art86', '2026-11-06', '2026-11-04', 'przekazany', '2026-11-05', 'r')
        """,
    )
    room.refused("sciezka_razem_ze_skrotem", "utworz_projekt", **draft_fields(skrot=None))
    room.refused("projekt_z_plikiem", "utworz_projekt", **draft_fields(sciezka=None, skrot=None))
    room.refused("u_wlasciciela_z_data", "daj_projekt_wlascicielowi", id=1)
    room.run("daj_projekt_wlascicielowi", id=1, u_wlasciciela_od="2026-11-04T11:00:00Z")
    room.run("cofnij_projekt_do_poprawy", id=1)
    room.run(
        "zmien_projekt",
        id=1,
        termin="2026-11-06",
        sciezka="projekty/art74-1-v2.typ",
        skrot="sha256-v2",
    )
    assert room.run("projekty_w_toku")[0]["skrot"] == "sha256-v2"


def draft_fields(**fields):
    return {
        "rodzaj": "art86",
        "termin": "2026-11-06",
        "utworzono": "2026-11-04T10:00:00Z",
        "sciezka": "projekty/art86-1.typ",
        "skrot": "sha256",
        **fields,
    }


@scenario("przekazany projekt wskazuje rejestr i nie ma pliku; przekazanego się nie zmienia")
def handed_over_draft_points_to_register():
    room = new_room()
    create_draft(room)
    room.run("daj_projekt_wlascicielowi", id=1, u_wlasciciela_od="2026-11-04T11:00:00Z")

    room.refused(
        "przekazany_z_wpisem_w_rejestrze",
        "odnotuj_przekazanie_zawiadomienia",
        id=1,
        przekazano="2026-11-05T09:00:00Z",
    )
    room.refused_sql(
        "przekazany_bez_tresci",
        """
        UPDATE kierownik_kontroli_projekty_wpisy
        SET stan = 'przekazany', przekazano = '2026-11-05T09:00:00Z', rejestr_odnosnik = 'r'
        WHERE id = 1
        """,
    )
    room.refused(
        "przekazanie_nie_przed_utworzeniem",
        "odnotuj_przekazanie_zawiadomienia",
        id=1,
        przekazano="2026-11-03T09:00:00Z",
        rejestr_odnosnik="pokoj-rejestru/zawiadomienie-1",
    )
    room.run(
        "odnotuj_przekazanie_zawiadomienia",
        id=1,
        przekazano="2026-11-05T09:00:00Z",
        rejestr_odnosnik="pokoj-rejestru/zawiadomienie-1",
    )

    rule = "a przekazanego i odstąpionego wcale"
    room.refused(rule, "cofnij_projekt_do_poprawy", id=1)
    room.refused(rule, "odnotuj_odstapienie", id=1, **owner_withdrawal())


@scenario("od projektu odstępuje właściciel z podpisem, a odstąpionego się nie zmienia")
def withdrawal_is_signed_by_owner():
    room = new_room()
    create_draft(room)

    rule = "odstapienie_z_podpisem_wlasciciela"
    room.refused(
        rule,
        "odnotuj_odstapienie",
        id=1,
        odstapil="tozsamosc-wlasciciela",
        odstapiono="2026-11-05T10:00:00Z",
    )
    room.refused_sql(
        rule, "UPDATE kierownik_kontroli_projekty_wpisy SET stan = 'odstapiono' WHERE id = 1"
    )
    room.run("odnotuj_odstapienie", id=1, **owner_withdrawal())

    room.refused(
        "a przekazanego i odstąpionego wcale",
        "zmien_projekt",
        id=1,
        termin="2026-11-07",
        sciezka="s",
        skrot="k",
    )
    room.refused_sql(
        "rodzaju, teczki ani daty projektu się nie zmienia",
        "UPDATE kierownik_kontroli_projekty_wpisy SET rodzaj = 'art90' WHERE id = 1",
    )


@scenario("podstawę dopisuje się raz i tylko do projektu w toku")
def basis_only_for_draft_in_progress():
    room = new_room()
    create_draft(room)
    create_draft(room, utworzono="2026-11-04T12:00:00Z")
    room.run("dopisz_podstawe", projekt_id=1, rejestr_odnosnik="pokoj-rejestru/sprawdzenie-1")
    room.run("odnotuj_odstapienie", id=2, **owner_withdrawal())

    room.refused(
        "podstawę dopisuje się tylko do projektu w toku",
        "dopisz_podstawe",
        projekt_id=2,
        rejestr_odnosnik="pokoj-rejestru/sprawdzenie-1",
    )
    room.refused(
        REPLACEMENT,
        "dopisz_podstawe",
        projekt_id=1,
        rejestr_odnosnik="pokoj-rejestru/sprawdzenie-1",
    )
    room.refused_sql(
        "podstawy projektu się nie zmienia",
        "UPDATE kierownik_kontroli_projekty_podstawy SET rejestr_odnosnik = 'inny' WHERE id = 1",
    )


# Personal data -------------------------------------------------------------------------------------


@scenario("sprawa danych osobowych powstaje otwarta, a załatwia ją odpowiedź właściciela")
def data_request_needs_owner_answer():
    room = new_room()
    add_data_request(room)
    add_data_request(room, rodzaj="naruszenie", termin="2026-11-07T09:00:00Z")

    room.refused_sql(
        "sprawa danych osobowych powstaje otwarta",
        """
        INSERT INTO kierownik_kontroli_dane_osobowe_wpisy (rodzaj, wplynelo, wplyw_odnosnik, termin,
          stan, odpowiedz_odnosnik, zalatwiono)
        VALUES ('sprzeciw_art21', '2026-11-04', 'w', '2026-12-04', 'zalatwiona', 'o', '2026-11-05')
        """,
    )
    room.refused(
        "termin_nie_przed_wplywem",
        "zapisz_sprawe_danych_osobowych",
        rodzaj="usuniecie_art17",
        wplynelo="2026-11-04T09:00:00Z",
        wplyw_odnosnik="w",
        termin="2026-11-01",
    )
    room.refused(
        "zalatwiona_z_odpowiedzia_wlasciciela",
        "odnotuj_zalatwienie",
        id=1,
        zalatwiono="2026-11-20T09:00:00Z",
    )
    room.refused(
        "zalatwienie_nie_przed_wplywem",
        "odnotuj_zalatwienie",
        id=1,
        odpowiedz_odnosnik="o",
        zalatwiono="2026-11-01T09:00:00Z",
    )
    room.run("odnotuj_zalatwienie", id=1, odpowiedz_odnosnik="o", zalatwiono="2026-11-20T09:00:00Z")

    rule = "a załatwionej wcale"
    room.refused(rule, "daj_sprawe_wlascicielowi", id=1)
    room.refused_sql(
        "rodzaju ani wpływu sprawy danych osobowych się nie zmienia",
        "UPDATE kierownik_kontroli_dane_osobowe_wpisy SET rodzaj = 'dostep_art15' WHERE id = 2",
    )
    assert [row["rodzaj"] for row in room.run("sprawy_danych_osobowych")] == [
        "naruszenie",
        "dostep_art15",
    ]


# R9 and replacement --------------------------------------------------------------------------------


def own_tables_of(room):
    return [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' "
            "AND name LIKE 'kierownik\\_kontroli\\_%' ESCAPE '\\'"
        )
    ]


@scenario("R9: wierszy pamięci się nie usuwa ani nie zastępuje przez REPLACE albo upsert")
def no_deletion_or_replacement():
    room = new_room()
    full_memory(room)

    tables = own_tables_of(room)
    assert len(tables) == 5, tables

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


@scenario("REPLACE nie usuwa podstawy projektu przez klucz UNIQUE")
def no_replacement_through_unique_basis():
    room = new_room()
    full_memory(room)

    table = "kierownik_kontroli_projekty_podstawy"
    room.refused_sql(
        REPLACEMENT,
        f"INSERT OR REPLACE INTO {table} (id, projekt_id, rejestr_odnosnik) "
        f"SELECT 99, projekt_id, rejestr_odnosnik FROM {table}",
    )
    room.refused_sql(
        REPLACEMENT,
        # `WHERE true` keeps SQLite from reading ON CONFLICT as a join constraint.
        f"INSERT INTO {table} (id, projekt_id, rejestr_odnosnik) "
        f"SELECT 99, projekt_id, rejestr_odnosnik FROM {table} "
        "WHERE true ON CONFLICT (projekt_id, rejestr_odnosnik) DO UPDATE SET id = excluded.id",
    )
