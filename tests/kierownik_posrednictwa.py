"""Scenarios of the brokerage head's memory room
(spec/positions/posrednictwo/kierownik_posrednictwa).

The room has the head's own kinds, three kinds of the agent memory and all seven kinds of the
department memory; the rules of the shared kinds are checked in stanowisko.py and kierownik_dzialu.py. Each scenario
starts from an empty room and checks one rule the database keeps: a write the rule allows and a
write it refuses. The first scenario follows a card from the office manager to a broker's new case
file with its keeper, and an offer and a search that meet in one transaction.
"""

from pathlib import Path

from stanowisko import first_row_copy
from kierownik_dzialu import add_case_to_list, appoint_agent
from room import Room

KIEROWNIK_POSREDNICTWA = (
    Path(__file__).resolve().parents[1]
    / "spec"
    / "positions"
    / "posrednictwo"
    / "kierownik_posrednictwa"
)

SCENARIOS = []

REPLACEMENT = "wiersza pamięci nie zastępuje się nowym wpisem"
DELETION = "wierszy pamięci się nie usuwa (R9)"


def scenario(title):
    def register(function):
        SCENARIOS.append((f"kierownik_posrednictwa: {title}", function))

        return function

    return register


def new_room():
    return Room(KIEROWNIK_POSREDNICTWA)


# Building blocks ---------------------------------------------------------------------------------


def broker_rule_fields(**fields):
    return {
        "posrednik": "tozsamosc-posrednika",
        "obszar": "Wrocław, Krzyki",
        "rodzaje_nieruchomosci": "mieszkania",
        "limit_spraw": 20,
        "przyjmuje_nowe": 1,
        "od": "2026-11-01T00:00:00Z",
        "wiadomosc_odnosnik": "wiadomosc-reguly-posrednika",
        "podpis": "podpis-kluczem-posrednika",
        **fields,
    }


def add_broker_rule(room, **fields):
    room.run("zapisz_regule_posrednika", **broker_rule_fields(**fields))


def take_card(room, **fields):
    room.run(
        "przyjmij_karte",
        **{
            "karta_odnosnik": "pokoj-kierownika-biura/karta-1",
            "otrzymano": "2026-11-04T10:00:00Z",
            "kontakt": "Anna Nowak, +48 600 000 000",
            **fields,
        },
    )


def propose_broker(room, card_id=1, **fields):
    room.run(
        "zaproponuj_posrednika",
        **{
            "id": card_id,
            "posrednik": "tozsamosc-posrednika",
            "regula_id": 1,
            "zaproponowano": "2026-11-04T10:30:00Z",
            **fields,
        },
    )


def human_approval():
    return {
        "zatwierdzil": "tozsamosc-wlasciciela",
        "podpis": "podpis-kluczem-wlasciciela",
        "zatwierdzono": "2026-11-04T10:20:00Z",
    }


def add_pair(room, **fields):
    room.run(
        "zapisz_pare",
        **{
            "teczka_oferty_odnosnik": "pokoj-teczki-oferty-1",
            "teczka_poszukiwania_odnosnik": "pokoj-teczki-poszukiwania-1",
            "stwierdzono": "2026-11-10T09:00:00Z",
            **fields,
        },
    )


def both_contracts_allow():
    return {
        "umowy_pozwalaja": "tak",
        "umowa_oferty_odnosnik": "pokoj-teczki-oferty-1/umowa",
        "umowa_poszukiwania_odnosnik": "pokoj-teczki-poszukiwania-1/umowa",
    }


def full_memory(room):
    """One row in every table of the head's own kinds, for the checks of R9 and replacement."""
    add_broker_rule(room)
    take_card(room)
    add_pair(room)


# A whole day ---------------------------------------------------------------------------------------


@scenario("kierownik proponuje pośrednika, powołuje opiekuna i pilnuje obu stron transakcji")
def whole_day():
    room = new_room()

    add_broker_rule(room)
    take_card(room)
    propose_broker(room)
    room.run(
        "odnotuj_zalozenie_teczki",
        id=1,
        teczka_odnosnik="pokoj-teczki-oferty-1",
        rozstrzygnieto="2026-11-04T12:00:00Z",
    )
    appoint_agent(
        room,
        "opiekun-oferty-1",
        wlasciciel="tozsamosc-posrednika",
        teczka_odnosnik="pokoj-teczki-oferty-1",
    )
    add_case_to_list(room, teczka_odnosnik="pokoj-teczki-oferty-1")

    add_pair(room)
    room.run("ocen_umowy_pary", id=1, **both_contracts_allow())
    room.run("odnotuj_przekazanie_pary", id=1, przekazano="2026-11-10T12:00:00Z")
    room.run("odnotuj_teczke_transakcji", id=1, transakcja_odnosnik="pokoj-transakcji-1")

    card = room.execute("SELECT stan, kontakt FROM kierownik_posrednictwa_przydzial_karty")[0]
    assert card == {"stan": "przyjeta", "kontakt": None}
    assert room.run("karty_otwarte") == []
    assert room.run("obciazenie_posrednikow") == [
        {
            "regula_id": 1,
            "posrednik": "tozsamosc-posrednika",
            "limit_spraw": 20,
            "przyjmuje_nowe": 1,
            "sprawy": 1,
        }
    ]
    assert room.run("pary_do_wyjasnienia") == []
    assert room.run("pary")[0]["transakcja_odnosnik"] == "pokoj-transakcji-1"


# Broker rules --------------------------------------------------------------------------------------


@scenario("reguła pośrednika powstaje aktualna, jest jedna i ma dodatni limit spraw")
def one_rule_per_broker():
    room = new_room()
    add_broker_rule(room)

    room.refused(
        "ten pośrednik ma już aktualną regułę", "zapisz_regule_posrednika", **broker_rule_fields()
    )
    room.refused(
        "limit_spraw",
        "zapisz_regule_posrednika",
        **broker_rule_fields(posrednik="tozsamosc-drugiego", limit_spraw=0),
    )
    room.refused_sql(
        "reguła pośrednika powstaje aktualna",
        """
        INSERT INTO kierownik_posrednictwa_przydzial_posrednicy (posrednik, obszar,
          rodzaje_nieruchomosci, limit_spraw, przyjmuje_nowe, od, wiadomosc_odnosnik, podpis,
          zastapione_od)
        VALUES ('p', 'o', 'r', 5, 1, '2026-11-01', 'w', 's', '2026-11-02')
        """,
    )

    room.refused(
        "zastapienie_nie_przed_poczatkiem",
        "zastap_regule_posrednika",
        id=1,
        zastapione_od="2026-10-01T00:00:00Z",
    )
    room.run("zastap_regule_posrednika", id=1, zastapione_od="2026-12-01T00:00:00Z")
    add_broker_rule(room, limit_spraw=10, od="2026-12-01T00:00:00Z")
    assert [row["limit_spraw"] for row in room.run("posrednicy")] == [10]
    assert len(room.run("historia_regul_posrednika", posrednik="tozsamosc-posrednika")) == 2


@scenario("reguły pośrednika się nie zmienia, a zastąpienie odnotowuje się raz")
def broker_rule_does_not_change():
    room = new_room()
    add_broker_rule(room)

    rule = "reguły pośrednika się nie zmienia"
    room.refused_sql(
        rule, "UPDATE kierownik_posrednictwa_przydzial_posrednicy SET limit_spraw = 99"
    )
    room.run("zastap_regule_posrednika", id=1, zastapione_od="2026-12-01T00:00:00Z")
    room.refused(rule, "zastap_regule_posrednika", id=1, zastapione_od="2026-12-02T00:00:00Z")


# Cards ---------------------------------------------------------------------------------------------


@scenario("karta powstaje czekająca, a propozycja według reguły wskazuje jej pośrednika")
def proposal_follows_the_broker_rule():
    room = new_room()
    add_broker_rule(room)
    add_broker_rule(room, posrednik="tozsamosc-pelnego", przyjmuje_nowe=0)
    take_card(room)

    room.refused_sql(
        "karta powstaje jako czekająca",
        """
        INSERT INTO kierownik_posrednictwa_przydzial_karty (karta_odnosnik, otrzymano, stan, powod,
          rozstrzygnieto)
        VALUES ('karta-9', '2026-11-04', 'odrzucona', 'p', '2026-11-04')
        """,
    )

    rule = "propozycja według reguły wskazuje pośrednika aktualnej reguły"
    room.refused(rule, "zaproponuj_posrednika", **proposal_fields(posrednik="tozsamosc-innego"))
    room.refused(
        rule,
        "zaproponuj_posrednika",
        **proposal_fields(posrednik="tozsamosc-pelnego", regula_id=2),
    )
    room.run("zastap_regule_posrednika", id=1, zastapione_od="2026-11-04T00:00:00Z")
    room.refused(rule, "zaproponuj_posrednika", **proposal_fields())

    add_broker_rule(room, od="2026-11-04T00:00:00Z")
    room.run("zaproponuj_posrednika", **proposal_fields(regula_id=3))


def proposal_fields(**fields):
    return {
        "id": 1,
        "posrednik": "tozsamosc-posrednika",
        "regula_id": 1,
        "zaproponowano": "2026-11-04T10:30:00Z",
        **fields,
    }


@scenario("pośrednik spoza reguł wymaga zgody człowieka, a propozycja ma pośrednika i datę")
def broker_outside_rules_needs_human():
    room = new_room()
    take_card(room)

    room.refused(
        "posrednik_poza_regulami_za_zgoda_czlowieka",
        "zaproponuj_posrednika",
        **proposal_fields(regula_id=None),
    )
    room.refused(
        "propozycja_z_posrednikiem",
        "zaproponuj_posrednika",
        **proposal_fields(regula_id=None, zaproponowano=None, **human_approval()),
    )
    room.refused(
        "zatwierdzenie_w_komplecie",
        "zaproponuj_posrednika",
        **proposal_fields(regula_id=None, zatwierdzil="tozsamosc-wlasciciela"),
    )
    room.run("zaproponuj_posrednika", **proposal_fields(regula_id=None, **human_approval()))

    room.run("cofnij_propozycje", id=1)
    assert [row["stan"] for row in room.run("karty_otwarte")] == ["czeka"]


@scenario("przyjęta karta ma teczkę, odrzucona powód, a obie są bez kontaktu i bez zmian")
def resolved_card_has_no_contact():
    room = new_room()
    add_broker_rule(room)
    take_card(room)
    take_card(room, karta_odnosnik="pokoj-kierownika-biura/karta-2")
    propose_broker(room)

    room.refused(
        "przyjeta_z_teczka", "odnotuj_zalozenie_teczki", id=1, rozstrzygnieto="2026-11-04T12:00:00Z"
    )
    room.refused(
        "rozstrzygnieta_z_data",
        "odnotuj_zalozenie_teczki",
        id=1,
        teczka_odnosnik="pokoj-teczki-oferty-1",
    )
    room.refused(
        "rozstrzygniecie_nie_przed_otrzymaniem",
        "odnotuj_zalozenie_teczki",
        id=1,
        teczka_odnosnik="pokoj-teczki-oferty-1",
        rozstrzygnieto="2026-11-03T12:00:00Z",
    )
    room.refused_sql(
        "rozstrzygnieta_bez_kontaktu",
        """
        UPDATE kierownik_posrednictwa_przydzial_karty
        SET stan = 'przyjeta', teczka_odnosnik = 't', rozstrzygnieto = '2026-11-04T12:00:00Z'
        WHERE id = 1
        """,
    )
    room.refused("odrzucona_z_powodem", "odrzuc_karte", id=2, rozstrzygnieto="2026-11-04T12:00:00Z")

    room.run(
        "odnotuj_zalozenie_teczki",
        id=1,
        teczka_odnosnik="pokoj-teczki-oferty-1",
        rozstrzygnieto="2026-11-04T12:00:00Z",
    )
    room.run(
        "odrzuc_karte",
        id=2,
        powod="Zgłoszenie spoza obszaru biura",
        rozstrzygnieto="2026-11-04T12:00:00Z",
    )

    rule = "karty od Kierownika biura się nie zmienia, a przyjętej ani odrzuconej wcale"
    room.refused(rule, "cofnij_propozycje", id=1)
    room.refused(rule, "zaproponuj_posrednika", **proposal_fields(id=2))
    room.refused_sql(
        rule,
        "UPDATE kierownik_posrednictwa_przydzial_karty SET otrzymano = '2026-11-01' WHERE id = 1",
    )


# Both sides ----------------------------------------------------------------------------------------


@scenario("para powstaje do wyjaśnienia, a „tak” wskazuje obie umowy")
def pair_starts_unclear():
    room = new_room()
    add_pair(room)

    room.refused_sql(
        "para powstaje do wyjaśnienia",
        """
        INSERT INTO kierownik_posrednictwa_obie_strony_pary (teczka_oferty_odnosnik,
          teczka_poszukiwania_odnosnik, stwierdzono, umowy_pozwalaja, umowa_oferty_odnosnik,
          umowa_poszukiwania_odnosnik)
        VALUES ('o2', 'p2', '2026-11-10', 'tak', 'u1', 'u2')
        """,
    )
    room.refused(
        "zgoda_umow_wskazuje_obie_umowy",
        "ocen_umowy_pary",
        id=1,
        umowy_pozwalaja="tak",
        umowa_oferty_odnosnik="pokoj-teczki-oferty-1/umowa",
    )
    room.run(
        "ocen_umowy_pary",
        id=1,
        umowy_pozwalaja="nie",
        umowa_oferty_odnosnik="pokoj-teczki-oferty-1/umowa",
    )
    room.run("ocen_umowy_pary", id=1, **both_contracts_allow())
    room.refused_sql(
        "teczek pary się nie zmienia",
        "UPDATE kierownik_posrednictwa_obie_strony_pary SET teczka_oferty_odnosnik = 'inna' WHERE id = 1",
    )
    assert room.run("pary_do_wyjasnienia") == []


@scenario(
    "ofertę przekazuje się dalej za zgodą obu umów albo człowieka, a potem się jej nie zmienia"
)
def handover_needs_both_contracts_or_human():
    room = new_room()
    add_pair(room)
    add_pair(room, teczka_poszukiwania_odnosnik="pokoj-teczki-poszukiwania-2")
    room.run("ocen_umowy_pary", id=2, umowy_pozwalaja="nie")

    rule = "przekazanie_za_zgoda_umow_albo_czlowieka"
    room.refused(rule, "odnotuj_przekazanie_pary", id=1, przekazano="2026-11-10T12:00:00Z")
    room.refused(rule, "odnotuj_przekazanie_pary", id=2, przekazano="2026-11-10T12:00:00Z")
    room.refused(
        "przekazanie_nie_przed_stwierdzeniem",
        "odnotuj_przekazanie_pary",
        id=2,
        przekazano="2026-11-09T12:00:00Z",
        **human_approval(),
    )
    room.refused(
        "transakcja_po_przekazaniu",
        "odnotuj_teczke_transakcji",
        id=2,
        transakcja_odnosnik="pokoj-transakcji-2",
    )
    room.run(
        "odnotuj_przekazanie_pary",
        id=2,
        przekazano="2026-11-10T12:00:00Z",
        **human_approval(),
    )

    rule = "po przekazaniu także oceny umów"
    room.refused(rule, "ocen_umowy_pary", id=2, **both_contracts_allow())
    room.run("odnotuj_teczke_transakcji", id=2, transakcja_odnosnik="pokoj-transakcji-2")
    room.refused(rule, "odnotuj_teczke_transakcji", id=2, transakcja_odnosnik="pokoj-transakcji-3")


# R9 and replacement --------------------------------------------------------------------------------


def own_tables_of(room):
    return [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' "
            "AND name LIKE 'kierownik\\_posrednictwa\\_%' ESCAPE '\\'"
        )
    ]


@scenario("R9: wierszy pamięci się nie usuwa ani nie zastępuje przez REPLACE albo upsert")
def no_deletion_or_replacement():
    room = new_room()
    full_memory(room)

    tables = own_tables_of(room)
    assert len(tables) == 3, tables

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


@scenario("REPLACE nie usuwa aktualnej reguły pośrednika, karty ani pary przez klucz UNIQUE")
def no_replacement_through_unique_keys():
    room = new_room()
    full_memory(room)

    for table, key, refusal in [
        ("kierownik_posrednictwa_przydzial_karty", "karta_odnosnik", REPLACEMENT),
        (
            "kierownik_posrednictwa_obie_strony_pary",
            "teczka_oferty_odnosnik, teczka_poszukiwania_odnosnik",
            REPLACEMENT,
        ),
        (
            "kierownik_posrednictwa_przydzial_posrednicy",
            None,
            "ten pośrednik ma już aktualną regułę",
        ),
    ]:
        columns = [row["name"] for row in room.execute(f"PRAGMA table_info({table})")]
        selected = ", ".join("99" if name == "id" else name for name in columns)
        room.refused_sql(
            refusal,
            f"INSERT OR REPLACE INTO {table} ({', '.join(columns)}) SELECT {selected} FROM {table}",
        )
        if key is not None:
            room.refused_sql(
                refusal,
                # `WHERE true` keeps SQLite from reading ON CONFLICT as a join constraint.
                f"INSERT INTO {table} ({', '.join(columns)}) SELECT {selected} FROM {table} "
                f"WHERE true ON CONFLICT ({key}) DO UPDATE SET id = excluded.id",
            )
