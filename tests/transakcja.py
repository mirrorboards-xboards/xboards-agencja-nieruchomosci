"""Scenarios of the sale case room (spec/cases/transakcja on the core).

Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
allows and a write it refuses. The first two scenarios follow a whole sale as the profile describes
it: a flat sold with a preliminary agreement and a mortgage, and a plot sold in two notarial deeds
because the municipality has a right of pre-emption.
"""

from pathlib import Path

from core import (
    add_account,
    add_flat,
    add_inflow,
    add_person,
    add_received_document,
    approved_decision,
    check,
    confirm_account,
    open_case,
)
from core_porzadek import first_row_copy, unique_keys_of
from room import Room

CASE = Path(__file__).resolve().parents[1] / "spec" / "cases" / "transakcja"

SCENARIOS = []


def scenario(title):
    def register(function):
        SCENARIOS.append((title, function))

        return function

    return register


def new_room():
    return Room(CASE)


# Building blocks of a sale ------------------------------------------------------------------------


def open_sale(room):
    open_case(room, typ_pokoju="transakcja")


def move_to(room, *etapy, decyzja_id=None):
    for etap in etapy:
        room.run(
            "zmien_etap_sprawy",
            etap=etap,
            od="2026-11-05T09:00:00Z",
            zmienil="tozsamosc-koordynatora",
            decyzja_id=decyzja_id,
        )


def add_parties(room):
    """The seller, a client of the office, and the buyer with his wife, who consents."""
    add_person(room, "sprzedajaca", konsument=1, klient_biura=1)
    add_person(room, "kupujacy", imiona="Jan", nazwisko="Nowak")
    add_person(
        room, "zona-kupujacego", imiona="Ewa", nazwisko="Nowak", stan_cywilny="w_malzenstwie"
    )
    room.run("nadaj_role_uczestnikowi", uczestnik_id="sprzedajaca", rola="sprzedajacy")
    room.run("nadaj_role_uczestnikowi", uczestnik_id="kupujacy", rola="kupujacy")
    room.run(
        "nadaj_role_uczestnikowi",
        uczestnik_id="zona-kupujacego",
        rola="malzonek",
        dotyczy_uczestnik_id="kupujacy",
    )


def add_notary_office(room, office_id="kancelaria", xnotariusz=1, od="2026-11-04"):
    room.run(
        "wpisz_kancelarie",
        id=office_id,
        nazwa="Kancelaria Notarialna Maria Wiśniewska",
        adres="ul. Rynek 1, Wrocław",
        notariusz="notariusz Maria Wiśniewska",
        xnotariusz=xnotariusz,
        od=od,
    )


def open_exchange_room(room, office_id="kancelaria"):
    """A human approves the exchange room with the notary office, then it opens."""
    approved_decision(
        room,
        f"otwarcie-{office_id}",
        "inna",
        "transakcja_umowy_kancelarie",
        office_id,
    )
    room.run(
        "otworz_pokoj_wymiany",
        id=office_id,
        otwarcie_decyzja_id=f"otwarcie-{office_id}",
        pokoj_odnosnik=f"pokoj-wymiany-{office_id}",
    )


def admitted_account_for(room, account_id, person_id, cel):
    """An account for a payment, confirmed by another channel, checked and admitted (R4)."""
    add_account(room, account_id, person_id, cel=cel, posiadacz="Anna Kowalska")
    confirm_account(room, account_id)
    check(room, f"sprawdzenie-{account_id}", "rachunek", rachunek_id=account_id)
    approved_decision(
        room,
        f"dopuszczenie-{account_id}",
        "dopuszczenie_rachunku",
        "sprawa_rachunki_wpisy",
        account_id,
    )


def prepare_agreement(room, agreement_id, rodzaj="przedwstepna", forma="pisemna", **fields):
    add_received_document(room, f"projekt-{agreement_id}", "umowa")
    room.run(
        "przygotuj_umowe_transakcji",
        **{
            "id": agreement_id,
            "rodzaj": rodzaj,
            "forma": forma,
            "projekt_dokument_id": f"projekt-{agreement_id}",
            "cena": 62_000_000,
            "termin_wydania": "14 dni od zapłaty całej ceny",
            **fields,
        },
    )


def analyse_draft(room, agreement_id, analysis_id=None, dokument_id=None):
    """The legal analyst's analysis of the agreement's current draft."""
    analysis_id = analysis_id or f"analiza-{agreement_id}"
    dokument_id = dokument_id or f"projekt-{agreement_id}"
    room.run(
        "wpisz_analize_transakcji",
        id=analysis_id,
        przedmiot="projekt_umowy",
        umowa_id=agreement_id,
        dokument_id=dokument_id,
        sporzadzono="2026-11-06T09:00:00Z",
        wnioski="Projekt zgodny z księgą; zadatek w rozumieniu art. 394 KC.",
        sciezka=f"analizy/{analysis_id}.typ",
        skrot=f"sha256-{analysis_id}",
    )


def conclude_agreement(room, agreement_id, forma="pisemna", zawarto="2026-11-10T12:00:00Z"):
    is_deed = forma == "akt_notarialny"
    add_received_document(
        room, f"podpisana-{agreement_id}", "akt_notarialny" if is_deed else "umowa"
    )
    room.run(
        "odnotuj_zawarcie_umowy_transakcji",
        id=agreement_id,
        dokument_id=f"podpisana-{agreement_id}",
        zawarto=zawarto,
        repertorium="Rep. A nr 1234/2026" if is_deed else None,
    )


def concluded_agreement(room, agreement_id, rodzaj="przedwstepna", forma="pisemna", **fields):
    """An agreement whose draft the analyst assessed and the parties signed."""
    prepare_agreement(room, agreement_id, rodzaj, forma, **fields)
    analyse_draft(room, agreement_id)
    conclude_agreement(room, agreement_id, forma)


def concluded_deed(room, agreement_id, rodzaj="przenoszaca", **fields):
    """A notarial deed signed at the notary office of the case."""
    concluded_agreement(
        room, agreement_id, rodzaj, "akt_notarialny", kancelaria_id="kancelaria", **fields
    )


def add_payment(room, payment_id, agreement_id, rodzaj="zadatek", **fields):
    room.run(
        "wpisz_platnosc_transakcji",
        **{
            "id": payment_id,
            "umowa_id": agreement_id,
            "rodzaj": rodzaj,
            "kwota": 6_000_000,
            "termin": "2026-11-12",
            "zrodlo": "wlasne",
            **fields,
        },
    )


def record_payment(room, payment_id):
    add_received_document(room, f"potwierdzenie-{payment_id}", "potwierdzenie_bankowe")
    room.run(
        "odnotuj_wplate_platnosci",
        id=payment_id,
        wplacono="2026-11-12T10:00:00Z",
        potwierdzenie_dokument_id=f"potwierdzenie-{payment_id}",
    )


def add_right_of_preemption(room, preemption_id="pierwokup-gminy"):
    room.run(
        "wpisz_pierwokup",
        id=preemption_id,
        uprawniony="Gmina Wrocław",
        podstawa="art. 109 ust. 1 pkt 3 ugn",
    )


def record_preemption_result(room, preemption_id="pierwokup-gminy", wynik="nie_wykonano"):
    add_inflow(
        room, f"wiadomosc-{preemption_id}", nadawca="Kancelaria Notarialna", kanal="pokoj_wymiany"
    )
    room.run("odnotuj_zawiadomienie_o_pierwokupie", id=preemption_id, zawiadomiono="2026-11-11")
    room.run(
        "odnotuj_wynik_pierwokupu",
        id=preemption_id,
        wynik=wynik,
        zrodlo_wplyw_id=f"wiadomosc-{preemption_id}",
    )


def add_deed_power(room, power_id="pelnomocnictwo-kupujacego", forma="akt_notarialny"):
    """The buyer's power of attorney for his brother, who signs the deed for him."""
    add_person(room, "brat-kupujacego", imiona="Piotr", nazwisko="Nowak")
    add_received_document(room, f"dokument-{power_id}", "pelnomocnictwo")
    room.run(
        "wpisz_umocowanie",
        id=power_id,
        rodzaj="pelnomocnictwo",
        mocodawca_uczestnik_id="kupujacy",
        umocowany_uczestnik_id="brat-kupujacego",
        umocowane_biuro=0,
        zakres="do_czynnosci",
        forma=forma,
        dokument_id=f"dokument-{power_id}",
        udzielono="2026-11-03",
    )


def signed_handover_protocol(room, property_id="lokal"):
    room.run(
        "przygotuj_protokol",
        id="protokol-przekazania",
        rodzaj="przekazanie",
        nieruchomosc_id=property_id,
        chwila="2026-11-20T10:00:00Z",
    )
    room.run(
        "dopisz_pozycje_protokolu",
        protokol_id="protokol-przekazania",
        rodzaj="klucze",
        nazwa="klucze do drzwi wejściowych",
        opis_stanu="3 sztuki",
    )
    add_received_document(room, "podpisany-protokol-przekazania", "protokol")
    room.run(
        "podpisz_protokol",
        id="protokol-przekazania",
        dokument_id="podpisany-protokol-przekazania",
    )


def full_sale_of_flat(room):
    """A flat sold with a reservation-free preliminary agreement, a deposit and a mortgage."""
    open_sale(room)
    add_parties(room)
    add_flat(room)
    room.run(
        "wpisz_powiazanie",
        rodzaj="sprawa_zrodlowa",
        typ_teczki="oferta",
        oznaczenie="Oferta 2026/041",
        odnosnik="pokoj-oferty-041",
        wpisal="tozsamosc-kierownika-transakcji",
        wpisano="2026-11-02T09:15:00Z",
    )

    # Preparation: the analyst reads the land register; the notary office gets an exchange room.
    move_to(room, "przygotowanie")
    add_received_document(
        room, "odpis-kw", "odpis_ksiegi_wieczystej", dotyczy_nieruchomosc_id="lokal"
    )
    room.run(
        "wpisz_analize_transakcji",
        id="analiza-ksiegi",
        przedmiot="ksiega",
        dokument_id="odpis-kw",
        sporzadzono="2026-11-04T09:00:00Z",
        wnioski="Dział III i IV bez wpisów; sprzedająca jedyną właścicielką.",
        sciezka="analizy/ksiega.typ",
        skrot="sha256-ksiega",
    )
    room.run(
        "dopisz_ryzyko_transakcji",
        analiza_id="analiza-ksiegi",
        opis="Wzmianka o wniosku w dziale III",
        skutek="Akt dopiero po wyjaśnieniu wzmianki",
    )
    add_notary_office(room)
    open_exchange_room(room)
    add_inflow(room, "zgoda-na-informacje", "kupujacy")
    room.run(
        "wpisz_zgode",
        id="informacje-od-kancelarii",
        uczestnik_id="kupujacy",
        rodzaj="informacje_od_kancelarii",
        tresc="Zgadzam się, żeby biuro dostawało projekt aktu od kancelarii.",
        zlozono="2026-11-04T10:00:00Z",
        wplyw_id="zgoda-na-informacje",
        trwaly_nosnik=1,
    )

    # The preliminary agreement in writing with a deposit to the seller's admitted account.
    move_to(room, "umowa_przedwstepna")
    prepare_agreement(room, "przedwstepna", termin_przyrzeczonej="2026-12-15")
    for party, side in (("sprzedajaca", "sprzedajacy"), ("kupujacy", "kupujacy")):
        room.run(
            "dopisz_strone_umowy_transakcji",
            umowa_id="przedwstepna",
            uczestnik_id=party,
            strona=side,
        )
    admitted_account_for(room, "rachunek-zadatku", "sprzedajaca", "zadatek")
    add_payment(room, "zadatek", "przedwstepna", rachunek_id="rachunek-zadatku")
    analyse_draft(room, "przedwstepna")
    conclude_agreement(room, "przedwstepna")
    record_payment(room, "zadatek")

    # Preparing the deed: the date confirmed by a human, the price from the buyer's loan.
    move_to(room, "przygotowanie_aktu")
    room.run(
        "wpisz_termin",
        id="termin-aktu",
        rodzaj="z_umowy",
        co="Akt notarialny umowy sprzedaży",
        koniec="2026-12-10T10:00:00Z",
        podstawa="§ 2 umowy przedwstępnej",
        pilnuje_stanowisko="koordynator_transakcji",
        kalendarz_odnosnik="kalendarz/akt-polna-7",
    )
    approved_decision(
        room,
        "potwierdzenie-aktu",
        "potwierdzenie_terminu",
        "sprawa_terminy_wpisy",
        "termin-aktu",
    )
    room.run(
        "odnotuj_potwierdzenie_terminu",
        id="termin-aktu",
        potwierdzenie_decyzja_id="potwierdzenie-aktu",
    )
    prepare_agreement(
        room,
        "akt",
        "przenoszaca",
        "akt_notarialny",
        oparta_na_id="przedwstepna",
        kancelaria_id="kancelaria",
        termin_zaplaty="2026-12-17",
    )
    add_deed_power(room)
    room.run(
        "dopisz_strone_umowy_transakcji",
        umowa_id="akt",
        uczestnik_id="sprzedajaca",
        strona="sprzedajacy",
    )
    room.run(
        "dopisz_strone_umowy_transakcji",
        umowa_id="akt",
        uczestnik_id="kupujacy",
        strona="kupujacy",
        umocowanie_id="pelnomocnictwo-kupujacego",
    )
    room.run(
        "dopisz_strone_umowy_transakcji",
        umowa_id="akt",
        uczestnik_id="zona-kupujacego",
        strona="malzonek",
    )
    admitted_account_for(room, "rachunek-ceny", "sprzedajaca", "zaplata_ceny")
    add_payment(
        room,
        "cena-z-kredytu",
        "akt",
        rodzaj="cena",
        kwota=56_000_000,
        zrodlo="kredyt",
        bank="Bank Przykładowy S.A.",
        warunki_wyplaty="po przedstawieniu wypisu aktu i wniosku o wpis hipoteki",
        rachunek_id="rachunek-ceny",
    )
    analyse_draft(room, "akt")

    # The deed: the notary files the land register application on the same day.
    move_to(room, "akt")
    conclude_agreement(room, "akt", "akt_notarialny", zawarto="2026-12-10T11:00:00Z")
    room.run("odnotuj_wniosek_do_ksiegi", id="akt", wniosek_kw_zlozono="2026-12-10")
    record_payment(room, "cena-z-kredytu")

    # Handover with a signed protocol, then the buyer's entry in the land register.
    move_to(room, "przekazanie")
    signed_handover_protocol(room)
    move_to(room, "po_przekazaniu")
    room.run("odnotuj_wpis_do_ksiegi", id="akt", wpis_kw="2027-01-20")


def full_sale_with_preemption(room):
    """A plot the municipality may buy first: a conditional deed, then the transfer deed."""
    open_sale(room)
    add_parties(room)
    add_flat(
        room,
        "dzialka",
        rodzaj="dzialka",
        prawo="wlasnosc",
        numer_budynku=None,
        numer_lokalu=None,
        powierzchnia_uzytkowa=None,
        liczba_pokoi=None,
        pietro=None,
    )
    add_notary_office(room)
    add_right_of_preemption(room)
    room.run(
        "wpisz_analize_transakcji",
        id="analiza-ograniczen",
        przedmiot="ograniczenia",
        sporzadzono="2026-11-04T09:00:00Z",
        wnioski="Działka w planie na cele publiczne; gminie przysługuje pierwokup.",
        sciezka="analizy/ograniczenia.typ",
        skrot="sha256-ograniczenia",
    )
    room.run(
        "dopisz_ryzyko_transakcji",
        analiza_id="analiza-ograniczen",
        opis="Gmina może wykonać pierwokup",
        skutek="Dwa akty; sprzedaż bezwarunkowa byłaby nieważna (art. 599 § 2 KC)",
    )
    move_to(room, "przygotowanie", "przygotowanie_aktu", "akt_warunkowy")

    concluded_deed(room, "warunkowa", "warunkowa")
    record_preemption_result(room)

    move_to(room, "akt")
    prepare_agreement(
        room,
        "akt",
        "przenoszaca",
        "akt_notarialny",
        oparta_na_id="warunkowa",
        kancelaria_id="kancelaria",
    )
    room.run(
        "dopisz_strone_umowy_transakcji",
        umowa_id="akt",
        uczestnik_id="sprzedajaca",
        strona="sprzedajacy",
    )
    room.run(
        "dopisz_strone_umowy_transakcji", umowa_id="akt", uczestnik_id="kupujacy", strona="kupujacy"
    )
    admitted_account_for(room, "rachunek-ceny", "sprzedajaca", "zaplata_ceny")
    add_payment(room, "cena", "akt", rodzaj="cena", kwota=62_000_000, rachunek_id="rachunek-ceny")
    analyse_draft(room, "akt")
    conclude_agreement(room, "akt", "akt_notarialny", zawarto="2026-12-20T11:00:00Z")
    record_payment(room, "cena")
    room.run("odnotuj_wniosek_do_ksiegi", id="akt", wniosek_kw_zlozono="2026-12-20")


# The whole case ------------------------------------------------------------------------------------


@scenario(
    "Przebieg transakcji lokalu: przedwstępna z zadatkiem, akt z kredytem, przekazanie, księga"
)
def whole_sale_of_flat():
    room = new_room()

    full_sale_of_flat(room)
    move_to(room, "zamknieta")

    etapy = [row["etap"] for row in room.run("etapy")]
    assert etapy == [
        "przyjeta",
        "przygotowanie",
        "umowa_przedwstepna",
        "przygotowanie_aktu",
        "akt",
        "przekazanie",
        "po_przekazaniu",
        "zamknieta",
    ], etapy
    agreements = {row["id"]: row for row in room.run("umowy_transakcji")}
    assert agreements["przedwstepna"]["stan"] == "zawarta", agreements
    assert agreements["akt"]["wpis_kw"] == "2027-01-20", agreements
    assert all(row["wplacono"] is not None for row in room.run("platnosci_transakcji"))
    assert room.run("kancelarie")[0]["pokoj_odnosnik"] == "pokoj-wymiany-kancelaria"
    current = {row["przedmiot"] for row in room.run("analizy_aktualne_transakcji")}
    assert current == {"ksiega", "projekt_umowy"}, current


@scenario(
    "Przebieg transakcji z pierwokupem gminy: akt warunkowy, wynik pierwokupu, akt przenoszący"
)
def whole_sale_with_preemption():
    room = new_room()

    full_sale_with_preemption(room)

    etapy = [row["etap"] for row in room.run("etapy")]
    assert etapy[-2:] == ["akt_warunkowy", "akt"], etapy
    assert room.run("pierwokupy")[0]["wynik"] == "nie_wykonano"
    assert [row["rodzaj"] for row in room.run("umowy_transakcji")] == ["warunkowa", "przenoszaca"]


# Rules on the core -------------------------------------------------------------------------------


@scenario("Reguły: teczka transakcji ma typ pokoju transakcja (T2)")
def room_type():
    room = new_room()

    room.refused(
        "teczka transakcji ma typ pokoju transakcja",
        "zaloz_metryke",
        oznaczenie="Najem 2026/017",
        typ_pokoju="najem",
        przyjal="tozsamosc-posrednika",
        przyjeto="2026-11-02T09:00:00Z",
    )

    open_sale(room)
    assert room.run("metryka")[0]["typ_pokoju"] == "transakcja"


@scenario("Reguły: etapy transakcji są fazami i idą po kolei (T8)")
def stages_in_order():
    room = new_room()
    open_sale(room)

    room.refused(
        "etapy transakcji idą po kolei",
        "zmien_etap_sprawy",
        etap="umowa",
        od="2026-11-05",
        zmienil="agent",
    )
    room.refused(
        "etapy transakcji idą po kolei",
        "zmien_etap_sprawy",
        etap="akt",
        od="2026-11-05",
        zmienil="agent",
    )

    # Without a preliminary agreement the case goes straight to preparing the deed.
    move_to(room, "przygotowanie", "przygotowanie_aktu")
    room.refused(
        "etapy transakcji idą po kolei",
        "zmien_etap_sprawy",
        etap="umowa_przedwstepna",
        od="2026-11-05",
        zmienil="agent",
    )
    move_to(room, "akt")
    room.refused(
        "etapy transakcji idą po kolei",
        "zmien_etap_sprawy",
        etap="zamknieta",
        od="2026-11-05",
        zmienil="agent",
    )


@scenario(
    "Reguły: każdą fazę można przerwać decyzją, a po przerwaniu przychodzi zamknięcie albo powrót"
)
def stages_interrupted():
    room = new_room()
    open_sale(room)
    move_to(room, "przygotowanie", "umowa_przedwstepna")

    approved_decision(room, "przerwanie", "przerwanie_sprawy")
    move_to(room, "przerwana", decyzja_id="przerwanie")
    room.refused(
        "etapy transakcji idą po kolei",
        "zmien_etap_sprawy",
        etap="przygotowanie_aktu",
        od="2026-11-06",
        zmienil="agent",
    )
    move_to(room, "zamknieta")


@scenario("Reguły: wznowiona transakcja wraca do fazy sprzed przerwania i idzie dalej po kolei")
def stages_resumed():
    room = new_room()
    open_sale(room)
    move_to(room, "przygotowanie", "umowa_przedwstepna")
    approved_decision(room, "przerwanie", "przerwanie_sprawy")
    move_to(room, "przerwana", decyzja_id="przerwanie")
    approved_decision(room, "wznowienie", "wznowienie_sprawy", podpisano="2026-11-05T12:00:00Z")

    room.refused(
        "etapy transakcji idą po kolei",
        "zmien_etap_sprawy",
        etap="przygotowanie_aktu",
        od="2026-11-06",
        zmienil="agent",
        decyzja_id="wznowienie",
    )
    move_to(room, "umowa_przedwstepna", decyzja_id="wznowienie")
    move_to(room, "przygotowanie_aktu")


@scenario("Reguły: faza przekazania wymaga zawartej umowy przenoszącej, a po przekazaniu protokołu")
def handover_stage_rules():
    room = new_room()
    open_sale(room)
    add_parties(room)
    add_flat(room)
    add_notary_office(room)
    move_to(room, "przygotowanie", "przygotowanie_aktu", "akt")

    room.refused(
        "faza przekazania wymaga zawartej umowy przenoszącej własność",
        "zmien_etap_sprawy",
        etap="przekazanie",
        od="2026-12-10",
        zmienil="agent",
    )
    concluded_deed(room, "akt")
    move_to(room, "przekazanie")

    room.refused(
        "faza po przekazaniu wymaga podpisanego protokołu przekazania",
        "zmien_etap_sprawy",
        etap="po_przekazaniu",
        od="2026-12-20",
        zmienil="agent",
    )
    signed_handover_protocol(room)
    move_to(room, "po_przekazaniu")


@scenario("Reguły: role, zgody i brak umów z biurem w teczce transakcji")
def roles_consents_and_office_contracts():
    room = new_room()
    open_sale(room)
    add_parties(room)

    room.refused(
        "rola w teczce transakcji to sprzedajacy, kupujacy albo malzonek",
        "nadaj_role_uczestnikowi",
        uczestnik_id="kupujacy",
        rola="klient",
    )

    add_inflow(room, "zadanie", "sprzedajaca")
    consent = {
        "uczestnik_id": "sprzedajaca",
        "tresc": "Treść oświadczenia",
        "zlozono": "2026-11-04T10:00:00Z",
        "wplyw_id": "zadanie",
        "trwaly_nosnik": 1,
    }
    room.refused(
        "zgoda w teczce transakcji to informacje_od_kancelarii",
        "wpisz_zgode",
        id="zadanie-pracy",
        rodzaj="zadanie_rozpoczecia_pracy",
        **consent,
    )
    room.run("wpisz_zgode", id="informacje", rodzaj="informacje_od_kancelarii", **consent)

    room.refused(
        "w teczce transakcji nie ma umów z biurem",
        "przygotuj_umowe",
        id="umowa-posrednictwa",
        rodzaj="posrednictwo",
        obie_strony=0,
        wylacznosc=0,
    )


# Notary office -----------------------------------------------------------------------------------


@scenario("Kancelaria: pokój wymiany otwiera zatwierdzona decyzja o tej kancelarii w XNotariuszu")
def exchange_room_needs_decision():
    room = new_room()
    open_sale(room)
    room.refused_sql(
        "kancelaria powstaje bez pokoju wymiany i bez końca",
        """INSERT INTO transakcja_umowy_kancelarie (id, nazwa, xnotariusz, od, do, powod_zmiany)
        VALUES ('od-razu', 'Kancelaria', 1, '2026-11-04', '2026-11-05', 'pomyłka')""",
    )

    add_notary_office(room)

    room.refused(
        "pokój wymiany otwiera tylko zatwierdzona decyzja inna, która wskazuje tę kancelarię",
        "otworz_pokoj_wymiany",
        id="kancelaria",
        otwarcie_decyzja_id="brak",
        pokoj_odnosnik="pokoj",
    )
    approved_decision(room, "o-innej", "inna", "transakcja_umowy_kancelarie", "inna-kancelaria")
    room.refused(
        "pokój wymiany otwiera tylko zatwierdzona decyzja inna",
        "otworz_pokoj_wymiany",
        id="kancelaria",
        otwarcie_decyzja_id="o-innej",
        pokoj_odnosnik="pokoj",
    )

    open_exchange_room(room)
    assert room.run("kancelarie")[0]["pokoj_odnosnik"] == "pokoj-wymiany-kancelaria"

    # Outside XNotariusz there is no exchange room, even with a decision.
    room.run(
        "zakoncz_kancelarie", id="kancelaria", do="2026-11-06", powod_zmiany="strony wybrały inną"
    )
    add_notary_office(room, "poza-xnotariuszem", xnotariusz=0, od="2026-11-06")
    approved_decision(
        room, "otwarcie-poza", "inna", "transakcja_umowy_kancelarie", "poza-xnotariuszem"
    )
    room.refused(
        "pokoj_tylko_z_xnotariuszem",
        "otworz_pokoj_wymiany",
        id="poza-xnotariuszem",
        otwarcie_decyzja_id="otwarcie-poza",
        pokoj_odnosnik="pokoj",
    )


@scenario("Kancelaria: jedna bieżąca, zmiana kończy poprzednią z powodem, otwartej się nie zmienia")
def one_current_office():
    room = new_room()
    open_sale(room)
    add_notary_office(room)

    room.refused(
        "teczka ma jedną bieżącą kancelarię",
        "wpisz_kancelarie",
        id="druga",
        nazwa="Druga kancelaria",
        xnotariusz=1,
        od="2026-11-05",
    )
    open_exchange_room(room)
    room.refused_sql(
        "kancelarii z pokojem wymiany ani zakończonej się nie zmienia",
        "UPDATE transakcja_umowy_kancelarie SET notariusz = 'inny notariusz' WHERE id = 'kancelaria'",
    )
    room.refused(
        "koniec_z_powodem",
        "zakoncz_kancelarie",
        id="kancelaria",
        do="2026-11-06",
    )

    room.run("zakoncz_kancelarie", id="kancelaria", do="2026-11-06", powod_zmiany="notariusz chory")
    room.refused_sql(
        "kancelarii z pokojem wymiany ani zakończonej się nie zmienia",
        "UPDATE transakcja_umowy_kancelarie SET powod_zmiany = 'inny powód' WHERE id = 'kancelaria'",
    )
    add_notary_office(room, "druga", od="2026-11-06")


# Agreements ----------------------------------------------------------------------------------------


@scenario("Umowa: forma według rodzaju; warunkowa i przenosząca tylko w akcie (art. 158 KC)")
def agreement_form_by_kind():
    room = new_room()
    open_sale(room)

    room.refused(
        "forma_wedlug_rodzaju",
        "przygotuj_umowe_transakcji",
        id="a",
        rodzaj="przenoszaca",
        forma="pisemna",
    )
    room.refused(
        "forma_wedlug_rodzaju",
        "przygotuj_umowe_transakcji",
        id="b",
        rodzaj="warunkowa",
        forma="pisemna",
    )
    room.refused(
        "forma_wedlug_rodzaju",
        "przygotuj_umowe_transakcji",
        id="c",
        rodzaj="rezerwacyjna",
        forma="akt_notarialny",
    )

    room.run(
        "przygotuj_umowe_transakcji",
        id="przedwstepna-w-akcie",
        rodzaj="przedwstepna",
        forma="akt_notarialny",
    )
    room.run(
        "przygotuj_umowe_transakcji", id="rezerwacyjna", rodzaj="rezerwacyjna", forma="pisemna"
    )


@scenario(
    "Umowa: powstaje jako projekt i przechodzi do zawartej albo niedoszłej, zawarta do rozwiązanej"
)
def agreement_states():
    room = new_room()
    open_sale(room)

    room.refused_sql(
        "umowa transakcji powstaje jako projekt",
        """INSERT INTO transakcja_umowy_wpisy (id, rodzaj, forma, stan, koniec_powod)
        VALUES ('od-razu', 'przedwstepna', 'pisemna', 'rozwiazana', 'pomyłka')""",
    )

    prepare_agreement(room, "nie-dojdzie")
    room.refused("koniec_ma_powod", "odnotuj_ze_umowa_transakcji_nie_doszla", id="nie-dojdzie")
    room.run(
        "odnotuj_ze_umowa_transakcji_nie_doszla",
        id="nie-dojdzie",
        koniec_powod="bank odmówił kredytu",
    )
    room.refused_sql(
        "umowa transakcji przechodzi z projektu do zawartej albo niedoszłej",
        "UPDATE transakcja_umowy_wpisy SET stan = 'zawarta' WHERE id = 'nie-dojdzie'",
    )

    concluded_agreement(room, "przedwstepna")
    room.run(
        "rozwiaz_umowe_transakcji",
        id="przedwstepna",
        koniec_powod="odstąpienie z zadatkiem (art. 394 KC)",
    )
    room.refused(
        "treść umowy transakcji zmienia się tylko w projekcie",
        "rozwiaz_umowe_transakcji",
        id="przedwstepna",
        koniec_powod="inny powód",
    )


@scenario("Umowa: zawarta ma podpisany dokument, a akt także kancelarię i numer repertorium")
def agreement_concluded_with_document():
    room = new_room()
    open_sale(room)
    add_notary_office(room)

    prepare_agreement(room, "przedwstepna")
    analyse_draft(room, "przedwstepna")
    add_received_document(room, "zly-dokument", "pismo")
    room.refused(
        "zawarta umowa wskazuje otrzymany dokument",
        "odnotuj_zawarcie_umowy_transakcji",
        id="przedwstepna",
        dokument_id="zly-dokument",
        zawarto="2026-11-10T12:00:00Z",
    )
    conclude_agreement(room, "przedwstepna")

    prepare_agreement(room, "akt", "przenoszaca", "akt_notarialny", kancelaria_id="kancelaria")
    analyse_draft(room, "akt")
    add_received_document(room, "wypis-aktu", "akt_notarialny")
    room.refused(
        "akt_ma_kancelarie_i_repertorium",
        "odnotuj_zawarcie_umowy_transakcji",
        id="akt",
        dokument_id="wypis-aktu",
        zawarto="2026-12-10T11:00:00Z",
    )
    room.run(
        "odnotuj_zawarcie_umowy_transakcji",
        id="akt",
        dokument_id="wypis-aktu",
        zawarto="2026-12-10T11:00:00Z",
        repertorium="Rep. A nr 77/2026",
    )


@scenario("Umowa: zawiera się ją tylko z aktualną analizą jej bieżącego projektu")
def agreement_needs_current_analysis():
    room = new_room()
    open_sale(room)
    prepare_agreement(room, "przedwstepna")
    add_received_document(room, "podpisana", "umowa")

    room.refused(
        "umowa zostaje zawarta tylko z aktualną analizą swojego projektu",
        "odnotuj_zawarcie_umowy_transakcji",
        id="przedwstepna",
        dokument_id="podpisana",
        zawarto="2026-11-10T12:00:00Z",
    )

    # The analysed draft is replaced by a new one, so the analysis no longer covers it.
    analyse_draft(room, "przedwstepna")
    add_received_document(room, "nowy-projekt", "umowa")
    room.run(
        "zmien_projekt_umowy_transakcji",
        id="przedwstepna",
        projekt_dokument_id="nowy-projekt",
        cena=61_000_000,
    )
    room.refused(
        "umowa zostaje zawarta tylko z aktualną analizą swojego projektu",
        "odnotuj_zawarcie_umowy_transakcji",
        id="przedwstepna",
        dokument_id="podpisana",
        zawarto="2026-11-10T12:00:00Z",
    )

    analyse_draft(room, "przedwstepna", "analiza-nowego", "nowy-projekt")
    room.run(
        "odnotuj_zawarcie_umowy_transakcji",
        id="przedwstepna",
        dokument_id="podpisana",
        zawarto="2026-11-10T12:00:00Z",
    )


@scenario("Umowa: po wyjściu z projektu treść stoi, a wniosek i wpis do księgi wpisuje się raz")
def agreement_frozen_after_draft():
    room = new_room()
    open_sale(room)
    add_notary_office(room)
    concluded_deed(room, "akt")

    room.refused(
        "treść umowy transakcji zmienia się tylko w projekcie",
        "zmien_projekt_umowy_transakcji",
        id="akt",
        kancelaria_id="kancelaria",
        projekt_dokument_id="projekt-akt",
        cena=1,
    )
    room.refused(
        "wpis_po_wniosku",
        "odnotuj_wpis_do_ksiegi",
        id="akt",
        wpis_kw="2027-01-20",
    )
    room.run("odnotuj_wniosek_do_ksiegi", id="akt", wniosek_kw_zlozono="2026-12-10")
    room.refused(
        "treść umowy transakcji zmienia się tylko w projekcie",
        "odnotuj_wniosek_do_ksiegi",
        id="akt",
        wniosek_kw_zlozono="2026-12-11",
    )
    room.run("odnotuj_wpis_do_ksiegi", id="akt", wpis_kw="2027-01-20")

    # Only a concluded transfer deed has a land register entry.
    concluded_agreement(room, "przedwstepna")
    room.refused(
        "ksiega_po_zawartej_umowie_przenoszacej",
        "odnotuj_wniosek_do_ksiegi",
        id="przedwstepna",
        wniosek_kw_zlozono="2026-12-10",
    )


# Right of pre-emption ------------------------------------------------------------------------------


@scenario("Pierwokup: umowa przenosząca dopiero po umowie warunkowej i wyniku każdego pierwokupu")
def preemption_needs_conditional_deed():
    room = new_room()
    open_sale(room)
    add_notary_office(room)
    add_right_of_preemption(room)

    # A transfer deed without the conditional one is void (art. 599 § 2 KC).
    prepare_agreement(
        room, "bezwarunkowa", "przenoszaca", "akt_notarialny", kancelaria_id="kancelaria"
    )
    analyse_draft(room, "bezwarunkowa")
    add_received_document(room, "wypis", "akt_notarialny")
    refusal = "przy pierwokupie umowa przenosząca wymaga zawartej umowy warunkowej"
    room.refused(
        refusal,
        "odnotuj_zawarcie_umowy_transakcji",
        id="bezwarunkowa",
        dokument_id="wypis",
        zawarto="2026-12-10T11:00:00Z",
        repertorium="Rep. A nr 1/2026",
    )

    # With the conditional deed, but while the municipality has not answered yet.
    concluded_deed(room, "warunkowa", "warunkowa")
    prepare_agreement(
        room,
        "przenoszaca",
        "przenoszaca",
        "akt_notarialny",
        kancelaria_id="kancelaria",
        oparta_na_id="warunkowa",
    )
    analyse_draft(room, "przenoszaca")
    add_received_document(room, "wypis-przenoszacej", "akt_notarialny")
    deed = {
        "id": "przenoszaca",
        "dokument_id": "wypis-przenoszacej",
        "zawarto": "2027-01-15T11:00:00Z",
        "repertorium": "Rep. A nr 2/2027",
    }
    room.refused(refusal, "odnotuj_zawarcie_umowy_transakcji", **deed)

    record_preemption_result(room, wynik="uplynal")
    room.run("odnotuj_zawarcie_umowy_transakcji", **deed)

    # A right of pre-emption entered after the transfer deed would go round its rule.
    room.refused(
        "pierwokup wpisuje się przed zawarciem umowy przenoszącej",
        "wpisz_pierwokup",
        id="pozniejszy",
        uprawniony="Krajowy Ośrodek Wsparcia Rolnictwa",
        podstawa="art. 3 ust. 4 ukur",
    )


@scenario("Pierwokup: wykonany wyklucza umowę przenoszącą; wynik jest ostateczny i ma źródło")
def preemption_exercised():
    room = new_room()
    open_sale(room)
    add_right_of_preemption(room)

    room.refused_sql(
        "pierwokup powstaje jako czekający na wynik",
        """INSERT INTO transakcja_umowy_pierwokupy (id, uprawniony, podstawa, wynik)
        VALUES ('od-razu', 'Gmina', 'art. 109 ugn', 'uplynal')""",
    )
    room.refused(
        "wynik_po_zawiadomieniu_z_wiadomoscia",
        "odnotuj_wynik_pierwokupu",
        id="pierwokup-gminy",
        wynik="wykonano",
        zrodlo_wplyw_id=None,
    )

    record_preemption_result(room, wynik="wykonano")
    room.refused(
        "wyniku pierwokupu ani raz wpisanego zawiadomienia się nie zmienia",
        "odnotuj_wynik_pierwokupu",
        id="pierwokup-gminy",
        wynik="nie_wykonano",
        zrodlo_wplyw_id="wiadomosc-pierwokup-gminy",
    )
    room.refused(
        "uprawniony wykonał pierwokup, więc umowy przenoszącej nie ma",
        "przygotuj_umowe_transakcji",
        id="przenoszaca",
        rodzaj="przenoszaca",
        forma="akt_notarialny",
    )
    room.run(
        "przygotuj_umowe_transakcji", id="warunkowa", rodzaj="warunkowa", forma="akt_notarialny"
    )


# Parties -------------------------------------------------------------------------------------------


@scenario(
    "Strona: pełnomocnik do aktu ma pełnomocnictwo w formie aktu notarialnego (art. 99 § 1 KC)"
)
def deed_power_in_notarial_form():
    room = new_room()
    open_sale(room)
    add_parties(room)
    add_notary_office(room)
    add_deed_power(room, "pisemne", forma="pisemna")
    prepare_agreement(room, "akt", "przenoszaca", "akt_notarialny", kancelaria_id="kancelaria")
    prepare_agreement(room, "przedwstepna")

    refusal = "pełnomocnik strony działa z jej umocowania"
    room.refused(
        refusal,
        "dopisz_strone_umowy_transakcji",
        umowa_id="akt",
        uczestnik_id="kupujacy",
        strona="kupujacy",
        umocowanie_id="pisemne",
    )
    room.refused(
        refusal,
        "dopisz_strone_umowy_transakcji",
        umowa_id="przedwstepna",
        uczestnik_id="sprzedajaca",
        strona="sprzedajacy",
        umocowanie_id="pisemne",
    )

    # A written power is enough for a preliminary agreement in writing.
    room.run(
        "dopisz_strone_umowy_transakcji",
        umowa_id="przedwstepna",
        uczestnik_id="kupujacy",
        strona="kupujacy",
        umocowanie_id="pisemne",
    )
    room.run(
        "wpisz_umocowanie",
        id="notarialne",
        rodzaj="pelnomocnictwo",
        mocodawca_uczestnik_id="kupujacy",
        umocowany_uczestnik_id="brat-kupujacego",
        umocowane_biuro=0,
        zakres="do_czynnosci",
        forma="akt_notarialny",
        udzielono="2026-11-03",
    )
    room.run(
        "dopisz_strone_umowy_transakcji",
        umowa_id="akt",
        uczestnik_id="kupujacy",
        strona="kupujacy",
        umocowanie_id="notarialne",
    )

    room.refused(
        refusal,
        "zmien_dane_umocowania",
        id="notarialne",
        rodzaj="pelnomocnictwo",
        mocodawca_uczestnik_id="kupujacy",
        umocowany_uczestnik_id="brat-kupujacego",
        umocowane_biuro=0,
        zakres="do_czynnosci",
        forma="pisemna",
    )


@scenario("Strona: dopisuje się ją tylko do projektu i się jej nie zmienia")
def parties_only_in_draft():
    room = new_room()
    open_sale(room)
    add_parties(room)
    prepare_agreement(room, "przedwstepna")
    room.run(
        "dopisz_strone_umowy_transakcji",
        umowa_id="przedwstepna",
        uczestnik_id="sprzedajaca",
        strona="sprzedajacy",
    )

    room.refused_sql(
        "strony umowy transakcji się nie zmienia",
        "UPDATE transakcja_umowy_strony SET strona = 'kupujacy'",
    )
    analyse_draft(room, "przedwstepna")
    conclude_agreement(room, "przedwstepna")
    room.refused(
        "strony dopisuje się tylko do projektu umowy transakcji",
        "dopisz_strone_umowy_transakcji",
        umowa_id="przedwstepna",
        uczestnik_id="kupujacy",
        strona="kupujacy",
    )


# Payments ------------------------------------------------------------------------------------------


@scenario("Płatność: idzie na rachunek dopuszczony decyzją i niezastąpiony (R4)")
def payment_to_admitted_account():
    room = new_room()
    open_sale(room)
    add_parties(room)
    prepare_agreement(room, "przedwstepna")
    add_account(room, "niedopuszczony", "sprzedajaca", cel="zadatek")

    refusal = "płatność wskazuje rachunek dopuszczony decyzją i niezastąpiony (R4)"
    room.refused(
        refusal, "wpisz_platnosc_transakcji", **payment_fields(rachunek_id="niedopuszczony")
    )
    add_payment(room, "zadatek", "przedwstepna")
    room.refused(refusal, "wskaz_rachunek_platnosci", id="zadatek", rachunek_id="niedopuszczony")

    admitted_account_for(room, "dopuszczony", "sprzedajaca", "zadatek")
    room.run("wskaz_rachunek_platnosci", id="zadatek", rachunek_id="dopuszczony")

    # The account is replaced before the money comes: the payment is checked again.
    add_account(room, "nowy", "sprzedajaca", cel="zadatek", numer="PL27114020040000300201355387")
    room.run(
        "zastap_rachunek",
        id="dopuszczony",
        zastapiony_przez_id="nowy",
        zastapil="tozsamosc-agenta",
        zastapiono="2026-11-11T09:00:00Z",
    )
    add_received_document(room, "potwierdzenie", "potwierdzenie_bankowe")
    room.refused(
        refusal,
        "odnotuj_wplate_platnosci",
        id="zadatek",
        wplacono="2026-11-12T10:00:00Z",
        potwierdzenie_dokument_id="potwierdzenie",
    )


def payment_fields(**fields):
    return {
        "id": "platnosc",
        "umowa_id": "przedwstepna",
        "rodzaj": "zadatek",
        "kwota": 6_000_000,
        "zrodlo": "wlasne",
        **fields,
    }


@scenario("Płatność: kredyt ma bank i warunki wypłaty; wpłata ma rachunek i potwierdzenie z banku")
def payment_loan_and_proof():
    room = new_room()
    open_sale(room)
    add_parties(room)
    prepare_agreement(room, "przedwstepna")

    room.refused(
        "kredyt_z_bankiem_i_warunkami",
        "wpisz_platnosc_transakcji",
        **payment_fields(rodzaj="cena", zrodlo="kredyt", bank="Bank Przykładowy S.A."),
    )
    room.refused(
        "kredyt_z_bankiem_i_warunkami",
        "wpisz_platnosc_transakcji",
        **payment_fields(bank="Bank Przykładowy S.A."),
    )
    add_payment(room, "zadatek", "przedwstepna")
    add_received_document(room, "potwierdzenie-bez-rachunku", "potwierdzenie_bankowe")
    room.refused(
        "wplata_z_rachunkiem_i_potwierdzeniem",
        "odnotuj_wplate_platnosci",
        id="zadatek",
        wplacono="2026-11-12T10:00:00Z",
        potwierdzenie_dokument_id="potwierdzenie-bez-rachunku",
    )

    admitted_account_for(room, "rachunek", "sprzedajaca", "zadatek")
    room.run("wskaz_rachunek_platnosci", id="zadatek", rachunek_id="rachunek")
    add_received_document(room, "zrzut-ekranu", "pismo")
    room.refused(
        "wpłata wskazuje otrzymane potwierdzenie bankowe",
        "odnotuj_wplate_platnosci",
        id="zadatek",
        wplacono="2026-11-12T10:00:00Z",
        potwierdzenie_dokument_id="zrzut-ekranu",
    )
    record_payment(room, "zadatek")


@scenario("Płatność: dopisuje się ją do projektu; po zawarciu warunki stoją, a zapłaconej nic")
def payment_terms_fixed():
    room = new_room()
    open_sale(room)
    add_parties(room)
    prepare_agreement(room, "przedwstepna")
    add_payment(room, "zadatek", "przedwstepna")
    room.run("zmien_platnosc_transakcji", id="zadatek", kwota=5_000_000, zrodlo="wlasne")

    room.refused_sql(
        "płatność dopisuje się do projektu umowy transakcji, jeszcze niezapłaconą",
        """INSERT INTO transakcja_umowy_platnosci (id, umowa_id, rodzaj, kwota, zrodlo, rachunek_id,
          wplacono, potwierdzenie_dokument_id)
        VALUES ('od-razu', 'przedwstepna', 'zaliczka', 100, 'wlasne', NULL, '2026-11-12', NULL)""",
    )

    analyse_draft(room, "przedwstepna")
    conclude_agreement(room, "przedwstepna")
    room.refused(
        "warunki płatności zmieniają się tylko w projekcie umowy",
        "zmien_platnosc_transakcji",
        id="zadatek",
        kwota=4_000_000,
        zrodlo="wlasne",
    )
    room.refused(
        "płatność dopisuje się do projektu umowy transakcji",
        "wpisz_platnosc_transakcji",
        **payment_fields(id="druga"),
    )

    # The account may still be named after signing, and then the money comes.
    admitted_account_for(room, "rachunek", "sprzedajaca", "zadatek")
    room.run("wskaz_rachunek_platnosci", id="zadatek", rachunek_id="rachunek")
    record_payment(room, "zadatek")
    admitted_account_for(room, "drugi", "sprzedajaca", "zadatek")
    room.refused(
        "zapłaconej płatności nie zmienia się wcale",
        "wskaz_rachunek_platnosci",
        id="zadatek",
        rachunek_id="drugi",
    )


# Analyses ------------------------------------------------------------------------------------------


@scenario("Analiza: nowa odbiera aktualność poprzedniej tego samego przedmiotu i umowy")
def analysis_supersedes_previous():
    room = new_room()
    open_sale(room)
    prepare_agreement(room, "przedwstepna")
    prepare_agreement(room, "rezerwacyjna", "rezerwacyjna")

    analyse_draft(room, "przedwstepna", "pierwsza")
    analyse_draft(room, "rezerwacyjna", "innej-umowy")
    analyse_draft(room, "przedwstepna", "druga")

    current = sorted(row["id"] for row in room.run("analizy_aktualne_transakcji"))
    assert current == ["druga", "innej-umowy"], current

    room.refused_sql(
        "analiza powstaje jako aktualna",
        """INSERT INTO transakcja_analizy_wpisy (id, przedmiot, sporzadzono, wnioski, sciezka, skrot,
          aktualna)
        VALUES ('stara', 'ksiega', '2026-11-01', 'wnioski', 'a.typ', 'sha', 0)""",
    )
    room.refused(
        "umowa_przy_analizie_umowy",
        "wpisz_analize_transakcji",
        id="bez-umowy",
        przedmiot="projekt_umowy",
        dokument_id="projekt-przedwstepna",
        sporzadzono="2026-11-06",
        wnioski="wnioski",
        sciezka="a.typ",
        skrot="sha",
    )


@scenario("Analiza: wpisu się nie zmienia, a ryzyko dopisuje się do aktualnej")
def analysis_immutable_and_risks():
    room = new_room()
    open_sale(room)
    prepare_agreement(room, "przedwstepna")
    analyse_draft(room, "przedwstepna", "pierwsza")

    room.refused_sql(
        "analizy się nie zmienia",
        "UPDATE transakcja_analizy_wpisy SET wnioski = 'inne' WHERE id = 'pierwsza'",
    )

    room.run("dopisz_ryzyko_transakcji", analiza_id="pierwsza", opis="Zadatek bez terminu")
    analyse_draft(room, "przedwstepna", "druga")
    room.refused(
        "ryzyko dopisuje się do aktualnej analizy",
        "dopisz_ryzyko_transakcji",
        analiza_id="pierwsza",
        opis="Spóźnione ryzyko",
    )
    room.refused_sql(
        "analizy się nie zmienia",
        "UPDATE transakcja_analizy_wpisy SET aktualna = 1 WHERE id = 'pierwsza'",
    )
    room.refused_sql(
        "ryzyka się nie zmienia",
        "UPDATE transakcja_analizy_ryzyka SET skutek = 'inny'",
    )


# R9: no deletion, no replacement -----------------------------------------------------------------


def transakcja_tables(room):
    return [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'transakcja\\_%' ESCAPE '\\'"
        )
    ]


@scenario("R9: wierszy typu transakcji się nie usuwa ani nie zastępuje przez REPLACE i upsert")
def no_deletion_or_replacement():
    room = new_room()
    full_sale_with_preemption(room)

    tables = transakcja_tables(room)
    assert len(tables) == 7, tables
    refusal = "wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE"

    for table in tables:
        assert room.value(f"SELECT count(*) FROM {table}") > 0, f"{table} has no row"

        room.refused_sql(
            "wierszy teczki się nie usuwa (R9)",
            f"DELETE FROM {table} WHERE rowid = (SELECT min(rowid) FROM {table})",
        )
        room.refused_sql(refusal, f"INSERT OR REPLACE {first_row_copy(room, table)}")
        room.refused_sql(
            refusal,
            f"INSERT {first_row_copy(room, table)} ON CONFLICT (id) DO UPDATE SET id = excluded.id",
        )
        if unique_keys_of(room, table):
            room.refused_sql(
                refusal,
                f"INSERT OR REPLACE {first_row_copy(room, table, replace_unique_key=True)}",
            )
