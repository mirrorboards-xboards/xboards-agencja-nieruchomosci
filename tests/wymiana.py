"""Scenarios of the exchange room of an office and a notary's office (spec/cases/wymiana).

The exchange room has no core of a case file, so its room is `Room(WYMIANA)`. Each scenario
starts from an empty room and checks one rule the database keeps: a write the rule allows and a
write it refuses. The first scenario follows a whole sale, from the opening of the room to its
closing, as the profile of the room describes it.
"""

from pathlib import Path

from room import Room

WYMIANA = Path(__file__).resolve().parents[1] / "spec" / "cases" / "wymiana"

SCENARIOS = []


def scenario(title):
    def register(function):
        SCENARIOS.append((title, function))

        return function

    return register


def new_room():
    return Room(WYMIANA)


# Building blocks of an exchange --------------------------------------------------------------------

OFFICE = {
    "strona": "biuro",
    "autor": "tozsamosc-koordynatora",
}
NOTARY = {
    "strona": "kancelaria",
    "autor": "tozsamosc-sekretarza",
}
FILE = {
    "sciezka": "pliki/dokument.pdf",
    "skrot": "sha256-dokumentu",
}


def open_exchange(room):
    """The office opens the room and names the parties: it represents the seller and the buyer."""
    room.run(
        "otworz_pokoj_wymiany",
        oznaczenie="Transakcja 2026/017, ul. Polna 7 m. 12",
        transakcja_odnosnik="pokoj-transakcji-017",
        czynnosc="przenoszaca",
        kancelaria_nazwa="Kancelaria Notarialna przy ul. Długiej",
        kancelaria_adres="ul. Długa 3, Kraków",
        notariusz="Jan Nowak",
        nieruchomosc_adres="ul. Polna 7 m. 12, Kraków",
        numer_kw="KR1P/00012345/6",
        otwarto="2026-11-02T09:00:00Z",
    )
    room.run(
        "wpisz_strone_aktu",
        id="sprzedajaca",
        nazwa="Anna Kowalska",
        rola="sprzedajacy",
        reprezentowana_przez_biuro=1,
    )
    room.run(
        "wpisz_strone_aktu",
        id="kupujacy",
        nazwa="Piotr Zieliński",
        rola="kupujacy",
        reprezentowana_przez_biuro=1,
    )
    room.run(
        "wpisz_strone_aktu",
        id="malzonka-kupujacego",
        nazwa="Ewa Zielińska",
        rola="kupujacy",
        reprezentowana_przez_biuro=0,
    )


def give_power(room, wpis_id, strona_id):
    room.run(
        "przekaz_pelnomocnictwo",
        wpis_id=wpis_id,
        **OFFICE,
        chwila="2026-11-02T10:00:00Z",
        **FILE,
        strona_id=strona_id,
        wystawca="Anna Kowalska",
        data_wystawienia="2026-11-01",
        zakres="uzgadnianie terminu aktu i przekazywanie dokumentów notariuszowi",
    )


def give_list(room, wpis_id="lista", poprawia_id=None):
    room.run(
        "podaj_liste_dokumentow",
        id=wpis_id,
        **NOTARY,
        chwila="2026-11-03T09:00:00Z",
        poprawia_id=poprawia_id,
    )
    room.run(
        "dopisz_pozycje_listy",
        wpis_id=wpis_id,
        pozycja_id="odpis-kw",
        nazwa="Odpis zupełny księgi wieczystej",
        postac="odpis",
        swiezosc_dni=30,
    )
    room.run(
        "dopisz_pozycje_listy",
        wpis_id=wpis_id,
        pozycja_id="zaswiadczenie-spoldzielni",
        nazwa="Zaświadczenie spółdzielni o braku zaległości",
        od_strony_id="sprzedajaca",
        postac="oryginal",
        swiezosc_dni=30,
        do_kiedy="2026-11-20",
    )


def document_fields(**fields):
    return {
        "wpis_id": "zaswiadczenie",
        **OFFICE,
        "chwila": "2026-11-04T09:00:00Z",
        **FILE,
        "pozycja_id": "zaswiadczenie-spoldzielni",
        "strona_id": "sprzedajaca",
        "wystawca": "Spółdzielnia Mieszkaniowa Polna",
        "data_wystawienia": "2026-11-03",
        "waznosc_do": "2026-12-03",
        "oryginal_w_dniu_aktu": 1,
        "kto_przyniesie": "Anna Kowalska",
        **fields,
    }


def give_draft(room, wpis_id, numer_wersji, ostateczna=0):
    room.run(
        "przekaz_projekt_aktu",
        wpis_id=wpis_id,
        **NOTARY,
        chwila="2026-11-10T09:00:00Z",
        sciezka=f"pliki/projekt-{numer_wersji}.pdf",
        skrot=f"sha256-projektu-{numer_wersji}",
        numer_wersji=numer_wersji,
        ostateczna=ostateczna,
    )


def person_fields(**fields):
    return {
        "wpis_id": "dane-sprzedajacej",
        **OFFICE,
        "chwila": "2026-11-04T10:00:00Z",
        "strona_id": "sprzedajaca",
        "udzial": "strona",
        "rodzaj_podmiotu": "osoba_fizyczna",
        "imiona": "Anna Maria",
        "nazwisko": "Kowalska",
        "imiona_rodzicow": "Jan, Maria",
        "miejsce_zamieszkania": "Kraków",
        "stan_cywilny": "rozwiedziona",
        "zrodlo_dokument_odnosnik": "pokoj-transakcji-017/dowod-sprzedajacej",
        **fields,
    }


def propose_term(room, termin_id, wpis_id, side=NOTARY):
    room.run(
        "zaproponuj_termin_aktu",
        wpis_id=wpis_id,
        **side,
        chwila="2026-11-11T09:00:00Z",
        id=termin_id,
        dzien="2026-11-25",
        godzina="10:30",
        miejsce="ul. Długa 3, Kraków",
        czynnosc="przenoszaca",
    )


def announce(room, wpis_id, termin_id, stan, side, decyzja_odnosnik=None):
    room.run(
        "oglos_stan_terminu",
        wpis_id=wpis_id,
        **side,
        chwila="2026-11-12T09:00:00Z",
        termin_id=termin_id,
        stan=stan,
        decyzja_odnosnik=decyzja_odnosnik,
    )


def confirm_by_both(room, termin_id):
    announce(
        room,
        f"{termin_id}-biuro",
        termin_id,
        "potwierdzony",
        OFFICE,
        "pokoj-transakcji-017/decyzja-termin",
    )
    announce(room, f"{termin_id}-kancelaria", termin_id, "potwierdzony", NOTARY)


def close(room, side):
    room.run("zamknij_pokoj_wymiany", strona_pokoju=side, chwila="2026-12-01T09:00:00Z")


def reopen(room, side):
    room.run("otworz_ponownie_pokoj_wymiany", strona_pokoju=side, chwila="2027-03-01T09:00:00Z")


def state(room):
    return room.run("stan_pokoju")[0]["stan"]


def full_exchange(room):
    """A sale at a notary's office, written into every table of the room."""
    open_exchange(room)
    room.run("podaj_sygnature_kancelarii", sygnatura_kancelarii="KN 1234/2026")

    # The office: powers of attorney, the documents from the notary's list and the data of the act.
    give_power(room, "pelnomocnictwo-sprzedajacej", "sprzedajaca")
    give_power(room, "pelnomocnictwo-kupujacego", "kupujacy")
    give_list(room)
    room.run("przekaz_dokument_z_listy", **document_fields())
    room.run(
        "przekaz_zgode_stron",
        wpis_id="zgoda-na-projekt",
        **OFFICE,
        chwila="2026-11-04T09:30:00Z",
        **FILE,
        strona_id="kupujacy",
        data_wystawienia="2026-11-04",
        zakres="biuro dostaje projekt aktu i informację po akcie",
    )
    room.run("przekaz_dane_osoby", **person_fields())
    room.run(
        "przekaz_dane_nieruchomosci",
        wpis_id="dane-lokalu",
        **OFFICE,
        chwila="2026-11-04T10:30:00Z",
        strona_id="sprzedajaca",
        numer_kw="KR1P/00012345/6",
        oznaczenie="lokal mieszkalny nr 12 w budynku przy ul. Polnej 7, pow. 54,20 m²",
        zrodlo="odpis zupełny KW z 2026-11-03",
    )
    room.run(
        "przekaz_kredyt",
        wpis_id="kredyt",
        **OFFICE,
        chwila="2026-11-04T11:00:00Z",
        strona_id="kupujacy",
        bank="Bank Polski S.A.",
        kwota=45_000_000,
        warunki_wyplaty="po przedłożeniu wypisu aktu i wniosku do księgi",
    )
    room.run(
        "podaj_rachunek",
        wpis_id="rachunek-ceny",
        **OFFICE,
        chwila="2026-11-05T09:00:00Z",
        numer="PL61109010140000071219812874",
        posiadacz="Anna Kowalska",
        cel="zaplata_ceny",
        decyzja_odnosnik="pokoj-transakcji-017/decyzja-rachunek",
    )

    # The notary's office: costs, the draft and the comments of the parties on it.
    room.run(
        "podaj_koszty_aktu",
        wpis_id="koszty",
        **NOTARY,
        chwila="2026-11-06T09:00:00Z",
        tresc="Taksa 1 010 zł netto, PCC 2%, opłata sądowa 200 zł.",
    )
    give_draft(room, "projekt-1", 1)
    room.run(
        "przekaz_uwagi_do_projektu",
        wpis_id="uwagi-1",
        **OFFICE,
        chwila="2026-11-11T08:00:00Z",
        tresc="Kupujący prosi o wydanie lokalu do 2026-12-15.",
        projekt_id="projekt-1",
        decyzja_odnosnik="pokoj-transakcji-017/decyzja-uwagi",
    )
    give_draft(room, "projekt-2", 2, ostateczna=1)

    # The date of the act, confirmed by both sides; the act, the pre-emption and after the act.
    propose_term(room, "termin", "propozycja")
    confirm_by_both(room, "termin")
    room.run(
        "podaj_pierwokup",
        wpis_id="pierwokup",
        **NOTARY,
        chwila="2026-11-20T09:00:00Z",
        uprawniony="Gmina Miejska Kraków",
        zawiadomienie_doreczono="2026-10-20",
        wynik="termin_uplynal",
        data="2026-11-20",
    )
    announce(room, "odbyl-sie", "termin", "odbyl_sie", NOTARY)
    room.run(
        "podaj_informacje_po_akcie",
        wpis_id="po-akcie",
        **NOTARY,
        chwila="2026-11-25T14:00:00Z",
        akt_sporzadzono="2026-11-25",
        wniosek_kw_zlozono="2026-11-25",
    )
    room.run(
        "wpisz_wiadomosc",
        id="pytanie",
        **OFFICE,
        chwila="2026-11-26T09:00:00Z",
        odpowiada_na_id="po-akcie",
        tresc="Czy kancelaria może podać numer wniosku do księgi?",
    )
    close(room, "biuro")


# The whole exchange --------------------------------------------------------------------------------


@scenario("Przebieg wymiany: od otwarcia przez dokumenty, projekt i termin aktu do zamknięcia")
def whole_exchange():
    room = new_room()

    full_exchange(room)
    assert state(room) == "otwarty"
    close(room, "kancelaria")

    assert state(room) == "zamkniety"
    assert room.run("metryka")[0]["sygnatura_kancelarii"] == "KN 1234/2026"
    assert [row["pozycja_id"] for row in room.run("biezaca_lista_dokumentow")] == [
        "odpis-kw",
        "zaswiadczenie-spoldzielni",
    ]
    assert [row["numer_wersji"] for row in room.run("wersje_projektu_aktu")] == [2, 1]
    assert room.run("uwagi_do_projektu")[0]["numer_wersji"] == 1
    assert room.run("terminy_aktu")[0]["stan"] == "odbyl_sie"
    assert room.run("informacje_po_akcie")[0]["wniosek_kw_zlozono"] == "2026-11-25"
    assert len(room.run("wpisy")) == 20


# Metryka -------------------------------------------------------------------------------------------


@scenario("Metryki pokoju się nie zmienia; kancelaria raz podaje swoją sygnaturę")
def record_fixed():
    room = new_room()
    open_exchange(room)
    rule = "metryki pokoju się nie zmienia"

    room.refused_sql(rule, "UPDATE wymiana_metryka_wpis SET notariusz = 'Inny Notariusz'")
    room.run("podaj_sygnature_kancelarii", sygnatura_kancelarii="KN 1234/2026")
    room.refused(rule, "podaj_sygnature_kancelarii", sygnatura_kancelarii="KN 9999/2026")
    assert room.run("metryka")[0]["sygnatura_kancelarii"] == "KN 1234/2026"


@scenario("Pokój zamyka się zgodą obu stron i dopiero zgodą obu otwiera się ponownie")
def closing_by_both_sides():
    room = new_room()
    room.refused(
        "pokój bez metryki nie ma czego zamykać",
        "zamknij_pokoj_wymiany",
        strona_pokoju="biuro",
        chwila="2026-12-01",
    )
    open_exchange(room)

    close(room, "biuro")
    assert state(room) == "otwarty"
    room.refused("na przemian", "zamknij_pokoj_wymiany", strona_pokoju="biuro", chwila="2026-12-01")
    room.refused(
        "ponownie otwiera tylko zamknięty",
        "otworz_ponownie_pokoj_wymiany",
        strona_pokoju="kancelaria",
        chwila="2026-12-01",
    )

    close(room, "kancelaria")
    assert state(room) == "zamkniety"

    reopen(room, "kancelaria")
    assert state(room) == "zamkniety"
    room.refused(
        "zamyka tylko otwarty",
        "zamknij_pokoj_wymiany",
        strona_pokoju="kancelaria",
        chwila="2027-03-01",
    )

    reopen(room, "biuro")
    assert state(room) == "otwarty"
    assert [row["rodzaj"] for row in room.run("zamkniecia")] == [
        "zamkniecie",
        "zamkniecie",
        "ponowne_otwarcie",
        "ponowne_otwarcie",
    ]


# Entries -------------------------------------------------------------------------------------------


def message(**fields):
    return {
        "id": "wiadomosc",
        **OFFICE,
        "chwila": "2026-11-03T09:00:00Z",
        "tresc": "Czy notariusz przyjmie zaświadczenie wystawione 20 dni przed aktem?",
        **fields,
    }


@scenario("Wpis powstaje w pokoju z metryką i nie powstaje w zamkniętym pokoju")
def entries_in_open_room():
    room = new_room()
    room.refused("wpis powstaje dopiero w pokoju z metryką", "wpisz_wiadomosc", **message())
    open_exchange(room)

    room.run("wpisz_wiadomosc", **message())
    close(room, "biuro")
    close(room, "kancelaria")
    room.refused(
        "w zamkniętym pokoju nie powstaje nowy wpis",
        "wpisz_wiadomosc",
        **message(id="po-zamknieciu"),
    )

    reopen(room, "biuro")
    room.refused(
        "w zamkniętym pokoju nie powstaje nowy wpis",
        "wpisz_wiadomosc",
        **message(id="po-zamknieciu"),
    )
    reopen(room, "kancelaria")
    room.run("wpisz_wiadomosc", **message(id="po-otwarciu"))


@scenario("Lista, projekt, koszty, pierwokup i informacja po akcie pochodzą tylko od kancelarii")
def entries_only_from_notary():
    room = new_room()
    open_exchange(room)

    room.refused(
        "wpis_tylko_od_kancelarii",
        "podaj_liste_dokumentow",
        id="lista",
        **OFFICE,
        chwila="2026-11-03",
    )
    room.refused(
        "wpis_tylko_od_kancelarii",
        "podaj_koszty_aktu",
        wpis_id="koszty",
        **OFFICE,
        chwila="2026-11-03",
        tresc="Taksa",
    )
    room.refused(
        "wpis_tylko_od_kancelarii",
        "przekaz_projekt_aktu",
        wpis_id="projekt",
        **OFFICE,
        chwila="2026-11-03",
        **FILE,
        numer_wersji=1,
        ostateczna=0,
    )
    room.refused(
        "wpis_tylko_od_kancelarii",
        "podaj_pierwokup",
        wpis_id="pierwokup",
        **OFFICE,
        chwila="2026-11-03",
        uprawniony="Gmina",
    )
    room.refused(
        "wpis_tylko_od_kancelarii",
        "podaj_informacje_po_akcie",
        wpis_id="po-akcie",
        **OFFICE,
        chwila="2026-11-03",
        akt_sporzadzono="2026-11-25",
    )
    give_list(room)
    give_draft(room, "projekt", 1)


@scenario("Dokumenty, pełnomocnictwa, zgody, uwagi i dane stron pochodzą tylko od biura")
def entries_only_from_office():
    room = new_room()
    open_exchange(room)
    give_power(room, "pelnomocnictwo", "sprzedajaca")
    give_list(room)
    give_draft(room, "projekt", 1)

    room.refused(
        "wpis_tylko_od_biura",
        "przekaz_pelnomocnictwo",
        wpis_id="pelnomocnictwo-kancelarii",
        **NOTARY,
        chwila="2026-11-03",
        **FILE,
        strona_id="sprzedajaca",
        zakres="wszystko",
    )
    room.refused("wpis_tylko_od_biura", "przekaz_dokument_z_listy", **document_fields(**NOTARY))
    room.refused("wpis_tylko_od_biura", "przekaz_dane_osoby", **person_fields(**NOTARY))
    room.refused(
        "wpis_tylko_od_biura",
        "przekaz_uwagi_do_projektu",
        wpis_id="uwagi",
        **NOTARY,
        chwila="2026-11-11",
        projekt_id="projekt",
        decyzja_odnosnik="decyzja",
    )
    room.run("przekaz_dokument_z_listy", **document_fields())
    room.run("przekaz_dane_osoby", **person_fields())


@scenario("Plik ma skrót, dokument ma plik, a wiadomość ma treść")
def entry_file_and_content():
    room = new_room()
    open_exchange(room)

    room.refused("plik_ze_skrotem", "wpisz_wiadomosc", **message(sciezka="pliki/zdjecie.jpg"))
    room.refused("wiadomosc_z_trescia", "wpisz_wiadomosc", **message(tresc=None))
    room.refused(
        "dokument_z_plikiem",
        "przekaz_pelnomocnictwo",
        wpis_id="pelnomocnictwo",
        **OFFICE,
        chwila="2026-11-02",
        strona_id="sprzedajaca",
        zakres="przekazywanie dokumentów",
    )
    room.run("wpisz_wiadomosc", **message(sciezka="pliki/zdjecie.jpg", skrot="sha256-zdjecia"))


@scenario("Poprawka wskazuje wcześniejszy wpis tego samego rodzaju i tej samej strony")
def correction_same_kind():
    room = new_room()
    open_exchange(room)
    give_power(room, "pelnomocnictwo", "sprzedajaca")
    room.run("przekaz_dane_osoby", **person_fields())
    room.run("wpisz_wiadomosc", **message(id="wiadomosc-kancelarii", **NOTARY))
    rule = "poprawka wskazuje wpis tego samego rodzaju i tej samej strony"

    room.refused(
        rule,
        "przekaz_dane_osoby",
        **person_fields(wpis_id="poprawka", poprawia_id="pelnomocnictwo"),
    )
    room.refused(
        rule, "wpisz_wiadomosc", **message(id="poprawka", poprawia_id="wiadomosc-kancelarii")
    )
    room.refused(
        "odpowiedz_nie_wskazuje_siebie",
        "wpisz_wiadomosc",
        **message(id="sama", odpowiada_na_id="sama"),
    )

    room.run(
        "przekaz_dane_osoby",
        **person_fields(
            wpis_id="poprawka", poprawia_id="dane-sprzedajacej", miejsce_zamieszkania="Wieliczka"
        ),
    )
    assert [row["miejsce_zamieszkania"] for row in room.run("dane_osob")] == ["Kraków", "Wieliczka"]


@scenario("Szczegół dopisuje się tylko razem ze swoim wpisem i do wpisu swojego rodzaju")
def detail_with_its_entry():
    room = new_room()
    open_exchange(room)
    give_list(room)
    room.run("wpisz_wiadomosc", **message())

    room.refused(
        "pozycja listy dokumentów należy do najnowszego wpisu lista_dokumentow",
        "dopisz_pozycje_listy",
        wpis_id="lista",
        pozycja_id="wypis-z-rejestru",
        nazwa="Wypis z rejestru gruntów",
        postac="oryginal",
    )
    room.refused(
        "pozycja listy dokumentów należy do najnowszego wpisu lista_dokumentow",
        "dopisz_pozycje_listy",
        wpis_id="wiadomosc",
        pozycja_id="wypis-z-rejestru",
        nazwa="Wypis z rejestru gruntów",
        postac="oryginal",
    )
    room.refused_sql(
        "rachunek należy do najnowszego wpisu rachunek",
        "INSERT INTO wymiana_dane_rachunki (wpis_id, numer, posiadacz, cel, decyzja_odnosnik) VALUES ('wiadomosc', 'PL1', 'Anna', 'inny', 'decyzja')",
    )
    room.refused_sql(
        "dokument należy do najnowszego wpisu tego samego rodzaju",
        "INSERT INTO wymiana_dokumenty_wpisy (wpis_id, rodzaj) VALUES ('wiadomosc', 'koszty')",
    )


# Documents -----------------------------------------------------------------------------------------


@scenario("Dokument biura wypełnia pozycję z listy kancelarii")
def document_from_list():
    room = new_room()
    open_exchange(room)
    give_power(room, "pelnomocnictwo", "sprzedajaca")

    room.refused(
        "dokument wypełnia pozycję z listy dokumentów kancelarii",
        "przekaz_dokument_z_listy",
        **document_fields(),
    )
    give_list(room)
    room.refused(
        "dokument wypełnia pozycję z listy dokumentów kancelarii",
        "przekaz_dokument_z_listy",
        **document_fields(pozycja_id="akt-notarialny-nabycia"),
    )
    room.run("przekaz_dokument_z_listy", **document_fields())


@scenario("Pisanie bez pełnomocnictwa: dokument, zgoda i dane strony wymagają jej pełnomocnictwa")
def writing_needs_power_of_attorney():
    room = new_room()
    open_exchange(room)
    give_list(room)
    document_rule = "dokument biura dla strony wymaga wpisanego wcześniej pełnomocnictwa tej strony"
    data_rule = "dane strony wymagają wpisanego wcześniej pełnomocnictwa tej strony"
    consent = {
        "wpis_id": "zgoda",
        **OFFICE,
        "chwila": "2026-11-04",
        **FILE,
        "strona_id": "sprzedajaca",
        "zakres": "biuro dostaje projekt aktu",
    }
    land = {
        "wpis_id": "dane-lokalu",
        **OFFICE,
        "chwila": "2026-11-04",
        "strona_id": "sprzedajaca",
        "oznaczenie": "lokal nr 12",
        "zrodlo": "odpis KW",
    }
    credit = {
        "wpis_id": "kredyt",
        **OFFICE,
        "chwila": "2026-11-04",
        "strona_id": "kupujacy",
        "bank": "Bank Polski S.A.",
        "kwota": 45_000_000,
    }

    room.refused(document_rule, "przekaz_dokument_z_listy", **document_fields())
    room.refused(document_rule, "przekaz_zgode_stron", **consent)
    room.refused(data_rule, "przekaz_dane_osoby", **person_fields())
    room.refused(data_rule, "przekaz_dane_nieruchomosci", **land)
    room.refused(data_rule, "przekaz_kredyt", **credit)

    give_power(room, "pelnomocnictwo-sprzedajacej", "sprzedajaca")
    room.run("przekaz_dokument_z_listy", **document_fields())
    room.run("przekaz_zgode_stron", **consent)
    room.run("przekaz_dane_osoby", **person_fields())
    room.run("przekaz_dane_nieruchomosci", **land)
    room.refused(data_rule, "przekaz_kredyt", **credit)

    give_power(room, "pelnomocnictwo-kupujacego", "kupujacy")
    room.run("przekaz_kredyt", **credit)


@scenario("Pełnomocnictwo daje strona, którą biuro reprezentuje według metryki")
def power_of_represented_party():
    room = new_room()
    open_exchange(room)

    room.refused(
        "pełnomocnictwo dotyczy strony, którą biuro reprezentuje według metryki",
        "przekaz_pelnomocnictwo",
        wpis_id="pelnomocnictwo",
        **OFFICE,
        chwila="2026-11-02",
        **FILE,
        strona_id="malzonka-kupujacego",
        zakres="przekazywanie dokumentów",
    )
    give_power(room, "pelnomocnictwo", "kupujacy")


@scenario("Dokument ma wystawcę i oryginał, pełnomocnictwo zakres, a projekt numer wersji")
def document_fields_by_kind():
    room = new_room()
    open_exchange(room)
    give_power(room, "pelnomocnictwo", "sprzedajaca")
    give_list(room)

    room.refused(
        "dokument_ma_wystawce_i_oryginal",
        "przekaz_dokument_z_listy",
        **document_fields(wystawca=None),
    )
    room.refused(
        "oryginal_ma_kto_przyniesie",
        "przekaz_dokument_z_listy",
        **document_fields(kto_przyniesie=None),
    )
    room.refused(
        "oryginal_ma_kto_przyniesie",
        "przekaz_dokument_z_listy",
        **document_fields(oryginal_w_dniu_aktu=0),
    )
    room.refused(
        "pelnomocnictwo_i_zgoda_z_zakresem",
        "przekaz_pelnomocnictwo",
        wpis_id="bez-zakresu",
        **OFFICE,
        chwila="2026-11-02",
        **FILE,
        strona_id="sprzedajaca",
    )
    room.refused(
        "projekt_ma_wersje",
        "przekaz_projekt_aktu",
        wpis_id="projekt",
        **NOTARY,
        chwila="2026-11-10",
        **FILE,
        numer_wersji=1,
    )
    room.run(
        "przekaz_dokument_z_listy", **document_fields(oryginal_w_dniu_aktu=0, kto_przyniesie=None)
    )


@scenario("Numer wersji projektu aktu rośnie; nowa wersja staje obok starej")
def draft_versions_grow():
    room = new_room()
    open_exchange(room)
    give_draft(room, "projekt-1", 1)
    give_draft(room, "projekt-2", 2)

    room.refused(
        "numer wersji projektu aktu rośnie", "przekaz_projekt_aktu", **draft_fields("projekt-2b", 2)
    )
    room.refused(
        "numer wersji projektu aktu rośnie", "przekaz_projekt_aktu", **draft_fields("projekt-1b", 1)
    )
    give_draft(room, "projekt-3", 3, ostateczna=1)
    assert [row["ostateczna"] for row in room.run("wersje_projektu_aktu")] == [1, 0, 0]


def draft_fields(wpis_id, numer_wersji):
    return {
        "wpis_id": wpis_id,
        **NOTARY,
        "chwila": "2026-11-10",
        **FILE,
        "numer_wersji": numer_wersji,
        "ostateczna": 0,
    }


@scenario("Uwagi stron wskazują wersję projektu i decyzję z teczki transakcji")
def comments_on_draft():
    room = new_room()
    open_exchange(room)
    give_power(room, "pelnomocnictwo", "kupujacy")
    give_draft(room, "projekt-1", 1)
    comments = {
        "wpis_id": "uwagi",
        **OFFICE,
        "chwila": "2026-11-11",
        "tresc": "Kupujący prosi o wydanie lokalu do 2026-12-15.",
        "projekt_id": "projekt-1",
        "decyzja_odnosnik": "pokoj-transakcji-017/decyzja-uwagi",
    }

    room.refused(
        "uwagi wskazują wersję projektu aktu",
        "przekaz_uwagi_do_projektu",
        **{**comments, "projekt_id": "pelnomocnictwo"},
    )
    room.refused(
        "wymiana_dokumenty_uwagi.decyzja_odnosnik",
        "przekaz_uwagi_do_projektu",
        **{**comments, "decyzja_odnosnik": None},
    )
    room.run("przekaz_uwagi_do_projektu", **comments)
    assert room.run("wpis", id="uwagi")[0]["odpowiada_na_id"] == "projekt-1"


# Data of the act -----------------------------------------------------------------------------------


@scenario("Dane osoby z art. 92 § 1 pkt 4 PoN: imiona rodziców strony, nazwa i siedziba podmiotu")
def person_data_of_the_act():
    room = new_room()
    open_exchange(room)
    give_power(room, "pelnomocnictwo", "sprzedajaca")

    room.refused(
        "strona_fizyczna_ma_imiona_rodzicow",
        "przekaz_dane_osoby",
        **person_fields(imiona_rodzicow=None),
    )
    room.refused(
        "strona_fizyczna_ma_imiona_rodzicow",
        "przekaz_dane_osoby",
        **person_fields(wpis_id="pelnomocnik", udzial="pelnomocnik"),
    )
    room.refused(
        "osoba_ma_imiona_nazwisko_i_zamieszkanie",
        "przekaz_dane_osoby",
        **person_fields(miejsce_zamieszkania=None),
    )
    company = person_fields(
        wpis_id="spolka",
        rodzaj_podmiotu="podmiot",
        imiona=None,
        nazwisko=None,
        imiona_rodzicow=None,
        miejsce_zamieszkania=None,
        stan_cywilny=None,
        nazwa="Polna Development sp. z o.o.",
        siedziba="Kraków",
    )
    room.refused(
        "podmiot_ma_nazwe_i_siedzibe", "przekaz_dane_osoby", **{**company, "siedziba": None}
    )
    room.refused(
        "podmiot_bez_danych_osoby", "przekaz_dane_osoby", **{**company, "stan_cywilny": "wolny"}
    )
    room.refused(
        "podmiot_tylko_strona", "przekaz_dane_osoby", **{**company, "udzial": "pelnomocnik"}
    )
    room.refused(
        "wymiana_dane_osoby.zrodlo_dokument_odnosnik",
        "przekaz_dane_osoby",
        **person_fields(zrodlo_dokument_odnosnik=None),
    )

    room.run("przekaz_dane_osoby", **person_fields())
    room.run("przekaz_dane_osoby", **company)
    room.run(
        "przekaz_dane_osoby",
        **person_fields(
            wpis_id="pelnomocnik",
            udzial="pelnomocnik",
            imiona="Marek",
            nazwisko="Wiśniewski",
            imiona_rodzicow=None,
            stan_cywilny=None,
        ),
    )


@scenario(
    "Rachunek od biura wskazuje decyzję, a rachunek od kancelarii jest informacją bez decyzji"
)
def accounts_by_side():
    room = new_room()
    open_exchange(room)
    rule = "rachunek od biura ma odnośnik do decyzji"
    account = {
        "chwila": "2026-11-05",
        "numer": "PL61109010140000071219812874",
        "posiadacz": "Anna Kowalska",
        "cel": "zaplata_ceny",
    }

    room.refused(rule, "podaj_rachunek", wpis_id="rachunek-biura", **OFFICE, **account)
    room.refused(
        rule,
        "podaj_rachunek",
        wpis_id="rachunek-kancelarii",
        **NOTARY,
        **account,
        decyzja_odnosnik="pokoj-transakcji-017/decyzja-rachunek",
    )
    room.run(
        "podaj_rachunek",
        wpis_id="rachunek-biura",
        **OFFICE,
        **account,
        decyzja_odnosnik="pokoj-transakcji-017/decyzja-rachunek",
    )
    room.run(
        "podaj_rachunek",
        wpis_id="rachunek-podatku",
        **NOTARY,
        **{**account, "cel": "podatki_i_oplaty", "posiadacz": "Urząd Skarbowy Kraków"},
    )
    assert [row["strona"] for row in room.run("rachunki_do_aktu")] == ["biuro", "kancelaria"]


# The date of the act -------------------------------------------------------------------------------


@scenario("Termin jest potwierdzony dopiero wtedy, gdy potwierdzą go obie strony; biuro z decyzją")
def term_confirmed_by_both():
    room = new_room()
    open_exchange(room)
    propose_term(room, "termin", "propozycja")
    rule = "potwierdzenie terminu przez biuro wskazuje decyzję"

    room.refused(
        rule,
        "oglos_stan_terminu",
        wpis_id="bez-decyzji",
        **OFFICE,
        chwila="2026-11-12",
        termin_id="termin",
        stan="potwierdzony",
    )
    room.refused(
        rule,
        "oglos_stan_terminu",
        wpis_id="z-decyzja",
        **NOTARY,
        chwila="2026-11-12",
        termin_id="termin",
        stan="potwierdzony",
        decyzja_odnosnik="pokoj-transakcji-017/decyzja-termin",
    )

    announce(room, "kancelaria", "termin", "potwierdzony", NOTARY)
    term = room.run("terminy_aktu")[0]
    assert (term["stan"], term["potwierdzila_kancelaria"], term["potwierdzila_biuro"]) == (
        "proponowany",
        1,
        0,
    )

    announce(room, "biuro", "termin", "potwierdzony", OFFICE, "pokoj-transakcji-017/decyzja-termin")
    assert room.run("terminy_aktu")[0]["stan"] == "potwierdzony"


@scenario(
    "Akt odbywa się w potwierdzonym terminie, a ogłasza to kancelaria; zakończony termin stoi"
)
def term_held_and_finished():
    room = new_room()
    open_exchange(room)
    propose_term(room, "termin", "propozycja", OFFICE)
    held = {
        "chwila": "2026-11-25",
        "termin_id": "termin",
        "stan": "odbyl_sie",
    }

    room.refused(
        "odbywa się tylko termin potwierdzony przez obie strony",
        "oglos_stan_terminu",
        wpis_id="za-wczesnie",
        **NOTARY,
        **held,
    )
    confirm_by_both(room, "termin")
    room.refused(
        "że akt się odbył, ogłasza kancelaria",
        "oglos_stan_terminu",
        wpis_id="od-biura",
        **OFFICE,
        **held,
    )
    room.run("oglos_stan_terminu", wpis_id="odbyl-sie", **NOTARY, **held)

    room.refused(
        "odwołany, przesunięty albo odbyty termin nie ma kolejnych stanów",
        "oglos_stan_terminu",
        wpis_id="odwolanie",
        **NOTARY,
        chwila="2026-11-26",
        termin_id="termin",
        stan="odwolany",
    )


@scenario("Przesunięty termin zastępuje nowa propozycja; przesuniętego się nie potwierdza")
def term_moved():
    room = new_room()
    open_exchange(room)
    propose_term(room, "termin-1", "propozycja-1")
    announce(room, "przesuniecie", "termin-1", "przesuniety", NOTARY)

    room.refused(
        "odwołany, przesunięty albo odbyty termin nie ma kolejnych stanów",
        "oglos_stan_terminu",
        wpis_id="potwierdzenie",
        **NOTARY,
        chwila="2026-11-13",
        termin_id="termin-1",
        stan="potwierdzony",
    )
    propose_term(room, "termin-2", "propozycja-2")
    confirm_by_both(room, "termin-2")
    assert [row["stan"] for row in room.run("terminy_aktu")] == ["przesuniety", "potwierdzony"]


@scenario("Pierwokup: wynik ma datę i przychodzi po doręczeniu zawiadomienia")
def pre_emption():
    room = new_room()
    open_exchange(room)
    fields = {
        "wpis_id": "pierwokup",
        **NOTARY,
        "chwila": "2026-10-21",
        "uprawniony": "Gmina Miejska Kraków",
    }

    room.refused(
        "wynik_z_data",
        "podaj_pierwokup",
        **fields,
        zawiadomienie_doreczono="2026-10-20",
        wynik="nie_wykonal",
    )
    room.refused(
        "wynik_po_zawiadomieniu",
        "podaj_pierwokup",
        **fields,
        wynik="nie_wykonal",
        data="2026-10-25",
    )
    room.run("podaj_pierwokup", **fields, zawiadomienie_doreczono="2026-10-20")
    room.run(
        "podaj_pierwokup",
        **{**fields, "wpis_id": "wynik", "chwila": "2026-11-20"},
        poprawia_id="pierwokup",
        zawiadomienie_doreczono="2026-10-20",
        wynik="termin_uplynal",
        data="2026-11-20",
    )


@scenario("Informacja po akcie ma co najmniej jedną datę")
def after_the_act():
    room = new_room()
    open_exchange(room)

    room.refused(
        "po_akcie_z_data",
        "podaj_informacje_po_akcie",
        wpis_id="po-akcie",
        **NOTARY,
        chwila="2026-11-25",
    )
    room.run(
        "podaj_informacje_po_akcie",
        wpis_id="po-akcie",
        **NOTARY,
        chwila="2026-11-25",
        akt_sporzadzono="2026-11-25",
    )


# R9 and replacement --------------------------------------------------------------------------------


def exchange_tables(room):
    return [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'wymiana\\_%' ESCAPE '\\'"
        )
    ]


@scenario("R9: wierszy pokoju wymiany się nie usuwa ani nie zmienia")
def no_deletion_or_change():
    room = new_room()
    full_exchange(room)
    tables = exchange_tables(room)
    assert len(tables) == 15, tables

    for table in tables:
        assert room.value(f"SELECT count(*) FROM {table}") > 0, f"{table} has no row"
        first_row = f"rowid = (SELECT min(rowid) FROM {table})"

        room.refused_sql(
            "wierszy pokoju wymiany się nie usuwa (R9)", f"DELETE FROM {table} WHERE {first_row}"
        )

        first_column = room.execute(f"PRAGMA table_info({table})")[0]["name"]
        refusal = (
            "metryki pokoju się nie zmienia"
            if table == "wymiana_metryka_wpis"
            else "wiersza pokoju wymiany się nie zmienia"
        )
        room.refused_sql(
            refusal, f"UPDATE {table} SET {first_column} = {first_column} WHERE {first_row}"
        )


@scenario("Wiersza pokoju wymiany nie zastępuje INSERT OR REPLACE ani upsert")
def no_replacement():
    room = new_room()
    full_exchange(room)
    refusal = "wiersza pokoju wymiany nie zastępuje się nowym wpisem"

    for table in exchange_tables(room):
        columns = ", ".join(row["name"] for row in room.execute(f"PRAGMA table_info({table})"))
        copy = f"INTO {table} ({columns}) SELECT {columns} FROM {table} WHERE rowid = (SELECT min(rowid) FROM {table})"

        # REPLACE deletes the old row without the triggers of DELETE and UPDATE, and an upsert
        # updates it without the triggers of UPDATE.
        room.refused_sql(refusal, f"INSERT OR REPLACE {copy}")
        room.refused_sql(refusal, f"INSERT {copy} ON CONFLICT DO UPDATE SET rowid = excluded.rowid")

    # A copy with a new key that collides only on UNIQUE (wpis_id) is refused too.
    room.refused_sql(
        refusal,
        "INSERT OR REPLACE INTO wymiana_akt_terminy (id, wpis_id, dzien, godzina, miejsce, czynnosc) "
        "SELECT 'inny-termin', wpis_id, dzien, godzina, miejsce, czynnosc FROM wymiana_akt_terminy",
    )
