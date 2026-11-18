//! Scenarios of the rental head's memory room (spec/positions/najem/kierownik_najmu).
//!
//! The room has the head's own kind of areas, three kinds of the agent memory and all seven kinds
//! of the department memory; the rules of the shared kinds are checked in stanowisko.rs and
//! kierownik_dzialu.rs. Each scenario starts from an empty room and checks one rule the database
//! keeps: a write the rule allows and a write it refuses. The first scenario follows an area from
//! the head's proposal through a human's approval to its operations coordinator and its flats.

// The building blocks of kierownik_dzialu.rs take the types of the shared catalog's room.
use xboards_agencja_nieruchomosci::positions::kierownik_dzialu::kierownik_dzialu_agenci as shared_agenci;
use xboards_agencja_nieruchomosci::positions::najem::kierownik_najmu::{
    self, kierownik_dzialu_agenci, kierownik_najmu_obszary,
};
use xboards_agencja_nieruchomosci::xmeta_room::{FromSqlValue, MemoryRoom, SqlValue};

use kierownik_dzialu_agenci::WymienAgentaParams;
use kierownik_najmu_obszary::{
    LokaleObszaruParams, ObszarLokaluParams, ObszaryKoordynatoraParams,
    PrzypiszLokalDoObszaruParams, WskazKoordynatoraObszaruParams, ZakonczPrzynaleznoscLokaluParams,
    ZaproponujObszarParams, ZatwierdzObszarParams, ZmienPropozycjeObszaruParams, ZniesObszarParams,
};

use crate::expect;
use crate::kierownik_dzialu::{agent, appoint_agent};
use crate::stanowisko::{DELETION, REPLACEMENT, first_row_copy};

fn new_room() -> MemoryRoom {
    kierownik_najmu::open_in_memory().expect("open the room of the Head of rental")
}

/// A text value to bind in raw SQL.
fn text(value: &str) -> SqlValue {
    SqlValue::Text(value.to_string())
}

// Building blocks ---------------------------------------------------------------------------------

/// The default parameters of the head's proposal of an area: the Old Town of Wrocław.
fn area_proposal() -> ZaproponujObszarParams {
    ZaproponujObszarParams {
        nazwa: "Stare Miasto".to_string(),
        granice: "Wrocław, w granicach fosy miejskiej".to_string(),
        zaproponowano: "2026-11-02T09:00:00Z".to_string(),
    }
}

fn propose_area(room: &MemoryRoom, params: &ZaproponujObszarParams) {
    kierownik_najmu_obszary::zaproponuj_obszar(room, params).expect("propose the area");
}

fn change_area_proposal(room: &MemoryRoom, nazwa: &str, granice: &str) {
    let params = ZmienPropozycjeObszaruParams {
        nazwa: nazwa.to_string(),
        granice: granice.to_string(),
        id: 1,
    };

    kierownik_najmu_obszary::zmien_propozycje_obszaru(room, &params)
        .expect("change the proposal of the area");
}

/// Writes the owner's signed approval of the area.
fn approve_area(room: &MemoryRoom, area_id: i64) {
    let params = ZatwierdzObszarParams {
        zatwierdzil: "tozsamosc-wlasciciela".to_string(),
        podpis: "podpis-kluczem-wlasciciela".to_string(),
        zatwierdzono: "2026-11-02T12:00:00Z".to_string(),
        id: area_id,
    };

    kierownik_najmu_obszary::zatwierdz_obszar(room, &params).expect("approve the area");
}

/// The default parameters of an operations coordinator appointed by the owner, without a case file.
fn coordinator(agent_id: &str) -> shared_agenci::PowolajAgentaParams {
    shared_agenci::PowolajAgentaParams {
        stanowisko: shared_agenci::WpisyStanowisko::KoordynatorOperacji,
        wlasciciel: "tozsamosc-wlasciciela".to_string(),
        teczka_odnosnik: None,
        ..agent(agent_id)
    }
}

/// The parameters of pointing `koordynator_id` as the coordinator of the first area.
fn area_coordinator(koordynator_id: &str) -> WskazKoordynatoraObszaruParams {
    WskazKoordynatoraObszaruParams {
        koordynator_id: koordynator_id.to_string(),
        id: 1,
    }
}

fn point_area_coordinator(room: &MemoryRoom, koordynator_id: &str) {
    kierownik_najmu_obszary::wskaz_koordynatora_obszaru(room, &area_coordinator(koordynator_id))
        .expect("point the coordinator of the area");
}

fn replace_agent(room: &MemoryRoom, params: &WymienAgentaParams) {
    kierownik_dzialu_agenci::wymien_agenta(room, params).expect("replace the agent");
}

/// The default parameters of the first flat assigned to the first area.
fn flat() -> PrzypiszLokalDoObszaruParams {
    PrzypiszLokalDoObszaruParams {
        obszar_id: 1,
        teczka_odnosnik: "pokoj-lokalu-1".to_string(),
        od: "2026-11-03".to_string(),
    }
}

fn assign_flat(room: &MemoryRoom, params: &PrzypiszLokalDoObszaruParams) {
    kierownik_najmu_obszary::przypisz_lokal_do_obszaru(room, params)
        .expect("assign the flat to the area");
}

/// The parameters of ending the membership of the first flat in its area.
fn membership_end(do_dnia: &str) -> ZakonczPrzynaleznoscLokaluParams {
    ZakonczPrzynaleznoscLokaluParams {
        do_dnia: do_dnia.to_string(),
        id: 1,
    }
}

fn end_membership(room: &MemoryRoom, do_dnia: &str) {
    kierownik_najmu_obszary::zakoncz_przynaleznosc_lokalu(room, &membership_end(do_dnia))
        .expect("end the membership of the flat");
}

/// The parameters of closing the area `area_id`.
fn area_closure(area_id: i64, zniesiono: &str) -> ZniesObszarParams {
    ZniesObszarParams {
        zniesiono: zniesiono.to_string(),
        id: area_id,
    }
}

fn close_area(room: &MemoryRoom, area_id: i64, zniesiono: &str) {
    kierownik_najmu_obszary::znies_obszar(room, &area_closure(area_id, zniesiono))
        .expect("close the area");
}

/// One row in every table of the head's own kind, for the checks of R9 and replacement.
fn full_memory(room: &MemoryRoom) {
    propose_area(room, &area_proposal());
    approve_area(room, 1);
    assign_flat(room, &flat());
}

// A whole day -------------------------------------------------------------------------------------

/// kierownik proponuje obszar, człowiek go zatwierdza, a obszar dostaje Koordynatora i lokale
#[test]
fn whole_day() {
    let room = new_room();

    propose_area(&room, &area_proposal());
    change_area_proposal(
        &room,
        "Stare Miasto i Ostrów Tumski",
        "Wrocław, w granicach fosy miejskiej i Ostrów Tumski",
    );
    approve_area(&room, 1);
    appoint_agent(&room, &coordinator("koordynator-1"));
    point_area_coordinator(&room, "koordynator-1");
    assign_flat(&room, &flat());
    assign_flat(
        &room,
        &PrzypiszLokalDoObszaruParams {
            teczka_odnosnik: "pokoj-lokalu-2".to_string(),
            ..flat()
        },
    );

    appoint_agent(
        &room,
        &shared_agenci::PowolajAgentaParams {
            powolano: "2026-11-10T09:00:00Z".to_string(),
            ..coordinator("koordynator-2")
        },
    );
    replace_agent(
        &room,
        &WymienAgentaParams {
            wymieniono: "2026-11-10T09:00:00Z".to_string(),
            powod_wymiany: "Zlecał pracę wykonawcy spoza rejestru bez zagrożenia".to_string(),
            nastepca_id: "koordynator-2".to_string(),
            id: "koordynator-1".to_string(),
        },
    );
    point_area_coordinator(&room, "koordynator-2");

    let flats_of_area =
        kierownik_najmu_obszary::lokale_obszaru(&room, &LokaleObszaruParams { obszar_id: 1 })
            .expect("read the flats of the area");
    let flats_of_area_cases: Vec<&str> = flats_of_area
        .iter()
        .map(|row| row.teczka_odnosnik.as_str())
        .collect();
    assert_eq!(flats_of_area_cases, ["pokoj-lokalu-1", "pokoj-lokalu-2"]);

    let area_of_flat = kierownik_najmu_obszary::obszar_lokalu(
        &room,
        &ObszarLokaluParams {
            teczka_odnosnik: "pokoj-lokalu-2".to_string(),
        },
    )
    .expect("read the area of the flat");
    assert_eq!(
        area_of_flat[0].koordynator_id.as_deref(),
        Some("koordynator-2")
    );

    let flats_without_coordinator = kierownik_najmu_obszary::lokale_bez_koordynatora(&room)
        .expect("read the flats without a coordinator");
    assert!(flats_without_coordinator.is_empty());

    let areas_of_coordinator = kierownik_najmu_obszary::obszary_koordynatora(
        &room,
        &ObszaryKoordynatoraParams {
            koordynator_id: "koordynator-2".to_string(),
        },
    )
    .expect("read the areas of the coordinator");
    assert_eq!(areas_of_coordinator.len(), 1);
}

// Areas -------------------------------------------------------------------------------------------

/// obszar powstaje jako propozycja, a zatwierdzenie jest kompletne i nie przed propozycją
#[test]
fn area_starts_as_proposal() {
    let room = new_room();
    propose_area(&room, &area_proposal());

    expect::refused(
        room.execute(
            "INSERT INTO kierownik_najmu_obszary_wpisy (nazwa, granice, zaproponowano, zatwierdzil,
              podpis, zatwierdzono)
            VALUES ('Krzyki', 'g', '2026-11-02', 'w', 'p', '2026-11-02')",
            &[],
        ),
        "obszar powstaje jako propozycja kierownika",
    );

    // A writer outside the client can leave out the date that `zatwierdz_obszar` requires. The
    // table refuses the approval without one.
    let approval_without_date = room.execute(
        "UPDATE kierownik_najmu_obszary_wpisy
        SET zatwierdzil = :zatwierdzil,
          podpis = :podpis,
          zatwierdzono = :zatwierdzono
        WHERE id = :id",
        &[
            ("zatwierdzil", text("w")),
            ("podpis", text("p")),
            ("zatwierdzono", SqlValue::Null),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(approval_without_date, "zatwierdzenie_w_komplecie");

    let approval_before_proposal = ZatwierdzObszarParams {
        zatwierdzil: "w".to_string(),
        podpis: "p".to_string(),
        zatwierdzono: "2026-11-01T00:00:00Z".to_string(),
        id: 1,
    };
    expect::refused(
        kierownik_najmu_obszary::zatwierdz_obszar(&room, &approval_before_proposal),
        "zatwierdzenie_nie_przed_propozycja",
    );
    approve_area(&room, 1);
}

/// zatwierdzonych granic się nie zmienia, a zniesionego obszaru wcale
#[test]
fn approved_boundaries_do_not_change() {
    let room = new_room();
    propose_area(&room, &area_proposal());
    change_area_proposal(&room, "Stare Miasto", "Nowe granice");
    approve_area(&room, 1);

    let rule = "zatwierdzonych granic obszaru się nie zmienia";
    let change_after_approval = ZmienPropozycjeObszaruParams {
        nazwa: "Stare Miasto".to_string(),
        granice: "Inne granice".to_string(),
        id: 1,
    };
    expect::refused(
        kierownik_najmu_obszary::zmien_propozycje_obszaru(&room, &change_after_approval),
        rule,
    );

    let second_approval = ZatwierdzObszarParams {
        zatwierdzil: "ktos-inny".to_string(),
        podpis: "p".to_string(),
        zatwierdzono: "2026-11-03T00:00:00Z".to_string(),
        id: 1,
    };
    expect::refused(
        kierownik_najmu_obszary::zatwierdz_obszar(&room, &second_approval),
        rule,
    );

    expect::refused(
        kierownik_najmu_obszary::znies_obszar(&room, &area_closure(1, "2026-11-01T00:00:00Z")),
        "zniesienie_nie_przed_propozycja",
    );
    close_area(&room, 1, "2026-12-01T00:00:00Z");
    expect::refused(
        kierownik_najmu_obszary::znies_obszar(&room, &area_closure(1, "2026-12-02T00:00:00Z")),
        rule,
    );

    let areas = kierownik_najmu_obszary::obszary(&room).expect("read the areas");
    assert_eq!(areas[0].zniesiono.as_deref(), Some("2026-12-01T00:00:00Z"));
}

/// Koordynatorem jest niewymieniony Koordynator operacji, a obszar jest zatwierdzony
#[test]
fn coordinator_is_an_operations_coordinator() {
    let room = new_room();
    propose_area(&room, &area_proposal());
    appoint_agent(&room, &coordinator("koordynator-1"));
    appoint_agent(&room, &coordinator("koordynator-2"));
    appoint_agent(
        &room,
        &shared_agenci::PowolajAgentaParams {
            stanowisko: shared_agenci::WpisyStanowisko::ZarzadcaObiektu,
            teczka_odnosnik: Some("pokoj-lokalu-1".to_string()),
            ..agent("zarzadca-1")
        },
    );

    expect::refused(
        kierownik_najmu_obszary::wskaz_koordynatora_obszaru(
            &room,
            &area_coordinator("koordynator-1"),
        ),
        "koordynator_obszaru_zatwierdzonego",
    );
    approve_area(&room, 1);

    let rule = "koordynatorem obszaru jest niewymieniony agent na stanowisku Koordynatora operacji";
    expect::refused(
        kierownik_najmu_obszary::wskaz_koordynatora_obszaru(&room, &area_coordinator("zarzadca-1")),
        rule,
    );
    replace_agent(
        &room,
        &WymienAgentaParams {
            wymieniono: "2026-11-05T09:00:00Z".to_string(),
            powod_wymiany: "Zawiódł".to_string(),
            nastepca_id: "koordynator-2".to_string(),
            id: "koordynator-1".to_string(),
        },
    );
    expect::refused(
        kierownik_najmu_obszary::wskaz_koordynatora_obszaru(
            &room,
            &area_coordinator("koordynator-1"),
        ),
        rule,
    );
    point_area_coordinator(&room, "koordynator-2");
}

/// obszar znosi się dopiero, gdy nie ma w nim lokali
#[test]
fn area_is_closed_without_flats() {
    let room = new_room();
    propose_area(&room, &area_proposal());
    approve_area(&room, 1);
    assign_flat(&room, &flat());

    let rule = "obszar znosi się dopiero, gdy nie ma w nim lokali";
    expect::refused(
        kierownik_najmu_obszary::znies_obszar(&room, &area_closure(1, "2026-12-01T00:00:00Z")),
        rule,
    );
    end_membership(&room, "2026-11-30");
    close_area(&room, 1, "2026-12-01T00:00:00Z");
}

// Flats -------------------------------------------------------------------------------------------

/// lokal należy do zatwierdzonego, niezniesionego obszaru i naraz do jednego
#[test]
fn flat_in_one_approved_area() {
    let room = new_room();
    propose_area(&room, &area_proposal());
    propose_area(
        &room,
        &ZaproponujObszarParams {
            nazwa: "Krzyki".to_string(),
            granice: "Wrocław, Krzyki".to_string(),
            ..area_proposal()
        },
    );
    propose_area(
        &room,
        &ZaproponujObszarParams {
            nazwa: "Psie Pole".to_string(),
            granice: "Wrocław, Psie Pole".to_string(),
            ..area_proposal()
        },
    );
    approve_area(&room, 1);
    approve_area(&room, 3);
    close_area(&room, 3, "2026-11-03T00:00:00Z");
    assign_flat(&room, &flat());

    let rule = "lokal należy tylko do obszaru zatwierdzonego i niezniesionego";
    let flat_in_proposed_area = PrzypiszLokalDoObszaruParams {
        obszar_id: 2,
        teczka_odnosnik: "pokoj-lokalu-2".to_string(),
        od: "2026-11-03".to_string(),
    };
    expect::refused(
        kierownik_najmu_obszary::przypisz_lokal_do_obszaru(&room, &flat_in_proposed_area),
        rule,
    );

    let flat_in_closed_area = PrzypiszLokalDoObszaruParams {
        obszar_id: 3,
        ..flat_in_proposed_area
    };
    expect::refused(
        kierownik_najmu_obszary::przypisz_lokal_do_obszaru(&room, &flat_in_closed_area),
        rule,
    );

    approve_area(&room, 2);
    let flat_moved_to_second_area = PrzypiszLokalDoObszaruParams {
        obszar_id: 2,
        teczka_odnosnik: "pokoj-lokalu-1".to_string(),
        od: "2026-11-10".to_string(),
    };
    expect::refused(
        kierownik_najmu_obszary::przypisz_lokal_do_obszaru(&room, &flat_moved_to_second_area),
        "lokal należy naraz do jednego obszaru",
    );
    end_membership(&room, "2026-11-10");
    assign_flat(&room, &flat_moved_to_second_area);

    let area_of_flat = kierownik_najmu_obszary::obszar_lokalu(
        &room,
        &ObszarLokaluParams {
            teczka_odnosnik: "pokoj-lokalu-1".to_string(),
        },
    )
    .expect("read the area of the flat");
    assert_eq!(area_of_flat[0].nazwa, "Krzyki");

    let flats_without_coordinator = kierownik_najmu_obszary::lokale_bez_koordynatora(&room)
        .expect("read the flats without a coordinator");
    let flats_without_coordinator_cases: Vec<&str> = flats_without_coordinator
        .iter()
        .map(|row| row.teczka_odnosnik.as_str())
        .collect();
    assert_eq!(flats_without_coordinator_cases, ["pokoj-lokalu-1"]);
}

/// przynależność lokalu powstaje otwarta, kończy się raz i nie przed początkiem
#[test]
fn flat_membership_ends_once() {
    let room = new_room();
    propose_area(&room, &area_proposal());
    approve_area(&room, 1);
    assign_flat(&room, &flat());

    expect::refused(
        room.execute(
            "INSERT INTO kierownik_najmu_obszary_lokale (obszar_id, teczka_odnosnik, od, do_dnia)
            VALUES (1, 'pokoj-lokalu-2', '2026-11-03', '2026-11-04')",
            &[],
        ),
        "lokal powstaje w obszarze",
    );
    expect::refused(
        kierownik_najmu_obszary::zakoncz_przynaleznosc_lokalu(&room, &membership_end("2026-11-01")),
        "koniec_nie_przed_poczatkiem",
    );

    let rule = "przynależności lokalu do obszaru się nie zmienia";
    expect::refused(
        room.execute(
            "UPDATE kierownik_najmu_obszary_lokale SET obszar_id = 1, od = '2026-11-01'",
            &[],
        ),
        rule,
    );
    end_membership(&room, "2026-11-30");
    expect::refused(
        kierownik_najmu_obszary::zakoncz_przynaleznosc_lokalu(&room, &membership_end("2026-12-01")),
        rule,
    );
}

// R9 and replacement ------------------------------------------------------------------------------

/// The head's own tables in the room.
fn own_tables_of(room: &MemoryRoom) -> Vec<String> {
    let rows = room
        .execute(
            r"SELECT name FROM sqlite_schema WHERE type = 'table'
            AND name LIKE 'kierownik\_najmu\_%' ESCAPE '\'",
            &[],
        )
        .expect("read the tables");

    rows.iter()
        .map(|row| row.get::<String>("name").expect("a table has a name"))
        .collect()
}

/// R9: wierszy pamięci się nie usuwa ani nie zastępuje przez REPLACE albo upsert
#[test]
fn no_deletion_or_replacement() {
    let room = new_room();
    full_memory(&room);

    let tables = own_tables_of(&room);
    assert_eq!(tables.len(), 2, "{tables:?}");

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

/// REPLACE nie usuwa otwartej przynależności lokalu przez klucz UNIQUE
#[test]
fn no_replacement_through_open_membership() {
    let room = new_room();
    full_memory(&room);

    expect::refused(
        room.execute(
            "INSERT OR REPLACE INTO kierownik_najmu_obszary_lokale (id, obszar_id, teczka_odnosnik,
              od)
            SELECT 99, obszar_id, teczka_odnosnik, od FROM kierownik_najmu_obszary_lokale",
            &[],
        ),
        "lokal należy naraz do jednego obszaru",
    );
}
