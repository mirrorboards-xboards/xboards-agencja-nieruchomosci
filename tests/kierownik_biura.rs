//! Scenarios of the office manager's memory room (spec/positions/biuro/kierownik_biura).
//!
//! The room has the manager's own kinds and three kinds of the agent memory (findings, sources and
//! orders); the rules of those are checked in stanowisko.rs. The office manager is no department
//! head, so the room has no department memory. Each scenario starts from an empty room and checks
//! one rule the database keeps: a write the rule allows and a write it refuses. The first scenario
//! follows a day of the manager: a card assigned by a rule, reports, a decision list, an absence, a
//! new policy and the monthly report.

use xboards_agencja_nieruchomosci::positions::biuro::kierownik_biura::{
    self, kierownik_biura_decyzje, kierownik_biura_meldunki, kierownik_biura_nieobecnosci,
    kierownik_biura_polisy, kierownik_biura_przydzial, kierownik_biura_sprawozdania,
};
use xboards_agencja_nieruchomosci::xmeta_room::{FromSqlValue, MemoryRoom, MetaRoom, SqlValue};

use kierownik_biura_decyzje::{
    ListaDecyzjiParams, OdnotujOsobneZgloszenieParams, OdnotujWyslaniePozycjiParams,
    OdnowPozycjeParams, PozycjeCzekajaceOdParams, PozycjeDzial, PozycjePowodZdjecia,
    PozycjeZdjeteOdParams, PrzyjmijPozycjeParams, ZdejmijPozycjeParams,
};
use kierownik_biura_meldunki::{
    DopiszMeldunekDzialuParams, DopiszMeldunekRecepcjiParams, MeldunkiOdParams, Nadawca,
    WpisyNadawca,
};
use kierownik_biura_nieobecnosci::{
    NieobecnosciOdParams, NieobecnosciWDniuParams, OdwolajNieobecnoscParams, WpisyPodpisalJako,
    ZapiszNieobecnoscParams,
};
use kierownik_biura_polisy::{
    OdnotujPrzypomnienieOPolisieParams, OdnotujZawiadomienieOPolisieParams, PolisaWDniuParams,
    PolisyDoPrzypomnieniaParams, WpisyRodzaj as RodzajPolisy, WpisyZawiadomionoKogo,
    ZapiszPoliseParams,
};
use kierownik_biura_przydzial::{
    KartyDzial, KartyKartotekaWynik, KartyRodzajKarty, KartyRodzajSprawy,
    KartyRozstrzygnieteOdParams, OdnotujSprawdzenieKartotekiParams, OdrzucKarteParams,
    PrzekazKarteCzlowiekowiParams, PrzydzielKarteParams, PrzyjmijKarteParams,
    ZapiszRegulePrzydzialuParams, ZastapRegulePrzydzialuParams,
};
use kierownik_biura_sprawozdania::{
    PrzekazSprawozdanieParams, WpisyRodzaj as RodzajSprawozdania, ZapiszSprawozdanieParams,
    ZmienSprawozdanieParams,
};

use crate::expect;
use crate::stanowisko::{DELETION, REPLACEMENT, first_row_copy};

/// Who signs the rules of assignment and approves the exceptions for the office owner, and the
/// signature of that approval.
const OFFICE_OWNER: &str = "tozsamosc-wlasciciela-biura";
const OFFICE_OWNER_SIGNATURE: &str = "podpis-kluczem-wlasciciela-biura";

/// When a human approves an assignment or a rejection.
const APPROVED_AT: &str = "2026-11-04T10:00:00Z";

fn new_room() -> MemoryRoom {
    kierownik_biura::open_in_memory().expect("open the room of the office manager")
}

/// A text value to bind in raw SQL.
fn text(value: &str) -> SqlValue {
    SqlValue::Text(value.to_string())
}

// Building blocks ---------------------------------------------------------------------------------

/// The default parameters of a rule the office owner signed: sales go to brokerage.
fn rule() -> ZapiszRegulePrzydzialuParams {
    ZapiszRegulePrzydzialuParams {
        rodzaj_zgloszenia: KartyRodzajSprawy::Sprzedaz,
        dzial: KartyDzial::Posrednictwo,
        od: "2026-11-01T00:00:00Z".to_string(),
        podpisal: OFFICE_OWNER.to_string(),
        podpis: OFFICE_OWNER_SIGNATURE.to_string(),
    }
}

fn add_rule(room: &impl MetaRoom, params: &ZapiszRegulePrzydzialuParams) {
    kierownik_biura_przydzial::zapisz_regule_przydzialu(room, params).expect("add the rule");
}

fn replace_rule(room: &MemoryRoom, rule_id: i64, zastapiona_od: &str) {
    let params = ZastapRegulePrzydzialuParams {
        zastapiona_od: zastapiona_od.to_string(),
        id: rule_id,
    };

    kierownik_biura_przydzial::zastap_regule_przydzialu(room, &params).expect("replace the rule");
}

/// The default parameters of a Reception card of a new request to sell a flat.
fn card() -> PrzyjmijKarteParams {
    PrzyjmijKarteParams {
        karta_odnosnik: "pokoj-recepcji/karta-1".to_string(),
        wplynela: "2026-11-04T09:30:00Z".to_string(),
        rodzaj_karty: KartyRodzajKarty::NoweZgloszenie,
        rodzaj_sprawy: Some(KartyRodzajSprawy::Sprzedaz),
        rodzaj_nieruchomosci: Some("mieszkanie".to_string()),
        miejscowosc: Some("Wrocław".to_string()),
        kanal: Some("telefon".to_string()),
        kontakt: Some("Anna Nowak, +48 600 000 000".to_string()),
    }
}

fn take_card(room: &impl MetaRoom, params: &PrzyjmijKarteParams) {
    kierownik_biura_przydzial::przyjmij_karte(room, params).expect("take the card");
}

/// The default parameters of a check of a card in the client register that found nobody.
fn register_check(card_id: i64) -> OdnotujSprawdzenieKartotekiParams {
    OdnotujSprawdzenieKartotekiParams {
        kartoteka_wynik: KartyKartotekaWynik::Brak,
        kartoteka_odnosnik: None,
        sprawdzono_kartoteke: "2026-11-04T09:40:00Z".to_string(),
        id: card_id,
    }
}

fn check_client_register(room: &impl MetaRoom, params: &OdnotujSprawdzenieKartotekiParams) {
    kierownik_biura_przydzial::odnotuj_sprawdzenie_kartoteki(room, params)
        .expect("record the check in the client register");
}

/// The default parameters of an assignment of a card to brokerage by the first rule, without a
/// human's approval.
fn assignment(card_id: i64) -> PrzydzielKarteParams {
    PrzydzielKarteParams {
        dzial: KartyDzial::Posrednictwo,
        regula_id: Some(1),
        rozstrzygnieto: "2026-11-04T10:00:00Z".to_string(),
        zatwierdzil: None,
        podpis: None,
        zatwierdzono: None,
        id: card_id,
    }
}

fn assign_card(room: &MemoryRoom, params: &PrzydzielKarteParams) {
    kierownik_biura_przydzial::przydziel_karte(room, params).expect("assign the card");
}

/// The default parameters of a brokerage report.
fn department_report() -> DopiszMeldunekDzialuParams {
    DopiszMeldunekDzialuParams {
        nadawca: Nadawca::Posrednictwo,
        tozsamosc: "tozsamosc-kierownika-posrednictwa".to_string(),
        otrzymano: "2026-11-04T16:00:00Z".to_string(),
        agenci_pracuja: Some(12),
        sprawy_zablokowane: Some(2),
        decyzje_czekaja: Some(3),
        najblizszy_termin: Some("2026-11-06".to_string()),
        twierdzenia: None,
    }
}

fn add_department_report(room: &impl MetaRoom, params: &DopiszMeldunekDzialuParams) {
    kierownik_biura_meldunki::dopisz_meldunek_dzialu(room, params)
        .expect("add the department's report");
}

/// The default parameters of a request for the broker's decision on the list.
fn position() -> PrzyjmijPozycjeParams {
    PrzyjmijPozycjeParams {
        prosba_odnosnik: "pokoj-teczki-1/decyzja-1".to_string(),
        dzial: PozycjeDzial::Posrednictwo,
        decyduje: "tozsamosc-posrednika".to_string(),
        opis: "Zgoda na obniżenie ceny ogłoszenia".to_string(),
        termin: None,
        przyjeto: "2026-11-04T10:00:00Z".to_string(),
    }
}

fn add_position(room: &impl MetaRoom, params: &PrzyjmijPozycjeParams) {
    kierownik_biura_decyzje::przyjmij_pozycje(room, params).expect("add the position");
}

fn remove_position(
    room: &MemoryRoom,
    position_id: i64,
    zdjeto: &str,
    powod_zdjecia: PozycjePowodZdjecia,
) {
    let params = ZdejmijPozycjeParams {
        zdjeto: zdjeto.to_string(),
        powod_zdjecia,
        id: position_id,
    };

    kierownik_biura_decyzje::zdejmij_pozycje(room, &params).expect("remove the position");
}

/// The default parameters of the broker's absence, signed by the broker, with a deputy.
fn absence() -> ZapiszNieobecnoscParams {
    ZapiszNieobecnoscParams {
        czlowiek: "tozsamosc-posrednika".to_string(),
        od_dnia: "2026-11-10".to_string(),
        do_dnia: "2026-11-14".to_string(),
        zastepca: "tozsamosc-wspolnika".to_string(),
        podpisal: "tozsamosc-posrednika".to_string(),
        podpisal_jako: WpisyPodpisalJako::Czlowiek,
        podpis: "podpis-kluczem-posrednika".to_string(),
        wiadomosc_odnosnik: "wiadomosc-urlop".to_string(),
        zapisano: "2026-11-04T11:00:00Z".to_string(),
    }
}

fn add_absence(room: &impl MetaRoom, params: &ZapiszNieobecnoscParams) {
    kierownik_biura_nieobecnosci::zapisz_nieobecnosc(room, params).expect("add the absence");
}

/// The parameters of the broker's absence with a short signature and message, for the writes a
/// rule refuses.
fn bare_absence() -> ZapiszNieobecnoscParams {
    ZapiszNieobecnoscParams {
        podpis: "p".to_string(),
        wiadomosc_odnosnik: "w".to_string(),
        ..absence()
    }
}

/// The default parameters of the broker's liability policy for 2027.
fn policy() -> ZapiszPoliseParams {
    ZapiszPoliseParams {
        rodzaj: RodzajPolisy::Posrednika,
        numer: "OC/2027/001".to_string(),
        ubezpieczyciel: "Towarzystwo Ubezpieczeń Przykładowe".to_string(),
        okres_od: "2027-01-01".to_string(),
        okres_do: "2027-12-31".to_string(),
        dokument_odnosnik: "dokumenty-biura/polisa-oc-2027".to_string(),
        zapisano: "2026-12-20T09:00:00Z".to_string(),
    }
}

fn add_policy(room: &impl MetaRoom, params: &ZapiszPoliseParams) {
    kierownik_biura_polisy::zapisz_polise(room, params).expect("add the policy");
}

fn notify_about_policy(room: &MemoryRoom, params: &OdnotujZawiadomienieOPolisieParams) {
    kierownik_biura_polisy::odnotuj_zawiadomienie_o_polisie(room, params)
        .expect("record the notice about the policy");
}

/// The default parameters of the monthly report for November.
fn report() -> ZapiszSprawozdanieParams {
    ZapiszSprawozdanieParams {
        rodzaj: RodzajSprawozdania::Miesieczne,
        okres_od: "2026-11-01".to_string(),
        okres_do: "2026-11-30".to_string(),
        sciezka: "sprawozdania/2026-11.typ".to_string(),
        skrot: "sha256-sprawozdanie-2026-11".to_string(),
        sporzadzono: "2026-12-01T09:00:00Z".to_string(),
    }
}

fn add_report(room: &impl MetaRoom, params: &ZapiszSprawozdanieParams) {
    kierownik_biura_sprawozdania::zapisz_sprawozdanie(room, params).expect("add the report");
}

/// The default parameters of a change of the monthly report for November.
fn report_change(report_id: i64) -> ZmienSprawozdanieParams {
    ZmienSprawozdanieParams {
        okres_od: "2026-11-01".to_string(),
        okres_do: "2026-11-30".to_string(),
        sciezka: "sprawozdania/2026-11.typ".to_string(),
        skrot: "sha256".to_string(),
        sporzadzono: "2026-12-01T09:00:00Z".to_string(),
        id: report_id,
    }
}

fn hand_over_report(room: &MemoryRoom, report_id: i64, przekazano: &str) {
    let params = PrzekazSprawozdanieParams {
        przekazano: przekazano.to_string(),
        id: report_id,
    };

    kierownik_biura_sprawozdania::przekaz_sprawozdanie(room, &params)
        .expect("hand over the report");
}

/// One row in every table of the manager's own kinds, for the checks of R9 and replacement.
fn full_memory(room: &impl MetaRoom) {
    add_rule(room, &rule());
    take_card(room, &card());
    add_department_report(room, &department_report());
    add_position(room, &position());
    add_absence(room, &absence());
    add_policy(room, &policy());
    add_report(room, &report());
}

// A whole day -------------------------------------------------------------------------------------

/// kierownik przydziela kartę według reguły, układa listę decyzji i pilnuje polisy
#[test]
fn whole_day() {
    let room = new_room();

    add_rule(&room, &rule());
    add_rule(
        &room,
        &ZapiszRegulePrzydzialuParams {
            rodzaj_zgloszenia: KartyRodzajSprawy::Zarzad,
            dzial: KartyDzial::Najem,
            ..rule()
        },
    );
    take_card(&room, &card());
    check_client_register(&room, &register_check(1));
    assign_card(&room, &assignment(1));

    add_department_report(&room, &department_report());
    kierownik_biura_meldunki::dopisz_meldunek_recepcji(
        &room,
        &DopiszMeldunekRecepcjiParams {
            tozsamosc: "tozsamosc-recepcji".to_string(),
            otrzymano: "2026-11-04T17:00:00Z".to_string(),
            wiadomosci_wplynely: Some(40),
            karty_wyslane: Some(3),
            zaklocenia: Some(1),
            twierdzenia: None,
        },
    )
    .expect("add the report of Reception");
    add_department_report(
        &room,
        &DopiszMeldunekDzialuParams {
            otrzymano: "2026-11-05T16:00:00Z".to_string(),
            agenci_pracuja: Some(13),
            ..department_report()
        },
    );

    add_position(&room, &position());
    add_position(
        &room,
        &PrzyjmijPozycjeParams {
            prosba_odnosnik: "pokoj-teczki-2/decyzja-1".to_string(),
            opis: "Zgoda na umowę z wyłącznością".to_string(),
            termin: Some("2026-11-05".to_string()),
            przyjeto: "2026-11-04T11:00:00Z".to_string(),
            ..position()
        },
    );
    kierownik_biura_decyzje::odnow_pozycje(
        &room,
        &OdnowPozycjeParams {
            opis: "Zgoda na obniżenie ceny ogłoszenia o 3%".to_string(),
            termin: None,
            odnowiono: "2026-11-04T15:00:00Z".to_string(),
            id: 1,
        },
    )
    .expect("renew the position");
    kierownik_biura_decyzje::odnotuj_wyslanie_pozycji(
        &room,
        &OdnotujWyslaniePozycjiParams {
            trafila_do_czlowieka: "2026-11-04T11:05:00Z".to_string(),
            id: 2,
        },
    )
    .expect("record the position as sent");

    add_absence(&room, &absence());
    add_policy(&room, &policy());
    notify_about_policy(
        &room,
        &OdnotujZawiadomienieOPolisieParams {
            zawiadomiono: "2026-12-20T10:00:00Z".to_string(),
            zawiadomiono_kogo: WpisyZawiadomionoKogo::KierownikPosrednictwa,
            id: 1,
        },
    );
    add_report(&room, &report());
    hand_over_report(&room, 1, "2026-12-01T12:00:00Z");

    let cards = room
        .execute(
            "SELECT stan, kontakt FROM kierownik_biura_przydzial_karty",
            &[],
        )
        .expect("read the card");
    let card_state = cards[0].get::<String>("stan").expect("a card has a state");
    let card_contact = cards[0]
        .get::<Option<String>>("kontakt")
        .expect("read the contact of the card");
    assert_eq!(card_state, "przydzielona");
    assert_eq!(card_contact, None);

    let latest = kierownik_biura_meldunki::ostatnie_meldunki(&room).expect("read the reports");
    let latest_senders: Vec<(WpisyNadawca, Option<i64>)> = latest
        .iter()
        .map(|row| (row.nadawca, row.agenci_pracuja))
        .collect();
    assert_eq!(
        latest_senders,
        [
            (WpisyNadawca::Posrednictwo, Some(13)),
            (WpisyNadawca::Recepcja, None),
        ]
    );

    let decisions = kierownik_biura_decyzje::lista_decyzji(
        &room,
        &ListaDecyzjiParams {
            decyduje: "tozsamosc-posrednika".to_string(),
            bliski_termin: "2026-11-06".to_string(),
        },
    )
    .expect("read the decision list");
    let decision_requests: Vec<&str> = decisions
        .iter()
        .map(|row| row.prosba_odnosnik.as_str())
        .collect();
    assert_eq!(
        decision_requests,
        ["pokoj-teczki-2/decyzja-1", "pokoj-teczki-1/decyzja-1"]
    );

    let absences = kierownik_biura_nieobecnosci::nieobecnosci_w_dniu(
        &room,
        &NieobecnosciWDniuParams {
            dzien: "2026-11-12".to_string(),
        },
    )
    .expect("read the absences of the day");
    let deputies: Vec<&str> = absences.iter().map(|row| row.zastepca.as_str()).collect();
    assert_eq!(deputies, ["tozsamosc-wspolnika"]);

    let policies_of_day = kierownik_biura_polisy::polisa_w_dniu(
        &room,
        &PolisaWDniuParams {
            rodzaj: RodzajPolisy::Posrednika,
            dzien: "2027-03-01".to_string(),
        },
    )
    .expect("read the policy of the day");
    assert_eq!(policies_of_day.len(), 1);

    let policies_to_remind = kierownik_biura_polisy::polisy_do_przypomnienia(
        &room,
        &PolisyDoPrzypomnieniaParams {
            do_dnia: "2028-01-30".to_string(),
        },
    )
    .expect("read the policies to remind of");
    let policy_ids: Vec<i64> = policies_to_remind.iter().map(|row| row.id).collect();
    assert_eq!(policy_ids, [1]);

    let cards_to_decide = kierownik_biura_przydzial::karty_do_rozstrzygniecia(&room)
        .expect("read the cards to decide");
    assert!(cards_to_decide.is_empty());
}

// Rules of assignment -----------------------------------------------------------------------------

/// reguła przydziału powstaje aktualna i jest jedna na rodzaj zgłoszenia
#[test]
fn one_rule_per_request_kind() {
    let room = new_room();
    add_rule(&room, &rule());

    expect::refused(
        kierownik_biura_przydzial::zapisz_regule_przydzialu(&room, &rule()),
        "ten rodzaj zgłoszenia ma już aktualną regułę",
    );
    expect::refused(
        room.execute(
            "INSERT INTO kierownik_biura_przydzial_reguly (rodzaj_zgloszenia, dzial, od, podpisal,
              podpis, zastapiona_od)
            VALUES ('kupno', 'posrednictwo', '2026-11-01', 'w', 'p', '2026-11-02')",
            &[],
        ),
        "reguła przydziału powstaje aktualna",
    );

    expect::refused(
        kierownik_biura_przydzial::zastap_regule_przydzialu(
            &room,
            &ZastapRegulePrzydzialuParams {
                zastapiona_od: "2026-10-01T00:00:00Z".to_string(),
                id: 1,
            },
        ),
        "zastapienie_nie_przed_poczatkiem",
    );
    replace_rule(&room, 1, "2026-12-01T00:00:00Z");
    add_rule(
        &room,
        &ZapiszRegulePrzydzialuParams {
            dzial: KartyDzial::Najem,
            od: "2026-12-01T00:00:00Z".to_string(),
            ..rule()
        },
    );

    let rules = kierownik_biura_przydzial::reguly_przydzialu(&room).expect("read the rules");
    let rule_departments: Vec<KartyDzial> = rules.iter().map(|row| row.dzial).collect();
    assert_eq!(rule_departments, [KartyDzial::Najem]);

    let history = kierownik_biura_przydzial::historia_regul(&room).expect("read the history");
    assert_eq!(history.len(), 2);
}

/// reguły przydziału się nie zmienia, a zastąpienie odnotowuje się raz
#[test]
fn rule_does_not_change() {
    let room = new_room();
    add_rule(&room, &rule());

    let rule = "reguły przydziału się nie zmienia";
    expect::refused(
        room.execute(
            "UPDATE kierownik_biura_przydzial_reguly SET dzial = 'najem'",
            &[],
        ),
        rule,
    );
    replace_rule(&room, 1, "2026-12-01T00:00:00Z");
    expect::refused(
        kierownik_biura_przydzial::zastap_regule_przydzialu(
            &room,
            &ZastapRegulePrzydzialuParams {
                zastapiona_od: "2026-12-02T00:00:00Z".to_string(),
                id: 1,
            },
        ),
        rule,
    );
}

// Cards -------------------------------------------------------------------------------------------

/// karta powstaje czekająca, a sprawdzenie w kartotece jest kompletne i raz
#[test]
fn card_starts_waiting_and_register_is_checked_once() {
    let room = new_room();
    take_card(&room, &card());

    expect::refused(
        room.execute(
            "INSERT INTO kierownik_biura_przydzial_karty (karta_odnosnik, wplynela, rodzaj_karty,
              stan, powod, rozstrzygnieto)
            VALUES ('karta-9', '2026-11-04', 'pismo_bez_sprawy', 'odrzucona', 'reklama',
              '2026-11-04')",
            &[],
        ),
        "karta powstaje jako czekająca",
    );

    let rule = "sprawdzenie_kartoteki_w_komplecie";
    expect::refused(
        kierownik_biura_przydzial::odnotuj_sprawdzenie_kartoteki(
            &room,
            &OdnotujSprawdzenieKartotekiParams {
                kartoteka_wynik: KartyKartotekaWynik::Jeden,
                ..register_check(1)
            },
        ),
        rule,
    );
    expect::refused(
        kierownik_biura_przydzial::odnotuj_sprawdzenie_kartoteki(
            &room,
            &OdnotujSprawdzenieKartotekiParams {
                kartoteka_odnosnik: Some("pokoj-kartoteki/klient-nowak".to_string()),
                ..register_check(1)
            },
        ),
        rule,
    );
    check_client_register(
        &room,
        &OdnotujSprawdzenieKartotekiParams {
            kartoteka_wynik: KartyKartotekaWynik::Jeden,
            kartoteka_odnosnik: Some("pokoj-kartoteki/klient-nowak".to_string()),
            ..register_check(1)
        },
    );
    expect::refused(
        kierownik_biura_przydzial::odnotuj_sprawdzenie_kartoteki(
            &room,
            &OdnotujSprawdzenieKartotekiParams {
                sprawdzono_kartoteke: "2026-11-04T09:50:00Z".to_string(),
                ..register_check(1)
            },
        ),
        "pól karty ani sprawdzenia w kartotece się nie zmienia",
    );
    expect::refused(
        room.execute(
            "UPDATE kierownik_biura_przydzial_karty SET miejscowosc = 'Opole' WHERE id = 1",
            &[],
        ),
        "pól karty ani sprawdzenia w kartotece się nie zmienia",
    );
}

/// przydzielone zgłoszenie ma dział i sprawdzenie w kartotece, a kontakt znika
#[test]
fn assigned_card_has_department_and_check() {
    let room = new_room();
    add_rule(&room, &rule());
    take_card(&room, &card());

    expect::refused(
        kierownik_biura_przydzial::przydziel_karte(&room, &assignment(1)),
        "przydzielone_zgloszenie_sprawdzone_w_kartotece",
    );
    check_client_register(&room, &register_check(1));

    // A writer outside the client can leave out the department that `przydziel_karte` requires.
    // The table refuses the assignment without one.
    let assignment_without_department = room.execute(
        "UPDATE kierownik_biura_przydzial_karty
        SET stan = 'przydzielona', dzial = :dzial, regula_id = :regula_id,
          rozstrzygnieto = :rozstrzygnieto, zatwierdzil = :zatwierdzil, podpis = :podpis,
          zatwierdzono = :zatwierdzono, kontakt = NULL
        WHERE id = :id",
        &[
            ("dzial", SqlValue::Null),
            ("regula_id", SqlValue::Null),
            ("rozstrzygnieto", text("2026-11-04T10:00:00Z")),
            ("zatwierdzil", text(OFFICE_OWNER)),
            ("podpis", text(OFFICE_OWNER_SIGNATURE)),
            ("zatwierdzono", text(APPROVED_AT)),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(assignment_without_department, "przydzielona_z_dzialem");

    // A writer outside the client can leave out the date of the decision that `przydziel_karte`
    // requires. The table refuses the assignment without one.
    let assignment_without_date = room.execute(
        "UPDATE kierownik_biura_przydzial_karty
        SET stan = 'przydzielona', dzial = :dzial, regula_id = :regula_id,
          rozstrzygnieto = :rozstrzygnieto, zatwierdzil = :zatwierdzil, podpis = :podpis,
          zatwierdzono = :zatwierdzono, kontakt = NULL
        WHERE id = :id",
        &[
            ("dzial", text("posrednictwo")),
            ("regula_id", SqlValue::Integer(1)),
            ("rozstrzygnieto", SqlValue::Null),
            ("zatwierdzil", SqlValue::Null),
            ("podpis", SqlValue::Null),
            ("zatwierdzono", SqlValue::Null),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(assignment_without_date, "rozstrzygnieta_z_data");

    expect::refused(
        kierownik_biura_przydzial::przydziel_karte(
            &room,
            &PrzydzielKarteParams {
                rozstrzygnieto: "2026-11-04T08:00:00Z".to_string(),
                ..assignment(1)
            },
        ),
        "rozstrzygniecie_nie_przed_wplywem",
    );
    expect::refused(
        room.execute(
            "UPDATE kierownik_biura_przydzial_karty
            SET stan = 'przydzielona', dzial = 'posrednictwo', regula_id = 1,
              rozstrzygnieto = '2026-11-04T10:00:00Z'
            WHERE id = 1",
            &[],
        ),
        "rozstrzygnieta_bez_kontaktu",
    );
    assign_card(&room, &assignment(1));
    expect::refused(
        kierownik_biura_przydzial::odrzuc_karte(
            &room,
            &OdrzucKarteParams {
                powod: "pomyłka".to_string(),
                rozstrzygnieto: "2026-11-05T10:00:00Z".to_string(),
                zatwierdzil: Some(OFFICE_OWNER.to_string()),
                podpis: Some(OFFICE_OWNER_SIGNATURE.to_string()),
                zatwierdzono: Some(APPROVED_AT.to_string()),
                id: 1,
            },
        ),
        "rozstrzygniętej karty wcale",
    );

    let decided_cards = kierownik_biura_przydzial::karty_rozstrzygniete_od(
        &room,
        &KartyRozstrzygnieteOdParams {
            od: "2026-11-04".to_string(),
        },
    )
    .expect("read the decided cards");
    assert_eq!(decided_cards.len(), 1);
}

/// przydział według reguły idzie do jej działu, dla jej rodzaju, gdy była aktualna
#[test]
fn assignment_follows_the_rule() {
    let room = new_room();
    add_rule(&room, &rule());
    replace_rule(&room, 1, "2026-11-10T00:00:00Z");
    add_rule(
        &room,
        &ZapiszRegulePrzydzialuParams {
            dzial: KartyDzial::Najem,
            od: "2026-11-10T00:00:00Z".to_string(),
            ..rule()
        },
    );
    take_card(&room, &card());
    take_card(
        &room,
        &PrzyjmijKarteParams {
            karta_odnosnik: "pokoj-recepcji/karta-2".to_string(),
            rodzaj_sprawy: Some(KartyRodzajSprawy::Kupno),
            ..card()
        },
    );
    check_client_register(&room, &register_check(1));
    check_client_register(&room, &register_check(2));

    let rule = "przydział według reguły idzie do jej działu";
    expect::refused(
        kierownik_biura_przydzial::przydziel_karte(
            &room,
            &PrzydzielKarteParams {
                dzial: KartyDzial::Najem,
                ..assignment(1)
            },
        ),
        rule,
    );
    expect::refused(
        kierownik_biura_przydzial::przydziel_karte(
            &room,
            &PrzydzielKarteParams {
                dzial: KartyDzial::Najem,
                regula_id: Some(2),
                ..assignment(1)
            },
        ),
        rule,
    );
    expect::refused(
        kierownik_biura_przydzial::przydziel_karte(&room, &assignment(2)),
        rule,
    );
    assign_card(&room, &assignment(1));
}

/// przydział poza regułami i odrzucenie zgłoszenia klienta wymagają zgody człowieka
#[test]
fn human_approves_exceptions() {
    let room = new_room();
    take_card(&room, &card());
    take_card(
        &room,
        &PrzyjmijKarteParams {
            karta_odnosnik: "pokoj-recepcji/karta-2".to_string(),
            ..card()
        },
    );
    take_card(
        &room,
        &PrzyjmijKarteParams {
            karta_odnosnik: "pokoj-recepcji/karta-3".to_string(),
            rodzaj_karty: KartyRodzajKarty::PismoBezSprawy,
            rodzaj_sprawy: None,
            kontakt: None,
            ..card()
        },
    );
    check_client_register(&room, &register_check(1));

    expect::refused(
        kierownik_biura_przydzial::przydziel_karte(
            &room,
            &PrzydzielKarteParams {
                dzial: KartyDzial::Transakcje,
                regula_id: None,
                ..assignment(1)
            },
        ),
        "przydzial_poza_regulami_za_zgoda_czlowieka",
    );
    kierownik_biura_przydzial::przekaz_karte_czlowiekowi(
        &room,
        &PrzekazKarteCzlowiekowiParams { id: 1 },
    )
    .expect("hand the card to a human");
    assign_card(
        &room,
        &PrzydzielKarteParams {
            dzial: KartyDzial::Transakcje,
            regula_id: None,
            zatwierdzil: Some(OFFICE_OWNER.to_string()),
            podpis: Some(OFFICE_OWNER_SIGNATURE.to_string()),
            zatwierdzono: Some(APPROVED_AT.to_string()),
            ..assignment(1)
        },
    );

    let unapproved_rejection = OdrzucKarteParams {
        powod: "Poza zakresem biura".to_string(),
        rozstrzygnieto: "2026-11-04T10:00:00Z".to_string(),
        zatwierdzil: None,
        podpis: None,
        zatwierdzono: None,
        id: 2,
    };
    expect::refused(
        kierownik_biura_przydzial::odrzuc_karte(&room, &unapproved_rejection),
        "odrzucenie_zgloszenia_za_zgoda_czlowieka",
    );

    // A writer outside the client can leave out the reason that `odrzuc_karte` requires. The table
    // refuses the rejection without one.
    let rejection_without_reason = room.execute(
        "UPDATE kierownik_biura_przydzial_karty
        SET stan = 'odrzucona', powod = :powod, rozstrzygnieto = :rozstrzygnieto,
          zatwierdzil = :zatwierdzil, podpis = :podpis, zatwierdzono = :zatwierdzono,
          kontakt = NULL
        WHERE id = :id",
        &[
            ("powod", SqlValue::Null),
            ("rozstrzygnieto", text("2026-11-04T10:00:00Z")),
            ("zatwierdzil", SqlValue::Null),
            ("podpis", SqlValue::Null),
            ("zatwierdzono", SqlValue::Null),
            ("id", SqlValue::Integer(3)),
        ],
    );
    expect::refused(rejection_without_reason, "odrzucona_z_powodem");
    kierownik_biura_przydzial::odrzuc_karte(
        &room,
        &OdrzucKarteParams {
            powod: "Reklama usług".to_string(),
            id: 3,
            ..unapproved_rejection.clone()
        },
    )
    .expect("reject the letter");

    expect::refused(
        kierownik_biura_przydzial::odrzuc_karte(
            &room,
            &OdrzucKarteParams {
                zatwierdzil: Some(OFFICE_OWNER.to_string()),
                ..unapproved_rejection.clone()
            },
        ),
        "zatwierdzenie_w_komplecie",
    );
    kierownik_biura_przydzial::odrzuc_karte(
        &room,
        &OdrzucKarteParams {
            zatwierdzil: Some(OFFICE_OWNER.to_string()),
            podpis: Some(OFFICE_OWNER_SIGNATURE.to_string()),
            zatwierdzono: Some(APPROVED_AT.to_string()),
            ..unapproved_rejection
        },
    )
    .expect("reject the request with a human's approval");
}

// Reports from the departments --------------------------------------------------------------------

/// meldunek ma pola działu albo pola Recepcji i tylko się go dopisuje
#[test]
fn report_fields_by_sender() {
    let room = new_room();
    add_department_report(&room, &department_report());

    expect::refused(
        kierownik_biura_meldunki::dopisz_meldunek_dzialu(
            &room,
            &DopiszMeldunekDzialuParams {
                nadawca: Nadawca::Najem,
                tozsamosc: "t".to_string(),
                otrzymano: "2026-11-04T16:00:00Z".to_string(),
                agenci_pracuja: None,
                sprawy_zablokowane: Some(0),
                decyzje_czekaja: Some(0),
                najblizszy_termin: None,
                twierdzenia: None,
            },
        ),
        "pola_meldunku_dzialu",
    );
    expect::refused(
        room.execute(
            "INSERT INTO kierownik_biura_meldunki_wpisy (nadawca, tozsamosc, otrzymano,
              najblizszy_termin, wiadomosci_wplynely, karty_wyslane, zaklocenia)
            VALUES ('recepcja', 't', '2026-11-04', '2026-11-06', 1, 0, 0)",
            &[],
        ),
        "pola_meldunku_dzialu",
    );
    expect::refused(
        room.execute(
            "INSERT INTO kierownik_biura_meldunki_wpisy (nadawca, tozsamosc, otrzymano,
              agenci_pracuja, sprawy_zablokowane, decyzje_czekaja, zaklocenia)
            VALUES ('najem', 't', '2026-11-04', 1, 0, 0, 2)",
            &[],
        ),
        "pola_meldunku_recepcji",
    );
    expect::refused(
        room.execute(
            "UPDATE kierownik_biura_meldunki_wpisy SET agenci_pracuja = 1 WHERE id = 1",
            &[],
        ),
        "meldunki tylko się dopisuje",
    );

    let reports = kierownik_biura_meldunki::meldunki_od(
        &room,
        &MeldunkiOdParams {
            od: "2026-11-01".to_string(),
        },
    )
    .expect("read the reports");
    assert_eq!(reports.len(), 1);
}

// Decision lists ----------------------------------------------------------------------------------

/// prośba jest na liście raz; ponowna odnawia pozycję, a po zdjęciu wraca nowa
#[test]
fn request_once_on_the_list() {
    let room = new_room();
    add_position(&room, &position());

    let short_position = PrzyjmijPozycjeParams {
        opis: "Zgoda".to_string(),
        ..position()
    };
    expect::refused(
        kierownik_biura_decyzje::przyjmij_pozycje(&room, &short_position),
        "ta prośba jest już na liście",
    );
    expect::refused(
        kierownik_biura_decyzje::odnow_pozycje(
            &room,
            &OdnowPozycjeParams {
                opis: "Nowy opis".to_string(),
                termin: None,
                odnowiono: "2026-11-03T10:00:00Z".to_string(),
                id: 1,
            },
        ),
        "odnowienie_nie_przed_przyjeciem",
    );
    kierownik_biura_decyzje::odnow_pozycje(
        &room,
        &OdnowPozycjeParams {
            opis: "Nowy opis".to_string(),
            termin: None,
            odnowiono: "2026-11-05T10:00:00Z".to_string(),
            id: 1,
        },
    )
    .expect("renew the position");

    // A writer outside the client can leave out the reason that `zdejmij_pozycje` requires. The
    // table refuses the removal without one.
    let removal_without_reason = room.execute(
        "UPDATE kierownik_biura_decyzje_pozycje
        SET zdjeto = :zdjeto, powod_zdjecia = :powod_zdjecia
        WHERE id = :id",
        &[
            ("zdjeto", text("2026-11-06T10:00:00Z")),
            ("powod_zdjecia", SqlValue::Null),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(removal_without_reason, "zdjecie_z_powodem");
    expect::refused(
        kierownik_biura_decyzje::zdejmij_pozycje(
            &room,
            &ZdejmijPozycjeParams {
                zdjeto: "2026-11-01T10:00:00Z".to_string(),
                powod_zdjecia: PozycjePowodZdjecia::Zbedna,
                id: 1,
            },
        ),
        "zdjecie_nie_przed_przyjeciem",
    );
    remove_position(
        &room,
        1,
        "2026-11-06T10:00:00Z",
        PozycjePowodZdjecia::Rozstrzygnieta,
    );
    add_position(
        &room,
        &PrzyjmijPozycjeParams {
            przyjeto: "2026-11-07T10:00:00Z".to_string(),
            ..short_position
        },
    );

    let removed = kierownik_biura_decyzje::pozycje_zdjete_od(
        &room,
        &PozycjeZdjeteOdParams {
            od: "2026-11-01".to_string(),
        },
    )
    .expect("read the removed positions");
    assert_eq!(removed.len(), 1);

    let waiting = kierownik_biura_decyzje::pozycje_czekajace_od(
        &room,
        &PozycjeCzekajaceOdParams {
            przed: "2026-11-08".to_string(),
        },
    )
    .expect("read the waiting positions");
    let waiting_ids: Vec<i64> = waiting.iter().map(|row| row.id).collect();
    assert_eq!(waiting_ids, [2]);
}

/// pozycja powstaje niewysłana, a jej prośby, działu ani człowieka się nie zmienia
#[test]
fn position_does_not_change() {
    let room = new_room();
    add_position(&room, &position());

    expect::refused(
        room.execute(
            "INSERT INTO kierownik_biura_decyzje_pozycje (prosba_odnosnik, dzial, decyduje, opis,
              przyjeto, trafila_do_czlowieka)
            VALUES ('pokoj-teczki-2/decyzja-1', 'najem', 'd', 'o', '2026-11-04', '2026-11-04')",
            &[],
        ),
        "pozycja powstaje na liście",
    );

    let rule = "prośby, działu, człowieka ani przyjęcia pozycji się nie zmienia";
    expect::refused(
        room.execute(
            "UPDATE kierownik_biura_decyzje_pozycje SET decyduje = 'inny' WHERE id = 1",
            &[],
        ),
        rule,
    );
    expect::refused(
        kierownik_biura_decyzje::odnotuj_wyslanie_pozycji(
            &room,
            &OdnotujWyslaniePozycjiParams {
                trafila_do_czlowieka: "2026-11-03T10:00:00Z".to_string(),
                id: 1,
            },
        ),
        "do_czlowieka_nie_przed_przyjeciem",
    );
    kierownik_biura_decyzje::odnotuj_wyslanie_pozycji(
        &room,
        &OdnotujWyslaniePozycjiParams {
            trafila_do_czlowieka: "2026-11-04T10:05:00Z".to_string(),
            id: 1,
        },
    )
    .expect("record the position as sent");
    expect::refused(
        kierownik_biura_decyzje::odnotuj_wyslanie_pozycji(
            &room,
            &OdnotujWyslaniePozycjiParams {
                trafila_do_czlowieka: "2026-11-04T15:00:00Z".to_string(),
                id: 1,
            },
        ),
        rule,
    );
    kierownik_biura_decyzje::odnotuj_osobne_zgloszenie(
        &room,
        &OdnotujOsobneZgloszenieParams {
            zgloszono_osobno: "2026-11-07T09:00:00Z".to_string(),
            id: 1,
        },
    )
    .expect("record the separate notice");
    expect::refused(
        kierownik_biura_decyzje::odnotuj_osobne_zgloszenie(
            &room,
            &OdnotujOsobneZgloszenieParams {
                zgloszono_osobno: "2026-11-08T09:00:00Z".to_string(),
                id: 1,
            },
        ),
        rule,
    );

    remove_position(
        &room,
        1,
        "2026-11-08T10:00:00Z",
        PozycjePowodZdjecia::Zbedna,
    );
    expect::refused(
        kierownik_biura_decyzje::odnow_pozycje(
            &room,
            &OdnowPozycjeParams {
                opis: "Znowu".to_string(),
                termin: None,
                odnowiono: "2026-11-09T10:00:00Z".to_string(),
                id: 1,
            },
        ),
        rule,
    );
}

// Absences ----------------------------------------------------------------------------------------

/// nieobecność ma okres, innego zastępcę i podpis człowieka albo właściciela biura
#[test]
fn absence_is_signed() {
    let room = new_room();
    add_absence(&room, &absence());
    add_absence(
        &room,
        &ZapiszNieobecnoscParams {
            czlowiek: "tozsamosc-pracownika".to_string(),
            podpisal: OFFICE_OWNER.to_string(),
            podpisal_jako: WpisyPodpisalJako::WlascicielBiura,
            podpis: OFFICE_OWNER_SIGNATURE.to_string(),
            ..absence()
        },
    );

    expect::refused(
        kierownik_biura_nieobecnosci::zapisz_nieobecnosc(
            &room,
            &ZapiszNieobecnoscParams {
                od_dnia: "2026-11-15".to_string(),
                ..bare_absence()
            },
        ),
        "nieobecnosc_od_przed_do",
    );
    expect::refused(
        kierownik_biura_nieobecnosci::zapisz_nieobecnosc(
            &room,
            &ZapiszNieobecnoscParams {
                zastepca: "tozsamosc-posrednika".to_string(),
                ..bare_absence()
            },
        ),
        "zastepca_to_ktos_inny",
    );
    expect::refused(
        kierownik_biura_nieobecnosci::zapisz_nieobecnosc(
            &room,
            &ZapiszNieobecnoscParams {
                podpisal: "tozsamosc-kogos-innego".to_string(),
                ..bare_absence()
            },
        ),
        "podpis_czlowieka_ktorego_dotyczy",
    );
    expect::refused(
        room.execute(
            "INSERT INTO kierownik_biura_nieobecnosci_wpisy (czlowiek, od_dnia, do_dnia, zastepca,
              podpisal, podpisal_jako, podpis, wiadomosc_odnosnik, zapisano, odwolal,
              odwolanie_podpis, odwolano)
            VALUES ('c', '2026-11-10', '2026-11-11', 'z', 'c', 'czlowiek', 'p', 'w', '2026-11-04',
              'c', 'p', '2026-11-05')",
            &[],
        ),
        "nieobecność powstaje nieodwołana",
    );

    let absences = kierownik_biura_nieobecnosci::nieobecnosci_od(
        &room,
        &NieobecnosciOdParams {
            od: "2026-11-01".to_string(),
        },
    )
    .expect("read the absences");
    assert_eq!(absences.len(), 2);
}

/// nieobecność odwołuje podpisem człowiek albo właściciel biura, raz, i się jej nie zmienia
#[test]
fn absence_is_withdrawn_once_by_its_signer() {
    let room = new_room();
    add_absence(&room, &absence());

    expect::refused(
        room.execute(
            "UPDATE kierownik_biura_nieobecnosci_wpisy SET zastepca = 'ktos-inny' WHERE id = 1",
            &[],
        ),
        "nieobecności się nie zmienia",
    );

    // A writer outside the client can leave out the signature that `odwolaj_nieobecnosc` requires.
    // The table refuses the withdrawal without one.
    let withdrawal_without_signature = room.execute(
        "UPDATE kierownik_biura_nieobecnosci_wpisy
        SET odwolal = :odwolal, odwolanie_podpis = :odwolanie_podpis, odwolano = :odwolano
        WHERE id = :id",
        &[
            ("odwolal", text("tozsamosc-posrednika")),
            ("odwolanie_podpis", SqlValue::Null),
            ("odwolano", text("2026-11-05T09:00:00Z")),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(withdrawal_without_signature, "odwolanie_z_podpisem");
    expect::refused(
        kierownik_biura_nieobecnosci::odwolaj_nieobecnosc(
            &room,
            &OdwolajNieobecnoscParams {
                odwolal: "tozsamosc-obcego".to_string(),
                odwolanie_podpis: "p".to_string(),
                odwolano: "2026-11-05T09:00:00Z".to_string(),
                id: 1,
            },
        ),
        "nieobecność odwołuje człowiek, którego dotyczy",
    );
    kierownik_biura_nieobecnosci::odwolaj_nieobecnosc(
        &room,
        &OdwolajNieobecnoscParams {
            odwolal: "tozsamosc-posrednika".to_string(),
            odwolanie_podpis: "podpis-kluczem-posrednika".to_string(),
            odwolano: "2026-11-05T09:00:00Z".to_string(),
            id: 1,
        },
    )
    .expect("withdraw the absence");
    expect::refused(
        kierownik_biura_nieobecnosci::odwolaj_nieobecnosc(
            &room,
            &OdwolajNieobecnoscParams {
                odwolal: "tozsamosc-posrednika".to_string(),
                odwolanie_podpis: "p".to_string(),
                odwolano: "2026-11-06T09:00:00Z".to_string(),
                id: 1,
            },
        ),
        "nieobecności się nie zmienia",
    );

    let absences = kierownik_biura_nieobecnosci::nieobecnosci_w_dniu(
        &room,
        &NieobecnosciWDniuParams {
            dzien: "2026-11-12".to_string(),
        },
    )
    .expect("read the absences of the day");
    assert!(absences.is_empty());
}

// Policies ----------------------------------------------------------------------------------------

/// o polisie zawiadamia się właściwego kierownika, raz, a jej danych się nie zmienia
#[test]
fn policy_notice_to_the_right_head() {
    let room = new_room();
    add_policy(&room, &policy());
    add_policy(
        &room,
        &ZapiszPoliseParams {
            rodzaj: RodzajPolisy::Zarzadcy,
            numer: "OC-Z/2027/001".to_string(),
            ..policy()
        },
    );

    expect::refused(
        kierownik_biura_polisy::zapisz_polise(
            &room,
            &ZapiszPoliseParams {
                numer: "OC/2027/002".to_string(),
                ubezpieczyciel: "U".to_string(),
                okres_do: "2026-12-31".to_string(),
                dokument_odnosnik: "d".to_string(),
                ..policy()
            },
        ),
        "okres_od_przed_do",
    );
    expect::refused(
        room.execute(
            "INSERT INTO kierownik_biura_polisy_wpisy (rodzaj, numer, ubezpieczyciel, okres_od,
              okres_do, dokument_odnosnik, zapisano, przypomniano)
            VALUES ('posrednika', 'X', 'U', '2027-01-01', '2027-12-31', 'd', '2026-12-20',
              '2026-12-21')",
            &[],
        ),
        "polisa powstaje bez przypomnienia i zawiadomienia",
    );

    let rule = "zawiadomiony_kierownik_wlasciwy_dla_polisy";
    expect::refused(
        kierownik_biura_polisy::odnotuj_zawiadomienie_o_polisie(
            &room,
            &OdnotujZawiadomienieOPolisieParams {
                zawiadomiono: "2026-12-20T10:00:00Z".to_string(),
                zawiadomiono_kogo: WpisyZawiadomionoKogo::KierownikNajmu,
                id: 1,
            },
        ),
        rule,
    );
    expect::refused(
        kierownik_biura_polisy::odnotuj_zawiadomienie_o_polisie(
            &room,
            &OdnotujZawiadomienieOPolisieParams {
                zawiadomiono: "2026-12-20T10:00:00Z".to_string(),
                zawiadomiono_kogo: WpisyZawiadomionoKogo::KierownikPosrednictwa,
                id: 2,
            },
        ),
        rule,
    );

    // A writer outside the client can leave out the head that `odnotuj_zawiadomienie_o_polisie`
    // requires. The table refuses the notice without one.
    let notice_without_head = room.execute(
        "UPDATE kierownik_biura_polisy_wpisy
        SET zawiadomiono = :zawiadomiono, zawiadomiono_kogo = :zawiadomiono_kogo
        WHERE id = :id",
        &[
            ("zawiadomiono", text("2026-12-20T10:00:00Z")),
            ("zawiadomiono_kogo", SqlValue::Null),
            ("id", SqlValue::Integer(2)),
        ],
    );
    expect::refused(notice_without_head, "zawiadomienie_w_komplecie");
    notify_about_policy(
        &room,
        &OdnotujZawiadomienieOPolisieParams {
            zawiadomiono: "2026-12-20T10:00:00Z".to_string(),
            zawiadomiono_kogo: WpisyZawiadomionoKogo::KierownikNajmu,
            id: 2,
        },
    );

    let rule = "danych polisy się nie zmienia";
    expect::refused(
        kierownik_biura_polisy::odnotuj_zawiadomienie_o_polisie(
            &room,
            &OdnotujZawiadomienieOPolisieParams {
                zawiadomiono: "2026-12-21T10:00:00Z".to_string(),
                zawiadomiono_kogo: WpisyZawiadomionoKogo::KierownikNajmu,
                id: 2,
            },
        ),
        rule,
    );
    expect::refused(
        room.execute(
            "UPDATE kierownik_biura_polisy_wpisy SET okres_do = '2028-01-31' WHERE id = 1",
            &[],
        ),
        rule,
    );
    kierownik_biura_polisy::odnotuj_przypomnienie_o_polisie(
        &room,
        &OdnotujPrzypomnienieOPolisieParams {
            przypomniano: "2027-12-01T09:00:00Z".to_string(),
            id: 1,
        },
    )
    .expect("record the reminder about the policy");
    expect::refused(
        kierownik_biura_polisy::odnotuj_przypomnienie_o_polisie(
            &room,
            &OdnotujPrzypomnienieOPolisieParams {
                przypomniano: "2027-12-02T09:00:00Z".to_string(),
                id: 1,
            },
        ),
        rule,
    );

    let policies = kierownik_biura_polisy::polisy(&room).expect("read the policies");
    assert_eq!(policies.len(), 2);
}

// Reports to the office owner ---------------------------------------------------------------------

/// sprawozdanie powstaje robocze, a przekazanego się nie zmienia
#[test]
fn handed_over_report_does_not_change() {
    let room = new_room();
    add_report(&room, &report());

    expect::refused(
        kierownik_biura_sprawozdania::zapisz_sprawozdanie(
            &room,
            &ZapiszSprawozdanieParams {
                okres_od: "2026-12-01".to_string(),
                skrot: "sha256".to_string(),
                ..report()
            },
        ),
        "okres_od_nie_po_do",
    );
    expect::refused(
        room.execute(
            "INSERT INTO kierownik_biura_sprawozdania_wpisy (rodzaj, okres_od, okres_do, sciezka,
              skrot, sporzadzono, przekazano)
            VALUES ('na_zadanie', '2026-11-01', '2026-11-15', 's', 'k', '2026-11-16',
              '2026-11-16')",
            &[],
        ),
        "sprawozdanie powstaje robocze",
    );
    kierownik_biura_sprawozdania::zmien_sprawozdanie(
        &room,
        &ZmienSprawozdanieParams {
            sciezka: "sprawozdania/2026-11-v2.typ".to_string(),
            skrot: "sha256-v2".to_string(),
            ..report_change(1)
        },
    )
    .expect("change the report");
    expect::refused(
        kierownik_biura_sprawozdania::przekaz_sprawozdanie(
            &room,
            &PrzekazSprawozdanieParams {
                przekazano: "2026-11-30T09:00:00Z".to_string(),
                id: 1,
            },
        ),
        "przekazanie_nie_przed_sporzadzeniem",
    );
    hand_over_report(&room, 1, "2026-12-01T12:00:00Z");
    expect::refused(
        kierownik_biura_sprawozdania::zmien_sprawozdanie(
            &room,
            &ZmienSprawozdanieParams {
                skrot: "sha256-v3".to_string(),
                ..report_change(1)
            },
        ),
        "przekazanego sprawozdania się nie zmienia",
    );

    let reports = kierownik_biura_sprawozdania::sprawozdania(&room).expect("read the reports");
    assert_eq!(reports[0].skrot, "sha256-v2");
}

// R9 and replacement ------------------------------------------------------------------------------

/// The tables of the manager's own kinds in the room.
fn own_tables_of(room: &MemoryRoom) -> Vec<String> {
    let rows = room
        .execute(
            r"SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'kierownik\_biura\_%' ESCAPE '\'",
            &[],
        )
        .expect("read the tables");

    rows.iter()
        .map(|row| row.get::<String>("name").expect("a table has a name"))
        .collect()
}

/// The names of the table's columns, in the order of the schema.
fn columns_of(room: &MemoryRoom, table: &str) -> Vec<String> {
    let rows = room
        .execute(&format!("PRAGMA table_info({table})"), &[])
        .expect("read the columns of the table");

    rows.iter()
        .map(|row| row.get::<String>("name").expect("a column has a name"))
        .collect()
}

/// R9: wierszy pamięci się nie usuwa ani nie zastępuje przez REPLACE albo upsert
#[test]
fn no_deletion_or_replacement() {
    let room = new_room();
    full_memory(&room);

    let tables = own_tables_of(&room);
    assert_eq!(tables.len(), 7, "{tables:?}");

    for table in &tables {
        let row_count = room
            .value(&format!("SELECT count(*) FROM {table}"), &[])
            .expect("count the rows");
        let row_count = i64::from_sql_value(&row_count).expect("the count is a number");
        assert!(row_count > 0, "{table} has no row");

        let replace = format!("INSERT OR REPLACE {}", first_row_copy(&room, table));
        expect::refused(room.execute(&replace, &[]), REPLACEMENT);

        let upsert = format!(
            "INSERT {} ON CONFLICT (id) DO UPDATE SET id = excluded.id",
            first_row_copy(&room, table)
        );
        expect::refused(room.execute(&upsert, &[]), REPLACEMENT);

        let delete = format!("DELETE FROM {table} WHERE rowid = (SELECT min(rowid) FROM {table})");
        expect::refused(room.execute(&delete, &[]), DELETION);
    }
}

/// REPLACE nie usuwa aktualnej reguły, karty ani niezdjętej pozycji przez klucz UNIQUE
#[test]
fn no_replacement_through_unique_keys() {
    let room = new_room();
    full_memory(&room);

    let unique_keys = [
        (
            "kierownik_biura_przydzial_karty",
            "karta_odnosnik",
            REPLACEMENT,
        ),
        (
            "kierownik_biura_przydzial_reguly",
            "rodzaj_zgloszenia",
            "ten rodzaj zgłoszenia ma już aktualną regułę",
        ),
        (
            "kierownik_biura_decyzje_pozycje",
            "prosba_odnosnik",
            "ta prośba jest już na liście",
        ),
    ];
    for (table, key, refusal) in unique_keys {
        let columns = columns_of(&room, table);
        let selected: Vec<&str> = columns
            .iter()
            .map(|name| if name == "id" { "99" } else { name.as_str() })
            .collect();
        let columns = columns.join(", ");
        let selected = selected.join(", ");

        let replace =
            format!("INSERT OR REPLACE INTO {table} ({columns}) SELECT {selected} FROM {table}");
        expect::refused(room.execute(&replace, &[]), refusal);

        if refusal == REPLACEMENT {
            // `WHERE true` keeps SQLite from reading ON CONFLICT as a join constraint.
            let upsert = format!(
                "INSERT INTO {table} ({columns}) SELECT {selected} FROM {table} \
                WHERE true ON CONFLICT ({key}) DO UPDATE SET id = excluded.id"
            );
            expect::refused(room.execute(&upsert, &[]), refusal);
        }
    }
}
