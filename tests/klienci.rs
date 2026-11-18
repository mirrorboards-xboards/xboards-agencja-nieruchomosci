//! Scenarios of the client file of the office (spec/registers/klienci).
//!
//! Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
//! allows and a write it refuses. The first scenario follows a client who changes her name and phone
//! number and is still recognised by the old number, as the profile describes it.

use xboards_agencja_nieruchomosci::registers::klienci::{self, klienci_kartoteka};
use xboards_agencja_nieruchomosci::xmeta_room::{FromSqlValue, MemoryRoom};

use klienci_kartoteka::{
    AdresyKlientaParams, AdresyRodzaj, DaneKlientaParams, InformacjeKlientaParams,
    InformacjeSposob, KlienciPoKontakcieParams, KlienciPoNazwiskuAlboNazwieParams,
    KoniecOstatniejUmowyKlientaParams, KontaktyKlientaParams, KontaktyRodzaj, OsobyFunkcja,
    OsobyKlientaParams, UmowyDzial, UmowyKlientaParams, UmowyRodzaj, UmowyStanyStan,
    UmowyTypTeczki, WpisyRodzajPodmiotu, WpiszAdresKlientaParams, WpiszDaneKlientaParams,
    WpiszInformacjeKlientaParams, WpiszKlientaParams, WpiszKontaktKlientaParams,
    WpiszOsobeKlientaParams, WpiszUmoweKlientaParams, ZakonczAdresKlientaParams,
    ZakonczKontaktKlientaParams, ZakonczOsobeKlientaParams, ZmienStanUmowyKlientaParams,
};

use crate::expect;

const REPLACEMENT: &str =
    "wiersza kartoteki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE";
const DELETION: &str = "wierszy kartoteki się nie usuwa (R9)";
const UNCHANGED: &str = "wpisu klienta, umowy, jej stanu ani informacji się nie zmienia";

fn new_room() -> MemoryRoom {
    klienci::open_in_memory().expect("open the client file room")
}

// Building blocks ---------------------------------------------------------------------------------

/// The default parameters of a client who is a natural person.
fn client(client_id: &str) -> WpiszKlientaParams {
    WpiszKlientaParams {
        id: client_id.to_string(),
        rodzaj_podmiotu: WpisyRodzajPodmiotu::OsobaFizyczna,
        wpisal: "kierownik-posrednictwa".to_string(),
        wpisano: "2026-03-02".to_string(),
        od: "2026-03-02".to_string(),
        imiona: Some("Anna".to_string()),
        nazwisko: Some("Nowak".to_string()),
        pesel: Some("80010112345".to_string()),
        nazwa: None,
        forma_prawna: None,
        krs: None,
        ceidg: false,
        nip: None,
        regon: None,
        firma: None,
        zrodlo_odnosnik: "teczka-oferty/dokument-umowa".to_string(),
        zrodlo_opis: None,
    }
}

fn add_client(room: &MemoryRoom, params: &WpiszKlientaParams) {
    klienci_kartoteka::wpisz_klienta(room, params).expect("add the client");
}

fn add_company(room: &MemoryRoom) {
    let params = WpiszKlientaParams {
        rodzaj_podmiotu: WpisyRodzajPodmiotu::OsobaPrawna,
        imiona: None,
        nazwisko: None,
        pesel: None,
        nazwa: Some("Leśna Inwestycje sp. z o.o.".to_string()),
        forma_prawna: Some("spółka z ograniczoną odpowiedzialnością".to_string()),
        krs: Some("0000123456".to_string()),
        ..client("lesna")
    };

    add_client(room, &params);
}

/// The default parameters of a sale brokerage contract of the client „nowak”.
fn contract() -> WpiszUmoweKlientaParams {
    WpiszUmoweKlientaParams {
        id: "umowa".to_string(),
        klient_id: "nowak".to_string(),
        rodzaj: UmowyRodzaj::PosrednictwoSprzedazy,
        dzial: UmowyDzial::Posrednictwo,
        nieruchomosc_adres: Some("ul. Polna 7 m. 12, Wrocław".to_string()),
        numer_kw: Some("WR1K/00012345/6".to_string()),
        teczka_odnosnik: "teczka-oferty".to_string(),
        typ_teczki: UmowyTypTeczki::Oferta,
        oznaczenie: "Oferta 2026/031".to_string(),
        umowa_odnosnik: "teczka-oferty/umowa".to_string(),
        podpisal_za_biuro: "posrednik".to_string(),
        zawarto: "2026-03-02".to_string(),
        koniec: None,
        konsument: true,
        obie_strony: false,
        wpisal: "kierownik-posrednictwa".to_string(),
        wpisano: "2026-03-02".to_string(),
    }
}

fn add_contract(room: &MemoryRoom, params: &WpiszUmoweKlientaParams) {
    klienci_kartoteka::wpisz_umowe_klienta(room, params).expect("add the contract");
}

fn add_phone(room: &MemoryRoom, wartosc: &str, od: &str) {
    let params = WpiszKontaktKlientaParams {
        klient_id: "nowak".to_string(),
        rodzaj: KontaktyRodzaj::Telefon,
        wartosc: wartosc.to_string(),
        preferowany: true,
        od: od.to_string(),
        zrodlo_odnosnik: "teczka-oferty/dokument-umowa".to_string(),
    };

    klienci_kartoteka::wpisz_kontakt_klienta(room, &params).expect("add the phone");
}

/// The first phone number of the client „nowak”.
const FIRST_PHONE: &str = "+48600100200";

/// The default parameters of the home address of the client „nowak”.
fn address() -> WpiszAdresKlientaParams {
    WpiszAdresKlientaParams {
        od: "2026-03-02".to_string(),
        klient_id: "nowak".to_string(),
        rodzaj: AdresyRodzaj::Zamieszkanie,
        adres: "ul. Polna 7 m. 12".to_string(),
        zrodlo_odnosnik: "teczka-oferty/dokument-umowa".to_string(),
    }
}

fn add_address(room: &MemoryRoom, params: &WpiszAdresKlientaParams) {
    klienci_kartoteka::wpisz_adres_klienta(room, params).expect("add the address");
}

fn end_contract(room: &MemoryRoom) {
    let params = ZmienStanUmowyKlientaParams {
        umowa_id: "umowa".to_string(),
        stan: UmowyStanyStan::Wykonana,
        od: "2026-09-30".to_string(),
        zrodlo_odnosnik: "teczka-oferty/raport-koncowy".to_string(),
        wpisal: "kierownik-posrednictwa".to_string(),
    };

    klienci_kartoteka::zmien_stan_umowy_klienta(room, &params).expect("end the contract");
}

/// The default parameters of a change of the state of the contract „umowa”.
fn state_change() -> ZmienStanUmowyKlientaParams {
    ZmienStanUmowyKlientaParams {
        umowa_id: "umowa".to_string(),
        stan: UmowyStanyStan::Wypowiedziana,
        od: "2026-05-01".to_string(),
        zrodlo_odnosnik: "teczka-oferty/wypowiedzenie".to_string(),
        wpisal: "kierownik-posrednictwa".to_string(),
    }
}

/// A client of every kind with a row in every table of the kind.
fn full_file(room: &MemoryRoom) {
    add_client(room, &client("nowak"));
    add_phone(room, FIRST_PHONE, "2026-03-02");
    add_address(room, &address());
    add_contract(room, &contract());

    let information = WpiszInformacjeKlientaParams {
        klient_id: "nowak".to_string(),
        poinformowano: "2026-03-02T10:00:00Z".to_string(),
        sposob: InformacjeSposob::Email,
        wysylka_odnosnik: "teczka-oferty/wysylka-rodo".to_string(),
    };
    klienci_kartoteka::wpisz_informacje_klienta(room, &information)
        .expect("record the information about the file");

    add_company(room);
    let person = WpiszOsobeKlientaParams {
        klient_id: "lesna".to_string(),
        funkcja: OsobyFunkcja::DzialaZaPodmiot,
        imiona: "Piotr".to_string(),
        nazwisko: "Leśny".to_string(),
        kontakt: None,
        podstawa_odnosnik: Some("teczka-oferty/odpis-krs".to_string()),
        od: "2026-04-01".to_string(),
        zrodlo_odnosnik: "teczka-oferty/umowa".to_string(),
    };
    klienci_kartoteka::wpisz_osobe_klienta(room, &person).expect("add the person of the entity");
}

/// The id of the newest contact of the client „nowak”.
fn newest_contact_id(room: &MemoryRoom) -> i64 {
    let contacts = klienci_kartoteka::kontakty_klienta(
        room,
        &KontaktyKlientaParams {
            klient_id: "nowak".to_string(),
        },
    )
    .expect("read the contacts");

    contacts[0].id
}

// A whole client ----------------------------------------------------------------------------------

/// Kartoteka: klientka zmienia nazwisko i telefon, a stary numer nadal ją wskazuje
#[test]
fn whole_client() {
    let room = new_room();
    add_client(&room, &client("nowak"));
    add_phone(&room, FIRST_PHONE, "2026-03-02");
    add_contract(&room, &contract());

    klienci_kartoteka::wpisz_dane_klienta(
        &room,
        &WpiszDaneKlientaParams {
            klient_id: "nowak".to_string(),
            od: "2027-05-10".to_string(),
            imiona: Some("Anna".to_string()),
            nazwisko: Some("Nowak-Lis".to_string()),
            pesel: Some("80010112345".to_string()),
            ceidg: false,
            zrodlo_odnosnik: "teczka-najmu/akt-malzenstwa".to_string(),
            zrodlo_opis: Some("odpis aktu małżeństwa".to_string()),
            wpisal: "kierownik-najmu".to_string(),
            ..WpiszDaneKlientaParams::default()
        },
    )
    .expect("record the new data of the client");
    let old_phone_id = newest_contact_id(&room);
    klienci_kartoteka::zakoncz_kontakt_klienta(
        &room,
        &ZakonczKontaktKlientaParams {
            r#do: Some("2027-05-10".to_string()),
            id: old_phone_id,
        },
    )
    .expect("end the old phone");
    add_phone(&room, "+48600999888", "2027-05-10");

    let versions = klienci_kartoteka::dane_klienta(
        &room,
        &DaneKlientaParams {
            klient_id: "nowak".to_string(),
        },
    )
    .expect("read the versions of the data");
    let names_and_currency: Vec<(Option<&str>, bool)> = versions
        .iter()
        .map(|row| (row.nazwisko.as_deref(), row.aktualne))
        .collect();
    assert_eq!(
        names_and_currency,
        [(Some("Nowak-Lis"), true), (Some("Nowak"), false)]
    );

    let clients = klienci_kartoteka::klienci(&room).expect("read the clients");
    assert_eq!(clients[0].nazwisko.as_deref(), Some("Nowak-Lis"));

    let found_by_contact = klienci_kartoteka::klienci_po_kontakcie(
        &room,
        &KlienciPoKontakcieParams {
            wartosc: FIRST_PHONE.to_string(),
        },
    )
    .expect("find the client by the old phone");
    assert_eq!(found_by_contact[0].klient_id, "nowak");

    let found_by_name = klienci_kartoteka::klienci_po_nazwisku_albo_nazwie(
        &room,
        &KlienciPoNazwiskuAlboNazwieParams {
            szukane: "Nowak".to_string(),
        },
    )
    .expect("find the client by the old name");
    assert_eq!(found_by_name[0].klient_id, "nowak");

    let last_contract_end = KoniecOstatniejUmowyKlientaParams {
        klient_id: "nowak".to_string(),
    };
    let end_before = klienci_kartoteka::koniec_ostatniej_umowy_klienta(&room, &last_contract_end)
        .expect("read the end of the last contract")
        .expect("the client has a contract");
    assert_eq!(end_before.koniec, None);

    end_contract(&room);
    let end_after = klienci_kartoteka::koniec_ostatniej_umowy_klienta(&room, &last_contract_end)
        .expect("read the end of the last contract")
        .expect("the client has a contract");
    assert_eq!(end_after.koniec.as_deref(), Some("2026-09-30"));

    let contracts = klienci_kartoteka::umowy_klienta(
        &room,
        &UmowyKlientaParams {
            klient_id: "nowak".to_string(),
        },
    )
    .expect("read the contracts");
    assert_eq!(contracts[0].stan, UmowyStanyStan::Wykonana);
}

// Data of a person and of an entity ---------------------------------------------------------------

/// Dane: osoba ma imiona i nazwisko, podmiot nazwę bez danych osoby
#[test]
fn person_and_entity_data() {
    let room = new_room();

    expect::refused(
        klienci_kartoteka::wpisz_klienta(
            &room,
            &WpiszKlientaParams {
                nazwisko: None,
                ..client("nowak")
            },
        ),
        "dane osoby fizycznej mają imiona i nazwisko",
    );
    expect::refused(
        klienci_kartoteka::wpisz_klienta(
            &room,
            &WpiszKlientaParams {
                rodzaj_podmiotu: WpisyRodzajPodmiotu::OsobaPrawna,
                nazwa: Some("Leśna sp. z o.o.".to_string()),
                ..client("lesna")
            },
        ),
        "dane podmiotu mają nazwę i nie mają imion, nazwiska ani PESEL",
    );
    expect::refused(
        klienci_kartoteka::wpisz_klienta(
            &room,
            &WpiszKlientaParams {
                pesel: Some("8001011234".to_string()),
                ..client("nowak")
            },
        ),
        "pesel_ma_11_cyfr",
    );
    expect::refused(
        klienci_kartoteka::wpisz_klienta(
            &room,
            &WpiszKlientaParams {
                krs: Some("0000123456".to_string()),
                ceidg: true,
                ..client("nowak")
            },
        ),
        "krs_albo_ceidg",
    );

    add_client(
        &room,
        &WpiszKlientaParams {
            firma: Some("Anna Nowak Usługi".to_string()),
            nip: Some("5250000000".to_string()),
            ceidg: true,
            ..client("nowak")
        },
    );
    add_company(&room);

    let clients = klienci_kartoteka::klienci(&room).expect("read the clients");
    assert_eq!(clients.len(), 2);
}

/// Dane: PESEL ma jeden klient; kolejna umowa trafia do istniejącego wpisu
#[test]
fn one_client_per_pesel() {
    let room = new_room();
    add_client(&room, &client("nowak"));

    expect::refused(
        klienci_kartoteka::wpisz_klienta(&room, &client("druga-nowak")),
        "ten PESEL ma już inny klient kartoteki",
    );

    add_contract(&room, &contract());
    add_contract(
        &room,
        &WpiszUmoweKlientaParams {
            id: "druga-umowa".to_string(),
            rodzaj: UmowyRodzaj::PosrednictwoKupna,
            typ_teczki: UmowyTypTeczki::Poszukiwanie,
            nieruchomosc_adres: None,
            numer_kw: None,
            ..contract()
        },
    );
    klienci_kartoteka::wpisz_dane_klienta(
        &room,
        &WpiszDaneKlientaParams {
            klient_id: "nowak".to_string(),
            od: "2026-06-01".to_string(),
            imiona: Some("Anna".to_string()),
            nazwisko: Some("Nowak".to_string()),
            pesel: Some("80010112345".to_string()),
            ceidg: false,
            zrodlo_odnosnik: "teczka-poszukiwania/umowa".to_string(),
            wpisal: "kierownik-posrednictwa".to_string(),
            ..WpiszDaneKlientaParams::default()
        },
    )
    .expect("record the data again");

    let contracts = klienci_kartoteka::umowy_klienta(
        &room,
        &UmowyKlientaParams {
            klient_id: "nowak".to_string(),
        },
    )
    .expect("read the contracts");
    assert_eq!(contracts.len(), 2);
}

// Versions of the data ----------------------------------------------------------------------------

/// The default parameters of a new version of the data of the client „nowak”.
fn data_version(od: &str, nazwisko: &str) -> WpiszDaneKlientaParams {
    WpiszDaneKlientaParams {
        klient_id: "nowak".to_string(),
        od: od.to_string(),
        imiona: Some("Anna".to_string()),
        nazwisko: Some(nazwisko.to_string()),
        ceidg: false,
        zrodlo_odnosnik: "teczka/aneks".to_string(),
        wpisal: "kierownik".to_string(),
        ..WpiszDaneKlientaParams::default()
    }
}

/// Wersje danych: nowa powstaje aktualna i odbiera aktualność poprzedniej
#[test]
fn data_versions() {
    let room = new_room();
    add_client(&room, &client("nowak"));

    expect::refused(
        room.execute(
            "INSERT INTO klienci_kartoteka_dane (klient_id, od, aktualne, imiona, nazwisko,
              zrodlo_odnosnik, wpisal)
            VALUES ('nowak', '2026-06-01', 0, 'Anna', 'Nowak', 'teczka/aneks', 'kierownik')",
            &[],
        ),
        "wersja danych klienta powstaje jako aktualna",
    );
    expect::refused(
        klienci_kartoteka::wpisz_dane_klienta(&room, &data_version("2026-01-01", "Nowak")),
        "nowa wersja danych klienta nie obowiązuje wcześniej niż aktualna",
    );

    klienci_kartoteka::wpisz_dane_klienta(&room, &data_version("2026-06-01", "Kowalska"))
        .expect("record a new version of the data");

    let versions = klienci_kartoteka::dane_klienta(
        &room,
        &DaneKlientaParams {
            klient_id: "nowak".to_string(),
        },
    )
    .expect("read the versions of the data");
    let current_names: Vec<Option<&str>> = versions
        .iter()
        .filter(|row| row.aktualne)
        .map(|row| row.nazwisko.as_deref())
        .collect();
    assert_eq!(current_names, [Some("Kowalska")]);
}

/// Wersje danych: wersji się nie zmienia, a aktualnej nie odbiera się bez nowszej
#[test]
fn data_unchanged() {
    let room = new_room();
    add_client(&room, &client("nowak"));

    expect::refused(
        room.execute(
            "UPDATE klienci_kartoteka_dane SET nazwisko = 'Lis' WHERE klient_id = 'nowak'",
            &[],
        ),
        "danych klienta się nie zmienia",
    );
    expect::refused(
        room.execute(
            "UPDATE klienci_kartoteka_dane SET aktualne = 0 WHERE klient_id = 'nowak'",
            &[],
        ),
        "danych klienta się nie zmienia",
    );

    klienci_kartoteka::wpisz_dane_klienta(&room, &data_version("2026-06-01", "Lis"))
        .expect("record a new version of the data");
    expect::refused(
        room.execute(
            "UPDATE klienci_kartoteka_dane SET aktualne = 1 WHERE nazwisko = 'Nowak'",
            &[],
        ),
        "danych klienta się nie zmienia",
    );
}

// Addresses, contacts and persons -----------------------------------------------------------------

/// Adresy: zamieszkanie ma osoba, siedzibę podmiot; bieżący adres rodzaju jest jeden
#[test]
fn addresses() {
    let room = new_room();
    add_client(&room, &client("nowak"));
    add_company(&room);

    expect::refused(
        klienci_kartoteka::wpisz_adres_klienta(
            &room,
            &WpiszAdresKlientaParams {
                rodzaj: AdresyRodzaj::Siedziba,
                adres: "ul. Leśna 1, Poznań".to_string(),
                zrodlo_odnosnik: "teczka/umowa".to_string(),
                ..address()
            },
        ),
        "miejsce zamieszkania ma tylko osoba fizyczna, a siedzibę tylko podmiot",
    );
    add_address(
        &room,
        &WpiszAdresKlientaParams {
            klient_id: "lesna".to_string(),
            rodzaj: AdresyRodzaj::Siedziba,
            adres: "ul. Leśna 1, Poznań".to_string(),
            ..address()
        },
    );

    add_address(&room, &address());
    add_address(
        &room,
        &WpiszAdresKlientaParams {
            adres: "ul. Długa 3".to_string(),
            od: "2026-08-01".to_string(),
            ..address()
        },
    );
    let addresses = klienci_kartoteka::adresy_klienta(
        &room,
        &AdresyKlientaParams {
            klient_id: "nowak".to_string(),
        },
    )
    .expect("read the addresses");
    let addresses_and_ends: Vec<(&str, Option<&str>)> = addresses
        .iter()
        .map(|row| (row.adres.as_str(), row.r#do.as_deref()))
        .collect();
    assert_eq!(
        addresses_and_ends,
        [
            ("ul. Długa 3", None),
            ("ul. Polna 7 m. 12", Some("2026-08-01")),
        ]
    );

    // A second current address of a kind, written past the operation that ends the current one.
    expect::refused(
        room.execute(
            "INSERT INTO klienci_kartoteka_adresy (klient_id, rodzaj, adres, od, zrodlo_odnosnik)
            VALUES ('nowak', 'zamieszkanie', 'ul. Krótka 5', '2026-09-01', 'teczka/aneks')",
            &[],
        ),
        "klient ma już bieżący adres tego rodzaju",
    );
}

/// Adresy, kontakty i osoby powstają bieżące i mogą się tylko raz zakończyć
#[test]
fn ending_rows() {
    let room = new_room();
    add_client(&room, &client("nowak"));

    expect::refused(
        room.execute(
            "INSERT INTO klienci_kartoteka_adresy (klient_id, rodzaj, adres, od, do, zrodlo_odnosnik)
            VALUES ('nowak', 'zamieszkanie', 'ul. Polna 7', '2026-03-02', '2026-04-01', 'teczka/umowa')",
            &[],
        ),
        "adres, kontakt i osoba powstają jako bieżące",
    );
    expect::refused(
        room.execute(
            "INSERT INTO klienci_kartoteka_kontakty (klient_id, rodzaj, wartosc, od, do, zrodlo_odnosnik)
            VALUES ('nowak', 'telefon', '+48600100200', '2026-03-02', '2026-04-01', 'teczka/umowa')",
            &[],
        ),
        "adres, kontakt i osoba powstają jako bieżące",
    );

    add_phone(&room, FIRST_PHONE, "2026-03-02");
    let phone_id = newest_contact_id(&room);
    let phone_end = |end: &str| ZakonczKontaktKlientaParams {
        r#do: Some(end.to_string()),
        id: phone_id,
    };
    expect::refused(
        klienci_kartoteka::zakoncz_kontakt_klienta(&room, &phone_end("2026-01-01")),
        "koniec_nie_przed_poczatkiem",
    );
    klienci_kartoteka::zakoncz_kontakt_klienta(&room, &phone_end("2026-05-01"))
        .expect("end the phone");
    expect::refused(
        klienci_kartoteka::zakoncz_kontakt_klienta(&room, &phone_end("2026-06-01")),
        "adresu, kontaktu ani osoby się nie zmienia",
    );
    expect::refused(
        room.execute(
            &format!(
                "UPDATE klienci_kartoteka_kontakty SET wartosc = '+48111' WHERE id = {phone_id}"
            ),
            &[],
        ),
        "adresu, kontaktu ani osoby się nie zmienia",
    );

    add_address(&room, &address());
    let addresses = klienci_kartoteka::adresy_klienta(
        &room,
        &AdresyKlientaParams {
            klient_id: "nowak".to_string(),
        },
    )
    .expect("read the addresses");
    let address_id = addresses[0].id;
    klienci_kartoteka::zakoncz_adres_klienta(
        &room,
        &ZakonczAdresKlientaParams {
            r#do: Some("2026-05-01".to_string()),
            id: address_id,
        },
    )
    .expect("end the address");
    expect::refused(
        room.execute(
            &format!(
                "UPDATE klienci_kartoteka_adresy SET adres = 'ul. Inna 1' WHERE id = {address_id}"
            ),
            &[],
        ),
        "adresu, kontaktu ani osoby się nie zmienia",
    );
}

/// Osoby: ma je tylko podmiot, a działający za podmiot ma podstawę umocowania
#[test]
fn persons_of_entity() {
    let room = new_room();
    add_client(&room, &client("nowak"));
    add_company(&room);
    let person = WpiszOsobeKlientaParams {
        klient_id: "lesna".to_string(),
        funkcja: OsobyFunkcja::Kontaktowa,
        imiona: "Ewa".to_string(),
        nazwisko: "Lis".to_string(),
        kontakt: Some("ewa.lis@lesna.example".to_string()),
        podstawa_odnosnik: None,
        od: "2026-04-01".to_string(),
        zrodlo_odnosnik: "teczka/umowa".to_string(),
    };

    expect::refused(
        klienci_kartoteka::wpisz_osobe_klienta(
            &room,
            &WpiszOsobeKlientaParams {
                klient_id: "nowak".to_string(),
                ..person.clone()
            },
        ),
        "osoby działające za klienta i osoby kontaktowe ma tylko podmiot",
    );
    expect::refused(
        klienci_kartoteka::wpisz_osobe_klienta(
            &room,
            &WpiszOsobeKlientaParams {
                funkcja: OsobyFunkcja::DzialaZaPodmiot,
                ..person.clone()
            },
        ),
        "dzialajacy_za_podmiot_ma_podstawe",
    );
    expect::refused(
        room.execute(
            "INSERT INTO klienci_kartoteka_osoby (klient_id, funkcja, imiona, nazwisko, od, do,
              zrodlo_odnosnik)
            VALUES ('lesna', 'kontaktowa', 'Ewa', 'Lis', '2026-04-01', '2026-05-01', 'teczka/umowa')",
            &[],
        ),
        "adres, kontakt i osoba powstają jako bieżące",
    );

    klienci_kartoteka::wpisz_osobe_klienta(&room, &person).expect("add the contact person");
    let persons = klienci_kartoteka::osoby_klienta(
        &room,
        &OsobyKlientaParams {
            klient_id: "lesna".to_string(),
        },
    )
    .expect("read the persons");
    let person_id = persons[0].id;
    klienci_kartoteka::zakoncz_osobe_klienta(
        &room,
        &ZakonczOsobeKlientaParams {
            r#do: Some("2026-09-01".to_string()),
            id: person_id,
        },
    )
    .expect("end the person");
    expect::refused(
        room.execute(
            &format!(
                "UPDATE klienci_kartoteka_osoby SET nazwisko = 'Leśna' WHERE id = {person_id}"
            ),
            &[],
        ),
        "adresu, kontaktu ani osoby się nie zmienia",
    );
}

// Contracts and their states ----------------------------------------------------------------------

/// Umowy: dział, teczka i nieruchomość pasują do rodzaju umowy
#[test]
fn contract_kind() {
    let room = new_room();
    add_client(&room, &client("nowak"));

    expect::refused(
        klienci_kartoteka::wpisz_umowe_klienta(
            &room,
            &WpiszUmoweKlientaParams {
                dzial: UmowyDzial::Najem,
                ..contract()
            },
        ),
        "dzial_pasuje_do_rodzaju",
    );
    expect::refused(
        klienci_kartoteka::wpisz_umowe_klienta(
            &room,
            &WpiszUmoweKlientaParams {
                typ_teczki: UmowyTypTeczki::Najem,
                ..contract()
            },
        ),
        "typ_teczki_pasuje_do_rodzaju",
    );
    expect::refused(
        klienci_kartoteka::wpisz_umowe_klienta(
            &room,
            &WpiszUmoweKlientaParams {
                nieruchomosc_adres: None,
                ..contract()
            },
        ),
        "nieruchomosc_tylko_przy_umowie_o_nieruchomosc",
    );
    expect::refused(
        klienci_kartoteka::wpisz_umowe_klienta(
            &room,
            &WpiszUmoweKlientaParams {
                rodzaj: UmowyRodzaj::PosrednictwoKupna,
                typ_teczki: UmowyTypTeczki::Poszukiwanie,
                numer_kw: None,
                ..contract()
            },
        ),
        "nieruchomosc_tylko_przy_umowie_o_nieruchomosc",
    );
    expect::refused(
        klienci_kartoteka::wpisz_umowe_klienta(
            &room,
            &WpiszUmoweKlientaParams {
                rodzaj: UmowyRodzaj::ZarzadzanieNajemDlugoterminowy,
                dzial: UmowyDzial::Najem,
                typ_teczki: UmowyTypTeczki::Najem,
                obie_strony: true,
                ..contract()
            },
        ),
        "obie_strony_tylko_przy_posrednictwie",
    );
    expect::refused(
        klienci_kartoteka::wpisz_umowe_klienta(
            &room,
            &WpiszUmoweKlientaParams {
                koniec: Some("2026-01-01".to_string()),
                ..contract()
            },
        ),
        "koniec_nie_przed_zawarciem",
    );

    add_contract(
        &room,
        &WpiszUmoweKlientaParams {
            id: "sprzedaz".to_string(),
            obie_strony: true,
            koniec: Some("2026-09-02".to_string()),
            ..contract()
        },
    );
    add_contract(
        &room,
        &WpiszUmoweKlientaParams {
            id: "kupno".to_string(),
            rodzaj: UmowyRodzaj::PosrednictwoKupna,
            typ_teczki: UmowyTypTeczki::Poszukiwanie,
            nieruchomosc_adres: None,
            numer_kw: None,
            ..contract()
        },
    );
    add_contract(
        &room,
        &WpiszUmoweKlientaParams {
            id: "lokal".to_string(),
            rodzaj: UmowyRodzaj::ZarzadzanieNajemKrotkoterminowy,
            dzial: UmowyDzial::Najem,
            typ_teczki: UmowyTypTeczki::LokalKrotkoterminowy,
            konsument: false,
            ..contract()
        },
    );

    let contracts = klienci_kartoteka::umowy_klienta(
        &room,
        &UmowyKlientaParams {
            klient_id: "nowak".to_string(),
        },
    )
    .expect("read the contracts");
    assert_eq!(contracts.len(), 3);
}

/// Umowy: konsumentem jest tylko osoba fizyczna, a umowy się nie zmienia
#[test]
fn contract_consumer_and_unchanged() {
    let room = new_room();
    add_company(&room);

    expect::refused(
        klienci_kartoteka::wpisz_umowe_klienta(
            &room,
            &WpiszUmoweKlientaParams {
                klient_id: "lesna".to_string(),
                ..contract()
            },
        ),
        "umowę jako konsument zawiera tylko osoba fizyczna",
    );
    add_contract(
        &room,
        &WpiszUmoweKlientaParams {
            klient_id: "lesna".to_string(),
            konsument: false,
            ..contract()
        },
    );

    expect::refused(
        room.execute(
            "UPDATE klienci_kartoteka_umowy SET obie_strony = 1 WHERE id = 'umowa'",
            &[],
        ),
        UNCHANGED,
    );
    expect::refused(
        room.execute(
            "UPDATE klienci_kartoteka_wpisy SET rodzaj_podmiotu = 'osoba_fizyczna'",
            &[],
        ),
        UNCHANGED,
    );
}

/// Stany umowy: od „obowiązuje” do jednego stanu końcowego, który jest ostateczny
#[test]
fn contract_states_forward() {
    let room = new_room();
    add_client(&room, &client("nowak"));
    add_contract(&room, &contract());

    expect::refused(
        klienci_kartoteka::zmien_stan_umowy_klienta(
            &room,
            &ZmienStanUmowyKlientaParams {
                stan: UmowyStanyStan::Obowiazuje,
                ..state_change()
            },
        ),
        "stan umowy przechodzi tylko naprzód",
    );
    klienci_kartoteka::zmien_stan_umowy_klienta(
        &room,
        &ZmienStanUmowyKlientaParams {
            stan: UmowyStanyStan::Odstapiono,
            od: "2026-03-10".to_string(),
            ..state_change()
        },
    )
    .expect("withdraw from the contract");
    expect::refused(
        klienci_kartoteka::zmien_stan_umowy_klienta(
            &room,
            &ZmienStanUmowyKlientaParams {
                stan: UmowyStanyStan::Wykonana,
                ..state_change()
            },
        ),
        "stan umowy przechodzi tylko naprzód",
    );

    // A contract written past the operation, without its first state.
    add_client(
        &room,
        &WpiszKlientaParams {
            pesel: Some("85020254321".to_string()),
            nazwisko: Some("Lis".to_string()),
            ..client("lis")
        },
    );
    room.execute(
        "INSERT INTO klienci_kartoteka_umowy (id, klient_id, rodzaj, dzial, nieruchomosc_adres,
          teczka_odnosnik, typ_teczki, oznaczenie, umowa_odnosnik, podpisal_za_biuro, zawarto,
          konsument, obie_strony, wpisal, wpisano)
        VALUES ('bez-stanu', 'lis', 'posrednictwo_sprzedazy', 'posrednictwo', 'ul. Polna 7',
          'teczka', 'oferta', 'Oferta 1', 'teczka/umowa', 'posrednik', '2026-03-02', 0, 0,
          'kierownik', '2026-03-02')",
        &[],
    )
    .expect("write a contract without its first state");
    expect::refused(
        klienci_kartoteka::zmien_stan_umowy_klienta(
            &room,
            &ZmienStanUmowyKlientaParams {
                umowa_id: "bez-stanu".to_string(),
                stan: UmowyStanyStan::Wykonana,
                ..state_change()
            },
        ),
        "stan umowy przechodzi tylko naprzód",
    );
    klienci_kartoteka::zmien_stan_umowy_klienta(
        &room,
        &ZmienStanUmowyKlientaParams {
            umowa_id: "bez-stanu".to_string(),
            stan: UmowyStanyStan::Obowiazuje,
            od: "2026-03-02".to_string(),
            ..state_change()
        },
    )
    .expect("record the first state");
}

/// Stany umowy: stan nie zaczyna się przed zawarciem ani przed poprzednim stanem
#[test]
fn contract_states_dates() {
    let room = new_room();
    add_client(&room, &client("nowak"));
    add_contract(&room, &contract());

    expect::refused(
        klienci_kartoteka::zmien_stan_umowy_klienta(
            &room,
            &ZmienStanUmowyKlientaParams {
                od: "2026-02-01".to_string(),
                ..state_change()
            },
        ),
        "stan umowy nie zaczyna się przed jej zawarciem",
    );
    klienci_kartoteka::zmien_stan_umowy_klienta(
        &room,
        &ZmienStanUmowyKlientaParams {
            od: "2026-03-02".to_string(),
            ..state_change()
        },
    )
    .expect("terminate the contract");
    expect::refused(
        room.execute(
            "UPDATE klienci_kartoteka_umowy_stany SET od = '2026-04-01' WHERE umowa_id = 'umowa'",
            &[],
        ),
        UNCHANGED,
    );
}

/// Informacja o kartotece wskazuje wysyłkę w teczce i się nie zmienia
#[test]
fn information_unchanged() {
    let room = new_room();
    full_file(&room);

    // A writer outside the client can leave out the shipment that `wpisz_informacje_klienta`
    // requires. The table refuses the information without one.
    expect::refused(
        room.execute(
            "INSERT INTO klienci_kartoteka_informacje (klient_id, poinformowano, sposob)
            VALUES ('nowak', '2026-03-02T10:00:00Z', 'email')",
            &[],
        ),
        "NOT NULL constraint failed: klienci_kartoteka_informacje.wysylka_odnosnik",
    );
    expect::refused(
        room.execute(
            "UPDATE klienci_kartoteka_informacje SET sposob = 'list'",
            &[],
        ),
        UNCHANGED,
    );

    let information = klienci_kartoteka::informacje_klienta(
        &room,
        &InformacjeKlientaParams {
            klient_id: "nowak".to_string(),
        },
    )
    .expect("read the information about the file");
    assert_eq!(information[0].sposob, InformacjeSposob::Email);
}

// R9 and replacement ------------------------------------------------------------------------------

fn tables_of(room: &MemoryRoom) -> Vec<String> {
    let rows = room
        .execute(
            r"SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'klienci\_%' ESCAPE '\'",
            &[],
        )
        .expect("read the tables");

    rows.iter()
        .map(|row| row.get("name").expect("a table has a name"))
        .collect()
}

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

/// R9: wierszy kartoteki się nie usuwa ani nie zastępuje przez REPLACE albo upsert
#[test]
fn no_deletion_or_replacement() {
    let room = new_room();
    full_file(&room);

    let tables = tables_of(&room);
    assert_eq!(tables.len(), 8, "{tables:?}");

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
}
