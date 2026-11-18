"""Scenarios of the office's catalogue of offers (spec/registers/katalog_ofert).

Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
allows and a write it refuses. The first scenario follows a public sale offer from its entry to the
sale, with a price change, an advertisement and a reservation that falls through.
"""

from pathlib import Path

from room import Room

KATALOG_OFERT = Path(__file__).resolve().parents[1] / "spec" / "registers" / "katalog_ofert"

SCENARIOS = []

REPLACEMENT = "wiersza katalogu nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE"
DELETION = "wierszy katalogu się nie usuwa (R9)"
UNCHANGED = "wpisu oferty, wersji ani statusu się nie zmienia"
TRANSITION = "status oferty: aktywna ⇄ zarezerwowana → sprzedana albo wynajęta"
NO_ADVERTISEMENTS = "oferta niepubliczna nie ma ogłoszeń"
FINISHED = "zakończona oferta nie dostaje nowych wersji ani ogłoszeń"


def scenario(title):
    def register(function):
        SCENARIOS.append((title, function))

        return function

    return register


def new_room():
    return Room(KATALOG_OFERT)


# Building blocks ----------------------------------------------------------------------------------


def version_fields(**fields):
    return {
        "od": "2026-05-04",
        "powierzchnia": 54.3,
        "lokalizacja_przyblizona": "Poznań, Jeżyce",
        "cena_sprzedazy": 62_000_000,
        "opis": "Trzy pokoje, balkon, drugie piętro.",
        "zdjecia_sciezka": "zdjecia/oferta-031",
        "zdjecia_skrot": "sha256-zdjec",
        "widocznosc": "publiczna",
        "wylacznosc": 0,
        "zrodlo_odnosnik": "teczka-oferty/zgoda-na-opis",
        "wpisal": "kierownik-posrednictwa",
        **fields,
    }


def add_offer(room, offer_id="oferta", numer_oferty="2026/031", rodzaj_oferty="sprzedaz", **fields):
    room.run(
        "wpisz_oferte",
        id=offer_id,
        numer_oferty=numer_oferty,
        teczka_odnosnik=f"teczka-{offer_id}",
        rodzaj_oferty=rodzaj_oferty,
        rodzaj_nieruchomosci="lokal",
        wpisano="2026-05-04",
        **version_fields(**fields),
    )


def new_version(room, offer_id="oferta", **fields):
    room.run("wpisz_wersje_oferty", oferta_id=offer_id, **version_fields(**fields))


def advertise(room, offer_id="oferta", numer_ogloszenia="OTO-123", od="2026-05-05"):
    room.run(
        "wpisz_ogloszenie",
        oferta_id=offer_id,
        portal="otodom",
        numer_ogloszenia=numer_ogloszenia,
        od=od,
    )


def change_status(room, status, offer_id="oferta", od="2026-06-01"):
    room.run(
        "zmien_status_oferty",
        oferta_id=offer_id,
        status=status,
        od=od,
        zrodlo_odnosnik="teczka-oferty/oferta-zainteresowanego",
        wpisal="kierownik-posrednictwa",
    )


def status_of(room, offer_id="oferta"):
    return room.run("statusy_oferty", oferta_id=offer_id)[-1]["status"]


# A whole offer ------------------------------------------------------------------------------------


@scenario("Oferta: wpis, ogłoszenie, nowa cena, rezerwacja, powrót do aktywnych i sprzedaż")
def whole_offer():
    room = new_room()
    add_offer(room)
    advertise(room)
    new_version(room, od="2026-05-20", cena_sprzedazy=59_000_000)

    change_status(room, "zarezerwowana")
    change_status(room, "aktywna", od="2026-06-10")
    change_status(room, "zarezerwowana", od="2026-06-20")
    change_status(room, "sprzedana_albo_wynajeta", od="2026-07-15")

    offers = room.run("oferty")
    assert [(row["numer_oferty"], row["cena_sprzedazy"], row["status"]) for row in offers] == [
        ("2026/031", 59_000_000, "sprzedana_albo_wynajeta")
    ], offers
    assert (
        room.run("oferta_po_numerze", numer_oferty="2026/031")[0]["teczka_odnosnik"]
        == "teczka-oferta"
    )
    found = room.run("oferta_po_ogloszeniu", portal="otodom", numer_ogloszenia="OTO-123")
    assert found[0]["oferta_id"] == "oferta", found
    assert len(room.run("wersje_oferty", oferta_id="oferta")) == 2


# Versions -----------------------------------------------------------------------------------------


@scenario("Wersja: kwoty pasują do rodzaju oferty")
def amounts_match_kind():
    room = new_room()
    amounts = "oferta sprzedaży ma cenę, oferta najmu czynsz"

    room.refused(amounts, "wpisz_oferte", **offer_fields(cena_sprzedazy=None))
    room.refused(amounts, "wpisz_oferte", **offer_fields(czynsz=320_000))
    room.refused(amounts, "wpisz_oferte", **offer_fields(rodzaj_oferty="najem"))
    room.refused(amounts, "wpisz_oferte", **offer_fields(rodzaj_oferty="sprzedaz_i_najem"))

    add_offer(room)
    add_offer(room, "najem", "2026/032", "najem", cena_sprzedazy=None, czynsz=320_000)
    add_offer(room, "oba", "2026/033", "sprzedaz_i_najem", czynsz=320_000)
    assert len(room.run("oferty")) == 3


def offer_fields(rodzaj_oferty="sprzedaz", **fields):
    return {
        "id": "oferta",
        "numer_oferty": "2026/031",
        "teczka_odnosnik": "teczka-oferty",
        "rodzaj_oferty": rodzaj_oferty,
        "rodzaj_nieruchomosci": "lokal",
        "wpisano": "2026-05-04",
        **version_fields(**fields),
    }


@scenario("Wersja: oferta niepubliczna ma granice zgody, a publiczna ich nie ma")
def consent_limits():
    room = new_room()
    limits = "granice_zgody_tylko_przy_niepublicznej"

    room.refused(limits, "wpisz_oferte", **offer_fields(widocznosc="niepubliczna"))
    room.refused(limits, "wpisz_oferte", **offer_fields(granice_zgody="tylko klienci z kredytem"))
    room.refused("zdjecia_ze_skrotem", "wpisz_oferte", **offer_fields(zdjecia_skrot=None))

    add_offer(
        room,
        widocznosc="niepubliczna",
        granice_zgody="tylko klientom biura z potwierdzonym finansowaniem, bez zdjęć wnętrz",
    )


@scenario("Wersja: zmiana widoczności albo wyłączności wymaga aneksu do umowy")
def visibility_change_needs_annex():
    room = new_room()
    add_offer(room)
    private = {
        "widocznosc": "niepubliczna",
        "granice_zgody": "tylko klientom biura",
        "od": "2026-05-10",
    }
    annex = "zmiana widoczności albo wyłączności wymaga aneksu do umowy pośrednictwa"

    room.refused(annex, "wpisz_wersje_oferty", oferta_id="oferta", **version_fields(**private))
    room.refused(annex, "wpisz_wersje_oferty", oferta_id="oferta", **version_fields(wylacznosc=1))

    new_version(room, cena_sprzedazy=60_000_000, od="2026-05-10")
    new_version(room, aneks_odnosnik="teczka-oferty/aneks-1", **private)
    assert room.run("oferty")[0]["widocznosc"] == "niepubliczna"


@scenario("Wersja: nie obowiązuje wcześniej niż poprzednia i poprawia wersję tej samej oferty")
def version_order_and_correction():
    room = new_room()
    add_offer(room)
    add_offer(room, "druga", "2026/032")
    other_version = room.run("wersje_oferty", oferta_id="druga")[0]["id"]
    own_version = room.run("wersje_oferty", oferta_id="oferta")[0]["id"]

    room.refused(
        "nowa wersja oferty nie obowiązuje wcześniej niż poprzednia",
        "wpisz_wersje_oferty",
        oferta_id="oferta",
        **version_fields(od="2026-05-01"),
    )
    room.refused(
        "wersja poprawia wcześniejszą wersję tej samej oferty",
        "wpisz_wersje_oferty",
        oferta_id="oferta",
        **version_fields(poprawia_id=other_version),
    )

    new_version(room, powierzchnia=53.4, poprawia_id=own_version)
    room.refused_sql(UNCHANGED, "UPDATE katalog_ofert_oferty_wersje SET opis = 'inny'")
    room.refused_sql(UNCHANGED, "UPDATE katalog_ofert_oferty_wpisy SET numer_oferty = '2026/099'")


# Advertisements -----------------------------------------------------------------------------------


@scenario("Ogłoszenia: tylko oferty publicznej, a przejście na niepubliczną po ich końcu")
def advertisements_only_public():
    room = new_room()
    add_offer(room, widocznosc="niepubliczna", granice_zgody="tylko klientom biura")
    room.refused(
        NO_ADVERTISEMENTS,
        "wpisz_ogloszenie",
        oferta_id="oferta",
        portal="otodom",
        numer_ogloszenia="OTO-1",
        od="2026-05-05",
    )

    add_offer(room, "publiczna", "2026/032")
    advertise(room, "publiczna")
    private = version_fields(
        widocznosc="niepubliczna",
        granice_zgody="tylko klientom biura",
        aneks_odnosnik="teczka/aneks",
        od="2026-06-01",
    )
    room.refused(NO_ADVERTISEMENTS, "wpisz_wersje_oferty", oferta_id="publiczna", **private)

    advertisement_id = room.run("ogloszenia_ofert")[0]["id"]
    room.run("zakoncz_ogloszenie", id=advertisement_id, do="2026-05-31")
    room.run("wpisz_wersje_oferty", oferta_id="publiczna", **private)


@scenario("Ogłoszenia: powstają trwające, kończą się raz, a numer portalu jest jeden")
def advertisement_rows():
    room = new_room()
    add_offer(room)

    room.refused_sql(
        "ogłoszenie powstaje jako trwające",
        """INSERT INTO katalog_ofert_oferty_ogloszenia (oferta_id, portal, numer_ogloszenia, od, do)
        VALUES ('oferta', 'otodom', 'OTO-1', '2026-05-05', '2026-05-06')""",
    )
    advertise(room)
    room.refused(
        REPLACEMENT,
        "wpisz_ogloszenie",
        oferta_id="oferta",
        portal="otodom",
        numer_ogloszenia="OTO-123",
        od="2026-05-06",
    )

    advertisement_id = room.run("ogloszenia_ofert")[0]["id"]
    room.refused(
        "koniec_nie_przed_poczatkiem", "zakoncz_ogloszenie", id=advertisement_id, do="2026-05-01"
    )
    room.run("zakoncz_ogloszenie", id=advertisement_id, do="2026-05-31")
    room.refused(
        "ogłoszenia się nie zmienia; może się tylko raz zakończyć",
        "zakoncz_ogloszenie",
        id=advertisement_id,
        do="2026-06-30",
    )


# Status -------------------------------------------------------------------------------------------


@scenario("Status: aktywna ⇄ zarezerwowana → sprzedana, z każdego do wycofanej, końcowe ostateczne")
def status_transitions():
    room = new_room()
    add_offer(room)

    room.refused(TRANSITION, "zmien_status_oferty", **status_fields("sprzedana_albo_wynajeta"))
    room.refused(TRANSITION, "zmien_status_oferty", **status_fields("aktywna"))
    change_status(room, "zarezerwowana")
    room.refused(TRANSITION, "zmien_status_oferty", **status_fields("zarezerwowana"))
    change_status(room, "wycofana", od="2026-06-15")
    room.refused(TRANSITION, "zmien_status_oferty", **status_fields("aktywna", od="2026-07-01"))

    add_offer(room, "druga", "2026/032")
    change_status(room, "wycofana", "druga")
    assert status_of(room, "druga") == "wycofana"

    # A status written past the operation, for an offer without the first one.
    room.execute(
        """INSERT INTO katalog_ofert_oferty_wpisy (id, numer_oferty, teczka_odnosnik, rodzaj_oferty,
          rodzaj_nieruchomosci, wpisal, wpisano)
        VALUES ('bez-statusu', '2026/040', 'teczka', 'sprzedaz', 'lokal', 'kierownik', '2026-05-04')"""
    )
    room.refused(
        TRANSITION, "zmien_status_oferty", **status_fields("zarezerwowana", offer_id="bez-statusu")
    )
    room.refused_sql(UNCHANGED, "UPDATE katalog_ofert_oferty_statusy SET od = '2026-01-01'")


def status_fields(status, offer_id="oferta", od="2026-06-01"):
    return {
        "oferta_id": offer_id,
        "status": status,
        "od": od,
        "zrodlo_odnosnik": "teczka-oferty/umowa",
        "wpisal": "kierownik-posrednictwa",
    }


@scenario(
    "Status nie zmienia się przed poprzednim; zakończona oferta nie dostaje wersji ani ogłoszeń"
)
def finished_offer():
    room = new_room()
    add_offer(room)

    room.refused(
        "status oferty nie zmienia się przed poprzednim",
        "zmien_status_oferty",
        **status_fields("zarezerwowana", od="2026-05-01"),
    )
    change_status(room, "wycofana")

    room.refused(
        FINISHED, "wpisz_wersje_oferty", oferta_id="oferta", **version_fields(od="2026-06-02")
    )
    room.refused(
        FINISHED,
        "wpisz_ogloszenie",
        oferta_id="oferta",
        portal="otodom",
        numer_ogloszenia="OTO-9",
        od="2026-06-02",
    )
    assert room.run("oferty")[0]["status"] == "wycofana"


# R9 and replacement -------------------------------------------------------------------------------


def first_row_copy(room, table):
    """SQL that inserts again the first row of the table, as a careless upsert would."""
    columns = ", ".join(row["name"] for row in room.execute(f"PRAGMA table_info({table})"))

    return (
        f"INTO {table} ({columns}) SELECT {columns} FROM {table} "
        f"WHERE rowid = (SELECT min(rowid) FROM {table})"
    )


@scenario("R9: wierszy katalogu się nie usuwa ani nie zastępuje przez REPLACE albo upsert")
def no_deletion_or_replacement():
    room = new_room()
    add_offer(room)
    advertise(room)

    tables = [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'katalog\\_%' ESCAPE '\\'"
        )
    ]
    assert len(tables) == 4, tables

    for table in tables:
        room.refused_sql(REPLACEMENT, f"INSERT OR REPLACE {first_row_copy(room, table)}")
        room.refused_sql(
            REPLACEMENT,
            f"INSERT {first_row_copy(room, table)} ON CONFLICT (id) DO UPDATE SET id = excluded.id",
        )
        room.refused_sql(
            DELETION, f"DELETE FROM {table} WHERE rowid = (SELECT min(rowid) FROM {table})"
        )

    # The offer number and the portal's advertisement number are UNIQUE keys of their own.
    room.refused_sql(
        REPLACEMENT,
        """INSERT OR REPLACE INTO katalog_ofert_oferty_wpisy (id, numer_oferty, teczka_odnosnik,
          rodzaj_oferty, rodzaj_nieruchomosci, wpisal, wpisano)
        VALUES ('nowy-klucz', '2026/031', 'teczka', 'sprzedaz', 'lokal', 'kierownik', '2026-05-04')""",
    )
    room.refused_sql(
        REPLACEMENT,
        """INSERT OR REPLACE INTO katalog_ofert_oferty_ogloszenia (oferta_id, portal,
          numer_ogloszenia, od)
        VALUES ('oferta', 'otodom', 'OTO-123', '2026-05-06')""",
    )
