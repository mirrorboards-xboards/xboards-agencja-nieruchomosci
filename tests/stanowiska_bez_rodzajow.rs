//! Scenarios of the memory rooms of the positions without kinds of their own.
//!
//! Seven positions keep in their memory only what the shared agent memory offers; what else their
//! profiles name lies in the case file (D3). Their position has no `meta/`; its `board.toml` only
//! imports the shared memory. The rules of the shared kinds are checked in stanowisko.rs; here each
//! room is built from its catalog and checked to hold exactly the kinds it declares, one row in
//! every table. The kinds a room declares are `KINDS` of its module, which the generator writes
//! from the `meta` list of the position's `board.toml`.

use std::path::Path;

use xboards_agencja_nieruchomosci::positions::{najem, posrednictwo, transakcje};
use xboards_agencja_nieruchomosci::xmeta_room::{FromSqlValue, Kind, MemoryRoom};

use crate::stanowisko::full_memory;

/// The kinds of the shared agent memory each position imports. Every position below needs
/// contacts outside the case file, so it imports stanowisko-kontakty as well.
const DECLARED_KINDS: [&str; 4] = [
    "stanowisko-kontakty",
    "stanowisko-polecenia",
    "stanowisko-ustalenia",
    "stanowisko-zrodla",
];

const TABLES: [&str; 5] = [
    "stanowisko_kontakty_wpisy",
    "stanowisko_polecenia_wpisy",
    "stanowisko_ustalenia_wpisy",
    "stanowisko_zrodla_wpisy",
    "stanowisko_zrodla_zdarzenia",
];

/// The tables of a room, by name.
fn tables_of(room: &MemoryRoom) -> Vec<String> {
    let rows = room
        .execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' ORDER BY name",
            &[],
        )
        .expect("read the tables of the room");

    rows.iter()
        .map(|row| row.get::<String>("name").expect("a table has a name"))
        .collect()
}

/// Checks that the position has no kinds of its own, that its room holds exactly the declared
/// kinds and their tables, and that full memory puts one row in every table.
#[track_caller]
fn room_has_declared_kinds(position: &str, room: &MemoryRoom, kinds: &[&Kind]) {
    let mut declared: Vec<&str> = kinds.iter().map(|kind| kind.name).collect();
    declared.sort();
    assert_eq!(declared, DECLARED_KINDS, "{position}");

    let position_folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../spec/positions")
        .join(position);
    let is_position = position_folder.join("board.toml").is_file();
    assert!(
        is_position,
        "{position} has no board.toml, so it is no position"
    );
    let has_own_kinds = position_folder.join("meta").exists();
    assert!(!has_own_kinds, "{position} has a meta/ folder of its own");

    let tables = tables_of(room);
    assert_eq!(tables, TABLES, "{position}");

    full_memory(room);
    for table in &tables {
        let row_count = room
            .value(&format!("SELECT count(*) FROM {table}"), &[])
            .expect("count the rows");
        let row_count = i64::from_sql_value(&row_count).expect("the count is a number");
        assert_eq!(row_count, 1, "{position}: {table}");
    }
}

/// pokój posrednictwo/opiekun_oferty ma tylko zadeklarowane rodzaje pamięci wspólnej i się buduje
#[test]
fn opiekun_oferty() {
    use posrednictwo::opiekun_oferty as position;

    let room = position::open_in_memory().expect("open the room");
    room_has_declared_kinds("posrednictwo/opiekun_oferty", &room, position::KINDS);
}

/// pokój posrednictwo/opiekun_poszukujacego ma tylko zadeklarowane rodzaje pamięci wspólnej i się
/// buduje
#[test]
fn opiekun_poszukujacego() {
    use posrednictwo::opiekun_poszukujacego as position;

    let room = position::open_in_memory().expect("open the room");
    room_has_declared_kinds("posrednictwo/opiekun_poszukujacego", &room, position::KINDS);
}

/// pokój transakcje/analityk_kredytowy ma tylko zadeklarowane rodzaje pamięci wspólnej i się buduje
#[test]
fn analityk_kredytowy() {
    use transakcje::analityk_kredytowy as position;

    let room = position::open_in_memory().expect("open the room");
    room_has_declared_kinds("transakcje/analityk_kredytowy", &room, position::KINDS);
}

/// pokój transakcje/koordynator_transakcji ma tylko zadeklarowane rodzaje pamięci wspólnej i się
/// buduje
#[test]
fn koordynator_transakcji() {
    use transakcje::koordynator_transakcji as position;

    let room = position::open_in_memory().expect("open the room");
    room_has_declared_kinds("transakcje/koordynator_transakcji", &room, position::KINDS);
}

/// pokój najem/opiekun_najmu ma tylko zadeklarowane rodzaje pamięci wspólnej i się buduje
#[test]
fn opiekun_najmu() {
    use najem::opiekun_najmu as position;

    let room = position::open_in_memory().expect("open the room");
    room_has_declared_kinds("najem/opiekun_najmu", &room, position::KINDS);
}

/// pokój najem/zarzadca_obiektu ma tylko zadeklarowane rodzaje pamięci wspólnej i się buduje
#[test]
fn zarzadca_obiektu() {
    use najem::zarzadca_obiektu as position;

    let room = position::open_in_memory().expect("open the room");
    room_has_declared_kinds("najem/zarzadca_obiektu", &room, position::KINDS);
}

/// pokój najem/koordynator_operacji ma tylko zadeklarowane rodzaje pamięci wspólnej i się buduje
#[test]
fn koordynator_operacji() {
    use najem::koordynator_operacji as position;

    let room = position::open_in_memory().expect("open the room");
    room_has_declared_kinds("najem/koordynator_operacji", &room, position::KINDS);
}
