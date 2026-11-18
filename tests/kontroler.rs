//! Scenarios of the Controller's memory room (spec/positions/kontrola/kontroler).
//!
//! The room is the Controller's own kind and the shared agent kinds it imports. The shared kinds
//! are checked in stanowisko.rs; here each scenario starts from an empty room and checks one rule
//! of kontroler-sprawdzenia: a write the rule allows and a write it refuses. The first scenario
//! follows a day of the Controller: an identity verified in a public register, beneficial owners
//! checked against a statement that turns out wrong, a proposed risk level, the checks passed to
//! the Head of control and, once they are in the control register, their details cleared (D13).

use std::collections::HashSet;

use xboards_agencja_nieruchomosci::positions::kontrola::kontroler::{
    self, kontroler_sprawdzenia, stanowisko_zrodla,
};
use xboards_agencja_nieruchomosci::positions::stanowisko::stanowisko_zrodla::{
    DopiszZdarzenieZrodlaParams, WpisyRodzaj as RodzajZrodla, WpiszZrodloParams, ZdarzeniaRodzaj,
};
use xboards_agencja_nieruchomosci::xmeta_room::MemoryRoom;

use kontroler_sprawdzenia::{
    OdnotujPrzekazanieSprawdzeniaParams, OdnotujWpisSprawdzeniaWRejestrzeParams, SprawdzeniaRow,
    SprawdzeniaTeczkiParams, WpisyProponowanyPoziom, WpisyRodzaj, WpisyWynik,
    ZapiszSprawdzenieParams, ZastapSprawdzenieParams,
};
use stanowisko_zrodla::{OcenZrodloParams, Ocena};

use crate::expect;
use crate::stanowisko::{
    DELETION, REPLACEMENT, add_source, add_source_event, first_row_copy, source, source_event,
};

/// The refusal of a change to a check handed over to the Head of control.
const HANDED_OVER: &str = "przekazanego sprawdzenia się nie zmienia";

fn new_room() -> MemoryRoom {
    kontroler::open_in_memory().expect("open the room of the Controller")
}

// Building blocks ---------------------------------------------------------------------------------

/// The default parameters of a check: a verification of the seller in a public register, which
/// matches.
fn check(check_id: &str) -> ZapiszSprawdzenieParams {
    ZapiszSprawdzenieParams {
        id: check_id.to_string(),
        rodzaj: WpisyRodzaj::Weryfikacja,
        teczka_odnosnik: "pokoj-teczki-oferty".to_string(),
        dotyczy_tabela: Some("sprawa_uczestnicy_wpisy".to_string()),
        dotyczy_odnosnik: Some("pokoj-teczki-oferty/sprzedajaca".to_string()),
        zrodlo: Some("RDO".to_string()),
        zrodlo_data: Some("2026-11-04".to_string()),
        zrodlo_id: None,
        wynik: WpisyWynik::Zgodne,
        opis: None,
        proponowany_poziom: None,
        czynniki: None,
        powod_watpliwosci: false,
        sprawdzono: "2026-11-04T10:00:00Z".to_string(),
    }
}

/// The default parameters of a risk assessment: a medium level proposed for a company client.
fn risk_assessment(check_id: &str) -> ZapiszSprawdzenieParams {
    ZapiszSprawdzenieParams {
        rodzaj: WpisyRodzaj::OcenaRyzyka,
        dotyczy_tabela: None,
        dotyczy_odnosnik: None,
        zrodlo: None,
        zrodlo_data: None,
        proponowany_poziom: Some(WpisyProponowanyPoziom::Srednie),
        czynniki: Some("klient spółka; wartość transakcji powyżej progu".to_string()),
        ..check(check_id)
    }
}

fn save_check(room: &MemoryRoom, params: &ZapiszSprawdzenieParams) {
    kontroler_sprawdzenia::zapisz_sprawdzenie(room, params).expect("save the check");
}

fn hand_over(room: &MemoryRoom, check_id: &str, przekazano: &str) {
    let params = OdnotujPrzekazanieSprawdzeniaParams {
        przekazano: przekazano.to_string(),
        id: check_id.to_string(),
    };

    kontroler_sprawdzenia::odnotuj_przekazanie_sprawdzenia(room, &params)
        .expect("hand the check over");
}

fn enter_in_register(room: &MemoryRoom, check_id: &str, rejestr_odnosnik: &str) {
    let params = OdnotujWpisSprawdzeniaWRejestrzeParams {
        rejestr_odnosnik: rejestr_odnosnik.to_string(),
        id: check_id.to_string(),
    };

    kontroler_sprawdzenia::odnotuj_wpis_sprawdzenia_w_rejestrze(room, &params)
        .expect("record the check's entry in the control register");
}

/// The parameters of replacing the check `check_id` with the check `replacement_id`.
fn replacement(check_id: &str, replacement_id: &str) -> ZastapSprawdzenieParams {
    ZastapSprawdzenieParams {
        zastapione_przez_id: replacement_id.to_string(),
        id: check_id.to_string(),
    }
}

/// The ids of the current checks.
fn current_check_ids(room: &MemoryRoom) -> HashSet<String> {
    let checks = kontroler_sprawdzenia::sprawdzenia(room).expect("read the checks");

    checks.into_iter().map(|row| row.id).collect()
}

// A whole day -------------------------------------------------------------------------------------

/// dzień Kontrolera: tożsamość, beneficjenci, ocena ryzyka, przekazanie i wpis w rejestrze
#[test]
fn whole_day() {
    let room = new_room();

    save_check(&room, &check("tozsamosc"));
    add_source(
        &room,
        &WpiszZrodloParams {
            nazwa: Some("Pełnomocnik spółki kupującej".to_string()),
            rodzaj: RodzajZrodla::Osoba,
            ..source("pelnomocnik")
        },
    );
    save_check(
        &room,
        &ZapiszSprawdzenieParams {
            rodzaj: WpisyRodzaj::Beneficjenci,
            teczka_odnosnik: "pokoj-teczki-transakcji".to_string(),
            dotyczy_odnosnik: Some("pokoj-teczki-transakcji/spolka-kupujaca".to_string()),
            zrodlo: Some("CRBR".to_string()),
            zrodlo_id: Some("pelnomocnik".to_string()),
            wynik: WpisyWynik::Uwagi,
            opis: Some("Oświadczenie pełnomocnika podaje innego beneficjenta niż CRBR".to_string()),
            powod_watpliwosci: true,
            ..check("beneficjenci-1")
        },
    );
    add_source_event(
        &room,
        &DopiszZdarzenieZrodlaParams {
            dotyczy_tabela: Some("sprawa_uczestnicy_wpisy".to_string()),
            dotyczy_odnosnik: Some("pokoj-teczki-transakcji/spolka-kupujaca".to_string()),
            ..source_event("pelnomocnik", ZdarzeniaRodzaj::Rozbieznosc)
        },
    );
    stanowisko_zrodla::ocen_zrodlo(
        &room,
        &OcenZrodloParams {
            ocena: Ocena::Zastrzezenia,
            oceniono: "2026-11-04T11:00:00Z".to_string(),
            id: "pelnomocnik".to_string(),
        },
    )
    .expect("assess the source");
    save_check(
        &room,
        &ZapiszSprawdzenieParams {
            rodzaj: WpisyRodzaj::Beneficjenci,
            teczka_odnosnik: "pokoj-teczki-transakcji".to_string(),
            dotyczy_odnosnik: Some("pokoj-teczki-transakcji/spolka-kupujaca".to_string()),
            zrodlo: Some("KRS i CRBR".to_string()),
            zrodlo_data: Some("2026-11-05".to_string()),
            sprawdzono: "2026-11-05T09:00:00Z".to_string(),
            ..check("beneficjenci-2")
        },
    );
    kontroler_sprawdzenia::zastap_sprawdzenie(
        &room,
        &replacement("beneficjenci-1", "beneficjenci-2"),
    )
    .expect("replace the check");
    save_check(
        &room,
        &ZapiszSprawdzenieParams {
            sprawdzono: "2026-11-05T10:00:00Z".to_string(),
            ..risk_assessment("ryzyko")
        },
    );

    let checks_to_hand_over = kontroler_sprawdzenia::sprawdzenia_do_przekazania(&room)
        .expect("read the checks to hand over");
    for row in checks_to_hand_over {
        hand_over(&room, &row.id, "2026-11-05T12:00:00Z");
    }
    enter_in_register(&room, "ryzyko", "pokoj-rejestru-kontroli/ocena-7");

    let checks_left_to_hand_over = kontroler_sprawdzenia::sprawdzenia_do_przekazania(&room)
        .expect("read the checks to hand over");
    assert!(checks_left_to_hand_over.is_empty());

    let expected_ids = HashSet::from([
        "tozsamosc".to_string(),
        "beneficjenci-2".to_string(),
        "ryzyko".to_string(),
    ]);
    assert_eq!(current_check_ids(&room), expected_ids);

    let checks_of_case = kontroler_sprawdzenia::sprawdzenia_teczki(
        &room,
        &SprawdzeniaTeczkiParams {
            teczka_odnosnik: "pokoj-teczki-transakcji".to_string(),
        },
    )
    .expect("read the checks of the case file");
    let checks_of_case_ids: Vec<&str> = checks_of_case.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(checks_of_case_ids, ["beneficjenci-2"]);

    // The whole row of the table, read past the operations.
    let entered_rows = room
        .execute(
            "SELECT * FROM kontroler_sprawdzenia_wpisy WHERE id = 'ryzyko'",
            &[],
        )
        .expect("read the entered check");
    let entered = SprawdzeniaRow::from_row(&entered_rows[0]).expect("read the row of the check");
    assert_eq!(entered.opis, None);
    assert_eq!(entered.czynniki, None);
    assert_eq!(entered.proponowany_poziom, None);
    assert_eq!(entered.rodzaj, WpisyRodzaj::OcenaRyzyka);
    assert_eq!(entered.wynik, WpisyWynik::Zgodne);
    assert_eq!(
        entered.rejestr_odnosnik.as_deref(),
        Some("pokoj-rejestru-kontroli/ocena-7")
    );
}

// Writing a check ---------------------------------------------------------------------------------

/// identyfikacja, weryfikacja i beneficjenci mają źródło z datą
#[test]
fn identity_check_has_source() {
    let room = new_room();
    save_check(&room, &check("tozsamosc"));
    save_check(
        &room,
        &ZapiszSprawdzenieParams {
            rodzaj: WpisyRodzaj::Sankcje,
            zrodlo: None,
            zrodlo_data: None,
            ..check("sankcje")
        },
    );

    let owners_without_source = ZapiszSprawdzenieParams {
        rodzaj: WpisyRodzaj::Beneficjenci,
        zrodlo: None,
        zrodlo_data: None,
        ..check("beneficjenci")
    };
    expect::refused(
        kontroler_sprawdzenia::zapisz_sprawdzenie(&room, &owners_without_source),
        "identyfikacja_ze_zrodlem_i_data",
    );

    let source_without_date = ZapiszSprawdzenieParams {
        rodzaj: WpisyRodzaj::Identyfikacja,
        zrodlo_data: None,
        ..check("identyfikacja")
    };
    expect::refused(
        kontroler_sprawdzenia::zapisz_sprawdzenie(&room, &source_without_date),
        "zrodlo_z_data",
    );
}

/// sprawdzany wiersz ma tabelę razem z odnośnikiem
#[test]
fn checked_row_has_table() {
    let room = new_room();
    save_check(
        &room,
        &ZapiszSprawdzenieParams {
            rodzaj: WpisyRodzaj::Pep,
            dotyczy_tabela: None,
            dotyczy_odnosnik: None,
            ..check("pep")
        },
    );

    let row_without_table = ZapiszSprawdzenieParams {
        rodzaj: WpisyRodzaj::Pep,
        dotyczy_tabela: None,
        ..check("pep-2")
    };
    expect::refused(
        kontroler_sprawdzenia::zapisz_sprawdzenie(&room, &row_without_table),
        "dotyczy_tabela_razem_z_odnosnikiem",
    );
}

/// poziom i czynniki są tylko przy ocenie ryzyka, a ocena ryzyka je ma
#[test]
fn level_and_factors_only_with_risk_assessment() {
    let room = new_room();
    save_check(&room, &risk_assessment("ryzyko"));

    let sanctions_with_level = ZapiszSprawdzenieParams {
        rodzaj: WpisyRodzaj::Sankcje,
        proponowany_poziom: Some(WpisyProponowanyPoziom::Niskie),
        ..check("sankcje")
    };
    expect::refused(
        kontroler_sprawdzenia::zapisz_sprawdzenie(&room, &sanctions_with_level),
        "poziom_i_czynniki_tylko_przy_ocenie_ryzyka",
    );

    let pep_with_factors = ZapiszSprawdzenieParams {
        rodzaj: WpisyRodzaj::Pep,
        czynniki: Some("stanowisko polityczne".to_string()),
        ..check("pep")
    };
    expect::refused(
        kontroler_sprawdzenia::zapisz_sprawdzenie(&room, &pep_with_factors),
        "poziom_i_czynniki_tylko_przy_ocenie_ryzyka",
    );

    let assessment_without_factors = ZapiszSprawdzenieParams {
        czynniki: None,
        ..risk_assessment("ryzyko-2")
    };
    expect::refused(
        kontroler_sprawdzenia::zapisz_sprawdzenie(&room, &assessment_without_factors),
        "ocena_ryzyka_z_poziomem_i_czynnikami",
    );

    let assessment_without_level = ZapiszSprawdzenieParams {
        proponowany_poziom: None,
        ..risk_assessment("ryzyko-3")
    };
    expect::refused(
        kontroler_sprawdzenia::zapisz_sprawdzenie(&room, &assessment_without_level),
        "ocena_ryzyka_z_poziomem_i_czynnikami",
    );
}

/// uwagi i brak ustalenia mają opis
#[test]
fn remarks_have_description() {
    let room = new_room();
    save_check(
        &room,
        &ZapiszSprawdzenieParams {
            rodzaj: WpisyRodzaj::ZrodloSrodkow,
            wynik: WpisyWynik::Uwagi,
            opis: Some("Brak wyciągu".to_string()),
            ..check("zrodlo-srodkow")
        },
    );

    let unknown_without_description = ZapiszSprawdzenieParams {
        rodzaj: WpisyRodzaj::Crbr,
        wynik: WpisyWynik::NieMoznaUstalic,
        ..check("crbr")
    };
    expect::refused(
        kontroler_sprawdzenia::zapisz_sprawdzenie(&room, &unknown_without_description),
        "uwagi_z_opisem",
    );
}

/// sprawdzenie powstaje robocze: bez przekazania, wpisu w rejestrze i zastąpienia
#[test]
fn check_starts_as_draft() {
    let room = new_room();
    save_check(&room, &check("tozsamosc"));

    let rule = "sprawdzenie powstaje robocze";
    let columns_set_later = [
        ("przekazano", "'2026-11-04T12:00:00Z'"),
        ("zastapione_przez_id", "'tozsamosc'"),
    ];
    for (column, value) in columns_set_later {
        let check_with_later_column = format!(
            "INSERT INTO kontroler_sprawdzenia_wpisy (id, rodzaj, teczka_odnosnik, wynik,
              powod_watpliwosci, sprawdzono, {column})
            VALUES ('gotowe', 'sankcje', 'pokoj-teczki-oferty', 'zgodne', 0,
              '2026-11-04T10:00:00Z', {value})"
        );
        expect::refused(room.execute(&check_with_later_column, &[]), rule);
    }
}

// Handover and the control register ---------------------------------------------------------------

/// wpis w rejestrze jest tylko po przekazaniu, a przekazanie po sprawdzeniu
#[test]
fn register_entry_after_handover() {
    let room = new_room();
    save_check(&room, &check("tozsamosc"));

    let entry_before_handover = OdnotujWpisSprawdzeniaWRejestrzeParams {
        rejestr_odnosnik: "pokoj-rejestru-kontroli/sprawdzenie-41".to_string(),
        id: "tozsamosc".to_string(),
    };
    expect::refused(
        kontroler_sprawdzenia::odnotuj_wpis_sprawdzenia_w_rejestrze(&room, &entry_before_handover),
        "wpis_w_rejestrze_po_przekazaniu",
    );

    let handover_before_check = OdnotujPrzekazanieSprawdzeniaParams {
        przekazano: "2026-11-01T00:00:00Z".to_string(),
        id: "tozsamosc".to_string(),
    };
    expect::refused(
        kontroler_sprawdzenia::odnotuj_przekazanie_sprawdzenia(&room, &handover_before_check),
        "przekazanie_nie_przed_sprawdzeniem",
    );
}

/// roboczy wpis się poprawia, a przekazanego się nie zmienia
#[test]
fn handed_over_check_is_fixed() {
    let room = new_room();
    save_check(
        &room,
        &ZapiszSprawdzenieParams {
            rodzaj: WpisyRodzaj::Sankcje,
            wynik: WpisyWynik::Uwagi,
            opis: Some("Zbieżność nazwiska".to_string()),
            ..check("sankcje")
        },
    );

    // The client has no operation that corrects a draft, so the correction is plain SQL.
    room.execute(
        "UPDATE kontroler_sprawdzenia_wpisy SET opis = 'Zbieżność nazwiska; inna data urodzenia' \
        WHERE id = 'sankcje'",
        &[],
    )
    .expect("correct the draft check");
    hand_over(&room, "sankcje", "2026-11-04T12:00:00Z");

    let changes = [
        ("wynik", "'zgodne'"),
        ("opis", "'Inny opis'"),
        ("zrodlo_data", "'2026-11-03'"),
        ("powod_watpliwosci", "1"),
        ("przekazano", "'2026-11-06T12:00:00Z'"),
    ];
    for (column, value) in changes {
        let change = format!(
            "UPDATE kontroler_sprawdzenia_wpisy SET {column} = {value} WHERE id = 'sankcje'"
        );
        expect::refused(room.execute(&change, &[]), HANDED_OVER);
    }
}

/// D13: po wpisie w rejestrze opis, czynniki i poziom znikają, a wpisu się nie zmienia
#[test]
fn register_entry_clears_details() {
    let room = new_room();
    save_check(&room, &risk_assessment("ryzyko"));
    hand_over(&room, "ryzyko", "2026-11-04T12:00:00Z");

    expect::refused(
        room.execute(
            "UPDATE kontroler_sprawdzenia_wpisy \
            SET rejestr_odnosnik = 'pokoj-rejestru-kontroli/ocena-7' WHERE id = 'ryzyko'",
            &[],
        ),
        "po_wpisie_w_rejestrze_bez_szczegolow",
    );
    enter_in_register(&room, "ryzyko", "pokoj-rejestru-kontroli/ocena-7");

    expect::refused(
        room.execute(
            "UPDATE kontroler_sprawdzenia_wpisy SET opis = 'Szczegóły wracają' WHERE id = 'ryzyko'",
            &[],
        ),
        "po_wpisie_w_rejestrze_bez_szczegolow",
    );

    let second_entry = OdnotujWpisSprawdzeniaWRejestrzeParams {
        rejestr_odnosnik: "pokoj-rejestru-kontroli/ocena-8".to_string(),
        id: "ryzyko".to_string(),
    };
    expect::refused(
        kontroler_sprawdzenia::odnotuj_wpis_sprawdzenia_w_rejestrze(&room, &second_entry),
        HANDED_OVER,
    );
    expect::refused(
        room.execute(
            "UPDATE kontroler_sprawdzenia_wpisy SET rejestr_odnosnik = NULL WHERE id = 'ryzyko'",
            &[],
        ),
        HANDED_OVER,
    );
}

// Replacement by a new check ----------------------------------------------------------------------

/// ponowne sprawdzenie tego samego rodzaju, nie starsze, zastępuje poprzednie raz
#[test]
fn new_check_replaces_old() {
    let room = new_room();
    save_check(&room, &check("tozsamosc"));
    hand_over(&room, "tozsamosc", "2026-11-04T12:00:00Z");
    save_check(
        &room,
        &ZapiszSprawdzenieParams {
            sprawdzono: "2026-11-06T10:00:00Z".to_string(),
            ..check("tozsamosc-2")
        },
    );
    save_check(
        &room,
        &ZapiszSprawdzenieParams {
            sprawdzono: "2026-11-07T10:00:00Z".to_string(),
            ..check("tozsamosc-3")
        },
    );
    save_check(
        &room,
        &ZapiszSprawdzenieParams {
            sprawdzono: "2026-11-01T10:00:00Z".to_string(),
            ..check("dawne")
        },
    );
    save_check(
        &room,
        &ZapiszSprawdzenieParams {
            rodzaj: WpisyRodzaj::Sankcje,
            sprawdzono: "2026-11-06T10:00:00Z".to_string(),
            ..check("sankcje")
        },
    );

    let rule = "sprawdzenie zastępuje ponowne sprawdzenie tego samego rodzaju";
    expect::refused(
        kontroler_sprawdzenia::zastap_sprawdzenie(&room, &replacement("tozsamosc", "sankcje")),
        rule,
    );
    expect::refused(
        kontroler_sprawdzenia::zastap_sprawdzenie(&room, &replacement("tozsamosc", "dawne")),
        rule,
    );
    expect::refused(
        kontroler_sprawdzenia::zastap_sprawdzenie(&room, &replacement("tozsamosc", "tozsamosc")),
        "zastapione_nie_przez_siebie",
    );

    kontroler_sprawdzenia::zastap_sprawdzenie(&room, &replacement("tozsamosc", "tozsamosc-2"))
        .expect("replace the check");
    expect::refused(
        kontroler_sprawdzenia::zastap_sprawdzenie(&room, &replacement("tozsamosc", "tozsamosc-3")),
        "zastąpionego sprawdzenia nie zastępuje się drugi raz",
    );
    assert!(!current_check_ids(&room).contains("tozsamosc"));
}

// R9 and replacement ------------------------------------------------------------------------------

/// R9: sprawdzeń się nie usuwa ani nie zastępuje przez REPLACE albo upsert
#[test]
fn no_deletion_or_replacement() {
    let room = new_room();
    save_check(&room, &check("tozsamosc"));

    let table_rows = room
        .execute(
            r"SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'kontroler\_%' ESCAPE '\'",
            &[],
        )
        .expect("read the tables");
    let tables: Vec<String> = table_rows
        .iter()
        .map(|row| row.get::<String>("name").expect("a table has a name"))
        .collect();
    assert_eq!(tables, ["kontroler_sprawdzenia_wpisy"], "{tables:?}");

    for table in &tables {
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
