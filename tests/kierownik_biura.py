"""Scenarios of the office manager's memory room (spec/positions/biuro/kierownik_biura).

The room has the manager's own kinds and three kinds of the agent memory (findings, sources and
orders); the rules of those are checked in stanowisko.py. The office manager is no department head, so
the room has no department memory. Each scenario starts from an empty room and checks one rule the
database keeps: a write the rule allows and a write it refuses. The first scenario follows a day of
the manager: a card assigned by a rule, reports, a decision list, an absence, a new policy and the
monthly report.
"""

from pathlib import Path

from stanowisko import first_row_copy
from room import Room

KIEROWNIK_BIURA = (
    Path(__file__).resolve().parents[1] / "spec" / "positions" / "biuro" / "kierownik_biura"
)

SCENARIOS = []

REPLACEMENT = "wiersza pamięci nie zastępuje się nowym wpisem"
DELETION = "wierszy pamięci się nie usuwa (R9)"


def scenario(title):
    def register(function):
        SCENARIOS.append((f"kierownik_biura: {title}", function))

        return function

    return register


def new_room():
    return Room(KIEROWNIK_BIURA)


# Building blocks ---------------------------------------------------------------------------------


def owner_signature():
    return {
        "podpisal": "tozsamosc-wlasciciela-biura",
        "podpis": "podpis-kluczem-wlasciciela-biura",
    }


def add_rule(room, **fields):
    room.run(
        "zapisz_regule_przydzialu",
        **{
            "rodzaj_zgloszenia": "sprzedaz",
            "dzial": "posrednictwo",
            "od": "2026-11-01T00:00:00Z",
            **owner_signature(),
            **fields,
        },
    )


def take_card(room, **fields):
    room.run(
        "przyjmij_karte",
        **{
            "karta_odnosnik": "pokoj-recepcji/karta-1",
            "wplynela": "2026-11-04T09:30:00Z",
            "rodzaj_karty": "nowe_zgloszenie",
            "rodzaj_sprawy": "sprzedaz",
            "rodzaj_nieruchomosci": "mieszkanie",
            "miejscowosc": "Wrocław",
            "kanal": "telefon",
            "kontakt": "Anna Nowak, +48 600 000 000",
            **fields,
        },
    )


def check_client_register(room, card_id=1, **fields):
    room.run(
        "odnotuj_sprawdzenie_kartoteki",
        **{
            "id": card_id,
            "kartoteka_wynik": "brak",
            "sprawdzono_kartoteke": "2026-11-04T09:40:00Z",
            **fields,
        },
    )


def human_approval():
    return {
        "zatwierdzil": "tozsamosc-wlasciciela-biura",
        "podpis": "podpis-kluczem-wlasciciela-biura",
        "zatwierdzono": "2026-11-04T10:00:00Z",
    }


def add_department_report(room, **fields):
    room.run(
        "dopisz_meldunek_dzialu",
        **{
            "nadawca": "posrednictwo",
            "tozsamosc": "tozsamosc-kierownika-posrednictwa",
            "otrzymano": "2026-11-04T16:00:00Z",
            "agenci_pracuja": 12,
            "sprawy_zablokowane": 2,
            "decyzje_czekaja": 3,
            "najblizszy_termin": "2026-11-06",
            **fields,
        },
    )


def add_position(room, **fields):
    room.run(
        "przyjmij_pozycje",
        **{
            "prosba_odnosnik": "pokoj-teczki-1/decyzja-1",
            "dzial": "posrednictwo",
            "decyduje": "tozsamosc-posrednika",
            "opis": "Zgoda na obniżenie ceny ogłoszenia",
            "przyjeto": "2026-11-04T10:00:00Z",
            **fields,
        },
    )


def add_absence(room, **fields):
    room.run(
        "zapisz_nieobecnosc",
        **{
            "czlowiek": "tozsamosc-posrednika",
            "od_dnia": "2026-11-10",
            "do_dnia": "2026-11-14",
            "zastepca": "tozsamosc-wspolnika",
            "podpisal": "tozsamosc-posrednika",
            "podpisal_jako": "czlowiek",
            "podpis": "podpis-kluczem-posrednika",
            "wiadomosc_odnosnik": "wiadomosc-urlop",
            "zapisano": "2026-11-04T11:00:00Z",
            **fields,
        },
    )


def add_policy(room, **fields):
    room.run(
        "zapisz_polise",
        **{
            "rodzaj": "posrednika",
            "numer": "OC/2027/001",
            "ubezpieczyciel": "Towarzystwo Ubezpieczeń Przykładowe",
            "okres_od": "2027-01-01",
            "okres_do": "2027-12-31",
            "dokument_odnosnik": "dokumenty-biura/polisa-oc-2027",
            "zapisano": "2026-12-20T09:00:00Z",
            **fields,
        },
    )


def add_report(room, **fields):
    room.run(
        "zapisz_sprawozdanie",
        **{
            "rodzaj": "miesieczne",
            "okres_od": "2026-11-01",
            "okres_do": "2026-11-30",
            "sciezka": "sprawozdania/2026-11.typ",
            "skrot": "sha256-sprawozdanie-2026-11",
            "sporzadzono": "2026-12-01T09:00:00Z",
            **fields,
        },
    )


def full_memory(room):
    """One row in every table of the manager's own kinds, for the checks of R9 and replacement."""
    add_rule(room)
    take_card(room)
    add_department_report(room)
    add_position(room)
    add_absence(room)
    add_policy(room)
    add_report(room)


# A whole day ---------------------------------------------------------------------------------------


@scenario("kierownik przydziela kartę według reguły, układa listę decyzji i pilnuje polisy")
def whole_day():
    room = new_room()

    add_rule(room)
    add_rule(room, rodzaj_zgloszenia="zarzad", dzial="najem")
    take_card(room)
    check_client_register(room)
    room.run(
        "przydziel_karte",
        id=1,
        dzial="posrednictwo",
        regula_id=1,
        rozstrzygnieto="2026-11-04T10:00:00Z",
    )

    add_department_report(room)
    room.run(
        "dopisz_meldunek_recepcji",
        tozsamosc="tozsamosc-recepcji",
        otrzymano="2026-11-04T17:00:00Z",
        wiadomosci_wplynely=40,
        karty_wyslane=3,
        zaklocenia=1,
    )
    add_department_report(room, otrzymano="2026-11-05T16:00:00Z", agenci_pracuja=13)

    add_position(room)
    add_position(
        room,
        prosba_odnosnik="pokoj-teczki-2/decyzja-1",
        opis="Zgoda na umowę z wyłącznością",
        termin="2026-11-05",
        przyjeto="2026-11-04T11:00:00Z",
    )
    room.run(
        "odnow_pozycje",
        id=1,
        opis="Zgoda na obniżenie ceny ogłoszenia o 3%",
        odnowiono="2026-11-04T15:00:00Z",
    )
    room.run("odnotuj_wyslanie_pozycji", id=2, trafila_do_czlowieka="2026-11-04T11:05:00Z")

    add_absence(room)
    add_policy(room)
    room.run(
        "odnotuj_zawiadomienie_o_polisie",
        id=1,
        zawiadomiono="2026-12-20T10:00:00Z",
        zawiadomiono_kogo="kierownik_posrednictwa",
    )
    add_report(room)
    room.run("przekaz_sprawozdanie", id=1, przekazano="2026-12-01T12:00:00Z")

    card = room.execute("SELECT stan, kontakt FROM kierownik_biura_przydzial_karty")[0]
    assert card == {"stan": "przydzielona", "kontakt": None}
    latest = room.run("ostatnie_meldunki")
    assert [(row["nadawca"], row["agenci_pracuja"]) for row in latest] == [
        ("posrednictwo", 13),
        ("recepcja", None),
    ]
    decisions = room.run(
        "lista_decyzji", decyduje="tozsamosc-posrednika", bliski_termin="2026-11-06"
    )
    assert [row["prosba_odnosnik"] for row in decisions] == [
        "pokoj-teczki-2/decyzja-1",
        "pokoj-teczki-1/decyzja-1",
    ]
    assert [row["zastepca"] for row in room.run("nieobecnosci_w_dniu", dzien="2026-11-12")] == [
        "tozsamosc-wspolnika"
    ]
    assert len(room.run("polisa_w_dniu", rodzaj="posrednika", dzien="2027-03-01")) == 1
    assert [row["id"] for row in room.run("polisy_do_przypomnienia", do_dnia="2028-01-30")] == [1]
    assert room.run("karty_do_rozstrzygniecia") == []


# Rules of assignment -------------------------------------------------------------------------------


@scenario("reguła przydziału powstaje aktualna i jest jedna na rodzaj zgłoszenia")
def one_rule_per_request_kind():
    room = new_room()
    add_rule(room)

    room.refused(
        "ten rodzaj zgłoszenia ma już aktualną regułę", "zapisz_regule_przydzialu", **rule_fields()
    )
    room.refused_sql(
        "reguła przydziału powstaje aktualna",
        """
        INSERT INTO kierownik_biura_przydzial_reguly (rodzaj_zgloszenia, dzial, od, podpisal, podpis,
          zastapiona_od)
        VALUES ('kupno', 'posrednictwo', '2026-11-01', 'w', 'p', '2026-11-02')
        """,
    )

    room.refused(
        "zastapienie_nie_przed_poczatkiem",
        "zastap_regule_przydzialu",
        id=1,
        zastapiona_od="2026-10-01T00:00:00Z",
    )
    room.run("zastap_regule_przydzialu", id=1, zastapiona_od="2026-12-01T00:00:00Z")
    room.run("zapisz_regule_przydzialu", **rule_fields(dzial="najem", od="2026-12-01T00:00:00Z"))
    assert [row["dzial"] for row in room.run("reguly_przydzialu")] == ["najem"]
    assert len(room.run("historia_regul")) == 2


def rule_fields(**fields):
    return {
        "rodzaj_zgloszenia": "sprzedaz",
        "dzial": "posrednictwo",
        "od": "2026-11-01T00:00:00Z",
        **owner_signature(),
        **fields,
    }


@scenario("reguły przydziału się nie zmienia, a zastąpienie odnotowuje się raz")
def rule_does_not_change():
    room = new_room()
    add_rule(room)

    rule = "reguły przydziału się nie zmienia"
    room.refused_sql(rule, "UPDATE kierownik_biura_przydzial_reguly SET dzial = 'najem'")
    room.run("zastap_regule_przydzialu", id=1, zastapiona_od="2026-12-01T00:00:00Z")
    room.refused(rule, "zastap_regule_przydzialu", id=1, zastapiona_od="2026-12-02T00:00:00Z")


# Cards ---------------------------------------------------------------------------------------------


@scenario("karta powstaje czekająca, a sprawdzenie w kartotece jest kompletne i raz")
def card_starts_waiting_and_register_is_checked_once():
    room = new_room()
    take_card(room)

    room.refused_sql(
        "karta powstaje jako czekająca",
        """
        INSERT INTO kierownik_biura_przydzial_karty (karta_odnosnik, wplynela, rodzaj_karty, stan,
          powod, rozstrzygnieto)
        VALUES ('karta-9', '2026-11-04', 'pismo_bez_sprawy', 'odrzucona', 'reklama', '2026-11-04')
        """,
    )

    rule = "sprawdzenie_kartoteki_w_komplecie"
    room.refused(
        rule,
        "odnotuj_sprawdzenie_kartoteki",
        id=1,
        kartoteka_wynik="jeden",
        sprawdzono_kartoteke="2026-11-04T09:40:00Z",
    )
    room.refused(
        rule,
        "odnotuj_sprawdzenie_kartoteki",
        id=1,
        kartoteka_wynik="brak",
        kartoteka_odnosnik="pokoj-kartoteki/klient-nowak",
        sprawdzono_kartoteke="2026-11-04T09:40:00Z",
    )
    check_client_register(
        room, kartoteka_wynik="jeden", kartoteka_odnosnik="pokoj-kartoteki/klient-nowak"
    )
    room.refused(
        "pól karty ani sprawdzenia w kartotece się nie zmienia",
        "odnotuj_sprawdzenie_kartoteki",
        id=1,
        kartoteka_wynik="brak",
        sprawdzono_kartoteke="2026-11-04T09:50:00Z",
    )
    room.refused_sql(
        "pól karty ani sprawdzenia w kartotece się nie zmienia",
        "UPDATE kierownik_biura_przydzial_karty SET miejscowosc = 'Opole' WHERE id = 1",
    )


@scenario("przydzielone zgłoszenie ma dział i sprawdzenie w kartotece, a kontakt znika")
def assigned_card_has_department_and_check():
    room = new_room()
    add_rule(room)
    take_card(room)

    room.refused(
        "przydzielone_zgloszenie_sprawdzone_w_kartotece",
        "przydziel_karte",
        id=1,
        dzial="posrednictwo",
        regula_id=1,
        rozstrzygnieto="2026-11-04T10:00:00Z",
    )
    check_client_register(room)
    room.refused(
        "przydzielona_z_dzialem",
        "przydziel_karte",
        id=1,
        rozstrzygnieto="2026-11-04T10:00:00Z",
        **human_approval(),
    )
    room.refused(
        "rozstrzygnieta_z_data",
        "przydziel_karte",
        id=1,
        dzial="posrednictwo",
        regula_id=1,
    )
    room.refused(
        "rozstrzygniecie_nie_przed_wplywem",
        "przydziel_karte",
        id=1,
        dzial="posrednictwo",
        regula_id=1,
        rozstrzygnieto="2026-11-04T08:00:00Z",
    )
    room.refused_sql(
        "rozstrzygnieta_bez_kontaktu",
        """
        UPDATE kierownik_biura_przydzial_karty
        SET stan = 'przydzielona', dzial = 'posrednictwo', regula_id = 1,
          rozstrzygnieto = '2026-11-04T10:00:00Z'
        WHERE id = 1
        """,
    )
    room.run(
        "przydziel_karte",
        id=1,
        dzial="posrednictwo",
        regula_id=1,
        rozstrzygnieto="2026-11-04T10:00:00Z",
    )
    room.refused(
        "rozstrzygniętej karty wcale",
        "odrzuc_karte",
        id=1,
        powod="pomyłka",
        rozstrzygnieto="2026-11-05T10:00:00Z",
        **human_approval(),
    )
    assert len(room.run("karty_rozstrzygniete_od", od="2026-11-04")) == 1


@scenario("przydział według reguły idzie do jej działu, dla jej rodzaju, gdy była aktualna")
def assignment_follows_the_rule():
    room = new_room()
    add_rule(room)
    room.run("zastap_regule_przydzialu", id=1, zastapiona_od="2026-11-10T00:00:00Z")
    add_rule(room, dzial="najem", od="2026-11-10T00:00:00Z")
    take_card(room)
    take_card(room, karta_odnosnik="pokoj-recepcji/karta-2", rodzaj_sprawy="kupno")
    check_client_register(room, card_id=1)
    check_client_register(room, card_id=2)

    rule = "przydział według reguły idzie do jej działu"
    room.refused(
        rule,
        "przydziel_karte",
        id=1,
        dzial="najem",
        regula_id=1,
        rozstrzygnieto="2026-11-04T10:00:00Z",
    )
    room.refused(
        rule,
        "przydziel_karte",
        id=1,
        dzial="najem",
        regula_id=2,
        rozstrzygnieto="2026-11-04T10:00:00Z",
    )
    room.refused(
        rule,
        "przydziel_karte",
        id=2,
        dzial="posrednictwo",
        regula_id=1,
        rozstrzygnieto="2026-11-04T10:00:00Z",
    )
    room.run(
        "przydziel_karte",
        id=1,
        dzial="posrednictwo",
        regula_id=1,
        rozstrzygnieto="2026-11-04T10:00:00Z",
    )


@scenario("przydział poza regułami i odrzucenie zgłoszenia klienta wymagają zgody człowieka")
def human_approves_exceptions():
    room = new_room()
    take_card(room)
    take_card(room, karta_odnosnik="pokoj-recepcji/karta-2")
    take_card(
        room,
        karta_odnosnik="pokoj-recepcji/karta-3",
        rodzaj_karty="pismo_bez_sprawy",
        rodzaj_sprawy=None,
        kontakt=None,
    )
    check_client_register(room, card_id=1)

    room.refused(
        "przydzial_poza_regulami_za_zgoda_czlowieka",
        "przydziel_karte",
        id=1,
        dzial="transakcje",
        rozstrzygnieto="2026-11-04T10:00:00Z",
    )
    room.run("przekaz_karte_czlowiekowi", id=1)
    room.run(
        "przydziel_karte",
        id=1,
        dzial="transakcje",
        rozstrzygnieto="2026-11-04T10:00:00Z",
        **human_approval(),
    )

    room.refused(
        "odrzucenie_zgloszenia_za_zgoda_czlowieka",
        "odrzuc_karte",
        id=2,
        powod="Poza zakresem biura",
        rozstrzygnieto="2026-11-04T10:00:00Z",
    )
    room.refused(
        "odrzucona_z_powodem",
        "odrzuc_karte",
        id=3,
        rozstrzygnieto="2026-11-04T10:00:00Z",
    )
    room.run("odrzuc_karte", id=3, powod="Reklama usług", rozstrzygnieto="2026-11-04T10:00:00Z")

    room.refused(
        "zatwierdzenie_w_komplecie",
        "odrzuc_karte",
        id=2,
        powod="Poza zakresem biura",
        rozstrzygnieto="2026-11-04T10:00:00Z",
        zatwierdzil="tozsamosc-wlasciciela-biura",
    )
    room.run(
        "odrzuc_karte",
        id=2,
        powod="Poza zakresem biura",
        rozstrzygnieto="2026-11-04T10:00:00Z",
        **human_approval(),
    )


# Reports from the departments --------------------------------------------------------------------


@scenario("meldunek ma pola działu albo pola Recepcji i tylko się go dopisuje")
def report_fields_by_sender():
    room = new_room()
    add_department_report(room)

    room.refused(
        "pola_meldunku_dzialu",
        "dopisz_meldunek_dzialu",
        nadawca="najem",
        tozsamosc="t",
        otrzymano="2026-11-04T16:00:00Z",
        sprawy_zablokowane=0,
        decyzje_czekaja=0,
    )
    room.refused_sql(
        "pola_meldunku_dzialu",
        """
        INSERT INTO kierownik_biura_meldunki_wpisy (nadawca, tozsamosc, otrzymano, najblizszy_termin,
          wiadomosci_wplynely, karty_wyslane, zaklocenia)
        VALUES ('recepcja', 't', '2026-11-04', '2026-11-06', 1, 0, 0)
        """,
    )
    room.refused_sql(
        "pola_meldunku_recepcji",
        """
        INSERT INTO kierownik_biura_meldunki_wpisy (nadawca, tozsamosc, otrzymano, agenci_pracuja,
          sprawy_zablokowane, decyzje_czekaja, zaklocenia)
        VALUES ('najem', 't', '2026-11-04', 1, 0, 0, 2)
        """,
    )
    room.refused_sql(
        "meldunki tylko się dopisuje",
        "UPDATE kierownik_biura_meldunki_wpisy SET agenci_pracuja = 1 WHERE id = 1",
    )
    assert len(room.run("meldunki_od", od="2026-11-01")) == 1


# Decision lists ------------------------------------------------------------------------------------


@scenario("prośba jest na liście raz; ponowna odnawia pozycję, a po zdjęciu wraca nowa")
def request_once_on_the_list():
    room = new_room()
    add_position(room)

    room.refused("ta prośba jest już na liście", "przyjmij_pozycje", **position_fields())
    room.refused(
        "odnowienie_nie_przed_przyjeciem",
        "odnow_pozycje",
        id=1,
        opis="Nowy opis",
        odnowiono="2026-11-03T10:00:00Z",
    )
    room.run("odnow_pozycje", id=1, opis="Nowy opis", odnowiono="2026-11-05T10:00:00Z")

    room.refused("zdjecie_z_powodem", "zdejmij_pozycje", id=1, zdjeto="2026-11-06T10:00:00Z")
    room.refused(
        "zdjecie_nie_przed_przyjeciem",
        "zdejmij_pozycje",
        id=1,
        zdjeto="2026-11-01T10:00:00Z",
        powod_zdjecia="zbedna",
    )
    room.run("zdejmij_pozycje", id=1, zdjeto="2026-11-06T10:00:00Z", powod_zdjecia="rozstrzygnieta")
    room.run("przyjmij_pozycje", **position_fields(przyjeto="2026-11-07T10:00:00Z"))
    assert len(room.run("pozycje_zdjete_od", od="2026-11-01")) == 1
    assert [row["id"] for row in room.run("pozycje_czekajace_od", przed="2026-11-08")] == [2]


def position_fields(**fields):
    return {
        "prosba_odnosnik": "pokoj-teczki-1/decyzja-1",
        "dzial": "posrednictwo",
        "decyduje": "tozsamosc-posrednika",
        "opis": "Zgoda",
        "przyjeto": "2026-11-04T10:00:00Z",
        **fields,
    }


@scenario("pozycja powstaje niewysłana, a jej prośby, działu ani człowieka się nie zmienia")
def position_does_not_change():
    room = new_room()
    add_position(room)

    room.refused_sql(
        "pozycja powstaje na liście",
        """
        INSERT INTO kierownik_biura_decyzje_pozycje (prosba_odnosnik, dzial, decyduje, opis, przyjeto,
          trafila_do_czlowieka)
        VALUES ('pokoj-teczki-2/decyzja-1', 'najem', 'd', 'o', '2026-11-04', '2026-11-04')
        """,
    )

    rule = "prośby, działu, człowieka ani przyjęcia pozycji się nie zmienia"
    room.refused_sql(
        rule, "UPDATE kierownik_biura_decyzje_pozycje SET decyduje = 'inny' WHERE id = 1"
    )
    room.refused(
        "do_czlowieka_nie_przed_przyjeciem",
        "odnotuj_wyslanie_pozycji",
        id=1,
        trafila_do_czlowieka="2026-11-03T10:00:00Z",
    )
    room.run("odnotuj_wyslanie_pozycji", id=1, trafila_do_czlowieka="2026-11-04T10:05:00Z")
    room.refused(
        rule, "odnotuj_wyslanie_pozycji", id=1, trafila_do_czlowieka="2026-11-04T15:00:00Z"
    )
    room.run("odnotuj_osobne_zgloszenie", id=1, zgloszono_osobno="2026-11-07T09:00:00Z")
    room.refused(rule, "odnotuj_osobne_zgloszenie", id=1, zgloszono_osobno="2026-11-08T09:00:00Z")

    room.run("zdejmij_pozycje", id=1, zdjeto="2026-11-08T10:00:00Z", powod_zdjecia="zbedna")
    room.refused(rule, "odnow_pozycje", id=1, opis="Znowu", odnowiono="2026-11-09T10:00:00Z")


# Absences ------------------------------------------------------------------------------------------


@scenario("nieobecność ma okres, innego zastępcę i podpis człowieka albo właściciela biura")
def absence_is_signed():
    room = new_room()
    add_absence(room)
    add_absence(
        room,
        czlowiek="tozsamosc-pracownika",
        podpisal="tozsamosc-wlasciciela-biura",
        podpisal_jako="wlasciciel_biura",
        podpis="podpis-kluczem-wlasciciela-biura",
    )

    room.refused(
        "nieobecnosc_od_przed_do", "zapisz_nieobecnosc", **absence_fields(od_dnia="2026-11-15")
    )
    room.refused(
        "zastepca_to_ktos_inny",
        "zapisz_nieobecnosc",
        **absence_fields(zastepca="tozsamosc-posrednika"),
    )
    room.refused(
        "podpis_czlowieka_ktorego_dotyczy",
        "zapisz_nieobecnosc",
        **absence_fields(podpisal="tozsamosc-kogos-innego"),
    )
    room.refused_sql(
        "nieobecność powstaje nieodwołana",
        """
        INSERT INTO kierownik_biura_nieobecnosci_wpisy (czlowiek, od_dnia, do_dnia, zastepca, podpisal,
          podpisal_jako, podpis, wiadomosc_odnosnik, zapisano, odwolal, odwolanie_podpis, odwolano)
        VALUES ('c', '2026-11-10', '2026-11-11', 'z', 'c', 'czlowiek', 'p', 'w', '2026-11-04', 'c',
          'p', '2026-11-05')
        """,
    )
    assert len(room.run("nieobecnosci_od", od="2026-11-01")) == 2


def absence_fields(**fields):
    return {
        "czlowiek": "tozsamosc-posrednika",
        "od_dnia": "2026-11-10",
        "do_dnia": "2026-11-14",
        "zastepca": "tozsamosc-wspolnika",
        "podpisal": "tozsamosc-posrednika",
        "podpisal_jako": "czlowiek",
        "podpis": "p",
        "wiadomosc_odnosnik": "w",
        "zapisano": "2026-11-04T11:00:00Z",
        **fields,
    }


@scenario(
    "nieobecność odwołuje podpisem człowiek albo właściciel biura, raz, i się jej nie zmienia"
)
def absence_is_withdrawn_once_by_its_signer():
    room = new_room()
    add_absence(room)

    room.refused_sql(
        "nieobecności się nie zmienia",
        "UPDATE kierownik_biura_nieobecnosci_wpisy SET zastepca = 'ktos-inny' WHERE id = 1",
    )
    room.refused(
        "odwolanie_z_podpisem",
        "odwolaj_nieobecnosc",
        id=1,
        odwolal="tozsamosc-posrednika",
        odwolano="2026-11-05T09:00:00Z",
    )
    room.refused(
        "nieobecność odwołuje człowiek, którego dotyczy",
        "odwolaj_nieobecnosc",
        id=1,
        odwolal="tozsamosc-obcego",
        odwolanie_podpis="p",
        odwolano="2026-11-05T09:00:00Z",
    )
    room.run(
        "odwolaj_nieobecnosc",
        id=1,
        odwolal="tozsamosc-posrednika",
        odwolanie_podpis="podpis-kluczem-posrednika",
        odwolano="2026-11-05T09:00:00Z",
    )
    room.refused(
        "nieobecności się nie zmienia",
        "odwolaj_nieobecnosc",
        id=1,
        odwolal="tozsamosc-posrednika",
        odwolanie_podpis="p",
        odwolano="2026-11-06T09:00:00Z",
    )
    assert room.run("nieobecnosci_w_dniu", dzien="2026-11-12") == []


# Policies ------------------------------------------------------------------------------------------


@scenario("o polisie zawiadamia się właściwego kierownika, raz, a jej danych się nie zmienia")
def policy_notice_to_the_right_head():
    room = new_room()
    add_policy(room)
    add_policy(room, rodzaj="zarzadcy", numer="OC-Z/2027/001")

    room.refused("okres_od_przed_do", "zapisz_polise", **policy_fields(okres_do="2026-12-31"))
    room.refused_sql(
        "polisa powstaje bez przypomnienia i zawiadomienia",
        """
        INSERT INTO kierownik_biura_polisy_wpisy (rodzaj, numer, ubezpieczyciel, okres_od, okres_do,
          dokument_odnosnik, zapisano, przypomniano)
        VALUES ('posrednika', 'X', 'U', '2027-01-01', '2027-12-31', 'd', '2026-12-20', '2026-12-21')
        """,
    )

    rule = "zawiadomiony_kierownik_wlasciwy_dla_polisy"
    room.refused(
        rule,
        "odnotuj_zawiadomienie_o_polisie",
        id=1,
        zawiadomiono="2026-12-20T10:00:00Z",
        zawiadomiono_kogo="kierownik_najmu",
    )
    room.refused(
        rule,
        "odnotuj_zawiadomienie_o_polisie",
        id=2,
        zawiadomiono="2026-12-20T10:00:00Z",
        zawiadomiono_kogo="kierownik_posrednictwa",
    )
    room.refused(
        "zawiadomienie_w_komplecie",
        "odnotuj_zawiadomienie_o_polisie",
        id=2,
        zawiadomiono="2026-12-20T10:00:00Z",
    )
    room.run(
        "odnotuj_zawiadomienie_o_polisie",
        id=2,
        zawiadomiono="2026-12-20T10:00:00Z",
        zawiadomiono_kogo="kierownik_najmu",
    )

    rule = "danych polisy się nie zmienia"
    room.refused(
        rule,
        "odnotuj_zawiadomienie_o_polisie",
        id=2,
        zawiadomiono="2026-12-21T10:00:00Z",
        zawiadomiono_kogo="kierownik_najmu",
    )
    room.refused_sql(
        rule, "UPDATE kierownik_biura_polisy_wpisy SET okres_do = '2028-01-31' WHERE id = 1"
    )
    room.run("odnotuj_przypomnienie_o_polisie", id=1, przypomniano="2027-12-01T09:00:00Z")
    room.refused(rule, "odnotuj_przypomnienie_o_polisie", id=1, przypomniano="2027-12-02T09:00:00Z")
    assert len(room.run("polisy")) == 2


def policy_fields(**fields):
    return {
        "rodzaj": "posrednika",
        "numer": "OC/2027/002",
        "ubezpieczyciel": "U",
        "okres_od": "2027-01-01",
        "okres_do": "2027-12-31",
        "dokument_odnosnik": "d",
        "zapisano": "2026-12-20T09:00:00Z",
        **fields,
    }


# Reports to the office owner -----------------------------------------------------------------------


@scenario("sprawozdanie powstaje robocze, a przekazanego się nie zmienia")
def handed_over_report_does_not_change():
    room = new_room()
    add_report(room)

    room.refused(
        "okres_od_nie_po_do", "zapisz_sprawozdanie", **report_fields(okres_od="2026-12-01")
    )
    room.refused_sql(
        "sprawozdanie powstaje robocze",
        """
        INSERT INTO kierownik_biura_sprawozdania_wpisy (rodzaj, okres_od, okres_do, sciezka, skrot,
          sporzadzono, przekazano)
        VALUES ('na_zadanie', '2026-11-01', '2026-11-15', 's', 'k', '2026-11-16', '2026-11-16')
        """,
    )
    room.run(
        "zmien_sprawozdanie",
        **report_fields(id=1, sciezka="sprawozdania/2026-11-v2.typ", skrot="sha256-v2"),
    )
    room.refused(
        "przekazanie_nie_przed_sporzadzeniem",
        "przekaz_sprawozdanie",
        id=1,
        przekazano="2026-11-30T09:00:00Z",
    )
    room.run("przekaz_sprawozdanie", id=1, przekazano="2026-12-01T12:00:00Z")
    room.refused(
        "przekazanego sprawozdania się nie zmienia",
        "zmien_sprawozdanie",
        **report_fields(id=1, skrot="sha256-v3"),
    )
    assert room.run("sprawozdania")[0]["skrot"] == "sha256-v2"


def report_fields(**fields):
    return {
        "rodzaj": "miesieczne",
        "okres_od": "2026-11-01",
        "okres_do": "2026-11-30",
        "sciezka": "sprawozdania/2026-11.typ",
        "skrot": "sha256",
        "sporzadzono": "2026-12-01T09:00:00Z",
        **fields,
    }


# R9 and replacement --------------------------------------------------------------------------------


def own_tables_of(room):
    return [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' "
            "AND name LIKE 'kierownik\\_biura\\_%' ESCAPE '\\'"
        )
    ]


@scenario("R9: wierszy pamięci się nie usuwa ani nie zastępuje przez REPLACE albo upsert")
def no_deletion_or_replacement():
    room = new_room()
    full_memory(room)

    tables = own_tables_of(room)
    assert len(tables) == 7, tables

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


@scenario("REPLACE nie usuwa aktualnej reguły, karty ani niezdjętej pozycji przez klucz UNIQUE")
def no_replacement_through_unique_keys():
    room = new_room()
    full_memory(room)

    for table, key, refusal in [
        ("kierownik_biura_przydzial_karty", "karta_odnosnik", REPLACEMENT),
        (
            "kierownik_biura_przydzial_reguly",
            "rodzaj_zgloszenia",
            "ten rodzaj zgłoszenia ma już aktualną regułę",
        ),
        (
            "kierownik_biura_decyzje_pozycje",
            "prosba_odnosnik",
            "ta prośba jest już na liście",
        ),
    ]:
        columns = [row["name"] for row in room.execute(f"PRAGMA table_info({table})")]
        selected = ", ".join("99" if name == "id" else name for name in columns)
        room.refused_sql(
            refusal,
            f"INSERT OR REPLACE INTO {table} ({', '.join(columns)}) SELECT {selected} FROM {table}",
        )
        if refusal == REPLACEMENT:
            room.refused_sql(
                refusal,
                # `WHERE true` keeps SQLite from reading ON CONFLICT as a join constraint.
                f"INSERT INTO {table} ({', '.join(columns)}) SELECT {selected} FROM {table} "
                f"WHERE true ON CONFLICT ({key}) DO UPDATE SET id = excluded.id",
            )
