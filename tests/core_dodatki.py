"""Scenarios of the core additions for the case types (spec/cases/sprawa): the views
of approved decisions (sprawa_decyzje_zatwierdzone), admitted accounts
(sprawa_rachunki_dopuszczone) and the start of work under a contract (sprawa_umowy_praca_od), the
last day to withdraw from a contract (odstapienie_do) and the handover protocols
(sprawa-protokoly).

Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
allows and a write it refuses. The views hold one rule each; the scenarios check both what a view
shows and that the rules of the core read it.
"""

from core import (
    WITHDRAWAL_ENDS,
    add_account,
    add_consent,
    add_flat,
    add_person,
    add_received_document,
    admitted_account,
    approve,
    approved_decision,
    ask_for_decision,
    check,
    checked_and_approved_payout,
    conclude_contract,
    new_room,
    prepare_contract,
    prepare_payout,
)

SCENARIOS = []


def scenario(title):
    def register(function):
        SCENARIOS.append((title, function))

        return function

    return register


def approved_ids(room):
    return sorted(row["id"] for row in room.run("decyzje_zatwierdzone"))


def admitted_ids(room):
    return [row["id"] for row in room.run("rachunki_dopuszczone")]


def withdraw_approval(room, decision_id, replaced_id, rodzaj, dotyczy_tabela, dotyczy_id):
    """A new decision on the same row that refuses what the replaced one allowed."""
    ask_for_decision(
        room,
        decision_id,
        rodzaj,
        dotyczy_tabela,
        dotyczy_id,
        zlozono="2026-11-21T09:00:00Z",
        zastepuje_decyzja_id=replaced_id,
    )
    approve(room, decision_id, "2026-11-21T10:00:00Z", rozstrzygniecie="odmowa")


# Approved decisions ------------------------------------------------------------------------------


@scenario(
    "Zatwierdzona decyzja to zgoda albo zgoda z warunkiem; czekająca, odmowa i zastąpiona nie"
)
def approved_decisions_view():
    room = new_room()
    ask_for_decision(room, "czeka", "inna", "sprawa_terminy_wpisy", "termin-1")
    ask_for_decision(room, "zbedna", "inna", "sprawa_terminy_wpisy", "termin-2")
    room.run("uznaj_prosbe_za_zbedna", id="zbedna", powod_zbednosci="termin przesunięty")
    ask_for_decision(room, "odmowa", "inna", "sprawa_terminy_wpisy", "termin-3")
    approve(room, "odmowa", rozstrzygniecie="odmowa")
    approved_decision(room, "zgoda", "inna", "sprawa_terminy_wpisy", "termin-4")
    ask_for_decision(room, "z-warunkiem", "inna", "sprawa_terminy_wpisy", "termin-5")
    room.run(
        "rozstrzygnij_decyzje",
        id="z-warunkiem",
        rozstrzygniecie="zgoda_z_warunkiem",
        warunek="po okazaniu odpisu z księgi",
        podpisal="tozsamosc-posrednika",
        podpis="podpis-kluczem-xkeys",
        podpisano="2026-11-03T12:00:00Z",
    )
    assert approved_ids(room) == ["z-warunkiem", "zgoda"]

    # A refusal that replaces an approval takes it out of the view.
    withdraw_approval(room, "cofniecie", "zgoda", "inna", "sprawa_terminy_wpisy", "termin-4")
    assert approved_ids(room) == ["z-warunkiem"]


@scenario("Reguły rdzenia czytają widok: zastąpiona zgoda nie potwierdza terminu")
def rules_read_approved_decisions():
    room = new_room()
    room.run(
        "wpisz_termin",
        id="termin",
        rodzaj="z_umowy",
        co="Przekazanie lokalu najemcy",
        koniec="2026-11-30T10:00:00Z",
        podstawa="§ 3 umowy najmu",
        pilnuje_stanowisko="opiekun_najmu",
    )
    approved_decision(room, "zgoda", "potwierdzenie_terminu", "sprawa_terminy_wpisy", "termin")
    withdraw_approval(
        room, "cofniecie", "zgoda", "potwierdzenie_terminu", "sprawa_terminy_wpisy", "termin"
    )

    room.refused(
        "termin potwierdza tylko zatwierdzona decyzja potwierdzenie_terminu",
        "odnotuj_potwierdzenie_terminu",
        id="termin",
        potwierdzenie_decyzja_id="zgoda",
    )

    approved_decision(
        room,
        "nowa-zgoda",
        "potwierdzenie_terminu",
        "sprawa_terminy_wpisy",
        "termin",
        zlozono="2026-11-22T09:00:00Z",
        podpisano="2026-11-22T10:00:00Z",
    )
    room.run("odnotuj_potwierdzenie_terminu", id="termin", potwierdzenie_decyzja_id="nowa-zgoda")


# Admitted accounts -------------------------------------------------------------------------------


@scenario("Rachunek dopuszczony: zatwierdzona decyzja dopuszczenie_rachunku i brak zastępcy")
def admitted_accounts_view():
    room = new_room()
    add_person(room, "najemca")
    add_account(room, "bez-decyzji", "najemca")
    admitted_account(room, "dopuszczony", "najemca")
    admitted_account(room, "zastapiony", "najemca")
    assert admitted_ids(room) == ["dopuszczony", "zastapiony"]

    add_account(room, "nowy-numer", "najemca", numer="PL27114020040000300201355387")
    room.run(
        "zastap_rachunek",
        id="zastapiony",
        zastapiony_przez_id="nowy-numer",
        zastapil="tozsamosc-agenta",
        zastapiono="2026-11-08T09:00:00Z",
    )
    assert admitted_ids(room) == ["dopuszczony"]

    withdraw_approval(
        room,
        "cofniecie",
        "dopuszczenie-dopuszczony",
        "dopuszczenie_rachunku",
        "sprawa_rachunki_wpisy",
        "dopuszczony",
    )
    assert admitted_ids(room) == []


@scenario(
    "Wypłata czyta rachunki dopuszczone: po cofnięciu dopuszczenia nie powstaje ani nie wychodzi"
)
def payouts_read_admitted_accounts():
    room = new_room()
    add_person(room, "najemca")
    admitted_account(room, "rachunek", "najemca")
    checked_and_approved_payout(room, "zwrot-kaucji", "najemca", "rachunek")
    add_received_document(room, "potwierdzenie-przelewu", "potwierdzenie_bankowe")

    withdraw_approval(
        room,
        "cofniecie",
        "dopuszczenie-rachunek",
        "dopuszczenie_rachunku",
        "sprawa_rachunki_wpisy",
        "rachunek",
    )

    room.refused(
        "wykonanie wymaga zgody na wypłatę po aktualnym sprawdzeniu niezastąpionego i dopuszczonego",
        "odnotuj_wykonanie_wyplaty",
        id="zwrot-kaucji",
        wykonal="tozsamosc-posrednika",
        wykonano="2026-11-21T13:00:00Z",
        potwierdzenie_dokument_id="potwierdzenie-przelewu",
    )
    room.refused(
        "wypłata idzie na rachunek odbiorcy dopuszczony decyzją i niezastąpiony",
        "przygotuj_wyplate",
        id="druga-wyplata",
        podstawa_tabela="najem_kaucje",
        podstawa_id="rozliczenie-kaucji",
        kwota=300_000,
        wyliczenie_sciezka="wyliczenia/druga-wyplata.json",
        wyliczenie_skrot="sha256-druga-wyplata",
        odbiorca_uczestnik_id="najemca",
        rachunek_id="rachunek",
        utworzono="2026-11-21T11:00:00Z",
    )

    # A new admission of the same account lets a new payout through.
    ask_for_decision(
        room,
        "ponowne-dopuszczenie",
        "dopuszczenie_rachunku",
        "sprawa_rachunki_wpisy",
        "rachunek",
        zlozono="2026-11-22T09:00:00Z",
        zastepuje_decyzja_id="cofniecie",
    )
    approve(room, "ponowne-dopuszczenie", "2026-11-22T10:00:00Z")
    prepare_payout(room, "druga-wyplata", "najemca", "rachunek", utworzono="2026-11-22T11:00:00Z")


# The last day to withdraw and the start of work --------------------------------------------------


def contract_for(room, contract_id, party_id):
    """A contract of a checked party, handed over to signing."""
    check(room, f"tozsamosc-{contract_id}", "tozsamosc", uczestnik_id=party_id)
    prepare_contract(room, contract_id, [party_id])
    room.run("przekaz_umowe_do_podpisu", id=contract_id)


@scenario(
    "Umowa z konsumentem poza lokalem albo na odległość ma odstapienie_do (art. 27 upk); inna nie"
)
def withdrawal_last_day():
    room = new_room()
    add_person(room, "konsumentka", konsument=1)
    add_person(room, "przedsiebiorca", imiona="Piotr", nazwa="Nowak Inwestycje", ceidg=1)
    refusal = (
        "ma odstapienie_do nie wcześniej niż koniec terminu z art. 27 upk; inna umowa go nie ma"
    )

    # Fourteen days from the day of conclusion, which is not counted (art. 111 § 2 KC).
    contract_for(room, "na-odleglosc", "konsumentka")
    for odstapienie_do in (None, "2026-11-17"):
        room.refused(
            refusal,
            "odnotuj_zawarcie_umowy",
            **concluded_fields(room, "na-odleglosc", "na_odleglosc", odstapienie_do),
        )
    room.refused(
        "odstapienie_do_jest_dniem_po_zawarciu",
        "odnotuj_zawarcie_umowy",
        **concluded_fields(room, "na-odleglosc", "na_odleglosc", "2026-11-18T23:59:00Z"),
    )
    conclude_contract(
        room, "na-odleglosc", sposob_zawarcia="na_odleglosc", odstapienie_do=WITHDRAWAL_ENDS
    )

    # After an unannounced visit the consumer has 30 days (art. 27 ust. 2 upk).
    contract_for(room, "wizyta", "konsumentka")
    room.refused(
        refusal,
        "odnotuj_zawarcie_umowy",
        **concluded_fields(room, "wizyta", "nieumowiona_wizyta", WITHDRAWAL_ENDS),
    )
    conclude_contract(
        room, "wizyta", sposob_zawarcia="nieumowiona_wizyta", odstapienie_do="2026-12-04"
    )

    # Without the information on withdrawal the right lasts 12 months longer (art. 29 ust. 1 upk).
    contract_for(room, "bez-informacji", "konsumentka")
    conclude_contract(
        room, "bez-informacji", sposob_zawarcia="poza_lokalem", odstapienie_do="2027-11-18"
    )

    # No right to withdraw: at the office, or no consumer among the parties.
    contract_for(room, "w-lokalu", "konsumentka")
    room.refused(
        refusal,
        "odnotuj_zawarcie_umowy",
        **concluded_fields(room, "w-lokalu", "w_lokalu_biura", WITHDRAWAL_ENDS),
    )
    conclude_contract(room, "w-lokalu")
    contract_for(room, "przedsiebiorcy", "przedsiebiorca")
    room.refused(
        refusal,
        "odnotuj_zawarcie_umowy",
        **concluded_fields(room, "przedsiebiorcy", "na_odleglosc", WITHDRAWAL_ENDS),
    )
    conclude_contract(room, "przedsiebiorcy", sposob_zawarcia="na_odleglosc")

    room.refused_sql(
        "dane zawarcia umowy się nie zmieniają",
        "UPDATE sprawa_umowy_wpisy SET odstapienie_do = '2026-12-31' WHERE id = 'na-odleglosc'",
    )


def concluded_fields(room, contract_id, sposob_zawarcia, odstapienie_do):
    """A try to conclude a contract on 2026-11-04, with its own signed document and policy."""
    for document_id, rodzaj in (
        (f"proba-umowy-{contract_id}", "umowa_z_biurem"),
        (f"proba-polisy-{contract_id}", "polisa"),
    ):
        if room.value("SELECT count(*) FROM sprawa_dokumenty_wpisy WHERE id = :id", id=document_id):
            continue
        add_received_document(room, document_id, rodzaj)

    return {
        "id": contract_id,
        "forma": "elektroniczna",
        "sposob_zawarcia": sposob_zawarcia,
        "zawarto": "2026-11-04T12:00:00Z",
        "podpisal_za_biuro": "tozsamosc-posrednika",
        "dokument_id": f"proba-umowy-{contract_id}",
        "polisa_dokument_id": f"proba-polisy-{contract_id}",
        "odstapienie_do": odstapienie_do,
    }


@scenario("Praca od: zawarcie, dzień po odstapienie_do albo wcześniejsze żądanie konsumenta")
def work_start():
    room = new_room()
    add_person(room, "konsumentka", konsument=1)

    contract_for(room, "w-lokalu", "konsumentka")
    conclude_contract(room, "w-lokalu")
    contract_for(room, "bez-zadania", "konsumentka")
    conclude_contract(
        room, "bez-zadania", sposob_zawarcia="na_odleglosc", odstapienie_do=WITHDRAWAL_ENDS
    )
    contract_for(room, "z-zadaniem", "konsumentka")
    conclude_contract(
        room, "z-zadaniem", sposob_zawarcia="na_odleglosc", odstapienie_do=WITHDRAWAL_ENDS
    )
    contract_for(room, "zadanie-w-umowie", "konsumentka")
    conclude_contract(
        room, "zadanie-w-umowie", sposob_zawarcia="na_odleglosc", odstapienie_do=WITHDRAWAL_ENDS
    )
    contract_for(room, "zadanie-po-terminie", "konsumentka")
    conclude_contract(
        room,
        "zadanie-po-terminie",
        sposob_zawarcia="na_odleglosc",
        odstapienie_do=WITHDRAWAL_ENDS,
    )
    prepare_contract(room, "projekt", [])

    add_consent(room, "zadanie", "konsumentka", "zadanie_rozpoczecia_pracy")
    for event_id, umowa_id, chwila in (
        ("zadanie-1", "z-zadaniem", "2026-11-06T09:00:00Z"),
        ("zadanie-2", "z-zadaniem", "2026-11-08T09:00:00Z"),
        # A request signed with the contract, dated a moment before the office signed it.
        ("zadanie-3", "zadanie-w-umowie", "2026-11-04T11:00:00Z"),
        ("zadanie-4", "zadanie-po-terminie", "2026-11-25T09:00:00Z"),
    ):
        room.run(
            "wpisz_zdarzenie_umowy",
            id=event_id,
            umowa_id=umowa_id,
            rodzaj="zadanie_rozpoczecia_pracy",
            chwila=chwila,
            zgoda_id="zadanie",
        )

    work_from = {row["umowa_id"]: row["od"] for row in room.run("praca_od_umow")}
    assert work_from == {
        "w-lokalu": "2026-11-04T12:00:00Z",
        "bez-zadania": "2026-11-19",
        "z-zadaniem": "2026-11-06T09:00:00Z",
        "zadanie-w-umowie": "2026-11-04T12:00:00Z",
        "zadanie-po-terminie": "2026-11-19",
    }, work_from


# Handover protocols ------------------------------------------------------------------------------


def draft_protocol(room, protocol_id="protokol", rodzaj="wydanie"):
    room.run(
        "przygotuj_protokol",
        id=protocol_id,
        rodzaj=rodzaj,
        nieruchomosc_id="lokal",
        chwila="2026-11-10T10:00:00Z",
    )


def add_item(room, protocol_id="protokol", **fields):
    room.run(
        "dopisz_pozycje_protokolu",
        **{
            "protokol_id": protocol_id,
            "rodzaj": "wyposazenie",
            "nazwa": "pralka",
            "opis_stanu": "sprawna, rysa na obudowie",
            **fields,
        },
    )


@scenario("Protokół powstaje jako projekt; podpisany ma otrzymany dokument rodzaju protokol")
def protocol_signed_with_document():
    room = new_room()
    add_flat(room)
    room.refused_sql(
        "protokół powstaje jako projekt",
        """INSERT INTO sprawa_protokoly_wpisy (id, rodzaj, nieruchomosc_id, chwila, stan, dokument_id)
        VALUES ('od-razu', 'wydanie', 'lokal', '2026-11-10T10:00:00Z', 'podpisany', NULL)""",
    )
    draft_protocol(room)

    add_received_document(room, "pismo", "pismo")
    room.run("wpisz_wymagany_dokument", id="protokol-w-drodze", rodzaj="protokol", nazwa="Protokół")
    refusal = "podpisany protokół wskazuje otrzymany dokument rodzaju protokol"
    for dokument_id in (None, "pismo", "protokol-w-drodze"):
        room.refused(refusal, "podpisz_protokol", id="protokol", dokument_id=dokument_id)

    add_received_document(room, "podpisany", "protokol")
    room.refused_sql(
        "podpisany_ma_dokument",
        "UPDATE sprawa_protokoly_wpisy SET dokument_id = 'podpisany' WHERE id = 'protokol'",
    )
    room.run("podpisz_protokol", id="protokol", dokument_id="podpisany")
    assert room.run("protokoly")[0]["stan"] == "podpisany"


@scenario("Pozycje protokołu: licznik z numerem i odczytem, zdjęcie ze skrótem")
def protocol_items():
    room = new_room()
    add_flat(room)
    draft_protocol(room)

    room.refused(
        "licznik_ma_numer_i_odczyt",
        "dopisz_pozycje_protokolu",
        protokol_id="protokol",
        rodzaj="licznik",
        nazwa="licznik wody zimnej",
        numer="W-1203",
    )
    room.refused(
        "zdjecie_ze_skrotem",
        "dopisz_pozycje_protokolu",
        protokol_id="protokol",
        rodzaj="pomieszczenie",
        nazwa="łazienka",
        zdjecie_sciezka="protokoly/lazienka.jpg",
    )
    add_item(
        room,
        rodzaj="licznik",
        nazwa="licznik wody zimnej",
        numer="W-1203",
        opis_stanu="45,210 m³",
    )
    add_item(room, rodzaj="klucze", nazwa="klucze do skrzynki pocztowej", opis_stanu="2 sztuki")
    assert len(room.run("pozycje_protokolow")) == 2


@scenario("Po podpisie protokołu i jego pozycji się nie zmienia; dokumentu się nie odrzuca")
def protocol_frozen_after_signing():
    room = new_room()
    add_flat(room)
    draft_protocol(room)
    draft_protocol(room, "drugi", rodzaj="zwrot")
    add_item(room)

    # In the draft an item is corrected, but it stays in its protocol.
    room.run(
        "popraw_pozycje_protokolu",
        id=1,
        nazwa="pralka",
        opis_stanu="sprawna, rysa na obudowie, brak węża",
    )
    room.refused_sql(
        "pozycja zostaje w swoim protokole",
        "UPDATE sprawa_protokoly_pozycje SET protokol_id = 'drugi' WHERE id = 1",
    )

    add_received_document(room, "podpisany", "protokol")
    room.run("podpisz_protokol", id="protokol", dokument_id="podpisany")

    room.refused(
        "pozycje dopisuje się tylko do projektu protokołu",
        "dopisz_pozycje_protokolu",
        protokol_id="protokol",
        rodzaj="uwaga",
        nazwa="dopisek po podpisie",
    )
    room.refused(
        "pozycji podpisanego protokołu się nie zmienia",
        "popraw_pozycje_protokolu",
        id=1,
        nazwa="pralka",
        opis_stanu="nowa",
    )
    room.refused_sql(
        "podpisanego protokołu się nie zmienia",
        "UPDATE sprawa_protokoly_wpisy SET chwila = '2026-11-11T10:00:00Z' WHERE id = 'protokol'",
    )
    room.refused(
        "dokument podpisanego protokołu nie zmienia rodzaju",
        "odrzuc_dokument",
        id="podpisany",
        powod_odrzucenia="nieczytelny podpis",
    )
    room.refused_sql(
        "dokument podpisanego protokołu nie zmienia rodzaju",
        "UPDATE sprawa_dokumenty_wpisy SET rodzaj = 'inny' WHERE id = 'podpisany'",
    )


@scenario("Protokołów i pozycji się nie usuwa ani nie zastępuje przez REPLACE albo upsert")
def protocol_no_deletion_or_replacement():
    room = new_room()
    add_flat(room)
    draft_protocol(room)
    add_item(room)
    replacement = "wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE"

    for table, columns in (
        ("sprawa_protokoly_wpisy", "id, rodzaj, nieruchomosc_id, chwila, stan, dokument_id"),
        (
            "sprawa_protokoly_pozycje",
            "id, protokol_id, rodzaj, nazwa, numer, opis_stanu, zdjecie_sciezka, zdjecie_skrot",
        ),
    ):
        copy = f"INTO {table} ({columns}) SELECT {columns} FROM {table}"
        room.refused_sql(replacement, f"INSERT OR REPLACE {copy}")
        room.refused_sql(
            replacement, f"INSERT {copy} WHERE true ON CONFLICT (id) DO UPDATE SET id = excluded.id"
        )
        room.refused_sql("wierszy teczki się nie usuwa (R9)", f"DELETE FROM {table}")
