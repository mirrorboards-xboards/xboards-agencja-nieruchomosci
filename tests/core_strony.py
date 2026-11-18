"""Scenarios of the core kinds between the office and the parties (spec/cases/sprawa):
powers of attorney (sprawa-umocowania), correspondence (sprawa-korespondencja), consents of
clients (sprawa-zgody) and contracts of clients with the office (sprawa-umowy).

Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
allows and a write it refuses.
"""

from core import (
    WITHDRAWAL_ENDS,
    add_company,
    add_consent,
    add_inflow,
    add_person,
    add_power_for_office,
    add_received_document,
    approve,
    approved_decision,
    ask_for_decision,
    check,
    conclude_contract,
    new_room,
    power_version_of,
    prepare_contract,
    prepare_letter,
    send,
)

SCENARIOS = []


def scenario(title):
    def register(function):
        SCENARIOS.append((title, function))

        return function

    return register


def power_fields(**fields):
    return {
        "id": "pelnomocnictwo",
        "rodzaj": "pelnomocnictwo",
        "mocodawca_uczestnik_id": "klientka",
        "umocowane_biuro": 0,
        "zakres": "rodzajowe",
        "forma": "pisemna",
        **fields,
    }


# Powers of attorney ------------------------------------------------------------------------------


@scenario("Umocowanie: umocowany jest osobą albo biurem, nie oba naraz")
def power_holder():
    room = new_room()
    add_person(room, "klientka")
    add_person(room, "syn", imiona="Piotr")

    room.refused("umocowany_osoba_albo_biuro", "wpisz_umocowanie", **power_fields())
    room.refused(
        "umocowany_osoba_albo_biuro",
        "wpisz_umocowanie",
        **power_fields(umocowany_uczestnik_id="syn", umocowane_biuro=1),
    )
    room.refused(
        "nie_umocowuje_siebie",
        "wpisz_umocowanie",
        **power_fields(umocowany_uczestnik_id="klientka"),
    )

    room.run("wpisz_umocowanie", **power_fields(umocowany_uczestnik_id="syn"))
    room.run("wpisz_umocowanie", **power_fields(id="dla-biura", umocowane_biuro=1))


@scenario("Umocowanie: reprezentację ma tylko podmiot")
def representation_of_entity():
    room = new_room()
    add_person(room, "klientka")
    add_person(room, "prezes", imiona="Piotr")
    add_company(room, "spolka")
    add_received_document(room, "odpis-krs", "odpis_rejestru")

    room.refused(
        "reprezentację ma tylko podmiot",
        "wpisz_umocowanie",
        **power_fields(rodzaj="reprezentacja", umocowany_uczestnik_id="prezes"),
    )
    room.run(
        "wpisz_umocowanie",
        **power_fields(
            id="reprezentacja",
            rodzaj="reprezentacja",
            mocodawca_uczestnik_id="spolka",
            umocowany_uczestnik_id="prezes",
            zakres="ogolne",
            forma="z_rejestru",
            sposob_reprezentacji="prezes zarządu samodzielnie",
            dokument_id="odpis-krs",
        ),
    )
    room.refused(
        "reprezentację ma tylko podmiot",
        "zmien_dane_umocowania",
        **power_fields(
            id="reprezentacja",
            rodzaj="reprezentacja",
            mocodawca_uczestnik_id="klientka",
            umocowany_uczestnik_id="prezes",
        ),
    )


@scenario("Umocowanie: wygaśnięcie ma powód i źródło wiadomości")
def power_expiry():
    room = new_room()
    add_person(room, "klientka")
    add_power_for_office(room, "pelnomocnictwo", "klientka")

    room.refused_sql(
        "wygasniecie_ma_powod_i_zrodlo",
        "UPDATE sprawa_umocowania_wpisy SET wygaslo = '2026-11-10'",
    )
    room.refused(
        "wygasniecie_ma_powod_i_zrodlo",
        "odnotuj_wygasniecie_umocowania",
        id="pelnomocnictwo",
        wygaslo="2026-11-10",
        wygasniecie_powod="odwolanie",
    )
    room.run(
        "odnotuj_wygasniecie_umocowania",
        id="pelnomocnictwo",
        wygaslo="2026-11-10",
        wygasniecie_powod="odwolanie",
        wygasniecie_zrodlo="wiadomość klientki z 10 listopada",
    )


@scenario("Umocowanie: każda zmiana i nowa czynność podnosi wersję; czynności się nie zmienia (D6)")
def power_version():
    room = new_room()
    add_person(room, "klientka")
    add_power_for_office(room, "pelnomocnictwo", "klientka", czynnosci=())
    assert power_version_of(room, "pelnomocnictwo") == 1

    room.run("dopisz_czynnosc_umocowania", umocowanie_id="pelnomocnictwo", czynnosc="odbior_odpisu")
    assert power_version_of(room, "pelnomocnictwo") == 2

    room.run(
        "zmien_dane_umocowania",
        **power_fields(umocowane_biuro=1, zakres="do_czynnosci", wazne_do="2027-01-31"),
    )
    assert power_version_of(room, "pelnomocnictwo") == 3

    room.refused(
        "czynnosc",
        "dopisz_czynnosc_umocowania",
        umocowanie_id="pelnomocnictwo",
        czynnosc="Odbiór odpisu",
    )
    room.refused_sql(
        "czynności w zakresie umocowania się nie zmienia",
        "UPDATE sprawa_umocowania_czynnosci SET czynnosc = 'uzgodnienie_terminu_aktu'",
    )


# Correspondence ----------------------------------------------------------------------------------


@scenario("Wysyłka: adresat to uczestnik albo instytucja z nazwą; adresata GIIF nie ma")
def letter_addressee():
    room = new_room()
    add_person(room, "klientka")

    room.refused(
        "adresat_uczestnik_albo_instytucja",
        "przygotuj_wysylke",
        id="dwoch-adresatow",
        adresat_uczestnik_id="klientka",
        adresat_rodzaj="bank",
        adresat_nazwa="Bank",
        w_imieniu="biuro",
        kanal="email",
        informacja_rodo=0,
        ujawnia_agenta=0,
    )
    room.refused(
        "adresat_uczestnik_albo_instytucja",
        "przygotuj_wysylke",
        id="bez-nazwy",
        adresat_rodzaj="bank",
        w_imieniu="biuro",
        kanal="email",
        informacja_rodo=0,
        ujawnia_agenta=0,
    )
    room.refused(
        "adresat_rodzaj",
        "przygotuj_wysylke",
        id="giif",
        adresat_rodzaj="giif",
        adresat_nazwa="Generalny Inspektor Informacji Finansowej",
        w_imieniu="biuro",
        kanal="list",
        informacja_rodo=0,
        ujawnia_agenta=0,
    )

    prepare_letter(room, "do-wspolnoty")
    prepare_letter(
        room,
        "do-klientki",
        adresat_rodzaj=None,
        adresat_nazwa=None,
        adresat_uczestnik_id="klientka",
    )


@scenario("Wysyłka: powstaje jako projekt i idzie tylko naprzód")
def letter_moves_forward():
    room = new_room()

    room.refused_sql(
        "wysyłka powstaje jako projekt",
        """INSERT INTO sprawa_korespondencja_wysylki (id, adresat_rodzaj, adresat_nazwa, w_imieniu,
          kanal, wzor, wzor_wersja, sciezka, skrot, stan, wyslano)
        VALUES ('od-razu', 'bank', 'Bank', 'biuro', 'email', 'w', '1', 'w.pdf', 'sha256-w',
          'wyslana', '2026-11-05T10:00:00Z')""",
    )

    prepare_letter(room, "pismo")
    send(room, "pismo")
    room.refused("wysyłka idzie tylko naprzód", "anuluj_wysylke", id="pismo")
    room.run("odnotuj_doreczenie", id="pismo", doreczono="2026-11-06T10:00:00Z")
    room.refused("wysyłka idzie tylko naprzód", "odnotuj_zwrot", id="pismo")
    room.refused_sql(
        "wysyłka idzie tylko naprzód",
        "UPDATE sprawa_korespondencja_wysylki SET stan = 'wyslana' WHERE id = 'pismo'",
    )

    prepare_letter(room, "niepotrzebne")
    room.run("anuluj_wysylke", id="niepotrzebne")
    room.refused(
        "wysyłka idzie tylko naprzód",
        "przekaz_wysylke_do_zgody",
        id="niepotrzebne",
    )


@scenario(
    "Wysyłka: pismo spoza wzorów czeka na zgodę i wychodzi z zatwierdzoną decyzją, która je wskazuje"
)
def letter_outside_templates():
    room = new_room()
    prepare_letter(room, "pismo-wlasne", wzor=None, wzor_wersja=None)
    prepare_letter(room, "inne-pismo", wzor=None, wzor_wersja=None)
    refusal = "wysyłka za zgodą wychodzi tylko z zatwierdzoną decyzją wyslanie"

    room.refused(
        "pismo spoza wzorów przed wysłaniem czeka na zgodę",
        "odnotuj_wyslanie",
        id="pismo-wlasne",
        wyslano="2026-11-05T10:00:00Z",
    )

    # A request for consent describes content that no longer changes.
    room.refused(
        "o decyzję wyslanie prosi się dla wysyłki, która czeka na zgodę",
        "popros_o_decyzje",
        id="za-wczesnie",
        rodzaj="wyslanie",
        prosi="agent",
        prosi_stanowisko="opiekun_najmu",
        decyduje="czlowiek",
        zlozono="2026-11-03T10:00:00Z",
        dotyczy_tabela="sprawa_korespondencja_wysylki",
        dotyczy_id="pismo-wlasne",
    )

    room.run("przekaz_wysylke_do_zgody", id="pismo-wlasne")
    room.run("przekaz_wysylke_do_zgody", id="inne-pismo")
    room.refused(
        refusal,
        "odnotuj_wyslanie",
        id="pismo-wlasne",
        wyslano="2026-11-05T10:00:00Z",
    )

    # A decision the letter points at is checked as it is written (decyzja wysyłki to …).
    wrong_decision = "decyzja wysyłki to zatwierdzona decyzja wyslanie, która ją wskazuje"
    approved_decision(
        room, "zgoda-na-inne", "wyslanie", "sprawa_korespondencja_wysylki", "inne-pismo"
    )
    room.refused(
        wrong_decision,
        "odnotuj_wyslanie",
        id="pismo-wlasne",
        wyslano="2026-11-05T10:00:00Z",
        decyzja_id="zgoda-na-inne",
    )

    ask_for_decision(room, "odmowa", "wyslanie", "sprawa_korespondencja_wysylki", "pismo-wlasne")
    approve(room, "odmowa", rozstrzygniecie="odmowa")
    room.refused(
        wrong_decision,
        "odnotuj_wyslanie",
        id="pismo-wlasne",
        wyslano="2026-11-05T10:00:00Z",
        decyzja_id="odmowa",
    )

    # The workaround from the audit: after the consent the agent swaps the file and sends.
    approved_decision(room, "zgoda", "wyslanie", "sprawa_korespondencja_wysylki", "pismo-wlasne")
    room.refused(
        "treść wysyłki zmienia się tylko w projekcie",
        "zmien_tresc_wysylki",
        id="pismo-wlasne",
        sciezka="wysylki/b.pdf",
        skrot="sha256-b",
    )
    send(room, "pismo-wlasne", decyzja_id="zgoda")


@scenario(
    "Wysyłka ze wzoru, która czeka na zgodę, wychodzi tylko z decyzją; bez zgody od razu (D15)"
)
def letter_waiting_for_consent():
    room = new_room()
    prepare_letter(room, "ze-wzoru")
    prepare_letter(room, "za-zgoda")
    room.run("przekaz_wysylke_do_zgody", id="za-zgoda")

    send(room, "ze-wzoru")
    room.refused(
        "wysyłka za zgodą wychodzi tylko z zatwierdzoną decyzją wyslanie",
        "odnotuj_wyslanie",
        id="za-zgoda",
        wyslano="2026-11-05T10:00:00Z",
    )

    approved_decision(room, "zgoda", "wyslanie", "sprawa_korespondencja_wysylki", "za-zgoda")
    send(room, "za-zgoda", decyzja_id="zgoda")


@scenario(
    "Wysyłka: decyzja to zatwierdzona decyzja wyslanie, która ją wskazuje, także przy wysłaniu z projektu"
)
def letter_decision_points_at_letter():
    room = new_room()
    prepare_letter(room, "ze-wzoru")
    prepare_letter(room, "za-zgoda")
    room.run("przekaz_wysylke_do_zgody", id="za-zgoda")
    approved_decision(room, "zgoda", "wyslanie", "sprawa_korespondencja_wysylki", "za-zgoda")
    approved_decision(room, "inna", "inna", "sprawa_korespondencja_wysylki", "za-zgoda")
    rule = "decyzja wysyłki to zatwierdzona decyzja wyslanie, która ją wskazuje"

    # The workaround from the builders: a letter from a draft sent with someone else's consent.
    room.refused(
        rule,
        "odnotuj_wyslanie",
        id="ze-wzoru",
        wyslano="2026-11-05T10:00:00Z",
        decyzja_id="zgoda",
    )
    room.refused_sql(
        rule,
        "UPDATE sprawa_korespondencja_wysylki SET decyzja_id = 'inna' WHERE id = 'za-zgoda'",
    )

    send(room, "ze-wzoru")
    send(room, "za-zgoda", decyzja_id="zgoda")


@scenario("Wysyłka: wzór się nie zmienia, a pismo spoza wzorów nie staje się pismem ze wzoru")
def letter_template_fixed():
    room = new_room()
    prepare_letter(room, "ze-wzoru")
    prepare_letter(room, "spoza-wzorow", wzor=None, wzor_wersja=None)
    refusal = "wzór wysyłki się nie zmienia"

    room.refused_sql(
        refusal,
        "UPDATE sprawa_korespondencja_wysylki SET wzor_wersja = '4' WHERE id = 'ze-wzoru'",
    )

    # The workaround from the audit: a letter outside the templates names a template and goes
    # out without a decision in one write.
    room.refused_sql(
        refusal,
        """UPDATE sprawa_korespondencja_wysylki
        SET wzor = 'cokolwiek', wzor_wersja = '9', stan = 'wyslana',
          wyslano = '2026-11-05T10:00:00Z'
        WHERE id = 'spoza-wzorow'""",
    )

    # Another template is a new draft; the old one is cancelled.
    room.run("anuluj_wysylke", id="ze-wzoru")
    prepare_letter(room, "ze-wzoru-4", wzor_wersja="4")
    send(room, "ze-wzoru-4")


@scenario(
    "Wysyłka w imieniu uczestnika wymaga jego ważnego umocowania biura, które obejmuje czynność"
)
def letter_on_behalf_of_participant():
    room = new_room()
    add_person(room, "klientka")
    add_person(room, "sasiad", imiona="Piotr")
    add_power_for_office(room, "pelnomocnictwo", "klientka")
    add_power_for_office(room, "od-sasiada", "sasiad")
    add_power_for_office(room, "przeterminowane", "klientka")
    add_power_for_office(room, "pozniejsze", "klientka", udzielono="2026-11-10")
    room.run(
        "zmien_dane_umocowania",
        **power_fields(
            id="przeterminowane", umocowane_biuro=1, zakres="do_czynnosci", wazne_do="2026-11-01"
        ),
    )
    add_power_for_office(room, "odwolane", "klientka")
    room.run(
        "odnotuj_wygasniecie_umocowania",
        id="odwolane",
        wygaslo="2026-11-04",
        wygasniecie_powod="odwolanie",
        wygasniecie_zrodlo="wiadomość klientki",
    )
    room.run(
        "wpisz_umocowanie",
        **power_fields(id="dla-syna", umocowany_uczestnik_id="sasiad", zakres="do_czynnosci"),
    )
    room.run(
        "dopisz_czynnosc_umocowania", umocowanie_id="dla-syna", czynnosc="wystapienie_do_wspolnoty"
    )

    room.refused(
        "w_imieniu_uczestnika_z_umocowaniem_i_czynnoscia",
        "przygotuj_wysylke",
        id="bez-umocowania",
        adresat_rodzaj="wspolnota",
        adresat_nazwa="Wspólnota",
        w_imieniu="uczestnik",
        w_imieniu_uczestnik_id="klientka",
        kanal="email",
        informacja_rodo=0,
        ujawnia_agenta=0,
    )

    def on_behalf(letter_id, umocowanie_id, czynnosc="wystapienie_do_wspolnoty"):
        prepare_letter(
            room,
            letter_id,
            w_imieniu="uczestnik",
            w_imieniu_uczestnik_id="klientka",
            umocowanie_id=umocowanie_id,
            czynnosc=czynnosc,
        )

    refusal = "pismo w imieniu uczestnika wymaga jego niewygasłego umocowania biura"
    for letter_id, umocowanie_id, czynnosc in (
        ("poza-zakresem", "pelnomocnictwo", "uzgodnienie_terminu_aktu"),
        ("cudze", "od-sasiada", "wystapienie_do_wspolnoty"),
        ("po-terminie", "przeterminowane", "wystapienie_do_wspolnoty"),
        ("po-odwolaniu", "odwolane", "wystapienie_do_wspolnoty"),
        ("przed-udzieleniem", "pozniejsze", "wystapienie_do_wspolnoty"),
        ("dla-osoby", "dla-syna", "wystapienie_do_wspolnoty"),
    ):
        on_behalf(letter_id, umocowanie_id, czynnosc)
        room.refused(refusal, "odnotuj_wyslanie", id=letter_id, wyslano="2026-11-05T10:00:00Z")

    on_behalf("w-zakresie", "pelnomocnictwo")
    send(room, "w-zakresie")


@scenario("Wysyłka: treść i załączniki zmieniają się tylko w projekcie, a ślad wysłania raz")
def letter_content_frozen_outside_draft():
    room = new_room()
    add_person(room, "klientka")
    add_power_for_office(room, "pelnomocnictwo", "klientka")
    add_power_for_office(room, "inne", "klientka", czynnosci=("inna_czynnosc",))
    add_received_document(room, "polisa", "polisa")
    add_received_document(room, "aneks", "umowa")
    prepare_letter(
        room,
        "pismo",
        w_imieniu="uczestnik",
        w_imieniu_uczestnik_id="klientka",
        umocowanie_id="pelnomocnictwo",
        czynnosc="wystapienie_do_wspolnoty",
    )

    room.run("zmien_tresc_wysylki", id="pismo", sciezka="wysylki/pismo-2.pdf", skrot="sha256-2")
    room.run("dolacz_dokument_do_wysylki", wysylka_id="pismo", dokument_id="polisa")
    send(room, "pismo")

    refusal = "treść wysyłki zmienia się tylko w projekcie"
    room.refused(
        refusal,
        "zmien_tresc_wysylki",
        id="pismo",
        sciezka="wysylki/pismo-3.pdf",
        skrot="sha256-3",
    )

    # The workaround from the audit: after sending, the trail is rewritten with another power,
    # another act, another addressee and an earlier moment.
    room.refused_sql(
        refusal,
        """UPDATE sprawa_korespondencja_wysylki
        SET umocowanie_id = 'inne', czynnosc = 'inna_czynnosc', adresat_nazwa = 'Urząd'
        WHERE id = 'pismo'""",
    )
    room.refused_sql(
        "decyzję, chwile wysłania i doręczenia, dowód doręczenia i odnośnik wysyłki wpisuje się raz",
        "UPDATE sprawa_korespondencja_wysylki SET wyslano = '2025-01-01' WHERE id = 'pismo'",
    )
    room.refused(
        "załączniki dołącza się tylko do projektu wysyłki",
        "dolacz_dokument_do_wysylki",
        wysylka_id="pismo",
        dokument_id="aneks",
    )
    room.refused_sql(
        "załącznika się nie zmienia",
        "UPDATE sprawa_korespondencja_zalaczniki SET dokument_id = 'aneks'",
    )

    # The state, the moment and the proof of delivery still come in.
    room.run(
        "odnotuj_doreczenie",
        id="pismo",
        doreczono="2026-11-05T12:00:00Z",
        dowod_doreczenia="potwierdzenie odbioru",
    )

    # A draft waiting for consent is frozen too, attachments included.
    prepare_letter(room, "czeka")
    room.run("przekaz_wysylke_do_zgody", id="czeka")
    room.refused_sql(
        refusal,
        "UPDATE sprawa_korespondencja_wysylki SET kanal = 'list' WHERE id = 'czeka'",
    )
    room.refused(
        "załączniki dołącza się tylko do projektu wysyłki",
        "dolacz_dokument_do_wysylki",
        wysylka_id="czeka",
        dokument_id="aneks",
    )
    room.refused_sql(
        "wysyłka powstaje jako projekt, bez decyzji i śladu wysłania",
        """INSERT INTO sprawa_korespondencja_wysylki (id, adresat_rodzaj, adresat_nazwa, w_imieniu,
          kanal, wzor, wzor_wersja, stan, wyslano)
        VALUES ('z-chwila', 'urzad', 'Urząd', 'biuro', 'email', 'w', '1', 'projekt',
          '2026-11-05T10:00:00Z')""",
    )


@scenario("Wysyłka: wysłana ma plik i chwilę wysłania, doręczona chwilę doręczenia")
def letter_sent_has_file():
    room = new_room()
    prepare_letter(room, "bez-pliku", sciezka=None, skrot=None)

    room.refused(
        "wyslana_ma_plik_i_chwile",
        "odnotuj_wyslanie",
        id="bez-pliku",
        wyslano="2026-11-05T10:00:00Z",
    )

    prepare_letter(room, "pismo")
    send(room, "pismo")
    room.refused_sql(
        "doreczona_ma_chwile",
        "UPDATE sprawa_korespondencja_wysylki SET stan = 'doreczona' WHERE id = 'pismo'",
    )


@scenario("Wpływ: niezgodny kontakt ma opis, a wpływów się nie zmienia")
def inflow_rules():
    room = new_room()

    room.refused("niezgodny_kontakt_ma_opis", "wpisz_wplyw", **inflow_fields(niezgodny_kontakt=1))
    room.run(
        "wpisz_wplyw",
        **inflow_fields(
            niezgodny_kontakt=1,
            niezgodnosc_opis="„kancelaria” pisze z adresu spoza kontaktów teczki",
        ),
    )

    room.refused_sql(
        "wpływów się nie zmienia",
        "UPDATE sprawa_korespondencja_wplywy SET niezgodny_kontakt = 0, niezgodnosc_opis = NULL",
    )


def inflow_fields(**fields):
    return {
        "id": "wplyw",
        "nadawca": "kancelaria@example.pl",
        "kanal": "email",
        "wplynelo": "2026-11-03T08:00:00Z",
        "sciezka": "wplywy/wplyw.eml",
        "skrot": "sha256-wplyw",
        "niezgodny_kontakt": 0,
        **fields,
    }


@scenario("Załącznik należy do wysyłki albo do wpływu")
def attachment_of_letter_or_inflow():
    room = new_room()
    add_received_document(room, "zaswiadczenie", "zaswiadczenie")
    prepare_letter(room, "pismo")
    add_inflow(room, "wplyw")

    room.refused_sql(
        "zalacznik_wysylki_albo_wplywu",
        """INSERT INTO sprawa_korespondencja_zalaczniki (wysylka_id, wplyw_id, dokument_id)
        VALUES ('pismo', 'wplyw', 'zaswiadczenie')""",
    )
    room.refused_sql(
        "zalacznik_wysylki_albo_wplywu",
        "INSERT INTO sprawa_korespondencja_zalaczniki (dokument_id) VALUES ('zaswiadczenie')",
    )

    room.run("dolacz_dokument_do_wplywu", wplyw_id="wplyw", dokument_id="zaswiadczenie")
    room.run("dolacz_dokument_do_wysylki", wysylka_id="pismo", dokument_id="zaswiadczenie")
    assert len(room.run("zalaczniki")) == 2


# Consents ----------------------------------------------------------------------------------------


@scenario("Zgoda ma dokument albo wiadomość, nie zmienia się, a nowa zastępuje poprzednią")
def consent_rules():
    room = new_room()
    add_person(room, "klientka")
    add_received_document(room, "oswiadczenie", "oswiadczenie")

    consent = {
        "uczestnik_id": "klientka",
        "rodzaj": "zatwierdzenie_ceny",
        "tresc": "Zatwierdzam cenę ofertową 650 000 zł.",
        "zlozono": "2026-11-04T10:00:00Z",
        "trwaly_nosnik": 0,
    }
    room.refused("zgoda_ma_dokument_albo_wiadomosc", "wpisz_zgode", id="ustna", **consent)
    room.refused(
        "rodzaj", "wpisz_zgode", id="zly-rodzaj", **{**consent, "rodzaj": "Zatwierdzenie ceny"}
    )
    room.run("wpisz_zgode", id="cena-1", dokument_id="oswiadczenie", **consent)

    room.refused_sql(
        "zgody się nie zmienia",
        "UPDATE sprawa_zgody_wpisy SET tresc = 'Zatwierdzam cenę 600 000 zł.'",
    )

    add_inflow(room, "nowa-cena", "klientka")
    room.run(
        "wpisz_zgode",
        id="cena-2",
        wplyw_id="nowa-cena",
        zastepuje_id="cena-1",
        **{**consent, "tresc": "Zatwierdzam cenę 640 000 zł.", "zlozono": "2026-11-06T10:00:00Z"},
    )
    assert [row["id"] for row in room.run("zgody_biezace")] == ["cena-2"]
    room.refused(
        "nie_zastepuje_samej_siebie",
        "wpisz_zgode",
        id="cena-3",
        wplyw_id="nowa-cena",
        zastepuje_id="cena-3",
        **consent,
    )


# Contracts with the office -----------------------------------------------------------------------


@scenario("Umowa: powstaje jako projekt i przechodzi tylko dozwolonymi krokami")
def contract_states():
    room = new_room()
    add_person(room, "klientka")
    check(room, "tozsamosc", "tozsamosc", uczestnik_id="klientka")

    room.refused_sql(
        "umowa powstaje jako projekt",
        """INSERT INTO sprawa_umowy_wpisy (id, rodzaj, stan, obie_strony, wylacznosc)
        VALUES ('od-razu', 'posrednictwo', 'zawarta', 0, 0)""",
    )

    prepare_contract(room, "umowa", ["klientka"])
    add_received_document(room, "podpisana-wczesniej", "umowa_z_biurem")
    add_received_document(room, "polisa-wczesniej", "polisa")
    room.refused(
        "umowa przechodzi",
        "odnotuj_zawarcie_umowy",
        id="umowa",
        forma="pisemna",
        sposob_zawarcia="w_lokalu_biura",
        zawarto="2026-11-04T12:00:00Z",
        podpisal_za_biuro="tozsamosc-posrednika",
        dokument_id="podpisana-wczesniej",
        polisa_dokument_id="polisa-wczesniej",
    )
    room.run("przekaz_umowe_do_podpisu", id="umowa")
    room.run("cofnij_umowe_do_projektu", id="umowa")
    room.run("przekaz_umowe_do_podpisu", id="umowa")
    conclude_contract(room, "umowa")
    room.refused("umowa przechodzi", "cofnij_umowe_do_projektu", id="umowa")

    room.refused_sql(
        "zakonczona_ma_powod",
        "UPDATE sprawa_umowy_wpisy SET stan = 'zakonczona' WHERE id = 'umowa'",
    )
    room.run("zakoncz_umowe", id="umowa", koniec_powod="wypowiedzenie przez klientkę")
    room.refused("umowa przechodzi", "przekaz_umowe_do_podpisu", id="umowa")


@scenario(
    "Umowa idzie do podpisu, gdy każda strona ma aktualne sprawdzenie tożsamości „zgodne” (R7)"
)
def contract_to_signing_after_identity_checks():
    room = new_room()
    add_person(room, "zona")
    add_person(room, "maz", imiona="Piotr")
    refusal = "umowa idzie do podpisu, gdy ma strony, a każda ma aktualne sprawdzenie tożsamości"

    prepare_contract(room, "bez-stron", [])
    room.refused(refusal, "przekaz_umowe_do_podpisu", id="bez-stron")

    prepare_contract(room, "umowa", ["zona", "maz"])
    check(room, "tozsamosc-zony", "tozsamosc", uczestnik_id="zona")
    room.refused(refusal, "przekaz_umowe_do_podpisu", id="umowa")

    check(room, "tozsamosc-meza", "tozsamosc", uczestnik_id="maz", wynik="uwagi")
    room.refused(refusal, "przekaz_umowe_do_podpisu", id="umowa")

    check(room, "tozsamosc-meza-2", "tozsamosc", uczestnik_id="maz")
    room.run(
        "zmien_dane_uczestnika",
        id="zona",
        rodzaj_podmiotu="osoba_fizyczna",
        imiona="Anna",
        nazwisko="Nowak",
        pesel_nie_nadano=0,
        konsument=0,
        ceidg=0,
    )
    room.refused(refusal, "przekaz_umowe_do_podpisu", id="umowa")

    check(room, "tozsamosc-zony-2", "tozsamosc", uczestnik_id="zona")
    room.run("przekaz_umowe_do_podpisu", id="umowa")

    add_person(room, "corka", imiona="Ewa")
    room.refused(
        "strony dopisuje się tylko do projektu umowy",
        "dopisz_strone_umowy",
        umowa_id="umowa",
        uczestnik_id="corka",
    )
    room.refused_sql(
        "strony umowy się nie zmienia",
        "UPDATE sprawa_umowy_strony SET uczestnik_id = 'zona'",
    )


@scenario("Umowa: dopóki stoi znacznik wątpliwości, nie idzie do podpisu")
def contract_blocked_by_doubt():
    room = new_room()
    add_person(room, "klientka")
    check(room, "tozsamosc", "tozsamosc", uczestnik_id="klientka")
    prepare_contract(room, "umowa", ["klientka"])
    room.run(
        "postaw_watpliwosc",
        id="watpliwosc",
        postawil="tozsamosc-kontrolera",
        postawiono="2026-11-03T09:00:00Z",
    )

    room.refused(
        "dopóki stoi znacznik wątpliwości, umowa nie idzie do podpisu",
        "przekaz_umowe_do_podpisu",
        id="umowa",
    )

    approved_decision(
        room, "zdjecie", "zdjecie_watpliwosci", "sprawa_metryka_watpliwosci", "watpliwosc"
    )
    room.run("zdejmij_watpliwosc", id="watpliwosc", zdjecie_decyzja_id="zdjecie")
    room.run("przekaz_umowe_do_podpisu", id="umowa")


@scenario(
    "Umowa zawarta ma podpisany dokument umowy z biurem, kopię polisy, formę i sposób zawarcia"
)
def contract_concluded_with_policy():
    room = new_room()
    add_person(room, "klientka")
    check(room, "tozsamosc", "tozsamosc", uczestnik_id="klientka")
    prepare_contract(room, "umowa", ["klientka"])
    room.run("przekaz_umowe_do_podpisu", id="umowa")
    add_received_document(room, "podpisana", "umowa_z_biurem")
    add_received_document(room, "polisa", "polisa")
    add_received_document(room, "nie-polisa", "zaswiadczenie")
    room.run("wpisz_wymagany_dokument", id="polisa-w-drodze", rodzaj="polisa", nazwa="Polisa OC")

    concluded = {
        "id": "umowa",
        "forma": "elektroniczna",
        "sposob_zawarcia": "na_odleglosc",
        "zawarto": "2026-11-04T12:00:00Z",
        "podpisal_za_biuro": "tozsamosc-posrednika",
        "dokument_id": "podpisana",
        "polisa_dokument_id": "polisa",
    }
    add_received_document(room, "pismo", "pismo")
    refusal = "umowa zawarta ma podpisany dokument umowy z biurem, kopię polisy, formę i sposób"
    for missing in (
        {"dokument_id": "pismo"},
        {"polisa_dokument_id": None},
        {"polisa_dokument_id": "nie-polisa"},
        {"polisa_dokument_id": "polisa-w-drodze"},
        {"dokument_id": None},
        {"sposob_zawarcia": None},
    ):
        room.refused(refusal, "odnotuj_zawarcie_umowy", **{**concluded, **missing})

    room.run("odnotuj_zawarcie_umowy", **concluded)
    assert room.run("umowy")[0]["warunki_od"] == "2026-11-04T12:00:00Z"
    room.refused_sql(
        "dane zawarcia umowy się nie zmieniają",
        "UPDATE sprawa_umowy_wpisy SET forma = NULL WHERE id = 'umowa'",
    )


@scenario(
    "Żądanie rozpoczęcia pracy wskazuje zgodę strony, a przy umowie poza lokalem trwały nośnik"
)
def contract_early_start_request():
    room = new_room()
    add_person(room, "klientka", konsument=1)
    add_person(room, "sasiad", imiona="Piotr")
    check(room, "tozsamosc", "tozsamosc", uczestnik_id="klientka")
    prepare_contract(room, "poza-lokalem", ["klientka"])
    room.run("przekaz_umowe_do_podpisu", id="poza-lokalem")
    conclude_contract(
        room, "poza-lokalem", sposob_zawarcia="poza_lokalem", odstapienie_do=WITHDRAWAL_ENDS
    )
    prepare_contract(room, "na-odleglosc", ["klientka"])
    room.run("przekaz_umowe_do_podpisu", id="na-odleglosc")
    conclude_contract(
        room, "na-odleglosc", sposob_zawarcia="na_odleglosc", odstapienie_do=WITHDRAWAL_ENDS
    )

    add_consent(room, "zadanie-sasiada", "sasiad", "zadanie_rozpoczecia_pracy")
    add_consent(room, "zadanie-w-mailu", "klientka", "zadanie_rozpoczecia_pracy")
    add_consent(room, "cena", "klientka", "zatwierdzenie_ceny")
    add_received_document(room, "podpisane-zadanie", "oswiadczenie")
    room.run(
        "wpisz_zgode",
        id="zadanie-na-nosniku",
        uczestnik_id="klientka",
        rodzaj="zadanie_rozpoczecia_pracy",
        tresc="Żądam rozpoczęcia pracy i wiem, że tracę prawo odstąpienia po pełnym wykonaniu.",
        zlozono="2026-11-04T13:00:00Z",
        dokument_id="podpisane-zadanie",
        trwaly_nosnik=1,
    )

    def request(event_id, umowa_id, zgoda_id):
        return {
            "id": event_id,
            "umowa_id": umowa_id,
            "rodzaj": "zadanie_rozpoczecia_pracy",
            "chwila": "2026-11-04T13:00:00Z",
            "zgoda_id": zgoda_id,
        }

    refusal = "żądanie rozpoczęcia pracy wskazuje zgodę zadanie_rozpoczecia_pracy strony umowy"
    room.refused(
        "zdarzenie_ma_dowod",
        "wpisz_zdarzenie_umowy",
        **{**request("aneks-bez-dowodu", "na-odleglosc", None), "rodzaj": "aneks"},
    )
    room.refused(
        refusal, "wpisz_zdarzenie_umowy", **request("cudze", "na-odleglosc", "zadanie-sasiada")
    )
    room.refused(refusal, "wpisz_zdarzenie_umowy", **request("cena", "na-odleglosc", "cena"))
    room.run("wpisz_zdarzenie_umowy", **request("zadanie-1", "na-odleglosc", "zadanie-w-mailu"))

    room.refused(
        "przy umowie poza lokalem żądanie rozpoczęcia pracy jest na trwałym nośniku",
        "wpisz_zdarzenie_umowy",
        **request("zadanie-2", "poza-lokalem", "zadanie-w-mailu"),
    )
    room.run("wpisz_zdarzenie_umowy", **request("zadanie-3", "poza-lokalem", "zadanie-na-nosniku"))

    room.refused_sql(
        "zdarzenia umowy się nie zmienia",
        "UPDATE sprawa_umowy_zdarzenia SET chwila = '2026-11-03T13:00:00Z'",
    )


@scenario("Odstąpienie od umowy wskazuje oświadczenie konsumenta, który jest jej stroną")
def contract_withdrawal():
    room = new_room()
    add_person(room, "konsumentka", konsument=1)
    add_person(room, "przedsiebiorca", imiona="Piotr", nazwa="Nowak Inwestycje", ceidg=1)
    for party_id in ("konsumentka", "przedsiebiorca"):
        check(room, f"tozsamosc-{party_id}", "tozsamosc", uczestnik_id=party_id)
        prepare_contract(room, f"umowa-{party_id}", [party_id])
        room.run("przekaz_umowe_do_podpisu", id=f"umowa-{party_id}")
        # Only the consumer may withdraw, so only her contract has the last day to do it.
        conclude_contract(
            room,
            f"umowa-{party_id}",
            sposob_zawarcia="na_odleglosc",
            odstapienie_do=WITHDRAWAL_ENDS if party_id == "konsumentka" else None,
        )
        add_consent(
            room,
            f"odstapienie-{party_id}",
            party_id,
            "odstapienie",
            tresc="Odstępuję od umowy pośrednictwa.",
        )

    def withdrawal(party_id):
        return {
            "id": f"zdarzenie-{party_id}",
            "umowa_id": f"umowa-{party_id}",
            "rodzaj": "odstapienie",
            "chwila": "2026-11-10T09:00:00Z",
            "zgoda_id": f"odstapienie-{party_id}",
        }

    room.refused(
        "odstąpienie wskazuje oświadczenie odstapienie konsumenta",
        "wpisz_zdarzenie_umowy",
        **withdrawal("przedsiebiorca"),
    )
    room.run("wpisz_zdarzenie_umowy", **withdrawal("konsumentka"))
    # The contract had no end; the withdrawal gives it one without an annex.
    room.run("odnotuj_koniec_po_wypowiedzeniu", id="umowa-konsumentka", koniec="2026-11-10")
    room.run("zakoncz_umowe", id="umowa-konsumentka", koniec_powod="odstąpienie konsumentki")


def concluded_contract_for(room, contract_id, party_id, sposob_zawarcia="w_lokalu_biura"):
    check(room, f"tozsamosc-{contract_id}", "tozsamosc", uczestnik_id=party_id)
    prepare_contract(room, contract_id, [party_id], koniec="2027-11-04")
    room.run("przekaz_umowe_do_podpisu", id=contract_id)
    conclude_contract(room, contract_id, sposob_zawarcia)


@scenario("Zdarzenia umowy powstają tylko przy umowie zawartej albo zakończonej")
def contract_events_only_after_conclusion():
    room = new_room()
    add_person(room, "klientka", konsument=1)
    add_consent(room, "zadanie-w-mailu", "klientka", "zadanie_rozpoczecia_pracy")
    check(room, "tozsamosc", "tozsamosc", uczestnik_id="klientka")
    prepare_contract(room, "projekt", ["klientka"])
    prepare_contract(room, "bez-podpisu", ["klientka"])
    room.run("zakoncz_umowe", id="bez-podpisu", koniec_powod="klientka nie podpisała")
    refusal = "zdarzenie powstaje tylko przy umowie zawartej albo zakończonej"

    # The workaround from the audit: a request without a durable medium on the draft, before the
    # way of conclusion is known.
    for umowa_id in ("projekt", "bez-podpisu"):
        room.refused(
            refusal,
            "wpisz_zdarzenie_umowy",
            id=f"zadanie-{umowa_id}",
            umowa_id=umowa_id,
            rodzaj="zadanie_rozpoczecia_pracy",
            chwila="2026-11-04T13:00:00Z",
            zgoda_id="zadanie-w-mailu",
        )

    room.run("przekaz_umowe_do_podpisu", id="projekt")
    conclude_contract(
        room, "projekt", sposob_zawarcia="na_odleglosc", odstapienie_do=WITHDRAWAL_ENDS
    )
    room.run(
        "wpisz_zdarzenie_umowy",
        id="zadanie",
        umowa_id="projekt",
        rodzaj="zadanie_rozpoczecia_pracy",
        chwila="2026-11-04T13:00:00Z",
        zgoda_id="zadanie-w-mailu",
    )


@scenario("Umowa po zawarciu: dane zawarcia stoją, warunki zmienia aneks, a polisę nowa polisa")
def contract_frozen_after_conclusion():
    room = new_room()
    add_person(room, "klientka")
    add_received_document(room, "podpisany-aneks", "umowa_z_biurem")
    add_received_document(room, "nowa-polisa", "polisa")
    concluded_contract_for(room, "umowa", "klientka")
    zawarcia = "dane zawarcia umowy się nie zmieniają"

    # The workaround from the audit: the conclusion rewritten after the fact.
    room.refused_sql(
        zawarcia,
        """UPDATE sprawa_umowy_wpisy
        SET zawarto = '2020-01-01', sposob_zawarcia = 'poza_lokalem',
          forma = 'elektroniczna', podpisal_za_biuro = 'ktos'
        WHERE id = 'umowa'""",
    )
    prepare_letter(room, "spozniona-informacja", wzor="informacja-o-odstapieniu")
    room.refused(
        zawarcia,
        "odnotuj_informacje_o_odstapieniu",
        id="umowa",
        informacja_o_odstapieniu_wysylka_id="spozniona-informacja",
    )

    aneks = "warunki zawartej umowy zmienia tylko aneks"
    terms = {
        "id": "umowa",
        "koniec": "2028-11-04",
        "okres_wypowiedzenia": "3 miesiące",
        "wylacznosc": 0,
        "wynagrodzenie_opis": "8% czynszu miesięcznie",
    }
    room.refused_sql(aneks, "UPDATE sprawa_umowy_wpisy SET wylacznosc = 0 WHERE id = 'umowa'")
    room.refused(aneks, "odnotuj_aneks_umowy", warunki_od="2026-12-01T10:00:00Z", **terms)
    room.refused(aneks, "zmien_warunki_umowy", **terms)

    room.run(
        "wpisz_zdarzenie_umowy",
        id="aneks-1",
        umowa_id="umowa",
        rodzaj="aneks",
        chwila="2026-12-01T10:00:00Z",
        dokument_id="podpisany-aneks",
    )
    room.refused(aneks, "odnotuj_aneks_umowy", warunki_od="2026-12-02T10:00:00Z", **terms)
    room.run("odnotuj_aneks_umowy", warunki_od="2026-12-01T10:00:00Z", **terms)
    assert room.run("umowy")[0]["wylacznosc"] == 0

    # The same annex does not change the terms a second time.
    room.refused(
        aneks,
        "odnotuj_aneks_umowy",
        warunki_od="2026-12-01T10:00:00Z",
        **{**terms, "wylacznosc": 1},
    )

    polisa = "polisę zawartej umowy zmienia tylko zdarzenie nowa_polisa"
    room.refused(polisa, "odnotuj_nowa_polise_umowy", id="umowa", polisa_dokument_id="nowa-polisa")
    room.run(
        "wpisz_zdarzenie_umowy",
        id="polisa-2027",
        umowa_id="umowa",
        rodzaj="nowa_polisa",
        chwila="2027-01-02T10:00:00Z",
        dokument_id="nowa-polisa",
    )
    room.run("odnotuj_nowa_polise_umowy", id="umowa", polisa_dokument_id="nowa-polisa")

    room.run("zakoncz_umowe", id="umowa", koniec_powod="wypowiedzenie")
    room.refused_sql(zawarcia, "UPDATE sprawa_umowy_wpisy SET poczatek = '2026-01-01'")


@scenario(
    "Umowa po wypowiedzeniu albo odstąpieniu: koniec wpisuje się albo skraca bez aneksu, "
    "nie przed dniem zdarzenia"
)
def contract_end_after_notice():
    room = new_room()
    add_person(room, "klientka")
    add_received_document(room, "pismo-wypowiedzenia", "pismo")
    add_received_document(room, "podpisany-aneks", "umowa_z_biurem")
    concluded_contract_for(room, "umowa", "klientka")
    aneks = "warunki zawartej umowy zmienia tylko aneks"
    rule = "po wypowiedzeniu albo odstąpieniu koniec umowy wpisuje się albo skraca"

    # Without a notice the end is a term like any other.
    room.refused(aneks, "odnotuj_koniec_po_wypowiedzeniu", id="umowa", koniec="2027-01-31")

    room.run(
        "wpisz_zdarzenie_umowy",
        id="wypowiedzenie",
        umowa_id="umowa",
        rodzaj="wypowiedzenie",
        chwila="2026-12-01T10:00:00Z",
        dokument_id="pismo-wypowiedzenia",
    )
    room.refused(rule, "odnotuj_koniec_po_wypowiedzeniu", id="umowa", koniec="2026-11-30")
    room.refused(rule, "odnotuj_koniec_po_wypowiedzeniu", id="umowa", koniec="2028-01-31")
    room.refused(rule, "odnotuj_koniec_po_wypowiedzeniu", id="umowa", koniec=None)
    # The other terms still change only by an annex.
    room.refused_sql(
        aneks,
        "UPDATE sprawa_umowy_wpisy SET koniec = '2027-01-31', wylacznosc = 0 WHERE id = 'umowa'",
    )

    room.run("odnotuj_koniec_po_wypowiedzeniu", id="umowa", koniec="2027-01-31")
    room.run("odnotuj_koniec_po_wypowiedzeniu", id="umowa", koniec="2026-12-01")
    assert room.run("umowy")[0]["koniec"] == "2026-12-01"

    # An annex after the notice sets new terms; the earlier notice no longer shortens them.
    room.run(
        "wpisz_zdarzenie_umowy",
        id="aneks",
        umowa_id="umowa",
        rodzaj="aneks",
        chwila="2026-12-01T12:00:00Z",
        dokument_id="podpisany-aneks",
    )
    room.run(
        "odnotuj_aneks_umowy",
        id="umowa",
        koniec="2027-11-04",
        wylacznosc=1,
        wynagrodzenie_opis="10% czynszu miesięcznie",
        warunki_od="2026-12-01T12:00:00Z",
    )
    room.refused(aneks, "odnotuj_koniec_po_wypowiedzeniu", id="umowa", koniec="2027-01-31")


@scenario("Umowa: rodzaj nie zmienia się od wpisu, także w projekcie; inny rodzaj to nowy projekt")
def contract_kind_fixed():
    room = new_room()
    add_person(room, "klientka")
    prepare_contract(room, "umowa", ["klientka"])
    rule = "rodzaj umowy się nie zmienia"

    room.refused_sql(
        rule, "UPDATE sprawa_umowy_wpisy SET rodzaj = 'posrednictwo' WHERE id = 'umowa'"
    )

    room.run(
        "zmien_warunki_umowy",
        id="umowa",
        obie_strony=0,
        wylacznosc=0,
        wynagrodzenie_opis="8% czynszu miesięcznie",
    )
    prepare_contract(room, "posrednictwo", ["klientka"], rodzaj="posrednictwo")
    assert [row["rodzaj"] for row in room.run("umowy")] == ["zarzadzanie", "posrednictwo"]


@scenario("Dokument zawartej umowy i dowód zgody nie zmieniają rodzaju i nie są odrzucane")
def basis_documents_stay():
    room = new_room()
    add_person(room, "klientka")
    add_person(room, "sasiad", imiona="Piotr")
    concluded_contract_for(room, "umowa", "klientka")
    add_received_document(room, "oswiadczenie", "oswiadczenie", dotyczy_uczestnik_id="klientka")
    room.run(
        "wpisz_zgode",
        id="cena",
        uczestnik_id="klientka",
        rodzaj="zatwierdzenie_ceny",
        tresc="Zatwierdzam cenę.",
        zlozono="2026-11-04T10:00:00Z",
        dokument_id="oswiadczenie",
        trwaly_nosnik=1,
    )

    umowy = "dokument, na którym stoi zawarta umowa, nie zmienia rodzaju"
    zgody = "dokument, który dowodzi zgody, nie zmienia rodzaju"
    for document_id, refusal in (
        ("podpisana-umowa", umowy),
        ("polisa-umowa", umowy),
        ("oswiadczenie", zgody),
    ):
        room.refused(refusal, "odrzuc_dokument", id=document_id, powod_odrzucenia="nieczytelny")
        room.refused_sql(
            refusal,
            "UPDATE sprawa_dokumenty_wpisy SET rodzaj = 'inny' WHERE id = :id",
            id=document_id,
        )
        room.refused_sql(
            refusal,
            "UPDATE sprawa_dokumenty_wpisy SET dotyczy_uczestnik_id = 'sasiad' WHERE id = :id",
            id=document_id,
        )
        room.run("oznacz_dokument_jako_sprawdzony", id=document_id)

    # A document no contract or consent stands on is still rejected as before.
    add_received_document(room, "luzny", "pismo")
    room.run("odrzuc_dokument", id="luzny", powod_odrzucenia="nieczytelny")


@scenario("Zgoda wskazuje dokument z plikiem: otrzymany albo sprawdzony")
def consent_document_has_file():
    room = new_room()
    add_person(room, "klientka")
    room.run(
        "wpisz_wymagany_dokument",
        id="oswiadczenie",
        rodzaj="oswiadczenie",
        nazwa="Oświadczenie o cenie",
        wymaga="umowa",
    )
    consent = {
        "id": "cena",
        "uczestnik_id": "klientka",
        "rodzaj": "zatwierdzenie_ceny",
        "tresc": "Zatwierdzam cenę.",
        "zlozono": "2026-11-04T10:00:00Z",
        "dokument_id": "oswiadczenie",
        "trwaly_nosnik": 0,
    }

    # The workaround from the audit: a consent standing on a required document without a file.
    room.refused(
        "zgoda wskazuje dokument z plikiem: otrzymany albo sprawdzony", "wpisz_zgode", **consent
    )

    room.run(
        "odnotuj_otrzymanie_dokumentu",
        id="oswiadczenie",
        pochodzenie="od klientki",
        kanal="email",
        postac="skan",
        sciezka="dokumenty/oswiadczenie.pdf",
        skrot="sha256-oswiadczenie",
    )
    room.run("wpisz_zgode", **consent)


@scenario("Umocowanie: data udzielenia się nie zmienia, a wygaśnięcie wpisuje się raz")
def power_granted_and_expiry_fixed():
    room = new_room()
    add_person(room, "klientka")
    add_power_for_office(room, "pelnomocnictwo", "klientka")
    room.run("dopisz_czynnosc_umocowania", umocowanie_id="pelnomocnictwo", czynnosc="inna_czynnosc")

    # The workaround from the audit: a grant moved after the sending date.
    room.refused_sql(
        "data udzielenia umocowania się nie zmienia",
        "UPDATE sprawa_umocowania_wpisy SET udzielono = '2027-01-01' WHERE id = 'pelnomocnictwo'",
    )

    expiry = {
        "id": "pelnomocnictwo",
        "wygaslo": "2026-11-10",
        "wygasniecie_powod": "odwolanie",
        "wygasniecie_zrodlo": "wiadomość klientki z 10 listopada",
    }
    room.run("odnotuj_wygasniecie_umocowania", **expiry)

    # The workaround from the audit: a revoked power revived by clearing its expiry.
    once = "wygaśnięcie umocowania wpisuje się raz i się go nie cofa"
    room.refused_sql(
        once,
        """UPDATE sprawa_umocowania_wpisy
        SET wygaslo = NULL, wygasniecie_powod = NULL, wygasniecie_zrodlo = NULL
        WHERE id = 'pelnomocnictwo'""",
    )
    room.refused(once, "odnotuj_wygasniecie_umocowania", **{**expiry, "wygaslo": "2026-12-01"})


@scenario("Zakres umowy dopisuje się tylko do projektu")
def contract_scope_only_in_draft():
    room = new_room()
    add_person(room, "klientka")
    check(room, "tozsamosc", "tozsamosc", uczestnik_id="klientka")
    prepare_contract(room, "umowa", ["klientka"])
    room.run("dopisz_czynnosc_umowy", umowa_id="umowa", czynnosc="publikacja_ogloszenia")
    room.run("przekaz_umowe_do_podpisu", id="umowa")

    room.refused(
        "zakres dopisuje się tylko do projektu umowy",
        "dopisz_czynnosc_umowy",
        umowa_id="umowa",
        czynnosc="negocjacje_ceny",
    )
    conclude_contract(room, "umowa")
    room.refused(
        "zakres dopisuje się tylko do projektu umowy",
        "dopisz_czynnosc_umowy",
        umowa_id="umowa",
        czynnosc="negocjacje_ceny",
    )


@scenario("Zawarcie umowy sprawdza jeszcze raz tożsamość stron i znacznik wątpliwości")
def contract_conclusion_rechecks_signing_conditions():
    room = new_room()
    add_person(room, "klientka")
    check(room, "tozsamosc", "tozsamosc", uczestnik_id="klientka")
    prepare_contract(room, "umowa", ["klientka"])
    room.run("przekaz_umowe_do_podpisu", id="umowa")
    refusal = "umowa zostaje zawarta, gdy każda strona ma aktualne sprawdzenie tożsamości"

    # The workaround from the audit: identity data changed after the hand-over to signing.
    room.run(
        "zmien_dane_uczestnika",
        id="klientka",
        imiona="Anna",
        nazwisko="Zmieniona",
        pesel_nie_nadano=0,
        konsument=0,
        ceidg=0,
    )
    room.refused(refusal, "odnotuj_zawarcie_umowy", **concluded_fields(room, "umowa"))

    check(room, "tozsamosc-2", "tozsamosc", uczestnik_id="klientka")
    room.run(
        "postaw_watpliwosc",
        id="watpliwosc",
        postawil="tozsamosc-kontrolera",
        postawiono="2026-11-03T09:00:00Z",
    )
    room.refused(refusal, "odnotuj_zawarcie_umowy", **concluded_fields(room, "umowa"))

    approved_decision(
        room, "zdjecie", "zdjecie_watpliwosci", "sprawa_metryka_watpliwosci", "watpliwosc"
    )
    room.run("zdejmij_watpliwosc", id="watpliwosc", zdjecie_decyzja_id="zdjecie")
    room.run("odnotuj_zawarcie_umowy", **concluded_fields(room, "umowa"))


def concluded_fields(room, contract_id):
    """The conclusion of a contract with its signed document and policy, added when missing."""
    for document_id, rodzaj in (
        (f"podpisana-{contract_id}", "umowa_z_biurem"),
        ("polisa", "polisa"),
    ):
        if room.value("SELECT count(*) FROM sprawa_dokumenty_wpisy WHERE id = :id", id=document_id):
            continue
        add_received_document(room, document_id, rodzaj)

    return {
        "id": contract_id,
        "forma": "pisemna",
        "sposob_zawarcia": "w_lokalu_biura",
        "zawarto": "2026-11-04T12:00:00Z",
        "podpisal_za_biuro": "tozsamosc-posrednika",
        "dokument_id": f"podpisana-{contract_id}",
        "polisa_dokument_id": "polisa",
    }


@scenario("Znacznik konsumenta strony umowy, która wyszła z projektu, się nie zmienia")
def consumer_flag_of_party_fixed():
    room = new_room()
    add_person(room, "klientka", konsument=1)
    check(room, "tozsamosc", "tozsamosc", uczestnik_id="klientka")
    prepare_contract(room, "umowa", ["klientka"])
    refusal = "znacznik konsumenta strony umowy, która wyszła z projektu, się nie zmienia"

    # In the draft the flag is still corrected from the client's statement.
    room.execute("UPDATE sprawa_uczestnicy_wpisy SET konsument = 0 WHERE id = 'klientka'")
    room.execute("UPDATE sprawa_uczestnicy_wpisy SET konsument = 1 WHERE id = 'klientka'")

    room.run("przekaz_umowe_do_podpisu", id="umowa")
    room.refused_sql(
        refusal, "UPDATE sprawa_uczestnicy_wpisy SET konsument = 0 WHERE id = 'klientka'"
    )

    # The workaround from the audit: the flag cleared after the withdrawal of a concluded contract.
    conclude_contract(room, "umowa", sposob_zawarcia="na_odleglosc", odstapienie_do=WITHDRAWAL_ENDS)
    add_consent(room, "odstapienie", "klientka", "odstapienie", tresc="Odstępuję od umowy.")
    room.run(
        "wpisz_zdarzenie_umowy",
        id="zdarzenie-odstapienia",
        umowa_id="umowa",
        rodzaj="odstapienie",
        chwila="2026-11-06T10:00:00Z",
        zgoda_id="odstapienie",
    )
    room.run("zakoncz_umowe", id="umowa", koniec_powod="odstąpienie konsumentki")
    room.refused_sql(
        refusal, "UPDATE sprawa_uczestnicy_wpisy SET konsument = 0 WHERE id = 'klientka'"
    )
