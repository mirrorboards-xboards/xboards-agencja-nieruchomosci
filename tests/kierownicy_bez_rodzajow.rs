//! Scenarios of the memory rooms of positions without kinds of their own.
//!
//! Such a position's room is only the shared kinds its board.toml imports by name: three or four
//! kinds of the agent memory and, for a department head, the kinds of the department memory it
//! keeps. Their rules are checked in stanowisko.rs and kierownik_dzialu.rs, so here each room is
//! checked to build and to hold exactly the declared kinds, no more (imports are not transitive)
//! and no fewer. The transaction head keeps no policy copies (D11), so its room has no
//! kierownik-dzialu-polisy.

use std::path::Path;

// The building blocks of kierownik_dzialu.rs take the types of the shared catalog's room.
use xboards_agencja_nieruchomosci::positions::kierownik_dzialu::{
    kierownik_dzialu_agenci as shared_agenci, kierownik_dzialu_raporty as shared_raporty,
    kierownik_dzialu_spis as shared_spis,
};
use xboards_agencja_nieruchomosci::positions::transakcje::kierownik_transakcji::{
    self, kierownik_dzialu_agenci, kierownik_dzialu_raporty, kierownik_dzialu_spis,
};
use xboards_agencja_nieruchomosci::xmeta_room::{MemoryRoom, RoomError};

use kierownik_dzialu_agenci::AgenciTeczkiParams;

use crate::kierownik_dzialu::{
    add_case_to_list, agent, appoint_agent, case_in_list, controller, controller_let_in,
    forward_request, let_in, receive_report, report, request,
};

/// The folder of the positions' specs, beside the tests.
const POSITIONS_FOLDER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../spec/positions");

/// Opens a position's room in memory.
type OpenRoom = fn() -> Result<MemoryRoom, RoomError>;

/// The kinds each room must hold, as its board.toml declares them.
const DECLARED_KINDS: [(&str, OpenRoom, &[&str]); 1] = [(
    "transakcje/kierownik_transakcji",
    kierownik_transakcji::open_in_memory,
    &[
        "stanowisko-polecenia",
        "stanowisko-ustalenia",
        "stanowisko-zrodla",
        "kierownik-dzialu-agenci",
        "kierownik-dzialu-raporty",
        "kierownik-dzialu-spis",
        "kierownik-dzialu-standardy",
        "kierownik-dzialu-zajrzenia",
        "kierownik-dzialu-zwyczaje",
    ],
)];

/// Every shared kind a room could import, to tell which of them a room holds.
const SHARED_KINDS: [&str; 11] = [
    "stanowisko-kontakty",
    "stanowisko-polecenia",
    "stanowisko-ustalenia",
    "stanowisko-zrodla",
    "kierownik-dzialu-agenci",
    "kierownik-dzialu-polisy",
    "kierownik-dzialu-raporty",
    "kierownik-dzialu-spis",
    "kierownik-dzialu-standardy",
    "kierownik-dzialu-zajrzenia",
    "kierownik-dzialu-zwyczaje",
];

fn transaction_head_room() -> MemoryRoom {
    kierownik_transakcji::open_in_memory().expect("open the room of the Head of transactions")
}

/// The shared kinds whose tables the room has.
fn kinds_in(room: &MemoryRoom) -> Vec<&'static str> {
    let rows = room
        .execute("SELECT name FROM sqlite_schema WHERE type = 'table'", &[])
        .expect("read the tables of the room");
    let tables: Vec<String> = rows
        .iter()
        .map(|row| row.get::<String>("name").expect("a table has a name"))
        .collect();

    SHARED_KINDS
        .into_iter()
        .filter(|kind| {
            let table_prefix = format!("{}_", kind.replace('-', "_"));

            tables.iter().any(|table| table.starts_with(&table_prefix))
        })
        .collect()
}

/// pokój każdego stanowiska bez rodzajów własnych ma dokładnie zadeklarowane rodzaje
#[test]
fn rooms_hold_the_declared_kinds() {
    for (position, open_room, declared) in DECLARED_KINDS {
        let room = open_room().expect("open the room");

        let kinds = kinds_in(&room);
        assert_eq!(kinds, declared, "{position}");

        let meta_folder = Path::new(POSITIONS_FOLDER).join(position).join("meta");
        assert!(!meta_folder.exists(), "{position}");
    }
}

/// Kierownik transakcji powołuje agenta, wpuszcza Kontrolera i przekazuje prośbę i spis
#[test]
fn transaction_head_day() {
    let room = transaction_head_room();

    appoint_agent(
        &room,
        &shared_agenci::PowolajAgentaParams {
            stanowisko: shared_agenci::WpisyStanowisko::KoordynatorTransakcji,
            wlasciciel: "tozsamosc-wlasciciela".to_string(),
            teczka_odnosnik: Some("pokoj-transakcji-1".to_string()),
            ..agent("koordynator-1")
        },
    );
    appoint_agent(&room, &controller("kontroler-sprzedajacej"));
    let_in(
        &room,
        &shared_agenci::OdnotujWpuszczenieParams {
            tozsamosc: "tozsamosc-kontroler-sprzedajacej".to_string(),
            agent_id: Some("kontroler-sprzedajacej".to_string()),
            pokoj_odnosnik: "pokoj-transakcji-1".to_string(),
            wpuscil: "tozsamosc-kierownika-transakcji".to_string(),
            ..controller_let_in()
        },
    );
    receive_report(
        &room,
        &shared_raporty::PrzyjmijRaportParams {
            etap_albo_okres: "umowa przedwstępna".to_string(),
            decyzja_odnosnik: Some("pokoj-transakcji-1/decyzja-1".to_string()),
            ..report("raport-1", "koordynator-1")
        },
    );
    forward_request(
        &room,
        &shared_raporty::PrzekazProsbeParams {
            decyzja_odnosnik: "pokoj-transakcji-1/decyzja-1".to_string(),
            ..request()
        },
    );
    add_case_to_list(
        &room,
        &shared_spis::WpiszSpraweDoSpisuParams {
            teczka_odnosnik: "pokoj-transakcji-1".to_string(),
            rodzaj: shared_spis::SprawyRodzaj::Transakcja,
            oznaczenie: "Transakcja 2026/12, ul. Polna 7 m. 12".to_string(),
            agent: "tozsamosc-koordynator-1".to_string(),
            kancelaria_xnotariusz: Some(true),
            ..case_in_list()
        },
    );

    let agents_of_case = kierownik_dzialu_agenci::agenci_teczki(
        &room,
        &AgenciTeczkiParams {
            teczka_odnosnik: "pokoj-transakcji-1".to_string(),
        },
    )
    .expect("read the agents of the case file");
    let agents_of_case_ids: Vec<&str> = agents_of_case.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(agents_of_case_ids, ["koordynator-1"]);

    let open_let_ins =
        kierownik_dzialu_agenci::wpuszczenia_otwarte(&room).expect("read the open let-ins");
    assert_eq!(open_let_ins.len(), 1);

    let requests = kierownik_dzialu_raporty::prosby_na_liscie(&room).expect("read the requests");
    assert_eq!(requests.len(), 1);

    let cases_for_reception = kierownik_dzialu_spis::do_przekazania_recepcji(&room)
        .expect("read the cases to hand to Reception");
    assert_eq!(cases_for_reception[0].kancelaria_xnotariusz, Some(true));
}
