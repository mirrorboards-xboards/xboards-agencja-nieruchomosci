"""Scenarios of the short-term rental case (spec/cases/lokal_krotkoterminowy).

A case room is the core of every case plus the kinds of the type. Each scenario starts from an
empty room and checks one rule the database keeps: a write the rule allows and a write it refuses.
The first scenario follows a whole case as the profile describes it: the management contract,
the flat, the offer, a guest's stay with its access code, the orders for contractors, the monthly
statement, the payout to the owner and the end of management.
"""

from pathlib import Path

from core import (
    WITHDRAWAL_ENDS,
    add_account,
    add_consent,
    add_flat,
    add_person,
    add_received_document,
    approved_decision,
    check,
    conclude_contract,
    confirm_account,
    open_case,
    prepare_contract,
    prepare_letter,
    send,
)
from core_porzadek import first_row_copy
from room import Room

CASE = Path(__file__).resolve().parents[1] / "spec" / "cases" / "lokal_krotkoterminowy"

TYPE = "lokal_krotkoterminowy"

SCENARIOS = []


def scenario(title):
    def register(function):
        SCENARIOS.append((title, function))

        return function

    return register


def new_room():
    """A short-term rental case room right after intake."""
    room = Room(CASE)
    open_case(room, typ_pokoju=TYPE)

    return room


# Building blocks of a short-term rental case -------------------------------------------------------

# The management contract is concluded on 2026-11-04T12:00:00Z (core.conclude_contract).
CONCLUDED = "2026-11-04T12:00:00Z"
CONTRACT_ENDS = "2027-01-31"


def stage(room, etap, od, decyzja_id=None):
    room.run("zmien_etap_sprawy", etap=etap, od=od, zmienil="agent-zarzadcy", decyzja_id=decyzja_id)


def add_owner(room, person_id="wlascicielka", **fields):
    add_person(room, person_id, klient_biura=1, kanal_kontaktu="email", **fields)
    room.run("nadaj_role_uczestnikowi", uczestnik_id=person_id, rola="wlasciciel_lokalu")


def add_guest(room, person_id="gosc"):
    add_person(
        room,
        person_id,
        imiona="Jan",
        nazwisko="Nowak",
        kanal_kontaktu="kanal_sprzedazy",
        dane_od="z_innego_zrodla",
        dane_pozyskano="2026-11-07T10:00:00Z",
    )
    room.run("nadaj_role_uczestnikowi", uczestnik_id=person_id, rola="gosc")


def sign_management_contract(room, consumer=False, koniec=CONTRACT_ENDS):
    """The owner checked by Kontroler signs; a consumer at a distance asks to start work at once."""
    check(room, "tozsamosc-wlascicielki", "tozsamosc", uczestnik_id="wlascicielka")
    prepare_contract(room, "umowa", ["wlascicielka"], nieruchomosc_id="lokal", koniec=koniec)
    room.run("przekaz_umowe_do_podpisu", id="umowa")

    if not consumer:
        conclude_contract(room, "umowa")
        return

    conclude_contract(room, "umowa", "na_odleglosc", WITHDRAWAL_ENDS)


def ask_to_start_work(room, chwila="2026-11-04T13:00:00Z"):
    """The consumer's explicit request to start before the withdrawal period ends (R8)."""
    add_consent(room, "zadanie", "wlascicielka", "zadanie_rozpoczecia_pracy", zlozono=chwila)
    room.run(
        "wpisz_zdarzenie_umowy",
        id="zdarzenie-zadanie",
        umowa_id="umowa",
        rodzaj="zadanie_rozpoczecia_pracy",
        chwila=chwila,
        zgoda_id="zadanie",
    )


def draft_terms(room, terms_id="warunki", od=CONCLUDED, aneks_zdarzenie_id=None, **fields):
    room.run(
        "wpisz_warunki_umowy_lokalu",
        **{
            "id": terms_id,
            "umowa_id": "umowa",
            "max_gosci": 4,
            "prowizja_rodzaj": "procent",
            "prowizja": 1500,
            "prowizja_podstawa": "przychód po prowizji kanału",
            "oplata_sprzatanie": 15_000,
            "oplata_sprzatanie_pokrywa_koszt": 1,
            "kwota_napraw": 50_000,
            "koszty_wlasciciela": "Biuro płaci z przychodu opłaty dla wspólnoty i internet.",
            "zgloszenie_nocy_dni": 14,
            "wyplata_dzien": 15,
            "rezerwa": 50_000,
            "po_koncu_rezerwacje": "Biuro odwołuje rezerwacje po końcu umowy za zgodą człowieka.",
            "aneks_zdarzenie_id": aneks_zdarzenie_id,
            "od": od,
            **fields,
        },
    )


def established_terms(room, terms_id="warunki", od=CONCLUDED, aneks_zdarzenie_id=None):
    """Terms with Airbnb on the office's account, Booking on the owner's and winter price limits."""
    draft_terms(room, terms_id, od, aneks_zdarzenie_id)
    room.run("dopisz_kanal_warunkow", warunki_id=terms_id, kanal="airbnb", konto="biura")
    room.run("dopisz_kanal_warunkow", warunki_id=terms_id, kanal="booking", konto="wlasciciela")
    room.run(
        "dopisz_widelki_warunkow",
        warunki_id=terms_id,
        sezon="zima 2026/2027",
        od_dnia="2026-11-01",
        do_dnia="2027-03-31",
        cena_min=25_000,
        cena_max=150_000,
        pobyt_min=2,
    )
    room.run("ustal_warunki_umowy_lokalu", id=terms_id)


def signed_protocol(room, protocol_id, rodzaj, chwila):
    room.run(
        "przygotuj_protokol",
        id=protocol_id,
        rodzaj=rodzaj,
        nieruchomosc_id="lokal",
        chwila=chwila,
    )
    room.run(
        "dopisz_pozycje_protokolu",
        protokol_id=protocol_id,
        rodzaj="klucze",
        nazwa="klucze do drzwi wejściowych",
        opis_stanu="2 sztuki",
    )
    add_received_document(room, f"podpisany-{protocol_id}", "protokol")
    room.run("podpisz_protokol", id=protocol_id, dokument_id=f"podpisany-{protocol_id}")


def draft_offer(room, offer_id="oferta-airbnb", kanal="airbnb", konto="biura", **fields):
    room.run(
        "przygotuj_oferte_w_kanale",
        **{
            "id": offer_id,
            "kanal": kanal,
            "konto": konto,
            "nazwa_konta": f"Biuro – {kanal}",
            **fields,
        },
    )


def publish(room, offer_id="oferta-airbnb", opublikowano="2026-11-06T10:00:00Z", **fields):
    room.run(
        "opublikuj_oferte_w_kanale",
        id=offer_id,
        numer_oferty=f"nr-{offer_id}",
        opublikowano=opublikowano,
        **fields,
    )


def managed_flat(room, consumer=False):
    """An owner, her flat, a concluded contract with established terms and a published offer."""
    add_owner(room, konsument=1 if consumer else 0)
    add_flat(room)
    sign_management_contract(room, consumer)
    if consumer:
        ask_to_start_work(room)
    established_terms(room)
    draft_offer(room)
    publish(room)


def book(room, stay_id="pobyt", przyjazd="2026-12-10", wyjazd="2026-12-13", **fields):
    room.run(
        "przyjmij_rezerwacje",
        **{
            "id": stay_id,
            "kanal": "airbnb",
            "numer_rezerwacji": f"HM-{stay_id}",
            "przyjeto": "2026-11-07T10:00:00Z",
            "przyjazd": przyjazd,
            "wyjazd": wyjazd,
            "gosci": 2,
            "gosc_uczestnik_id": "gosc",
            "kwota_goscia": 300_000,
            "oplata_sprzatanie": 15_000,
            "prowizja_kanalu": 45_000,
            "oplacona": 0,
            **fields,
        },
    )


def booked_flat(room):
    """A managed flat with a guest's stay from 10 to 13 December."""
    managed_flat(room)
    add_guest(room)
    book(room)


def order(room, order_id="sprzatanie", **fields):
    """An order from the register's price list, after the guest leaves."""
    room.run(
        "zlec_prace",
        **{
            "id": order_id,
            "rodzaj": "sprzatanie",
            "przyczyna": "wyjazd_goscia",
            "pobyt_id": "pobyt",
            "wykonawca_odnosnik": "rejestr-wykonawcow/czysty-dom",
            "spoza_rejestru": 0,
            "cennik_wersja_odnosnik": "rejestr-wykonawcow/czysty-dom/cennik-3",
            "poza_standardem": 0,
            "okno_od": "2026-12-13T10:00:00Z",
            "okno_do": "2026-12-13T15:00:00Z",
            "cena": 15_000,
            "cena_zrodlo": "cennik",
            "zlecil": "agent-koordynatora",
            "zlecono": "2026-12-01T09:00:00Z",
            **fields,
        },
    )


def confirm_order(room, order_id="sprzatanie", potwierdzono="2026-12-13T16:00:00Z"):
    room.run(
        "potwierdz_wykonanie_zlecenia",
        id=order_id,
        raport_sciezka=f"raporty/{order_id}.pdf",
        raport_skrot=f"sha256-{order_id}",
        potwierdzil="agent-koordynatora",
        potwierdzono=potwierdzono,
    )


def settle_stay(room, stay_id="pobyt", rozliczono="2026-12-15T09:00:00Z"):
    room.run(
        "odnotuj_rozliczenie_pobytu",
        id=stay_id,
        prowizja_kanalu=45_000,
        wyplata_kanalu=255_000,
        wyplacono="2026-12-14",
        rozliczono=rozliczono,
    )


def finished_stay(room, stay_id="pobyt"):
    room.run("odnotuj_oplacenie_pobytu", id=stay_id)
    room.run("zmien_stan_pobytu", id=stay_id, stan="trwa")
    room.run("zakoncz_pobyt", id=stay_id)


def send_with_consent(room, letter_id, day):
    """A letter that waits for the human's consent and goes out with it."""
    room.run("przekaz_wysylke_do_zgody", id=letter_id)
    approved_decision(
        room,
        f"wyslanie-{letter_id}",
        "wyslanie",
        "sprawa_korespondencja_wysylki",
        letter_id,
        zlozono=f"{day}T09:00:00Z",
        podpisano=f"{day}T10:00:00Z",
    )
    send(room, letter_id, decyzja_id=f"wyslanie-{letter_id}", wyslano=f"{day}T11:00:00Z")


def letter_to_owner(room, letter_id, wzor="zestawienie-miesieczne"):
    prepare_letter(
        room,
        letter_id,
        adresat_rodzaj=None,
        adresat_nazwa=None,
        adresat_uczestnik_id="wlascicielka",
        wzor=wzor,
    )


def letter_to_guest(room, letter_id):
    prepare_letter(
        room,
        letter_id,
        adresat_rodzaj=None,
        adresat_nazwa=None,
        adresat_uczestnik_id="gosc",
        kanal="kanal_sprzedazy",
        wzor="odpowiedz-na-opinie",
    )


def owner_account(room, account_id, cel):
    """An account of the owner confirmed by another channel, checked and admitted (R4)."""
    add_account(room, account_id, "wlascicielka", cel=cel, posiadacz="Anna Kowalska")
    confirm_account(room, account_id)
    check(room, f"sprawdzenie-{account_id}", "rachunek", rachunek_id=account_id)
    approved_decision(
        room,
        f"dopuszczenie-{account_id}",
        "dopuszczenie_rachunku",
        "sprawa_rachunki_wpisy",
        account_id,
    )


def draft_statement(room, statement_id="zestawienie", miesiac="2026-12", **fields):
    room.run(
        "przygotuj_zestawienie",
        **{
            "id": statement_id,
            "miesiac": miesiac,
            "utworzono": "2027-01-05T09:00:00Z",
            **fields,
        },
    )


def add_line(room, rodzaj, kwota, statement_id="zestawienie", **source):
    room.run(
        "dopisz_pozycje_zestawienia",
        zestawienie_id=statement_id,
        rodzaj=rodzaj,
        kwota=kwota,
        **source,
    )


def close_statement(room, kwota_do_wyplaty, statement_id="zestawienie", rezerwa_po=50_000):
    room.run(
        "zamknij_zestawienie",
        id=statement_id,
        sciezka=f"zestawienia/{statement_id}.pdf",
        skrot=f"sha256-{statement_id}",
        kwota_do_wyplaty=kwota_do_wyplaty,
        rezerwa_po=rezerwa_po,
    )


def send_statement(room, statement_id="zestawienie", day="2027-01-06"):
    letter_id = f"wysylka-{statement_id}"
    letter_to_owner(room, letter_id)
    room.run("wskaz_wysylke_zestawienia", id=statement_id, wysylka_id=letter_id)
    send_with_consent(room, letter_id, day)
    room.run("odnotuj_wyslanie_zestawienia", id=statement_id)


def prepare_owner_payout(room, payout_id, statement_id, kwota, day):
    room.run(
        "przygotuj_wyplate",
        id=payout_id,
        podstawa_tabela="lokal_krotkoterminowy_zestawienia_wpisy",
        podstawa_id=statement_id,
        kwota=kwota,
        wyliczenie_sciezka=f"wyliczenia/{payout_id}.json",
        wyliczenie_skrot=f"sha256-{payout_id}",
        odbiorca_uczestnik_id="wlascicielka",
        rachunek_id="rachunek-wyplat",
        utworzono=f"{day}T09:00:00Z",
    )


def pay_owner(room, payout_id, statement_id, kwota, day):
    """The payout of a sent statement: the account checked anew, the human's consent, the transfer."""
    prepare_owner_payout(room, payout_id, statement_id, kwota, day)
    check(
        room,
        f"sprawdzenie-{payout_id}",
        "rachunek",
        sprawdzono=f"{day}T10:00:00Z",
        rachunek_id="rachunek-wyplat",
    )
    room.run("wskaz_sprawdzenie_wyplaty", id=payout_id, sprawdzenie_id=f"sprawdzenie-{payout_id}")
    approved_decision(
        room,
        f"zgoda-{payout_id}",
        "wyplata",
        "sprawa_wyplaty_wpisy",
        payout_id,
        zlozono=f"{day}T11:00:00Z",
        podpisano=f"{day}T12:00:00Z",
    )
    room.run("wskaz_zgode_na_wyplate", id=payout_id, decyzja_id=f"zgoda-{payout_id}")
    add_received_document(room, f"przelew-{payout_id}", "potwierdzenie_bankowe")
    room.run(
        "odnotuj_wykonanie_wyplaty",
        id=payout_id,
        wykonal="tozsamosc-posrednika",
        wykonano=f"{day}T13:00:00Z",
        potwierdzenie_dokument_id=f"przelew-{payout_id}",
    )


def full_case(room):
    """A short-term rental case with a row in every table of the type, as the profile goes."""
    # Intake, the owner's identity, the contract with a consumer at a distance (R7, R8).
    stage(room, "weryfikacja_wlasciciela", "2026-11-02T12:00:00Z")
    add_owner(room, konsument=1)
    add_flat(room)
    stage(room, "umowa_o_zarzadzanie", "2026-11-03T09:00:00Z")
    sign_management_contract(room, consumer=True)
    ask_to_start_work(room)
    established_terms(room)

    # The flat: data, house rules, equipment bought by the office, a check-up and a resolution.
    stage(room, "przygotowanie_lokalu", "2026-11-05T09:00:00Z")
    add_received_document(room, "standard", "standard", nazwa="Standard lokalu")
    add_received_document(room, "instrukcja", "instrukcja", nazwa="Instrukcja wejścia")
    room.run(
        "wpisz_dane_lokalu",
        nieruchomosc_id="lokal",
        lozka=3,
        czas_sprzatania_min=120,
        klucze_liczba=2,
        standard_dokument_id="standard",
        instrukcja_dokument_id="instrukcja",
    )
    add_received_document(room, "regulamin-1", "regulamin", nazwa="Regulamin lokalu")
    room.run("wpisz_regulamin_lokalu", id="regulamin-1", dokument_id="regulamin-1", od=CONCLUDED)
    add_consent(room, "zgoda-poscel", "wlascicielka", "zgoda_na_wyposazenie", tresc="Pościel.")
    room.run(
        "wpisz_wyposazenie_lokalu",
        rzecz="pościel, 3 komplety",
        od="2026-11-05",
        koszt=45_000,
        zgoda_id="zgoda-poscel",
    )
    add_received_document(room, "protokol-gazowy", "inny", nazwa="Przegląd instalacji gazowej")
    room.run(
        "wpisz_przeglad_lokalu",
        rodzaj="instalacja_gazowa",
        data="2026-10-20",
        nastepny="2027-10-20",
        dokument_id="protokol-gazowy",
    )
    add_received_document(room, "uchwala", "uchwala", nazwa="Uchwała wspólnoty nr 4/2026")
    room.run(
        "wpisz_uchwale_wspolnoty",
        data="2026-10-01",
        tresc="Najem krótkoterminowy tylko z ciszą nocną od 22.",
        dokument_id="uchwala",
        dowiedziano="2026-11-05",
    )
    signed_protocol(room, "przejecie", "przejecie", "2026-11-05T10:00:00Z")
    order(
        room,
        "pierwsze-sprzatanie",
        przyczyna="przygotowanie_lokalu",
        pobyt_id=None,
        okno_od="2026-11-05T12:00:00Z",
        okno_do="2026-11-05T16:00:00Z",
        cena=20_000,
        zlecono="2026-11-05T09:00:00Z",
    )
    confirm_order(room, "pierwsze-sprzatanie", "2026-11-05T17:00:00Z")
    room.run("wpisz_koszt_zlecenia", id="pierwsze-sprzatanie", koszt=20_000)

    # The offer in the channel with the account the channel pays to, and a price within limits.
    owner_account(room, "rachunek-kanalu", "wyplata_w_kanale")
    stage(room, "oferta_w_kanalach", "2026-11-06T09:00:00Z")
    draft_offer(room, rachunek_id="rachunek-kanalu")
    publish(room)
    room.run(
        "wpisz_wyliczenie_cen",
        id="wyliczenie",
        chwila="2026-11-06T11:00:00Z",
        dane_wejsciowe="sezon zimowy, jarmark świąteczny, obłożenie 60%",
        zrodla="kalendarz miasta z 2026-11-06; ceny 5 podobnych lokali z 2026-11-06",
        sciezka="wyliczenia/ceny-grudzien.json",
        skrot="sha256-ceny-grudzien",
    )
    room.run(
        "ustaw_cene_nocy",
        id="cena-10-12",
        data="2026-12-10",
        kanal="airbnb",
        cena=100_000,
        ustawiono="2026-11-06T11:05:00Z",
        wyliczenie_id="wyliczenie",
    )
    stage(room, "prowadzenie_lokalu", "2026-11-07T09:00:00Z")

    # The calendar: a free night, the owner's night and the guest's stay.
    room.run("wpisz_noc", data="2026-12-01", stan="wolna", zmieniono="2026-11-07T09:00:00Z")
    add_consent(
        room,
        "noce-swieta",
        "wlascicielka",
        "noce_wlasciciela",
        tresc="Wigilia dla mnie.",
        zlozono="2026-11-08T09:00:00Z",
    )
    room.run(
        "wpisz_noc",
        data="2026-12-24",
        stan="wlasciciel",
        zgoda_id="noce-swieta",
        zmieniono="2026-11-08T10:00:00Z",
    )
    add_guest(room)
    book(room)
    for data in ("2026-12-10", "2026-12-11", "2026-12-12"):
        room.run(
            "wpisz_noc",
            data=data,
            stan="zarezerwowana",
            pobyt_id="pobyt",
            zmieniono="2026-11-07T10:05:00Z",
        )
    room.run(
        "wpisz_zmiane_pobytu",
        pobyt_id="pobyt",
        chwila="2026-11-20T10:00:00Z",
        co="liczba_gosci",
        przyczyna="gość przyjeżdża z partnerką",
        zdecydowal="agent-zarzadcy",
    )
    room.run(
        "zmien_rezerwacje",
        id="pobyt",
        przyjazd="2026-12-10",
        wyjazd="2026-12-13",
        gosci=3,
        kwota_goscia=300_000,
        oplata_sprzatanie=15_000,
    )

    # Before arrival: payment, a late check-out by exception, the code and the entry instruction.
    room.run("odnotuj_oplacenie_pobytu", id="pobyt")
    approved_decision(
        room, "pozne-wymeldowanie", "inna", "lokal_krotkoterminowy_pobyty_wyjatki", "wyjatek"
    )
    room.run(
        "wpisz_wyjatek_pobytu",
        id="wyjatek",
        pobyt_id="pobyt",
        opis="wymeldowanie o 12:00",
        decyzja_id="pozne-wymeldowanie",
    )
    room.run(
        "ustaw_dostep",
        pobyt_id="pobyt",
        rodzaj="kod",
        kod="4821",
        zamek_odnosnik="zamki/dluga-7-3",
        wazny_od="2026-12-10T15:00:00Z",
        wazny_do="2026-12-13T12:00:00Z",
        kto="gość z rezerwacji HM-pobyt",
        ustawiono="2026-12-09T09:00:00Z",
    )
    room.run("odnotuj_wydanie_dostepu", id=1, wydano="2026-12-10T10:00:00Z")
    room.run("odnotuj_instrukcje_wejscia", id="pobyt", instrukcja_wyslano="2026-12-10T10:00:00Z")
    room.run("zmien_stan_pobytu", id="pobyt", stan="trwa")

    # During the stay: a broken boiler, a repair from a quote within the repair amount, at a cost
    # other than the quote, and a compensation for the night without hot water.
    room.run(
        "wpisz_zgloszenie_goscia",
        id="zgloszenie",
        pobyt_id="pobyt",
        chwila="2026-12-11T20:00:00Z",
        opis="brak ciepłej wody",
        zagrozenie=0,
    )
    room.run(
        "odnotuj_przekazanie_zgloszenia",
        id="zgloszenie",
        przekazano="Koordynator operacji, 20:05",
    )
    order(
        room,
        "naprawa",
        rodzaj="naprawa",
        przyczyna="zgloszenie",
        zgloszenie_id="zgloszenie",
        wykonawca_odnosnik="rejestr-wykonawcow/hydraulik",
        cennik_wersja_odnosnik=None,
        okno_od="2026-12-20T08:00:00Z",
        okno_do="2026-12-20T18:00:00Z",
        cena=30_000,
        cena_zrodlo="wycena",
        zlecono="2026-12-11T20:30:00Z",
    )
    room.run(
        "wpisz_noc",
        data="2026-12-20",
        stan="prace",
        zlecenie_id="naprawa",
        zmieniono="2026-12-11T20:35:00Z",
    )
    room.run(
        "ustaw_dostep",
        zlecenie_id="naprawa",
        rodzaj="klucze",
        klucze_liczba=1,
        wazny_od="2026-12-20T08:00:00Z",
        wazny_do="2026-12-20T18:00:00Z",
        kto="hydraulik z rejestru",
        ustawiono="2026-12-19T09:00:00Z",
    )
    approved_decision(
        room,
        "rekompensata-decyzja",
        "inna",
        "lokal_krotkoterminowy_pobyty_pieniadze",
        "rekompensata",
    )
    room.run(
        "wpisz_pieniadze_pobytu",
        id="rekompensata",
        pobyt_id="pobyt",
        rodzaj="rekompensata",
        wedlug_zasad_kanalu=0,
        kwota=10_000,
        chwila="2026-12-12T09:00:00Z",
        decyzja_id="rekompensata-decyzja",
    )
    room.run("wpisz_skarge", chwila="2026-12-12T23:30:00Z", kto="sąsiad z lokalu 4", tresc="hałas")
    prepare_letter(
        room,
        "odpowiedz-sasiadowi",
        adresat_rodzaj="inny",
        adresat_nazwa="Sąsiad z lokalu 4",
        wzor="odpowiedz-na-skarge",
    )
    room.run("wskaz_odpowiedz_na_skarge", id=1, odpowiedz_wysylka_id="odpowiedz-sasiadowi")
    send_with_consent(room, "odpowiedz-sasiadowi", "2026-12-13")

    # After departure: cleaning in the window, the code expires, a low review answered with
    # consent, the channel's settlement.
    room.run("zakoncz_pobyt", id="pobyt", rzeczy_pozostawione="ładowarka do telefonu")
    room.run("odnotuj_odebranie_dostepu", id=1, odebrano="2026-12-13T12:00:00Z")
    order(room)
    room.run(
        "ustaw_dostep",
        zlecenie_id="sprzatanie",
        rodzaj="kod",
        kod="9930",
        wazny_od="2026-12-13T10:00:00Z",
        wazny_do="2026-12-13T15:00:00Z",
        kto="ekipa sprzątająca",
        ustawiono="2026-12-12T09:00:00Z",
    )
    confirm_order(room)
    room.run("wpisz_koszt_zlecenia", id="sprzatanie", koszt=15_000)
    confirm_order(room, "naprawa", "2026-12-20T17:00:00Z")
    approved_decision(
        room,
        "koszt-naprawy",
        "inna",
        "lokal_krotkoterminowy_zlecenia_wpisy",
        "naprawa",
        zlozono="2026-12-20T17:30:00Z",
        podpisano="2026-12-20T18:00:00Z",
    )
    room.run("wpisz_koszt_zlecenia", id="naprawa", koszt=32_000, koszt_decyzja_id="koszt-naprawy")
    room.run(
        "wpisz_opinie_goscia",
        id="opinia",
        pobyt_id="pobyt",
        ocena=2,
        opublikowano="2026-12-14T09:00:00Z",
    )
    letter_to_guest(room, "odpowiedz-na-opinie")
    room.run("wskaz_odpowiedz_na_opinie", id="opinia", odpowiedz_wysylka_id="odpowiedz-na-opinie")
    send_with_consent(room, "odpowiedz-na-opinie", "2026-12-15")
    settle_stay(room)

    # The statement for December: every line with its source, the amount is their sum.
    add_received_document(room, "naliczenie-wspolnoty", "pismo", nazwa="Naliczenie za grudzień")
    draft_statement(room)
    add_line(room, "przychod", 300_000, pobyt_id="pobyt")
    add_line(room, "prowizja_kanalu", -45_000, pobyt_id="pobyt")
    add_line(room, "prowizja_biura", -38_250, pobyt_id="pobyt", warunki_id="warunki")
    add_line(room, "koszt", -20_000, zlecenie_id="pierwsze-sprzatanie")
    add_line(room, "koszt", -15_000, zlecenie_id="sprzatanie")
    add_line(room, "koszt", -32_000, zlecenie_id="naprawa")
    add_line(room, "zwrot", -10_000, pieniadze_id="rekompensata")
    add_line(room, "oplata_lokalu", -5_000, dokument_id="naliczenie-wspolnoty")
    add_line(room, "rezerwa", -50_000, warunki_id="warunki")
    close_statement(room, 84_750)
    send_statement(room)

    # The payout after the account is checked anew (R4, D13).
    owner_account(room, "rachunek-wyplat", "wyplata_dla_wlasciciela")
    pay_owner(room, "wyplata-grudzien", "zestawienie", 84_750, "2027-01-08")

    # The end: no nights after the end date, the offer closed, the flat and keys returned.
    stage(room, "koniec_umowy", "2027-01-10T09:00:00Z")
    room.run("wpisz_noc", data="2027-02-01", stan="po_koncu", zmieniono="2027-01-10T09:05:00Z")
    stage(room, "zamkniecie_lokalu", "2027-02-01T09:00:00Z")
    room.run("zmien_stan_oferty_w_kanale", id="oferta-airbnb", stan="zamknieta")
    signed_protocol(room, "zwrot", "zwrot", "2027-02-01T10:00:00Z")
    stage(room, "ostatnie_rozliczenie", "2027-02-02T09:00:00Z")

    # The last statement corrects December with the human's consent and releases the reserve.
    add_received_document(room, "korekta-wspolnoty", "pismo", nazwa="Korekta naliczenia")
    approved_decision(
        room,
        "korekta",
        "inna",
        "lokal_krotkoterminowy_zestawienia_wpisy",
        "zestawienie-ostatnie",
    )
    draft_statement(
        room,
        "zestawienie-ostatnie",
        "2027-01",
        korekta_do_id="zestawienie",
        korekta_decyzja_id="korekta",
    )
    add_line(room, "korekta", 5_000, "zestawienie-ostatnie", dokument_id="korekta-wspolnoty")
    add_line(room, "rezerwa", 50_000, "zestawienie-ostatnie", warunki_id="warunki")
    close_statement(room, 55_000, "zestawienie-ostatnie", rezerwa_po=0)
    send_statement(room, "zestawienie-ostatnie", "2027-02-03")
    pay_owner(room, "wyplata-ostatnia", "zestawienie-ostatnie", 55_000, "2027-02-04")
    stage(room, "po_zarzadzaniu", "2027-02-05T09:00:00Z")


# The whole case ------------------------------------------------------------------------------------


@scenario("Przebieg lokalu: umowa, oferta, pobyt z kodem, zlecenia, zestawienia i wypłaty")
def whole_case():
    room = new_room()

    full_case(room)
    stage(room, "zamknieta", "2027-02-06T09:00:00Z")

    etapy = [row["etap"] for row in room.run("etapy")]
    assert etapy == [
        "przyjeta",
        "weryfikacja_wlasciciela",
        "umowa_o_zarzadzanie",
        "przygotowanie_lokalu",
        "oferta_w_kanalach",
        "prowadzenie_lokalu",
        "koniec_umowy",
        "zamkniecie_lokalu",
        "ostatnie_rozliczenie",
        "po_zarzadzaniu",
        "zamknieta",
    ], etapy
    assert [row["stan"] for row in room.run("wyplaty")] == ["wykonana", "wykonana"]
    assert [row["stan"] for row in room.run("zestawienia")] == ["wyslane", "wyslane"]
    assert room.run("pobyty_w_toku") == []
    assert room.run("zlecenia_otwarte") == []
    assert room.run("umowa_lokalu")[0]["praca_od"] == "2026-11-04T13:00:00Z"
    assert room.run("biezace_warunki_umowy_lokalu")[0]["id"] == "warunki"
    assert room.run("ceny_nocy")[0]["cena"] == 100_000

    dostep = room.run("dostepy_do_lokalu")[0]
    assert dostep["kod"] == "4821" and dostep["odebrano"] == "2026-12-13T12:00:00Z", dostep


# Rules of the case on the core ---------------------------------------------------------------------


@scenario("Reguły: teczka lokalu ma typ pokoju lokal_krotkoterminowy")
def room_type():
    room = Room(CASE)

    room.refused(
        "teczka lokalu krótkoterminowego ma typ pokoju lokal_krotkoterminowy",
        "zaloz_metryke",
        oznaczenie="Najem 2026/017",
        typ_pokoju="najem",
        przyjal="tozsamosc-posrednika",
        przyjeto="2026-11-02T09:00:00Z",
    )

    open_case(room, typ_pokoju=TYPE)
    assert room.run("metryka")[0]["typ_pokoju"] == TYPE


@scenario("Reguły: etapy lokalu idą po kolei, a przerwać można każdy przed zamknięciem")
def stages_in_order():
    room = new_room()

    room.refused(
        "etap spoza etapów lokalu krótkoterminowego",
        "zmien_etap_sprawy",
        etap="umowa",
        od="2026-11-02T10:00:00Z",
        zmienil="agent",
    )
    room.refused(
        "etapy lokalu idą po kolei",
        "zmien_etap_sprawy",
        etap="umowa_o_zarzadzanie",
        od="2026-11-02T10:00:00Z",
        zmienil="agent",
    )
    stage(room, "weryfikacja_wlasciciela", "2026-11-02T10:00:00Z")
    room.refused(
        "etapy lokalu idą po kolei",
        "zmien_etap_sprawy",
        etap="weryfikacja_wlasciciela",
        od="2026-11-02T11:00:00Z",
        zmienil="agent",
    )
    room.refused(
        "etapy lokalu idą po kolei",
        "zmien_etap_sprawy",
        etap="zamknieta",
        od="2026-11-02T11:00:00Z",
        zmienil="agent",
    )

    approved_decision(room, "przerwanie", "przerwanie_sprawy")
    stage(room, "przerwana", "2026-11-03T13:00:00Z", decyzja_id="przerwanie")
    room.refused(
        "etapy lokalu idą po kolei",
        "zmien_etap_sprawy",
        etap="umowa_o_zarzadzanie",
        od="2026-11-03T14:00:00Z",
        zmienil="agent",
    )
    stage(room, "zamknieta", "2026-11-03T14:00:00Z")


@scenario(
    "Reguły: wznowiony lokal wraca do etapu sprzed przerwania, który dalej wymaga zawartej umowy"
)
def stages_resumed():
    room = new_room()
    add_owner(room)
    add_flat(room)
    to_contract_stage(room)
    sign_management_contract(room)
    stage(room, "przygotowanie_lokalu", "2026-11-05T09:00:00Z")
    approved_decision(room, "przerwanie", "przerwanie_sprawy")
    stage(room, "przerwana", "2026-11-06T09:00:00Z", decyzja_id="przerwanie")
    approved_decision(room, "wznowienie", "wznowienie_sprawy", podpisano="2026-11-06T10:00:00Z")

    room.refused(
        "etapy lokalu idą po kolei",
        "zmien_etap_sprawy",
        etap="umowa_o_zarzadzanie",
        od="2026-11-06T11:00:00Z",
        zmienil="agent",
        decyzja_id="wznowienie",
    )
    stage(room, "przygotowanie_lokalu", "2026-11-06T11:00:00Z", decyzja_id="wznowienie")

    approved_decision(room, "drugie-przerwanie", "przerwanie_sprawy")
    stage(room, "przerwana", "2026-11-07T09:00:00Z", decyzja_id="drugie-przerwanie")
    room.run("zakoncz_umowe", id="umowa", koniec_powod="wypowiedzenie przez właścicielkę")
    approved_decision(
        room, "drugie-wznowienie", "wznowienie_sprawy", podpisano="2026-11-07T10:00:00Z"
    )
    room.refused(
        "etap przygotowanie_lokalu wymaga zawartej umowy o zarządzanie",
        "zmien_etap_sprawy",
        etap="przygotowanie_lokalu",
        od="2026-11-07T11:00:00Z",
        zmienil="agent",
        decyzja_id="drugie-wznowienie",
    )
    stage(room, "zamknieta", "2026-11-07T12:00:00Z")


def to_contract_stage(room):
    stage(room, "weryfikacja_wlasciciela", "2026-11-02T10:00:00Z")
    stage(room, "umowa_o_zarzadzanie", "2026-11-03T09:00:00Z")


@scenario("Reguły: przygotowanie lokalu wymaga zawartej umowy o zarządzanie")
def preparation_after_contract():
    room = new_room()
    add_owner(room)
    add_flat(room)
    to_contract_stage(room)

    room.refused(
        "etap przygotowanie_lokalu wymaga zawartej umowy o zarządzanie",
        "zmien_etap_sprawy",
        etap="przygotowanie_lokalu",
        od="2026-11-04T09:00:00Z",
        zmienil="agent",
    )

    sign_management_contract(room)
    stage(room, "przygotowanie_lokalu", "2026-11-05T09:00:00Z")


@scenario("Reguły: oferta w kanałach od praca_od konsumenta i po protokole przejęcia")
def offer_stage_after_work_start():
    room = new_room()
    add_owner(room, konsument=1)
    add_flat(room)
    to_contract_stage(room)
    sign_management_contract(room, consumer=True)
    stage(room, "przygotowanie_lokalu", "2026-11-05T09:00:00Z")
    signed_protocol(room, "przejecie", "przejecie", "2026-11-05T10:00:00Z")
    refusal = "etap oferta_w_kanalach wymaga zawartej umowy, od której biuro już może pracować"

    # Without the consumer's request the office works from the day after the withdrawal period.
    room.refused(
        refusal,
        "zmien_etap_sprawy",
        etap="oferta_w_kanalach",
        od="2026-11-06T09:00:00Z",
        zmienil="agent",
    )

    ask_to_start_work(room)
    stage(room, "oferta_w_kanalach", "2026-11-06T09:00:00Z")


@scenario("Reguły: oferta w kanałach wymaga podpisanego protokołu przejęcia")
def offer_stage_after_takeover():
    room = new_room()
    add_owner(room)
    add_flat(room)
    to_contract_stage(room)
    sign_management_contract(room)
    stage(room, "przygotowanie_lokalu", "2026-11-05T09:00:00Z")
    room.run(
        "przygotuj_protokol",
        id="przejecie",
        rodzaj="przejecie",
        nieruchomosc_id="lokal",
        chwila="2026-11-05T10:00:00Z",
    )

    room.refused(
        "i podpisanego protokołu przejęcia",
        "zmien_etap_sprawy",
        etap="oferta_w_kanalach",
        od="2026-11-06T09:00:00Z",
        zmienil="agent",
    )

    add_received_document(room, "podpisany", "protokol")
    room.run("podpisz_protokol", id="przejecie", dokument_id="podpisany")
    stage(room, "oferta_w_kanalach", "2026-11-06T09:00:00Z")


def offering_flat(room, koniec=None):
    """A case in the offer stage, with the contract without or with an end date."""
    add_owner(room)
    add_flat(room)
    to_contract_stage(room)
    sign_management_contract(room, koniec=koniec)
    stage(room, "przygotowanie_lokalu", "2026-11-05T09:00:00Z")
    signed_protocol(room, "przejecie", "przejecie", "2026-11-05T10:00:00Z")
    stage(room, "oferta_w_kanalach", "2026-11-06T09:00:00Z")


@scenario("Reguły: koniec umowy ma podstawę: datę końca albo wypowiedzenie")
def contract_end_needs_ground():
    refusal = "etap koniec_umowy wymaga daty końca umowy o zarządzanie, jej wypowiedzenia"

    # A contract for a fixed term ends on its end date, set here by an annex.
    room = new_room()
    offering_flat(room)
    room.refused(
        refusal,
        "zmien_etap_sprawy",
        etap="koniec_umowy",
        od="2026-12-01T09:00:00Z",
        zmienil="agent",
    )
    room.run(
        "wpisz_zdarzenie_umowy",
        id="aneks-koniec",
        umowa_id="umowa",
        rodzaj="aneks",
        chwila="2026-11-30T12:00:00Z",
        dokument_id="podpisana-umowa",
    )
    room.run(
        "odnotuj_aneks_umowy",
        id="umowa",
        koniec="2027-01-31",
        wylacznosc=1,
        wynagrodzenie_opis="10% czynszu miesięcznie",
        warunki_od="2026-11-30T12:00:00Z",
    )
    stage(room, "koniec_umowy", "2026-12-01T09:00:00Z")

    # A contract for an indefinite term ends by notice; the core changes its end date only by annex.
    room = new_room()
    offering_flat(room)
    add_received_document(room, "wypowiedzenie", "pismo")
    room.run(
        "wpisz_zdarzenie_umowy",
        id="wypowiedzenie",
        umowa_id="umowa",
        rodzaj="wypowiedzenie",
        chwila="2026-11-30T12:00:00Z",
        dokument_id="wypowiedzenie",
    )
    stage(room, "koniec_umowy", "2026-12-01T09:00:00Z")


@scenario("Reguły: ostatnie rozliczenie po podpisanym protokole zwrotu lokalu")
def last_settlement_after_return():
    room = new_room()
    offering_flat(room, koniec=CONTRACT_ENDS)
    stage(room, "koniec_umowy", "2027-01-10T09:00:00Z")
    stage(room, "zamkniecie_lokalu", "2027-02-01T09:00:00Z")

    room.refused(
        "etap ostatnie_rozliczenie wymaga podpisanego protokołu zwrotu lokalu",
        "zmien_etap_sprawy",
        etap="ostatnie_rozliczenie",
        od="2027-02-02T09:00:00Z",
        zmienil="agent",
    )

    signed_protocol(room, "zwrot", "zwrot", "2027-02-01T10:00:00Z")
    stage(room, "ostatnie_rozliczenie", "2027-02-02T09:00:00Z")


@scenario("Reguły: role, zgody i umowa o zarządzanie z listy lokalu")
def roles_consents_contract_kind():
    room = new_room()
    add_person(room, "osoba")

    room.refused(
        "rola spoza ról lokalu", "nadaj_role_uczestnikowi", uczestnik_id="osoba", rola="klient"
    )
    room.run("nadaj_role_uczestnikowi", uczestnik_id="osoba", rola="zarzadca_budynku")
    room.refused_sql("rola spoza ról lokalu", "UPDATE sprawa_uczestnicy_role SET rola = 'najemca'")

    room.refused(
        "zgoda spoza zgód lokalu",
        "wpisz_zgode",
        id="cena",
        uczestnik_id="osoba",
        rodzaj="zatwierdzenie_ceny",
        tresc="Cena 500 zł",
        zlozono="2026-11-04T13:00:00Z",
        wplyw_id=None,
        dokument_id=None,
        trwaly_nosnik=0,
    )
    add_consent(room, "kanal", "osoba", "zgoda_na_kanal", tresc="Zgadzam się na Vrbo.")

    room.refused(
        "teczka lokalu ma tylko umowę o zarządzanie",
        "przygotuj_umowe",
        id="posrednictwo",
        rodzaj="posrednictwo",
        obie_strony=0,
        wylacznosc=0,
    )
    room.run("przygotuj_umowe", id="umowa", rodzaj="zarzadzanie", obie_strony=0, wylacznosc=0)
    room.refused_sql(
        "rodzaj umowy się nie zmienia",
        "UPDATE sprawa_umowy_wpisy SET rodzaj = 'posrednictwo'",
    )


@scenario("Reguły: teczka lokalu ma jedną zawartą umowę; nowa umowa to nowa teczka")
def one_concluded_contract():
    room = new_room()
    add_owner(room)
    add_flat(room)
    sign_management_contract(room)
    prepare_contract(room, "druga", ["wlascicielka"])
    room.run("przekaz_umowe_do_podpisu", id="druga")

    room.refused(
        "teczka lokalu ma najwyżej jedną zawartą umowę o zarządzanie",
        "odnotuj_zawarcie_umowy",
        id="druga",
        forma="pisemna",
        sposob_zawarcia="w_lokalu_biura",
        zawarto="2026-11-10T12:00:00Z",
        podpisal_za_biuro="tozsamosc-posrednika",
        dokument_id="podpisana-umowa",
        polisa_dokument_id="polisa-umowa",
    )

    room.run("zakoncz_umowe", id="druga", koniec_powod="klientka podpisała pierwszą umowę")


@scenario("Reguły: każde pismo do wspólnoty i zarządcy budynku wychodzi za zgodą")
def letters_to_association_with_consent():
    room = new_room()
    add_person(room, "zarzadca", imiona="Piotr", nazwisko="Zieliński")
    room.run("nadaj_role_uczestnikowi", uczestnik_id="zarzadca", rola="zarzadca_budynku")
    prepare_letter(room, "do-wspolnoty")
    prepare_letter(
        room,
        "do-zarzadcy",
        adresat_rodzaj=None,
        adresat_nazwa=None,
        adresat_uczestnik_id="zarzadca",
    )
    prepare_letter(room, "do-urzedu", adresat_rodzaj="urzad", adresat_nazwa="Urząd Miasta")
    refusal = "pismo do wspólnoty, spółdzielni albo zarządcy budynku przed wysłaniem czeka na zgodę"

    room.refused(refusal, "odnotuj_wyslanie", id="do-wspolnoty", wyslano="2026-11-05T10:00:00Z")
    room.refused(refusal, "odnotuj_wyslanie", id="do-zarzadcy", wyslano="2026-11-05T10:00:00Z")

    send_with_consent(room, "do-wspolnoty", "2026-11-05")
    send(room, "do-urzedu")


# Terms of the management contract ------------------------------------------------------------------


@scenario("Umowa: warunki powstają jako projekt przy zawartej umowie i obowiązują od zawarcia")
def terms_start_with_contract():
    room = new_room()
    add_owner(room)
    add_flat(room)
    prepare_contract(room, "umowa", ["wlascicielka"])

    room.refused(
        "warunki wpisuje się do zawartej umowy o zarządzanie",
        "wpisz_warunki_umowy_lokalu",
        **terms_fields(),
    )

    check(room, "tozsamosc", "tozsamosc", uczestnik_id="wlascicielka")
    room.run("przekaz_umowe_do_podpisu", id="umowa")
    conclude_contract(room, "umowa")
    room.refused(
        "pierwsze warunki obowiązują od zawarcia umowy",
        "wpisz_warunki_umowy_lokalu",
        **terms_fields(od="2026-11-05T09:00:00Z"),
    )
    room.refused_sql(
        "warunki umowy powstają jako projekt",
        "INSERT INTO lokal_krotkoterminowy_umowa_warunki (id, umowa_id, max_gosci, prowizja_rodzaj, prowizja, prowizja_podstawa, oplata_sprzatanie, oplata_sprzatanie_pokrywa_koszt, kwota_napraw, koszty_wlasciciela, zgloszenie_nocy_dni, wyplata_dzien, rezerwa, po_koncu_rezerwacje, od, stan) VALUES ('od-razu', 'umowa', 4, 'procent', 1500, 'przychód', 0, 1, 0, 'brak', 14, 15, 0, 'brak', :od, 'obowiazuje')",
        od=CONCLUDED,
    )

    established_terms(room)
    assert room.run("biezace_warunki_umowy_lokalu")[0]["stan"] == "obowiazuje"


def terms_fields(**fields):
    return {
        "id": "warunki",
        "umowa_id": "umowa",
        "max_gosci": 4,
        "prowizja_rodzaj": "procent",
        "prowizja": 1500,
        "prowizja_podstawa": "przychód po prowizji kanału",
        "oplata_sprzatanie": 15_000,
        "oplata_sprzatanie_pokrywa_koszt": 1,
        "kwota_napraw": 50_000,
        "koszty_wlasciciela": "opłaty wspólnoty z przychodu",
        "zgloszenie_nocy_dni": 14,
        "wyplata_dzien": 15,
        "rezerwa": 50_000,
        "po_koncu_rezerwacje": "odwołanie za zgodą człowieka",
        "od": CONCLUDED,
        **fields,
    }


@scenario("Umowa: następne warunki wskazują aneks i obowiązują od jego chwili")
def terms_change_by_annex():
    room = new_room()
    add_owner(room)
    add_flat(room)
    sign_management_contract(room)
    established_terms(room)

    room.refused(
        "każde następne wskazują aneks tej umowy",
        "wpisz_warunki_umowy_lokalu",
        **terms_fields(id="bez-aneksu", od="2026-12-01T09:00:00Z"),
    )

    room.run(
        "wpisz_zdarzenie_umowy",
        id="aneks",
        umowa_id="umowa",
        rodzaj="aneks",
        chwila="2026-12-01T09:00:00Z",
        dokument_id="podpisana-umowa",
    )
    room.refused(
        "każde następne wskazują aneks tej umowy",
        "wpisz_warunki_umowy_lokalu",
        **terms_fields(id="zla-chwila", od="2026-12-02T09:00:00Z", aneks_zdarzenie_id="aneks"),
    )

    established_terms(room, "warunki-2", "2026-12-01T09:00:00Z", "aneks")
    assert room.run("biezace_warunki_umowy_lokalu")[0]["id"] == "warunki-2"


@scenario(
    "Umowa: ustalone warunki mają kanał i się nie zmieniają, a kanały i widełki tylko w projekcie"
)
def terms_frozen():
    room = new_room()
    add_owner(room)
    add_flat(room)
    sign_management_contract(room)
    room.refused(
        "procent_najwyzej_100", "wpisz_warunki_umowy_lokalu", **terms_fields(prowizja=12_000)
    )
    draft_terms(room)

    room.refused(
        "ustalone warunki mają co najmniej jeden kanał sprzedaży",
        "ustal_warunki_umowy_lokalu",
        id="warunki",
    )

    room.run("dopisz_kanal_warunkow", warunki_id="warunki", kanal="airbnb", konto="biura")
    room.run("ustal_warunki_umowy_lokalu", id="warunki")

    room.refused_sql(
        "warunków umowy się nie zmienia",
        "UPDATE lokal_krotkoterminowy_umowa_warunki SET kwota_napraw = 90000",
    )
    room.refused(
        "kanały dopisuje się tylko do projektu warunków",
        "dopisz_kanal_warunkow",
        warunki_id="warunki",
        kanal="booking",
        konto="biura",
    )
    room.refused(
        "widełki dopisuje się tylko do projektu warunków",
        "dopisz_widelki_warunkow",
        warunki_id="warunki",
        sezon="lato",
        od_dnia="2027-06-01",
        do_dnia="2027-08-31",
        cena_min=30_000,
        cena_max=90_000,
        pobyt_min=2,
    )
    room.refused_sql(
        "kanału warunków się nie zmienia",
        "UPDATE lokal_krotkoterminowy_umowa_kanaly SET konto = 'wlasciciela'",
    )


@scenario(
    "Umowa: widełki mają cenę najniższą nie większą niż najwyższą, a okresy się nie nakładają"
)
def price_limits():
    room = new_room()
    add_owner(room)
    add_flat(room)
    sign_management_contract(room)
    draft_terms(room)
    limits = {
        "warunki_id": "warunki",
        "sezon": "zima",
        "od_dnia": "2026-11-01",
        "do_dnia": "2027-03-31",
        "cena_min": 25_000,
        "cena_max": 90_000,
        "pobyt_min": 2,
    }

    room.refused(
        "cena_min_nie_wieksza_niz_max", "dopisz_widelki_warunkow", **{**limits, "cena_max": 20_000}
    )
    room.run("dopisz_widelki_warunkow", **limits)
    room.refused(
        "okresy widełek jednej wersji warunków się nie nakładają",
        "dopisz_widelki_warunkow",
        **{**limits, "sezon": "ferie", "od_dnia": "2027-02-01", "do_dnia": "2027-02-28"},
    )
    room.run(
        "dopisz_widelki_warunkow",
        **{**limits, "sezon": "wiosna", "od_dnia": "2027-04-01", "do_dnia": "2027-05-31"},
    )
    room.refused_sql(
        "widełek się nie zmienia", "UPDATE lokal_krotkoterminowy_umowa_widelki SET cena_max = 99000"
    )


# The flat ------------------------------------------------------------------------------------------


@scenario("Lokal: dane lokalu w jednym wierszu, rejestracja z numerem i datą")
def flat_data():
    room = new_room()
    add_flat(room)

    room.refused(
        "rejestracja_numer_z_data",
        "wpisz_dane_lokalu",
        nieruchomosc_id="lokal",
        lozka=2,
        rejestracja_numer="REJ/1",
    )
    room.run("wpisz_dane_lokalu", nieruchomosc_id="lokal", lozka=2)
    room.refused(
        "wiersza teczki nie zastępuje się nowym wpisem",
        "wpisz_dane_lokalu",
        nieruchomosc_id="lokal",
        lozka=3,
    )
    room.run("zmien_dane_lokalu", lozka=3, rejestracja_numer="REJ/1", rejestracja_data="2026-12-01")

    assert room.run("dane_lokalu")[0]["rejestracja_numer"] == "REJ/1"


@scenario("Lokal: każdy regulamin po pierwszym ma zgodę człowieka i się nie zmienia")
def house_rules_change_with_consent():
    room = new_room()
    add_received_document(room, "regulamin-1", "regulamin")
    add_received_document(room, "regulamin-2", "regulamin")
    room.run("wpisz_regulamin_lokalu", id="regulamin-1", dokument_id="regulamin-1", od="2026-11-05")

    room.refused(
        "każdy regulamin po pierwszym wskazuje zatwierdzoną decyzję inna",
        "wpisz_regulamin_lokalu",
        id="regulamin-2",
        dokument_id="regulamin-2",
        od="2026-12-01",
    )
    approved_decision(
        room, "inny-wiersz", "inna", "lokal_krotkoterminowy_lokal_regulaminy", "regulamin-1"
    )
    room.refused(
        "każdy regulamin po pierwszym wskazuje zatwierdzoną decyzję inna",
        "wpisz_regulamin_lokalu",
        id="regulamin-2",
        dokument_id="regulamin-2",
        od="2026-12-01",
        decyzja_id="inny-wiersz",
    )

    approved_decision(
        room, "zmiana", "inna", "lokal_krotkoterminowy_lokal_regulaminy", "regulamin-2"
    )
    room.run(
        "wpisz_regulamin_lokalu",
        id="regulamin-2",
        dokument_id="regulamin-2",
        od="2026-12-01",
        decyzja_id="zmiana",
    )
    room.refused_sql(
        "regulaminu się nie zmienia",
        "UPDATE lokal_krotkoterminowy_lokal_regulaminy SET od = '2026-12-02'",
    )


@scenario("Lokal: wyposażenie z kosztem ma datę, zgoda to zgoda_na_wyposazenie, wycofanie raz")
def equipment():
    room = new_room()
    add_owner(room)
    add_consent(room, "naprawa", "wlascicielka", "zgoda_na_naprawe")
    add_consent(room, "wyposazenie", "wlascicielka", "zgoda_na_wyposazenie")

    room.refused("koszt_z_data", "wpisz_wyposazenie_lokalu", rzecz="czajnik", koszt=12_000)
    room.refused(
        "wyposażenie wskazuje zgodę zgoda_na_wyposazenie",
        "wpisz_wyposazenie_lokalu",
        rzecz="czajnik",
        od="2026-11-05",
        koszt=12_000,
        zgoda_id="naprawa",
    )
    room.run(
        "wpisz_wyposazenie_lokalu",
        rzecz="czajnik",
        od="2026-11-05",
        koszt=12_000,
        zgoda_id="wyposazenie",
    )

    room.refused_sql(
        "wyposażenia się nie zmienia",
        "UPDATE lokal_krotkoterminowy_lokal_wyposazenie SET koszt = 9000",
    )
    room.refused(
        "wycofanie_nie_przed_zakupem",
        "wycofaj_wyposazenie_lokalu",
        id=1,
        do_dnia="2026-11-01",
        powod="zepsuty",
    )
    room.run("wycofaj_wyposazenie_lokalu", id=1, do_dnia="2027-01-10", powod="zepsuty")
    room.refused(
        "wyposażenia się nie zmienia",
        "wycofaj_wyposazenie_lokalu",
        id=1,
        do_dnia="2027-01-11",
        powod="zgubiony",
    )


@scenario(
    "Lokal: przegląd ma następny termin po sobie, uchwała jest znana po podjęciu; obu się nie zmienia"
)
def checkups_and_resolutions():
    room = new_room()

    room.refused(
        "nastepny_po_przegladzie",
        "wpisz_przeglad_lokalu",
        rodzaj="czujnik_co",
        data="2026-11-05",
        nastepny="2026-11-05",
    )
    room.run("wpisz_przeglad_lokalu", rodzaj="czujnik_co", data="2026-11-05", nastepny="2027-11-05")
    room.refused_sql(
        "przeglądu się nie zmienia",
        "UPDATE lokal_krotkoterminowy_lokal_przeglady SET nastepny = '2028-11-05'",
    )

    room.refused(
        "dowiedziano_po_podjeciu",
        "wpisz_uchwale_wspolnoty",
        data="2026-10-01",
        tresc="Zakaz",
        dowiedziano="2026-09-30",
    )
    room.run("wpisz_uchwale_wspolnoty", data="2026-10-01", tresc="Zakaz", dowiedziano="2026-11-05")
    room.refused_sql(
        "uchwały się nie zmienia", "UPDATE lokal_krotkoterminowy_lokal_uchwaly SET tresc = 'Zgoda'"
    )


# Stays ---------------------------------------------------------------------------------------------


@scenario("Pobyty: rezerwacja przy zawartej umowie, nie przed praca_od konsumenta")
def booking_after_work_start():
    room = new_room()
    add_owner(room, konsument=1)
    add_flat(room)
    sign_management_contract(room, consumer=True)
    established_terms(room)
    draft_offer(room)
    # The publication waits for the end of the withdrawal period, the day after WITHDRAWAL_ENDS.
    room.refused(
        "ofertę publikuje się przy zawartej umowie o zarządzanie, nie wcześniej niż praca_od",
        "opublikuj_oferte_w_kanale",
        id="oferta-airbnb",
        numer_oferty="nr-1",
        opublikowano="2026-11-06T10:00:00Z",
    )
    publish(room, opublikowano="2026-11-19T08:00:00Z")
    add_guest(room)

    # A booking comes after the publication, so the publication already keeps praca_od.
    book(room, przyjeto="2026-11-19T09:00:00Z")

    room.run("zakoncz_umowe", id="umowa", koniec_powod="wypowiedzenie")
    room.refused(
        "rezerwację przyjmuje się przy zawartej umowie o zarządzanie",
        "przyjmij_rezerwacje",
        **booking_fields(id="po-koncu", numer_rezerwacji="HM-2", przyjeto="2026-11-20T09:00:00Z"),
    )


def booking_fields(**fields):
    return {
        "id": "pobyt",
        "kanal": "airbnb",
        "numer_rezerwacji": "HM-pobyt",
        "przyjeto": "2026-11-07T10:00:00Z",
        "przyjazd": "2026-12-10",
        "wyjazd": "2026-12-13",
        "gosci": 2,
        "gosc_uczestnik_id": "gosc",
        "kwota_goscia": 300_000,
        "oplata_sprzatanie": 15_000,
        "oplacona": 0,
        **fields,
    }


@scenario("Pobyty: rezerwacja z kanału z opublikowaną ofertą, nie sprzed publikacji")
def booking_from_channel_with_offer():
    room = new_room()
    managed_flat(room)
    add_guest(room)

    room.refused(
        "rezerwacja przychodzi z kanału, w którym lokal ma opublikowaną ofertę",
        "przyjmij_rezerwacje",
        **booking_fields(kanal="booking"),
    )
    room.refused(
        "rezerwacja przychodzi z kanału, w którym lokal ma opublikowaną ofertę",
        "przyjmij_rezerwacje",
        **booking_fields(przyjeto="2026-11-06T09:00:00Z"),
    )
    book(room)


@scenario("Pobyty: liczba gości do największej z umowy, gość z rolą gosc, wyjazd po przyjeździe")
def booking_guests():
    room = new_room()
    managed_flat(room)
    add_person(room, "bez-roli", imiona="Ewa", nazwisko="Lis")
    add_guest(room)

    room.refused(
        "liczba gości nie przekracza największej liczby gości",
        "przyjmij_rezerwacje",
        **booking_fields(gosci=5),
    )
    room.refused(
        "pobyt wskazuje uczestnika z rolą gosc",
        "przyjmij_rezerwacje",
        **booking_fields(gosc_uczestnik_id="bez-roli"),
    )
    room.refused(
        "wyjazd_po_przyjezdzie", "przyjmij_rezerwacje", **booking_fields(wyjazd="2026-12-10")
    )
    book(room)

    room.refused(
        "liczba gości nie przekracza największej liczby gości",
        "zmien_rezerwacje",
        id="pobyt",
        przyjazd="2026-12-10",
        wyjazd="2026-12-13",
        gosci=5,
        kwota_goscia=300_000,
        oplata_sprzatanie=15_000,
    )
    room.run(
        "zmien_rezerwacje",
        id="pobyt",
        przyjazd="2026-12-10",
        wyjazd="2026-12-14",
        gosci=4,
        kwota_goscia=400_000,
        oplata_sprzatanie=15_000,
    )
    room.refused(
        "wiersza teczki nie zastępuje się nowym wpisem",
        "przyjmij_rezerwacje",
        **booking_fields(id="drugi"),
    )


@scenario("Pobyty: noce po dacie końca umowy wymagają zgody człowieka, także po zmianie terminu")
def booking_after_contract_end():
    room = new_room()
    add_owner(room)
    add_flat(room)
    sign_management_contract(room)
    established_terms(room)
    draft_offer(room)
    publish(room)
    add_guest(room)
    refusal = "pobyt z nocami po dacie końca umowy wskazuje zatwierdzoną decyzję inna"

    # The last night of 30 January to 1 February is 31 January, the end date itself.
    book(room, "na-koniec", "2027-01-30", "2027-02-01")
    room.refused(
        refusal,
        "przyjmij_rezerwacje",
        **booking_fields(
            id="po-koncu", numer_rezerwacji="HM-2", przyjazd="2027-01-31", wyjazd="2027-02-02"
        ),
    )
    room.refused(
        refusal,
        "zmien_rezerwacje",
        id="na-koniec",
        przyjazd="2027-01-30",
        wyjazd="2027-02-02",
        gosci=2,
        kwota_goscia=300_000,
        oplata_sprzatanie=15_000,
    )

    approved_decision(
        room, "po-koncu-decyzja", "inna", "lokal_krotkoterminowy_pobyty_wpisy", "po-koncu"
    )
    book(
        room,
        "po-koncu",
        "2027-01-31",
        "2027-02-02",
        numer_rezerwacji="HM-2",
        po_koncu_decyzja_id="po-koncu-decyzja",
    )


@scenario("Pobyty: przebieg pobytu, instrukcja tylko do opłaconej, rozliczenie raz")
def stay_course():
    room = new_room()
    booked_flat(room)

    room.refused(
        "instrukcja_tylko_do_oplaconej",
        "odnotuj_instrukcje_wejscia",
        id="pobyt",
        instrukcja_wyslano="2026-12-10T10:00:00Z",
    )
    room.refused("pobyt przechodzi", "zakoncz_pobyt", id="pobyt")
    room.refused_sql(
        "pobyt powstaje potwierdzony",
        "INSERT INTO lokal_krotkoterminowy_pobyty_wpisy (id, kanal, numer_rezerwacji, przyjeto, przyjazd, wyjazd, gosci, gosc_uczestnik_id, kwota_goscia, oplata_sprzatanie, oplacona, stan) VALUES ('trwa', 'airbnb', 'HM-3', '2026-11-08T10:00:00Z', '2026-12-20', '2026-12-22', 2, 'gosc', 1, 1, 1, 'trwa')",
    )

    room.run("odnotuj_oplacenie_pobytu", id="pobyt")
    room.run("odnotuj_instrukcje_wejscia", id="pobyt", instrukcja_wyslano="2026-12-10T10:00:00Z")
    room.refused(
        "instrukcję wejścia i rozliczenie pobytu wpisuje się raz",
        "odnotuj_instrukcje_wejscia",
        id="pobyt",
        instrukcja_wyslano="2026-12-10T11:00:00Z",
    )
    room.refused_sql(
        "kanał, numer rezerwacji, chwila przyjęcia i gość się nie zmieniają",
        "UPDATE lokal_krotkoterminowy_pobyty_wpisy SET kanal = 'booking'",
    )

    room.run("zmien_stan_pobytu", id="pobyt", stan="trwa")
    room.run("zakoncz_pobyt", id="pobyt")
    room.refused(
        "termin, goście, opłacenie i instrukcja zakończonego albo odwołanego pobytu się nie zmieniają",
        "zmien_rezerwacje",
        id="pobyt",
        przyjazd="2026-12-10",
        wyjazd="2026-12-14",
        gosci=2,
        kwota_goscia=300_000,
        oplata_sprzatanie=15_000,
    )

    room.refused(
        "rozliczenie_w_komplecie",
        "odnotuj_rozliczenie_pobytu",
        id="pobyt",
        prowizja_kanalu=45_000,
        rozliczono="2026-12-15T09:00:00Z",
    )
    settle_stay(room)
    room.refused_sql(
        "instrukcję wejścia i rozliczenie pobytu wpisuje się raz",
        "UPDATE lokal_krotkoterminowy_pobyty_wpisy SET wyplata_kanalu = 1",
    )


@scenario("Pobyty: zgłoszenie zagrożenia ma zawiadomienie, a przekazanie wpisuje się raz")
def guest_reports():
    room = new_room()
    booked_flat(room)
    report = {
        "id": "zgloszenie",
        "pobyt_id": "pobyt",
        "chwila": "2026-12-11T20:00:00Z",
        "opis": "zapach gazu",
        "zagrozenie": 1,
    }

    room.refused("zagrozenie_ma_zawiadomienie", "wpisz_zgloszenie_goscia", **report)
    room.run(
        "wpisz_zgloszenie_goscia",
        **report,
        zawiadomiono="gość zadzwonił pod 112; zawiadomiony człowiek prowadzący lokal",
    )
    room.run(
        "odnotuj_przekazanie_zgloszenia", id="zgloszenie", przekazano="Koordynator operacji, 20:02"
    )

    room.refused(
        "zgłoszenia się nie zmienia",
        "odnotuj_przekazanie_zgloszenia",
        id="zgloszenie",
        przekazano="Zarządca",
    )
    room.refused_sql(
        "zgłoszenia się nie zmienia",
        "UPDATE lokal_krotkoterminowy_pobyty_zgloszenia SET opis = 'nic'",
    )
    room.run(
        "wpisz_zmiane_pobytu",
        pobyt_id="pobyt",
        chwila="2026-11-20T10:00:00Z",
        co="cena",
        przyczyna="rabat",
        zdecydowal="agent-zarzadcy",
    )
    room.refused_sql(
        "zmiany pobytu się nie zmienia",
        "UPDATE lokal_krotkoterminowy_pobyty_zmiany SET przyczyna = 'inna'",
    )


@scenario(
    "Pobyty: wyjątek, rekompensata, roszczenie i zwrot ponad zasady kanału mają zgodę człowieka"
)
def stay_exceptions_and_money():
    room = new_room()
    booked_flat(room)
    money = {
        "pobyt_id": "pobyt",
        "wedlug_zasad_kanalu": 0,
        "kwota": 10_000,
        "chwila": "2026-12-12T09:00:00Z",
    }
    refusal = "rekompensata, roszczenie, wniosek kanału i zwrot ponad zasady kanału wskazują zatwierdzoną decyzję"

    room.refused(
        "wyjątek od regulaminu wskazuje zatwierdzoną decyzję inna",
        "wpisz_wyjatek_pobytu",
        id="zwierze",
        pobyt_id="pobyt",
        opis="pies",
    )
    approved_decision(room, "pies", "inna", "lokal_krotkoterminowy_pobyty_wyjatki", "zwierze")
    room.run("wpisz_wyjatek_pobytu", id="zwierze", pobyt_id="pobyt", opis="pies", decyzja_id="pies")
    room.refused_sql(
        "wyjątku się nie zmienia",
        "UPDATE lokal_krotkoterminowy_pobyty_wyjatki SET opis = 'dwa psy'",
    )

    for rodzaj in ("rekompensata", "roszczenie", "wniosek_kanalu", "zwrot"):
        room.refused(refusal, "wpisz_pieniadze_pobytu", id=rodzaj, rodzaj=rodzaj, **money)
    room.refused(
        "wedlug_zasad_kanalu_tylko_zwrot",
        "wpisz_pieniadze_pobytu",
        id="r",
        rodzaj="odzysk",
        **{**money, "wedlug_zasad_kanalu": 1},
    )

    room.run(
        "wpisz_pieniadze_pobytu", id="zwrot", rodzaj="zwrot", **{**money, "wedlug_zasad_kanalu": 1}
    )
    room.run("wpisz_pieniadze_pobytu", id="odzysk", rodzaj="odzysk", **money)
    approved_decision(
        room, "szkoda", "inna", "lokal_krotkoterminowy_pobyty_pieniadze", "roszczenie"
    )
    room.run(
        "wpisz_pieniadze_pobytu", id="roszczenie", rodzaj="roszczenie", decyzja_id="szkoda", **money
    )
    room.refused_sql(
        "pieniędzy pobytu się nie zmienia",
        "UPDATE lokal_krotkoterminowy_pobyty_pieniadze SET kwota = 1",
    )


@scenario(
    "Pobyty: odpowiedź na opinię 3 i niżej oraz na skargę czeka na zgodę; wskazuje się ją przed wysłaniem"
)
def answers_with_consent():
    room = new_room()
    booked_flat(room)
    room.run(
        "wpisz_opinie_goscia",
        id="slaba",
        pobyt_id="pobyt",
        ocena=3,
        opublikowano="2026-12-14T09:00:00Z",
    )
    room.run(
        "wpisz_skarge",
        chwila="2026-12-12T23:30:00Z",
        kto="sąsiad z lokalu 4",
        tresc="hałas",
        pobyt_id="pobyt",
    )
    refusal = "odpowiedź na opinię z oceną 3 albo niższą i odpowiedź na skargę przed wysłaniem czekają na zgodę"

    letter_to_guest(room, "na-slaba")
    room.run("wskaz_odpowiedz_na_opinie", id="slaba", odpowiedz_wysylka_id="na-slaba")
    room.refused(refusal, "odnotuj_wyslanie", id="na-slaba", wyslano="2026-12-15T10:00:00Z")
    send_with_consent(room, "na-slaba", "2026-12-15")

    prepare_letter(room, "na-skarge", adresat_rodzaj="inny", adresat_nazwa="Sąsiad z lokalu 4")
    room.run("wskaz_odpowiedz_na_skarge", id=1, odpowiedz_wysylka_id="na-skarge")
    room.refused(refusal, "odnotuj_wyslanie", id="na-skarge", wyslano="2026-12-15T10:00:00Z")

    # A good review gets its answer without consent; the answer is named before it goes out.
    book(room, "drugi", "2026-12-20", "2026-12-22", numer_rezerwacji="HM-2")
    room.run(
        "wpisz_opinie_goscia",
        id="dobra",
        pobyt_id="drugi",
        ocena=5,
        opublikowano="2026-12-23T09:00:00Z",
    )
    letter_to_guest(room, "na-dobra")
    send(room, "na-dobra")
    room.refused(
        "odpowiedź na opinię albo skargę wskazuje wysyłkę, zanim wyjdzie",
        "wskaz_odpowiedz_na_opinie",
        id="dobra",
        odpowiedz_wysylka_id="na-dobra",
    )
    letter_to_guest(room, "na-dobra-2")
    room.run("wskaz_odpowiedz_na_opinie", id="dobra", odpowiedz_wysylka_id="na-dobra-2")
    send(room, "na-dobra-2")
    room.refused(
        "odpowiedź wskazuje się raz",
        "wskaz_odpowiedz_na_opinie",
        id="dobra",
        odpowiedz_wysylka_id="na-slaba",
    )


@scenario("Pobyty: ostatnie rozliczenie dopiero, gdy żaden pobyt nie czeka ani nie trwa")
def last_settlement_after_last_stay():
    room = new_room()
    offering_flat(room, koniec=CONTRACT_ENDS)
    established_terms(room)
    draft_offer(room)
    publish(room)
    add_guest(room)
    book(room)
    stage(room, "prowadzenie_lokalu", "2026-11-07T09:00:00Z")
    stage(room, "koniec_umowy", "2027-01-10T09:00:00Z")
    stage(room, "zamkniecie_lokalu", "2027-02-01T09:00:00Z")
    room.run("zmien_stan_oferty_w_kanale", id="oferta-airbnb", stan="zamknieta")
    signed_protocol(room, "zwrot", "zwrot", "2027-02-01T10:00:00Z")

    room.refused(
        "etap ostatnie_rozliczenie przychodzi, gdy żaden pobyt nie jest potwierdzony",
        "zmien_etap_sprawy",
        etap="ostatnie_rozliczenie",
        od="2027-02-02T09:00:00Z",
        zmienil="agent",
    )

    finished_stay(room)
    stage(room, "ostatnie_rozliczenie", "2027-02-02T09:00:00Z")


# Orders and access ---------------------------------------------------------------------------------


@scenario("Zlecenia: spoza rejestru, poza standardem i wymiana zamka mają zgodę człowieka")
def orders_with_consent():
    room = new_room()
    booked_flat(room)
    refusal = "zlecenie spoza rejestru, poza standardem, wymiana zamka i wycena ponad kwotę napraw wskazują"
    outside_register = {
        "spoza_rejestru": 1,
        "cennik_wersja_odnosnik": None,
        "cena_zrodlo": "wycena",
    }

    room.refused(
        refusal, "zlec_prace", **{**zlecenie_defaults(), **outside_register, "id": "spoza"}
    )
    room.refused(
        refusal,
        "zlec_prace",
        **{**zlecenie_defaults(), "id": "poza-standardem", "poza_standardem": 1},
    )
    room.refused(
        refusal, "zlec_prace", **{**zlecenie_defaults(), "id": "zamek", "rodzaj": "wymiana_zamka"}
    )
    room.refused(
        "cena_z_cennika_rejestru",
        "zlec_prace",
        **{**zlecenie_defaults(), "id": "bez-cennika", "cennik_wersja_odnosnik": None},
    )

    # A decision about another order does not cover this one.
    approved_decision(
        room, "decyzja-zamek", "inna", "lokal_krotkoterminowy_zlecenia_wpisy", "zamek"
    )
    room.refused(
        refusal,
        "zlec_prace",
        **{**zlecenie_defaults(), **outside_register, "id": "spoza", "decyzja_id": "decyzja-zamek"},
    )
    order(room, "zamek", rodzaj="wymiana_zamka", decyzja_id="decyzja-zamek")
    approved_decision(
        room, "decyzja-spoza", "inna", "lokal_krotkoterminowy_zlecenia_wpisy", "spoza"
    )
    order(room, "spoza", decyzja_id="decyzja-spoza", **outside_register)
    order(room)


def zlecenie_defaults():
    """The required fields of zlec_prace that `order` fills, for a direct refusal."""
    return {
        "rodzaj": "sprzatanie",
        "przyczyna": "wyjazd_goscia",
        "pobyt_id": "pobyt",
        "wykonawca_odnosnik": "rejestr-wykonawcow/czysty-dom",
        "spoza_rejestru": 0,
        "cennik_wersja_odnosnik": "rejestr-wykonawcow/czysty-dom/cennik-3",
        "poza_standardem": 0,
        "okno_od": "2026-12-13T10:00:00Z",
        "okno_do": "2026-12-13T15:00:00Z",
        "cena": 15_000,
        "cena_zrodlo": "cennik",
        "zlecil": "agent-koordynatora",
        "zlecono": "2026-12-01T09:00:00Z",
    }


@scenario("Zlecenia: wycena ponad kwotę napraw ma zgodę właściciela i człowieka")
def quote_above_repair_amount():
    room = new_room()
    booked_flat(room)
    repair = {
        "rodzaj": "naprawa",
        "przyczyna": "standard_lokalu",
        "pobyt_id": None,
        "cennik_wersja_odnosnik": None,
        "cena_zrodlo": "wycena",
    }

    order(room, "w-kwocie", cena=50_000, **repair)

    approved_decision(
        room, "decyzja-boiler", "inna", "lokal_krotkoterminowy_zlecenia_wpisy", "boiler"
    )
    room.refused(
        "cena z wyceny ponad kwotę napraw wskazuje zgodę zgoda_na_naprawe właściciela",
        "zlec_prace",
        **{
            **zlecenie_defaults(),
            **repair,
            "id": "boiler",
            "cena": 80_000,
            "decyzja_id": "decyzja-boiler",
        },
    )
    add_consent(
        room,
        "zgoda-inna",
        "wlascicielka",
        "zgoda_na_naprawe",
        tresc="Naprawa pralki",
        dotyczy_tabela="lokal_krotkoterminowy_zlecenia_wpisy",
        dotyczy_id="pralka",
    )
    room.refused(
        "cena z wyceny ponad kwotę napraw wskazuje zgodę zgoda_na_naprawe właściciela",
        "zlec_prace",
        **{
            **zlecenie_defaults(),
            **repair,
            "id": "boiler",
            "cena": 80_000,
            "decyzja_id": "decyzja-boiler",
            "zgoda_id": "zgoda-inna",
        },
    )
    add_consent(
        room,
        "zgoda-boiler",
        "wlascicielka",
        "zgoda_na_naprawe",
        tresc="Wymiana bojlera do 800 zł",
        dotyczy_tabela="lokal_krotkoterminowy_zlecenia_wpisy",
        dotyczy_id="boiler",
    )
    room.refused(
        "wycena ponad kwotę napraw wskazują zatwierdzoną decyzję",
        "zlec_prace",
        **{
            **zlecenie_defaults(),
            **repair,
            "id": "boiler",
            "cena": 80_000,
            "zgoda_id": "zgoda-boiler",
        },
    )

    order(
        room, "boiler", cena=80_000, zgoda_id="zgoda-boiler", decyzja_id="decyzja-boiler", **repair
    )


@scenario("Zlecenia: koszt po potwierdzeniu, inny niż cena z osobną zgodą, wpisany raz")
def order_cost():
    room = new_room()
    booked_flat(room)
    order(room)

    room.refused("koszt_po_potwierdzeniu", "wpisz_koszt_zlecenia", id="sprzatanie", koszt=15_000)
    room.refused(
        "potwierdzone_z_raportem",
        "potwierdz_wykonanie_zlecenia",
        id="sprzatanie",
        potwierdzil="agent",
        potwierdzono="2026-12-13T16:00:00Z",
    )
    confirm_order(room)
    room.refused(
        "koszt inny niż cena wskazuje zatwierdzoną decyzję",
        "wpisz_koszt_zlecenia",
        id="sprzatanie",
        koszt=18_000,
    )

    # A decision asked before the work was confirmed is about something else.
    approved_decision(
        room,
        "za-wczesna",
        "inna",
        "lokal_krotkoterminowy_zlecenia_wpisy",
        "sprzatanie",
        zlozono="2026-12-13T15:00:00Z",
        podpisano="2026-12-13T15:30:00Z",
    )
    room.refused(
        "koszt inny niż cena wskazuje zatwierdzoną decyzję",
        "wpisz_koszt_zlecenia",
        id="sprzatanie",
        koszt=18_000,
        koszt_decyzja_id="za-wczesna",
    )
    approved_decision(
        room,
        "koszt",
        "inna",
        "lokal_krotkoterminowy_zlecenia_wpisy",
        "sprzatanie",
        zlozono="2026-12-13T17:00:00Z",
        podpisano="2026-12-13T18:00:00Z",
    )
    room.run("wpisz_koszt_zlecenia", id="sprzatanie", koszt=18_000, koszt_decyzja_id="koszt")

    room.refused(
        "potwierdzenie i koszt zlecenia wpisuje się raz",
        "wpisz_koszt_zlecenia",
        id="sprzatanie",
        koszt=15_000,
    )
    room.refused("zlecenie przechodzi", "zmien_stan_zlecenia", id="sprzatanie", stan="anulowane")


@scenario("Zlecenia: wstrzymane wraca po decyzji, anulowane jest ostateczne, treść się nie zmienia")
def order_states():
    room = new_room()
    booked_flat(room)
    order(room)

    room.run("zmien_stan_zlecenia", id="sprzatanie", stan="wstrzymane")
    room.refused(
        "zlecenie przechodzi",
        "potwierdz_wykonanie_zlecenia",
        id="sprzatanie",
        raport_sciezka="r.pdf",
        raport_skrot="s",
        potwierdzil="agent",
        potwierdzono="2026-12-13T16:00:00Z",
    )
    room.run("zmien_stan_zlecenia", id="sprzatanie", stan="zlecone")
    room.refused_sql(
        "treść zlecenia się nie zmienia", "UPDATE lokal_krotkoterminowy_zlecenia_wpisy SET cena = 1"
    )
    room.refused(
        "przyczyna_wskazuje_pobyt_albo_zgloszenie",
        "zlec_prace",
        **{**zlecenie_defaults(), "id": "bez-pobytu", "pobyt_id": None},
    )

    room.run("zmien_stan_zlecenia", id="sprzatanie", stan="anulowane")
    room.refused(
        "anulowanego zlecenia się nie zmienia",
        "zmien_stan_zlecenia",
        id="sprzatanie",
        stan="zlecone",
    )


def access_fields(**fields):
    return {
        "pobyt_id": "pobyt",
        "rodzaj": "kod",
        "kod": "4821",
        "wazny_od": "2026-12-10T15:00:00Z",
        "wazny_do": "2026-12-13T11:00:00Z",
        "kto": "gość",
        "ustawiono": "2026-12-09T09:00:00Z",
        **fields,
    }


@scenario("Dostępy: kod dla gościa tylko do opłaconego pobytu i w jego dniach (T7)")
def access_for_paid_stay():
    room = new_room()
    booked_flat(room)
    refusal = "dostęp dla pobytu wymaga opłaconej rezerwacji"

    room.refused(refusal, "ustaw_dostep", **access_fields())
    room.run("odnotuj_oplacenie_pobytu", id="pobyt")
    room.refused(refusal, "ustaw_dostep", **access_fields(wazny_od="2026-12-09T15:00:00Z"))
    room.refused(refusal, "ustaw_dostep", **access_fields(wazny_do="2026-12-14T11:00:00Z"))
    room.refused("kod_albo_klucze", "ustaw_dostep", **access_fields(kod=None))
    room.refused("dostep_dla_pobytu_albo_zlecenia", "ustaw_dostep", **access_fields(pobyt_id=None))
    room.run("ustaw_dostep", **access_fields())

    assert room.run("dostepy_do_lokalu")[0]["kod"] == "4821"
    room.refused_sql(
        "dostępu się nie zmienia",
        "UPDATE lokal_krotkoterminowy_zlecenia_dostepy SET wazny_do = '2026-12-20T11:00:00Z'",
    )

    # The stay is cancelled before the code went out: the code is not given.
    room.run("zmien_stan_pobytu", id="pobyt", stan="odwolana")
    room.refused(
        "kod albo klucze wydaje się tylko dla opłaconego pobytu",
        "odnotuj_wydanie_dostepu",
        id=1,
        wydano="2026-12-10T10:00:00Z",
    )
    room.run("odnotuj_odebranie_dostepu", id=1, odebrano="2026-12-09T12:00:00Z")


@scenario("Dostępy: kod i klucze dla wykonawcy tylko w oknie zlecenia; wydanie i odebranie raz")
def access_for_order():
    room = new_room()
    booked_flat(room)
    order(room)
    fields = {
        "pobyt_id": None,
        "zlecenie_id": "sprzatanie",
        "rodzaj": "klucze",
        "kod": None,
        "klucze_liczba": 1,
        "wazny_od": "2026-12-13T10:00:00Z",
        "wazny_do": "2026-12-13T15:00:00Z",
        "kto": "ekipa sprzątająca",
    }
    refusal = "dostęp dla zlecenia mieści się w oknie zlecenia"

    room.refused(
        refusal, "ustaw_dostep", **access_fields(**{**fields, "wazny_do": "2026-12-13T16:00:00Z"})
    )
    room.run("ustaw_dostep", **access_fields(**fields))
    room.refused(
        "klucze_odebrane_po_wydaniu",
        "odnotuj_odebranie_dostepu",
        id=1,
        odebrano="2026-12-13T15:00:00Z",
    )
    room.run("odnotuj_wydanie_dostepu", id=1, wydano="2026-12-13T09:50:00Z")
    room.refused(
        "dostępu się nie zmienia", "odnotuj_wydanie_dostepu", id=1, wydano="2026-12-13T09:55:00Z"
    )
    room.run("odnotuj_odebranie_dostepu", id=1, odebrano="2026-12-13T15:00:00Z")

    room.run("zmien_stan_zlecenia", id="sprzatanie", stan="wstrzymane")
    room.refused(refusal, "ustaw_dostep", **access_fields(**fields))


# Calendar, prices and offers -----------------------------------------------------------------------


@scenario("Kalendarz: noc zarezerwowana leży w dniach pobytu, a dzień wyjazdu jest wolny")
def nights_of_a_stay():
    room = new_room()
    booked_flat(room)
    refusal = "noc zarezerwowana należy do potwierdzonego, trwającego albo zakończonego pobytu"

    room.refused(
        refusal,
        "wpisz_noc",
        data="2026-12-13",
        stan="zarezerwowana",
        pobyt_id="pobyt",
        zmieniono="2026-11-07T10:05:00Z",
    )
    room.refused(
        "stan_wskazuje_pobyt_zlecenie_albo_zgode",
        "wpisz_noc",
        data="2026-12-10",
        stan="wolna",
        pobyt_id="pobyt",
        zmieniono="2026-11-07T10:05:00Z",
    )
    room.run(
        "wpisz_noc",
        data="2026-12-10",
        stan="zarezerwowana",
        pobyt_id="pobyt",
        zmieniono="2026-11-07T10:05:00Z",
    )
    room.run("wpisz_noc", data="2026-12-13", stan="wolna", zmieniono="2026-11-07T10:05:00Z")
    room.refused_sql(
        "data nocy się nie zmienia",
        "UPDATE lokal_krotkoterminowy_kalendarz_noce SET data = '2026-12-14' WHERE data = '2026-12-13'",
    )


@scenario("Kalendarz: tej samej nocy nie sprzeda się dwa razy; druga rezerwacja to kolizja")
def double_booking_is_a_collision():
    room = new_room()
    booked_flat(room)
    room.run(
        "wpisz_noc",
        data="2026-12-11",
        stan="zarezerwowana",
        pobyt_id="pobyt",
        zmieniono="2026-11-07T10:05:00Z",
    )
    draft_offer(room, "oferta-booking", "booking", "wlasciciela")
    publish(room, "oferta-booking")
    book(room, "z-bookingu", "2026-12-11", "2026-12-12", kanal="booking", numer_rezerwacji="BK-1")

    # The second channel's guest cannot take the night: the row exists and is taken.
    room.refused(
        "wiersza teczki nie zastępuje się nowym wpisem",
        "wpisz_noc",
        data="2026-12-11",
        stan="zarezerwowana",
        pobyt_id="z-bookingu",
        zmieniono="2026-11-07T11:00:00Z",
    )
    room.refused(
        "zajęta noc najpierw się zwalnia",
        "zmien_noc",
        data="2026-12-11",
        stan="zarezerwowana",
        pobyt_id="z-bookingu",
        zmieniono="2026-11-07T11:00:00Z",
    )
    room.refused(
        "noc pobytu zwalnia się, gdy pobyt jest odwołany",
        "zmien_noc",
        data="2026-12-11",
        stan="wolna",
        zmieniono="2026-11-07T11:00:00Z",
    )
    room.run("zmien_stan_pobytu", id="z-bookingu", stan="kolizja")

    # The human keeps the second stay: the first is cancelled, its night freed and taken.
    room.run("zmien_stan_pobytu", id="pobyt", stan="odwolana")
    room.run("zmien_noc", data="2026-12-11", stan="wolna", zmieniono="2026-11-07T12:00:00Z")
    room.refused(
        "noc zarezerwowana należy do potwierdzonego",
        "zmien_noc",
        data="2026-12-11",
        stan="zarezerwowana",
        pobyt_id="z-bookingu",
        zmieniono="2026-11-07T12:05:00Z",
    )
    room.run("zmien_stan_pobytu", id="z-bookingu", stan="potwierdzona")
    room.run(
        "zmien_noc",
        data="2026-12-11",
        stan="zarezerwowana",
        pobyt_id="z-bookingu",
        zmieniono="2026-11-07T12:05:00Z",
    )


@scenario("Kalendarz: noc na prace w oknie zlecenia, noc właściciela zgłoszona na czas")
def blocked_nights():
    room = new_room()
    booked_flat(room)
    order(
        room,
        "naprawa",
        przyczyna="standard_lokalu",
        pobyt_id=None,
        rodzaj="naprawa",
        okno_od="2026-12-20T08:00:00Z",
        okno_do="2026-12-20T18:00:00Z",
    )

    room.refused(
        "noc na prace leży w oknie zlecenia",
        "wpisz_noc",
        data="2026-12-21",
        stan="prace",
        zlecenie_id="naprawa",
        zmieniono="2026-12-01T09:00:00Z",
    )
    room.run(
        "wpisz_noc",
        data="2026-12-20",
        stan="prace",
        zlecenie_id="naprawa",
        zmieniono="2026-12-01T09:00:00Z",
    )

    # Terms say 14 days ahead: a request of 15 December is late for the 24th.
    add_consent(
        room,
        "spoznione",
        "wlascicielka",
        "noce_wlasciciela",
        tresc="Wigilia",
        zlozono="2026-12-15T09:00:00Z",
    )
    add_consent(
        room,
        "na-czas",
        "wlascicielka",
        "noce_wlasciciela",
        tresc="Wigilia",
        zlozono="2026-12-01T09:00:00Z",
    )
    add_consent(room, "inna-zgoda", "wlascicielka", "zgoda_na_naprawe")
    refusal = "noc właściciela wskazuje zgodę noce_wlasciciela złożoną z wyprzedzeniem"
    room.refused(
        refusal,
        "wpisz_noc",
        data="2026-12-24",
        stan="wlasciciel",
        zgoda_id="spoznione",
        zmieniono="2026-12-15T10:00:00Z",
    )
    room.refused(
        refusal,
        "wpisz_noc",
        data="2026-12-24",
        stan="wlasciciel",
        zgoda_id="inna-zgoda",
        zmieniono="2026-12-15T10:00:00Z",
    )
    room.run(
        "wpisz_noc",
        data="2026-12-24",
        stan="wlasciciel",
        zgoda_id="na-czas",
        zmieniono="2026-12-01T10:00:00Z",
    )

    # A blocked night is freed before anything else takes it.
    room.refused(
        "zajęta noc najpierw się zwalnia",
        "zmien_noc",
        data="2026-12-20",
        stan="wlasciciel",
        zgoda_id="na-czas",
        zmieniono="2026-12-02T10:00:00Z",
    )
    room.run("zmien_noc", data="2026-12-20", stan="wolna", zmieniono="2026-12-02T10:00:00Z")


@scenario("Kalendarz: noc po dacie końca umowy nie jest wolna, a stan po_koncu ma tylko taka noc")
def nights_after_contract_end():
    room = new_room()
    offering_flat(room, koniec=CONTRACT_ENDS)
    refusal = "noc po dacie końca umowy nie jest wolna"

    room.refused(
        refusal, "wpisz_noc", data="2027-02-01", stan="wolna", zmieniono="2026-11-07T09:00:00Z"
    )
    room.refused(
        refusal, "wpisz_noc", data="2027-01-31", stan="po_koncu", zmieniono="2026-11-07T09:00:00Z"
    )
    room.run("wpisz_noc", data="2027-01-31", stan="wolna", zmieniono="2026-11-07T09:00:00Z")
    room.run("wpisz_noc", data="2027-02-01", stan="po_koncu", zmieniono="2026-11-07T09:00:00Z")
    room.refused(
        refusal, "zmien_noc", data="2027-02-01", stan="wolna", zmieniono="2026-11-08T09:00:00Z"
    )


@scenario("Kalendarz: cena poza widełkami ma zgodę człowieka, w kanale z umowy albo z ofertą")
def night_prices():
    room = new_room()
    managed_flat(room)
    price = {"data": "2026-12-10", "kanal": "airbnb", "ustawiono": "2026-11-06T11:00:00Z"}
    refusal = "cena poza widełkami z bieżących warunków umowy wskazuje zatwierdzoną decyzję"

    room.run("ustaw_cene_nocy", id="w-widelkach", cena=100_000, **price)
    room.refused(refusal, "ustaw_cene_nocy", id="za-wysoka", cena=200_000, **price)
    room.refused(
        refusal,
        "ustaw_cene_nocy",
        id="bez-widelek",
        cena=100_000,
        **{**price, "data": "2027-06-10"},
    )
    approved_decision(
        room, "sylwester", "inna", "lokal_krotkoterminowy_kalendarz_ceny", "za-wysoka"
    )
    room.run(
        "ustaw_cene_nocy",
        id="za-wysoka",
        cena=200_000,
        decyzja_id="sylwester",
        **{**price, "ustawiono": "2026-11-06T12:00:00Z"},
    )
    room.refused(
        "cenę ustawia się w kanale z umowy albo w kanale, w którym lokal ma ofertę",
        "ustaw_cene_nocy",
        id="vrbo",
        cena=100_000,
        **{**price, "kanal": "vrbo"},
    )

    assert [row["id"] for row in room.run("ceny_nocy")] == ["za-wysoka"]
    room.refused_sql(
        "ceny się nie zmienia", "UPDATE lokal_krotkoterminowy_kalendarz_ceny SET cena = 1"
    )
    room.run(
        "wpisz_wyliczenie_cen",
        id="w",
        chwila="2026-11-06T11:00:00Z",
        dane_wejsciowe="sezon",
        zrodla="ceny podobnych lokali z 2026-11-06",
        sciezka="w.json",
        skrot="sha256-w",
    )
    room.refused_sql(
        "wyliczenia się nie zmienia",
        "UPDATE lokal_krotkoterminowy_kalendarz_wyliczenia SET zrodla = 'inne'",
    )


@scenario(
    "Kalendarz: publikacja w kanale z umowy z tym samym kontem albo za zgodą właściciela i człowieka"
)
def publication_channel():
    room = new_room()
    add_owner(room)
    add_flat(room)
    sign_management_contract(room)
    established_terms(room)
    refusal = "ofertę publikuje się w kanale z bieżących warunków umowy z tym samym kontem"

    draft_offer(room, "booking-na-biuro", "booking", "biura")
    room.refused(
        refusal,
        "opublikuj_oferte_w_kanale",
        id="booking-na-biuro",
        numer_oferty="1",
        opublikowano="2026-11-06T10:00:00Z",
    )

    add_consent(room, "vrbo", "wlascicielka", "zgoda_na_kanal", tresc="Zgadzam się na Vrbo.")
    draft_offer(room, "oferta-vrbo", "vrbo", zgoda_id="vrbo")
    room.refused(
        refusal,
        "opublikuj_oferte_w_kanale",
        id="oferta-vrbo",
        numer_oferty="1",
        opublikowano="2026-11-06T10:00:00Z",
    )
    approved_decision(
        room, "nowy-kanal", "inna", "lokal_krotkoterminowy_kalendarz_kanaly", "oferta-vrbo"
    )
    publish(room, "oferta-vrbo", nowy_kanal_decyzja_id="nowy-kanal")

    draft_offer(room)
    room.refused(
        "nie wcześniej niż praca_od",
        "zmien_stan_oferty_w_kanale",
        id="oferta-airbnb",
        stan="aktywna",
    )
    publish(room)
    room.refused(
        "oferta w kanale przechodzi",
        "zmien_stan_oferty_w_kanale",
        id="oferta-airbnb",
        stan="projekt",
    )
    room.run("zmien_stan_oferty_w_kanale", id="oferta-airbnb", stan="zawieszona")
    room.run("zmien_stan_oferty_w_kanale", id="oferta-airbnb", stan="zamknieta")
    room.refused(
        "zamkniętej oferty się nie zmienia",
        "wskaz_rachunek_oferty_w_kanale",
        id="oferta-airbnb",
        rachunek_id=None,
    )
    room.refused_sql(
        "kanał, konto i publikację oferty wpisuje się raz",
        "UPDATE lokal_krotkoterminowy_kalendarz_kanaly SET konto = 'wlasciciela' WHERE id = 'oferta-vrbo'",
    )


@scenario("Kalendarz: kanał wypłaca na rachunek dopuszczony po sprawdzeniu Kontrolera (R4)")
def channel_account_admitted():
    room = new_room()
    managed_flat(room)
    refusal = "rachunek do wypłat w kanale jest dopuszczony decyzją, niezastąpiony i ma cel wyplata_w_kanale"
    add_account(room, "niesprawdzony", "wlascicielka", cel="wyplata_w_kanale")
    owner_account(room, "na-wyplaty", "wyplata_dla_wlasciciela")
    owner_account(room, "kanalu", "wyplata_w_kanale")

    room.refused(
        refusal, "wskaz_rachunek_oferty_w_kanale", id="oferta-airbnb", rachunek_id="niesprawdzony"
    )
    room.refused(
        refusal, "wskaz_rachunek_oferty_w_kanale", id="oferta-airbnb", rachunek_id="na-wyplaty"
    )
    room.refused(
        refusal,
        "przygotuj_oferte_w_kanale",
        id="druga",
        kanal="booking",
        konto="wlasciciela",
        nazwa_konta="Właścicielka",
        rachunek_id="niesprawdzony",
    )
    room.run("wskaz_rachunek_oferty_w_kanale", id="oferta-airbnb", rachunek_id="kanalu")


@scenario("Kalendarz: ostatnie rozliczenie po zamknięciu ofert we wszystkich kanałach")
def last_settlement_after_offers_closed():
    room = new_room()
    offering_flat(room, koniec=CONTRACT_ENDS)
    established_terms(room)
    draft_offer(room)
    publish(room)
    stage(room, "prowadzenie_lokalu", "2026-11-07T09:00:00Z")
    stage(room, "koniec_umowy", "2027-01-10T09:00:00Z")
    stage(room, "zamkniecie_lokalu", "2027-02-01T09:00:00Z")
    signed_protocol(room, "zwrot", "zwrot", "2027-02-01T10:00:00Z")
    room.run("zmien_stan_oferty_w_kanale", id="oferta-airbnb", stan="zawieszona")

    room.refused(
        "etap ostatnie_rozliczenie przychodzi po zamknięciu ofert",
        "zmien_etap_sprawy",
        etap="ostatnie_rozliczenie",
        od="2027-02-02T09:00:00Z",
        zmienil="agent",
    )

    room.run("zmien_stan_oferty_w_kanale", id="oferta-airbnb", stan="zamknieta")
    stage(room, "ostatnie_rozliczenie", "2027-02-02T09:00:00Z")


# Statements ----------------------------------------------------------------------------------------


def settled_flat(room):
    """A booked flat whose stay ended and was settled on 15 December, with a confirmed cleaning."""
    booked_flat(room)
    finished_stay(room)
    settle_stay(room)
    order(room)
    confirm_order(room)
    room.run("wpisz_koszt_zlecenia", id="sprzatanie", koszt=15_000)


@scenario("Zestawienia: pozycja ma źródło i znak, przychód z rozliczonego pobytu z jego kwotą")
def statement_lines():
    room = new_room()
    settled_flat(room)
    draft_statement(room, "listopad", "2026-11")
    draft_statement(room)

    room.refused(
        "pozycja_ma_zrodlo",
        "dopisz_pozycje_zestawienia",
        zestawienie_id="zestawienie",
        rodzaj="oplata_lokalu",
        kwota=-5_000,
    )
    room.refused(
        "znak_kwoty_wedlug_rodzaju",
        "dopisz_pozycje_zestawienia",
        zestawienie_id="zestawienie",
        rodzaj="prowizja_biura",
        kwota=1_000,
        pobyt_id="pobyt",
    )
    refusal = (
        "przychód i prowizja kanału wskazują pobyt rozliczony najpóźniej w miesiącu zestawienia"
    )
    room.refused(
        refusal,
        "dopisz_pozycje_zestawienia",
        zestawienie_id="listopad",
        rodzaj="przychod",
        kwota=300_000,
        pobyt_id="pobyt",
    )
    room.refused(
        refusal,
        "dopisz_pozycje_zestawienia",
        zestawienie_id="zestawienie",
        rodzaj="przychod",
        kwota=290_000,
        pobyt_id="pobyt",
    )
    room.refused(
        refusal,
        "dopisz_pozycje_zestawienia",
        zestawienie_id="zestawienie",
        rodzaj="prowizja_kanalu",
        kwota=-40_000,
        pobyt_id="pobyt",
    )

    add_line(room, "przychod", 300_000, pobyt_id="pobyt")
    add_line(room, "prowizja_kanalu", -45_000, pobyt_id="pobyt")
    room.refused(
        "przychód, prowizja kanału, koszt zlecenia i pieniądze pobytu trafiają do jednego zestawienia",
        "dopisz_pozycje_zestawienia",
        zestawienie_id="listopad",
        rodzaj="przychod",
        kwota=300_000,
        pobyt_id="pobyt",
    )
    room.refused_sql(
        "pozycji zestawienia się nie zmienia",
        "UPDATE lokal_krotkoterminowy_zestawienia_pozycje SET kwota = 1",
    )


@scenario(
    "Zestawienia: koszt z potwierdzonego zlecenia, zwrot z pieniędzy pobytu, korekta tylko w korekcie"
)
def statement_costs_and_refunds():
    room = new_room()
    settled_flat(room)
    order(room, "otwarte", okno_od="2026-12-14T10:00:00Z", okno_do="2026-12-14T15:00:00Z")
    room.run(
        "wpisz_pieniadze_pobytu",
        id="zwrot",
        pobyt_id="pobyt",
        rodzaj="zwrot",
        wedlug_zasad_kanalu=1,
        kwota=10_000,
        chwila="2026-12-12T09:00:00Z",
    )
    add_received_document(room, "pismo")
    draft_statement(room)

    room.refused(
        "koszt wskazuje potwierdzone zlecenie z wpisanym kosztem",
        "dopisz_pozycje_zestawienia",
        zestawienie_id="zestawienie",
        rodzaj="koszt",
        kwota=-15_000,
        zlecenie_id="otwarte",
    )
    room.refused(
        "koszt wskazuje potwierdzone zlecenie z wpisanym kosztem",
        "dopisz_pozycje_zestawienia",
        zestawienie_id="zestawienie",
        rodzaj="koszt",
        kwota=-14_000,
        zlecenie_id="sprzatanie",
    )
    add_line(room, "koszt", -15_000, zlecenie_id="sprzatanie")
    room.refused(
        "przychód, prowizja kanału, koszt zlecenia i pieniądze pobytu trafiają do jednego zestawienia",
        "dopisz_pozycje_zestawienia",
        zestawienie_id="zestawienie",
        rodzaj="koszt",
        kwota=-15_000,
        zlecenie_id="sprzatanie",
    )

    room.refused(
        "zwrot wskazuje zwrot albo rekompensatę",
        "dopisz_pozycje_zestawienia",
        zestawienie_id="zestawienie",
        rodzaj="odzysk",
        kwota=10_000,
        pieniadze_id="zwrot",
    )
    add_line(room, "zwrot", -10_000, pieniadze_id="zwrot")
    room.refused(
        "pozycja korekty leży w zestawieniu, które wskazuje korygowane zestawienie",
        "dopisz_pozycje_zestawienia",
        zestawienie_id="zestawienie",
        rodzaj="korekta",
        kwota=5_000,
        dokument_id="pismo",
    )


@scenario(
    "Zestawienia: kwota do wypłaty to suma pozycji; gotowe zestawienie zamraża plik, kwoty i pozycje"
)
def statement_ready():
    room = new_room()
    settled_flat(room)
    draft_statement(room)
    add_line(room, "przychod", 300_000, pobyt_id="pobyt")
    add_line(room, "koszt", -15_000, zlecenie_id="sprzatanie")

    room.refused(
        "kwota do wypłaty to suma pozycji zestawienia",
        "zamknij_zestawienie",
        id="zestawienie",
        sciezka="z.pdf",
        skrot="s",
        kwota_do_wyplaty=300_000,
        rezerwa_po=0,
    )
    close_statement(room, 285_000)

    room.refused(
        "pozycje dopisuje się tylko do projektu zestawienia",
        "dopisz_pozycje_zestawienia",
        zestawienie_id="zestawienie",
        rodzaj="prowizja_kanalu",
        kwota=-45_000,
        pobyt_id="pobyt",
    )
    room.refused_sql(
        "plik i kwoty zestawienia zmieniają się tylko w projekcie",
        "UPDATE lokal_krotkoterminowy_zestawienia_wpisy SET kwota_do_wyplaty = 1",
    )
    room.refused(
        "zestawienie jest wysłane, gdy jego wysyłka wyszła",
        "odnotuj_wyslanie_zestawienia",
        id="zestawienie",
    )

    # Back to draft for a missing line, then ready again with the new sum.
    room.run("cofnij_zestawienie_do_projektu", id="zestawienie")
    add_line(room, "prowizja_kanalu", -45_000, pobyt_id="pobyt")
    close_statement(room, 240_000)
    room.refused_sql(
        "wiersza teczki nie zastępuje się nowym wpisem",
        "INSERT INTO lokal_krotkoterminowy_zestawienia_wpisy (id, miesiac, utworzono, stan) VALUES ('drugie', '2026-12', '2027-01-05', 'projekt')",
    )


@scenario("Zestawienia: wysłanie za zgodą człowieka; wysłanego zestawienia się nie zmienia")
def statement_sent_with_consent():
    room = new_room()
    settled_flat(room)
    draft_statement(room)
    add_line(room, "przychod", 300_000, pobyt_id="pobyt")
    letter_to_owner(room, "wysylka")

    room.refused(
        "gotowe zestawienie wskazuje wysyłkę raz, zanim wyjdzie",
        "wskaz_wysylke_zestawienia",
        id="zestawienie",
        wysylka_id="wysylka",
    )
    close_statement(room, 300_000)
    room.run("wskaz_wysylke_zestawienia", id="zestawienie", wysylka_id="wysylka")
    room.refused(
        "zestawienie dla właściciela przed wysłaniem czeka na zgodę",
        "odnotuj_wyslanie",
        id="wysylka",
        wyslano="2027-01-06T11:00:00Z",
    )
    room.refused(
        "zestawienie jest wysłane, gdy jego wysyłka wyszła",
        "odnotuj_wyslanie_zestawienia",
        id="zestawienie",
    )
    room.refused(
        "do projektu wraca tylko bez wysyłki albo z anulowaną",
        "cofnij_zestawienie_do_projektu",
        id="zestawienie",
    )

    send_with_consent(room, "wysylka", "2027-01-06")
    room.run("odnotuj_wyslanie_zestawienia", id="zestawienie")
    room.refused_sql(
        "wysłanego zestawienia się nie zmienia",
        "UPDATE lokal_krotkoterminowy_zestawienia_wpisy SET rezerwa_po = 0",
    )


@scenario("Zestawienia: korekta wskazuje wcześniejsze wysłane zestawienie i zgodę człowieka")
def statement_correction():
    room = new_room()
    settled_flat(room)
    draft_statement(room)
    add_line(room, "przychod", 300_000, pobyt_id="pobyt")
    close_statement(room, 300_000)
    refusal = "korekta wskazuje wcześniejsze wysłane zestawienie i zatwierdzoną decyzję inna"
    approved_decision(room, "korekta", "inna", "lokal_krotkoterminowy_zestawienia_wpisy", "styczen")

    room.refused(
        refusal,
        "przygotuj_zestawienie",
        id="styczen",
        miesiac="2027-01",
        utworzono="2027-02-05T09:00:00Z",
        korekta_do_id="zestawienie",
        korekta_decyzja_id="korekta",
    )
    send_statement(room)
    room.refused(
        refusal,
        "przygotuj_zestawienie",
        id="styczen",
        miesiac="2027-01",
        utworzono="2027-02-05T09:00:00Z",
        korekta_do_id="zestawienie",
    )
    room.refused(
        refusal,
        "przygotuj_zestawienie",
        id="inne",
        miesiac="2027-01",
        utworzono="2027-02-05T09:00:00Z",
        korekta_do_id="zestawienie",
        korekta_decyzja_id="korekta",
    )
    room.run(
        "przygotuj_zestawienie",
        id="styczen",
        miesiac="2027-01",
        utworzono="2027-02-05T09:00:00Z",
        korekta_do_id="zestawienie",
        korekta_decyzja_id="korekta",
    )
    add_received_document(room, "pismo")
    add_line(room, "korekta", -5_000, "styczen", dokument_id="pismo")


@scenario("Zestawienia: wypłata wskazuje wysłane zestawienie z jego kwotą, jedna na zestawienie")
def payout_from_statement():
    room = new_room()
    settled_flat(room)
    owner_account(room, "rachunek-wyplat", "wyplata_dla_wlasciciela")
    draft_statement(room)
    add_line(room, "przychod", 300_000, pobyt_id="pobyt")
    close_statement(room, 300_000)
    refusal = "wypłata wskazuje wysłane zestawienie i ma jego kwotę do wypłaty"

    room.refused(refusal, "przygotuj_wyplate", **payout_fields(kwota=300_000))
    send_statement(room)
    room.refused(refusal, "przygotuj_wyplate", **payout_fields(kwota=299_000))
    room.refused(
        refusal, "przygotuj_wyplate", **payout_fields(kwota=300_000, podstawa_tabela="najem_kaucje")
    )

    pay_owner(room, "wyplata", "zestawienie", 300_000, "2027-01-08")
    room.refused(
        "zestawienie albo zwrot ma jedną wypłatę",
        "przygotuj_wyplate",
        **payout_fields(id="druga", kwota=300_000),
    )


def payout_fields(**fields):
    return {
        "id": "wyplata",
        "podstawa_tabela": "lokal_krotkoterminowy_zestawienia_wpisy",
        "podstawa_id": "zestawienie",
        "wyliczenie_sciezka": "wyliczenia/wyplata.json",
        "wyliczenie_skrot": "sha256-wyplata",
        "odbiorca_uczestnik_id": "wlascicielka",
        "rachunek_id": "rachunek-wyplat",
        "utworzono": "2027-01-08T09:00:00Z",
        **fields,
    }


@scenario(
    "Zestawienia: po zarządzaniu wszystko wysłane, a każdy rozliczony pobyt i koszt w zestawieniu"
)
def management_ends_with_everything_settled():
    room = new_room()
    offering_flat(room, koniec=CONTRACT_ENDS)
    established_terms(room)
    draft_offer(room)
    publish(room)
    add_guest(room)
    book(room)
    stage(room, "prowadzenie_lokalu", "2026-11-07T09:00:00Z")
    finished_stay(room)
    settle_stay(room)
    order(room)
    confirm_order(room)
    room.run("wpisz_koszt_zlecenia", id="sprzatanie", koszt=15_000)
    stage(room, "koniec_umowy", "2027-01-10T09:00:00Z")
    stage(room, "zamkniecie_lokalu", "2027-02-01T09:00:00Z")
    room.run("zmien_stan_oferty_w_kanale", id="oferta-airbnb", stan="zamknieta")
    signed_protocol(room, "zwrot", "zwrot", "2027-02-01T10:00:00Z")
    stage(room, "ostatnie_rozliczenie", "2027-02-02T09:00:00Z")
    refusal = "etap po_zarzadzaniu wymaga wysłanych zestawień"

    draft_statement(room)
    add_line(room, "przychod", 300_000, pobyt_id="pobyt")
    room.refused(
        refusal,
        "zmien_etap_sprawy",
        etap="po_zarzadzaniu",
        od="2027-02-05T09:00:00Z",
        zmienil="agent",
    )
    close_statement(room, 300_000)
    send_statement(room)
    room.refused(
        refusal,
        "zmien_etap_sprawy",
        etap="po_zarzadzaniu",
        od="2027-02-05T09:00:00Z",
        zmienil="agent",
    )

    draft_statement(room, "styczen", "2027-01")
    add_line(room, "koszt", -15_000, "styczen", zlecenie_id="sprzatanie")
    add_line(room, "rezerwa", 15_000, "styczen", warunki_id="warunki")
    close_statement(room, 0, "styczen", rezerwa_po=35_000)
    send_statement(room, "styczen", "2027-02-03")
    stage(room, "po_zarzadzaniu", "2027-02-05T09:00:00Z")


# R9 and replacement --------------------------------------------------------------------------------

# The table of a case type, its key and the columns of its UNIQUE keys other than the key.
TYPE_TABLES = {
    "lokal_krotkoterminowy_umowa_warunki": ("id", []),
    "lokal_krotkoterminowy_umowa_kanaly": ("id", ["warunki_id, kanal"]),
    "lokal_krotkoterminowy_umowa_widelki": ("id", []),
    "lokal_krotkoterminowy_lokal_dane": ("id", []),
    "lokal_krotkoterminowy_lokal_regulaminy": ("id", []),
    "lokal_krotkoterminowy_lokal_wyposazenie": ("id", []),
    "lokal_krotkoterminowy_lokal_przeglady": ("id", []),
    "lokal_krotkoterminowy_lokal_uchwaly": ("id", []),
    "lokal_krotkoterminowy_pobyty_wpisy": ("id", ["kanal, numer_rezerwacji"]),
    "lokal_krotkoterminowy_pobyty_zmiany": ("id", []),
    "lokal_krotkoterminowy_pobyty_zgloszenia": ("id", []),
    "lokal_krotkoterminowy_pobyty_wyjatki": ("id", []),
    "lokal_krotkoterminowy_pobyty_opinie": ("id", ["pobyt_id"]),
    "lokal_krotkoterminowy_pobyty_pieniadze": ("id", []),
    "lokal_krotkoterminowy_pobyty_skargi": ("id", []),
    "lokal_krotkoterminowy_zlecenia_wpisy": ("id", []),
    "lokal_krotkoterminowy_zlecenia_dostepy": ("id", []),
    "lokal_krotkoterminowy_kalendarz_noce": ("data", []),
    "lokal_krotkoterminowy_kalendarz_wyliczenia": ("id", []),
    "lokal_krotkoterminowy_kalendarz_ceny": ("id", []),
    "lokal_krotkoterminowy_kalendarz_kanaly": ("id", []),
    "lokal_krotkoterminowy_zestawienia_wpisy": ("id", ["miesiac"]),
    "lokal_krotkoterminowy_zestawienia_pozycje": ("id", []),
}


def type_tables(room):
    return sorted(
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'lokal\\_krotkoterminowy\\_%' ESCAPE '\\'"
        )
    )


@scenario("R9: wierszy typu lokalu się nie usuwa")
def no_deletion():
    room = new_room()
    full_case(room)

    assert type_tables(room) == sorted(TYPE_TABLES), type_tables(room)
    for table in TYPE_TABLES:
        assert room.value(f"SELECT count(*) FROM {table}") > 0, f"{table} has no row to delete"
        room.refused_sql(
            "wierszy teczki się nie usuwa (R9)",
            f"DELETE FROM {table} WHERE rowid = (SELECT min(rowid) FROM {table})",
        )


@scenario("Wiersza typu lokalu nie zastępuje INSERT OR REPLACE ani upsert")
def no_replacement():
    room = new_room()
    full_case(room)
    refusal = "wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE"

    for table, (key, unique_keys) in TYPE_TABLES.items():
        assert room.value(f"SELECT count(*) FROM {table}") > 0, f"{table} has no row to replace"

        room.refused_sql(refusal, f"INSERT OR REPLACE {first_row_copy(room, table)}")
        room.refused_sql(
            refusal,
            f"INSERT {first_row_copy(room, table)} ON CONFLICT ({key}) DO UPDATE SET {key} = excluded.{key}",
        )
        for columns in unique_keys:
            room.refused_sql(
                refusal,
                f"INSERT OR REPLACE {first_row_copy(room, table, replace_unique_key=True)}",
            )
            room.refused_sql(
                refusal,
                f"INSERT {first_row_copy(room, table, replace_unique_key=True)} "
                f"ON CONFLICT ({columns}) DO UPDATE SET {key} = excluded.{key}",
            )

    # The lines of a statement have partial unique keys: a second revenue line of the same stay.
    room.refused_sql(
        "przychód, prowizja kanału, koszt zlecenia i pieniądze pobytu trafiają do jednego zestawienia",
        "INSERT OR REPLACE INTO lokal_krotkoterminowy_zestawienia_pozycje "
        "(zestawienie_id, rodzaj, kwota, pobyt_id) VALUES ('zestawienie', 'przychod', 300000, 'pobyt')",
    )
