"""Scenarios of the core of every case room (spec/cases/sprawa).

Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
allows and a write it refuses. The first scenario follows a whole rental management case, from
intake to the return of the deposit and the closing of the case, as the profiles describe it.

This module also holds the building blocks of a case, which the other core_* modules import.
"""

from pathlib import Path

from room import Room

CORE = Path(__file__).resolve().parents[1] / "spec" / "cases" / "sprawa"

SCENARIOS = []


def scenario(title):
    def register(function):
        SCENARIOS.append((title, function))

        return function

    return register


def new_room():
    return Room(CORE)


# Building blocks of a case -------------------------------------------------------------------------


def signature(podpisano):
    """The signature of the human who decides; the app verifies it, the database only requires it."""
    return {
        "podpisal": "tozsamosc-posrednika",
        "podpis": "podpis-kluczem-xkeys",
        "podpisano": podpisano,
    }


def open_case(room, typ_pokoju="najem"):
    room.run(
        "zaloz_metryke",
        oznaczenie="Najem 2026/017, ul. Polna 7 m. 12",
        typ_pokoju=typ_pokoju,
        przyjal="tozsamosc-posrednika",
        przyjeto="2026-11-02T09:00:00Z",
    )


def ask_for_decision(room, decision_id, rodzaj, dotyczy_tabela=None, dotyczy_id=None, **fields):
    room.run(
        "popros_o_decyzje",
        **{
            "id": decision_id,
            "rodzaj": rodzaj,
            "prosi": "tozsamosc-agenta",
            "prosi_stanowisko": "opiekun_najmu",
            "decyduje": "tozsamosc-posrednika",
            "zlozono": "2026-11-03T10:00:00Z",
            "dotyczy_tabela": dotyczy_tabela,
            "dotyczy_id": dotyczy_id,
            **fields,
        },
    )


def approve(room, decision_id, podpisano="2026-11-03T12:00:00Z", rozstrzygniecie="zgoda"):
    room.run(
        "rozstrzygnij_decyzje",
        id=decision_id,
        rozstrzygniecie=rozstrzygniecie,
        **signature(podpisano),
    )


def approved_decision(
    room,
    decision_id,
    rodzaj,
    dotyczy_tabela=None,
    dotyczy_id=None,
    zlozono="2026-11-03T10:00:00Z",
    podpisano="2026-11-03T12:00:00Z",
):
    ask_for_decision(room, decision_id, rodzaj, dotyczy_tabela, dotyczy_id, zlozono=zlozono)
    approve(room, decision_id, podpisano)


def add_person(room, person_id, **fields):
    room.run(
        "wpisz_uczestnika",
        **{
            "id": person_id,
            "rodzaj_podmiotu": "osoba_fizyczna",
            "imiona": "Anna",
            "nazwisko": "Kowalska",
            "pesel_nie_nadano": 0,
            "konsument": 0,
            "ceidg": 0,
            "klient_biura": 0,
            "dane_od": "od_osoby",
            "dane_pozyskano": "2026-11-02T09:30:00Z",
            **fields,
        },
    )


def add_company(room, company_id):
    room.run(
        "wpisz_uczestnika",
        id=company_id,
        rodzaj_podmiotu="osoba_prawna",
        nazwa="Polna Development sp. z o.o.",
        forma_prawna="spółka z ograniczoną odpowiedzialnością",
        krs="0000123456",
        nip="5250000000",
        pesel_nie_nadano=0,
        konsument=0,
        ceidg=0,
        klient_biura=0,
        dane_od="z_innego_zrodla",
        dane_pozyskano="2026-11-02T09:30:00Z",
    )


def add_flat(room, flat_id="lokal", **fields):
    room.run(
        "wpisz_nieruchomosc",
        **{
            "id": flat_id,
            "rodzaj": "lokal_mieszkalny",
            "prawo": "odrebna_wlasnosc_lokalu",
            "miejscowosc": "Wrocław",
            "gmina": "Wrocław",
            "ulica": "Polna",
            "numer_budynku": "7",
            "numer_lokalu": "12",
            "numer_kw": "WR1K/00012345/6",
            "kw_brak": 0,
            "powierzchnia_uzytkowa": 4820,
            "liczba_pokoi": 2,
            "pietro": 3,
            **fields,
        },
    )


def add_received_document(room, document_id, rodzaj="inny", **fields):
    room.run(
        "wpisz_otrzymany_dokument",
        **{
            "id": document_id,
            "rodzaj": rodzaj,
            "nazwa": f"Dokument {document_id}",
            "pochodzenie": "od klientki",
            "kanal": "email",
            "postac": "skan",
            "sciezka": f"dokumenty/{document_id}.pdf",
            "skrot": f"sha256-{document_id}",
            **fields,
        },
    )


def add_inflow(room, inflow_id, nadawca_uczestnik_id=None, **fields):
    room.run(
        "wpisz_wplyw",
        **{
            "id": inflow_id,
            "nadawca": "Anna Kowalska <anna@example.pl>",
            "nadawca_uczestnik_id": nadawca_uczestnik_id,
            "kanal": "email",
            "wplynelo": "2026-11-03T08:00:00Z",
            "sciezka": f"wplywy/{inflow_id}.eml",
            "skrot": f"sha256-{inflow_id}",
            "niezgodny_kontakt": 0,
            **fields,
        },
    )


def version_of(room, person_id):
    return room.value("SELECT wersja FROM sprawa_uczestnicy_wpisy WHERE id = :id", id=person_id)


def power_version_of(room, power_id):
    return room.value("SELECT wersja FROM sprawa_umocowania_wpisy WHERE id = :id", id=power_id)


def check(room, check_id, rodzaj, sprawdzono="2026-11-03T09:00:00Z", wynik="zgodne", **target):
    """Kontroler's check of a row in its current version."""
    if rodzaj in ("tozsamosc", "tytul"):
        wersja_wiersza = version_of(room, target["uczestnik_id"])
    elif rodzaj == "umocowanie":
        wersja_wiersza = power_version_of(room, target["umocowanie_id"])
    else:
        wersja_wiersza = None

    room.run(
        "wpisz_sprawdzenie",
        id=check_id,
        rodzaj=rodzaj,
        wersja_wiersza=wersja_wiersza,
        zrodlo="odpis z centralnej bazy ksiąg wieczystych",
        zrodlo_data=sprawdzono[:10],
        wynik=wynik,
        opis=None if wynik == "zgodne" else "brak odpisu z dnia sprawdzenia",
        sprawdzil="tozsamosc-kontrolera",
        sprawdzono=sprawdzono,
        **target,
    )


def add_account(room, account_id, person_id, numer="PL61109010140000071219812874", **fields):
    add_received_document(room, f"zrodlo-{account_id}", "pismo")
    room.run(
        "wpisz_rachunek",
        **{
            "id": account_id,
            "uczestnik_id": person_id,
            "numer": numer,
            "posiadacz": "Jan Nowak",
            "cel": "zwrot_kaucji",
            "zrodlo_dokument_id": f"zrodlo-{account_id}",
            "wpisano": "2026-11-03T08:00:00Z",
            **fields,
        },
    )


def confirm_account(room, account_id):
    room.run(
        "odnotuj_potwierdzenie_rachunku",
        id=account_id,
        potwierdzil="tozsamosc-agenta",
        potwierdzono="2026-11-03T08:30:00Z",
        potwierdzenie_sposob="telefonicznie pod numerem z umowy najmu",
    )


def admitted_account(room, account_id, person_id):
    """An account confirmed by another channel, checked by Kontroler and admitted by a human."""
    add_account(room, account_id, person_id)
    confirm_account(room, account_id)
    check(room, f"sprawdzenie-{account_id}", "rachunek", rachunek_id=account_id)
    approved_decision(
        room,
        f"dopuszczenie-{account_id}",
        "dopuszczenie_rachunku",
        "sprawa_rachunki_wpisy",
        account_id,
    )


def prepare_payout(room, payout_id, person_id, account_id, utworzono="2026-11-20T09:00:00Z"):
    room.run(
        "przygotuj_wyplate",
        id=payout_id,
        podstawa_tabela="najem_kaucje",
        podstawa_id="rozliczenie-kaucji",
        kwota=300_000,
        wyliczenie_sciezka=f"wyliczenia/{payout_id}.json",
        wyliczenie_skrot=f"sha256-{payout_id}",
        odbiorca_uczestnik_id=person_id,
        rachunek_id=account_id,
        utworzono=utworzono,
    )


def checked_and_approved_payout(room, payout_id, person_id, account_id):
    """A payout checked anew after it was prepared and approved after that check (D13)."""
    prepare_payout(room, payout_id, person_id, account_id)
    check(
        room,
        f"sprawdzenie-{payout_id}",
        "rachunek",
        sprawdzono="2026-11-20T10:00:00Z",
        rachunek_id=account_id,
    )
    room.run("wskaz_sprawdzenie_wyplaty", id=payout_id, sprawdzenie_id=f"sprawdzenie-{payout_id}")
    approved_decision(
        room,
        f"zgoda-{payout_id}",
        "wyplata",
        "sprawa_wyplaty_wpisy",
        payout_id,
        zlozono="2026-11-20T11:00:00Z",
        podpisano="2026-11-20T12:00:00Z",
    )
    room.run("wskaz_zgode_na_wyplate", id=payout_id, decyzja_id=f"zgoda-{payout_id}")


def add_power_for_office(
    room,
    power_id,
    principal_id,
    czynnosci=("wystapienie_do_wspolnoty",),
    udzielono="2026-11-03",
):
    """A power of attorney the principal gave the office, for the named acts."""
    add_received_document(room, f"dokument-{power_id}", "pelnomocnictwo")
    room.run(
        "wpisz_umocowanie",
        id=power_id,
        rodzaj="pelnomocnictwo",
        mocodawca_uczestnik_id=principal_id,
        umocowane_biuro=1,
        zakres="do_czynnosci",
        forma="pisemna",
        dokument_id=f"dokument-{power_id}",
        udzielono=udzielono,
    )
    for czynnosc in czynnosci:
        room.run("dopisz_czynnosc_umocowania", umocowanie_id=power_id, czynnosc=czynnosc)


def prepare_letter(room, letter_id, **fields):
    room.run(
        "przygotuj_wysylke",
        **{
            "id": letter_id,
            "adresat_rodzaj": "wspolnota",
            "adresat_nazwa": "Wspólnota Mieszkaniowa Polna 7",
            "w_imieniu": "biuro",
            "kanal": "email",
            "wzor": "zapytanie-do-wspolnoty",
            "wzor_wersja": "3",
            "sciezka": f"wysylki/{letter_id}.pdf",
            "skrot": f"sha256-{letter_id}",
            "informacja_rodo": 0,
            "ujawnia_agenta": 0,
            **fields,
        },
    )


def send(room, letter_id, decyzja_id=None, wyslano="2026-11-05T10:00:00Z"):
    room.run("odnotuj_wyslanie", id=letter_id, wyslano=wyslano, decyzja_id=decyzja_id)


def prepare_contract(room, contract_id, party_ids, **fields):
    room.run(
        "przygotuj_umowe",
        **{
            "id": contract_id,
            "rodzaj": "zarzadzanie",
            "nieruchomosc_id": None,
            "obie_strony": 0,
            "wylacznosc": 1,
            "wynagrodzenie_opis": "10% czynszu miesięcznie",
            **fields,
        },
    )
    for party_id in party_ids:
        room.run("dopisz_strone_umowy", umowa_id=contract_id, uczestnik_id=party_id)


# The last day to withdraw from a contract concluded on 2026-11-04 (art. 27 ust. 1 upk).
WITHDRAWAL_ENDS = "2026-11-18"


def conclude_contract(room, contract_id, sposob_zawarcia="w_lokalu_biura", odstapienie_do=None):
    """Concludes a contract on 2026-11-04; a consumer's contract off premises or at a distance
    needs `odstapienie_do`, e.g. WITHDRAWAL_ENDS."""
    add_received_document(room, f"podpisana-{contract_id}", "umowa_z_biurem")
    add_received_document(room, f"polisa-{contract_id}", "polisa")
    room.run(
        "odnotuj_zawarcie_umowy",
        id=contract_id,
        forma="pisemna",
        sposob_zawarcia=sposob_zawarcia,
        zawarto="2026-11-04T12:00:00Z",
        podpisal_za_biuro="tozsamosc-posrednika",
        dokument_id=f"podpisana-{contract_id}",
        polisa_dokument_id=f"polisa-{contract_id}",
        odstapienie_do=odstapienie_do,
    )


def concluded_contract(room, contract_id, party_id, sposob_zawarcia="w_lokalu_biura"):
    """A contract whose party Kontroler checked, signed and concluded."""
    check(room, f"tozsamosc-{contract_id}", "tozsamosc", uczestnik_id=party_id)
    prepare_contract(room, contract_id, [party_id])
    room.run("przekaz_umowe_do_podpisu", id=contract_id)
    conclude_contract(room, contract_id, sposob_zawarcia)


def add_consent(room, consent_id, person_id, rodzaj, **fields):
    add_inflow(room, f"wplyw-{consent_id}", person_id)
    room.run(
        "wpisz_zgode",
        **{
            "id": consent_id,
            "uczestnik_id": person_id,
            "rodzaj": rodzaj,
            "tresc": "Żądam rozpoczęcia pracy przed upływem terminu na odstąpienie.",
            "zlozono": "2026-11-04T13:00:00Z",
            "wplyw_id": f"wplyw-{consent_id}",
            "trwaly_nosnik": 0,
            **fields,
        },
    )


def full_case(room):
    """A rental management case with a row in every table of the core, as the profiles go."""
    open_case(room)

    # The owner, a consumer client of the office, and the tenant, whose data came from her.
    add_person(room, "wlascicielka", konsument=1, klient_biura=1, kanal_kontaktu="email")
    add_person(
        room,
        "najemca",
        imiona="Jan",
        nazwisko="Nowak",
        dane_od="z_innego_zrodla",
        dane_pozyskano="2026-11-02T10:00:00Z",
    )
    room.run(
        "wpisz_adres_uczestnika",
        uczestnik_id="wlascicielka",
        rodzaj="zamieszkanie",
        kraj="PL",
        miejscowosc="Wrocław",
        ulica="Kwiatowa",
        numer_budynku="3",
    )
    room.run("wpisz_obywatelstwo_uczestnika", uczestnik_id="wlascicielka", panstwo="PL")
    room.run("nadaj_role_uczestnikowi", uczestnik_id="wlascicielka", rola="klient")
    room.run("nadaj_role_uczestnikowi", uczestnik_id="najemca", rola="druga_strona")

    # The flat with its land register, its plot and the source of the register number.
    add_flat(room)
    room.run("wpisz_dzialke", nieruchomosc_id="lokal", obreb="0012 Krzyki", numer_dzialki="45/7")
    add_received_document(
        room,
        "odpis-kw",
        "odpis_ksiegi_wieczystej",
        dotyczy_nieruchomosc_id="lokal",
        pochodzenie="pobrany z centralnej bazy ksiąg wieczystych",
        kanal="rejestr_elektroniczny",
        postac="wydruk",
    )
    room.run(
        "wpisz_zrodlo",
        dotyczy_tabela="sprawa_nieruchomosci_wpisy",
        dotyczy_id="lokal",
        pole="numer_kw",
        dokument_id="odpis-kw",
        potwierdzone=1,
        wpisano="2026-11-02T11:00:00Z",
    )
    room.run("zmien_etap_sprawy", etap="umowa", od="2026-11-02T12:00:00Z", zmienil="agent-najmu")

    # The management contract, signed at a distance; the information on withdrawal goes to the
    # client before she signs (art. 12 upk).
    check(room, "tozsamosc-umowa-zarzadzania", "tozsamosc", uczestnik_id="wlascicielka")
    prepare_contract(room, "umowa-zarzadzania", ["wlascicielka"])
    room.run(
        "dopisz_czynnosc_umowy", umowa_id="umowa-zarzadzania", czynnosc="publikacja_ogloszenia"
    )
    prepare_letter(
        room,
        "informacja-dla-klientki",
        adresat_rodzaj=None,
        adresat_nazwa=None,
        adresat_uczestnik_id="wlascicielka",
        wzor="informacja-o-odstapieniu",
        informacja_rodo=1,
        ujawnia_agenta=1,
    )
    send(room, "informacja-dla-klientki", wyslano="2026-11-04T10:00:00Z")
    room.run(
        "odnotuj_doreczenie",
        id="informacja-dla-klientki",
        doreczono="2026-11-04T10:01:00Z",
        dowod_doreczenia="potwierdzenie odczytu",
    )
    room.run(
        "odnotuj_informacje_o_odstapieniu",
        id="umowa-zarzadzania",
        informacja_o_odstapieniu_wysylka_id="informacja-dla-klientki",
    )
    room.run("przekaz_umowe_do_podpisu", id="umowa-zarzadzania")
    conclude_contract(
        room,
        "umowa-zarzadzania",
        sposob_zawarcia="na_odleglosc",
        odstapienie_do=WITHDRAWAL_ENDS,
    )
    add_consent(room, "zadanie", "wlascicielka", "zadanie_rozpoczecia_pracy")
    room.run(
        "wpisz_zdarzenie_umowy",
        id="zdarzenie-zadanie",
        umowa_id="umowa-zarzadzania",
        rodzaj="zadanie_rozpoczecia_pracy",
        chwila="2026-11-04T13:00:00Z",
        zgoda_id="zadanie",
    )
    room.run(
        "wpisz_termin",
        id="termin-odstapienia",
        rodzaj="ustawowy",
        co="Koniec terminu na odstąpienie od umowy",
        zdarzenie_tabela="sprawa_umowy_wpisy",
        zdarzenie_id="umowa-zarzadzania",
        zdarzenie_chwila="2026-11-04T12:00:00Z",
        koniec="2026-11-18",
        podstawa="art. 27 ust. 1 i art. 28 pkt 2 upk",
        pilnuje_stanowisko="opiekun_najmu",
    )

    # A letter to the owners' association on the client's behalf, ordering a certificate.
    add_power_for_office(room, "pelnomocnictwo", "wlascicielka")
    check(room, "sprawdzenie-pelnomocnictwa", "umocowanie", umocowanie_id="pelnomocnictwo")
    room.run(
        "wpisz_wymagany_dokument",
        id="zaswiadczenie-wspolnoty",
        rodzaj="zaswiadczenie",
        nazwa="Zaświadczenie o braku zaległości",
        dotyczy_nieruchomosc_id="lokal",
        wymaga="umowa",
    )
    prepare_letter(
        room,
        "wniosek-do-wspolnoty",
        w_imieniu="uczestnik",
        w_imieniu_uczestnik_id="wlascicielka",
        umocowanie_id="pelnomocnictwo",
        czynnosc="wystapienie_do_wspolnoty",
        zamawia_dokument_id="zaswiadczenie-wspolnoty",
    )
    room.run(
        "dolacz_dokument_do_wysylki",
        wysylka_id="wniosek-do-wspolnoty",
        dokument_id="dokument-pelnomocnictwo",
    )
    send(room, "wniosek-do-wspolnoty")
    room.run("odnotuj_zamowienie_dokumentu", id="zaswiadczenie-wspolnoty")
    add_inflow(room, "odpowiedz-wspolnoty", nadawca="Wspólnota Mieszkaniowa Polna 7")
    room.run(
        "odnotuj_otrzymanie_dokumentu",
        id="zaswiadczenie-wspolnoty",
        wystawca="Wspólnota Mieszkaniowa Polna 7",
        data_wystawienia="2026-11-06",
        waznosc_do="2026-12-06",
        waznosc_zrodlo="instytucja",
        pochodzenie="od wspólnoty",
        kanal="email",
        postac="skan",
        sciezka="dokumenty/zaswiadczenie-wspolnoty.pdf",
        skrot="sha256-zaswiadczenie-wspolnoty",
    )
    room.run(
        "dolacz_dokument_do_wplywu",
        wplyw_id="odpowiedz-wspolnoty",
        dokument_id="zaswiadczenie-wspolnoty",
    )

    # Kontroler raises a doubt; the human lifts it with a signed decision without any content.
    room.run(
        "postaw_watpliwosc",
        id="watpliwosc",
        postawil="tozsamosc-kontrolera",
        postawiono="2026-11-07T09:00:00Z",
    )
    ask_for_decision(
        room,
        "zdjecie-watpliwosci",
        "zdjecie_watpliwosci",
        "sprawa_metryka_watpliwosci",
        "watpliwosc",
        prosi_stanowisko="kontroler",
        decyduje="tozsamosc-wlasciciela-kierownika-kontroli",
        zlozono="2026-11-07T10:00:00Z",
    )
    approve(room, "zdjecie-watpliwosci", "2026-11-07T11:00:00Z")
    room.run("zdejmij_watpliwosc", id="watpliwosc", zdjecie_decyzja_id="zdjecie-watpliwosci")

    # The handover date, confirmed by a human.
    room.run(
        "wpisz_termin",
        id="termin-przekazania",
        rodzaj="z_umowy",
        co="Przekazanie lokalu najemcy",
        koniec="2026-11-10T10:00:00Z",
        podstawa="§ 3 umowy najmu",
        pilnuje_stanowisko="opiekun_najmu",
        kalendarz_odnosnik="kalendarz/przekazanie-polna-7",
    )
    approved_decision(
        room,
        "potwierdzenie-przekazania",
        "potwierdzenie_terminu",
        "sprawa_terminy_wpisy",
        "termin-przekazania",
        zlozono="2026-11-08T10:00:00Z",
        podpisano="2026-11-08T12:00:00Z",
    )
    room.run(
        "odnotuj_potwierdzenie_terminu",
        id="termin-przekazania",
        potwierdzenie_decyzja_id="potwierdzenie-przekazania",
    )
    room.run("zmien_stan_terminu", id="termin-przekazania", stan="dotrzymany")

    # The handover protocol with the meter reading and the keys, signed by both sides.
    room.run(
        "przygotuj_protokol",
        id="protokol-wydania",
        rodzaj="wydanie",
        nieruchomosc_id="lokal",
        chwila="2026-11-10T10:00:00Z",
    )
    room.run(
        "dopisz_pozycje_protokolu",
        protokol_id="protokol-wydania",
        rodzaj="licznik",
        nazwa="licznik energii elektrycznej",
        numer="E-778812",
        opis_stanu="12 345,6 kWh",
        zdjecie_sciezka="protokoly/wydanie/licznik.jpg",
        zdjecie_skrot="sha256-licznik",
    )
    room.run(
        "dopisz_pozycje_protokolu",
        protokol_id="protokol-wydania",
        rodzaj="klucze",
        nazwa="klucze do drzwi wejściowych",
        opis_stanu="3 sztuki",
    )
    add_received_document(room, "podpisany-protokol-wydania", "protokol")
    room.run(
        "podpisz_protokol",
        id="protokol-wydania",
        dokument_id="podpisany-protokol-wydania",
    )

    # The deposit goes back to the tenant: an admitted account, a fresh check, the human's
    # consent, the transfer with the bank's confirmation.
    admitted_account(room, "rachunek-najemcy", "najemca")
    checked_and_approved_payout(room, "zwrot-kaucji", "najemca", "rachunek-najemcy")
    add_received_document(room, "potwierdzenie-przelewu", "potwierdzenie_bankowe")
    room.run(
        "odnotuj_wykonanie_wyplaty",
        id="zwrot-kaucji",
        wykonal="tozsamosc-posrednika",
        wykonano="2026-11-20T13:00:00Z",
        potwierdzenie_dokument_id="potwierdzenie-przelewu",
    )

    # Kontroler leaves a note on the account; the link to the search case of the tenant.
    room.run(
        "dopisz_notatke",
        id="notatka-rachunek",
        od="tozsamosc-kontrolera",
        od_stanowiska="kontroler",
        do_stanowiska="opiekun_najmu",
        tresc="Rachunek najemcy sprawdzony przed zwrotem kaucji.",
        utworzono="2026-11-20T10:05:00Z",
    )
    room.run(
        "wskaz_wiersz_notatki",
        notatka_id="notatka-rachunek",
        dotyczy_tabela="sprawa_rachunki_wpisy",
        dotyczy_id="rachunek-najemcy",
    )
    room.run("oznacz_notatke_jako_wykonana", id="notatka-rachunek")
    room.run(
        "wpisz_powiazanie",
        rodzaj="sprawa_zrodlowa",
        typ_teczki="oferta",
        oznaczenie="Oferta 2026/041",
        odnosnik="pokoj-oferty-041",
        wpisal="tozsamosc-kierownika-najmu",
        wpisano="2026-11-02T09:15:00Z",
    )


# The whole case ------------------------------------------------------------------------------------


@scenario("Przebieg najmu: od przyjęcia przez umowę, pismo w imieniu klientki i zwrot kaucji")
def whole_case():
    room = new_room()

    full_case(room)
    room.run("zmien_etap_sprawy", etap="zamknieta", od="2026-11-21T09:00:00Z", zmienil="agent")

    etapy = [row["etap"] for row in room.run("etapy")]
    assert etapy == ["przyjeta", "umowa", "zamknieta"], etapy
    assert room.run("biezacy_etap")[0]["etap"] == "zamknieta"
    assert room.run("watpliwosci_stojace") == []
    assert room.run("wyplaty")[0]["stan"] == "wykonana"
    assert room.run("dokumenty_do_zebrania") == []
    assert [row["id"] for row in room.run("zgody_biezace")] == ["zadanie"]

    sent = room.run("wysylki_do_uczestnika", adresat_uczestnik_id="wlascicielka")
    assert sent[0]["informacja_rodo"] == 1 and sent[0]["ujawnia_agenta"] == 1, sent


# Decisions ---------------------------------------------------------------------------------------


@scenario("Decyzja: prośba powstaje jako czekająca")
def decision_starts_waiting():
    room = new_room()

    ask_for_decision(room, "prosba", "inna", tresc="Zgoda na ogłoszenie bez zdjęć")
    assert room.run("decyzje_czekajace")[0]["id"] == "prosba"

    room.refused_sql(
        "prośba o decyzję powstaje jako czekająca",
        """INSERT INTO sprawa_decyzje_wpisy (id, rodzaj, prosi, prosi_stanowisko, decyduje, zlozono,
          stan, rozstrzygniecie, podpisal, podpis, podpisano)
        VALUES ('od-razu', 'inna', 'agent', 'opiekun_najmu', 'czlowiek', '2026-11-03T10:00:00Z',
          'rozstrzygnieta', 'zgoda', 'czlowiek', 'podpis', '2026-11-03T12:00:00Z')""",
    )


@scenario("Decyzja: rozstrzygnięcie ma pełny podpis, a czekająca żadnego")
def decision_signature_complete():
    room = new_room()
    ask_for_decision(room, "prosba", "inna")

    room.refused(
        "podpis_w_komplecie",
        "rozstrzygnij_decyzje",
        id="prosba",
        rozstrzygniecie="zgoda",
        podpisal="tozsamosc-posrednika",
        podpisano="2026-11-03T12:00:00Z",
    )
    room.refused_sql(
        "podpis_tylko_przy_rozstrzygnieciu",
        """UPDATE sprawa_decyzje_wpisy
        SET rozstrzygniecie = 'zgoda', podpisal = 'czlowiek', podpis = 'podpis',
          podpisano = '2026-11-03T12:00:00Z'
        WHERE id = 'prosba'""",
    )

    room.run(
        "rozstrzygnij_decyzje",
        id="prosba",
        rozstrzygniecie="zgoda",
        za_kogo="tozsamosc-nieobecnego-posrednika",
        **signature("2026-11-03T12:00:00Z"),
    )
    assert room.run("decyzje")[0]["za_kogo"] == "tozsamosc-nieobecnego-posrednika"


@scenario("Decyzja: rozstrzygnięcie zapada po złożeniu prośby")
def decision_not_before_request():
    room = new_room()
    ask_for_decision(room, "prosba", "inna", zlozono="2026-11-03T10:00:00Z")

    room.refused(
        "rozstrzygniecie_nie_przed_zlozeniem",
        "rozstrzygnij_decyzje",
        id="prosba",
        rozstrzygniecie="odmowa",
        **signature("2026-11-03T09:00:00Z"),
    )

    approve(room, "prosba", "2026-11-03T10:00:00Z", rozstrzygniecie="odmowa")


@scenario("Decyzja: rozstrzygniętej i zbędnej się nie zmienia; nowa zastępuje poprzednią")
def decision_is_final():
    room = new_room()
    approved_decision(room, "pierwsza", "inna")

    room.refused(
        "rozstrzygniętej ani zbędnej decyzji się nie zmienia",
        "uznaj_prosbe_za_zbedna",
        id="pierwsza",
        powod_zbednosci="sprawa się zmieniła",
    )
    room.refused_sql(
        "rozstrzygniętej ani zbędnej decyzji się nie zmienia",
        "UPDATE sprawa_decyzje_wpisy SET uzasadnienie = 'dopisek' WHERE id = 'pierwsza'",
    )

    ask_for_decision(room, "zbedna", "inna")
    room.run("uznaj_prosbe_za_zbedna", id="zbedna", powod_zbednosci="klientka wycofała prośbę")
    room.refused(
        "rozstrzygniętej ani zbędnej decyzji się nie zmienia",
        "rozstrzygnij_decyzje",
        id="zbedna",
        rozstrzygniecie="zgoda",
        **signature("2026-11-03T12:00:00Z"),
    )


@scenario("Decyzja: nowa zastępuje obowiązującą decyzję tego samego rodzaju o tym samym wierszu")
def decision_replaced_by_newer():
    room = new_room()
    approved_decision(room, "pierwsza", "inna", "sprawa_terminy_wpisy", "termin")
    refusal = "nowa decyzja zastępuje obowiązującą rozstrzygniętą decyzję tego samego rodzaju"

    room.refused(
        refusal,
        "popros_o_decyzje",
        id="innego-rodzaju",
        rodzaj="potwierdzenie_terminu",
        prosi="agent",
        prosi_stanowisko="opiekun_najmu",
        decyduje="czlowiek",
        zlozono="2026-11-04T10:00:00Z",
        dotyczy_tabela="sprawa_terminy_wpisy",
        dotyczy_id="termin",
        zastepuje_decyzja_id="pierwsza",
    )
    room.refused(
        refusal,
        "popros_o_decyzje",
        id="o-innym-wierszu",
        rodzaj="inna",
        prosi="agent",
        prosi_stanowisko="opiekun_najmu",
        decyduje="czlowiek",
        zlozono="2026-11-04T10:00:00Z",
        dotyczy_tabela="sprawa_terminy_wpisy",
        dotyczy_id="inny-termin",
        zastepuje_decyzja_id="pierwsza",
    )
    ask_for_decision(room, "czekajaca", "inna", "sprawa_terminy_wpisy", "termin")
    room.refused(
        refusal,
        "popros_o_decyzje",
        id="zastepuje-czekajaca",
        rodzaj="inna",
        prosi="agent",
        prosi_stanowisko="opiekun_najmu",
        decyduje="czlowiek",
        zlozono="2026-11-04T10:00:00Z",
        dotyczy_tabela="sprawa_terminy_wpisy",
        dotyczy_id="termin",
        zastepuje_decyzja_id="czekajaca",
    )

    # Without a newer decision a settled one does not turn into a replaced one.
    room.refused_sql(
        "rozstrzygniętej ani zbędnej decyzji się nie zmienia",
        "UPDATE sprawa_decyzje_wpisy SET stan = 'zastapiona' WHERE id = 'pierwsza'",
    )

    # Two requests replace the same decision; only the first one settled takes its place.
    for decision_id in ("druga", "rownolegla"):
        ask_for_decision(
            room,
            decision_id,
            "inna",
            "sprawa_terminy_wpisy",
            "termin",
            zastepuje_decyzja_id="pierwsza",
        )
    assert room.value("SELECT stan FROM sprawa_decyzje_wpisy WHERE id = 'pierwsza'") == (
        "rozstrzygnieta"
    )
    approve(room, "druga", rozstrzygniecie="odmowa")
    pierwsza = room.run(
        "decyzje_wiersza", dotyczy_tabela="sprawa_terminy_wpisy", dotyczy_id="termin"
    )
    replaced = next(row for row in pierwsza if row["id"] == "pierwsza")
    assert replaced["stan"] == "zastapiona" and replaced["rozstrzygniecie"] == "zgoda", replaced

    room.refused(
        refusal,
        "rozstrzygnij_decyzje",
        id="rownolegla",
        rozstrzygniecie="zgoda",
        **signature("2026-11-03T12:00:00Z"),
    )
    room.refused_sql(
        "rozstrzygniętej ani zbędnej decyzji się nie zmienia",
        "UPDATE sprawa_decyzje_wpisy SET stan = 'rozstrzygnieta' WHERE id = 'pierwsza'",
    )


@scenario("Decyzja: zgoda z warunkiem ma warunek, a prośba zbędna powód")
def decision_condition_and_reason():
    room = new_room()
    ask_for_decision(room, "z-warunkiem", "inna")
    ask_for_decision(room, "odmowa", "inna")

    room.refused(
        "zgoda_z_warunkiem_ma_warunek",
        "rozstrzygnij_decyzje",
        id="z-warunkiem",
        rozstrzygniecie="zgoda_z_warunkiem",
        **signature("2026-11-03T12:00:00Z"),
    )
    room.refused(
        "zgoda_z_warunkiem_ma_warunek",
        "rozstrzygnij_decyzje",
        id="odmowa",
        rozstrzygniecie="odmowa",
        warunek="po obniżce ceny",
        **signature("2026-11-03T12:00:00Z"),
    )
    room.run(
        "rozstrzygnij_decyzje",
        id="z-warunkiem",
        rozstrzygniecie="zgoda_z_warunkiem",
        warunek="ogłoszenie dopiero po sesji zdjęciowej",
        uzasadnienie="klientka chce zdjęć",
        **signature("2026-11-03T12:00:00Z"),
    )

    ask_for_decision(room, "zbedna", "inna")
    room.refused_sql(
        "zbedna_ma_powod",
        "UPDATE sprawa_decyzje_wpisy SET stan = 'zbedna' WHERE id = 'zbedna'",
    )
    room.run("uznaj_prosbe_za_zbedna", id="zbedna", powod_zbednosci="termin się zmienił")


@scenario("Decyzja o zdjęciu wątpliwości nie niesie treści ani uzasadnienia (art. 54 AML)")
def decision_aml_without_content():
    room = new_room()

    room.refused(
        "decyzja_aml_bez_tresci",
        "popros_o_decyzje",
        id="z-trescia",
        rodzaj="zdjecie_watpliwosci",
        prosi="kontroler",
        prosi_stanowisko="kontroler",
        decyduje="czlowiek",
        zlozono="2026-11-03T10:00:00Z",
        tresc="beneficjent wyjaśniony",
        dotyczy_tabela="sprawa_metryka_watpliwosci",
        dotyczy_id="watpliwosc",
    )

    ask_for_decision(
        room, "bez-tresci", "zdjecie_watpliwosci", "sprawa_metryka_watpliwosci", "watpliwosc"
    )
    room.refused(
        "decyzja_aml_bez_tresci",
        "rozstrzygnij_decyzje",
        id="bez-tresci",
        rozstrzygniecie="zgoda",
        uzasadnienie="źródło środków potwierdzone",
        **signature("2026-11-03T12:00:00Z"),
    )
    approve(room, "bez-tresci")


@scenario("Decyzja: wskazany wiersz to tabela z identyfikatorem, a skrót tylko przy wierszu")
def decision_row_reference():
    room = new_room()

    room.refused(
        "dotyczy_tabela_razem_z_id",
        "popros_o_decyzje",
        id="bez-id",
        rodzaj="inna",
        prosi="agent",
        prosi_stanowisko="opiekun_najmu",
        decyduje="czlowiek",
        zlozono="2026-11-03T10:00:00Z",
        dotyczy_tabela="sprawa_rachunki_wpisy",
    )
    room.refused(
        "skrot_tylko_przy_wierszu",
        "popros_o_decyzje",
        id="skrot-bez-wiersza",
        rodzaj="inna",
        prosi="agent",
        prosi_stanowisko="opiekun_najmu",
        decyduje="czlowiek",
        zlozono="2026-11-03T10:00:00Z",
        dotyczy_skrot="sha256-czegos",
    )

    ask_for_decision(
        room,
        "o-wierszu",
        "inna",
        "oferta_ceny",
        "cena-1",
        dotyczy_skrot="sha256-cena-1",
    )
    assert room.run("decyzje_wiersza", dotyczy_tabela="oferta_ceny", dotyczy_id="cena-1")


# Case record, stages and doubts --------------------------------------------------------------------


@scenario("Metryka: typ pokoju jest nazwą folderu, a metryki się nie zmienia")
def record_room_type():
    room = new_room()

    room.refused(
        "typ_pokoju",
        "zaloz_metryke",
        oznaczenie="Najem 2026/017",
        typ_pokoju="Najem",
        przyjal="czlowiek",
        przyjeto="2026-11-02T09:00:00Z",
    )

    open_case(room)
    for column, value in (
        ("typ_pokoju", "oferta"),
        ("oznaczenie", "Najem 2026/017, ul. Polna 7 m. 14"),
        ("przyjal", "ktos-inny"),
        ("przyjeto", "2026-11-01T09:00:00Z"),
    ):
        room.refused_sql(
            "metryki teczki się nie zmienia",
            f"UPDATE sprawa_metryka_wpis SET {column} = :value",
            value=value,
        )


@scenario("Etapy: pierwszy to przyjęta, a dziennika się nie zmienia")
def stages_start_accepted():
    room = new_room()

    room.refused(
        "pierwszy etap sprawy to przyjeta",
        "zmien_etap_sprawy",
        etap="umowa",
        od="2026-11-02T09:00:00Z",
        zmienil="agent",
    )

    open_case(room)
    room.run("zmien_etap_sprawy", etap="umowa", od="2026-11-02T12:00:00Z", zmienil="agent")
    assert room.run("biezacy_etap")[0]["etap"] == "umowa"

    room.refused_sql(
        "wpisu w dzienniku etapów się nie zmienia",
        "UPDATE sprawa_metryka_etapy SET etap = 'przyjeta' WHERE etap = 'umowa'",
    )
    room.refused(
        "etap",
        "zmien_etap_sprawy",
        etap="Umowa",
        od="2026-11-02T13:00:00Z",
        zmienil="agent",
    )


@scenario("Etapy: przerwanie wymaga zatwierdzonej decyzji przerwanie_sprawy")
def stage_interrupted_needs_decision():
    room = new_room()
    open_case(room)

    room.refused(
        "etap przerwana wymaga zatwierdzonej decyzji przerwanie_sprawy",
        "zmien_etap_sprawy",
        etap="przerwana",
        od="2026-11-05T09:00:00Z",
        zmienil="agent",
    )

    ask_for_decision(room, "odmowa-przerwania", "przerwanie_sprawy")
    approve(room, "odmowa-przerwania", rozstrzygniecie="odmowa")
    room.refused(
        "etap przerwana wymaga zatwierdzonej decyzji przerwanie_sprawy",
        "zmien_etap_sprawy",
        etap="przerwana",
        od="2026-11-05T09:00:00Z",
        zmienil="agent",
        decyzja_id="odmowa-przerwania",
    )

    approved_decision(room, "inna-decyzja", "inna")
    room.refused(
        "etap przerwana wymaga zatwierdzonej decyzji przerwanie_sprawy",
        "zmien_etap_sprawy",
        etap="przerwana",
        od="2026-11-05T09:00:00Z",
        zmienil="agent",
        decyzja_id="inna-decyzja",
    )

    approved_decision(room, "przerwanie", "przerwanie_sprawy")
    room.run(
        "zmien_etap_sprawy",
        etap="przerwana",
        od="2026-11-05T09:00:00Z",
        zmienil="agent",
        decyzja_id="przerwanie",
    )


def enter_stage(room, etap, od, decyzja_id=None):
    room.run("zmien_etap_sprawy", etap=etap, od=od, zmienil="agent", decyzja_id=decyzja_id)


@scenario(
    "Etapy: przerwana sprawa wraca do etapu sprzed przerwania za decyzją wznowienie_sprawy "
    "albo się zamyka (T1)"
)
def stage_resumed_after_interruption():
    room = new_room()
    open_case(room)
    enter_stage(room, "umowa", "2026-11-03T09:00:00Z")
    approved_decision(room, "przerwanie", "przerwanie_sprawy", podpisano="2026-11-04T12:00:00Z")
    # A consent to resume given before the interruption concerns another situation.
    approved_decision(
        room, "wczesne-wznowienie", "wznowienie_sprawy", podpisano="2026-11-04T13:00:00Z"
    )
    enter_stage(room, "przerwana", "2026-11-05T09:00:00Z", decyzja_id="przerwanie")
    decision = "powrót z etapu przerwana wymaga zatwierdzonej decyzji wznowienie_sprawy"
    other_stage = "po etapie przerwana przychodzi zamknieta albo powrót do etapu sprzed przerwania"

    room.refused(
        decision, "zmien_etap_sprawy", etap="umowa", od="2026-11-06T09:00:00Z", zmienil="agent"
    )
    room.refused(
        decision,
        "zmien_etap_sprawy",
        etap="umowa",
        od="2026-11-06T09:00:00Z",
        zmienil="agent",
        decyzja_id="wczesne-wznowienie",
    )
    ask_for_decision(room, "odmowa-wznowienia", "wznowienie_sprawy")
    approve(room, "odmowa-wznowienia", "2026-11-06T08:00:00Z", rozstrzygniecie="odmowa")
    room.refused(
        decision,
        "zmien_etap_sprawy",
        etap="umowa",
        od="2026-11-06T09:00:00Z",
        zmienil="agent",
        decyzja_id="odmowa-wznowienia",
    )

    approved_decision(room, "wznowienie", "wznowienie_sprawy", podpisano="2026-11-06T08:00:00Z")
    for etap in ("przyjeta", "inny_etap"):
        room.refused(
            other_stage,
            "zmien_etap_sprawy",
            etap=etap,
            od="2026-11-06T09:00:00Z",
            zmienil="agent",
            decyzja_id="wznowienie",
        )
    enter_stage(room, "umowa", "2026-11-06T09:00:00Z", decyzja_id="wznowienie")

    # The second interruption needs its own consent, and so does the second resumption.
    room.refused(
        "jedna decyzja stoi za jednym wejściem w etap",
        "zmien_etap_sprawy",
        etap="przerwana",
        od="2026-11-07T09:00:00Z",
        zmienil="agent",
        decyzja_id="przerwanie",
    )
    approved_decision(
        room, "drugie-przerwanie", "przerwanie_sprawy", podpisano="2026-11-06T09:30:00Z"
    )
    # The second interruption is entered with an earlier time, so the first consent to resume
    # passes the time check and only the rule of one entry per decision refuses it.
    enter_stage(room, "przerwana", "2026-11-06T07:00:00Z", decyzja_id="drugie-przerwanie")
    room.refused(
        "jedna decyzja stoi za jednym wejściem w etap",
        "zmien_etap_sprawy",
        etap="umowa",
        od="2026-11-07T09:00:00Z",
        zmienil="agent",
        decyzja_id="wznowienie",
    )

    enter_stage(room, "zamknieta", "2026-11-08T09:00:00Z")
    etapy = [row["etap"] for row in room.run("etapy")]
    assert etapy == ["przyjeta", "umowa", "przerwana", "umowa", "przerwana", "zamknieta"], etapy


@scenario("Etapy: po zamknięciu teczki nie ma kolejnych etapów")
def no_stage_after_closing():
    room = new_room()
    open_case(room)

    room.run("zmien_etap_sprawy", etap="zamknieta", od="2026-11-30T09:00:00Z", zmienil="agent")
    room.refused(
        "po etapie zamknieta sprawa nie ma kolejnych etapów",
        "zmien_etap_sprawy",
        etap="umowa",
        od="2026-12-01T09:00:00Z",
        zmienil="agent",
    )


@scenario("Wątpliwość: powstaje stojąca, zdejmuje ją decyzja, która ją wskazuje i zapadła po niej")
def doubt_lifted_by_decision():
    room = new_room()
    open_case(room)

    room.refused_sql(
        "znacznik wątpliwości powstaje stojący",
        """INSERT INTO sprawa_metryka_watpliwosci (id, postawil, postawiono, zdjecie_decyzja_id)
        VALUES ('od-razu-zdjeta', 'kontroler', '2026-11-03T09:00:00Z', 'decyzja')""",
    )

    # A decision signed before the doubt was raised does not lift it.
    ask_for_decision(
        room,
        "wczesniejsza",
        "zdjecie_watpliwosci",
        "sprawa_metryka_watpliwosci",
        "watpliwosc",
        zlozono="2026-11-03T08:00:00Z",
    )
    approve(room, "wczesniejsza", "2026-11-03T08:30:00Z")
    for kontroler, postawiono in (("kontroler-sprzedajacej", "09"), ("kontroler-kupujacego", "10")):
        room.run(
            "postaw_watpliwosc",
            id="watpliwosc" if kontroler == "kontroler-sprzedajacej" else "druga",
            postawil=kontroler,
            postawiono=f"2026-11-03T{postawiono}:00:00Z",
        )
    assert len(room.run("watpliwosci_stojace")) == 2

    room.refused(
        "znacznik zdejmuje tylko zatwierdzona decyzja zdjecie_watpliwosci",
        "zdejmij_watpliwosc",
        id="watpliwosc",
        zdjecie_decyzja_id="wczesniejsza",
    )

    # A decision on another doubt does not lift this one.
    ask_for_decision(
        room,
        "o-drugiej",
        "zdjecie_watpliwosci",
        "sprawa_metryka_watpliwosci",
        "druga",
        zlozono="2026-11-03T11:00:00Z",
    )
    approve(room, "o-drugiej", "2026-11-03T12:00:00Z")
    room.refused(
        "znacznik zdejmuje tylko zatwierdzona decyzja zdjecie_watpliwosci",
        "zdejmij_watpliwosc",
        id="watpliwosc",
        zdjecie_decyzja_id="o-drugiej",
    )

    ask_for_decision(
        room,
        "o-pierwszej",
        "zdjecie_watpliwosci",
        "sprawa_metryka_watpliwosci",
        "watpliwosc",
        zlozono="2026-11-03T11:00:00Z",
    )
    approve(room, "o-pierwszej", "2026-11-03T12:00:00Z")
    room.run("zdejmij_watpliwosc", id="watpliwosc", zdjecie_decyzja_id="o-pierwszej")
    assert [row["id"] for row in room.run("watpliwosci_stojace")] == ["druga"]

    room.refused(
        "znacznika wątpliwości się nie zmienia",
        "zdejmij_watpliwosc",
        id="watpliwosc",
        zdjecie_decyzja_id="o-drugiej",
    )
    room.refused_sql(
        "znacznika wątpliwości się nie zmienia",
        "UPDATE sprawa_metryka_watpliwosci SET postawil = 'ktos-inny' WHERE id = 'druga'",
    )


# Participants ------------------------------------------------------------------------------------


@scenario("Uczestnik: osoba ma imiona i nazwisko, podmiot nazwę i nie ma danych osoby")
def participant_names():
    room = new_room()

    room.refused("osoba_ma_imie_i_nazwisko", "wpisz_uczestnika", **person_fields(nazwisko=None))
    room.refused(
        "podmiot_ma_nazwe",
        "wpisz_uczestnika",
        **person_fields(rodzaj_podmiotu="osoba_prawna", imiona=None, nazwisko=None),
    )
    room.refused(
        "podmiot_bez_danych_osoby",
        "wpisz_uczestnika",
        **person_fields(rodzaj_podmiotu="osoba_prawna", nazwa="Polna sp. z o.o."),
    )

    add_person(room, "osoba")
    add_company(room, "spolka")


def person_fields(**fields):
    return {
        "id": "osoba",
        "rodzaj_podmiotu": "osoba_fizyczna",
        "imiona": "Anna",
        "nazwisko": "Kowalska",
        "pesel_nie_nadano": 0,
        "konsument": 0,
        "ceidg": 0,
        "klient_biura": 0,
        "dane_od": "od_osoby",
        "dane_pozyskano": "2026-11-02T09:30:00Z",
        **fields,
    }


@scenario("Uczestnik: PESEL ma 11 cyfr, a „PESEL nie nadano” go wyklucza")
def participant_pesel():
    room = new_room()

    room.refused("pesel_ma_11_cyfr", "wpisz_uczestnika", **person_fields(pesel="8001011234"))
    room.refused("pesel_ma_11_cyfr", "wpisz_uczestnika", **person_fields(pesel="8001011234X"))
    room.refused(
        "pesel_nie_nadano_wyklucza_pesel",
        "wpisz_uczestnika",
        **person_fields(pesel="80010112345", pesel_nie_nadano=1),
    )

    add_person(room, "z-peselem", pesel="80010112345")
    add_person(room, "cudzoziemiec", pesel_nie_nadano=1)


@scenario("Uczestnik: rodzaj podmiotu się nie zmienia")
def participant_kind_fixed():
    room = new_room()
    add_person(room, "malzonek")
    add_person(room, "zona", imiona="Ewa")
    room.run(
        "wpisz_adres_uczestnika",
        uczestnik_id="zona",
        rodzaj="zamieszkanie",
        kraj="PL",
        miejscowosc="Wrocław",
    )
    room.run("wpisz_obywatelstwo_uczestnika", uczestnik_id="zona", panstwo="PL")
    room.run(
        "nadaj_role_uczestnikowi",
        uczestnik_id="zona",
        rola="malzonek",
        dotyczy_uczestnik_id="malzonek",
    )

    # The workaround from the audit: a person with a home address, a citizenship and a spouse
    # turns into a company.
    room.refused_sql(
        "rodzaj podmiotu uczestnika się nie zmienia",
        """UPDATE sprawa_uczestnicy_wpisy
        SET rodzaj_podmiotu = 'osoba_prawna', nazwa = 'X', imiona = NULL, nazwisko = NULL
        WHERE id = 'zona'""",
    )
    room.run("zmien_dane_uczestnika", **person_fields(id="zona", nazwisko="Nowak"))


@scenario("Uczestnik: ustrój majątkowy tylko przy małżeństwie, konsument tylko osoba fizyczna")
def participant_family_and_consumer():
    room = new_room()
    add_person(room, "osoba")

    room.refused(
        "ustroj_tylko_w_malzenstwie",
        "ustal_sytuacje_rodzinna_uczestnika",
        id="osoba",
        stan_cywilny="rozwiedziony",
        ustroj_majatkowy="wspolnosc_ustawowa",
    )
    room.run(
        "ustal_sytuacje_rodzinna_uczestnika",
        id="osoba",
        stan_cywilny="w_separacji",
        ustroj_majatkowy="rozdzielnosc_z_mocy_prawa",
    )

    room.refused(
        "konsument_tylko_osoba_fizyczna",
        "wpisz_uczestnika",
        **person_fields(
            id="spolka",
            rodzaj_podmiotu="osoba_prawna",
            imiona=None,
            nazwisko=None,
            nazwa="Polna sp. z o.o.",
            konsument=1,
        ),
    )
    add_person(room, "konsumentka", konsument=1)

    room.refused(
        "krs_albo_ceidg",
        "wpisz_uczestnika",
        **person_fields(id="przedsiebiorca", nazwa="Kowalska Nieruchomości", krs="1", ceidg=1),
    )


@scenario(
    "Rola: małżonek wskazuje osobę fizyczną; pełnomocnika i reprezentanta daje umocowanie;"
    " rola kończy się raz, z powodem"
)
def participant_roles():
    room = new_room()
    add_person(room, "zona")
    add_person(room, "maz", imiona="Piotr")
    add_company(room, "spolka")

    room.refused(
        "malzonek_wskazuje_osobe", "nadaj_role_uczestnikowi", uczestnik_id="maz", rola="malzonek"
    )
    room.refused(
        "małżonek i osoba, której jest małżonkiem, to osoby fizyczne",
        "nadaj_role_uczestnikowi",
        uczestnik_id="maz",
        rola="malzonek",
        dotyczy_uczestnik_id="spolka",
    )
    room.refused(
        "rola_nie_dotyczy_siebie",
        "nadaj_role_uczestnikowi",
        uczestnik_id="maz",
        rola="malzonek",
        dotyczy_uczestnik_id="maz",
    )
    room.run(
        "nadaj_role_uczestnikowi", uczestnik_id="maz", rola="malzonek", dotyczy_uczestnik_id="zona"
    )

    for rola in ("pelnomocnik", "reprezentant"):
        room.refused(
            "pelnomocnik_i_reprezentant_z_umocowan",
            "nadaj_role_uczestnikowi",
            uczestnik_id="zona",
            rola=rola,
        )
    room.refused("rola", "nadaj_role_uczestnikowi", uczestnik_id="zona", rola="Klientka")
    room.run("nadaj_role_uczestnikowi", uczestnik_id="zona", rola="klient")

    room.refused_sql(
        "roli w teczce się nie zmienia",
        "UPDATE sprawa_uczestnicy_role SET rola = 'druga_strona' WHERE rola = 'klient'",
    )

    role_id = room.value("SELECT id FROM sprawa_uczestnicy_role WHERE rola = 'klient'")
    room.refused_sql(
        "rola powstaje jako obowiązująca",
        "INSERT INTO sprawa_uczestnicy_role (uczestnik_id, rola, zakonczono, powod_zakonczenia) "
        "VALUES ('maz', 'klient', '2026-11-17', 'pomyłka')",
    )
    room.refused(
        "zakonczenie_roli_z_powodem",
        "zakoncz_role_uczestnika",
        id=role_id,
        zakonczono="2026-11-17",
        powod_zakonczenia=None,
    )
    room.run(
        "zakoncz_role_uczestnika",
        id=role_id,
        zakonczono="2026-11-17",
        powod_zakonczenia="pomyłka: klientem jest mąż",
    )
    room.refused(
        "rola może się tylko raz zakończyć",
        "zakoncz_role_uczestnika",
        id=role_id,
        zakonczono="2026-11-18",
        powod_zakonczenia="inny powód",
    )


@scenario("Adres i obywatelstwo: zamieszkanie i obywatelstwo ma osoba, siedzibę podmiot")
def participant_addresses():
    room = new_room()
    add_person(room, "osoba")
    add_company(room, "spolka")

    def address(uczestnik_id, rodzaj):
        return {
            "uczestnik_id": uczestnik_id,
            "rodzaj": rodzaj,
            "kraj": "PL",
            "miejscowosc": "Wrocław",
        }

    room.refused(
        "miejsce zamieszkania ma tylko osoba fizyczna",
        "wpisz_adres_uczestnika",
        **address("spolka", "zamieszkanie"),
    )
    room.refused(
        "miejsce zamieszkania ma tylko osoba fizyczna",
        "wpisz_adres_uczestnika",
        **address("osoba", "siedziba"),
    )
    room.run("wpisz_adres_uczestnika", **address("osoba", "zamieszkanie"))
    room.run("wpisz_adres_uczestnika", **address("spolka", "siedziba"))
    room.run("wpisz_adres_uczestnika", **address("spolka", "do_korespondencji"))
    room.refused_sql(
        "miejsce zamieszkania ma tylko osoba fizyczna",
        """UPDATE sprawa_uczestnicy_adresy SET rodzaj = 'zamieszkanie'
        WHERE uczestnik_id = 'spolka' AND rodzaj = 'do_korespondencji'""",
    )

    room.refused(
        "obywatelstwo ma tylko osoba fizyczna",
        "wpisz_obywatelstwo_uczestnika",
        uczestnik_id="spolka",
        panstwo="PL",
    )
    room.run("wpisz_obywatelstwo_uczestnika", uczestnik_id="osoba", panstwo="UA")
    room.refused_sql(
        "obywatelstwo ma tylko osoba fizyczna",
        "UPDATE sprawa_uczestnicy_obywatelstwa SET uczestnik_id = 'spolka'",
    )


@scenario("Uczestnik: zmiana danych identyfikujących podnosi wersję, kontakt i rodzina nie (D6)")
def participant_version():
    room = new_room()
    add_person(room, "osoba", pesel="80010112345")
    assert version_of(room, "osoba") == 1

    room.run(
        "zmien_kontakt_uczestnika", id="osoba", telefon="+48 600 000 000", kanal_kontaktu="sms"
    )
    room.run(
        "ustal_sytuacje_rodzinna_uczestnika",
        id="osoba",
        stan_cywilny="w_malzenstwie",
        ustroj_majatkowy="wspolnosc_ustawowa",
    )
    room.run(
        "wpisz_adres_uczestnika",
        uczestnik_id="osoba",
        rodzaj="do_korespondencji",
        kraj="PL",
        miejscowosc="Opole",
    )
    assert version_of(room, "osoba") == 1

    room.run("zmien_dane_uczestnika", **person_fields(pesel="80010112345", nazwisko="Nowak"))
    assert version_of(room, "osoba") == 2

    room.run(
        "wpisz_adres_uczestnika",
        uczestnik_id="osoba",
        rodzaj="zamieszkanie",
        kraj="PL",
        miejscowosc="Wrocław",
    )
    room.run("wpisz_obywatelstwo_uczestnika", uczestnik_id="osoba", panstwo="PL")
    assert version_of(room, "osoba") == 4

    # D14: a mistaken address or citizenship may be deleted, and that changes the version too.
    address_id = room.value(
        "SELECT id FROM sprawa_uczestnicy_adresy WHERE rodzaj = 'zamieszkanie'",
    )
    room.run("wykresl_adres_uczestnika", id=address_id)
    citizenship_id = room.value("SELECT id FROM sprawa_uczestnicy_obywatelstwa")
    room.run("wykresl_obywatelstwo_uczestnika", id=citizenship_id)
    assert version_of(room, "osoba") == 6


# Properties --------------------------------------------------------------------------------------


@scenario("Nieruchomość: numer księgi ma format XXXX/NNNNNNNN/N albo nieruchomość nie ma księgi")
def property_land_register():
    room = new_room()

    room.refused(
        "numer_kw_albo_brak_ksiegi", "wpisz_nieruchomosc", **flat_fields(numer_kw="WR1K/123/6")
    )
    room.refused("numer_kw_albo_brak_ksiegi", "wpisz_nieruchomosc", **flat_fields(numer_kw=None))
    room.refused("numer_kw_albo_brak_ksiegi", "wpisz_nieruchomosc", **flat_fields(kw_brak=1))

    add_flat(room)
    add_flat(
        room,
        "spoldzielcze",
        prawo="spoldzielcze_wlasnosciowe_prawo_do_lokalu",
        numer_kw=None,
        kw_brak=1,
    )
    room.refused(
        "numer_kw_albo_brak_ksiegi",
        "zmien_dane_nieruchomosci",
        **flat_fields(id="spoldzielcze", kw_brak=0, numer_kw=None),
    )


def flat_fields(**fields):
    return {
        "id": "lokal",
        "rodzaj": "lokal_mieszkalny",
        "prawo": "odrebna_wlasnosc_lokalu",
        "miejscowosc": "Wrocław",
        "gmina": "Wrocław",
        "numer_kw": "WR1K/00012345/6",
        "kw_brak": 0,
        **fields,
    }


@scenario("Nieruchomość: pomieszczenie przynależne wskazuje lokal; działkę wolno wykreślić (D14)")
def property_appurtenant_room():
    room = new_room()
    add_flat(room)
    add_flat(room, "dom", rodzaj="dom", prawo="wlasnosc", numer_kw="WR1K/00054321/0")

    room.refused(
        "pomieszczenie_przynalezne_wskazuje_lokal",
        "wpisz_nieruchomosc",
        **flat_fields(id="piwnica", rodzaj="pomieszczenie_przynalezne"),
    )
    room.refused(
        "pomieszczenie_przynalezne_wskazuje_lokal",
        "wpisz_nieruchomosc",
        **flat_fields(id="garaz", rodzaj="garaz", nadrzedna_id="lokal"),
    )
    room.refused(
        "pomieszczenie przynależne wskazuje lokal",
        "wpisz_nieruchomosc",
        **flat_fields(id="piwnica", rodzaj="pomieszczenie_przynalezne", nadrzedna_id="dom"),
    )
    room.run(
        "wpisz_nieruchomosc",
        **flat_fields(id="piwnica", rodzaj="pomieszczenie_przynalezne", nadrzedna_id="lokal"),
    )
    room.refused(
        "pomieszczenie przynależne wskazuje lokal",
        "zmien_dane_nieruchomosci",
        **flat_fields(id="piwnica", rodzaj="pomieszczenie_przynalezne", nadrzedna_id="dom"),
    )

    # The workaround from the audit: the flat of an appurtenant room turns into a house.
    room.refused_sql(
        "rodzaj nieruchomości się nie zmienia",
        "UPDATE sprawa_nieruchomosci_wpisy SET rodzaj = 'dom' WHERE id = 'lokal'",
    )
    room.run(
        "zmien_dane_nieruchomosci",
        **flat_fields(id="lokal", numer_lokalu="12A"),
    )

    room.run("wpisz_dzialke", nieruchomosc_id="dom", obreb="0012", numer_dzialki="45/7")
    plot_id = room.value("SELECT id FROM sprawa_nieruchomosci_dzialki")
    room.run("zmien_dzialke", id=plot_id, obreb="0012", numer_dzialki="45/8", powierzchnia=812)
    room.run("wykresl_dzialke", id=plot_id)
    assert room.run("dzialki") == []


# Documents ---------------------------------------------------------------------------------------


@scenario("Dokument: powstaje wymagany, zamówiony albo otrzymany; otrzymany ma plik ze skrótem")
def document_states_and_file():
    room = new_room()

    room.refused_sql(
        "dokument powstaje jako wymagany, zamówiony albo otrzymany",
        """INSERT INTO sprawa_dokumenty_wpisy (id, rodzaj, nazwa, stan, sciezka, skrot)
        VALUES ('od-razu', 'inny', 'Dokument', 'sprawdzony', 'd.pdf', 'sha256-d')""",
    )

    room.run(
        "wpisz_wymagany_dokument",
        id="swiadectwo",
        rodzaj="swiadectwo_energetyczne",
        nazwa="Świadectwo charakterystyki energetycznej",
        wymaga="przepis",
    )
    room.refused(
        "otrzymany_ma_plik",
        "odnotuj_otrzymanie_dokumentu",
        id="swiadectwo",
        sciezka=None,
        skrot=None,
    )
    room.refused(
        "plik_ze_skrotem",
        "odnotuj_otrzymanie_dokumentu",
        id="swiadectwo",
        sciezka="dokumenty/swiadectwo.pdf",
    )
    room.run(
        "odnotuj_otrzymanie_dokumentu",
        id="swiadectwo",
        wystawca="inż. Jan Audytor",
        data_wystawienia="2020-03-01",
        waznosc_do="2030-03-01",
        waznosc_zrodlo="przepis",
        sciezka="dokumenty/swiadectwo.pdf",
        skrot="sha256-swiadectwo",
    )
    room.run("oznacz_dokument_jako_sprawdzony", id="swiadectwo")


@scenario("Dokument: odrzucony ma powód, ważność ma źródło i nie jest wcześniejsza niż wystawienie")
def document_rejection_and_validity():
    room = new_room()
    add_received_document(room, "zaswiadczenie", "zaswiadczenie")

    room.refused_sql(
        "odrzucony_ma_powod",
        "UPDATE sprawa_dokumenty_wpisy SET stan = 'odrzucony' WHERE id = 'zaswiadczenie'",
    )
    room.run("odrzuc_dokument", id="zaswiadczenie", powod_odrzucenia="nieczytelna pieczęć")

    room.refused(
        "waznosc_nie_przed_wystawieniem",
        "wpisz_otrzymany_dokument",
        id="przeterminowane",
        rodzaj="zaswiadczenie",
        nazwa="Zaświadczenie spółdzielni",
        data_wystawienia="2026-11-03",
        waznosc_do="2026-11-01",
        waznosc_zrodlo="instytucja",
        sciezka="dokumenty/z.pdf",
        skrot="sha256-z",
    )
    room.refused(
        "waznosc_ze_zrodlem",
        "wpisz_otrzymany_dokument",
        id="bez-zrodla",
        rodzaj="zaswiadczenie",
        nazwa="Zaświadczenie spółdzielni",
        waznosc_do="2026-12-03",
        sciezka="dokumenty/z.pdf",
        skrot="sha256-z",
    )
    add_received_document(
        room,
        "wazne",
        "zaswiadczenie",
        data_wystawienia="2026-11-03",
        waznosc_do="2026-12-03",
        waznosc_zrodlo="instytucja",
    )


@scenario("Dokument: plik się nie zmienia, także skan dowodu (D16)")
def document_file_immutable():
    room = new_room()
    add_person(room, "klientka")
    add_received_document(room, "skan-dowodu", "tozsamosc", dotyczy_uczestnik_id="klientka")

    for sciezka, skrot in (
        ("dokumenty/inny.pdf", "sha256-skan-dowodu"),
        ("dokumenty/skan-dowodu.pdf", "sha256-inny"),
        (None, None),
    ):
        room.refused_sql(
            "plik dokumentu się nie zmienia",
            "UPDATE sprawa_dokumenty_wpisy SET sciezka = :sciezka, skrot = :skrot",
            sciezka=sciezka,
            skrot=skrot,
        )

    # The first file of a required document is not a change.
    room.run("wpisz_wymagany_dokument", id="odpis", rodzaj="odpis_rejestru", nazwa="Odpis z KRS")
    room.run(
        "odnotuj_otrzymanie_dokumentu", id="odpis", sciezka="dokumenty/odpis.pdf", skrot="sha256-o"
    )


@scenario("Dokument: regulamin, standard, instrukcja i uchwała mają własne rodzaje")
def document_kinds_of_a_flat():
    room = new_room()

    for rodzaj in ("regulamin", "standard", "instrukcja", "uchwala"):
        add_received_document(room, rodzaj, rodzaj)

    room.refused(
        "rodzaj IN",
        "wpisz_wymagany_dokument",
        id="nieznany",
        rodzaj="uchwala_wspolnoty",
        nazwa="Uchwała wspólnoty",
    )


@scenario("Dokument: zbędny staje się tylko wymagany albo zamówiony")
def document_unneeded():
    room = new_room()
    room.run(
        "wpisz_wymagany_dokument", id="wymagany", rodzaj="zaswiadczenie", nazwa="Zaświadczenie"
    )
    room.run("wpisz_wymagany_dokument", id="zamowiony", rodzaj="zaswiadczenie", nazwa="Odpis")
    room.run("odnotuj_zamowienie_dokumentu", id="zamowiony")
    add_received_document(room, "otrzymany")

    room.run("uznaj_dokument_za_zbedny", id="wymagany")
    room.run("uznaj_dokument_za_zbedny", id="zamowiony")
    room.refused(
        "zbędny staje się tylko dokument wymagany albo zamówiony",
        "uznaj_dokument_za_zbedny",
        id="otrzymany",
    )


@scenario("Dokument dotyczy uczestnika albo nieruchomości; źródło wskazuje dokument albo osobę")
def document_subject_and_source():
    room = new_room()
    add_person(room, "klientka")
    add_flat(room)

    room.refused(
        "dotyczy_uczestnika_albo_nieruchomosci",
        "wpisz_wymagany_dokument",
        id="dokument",
        rodzaj="inny",
        nazwa="Dokument",
        dotyczy_uczestnik_id="klientka",
        dotyczy_nieruchomosc_id="lokal",
    )
    add_received_document(
        room, "akt-malzenstwa", "akt_stanu_cywilnego", dotyczy_uczestnik_id="klientka"
    )

    source = {
        "dotyczy_tabela": "sprawa_uczestnicy_wpisy",
        "dotyczy_id": "klientka",
        "pole": "stan_cywilny",
        "potwierdzone": 0,
        "wpisano": "2026-11-03T09:00:00Z",
    }
    room.refused("zrodlo_wskazuje_dokument_albo_osobe", "wpisz_zrodlo", **source)
    room.refused(
        "zrodlo_wskazuje_dokument_albo_osobe",
        "wpisz_zrodlo",
        **source,
        dokument_id="akt-malzenstwa",
        od_uczestnik_id="klientka",
    )

    room.run("wpisz_zrodlo", **source, od_uczestnik_id="klientka")
    room.run("wpisz_zrodlo", **source, dokument_id="akt-malzenstwa")
    room.run("wpisz_zrodlo", **{**source, "pole": "telefon"}, od_kogo="zarządca budynku")

    oral_source_id = room.value(
        "SELECT id FROM sprawa_dokumenty_zrodla WHERE od_uczestnik_id IS NOT NULL"
    )
    room.run("potwierdz_zrodlo", id=oral_source_id)
    rows = room.run(
        "zrodla_wiersza", dotyczy_tabela="sprawa_uczestnicy_wpisy", dotyczy_id="klientka"
    )
    assert len(rows) == 3
