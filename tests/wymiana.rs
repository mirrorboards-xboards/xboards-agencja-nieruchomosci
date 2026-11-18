//! Scenarios of the exchange room of an office and a notary's office (spec/cases/wymiana).
//!
//! The exchange room has no core of a case file, so its room is the room of `cases::wymiana`
//! alone. Each scenario starts from an empty room and checks one rule the database keeps: a write
//! the rule allows and a write it refuses. The first scenario follows a whole sale, from the
//! opening of the room to its closing, as the profile of the room describes it.

use xboards_agencja_nieruchomosci::cases::wymiana::{
    self, wymiana_akt, wymiana_dane, wymiana_dokumenty, wymiana_metryka, wymiana_wpisy,
};
use xboards_agencja_nieruchomosci::xmeta_room::{
    FromSqlValue, MemoryRoom, MetaRoom, SqlValue, Statement,
};

use wymiana_akt::{
    OglosStanTerminuParams, PierwokupyWynik, PodajInformacjePoAkcieParams, PodajPierwokupParams,
    Stan as StanTerminu, Strona as StronaAktu, TerminyCzynnosc, TerminyStanyStan,
    ZaproponujTerminAktuParams,
};
use wymiana_dane::{
    OsobyRodzajPodmiotu, OsobyUdzial, PodajRachunekParams, PrzekazDaneNieruchomosciParams,
    PrzekazDaneOsobyParams, PrzekazKredytParams, RachunkiCel, Strona as StronaDanych,
};
use wymiana_dokumenty::{
    DopiszPozycjeListyParams, ListaPostac, PodajKosztyAktuParams, PodajListeDokumentowParams,
    PrzekazDokumentZListyParams, PrzekazPelnomocnictwoParams, PrzekazProjektAktuParams,
    PrzekazUwagiDoProjektuParams, PrzekazZgodeStronParams, Strona as StronaDokumentu,
};
use wymiana_metryka::{
    OtworzPokojWymianyParams, OtworzPonowniePokojWymianyParams, PodajSygnatureKancelariiParams,
    Stan as StanPokoju, StronyRola, WpisCzynnosc, WpiszStroneAktuParams, ZamknieciaRodzaj,
    ZamknieciaStronaPokoju, ZamknijPokojWymianyParams,
};
use wymiana_wpisy::{WpisParams, WpisyStrona, WpiszWiadomoscParams};

use crate::expect;

fn new_room() -> MemoryRoom {
    wymiana::open_in_memory().expect("open the exchange room")
}

/// A text value to bind in raw SQL.
fn text(value: &str) -> SqlValue {
    SqlValue::Text(value.to_string())
}

// Building blocks of an exchange ------------------------------------------------------------------

/// Who writes the entries of the office.
const OFFICE_AUTHOR: &str = "tozsamosc-koordynatora";
/// Who writes the entries of the notary's office.
const NOTARY_AUTHOR: &str = "tozsamosc-sekretarza";

/// The file most documents of the scenarios carry.
const FILE_PATH: &str = "pliki/dokument.pdf";
const FILE_DIGEST: &str = "sha256-dokumentu";

/// The author of an entry about the date of the act written by a side.
fn act_author(strona: StronaAktu) -> String {
    let author = match strona {
        StronaAktu::Biuro => OFFICE_AUTHOR,
        StronaAktu::Kancelaria => NOTARY_AUTHOR,
    };

    author.to_string()
}

/// The office opens the room and names the parties: it represents the seller and the buyer.
fn open_exchange(room: &MemoryRoom) {
    let record = OtworzPokojWymianyParams {
        oznaczenie: "Transakcja 2026/017, ul. Polna 7 m. 12".to_string(),
        transakcja_odnosnik: "pokoj-transakcji-017".to_string(),
        czynnosc: WpisCzynnosc::Przenoszaca,
        kancelaria_nazwa: "Kancelaria Notarialna przy ul. Długiej".to_string(),
        kancelaria_adres: "ul. Długa 3, Kraków".to_string(),
        notariusz: "Jan Nowak".to_string(),
        nieruchomosc_adres: "ul. Polna 7 m. 12, Kraków".to_string(),
        numer_kw: Some("KR1P/00012345/6".to_string()),
        otwarto: "2026-11-02T09:00:00Z".to_string(),
    };
    wymiana_metryka::otworz_pokoj_wymiany(room, &record).expect("open the exchange room");

    let parties = [
        (
            "sprzedajaca",
            "Anna Kowalska",
            StronyRola::Sprzedajacy,
            true,
        ),
        ("kupujacy", "Piotr Zieliński", StronyRola::Kupujacy, true),
        (
            "malzonka-kupujacego",
            "Ewa Zielińska",
            StronyRola::Kupujacy,
            false,
        ),
    ];
    for (party_id, nazwa, rola, reprezentowana_przez_biuro) in parties {
        let party = WpiszStroneAktuParams {
            id: party_id.to_string(),
            nazwa: nazwa.to_string(),
            rola,
            reprezentowana_przez_biuro,
        };

        wymiana_metryka::wpisz_strone_aktu(room, &party).expect("name a party of the act");
    }
}

fn give_power(room: &MemoryRoom, wpis_id: &str, strona_id: &str) {
    let params = PrzekazPelnomocnictwoParams {
        wpis_id: wpis_id.to_string(),
        strona: StronaDokumentu::Biuro,
        autor: OFFICE_AUTHOR.to_string(),
        chwila: "2026-11-02T10:00:00Z".to_string(),
        poprawia_id: None,
        tresc: None,
        sciezka: FILE_PATH.to_string(),
        skrot: FILE_DIGEST.to_string(),
        strona_id: Some(strona_id.to_string()),
        wystawca: Some("Anna Kowalska".to_string()),
        data_wystawienia: Some("2026-11-01".to_string()),
        waznosc_do: None,
        zakres: Some(
            "uzgadnianie terminu aktu i przekazywanie dokumentów notariuszowi".to_string(),
        ),
    };

    wymiana_dokumenty::przekaz_pelnomocnictwo(room, &params).expect("give the power of attorney");
}

/// The notary's office lists two documents under the entry „lista”.
fn give_list(room: &MemoryRoom) {
    let list = PodajListeDokumentowParams {
        id: "lista".to_string(),
        strona: StronaDokumentu::Kancelaria,
        autor: NOTARY_AUTHOR.to_string(),
        chwila: "2026-11-03T09:00:00Z".to_string(),
        poprawia_id: None,
        tresc: None,
    };
    wymiana_dokumenty::podaj_liste_dokumentow(room, &list).expect("give the list of documents");

    let land_register_copy = DopiszPozycjeListyParams {
        wpis_id: "lista".to_string(),
        pozycja_id: "odpis-kw".to_string(),
        nazwa: "Odpis zupełny księgi wieczystej".to_string(),
        od_strony_id: None,
        postac: ListaPostac::Odpis,
        swiezosc_dni: Some(30),
        do_kiedy: None,
    };
    wymiana_dokumenty::dopisz_pozycje_listy(room, &land_register_copy)
        .expect("list the copy of the land register");

    let cooperative_certificate = DopiszPozycjeListyParams {
        wpis_id: "lista".to_string(),
        pozycja_id: "zaswiadczenie-spoldzielni".to_string(),
        nazwa: "Zaświadczenie spółdzielni o braku zaległości".to_string(),
        od_strony_id: Some("sprzedajaca".to_string()),
        postac: ListaPostac::Oryginal,
        swiezosc_dni: Some(30),
        do_kiedy: Some("2026-11-20".to_string()),
    };
    wymiana_dokumenty::dopisz_pozycje_listy(room, &cooperative_certificate)
        .expect("list the certificate of the cooperative");
}

/// The default parameters of the office's certificate of the cooperative from the list.
fn document() -> PrzekazDokumentZListyParams {
    PrzekazDokumentZListyParams {
        wpis_id: "zaswiadczenie".to_string(),
        strona: StronaDokumentu::Biuro,
        autor: OFFICE_AUTHOR.to_string(),
        chwila: "2026-11-04T09:00:00Z".to_string(),
        odpowiada_na_id: None,
        poprawia_id: None,
        tresc: None,
        sciezka: FILE_PATH.to_string(),
        skrot: FILE_DIGEST.to_string(),
        pozycja_id: Some("zaswiadczenie-spoldzielni".to_string()),
        strona_id: Some("sprzedajaca".to_string()),
        wystawca: Some("Spółdzielnia Mieszkaniowa Polna".to_string()),
        data_wystawienia: Some("2026-11-03".to_string()),
        waznosc_do: Some("2026-12-03".to_string()),
        oryginal_w_dniu_aktu: Some(true),
        kto_przyniesie: Some("Anna Kowalska".to_string()),
    }
}

fn give_document(room: &MemoryRoom, params: &PrzekazDokumentZListyParams) {
    wymiana_dokumenty::przekaz_dokument_z_listy(room, params).expect("give the document");
}

fn give_draft(room: &MemoryRoom, wpis_id: &str, numer_wersji: i64, ostateczna: bool) {
    let params = PrzekazProjektAktuParams {
        wpis_id: wpis_id.to_string(),
        strona: StronaDokumentu::Kancelaria,
        autor: NOTARY_AUTHOR.to_string(),
        chwila: "2026-11-10T09:00:00Z".to_string(),
        odpowiada_na_id: None,
        tresc: None,
        sciezka: format!("pliki/projekt-{numer_wersji}.pdf"),
        skrot: format!("sha256-projektu-{numer_wersji}"),
        numer_wersji: Some(numer_wersji),
        ostateczna: Some(ostateczna),
    };

    wymiana_dokumenty::przekaz_projekt_aktu(room, &params).expect("give the draft of the act");
}

/// The default parameters of the data of the seller as a party of the act.
fn person() -> PrzekazDaneOsobyParams {
    PrzekazDaneOsobyParams {
        wpis_id: "dane-sprzedajacej".to_string(),
        strona: StronaDanych::Biuro,
        autor: OFFICE_AUTHOR.to_string(),
        chwila: "2026-11-04T10:00:00Z".to_string(),
        odpowiada_na_id: None,
        poprawia_id: None,
        strona_id: "sprzedajaca".to_string(),
        udzial: OsobyUdzial::Strona,
        rodzaj_podmiotu: OsobyRodzajPodmiotu::OsobaFizyczna,
        imiona: Some("Anna Maria".to_string()),
        nazwisko: Some("Kowalska".to_string()),
        imiona_rodzicow: Some("Jan, Maria".to_string()),
        miejsce_zamieszkania: Some("Kraków".to_string()),
        nazwa: None,
        siedziba: None,
        obywatelstwo: None,
        stan_cywilny: Some("rozwiedziona".to_string()),
        zrodlo_dokument_odnosnik: "pokoj-transakcji-017/dowod-sprzedajacej".to_string(),
    }
}

fn give_person(room: &MemoryRoom, params: &PrzekazDaneOsobyParams) {
    wymiana_dane::przekaz_dane_osoby(room, params).expect("give the data of a person");
}

fn propose_term(room: &MemoryRoom, termin_id: &str, wpis_id: &str, strona: StronaAktu) {
    let params = ZaproponujTerminAktuParams {
        wpis_id: wpis_id.to_string(),
        strona,
        autor: act_author(strona),
        chwila: "2026-11-11T09:00:00Z".to_string(),
        odpowiada_na_id: None,
        tresc: None,
        id: termin_id.to_string(),
        dzien: "2026-11-25".to_string(),
        godzina: "10:30".to_string(),
        miejsce: "ul. Długa 3, Kraków".to_string(),
        czynnosc: TerminyCzynnosc::Przenoszaca,
    };

    wymiana_akt::zaproponuj_termin_aktu(room, &params).expect("propose a date of the act");
}

/// The default parameters of an announcement of a state of a date, by a side.
fn announcement(
    wpis_id: &str,
    termin_id: &str,
    stan: TerminyStanyStan,
    strona: StronaAktu,
) -> OglosStanTerminuParams {
    OglosStanTerminuParams {
        wpis_id: wpis_id.to_string(),
        strona,
        autor: act_author(strona),
        chwila: "2026-11-12T09:00:00Z".to_string(),
        odpowiada_na_id: None,
        tresc: None,
        termin_id: termin_id.to_string(),
        stan,
        decyzja_odnosnik: None,
    }
}

fn announce(room: &MemoryRoom, params: &OglosStanTerminuParams) {
    wymiana_akt::oglos_stan_terminu(room, params).expect("announce the state of the date");
}

fn confirm_by_both(room: &MemoryRoom, termin_id: &str) {
    let by_office = OglosStanTerminuParams {
        decyzja_odnosnik: Some("pokoj-transakcji-017/decyzja-termin".to_string()),
        ..announcement(
            &format!("{termin_id}-biuro"),
            termin_id,
            TerminyStanyStan::Potwierdzony,
            StronaAktu::Biuro,
        )
    };
    announce(room, &by_office);

    let by_notary = announcement(
        &format!("{termin_id}-kancelaria"),
        termin_id,
        TerminyStanyStan::Potwierdzony,
        StronaAktu::Kancelaria,
    );
    announce(room, &by_notary);
}

fn close(room: &MemoryRoom, strona_pokoju: ZamknieciaStronaPokoju) {
    let params = ZamknijPokojWymianyParams {
        strona_pokoju,
        chwila: "2026-12-01T09:00:00Z".to_string(),
    };

    wymiana_metryka::zamknij_pokoj_wymiany(room, &params).expect("close the room");
}

fn reopen(room: &MemoryRoom, strona_pokoju: ZamknieciaStronaPokoju) {
    let params = OtworzPonowniePokojWymianyParams {
        strona_pokoju,
        chwila: "2027-03-01T09:00:00Z".to_string(),
    };

    wymiana_metryka::otworz_ponownie_pokoj_wymiany(room, &params).expect("reopen the room");
}

fn state(room: &MemoryRoom) -> StanPokoju {
    let room_state = wymiana_metryka::stan_pokoju(room)
        .expect("read the state of the room")
        .expect("the room has a state");

    room_state.stan
}

/// A sale at a notary's office, written into every table of the room.
fn full_exchange(room: &MemoryRoom) {
    open_exchange(room);
    wymiana_metryka::podaj_sygnature_kancelarii(
        room,
        &PodajSygnatureKancelariiParams {
            sygnatura_kancelarii: "KN 1234/2026".to_string(),
        },
    )
    .expect("give the reference of the notary's office");

    // The office: powers of attorney, the documents from the notary's list and the data of the act.
    give_power(room, "pelnomocnictwo-sprzedajacej", "sprzedajaca");
    give_power(room, "pelnomocnictwo-kupujacego", "kupujacy");
    give_list(room);
    give_document(room, &document());
    wymiana_dokumenty::przekaz_zgode_stron(
        room,
        &PrzekazZgodeStronParams {
            wpis_id: "zgoda-na-projekt".to_string(),
            strona: StronaDokumentu::Biuro,
            autor: OFFICE_AUTHOR.to_string(),
            chwila: "2026-11-04T09:30:00Z".to_string(),
            poprawia_id: None,
            tresc: None,
            sciezka: FILE_PATH.to_string(),
            skrot: FILE_DIGEST.to_string(),
            strona_id: Some("kupujacy".to_string()),
            data_wystawienia: Some("2026-11-04".to_string()),
            zakres: Some("biuro dostaje projekt aktu i informację po akcie".to_string()),
        },
    )
    .expect("give the consent of the parties");
    give_person(room, &person());
    wymiana_dane::przekaz_dane_nieruchomosci(
        room,
        &PrzekazDaneNieruchomosciParams {
            wpis_id: "dane-lokalu".to_string(),
            strona: StronaDanych::Biuro,
            autor: OFFICE_AUTHOR.to_string(),
            chwila: "2026-11-04T10:30:00Z".to_string(),
            odpowiada_na_id: None,
            poprawia_id: None,
            strona_id: "sprzedajaca".to_string(),
            numer_kw: Some("KR1P/00012345/6".to_string()),
            oznaczenie: "lokal mieszkalny nr 12 w budynku przy ul. Polnej 7, pow. 54,20 m²"
                .to_string(),
            dzialki: None,
            zrodlo: "odpis zupełny KW z 2026-11-03".to_string(),
        },
    )
    .expect("give the data of the flat");
    wymiana_dane::przekaz_kredyt(
        room,
        &PrzekazKredytParams {
            wpis_id: "kredyt".to_string(),
            strona: StronaDanych::Biuro,
            autor: OFFICE_AUTHOR.to_string(),
            chwila: "2026-11-04T11:00:00Z".to_string(),
            odpowiada_na_id: None,
            poprawia_id: None,
            strona_id: "kupujacy".to_string(),
            bank: "Bank Polski S.A.".to_string(),
            kwota: 45_000_000,
            warunki_wyplaty: Some("po przedłożeniu wypisu aktu i wniosku do księgi".to_string()),
        },
    )
    .expect("give the credit");
    wymiana_dane::podaj_rachunek(
        room,
        &PodajRachunekParams {
            wpis_id: "rachunek-ceny".to_string(),
            strona: StronaDanych::Biuro,
            autor: OFFICE_AUTHOR.to_string(),
            chwila: "2026-11-05T09:00:00Z".to_string(),
            odpowiada_na_id: None,
            poprawia_id: None,
            tresc: None,
            numer: "PL61109010140000071219812874".to_string(),
            posiadacz: "Anna Kowalska".to_string(),
            cel: RachunkiCel::ZaplataCeny,
            decyzja_odnosnik: Some("pokoj-transakcji-017/decyzja-rachunek".to_string()),
        },
    )
    .expect("give the account for the price");

    // The notary's office: costs, the draft and the comments of the parties on it.
    wymiana_dokumenty::podaj_koszty_aktu(
        room,
        &PodajKosztyAktuParams {
            wpis_id: "koszty".to_string(),
            strona: StronaDokumentu::Kancelaria,
            autor: NOTARY_AUTHOR.to_string(),
            chwila: "2026-11-06T09:00:00Z".to_string(),
            poprawia_id: None,
            tresc: Some("Taksa 1 010 zł netto, PCC 2%, opłata sądowa 200 zł.".to_string()),
            sciezka: None,
            skrot: None,
        },
    )
    .expect("give the costs of the act");
    give_draft(room, "projekt-1", 1, false);
    wymiana_dokumenty::przekaz_uwagi_do_projektu(
        room,
        &PrzekazUwagiDoProjektuParams {
            wpis_id: "uwagi-1".to_string(),
            strona: StronaDokumentu::Biuro,
            autor: OFFICE_AUTHOR.to_string(),
            chwila: "2026-11-11T08:00:00Z".to_string(),
            projekt_id: "projekt-1".to_string(),
            tresc: Some("Kupujący prosi o wydanie lokalu do 2026-12-15.".to_string()),
            sciezka: None,
            skrot: None,
            decyzja_odnosnik: "pokoj-transakcji-017/decyzja-uwagi".to_string(),
        },
    )
    .expect("give the comments on the draft");
    give_draft(room, "projekt-2", 2, true);

    // The date of the act, confirmed by both sides; the act, the pre-emption and after the act.
    propose_term(room, "termin", "propozycja", StronaAktu::Kancelaria);
    confirm_by_both(room, "termin");
    wymiana_akt::podaj_pierwokup(
        room,
        &PodajPierwokupParams {
            wpis_id: "pierwokup".to_string(),
            strona: StronaAktu::Kancelaria,
            autor: NOTARY_AUTHOR.to_string(),
            chwila: "2026-11-20T09:00:00Z".to_string(),
            poprawia_id: None,
            tresc: None,
            uprawniony: "Gmina Miejska Kraków".to_string(),
            zawiadomienie_doreczono: Some("2026-10-20".to_string()),
            wynik: Some(PierwokupyWynik::TerminUplynal),
            data: Some("2026-11-20".to_string()),
        },
    )
    .expect("give the pre-emption");
    announce(
        room,
        &announcement(
            "odbyl-sie",
            "termin",
            TerminyStanyStan::OdbylSie,
            StronaAktu::Kancelaria,
        ),
    );
    wymiana_akt::podaj_informacje_po_akcie(
        room,
        &PodajInformacjePoAkcieParams {
            wpis_id: "po-akcie".to_string(),
            strona: StronaAktu::Kancelaria,
            autor: NOTARY_AUTHOR.to_string(),
            chwila: "2026-11-25T14:00:00Z".to_string(),
            poprawia_id: None,
            tresc: None,
            akt_sporzadzono: Some("2026-11-25".to_string()),
            wniosek_kw_zlozono: Some("2026-11-25".to_string()),
        },
    )
    .expect("give the information after the act");
    wymiana_wpisy::wpisz_wiadomosc(
        room,
        &WpiszWiadomoscParams {
            id: "pytanie".to_string(),
            chwila: "2026-11-26T09:00:00Z".to_string(),
            odpowiada_na_id: Some("po-akcie".to_string()),
            tresc: Some("Czy kancelaria może podać numer wniosku do księgi?".to_string()),
            ..message()
        },
    )
    .expect("write the question");
    close(room, ZamknieciaStronaPokoju::Biuro);
}

// The whole exchange ------------------------------------------------------------------------------

/// Przebieg wymiany: od otwarcia przez dokumenty, projekt i termin aktu do zamknięcia
#[test]
fn whole_exchange() {
    let room = new_room();

    full_exchange(&room);
    assert_eq!(state(&room), StanPokoju::Otwarty);
    close(&room, ZamknieciaStronaPokoju::Kancelaria);

    assert_eq!(state(&room), StanPokoju::Zamkniety);

    let record = wymiana_metryka::metryka(&room)
        .expect("read the record")
        .expect("the room has a record");
    assert_eq!(record.sygnatura_kancelarii.as_deref(), Some("KN 1234/2026"));

    let current_list =
        wymiana_dokumenty::biezaca_lista_dokumentow(&room).expect("read the current list");
    let item_ids: Vec<&str> = current_list
        .iter()
        .map(|row| row.pozycja_id.as_str())
        .collect();
    assert_eq!(item_ids, ["odpis-kw", "zaswiadczenie-spoldzielni"]);

    let drafts = wymiana_dokumenty::wersje_projektu_aktu(&room).expect("read the drafts");
    let draft_numbers: Vec<Option<i64>> = drafts.iter().map(|row| row.numer_wersji).collect();
    assert_eq!(draft_numbers, [Some(2), Some(1)]);

    let comments = wymiana_dokumenty::uwagi_do_projektu(&room).expect("read the comments");
    assert_eq!(comments[0].numer_wersji, Some(1));

    let terms = wymiana_akt::terminy_aktu(&room).expect("read the dates of the act");
    assert_eq!(terms[0].stan, StanTerminu::OdbylSie);

    let after_the_act =
        wymiana_akt::informacje_po_akcie(&room).expect("read the information after the act");
    assert_eq!(
        after_the_act[0].wniosek_kw_zlozono.as_deref(),
        Some("2026-11-25")
    );

    let entries = wymiana_wpisy::wpisy(&room).expect("read the entries");
    assert_eq!(entries.len(), 20);
}

// Metryka -----------------------------------------------------------------------------------------

/// Metryki pokoju się nie zmienia; kancelaria raz podaje swoją sygnaturę
#[test]
fn record_fixed() {
    let room = new_room();
    open_exchange(&room);
    let rule = "metryki pokoju się nie zmienia";

    expect::refused(
        room.execute(
            "UPDATE wymiana_metryka_wpis SET notariusz = 'Inny Notariusz'",
            &[],
        ),
        rule,
    );
    wymiana_metryka::podaj_sygnature_kancelarii(
        &room,
        &PodajSygnatureKancelariiParams {
            sygnatura_kancelarii: "KN 1234/2026".to_string(),
        },
    )
    .expect("give the reference of the notary's office");
    expect::refused(
        wymiana_metryka::podaj_sygnature_kancelarii(
            &room,
            &PodajSygnatureKancelariiParams {
                sygnatura_kancelarii: "KN 9999/2026".to_string(),
            },
        ),
        rule,
    );

    let record = wymiana_metryka::metryka(&room)
        .expect("read the record")
        .expect("the room has a record");
    assert_eq!(record.sygnatura_kancelarii.as_deref(), Some("KN 1234/2026"));
}

/// Pokój zamyka się zgodą obu stron i dopiero zgodą obu otwiera się ponownie
#[test]
fn closing_by_both_sides() {
    let room = new_room();
    let closing = |strona_pokoju: ZamknieciaStronaPokoju, chwila: &str| ZamknijPokojWymianyParams {
        strona_pokoju,
        chwila: chwila.to_string(),
    };

    expect::refused(
        wymiana_metryka::zamknij_pokoj_wymiany(
            &room,
            &closing(ZamknieciaStronaPokoju::Biuro, "2026-12-01"),
        ),
        "pokój bez metryki nie ma czego zamykać",
    );
    open_exchange(&room);

    close(&room, ZamknieciaStronaPokoju::Biuro);
    assert_eq!(state(&room), StanPokoju::Otwarty);
    expect::refused(
        wymiana_metryka::zamknij_pokoj_wymiany(
            &room,
            &closing(ZamknieciaStronaPokoju::Biuro, "2026-12-01"),
        ),
        "na przemian",
    );
    expect::refused(
        wymiana_metryka::otworz_ponownie_pokoj_wymiany(
            &room,
            &OtworzPonowniePokojWymianyParams {
                strona_pokoju: ZamknieciaStronaPokoju::Kancelaria,
                chwila: "2026-12-01".to_string(),
            },
        ),
        "ponownie otwiera tylko zamknięty",
    );

    close(&room, ZamknieciaStronaPokoju::Kancelaria);
    assert_eq!(state(&room), StanPokoju::Zamkniety);

    reopen(&room, ZamknieciaStronaPokoju::Kancelaria);
    assert_eq!(state(&room), StanPokoju::Zamkniety);
    expect::refused(
        wymiana_metryka::zamknij_pokoj_wymiany(
            &room,
            &closing(ZamknieciaStronaPokoju::Kancelaria, "2027-03-01"),
        ),
        "zamyka tylko otwarty",
    );
    reopen(&room, ZamknieciaStronaPokoju::Biuro);
    assert_eq!(state(&room), StanPokoju::Otwarty);

    let closings = wymiana_metryka::zamkniecia(&room).expect("read the closings");
    let closing_kinds: Vec<ZamknieciaRodzaj> = closings.iter().map(|row| row.rodzaj).collect();
    assert_eq!(
        closing_kinds,
        [
            ZamknieciaRodzaj::Zamkniecie,
            ZamknieciaRodzaj::Zamkniecie,
            ZamknieciaRodzaj::PonowneOtwarcie,
            ZamknieciaRodzaj::PonowneOtwarcie,
        ]
    );
}

// Entries -----------------------------------------------------------------------------------------

/// The default parameters of a message of the office.
fn message() -> WpiszWiadomoscParams {
    WpiszWiadomoscParams {
        id: "wiadomosc".to_string(),
        strona: WpisyStrona::Biuro,
        autor: OFFICE_AUTHOR.to_string(),
        chwila: "2026-11-03T09:00:00Z".to_string(),
        odpowiada_na_id: None,
        poprawia_id: None,
        tresc: Some(
            "Czy notariusz przyjmie zaświadczenie wystawione 20 dni przed aktem?".to_string(),
        ),
        sciezka: None,
        skrot: None,
    }
}

/// Wpis powstaje w pokoju z metryką i nie powstaje w zamkniętym pokoju
#[test]
fn entries_in_open_room() {
    let room = new_room();
    expect::refused(
        wymiana_wpisy::wpisz_wiadomosc(&room, &message()),
        "wpis powstaje dopiero w pokoju z metryką",
    );
    open_exchange(&room);

    wymiana_wpisy::wpisz_wiadomosc(&room, &message()).expect("write the message");
    close(&room, ZamknieciaStronaPokoju::Biuro);
    close(&room, ZamknieciaStronaPokoju::Kancelaria);
    let after_closing = WpiszWiadomoscParams {
        id: "po-zamknieciu".to_string(),
        ..message()
    };
    expect::refused(
        wymiana_wpisy::wpisz_wiadomosc(&room, &after_closing),
        "w zamkniętym pokoju nie powstaje nowy wpis",
    );

    reopen(&room, ZamknieciaStronaPokoju::Biuro);
    expect::refused(
        wymiana_wpisy::wpisz_wiadomosc(&room, &after_closing),
        "w zamkniętym pokoju nie powstaje nowy wpis",
    );
    reopen(&room, ZamknieciaStronaPokoju::Kancelaria);
    wymiana_wpisy::wpisz_wiadomosc(
        &room,
        &WpiszWiadomoscParams {
            id: "po-otwarciu".to_string(),
            ..message()
        },
    )
    .expect("write the message after reopening");
}

/// Lista, projekt, koszty, pierwokup i informacja po akcie pochodzą tylko od kancelarii
#[test]
fn entries_only_from_notary() {
    let room = new_room();
    open_exchange(&room);

    expect::refused(
        wymiana_dokumenty::podaj_liste_dokumentow(
            &room,
            &PodajListeDokumentowParams {
                id: "lista".to_string(),
                strona: StronaDokumentu::Biuro,
                autor: OFFICE_AUTHOR.to_string(),
                chwila: "2026-11-03".to_string(),
                poprawia_id: None,
                tresc: None,
            },
        ),
        "wpis_tylko_od_kancelarii",
    );
    expect::refused(
        wymiana_dokumenty::podaj_koszty_aktu(
            &room,
            &PodajKosztyAktuParams {
                wpis_id: "koszty".to_string(),
                strona: StronaDokumentu::Biuro,
                autor: OFFICE_AUTHOR.to_string(),
                chwila: "2026-11-03".to_string(),
                poprawia_id: None,
                tresc: Some("Taksa".to_string()),
                sciezka: None,
                skrot: None,
            },
        ),
        "wpis_tylko_od_kancelarii",
    );
    expect::refused(
        wymiana_dokumenty::przekaz_projekt_aktu(
            &room,
            &PrzekazProjektAktuParams {
                wpis_id: "projekt".to_string(),
                strona: StronaDokumentu::Biuro,
                autor: OFFICE_AUTHOR.to_string(),
                chwila: "2026-11-03".to_string(),
                odpowiada_na_id: None,
                tresc: None,
                sciezka: FILE_PATH.to_string(),
                skrot: FILE_DIGEST.to_string(),
                numer_wersji: Some(1),
                ostateczna: Some(false),
            },
        ),
        "wpis_tylko_od_kancelarii",
    );
    expect::refused(
        wymiana_akt::podaj_pierwokup(
            &room,
            &PodajPierwokupParams {
                wpis_id: "pierwokup".to_string(),
                strona: StronaAktu::Biuro,
                autor: OFFICE_AUTHOR.to_string(),
                chwila: "2026-11-03".to_string(),
                poprawia_id: None,
                tresc: None,
                uprawniony: "Gmina".to_string(),
                zawiadomienie_doreczono: None,
                wynik: None,
                data: None,
            },
        ),
        "wpis_tylko_od_kancelarii",
    );
    expect::refused(
        wymiana_akt::podaj_informacje_po_akcie(
            &room,
            &PodajInformacjePoAkcieParams {
                wpis_id: "po-akcie".to_string(),
                strona: StronaAktu::Biuro,
                autor: OFFICE_AUTHOR.to_string(),
                chwila: "2026-11-03".to_string(),
                poprawia_id: None,
                tresc: None,
                akt_sporzadzono: Some("2026-11-25".to_string()),
                wniosek_kw_zlozono: None,
            },
        ),
        "wpis_tylko_od_kancelarii",
    );
    give_list(&room);
    give_draft(&room, "projekt", 1, false);
}

/// Dokumenty, pełnomocnictwa, zgody, uwagi i dane stron pochodzą tylko od biura
#[test]
fn entries_only_from_office() {
    let room = new_room();
    open_exchange(&room);
    give_power(&room, "pelnomocnictwo", "sprzedajaca");
    give_list(&room);
    give_draft(&room, "projekt", 1, false);

    expect::refused(
        wymiana_dokumenty::przekaz_pelnomocnictwo(
            &room,
            &PrzekazPelnomocnictwoParams {
                wpis_id: "pelnomocnictwo-kancelarii".to_string(),
                strona: StronaDokumentu::Kancelaria,
                autor: NOTARY_AUTHOR.to_string(),
                chwila: "2026-11-03".to_string(),
                poprawia_id: None,
                tresc: None,
                sciezka: FILE_PATH.to_string(),
                skrot: FILE_DIGEST.to_string(),
                strona_id: Some("sprzedajaca".to_string()),
                wystawca: None,
                data_wystawienia: None,
                waznosc_do: None,
                zakres: Some("wszystko".to_string()),
            },
        ),
        "wpis_tylko_od_biura",
    );
    expect::refused(
        wymiana_dokumenty::przekaz_dokument_z_listy(
            &room,
            &PrzekazDokumentZListyParams {
                strona: StronaDokumentu::Kancelaria,
                autor: NOTARY_AUTHOR.to_string(),
                ..document()
            },
        ),
        "wpis_tylko_od_biura",
    );
    expect::refused(
        wymiana_dane::przekaz_dane_osoby(
            &room,
            &PrzekazDaneOsobyParams {
                strona: StronaDanych::Kancelaria,
                autor: NOTARY_AUTHOR.to_string(),
                ..person()
            },
        ),
        "wpis_tylko_od_biura",
    );
    expect::refused(
        wymiana_dokumenty::przekaz_uwagi_do_projektu(
            &room,
            &PrzekazUwagiDoProjektuParams {
                wpis_id: "uwagi".to_string(),
                strona: StronaDokumentu::Kancelaria,
                autor: NOTARY_AUTHOR.to_string(),
                chwila: "2026-11-11".to_string(),
                projekt_id: "projekt".to_string(),
                tresc: None,
                sciezka: None,
                skrot: None,
                decyzja_odnosnik: "decyzja".to_string(),
            },
        ),
        "wpis_tylko_od_biura",
    );
    give_document(&room, &document());
    give_person(&room, &person());
}

/// Plik ma skrót, dokument ma plik, a wiadomość ma treść
#[test]
fn entry_file_and_content() {
    let room = new_room();
    open_exchange(&room);

    expect::refused(
        wymiana_wpisy::wpisz_wiadomosc(
            &room,
            &WpiszWiadomoscParams {
                sciezka: Some("pliki/zdjecie.jpg".to_string()),
                ..message()
            },
        ),
        "plik_ze_skrotem",
    );
    expect::refused(
        wymiana_wpisy::wpisz_wiadomosc(
            &room,
            &WpiszWiadomoscParams {
                tresc: None,
                ..message()
            },
        ),
        "wiadomosc_z_trescia",
    );

    // A writer outside the client can leave out the file that `przekaz_pelnomocnictwo` requires.
    // The table refuses the document without one. Both statements of the operation run in one
    // write, so the refusal leaves nothing behind.
    let power_without_file = room.write(&[
        Statement {
            sql:
                "INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, sciezka, skrot)
            VALUES (:wpis_id, :strona, :autor, 'pelnomocnictwo', :chwila, :sciezka, :skrot)",
            parameters: vec![
                ("wpis_id", text("pelnomocnictwo")),
                ("strona", text("biuro")),
                ("autor", text(OFFICE_AUTHOR)),
                ("chwila", text("2026-11-02")),
                ("sciezka", SqlValue::Null),
                ("skrot", SqlValue::Null),
            ],
        },
        Statement {
            sql: "INSERT INTO wymiana_dokumenty_wpisy (wpis_id, rodzaj, strona_id, zakres)
            VALUES (:wpis_id, 'pelnomocnictwo', :strona_id, :zakres)",
            parameters: vec![
                ("wpis_id", text("pelnomocnictwo")),
                ("strona_id", text("sprzedajaca")),
                ("zakres", text("przekazywanie dokumentów")),
            ],
        },
    ]);
    expect::refused(power_without_file, "dokument_z_plikiem");

    wymiana_wpisy::wpisz_wiadomosc(
        &room,
        &WpiszWiadomoscParams {
            sciezka: Some("pliki/zdjecie.jpg".to_string()),
            skrot: Some("sha256-zdjecia".to_string()),
            ..message()
        },
    )
    .expect("write the message with a photo");
}

/// Poprawka wskazuje wcześniejszy wpis tego samego rodzaju i tej samej strony
#[test]
fn correction_same_kind() {
    let room = new_room();
    open_exchange(&room);
    give_power(&room, "pelnomocnictwo", "sprzedajaca");
    give_person(&room, &person());
    wymiana_wpisy::wpisz_wiadomosc(
        &room,
        &WpiszWiadomoscParams {
            id: "wiadomosc-kancelarii".to_string(),
            strona: WpisyStrona::Kancelaria,
            autor: NOTARY_AUTHOR.to_string(),
            ..message()
        },
    )
    .expect("write the message of the notary's office");
    let rule = "poprawka wskazuje wpis tego samego rodzaju i tej samej strony";

    expect::refused(
        wymiana_dane::przekaz_dane_osoby(
            &room,
            &PrzekazDaneOsobyParams {
                wpis_id: "poprawka".to_string(),
                poprawia_id: Some("pelnomocnictwo".to_string()),
                ..person()
            },
        ),
        rule,
    );
    expect::refused(
        wymiana_wpisy::wpisz_wiadomosc(
            &room,
            &WpiszWiadomoscParams {
                id: "poprawka".to_string(),
                poprawia_id: Some("wiadomosc-kancelarii".to_string()),
                ..message()
            },
        ),
        rule,
    );
    expect::refused(
        wymiana_wpisy::wpisz_wiadomosc(
            &room,
            &WpiszWiadomoscParams {
                id: "sama".to_string(),
                odpowiada_na_id: Some("sama".to_string()),
                ..message()
            },
        ),
        "odpowiedz_nie_wskazuje_siebie",
    );

    give_person(
        &room,
        &PrzekazDaneOsobyParams {
            wpis_id: "poprawka".to_string(),
            poprawia_id: Some("dane-sprzedajacej".to_string()),
            miejsce_zamieszkania: Some("Wieliczka".to_string()),
            ..person()
        },
    );

    let persons = wymiana_dane::dane_osob(&room).expect("read the data of persons");
    let places: Vec<Option<&str>> = persons
        .iter()
        .map(|row| row.miejsce_zamieszkania.as_deref())
        .collect();
    assert_eq!(places, [Some("Kraków"), Some("Wieliczka")]);
}

/// Szczegół dopisuje się tylko razem ze swoim wpisem i do wpisu swojego rodzaju
#[test]
fn detail_with_its_entry() {
    let room = new_room();
    open_exchange(&room);
    give_list(&room);
    wymiana_wpisy::wpisz_wiadomosc(&room, &message()).expect("write the message");
    let late_item = |wpis_id: &str| DopiszPozycjeListyParams {
        wpis_id: wpis_id.to_string(),
        pozycja_id: "wypis-z-rejestru".to_string(),
        nazwa: "Wypis z rejestru gruntów".to_string(),
        od_strony_id: None,
        postac: ListaPostac::Oryginal,
        swiezosc_dni: None,
        do_kiedy: None,
    };

    expect::refused(
        wymiana_dokumenty::dopisz_pozycje_listy(&room, &late_item("lista")),
        "pozycja listy dokumentów należy do najnowszego wpisu lista_dokumentow",
    );
    expect::refused(
        wymiana_dokumenty::dopisz_pozycje_listy(&room, &late_item("wiadomosc")),
        "pozycja listy dokumentów należy do najnowszego wpisu lista_dokumentow",
    );
    expect::refused(
        room.execute(
            "INSERT INTO wymiana_dane_rachunki (wpis_id, numer, posiadacz, cel, decyzja_odnosnik)
            VALUES ('wiadomosc', 'PL1', 'Anna', 'inny', 'decyzja')",
            &[],
        ),
        "rachunek należy do najnowszego wpisu rachunek",
    );
    expect::refused(
        room.execute(
            "INSERT INTO wymiana_dokumenty_wpisy (wpis_id, rodzaj) VALUES ('wiadomosc', 'koszty')",
            &[],
        ),
        "dokument należy do najnowszego wpisu tego samego rodzaju",
    );
}

// Documents ---------------------------------------------------------------------------------------

/// Dokument biura wypełnia pozycję z listy kancelarii
#[test]
fn document_from_list() {
    let room = new_room();
    open_exchange(&room);
    give_power(&room, "pelnomocnictwo", "sprzedajaca");

    expect::refused(
        wymiana_dokumenty::przekaz_dokument_z_listy(&room, &document()),
        "dokument wypełnia pozycję z listy dokumentów kancelarii",
    );
    give_list(&room);
    expect::refused(
        wymiana_dokumenty::przekaz_dokument_z_listy(
            &room,
            &PrzekazDokumentZListyParams {
                pozycja_id: Some("akt-notarialny-nabycia".to_string()),
                ..document()
            },
        ),
        "dokument wypełnia pozycję z listy dokumentów kancelarii",
    );
    give_document(&room, &document());
}

/// Pisanie bez pełnomocnictwa: dokument, zgoda i dane strony wymagają jej pełnomocnictwa
#[test]
fn writing_needs_power_of_attorney() {
    let room = new_room();
    open_exchange(&room);
    give_list(&room);
    let document_rule =
        "dokument biura dla strony wymaga wpisanego wcześniej pełnomocnictwa tej strony";
    let data_rule = "dane strony wymagają wpisanego wcześniej pełnomocnictwa tej strony";
    let consent = PrzekazZgodeStronParams {
        wpis_id: "zgoda".to_string(),
        strona: StronaDokumentu::Biuro,
        autor: OFFICE_AUTHOR.to_string(),
        chwila: "2026-11-04".to_string(),
        poprawia_id: None,
        tresc: None,
        sciezka: FILE_PATH.to_string(),
        skrot: FILE_DIGEST.to_string(),
        strona_id: Some("sprzedajaca".to_string()),
        data_wystawienia: None,
        zakres: Some("biuro dostaje projekt aktu".to_string()),
    };
    let land = PrzekazDaneNieruchomosciParams {
        wpis_id: "dane-lokalu".to_string(),
        strona: StronaDanych::Biuro,
        autor: OFFICE_AUTHOR.to_string(),
        chwila: "2026-11-04".to_string(),
        odpowiada_na_id: None,
        poprawia_id: None,
        strona_id: "sprzedajaca".to_string(),
        numer_kw: None,
        oznaczenie: "lokal nr 12".to_string(),
        dzialki: None,
        zrodlo: "odpis KW".to_string(),
    };
    let credit = PrzekazKredytParams {
        wpis_id: "kredyt".to_string(),
        strona: StronaDanych::Biuro,
        autor: OFFICE_AUTHOR.to_string(),
        chwila: "2026-11-04".to_string(),
        odpowiada_na_id: None,
        poprawia_id: None,
        strona_id: "kupujacy".to_string(),
        bank: "Bank Polski S.A.".to_string(),
        kwota: 45_000_000,
        warunki_wyplaty: None,
    };

    expect::refused(
        wymiana_dokumenty::przekaz_dokument_z_listy(&room, &document()),
        document_rule,
    );
    expect::refused(
        wymiana_dokumenty::przekaz_zgode_stron(&room, &consent),
        document_rule,
    );
    expect::refused(
        wymiana_dane::przekaz_dane_osoby(&room, &person()),
        data_rule,
    );
    expect::refused(
        wymiana_dane::przekaz_dane_nieruchomosci(&room, &land),
        data_rule,
    );
    expect::refused(wymiana_dane::przekaz_kredyt(&room, &credit), data_rule);

    give_power(&room, "pelnomocnictwo-sprzedajacej", "sprzedajaca");
    give_document(&room, &document());
    wymiana_dokumenty::przekaz_zgode_stron(&room, &consent).expect("give the consent");
    give_person(&room, &person());
    wymiana_dane::przekaz_dane_nieruchomosci(&room, &land).expect("give the data of the flat");
    expect::refused(wymiana_dane::przekaz_kredyt(&room, &credit), data_rule);

    give_power(&room, "pelnomocnictwo-kupujacego", "kupujacy");
    wymiana_dane::przekaz_kredyt(&room, &credit).expect("give the credit");
}

/// Pełnomocnictwo daje strona, którą biuro reprezentuje według metryki
#[test]
fn power_of_represented_party() {
    let room = new_room();
    open_exchange(&room);

    expect::refused(
        wymiana_dokumenty::przekaz_pelnomocnictwo(
            &room,
            &PrzekazPelnomocnictwoParams {
                wpis_id: "pelnomocnictwo".to_string(),
                strona: StronaDokumentu::Biuro,
                autor: OFFICE_AUTHOR.to_string(),
                chwila: "2026-11-02".to_string(),
                poprawia_id: None,
                tresc: None,
                sciezka: FILE_PATH.to_string(),
                skrot: FILE_DIGEST.to_string(),
                strona_id: Some("malzonka-kupujacego".to_string()),
                wystawca: None,
                data_wystawienia: None,
                waznosc_do: None,
                zakres: Some("przekazywanie dokumentów".to_string()),
            },
        ),
        "pełnomocnictwo dotyczy strony, którą biuro reprezentuje według metryki",
    );
    give_power(&room, "pelnomocnictwo", "kupujacy");
}

/// Dokument ma wystawcę i oryginał, pełnomocnictwo zakres, a projekt numer wersji
#[test]
fn document_fields_by_kind() {
    let room = new_room();
    open_exchange(&room);
    give_power(&room, "pelnomocnictwo", "sprzedajaca");
    give_list(&room);

    expect::refused(
        wymiana_dokumenty::przekaz_dokument_z_listy(
            &room,
            &PrzekazDokumentZListyParams {
                wystawca: None,
                ..document()
            },
        ),
        "dokument_ma_wystawce_i_oryginal",
    );
    expect::refused(
        wymiana_dokumenty::przekaz_dokument_z_listy(
            &room,
            &PrzekazDokumentZListyParams {
                kto_przyniesie: None,
                ..document()
            },
        ),
        "oryginal_ma_kto_przyniesie",
    );
    expect::refused(
        wymiana_dokumenty::przekaz_dokument_z_listy(
            &room,
            &PrzekazDokumentZListyParams {
                oryginal_w_dniu_aktu: Some(false),
                ..document()
            },
        ),
        "oryginal_ma_kto_przyniesie",
    );
    expect::refused(
        wymiana_dokumenty::przekaz_pelnomocnictwo(
            &room,
            &PrzekazPelnomocnictwoParams {
                wpis_id: "bez-zakresu".to_string(),
                strona: StronaDokumentu::Biuro,
                autor: OFFICE_AUTHOR.to_string(),
                chwila: "2026-11-02".to_string(),
                poprawia_id: None,
                tresc: None,
                sciezka: FILE_PATH.to_string(),
                skrot: FILE_DIGEST.to_string(),
                strona_id: Some("sprzedajaca".to_string()),
                wystawca: None,
                data_wystawienia: None,
                waznosc_do: None,
                zakres: None,
            },
        ),
        "pelnomocnictwo_i_zgoda_z_zakresem",
    );
    expect::refused(
        wymiana_dokumenty::przekaz_projekt_aktu(
            &room,
            &PrzekazProjektAktuParams {
                ostateczna: None,
                ..draft("projekt", 1)
            },
        ),
        "projekt_ma_wersje",
    );
    give_document(
        &room,
        &PrzekazDokumentZListyParams {
            oryginal_w_dniu_aktu: Some(false),
            kto_przyniesie: None,
            ..document()
        },
    );
}

/// The default parameters of a draft of the act from the notary's office.
fn draft(wpis_id: &str, numer_wersji: i64) -> PrzekazProjektAktuParams {
    PrzekazProjektAktuParams {
        wpis_id: wpis_id.to_string(),
        strona: StronaDokumentu::Kancelaria,
        autor: NOTARY_AUTHOR.to_string(),
        chwila: "2026-11-10".to_string(),
        odpowiada_na_id: None,
        tresc: None,
        sciezka: FILE_PATH.to_string(),
        skrot: FILE_DIGEST.to_string(),
        numer_wersji: Some(numer_wersji),
        ostateczna: Some(false),
    }
}

/// Numer wersji projektu aktu rośnie; nowa wersja staje obok starej
#[test]
fn draft_versions_grow() {
    let room = new_room();
    open_exchange(&room);
    give_draft(&room, "projekt-1", 1, false);
    give_draft(&room, "projekt-2", 2, false);

    expect::refused(
        wymiana_dokumenty::przekaz_projekt_aktu(&room, &draft("projekt-2b", 2)),
        "numer wersji projektu aktu rośnie",
    );
    expect::refused(
        wymiana_dokumenty::przekaz_projekt_aktu(&room, &draft("projekt-1b", 1)),
        "numer wersji projektu aktu rośnie",
    );
    give_draft(&room, "projekt-3", 3, true);

    let drafts = wymiana_dokumenty::wersje_projektu_aktu(&room).expect("read the drafts");
    let final_flags: Vec<Option<bool>> = drafts.iter().map(|row| row.ostateczna).collect();
    assert_eq!(final_flags, [Some(true), Some(false), Some(false)]);
}

/// Uwagi stron wskazują wersję projektu i decyzję z teczki transakcji
#[test]
fn comments_on_draft() {
    let room = new_room();
    open_exchange(&room);
    give_power(&room, "pelnomocnictwo", "kupujacy");
    give_draft(&room, "projekt-1", 1, false);
    let comments = PrzekazUwagiDoProjektuParams {
        wpis_id: "uwagi".to_string(),
        strona: StronaDokumentu::Biuro,
        autor: OFFICE_AUTHOR.to_string(),
        chwila: "2026-11-11".to_string(),
        projekt_id: "projekt-1".to_string(),
        tresc: Some("Kupujący prosi o wydanie lokalu do 2026-12-15.".to_string()),
        sciezka: None,
        skrot: None,
        decyzja_odnosnik: "pokoj-transakcji-017/decyzja-uwagi".to_string(),
    };

    expect::refused(
        wymiana_dokumenty::przekaz_uwagi_do_projektu(
            &room,
            &PrzekazUwagiDoProjektuParams {
                projekt_id: "pelnomocnictwo".to_string(),
                ..comments.clone()
            },
        ),
        "uwagi wskazują wersję projektu aktu",
    );

    // A writer outside the client can leave out the decision that `przekaz_uwagi_do_projektu`
    // requires. The table refuses the comments without one. Both statements of the operation run
    // in one write, so the refusal leaves nothing behind.
    let comments_without_decision = room.write(&[
        Statement {
            sql: "INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id,
              tresc)
            VALUES (:wpis_id, :strona, :autor, 'uwagi_do_projektu', :chwila, :projekt_id, :tresc)",
            parameters: vec![
                ("wpis_id", text("uwagi")),
                ("strona", text("biuro")),
                ("autor", text(OFFICE_AUTHOR)),
                ("chwila", text("2026-11-11")),
                ("projekt_id", text("projekt-1")),
                (
                    "tresc",
                    text("Kupujący prosi o wydanie lokalu do 2026-12-15."),
                ),
            ],
        },
        Statement {
            sql: "INSERT INTO wymiana_dokumenty_uwagi (wpis_id, projekt_id, decyzja_odnosnik)
            VALUES (:wpis_id, :projekt_id, :decyzja_odnosnik)",
            parameters: vec![
                ("wpis_id", text("uwagi")),
                ("projekt_id", text("projekt-1")),
                ("decyzja_odnosnik", SqlValue::Null),
            ],
        },
    ]);
    expect::refused(
        comments_without_decision,
        "wymiana_dokumenty_uwagi.decyzja_odnosnik",
    );

    wymiana_dokumenty::przekaz_uwagi_do_projektu(&room, &comments)
        .expect("give the comments on the draft");

    let entry = wymiana_wpisy::wpis(
        &room,
        &WpisParams {
            id: "uwagi".to_string(),
        },
    )
    .expect("read the entry")
    .expect("the entry exists");
    assert_eq!(entry.odpowiada_na_id.as_deref(), Some("projekt-1"));
}

// Data of the act ---------------------------------------------------------------------------------

/// Dane osoby z art. 92 § 1 pkt 4 PoN: imiona rodziców strony, nazwa i siedziba podmiotu
#[test]
fn person_data_of_the_act() {
    let room = new_room();
    open_exchange(&room);
    give_power(&room, "pelnomocnictwo", "sprzedajaca");

    expect::refused(
        wymiana_dane::przekaz_dane_osoby(
            &room,
            &PrzekazDaneOsobyParams {
                imiona_rodzicow: None,
                ..person()
            },
        ),
        "strona_fizyczna_ma_imiona_rodzicow",
    );
    expect::refused(
        wymiana_dane::przekaz_dane_osoby(
            &room,
            &PrzekazDaneOsobyParams {
                wpis_id: "pelnomocnik".to_string(),
                udzial: OsobyUdzial::Pelnomocnik,
                ..person()
            },
        ),
        "strona_fizyczna_ma_imiona_rodzicow",
    );
    expect::refused(
        wymiana_dane::przekaz_dane_osoby(
            &room,
            &PrzekazDaneOsobyParams {
                miejsce_zamieszkania: None,
                ..person()
            },
        ),
        "osoba_ma_imiona_nazwisko_i_zamieszkanie",
    );
    let company = PrzekazDaneOsobyParams {
        wpis_id: "spolka".to_string(),
        rodzaj_podmiotu: OsobyRodzajPodmiotu::Podmiot,
        imiona: None,
        nazwisko: None,
        imiona_rodzicow: None,
        miejsce_zamieszkania: None,
        stan_cywilny: None,
        nazwa: Some("Polna Development sp. z o.o.".to_string()),
        siedziba: Some("Kraków".to_string()),
        ..person()
    };
    expect::refused(
        wymiana_dane::przekaz_dane_osoby(
            &room,
            &PrzekazDaneOsobyParams {
                siedziba: None,
                ..company.clone()
            },
        ),
        "podmiot_ma_nazwe_i_siedzibe",
    );
    expect::refused(
        wymiana_dane::przekaz_dane_osoby(
            &room,
            &PrzekazDaneOsobyParams {
                stan_cywilny: Some("wolny".to_string()),
                ..company.clone()
            },
        ),
        "podmiot_bez_danych_osoby",
    );
    expect::refused(
        wymiana_dane::przekaz_dane_osoby(
            &room,
            &PrzekazDaneOsobyParams {
                udzial: OsobyUdzial::Pelnomocnik,
                ..company.clone()
            },
        ),
        "podmiot_tylko_strona",
    );

    // A writer outside the client can leave out the source document that `przekaz_dane_osoby`
    // requires. The table refuses the data without one. Both statements of the operation run in
    // one write, so the refusal leaves nothing behind.
    let data_without_source = room.write(&[
        Statement {
            sql: "INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila)
            VALUES (:wpis_id, :strona, :autor, 'dane_osoby', :chwila)",
            parameters: vec![
                ("wpis_id", text("dane-sprzedajacej")),
                ("strona", text("biuro")),
                ("autor", text(OFFICE_AUTHOR)),
                ("chwila", text("2026-11-04T10:00:00Z")),
            ],
        },
        Statement {
            sql: "INSERT INTO wymiana_dane_osoby (wpis_id, strona_id, udzial, rodzaj_podmiotu, imiona,
              nazwisko, imiona_rodzicow, miejsce_zamieszkania, stan_cywilny, zrodlo_dokument_odnosnik)
            VALUES (:wpis_id, :strona_id, :udzial, :rodzaj_podmiotu, :imiona, :nazwisko,
              :imiona_rodzicow, :miejsce_zamieszkania, :stan_cywilny, :zrodlo_dokument_odnosnik)",
            parameters: vec![
                ("wpis_id", text("dane-sprzedajacej")),
                ("strona_id", text("sprzedajaca")),
                ("udzial", text("strona")),
                ("rodzaj_podmiotu", text("osoba_fizyczna")),
                ("imiona", text("Anna Maria")),
                ("nazwisko", text("Kowalska")),
                ("imiona_rodzicow", text("Jan, Maria")),
                ("miejsce_zamieszkania", text("Kraków")),
                ("stan_cywilny", text("rozwiedziona")),
                ("zrodlo_dokument_odnosnik", SqlValue::Null),
            ],
        },
    ]);
    expect::refused(
        data_without_source,
        "wymiana_dane_osoby.zrodlo_dokument_odnosnik",
    );

    give_person(&room, &person());
    give_person(&room, &company);
    give_person(
        &room,
        &PrzekazDaneOsobyParams {
            wpis_id: "pelnomocnik".to_string(),
            udzial: OsobyUdzial::Pelnomocnik,
            imiona: Some("Marek".to_string()),
            nazwisko: Some("Wiśniewski".to_string()),
            imiona_rodzicow: None,
            stan_cywilny: None,
            ..person()
        },
    );
}

/// Rachunek od biura wskazuje decyzję, a rachunek od kancelarii jest informacją bez decyzji
#[test]
fn accounts_by_side() {
    let room = new_room();
    open_exchange(&room);
    let rule = "rachunek od biura ma odnośnik do decyzji";
    let account = |wpis_id: &str, strona: StronaDanych| PodajRachunekParams {
        wpis_id: wpis_id.to_string(),
        strona,
        autor: match strona {
            StronaDanych::Biuro => OFFICE_AUTHOR.to_string(),
            StronaDanych::Kancelaria => NOTARY_AUTHOR.to_string(),
        },
        chwila: "2026-11-05".to_string(),
        odpowiada_na_id: None,
        poprawia_id: None,
        tresc: None,
        numer: "PL61109010140000071219812874".to_string(),
        posiadacz: "Anna Kowalska".to_string(),
        cel: RachunkiCel::ZaplataCeny,
        decyzja_odnosnik: None,
    };
    let decision = Some("pokoj-transakcji-017/decyzja-rachunek".to_string());

    expect::refused(
        wymiana_dane::podaj_rachunek(&room, &account("rachunek-biura", StronaDanych::Biuro)),
        rule,
    );
    expect::refused(
        wymiana_dane::podaj_rachunek(
            &room,
            &PodajRachunekParams {
                decyzja_odnosnik: decision.clone(),
                ..account("rachunek-kancelarii", StronaDanych::Kancelaria)
            },
        ),
        rule,
    );
    wymiana_dane::podaj_rachunek(
        &room,
        &PodajRachunekParams {
            decyzja_odnosnik: decision,
            ..account("rachunek-biura", StronaDanych::Biuro)
        },
    )
    .expect("give the office's account");
    wymiana_dane::podaj_rachunek(
        &room,
        &PodajRachunekParams {
            cel: RachunkiCel::PodatkiIOplaty,
            posiadacz: "Urząd Skarbowy Kraków".to_string(),
            ..account("rachunek-podatku", StronaDanych::Kancelaria)
        },
    )
    .expect("give the notary's account for taxes");

    let accounts = wymiana_dane::rachunki_do_aktu(&room).expect("read the accounts");
    let sides: Vec<StronaDanych> = accounts.iter().map(|row| row.strona).collect();
    assert_eq!(sides, [StronaDanych::Biuro, StronaDanych::Kancelaria]);
}

// The date of the act -----------------------------------------------------------------------------

/// Termin jest potwierdzony dopiero wtedy, gdy potwierdzą go obie strony; biuro z decyzją
#[test]
fn term_confirmed_by_both() {
    let room = new_room();
    open_exchange(&room);
    propose_term(&room, "termin", "propozycja", StronaAktu::Kancelaria);
    let rule = "potwierdzenie terminu przez biuro wskazuje decyzję";

    expect::refused(
        wymiana_akt::oglos_stan_terminu(
            &room,
            &OglosStanTerminuParams {
                chwila: "2026-11-12".to_string(),
                ..announcement(
                    "bez-decyzji",
                    "termin",
                    TerminyStanyStan::Potwierdzony,
                    StronaAktu::Biuro,
                )
            },
        ),
        rule,
    );
    expect::refused(
        wymiana_akt::oglos_stan_terminu(
            &room,
            &OglosStanTerminuParams {
                chwila: "2026-11-12".to_string(),
                decyzja_odnosnik: Some("pokoj-transakcji-017/decyzja-termin".to_string()),
                ..announcement(
                    "z-decyzja",
                    "termin",
                    TerminyStanyStan::Potwierdzony,
                    StronaAktu::Kancelaria,
                )
            },
        ),
        rule,
    );

    announce(
        &room,
        &announcement(
            "kancelaria",
            "termin",
            TerminyStanyStan::Potwierdzony,
            StronaAktu::Kancelaria,
        ),
    );
    let terms = wymiana_akt::terminy_aktu(&room).expect("read the dates of the act");
    let term = &terms[0];
    assert_eq!(
        (
            term.stan,
            term.potwierdzila_kancelaria,
            term.potwierdzila_biuro
        ),
        (StanTerminu::Proponowany, true, false)
    );

    announce(
        &room,
        &OglosStanTerminuParams {
            decyzja_odnosnik: Some("pokoj-transakcji-017/decyzja-termin".to_string()),
            ..announcement(
                "biuro",
                "termin",
                TerminyStanyStan::Potwierdzony,
                StronaAktu::Biuro,
            )
        },
    );
    let terms_after_office = wymiana_akt::terminy_aktu(&room).expect("read the dates of the act");
    assert_eq!(terms_after_office[0].stan, StanTerminu::Potwierdzony);
}

/// Akt odbywa się w potwierdzonym terminie, a ogłasza to kancelaria; zakończony termin stoi
#[test]
fn term_held_and_finished() {
    let room = new_room();
    open_exchange(&room);
    propose_term(&room, "termin", "propozycja", StronaAktu::Biuro);
    let held = |wpis_id: &str, strona: StronaAktu| OglosStanTerminuParams {
        chwila: "2026-11-25".to_string(),
        ..announcement(wpis_id, "termin", TerminyStanyStan::OdbylSie, strona)
    };

    expect::refused(
        wymiana_akt::oglos_stan_terminu(&room, &held("za-wczesnie", StronaAktu::Kancelaria)),
        "odbywa się tylko termin potwierdzony przez obie strony",
    );
    confirm_by_both(&room, "termin");
    expect::refused(
        wymiana_akt::oglos_stan_terminu(&room, &held("od-biura", StronaAktu::Biuro)),
        "że akt się odbył, ogłasza kancelaria",
    );
    announce(&room, &held("odbyl-sie", StronaAktu::Kancelaria));

    expect::refused(
        wymiana_akt::oglos_stan_terminu(
            &room,
            &OglosStanTerminuParams {
                chwila: "2026-11-26".to_string(),
                ..announcement(
                    "odwolanie",
                    "termin",
                    TerminyStanyStan::Odwolany,
                    StronaAktu::Kancelaria,
                )
            },
        ),
        "odwołany, przesunięty albo odbyty termin nie ma kolejnych stanów",
    );
}

/// Przesunięty termin zastępuje nowa propozycja; przesuniętego się nie potwierdza
#[test]
fn term_moved() {
    let room = new_room();
    open_exchange(&room);
    propose_term(&room, "termin-1", "propozycja-1", StronaAktu::Kancelaria);
    announce(
        &room,
        &announcement(
            "przesuniecie",
            "termin-1",
            TerminyStanyStan::Przesuniety,
            StronaAktu::Kancelaria,
        ),
    );

    expect::refused(
        wymiana_akt::oglos_stan_terminu(
            &room,
            &OglosStanTerminuParams {
                chwila: "2026-11-13".to_string(),
                ..announcement(
                    "potwierdzenie",
                    "termin-1",
                    TerminyStanyStan::Potwierdzony,
                    StronaAktu::Kancelaria,
                )
            },
        ),
        "odwołany, przesunięty albo odbyty termin nie ma kolejnych stanów",
    );
    propose_term(&room, "termin-2", "propozycja-2", StronaAktu::Kancelaria);
    confirm_by_both(&room, "termin-2");

    let terms = wymiana_akt::terminy_aktu(&room).expect("read the dates of the act");
    let term_states: Vec<StanTerminu> = terms.iter().map(|row| row.stan).collect();
    assert_eq!(
        term_states,
        [StanTerminu::Przesuniety, StanTerminu::Potwierdzony]
    );
}

/// Pierwokup: wynik ma datę i przychodzi po doręczeniu zawiadomienia
#[test]
fn pre_emption() {
    let room = new_room();
    open_exchange(&room);
    let pre_emption = PodajPierwokupParams {
        wpis_id: "pierwokup".to_string(),
        strona: StronaAktu::Kancelaria,
        autor: NOTARY_AUTHOR.to_string(),
        chwila: "2026-10-21".to_string(),
        poprawia_id: None,
        tresc: None,
        uprawniony: "Gmina Miejska Kraków".to_string(),
        zawiadomienie_doreczono: None,
        wynik: None,
        data: None,
    };

    expect::refused(
        wymiana_akt::podaj_pierwokup(
            &room,
            &PodajPierwokupParams {
                zawiadomienie_doreczono: Some("2026-10-20".to_string()),
                wynik: Some(PierwokupyWynik::NieWykonal),
                ..pre_emption.clone()
            },
        ),
        "wynik_z_data",
    );
    expect::refused(
        wymiana_akt::podaj_pierwokup(
            &room,
            &PodajPierwokupParams {
                wynik: Some(PierwokupyWynik::NieWykonal),
                data: Some("2026-10-25".to_string()),
                ..pre_emption.clone()
            },
        ),
        "wynik_po_zawiadomieniu",
    );
    wymiana_akt::podaj_pierwokup(
        &room,
        &PodajPierwokupParams {
            zawiadomienie_doreczono: Some("2026-10-20".to_string()),
            ..pre_emption.clone()
        },
    )
    .expect("give the notice of the pre-emption");
    wymiana_akt::podaj_pierwokup(
        &room,
        &PodajPierwokupParams {
            wpis_id: "wynik".to_string(),
            chwila: "2026-11-20".to_string(),
            poprawia_id: Some("pierwokup".to_string()),
            zawiadomienie_doreczono: Some("2026-10-20".to_string()),
            wynik: Some(PierwokupyWynik::TerminUplynal),
            data: Some("2026-11-20".to_string()),
            ..pre_emption
        },
    )
    .expect("give the result of the pre-emption");
}

/// Informacja po akcie ma co najmniej jedną datę
#[test]
fn after_the_act() {
    let room = new_room();
    open_exchange(&room);
    let after_the_act = PodajInformacjePoAkcieParams {
        wpis_id: "po-akcie".to_string(),
        strona: StronaAktu::Kancelaria,
        autor: NOTARY_AUTHOR.to_string(),
        chwila: "2026-11-25".to_string(),
        poprawia_id: None,
        tresc: None,
        akt_sporzadzono: None,
        wniosek_kw_zlozono: None,
    };

    expect::refused(
        wymiana_akt::podaj_informacje_po_akcie(&room, &after_the_act),
        "po_akcie_z_data",
    );
    wymiana_akt::podaj_informacje_po_akcie(
        &room,
        &PodajInformacjePoAkcieParams {
            akt_sporzadzono: Some("2026-11-25".to_string()),
            ..after_the_act
        },
    )
    .expect("give the information after the act");
}

// R9 and replacement ------------------------------------------------------------------------------

fn exchange_tables(room: &MemoryRoom) -> Vec<String> {
    let rows = room
        .execute(
            r"SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'wymiana\_%' ESCAPE '\'",
            &[],
        )
        .expect("read the tables");

    rows.iter()
        .map(|row| row.get("name").expect("a table has a name"))
        .collect()
}

/// The names of the columns of a table, in their order.
fn column_names_of(room: &MemoryRoom, table: &str) -> Vec<String> {
    let columns_info = room
        .execute(&format!("PRAGMA table_info({table})"), &[])
        .expect("read the columns");

    columns_info
        .iter()
        .map(|row| row.get("name").expect("a column has a name"))
        .collect()
}

/// R9: wierszy pokoju wymiany się nie usuwa ani nie zmienia
#[test]
fn no_deletion_or_change() {
    let room = new_room();
    full_exchange(&room);
    let tables = exchange_tables(&room);
    assert_eq!(tables.len(), 15, "{tables:?}");

    for table in &tables {
        let row_count = room
            .value(&format!("SELECT count(*) FROM {table}"), &[])
            .expect("count the rows");
        let row_count = i64::from_sql_value(&row_count).expect("a count is a number");
        assert!(row_count > 0, "{table} has no row");
        let first_row = format!("rowid = (SELECT min(rowid) FROM {table})");

        expect::refused(
            room.execute(&format!("DELETE FROM {table} WHERE {first_row}"), &[]),
            "wierszy pokoju wymiany się nie usuwa (R9)",
        );

        let column_names = column_names_of(&room, table);
        let first_column = &column_names[0];
        let refusal = if table == "wymiana_metryka_wpis" {
            "metryki pokoju się nie zmienia"
        } else {
            "wiersza pokoju wymiany się nie zmienia"
        };
        expect::refused(
            room.execute(
                &format!("UPDATE {table} SET {first_column} = {first_column} WHERE {first_row}"),
                &[],
            ),
            refusal,
        );
    }
}

/// Wiersza pokoju wymiany nie zastępuje INSERT OR REPLACE ani upsert
#[test]
fn no_replacement() {
    let room = new_room();
    full_exchange(&room);
    let refusal = "wiersza pokoju wymiany nie zastępuje się nowym wpisem";

    for table in exchange_tables(&room) {
        let columns = column_names_of(&room, &table).join(", ");
        let copy = format!(
            "INTO {table} ({columns}) SELECT {columns} FROM {table} \
            WHERE rowid = (SELECT min(rowid) FROM {table})"
        );

        // REPLACE deletes the old row without the triggers of DELETE and UPDATE, and an upsert
        // updates it without the triggers of UPDATE.
        expect::refused(
            room.execute(&format!("INSERT OR REPLACE {copy}"), &[]),
            refusal,
        );
        expect::refused(
            room.execute(
                &format!("INSERT {copy} ON CONFLICT DO UPDATE SET rowid = excluded.rowid"),
                &[],
            ),
            refusal,
        );
    }

    // A copy with a new key that collides only on UNIQUE (wpis_id) is refused too.
    expect::refused(
        room.execute(
            "INSERT OR REPLACE INTO wymiana_akt_terminy (id, wpis_id, dzien, godzina, miejsce, czynnosc)
            SELECT 'inny-termin', wpis_id, dzien, godzina, miejsce, czynnosc FROM wymiana_akt_terminy",
            &[],
        ),
        refusal,
    );
}
