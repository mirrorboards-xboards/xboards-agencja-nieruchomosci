"""Scenarios of the client file of the office (spec/registers/klienci).

Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
allows and a write it refuses. The first scenario follows a client who changes her name and phone
number and is still recognised by the old number, as the profile describes it.
"""

from pathlib import Path

from room import Room

KLIENCI = Path(__file__).resolve().parents[1] / "spec" / "registers" / "klienci"

SCENARIOS = []

REPLACEMENT = "wiersza kartoteki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE"
DELETION = "wierszy kartoteki się nie usuwa (R9)"
UNCHANGED = "wpisu klienta, umowy, jej stanu ani informacji się nie zmienia"


def scenario(title):
    def register(function):
        SCENARIOS.append((title, function))

        return function

    return register


def new_room():
    return Room(KLIENCI)


# Building blocks ----------------------------------------------------------------------------------


def add_client(room, client_id="nowak", **fields):
    room.run(
        "wpisz_klienta",
        **{
            "id": client_id,
            "rodzaj_podmiotu": "osoba_fizyczna",
            "wpisal": "kierownik-posrednictwa",
            "wpisano": "2026-03-02",
            "od": "2026-03-02",
            "imiona": "Anna",
            "nazwisko": "Nowak",
            "pesel": "80010112345",
            "ceidg": 0,
            "zrodlo_odnosnik": "teczka-oferty/dokument-umowa",
            **fields,
        },
    )


def add_company(room, client_id="lesna"):
    add_client(
        room,
        client_id,
        rodzaj_podmiotu="osoba_prawna",
        imiona=None,
        nazwisko=None,
        pesel=None,
        nazwa="Leśna Inwestycje sp. z o.o.",
        forma_prawna="spółka z ograniczoną odpowiedzialnością",
        krs="0000123456",
    )


def add_contract(room, contract_id="umowa", client_id="nowak", **fields):
    room.run(
        "wpisz_umowe_klienta",
        **{
            "id": contract_id,
            "klient_id": client_id,
            "rodzaj": "posrednictwo_sprzedazy",
            "dzial": "posrednictwo",
            "nieruchomosc_adres": "ul. Polna 7 m. 12, Wrocław",
            "numer_kw": "WR1K/00012345/6",
            "teczka_odnosnik": "teczka-oferty",
            "typ_teczki": "oferta",
            "oznaczenie": "Oferta 2026/031",
            "umowa_odnosnik": "teczka-oferty/umowa",
            "podpisal_za_biuro": "posrednik",
            "zawarto": "2026-03-02",
            "konsument": 1,
            "obie_strony": 0,
            "wpisal": "kierownik-posrednictwa",
            "wpisano": "2026-03-02",
            **fields,
        },
    )


def add_phone(room, client_id="nowak", wartosc="+48600100200", od="2026-03-02"):
    room.run(
        "wpisz_kontakt_klienta",
        klient_id=client_id,
        rodzaj="telefon",
        wartosc=wartosc,
        preferowany=1,
        od=od,
        zrodlo_odnosnik="teczka-oferty/dokument-umowa",
    )


def add_address(
    room, client_id="nowak", rodzaj="zamieszkanie", adres="ul. Polna 7 m. 12", od="2026-03-02"
):
    room.run(
        "wpisz_adres_klienta",
        klient_id=client_id,
        rodzaj=rodzaj,
        adres=adres,
        od=od,
        zrodlo_odnosnik="teczka-oferty/dokument-umowa",
    )


def end_contract(room, contract_id="umowa", stan="wykonana", od="2026-09-30"):
    room.run(
        "zmien_stan_umowy_klienta",
        umowa_id=contract_id,
        stan=stan,
        od=od,
        zrodlo_odnosnik="teczka-oferty/raport-koncowy",
        wpisal="kierownik-posrednictwa",
    )


def full_file(room):
    """A client of every kind with a row in every table of the kind."""
    add_client(room)
    add_phone(room)
    add_address(room)
    add_contract(room)
    room.run(
        "wpisz_informacje_klienta",
        klient_id="nowak",
        poinformowano="2026-03-02T10:00:00Z",
        sposob="email",
        wysylka_odnosnik="teczka-oferty/wysylka-rodo",
    )
    add_company(room)
    room.run(
        "wpisz_osobe_klienta",
        klient_id="lesna",
        funkcja="dziala_za_podmiot",
        imiona="Piotr",
        nazwisko="Leśny",
        podstawa_odnosnik="teczka-oferty/odpis-krs",
        od="2026-04-01",
        zrodlo_odnosnik="teczka-oferty/umowa",
    )


# A whole client -----------------------------------------------------------------------------------


@scenario("Kartoteka: klientka zmienia nazwisko i telefon, a stary numer nadal ją wskazuje")
def whole_client():
    room = new_room()
    add_client(room)
    add_phone(room)
    add_contract(room)

    room.run(
        "wpisz_dane_klienta",
        klient_id="nowak",
        od="2027-05-10",
        imiona="Anna",
        nazwisko="Nowak-Lis",
        pesel="80010112345",
        ceidg=0,
        zrodlo_odnosnik="teczka-najmu/akt-malzenstwa",
        zrodlo_opis="odpis aktu małżeństwa",
        wpisal="kierownik-najmu",
    )
    old_phone_id = room.run("kontakty_klienta", klient_id="nowak")[0]["id"]
    room.run("zakoncz_kontakt_klienta", id=old_phone_id, do="2027-05-10")
    add_phone(room, wartosc="+48600999888", od="2027-05-10")

    versions = room.run("dane_klienta", klient_id="nowak")
    assert [(row["nazwisko"], row["aktualne"]) for row in versions] == [
        ("Nowak-Lis", 1),
        ("Nowak", 0),
    ], versions
    assert room.run("klienci")[0]["nazwisko"] == "Nowak-Lis"
    assert room.run("klienci_po_kontakcie", wartosc="+48600100200")[0]["klient_id"] == "nowak"
    assert room.run("klienci_po_nazwisku_albo_nazwie", szukane="Nowak")[0]["klient_id"] == "nowak"

    assert room.run("koniec_ostatniej_umowy_klienta", klient_id="nowak")[0]["koniec"] is None
    end_contract(room)
    assert (
        room.run("koniec_ostatniej_umowy_klienta", klient_id="nowak")[0]["koniec"] == "2026-09-30"
    )
    assert room.run("umowy_klienta", klient_id="nowak")[0]["stan"] == "wykonana"


# Data of a person and of an entity ----------------------------------------------------------------


@scenario("Dane: osoba ma imiona i nazwisko, podmiot nazwę bez danych osoby")
def person_and_entity_data():
    room = new_room()

    room.refused(
        "dane osoby fizycznej mają imiona i nazwisko",
        "wpisz_klienta",
        **client_fields(nazwisko=None),
    )
    room.refused(
        "dane podmiotu mają nazwę i nie mają imion, nazwiska ani PESEL",
        "wpisz_klienta",
        **client_fields(
            client_id="lesna", rodzaj_podmiotu="osoba_prawna", nazwa="Leśna sp. z o.o."
        ),
    )
    room.refused("pesel_ma_11_cyfr", "wpisz_klienta", **client_fields(pesel="8001011234"))
    room.refused("krs_albo_ceidg", "wpisz_klienta", **client_fields(krs="0000123456", ceidg=1))

    add_client(room, firma="Anna Nowak Usługi", nip="5250000000", ceidg=1)
    add_company(room)
    assert len(room.run("klienci")) == 2


def client_fields(client_id="nowak", **fields):
    return {
        "id": client_id,
        "rodzaj_podmiotu": "osoba_fizyczna",
        "wpisal": "kierownik-posrednictwa",
        "wpisano": "2026-03-02",
        "od": "2026-03-02",
        "imiona": "Anna",
        "nazwisko": "Nowak",
        "pesel": "80010112345",
        "ceidg": 0,
        "zrodlo_odnosnik": "teczka-oferty/dokument-umowa",
        **fields,
    }


@scenario("Dane: PESEL ma jeden klient; kolejna umowa trafia do istniejącego wpisu")
def one_client_per_pesel():
    room = new_room()
    add_client(room)

    room.refused(
        "ten PESEL ma już inny klient kartoteki",
        "wpisz_klienta",
        **client_fields(client_id="druga-nowak"),
    )

    add_contract(room)
    add_contract(
        room,
        "druga-umowa",
        rodzaj="posrednictwo_kupna",
        typ_teczki="poszukiwanie",
        nieruchomosc_adres=None,
        numer_kw=None,
    )
    room.run(
        "wpisz_dane_klienta",
        klient_id="nowak",
        od="2026-06-01",
        imiona="Anna",
        nazwisko="Nowak",
        pesel="80010112345",
        ceidg=0,
        zrodlo_odnosnik="teczka-poszukiwania/umowa",
        wpisal="kierownik-posrednictwa",
    )
    assert len(room.run("umowy_klienta", klient_id="nowak")) == 2


# Versions of the data ---------------------------------------------------------------------------


@scenario("Wersje danych: nowa powstaje aktualna i odbiera aktualność poprzedniej")
def data_versions():
    room = new_room()
    add_client(room)

    room.refused_sql(
        "wersja danych klienta powstaje jako aktualna",
        """INSERT INTO klienci_kartoteka_dane (klient_id, od, aktualne, imiona, nazwisko,
          zrodlo_odnosnik, wpisal)
        VALUES ('nowak', '2026-06-01', 0, 'Anna', 'Nowak', 'teczka/aneks', 'kierownik')""",
    )
    room.refused(
        "nowa wersja danych klienta nie obowiązuje wcześniej niż aktualna",
        "wpisz_dane_klienta",
        klient_id="nowak",
        od="2026-01-01",
        imiona="Anna",
        nazwisko="Nowak",
        ceidg=0,
        zrodlo_odnosnik="teczka/aneks",
        wpisal="kierownik",
    )

    room.run(
        "wpisz_dane_klienta",
        klient_id="nowak",
        od="2026-06-01",
        imiona="Anna",
        nazwisko="Kowalska",
        ceidg=0,
        zrodlo_odnosnik="teczka/aneks",
        wpisal="kierownik",
    )
    current = [row for row in room.run("dane_klienta", klient_id="nowak") if row["aktualne"] == 1]
    assert [row["nazwisko"] for row in current] == ["Kowalska"], current


@scenario("Wersje danych: wersji się nie zmienia, a aktualnej nie odbiera się bez nowszej")
def data_unchanged():
    room = new_room()
    add_client(room)

    room.refused_sql(
        "danych klienta się nie zmienia",
        "UPDATE klienci_kartoteka_dane SET nazwisko = 'Lis' WHERE klient_id = 'nowak'",
    )
    room.refused_sql(
        "danych klienta się nie zmienia",
        "UPDATE klienci_kartoteka_dane SET aktualne = 0 WHERE klient_id = 'nowak'",
    )

    room.run(
        "wpisz_dane_klienta",
        klient_id="nowak",
        od="2026-06-01",
        imiona="Anna",
        nazwisko="Lis",
        ceidg=0,
        zrodlo_odnosnik="teczka/aneks",
        wpisal="kierownik",
    )
    room.refused_sql(
        "danych klienta się nie zmienia",
        "UPDATE klienci_kartoteka_dane SET aktualne = 1 WHERE nazwisko = 'Nowak'",
    )


# Addresses, contacts and persons ------------------------------------------------------------------


@scenario("Adresy: zamieszkanie ma osoba, siedzibę podmiot; bieżący adres rodzaju jest jeden")
def addresses():
    room = new_room()
    add_client(room)
    add_company(room)

    room.refused(
        "miejsce zamieszkania ma tylko osoba fizyczna, a siedzibę tylko podmiot",
        "wpisz_adres_klienta",
        klient_id="nowak",
        rodzaj="siedziba",
        adres="ul. Leśna 1, Poznań",
        od="2026-03-02",
        zrodlo_odnosnik="teczka/umowa",
    )
    add_address(room, "lesna", "siedziba", "ul. Leśna 1, Poznań")

    add_address(room)
    add_address(room, adres="ul. Długa 3", od="2026-08-01")
    addresses = room.run("adresy_klienta", klient_id="nowak")
    assert [(row["adres"], row["do"]) for row in addresses] == [
        ("ul. Długa 3", None),
        ("ul. Polna 7 m. 12", "2026-08-01"),
    ], addresses

    room.refused_sql(
        "klient ma już bieżący adres tego rodzaju",
        """INSERT INTO klienci_kartoteka_adresy (klient_id, rodzaj, adres, od, zrodlo_odnosnik)
        VALUES ('nowak', 'zamieszkanie', 'ul. Krótka 5', '2026-09-01', 'teczka/aneks')""",
    )


@scenario("Adresy, kontakty i osoby powstają bieżące i mogą się tylko raz zakończyć")
def ending_rows():
    room = new_room()
    add_client(room)

    room.refused_sql(
        "adres, kontakt i osoba powstają jako bieżące",
        """INSERT INTO klienci_kartoteka_adresy (klient_id, rodzaj, adres, od, do, zrodlo_odnosnik)
        VALUES ('nowak', 'zamieszkanie', 'ul. Polna 7', '2026-03-02', '2026-04-01', 'teczka/umowa')""",
    )
    room.refused_sql(
        "adres, kontakt i osoba powstają jako bieżące",
        """INSERT INTO klienci_kartoteka_kontakty (klient_id, rodzaj, wartosc, od, do, zrodlo_odnosnik)
        VALUES ('nowak', 'telefon', '+48600100200', '2026-03-02', '2026-04-01', 'teczka/umowa')""",
    )

    add_phone(room)
    phone_id = room.run("kontakty_klienta", klient_id="nowak")[0]["id"]
    room.refused(
        "koniec_nie_przed_poczatkiem", "zakoncz_kontakt_klienta", id=phone_id, do="2026-01-01"
    )
    room.run("zakoncz_kontakt_klienta", id=phone_id, do="2026-05-01")
    room.refused(
        "adresu, kontaktu ani osoby się nie zmienia",
        "zakoncz_kontakt_klienta",
        id=phone_id,
        do="2026-06-01",
    )
    room.refused_sql(
        "adresu, kontaktu ani osoby się nie zmienia",
        f"UPDATE klienci_kartoteka_kontakty SET wartosc = '+48111' WHERE id = {phone_id}",
    )

    add_address(room)
    address_id = room.run("adresy_klienta", klient_id="nowak")[0]["id"]
    room.run("zakoncz_adres_klienta", id=address_id, do="2026-05-01")
    room.refused_sql(
        "adresu, kontaktu ani osoby się nie zmienia",
        f"UPDATE klienci_kartoteka_adresy SET adres = 'ul. Inna 1' WHERE id = {address_id}",
    )


@scenario("Osoby: ma je tylko podmiot, a działający za podmiot ma podstawę umocowania")
def persons_of_entity():
    room = new_room()
    add_client(room)
    add_company(room)
    person = {
        "funkcja": "kontaktowa",
        "imiona": "Ewa",
        "nazwisko": "Lis",
        "kontakt": "ewa.lis@lesna.example",
        "od": "2026-04-01",
        "zrodlo_odnosnik": "teczka/umowa",
    }

    room.refused(
        "osoby działające za klienta i osoby kontaktowe ma tylko podmiot",
        "wpisz_osobe_klienta",
        klient_id="nowak",
        **person,
    )
    room.refused(
        "dzialajacy_za_podmiot_ma_podstawe",
        "wpisz_osobe_klienta",
        klient_id="lesna",
        **{**person, "funkcja": "dziala_za_podmiot"},
    )
    room.refused_sql(
        "adres, kontakt i osoba powstają jako bieżące",
        """INSERT INTO klienci_kartoteka_osoby (klient_id, funkcja, imiona, nazwisko, od, do,
          zrodlo_odnosnik)
        VALUES ('lesna', 'kontaktowa', 'Ewa', 'Lis', '2026-04-01', '2026-05-01', 'teczka/umowa')""",
    )

    room.run("wpisz_osobe_klienta", klient_id="lesna", **person)
    person_id = room.run("osoby_klienta", klient_id="lesna")[0]["id"]
    room.run("zakoncz_osobe_klienta", id=person_id, do="2026-09-01")
    room.refused_sql(
        "adresu, kontaktu ani osoby się nie zmienia",
        f"UPDATE klienci_kartoteka_osoby SET nazwisko = 'Leśna' WHERE id = {person_id}",
    )


# Contracts and their states -----------------------------------------------------------------------


@scenario("Umowy: dział, teczka i nieruchomość pasują do rodzaju umowy")
def contract_kind():
    room = new_room()
    add_client(room)

    room.refused("dzial_pasuje_do_rodzaju", "wpisz_umowe_klienta", **contract_fields(dzial="najem"))
    room.refused(
        "typ_teczki_pasuje_do_rodzaju", "wpisz_umowe_klienta", **contract_fields(typ_teczki="najem")
    )
    room.refused(
        "nieruchomosc_tylko_przy_umowie_o_nieruchomosc",
        "wpisz_umowe_klienta",
        **contract_fields(nieruchomosc_adres=None),
    )
    room.refused(
        "nieruchomosc_tylko_przy_umowie_o_nieruchomosc",
        "wpisz_umowe_klienta",
        **contract_fields(rodzaj="posrednictwo_kupna", typ_teczki="poszukiwanie", numer_kw=None),
    )
    room.refused(
        "obie_strony_tylko_przy_posrednictwie",
        "wpisz_umowe_klienta",
        **contract_fields(
            rodzaj="zarzadzanie_najem_dlugoterminowy",
            dzial="najem",
            typ_teczki="najem",
            obie_strony=1,
        ),
    )
    room.refused(
        "koniec_nie_przed_zawarciem", "wpisz_umowe_klienta", **contract_fields(koniec="2026-01-01")
    )

    add_contract(room, "sprzedaz", obie_strony=1, koniec="2026-09-02")
    add_contract(
        room,
        "kupno",
        rodzaj="posrednictwo_kupna",
        typ_teczki="poszukiwanie",
        nieruchomosc_adres=None,
        numer_kw=None,
    )
    add_contract(
        room,
        "lokal",
        rodzaj="zarzadzanie_najem_krotkoterminowy",
        dzial="najem",
        typ_teczki="lokal_krotkoterminowy",
        konsument=0,
    )
    assert len(room.run("umowy_klienta", klient_id="nowak")) == 3


def contract_fields(**fields):
    return {
        "id": "umowa",
        "klient_id": "nowak",
        "rodzaj": "posrednictwo_sprzedazy",
        "dzial": "posrednictwo",
        "nieruchomosc_adres": "ul. Polna 7 m. 12, Wrocław",
        "numer_kw": "WR1K/00012345/6",
        "teczka_odnosnik": "teczka-oferty",
        "typ_teczki": "oferta",
        "oznaczenie": "Oferta 2026/031",
        "umowa_odnosnik": "teczka-oferty/umowa",
        "podpisal_za_biuro": "posrednik",
        "zawarto": "2026-03-02",
        "konsument": 1,
        "obie_strony": 0,
        "wpisal": "kierownik-posrednictwa",
        "wpisano": "2026-03-02",
        **fields,
    }


@scenario("Umowy: konsumentem jest tylko osoba fizyczna, a umowy się nie zmienia")
def contract_consumer_and_unchanged():
    room = new_room()
    add_company(room)

    room.refused(
        "umowę jako konsument zawiera tylko osoba fizyczna",
        "wpisz_umowe_klienta",
        **contract_fields(klient_id="lesna"),
    )
    add_contract(room, client_id="lesna", konsument=0)

    room.refused_sql(
        UNCHANGED, "UPDATE klienci_kartoteka_umowy SET obie_strony = 1 WHERE id = 'umowa'"
    )
    room.refused_sql(
        UNCHANGED, "UPDATE klienci_kartoteka_wpisy SET rodzaj_podmiotu = 'osoba_fizyczna'"
    )


@scenario("Stany umowy: od „obowiązuje” do jednego stanu końcowego, który jest ostateczny")
def contract_states_forward():
    room = new_room()
    add_client(room)
    add_contract(room)

    room.refused(
        "stan umowy przechodzi tylko naprzód",
        "zmien_stan_umowy_klienta",
        **state_fields(stan="obowiazuje"),
    )
    room.run("zmien_stan_umowy_klienta", **state_fields(stan="odstapiono", od="2026-03-10"))
    room.refused(
        "stan umowy przechodzi tylko naprzód",
        "zmien_stan_umowy_klienta",
        **state_fields(stan="wykonana"),
    )

    # A contract written past the operation, without its first state.
    room.run("wpisz_klienta", **client_fields(client_id="lis", pesel="85020254321", nazwisko="Lis"))
    room.execute(
        """INSERT INTO klienci_kartoteka_umowy (id, klient_id, rodzaj, dzial, nieruchomosc_adres,
          teczka_odnosnik, typ_teczki, oznaczenie, umowa_odnosnik, podpisal_za_biuro, zawarto,
          konsument, obie_strony, wpisal, wpisano)
        VALUES ('bez-stanu', 'lis', 'posrednictwo_sprzedazy', 'posrednictwo', 'ul. Polna 7',
          'teczka', 'oferta', 'Oferta 1', 'teczka/umowa', 'posrednik', '2026-03-02', 0, 0,
          'kierownik', '2026-03-02')"""
    )
    room.refused(
        "stan umowy przechodzi tylko naprzód",
        "zmien_stan_umowy_klienta",
        **state_fields(umowa_id="bez-stanu", stan="wykonana"),
    )
    room.run(
        "zmien_stan_umowy_klienta",
        **state_fields(umowa_id="bez-stanu", stan="obowiazuje", od="2026-03-02"),
    )


def state_fields(**fields):
    return {
        "umowa_id": "umowa",
        "stan": "wypowiedziana",
        "od": "2026-05-01",
        "zrodlo_odnosnik": "teczka-oferty/wypowiedzenie",
        "wpisal": "kierownik-posrednictwa",
        **fields,
    }


@scenario("Stany umowy: stan nie zaczyna się przed zawarciem ani przed poprzednim stanem")
def contract_states_dates():
    room = new_room()
    add_client(room)
    add_contract(room)

    room.refused(
        "stan umowy nie zaczyna się przed jej zawarciem",
        "zmien_stan_umowy_klienta",
        **state_fields(od="2026-02-01"),
    )
    room.run("zmien_stan_umowy_klienta", **state_fields(od="2026-03-02"))
    room.refused_sql(
        UNCHANGED,
        "UPDATE klienci_kartoteka_umowy_stany SET od = '2026-04-01' WHERE umowa_id = 'umowa'",
    )


@scenario("Informacja o kartotece wskazuje wysyłkę w teczce i się nie zmienia")
def information_unchanged():
    room = new_room()
    full_file(room)

    room.refused_sql(
        "NOT NULL constraint failed: klienci_kartoteka_informacje.wysylka_odnosnik",
        """INSERT INTO klienci_kartoteka_informacje (klient_id, poinformowano, sposob)
        VALUES ('nowak', '2026-03-02T10:00:00Z', 'email')""",
    )
    room.refused_sql(UNCHANGED, "UPDATE klienci_kartoteka_informacje SET sposob = 'list'")
    assert room.run("informacje_klienta", klient_id="nowak")[0]["sposob"] == "email"


# R9 and replacement -------------------------------------------------------------------------------


def tables_of(room):
    return [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'klienci\\_%' ESCAPE '\\'"
        )
    ]


def first_row_copy(room, table):
    """SQL that inserts again the first row of the table, as a careless upsert would."""
    columns = ", ".join(row["name"] for row in room.execute(f"PRAGMA table_info({table})"))

    return (
        f"INTO {table} ({columns}) SELECT {columns} FROM {table} "
        f"WHERE rowid = (SELECT min(rowid) FROM {table})"
    )


@scenario("R9: wierszy kartoteki się nie usuwa ani nie zastępuje przez REPLACE albo upsert")
def no_deletion_or_replacement():
    room = new_room()
    full_file(room)

    tables = tables_of(room)
    assert len(tables) == 8, tables

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
