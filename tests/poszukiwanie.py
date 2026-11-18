"""Scenarios of the search case catalog (spec/cases/poszukiwanie) on top of the core.

A search room is the core of every case plus the kinds of the search type. Each scenario starts
from an empty room and checks one rule the database keeps: a write the rule allows and a write it
refuses. The first scenario follows a whole search case as the profile describes it, from intake
through the brokerage contract, the search profile, the credit analysis, proposals and viewings
to the client's accepted offer.
"""

from pathlib import Path

from core import (
    WITHDRAWAL_ENDS,
    add_consent,
    add_inflow,
    add_person,
    add_received_document,
    approve,
    approved_decision,
    ask_for_decision,
    check,
    conclude_contract,
    open_case,
    prepare_contract,
    prepare_letter,
    send,
)
from core_porzadek import first_row_copy, unique_keys_of
from room import Room

POSZUKIWANIE = Path(__file__).resolve().parents[1] / "spec" / "cases" / "poszukiwanie"

SCENARIOS = []

REPLACEMENT = "wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE"
DELETION = "wierszy teczki się nie usuwa (R9)"
STANDARD_DISCLAIMER = "Analiza jest orientacyjna; nie jest ofertą ani rekomendacją kredytu."


def scenario(title):
    def register(function):
        SCENARIOS.append((title, function))

        return function

    return register


def new_room():
    return Room(POSZUKIWANIE)


# Building blocks of a search case ----------------------------------------------------------------


def approved_other(room, decision_id, dotyczy_tabela, dotyczy_id, rodzaj="inna"):
    """A decision of the broker on a row of the search type, asked by the searcher's keeper."""
    ask_for_decision(
        room,
        decision_id,
        rodzaj,
        dotyczy_tabela,
        dotyczy_id,
        prosi_stanowisko="opiekun_poszukujacego",
    )
    approve(room, decision_id)


def client(room, person_id="klient", **fields):
    add_person(room, person_id, imiona="Marek", nazwisko="Wiśniewski", konsument=1, **fields)
    room.run("nadaj_role_uczestnikowi", uczestnik_id=person_id, rola="klient")


def brokerage_contract(room, sposob_zawarcia="w_lokalu_biura", odstapienie_do=None):
    """A brokerage contract of the buyer, checked, signed and concluded on 2026-11-04."""
    check(room, "tozsamosc-klienta", "tozsamosc", uczestnik_id="klient")
    prepare_contract(
        room,
        "umowa",
        ["klient"],
        rodzaj="posrednictwo",
        wylacznosc=0,
        wynagrodzenie_opis="2% ceny",
        wynagrodzenie_zdarzenie="zawarcie umowy przyrzeczonej",
    )
    for czynnosc in ("wyszukiwanie_ofert", "prezentacje", "wysylanie_ofert"):
        room.run("dopisz_czynnosc_umowy", umowa_id="umowa", czynnosc=czynnosc)
    room.run("przekaz_umowe_do_podpisu", id="umowa")
    conclude_contract(room, "umowa", sposob_zawarcia, odstapienie_do)


def search_case(room, **contract_fields):
    """A search case with the client and a concluded brokerage contract."""
    open_case(room, typ_pokoju="poszukiwanie")
    client(room)
    brokerage_contract(room, **contract_fields)


def client_consent(room, consent_id, rodzaj, dotyczy_tabela, dotyczy_id, person_id="klient"):
    add_consent(
        room,
        consent_id,
        person_id,
        rodzaj,
        tresc=f"Zgoda {rodzaj}.",
        dotyczy_tabela=dotyczy_tabela,
        dotyczy_id=dotyczy_id,
    )


def profile_version(room, version_id="wersja-1", od="2026-11-05T09:00:00Z", **fields):
    room.run(
        "wpisz_wersje_profilu",
        **{
            "id": version_id,
            "profil_id": "profil",
            "cel": "kupno",
            "na_wynajem": 0,
            "rodzaj_nieruchomosci": "mieszkanie",
            "polozenie": "Wrocław, Krzyki albo Fabryczna",
            "pokoje_min": 3,
            "budzet": 75_000_000,
            "budzet_z_kosztami": 0,
            "od": od,
            **fields,
        },
    )


def confirm_profile(room, version_id):
    client_consent(
        room,
        f"zgoda-{version_id}",
        "potwierdzenie_profilu",
        "poszukiwanie_profile_wersje",
        version_id,
    )
    room.run("potwierdz_wersje_profilu", id=version_id, zgoda_id=f"zgoda-{version_id}")


def confirmed_profile(room, version_id="wersja-1", od="2026-11-05T09:00:00Z", **fields):
    if not room.run("profile_poszukiwania"):
        room.run("zaloz_profil_poszukiwania", id="profil", nazwa="Mieszkanie dla rodziny")
    profile_version(room, version_id, od, **fields)
    confirm_profile(room, version_id)


def proposal(room, proposal_id="propozycja", profil_wersja_id="wersja-1", **fields):
    room.run(
        "wpisz_propozycje",
        **{
            "id": proposal_id,
            "profil_wersja_id": profil_wersja_id,
            "zrodlo": "portal",
            "ogloszenie_sciezka": f"propozycje/{proposal_id}.html",
            "ogloszenie_skrot": f"sha256-{proposal_id}",
            "portal": "otodom.pl",
            "numer": "OT-445566",
            "prowadzi": "wlasciciel",
            "niepubliczna": 0,
            "zgodnosc": "3 pokoje, Krzyki, balkon",
            "cena_ogloszenia": 66_000_000,
            "utworzono": "2026-11-06T09:00:00Z",
            **fields,
        },
    )


def presented_proposal(room, proposal_id="propozycja", **fields):
    proposal(room, proposal_id, **fields)
    room.run(
        "odnotuj_przedstawienie_propozycji", id=proposal_id, przedstawiono="2026-11-06T10:00:00Z"
    )


def offer_ready(room, **contract_fields):
    """A search case with a confirmed profile and a proposal the client got."""
    search_case(room, **contract_fields)
    confirmed_profile(room)
    presented_proposal(room)


def client_offer(room, offer_id="oferta", kwota=61_000_000, **fields):
    room.run(
        "przygotuj_oferte_klienta",
        **{
            "id": offer_id,
            "dotyczy": "kupno",
            "propozycja_id": "propozycja",
            "adresat": "Jan Nowak, właściciel, bez biura",
            "wpisano": "2026-11-08T09:00:00Z",
            "kwota": kwota,
            "finansowanie_w_ofercie": "część ceny z kredytu hipotecznego",
            "wiaze_do": "2026-11-12",
            **fields,
        },
    )


def confirm_offer(room, offer_id):
    client_consent(
        room, f"zgoda-{offer_id}", "potwierdzenie_oferty", "poszukiwanie_oferty_wpisy", offer_id
    )
    room.run("potwierdz_oferte_klienta", id=offer_id, zgoda_id=f"zgoda-{offer_id}")


def letter_awaiting_consent(room, letter_id):
    """An offer letter to the other side, waiting for the human's consent to send it."""
    room.run("przekaz_wysylke_do_zgody", id=letter_id)
    approved_other(
        room, f"wyslanie-{letter_id}", "sprawa_korespondencja_wysylki", letter_id, "wyslanie"
    )


def prepare_offer_letter(room, letter_id):
    prepare_letter(
        room,
        letter_id,
        adresat_rodzaj="inny",
        adresat_nazwa="Jan Nowak",
        wzor="oferta-zakupu",
    )


def sent_client_offer(room, offer_id="oferta", wyslano="2026-11-08T12:00:00Z", **fields):
    """A client offer the client confirmed and a human allowed to send."""
    client_offer(room, offer_id, **fields)
    confirm_offer(room, offer_id)
    letter_id = f"wysylka-{offer_id}"
    prepare_offer_letter(room, letter_id)
    room.run("wskaz_wysylke_oferty_klienta", id=offer_id, wysylka_id=letter_id)
    letter_awaiting_consent(room, letter_id)
    send(room, letter_id, decyzja_id=f"wyslanie-{letter_id}", wyslano=wyslano)


def counteroffer_of_other_side(room, counter_id, answers_id, kwota=66_000_000):
    add_inflow(room, f"wplyw-{counter_id}", nadawca="Jan Nowak <jan@example.pl>")
    room.run(
        "odnotuj_odpowiedz_drugiej_strony",
        id=answers_id,
        odpowiedz_drugiej_strony="kontroferta",
        odpowiedz_wplyw_id=f"wplyw-{counter_id}",
        odpowiedz_przedstawiono="2026-11-09T10:00:00Z",
    )
    room.run(
        "wpisz_kontroferte_drugiej_strony",
        id=counter_id,
        dotyczy="kupno",
        propozycja_id="propozycja",
        adresat="Jan Nowak, właściciel, bez biura",
        odpowiada_na_id=answers_id,
        wplyw_id=f"wplyw-{counter_id}",
        wpisano="2026-11-09T09:00:00Z",
        kwota=kwota,
        sposob_zlozenia="na_pismie",
    )


def stage(room, etap, od="2026-11-05T10:00:00Z", **fields):
    room.run("zmien_etap_sprawy", etap=etap, od=od, zmienil="agent-poszukiwania", **fields)


def credit_analysis(room, analysis_id, utworzono="2026-11-07T10:00:00Z", **fields):
    room.run(
        "wpisz_analize_kredytowa",
        **{
            "id": analysis_id,
            "wersja_parametrow": "standard-2026-10",
            "okres_lat": 25,
            "poza_standardem": 0,
            "kwota_od": 52_000_000,
            "kwota_do": 58_000_000,
            "rata_od": 340_000,
            "rata_do": 380_000,
            "udzial_wkladu": 1800,
            "wynik_sciezka": f"analizy/{analysis_id}.json",
            "wynik_skrot": f"sha256-{analysis_id}",
            "wyjasnienie": "Przy cenie 650 000 zł wkład wynosi około 18% ceny.",
            "zastrzezenie": STANDARD_DISCLAIMER,
            "utworzono": utworzono,
            **fields,
        },
    )


def income_from_document(room, data_id="dochod", wpisano="2026-11-07T09:00:00Z"):
    add_received_document(room, f"dokument-{data_id}", "zaswiadczenie", wymaga="bank")
    room.run(
        "wpisz_dana_kredytowa",
        id=data_id,
        rodzaj="dochod",
        opis="umowa o pracę, netto",
        wartosc=900_000,
        okres_zrodla="średnia z 6 miesięcy",
        dokument_id=f"dokument-{data_id}",
        deklaracja=0,
        wpisano=wpisano,
    )


def full_case(room):
    """A search case with a row in every table of the search type, as the profile goes."""
    open_case(room, typ_pokoju="poszukiwanie")
    stage(room, "weryfikacja_klienta", od="2026-11-02T10:00:00Z")

    # The client and his partner, who buys with him but is not a party to the contract.
    client(room)
    add_person(room, "partnerka", imiona="Ewa", nazwisko="Lis")
    room.run("nadaj_role_uczestnikowi", uczestnik_id="partnerka", rola="wspolnabywca")

    # The brokerage contract signed at a distance by a consumer: the office starts work before
    # the end of the withdrawal period only on his demand.
    stage(room, "umowa_posrednictwa", od="2026-11-03T12:00:00Z")
    brokerage_contract(room, sposob_zawarcia="na_odleglosc", odstapienie_do=WITHDRAWAL_ENDS)
    add_consent(room, "zadanie", "klient", "zadanie_rozpoczecia_pracy")
    room.run(
        "wpisz_zdarzenie_umowy",
        id="zdarzenie-zadanie",
        umowa_id="umowa",
        rodzaj="zadanie_rozpoczecia_pracy",
        chwila="2026-11-04T13:00:00Z",
        zgoda_id="zadanie",
    )

    # The search profile with its features and financing, confirmed by the client.
    stage(room, "profil_poszukiwania", od="2026-11-04T14:00:00Z")
    room.run("zaloz_profil_poszukiwania", id="profil", nazwa="Mieszkanie dla rodziny")
    profile_version(room, termin="2027-02-28", termin_powod="koniec obecnego najmu")
    room.run("dopisz_ceche_profilu", wersja_id="wersja-1", cecha="balkon", waga="warunek")
    room.run("dopisz_ceche_profilu", wersja_id="wersja-1", cecha="garaż", waga="zyczenie")
    room.run("dopisz_finansowanie_profilu", wersja_id="wersja-1", zrodlo="kredyt", kwota=60_000_000)
    room.run(
        "dopisz_finansowanie_profilu", wersja_id="wersja-1", zrodlo="oszczednosci", kwota=15_000_000
    )
    confirm_profile(room, "wersja-1")
    stage(room, "propozycje", od="2026-11-05T10:00:00Z")

    # The credit analysis from documents, with the bank the client chose and the schedule.
    income_from_document(room)
    room.run(
        "wpisz_dana_kredytowa",
        id="rata-samochodu",
        rodzaj="zobowiazanie",
        opis="rata za samochód według klienta",
        wartosc=80_000,
        deklaracja=1,
        wpisano="2026-11-07T09:10:00Z",
    )
    credit_analysis(room, "analiza-1")
    room.run("dopisz_dana_do_analizy", analiza_id="analiza-1", dana_id="dochod")
    add_inflow(room, "wybor-banku", "klient", nadawca="Marek Wiśniewski")
    room.run(
        "wpisz_bank_klienta",
        bank="Bank Przykładowy S.A.",
        wybrano="2026-11-07",
        zrodlo_wplyw_id="wybor-banku",
    )
    room.run(
        "odnotuj_wymagania_banku",
        id=1,
        wymagania_zrodlo="pismo banku do klienta",
        wymagania_data="2026-11-07",
    )
    room.run("wpisz_krok_harmonogramu", krok="wniosek", przewidywana="2026-11-15")
    room.run("odnotuj_krok_harmonogramu", id=1, rzeczywista="2026-11-14")

    # After three viewings the client allows another district: a new confirmed version.
    profile_version(
        room,
        "wersja-2",
        od="2026-11-06T08:00:00Z",
        polozenie="Wrocław, Krzyki albo Fabryczna albo Psie Pole",
    )
    room.run("dopisz_ceche_profilu", wersja_id="wersja-2", cecha="balkon", waga="zyczenie")
    confirm_profile(room, "wersja-2")

    # A proposal from a portal with the land register lookup and comparable sales, a viewing
    # with the client's opinion and an open question.
    presented_proposal(room, profil_wersja_id="wersja-2", numer_kw="WR1K/00098765/4")
    room.run(
        "odnotuj_przegladanie_ksiegi_propozycji",
        id="propozycja",
        numer_kw="WR1K/00098765/4",
        kw_przegladano="2026-11-06T11:00:00Z",
        kw_ogloszajacy_wpisany="tak",
        kw_wpisy_iii_iv="hipoteka umowna",
    )
    add_received_document(room, "ceny-rcn", "inny", pochodzenie="rejestr cen nieruchomości")
    room.run("odnotuj_ceny_transakcji_propozycji", id="propozycja", ceny_rcn_dokument_id="ceny-rcn")
    room.run(
        "odnotuj_reakcje_na_propozycje",
        id="propozycja",
        reakcja="chce_obejrzec",
        reakcja_powod="blisko szkoły",
    )
    room.run(
        "umow_prezentacje_propozycji",
        id="prezentacja",
        propozycja_id="propozycja",
        termin="2026-11-07T17:00:00Z",
        kto_pokazuje="wlasciciel",
        dane_klienta_podane="imię, nazwisko, telefon",
        szerszy_zakres=0,
        uczestnicy="klient i partnerka, pośrednik biura",
    )
    room.run(
        "odnotuj_opinie_klienta",
        id="prezentacja",
        opinia_klienta="Podoba się kuchnia, chce złożyć ofertę.",
    )
    room.run(
        "odnotuj_notatke_posrednika",
        id="prezentacja",
        notatka_posrednika="Zawilgocenie w łazience przy oknie.",
    )
    room.run(
        "dopisz_pytanie_do_prezentacji",
        prezentacja_id="prezentacja",
        pytanie="Ile wynosi zaliczka do wspólnoty?",
    )
    add_inflow(room, "odpowiedz-wlasciciela", nadawca="Jan Nowak <jan@example.pl>")
    room.run(
        "odnotuj_odpowiedz_na_pytanie",
        id=1,
        odpowiedz="420 zł miesięcznie (twierdzenie właściciela)",
        odpowiedz_wplyw_id="odpowiedz-wlasciciela",
    )

    # The chain of offers: the client offers, the owner counters, the client counters again and
    # the owner accepts; every offer of the client waits for the human's consent.
    sent_client_offer(room)
    stage(room, "negocjacje", od="2026-11-08T12:30:00Z")
    counteroffer_of_other_side(room, "kontroferta-wlasciciela", "oferta")
    room.run(
        "odnotuj_przedstawienie_kontroferty",
        id="kontroferta-wlasciciela",
        przedstawiono="2026-11-09T10:00:00Z",
    )
    room.run(
        "odnotuj_decyzje_klienta_o_kontrofercie",
        id="kontroferta-wlasciciela",
        decyzja_klienta="kontroferta",
    )
    sent_client_offer(
        room,
        "kontroferta-klienta",
        kwota=64_000_000,
        odpowiada_na_id="kontroferta-wlasciciela",
        wpisano="2026-11-09T11:00:00Z",
        wyslano="2026-11-09T12:00:00Z",
    )
    add_inflow(room, "przyjecie", nadawca="Jan Nowak <jan@example.pl>")
    room.run(
        "odnotuj_odpowiedz_drugiej_strony",
        id="kontroferta-klienta",
        odpowiedz_drugiej_strony="przyjeta",
        odpowiedz_wplyw_id="przyjecie",
        odpowiedz_przedstawiono="2026-11-10T09:00:00Z",
    )
    stage(room, "oferta_przyjeta", od="2026-11-10T10:00:00Z")

    # The accepted offer goes to a transaction case.
    room.run(
        "wpisz_powiazanie",
        rodzaj="sprawa_zrodlowa",
        typ_teczki="transakcja",
        oznaczenie="Transakcja 2026/014",
        odnosnik="pokoj-transakcji-014",
        wpisal="tozsamosc-kierownika-posrednictwa",
        wpisano="2026-11-10T11:00:00Z",
    )


# The whole case ----------------------------------------------------------------------------------


@scenario("Przebieg poszukiwania: od umowy przez profil, analizę i prezentację do przyjętej oferty")
def whole_case():
    room = new_room()

    full_case(room)
    room.run("zakoncz_umowe", id="umowa", koniec_powod="transakcja doszła do skutku")
    stage(room, "koniec_poszukiwania", od="2027-01-20T09:00:00Z")
    stage(room, "zamknieta", od="2027-01-21T09:00:00Z")

    etapy = [row["etap"] for row in room.run("etapy")]
    assert etapy == [
        "przyjeta",
        "weryfikacja_klienta",
        "umowa_posrednictwa",
        "profil_poszukiwania",
        "propozycje",
        "negocjacje",
        "oferta_przyjeta",
        "koniec_poszukiwania",
        "zamknieta",
    ], etapy
    assert [row["id"] for row in room.run("profile_biezace")] == ["wersja-2"]
    assert room.run("analiza_aktualna")[0]["id"] == "analiza-1"
    assert [row["kierunek"] for row in room.run("oferty_klienta")] == [
        "od_klienta",
        "od_drugiej_strony",
        "od_klienta",
    ]
    assert room.run("oferty_klienta")[2]["odpowiedz_drugiej_strony"] == "przyjeta"
    assert room.run("pytania_otwarte") == []


# poszukiwanie-reguly -----------------------------------------------------------------------------


@scenario("Reguły: teczka poszukiwania ma typ pokoju poszukiwanie")
def room_type():
    room = new_room()
    record = {
        "oznaczenie": "Poszukiwanie 2026/017",
        "przyjal": "tozsamosc-posrednika",
        "przyjeto": "2026-11-02T09:00:00Z",
    }

    room.refused(
        "teczka poszukiwania ma typ pokoju poszukiwanie",
        "zaloz_metryke",
        typ_pokoju="oferta",
        **record,
    )
    room.run("zaloz_metryke", typ_pokoju="poszukiwanie", **record)


@scenario(
    "Reguły: etapy poszukiwania idą po kolei; z negocjacji i przyjętej oferty wraca się do propozycji"
)
def stage_transitions():
    room = new_room()
    search_case(room)
    confirmed_profile(room)
    rule = "etap poszukiwania przechodzi"

    room.refused(
        rule, "zmien_etap_sprawy", etap="profil_poszukiwania", od="2026-11-05", zmienil="a"
    )
    room.refused(rule, "zmien_etap_sprawy", etap="negocjacje", od="2026-11-05", zmienil="a")

    for etap in ("weryfikacja_klienta", "umowa_posrednictwa", "profil_poszukiwania", "propozycje"):
        stage(room, etap)
    room.refused(rule, "zmien_etap_sprawy", etap="oferta_przyjeta", od="2026-11-06", zmienil="a")

    for etap in ("negocjacje", "propozycje", "negocjacje", "oferta_przyjeta", "propozycje"):
        stage(room, etap)
    room.refused(rule, "zmien_etap_sprawy", etap="zamknieta", od="2026-11-07", zmienil="a")
    stage(room, "koniec_poszukiwania")
    stage(room, "zamknieta")


@scenario(
    "Reguły: przerwać wolno każdy etap przed zamknięciem; z przerwania nie ma skoku do innego etapu"
)
def stage_interrupted():
    room = new_room()
    search_case(room)
    stage(room, "weryfikacja_klienta")
    approved_other(room, "przerwanie", None, None, "przerwanie_sprawy")

    stage(room, "przerwana", decyzja_id="przerwanie")
    room.refused(
        "etap poszukiwania przechodzi",
        "zmien_etap_sprawy",
        etap="koniec_poszukiwania",
        od="2026-11-06",
        zmienil="a",
    )
    stage(room, "zamknieta")


@scenario(
    "Reguły: wznowione poszukiwanie wraca do etapu sprzed przerwania, który dalej wymaga umowy"
)
def stage_resumed():
    room = new_room()
    search_case(room)
    for etap in ("weryfikacja_klienta", "umowa_posrednictwa", "profil_poszukiwania"):
        stage(room, etap)
    approved_decision(room, "przerwanie", "przerwanie_sprawy")
    stage(room, "przerwana", decyzja_id="przerwanie")
    approved_decision(room, "wznowienie", "wznowienie_sprawy", podpisano="2026-11-05T12:00:00Z")

    room.refused(
        "etap poszukiwania przechodzi",
        "zmien_etap_sprawy",
        etap="koniec_poszukiwania",
        od="2026-11-05T13:00:00Z",
        zmienil="agent-poszukiwania",
        decyzja_id="wznowienie",
    )
    stage(room, "profil_poszukiwania", "2026-11-05T13:00:00Z", decyzja_id="wznowienie")

    approved_decision(room, "drugie-przerwanie", "przerwanie_sprawy")
    stage(room, "przerwana", "2026-11-06T09:00:00Z", decyzja_id="drugie-przerwanie")
    room.run("zakoncz_umowe", id="umowa", koniec_powod="wypowiedzenie przez klienta")
    approved_decision(
        room, "drugie-wznowienie", "wznowienie_sprawy", podpisano="2026-11-06T10:00:00Z"
    )
    room.refused(
        "etap profil_poszukiwania wymaga zawartej umowy pośrednictwa",
        "zmien_etap_sprawy",
        etap="profil_poszukiwania",
        od="2026-11-06T11:00:00Z",
        zmienil="agent-poszukiwania",
        decyzja_id="drugie-wznowienie",
    )
    stage(room, "zamknieta", "2026-11-06T12:00:00Z")


@scenario("Reguły: profil po zawarciu umowy, propozycje po potwierdzeniu profilu przez klienta")
def stages_need_contract_and_profile():
    room = new_room()
    open_case(room, typ_pokoju="poszukiwanie")
    client(room)
    for etap in ("weryfikacja_klienta", "umowa_posrednictwa"):
        stage(room, etap)

    room.refused(
        "etap profil_poszukiwania wymaga zawartej umowy pośrednictwa",
        "zmien_etap_sprawy",
        etap="profil_poszukiwania",
        od="2026-11-04",
        zmienil="a",
    )
    brokerage_contract(room)
    stage(room, "profil_poszukiwania")

    room.run("zaloz_profil_poszukiwania", id="profil", nazwa="Mieszkanie")
    profile_version(room)
    rule = "etap propozycje wymaga profilu poszukiwania potwierdzonego przez klienta"
    room.refused(rule, "zmien_etap_sprawy", etap="propozycje", od="2026-11-05", zmienil="a")
    confirm_profile(room, "wersja-1")
    stage(room, "propozycje")


@scenario("Reguły: role w teczce poszukiwania to klient, współnabywca i małżonek")
def roles():
    room = new_room()
    add_person(room, "osoba")

    room.refused(
        "rola w teczce poszukiwania",
        "nadaj_role_uczestnikowi",
        uczestnik_id="osoba",
        rola="wspolwlasciciel",
    )
    room.run("nadaj_role_uczestnikowi", uczestnik_id="osoba", rola="wspolnabywca")


@scenario(
    "Reguły: zgody w teczce poszukiwania to potwierdzenia profilu i oferty oraz zgody rdzenia"
)
def consent_kinds():
    room = new_room()
    client(room)
    add_inflow(room, "wiadomosc", "klient")

    room.refused(
        "zgoda w teczce poszukiwania",
        "wpisz_zgode",
        **consent_fields("zatwierdzenie_ceny"),
    )
    room.run("wpisz_zgode", **consent_fields("potwierdzenie_profilu"))


def consent_fields(rodzaj):
    return {
        "id": f"zgoda-{rodzaj}",
        "uczestnik_id": "klient",
        "rodzaj": rodzaj,
        "tresc": "Potwierdzam.",
        "zlozono": "2026-11-05T09:00:00Z",
        "wplyw_id": "wiadomosc",
        "trwaly_nosnik": 0,
    }


@scenario("Reguły: umowa w teczce poszukiwania to umowa pośrednictwa; rachunków i wypłat nie ma")
def contract_kind_and_no_money():
    room = new_room()
    client(room)
    add_received_document(room, "pismo", "pismo")
    rule = "umowa w teczce poszukiwania to umowa pośrednictwa"

    room.refused(
        rule, "przygotuj_umowe", id="zarzadzanie", rodzaj="zarzadzanie", obie_strony=0, wylacznosc=0
    )
    room.run("przygotuj_umowe", id="umowa", rodzaj="posrednictwo", obie_strony=0, wylacznosc=0)
    room.refused_sql(
        "rodzaj umowy się nie zmienia", "UPDATE sprawa_umowy_wpisy SET rodzaj = 'zarzadzanie'"
    )

    money = "teczka poszukiwania nie przyjmuje pieniędzy"
    room.refused(
        money,
        "wpisz_rachunek",
        id="rachunek",
        uczestnik_id="klient",
        numer="PL61109010140000071219812874",
        posiadacz="Jan Nowak",
        cel="zwrot_kaucji",
        zrodlo_dokument_id="pismo",
        wpisano="2026-11-03T08:00:00Z",
    )
    room.refused(
        money,
        "przygotuj_wyplate",
        id="wyplata",
        podstawa_tabela="poszukiwanie_oferty_wpisy",
        podstawa_id="oferta",
        kwota=100,
        wyliczenie_sciezka="w.json",
        wyliczenie_skrot="sha256-w",
        odbiorca_uczestnik_id="klient",
        rachunek_id="rachunek",
        utworzono="2026-11-03T08:00:00Z",
    )


# poszukiwanie-profile ----------------------------------------------------------------------------


@scenario("Profil: wersja powstaje niepotwierdzona i jest późniejsza niż poprzednie wersje")
def profile_version_order():
    room = new_room()
    client(room)
    room.run("zaloz_profil_poszukiwania", id="profil", nazwa="Mieszkanie")
    client_consent(room, "zgoda", "potwierdzenie_profilu", "poszukiwanie_profile_wersje", "od-razu")

    room.refused_sql(
        "wersja profilu powstaje niepotwierdzona",
        "INSERT INTO poszukiwanie_profile_wersje (id, profil_id, cel, rodzaj_nieruchomosci, "
        "polozenie, budzet, budzet_z_kosztami, zgoda_id, od) VALUES ('od-razu', 'profil', 'kupno', "
        "'mieszkanie', 'Wrocław', 1, 0, 'zgoda', '2026-11-05')",
    )
    profile_version(room)
    room.refused(
        "nowa wersja profilu jest późniejsza",
        "wpisz_wersje_profilu",
        **profile_fields("wersja-2", od="2026-11-05T09:00:00Z"),
    )
    room.run("wpisz_wersje_profilu", **profile_fields("wersja-2", od="2026-11-06T09:00:00Z"))


def profile_fields(version_id, **fields):
    return {
        "id": version_id,
        "profil_id": "profil",
        "cel": "kupno",
        "na_wynajem": 0,
        "rodzaj_nieruchomosci": "mieszkanie",
        "polozenie": "Wrocław",
        "budzet": 75_000_000,
        "budzet_z_kosztami": 0,
        "od": "2026-11-05T09:00:00Z",
        **fields,
    }


@scenario("Profil potwierdza klient zgodą potwierdzenie_profilu, która dotyczy tej wersji")
def profile_confirmed_by_client():
    room = new_room()
    client(room)
    add_person(room, "partnerka", imiona="Ewa", nazwisko="Lis")
    room.run("zaloz_profil_poszukiwania", id="profil", nazwa="Mieszkanie")
    profile_version(room)
    rule = "wersja profilu ma zgodę potwierdzenie_profilu klienta"

    client_consent(
        room,
        "zgoda-partnerki",
        "potwierdzenie_profilu",
        "poszukiwanie_profile_wersje",
        "wersja-1",
        "partnerka",
    )
    room.refused(rule, "potwierdz_wersje_profilu", id="wersja-1", zgoda_id="zgoda-partnerki")
    client_consent(
        room, "zgoda-oferty", "potwierdzenie_oferty", "poszukiwanie_profile_wersje", "wersja-1"
    )
    room.refused(rule, "potwierdz_wersje_profilu", id="wersja-1", zgoda_id="zgoda-oferty")
    client_consent(
        room, "zgoda-innej", "potwierdzenie_profilu", "poszukiwanie_profile_wersje", "inna"
    )
    room.refused(rule, "potwierdz_wersje_profilu", id="wersja-1", zgoda_id="zgoda-innej")

    confirm_profile(room, "wersja-1")
    assert [row["id"] for row in room.run("profile_biezace")] == ["wersja-1"]


@scenario("Profil: potwierdzonej wersji się nie zmienia i nie dostaje nowych cech ani finansowania")
def profile_confirmed_is_fixed():
    room = new_room()
    client(room)
    room.run("zaloz_profil_poszukiwania", id="profil", nazwa="Mieszkanie")
    profile_version(room)
    room.run("dopisz_ceche_profilu", wersja_id="wersja-1", cecha="balkon", waga="warunek")
    room.refused(
        REPLACEMENT, "dopisz_ceche_profilu", wersja_id="wersja-1", cecha="balkon", waga="zyczenie"
    )
    confirm_profile(room, "wersja-1")
    rule = "cechy i finansowanie dopisuje się tylko do niepotwierdzonej wersji profilu"

    room.refused(rule, "dopisz_ceche_profilu", wersja_id="wersja-1", cecha="garaż", waga="zyczenie")
    room.refused(
        rule, "dopisz_finansowanie_profilu", wersja_id="wersja-1", zrodlo="kredyt", kwota=1
    )
    room.refused_sql(
        "wersji profilu się nie zmienia",
        "UPDATE poszukiwanie_profile_wersje SET budzet = 80000000",
    )
    room.refused_sql(
        "cechy profilu się nie zmienia", "UPDATE poszukiwanie_profile_cechy SET waga = 'zyczenie'"
    )


@scenario("Profil: dane najmu tylko przy najmie, z okresem, osobami i odpowiedzią o innym lokalu")
def profile_rental_fields():
    room = new_room()
    room.run("zaloz_profil_poszukiwania", id="profil", nazwa="Mieszkanie do wynajęcia")
    rental = {
        "cel": "najem",
        "budzet": 320_000,
        "najem_od": "2026-12-01",
        "osoby": 2,
        "okazjonalny_dopuszcza": 0,
    }

    room.refused(
        "dane_najmu_tylko_przy_najmie",
        "wpisz_wersje_profilu",
        **profile_fields("wersja-1", osoby=2),
    )
    room.refused(
        "najem_ma_okres_osoby_i_rodzaj",
        "wpisz_wersje_profilu",
        **profile_fields("wersja-1", **{**rental, "osoby": None}),
    )
    room.refused(
        "okazjonalny_z_odpowiedzia_o_innym_lokalu",
        "wpisz_wersje_profilu",
        **profile_fields("wersja-1", **{**rental, "okazjonalny_dopuszcza": 1}),
    )
    room.refused(
        "na_wynajem_przy_kupnie",
        "wpisz_wersje_profilu",
        **profile_fields("wersja-1", **rental, na_wynajem=1),
    )
    room.refused(
        "powierzchnia_od_do",
        "wpisz_wersje_profilu",
        **profile_fields("wersja-1", pow_min=6000, pow_max=4000),
    )
    room.run(
        "wpisz_wersje_profilu",
        **profile_fields("wersja-1", **{**rental, "okazjonalny_dopuszcza": 1}, inny_lokal_ma=1),
    )


# poszukiwanie-kredyt -----------------------------------------------------------------------------


@scenario("Kredyt: dana z dokumentem z plikiem albo deklaracja klienta; danej się nie zmienia")
def credit_data_source():
    room = new_room()
    room.run(
        "wpisz_wymagany_dokument",
        id="wymagany",
        rodzaj="zaswiadczenie",
        nazwa="Zaświadczenie o dochodach",
        wymaga="bank",
    )
    data = {
        "id": "dochod",
        "rodzaj": "dochod",
        "opis": "umowa o pracę, netto",
        "wartosc": 900_000,
        "wpisano": "2026-11-07T09:00:00Z",
    }

    room.refused("dana_z_dokumentu_albo_deklaracja", "wpisz_dana_kredytowa", **data, deklaracja=0)
    room.refused(
        "dana wejściowa wskazuje dokument z plikiem",
        "wpisz_dana_kredytowa",
        **data,
        dokument_id="wymagany",
        deklaracja=0,
    )
    room.run("wpisz_dana_kredytowa", **data, deklaracja=1)
    room.refused_sql(
        "danej wejściowej się nie zmienia", "UPDATE poszukiwanie_kredyt_dane SET wartosc = 1"
    )


@scenario("Kredyt: do programu idą tylko dane z dokumentów wpisane przed analizą")
def credit_program_inputs():
    room = new_room()
    income_from_document(room)
    income_from_document(room, "pozniejszy", wpisano="2026-11-08T09:00:00Z")
    room.run(
        "wpisz_dana_kredytowa",
        id="deklaracja",
        rodzaj="zobowiazanie",
        opis="rata według klienta",
        wartosc=80_000,
        deklaracja=1,
        wpisano="2026-11-07T09:10:00Z",
    )
    credit_analysis(room, "analiza")
    rule = "do programu idą tylko dane z dokumentów wpisane przed analizą"

    room.refused(rule, "dopisz_dana_do_analizy", analiza_id="analiza", dana_id="deklaracja")
    room.refused(rule, "dopisz_dana_do_analizy", analiza_id="analiza", dana_id="pozniejszy")
    room.run("dopisz_dana_do_analizy", analiza_id="analiza", dana_id="dochod")
    room.refused(REPLACEMENT, "dopisz_dana_do_analizy", analiza_id="analiza", dana_id="dochod")


@scenario("Kredyt: analiza poza standardem ma zatwierdzoną decyzję, która ją wskazuje (R2)")
def credit_outside_standard():
    room = new_room()
    rule = "analiza poza standardem wymaga zatwierdzonej decyzji inna"

    ask_for_decision(
        room,
        "czeka",
        "inna",
        "poszukiwanie_kredyt_analizy",
        "analiza",
        prosi_stanowisko="analityk_kredytowy",
    )
    room.refused(
        rule,
        "wpisz_analize_kredytowa",
        **credit_fields(
            "analiza", okres_lat=35, poza_standardem=1, poza_standardem_decyzja_id="czeka"
        ),
    )
    approved_other(room, "inna-analiza", "poszukiwanie_kredyt_analizy", "inna")
    room.refused(
        rule,
        "wpisz_analize_kredytowa",
        **credit_fields(
            "analiza", okres_lat=35, poza_standardem=1, poza_standardem_decyzja_id="inna-analiza"
        ),
    )
    room.refused(
        "poza_standardem_z_decyzja",
        "wpisz_analize_kredytowa",
        **credit_fields("analiza", poza_standardem_decyzja_id="inna-analiza"),
    )
    approved_other(room, "zgoda", "poszukiwanie_kredyt_analizy", "analiza")
    room.run(
        "wpisz_analize_kredytowa",
        **credit_fields(
            "analiza", okres_lat=35, poza_standardem=1, poza_standardem_decyzja_id="zgoda"
        ),
    )


def credit_fields(analysis_id, **fields):
    return {
        "id": analysis_id,
        "wersja_parametrow": "standard-2026-10",
        "okres_lat": 25,
        "poza_standardem": 0,
        "kwota_od": 52_000_000,
        "kwota_do": 58_000_000,
        "rata_od": 340_000,
        "rata_do": 380_000,
        "udzial_wkladu": 1800,
        "wynik_sciezka": f"analizy/{analysis_id}.json",
        "wynik_skrot": f"sha256-{analysis_id}",
        "wyjasnienie": "Wkład około 18% ceny.",
        "zastrzezenie": STANDARD_DISCLAIMER,
        "utworzono": "2026-11-07T10:00:00Z",
        **fields,
    }


@scenario(
    "Kredyt: analiza ma zastrzeżenie, nowa odbiera aktualność poprzedniej, wyniku się nie zmienia"
)
def credit_analysis_current():
    room = new_room()

    room.refused(
        "zastrzezenie", "wpisz_analize_kredytowa", **credit_fields("pusta", zastrzezenie="")
    )
    room.refused(
        "kwota_od_do", "wpisz_analize_kredytowa", **credit_fields("odwrotna", kwota_od=60_000_000)
    )
    room.refused_sql(
        "analiza powstaje jako aktualna",
        "INSERT INTO poszukiwanie_kredyt_analizy (id, wersja_parametrow, okres_lat, poza_standardem, "
        "kwota_od, kwota_do, rata_od, rata_do, udzial_wkladu, wynik_sciezka, wynik_skrot, "
        "wyjasnienie, zastrzezenie, aktualna, utworzono) VALUES ('stara', 'p', 25, 0, 1, 2, 1, 2, "
        "1800, 'a.json', 'sha256-a', 'w', 'z', 0, '2026-11-07')",
    )
    credit_analysis(room, "analiza-1")
    credit_analysis(room, "analiza-2", utworzono="2026-11-20T10:00:00Z")
    assert room.run("analiza_aktualna")[0]["id"] == "analiza-2"

    rule = "wyniku analizy się nie zmienia"
    room.refused_sql(
        rule, "UPDATE poszukiwanie_kredyt_analizy SET kwota_do = 1 WHERE id = 'analiza-2'"
    )
    room.refused_sql(
        rule, "UPDATE poszukiwanie_kredyt_analizy SET aktualna = 1 WHERE id = 'analiza-1'"
    )


@scenario("Kredyt: bank wybiera klient wiadomością; wymagania banku i kroki harmonogramu raz")
def credit_bank_and_schedule():
    room = new_room()
    client(room)
    add_inflow(room, "wybor-banku", "klient", nadawca="Marek")
    bank = {"bank": "Bank Przykładowy S.A.", "wybrano": "2026-11-07"}

    room.refused("NOT NULL constraint failed", "wpisz_bank_klienta", **bank)
    room.run("wpisz_bank_klienta", **bank, zrodlo_wplyw_id="wybor-banku")
    room.refused(
        "wymagania_ze_zrodlem_i_data",
        "odnotuj_wymagania_banku",
        id=1,
        wymagania_zrodlo="strona banku",
    )
    room.run(
        "odnotuj_wymagania_banku",
        id=1,
        wymagania_zrodlo="strona banku",
        wymagania_data="2026-11-07",
    )
    rule = "wyboru banku się nie zmienia"
    room.refused(
        rule, "odnotuj_wymagania_banku", id=1, wymagania_zrodlo="pismo", wymagania_data="2026-11-08"
    )
    room.refused_sql(rule, "UPDATE poszukiwanie_kredyt_banki SET bank = 'Inny Bank'")

    room.run("wpisz_krok_harmonogramu", krok="decyzja_kredytowa", przewidywana="2026-12-01")
    room.run("przelicz_krok_harmonogramu", id=1, przewidywana="2026-12-05")
    room.run("odnotuj_krok_harmonogramu", id=1, rzeczywista="2026-12-04")
    rule = "kroku harmonogramu po fakcie się nie zmienia"
    room.refused(rule, "przelicz_krok_harmonogramu", id=1, przewidywana="2026-12-06")
    room.refused(rule, "odnotuj_krok_harmonogramu", id=1, rzeczywista="2026-12-03")


# poszukiwanie-propozycje -------------------------------------------------------------------------


@scenario("Propozycja wskazuje bieżącą wersję profilu, którą klient potwierdził")
def proposal_current_profile():
    room = new_room()
    search_case(room)
    confirmed_profile(room)
    profile_version(room, "wersja-2", od="2026-11-06T08:00:00Z")
    rule = "propozycja wskazuje bieżącą wersję profilu potwierdzoną przez klienta"

    room.refused(rule, "wpisz_propozycje", **proposal_fields(profil_wersja_id="wersja-2"))
    confirm_profile(room, "wersja-2")
    room.refused(rule, "wpisz_propozycje", **proposal_fields(profil_wersja_id="wersja-1"))
    room.run("wpisz_propozycje", **proposal_fields(profil_wersja_id="wersja-2"))


def proposal_fields(**fields):
    return {
        "id": "propozycja",
        "profil_wersja_id": "wersja-1",
        "zrodlo": "portal",
        "ogloszenie_sciezka": "propozycje/propozycja.html",
        "ogloszenie_skrot": "sha256-propozycja",
        "portal": "otodom.pl",
        "numer": "OT-445566",
        "prowadzi": "wlasciciel",
        "niepubliczna": 0,
        "zgodnosc": "3 pokoje, Krzyki",
        "cena_ogloszenia": 66_000_000,
        "utworzono": "2026-11-06T09:00:00Z",
        **fields,
    }


@scenario("Propozycja: z portalu ma kopię i numer, z katalogu odnośnik i datę wersji")
def proposal_source():
    room = new_room()
    search_case(room)
    confirmed_profile(room)

    room.refused("z_portalu_z_kopia_i_numerem", "wpisz_propozycje", **proposal_fields(numer=None))
    room.refused(
        "z_katalogu_z_odnosnikiem_i_wersja",
        "wpisz_propozycje",
        **catalog_proposal(katalog_wersja_od=None),
    )
    room.refused(
        "niepubliczna_tylko_z_katalogu", "wpisz_propozycje", **proposal_fields(niepubliczna=1)
    )
    room.refused(
        "z_katalogu_prowadzi_opiekun_oferty",
        "wpisz_propozycje",
        **catalog_proposal(prowadzi="wlasciciel"),
    )
    room.run("wpisz_propozycje", **catalog_proposal())


def catalog_proposal(**fields):
    return proposal_fields(
        **{
            "zrodlo": "katalog_biura",
            "ogloszenie_sciezka": None,
            "ogloszenie_skrot": None,
            "portal": None,
            "numer": None,
            "katalog_odnosnik": "katalog-ofert/oferta-041",
            "katalog_wersja_od": "2026-11-05T10:00:00Z",
            "prowadzi": "opiekun_oferty",
            "niepubliczna": 1,
            **fields,
        }
    )


@scenario(
    "Propozycja: przedstawienie przy zawartej umowie, nie wcześniej niż praca od (art. 15 upk)"
)
def proposal_presented_after_work_starts():
    room = new_room()
    search_case(room, sposob_zawarcia="na_odleglosc", odstapienie_do=WITHDRAWAL_ENDS)
    confirmed_profile(room)
    proposal(room)
    rule = (
        "propozycję przedstawia się przy zawartej umowie pośrednictwa, nie wcześniej niż praca od"
    )

    room.refused(
        rule,
        "odnotuj_przedstawienie_propozycji",
        id="propozycja",
        przedstawiono="2026-11-18T10:00:00Z",
    )
    room.run(
        "odnotuj_przedstawienie_propozycji", id="propozycja", przedstawiono="2026-11-19T10:00:00Z"
    )
    room.refused(
        "propozycji się nie zmienia",
        "odnotuj_przedstawienie_propozycji",
        id="propozycja",
        przedstawiono="2026-11-20T10:00:00Z",
    )


@scenario("Propozycja: reakcja po przedstawieniu; po „czeka” klient może odpowiedzieć jeszcze raz")
def proposal_reaction():
    room = new_room()
    search_case(room)
    confirmed_profile(room)
    proposal(room)

    room.refused(
        "reakcja_po_przedstawieniu",
        "odnotuj_reakcje_na_propozycje",
        id="propozycja",
        reakcja="czeka",
    )
    room.run(
        "odnotuj_przedstawienie_propozycji", id="propozycja", przedstawiono="2026-11-06T10:00:00Z"
    )
    room.run("odnotuj_reakcje_na_propozycje", id="propozycja", reakcja="czeka")
    room.run(
        "odnotuj_reakcje_na_propozycje",
        id="propozycja",
        reakcja="odrzuca",
        reakcja_powod="za daleko od szkoły",
    )
    room.refused(
        "propozycji się nie zmienia",
        "odnotuj_reakcje_na_propozycje",
        id="propozycja",
        reakcja="chce_obejrzec",
    )


@scenario("Propozycja: wynik przeglądania księgi z numerem, raz; źródła i ceny się nie zmienia")
def proposal_land_register():
    room = new_room()
    search_case(room)
    confirmed_profile(room)
    proposal(room)

    room.refused(
        "przegladanie_ksiegi_z_numerem_i_wynikiem",
        "odnotuj_przegladanie_ksiegi_propozycji",
        id="propozycja",
        numer_kw="WR1K/00098765/4",
        kw_przegladano="2026-11-06T11:00:00Z",
    )
    room.run(
        "odnotuj_przegladanie_ksiegi_propozycji",
        id="propozycja",
        numer_kw="WR1K/00098765/4",
        kw_przegladano="2026-11-06T11:00:00Z",
        kw_ogloszajacy_wpisany="nie",
    )
    rule = "propozycji się nie zmienia"
    room.refused(
        rule,
        "odnotuj_przegladanie_ksiegi_propozycji",
        id="propozycja",
        numer_kw="WR1K/00098765/4",
        kw_przegladano="2026-11-07T11:00:00Z",
        kw_ogloszajacy_wpisany="tak",
    )
    room.refused_sql(rule, "UPDATE poszukiwanie_propozycje_wpisy SET cena_ogloszenia = 1")

    room.run("wpisz_wymagany_dokument", id="wymagany", rodzaj="inny", nazwa="Ceny transakcji")
    room.refused(
        "ceny transakcji z rejestru cen to dokument z plikiem",
        "odnotuj_ceny_transakcji_propozycji",
        id="propozycja",
        ceny_rcn_dokument_id="wymagany",
    )


@scenario("Prezentacja: tylko przedstawionej propozycji; szerszy zakres danych klienta za zgodą")
def proposal_viewing():
    room = new_room()
    search_case(room)
    confirmed_profile(room)
    proposal(room)

    room.refused(
        "prezentacja dotyczy propozycji przedstawionej klientowi",
        "umow_prezentacje_propozycji",
        **viewing_fields("prezentacja"),
    )
    room.run(
        "odnotuj_przedstawienie_propozycji", id="propozycja", przedstawiono="2026-11-06T10:00:00Z"
    )
    rule = "szerszy zakres danych klienta wymaga zatwierdzonej decyzji inna"
    approved_other(room, "inna-prezentacja", "poszukiwanie_propozycje_prezentacje", "inna")
    room.refused(
        rule,
        "umow_prezentacje_propozycji",
        **viewing_fields(
            "prezentacja", szerszy_zakres=1, szerszy_zakres_decyzja_id="inna-prezentacja"
        ),
    )
    room.refused(
        "szerszy_zakres_z_decyzja",
        "umow_prezentacje_propozycji",
        **viewing_fields("prezentacja", szerszy_zakres_decyzja_id="inna-prezentacja"),
    )
    approved_other(room, "szerszy-zakres", "poszukiwanie_propozycje_prezentacje", "prezentacja")
    room.run(
        "umow_prezentacje_propozycji",
        **viewing_fields(
            "prezentacja",
            dane_klienta_podane="imię, nazwisko, telefon, adres e-mail i pracodawca",
            szerszy_zakres=1,
            szerszy_zakres_decyzja_id="szerszy-zakres",
        ),
    )


def viewing_fields(viewing_id, **fields):
    return {
        "id": viewing_id,
        "propozycja_id": "propozycja",
        "termin": "2026-11-07T17:00:00Z",
        "kto_pokazuje": "wlasciciel",
        "dane_klienta_podane": "imię, nazwisko, telefon",
        "szerszy_zakres": 0,
        **fields,
    }


@scenario("Prezentacja: opinia i notatka raz, odwołana bez opinii; pytanie otwarte, odpowiedź raz")
def proposal_viewing_record():
    room = new_room()
    offer_ready(room)
    room.run("umow_prezentacje_propozycji", **viewing_fields("prezentacja"))
    room.run("umow_prezentacje_propozycji", **viewing_fields("odwolana"))
    rule = "prezentacji się nie zmienia"

    room.run("odnotuj_opinie_klienta", id="prezentacja", opinia_klienta="Za ciemno.")
    room.refused(rule, "odnotuj_opinie_klienta", id="prezentacja", opinia_klienta="Jasno.")
    room.refused_sql(rule, "UPDATE poszukiwanie_propozycje_prezentacje SET termin = '2026-11-08'")
    room.refused(
        "odwolana_bez_opinii_i_notatki",
        "odwolaj_prezentacje_propozycji",
        id="prezentacja",
        odwolano="2026-11-07T12:00:00Z",
        odwolal="właściciel",
        powod_odwolania="wyjazd",
    )
    room.run(
        "odwolaj_prezentacje_propozycji",
        id="odwolana",
        odwolano="2026-11-07T12:00:00Z",
        odwolal="właściciel",
        powod_odwolania="wyjazd",
    )
    room.refused(rule, "odnotuj_notatke_posrednika", id="odwolana", notatka_posrednika="x")

    room.run("dopisz_pytanie_do_prezentacji", prezentacja_id="prezentacja", pytanie="Ile czynszu?")
    add_inflow(room, "odpowiedz", nadawca="Jan Nowak")
    room.refused_sql(
        "odpowiedz_z_wplywem",
        "UPDATE poszukiwanie_propozycje_pytania SET odpowiedz = '420 zł' WHERE id = 1",
    )
    room.run(
        "odnotuj_odpowiedz_na_pytanie", id=1, odpowiedz="420 zł", odpowiedz_wplyw_id="odpowiedz"
    )
    room.refused(
        "pytania się nie zmienia",
        "odnotuj_odpowiedz_na_pytanie",
        id=1,
        odpowiedz="450 zł",
        odpowiedz_wplyw_id="odpowiedz",
    )

    room.refused_sql(
        "pytanie powstaje otwarte",
        "INSERT INTO poszukiwanie_propozycje_pytania (prezentacja_id, pytanie, odpowiedz, "
        "odpowiedz_wplyw_id) VALUES ('prezentacja', 'Czy jest piwnica?', 'tak', 'odpowiedz')",
    )
    room.refused_sql(
        "prezentacja powstaje umówiona",
        "INSERT INTO poszukiwanie_propozycje_prezentacje (id, propozycja_id, termin, kto_pokazuje, "
        "dane_klienta_podane, szerszy_zakres, opinia_klienta) VALUES ('z-opinia', 'propozycja', "
        "'2026-11-07', 'wlasciciel', 'imię', 0, 'dobra')",
    )
    room.refused_sql(
        "propozycja powstaje nieprzedstawiona",
        "INSERT INTO poszukiwanie_propozycje_wpisy (id, profil_wersja_id, zrodlo, prowadzi, "
        "niepubliczna, zgodnosc, cena_ogloszenia, utworzono, przedstawiono) VALUES ('od-razu', "
        "'wersja-1', 'inne_biuro', 'inne_biuro', 0, 'tak', 1, '2026-11-06', '2026-11-06')",
    )


# poszukiwanie-oferty -----------------------------------------------------------------------------


@scenario("Oferta klienta dotyczy przedstawionej propozycji i celu jej profilu")
def offer_concerns_presented_proposal():
    room = new_room()
    search_case(room)
    confirmed_profile(room)
    proposal(room)
    rule = "oferta dotyczy propozycji przedstawionej klientowi i celu jej profilu"

    room.refused(rule, "przygotuj_oferte_klienta", **offer_fields())
    room.run(
        "odnotuj_przedstawienie_propozycji", id="propozycja", przedstawiono="2026-11-06T10:00:00Z"
    )
    room.refused(rule, "przygotuj_oferte_klienta", **offer_fields(dotyczy="najem"))
    room.refused(
        "oferta_ma_termin_albo_sposob_zlozenia",
        "przygotuj_oferte_klienta",
        **offer_fields(wiaze_do=None),
    )
    room.run("przygotuj_oferte_klienta", **offer_fields())
    room.refused_sql(
        "oferty się nie zmienia", "UPDATE poszukiwanie_oferty_wpisy SET kwota = 62000000"
    )


def offer_fields(**fields):
    return {
        "id": "oferta",
        "dotyczy": "kupno",
        "propozycja_id": "propozycja",
        "adresat": "Jan Nowak, właściciel, bez biura",
        "wpisano": "2026-11-08T09:00:00Z",
        "kwota": 61_000_000,
        "wiaze_do": "2026-11-12",
        **fields,
    }


@scenario(
    "Oferta klienta: potwierdza ją klient zgodą potwierdzenie_oferty przed wskazaniem wysyłki"
)
def offer_confirmed_by_client():
    room = new_room()
    offer_ready(room)
    add_person(room, "partnerka", imiona="Ewa", nazwisko="Lis")
    client_offer(room)
    prepare_offer_letter(room, "wysylka")
    rule = "oferta klienta i jego decyzja mają zgodę potwierdzenie_oferty klienta"

    room.refused(
        "wysylka_po_potwierdzeniu",
        "wskaz_wysylke_oferty_klienta",
        id="oferta",
        wysylka_id="wysylka",
    )
    client_consent(
        room,
        "zgoda-partnerki",
        "potwierdzenie_oferty",
        "poszukiwanie_oferty_wpisy",
        "oferta",
        "partnerka",
    )
    room.refused(rule, "potwierdz_oferte_klienta", id="oferta", zgoda_id="zgoda-partnerki")
    client_consent(
        room, "zgoda-profilu", "potwierdzenie_profilu", "poszukiwanie_oferty_wpisy", "oferta"
    )
    room.refused(rule, "potwierdz_oferte_klienta", id="oferta", zgoda_id="zgoda-profilu")
    confirm_offer(room, "oferta")
    room.run("wskaz_wysylke_oferty_klienta", id="oferta", wysylka_id="wysylka")
    room.refused(
        "oferty się nie zmienia", "potwierdz_oferte_klienta", id="oferta", zgoda_id="zgoda-profilu"
    )


@scenario("Oferta klienta wychodzi tylko za zgodą człowieka, przy niewysłanej wysyłce")
def offer_sent_with_consent():
    room = new_room()
    offer_ready(room)
    client_offer(room)
    confirm_offer(room, "oferta")
    prepare_offer_letter(room, "wyslana")
    send(room, "wyslana")

    room.refused(
        "oferta i odpowiedź wskazują wysyłkę, która jeszcze nie wyszła",
        "wskaz_wysylke_oferty_klienta",
        id="oferta",
        wysylka_id="wyslana",
    )
    prepare_offer_letter(room, "wysylka")
    room.run("wskaz_wysylke_oferty_klienta", id="oferta", wysylka_id="wysylka")
    room.refused(
        "oferta klienta i odpowiedź przed wysłaniem czekają na zgodę",
        "odnotuj_wyslanie",
        id="wysylka",
        wyslano="2026-11-08T12:00:00Z",
    )
    letter_awaiting_consent(room, "wysylka")
    send(room, "wysylka", decyzja_id="wyslanie-wysylka", wyslano="2026-11-08T12:00:00Z")


@scenario("Oferta klienta nie wychodzi przed praca od ani przy znaczniku wątpliwości")
def offer_not_before_work_nor_with_doubt():
    room = new_room()
    # A consumer's contract at a distance without his demand: work starts on 2026-11-19. The office
    # may prepare the offer earlier, but it goes out only from that day.
    search_case(room, sposob_zawarcia="na_odleglosc", odstapienie_do=WITHDRAWAL_ENDS)
    confirmed_profile(room)
    proposal(room)
    room.run(
        "odnotuj_przedstawienie_propozycji", id="propozycja", przedstawiono="2026-11-19T08:00:00Z"
    )
    client_offer(room)
    confirm_offer(room, "oferta")
    prepare_offer_letter(room, "wysylka")
    room.run("wskaz_wysylke_oferty_klienta", id="oferta", wysylka_id="wysylka")
    letter_awaiting_consent(room, "wysylka")

    room.refused(
        "oferta klienta wychodzi przy zawartej umowie pośrednictwa, nie wcześniej niż praca od",
        "odnotuj_wyslanie",
        id="wysylka",
        wyslano="2026-11-18T12:00:00Z",
        decyzja_id="wyslanie-wysylka",
    )
    room.run(
        "postaw_watpliwosc",
        id="watpliwosc",
        postawil="tozsamosc-kontrolera",
        postawiono="2026-11-18T13:00:00Z",
    )
    room.refused(
        "dopóki stoi znacznik wątpliwości, oferta klienta i odpowiedź nie wychodzą",
        "odnotuj_wyslanie",
        id="wysylka",
        wyslano="2026-11-19T12:00:00Z",
        decyzja_id="wyslanie-wysylka",
    )
    ask_for_decision(
        room,
        "zdjecie",
        "zdjecie_watpliwosci",
        "sprawa_metryka_watpliwosci",
        "watpliwosc",
        prosi_stanowisko="kontroler",
        zlozono="2026-11-18T14:00:00Z",
    )
    approve(room, "zdjecie", "2026-11-18T15:00:00Z")
    room.run("zdejmij_watpliwosc", id="watpliwosc", zdjecie_decyzja_id="zdjecie")
    send(room, "wysylka", decyzja_id="wyslanie-wysylka", wyslano="2026-11-19T12:00:00Z")


@scenario("Kontroferty: druga strona odpowiada na wysłaną ofertę, klient na jej kontrofertę")
def offer_counter_chain():
    room = new_room()
    offer_ready(room)
    client_offer(room, "niewyslana")
    sent_client_offer(room)
    add_inflow(room, "wplyw", nadawca="Jan Nowak")

    room.refused(
        "odpowiedz_drugiej_strony_z_wplywem",
        "odnotuj_odpowiedz_drugiej_strony",
        id="niewyslana",
        odpowiedz_drugiej_strony="odrzucona",
        odpowiedz_wplyw_id="wplyw",
    )
    rule = "kontroferta odpowiada na ofertę tej samej propozycji"
    room.refused(rule, "wpisz_kontroferte_drugiej_strony", **counter_fields("oferta"))
    counteroffer_of_other_side(room, "kontroferta", "oferta")
    room.refused(
        rule, "przygotuj_oferte_klienta", **offer_fields(id="kontra", odpowiada_na_id="kontroferta")
    )
    room.run(
        "odnotuj_przedstawienie_kontroferty", id="kontroferta", przedstawiono="2026-11-09T10:00:00Z"
    )
    room.run(
        "odnotuj_decyzje_klienta_o_kontrofercie", id="kontroferta", decyzja_klienta="kontroferta"
    )
    room.run("przygotuj_oferte_klienta", **offer_fields(id="kontra", odpowiada_na_id="kontroferta"))


def counter_fields(odpowiada_na_id):
    return {
        "id": "kontroferta",
        "dotyczy": "kupno",
        "propozycja_id": "propozycja",
        "adresat": "Jan Nowak",
        "odpowiada_na_id": odpowiada_na_id,
        "wplyw_id": "wplyw",
        "wpisano": "2026-11-09T09:00:00Z",
        "kwota": 66_000_000,
        "sposob_zlozenia": "na_pismie",
    }


@scenario(
    "Kontroferta drugiej strony: przyjęcie i odrzucenie ze zgodą klienta, odpowiedź za zgodą człowieka"
)
def counteroffer_decision():
    room = new_room()
    offer_ready(room)
    sent_client_offer(room)
    counteroffer_of_other_side(room, "kontroferta", "oferta")

    room.refused(
        "decyzja_po_przedstawieniu",
        "odnotuj_decyzje_klienta_o_kontrofercie",
        id="kontroferta",
        decyzja_klienta="kontroferta",
    )
    room.run(
        "odnotuj_przedstawienie_kontroferty", id="kontroferta", przedstawiono="2026-11-09T10:00:00Z"
    )
    room.refused(
        "decyzja_klienta_ze_zgoda",
        "odnotuj_decyzje_klienta_o_kontrofercie",
        id="kontroferta",
        decyzja_klienta="przyjeta",
    )
    client_consent(
        room,
        "zgoda-kontroferty",
        "potwierdzenie_oferty",
        "poszukiwanie_oferty_wpisy",
        "kontroferta",
    )
    room.refused(
        "decyzja_klienta_ze_zgoda",
        "odnotuj_decyzje_klienta_o_kontrofercie",
        id="kontroferta",
        decyzja_klienta="kontroferta",
        zgoda_id="zgoda-kontroferty",
    )
    room.run(
        "odnotuj_decyzje_klienta_o_kontrofercie",
        id="kontroferta",
        decyzja_klienta="przyjeta",
        zgoda_id="zgoda-kontroferty",
    )

    prepare_offer_letter(room, "przyjecie")
    room.run("wskaz_odpowiedz_na_kontroferte", id="kontroferta", odpowiedz_wysylka_id="przyjecie")
    room.refused(
        "oferta klienta i odpowiedź przed wysłaniem czekają na zgodę",
        "odnotuj_wyslanie",
        id="przyjecie",
        wyslano="2026-11-09T12:00:00Z",
    )
    letter_awaiting_consent(room, "przyjecie")
    send(room, "przyjecie", decyzja_id="wyslanie-przyjecie", wyslano="2026-11-09T12:00:00Z")
    room.refused(
        "oferty się nie zmienia",
        "odnotuj_decyzje_klienta_o_kontrofercie",
        id="kontroferta",
        decyzja_klienta="odrzucona",
        zgoda_id="zgoda-kontroferty",
    )


@scenario("Oferta powstaje bez potwierdzenia, wysyłki, odpowiedzi i decyzji")
def offer_starts_without_course():
    room = new_room()
    offer_ready(room)
    client_consent(room, "zgoda", "potwierdzenie_oferty", "poszukiwanie_oferty_wpisy", "od-razu")

    room.refused_sql(
        "oferta powstaje bez potwierdzenia, wysyłki, odpowiedzi i decyzji",
        "INSERT INTO poszukiwanie_oferty_wpisy (id, dotyczy, kierunek, propozycja_id, adresat, "
        "wpisano, kwota, wiaze_do, zgoda_id) VALUES ('od-razu', 'kupno', 'od_klienta', "
        "'propozycja', 'Jan Nowak', '2026-11-08', 1, '2026-11-12', 'zgoda')",
    )
    room.refused_sql(
        "kontroferta_drugiej_strony_z_wplywem_w_lancuchu",
        "INSERT INTO poszukiwanie_oferty_wpisy (id, dotyczy, kierunek, propozycja_id, adresat, "
        "wpisano, kwota, wiaze_do) VALUES ('bez-wplywu', 'kupno', 'od_drugiej_strony', "
        "'propozycja', 'Jan Nowak', '2026-11-08', 1, '2026-11-12')",
    )


# R9 and replacement ------------------------------------------------------------------------------


def search_tables(room):
    return [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' "
            "AND name LIKE 'poszukiwanie\\_%' ESCAPE '\\'"
        )
    ]


@scenario("R9: wierszy teczki poszukiwania się nie usuwa")
def no_deletion():
    room = new_room()
    full_case(room)

    tables = search_tables(room)
    assert len(tables) == 13, tables
    for table in tables:
        assert room.value(f"SELECT count(*) FROM {table}") > 0, f"{table} has no row to delete"
        room.refused_sql(
            DELETION, f"DELETE FROM {table} WHERE rowid = (SELECT min(rowid) FROM {table})"
        )


@scenario("Wiersza teczki poszukiwania nie zastępuje INSERT OR REPLACE ani upsert")
def no_replacement():
    room = new_room()
    full_case(room)

    for table in search_tables(room):
        room.refused_sql(REPLACEMENT, f"INSERT OR REPLACE {first_row_copy(room, table)}")
        room.refused_sql(
            REPLACEMENT,
            f"INSERT {first_row_copy(room, table)} ON CONFLICT (id) DO UPDATE SET id = excluded.id",
        )
        if unique_keys_of(room, table):
            room.refused_sql(
                REPLACEMENT,
                f"INSERT OR REPLACE {first_row_copy(room, table, replace_unique_key=True)}",
            )
