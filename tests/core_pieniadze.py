"""Scenarios of the core kinds around clients' money (spec/cases/sprawa): accounts
(sprawa-rachunki), Kontroler's checks (sprawa-sprawdzenia) and payouts (sprawa-wyplaty).

Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
allows and a write it refuses. Together they follow R4 and D13: a new account number is a new row,
a human admits an account only after Kontroler's check and a confirmation by another channel, and
every payout waits for a fresh check of the account and for the human's consent.
"""

from core import (
    add_account,
    add_flat,
    add_person,
    add_power_for_office,
    add_received_document,
    admitted_account,
    approve,
    approved_decision,
    ask_for_decision,
    check,
    checked_and_approved_payout,
    confirm_account,
    new_room,
    open_case,
    prepare_payout,
    version_of,
)

SCENARIOS = []


def scenario(title):
    def register(function):
        SCENARIOS.append((title, function))

        return function

    return register


def signature(podpisano):
    return {
        "podpisal": "tozsamosc-posrednika",
        "podpis": "podpis-kluczem-xkeys",
        "podpisano": podpisano,
    }


# Accounts ----------------------------------------------------------------------------------------


@scenario(
    "Rachunek: numer, posiadacz, uczestnik, cel i źródło się nie zmieniają; nowy numer to nowy wiersz"
)
def account_immutable():
    room = new_room()
    add_person(room, "sprzedajaca")
    add_account(room, "rachunek", "sprzedajaca", cel="zaplata_ceny")

    for column, value in (
        ("numer", "PL27114020040000300201355387"),
        ("posiadacz", "Ktoś Inny"),
        ("cel", "zadatek"),
        ("uczestnik_id", "sprzedajaca-2"),
    ):
        room.refused_sql(
            "numer, posiadacz, uczestnik, cel i źródło rachunku się nie zmieniają",
            f"UPDATE sprawa_rachunki_wpisy SET {column} = :value",
            value=value,
        )

    # Two days before the act a message "from the seller" brings a new number: a new row.
    add_account(
        room,
        "nowy-rachunek",
        "sprzedajaca",
        numer="PL27114020040000300201355387",
        cel="zaplata_ceny",
    )
    room.refused(
        "zastapienie_ma_autora_i_date",
        "zastap_rachunek",
        id="rachunek",
        zastapiony_przez_id="nowy-rachunek",
        zastapil="tozsamosc-agenta",
        zastapiono=None,
    )
    room.run(
        "zastap_rachunek",
        id="rachunek",
        zastapiony_przez_id="nowy-rachunek",
        zastapil="tozsamosc-agenta",
        zastapiono="2026-11-08T09:00:00Z",
    )
    assert [row["id"] for row in room.run("rachunki_biezace")] == ["nowy-rachunek"]

    room.refused(
        "zastąpionego rachunku się nie przywraca",
        "zastap_rachunek",
        id="rachunek",
        zastapiony_przez_id="nowy-rachunek",
        zastapil="ktos-inny",
        zastapiono="2026-11-09T09:00:00Z",
    )


@scenario("Rachunek ma jedno źródło: dokument albo wiadomość; potwierdzenie jest w komplecie")
def account_source_and_confirmation():
    room = new_room()
    add_person(room, "najemca")
    add_received_document(room, "umowa-najmu", "umowa")

    account = {
        "uczestnik_id": "najemca",
        "numer": "PL61109010140000071219812874",
        "posiadacz": "Jan Nowak",
        "cel": "zwrot_kaucji",
        "wpisano": "2026-11-03T08:00:00Z",
    }
    room.refused("rachunek_ma_zrodlo", "wpisz_rachunek", id="bez-zrodla", **account)
    room.run("wpisz_rachunek", id="rachunek", zrodlo_dokument_id="umowa-najmu", **account)

    room.refused(
        "potwierdzenie_w_komplecie",
        "odnotuj_potwierdzenie_rachunku",
        id="rachunek",
        potwierdzil="tozsamosc-agenta",
        potwierdzono="2026-11-03T08:30:00Z",
    )
    confirm_account(room, "rachunek")


# Checks ------------------------------------------------------------------------------------------


@scenario("Sprawdzenie: rodzaj pasuje do wiersza, a wynik inny niż „zgodne” ma opis")
def check_kind_matches_row():
    room = new_room()
    add_person(room, "klientka")
    add_flat(room)
    add_account(room, "rachunek", "klientka")

    def write(check_id, rodzaj, wersja_wiersza=None, wynik="zgodne", opis=None, **target):
        return {
            "id": check_id,
            "rodzaj": rodzaj,
            "wersja_wiersza": wersja_wiersza,
            "zrodlo": "odpis",
            "wynik": wynik,
            "opis": opis,
            "sprawdzil": "tozsamosc-kontrolera",
            "sprawdzono": "2026-11-03T09:00:00Z",
            **target,
        }

    for refused in (
        write("a", "tozsamosc", 1, rachunek_id="rachunek"),
        write("b", "tozsamosc", 1, uczestnik_id="klientka", nieruchomosc_id="lokal"),
        write("c", "tytul", 1, uczestnik_id="klientka"),
        write("d", "rachunek", 1, uczestnik_id="klientka"),
        write("e", "umocowanie", 1, rachunek_id="rachunek"),
    ):
        room.refused("rodzaj_pasuje_do_wiersza", "wpisz_sprawdzenie", **refused)

    room.refused(
        "wersja_wiersza_przy_wierszu_z_wersja",
        "wpisz_sprawdzenie",
        **write("f", "rachunek", 1, rachunek_id="rachunek"),
    )
    room.refused(
        "wynik_z_uwagami_ma_opis",
        "wpisz_sprawdzenie",
        **write("g", "tytul", 1, wynik="uwagi", uczestnik_id="klientka", nieruchomosc_id="lokal"),
    )

    check(room, "tytul", "tytul", uczestnik_id="klientka", nieruchomosc_id="lokal")
    check(room, "rachunku", "rachunek", wynik="nie_mozna_ustalic", rachunek_id="rachunek")


@scenario("Sprawdzenie powstaje aktualne, dotyczy bieżącej wersji wiersza i się nie zmienia")
def check_current_version():
    room = new_room()
    add_person(room, "klientka")

    room.refused_sql(
        "sprawdzenie powstaje jako aktualne",
        """INSERT INTO sprawa_sprawdzenia_wpisy (id, rodzaj, uczestnik_id, wersja_wiersza, zrodlo,
          wynik, sprawdzil, sprawdzono, aktualne)
        VALUES ('a', 'tozsamosc', 'klientka', 1, 'dowód', 'zgodne', 'k', '2026-11-03T09:00:00Z', 0)""",
    )
    room.refused(
        "sprawdzenie dotyczy bieżącej wersji wiersza",
        "wpisz_sprawdzenie",
        id="stara-wersja",
        rodzaj="tozsamosc",
        uczestnik_id="klientka",
        wersja_wiersza=2,
        zrodlo="dowód osobisty okazany w biurze",
        wynik="zgodne",
        sprawdzil="tozsamosc-kontrolera",
        sprawdzono="2026-11-03T09:00:00Z",
    )

    check(room, "tozsamosc", "tozsamosc", uczestnik_id="klientka")
    room.refused_sql(
        "wyniku sprawdzenia się nie zmienia",
        "UPDATE sprawa_sprawdzenia_wpisy SET wynik = 'uwagi', opis = 'brak'",
    )
    room.run(
        "wpisz_adres_uczestnika",
        uczestnik_id="klientka",
        rodzaj="zamieszkanie",
        kraj="PL",
        miejscowosc="Wrocław",
    )
    room.refused_sql(
        "wyniku sprawdzenia się nie zmienia",
        "UPDATE sprawa_sprawdzenia_wpisy SET aktualne = 1",
    )


@scenario("Sprawdzenie: nowe sprawdzenie tego samego wiersza odbiera aktualność poprzedniemu")
def check_replaced_by_new_one():
    room = new_room()
    add_person(room, "klientka")
    add_person(room, "druga", imiona="Ewa")
    add_account(room, "rachunek", "klientka")

    check(room, "pierwsze", "rachunek", rachunek_id="rachunek")
    check(room, "tozsamosc-drugiej", "tozsamosc", uczestnik_id="druga")
    check(room, "tozsamosc-klientki", "tozsamosc", uczestnik_id="klientka")
    check(room, "drugie", "rachunek", sprawdzono="2026-11-04T09:00:00Z", rachunek_id="rachunek")

    aktualne = {row["id"] for row in room.run("sprawdzenia_aktualne")}
    assert aktualne == {"drugie", "tozsamosc-drugiej", "tozsamosc-klientki"}, aktualne


@scenario(
    "Sprawdzenie: zmiana danych identyfikujących uczestnika albo umocowania odbiera aktualność (D6)"
)
def check_lost_on_data_change():
    room = new_room()
    add_person(room, "klientka")
    add_flat(room)
    add_power_for_office(room, "pelnomocnictwo", "klientka")
    check(room, "tozsamosc", "tozsamosc", uczestnik_id="klientka")
    check(room, "tytul", "tytul", uczestnik_id="klientka", nieruchomosc_id="lokal")
    check(room, "umocowania", "umocowanie", umocowanie_id="pelnomocnictwo")

    # Contact data do not identify the person, so the checks stay current.
    room.run(
        "zmien_kontakt_uczestnika", id="klientka", email="anna@example.pl", kanal_kontaktu="email"
    )
    assert len(room.run("sprawdzenia_aktualne")) == 3

    room.run("wpisz_obywatelstwo_uczestnika", uczestnik_id="klientka", panstwo="PL")
    assert [row["id"] for row in room.run("sprawdzenia_aktualne")] == ["umocowania"]
    assert version_of(room, "klientka") == 2

    room.run("dopisz_czynnosc_umocowania", umocowanie_id="pelnomocnictwo", czynnosc="odbior_odpisu")
    assert room.run("sprawdzenia_aktualne") == []

    check(
        room, "tozsamosc-2", "tozsamosc", sprawdzono="2026-11-05T09:00:00Z", uczestnik_id="klientka"
    )
    assert [row["id"] for row in room.run("sprawdzenia_aktualne")] == ["tozsamosc-2"]


@scenario(
    "Zgoda na rachunek wymaga aktualnego sprawdzenia „zgodne”, potwierdzenia i niezastąpionego rachunku"
)
def account_admission_needs_check():
    room = new_room()
    add_person(room, "najemca")
    refusal = "zgoda na rachunek wymaga aktualnego sprawdzenia"

    def admit(decision_id, account_id):
        ask_for_decision(
            room, decision_id, "dopuszczenie_rachunku", "sprawa_rachunki_wpisy", account_id
        )
        room.refused(
            refusal,
            "rozstrzygnij_decyzje",
            id=decision_id,
            rozstrzygniecie="zgoda",
            **signature("2026-11-03T12:00:00Z"),
        )

    # Not confirmed by another channel.
    add_account(room, "niepotwierdzony", "najemca")
    check(room, "sprawdzenie-niepotwierdzonego", "rachunek", rachunek_id="niepotwierdzony")
    admit("zgoda-niepotwierdzony", "niepotwierdzony")

    # Not checked, or checked with remarks.
    add_account(room, "niesprawdzony", "najemca", numer="PL02")
    confirm_account(room, "niesprawdzony")
    admit("zgoda-niesprawdzony", "niesprawdzony")
    check(room, "z-uwagami", "rachunek", wynik="uwagi", rachunek_id="niesprawdzony")
    admit("zgoda-z-uwagami", "niesprawdzony")

    # Replaced by a new number.
    admitted_account(room, "stary", "najemca")
    add_account(room, "nowy", "najemca", numer="PL03")
    room.run(
        "zastap_rachunek",
        id="stary",
        zastapiony_przez_id="nowy",
        zastapil="tozsamosc-agenta",
        zastapiono="2026-11-04T09:00:00Z",
    )
    admit("zgoda-zastapiony", "stary")

    # A refusal is always allowed; a confirmed, checked, current account gets the consent.
    room.run(
        "rozstrzygnij_decyzje",
        id="zgoda-zastapiony",
        rozstrzygniecie="odmowa",
        uzasadnienie="numer zmieniony dwa dni przed aktem",
        **signature("2026-11-03T12:00:00Z"),
    )
    admitted_account(room, "dobry", "najemca")


# Payouts -----------------------------------------------------------------------------------------


@scenario("Wypłata powstaje przygotowana na rachunek odbiorcy dopuszczony decyzją i niezastąpiony")
def payout_to_admitted_account():
    room = new_room()
    add_person(room, "najemca")
    add_person(room, "wlascicielka", imiona="Ewa")
    refusal = "wypłata idzie na rachunek odbiorcy dopuszczony decyzją i niezastąpiony"

    add_account(room, "niedopuszczony", "najemca")
    room.refused(refusal, "przygotuj_wyplate", **payout_fields("w-1", "najemca", "niedopuszczony"))

    admitted_account(room, "rachunek-najemcy", "najemca")
    room.refused(
        refusal, "przygotuj_wyplate", **payout_fields("w-2", "wlascicielka", "rachunek-najemcy")
    )

    room.refused_sql(
        "wypłata powstaje jako przygotowana, bez sprawdzenia i zgody",
        """INSERT INTO sprawa_wyplaty_wpisy (id, podstawa_tabela, podstawa_id, kwota,
          wyliczenie_sciezka, wyliczenie_skrot, odbiorca_uczestnik_id, rachunek_id, utworzono, stan,
          sprawdzenie_id)
        VALUES ('w-3', 'najem_kaucje', 'r', 100, 'w.json', 'sha256-w', 'najemca', 'rachunek-najemcy',
          '2026-11-20T09:00:00Z', 'przygotowana', 'sprawdzenie-rachunek-najemcy')""",
    )

    prepare_payout(room, "zwrot-kaucji", "najemca", "rachunek-najemcy")

    add_account(room, "nowy", "najemca", numer="PL03")
    room.run(
        "zastap_rachunek",
        id="rachunek-najemcy",
        zastapiony_przez_id="nowy",
        zastapil="tozsamosc-agenta",
        zastapiono="2026-11-20T09:30:00Z",
    )
    room.refused(
        refusal, "przygotuj_wyplate", **payout_fields("w-4", "najemca", "rachunek-najemcy")
    )
    room.refused_sql(
        "treść wypłaty się nie zmienia",
        "UPDATE sprawa_wyplaty_wpisy SET rachunek_id = 'nowy' WHERE id = 'zwrot-kaucji'",
    )


def payout_fields(payout_id, person_id, account_id):
    return {
        "id": payout_id,
        "podstawa_tabela": "najem_kaucje",
        "podstawa_id": "rozliczenie-kaucji",
        "kwota": 300_000,
        "wyliczenie_sciezka": f"wyliczenia/{payout_id}.json",
        "wyliczenie_skrot": f"sha256-{payout_id}",
        "odbiorca_uczestnik_id": person_id,
        "rachunek_id": account_id,
        "utworzono": "2026-11-20T09:00:00Z",
    }


@scenario("Wypłata: Kontroler sprawdza rachunek na nowo po przygotowaniu wypłaty (D13)")
def payout_check_after_preparation():
    room = new_room()
    add_person(room, "najemca")
    admitted_account(room, "rachunek-najemcy", "najemca")
    add_account(room, "inny", "najemca", numer="PL04")
    check(
        room,
        "sprawdzenie-innego",
        "rachunek",
        sprawdzono="2026-11-20T10:00:00Z",
        rachunek_id="inny",
    )
    prepare_payout(
        room, "zwrot-kaucji", "najemca", "rachunek-najemcy", utworzono="2026-11-20T09:00:00Z"
    )
    refusal = (
        "wypłata wskazuje aktualne sprawdzenie „zgodne” jej rachunku zrobione po przygotowaniu"
    )

    # The check made when the account was admitted is older than the payout.
    room.refused(
        refusal,
        "wskaz_sprawdzenie_wyplaty",
        id="zwrot-kaucji",
        sprawdzenie_id="sprawdzenie-rachunek-najemcy",
    )
    room.refused(
        refusal, "wskaz_sprawdzenie_wyplaty", id="zwrot-kaucji", sprawdzenie_id="sprawdzenie-innego"
    )
    check(
        room,
        "z-uwagami",
        "rachunek",
        sprawdzono="2026-11-20T10:00:00Z",
        wynik="uwagi",
        rachunek_id="rachunek-najemcy",
    )
    room.refused(
        refusal, "wskaz_sprawdzenie_wyplaty", id="zwrot-kaucji", sprawdzenie_id="z-uwagami"
    )

    check(
        room,
        "swieze",
        "rachunek",
        sprawdzono="2026-11-20T10:30:00Z",
        rachunek_id="rachunek-najemcy",
    )
    room.run("wskaz_sprawdzenie_wyplaty", id="zwrot-kaucji", sprawdzenie_id="swieze")


@scenario("Wypłata: zgoda człowieka zapada po sprawdzeniu rachunku i wskazuje tę wypłatę")
def payout_consent_after_check():
    room = new_room()
    add_person(room, "najemca")
    admitted_account(room, "rachunek-najemcy", "najemca")
    prepare_payout(room, "zwrot-kaucji", "najemca", "rachunek-najemcy")
    prepare_payout(room, "inna-wyplata", "najemca", "rachunek-najemcy")
    refusal = "zgoda na wypłatę zapada po sprawdzeniu rachunku, które wypłata wskazuje"

    ask_for_decision(
        room,
        "za-wczesnie",
        "wyplata",
        "sprawa_wyplaty_wpisy",
        "zwrot-kaucji",
        zlozono="2026-11-20T09:30:00Z",
    )
    room.refused(
        refusal,
        "rozstrzygnij_decyzje",
        id="za-wczesnie",
        rozstrzygniecie="zgoda",
        **signature("2026-11-20T09:45:00Z"),
    )

    check(
        room,
        "swieze",
        "rachunek",
        sprawdzono="2026-11-20T10:00:00Z",
        rachunek_id="rachunek-najemcy",
    )
    room.run("wskaz_sprawdzenie_wyplaty", id="zwrot-kaucji", sprawdzenie_id="swieze")
    room.refused(
        refusal,
        "rozstrzygnij_decyzje",
        id="za-wczesnie",
        rozstrzygniecie="zgoda",
        **signature("2026-11-20T09:45:00Z"),
    )
    approve(room, "za-wczesnie", "2026-11-20T11:00:00Z")

    approved_decision(
        room,
        "na-inna",
        "inna",
        "sprawa_wyplaty_wpisy",
        "inna-wyplata",
        zlozono="2026-11-20T11:00:00Z",
        podpisano="2026-11-20T11:30:00Z",
    )
    room.refused(
        "wypłata wskazuje zatwierdzoną decyzję wyplata, która ją wskazuje",
        "wskaz_zgode_na_wyplate",
        id="zwrot-kaucji",
        decyzja_id="na-inna",
    )
    room.run("wskaz_zgode_na_wyplate", id="zwrot-kaucji", decyzja_id="za-wczesnie")


@scenario("Wypłata: wykonanie wymaga zgody, potwierdzenia z banku i wciąż aktualnego rachunku")
def payout_execution():
    room = new_room()
    add_person(room, "najemca")
    admitted_account(room, "rachunek-najemcy", "najemca")
    add_received_document(room, "potwierdzenie", "potwierdzenie_bankowe")
    executed = {
        "id": "zwrot-kaucji",
        "wykonal": "tozsamosc-posrednika",
        "wykonano": "2026-11-20T13:00:00Z",
        "potwierdzenie_dokument_id": "potwierdzenie",
    }
    refusal = "wykonanie wymaga zgody na wypłatę po aktualnym sprawdzeniu niezastąpionego i dopuszczonego rachunku"

    prepare_payout(room, "bez-zgody", "najemca", "rachunek-najemcy")
    room.refused(refusal, "odnotuj_wykonanie_wyplaty", **{**executed, "id": "bez-zgody"})
    room.run("anuluj_wyplate", id="bez-zgody")

    checked_and_approved_payout(room, "zwrot-kaucji", "najemca", "rachunek-najemcy")
    room.refused(
        "wykonana_ma_wykonawce_i_potwierdzenie",
        "odnotuj_wykonanie_wyplaty",
        **{**executed, "potwierdzenie_dokument_id": None},
    )

    # A newer check of the account, e.g. with remarks, takes the consent's ground away.
    check(
        room,
        "nowsze",
        "rachunek",
        sprawdzono="2026-11-20T12:30:00Z",
        wynik="uwagi",
        rachunek_id="rachunek-najemcy",
    )
    room.refused(refusal, "odnotuj_wykonanie_wyplaty", **executed)

    check(
        room,
        "jeszcze-nowsze",
        "rachunek",
        sprawdzono="2026-11-20T12:40:00Z",
        rachunek_id="rachunek-najemcy",
    )
    room.refused(refusal, "odnotuj_wykonanie_wyplaty", **executed)

    prepare_payout(room, "druga", "najemca", "rachunek-najemcy")
    checked_and_approved_payout(room, "trzecia", "najemca", "rachunek-najemcy")
    room.run("odnotuj_wykonanie_wyplaty", **{**executed, "id": "trzecia"})
    room.refused(
        "wykonanej ani anulowanej wypłaty się nie zmienia",
        "anuluj_wyplate",
        id="trzecia",
    )
    room.run("anuluj_wyplate", id="druga")
    room.refused(
        "wykonanej ani anulowanej wypłaty się nie zmienia",
        "odnotuj_wykonanie_wyplaty",
        **{**executed, "id": "druga"},
    )


@scenario("Wypłata: wykonanie po zastąpieniu rachunku jest odmówione")
def payout_execution_after_account_replaced():
    room = new_room()
    add_person(room, "najemca")
    admitted_account(room, "rachunek-najemcy", "najemca")
    checked_and_approved_payout(room, "zwrot-kaucji", "najemca", "rachunek-najemcy")
    add_received_document(room, "potwierdzenie", "potwierdzenie_bankowe")
    add_account(room, "nowy", "najemca", numer="PL05")
    room.run(
        "zastap_rachunek",
        id="rachunek-najemcy",
        zastapiony_przez_id="nowy",
        zastapil="tozsamosc-agenta",
        zastapiono="2026-11-20T12:30:00Z",
    )

    room.refused(
        "wykonanie wymaga zgody na wypłatę po aktualnym sprawdzeniu niezastąpionego i dopuszczonego rachunku",
        "odnotuj_wykonanie_wyplaty",
        id="zwrot-kaucji",
        wykonal="tozsamosc-posrednika",
        wykonano="2026-11-20T13:00:00Z",
        potwierdzenie_dokument_id="potwierdzenie",
    )
    room.run("anuluj_wyplate", id="zwrot-kaucji")


@scenario("Teczki nie zamyka się, dopóki jakaś wypłata jest przygotowana (D19)")
def closing_after_payouts():
    room = new_room()
    open_case(room)
    add_person(room, "najemca")
    admitted_account(room, "rachunek-najemcy", "najemca")
    prepare_payout(room, "zwrot-kaucji", "najemca", "rachunek-najemcy")

    room.refused(
        "teczki nie zamyka się, dopóki jakaś wypłata jest przygotowana",
        "zmien_etap_sprawy",
        etap="zamknieta",
        od="2026-11-30T09:00:00Z",
        zmienil="agent",
    )

    room.run("anuluj_wyplate", id="zwrot-kaucji")
    room.run("zmien_etap_sprawy", etap="zamknieta", od="2026-11-30T09:00:00Z", zmienil="agent")


@scenario("Dopóki stoi znacznik wątpliwości, nie ma zgody na termin, rachunek ani wypłatę")
def doubt_blocks_consents():
    room = new_room()
    open_case(room)
    add_person(room, "najemca")
    admitted_account(room, "rachunek-najemcy", "najemca")
    prepare_payout(room, "zwrot-kaucji", "najemca", "rachunek-najemcy")
    check(
        room,
        "swieze",
        "rachunek",
        sprawdzono="2026-11-20T10:00:00Z",
        rachunek_id="rachunek-najemcy",
    )
    room.run("wskaz_sprawdzenie_wyplaty", id="zwrot-kaucji", sprawdzenie_id="swieze")
    add_account(room, "drugi", "najemca", numer="PL06")
    confirm_account(room, "drugi")
    check(
        room,
        "sprawdzenie-drugiego",
        "rachunek",
        sprawdzono="2026-11-20T10:00:00Z",
        rachunek_id="drugi",
    )
    room.run(
        "wpisz_termin",
        id="termin-aktu",
        rodzaj="z_umowy",
        co="Akt notarialny",
        koniec="2026-12-01T10:00:00Z",
        podstawa="§ 2 umowy przedwstępnej",
        pilnuje_stanowisko="koordynator_transakcji",
    )
    room.run(
        "postaw_watpliwosc",
        id="watpliwosc",
        postawil="tozsamosc-kontrolera",
        postawiono="2026-11-20T10:30:00Z",
    )

    requests = (
        ("termin", "potwierdzenie_terminu", "sprawa_terminy_wpisy", "termin-aktu"),
        ("rachunek", "dopuszczenie_rachunku", "sprawa_rachunki_wpisy", "drugi"),
        ("wyplata", "wyplata", "sprawa_wyplaty_wpisy", "zwrot-kaucji"),
    )
    for decision_id, rodzaj, tabela, wiersz in requests:
        ask_for_decision(room, decision_id, rodzaj, tabela, wiersz, zlozono="2026-11-20T11:00:00Z")
        room.refused(
            "dopóki stoi znacznik wątpliwości, nie ma zgody na termin, rachunek ani wypłatę",
            "rozstrzygnij_decyzje",
            id=decision_id,
            rozstrzygniecie="zgoda",
            **signature("2026-11-20T12:00:00Z"),
        )

    # A refusal is always allowed while the doubt stands.
    ask_for_decision(room, "odmowa", "potwierdzenie_terminu", "sprawa_terminy_wpisy", "termin-aktu")
    approve(room, "odmowa", rozstrzygniecie="odmowa")

    ask_for_decision(
        room,
        "zdjecie",
        "zdjecie_watpliwosci",
        "sprawa_metryka_watpliwosci",
        "watpliwosc",
        zlozono="2026-11-20T11:00:00Z",
    )
    approve(room, "zdjecie", "2026-11-20T11:30:00Z")
    room.run("zdejmij_watpliwosc", id="watpliwosc", zdjecie_decyzja_id="zdjecie")
    for decision_id, _rodzaj, _tabela, _wiersz in requests:
        approve(room, decision_id, "2026-11-20T12:00:00Z")


@scenario("Rachunek: potwierdzenie wpisuje się raz, a zastępca ma tego samego uczestnika i cel")
def account_confirmation_once_and_replacement_of_same_kind():
    room = new_room()
    add_person(room, "najemca")
    add_person(room, "firma-najemcy", imiona="Piotr")
    admitted_account(room, "kaucja", "najemca")

    for assignment in (
        "potwierdzil = NULL, potwierdzono = NULL, potwierdzenie_sposob = NULL",
        "potwierdzenie_sposob = 'mailem z adresu z wiadomości z numerem'",
    ):
        room.refused_sql(
            "potwierdzenie posiadacza rachunku wpisuje się raz",
            f"UPDATE sprawa_rachunki_wpisy SET {assignment} WHERE id = 'kaucja'",
        )

    add_account(room, "czynsz-firmy", "firma-najemcy", numer="PL09", cel="czynsz")
    add_account(room, "czynsz-najemcy", "najemca", numer="PL10", cel="czynsz")
    add_account(room, "nowa-kaucja", "najemca", numer="PL11")
    replacement = {
        "id": "kaucja",
        "zastapil": "tozsamosc-agenta",
        "zastapiono": "2026-11-08T09:00:00Z",
    }
    for other_account in ("czynsz-firmy", "czynsz-najemcy"):
        room.refused(
            "rachunek zastępujący ma tego samego uczestnika i ten sam cel",
            "zastap_rachunek",
            zastapiony_przez_id=other_account,
            **replacement,
        )
    room.refused_sql(
        "rachunek zastępujący ma tego samego uczestnika i ten sam cel",
        """INSERT INTO sprawa_rachunki_wpisy (id, uczestnik_id, numer, posiadacz, cel,
          zrodlo_dokument_id, wpisano, zastapiony_przez_id, zastapil, zastapiono)
        VALUES ('stary', 'najemca', 'PL12', 'Jan Nowak', 'zwrot_kaucji', 'zrodlo-kaucja',
          '2026-11-03T08:00:00Z', 'czynsz-firmy', 'agent', '2026-11-08T09:00:00Z')""",
    )
    room.run("zastap_rachunek", zastapiony_przez_id="nowa-kaucja", **replacement)


@scenario("Wypłata: treść nie zmienia się od wpisu, a sprawdzenie i zgodę wskazuje się raz")
def payout_content_frozen():
    room = new_room()
    add_person(room, "najemca")
    admitted_account(room, "rachunek-najemcy", "najemca")
    checked_and_approved_payout(room, "zwrot-kaucji", "najemca", "rachunek-najemcy")
    refusal = "treść wypłaty się nie zmienia"

    # The workarounds from the audit: a new amount after the consent, and an earlier creation
    # moment that would let an old check of the account count as fresh.
    room.refused_sql(
        refusal,
        """UPDATE sprawa_wyplaty_wpisy
        SET kwota = 999999, wyliczenie_skrot = 'inny', podstawa_id = '9'
        WHERE id = 'zwrot-kaucji'""",
    )
    room.refused_sql(
        refusal,
        """UPDATE sprawa_wyplaty_wpisy
        SET utworzono = '2026-11-01T00:00:00Z'
        WHERE id = 'zwrot-kaucji'""",
    )

    prepare_payout(room, "druga", "najemca", "rachunek-najemcy")
    room.refused_sql(
        refusal,
        "UPDATE sprawa_wyplaty_wpisy SET kwota = 1 WHERE id = 'druga'",
    )

    once = "wypłata wskazuje sprawdzenie i zgodę raz"
    room.refused_sql(
        once,
        "UPDATE sprawa_wyplaty_wpisy SET sprawdzenie_id = NULL WHERE id = 'zwrot-kaucji'",
    )
    room.refused_sql(
        once,
        "UPDATE sprawa_wyplaty_wpisy SET decyzja_id = NULL WHERE id = 'zwrot-kaucji'",
    )

    # Another amount is a cancellation and a new payout.
    room.run("anuluj_wyplate", id="druga")
    prepare_payout(room, "druga-poprawiona", "najemca", "rachunek-najemcy")


@scenario("Zastąpiona zgoda na rachunek albo na wypłatę nie wystarcza do wypłaty")
def superseded_consent_does_not_count():
    room = new_room()
    add_person(room, "najemca")
    admitted_account(room, "rachunek-najemcy", "najemca")
    checked_and_approved_payout(room, "zwrot-kaucji", "najemca", "rachunek-najemcy")
    add_received_document(room, "potwierdzenie", "potwierdzenie_bankowe")

    # The workaround from the audit: a later refusal of the same account replaces its admission.
    ask_for_decision(
        room,
        "cofniecie-dopuszczenia",
        "dopuszczenie_rachunku",
        "sprawa_rachunki_wpisy",
        "rachunek-najemcy",
        zlozono="2026-11-20T12:10:00Z",
        zastepuje_decyzja_id="dopuszczenie-rachunek-najemcy",
    )
    approve(room, "cofniecie-dopuszczenia", "2026-11-20T12:20:00Z", rozstrzygniecie="odmowa")
    assert (
        room.value(
            "SELECT stan FROM sprawa_decyzje_wpisy WHERE id = 'dopuszczenie-rachunek-najemcy'"
        )
        == "zastapiona"
    )

    room.refused(
        "wypłata idzie na rachunek odbiorcy dopuszczony decyzją i niezastąpiony",
        "przygotuj_wyplate",
        id="nowa",
        podstawa_tabela="najem_kaucje",
        podstawa_id="rozliczenie-kaucji",
        kwota=300_000,
        wyliczenie_sciezka="wyliczenia/nowa.json",
        wyliczenie_skrot="sha256-nowa",
        odbiorca_uczestnik_id="najemca",
        rachunek_id="rachunek-najemcy",
        utworzono="2026-11-20T12:30:00Z",
    )
    room.refused(
        "wykonanie wymaga zgody na wypłatę po aktualnym sprawdzeniu niezastąpionego i "
        "dopuszczonego rachunku",
        "odnotuj_wykonanie_wyplaty",
        id="zwrot-kaucji",
        wykonal="tozsamosc-posrednika",
        wykonano="2026-11-20T13:00:00Z",
        potwierdzenie_dokument_id="potwierdzenie",
    )
    room.run("anuluj_wyplate", id="zwrot-kaucji")

    # A refusal that replaces the consent to a payout stops its execution too.
    admitted_account(room, "drugi-rachunek", "najemca")
    checked_and_approved_payout(room, "druga", "najemca", "drugi-rachunek")
    ask_for_decision(
        room,
        "cofniecie-zgody",
        "wyplata",
        "sprawa_wyplaty_wpisy",
        "druga",
        zlozono="2026-11-20T12:10:00Z",
        zastepuje_decyzja_id="zgoda-druga",
    )
    approve(room, "cofniecie-zgody", "2026-11-20T12:20:00Z", rozstrzygniecie="odmowa")
    room.refused(
        "wykonanie wymaga zgody na wypłatę",
        "odnotuj_wykonanie_wyplaty",
        id="druga",
        wykonal="tozsamosc-posrednika",
        wykonano="2026-11-20T13:00:00Z",
        potwierdzenie_dokument_id="potwierdzenie",
    )
