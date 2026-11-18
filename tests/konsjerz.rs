//! Scenarios of the Concierge's memory room (spec/positions/najem/konsjerz).
//!
//! The room is the Concierge's own kinds and the shared agent kinds it imports. The shared kinds
//! are checked in stanowisko.rs; here each scenario starts from an empty room and checks one rule
//! of konsjerz-pytania or konsjerz-reguly: a write the rule allows and a write it refuses. The
//! first scenario follows a season of a flat: guests ask about the garage, the answer helps, a
//! better answer replaces it, the Concierge proposes a change of the flat's description and
//! assesses a kind of report instead of a guest.

use std::collections::HashMap;

use xboards_agencja_nieruchomosci::positions::najem::konsjerz::{
    self, konsjerz_pytania, stanowisko_zrodla as zrodla_konsjerza,
};
use xboards_agencja_nieruchomosci::positions::stanowisko::stanowisko_zrodla::{
    DopiszZdarzenieZrodlaParams, WpisyRodzaj as RodzajZrodla, WpiszZrodloParams, ZdarzeniaRodzaj,
};
use xboards_agencja_nieruchomosci::xmeta_room::{MemoryRoom, MetaRoom, SqlValue};

use konsjerz_pytania::{
    DopiszWystapieniePytaniaParams, OdnotujCzyOdpowiedzPomoglaParams, OdnotujPropozycjeOpisuParams,
    ZapiszPytanieParams, ZastapOdpowiedzParams,
};

use crate::expect;
use crate::stanowisko::{
    DELETION, REPLACEMENT, add_source, add_source_event, first_row_copy, source, source_event,
};

fn new_room() -> MemoryRoom {
    konsjerz::open_in_memory().expect("open the room of the Concierge")
}

/// A text value to bind in raw SQL.
fn text(value: &str) -> SqlValue {
    SqlValue::Text(value.to_string())
}

// Building blocks ---------------------------------------------------------------------------------

/// The default parameters of a guests' question about the garage with its answer.
fn question() -> ZapiszPytanieParams {
    ZapiszPytanieParams {
        temat: "wjazd do garażu".to_string(),
        odpowiedz: "Wjazd jest od ul. Bocznej; pilot leży w szufladzie przy drzwiach".to_string(),
        zapisano: "2026-07-01T10:00:00Z".to_string(),
    }
}

fn save_question(room: &impl MetaRoom, params: &ZapiszPytanieParams) {
    konsjerz_pytania::zapisz_pytanie(room, params).expect("save the question");
}

fn add_occurrence(room: &MemoryRoom, pytanie_id: i64, zadano: &str) {
    let params = DopiszWystapieniePytaniaParams {
        pytanie_id,
        zadano: zadano.to_string(),
    };

    konsjerz_pytania::dopisz_wystapienie_pytania(room, &params)
        .expect("add an occurrence of the question");
}

fn record_whether_answer_helped(room: &MemoryRoom, occurrence_id: i64, pomogla: bool) {
    let params = OdnotujCzyOdpowiedzPomoglaParams {
        pomogla,
        id: occurrence_id,
    };

    konsjerz_pytania::odnotuj_czy_odpowiedz_pomogla(room, &params)
        .expect("record whether the answer helped");
}

fn replace_answer(room: &MemoryRoom, question_id: i64, zastapione_przez_id: i64) {
    let params = ZastapOdpowiedzParams {
        zastapione_przez_id,
        id: question_id,
    };

    konsjerz_pytania::zastap_odpowiedz(room, &params).expect("replace the answer");
}

/// The source of the kind of report about missing hot water.
fn missing_water_source() -> WpiszZrodloParams {
    WpiszZrodloParams {
        nazwa: Some("zgłoszenie braku ciepłej wody".to_string()),
        rodzaj: RodzajZrodla::RodzajZgloszenia,
        ..source("brak-wody")
    }
}

// A whole season ----------------------------------------------------------------------------------

/// sezon lokalu: pytania gości, lepsza odpowiedź, propozycja opisu i ocena zgłoszeń
#[test]
fn whole_season() {
    let room = new_room();

    save_question(&room, &question());
    add_occurrence(&room, 1, "2026-07-02T18:00:00Z");
    add_occurrence(&room, 1, "2026-07-09T20:00:00Z");
    record_whether_answer_helped(&room, 1, false);
    record_whether_answer_helped(&room, 2, false);
    save_question(
        &room,
        &ZapiszPytanieParams {
            odpowiedz: "Wjazd od ul. Bocznej, druga brama za sklepem; pilot przy drzwiach"
                .to_string(),
            zapisano: "2026-07-10T09:00:00Z".to_string(),
            ..question()
        },
    );
    replace_answer(&room, 1, 2);
    add_occurrence(&room, 2, "2026-07-12T17:00:00Z");
    record_whether_answer_helped(&room, 3, true);
    save_question(
        &room,
        &ZapiszPytanieParams {
            temat: "ciepła woda".to_string(),
            odpowiedz: "Wspólnota wyłącza wodę we wtorki 8–10".to_string(),
            zapisano: "2026-07-12T09:00:00Z".to_string(),
        },
    );
    konsjerz_pytania::odnotuj_propozycje_opisu(
        &room,
        &OdnotujPropozycjeOpisuParams {
            propozycja_przekazano: "2026-07-15T10:00:00Z".to_string(),
            notatka_odnosnik: "pokoj-teczki-lokalu/notatka-12".to_string(),
            id: 2,
        },
    )
    .expect("record the proposal of the description");

    add_source(&room, &missing_water_source());
    add_source_event(
        &room,
        &DopiszZdarzenieZrodlaParams {
            opis: "Zgłoszony brak wody był wyłączeniem przez wspólnotę".to_string(),
            ..source_event("brak-wody", ZdarzeniaRodzaj::Rozbieznosc)
        },
    );
    zrodla_konsjerza::ocen_zrodlo(
        &room,
        &zrodla_konsjerza::OcenZrodloParams {
            ocena: zrodla_konsjerza::Ocena::Zastrzezenia,
            oceniono: "2026-07-16T10:00:00Z".to_string(),
            id: "brak-wody".to_string(),
        },
    )
    .expect("assess the source");

    let rows = konsjerz_pytania::pytania_gosci(&room).expect("read the guests' questions");
    let questions: HashMap<&str, (i64, i64, i64)> = rows
        .iter()
        .map(|row| (row.temat.as_str(), (row.id, row.wystapienia, row.pomocne)))
        .collect();
    assert_eq!(
        questions,
        HashMap::from([("wjazd do garażu", (2, 1, 1)), ("ciepła woda", (3, 0, 0))])
    );
}

// Questions ---------------------------------------------------------------------------------------

/// pytanie powstaje aktualne i bez propozycji
#[test]
fn question_starts_current() {
    let room = new_room();
    save_question(&room, &question());

    let rule = "pytanie powstaje aktualne i bez propozycji";
    let later_columns = [
        (
            "propozycja_przekazano, notatka_odnosnik",
            "'2026-07-02', 'pokoj-teczki-lokalu/notatka-1'",
        ),
        ("zastapione_przez_id", "1"),
    ];
    for (columns, values) in later_columns {
        let insert = format!(
            "INSERT INTO konsjerz_pytania_wpisy (temat, odpowiedz, zapisano, {columns})
            VALUES ('parking', 'Parking pod budynkiem', '2026-07-01', {values})"
        );
        expect::refused(room.execute(&insert, &[]), rule);
    }
}

/// propozycja ma notatkę, nie poprzedza zapisu i odnotowuje się ją raz
#[test]
fn proposal_has_note() {
    let room = new_room();
    save_question(&room, &question());
    save_question(
        &room,
        &ZapiszPytanieParams {
            temat: "parking".to_string(),
            odpowiedz: "Parking pod budynkiem".to_string(),
            ..question()
        },
    );

    // A writer outside the client can leave out the note that `odnotuj_propozycje_opisu` requires.
    // The table refuses the proposal without one.
    let proposal_without_note = room.execute(
        "UPDATE konsjerz_pytania_wpisy
        SET propozycja_przekazano = :propozycja_przekazano, notatka_odnosnik = :notatka_odnosnik
        WHERE id = :id",
        &[
            ("propozycja_przekazano", text("2026-07-03T10:00:00Z")),
            ("notatka_odnosnik", SqlValue::Null),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(proposal_without_note, "propozycja_z_notatka");
    expect::refused(
        konsjerz_pytania::odnotuj_propozycje_opisu(
            &room,
            &OdnotujPropozycjeOpisuParams {
                propozycja_przekazano: "2026-06-30T10:00:00Z".to_string(),
                notatka_odnosnik: "pokoj-teczki-lokalu/notatka-1".to_string(),
                id: 1,
            },
        ),
        "propozycja_nie_przed_zapisem",
    );
    konsjerz_pytania::odnotuj_propozycje_opisu(
        &room,
        &OdnotujPropozycjeOpisuParams {
            propozycja_przekazano: "2026-07-03T10:00:00Z".to_string(),
            notatka_odnosnik: "pokoj-teczki-lokalu/notatka-1".to_string(),
            id: 1,
        },
    )
    .expect("record the proposal of the description");
    expect::refused(
        konsjerz_pytania::odnotuj_propozycje_opisu(
            &room,
            &OdnotujPropozycjeOpisuParams {
                propozycja_przekazano: "2026-07-04T10:00:00Z".to_string(),
                notatka_odnosnik: "pokoj-teczki-lokalu/notatka-2".to_string(),
                id: 1,
            },
        ),
        "pytania ani odpowiedzi się nie zmienia",
    );
}

/// pytania ani odpowiedzi się nie zmienia; lepsza odpowiedź to nowy wiersz
#[test]
fn answer_is_fixed() {
    let room = new_room();
    save_question(&room, &question());

    let rule = "pytania ani odpowiedzi się nie zmienia";
    let changes = [
        ("temat", "'parking'"),
        ("odpowiedz", "'Inna odpowiedź'"),
        ("zapisano", "'2026-06-01'"),
    ];
    for (column, value) in changes {
        let update = format!("UPDATE konsjerz_pytania_wpisy SET {column} = {value} WHERE id = 1");
        expect::refused(room.execute(&update, &[]), rule);
    }
}

/// nowsza odpowiedź na ten sam temat zastępuje poprzednią raz
#[test]
fn newer_answer_replaces() {
    let room = new_room();
    save_question(&room, &question());
    save_question(
        &room,
        &ZapiszPytanieParams {
            odpowiedz: "Lepsza odpowiedź".to_string(),
            zapisano: "2026-07-10T09:00:00Z".to_string(),
            ..question()
        },
    );
    save_question(
        &room,
        &ZapiszPytanieParams {
            odpowiedz: "Jeszcze lepsza".to_string(),
            zapisano: "2026-07-20T09:00:00Z".to_string(),
            ..question()
        },
    );
    save_question(
        &room,
        &ZapiszPytanieParams {
            odpowiedz: "Dawna".to_string(),
            zapisano: "2026-06-01T09:00:00Z".to_string(),
            ..question()
        },
    );
    save_question(
        &room,
        &ZapiszPytanieParams {
            temat: "parking".to_string(),
            odpowiedz: "Parking".to_string(),
            zapisano: "2026-07-10T09:00:00Z".to_string(),
        },
    );

    let rule = "pytanie zastępuje nowsza odpowiedź na ten sam temat";
    expect::refused(
        konsjerz_pytania::zastap_odpowiedz(
            &room,
            &ZastapOdpowiedzParams {
                zastapione_przez_id: 5,
                id: 1,
            },
        ),
        rule,
    );
    expect::refused(
        konsjerz_pytania::zastap_odpowiedz(
            &room,
            &ZastapOdpowiedzParams {
                zastapione_przez_id: 4,
                id: 1,
            },
        ),
        rule,
    );
    expect::refused(
        konsjerz_pytania::zastap_odpowiedz(
            &room,
            &ZastapOdpowiedzParams {
                zastapione_przez_id: 1,
                id: 1,
            },
        ),
        "zastapione_nie_przez_siebie",
    );

    replace_answer(&room, 1, 2);
    expect::refused(
        konsjerz_pytania::zastap_odpowiedz(
            &room,
            &ZastapOdpowiedzParams {
                zastapione_przez_id: 3,
                id: 1,
            },
        ),
        "pytania ani odpowiedzi się nie zmienia",
    );
}

// Occurrences -------------------------------------------------------------------------------------

/// wystąpienie dotyczy aktualnej odpowiedzi i nie poprzedza jej zapisania
#[test]
fn occurrence_of_current_answer() {
    let room = new_room();
    save_question(&room, &question());
    save_question(
        &room,
        &ZapiszPytanieParams {
            odpowiedz: "Lepsza odpowiedź".to_string(),
            zapisano: "2026-07-10T09:00:00Z".to_string(),
            ..question()
        },
    );
    add_occurrence(&room, 1, "2026-07-02T18:00:00Z");

    let rule = "wystąpienie dotyczy aktualnej odpowiedzi";
    expect::refused(
        konsjerz_pytania::dopisz_wystapienie_pytania(
            &room,
            &DopiszWystapieniePytaniaParams {
                pytanie_id: 2,
                zadano: "2026-07-09T09:00:00Z".to_string(),
            },
        ),
        rule,
    );
    replace_answer(&room, 1, 2);
    expect::refused(
        konsjerz_pytania::dopisz_wystapienie_pytania(
            &room,
            &DopiszWystapieniePytaniaParams {
                pytanie_id: 1,
                zadano: "2026-07-11T09:00:00Z".to_string(),
            },
        ),
        rule,
    );
    add_occurrence(&room, 2, "2026-07-11T09:00:00Z");
}

/// wystąpienia tylko się dopisuje; czy odpowiedź pomogła, odnotowuje się raz
#[test]
fn occurrences_are_appended() {
    let room = new_room();
    save_question(&room, &question());
    add_occurrence(&room, 1, "2026-07-02T18:00:00Z");

    expect::refused(
        room.execute(
            "INSERT INTO konsjerz_pytania_wystapienia (pytanie_id, zadano, pomogla)
            VALUES (1, '2026-07-03T09:00:00Z', 1)",
            &[],
        ),
        "wystąpienie powstaje bez oceny",
    );

    let rule = "wystąpienia tylko się dopisuje";
    expect::refused(
        room.execute(
            "UPDATE konsjerz_pytania_wystapienia SET zadano = '2026-07-05T09:00:00Z' WHERE id = 1",
            &[],
        ),
        rule,
    );
    record_whether_answer_helped(&room, 1, true);
    expect::refused(
        konsjerz_pytania::odnotuj_czy_odpowiedz_pomogla(
            &room,
            &OdnotujCzyOdpowiedzPomoglaParams {
                pomogla: false,
                id: 1,
            },
        ),
        rule,
    );
}

// Rules on the shared memory ----------------------------------------------------------------------

/// D14: Konsjerż ocenia rodzaj zgłoszenia, a nie gościa
#[test]
fn source_is_not_a_guest() {
    let room = new_room();
    add_source(&room, &missing_water_source());
    add_source(
        &room,
        &WpiszZrodloParams {
            nazwa: Some("Wspólnota Mieszkaniowa Polna 7".to_string()),
            rodzaj: RodzajZrodla::Instytucja,
            ..source("wspolnota")
        },
    );

    let rule = "Konsjerż ocenia rodzaj zgłoszenia, a nie gościa";
    expect::refused(
        zrodla_konsjerza::wpisz_zrodlo(
            &room,
            &zrodla_konsjerza::WpiszZrodloParams {
                id: "gosc".to_string(),
                nazwa: Some("Gość z pobytu 12".to_string()),
                rodzaj: zrodla_konsjerza::WpisyRodzaj::Osoba,
                wpis_odnosnik: None,
                tozsamosc: None,
            },
        ),
        rule,
    );
    expect::refused(
        zrodla_konsjerza::wpisz_zrodlo(
            &room,
            &zrodla_konsjerza::WpiszZrodloParams {
                id: "gosc-z-teczki".to_string(),
                nazwa: None,
                rodzaj: zrodla_konsjerza::WpisyRodzaj::Osoba,
                wpis_odnosnik: Some("pokoj-teczki-lokalu/pobyt-12".to_string()),
                tozsamosc: None,
            },
        ),
        rule,
    );
}

// R9 and replacement ------------------------------------------------------------------------------

/// R9: wierszy się nie usuwa ani nie zastępuje przez REPLACE albo upsert
#[test]
fn no_deletion_or_replacement() {
    let room = new_room();
    save_question(&room, &question());
    add_occurrence(&room, 1, "2026-07-02T18:00:00Z");

    let rows = room
        .execute(
            r"SELECT name FROM sqlite_schema WHERE type = 'table'
            AND name LIKE 'konsjerz\_%' ESCAPE '\' ORDER BY name",
            &[],
        )
        .expect("read the tables");
    let tables: Vec<String> = rows
        .iter()
        .map(|row| row.get::<String>("name").expect("a table has a name"))
        .collect();
    assert_eq!(
        tables,
        ["konsjerz_pytania_wpisy", "konsjerz_pytania_wystapienia"]
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
