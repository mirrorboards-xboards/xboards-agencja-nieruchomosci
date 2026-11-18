//! Scenarios of the brokerage head's memory room
//! (spec/positions/posrednictwo/kierownik_posrednictwa).
//!
//! The room has the head's own kinds, three kinds of the agent memory and all seven kinds of the
//! department memory; the rules of the shared kinds are checked in stanowisko.rs and
//! kierownik_dzialu.rs. Each scenario starts from an empty room and checks one rule the database
//! keeps: a write the rule allows and a write it refuses. The first scenario follows a card from
//! the office manager to a broker's new case file with its keeper, and an offer and a search that
//! meet in one transaction.

// The building blocks of kierownik_dzialu.rs take the types of the shared catalog's room.
use xboards_agencja_nieruchomosci::positions::kierownik_dzialu::{
    kierownik_dzialu_agenci as shared_agenci, kierownik_dzialu_spis as shared_spis,
};
use xboards_agencja_nieruchomosci::positions::posrednictwo::kierownik_posrednictwa::{
    self, kierownik_posrednictwa_obie_strony, kierownik_posrednictwa_przydzial,
};
use xboards_agencja_nieruchomosci::xmeta_room::{FromSqlValue, MemoryRoom, SqlValue};

use kierownik_posrednictwa_obie_strony::{
    OcenUmowyParyParams, OdnotujPrzekazanieParyParams, OdnotujTeczkeTransakcjiParams,
    ParyUmowyPozwalaja, ZapiszPareParams,
};
use kierownik_posrednictwa_przydzial::{
    CofnijPropozycjeParams, HistoriaRegulPosrednikaParams, KartyStan, ObciazeniePosrednikowRow,
    OdnotujZalozenieTeczkiParams, OdrzucKarteParams, PrzyjmijKarteParams,
    ZapiszRegulePosrednikaParams, ZaproponujPosrednikaParams, ZastapRegulePosrednikaParams,
};

use crate::expect;
use crate::kierownik_dzialu::{add_case_to_list, agent, appoint_agent, case_in_list};
use crate::stanowisko::{DELETION, REPLACEMENT, first_row_copy};

/// Who approves for the owner, the signature of the approval and when the owner approves.
const OWNER: &str = "tozsamosc-wlasciciela";
const OWNER_SIGNATURE: &str = "podpis-kluczem-wlasciciela";
const APPROVED_AT: &str = "2026-11-04T10:20:00Z";

fn new_room() -> MemoryRoom {
    kierownik_posrednictwa::open_in_memory().expect("open the room of the Head of brokerage")
}

/// A text value to bind in raw SQL.
fn text(value: &str) -> SqlValue {
    SqlValue::Text(value.to_string())
}

// Building blocks ---------------------------------------------------------------------------------

/// The default parameters of a broker's signed rule: flats in Krzyki, up to 20 cases.
fn broker_rule() -> ZapiszRegulePosrednikaParams {
    ZapiszRegulePosrednikaParams {
        posrednik: "tozsamosc-posrednika".to_string(),
        obszar: "Wrocław, Krzyki".to_string(),
        rodzaje_nieruchomosci: "mieszkania".to_string(),
        limit_spraw: 20,
        przyjmuje_nowe: true,
        od: "2026-11-01T00:00:00Z".to_string(),
        wiadomosc_odnosnik: "wiadomosc-reguly-posrednika".to_string(),
        podpis: "podpis-kluczem-posrednika".to_string(),
    }
}

fn add_broker_rule(room: &MemoryRoom, params: &ZapiszRegulePosrednikaParams) {
    kierownik_posrednictwa_przydzial::zapisz_regule_posrednika(room, params)
        .expect("add the broker's rule");
}

/// The parameters of replacing the first broker rule from `zastapione_od`.
fn rule_replacement(zastapione_od: &str) -> ZastapRegulePosrednikaParams {
    ZastapRegulePosrednikaParams {
        zastapione_od: zastapione_od.to_string(),
        id: 1,
    }
}

fn replace_broker_rule(room: &MemoryRoom, zastapione_od: &str) {
    kierownik_posrednictwa_przydzial::zastap_regule_posrednika(
        room,
        &rule_replacement(zastapione_od),
    )
    .expect("replace the broker's rule");
}

/// The default parameters of a card from the office manager, with the client's contact.
fn card() -> PrzyjmijKarteParams {
    PrzyjmijKarteParams {
        karta_odnosnik: "pokoj-kierownika-biura/karta-1".to_string(),
        otrzymano: "2026-11-04T10:00:00Z".to_string(),
        kontakt: Some("Anna Nowak, +48 600 000 000".to_string()),
    }
}

fn take_card(room: &MemoryRoom, params: &PrzyjmijKarteParams) {
    kierownik_posrednictwa_przydzial::przyjmij_karte(room, params).expect("take the card");
}

/// The default parameters of proposing the broker of the first rule for the first card.
fn proposal() -> ZaproponujPosrednikaParams {
    ZaproponujPosrednikaParams {
        posrednik: "tozsamosc-posrednika".to_string(),
        regula_id: Some(1),
        zaproponowano: "2026-11-04T10:30:00Z".to_string(),
        zatwierdzil: None,
        podpis: None,
        zatwierdzono: None,
        id: 1,
    }
}

/// The parameters of proposing a broker outside the rules, with the owner's approval.
fn proposal_approved_by_owner() -> ZaproponujPosrednikaParams {
    ZaproponujPosrednikaParams {
        regula_id: None,
        zatwierdzil: Some(OWNER.to_string()),
        podpis: Some(OWNER_SIGNATURE.to_string()),
        zatwierdzono: Some(APPROVED_AT.to_string()),
        ..proposal()
    }
}

fn propose_broker(room: &MemoryRoom, params: &ZaproponujPosrednikaParams) {
    kierownik_posrednictwa_przydzial::zaproponuj_posrednika(room, params)
        .expect("propose the broker");
}

/// The parameters of recording the case file opened for the first card.
fn case_file_opened() -> OdnotujZalozenieTeczkiParams {
    OdnotujZalozenieTeczkiParams {
        teczka_odnosnik: "pokoj-teczki-oferty-1".to_string(),
        rozstrzygnieto: "2026-11-04T12:00:00Z".to_string(),
        id: 1,
    }
}

fn record_case_file_opened(room: &MemoryRoom, params: &OdnotujZalozenieTeczkiParams) {
    kierownik_posrednictwa_przydzial::odnotuj_zalozenie_teczki(room, params)
        .expect("record the case file opened for the card");
}

/// The default parameters of an offer and a search that meet.
fn pair() -> ZapiszPareParams {
    ZapiszPareParams {
        teczka_oferty_odnosnik: "pokoj-teczki-oferty-1".to_string(),
        teczka_poszukiwania_odnosnik: "pokoj-teczki-poszukiwania-1".to_string(),
        stwierdzono: "2026-11-10T09:00:00Z".to_string(),
    }
}

fn add_pair(room: &MemoryRoom, params: &ZapiszPareParams) {
    kierownik_posrednictwa_obie_strony::zapisz_pare(room, params).expect("add the pair");
}

/// The assessment that both contracts of the pair `pair_id` allow serving both sides.
fn both_contracts_allow(pair_id: i64) -> OcenUmowyParyParams {
    OcenUmowyParyParams {
        umowy_pozwalaja: ParyUmowyPozwalaja::Tak,
        umowa_oferty_odnosnik: Some("pokoj-teczki-oferty-1/umowa".to_string()),
        umowa_poszukiwania_odnosnik: Some("pokoj-teczki-poszukiwania-1/umowa".to_string()),
        id: pair_id,
    }
}

fn assess_contracts(room: &MemoryRoom, params: &OcenUmowyParyParams) {
    kierownik_posrednictwa_obie_strony::ocen_umowy_pary(room, params)
        .expect("assess the contracts of the pair");
}

/// The parameters of handing the offer of the pair `pair_id` over, without a human's approval.
fn pair_handover(pair_id: i64, przekazano: &str) -> OdnotujPrzekazanieParyParams {
    OdnotujPrzekazanieParyParams {
        przekazano: przekazano.to_string(),
        zatwierdzil: None,
        podpis: None,
        zatwierdzono: None,
        id: pair_id,
    }
}

/// The parameters of handing the offer of the pair `pair_id` over with the owner's approval.
fn pair_handover_approved_by_owner(pair_id: i64, przekazano: &str) -> OdnotujPrzekazanieParyParams {
    OdnotujPrzekazanieParyParams {
        zatwierdzil: Some(OWNER.to_string()),
        podpis: Some(OWNER_SIGNATURE.to_string()),
        zatwierdzono: Some(APPROVED_AT.to_string()),
        ..pair_handover(pair_id, przekazano)
    }
}

fn hand_pair_over(room: &MemoryRoom, params: &OdnotujPrzekazanieParyParams) {
    kierownik_posrednictwa_obie_strony::odnotuj_przekazanie_pary(room, params)
        .expect("hand the offer of the pair over");
}

/// The parameters of recording the transaction's case file of the pair `pair_id`.
fn transaction_file(pair_id: i64, transakcja_odnosnik: &str) -> OdnotujTeczkeTransakcjiParams {
    OdnotujTeczkeTransakcjiParams {
        transakcja_odnosnik: transakcja_odnosnik.to_string(),
        id: pair_id,
    }
}

fn record_transaction_file(room: &MemoryRoom, params: &OdnotujTeczkeTransakcjiParams) {
    kierownik_posrednictwa_obie_strony::odnotuj_teczke_transakcji(room, params)
        .expect("record the transaction's case file");
}

/// One row in every table of the head's own kinds, for the checks of R9 and replacement.
fn full_memory(room: &MemoryRoom) {
    add_broker_rule(room, &broker_rule());
    take_card(room, &card());
    add_pair(room, &pair());
}

// A whole day -------------------------------------------------------------------------------------

/// kierownik proponuje pośrednika, powołuje opiekuna i pilnuje obu stron transakcji
#[test]
fn whole_day() {
    let room = new_room();

    add_broker_rule(&room, &broker_rule());
    take_card(&room, &card());
    propose_broker(&room, &proposal());
    record_case_file_opened(&room, &case_file_opened());
    appoint_agent(
        &room,
        &shared_agenci::PowolajAgentaParams {
            wlasciciel: "tozsamosc-posrednika".to_string(),
            teczka_odnosnik: Some("pokoj-teczki-oferty-1".to_string()),
            ..agent("opiekun-oferty-1")
        },
    );
    add_case_to_list(
        &room,
        &shared_spis::WpiszSpraweDoSpisuParams {
            teczka_odnosnik: "pokoj-teczki-oferty-1".to_string(),
            ..case_in_list()
        },
    );

    add_pair(&room, &pair());
    assess_contracts(&room, &both_contracts_allow(1));
    hand_pair_over(&room, &pair_handover(1, "2026-11-10T12:00:00Z"));
    record_transaction_file(&room, &transaction_file(1, "pokoj-transakcji-1"));

    // The contact is read past the operations: the open cards no longer show the card.
    let card_rows = room
        .execute(
            "SELECT stan, kontakt FROM kierownik_posrednictwa_przydzial_karty",
            &[],
        )
        .expect("read the card");
    let card_state = card_rows[0]
        .get::<KartyStan>("stan")
        .expect("the card has a state");
    let card_contact = card_rows[0]
        .get::<Option<String>>("kontakt")
        .expect("read the card's contact");
    assert_eq!((card_state, card_contact), (KartyStan::Przyjeta, None));

    let open_cards =
        kierownik_posrednictwa_przydzial::karty_otwarte(&room).expect("read the open cards");
    assert!(open_cards.is_empty());

    let workload = kierownik_posrednictwa_przydzial::obciazenie_posrednikow(&room)
        .expect("read the brokers' workload");
    let expected_workload = [ObciazeniePosrednikowRow {
        regula_id: 1,
        posrednik: "tozsamosc-posrednika".to_string(),
        limit_spraw: 20,
        przyjmuje_nowe: true,
        sprawy: 1,
    }];
    assert_eq!(workload, expected_workload);

    let pairs_to_clear = kierownik_posrednictwa_obie_strony::pary_do_wyjasnienia(&room)
        .expect("read the pairs to clear up");
    assert!(pairs_to_clear.is_empty());

    let pairs = kierownik_posrednictwa_obie_strony::pary(&room).expect("read the pairs");
    assert_eq!(
        pairs[0].transakcja_odnosnik.as_deref(),
        Some("pokoj-transakcji-1")
    );
}

// Broker rules ------------------------------------------------------------------------------------

/// reguła pośrednika powstaje aktualna, jest jedna i ma dodatni limit spraw
#[test]
fn one_rule_per_broker() {
    let room = new_room();
    add_broker_rule(&room, &broker_rule());

    expect::refused(
        kierownik_posrednictwa_przydzial::zapisz_regule_posrednika(&room, &broker_rule()),
        "ten pośrednik ma już aktualną regułę",
    );

    let rule_without_cases = ZapiszRegulePosrednikaParams {
        posrednik: "tozsamosc-drugiego".to_string(),
        limit_spraw: 0,
        ..broker_rule()
    };
    expect::refused(
        kierownik_posrednictwa_przydzial::zapisz_regule_posrednika(&room, &rule_without_cases),
        "limit_spraw",
    );
    expect::refused(
        room.execute(
            "INSERT INTO kierownik_posrednictwa_przydzial_posrednicy (posrednik, obszar,
              rodzaje_nieruchomosci, limit_spraw, przyjmuje_nowe, od, wiadomosc_odnosnik, podpis,
              zastapione_od)
            VALUES ('p', 'o', 'r', 5, 1, '2026-11-01', 'w', 's', '2026-11-02')",
            &[],
        ),
        "reguła pośrednika powstaje aktualna",
    );

    expect::refused(
        kierownik_posrednictwa_przydzial::zastap_regule_posrednika(
            &room,
            &rule_replacement("2026-10-01T00:00:00Z"),
        ),
        "zastapienie_nie_przed_poczatkiem",
    );
    replace_broker_rule(&room, "2026-12-01T00:00:00Z");
    add_broker_rule(
        &room,
        &ZapiszRegulePosrednikaParams {
            limit_spraw: 10,
            od: "2026-12-01T00:00:00Z".to_string(),
            ..broker_rule()
        },
    );

    let brokers = kierownik_posrednictwa_przydzial::posrednicy(&room).expect("read the brokers");
    let case_limits: Vec<i64> = brokers.iter().map(|row| row.limit_spraw).collect();
    assert_eq!(case_limits, [10]);

    let rule_history = kierownik_posrednictwa_przydzial::historia_regul_posrednika(
        &room,
        &HistoriaRegulPosrednikaParams {
            posrednik: "tozsamosc-posrednika".to_string(),
        },
    )
    .expect("read the history of the broker's rules");
    assert_eq!(rule_history.len(), 2);
}

/// reguły pośrednika się nie zmienia, a zastąpienie odnotowuje się raz
#[test]
fn broker_rule_does_not_change() {
    let room = new_room();
    add_broker_rule(&room, &broker_rule());

    let rule = "reguły pośrednika się nie zmienia";
    expect::refused(
        room.execute(
            "UPDATE kierownik_posrednictwa_przydzial_posrednicy SET limit_spraw = 99",
            &[],
        ),
        rule,
    );
    replace_broker_rule(&room, "2026-12-01T00:00:00Z");
    expect::refused(
        kierownik_posrednictwa_przydzial::zastap_regule_posrednika(
            &room,
            &rule_replacement("2026-12-02T00:00:00Z"),
        ),
        rule,
    );
}

// Cards -------------------------------------------------------------------------------------------

/// karta powstaje czekająca, a propozycja według reguły wskazuje jej pośrednika
#[test]
fn proposal_follows_the_broker_rule() {
    let room = new_room();
    add_broker_rule(&room, &broker_rule());
    add_broker_rule(
        &room,
        &ZapiszRegulePosrednikaParams {
            posrednik: "tozsamosc-pelnego".to_string(),
            przyjmuje_nowe: false,
            ..broker_rule()
        },
    );
    take_card(&room, &card());

    expect::refused(
        room.execute(
            "INSERT INTO kierownik_posrednictwa_przydzial_karty (karta_odnosnik, otrzymano, stan,
              powod, rozstrzygnieto)
            VALUES ('karta-9', '2026-11-04', 'odrzucona', 'p', '2026-11-04')",
            &[],
        ),
        "karta powstaje jako czekająca",
    );

    let rule = "propozycja według reguły wskazuje pośrednika aktualnej reguły";
    let other_broker = ZaproponujPosrednikaParams {
        posrednik: "tozsamosc-innego".to_string(),
        ..proposal()
    };
    expect::refused(
        kierownik_posrednictwa_przydzial::zaproponuj_posrednika(&room, &other_broker),
        rule,
    );

    let full_broker = ZaproponujPosrednikaParams {
        posrednik: "tozsamosc-pelnego".to_string(),
        regula_id: Some(2),
        ..proposal()
    };
    expect::refused(
        kierownik_posrednictwa_przydzial::zaproponuj_posrednika(&room, &full_broker),
        rule,
    );
    replace_broker_rule(&room, "2026-11-04T00:00:00Z");
    expect::refused(
        kierownik_posrednictwa_przydzial::zaproponuj_posrednika(&room, &proposal()),
        rule,
    );

    add_broker_rule(
        &room,
        &ZapiszRegulePosrednikaParams {
            od: "2026-11-04T00:00:00Z".to_string(),
            ..broker_rule()
        },
    );
    propose_broker(
        &room,
        &ZaproponujPosrednikaParams {
            regula_id: Some(3),
            ..proposal()
        },
    );
}

/// pośrednik spoza reguł wymaga zgody człowieka, a propozycja ma pośrednika i datę
#[test]
fn broker_outside_rules_needs_human() {
    let room = new_room();
    take_card(&room, &card());

    let outside_rules_without_human = ZaproponujPosrednikaParams {
        regula_id: None,
        ..proposal()
    };
    expect::refused(
        kierownik_posrednictwa_przydzial::zaproponuj_posrednika(
            &room,
            &outside_rules_without_human,
        ),
        "posrednik_poza_regulami_za_zgoda_czlowieka",
    );

    // A writer outside the client can leave out the date that `zaproponuj_posrednika` requires.
    // The table refuses the proposal without one.
    let proposal_without_date = room.execute(
        "UPDATE kierownik_posrednictwa_przydzial_karty
        SET stan = 'zaproponowana',
          posrednik = :posrednik,
          regula_id = :regula_id,
          zaproponowano = :zaproponowano,
          zatwierdzil = :zatwierdzil,
          podpis = :podpis,
          zatwierdzono = :zatwierdzono
        WHERE id = :id",
        &[
            ("posrednik", text("tozsamosc-posrednika")),
            ("regula_id", SqlValue::Null),
            ("zaproponowano", SqlValue::Null),
            ("zatwierdzil", text(OWNER)),
            ("podpis", text(OWNER_SIGNATURE)),
            ("zatwierdzono", text(APPROVED_AT)),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(proposal_without_date, "propozycja_z_posrednikiem");

    let approval_without_signature = ZaproponujPosrednikaParams {
        regula_id: None,
        zatwierdzil: Some(OWNER.to_string()),
        ..proposal()
    };
    expect::refused(
        kierownik_posrednictwa_przydzial::zaproponuj_posrednika(&room, &approval_without_signature),
        "zatwierdzenie_w_komplecie",
    );
    propose_broker(&room, &proposal_approved_by_owner());

    kierownik_posrednictwa_przydzial::cofnij_propozycje(&room, &CofnijPropozycjeParams { id: 1 })
        .expect("withdraw the proposal");

    let open_cards =
        kierownik_posrednictwa_przydzial::karty_otwarte(&room).expect("read the open cards");
    let open_card_states: Vec<KartyStan> = open_cards.iter().map(|row| row.stan).collect();
    assert_eq!(open_card_states, [KartyStan::Czeka]);
}

/// przyjęta karta ma teczkę, odrzucona powód, a obie są bez kontaktu i bez zmian
#[test]
fn resolved_card_has_no_contact() {
    let room = new_room();
    add_broker_rule(&room, &broker_rule());
    take_card(&room, &card());
    take_card(
        &room,
        &PrzyjmijKarteParams {
            karta_odnosnik: "pokoj-kierownika-biura/karta-2".to_string(),
            ..card()
        },
    );
    propose_broker(&room, &proposal());

    let case_file_opening = "UPDATE kierownik_posrednictwa_przydzial_karty
        SET stan = 'przyjeta',
          teczka_odnosnik = :teczka_odnosnik,
          rozstrzygnieto = :rozstrzygnieto,
          kontakt = NULL
        WHERE id = :id";

    // A writer outside the client can leave out the case file that `odnotuj_zalozenie_teczki`
    // requires. The table refuses the accepted card without one.
    let accepted_without_case_file = room.execute(
        case_file_opening,
        &[
            ("teczka_odnosnik", SqlValue::Null),
            ("rozstrzygnieto", text("2026-11-04T12:00:00Z")),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(accepted_without_case_file, "przyjeta_z_teczka");

    // A writer outside the client can leave out the date that `odnotuj_zalozenie_teczki`
    // requires. The table refuses the accepted card without one.
    let accepted_without_date = room.execute(
        case_file_opening,
        &[
            ("teczka_odnosnik", text("pokoj-teczki-oferty-1")),
            ("rozstrzygnieto", SqlValue::Null),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(accepted_without_date, "rozstrzygnieta_z_data");

    let accepted_before_received = OdnotujZalozenieTeczkiParams {
        rozstrzygnieto: "2026-11-03T12:00:00Z".to_string(),
        ..case_file_opened()
    };
    expect::refused(
        kierownik_posrednictwa_przydzial::odnotuj_zalozenie_teczki(
            &room,
            &accepted_before_received,
        ),
        "rozstrzygniecie_nie_przed_otrzymaniem",
    );
    expect::refused(
        room.execute(
            "UPDATE kierownik_posrednictwa_przydzial_karty
            SET stan = 'przyjeta', teczka_odnosnik = 't', rozstrzygnieto = '2026-11-04T12:00:00Z'
            WHERE id = 1",
            &[],
        ),
        "rozstrzygnieta_bez_kontaktu",
    );

    // A writer outside the client can leave out the reason that `odrzuc_karte` requires. The
    // table refuses the rejected card without one.
    let rejected_without_reason = room.execute(
        "UPDATE kierownik_posrednictwa_przydzial_karty
        SET stan = 'odrzucona',
          posrednik = NULL,
          regula_id = NULL,
          zaproponowano = NULL,
          powod = :powod,
          rozstrzygnieto = :rozstrzygnieto,
          kontakt = NULL
        WHERE id = :id",
        &[
            ("powod", SqlValue::Null),
            ("rozstrzygnieto", text("2026-11-04T12:00:00Z")),
            ("id", SqlValue::Integer(2)),
        ],
    );
    expect::refused(rejected_without_reason, "odrzucona_z_powodem");

    record_case_file_opened(&room, &case_file_opened());
    kierownik_posrednictwa_przydzial::odrzuc_karte(
        &room,
        &OdrzucKarteParams {
            powod: "Zgłoszenie spoza obszaru biura".to_string(),
            rozstrzygnieto: "2026-11-04T12:00:00Z".to_string(),
            id: 2,
        },
    )
    .expect("reject the card");

    let rule = "karty od Kierownika biura się nie zmienia, a przyjętej ani odrzuconej wcale";
    expect::refused(
        kierownik_posrednictwa_przydzial::cofnij_propozycje(
            &room,
            &CofnijPropozycjeParams { id: 1 },
        ),
        rule,
    );

    let proposal_for_rejected = ZaproponujPosrednikaParams {
        id: 2,
        ..proposal()
    };
    expect::refused(
        kierownik_posrednictwa_przydzial::zaproponuj_posrednika(&room, &proposal_for_rejected),
        rule,
    );
    expect::refused(
        room.execute(
            "UPDATE kierownik_posrednictwa_przydzial_karty SET otrzymano = '2026-11-01' WHERE id = 1",
            &[],
        ),
        rule,
    );
}

// Both sides --------------------------------------------------------------------------------------

/// para powstaje do wyjaśnienia, a „tak” wskazuje obie umowy
#[test]
fn pair_starts_unclear() {
    let room = new_room();
    add_pair(&room, &pair());

    expect::refused(
        room.execute(
            "INSERT INTO kierownik_posrednictwa_obie_strony_pary (teczka_oferty_odnosnik,
              teczka_poszukiwania_odnosnik, stwierdzono, umowy_pozwalaja, umowa_oferty_odnosnik,
              umowa_poszukiwania_odnosnik)
            VALUES ('o2', 'p2', '2026-11-10', 'tak', 'u1', 'u2')",
            &[],
        ),
        "para powstaje do wyjaśnienia",
    );

    let yes_with_one_contract = OcenUmowyParyParams {
        umowa_poszukiwania_odnosnik: None,
        ..both_contracts_allow(1)
    };
    expect::refused(
        kierownik_posrednictwa_obie_strony::ocen_umowy_pary(&room, &yes_with_one_contract),
        "zgoda_umow_wskazuje_obie_umowy",
    );
    assess_contracts(
        &room,
        &OcenUmowyParyParams {
            umowy_pozwalaja: ParyUmowyPozwalaja::Nie,
            ..yes_with_one_contract
        },
    );
    assess_contracts(&room, &both_contracts_allow(1));
    expect::refused(
        room.execute(
            "UPDATE kierownik_posrednictwa_obie_strony_pary SET teczka_oferty_odnosnik = 'inna' \
            WHERE id = 1",
            &[],
        ),
        "teczek pary się nie zmienia",
    );

    let pairs_to_clear = kierownik_posrednictwa_obie_strony::pary_do_wyjasnienia(&room)
        .expect("read the pairs to clear up");
    assert!(pairs_to_clear.is_empty());
}

/// ofertę przekazuje się dalej za zgodą obu umów albo człowieka, a potem się jej nie zmienia
#[test]
fn handover_needs_both_contracts_or_human() {
    let room = new_room();
    add_pair(&room, &pair());
    add_pair(
        &room,
        &ZapiszPareParams {
            teczka_poszukiwania_odnosnik: "pokoj-teczki-poszukiwania-2".to_string(),
            ..pair()
        },
    );
    assess_contracts(
        &room,
        &OcenUmowyParyParams {
            umowy_pozwalaja: ParyUmowyPozwalaja::Nie,
            umowa_oferty_odnosnik: None,
            umowa_poszukiwania_odnosnik: None,
            id: 2,
        },
    );

    let rule = "przekazanie_za_zgoda_umow_albo_czlowieka";
    expect::refused(
        kierownik_posrednictwa_obie_strony::odnotuj_przekazanie_pary(
            &room,
            &pair_handover(1, "2026-11-10T12:00:00Z"),
        ),
        rule,
    );
    expect::refused(
        kierownik_posrednictwa_obie_strony::odnotuj_przekazanie_pary(
            &room,
            &pair_handover(2, "2026-11-10T12:00:00Z"),
        ),
        rule,
    );
    expect::refused(
        kierownik_posrednictwa_obie_strony::odnotuj_przekazanie_pary(
            &room,
            &pair_handover_approved_by_owner(2, "2026-11-09T12:00:00Z"),
        ),
        "przekazanie_nie_przed_stwierdzeniem",
    );
    expect::refused(
        kierownik_posrednictwa_obie_strony::odnotuj_teczke_transakcji(
            &room,
            &transaction_file(2, "pokoj-transakcji-2"),
        ),
        "transakcja_po_przekazaniu",
    );
    hand_pair_over(
        &room,
        &pair_handover_approved_by_owner(2, "2026-11-10T12:00:00Z"),
    );

    let rule = "po przekazaniu także oceny umów";
    expect::refused(
        kierownik_posrednictwa_obie_strony::ocen_umowy_pary(&room, &both_contracts_allow(2)),
        rule,
    );
    record_transaction_file(&room, &transaction_file(2, "pokoj-transakcji-2"));
    expect::refused(
        kierownik_posrednictwa_obie_strony::odnotuj_teczke_transakcji(
            &room,
            &transaction_file(2, "pokoj-transakcji-3"),
        ),
        rule,
    );
}

// R9 and replacement ------------------------------------------------------------------------------

/// The head's own tables in the room.
fn own_tables_of(room: &MemoryRoom) -> Vec<String> {
    let rows = room
        .execute(
            r"SELECT name FROM sqlite_schema WHERE type = 'table'
            AND name LIKE 'kierownik\_posrednictwa\_%' ESCAPE '\'",
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
    assert_eq!(tables.len(), 3, "{tables:?}");

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

/// REPLACE nie usuwa aktualnej reguły pośrednika, karty ani pary przez klucz UNIQUE
#[test]
fn no_replacement_through_unique_keys() {
    let room = new_room();
    full_memory(&room);

    // The table, its unique key for an upsert (none for the current broker rule) and the refusal.
    let unique_keys = [
        (
            "kierownik_posrednictwa_przydzial_karty",
            Some("karta_odnosnik"),
            REPLACEMENT,
        ),
        (
            "kierownik_posrednictwa_obie_strony_pary",
            Some("teczka_oferty_odnosnik, teczka_poszukiwania_odnosnik"),
            REPLACEMENT,
        ),
        (
            "kierownik_posrednictwa_przydzial_posrednicy",
            None,
            "ten pośrednik ma już aktualną regułę",
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

        if let Some(key) = key {
            // `WHERE true` keeps SQLite from reading ON CONFLICT as a join constraint.
            let upsert = format!(
                "INSERT INTO {table} ({columns}) SELECT {selected} FROM {table} \
                WHERE true ON CONFLICT ({key}) DO UPDATE SET id = excluded.id"
            );
            expect::refused(room.execute(&upsert, &[]), refusal);
        }
    }
}
