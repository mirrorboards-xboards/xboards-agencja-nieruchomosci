//! Scenarios of the Reception's memory room (spec/positions/biuro/recepcja).
//!
//! The room has the Reception's own kinds and three kinds of the agent memory (findings, sources
//! and orders); the rules of those are checked in stanowisko.rs. Each scenario starts from an empty
//! room and checks one rule the database keeps: a write the rule allows and a write it refuses. The
//! first scenario follows a day of the Reception: a letter assigned to a case by its mark, a portal
//! question assigned to an offer by the offer catalog, a new request sent on a card and resolved, a
//! suspicious sender and the daily report.

use xboards_agencja_nieruchomosci::positions::biuro::recepcja::{
    self, recepcja_karty, recepcja_sprawy, recepcja_wplywy,
};
use xboards_agencja_nieruchomosci::positions::stanowisko::stanowisko_polecenia::{
    PrzyjmijPolecenieParams, WpisyWydalJako,
};
use xboards_agencja_nieruchomosci::positions::stanowisko::stanowisko_zrodla::{
    DopiszZdarzenieZrodlaParams, WpisyRodzaj as RodzajZrodla, WpiszZrodloParams, ZdarzeniaRodzaj,
};
use xboards_agencja_nieruchomosci::xmeta_room::{FromSqlValue, MemoryRoom, MetaRoom, SqlValue};

use recepcja::stanowisko_zrodla::ZdarzeniaRodzaj as RodzajZdarzeniaWPokoju;
use recepcja_karty::{
    MeldunekDziennyParams, MeldunekDziennyRow, OdnotujRozstrzygniecieKartyParams, WpisyDzial,
    WpisyKanal, WpisyRodzaj as RodzajKarty, WpisyRodzajSprawy, WpisyRozstrzygniecie,
    WyslijKarteParams,
};
use recepcja_sprawy::{
    DodajZnakSprawyParams, SprawyPoZnakuParams, WpisyRodzaj as RodzajSprawy, WpiszSpraweParams,
    WycofajZnakSprawyParams, ZamknijSpraweParams, ZmienSpraweParams, ZnakiRodzaj,
    ZnakiSprawyParams,
};
use recepcja_wplywy::{
    DziennikDniaParams, DziennikKanal, DziennikOddano, OdnotujWplywParams, WplywyOdNadawcyParams,
    WplywySprawyParams, WplywyWedlugKanalowParams, WplywyWedlugKanalowRow,
};

use crate::expect;
use crate::stanowisko::{
    DELETION, REPLACEMENT, accept_order, add_source, add_source_event, first_row_copy, order,
    source, source_event,
};

fn new_room() -> MemoryRoom {
    recepcja::open_in_memory().expect("open the room of the Reception")
}

/// A text value to bind in raw SQL.
fn text(value: &str) -> SqlValue {
    SqlValue::Text(value.to_string())
}

// Building blocks ---------------------------------------------------------------------------------

/// The default parameters of an offer's case the head of brokerage handed over.
fn case() -> WpiszSpraweParams {
    WpiszSpraweParams {
        teczka_odnosnik: "pokoj-teczki-1".to_string(),
        rodzaj: RodzajSprawy::Oferta,
        oznaczenie: "Oferta 2026/031, ul. Polna 7 m. 12".to_string(),
        agent: "tozsamosc-opiekuna-oferty".to_string(),
        otwarto: "2026-11-02T09:00:00Z".to_string(),
        kancelaria_xnotariusz: None,
        przekazal: "tozsamosc-kierownika-posrednictwa".to_string(),
        przekazano: "2026-11-02T09:05:00Z".to_string(),
    }
}

fn add_case(room: &impl MetaRoom, params: &WpiszSpraweParams) {
    recepcja_sprawy::wpisz_sprawe(room, params).expect("add the case");
}

/// The parameters of a case with a short designation, for the writes a rule refuses.
fn bare_case() -> WpiszSpraweParams {
    WpiszSpraweParams {
        oznaczenie: "Oferta".to_string(),
        agent: "a".to_string(),
        przekazal: "k".to_string(),
        ..case()
    }
}

/// The default parameters of the address of the property as a mark of a case.
fn mark(sprawa_id: i64) -> DodajZnakSprawyParams {
    DodajZnakSprawyParams {
        sprawa_id,
        rodzaj: ZnakiRodzaj::AdresNieruchomosci,
        wartosc: "ul. Polna 7 m. 12, Wrocław".to_string(),
        przekazal: "tozsamosc-kierownika-posrednictwa".to_string(),
        dodano: "2026-11-02T09:05:00Z".to_string(),
    }
}

fn add_mark(room: &impl MetaRoom, params: &DodajZnakSprawyParams) {
    recepcja_sprawy::dodaj_znak_sprawy(room, params).expect("add the mark");
}

fn withdraw_mark(room: &MemoryRoom, mark_id: i64, wycofano: &str) {
    let params = WycofajZnakSprawyParams {
        wycofano: wycofano.to_string(),
        id: mark_id,
    };

    recepcja_sprawy::wycofaj_znak_sprawy(room, &params).expect("withdraw the mark");
}

/// The default parameters of a letter of a housing community, handed over nowhere.
fn message() -> OdnotujWplywParams {
    OdnotujWplywParams {
        wplynelo: "2026-11-04T09:12".to_string(),
        kanal: DziennikKanal::Email,
        nadawca: "administracja@polna7.example".to_string(),
        rodzaj_pisma: "pismo wspólnoty".to_string(),
        wiadomosc_odnosnik: "skrzynka/1".to_string(),
        sprawa_id: None,
        znak_id: None,
        katalog_odnosnik: None,
        oddano: DziennikOddano::Nigdzie,
        oddano_komu: None,
        oddano_o: None,
        kanal_nie_pasuje: false,
    }
}

fn log_message(room: &impl MetaRoom, params: &OdnotujWplywParams) {
    recepcja_wplywy::odnotuj_wplyw(room, params).expect("log the message");
}

/// The default parameters of the letter, assigned to the first case by its first mark and handed
/// to the case's agent.
fn message_by_mark() -> OdnotujWplywParams {
    OdnotujWplywParams {
        sprawa_id: Some(1),
        znak_id: Some(1),
        oddano: DziennikOddano::AgentSprawy,
        oddano_komu: Some("tozsamosc-opiekuna-oferty".to_string()),
        oddano_o: Some("2026-11-04T09:20".to_string()),
        ..message()
    }
}

/// The default parameters of a new request from the website's form, handed over on a card.
fn message_to_card() -> OdnotujWplywParams {
    OdnotujWplywParams {
        kanal: DziennikKanal::Formularz,
        nadawca: "formularz-strony".to_string(),
        rodzaj_pisma: "nowe zgłoszenie".to_string(),
        oddano: DziennikOddano::Karta,
        oddano_o: Some("2026-11-04T09:30".to_string()),
        ..message()
    }
}

/// The parameters of a message with a short sender and kind, for the writes a rule refuses.
fn bare_message() -> OdnotujWplywParams {
    OdnotujWplywParams {
        nadawca: "n".to_string(),
        rodzaj_pisma: "pismo".to_string(),
        wiadomosc_odnosnik: "skrzynka/x".to_string(),
        ..message()
    }
}

/// The default parameters of a card of a new request to sell a flat, sent from a message.
fn card(wplyw_id: i64) -> WyslijKarteParams {
    WyslijKarteParams {
        rodzaj: RodzajKarty::NoweZgloszenie,
        wplyw_id,
        wyslano: "2026-11-04T09:30:00Z".to_string(),
        rodzaj_sprawy: Some(WpisyRodzajSprawy::Sprzedaz),
        rodzaj_nieruchomosci: Some("mieszkanie".to_string()),
        miejscowosc: Some("Wrocław".to_string()),
        kanal: Some(WpisyKanal::Telefon),
        zglaszajacy: Some("Anna Nowak".to_string()),
        kontakt: Some("+48 600 000 000".to_string()),
        wiadomosc_odnosnik: "skrzynka/2".to_string(),
    }
}

fn send_card(room: &impl MetaRoom, params: &WyslijKarteParams) {
    recepcja_karty::wyslij_karte(room, params).expect("send the card");
}

/// The parameters of a card of a request to buy, with the requester's contact only, for the writes
/// a rule refuses.
fn bare_card(wplyw_id: i64) -> WyslijKarteParams {
    WyslijKarteParams {
        rodzaj: RodzajKarty::NoweZgloszenie,
        wplyw_id,
        wyslano: "2026-11-04T09:30:00Z".to_string(),
        rodzaj_sprawy: Some(WpisyRodzajSprawy::Kupno),
        rodzaj_nieruchomosci: None,
        miejscowosc: None,
        kanal: None,
        zglaszajacy: Some("Jan Kowalski".to_string()),
        kontakt: Some("jan@example.com".to_string()),
        wiadomosc_odnosnik: "skrzynka/2".to_string(),
    }
}

fn resolve_card(room: &MemoryRoom, params: &OdnotujRozstrzygniecieKartyParams) {
    recepcja_karty::odnotuj_rozstrzygniecie_karty(room, params)
        .expect("record the resolution of the card");
}

/// One row in every table of the Reception's own kinds, for the checks of R9 and replacement.
fn full_memory(room: &impl MetaRoom) {
    add_case(room, &case());
    add_mark(room, &mark(1));
    log_message(room, &message_by_mark());
    log_message(
        room,
        &OdnotujWplywParams {
            wiadomosc_odnosnik: "skrzynka/2".to_string(),
            ..message_to_card()
        },
    );
    send_card(room, &card(2));
}

// A whole day -------------------------------------------------------------------------------------

/// Recepcja przypisuje wiadomości po znakach, wysyła kartę i składa meldunek
#[test]
fn whole_day() {
    let room = new_room();
    accept_order(
        &room,
        &PrzyjmijPolecenieParams {
            wydal_jako: WpisyWydalJako::Upowazniony,
            tresc: "Wstrzymaj rozdział ze skrzynki portalu".to_string(),
            ..order()
        },
    );

    add_case(&room, &case());
    add_mark(&room, &mark(1));
    log_message(&room, &message_by_mark());
    log_message(
        &room,
        &OdnotujWplywParams {
            wplynelo: "2026-11-04T10:00".to_string(),
            kanal: DziennikKanal::Portal,
            nadawca: "konto-portalu-123".to_string(),
            rodzaj_pisma: "zapytanie o ofertę".to_string(),
            wiadomosc_odnosnik: "skrzynka/portal-1".to_string(),
            sprawa_id: Some(1),
            znak_id: None,
            katalog_odnosnik: Some("pokoj-katalogu/oferta-031".to_string()),
            oddano: DziennikOddano::AgentSprawy,
            oddano_komu: Some("tozsamosc-opiekuna-oferty".to_string()),
            oddano_o: Some("2026-11-04T11:30".to_string()),
            kanal_nie_pasuje: false,
        },
    );

    log_message(
        &room,
        &OdnotujWplywParams {
            wiadomosc_odnosnik: "skrzynka/2".to_string(),
            ..message_to_card()
        },
    );
    send_card(&room, &card(3));
    resolve_card(
        &room,
        &OdnotujRozstrzygniecieKartyParams {
            rozstrzygnieto: "2026-11-04T12:00:00Z".to_string(),
            rozstrzygniecie: WpisyRozstrzygniecie::Przydzielona,
            dzial: Some(WpisyDzial::Posrednictwo),
            id: 1,
        },
    );

    log_message(
        &room,
        &OdnotujWplywParams {
            wplynelo: "2026-11-04T13:00".to_string(),
            nadawca: "kancelaria@falszywa.example".to_string(),
            rodzaj_pisma: "pismo notariusza".to_string(),
            wiadomosc_odnosnik: "skrzynka/3".to_string(),
            kanal_nie_pasuje: true,
            ..message()
        },
    );
    add_source(
        &room,
        &WpiszZrodloParams {
            nazwa: Some("kancelaria@falszywa.example".to_string()),
            rodzaj: RodzajZrodla::Firma,
            ..source("falszywa-kancelaria")
        },
    );
    add_source_event(
        &room,
        &DopiszZdarzenieZrodlaParams {
            zaszlo: "2026-11-04T13:00:00Z".to_string(),
            dotyczy_tabela: Some("recepcja_wplywy_dziennik".to_string()),
            dotyczy_odnosnik: Some("pokoj-recepcji/4".to_string()),
            ..source_event("falszywa-kancelaria", ZdarzeniaRodzaj::Podszywanie)
        },
    );

    let matching = recepcja_sprawy::sprawy_po_znaku(
        &room,
        &SprawyPoZnakuParams {
            rodzaj: ZnakiRodzaj::AdresNieruchomosci,
            wartosc: "ul. Polna 7 m. 12, Wrocław".to_string(),
        },
    )
    .expect("read the cases by the mark");
    let matching_case_files: Vec<&str> = matching
        .iter()
        .map(|row| row.teczka_odnosnik.as_str())
        .collect();
    assert_eq!(matching_case_files, ["pokoj-teczki-1"]);

    let waiting_cards = recepcja_karty::karty_czekajace(&room).expect("read the waiting cards");
    assert!(waiting_cards.is_empty());

    let daily_report = recepcja_karty::meldunek_dzienny(
        &room,
        &MeldunekDziennyParams {
            dzien: "2026-11-04".to_string(),
        },
    )
    .expect("read the daily report");
    assert_eq!(
        daily_report,
        Some(MeldunekDziennyRow {
            wplynelo: 4,
            oddano_agentom: 2,
            zapytania_po_godzinie: 1,
            wyslano_kart: 1,
            karty_czekaja_od_wczesniej: 0,
            niejednoznaczne: 0,
        })
    );

    let by_channel = recepcja_wplywy::wplywy_wedlug_kanalow(
        &room,
        &WplywyWedlugKanalowParams {
            dzien: "2026-11-04".to_string(),
        },
    )
    .expect("read the messages by channel");
    assert_eq!(
        by_channel,
        [
            WplywyWedlugKanalowRow {
                kanal: DziennikKanal::Email,
                liczba: 2,
            },
            WplywyWedlugKanalowRow {
                kanal: DziennikKanal::Formularz,
                liczba: 1,
            },
            WplywyWedlugKanalowRow {
                kanal: DziennikKanal::Portal,
                liczba: 1,
            },
        ]
    );

    let messages_of_case =
        recepcja_wplywy::wplywy_sprawy(&room, &WplywySprawyParams { sprawa_id: 1 })
            .expect("read the messages of the case");
    assert_eq!(messages_of_case.len(), 2);

    let source_events =
        recepcja::stanowisko_zrodla::zdarzenia_zrodel(&room).expect("read the events of sources");
    assert_eq!(source_events[0].rodzaj, RodzajZdarzeniaWPokoju::Podszywanie);
}

// Cases and marks ---------------------------------------------------------------------------------

/// sprawa powstaje otwarta, a zmienia ją tylko nowe przekazanie kierownika działu
#[test]
fn case_changes_only_by_handover() {
    let room = new_room();
    add_case(&room, &case());

    expect::refused(
        room.execute(
            "INSERT INTO recepcja_sprawy_wpisy (teczka_odnosnik, rodzaj, oznaczenie, agent, stan,
              otwarto, zamknieto, przekazal, przekazano)
            VALUES ('pokoj-teczki-2', 'najem', 'Najem 2026/4', 'a', 'zamknieta', '2026-11-01',
              '2026-11-02', 'k', '2026-11-02')",
            &[],
        ),
        "sprawa powstaje w spisie otwarta",
    );

    let rule = "sprawę w spisie zmienia tylko nowe przekazanie kierownika działu";
    expect::refused(
        recepcja_sprawy::zmien_sprawe(
            &room,
            &ZmienSpraweParams {
                oznaczenie: "Oferta 2026/031".to_string(),
                agent: "tozsamosc-innego-agenta".to_string(),
                kancelaria_xnotariusz: None,
                przekazal: "tozsamosc-kierownika-posrednictwa".to_string(),
                przekazano: "2026-11-02T09:05:00Z".to_string(),
                id: 1,
            },
        ),
        rule,
    );
    recepcja_sprawy::zmien_sprawe(
        &room,
        &ZmienSpraweParams {
            oznaczenie: "Oferta 2026/031".to_string(),
            agent: "tozsamosc-nastepcy".to_string(),
            kancelaria_xnotariusz: None,
            przekazal: "tozsamosc-kierownika-posrednictwa".to_string(),
            przekazano: "2026-11-06T09:00:00Z".to_string(),
            id: 1,
        },
    )
    .expect("change the case");

    let cases = recepcja_sprawy::spis_spraw(&room).expect("read the list of cases");
    assert_eq!(cases[0].agent, "tozsamosc-nastepcy");
}

/// zamknięta sprawa ma datę zamknięcia i zostaje w spisie bez zmian
#[test]
fn closed_case_stays() {
    let room = new_room();
    add_case(&room, &case());

    // A writer outside the client can leave out the closing date that `zamknij_sprawe` requires.
    // The table refuses the closing without one.
    let closing_without_date = room.execute(
        "UPDATE recepcja_sprawy_wpisy
        SET stan = 'zamknieta', zamknieto = :zamknieto, przekazal = :przekazal,
          przekazano = :przekazano
        WHERE id = :id",
        &[
            ("zamknieto", SqlValue::Null),
            ("przekazal", text("k")),
            ("przekazano", text("2026-12-01T09:00:00Z")),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(closing_without_date, "zamknieta_z_data_zamkniecia");
    expect::refused(
        recepcja_sprawy::zamknij_sprawe(
            &room,
            &ZamknijSpraweParams {
                zamknieto: "2026-11-01T00:00:00Z".to_string(),
                przekazal: "k".to_string(),
                przekazano: "2026-12-01T09:00:00Z".to_string(),
                id: 1,
            },
        ),
        "zamkniecie_nie_przed_otwarciem",
    );
    recepcja_sprawy::zamknij_sprawe(
        &room,
        &ZamknijSpraweParams {
            zamknieto: "2026-12-01T00:00:00Z".to_string(),
            przekazal: "k".to_string(),
            przekazano: "2026-12-01T09:00:00Z".to_string(),
            id: 1,
        },
    )
    .expect("close the case");

    expect::refused(
        recepcja_sprawy::zmien_sprawe(
            &room,
            &ZmienSpraweParams {
                oznaczenie: "Oferta 2026/031".to_string(),
                agent: "a".to_string(),
                kancelaria_xnotariusz: None,
                przekazal: "k".to_string(),
                przekazano: "2027-01-01T09:00:00Z".to_string(),
                id: 1,
            },
        ),
        "zamkniętej sprawy wcale",
    );
    expect::refused(
        room.execute(
            "UPDATE recepcja_sprawy_wpisy
            SET stan = 'otwarta', zamknieto = NULL, przekazano = '2027-01-02'",
            &[],
        ),
        "teczki, rodzaju ani daty otwarcia sprawy się nie zmienia",
    );
}

/// kancelaria na XNotariuszu jest odnotowana tylko przy transakcji
#[test]
fn notary_only_for_transaction() {
    let room = new_room();
    add_case(
        &room,
        &WpiszSpraweParams {
            teczka_odnosnik: "pokoj-transakcji-1".to_string(),
            rodzaj: RodzajSprawy::Transakcja,
            kancelaria_xnotariusz: Some(true),
            ..case()
        },
    );

    let rule = "kancelaria_tylko_przy_transakcji";
    expect::refused(
        recepcja_sprawy::wpisz_sprawe(
            &room,
            &WpiszSpraweParams {
                teczka_odnosnik: "t2".to_string(),
                kancelaria_xnotariusz: Some(false),
                ..bare_case()
            },
        ),
        rule,
    );
    expect::refused(
        recepcja_sprawy::wpisz_sprawe(
            &room,
            &WpiszSpraweParams {
                teczka_odnosnik: "t3".to_string(),
                rodzaj: RodzajSprawy::Transakcja,
                ..bare_case()
            },
        ),
        rule,
    );
}

/// znak powstaje niewycofany, wycofuje się go raz i nie przed dodaniem
#[test]
fn mark_is_withdrawn_once() {
    let room = new_room();
    add_case(&room, &case());
    add_mark(&room, &mark(1));

    expect::refused(
        room.execute(
            "INSERT INTO recepcja_sprawy_znaki (sprawa_id, rodzaj, wartosc, przekazal, dodano,
              wycofano)
            VALUES (1, 'adres_poczty_klienta', 'nowak@example.com', 'k', '2026-11-02',
              '2026-11-03')",
            &[],
        ),
        "znak powstaje niewycofany",
    );
    expect::refused(
        recepcja_sprawy::wycofaj_znak_sprawy(
            &room,
            &WycofajZnakSprawyParams {
                wycofano: "2026-11-01T00:00:00Z".to_string(),
                id: 1,
            },
        ),
        "wycofanie_nie_przed_dodaniem",
    );
    expect::refused(
        room.execute(
            "UPDATE recepcja_sprawy_znaki SET wartosc = 'ul. Polna 9' WHERE id = 1",
            &[],
        ),
        "znaku się nie zmienia",
    );

    withdraw_mark(&room, 1, "2026-11-06T09:00:00Z");
    expect::refused(
        recepcja_sprawy::wycofaj_znak_sprawy(
            &room,
            &WycofajZnakSprawyParams {
                wycofano: "2026-11-07T09:00:00Z".to_string(),
                id: 1,
            },
        ),
        "znaku się nie zmienia",
    );

    let matching = recepcja_sprawy::sprawy_po_znaku(
        &room,
        &SprawyPoZnakuParams {
            rodzaj: ZnakiRodzaj::AdresNieruchomosci,
            wartosc: "ul. Polna 7 m. 12, Wrocław".to_string(),
        },
    )
    .expect("read the cases by the mark");
    assert!(matching.is_empty());

    let marks = recepcja_sprawy::znaki_sprawy(&room, &ZnakiSprawyParams { sprawa_id: 1 })
        .expect("read the marks of the case");
    assert_eq!(marks.len(), 1);
}

// Log of incoming messages ------------------------------------------------------------------------

/// wpływ ma czas co do minuty, a sprawę tylko po znaku albo numerze z katalogu
#[test]
fn message_case_only_by_mark() {
    let room = new_room();
    add_case(&room, &case());
    add_mark(&room, &mark(1));
    log_message(&room, &message_by_mark());

    expect::refused(
        recepcja_wplywy::odnotuj_wplyw(
            &room,
            &OdnotujWplywParams {
                wplynelo: "2026-11-04".to_string(),
                ..bare_message()
            },
        ),
        "wplynelo_co_do_minuty",
    );

    let rule = "sprawa_tylko_po_znaku_albo_katalogu";
    expect::refused(
        recepcja_wplywy::odnotuj_wplyw(
            &room,
            &OdnotujWplywParams {
                sprawa_id: Some(1),
                ..bare_message()
            },
        ),
        rule,
    );
    expect::refused(
        recepcja_wplywy::odnotuj_wplyw(
            &room,
            &OdnotujWplywParams {
                sprawa_id: Some(1),
                znak_id: Some(1),
                katalog_odnosnik: Some("pokoj-katalogu/oferta-031".to_string()),
                ..bare_message()
            },
        ),
        rule,
    );
}

/// znak musi być niewycofanym znakiem tej sprawy, a numer z katalogu wskazuje ofertę
#[test]
fn mark_of_the_same_case() {
    let room = new_room();
    add_case(&room, &case());
    add_case(
        &room,
        &WpiszSpraweParams {
            teczka_odnosnik: "pokoj-najmu-1".to_string(),
            rodzaj: RodzajSprawy::Najem,
            oznaczenie: "Najem 2026/4".to_string(),
            ..case()
        },
    );
    add_mark(&room, &mark(1));
    add_mark(
        &room,
        &DodajZnakSprawyParams {
            wartosc: "ul. Leśna 3, Wrocław".to_string(),
            ..mark(2)
        },
    );
    withdraw_mark(&room, 2, "2026-11-03T09:00:00Z");

    let rule = "wiadomość trafia do sprawy po jej niewycofanym znaku";
    expect::refused(
        recepcja_wplywy::odnotuj_wplyw(
            &room,
            &OdnotujWplywParams {
                sprawa_id: Some(2),
                znak_id: Some(1),
                ..bare_message()
            },
        ),
        rule,
    );
    expect::refused(
        recepcja_wplywy::odnotuj_wplyw(
            &room,
            &OdnotujWplywParams {
                sprawa_id: Some(2),
                znak_id: Some(2),
                ..bare_message()
            },
        ),
        rule,
    );

    expect::refused(
        recepcja_wplywy::odnotuj_wplyw(
            &room,
            &OdnotujWplywParams {
                sprawa_id: Some(2),
                katalog_odnosnik: Some("pokoj-katalogu/oferta-031".to_string()),
                ..bare_message()
            },
        ),
        "po numerze z katalogu ofert wiadomość trafia do sprawy oferty",
    );
    log_message(
        &room,
        &OdnotujWplywParams {
            sprawa_id: Some(1),
            katalog_odnosnik: Some("pokoj-katalogu/oferta-031".to_string()),
            ..bare_message()
        },
    );
}

/// oddana wiadomość ma adresata i czas, a agent sprawy dostaje ją tylko ze sprawą
#[test]
fn handed_over_message() {
    let room = new_room();
    log_message(
        &room,
        &OdnotujWplywParams {
            rodzaj_pisma: "list do pracownika".to_string(),
            oddano: DziennikOddano::Pracownik,
            oddano_komu: Some("tozsamosc-asystenta".to_string()),
            oddano_o: Some("2026-11-04T09:15".to_string()),
            ..message()
        },
    );

    expect::refused(
        recepcja_wplywy::odnotuj_wplyw(
            &room,
            &OdnotujWplywParams {
                oddano: DziennikOddano::AgentSprawy,
                oddano_komu: Some("a".to_string()),
                oddano_o: Some("2026-11-04T09:20".to_string()),
                ..bare_message()
            },
        ),
        "agent_sprawy_dostaje_wiadomosc_ze_sprawa",
    );
    expect::refused(
        recepcja_wplywy::odnotuj_wplyw(
            &room,
            &OdnotujWplywParams {
                oddano: DziennikOddano::Pracownik,
                oddano_o: Some("2026-11-04T09:20".to_string()),
                ..bare_message()
            },
        ),
        "oddane_komus",
    );
    expect::refused(
        recepcja_wplywy::odnotuj_wplyw(
            &room,
            &OdnotujWplywParams {
                oddano: DziennikOddano::Karta,
                oddano_komu: Some("k".to_string()),
                oddano_o: Some("2026-11-04T09:20".to_string()),
                ..bare_message()
            },
        ),
        "oddane_komus",
    );
    expect::refused(
        recepcja_wplywy::odnotuj_wplyw(
            &room,
            &OdnotujWplywParams {
                oddano: DziennikOddano::Karta,
                ..bare_message()
            },
        ),
        "oddane_z_czasem",
    );
    expect::refused(
        recepcja_wplywy::odnotuj_wplyw(
            &room,
            &OdnotujWplywParams {
                oddano: DziennikOddano::Karta,
                oddano_o: Some("2026-11-04T09:00".to_string()),
                ..bare_message()
            },
        ),
        "oddanie_nie_przed_wplywem",
    );
}

/// dziennik wpływów tylko się dopisuje
#[test]
fn log_is_append_only() {
    let room = new_room();
    log_message(&room, &message());

    expect::refused(
        room.execute(
            "UPDATE recepcja_wplywy_dziennik SET nadawca = 'inny' WHERE id = 1",
            &[],
        ),
        "dziennik wpływów tylko się dopisuje",
    );

    let from_sender = recepcja_wplywy::wplywy_od_nadawcy(
        &room,
        &WplywyOdNadawcyParams {
            nadawca: "administracja@polna7.example".to_string(),
        },
    )
    .expect("read the messages from the sender");
    assert_eq!(from_sender.len(), 1);

    let log_of_day = recepcja_wplywy::dziennik_dnia(
        &room,
        &DziennikDniaParams {
            dzien: "2026-11-04".to_string(),
        },
    )
    .expect("read the log of the day");
    assert_eq!(log_of_day.len(), 1);
}

// Cards -------------------------------------------------------------------------------------------

/// karta stoi na wpływie oddanym na kartę i powstaje czekająca z danymi zgłoszenia
#[test]
fn card_starts_waiting() {
    let room = new_room();
    log_message(&room, &message());
    log_message(&room, &message_to_card());
    send_card(&room, &card(2));

    expect::refused(
        recepcja_karty::wyslij_karte(&room, &bare_card(1)),
        "karta stoi na wpisie dziennika oddanym na kartę",
    );
    expect::refused(
        recepcja_karty::wyslij_karte(
            &room,
            &WyslijKarteParams {
                kontakt: None,
                ..bare_card(2)
            },
        ),
        "czekajace_zgloszenie_z_danymi",
    );
    send_card(
        &room,
        &WyslijKarteParams {
            rodzaj: RodzajKarty::PismoBezSprawy,
            rodzaj_sprawy: None,
            kontakt: None,
            ..bare_card(2)
        },
    );
    expect::refused(
        room.execute(
            "INSERT INTO recepcja_karty_wpisy (rodzaj, wplyw_id, wyslano, wiadomosc_odnosnik, stan,
              rozstrzygnieto, rozstrzygniecie)
            VALUES ('skarga', 2, '2026-11-04', 'skrzynka/9', 'rozstrzygnieta', '2026-11-04',
              'odrzucona')",
            &[],
        ),
        "karta powstaje jako czekająca",
    );

    let waiting_cards = recepcja_karty::karty_czekajace(&room).expect("read the waiting cards");
    assert_eq!(waiting_cards.len(), 2);
}

/// po rozstrzygnięciu dane zgłaszającego znikają z karty, a dział ma tylko przydział
#[test]
fn resolved_card_has_no_contact() {
    let room = new_room();
    log_message(&room, &message_to_card());
    send_card(&room, &card(1));

    expect::refused(
        room.execute(
            "UPDATE recepcja_karty_wpisy
            SET stan = 'rozstrzygnieta', rozstrzygnieto = '2026-11-04T12:00:00Z',
              rozstrzygniecie = 'odrzucona'
            WHERE id = 1",
            &[],
        ),
        "rozstrzygnieta_bez_danych_zglaszajacego",
    );

    // A writer outside the client can leave out the date of the resolution that
    // `odnotuj_rozstrzygniecie_karty` requires. The table refuses the resolution without one.
    let resolution_without_date = room.execute(
        "UPDATE recepcja_karty_wpisy
        SET stan = 'rozstrzygnieta', rozstrzygnieto = :rozstrzygnieto,
          rozstrzygniecie = :rozstrzygniecie, dzial = :dzial, zglaszajacy = NULL, kontakt = NULL
        WHERE id = :id",
        &[
            ("rozstrzygnieto", SqlValue::Null),
            ("rozstrzygniecie", text("odrzucona")),
            ("dzial", SqlValue::Null),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(resolution_without_date, "rozstrzygnieta_z_data_i_sposobem");
    expect::refused(
        recepcja_karty::odnotuj_rozstrzygniecie_karty(
            &room,
            &OdnotujRozstrzygniecieKartyParams {
                rozstrzygnieto: "2026-11-04T12:00:00Z".to_string(),
                rozstrzygniecie: WpisyRozstrzygniecie::Przydzielona,
                dzial: None,
                id: 1,
            },
        ),
        "dzial_przy_przydziale",
    );
    expect::refused(
        recepcja_karty::odnotuj_rozstrzygniecie_karty(
            &room,
            &OdnotujRozstrzygniecieKartyParams {
                rozstrzygnieto: "2026-11-04T12:00:00Z".to_string(),
                rozstrzygniecie: WpisyRozstrzygniecie::Odrzucona,
                dzial: Some(WpisyDzial::Najem),
                id: 1,
            },
        ),
        "dzial_przy_przydziale",
    );
    expect::refused(
        recepcja_karty::odnotuj_rozstrzygniecie_karty(
            &room,
            &OdnotujRozstrzygniecieKartyParams {
                rozstrzygnieto: "2026-11-04T08:00:00Z".to_string(),
                rozstrzygniecie: WpisyRozstrzygniecie::Odrzucona,
                dzial: None,
                id: 1,
            },
        ),
        "rozstrzygniecie_nie_przed_wyslaniem",
    );

    resolve_card(
        &room,
        &OdnotujRozstrzygniecieKartyParams {
            rozstrzygnieto: "2026-11-04T12:00:00Z".to_string(),
            rozstrzygniecie: WpisyRozstrzygniecie::Przydzielona,
            dzial: Some(WpisyDzial::Najem),
            id: 1,
        },
    );

    let cards = room
        .execute(
            "SELECT zglaszajacy, kontakt, dzial FROM recepcja_karty_wpisy",
            &[],
        )
        .expect("read the card");
    let requester = cards[0]
        .get::<Option<String>>("zglaszajacy")
        .expect("read the requester of the card");
    let contact = cards[0]
        .get::<Option<String>>("kontakt")
        .expect("read the contact of the card");
    let department = cards[0]
        .get::<Option<String>>("dzial")
        .expect("read the department of the card");
    assert_eq!(requester, None);
    assert_eq!(contact, None);
    assert_eq!(department.as_deref(), Some("najem"));
}

/// wysłanej karty się nie zmienia, a rozstrzygniętej wcale
#[test]
fn sent_card_does_not_change() {
    let room = new_room();
    log_message(&room, &message_to_card());
    send_card(&room, &card(1));

    let rule = "wysłanej karty się nie zmienia";
    expect::refused(
        room.execute(
            "UPDATE recepcja_karty_wpisy SET miejscowosc = 'Opole' WHERE id = 1",
            &[],
        ),
        rule,
    );
    resolve_card(
        &room,
        &OdnotujRozstrzygniecieKartyParams {
            rozstrzygnieto: "2026-11-04T12:00:00Z".to_string(),
            rozstrzygniecie: WpisyRozstrzygniecie::Odrzucona,
            dzial: None,
            id: 1,
        },
    );
    expect::refused(
        recepcja_karty::odnotuj_rozstrzygniecie_karty(
            &room,
            &OdnotujRozstrzygniecieKartyParams {
                rozstrzygnieto: "2026-11-05T12:00:00Z".to_string(),
                rozstrzygniecie: WpisyRozstrzygniecie::Przydzielona,
                dzial: Some(WpisyDzial::Najem),
                id: 1,
            },
        ),
        rule,
    );
}

// R9 and replacement ------------------------------------------------------------------------------

/// The tables of the Reception's own kinds in the room.
fn own_tables_of(room: &MemoryRoom) -> Vec<String> {
    let rows = room
        .execute(
            r"SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'recepcja\_%' ESCAPE '\'",
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
    assert_eq!(tables.len(), 4, "{tables:?}");

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

/// REPLACE nie usuwa sprawy tej samej teczki przez klucz UNIQUE
#[test]
fn no_replacement_through_unique_case_file() {
    let room = new_room();
    full_memory(&room);

    let table = "recepcja_sprawy_wpisy";
    let column_rows = room
        .execute(&format!("PRAGMA table_info({table})"), &[])
        .expect("read the columns of the table");
    let column_names: Vec<String> = column_rows
        .iter()
        .map(|row| row.get::<String>("name").expect("a column has a name"))
        .collect();
    let selected: Vec<&str> = column_names
        .iter()
        .map(|name| if name == "id" { "99" } else { name.as_str() })
        .collect();
    let columns = column_names.join(", ");
    let selected = selected.join(", ");

    let replace =
        format!("INSERT OR REPLACE INTO {table} ({columns}) SELECT {selected} FROM {table}");
    expect::refused(room.execute(&replace, &[]), REPLACEMENT);

    // `WHERE true` keeps SQLite from reading ON CONFLICT as a join constraint.
    let upsert = format!(
        "INSERT INTO {table} ({columns}) SELECT {selected} FROM {table} \
        WHERE true ON CONFLICT (teczka_odnosnik) DO UPDATE SET id = excluded.id"
    );
    expect::refused(room.execute(&upsert, &[]), REPLACEMENT);
}
