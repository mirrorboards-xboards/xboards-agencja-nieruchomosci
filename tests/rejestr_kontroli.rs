//! Scenarios of the control register of the office (spec/registers/rejestr_kontroli).
//!
//! Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
//! allows and a write it refuses. The first scenario follows a client from the identity check before
//! the first contract to the end of the business relationship and the client's return.

use xboards_agencja_nieruchomosci::registers::rejestr_kontroli::{
    self, rejestr_kontroli_decyzje, rejestr_kontroli_klienci, rejestr_kontroli_oceny,
    rejestr_kontroli_organizacja, rejestr_kontroli_sprawdzenia, rejestr_kontroli_watpliwosci,
    rejestr_kontroli_zawiadomienia,
};
use xboards_agencja_nieruchomosci::xmeta_room::{FromSqlValue, MemoryRoom, SqlValue};

use rejestr_kontroli_decyzje::{
    PoprosODecyzjeKontroliParams, RozstrzygnijDecyzjeKontroliParams,
    UznajProsbeKontroliZaZbednaParams, WpisyDecydujeFunkcja, WpisyRodzaj as RodzajDecyzji,
    WpisyRozstrzygniecie,
};
use rejestr_kontroli_klienci::{
    SprawyKlientaKontroliParams, SprawyObjeta, SprawyRodzajSprawy, StosunkiKlientaZKartotekiParams,
    WpiszKlientaKontroliParams, WpiszNawiazanieStosunkowParams, WpiszOdnosnikDoKartotekiParams,
    WpiszZakonczenieStosunkowParams, ZakwalifikujSpraweParams,
};
use rejestr_kontroli_oceny::{
    KlientowPoziom, KlientowSrodki, OcenyRyzykaKlientaParams, WpiszOceneRyzykaBiuraParams,
    WpiszOceneRyzykaKlientaParams, WpiszZatwierdzenieOcenyParams,
};
use rejestr_kontroli_organizacja::{
    WpiszProcedureParams, WpiszSzkolenieParams, WpiszWyznaczenieParams, WyznaczeniaFunkcja,
    ZakonczWyznaczenieParams,
};
use rejestr_kontroli_sprawdzenia::{
    IdentyfikacjeKlientaParams, IdentyfikacjeKogo, IdentyfikacjeRodzajPodmiotu, InformacjeSposob,
    MonitorowanieWynik, PepDotyczy, PepWynik, PozaUstawaRodzaj, PozaUstawaWynik, SankcjeWynik,
    SprawdzeniaPepKlientaParams, SprawdzeniaPozaUstawaTeczkiParams,
    SprawdzeniaSankcjiKlientaParams, WeryfikacjePodstawa, WpiszAkceptacjePepParams,
    WpiszAnalizeParams, WpiszCelStosunkowParams, WpiszIdentyfikacjeParams,
    WpiszInformacjeAmlParams, WpiszInformacjeOZamrozeniuParams, WpiszKopieDokumentuParams,
    WpiszOdpisCrbrParams, WpiszSprawdzeniePepParams, WpiszSprawdzeniePozaUstawaParams,
    WpiszSprawdzenieSankcjiParams, WpiszWeryfikacjeParams, WpiszZamrozenieParams,
    WpiszZrodloSrodkowParams,
};
use rejestr_kontroli_watpliwosci::{
    NiemoznosciSrodek, WpiszNiemoznoscParams, WpiszWatpliwoscParams, WpiszZdjecieWatpliwosciParams,
};
use rejestr_kontroli_zawiadomienia::{
    WpisyRodzaj as RodzajZawiadomienia, WpiszKontroleOrganuParams, WpiszOdpowiedzNaZadanieParams,
    WpiszOdpowiedzOrganuParams, WpiszPotwierdzeniePrzyjeciaParams, WpiszZadanieOrganuParams,
    WpiszZawiadomienieParams,
};

use crate::expect;

const REPLACEMENT: &str =
    "wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE";
const DELETION: &str = "wierszy rejestru się nie usuwa (R9)";
const CHECK_UNCHANGED: &str = "wpisu sprawdzenia się nie zmienia; ponowne sprawdzenie to nowy wpis";
const CLIENTS: &str = "rejestr_kontroli_klienci_wpisy";

/// The tables of the checks whose re-check replaces an entry, and the column the two entries share.
const RECHECKED_TABLES: [(&str, &str); 8] = [
    ("rejestr_kontroli_sprawdzenia_identyfikacje", "klient_id"),
    (
        "rejestr_kontroli_sprawdzenia_weryfikacje",
        "identyfikacja_id",
    ),
    ("rejestr_kontroli_sprawdzenia_crbr", "klient_id"),
    ("rejestr_kontroli_sprawdzenia_pep", "klient_id"),
    ("rejestr_kontroli_sprawdzenia_sankcje", "klient_id"),
    ("rejestr_kontroli_sprawdzenia_cele", "klient_id"),
    ("rejestr_kontroli_sprawdzenia_zrodla_srodkow", "klient_id"),
    (
        "rejestr_kontroli_sprawdzenia_poza_ustawa",
        "wiersz_odnosnik",
    ),
];

fn new_room() -> MemoryRoom {
    rejestr_kontroli::open_in_memory().expect("open the control register")
}

/// The number a read returns, e.g. a count or the id of a row.
fn number_of(room: &MemoryRoom, sql: &str) -> i64 {
    let value = room.value(sql, &[]).expect("read a number");

    i64::from_sql_value(&value).expect("the read returns a number")
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

/// Who signs the decisions of a function.
fn decider_of(funkcja: WpisyDecydujeFunkcja) -> &'static str {
    match funkcja {
        WpisyDecydujeFunkcja::WlascicielBiura => "wlasciciel-biura",
        WpisyDecydujeFunkcja::OsobaWyznaczona => "osoba-wyznaczona",
    }
}

/// The default parameters of a request for a decision of `funkcja` about no row of the register.
fn request(
    decision_id: &str,
    rodzaj: RodzajDecyzji,
    funkcja: WpisyDecydujeFunkcja,
) -> PoprosODecyzjeKontroliParams {
    PoprosODecyzjeKontroliParams {
        id: decision_id.to_string(),
        rodzaj,
        prosi: "kierownik-kontroli".to_string(),
        decyduje: decider_of(funkcja).to_string(),
        decyduje_funkcja: funkcja,
        zlozono: "2026-03-01T10:00:00Z".to_string(),
        tresc: None,
        dotyczy_tabela: None,
        dotyczy_id: None,
        dotyczy_skrot: None,
    }
}

/// The default parameters of a request for a decision of `funkcja` about the row `dotyczy_id` of
/// the table `dotyczy_tabela`.
fn request_about(
    decision_id: &str,
    rodzaj: RodzajDecyzji,
    funkcja: WpisyDecydujeFunkcja,
    dotyczy_tabela: &str,
    dotyczy_id: &str,
) -> PoprosODecyzjeKontroliParams {
    PoprosODecyzjeKontroliParams {
        dotyczy_tabela: Some(dotyczy_tabela.to_string()),
        dotyczy_id: Some(dotyczy_id.to_string()),
        ..request(decision_id, rodzaj, funkcja)
    }
}

fn ask(room: &MemoryRoom, params: &PoprosODecyzjeKontroliParams) {
    rejestr_kontroli_decyzje::popros_o_decyzje_kontroli(room, params).expect("ask for a decision");
}

/// The default parameters of the consent of the person of `funkcja`, signed with their key.
fn resolution(
    decision_id: &str,
    funkcja: WpisyDecydujeFunkcja,
) -> RozstrzygnijDecyzjeKontroliParams {
    RozstrzygnijDecyzjeKontroliParams {
        rozstrzygniecie: Some(WpisyRozstrzygniecie::Zgoda),
        uzasadnienie: None,
        podpisal: Some(decider_of(funkcja).to_string()),
        podpis: Some("podpis-kluczem-xkeys".to_string()),
        podpisano: Some("2026-03-01T12:00:00Z".to_string()),
        teczka_decyzja_odnosnik: None,
        id: decision_id.to_string(),
    }
}

/// The default parameters of the consent of the designated person, which the scenarios of the
/// rules of resolving override.
fn designated_consent(decision_id: &str) -> RozstrzygnijDecyzjeKontroliParams {
    RozstrzygnijDecyzjeKontroliParams {
        podpis: Some("podpis".to_string()),
        ..resolution(decision_id, WpisyDecydujeFunkcja::OsobaWyznaczona)
    }
}

fn resolve(room: &MemoryRoom, params: &RozstrzygnijDecyzjeKontroliParams) {
    rejestr_kontroli_decyzje::rozstrzygnij_decyzje_kontroli(room, params)
        .expect("resolve the decision");
}

/// Asks for the decision of `params` and resolves it with the consent of the person who decides.
fn approve(room: &MemoryRoom, params: &PoprosODecyzjeKontroliParams) {
    ask(room, params);
    resolve(room, &resolution(&params.id, params.decyduje_funkcja));
}

/// The default parameters of a client: Anna Nowak with her file in the client register.
fn client(client_id: &str) -> WpiszKlientaKontroliParams {
    WpiszKlientaKontroliParams {
        id: client_id.to_string(),
        klient: "Anna Nowak".to_string(),
        kartoteka_odnosnik: Some("kartoteka/nowak".to_string()),
        kontroler: "kontroler-nowak".to_string(),
        stosunki_od: None,
        stosunki_do: None,
        zrodlo: "raport-kontrolera/1".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-01".to_string(),
    }
}

fn add_client(room: &MemoryRoom, params: &WpiszKlientaKontroliParams) {
    rejestr_kontroli_klienci::wpisz_klienta_kontroli(room, params).expect("enter the client");
}

/// The default parameters of the identification of Anna Nowak by her identity card.
fn person_identification() -> WpiszIdentyfikacjeParams {
    WpiszIdentyfikacjeParams {
        klient_id: "nowak".to_string(),
        kogo: IdentyfikacjeKogo::Klient,
        rodzaj_podmiotu: IdentyfikacjeRodzajPodmiotu::OsobaFizyczna,
        imiona: Some("Anna".to_string()),
        nazwisko: Some("Nowak".to_string()),
        obywatelstwo: Some("PL".to_string()),
        pesel: Some("80010112345".to_string()),
        data_urodzenia: None,
        panstwo_urodzenia: None,
        dokument_tozsamosci: Some("ABC123456".to_string()),
        adres_zamieszkania: None,
        firma: None,
        nip: None,
        adres_dzialalnosci: None,
        nazwa: None,
        forma_organizacyjna: None,
        adres_siedziby: None,
        rejestr_panstwo: None,
        rejestr_nazwa: None,
        rejestr_numer: None,
        rejestr_data: None,
        struktura_wlasnosci: None,
        umocowanie_dokument: None,
        zrodlo: "dowód osobisty".to_string(),
        raport_odnosnik: "raport-kontrolera/1".to_string(),
        sprawdzil: "kontroler-nowak".to_string(),
        sprawdzono: "2026-03-01".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-01".to_string(),
        poprzedni_id: None,
    }
}

/// Enters an identification; the id of the newest identification of its client.
fn identify(room: &MemoryRoom, params: &WpiszIdentyfikacjeParams) -> i64 {
    rejestr_kontroli_sprawdzenia::wpisz_identyfikacje(room, params)
        .expect("enter the identification");

    let of_client = IdentyfikacjeKlientaParams {
        klient_id: params.klient_id.clone(),
    };
    let identifications = rejestr_kontroli_sprawdzenia::identyfikacje_klienta(room, &of_client)
        .expect("read the identifications");

    identifications[0].id
}

/// The default parameters of the verification of an identification against the identity card.
fn verification(identyfikacja_id: i64) -> WpiszWeryfikacjeParams {
    WpiszWeryfikacjeParams {
        identyfikacja_id,
        podstawa: WeryfikacjePodstawa::DowodOsobisty,
        wynik: PozaUstawaWynik::Zgodne,
        opis: None,
        zrodlo: "dowód osobisty".to_string(),
        raport_odnosnik: "raport-kontrolera/1".to_string(),
        sprawdzil: "kontroler-nowak".to_string(),
        sprawdzono: "2026-03-01".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-01".to_string(),
        poprzedni_id: None,
    }
}

/// The default parameters of a PEP check of Anna Nowak: no politically exposed person, with a
/// statement under the clause.
fn pep() -> WpiszSprawdzeniePepParams {
    WpiszSprawdzeniePepParams {
        klient_id: "nowak".to_string(),
        dotyczy: PepDotyczy::Klient,
        osoba: "Anna Nowak".to_string(),
        oswiadczenie_sciezka: Some("oswiadczenia/nowak.pdf".to_string()),
        oswiadczenie_skrot: Some("sha256-oswiadczenia".to_string()),
        oswiadczenie_z_klauzula: true,
        wykaz_sprawdzono: "2026-03-01".to_string(),
        wynik: PepWynik::NiePep,
        stanowisko_do: None,
        zrodlo_majatku: None,
        zrodlo: "dowód osobisty".to_string(),
        raport_odnosnik: "raport-kontrolera/1".to_string(),
        sprawdzil: "kontroler-nowak".to_string(),
        sprawdzono: "2026-03-01".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-01".to_string(),
        poprzedni_id: None,
    }
}

/// The default parameters of a check outside the act: the bank account of a rent payout, confirmed
/// by phone.
fn out_of_act() -> WpiszSprawdzeniePozaUstawaParams {
    WpiszSprawdzeniePozaUstawaParams {
        klient_id: Some("nowak".to_string()),
        rodzaj: PozaUstawaRodzaj::Rachunek,
        teczka_odnosnik: "teczka-najmu".to_string(),
        wiersz_odnosnik: "teczka-najmu/rachunek-1".to_string(),
        wersja_wiersza: None,
        sprawdzenie_odnosnik: "teczka-najmu/sprawdzenie-1".to_string(),
        wyplata_odnosnik: Some("teczka-najmu/wyplata-1".to_string()),
        zrodlo_data: None,
        wynik: PozaUstawaWynik::Zgodne,
        opis: None,
        zrodlo: "telefon do posiadacza z numeru z umowy".to_string(),
        raport_odnosnik: "raport-kontrolera/1".to_string(),
        sprawdzil: "kontroler-nowak".to_string(),
        sprawdzono: "2026-03-01".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-01".to_string(),
        poprzedni_id: None,
    }
}

/// The default parameters of a client's risk assessment: low risk, usual measures.
fn assessment(assessment_id: &str) -> WpiszOceneRyzykaKlientaParams {
    WpiszOceneRyzykaKlientaParams {
        id: assessment_id.to_string(),
        klient_id: "nowak".to_string(),
        poziom: KlientowPoziom::Niskie,
        czynniki: "osoba fizyczna, rezydent, sprzedaż mieszkania".to_string(),
        srodki: KlientowSrodki::Zwykle,
        zaproponowal: "kontroler-nowak".to_string(),
        raport_odnosnik: "raport-kontrolera/2".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-02".to_string(),
    }
}

fn add_assessment(room: &MemoryRoom, params: &WpiszOceneRyzykaKlientaParams) {
    rejestr_kontroli_oceny::wpisz_ocene_ryzyka_klienta(room, params)
        .expect("enter the client's risk assessment");
}

/// The default parameters of the first version of the office's risk assessment.
fn first_office_assessment() -> WpiszOceneRyzykaBiuraParams {
    WpiszOceneRyzykaBiuraParams {
        id: "ocena-biura-1".to_string(),
        wersja: 1,
        czynniki: "klienci, obszary, usługi, transakcje i kanały biura".to_string(),
        sporzadzono: "2026-01-10".to_string(),
        aktualizacja_do: "2028-01-10".to_string(),
        dokument_sciezka: "oceny/biuro-1.pdf".to_string(),
        dokument_skrot: "sha256-oceny".to_string(),
        akceptacja_decyzja_id: "akceptacja-oceny-1".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-01".to_string(),
    }
}

/// The default parameters of a doubt about a cash payment in the transaction's case file.
fn doubt() -> WpiszWatpliwoscParams {
    WpiszWatpliwoscParams {
        id: "watpliwosc".to_string(),
        teczka_odnosnik: "teczka-transakcji".to_string(),
        znacznik_odnosnik: "teczka-transakcji/znacznik-1".to_string(),
        klient_id: Some("nowak".to_string()),
        zglosil: "kontroler-nowak".to_string(),
        zgloszono: "2026-03-03T09:00:00Z".to_string(),
        powod: "zapłata gotówką z niewyjaśnionego źródła".to_string(),
        raport_odnosnik: "raport-kontrolera/3".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-03".to_string(),
    }
}

/// The default parameters of a notification under art. 74 AML after the decisions `potwierdzenie`
/// and `przekazanie`.
fn art74_notification() -> WpiszZawiadomienieParams {
    WpiszZawiadomienieParams {
        id: "zawiadomienie".to_string(),
        rodzaj: RodzajZawiadomienia::Art74,
        potwierdzenie_decyzja_id: "potwierdzenie".to_string(),
        przekazanie_decyzja_id: "przekazanie".to_string(),
        przekazal: "osoba-wyznaczona".to_string(),
        przekazano: "2026-03-02T09:00:00Z".to_string(),
        podstawy: "watpliwosc, analiza 12".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-02".to_string(),
    }
}

/// The default parameters of the first version of the procedure.
fn first_procedure() -> WpiszProcedureParams {
    WpiszProcedureParams {
        id: "procedura-1".to_string(),
        wersja: 1,
        dokument_sciezka: "procedury/1.pdf".to_string(),
        dokument_skrot: "sha256-procedury".to_string(),
        akceptacja_decyzja_id: "akceptacja-procedury-1".to_string(),
        wprowadzono: "2026-03-01".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-01".to_string(),
    }
}

/// A row in every table of the register.
fn full_register(room: &MemoryRoom) {
    add_client(room, &client("nowak"));
    let qualification = ZakwalifikujSpraweParams {
        klient_id: "nowak".to_string(),
        teczka_odnosnik: "teczka-oferty".to_string(),
        rodzaj_sprawy: SprawyRodzajSprawy::SprzedazAlboKupno,
        objeta: SprawyObjeta::Tak,
        podstawa: "art. 2 ust. 1 pkt 18 AML".to_string(),
        czynsz: None,
        kurs_euro: None,
        dzien_kursu: None,
        zakwalifikowano: "2026-03-01".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
    };
    rejestr_kontroli_klienci::zakwalifikuj_sprawe(room, &qualification).expect("qualify the case");

    let identification_id = identify(room, &person_identification());
    rejestr_kontroli_sprawdzenia::wpisz_weryfikacje(room, &verification(identification_id))
        .expect("enter the verification");
    let copy = WpiszKopieDokumentuParams {
        identyfikacja_id: identification_id,
        plik_sciezka: "kopie/nowak.pdf".to_string(),
        plik_skrot: "sha256-kopii".to_string(),
        zrodlo: "dowód osobisty".to_string(),
        raport_odnosnik: "raport-kontrolera/1".to_string(),
        sprawdzil: "kontroler-nowak".to_string(),
        sprawdzono: "2026-03-01".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-01".to_string(),
    };
    rejestr_kontroli_sprawdzenia::wpisz_kopie_dokumentu(room, &copy)
        .expect("enter the copy of the document");
    let crbr_extract = WpiszOdpisCrbrParams {
        klient_id: "nowak".to_string(),
        odpis_sciezka: "crbr/nowak.pdf".to_string(),
        odpis_skrot: "sha256-odpisu".to_string(),
        rozbieznosc: false,
        rozbieznosc_opis: None,
        wyjasnienie: None,
        zrodlo: "dowód osobisty".to_string(),
        raport_odnosnik: "raport-kontrolera/1".to_string(),
        sprawdzil: "kontroler-nowak".to_string(),
        sprawdzono: "2026-03-01".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-01".to_string(),
        poprzedni_id: None,
    };
    rejestr_kontroli_sprawdzenia::wpisz_odpis_crbr(room, &crbr_extract)
        .expect("enter the CRBR extract");
    rejestr_kontroli_sprawdzenia::wpisz_sprawdzenie_pep(room, &pep()).expect("enter the PEP check");
    let sanctions_check = WpiszSprawdzenieSankcjiParams {
        klient_id: "nowak".to_string(),
        listy: "ONZ, UE, lista GIIF".to_string(),
        wynik: SankcjeWynik::BrakTrafien,
        zrodlo: "dowód osobisty".to_string(),
        raport_odnosnik: "raport-kontrolera/1".to_string(),
        sprawdzil: "kontroler-nowak".to_string(),
        sprawdzono: "2026-03-01".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-01".to_string(),
        poprzedni_id: None,
    };
    rejestr_kontroli_sprawdzenia::wpisz_sprawdzenie_sankcji(room, &sanctions_check)
        .expect("enter the sanctions check");
    let purpose = WpiszCelStosunkowParams {
        klient_id: "nowak".to_string(),
        cel: "sprzedaż mieszkania odziedziczonego po matce".to_string(),
        charakter: "jednorazowa sprzedaż".to_string(),
        zrodlo: "dowód osobisty".to_string(),
        raport_odnosnik: "raport-kontrolera/1".to_string(),
        sprawdzil: "kontroler-nowak".to_string(),
        sprawdzono: "2026-03-01".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-01".to_string(),
        poprzedni_id: None,
    };
    rejestr_kontroli_sprawdzenia::wpisz_cel_stosunkow(room, &purpose)
        .expect("enter the purpose of the relationship");
    let source_of_funds = WpiszZrodloSrodkowParams {
        klient_id: "nowak".to_string(),
        pochodzenie_srodkow: "spadek".to_string(),
        dokument_sciezka: "zrodla/postanowienie.pdf".to_string(),
        dokument_skrot: "sha256-postanowienia".to_string(),
        zrodlo: "dowód osobisty".to_string(),
        raport_odnosnik: "raport-kontrolera/1".to_string(),
        sprawdzil: "kontroler-nowak".to_string(),
        sprawdzono: "2026-03-01".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-01".to_string(),
        poprzedni_id: None,
    };
    rejestr_kontroli_sprawdzenia::wpisz_zrodlo_srodkow(room, &source_of_funds)
        .expect("enter the source of funds");
    let analysis = WpiszAnalizeParams {
        klient_id: "nowak".to_string(),
        teczka_odnosnik: "teczka-transakcji".to_string(),
        analiza: "cena zgodna z rynkiem, zapłata przelewem z rachunku kupującego".to_string(),
        wynik: MonitorowanieWynik::Zgodne,
        opis: None,
        zrodlo: "dowód osobisty".to_string(),
        raport_odnosnik: "raport-kontrolera/1".to_string(),
        sprawdzil: "kontroler-nowak".to_string(),
        sprawdzono: "2026-03-01".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-01".to_string(),
    };
    rejestr_kontroli_sprawdzenia::wpisz_analize(room, &analysis).expect("enter the analysis");
    rejestr_kontroli_sprawdzenia::wpisz_sprawdzenie_poza_ustawa(room, &out_of_act())
        .expect("enter the check outside the act");
    let aml_information = WpiszInformacjeAmlParams {
        klient_id: "nowak".to_string(),
        poinformowano: "2026-03-01T09:00:00Z".to_string(),
        sposob: InformacjeSposob::Email,
        zrodlo: "teczka-oferty/wysylka-informacji".to_string(),
        raport_odnosnik: "raport-kontrolera/1".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-01".to_string(),
    };
    rejestr_kontroli_sprawdzenia::wpisz_informacje_aml(room, &aml_information)
        .expect("enter the AML information");

    approve(
        room,
        &request_about(
            "zatwierdzenie-oceny",
            RodzajDecyzji::ZatwierdzenieOceny,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
            "rejestr_kontroli_oceny_klientow",
            "ocena",
        ),
    );
    add_assessment(room, &assessment("ocena"));
    let approval = WpiszZatwierdzenieOcenyParams {
        zatwierdzenie_decyzja_id: "zatwierdzenie-oceny".to_string(),
        id: "ocena".to_string(),
    };
    rejestr_kontroli_oceny::wpisz_zatwierdzenie_oceny(room, &approval)
        .expect("enter the approval of the assessment");
    approve(
        room,
        &request_about(
            "akceptacja-oceny-1",
            RodzajDecyzji::AkceptacjaOcenyBiura,
            WpisyDecydujeFunkcja::WlascicielBiura,
            "rejestr_kontroli_oceny_biura",
            "ocena-biura-1",
        ),
    );
    rejestr_kontroli_oceny::wpisz_ocene_ryzyka_biura(room, &first_office_assessment())
        .expect("enter the office's risk assessment");

    rejestr_kontroli_watpliwosci::wpisz_watpliwosc(room, &doubt()).expect("enter the doubt");
    approve(
        room,
        &request_about(
            "nienawiazanie",
            RodzajDecyzji::NienawiazanieStosunkow,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
            CLIENTS,
            "nowak",
        ),
    );
    let impossibility = WpiszNiemoznoscParams {
        klient_id: "nowak".to_string(),
        srodek: NiemoznosciSrodek::Beneficjent,
        opis: "klient nie wskazał beneficjenta spółki".to_string(),
        decyzja_id: "nienawiazanie".to_string(),
        ocena_zawiadomienia: "brak podstaw do zawiadomienia".to_string(),
        raport_odnosnik: "raport-kontrolera/4".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-05".to_string(),
    };
    rejestr_kontroli_watpliwosci::wpisz_niemoznosc(room, &impossibility)
        .expect("enter the impossibility of a measure");

    approve(
        room,
        &request(
            "potwierdzenie",
            RodzajDecyzji::PotwierdzeniePodejrzenia,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
        ),
    );
    approve(
        room,
        &request(
            "przekazanie",
            RodzajDecyzji::PrzekazanieZawiadomienia,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
        ),
    );
    rejestr_kontroli_zawiadomienia::wpisz_zawiadomienie(room, &art74_notification())
        .expect("enter the notification");
    let authority_request = WpiszZadanieOrganuParams {
        organ: "Generalny Inspektor Informacji Finansowej".to_string(),
        zadano: "2026-04-01T09:00:00Z".to_string(),
        tresc: "dane klienta i transakcji".to_string(),
        odebral: "osoba-wyznaczona".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-04-01".to_string(),
    };
    rejestr_kontroli_zawiadomienia::wpisz_zadanie_organu(room, &authority_request)
        .expect("enter the authority's request");
    let inspection = WpiszKontroleOrganuParams {
        organ: "Krajowa Administracja Skarbowa".to_string(),
        od: "2026-05-04".to_string(),
        r#do: "2026-05-08".to_string(),
        protokol_sciezka: "kontrole/2026-05.pdf".to_string(),
        protokol_skrot: "sha256-protokolu".to_string(),
        zalecenia: None,
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-05-10".to_string(),
    };
    rejestr_kontroli_zawiadomienia::wpisz_kontrole_organu(room, &inspection)
        .expect("enter the authority's inspection");

    approve(
        room,
        &request_about(
            "akceptacja-procedury-1",
            RodzajDecyzji::AkceptacjaProcedury,
            WpisyDecydujeFunkcja::WlascicielBiura,
            "rejestr_kontroli_organizacja_procedury",
            "procedura-1",
        ),
    );
    rejestr_kontroli_organizacja::wpisz_procedure(room, &first_procedure())
        .expect("enter the procedure");
    let appointment = WpiszWyznaczenieParams {
        funkcja: WyznaczeniaFunkcja::Art8,
        osoba: "osoba-wyznaczona".to_string(),
        od: "2026-01-01".to_string(),
        podstawa: None,
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-01".to_string(),
    };
    rejestr_kontroli_organizacja::wpisz_wyznaczenie(room, &appointment)
        .expect("enter the appointment");
    let training = WpiszSzkolenieParams {
        uczestnik: "kierownik-kontroli".to_string(),
        odbyto: "2026-02-15".to_string(),
        temat: "standard Kontroli, wersja 3".to_string(),
        potwierdzil: "osoba-wyznaczona".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-01".to_string(),
    };
    rejestr_kontroli_organizacja::wpisz_szkolenie(room, &training).expect("enter the training");
}

// A whole client ----------------------------------------------------------------------------------

/// Rejestr kontroli: klient od weryfikacji przed umową do końca stosunków i powrotu
#[test]
fn whole_client() {
    let room = new_room();

    add_client(
        &room,
        &WpiszKlientaKontroliParams {
            kartoteka_odnosnik: None,
            ..client("nowak")
        },
    );
    identify(&room, &person_identification());
    rejestr_kontroli_sprawdzenia::wpisz_sprawdzenie_pep(&room, &pep())
        .expect("enter the PEP check");
    approve(
        &room,
        &request_about(
            "zatwierdzenie",
            RodzajDecyzji::ZatwierdzenieOceny,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
            "rejestr_kontroli_oceny_klientow",
            "ocena",
        ),
    );
    add_assessment(&room, &assessment("ocena"));
    rejestr_kontroli_oceny::wpisz_zatwierdzenie_oceny(
        &room,
        &WpiszZatwierdzenieOcenyParams {
            zatwierdzenie_decyzja_id: "zatwierdzenie".to_string(),
            id: "ocena".to_string(),
        },
    )
    .expect("enter the approval of the assessment");

    rejestr_kontroli_klienci::wpisz_odnosnik_do_kartoteki(
        &room,
        &WpiszOdnosnikDoKartotekiParams {
            kartoteka_odnosnik: "kartoteka/nowak".to_string(),
            id: "nowak".to_string(),
        },
    )
    .expect("link the client's file");
    rejestr_kontroli_klienci::wpisz_nawiazanie_stosunkow(
        &room,
        &WpiszNawiazanieStosunkowParams {
            stosunki_od: "2026-03-05".to_string(),
            id: "nowak".to_string(),
        },
    )
    .expect("enter the start of the relationship");
    rejestr_kontroli_klienci::wpisz_zakonczenie_stosunkow(
        &room,
        &WpiszZakonczenieStosunkowParams {
            stosunki_do: "2026-10-30".to_string(),
            id: "nowak".to_string(),
        },
    )
    .expect("enter the end of the relationship");

    add_client(
        &room,
        &WpiszKlientaKontroliParams {
            wpisano: "2028-02-01".to_string(),
            ..client("nowak-2028")
        },
    );

    let history = rejestr_kontroli_klienci::stosunki_klienta_z_kartoteki(
        &room,
        &StosunkiKlientaZKartotekiParams {
            kartoteka_odnosnik: "kartoteka/nowak".to_string(),
        },
    )
    .expect("read the relationships of the client's file");
    let relationships: Vec<(&str, Option<&str>)> = history
        .iter()
        .map(|row| (row.id.as_str(), row.stosunki_do.as_deref()))
        .collect();
    assert_eq!(
        relationships,
        [("nowak", Some("2026-10-30")), ("nowak-2028", None)]
    );

    let assessments = rejestr_kontroli_oceny::oceny_ryzyka_klienta(
        &room,
        &OcenyRyzykaKlientaParams {
            klient_id: "nowak".to_string(),
        },
    )
    .expect("read the client's risk assessments");
    assert_eq!(
        assessments[0].zatwierdzenie_decyzja_id.as_deref(),
        Some("zatwierdzenie")
    );
}

// Decisions ---------------------------------------------------------------------------------------

/// Decyzje: prośba powstaje czekająca, rozstrzygnięcie ma pełny podpis tego, kto decyduje
#[test]
fn decision_signature() {
    let room = new_room();

    expect::refused(
        room.execute(
            "INSERT INTO rejestr_kontroli_decyzje_wpisy (id, rodzaj, prosi, decyduje, decyduje_funkcja,
              zlozono, stan)
            VALUES ('od-razu', 'zatwierdzenie_oceny', 'kierownik', 'osoba', 'osoba_wyznaczona',
              '2026-03-01T10:00:00Z', 'rozstrzygnieta')",
            &[],
        ),
        "prośba o decyzję powstaje jako czekająca",
    );
    ask(
        &room,
        &request(
            "prosba",
            RodzajDecyzji::ZatwierdzenieOceny,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
        ),
    );

    let without_signature = RozstrzygnijDecyzjeKontroliParams {
        podpis: None,
        ..designated_consent("prosba")
    };
    expect::refused(
        rejestr_kontroli_decyzje::rozstrzygnij_decyzje_kontroli(&room, &without_signature),
        "podpis_w_komplecie",
    );
    expect::refused(
        room.execute(
            "UPDATE rejestr_kontroli_decyzje_wpisy
            SET rozstrzygniecie = 'zgoda', podpisal = 'osoba-wyznaczona', podpis = 'podpis',
              podpisano = '2026-03-01T12:00:00Z'
            WHERE id = 'prosba'",
            &[],
        ),
        "podpis_tylko_przy_rozstrzygnieciu",
    );
    let signed_by_manager = RozstrzygnijDecyzjeKontroliParams {
        podpisal: Some("kierownik-kontroli".to_string()),
        ..designated_consent("prosba")
    };
    expect::refused(
        rejestr_kontroli_decyzje::rozstrzygnij_decyzje_kontroli(&room, &signed_by_manager),
        "podpisuje_ten_kto_decyduje",
    );
    let signed_before_request = RozstrzygnijDecyzjeKontroliParams {
        podpisano: Some("2026-03-01T09:00:00Z".to_string()),
        ..designated_consent("prosba")
    };
    expect::refused(
        rejestr_kontroli_decyzje::rozstrzygnij_decyzje_kontroli(&room, &signed_before_request),
        "rozstrzygniecie_nie_przed_zlozeniem",
    );

    resolve(
        &room,
        &resolution("prosba", WpisyDecydujeFunkcja::OsobaWyznaczona),
    );

    let waiting = rejestr_kontroli_decyzje::decyzje_kontroli_czekajace(&room)
        .expect("read the waiting decisions");
    assert!(waiting.is_empty());
}

/// Decyzje: rozstrzyga właściwa osoba, a zdjęcie wątpliwości ma uzasadnienie
#[test]
fn decision_right_person() {
    let room = new_room();

    let pep_acceptance_by_designated_person = request(
        "prosba",
        RodzajDecyzji::AkceptacjaPep,
        WpisyDecydujeFunkcja::OsobaWyznaczona,
    );
    expect::refused(
        rejestr_kontroli_decyzje::popros_o_decyzje_kontroli(
            &room,
            &pep_acceptance_by_designated_person,
        ),
        "rozstrzyga_wlasciwa_osoba",
    );
    let approval_by_owner = PoprosODecyzjeKontroliParams {
        decyduje_funkcja: WpisyDecydujeFunkcja::WlascicielBiura,
        ..request(
            "prosba",
            RodzajDecyzji::ZatwierdzenieOceny,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
        )
    };
    expect::refused(
        rejestr_kontroli_decyzje::popros_o_decyzje_kontroli(&room, &approval_by_owner),
        "rozstrzyga_wlasciwa_osoba",
    );
    let table_without_row = PoprosODecyzjeKontroliParams {
        dotyczy_tabela: Some(CLIENTS.to_string()),
        ..request(
            "prosba",
            RodzajDecyzji::ZatwierdzenieOceny,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
        )
    };
    expect::refused(
        rejestr_kontroli_decyzje::popros_o_decyzje_kontroli(&room, &table_without_row),
        "dotyczy_tabela_razem_z_id",
    );
    ask(
        &room,
        &request(
            "nienawiazanie-wlasciciel",
            RodzajDecyzji::NienawiazanieStosunkow,
            WpisyDecydujeFunkcja::WlascicielBiura,
        ),
    );
    ask(
        &room,
        &request(
            "nienawiazanie-osoba",
            RodzajDecyzji::NienawiazanieStosunkow,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
        ),
    );

    ask(
        &room,
        &request(
            "zdjecie",
            RodzajDecyzji::ZdjecieWatpliwosci,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
        ),
    );
    expect::refused(
        rejestr_kontroli_decyzje::rozstrzygnij_decyzje_kontroli(
            &room,
            &designated_consent("zdjecie"),
        ),
        "zdjecie_watpliwosci_z_uzasadnieniem",
    );
    ask(
        &room,
        &request(
            "ocena",
            RodzajDecyzji::ZatwierdzenieOceny,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
        ),
    );
    let approval_with_case_file_decision = RozstrzygnijDecyzjeKontroliParams {
        teczka_decyzja_odnosnik: Some("teczka/decyzja".to_string()),
        ..designated_consent("ocena")
    };
    expect::refused(
        rejestr_kontroli_decyzje::rozstrzygnij_decyzje_kontroli(
            &room,
            &approval_with_case_file_decision,
        ),
        "decyzja_w_teczce_tylko_przy_zdjeciu_watpliwosci",
    );
    resolve(
        &room,
        &RozstrzygnijDecyzjeKontroliParams {
            uzasadnienie: Some("źródło środków wyjaśnione umową darowizny".to_string()),
            teczka_decyzja_odnosnik: Some("teczka/decyzja".to_string()),
            ..resolution("zdjecie", WpisyDecydujeFunkcja::OsobaWyznaczona)
        },
    );
}

/// Decyzje: prośby się nie zmienia, zbędna ma powód, rozstrzygnięta jest ostateczna
#[test]
fn decision_final() {
    let room = new_room();
    ask(
        &room,
        &request(
            "prosba",
            RodzajDecyzji::ZatwierdzenieOceny,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
        ),
    );

    expect::refused(
        room.execute(
            "UPDATE rejestr_kontroli_decyzje_wpisy SET tresc = 'inna' WHERE id = 'prosba'",
            &[],
        ),
        "prośby o decyzję się nie zmienia",
    );
    expect::refused(
        room.execute(
            "UPDATE rejestr_kontroli_decyzje_wpisy SET stan = 'zbedna' WHERE id = 'prosba'",
            &[],
        ),
        "zbedna_ma_powod",
    );
    rejestr_kontroli_decyzje::uznaj_prosbe_kontroli_za_zbedna(
        &room,
        &UznajProsbeKontroliZaZbednaParams {
            powod_zbednosci: "klient zrezygnował".to_string(),
            id: "prosba".to_string(),
        },
    )
    .expect("mark the request as unneeded");
    expect::refused(
        rejestr_kontroli_decyzje::rozstrzygnij_decyzje_kontroli(
            &room,
            &designated_consent("prosba"),
        ),
        "rozstrzygniętej ani zbędnej decyzji się nie zmienia",
    );

    approve(
        &room,
        &request(
            "druga",
            RodzajDecyzji::ZatwierdzenieOceny,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
        ),
    );
    let refusal_after_consent = RozstrzygnijDecyzjeKontroliParams {
        rozstrzygniecie: Some(WpisyRozstrzygniecie::Odmowa),
        ..designated_consent("druga")
    };
    expect::refused(
        rejestr_kontroli_decyzje::rozstrzygnij_decyzje_kontroli(&room, &refusal_after_consent),
        "rozstrzygniętej ani zbędnej decyzji się nie zmienia",
    );
}

// Clients and cases -------------------------------------------------------------------------------

/// Klienci: jedne trwające stosunki; powrót klienta to nowy wpis po zakończeniu poprzednich
#[test]
fn client_relationships() {
    let room = new_room();
    add_client(&room, &client("nowak"));
    let ongoing = "klient ma już trwające stosunki";

    expect::refused(
        rejestr_kontroli_klienci::wpisz_klienta_kontroli(&room, &client("nowak-2")),
        ongoing,
    );
    add_client(
        &room,
        &WpiszKlientaKontroliParams {
            kartoteka_odnosnik: None,
            ..client("bez-kartoteki")
        },
    );
    let link_to_taken_file = WpiszOdnosnikDoKartotekiParams {
        kartoteka_odnosnik: "kartoteka/nowak".to_string(),
        id: "bez-kartoteki".to_string(),
    };
    expect::refused(
        rejestr_kontroli_klienci::wpisz_odnosnik_do_kartoteki(&room, &link_to_taken_file),
        ongoing,
    );

    let end_before_start = WpiszZakonczenieStosunkowParams {
        stosunki_do: "2026-10-30".to_string(),
        id: "nowak".to_string(),
    };
    expect::refused(
        rejestr_kontroli_klienci::wpisz_zakonczenie_stosunkow(&room, &end_before_start),
        "stosunki_konczy_sie_po_nawiazaniu",
    );
    rejestr_kontroli_klienci::wpisz_nawiazanie_stosunkow(
        &room,
        &WpiszNawiazanieStosunkowParams {
            stosunki_od: "2026-03-05".to_string(),
            id: "nowak".to_string(),
        },
    )
    .expect("enter the start of the relationship");
    let end_before_beginning = WpiszZakonczenieStosunkowParams {
        stosunki_do: "2026-03-01".to_string(),
        id: "nowak".to_string(),
    };
    expect::refused(
        rejestr_kontroli_klienci::wpisz_zakonczenie_stosunkow(&room, &end_before_beginning),
        "stosunki_konczy_sie_po_nawiazaniu",
    );
    rejestr_kontroli_klienci::wpisz_zakonczenie_stosunkow(
        &room,
        &WpiszZakonczenieStosunkowParams {
            stosunki_do: "2026-10-30".to_string(),
            id: "nowak".to_string(),
        },
    )
    .expect("enter the end of the relationship");
    add_client(&room, &client("nowak-2"));

    let unchanged = "wpisu klienta się nie zmienia";
    let second_end = WpiszZakonczenieStosunkowParams {
        stosunki_do: "2026-11-30".to_string(),
        id: "nowak".to_string(),
    };
    expect::refused(
        rejestr_kontroli_klienci::wpisz_zakonczenie_stosunkow(&room, &second_end),
        unchanged,
    );
    let second_start = WpiszNawiazanieStosunkowParams {
        stosunki_od: "2026-03-06".to_string(),
        id: "nowak".to_string(),
    };
    expect::refused(
        rejestr_kontroli_klienci::wpisz_nawiazanie_stosunkow(&room, &second_start),
        unchanged,
    );
    let other_file = WpiszOdnosnikDoKartotekiParams {
        kartoteka_odnosnik: "kartoteka/inna".to_string(),
        id: "nowak".to_string(),
    };
    expect::refused(
        rejestr_kontroli_klienci::wpisz_odnosnik_do_kartoteki(&room, &other_file),
        unchanged,
    );
    expect::refused(
        room.execute(
            &format!("UPDATE {CLIENTS} SET kontroler = 'inny' WHERE id = 'nowak'"),
            &[],
        ),
        unchanged,
    );
}

/// Sprawy: przy najmie czynsz z kursem i dniem kursu, a próg 10 000 euro rozstrzyga objęcie
#[test]
fn case_qualification() {
    let room = new_room();
    add_client(&room, &client("nowak"));

    // Every qualification below names `objeta` itself; the value here only fills the field.
    let rent = ZakwalifikujSpraweParams {
        klient_id: "nowak".to_string(),
        teczka_odnosnik: "teczka-najmu".to_string(),
        rodzaj_sprawy: SprawyRodzajSprawy::PosrednictwoNajmu,
        objeta: SprawyObjeta::Nie,
        podstawa: "art. 2 ust. 1 pkt 18 AML".to_string(),
        czynsz: None,
        kurs_euro: None,
        dzien_kursu: None,
        zakwalifikowano: "2026-03-01".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
    };

    let without_rent = ZakwalifikujSpraweParams {
        objeta: SprawyObjeta::Nie,
        ..rent.clone()
    };
    expect::refused(
        rejestr_kontroli_klienci::zakwalifikuj_sprawe(&room, &without_rent),
        "najem_ma_czynsz",
    );
    let rent_without_rate = ZakwalifikujSpraweParams {
        objeta: SprawyObjeta::Nie,
        czynsz: Some(500_000),
        ..rent.clone()
    };
    expect::refused(
        rejestr_kontroli_klienci::zakwalifikuj_sprawe(&room, &rent_without_rate),
        "czynsz_z_kursem_i_dniem",
    );
    let low_rent_covered = ZakwalifikujSpraweParams {
        objeta: SprawyObjeta::Tak,
        czynsz: Some(500_000),
        kurs_euro: Some(4.25),
        dzien_kursu: Some("2026-02-27".to_string()),
        ..rent.clone()
    };
    expect::refused(
        rejestr_kontroli_klienci::zakwalifikuj_sprawe(&room, &low_rent_covered),
        "prog_najmu",
    );
    let high_rent_not_covered = ZakwalifikujSpraweParams {
        objeta: SprawyObjeta::Nie,
        czynsz: Some(4_500_000),
        kurs_euro: Some(4.25),
        dzien_kursu: Some("2026-02-27".to_string()),
        ..rent.clone()
    };
    expect::refused(
        rejestr_kontroli_klienci::zakwalifikuj_sprawe(&room, &high_rent_not_covered),
        "prog_najmu",
    );

    rejestr_kontroli_klienci::zakwalifikuj_sprawe(
        &room,
        &ZakwalifikujSpraweParams {
            objeta: SprawyObjeta::Nie,
            czynsz: Some(500_000),
            kurs_euro: Some(4.25),
            dzien_kursu: Some("2026-02-27".to_string()),
            ..rent.clone()
        },
    )
    .expect("qualify the rent below the threshold");
    rejestr_kontroli_klienci::zakwalifikuj_sprawe(
        &room,
        &ZakwalifikujSpraweParams {
            objeta: SprawyObjeta::Tak,
            czynsz: Some(4_500_000),
            kurs_euro: Some(4.25),
            dzien_kursu: Some("2026-02-27".to_string()),
            ..rent.clone()
        },
    )
    .expect("qualify the rent above the threshold");
    rejestr_kontroli_klienci::zakwalifikuj_sprawe(
        &room,
        &ZakwalifikujSpraweParams {
            rodzaj_sprawy: SprawyRodzajSprawy::Zarzadzanie,
            teczka_odnosnik: "teczka-lokalu".to_string(),
            objeta: SprawyObjeta::DoRozstrzygniecia,
            ..rent
        },
    )
    .expect("qualify the management case");

    let cases = rejestr_kontroli_klienci::sprawy_klienta_kontroli(
        &room,
        &SprawyKlientaKontroliParams {
            klient_id: "nowak".to_string(),
        },
    )
    .expect("read the client's cases");
    assert_eq!(cases.len(), 3);

    expect::refused(
        room.execute(
            "UPDATE rejestr_kontroli_klienci_sprawy SET objeta = 'tak'",
            &[],
        ),
        "kwalifikacji sprawy się nie zmienia",
    );
}

// Checks ------------------------------------------------------------------------------------------

/// Identyfikacja: dane z art. 36 AML osoby, podmiotu, beneficjenta i upoważnionego
#[test]
fn identification_data() {
    let room = new_room();
    add_client(&room, &client("nowak"));
    let person = person_identification();
    let entity = WpiszIdentyfikacjeParams {
        rodzaj_podmiotu: IdentyfikacjeRodzajPodmiotu::Podmiot,
        imiona: None,
        nazwisko: None,
        obywatelstwo: None,
        pesel: None,
        dokument_tozsamosci: None,
        nazwa: Some("Leśna Inwestycje sp. z o.o.".to_string()),
        forma_organizacyjna: Some("spółka z ograniczoną odpowiedzialnością".to_string()),
        adres_siedziby: Some("ul. Leśna 1, Poznań".to_string()),
        ..person.clone()
    };

    let refusals = [
        (
            "osoba_ma_imie_i_nazwisko",
            WpiszIdentyfikacjeParams {
                nazwisko: None,
                ..person.clone()
            },
        ),
        (
            "osoba_ma_obywatelstwo_pesel_albo_urodzenie_i_dokument",
            WpiszIdentyfikacjeParams {
                obywatelstwo: None,
                ..person.clone()
            },
        ),
        (
            "osoba_ma_obywatelstwo_pesel_albo_urodzenie_i_dokument",
            WpiszIdentyfikacjeParams {
                pesel: None,
                data_urodzenia: Some("1980-01-01".to_string()),
                ..person.clone()
            },
        ),
        (
            "osoba_ma_obywatelstwo_pesel_albo_urodzenie_i_dokument",
            WpiszIdentyfikacjeParams {
                dokument_tozsamosci: None,
                ..person.clone()
            },
        ),
        (
            "pesel_ma_11_cyfr",
            WpiszIdentyfikacjeParams {
                pesel: Some("8001011234".to_string()),
                ..person.clone()
            },
        ),
        (
            "podmiotem_bywa_tylko_klient",
            WpiszIdentyfikacjeParams {
                kogo: IdentyfikacjeKogo::Beneficjent,
                ..entity.clone()
            },
        ),
        (
            "podmiot_ma_nazwe_forme_i_siedzibe",
            WpiszIdentyfikacjeParams {
                adres_siedziby: None,
                nip: Some("5250000000".to_string()),
                ..entity.clone()
            },
        ),
        ("podmiot_ma_nip_albo_rejestr", entity.clone()),
        (
            "upowazniony_ma_dokument_umocowania",
            WpiszIdentyfikacjeParams {
                kogo: IdentyfikacjeKogo::Upowazniony,
                ..person.clone()
            },
        ),
    ];
    for (rule, params) in refusals {
        expect::refused(
            rejestr_kontroli_sprawdzenia::wpisz_identyfikacje(&room, &params),
            rule,
        );
    }

    identify(&room, &person);
    identify(
        &room,
        &WpiszIdentyfikacjeParams {
            pesel: None,
            data_urodzenia: Some("1980-01-01".to_string()),
            panstwo_urodzenia: Some("UA".to_string()),
            obywatelstwo: Some("UA".to_string()),
            ..person.clone()
        },
    );
    identify(
        &room,
        &WpiszIdentyfikacjeParams {
            kogo: IdentyfikacjeKogo::Beneficjent,
            obywatelstwo: None,
            pesel: None,
            dokument_tozsamosci: None,
            ..person.clone()
        },
    );
    identify(
        &room,
        &WpiszIdentyfikacjeParams {
            kogo: IdentyfikacjeKogo::Upowazniony,
            umocowanie_dokument: Some("teczka/pelnomocnictwo".to_string()),
            ..person
        },
    );
    identify(
        &room,
        &WpiszIdentyfikacjeParams {
            nip: Some("5250000000".to_string()),
            ..entity.clone()
        },
    );
    identify(
        &room,
        &WpiszIdentyfikacjeParams {
            rejestr_panstwo: Some("DE".to_string()),
            rejestr_nazwa: Some("Handelsregister".to_string()),
            rejestr_numer: Some("HRB 1234".to_string()),
            rejestr_data: Some("2020-05-01".to_string()),
            ..entity
        },
    );

    let identifications = rejestr_kontroli_sprawdzenia::identyfikacje_klienta(
        &room,
        &IdentyfikacjeKlientaParams {
            klient_id: "nowak".to_string(),
        },
    )
    .expect("read the identifications");
    assert_eq!(identifications.len(), 6);
}

/// Ponowne sprawdzenie: wskazuje aktualny wpis tego samego klienta albo wiersza i go zastępuje
#[test]
fn recheck_replaces_previous() {
    let room = new_room();
    full_register(&room);
    add_client(
        &room,
        &WpiszKlientaKontroliParams {
            kartoteka_odnosnik: Some("kartoteka/lis".to_string()),
            ..client("lis")
        },
    );

    for (table, owner) in RECHECKED_TABLES {
        let first_id = number_of(&room, &format!("SELECT min(id) FROM {table}"));
        let columns: Vec<String> = column_names_of(&room, table)
            .into_iter()
            .filter(|name| !["id", "aktualne", "poprzedni_id"].contains(&name.as_str()))
            .collect();
        let column_list = columns.join(", ");
        let recheck = format!(
            "INSERT INTO {table} ({column_list}, aktualne, poprzedni_id) \
            SELECT {column_list}, 1, :poprzedni_id FROM {table} WHERE id = :poprzedni_id"
        );
        let recheck_of_first = [("poprzedni_id", SqlValue::Integer(first_id))];

        let past_entry = format!(
            "INSERT INTO {table} ({column_list}, aktualne) \
            SELECT {column_list}, 0 FROM {table} WHERE id = {first_id}"
        );
        expect::refused(
            room.execute(&past_entry, &[]),
            "wpis sprawdzenia powstaje jako aktualny",
        );
        expect::refused(
            room.execute(
                &format!("UPDATE {table} SET aktualne = 0 WHERE id = {first_id}"),
                &[],
            ),
            CHECK_UNCHANGED,
        );
        expect::refused(
            room.execute(
                &format!("UPDATE {table} SET wpisano = '2026-12-31' WHERE id = {first_id}"),
                &[],
            ),
            CHECK_UNCHANGED,
        );

        room.execute(&recheck, &recheck_of_first)
            .expect("re-check the first entry");
        let first_is_current = number_of(
            &room,
            &format!("SELECT aktualne FROM {table} WHERE id = {first_id}"),
        );
        assert_eq!(first_is_current, 0, "{table}");
        expect::refused(
            room.execute(&recheck, &recheck_of_first),
            "ponowne sprawdzenie wskazuje aktualny wpis",
        );
        expect::refused(
            room.execute(
                &format!("UPDATE {table} SET aktualne = 1 WHERE id = {first_id}"),
                &[],
            ),
            CHECK_UNCHANGED,
        );

        if owner == "klient_id" {
            let latest_id = number_of(&room, &format!("SELECT max(id) FROM {table}"));
            let columns_for_other_client = column_list.replace("klient_id", "'lis'");
            let recheck_for_other_client = format!(
                "INSERT INTO {table} ({column_list}, aktualne, poprzedni_id) \
                SELECT {columns_for_other_client}, 1, {latest_id} FROM {table} \
                WHERE id = {latest_id}"
            );
            expect::refused(
                room.execute(&recheck_for_other_client, &[]),
                "ponowne sprawdzenie wskazuje aktualny wpis",
            );
        }
    }
}

/// Weryfikacja, CRBR, analiza i sprawdzenie spoza ustawy: wyniki z opisem i odnośnikami
#[test]
fn check_results() {
    let room = new_room();
    add_client(&room, &client("nowak"));
    let identification_id = identify(&room, &person_identification());

    let remarks_without_description = WpiszWeryfikacjeParams {
        wynik: PozaUstawaWynik::Uwagi,
        ..verification(identification_id)
    };
    expect::refused(
        rejestr_kontroli_sprawdzenia::wpisz_weryfikacje(&room, &remarks_without_description),
        "wynik_z_uwagami_ma_opis",
    );
    let crbr = WpiszOdpisCrbrParams {
        klient_id: "nowak".to_string(),
        odpis_sciezka: "crbr.pdf".to_string(),
        odpis_skrot: "sha256".to_string(),
        rozbieznosc: false,
        rozbieznosc_opis: None,
        wyjasnienie: None,
        zrodlo: "dowód osobisty".to_string(),
        raport_odnosnik: "raport-kontrolera/1".to_string(),
        sprawdzil: "kontroler-nowak".to_string(),
        sprawdzono: "2026-03-01".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-01".to_string(),
        poprzedni_id: None,
    };
    let discrepancy_without_description = WpiszOdpisCrbrParams {
        rozbieznosc: true,
        ..crbr.clone()
    };
    expect::refused(
        rejestr_kontroli_sprawdzenia::wpisz_odpis_crbr(&room, &discrepancy_without_description),
        "rozbieznosc_ma_opis",
    );
    let explanation_without_discrepancy = WpiszOdpisCrbrParams {
        rozbieznosc: false,
        wyjasnienie: Some("brak".to_string()),
        ..crbr.clone()
    };
    expect::refused(
        rejestr_kontroli_sprawdzenia::wpisz_odpis_crbr(&room, &explanation_without_discrepancy),
        "wyjasnienie_tylko_przy_rozbieznosci",
    );
    rejestr_kontroli_sprawdzenia::wpisz_odpis_crbr(
        &room,
        &WpiszOdpisCrbrParams {
            rozbieznosc: true,
            rozbieznosc_opis: Some("inny beneficjent w CRBR".to_string()),
            wyjasnienie: Some("zgłoszenie nieaktualne".to_string()),
            ..crbr
        },
    )
    .expect("enter the CRBR extract with a discrepancy");
    let analysis_without_description = WpiszAnalizeParams {
        klient_id: "nowak".to_string(),
        teczka_odnosnik: "teczka".to_string(),
        analiza: "zapłata gotówką".to_string(),
        wynik: MonitorowanieWynik::Uwagi,
        opis: None,
        zrodlo: "dowód osobisty".to_string(),
        raport_odnosnik: "raport-kontrolera/1".to_string(),
        sprawdzil: "kontroler-nowak".to_string(),
        sprawdzono: "2026-03-01".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-01".to_string(),
    };
    expect::refused(
        rejestr_kontroli_sprawdzenia::wpisz_analize(&room, &analysis_without_description),
        "wynik_z_uwagami_ma_opis",
    );

    let account_with_row_version = WpiszSprawdzeniePozaUstawaParams {
        wersja_wiersza: Some(1),
        ..out_of_act()
    };
    expect::refused(
        rejestr_kontroli_sprawdzenia::wpisz_sprawdzenie_poza_ustawa(
            &room,
            &account_with_row_version,
        ),
        "wersja_wiersza_przy_wierszu_z_wersja",
    );
    let identity_without_row_version = WpiszSprawdzeniePozaUstawaParams {
        rodzaj: PozaUstawaRodzaj::Tozsamosc,
        wyplata_odnosnik: None,
        ..out_of_act()
    };
    expect::refused(
        rejestr_kontroli_sprawdzenia::wpisz_sprawdzenie_poza_ustawa(
            &room,
            &identity_without_row_version,
        ),
        "wersja_wiersza_przy_wierszu_z_wersja",
    );
    let identity_with_payout = WpiszSprawdzeniePozaUstawaParams {
        rodzaj: PozaUstawaRodzaj::Tozsamosc,
        wersja_wiersza: Some(2),
        ..out_of_act()
    };
    expect::refused(
        rejestr_kontroli_sprawdzenia::wpisz_sprawdzenie_poza_ustawa(&room, &identity_with_payout),
        "wyplata_tylko_przy_rachunku",
    );
    rejestr_kontroli_sprawdzenia::wpisz_sprawdzenie_poza_ustawa(&room, &out_of_act())
        .expect("enter the check of the account");
    rejestr_kontroli_sprawdzenia::wpisz_sprawdzenie_poza_ustawa(
        &room,
        &WpiszSprawdzeniePozaUstawaParams {
            klient_id: None,
            rodzaj: PozaUstawaRodzaj::Tytul,
            wiersz_odnosnik: "teczka-najmu/uczestnik-wynajmujacy".to_string(),
            wersja_wiersza: Some(1),
            wyplata_odnosnik: None,
            wynik: PozaUstawaWynik::Uwagi,
            opis: Some("w księdze inny właściciel".to_string()),
            ..out_of_act()
        },
    )
    .expect("enter the check of the title");

    let checks = rejestr_kontroli_sprawdzenia::sprawdzenia_poza_ustawa_teczki(
        &room,
        &SprawdzeniaPozaUstawaTeczkiParams {
            teczka_odnosnik: "teczka-najmu".to_string(),
        },
    )
    .expect("read the checks of the case file");
    assert_eq!(checks.len(), 2);
}

/// PEP: oświadczenie z klauzulą, źródło majątku i akceptacja właściciela o tym kliencie
#[test]
fn politically_exposed() {
    let room = new_room();
    add_client(&room, &client("nowak"));
    add_client(
        &room,
        &WpiszKlientaKontroliParams {
            kartoteka_odnosnik: Some("kartoteka/lis".to_string()),
            ..client("lis")
        },
    );
    let accepted = "akceptacja przy eksponowanym stanowisku wskazuje zatwierdzoną decyzję \
        akceptacja_pep o tym kliencie";
    let exposed = WpiszSprawdzeniePepParams {
        wynik: PepWynik::Pep,
        zrodlo_majatku: Some("wynagrodzenie posła i spadek".to_string()),
        ..pep()
    };

    let statement_without_clause = WpiszSprawdzeniePepParams {
        oswiadczenie_z_klauzula: false,
        ..pep()
    };
    expect::refused(
        rejestr_kontroli_sprawdzenia::wpisz_sprawdzenie_pep(&room, &statement_without_clause),
        "oswiadczenie_z_klauzula",
    );
    let family_member_without_source = WpiszSprawdzeniePepParams {
        wynik: PepWynik::CzlonekRodziny,
        ..pep()
    };
    expect::refused(
        rejestr_kontroli_sprawdzenia::wpisz_sprawdzenie_pep(&room, &family_member_without_source),
        "pep_ma_zrodlo_majatku",
    );
    rejestr_kontroli_sprawdzenia::wpisz_sprawdzenie_pep(
        &room,
        &WpiszSprawdzeniePepParams {
            oswiadczenie_sciezka: None,
            oswiadczenie_skrot: None,
            oswiadczenie_z_klauzula: false,
            ..pep()
        },
    )
    .expect("enter the PEP check without a statement");
    rejestr_kontroli_sprawdzenia::wpisz_sprawdzenie_pep(&room, &exposed)
        .expect("enter the PEP check of an exposed person");

    let pep_checks = rejestr_kontroli_sprawdzenia::sprawdzenia_pep_klienta(
        &room,
        &SprawdzeniaPepKlientaParams {
            klient_id: "nowak".to_string(),
        },
    )
    .expect("read the PEP checks");
    let pep_id = pep_checks[0].id;
    let no_pep_id = pep_checks[1].id;

    approve(
        &room,
        &request_about(
            "akceptacja-lis",
            RodzajDecyzji::AkceptacjaPep,
            WpisyDecydujeFunkcja::WlascicielBiura,
            CLIENTS,
            "lis",
        ),
    );
    ask(
        &room,
        &request_about(
            "czekajaca",
            RodzajDecyzji::AkceptacjaPep,
            WpisyDecydujeFunkcja::WlascicielBiura,
            CLIENTS,
            "nowak",
        ),
    );
    approve(
        &room,
        &request_about(
            "akceptacja",
            RodzajDecyzji::AkceptacjaPep,
            WpisyDecydujeFunkcja::WlascicielBiura,
            CLIENTS,
            "nowak",
        ),
    );
    let acceptance_about_other_client = WpiszAkceptacjePepParams {
        akceptacja_decyzja_id: "akceptacja-lis".to_string(),
        id: pep_id,
    };
    expect::refused(
        rejestr_kontroli_sprawdzenia::wpisz_akceptacje_pep(&room, &acceptance_about_other_client),
        accepted,
    );
    let waiting_acceptance = WpiszAkceptacjePepParams {
        akceptacja_decyzja_id: "czekajaca".to_string(),
        id: pep_id,
    };
    expect::refused(
        rejestr_kontroli_sprawdzenia::wpisz_akceptacje_pep(&room, &waiting_acceptance),
        accepted,
    );
    let acceptance_without_pep = WpiszAkceptacjePepParams {
        akceptacja_decyzja_id: "akceptacja".to_string(),
        id: no_pep_id,
    };
    expect::refused(
        rejestr_kontroli_sprawdzenia::wpisz_akceptacje_pep(&room, &acceptance_without_pep),
        "akceptacja_tylko_przy_pep",
    );
    expect::refused(
        room.execute(
            "INSERT INTO rejestr_kontroli_sprawdzenia_pep (klient_id, dotyczy, osoba,
              wykaz_sprawdzono, wynik, zrodlo_majatku, akceptacja_decyzja_id, zrodlo,
              raport_odnosnik, sprawdzil, sprawdzono, wpisal, wpisano, aktualne)
            VALUES ('nowak', 'klient', 'Anna Nowak', '2026-03-01', 'pep', 'spadek',
              'akceptacja-lis', 'wykaz', 'raport', 'kontroler', '2026-03-01', 'kierownik',
              '2026-03-01', 1)",
            &[],
        ),
        accepted,
    );

    rejestr_kontroli_sprawdzenia::wpisz_akceptacje_pep(
        &room,
        &WpiszAkceptacjePepParams {
            akceptacja_decyzja_id: "akceptacja".to_string(),
            id: pep_id,
        },
    )
    .expect("enter the owner's acceptance");
    approve(
        &room,
        &request_about(
            "druga-akceptacja",
            RodzajDecyzji::AkceptacjaPep,
            WpisyDecydujeFunkcja::WlascicielBiura,
            CLIENTS,
            "nowak",
        ),
    );
    let second_acceptance = WpiszAkceptacjePepParams {
        akceptacja_decyzja_id: "druga-akceptacja".to_string(),
        id: pep_id,
    };
    expect::refused(
        rejestr_kontroli_sprawdzenia::wpisz_akceptacje_pep(&room, &second_acceptance),
        CHECK_UNCHANGED,
    );
}

/// Sankcje: zamrożenie tylko przy trafieniu, informacja po zamrożeniu, każde raz
#[test]
fn sanctions() {
    let room = new_room();
    add_client(&room, &client("nowak"));
    let sanction = WpiszSprawdzenieSankcjiParams {
        klient_id: "nowak".to_string(),
        listy: "ONZ, UE, lista GIIF".to_string(),
        wynik: SankcjeWynik::BrakTrafien,
        zrodlo: "dowód osobisty".to_string(),
        raport_odnosnik: "raport-kontrolera/1".to_string(),
        sprawdzil: "kontroler-nowak".to_string(),
        sprawdzono: "2026-03-01".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-01".to_string(),
        poprzedni_id: None,
    };
    let of_client = SprawdzeniaSankcjiKlientaParams {
        klient_id: "nowak".to_string(),
    };

    rejestr_kontroli_sprawdzenia::wpisz_sprawdzenie_sankcji(&room, &sanction)
        .expect("enter the sanctions check without hits");
    let clear_checks = rejestr_kontroli_sprawdzenia::sprawdzenia_sankcji_klienta(&room, &of_client)
        .expect("read the sanctions checks");
    let clear_id = clear_checks[0].id;
    let freeze_without_hit = WpiszZamrozenieParams {
        zamrozono: "2026-03-02T10:00:00Z".to_string(),
        id: clear_id,
    };
    expect::refused(
        rejestr_kontroli_sprawdzenia::wpisz_zamrozenie(&room, &freeze_without_hit),
        "zamrozenie_tylko_przy_trafieniu",
    );

    rejestr_kontroli_sprawdzenia::wpisz_sprawdzenie_sankcji(
        &room,
        &WpiszSprawdzenieSankcjiParams {
            wynik: SankcjeWynik::Trafienie,
            ..sanction
        },
    )
    .expect("enter the sanctions check with a hit");
    let hit_checks = rejestr_kontroli_sprawdzenia::sprawdzenia_sankcji_klienta(&room, &of_client)
        .expect("read the sanctions checks");
    let hit_id = hit_checks[0].id;
    let information_before_freeze = WpiszInformacjeOZamrozeniuParams {
        zamrozenie_informacja: "2026-03-03T10:00:00Z".to_string(),
        id: hit_id,
    };
    expect::refused(
        rejestr_kontroli_sprawdzenia::wpisz_informacje_o_zamrozeniu(
            &room,
            &information_before_freeze,
        ),
        "informacja_po_zamrozeniu",
    );
    rejestr_kontroli_sprawdzenia::wpisz_zamrozenie(
        &room,
        &WpiszZamrozenieParams {
            zamrozono: "2026-03-02T10:00:00Z".to_string(),
            id: hit_id,
        },
    )
    .expect("enter the freeze");
    let information_earlier_than_freeze = WpiszInformacjeOZamrozeniuParams {
        zamrozenie_informacja: "2026-03-01T10:00:00Z".to_string(),
        id: hit_id,
    };
    expect::refused(
        rejestr_kontroli_sprawdzenia::wpisz_informacje_o_zamrozeniu(
            &room,
            &information_earlier_than_freeze,
        ),
        "informacja_po_zamrozeniu",
    );
    rejestr_kontroli_sprawdzenia::wpisz_informacje_o_zamrozeniu(
        &room,
        &WpiszInformacjeOZamrozeniuParams {
            zamrozenie_informacja: "2026-03-03T10:00:00Z".to_string(),
            id: hit_id,
        },
    )
    .expect("enter the information about the freeze");

    let second_freeze = WpiszZamrozenieParams {
        zamrozono: "2026-03-02T11:00:00Z".to_string(),
        id: hit_id,
    };
    expect::refused(
        rejestr_kontroli_sprawdzenia::wpisz_zamrozenie(&room, &second_freeze),
        CHECK_UNCHANGED,
    );
    let second_information = WpiszInformacjeOZamrozeniuParams {
        zamrozenie_informacja: "2026-03-04T10:00:00Z".to_string(),
        id: hit_id,
    };
    expect::refused(
        rejestr_kontroli_sprawdzenia::wpisz_informacje_o_zamrozeniu(&room, &second_information),
        CHECK_UNCHANGED,
    );
}

/// Kopie, informacje i analizy się nie zmieniają
#[test]
fn immutable_check_entries() {
    let room = new_room();
    full_register(&room);

    for table in [
        "rejestr_kontroli_sprawdzenia_kopie",
        "rejestr_kontroli_sprawdzenia_monitorowanie",
        "rejestr_kontroli_sprawdzenia_informacje",
    ] {
        expect::refused(
            room.execute(&format!("UPDATE {table} SET wpisano = '2026-12-31'"), &[]),
            CHECK_UNCHANGED,
        );
    }
}

// Risk assessments --------------------------------------------------------------------------------

/// Ocena klienta: środki pasują do poziomu, nowa ocena odbiera aktualność poprzedniej
#[test]
fn client_assessment() {
    let room = new_room();
    add_client(&room, &client("nowak"));

    let simplified_at_medium = WpiszOceneRyzykaKlientaParams {
        poziom: KlientowPoziom::Srednie,
        czynniki: "osoba fizyczna, rezydent".to_string(),
        srodki: KlientowSrodki::Uproszczone,
        ..assessment("ocena")
    };
    expect::refused(
        rejestr_kontroli_oceny::wpisz_ocene_ryzyka_klienta(&room, &simplified_at_medium),
        "uproszczone_tylko_przy_niskim",
    );
    let usual_at_high = WpiszOceneRyzykaKlientaParams {
        poziom: KlientowPoziom::Wysokie,
        czynniki: "osoba fizyczna, rezydent".to_string(),
        srodki: KlientowSrodki::Zwykle,
        ..assessment("ocena")
    };
    expect::refused(
        rejestr_kontroli_oceny::wpisz_ocene_ryzyka_klienta(&room, &usual_at_high),
        "wzmozone_przy_wysokim",
    );
    expect::refused(
        room.execute(
            "INSERT INTO rejestr_kontroli_oceny_klientow (id, klient_id, poziom, czynniki, srodki,
              zaproponowal, raport_odnosnik, wpisal, wpisano, aktualna)
            VALUES ('stara', 'nowak', 'niskie', 'czynniki', 'zwykle', 'kontroler', 'raport',
              'kierownik', '2026-03-02', 0)",
            &[],
        ),
        "ocena ryzyka klienta powstaje jako aktualna",
    );

    add_assessment(
        &room,
        &WpiszOceneRyzykaKlientaParams {
            srodki: KlientowSrodki::Uproszczone,
            ..assessment("pierwsza")
        },
    );
    expect::refused(
        room.execute(
            "UPDATE rejestr_kontroli_oceny_klientow SET aktualna = 0 WHERE id = 'pierwsza'",
            &[],
        ),
        "oceny ryzyka klienta się nie zmienia",
    );
    add_assessment(
        &room,
        &WpiszOceneRyzykaKlientaParams {
            poziom: KlientowPoziom::Wysokie,
            srodki: KlientowSrodki::Wzmozone,
            wpisano: "2026-06-01".to_string(),
            ..assessment("druga")
        },
    );

    let assessments = rejestr_kontroli_oceny::oceny_ryzyka_klienta(
        &room,
        &OcenyRyzykaKlientaParams {
            klient_id: "nowak".to_string(),
        },
    )
    .expect("read the client's risk assessments");
    let current: Vec<&str> = assessments
        .iter()
        .filter(|row| row.aktualna)
        .map(|row| row.id.as_str())
        .collect();
    assert_eq!(current, ["druga"]);

    expect::refused(
        room.execute(
            "UPDATE rejestr_kontroli_oceny_klientow SET poziom = 'srednie' WHERE id = 'druga'",
            &[],
        ),
        "oceny ryzyka klienta się nie zmienia",
    );
}

/// Ocena klienta: zatwierdza raz aktualną ocenę zatwierdzona decyzja o tej ocenie
#[test]
fn client_assessment_approval() {
    let room = new_room();
    add_client(&room, &client("nowak"));
    add_assessment(&room, &assessment("pierwsza"));
    let rule = "zatwierdzenie oceny wskazuje zatwierdzoną decyzję zatwierdzenie_oceny o tej ocenie";

    approve(
        &room,
        &request_about(
            "o-innej",
            RodzajDecyzji::ZatwierdzenieOceny,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
            "rejestr_kontroli_oceny_klientow",
            "inna",
        ),
    );
    ask(
        &room,
        &request_about(
            "czekajaca",
            RodzajDecyzji::ZatwierdzenieOceny,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
            "rejestr_kontroli_oceny_klientow",
            "pierwsza",
        ),
    );
    ask(
        &room,
        &request_about(
            "zla",
            RodzajDecyzji::ZdjecieWatpliwosci,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
            "rejestr_kontroli_oceny_klientow",
            "pierwsza",
        ),
    );
    resolve(
        &room,
        &RozstrzygnijDecyzjeKontroliParams {
            uzasadnienie: Some("x".to_string()),
            ..resolution("zla", WpisyDecydujeFunkcja::OsobaWyznaczona)
        },
    );
    ask(
        &room,
        &request_about(
            "odmowa",
            RodzajDecyzji::ZatwierdzenieOceny,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
            "rejestr_kontroli_oceny_klientow",
            "pierwsza",
        ),
    );
    resolve(
        &room,
        &RozstrzygnijDecyzjeKontroliParams {
            rozstrzygniecie: Some(WpisyRozstrzygniecie::Odmowa),
            ..resolution("odmowa", WpisyDecydujeFunkcja::OsobaWyznaczona)
        },
    );
    for decision_id in ["o-innej", "czekajaca", "zla", "odmowa"] {
        let approval = WpiszZatwierdzenieOcenyParams {
            zatwierdzenie_decyzja_id: decision_id.to_string(),
            id: "pierwsza".to_string(),
        };
        expect::refused(
            rejestr_kontroli_oceny::wpisz_zatwierdzenie_oceny(&room, &approval),
            rule,
        );
    }
    expect::refused(
        room.execute(
            "INSERT INTO rejestr_kontroli_oceny_klientow (id, klient_id, poziom, czynniki, srodki,
              zaproponowal, raport_odnosnik, wpisal, wpisano, aktualna, zatwierdzenie_decyzja_id)
            VALUES ('druga', 'nowak', 'niskie', 'czynniki', 'zwykle', 'kontroler', 'raport',
              'kierownik', '2026-03-02', 1, 'o-innej')",
            &[],
        ),
        rule,
    );

    approve(
        &room,
        &request_about(
            "zatwierdzenie",
            RodzajDecyzji::ZatwierdzenieOceny,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
            "rejestr_kontroli_oceny_klientow",
            "pierwsza",
        ),
    );
    rejestr_kontroli_oceny::wpisz_zatwierdzenie_oceny(
        &room,
        &WpiszZatwierdzenieOcenyParams {
            zatwierdzenie_decyzja_id: "zatwierdzenie".to_string(),
            id: "pierwsza".to_string(),
        },
    )
    .expect("enter the approval of the assessment");
    approve(
        &room,
        &request_about(
            "ponowne",
            RodzajDecyzji::ZatwierdzenieOceny,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
            "rejestr_kontroli_oceny_klientow",
            "pierwsza",
        ),
    );
    let second_approval = WpiszZatwierdzenieOcenyParams {
        zatwierdzenie_decyzja_id: "ponowne".to_string(),
        id: "pierwsza".to_string(),
    };
    expect::refused(
        rejestr_kontroli_oceny::wpisz_zatwierdzenie_oceny(&room, &second_approval),
        "oceny ryzyka klienta się nie zmienia",
    );

    // An assessment replaced by a newer one is approved no more.
    add_assessment(
        &room,
        &WpiszOceneRyzykaKlientaParams {
            wpisano: "2026-06-01".to_string(),
            ..assessment("trzecia")
        },
    );
    add_assessment(
        &room,
        &WpiszOceneRyzykaKlientaParams {
            wpisano: "2026-06-02".to_string(),
            ..assessment("czwarta")
        },
    );
    approve(
        &room,
        &request_about(
            "o-trzeciej",
            RodzajDecyzji::ZatwierdzenieOceny,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
            "rejestr_kontroli_oceny_klientow",
            "trzecia",
        ),
    );
    let approval_of_replaced = WpiszZatwierdzenieOcenyParams {
        zatwierdzenie_decyzja_id: "o-trzeciej".to_string(),
        id: "trzecia".to_string(),
    };
    expect::refused(
        rejestr_kontroli_oceny::wpisz_zatwierdzenie_oceny(&room, &approval_of_replaced),
        "oceny ryzyka klienta się nie zmienia",
    );
}

/// Ocena biura: kolejna wersja z akceptacją właściciela i aktualizacją najpóźniej po 2 latach
#[test]
fn office_assessment() {
    let room = new_room();
    approve(
        &room,
        &request_about(
            "akceptacja-oceny-1",
            RodzajDecyzji::AkceptacjaOcenyBiura,
            WpisyDecydujeFunkcja::WlascicielBiura,
            "rejestr_kontroli_oceny_biura",
            "ocena-biura-1",
        ),
    );
    let accepted =
        "ocena ryzyka biura wskazuje zatwierdzoną decyzję akceptacja_oceny_biura o tej wersji";

    let update_too_late = WpiszOceneRyzykaBiuraParams {
        aktualizacja_do: "2028-01-11".to_string(),
        ..first_office_assessment()
    };
    expect::refused(
        rejestr_kontroli_oceny::wpisz_ocene_ryzyka_biura(&room, &update_too_late),
        "aktualizacja_najpozniej_po_2_latach",
    );
    let skipped_version = WpiszOceneRyzykaBiuraParams {
        wersja: 2,
        ..first_office_assessment()
    };
    expect::refused(
        rejestr_kontroli_oceny::wpisz_ocene_ryzyka_biura(&room, &skipped_version),
        "wersja oceny ryzyka biura ma kolejny numer",
    );
    let other_version = WpiszOceneRyzykaBiuraParams {
        id: "ocena-biura-inna".to_string(),
        ..first_office_assessment()
    };
    expect::refused(
        rejestr_kontroli_oceny::wpisz_ocene_ryzyka_biura(&room, &other_version),
        accepted,
    );

    rejestr_kontroli_oceny::wpisz_ocene_ryzyka_biura(&room, &first_office_assessment())
        .expect("enter the office's risk assessment");
    expect::refused(
        room.execute(
            "UPDATE rejestr_kontroli_oceny_biura SET aktualizacja_do = '2027-01-10'",
            &[],
        ),
        "oceny ryzyka biura się nie zmienia",
    );

    let office_assessments =
        rejestr_kontroli_oceny::oceny_ryzyka_biura(&room).expect("read the office's assessments");
    assert_eq!(office_assessments[0].wersja, 1);
}

// Doubts ------------------------------------------------------------------------------------------

/// Wątpliwość: powstaje stojąca, zdejmuje ją raz decyzja o tej wątpliwości
#[test]
fn doubt_lifted() {
    let room = new_room();
    add_client(&room, &client("nowak"));
    let rule =
        "zdjęcie wątpliwości wskazuje zatwierdzoną decyzję zdjecie_watpliwosci o tej wątpliwości";

    ask(
        &room,
        &request_about(
            "o-innej",
            RodzajDecyzji::ZdjecieWatpliwosci,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
            "rejestr_kontroli_watpliwosci_wpisy",
            "inna",
        ),
    );
    resolve(
        &room,
        &RozstrzygnijDecyzjeKontroliParams {
            uzasadnienie: Some("wyjaśnione".to_string()),
            ..resolution("o-innej", WpisyDecydujeFunkcja::OsobaWyznaczona)
        },
    );
    expect::refused(
        room.execute(
            "INSERT INTO rejestr_kontroli_watpliwosci_wpisy (id, teczka_odnosnik,
              znacznik_odnosnik, zglosil, zgloszono, powod, raport_odnosnik, wpisal, wpisano,
              zdjecie_decyzja_id)
            VALUES ('od-razu', 'teczka', 'znacznik', 'kontroler', '2026-03-03T09:00:00Z', 'powód',
              'raport', 'kierownik', '2026-03-03', 'o-innej')",
            &[],
        ),
        "wątpliwość powstaje jako stojąca",
    );
    rejestr_kontroli_watpliwosci::wpisz_watpliwosc(&room, &doubt()).expect("enter the doubt");
    let same_doubt_again = WpiszWatpliwoscParams {
        id: "ta-sama".to_string(),
        ..doubt()
    };
    expect::refused(
        rejestr_kontroli_watpliwosci::wpisz_watpliwosc(&room, &same_doubt_again),
        REPLACEMENT,
    );

    let lifted_by_other_decision = WpiszZdjecieWatpliwosciParams {
        zdjecie_decyzja_id: "o-innej".to_string(),
        id: "watpliwosc".to_string(),
    };
    expect::refused(
        rejestr_kontroli_watpliwosci::wpisz_zdjecie_watpliwosci(&room, &lifted_by_other_decision),
        rule,
    );
    ask(
        &room,
        &request_about(
            "zdjecie",
            RodzajDecyzji::ZdjecieWatpliwosci,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
            "rejestr_kontroli_watpliwosci_wpisy",
            "watpliwosc",
        ),
    );
    resolve(
        &room,
        &RozstrzygnijDecyzjeKontroliParams {
            uzasadnienie: Some("źródło środków wyjaśnione".to_string()),
            ..resolution("zdjecie", WpisyDecydujeFunkcja::OsobaWyznaczona)
        },
    );
    rejestr_kontroli_watpliwosci::wpisz_zdjecie_watpliwosci(
        &room,
        &WpiszZdjecieWatpliwosciParams {
            zdjecie_decyzja_id: "zdjecie".to_string(),
            id: "watpliwosc".to_string(),
        },
    )
    .expect("lift the doubt");
    ask(
        &room,
        &request_about(
            "ponowne",
            RodzajDecyzji::ZdjecieWatpliwosci,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
            "rejestr_kontroli_watpliwosci_wpisy",
            "watpliwosc",
        ),
    );
    resolve(
        &room,
        &RozstrzygnijDecyzjeKontroliParams {
            uzasadnienie: Some("j.w.".to_string()),
            ..resolution("ponowne", WpisyDecydujeFunkcja::OsobaWyznaczona)
        },
    );
    let lifted_again = WpiszZdjecieWatpliwosciParams {
        zdjecie_decyzja_id: "ponowne".to_string(),
        id: "watpliwosc".to_string(),
    };
    expect::refused(
        rejestr_kontroli_watpliwosci::wpisz_zdjecie_watpliwosci(&room, &lifted_again),
        "wątpliwości się nie zmienia",
    );
    expect::refused(
        room.execute(
            "UPDATE rejestr_kontroli_watpliwosci_wpisy SET powod = 'inny'",
            &[],
        ),
        "wątpliwości się nie zmienia",
    );
}

/// Niemożność środka wskazuje decyzję o nienawiązaniu stosunków z tym klientem
#[test]
fn measure_impossible() {
    let room = new_room();
    add_client(&room, &client("nowak"));
    add_client(
        &room,
        &WpiszKlientaKontroliParams {
            kartoteka_odnosnik: Some("kartoteka/lis".to_string()),
            ..client("lis")
        },
    );
    approve(
        &room,
        &request_about(
            "o-lisie",
            RodzajDecyzji::NienawiazanieStosunkow,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
            CLIENTS,
            "lis",
        ),
    );
    approve(
        &room,
        &request_about(
            "nienawiazanie",
            RodzajDecyzji::NienawiazanieStosunkow,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
            CLIENTS,
            "nowak",
        ),
    );
    let impossibility = WpiszNiemoznoscParams {
        klient_id: "nowak".to_string(),
        srodek: NiemoznosciSrodek::Beneficjent,
        opis: "klient nie wskazał beneficjenta".to_string(),
        decyzja_id: "nienawiazanie".to_string(),
        ocena_zawiadomienia: "brak podstaw do zawiadomienia".to_string(),
        raport_odnosnik: "raport-kontrolera/4".to_string(),
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-03-05".to_string(),
    };

    let about_other_client = WpiszNiemoznoscParams {
        decyzja_id: "o-lisie".to_string(),
        ..impossibility.clone()
    };
    expect::refused(
        rejestr_kontroli_watpliwosci::wpisz_niemoznosc(&room, &about_other_client),
        "niemożność środka wskazuje zatwierdzoną decyzję nienawiazanie_stosunkow o tym kliencie",
    );
    rejestr_kontroli_watpliwosci::wpisz_niemoznosc(&room, &impossibility)
        .expect("enter the impossibility of a measure");
    expect::refused(
        room.execute(
            "UPDATE rejestr_kontroli_watpliwosci_niemoznosci SET opis = 'inny'",
            &[],
        ),
        "wpisu niemożności się nie zmienia",
    );
}

// Notifications, requests and inspections ---------------------------------------------------------

/// Zawiadomienie: po potwierdzeniu podejrzenia i decyzji o przekazaniu, nie wcześniej
#[test]
fn notification() {
    let room = new_room();
    approve(
        &room,
        &request(
            "potwierdzenie",
            RodzajDecyzji::PotwierdzeniePodejrzenia,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
        ),
    );
    approve(
        &room,
        &request(
            "przekazanie",
            RodzajDecyzji::PrzekazanieZawiadomienia,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
        ),
    );
    ask(
        &room,
        &request(
            "czekajace",
            RodzajDecyzji::PotwierdzeniePodejrzenia,
            WpisyDecydujeFunkcja::OsobaWyznaczona,
        ),
    );

    let waiting_confirmation = WpiszZawiadomienieParams {
        potwierdzenie_decyzja_id: "czekajace".to_string(),
        ..art74_notification()
    };
    expect::refused(
        rejestr_kontroli_zawiadomienia::wpisz_zawiadomienie(&room, &waiting_confirmation),
        "zawiadomienie wskazuje zatwierdzoną decyzję potwierdzenie_podejrzenia",
    );
    let confirmation_as_handover = WpiszZawiadomienieParams {
        przekazanie_decyzja_id: "potwierdzenie".to_string(),
        ..art74_notification()
    };
    expect::refused(
        rejestr_kontroli_zawiadomienia::wpisz_zawiadomienie(&room, &confirmation_as_handover),
        "zawiadomienie wskazuje zatwierdzoną decyzję przekazanie_zawiadomienia",
    );
    let handed_over_before_confirmation = WpiszZawiadomienieParams {
        przekazano: "2026-03-01T11:00:00Z".to_string(),
        ..art74_notification()
    };
    expect::refused(
        rejestr_kontroli_zawiadomienia::wpisz_zawiadomienie(
            &room,
            &handed_over_before_confirmation,
        ),
        "przekazanie zawiadomienia nie jest wcześniejsze niż potwierdzenie podejrzenia",
    );
    expect::refused(
        room.execute(
            "INSERT INTO rejestr_kontroli_zawiadomienia_wpisy (id, rodzaj,
              potwierdzenie_decyzja_id, przekazanie_decyzja_id, przekazal, przekazano, podstawy,
              potwierdzenie_przyjecia, wpisal, wpisano)
            VALUES ('od-razu', 'art86', 'potwierdzenie', 'przekazanie', 'osoba',
              '2026-03-02T09:00:00Z', 'podstawy', 'UPO', 'kierownik', '2026-03-02')",
            &[],
        ),
        "zawiadomienie powstaje bez potwierdzenia przyjęcia i odpowiedzi organu",
    );

    rejestr_kontroli_zawiadomienia::wpisz_zawiadomienie(&room, &art74_notification())
        .expect("enter the notification");
    rejestr_kontroli_zawiadomienia::wpisz_potwierdzenie_przyjecia(
        &room,
        &WpiszPotwierdzeniePrzyjeciaParams {
            potwierdzenie_przyjecia: "UPO 123".to_string(),
            id: "zawiadomienie".to_string(),
        },
    )
    .expect("enter the confirmation of receipt");
    rejestr_kontroli_zawiadomienia::wpisz_odpowiedz_organu(
        &room,
        &WpiszOdpowiedzOrganuParams {
            odpowiedz_organu: "żądanie wstrzymania transakcji".to_string(),
            id: "zawiadomienie".to_string(),
        },
    )
    .expect("enter the authority's answer");

    let unchanged = "śladu zawiadomienia się nie zmienia";
    let second_confirmation = WpiszPotwierdzeniePrzyjeciaParams {
        potwierdzenie_przyjecia: "UPO 124".to_string(),
        id: "zawiadomienie".to_string(),
    };
    expect::refused(
        rejestr_kontroli_zawiadomienia::wpisz_potwierdzenie_przyjecia(&room, &second_confirmation),
        unchanged,
    );
    let second_answer = WpiszOdpowiedzOrganuParams {
        odpowiedz_organu: "inna".to_string(),
        id: "zawiadomienie".to_string(),
    };
    expect::refused(
        rejestr_kontroli_zawiadomienia::wpisz_odpowiedz_organu(&room, &second_answer),
        unchanged,
    );
    expect::refused(
        room.execute(
            "UPDATE rejestr_kontroli_zawiadomienia_wpisy SET przekazano = '2026-03-03T09:00:00Z'",
            &[],
        ),
        unchanged,
    );
}

/// Żądanie organu: przekazanie raz, w komplecie i po żądaniu; kontroli się nie zmienia
#[test]
fn authority_requests_and_inspections() {
    let room = new_room();
    full_register(&room);
    let authority_requests =
        rejestr_kontroli_zawiadomienia::zadania_organow(&room).expect("read the requests");
    let request_id = authority_requests[0].id;

    expect::refused(
        room.execute(
            "INSERT INTO rejestr_kontroli_zawiadomienia_zadania (organ, zadano, tresc, odebral,
              przekazano, co_przekazano, wpisal, wpisano)
            VALUES ('GIIF', '2026-04-01T09:00:00Z', 'dane', 'osoba', '2026-04-02T09:00:00Z', 'dane',
              'kierownik', '2026-04-02')",
            &[],
        ),
        "żądanie organu powstaje bez przekazania",
    );
    let handover_without_content = format!(
        "UPDATE rejestr_kontroli_zawiadomienia_zadania SET przekazano = '2026-04-02T09:00:00Z' \
        WHERE id = {request_id}"
    );
    expect::refused(
        room.execute(&handover_without_content, &[]),
        "przekazanie_w_komplecie",
    );
    let handover_before_request = WpiszOdpowiedzNaZadanieParams {
        przekazano: "2026-03-31T09:00:00Z".to_string(),
        co_przekazano: "dane klienta".to_string(),
        id: request_id,
    };
    expect::refused(
        rejestr_kontroli_zawiadomienia::wpisz_odpowiedz_na_zadanie(&room, &handover_before_request),
        "przekazanie_po_zadaniu",
    );
    rejestr_kontroli_zawiadomienia::wpisz_odpowiedz_na_zadanie(
        &room,
        &WpiszOdpowiedzNaZadanieParams {
            przekazano: "2026-04-02T09:00:00Z".to_string(),
            co_przekazano: "dane klienta".to_string(),
            id: request_id,
        },
    )
    .expect("enter the answer to the request");
    let second_handover = WpiszOdpowiedzNaZadanieParams {
        przekazano: "2026-04-03T09:00:00Z".to_string(),
        co_przekazano: "więcej".to_string(),
        id: request_id,
    };
    expect::refused(
        rejestr_kontroli_zawiadomienia::wpisz_odpowiedz_na_zadanie(&room, &second_handover),
        "żądania organu się nie zmienia",
    );

    let inspection_ending_before_start = WpiszKontroleOrganuParams {
        organ: "KAS".to_string(),
        od: "2026-05-04".to_string(),
        r#do: "2026-05-01".to_string(),
        protokol_sciezka: "protokol.pdf".to_string(),
        protokol_skrot: "sha256".to_string(),
        zalecenia: None,
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-05-10".to_string(),
    };
    expect::refused(
        rejestr_kontroli_zawiadomienia::wpisz_kontrole_organu(
            &room,
            &inspection_ending_before_start,
        ),
        "koniec_nie_przed_poczatkiem",
    );
    expect::refused(
        room.execute(
            "UPDATE rejestr_kontroli_zawiadomienia_kontrole SET zalecenia = 'brak'",
            &[],
        ),
        "wpisu kontroli się nie zmienia",
    );
}

// Organisation ------------------------------------------------------------------------------------

/// Procedura: kolejna wersja z akceptacją właściciela, wprowadzona nie przed akceptacją
#[test]
fn procedure() {
    let room = new_room();
    approve(
        &room,
        &request_about(
            "akceptacja-procedury-1",
            RodzajDecyzji::AkceptacjaProcedury,
            WpisyDecydujeFunkcja::WlascicielBiura,
            "rejestr_kontroli_organizacja_procedury",
            "procedura-1",
        ),
    );
    approve(
        &room,
        &request_about(
            "o-innej",
            RodzajDecyzji::AkceptacjaProcedury,
            WpisyDecydujeFunkcja::WlascicielBiura,
            "rejestr_kontroli_organizacja_procedury",
            "inna",
        ),
    );

    let accepted_for_other = WpiszProcedureParams {
        akceptacja_decyzja_id: "o-innej".to_string(),
        ..first_procedure()
    };
    expect::refused(
        rejestr_kontroli_organizacja::wpisz_procedure(&room, &accepted_for_other),
        "procedura wskazuje zatwierdzoną decyzję akceptacja_procedury o tej wersji",
    );
    let skipped_version = WpiszProcedureParams {
        wersja: 2,
        ..first_procedure()
    };
    expect::refused(
        rejestr_kontroli_organizacja::wpisz_procedure(&room, &skipped_version),
        "wersja procedury ma kolejny numer",
    );
    let introduced_before_acceptance = WpiszProcedureParams {
        wprowadzono: "2026-02-28".to_string(),
        ..first_procedure()
    };
    expect::refused(
        rejestr_kontroli_organizacja::wpisz_procedure(&room, &introduced_before_acceptance),
        "procedurę wprowadza się nie wcześniej niż w dniu akceptacji",
    );

    rejestr_kontroli_organizacja::wpisz_procedure(&room, &first_procedure())
        .expect("enter the procedure");
    expect::refused(
        room.execute(
            "UPDATE rejestr_kontroli_organizacja_procedury SET wprowadzono = '2026-04-01'",
            &[],
        ),
        "wersji procedury się nie zmienia",
    );
}

/// The default parameters of an appointment entered on 2026-06-01.
fn appointment(funkcja: WyznaczeniaFunkcja, osoba: &str, od: &str) -> WpiszWyznaczenieParams {
    WpiszWyznaczenieParams {
        funkcja,
        osoba: osoba.to_string(),
        od: od.to_string(),
        podstawa: None,
        wpisal: "kierownik-kontroli".to_string(),
        wpisano: "2026-06-01".to_string(),
    }
}

/// Wyznaczenia: jedna osoba z art. 8 AML, wyznaczenie kończy się raz; szkoleń się nie zmienia
#[test]
fn appointments_and_training() {
    let room = new_room();
    full_register(&room);

    let second_art8 = appointment(WyznaczeniaFunkcja::Art8, "nastepca", "2026-06-01");
    expect::refused(
        rejestr_kontroli_organizacja::wpisz_wyznaczenie(&room, &second_art8),
        "osoba z art. 8 AML jest jedna",
    );
    rejestr_kontroli_organizacja::wpisz_wyznaczenie(
        &room,
        &appointment(WyznaczeniaFunkcja::Art6, "wlasciciel-biura", "2026-01-01"),
    )
    .expect("appoint the owner under art. 6");
    rejestr_kontroli_organizacja::wpisz_wyznaczenie(
        &room,
        &appointment(WyznaczeniaFunkcja::Art6, "wspolnik", "2026-01-01"),
    )
    .expect("appoint the partner under art. 6");
    expect::refused(
        room.execute(
            "INSERT INTO rejestr_kontroli_organizacja_wyznaczenia (funkcja, osoba, od, do, wpisal,
              wpisano)
            VALUES ('art6', 'dawny', '2025-01-01', '2025-12-31', 'kierownik', '2026-06-01')",
            &[],
        ),
        "wyznaczenie powstaje jako bieżące",
    );

    let appointments =
        rejestr_kontroli_organizacja::wyznaczenia(&room).expect("read the appointments");
    let art8_id = appointments
        .iter()
        .find(|row| row.funkcja == WyznaczeniaFunkcja::Art8)
        .expect("a person is appointed under art. 8")
        .id;
    let end_before_start = ZakonczWyznaczenieParams {
        r#do: "2025-12-31".to_string(),
        id: art8_id,
    };
    expect::refused(
        rejestr_kontroli_organizacja::zakoncz_wyznaczenie(&room, &end_before_start),
        "koniec_nie_przed_poczatkiem",
    );
    rejestr_kontroli_organizacja::zakoncz_wyznaczenie(
        &room,
        &ZakonczWyznaczenieParams {
            r#do: "2026-05-31".to_string(),
            id: art8_id,
        },
    )
    .expect("end the appointment under art. 8");
    rejestr_kontroli_organizacja::wpisz_wyznaczenie(
        &room,
        &appointment(WyznaczeniaFunkcja::Art8, "nastepca", "2026-06-01"),
    )
    .expect("appoint the successor under art. 8");
    let second_end = ZakonczWyznaczenieParams {
        r#do: "2026-06-30".to_string(),
        id: art8_id,
    };
    expect::refused(
        rejestr_kontroli_organizacja::zakoncz_wyznaczenie(&room, &second_end),
        "wyznaczenia się nie zmienia",
    );

    expect::refused(
        room.execute(
            "UPDATE rejestr_kontroli_organizacja_szkolenia SET temat = 'inny'",
            &[],
        ),
        "wpisu szkolenia się nie zmienia",
    );
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

/// R9: wierszy rejestru kontroli się nie usuwa ani nie zastępuje przez REPLACE albo upsert
#[test]
fn no_deletion_or_replacement() {
    let room = new_room();
    full_register(&room);

    let table_rows = room
        .execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'rejestr\\_%' ESCAPE '\\'",
            &[],
        )
        .expect("read the tables");
    let tables: Vec<String> = table_rows
        .iter()
        .map(|row| row.get::<String>("name").expect("a table has a name"))
        .collect();
    assert_eq!(tables.len(), 24, "{tables:?}");

    for table in &tables {
        let row_count = number_of(&room, &format!("SELECT count(*) FROM {table}"));
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

    // Version numbers and the flag of a case file are UNIQUE keys of their own.
    let copies_with_new_key = [
        (
            "rejestr_kontroli_oceny_biura",
            "'nowy-klucz', wersja, czynniki, sporzadzono, aktualizacja_do, dokument_sciezka, \
            dokument_skrot, akceptacja_decyzja_id, wpisal, wpisano",
        ),
        (
            "rejestr_kontroli_organizacja_procedury",
            "'nowy-klucz', wersja, dokument_sciezka, dokument_skrot, akceptacja_decyzja_id, \
            wprowadzono, wpisal, wpisano",
        ),
        (
            "rejestr_kontroli_watpliwosci_wpisy",
            "'nowy-klucz', teczka_odnosnik, znacznik_odnosnik, klient_id, zglosil, zgloszono, \
            powod, raport_odnosnik, wpisal, wpisano, zdjecie_decyzja_id",
        ),
    ];
    for (table, copy) in copies_with_new_key {
        let columns = column_names_of(&room, table).join(", ");
        expect::refused(
            room.execute(
                &format!("INSERT OR REPLACE INTO {table} ({columns}) SELECT {copy} FROM {table}"),
                &[],
            ),
            REPLACEMENT,
        );
    }
}
