"""Scenarios of the offer case catalog (spec/cases/oferta) on top of the core.

An offer room is the core of every case plus the kinds of the offer type. Each scenario starts
from an empty room and checks one rule the database keeps: a write the rule allows and a write it
refuses. The first scenario follows a whole offer case as the profile describes it, from intake
through the brokerage contract, price, advertisement, viewing and the chain of offers to closing.
"""

from pathlib import Path

from core import (
    WITHDRAWAL_ENDS,
    add_consent,
    add_flat,
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

OFERTA = Path(__file__).resolve().parents[1] / "spec" / "cases" / "oferta"

SCENARIOS = []

CONCLUDED = "2026-11-04T12:00:00Z"
REPLACEMENT = "wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE"
DELETION = "wierszy teczki się nie usuwa (R9)"


def scenario(title):
    def register(function):
        SCENARIOS.append((title, function))

        return function

    return register


def new_room():
    return Room(OFERTA)


# Building blocks of an offer case ----------------------------------------------------------------


def approved_other(room, decision_id, dotyczy_tabela, dotyczy_id, rodzaj="inna"):
    """A decision of the broker on a row of the offer type, asked by the offer keeper."""
    ask_for_decision(
        room,
        decision_id,
        rodzaj,
        dotyczy_tabela,
        dotyczy_id,
        prosi_stanowisko="opiekun_oferty",
    )
    approve(room, decision_id)


def client(room, person_id="klientka", **fields):
    add_person(room, person_id, konsument=1, klient_biura=1, **fields)
    room.run("nadaj_role_uczestnikowi", uczestnik_id=person_id, rola="klient")


def brokerage_contract(
    room,
    contract_id="umowa",
    party_id="klientka",
    czynnosci=("publikacja_ogloszen", "prezentacje"),
    obie_strony=0,
    sposob_zawarcia="w_lokalu_biura",
    odstapienie_do=None,
):
    """A brokerage contract for the flat, checked, signed and concluded on 2026-11-04."""
    check(room, f"tozsamosc-{contract_id}", "tozsamosc", uczestnik_id=party_id)
    prepare_contract(
        room,
        contract_id,
        [party_id],
        rodzaj="posrednictwo",
        nieruchomosc_id="lokal",
        obie_strony=obie_strony,
        wynagrodzenie_opis="2,5% ceny",
        wynagrodzenie_zdarzenie="zawarcie umowy przyrzeczonej",
    )
    for czynnosc in czynnosci:
        room.run("dopisz_czynnosc_umowy", umowa_id=contract_id, czynnosc=czynnosc)
    room.run("przekaz_umowe_do_podpisu", id=contract_id)
    conclude_contract(room, contract_id, sposob_zawarcia, odstapienie_do)


def offer_case(room, **contract_fields):
    """An offer case with the client, the flat and a concluded brokerage contract."""
    open_case(room, typ_pokoju="oferta")
    client(room)
    add_flat(room)
    brokerage_contract(room, **contract_fields)


def visibility(room, version_id="widocznosc", rodzaj_oferty="sprzedaz", **fields):
    room.run(
        "wpisz_wersje_widocznosci",
        **{
            "id": version_id,
            "umowa_id": "umowa",
            "rodzaj_oferty": rodzaj_oferty,
            "widocznosc": "publiczna",
            "od": CONCLUDED,
            **fields,
        },
    )


def client_consent(room, consent_id, rodzaj, dotyczy_tabela, dotyczy_id, person_id="klientka"):
    add_consent(
        room,
        consent_id,
        person_id,
        rodzaj,
        tresc=f"Zgoda {rodzaj}.",
        dotyczy_tabela=dotyczy_tabela,
        dotyczy_id=dotyczy_id,
    )


def offer_price(room, price_id="cena", kwota=65_900_000, dotyczy="sprzedaz", **fields):
    client_consent(
        room, f"zgoda-{price_id}", "zatwierdzenie_ceny", "oferta_ceny_ofertowe", price_id
    )
    room.run(
        "wpisz_cene_ofertowa",
        **{
            "id": price_id,
            "dotyczy": dotyczy,
            "kwota": kwota,
            "zgoda_id": f"zgoda-{price_id}",
            "obowiazuje_od": "2026-11-04T14:00:00Z",
            **fields,
        },
    )


def draft_ad(room, ad_id="ogloszenie", cena=65_900_000, dotyczy="sprzedaz", **version_fields):
    room.run("przygotuj_ogloszenie", id=ad_id, dotyczy=dotyczy, kanal="portal", portal="otodom.pl")
    add_ad_version(room, ad_id, 1, cena=cena, **version_fields)


def add_ad_version(room, ad_id, numer, cena=65_900_000, od="2026-11-05T09:00:00Z", **fields):
    room.run(
        "dopisz_wersje_ogloszenia",
        **{
            "ogloszenie_id": ad_id,
            "numer": numer,
            "od": od,
            "tytul": "2 pokoje, 48 m², Krzyki",
            "cena": cena,
            "lokalizacja": "Wrocław, Krzyki, ul. Polna",
            "tresc_sciezka": f"ogloszenia/{ad_id}-{numer}.json",
            "tresc_skrot": f"sha256-{ad_id}-{numer}",
            **fields,
        },
    )


def publish(room, ad_id="ogloszenie", opublikowano="2026-11-05T10:00:00Z"):
    room.run("odnotuj_publikacje_ogloszenia", id=ad_id, opublikowano=opublikowano)


def ready_to_publish(room):
    """An offer case with public visibility, a price and an ad draft the broker approved."""
    offer_case(room)
    visibility(room)
    offer_price(room)
    draft_ad(room)
    approved_other(room, "pierwsza-publikacja", "oferta_ogloszenia_wpisy", "ogloszenie")


def interested_person(room, person_id="zainteresowany", **fields):
    room.run(
        "wpisz_zainteresowanego",
        **{
            "id": person_id,
            "imie": "Piotr",
            "telefon": "+48 600 100 200",
            "dziala_jako": "sam",
            "dane_pozyskano": "2026-11-06T09:00:00Z",
            **fields,
        },
    )


def privacy_notice(room, letter_id, adresat_nazwa="Piotr"):
    """A sent letter with the information on personal data (art. 13 RODO)."""
    prepare_letter(
        room,
        letter_id,
        adresat_rodzaj="inny",
        adresat_nazwa=adresat_nazwa,
        wzor="informacja-o-danych",
        informacja_rodo=1,
    )
    send(room, letter_id, wyslano="2026-11-06T09:30:00Z")


def offer_from_interested(room, offer_id, kwota=61_000_000, **fields):
    add_inflow(room, f"wplyw-{offer_id}", nadawca="Piotr <piotr@example.pl>")
    room.run(
        "wpisz_oferte_zainteresowanego",
        **{
            "id": offer_id,
            "dotyczy": "sprzedaz",
            "osoba_id": "zainteresowany",
            "wplyw_id": f"wplyw-{offer_id}",
            "wpisano": "2026-11-10T09:00:00Z",
            "kwota": kwota,
            "finansowanie": "kredyt hipoteczny, decyzja wstępna z banku",
            "wiaze_do": "2026-11-12",
            **fields,
        },
    )


def client_decision(room, offer_id, decyzja_klienta):
    room.run("odnotuj_przedstawienie_oferty", id=offer_id, przedstawiono="2026-11-10T10:00:00Z")
    client_consent(
        room, f"decyzja-{offer_id}", "odpowiedz_na_oferte", "oferta_oferty_wpisy", offer_id
    )
    room.run(
        "odnotuj_decyzje_klienta_o_ofercie",
        id=offer_id,
        decyzja_klienta=decyzja_klienta,
        zgoda_id=f"decyzja-{offer_id}",
    )


def reply_awaiting_consent(room, offer_id):
    """The reply to an offer, linked and waiting for the human's consent to send it."""
    letter_id = f"odpowiedz-{offer_id}"
    prepare_letter(
        room,
        letter_id,
        adresat_rodzaj="inny",
        adresat_nazwa="Piotr",
        wzor="odpowiedz-na-oferte",
    )
    room.run("wskaz_odpowiedz_na_oferte", id=offer_id, odpowiedz_wysylka_id=letter_id)
    room.run("przekaz_wysylke_do_zgody", id=letter_id)
    approved_other(
        room, f"wyslanie-{letter_id}", "sprawa_korespondencja_wysylki", letter_id, "wyslanie"
    )

    return letter_id


def stage(room, etap, od="2026-11-04T15:00:00Z", **fields):
    room.run("zmien_etap_sprawy", etap=etap, od=od, zmienil="agent-oferty", **fields)


def full_case(room):
    """An offer case with a row in every table of the offer type, as the profile goes."""
    open_case(room, typ_pokoju="oferta")
    stage(room, "weryfikacja_klienta", od="2026-11-02T10:00:00Z")

    # The client, her husband and the flat with its land register entries looked up.
    client(room)
    add_person(room, "maz", imiona="Jan", nazwisko="Kowalski")
    room.run(
        "nadaj_role_uczestnikowi",
        uczestnik_id="maz",
        rola="malzonek",
        dotyczy_uczestnik_id="klientka",
    )
    add_flat(room)
    room.run(
        "wpisz_przegladanie_ksiegi",
        id="przegladanie",
        nieruchomosc_id="lokal",
        numer_kw="WR1K/00012345/6",
        przegladano="2026-11-03T09:00:00Z",
        wlasciciele_dzial_ii="Anna Kowalska",
        wpisy_dzial_iv="hipoteka umowna 300 000 zł",
        rozbieznosc=0,
        wydruk_sciezka="ksiega/przegladanie.pdf",
        wydruk_skrot="sha256-przegladanie",
    )
    add_received_document(
        room,
        "swiadectwo",
        "swiadectwo_energetyczne",
        dotyczy_nieruchomosc_id="lokal",
        data_wystawienia="2020-05-10",
        waznosc_do="2030-05-10",
        waznosc_zrodlo="przepis",
    )
    room.run(
        "wpisz_opis_nieruchomosci",
        nieruchomosc_id="lokal",
        liczba_pieter=4,
        winda=0,
        rok_budowy=1978,
        ogrzewanie="miejskie",
        oplata_miesieczna=48_000,
        swiadectwo="jest",
        swiadectwo_dokument_id="swiadectwo",
        wskazniki_energetyczne="EP 120 kWh/(m²·rok)",
        osoby_w_lokalu="nikt nie mieszka",
    )
    room.run(
        "wpisz_zrodlo",
        dotyczy_tabela="oferta_nieruchomosc_opis",
        dotyczy_id="lokal",
        pole="oplata_miesieczna",
        od_uczestnik_id="klientka",
        potwierdzone=0,
        wpisano="2026-11-03T10:00:00Z",
    )

    # The brokerage contract signed at a distance by a consumer: the office starts work before
    # the end of the withdrawal period only on her demand.
    stage(room, "umowa_posrednictwa", od="2026-11-03T12:00:00Z")
    brokerage_contract(room, sposob_zawarcia="na_odleglosc", odstapienie_do=WITHDRAWAL_ENDS)
    add_consent(room, "zadanie", "klientka", "zadanie_rozpoczecia_pracy")
    room.run(
        "wpisz_zdarzenie_umowy",
        id="zdarzenie-zadanie",
        umowa_id="umowa",
        rodzaj="zadanie_rozpoczecia_pracy",
        chwila="2026-11-04T13:00:00Z",
        zgoda_id="zadanie",
    )
    stage(room, "przygotowanie_oferty", od="2026-11-04T13:30:00Z")
    visibility(room)

    # The price: the office's proposal from comparable sales, the price the client approved and
    # her lowest price.
    room.run(
        "wpisz_propozycje_ceny",
        id="propozycja",
        dotyczy="sprzedaz",
        kwota=64_000_000,
        uzasadnienie="pięć transakcji z ostatniego roku na tym samym osiedlu",
        utworzono="2026-11-04T13:40:00Z",
        wyliczenie_sciezka="ceny/propozycja.json",
        wyliczenie_skrot="sha256-propozycja",
    )
    room.run(
        "dopisz_porownanie_transakcji",
        propozycja_id="propozycja",
        data_transakcji="2026-03-14",
        cena=61_500_000,
        powierzchnia=4710,
        roznice="parter",
        zrodlo="rejestr cen nieruchomości, Wrocław",
    )
    offer_price(room)
    client_consent(room, "zgoda-granice", "granice_negocjacji", "oferta_ceny_granice", "granice")
    room.run(
        "wpisz_granice_negocjacji",
        id="granice",
        dotyczy="sprzedaz",
        najnizsza_kwota=63_000_000,
        warunki="wydanie lokalu najpóźniej w lutym",
        zgoda_id="zgoda-granice",
    )

    # The advertisement, approved by the broker before its first publication.
    draft_ad(room, wskazniki_energetyczne="EP 120 kWh/(m²·rok)")
    approved_other(room, "pierwsza-publikacja", "oferta_ogloszenia_wpisy", "ogloszenie")
    publish(room)
    room.run("odnotuj_numer_ogloszenia", id="ogloszenie", numer_ogloszenia="OT-778812")
    room.run(
        "wpisz_statystyki_ogloszenia",
        ogloszenie_id="ogloszenie",
        odczytano="2026-11-09T08:00:00Z",
        wyswietlenia=412,
        zapytania=3,
    )
    stage(room, "oferta_aktywna", od="2026-11-05T10:00:00Z")

    # An inquiry, the information on personal data, the answer and a viewing.
    interested_person(room)
    add_inflow(room, "zapytanie", nadawca="Piotr <piotr@example.pl>", kanal="portal")
    room.run("wpisz_zapytanie", osoba_id="zainteresowany", wplyw_id="zapytanie")
    privacy_notice(room, "informacja-piotr")
    room.run(
        "odnotuj_informacje_dla_zainteresowanego",
        id="zainteresowany",
        informacja_wysylka_id="informacja-piotr",
    )
    room.run("odnotuj_odpowiedz_na_zapytanie", id=1, odpowiedz_wysylka_id="informacja-piotr")
    room.run(
        "umow_prezentacje",
        id="prezentacja",
        termin="2026-11-07T17:00:00Z",
        posrednik="tozsamosc-posrednika",
        otwiera="posrednik",
    )
    room.run(
        "dopisz_uczestnika_prezentacji",
        prezentacja_id="prezentacja",
        osoba_id="zainteresowany",
        adres_podano="2026-11-06T12:00:00Z",
    )
    room.run("odnotuj_obecnosc_na_prezentacji", id=1, przyszedl=1)
    room.run(
        "odnotuj_przebieg_prezentacji",
        id="prezentacja",
        przebieg="Pytał o koszty ogrzewania i o miejsce postojowe.",
    )
    room.run("odnotuj_wynik_zapytania", id=1, skonczylo_sie_prezentacja=1)

    # The client lowers the price; the ad gets a new version with it.
    offer_price(
        room,
        "nowa-cena",
        kwota=64_500_000,
        obowiazuje_od="2026-11-08T09:00:00Z",
        poprzednia_id="cena",
        powod="dwie prezentacje bez oferty",
    )
    add_ad_version(
        room,
        "ogloszenie",
        2,
        cena=64_500_000,
        od="2026-11-08T10:00:00Z",
        wskazniki_energetyczne="EP 120 kWh/(m²·rok)",
    )
    room.run("odnotuj_cene_w_ogloszeniach", id="nowa-cena", w_ogloszeniach_od="2026-11-08")

    # The chain of offers: the buyer offers, the client counters, the buyer counters again and
    # the client accepts; every reply waits for the human's consent.
    offer_from_interested(room, "oferta-1", kwota=61_000_000)
    client_decision(room, "oferta-1", "kontroferta")
    room.run(
        "wpisz_kontroferte_klienta",
        id="kontroferta",
        dotyczy="sprzedaz",
        osoba_id="zainteresowany",
        odpowiada_na_id="oferta-1",
        wpisano="2026-11-10T11:00:00Z",
        kwota=64_000_000,
        wiaze_do="2026-11-13",
        zgoda_id="decyzja-oferta-1",
    )
    letter = reply_awaiting_consent(room, "oferta-1")
    send(room, letter, decyzja_id=f"wyslanie-{letter}", wyslano="2026-11-10T12:00:00Z")
    add_inflow(room, "odpowiedz-piotra", nadawca="Piotr <piotr@example.pl>")
    room.run(
        "odnotuj_odpowiedz_zainteresowanego",
        id="kontroferta",
        odpowiedz_drugiej_strony="kontroferta",
        odpowiedz_wplyw_id="odpowiedz-piotra",
    )
    offer_from_interested(
        room,
        "oferta-2",
        kwota=62_500_000,
        odpowiada_na_id="kontroferta",
        wpisano="2026-11-11T09:00:00Z",
    )
    client_decision(room, "oferta-2", "przyjeta")
    letter = reply_awaiting_consent(room, "oferta-2")
    send(room, letter, decyzja_id=f"wyslanie-{letter}", wyslano="2026-11-11T12:00:00Z")
    stage(room, "oferta_przyjeta", od="2026-11-11T12:30:00Z")

    # The accepted offer goes to a transaction case; the ad comes down.
    room.run(
        "zdejmij_ogloszenie",
        id="ogloszenie",
        zdjeto="2026-11-11T13:00:00Z",
        powod_zdjecia="przyjęta oferta kupna",
    )
    room.run(
        "wpisz_powiazanie",
        rodzaj="sprawa_zrodlowa",
        typ_teczki="transakcja",
        oznaczenie="Transakcja 2026/012",
        odnosnik="pokoj-transakcji-012",
        wpisal="tozsamosc-kierownika-posrednictwa",
        wpisano="2026-11-11T14:00:00Z",
    )


# The whole case ----------------------------------------------------------------------------------


@scenario(
    "Przebieg oferty: od przyjęcia przez umowę, cenę, ogłoszenie i prezentację do przyjętej oferty"
)
def whole_case():
    room = new_room()

    full_case(room)
    room.run("zakoncz_umowe", id="umowa", koniec_powod="transakcja doszła do skutku")
    stage(room, "koniec_oferty", od="2026-12-20T09:00:00Z")
    stage(room, "zamknieta", od="2026-12-21T09:00:00Z")

    etapy = [row["etap"] for row in room.run("etapy")]
    assert etapy == [
        "przyjeta",
        "weryfikacja_klienta",
        "umowa_posrednictwa",
        "przygotowanie_oferty",
        "oferta_aktywna",
        "oferta_przyjeta",
        "koniec_oferty",
        "zamknieta",
    ], etapy
    assert [row["id"] for row in room.run("ceny_ofertowe_biezace")] == ["nowa-cena"]
    assert room.run("biezaca_widocznosc")[0]["widocznosc"] == "publiczna"
    assert [row["numer"] for row in room.run("najnowsze_wersje_ogloszen")] == [2]
    assert [row["kierunek"] for row in room.run("oferty")] == [
        "od_zainteresowanego",
        "od_klienta",
        "od_zainteresowanego",
    ]
    assert room.run("oferty_do_przedstawienia") == []
    assert room.run("oferty")[2]["decyzja_klienta"] == "przyjeta"
    assert [row["zdjeto"] is not None for row in room.run("ogloszenia")] == [True]


# oferta-reguly -----------------------------------------------------------------------------------


@scenario("Reguły: teczka oferty ma typ pokoju oferta")
def room_type():
    room = new_room()
    room.refused("teczka oferty ma typ pokoju oferta", "zaloz_metryke", **case_record("najem"))

    room.run("zaloz_metryke", **case_record("oferta"))


def case_record(typ_pokoju):
    return {
        "oznaczenie": "Oferta 2026/041, ul. Polna 7 m. 12",
        "typ_pokoju": typ_pokoju,
        "przyjal": "tozsamosc-posrednika",
        "przyjeto": "2026-11-02T09:00:00Z",
    }


@scenario("Reguły: etapy oferty idą po kolei, a przyjęta oferta może wrócić do aktywnej")
def stage_transitions():
    room = new_room()
    offer_case(room)
    rule = "etap oferty przechodzi"

    room.refused(rule, "zmien_etap_sprawy", etap="umowa_posrednictwa", od="2026-11-04", zmienil="a")
    room.refused(rule, "zmien_etap_sprawy", etap="umowa", od="2026-11-04", zmienil="a")
    room.refused(rule, "zmien_etap_sprawy", etap="koniec_oferty", od="2026-11-04", zmienil="a")

    for etap in ("weryfikacja_klienta", "umowa_posrednictwa", "przygotowanie_oferty"):
        stage(room, etap)
    room.refused(rule, "zmien_etap_sprawy", etap="oferta_przyjeta", od="2026-11-05", zmienil="a")

    for etap in ("oferta_aktywna", "oferta_przyjeta", "oferta_aktywna", "koniec_oferty"):
        stage(room, etap)
    room.refused(rule, "zmien_etap_sprawy", etap="oferta_aktywna", od="2026-11-06", zmienil="a")
    stage(room, "zamknieta")


@scenario(
    "Reguły: przerwać wolno każdy etap przed zamknięciem; z przerwania nie ma skoku do innego etapu"
)
def stage_interrupted():
    room = new_room()
    offer_case(room)
    stage(room, "weryfikacja_klienta")
    approved_other(room, "przerwanie", None, None, "przerwanie_sprawy")

    stage(room, "przerwana", decyzja_id="przerwanie")
    room.refused(
        "etap oferty przechodzi",
        "zmien_etap_sprawy",
        etap="oferta_aktywna",
        od="2026-11-05",
        zmienil="a",
    )
    stage(room, "zamknieta")


@scenario(
    "Reguły: wznowiona oferta wraca do etapu sprzed przerwania, który dalej wymaga zawartej umowy"
)
def stage_resumed():
    room = new_room()
    offer_case(room)
    for etap in ("weryfikacja_klienta", "umowa_posrednictwa", "przygotowanie_oferty"):
        stage(room, etap)
    approved_decision(room, "przerwanie", "przerwanie_sprawy")
    stage(room, "przerwana", decyzja_id="przerwanie")
    approved_decision(room, "wznowienie", "wznowienie_sprawy", podpisano="2026-11-05T10:00:00Z")

    room.refused(
        "etap oferty przechodzi",
        "zmien_etap_sprawy",
        etap="oferta_aktywna",
        od="2026-11-05T11:00:00Z",
        zmienil="agent-oferty",
        decyzja_id="wznowienie",
    )
    stage(room, "przygotowanie_oferty", "2026-11-05T11:00:00Z", decyzja_id="wznowienie")

    approved_decision(room, "drugie-przerwanie", "przerwanie_sprawy")
    stage(room, "przerwana", "2026-11-06T09:00:00Z", decyzja_id="drugie-przerwanie")
    room.run("zakoncz_umowe", id="umowa", koniec_powod="wypowiedzenie przez klientkę")
    approved_decision(
        room, "drugie-wznowienie", "wznowienie_sprawy", podpisano="2026-11-06T10:00:00Z"
    )
    room.refused(
        "wymagają zawartej umowy pośrednictwa",
        "zmien_etap_sprawy",
        etap="przygotowanie_oferty",
        od="2026-11-06T11:00:00Z",
        zmienil="agent-oferty",
        decyzja_id="drugie-wznowienie",
    )
    stage(room, "zamknieta", "2026-11-06T12:00:00Z")


@scenario("Reguły: przygotowanie oferty i oferta aktywna wymagają zawartej umowy pośrednictwa")
def stage_needs_contract():
    room = new_room()
    open_case(room, typ_pokoju="oferta")
    client(room)
    add_flat(room)
    for etap in ("weryfikacja_klienta", "umowa_posrednictwa"):
        stage(room, etap)
    rule = "wymagają zawartej umowy pośrednictwa"

    room.refused(
        rule, "zmien_etap_sprawy", etap="przygotowanie_oferty", od="2026-11-04", zmienil="a"
    )

    brokerage_contract(room)
    stage(room, "przygotowanie_oferty")
    room.run("zakoncz_umowe", id="umowa", koniec_powod="wypowiedzenie przez klientkę")
    room.refused(rule, "zmien_etap_sprawy", etap="oferta_aktywna", od="2026-11-05", zmienil="a")


@scenario("Reguły: role w teczce oferty to klient, współwłaściciel i małżonek")
def roles():
    room = new_room()
    add_person(room, "osoba")

    room.refused(
        "rola w teczce oferty",
        "nadaj_role_uczestnikowi",
        uczestnik_id="osoba",
        rola="druga_strona",
    )
    room.run("nadaj_role_uczestnikowi", uczestnik_id="osoba", rola="wspolwlasciciel")


@scenario("Reguły: zgody w teczce oferty to zgody oferty i zgody rdzenia")
def consent_kinds():
    room = new_room()
    add_person(room, "klientka")
    add_received_document(room, "podpis", "oswiadczenie")

    room.refused(
        "zgoda w teczce oferty",
        "wpisz_zgode",
        **consent_fields("potwierdzenie_profilu"),
    )
    room.run("wpisz_zgode", **consent_fields("zatwierdzenie_opisu"))


def consent_fields(rodzaj):
    return {
        "id": f"zgoda-{rodzaj}",
        "uczestnik_id": "klientka",
        "rodzaj": rodzaj,
        "tresc": "Zatwierdzam opis.",
        "zlozono": "2026-11-04T13:00:00Z",
        "dokument_id": "podpis",
        "trwaly_nosnik": 0,
    }


@scenario("Reguły: umowa w teczce oferty to tylko umowa pośrednictwa")
def contract_kind():
    room = new_room()
    rule = "umowa w teczce oferty to umowa pośrednictwa"

    room.refused(
        rule, "przygotuj_umowe", id="zarzadzanie", rodzaj="zarzadzanie", obie_strony=0, wylacznosc=0
    )
    room.run("przygotuj_umowe", id="umowa", rodzaj="posrednictwo", obie_strony=0, wylacznosc=0)
    room.refused_sql(
        "rodzaj umowy się nie zmienia", "UPDATE sprawa_umowy_wpisy SET rodzaj = 'zarzadzanie'"
    )


@scenario("Reguły: teczka oferty nie ma rachunków ani wypłat")
def no_money():
    room = new_room()
    add_person(room, "zainteresowany")
    add_received_document(room, "pismo", "pismo")
    rule = "teczka oferty nie przyjmuje pieniędzy"

    room.refused(
        rule,
        "wpisz_rachunek",
        id="rachunek",
        uczestnik_id="zainteresowany",
        numer="PL61109010140000071219812874",
        posiadacz="Piotr",
        cel="zwrot_kaucji",
        zrodlo_dokument_id="pismo",
        wpisano="2026-11-03T08:00:00Z",
    )
    room.refused(
        rule,
        "przygotuj_wyplate",
        id="wyplata",
        podstawa_tabela="oferta_oferty_wpisy",
        podstawa_id="oferta",
        kwota=100,
        wyliczenie_sciezka="w.json",
        wyliczenie_skrot="sha256-w",
        odbiorca_uczestnik_id="zainteresowany",
        rachunek_id="rachunek",
        utworzono="2026-11-03T08:00:00Z",
    )


# oferta-widocznosc -------------------------------------------------------------------------------


@scenario("Widoczność: pierwsza wersja powstaje przy zawartej umowie i obowiązuje od zawarcia")
def visibility_first_version():
    room = new_room()
    open_case(room, typ_pokoju="oferta")
    client(room)
    add_flat(room)
    room.run("przygotuj_umowe", id="umowa", rodzaj="posrednictwo", obie_strony=0, wylacznosc=0)
    rule = "pierwsza wersja widoczności powstaje przy zawartej umowie"

    room.refused(rule, "wpisz_wersje_widocznosci", **visibility_fields())

    room.run("zakoncz_umowe", id="umowa", koniec_powod="klientka nie podpisała")
    brokerage_contract(room, contract_id="umowa-2")
    room.refused(
        rule, "wpisz_wersje_widocznosci", **visibility_fields(umowa_id="umowa-2", od="2026-11-05")
    )
    room.run("wpisz_wersje_widocznosci", **visibility_fields(umowa_id="umowa-2"))


def visibility_fields(**fields):
    return {
        "id": "widocznosc",
        "umowa_id": "umowa",
        "rodzaj_oferty": "sprzedaz",
        "widocznosc": "publiczna",
        "od": CONCLUDED,
        **fields,
    }


@scenario("Widoczność: oferta niepubliczna ma granice zgody, a zmianę wpisuje się z aneksu")
def visibility_changed_by_annex():
    room = new_room()
    offer_case(room)
    room.refused(
        "niepubliczna_ma_granice_zgody",
        "wpisz_wersje_widocznosci",
        **visibility_fields(widocznosc="niepubliczna"),
    )
    visibility(room)
    rule = "kolejna wersja widoczności wskazuje aneks"

    room.refused(
        rule, "wpisz_wersje_widocznosci", **visibility_fields(id="bez-aneksu", od="2026-11-20")
    )

    add_received_document(room, "aneks", "umowa_z_biurem")
    room.run(
        "wpisz_zdarzenie_umowy",
        id="aneks",
        umowa_id="umowa",
        rodzaj="aneks",
        chwila="2026-11-20T10:00:00Z",
        opis="oferta niepubliczna",
        dokument_id="aneks",
    )
    room.refused(
        rule,
        "wpisz_wersje_widocznosci",
        **visibility_fields(
            id="inna-chwila", aneks_zdarzenie_id="aneks", od="2026-11-21T10:00:00Z"
        ),
    )
    room.run(
        "wpisz_wersje_widocznosci",
        **visibility_fields(
            id="niepubliczna",
            widocznosc="niepubliczna",
            granice_zgody="tylko klientom z potwierdzonym finansowaniem, bez zdjęć wnętrz",
            aneks_zdarzenie_id="aneks",
            od="2026-11-20T10:00:00Z",
        ),
    )
    assert room.run("biezaca_widocznosc")[0]["id"] == "niepubliczna"

    room.refused_sql(
        "wersji widoczności się nie zmienia",
        "UPDATE oferta_widocznosc_wersje SET widocznosc = 'publiczna' WHERE id = 'niepubliczna'",
    )


# oferta-nieruchomosc -----------------------------------------------------------------------------


@scenario("Nieruchomość: świadectwo, które jest, ma dokument z ważnością i wskaźniki")
def property_certificate():
    room = new_room()
    add_flat(room)
    add_received_document(room, "pismo", "pismo")
    add_received_document(room, "bez-waznosci", "swiadectwo_energetyczne")
    add_received_document(
        room,
        "swiadectwo",
        "swiadectwo_energetyczne",
        data_wystawienia="2020-05-10",
        waznosc_do="2030-05-10",
        waznosc_zrodlo="przepis",
    )
    rule = "świadectwo wskazuje otrzymany dokument swiadectwo_energetyczne"

    room.refused(
        "swiadectwo_jest_z_dokumentem_i_wskaznikami",
        "wpisz_opis_nieruchomosci",
        nieruchomosc_id="lokal",
        swiadectwo="jest",
        swiadectwo_dokument_id="swiadectwo",
    )
    room.refused(rule, "wpisz_opis_nieruchomosci", **certificate_fields("pismo"))
    room.refused(rule, "wpisz_opis_nieruchomosci", **certificate_fields("bez-waznosci"))
    room.run("wpisz_opis_nieruchomosci", **certificate_fields("swiadectwo"))
    room.refused(rule, "zmien_opis_nieruchomosci", **certificate_fields("pismo"))

    room.refused(
        "dokument świadectwa z opisu nieruchomości",
        "odrzuc_dokument",
        id="swiadectwo",
        powod_odrzucenia="nieczytelny",
    )
    room.run("zmien_opis_nieruchomosci", nieruchomosc_id="lokal", swiadectwo="brak")
    room.refused_sql(
        "opis nie przechodzi na inną nieruchomość",
        "UPDATE oferta_nieruchomosc_opis SET nieruchomosc_id = 'inny-lokal'",
    )


def certificate_fields(document_id):
    return {
        "nieruchomosc_id": "lokal",
        "swiadectwo": "jest",
        "swiadectwo_dokument_id": document_id,
        "wskazniki_energetyczne": "EP 120 kWh/(m²·rok)",
    }


@scenario("Nieruchomość: rozbieżność z księgą ma opis, a przeglądania się nie zmienia")
def property_land_register_lookup():
    room = new_room()
    add_flat(room)
    lookup = {
        "id": "przegladanie",
        "nieruchomosc_id": "lokal",
        "numer_kw": "WR1K/00012345/6",
        "przegladano": "2026-11-03T09:00:00Z",
        "wlasciciele_dzial_ii": "Zofia Kowalska (spadkodawczyni)",
        "rozbieznosc": 1,
    }

    room.refused("rozbieznosc_ma_opis", "wpisz_przegladanie_ksiegi", **lookup)
    room.run(
        "wpisz_przegladanie_ksiegi",
        **lookup,
        rozbieznosc_opis="w księdze wpisana spadkodawczyni klientki",
    )
    room.refused_sql(
        "przeglądania księgi się nie zmienia",
        "UPDATE oferta_nieruchomosc_przegladania SET rozbieznosc = 0, rozbieznosc_opis = NULL",
    )


# oferta-ceny -------------------------------------------------------------------------------------


@scenario("Cena ofertowa ma zgodę zatwierdzenie_ceny klienta, która jej dotyczy")
def price_needs_client_consent():
    room = new_room()
    client(room)
    add_person(room, "maz", imiona="Jan", nazwisko="Kowalski")
    rule = "cena ofertowa ma zgodę zatwierdzenie_ceny klienta"
    price = {
        "id": "cena",
        "dotyczy": "sprzedaz",
        "kwota": 65_900_000,
        "obowiazuje_od": "2026-11-04T14:00:00Z",
    }

    client_consent(room, "zgoda-meza", "zatwierdzenie_ceny", "oferta_ceny_ofertowe", "cena", "maz")
    room.refused(rule, "wpisz_cene_ofertowa", **price, zgoda_id="zgoda-meza")
    client_consent(room, "zgoda-opisu", "zatwierdzenie_opisu", "oferta_ceny_ofertowe", "cena")
    room.refused(rule, "wpisz_cene_ofertowa", **price, zgoda_id="zgoda-opisu")
    client_consent(room, "zgoda-innej", "zatwierdzenie_ceny", "oferta_ceny_ofertowe", "inna-cena")
    room.refused(rule, "wpisz_cene_ofertowa", **price, zgoda_id="zgoda-innej")

    client_consent(room, "zgoda", "zatwierdzenie_ceny", "oferta_ceny_ofertowe", "cena")
    room.run("wpisz_cene_ofertowa", **price, zgoda_id="zgoda")


@scenario("Cena ofertowa: zmiana wskazuje obowiązującą cenę, ma powód i późniejszą chwilę")
def price_change_chain():
    room = new_room()
    client(room)
    offer_price(room)
    for price_id in (
        "bez-poprzedniej",
        "bez-powodu",
        "wczesniejsza",
        "czynsz",
        "nowa",
        "od-starej",
    ):
        client_consent(
            room, f"zgoda-{price_id}", "zatwierdzenie_ceny", "oferta_ceny_ofertowe", price_id
        )
    rule = "zmiana ceny wskazuje obowiązującą cenę"

    room.refused(rule, "wpisz_cene_ofertowa", **price_change("bez-poprzedniej", poprzednia_id=None))
    room.refused(
        "zmiana_ceny_ma_powod", "wpisz_cene_ofertowa", **price_change("bez-powodu", powod=None)
    )
    room.refused(
        rule,
        "wpisz_cene_ofertowa",
        **price_change("wczesniejsza", obowiazuje_od="2026-11-04T13:00:00Z"),
    )
    room.refused(rule, "wpisz_cene_ofertowa", **price_change("czynsz", dotyczy="najem"))

    room.run("wpisz_cene_ofertowa", **price_change("nowa"))
    room.refused(
        rule, "wpisz_cene_ofertowa", **price_change("od-starej", obowiazuje_od="2026-11-09")
    )
    assert [row["id"] for row in room.run("ceny_ofertowe_biezace")] == ["nowa"]


def price_change(price_id, **fields):
    return {
        "id": price_id,
        "dotyczy": "sprzedaz",
        "kwota": 64_500_000,
        "zgoda_id": f"zgoda-{price_id}",
        "obowiazuje_od": "2026-11-08T09:00:00Z",
        "poprzednia_id": "cena",
        "powod": "dwie prezentacje bez oferty",
        **fields,
    }


@scenario("Cena ofertowa: wejście do ogłoszeń wpisuje się raz, a reszty się nie zmienia")
def price_in_ads_once():
    room = new_room()
    client(room)
    offer_price(room)

    room.refused(
        "w_ogloszeniach_nie_przed_cena",
        "odnotuj_cene_w_ogloszeniach",
        id="cena",
        w_ogloszeniach_od="2026-11-01",
    )
    room.run("odnotuj_cene_w_ogloszeniach", id="cena", w_ogloszeniach_od="2026-11-05")
    rule = "ceny ofertowej się nie zmienia"
    room.refused(rule, "odnotuj_cene_w_ogloszeniach", id="cena", w_ogloszeniach_od="2026-11-06")
    client_consent(room, "zgoda-od-razu", "zatwierdzenie_ceny", "oferta_ceny_ofertowe", "od-razu")
    room.refused_sql(
        "cena ofertowa powstaje bez daty wejścia do ogłoszeń",
        "INSERT INTO oferta_ceny_ofertowe (id, dotyczy, kwota, zgoda_id, obowiazuje_od, poprzednia_id, "
        "powod, w_ogloszeniach_od) VALUES ('od-razu', 'sprzedaz', 1, 'zgoda-od-razu', "
        "'2026-11-09T09:00:00Z', 'cena', 'obniżka', '2026-11-09')",
    )
    room.refused_sql(rule, "UPDATE oferta_ceny_ofertowe SET kwota = 1")


@scenario("Granice negocjacji mają zgodę klienta i zastępują tylko obowiązujące granice")
def negotiation_limits():
    room = new_room()
    client(room)
    limits = {"dotyczy": "sprzedaz", "najnizsza_kwota": 63_000_000}

    client_consent(room, "zgoda-ceny", "zatwierdzenie_ceny", "oferta_ceny_granice", "granice")
    room.refused(
        "granice negocjacji mają zgodę granice_negocjacji klienta",
        "wpisz_granice_negocjacji",
        id="granice",
        zgoda_id="zgoda-ceny",
        **limits,
    )
    for limits_id in ("granice", "nowe", "inne"):
        client_consent(
            room, f"zgoda-{limits_id}", "granice_negocjacji", "oferta_ceny_granice", limits_id
        )
    room.run("wpisz_granice_negocjacji", id="granice", zgoda_id="zgoda-granice", **limits)

    rule = "nowe granice negocjacji zastępują obowiązujące"
    room.refused(rule, "wpisz_granice_negocjacji", id="nowe", zgoda_id="zgoda-nowe", **limits)
    room.run(
        "wpisz_granice_negocjacji",
        id="nowe",
        zgoda_id="zgoda-nowe",
        zastepuje_id="granice",
        **limits,
    )
    room.refused(
        rule,
        "wpisz_granice_negocjacji",
        id="inne",
        zgoda_id="zgoda-inne",
        zastepuje_id="granice",
        **limits,
    )
    assert [row["id"] for row in room.run("granice_negocjacji_biezace")] == ["nowe"]
    room.refused_sql(
        "granic negocjacji się nie zmienia", "UPDATE oferta_ceny_granice SET warunki = 'x'"
    )


@scenario("Propozycji ceny i porównań transakcji się nie zmienia")
def price_proposals_fixed():
    room = new_room()
    room.run(
        "wpisz_propozycje_ceny",
        id="propozycja",
        dotyczy="sprzedaz",
        kwota=64_000_000,
        uzasadnienie="pięć transakcji",
        utworzono="2026-11-04T13:40:00Z",
        wyliczenie_sciezka="ceny/propozycja.json",
        wyliczenie_skrot="sha256-propozycja",
    )
    room.run(
        "dopisz_porownanie_transakcji",
        propozycja_id="propozycja",
        data_transakcji="2026-03-14",
        cena=61_500_000,
        zrodlo="rejestr cen nieruchomości",
    )

    room.refused_sql(
        "propozycji ceny się nie zmienia", "UPDATE oferta_ceny_propozycje SET kwota = 1"
    )
    room.refused_sql(
        "porównania transakcji się nie zmienia", "UPDATE oferta_ceny_porownania SET cena = 1"
    )


# oferta-najem ------------------------------------------------------------------------------------


@scenario("Warunki najmu dopuszczają rodzaj najmu, mają zgodę klienta i zastępują obowiązujące")
def rental_terms():
    room = new_room()
    client(room)
    for terms_id in ("warunki", "nowe", "inne"):
        client_consent(room, f"zgoda-{terms_id}", "warunki_najmu", "oferta_najem_warunki", terms_id)
    client_consent(room, "zgoda-ceny", "zatwierdzenie_ceny", "oferta_najem_warunki", "warunki")

    room.refused(
        "dopuszcza_rodzaj_najmu",
        "wpisz_warunki_najmu",
        **rental_terms_fields("warunki", dopuszcza_zwykly=0),
    )
    room.refused(
        "warunki najmu mają zgodę warunki_najmu klienta",
        "wpisz_warunki_najmu",
        **rental_terms_fields("warunki", zgoda_id="zgoda-ceny"),
    )
    room.run("wpisz_warunki_najmu", **rental_terms_fields("warunki"))

    rule = "nowe warunki najmu zastępują warunki, które obowiązują"
    room.refused(rule, "wpisz_warunki_najmu", **rental_terms_fields("nowe"))
    room.run("wpisz_warunki_najmu", **rental_terms_fields("nowe", zastepuje_id="warunki"))
    room.refused(rule, "wpisz_warunki_najmu", **rental_terms_fields("inne", zastepuje_id="warunki"))
    assert room.run("warunki_najmu_biezace")[0]["id"] == "nowe"
    room.refused_sql("warunków najmu się nie zmienia", "UPDATE oferta_najem_warunki SET kaucja = 1")


def rental_terms_fields(terms_id, **fields):
    return {
        "id": terms_id,
        "kaucja": 600_000,
        "dopuszcza_zwykly": 1,
        "dopuszcza_okazjonalny": 0,
        "dopuszcza_instytucjonalny": 0,
        "wolny_od": "2026-12-01",
        "okres_min": "12 miesięcy",
        "prowadzenie_najmu": 1,
        "zgoda_id": f"zgoda-{terms_id}",
        **fields,
    }


# oferta-zainteresowani ---------------------------------------------------------------------------


@scenario("Zainteresowany ma kontakt, a działający przez biuro wskazuje biuro")
def interested_contact():
    room = new_room()

    room.refused("osoba_ma_kontakt", "wpisz_zainteresowanego", **interested_fields(telefon=None))
    room.refused(
        "przez_biuro_wskazuje_biuro",
        "wpisz_zainteresowanego",
        **interested_fields(dziala_jako="przez_biuro"),
    )
    room.run(
        "wpisz_zainteresowanego",
        **interested_fields(
            dziala_jako="przez_biuro", biuro="Nieruchomości Odra", dla_kogo="kupujący"
        ),
    )
    room.run("zmien_kontakt_zainteresowanego", id="zainteresowany", email="piotr@example.pl")


def interested_fields(**fields):
    return {
        "id": "zainteresowany",
        "imie": "Piotr",
        "telefon": "+48 600 100 200",
        "dziala_jako": "sam",
        "dane_pozyskano": "2026-11-06T09:00:00Z",
        **fields,
    }


@scenario("Zainteresowany: informacja o danych to wysłana wysyłka RODO albo data od Recepcji, raz")
def interested_privacy_notice():
    room = new_room()
    interested_person(room)
    rule = "informacja o danych to wysłana wysyłka ze znacznikiem informacja_rodo"

    prepare_letter(room, "bez-rodo", adresat_rodzaj="inny", adresat_nazwa="Piotr")
    send(room, "bez-rodo")
    room.refused(
        rule,
        "odnotuj_informacje_dla_zainteresowanego",
        id="zainteresowany",
        informacja_wysylka_id="bez-rodo",
    )
    prepare_letter(
        room, "niewyslana", adresat_rodzaj="inny", adresat_nazwa="Piotr", informacja_rodo=1
    )
    room.refused(
        rule,
        "odnotuj_informacje_dla_zainteresowanego",
        id="zainteresowany",
        informacja_wysylka_id="niewyslana",
    )

    privacy_notice(room, "informacja")
    room.refused(
        "informacja_wysylka_albo_od_recepcji",
        "odnotuj_informacje_dla_zainteresowanego",
        id="zainteresowany",
        informacja_wysylka_id="informacja",
        informacja_od_recepcji="2026-11-06",
    )
    room.run(
        "odnotuj_informacje_dla_zainteresowanego",
        id="zainteresowany",
        informacja_wysylka_id="informacja",
    )
    room.refused(
        "informację o danych zainteresowanego wpisuje się raz",
        "odnotuj_informacje_dla_zainteresowanego",
        id="zainteresowany",
        informacja_od_recepcji="2026-11-06",
    )
    room.run(
        "wpisz_zainteresowanego",
        **interested_fields(id="od-recepcji", informacja_od_recepcji="2026-11-06"),
    )


@scenario("Zapytanie: odpowiedź przychodzi po informacji o danych, a odpowiedź i wynik raz")
def inquiry_answer():
    room = new_room()
    interested_person(room)
    add_inflow(room, "zapytanie", nadawca="Piotr", kanal="portal")
    room.run("wpisz_zapytanie", osoba_id="zainteresowany", wplyw_id="zapytanie")
    privacy_notice(room, "odpowiedz")

    room.refused(
        "odpowiedź na zapytanie wymaga informacji o danych zainteresowanego",
        "odnotuj_odpowiedz_na_zapytanie",
        id=1,
        odpowiedz_wysylka_id="odpowiedz",
    )
    room.run(
        "odnotuj_informacje_dla_zainteresowanego",
        id="zainteresowany",
        informacja_wysylka_id="odpowiedz",
    )
    room.run("odnotuj_odpowiedz_na_zapytanie", id=1, odpowiedz_wysylka_id="odpowiedz")
    room.run("odnotuj_wynik_zapytania", id=1, skonczylo_sie_prezentacja=0)

    rule = "zapytania się nie zmienia"
    room.refused(rule, "odnotuj_wynik_zapytania", id=1, skonczylo_sie_prezentacja=1)
    room.refused_sql(rule, "UPDATE oferta_zainteresowani_zapytania SET wplyw_id = 'inny'")
    room.refused_sql(
        "zapytanie powstaje bez odpowiedzi i wyniku",
        "INSERT INTO oferta_zainteresowani_zapytania (osoba_id, wplyw_id, skonczylo_sie_prezentacja) "
        "VALUES ('zainteresowany', 'zapytanie', 1)",
    )


@scenario("Prezentacja: przy zawartej umowie, nie wcześniej niż praca od (art. 15 ust. 3 upk)")
def viewing_after_work_starts():
    room = new_room()
    open_case(room, typ_pokoju="oferta")
    client(room)
    add_flat(room)
    rule = "prezentacja wymaga zawartej umowy pośrednictwa i odbywa się nie wcześniej niż praca od"

    room.refused(rule, "umow_prezentacje", **viewing_fields("2026-11-07T17:00:00Z"))

    # A consumer's contract off premises: work starts the day after the withdrawal period.
    brokerage_contract(room, sposob_zawarcia="na_odleglosc", odstapienie_do=WITHDRAWAL_ENDS)
    room.refused(rule, "umow_prezentacje", **viewing_fields("2026-11-18T17:00:00Z"))
    room.run("umow_prezentacje", **viewing_fields("2026-11-19T17:00:00Z"))


def viewing_fields(termin, viewing_id=None):
    return {
        "id": viewing_id or f"prezentacja-{termin}",
        "termin": termin,
        "posrednik": "tozsamosc-posrednika",
        "otwiera": "klient",
    }


@scenario("Prezentacja: przebieg albo odwołanie raz, termin się nie zmienia, uczestnik raz")
def viewing_record():
    room = new_room()
    offer_case(room)
    interested_person(room)
    room.run("umow_prezentacje", **viewing_fields("2026-11-07T17:00:00Z", "prezentacja"))
    room.run(
        "dopisz_uczestnika_prezentacji", prezentacja_id="prezentacja", osoba_id="zainteresowany"
    )

    room.refused(
        REPLACEMENT,
        "dopisz_uczestnika_prezentacji",
        prezentacja_id="prezentacja",
        osoba_id="zainteresowany",
    )
    room.run("odnotuj_podanie_adresu", id=1, adres_podano="2026-11-06T12:00:00Z")
    room.run("odnotuj_obecnosc_na_prezentacji", id=1, przyszedl=0)
    room.refused(
        "uczestnika prezentacji się nie zmienia",
        "odnotuj_obecnosc_na_prezentacji",
        id=1,
        przyszedl=1,
    )

    rule = "prezentacji się nie zmienia"
    room.refused_sql(
        rule, "UPDATE oferta_zainteresowani_prezentacje SET termin = '2026-11-08T17:00:00Z'"
    )
    room.refused(
        "odwolanie_kto_kiedy_dlaczego",
        "odwolaj_prezentacje",
        id="prezentacja",
        odwolano="2026-11-07T12:00:00Z",
        odwolal="klientka",
    )
    room.run("odnotuj_przebieg_prezentacji", id="prezentacja", przebieg="Nikt nie przyszedł.")
    room.refused(
        rule,
        "odwolaj_prezentacje",
        id="prezentacja",
        odwolano="2026-11-07T12:00:00Z",
        odwolal="klientka",
        powod_odwolania="choroba",
    )
    room.run("umow_prezentacje", **viewing_fields("2026-11-08T17:00:00Z", "druga"))
    room.run(
        "odwolaj_prezentacje",
        id="druga",
        odwolano="2026-11-07T12:00:00Z",
        odwolal="klientka",
        powod_odwolania="choroba",
    )
    room.refused(rule, "odnotuj_przebieg_prezentacji", id="druga", przebieg="nikt nie przyszedł")
    room.refused_sql(
        "prezentacja powstaje umówiona",
        "INSERT INTO oferta_zainteresowani_prezentacje (id, termin, posrednik, otwiera, przebieg) "
        "VALUES ('z-przebiegiem', '2026-11-07T17:00:00Z', 'p', 'klient', 'x')",
    )


# oferta-ogloszenia -------------------------------------------------------------------------------


@scenario("Ogłoszenie powstaje jako projekt i wychodzi tylko z wersją treści sprzed publikacji")
def ad_starts_as_draft():
    room = new_room()
    ready_to_publish(room)

    room.refused_sql(
        "ogłoszenie powstaje jako projekt",
        "INSERT INTO oferta_ogloszenia_wpisy (id, dotyczy, kanal, opublikowano) "
        "VALUES ('od-razu', 'sprzedaz', 'strona_biura', '2026-11-05T10:00:00Z')",
    )
    room.refused(
        "portal_przy_kanale_portal",
        "przygotuj_ogloszenie",
        id="bez-portalu",
        dotyczy="sprzedaz",
        kanal="portal",
    )
    room.refused(
        "publikacja wymaga wersji treści sprzed publikacji",
        "odnotuj_publikacje_ogloszenia",
        id="ogloszenie",
        opublikowano="2026-11-05T08:00:00Z",
    )
    publish(room)


@scenario("Ogłoszenie: zawarta umowa z czynnością publikacja_ogloszen (art. 180 ust. 3 ugn)")
def ad_within_contract_scope():
    room = new_room()
    offer_case(room, czynnosci=("prezentacje",))
    visibility(room)
    offer_price(room)
    draft_ad(room)
    approved_other(room, "pierwsza-publikacja", "oferta_ogloszenia_wpisy", "ogloszenie")
    rule = (
        "ogłoszenie wymaga zawartej umowy pośrednictwa, której zakres obejmuje publikacja_ogloszen"
    )

    room.refused(
        rule, "odnotuj_publikacje_ogloszenia", id="ogloszenie", opublikowano="2026-11-05T10:00:00Z"
    )

    room.run("zakoncz_umowe", id="umowa", koniec_powod="aneks rozszerza zakres w nowej umowie")
    brokerage_contract(room, contract_id="umowa-2")
    publish(room)


@scenario("Ogłoszenie wychodzi nie wcześniej niż praca od według umowy (art. 15 ust. 3 upk)")
def ad_after_work_starts():
    room = new_room()
    offer_case(room, sposob_zawarcia="na_odleglosc", odstapienie_do=WITHDRAWAL_ENDS)
    visibility(room)
    offer_price(room)
    draft_ad(room)
    approved_other(room, "pierwsza-publikacja", "oferta_ogloszenia_wpisy", "ogloszenie")
    rule = "ogłoszenie wychodzi nie wcześniej niż praca od"

    room.refused(
        rule, "odnotuj_publikacje_ogloszenia", id="ogloszenie", opublikowano="2026-11-18T10:00:00Z"
    )
    publish(room, opublikowano="2026-11-19T08:00:00Z")
    room.refused(
        rule, "dopisz_wersje_ogloszenia", **ad_version_fields(2, od="2026-11-18T23:00:00Z")
    )


def ad_version_fields(numer, cena=65_900_000, od="2026-11-20T09:00:00Z", **fields):
    return {
        "ogloszenie_id": "ogloszenie",
        "numer": numer,
        "od": od,
        "tytul": "2 pokoje, 48 m², Krzyki",
        "cena": cena,
        "lokalizacja": "Wrocław, Krzyki",
        "tresc_sciezka": f"ogloszenia/{numer}.json",
        "tresc_skrot": f"sha256-{numer}",
        **fields,
    }


@scenario("Ogłoszenie: tylko oferta publiczna, której rodzaj obejmuje ogłoszenie")
def ad_only_public_offer():
    room = new_room()
    offer_case(room)
    visibility(
        room,
        widocznosc="niepubliczna",
        granice_zgody="tylko klientom biura z potwierdzonym finansowaniem",
    )
    offer_price(room)
    offer_price(room, "czynsz", kwota=320_000, dotyczy="najem")
    draft_ad(room)
    draft_ad(room, "najem", cena=320_000, dotyczy="najem")
    approved_other(room, "pierwsza-publikacja", "oferta_ogloszenia_wpisy", "ogloszenie")
    rule = "ogłasza się tylko ofertę publiczną"

    room.refused(
        rule, "odnotuj_publikacje_ogloszenia", id="ogloszenie", opublikowano="2026-11-05T10:00:00Z"
    )

    add_received_document(room, "aneks", "umowa_z_biurem")
    room.run(
        "wpisz_zdarzenie_umowy",
        id="aneks",
        umowa_id="umowa",
        rodzaj="aneks",
        chwila="2026-11-05T09:30:00Z",
        dokument_id="aneks",
    )
    visibility(room, "publiczna", aneks_zdarzenie_id="aneks", od="2026-11-05T09:30:00Z")
    publish(room)
    room.refused(
        rule, "odnotuj_publikacje_ogloszenia", id="najem", opublikowano="2026-11-05T10:00:00Z"
    )


@scenario("Ogłoszenie: cena w wersji to obowiązująca cena ofertowa, przy publikacji i nowej wersji")
def ad_price_is_offer_price():
    room = new_room()
    offer_case(room)
    visibility(room)
    offer_price(room)
    draft_ad(room, cena=60_000_000)
    approved_other(room, "pierwsza-publikacja", "oferta_ogloszenia_wpisy", "ogloszenie")
    rule = "cena w ogłoszeniu to obowiązująca cena ofertowa"

    room.refused(
        rule, "odnotuj_publikacje_ogloszenia", id="ogloszenie", opublikowano="2026-11-05T10:00:00Z"
    )
    add_ad_version(room, "ogloszenie", 2, od="2026-11-05T09:30:00Z")
    publish(room)

    offer_price(
        room,
        "nowa",
        kwota=64_500_000,
        obowiazuje_od="2026-11-08T09:00:00Z",
        poprzednia_id="cena",
        powod="brak zapytań",
    )
    room.refused(rule, "dopisz_wersje_ogloszenia", **ad_version_fields(3))
    room.run("dopisz_wersje_ogloszenia", **ad_version_fields(3, cena=64_500_000))
    room.refused(
        "wersje ogłoszenia mają kolejne numery",
        "dopisz_wersje_ogloszenia",
        **ad_version_fields(5, cena=64_500_000),
    )
    room.refused(
        "wersje ogłoszenia mają kolejne numery",
        "dopisz_wersje_ogloszenia",
        **ad_version_fields(4, cena=64_500_000, od="2026-11-19T09:00:00Z"),
    )
    room.refused_sql(
        "wersji ogłoszenia się nie zmienia", "UPDATE oferta_ogloszenia_wersje SET cena = 1"
    )


@scenario("Ogłoszenie: przy świadectwie ma wskaźniki, a świadectwo jest ważne w dniu publikacji")
def ad_energy_certificate():
    room = new_room()
    ready_to_publish(room)
    add_received_document(
        room,
        "swiadectwo",
        "swiadectwo_energetyczne",
        data_wystawienia="2016-11-01",
        waznosc_do="2026-11-30",
        waznosc_zrodlo="przepis",
    )
    room.run("wpisz_opis_nieruchomosci", **certificate_fields("swiadectwo"))
    rule = "przy świadectwie energetycznym ogłoszenie ma jego wskaźniki"

    room.refused(
        rule, "odnotuj_publikacje_ogloszenia", id="ogloszenie", opublikowano="2026-11-05T10:00:00Z"
    )
    add_ad_version(room, "ogloszenie", 2, wskazniki_energetyczne="EP 120 kWh/(m²·rok)")
    publish(room)

    room.refused(
        rule,
        "dopisz_wersje_ogloszenia",
        **ad_version_fields(3, od="2026-12-01T09:00:00Z", wskazniki_energetyczne="EP 120"),
    )
    room.run(
        "dopisz_wersje_ogloszenia",
        **ad_version_fields(3, od="2026-11-30T09:00:00Z", wskazniki_energetyczne="EP 120"),
    )


@scenario("Ogłoszenie: rozbieżność z księgą wymaga decyzji, która wskazuje ostatnie przeglądanie")
def ad_land_register_discrepancy():
    room = new_room()
    ready_to_publish(room)
    lookup = {
        "nieruchomosc_id": "lokal",
        "numer_kw": "WR1K/00012345/6",
        "wlasciciele_dzial_ii": "Zofia Kowalska",
        "rozbieznosc": 1,
        "rozbieznosc_opis": "w księdze wpisana spadkodawczyni klientki",
    }
    room.run("wpisz_przegladanie_ksiegi", id="stare", przegladano="2026-11-01T09:00:00Z", **lookup)
    room.run(
        "wpisz_przegladanie_ksiegi", id="przegladanie", przegladano="2026-11-03T09:00:00Z", **lookup
    )
    approved_other(room, "zgoda-na-stare", "oferta_nieruchomosc_przegladania", "stare")
    rule = "przy rozbieżności z księgą ogłoszenie wymaga zatwierdzonej decyzji inna"

    room.refused(
        rule, "odnotuj_publikacje_ogloszenia", id="ogloszenie", opublikowano="2026-11-05T10:00:00Z"
    )
    approved_other(
        room, "zgoda-mimo-rozbieznosci", "oferta_nieruchomosc_przegladania", "przegladanie"
    )
    publish(room)

    room.run(
        "wpisz_przegladanie_ksiegi",
        id="nowe",
        przegladano="2026-11-10T09:00:00Z",
        **{**lookup, "wlasciciele_dzial_ii": "Zofia Kowalska, Jan Nowak"},
    )
    room.refused(rule, "dopisz_wersje_ogloszenia", **ad_version_fields(2))


@scenario(
    "Ogłoszenie: pierwszą publikację w teczce zatwierdza pośrednik decyzją, która ją wskazuje"
)
def ad_first_publication_approved():
    room = new_room()
    offer_case(room)
    visibility(room)
    offer_price(room)
    draft_ad(room)
    draft_ad(room, "drugie")
    approved_other(room, "zgoda-na-drugie", "oferta_ogloszenia_wpisy", "drugie")
    rule = "pierwsza publikacja w teczce wymaga zatwierdzonej decyzji inna"

    room.refused(
        rule, "odnotuj_publikacje_ogloszenia", id="ogloszenie", opublikowano="2026-11-05T10:00:00Z"
    )
    publish(room, "drugie")
    publish(room, "ogloszenie")


@scenario("Ogłoszenie: zdjęte ma powód i nie wraca; numer i statystyki po publikacji")
def ad_taken_down():
    room = new_room()
    ready_to_publish(room)

    room.refused(
        "numer_po_publikacji", "odnotuj_numer_ogloszenia", id="ogloszenie", numer_ogloszenia="OT-1"
    )
    room.refused(
        "statystyki odczytuje się dla ogłoszenia po jego publikacji",
        "wpisz_statystyki_ogloszenia",
        ogloszenie_id="ogloszenie",
        odczytano="2026-11-06T08:00:00Z",
        wyswietlenia=0,
        zapytania=0,
    )
    publish(room)
    room.run("odnotuj_numer_ogloszenia", id="ogloszenie", numer_ogloszenia="OT-1")
    room.run(
        "wpisz_statystyki_ogloszenia",
        ogloszenie_id="ogloszenie",
        odczytano="2026-11-06T08:00:00Z",
        wyswietlenia=120,
        zapytania=2,
    )
    room.refused_sql(
        "odczytu statystyk się nie zmienia", "UPDATE oferta_ogloszenia_statystyki SET zapytania = 9"
    )

    rule = "ogłoszenia się nie zmienia"
    room.refused(rule, "odnotuj_numer_ogloszenia", id="ogloszenie", numer_ogloszenia="OT-2")
    room.refused_sql(
        "zdjete_ma_powod",
        "UPDATE oferta_ogloszenia_wpisy SET zdjeto = '2026-11-11T13:00:00Z' WHERE id = 'ogloszenie'",
    )
    room.run(
        "zdejmij_ogloszenie",
        id="ogloszenie",
        zdjeto="2026-11-11T13:00:00Z",
        powod_zdjecia="koniec umowy pośrednictwa",
    )
    room.refused(
        "zdjęte ogłoszenie nie dostaje nowych wersji",
        "dopisz_wersje_ogloszenia",
        **ad_version_fields(2),
    )
    room.refused_sql(rule, "UPDATE oferta_ogloszenia_wpisy SET zdjeto = NULL, powod_zdjecia = NULL")


# oferta-oferty -----------------------------------------------------------------------------------


def offers_ready(room, **contract_fields):
    """An offer case with an interested person informed about his data."""
    offer_case(room, **contract_fields)
    interested_person(room)


@scenario(
    "Oferta zainteresowanego: z wpływu albo notatki, z terminem związania albo sposobem złożenia"
)
def offer_source_and_binding():
    room = new_room()
    offers_ready(room)
    add_inflow(room, "wplyw", nadawca="Piotr")
    add_received_document(room, "notatka", "pismo")
    offer = {
        "id": "oferta",
        "dotyczy": "sprzedaz",
        "osoba_id": "zainteresowany",
        "wpisano": "2026-11-10T09:00:00Z",
        "kwota": 61_000_000,
    }

    room.refused(
        "oferta_ma_termin_albo_sposob_zlozenia",
        "wpisz_oferte_zainteresowanego",
        **offer,
        wplyw_id="wplyw",
    )
    room.refused(
        "oferta_zainteresowanego_z_wplywu_albo_notatki",
        "wpisz_oferte_zainteresowanego",
        **offer,
        wplyw_id="wplyw",
        dokument_id="notatka",
        sposob_zlozenia="w_rozmowie",
    )
    room.run(
        "wpisz_oferte_zainteresowanego",
        **offer,
        dokument_id="notatka",
        sposob_zlozenia="w_rozmowie",
    )
    room.refused_sql(
        "oferta powstaje bez przedstawienia, decyzji i odpowiedzi",
        "INSERT INTO oferta_oferty_wpisy (id, dotyczy, kierunek, osoba_id, wplyw_id, wpisano, kwota, "
        "sposob_zlozenia, przedstawiono) VALUES ('od-razu', 'sprzedaz', 'od_zainteresowanego', "
        "'zainteresowany', 'wplyw', '2026-11-10', 1, 'na_pismie', '2026-11-10')",
    )
    room.refused_sql(
        "oferty się nie zmienia",
        "UPDATE oferta_oferty_wpisy SET kwota = 62000000 WHERE id = 'oferta'",
    )


@scenario("Decyzja klienta o ofercie: po przedstawieniu, z jego zgodą odpowiedz_na_oferte, raz")
def offer_client_decision():
    room = new_room()
    offers_ready(room)
    add_person(room, "maz", imiona="Jan", nazwisko="Kowalski")
    offer_from_interested(room, "oferta")
    rule = "decyzja klienta o ofercie wskazuje jego zgodę odpowiedz_na_oferte"

    client_consent(room, "decyzja-oferta", "odpowiedz_na_oferte", "oferta_oferty_wpisy", "oferta")
    room.refused(
        "decyzja_po_przedstawieniu",
        "odnotuj_decyzje_klienta_o_ofercie",
        id="oferta",
        decyzja_klienta="przyjeta",
        zgoda_id="decyzja-oferta",
    )
    room.run("odnotuj_przedstawienie_oferty", id="oferta", przedstawiono="2026-11-10T10:00:00Z")
    client_consent(
        room, "zgoda-meza", "odpowiedz_na_oferte", "oferta_oferty_wpisy", "oferta", "maz"
    )
    room.refused(
        rule,
        "odnotuj_decyzje_klienta_o_ofercie",
        id="oferta",
        decyzja_klienta="przyjeta",
        zgoda_id="zgoda-meza",
    )
    client_consent(room, "zgoda-ceny", "zatwierdzenie_ceny", "oferta_oferty_wpisy", "oferta")
    room.refused(
        rule,
        "odnotuj_decyzje_klienta_o_ofercie",
        id="oferta",
        decyzja_klienta="przyjeta",
        zgoda_id="zgoda-ceny",
    )
    room.run(
        "odnotuj_decyzje_klienta_o_ofercie",
        id="oferta",
        decyzja_klienta="odrzucona",
        zgoda_id="decyzja-oferta",
    )

    rule = "oferty się nie zmienia"
    room.refused(
        rule,
        "odnotuj_decyzje_klienta_o_ofercie",
        id="oferta",
        decyzja_klienta="przyjeta",
        zgoda_id="decyzja-oferta",
    )
    room.refused(
        rule, "odnotuj_przedstawienie_oferty", id="oferta", przedstawiono="2026-11-10T11:00:00Z"
    )


@scenario("Kontroferty: klient odpowiada na decyzję kontroferta, zainteresowany na swoją odpowiedź")
def offer_counter_chain():
    room = new_room()
    offers_ready(room)
    offer_from_interested(room, "oferta-1")
    offer_from_interested(room, "oferta-bez-decyzji")
    client_decision(room, "oferta-1", "kontroferta")
    counter = {
        "dotyczy": "sprzedaz",
        "osoba_id": "zainteresowany",
        "wpisano": "2026-11-10T11:00:00Z",
        "kwota": 64_000_000,
        "wiaze_do": "2026-11-13",
        "zgoda_id": "decyzja-oferta-1",
    }
    rule = "kontroferta klienta odpowiada na ofertę zainteresowanego z decyzją kontroferta"

    room.refused(
        rule, "wpisz_kontroferte_klienta", id="k", odpowiada_na_id="oferta-bez-decyzji", **counter
    )
    room.refused(
        rule,
        "wpisz_kontroferte_klienta",
        id="k",
        odpowiada_na_id="oferta-1",
        **{**counter, "dotyczy": "najem"},
    )
    room.run("wpisz_kontroferte_klienta", id="kontroferta", odpowiada_na_id="oferta-1", **counter)

    rule = "kontroferta zainteresowanego odpowiada na kontrofertę klienta"
    room.refused(rule, "wpisz_oferte_zainteresowanego", **counter_from_interested("kontroferta"))
    add_inflow(room, "odpowiedz-piotra", nadawca="Piotr")
    room.refused(
        "odpowiedz_drugiej_strony_na_kontroferte",
        "odnotuj_odpowiedz_zainteresowanego",
        id="oferta-1",
        odpowiedz_drugiej_strony="kontroferta",
        odpowiedz_wplyw_id="odpowiedz-piotra",
    )
    room.run(
        "odnotuj_odpowiedz_zainteresowanego",
        id="kontroferta",
        odpowiedz_drugiej_strony="kontroferta",
        odpowiedz_wplyw_id="odpowiedz-piotra",
    )
    room.refused(rule, "wpisz_oferte_zainteresowanego", **counter_from_interested("oferta-1"))
    room.run("wpisz_oferte_zainteresowanego", **counter_from_interested("kontroferta"))


def counter_from_interested(odpowiada_na_id):
    return {
        "id": "oferta-2",
        "dotyczy": "sprzedaz",
        "osoba_id": "zainteresowany",
        "odpowiada_na_id": odpowiada_na_id,
        "wplyw_id": "odpowiedz-piotra",
        "wpisano": "2026-11-11T09:00:00Z",
        "kwota": 62_500_000,
        "sposob_zlozenia": "na_pismie",
    }


@scenario("Odpowiedź na ofertę: niewysłana wysyłka, która wychodzi tylko za zgodą człowieka")
def offer_reply_needs_consent():
    room = new_room()
    offers_ready(room)
    offer_from_interested(room, "oferta")
    client_decision(room, "oferta", "odrzucona")
    reply = {"adresat_rodzaj": "inny", "adresat_nazwa": "Piotr", "wzor": "odpowiedz-na-oferte"}

    prepare_letter(room, "wyslana", **reply)
    send(room, "wyslana")
    room.refused(
        "odpowiedź na ofertę wskazuje wysyłkę, która jeszcze nie wyszła",
        "wskaz_odpowiedz_na_oferte",
        id="oferta",
        odpowiedz_wysylka_id="wyslana",
    )
    prepare_letter(room, "odpowiedz", **reply)
    room.run("wskaz_odpowiedz_na_oferte", id="oferta", odpowiedz_wysylka_id="odpowiedz")
    room.refused(
        "odpowiedź na ofertę przed wysłaniem czeka na zgodę",
        "odnotuj_wyslanie",
        id="odpowiedz",
        wyslano="2026-11-10T12:00:00Z",
    )

    room.run("przekaz_wysylke_do_zgody", id="odpowiedz")
    approved_other(room, "wyslanie", "sprawa_korespondencja_wysylki", "odpowiedz", "wyslanie")
    send(room, "odpowiedz", decyzja_id="wyslanie")


@scenario("Przyjęcie oferty nie wychodzi, dopóki stoi znacznik wątpliwości")
def offer_acceptance_blocked_by_doubt():
    room = new_room()
    offers_ready(room)
    offer_from_interested(room, "oferta")
    client_decision(room, "oferta", "przyjeta")
    letter = reply_awaiting_consent(room, "oferta")
    room.run(
        "postaw_watpliwosc",
        id="watpliwosc",
        postawil="tozsamosc-kontrolera",
        postawiono="2026-11-10T13:00:00Z",
    )
    rule = "dopóki stoi znacznik wątpliwości, przyjęcie oferty nie wychodzi"

    room.refused(
        rule,
        "odnotuj_wyslanie",
        id=letter,
        wyslano="2026-11-10T14:00:00Z",
        decyzja_id=f"wyslanie-{letter}",
    )

    ask_for_decision(
        room,
        "zdjecie",
        "zdjecie_watpliwosci",
        "sprawa_metryka_watpliwosci",
        "watpliwosc",
        prosi_stanowisko="kontroler",
        zlozono="2026-11-10T15:00:00Z",
    )
    approve(room, "zdjecie", "2026-11-10T16:00:00Z")
    room.run("zdejmij_watpliwosc", id="watpliwosc", zdjecie_decyzja_id="zdjecie")
    send(room, letter, decyzja_id=f"wyslanie-{letter}", wyslano="2026-11-10T17:00:00Z")


@scenario(
    "Oferta klienta biura bez zgody umowy na obie strony idzie do klienta za zgodą człowieka (R1)"
)
def offer_from_office_client():
    room = new_room()
    offers_ready(room)
    interested_person(room, "klient-biura", dziala_jako="klient_biura")
    offer_from_interested(room, "oferta", osoba_id="klient-biura")
    rule = "ofertę klienta biura bez zgody umowy na obie strony"

    room.refused(
        rule, "odnotuj_przedstawienie_oferty", id="oferta", przedstawiono="2026-11-10T10:00:00Z"
    )
    approved_other(room, "obie-strony", "oferta_oferty_wpisy", "oferta")
    room.run(
        "odnotuj_przedstawienie_oferty",
        id="oferta",
        przedstawiono="2026-11-10T10:00:00Z",
        obie_strony_decyzja_id="obie-strony",
    )


@scenario("Oferta klienta biura przy umowie na obie strony nie wymaga decyzji człowieka")
def offer_from_office_client_both_sides():
    room = new_room()
    offers_ready(room, obie_strony=1)
    interested_person(room, "klient-biura", dziala_jako="klient_biura")
    offer_from_interested(room, "oferta", osoba_id="klient-biura")

    room.run("odnotuj_przedstawienie_oferty", id="oferta", przedstawiono="2026-11-10T10:00:00Z")


# R9 and replacement ------------------------------------------------------------------------------


def offer_tables(room):
    return [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'oferta\\_%' ESCAPE '\\'"
        )
    ]


def full_case_with_rental_terms(room):
    full_case(room)
    client_consent(room, "zgoda-warunki", "warunki_najmu", "oferta_najem_warunki", "warunki")
    room.run("wpisz_warunki_najmu", **rental_terms_fields("warunki"))


def primary_key_of(room, table):
    return [row["name"] for row in room.execute(f"PRAGMA table_info({table})") if row["pk"] > 0]


@scenario("R9: wierszy teczki oferty się nie usuwa")
def no_deletion():
    room = new_room()
    full_case_with_rental_terms(room)

    tables = offer_tables(room)
    assert len(tables) == 16, tables
    for table in tables:
        assert room.value(f"SELECT count(*) FROM {table}") > 0, f"{table} has no row to delete"
        room.refused_sql(
            DELETION, f"DELETE FROM {table} WHERE rowid = (SELECT min(rowid) FROM {table})"
        )


@scenario("Wiersza teczki oferty nie zastępuje INSERT OR REPLACE ani upsert")
def no_replacement():
    room = new_room()
    full_case_with_rental_terms(room)

    for table in offer_tables(room):
        key = ", ".join(primary_key_of(room, table))

        room.refused_sql(REPLACEMENT, f"INSERT OR REPLACE {first_row_copy(room, table)}")
        room.refused_sql(
            REPLACEMENT,
            f"INSERT {first_row_copy(room, table)} ON CONFLICT ({key}) DO UPDATE SET {key} = excluded.{key}",
        )
        if unique_keys_of(room, table):
            room.refused_sql(
                REPLACEMENT,
                f"INSERT OR REPLACE {first_row_copy(room, table, replace_unique_key=True)}",
            )
