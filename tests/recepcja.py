"""Scenarios of the Reception's memory room (spec/positions/biuro/recepcja).

The room has the Reception's own kinds and three kinds of the agent memory (findings, sources and
orders); the rules of those are checked in stanowisko.py. Each scenario starts from an empty room and
checks one rule the database keeps: a write the rule allows and a write it refuses. The first
scenario follows a day of the Reception: a letter assigned to a case by its mark, a portal question
assigned to an offer by the offer catalog, a new request sent on a card and resolved, a suspicious
sender and the daily report.
"""

from pathlib import Path

from stanowisko import accept_order, add_source, add_source_event, first_row_copy
from room import Room

RECEPCJA = Path(__file__).resolve().parents[1] / "spec" / "positions" / "biuro" / "recepcja"

SCENARIOS = []

REPLACEMENT = "wiersza pamięci nie zastępuje się nowym wpisem"
DELETION = "wierszy pamięci się nie usuwa (R9)"


def scenario(title):
    def register(function):
        SCENARIOS.append((f"recepcja: {title}", function))

        return function

    return register


def new_room():
    return Room(RECEPCJA)


# Building blocks ---------------------------------------------------------------------------------


def add_case(room, **fields):
    room.run(
        "wpisz_sprawe",
        **{
            "teczka_odnosnik": "pokoj-teczki-1",
            "rodzaj": "oferta",
            "oznaczenie": "Oferta 2026/031, ul. Polna 7 m. 12",
            "agent": "tozsamosc-opiekuna-oferty",
            "otwarto": "2026-11-02T09:00:00Z",
            "przekazal": "tozsamosc-kierownika-posrednictwa",
            "przekazano": "2026-11-02T09:05:00Z",
            **fields,
        },
    )


def add_mark(room, sprawa_id=1, **fields):
    room.run(
        "dodaj_znak_sprawy",
        **{
            "sprawa_id": sprawa_id,
            "rodzaj": "adres_nieruchomosci",
            "wartosc": "ul. Polna 7 m. 12, Wrocław",
            "przekazal": "tozsamosc-kierownika-posrednictwa",
            "dodano": "2026-11-02T09:05:00Z",
            **fields,
        },
    )


def log_message(room, **fields):
    room.run(
        "odnotuj_wplyw",
        **{
            "wplynelo": "2026-11-04T09:12",
            "kanal": "email",
            "nadawca": "administracja@polna7.example",
            "rodzaj_pisma": "pismo wspólnoty",
            "wiadomosc_odnosnik": "skrzynka/1",
            "oddano": "nigdzie",
            "kanal_nie_pasuje": 0,
            **fields,
        },
    )


def log_message_by_mark(room, **fields):
    log_message(
        room,
        **{
            "sprawa_id": 1,
            "znak_id": 1,
            "oddano": "agent_sprawy",
            "oddano_komu": "tozsamosc-opiekuna-oferty",
            "oddano_o": "2026-11-04T09:20",
            **fields,
        },
    )


def log_message_to_card(room, **fields):
    log_message(
        room,
        **{
            "kanal": "formularz",
            "nadawca": "formularz-strony",
            "rodzaj_pisma": "nowe zgłoszenie",
            "oddano": "karta",
            "oddano_o": "2026-11-04T09:30",
            **fields,
        },
    )


def send_card(room, wplyw_id, **fields):
    room.run(
        "wyslij_karte",
        **{
            "rodzaj": "nowe_zgloszenie",
            "wplyw_id": wplyw_id,
            "wyslano": "2026-11-04T09:30:00Z",
            "rodzaj_sprawy": "sprzedaz",
            "rodzaj_nieruchomosci": "mieszkanie",
            "miejscowosc": "Wrocław",
            "kanal": "telefon",
            "zglaszajacy": "Anna Nowak",
            "kontakt": "+48 600 000 000",
            "wiadomosc_odnosnik": "skrzynka/2",
            **fields,
        },
    )


def full_memory(room):
    """One row in every table of the Reception's own kinds, for the checks of R9 and replacement."""
    add_case(room)
    add_mark(room)
    log_message_by_mark(room)
    log_message_to_card(room, wiadomosc_odnosnik="skrzynka/2")
    send_card(room, wplyw_id=2)


# A whole day ---------------------------------------------------------------------------------------


@scenario("Recepcja przypisuje wiadomości po znakach, wysyła kartę i składa meldunek")
def whole_day():
    room = new_room()
    accept_order(room, wydal_jako="upowazniony", tresc="Wstrzymaj rozdział ze skrzynki portalu")

    add_case(room)
    add_mark(room)
    log_message_by_mark(room)
    room.run(
        "odnotuj_wplyw",
        wplynelo="2026-11-04T10:00",
        kanal="portal",
        nadawca="konto-portalu-123",
        rodzaj_pisma="zapytanie o ofertę",
        wiadomosc_odnosnik="skrzynka/portal-1",
        sprawa_id=1,
        katalog_odnosnik="pokoj-katalogu/oferta-031",
        oddano="agent_sprawy",
        oddano_komu="tozsamosc-opiekuna-oferty",
        oddano_o="2026-11-04T11:30",
        kanal_nie_pasuje=0,
    )

    log_message_to_card(room, wiadomosc_odnosnik="skrzynka/2")
    send_card(room, wplyw_id=3)
    room.run(
        "odnotuj_rozstrzygniecie_karty",
        id=1,
        rozstrzygnieto="2026-11-04T12:00:00Z",
        rozstrzygniecie="przydzielona",
        dzial="posrednictwo",
    )

    log_message(
        room,
        wplynelo="2026-11-04T13:00",
        nadawca="kancelaria@falszywa.example",
        rodzaj_pisma="pismo notariusza",
        wiadomosc_odnosnik="skrzynka/3",
        kanal_nie_pasuje=1,
    )
    add_source(room, "falszywa-kancelaria", nazwa="kancelaria@falszywa.example", rodzaj="firma")
    add_source_event(
        room,
        "falszywa-kancelaria",
        "podszywanie",
        zaszlo="2026-11-04T13:00:00Z",
        dotyczy_tabela="recepcja_wplywy_dziennik",
        dotyczy_odnosnik="pokoj-recepcji/4",
    )

    matching = room.run(
        "sprawy_po_znaku", rodzaj="adres_nieruchomosci", wartosc="ul. Polna 7 m. 12, Wrocław"
    )
    assert [row["teczka_odnosnik"] for row in matching] == ["pokoj-teczki-1"]
    assert room.run("karty_czekajace") == []
    assert room.run("meldunek_dzienny", dzien="2026-11-04") == [
        {
            "wplynelo": 4,
            "oddano_agentom": 2,
            "zapytania_po_godzinie": 1,
            "wyslano_kart": 1,
            "karty_czekaja_od_wczesniej": 0,
            "niejednoznaczne": 0,
        }
    ]
    assert room.run("wplywy_wedlug_kanalow", dzien="2026-11-04") == [
        {"kanal": "email", "liczba": 2},
        {"kanal": "formularz", "liczba": 1},
        {"kanal": "portal", "liczba": 1},
    ]
    assert len(room.run("wplywy_sprawy", sprawa_id=1)) == 2
    assert room.run("zdarzenia_zrodel")[0]["rodzaj"] == "podszywanie"


# Cases and marks -----------------------------------------------------------------------------------


@scenario("sprawa powstaje otwarta, a zmienia ją tylko nowe przekazanie kierownika działu")
def case_changes_only_by_handover():
    room = new_room()
    add_case(room)

    room.refused_sql(
        "sprawa powstaje w spisie otwarta",
        """
        INSERT INTO recepcja_sprawy_wpisy (teczka_odnosnik, rodzaj, oznaczenie, agent, stan, otwarto,
          zamknieto, przekazal, przekazano)
        VALUES ('pokoj-teczki-2', 'najem', 'Najem 2026/4', 'a', 'zamknieta', '2026-11-01',
          '2026-11-02', 'k', '2026-11-02')
        """,
    )

    rule = "sprawę w spisie zmienia tylko nowe przekazanie kierownika działu"
    room.refused(
        rule,
        "zmien_sprawe",
        id=1,
        oznaczenie="Oferta 2026/031",
        agent="tozsamosc-innego-agenta",
        przekazal="tozsamosc-kierownika-posrednictwa",
        przekazano="2026-11-02T09:05:00Z",
    )
    room.run(
        "zmien_sprawe",
        id=1,
        oznaczenie="Oferta 2026/031",
        agent="tozsamosc-nastepcy",
        przekazal="tozsamosc-kierownika-posrednictwa",
        przekazano="2026-11-06T09:00:00Z",
    )
    assert room.run("spis_spraw")[0]["agent"] == "tozsamosc-nastepcy"


@scenario("zamknięta sprawa ma datę zamknięcia i zostaje w spisie bez zmian")
def closed_case_stays():
    room = new_room()
    add_case(room)

    room.refused(
        "zamknieta_z_data_zamkniecia",
        "zamknij_sprawe",
        id=1,
        przekazal="k",
        przekazano="2026-12-01T09:00:00Z",
    )
    room.refused(
        "zamkniecie_nie_przed_otwarciem",
        "zamknij_sprawe",
        id=1,
        zamknieto="2026-11-01T00:00:00Z",
        przekazal="k",
        przekazano="2026-12-01T09:00:00Z",
    )
    room.run(
        "zamknij_sprawe",
        id=1,
        zamknieto="2026-12-01T00:00:00Z",
        przekazal="k",
        przekazano="2026-12-01T09:00:00Z",
    )

    room.refused(
        "zamkniętej sprawy wcale",
        "zmien_sprawe",
        id=1,
        oznaczenie="Oferta 2026/031",
        agent="a",
        przekazal="k",
        przekazano="2027-01-01T09:00:00Z",
    )
    room.refused_sql(
        "teczki, rodzaju ani daty otwarcia sprawy się nie zmienia",
        "UPDATE recepcja_sprawy_wpisy SET stan = 'otwarta', zamknieto = NULL, przekazano = '2027-01-02'",
    )


@scenario("kancelaria na XNotariuszu jest odnotowana tylko przy transakcji")
def notary_only_for_transaction():
    room = new_room()
    add_case(
        room, teczka_odnosnik="pokoj-transakcji-1", rodzaj="transakcja", kancelaria_xnotariusz=1
    )

    rule = "kancelaria_tylko_przy_transakcji"
    room.refused(rule, "wpisz_sprawe", **case_fields(teczka_odnosnik="t2", kancelaria_xnotariusz=0))
    room.refused(rule, "wpisz_sprawe", **case_fields(teczka_odnosnik="t3", rodzaj="transakcja"))


def case_fields(**fields):
    return {
        "teczka_odnosnik": "pokoj-teczki-1",
        "rodzaj": "oferta",
        "oznaczenie": "Oferta",
        "agent": "a",
        "otwarto": "2026-11-02T09:00:00Z",
        "przekazal": "k",
        "przekazano": "2026-11-02T09:05:00Z",
        **fields,
    }


@scenario("znak powstaje niewycofany, wycofuje się go raz i nie przed dodaniem")
def mark_is_withdrawn_once():
    room = new_room()
    add_case(room)
    add_mark(room)

    room.refused_sql(
        "znak powstaje niewycofany",
        """
        INSERT INTO recepcja_sprawy_znaki (sprawa_id, rodzaj, wartosc, przekazal, dodano, wycofano)
        VALUES (1, 'adres_poczty_klienta', 'nowak@example.com', 'k', '2026-11-02', '2026-11-03')
        """,
    )
    room.refused(
        "wycofanie_nie_przed_dodaniem", "wycofaj_znak_sprawy", id=1, wycofano="2026-11-01T00:00:00Z"
    )
    room.refused_sql(
        "znaku się nie zmienia",
        "UPDATE recepcja_sprawy_znaki SET wartosc = 'ul. Polna 9' WHERE id = 1",
    )

    room.run("wycofaj_znak_sprawy", id=1, wycofano="2026-11-06T09:00:00Z")
    room.refused(
        "znaku się nie zmienia", "wycofaj_znak_sprawy", id=1, wycofano="2026-11-07T09:00:00Z"
    )
    assert (
        room.run(
            "sprawy_po_znaku", rodzaj="adres_nieruchomosci", wartosc="ul. Polna 7 m. 12, Wrocław"
        )
        == []
    )
    assert len(room.run("znaki_sprawy", sprawa_id=1)) == 1


# Log of incoming messages --------------------------------------------------------------------------


@scenario("wpływ ma czas co do minuty, a sprawę tylko po znaku albo numerze z katalogu")
def message_case_only_by_mark():
    room = new_room()
    add_case(room)
    add_mark(room)
    log_message_by_mark(room)

    room.refused("wplynelo_co_do_minuty", "odnotuj_wplyw", **message_fields(wplynelo="2026-11-04"))

    rule = "sprawa_tylko_po_znaku_albo_katalogu"
    room.refused(rule, "odnotuj_wplyw", **message_fields(sprawa_id=1))
    room.refused(
        rule,
        "odnotuj_wplyw",
        **message_fields(sprawa_id=1, znak_id=1, katalog_odnosnik="pokoj-katalogu/oferta-031"),
    )


def message_fields(**fields):
    return {
        "wplynelo": "2026-11-04T09:12",
        "kanal": "email",
        "nadawca": "n",
        "rodzaj_pisma": "pismo",
        "wiadomosc_odnosnik": "skrzynka/x",
        "oddano": "nigdzie",
        "kanal_nie_pasuje": 0,
        **fields,
    }


@scenario("znak musi być niewycofanym znakiem tej sprawy, a numer z katalogu wskazuje ofertę")
def mark_of_the_same_case():
    room = new_room()
    add_case(room)
    add_case(room, teczka_odnosnik="pokoj-najmu-1", rodzaj="najem", oznaczenie="Najem 2026/4")
    add_mark(room)
    add_mark(room, sprawa_id=2, wartosc="ul. Leśna 3, Wrocław")
    room.run("wycofaj_znak_sprawy", id=2, wycofano="2026-11-03T09:00:00Z")

    rule = "wiadomość trafia do sprawy po jej niewycofanym znaku"
    room.refused(rule, "odnotuj_wplyw", **message_fields(sprawa_id=2, znak_id=1))
    room.refused(rule, "odnotuj_wplyw", **message_fields(sprawa_id=2, znak_id=2))

    room.refused(
        "po numerze z katalogu ofert wiadomość trafia do sprawy oferty",
        "odnotuj_wplyw",
        **message_fields(sprawa_id=2, katalog_odnosnik="pokoj-katalogu/oferta-031"),
    )
    room.run(
        "odnotuj_wplyw",
        **message_fields(sprawa_id=1, katalog_odnosnik="pokoj-katalogu/oferta-031"),
    )


@scenario("oddana wiadomość ma adresata i czas, a agent sprawy dostaje ją tylko ze sprawą")
def handed_over_message():
    room = new_room()
    log_message(
        room,
        rodzaj_pisma="list do pracownika",
        oddano="pracownik",
        oddano_komu="tozsamosc-asystenta",
        oddano_o="2026-11-04T09:15",
    )

    room.refused(
        "agent_sprawy_dostaje_wiadomosc_ze_sprawa",
        "odnotuj_wplyw",
        **message_fields(oddano="agent_sprawy", oddano_komu="a", oddano_o="2026-11-04T09:20"),
    )
    room.refused(
        "oddane_komus",
        "odnotuj_wplyw",
        **message_fields(oddano="pracownik", oddano_o="2026-11-04T09:20"),
    )
    room.refused(
        "oddane_komus",
        "odnotuj_wplyw",
        **message_fields(oddano="karta", oddano_komu="k", oddano_o="2026-11-04T09:20"),
    )
    room.refused("oddane_z_czasem", "odnotuj_wplyw", **message_fields(oddano="karta"))
    room.refused(
        "oddanie_nie_przed_wplywem",
        "odnotuj_wplyw",
        **message_fields(oddano="karta", oddano_o="2026-11-04T09:00"),
    )


@scenario("dziennik wpływów tylko się dopisuje")
def log_is_append_only():
    room = new_room()
    log_message(room)

    room.refused_sql(
        "dziennik wpływów tylko się dopisuje",
        "UPDATE recepcja_wplywy_dziennik SET nadawca = 'inny' WHERE id = 1",
    )
    assert len(room.run("wplywy_od_nadawcy", nadawca="administracja@polna7.example")) == 1
    assert len(room.run("dziennik_dnia", dzien="2026-11-04")) == 1


# Cards ---------------------------------------------------------------------------------------------


@scenario("karta stoi na wpływie oddanym na kartę i powstaje czekająca z danymi zgłoszenia")
def card_starts_waiting():
    room = new_room()
    log_message(room)
    log_message_to_card(room)
    send_card(room, wplyw_id=2)

    room.refused(
        "karta stoi na wpisie dziennika oddanym na kartę", "wyslij_karte", **card_fields(wplyw_id=1)
    )
    room.refused(
        "czekajace_zgloszenie_z_danymi", "wyslij_karte", **card_fields(wplyw_id=2, kontakt=None)
    )
    room.run(
        "wyslij_karte",
        **card_fields(wplyw_id=2, rodzaj="pismo_bez_sprawy", rodzaj_sprawy=None, kontakt=None),
    )
    room.refused_sql(
        "karta powstaje jako czekająca",
        """
        INSERT INTO recepcja_karty_wpisy (rodzaj, wplyw_id, wyslano, wiadomosc_odnosnik, stan,
          rozstrzygnieto, rozstrzygniecie)
        VALUES ('skarga', 2, '2026-11-04', 'skrzynka/9', 'rozstrzygnieta', '2026-11-04', 'odrzucona')
        """,
    )
    assert len(room.run("karty_czekajace")) == 2


def card_fields(**fields):
    return {
        "rodzaj": "nowe_zgloszenie",
        "wplyw_id": 1,
        "wyslano": "2026-11-04T09:30:00Z",
        "rodzaj_sprawy": "kupno",
        "zglaszajacy": "Jan Kowalski",
        "kontakt": "jan@example.com",
        "wiadomosc_odnosnik": "skrzynka/2",
        **fields,
    }


@scenario("po rozstrzygnięciu dane zgłaszającego znikają z karty, a dział ma tylko przydział")
def resolved_card_has_no_contact():
    room = new_room()
    log_message_to_card(room)
    send_card(room, wplyw_id=1)

    room.refused_sql(
        "rozstrzygnieta_bez_danych_zglaszajacego",
        """
        UPDATE recepcja_karty_wpisy
        SET stan = 'rozstrzygnieta', rozstrzygnieto = '2026-11-04T12:00:00Z',
          rozstrzygniecie = 'odrzucona'
        WHERE id = 1
        """,
    )
    room.refused(
        "rozstrzygnieta_z_data_i_sposobem",
        "odnotuj_rozstrzygniecie_karty",
        id=1,
        rozstrzygniecie="odrzucona",
    )
    room.refused(
        "dzial_przy_przydziale",
        "odnotuj_rozstrzygniecie_karty",
        id=1,
        rozstrzygnieto="2026-11-04T12:00:00Z",
        rozstrzygniecie="przydzielona",
    )
    room.refused(
        "dzial_przy_przydziale",
        "odnotuj_rozstrzygniecie_karty",
        id=1,
        rozstrzygnieto="2026-11-04T12:00:00Z",
        rozstrzygniecie="odrzucona",
        dzial="najem",
    )
    room.refused(
        "rozstrzygniecie_nie_przed_wyslaniem",
        "odnotuj_rozstrzygniecie_karty",
        id=1,
        rozstrzygnieto="2026-11-04T08:00:00Z",
        rozstrzygniecie="odrzucona",
    )

    room.run(
        "odnotuj_rozstrzygniecie_karty",
        id=1,
        rozstrzygnieto="2026-11-04T12:00:00Z",
        rozstrzygniecie="przydzielona",
        dzial="najem",
    )
    card = room.execute("SELECT zglaszajacy, kontakt, dzial FROM recepcja_karty_wpisy")[0]
    assert card == {"zglaszajacy": None, "kontakt": None, "dzial": "najem"}


@scenario("wysłanej karty się nie zmienia, a rozstrzygniętej wcale")
def sent_card_does_not_change():
    room = new_room()
    log_message_to_card(room)
    send_card(room, wplyw_id=1)

    rule = "wysłanej karty się nie zmienia"
    room.refused_sql(rule, "UPDATE recepcja_karty_wpisy SET miejscowosc = 'Opole' WHERE id = 1")
    room.run(
        "odnotuj_rozstrzygniecie_karty",
        id=1,
        rozstrzygnieto="2026-11-04T12:00:00Z",
        rozstrzygniecie="odrzucona",
    )
    room.refused(
        rule,
        "odnotuj_rozstrzygniecie_karty",
        id=1,
        rozstrzygnieto="2026-11-05T12:00:00Z",
        rozstrzygniecie="przydzielona",
        dzial="najem",
    )


# R9 and replacement --------------------------------------------------------------------------------


def own_tables_of(room):
    return [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'recepcja\\_%' ESCAPE '\\'"
        )
    ]


@scenario("R9: wierszy pamięci się nie usuwa ani nie zastępuje przez REPLACE albo upsert")
def no_deletion_or_replacement():
    room = new_room()
    full_memory(room)

    tables = own_tables_of(room)
    assert len(tables) == 4, tables

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


@scenario("REPLACE nie usuwa sprawy tej samej teczki przez klucz UNIQUE")
def no_replacement_through_unique_case_file():
    room = new_room()
    full_memory(room)

    table = "recepcja_sprawy_wpisy"
    columns = [row["name"] for row in room.execute(f"PRAGMA table_info({table})")]
    selected = ", ".join("99" if name == "id" else name for name in columns)
    room.refused_sql(
        REPLACEMENT,
        f"INSERT OR REPLACE INTO {table} ({', '.join(columns)}) SELECT {selected} FROM {table}",
    )
    room.refused_sql(
        REPLACEMENT,
        # `WHERE true` keeps SQLite from reading ON CONFLICT as a join constraint.
        f"INSERT INTO {table} ({', '.join(columns)}) SELECT {selected} FROM {table} "
        f"WHERE true ON CONFLICT (teczka_odnosnik) DO UPDATE SET id = excluded.id",
    )
