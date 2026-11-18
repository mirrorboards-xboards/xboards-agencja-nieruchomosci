//! Scenarios of the register of contractors (spec/registers/wykonawcy).
//!
//! Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
//! allows and a write it refuses. The first scenario follows a cleaning company from its admission
//! with a price list to its suspension and removal, as the profile describes it.

use xboards_agencja_nieruchomosci::registers::wykonawcy::{
    self, wykonawcy_cenniki, wykonawcy_spis,
};
use xboards_agencja_nieruchomosci::xmeta_room::{FromSqlValue, MemoryRoom, SqlValue};

use wykonawcy_cenniki::{
    DopiszPozycjeCennikaParams, ObowiazujacaWersjaCennikaParams, PoprawPozycjeCennikaParams,
    PozycjeCennikaParams, PrzypiszCennikParams, WpiszCennikParams, WpiszWersjeCennikaParams,
    ZakonczPrzypisanieCennikaParams, ZamknijWersjeCennikaParams,
};
use wykonawcy_spis::{
    DaneWykonawcyParams, DecyzjeRodzaj, KanalyRodzaj, OsobyWykonawcyParams, StanyStan,
    StanyWykonawcyParams, WpisyRodzaj as RodzajWykonawcy, WpiszDaneWykonawcyParams,
    WpiszDecyzjeOWykonawcyParams, WpiszKanalWykonawcyParams, WpiszObszarWykonawcyParams,
    WpiszOsobeWykonawcyParams, WpiszWykonawceParams, ZakonczKanalWykonawcyParams,
    ZakonczObszarWykonawcyParams, ZakonczOsobeWykonawcyParams, ZmienStanWykonawcyParams,
};

use crate::expect;

const REPLACEMENT: &str =
    "wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE";
const DELETION: &str = "wierszy rejestru się nie usuwa (R9)";
const UNCHANGED: &str = "decyzji, wpisu wykonawcy ani jego stanu się nie zmienia";
const AFTER_DATA_DECISION: &str =
    "dane, osobę, kanał i obszar wpisuje się po decyzji o przyjęciu albo zmianie danych";
const AFTER_END_DECISION: &str =
    "koniec osoby, kanału i obszaru wpisuje się po decyzji o zmianie danych albo wykreśleniu";
const NOT_REMOVED: &str =
    "wykreślony wykonawca nie dostaje nowych danych, osób, kanałów ani obszarów";

fn new_room() -> MemoryRoom {
    wykonawcy::open_in_memory().expect("open the contractors room")
}

/// A text value to bind in raw SQL.
fn text(value: &str) -> SqlValue {
    SqlValue::Text(value.to_string())
}

// Building blocks ---------------------------------------------------------------------------------

/// A decision of the human, signed with their key and copied into the register.
fn decide(room: &MemoryRoom, decision_id: &str, rodzaj: DecyzjeRodzaj) {
    let params = WpiszDecyzjeOWykonawcyParams {
        id: decision_id.to_string(),
        rodzaj,
        decyduje: "wlasciciel-kierownika-najmu".to_string(),
        tresc: format!("decyzja: {}", rodzaj.as_str()),
        dotyczy_skrot: "sha256-wpisu".to_string(),
        podpis: "podpis-kluczem-xkeys".to_string(),
        podpisano: "2026-04-01T10:00:00Z".to_string(),
        wpisal: "kierownik-najmu".to_string(),
        wpisano: "2026-04-01T12:00:00Z".to_string(),
    };

    wykonawcy_spis::wpisz_decyzje_o_wykonawcy(room, &params).expect("record the decision");
}

/// The default parameters of the cleaning company „czysto”, admitted by a decision.
fn contractor(contractor_id: &str, decision_id: &str) -> WpiszWykonawceParams {
    WpiszWykonawceParams {
        id: contractor_id.to_string(),
        nazwa: "Czysto sp. c.".to_string(),
        rodzaj: RodzajWykonawcy::Sprzatanie,
        specjalnosc: None,
        decyzja_id: decision_id.to_string(),
        wpisano: "2026-04-01T12:00:00Z".to_string(),
        firma: "Czysto sp. c.".to_string(),
        nip: Some("5250000000".to_string()),
        krs: None,
        ceidg: true,
        adres: "ul. Szewska 1, Kraków".to_string(),
        zakres_uslug: "sprzątanie i pranie; bez mycia okien na wysokości".to_string(),
        srodki_wlasne: true,
        umowa_od: "2026-04-01".to_string(),
        umowa_do: Some("2027-03-31".to_string()),
        umowa_podpisal: "wlasciciel-kierownika-najmu".to_string(),
        umowa_sciezka: "umowy/czysto.pdf".to_string(),
        umowa_skrot: "sha256-umowy".to_string(),
        polisa_do: None,
        polisa_sciezka: None,
        polisa_skrot: None,
    }
}

/// The default parameters of a new version of the data of the contractor „czysto”.
fn contractor_data(decision_id: &str) -> WpiszDaneWykonawcyParams {
    WpiszDaneWykonawcyParams {
        wykonawca_id: "czysto".to_string(),
        firma: "Czysto sp. c.".to_string(),
        nip: Some("5250000000".to_string()),
        krs: None,
        ceidg: true,
        adres: "ul. Szewska 1, Kraków".to_string(),
        zakres_uslug: "sprzątanie i pranie; bez mycia okien na wysokości".to_string(),
        srodki_wlasne: true,
        umowa_od: "2026-04-01".to_string(),
        umowa_do: Some("2027-03-31".to_string()),
        umowa_podpisal: "wlasciciel-kierownika-najmu".to_string(),
        umowa_sciezka: "umowy/czysto.pdf".to_string(),
        umowa_skrot: "sha256-umowy".to_string(),
        polisa_do: None,
        polisa_sciezka: None,
        polisa_skrot: None,
        decyzja_id: decision_id.to_string(),
        wpisano: "2026-04-01T12:00:00Z".to_string(),
    }
}

fn admit(room: &MemoryRoom) {
    decide(room, "przyjecie-czysto", DecyzjeRodzaj::Przyjecie);

    let params = contractor("czysto", "przyjecie-czysto");
    wykonawcy_spis::wpisz_wykonawce(room, &params).expect("admit the contractor");
}

/// The default parameters of a worker of the contractor „czysto”.
fn worker(decision_id: &str) -> WpiszOsobeWykonawcyParams {
    WpiszOsobeWykonawcyParams {
        wykonawca_id: "czysto".to_string(),
        imiona: "Maria".to_string(),
        nazwisko: "Wójcik".to_string(),
        telefon: "+48600300400".to_string(),
        kody_do_zamkow: true,
        od: "2026-04-01".to_string(),
        decyzja_id: decision_id.to_string(),
    }
}

/// The default parameters of a channel of the contractor „czysto”.
fn channel(decision_id: &str) -> WpiszKanalWykonawcyParams {
    WpiszKanalWykonawcyParams {
        wykonawca_id: "czysto".to_string(),
        rodzaj: KanalyRodzaj::Sms,
        adres: "+48600300400".to_string(),
        od: "2026-04-01".to_string(),
        decyzja_id: decision_id.to_string(),
    }
}

/// The default parameters of an area of the contractor „czysto”.
fn area(decision_id: &str) -> WpiszObszarWykonawcyParams {
    WpiszObszarWykonawcyParams {
        wykonawca_id: "czysto".to_string(),
        obszar: "Stare Miasto".to_string(),
        dostepnosc: "pon.–sob. 8–20".to_string(),
        czas_dojazdu_do_usterki: None,
        od: "2026-04-01".to_string(),
        decyzja_id: decision_id.to_string(),
    }
}

fn add_worker(room: &MemoryRoom, decision_id: &str) {
    wykonawcy_spis::wpisz_osobe_wykonawcy(room, &worker(decision_id)).expect("add the worker");
}

fn add_channel(room: &MemoryRoom, decision_id: &str) {
    wykonawcy_spis::wpisz_kanal_wykonawcy(room, &channel(decision_id)).expect("add the channel");
}

fn add_area(room: &MemoryRoom, decision_id: &str) {
    wykonawcy_spis::wpisz_obszar_wykonawcy(room, &area(decision_id)).expect("add the area");
}

fn change_state(
    room: &MemoryRoom,
    decision_id: &str,
    stan: StanyStan,
    od: &str,
    powod: Option<&str>,
) {
    let params = ZmienStanWykonawcyParams {
        wykonawca_id: "czysto".to_string(),
        stan,
        od: od.to_string(),
        powod: powod.map(str::to_string),
        decyzja_id: decision_id.to_string(),
    };

    wykonawcy_spis::zmien_stan_wykonawcy(room, &params).expect("change the contractor's state");
}

/// The default parameters of a change of the state of the contractor „czysto”.
fn state_change(decision_id: &str, stan: StanyStan) -> ZmienStanWykonawcyParams {
    ZmienStanWykonawcyParams {
        wykonawca_id: "czysto".to_string(),
        stan,
        od: "2026-06-01".to_string(),
        powod: Some("nie trzyma standardu".to_string()),
        decyzja_id: decision_id.to_string(),
    }
}

fn add_price_list(room: &MemoryRoom) {
    let params = WpiszCennikParams {
        id: "cennik".to_string(),
        nazwa: "Sprzątanie Stare Miasto".to_string(),
        wpisano: "2026-04-01".to_string(),
    };

    wykonawcy_cenniki::wpisz_cennik(room, &params).expect("add the price list");
}

fn price_list_version(room: &MemoryRoom) {
    let params = WpiszWersjeCennikaParams {
        obowiazuje_od: "2026-04-01".to_string(),
        ..version()
    };

    wykonawcy_cenniki::wpisz_wersje_cennika(room, &params)
        .expect("add a version of the price list");
}

/// The default parameters of the first version of the price list „cennik”.
fn version() -> WpiszWersjeCennikaParams {
    WpiszWersjeCennikaParams {
        id: "wersja-1".to_string(),
        cennik_id: "cennik".to_string(),
        numer: 1,
        obowiazuje_od: "2026-05-01".to_string(),
        wpisano: "2026-04-01T12:00:00Z".to_string(),
        decyzja_id: "cennik-decyzja".to_string(),
    }
}

fn add_item(room: &MemoryRoom, cena: i64) {
    let params = DopiszPozycjeCennikaParams {
        wersja_id: "wersja-1".to_string(),
        usluga: "sprzątanie po wyjeździe gościa".to_string(),
        doplata: false,
        jednostka: "lokal do 40 m²".to_string(),
        cena,
        z_vat: true,
        warunki: None,
    };

    wykonawcy_cenniki::dopisz_pozycje_cennika(room, &params)
        .expect("add an item of the price list");
}

fn close_version(room: &MemoryRoom) {
    let params = ZamknijWersjeCennikaParams {
        id: "wersja-1".to_string(),
    };

    wykonawcy_cenniki::zamknij_wersje_cennika(room, &params)
        .expect("close the version of the price list");
}

fn closed_price_list(room: &MemoryRoom) {
    decide(room, "cennik-decyzja", DecyzjeRodzaj::Cennik);
    add_price_list(room);
    price_list_version(room);
    add_item(room, 15000);
    close_version(room);
}

/// A contractor and a price list with a row in every table of the catalog.
fn full_register(room: &MemoryRoom) {
    admit(room);
    add_worker(room, "przyjecie-czysto");
    add_channel(room, "przyjecie-czysto");
    add_area(room, "przyjecie-czysto");
    closed_price_list(room);

    decide(room, "przypisanie", DecyzjeRodzaj::Przypisanie);
    let assignment = PrzypiszCennikParams {
        cennik_id: "cennik".to_string(),
        wykonawca_id: "czysto".to_string(),
        od: "2026-04-02".to_string(),
        decyzja_id: "przypisanie".to_string(),
    };
    wykonawcy_cenniki::przypisz_cennik(room, &assignment).expect("assign the price list");
}

/// The id of the newest worker of the contractor „czysto”.
fn newest_worker_id(room: &MemoryRoom) -> i64 {
    let workers = wykonawcy_spis::osoby_wykonawcy(
        room,
        &OsobyWykonawcyParams {
            wykonawca_id: "czysto".to_string(),
        },
    )
    .expect("read the workers");

    workers[0].id
}

// A whole contractor ------------------------------------------------------------------------------

/// Wykonawca: przyjęcie z cennikiem, wstrzymanie, przywrócenie i wykreślenie
#[test]
fn whole_contractor() {
    let room = new_room();
    full_register(&room);

    let contractors = wykonawcy_spis::wykonawcy(&room).expect("read the contractors");
    let ids_and_states: Vec<(&str, StanyStan)> = contractors
        .iter()
        .map(|row| (row.id.as_str(), row.stan))
        .collect();
    assert_eq!(ids_and_states, [("czysto", StanyStan::Zatwierdzony)]);

    let in_force = wykonawcy_cenniki::obowiazujaca_wersja_cennika(
        &room,
        &ObowiazujacaWersjaCennikaParams {
            cennik_id: "cennik".to_string(),
            dzien: "2026-05-01".to_string(),
        },
    )
    .expect("read the version in force")
    .expect("a version is in force");
    assert_eq!(in_force.id, "wersja-1");

    decide(&room, "wstrzymanie", DecyzjeRodzaj::Wstrzymanie);
    change_state(
        &room,
        "wstrzymanie",
        StanyStan::Wstrzymany,
        "2026-06-01",
        Some("nie przychodzi na zlecenia"),
    );
    decide(&room, "przywrocenie", DecyzjeRodzaj::Przywrocenie);
    change_state(
        &room,
        "przywrocenie",
        StanyStan::Zatwierdzony,
        "2026-06-15",
        None,
    );
    decide(&room, "wykreslenie", DecyzjeRodzaj::Wykreslenie);
    change_state(
        &room,
        "wykreslenie",
        StanyStan::Wykreslony,
        "2026-09-01",
        Some("koniec umowy"),
    );

    let worker_id = newest_worker_id(&room);
    wykonawcy_spis::zakoncz_osobe_wykonawcy(
        &room,
        &ZakonczOsobeWykonawcyParams {
            r#do: Some("2026-09-01".to_string()),
            zakonczenie_decyzja_id: "wykreslenie".to_string(),
            id: worker_id,
        },
    )
    .expect("end the worker");

    let workers = wykonawcy_spis::osoby_wykonawcow(&room).expect("read the workers");
    assert!(workers.is_empty());

    let states = wykonawcy_spis::stany_wykonawcy(
        &room,
        &StanyWykonawcyParams {
            wykonawca_id: "czysto".to_string(),
        },
    )
    .expect("read the states");
    let state_values: Vec<StanyStan> = states.iter().map(|row| row.stan).collect();
    assert_eq!(
        state_values,
        [
            StanyStan::Zatwierdzony,
            StanyStan::Wstrzymany,
            StanyStan::Zatwierdzony,
            StanyStan::Wykreslony,
        ]
    );
}

// Decisions ---------------------------------------------------------------------------------------

/// Decyzje: podpis w komplecie, wpis nie przed podpisem, decyzji się nie zmienia
#[test]
fn decisions() {
    let room = new_room();
    let decision = WpiszDecyzjeOWykonawcyParams {
        id: "decyzja".to_string(),
        rodzaj: DecyzjeRodzaj::Przyjecie,
        decyduje: "wlasciciel".to_string(),
        tresc: "przyjęcie".to_string(),
        dotyczy_skrot: "sha256".to_string(),
        podpis: "podpis".to_string(),
        podpisano: "2026-04-01T10:00:00Z".to_string(),
        wpisal: "kierownik-najmu".to_string(),
        wpisano: "2026-04-01T12:00:00Z".to_string(),
    };

    // A writer outside the client can leave out the signature that `wpisz_decyzje_o_wykonawcy`
    // requires. The table refuses the decision without one.
    let decision_without_signature = room.execute(
        "INSERT INTO wykonawcy_spis_decyzje (id, rodzaj, decyduje, tresc, dotyczy_skrot, podpis,
          podpisano, wpisal, wpisano)
        VALUES (:id, :rodzaj, :decyduje, :tresc, :dotyczy_skrot, :podpis, :podpisano, :wpisal,
          :wpisano)",
        &[
            ("id", text("decyzja")),
            ("rodzaj", text("przyjecie")),
            ("decyduje", text("wlasciciel")),
            ("tresc", text("przyjęcie")),
            ("dotyczy_skrot", text("sha256")),
            ("podpis", SqlValue::Null),
            ("podpisano", text("2026-04-01T10:00:00Z")),
            ("wpisal", text("kierownik-najmu")),
            ("wpisano", text("2026-04-01T12:00:00Z")),
        ],
    );
    expect::refused(
        decision_without_signature,
        "NOT NULL constraint failed: wykonawcy_spis_decyzje.podpis",
    );
    expect::refused(
        wykonawcy_spis::wpisz_decyzje_o_wykonawcy(
            &room,
            &WpiszDecyzjeOWykonawcyParams {
                wpisano: "2026-04-01T09:00:00Z".to_string(),
                ..decision.clone()
            },
        ),
        "wpis_nie_przed_podpisem",
    );

    wykonawcy_spis::wpisz_decyzje_o_wykonawcy(&room, &decision).expect("record the decision");
    expect::refused(
        room.execute("UPDATE wykonawcy_spis_decyzje SET tresc = 'inna'", &[]),
        UNCHANGED,
    );
}

/// Wpis wykonawcy: po decyzji o przyjęciu; specjalność ma wykonawca napraw
#[test]
fn contractor_entry() {
    let room = new_room();
    decide(&room, "zmiana", DecyzjeRodzaj::ZmianaDanych);

    expect::refused(
        wykonawcy_spis::wpisz_wykonawce(&room, &contractor("czysto", "zmiana")),
        "wykonawcę wpisuje się po decyzji o przyjęciu",
    );

    decide(&room, "przyjecie-hydraulika", DecyzjeRodzaj::Przyjecie);
    let plumber = WpiszWykonawceParams {
        nazwa: "Jan Kowalski".to_string(),
        rodzaj: RodzajWykonawcy::NaprawyISerwis,
        ..contractor("kowalski", "przyjecie-hydraulika")
    };
    expect::refused(
        wykonawcy_spis::wpisz_wykonawce(&room, &plumber),
        "specjalnosc_przy_naprawach",
    );
    wykonawcy_spis::wpisz_wykonawce(
        &room,
        &WpiszWykonawceParams {
            specjalnosc: Some("hydraulik".to_string()),
            ..plumber
        },
    )
    .expect("admit the plumber");
    expect::refused(
        room.execute(
            "UPDATE wykonawcy_spis_wpisy SET nazwa = 'Kowalski i syn'",
            &[],
        ),
        UNCHANGED,
    );
}

/// Dane wykonawcy: KRS albo CEIDG, umowa i polisa w komplecie, nowa wersja po decyzji
#[test]
fn contractor_data_versions() {
    let room = new_room();
    decide(&room, "przyjecie-czysto", DecyzjeRodzaj::Przyjecie);
    let entry = contractor("czysto", "przyjecie-czysto");

    expect::refused(
        wykonawcy_spis::wpisz_wykonawce(
            &room,
            &WpiszWykonawceParams {
                krs: Some("0000123456".to_string()),
                ..entry.clone()
            },
        ),
        "krs_albo_ceidg",
    );
    expect::refused(
        wykonawcy_spis::wpisz_wykonawce(
            &room,
            &WpiszWykonawceParams {
                umowa_do: Some("2026-01-01".to_string()),
                ..entry.clone()
            },
        ),
        "koniec_umowy_nie_przed_poczatkiem",
    );
    expect::refused(
        wykonawcy_spis::wpisz_wykonawce(
            &room,
            &WpiszWykonawceParams {
                polisa_do: Some("2027-01-01".to_string()),
                ..entry.clone()
            },
        ),
        "polisa_w_komplecie",
    );
    wykonawcy_spis::wpisz_wykonawce(&room, &entry).expect("admit the contractor");

    decide(&room, "wstrzymanie", DecyzjeRodzaj::Wstrzymanie);
    expect::refused(
        wykonawcy_spis::wpisz_dane_wykonawcy(&room, &contractor_data("wstrzymanie")),
        AFTER_DATA_DECISION,
    );
    expect::refused(
        room.execute(
            "INSERT INTO wykonawcy_spis_dane (wykonawca_id, aktualne, firma, adres, zakres_uslug,
              srodki_wlasne, umowa_od, umowa_podpisal, umowa_sciezka, umowa_skrot, decyzja_id, wpisano)
            VALUES ('czysto', 0, 'Czysto', 'ul. Szewska 1', 'sprzątanie', 0, '2026-04-01', 'wlasciciel',
              'umowa.pdf', 'sha256', 'przyjecie-czysto', '2026-04-01')",
            &[],
        ),
        "wersja danych wykonawcy powstaje jako aktualna",
    );

    decide(&room, "przedluzenie", DecyzjeRodzaj::ZmianaDanych);
    wykonawcy_spis::wpisz_dane_wykonawcy(
        &room,
        &WpiszDaneWykonawcyParams {
            umowa_do: Some("2028-03-31".to_string()),
            polisa_do: Some("2027-12-31".to_string()),
            polisa_sciezka: Some("polisa.pdf".to_string()),
            polisa_skrot: Some("sha256-polisy".to_string()),
            ..contractor_data("przedluzenie")
        },
    )
    .expect("extend the contract");

    let versions = wykonawcy_spis::dane_wykonawcy(
        &room,
        &DaneWykonawcyParams {
            wykonawca_id: "czysto".to_string(),
        },
    )
    .expect("read the versions of the data");
    let contract_ends_and_currency: Vec<(Option<&str>, bool)> = versions
        .iter()
        .map(|row| (row.umowa_do.as_deref(), row.aktualne))
        .collect();
    assert_eq!(
        contract_ends_and_currency,
        [(Some("2028-03-31"), true), (Some("2027-03-31"), false)]
    );

    expect::refused(
        room.execute(
            "UPDATE wykonawcy_spis_dane SET adres = 'ul. Inna 2' WHERE aktualne = 1",
            &[],
        ),
        "danych wykonawcy się nie poprawia",
    );
    expect::refused(
        room.execute(
            "UPDATE wykonawcy_spis_dane SET aktualne = 0 WHERE aktualne = 1",
            &[],
        ),
        "danych wykonawcy się nie poprawia",
    );
}

// Workers, channels and areas ---------------------------------------------------------------------

/// Osoby, kanały i obszary: wpis po decyzji o przyjęciu albo zmianie danych
#[test]
fn rows_after_decision() {
    let room = new_room();
    admit(&room);
    decide(&room, "cennik-decyzja", DecyzjeRodzaj::Cennik);

    expect::refused(
        wykonawcy_spis::wpisz_osobe_wykonawcy(&room, &worker("cennik-decyzja")),
        AFTER_DATA_DECISION,
    );
    expect::refused(
        wykonawcy_spis::wpisz_kanal_wykonawcy(&room, &channel("cennik-decyzja")),
        AFTER_DATA_DECISION,
    );
    expect::refused(
        wykonawcy_spis::wpisz_obszar_wykonawcy(&room, &area("cennik-decyzja")),
        AFTER_DATA_DECISION,
    );

    decide(&room, "nowa-brygadzistka", DecyzjeRodzaj::ZmianaDanych);
    add_worker(&room, "nowa-brygadzistka");
    add_channel(&room, "nowa-brygadzistka");
    add_area(&room, "nowa-brygadzistka");
}

/// Osoby, kanały i obszary: powstają bieżące, kończą się raz i tylko decyzją
#[test]
fn rows_end_with_decision() {
    let room = new_room();
    admit(&room);

    let tables_with_values = [
        (
            "wykonawcy_spis_osoby",
            "imiona, nazwisko, telefon, kody_do_zamkow",
            "'Maria', 'Wójcik', '+48600', 1",
        ),
        ("wykonawcy_spis_kanaly", "rodzaj, adres", "'sms', '+48600'"),
        (
            "wykonawcy_spis_obszary",
            "obszar, dostepnosc",
            "'Kazimierz', 'pon.–pt.'",
        ),
    ];
    for (table, columns, values) in tables_with_values {
        let ended_row = format!(
            "INSERT INTO {table} (wykonawca_id, {columns}, od, do, decyzja_id,
              zakonczenie_decyzja_id)
            VALUES ('czysto', {values}, '2026-04-01', '2026-05-01', 'przyjecie-czysto',
              'przyjecie-czysto')"
        );
        expect::refused(
            room.execute(&ended_row, &[]),
            "osoba, kanał i obszar powstają jako bieżące",
        );
    }

    add_worker(&room, "przyjecie-czysto");
    add_channel(&room, "przyjecie-czysto");
    add_area(&room, "przyjecie-czysto");
    let worker_id = newest_worker_id(&room);
    let channels = wykonawcy_spis::kanaly_wykonawcow(&room).expect("read the channels");
    let channel_id = channels[0].id;
    let areas = wykonawcy_spis::obszary_wykonawcow(&room).expect("read the areas");
    let area_id = areas[0].id;
    decide(&room, "zmiana", DecyzjeRodzaj::ZmianaDanych);
    decide(&room, "wstrzymanie", DecyzjeRodzaj::Wstrzymanie);

    let worker_end = |end: &str, decision_id: &str| ZakonczOsobeWykonawcyParams {
        r#do: Some(end.to_string()),
        zakonczenie_decyzja_id: decision_id.to_string(),
        id: worker_id,
    };
    let channel_end = |decision_id: &str| ZakonczKanalWykonawcyParams {
        r#do: Some("2026-05-01".to_string()),
        zakonczenie_decyzja_id: decision_id.to_string(),
        id: channel_id,
    };
    let area_end = |decision_id: &str| ZakonczObszarWykonawcyParams {
        r#do: Some("2026-05-01".to_string()),
        zakonczenie_decyzja_id: decision_id.to_string(),
        id: area_id,
    };

    expect::refused(
        room.execute(
            &format!("UPDATE wykonawcy_spis_osoby SET do = '2026-05-01' WHERE id = {worker_id}"),
            &[],
        ),
        "koniec_z_decyzja",
    );
    expect::refused(
        wykonawcy_spis::zakoncz_osobe_wykonawcy(&room, &worker_end("2026-05-01", "wstrzymanie")),
        AFTER_END_DECISION,
    );
    expect::refused(
        wykonawcy_spis::zakoncz_kanal_wykonawcy(&room, &channel_end("wstrzymanie")),
        AFTER_END_DECISION,
    );
    expect::refused(
        wykonawcy_spis::zakoncz_obszar_wykonawcy(&room, &area_end("wstrzymanie")),
        AFTER_END_DECISION,
    );
    expect::refused(
        wykonawcy_spis::zakoncz_osobe_wykonawcy(&room, &worker_end("2026-01-01", "zmiana")),
        "koniec_nie_przed_poczatkiem",
    );

    wykonawcy_spis::zakoncz_osobe_wykonawcy(&room, &worker_end("2026-05-01", "zmiana"))
        .expect("end the worker");
    wykonawcy_spis::zakoncz_kanal_wykonawcy(&room, &channel_end("zmiana"))
        .expect("end the channel");
    wykonawcy_spis::zakoncz_obszar_wykonawcy(&room, &area_end("zmiana")).expect("end the area");

    let unchanged =
        "osoby, kanału ani obszaru się nie zmienia; mogą się tylko raz zakończyć decyzją";
    expect::refused(
        wykonawcy_spis::zakoncz_osobe_wykonawcy(&room, &worker_end("2026-06-01", "zmiana")),
        unchanged,
    );
    expect::refused(
        room.execute(
            &format!("UPDATE wykonawcy_spis_osoby SET telefon = '+48111' WHERE id = {worker_id}"),
            &[],
        ),
        unchanged,
    );
    expect::refused(
        room.execute(
            &format!("UPDATE wykonawcy_spis_kanaly SET adres = '+48111' WHERE id = {channel_id}"),
            &[],
        ),
        unchanged,
    );
    expect::refused(
        room.execute(
            &format!("UPDATE wykonawcy_spis_obszary SET obszar = 'Podgórze' WHERE id = {area_id}"),
            &[],
        ),
        unchanged,
    );
}

// States ------------------------------------------------------------------------------------------

/// Stan wykonawcy: wstrzymany ⇄ zatwierdzony, wykreślony na zawsze, każdy z decyzją
#[test]
fn contractor_states() {
    let room = new_room();
    admit(&room);
    decide(&room, "wstrzymanie", DecyzjeRodzaj::Wstrzymanie);
    decide(&room, "przywrocenie", DecyzjeRodzaj::Przywrocenie);
    decide(&room, "wykreslenie", DecyzjeRodzaj::Wykreslenie);
    let transition = "stan wykonawcy: zatwierdzony przy przyjęciu, wstrzymany ⇄ zatwierdzony";
    let decision_kind = "zatwierdzenie ma decyzję o przyjęciu albo przywróceniu";

    expect::refused(
        wykonawcy_spis::zmien_stan_wykonawcy(
            &room,
            &state_change("przywrocenie", StanyStan::Zatwierdzony),
        ),
        transition,
    );
    expect::refused(
        wykonawcy_spis::zmien_stan_wykonawcy(
            &room,
            &state_change("przywrocenie", StanyStan::Wstrzymany),
        ),
        decision_kind,
    );
    expect::refused(
        wykonawcy_spis::zmien_stan_wykonawcy(
            &room,
            &ZmienStanWykonawcyParams {
                powod: None,
                ..state_change("wstrzymanie", StanyStan::Wstrzymany)
            },
        ),
        "wstrzymanie_i_wykreslenie_z_powodem",
    );

    wykonawcy_spis::zmien_stan_wykonawcy(
        &room,
        &state_change("wstrzymanie", StanyStan::Wstrzymany),
    )
    .expect("suspend the contractor");
    expect::refused(
        wykonawcy_spis::zmien_stan_wykonawcy(
            &room,
            &state_change("wstrzymanie", StanyStan::Zatwierdzony),
        ),
        decision_kind,
    );
    wykonawcy_spis::zmien_stan_wykonawcy(
        &room,
        &ZmienStanWykonawcyParams {
            powod: None,
            ..state_change("przywrocenie", StanyStan::Zatwierdzony)
        },
    )
    .expect("restore the contractor");
    wykonawcy_spis::zmien_stan_wykonawcy(
        &room,
        &state_change("wykreslenie", StanyStan::Wykreslony),
    )
    .expect("remove the contractor");
    expect::refused(
        wykonawcy_spis::zmien_stan_wykonawcy(
            &room,
            &state_change("przywrocenie", StanyStan::Zatwierdzony),
        ),
        transition,
    );

    expect::refused(
        room.execute("UPDATE wykonawcy_spis_stany SET powod = 'inny'", &[]),
        UNCHANGED,
    );
}

/// Wykreślony wykonawca nie dostaje nowych danych, osób, kanałów ani obszarów
#[test]
fn removed_contractor() {
    let room = new_room();
    admit(&room);
    decide(&room, "zmiana", DecyzjeRodzaj::ZmianaDanych);
    decide(&room, "wstrzymanie", DecyzjeRodzaj::Wstrzymanie);
    decide(&room, "wykreslenie", DecyzjeRodzaj::Wykreslenie);

    wykonawcy_spis::zmien_stan_wykonawcy(
        &room,
        &state_change("wstrzymanie", StanyStan::Wstrzymany),
    )
    .expect("suspend the contractor");
    add_worker(&room, "zmiana");

    wykonawcy_spis::zmien_stan_wykonawcy(
        &room,
        &state_change("wykreslenie", StanyStan::Wykreslony),
    )
    .expect("remove the contractor");
    expect::refused(
        wykonawcy_spis::wpisz_osobe_wykonawcy(&room, &worker("zmiana")),
        NOT_REMOVED,
    );
    expect::refused(
        wykonawcy_spis::wpisz_kanal_wykonawcy(&room, &channel("zmiana")),
        NOT_REMOVED,
    );
    expect::refused(
        wykonawcy_spis::wpisz_obszar_wykonawcy(&room, &area("zmiana")),
        NOT_REMOVED,
    );
    expect::refused(
        wykonawcy_spis::wpisz_dane_wykonawcy(&room, &contractor_data("zmiana")),
        NOT_REMOVED,
    );
}

// Price lists -------------------------------------------------------------------------------------

/// Cennik: wersja po decyzji o cenniku powstaje jako projekt i nie obowiązuje wstecz
#[test]
fn price_list_version_rules() {
    let room = new_room();
    decide(&room, "cennik-decyzja", DecyzjeRodzaj::Cennik);
    decide(&room, "przyjecie", DecyzjeRodzaj::Przyjecie);
    add_price_list(&room);

    expect::refused(
        wykonawcy_cenniki::wpisz_wersje_cennika(
            &room,
            &WpiszWersjeCennikaParams {
                decyzja_id: "przyjecie".to_string(),
                ..version()
            },
        ),
        "wersję cennika wpisuje się po decyzji o cenniku",
    );
    expect::refused(
        wykonawcy_cenniki::wpisz_wersje_cennika(
            &room,
            &WpiszWersjeCennikaParams {
                obowiazuje_od: "2026-03-31".to_string(),
                ..version()
            },
        ),
        "obowiazuje_od_nie_przed_wpisem",
    );
    expect::refused(
        wykonawcy_cenniki::wpisz_wersje_cennika(
            &room,
            &WpiszWersjeCennikaParams {
                numer: 2,
                ..version()
            },
        ),
        "wersja cennika ma kolejny numer",
    );
    expect::refused(
        room.execute(
            "INSERT INTO wykonawcy_cenniki_wersje (id, cennik_id, numer, obowiazuje_od, wpisano, stan,
              decyzja_id)
            VALUES ('od-razu', 'cennik', 1, '2026-04-01', '2026-04-01T12:00:00Z', 'zamknieta',
              'cennik-decyzja')",
            &[],
        ),
        "wersja cennika powstaje jako projekt",
    );

    wykonawcy_cenniki::wpisz_wersje_cennika(&room, &version())
        .expect("add the first version of the price list");
    expect::refused(
        wykonawcy_cenniki::wpisz_wersje_cennika(
            &room,
            &WpiszWersjeCennikaParams {
                id: "wersja-2".to_string(),
                numer: 2,
                obowiazuje_od: "2026-04-15".to_string(),
                wpisano: "2026-04-02T12:00:00Z".to_string(),
                ..version()
            },
        ),
        "nie obowiązuje wcześniej niż poprzednia",
    );
    wykonawcy_cenniki::wpisz_wersje_cennika(
        &room,
        &WpiszWersjeCennikaParams {
            id: "wersja-2".to_string(),
            numer: 2,
            obowiazuje_od: "2026-05-01".to_string(),
            ..version()
        },
    )
    .expect("add the second version of the price list");
}

/// Cennik: pozycje tylko w projekcie; zamknięta wersja ma pozycje i się nie zmienia
#[test]
fn price_list_items() {
    let room = new_room();
    decide(&room, "cennik-decyzja", DecyzjeRodzaj::Cennik);
    add_price_list(&room);
    price_list_version(&room);

    let closing = ZamknijWersjeCennikaParams {
        id: "wersja-1".to_string(),
    };
    expect::refused(
        wykonawcy_cenniki::zamknij_wersje_cennika(&room, &closing),
        "zamknięta wersja cennika ma co najmniej jedną pozycję",
    );
    add_item(&room, 14000);

    let items_of_version = PozycjeCennikaParams {
        wersja_id: "wersja-1".to_string(),
    };
    let items = wykonawcy_cenniki::pozycje_cennika(&room, &items_of_version)
        .expect("read the items of the price list");
    let item_id = items[0].id;
    wykonawcy_cenniki::popraw_pozycje_cennika(
        &room,
        &PoprawPozycjeCennikaParams {
            usluga: "sprzątanie po wyjeździe gościa".to_string(),
            doplata: false,
            jednostka: "lokal do 40 m²".to_string(),
            cena: 15000,
            z_vat: true,
            warunki: None,
            id: item_id,
        },
    )
    .expect("correct the item of the price list");
    close_version(&room);

    expect::refused(
        wykonawcy_cenniki::dopisz_pozycje_cennika(
            &room,
            &DopiszPozycjeCennikaParams {
                wersja_id: "wersja-1".to_string(),
                usluga: "pranie".to_string(),
                doplata: false,
                jednostka: "komplet pościeli".to_string(),
                cena: 2000,
                z_vat: true,
                warunki: None,
            },
        ),
        "pozycje dopisuje się tylko do projektu wersji cennika",
    );
    expect::refused(
        room.execute(
            &format!("UPDATE wykonawcy_cenniki_pozycje SET cena = 1 WHERE id = {item_id}"),
            &[],
        ),
        "pozycji zamkniętej wersji cennika się nie zmienia",
    );
    expect::refused(
        room.execute(
            "UPDATE wykonawcy_cenniki_wersje SET obowiazuje_od = '2026-06-01' WHERE id = 'wersja-1'",
            &[],
        ),
        "wersji cennika się nie zmienia",
    );
    expect::refused(
        room.execute("UPDATE wykonawcy_cenniki_wpisy SET nazwa = 'inna'", &[]),
        "cennika się nie zmienia; zmiana stawek to nowa wersja",
    );

    let items_after_closing = wykonawcy_cenniki::pozycje_cennika(&room, &items_of_version)
        .expect("read the items of the price list");
    assert_eq!(items_after_closing[0].cena, 15000);
}

/// Przypisanie cennika: zatwierdzonemu wykonawcy, z zamkniętą wersją i decyzjami
#[test]
fn price_list_assignment() {
    let room = new_room();
    admit(&room);
    decide(&room, "cennik-decyzja", DecyzjeRodzaj::Cennik);
    decide(&room, "przypisanie", DecyzjeRodzaj::Przypisanie);
    decide(&room, "koniec-przypisania", DecyzjeRodzaj::Przypisanie);
    add_price_list(&room);
    price_list_version(&room);
    let assignment = PrzypiszCennikParams {
        cennik_id: "cennik".to_string(),
        wykonawca_id: "czysto".to_string(),
        od: "2026-04-02".to_string(),
        decyzja_id: "przypisanie".to_string(),
    };

    expect::refused(
        wykonawcy_cenniki::przypisz_cennik(&room, &assignment),
        "przypisuje się cennik, który ma zamkniętą wersję",
    );
    add_item(&room, 15000);
    close_version(&room);
    expect::refused(
        wykonawcy_cenniki::przypisz_cennik(
            &room,
            &PrzypiszCennikParams {
                decyzja_id: "cennik-decyzja".to_string(),
                ..assignment.clone()
            },
        ),
        "przypisanie cennika i jego koniec wpisuje się po decyzji o przypisaniu",
    );
    expect::refused(
        room.execute(
            "INSERT INTO wykonawcy_cenniki_przypisania (cennik_id, wykonawca_id, od, do, decyzja_id,
              zakonczenie_decyzja_id)
            VALUES ('cennik', 'czysto', '2026-04-02', '2026-05-01', 'przypisanie', 'przypisanie')",
            &[],
        ),
        "przypisanie cennika powstaje jako bieżące",
    );

    wykonawcy_cenniki::przypisz_cennik(&room, &assignment).expect("assign the price list");
    let assignments = wykonawcy_cenniki::przypisania_cennikow(&room).expect("read the assignments");
    let assignment_id = assignments[0].id;
    let assignment_end = |decision_id: &str| ZakonczPrzypisanieCennikaParams {
        r#do: Some("2026-06-01".to_string()),
        zakonczenie_decyzja_id: decision_id.to_string(),
        id: assignment_id,
    };
    expect::refused(
        wykonawcy_cenniki::zakoncz_przypisanie_cennika(&room, &assignment_end("cennik-decyzja")),
        "przypisanie cennika i jego koniec wpisuje się po decyzji o przypisaniu",
    );
    wykonawcy_cenniki::zakoncz_przypisanie_cennika(&room, &assignment_end("koniec-przypisania"))
        .expect("end the assignment");
    expect::refused(
        room.execute(
            &format!(
                "UPDATE wykonawcy_cenniki_przypisania SET do = '2026-07-01' WHERE id = {assignment_id}"
            ),
            &[],
        ),
        "przypisania cennika się nie zmienia",
    );

    decide(&room, "wstrzymanie", DecyzjeRodzaj::Wstrzymanie);
    wykonawcy_spis::zmien_stan_wykonawcy(
        &room,
        &state_change("wstrzymanie", StanyStan::Wstrzymany),
    )
    .expect("suspend the contractor");
    expect::refused(
        wykonawcy_cenniki::przypisz_cennik(&room, &assignment),
        "cennik przypisuje się zatwierdzonemu wykonawcy",
    );
}

// R9 and replacement ------------------------------------------------------------------------------

/// SQL that inserts again the first row of the table, as a careless upsert would.
fn first_row_copy(room: &MemoryRoom, table: &str) -> String {
    let columns_info = room
        .execute(&format!("PRAGMA table_info({table})"), &[])
        .expect("read the columns");
    let column_names: Vec<String> = columns_info
        .iter()
        .map(|row| row.get("name").expect("a column has a name"))
        .collect();
    let columns = column_names.join(", ");

    format!(
        "INTO {table} ({columns}) SELECT {columns} FROM {table} \
        WHERE rowid = (SELECT min(rowid) FROM {table})"
    )
}

/// R9: wierszy rejestru wykonawców się nie usuwa ani nie zastępuje przez REPLACE albo upsert
#[test]
fn no_deletion_or_replacement() {
    let room = new_room();
    full_register(&room);

    let table_rows = room
        .execute(
            r"SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'wykonawcy\_%' ESCAPE '\'",
            &[],
        )
        .expect("read the tables");
    let tables: Vec<String> = table_rows
        .iter()
        .map(|row| row.get("name").expect("a table has a name"))
        .collect();
    assert_eq!(tables.len(), 11, "{tables:?}");

    for table in &tables {
        let row_count = room
            .value(&format!("SELECT count(*) FROM {table}"), &[])
            .expect("count the rows");
        let row_count = i64::from_sql_value(&row_count).expect("a count is a number");
        assert!(row_count > 0, "{table} has no row");

        let row_copy = first_row_copy(&room, table);
        expect::refused(
            room.execute(&format!("INSERT OR REPLACE {row_copy}"), &[]),
            REPLACEMENT,
        );
        expect::refused(
            room.execute(
                &format!("INSERT {row_copy} ON CONFLICT (id) DO UPDATE SET id = excluded.id"),
                &[],
            ),
            REPLACEMENT,
        );
        expect::refused(
            room.execute(
                &format!("DELETE FROM {table} WHERE rowid = (SELECT min(rowid) FROM {table})"),
                &[],
            ),
            DELETION,
        );
    }

    // The version number is a UNIQUE key of its own: a copy under a new key replaces by it.
    expect::refused(
        room.execute(
            "INSERT OR REPLACE INTO wykonawcy_cenniki_wersje (id, cennik_id, numer, obowiazuje_od,
              wpisano, stan, decyzja_id)
            SELECT 'nowy-klucz', cennik_id, numer, obowiazuje_od, wpisano, stan, decyzja_id
            FROM wykonawcy_cenniki_wersje
            WHERE id = 'wersja-1'",
            &[],
        ),
        REPLACEMENT,
    );
}
