"""Scenarios of the control register of the office (spec/registers/rejestr_kontroli).

Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
allows and a write it refuses. The first scenario follows a client from the identity check before
the first contract to the end of the business relationship and the client's return.
"""

from pathlib import Path

from room import Room

REJESTR_KONTROLI = Path(__file__).resolve().parents[1] / "spec" / "registers" / "rejestr_kontroli"

SCENARIOS = []

REPLACEMENT = "wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE"
DELETION = "wierszy rejestru się nie usuwa (R9)"
CHECK_UNCHANGED = "wpisu sprawdzenia się nie zmienia; ponowne sprawdzenie to nowy wpis"
CLIENTS = "rejestr_kontroli_klienci_wpisy"

# The tables of the checks whose re-check replaces an entry, and the column the two entries share.
RECHECKED_TABLES = {
    "rejestr_kontroli_sprawdzenia_identyfikacje": "klient_id",
    "rejestr_kontroli_sprawdzenia_weryfikacje": "identyfikacja_id",
    "rejestr_kontroli_sprawdzenia_crbr": "klient_id",
    "rejestr_kontroli_sprawdzenia_pep": "klient_id",
    "rejestr_kontroli_sprawdzenia_sankcje": "klient_id",
    "rejestr_kontroli_sprawdzenia_cele": "klient_id",
    "rejestr_kontroli_sprawdzenia_zrodla_srodkow": "klient_id",
    "rejestr_kontroli_sprawdzenia_poza_ustawa": "wiersz_odnosnik",
}


def scenario(title):
    def register(function):
        SCENARIOS.append((title, function))

        return function

    return register


def new_room():
    return Room(REJESTR_KONTROLI)


# Building blocks ----------------------------------------------------------------------------------


def ask(room, decision_id, rodzaj, funkcja="osoba_wyznaczona", dotyczy=(None, None)):
    room.run(
        "popros_o_decyzje_kontroli",
        id=decision_id,
        rodzaj=rodzaj,
        prosi="kierownik-kontroli",
        decyduje=decider_of(funkcja),
        decyduje_funkcja=funkcja,
        zlozono="2026-03-01T10:00:00Z",
        dotyczy_tabela=dotyczy[0],
        dotyczy_id=dotyczy[1],
    )


def decider_of(funkcja):
    return "wlasciciel-biura" if funkcja == "wlasciciel_biura" else "osoba-wyznaczona"


def resolve(room, decision_id, funkcja="osoba_wyznaczona", rozstrzygniecie="zgoda", **fields):
    room.run(
        "rozstrzygnij_decyzje_kontroli",
        **{
            "id": decision_id,
            "rozstrzygniecie": rozstrzygniecie,
            "podpisal": decider_of(funkcja),
            "podpis": "podpis-kluczem-xkeys",
            "podpisano": "2026-03-01T12:00:00Z",
            **fields,
        },
    )


def approved(room, decision_id, rodzaj, funkcja="osoba_wyznaczona", dotyczy=(None, None), **fields):
    ask(room, decision_id, rodzaj, funkcja, dotyczy)
    resolve(room, decision_id, funkcja, **fields)


def add_client(room, client_id="nowak", kartoteka_odnosnik="kartoteka/nowak", **fields):
    room.run(
        "wpisz_klienta_kontroli",
        **{
            "id": client_id,
            "klient": "Anna Nowak",
            "kartoteka_odnosnik": kartoteka_odnosnik,
            "kontroler": "kontroler-nowak",
            "zrodlo": "raport-kontrolera/1",
            "wpisal": "kierownik-kontroli",
            "wpisano": "2026-03-01",
            **fields,
        },
    )


def checked(**fields):
    """The columns every check entry has: source, report, Controller and who wrote it."""
    return {
        "zrodlo": "dowód osobisty",
        "raport_odnosnik": "raport-kontrolera/1",
        "sprawdzil": "kontroler-nowak",
        "sprawdzono": "2026-03-01",
        "wpisal": "kierownik-kontroli",
        "wpisano": "2026-03-01",
        **fields,
    }


def person_identification(**fields):
    return checked(
        **{
            "klient_id": "nowak",
            "kogo": "klient",
            "rodzaj_podmiotu": "osoba_fizyczna",
            "imiona": "Anna",
            "nazwisko": "Nowak",
            "obywatelstwo": "PL",
            "pesel": "80010112345",
            "dokument_tozsamosci": "ABC123456",
            **fields,
        }
    )


def identify(room, **fields):
    room.run("wpisz_identyfikacje", **person_identification(**fields))

    return room.run("identyfikacje_klienta", klient_id=fields.get("klient_id", "nowak"))[0]["id"]


def pep_fields(**fields):
    return checked(
        **{
            "klient_id": "nowak",
            "dotyczy": "klient",
            "osoba": "Anna Nowak",
            "oswiadczenie_sciezka": "oswiadczenia/nowak.pdf",
            "oswiadczenie_skrot": "sha256-oswiadczenia",
            "oswiadczenie_z_klauzula": 1,
            "wykaz_sprawdzono": "2026-03-01",
            "wynik": "nie_pep",
            **fields,
        }
    )


def out_of_act_fields(**fields):
    return checked(
        **{
            "klient_id": "nowak",
            "rodzaj": "rachunek",
            "teczka_odnosnik": "teczka-najmu",
            "wiersz_odnosnik": "teczka-najmu/rachunek-1",
            "sprawdzenie_odnosnik": "teczka-najmu/sprawdzenie-1",
            "wyplata_odnosnik": "teczka-najmu/wyplata-1",
            "zrodlo": "telefon do posiadacza z numeru z umowy",
            "wynik": "zgodne",
            **fields,
        }
    )


def assessment(room, assessment_id="ocena", poziom="niskie", srodki="zwykle", **fields):
    room.run(
        "wpisz_ocene_ryzyka_klienta",
        **{
            "id": assessment_id,
            "klient_id": "nowak",
            "poziom": poziom,
            "czynniki": "osoba fizyczna, rezydent, sprzedaż mieszkania",
            "srodki": srodki,
            "zaproponowal": "kontroler-nowak",
            "raport_odnosnik": "raport-kontrolera/2",
            "wpisal": "kierownik-kontroli",
            "wpisano": "2026-03-02",
            **fields,
        },
    )


def office_assessment_fields(**fields):
    return {
        "id": "ocena-biura-1",
        "wersja": 1,
        "czynniki": "klienci, obszary, usługi, transakcje i kanały biura",
        "sporzadzono": "2026-01-10",
        "aktualizacja_do": "2028-01-10",
        "dokument_sciezka": "oceny/biuro-1.pdf",
        "dokument_skrot": "sha256-oceny",
        "akceptacja_decyzja_id": "akceptacja-oceny-1",
        "wpisal": "kierownik-kontroli",
        "wpisano": "2026-03-01",
        **fields,
    }


def doubt_fields(**fields):
    return {
        "id": "watpliwosc",
        "teczka_odnosnik": "teczka-transakcji",
        "znacznik_odnosnik": "teczka-transakcji/znacznik-1",
        "klient_id": "nowak",
        "zglosil": "kontroler-nowak",
        "zgloszono": "2026-03-03T09:00:00Z",
        "powod": "zapłata gotówką z niewyjaśnionego źródła",
        "raport_odnosnik": "raport-kontrolera/3",
        "wpisal": "kierownik-kontroli",
        "wpisano": "2026-03-03",
        **fields,
    }


def notification_fields(**fields):
    return {
        "id": "zawiadomienie",
        "rodzaj": "art74",
        "potwierdzenie_decyzja_id": "potwierdzenie",
        "przekazanie_decyzja_id": "przekazanie",
        "przekazal": "osoba-wyznaczona",
        "przekazano": "2026-03-02T09:00:00Z",
        "podstawy": "watpliwosc, analiza 12",
        "wpisal": "kierownik-kontroli",
        "wpisano": "2026-03-02",
        **fields,
    }


def procedure_fields(**fields):
    return {
        "id": "procedura-1",
        "wersja": 1,
        "dokument_sciezka": "procedury/1.pdf",
        "dokument_skrot": "sha256-procedury",
        "akceptacja_decyzja_id": "akceptacja-procedury-1",
        "wprowadzono": "2026-03-01",
        "wpisal": "kierownik-kontroli",
        "wpisano": "2026-03-01",
        **fields,
    }


def full_register(room):
    """A row in every table of the register."""
    add_client(room)
    room.run(
        "zakwalifikuj_sprawe",
        klient_id="nowak",
        teczka_odnosnik="teczka-oferty",
        rodzaj_sprawy="sprzedaz_albo_kupno",
        objeta="tak",
        podstawa="art. 2 ust. 1 pkt 18 AML",
        zakwalifikowano="2026-03-01",
        wpisal="kierownik-kontroli",
    )

    identification_id = identify(room)
    room.run(
        "wpisz_weryfikacje",
        identyfikacja_id=identification_id,
        podstawa="dowod_osobisty",
        wynik="zgodne",
        **checked(),
    )
    room.run(
        "wpisz_kopie_dokumentu",
        identyfikacja_id=identification_id,
        plik_sciezka="kopie/nowak.pdf",
        plik_skrot="sha256-kopii",
        **checked(),
    )
    room.run(
        "wpisz_odpis_crbr",
        klient_id="nowak",
        odpis_sciezka="crbr/nowak.pdf",
        odpis_skrot="sha256-odpisu",
        rozbieznosc=0,
        **checked(),
    )
    room.run("wpisz_sprawdzenie_pep", **pep_fields())
    room.run(
        "wpisz_sprawdzenie_sankcji",
        klient_id="nowak",
        listy="ONZ, UE, lista GIIF",
        wynik="brak_trafien",
        **checked(),
    )
    room.run(
        "wpisz_cel_stosunkow",
        klient_id="nowak",
        cel="sprzedaż mieszkania odziedziczonego po matce",
        charakter="jednorazowa sprzedaż",
        **checked(),
    )
    room.run(
        "wpisz_zrodlo_srodkow",
        klient_id="nowak",
        pochodzenie_srodkow="spadek",
        dokument_sciezka="zrodla/postanowienie.pdf",
        dokument_skrot="sha256-postanowienia",
        **checked(),
    )
    room.run(
        "wpisz_analize",
        klient_id="nowak",
        teczka_odnosnik="teczka-transakcji",
        analiza="cena zgodna z rynkiem, zapłata przelewem z rachunku kupującego",
        wynik="zgodne",
        **checked(),
    )
    room.run("wpisz_sprawdzenie_poza_ustawa", **out_of_act_fields())
    room.run(
        "wpisz_informacje_aml",
        klient_id="nowak",
        poinformowano="2026-03-01T09:00:00Z",
        sposob="email",
        zrodlo="teczka-oferty/wysylka-informacji",
        raport_odnosnik="raport-kontrolera/1",
        wpisal="kierownik-kontroli",
        wpisano="2026-03-01",
    )

    approved(
        room,
        "zatwierdzenie-oceny",
        "zatwierdzenie_oceny",
        dotyczy=("rejestr_kontroli_oceny_klientow", "ocena"),
    )
    assessment(room)
    room.run(
        "wpisz_zatwierdzenie_oceny", id="ocena", zatwierdzenie_decyzja_id="zatwierdzenie-oceny"
    )
    approved(
        room,
        "akceptacja-oceny-1",
        "akceptacja_oceny_biura",
        "wlasciciel_biura",
        ("rejestr_kontroli_oceny_biura", "ocena-biura-1"),
    )
    room.run("wpisz_ocene_ryzyka_biura", **office_assessment_fields())

    room.run("wpisz_watpliwosc", **doubt_fields())
    approved(room, "nienawiazanie", "nienawiazanie_stosunkow", dotyczy=(CLIENTS, "nowak"))
    room.run(
        "wpisz_niemoznosc",
        klient_id="nowak",
        srodek="beneficjent",
        opis="klient nie wskazał beneficjenta spółki",
        decyzja_id="nienawiazanie",
        ocena_zawiadomienia="brak podstaw do zawiadomienia",
        raport_odnosnik="raport-kontrolera/4",
        wpisal="kierownik-kontroli",
        wpisano="2026-03-05",
    )

    approved(room, "potwierdzenie", "potwierdzenie_podejrzenia")
    approved(room, "przekazanie", "przekazanie_zawiadomienia")
    room.run("wpisz_zawiadomienie", **notification_fields())
    room.run(
        "wpisz_zadanie_organu",
        organ="Generalny Inspektor Informacji Finansowej",
        zadano="2026-04-01T09:00:00Z",
        tresc="dane klienta i transakcji",
        odebral="osoba-wyznaczona",
        wpisal="kierownik-kontroli",
        wpisano="2026-04-01",
    )
    room.run(
        "wpisz_kontrole_organu",
        organ="Krajowa Administracja Skarbowa",
        od="2026-05-04",
        do="2026-05-08",
        protokol_sciezka="kontrole/2026-05.pdf",
        protokol_skrot="sha256-protokolu",
        wpisal="kierownik-kontroli",
        wpisano="2026-05-10",
    )

    approved(
        room,
        "akceptacja-procedury-1",
        "akceptacja_procedury",
        "wlasciciel_biura",
        ("rejestr_kontroli_organizacja_procedury", "procedura-1"),
    )
    room.run("wpisz_procedure", **procedure_fields())
    room.run(
        "wpisz_wyznaczenie",
        funkcja="art8",
        osoba="osoba-wyznaczona",
        od="2026-01-01",
        wpisal="kierownik-kontroli",
        wpisano="2026-03-01",
    )
    room.run(
        "wpisz_szkolenie",
        uczestnik="kierownik-kontroli",
        odbyto="2026-02-15",
        temat="standard Kontroli, wersja 3",
        potwierdzil="osoba-wyznaczona",
        wpisal="kierownik-kontroli",
        wpisano="2026-03-01",
    )


# A whole client -----------------------------------------------------------------------------------


@scenario("Rejestr kontroli: klient od weryfikacji przed umową do końca stosunków i powrotu")
def whole_client():
    room = new_room()
    add_client(room, kartoteka_odnosnik=None)
    identify(room)
    room.run("wpisz_sprawdzenie_pep", **pep_fields())
    approved(
        room,
        "zatwierdzenie",
        "zatwierdzenie_oceny",
        dotyczy=("rejestr_kontroli_oceny_klientow", "ocena"),
    )
    assessment(room)
    room.run("wpisz_zatwierdzenie_oceny", id="ocena", zatwierdzenie_decyzja_id="zatwierdzenie")

    room.run("wpisz_odnosnik_do_kartoteki", id="nowak", kartoteka_odnosnik="kartoteka/nowak")
    room.run("wpisz_nawiazanie_stosunkow", id="nowak", stosunki_od="2026-03-05")
    room.run("wpisz_zakonczenie_stosunkow", id="nowak", stosunki_do="2026-10-30")

    add_client(room, "nowak-2028", wpisano="2028-02-01")
    history = room.run("stosunki_klienta_z_kartoteki", kartoteka_odnosnik="kartoteka/nowak")
    assert [(row["id"], row["stosunki_do"]) for row in history] == [
        ("nowak", "2026-10-30"),
        ("nowak-2028", None),
    ], history
    assert (
        room.run("oceny_ryzyka_klienta", klient_id="nowak")[0]["zatwierdzenie_decyzja_id"]
        == "zatwierdzenie"
    )


# Decisions ----------------------------------------------------------------------------------------


@scenario("Decyzje: prośba powstaje czekająca, rozstrzygnięcie ma pełny podpis tego, kto decyduje")
def decision_signature():
    room = new_room()
    room.refused_sql(
        "prośba o decyzję powstaje jako czekająca",
        """INSERT INTO rejestr_kontroli_decyzje_wpisy (id, rodzaj, prosi, decyduje, decyduje_funkcja,
          zlozono, stan)
        VALUES ('od-razu', 'zatwierdzenie_oceny', 'kierownik', 'osoba', 'osoba_wyznaczona',
          '2026-03-01T10:00:00Z', 'rozstrzygnieta')""",
    )
    ask(room, "prosba", "zatwierdzenie_oceny")

    room.refused(
        "podpis_w_komplecie",
        "rozstrzygnij_decyzje_kontroli",
        id="prosba",
        rozstrzygniecie="zgoda",
        podpisal="osoba-wyznaczona",
        podpisano="2026-03-01T12:00:00Z",
    )
    room.refused_sql(
        "podpis_tylko_przy_rozstrzygnieciu",
        """UPDATE rejestr_kontroli_decyzje_wpisy
        SET rozstrzygniecie = 'zgoda', podpisal = 'osoba-wyznaczona', podpis = 'podpis',
          podpisano = '2026-03-01T12:00:00Z'
        WHERE id = 'prosba'""",
    )
    room.refused(
        "podpisuje_ten_kto_decyduje",
        "rozstrzygnij_decyzje_kontroli",
        id="prosba",
        rozstrzygniecie="zgoda",
        podpisal="kierownik-kontroli",
        podpis="podpis",
        podpisano="2026-03-01T12:00:00Z",
    )
    room.refused(
        "rozstrzygniecie_nie_przed_zlozeniem",
        "rozstrzygnij_decyzje_kontroli",
        id="prosba",
        rozstrzygniecie="zgoda",
        podpisal="osoba-wyznaczona",
        podpis="podpis",
        podpisano="2026-03-01T09:00:00Z",
    )

    resolve(room, "prosba")
    assert room.run("decyzje_kontroli_czekajace") == []


@scenario("Decyzje: rozstrzyga właściwa osoba, a zdjęcie wątpliwości ma uzasadnienie")
def decision_right_person():
    room = new_room()

    room.refused(
        "rozstrzyga_wlasciwa_osoba",
        "popros_o_decyzje_kontroli",
        **request_fields(rodzaj="akceptacja_pep"),
    )
    room.refused(
        "rozstrzyga_wlasciwa_osoba",
        "popros_o_decyzje_kontroli",
        **request_fields(decyduje_funkcja="wlasciciel_biura"),
    )
    room.refused(
        "dotyczy_tabela_razem_z_id",
        "popros_o_decyzje_kontroli",
        **request_fields(dotyczy_tabela=CLIENTS),
    )
    ask(room, "nienawiazanie-wlasciciel", "nienawiazanie_stosunkow", "wlasciciel_biura")
    ask(room, "nienawiazanie-osoba", "nienawiazanie_stosunkow")

    ask(room, "zdjecie", "zdjecie_watpliwosci")
    room.refused(
        "zdjecie_watpliwosci_z_uzasadnieniem",
        "rozstrzygnij_decyzje_kontroli",
        **resolution_fields("zdjecie"),
    )
    ask(room, "ocena", "zatwierdzenie_oceny")
    room.refused(
        "decyzja_w_teczce_tylko_przy_zdjeciu_watpliwosci",
        "rozstrzygnij_decyzje_kontroli",
        **resolution_fields("ocena", teczka_decyzja_odnosnik="teczka/decyzja"),
    )
    resolve(
        room,
        "zdjecie",
        uzasadnienie="źródło środków wyjaśnione umową darowizny",
        teczka_decyzja_odnosnik="teczka/decyzja",
    )


def request_fields(**fields):
    return {
        "id": "prosba",
        "rodzaj": "zatwierdzenie_oceny",
        "prosi": "kierownik-kontroli",
        "decyduje": "osoba-wyznaczona",
        "decyduje_funkcja": "osoba_wyznaczona",
        "zlozono": "2026-03-01T10:00:00Z",
        **fields,
    }


def resolution_fields(decision_id, **fields):
    return {
        "id": decision_id,
        "rozstrzygniecie": "zgoda",
        "podpisal": "osoba-wyznaczona",
        "podpis": "podpis",
        "podpisano": "2026-03-01T12:00:00Z",
        **fields,
    }


@scenario("Decyzje: prośby się nie zmienia, zbędna ma powód, rozstrzygnięta jest ostateczna")
def decision_final():
    room = new_room()
    ask(room, "prosba", "zatwierdzenie_oceny")

    room.refused_sql(
        "prośby o decyzję się nie zmienia",
        "UPDATE rejestr_kontroli_decyzje_wpisy SET tresc = 'inna' WHERE id = 'prosba'",
    )
    room.refused_sql(
        "zbedna_ma_powod",
        "UPDATE rejestr_kontroli_decyzje_wpisy SET stan = 'zbedna' WHERE id = 'prosba'",
    )
    room.run("uznaj_prosbe_kontroli_za_zbedna", id="prosba", powod_zbednosci="klient zrezygnował")
    room.refused(
        "rozstrzygniętej ani zbędnej decyzji się nie zmienia",
        "rozstrzygnij_decyzje_kontroli",
        **resolution_fields("prosba"),
    )

    approved(room, "druga", "zatwierdzenie_oceny")
    room.refused(
        "rozstrzygniętej ani zbędnej decyzji się nie zmienia",
        "rozstrzygnij_decyzje_kontroli",
        **resolution_fields("druga", rozstrzygniecie="odmowa"),
    )


# Clients and cases --------------------------------------------------------------------------------


@scenario(
    "Klienci: jedne trwające stosunki; powrót klienta to nowy wpis po zakończeniu poprzednich"
)
def client_relationships():
    room = new_room()
    add_client(room)
    ongoing = "klient ma już trwające stosunki"

    room.refused(ongoing, "wpisz_klienta_kontroli", **client_fields("nowak-2"))
    add_client(room, "bez-kartoteki", kartoteka_odnosnik=None)
    room.refused(
        ongoing,
        "wpisz_odnosnik_do_kartoteki",
        id="bez-kartoteki",
        kartoteka_odnosnik="kartoteka/nowak",
    )

    room.refused(
        "stosunki_konczy_sie_po_nawiazaniu",
        "wpisz_zakonczenie_stosunkow",
        id="nowak",
        stosunki_do="2026-10-30",
    )
    room.run("wpisz_nawiazanie_stosunkow", id="nowak", stosunki_od="2026-03-05")
    room.refused(
        "stosunki_konczy_sie_po_nawiazaniu",
        "wpisz_zakonczenie_stosunkow",
        id="nowak",
        stosunki_do="2026-03-01",
    )
    room.run("wpisz_zakonczenie_stosunkow", id="nowak", stosunki_do="2026-10-30")
    add_client(room, "nowak-2")

    unchanged = "wpisu klienta się nie zmienia"
    room.refused(unchanged, "wpisz_zakonczenie_stosunkow", id="nowak", stosunki_do="2026-11-30")
    room.refused(unchanged, "wpisz_nawiazanie_stosunkow", id="nowak", stosunki_od="2026-03-06")
    room.refused(
        unchanged, "wpisz_odnosnik_do_kartoteki", id="nowak", kartoteka_odnosnik="kartoteka/inna"
    )
    room.refused_sql(unchanged, f"UPDATE {CLIENTS} SET kontroler = 'inny' WHERE id = 'nowak'")


def client_fields(client_id, **fields):
    return {
        "id": client_id,
        "klient": "Anna Nowak",
        "kartoteka_odnosnik": "kartoteka/nowak",
        "kontroler": "kontroler-nowak",
        "zrodlo": "raport-kontrolera/1",
        "wpisal": "kierownik-kontroli",
        "wpisano": "2026-03-01",
        **fields,
    }


@scenario(
    "Sprawy: przy najmie czynsz z kursem i dniem kursu, a próg 10 000 euro rozstrzyga objęcie"
)
def case_qualification():
    room = new_room()
    add_client(room)
    rent = {
        "klient_id": "nowak",
        "teczka_odnosnik": "teczka-najmu",
        "rodzaj_sprawy": "posrednictwo_najmu",
        "podstawa": "art. 2 ust. 1 pkt 18 AML",
        "zakwalifikowano": "2026-03-01",
        "wpisal": "kierownik-kontroli",
    }

    room.refused("najem_ma_czynsz", "zakwalifikuj_sprawe", **rent, objeta="nie")
    room.refused(
        "czynsz_z_kursem_i_dniem", "zakwalifikuj_sprawe", **rent, objeta="nie", czynsz=500_000
    )
    room.refused(
        "prog_najmu",
        "zakwalifikuj_sprawe",
        **rent,
        objeta="tak",
        czynsz=500_000,
        kurs_euro=4.25,
        dzien_kursu="2026-02-27",
    )
    room.refused(
        "prog_najmu",
        "zakwalifikuj_sprawe",
        **rent,
        objeta="nie",
        czynsz=4_500_000,
        kurs_euro=4.25,
        dzien_kursu="2026-02-27",
    )

    room.run(
        "zakwalifikuj_sprawe",
        **rent,
        objeta="nie",
        czynsz=500_000,
        kurs_euro=4.25,
        dzien_kursu="2026-02-27",
    )
    room.run(
        "zakwalifikuj_sprawe",
        **rent,
        objeta="tak",
        czynsz=4_500_000,
        kurs_euro=4.25,
        dzien_kursu="2026-02-27",
    )
    room.run(
        "zakwalifikuj_sprawe",
        **{**rent, "rodzaj_sprawy": "zarzadzanie", "teczka_odnosnik": "teczka-lokalu"},
        objeta="do_rozstrzygniecia",
    )
    assert len(room.run("sprawy_klienta_kontroli", klient_id="nowak")) == 3
    room.refused_sql(
        "kwalifikacji sprawy się nie zmienia",
        "UPDATE rejestr_kontroli_klienci_sprawy SET objeta = 'tak'",
    )


# Checks -------------------------------------------------------------------------------------------


@scenario("Identyfikacja: dane z art. 36 AML osoby, podmiotu, beneficjenta i upoważnionego")
def identification_data():
    room = new_room()
    add_client(room)
    entity = {
        "rodzaj_podmiotu": "podmiot",
        "imiona": None,
        "nazwisko": None,
        "obywatelstwo": None,
        "pesel": None,
        "dokument_tozsamosci": None,
        "nazwa": "Leśna Inwestycje sp. z o.o.",
        "forma_organizacyjna": "spółka z ograniczoną odpowiedzialnością",
        "adres_siedziby": "ul. Leśna 1, Poznań",
    }

    refusals = [
        ("osoba_ma_imie_i_nazwisko", {"nazwisko": None}),
        ("osoba_ma_obywatelstwo_pesel_albo_urodzenie_i_dokument", {"obywatelstwo": None}),
        (
            "osoba_ma_obywatelstwo_pesel_albo_urodzenie_i_dokument",
            {"pesel": None, "data_urodzenia": "1980-01-01"},
        ),
        ("osoba_ma_obywatelstwo_pesel_albo_urodzenie_i_dokument", {"dokument_tozsamosci": None}),
        ("pesel_ma_11_cyfr", {"pesel": "8001011234"}),
        ("podmiotem_bywa_tylko_klient", {**entity, "kogo": "beneficjent"}),
        (
            "podmiot_ma_nazwe_forme_i_siedzibe",
            {**entity, "adres_siedziby": None, "nip": "5250000000"},
        ),
        ("podmiot_ma_nip_albo_rejestr", entity),
        ("upowazniony_ma_dokument_umocowania", {"kogo": "upowazniony"}),
    ]
    for rule, fields in refusals:
        room.refused(rule, "wpisz_identyfikacje", **person_identification(**fields))

    identify(room)
    identify(
        room, pesel=None, data_urodzenia="1980-01-01", panstwo_urodzenia="UA", obywatelstwo="UA"
    )
    identify(room, kogo="beneficjent", obywatelstwo=None, pesel=None, dokument_tozsamosci=None)
    identify(room, kogo="upowazniony", umocowanie_dokument="teczka/pelnomocnictwo")
    identify(room, **entity, nip="5250000000")
    identify(
        room,
        **entity,
        rejestr_panstwo="DE",
        rejestr_nazwa="Handelsregister",
        rejestr_numer="HRB 1234",
        rejestr_data="2020-05-01",
    )
    assert len(room.run("identyfikacje_klienta", klient_id="nowak")) == 6


@scenario(
    "Ponowne sprawdzenie: wskazuje aktualny wpis tego samego klienta albo wiersza i go zastępuje"
)
def recheck_replaces_previous():
    room = new_room()
    full_register(room)
    add_client(room, "lis", kartoteka_odnosnik="kartoteka/lis")

    for table, owner in RECHECKED_TABLES.items():
        first_id = room.value(f"SELECT min(id) FROM {table}")
        columns = [
            row["name"]
            for row in room.execute(f"PRAGMA table_info({table})")
            if row["name"] not in ("id", "aktualne", "poprzedni_id")
        ]
        column_list = ", ".join(columns)
        recheck = (
            f"INSERT INTO {table} ({column_list}, aktualne, poprzedni_id) "
            f"SELECT {column_list}, 1, :poprzedni_id FROM {table} WHERE id = :poprzedni_id"
        )

        room.refused_sql(
            "wpis sprawdzenia powstaje jako aktualny",
            f"INSERT INTO {table} ({column_list}, aktualne) SELECT {column_list}, 0 FROM {table} WHERE id = {first_id}",
        )
        room.refused_sql(CHECK_UNCHANGED, f"UPDATE {table} SET aktualne = 0 WHERE id = {first_id}")
        room.refused_sql(
            CHECK_UNCHANGED, f"UPDATE {table} SET wpisano = '2026-12-31' WHERE id = {first_id}"
        )

        room.execute(recheck, poprzedni_id=first_id)
        assert room.value(f"SELECT aktualne FROM {table} WHERE id = {first_id}") == 0, table
        room.refused_sql(
            "ponowne sprawdzenie wskazuje aktualny wpis",
            recheck,
            poprzedni_id=first_id,
        )
        room.refused_sql(CHECK_UNCHANGED, f"UPDATE {table} SET aktualne = 1 WHERE id = {first_id}")

        if owner == "klient_id":
            latest_id = room.value(f"SELECT max(id) FROM {table}")
            room.refused_sql(
                "ponowne sprawdzenie wskazuje aktualny wpis",
                f"INSERT INTO {table} ({column_list}, aktualne, poprzedni_id) "
                f"SELECT {column_list.replace('klient_id', "'lis'")}, 1, {latest_id} FROM {table} WHERE id = {latest_id}",
            )


@scenario("Weryfikacja, CRBR, analiza i sprawdzenie spoza ustawy: wyniki z opisem i odnośnikami")
def check_results():
    room = new_room()
    add_client(room)
    identification_id = identify(room)

    room.refused(
        "wynik_z_uwagami_ma_opis",
        "wpisz_weryfikacje",
        identyfikacja_id=identification_id,
        podstawa="dowod_osobisty",
        wynik="uwagi",
        **checked(),
    )
    crbr = {
        "klient_id": "nowak",
        "odpis_sciezka": "crbr.pdf",
        "odpis_skrot": "sha256",
        **checked(),
    }
    room.refused("rozbieznosc_ma_opis", "wpisz_odpis_crbr", **crbr, rozbieznosc=1)
    room.refused(
        "wyjasnienie_tylko_przy_rozbieznosci",
        "wpisz_odpis_crbr",
        **crbr,
        rozbieznosc=0,
        wyjasnienie="brak",
    )
    room.run(
        "wpisz_odpis_crbr",
        **crbr,
        rozbieznosc=1,
        rozbieznosc_opis="inny beneficjent w CRBR",
        wyjasnienie="zgłoszenie nieaktualne",
    )
    room.refused(
        "wynik_z_uwagami_ma_opis",
        "wpisz_analize",
        klient_id="nowak",
        teczka_odnosnik="teczka",
        analiza="zapłata gotówką",
        wynik="uwagi",
        **checked(),
    )

    room.refused(
        "wersja_wiersza_przy_wierszu_z_wersja",
        "wpisz_sprawdzenie_poza_ustawa",
        **out_of_act_fields(wersja_wiersza=1),
    )
    room.refused(
        "wersja_wiersza_przy_wierszu_z_wersja",
        "wpisz_sprawdzenie_poza_ustawa",
        **out_of_act_fields(rodzaj="tozsamosc", wyplata_odnosnik=None),
    )
    room.refused(
        "wyplata_tylko_przy_rachunku",
        "wpisz_sprawdzenie_poza_ustawa",
        **out_of_act_fields(rodzaj="tozsamosc", wersja_wiersza=2),
    )
    room.run("wpisz_sprawdzenie_poza_ustawa", **out_of_act_fields())
    room.run(
        "wpisz_sprawdzenie_poza_ustawa",
        **out_of_act_fields(
            klient_id=None,
            rodzaj="tytul",
            wiersz_odnosnik="teczka-najmu/uczestnik-wynajmujacy",
            wersja_wiersza=1,
            wyplata_odnosnik=None,
            wynik="uwagi",
            opis="w księdze inny właściciel",
        ),
    )
    assert len(room.run("sprawdzenia_poza_ustawa_teczki", teczka_odnosnik="teczka-najmu")) == 2


@scenario("PEP: oświadczenie z klauzulą, źródło majątku i akceptacja właściciela o tym kliencie")
def politically_exposed():
    room = new_room()
    add_client(room)
    add_client(room, "lis", kartoteka_odnosnik="kartoteka/lis")
    accepted = "akceptacja przy eksponowanym stanowisku wskazuje zatwierdzoną decyzję akceptacja_pep o tym kliencie"
    pep = {"wynik": "pep", "zrodlo_majatku": "wynagrodzenie posła i spadek"}

    room.refused(
        "oswiadczenie_z_klauzula", "wpisz_sprawdzenie_pep", **pep_fields(oswiadczenie_z_klauzula=0)
    )
    room.refused(
        "pep_ma_zrodlo_majatku", "wpisz_sprawdzenie_pep", **pep_fields(wynik="czlonek_rodziny")
    )
    room.run(
        "wpisz_sprawdzenie_pep",
        **pep_fields(oswiadczenie_sciezka=None, oswiadczenie_skrot=None, oswiadczenie_z_klauzula=0),
    )
    room.run("wpisz_sprawdzenie_pep", **pep_fields(**pep))
    pep_id = room.run("sprawdzenia_pep_klienta", klient_id="nowak")[0]["id"]
    no_pep_id = room.run("sprawdzenia_pep_klienta", klient_id="nowak")[1]["id"]

    approved(room, "akceptacja-lis", "akceptacja_pep", "wlasciciel_biura", (CLIENTS, "lis"))
    ask(room, "czekajaca", "akceptacja_pep", "wlasciciel_biura", (CLIENTS, "nowak"))
    approved(room, "akceptacja", "akceptacja_pep", "wlasciciel_biura", (CLIENTS, "nowak"))
    room.refused(
        accepted, "wpisz_akceptacje_pep", id=pep_id, akceptacja_decyzja_id="akceptacja-lis"
    )
    room.refused(accepted, "wpisz_akceptacje_pep", id=pep_id, akceptacja_decyzja_id="czekajaca")
    room.refused(
        "akceptacja_tylko_przy_pep",
        "wpisz_akceptacje_pep",
        id=no_pep_id,
        akceptacja_decyzja_id="akceptacja",
    )
    room.refused_sql(
        accepted,
        "INSERT INTO rejestr_kontroli_sprawdzenia_pep (klient_id, dotyczy, osoba, wykaz_sprawdzono, wynik, zrodlo_majatku, akceptacja_decyzja_id, zrodlo, raport_odnosnik, sprawdzil, sprawdzono, wpisal, wpisano, aktualne) "
        "VALUES ('nowak', 'klient', 'Anna Nowak', '2026-03-01', 'pep', 'spadek', 'akceptacja-lis', 'wykaz', 'raport', 'kontroler', '2026-03-01', 'kierownik', '2026-03-01', 1)",
    )

    room.run("wpisz_akceptacje_pep", id=pep_id, akceptacja_decyzja_id="akceptacja")
    approved(room, "druga-akceptacja", "akceptacja_pep", "wlasciciel_biura", (CLIENTS, "nowak"))
    room.refused(
        CHECK_UNCHANGED, "wpisz_akceptacje_pep", id=pep_id, akceptacja_decyzja_id="druga-akceptacja"
    )


@scenario("Sankcje: zamrożenie tylko przy trafieniu, informacja po zamrożeniu, każde raz")
def sanctions():
    room = new_room()
    add_client(room)
    sanction = {"klient_id": "nowak", "listy": "ONZ, UE, lista GIIF", **checked()}

    room.run("wpisz_sprawdzenie_sankcji", **sanction, wynik="brak_trafien")
    clear_id = room.run("sprawdzenia_sankcji_klienta", klient_id="nowak")[0]["id"]
    room.refused(
        "zamrozenie_tylko_przy_trafieniu",
        "wpisz_zamrozenie",
        id=clear_id,
        zamrozono="2026-03-02T10:00:00Z",
    )

    room.run("wpisz_sprawdzenie_sankcji", **sanction, wynik="trafienie")
    hit_id = room.run("sprawdzenia_sankcji_klienta", klient_id="nowak")[0]["id"]
    room.refused(
        "informacja_po_zamrozeniu",
        "wpisz_informacje_o_zamrozeniu",
        id=hit_id,
        zamrozenie_informacja="2026-03-03T10:00:00Z",
    )
    room.run("wpisz_zamrozenie", id=hit_id, zamrozono="2026-03-02T10:00:00Z")
    room.refused(
        "informacja_po_zamrozeniu",
        "wpisz_informacje_o_zamrozeniu",
        id=hit_id,
        zamrozenie_informacja="2026-03-01T10:00:00Z",
    )
    room.run(
        "wpisz_informacje_o_zamrozeniu", id=hit_id, zamrozenie_informacja="2026-03-03T10:00:00Z"
    )

    room.refused(CHECK_UNCHANGED, "wpisz_zamrozenie", id=hit_id, zamrozono="2026-03-02T11:00:00Z")
    room.refused(
        CHECK_UNCHANGED,
        "wpisz_informacje_o_zamrozeniu",
        id=hit_id,
        zamrozenie_informacja="2026-03-04T10:00:00Z",
    )


@scenario("Kopie, informacje i analizy się nie zmieniają")
def immutable_check_entries():
    room = new_room()
    full_register(room)

    for table in (
        "rejestr_kontroli_sprawdzenia_kopie",
        "rejestr_kontroli_sprawdzenia_monitorowanie",
        "rejestr_kontroli_sprawdzenia_informacje",
    ):
        room.refused_sql(CHECK_UNCHANGED, f"UPDATE {table} SET wpisano = '2026-12-31'")


# Risk assessments ---------------------------------------------------------------------------------


@scenario("Ocena klienta: środki pasują do poziomu, nowa ocena odbiera aktualność poprzedniej")
def client_assessment():
    room = new_room()
    add_client(room)

    room.refused(
        "uproszczone_tylko_przy_niskim",
        "wpisz_ocene_ryzyka_klienta",
        **assessment_fields(poziom="srednie", srodki="uproszczone"),
    )
    room.refused(
        "wzmozone_przy_wysokim",
        "wpisz_ocene_ryzyka_klienta",
        **assessment_fields(poziom="wysokie", srodki="zwykle"),
    )
    room.refused_sql(
        "ocena ryzyka klienta powstaje jako aktualna",
        """INSERT INTO rejestr_kontroli_oceny_klientow (id, klient_id, poziom, czynniki, srodki,
          zaproponowal, raport_odnosnik, wpisal, wpisano, aktualna)
        VALUES ('stara', 'nowak', 'niskie', 'czynniki', 'zwykle', 'kontroler', 'raport', 'kierownik',
          '2026-03-02', 0)""",
    )

    assessment(room, "pierwsza", "niskie", "uproszczone")
    room.refused_sql(
        "oceny ryzyka klienta się nie zmienia",
        "UPDATE rejestr_kontroli_oceny_klientow SET aktualna = 0 WHERE id = 'pierwsza'",
    )
    assessment(room, "druga", "wysokie", "wzmozone", wpisano="2026-06-01")
    current = [
        row["id"]
        for row in room.run("oceny_ryzyka_klienta", klient_id="nowak")
        if row["aktualna"] == 1
    ]
    assert current == ["druga"], current
    room.refused_sql(
        "oceny ryzyka klienta się nie zmienia",
        "UPDATE rejestr_kontroli_oceny_klientow SET poziom = 'srednie' WHERE id = 'druga'",
    )


def assessment_fields(**fields):
    return {
        "id": "ocena",
        "klient_id": "nowak",
        "poziom": "niskie",
        "czynniki": "osoba fizyczna, rezydent",
        "srodki": "zwykle",
        "zaproponowal": "kontroler-nowak",
        "raport_odnosnik": "raport-kontrolera/2",
        "wpisal": "kierownik-kontroli",
        "wpisano": "2026-03-02",
        **fields,
    }


@scenario("Ocena klienta: zatwierdza raz aktualną ocenę zatwierdzona decyzja o tej ocenie")
def client_assessment_approval():
    room = new_room()
    add_client(room)
    assessment(room, "pierwsza")
    about_first = ("rejestr_kontroli_oceny_klientow", "pierwsza")
    rule = "zatwierdzenie oceny wskazuje zatwierdzoną decyzję zatwierdzenie_oceny o tej ocenie"

    approved(
        room, "o-innej", "zatwierdzenie_oceny", dotyczy=("rejestr_kontroli_oceny_klientow", "inna")
    )
    ask(room, "czekajaca", "zatwierdzenie_oceny", dotyczy=about_first)
    approved(room, "zla", "zdjecie_watpliwosci", dotyczy=about_first, uzasadnienie="x")
    approved(room, "odmowa", "zatwierdzenie_oceny", dotyczy=about_first, rozstrzygniecie="odmowa")
    for decision_id in ("o-innej", "czekajaca", "zla", "odmowa"):
        room.refused(
            rule, "wpisz_zatwierdzenie_oceny", id="pierwsza", zatwierdzenie_decyzja_id=decision_id
        )
    room.refused_sql(
        rule,
        """INSERT INTO rejestr_kontroli_oceny_klientow (id, klient_id, poziom, czynniki, srodki,
          zaproponowal, raport_odnosnik, wpisal, wpisano, aktualna, zatwierdzenie_decyzja_id)
        VALUES ('druga', 'nowak', 'niskie', 'czynniki', 'zwykle', 'kontroler', 'raport', 'kierownik',
          '2026-03-02', 1, 'o-innej')""",
    )

    approved(room, "zatwierdzenie", "zatwierdzenie_oceny", dotyczy=about_first)
    room.run("wpisz_zatwierdzenie_oceny", id="pierwsza", zatwierdzenie_decyzja_id="zatwierdzenie")
    approved(room, "ponowne", "zatwierdzenie_oceny", dotyczy=about_first)
    room.refused(
        "oceny ryzyka klienta się nie zmienia",
        "wpisz_zatwierdzenie_oceny",
        id="pierwsza",
        zatwierdzenie_decyzja_id="ponowne",
    )

    # An assessment replaced by a newer one is approved no more.
    assessment(room, "trzecia", wpisano="2026-06-01")
    assessment(room, "czwarta", wpisano="2026-06-02")
    approved(
        room,
        "o-trzeciej",
        "zatwierdzenie_oceny",
        dotyczy=("rejestr_kontroli_oceny_klientow", "trzecia"),
    )
    room.refused(
        "oceny ryzyka klienta się nie zmienia",
        "wpisz_zatwierdzenie_oceny",
        id="trzecia",
        zatwierdzenie_decyzja_id="o-trzeciej",
    )


@scenario(
    "Ocena biura: kolejna wersja z akceptacją właściciela i aktualizacją najpóźniej po 2 latach"
)
def office_assessment():
    room = new_room()
    approved(
        room,
        "akceptacja-oceny-1",
        "akceptacja_oceny_biura",
        "wlasciciel_biura",
        ("rejestr_kontroli_oceny_biura", "ocena-biura-1"),
    )
    accepted = (
        "ocena ryzyka biura wskazuje zatwierdzoną decyzję akceptacja_oceny_biura o tej wersji"
    )

    room.refused(
        "aktualizacja_najpozniej_po_2_latach",
        "wpisz_ocene_ryzyka_biura",
        **office_assessment_fields(aktualizacja_do="2028-01-11"),
    )
    room.refused(
        "wersja oceny ryzyka biura ma kolejny numer",
        "wpisz_ocene_ryzyka_biura",
        **office_assessment_fields(wersja=2),
    )
    room.refused(
        accepted, "wpisz_ocene_ryzyka_biura", **office_assessment_fields(id="ocena-biura-inna")
    )

    room.run("wpisz_ocene_ryzyka_biura", **office_assessment_fields())
    room.refused_sql(
        "oceny ryzyka biura się nie zmienia",
        "UPDATE rejestr_kontroli_oceny_biura SET aktualizacja_do = '2027-01-10'",
    )
    assert room.run("oceny_ryzyka_biura")[0]["wersja"] == 1


# Doubts -------------------------------------------------------------------------------------------


@scenario("Wątpliwość: powstaje stojąca, zdejmuje ją raz decyzja o tej wątpliwości")
def doubt_lifted():
    room = new_room()
    add_client(room)
    rule = "zdjęcie wątpliwości wskazuje zatwierdzoną decyzję zdjecie_watpliwosci o tej wątpliwości"

    approved(
        room,
        "o-innej",
        "zdjecie_watpliwosci",
        dotyczy=("rejestr_kontroli_watpliwosci_wpisy", "inna"),
        uzasadnienie="wyjaśnione",
    )
    room.refused_sql(
        "wątpliwość powstaje jako stojąca",
        """INSERT INTO rejestr_kontroli_watpliwosci_wpisy (id, teczka_odnosnik, znacznik_odnosnik,
          zglosil, zgloszono, powod, raport_odnosnik, wpisal, wpisano, zdjecie_decyzja_id)
        VALUES ('od-razu', 'teczka', 'znacznik', 'kontroler', '2026-03-03T09:00:00Z', 'powód',
          'raport', 'kierownik', '2026-03-03', 'o-innej')""",
    )
    room.run("wpisz_watpliwosc", **doubt_fields())
    room.refused(REPLACEMENT, "wpisz_watpliwosc", **doubt_fields(id="ta-sama"))

    room.refused(rule, "wpisz_zdjecie_watpliwosci", id="watpliwosc", zdjecie_decyzja_id="o-innej")
    approved(
        room,
        "zdjecie",
        "zdjecie_watpliwosci",
        dotyczy=("rejestr_kontroli_watpliwosci_wpisy", "watpliwosc"),
        uzasadnienie="źródło środków wyjaśnione",
    )
    room.run("wpisz_zdjecie_watpliwosci", id="watpliwosc", zdjecie_decyzja_id="zdjecie")
    approved(
        room,
        "ponowne",
        "zdjecie_watpliwosci",
        dotyczy=("rejestr_kontroli_watpliwosci_wpisy", "watpliwosc"),
        uzasadnienie="j.w.",
    )
    room.refused(
        "wątpliwości się nie zmienia",
        "wpisz_zdjecie_watpliwosci",
        id="watpliwosc",
        zdjecie_decyzja_id="ponowne",
    )
    room.refused_sql(
        "wątpliwości się nie zmienia",
        "UPDATE rejestr_kontroli_watpliwosci_wpisy SET powod = 'inny'",
    )


@scenario("Niemożność środka wskazuje decyzję o nienawiązaniu stosunków z tym klientem")
def measure_impossible():
    room = new_room()
    add_client(room)
    add_client(room, "lis", kartoteka_odnosnik="kartoteka/lis")
    approved(room, "o-lisie", "nienawiazanie_stosunkow", dotyczy=(CLIENTS, "lis"))
    approved(room, "nienawiazanie", "nienawiazanie_stosunkow", dotyczy=(CLIENTS, "nowak"))
    impossibility = {
        "klient_id": "nowak",
        "srodek": "beneficjent",
        "opis": "klient nie wskazał beneficjenta",
        "ocena_zawiadomienia": "brak podstaw do zawiadomienia",
        "raport_odnosnik": "raport-kontrolera/4",
        "wpisal": "kierownik-kontroli",
        "wpisano": "2026-03-05",
    }

    room.refused(
        "niemożność środka wskazuje zatwierdzoną decyzję nienawiazanie_stosunkow o tym kliencie",
        "wpisz_niemoznosc",
        **impossibility,
        decyzja_id="o-lisie",
    )
    room.run("wpisz_niemoznosc", **impossibility, decyzja_id="nienawiazanie")
    room.refused_sql(
        "wpisu niemożności się nie zmienia",
        "UPDATE rejestr_kontroli_watpliwosci_niemoznosci SET opis = 'inny'",
    )


# Notifications, requests and inspections ----------------------------------------------------------


@scenario("Zawiadomienie: po potwierdzeniu podejrzenia i decyzji o przekazaniu, nie wcześniej")
def notification():
    room = new_room()
    approved(room, "potwierdzenie", "potwierdzenie_podejrzenia")
    approved(room, "przekazanie", "przekazanie_zawiadomienia")
    ask(room, "czekajace", "potwierdzenie_podejrzenia")

    room.refused(
        "zawiadomienie wskazuje zatwierdzoną decyzję potwierdzenie_podejrzenia",
        "wpisz_zawiadomienie",
        **notification_fields(potwierdzenie_decyzja_id="czekajace"),
    )
    room.refused(
        "zawiadomienie wskazuje zatwierdzoną decyzję przekazanie_zawiadomienia",
        "wpisz_zawiadomienie",
        **notification_fields(przekazanie_decyzja_id="potwierdzenie"),
    )
    room.refused(
        "przekazanie zawiadomienia nie jest wcześniejsze niż potwierdzenie podejrzenia",
        "wpisz_zawiadomienie",
        **notification_fields(przekazano="2026-03-01T11:00:00Z"),
    )
    room.refused_sql(
        "zawiadomienie powstaje bez potwierdzenia przyjęcia i odpowiedzi organu",
        """INSERT INTO rejestr_kontroli_zawiadomienia_wpisy (id, rodzaj, potwierdzenie_decyzja_id,
          przekazanie_decyzja_id, przekazal, przekazano, podstawy, potwierdzenie_przyjecia, wpisal,
          wpisano)
        VALUES ('od-razu', 'art86', 'potwierdzenie', 'przekazanie', 'osoba', '2026-03-02T09:00:00Z',
          'podstawy', 'UPO', 'kierownik', '2026-03-02')""",
    )

    room.run("wpisz_zawiadomienie", **notification_fields())
    room.run("wpisz_potwierdzenie_przyjecia", id="zawiadomienie", potwierdzenie_przyjecia="UPO 123")
    room.run(
        "wpisz_odpowiedz_organu",
        id="zawiadomienie",
        odpowiedz_organu="żądanie wstrzymania transakcji",
    )
    unchanged = "śladu zawiadomienia się nie zmienia"
    room.refused(
        unchanged,
        "wpisz_potwierdzenie_przyjecia",
        id="zawiadomienie",
        potwierdzenie_przyjecia="UPO 124",
    )
    room.refused(unchanged, "wpisz_odpowiedz_organu", id="zawiadomienie", odpowiedz_organu="inna")
    room.refused_sql(
        unchanged,
        "UPDATE rejestr_kontroli_zawiadomienia_wpisy SET przekazano = '2026-03-03T09:00:00Z'",
    )


@scenario("Żądanie organu: przekazanie raz, w komplecie i po żądaniu; kontroli się nie zmienia")
def authority_requests_and_inspections():
    room = new_room()
    full_register(room)
    request_id = room.run("zadania_organow")[0]["id"]

    room.refused_sql(
        "żądanie organu powstaje bez przekazania",
        """INSERT INTO rejestr_kontroli_zawiadomienia_zadania (organ, zadano, tresc, odebral,
          przekazano, co_przekazano, wpisal, wpisano)
        VALUES ('GIIF', '2026-04-01T09:00:00Z', 'dane', 'osoba', '2026-04-02T09:00:00Z', 'dane',
          'kierownik', '2026-04-02')""",
    )
    room.refused_sql(
        "przekazanie_w_komplecie",
        f"UPDATE rejestr_kontroli_zawiadomienia_zadania SET przekazano = '2026-04-02T09:00:00Z' WHERE id = {request_id}",
    )
    room.refused(
        "przekazanie_po_zadaniu",
        "wpisz_odpowiedz_na_zadanie",
        id=request_id,
        przekazano="2026-03-31T09:00:00Z",
        co_przekazano="dane klienta",
    )
    room.run(
        "wpisz_odpowiedz_na_zadanie",
        id=request_id,
        przekazano="2026-04-02T09:00:00Z",
        co_przekazano="dane klienta",
    )
    room.refused(
        "żądania organu się nie zmienia",
        "wpisz_odpowiedz_na_zadanie",
        id=request_id,
        przekazano="2026-04-03T09:00:00Z",
        co_przekazano="więcej",
    )

    room.refused(
        "koniec_nie_przed_poczatkiem",
        "wpisz_kontrole_organu",
        organ="KAS",
        od="2026-05-04",
        do="2026-05-01",
        protokol_sciezka="protokol.pdf",
        protokol_skrot="sha256",
        wpisal="kierownik-kontroli",
        wpisano="2026-05-10",
    )
    room.refused_sql(
        "wpisu kontroli się nie zmienia",
        "UPDATE rejestr_kontroli_zawiadomienia_kontrole SET zalecenia = 'brak'",
    )


# Organisation -------------------------------------------------------------------------------------


@scenario("Procedura: kolejna wersja z akceptacją właściciela, wprowadzona nie przed akceptacją")
def procedure():
    room = new_room()
    approved(
        room,
        "akceptacja-procedury-1",
        "akceptacja_procedury",
        "wlasciciel_biura",
        ("rejestr_kontroli_organizacja_procedury", "procedura-1"),
    )
    approved(
        room,
        "o-innej",
        "akceptacja_procedury",
        "wlasciciel_biura",
        ("rejestr_kontroli_organizacja_procedury", "inna"),
    )

    room.refused(
        "procedura wskazuje zatwierdzoną decyzję akceptacja_procedury o tej wersji",
        "wpisz_procedure",
        **procedure_fields(akceptacja_decyzja_id="o-innej"),
    )
    room.refused(
        "wersja procedury ma kolejny numer", "wpisz_procedure", **procedure_fields(wersja=2)
    )
    room.refused(
        "procedurę wprowadza się nie wcześniej niż w dniu akceptacji",
        "wpisz_procedure",
        **procedure_fields(wprowadzono="2026-02-28"),
    )

    room.run("wpisz_procedure", **procedure_fields())
    room.refused_sql(
        "wersji procedury się nie zmienia",
        "UPDATE rejestr_kontroli_organizacja_procedury SET wprowadzono = '2026-04-01'",
    )


@scenario(
    "Wyznaczenia: jedna osoba z art. 8 AML, wyznaczenie kończy się raz; szkoleń się nie zmienia"
)
def appointments_and_training():
    room = new_room()
    full_register(room)
    appointment = {"wpisal": "kierownik-kontroli", "wpisano": "2026-06-01"}

    room.refused(
        "osoba z art. 8 AML jest jedna",
        "wpisz_wyznaczenie",
        funkcja="art8",
        osoba="nastepca",
        od="2026-06-01",
        **appointment,
    )
    room.run(
        "wpisz_wyznaczenie",
        funkcja="art6",
        osoba="wlasciciel-biura",
        od="2026-01-01",
        **appointment,
    )
    room.run("wpisz_wyznaczenie", funkcja="art6", osoba="wspolnik", od="2026-01-01", **appointment)
    room.refused_sql(
        "wyznaczenie powstaje jako bieżące",
        """INSERT INTO rejestr_kontroli_organizacja_wyznaczenia (funkcja, osoba, od, do, wpisal, wpisano)
        VALUES ('art6', 'dawny', '2025-01-01', '2025-12-31', 'kierownik', '2026-06-01')""",
    )

    art8_id = next(row for row in room.run("wyznaczenia") if row["funkcja"] == "art8")["id"]
    room.refused("koniec_nie_przed_poczatkiem", "zakoncz_wyznaczenie", id=art8_id, do="2025-12-31")
    room.run("zakoncz_wyznaczenie", id=art8_id, do="2026-05-31")
    room.run("wpisz_wyznaczenie", funkcja="art8", osoba="nastepca", od="2026-06-01", **appointment)
    room.refused("wyznaczenia się nie zmienia", "zakoncz_wyznaczenie", id=art8_id, do="2026-06-30")

    room.refused_sql(
        "wpisu szkolenia się nie zmienia",
        "UPDATE rejestr_kontroli_organizacja_szkolenia SET temat = 'inny'",
    )


# R9 and replacement -------------------------------------------------------------------------------


def first_row_copy(room, table):
    """SQL that inserts again the first row of the table, as a careless upsert would."""
    columns = ", ".join(row["name"] for row in room.execute(f"PRAGMA table_info({table})"))

    return (
        f"INTO {table} ({columns}) SELECT {columns} FROM {table} "
        f"WHERE rowid = (SELECT min(rowid) FROM {table})"
    )


@scenario("R9: wierszy rejestru kontroli się nie usuwa ani nie zastępuje przez REPLACE albo upsert")
def no_deletion_or_replacement():
    room = new_room()
    full_register(room)

    tables = [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'rejestr\\_%' ESCAPE '\\'"
        )
    ]
    assert len(tables) == 24, tables

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

    # Version numbers and the flag of a case file are UNIQUE keys of their own.
    for table, copy in (
        (
            "rejestr_kontroli_oceny_biura",
            (
                "'nowy-klucz', wersja, czynniki, sporzadzono, aktualizacja_do, dokument_sciezka, "
                "dokument_skrot, akceptacja_decyzja_id, wpisal, wpisano"
            ),
        ),
        (
            "rejestr_kontroli_organizacja_procedury",
            (
                "'nowy-klucz', wersja, dokument_sciezka, dokument_skrot, akceptacja_decyzja_id, "
                "wprowadzono, wpisal, wpisano"
            ),
        ),
        (
            "rejestr_kontroli_watpliwosci_wpisy",
            (
                "'nowy-klucz', teczka_odnosnik, znacznik_odnosnik, klient_id, zglosil, zgloszono, "
                "powod, raport_odnosnik, wpisal, wpisano, zdjecie_decyzja_id"
            ),
        ),
    ):
        columns = ", ".join(row["name"] for row in room.execute(f"PRAGMA table_info({table})"))
        room.refused_sql(
            REPLACEMENT, f"INSERT OR REPLACE INTO {table} ({columns}) SELECT {copy} FROM {table}"
        )
