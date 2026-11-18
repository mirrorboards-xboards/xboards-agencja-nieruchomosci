//! Scenarios of the office's catalogue of offers (spec/registers/katalog_ofert).
//!
//! Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
//! allows and a write it refuses. The first scenario follows a public sale offer from its entry to the
//! sale, with a price change, an advertisement and a reservation that falls through.

use xboards_agencja_nieruchomosci::registers::katalog_ofert::{self, katalog_ofert_oferty};
use xboards_agencja_nieruchomosci::xmeta_room::MemoryRoom;

use katalog_ofert_oferty::{
    OfertaPoNumerzeParams, OfertaPoOgloszeniuParams, StatusyOfertyParams, StatusyStatus,
    WersjeOfertyParams, WersjeWidocznosc, WpisyRodzajNieruchomosci, WpisyRodzajOferty,
    WpiszOferteParams, WpiszOgloszenieParams, WpiszWersjeOfertyParams, ZakonczOgloszenieParams,
    ZmienStatusOfertyParams,
};

use crate::expect;

const REPLACEMENT: &str =
    "wiersza katalogu nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE";
const DELETION: &str = "wierszy katalogu się nie usuwa (R9)";
const UNCHANGED: &str = "wpisu oferty, wersji ani statusu się nie zmienia";
const TRANSITION: &str = "status oferty: aktywna ⇄ zarezerwowana → sprzedana albo wynajęta";
const NO_ADVERTISEMENTS: &str = "oferta niepubliczna nie ma ogłoszeń";
const FINISHED: &str = "zakończona oferta nie dostaje nowych wersji ani ogłoszeń";

fn new_room() -> MemoryRoom {
    katalog_ofert::open_in_memory().expect("open the catalogue of offers")
}

/// The names of the columns of a table, in the order of the table.
fn column_names_of(room: &MemoryRoom, table: &str) -> Vec<String> {
    let columns = room
        .execute(&format!("PRAGMA table_info({table})"), &[])
        .expect("read the columns");

    columns
        .iter()
        .map(|column| column.get::<String>("name").expect("a column has a name"))
        .collect()
}

// Building blocks ---------------------------------------------------------------------------------

/// The default parameters of a sale offer of a flat in Jeżyce, published without exclusivity; its
/// case file is `teczka-<offer_id>`.
fn offer(offer_id: &str, numer_oferty: &str) -> WpiszOferteParams {
    WpiszOferteParams {
        id: offer_id.to_string(),
        numer_oferty: numer_oferty.to_string(),
        teczka_odnosnik: format!("teczka-{offer_id}"),
        rodzaj_oferty: WpisyRodzajOferty::Sprzedaz,
        rodzaj_nieruchomosci: WpisyRodzajNieruchomosci::Lokal,
        wpisal: "kierownik-posrednictwa".to_string(),
        wpisano: "2026-05-04".to_string(),
        od: "2026-05-04".to_string(),
        powierzchnia: 54.3,
        lokalizacja_przyblizona: "Poznań, Jeżyce".to_string(),
        cena_sprzedazy: Some(62_000_000),
        czynsz: None,
        opis: "Trzy pokoje, balkon, drugie piętro.".to_string(),
        zdjecia_sciezka: Some("zdjecia/oferta-031".to_string()),
        zdjecia_skrot: Some("sha256-zdjec".to_string()),
        widocznosc: WersjeWidocznosc::Publiczna,
        granice_zgody: None,
        wylacznosc: false,
        zrodlo_odnosnik: "teczka-oferty/zgoda-na-opis".to_string(),
    }
}

/// The default parameters of the offer `oferta` in the case file `teczka-oferty`, which the
/// scenarios of the rules of a version override.
fn sale_offer() -> WpiszOferteParams {
    WpiszOferteParams {
        teczka_odnosnik: "teczka-oferty".to_string(),
        ..offer("oferta", "2026/031")
    }
}

/// The default parameters of a new version of an offer, with the content of its first version.
fn version(offer_id: &str) -> WpiszWersjeOfertyParams {
    WpiszWersjeOfertyParams {
        oferta_id: offer_id.to_string(),
        od: "2026-05-04".to_string(),
        powierzchnia: 54.3,
        lokalizacja_przyblizona: "Poznań, Jeżyce".to_string(),
        cena_sprzedazy: Some(62_000_000),
        czynsz: None,
        opis: "Trzy pokoje, balkon, drugie piętro.".to_string(),
        zdjecia_sciezka: Some("zdjecia/oferta-031".to_string()),
        zdjecia_skrot: Some("sha256-zdjec".to_string()),
        widocznosc: WersjeWidocznosc::Publiczna,
        granice_zgody: None,
        wylacznosc: false,
        zrodlo_odnosnik: "teczka-oferty/zgoda-na-opis".to_string(),
        aneks_odnosnik: None,
        poprawia_id: None,
        wpisal: "kierownik-posrednictwa".to_string(),
    }
}

fn add_offer(room: &MemoryRoom, params: &WpiszOferteParams) {
    katalog_ofert_oferty::wpisz_oferte(room, params).expect("enter the offer");
}

fn new_version(room: &MemoryRoom, params: &WpiszWersjeOfertyParams) {
    katalog_ofert_oferty::wpisz_wersje_oferty(room, params).expect("enter a version of the offer");
}

/// Advertises an offer as OTO-123 on otodom from 2026-05-05.
fn advertise(room: &MemoryRoom, offer_id: &str) {
    let params = WpiszOgloszenieParams {
        oferta_id: offer_id.to_string(),
        portal: "otodom".to_string(),
        numer_ogloszenia: "OTO-123".to_string(),
        od: "2026-05-05".to_string(),
    };

    katalog_ofert_oferty::wpisz_ogloszenie(room, &params).expect("advertise the offer");
}

/// Changes the status of an offer after an interested party's offer.
fn change_status(room: &MemoryRoom, offer_id: &str, status: StatusyStatus, od: &str) {
    let params = ZmienStatusOfertyParams {
        oferta_id: offer_id.to_string(),
        status,
        od: od.to_string(),
        zrodlo_odnosnik: "teczka-oferty/oferta-zainteresowanego".to_string(),
        wpisal: "kierownik-posrednictwa".to_string(),
    };

    katalog_ofert_oferty::zmien_status_oferty(room, &params).expect("change the offer's status");
}

/// The parameters of a status change after the agency agreement, for the scenarios of refusals.
fn status_change(offer_id: &str, status: StatusyStatus, od: &str) -> ZmienStatusOfertyParams {
    ZmienStatusOfertyParams {
        oferta_id: offer_id.to_string(),
        status,
        od: od.to_string(),
        zrodlo_odnosnik: "teczka-oferty/umowa".to_string(),
        wpisal: "kierownik-posrednictwa".to_string(),
    }
}

/// The newest status of an offer.
fn status_of(room: &MemoryRoom, offer_id: &str) -> StatusyStatus {
    let of_offer = StatusyOfertyParams {
        oferta_id: offer_id.to_string(),
    };
    let statuses =
        katalog_ofert_oferty::statusy_oferty(room, &of_offer).expect("read the statuses");

    statuses.last().expect("the offer has a status").status
}

// A whole offer -----------------------------------------------------------------------------------

/// Oferta: wpis, ogłoszenie, nowa cena, rezerwacja, powrót do aktywnych i sprzedaż
#[test]
fn whole_offer() {
    let room = new_room();
    add_offer(&room, &offer("oferta", "2026/031"));
    advertise(&room, "oferta");
    new_version(
        &room,
        &WpiszWersjeOfertyParams {
            od: "2026-05-20".to_string(),
            cena_sprzedazy: Some(59_000_000),
            ..version("oferta")
        },
    );

    change_status(&room, "oferta", StatusyStatus::Zarezerwowana, "2026-06-01");
    change_status(&room, "oferta", StatusyStatus::Aktywna, "2026-06-10");
    change_status(&room, "oferta", StatusyStatus::Zarezerwowana, "2026-06-20");
    change_status(
        &room,
        "oferta",
        StatusyStatus::SprzedanaAlboWynajeta,
        "2026-07-15",
    );

    let offers = katalog_ofert_oferty::oferty(&room).expect("read the offers");
    let summaries: Vec<(&str, Option<i64>, StatusyStatus)> = offers
        .iter()
        .map(|row| (row.numer_oferty.as_str(), row.cena_sprzedazy, row.status))
        .collect();
    assert_eq!(
        summaries,
        [(
            "2026/031",
            Some(59_000_000),
            StatusyStatus::SprzedanaAlboWynajeta
        )]
    );

    let by_number = katalog_ofert_oferty::oferta_po_numerze(
        &room,
        &OfertaPoNumerzeParams {
            numer_oferty: "2026/031".to_string(),
        },
    )
    .expect("read the offer by its number")
    .expect("the offer has the number");
    assert_eq!(by_number.teczka_odnosnik, "teczka-oferta");

    let found = katalog_ofert_oferty::oferta_po_ogloszeniu(
        &room,
        &OfertaPoOgloszeniuParams {
            portal: "otodom".to_string(),
            numer_ogloszenia: "OTO-123".to_string(),
        },
    )
    .expect("read the offer by its advertisement")
    .expect("the advertisement exists");
    assert_eq!(found.oferta_id, "oferta");

    let versions = katalog_ofert_oferty::wersje_oferty(
        &room,
        &WersjeOfertyParams {
            oferta_id: "oferta".to_string(),
        },
    )
    .expect("read the versions");
    assert_eq!(versions.len(), 2);
}

// Versions ----------------------------------------------------------------------------------------

/// Wersja: kwoty pasują do rodzaju oferty
#[test]
fn amounts_match_kind() {
    let room = new_room();
    let amounts = "oferta sprzedaży ma cenę, oferta najmu czynsz";

    let sale_without_price = WpiszOferteParams {
        cena_sprzedazy: None,
        ..sale_offer()
    };
    expect::refused(
        katalog_ofert_oferty::wpisz_oferte(&room, &sale_without_price),
        amounts,
    );
    let sale_with_rent = WpiszOferteParams {
        czynsz: Some(320_000),
        ..sale_offer()
    };
    expect::refused(
        katalog_ofert_oferty::wpisz_oferte(&room, &sale_with_rent),
        amounts,
    );
    let rent_without_rent = WpiszOferteParams {
        rodzaj_oferty: WpisyRodzajOferty::Najem,
        ..sale_offer()
    };
    expect::refused(
        katalog_ofert_oferty::wpisz_oferte(&room, &rent_without_rent),
        amounts,
    );
    let both_without_rent = WpiszOferteParams {
        rodzaj_oferty: WpisyRodzajOferty::SprzedazINajem,
        ..sale_offer()
    };
    expect::refused(
        katalog_ofert_oferty::wpisz_oferte(&room, &both_without_rent),
        amounts,
    );

    add_offer(&room, &offer("oferta", "2026/031"));
    add_offer(
        &room,
        &WpiszOferteParams {
            rodzaj_oferty: WpisyRodzajOferty::Najem,
            cena_sprzedazy: None,
            czynsz: Some(320_000),
            ..offer("najem", "2026/032")
        },
    );
    add_offer(
        &room,
        &WpiszOferteParams {
            rodzaj_oferty: WpisyRodzajOferty::SprzedazINajem,
            czynsz: Some(320_000),
            ..offer("oba", "2026/033")
        },
    );

    let offers = katalog_ofert_oferty::oferty(&room).expect("read the offers");
    assert_eq!(offers.len(), 3);
}

/// Wersja: oferta niepubliczna ma granice zgody, a publiczna ich nie ma
#[test]
fn consent_limits() {
    let room = new_room();
    let limits = "granice_zgody_tylko_przy_niepublicznej";

    let private_without_limits = WpiszOferteParams {
        widocznosc: WersjeWidocznosc::Niepubliczna,
        ..sale_offer()
    };
    expect::refused(
        katalog_ofert_oferty::wpisz_oferte(&room, &private_without_limits),
        limits,
    );
    let public_with_limits = WpiszOferteParams {
        granice_zgody: Some("tylko klienci z kredytem".to_string()),
        ..sale_offer()
    };
    expect::refused(
        katalog_ofert_oferty::wpisz_oferte(&room, &public_with_limits),
        limits,
    );
    let photos_without_hash = WpiszOferteParams {
        zdjecia_skrot: None,
        ..sale_offer()
    };
    expect::refused(
        katalog_ofert_oferty::wpisz_oferte(&room, &photos_without_hash),
        "zdjecia_ze_skrotem",
    );

    add_offer(
        &room,
        &WpiszOferteParams {
            widocznosc: WersjeWidocznosc::Niepubliczna,
            granice_zgody: Some(
                "tylko klientom biura z potwierdzonym finansowaniem, bez zdjęć wnętrz".to_string(),
            ),
            ..offer("oferta", "2026/031")
        },
    );
}

/// Wersja: zmiana widoczności albo wyłączności wymaga aneksu do umowy
#[test]
fn visibility_change_needs_annex() {
    let room = new_room();
    add_offer(&room, &offer("oferta", "2026/031"));
    let private = WpiszWersjeOfertyParams {
        widocznosc: WersjeWidocznosc::Niepubliczna,
        granice_zgody: Some("tylko klientom biura".to_string()),
        od: "2026-05-10".to_string(),
        ..version("oferta")
    };
    let annex = "zmiana widoczności albo wyłączności wymaga aneksu do umowy pośrednictwa";

    expect::refused(
        katalog_ofert_oferty::wpisz_wersje_oferty(&room, &private),
        annex,
    );
    let exclusive_without_annex = WpiszWersjeOfertyParams {
        wylacznosc: true,
        ..version("oferta")
    };
    expect::refused(
        katalog_ofert_oferty::wpisz_wersje_oferty(&room, &exclusive_without_annex),
        annex,
    );

    new_version(
        &room,
        &WpiszWersjeOfertyParams {
            cena_sprzedazy: Some(60_000_000),
            od: "2026-05-10".to_string(),
            ..version("oferta")
        },
    );
    new_version(
        &room,
        &WpiszWersjeOfertyParams {
            aneks_odnosnik: Some("teczka-oferty/aneks-1".to_string()),
            ..private
        },
    );

    let offers = katalog_ofert_oferty::oferty(&room).expect("read the offers");
    assert_eq!(offers[0].widocznosc, WersjeWidocznosc::Niepubliczna);
}

/// Wersja: nie obowiązuje wcześniej niż poprzednia i poprawia wersję tej samej oferty
#[test]
fn version_order_and_correction() {
    let room = new_room();
    add_offer(&room, &offer("oferta", "2026/031"));
    add_offer(&room, &offer("druga", "2026/032"));
    let other_versions = katalog_ofert_oferty::wersje_oferty(
        &room,
        &WersjeOfertyParams {
            oferta_id: "druga".to_string(),
        },
    )
    .expect("read the versions of the other offer");
    let other_version = other_versions[0].id;
    let own_versions = katalog_ofert_oferty::wersje_oferty(
        &room,
        &WersjeOfertyParams {
            oferta_id: "oferta".to_string(),
        },
    )
    .expect("read the versions of the offer");
    let own_version = own_versions[0].id;

    let earlier_version = WpiszWersjeOfertyParams {
        od: "2026-05-01".to_string(),
        ..version("oferta")
    };
    expect::refused(
        katalog_ofert_oferty::wpisz_wersje_oferty(&room, &earlier_version),
        "nowa wersja oferty nie obowiązuje wcześniej niż poprzednia",
    );
    let correction_of_other_offer = WpiszWersjeOfertyParams {
        poprawia_id: Some(other_version),
        ..version("oferta")
    };
    expect::refused(
        katalog_ofert_oferty::wpisz_wersje_oferty(&room, &correction_of_other_offer),
        "wersja poprawia wcześniejszą wersję tej samej oferty",
    );

    new_version(
        &room,
        &WpiszWersjeOfertyParams {
            powierzchnia: 53.4,
            poprawia_id: Some(own_version),
            ..version("oferta")
        },
    );
    expect::refused(
        room.execute("UPDATE katalog_ofert_oferty_wersje SET opis = 'inny'", &[]),
        UNCHANGED,
    );
    expect::refused(
        room.execute(
            "UPDATE katalog_ofert_oferty_wpisy SET numer_oferty = '2026/099'",
            &[],
        ),
        UNCHANGED,
    );
}

// Advertisements ----------------------------------------------------------------------------------

/// Ogłoszenia: tylko oferty publicznej, a przejście na niepubliczną po ich końcu
#[test]
fn advertisements_only_public() {
    let room = new_room();
    add_offer(
        &room,
        &WpiszOferteParams {
            widocznosc: WersjeWidocznosc::Niepubliczna,
            granice_zgody: Some("tylko klientom biura".to_string()),
            ..offer("oferta", "2026/031")
        },
    );
    let advertisement_of_private = WpiszOgloszenieParams {
        oferta_id: "oferta".to_string(),
        portal: "otodom".to_string(),
        numer_ogloszenia: "OTO-1".to_string(),
        od: "2026-05-05".to_string(),
    };
    expect::refused(
        katalog_ofert_oferty::wpisz_ogloszenie(&room, &advertisement_of_private),
        NO_ADVERTISEMENTS,
    );

    add_offer(&room, &offer("publiczna", "2026/032"));
    advertise(&room, "publiczna");
    let private = WpiszWersjeOfertyParams {
        widocznosc: WersjeWidocznosc::Niepubliczna,
        granice_zgody: Some("tylko klientom biura".to_string()),
        aneks_odnosnik: Some("teczka/aneks".to_string()),
        od: "2026-06-01".to_string(),
        ..version("publiczna")
    };
    expect::refused(
        katalog_ofert_oferty::wpisz_wersje_oferty(&room, &private),
        NO_ADVERTISEMENTS,
    );

    let advertisements =
        katalog_ofert_oferty::ogloszenia_ofert(&room).expect("read the advertisements");
    let advertisement_id = advertisements[0].id;
    katalog_ofert_oferty::zakoncz_ogloszenie(
        &room,
        &ZakonczOgloszenieParams {
            r#do: Some("2026-05-31".to_string()),
            id: advertisement_id,
        },
    )
    .expect("end the advertisement");
    new_version(&room, &private);
}

/// Ogłoszenia: powstają trwające, kończą się raz, a numer portalu jest jeden
#[test]
fn advertisement_rows() {
    let room = new_room();
    add_offer(&room, &offer("oferta", "2026/031"));

    expect::refused(
        room.execute(
            "INSERT INTO katalog_ofert_oferty_ogloszenia (oferta_id, portal, numer_ogloszenia, od, do)
            VALUES ('oferta', 'otodom', 'OTO-1', '2026-05-05', '2026-05-06')",
            &[],
        ),
        "ogłoszenie powstaje jako trwające",
    );
    advertise(&room, "oferta");
    let same_number_again = WpiszOgloszenieParams {
        oferta_id: "oferta".to_string(),
        portal: "otodom".to_string(),
        numer_ogloszenia: "OTO-123".to_string(),
        od: "2026-05-06".to_string(),
    };
    expect::refused(
        katalog_ofert_oferty::wpisz_ogloszenie(&room, &same_number_again),
        REPLACEMENT,
    );

    let advertisements =
        katalog_ofert_oferty::ogloszenia_ofert(&room).expect("read the advertisements");
    let advertisement_id = advertisements[0].id;
    let end_before_start = ZakonczOgloszenieParams {
        r#do: Some("2026-05-01".to_string()),
        id: advertisement_id,
    };
    expect::refused(
        katalog_ofert_oferty::zakoncz_ogloszenie(&room, &end_before_start),
        "koniec_nie_przed_poczatkiem",
    );
    katalog_ofert_oferty::zakoncz_ogloszenie(
        &room,
        &ZakonczOgloszenieParams {
            r#do: Some("2026-05-31".to_string()),
            id: advertisement_id,
        },
    )
    .expect("end the advertisement");
    let second_end = ZakonczOgloszenieParams {
        r#do: Some("2026-06-30".to_string()),
        id: advertisement_id,
    };
    expect::refused(
        katalog_ofert_oferty::zakoncz_ogloszenie(&room, &second_end),
        "ogłoszenia się nie zmienia; może się tylko raz zakończyć",
    );
}

// Status ------------------------------------------------------------------------------------------

/// Status: aktywna ⇄ zarezerwowana → sprzedana, z każdego do wycofanej, końcowe ostateczne
#[test]
fn status_transitions() {
    let room = new_room();
    add_offer(&room, &offer("oferta", "2026/031"));

    let sold_while_active =
        status_change("oferta", StatusyStatus::SprzedanaAlboWynajeta, "2026-06-01");
    expect::refused(
        katalog_ofert_oferty::zmien_status_oferty(&room, &sold_while_active),
        TRANSITION,
    );
    let active_again = status_change("oferta", StatusyStatus::Aktywna, "2026-06-01");
    expect::refused(
        katalog_ofert_oferty::zmien_status_oferty(&room, &active_again),
        TRANSITION,
    );
    change_status(&room, "oferta", StatusyStatus::Zarezerwowana, "2026-06-01");
    let reserved_again = status_change("oferta", StatusyStatus::Zarezerwowana, "2026-06-01");
    expect::refused(
        katalog_ofert_oferty::zmien_status_oferty(&room, &reserved_again),
        TRANSITION,
    );
    change_status(&room, "oferta", StatusyStatus::Wycofana, "2026-06-15");
    let active_after_withdrawal = status_change("oferta", StatusyStatus::Aktywna, "2026-07-01");
    expect::refused(
        katalog_ofert_oferty::zmien_status_oferty(&room, &active_after_withdrawal),
        TRANSITION,
    );

    add_offer(&room, &offer("druga", "2026/032"));
    change_status(&room, "druga", StatusyStatus::Wycofana, "2026-06-01");
    assert_eq!(status_of(&room, "druga"), StatusyStatus::Wycofana);

    // A status written past the operation, for an offer without the first one.
    room.execute(
        "INSERT INTO katalog_ofert_oferty_wpisy (id, numer_oferty, teczka_odnosnik, rodzaj_oferty,
          rodzaj_nieruchomosci, wpisal, wpisano)
        VALUES ('bez-statusu', '2026/040', 'teczka', 'sprzedaz', 'lokal', 'kierownik', '2026-05-04')",
        &[],
    )
    .expect("enter an offer without a status");
    let reserved_without_status =
        status_change("bez-statusu", StatusyStatus::Zarezerwowana, "2026-06-01");
    expect::refused(
        katalog_ofert_oferty::zmien_status_oferty(&room, &reserved_without_status),
        TRANSITION,
    );
    expect::refused(
        room.execute(
            "UPDATE katalog_ofert_oferty_statusy SET od = '2026-01-01'",
            &[],
        ),
        UNCHANGED,
    );
}

/// Status nie zmienia się przed poprzednim; zakończona oferta nie dostaje wersji ani ogłoszeń
#[test]
fn finished_offer() {
    let room = new_room();
    add_offer(&room, &offer("oferta", "2026/031"));

    let reserved_before_entry = status_change("oferta", StatusyStatus::Zarezerwowana, "2026-05-01");
    expect::refused(
        katalog_ofert_oferty::zmien_status_oferty(&room, &reserved_before_entry),
        "status oferty nie zmienia się przed poprzednim",
    );
    change_status(&room, "oferta", StatusyStatus::Wycofana, "2026-06-01");

    let version_after_withdrawal = WpiszWersjeOfertyParams {
        od: "2026-06-02".to_string(),
        ..version("oferta")
    };
    expect::refused(
        katalog_ofert_oferty::wpisz_wersje_oferty(&room, &version_after_withdrawal),
        FINISHED,
    );
    let advertisement_after_withdrawal = WpiszOgloszenieParams {
        oferta_id: "oferta".to_string(),
        portal: "otodom".to_string(),
        numer_ogloszenia: "OTO-9".to_string(),
        od: "2026-06-02".to_string(),
    };
    expect::refused(
        katalog_ofert_oferty::wpisz_ogloszenie(&room, &advertisement_after_withdrawal),
        FINISHED,
    );

    let offers = katalog_ofert_oferty::oferty(&room).expect("read the offers");
    assert_eq!(offers[0].status, StatusyStatus::Wycofana);
}

// R9 and replacement ------------------------------------------------------------------------------

/// SQL that inserts again the first row of the table, as a careless upsert would.
fn first_row_copy(room: &MemoryRoom, table: &str) -> String {
    let columns = column_names_of(room, table).join(", ");

    format!(
        "INTO {table} ({columns}) SELECT {columns} FROM {table} \
        WHERE rowid = (SELECT min(rowid) FROM {table})"
    )
}

/// R9: wierszy katalogu się nie usuwa ani nie zastępuje przez REPLACE albo upsert
#[test]
fn no_deletion_or_replacement() {
    let room = new_room();
    add_offer(&room, &offer("oferta", "2026/031"));
    advertise(&room, "oferta");

    let table_rows = room
        .execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'katalog\\_%' ESCAPE '\\'",
            &[],
        )
        .expect("read the tables");
    let tables: Vec<String> = table_rows
        .iter()
        .map(|row| row.get::<String>("name").expect("a table has a name"))
        .collect();
    assert_eq!(tables.len(), 4, "{tables:?}");

    for table in &tables {
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

    // The offer number and the portal's advertisement number are UNIQUE keys of their own.
    expect::refused(
        room.execute(
            "INSERT OR REPLACE INTO katalog_ofert_oferty_wpisy (id, numer_oferty, teczka_odnosnik,
              rodzaj_oferty, rodzaj_nieruchomosci, wpisal, wpisano)
            VALUES ('nowy-klucz', '2026/031', 'teczka', 'sprzedaz', 'lokal', 'kierownik', '2026-05-04')",
            &[],
        ),
        REPLACEMENT,
    );
    expect::refused(
        room.execute(
            "INSERT OR REPLACE INTO katalog_ofert_oferty_ogloszenia (oferta_id, portal,
              numer_ogloszenia, od)
            VALUES ('oferta', 'otodom', 'OTO-123', '2026-05-06')",
            &[],
        ),
        REPLACEMENT,
    );
}
