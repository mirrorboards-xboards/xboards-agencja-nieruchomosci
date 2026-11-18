//! Scenarios of the memory every agent keeps (spec/positions/stanowisko).
//!
//! The catalog goes into the memory room of every position, so its rules are checked once here, in
//! a room of the shared catalog alone. Each scenario starts from an empty room and checks one rule
//! the database keeps: a write the rule allows and a write it refuses. The first scenario follows
//! an agent that learns from a source, finds out the source lied, assesses its findings again and
//! keeps the contacts of its case.
//!
//! The building blocks marked `pub(crate)` are shared with the scenarios of the positions: their
//! rooms hold the same kinds, so the operations of this room write there too.

use std::collections::{HashMap, HashSet};

use xboards_agencja_nieruchomosci::positions::stanowisko::{
    self, stanowisko_kontakty, stanowisko_polecenia, stanowisko_ustalenia, stanowisko_zrodla,
};
use xboards_agencja_nieruchomosci::xmeta_room::{FromSqlValue, MemoryRoom, MetaRoom, SqlValue};

use stanowisko_kontakty::{
    KontaktySprawyParams, UniewaznijKontaktParams, WpisyKanal, ZapiszKontaktParams,
};
use stanowisko_polecenia::{
    OdmowPoleceniaParams, OdnotujWykonaniePoleceniaParams, PrzyjmijPolecenieParams, WpisyWydalJako,
};
use stanowisko_ustalenia::{
    OcenUstalenieNaNowoParams, OdnotujWykorzystanieUstaleniaParams, UniewaznijUstalenieParams,
    UstaleniaOParams, UstaleniaOdZrodlaParams, WpisyPewnosc, ZapiszUstalenieParams,
};
use stanowisko_zrodla::{
    DopiszZdarzenieZrodlaParams, OcenZrodloParams, Ocena, OdnotujZgloszenieZdarzeniaParams,
    WpisyRodzaj as RodzajZrodla, WpiszZrodloParams, ZdarzeniaRodzaj, ZdarzeniaZrodelRow,
};

use crate::expect;

/// The refusal of a write that would replace a row of memory (INSERT OR REPLACE, upsert).
pub(crate) const REPLACEMENT: &str = "wiersza pamięci nie zastępuje się nowym wpisem";

/// The refusal of a deleted row of memory (R9).
pub(crate) const DELETION: &str = "wierszy pamięci się nie usuwa (R9)";

fn new_room() -> MemoryRoom {
    stanowisko::open_in_memory().expect("open the room of the shared memory")
}

/// A text value to bind in raw SQL.
fn text(value: &str) -> SqlValue {
    SqlValue::Text(value.to_string())
}

// Building blocks, also for the scenarios of the positions ----------------------------------------

/// The default parameters of a source: a housing cooperative with a name.
pub(crate) fn source(source_id: &str) -> WpiszZrodloParams {
    WpiszZrodloParams {
        id: source_id.to_string(),
        nazwa: Some("Spółdzielnia Mieszkaniowa Leśna".to_string()),
        rodzaj: RodzajZrodla::Instytucja,
        wpis_odnosnik: None,
        tozsamosc: None,
    }
}

/// Writes a source into the agent's memory.
pub(crate) fn add_source(room: &impl MetaRoom, params: &WpiszZrodloParams) {
    stanowisko_zrodla::wpisz_zrodlo(room, params).expect("add the source");
}

/// The default parameters of an event of a source, described by its kind.
pub(crate) fn source_event(
    source_id: &str,
    rodzaj: ZdarzeniaRodzaj,
) -> DopiszZdarzenieZrodlaParams {
    DopiszZdarzenieZrodlaParams {
        zrodlo_id: source_id.to_string(),
        zaszlo: "2026-11-05T10:00:00Z".to_string(),
        rodzaj,
        opis: format!("Zdarzenie {}", rodzaj.as_str()),
        dotyczy_tabela: None,
        dotyczy_odnosnik: None,
    }
}

/// Appends an event to a source's track record.
pub(crate) fn add_source_event(room: &impl MetaRoom, params: &DopiszZdarzenieZrodlaParams) {
    stanowisko_zrodla::dopisz_zdarzenie_zrodla(room, params).expect("add the source's event");
}

/// The default parameters of a probable finding that follows from a document in a case file.
fn finding(finding_id: &str) -> ZapiszUstalenieParams {
    ZapiszUstalenieParams {
        id: finding_id.to_string(),
        tresc: "Lokal ma uregulowany stan prawny w spółdzielni".to_string(),
        zrodlo_id: None,
        wynika_z_tabela: Some("sprawa_dokumenty_wpisy".to_string()),
        wynika_z_odnosnik: Some("pokoj-teczki/zaswiadczenie-spoldzielni".to_string()),
        dotyczy_tabela: None,
        dotyczy_odnosnik: None,
        pewnosc: WpisyPewnosc::Prawdopodobne,
        sposob_sprawdzenia: None,
        ustalono: "2026-11-04T09:00:00Z".to_string(),
    }
}

fn add_finding(room: &impl MetaRoom, params: &ZapiszUstalenieParams) {
    stanowisko_ustalenia::zapisz_ustalenie(room, params).expect("add the finding");
}

/// The default parameters of an order the owner signed.
pub(crate) fn order() -> PrzyjmijPolecenieParams {
    PrzyjmijPolecenieParams {
        wydal: "tozsamosc-wlasciciela".to_string(),
        wydal_jako: WpisyWydalJako::Wlasciciel,
        otrzymano: "2026-11-04T08:00:00Z".to_string(),
        tresc: "Raportuj codziennie o 16:00".to_string(),
        podpis: "podpis-kluczem-wlasciciela".to_string(),
    }
}

/// Writes a signed order the agent accepted.
pub(crate) fn accept_order(room: &impl MetaRoom, params: &PrzyjmijPolecenieParams) {
    stanowisko_polecenia::przyjmij_polecenie(room, params).expect("accept the order");
}

/// The default parameters of a named contact of a case: the administrator of a housing community.
fn contact() -> ZapiszKontaktParams {
    ZapiszKontaktParams {
        dotyczy_odnosnik: Some("pokoj-teczki".to_string()),
        instytucja: Some("Wspólnota Mieszkaniowa Polna 7".to_string()),
        rola: "administrator".to_string(),
        wpis_odnosnik: None,
        nazwa: Some("Administracja Polna 7".to_string()),
        adres: Some("administracja@polna7.example".to_string()),
        kanal: WpisyKanal::Email,
        pory: Some("dni robocze 8–12".to_string()),
        zapisano: "2026-11-04T09:00:00Z".to_string(),
    }
}

fn add_contact(room: &impl MetaRoom, params: &ZapiszKontaktParams) {
    stanowisko_kontakty::zapisz_kontakt(room, params).expect("add the contact");
}

/// Writes one row into every table of the catalog (an order, a source, its event, a finding and a
/// contact), for the checks of R9 and replacement.
pub(crate) fn full_memory(room: &impl MetaRoom) {
    accept_order(room, &order());
    add_source(room, &source("spoldzielnia"));
    add_source_event(
        room,
        &source_event("spoldzielnia", ZdarzeniaRodzaj::Potwierdzenie),
    );
    add_finding(
        room,
        &ZapiszUstalenieParams {
            zrodlo_id: Some("spoldzielnia".to_string()),
            ..finding("stan-prawny")
        },
    );
    add_contact(room, &contact());
}

fn assess_source(room: &MemoryRoom, source_id: &str, ocena: Ocena, oceniono: &str) {
    let params = OcenZrodloParams {
        ocena,
        oceniono: oceniono.to_string(),
        id: source_id.to_string(),
    };

    stanowisko_zrodla::ocen_zrodlo(room, &params).expect("assess the source");
}

fn reassess_finding(room: &MemoryRoom, finding_id: &str, pewnosc: WpisyPewnosc) {
    let params = OcenUstalenieNaNowoParams {
        pewnosc,
        sposob_sprawdzenia: None,
        id: finding_id.to_string(),
    };

    stanowisko_ustalenia::ocen_ustalenie_na_nowo(room, &params).expect("assess the finding again");
}

fn record_report_of_event(room: &MemoryRoom, event_id: i64, zgloszono: &str) {
    let params = OdnotujZgloszenieZdarzeniaParams {
        zgloszono: zgloszono.to_string(),
        id: event_id,
    };

    stanowisko_zrodla::odnotuj_zgloszenie_zdarzenia(room, &params)
        .expect("record the report of the event");
}

fn record_order_done(room: &MemoryRoom, order_id: i64, zamknieto: &str) {
    let params = OdnotujWykonaniePoleceniaParams {
        zamknieto: zamknieto.to_string(),
        uwagi: None,
        id: order_id,
    };

    stanowisko_polecenia::odnotuj_wykonanie_polecenia(room, &params)
        .expect("record the order as done");
}

fn invalidate_contact(room: &MemoryRoom, contact_id: i64, nieaktualny_od: &str) {
    let params = UniewaznijKontaktParams {
        nieaktualny_od: nieaktualny_od.to_string(),
        id: contact_id,
    };

    stanowisko_kontakty::uniewaznij_kontakt(room, &params).expect("invalidate the contact");
}

// A whole day -------------------------------------------------------------------------------------

/// agent ustala ze źródła, po nieprawdzie ocenia źródło i ustalenia na nowo
#[test]
fn whole_day() {
    let room = new_room();

    accept_order(&room, &order());
    add_source(&room, &source("spoldzielnia"));
    add_source(
        &room,
        &WpiszZrodloParams {
            nazwa: None,
            rodzaj: RodzajZrodla::Osoba,
            wpis_odnosnik: Some("pokoj-teczki/uczestnik-1".to_string()),
            ..source("sprzedajaca")
        },
    );
    add_finding(
        &room,
        &ZapiszUstalenieParams {
            tresc: "Lokal nie ma zaległości".to_string(),
            zrodlo_id: Some("spoldzielnia".to_string()),
            ..finding("zaleglosci")
        },
    );
    add_finding(
        &room,
        &ZapiszUstalenieParams {
            zrodlo_id: Some("sprzedajaca".to_string()),
            tresc: "Lokal ma 52 m²".to_string(),
            dotyczy_tabela: Some("sprawa_metryka_wpis".to_string()),
            dotyczy_odnosnik: Some("pokoj-teczki/metryka".to_string()),
            pewnosc: WpisyPewnosc::Potwierdzone,
            sposob_sprawdzenia: Some("pomiar w protokole oględzin".to_string()),
            ..finding("metraz")
        },
    );
    stanowisko_ustalenia::odnotuj_wykorzystanie_ustalenia(
        &room,
        &OdnotujWykorzystanieUstaleniaParams {
            wykorzystano_tabela: "sprawa_notatki_wpisy".to_string(),
            wykorzystano_odnosnik: "pokoj-teczki/notatka-1".to_string(),
            id: "metraz".to_string(),
        },
    )
    .expect("record where the finding was used");
    add_contact(&room, &contact());

    add_source_event(
        &room,
        &DopiszZdarzenieZrodlaParams {
            opis: "Zaświadczenie pominęło zaległość".to_string(),
            ..source_event("spoldzielnia", ZdarzeniaRodzaj::Nieprawda)
        },
    );
    assess_source(
        &room,
        "spoldzielnia",
        Ocena::Nierzetelne,
        "2026-11-05T11:00:00Z",
    );

    let to_assess = stanowisko_ustalenia::ustalenia_od_zrodla(
        &room,
        &UstaleniaOdZrodlaParams {
            zrodlo_id: "spoldzielnia".to_string(),
        },
    )
    .expect("read the findings from the source");
    let to_assess_ids: Vec<&str> = to_assess.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(to_assess_ids, ["zaleglosci"]);
    reassess_finding(&room, "zaleglosci", WpisyPewnosc::Niepewne);

    add_finding(
        &room,
        &ZapiszUstalenieParams {
            tresc: "Lokal ma zaległość 1 200 zł".to_string(),
            wynika_z_odnosnik: Some("pokoj-teczki/zaswiadczenie-2".to_string()),
            ustalono: "2026-11-06T09:00:00Z".to_string(),
            ..finding("zaleglosci-nowe")
        },
    );
    stanowisko_ustalenia::uniewaznij_ustalenie(
        &room,
        &UniewaznijUstalenieParams {
            nieaktualne_od: "2026-11-06T09:00:00Z".to_string(),
            zastapione_przez_id: Some("zaleglosci-nowe".to_string()),
            id: "zaleglosci".to_string(),
        },
    )
    .expect("replace the finding");

    add_contact(
        &room,
        &ZapiszKontaktParams {
            adres: Some("nowa-administracja@polna7.example".to_string()),
            zapisano: "2026-11-06T10:00:00Z".to_string(),
            ..contact()
        },
    );
    invalidate_contact(&room, 1, "2026-11-06T10:00:00Z");

    let events_to_report =
        stanowisko_zrodla::zdarzenia_do_zgloszenia(&room).expect("read the events to report");
    for event in events_to_report {
        record_report_of_event(&room, event.id, "2026-11-06T16:00:00Z");
    }
    record_order_done(&room, 1, "2026-11-06T16:00:00Z");

    let findings = stanowisko_ustalenia::ustalenia(&room).expect("read the findings");
    let finding_ids: HashSet<&str> = findings.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(finding_ids, HashSet::from(["metraz", "zaleglosci-nowe"]));

    let findings_about_area = stanowisko_ustalenia::ustalenia_o(
        &room,
        &UstaleniaOParams {
            dotyczy_odnosnik: "pokoj-teczki/metryka".to_string(),
        },
    )
    .expect("read the findings about the area");
    let findings_about_area_ids: Vec<&str> = findings_about_area
        .iter()
        .map(|row| row.id.as_str())
        .collect();
    assert_eq!(findings_about_area_ids, ["metraz"]);

    let sources = stanowisko_zrodla::zrodla(&room).expect("read the sources");
    assert_eq!(sources[0].id, "spoldzielnia");

    let open_orders = stanowisko_polecenia::polecenia_otwarte(&room).expect("read open orders");
    assert!(open_orders.is_empty());

    let case_contacts = stanowisko_kontakty::kontakty_sprawy(
        &room,
        &KontaktySprawyParams {
            dotyczy_odnosnik: "pokoj-teczki".to_string(),
        },
    )
    .expect("read the contacts of the case");
    let case_contact_addresses: Vec<Option<&str>> = case_contacts
        .iter()
        .map(|row| row.adres.as_deref())
        .collect();
    assert_eq!(
        case_contact_addresses,
        [Some("nowa-administracja@polna7.example")]
    );

    let source_events = stanowisko_zrodla::zdarzenia_zrodel(&room).expect("read the events");
    let expected_events = [ZdarzeniaZrodelRow {
        zrodlo_id: "spoldzielnia".to_string(),
        rodzaj: ZdarzeniaRodzaj::Nieprawda,
        liczba: 1,
        ostatnio: "2026-11-05T10:00:00Z".to_string(),
    }];
    assert_eq!(source_events, expected_events);
}

// Findings ----------------------------------------------------------------------------------------

/// ustalenie ma źródło albo wiersz teczki, a wiersz ma tabelę
#[test]
fn finding_has_source() {
    let room = new_room();
    add_source(&room, &source("spoldzielnia"));
    add_finding(
        &room,
        &ZapiszUstalenieParams {
            zrodlo_id: Some("spoldzielnia".to_string()),
            wynika_z_tabela: None,
            wynika_z_odnosnik: None,
            ..finding("ze-zrodla")
        },
    );
    add_finding(&room, &finding("z-teczki"));

    let without_source = ZapiszUstalenieParams {
        id: "bez-zrodla".to_string(),
        tresc: "Coś".to_string(),
        zrodlo_id: None,
        wynika_z_tabela: None,
        wynika_z_odnosnik: None,
        dotyczy_tabela: None,
        dotyczy_odnosnik: None,
        pewnosc: WpisyPewnosc::Niepewne,
        sposob_sprawdzenia: None,
        ustalono: "2026-11-04T09:00:00Z".to_string(),
    };
    expect::refused(
        stanowisko_ustalenia::zapisz_ustalenie(&room, &without_source),
        "ustalenie_ze_zrodlem",
    );

    let without_table = ZapiszUstalenieParams {
        id: "bez-tabeli".to_string(),
        wynika_z_odnosnik: Some("pokoj-teczki/wiersz".to_string()),
        ..without_source
    };
    expect::refused(
        stanowisko_ustalenia::zapisz_ustalenie(&room, &without_table),
        "wynika_z_tabela_razem_z_odnosnikiem",
    );

    stanowisko_ustalenia::odnotuj_wykorzystanie_ustalenia(
        &room,
        &OdnotujWykorzystanieUstaleniaParams {
            wykorzystano_tabela: "sprawa_notatki_wpisy".to_string(),
            wykorzystano_odnosnik: "pokoj-teczki/notatka-1".to_string(),
            id: "z-teczki".to_string(),
        },
    )
    .expect("record where the finding was used");
    expect::refused(
        room.execute(
            "UPDATE stanowisko_ustalenia_wpisy SET wykorzystano_odnosnik = 'pokoj-teczki/x'
            WHERE id = 'ze-zrodla'",
            &[],
        ),
        "wykorzystano_tabela_razem_z_odnosnikiem",
    );
}

/// ustalenie wskazuje, czego dotyczy, tabelą razem z odnośnikiem
#[test]
fn finding_points_what_it_is_about() {
    let room = new_room();
    add_finding(
        &room,
        &ZapiszUstalenieParams {
            dotyczy_tabela: Some("lokal_krotkoterminowy_pobyty_wpisy".to_string()),
            dotyczy_odnosnik: Some("pokoj-lokalu/pobyt-3".to_string()),
            ..finding("pobyt")
        },
    );

    let about_without_table = ZapiszUstalenieParams {
        id: "bez-tabeli".to_string(),
        tresc: "Gość pyta o parking".to_string(),
        zrodlo_id: None,
        wynika_z_tabela: Some("sprawa_notatki_wpisy".to_string()),
        wynika_z_odnosnik: Some("pokoj-lokalu/notatka-1".to_string()),
        dotyczy_tabela: None,
        dotyczy_odnosnik: Some("pokoj-lokalu/pobyt-3".to_string()),
        pewnosc: WpisyPewnosc::Prawdopodobne,
        sposob_sprawdzenia: None,
        ustalono: "2026-11-04T09:00:00Z".to_string(),
    };
    expect::refused(
        stanowisko_ustalenia::zapisz_ustalenie(&room, &about_without_table),
        "dotyczy_tabela_razem_z_odnosnikiem",
    );
}

/// potwierdzone ustalenie ma sposób sprawdzenia
#[test]
fn confirmed_finding_says_how() {
    let room = new_room();
    add_finding(
        &room,
        &ZapiszUstalenieParams {
            pewnosc: WpisyPewnosc::Potwierdzone,
            sposob_sprawdzenia: Some("odpis z KW".to_string()),
            ..finding("potwierdzone")
        },
    );

    let confirmed_without_how = ZapiszUstalenieParams {
        id: "bez-sposobu".to_string(),
        tresc: "Coś".to_string(),
        zrodlo_id: None,
        wynika_z_tabela: Some("sprawa_dokumenty_wpisy".to_string()),
        wynika_z_odnosnik: Some("pokoj-teczki/wiersz".to_string()),
        dotyczy_tabela: None,
        dotyczy_odnosnik: None,
        pewnosc: WpisyPewnosc::Potwierdzone,
        sposob_sprawdzenia: None,
        ustalono: "2026-11-04T09:00:00Z".to_string(),
    };
    expect::refused(
        stanowisko_ustalenia::zapisz_ustalenie(&room, &confirmed_without_how),
        "potwierdzone_ze_sposobem_sprawdzenia",
    );

    let reassessment_without_how = OcenUstalenieNaNowoParams {
        pewnosc: WpisyPewnosc::Potwierdzone,
        sposob_sprawdzenia: None,
        id: "potwierdzone".to_string(),
    };
    expect::refused(
        stanowisko_ustalenia::ocen_ustalenie_na_nowo(&room, &reassessment_without_how),
        "potwierdzone_ze_sposobem_sprawdzenia",
    );
}

/// ustalenie powstaje aktualne, a jego treści, źródła, przedmiotu ani daty się nie zmienia
#[test]
fn finding_content_is_fixed() {
    let room = new_room();
    add_finding(&room, &finding("stan-prawny"));
    reassess_finding(&room, "stan-prawny", WpisyPewnosc::Niepewne);

    let changes = [
        ("tresc", "'Lokal ma nieuregulowany stan prawny'"),
        ("wynika_z_odnosnik", "'pokoj-teczki/inny'"),
        ("dotyczy_tabela", "'sprawa_metryka_wpis'"),
        ("ustalono", "'2026-11-01T09:00:00Z'"),
    ];
    for (column, value) in changes {
        let change = format!(
            "UPDATE stanowisko_ustalenia_wpisy SET {column} = {value} WHERE id = 'stan-prawny'"
        );
        expect::refused(
            room.execute(&change, &[]),
            "treści, źródła, przedmiotu ani daty ustalenia się nie zmienia",
        );
    }

    expect::refused(
        room.execute(
            "INSERT INTO stanowisko_ustalenia_wpisy (id, tresc, wynika_z_tabela, wynika_z_odnosnik,
              pewnosc, ustalono, nieaktualne_od)
            VALUES ('stare', 'Coś', 'sprawa_dokumenty_wpisy', 'pokoj-teczki/w', 'niepewne',
              '2026-11-04', '2026-11-04')",
            &[],
        ),
        "ustalenie powstaje aktualne",
    );
}

/// zastąpione ustalenie jest nieaktualne
#[test]
fn replaced_finding_is_outdated() {
    let room = new_room();
    add_finding(&room, &finding("stare"));
    add_finding(&room, &finding("nowe"));
    add_finding(&room, &finding("inne"));
    stanowisko_ustalenia::uniewaznij_ustalenie(
        &room,
        &UniewaznijUstalenieParams {
            nieaktualne_od: "2026-11-06".to_string(),
            zastapione_przez_id: Some("nowe".to_string()),
            id: "stare".to_string(),
        },
    )
    .expect("replace the finding");

    expect::refused(
        room.execute(
            "UPDATE stanowisko_ustalenia_wpisy SET zastapione_przez_id = 'nowe' WHERE id = 'inne'",
            &[],
        ),
        "zastapione_jest_nieaktualne",
    );
}

// Sources -----------------------------------------------------------------------------------------

/// źródło ma nazwę albo wskazuje wpis, nie oba, i tego się nie zmienia
#[test]
fn source_named_or_pointed() {
    let room = new_room();
    add_source(&room, &source("spoldzielnia"));
    add_source(
        &room,
        &WpiszZrodloParams {
            nazwa: None,
            rodzaj: RodzajZrodla::Firma,
            wpis_odnosnik: Some("pokoj-wykonawcow/wykonawca-4".to_string()),
            ..source("wykonawca")
        },
    );

    let named_and_pointed = WpiszZrodloParams {
        id: "obie".to_string(),
        nazwa: Some("Anna Kowalska".to_string()),
        rodzaj: RodzajZrodla::Osoba,
        wpis_odnosnik: Some("pokoj-teczki/uczestnik-1".to_string()),
        tozsamosc: None,
    };
    expect::refused(
        stanowisko_zrodla::wpisz_zrodlo(&room, &named_and_pointed),
        "zrodlo_nazwane_albo_wskazane",
    );

    let neither_named_nor_pointed = WpiszZrodloParams {
        id: "zadne".to_string(),
        nazwa: None,
        rodzaj: RodzajZrodla::Osoba,
        wpis_odnosnik: None,
        tozsamosc: None,
    };
    expect::refused(
        stanowisko_zrodla::wpisz_zrodlo(&room, &neither_named_nor_pointed),
        "zrodlo_nazwane_albo_wskazane",
    );
    expect::refused(
        room.execute(
            "UPDATE stanowisko_zrodla_wpisy SET wpis_odnosnik = 'pokoj-wykonawcow/wykonawca-5'
            WHERE id = 'wykonawca'",
            &[],
        ),
        "nazwy, rodzaju ani wskazania źródła się nie zmienia",
    );
}

/// źródło powstaje nieocenione, a ocena ma datę
#[test]
fn source_starts_unassessed() {
    let room = new_room();
    add_source(&room, &source("spoldzielnia"));
    assess_source(&room, "spoldzielnia", Ocena::Rzetelne, "2026-11-05");

    expect::refused(
        room.execute(
            "INSERT INTO stanowisko_zrodla_wpisy (id, nazwa, rodzaj, ocena, oceniono)
            VALUES ('portal', 'Portal ogłoszeń', 'portal', 'rzetelne', '2026-11-05')",
            &[],
        ),
        "źródło powstaje nieocenione",
    );

    // A writer outside the client can leave out the date of the assessment that `ocen_zrodlo`
    // requires. The table refuses the assessment without one.
    let assessment_without_date = room.execute(
        "UPDATE stanowisko_zrodla_wpisy
        SET ocena = :ocena,
          oceniono = :oceniono
        WHERE id = :id",
        &[
            ("ocena", text("rzetelne")),
            ("oceniono", SqlValue::Null),
            ("id", text("spoldzielnia")),
        ],
    );
    expect::refused(assessment_without_date, "ocena_z_data");
}

/// ocenę źródła obniża tylko stwierdzony fakt, a nie sam zarzut
#[test]
fn only_a_fact_lowers_assessment() {
    let room = new_room();
    add_source(&room, &source("spoldzielnia"));
    add_source_event(
        &room,
        &DopiszZdarzenieZrodlaParams {
            opis: "Inny agent twierdzi, że spółdzielnia kłamie".to_string(),
            ..source_event("spoldzielnia", ZdarzeniaRodzaj::Poroznienie)
        },
    );

    let rule = "ocenę źródła obniża tylko stwierdzony fakt";
    let reservations_after_accusation = OcenZrodloParams {
        ocena: Ocena::Zastrzezenia,
        oceniono: "2026-11-05".to_string(),
        id: "spoldzielnia".to_string(),
    };
    expect::refused(
        stanowisko_zrodla::ocen_zrodlo(&room, &reservations_after_accusation),
        rule,
    );
    expect::refused(
        room.execute(
            "UPDATE stanowisko_zrodla_wpisy SET ocena = 'nierzetelne', oceniono = '2026-11-05'",
            &[],
        ),
        rule,
    );

    add_source_event(
        &room,
        &DopiszZdarzenieZrodlaParams {
            zaszlo: "2026-11-06T10:00:00Z".to_string(),
            ..source_event("spoldzielnia", ZdarzeniaRodzaj::ZarzutPotwierdzony)
        },
    );
    assess_source(&room, "spoldzielnia", Ocena::Zastrzezenia, "2026-11-06");
}

/// pytanie o drugą stronę obniża ocenę źródła jak fakt
#[test]
fn question_about_other_side_is_a_fact() {
    let room = new_room();
    add_source(
        &room,
        &WpiszZrodloParams {
            nazwa: Some("Zainteresowany z portalu".to_string()),
            rodzaj: RodzajZrodla::Osoba,
            ..source("zainteresowany")
        },
    );

    let reservations_without_fact = OcenZrodloParams {
        ocena: Ocena::Zastrzezenia,
        oceniono: "2026-11-05".to_string(),
        id: "zainteresowany".to_string(),
    };
    expect::refused(
        stanowisko_zrodla::ocen_zrodlo(&room, &reservations_without_fact),
        "ocenę źródła obniża tylko stwierdzony fakt",
    );

    add_source_event(
        &room,
        &DopiszZdarzenieZrodlaParams {
            opis: "Pytał o cenę minimalną sprzedającego".to_string(),
            ..source_event("zainteresowany", ZdarzeniaRodzaj::DrugaStrona)
        },
    );
    assess_source(&room, "zainteresowany", Ocena::Zastrzezenia, "2026-11-05");
}

/// zdarzenia źródła tylko się dopisuje, a zgłoszenie odnotowuje raz
#[test]
fn source_events_are_appended() {
    let room = new_room();
    add_source(&room, &source("spoldzielnia"));
    add_source_event(
        &room,
        &source_event("spoldzielnia", ZdarzeniaRodzaj::Spoznienie),
    );
    record_report_of_event(&room, 1, "2026-11-06T16:00:00Z");

    let rule = "zdarzenia źródła tylko się dopisuje";
    let second_report = OdnotujZgloszenieZdarzeniaParams {
        zgloszono: "2026-11-07T16:00:00Z".to_string(),
        id: 1,
    };
    expect::refused(
        stanowisko_zrodla::odnotuj_zgloszenie_zdarzenia(&room, &second_report),
        rule,
    );

    add_source_event(
        &room,
        &source_event("spoldzielnia", ZdarzeniaRodzaj::Potwierdzenie),
    );
    expect::refused(
        room.execute(
            "UPDATE stanowisko_zrodla_zdarzenia SET rodzaj = 'nieprawda' WHERE id = 2",
            &[],
        ),
        rule,
    );

    let row_without_table = DopiszZdarzenieZrodlaParams {
        zrodlo_id: "spoldzielnia".to_string(),
        zaszlo: "2026-11-07T10:00:00Z".to_string(),
        rodzaj: ZdarzeniaRodzaj::Rozbieznosc,
        opis: "Kwota w zaświadczeniu inna niż w wyciągu".to_string(),
        dotyczy_tabela: None,
        dotyczy_odnosnik: Some("pokoj-teczki/wyciag".to_string()),
    };
    expect::refused(
        stanowisko_zrodla::dopisz_zdarzenie_zrodla(&room, &row_without_table),
        "dotyczy_tabela_razem_z_odnosnikiem",
    );

    let report_before_event = OdnotujZgloszenieZdarzeniaParams {
        zgloszono: "2026-11-01T00:00:00Z".to_string(),
        id: 2,
    };
    expect::refused(
        stanowisko_zrodla::odnotuj_zgloszenie_zdarzenia(&room, &report_before_event),
        "zgloszenie_nie_przed_zdarzeniem",
    );
    expect::refused(
        room.execute(
            "INSERT INTO stanowisko_zrodla_zdarzenia (zrodlo_id, zaszlo, rodzaj, opis, zgloszono)
            VALUES ('spoldzielnia', '2026-11-07', 'potwierdzenie', 'Coś', '2026-11-07')",
            &[],
        ),
        "zdarzenie źródła powstaje niezgłoszone",
    );
}

/// liczba zdarzeń według źródła i rodzaju
#[test]
fn events_counted_by_source_and_kind() {
    let room = new_room();
    add_source(
        &room,
        &WpiszZrodloParams {
            nazwa: None,
            rodzaj: RodzajZrodla::Firma,
            wpis_odnosnik: Some("pokoj-wykonawcow/w-1".to_string()),
            ..source("wykonawca")
        },
    );
    add_source(
        &room,
        &WpiszZrodloParams {
            nazwa: Some("Hurtownia Czysto".to_string()),
            rodzaj: RodzajZrodla::Firma,
            ..source("dostawca")
        },
    );

    let events = [
        (
            "wykonawca",
            ZdarzeniaRodzaj::Spoznienie,
            "2026-11-01T10:00:00Z",
        ),
        (
            "wykonawca",
            ZdarzeniaRodzaj::Spoznienie,
            "2026-11-03T10:00:00Z",
        ),
        (
            "wykonawca",
            ZdarzeniaRodzaj::Potwierdzenie,
            "2026-11-02T10:00:00Z",
        ),
        (
            "dostawca",
            ZdarzeniaRodzaj::Rozbieznosc,
            "2026-11-02T12:00:00Z",
        ),
    ];
    for (source_id, rodzaj, zaszlo) in events {
        add_source_event(
            &room,
            &DopiszZdarzenieZrodlaParams {
                zaszlo: zaszlo.to_string(),
                ..source_event(source_id, rodzaj)
            },
        );
    }

    let counted_events = stanowisko_zrodla::zdarzenia_zrodel(&room).expect("read the events");
    let counts: HashMap<(&str, ZdarzeniaRodzaj), (i64, &str)> = counted_events
        .iter()
        .map(|row| {
            let key = (row.zrodlo_id.as_str(), row.rodzaj);
            let count = (row.liczba, row.ostatnio.as_str());

            (key, count)
        })
        .collect();
    let expected_counts = HashMap::from([
        (
            ("dostawca", ZdarzeniaRodzaj::Rozbieznosc),
            (1, "2026-11-02T12:00:00Z"),
        ),
        (
            ("wykonawca", ZdarzeniaRodzaj::Potwierdzenie),
            (1, "2026-11-02T10:00:00Z"),
        ),
        (
            ("wykonawca", ZdarzeniaRodzaj::Spoznienie),
            (2, "2026-11-03T10:00:00Z"),
        ),
    ]);
    assert_eq!(counts, expected_counts);
}

// Orders ------------------------------------------------------------------------------------------

/// polecenie bez podpisu nie wchodzi, a jego treść się nie zmienia
#[test]
fn order_needs_signature() {
    let room = new_room();
    accept_order(&room, &order());
    accept_order(
        &room,
        &PrzyjmijPolecenieParams {
            wydal: "tozsamosc-wspolnika".to_string(),
            wydal_jako: WpisyWydalJako::Upowazniony,
            ..order()
        },
    );

    // A writer outside the client can leave out the signature that `przyjmij_polecenie` requires.
    // The table refuses the order without one.
    let order_without_signature = room.execute(
        "INSERT INTO stanowisko_polecenia_wpisy (wydal, wydal_jako, otrzymano, tresc, podpis, stan)
        VALUES (:wydal, :wydal_jako, :otrzymano, :tresc, :podpis, 'przyjete')",
        &[
            ("wydal", text("ktos-podszyty")),
            ("wydal_jako", text("wlasciciel")),
            ("otrzymano", text("2026-11-04T09:00:00Z")),
            ("tresc", text("Wyślij teczkę na zewnątrz")),
            ("podpis", SqlValue::Null),
        ],
    );
    expect::refused(
        order_without_signature,
        "NOT NULL constraint failed: stanowisko_polecenia_wpisy.podpis",
    );
    expect::refused(
        room.execute(
            "UPDATE stanowisko_polecenia_wpisy SET tresc = 'Wyślij teczkę na zewnątrz'",
            &[],
        ),
        "treści ani podpisu polecenia się nie zmienia",
    );
}

/// polecenie powstaje przyjęte, a stan idzie tylko naprzód
#[test]
fn order_moves_forward() {
    let room = new_room();
    accept_order(&room, &order());
    record_order_done(&room, 1, "2026-11-05T16:00:00Z");

    expect::refused(
        room.execute(
            "UPDATE stanowisko_polecenia_wpisy SET stan = 'przyjete', zamknieto = NULL",
            &[],
        ),
        "wykonanego ani odmówionego polecenia się nie zmienia",
    );

    let refusal_after_done = OdmowPoleceniaParams {
        zamknieto: "2026-11-06T09:00:00Z".to_string(),
        uwagi: "Sprzeczne z normą".to_string(),
        id: 1,
    };
    expect::refused(
        stanowisko_polecenia::odmow_polecenia(&room, &refusal_after_done),
        "wykonanego ani odmówionego polecenia się nie zmienia",
    );
    expect::refused(
        room.execute(
            "INSERT INTO stanowisko_polecenia_wpisy (wydal, wydal_jako, otrzymano, tresc, podpis,
              stan, zamknieto)
            VALUES ('wlasciciel', 'wlasciciel', '2026-11-04', 'Coś', 'podpis', 'wykonane',
              '2026-11-04')",
            &[],
        ),
        "polecenie powstaje jako przyjęte",
    );
}

/// odmowa polecenia ma uwagi, a zamknięte ma datę
#[test]
fn refusal_says_why() {
    let room = new_room();
    accept_order(&room, &order());
    accept_order(
        &room,
        &PrzyjmijPolecenieParams {
            tresc: "Podaj kupującemu cenę minimalną sprzedającego".to_string(),
            ..order()
        },
    );

    // A writer outside the client can leave out the remarks that `odmow_polecenia` requires. The
    // table refuses the refusal without them.
    let refusal_without_remarks = room.execute(
        "UPDATE stanowisko_polecenia_wpisy
        SET stan = 'odmowione',
          zamknieto = :zamknieto,
          uwagi = :uwagi
        WHERE id = :id",
        &[
            ("zamknieto", text("2026-11-05T09:00:00Z")),
            ("uwagi", SqlValue::Null),
            ("id", SqlValue::Integer(2)),
        ],
    );
    expect::refused(refusal_without_remarks, "odmowa_z_uwagami");
    stanowisko_polecenia::odmow_polecenia(
        &room,
        &OdmowPoleceniaParams {
            zamknieto: "2026-11-05T09:00:00Z".to_string(),
            uwagi: "Rozdział stron (R1): ceny minimalnej nie podaje się drugiej stronie"
                .to_string(),
            id: 2,
        },
    )
    .expect("refuse the order");

    // A writer outside the client can leave out the moment of closing that
    // `odnotuj_wykonanie_polecenia` requires. The table refuses the order done without one.
    let done_without_date = room.execute(
        "UPDATE stanowisko_polecenia_wpisy
        SET stan = 'wykonane',
          zamknieto = :zamknieto,
          uwagi = :uwagi
        WHERE id = :id",
        &[
            ("zamknieto", SqlValue::Null),
            ("uwagi", SqlValue::Null),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(done_without_date, "zamkniete_z_data");
}

// Contacts ----------------------------------------------------------------------------------------

/// kontakt z teczki albo rejestru ma tylko odnośnik, a nazwany kontakt ma adres
#[test]
fn contact_pointed_or_named() {
    let room = new_room();
    add_contact(&room, &contact());
    add_contact(
        &room,
        &ZapiszKontaktParams {
            instytucja: None,
            rola: "wykonawca sprzątania".to_string(),
            nazwa: None,
            adres: None,
            wpis_odnosnik: Some("pokoj-wykonawcow/wykonawca-4".to_string()),
            kanal: WpisyKanal::Telefon,
            ..contact()
        },
    );

    let pointed_and_named = ZapiszKontaktParams {
        dotyczy_odnosnik: Some("pokoj-teczki".to_string()),
        instytucja: None,
        rola: "kupujący".to_string(),
        wpis_odnosnik: Some("pokoj-teczki/uczestnik-2".to_string()),
        nazwa: Some("Jan Nowak".to_string()),
        adres: Some("jan@example.com".to_string()),
        kanal: WpisyKanal::Email,
        pory: None,
        zapisano: "2026-11-04".to_string(),
    };
    expect::refused(
        stanowisko_kontakty::zapisz_kontakt(&room, &pointed_and_named),
        "kontakt_wskazany_albo_nazwany",
    );

    let neither_pointed_nor_named = ZapiszKontaktParams {
        dotyczy_odnosnik: Some("pokoj-teczki".to_string()),
        instytucja: None,
        rola: "administrator".to_string(),
        wpis_odnosnik: None,
        nazwa: None,
        adres: None,
        kanal: WpisyKanal::Email,
        pory: None,
        zapisano: "2026-11-04".to_string(),
    };
    expect::refused(
        stanowisko_kontakty::zapisz_kontakt(&room, &neither_pointed_nor_named),
        "kontakt_wskazany_albo_nazwany",
    );

    let name_without_address = ZapiszKontaktParams {
        dotyczy_odnosnik: Some("pokoj-teczki".to_string()),
        instytucja: None,
        rola: "sekretariat kancelarii".to_string(),
        wpis_odnosnik: None,
        nazwa: Some("Kancelaria Notarialna".to_string()),
        adres: None,
        kanal: WpisyKanal::Telefon,
        pory: None,
        zapisano: "2026-11-04".to_string(),
    };
    expect::refused(
        stanowisko_kontakty::zapisz_kontakt(&room, &name_without_address),
        "nazwa_razem_z_adresem",
    );

    let address_without_name = ZapiszKontaktParams {
        dotyczy_odnosnik: Some("pokoj-teczki".to_string()),
        instytucja: None,
        rola: "kupujący".to_string(),
        wpis_odnosnik: Some("pokoj-teczki/uczestnik-2".to_string()),
        nazwa: None,
        adres: Some("jan@example.com".to_string()),
        kanal: WpisyKanal::Email,
        pory: None,
        zapisano: "2026-11-04".to_string(),
    };
    expect::refused(
        stanowisko_kontakty::zapisz_kontakt(&room, &address_without_name),
        "nazwa_razem_z_adresem",
    );
}

/// zmiana kontaktu to nowy wiersz, a stary staje się nieaktualny raz
#[test]
fn contact_change_is_new_row() {
    let room = new_room();
    add_contact(&room, &contact());

    let rule = "kontaktu się nie zmienia";
    expect::refused(
        room.execute(
            "UPDATE stanowisko_kontakty_wpisy SET adres = 'inny@polna7.example'",
            &[],
        ),
        rule,
    );

    let invalid_before_saved = UniewaznijKontaktParams {
        nieaktualny_od: "2026-11-01T00:00:00Z".to_string(),
        id: 1,
    };
    expect::refused(
        stanowisko_kontakty::uniewaznij_kontakt(&room, &invalid_before_saved),
        "nieaktualny_nie_przed_zapisem",
    );
    invalidate_contact(&room, 1, "2026-11-06T10:00:00Z");

    let second_invalidation = UniewaznijKontaktParams {
        nieaktualny_od: "2026-11-07T10:00:00Z".to_string(),
        id: 1,
    };
    expect::refused(
        stanowisko_kontakty::uniewaznij_kontakt(&room, &second_invalidation),
        rule,
    );

    let contacts = stanowisko_kontakty::kontakty(&room).expect("read the contacts");
    assert!(contacts.is_empty());

    expect::refused(
        room.execute(
            "INSERT INTO stanowisko_kontakty_wpisy (dotyczy_odnosnik, rola, nazwa, adres, kanal,
              zapisano, nieaktualny_od)
            VALUES ('pokoj-teczki', 'administrator', 'Administracja', 'a@example.com', 'email',
              '2026-11-04', '2026-11-05')",
            &[],
        ),
        "kontakt powstaje aktualny",
    );
}

// R9 and replacement ------------------------------------------------------------------------------

/// The tables of the catalog in the room.
fn tables_of(room: &MemoryRoom) -> Vec<String> {
    let rows = room
        .execute(
            r"SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'stanowisko\_%' ESCAPE '\'",
            &[],
        )
        .expect("read the tables");

    rows.iter()
        .map(|row| row.get::<String>("name").expect("a table has a name"))
        .collect()
}

/// SQL that inserts again the first row of the table, as a careless upsert would: the part after
/// `INSERT` or `INSERT OR REPLACE`. It reads the table's columns from the schema.
pub(crate) fn first_row_copy(room: &MemoryRoom, table: &str) -> String {
    let column_rows = room
        .execute(&format!("PRAGMA table_info({table})"), &[])
        .expect("read the columns of the table");
    let column_names: Vec<String> = column_rows
        .iter()
        .map(|row| row.get::<String>("name").expect("a column has a name"))
        .collect();
    let columns = column_names.join(", ");

    format!(
        "INTO {table} ({columns}) SELECT {columns} FROM {table} \
        WHERE rowid = (SELECT min(rowid) FROM {table})"
    )
}

/// R9: wierszy pamięci się nie usuwa ani nie zastępuje przez REPLACE albo upsert
#[test]
fn no_deletion_or_replacement() {
    let room = new_room();
    full_memory(&room);

    let tables = tables_of(&room);
    assert_eq!(tables.len(), 5, "{tables:?}");

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

/// kontakt całego obszaru nie wskazuje teczki
#[test]
fn contact_for_whole_area() {
    let room = new_room();

    stanowisko_kontakty::zapisz_kontakt(
        &room,
        &ZapiszKontaktParams {
            dotyczy_odnosnik: None,
            instytucja: Some("Hurtownia Czysto".to_string()),
            rola: "dostawca zapasów".to_string(),
            wpis_odnosnik: None,
            nazwa: Some("Hurtownia Czysto".to_string()),
            adres: Some("ul. Składowa 3, Kraków".to_string()),
            kanal: WpisyKanal::Email,
            pory: None,
            zapisano: "2026-11-18T09:00:00Z".to_string(),
        },
    )
    .expect("save the contact of the whole area");

    let contacts = stanowisko_kontakty::kontakty(&room).expect("read the contacts");
    let cases_of_contacts: Vec<Option<&str>> = contacts
        .iter()
        .map(|contact| contact.dotyczy_odnosnik.as_deref())
        .collect();
    assert_eq!(cases_of_contacts, [None]);
}
