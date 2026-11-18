"""Scenarios of the rental case room (spec/cases/najem on the core).

Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
allows and a write it refuses. The first two scenarios follow a whole rental as the profile
describes it: an ordinary lease the office manages from the contract to the return of the deposit,
and an occasional lease the office only brokers, which ends after the flat is handed over.
"""

from pathlib import Path

from core import (
    add_account,
    add_company,
    add_flat,
    add_inflow,
    add_person,
    add_received_document,
    approve,
    approved_decision,
    ask_for_decision,
    check,
    concluded_contract,
    confirm_account,
    open_case,
    prepare_letter,
    send,
)
from core_porzadek import first_row_copy, unique_keys_of
from room import Room

CASE = Path(__file__).resolve().parents[1] / "spec" / "cases" / "najem"

SCENARIOS = []


def scenario(title):
    def register(function):
        SCENARIOS.append((title, function))

        return function

    return register


def new_room():
    return Room(CASE)


# Building blocks of a rental ----------------------------------------------------------------------

# The management contract of core.concluded_contract is concluded at this moment.
MANAGEMENT_CONCLUDED = "2026-11-04T12:00:00Z"


def open_rental(room):
    open_case(room, typ_pokoju="najem")


def move_to(room, *etapy, decyzja_id=None):
    for etap in etapy:
        room.run(
            "zmien_etap_sprawy",
            etap=etap,
            od="2026-11-05T09:00:00Z",
            zmienil="tozsamosc-opiekuna",
            decyzja_id=decyzja_id,
        )


def refused_stage(room, rule, etap):
    room.refused(rule, "zmien_etap_sprawy", etap=etap, od="2026-11-05", zmienil="agent")


def add_rental_parties(room):
    """The owner, a consumer and a client of the office, and the tenant found for her."""
    add_person(room, "wlascicielka", konsument=1, klient_biura=1)
    add_person(room, "najemca", imiona="Jan", nazwisko="Nowak")
    room.run("nadaj_role_uczestnikowi", uczestnik_id="wlascicielka", rola="wynajmujacy")
    room.run("nadaj_role_uczestnikowi", uczestnik_id="najemca", rola="najemca")


def management(room, kwota_napraw=50_000, przeplyw="przez_biuro"):
    """The management contract with the owner and its first terms."""
    concluded_contract(room, "zarzadzanie", "wlascicielka")
    room.run(
        "wpisz_warunki_zarzadzania",
        umowa_id="zarzadzanie",
        kwota_napraw=kwota_napraw,
        przeplyw=przeplyw,
        od=MANAGEMENT_CONCLUDED,
    )


def prepare_lease(room, lease_id="najem", rodzaj="zwykly", **fields):
    add_received_document(room, f"projekt-{lease_id}", "umowa")
    room.run(
        "przygotuj_umowe_najmu",
        **{
            "id": lease_id,
            "rodzaj": rodzaj,
            "nieruchomosc_id": "lokal",
            "poczatek": "2026-12-01",
            "koniec": "2028-11-30",
            "forma": "pisemna",
            "projekt_dokument_id": f"projekt-{lease_id}",
            "czynsz": 300_000,
            "termin_platnosci": "z góry do 10. dnia miesiąca",
            "kaucja": 600_000,
            **fields,
        },
    )


def add_tenant_declaration(room, lease_id="najem"):
    """The tenant's declaration as a notarial deed, an annex of an occasional lease."""
    add_received_document(room, f"oswiadczenie-{lease_id}", "akt_notarialny")
    room.run(
        "wskaz_oswiadczenie_najemcy",
        id=lease_id,
        oswiadczenie_dokument_id=f"oswiadczenie-{lease_id}",
    )


def add_substitute_flat(room, lease_id="najem"):
    room.run(
        "wpisz_lokal_zastepczy",
        najem_id=lease_id,
        adres="ul. Lipowa 4 m. 2, Wrocław",
        wskazano="2026-11-20",
    )
    substitute_id = room.value("SELECT max(id) FROM najem_umowy_lokale_zastepcze")
    add_received_document(room, f"zgoda-lokal-{substitute_id}", "oswiadczenie")
    room.run(
        "wskaz_zgode_na_lokal_zastepczy",
        id=substitute_id,
        zgoda_dokument_id=f"zgoda-lokal-{substitute_id}",
    )

    return substitute_id


def lease_to_signing(room, lease_id="najem"):
    approved_decision(room, f"do-podpisu-{lease_id}", "inna", "najem_umowy_najmu", lease_id)
    room.run(
        "przekaz_umowe_najmu_do_podpisu",
        id=lease_id,
        do_podpisu_decyzja_id=f"do-podpisu-{lease_id}",
    )


def conclude_lease(room, lease_id="najem", zawarto="2026-11-25T12:00:00Z"):
    add_received_document(room, f"podpisana-{lease_id}", "umowa")
    room.run(
        "odnotuj_zawarcie_umowy_najmu",
        id=lease_id,
        dokument_id=f"podpisana-{lease_id}",
        zawarto=zawarto,
    )


def concluded_lease(room, lease_id="najem", rodzaj="zwykly", **fields):
    """A lease signed by the parties, with the annexes its kind requires."""
    prepare_lease(room, lease_id, rodzaj, **fields)
    if rodzaj != "zwykly":
        add_tenant_declaration(room, lease_id)
    if rodzaj == "okazjonalny":
        add_substitute_flat(room, lease_id)
    lease_to_signing(room, lease_id)
    conclude_lease(room, lease_id)


def signed_protocol(room, rodzaj, chwila="2026-11-30T10:00:00Z"):
    room.run(
        "przygotuj_protokol",
        id=f"protokol-{rodzaj}",
        rodzaj=rodzaj,
        nieruchomosc_id="lokal",
        chwila=chwila,
    )
    room.run(
        "dopisz_pozycje_protokolu",
        protokol_id=f"protokol-{rodzaj}",
        rodzaj="licznik",
        nazwa="licznik energii elektrycznej",
        numer="E-778812",
        opis_stanu="12 345,6 kWh",
    )
    add_received_document(room, f"podpisany-protokol-{rodzaj}", "protokol")
    room.run(
        "podpisz_protokol",
        id=f"protokol-{rodzaj}",
        dokument_id=f"podpisany-protokol-{rodzaj}",
    )


def admitted_account_for(room, account_id, person_id, cel, posiadacz="Anna Kowalska"):
    """An account confirmed by another channel, checked by Kontroler and admitted (R4)."""
    add_account(room, account_id, person_id, cel=cel, posiadacz=posiadacz)
    confirm_account(room, account_id)
    check(room, f"sprawdzenie-{account_id}", "rachunek", rachunek_id=account_id)
    approved_decision(
        room,
        f"dopuszczenie-{account_id}",
        "dopuszczenie_rachunku",
        "sprawa_rachunki_wpisy",
        account_id,
    )


def prepare_payout(room, payout_id, podstawa_tabela, podstawa_id, kwota, person_id, account_id):
    room.run(
        "przygotuj_wyplate",
        id=payout_id,
        podstawa_tabela=podstawa_tabela,
        podstawa_id=podstawa_id,
        kwota=kwota,
        wyliczenie_sciezka=f"wyliczenia/{payout_id}.json",
        wyliczenie_skrot=f"sha256-{payout_id}",
        odbiorca_uczestnik_id=person_id,
        rachunek_id=account_id,
        utworzono="2027-01-12T09:00:00Z",
    )


def execute_payout(room, payout_id, account_id):
    """A fresh check of the account, a human's consent and the transfer (D13)."""
    check(
        room,
        f"sprawdzenie-{payout_id}",
        "rachunek",
        sprawdzono="2027-01-12T10:00:00Z",
        rachunek_id=account_id,
    )
    room.run("wskaz_sprawdzenie_wyplaty", id=payout_id, sprawdzenie_id=f"sprawdzenie-{payout_id}")
    approved_decision(
        room,
        f"zgoda-{payout_id}",
        "wyplata",
        "sprawa_wyplaty_wpisy",
        payout_id,
        zlozono="2027-01-12T11:00:00Z",
        podpisano="2027-01-12T12:00:00Z",
    )
    room.run("wskaz_zgode_na_wyplate", id=payout_id, decyzja_id=f"zgoda-{payout_id}")
    add_received_document(room, f"przelew-{payout_id}", "potwierdzenie_bankowe")
    room.run(
        "odnotuj_wykonanie_wyplaty",
        id=payout_id,
        wykonal="tozsamosc-posrednika",
        wykonano="2027-01-12T13:00:00Z",
        potwierdzenie_dokument_id=f"przelew-{payout_id}",
    )


def letter_to_tenant(room, letter_id, wzor):
    prepare_letter(
        room,
        letter_id,
        adresat_rodzaj=None,
        adresat_nazwa=None,
        adresat_uczestnik_id="najemca",
        wzor=wzor,
    )


def send_with_consent(room, letter_id, wyslano="2027-02-25T10:00:00Z"):
    """A letter that always needs a human: waiting for consent, the decision, then sending."""
    room.run("przekaz_wysylke_do_zgody", id=letter_id)
    ask_for_decision(
        room, f"wyslanie-{letter_id}", "wyslanie", "sprawa_korespondencja_wysylki", letter_id
    )
    approve(room, f"wyslanie-{letter_id}")
    send(room, letter_id, decyzja_id=f"wyslanie-{letter_id}", wyslano=wyslano)


def rent_paid_to_office(room, payment_id="wplata-grudzien", kwota=300_000):
    add_received_document(room, f"wyciag-{payment_id}", "potwierdzenie_bankowe")
    room.run(
        "wpisz_wplate_czynszu",
        id=payment_id,
        najem_id="najem",
        chwila="2026-12-08T10:00:00Z",
        kwota=kwota,
        placacy_uczestnik_id="najemca",
        na_rachunek="biura",
        potwierdzenie="wyciag_biura",
        dokument_id=f"wyciag-{payment_id}",
    )


def add_rent_due(room, due_id="czynsz-grudzien", okres="2026-12", kwota=300_000):
    room.run(
        "wpisz_naleznosc",
        id=due_id,
        najem_id="najem",
        tytul="czynsz",
        okres=okres,
        kwota=kwota,
        termin=f"{okres}-10",
    )


def deposit_at_office(room, deposit_id="kaucja"):
    room.run(
        "wpisz_pobranie_kaucji",
        id=deposit_id,
        najem_id="najem",
        kwota=600_000,
        krotnosc=2.0,
        czynsz_przy_zawarciu=300_000,
        wplacono="2026-11-25T13:00:00Z",
        lezy="u_biura",
    )


def deposit_settlement(room, settlement_id="rozliczenie", kwota_zwaloryzowana=640_000):
    room.run(
        "przygotuj_rozliczenie_kaucji",
        id=settlement_id,
        pobranie_id="kaucja",
        oproznienie="2027-08-31",
        czynsz_w_dniu_zwrotu=320_000,
        kwota_zwaloryzowana=kwota_zwaloryzowana,
        wyliczenie_sciezka=f"wyliczenia/{settlement_id}.json",
        wyliczenie_skrot=f"sha256-{settlement_id}",
    )


def add_deduction(room, deduction_id, settlement_id="rozliczenie", kwota=40_000, approved=True):
    add_received_document(room, f"podstawa-{deduction_id}", "inny")
    room.run(
        "dopisz_potracenie_z_kaucji",
        id=deduction_id,
        rozliczenie_id=settlement_id,
        tytul="naprawa blatu z protokołu zwrotu",
        kwota=kwota,
        dokument_id=f"podstawa-{deduction_id}",
    )
    if approved:
        approved_decision(
            room, f"zgoda-{deduction_id}", "inna", "najem_kaucja_potracenia", deduction_id
        )
        room.run("wskaz_zgode_na_potracenie", id=deduction_id, decyzja_id=f"zgoda-{deduction_id}")


def report_defect(room, defect_id="usterka", obciaza="wynajmujacy"):
    add_inflow(room, f"zgloszenie-{defect_id}", "najemca")
    room.run(
        "zglos_usterke",
        id=defect_id,
        chwila="2027-01-20T08:00:00Z",
        zglosil_uczestnik_id="najemca",
        wplyw_id=f"zgloszenie-{defect_id}",
        opis="Cieknie syfon pod zlewem",
        uniemozliwia=0,
        obciaza=obciaza,
    )


def order_repair(room, defect_id="usterka", koszt=30_000, **fields):
    room.run(
        "zlec_naprawe_usterki",
        id=defect_id,
        koszt=koszt,
        zlecil="tozsamosc-posrednika",
        zlecono="2027-01-21T09:00:00Z",
        wykonawca="Hydraulik Jan Kowalski",
        **fields,
    )


def full_managed_rental(room):
    """An ordinary lease the office manages, from the contract to the return of the deposit."""
    open_rental(room)
    add_rental_parties(room)
    add_person(room, "partnerka", imiona="Ewa", nazwisko="Zielińska")
    room.run("nadaj_role_uczestnikowi", uczestnik_id="partnerka", rola="zamieszkujacy")
    add_flat(room)
    room.run(
        "wpisz_powiazanie",
        rodzaj="sprawa_zrodlowa",
        typ_teczki="oferta",
        oznaczenie="Oferta 2026/041",
        odnosnik="pokoj-oferty-041",
        wpisal="tozsamosc-kierownika-najmu",
        wpisano="2026-11-02T09:15:00Z",
    )

    # The management contract: repairs up to 500 zł without asking, money through the office.
    move_to(room, "umowa_z_biurem")
    management(room)

    # The lease: an ordinary lease for two years, signed after a human's consent.
    move_to(room, "projekt_umowy")
    prepare_lease(room)
    room.run(
        "dopisz_oplate_najmu",
        najem_id="najem",
        rodzaj="energia",
        sposob="najemca płaci dostawcy według licznika",
        umowa_z_dostawca_ma="najemca",
    )
    room.run(
        "dopisz_oplate_najmu",
        najem_id="najem",
        rodzaj="woda",
        sposob="zaliczka 80 zł miesięcznie, rozliczenie według wodomierza",
        umowa_z_dostawca_ma="wynajmujacy",
    )
    lease_to_signing(room)
    move_to(room, "podpisanie_umowy")
    conclude_lease(room)
    room.run("odnotuj_przekazanie_swiadectwa", id="najem", swiadectwo_przekazano="2026-11-25")
    room.run("wpisz_zamieszkujacego", najem_id="najem", uczestnik_id="partnerka", od="2026-12-01")
    deposit_at_office(room)

    # Handover with a signed protocol; the lease runs.
    move_to(room, "wydanie_lokalu")
    signed_protocol(room, "wydanie")
    move_to(room, "trwanie_najmu")

    # The rent of December through the office and its transfer to the owner (T13).
    add_rent_due(room)
    rent_paid_to_office(room)
    room.run(
        "rozlicz_wplate", wplata_id="wplata-grudzien", naleznosc_id="czynsz-grudzien", kwota=300_000
    )
    admitted_account_for(room, "rachunek-wlascicielki", "wlascicielka", "wyplata_dla_wlasciciela")
    prepare_payout(
        room,
        "przekazanie-grudzien",
        "najem_czynsz_wplaty",
        "wplata-grudzien",
        270_000,
        "wlascicielka",
        "rachunek-wlascicielki",
    )
    execute_payout(room, "przekazanie-grudzien", "rachunek-wlascicielki")

    # Meters, a defect within the repair limit, an inspection.
    room.run(
        "wpisz_odczyt_licznika",
        najem_id="najem",
        licznik="woda_zimna",
        numer="W-1201",
        chwila="2027-01-05T09:00:00Z",
        stan="231,4 m³",
        zdjecie_sciezka="odczyty/woda-2027-01.jpg",
        zdjecie_skrot="sha256-woda-2027-01",
    )
    report_defect(room)
    order_repair(room)
    add_inflow(room, "potwierdzenie-usuniecia", "najemca")
    room.run(
        "odnotuj_usuniecie_usterki",
        id="usterka",
        usunieto="2027-01-22",
        potwierdzenie_wplyw_id="potwierdzenie-usuniecia",
    )
    room.run("wpisz_przeglad", data="2027-02-01", wynik="Lokal bez uwag")

    # An annex allowing a cat, and a rent increase by notice (art. 8a UOPL).
    room.run("przygotuj_aneks_najmu", id="aneks-kot", najem_id="najem", opis="Zgoda na kota")
    approved_decision(room, "zgoda-aneks-kot", "inna", "najem_umowy_aneksy", "aneks-kot")
    room.run("wskaz_zgode_na_aneks_najmu", id="aneks-kot", decyzja_id="zgoda-aneks-kot")
    add_received_document(room, "podpisany-aneks-kot", "umowa")
    room.run(
        "odnotuj_podpisanie_aneksu_najmu",
        id="aneks-kot",
        dokument_id="podpisany-aneks-kot",
        obowiazuje_od="2027-01-15",
    )
    letter_to_tenant(room, "podwyzka", "wypowiedzenie-wysokosci-czynszu")
    send(room, "podwyzka", wyslano="2027-01-14T10:00:00Z")
    room.run("odnotuj_doreczenie", id="podwyzka", doreczono="2027-01-15T10:00:00Z")
    room.run(
        "wpisz_zmiane_czynszu",
        najem_id="najem",
        stara=300_000,
        nowa=320_000,
        pismo_wysylka_id="podwyzka",
        doreczono="2027-01-15",
        obowiazuje_od="2027-05-01",
        podstawa="art. 8a UOPL, wzrost kosztów utrzymania",
    )

    # Arrears of three periods: a warning sent with a human's consent, an extra month, paid.
    room.run("wpisz_zaleglosc", id="zaleglosc", najem_id="najem", trzy_okresy_od="2027-02-11")
    letter_to_tenant(room, "uprzedzenie", "uprzedzenie-o-wypowiedzeniu")
    room.run("wskaz_uprzedzenie_o_zaleglosci", id="zaleglosc", uprzedzenie_wysylka_id="uprzedzenie")
    send_with_consent(room, "uprzedzenie")
    room.run("odnotuj_doreczenie", id="uprzedzenie", doreczono="2027-02-26T10:00:00Z")
    room.run("ustal_dodatkowy_termin_zaleglosci", id="zaleglosc", dodatkowy_termin_do="2027-03-26")
    room.run("odnotuj_wynik_zaleglosci", id="zaleglosc", wynik="splacona")

    # Rent data for the municipality, sent by a human.
    room.run(
        "przygotuj_dane_dla_gminy",
        najem_id="najem",
        okres="2027",
        czynsz=320_000,
        gmina="Wrocław",
        rok_budowy=1998,
        stan_techniczny="dobry",
        powierzchnia=4820,
        standard="wyposażenie z protokołu wydania",
    )
    room.run("odnotuj_oddanie_danych_dla_gminy", id=1, oddano="2027-03-01")
    room.run(
        "odnotuj_wyslanie_danych_dla_gminy",
        id=1,
        wyslano="2027-03-05",
        wyslal="tozsamosc-kierownika-najmu",
    )

    # The parties end the lease by agreement; the flat comes back with a protocol.
    move_to(room, "koniec_najmu")
    room.run(
        "wpisz_koniec_najmu",
        id="koniec",
        najem_id="najem",
        sposob="porozumienie",
        koniec_najmu="2027-08-31",
    )
    room.run("odnotuj_oproznienie_lokalu", id="koniec", oproznienie="2027-08-31")
    room.run("zakoncz_umowe_najmu", id="najem", koniec_powod="porozumienie stron")
    move_to(room, "zwrot_lokalu")
    signed_protocol(room, "zwrot", chwila="2027-08-31T10:00:00Z")

    # The deposit: indexed to the rent on the return day, one deduction, back to the tenant.
    move_to(room, "rozliczenie_kaucji")
    deposit_settlement(room)
    add_deduction(room, "potracenie-blat")
    room.run("zatwierdz_rozliczenie_kaucji", id="rozliczenie")
    letter_to_tenant(room, "rozliczenie-kaucji", "rozliczenie-kaucji")
    send(room, "rozliczenie-kaucji", wyslano="2027-09-10T10:00:00Z")
    room.run(
        "odnotuj_wyslanie_rozliczenia_kaucji", id="rozliczenie", wysylka_id="rozliczenie-kaucji"
    )
    admitted_account_for(room, "rachunek-najemcy", "najemca", "zwrot_kaucji", posiadacz="Jan Nowak")
    prepare_payout(
        room,
        "zwrot-kaucji",
        "najem_kaucja_rozliczenia",
        "rozliczenie",
        600_000,
        "najemca",
        "rachunek-najemcy",
    )
    execute_payout(room, "zwrot-kaucji", "rachunek-najemcy")
    move_to(room, "po_najmie")


def full_brokered_occasional_rental(room):
    """An occasional lease the office only brokers: it ends after the tax office notification."""
    open_rental(room)
    add_rental_parties(room)
    add_person(room, "wlasciciel-lipowej", imiona="Piotr", nazwisko="Lis")
    room.run(
        "nadaj_role_uczestnikowi",
        uczestnik_id="wlasciciel-lipowej",
        rola="wlasciciel_innego_lokalu",
    )
    add_flat(room)

    move_to(room, "projekt_umowy")
    prepare_lease(room, rodzaj="okazjonalny", kaucja=900_000)
    add_substitute_flat(room)
    move_to(room, "oswiadczenie_najemcy")
    add_tenant_declaration(room)
    lease_to_signing(room)
    move_to(room, "podpisanie_umowy")
    conclude_lease(room)
    admitted_account_for(room, "rachunek-kaucji", "wlascicielka", "kaucja")
    room.run(
        "wpisz_pobranie_kaucji",
        id="kaucja",
        najem_id="najem",
        kwota=900_000,
        krotnosc=3.0,
        czynsz_przy_zawarciu=300_000,
        wplacono="2026-11-25T13:00:00Z",
        lezy="u_wynajmujacego",
        rachunek_id="rachunek-kaucji",
    )

    move_to(room, "wydanie_lokalu")
    signed_protocol(room, "wydanie")
    move_to(room, "zgloszenie_najmu")
    add_received_document(room, "zgloszenie-us", "pismo")
    room.run(
        "odnotuj_zgloszenie_najmu_do_urzedu",
        id="najem",
        zgloszono_us="2026-12-10",
        zgloszenie_us_dokument_id="zgloszenie-us",
    )
    move_to(room, "po_najmie", "zamknieta")


# The whole case ------------------------------------------------------------------------------------


@scenario("Przebieg najmu z zarządzaniem: umowa, wydanie, czynsz przez biuro, koniec, zwrot kaucji")
def whole_managed_rental():
    room = new_room()

    full_managed_rental(room)
    move_to(room, "zamknieta")

    etapy = [row["etap"] for row in room.run("etapy")]
    assert etapy == [
        "przyjeta",
        "umowa_z_biurem",
        "projekt_umowy",
        "podpisanie_umowy",
        "wydanie_lokalu",
        "trwanie_najmu",
        "koniec_najmu",
        "zwrot_lokalu",
        "rozliczenie_kaucji",
        "po_najmie",
        "zamknieta",
    ], etapy
    assert [row["stan"] for row in room.run("wyplaty")] == ["wykonana", "wykonana"]
    balance = room.run("saldo_naleznosci")[0]
    assert balance["zaplacono"] == 300_000 and balance["zostalo"] == 0, balance
    assert room.run("zmiany_czynszu")[0]["nowa"] == 320_000
    assert room.run("rozliczenia_kaucji")[0]["stan"] == "wyslane"
    assert room.run("dane_dla_gminy")[0]["wyslal"] == "tozsamosc-kierownika-najmu"


@scenario("Przebieg najmu okazjonalnego przy samym pośrednictwie: załączniki, wydanie, zgłoszenie")
def whole_brokered_occasional_rental():
    room = new_room()

    full_brokered_occasional_rental(room)

    etapy = [row["etap"] for row in room.run("etapy")]
    assert etapy == [
        "przyjeta",
        "projekt_umowy",
        "oswiadczenie_najemcy",
        "podpisanie_umowy",
        "wydanie_lokalu",
        "zgloszenie_najmu",
        "po_najmie",
        "zamknieta",
    ], etapy
    assert room.run("umowy_najmu")[0]["zgloszono_us"] == "2026-12-10"


# Rules on the core -------------------------------------------------------------------------------


@scenario("Reguły: teczka najmu ma typ pokoju najem (T2)")
def room_type():
    room = new_room()

    room.refused(
        "teczka najmu ma typ pokoju najem",
        "zaloz_metryke",
        oznaczenie="Transakcja 2026/003",
        typ_pokoju="transakcja",
        przyjal="tozsamosc-posrednika",
        przyjeto="2026-11-02T09:00:00Z",
    )

    open_rental(room)
    assert room.run("metryka")[0]["typ_pokoju"] == "najem"


@scenario("Reguły: etapy najmu idą po kolei; umowę z biurem pomija się przy samym pośrednictwie")
def stages_in_order():
    room = new_room()
    open_rental(room)

    rule = "etapy najmu idą po kolei"
    refused_stage(room, rule, "umowa")
    refused_stage(room, rule, "wydanie_lokalu")
    move_to(room, "projekt_umowy")
    refused_stage(room, rule, "umowa_z_biurem")

    approved_decision(room, "przerwanie", "przerwanie_sprawy")
    move_to(room, "przerwana", decyzja_id="przerwanie")
    refused_stage(room, rule, "podpisanie_umowy")
    move_to(room, "zamknieta")


@scenario("Reguły: wznowiony najem wraca do etapu sprzed przerwania i idzie dalej po kolei")
def stages_resumed():
    room = new_room()
    open_rental(room)
    move_to(room, "projekt_umowy")
    approved_decision(room, "przerwanie", "przerwanie_sprawy")
    move_to(room, "przerwana", decyzja_id="przerwanie")
    approved_decision(room, "wznowienie", "wznowienie_sprawy", podpisano="2026-11-05T12:00:00Z")

    room.refused(
        "etapy najmu idą po kolei",
        "zmien_etap_sprawy",
        etap="umowa_z_biurem",
        od="2026-11-06",
        zmienil="agent",
        decyzja_id="wznowienie",
    )
    move_to(room, "projekt_umowy", decyzja_id="wznowienie")
    move_to(room, "oswiadczenie_najemcy")


@scenario("Reguły: z wydania lokalu i ze zwrotu wychodzi się z podpisanym protokołem")
def stages_need_protocols():
    room = new_room()
    open_rental(room)
    add_rental_parties(room)
    add_flat(room)
    concluded_lease(room)
    move_to(room, "projekt_umowy", "podpisanie_umowy", "wydanie_lokalu")

    refused_stage(
        room,
        "sprawa wychodzi z wydania lokalu tylko z podpisanym protokołem wydania",
        "trwanie_najmu",
    )
    signed_protocol(room, "wydanie")
    move_to(room, "trwanie_najmu", "koniec_najmu", "zwrot_lokalu")

    refused_stage(
        room, "sprawa wychodzi ze zwrotu lokalu tylko z podpisanym protokołem zwrotu", "po_najmie"
    )
    signed_protocol(room, "zwrot", chwila="2027-08-31T10:00:00Z")
    move_to(room, "po_najmie")


@scenario("Reguły: przy umowie o zarządzanie najem nie kończy się po wydaniu lokalu (R6)")
def management_does_not_end_after_handover():
    room = new_room()
    open_rental(room)
    add_rental_parties(room)
    add_flat(room)
    concluded_lease(room)
    move_to(room, "projekt_umowy", "podpisanie_umowy", "wydanie_lokalu")
    signed_protocol(room, "wydanie")
    management(room)

    rule = "przy zawartej umowie o zarządzanie najem nie kończy się po wydaniu lokalu (R6)"
    refused_stage(room, rule, "po_najmie")
    refused_stage(room, rule, "zamknieta")

    room.run("zakoncz_umowe", id="zarzadzanie", koniec_powod="wypowiedzenie przez właścicielkę")
    move_to(room, "po_najmie")


@scenario("Reguły: role, zgody i tylko umowa o zarządzanie w teczce najmu")
def roles_consents_and_office_contract():
    room = new_room()
    open_rental(room)
    add_rental_parties(room)

    room.refused(
        "rola w teczce najmu to wynajmujacy",
        "nadaj_role_uczestnikowi",
        uczestnik_id="najemca",
        rola="kupujacy",
    )
    room.run(
        "nadaj_role_uczestnikowi",
        uczestnik_id="najemca",
        rola="malzonek",
        dotyczy_uczestnik_id="wlascicielka",
    )

    add_inflow(room, "oswiadczenie", "wlascicielka")
    consent = {
        "uczestnik_id": "wlascicielka",
        "tresc": "Treść oświadczenia",
        "zlozono": "2026-11-04T10:00:00Z",
        "wplyw_id": "oswiadczenie",
        "trwaly_nosnik": 1,
    }
    room.refused(
        "zgoda w teczce najmu to wybor_rodzaju_najmu",
        "wpisz_zgode",
        id="cena",
        rodzaj="zatwierdzenie_ceny",
        **consent,
    )
    room.run("wpisz_zgode", id="rodzaj-najmu", rodzaj="wybor_rodzaju_najmu", **consent)
    room.run("wpisz_zgode", id="zadanie", rodzaj="zadanie_rozpoczecia_pracy", **consent)

    refusal = "umowa z biurem w teczce najmu to umowa o zarządzanie"
    room.refused(
        refusal,
        "przygotuj_umowe",
        id="posrednictwo",
        rodzaj="posrednictwo",
        obie_strony=0,
        wylacznosc=0,
    )
    room.run("przygotuj_umowe", id="zarzadzanie", rodzaj="zarzadzanie", obie_strony=0, wylacznosc=1)
    room.refused_sql(
        "rodzaj umowy się nie zmienia",
        "UPDATE sprawa_umowy_wpisy SET rodzaj = 'posrednictwo' WHERE id = 'zarzadzanie'",
    )


# Management terms --------------------------------------------------------------------------------


@scenario("Zarządzanie: pierwsze warunki od zawarcia umowy, kolejne od jej aneksu")
def management_terms_versions():
    room = new_room()
    open_rental(room)
    add_rental_parties(room)
    concluded_contract(room, "zarzadzanie", "wlascicielka")

    rule = "pierwsze warunki zarządzania obowiązują od zawarcia umowy"
    terms = {"umowa_id": "zarzadzanie", "kwota_napraw": 50_000, "przeplyw": "przez_biuro"}
    room.refused(rule, "wpisz_warunki_zarzadzania", od="2026-11-01T00:00:00Z", **terms)
    room.run("wpisz_warunki_zarzadzania", od=MANAGEMENT_CONCLUDED, **terms)

    # The next terms need an annex of the same contract.
    room.refused(rule, "wpisz_warunki_zarzadzania", od="2026-12-01T00:00:00Z", **terms)
    add_received_document(room, "aneks-zarzadzania", "umowa_z_biurem")
    room.run(
        "wpisz_zdarzenie_umowy",
        id="aneks",
        umowa_id="zarzadzanie",
        rodzaj="aneks",
        chwila="2026-12-01T00:00:00Z",
        dokument_id="aneks-zarzadzania",
    )
    room.run(
        "wpisz_warunki_zarzadzania",
        umowa_id="zarzadzanie",
        kwota_napraw=100_000,
        przeplyw="wprost",
        aneks_zdarzenie_id="aneks",
        od="2026-12-01T00:00:00Z",
    )
    assert room.run("warunki_zarzadzania")[-1]["przeplyw"] == "wprost"

    room.refused_sql(
        "warunków zarządzania się nie zmienia",
        "UPDATE najem_umowy_zarzadzanie SET kwota_napraw = 0",
    )


# Lease -------------------------------------------------------------------------------------------


@scenario("Umowa najmu: czas, forma i granica kaucji według rodzaju najmu (UOPL)")
def lease_kind_limits():
    room = new_room()
    open_rental(room)
    add_flat(room)

    room.refused(
        "okazjonalny_najwyzej_10_lat",
        "przygotuj_umowe_najmu",
        **lease_fields(rodzaj="okazjonalny", koniec="2036-12-02", kaucja=0),
    )
    room.refused(
        "okazjonalny_najwyzej_10_lat",
        "przygotuj_umowe_najmu",
        **lease_fields(rodzaj="okazjonalny", koniec=None, kaucja=0),
    )
    room.refused(
        "instytucjonalny_na_czas_oznaczony",
        "przygotuj_umowe_najmu",
        **lease_fields(rodzaj="instytucjonalny", koniec=None, kaucja=0),
    )
    room.refused(
        "forma_pisemna_przy_okazjonalnym_i_instytucjonalnym",
        "przygotuj_umowe_najmu",
        **lease_fields(rodzaj="instytucjonalny", forma="dokumentowa", kaucja=0),
    )
    room.refused("kaucja_w_granicy", "przygotuj_umowe_najmu", **lease_fields(kaucja=3_600_001))
    room.refused(
        "kaucja_w_granicy",
        "przygotuj_umowe_najmu",
        **lease_fields(rodzaj="okazjonalny", kaucja=1_800_001),
    )

    room.run(
        "przygotuj_umowe_najmu",
        **lease_fields(id="zwykly", kaucja=3_600_000, forma="dokumentowa", koniec=None),
    )
    room.run(
        "przygotuj_umowe_najmu",
        **lease_fields(
            id="okazjonalny", rodzaj="okazjonalny", koniec="2036-12-01", kaucja=1_800_000
        ),
    )

    add_received_document(room, "oswiadczenie", "akt_notarialny")
    room.refused(
        "oswiadczenie_przy_okazjonalnym_i_instytucjonalnym",
        "wskaz_oswiadczenie_najemcy",
        id="zwykly",
        oswiadczenie_dokument_id="oswiadczenie",
    )
    room.run(
        "wskaz_oswiadczenie_najemcy", id="okazjonalny", oswiadczenie_dokument_id="oswiadczenie"
    )


def lease_fields(**fields):
    return {
        "id": "najem",
        "rodzaj": "zwykly",
        "nieruchomosc_id": "lokal",
        "poczatek": "2026-12-01",
        "koniec": "2028-11-30",
        "forma": "pisemna",
        "czynsz": 300_000,
        "kaucja": 600_000,
        **fields,
    }


@scenario(
    "Umowa najmu: powstaje jako projekt, z podpisu nie wraca, zawarta ma podpisany egzemplarz"
)
def lease_states():
    room = new_room()
    open_rental(room)
    add_rental_parties(room)
    add_flat(room)

    room.refused_sql(
        "umowa najmu powstaje jako projekt",
        """INSERT INTO najem_umowy_najmu (id, rodzaj, nieruchomosc_id, poczatek, forma, stan,
          czynsz, kaucja, koniec_powod)
        VALUES ('od-razu', 'zwykly', 'lokal', '2026-12-01', 'pisemna', 'zakonczona', 1, 0, 'pomyłka')""",
    )
    prepare_lease(room)
    lease_to_signing(room)
    room.refused_sql(
        "z podpisu nie wraca do projektu",
        "UPDATE najem_umowy_najmu SET stan = 'projekt' WHERE id = 'najem'",
    )

    add_received_document(room, "zly-dokument", "pismo")
    room.refused(
        "zawarta umowa najmu wskazuje otrzymany podpisany egzemplarz",
        "odnotuj_zawarcie_umowy_najmu",
        id="najem",
        dokument_id="zly-dokument",
        zawarto="2026-11-25T12:00:00Z",
    )
    conclude_lease(room)


@scenario("Umowa najmu: do podpisu za zgodą człowieka i nie przy stojącym znaczniku wątpliwości")
def lease_signing_needs_consent():
    room = new_room()
    open_rental(room)
    add_rental_parties(room)
    add_flat(room)
    prepare_lease(room)

    room.refused(
        "umowa najmu idzie do podpisu za zatwierdzoną decyzją inna",
        "przekaz_umowe_najmu_do_podpisu",
        id="najem",
        do_podpisu_decyzja_id="brak",
    )
    approved_decision(room, "o-innej", "inna", "najem_umowy_najmu", "inna-umowa")
    room.refused(
        "umowa najmu idzie do podpisu za zatwierdzoną decyzją inna",
        "przekaz_umowe_najmu_do_podpisu",
        id="najem",
        do_podpisu_decyzja_id="o-innej",
    )

    approved_decision(room, "do-podpisu-najem", "inna", "najem_umowy_najmu", "najem")
    room.run(
        "postaw_watpliwosc",
        id="watpliwosc",
        postawil="tozsamosc-kontrolera",
        postawiono="2026-11-07T09:00:00Z",
    )
    room.refused(
        "dopóki stoi znacznik wątpliwości, umowa najmu nie idzie do podpisu",
        "przekaz_umowe_najmu_do_podpisu",
        id="najem",
        do_podpisu_decyzja_id="do-podpisu-najem",
    )


@scenario("Umowa najmu: strony według rodzaju najmu (art. 19a ust. 1 i art. 19f ust. 1 UOPL)")
def lease_parties_by_kind():
    room = new_room()
    open_rental(room)
    add_flat(room)
    add_company(room, "spolka")
    room.run("nadaj_role_uczestnikowi", uczestnik_id="spolka", rola="wynajmujacy")
    rule = "umowa najmu ma wynajmującego i najemcę"

    # Without a tenant there is nobody to sign.
    prepare_lease(room, "zwykly")
    approved_decision(room, "do-podpisu-zwykly", "inna", "najem_umowy_najmu", "zwykly")
    room.refused(
        rule,
        "przekaz_umowe_najmu_do_podpisu",
        id="zwykly",
        do_podpisu_decyzja_id="do-podpisu-zwykly",
    )
    add_person(room, "najemca", imiona="Jan", nazwisko="Nowak")
    room.run("nadaj_role_uczestnikowi", uczestnik_id="najemca", rola="najemca")
    room.run(
        "przekaz_umowe_najmu_do_podpisu", id="zwykly", do_podpisu_decyzja_id="do-podpisu-zwykly"
    )

    # An occasional lease has an individual as the landlord, not a company.
    prepare_lease(room, "okazjonalny", "okazjonalny", kaucja=0)
    add_tenant_declaration(room, "okazjonalny")
    add_substitute_flat(room, "okazjonalny")
    approved_decision(room, "do-podpisu-okazjonalny", "inna", "najem_umowy_najmu", "okazjonalny")
    room.refused(
        rule,
        "przekaz_umowe_najmu_do_podpisu",
        id="okazjonalny",
        do_podpisu_decyzja_id="do-podpisu-okazjonalny",
    )

    # An institutional landlord has a tax number and a register entry; the company has both.
    prepare_lease(room, "instytucjonalny", "instytucjonalny", kaucja=0)
    add_tenant_declaration(room, "instytucjonalny")
    lease_to_signing(room, "instytucjonalny")

    add_person(room, "osoba-bez-nip")
    room.run("nadaj_role_uczestnikowi", uczestnik_id="osoba-bez-nip", rola="wynajmujacy")
    prepare_lease(room, "drugi-instytucjonalny", "instytucjonalny", kaucja=0)
    add_tenant_declaration(room, "drugi-instytucjonalny")
    approved_decision(
        room, "do-podpisu-drugi", "inna", "najem_umowy_najmu", "drugi-instytucjonalny"
    )
    room.refused(
        rule,
        "przekaz_umowe_najmu_do_podpisu",
        id="drugi-instytucjonalny",
        do_podpisu_decyzja_id="do-podpisu-drugi",
    )


@scenario(
    "Umowa najmu: okazjonalna i instytucjonalna do podpisu tylko z załącznikami (art. 19a ust. 2)"
)
def lease_signing_needs_annexes():
    room = new_room()
    open_rental(room)
    add_rental_parties(room)
    add_flat(room)
    rule = "umowa najmu okazjonalnego i instytucjonalnego idzie do podpisu z oświadczeniem najemcy"

    prepare_lease(room, rodzaj="okazjonalny", kaucja=0)
    approved_decision(room, "do-podpisu-najem", "inna", "najem_umowy_najmu", "najem")
    signing = {"id": "najem", "do_podpisu_decyzja_id": "do-podpisu-najem"}
    room.refused(rule, "przekaz_umowe_najmu_do_podpisu", **signing)

    add_tenant_declaration(room)
    room.refused(rule, "przekaz_umowe_najmu_do_podpisu", **signing)

    # A substitute flat without its owner's consent, then one the tenant can no longer use.
    room.run("wpisz_lokal_zastepczy", najem_id="najem", adres="ul. Długa 1", wskazano="2026-11-20")
    room.refused(rule, "przekaz_umowe_najmu_do_podpisu", **signing)
    substitute_id = add_substitute_flat(room)
    room.run("odnotuj_utrate_lokalu_zastepczego", id=substitute_id, niemozliwy_od="2026-11-22")
    room.refused(rule, "przekaz_umowe_najmu_do_podpisu", **signing)

    add_substitute_flat(room)
    room.run("przekaz_umowe_najmu_do_podpisu", **signing)


@scenario(
    "Umowa najmu: po wyjściu z projektu treść stoi, a zgłoszenie i świadectwo wpisuje się raz"
)
def lease_frozen_after_draft():
    room = new_room()
    open_rental(room)
    add_rental_parties(room)
    add_flat(room)
    prepare_lease(room, rodzaj="okazjonalny", kaucja=0)
    room.run("zmien_projekt_umowy_najmu", **lease_change(czynsz=310_000))
    add_tenant_declaration(room)
    add_substitute_flat(room)
    lease_to_signing(room)

    rule = "treść umowy najmu zmienia się tylko w projekcie"
    room.refused(rule, "zmien_projekt_umowy_najmu", **lease_change(czynsz=320_000))
    conclude_lease(room)
    room.refused(rule, "zmien_projekt_umowy_najmu", **lease_change(czynsz=320_000))

    room.run("odnotuj_przekazanie_swiadectwa", id="najem", swiadectwo_przekazano="2026-11-25")
    room.refused(
        rule, "odnotuj_przekazanie_swiadectwa", id="najem", swiadectwo_przekazano="2026-11-26"
    )
    add_received_document(room, "zgloszenie", "pismo")
    room.run(
        "odnotuj_zgloszenie_najmu_do_urzedu",
        id="najem",
        zgloszono_us="2026-12-10",
        zgloszenie_us_dokument_id="zgloszenie",
    )
    room.refused(
        rule,
        "odnotuj_zgloszenie_najmu_do_urzedu",
        id="najem",
        zgloszono_us="2026-12-11",
        zgloszenie_us_dokument_id="zgloszenie",
    )


def lease_change(**fields):
    return {
        "id": "najem",
        "poczatek": "2026-12-01",
        "koniec": "2028-11-30",
        "forma": "pisemna",
        "projekt_dokument_id": "projekt-najem",
        "czynsz": 300_000,
        "kaucja": 0,
        **fields,
    }


@scenario("Umowa najmu: zgłoszenie do urzędu skarbowego jest tylko przy najmie okazjonalnym")
def tax_office_notification_only_occasional():
    room = new_room()
    open_rental(room)
    add_rental_parties(room)
    add_flat(room)
    concluded_lease(room)
    add_received_document(room, "zgloszenie", "pismo")

    room.refused(
        "zgloszenie_us_przy_okazjonalnym",
        "odnotuj_zgloszenie_najmu_do_urzedu",
        id="najem",
        zgloszono_us="2026-12-10",
        zgloszenie_us_dokument_id="zgloszenie",
    )


@scenario(
    "Etapy: oświadczenie pomija się tylko przy zwykłym, zgłoszenie tylko przy innym niż okazjonalny"
)
def stages_skipped_by_lease_kind():
    room = new_room()
    open_rental(room)
    add_rental_parties(room)
    add_flat(room)
    move_to(room, "projekt_umowy")

    rule = "etap oświadczenia najemcy pomija się tylko przy najmie zwykłym"
    refused_stage(room, rule, "podpisanie_umowy")
    prepare_lease(room, rodzaj="okazjonalny", kaucja=0)
    refused_stage(room, rule, "podpisanie_umowy")
    move_to(room, "oswiadczenie_najemcy")
    add_tenant_declaration(room)
    add_substitute_flat(room)
    lease_to_signing(room)
    conclude_lease(room)
    move_to(room, "podpisanie_umowy", "wydanie_lokalu")
    signed_protocol(room, "wydanie")

    rule = "przy najmie okazjonalnym sprawa idzie z wydania lokalu przez zgłoszenie najmu"
    refused_stage(room, rule, "trwanie_najmu")
    refused_stage(room, rule, "po_najmie")
    move_to(room, "zgloszenie_najmu", "trwanie_najmu")


@scenario("Opłaty: dopisuje się je do projektu, a przy zwykłym tylko niezależne od właściciela")
def lease_fees():
    room = new_room()
    open_rental(room)
    add_flat(room)
    prepare_lease(room)
    fee = {
        "najem_id": "najem",
        "sposob": "zaliczka miesięczna",
        "umowa_z_dostawca_ma": "wynajmujacy",
    }

    room.refused(
        "przy najmie zwykłym wynajmujący pobiera obok czynszu tylko opłaty niezależne od właściciela",
        "dopisz_oplate_najmu",
        rodzaj="internet",
        **fee,
    )
    room.run("dopisz_oplate_najmu", rodzaj="cieplo", **fee)
    room.run("dopisz_oplate_najmu", rodzaj="internet", **{**fee, "umowa_z_dostawca_ma": "najemca"})
    room.refused_sql(
        "opłaty umowy najmu się nie zmienia", "UPDATE najem_umowy_oplaty SET sposob = 'inny'"
    )

    # An occasional lease may say otherwise (art. 19c ust. 1 UOPL).
    prepare_lease(room, "okazjonalny", "okazjonalny", kaucja=0)
    room.run("dopisz_oplate_najmu", rodzaj="internet", **{**fee, "najem_id": "okazjonalny"})

    approved_decision(room, "do-podpisu-najem", "inna", "najem_umowy_najmu", "najem")
    add_person(room, "wynajmujaca")
    add_person(room, "najemca", imiona="Jan", nazwisko="Nowak")
    room.run("nadaj_role_uczestnikowi", uczestnik_id="wynajmujaca", rola="wynajmujacy")
    room.run("nadaj_role_uczestnikowi", uczestnik_id="najemca", rola="najemca")
    room.run("przekaz_umowe_najmu_do_podpisu", id="najem", do_podpisu_decyzja_id="do-podpisu-najem")
    room.refused(
        "opłatę dopisuje się do projektu umowy najmu", "dopisz_oplate_najmu", rodzaj="gaz", **fee
    )


@scenario(
    "Zamieszkujący i lokal zastępczy: daty i zgoda wpisuje się raz, lokal tylko przy okazjonalnym"
)
def household_and_substitute_flat():
    room = new_room()
    open_rental(room)
    add_rental_parties(room)
    add_flat(room)
    concluded_lease(room)

    room.run("wpisz_zamieszkujacego", najem_id="najem", uczestnik_id="najemca", od="2026-12-01")
    room.refused(
        "wyprowadzka_nie_przed_zamieszkaniem",
        "odnotuj_wyprowadzke_zamieszkujacego",
        id=1,
        do="2026-11-30",
    )
    room.run("odnotuj_wyprowadzke_zamieszkujacego", id=1, do="2027-03-31")
    room.refused(
        "osoby zamieszkującej się nie zmienia",
        "odnotuj_wyprowadzke_zamieszkujacego",
        id=1,
        do="2027-04-30",
    )

    room.refused(
        "lokal zastępczy wskazuje się tylko przy najmie okazjonalnym",
        "wpisz_lokal_zastepczy",
        najem_id="najem",
        adres="ul. Lipowa 4",
        wskazano="2026-11-20",
    )
    prepare_lease(room, "okazjonalny", "okazjonalny", kaucja=0)
    room.run(
        "wpisz_lokal_zastepczy", najem_id="okazjonalny", adres="ul. Lipowa 4", wskazano="2026-11-20"
    )
    add_received_document(room, "pismo", "pismo")
    room.refused(
        "zgoda na lokal zastępczy wskazuje otrzymane oświadczenie",
        "wskaz_zgode_na_lokal_zastepczy",
        id=1,
        zgoda_dokument_id="pismo",
    )
    room.refused_sql(
        "lokal zastępczy powstaje bez zgody i bez utraty",
        """INSERT INTO najem_umowy_lokale_zastepcze (najem_id, adres, wskazano, niemozliwy_od)
        VALUES ('okazjonalny', 'ul. Długa 1', '2026-11-20', '2026-11-21')""",
    )
    room.run("odnotuj_utrate_lokalu_zastepczego", id=1, niemozliwy_od="2027-01-10")
    room.refused(
        "lokalu zastępczego się nie zmienia",
        "odnotuj_utrate_lokalu_zastepczego",
        id=1,
        niemozliwy_od="2027-01-11",
    )


@scenario("Aneks: przy zawartej umowie, za zgodą człowieka, obowiązuje z podpisanym egzemplarzem")
def lease_annex():
    room = new_room()
    open_rental(room)
    add_rental_parties(room)
    add_flat(room)
    prepare_lease(room)

    room.refused(
        "aneks powstaje przy zawartej umowie najmu",
        "przygotuj_aneks_najmu",
        id="aneks",
        najem_id="najem",
        opis="Inny termin płatności",
    )
    lease_to_signing(room)
    conclude_lease(room)
    room.run("przygotuj_aneks_najmu", id="aneks", najem_id="najem", opis="Inny termin płatności")

    add_received_document(room, "podpisany-aneks", "umowa")
    room.refused(
        "aneks obowiązuje z podpisanym egzemplarzem i obowiązującą zgodą człowieka",
        "odnotuj_podpisanie_aneksu_najmu",
        id="aneks",
        dokument_id="podpisany-aneks",
        obowiazuje_od="2027-01-15",
    )
    room.refused(
        "aneks idzie do stron za zatwierdzoną decyzją inna",
        "wskaz_zgode_na_aneks_najmu",
        id="aneks",
        decyzja_id="brak",
    )
    approved_decision(room, "zgoda-aneks", "inna", "najem_umowy_aneksy", "aneks")
    room.run("wskaz_zgode_na_aneks_najmu", id="aneks", decyzja_id="zgoda-aneks")
    room.refused_sql(
        "treść aneksu stoi od zgody człowieka",
        "UPDATE najem_umowy_aneksy SET opis = 'Inna treść' WHERE id = 'aneks'",
    )

    add_received_document(room, "pismo", "pismo")
    room.refused(
        "aneks obowiązuje z podpisanym egzemplarzem",
        "odnotuj_podpisanie_aneksu_najmu",
        id="aneks",
        dokument_id="pismo",
        obowiazuje_od="2027-01-15",
    )
    room.run(
        "odnotuj_podpisanie_aneksu_najmu",
        id="aneks",
        dokument_id="podpisany-aneks",
        obowiazuje_od="2027-01-15",
    )


@scenario("Koniec najmu: wypowiedzenie na piśmie z przyczyną wychodzi tylko za zgodą człowieka")
def lease_notice_with_consent():
    room = new_room()
    open_rental(room)
    add_rental_parties(room)
    add_flat(room)
    concluded_lease(room)
    end = {
        "id": "koniec",
        "najem_id": "najem",
        "sposob": "wypowiedzenie_wynajmujacego",
        "koniec_najmu": "2027-04-30",
    }

    room.refused("wypowiedzenie_wynajmujacego_na_pismie_z_przyczyna", "wpisz_koniec_najmu", **end)

    letter_to_tenant(room, "wyslane-bez-zgody", "wypowiedzenie")
    send(room, "wyslane-bez-zgody")
    room.refused(
        "koniec najmu wskazuje pismo, które jeszcze nie wyszło",
        "wpisz_koniec_najmu",
        wysylka_id="wyslane-bez-zgody",
        przyczyna="zaległość za trzy okresy",
        **end,
    )

    letter_to_tenant(room, "wypowiedzenie", "wypowiedzenie")
    room.run(
        "wpisz_koniec_najmu",
        wysylka_id="wypowiedzenie",
        przyczyna="zaległość za trzy okresy",
        **end,
    )
    room.refused(
        "wypowiedzenie i żądanie opróżnienia lokalu wychodzą przez czeka_na_zgode",
        "odnotuj_wyslanie",
        id="wypowiedzenie",
        wyslano="2027-03-01T10:00:00Z",
    )
    send_with_consent(room, "wypowiedzenie")
    room.run("odnotuj_doreczenie_konca_najmu", id="koniec", doreczono="2027-03-03")
    room.refused(
        "końca najmu się nie zmienia",
        "odnotuj_doreczenie_konca_najmu",
        id="koniec",
        doreczono="2027-03-04",
    )


@scenario(
    "Koniec najmu: po śmierci najemcy dzień końca przychodzi z decyzją człowieka (art. 691 KC)"
)
def tenant_death_needs_decision():
    room = new_room()
    open_rental(room)
    add_rental_parties(room)
    add_flat(room)
    concluded_lease(room)
    room.run("wpisz_koniec_najmu", id="smierc", najem_id="najem", sposob="smierc_najemcy")

    room.refused(
        "smierc_najemcy_z_decyzja",
        "odnotuj_decyzje_po_smierci_najemcy",
        id="smierc",
        decyzja_id=None,
        koniec_najmu="2027-02-28",
    )
    room.refused(
        "koniec najmu wskazuje zatwierdzoną decyzję inna",
        "odnotuj_decyzje_po_smierci_najemcy",
        id="smierc",
        decyzja_id="brak",
        koniec_najmu="2027-02-28",
    )
    approved_decision(room, "decyzja-smierc", "inna", "najem_umowy_konce", "smierc")
    room.run(
        "odnotuj_decyzje_po_smierci_najemcy",
        id="smierc",
        decyzja_id="decyzja-smierc",
        koniec_najmu="2027-02-28",
    )
    room.refused(
        "koniec_najmu_znany",
        "wpisz_koniec_najmu",
        id="porozumienie",
        najem_id="najem",
        sposob="porozumienie",
    )


@scenario(
    "Żądanie opróżnienia: tylko przy okazjonalnym i instytucjonalnym, z terminem 7 albo 14 dni"
)
def eviction_demand_term():
    room = new_room()
    open_rental(room)
    add_rental_parties(room)
    add_flat(room)
    concluded_lease(room, "zwykly")
    concluded_lease(room, "okazjonalny", "okazjonalny", kaucja=0)
    rule = "żądanie opróżnienia jest przy najmie okazjonalnym"

    for lease_id in ("zwykly", "okazjonalny"):
        room.run(
            "wpisz_koniec_najmu",
            id=f"koniec-{lease_id}",
            najem_id=lease_id,
            sposob="uplyw_czasu",
            koniec_najmu="2028-11-30",
        )
        letter_to_tenant(room, f"zadanie-{lease_id}", "zadanie-oproznienia")

    room.refused(
        rule, "wskaz_zadanie_oproznienia", id="koniec-zwykly", zadanie_wysylka_id="zadanie-zwykly"
    )

    room.run(
        "wskaz_zadanie_oproznienia",
        id="koniec-okazjonalny",
        zadanie_wysylka_id="zadanie-okazjonalny",
    )
    send_with_consent(room, "zadanie-okazjonalny", wyslano="2028-12-01T10:00:00Z")
    room.refused(
        rule,
        "odnotuj_doreczenie_zadania_oproznienia",
        id="koniec-okazjonalny",
        zadanie_doreczono="2028-12-02",
        zadanie_termin="2028-12-08",
    )
    room.run(
        "odnotuj_doreczenie_zadania_oproznienia",
        id="koniec-okazjonalny",
        zadanie_doreczono="2028-12-02",
        zadanie_termin="2028-12-09",
    )


@scenario("Żądanie opróżnienia przy najmie instytucjonalnym ma termin co najmniej 14 dni")
def eviction_demand_institutional():
    room = new_room()
    open_rental(room)
    add_flat(room)
    add_company(room, "spolka")
    add_person(room, "najemca", imiona="Jan", nazwisko="Nowak")
    room.run("nadaj_role_uczestnikowi", uczestnik_id="spolka", rola="wynajmujacy")
    room.run("nadaj_role_uczestnikowi", uczestnik_id="najemca", rola="najemca")
    concluded_lease(room, rodzaj="instytucjonalny", kaucja=0)
    room.run(
        "wpisz_koniec_najmu",
        id="koniec",
        najem_id="najem",
        sposob="uplyw_czasu",
        koniec_najmu="2028-11-30",
    )
    letter_to_tenant(room, "zadanie", "zadanie-oproznienia")
    room.run("wskaz_zadanie_oproznienia", id="koniec", zadanie_wysylka_id="zadanie")
    send_with_consent(room, "zadanie", wyslano="2028-12-01T10:00:00Z")

    room.refused(
        "instytucjonalnym (co najmniej 14 dni)",
        "odnotuj_doreczenie_zadania_oproznienia",
        id="koniec",
        zadanie_doreczono="2028-12-02",
        zadanie_termin="2028-12-15",
    )
    room.run(
        "odnotuj_doreczenie_zadania_oproznienia",
        id="koniec",
        zadanie_doreczono="2028-12-02",
        zadanie_termin="2028-12-16",
    )


# Rent ----------------------------------------------------------------------------------------------


def managed_lease(room, przeplyw="przez_biuro"):
    """A concluded ordinary lease under a management contract."""
    open_rental(room)
    add_rental_parties(room)
    add_flat(room)
    management(room, przeplyw=przeplyw)
    concluded_lease(room)


@scenario(
    "Wpłata: na rachunek biura tylko przy przepływie przez biuro, wprost na rachunek dopuszczony"
)
def rent_payment_account():
    room = new_room()
    managed_lease(room, przeplyw="wprost")
    add_received_document(room, "potwierdzenie", "potwierdzenie_bankowe")
    payment = {
        "najem_id": "najem",
        "chwila": "2026-12-08T10:00:00Z",
        "kwota": 300_000,
        "placacy_uczestnik_id": "najemca",
        "potwierdzenie": "od_wynajmujacego",
        "dokument_id": "potwierdzenie",
    }

    room.refused(
        "na rachunek biura czynsz wpływa tylko przy przepływie przez biuro",
        "wpisz_wplate_czynszu",
        id="do-biura",
        na_rachunek="biura",
        **payment,
    )
    room.refused(
        "rachunek_wynajmujacego_wskazany",
        "wpisz_wplate_czynszu",
        id="bez-rachunku",
        na_rachunek="wynajmujacego",
        **payment,
    )
    add_account(room, "niedopuszczony", "wlascicielka", cel="czynsz")
    room.refused(
        "wpłata idzie na rachunek dopuszczony decyzją i niezastąpiony (R4)",
        "wpisz_wplate_czynszu",
        id="na-niedopuszczony",
        na_rachunek="wynajmujacego",
        rachunek_id="niedopuszczony",
        **payment,
    )
    admitted_account_for(room, "rachunek-czynszu", "wlascicielka", "czynsz")
    room.run(
        "wpisz_wplate_czynszu",
        id="wprost",
        na_rachunek="wynajmujacego",
        rachunek_id="rachunek-czynszu",
        **payment,
    )
    room.refused_sql("wpłaty się nie zmienia", "UPDATE najem_czynsz_wplaty SET kwota = 1")


@scenario(
    "Należność i rozliczenie: poprawka przed wpłatą, rozliczenie w granicach wpłaty i należności"
)
def rent_due_and_settlement():
    room = new_room()
    managed_lease(room)
    add_rent_due(room)
    room.run(
        "popraw_naleznosc",
        id="czynsz-grudzien",
        tytul="czynsz",
        okres="2026-12",
        kwota=290_000,
        termin="2026-12-10",
    )
    rent_paid_to_office(room, kwota=200_000)

    rule = "rozliczenie nie przekracza kwoty wpłaty ani kwoty należności"
    room.refused(
        rule,
        "rozlicz_wplate",
        wplata_id="wplata-grudzien",
        naleznosc_id="czynsz-grudzien",
        kwota=200_001,
    )
    room.run(
        "rozlicz_wplate", wplata_id="wplata-grudzien", naleznosc_id="czynsz-grudzien", kwota=200_000
    )
    rent_paid_to_office(room, "druga-wplata", kwota=200_000)
    room.refused(
        rule,
        "rozlicz_wplate",
        wplata_id="druga-wplata",
        naleznosc_id="czynsz-grudzien",
        kwota=90_001,
    )
    room.run(
        "rozlicz_wplate", wplata_id="druga-wplata", naleznosc_id="czynsz-grudzien", kwota=90_000
    )

    balance = room.run("saldo_naleznosci")[0]
    assert balance["zaplacono"] == 290_000 and balance["zostalo"] == 0, balance
    room.refused(
        "należność zmienia się tylko, dopóki nie pokrywa jej żadna wpłata",
        "popraw_naleznosc",
        id="czynsz-grudzien",
        tytul="czynsz",
        okres="2026-12",
        kwota=300_000,
        termin="2026-12-10",
    )
    room.refused_sql(
        "rozliczenia wpłaty się nie zmienia", "UPDATE najem_czynsz_rozliczenia SET kwota = 1"
    )


@scenario("Przekazanie czynszu (T13): jedna wpłata na rachunek biura, do wynajmującego, raz")
def rent_transfer_to_owner():
    room = new_room()
    managed_lease(room)
    rent_paid_to_office(room)
    admitted_account_for(room, "rachunek-wlascicielki", "wlascicielka", "wyplata_dla_wlasciciela")
    admitted_account_for(room, "rachunek-najemcy", "najemca", "inny", posiadacz="Jan Nowak")
    rule = "przekazanie czynszu wskazuje jedną wpłatę na rachunek biura"
    args = ("najem_czynsz_wplaty", "wplata-grudzien")

    room.refused(
        rule,
        "przygotuj_wyplate",
        **payout_fields("ponad-wplate", *args, 300_001, "wlascicielka", "rachunek-wlascicielki"),
    )
    room.refused(
        rule,
        "przygotuj_wyplate",
        **payout_fields("do-najemcy", *args, 270_000, "najemca", "rachunek-najemcy"),
    )
    room.refused(
        rule,
        "przygotuj_wyplate",
        **payout_fields(
            "bez-wplaty",
            "najem_czynsz_wplaty",
            "brak",
            270_000,
            "wlascicielka",
            "rachunek-wlascicielki",
        ),
    )

    prepare_payout(room, "przekazanie", *args, 270_000, "wlascicielka", "rachunek-wlascicielki")
    room.refused(
        rule,
        "przygotuj_wyplate",
        **payout_fields("drugie", *args, 1_000, "wlascicielka", "rachunek-wlascicielki"),
    )
    room.run("anuluj_wyplate", id="przekazanie")
    prepare_payout(room, "ponowne", *args, 270_000, "wlascicielka", "rachunek-wlascicielki")


def payout_fields(payout_id, podstawa_tabela, podstawa_id, kwota, person_id, account_id):
    return {
        "id": payout_id,
        "podstawa_tabela": podstawa_tabela,
        "podstawa_id": podstawa_id,
        "kwota": kwota,
        "wyliczenie_sciezka": f"wyliczenia/{payout_id}.json",
        "wyliczenie_skrot": f"sha256-{payout_id}",
        "odbiorca_uczestnik_id": person_id,
        "rachunek_id": account_id,
        "utworzono": "2027-01-12T09:00:00Z",
    }


@scenario("Przekazanie czynszu (T13) wymaga przepływu przez biuro według bieżących warunków")
def rent_transfer_needs_flow_through_office():
    room = new_room()
    managed_lease(room)
    rent_paid_to_office(room)
    add_received_document(room, "aneks-zarzadzania", "umowa_z_biurem")
    room.run(
        "wpisz_zdarzenie_umowy",
        id="aneks",
        umowa_id="zarzadzanie",
        rodzaj="aneks",
        chwila="2026-12-20T00:00:00Z",
        dokument_id="aneks-zarzadzania",
    )
    room.run(
        "wpisz_warunki_zarzadzania",
        umowa_id="zarzadzanie",
        kwota_napraw=50_000,
        przeplyw="wprost",
        aneks_zdarzenie_id="aneks",
        od="2026-12-20T00:00:00Z",
    )
    admitted_account_for(room, "rachunek-wlascicielki", "wlascicielka", "wyplata_dla_wlasciciela")

    room.refused(
        "przekazanie czynszu wskazuje jedną wpłatę na rachunek biura przy przepływie przez biuro",
        "przygotuj_wyplate",
        **payout_fields(
            "przekazanie",
            "najem_czynsz_wplaty",
            "wplata-grudzien",
            270_000,
            "wlascicielka",
            "rachunek-wlascicielki",
        ),
    )


@scenario("Zmiana czynszu: od bieżącego czynszu; z porozumienia wskazuje obowiązujący aneks")
def rent_change_from_current_rent():
    room = new_room()
    managed_lease(room)
    rule = "zmiana czynszu zaczyna się od obowiązującego czynszu"
    change = {"najem_id": "najem", "obowiazuje_od": "2027-06-01", "podstawa": "porozumienie stron"}

    room.refused(rule, "wpisz_zmiane_czynszu", stara=310_000, nowa=290_000, **change)
    room.refused(
        "zmiana_zmienia_kwote", "wpisz_zmiane_czynszu", stara=300_000, nowa=300_000, **change
    )

    room.run("przygotuj_aneks_najmu", id="aneks", najem_id="najem", opis="Wyższy czynsz od czerwca")
    room.refused(
        rule, "wpisz_zmiane_czynszu", stara=300_000, nowa=330_000, aneks_id="aneks", **change
    )
    approved_decision(room, "zgoda-aneks", "inna", "najem_umowy_aneksy", "aneks")
    room.run("wskaz_zgode_na_aneks_najmu", id="aneks", decyzja_id="zgoda-aneks")
    add_received_document(room, "podpisany-aneks", "umowa")
    room.run(
        "odnotuj_podpisanie_aneksu_najmu",
        id="aneks",
        dokument_id="podpisany-aneks",
        obowiazuje_od="2027-05-20",
    )
    room.run("wpisz_zmiane_czynszu", stara=300_000, nowa=330_000, aneks_id="aneks", **change)

    # An agreed reduction later starts from the new rent.
    room.refused(rule, "wpisz_zmiane_czynszu", stara=300_000, nowa=280_000, **change)
    room.run("wpisz_zmiane_czynszu", stara=330_000, nowa=280_000, **change)
    room.refused_sql(
        "zmiany czynszu się nie zmienia", "UPDATE najem_czynsz_zmiany SET podstawa = 'inna'"
    )


@scenario(
    "Podwyżka przy najmie zwykłym: wypowiedzenie na piśmie, 3 miesiące, od 1. dnia, co 6 miesięcy"
)
def rent_increase_ordinary_lease():
    room = new_room()
    managed_lease(room)
    letter_to_tenant(room, "podwyzka", "wypowiedzenie-wysokosci-czynszu")
    rule = "podwyżka przy najmie zwykłym obowiązuje od pierwszego dnia miesiąca"
    increase = {
        "najem_id": "najem",
        "stara": 300_000,
        "nowa": 320_000,
        "pismo_wysylka_id": "podwyzka",
        "podstawa": "art. 8a UOPL",
    }

    room.refused(
        rule,
        "wpisz_zmiane_czynszu",
        obowiazuje_od="2027-05-01",
        **{**increase, "pismo_wysylka_id": None},
    )
    # Delivered on 1 February: the three months end on 1 May, so the notice works from 1 June.
    room.refused(
        rule, "wpisz_zmiane_czynszu", doreczono="2027-02-01", obowiazuje_od="2027-05-01", **increase
    )
    room.refused(
        rule, "wpisz_zmiane_czynszu", doreczono="2027-02-01", obowiazuje_od="2027-06-15", **increase
    )
    room.run("wpisz_zmiane_czynszu", doreczono="2027-02-01", obowiazuje_od="2027-06-01", **increase)

    # The next increase comes six months after the previous one started.
    letter_to_tenant(room, "druga-podwyzka", "wypowiedzenie-wysokosci-czynszu")
    second = {**increase, "stara": 320_000, "nowa": 340_000, "pismo_wysylka_id": "druga-podwyzka"}
    room.refused(
        "przy najmie zwykłym czynszu nie podwyższa się częściej niż co 6 miesięcy",
        "wpisz_zmiane_czynszu",
        doreczono="2027-07-01",
        obowiazuje_od="2027-11-01",
        **second,
    )
    room.run("wpisz_zmiane_czynszu", doreczono="2027-08-01", obowiazuje_od="2027-12-01", **second)


@scenario("Podwyżka przy najmie okazjonalnym idzie według umowy, bez reguł art. 8a UOPL")
def rent_increase_occasional_lease():
    room = new_room()
    open_rental(room)
    add_rental_parties(room)
    add_flat(room)
    concluded_lease(room, rodzaj="okazjonalny", kaucja=0)

    room.run(
        "wpisz_zmiane_czynszu",
        najem_id="najem",
        stara=300_000,
        nowa=315_000,
        obowiazuje_od="2027-01-15",
        podstawa="§ 4 umowy: waloryzacja o wskaźnik inflacji",
    )


@scenario("Zaległość: uprzedzenie za zgodą człowieka, dodatkowy termin co najmniej miesiąc")
def arrears_warning_and_extra_month():
    room = new_room()
    managed_lease(room)
    room.refused_sql(
        "zaległość powstaje jako czekająca",
        """INSERT INTO najem_czynsz_zaleglosci (id, najem_id, trzy_okresy_od, wynik)
        VALUES ('od-razu', 'najem', '2027-02-11', 'splacona')""",
    )
    room.run("wpisz_zaleglosc", id="zaleglosc", najem_id="najem", trzy_okresy_od="2027-02-11")

    letter_to_tenant(room, "wyslane-bez-zgody", "uprzedzenie-o-wypowiedzeniu")
    send(room, "wyslane-bez-zgody")
    room.refused(
        "zaległość wskazuje uprzedzenie, które jeszcze nie wyszło",
        "wskaz_uprzedzenie_o_zaleglosci",
        id="zaleglosc",
        uprzedzenie_wysylka_id="wyslane-bez-zgody",
    )

    letter_to_tenant(room, "uprzedzenie", "uprzedzenie-o-wypowiedzeniu")
    room.run("wskaz_uprzedzenie_o_zaleglosci", id="zaleglosc", uprzedzenie_wysylka_id="uprzedzenie")
    room.refused(
        "uprzedzenie o zamiarze wypowiedzenia wychodzi przez czeka_na_zgode",
        "odnotuj_wyslanie",
        id="uprzedzenie",
        wyslano="2027-02-25T10:00:00Z",
    )
    room.refused(
        "wypowiedzenie_po_dodatkowym_terminie",
        "odnotuj_wynik_zaleglosci",
        id="zaleglosc",
        wynik="wypowiedzenie",
    )
    send_with_consent(room, "uprzedzenie")

    rule = "dodatkowy termin trwa co najmniej miesiąc od doręczenia uprzedzenia"
    room.refused(
        rule, "ustal_dodatkowy_termin_zaleglosci", id="zaleglosc", dodatkowy_termin_do="2027-03-26"
    )
    room.run("odnotuj_doreczenie", id="uprzedzenie", doreczono="2027-02-26T10:00:00Z")
    room.refused(
        rule, "ustal_dodatkowy_termin_zaleglosci", id="zaleglosc", dodatkowy_termin_do="2027-03-25"
    )
    room.run("ustal_dodatkowy_termin_zaleglosci", id="zaleglosc", dodatkowy_termin_do="2027-03-26")
    room.run("odnotuj_wynik_zaleglosci", id="zaleglosc", wynik="wypowiedzenie")
    room.refused(
        "zaległości się nie zmienia",
        "odnotuj_wynik_zaleglosci",
        id="zaleglosc",
        wynik="splacona",
    )


@scenario(
    "Odczyty liczników się nie zmienia; dane dla gminy tylko przy zarządzaniu lokalem mieszkalnym"
)
def meter_readings_and_municipality_data():
    room = new_room()
    open_rental(room)
    add_rental_parties(room)
    add_flat(room)
    add_flat(
        room, "uzytkowy", rodzaj="lokal_uzytkowy", numer_lokalu="U1", numer_kw="WR1K/00054321/0"
    )
    concluded_lease(room)
    prepare_lease(room, "uzytkowego", nieruchomosc_id="uzytkowy", kaucja=0)
    data = {"okres": "2027", "czynsz": 300_000, "gmina": "Wrocław"}
    rule = "dane dla gminy przygotowuje się przy zawartej umowie o zarządzanie"

    room.refused(rule, "przygotuj_dane_dla_gminy", najem_id="najem", **data)
    management(room)
    room.refused(rule, "przygotuj_dane_dla_gminy", najem_id="uzytkowego", **data)
    room.run("przygotuj_dane_dla_gminy", najem_id="najem", **data)
    room.run("popraw_dane_dla_gminy", id=1, czynsz=310_000, powierzchnia=4820)
    room.refused(
        "wyslanie_po_oddaniu",
        "odnotuj_wyslanie_danych_dla_gminy",
        id=1,
        wyslano="2027-03-05",
        wyslal="czlowiek",
    )
    room.run("odnotuj_oddanie_danych_dla_gminy", id=1, oddano="2027-03-01")
    room.refused(
        "oddanych danych dla gminy się nie zmienia",
        "popraw_dane_dla_gminy",
        id=1,
        czynsz=320_000,
        powierzchnia=4820,
    )

    room.run(
        "wpisz_odczyt_licznika",
        najem_id="najem",
        licznik="gaz",
        numer="G-77",
        chwila="2027-01-05T09:00:00Z",
        stan="1 024 m³",
        zdjecie_sciezka="odczyty/gaz.jpg",
        zdjecie_skrot="sha256-gaz",
    )
    room.refused_sql(
        "odczytu licznika się nie zmienia", "UPDATE najem_czynsz_odczyty SET stan = '0 m³'"
    )


# Deposit -------------------------------------------------------------------------------------------


@scenario("Kaucja: pobranie w granicy umowy, z czynszem z dnia zawarcia, na rachunek dopuszczony")
def deposit_collection():
    room = new_room()
    managed_lease(room)
    deposit = {
        "najem_id": "najem",
        "krotnosc": 2.0,
        "czynsz_przy_zawarciu": 300_000,
        "wplacono": "2026-11-25T13:00:00Z",
        "lezy": "u_biura",
    }
    rule = "pobrana kaucja mieści się w kaucji z umowy najmu"

    room.refused(rule, "wpisz_pobranie_kaucji", id="za-duza", kwota=600_001, **deposit)
    room.refused(
        rule,
        "wpisz_pobranie_kaucji",
        id="inny-czynsz",
        kwota=600_000,
        **{**deposit, "czynsz_przy_zawarciu": 290_000},
    )
    add_account(room, "niedopuszczony", "wlascicielka", cel="kaucja")
    room.refused(
        "kaucja idzie na rachunek dopuszczony decyzją i niezastąpiony (R4)",
        "wpisz_pobranie_kaucji",
        id="na-niedopuszczony",
        kwota=600_000,
        **{**deposit, "lezy": "u_wynajmujacego", "rachunek_id": "niedopuszczony"},
    )
    room.run("wpisz_pobranie_kaucji", id="kaucja", kwota=600_000, **deposit)
    room.refused_sql(
        "pobrania kaucji się nie zmienia", "UPDATE najem_kaucja_pobrania SET kwota = 1"
    )


@scenario("Kaucja: rozliczenia nie pomija się, gdy kaucję pobrano")
def deposit_settlement_stage():
    room = new_room()
    managed_lease(room)
    move_to(room, "projekt_umowy", "podpisanie_umowy", "wydanie_lokalu")
    signed_protocol(room, "wydanie")
    move_to(room, "trwanie_najmu", "koniec_najmu", "zwrot_lokalu")
    signed_protocol(room, "zwrot", chwila="2027-08-31T10:00:00Z")
    deposit_at_office(room)

    refused_stage(
        room, "po zwrocie lokalu z pobraną kaucją przychodzi rozliczenie kaucji", "po_najmie"
    )
    move_to(room, "rozliczenie_kaucji", "po_najmie")


@scenario("Kaucja: wykorzystanie na zaległość tylko przy najmie instytucjonalnym (art. 19f ust. 7)")
def deposit_used_only_institutional():
    room = new_room()
    managed_lease(room)
    deposit_at_office(room)
    room.run("wpisz_zaleglosc", id="zaleglosc", najem_id="najem", trzy_okresy_od="2027-02-11")
    rule = "kaucję wykorzystuje się na zaległość tego najmu tylko przy najmie instytucjonalnym"

    room.refused(
        rule,
        "wpisz_wykorzystanie_kaucji",
        pobranie_id="kaucja",
        kwota=100_000,
        chwila="2027-02-12",
        zaleglosc_id="zaleglosc",
    )

    institutional_deposit_used(room)
    room.run("odnotuj_uzupelnienie_kaucji", id=1, uzupelniono="2027-03-01")
    room.refused(
        "wykorzystania kaucji się nie zmienia",
        "odnotuj_uzupelnienie_kaucji",
        id=1,
        uzupelniono="2027-03-02",
    )


def institutional_deposit_used(room):
    """An institutional lease whose deposit covered arrears; the landlord is a company."""
    add_company(room, "spolka")
    prepare_lease(room, "instytucjonalny", "instytucjonalny", kaucja=600_000)
    room.run(
        "wpisz_zaleglosc",
        id="zaleglosc-instytucjonalna",
        najem_id="instytucjonalny",
        trzy_okresy_od="2027-02-11",
    )
    room.run(
        "wpisz_pobranie_kaucji",
        id="kaucja-instytucjonalna",
        najem_id="instytucjonalny",
        kwota=600_000,
        krotnosc=2.0,
        czynsz_przy_zawarciu=300_000,
        wplacono="2026-11-25T13:00:00Z",
        lezy="u_biura",
    )
    room.run(
        "wpisz_wykorzystanie_kaucji",
        pobranie_id="kaucja-instytucjonalna",
        kwota=100_000,
        chwila="2027-02-12",
        zaleglosc_id="zaleglosc-instytucjonalna",
        uzupelnic_do="2027-03-12",
    )


@scenario(
    "Rozliczenie kaucji: waloryzacja nie mniej niż pobrana; zatwierdzenie po zgodzie na potrącenia"
)
def deposit_settlement_rules():
    room = new_room()
    managed_lease(room)
    deposit_at_office(room)

    room.refused(
        "kaucja zwaloryzowana nie jest mniejsza niż pobrana",
        "przygotuj_rozliczenie_kaucji",
        **settlement_fields(kwota_zwaloryzowana=599_999),
    )
    room.refused_sql(
        "rozliczenie kaucji powstaje jako projekt",
        """INSERT INTO najem_kaucja_rozliczenia (id, pobranie_id, oproznienie, czynsz_w_dniu_zwrotu,
          kwota_zwaloryzowana, wyliczenie_sciezka, wyliczenie_skrot, stan)
        VALUES ('od-razu', 'kaucja', '2027-08-31', 320000, 640000, 'w.json', 'sha', 'zatwierdzone')""",
    )
    deposit_settlement(room)
    room.refused_sql(
        "kaucja zwaloryzowana nie jest mniejsza niż pobrana",
        "UPDATE najem_kaucja_rozliczenia SET kwota_zwaloryzowana = 1",
    )

    add_deduction(room, "bez-zgody", approved=False)
    rule = "rozliczenie kaucji jest zatwierdzone, gdy każde potrącenie ma zatwierdzoną decyzję"
    room.refused(rule, "zatwierdz_rozliczenie_kaucji", id="rozliczenie")
    room.refused(
        "potrącenie wskazuje zatwierdzoną decyzję inna",
        "wskaz_zgode_na_potracenie",
        id="bez-zgody",
        decyzja_id="brak",
    )
    approved_decision(room, "zgoda-bez-zgody", "inna", "najem_kaucja_potracenia", "bez-zgody")
    room.run("wskaz_zgode_na_potracenie", id="bez-zgody", decyzja_id="zgoda-bez-zgody")
    add_deduction(room, "ponad-kaucje", kwota=600_001)
    room.refused(rule, "zatwierdz_rozliczenie_kaucji", id="rozliczenie")


def settlement_fields(**fields):
    return {
        "id": "rozliczenie",
        "pobranie_id": "kaucja",
        "oproznienie": "2027-08-31",
        "czynsz_w_dniu_zwrotu": 320_000,
        "kwota_zwaloryzowana": 640_000,
        "wyliczenie_sciezka": "wyliczenia/rozliczenie.json",
        "wyliczenie_skrot": "sha256-rozliczenie",
        **fields,
    }


@scenario(
    "Rozliczenie kaucji: projekt, zatwierdzone, wysłane; po zatwierdzeniu stoi z potrąceniami"
)
def deposit_settlement_states():
    room = new_room()
    managed_lease(room)
    deposit_at_office(room)
    deposit_settlement(room)
    add_deduction(room, "potracenie")
    letter_to_tenant(room, "rozliczenie-kaucji", "rozliczenie-kaucji")

    room.refused(
        "rozliczenie kaucji przechodzi: projekt, zatwierdzone, wysłane",
        "odnotuj_wyslanie_rozliczenia_kaucji",
        id="rozliczenie",
        wysylka_id="rozliczenie-kaucji",
    )
    room.run("zatwierdz_rozliczenie_kaucji", id="rozliczenie")
    room.refused_sql(
        "zatwierdzonego rozliczenia kaucji się nie zmienia",
        "UPDATE najem_kaucja_rozliczenia SET czynsz_w_dniu_zwrotu = 1",
    )
    room.refused(
        "potrącenie dopisuje się do projektu rozliczenia kaucji",
        "dopisz_potracenie_z_kaucji",
        id="pozne",
        rozliczenie_id="rozliczenie",
        tytul="spóźnione",
        kwota=1,
        dokument_id="podstawa-potracenie",
    )
    room.refused_sql(
        "potrącenia się nie zmienia",
        "UPDATE najem_kaucja_potracenia SET kwota = 1",
    )
    room.run(
        "odnotuj_wyslanie_rozliczenia_kaucji", id="rozliczenie", wysylka_id="rozliczenie-kaucji"
    )


@scenario("Zwrot kaucji u biura: wypłata według zatwierdzonego rozliczenia, do najemcy, raz")
def deposit_return_payout():
    room = new_room()
    managed_lease(room)
    deposit_at_office(room)
    deposit_settlement(room)
    add_deduction(room, "potracenie")
    admitted_account_for(room, "rachunek-najemcy", "najemca", "zwrot_kaucji", posiadacz="Jan Nowak")
    admitted_account_for(room, "rachunek-wlascicielki", "wlascicielka", "zwrot_kaucji")
    rule = "zwrot kaucji u biura wskazuje zatwierdzone rozliczenie"
    args = ("najem_kaucja_rozliczenia", "rozliczenie")

    room.refused(
        rule,
        "przygotuj_wyplate",
        **payout_fields("przed-zatwierdzeniem", *args, 600_000, "najemca", "rachunek-najemcy"),
    )
    room.run("zatwierdz_rozliczenie_kaucji", id="rozliczenie")
    room.refused(
        rule,
        "przygotuj_wyplate",
        **payout_fields("zla-kwota", *args, 640_000, "najemca", "rachunek-najemcy"),
    )
    room.refused(
        rule,
        "przygotuj_wyplate",
        **payout_fields("do-wlascicielki", *args, 600_000, "wlascicielka", "rachunek-wlascicielki"),
    )

    prepare_payout(room, "zwrot", *args, 600_000, "najemca", "rachunek-najemcy")
    room.refused(
        rule,
        "przygotuj_wyplate",
        **payout_fields("drugi", *args, 600_000, "najemca", "rachunek-najemcy"),
    )


# Defects -------------------------------------------------------------------------------------------


@scenario(
    "Usterka: spór rozstrzyga decyzja człowieka; zlecenie tylko naprawy obciążającej wynajmującego"
)
def defect_dispute():
    room = new_room()
    managed_lease(room)
    report_defect(room, obciaza="spor")

    room.refused(
        "spór o naprawę rozstrzyga zatwierdzona decyzja inna",
        "ustal_kogo_obciaza_usterka",
        id="usterka",
        obciaza="wynajmujacy",
    )
    room.refused("zlecenie_obciaza_wynajmujacego", "zlec_naprawe_usterki", **repair_fields())
    approved_decision(room, "spor", "inna", "najem_usterki_zgloszenia", "usterka")
    room.run(
        "ustal_kogo_obciaza_usterka", id="usterka", obciaza="wynajmujacy", spor_decyzja_id="spor"
    )
    order_repair(room)

    room.refused_sql(
        "zgłoszenie usterki powstaje bez decyzji, zlecenia i usunięcia",
        """INSERT INTO najem_usterki_zgloszenia (id, chwila, opis, uniemozliwia, obciaza, zlecil, zlecono,
          wykonawca, koszt)
        VALUES ('od-razu', '2027-01-20', 'opis', 0, 'wynajmujacy', 'c', '2027-01-21', 'w', 1)""",
    )


def repair_fields(**fields):
    return {
        "id": "usterka",
        "koszt": 30_000,
        "zlecil": "tozsamosc-posrednika",
        "zlecono": "2027-01-21T09:00:00Z",
        "wykonawca": "Hydraulik Jan Kowalski",
        **fields,
    }


@scenario("Usterka: naprawa ponad kwotę z umowy o zarządzanie za zgodą wynajmującego i człowieka")
def defect_repair_above_limit():
    room = new_room()
    managed_lease(room)
    report_defect(room)
    rule = "naprawa ponad kwotę napraw z umowy o zarządzanie wymaga zgody wynajmującego"

    room.refused(rule, "zlec_naprawe_usterki", **repair_fields(koszt=50_001))

    add_inflow(room, "zgoda-na-naprawe", "wlascicielka")
    room.run(
        "wpisz_zgode",
        id="zgoda-wynajmujacej",
        uczestnik_id="wlascicielka",
        rodzaj="zgoda_na_naprawe",
        tresc="Zgadzam się na wymianę syfonu za 800 zł.",
        zlozono="2027-01-20T12:00:00Z",
        wplyw_id="zgoda-na-naprawe",
        trwaly_nosnik=1,
        dotyczy_tabela="najem_usterki_zgloszenia",
        dotyczy_id="usterka",
    )
    room.refused(
        rule,
        "zlec_naprawe_usterki",
        **repair_fields(koszt=80_000, zgoda_wynajmujacego_id="zgoda-wynajmujacej"),
    )
    approved_decision(room, "zlecenie", "inna", "najem_usterki_zgloszenia", "usterka")
    order_repair(
        room,
        koszt=80_000,
        zgoda_wynajmujacego_id="zgoda-wynajmujacej",
        zlecenie_decyzja_id="zlecenie",
    )

    room.refused(
        "zgłoszenia usterki się nie zmienia",
        "zlec_naprawe_usterki",
        **repair_fields(koszt=10_000),
    )


@scenario("Usterka: usunięcie potwierdza najemca wiadomością od siebie; przeglądów się nie zmienia")
def defect_removal_confirmed_by_tenant():
    room = new_room()
    managed_lease(room)
    report_defect(room)
    order_repair(room)

    add_inflow(room, "od-wlascicielki", "wlascicielka")
    room.refused(
        "usunięcie usterki potwierdza najemca wiadomością od siebie",
        "odnotuj_usuniecie_usterki",
        id="usterka",
        usunieto="2027-01-22",
        potwierdzenie_wplyw_id="od-wlascicielki",
    )
    room.refused(
        "usuniecie_z_potwierdzeniem_najemcy",
        "odnotuj_usuniecie_usterki",
        id="usterka",
        usunieto="2027-01-22",
        potwierdzenie_wplyw_id=None,
    )
    add_inflow(room, "od-najemcy", "najemca")
    room.run(
        "odnotuj_usuniecie_usterki",
        id="usterka",
        usunieto="2027-01-22",
        potwierdzenie_wplyw_id="od-najemcy",
    )

    room.run("wpisz_przeglad", data="2027-02-01", wynik="Lokal bez uwag")
    room.refused_sql(
        "przeglądu się nie zmienia", "UPDATE najem_usterki_przeglady SET wynik = 'inny'"
    )


# R9: no deletion, no replacement -----------------------------------------------------------------


def najem_tables(room):
    return [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'najem\\_%' ESCAPE '\\'"
        )
    ]


@scenario("R9: wierszy typu najmu się nie usuwa ani nie zastępuje przez REPLACE i upsert")
def no_deletion_or_replacement():
    room = new_room()
    full_managed_rental(room)

    # The managed ordinary lease has no substitute flat and no use of the deposit during the lease.
    prepare_lease(room, "okazjonalny", "okazjonalny", kaucja=0)
    room.run(
        "wpisz_lokal_zastepczy", najem_id="okazjonalny", adres="ul. Lipowa 4", wskazano="2026-11-20"
    )
    institutional_deposit_used(room)

    tables = najem_tables(room)
    assert len(tables) == 20, tables
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
