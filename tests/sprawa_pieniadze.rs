//! Scenarios of the core kinds around clients' money (spec/cases/sprawa): accounts
//! (sprawa-rachunki), Kontroler's checks (sprawa-sprawdzenia) and payouts (sprawa-wyplaty).
//!
//! Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
//! allows and a write it refuses. Together they follow R4 and D13: a new account number is a new row,
//! a human admits an account only after Kontroler's check and a confirmation by another channel, and
//! every payout waits for a fresh check of the account and for the human's consent.

use std::collections::BTreeSet;

use xboards_agencja_nieruchomosci::cases::sprawa::{
    sprawa_decyzje, sprawa_dokumenty, sprawa_metryka, sprawa_rachunki, sprawa_sprawdzenia,
    sprawa_terminy, sprawa_uczestnicy, sprawa_umocowania, sprawa_wyplaty,
};
use xboards_agencja_nieruchomosci::xmeta_room::{FromSqlValue, MetaRoom, SqlValue};

use sprawa_decyzje::{
    PoprosODecyzjeParams, RozstrzygnijDecyzjeParams, WpisyRodzaj as RodzajDecyzji,
    WpisyRozstrzygniecie as Rozstrzygniecie,
};
use sprawa_dokumenty::WpisyRodzaj as RodzajDokumentu;
use sprawa_metryka::{PostawWatpliwoscParams, ZdejmijWatpliwoscParams, ZmienEtapSprawyParams};
use sprawa_rachunki::{WpisyCel as CelRachunku, WpiszRachunekParams, ZastapRachunekParams};
use sprawa_sprawdzenia::{
    WpisyRodzaj as RodzajSprawdzenia, WpisyWynik as WynikSprawdzenia, WpiszSprawdzenieParams,
};
use sprawa_terminy::{
    WpisyPilnujeStanowisko as PilnujeStanowisko, WpisyRodzaj as RodzajTerminu, WpiszTerminParams,
};
use sprawa_uczestnicy::{
    AdresyRodzaj, WpisyKanalKontaktu as KanalKontaktu, WpiszAdresUczestnikaParams,
    WpiszObywatelstwoUczestnikaParams, WpiszUczestnikaParams, ZmienKontaktUczestnikaParams,
};
use sprawa_umocowania::DopiszCzynnoscUmocowaniaParams;
use sprawa_wyplaty::{
    AnulujWyplateParams, OdnotujWykonanieWyplatyParams, PrzygotujWyplateParams,
    WskazSprawdzenieWyplatyParams, WskazZgodeNaWyplateParams,
};

use crate::expect;
use crate::sprawa::{
    account, add_account, add_flat, add_person, add_power_for_office, add_received_document,
    admitted_account, approve, approved_decision, ask_for_decision, check, check_identity,
    check_of, checked_and_approved_payout, confirm_account, decision_request_about, flat, new_room,
    open_case, payout, person, prepare_payout, received_document, settle_decision, settlement,
    version_of,
};

/// A text value to bind in raw SQL.
fn text(value: &str) -> SqlValue {
    SqlValue::Text(value.to_string())
}

/// The default parameters of the account `account_id` of `person_id` (see [`account`]) with the
/// number `numer`.
fn account_numbered(account_id: &str, person_id: &str, numer: &str) -> WpiszRachunekParams {
    WpiszRachunekParams {
        numer: numer.to_string(),
        ..account(account_id, person_id)
    }
}

/// The agent's replacement of the account `account_id` by `zastapiony_przez_id` at `zastapiono`.
fn replacement(
    account_id: &str,
    zastapiony_przez_id: &str,
    zastapiono: &str,
) -> ZastapRachunekParams {
    ZastapRachunekParams {
        zastapiony_przez_id: zastapiony_przez_id.to_string(),
        zastapil: "tozsamosc-agenta".to_string(),
        zastapiono: zastapiono.to_string(),
        id: account_id.to_string(),
    }
}

/// Replaces the account of `params` (see [`replacement`]).
fn replace_account(room: &impl MetaRoom, params: &ZastapRachunekParams) {
    sprawa_rachunki::zastap_rachunek(room, params).expect("replace the account");
}

/// The parameters of Kontroler's check `check_id` of the account `account_id` (see [`check_of`]).
fn account_check(
    check_id: &str,
    account_id: &str,
    sprawdzono: &str,
    wynik: WynikSprawdzenia,
) -> WpiszSprawdzenieParams {
    WpiszSprawdzenieParams {
        rachunek_id: Some(account_id.to_string()),
        ..check_of(check_id, RodzajSprawdzenia::Rachunek, sprawdzono, wynik)
    }
}

/// Points the payout `payout_id` to the check `check_id` of its account.
fn point_payout_to_check(room: &impl MetaRoom, payout_id: &str, check_id: &str) {
    let params = WskazSprawdzenieWyplatyParams {
        sprawdzenie_id: check_id.to_string(),
        id: payout_id.to_string(),
    };

    sprawa_wyplaty::wskaz_sprawdzenie_wyplaty(room, &params)
        .expect("point the payout to its check");
}

/// Cancels the payout `payout_id`.
fn cancel_payout(room: &impl MetaRoom, payout_id: &str) {
    let params = AnulujWyplateParams {
        id: payout_id.to_string(),
    };

    sprawa_wyplaty::anuluj_wyplate(room, &params).expect("cancel the payout");
}

/// The broker's execution of the payout `payout_id` on 2026-11-20 at 13:00, confirmed by the
/// bank's document `potwierdzenie`.
fn execution(payout_id: &str) -> OdnotujWykonanieWyplatyParams {
    OdnotujWykonanieWyplatyParams {
        wykonal: "tozsamosc-posrednika".to_string(),
        wykonano: "2026-11-20T13:00:00Z".to_string(),
        potwierdzenie_dokument_id: "potwierdzenie".to_string(),
        id: payout_id.to_string(),
    }
}

/// The ids of the current checks.
fn current_check_ids(room: &impl MetaRoom) -> Vec<String> {
    let current_checks =
        sprawa_sprawdzenia::sprawdzenia_aktualne(room).expect("read the current checks");

    current_checks.into_iter().map(|row| row.id).collect()
}

// Accounts ----------------------------------------------------------------------------------------

/// Rachunek: numer, posiadacz, uczestnik, cel i źródło się nie zmieniają; nowy numer to nowy wiersz
#[test]
fn account_immutable() {
    let room = new_room();
    add_person(&room, &person("sprzedajaca"));
    add_account(
        &room,
        &WpiszRachunekParams {
            cel: CelRachunku::ZaplataCeny,
            ..account("rachunek", "sprzedajaca")
        },
    );

    let changes = [
        ("numer", "PL27114020040000300201355387"),
        ("posiadacz", "Ktoś Inny"),
        ("cel", "zadatek"),
        ("uczestnik_id", "sprzedajaca-2"),
    ];
    for (column, value) in changes {
        let change = format!("UPDATE sprawa_rachunki_wpisy SET {column} = :value");

        expect::refused(
            room.execute(&change, &[("value", text(value))]),
            "numer, posiadacz, uczestnik, cel i źródło rachunku się nie zmieniają",
        );
    }

    // Two days before the act a message "from the seller" brings a new number: a new row.
    add_account(
        &room,
        &WpiszRachunekParams {
            cel: CelRachunku::ZaplataCeny,
            ..account_numbered(
                "nowy-rachunek",
                "sprzedajaca",
                "PL27114020040000300201355387",
            )
        },
    );
    // A writer outside the client can leave out the moment that `zastap_rachunek` requires. The
    // table refuses the replacement without one.
    let replacement_without_moment = room.execute(
        "UPDATE sprawa_rachunki_wpisy
        SET zastapiony_przez_id = :zastapiony_przez_id,
          zastapil = :zastapil,
          zastapiono = :zastapiono
        WHERE id = :id",
        &[
            ("zastapiony_przez_id", text("nowy-rachunek")),
            ("zastapil", text("tozsamosc-agenta")),
            ("zastapiono", SqlValue::Null),
            ("id", text("rachunek")),
        ],
    );
    expect::refused(replacement_without_moment, "zastapienie_ma_autora_i_date");
    replace_account(
        &room,
        &replacement("rachunek", "nowy-rachunek", "2026-11-08T09:00:00Z"),
    );

    let current_accounts =
        sprawa_rachunki::rachunki_biezace(&room).expect("read the current accounts");
    let account_ids: Vec<&str> = current_accounts.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(account_ids, ["nowy-rachunek"]);

    expect::refused(
        sprawa_rachunki::zastap_rachunek(
            &room,
            &ZastapRachunekParams {
                zastapil: "ktos-inny".to_string(),
                ..replacement("rachunek", "nowy-rachunek", "2026-11-09T09:00:00Z")
            },
        ),
        "zastąpionego rachunku się nie przywraca",
    );
}

/// Rachunek ma jedno źródło: dokument albo wiadomość; potwierdzenie jest w komplecie
#[test]
fn account_source_and_confirmation() {
    let room = new_room();
    add_person(&room, &person("najemca"));
    add_received_document(
        &room,
        &received_document("umowa-najmu", RodzajDokumentu::Umowa),
    );

    let without_source = WpiszRachunekParams {
        zrodlo_dokument_id: None,
        ..account("bez-zrodla", "najemca")
    };
    expect::refused(
        sprawa_rachunki::wpisz_rachunek(&room, &without_source),
        "rachunek_ma_zrodlo",
    );
    sprawa_rachunki::wpisz_rachunek(
        &room,
        &WpiszRachunekParams {
            zrodlo_dokument_id: Some("umowa-najmu".to_string()),
            ..account("rachunek", "najemca")
        },
    )
    .expect("add the account from the lease");

    // A writer outside the client can leave out the way of confirmation that
    // `odnotuj_potwierdzenie_rachunku` requires. The table refuses the confirmation without one.
    let confirmation_without_way = room.execute(
        "UPDATE sprawa_rachunki_wpisy
        SET potwierdzil = :potwierdzil,
          potwierdzono = :potwierdzono,
          potwierdzenie_sposob = :potwierdzenie_sposob
        WHERE id = :id",
        &[
            ("potwierdzil", text("tozsamosc-agenta")),
            ("potwierdzono", text("2026-11-03T08:30:00Z")),
            ("potwierdzenie_sposob", SqlValue::Null),
            ("id", text("rachunek")),
        ],
    );
    expect::refused(confirmation_without_way, "potwierdzenie_w_komplecie");
    confirm_account(&room, "rachunek");
}

// Checks ------------------------------------------------------------------------------------------

/// The parameters of a check `check_id` of no row yet, from a land register copy, in the version
/// `wersja_wiersza`, without a description.
fn check_write(
    check_id: &str,
    rodzaj: RodzajSprawdzenia,
    wersja_wiersza: Option<i64>,
    wynik: WynikSprawdzenia,
) -> WpiszSprawdzenieParams {
    WpiszSprawdzenieParams {
        id: check_id.to_string(),
        rodzaj,
        uczestnik_id: None,
        nieruchomosc_id: None,
        dokument_id: None,
        umocowanie_id: None,
        rachunek_id: None,
        wersja_wiersza,
        zrodlo: "odpis".to_string(),
        zrodlo_data: None,
        wynik,
        opis: None,
        sprawdzil: "tozsamosc-kontrolera".to_string(),
        sprawdzono: "2026-11-03T09:00:00Z".to_string(),
    }
}

/// Sprawdzenie: rodzaj pasuje do wiersza, a wynik inny niż „zgodne” ma opis
#[test]
fn check_kind_matches_row() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    add_flat(&room, &flat("lokal"));
    add_account(&room, &account("rachunek", "klientka"));
    let compliant = WynikSprawdzenia::Zgodne;

    let mismatched_checks = [
        WpiszSprawdzenieParams {
            rachunek_id: Some("rachunek".to_string()),
            ..check_write("a", RodzajSprawdzenia::Tozsamosc, Some(1), compliant)
        },
        WpiszSprawdzenieParams {
            uczestnik_id: Some("klientka".to_string()),
            nieruchomosc_id: Some("lokal".to_string()),
            ..check_write("b", RodzajSprawdzenia::Tozsamosc, Some(1), compliant)
        },
        WpiszSprawdzenieParams {
            uczestnik_id: Some("klientka".to_string()),
            ..check_write("c", RodzajSprawdzenia::Tytul, Some(1), compliant)
        },
        WpiszSprawdzenieParams {
            uczestnik_id: Some("klientka".to_string()),
            ..check_write("d", RodzajSprawdzenia::Rachunek, Some(1), compliant)
        },
        WpiszSprawdzenieParams {
            rachunek_id: Some("rachunek".to_string()),
            ..check_write("e", RodzajSprawdzenia::Umocowanie, Some(1), compliant)
        },
    ];
    for mismatched_check in &mismatched_checks {
        expect::refused(
            sprawa_sprawdzenia::wpisz_sprawdzenie(&room, mismatched_check),
            "rodzaj_pasuje_do_wiersza",
        );
    }

    let account_check_with_version = WpiszSprawdzenieParams {
        rachunek_id: Some("rachunek".to_string()),
        ..check_write("f", RodzajSprawdzenia::Rachunek, Some(1), compliant)
    };
    expect::refused(
        sprawa_sprawdzenia::wpisz_sprawdzenie(&room, &account_check_with_version),
        "wersja_wiersza_przy_wierszu_z_wersja",
    );
    let remarks_without_description = WpiszSprawdzenieParams {
        uczestnik_id: Some("klientka".to_string()),
        nieruchomosc_id: Some("lokal".to_string()),
        ..check_write(
            "g",
            RodzajSprawdzenia::Tytul,
            Some(1),
            WynikSprawdzenia::Uwagi,
        )
    };
    expect::refused(
        sprawa_sprawdzenia::wpisz_sprawdzenie(&room, &remarks_without_description),
        "wynik_z_uwagami_ma_opis",
    );

    check(
        &room,
        &WpiszSprawdzenieParams {
            uczestnik_id: Some("klientka".to_string()),
            nieruchomosc_id: Some("lokal".to_string()),
            ..check_of(
                "tytul",
                RodzajSprawdzenia::Tytul,
                "2026-11-03T09:00:00Z",
                compliant,
            )
        },
    );
    check(
        &room,
        &account_check(
            "rachunku",
            "rachunek",
            "2026-11-03T09:00:00Z",
            WynikSprawdzenia::NieMoznaUstalic,
        ),
    );
}

/// Sprawdzenie powstaje aktualne, dotyczy bieżącej wersji wiersza i się nie zmienia
#[test]
fn check_current_version() {
    let room = new_room();
    add_person(&room, &person("klientka"));

    expect::refused(
        room.execute(
            "INSERT INTO sprawa_sprawdzenia_wpisy (id, rodzaj, uczestnik_id, wersja_wiersza, zrodlo,
              wynik, sprawdzil, sprawdzono, aktualne)
            VALUES ('a', 'tozsamosc', 'klientka', 1, 'dowód', 'zgodne', 'k', '2026-11-03T09:00:00Z',
              0)",
            &[],
        ),
        "sprawdzenie powstaje jako aktualne",
    );
    let check_of_old_version = WpiszSprawdzenieParams {
        uczestnik_id: Some("klientka".to_string()),
        zrodlo: "dowód osobisty okazany w biurze".to_string(),
        ..check_write(
            "stara-wersja",
            RodzajSprawdzenia::Tozsamosc,
            Some(2),
            WynikSprawdzenia::Zgodne,
        )
    };
    expect::refused(
        sprawa_sprawdzenia::wpisz_sprawdzenie(&room, &check_of_old_version),
        "sprawdzenie dotyczy bieżącej wersji wiersza",
    );

    check_identity(&room, "tozsamosc", "klientka");
    expect::refused(
        room.execute(
            "UPDATE sprawa_sprawdzenia_wpisy SET wynik = 'uwagi', opis = 'brak'",
            &[],
        ),
        "wyniku sprawdzenia się nie zmienia",
    );
    sprawa_uczestnicy::wpisz_adres_uczestnika(
        &room,
        &WpiszAdresUczestnikaParams {
            uczestnik_id: "klientka".to_string(),
            rodzaj: AdresyRodzaj::Zamieszkanie,
            kraj: "PL".to_string(),
            miejscowosc: "Wrocław".to_string(),
            ulica: None,
            numer_budynku: None,
            numer_lokalu: None,
            kod_pocztowy: None,
        },
    )
    .expect("add the client's address");
    expect::refused(
        room.execute("UPDATE sprawa_sprawdzenia_wpisy SET aktualne = 1", &[]),
        "wyniku sprawdzenia się nie zmienia",
    );
}

/// Sprawdzenie: nowe sprawdzenie tego samego wiersza odbiera aktualność poprzedniemu
#[test]
fn check_replaced_by_new_one() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    add_person(
        &room,
        &WpiszUczestnikaParams {
            imiona: Some("Ewa".to_string()),
            ..person("druga")
        },
    );
    add_account(&room, &account("rachunek", "klientka"));

    check(
        &room,
        &account_check(
            "pierwsze",
            "rachunek",
            "2026-11-03T09:00:00Z",
            WynikSprawdzenia::Zgodne,
        ),
    );
    check_identity(&room, "tozsamosc-drugiej", "druga");
    check_identity(&room, "tozsamosc-klientki", "klientka");
    check(
        &room,
        &account_check(
            "drugie",
            "rachunek",
            "2026-11-04T09:00:00Z",
            WynikSprawdzenia::Zgodne,
        ),
    );

    let current: BTreeSet<String> = current_check_ids(&room).into_iter().collect();
    let expected: BTreeSet<String> = ["drugie", "tozsamosc-drugiej", "tozsamosc-klientki"]
        .map(str::to_string)
        .into();
    assert_eq!(current, expected);
}

/// Sprawdzenie: zmiana danych identyfikujących uczestnika albo umocowania odbiera aktualność (D6)
#[test]
fn check_lost_on_data_change() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    add_flat(&room, &flat("lokal"));
    add_power_for_office(
        &room,
        "pelnomocnictwo",
        "klientka",
        &["wystapienie_do_wspolnoty"],
        "2026-11-03",
    );
    check_identity(&room, "tozsamosc", "klientka");
    check(
        &room,
        &WpiszSprawdzenieParams {
            uczestnik_id: Some("klientka".to_string()),
            nieruchomosc_id: Some("lokal".to_string()),
            ..check_of(
                "tytul",
                RodzajSprawdzenia::Tytul,
                "2026-11-03T09:00:00Z",
                WynikSprawdzenia::Zgodne,
            )
        },
    );
    check(
        &room,
        &WpiszSprawdzenieParams {
            umocowanie_id: Some("pelnomocnictwo".to_string()),
            ..check_of(
                "umocowania",
                RodzajSprawdzenia::Umocowanie,
                "2026-11-03T09:00:00Z",
                WynikSprawdzenia::Zgodne,
            )
        },
    );

    // Contact data do not identify the person, so the checks stay current.
    sprawa_uczestnicy::zmien_kontakt_uczestnika(
        &room,
        &ZmienKontaktUczestnikaParams {
            telefon: None,
            email: Some("anna@example.pl".to_string()),
            kanal_kontaktu: Some(KanalKontaktu::Email),
            id: "klientka".to_string(),
        },
    )
    .expect("change the client's contact");
    assert_eq!(current_check_ids(&room).len(), 3);

    sprawa_uczestnicy::wpisz_obywatelstwo_uczestnika(
        &room,
        &WpiszObywatelstwoUczestnikaParams {
            uczestnik_id: "klientka".to_string(),
            panstwo: "PL".to_string(),
        },
    )
    .expect("add the client's citizenship");
    assert_eq!(current_check_ids(&room), ["umocowania"]);
    assert_eq!(version_of(&room, "klientka"), 2);

    sprawa_umocowania::dopisz_czynnosc_umocowania(
        &room,
        &DopiszCzynnoscUmocowaniaParams {
            umocowanie_id: "pelnomocnictwo".to_string(),
            czynnosc: "odbior_odpisu".to_string(),
        },
    )
    .expect("add an act of the power");
    assert!(current_check_ids(&room).is_empty());

    check(
        &room,
        &WpiszSprawdzenieParams {
            uczestnik_id: Some("klientka".to_string()),
            ..check_of(
                "tozsamosc-2",
                RodzajSprawdzenia::Tozsamosc,
                "2026-11-05T09:00:00Z",
                WynikSprawdzenia::Zgodne,
            )
        },
    );
    assert_eq!(current_check_ids(&room), ["tozsamosc-2"]);
}

/// Zgoda na rachunek wymaga aktualnego sprawdzenia „zgodne”, potwierdzenia i niezastąpionego rachunku
#[test]
fn account_admission_needs_check() {
    let room = new_room();
    add_person(&room, &person("najemca"));
    let refusal = "zgoda na rachunek wymaga aktualnego sprawdzenia";

    let admit = |decision_id: &str, account_id: &str| {
        let request = decision_request_about(
            decision_id,
            RodzajDecyzji::DopuszczenieRachunku,
            "sprawa_rachunki_wpisy",
            account_id,
        );
        ask_for_decision(&room, &request);

        let consent = settlement(decision_id, Rozstrzygniecie::Zgoda, "2026-11-03T12:00:00Z");
        expect::refused(
            sprawa_decyzje::rozstrzygnij_decyzje(&room, &consent),
            refusal,
        );
    };

    // Not confirmed by another channel.
    add_account(&room, &account("niepotwierdzony", "najemca"));
    check(
        &room,
        &account_check(
            "sprawdzenie-niepotwierdzonego",
            "niepotwierdzony",
            "2026-11-03T09:00:00Z",
            WynikSprawdzenia::Zgodne,
        ),
    );
    admit("zgoda-niepotwierdzony", "niepotwierdzony");

    // Not checked, or checked with remarks.
    add_account(&room, &account_numbered("niesprawdzony", "najemca", "PL02"));
    confirm_account(&room, "niesprawdzony");
    admit("zgoda-niesprawdzony", "niesprawdzony");
    check(
        &room,
        &account_check(
            "z-uwagami",
            "niesprawdzony",
            "2026-11-03T09:00:00Z",
            WynikSprawdzenia::Uwagi,
        ),
    );
    admit("zgoda-z-uwagami", "niesprawdzony");

    // Replaced by a new number.
    admitted_account(&room, "stary", "najemca");
    add_account(&room, &account_numbered("nowy", "najemca", "PL03"));
    replace_account(&room, &replacement("stary", "nowy", "2026-11-04T09:00:00Z"));
    admit("zgoda-zastapiony", "stary");

    // A refusal is always allowed; a confirmed, checked, current account gets the consent.
    settle_decision(
        &room,
        &RozstrzygnijDecyzjeParams {
            uzasadnienie: Some("numer zmieniony dwa dni przed aktem".to_string()),
            ..settlement(
                "zgoda-zastapiony",
                Rozstrzygniecie::Odmowa,
                "2026-11-03T12:00:00Z",
            )
        },
    );
    admitted_account(&room, "dobry", "najemca");
}

// Payouts -----------------------------------------------------------------------------------------

/// Wypłata powstaje przygotowana na rachunek odbiorcy dopuszczony decyzją i niezastąpiony
#[test]
fn payout_to_admitted_account() {
    let room = new_room();
    add_person(&room, &person("najemca"));
    add_person(
        &room,
        &WpiszUczestnikaParams {
            imiona: Some("Ewa".to_string()),
            ..person("wlascicielka")
        },
    );
    let refusal = "wypłata idzie na rachunek odbiorcy dopuszczony decyzją i niezastąpiony";

    add_account(&room, &account("niedopuszczony", "najemca"));
    expect::refused(
        sprawa_wyplaty::przygotuj_wyplate(&room, &payout("w-1", "najemca", "niedopuszczony")),
        refusal,
    );

    admitted_account(&room, "rachunek-najemcy", "najemca");
    expect::refused(
        sprawa_wyplaty::przygotuj_wyplate(
            &room,
            &payout("w-2", "wlascicielka", "rachunek-najemcy"),
        ),
        refusal,
    );

    expect::refused(
        room.execute(
            "INSERT INTO sprawa_wyplaty_wpisy (id, podstawa_tabela, podstawa_id, kwota,
              wyliczenie_sciezka, wyliczenie_skrot, odbiorca_uczestnik_id, rachunek_id, utworzono,
              stan, sprawdzenie_id)
            VALUES ('w-3', 'najem_kaucje', 'r', 100, 'w.json', 'sha256-w', 'najemca',
              'rachunek-najemcy', '2026-11-20T09:00:00Z', 'przygotowana',
              'sprawdzenie-rachunek-najemcy')",
            &[],
        ),
        "wypłata powstaje jako przygotowana, bez sprawdzenia i zgody",
    );

    prepare_payout(
        &room,
        &payout("zwrot-kaucji", "najemca", "rachunek-najemcy"),
    );

    add_account(&room, &account_numbered("nowy", "najemca", "PL03"));
    replace_account(
        &room,
        &replacement("rachunek-najemcy", "nowy", "2026-11-20T09:30:00Z"),
    );
    expect::refused(
        sprawa_wyplaty::przygotuj_wyplate(&room, &payout("w-4", "najemca", "rachunek-najemcy")),
        refusal,
    );
    expect::refused(
        room.execute(
            "UPDATE sprawa_wyplaty_wpisy SET rachunek_id = 'nowy' WHERE id = 'zwrot-kaucji'",
            &[],
        ),
        "treść wypłaty się nie zmienia",
    );
}

/// Wypłata: Kontroler sprawdza rachunek na nowo po przygotowaniu wypłaty (D13)
#[test]
fn payout_check_after_preparation() {
    let room = new_room();
    add_person(&room, &person("najemca"));
    admitted_account(&room, "rachunek-najemcy", "najemca");
    add_account(&room, &account_numbered("inny", "najemca", "PL04"));
    check(
        &room,
        &account_check(
            "sprawdzenie-innego",
            "inny",
            "2026-11-20T10:00:00Z",
            WynikSprawdzenia::Zgodne,
        ),
    );
    prepare_payout(
        &room,
        &PrzygotujWyplateParams {
            utworzono: "2026-11-20T09:00:00Z".to_string(),
            ..payout("zwrot-kaucji", "najemca", "rachunek-najemcy")
        },
    );
    let refusal =
        "wypłata wskazuje aktualne sprawdzenie „zgodne” jej rachunku zrobione po przygotowaniu";
    let check_of_payout = |check_id: &str| WskazSprawdzenieWyplatyParams {
        sprawdzenie_id: check_id.to_string(),
        id: "zwrot-kaucji".to_string(),
    };

    // The check made when the account was admitted is older than the payout.
    expect::refused(
        sprawa_wyplaty::wskaz_sprawdzenie_wyplaty(
            &room,
            &check_of_payout("sprawdzenie-rachunek-najemcy"),
        ),
        refusal,
    );
    expect::refused(
        sprawa_wyplaty::wskaz_sprawdzenie_wyplaty(&room, &check_of_payout("sprawdzenie-innego")),
        refusal,
    );
    check(
        &room,
        &account_check(
            "z-uwagami",
            "rachunek-najemcy",
            "2026-11-20T10:00:00Z",
            WynikSprawdzenia::Uwagi,
        ),
    );
    expect::refused(
        sprawa_wyplaty::wskaz_sprawdzenie_wyplaty(&room, &check_of_payout("z-uwagami")),
        refusal,
    );

    check(
        &room,
        &account_check(
            "swieze",
            "rachunek-najemcy",
            "2026-11-20T10:30:00Z",
            WynikSprawdzenia::Zgodne,
        ),
    );
    point_payout_to_check(&room, "zwrot-kaucji", "swieze");
}

/// Wypłata: zgoda człowieka zapada po sprawdzeniu rachunku i wskazuje tę wypłatę
#[test]
fn payout_consent_after_check() {
    let room = new_room();
    add_person(&room, &person("najemca"));
    admitted_account(&room, "rachunek-najemcy", "najemca");
    prepare_payout(
        &room,
        &payout("zwrot-kaucji", "najemca", "rachunek-najemcy"),
    );
    prepare_payout(
        &room,
        &payout("inna-wyplata", "najemca", "rachunek-najemcy"),
    );
    let refusal = "zgoda na wypłatę zapada po sprawdzeniu rachunku, które wypłata wskazuje";
    let early_consent = settlement(
        "za-wczesnie",
        Rozstrzygniecie::Zgoda,
        "2026-11-20T09:45:00Z",
    );

    ask_for_decision(
        &room,
        &PoprosODecyzjeParams {
            zlozono: "2026-11-20T09:30:00Z".to_string(),
            ..decision_request_about(
                "za-wczesnie",
                RodzajDecyzji::Wyplata,
                "sprawa_wyplaty_wpisy",
                "zwrot-kaucji",
            )
        },
    );
    expect::refused(
        sprawa_decyzje::rozstrzygnij_decyzje(&room, &early_consent),
        refusal,
    );

    check(
        &room,
        &account_check(
            "swieze",
            "rachunek-najemcy",
            "2026-11-20T10:00:00Z",
            WynikSprawdzenia::Zgodne,
        ),
    );
    point_payout_to_check(&room, "zwrot-kaucji", "swieze");
    expect::refused(
        sprawa_decyzje::rozstrzygnij_decyzje(&room, &early_consent),
        refusal,
    );
    approve(&room, "za-wczesnie", "2026-11-20T11:00:00Z");

    let other_decision = PoprosODecyzjeParams {
        zlozono: "2026-11-20T11:00:00Z".to_string(),
        ..decision_request_about(
            "na-inna",
            RodzajDecyzji::Inna,
            "sprawa_wyplaty_wpisy",
            "inna-wyplata",
        )
    };
    approved_decision(&room, &other_decision, "2026-11-20T11:30:00Z");
    expect::refused(
        sprawa_wyplaty::wskaz_zgode_na_wyplate(
            &room,
            &WskazZgodeNaWyplateParams {
                decyzja_id: "na-inna".to_string(),
                id: "zwrot-kaucji".to_string(),
            },
        ),
        "wypłata wskazuje zatwierdzoną decyzję wyplata, która ją wskazuje",
    );
    sprawa_wyplaty::wskaz_zgode_na_wyplate(
        &room,
        &WskazZgodeNaWyplateParams {
            decyzja_id: "za-wczesnie".to_string(),
            id: "zwrot-kaucji".to_string(),
        },
    )
    .expect("point the payout to its consent");
}

/// Wypłata: wykonanie wymaga zgody, potwierdzenia z banku i wciąż aktualnego rachunku
#[test]
fn payout_execution() {
    let room = new_room();
    add_person(&room, &person("najemca"));
    admitted_account(&room, "rachunek-najemcy", "najemca");
    add_received_document(
        &room,
        &received_document("potwierdzenie", RodzajDokumentu::PotwierdzenieBankowe),
    );
    let refusal = "wykonanie wymaga zgody na wypłatę po aktualnym sprawdzeniu niezastąpionego i dopuszczonego rachunku";

    prepare_payout(&room, &payout("bez-zgody", "najemca", "rachunek-najemcy"));
    expect::refused(
        sprawa_wyplaty::odnotuj_wykonanie_wyplaty(&room, &execution("bez-zgody")),
        refusal,
    );
    cancel_payout(&room, "bez-zgody");

    checked_and_approved_payout(&room, "zwrot-kaucji", "najemca", "rachunek-najemcy");
    // A writer outside the client can leave out the bank's confirmation that
    // `odnotuj_wykonanie_wyplaty` requires. The table refuses the execution without one.
    let execution_without_confirmation = room.execute(
        "UPDATE sprawa_wyplaty_wpisy
        SET stan = 'wykonana',
          wykonal = :wykonal,
          wykonano = :wykonano,
          potwierdzenie_dokument_id = :potwierdzenie_dokument_id
        WHERE id = :id",
        &[
            ("wykonal", text("tozsamosc-posrednika")),
            ("wykonano", text("2026-11-20T13:00:00Z")),
            ("potwierdzenie_dokument_id", SqlValue::Null),
            ("id", text("zwrot-kaucji")),
        ],
    );
    expect::refused(
        execution_without_confirmation,
        "wykonana_ma_wykonawce_i_potwierdzenie",
    );

    // A newer check of the account, e.g. with remarks, takes the consent's ground away.
    check(
        &room,
        &account_check(
            "nowsze",
            "rachunek-najemcy",
            "2026-11-20T12:30:00Z",
            WynikSprawdzenia::Uwagi,
        ),
    );
    expect::refused(
        sprawa_wyplaty::odnotuj_wykonanie_wyplaty(&room, &execution("zwrot-kaucji")),
        refusal,
    );

    check(
        &room,
        &account_check(
            "jeszcze-nowsze",
            "rachunek-najemcy",
            "2026-11-20T12:40:00Z",
            WynikSprawdzenia::Zgodne,
        ),
    );
    expect::refused(
        sprawa_wyplaty::odnotuj_wykonanie_wyplaty(&room, &execution("zwrot-kaucji")),
        refusal,
    );

    prepare_payout(&room, &payout("druga", "najemca", "rachunek-najemcy"));
    checked_and_approved_payout(&room, "trzecia", "najemca", "rachunek-najemcy");
    sprawa_wyplaty::odnotuj_wykonanie_wyplaty(&room, &execution("trzecia"))
        .expect("record the execution of the payout");
    expect::refused(
        sprawa_wyplaty::anuluj_wyplate(
            &room,
            &AnulujWyplateParams {
                id: "trzecia".to_string(),
            },
        ),
        "wykonanej ani anulowanej wypłaty się nie zmienia",
    );
    cancel_payout(&room, "druga");
    expect::refused(
        sprawa_wyplaty::odnotuj_wykonanie_wyplaty(&room, &execution("druga")),
        "wykonanej ani anulowanej wypłaty się nie zmienia",
    );
}

/// Wypłata: wykonanie po zastąpieniu rachunku jest odmówione
#[test]
fn payout_execution_after_account_replaced() {
    let room = new_room();
    add_person(&room, &person("najemca"));
    admitted_account(&room, "rachunek-najemcy", "najemca");
    checked_and_approved_payout(&room, "zwrot-kaucji", "najemca", "rachunek-najemcy");
    add_received_document(
        &room,
        &received_document("potwierdzenie", RodzajDokumentu::PotwierdzenieBankowe),
    );
    add_account(&room, &account_numbered("nowy", "najemca", "PL05"));
    replace_account(
        &room,
        &replacement("rachunek-najemcy", "nowy", "2026-11-20T12:30:00Z"),
    );

    expect::refused(
        sprawa_wyplaty::odnotuj_wykonanie_wyplaty(&room, &execution("zwrot-kaucji")),
        "wykonanie wymaga zgody na wypłatę po aktualnym sprawdzeniu niezastąpionego i dopuszczonego rachunku",
    );
    cancel_payout(&room, "zwrot-kaucji");
}

/// Teczki nie zamyka się, dopóki jakaś wypłata jest przygotowana (D19)
#[test]
fn closing_after_payouts() {
    let room = new_room();
    open_case(&room, "najem");
    add_person(&room, &person("najemca"));
    admitted_account(&room, "rachunek-najemcy", "najemca");
    prepare_payout(
        &room,
        &payout("zwrot-kaucji", "najemca", "rachunek-najemcy"),
    );
    let closing = ZmienEtapSprawyParams {
        etap: "zamknieta".to_string(),
        od: "2026-11-30T09:00:00Z".to_string(),
        zmienil: "agent".to_string(),
        decyzja_id: None,
    };

    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &closing),
        "teczki nie zamyka się, dopóki jakaś wypłata jest przygotowana",
    );

    cancel_payout(&room, "zwrot-kaucji");
    sprawa_metryka::zmien_etap_sprawy(&room, &closing).expect("close the case");
}

/// Dopóki stoi znacznik wątpliwości, nie ma zgody na termin, rachunek ani wypłatę
#[test]
fn doubt_blocks_consents() {
    let room = new_room();
    open_case(&room, "najem");
    add_person(&room, &person("najemca"));
    admitted_account(&room, "rachunek-najemcy", "najemca");
    prepare_payout(
        &room,
        &payout("zwrot-kaucji", "najemca", "rachunek-najemcy"),
    );
    check(
        &room,
        &account_check(
            "swieze",
            "rachunek-najemcy",
            "2026-11-20T10:00:00Z",
            WynikSprawdzenia::Zgodne,
        ),
    );
    point_payout_to_check(&room, "zwrot-kaucji", "swieze");
    add_account(&room, &account_numbered("drugi", "najemca", "PL06"));
    confirm_account(&room, "drugi");
    check(
        &room,
        &account_check(
            "sprawdzenie-drugiego",
            "drugi",
            "2026-11-20T10:00:00Z",
            WynikSprawdzenia::Zgodne,
        ),
    );
    sprawa_terminy::wpisz_termin(
        &room,
        &WpiszTerminParams {
            id: "termin-aktu".to_string(),
            rodzaj: RodzajTerminu::ZUmowy,
            co: "Akt notarialny".to_string(),
            zdarzenie_tabela: None,
            zdarzenie_id: None,
            zdarzenie_chwila: None,
            koniec: "2026-12-01T10:00:00Z".to_string(),
            podstawa: Some("§ 2 umowy przedwstępnej".to_string()),
            pilnuje_stanowisko: PilnujeStanowisko::KoordynatorTransakcji,
            kalendarz_odnosnik: None,
        },
    )
    .expect("add the deadline of the act");
    sprawa_metryka::postaw_watpliwosc(
        &room,
        &PostawWatpliwoscParams {
            id: "watpliwosc".to_string(),
            postawil: "tozsamosc-kontrolera".to_string(),
            postawiono: "2026-11-20T10:30:00Z".to_string(),
        },
    )
    .expect("raise the doubt");

    let requests = [
        (
            "termin",
            RodzajDecyzji::PotwierdzenieTerminu,
            "sprawa_terminy_wpisy",
            "termin-aktu",
        ),
        (
            "rachunek",
            RodzajDecyzji::DopuszczenieRachunku,
            "sprawa_rachunki_wpisy",
            "drugi",
        ),
        (
            "wyplata",
            RodzajDecyzji::Wyplata,
            "sprawa_wyplaty_wpisy",
            "zwrot-kaucji",
        ),
    ];
    for (decision_id, rodzaj, tabela, wiersz) in requests {
        let request = PoprosODecyzjeParams {
            zlozono: "2026-11-20T11:00:00Z".to_string(),
            ..decision_request_about(decision_id, rodzaj, tabela, wiersz)
        };
        ask_for_decision(&room, &request);

        let consent = settlement(decision_id, Rozstrzygniecie::Zgoda, "2026-11-20T12:00:00Z");
        expect::refused(
            sprawa_decyzje::rozstrzygnij_decyzje(&room, &consent),
            "dopóki stoi znacznik wątpliwości, nie ma zgody na termin, rachunek ani wypłatę",
        );
    }

    // A refusal is always allowed while the doubt stands.
    ask_for_decision(
        &room,
        &decision_request_about(
            "odmowa",
            RodzajDecyzji::PotwierdzenieTerminu,
            "sprawa_terminy_wpisy",
            "termin-aktu",
        ),
    );
    settle_decision(
        &room,
        &settlement("odmowa", Rozstrzygniecie::Odmowa, "2026-11-03T12:00:00Z"),
    );

    ask_for_decision(
        &room,
        &PoprosODecyzjeParams {
            zlozono: "2026-11-20T11:00:00Z".to_string(),
            ..decision_request_about(
                "zdjecie",
                RodzajDecyzji::ZdjecieWatpliwosci,
                "sprawa_metryka_watpliwosci",
                "watpliwosc",
            )
        },
    );
    approve(&room, "zdjecie", "2026-11-20T11:30:00Z");
    sprawa_metryka::zdejmij_watpliwosc(
        &room,
        &ZdejmijWatpliwoscParams {
            zdjecie_decyzja_id: "zdjecie".to_string(),
            id: "watpliwosc".to_string(),
        },
    )
    .expect("lift the doubt");
    for (decision_id, _rodzaj, _tabela, _wiersz) in requests {
        approve(&room, decision_id, "2026-11-20T12:00:00Z");
    }
}

/// Rachunek: potwierdzenie wpisuje się raz, a zastępca ma tego samego uczestnika i cel
#[test]
fn account_confirmation_once_and_replacement_of_same_kind() {
    let room = new_room();
    add_person(&room, &person("najemca"));
    add_person(
        &room,
        &WpiszUczestnikaParams {
            imiona: Some("Piotr".to_string()),
            ..person("firma-najemcy")
        },
    );
    admitted_account(&room, "kaucja", "najemca");

    let assignments = [
        "potwierdzil = NULL, potwierdzono = NULL, potwierdzenie_sposob = NULL",
        "potwierdzenie_sposob = 'mailem z adresu z wiadomości z numerem'",
    ];
    for assignment in assignments {
        let change = format!("UPDATE sprawa_rachunki_wpisy SET {assignment} WHERE id = 'kaucja'");

        expect::refused(
            room.execute(&change, &[]),
            "potwierdzenie posiadacza rachunku wpisuje się raz",
        );
    }

    add_account(
        &room,
        &WpiszRachunekParams {
            cel: CelRachunku::Czynsz,
            ..account_numbered("czynsz-firmy", "firma-najemcy", "PL09")
        },
    );
    add_account(
        &room,
        &WpiszRachunekParams {
            cel: CelRachunku::Czynsz,
            ..account_numbered("czynsz-najemcy", "najemca", "PL10")
        },
    );
    add_account(&room, &account_numbered("nowa-kaucja", "najemca", "PL11"));
    for other_account in ["czynsz-firmy", "czynsz-najemcy"] {
        expect::refused(
            sprawa_rachunki::zastap_rachunek(
                &room,
                &replacement("kaucja", other_account, "2026-11-08T09:00:00Z"),
            ),
            "rachunek zastępujący ma tego samego uczestnika i ten sam cel",
        );
    }
    expect::refused(
        room.execute(
            "INSERT INTO sprawa_rachunki_wpisy (id, uczestnik_id, numer, posiadacz, cel,
              zrodlo_dokument_id, wpisano, zastapiony_przez_id, zastapil, zastapiono)
            VALUES ('stary', 'najemca', 'PL12', 'Jan Nowak', 'zwrot_kaucji', 'zrodlo-kaucja',
              '2026-11-03T08:00:00Z', 'czynsz-firmy', 'agent', '2026-11-08T09:00:00Z')",
            &[],
        ),
        "rachunek zastępujący ma tego samego uczestnika i ten sam cel",
    );
    replace_account(
        &room,
        &replacement("kaucja", "nowa-kaucja", "2026-11-08T09:00:00Z"),
    );
}

/// Wypłata: treść nie zmienia się od wpisu, a sprawdzenie i zgodę wskazuje się raz
#[test]
fn payout_content_frozen() {
    let room = new_room();
    add_person(&room, &person("najemca"));
    admitted_account(&room, "rachunek-najemcy", "najemca");
    checked_and_approved_payout(&room, "zwrot-kaucji", "najemca", "rachunek-najemcy");
    let refusal = "treść wypłaty się nie zmienia";

    // The workarounds from the audit: a new amount after the consent, and an earlier creation
    // moment that would let an old check of the account count as fresh.
    expect::refused(
        room.execute(
            "UPDATE sprawa_wyplaty_wpisy
            SET kwota = 999999, wyliczenie_skrot = 'inny', podstawa_id = '9'
            WHERE id = 'zwrot-kaucji'",
            &[],
        ),
        refusal,
    );
    expect::refused(
        room.execute(
            "UPDATE sprawa_wyplaty_wpisy
            SET utworzono = '2026-11-01T00:00:00Z'
            WHERE id = 'zwrot-kaucji'",
            &[],
        ),
        refusal,
    );

    prepare_payout(&room, &payout("druga", "najemca", "rachunek-najemcy"));
    expect::refused(
        room.execute(
            "UPDATE sprawa_wyplaty_wpisy SET kwota = 1 WHERE id = 'druga'",
            &[],
        ),
        refusal,
    );

    let once = "wypłata wskazuje sprawdzenie i zgodę raz";
    expect::refused(
        room.execute(
            "UPDATE sprawa_wyplaty_wpisy SET sprawdzenie_id = NULL WHERE id = 'zwrot-kaucji'",
            &[],
        ),
        once,
    );
    expect::refused(
        room.execute(
            "UPDATE sprawa_wyplaty_wpisy SET decyzja_id = NULL WHERE id = 'zwrot-kaucji'",
            &[],
        ),
        once,
    );

    // Another amount is a cancellation and a new payout.
    cancel_payout(&room, "druga");
    prepare_payout(
        &room,
        &payout("druga-poprawiona", "najemca", "rachunek-najemcy"),
    );
}

/// Zastąpiona zgoda na rachunek albo na wypłatę nie wystarcza do wypłaty
#[test]
fn superseded_consent_does_not_count() {
    let room = new_room();
    add_person(&room, &person("najemca"));
    admitted_account(&room, "rachunek-najemcy", "najemca");
    checked_and_approved_payout(&room, "zwrot-kaucji", "najemca", "rachunek-najemcy");
    add_received_document(
        &room,
        &received_document("potwierdzenie", RodzajDokumentu::PotwierdzenieBankowe),
    );

    // The workaround from the audit: a later refusal of the same account replaces its admission.
    ask_for_decision(
        &room,
        &PoprosODecyzjeParams {
            zlozono: "2026-11-20T12:10:00Z".to_string(),
            zastepuje_decyzja_id: Some("dopuszczenie-rachunek-najemcy".to_string()),
            ..decision_request_about(
                "cofniecie-dopuszczenia",
                RodzajDecyzji::DopuszczenieRachunku,
                "sprawa_rachunki_wpisy",
                "rachunek-najemcy",
            )
        },
    );
    settle_decision(
        &room,
        &settlement(
            "cofniecie-dopuszczenia",
            Rozstrzygniecie::Odmowa,
            "2026-11-20T12:20:00Z",
        ),
    );
    let admission_state = room
        .value(
            "SELECT stan FROM sprawa_decyzje_wpisy WHERE id = 'dopuszczenie-rachunek-najemcy'",
            &[],
        )
        .expect("read the state of the admission");
    let admission_state = String::from_sql_value(&admission_state).expect("the state is text");
    assert_eq!(admission_state, "zastapiona");

    expect::refused(
        sprawa_wyplaty::przygotuj_wyplate(
            &room,
            &PrzygotujWyplateParams {
                utworzono: "2026-11-20T12:30:00Z".to_string(),
                ..payout("nowa", "najemca", "rachunek-najemcy")
            },
        ),
        "wypłata idzie na rachunek odbiorcy dopuszczony decyzją i niezastąpiony",
    );
    expect::refused(
        sprawa_wyplaty::odnotuj_wykonanie_wyplaty(&room, &execution("zwrot-kaucji")),
        "wykonanie wymaga zgody na wypłatę po aktualnym sprawdzeniu niezastąpionego i \
         dopuszczonego rachunku",
    );
    cancel_payout(&room, "zwrot-kaucji");

    // A refusal that replaces the consent to a payout stops its execution too.
    admitted_account(&room, "drugi-rachunek", "najemca");
    checked_and_approved_payout(&room, "druga", "najemca", "drugi-rachunek");
    ask_for_decision(
        &room,
        &PoprosODecyzjeParams {
            zlozono: "2026-11-20T12:10:00Z".to_string(),
            zastepuje_decyzja_id: Some("zgoda-druga".to_string()),
            ..decision_request_about(
                "cofniecie-zgody",
                RodzajDecyzji::Wyplata,
                "sprawa_wyplaty_wpisy",
                "druga",
            )
        },
    );
    settle_decision(
        &room,
        &settlement(
            "cofniecie-zgody",
            Rozstrzygniecie::Odmowa,
            "2026-11-20T12:20:00Z",
        ),
    );
    expect::refused(
        sprawa_wyplaty::odnotuj_wykonanie_wyplaty(&room, &execution("druga")),
        "wykonanie wymaga zgody na wypłatę",
    );
}
