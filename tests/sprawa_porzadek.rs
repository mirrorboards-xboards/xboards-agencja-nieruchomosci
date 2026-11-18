//! Scenarios of the core kinds that keep the case in order (spec/cases/sprawa): deadlines
//! (sprawa-terminy), hand-over notes (sprawa-notatki), links to other rooms (sprawa-powiazania),
//! and R9: rows of a case file are never deleted, except the corrections D14 allows.
//!
//! Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
//! allows and a write it refuses.

use xboards_agencja_nieruchomosci::cases::sprawa::{
    sprawa_decyzje, sprawa_notatki, sprawa_powiazania, sprawa_terminy,
};
use xboards_agencja_nieruchomosci::xmeta_room::{FromSqlValue, MemoryRoom, SqlValue};

use sprawa_decyzje::{WpisyRodzaj as RodzajDecyzji, WpisyRozstrzygniecie as Rozstrzygniecie};
use sprawa_notatki::{
    DopiszNotatkeParams, NotatkiOtwarteDoStanowiskaParams, OdrzucNotatkeParams,
    OznaczNotatkeJakoWykonanaParams, WpisyOdStanowiska as Stanowisko, WskazWierszNotatkiParams,
};
use sprawa_powiazania::{WpisyRodzaj as RodzajPowiazania, WpiszPowiazanieParams};
use sprawa_terminy::{
    OdnotujPotwierdzenieTerminuParams, PrzeliczTerminParams,
    WpisyPilnujeStanowisko as PilnujeStanowisko, WpisyRodzaj as RodzajTerminu,
    WpisyStan as StanTerminu, WpiszTerminParams, ZmienStanTerminuParams,
};

use crate::expect;
use crate::sprawa::{
    add_person, approved_decision, ask_for_decision, decision_request_about, full_case, new_room,
    person, settle_decision, settlement,
};

/// The single integer a read in raw SQL returns, e.g. a count.
fn integer_value(room: &MemoryRoom, sql: &str) -> i64 {
    let value = room.value(sql, &[]).expect("read the value");

    i64::from_sql_value(&value).expect("the value is an integer")
}

// Deadlines ---------------------------------------------------------------------------------------

/// The default parameters of a deadline: the statutory month to inform the seller about the data
/// taken from the buyer (art. 14 ust. 3 lit. a RODO).
fn deadline() -> WpiszTerminParams {
    WpiszTerminParams {
        id: "termin".to_string(),
        rodzaj: RodzajTerminu::Ustawowy,
        co: "Informacja o danych pozyskanych od kupującego".to_string(),
        zdarzenie_tabela: Some("sprawa_uczestnicy_wpisy".to_string()),
        zdarzenie_id: Some("sprzedajacy".to_string()),
        zdarzenie_chwila: Some("2026-11-02T10:00:00Z".to_string()),
        koniec: "2026-12-02".to_string(),
        podstawa: Some("art. 14 ust. 3 lit. a RODO".to_string()),
        pilnuje_stanowisko: PilnujeStanowisko::OpiekunPoszukujacego,
        kalendarz_odnosnik: None,
    }
}

/// Termin ustawowy ma podstawę i zdarzenie; termin powstaje biegnący
#[test]
fn deadline_statutory() {
    let room = new_room();

    let without_basis = WpiszTerminParams {
        podstawa: None,
        ..deadline()
    };
    expect::refused(
        sprawa_terminy::wpisz_termin(&room, &without_basis),
        "ustawowy_ma_podstawe_i_zdarzenie",
    );
    let without_event = WpiszTerminParams {
        zdarzenie_tabela: None,
        zdarzenie_id: None,
        ..deadline()
    };
    expect::refused(
        sprawa_terminy::wpisz_termin(&room, &without_event),
        "ustawowy_ma_podstawe_i_zdarzenie",
    );
    let event_table_without_id = WpiszTerminParams {
        zdarzenie_id: None,
        ..deadline()
    };
    expect::refused(
        sprawa_terminy::wpisz_termin(&room, &event_table_without_id),
        "zdarzenie_tabela_razem_z_id",
    );
    sprawa_terminy::wpisz_termin(&room, &deadline()).expect("add the statutory deadline");
    sprawa_terminy::wpisz_termin(
        &room,
        &WpiszTerminParams {
            id: "odpowiedz".to_string(),
            rodzaj: RodzajTerminu::Wewnetrzny,
            co: "Odpowiedź na wiadomość klienta".to_string(),
            zdarzenie_tabela: None,
            zdarzenie_id: None,
            zdarzenie_chwila: None,
            podstawa: None,
            ..deadline()
        },
    )
    .expect("add the internal deadline");

    expect::refused(
        room.execute(
            "INSERT INTO sprawa_terminy_wpisy (id, rodzaj, co, koniec, pilnuje_stanowisko, stan)
            VALUES ('minal', 'wewnetrzny', 'Raport', '2026-11-01', 'opiekun_oferty', 'minal')",
            &[],
        ),
        "termin powstaje jako biegnący i niepotwierdzony",
    );
    sprawa_terminy::zmien_stan_terminu(
        &room,
        &ZmienStanTerminuParams {
            stan: StanTerminu::Minal,
            id: "odpowiedz".to_string(),
        },
    )
    .expect("record the deadline as passed");

    let running = sprawa_terminy::terminy_biegnace(&room).expect("read the running deadlines");
    let running_ids: Vec<&str> = running.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(running_ids, ["termin"]);
}

/// Termin potwierdza tylko zatwierdzona decyzja potwierdzenie_terminu, która go wskazuje
#[test]
fn deadline_confirmed_by_decision() {
    let room = new_room();
    sprawa_terminy::wpisz_termin(
        &room,
        &WpiszTerminParams {
            id: "akt".to_string(),
            rodzaj: RodzajTerminu::ZUmowy,
            co: "Akt notarialny".to_string(),
            podstawa: Some("§ 2 umowy przedwstępnej".to_string()),
            pilnuje_stanowisko: PilnujeStanowisko::KoordynatorTransakcji,
            ..deadline()
        },
    )
    .expect("add the deadline of the deed");
    sprawa_terminy::wpisz_termin(
        &room,
        &WpiszTerminParams {
            id: "inny".to_string(),
            rodzaj: RodzajTerminu::ZUmowy,
            co: "Przekazanie lokalu".to_string(),
            ..deadline()
        },
    )
    .expect("add another deadline");
    let refusal =
        "termin potwierdza tylko zatwierdzona decyzja potwierdzenie_terminu, która go wskazuje";
    let confirmation_by = |decision_id: &str| OdnotujPotwierdzenieTerminuParams {
        potwierdzenie_decyzja_id: decision_id.to_string(),
        id: "akt".to_string(),
    };

    expect::refused(
        room.execute(
            "INSERT INTO sprawa_terminy_wpisy (id, rodzaj, co, koniec, pilnuje_stanowisko, stan,
              potwierdzenie_decyzja_id)
            VALUES ('od-razu', 'z_umowy', 'Akt', '2026-12-01', 'koordynator_transakcji', 'biegnie',
              'x')",
            &[],
        ),
        "termin powstaje jako biegnący i niepotwierdzony",
    );

    approved_decision(
        &room,
        &decision_request_about(
            "o-innym",
            RodzajDecyzji::PotwierdzenieTerminu,
            "sprawa_terminy_wpisy",
            "inny",
        ),
        "2026-11-03T12:00:00Z",
    );
    expect::refused(
        sprawa_terminy::odnotuj_potwierdzenie_terminu(&room, &confirmation_by("o-innym")),
        refusal,
    );

    ask_for_decision(
        &room,
        &decision_request_about(
            "odmowa",
            RodzajDecyzji::PotwierdzenieTerminu,
            "sprawa_terminy_wpisy",
            "akt",
        ),
    );
    settle_decision(
        &room,
        &settlement("odmowa", Rozstrzygniecie::Odmowa, "2026-11-03T12:00:00Z"),
    );
    expect::refused(
        sprawa_terminy::odnotuj_potwierdzenie_terminu(&room, &confirmation_by("odmowa")),
        refusal,
    );

    approved_decision(
        &room,
        &decision_request_about("inna", RodzajDecyzji::Inna, "sprawa_terminy_wpisy", "akt"),
        "2026-11-03T12:00:00Z",
    );
    expect::refused(
        sprawa_terminy::odnotuj_potwierdzenie_terminu(&room, &confirmation_by("inna")),
        refusal,
    );

    approved_decision(
        &room,
        &decision_request_about(
            "zgoda",
            RodzajDecyzji::PotwierdzenieTerminu,
            "sprawa_terminy_wpisy",
            "akt",
        ),
        "2026-11-03T12:00:00Z",
    );
    sprawa_terminy::odnotuj_potwierdzenie_terminu(&room, &confirmation_by("zgoda"))
        .expect("confirm the deadline");
}

/// Potwierdzony termin zmienia już tylko stan, a potwierdzenia się nie zeruje
#[test]
fn confirmed_deadline_frozen() {
    let room = new_room();
    sprawa_terminy::wpisz_termin(
        &room,
        &WpiszTerminParams {
            id: "akt".to_string(),
            rodzaj: RodzajTerminu::ZUmowy,
            co: "Akt notarialny".to_string(),
            podstawa: Some("§ 2".to_string()),
            ..deadline()
        },
    )
    .expect("add the deadline of the deed");
    sprawa_terminy::przelicz_termin(
        &room,
        &PrzeliczTerminParams {
            zdarzenie_tabela: None,
            zdarzenie_id: None,
            zdarzenie_chwila: None,
            koniec: "2026-12-02T10:00:00Z".to_string(),
            kalendarz_odnosnik: Some("kalendarz/akt".to_string()),
            id: "akt".to_string(),
        },
    )
    .expect("recalculate the deadline");
    approved_decision(
        &room,
        &decision_request_about(
            "zgoda",
            RodzajDecyzji::PotwierdzenieTerminu,
            "sprawa_terminy_wpisy",
            "akt",
        ),
        "2026-11-03T12:00:00Z",
    );
    sprawa_terminy::odnotuj_potwierdzenie_terminu(
        &room,
        &OdnotujPotwierdzenieTerminuParams {
            potwierdzenie_decyzja_id: "zgoda".to_string(),
            id: "akt".to_string(),
        },
    )
    .expect("confirm the deadline");
    let refusal = "potwierdzonego terminu się nie zmienia";

    // The workarounds from the audit: a new end and subject after the confirmation, and the
    // confirmation cleared.
    expect::refused(
        room.execute(
            "UPDATE sprawa_terminy_wpisy SET koniec = '2027-01-01', co = 'Inny akt'
            WHERE id = 'akt'",
            &[],
        ),
        refusal,
    );
    let new_end = PrzeliczTerminParams {
        zdarzenie_tabela: None,
        zdarzenie_id: None,
        zdarzenie_chwila: None,
        koniec: "2027-01-01".to_string(),
        kalendarz_odnosnik: None,
        id: "akt".to_string(),
    };
    expect::refused(sprawa_terminy::przelicz_termin(&room, &new_end), refusal);
    // A writer outside the client can leave out the decision that `odnotuj_potwierdzenie_terminu`
    // requires. The table refuses to clear the confirmation.
    expect::refused(
        room.execute(
            "UPDATE sprawa_terminy_wpisy
            SET potwierdzenie_decyzja_id = :potwierdzenie_decyzja_id
            WHERE id = 'akt'",
            &[("potwierdzenie_decyzja_id", SqlValue::Null)],
        ),
        refusal,
    );

    sprawa_terminy::zmien_stan_terminu(
        &room,
        &ZmienStanTerminuParams {
            stan: StanTerminu::Zbedny,
            id: "akt".to_string(),
        },
    )
    .expect("record the deadline as unneeded");
}

// Hand-over notes ---------------------------------------------------------------------------------

/// The default parameters of a hand-over note from Kontroler to the transaction coordinator, due
/// on 2026-11-07.
fn note() -> DopiszNotatkeParams {
    DopiszNotatkeParams {
        id: "notatka".to_string(),
        od: "tozsamosc-kontrolera".to_string(),
        od_stanowiska: Stanowisko::Kontroler,
        do_stanowiska: Stanowisko::KoordynatorTransakcji,
        do_klienta_uczestnik_id: None,
        tresc: "Potrzebny odpis z KRS spółki z dnia aktu.".to_string(),
        utworzono: "2026-11-05T09:00:00Z".to_string(),
        do_kiedy: Some("2026-11-07".to_string()),
        w_odpowiedzi_na: None,
    }
}

/// Notatka powstaje otwarta, jej treść się nie zmienia, a stan zmienia się raz
#[test]
fn note_rules() {
    let room = new_room();

    expect::refused(
        room.execute(
            "INSERT INTO sprawa_notatki_wpisy (id, od, od_stanowiska, do_stanowiska, tresc,
              utworzono, stan)
            VALUES ('od-razu', 'k', 'kontroler', 'opiekun_oferty', 'Treść', '2026-11-05T09:00:00Z',
              'wykonana')",
            &[],
        ),
        "notatka powstaje jako otwarta",
    );

    sprawa_notatki::dopisz_notatke(&room, &note()).expect("add the note");
    sprawa_notatki::wskaz_wiersz_notatki(
        &room,
        &WskazWierszNotatkiParams {
            notatka_id: "notatka".to_string(),
            dotyczy_tabela: "sprawa_umowy_wpisy".to_string(),
            dotyczy_id: "u".to_string(),
        },
    )
    .expect("point the note to a row");
    expect::refused(
        room.execute("UPDATE sprawa_notatki_wpisy SET tresc = 'Inna treść'", &[]),
        "treść notatki się nie zmienia",
    );
    expect::refused(
        room.execute("UPDATE sprawa_notatki_wiersze SET dotyczy_id = 'v'", &[]),
        "wiersza notatki się nie zmienia",
    );

    expect::refused(
        room.execute("UPDATE sprawa_notatki_wpisy SET stan = 'odrzucona'", &[]),
        "odrzucona_ma_powod",
    );
    sprawa_notatki::odrzuc_notatke(
        &room,
        &OdrzucNotatkeParams {
            powod: "Odpis z KRS pobiera Analityk stanu prawnego.".to_string(),
            id: "notatka".to_string(),
        },
    )
    .expect("reject the note");
    let rejected_done = OznaczNotatkeJakoWykonanaParams {
        id: "notatka".to_string(),
    };
    expect::refused(
        sprawa_notatki::oznacz_notatke_jako_wykonana(&room, &rejected_done),
        "stan notatki zmienia się raz",
    );

    sprawa_notatki::dopisz_notatke(
        &room,
        &DopiszNotatkeParams {
            id: "odpowiedz".to_string(),
            w_odpowiedzi_na: Some("notatka".to_string()),
            od_stanowiska: Stanowisko::KoordynatorTransakcji,
            do_stanowiska: Stanowisko::AnalitykStanuPrawnego,
            ..note()
        },
    )
    .expect("add the answer");
    sprawa_notatki::oznacz_notatke_jako_wykonana(
        &room,
        &OznaczNotatkeJakoWykonanaParams {
            id: "odpowiedz".to_string(),
        },
    )
    .expect("mark the answer as done");

    let open_notes = sprawa_notatki::notatki_otwarte_do_stanowiska(
        &room,
        &NotatkiOtwarteDoStanowiskaParams {
            do_stanowiska: Stanowisko::AnalitykStanuPrawnego,
        },
    )
    .expect("read the open notes");
    assert!(open_notes.is_empty());
}

/// Notatka wskazuje klienta tylko wtedy, gdy idzie do Kontrolera (R1)
#[test]
fn note_to_controller_of_client() {
    let room = new_room();
    add_person(&room, &person("kupujacy"));

    let to_offer_agent = DopiszNotatkeParams {
        od_stanowiska: Stanowisko::KoordynatorTransakcji,
        do_stanowiska: Stanowisko::OpiekunOferty,
        do_klienta_uczestnik_id: Some("kupujacy".to_string()),
        ..note()
    };
    expect::refused(
        sprawa_notatki::dopisz_notatke(&room, &to_offer_agent),
        "klient_tylko_przy_kontrolerze",
    );
    sprawa_notatki::dopisz_notatke(
        &room,
        &DopiszNotatkeParams {
            od_stanowiska: Stanowisko::KoordynatorTransakcji,
            do_stanowiska: Stanowisko::Kontroler,
            do_klienta_uczestnik_id: Some("kupujacy".to_string()),
            ..note()
        },
    )
    .expect("add the note to Kontroler");

    let open_notes = sprawa_notatki::notatki_otwarte_do_stanowiska(
        &room,
        &NotatkiOtwarteDoStanowiskaParams {
            do_stanowiska: Stanowisko::Kontroler,
        },
    )
    .expect("read the open notes");
    assert!(!open_notes.is_empty());
}

// Links -------------------------------------------------------------------------------------------

/// Powiązania się nie zmienia; nowe powiązanie to nowy wiersz
#[test]
fn link_rules() {
    let room = new_room();

    sprawa_powiazania::wpisz_powiazanie(
        &room,
        &WpiszPowiazanieParams {
            rodzaj: RodzajPowiazania::PokojWymiany,
            typ_teczki: None,
            oznaczenie: None,
            odnosnik: "pokoj-wymiany-kancelaria-7".to_string(),
            wpisal: "tozsamosc-koordynatora".to_string(),
            wpisano: "2026-11-05T09:00:00Z".to_string(),
        },
    )
    .expect("add the link to the exchange room");
    let capitalised_case_type = WpiszPowiazanieParams {
        rodzaj: RodzajPowiazania::SprawaZrodlowa,
        typ_teczki: Some("Oferta".to_string()),
        oznaczenie: Some("Oferta 2026/041".to_string()),
        odnosnik: "pokoj-oferty-041".to_string(),
        wpisal: "tozsamosc-kierownika".to_string(),
        wpisano: "2026-11-05T09:00:00Z".to_string(),
    };
    expect::refused(
        sprawa_powiazania::wpisz_powiazanie(&room, &capitalised_case_type),
        "typ_teczki",
    );
    expect::refused(
        room.execute(
            "UPDATE sprawa_powiazania_wpisy SET odnosnik = 'inny-pokoj'",
            &[],
        ),
        "powiązania się nie zmienia",
    );
}

// R9: no deletion ---------------------------------------------------------------------------------

/// D14: a mistaken address, citizenship or plot may be deleted, because the room's XSql journal
/// keeps the previous state with its author.
const CORRECTABLE_TABLES: [&str; 3] = [
    "sprawa_nieruchomosci_dzialki",
    "sprawa_uczestnicy_adresy",
    "sprawa_uczestnicy_obywatelstwa",
];

/// The tables of the core, whose names start with `sprawa_`.
fn tables_of_core(room: &MemoryRoom) -> Vec<String> {
    let rows = room
        .execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'sprawa\\_%' ESCAPE '\\'",
            &[],
        )
        .expect("read the tables");

    rows.iter()
        .map(|row| row.get::<String>("name").expect("a table has a name"))
        .collect()
}

/// R9: wierszy teczki się nie usuwa; wyjątkiem są adresy, obywatelstwa i działki (D14)
#[test]
fn no_deletion() {
    let room = new_room();
    full_case(&room);

    let tables = tables_of_core(&room);
    assert_eq!(tables.len(), 31, "{tables:?}");

    for table in &tables {
        let rows = integer_value(&room, &format!("SELECT count(*) FROM {table}"));
        assert!(rows > 0, "{table} has no row to delete");

        if CORRECTABLE_TABLES.contains(&table.as_str()) {
            continue;
        }

        let sql = format!("DELETE FROM {table} WHERE rowid = (SELECT min(rowid) FROM {table})");
        expect::refused(room.execute(&sql, &[]), "wierszy teczki się nie usuwa (R9)");
    }

    for table in CORRECTABLE_TABLES {
        room.execute(&format!("DELETE FROM {table}"), &[])
            .expect("delete the rows of a correctable table");

        let rows = integer_value(&room, &format!("SELECT count(*) FROM {table}"));
        assert_eq!(rows, 0, "{table}");
    }
}

/// SQL, without its leading `INSERT`, that inserts again the first row of `table` with the
/// `columns`.
fn copy_of_first_row(table: &str, columns: &[String]) -> String {
    let column_list = columns.join(", ");

    format!(
        "INTO {table} ({column_list}) SELECT {column_list} FROM {table} \
        WHERE rowid = (SELECT min(rowid) FROM {table})"
    )
}

/// The names of the columns of `table`, from its schema.
fn columns_of(room: &MemoryRoom, table: &str) -> Vec<String> {
    let rows = room
        .execute(&format!("PRAGMA table_info({table})"), &[])
        .expect("read the columns of the table");

    rows.iter()
        .map(|row| row.get::<String>("name").expect("a column has a name"))
        .collect()
}

/// SQL, without its leading `INSERT` or `INSERT OR REPLACE`, that inserts again the first row of
/// `table`, as a careless upsert would; its `id` collides.
pub(crate) fn first_row_copy(room: &MemoryRoom, table: &str) -> String {
    let columns = columns_of(room, table);

    copy_of_first_row(table, &columns)
}

/// [`first_row_copy`] that leaves the `id` out, so only a UNIQUE key collides (Python:
/// `first_row_copy(room, table, replace_unique_key=True)`).
pub(crate) fn first_row_copy_without_id(room: &MemoryRoom, table: &str) -> String {
    let columns: Vec<String> = columns_of(room, table)
        .into_iter()
        .filter(|column| column != "id")
        .collect();

    copy_of_first_row(table, &columns)
}

/// The names of the UNIQUE indexes of `table` other than its primary key, from its schema.
pub(crate) fn unique_keys_of(room: &MemoryRoom, table: &str) -> Vec<String> {
    let indexes = room
        .execute(&format!("PRAGMA index_list({table})"), &[])
        .expect("read the indexes of the table");

    let mut unique_keys = Vec::new();
    for index in &indexes {
        let unique = index
            .get::<i64>("unique")
            .expect("an index says if it is unique");
        let origin = index
            .get::<String>("origin")
            .expect("an index has an origin");

        let declared_unique = unique == 1 && origin == "u";
        if declared_unique {
            unique_keys.push(index.get::<String>("name").expect("an index has a name"));
        }
    }

    unique_keys
}

/// Wiersza teczki nie zastępuje INSERT OR REPLACE ani upsert; wyjątkiem są poprawki D14
#[test]
fn no_replacement() {
    let room = new_room();
    full_case(&room);

    let tables = tables_of_core(&room);
    let refusal = "wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE";

    for table in &tables {
        let rows = integer_value(&room, &format!("SELECT count(*) FROM {table}"));
        assert!(rows > 0, "{table} has no row to replace");

        if CORRECTABLE_TABLES.contains(&table.as_str()) {
            continue;
        }

        // The workaround from the audit: REPLACE deletes the old row without the triggers of
        // DELETE and UPDATE, and an upsert updates it without the triggers of UPDATE.
        let replace = format!("INSERT OR REPLACE {}", first_row_copy(&room, table));
        expect::refused(room.execute(&replace, &[]), refusal);

        let upsert = format!(
            "INSERT {} ON CONFLICT (id) DO UPDATE SET id = excluded.id",
            first_row_copy(&room, table)
        );
        expect::refused(room.execute(&upsert, &[]), refusal);

        if !unique_keys_of(&room, table).is_empty() {
            let replace_by_unique_key = format!(
                "INSERT OR REPLACE {}",
                first_row_copy_without_id(&room, table)
            );
            expect::refused(room.execute(&replace_by_unique_key, &[]), refusal);
        }
    }

    // Corrections D14 may replace their rows.
    for table in CORRECTABLE_TABLES {
        let rows_before = integer_value(&room, &format!("SELECT count(*) FROM {table}"));

        let replace = format!("INSERT OR REPLACE {}", first_row_copy(&room, table));
        room.execute(&replace, &[])
            .expect("replace a row of a correctable table");
        let upsert = format!(
            "INSERT {} ON CONFLICT (id) DO UPDATE SET id = excluded.id",
            first_row_copy(&room, table)
        );
        room.execute(&upsert, &[])
            .expect("upsert a row of a correctable table");

        let rows_after = integer_value(&room, &format!("SELECT count(*) FROM {table}"));
        assert_eq!(rows_after, rows_before, "{table}");
    }
}
