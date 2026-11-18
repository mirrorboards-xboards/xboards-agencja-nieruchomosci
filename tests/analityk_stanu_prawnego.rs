//! Scenarios of the Legal status analyst's memory room
//! (spec/positions/transakcje/analityk_stanu_prawnego).
//!
//! The room is the analyst's two own kinds and the shared agent kinds it imports. The shared kinds
//! are checked in stanowisko.rs; here each scenario starts from an empty room and checks one rule
//! of analityk-stanu-prawnego-ksiega or analityk-stanu-prawnego-sprawdzenia: a write the rule
//! allows and a write it refuses. The first scenario follows a transaction: a land register
//! extract, checks of its sections and of the municipality's pre-emption right, a fresh viewing of
//! the register the day before the deed and the changes between the two states.

use std::collections::HashSet;

use xboards_agencja_nieruchomosci::positions::transakcje::analityk_stanu_prawnego::{
    self, analityk_stanu_prawnego_ksiega, analityk_stanu_prawnego_sprawdzenia,
};
use xboards_agencja_nieruchomosci::xmeta_room::MemoryRoom;

use analityk_stanu_prawnego_ksiega::{
    StanyKsiegiParams, StanySposob, WpisyDzial, WpisyRodzaj, ZapiszStanKsiegiParams,
    ZapiszWpisKsiegiParams, Zmiana, ZmianyKsiegiParams,
};
use analityk_stanu_prawnego_sprawdzenia::{
    OdnotujAnalizeSprawdzeniaParams, SprawdzeniaDokumentuParams, WpisyPrzedmiot, WpisyWynik,
    ZapiszSprawdzenieStanuPrawnegoParams, ZastapSprawdzenieStanuPrawnegoParams,
};

use crate::expect;
use crate::stanowisko::{DELETION, REPLACEMENT, first_row_copy};

/// The land register of the scenarios.
const NUMER_KW: &str = "WA1M/00012345/6";

fn new_room() -> MemoryRoom {
    analityk_stanu_prawnego::open_in_memory().expect("open the room of the Legal status analyst")
}

// Building blocks ---------------------------------------------------------------------------------

/// The default parameters of a state of the register: an extract in the transaction's case file.
fn state() -> ZapiszStanKsiegiParams {
    ZapiszStanKsiegiParams {
        numer_kw: NUMER_KW.to_string(),
        chwila: "2026-11-02T08:00:00Z".to_string(),
        sposob: StanySposob::Odpis,
        dokument_odnosnik: Some("pokoj-teczki-transakcji/odpis-1".to_string()),
    }
}

/// The parameters of a viewing of the register at the first moment, for the writes a rule refuses.
fn viewed_state() -> ZapiszStanKsiegiParams {
    ZapiszStanKsiegiParams {
        sposob: StanySposob::Przegladanie,
        dokument_odnosnik: None,
        ..state()
    }
}

fn save_state(room: &MemoryRoom, params: &ZapiszStanKsiegiParams) {
    analityk_stanu_prawnego_ksiega::zapisz_stan_ksiegi(room, params)
        .expect("save the state of the register");
}

/// The default parameters of an entry of the first state: a mortgage in section IV.
fn register_entry() -> ZapiszWpisKsiegiParams {
    ZapiszWpisKsiegiParams {
        stan_id: 1,
        dzial: WpisyDzial::Iv,
        numer: "1".to_string(),
        rodzaj: WpisyRodzaj::Wpis,
        przedmiot: "hipoteka umowna na rzecz banku".to_string(),
    }
}

fn save_register_entry(room: &MemoryRoom, params: &ZapiszWpisKsiegiParams) {
    analityk_stanu_prawnego_ksiega::zapisz_wpis_ksiegi(room, params)
        .expect("save the entry of the register");
}

/// The default parameters of a check of section IV of the first state, which finds a risk.
fn check(check_id: &str) -> ZapiszSprawdzenieStanuPrawnegoParams {
    ZapiszSprawdzenieStanuPrawnegoParams {
        id: check_id.to_string(),
        przedmiot: WpisyPrzedmiot::DzialIv,
        dokument_odnosnik: "pokoj-teczki-transakcji/odpis-1".to_string(),
        dokument_data: "2026-11-02".to_string(),
        przepis: "art. 65 ust. 1 ukwh".to_string(),
        wynik: WpisyWynik::Ryzyko,
        opis: Some("Hipoteka do spłaty z ceny; projekt umowy musi to opisać".to_string()),
        ksiega_stan_id: Some(1),
        sprawdzono: "2026-11-03T10:00:00Z".to_string(),
    }
}

fn save_check(room: &MemoryRoom, params: &ZapiszSprawdzenieStanuPrawnegoParams) {
    analityk_stanu_prawnego_sprawdzenia::zapisz_sprawdzenie_stanu_prawnego(room, params)
        .expect("save the check");
}

fn record_analysis(room: &MemoryRoom, check_id: &str, analiza_odnosnik: &str) {
    let params = OdnotujAnalizeSprawdzeniaParams {
        analiza_odnosnik: analiza_odnosnik.to_string(),
        id: check_id.to_string(),
    };

    analityk_stanu_prawnego_sprawdzenia::odnotuj_analize_sprawdzenia(room, &params)
        .expect("record the analysis of the check");
}

/// The parameters of replacing the check `check_id` with the check `replacement_id`.
fn replacement(check_id: &str, replacement_id: &str) -> ZastapSprawdzenieStanuPrawnegoParams {
    ZastapSprawdzenieStanuPrawnegoParams {
        zastapione_przez_id: replacement_id.to_string(),
        id: check_id.to_string(),
    }
}

fn register_changes(room: &MemoryRoom) -> HashSet<(Zmiana, WpisyDzial, String, WpisyRodzaj)> {
    let params = ZmianyKsiegiParams {
        numer_kw: NUMER_KW.to_string(),
    };
    let rows = analityk_stanu_prawnego_ksiega::zmiany_ksiegi(room, &params)
        .expect("read the changes of the register");

    rows.into_iter()
        .map(|row| (row.zmiana, row.dzial, row.numer, row.rodzaj))
        .collect()
}

// A whole transaction -----------------------------------------------------------------------------

/// transakcja: odpis, sprawdzenia działów i pierwokupu, przeglądanie przed aktem i zmiany
#[test]
fn whole_transaction() {
    let room = new_room();

    save_state(&room, &state());
    save_register_entry(
        &room,
        &ZapiszWpisKsiegiParams {
            dzial: WpisyDzial::Iii,
            przedmiot: "dożywocie na rzecz matki zbywcy".to_string(),
            ..register_entry()
        },
    );
    save_register_entry(&room, &register_entry());
    save_check(
        &room,
        &ZapiszSprawdzenieStanuPrawnegoParams {
            przedmiot: WpisyPrzedmiot::DzialIii,
            przepis: "art. 7 ukwh".to_string(),
            opis: Some("Dożywocie przechodzi na nabywcę".to_string()),
            ..check("dzial-iii")
        },
    );
    save_check(&room, &check("dzial-iv"));
    save_check(
        &room,
        &ZapiszSprawdzenieStanuPrawnegoParams {
            przedmiot: WpisyPrzedmiot::Pierwokup,
            przepis: "art. 109 ust. 1 ugn".to_string(),
            wynik: WpisyWynik::BezUwag,
            opis: None,
            ksiega_stan_id: None,
            ..check("pierwokup")
        },
    );
    let checks_to_analyse = analityk_stanu_prawnego_sprawdzenia::sprawdzenia_bez_analizy(&room)
        .expect("read the checks without analysis");
    for row in checks_to_analyse {
        record_analysis(&room, &row.id, "pokoj-teczki-transakcji/analiza-ksiegi");
    }

    assert!(register_changes(&room).is_empty());

    save_state(
        &room,
        &ZapiszStanKsiegiParams {
            chwila: "2026-11-19T07:30:00Z".to_string(),
            sposob: StanySposob::Przegladanie,
            dokument_odnosnik: None,
            ..state()
        },
    );
    save_register_entry(
        &room,
        &ZapiszWpisKsiegiParams {
            stan_id: 2,
            dzial: WpisyDzial::Iii,
            przedmiot: "dożywocie na rzecz matki zbywcy".to_string(),
            ..register_entry()
        },
    );
    save_register_entry(
        &room,
        &ZapiszWpisKsiegiParams {
            stan_id: 2,
            rodzaj: WpisyRodzaj::Wzmianka,
            numer: "Dz.Kw./4567/26".to_string(),
            przedmiot: "wniosek o wykreślenie hipoteki".to_string(),
            ..register_entry()
        },
    );
    save_check(
        &room,
        &ZapiszSprawdzenieStanuPrawnegoParams {
            dokument_odnosnik: "pokoj-teczki-transakcji/wydruk-2".to_string(),
            dokument_data: "2026-11-19".to_string(),
            ksiega_stan_id: Some(2),
            opis: Some("Wzmianka o wniosku wyłącza rękojmię do jego rozpoznania".to_string()),
            sprawdzono: "2026-11-19T08:00:00Z".to_string(),
            ..check("dzial-iv-2")
        },
    );
    analityk_stanu_prawnego_sprawdzenia::zastap_sprawdzenie_stanu_prawnego(
        &room,
        &replacement("dzial-iv", "dzial-iv-2"),
    )
    .expect("replace the check");

    let expected_changes = HashSet::from([
        (
            Zmiana::Usuniety,
            WpisyDzial::Iv,
            "1".to_string(),
            WpisyRodzaj::Wpis,
        ),
        (
            Zmiana::Dodany,
            WpisyDzial::Iv,
            "Dz.Kw./4567/26".to_string(),
            WpisyRodzaj::Wzmianka,
        ),
    ]);
    assert_eq!(register_changes(&room), expected_changes);

    let states = analityk_stanu_prawnego_ksiega::stany_ksiegi(
        &room,
        &StanyKsiegiParams {
            numer_kw: NUMER_KW.to_string(),
        },
    )
    .expect("read the states of the register");
    let state_ids: Vec<i64> = states.iter().map(|row| row.id).collect();
    assert_eq!(state_ids, [2, 1]);

    let checks = analityk_stanu_prawnego_sprawdzenia::sprawdzenia_stanu_prawnego(&room)
        .expect("read the checks");
    let check_ids: HashSet<&str> = checks.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(
        check_ids,
        HashSet::from(["dzial-iii", "dzial-iv-2", "pierwokup"])
    );

    let checks_without_analysis =
        analityk_stanu_prawnego_sprawdzenia::sprawdzenia_bez_analizy(&room)
            .expect("read the checks without analysis");
    let checks_without_analysis_ids: Vec<&str> = checks_without_analysis
        .iter()
        .map(|row| row.id.as_str())
        .collect();
    assert_eq!(checks_without_analysis_ids, ["dzial-iv-2"]);

    let checks_of_extract = analityk_stanu_prawnego_sprawdzenia::sprawdzenia_dokumentu(
        &room,
        &SprawdzeniaDokumentuParams {
            dokument_odnosnik: "pokoj-teczki-transakcji/odpis-1".to_string(),
        },
    )
    .expect("read the checks of the extract");
    let checks_of_extract_ids: Vec<&str> = checks_of_extract
        .iter()
        .map(|row| row.id.as_str())
        .collect();
    assert_eq!(checks_of_extract_ids, ["dzial-iii", "pierwokup"]);
}

// The land register -------------------------------------------------------------------------------

/// odpis wskazuje dokument w teczce, a przeglądanie nie musi
#[test]
fn extract_points_document() {
    let room = new_room();
    save_state(&room, &state());
    save_state(
        &room,
        &ZapiszStanKsiegiParams {
            chwila: "2026-11-05T08:00:00Z".to_string(),
            ..viewed_state()
        },
    );

    let extract_without_document = ZapiszStanKsiegiParams {
        chwila: "2026-11-06T08:00:00Z".to_string(),
        sposob: StanySposob::Odpis,
        ..viewed_state()
    };
    expect::refused(
        analityk_stanu_prawnego_ksiega::zapisz_stan_ksiegi(&room, &extract_without_document),
        "odpis_z_dokumentem",
    );

    let wrong_number = ZapiszStanKsiegiParams {
        numer_kw: "WA1M/123/6".to_string(),
        chwila: "2026-11-06T08:00:00Z".to_string(),
        ..viewed_state()
    };
    expect::refused(
        analityk_stanu_prawnego_ksiega::zapisz_stan_ksiegi(&room, &wrong_number),
        "CHECK constraint failed: numer_kw",
    );
}

/// nowy stan księgi pokazuje ją później niż poprzedni
#[test]
fn new_state_is_later() {
    let room = new_room();
    save_state(&room, &state());
    save_state(
        &room,
        &ZapiszStanKsiegiParams {
            numer_kw: "KR1P/00000001/1".to_string(),
            chwila: "2026-11-01T08:00:00Z".to_string(),
            ..state()
        },
    );

    let rule = "nowy stan księgi pokazuje ją później niż poprzedni";
    let earlier_state = ZapiszStanKsiegiParams {
        chwila: "2026-11-01T08:00:00Z".to_string(),
        ..viewed_state()
    };
    expect::refused(
        analityk_stanu_prawnego_ksiega::zapisz_stan_ksiegi(&room, &earlier_state),
        rule,
    );
    expect::refused(
        analityk_stanu_prawnego_ksiega::zapisz_stan_ksiegi(&room, &viewed_state()),
        REPLACEMENT,
    );
}

/// wpis księgi dopisuje się tylko do ostatniego stanu i tylko raz
#[test]
fn entries_go_to_latest_state() {
    let room = new_room();
    save_state(&room, &state());
    save_register_entry(&room, &register_entry());
    save_register_entry(
        &room,
        &ZapiszWpisKsiegiParams {
            rodzaj: WpisyRodzaj::Wzmianka,
            numer: "Dz.Kw./1/26".to_string(),
            przedmiot: "wniosek".to_string(),
            ..register_entry()
        },
    );

    expect::refused(
        analityk_stanu_prawnego_ksiega::zapisz_wpis_ksiegi(&room, &register_entry()),
        REPLACEMENT,
    );

    save_state(
        &room,
        &ZapiszStanKsiegiParams {
            chwila: "2026-11-05T08:00:00Z".to_string(),
            ..state()
        },
    );
    let entry_of_old_state = ZapiszWpisKsiegiParams {
        numer: "2".to_string(),
        ..register_entry()
    };
    expect::refused(
        analityk_stanu_prawnego_ksiega::zapisz_wpis_ksiegi(&room, &entry_of_old_state),
        "wpis księgi dopisuje się tylko do ostatniego stanu księgi",
    );
    save_register_entry(
        &room,
        &ZapiszWpisKsiegiParams {
            stan_id: 2,
            numer: "2".to_string(),
            ..register_entry()
        },
    );
}

/// stanu księgi ani jego wpisów się nie zmienia
#[test]
fn state_is_fixed() {
    let room = new_room();
    save_state(&room, &state());
    save_register_entry(&room, &register_entry());

    let rule = "stanu księgi ani jego wpisów się nie zmienia";
    expect::refused(
        room.execute(
            "UPDATE analityk_stanu_prawnego_ksiega_stany SET chwila = '2026-11-03T08:00:00Z'",
            &[],
        ),
        rule,
    );
    expect::refused(
        room.execute(
            "UPDATE analityk_stanu_prawnego_ksiega_wpisy SET przedmiot = 'hipoteka wykreślona'",
            &[],
        ),
        rule,
    );
}

// Checks ------------------------------------------------------------------------------------------

/// ryzyko i brak ustalenia mają opis
#[test]
fn risk_has_description() {
    let room = new_room();
    save_state(&room, &state());
    save_check(&room, &check("dzial-iv"));
    save_check(
        &room,
        &ZapiszSprawdzenieStanuPrawnegoParams {
            przedmiot: WpisyPrzedmiot::DzialIi,
            wynik: WpisyWynik::BezUwag,
            opis: None,
            ..check("dzial-ii")
        },
    );

    let unknown_without_description = ZapiszSprawdzenieStanuPrawnegoParams {
        przedmiot: WpisyPrzedmiot::DzialIii,
        wynik: WpisyWynik::NieMoznaUstalic,
        opis: None,
        ..check("dzial-iii")
    };
    expect::refused(
        analityk_stanu_prawnego_sprawdzenia::zapisz_sprawdzenie_stanu_prawnego(
            &room,
            &unknown_without_description,
        ),
        "ryzyko_z_opisem",
    );
}

/// sprawdzenie działu księgi stoi na stanie księgi
#[test]
fn register_section_needs_state() {
    let room = new_room();
    save_state(&room, &state());
    save_check(
        &room,
        &ZapiszSprawdzenieStanuPrawnegoParams {
            przedmiot: WpisyPrzedmiot::ProjektUmowy,
            ksiega_stan_id: None,
            ..check("projekt")
        },
    );

    let section_without_state = ZapiszSprawdzenieStanuPrawnegoParams {
        przedmiot: WpisyPrzedmiot::DzialI,
        ksiega_stan_id: None,
        ..check("dzial-i")
    };
    expect::refused(
        analityk_stanu_prawnego_sprawdzenia::zapisz_sprawdzenie_stanu_prawnego(
            &room,
            &section_without_state,
        ),
        "dzial_ksiegi_ze_stanem",
    );

    let missing_state = ZapiszSprawdzenieStanuPrawnegoParams {
        ksiega_stan_id: Some(7),
        ..check("dzial-iv")
    };
    expect::refused(
        analityk_stanu_prawnego_sprawdzenia::zapisz_sprawdzenie_stanu_prawnego(
            &room,
            &missing_state,
        ),
        "FOREIGN KEY constraint failed",
    );
}

/// dokument nie jest późniejszy od sprawdzenia
#[test]
fn document_not_after_check() {
    let room = new_room();
    save_state(&room, &state());
    save_check(
        &room,
        &ZapiszSprawdzenieStanuPrawnegoParams {
            dokument_data: "2026-11-03".to_string(),
            ..check("dzial-iv")
        },
    );

    let document_after_check = ZapiszSprawdzenieStanuPrawnegoParams {
        dokument_data: "2026-11-04".to_string(),
        ..check("dzial-iv-2")
    };
    expect::refused(
        analityk_stanu_prawnego_sprawdzenia::zapisz_sprawdzenie_stanu_prawnego(
            &room,
            &document_after_check,
        ),
        "dokument_nie_po_sprawdzeniu",
    );
}

/// sprawdzenie powstaje aktualne i bez analizy
#[test]
fn check_starts_current() {
    let room = new_room();
    save_state(&room, &state());
    save_check(&room, &check("dzial-iv"));

    let rule = "sprawdzenie powstaje aktualne i bez analizy";
    let columns_set_later = [
        ("analiza_odnosnik", "'pokoj-teczki-transakcji/analiza-1'"),
        ("zastapione_przez_id", "'dzial-iv'"),
    ];
    for (column, value) in columns_set_later {
        let check_with_later_column = format!(
            "INSERT INTO analityk_stanu_prawnego_sprawdzenia_wpisy (id, przedmiot,
              dokument_odnosnik, dokument_data, przepis, wynik, sprawdzono, {column})
            VALUES ('projekt', 'projekt_umowy', 'pokoj-teczki-transakcji/projekt-1', '2026-11-02',
              'art. 389 KC', 'bez_uwag', '2026-11-03T10:00:00Z', {value})"
        );
        expect::refused(room.execute(&check_with_later_column, &[]), rule);
    }
}

/// sprawdzenia się nie zmienia; analizę odnotowuje się raz
#[test]
fn check_is_fixed() {
    let room = new_room();
    save_state(&room, &state());
    save_check(&room, &check("dzial-iv"));

    let rule = "sprawdzenia się nie zmienia";
    let changes = [
        ("wynik", "'bez_uwag'"),
        ("opis", "'Inny opis'"),
        ("przepis", "'art. 66 ukwh'"),
        ("dokument_data", "'2026-11-01'"),
    ];
    for (column, value) in changes {
        let change = format!(
            "UPDATE analityk_stanu_prawnego_sprawdzenia_wpisy SET {column} = {value} \
            WHERE id = 'dzial-iv'"
        );
        expect::refused(room.execute(&change, &[]), rule);
    }

    record_analysis(&room, "dzial-iv", "pokoj-teczki-transakcji/analiza-1");
    let second_analysis = OdnotujAnalizeSprawdzeniaParams {
        analiza_odnosnik: "pokoj-teczki-transakcji/analiza-2".to_string(),
        id: "dzial-iv".to_string(),
    };
    expect::refused(
        analityk_stanu_prawnego_sprawdzenia::odnotuj_analize_sprawdzenia(&room, &second_analysis),
        rule,
    );
}

/// ponowne sprawdzenie tego samego przedmiotu, nie starsze, zastępuje poprzednie raz
#[test]
fn new_check_replaces_old() {
    let room = new_room();
    save_state(&room, &state());
    save_check(&room, &check("dzial-iv"));
    save_check(
        &room,
        &ZapiszSprawdzenieStanuPrawnegoParams {
            sprawdzono: "2026-11-04T10:00:00Z".to_string(),
            ..check("dzial-iv-2")
        },
    );
    save_check(
        &room,
        &ZapiszSprawdzenieStanuPrawnegoParams {
            sprawdzono: "2026-11-05T10:00:00Z".to_string(),
            ..check("dzial-iv-3")
        },
    );
    save_check(
        &room,
        &ZapiszSprawdzenieStanuPrawnegoParams {
            sprawdzono: "2026-11-02T10:00:00Z".to_string(),
            ..check("dzial-iv-dawny")
        },
    );
    save_check(
        &room,
        &ZapiszSprawdzenieStanuPrawnegoParams {
            przedmiot: WpisyPrzedmiot::DzialIii,
            sprawdzono: "2026-11-04T10:00:00Z".to_string(),
            ..check("dzial-iii")
        },
    );

    let rule = "sprawdzenie zastępuje ponowne sprawdzenie tego samego przedmiotu";
    expect::refused(
        analityk_stanu_prawnego_sprawdzenia::zastap_sprawdzenie_stanu_prawnego(
            &room,
            &replacement("dzial-iv", "dzial-iii"),
        ),
        rule,
    );
    expect::refused(
        analityk_stanu_prawnego_sprawdzenia::zastap_sprawdzenie_stanu_prawnego(
            &room,
            &replacement("dzial-iv", "dzial-iv-dawny"),
        ),
        rule,
    );
    expect::refused(
        analityk_stanu_prawnego_sprawdzenia::zastap_sprawdzenie_stanu_prawnego(
            &room,
            &replacement("dzial-iv", "dzial-iv"),
        ),
        "zastapione_nie_przez_siebie",
    );

    analityk_stanu_prawnego_sprawdzenia::zastap_sprawdzenie_stanu_prawnego(
        &room,
        &replacement("dzial-iv", "dzial-iv-2"),
    )
    .expect("replace the check");
    expect::refused(
        analityk_stanu_prawnego_sprawdzenia::zastap_sprawdzenie_stanu_prawnego(
            &room,
            &replacement("dzial-iv", "dzial-iv-3"),
        ),
        "sprawdzenia się nie zmienia",
    );
}

// R9 and replacement ------------------------------------------------------------------------------

/// The analyst's own tables in the room, by name.
fn own_tables(room: &MemoryRoom) -> Vec<String> {
    let rows = room
        .execute(
            r"SELECT name FROM sqlite_schema WHERE type = 'table'
            AND name LIKE 'analityk\_stanu\_prawnego\_%' ESCAPE '\' ORDER BY name",
            &[],
        )
        .expect("read the tables");

    rows.iter()
        .map(|row| row.get::<String>("name").expect("a table has a name"))
        .collect()
}

/// R9: wierszy się nie usuwa ani nie zastępuje przez REPLACE albo upsert
#[test]
fn no_deletion_or_replacement() {
    let room = new_room();
    save_state(&room, &state());
    save_register_entry(&room, &register_entry());
    save_check(&room, &check("dzial-iv"));

    let tables = own_tables(&room);
    assert_eq!(
        tables,
        [
            "analityk_stanu_prawnego_ksiega_stany",
            "analityk_stanu_prawnego_ksiega_wpisy",
            "analityk_stanu_prawnego_sprawdzenia_wpisy",
        ],
        "{tables:?}"
    );

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

/// REPLACE i upsert nie zastępują stanu ani wpisu księgi przez klucz jednoznaczny
#[test]
fn no_replacement_through_unique_keys() {
    let room = new_room();
    save_state(&room, &state());
    save_register_entry(&room, &register_entry());

    let state_again = format!(
        "INSERT OR REPLACE INTO analityk_stanu_prawnego_ksiega_stany (numer_kw, chwila, sposob)
        VALUES ('{NUMER_KW}', '2026-11-02T08:00:00Z', 'przegladanie')"
    );
    expect::refused(room.execute(&state_again, &[]), REPLACEMENT);
    expect::refused(
        room.execute(
            "INSERT INTO analityk_stanu_prawnego_ksiega_wpisy (stan_id, dzial, numer, rodzaj,
              przedmiot)
            VALUES (1, 'iv', '1', 'wpis', 'hipoteka wykreślona')
            ON CONFLICT (stan_id, dzial, rodzaj, numer) DO UPDATE SET przedmiot = excluded.przedmiot",
            &[],
        ),
        REPLACEMENT,
    );
}
