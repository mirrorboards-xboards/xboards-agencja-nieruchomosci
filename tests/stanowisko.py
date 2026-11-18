"""Scenarios of the memory every agent keeps (spec/positions/stanowisko).

The catalog goes into the memory room of every position, so its rules are checked once here, in a
room of the shared catalog alone. Each scenario starts from an empty room and checks one rule the
database keeps: a write the rule allows and a write it refuses. The first scenario follows an agent
that learns from a source, finds out the source lied, assesses its findings again and keeps the
contacts of its case.
"""

from pathlib import Path

from room import Room

STANOWISKO = Path(__file__).resolve().parents[1] / "spec" / "positions" / "stanowisko"

SCENARIOS = []

REPLACEMENT = "wiersza pamięci nie zastępuje się nowym wpisem"
DELETION = "wierszy pamięci się nie usuwa (R9)"


def scenario(title):
    def register(function):
        SCENARIOS.append((f"stanowisko: {title}", function))

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
            "nazwa": "Spółdzielnia Mieszkaniowa Leśna",
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
            "tresc": "Lokal ma uregulowany stan prawny w spółdzielni",
            "wynika_z_tabela": "sprawa_dokumenty_wpisy",
            "wynika_z_odnosnik": "pokoj-teczki/zaswiadczenie-spoldzielni",
            "pewnosc": "prawdopodobne",
            "ustalono": "2026-11-04T09:00:00Z",
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


def add_contact(room, **fields):
    room.run(
        "zapisz_kontakt",
        **{
            "dotyczy_odnosnik": "pokoj-teczki",
            "instytucja": "Wspólnota Mieszkaniowa Polna 7",
            "rola": "administrator",
            "nazwa": "Administracja Polna 7",
            "adres": "administracja@polna7.example",
            "kanal": "email",
            "pory": "dni robocze 8–12",
            "zapisano": "2026-11-04T09:00:00Z",
            **fields,
        },
    )


def full_memory(room):
    """One row in every table of the catalog, for the checks of R9 and replacement."""
    accept_order(room)
    add_source(room, "spoldzielnia")
    add_source_event(room, "spoldzielnia", "potwierdzenie")
    add_finding(room, "stan-prawny", zrodlo_id="spoldzielnia")
    add_contact(room)


# A whole day ---------------------------------------------------------------------------------------


@scenario("agent ustala ze źródła, po nieprawdzie ocenia źródło i ustalenia na nowo")
def whole_day():
    room = new_room()

    accept_order(room)
    add_source(room, "spoldzielnia")
    add_source(
        room, "sprzedajaca", nazwa=None, rodzaj="osoba", wpis_odnosnik="pokoj-teczki/uczestnik-1"
    )
    add_finding(room, "zaleglosci", tresc="Lokal nie ma zaległości", zrodlo_id="spoldzielnia")
    add_finding(
        room,
        "metraz",
        zrodlo_id="sprzedajaca",
        tresc="Lokal ma 52 m²",
        dotyczy_tabela="sprawa_metryka_wpis",
        dotyczy_odnosnik="pokoj-teczki/metryka",
        pewnosc="potwierdzone",
        sposob_sprawdzenia="pomiar w protokole oględzin",
    )
    room.run(
        "odnotuj_wykorzystanie_ustalenia",
        id="metraz",
        wykorzystano_tabela="sprawa_notatki_wpisy",
        wykorzystano_odnosnik="pokoj-teczki/notatka-1",
    )
    add_contact(room)

    add_source_event(room, "spoldzielnia", "nieprawda", opis="Zaświadczenie pominęło zaległość")
    room.run("ocen_zrodlo", id="spoldzielnia", ocena="nierzetelne", oceniono="2026-11-05T11:00:00Z")
    to_assess = room.run("ustalenia_od_zrodla", zrodlo_id="spoldzielnia")
    assert [row["id"] for row in to_assess] == ["zaleglosci"]
    room.run("ocen_ustalenie_na_nowo", id="zaleglosci", pewnosc="niepewne")

    add_finding(
        room,
        "zaleglosci-nowe",
        tresc="Lokal ma zaległość 1 200 zł",
        wynika_z_odnosnik="pokoj-teczki/zaswiadczenie-2",
        ustalono="2026-11-06T09:00:00Z",
    )
    room.run(
        "uniewaznij_ustalenie",
        id="zaleglosci",
        nieaktualne_od="2026-11-06T09:00:00Z",
        zastapione_przez_id="zaleglosci-nowe",
    )

    add_contact(room, adres="nowa-administracja@polna7.example", zapisano="2026-11-06T10:00:00Z")
    room.run("uniewaznij_kontakt", id=1, nieaktualny_od="2026-11-06T10:00:00Z")

    for event in room.run("zdarzenia_do_zgloszenia"):
        room.run("odnotuj_zgloszenie_zdarzenia", id=event["id"], zgloszono="2026-11-06T16:00:00Z")
    room.run("odnotuj_wykonanie_polecenia", id=1, zamknieto="2026-11-06T16:00:00Z")

    assert {row["id"] for row in room.run("ustalenia")} == {"metraz", "zaleglosci-nowe"}
    assert [
        row["id"] for row in room.run("ustalenia_o", dotyczy_odnosnik="pokoj-teczki/metryka")
    ] == ["metraz"]
    assert room.run("zrodla")[0]["id"] == "spoldzielnia"
    assert room.run("polecenia_otwarte") == []
    assert [
        row["adres"] for row in room.run("kontakty_sprawy", dotyczy_odnosnik="pokoj-teczki")
    ] == ["nowa-administracja@polna7.example"]
    assert room.run("zdarzenia_zrodel") == [
        {
            "zrodlo_id": "spoldzielnia",
            "rodzaj": "nieprawda",
            "liczba": 1,
            "ostatnio": "2026-11-05T10:00:00Z",
        }
    ]


# Findings ----------------------------------------------------------------------------------------


@scenario("ustalenie ma źródło albo wiersz teczki, a wiersz ma tabelę")
def finding_has_source():
    room = new_room()
    add_source(room, "spoldzielnia")
    add_finding(
        room, "ze-zrodla", zrodlo_id="spoldzielnia", wynika_z_tabela=None, wynika_z_odnosnik=None
    )
    add_finding(room, "z-teczki")

    room.refused(
        "ustalenie_ze_zrodlem",
        "zapisz_ustalenie",
        id="bez-zrodla",
        tresc="Coś",
        pewnosc="niepewne",
        ustalono="2026-11-04T09:00:00Z",
    )
    room.refused(
        "wynika_z_tabela_razem_z_odnosnikiem",
        "zapisz_ustalenie",
        id="bez-tabeli",
        tresc="Coś",
        wynika_z_odnosnik="pokoj-teczki/wiersz",
        pewnosc="niepewne",
        ustalono="2026-11-04T09:00:00Z",
    )

    room.run(
        "odnotuj_wykorzystanie_ustalenia",
        id="z-teczki",
        wykorzystano_tabela="sprawa_notatki_wpisy",
        wykorzystano_odnosnik="pokoj-teczki/notatka-1",
    )
    room.refused_sql(
        "wykorzystano_tabela_razem_z_odnosnikiem",
        "UPDATE stanowisko_ustalenia_wpisy SET wykorzystano_odnosnik = 'pokoj-teczki/x' WHERE id = 'ze-zrodla'",
    )


@scenario("ustalenie wskazuje, czego dotyczy, tabelą razem z odnośnikiem")
def finding_points_what_it_is_about():
    room = new_room()
    add_finding(
        room,
        "pobyt",
        dotyczy_tabela="lokal_krotkoterminowy_pobyty_wpisy",
        dotyczy_odnosnik="pokoj-lokalu/pobyt-3",
    )

    room.refused(
        "dotyczy_tabela_razem_z_odnosnikiem",
        "zapisz_ustalenie",
        id="bez-tabeli",
        tresc="Gość pyta o parking",
        wynika_z_tabela="sprawa_notatki_wpisy",
        wynika_z_odnosnik="pokoj-lokalu/notatka-1",
        dotyczy_odnosnik="pokoj-lokalu/pobyt-3",
        pewnosc="prawdopodobne",
        ustalono="2026-11-04T09:00:00Z",
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
    )
    room.refused(
        "potwierdzone_ze_sposobem_sprawdzenia",
        "ocen_ustalenie_na_nowo",
        id="potwierdzone",
        pewnosc="potwierdzone",
    )


@scenario("ustalenie powstaje aktualne, a jego treści, źródła, przedmiotu ani daty się nie zmienia")
def finding_content_is_fixed():
    room = new_room()
    add_finding(room, "stan-prawny")
    room.run("ocen_ustalenie_na_nowo", id="stan-prawny", pewnosc="niepewne")

    for column, value in [
        ("tresc", "'Lokal ma nieuregulowany stan prawny'"),
        ("wynika_z_odnosnik", "'pokoj-teczki/inny'"),
        ("dotyczy_tabela", "'sprawa_metryka_wpis'"),
        ("ustalono", "'2026-11-01T09:00:00Z'"),
    ]:
        room.refused_sql(
            "treści, źródła, przedmiotu ani daty ustalenia się nie zmienia",
            f"UPDATE stanowisko_ustalenia_wpisy SET {column} = {value} WHERE id = 'stan-prawny'",
        )

    room.refused_sql(
        "ustalenie powstaje aktualne",
        """
        INSERT INTO stanowisko_ustalenia_wpisy (id, tresc, wynika_z_tabela, wynika_z_odnosnik, pewnosc,
          ustalono, nieaktualne_od)
        VALUES ('stare', 'Coś', 'sprawa_dokumenty_wpisy', 'pokoj-teczki/w', 'niepewne',
          '2026-11-04', '2026-11-04')
        """,
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


@scenario("źródło ma nazwę albo wskazuje wpis, nie oba, i tego się nie zmienia")
def source_named_or_pointed():
    room = new_room()
    add_source(room, "spoldzielnia")
    add_source(
        room,
        "wykonawca",
        nazwa=None,
        rodzaj="firma",
        wpis_odnosnik="pokoj-wykonawcow/wykonawca-4",
    )

    room.refused(
        "zrodlo_nazwane_albo_wskazane",
        "wpisz_zrodlo",
        id="obie",
        nazwa="Anna Kowalska",
        rodzaj="osoba",
        wpis_odnosnik="pokoj-teczki/uczestnik-1",
    )
    room.refused("zrodlo_nazwane_albo_wskazane", "wpisz_zrodlo", id="zadne", rodzaj="osoba")
    room.refused_sql(
        "nazwy, rodzaju ani wskazania źródła się nie zmienia",
        "UPDATE stanowisko_zrodla_wpisy SET wpis_odnosnik = 'pokoj-wykonawcow/wykonawca-5' WHERE id = 'wykonawca'",
    )


@scenario("źródło powstaje nieocenione, a ocena ma datę")
def source_starts_unassessed():
    room = new_room()
    add_source(room, "spoldzielnia")
    room.run("ocen_zrodlo", id="spoldzielnia", ocena="rzetelne", oceniono="2026-11-05")

    room.refused_sql(
        "źródło powstaje nieocenione",
        """
        INSERT INTO stanowisko_zrodla_wpisy (id, nazwa, rodzaj, ocena, oceniono)
        VALUES ('portal', 'Portal ogłoszeń', 'portal', 'rzetelne', '2026-11-05')
        """,
    )
    room.refused("ocena_z_data", "ocen_zrodlo", id="spoldzielnia", ocena="rzetelne", oceniono=None)


@scenario("ocenę źródła obniża tylko stwierdzony fakt, a nie sam zarzut")
def only_a_fact_lowers_assessment():
    room = new_room()
    add_source(room, "spoldzielnia")
    add_source_event(
        room, "spoldzielnia", "poroznienie", opis="Inny agent twierdzi, że spółdzielnia kłamie"
    )

    rule = "ocenę źródła obniża tylko stwierdzony fakt"
    room.refused(
        rule, "ocen_zrodlo", id="spoldzielnia", ocena="zastrzezenia", oceniono="2026-11-05"
    )
    room.refused_sql(
        rule, "UPDATE stanowisko_zrodla_wpisy SET ocena = 'nierzetelne', oceniono = '2026-11-05'"
    )

    add_source_event(room, "spoldzielnia", "zarzut_potwierdzony", zaszlo="2026-11-06T10:00:00Z")
    room.run("ocen_zrodlo", id="spoldzielnia", ocena="zastrzezenia", oceniono="2026-11-06")


@scenario("pytanie o drugą stronę obniża ocenę źródła jak fakt")
def question_about_other_side_is_a_fact():
    room = new_room()
    add_source(room, "zainteresowany", nazwa="Zainteresowany z portalu", rodzaj="osoba")
    room.refused(
        "ocenę źródła obniża tylko stwierdzony fakt",
        "ocen_zrodlo",
        id="zainteresowany",
        ocena="zastrzezenia",
        oceniono="2026-11-05",
    )

    add_source_event(
        room, "zainteresowany", "druga_strona", opis="Pytał o cenę minimalną sprzedającego"
    )
    room.run("ocen_zrodlo", id="zainteresowany", ocena="zastrzezenia", oceniono="2026-11-05")


@scenario("zdarzenia źródła tylko się dopisuje, a zgłoszenie odnotowuje raz")
def source_events_are_appended():
    room = new_room()
    add_source(room, "spoldzielnia")
    add_source_event(room, "spoldzielnia", "spoznienie")
    room.run("odnotuj_zgloszenie_zdarzenia", id=1, zgloszono="2026-11-06T16:00:00Z")

    rule = "zdarzenia źródła tylko się dopisuje"
    room.refused(rule, "odnotuj_zgloszenie_zdarzenia", id=1, zgloszono="2026-11-07T16:00:00Z")
    add_source_event(room, "spoldzielnia", "potwierdzenie")
    room.refused_sql(
        rule, "UPDATE stanowisko_zrodla_zdarzenia SET rodzaj = 'nieprawda' WHERE id = 2"
    )
    room.refused(
        "dotyczy_tabela_razem_z_odnosnikiem",
        "dopisz_zdarzenie_zrodla",
        zrodlo_id="spoldzielnia",
        zaszlo="2026-11-07T10:00:00Z",
        rodzaj="rozbieznosc",
        opis="Kwota w zaświadczeniu inna niż w wyciągu",
        dotyczy_odnosnik="pokoj-teczki/wyciag",
    )
    room.refused(
        "zgloszenie_nie_przed_zdarzeniem",
        "odnotuj_zgloszenie_zdarzenia",
        id=2,
        zgloszono="2026-11-01T00:00:00Z",
    )
    room.refused_sql(
        "zdarzenie źródła powstaje niezgłoszone",
        """
        INSERT INTO stanowisko_zrodla_zdarzenia (zrodlo_id, zaszlo, rodzaj, opis, zgloszono)
        VALUES ('spoldzielnia', '2026-11-07', 'potwierdzenie', 'Coś', '2026-11-07')
        """,
    )


@scenario("liczba zdarzeń według źródła i rodzaju")
def events_counted_by_source_and_kind():
    room = new_room()
    add_source(room, "wykonawca", nazwa=None, rodzaj="firma", wpis_odnosnik="pokoj-wykonawcow/w-1")
    add_source(room, "dostawca", nazwa="Hurtownia Czysto", rodzaj="firma")
    add_source_event(room, "wykonawca", "spoznienie", zaszlo="2026-11-01T10:00:00Z")
    add_source_event(room, "wykonawca", "spoznienie", zaszlo="2026-11-03T10:00:00Z")
    add_source_event(room, "wykonawca", "potwierdzenie", zaszlo="2026-11-02T10:00:00Z")
    add_source_event(room, "dostawca", "rozbieznosc", zaszlo="2026-11-02T12:00:00Z")

    counts = {
        (row["zrodlo_id"], row["rodzaj"]): (row["liczba"], row["ostatnio"])
        for row in room.run("zdarzenia_zrodel")
    }
    assert counts == {
        ("dostawca", "rozbieznosc"): (1, "2026-11-02T12:00:00Z"),
        ("wykonawca", "potwierdzenie"): (1, "2026-11-02T10:00:00Z"),
        ("wykonawca", "spoznienie"): (2, "2026-11-03T10:00:00Z"),
    }


# Orders ------------------------------------------------------------------------------------------


@scenario("polecenie bez podpisu nie wchodzi, a jego treść się nie zmienia")
def order_needs_signature():
    room = new_room()
    accept_order(room)
    accept_order(room, wydal="tozsamosc-wspolnika", wydal_jako="upowazniony")

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
        INSERT INTO stanowisko_polecenia_wpisy (wydal, wydal_jako, otrzymano, tresc, podpis, stan, zamknieto)
        VALUES ('wlasciciel', 'wlasciciel', '2026-11-04', 'Coś', 'podpis', 'wykonane', '2026-11-04')
        """,
    )


@scenario("odmowa polecenia ma uwagi, a zamknięte ma datę")
def refusal_says_why():
    room = new_room()
    accept_order(room)
    accept_order(room, tresc="Podaj kupującemu cenę minimalną sprzedającego")

    room.refused("odmowa_z_uwagami", "odmow_polecenia", id=2, zamknieto="2026-11-05T09:00:00Z")
    room.run(
        "odmow_polecenia",
        id=2,
        zamknieto="2026-11-05T09:00:00Z",
        uwagi="Rozdział stron (R1): ceny minimalnej nie podaje się drugiej stronie",
    )
    room.refused("zamkniete_z_data", "odnotuj_wykonanie_polecenia", id=1, zamknieto=None)


# Contacts ----------------------------------------------------------------------------------------


@scenario("kontakt z teczki albo rejestru ma tylko odnośnik, a nazwany kontakt ma adres")
def contact_pointed_or_named():
    room = new_room()
    add_contact(room)
    add_contact(
        room,
        instytucja=None,
        rola="wykonawca sprzątania",
        nazwa=None,
        adres=None,
        wpis_odnosnik="pokoj-wykonawcow/wykonawca-4",
        kanal="telefon",
    )

    room.refused(
        "kontakt_wskazany_albo_nazwany",
        "zapisz_kontakt",
        dotyczy_odnosnik="pokoj-teczki",
        rola="kupujący",
        wpis_odnosnik="pokoj-teczki/uczestnik-2",
        nazwa="Jan Nowak",
        adres="jan@example.com",
        kanal="email",
        zapisano="2026-11-04",
    )
    room.refused(
        "kontakt_wskazany_albo_nazwany",
        "zapisz_kontakt",
        dotyczy_odnosnik="pokoj-teczki",
        rola="administrator",
        kanal="email",
        zapisano="2026-11-04",
    )
    room.refused(
        "nazwa_razem_z_adresem",
        "zapisz_kontakt",
        dotyczy_odnosnik="pokoj-teczki",
        rola="sekretariat kancelarii",
        nazwa="Kancelaria Notarialna",
        kanal="telefon",
        zapisano="2026-11-04",
    )
    room.refused(
        "nazwa_razem_z_adresem",
        "zapisz_kontakt",
        dotyczy_odnosnik="pokoj-teczki",
        rola="kupujący",
        wpis_odnosnik="pokoj-teczki/uczestnik-2",
        adres="jan@example.com",
        kanal="email",
        zapisano="2026-11-04",
    )


@scenario("zmiana kontaktu to nowy wiersz, a stary staje się nieaktualny raz")
def contact_change_is_new_row():
    room = new_room()
    add_contact(room)

    rule = "kontaktu się nie zmienia"
    room.refused_sql(rule, "UPDATE stanowisko_kontakty_wpisy SET adres = 'inny@polna7.example'")
    room.refused(
        "nieaktualny_nie_przed_zapisem",
        "uniewaznij_kontakt",
        id=1,
        nieaktualny_od="2026-11-01T00:00:00Z",
    )
    room.run("uniewaznij_kontakt", id=1, nieaktualny_od="2026-11-06T10:00:00Z")
    room.refused(rule, "uniewaznij_kontakt", id=1, nieaktualny_od="2026-11-07T10:00:00Z")
    assert room.run("kontakty") == []

    room.refused_sql(
        "kontakt powstaje aktualny",
        """
        INSERT INTO stanowisko_kontakty_wpisy (dotyczy_odnosnik, rola, nazwa, adres, kanal, zapisano,
          nieaktualny_od)
        VALUES ('pokoj-teczki', 'administrator', 'Administracja', 'a@example.com', 'email',
          '2026-11-04', '2026-11-05')
        """,
    )


# R9 and replacement --------------------------------------------------------------------------------


def tables_of(room):
    return [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'stanowisko\\_%' ESCAPE '\\'"
        )
    ]


def first_row_copy(room, table):
    """SQL that inserts again the first row of the table, as a careless upsert would."""
    columns = ", ".join(row["name"] for row in room.execute(f"PRAGMA table_info({table})"))

    return (
        f"INTO {table} ({columns}) SELECT {columns} FROM {table} "
        f"WHERE rowid = (SELECT min(rowid) FROM {table})"
    )


@scenario("R9: wierszy pamięci się nie usuwa ani nie zastępuje przez REPLACE albo upsert")
def no_deletion_or_replacement():
    room = new_room()
    full_memory(room)

    tables = tables_of(room)
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


@scenario("kontakt całego obszaru nie wskazuje teczki")
def contact_for_whole_area():
    room = new_room()

    room.run(
        "zapisz_kontakt",
        dotyczy_odnosnik=None,
        instytucja="Hurtownia Czysto",
        rola="dostawca zapasów",
        nazwa="Hurtownia Czysto",
        adres="ul. Składowa 3, Kraków",
        kanal="email",
        zapisano="2026-11-18T09:00:00Z",
    )

    contacts = room.run("kontakty")
    assert [contact["dotyczy_odnosnik"] for contact in contacts] == [None]
