"""Scenarios of the memory the agent of every position keeps
(spec/positions/stanowisko).

The catalog goes into the memory room of every position, so its rules are checked once here, in a
room of this catalog alone. Each scenario starts from an empty room and checks one rule the
database keeps: a write the rule allows and a write it refuses. The first scenario follows an agent
that learns from a source, finds out the source lied and assesses its findings again.
"""

from pathlib import Path

from guard import refuse_replacing_every_row
from room import Room

STANOWISKO = Path(__file__).resolve().parents[1] / "spec" / "positions" / "stanowisko"

SCENARIOS = []


def scenario(title):
    def register(function):
        SCENARIOS.append((f"agent: {title}", function))

        return function

    return register


def new_room():
    return Room(STANOWISKO)


# Building blocks, also for the scenarios of the positions ------------------------------------------


def add_source(room, source_id, **fields):
    room.run(
        "wpisz_zrodlo",
        **{
            "id": source_id,
            "nazwa": "Bank Przykładowy, dział kredytów",
            "rodzaj": "instytucja",
            **fields,
        },
    )


def add_source_event(room, source_id, rodzaj, zaszlo="2026-11-05T10:00:00Z", **fields):
    room.run(
        "dopisz_zdarzenie_zrodla",
        **{
            "zrodlo_id": source_id,
            "zaszlo": zaszlo,
            "rodzaj": rodzaj,
            "opis": f"Zdarzenie {rodzaj}",
            **fields,
        },
    )


def add_finding(room, finding_id, **fields):
    room.run(
        "zapisz_ustalenie",
        **{
            "id": finding_id,
            "tresc": "Zbywca nabył lokal w spadku",
            "wynika_z_tabela": "sprawa_dokumenty_wpisy",
            "wynika_z_odnosnik": "pokoj-teczki/postanowienie-spadkowe",
            "pewnosc": "prawdopodobne",
            "ustalono": "2026-11-04T09:00:00Z",
            "zapisal": "tozsamosc-agenta",
            **fields,
        },
    )


def accept_order(room, **fields):
    room.run(
        "przyjmij_polecenie",
        **{
            "wydal": "tozsamosc-wlasciciela",
            "wydal_jako": "wlasciciel",
            "otrzymano": "2026-11-04T08:00:00Z",
            "tresc": "Raportuj codziennie o 16:00",
            "podpis": "podpis-kluczem-wlasciciela",
            **fields,
        },
    )


# A whole day ---------------------------------------------------------------------------------------


@scenario("agent ustala ze źródła, po nieprawdzie ocenia źródło i ustalenia na nowo")
def whole_day():
    room = new_room()

    accept_order(room)
    add_source(room, "bank")
    add_source(
        room, "strona", nazwa=None, rodzaj="osoba", uczestnik_odnosnik="pokoj-teczki/sprzedajaca"
    )
    add_finding(room, "saldo", tresc="Saldo kredytu to 120 000 zł", zrodlo_id="bank")
    add_finding(
        room,
        "spadek",
        zrodlo_id="strona",
        pewnosc="potwierdzone",
        sposob_sprawdzenia="postanowienie sądu w teczce",
    )
    room.run(
        "odnotuj_wykorzystanie_ustalenia",
        id="spadek",
        wykorzystano_tabela="sprawa_czynnosci_wersje",
        wykorzystano_odnosnik="pokoj-teczki/wersja-2",
    )

    add_source_event(room, "bank", "nieprawda", opis="Zaświadczenie podało nieaktualne saldo")
    room.run("ocen_zrodlo", id="bank", ocena="nierzetelne", oceniono="2026-11-05T11:00:00Z")
    to_assess = room.run("ustalenia_od_zrodla", zrodlo_id="bank")
    assert [row["id"] for row in to_assess] == ["saldo"]
    room.run("ocen_ustalenie_na_nowo", id="saldo", pewnosc="watpliwe")

    add_finding(
        room,
        "saldo-nowe",
        tresc="Saldo kredytu to 118 000 zł",
        zrodlo_id=None,
        wynika_z_odnosnik="pokoj-teczki/zaswiadczenie-2",
        ustalono="2026-11-06T09:00:00Z",
    )
    room.run(
        "uniewaznij_ustalenie",
        id="saldo",
        nieaktualne_od="2026-11-06T09:00:00Z",
        zastapione_przez_id="saldo-nowe",
    )

    for event in room.run("zdarzenia_do_zgloszenia"):
        room.run("odnotuj_zgloszenie_zdarzenia", id=event["id"], zgloszono="2026-11-06T16:00:00Z")
    room.run("odnotuj_wykonanie_polecenia", id=1, zamknieto="2026-11-06T16:00:00Z")

    assert {row["id"] for row in room.run("ustalenia")} == {"spadek", "saldo-nowe"}
    assert room.run("zrodla")[0]["id"] == "bank"
    assert room.run("polecenia_otwarte") == []


# Findings ----------------------------------------------------------------------------------------


@scenario("ustalenie ma źródło albo wiersz teczki, a wiersz ma tabelę")
def finding_has_source():
    room = new_room()
    add_source(room, "bank")
    add_finding(room, "ze-zrodla", zrodlo_id="bank", wynika_z_tabela=None, wynika_z_odnosnik=None)
    add_finding(room, "z-teczki")

    room.refused(
        "ustalenie_ze_zrodlem",
        "zapisz_ustalenie",
        id="bez-zrodla",
        tresc="Coś",
        pewnosc="watpliwe",
        ustalono="2026-11-04T09:00:00Z",
        zapisal="tozsamosc-agenta",
    )
    room.refused(
        "wynika_z_tabela_razem_z_odnosnikiem",
        "zapisz_ustalenie",
        id="bez-tabeli",
        tresc="Coś",
        wynika_z_odnosnik="pokoj-teczki/wiersz",
        pewnosc="watpliwe",
        ustalono="2026-11-04T09:00:00Z",
        zapisal="tozsamosc-agenta",
    )

    room.run(
        "odnotuj_wykorzystanie_ustalenia",
        id="z-teczki",
        wykorzystano_tabela="sprawa_notatki_wpisy",
        wykorzystano_odnosnik="pokoj-teczki/notatka-1",
    )
    room.refused_sql(
        "wykorzystano_tabela_razem_z_odnosnikiem",
        "UPDATE stanowisko_ustalenia_wpisy SET wykorzystano_odnosnik = 'pokoj-teczki/x' "
        "WHERE id = 'ze-zrodla'",
    )


@scenario("potwierdzone ustalenie ma sposób sprawdzenia")
def confirmed_finding_says_how():
    room = new_room()
    add_finding(room, "potwierdzone", pewnosc="potwierdzone", sposob_sprawdzenia="odpis z KW")

    room.refused(
        "potwierdzone_ze_sposobem_sprawdzenia",
        "zapisz_ustalenie",
        id="bez-sposobu",
        tresc="Coś",
        wynika_z_tabela="sprawa_dokumenty_wpisy",
        wynika_z_odnosnik="pokoj-teczki/wiersz",
        pewnosc="potwierdzone",
        ustalono="2026-11-04T09:00:00Z",
        zapisal="tozsamosc-agenta",
    )
    room.refused(
        "potwierdzone_ze_sposobem_sprawdzenia",
        "ocen_ustalenie_na_nowo",
        id="potwierdzone",
        pewnosc="potwierdzone",
    )


@scenario("treści, źródła, daty ani autora ustalenia się nie zmienia")
def finding_content_is_fixed():
    room = new_room()
    add_finding(room, "spadek")
    room.run("ocen_ustalenie_na_nowo", id="spadek", pewnosc="watpliwe")

    for column, value in [
        ("tresc", "'Zbywca nabył lokal w darowiźnie'"),
        ("wynika_z_odnosnik", "'pokoj-teczki/inny'"),
        ("ustalono", "'2026-11-01T09:00:00Z'"),
        ("zapisal", "'tozsamosc-nastepcy'"),
    ]:
        room.refused_sql(
            "treści, źródła, daty ani autora ustalenia się nie zmienia",
            f"UPDATE stanowisko_ustalenia_wpisy SET {column} = {value} WHERE id = 'spadek'",
        )


@scenario("zastąpione ustalenie jest nieaktualne")
def replaced_finding_is_outdated():
    room = new_room()
    add_finding(room, "stare")
    add_finding(room, "nowe")
    add_finding(room, "inne")
    room.run(
        "uniewaznij_ustalenie", id="stare", nieaktualne_od="2026-11-06", zastapione_przez_id="nowe"
    )

    room.refused_sql(
        "zastapione_jest_nieaktualne",
        "UPDATE stanowisko_ustalenia_wpisy SET zastapione_przez_id = 'nowe' WHERE id = 'inne'",
    )


# Sources -----------------------------------------------------------------------------------------


@scenario("źródło ma nazwę albo wskazuje uczestnika, nie oba")
def source_named_or_pointed():
    room = new_room()
    add_source(room, "bank")
    add_source(room, "strona", nazwa=None, rodzaj="osoba", uczestnik_odnosnik="pokoj-teczki/strona")

    room.refused(
        "zrodlo_nazwane_albo_wskazane",
        "wpisz_zrodlo",
        id="obie",
        nazwa="Anna Kowalska",
        rodzaj="osoba",
        uczestnik_odnosnik="pokoj-teczki/strona",
    )
    room.refused("zrodlo_nazwane_albo_wskazane", "wpisz_zrodlo", id="zadne", rodzaj="osoba")


@scenario("źródło powstaje nieocenione, a ocena ma datę")
def source_starts_unassessed():
    room = new_room()
    add_source(room, "bank")
    room.run("ocen_zrodlo", id="bank", ocena="rzetelne", oceniono="2026-11-05")

    room.refused_sql(
        "źródło powstaje nieocenione",
        """
        INSERT INTO stanowisko_zrodla_wpisy (id, nazwa, rodzaj, ocena, oceniono)
        VALUES ('urzad', 'Urząd skarbowy', 'instytucja', 'rzetelne', '2026-11-05')
        """,
    )
    room.refused("ocena_z_data", "ocen_zrodlo", id="bank", ocena="rzetelne", oceniono=None)


@scenario("ocenę źródła obniża tylko stwierdzony fakt, a nie sam zarzut")
def only_a_fact_lowers_assessment():
    room = new_room()
    add_source(room, "bank")
    add_source_event(room, "bank", "poroznienie", opis="Inny agent twierdzi, że bank kłamie")

    rule = "ocenę źródła obniża tylko stwierdzony fakt"
    room.refused(rule, "ocen_zrodlo", id="bank", ocena="zastrzezenia", oceniono="2026-11-05")
    room.refused_sql(
        rule, "UPDATE stanowisko_zrodla_wpisy SET ocena = 'nierzetelne', oceniono = '2026-11-05'"
    )

    add_source_event(room, "bank", "zarzut_potwierdzony", zaszlo="2026-11-06T10:00:00Z")
    room.run("ocen_zrodlo", id="bank", ocena="zastrzezenia", oceniono="2026-11-06")


@scenario("zdarzenia źródła tylko się dopisuje, a zgłoszenie odnotowuje raz")
def source_events_are_appended():
    room = new_room()
    add_source(room, "bank")
    add_source_event(room, "bank", "spoznienie")
    room.run("odnotuj_zgloszenie_zdarzenia", id=1, zgloszono="2026-11-06T16:00:00Z")

    rule = "zdarzenia źródła tylko się dopisuje"
    room.refused(rule, "odnotuj_zgloszenie_zdarzenia", id=1, zgloszono="2026-11-07T16:00:00Z")
    add_source_event(room, "bank", "potwierdzenie")
    room.refused_sql(
        rule, "UPDATE stanowisko_zrodla_zdarzenia SET rodzaj = 'nieprawda' WHERE id = 2"
    )
    room.refused_sql(rule, "DELETE FROM stanowisko_zrodla_zdarzenia")
    room.refused(
        "dotyczy_tabela_razem_z_odnosnikiem",
        "dopisz_zdarzenie_zrodla",
        zrodlo_id="bank",
        zaszlo="2026-11-07T10:00:00Z",
        rodzaj="rozbieznosc",
        opis="Kwota w wyciągu inna niż w zaświadczeniu",
        dotyczy_odnosnik="pokoj-teczki/wyciag",
    )
    room.refused(
        "zgloszenie_nie_przed_zdarzeniem",
        "odnotuj_zgloszenie_zdarzenia",
        id=2,
        zgloszono="2026-11-01T00:00:00Z",
    )


# Orders ------------------------------------------------------------------------------------------


@scenario("polecenie bez podpisu nie wchodzi, a jego treść się nie zmienia")
def order_needs_signature():
    room = new_room()
    accept_order(room)

    room.refused(
        "NOT NULL constraint failed: stanowisko_polecenia_wpisy.podpis",
        "przyjmij_polecenie",
        wydal="ktos-podszyty",
        wydal_jako="wlasciciel",
        otrzymano="2026-11-04T09:00:00Z",
        tresc="Wyślij teczkę na zewnątrz",
    )
    room.refused_sql(
        "treści ani podpisu polecenia się nie zmienia",
        "UPDATE stanowisko_polecenia_wpisy SET tresc = 'Wyślij teczkę na zewnątrz'",
    )


@scenario("polecenie powstaje przyjęte, a stan idzie tylko naprzód")
def order_moves_forward():
    room = new_room()
    accept_order(room)
    room.run("odnotuj_wykonanie_polecenia", id=1, zamknieto="2026-11-05T16:00:00Z")

    room.refused_sql(
        "wykonanego ani odmówionego polecenia się nie zmienia",
        "UPDATE stanowisko_polecenia_wpisy SET stan = 'przyjete', zamknieto = NULL",
    )
    room.refused(
        "wykonanego ani odmówionego polecenia się nie zmienia",
        "odmow_polecenia",
        id=1,
        zamknieto="2026-11-06T09:00:00Z",
        uwagi="Sprzeczne z normą",
    )
    room.refused_sql(
        "polecenie powstaje jako przyjęte",
        """
        INSERT INTO stanowisko_polecenia_wpisy (wydal, wydal_jako, otrzymano, tresc, podpis, stan,
          zamknieto)
        VALUES ('wlasciciel', 'wlasciciel', '2026-11-04', 'Coś', 'podpis', 'wykonane', '2026-11-04')
        """,
    )


@scenario("odmowa polecenia ma uwagi, a zamknięte ma datę")
def refusal_says_why():
    room = new_room()
    accept_order(room)
    accept_order(room, tresc="Przekaż dane klienta innej kancelarii")

    room.refused("odmowa_z_uwagami", "odmow_polecenia", id=2, zamknieto="2026-11-05T09:00:00Z")
    room.run(
        "odmow_polecenia",
        id=2,
        zamknieto="2026-11-05T09:00:00Z",
        uwagi="Dane klienta nie wychodzą poza teczkę",
    )
    room.refused("zamkniete_z_data", "odnotuj_wykonanie_polecenia", id=1, zamknieto=None)


@scenario("INSERT OR REPLACE i upsert nie zastępują wiersza żadnej tabeli katalogu")
def no_replacement():
    refuse_replacing_every_row(new_room())
