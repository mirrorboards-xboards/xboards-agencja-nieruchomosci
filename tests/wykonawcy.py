"""Scenarios of the register of contractors (spec/registers/wykonawcy).

Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
allows and a write it refuses. The first scenario follows a cleaning company from its admission
with a price list to its suspension and removal, as the profile describes it.
"""

from pathlib import Path

from room import Room

WYKONAWCY = Path(__file__).resolve().parents[1] / "spec" / "registers" / "wykonawcy"

SCENARIOS = []

REPLACEMENT = "wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE"
DELETION = "wierszy rejestru się nie usuwa (R9)"
UNCHANGED = "decyzji, wpisu wykonawcy ani jego stanu się nie zmienia"
AFTER_DATA_DECISION = (
    "dane, osobę, kanał i obszar wpisuje się po decyzji o przyjęciu albo zmianie danych"
)
AFTER_END_DECISION = (
    "koniec osoby, kanału i obszaru wpisuje się po decyzji o zmianie danych albo wykreśleniu"
)
NOT_REMOVED = "wykreślony wykonawca nie dostaje nowych danych, osób, kanałów ani obszarów"


def scenario(title):
    def register(function):
        SCENARIOS.append((title, function))

        return function

    return register


def new_room():
    return Room(WYKONAWCY)


# Building blocks ----------------------------------------------------------------------------------


def decide(room, decision_id, rodzaj, podpisano="2026-04-01T10:00:00Z"):
    """A decision of the human, signed with their key and copied into the register."""
    room.run(
        "wpisz_decyzje_o_wykonawcy",
        id=decision_id,
        rodzaj=rodzaj,
        decyduje="wlasciciel-kierownika-najmu",
        tresc=f"decyzja: {rodzaj}",
        dotyczy_skrot="sha256-wpisu",
        podpis="podpis-kluczem-xkeys",
        podpisano=podpisano,
        wpisal="kierownik-najmu",
        wpisano="2026-04-01T12:00:00Z",
    )


def contractor_data(**fields):
    return {
        "firma": "Czysto sp. c.",
        "nip": "5250000000",
        "ceidg": 1,
        "adres": "ul. Szewska 1, Kraków",
        "zakres_uslug": "sprzątanie i pranie; bez mycia okien na wysokości",
        "srodki_wlasne": 1,
        "umowa_od": "2026-04-01",
        "umowa_do": "2027-03-31",
        "umowa_podpisal": "wlasciciel-kierownika-najmu",
        "umowa_sciezka": "umowy/czysto.pdf",
        "umowa_skrot": "sha256-umowy",
        "wpisano": "2026-04-01T12:00:00Z",
        **fields,
    }


def admit(room, contractor_id="czysto", decision_id="przyjecie-czysto", **fields):
    decide(room, decision_id, "przyjecie")
    room.run(
        "wpisz_wykonawce",
        **{
            "id": contractor_id,
            "nazwa": "Czysto sp. c.",
            "rodzaj": "sprzatanie",
            "decyzja_id": decision_id,
            **contractor_data(),
            **fields,
        },
    )


def add_worker(room, decision_id="przyjecie-czysto", contractor_id="czysto", **fields):
    room.run(
        "wpisz_osobe_wykonawcy",
        **{
            "wykonawca_id": contractor_id,
            "imiona": "Maria",
            "nazwisko": "Wójcik",
            "telefon": "+48600300400",
            "kody_do_zamkow": 1,
            "od": "2026-04-01",
            "decyzja_id": decision_id,
            **fields,
        },
    )


def add_channel(room, decision_id="przyjecie-czysto", **fields):
    room.run(
        "wpisz_kanal_wykonawcy",
        **{
            "wykonawca_id": "czysto",
            "rodzaj": "sms",
            "adres": "+48600300400",
            "od": "2026-04-01",
            "decyzja_id": decision_id,
            **fields,
        },
    )


def add_area(room, decision_id="przyjecie-czysto", **fields):
    room.run(
        "wpisz_obszar_wykonawcy",
        **{
            "wykonawca_id": "czysto",
            "obszar": "Stare Miasto",
            "dostepnosc": "pon.–sob. 8–20",
            "od": "2026-04-01",
            "decyzja_id": decision_id,
            **fields,
        },
    )


def change_state(room, decision_id, stan, od="2026-06-01", powod="nie przychodzi na zlecenia"):
    room.run(
        "zmien_stan_wykonawcy",
        wykonawca_id="czysto",
        stan=stan,
        od=od,
        powod=powod,
        decyzja_id=decision_id,
    )


def price_list_version(room, version_id="wersja-1", numer=1, obowiazuje_od="2026-04-01", **fields):
    room.run(
        "wpisz_wersje_cennika",
        **{
            "id": version_id,
            "cennik_id": "cennik",
            "numer": numer,
            "obowiazuje_od": obowiazuje_od,
            "wpisano": "2026-04-01T12:00:00Z",
            "decyzja_id": "cennik-decyzja",
            **fields,
        },
    )


def add_item(room, version_id="wersja-1", cena=15000):
    room.run(
        "dopisz_pozycje_cennika",
        wersja_id=version_id,
        usluga="sprzątanie po wyjeździe gościa",
        doplata=0,
        jednostka="lokal do 40 m²",
        cena=cena,
        z_vat=1,
    )


def closed_price_list(room):
    decide(room, "cennik-decyzja", "cennik")
    room.run("wpisz_cennik", id="cennik", nazwa="Sprzątanie Stare Miasto", wpisano="2026-04-01")
    price_list_version(room)
    add_item(room)
    room.run("zamknij_wersje_cennika", id="wersja-1")


def full_register(room):
    """A contractor and a price list with a row in every table of the catalog."""
    admit(room)
    add_worker(room)
    add_channel(room)
    add_area(room)
    closed_price_list(room)
    decide(room, "przypisanie", "przypisanie")
    room.run(
        "przypisz_cennik",
        cennik_id="cennik",
        wykonawca_id="czysto",
        od="2026-04-02",
        decyzja_id="przypisanie",
    )


# A whole contractor -------------------------------------------------------------------------------


@scenario("Wykonawca: przyjęcie z cennikiem, wstrzymanie, przywrócenie i wykreślenie")
def whole_contractor():
    room = new_room()
    full_register(room)

    contractors = room.run("wykonawcy")
    assert [(row["id"], row["stan"]) for row in contractors] == [("czysto", "zatwierdzony")]
    in_force = room.run("obowiazujaca_wersja_cennika", cennik_id="cennik", dzien="2026-05-01")
    assert in_force[0]["id"] == "wersja-1", in_force

    decide(room, "wstrzymanie", "wstrzymanie")
    change_state(room, "wstrzymanie", "wstrzymany")
    decide(room, "przywrocenie", "przywrocenie")
    change_state(room, "przywrocenie", "zatwierdzony", od="2026-06-15", powod=None)
    decide(room, "wykreslenie", "wykreslenie")
    change_state(room, "wykreslenie", "wykreslony", od="2026-09-01", powod="koniec umowy")

    worker_id = room.run("osoby_wykonawcy", wykonawca_id="czysto")[0]["id"]
    room.run(
        "zakoncz_osobe_wykonawcy",
        id=worker_id,
        do="2026-09-01",
        zakonczenie_decyzja_id="wykreslenie",
    )
    assert room.run("osoby_wykonawcow") == []
    states = [row["stan"] for row in room.run("stany_wykonawcy", wykonawca_id="czysto")]
    assert states == ["zatwierdzony", "wstrzymany", "zatwierdzony", "wykreslony"], states


# Decisions ----------------------------------------------------------------------------------------


@scenario("Decyzje: podpis w komplecie, wpis nie przed podpisem, decyzji się nie zmienia")
def decisions():
    room = new_room()
    decision = {
        "id": "decyzja",
        "rodzaj": "przyjecie",
        "decyduje": "wlasciciel",
        "tresc": "przyjęcie",
        "dotyczy_skrot": "sha256",
        "podpis": "podpis",
        "podpisano": "2026-04-01T10:00:00Z",
        "wpisal": "kierownik-najmu",
        "wpisano": "2026-04-01T12:00:00Z",
    }

    room.refused(
        "NOT NULL constraint failed: wykonawcy_spis_decyzje.podpis",
        "wpisz_decyzje_o_wykonawcy",
        **{**decision, "podpis": None},
    )
    room.refused(
        "wpis_nie_przed_podpisem",
        "wpisz_decyzje_o_wykonawcy",
        **{**decision, "wpisano": "2026-04-01T09:00:00Z"},
    )

    room.run("wpisz_decyzje_o_wykonawcy", **decision)
    room.refused_sql(UNCHANGED, "UPDATE wykonawcy_spis_decyzje SET tresc = 'inna'")


@scenario("Wpis wykonawcy: po decyzji o przyjęciu; specjalność ma wykonawca napraw")
def contractor_entry():
    room = new_room()
    decide(room, "zmiana", "zmiana_danych")

    room.refused(
        "wykonawcę wpisuje się po decyzji o przyjęciu",
        "wpisz_wykonawce",
        id="czysto",
        nazwa="Czysto sp. c.",
        rodzaj="sprzatanie",
        decyzja_id="zmiana",
        **contractor_data(),
    )

    decide(room, "przyjecie-hydraulika", "przyjecie")
    room.refused(
        "specjalnosc_przy_naprawach",
        "wpisz_wykonawce",
        id="kowalski",
        nazwa="Jan Kowalski",
        rodzaj="naprawy_i_serwis",
        decyzja_id="przyjecie-hydraulika",
        **contractor_data(),
    )
    room.run(
        "wpisz_wykonawce",
        id="kowalski",
        nazwa="Jan Kowalski",
        rodzaj="naprawy_i_serwis",
        specjalnosc="hydraulik",
        decyzja_id="przyjecie-hydraulika",
        **contractor_data(),
    )
    room.refused_sql(UNCHANGED, "UPDATE wykonawcy_spis_wpisy SET nazwa = 'Kowalski i syn'")


@scenario("Dane wykonawcy: KRS albo CEIDG, umowa i polisa w komplecie, nowa wersja po decyzji")
def contractor_data_versions():
    room = new_room()
    decide(room, "przyjecie-czysto", "przyjecie")
    entry = {
        "id": "czysto",
        "nazwa": "Czysto sp. c.",
        "rodzaj": "sprzatanie",
        "decyzja_id": "przyjecie-czysto",
    }

    room.refused("krs_albo_ceidg", "wpisz_wykonawce", **entry, **contractor_data(krs="0000123456"))
    room.refused(
        "koniec_umowy_nie_przed_poczatkiem",
        "wpisz_wykonawce",
        **entry,
        **contractor_data(umowa_do="2026-01-01"),
    )
    room.refused(
        "polisa_w_komplecie", "wpisz_wykonawce", **entry, **contractor_data(polisa_do="2027-01-01")
    )
    room.run("wpisz_wykonawce", **entry, **contractor_data())

    decide(room, "wstrzymanie", "wstrzymanie")
    room.refused(
        AFTER_DATA_DECISION,
        "wpisz_dane_wykonawcy",
        wykonawca_id="czysto",
        decyzja_id="wstrzymanie",
        **contractor_data(),
    )
    room.refused_sql(
        "wersja danych wykonawcy powstaje jako aktualna",
        """INSERT INTO wykonawcy_spis_dane (wykonawca_id, aktualne, firma, adres, zakres_uslug,
          srodki_wlasne, umowa_od, umowa_podpisal, umowa_sciezka, umowa_skrot, decyzja_id, wpisano)
        VALUES ('czysto', 0, 'Czysto', 'ul. Szewska 1', 'sprzątanie', 0, '2026-04-01', 'wlasciciel',
          'umowa.pdf', 'sha256', 'przyjecie-czysto', '2026-04-01')""",
    )

    decide(room, "przedluzenie", "zmiana_danych")
    room.run(
        "wpisz_dane_wykonawcy",
        wykonawca_id="czysto",
        decyzja_id="przedluzenie",
        **contractor_data(
            umowa_do="2028-03-31",
            polisa_do="2027-12-31",
            polisa_sciezka="polisa.pdf",
            polisa_skrot="sha256-polisy",
        ),
    )
    versions = room.run("dane_wykonawcy", wykonawca_id="czysto")
    assert [(row["umowa_do"], row["aktualne"]) for row in versions] == [
        ("2028-03-31", 1),
        ("2027-03-31", 0),
    ], versions

    room.refused_sql(
        "danych wykonawcy się nie poprawia",
        "UPDATE wykonawcy_spis_dane SET adres = 'ul. Inna 2' WHERE aktualne = 1",
    )
    room.refused_sql(
        "danych wykonawcy się nie poprawia",
        "UPDATE wykonawcy_spis_dane SET aktualne = 0 WHERE aktualne = 1",
    )


# Workers, channels and areas ----------------------------------------------------------------------


@scenario("Osoby, kanały i obszary: wpis po decyzji o przyjęciu albo zmianie danych")
def rows_after_decision():
    room = new_room()
    admit(room)
    decide(room, "cennik-decyzja", "cennik")

    room.refused(
        AFTER_DATA_DECISION, "wpisz_osobe_wykonawcy", **worker_fields(decyzja_id="cennik-decyzja")
    )
    room.refused(
        AFTER_DATA_DECISION, "wpisz_kanal_wykonawcy", **channel_fields(decyzja_id="cennik-decyzja")
    )
    room.refused(
        AFTER_DATA_DECISION, "wpisz_obszar_wykonawcy", **area_fields(decyzja_id="cennik-decyzja")
    )

    decide(room, "nowa-brygadzistka", "zmiana_danych")
    add_worker(room, "nowa-brygadzistka")
    add_channel(room, "nowa-brygadzistka")
    add_area(room, "nowa-brygadzistka")


def worker_fields(**fields):
    return {
        "wykonawca_id": "czysto",
        "imiona": "Maria",
        "nazwisko": "Wójcik",
        "telefon": "+48600300400",
        "kody_do_zamkow": 1,
        "od": "2026-04-01",
        "decyzja_id": "przyjecie-czysto",
        **fields,
    }


def channel_fields(**fields):
    return {
        "wykonawca_id": "czysto",
        "rodzaj": "sms",
        "adres": "+48600300400",
        "od": "2026-04-01",
        "decyzja_id": "przyjecie-czysto",
        **fields,
    }


def area_fields(**fields):
    return {
        "wykonawca_id": "czysto",
        "obszar": "Stare Miasto",
        "dostepnosc": "pon.–sob. 8–20",
        "od": "2026-04-01",
        "decyzja_id": "przyjecie-czysto",
        **fields,
    }


@scenario("Osoby, kanały i obszary: powstają bieżące, kończą się raz i tylko decyzją")
def rows_end_with_decision():
    room = new_room()
    admit(room)

    for table, columns in (
        ("wykonawcy_spis_osoby", "imiona, nazwisko, telefon, kody_do_zamkow"),
        ("wykonawcy_spis_kanaly", "rodzaj, adres"),
        ("wykonawcy_spis_obszary", "obszar, dostepnosc"),
    ):
        values = {
            "wykonawcy_spis_osoby": "'Maria', 'Wójcik', '+48600', 1",
            "wykonawcy_spis_kanaly": "'sms', '+48600'",
            "wykonawcy_spis_obszary": "'Kazimierz', 'pon.–pt.'",
        }[table]
        room.refused_sql(
            "osoba, kanał i obszar powstają jako bieżące",
            f"""INSERT INTO {table} (wykonawca_id, {columns}, od, do, decyzja_id,
              zakonczenie_decyzja_id)
            VALUES ('czysto', {values}, '2026-04-01', '2026-05-01', 'przyjecie-czysto',
              'przyjecie-czysto')""",
        )

    add_worker(room)
    add_channel(room)
    add_area(room)
    worker_id = room.run("osoby_wykonawcy", wykonawca_id="czysto")[0]["id"]
    channel_id = room.run("kanaly_wykonawcow")[0]["id"]
    area_id = room.run("obszary_wykonawcow")[0]["id"]
    decide(room, "zmiana", "zmiana_danych")
    decide(room, "wstrzymanie", "wstrzymanie")

    room.refused_sql(
        "koniec_z_decyzja",
        f"UPDATE wykonawcy_spis_osoby SET do = '2026-05-01' WHERE id = {worker_id}",
    )
    room.refused(
        AFTER_END_DECISION,
        "zakoncz_osobe_wykonawcy",
        id=worker_id,
        do="2026-05-01",
        zakonczenie_decyzja_id="wstrzymanie",
    )
    room.refused(
        AFTER_END_DECISION,
        "zakoncz_kanal_wykonawcy",
        id=channel_id,
        do="2026-05-01",
        zakonczenie_decyzja_id="wstrzymanie",
    )
    room.refused(
        AFTER_END_DECISION,
        "zakoncz_obszar_wykonawcy",
        id=area_id,
        do="2026-05-01",
        zakonczenie_decyzja_id="wstrzymanie",
    )
    room.refused(
        "koniec_nie_przed_poczatkiem",
        "zakoncz_osobe_wykonawcy",
        id=worker_id,
        do="2026-01-01",
        zakonczenie_decyzja_id="zmiana",
    )

    room.run(
        "zakoncz_osobe_wykonawcy", id=worker_id, do="2026-05-01", zakonczenie_decyzja_id="zmiana"
    )
    room.run(
        "zakoncz_kanal_wykonawcy", id=channel_id, do="2026-05-01", zakonczenie_decyzja_id="zmiana"
    )
    room.run(
        "zakoncz_obszar_wykonawcy", id=area_id, do="2026-05-01", zakonczenie_decyzja_id="zmiana"
    )

    unchanged = "osoby, kanału ani obszaru się nie zmienia; mogą się tylko raz zakończyć decyzją"
    room.refused(
        unchanged,
        "zakoncz_osobe_wykonawcy",
        id=worker_id,
        do="2026-06-01",
        zakonczenie_decyzja_id="zmiana",
    )
    room.refused_sql(
        unchanged, f"UPDATE wykonawcy_spis_osoby SET telefon = '+48111' WHERE id = {worker_id}"
    )
    room.refused_sql(
        unchanged, f"UPDATE wykonawcy_spis_kanaly SET adres = '+48111' WHERE id = {channel_id}"
    )
    room.refused_sql(
        unchanged, f"UPDATE wykonawcy_spis_obszary SET obszar = 'Podgórze' WHERE id = {area_id}"
    )


# States -------------------------------------------------------------------------------------------


@scenario("Stan wykonawcy: wstrzymany ⇄ zatwierdzony, wykreślony na zawsze, każdy z decyzją")
def contractor_states():
    room = new_room()
    admit(room)
    decide(room, "wstrzymanie", "wstrzymanie")
    decide(room, "przywrocenie", "przywrocenie")
    decide(room, "wykreslenie", "wykreslenie")
    transition = "stan wykonawcy: zatwierdzony przy przyjęciu, wstrzymany ⇄ zatwierdzony"
    decision_kind = "zatwierdzenie ma decyzję o przyjęciu albo przywróceniu"

    room.refused(transition, "zmien_stan_wykonawcy", **state_fields("przywrocenie", "zatwierdzony"))
    room.refused(
        decision_kind, "zmien_stan_wykonawcy", **state_fields("przywrocenie", "wstrzymany")
    )
    room.refused(
        "wstrzymanie_i_wykreslenie_z_powodem",
        "zmien_stan_wykonawcy",
        **state_fields("wstrzymanie", "wstrzymany", powod=None),
    )

    room.run("zmien_stan_wykonawcy", **state_fields("wstrzymanie", "wstrzymany"))
    room.refused(
        decision_kind, "zmien_stan_wykonawcy", **state_fields("wstrzymanie", "zatwierdzony")
    )
    room.run("zmien_stan_wykonawcy", **state_fields("przywrocenie", "zatwierdzony", powod=None))
    room.run("zmien_stan_wykonawcy", **state_fields("wykreslenie", "wykreslony"))
    room.refused(transition, "zmien_stan_wykonawcy", **state_fields("przywrocenie", "zatwierdzony"))

    room.refused_sql(UNCHANGED, "UPDATE wykonawcy_spis_stany SET powod = 'inny'")


def state_fields(decision_id, stan, powod="nie trzyma standardu"):
    return {
        "wykonawca_id": "czysto",
        "stan": stan,
        "od": "2026-06-01",
        "powod": powod,
        "decyzja_id": decision_id,
    }


@scenario("Wykreślony wykonawca nie dostaje nowych danych, osób, kanałów ani obszarów")
def removed_contractor():
    room = new_room()
    admit(room)
    decide(room, "zmiana", "zmiana_danych")
    decide(room, "wstrzymanie", "wstrzymanie")
    decide(room, "wykreslenie", "wykreslenie")

    room.run("zmien_stan_wykonawcy", **state_fields("wstrzymanie", "wstrzymany"))
    add_worker(room, "zmiana")

    room.run("zmien_stan_wykonawcy", **state_fields("wykreslenie", "wykreslony"))
    room.refused(NOT_REMOVED, "wpisz_osobe_wykonawcy", **worker_fields(decyzja_id="zmiana"))
    room.refused(NOT_REMOVED, "wpisz_kanal_wykonawcy", **channel_fields(decyzja_id="zmiana"))
    room.refused(NOT_REMOVED, "wpisz_obszar_wykonawcy", **area_fields(decyzja_id="zmiana"))
    room.refused(
        NOT_REMOVED,
        "wpisz_dane_wykonawcy",
        wykonawca_id="czysto",
        decyzja_id="zmiana",
        **contractor_data(),
    )


# Price lists --------------------------------------------------------------------------------------


@scenario("Cennik: wersja po decyzji o cenniku powstaje jako projekt i nie obowiązuje wstecz")
def price_list_version_rules():
    room = new_room()
    decide(room, "cennik-decyzja", "cennik")
    decide(room, "przyjecie", "przyjecie")
    room.run("wpisz_cennik", id="cennik", nazwa="Sprzątanie Stare Miasto", wpisano="2026-04-01")

    room.refused(
        "wersję cennika wpisuje się po decyzji o cenniku",
        "wpisz_wersje_cennika",
        **version_fields(decyzja_id="przyjecie"),
    )
    room.refused(
        "obowiazuje_od_nie_przed_wpisem",
        "wpisz_wersje_cennika",
        **version_fields(obowiazuje_od="2026-03-31"),
    )
    room.refused(
        "wersja cennika ma kolejny numer", "wpisz_wersje_cennika", **version_fields(numer=2)
    )
    room.refused_sql(
        "wersja cennika powstaje jako projekt",
        """INSERT INTO wykonawcy_cenniki_wersje (id, cennik_id, numer, obowiazuje_od, wpisano, stan,
          decyzja_id)
        VALUES ('od-razu', 'cennik', 1, '2026-04-01', '2026-04-01T12:00:00Z', 'zamknieta',
          'cennik-decyzja')""",
    )

    room.run("wpisz_wersje_cennika", **version_fields())
    room.refused(
        "nie obowiązuje wcześniej niż poprzednia",
        "wpisz_wersje_cennika",
        **version_fields(
            id="wersja-2", numer=2, obowiazuje_od="2026-04-15", wpisano="2026-04-02T12:00:00Z"
        ),
    )
    room.run(
        "wpisz_wersje_cennika",
        **version_fields(id="wersja-2", numer=2, obowiazuje_od="2026-05-01"),
    )


def version_fields(**fields):
    return {
        "id": "wersja-1",
        "cennik_id": "cennik",
        "numer": 1,
        "obowiazuje_od": "2026-05-01",
        "wpisano": "2026-04-01T12:00:00Z",
        "decyzja_id": "cennik-decyzja",
        **fields,
    }


@scenario("Cennik: pozycje tylko w projekcie; zamknięta wersja ma pozycje i się nie zmienia")
def price_list_items():
    room = new_room()
    decide(room, "cennik-decyzja", "cennik")
    room.run("wpisz_cennik", id="cennik", nazwa="Sprzątanie Stare Miasto", wpisano="2026-04-01")
    price_list_version(room)

    room.refused(
        "zamknięta wersja cennika ma co najmniej jedną pozycję",
        "zamknij_wersje_cennika",
        id="wersja-1",
    )
    add_item(room, cena=14000)
    item_id = room.run("pozycje_cennika", wersja_id="wersja-1")[0]["id"]
    room.run(
        "popraw_pozycje_cennika",
        id=item_id,
        usluga="sprzątanie po wyjeździe gościa",
        doplata=0,
        jednostka="lokal do 40 m²",
        cena=15000,
        z_vat=1,
    )
    room.run("zamknij_wersje_cennika", id="wersja-1")

    room.refused(
        "pozycje dopisuje się tylko do projektu wersji cennika",
        "dopisz_pozycje_cennika",
        wersja_id="wersja-1",
        usluga="pranie",
        doplata=0,
        jednostka="komplet pościeli",
        cena=2000,
        z_vat=1,
    )
    room.refused_sql(
        "pozycji zamkniętej wersji cennika się nie zmienia",
        f"UPDATE wykonawcy_cenniki_pozycje SET cena = 1 WHERE id = {item_id}",
    )
    room.refused_sql(
        "wersji cennika się nie zmienia",
        "UPDATE wykonawcy_cenniki_wersje SET obowiazuje_od = '2026-06-01' WHERE id = 'wersja-1'",
    )
    room.refused_sql(
        "cennika się nie zmienia; zmiana stawek to nowa wersja",
        "UPDATE wykonawcy_cenniki_wpisy SET nazwa = 'inna'",
    )
    assert room.run("pozycje_cennika", wersja_id="wersja-1")[0]["cena"] == 15000


@scenario("Przypisanie cennika: zatwierdzonemu wykonawcy, z zamkniętą wersją i decyzjami")
def price_list_assignment():
    room = new_room()
    admit(room)
    decide(room, "cennik-decyzja", "cennik")
    decide(room, "przypisanie", "przypisanie")
    decide(room, "koniec-przypisania", "przypisanie")
    room.run("wpisz_cennik", id="cennik", nazwa="Sprzątanie Stare Miasto", wpisano="2026-04-01")
    price_list_version(room)
    assignment = {
        "cennik_id": "cennik",
        "wykonawca_id": "czysto",
        "od": "2026-04-02",
        "decyzja_id": "przypisanie",
    }

    room.refused(
        "przypisuje się cennik, który ma zamkniętą wersję", "przypisz_cennik", **assignment
    )
    add_item(room)
    room.run("zamknij_wersje_cennika", id="wersja-1")
    room.refused(
        "przypisanie cennika i jego koniec wpisuje się po decyzji o przypisaniu",
        "przypisz_cennik",
        **{**assignment, "decyzja_id": "cennik-decyzja"},
    )
    room.refused_sql(
        "przypisanie cennika powstaje jako bieżące",
        """INSERT INTO wykonawcy_cenniki_przypisania (cennik_id, wykonawca_id, od, do, decyzja_id,
          zakonczenie_decyzja_id)
        VALUES ('cennik', 'czysto', '2026-04-02', '2026-05-01', 'przypisanie', 'przypisanie')""",
    )

    room.run("przypisz_cennik", **assignment)
    assignment_id = room.run("przypisania_cennikow")[0]["id"]
    room.refused(
        "przypisanie cennika i jego koniec wpisuje się po decyzji o przypisaniu",
        "zakoncz_przypisanie_cennika",
        id=assignment_id,
        do="2026-06-01",
        zakonczenie_decyzja_id="cennik-decyzja",
    )
    room.run(
        "zakoncz_przypisanie_cennika",
        id=assignment_id,
        do="2026-06-01",
        zakonczenie_decyzja_id="koniec-przypisania",
    )
    room.refused_sql(
        "przypisania cennika się nie zmienia",
        f"UPDATE wykonawcy_cenniki_przypisania SET do = '2026-07-01' WHERE id = {assignment_id}",
    )

    decide(room, "wstrzymanie", "wstrzymanie")
    room.run("zmien_stan_wykonawcy", **state_fields("wstrzymanie", "wstrzymany"))
    room.refused("cennik przypisuje się zatwierdzonemu wykonawcy", "przypisz_cennik", **assignment)


# R9 and replacement -------------------------------------------------------------------------------


def first_row_copy(room, table):
    """SQL that inserts again the first row of the table, as a careless upsert would."""
    columns = ", ".join(row["name"] for row in room.execute(f"PRAGMA table_info({table})"))

    return (
        f"INTO {table} ({columns}) SELECT {columns} FROM {table} "
        f"WHERE rowid = (SELECT min(rowid) FROM {table})"
    )


@scenario(
    "R9: wierszy rejestru wykonawców się nie usuwa ani nie zastępuje przez REPLACE albo upsert"
)
def no_deletion_or_replacement():
    room = new_room()
    full_register(room)

    tables = [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'wykonawcy\\_%' ESCAPE '\\'"
        )
    ]
    assert len(tables) == 11, tables

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

    # The version number is a UNIQUE key of its own: a copy under a new key replaces by it.
    room.refused_sql(
        REPLACEMENT,
        """INSERT OR REPLACE INTO wykonawcy_cenniki_wersje (id, cennik_id, numer, obowiazuje_od,
          wpisano, stan, decyzja_id)
        SELECT 'nowy-klucz', cennik_id, numer, obowiazuje_od, wpisano, stan, decyzja_id
        FROM wykonawcy_cenniki_wersje
        WHERE id = 'wersja-1'""",
    )
