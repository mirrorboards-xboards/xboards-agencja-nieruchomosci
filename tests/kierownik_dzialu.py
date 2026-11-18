"""Scenarios of the memory a department head adds (spec/positions/kierownik_dzialu).

The catalog goes into the memory rooms of the four department heads, so its rules are checked once
here, in a room of the shared catalog. Its habits name their source in stanowisko-zrodla, so the room
imports that kind of the agent memory. Each scenario starts from an empty room and checks one rule
the database keeps: a write the rule allows and a write it refuses. The first scenario follows a
head who appoints an agent, takes its report, forwards its request, looks into its case file, hands
the case to Reception, sends out a new policy and replaces the agent.
"""

from pathlib import Path

from stanowisko import add_source, add_source_event
from room import Room

KIEROWNIK_DZIALU = Path(__file__).resolve().parents[1] / "spec" / "positions" / "kierownik_dzialu"

SCENARIOS = []

REPLACEMENT = "wiersza pamięci nie zastępuje się nowym wpisem"
DELETION = "wierszy pamięci się nie usuwa (R9)"


def scenario(title):
    def register(function):
        SCENARIOS.append((f"kierownik_dzialu: {title}", function))

        return function

    return register


def new_room():
    return Room(KIEROWNIK_DZIALU)


# Building blocks, also for the scenarios of the department heads -----------------------------------


def appoint_agent(room, agent_id, **fields):
    room.run(
        "powolaj_agenta",
        **{
            "id": agent_id,
            "stanowisko": "opiekun_oferty",
            "tozsamosc": f"tozsamosc-{agent_id}",
            "wlasciciel": "tozsamosc-posrednika",
            "teczka_odnosnik": "pokoj-teczki-1",
            "powolano": "2026-11-02T09:00:00Z",
            **fields,
        },
    )


def appoint_controller(room, agent_id, **fields):
    appoint_agent(
        room,
        agent_id,
        **{
            "stanowisko": "kontroler",
            "wlasciciel": "tozsamosc-wlasciciela-kontroli",
            "teczka_odnosnik": None,
            "klient_odnosnik": "pokoj-kartoteki/klient-nowak",
            **fields,
        },
    )


def let_in(room, **fields):
    room.run(
        "odnotuj_wpuszczenie",
        **{
            "tozsamosc": "tozsamosc-kontrolera",
            "stanowisko": "kontroler",
            "pokoj_odnosnik": "pokoj-teczki-1",
            "klient_odnosnik": "pokoj-kartoteki/klient-nowak",
            "prosil": "tozsamosc-kierownika-kontroli",
            "prosba_odnosnik": "wiadomosc-wskazanie-kontrolera",
            "wpuscil": "tozsamosc-kierownika-posrednictwa",
            "wpuszczono": "2026-11-02T10:00:00Z",
            **fields,
        },
    )


def human_approval(zatwierdzono="2026-11-02T09:30:00Z"):
    return {
        "zatwierdzil": "tozsamosc-wlasciciela",
        "podpis": "podpis-kluczem-wlasciciela",
        "zatwierdzono": zatwierdzono,
    }


def receive_report(room, report_id, agent_id, **fields):
    room.run(
        "przyjmij_raport",
        **{
            "id": report_id,
            "agent_id": agent_id,
            "otrzymano": "2026-11-03T16:00:00Z",
            "rodzaj": "biezacy",
            "etap_albo_okres": "publikacja",
            **fields,
        },
    )


def forward_request(room, decyzja_odnosnik="pokoj-teczki-1/decyzja-1", **fields):
    room.run(
        "przekaz_prosbe",
        **{
            "decyzja_odnosnik": decyzja_odnosnik,
            "raport_id": "raport-1",
            "przekazano_do": "kierownik_biura",
            "przekazano": "2026-11-03T17:00:00Z",
            **fields,
        },
    )


def look_into_case(room, **fields):
    room.run(
        "zapisz_zajrzenie",
        **{
            "teczka_odnosnik": "pokoj-teczki-1",
            "zajrzano": "2026-11-04T10:00:00Z",
            "powod": "raport_sygnalizuje_problem",
            "opis": "Raport mówi, że portal od tygodnia nie publikuje ogłoszenia",
            **fields,
        },
    )


def propose_standard(room, numer, standard="standard_ogloszenia", **fields):
    room.run(
        "zaproponuj_standard",
        **{
            "standard": standard,
            "numer": numer,
            "sciezka": f"standardy/{standard}-{numer}.typ",
            "skrot": f"sha256-{standard}-{numer}",
            "opis_zmian": "Zdjęcia w kolejności pomieszczeń",
            "zaproponowano": "2026-11-01T09:00:00Z",
            **fields,
        },
    )


def owner_signature(podpisano="2026-11-02T09:00:00Z", podpisal_jako="wlasciciel"):
    return {
        "podpisal": "tozsamosc-wlasciciela",
        "podpisal_jako": podpisal_jako,
        "podpis": "podpis-kluczem-wlasciciela",
        "podpisano": podpisano,
    }


def add_habit(room, **fields):
    room.run(
        "wpisz_zwyczaj",
        **{
            "kontrahent_rodzaj": "portal",
            "kontrahent": "Portal Ogłoszeń",
            "temat": "odświeżanie",
            "opis": "Usuwa ogłoszenie po 30 dniach bez odświeżenia",
            "zrodlo_odnosnik": "raport-1",
            "ustalono": "2026-11-03T09:00:00Z",
            "pewnosc": "prawdopodobny",
            **fields,
        },
    )


def add_case_to_list(room, **fields):
    room.run(
        "wpisz_sprawe_do_spisu",
        **{
            "teczka_odnosnik": "pokoj-teczki-1",
            "rodzaj": "oferta",
            "oznaczenie": "Oferta 2026/031, ul. Polna 7 m. 12",
            "agent": "tozsamosc-opiekun-1",
            "otwarto": "2026-11-02",
            **fields,
        },
    )


def add_mark(room, sprawa_id=1, **fields):
    room.run(
        "dodaj_znak",
        **{
            "sprawa_id": sprawa_id,
            "rodzaj": "adres_nieruchomosci",
            "wartosc": "ul. Polna 7 m. 12, Wrocław",
            "dodano": "2026-11-02T09:00:00Z",
            **fields,
        },
    )


def record_policy_copy(room, **fields):
    room.run(
        "odnotuj_rozeslanie_polisy",
        **{
            "polisa_numer": "OC/2027/001",
            "polisa_odnosnik": "pokoj-kierownika-biura/polisa-2027",
            "umowa_odnosnik": "pokoj-teczki-1/umowa-posrednictwa",
            "raport_id": "raport-1",
            "rozeslano": "2026-11-05T10:00:00Z",
            **fields,
        },
    )


def full_memory(room):
    """One row in every table of the catalog, for the checks of R9 and replacement."""
    appoint_agent(room, "opiekun-1")
    let_in(room)
    receive_report(room, "raport-1", "opiekun-1", decyzja_odnosnik="pokoj-teczki-1/decyzja-1")
    forward_request(room)
    look_into_case(room, raport_id="raport-1")
    propose_standard(room, 1)
    add_source(room, "portal", nazwa="Portal Ogłoszeń", rodzaj="portal")
    add_source_event(room, "portal", "potwierdzenie")
    add_habit(room, zrodlo_id="portal")
    add_case_to_list(room)
    add_mark(room)
    record_policy_copy(room)


# A whole day ---------------------------------------------------------------------------------------


@scenario("kierownik powołuje agenta, przekazuje prośbę, daje Recepcji spis i wymienia agenta")
def whole_day():
    room = new_room()

    appoint_agent(room, "opiekun-1", **human_approval())
    add_case_to_list(room)
    add_mark(room)
    for case in room.run("do_przekazania_recepcji"):
        room.run("odnotuj_przekazanie_sprawy", id=case["id"], przekazano="2026-11-02T09:10:00Z")
    for mark in room.run("znaki_do_przekazania_recepcji"):
        room.run("odnotuj_przekazanie_znaku", id=mark["id"], przekazano="2026-11-02T09:10:00Z")
    let_in(room)

    receive_report(
        room,
        "raport-1",
        "opiekun-1",
        blokada="Klient nie zatwierdził zdjęć",
        najblizszy_termin="2026-11-10",
        decyzja_odnosnik="pokoj-teczki-1/decyzja-1",
    )
    forward_request(room)
    look_into_case(room, raport_id="raport-1")
    record_policy_copy(room)

    room.run(
        "zglos_zdjecie_prosby",
        id=1,
        zgloszono_zdjecie="2026-11-04T12:00:00Z",
        powod_zdjecia="rozstrzygnieta",
    )

    appoint_agent(room, "opiekun-2", powolano="2026-11-06T09:00:00Z")
    room.run(
        "wymien_agenta",
        id="opiekun-1",
        wymieniono="2026-11-06T09:00:00Z",
        powod_wymiany="Opiekun przyjął polecenie z wiadomości klienta",
        nastepca_id="opiekun-2",
    )
    room.run(
        "zmien_sprawe_w_spisie",
        id=1,
        oznaczenie="Oferta 2026/031, ul. Polna 7 m. 12",
        agent="tozsamosc-opiekun-2",
    )
    assert [case["agent"] for case in room.run("do_przekazania_recepcji")] == [
        "tozsamosc-opiekun-2"
    ]

    assert [agent["id"] for agent in room.run("agenci")] == ["opiekun-2", "opiekun-1"]
    assert room.run("prosby_na_liscie") == []
    assert len(room.run("zajrzenia_od", zajrzano="2026-11-01")) == 1
    assert [row["tozsamosc"] for row in room.run("wpuszczenia_otwarte")] == ["tozsamosc-kontrolera"]
    assert len(room.run("rozeslania_polisy", polisa_numer="OC/2027/001")) == 1


# Agents ------------------------------------------------------------------------------------------


@scenario("agent sprawy ma teczkę, a Koordynator operacji i Kontroler jej nie mają")
def case_file_by_position():
    room = new_room()
    appoint_agent(room, "opiekun")
    appoint_agent(room, "koordynator", stanowisko="koordynator_operacji", teczka_odnosnik=None)
    appoint_controller(room, "kontroler")

    rule = "teczka_poza_koordynatorem_operacji_i_kontrolerem"
    room.refused(
        rule,
        "powolaj_agenta",
        id="bez-teczki",
        stanowisko="zarzadca_obiektu",
        tozsamosc="t",
        wlasciciel="w",
        powolano="2026-11-02",
    )
    room.refused(
        rule,
        "powolaj_agenta",
        id="koordynator-z-teczka",
        stanowisko="koordynator_operacji",
        tozsamosc="t",
        wlasciciel="w",
        teczka_odnosnik="pokoj-lokalu-1",
        powolano="2026-11-02",
    )


@scenario("Kontroler ma klienta, a inni agenci go nie mają")
def controller_has_client():
    room = new_room()
    appoint_controller(room, "kontroler")

    room.refused(
        "klient_tylko_u_kontrolera",
        "powolaj_agenta",
        id="kontroler-bez-klienta",
        stanowisko="kontroler",
        tozsamosc="t",
        wlasciciel="w",
        powolano="2026-11-02",
    )
    room.refused(
        "klient_tylko_u_kontrolera",
        "powolaj_agenta",
        id="opiekun-z-klientem",
        stanowisko="opiekun_oferty",
        tozsamosc="t",
        wlasciciel="w",
        teczka_odnosnik="pokoj-teczki-1",
        klient_odnosnik="pokoj-kartoteki/klient-nowak",
        powolano="2026-11-02",
    )
    assert [
        row["id"]
        for row in room.run("agenci_klienta", klient_odnosnik="pokoj-kartoteki/klient-nowak")
    ] == ["kontroler"]


@scenario("agent powstaje pracujący, a jego powołania się nie zmienia")
def agent_starts_working():
    room = new_room()
    appoint_agent(room, "opiekun")
    room.run("uspij_agenta", id="opiekun")
    room.run("obudz_agenta", id="opiekun")

    room.refused_sql(
        "agent powstaje jako pracujący",
        """
        INSERT INTO kierownik_dzialu_agenci_wpisy (id, stanowisko, tozsamosc, wlasciciel, teczka_odnosnik,
          powolano, stan)
        VALUES ('spiacy', 'opiekun_oferty', 't', 'w', 'pokoj-teczki-1', '2026-11-02', 'spi')
        """,
    )
    for column, value in [
        ("stanowisko", "'opiekun_najmu'"),
        ("teczka_odnosnik", "'pokoj-teczki-2'"),
        ("tozsamosc", "'inna'"),
    ]:
        room.refused_sql(
            "stanowiska, tożsamości, teczki, klienta, daty powołania ani zatwierdzenia agenta się nie zmienia",
            f"UPDATE kierownik_dzialu_agenci_wpisy SET {column} = {value} WHERE id = 'opiekun'",
        )


@scenario("wymiana wymaga powodu, daty i następcy i jest ostateczna")
def replacement_needs_reason():
    room = new_room()
    appoint_agent(room, "opiekun-1")
    appoint_agent(room, "opiekun-2")

    room.refused(
        "wymiana_z_powodem_data_i_nastepca",
        "wymien_agenta",
        id="opiekun-1",
        wymieniono="2026-11-06",
        nastepca_id="opiekun-2",
    )
    room.run(
        "wymien_agenta",
        id="opiekun-1",
        wymieniono="2026-11-06",
        powod_wymiany="Zawiódł",
        nastepca_id="opiekun-2",
    )
    room.refused("wymienionego agenta się nie zmienia", "obudz_agenta", id="opiekun-1")


@scenario("następca ma to samo stanowisko i tę samą teczkę, a Kontroler tego samego klienta")
def successor_takes_same_work():
    room = new_room()
    appoint_agent(room, "opiekun-1")
    appoint_agent(room, "inna-teczka", teczka_odnosnik="pokoj-teczki-2")
    appoint_agent(room, "inne-stanowisko", stanowisko="opiekun_poszukujacego")
    appoint_controller(room, "kontroler-1")
    appoint_controller(room, "kontroler-innego", klient_odnosnik="pokoj-kartoteki/klient-kowalski")
    appoint_controller(room, "kontroler-2")

    rule = "następca ma to samo stanowisko i tę samą teczkę"
    for successor in ["inna-teczka", "inne-stanowisko", "opiekun-1"]:
        room.refused(
            rule,
            "wymien_agenta",
            id="opiekun-1",
            wymieniono="2026-11-06",
            powod_wymiany="Zawiódł",
            nastepca_id=successor,
        )
    room.refused(
        rule,
        "wymien_agenta",
        id="kontroler-1",
        wymieniono="2026-11-06",
        powod_wymiany="Zawiódł",
        nastepca_id="kontroler-innego",
    )
    room.run(
        "wymien_agenta",
        id="kontroler-1",
        wymieniono="2026-11-06",
        powod_wymiany="Zawiódł",
        nastepca_id="kontroler-2",
    )


@scenario("zatwierdzenie powołania ma komplet kolumn i nie zmienia się po wpisaniu")
def appointment_approval_complete():
    room = new_room()
    appoint_agent(room, "opiekun-1")
    room.refused(
        "zatwierdzenie_w_komplecie",
        "zatwierdz_powolanie",
        id="opiekun-1",
        zatwierdzil="tozsamosc-wlasciciela",
        zatwierdzono="2026-11-02",
    )
    room.run("zatwierdz_powolanie", id="opiekun-1", **human_approval())
    room.refused(
        "stanowiska, tożsamości, teczki, klienta, daty powołania ani zatwierdzenia agenta się nie zmienia",
        "zatwierdz_powolanie",
        id="opiekun-1",
        **human_approval("2026-11-03"),
    )


# Let-ins -----------------------------------------------------------------------------------------


@scenario("jedno otwarte wpuszczenie tożsamości do pokoju; nowe po usunięciu")
def one_open_let_in_per_room():
    room = new_room()
    let_in(room)
    let_in(room, pokoj_odnosnik="pokoj-teczki-2")

    rule = "ta tożsamość ma już otwarte wpuszczenie do tego pokoju"
    room.refused(rule, "odnotuj_wpuszczenie", **let_in_fields())
    room.run("odnotuj_usuniecie_z_pokoju", id=1, usunieto="2026-11-05T10:00:00Z")
    let_in(room, wpuszczono="2026-11-06T10:00:00Z")
    assert len(room.run("wpuszczenia_pokoju", pokoj_odnosnik="pokoj-teczki-1")) == 2


def let_in_fields(**fields):
    return {
        "tozsamosc": "tozsamosc-kontrolera",
        "stanowisko": "kontroler",
        "pokoj_odnosnik": "pokoj-teczki-1",
        "klient_odnosnik": "pokoj-kartoteki/klient-nowak",
        "wpuscil": "tozsamosc-kierownika-posrednictwa",
        "wpuszczono": "2026-11-03T10:00:00Z",
        **fields,
    }


@scenario("wpuszczenie: Kontroler z klientem, prośba z proszącym, usunięcie po wpuszczeniu")
def let_in_fields_go_together():
    room = new_room()
    let_in(room)

    room.refused(
        "klient_tylko_u_kontrolera",
        "odnotuj_wpuszczenie",
        **let_in_fields(pokoj_odnosnik="pokoj-teczki-2", klient_odnosnik=None),
    )
    room.refused(
        "prosba_razem_z_proszacym",
        "odnotuj_wpuszczenie",
        **let_in_fields(pokoj_odnosnik="pokoj-teczki-2", prosil="tozsamosc-kierownika-kontroli"),
    )
    room.refused(
        "usuniecie_nie_przed_wpuszczeniem",
        "odnotuj_usuniecie_z_pokoju",
        id=1,
        usunieto="2026-11-01T00:00:00Z",
    )
    room.refused(
        "zatwierdzenie_w_komplecie",
        "zatwierdz_wpuszczenie",
        id=1,
        zatwierdzil="tozsamosc-wlasciciela",
        zatwierdzono="2026-11-02",
    )


@scenario("wpuszczenie własnego agenta ma jego tożsamość, stanowisko i klienta")
def own_agent_let_in_matches():
    room = new_room()
    appoint_agent(room, "koordynator", stanowisko="koordynator_transakcji")
    room.run(
        "odnotuj_wpuszczenie",
        tozsamosc="tozsamosc-koordynator",
        stanowisko="koordynator_transakcji",
        agent_id="koordynator",
        pokoj_odnosnik="pokoj-wymiany-1",
        wpuscil="tozsamosc-kierownika-transakcji",
        wpuszczono="2026-11-03T10:00:00Z",
    )

    room.refused(
        "wpuszczenie własnego agenta ma jego tożsamość, stanowisko i klienta",
        "odnotuj_wpuszczenie",
        tozsamosc="tozsamosc-kogos-innego",
        stanowisko="koordynator_transakcji",
        agent_id="koordynator",
        pokoj_odnosnik="pokoj-wymiany-2",
        wpuscil="tozsamosc-kierownika-transakcji",
        wpuszczono="2026-11-03T10:00:00Z",
    )


@scenario("wpuszczenie powstaje otwarte, a usunięcie i zatwierdzenie odnotowuje się raz")
def let_in_is_a_trace():
    room = new_room()
    let_in(room)
    room.run("zatwierdz_wpuszczenie", id=1, **human_approval())
    room.run("odnotuj_usuniecie_z_pokoju", id=1, usunieto="2026-11-05T10:00:00Z")

    rule = "wpuszczenia się nie zmienia"
    room.refused(rule, "odnotuj_usuniecie_z_pokoju", id=1, usunieto="2026-11-06T10:00:00Z")
    room.refused(rule, "zatwierdz_wpuszczenie", id=1, **human_approval("2026-11-03"))
    room.refused_sql(rule, "UPDATE kierownik_dzialu_agenci_wpuszczenia SET wpuscil = 'ktos-inny'")
    room.refused_sql(
        "wpuszczenie powstaje otwarte",
        """
        INSERT INTO kierownik_dzialu_agenci_wpuszczenia (tozsamosc, stanowisko, pokoj_odnosnik, wpuscil,
          wpuszczono, usunieto)
        VALUES ('t', 'opiekun_najmu', 'pokoj-teczki-3', 'k', '2026-11-03', '2026-11-04')
        """,
    )


# Reports and requests ----------------------------------------------------------------------------


@scenario("raporty tylko się dopisuje")
def reports_are_appended():
    room = new_room()
    appoint_agent(room, "opiekun-1")
    receive_report(room, "raport-1", "opiekun-1")

    room.refused_sql(
        "raporty tylko się dopisuje",
        "UPDATE kierownik_dzialu_raporty_wpisy SET etap_albo_okres = 'koniec'",
    )
    room.refused(
        "FOREIGN KEY constraint failed",
        "przyjmij_raport",
        id="raport-obcego",
        agent_id="nieznany",
        otrzymano="2026-11-03",
        rodzaj="biezacy",
        etap_albo_okres="publikacja",
    )


@scenario("prośby nie przekazuje się drugi raz, dopóki nie zgłoszono jej zdjęcia z powodem")
def request_forwarded_once():
    room = new_room()
    appoint_agent(room, "opiekun-1")
    receive_report(room, "raport-1", "opiekun-1")
    forward_request(room)

    room.refused(
        "tej prośby nie przekazuje się drugi raz",
        "przekaz_prosbe",
        decyzja_odnosnik="pokoj-teczki-1/decyzja-1",
        raport_id="raport-1",
        przekazano_do="kierownik_biura",
        przekazano="2026-11-04T09:00:00Z",
    )
    room.refused(
        "zdjecie_z_powodem", "zglos_zdjecie_prosby", id=1, zgloszono_zdjecie="2026-11-04T09:00:00Z"
    )
    room.refused(
        "zdjecie_nie_przed_przekazaniem",
        "zglos_zdjecie_prosby",
        id=1,
        zgloszono_zdjecie="2026-11-01T09:00:00Z",
        powod_zdjecia="zbedna",
    )
    room.run(
        "zglos_zdjecie_prosby",
        id=1,
        zgloszono_zdjecie="2026-11-04T09:00:00Z",
        powod_zdjecia="zbedna",
    )
    forward_request(room, przekazano_do="wlasciciel", przekazano="2026-11-05T09:00:00Z")
    assert [row["przekazano_do"] for row in room.run("prosby_na_liscie")] == ["wlasciciel"]


@scenario("przekazana prośba powstaje na liście, a po zdjęciu się jej nie zmienia")
def request_starts_on_list():
    room = new_room()
    appoint_agent(room, "opiekun-1")
    receive_report(room, "raport-1", "opiekun-1")
    forward_request(room)
    room.run(
        "zglos_zdjecie_prosby",
        id=1,
        zgloszono_zdjecie="2026-11-04T09:00:00Z",
        powod_zdjecia="rozstrzygnieta",
    )

    room.refused(
        "przekazanej prośby się nie zmienia",
        "zglos_zdjecie_prosby",
        id=1,
        zgloszono_zdjecie="2026-11-05T09:00:00Z",
        powod_zdjecia="zbedna",
    )
    room.refused_sql(
        "przekazana prośba powstaje na liście",
        """
        INSERT INTO kierownik_dzialu_raporty_prosby (decyzja_odnosnik, raport_id, przekazano_do, przekazano,
          zgloszono_zdjecie, powod_zdjecia)
        VALUES ('pokoj-teczki-1/decyzja-2', 'raport-1', 'kierownik_biura', '2026-11-03',
          '2026-11-04', 'zbedna')
        """,
    )


# Look-ins ----------------------------------------------------------------------------------------


@scenario("zajrzenie ma jeden z czterech powodów i opis, a potem się go nie zmienia")
def look_in_is_fixed():
    room = new_room()
    look_into_case(room)

    room.refused(
        "CHECK constraint failed: powod IN",
        "zapisz_zajrzenie",
        teczka_odnosnik="pokoj-teczki-1",
        zajrzano="2026-11-04",
        powod="ciekawosc",
        opis="Bez powodu",
    )
    room.refused(
        "NOT NULL constraint failed: kierownik_dzialu_zajrzenia_wpisy.opis",
        "zapisz_zajrzenie",
        teczka_odnosnik="pokoj-teczki-1",
        zajrzano="2026-11-04",
        powod="sprawdzenie_jakosci",
    )
    room.refused_sql(
        "zajrzenia się nie zmienia",
        "UPDATE kierownik_dzialu_zajrzenia_wpisy SET opis = 'Inny powód'",
    )


# Standards ---------------------------------------------------------------------------------------


@scenario("wersja standardu powstaje jako propozycja, a numer jest jeden w standardzie")
def standard_starts_as_proposal():
    room = new_room()
    propose_standard(room, 1)
    propose_standard(room, 1, standard="wzor_umowy_posrednictwa")

    room.refused(
        REPLACEMENT,
        "zaproponuj_standard",
        standard="standard_ogloszenia",
        numer=1,
        sciezka="s",
        skrot="k",
        opis_zmian="o",
        zaproponowano="2026-11-01",
    )
    room.refused_sql(
        "wersja standardu powstaje jako propozycja",
        """
        INSERT INTO kierownik_dzialu_standardy_wersje (standard, numer, sciezka, skrot, opis_zmian,
          zaproponowano, rozstrzygniecie, podpisal, podpisal_jako, podpis, podpisano)
        VALUES ('standard_ogloszenia', 2, 's', 'k', 'o', '2026-11-01', 'zatwierdzona', 'w',
          'wlasciciel', 'p', '2026-11-02')
        """,
    )


@scenario("rozstrzygnięcie standardu wymaga kompletu podpisu i jest ostateczne")
def standard_decision_signed_and_final():
    room = new_room()
    propose_standard(room, 1)

    room.refused(
        "podpis_w_komplecie",
        "rozstrzygnij_standard",
        id=1,
        rozstrzygniecie="zatwierdzona",
        podpisal="tozsamosc-wlasciciela",
        podpisal_jako="wlasciciel",
        podpisano="2026-11-02",
    )
    room.refused(
        "rozstrzygniecie_nie_przed_propozycja",
        "rozstrzygnij_standard",
        id=1,
        rozstrzygniecie="zatwierdzona",
        **owner_signature("2026-10-01"),
    )
    room.refused_sql(
        "rozstrzygniętej wersji standardu się nie zmienia",
        "UPDATE kierownik_dzialu_standardy_wersje SET skrot = 'inny' WHERE id = 1",
    )
    room.run(
        "rozstrzygnij_standard",
        id=1,
        rozstrzygniecie="zatwierdzona",
        **owner_signature(podpisal_jako="wlasciciel_biura"),
    )
    room.refused(
        "rozstrzygniętej wersji standardu się nie zmienia",
        "rozstrzygnij_standard",
        id=1,
        rozstrzygniecie="odrzucona",
        **owner_signature(),
    )


@scenario("obowiązuje najnowsza zatwierdzona wersja każdego standardu")
def current_standards():
    room = new_room()
    propose_standard(room, 1)
    propose_standard(room, 2)
    propose_standard(room, 3)
    propose_standard(room, 1, standard="parametry_programu")
    room.run("rozstrzygnij_standard", id=1, rozstrzygniecie="zatwierdzona", **owner_signature())
    room.run("rozstrzygnij_standard", id=2, rozstrzygniecie="zatwierdzona", **owner_signature())
    room.run("rozstrzygnij_standard", id=3, rozstrzygniecie="odrzucona", **owner_signature())
    room.run("rozstrzygnij_standard", id=4, rozstrzygniecie="zatwierdzona", **owner_signature())

    current = {row["standard"]: row["numer"] for row in room.run("obowiazujace_standardy")}
    assert current == {"parametry_programu": 1, "standard_ogloszenia": 2}
    assert [
        row["numer"] for row in room.run("wersje_standardu", standard="standard_ogloszenia")
    ] == [
        3,
        2,
        1,
    ]


# Habits ------------------------------------------------------------------------------------------


@scenario("zwyczaj ma źródło i powstaje niepotwierdzony, a pewny wymaga potwierdzenia")
def habit_needs_source_and_confirmation():
    room = new_room()
    add_habit(room)

    room.refused(
        "zwyczaj_ze_zrodlem",
        "wpisz_zwyczaj",
        kontrahent_rodzaj="bank",
        kontrahent="Bank",
        temat="wypłata",
        opis="Wypłaca po wypisie aktu",
        ustalono="2026-11-03",
        pewnosc="niepewny",
    )
    room.refused(
        "pewny_z_potwierdzeniem",
        "wpisz_zwyczaj",
        kontrahent_rodzaj="bank",
        kontrahent="Bank",
        temat="wypłata",
        opis="Wypłaca po wypisie aktu",
        zrodlo_odnosnik="raport-2",
        ustalono="2026-11-03",
        pewnosc="pewny",
    )
    room.refused_sql(
        "zwyczaj powstaje niepotwierdzony i niewykreślony",
        """
        INSERT INTO kierownik_dzialu_zwyczaje_wpisy (kontrahent_rodzaj, kontrahent, temat, opis, zrodlo_odnosnik,
          ustalono, pewnosc, potwierdzenie, potwierdzenie_odnosnik, potwierdzono)
        VALUES ('portal', 'Portal', 't', 'o', 'r', '2026-11-03', 'pewny', 'regulamin', 'regulamin-portalu',
          '2026-11-03')
        """,
    )


@scenario("zwyczaj potwierdza regulamin albo kontrahent z odnośnikiem, a człowiek biura podpisem")
def habit_confirmation_kinds():
    room = new_room()
    add_habit(room)
    add_habit(room, kontrahent_rodzaj="bank", kontrahent="Bank Przykładowy", temat="wypłata")

    room.refused(
        "potwierdzenie_regulaminem_albo_kontrahenta_wskazane",
        "potwierdz_zwyczaj",
        id=1,
        potwierdzenie="regulamin",
        potwierdzono="2026-11-04",
    )
    room.refused(
        "potwierdzenie_czlowieka_biura_z_podpisem",
        "potwierdz_zwyczaj",
        id=2,
        potwierdzenie="czlowiek_biura",
        potwierdzil="tozsamosc-pośrednika",
        potwierdzono="2026-11-04",
    )
    room.refused(
        "potwierdzenie_czlowieka_biura_z_podpisem",
        "potwierdz_zwyczaj",
        id=1,
        potwierdzenie="regulamin",
        potwierdzenie_odnosnik="regulamin-portalu",
        potwierdzil="tozsamosc-pośrednika",
        podpis="podpis",
        potwierdzono="2026-11-04",
    )
    room.refused(
        "potwierdzenie_z_data",
        "potwierdz_zwyczaj",
        id=1,
        potwierdzenie="regulamin",
        potwierdzenie_odnosnik="regulamin-portalu",
    )
    room.run(
        "potwierdz_zwyczaj",
        id=1,
        potwierdzenie="regulamin",
        potwierdzenie_odnosnik="regulamin-portalu",
        potwierdzono="2026-11-04",
    )
    room.run(
        "potwierdz_zwyczaj",
        id=2,
        potwierdzenie="czlowiek_biura",
        potwierdzil="tozsamosc-wlasciciela",
        podpis="podpis-kluczem-wlasciciela",
        potwierdzono="2026-11-04",
    )
    assert room.run("zwyczaje_kontrahenta", kontrahent="Portal Ogłoszeń")[0]["pewnosc"] == "pewny"


@scenario("zakwestionowany zwyczaj traci potwierdzenie, a wykreślonego się nie zmienia")
def habit_questioned_and_struck():
    room = new_room()
    add_habit(room)
    room.run(
        "potwierdz_zwyczaj",
        id=1,
        potwierdzenie="regulamin",
        potwierdzenie_odnosnik="regulamin-portalu",
        potwierdzono="2026-11-04",
    )
    room.run("zakwestionuj_zwyczaj", id=1, pewnosc="niepewny")
    assert (
        room.value("SELECT potwierdzenie FROM kierownik_dzialu_zwyczaje_wpisy WHERE id = 1") is None
    )

    room.refused_sql(
        "kontrahenta, treści, źródła ani daty zwyczaju się nie zmienia",
        "UPDATE kierownik_dzialu_zwyczaje_wpisy SET opis = 'Usuwa po 60 dniach'",
    )
    room.refused("wykreslenie_z_powodem", "wykresl_zwyczaj", id=1, wykreslono="2026-11-05")
    room.run(
        "wykresl_zwyczaj",
        id=1,
        wykreslono="2026-11-05",
        powod_wykreslenia="Portal zmienił regulamin",
    )
    room.refused(
        "wykreślonego zwyczaju się nie zmienia",
        "zakwestionuj_zwyczaj",
        id=1,
        pewnosc="prawdopodobny",
    )
    assert room.run("zwyczaje_kontrahenta", kontrahent="Portal Ogłoszeń") == []
    assert len(room.run("zwyczaje")) == 1


@scenario("zwyczaje od źródła, które podało nieprawdę, wracają do ponownej oceny")
def habits_from_one_source():
    room = new_room()
    add_source(room, "doradca", nazwa="Doradca z banku", rodzaj="osoba")
    add_source(room, "regulamin", nazwa="Regulamin portalu", rodzaj="dokument")
    add_habit(
        room, zrodlo_id="doradca", zrodlo_odnosnik=None, kontrahent_rodzaj="bank", kontrahent="Bank"
    )
    add_habit(room, zrodlo_id="regulamin")
    add_habit(
        room, zrodlo_id="doradca", kontrahent_rodzaj="bank", kontrahent="Bank", temat="termin"
    )
    room.run("wykresl_zwyczaj", id=3, wykreslono="2026-11-04", powod_wykreslenia="Nieaktualny")

    assert [row["id"] for row in room.run("zwyczaje_od_zrodla", zrodlo_id="doradca")] == [1]
    room.refused(
        "FOREIGN KEY constraint failed",
        "wpisz_zwyczaj",
        kontrahent_rodzaj="bank",
        kontrahent="Bank",
        temat="t",
        opis="o",
        zrodlo_id="nieznane-zrodlo",
        ustalono="2026-11-03",
        pewnosc="niepewny",
    )


# Reception list ----------------------------------------------------------------------------------


@scenario("sprawa jest w spisie raz, a kancelaria na XNotariuszu tylko przy transakcji")
def case_in_list_once():
    room = new_room()
    add_case_to_list(room)
    add_case_to_list(
        room,
        teczka_odnosnik="pokoj-transakcji-1",
        rodzaj="transakcja",
        oznaczenie="Transakcja 2026/007",
        kancelaria_xnotariusz=1,
    )

    room.refused(REPLACEMENT, "wpisz_sprawe_do_spisu", **case_fields())
    room.refused(
        "kancelaria_tylko_przy_transakcji",
        "wpisz_sprawe_do_spisu",
        **case_fields(teczka_odnosnik="pokoj-teczki-2", kancelaria_xnotariusz=0),
    )
    room.refused(
        "kancelaria_tylko_przy_transakcji",
        "wpisz_sprawe_do_spisu",
        **case_fields(teczka_odnosnik="pokoj-transakcji-2", rodzaj="transakcja"),
    )


def case_fields(**fields):
    return {
        "teczka_odnosnik": "pokoj-teczki-1",
        "rodzaj": "oferta",
        "oznaczenie": "Oferta",
        "agent": "tozsamosc-opiekuna",
        "otwarto": "2026-11-02",
        **fields,
    }


@scenario("sprawa powstaje otwarta, zmiana czeka na przekazanie, a zamkniętej się nie zmienia")
def case_changes_wait_for_reception():
    room = new_room()
    room.refused(
        "otwarto", "wpisz_sprawe_do_spisu", **case_fields(teczka_odnosnik="bez-daty", otwarto="")
    )
    add_case_to_list(room)
    room.run("odnotuj_przekazanie_sprawy", id=1, przekazano="2026-11-02T10:00:00Z")
    assert room.run("do_przekazania_recepcji") == []

    room.run(
        "zmien_sprawe_w_spisie", id=1, oznaczenie="Oferta 2026/031", agent="tozsamosc-nastepcy"
    )
    assert len(room.run("do_przekazania_recepcji")) == 1
    room.run("odnotuj_przekazanie_sprawy", id=1, przekazano="2026-11-03T10:00:00Z")

    room.refused(
        "zamkniecie_przekazane_po_zamknieciu",
        "odnotuj_przekazanie_zamkniecia",
        id=1,
        zamkniecie_przekazano="2026-11-04",
    )
    room.run("zamknij_sprawe_w_spisie", id=1, zamknieto="2026-11-20")
    assert len(room.run("do_przekazania_recepcji")) == 1
    room.run("odnotuj_przekazanie_zamkniecia", id=1, zamkniecie_przekazano="2026-11-20")

    rule = "teczki ani rodzaju sprawy w spisie się nie zmienia"
    room.refused(rule, "zmien_sprawe_w_spisie", id=1, oznaczenie="Inna", agent="inny")
    room.refused(rule, "odnotuj_przekazanie_zamkniecia", id=1, zamkniecie_przekazano="2026-11-21")
    room.refused_sql(rule, "UPDATE kierownik_dzialu_spis_sprawy SET rodzaj = 'najem'")
    room.refused_sql(
        "sprawa powstaje w spisie otwarta",
        """
        INSERT INTO kierownik_dzialu_spis_sprawy (teczka_odnosnik, rodzaj, oznaczenie, agent, zamknieto)
        VALUES ('pokoj-teczki-9', 'oferta', 'Oferta', 'agent', '2026-11-02')
        """,
    )
    assert room.run("sprawy_w_spisie")[0]["zamknieto"] == "2026-11-20"


@scenario("znaku się nie usuwa ani nie zmienia, tylko wycofuje, a wycofanie przekazuje się raz")
def marks_are_withdrawn():
    room = new_room()
    add_case_to_list(room)
    add_mark(room)
    add_mark(room, rodzaj="adres_poczty_klienta", wartosc="nowak@example.com")

    room.refused(
        "wycofanie_przekazane_po_wycofaniu_przekazanego_znaku",
        "odnotuj_przekazanie_wycofania",
        id=1,
        wycofanie_przekazano="2026-11-03",
    )
    room.run("odnotuj_przekazanie_znaku", id=1, przekazano="2026-11-02T10:00:00Z")
    room.run("wycofaj_znak", id=1, wycofano="2026-11-05T10:00:00Z")
    room.run("wycofaj_znak", id=2, wycofano="2026-11-05T10:00:00Z")
    assert [row["id"] for row in room.run("znaki_do_przekazania_recepcji")] == [1]
    room.run("odnotuj_przekazanie_wycofania", id=1, wycofanie_przekazano="2026-11-05T11:00:00Z")
    assert room.run("znaki_do_przekazania_recepcji") == []

    rule = "znaku się nie zmienia"
    room.refused(rule, "wycofaj_znak", id=1, wycofano="2026-11-06")
    room.refused(rule, "odnotuj_przekazanie_znaku", id=1, przekazano="2026-11-06")
    room.refused_sql(
        rule, "UPDATE kierownik_dzialu_spis_znaki SET wartosc = 'ul. Leśna 1' WHERE id = 2"
    )
    room.refused_sql(
        "znaku się nie usuwa, tylko wycofuje",
        "DELETE FROM kierownik_dzialu_spis_znaki WHERE id = 2",
    )
    add_mark(room, rodzaj="oznaczenie_transakcji", wartosc="Transakcja 2026/007")
    room.refused("wycofanie_nie_przed_dodaniem", "wycofaj_znak", id=3, wycofano="2026-11-01")
    room.refused(
        "przekazanie_nie_przed_dodaniem", "odnotuj_przekazanie_znaku", id=3, przekazano="2026-11-01"
    )
    room.refused_sql(
        "znak powstaje niewycofany i jeszcze nieprzekazany",
        """
        INSERT INTO kierownik_dzialu_spis_znaki (sprawa_id, rodzaj, wartosc, dodano, przekazano)
        VALUES (1, 'oznaczenie_transakcji', 'T/1', '2026-11-02', '2026-11-02')
        """,
    )
    assert len(room.run("znaki_sprawy", sprawa_id=1)) == 3


# Policies ----------------------------------------------------------------------------------------


@scenario("umowa dostaje kopię jednej polisy raz, a rozesłania się nie zmienia")
def policy_copy_once_per_contract():
    room = new_room()
    appoint_agent(room, "opiekun-1")
    receive_report(room, "raport-1", "opiekun-1")
    record_policy_copy(room)
    record_policy_copy(room, umowa_odnosnik="pokoj-teczki-2/umowa-posrednictwa")
    record_policy_copy(room, polisa_numer="OC/2028/001")

    room.refused(
        REPLACEMENT,
        "odnotuj_rozeslanie_polisy",
        polisa_numer="OC/2027/001",
        umowa_odnosnik="pokoj-teczki-1/umowa-posrednictwa",
        raport_id="raport-1",
        rozeslano="2026-11-06",
    )
    room.refused_sql(
        "rozesłania polisy się nie zmienia",
        "UPDATE kierownik_dzialu_polisy_rozeslania SET rozeslano = '2026-11-07'",
    )
    room.refused(
        "NOT NULL constraint failed: kierownik_dzialu_polisy_rozeslania.raport_id",
        "odnotuj_rozeslanie_polisy",
        polisa_numer="OC/2027/001",
        umowa_odnosnik="pokoj-teczki-3/umowa",
        rozeslano="2026-11-06",
    )
    assert len(room.run("rozeslania_polisy", polisa_numer="OC/2027/001")) == 2


# R9 and replacement --------------------------------------------------------------------------------


def tables_of(room):
    return [
        row["name"]
        for row in room.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'kierownik\\_dzialu\\_%' ESCAPE '\\'"
        )
    ]


def first_row_copy(room, table):
    """SQL that inserts again the first row of the table, as a careless upsert would."""
    columns = ", ".join(row["name"] for row in room.execute(f"PRAGMA table_info({table})"))

    return (
        f"INTO {table} ({columns}) SELECT {columns} FROM {table} "
        f"WHERE rowid = (SELECT min(rowid) FROM {table})"
    )


@scenario("R9: wierszy pamięci się nie usuwa ani nie zastępuje przez REPLACE albo upsert")
def no_deletion_or_replacement():
    room = new_room()
    full_memory(room)

    tables = tables_of(room)
    assert len(tables) == 10, tables

    for table in tables:
        assert room.value(f"SELECT count(*) FROM {table}") > 0, f"{table} has no row"

        room.refused_sql(REPLACEMENT, f"INSERT OR REPLACE {first_row_copy(room, table)}")
        room.refused_sql(
            REPLACEMENT,
            f"INSERT {first_row_copy(room, table)} ON CONFLICT (id) DO UPDATE SET id = excluded.id",
        )
        room.refused_sql(
            "wierszy pamięci się nie usuwa (R9)",
            f"DELETE FROM {table} WHERE rowid = (SELECT min(rowid) FROM {table})",
        )


@scenario(
    "REPLACE nie usuwa otwartego wpuszczenia, prośby, sprawy ani rozesłania przez klucz UNIQUE"
)
def no_replacement_through_unique_keys():
    room = new_room()
    full_memory(room)

    room.refused_sql(
        "ta tożsamość ma już otwarte wpuszczenie do tego pokoju",
        "INSERT OR REPLACE INTO kierownik_dzialu_agenci_wpuszczenia (id, tozsamosc, stanowisko, pokoj_odnosnik, "
        "klient_odnosnik, wpuscil, wpuszczono) SELECT 99, tozsamosc, stanowisko, pokoj_odnosnik, "
        "klient_odnosnik, wpuscil, wpuszczono FROM kierownik_dzialu_agenci_wpuszczenia",
    )
    room.refused_sql(
        "tej prośby nie przekazuje się drugi raz",
        "INSERT OR REPLACE INTO kierownik_dzialu_raporty_prosby (id, decyzja_odnosnik, raport_id, przekazano_do, "
        "przekazano) SELECT 99, decyzja_odnosnik, raport_id, przekazano_do, przekazano "
        "FROM kierownik_dzialu_raporty_prosby",
    )
    for table, key in [
        ("kierownik_dzialu_standardy_wersje", "standard, numer"),
        ("kierownik_dzialu_spis_sprawy", "teczka_odnosnik"),
        ("kierownik_dzialu_polisy_rozeslania", "polisa_numer, umowa_odnosnik"),
    ]:
        columns = [row["name"] for row in room.execute(f"PRAGMA table_info({table})")]
        selected = ", ".join("99" if name == "id" else name for name in columns)
        room.refused_sql(
            REPLACEMENT,
            f"INSERT OR REPLACE INTO {table} ({', '.join(columns)}) SELECT {selected} FROM {table}",
        )
        room.refused_sql(
            REPLACEMENT,
            # `WHERE true` keeps SQLite from reading ON CONFLICT as a join constraint.
            f"INSERT INTO {table} ({', '.join(columns)}) SELECT {selected} FROM {table} "
            f"WHERE true ON CONFLICT ({key}) DO UPDATE SET id = excluded.id",
        )
