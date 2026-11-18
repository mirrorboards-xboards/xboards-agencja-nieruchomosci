//! Scenarios of the core of every case room (spec/cases/sprawa).
//!
//! Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
//! allows and a write it refuses. The first scenario follows a whole rental management case, from
//! intake to the return of the deposit and the closing of the case, as the profiles describe it.
//!
//! This module also holds the building blocks of a case, which the other core modules import. The
//! building blocks marked `pub(crate)` are shared with the scenarios of the case types too: their
//! rooms hold the kinds of the core, so the operations of `cases::sprawa` run on them.

use xboards_agencja_nieruchomosci::cases::sprawa::{
    self, sprawa_decyzje, sprawa_dokumenty, sprawa_korespondencja, sprawa_metryka,
    sprawa_nieruchomosci, sprawa_notatki, sprawa_powiazania, sprawa_protokoly, sprawa_rachunki,
    sprawa_sprawdzenia, sprawa_terminy, sprawa_uczestnicy, sprawa_umocowania, sprawa_umowy,
    sprawa_wyplaty, sprawa_zgody,
};
use xboards_agencja_nieruchomosci::xmeta_room::{
    FromSqlValue, MemoryRoom, MetaRoom, SqlValue, ToSqlValue,
};

use sprawa_decyzje::{
    DecyzjeWierszaParams, PoprosODecyzjeParams, RozstrzygnijDecyzjeParams,
    UznajProsbeZaZbednaParams, WpisyProsiStanowisko as ProsiStanowisko,
    WpisyRodzaj as RodzajDecyzji, WpisyRozstrzygniecie as Rozstrzygniecie,
    WpisyStan as StanDecyzji,
};
use sprawa_dokumenty::{
    OdnotujOtrzymanieDokumentuParams, OdnotujZamowienieDokumentuParams, OdrzucDokumentParams,
    OznaczDokumentJakoSprawdzonyParams, PotwierdzZrodloParams, UznajDokumentZaZbednyParams,
    WpisyKanal as KanalDokumentu, WpisyPostac as PostacDokumentu, WpisyRodzaj as RodzajDokumentu,
    WpisyWaznoscZrodlo as WaznoscZrodlo, WpisyWymaga as WymagaDokumentu,
    WpiszOtrzymanyDokumentParams, WpiszWymaganyDokumentParams, WpiszZrodloParams,
    ZrodlaWierszaParams,
};
use sprawa_korespondencja::{
    DolaczDokumentDoWplywuParams, DolaczDokumentDoWysylkiParams, OdnotujDoreczenieParams,
    OdnotujWyslanieParams, PrzygotujWysylkeParams, WpiszWplywParams, WplywyKanal,
    WysylkiAdresatRodzaj, WysylkiDoUczestnikaParams, WysylkiKanal, WysylkiWImieniu,
};
use sprawa_metryka::{
    PostawWatpliwoscParams, ZalozMetrykeParams, ZdejmijWatpliwoscParams, ZmienEtapSprawyParams,
};
use sprawa_nieruchomosci::{
    WpisyPrawo as PrawoDoNieruchomosci, WpisyRodzaj as RodzajNieruchomosci, WpiszDzialkeParams,
    WpiszNieruchomoscParams, WykreslDzialkeParams, ZmienDaneNieruchomosciParams,
    ZmienDzialkeParams,
};
use sprawa_notatki::{
    DopiszNotatkeParams, OznaczNotatkeJakoWykonanaParams, WpisyOdStanowiska as StanowiskoNotatki,
    WskazWierszNotatkiParams,
};
use sprawa_powiazania::{WpisyRodzaj as RodzajPowiazania, WpiszPowiazanieParams};
use sprawa_protokoly::{
    DopiszPozycjeProtokoluParams, PodpiszProtokolParams, PozycjeRodzaj as RodzajPozycjiProtokolu,
    PrzygotujProtokolParams, WpisyRodzaj as RodzajProtokolu,
};
use sprawa_rachunki::{
    OdnotujPotwierdzenieRachunkuParams, WpisyCel as CelRachunku, WpiszRachunekParams,
};
use sprawa_sprawdzenia::{
    WpisyRodzaj as RodzajSprawdzenia, WpisyWynik as WynikSprawdzenia, WpiszSprawdzenieParams,
};
use sprawa_terminy::{
    OdnotujPotwierdzenieTerminuParams, WpisyPilnujeStanowisko as PilnujeStanowisko,
    WpisyRodzaj as RodzajTerminu, WpisyStan as StanTerminu, WpiszTerminParams,
    ZmienStanTerminuParams,
};
use sprawa_uczestnicy::{
    AdresyRodzaj, NadajRoleUczestnikowiParams, UczestnikParams,
    UstalSytuacjeRodzinnaUczestnikaParams, WpisyDaneOd as DaneOd,
    WpisyKanalKontaktu as KanalKontaktu, WpisyRodzajPodmiotu as RodzajPodmiotu,
    WpisyStanCywilny as StanCywilny, WpisyUstrojMajatkowy as UstrojMajatkowy,
    WpiszAdresUczestnikaParams, WpiszObywatelstwoUczestnikaParams, WpiszUczestnikaParams,
    WykreslAdresUczestnikaParams, WykreslObywatelstwoUczestnikaParams, ZakonczRoleUczestnikaParams,
    ZmienDaneUczestnikaParams, ZmienKontaktUczestnikaParams,
};
use sprawa_umocowania::{
    DopiszCzynnoscUmocowaniaParams, WpisyForma as FormaUmocowania, WpisyRodzaj as RodzajUmocowania,
    WpisyZakres as ZakresUmocowania, WpiszUmocowanieParams,
};
use sprawa_umowy::{
    DopiszCzynnoscUmowyParams, DopiszStroneUmowyParams, OdnotujInformacjeOOdstapieniuParams,
    OdnotujZawarcieUmowyParams, PrzekazUmoweDoPodpisuParams, PrzygotujUmoweParams,
    WpisyForma as FormaUmowy, WpisyRodzaj as RodzajUmowy, WpisySposobZawarcia as SposobZawarcia,
    WpiszZdarzenieUmowyParams, ZdarzeniaRodzaj as RodzajZdarzeniaUmowy,
};
use sprawa_wyplaty::{
    OdnotujWykonanieWyplatyParams, PrzygotujWyplateParams, WpisyStan as StanWyplaty,
    WskazSprawdzenieWyplatyParams, WskazZgodeNaWyplateParams,
};
use sprawa_zgody::WpiszZgodeParams;

use crate::expect;

/// A new room of the core alone.
pub(crate) fn new_room() -> MemoryRoom {
    sprawa::open_in_memory().expect("open the room of the core")
}

/// A text value to bind in raw SQL.
fn text(value: &str) -> SqlValue {
    SqlValue::Text(value.to_string())
}

/// The single integer a read in raw SQL returns, e.g. a count or an id.
fn integer_value(room: &MemoryRoom, sql: &str) -> i64 {
    let value = room.value(sql, &[]).expect("read the value");

    i64::from_sql_value(&value).expect("the value is an integer")
}

// Building blocks of a case -----------------------------------------------------------------------

/// Writes the record of the case Najem 2026/017, accepted by the broker, in a room of the type
/// `typ_pokoju` (the Python default is „najem”).
pub(crate) fn open_case(room: &impl MetaRoom, typ_pokoju: &str) {
    let params = ZalozMetrykeParams {
        oznaczenie: "Najem 2026/017, ul. Polna 7 m. 12".to_string(),
        typ_pokoju: typ_pokoju.to_string(),
        przyjal: "tozsamosc-posrednika".to_string(),
        przyjeto: "2026-11-02T09:00:00Z".to_string(),
    };

    sprawa_metryka::zaloz_metryke(room, &params).expect("open the case");
}

/// The default parameters of a request for a decision about no row: the rental agent asks the
/// broker on 2026-11-03 at 10:00.
pub(crate) fn decision_request(decision_id: &str, rodzaj: RodzajDecyzji) -> PoprosODecyzjeParams {
    PoprosODecyzjeParams {
        id: decision_id.to_string(),
        rodzaj,
        prosi: "tozsamosc-agenta".to_string(),
        prosi_stanowisko: ProsiStanowisko::OpiekunNajmu,
        decyduje: "tozsamosc-posrednika".to_string(),
        do_kiedy: None,
        zlozono: "2026-11-03T10:00:00Z".to_string(),
        tresc: None,
        dotyczy_tabela: None,
        dotyczy_id: None,
        dotyczy_skrot: None,
        zastepuje_decyzja_id: None,
    }
}

/// The default parameters of a request for a decision (see [`decision_request`]) about the row
/// `dotyczy_id` of the table `dotyczy_tabela`.
pub(crate) fn decision_request_about(
    decision_id: &str,
    rodzaj: RodzajDecyzji,
    dotyczy_tabela: &str,
    dotyczy_id: &str,
) -> PoprosODecyzjeParams {
    PoprosODecyzjeParams {
        dotyczy_tabela: Some(dotyczy_tabela.to_string()),
        dotyczy_id: Some(dotyczy_id.to_string()),
        ..decision_request(decision_id, rodzaj)
    }
}

/// Writes the request for a decision of `params` (see [`decision_request`]); it waits for the
/// human.
pub(crate) fn ask_for_decision(room: &impl MetaRoom, params: &PoprosODecyzjeParams) {
    sprawa_decyzje::popros_o_decyzje(room, params).expect("ask for the decision");
}

/// The parameters of the broker's settlement of a decision, signed at `podpisano`. The app verifies
/// the signature; the database only requires it.
pub(crate) fn settlement(
    decision_id: &str,
    rozstrzygniecie: Rozstrzygniecie,
    podpisano: &str,
) -> RozstrzygnijDecyzjeParams {
    RozstrzygnijDecyzjeParams {
        rozstrzygniecie,
        warunek: None,
        uzasadnienie: None,
        podpisal: "tozsamosc-posrednika".to_string(),
        za_kogo: None,
        podpis: "podpis-kluczem-xkeys".to_string(),
        podpisano: podpisano.to_string(),
        id: decision_id.to_string(),
    }
}

/// Writes the settlement of a decision of `params` (see [`settlement`]).
pub(crate) fn settle_decision(room: &impl MetaRoom, params: &RozstrzygnijDecyzjeParams) {
    sprawa_decyzje::rozstrzygnij_decyzje(room, params).expect("settle the decision");
}

/// Settles the waiting decision `decision_id` with the broker's consent signed at `podpisano`.
pub(crate) fn approve(room: &impl MetaRoom, decision_id: &str, podpisano: &str) {
    let params = settlement(decision_id, Rozstrzygniecie::Zgoda, podpisano);

    settle_decision(room, &params);
}

/// Writes the request for a decision of `request` (see [`decision_request`]) and settles it with
/// the broker's consent signed at `podpisano`.
pub(crate) fn approved_decision(
    room: &impl MetaRoom,
    request: &PoprosODecyzjeParams,
    podpisano: &str,
) {
    ask_for_decision(room, request);
    approve(room, &request.id, podpisano);
}

/// The default parameters of a participant: Anna Kowalska, a natural person who gave her data
/// herself, neither a consumer nor a client of the office.
pub(crate) fn person(person_id: &str) -> WpiszUczestnikaParams {
    WpiszUczestnikaParams {
        id: person_id.to_string(),
        rodzaj_podmiotu: RodzajPodmiotu::OsobaFizyczna,
        imiona: Some("Anna".to_string()),
        nazwisko: Some("Kowalska".to_string()),
        imie_ojca: None,
        imie_matki: None,
        pesel: None,
        pesel_nie_nadano: false,
        konsument: false,
        nazwa: None,
        forma_prawna: None,
        krs: None,
        ceidg: false,
        nip: None,
        regon: None,
        telefon: None,
        email: None,
        kanal_kontaktu: None,
        klient_biura: false,
        kartoteka_odnosnik: None,
        dane_od: DaneOd::OdOsoby,
        dane_pozyskano: "2026-11-02T09:30:00Z".to_string(),
    }
}

/// Writes the participant of `params` (see [`person`]).
pub(crate) fn add_person(room: &impl MetaRoom, params: &WpiszUczestnikaParams) {
    sprawa_uczestnicy::wpisz_uczestnika(room, params).expect("add the participant");
}

/// Writes the participant `company_id`: Polna Development sp. z o.o., whose data came from another
/// source.
pub(crate) fn add_company(room: &impl MetaRoom, company_id: &str) {
    let params = WpiszUczestnikaParams {
        rodzaj_podmiotu: RodzajPodmiotu::OsobaPrawna,
        imiona: None,
        nazwisko: None,
        nazwa: Some("Polna Development sp. z o.o.".to_string()),
        forma_prawna: Some("spółka z ograniczoną odpowiedzialnością".to_string()),
        krs: Some("0000123456".to_string()),
        nip: Some("5250000000".to_string()),
        dane_od: DaneOd::ZInnegoZrodla,
        ..person(company_id)
    };

    sprawa_uczestnicy::wpisz_uczestnika(room, &params).expect("add the company");
}

/// The default parameters of a property: the flat at ul. Polna 7 m. 12 in Wrocław, a separate
/// ownership with its land register, 48.20 m², two rooms on the third floor (the Python default
/// id is „lokal”).
pub(crate) fn flat(flat_id: &str) -> WpiszNieruchomoscParams {
    WpiszNieruchomoscParams {
        id: flat_id.to_string(),
        rodzaj: RodzajNieruchomosci::LokalMieszkalny,
        prawo: PrawoDoNieruchomosci::OdrebnaWlasnoscLokalu,
        miejscowosc: "Wrocław".to_string(),
        gmina: "Wrocław".to_string(),
        ulica: Some("Polna".to_string()),
        numer_budynku: Some("7".to_string()),
        numer_lokalu: Some("12".to_string()),
        kod_pocztowy: None,
        numer_kw: Some("WR1K/00012345/6".to_string()),
        kw_brak: false,
        powierzchnia_uzytkowa: Some(4820),
        liczba_pokoi: Some(2),
        pietro: Some(3),
        nadrzedna_id: None,
    }
}

/// Writes the property of `params` (see [`flat`]).
pub(crate) fn add_flat(room: &impl MetaRoom, params: &WpiszNieruchomoscParams) {
    sprawa_nieruchomosci::wpisz_nieruchomosc(room, params).expect("add the property");
}

/// The default parameters of a document received from the client as a scan by e-mail, with its
/// file `dokumenty/<id>.pdf` (the Python default kind is „inny”).
pub(crate) fn received_document(
    document_id: &str,
    rodzaj: RodzajDokumentu,
) -> WpiszOtrzymanyDokumentParams {
    WpiszOtrzymanyDokumentParams {
        id: document_id.to_string(),
        rodzaj,
        nazwa: format!("Dokument {document_id}"),
        dotyczy_uczestnik_id: None,
        dotyczy_nieruchomosc_id: None,
        wystawca: None,
        data_wystawienia: None,
        waznosc_do: None,
        waznosc_zrodlo: None,
        wymaga: None,
        pochodzenie: Some("od klientki".to_string()),
        kanal: Some(KanalDokumentu::Email),
        postac: Some(PostacDokumentu::Skan),
        sciezka: format!("dokumenty/{document_id}.pdf"),
        skrot: format!("sha256-{document_id}"),
    }
}

/// Writes the received document of `params` (see [`received_document`]).
pub(crate) fn add_received_document(room: &impl MetaRoom, params: &WpiszOtrzymanyDokumentParams) {
    sprawa_dokumenty::wpisz_otrzymany_dokument(room, params).expect("add the received document");
}

/// The default parameters of an e-mail from Anna Kowalska, with no participant named as its
/// sender, and its file `wplywy/<id>.eml`.
pub(crate) fn inflow(inflow_id: &str) -> WpiszWplywParams {
    WpiszWplywParams {
        id: inflow_id.to_string(),
        nadawca: "Anna Kowalska <anna@example.pl>".to_string(),
        nadawca_uczestnik_id: None,
        kanal: WplywyKanal::Email,
        wplynelo: "2026-11-03T08:00:00Z".to_string(),
        sciezka: format!("wplywy/{inflow_id}.eml"),
        skrot: format!("sha256-{inflow_id}"),
        niezgodny_kontakt: false,
        niezgodnosc_opis: None,
        odnosnik_zewnetrzny: None,
    }
}

/// Writes the inflow of `params` (see [`inflow`]).
pub(crate) fn add_inflow(room: &impl MetaRoom, params: &WpiszWplywParams) {
    sprawa_korespondencja::wpisz_wplyw(room, params).expect("add the inflow");
}

/// The current version of the participant's identifying data.
pub(crate) fn version_of(room: &impl MetaRoom, person_id: &str) -> i64 {
    let params = UczestnikParams {
        id: person_id.to_string(),
    };

    let participant = sprawa_uczestnicy::uczestnik(room, &params)
        .expect("read the participant")
        .expect("the participant exists");

    participant.wersja
}

/// The current version of the power of attorney or representation.
pub(crate) fn power_version_of(room: &impl MetaRoom, power_id: &str) -> i64 {
    let powers = sprawa_umocowania::umocowania(room).expect("read the powers");
    let power = powers
        .iter()
        .find(|row| row.id == power_id)
        .expect("the power exists");

    power.wersja
}

/// The parameters of Kontroler's check of no row yet, made at `sprawdzono` from the land register,
/// with the description a result other than „zgodne” needs. Set the checked row with struct update
/// and write it with [`check`].
pub(crate) fn check_of(
    check_id: &str,
    rodzaj: RodzajSprawdzenia,
    sprawdzono: &str,
    wynik: WynikSprawdzenia,
) -> WpiszSprawdzenieParams {
    let compliant = matches!(wynik, WynikSprawdzenia::Zgodne);
    let opis = if compliant {
        None
    } else {
        Some("brak odpisu z dnia sprawdzenia".to_string())
    };

    let day_of_check = &sprawdzono[..10];

    WpiszSprawdzenieParams {
        id: check_id.to_string(),
        rodzaj,
        uczestnik_id: None,
        nieruchomosc_id: None,
        dokument_id: None,
        umocowanie_id: None,
        rachunek_id: None,
        wersja_wiersza: None,
        zrodlo: "odpis z centralnej bazy ksiąg wieczystych".to_string(),
        zrodlo_data: Some(day_of_check.to_string()),
        wynik,
        opis,
        sprawdzil: "tozsamosc-kontrolera".to_string(),
        sprawdzono: sprawdzono.to_string(),
    }
}

/// Writes Kontroler's check of `params` (see [`check_of`]) of a row in its current version: its
/// `wersja_wiersza` becomes the current version of the checked participant (tozsamosc, tytul) or
/// power (umocowanie), and stays NULL for an account.
pub(crate) fn check(room: &impl MetaRoom, params: &WpiszSprawdzenieParams) {
    let wersja_wiersza = match params.rodzaj {
        RodzajSprawdzenia::Tozsamosc | RodzajSprawdzenia::Tytul => {
            let person_id = params
                .uczestnik_id
                .as_deref()
                .expect("a check of identity or title names its participant");

            Some(version_of(room, person_id))
        }
        RodzajSprawdzenia::Umocowanie => {
            let power_id = params
                .umocowanie_id
                .as_deref()
                .expect("a check of a power names the power");

            Some(power_version_of(room, power_id))
        }
        RodzajSprawdzenia::Rachunek => None,
    };

    let check_of_current_version = WpiszSprawdzenieParams {
        wersja_wiersza,
        ..params.clone()
    };

    sprawa_sprawdzenia::wpisz_sprawdzenie(room, &check_of_current_version)
        .expect("write the check");
}

/// Writes Kontroler's compliant check `check_id` of the identity of the participant `person_id`,
/// made on 2026-11-03 at 09:00 (Python: `check(room, id, "tozsamosc", uczestnik_id=…)`).
pub(crate) fn check_identity(room: &impl MetaRoom, check_id: &str, person_id: &str) {
    let params = WpiszSprawdzenieParams {
        uczestnik_id: Some(person_id.to_string()),
        ..check_of(
            check_id,
            RodzajSprawdzenia::Tozsamosc,
            "2026-11-03T09:00:00Z",
            WynikSprawdzenia::Zgodne,
        )
    };

    check(room, &params);
}

/// The default parameters of the account of `person_id` for the return of the deposit, held by
/// Jan Nowak and taken from the letter `zrodlo-<id>`.
pub(crate) fn account(account_id: &str, person_id: &str) -> WpiszRachunekParams {
    WpiszRachunekParams {
        id: account_id.to_string(),
        uczestnik_id: person_id.to_string(),
        numer: "PL61109010140000071219812874".to_string(),
        posiadacz: "Jan Nowak".to_string(),
        cel: CelRachunku::ZwrotKaucji,
        zrodlo_dokument_id: Some(format!("zrodlo-{account_id}")),
        zrodlo_wplyw_id: None,
        wpisano: "2026-11-03T08:00:00Z".to_string(),
    }
}

/// Writes the received letter `zrodlo-<id>` the account comes from, then the account of `params`
/// (see [`account`]).
pub(crate) fn add_account(room: &impl MetaRoom, params: &WpiszRachunekParams) {
    let source_id = format!("zrodlo-{}", params.id);
    add_received_document(room, &received_document(&source_id, RodzajDokumentu::Pismo));

    sprawa_rachunki::wpisz_rachunek(room, params).expect("add the account");
}

/// Records that the agent confirmed the account `account_id` by phone, at the number from the
/// lease.
pub(crate) fn confirm_account(room: &impl MetaRoom, account_id: &str) {
    let params = OdnotujPotwierdzenieRachunkuParams {
        potwierdzil: "tozsamosc-agenta".to_string(),
        potwierdzono: "2026-11-03T08:30:00Z".to_string(),
        potwierdzenie_sposob: "telefonicznie pod numerem z umowy najmu".to_string(),
        id: account_id.to_string(),
    };

    sprawa_rachunki::odnotuj_potwierdzenie_rachunku(room, &params).expect("confirm the account");
}

/// Writes the account `account_id` of `person_id` (see [`account`]), confirmed by another channel,
/// checked by Kontroler (`sprawdzenie-<id>`) and admitted by a human (`dopuszczenie-<id>`).
pub(crate) fn admitted_account(room: &impl MetaRoom, account_id: &str, person_id: &str) {
    add_account(room, &account(account_id, person_id));
    confirm_account(room, account_id);

    let account_check = WpiszSprawdzenieParams {
        rachunek_id: Some(account_id.to_string()),
        ..check_of(
            &format!("sprawdzenie-{account_id}"),
            RodzajSprawdzenia::Rachunek,
            "2026-11-03T09:00:00Z",
            WynikSprawdzenia::Zgodne,
        )
    };
    check(room, &account_check);

    let admission = decision_request_about(
        &format!("dopuszczenie-{account_id}"),
        RodzajDecyzji::DopuszczenieRachunku,
        "sprawa_rachunki_wpisy",
        account_id,
    );
    approved_decision(room, &admission, "2026-11-03T12:00:00Z");
}

/// The default parameters of a payout of 3000.00 zł to `person_id` on `account_id`, the return of
/// the deposit, prepared on 2026-11-20 at 09:00.
pub(crate) fn payout(payout_id: &str, person_id: &str, account_id: &str) -> PrzygotujWyplateParams {
    PrzygotujWyplateParams {
        id: payout_id.to_string(),
        podstawa_tabela: "najem_kaucje".to_string(),
        podstawa_id: "rozliczenie-kaucji".to_string(),
        kwota: 300_000,
        wyliczenie_sciezka: format!("wyliczenia/{payout_id}.json"),
        wyliczenie_skrot: format!("sha256-{payout_id}"),
        odbiorca_uczestnik_id: person_id.to_string(),
        rachunek_id: account_id.to_string(),
        utworzono: "2026-11-20T09:00:00Z".to_string(),
    }
}

/// Writes the payout of `params` (see [`payout`]).
pub(crate) fn prepare_payout(room: &impl MetaRoom, params: &PrzygotujWyplateParams) {
    sprawa_wyplaty::przygotuj_wyplate(room, params).expect("prepare the payout");
}

/// Writes the payout `payout_id` (see [`payout`]), checked anew after it was prepared
/// (`sprawdzenie-<id>`) and approved after that check (`zgoda-<id>`), as D13 requires.
pub(crate) fn checked_and_approved_payout(
    room: &impl MetaRoom,
    payout_id: &str,
    person_id: &str,
    account_id: &str,
) {
    prepare_payout(room, &payout(payout_id, person_id, account_id));

    let check_id = format!("sprawdzenie-{payout_id}");
    let account_check = WpiszSprawdzenieParams {
        rachunek_id: Some(account_id.to_string()),
        ..check_of(
            &check_id,
            RodzajSprawdzenia::Rachunek,
            "2026-11-20T10:00:00Z",
            WynikSprawdzenia::Zgodne,
        )
    };
    check(room, &account_check);

    let check_of_payout = WskazSprawdzenieWyplatyParams {
        sprawdzenie_id: check_id,
        id: payout_id.to_string(),
    };
    sprawa_wyplaty::wskaz_sprawdzenie_wyplaty(room, &check_of_payout)
        .expect("point the payout to its check");

    let decision_id = format!("zgoda-{payout_id}");
    let consent_request = PoprosODecyzjeParams {
        zlozono: "2026-11-20T11:00:00Z".to_string(),
        ..decision_request_about(
            &decision_id,
            RodzajDecyzji::Wyplata,
            "sprawa_wyplaty_wpisy",
            payout_id,
        )
    };
    approved_decision(room, &consent_request, "2026-11-20T12:00:00Z");

    let consent_to_payout = WskazZgodeNaWyplateParams {
        decyzja_id: decision_id,
        id: payout_id.to_string(),
    };
    sprawa_wyplaty::wskaz_zgode_na_wyplate(room, &consent_to_payout)
        .expect("point the payout to its consent");
}

/// Writes the received document `dokument-<id>` and the written power of attorney `power_id` that
/// the principal gave the office for the acts `czynnosci`, given on `udzielono` (the Python
/// defaults are `["wystapienie_do_wspolnoty"]` and „2026-11-03”).
pub(crate) fn add_power_for_office(
    room: &impl MetaRoom,
    power_id: &str,
    principal_id: &str,
    czynnosci: &[&str],
    udzielono: &str,
) {
    let document_id = format!("dokument-{power_id}");
    add_received_document(
        room,
        &received_document(&document_id, RodzajDokumentu::Pelnomocnictwo),
    );

    let power = WpiszUmocowanieParams {
        id: power_id.to_string(),
        rodzaj: RodzajUmocowania::Pelnomocnictwo,
        mocodawca_uczestnik_id: principal_id.to_string(),
        umocowany_uczestnik_id: None,
        umocowane_biuro: true,
        zakres: ZakresUmocowania::DoCzynnosci,
        forma: Some(FormaUmocowania::Pisemna),
        sposob_reprezentacji: None,
        dokument_id: Some(document_id),
        udzielono: Some(udzielono.to_string()),
        wazne_do: None,
    };
    sprawa_umocowania::wpisz_umocowanie(room, &power).expect("add the power of attorney");

    for czynnosc in czynnosci {
        let act = DopiszCzynnoscUmocowaniaParams {
            umocowanie_id: power_id.to_string(),
            czynnosc: czynnosc.to_string(),
        };

        sprawa_umocowania::dopisz_czynnosc_umocowania(room, &act).expect("add an act of the power");
    }
}

/// The default parameters of a letter of the office by e-mail to the owners' association of
/// Polna 7, from the template „zapytanie-do-wspolnoty” version 3, with its file.
pub(crate) fn letter(letter_id: &str) -> PrzygotujWysylkeParams {
    PrzygotujWysylkeParams {
        id: letter_id.to_string(),
        adresat_uczestnik_id: None,
        adresat_rodzaj: Some(WysylkiAdresatRodzaj::Wspolnota),
        adresat_nazwa: Some("Wspólnota Mieszkaniowa Polna 7".to_string()),
        w_imieniu: WysylkiWImieniu::Biuro,
        w_imieniu_uczestnik_id: None,
        umocowanie_id: None,
        czynnosc: None,
        kanal: WysylkiKanal::Email,
        wzor: Some("zapytanie-do-wspolnoty".to_string()),
        wzor_wersja: Some("3".to_string()),
        zamawia_dokument_id: None,
        sciezka: Some(format!("wysylki/{letter_id}.pdf")),
        skrot: Some(format!("sha256-{letter_id}")),
        informacja_rodo: false,
        ujawnia_agenta: false,
    }
}

/// Writes the letter of `params` (see [`letter`]) as a draft.
pub(crate) fn prepare_letter(room: &impl MetaRoom, params: &PrzygotujWysylkeParams) {
    sprawa_korespondencja::przygotuj_wysylke(room, params).expect("prepare the letter");
}

/// The default parameters of sending a letter on 2026-11-05 at 10:00, with no decision.
pub(crate) fn sending(letter_id: &str) -> OdnotujWyslanieParams {
    OdnotujWyslanieParams {
        wyslano: "2026-11-05T10:00:00Z".to_string(),
        decyzja_id: None,
        odnosnik_zewnetrzny: None,
        id: letter_id.to_string(),
    }
}

/// Records the sending of a letter of `params` (see [`sending`]).
pub(crate) fn send(room: &impl MetaRoom, params: &OdnotujWyslanieParams) {
    sprawa_korespondencja::odnotuj_wyslanie(room, params).expect("send the letter");
}

/// The default parameters of an exclusive management contract for 10% of the monthly rent, about
/// no property.
pub(crate) fn contract(contract_id: &str) -> PrzygotujUmoweParams {
    PrzygotujUmoweParams {
        id: contract_id.to_string(),
        rodzaj: RodzajUmowy::Zarzadzanie,
        nieruchomosc_id: None,
        poczatek: None,
        koniec: None,
        okres_wypowiedzenia: None,
        obie_strony: false,
        wylacznosc: true,
        wynagrodzenie_opis: Some("10% czynszu miesięcznie".to_string()),
        wynagrodzenie_zdarzenie: None,
    }
}

/// Writes the draft contract of `params` (see [`contract`]) and its parties `party_ids`.
pub(crate) fn prepare_contract(
    room: &impl MetaRoom,
    params: &PrzygotujUmoweParams,
    party_ids: &[&str],
) {
    sprawa_umowy::przygotuj_umowe(room, params).expect("prepare the contract");

    for party_id in party_ids {
        let party = DopiszStroneUmowyParams {
            umowa_id: params.id.clone(),
            uczestnik_id: party_id.to_string(),
        };

        sprawa_umowy::dopisz_strone_umowy(room, &party).expect("add a party of the contract");
    }
}

/// The last day to withdraw from a contract concluded on 2026-11-04 (art. 27 ust. 1 upk).
pub(crate) const WITHDRAWAL_ENDS: &str = "2026-11-18";

/// Writes the signed contract `podpisana-<id>` and the policy `polisa-<id>` as received documents
/// and concludes the contract `contract_id` in writing on 2026-11-04, signed by the broker. A
/// consumer's contract off premises or at a distance needs `odstapienie_do`, e.g.
/// [`WITHDRAWAL_ENDS`] (the Python defaults are „w_lokalu_biura” and None).
pub(crate) fn conclude_contract(
    room: &impl MetaRoom,
    contract_id: &str,
    sposob_zawarcia: SposobZawarcia,
    odstapienie_do: Option<&str>,
) {
    let signed_id = format!("podpisana-{contract_id}");
    let policy_id = format!("polisa-{contract_id}");
    add_received_document(
        room,
        &received_document(&signed_id, RodzajDokumentu::UmowaZBiurem),
    );
    add_received_document(
        room,
        &received_document(&policy_id, RodzajDokumentu::Polisa),
    );

    let conclusion = OdnotujZawarcieUmowyParams {
        forma: FormaUmowy::Pisemna,
        sposob_zawarcia,
        zawarto: "2026-11-04T12:00:00Z".to_string(),
        podpisal_za_biuro: "tozsamosc-posrednika".to_string(),
        dokument_id: signed_id,
        polisa_dokument_id: policy_id,
        odstapienie_do: odstapienie_do.map(str::to_string),
        id: contract_id.to_string(),
    };
    sprawa_umowy::odnotuj_zawarcie_umowy(room, &conclusion).expect("conclude the contract");
}

/// Writes the contract `contract_id` (see [`contract`]) whose one party `party_id` Kontroler
/// checked (`tozsamosc-<id>`), hands it over for signing and concludes it without a withdrawal
/// date (the Python default `sposob_zawarcia` is „w_lokalu_biura”).
#[allow(
    dead_code,
    reason = "the scenarios of najem use it; they are ported after this module"
)]
pub(crate) fn concluded_contract(
    room: &impl MetaRoom,
    contract_id: &str,
    party_id: &str,
    sposob_zawarcia: SposobZawarcia,
) {
    check_identity(room, &format!("tozsamosc-{contract_id}"), party_id);
    prepare_contract(room, &contract(contract_id), &[party_id]);

    let for_signing = PrzekazUmoweDoPodpisuParams {
        id: contract_id.to_string(),
    };
    sprawa_umowy::przekaz_umowe_do_podpisu(room, &for_signing)
        .expect("hand the contract over for signing");

    conclude_contract(room, contract_id, sposob_zawarcia, None);
}

/// The default parameters of the consent `rodzaj` of `person_id`, given on 2026-11-04 at 13:00 in
/// the inflow `wplyw-<id>`: the demand to start work before the withdrawal period ends.
pub(crate) fn consent(consent_id: &str, person_id: &str, rodzaj: &str) -> WpiszZgodeParams {
    WpiszZgodeParams {
        id: consent_id.to_string(),
        uczestnik_id: person_id.to_string(),
        rodzaj: rodzaj.to_string(),
        tresc: "Żądam rozpoczęcia pracy przed upływem terminu na odstąpienie.".to_string(),
        zlozono: "2026-11-04T13:00:00Z".to_string(),
        dokument_id: None,
        wplyw_id: Some(format!("wplyw-{consent_id}")),
        trwaly_nosnik: false,
        dotyczy_tabela: None,
        dotyczy_id: None,
        zastepuje_id: None,
    }
}

/// Writes the inflow `wplyw-<id>` from the participant of the consent (see [`inflow`]), then the
/// consent of `params` (see [`consent`]).
pub(crate) fn add_consent(room: &impl MetaRoom, params: &WpiszZgodeParams) {
    let carrier = WpiszWplywParams {
        nadawca_uczestnik_id: Some(params.uczestnik_id.clone()),
        ..inflow(&format!("wplyw-{}", params.id))
    };
    add_inflow(room, &carrier);

    sprawa_zgody::wpisz_zgode(room, params).expect("add the consent");
}

/// The parameters of a change of the stage of the case by the agent, with no decision.
fn stage_change(etap: &str, od: &str) -> ZmienEtapSprawyParams {
    ZmienEtapSprawyParams {
        etap: etap.to_string(),
        od: od.to_string(),
        zmienil: "agent".to_string(),
        decyzja_id: None,
    }
}

/// Moves the case to the stage of `params` (see [`stage_change`]).
fn enter_stage(room: &impl MetaRoom, params: &ZmienEtapSprawyParams) {
    sprawa_metryka::zmien_etap_sprawy(room, params).expect("change the stage of the case");
}

/// Writes a rental management case with a row in every table of the core, as the profiles go.
pub(crate) fn full_case(room: &impl MetaRoom) {
    open_case(room, "najem");

    // The owner, a consumer client of the office, and the tenant, whose data came from her.
    add_person(
        room,
        &WpiszUczestnikaParams {
            konsument: true,
            klient_biura: true,
            kanal_kontaktu: Some(KanalKontaktu::Email),
            ..person("wlascicielka")
        },
    );
    add_person(
        room,
        &WpiszUczestnikaParams {
            imiona: Some("Jan".to_string()),
            nazwisko: Some("Nowak".to_string()),
            dane_od: DaneOd::ZInnegoZrodla,
            dane_pozyskano: "2026-11-02T10:00:00Z".to_string(),
            ..person("najemca")
        },
    );
    sprawa_uczestnicy::wpisz_adres_uczestnika(
        room,
        &WpiszAdresUczestnikaParams {
            uczestnik_id: "wlascicielka".to_string(),
            rodzaj: AdresyRodzaj::Zamieszkanie,
            kraj: "PL".to_string(),
            miejscowosc: "Wrocław".to_string(),
            ulica: Some("Kwiatowa".to_string()),
            numer_budynku: Some("3".to_string()),
            numer_lokalu: None,
            kod_pocztowy: None,
        },
    )
    .expect("add the owner's home address");
    sprawa_uczestnicy::wpisz_obywatelstwo_uczestnika(
        room,
        &WpiszObywatelstwoUczestnikaParams {
            uczestnik_id: "wlascicielka".to_string(),
            panstwo: "PL".to_string(),
        },
    )
    .expect("add the owner's citizenship");
    for (uczestnik_id, rola) in [("wlascicielka", "klient"), ("najemca", "druga_strona")] {
        let role = NadajRoleUczestnikowiParams {
            uczestnik_id: uczestnik_id.to_string(),
            rola: rola.to_string(),
            dotyczy_uczestnik_id: None,
        };

        sprawa_uczestnicy::nadaj_role_uczestnikowi(room, &role).expect("give a role");
    }

    // The flat with its land register, its plot and the source of the register number.
    add_flat(room, &flat("lokal"));
    sprawa_nieruchomosci::wpisz_dzialke(
        room,
        &WpiszDzialkeParams {
            nieruchomosc_id: "lokal".to_string(),
            obreb: "0012 Krzyki".to_string(),
            numer_dzialki: "45/7".to_string(),
            powierzchnia: None,
        },
    )
    .expect("add the plot");
    add_received_document(
        room,
        &WpiszOtrzymanyDokumentParams {
            dotyczy_nieruchomosc_id: Some("lokal".to_string()),
            pochodzenie: Some("pobrany z centralnej bazy ksiąg wieczystych".to_string()),
            kanal: Some(KanalDokumentu::RejestrElektroniczny),
            postac: Some(PostacDokumentu::Wydruk),
            ..received_document("odpis-kw", RodzajDokumentu::OdpisKsiegiWieczystej)
        },
    );
    sprawa_dokumenty::wpisz_zrodlo(
        room,
        &WpiszZrodloParams {
            dotyczy_tabela: "sprawa_nieruchomosci_wpisy".to_string(),
            dotyczy_id: "lokal".to_string(),
            pole: "numer_kw".to_string(),
            dokument_id: Some("odpis-kw".to_string()),
            od_uczestnik_id: None,
            od_kogo: None,
            potwierdzone: true,
            wpisano: "2026-11-02T11:00:00Z".to_string(),
        },
    )
    .expect("add the source of the register number");
    enter_stage(
        room,
        &ZmienEtapSprawyParams {
            zmienil: "agent-najmu".to_string(),
            ..stage_change("umowa", "2026-11-02T12:00:00Z")
        },
    );

    // The management contract, signed at a distance; the information on withdrawal goes to the
    // client before she signs (art. 12 upk).
    check_identity(room, "tozsamosc-umowa-zarzadzania", "wlascicielka");
    prepare_contract(room, &contract("umowa-zarzadzania"), &["wlascicielka"]);
    sprawa_umowy::dopisz_czynnosc_umowy(
        room,
        &DopiszCzynnoscUmowyParams {
            umowa_id: "umowa-zarzadzania".to_string(),
            czynnosc: "publikacja_ogloszenia".to_string(),
        },
    )
    .expect("add an act of the contract");
    prepare_letter(
        room,
        &PrzygotujWysylkeParams {
            adresat_rodzaj: None,
            adresat_nazwa: None,
            adresat_uczestnik_id: Some("wlascicielka".to_string()),
            wzor: Some("informacja-o-odstapieniu".to_string()),
            informacja_rodo: true,
            ujawnia_agenta: true,
            ..letter("informacja-dla-klientki")
        },
    );
    send(
        room,
        &OdnotujWyslanieParams {
            wyslano: "2026-11-04T10:00:00Z".to_string(),
            ..sending("informacja-dla-klientki")
        },
    );
    sprawa_korespondencja::odnotuj_doreczenie(
        room,
        &OdnotujDoreczenieParams {
            doreczono: "2026-11-04T10:01:00Z".to_string(),
            dowod_doreczenia: Some("potwierdzenie odczytu".to_string()),
            id: "informacja-dla-klientki".to_string(),
        },
    )
    .expect("record the delivery of the information");
    sprawa_umowy::odnotuj_informacje_o_odstapieniu(
        room,
        &OdnotujInformacjeOOdstapieniuParams {
            informacja_o_odstapieniu_wysylka_id: "informacja-dla-klientki".to_string(),
            id: "umowa-zarzadzania".to_string(),
        },
    )
    .expect("record the information on withdrawal");
    sprawa_umowy::przekaz_umowe_do_podpisu(
        room,
        &PrzekazUmoweDoPodpisuParams {
            id: "umowa-zarzadzania".to_string(),
        },
    )
    .expect("hand the contract over for signing");
    conclude_contract(
        room,
        "umowa-zarzadzania",
        SposobZawarcia::NaOdleglosc,
        Some(WITHDRAWAL_ENDS),
    );
    add_consent(
        room,
        &consent("zadanie", "wlascicielka", "zadanie_rozpoczecia_pracy"),
    );
    sprawa_umowy::wpisz_zdarzenie_umowy(
        room,
        &WpiszZdarzenieUmowyParams {
            id: "zdarzenie-zadanie".to_string(),
            umowa_id: "umowa-zarzadzania".to_string(),
            rodzaj: RodzajZdarzeniaUmowy::ZadanieRozpoczeciaPracy,
            chwila: "2026-11-04T13:00:00Z".to_string(),
            opis: None,
            dokument_id: None,
            wysylka_id: None,
            wplyw_id: None,
            zgoda_id: Some("zadanie".to_string()),
        },
    )
    .expect("add the event of the demand");
    sprawa_terminy::wpisz_termin(
        room,
        &WpiszTerminParams {
            id: "termin-odstapienia".to_string(),
            rodzaj: RodzajTerminu::Ustawowy,
            co: "Koniec terminu na odstąpienie od umowy".to_string(),
            zdarzenie_tabela: Some("sprawa_umowy_wpisy".to_string()),
            zdarzenie_id: Some("umowa-zarzadzania".to_string()),
            zdarzenie_chwila: Some("2026-11-04T12:00:00Z".to_string()),
            koniec: "2026-11-18".to_string(),
            podstawa: Some("art. 27 ust. 1 i art. 28 pkt 2 upk".to_string()),
            pilnuje_stanowisko: PilnujeStanowisko::OpiekunNajmu,
            kalendarz_odnosnik: None,
        },
    )
    .expect("add the withdrawal deadline");

    // A letter to the owners' association on the client's behalf, ordering a certificate.
    add_power_for_office(
        room,
        "pelnomocnictwo",
        "wlascicielka",
        &["wystapienie_do_wspolnoty"],
        "2026-11-03",
    );
    check(
        room,
        &WpiszSprawdzenieParams {
            umocowanie_id: Some("pelnomocnictwo".to_string()),
            ..check_of(
                "sprawdzenie-pelnomocnictwa",
                RodzajSprawdzenia::Umocowanie,
                "2026-11-03T09:00:00Z",
                WynikSprawdzenia::Zgodne,
            )
        },
    );
    sprawa_dokumenty::wpisz_wymagany_dokument(
        room,
        &WpiszWymaganyDokumentParams {
            id: "zaswiadczenie-wspolnoty".to_string(),
            rodzaj: RodzajDokumentu::Zaswiadczenie,
            nazwa: "Zaświadczenie o braku zaległości".to_string(),
            dotyczy_uczestnik_id: None,
            dotyczy_nieruchomosc_id: Some("lokal".to_string()),
            wymaga: Some(WymagaDokumentu::Umowa),
        },
    )
    .expect("require the certificate");
    prepare_letter(
        room,
        &PrzygotujWysylkeParams {
            w_imieniu: WysylkiWImieniu::Uczestnik,
            w_imieniu_uczestnik_id: Some("wlascicielka".to_string()),
            umocowanie_id: Some("pelnomocnictwo".to_string()),
            czynnosc: Some("wystapienie_do_wspolnoty".to_string()),
            zamawia_dokument_id: Some("zaswiadczenie-wspolnoty".to_string()),
            ..letter("wniosek-do-wspolnoty")
        },
    );
    sprawa_korespondencja::dolacz_dokument_do_wysylki(
        room,
        &DolaczDokumentDoWysylkiParams {
            wysylka_id: "wniosek-do-wspolnoty".to_string(),
            dokument_id: "dokument-pelnomocnictwo".to_string(),
        },
    )
    .expect("attach the power to the letter");
    send(room, &sending("wniosek-do-wspolnoty"));
    sprawa_dokumenty::odnotuj_zamowienie_dokumentu(
        room,
        &OdnotujZamowienieDokumentuParams {
            id: "zaswiadczenie-wspolnoty".to_string(),
        },
    )
    .expect("record the order of the certificate");
    add_inflow(
        room,
        &WpiszWplywParams {
            nadawca: "Wspólnota Mieszkaniowa Polna 7".to_string(),
            ..inflow("odpowiedz-wspolnoty")
        },
    );
    sprawa_dokumenty::odnotuj_otrzymanie_dokumentu(
        room,
        &OdnotujOtrzymanieDokumentuParams {
            wystawca: Some("Wspólnota Mieszkaniowa Polna 7".to_string()),
            data_wystawienia: Some("2026-11-06".to_string()),
            waznosc_do: Some("2026-12-06".to_string()),
            waznosc_zrodlo: Some(WaznoscZrodlo::Instytucja),
            pochodzenie: Some("od wspólnoty".to_string()),
            kanal: Some(KanalDokumentu::Email),
            postac: Some(PostacDokumentu::Skan),
            sciezka: "dokumenty/zaswiadczenie-wspolnoty.pdf".to_string(),
            skrot: "sha256-zaswiadczenie-wspolnoty".to_string(),
            id: "zaswiadczenie-wspolnoty".to_string(),
        },
    )
    .expect("record the receipt of the certificate");
    sprawa_korespondencja::dolacz_dokument_do_wplywu(
        room,
        &DolaczDokumentDoWplywuParams {
            wplyw_id: "odpowiedz-wspolnoty".to_string(),
            dokument_id: "zaswiadczenie-wspolnoty".to_string(),
        },
    )
    .expect("attach the certificate to the inflow");

    // Kontroler raises a doubt; the human lifts it with a signed decision without any content.
    sprawa_metryka::postaw_watpliwosc(
        room,
        &PostawWatpliwoscParams {
            id: "watpliwosc".to_string(),
            postawil: "tozsamosc-kontrolera".to_string(),
            postawiono: "2026-11-07T09:00:00Z".to_string(),
        },
    )
    .expect("raise the doubt");
    ask_for_decision(
        room,
        &PoprosODecyzjeParams {
            prosi_stanowisko: ProsiStanowisko::Kontroler,
            decyduje: "tozsamosc-wlasciciela-kierownika-kontroli".to_string(),
            zlozono: "2026-11-07T10:00:00Z".to_string(),
            ..decision_request_about(
                "zdjecie-watpliwosci",
                RodzajDecyzji::ZdjecieWatpliwosci,
                "sprawa_metryka_watpliwosci",
                "watpliwosc",
            )
        },
    );
    approve(room, "zdjecie-watpliwosci", "2026-11-07T11:00:00Z");
    sprawa_metryka::zdejmij_watpliwosc(
        room,
        &ZdejmijWatpliwoscParams {
            zdjecie_decyzja_id: "zdjecie-watpliwosci".to_string(),
            id: "watpliwosc".to_string(),
        },
    )
    .expect("lift the doubt");

    // The handover date, confirmed by a human.
    sprawa_terminy::wpisz_termin(
        room,
        &WpiszTerminParams {
            id: "termin-przekazania".to_string(),
            rodzaj: RodzajTerminu::ZUmowy,
            co: "Przekazanie lokalu najemcy".to_string(),
            zdarzenie_tabela: None,
            zdarzenie_id: None,
            zdarzenie_chwila: None,
            koniec: "2026-11-10T10:00:00Z".to_string(),
            podstawa: Some("§ 3 umowy najmu".to_string()),
            pilnuje_stanowisko: PilnujeStanowisko::OpiekunNajmu,
            kalendarz_odnosnik: Some("kalendarz/przekazanie-polna-7".to_string()),
        },
    )
    .expect("add the handover date");
    approved_decision(
        room,
        &PoprosODecyzjeParams {
            zlozono: "2026-11-08T10:00:00Z".to_string(),
            ..decision_request_about(
                "potwierdzenie-przekazania",
                RodzajDecyzji::PotwierdzenieTerminu,
                "sprawa_terminy_wpisy",
                "termin-przekazania",
            )
        },
        "2026-11-08T12:00:00Z",
    );
    sprawa_terminy::odnotuj_potwierdzenie_terminu(
        room,
        &OdnotujPotwierdzenieTerminuParams {
            potwierdzenie_decyzja_id: "potwierdzenie-przekazania".to_string(),
            id: "termin-przekazania".to_string(),
        },
    )
    .expect("confirm the handover date");
    sprawa_terminy::zmien_stan_terminu(
        room,
        &ZmienStanTerminuParams {
            stan: StanTerminu::Dotrzymany,
            id: "termin-przekazania".to_string(),
        },
    )
    .expect("record the handover date as kept");

    // The handover protocol with the meter reading and the keys, signed by both sides.
    sprawa_protokoly::przygotuj_protokol(
        room,
        &PrzygotujProtokolParams {
            id: "protokol-wydania".to_string(),
            rodzaj: RodzajProtokolu::Wydanie,
            nieruchomosc_id: "lokal".to_string(),
            chwila: "2026-11-10T10:00:00Z".to_string(),
        },
    )
    .expect("prepare the handover protocol");
    sprawa_protokoly::dopisz_pozycje_protokolu(
        room,
        &DopiszPozycjeProtokoluParams {
            protokol_id: "protokol-wydania".to_string(),
            rodzaj: RodzajPozycjiProtokolu::Licznik,
            nazwa: "licznik energii elektrycznej".to_string(),
            numer: Some("E-778812".to_string()),
            opis_stanu: Some("12 345,6 kWh".to_string()),
            zdjecie_sciezka: Some("protokoly/wydanie/licznik.jpg".to_string()),
            zdjecie_skrot: Some("sha256-licznik".to_string()),
        },
    )
    .expect("add the meter reading");
    sprawa_protokoly::dopisz_pozycje_protokolu(
        room,
        &DopiszPozycjeProtokoluParams {
            protokol_id: "protokol-wydania".to_string(),
            rodzaj: RodzajPozycjiProtokolu::Klucze,
            nazwa: "klucze do drzwi wejściowych".to_string(),
            numer: None,
            opis_stanu: Some("3 sztuki".to_string()),
            zdjecie_sciezka: None,
            zdjecie_skrot: None,
        },
    )
    .expect("add the keys");
    add_received_document(
        room,
        &received_document("podpisany-protokol-wydania", RodzajDokumentu::Protokol),
    );
    sprawa_protokoly::podpisz_protokol(
        room,
        &PodpiszProtokolParams {
            dokument_id: "podpisany-protokol-wydania".to_string(),
            id: "protokol-wydania".to_string(),
        },
    )
    .expect("sign the handover protocol");

    // The deposit goes back to the tenant: an admitted account, a fresh check, the human's
    // consent, the transfer with the bank's confirmation.
    admitted_account(room, "rachunek-najemcy", "najemca");
    checked_and_approved_payout(room, "zwrot-kaucji", "najemca", "rachunek-najemcy");
    add_received_document(
        room,
        &received_document(
            "potwierdzenie-przelewu",
            RodzajDokumentu::PotwierdzenieBankowe,
        ),
    );
    sprawa_wyplaty::odnotuj_wykonanie_wyplaty(
        room,
        &OdnotujWykonanieWyplatyParams {
            wykonal: "tozsamosc-posrednika".to_string(),
            wykonano: "2026-11-20T13:00:00Z".to_string(),
            potwierdzenie_dokument_id: "potwierdzenie-przelewu".to_string(),
            id: "zwrot-kaucji".to_string(),
        },
    )
    .expect("record the transfer");

    // Kontroler leaves a note on the account; the link to the search case of the tenant.
    sprawa_notatki::dopisz_notatke(
        room,
        &DopiszNotatkeParams {
            id: "notatka-rachunek".to_string(),
            od: "tozsamosc-kontrolera".to_string(),
            od_stanowiska: StanowiskoNotatki::Kontroler,
            do_stanowiska: StanowiskoNotatki::OpiekunNajmu,
            do_klienta_uczestnik_id: None,
            tresc: "Rachunek najemcy sprawdzony przed zwrotem kaucji.".to_string(),
            utworzono: "2026-11-20T10:05:00Z".to_string(),
            do_kiedy: None,
            w_odpowiedzi_na: None,
        },
    )
    .expect("add the note");
    sprawa_notatki::wskaz_wiersz_notatki(
        room,
        &WskazWierszNotatkiParams {
            notatka_id: "notatka-rachunek".to_string(),
            dotyczy_tabela: "sprawa_rachunki_wpisy".to_string(),
            dotyczy_id: "rachunek-najemcy".to_string(),
        },
    )
    .expect("point the note to the account");
    sprawa_notatki::oznacz_notatke_jako_wykonana(
        room,
        &OznaczNotatkeJakoWykonanaParams {
            id: "notatka-rachunek".to_string(),
        },
    )
    .expect("mark the note as done");
    sprawa_powiazania::wpisz_powiazanie(
        room,
        &WpiszPowiazanieParams {
            rodzaj: RodzajPowiazania::SprawaZrodlowa,
            typ_teczki: Some("oferta".to_string()),
            oznaczenie: Some("Oferta 2026/041".to_string()),
            odnosnik: "pokoj-oferty-041".to_string(),
            wpisal: "tozsamosc-kierownika-najmu".to_string(),
            wpisano: "2026-11-02T09:15:00Z".to_string(),
        },
    )
    .expect("add the link to the source case");
}

// The whole case ----------------------------------------------------------------------------------

/// Przebieg najmu: od przyjęcia przez umowę, pismo w imieniu klientki i zwrot kaucji
#[test]
fn whole_case() {
    let room = new_room();

    full_case(&room);
    enter_stage(&room, &stage_change("zamknieta", "2026-11-21T09:00:00Z"));

    let stages = sprawa_metryka::etapy(&room).expect("read the stages");
    let etapy: Vec<&str> = stages.iter().map(|row| row.etap.as_str()).collect();
    assert_eq!(etapy, ["przyjeta", "umowa", "zamknieta"]);

    let current_stage = sprawa_metryka::biezacy_etap(&room)
        .expect("read the current stage")
        .expect("the case has a stage");
    assert_eq!(current_stage.etap, "zamknieta");

    let standing_doubts = sprawa_metryka::watpliwosci_stojace(&room).expect("read the doubts");
    assert!(standing_doubts.is_empty());

    let payouts = sprawa_wyplaty::wyplaty(&room).expect("read the payouts");
    assert_eq!(payouts[0].stan, StanWyplaty::Wykonana);

    let documents_to_collect =
        sprawa_dokumenty::dokumenty_do_zebrania(&room).expect("read the documents to collect");
    assert!(documents_to_collect.is_empty());

    let current_consents = sprawa_zgody::zgody_biezace(&room).expect("read the consents");
    let consent_ids: Vec<&str> = current_consents.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(consent_ids, ["zadanie"]);

    let sent = sprawa_korespondencja::wysylki_do_uczestnika(
        &room,
        &WysylkiDoUczestnikaParams {
            adresat_uczestnik_id: "wlascicielka".to_string(),
        },
    )
    .expect("read the letters to the owner");
    assert!(
        sent[0].informacja_rodo && sent[0].ujawnia_agenta,
        "{sent:?}"
    );
}

// Decisions ---------------------------------------------------------------------------------------

/// Decyzja: prośba powstaje jako czekająca
#[test]
fn decision_starts_waiting() {
    let room = new_room();

    ask_for_decision(
        &room,
        &PoprosODecyzjeParams {
            tresc: Some("Zgoda na ogłoszenie bez zdjęć".to_string()),
            ..decision_request("prosba", RodzajDecyzji::Inna)
        },
    );
    let waiting = sprawa_decyzje::decyzje_czekajace(&room).expect("read the waiting decisions");
    assert_eq!(waiting[0].id, "prosba");

    expect::refused(
        room.execute(
            "INSERT INTO sprawa_decyzje_wpisy (id, rodzaj, prosi, prosi_stanowisko, decyduje,
              zlozono, stan, rozstrzygniecie, podpisal, podpis, podpisano)
            VALUES ('od-razu', 'inna', 'agent', 'opiekun_najmu', 'czlowiek',
              '2026-11-03T10:00:00Z', 'rozstrzygnieta', 'zgoda', 'czlowiek', 'podpis',
              '2026-11-03T12:00:00Z')",
            &[],
        ),
        "prośba o decyzję powstaje jako czekająca",
    );
}

/// Decyzja: rozstrzygnięcie ma pełny podpis, a czekająca żadnego
#[test]
fn decision_signature_complete() {
    let room = new_room();
    ask_for_decision(&room, &decision_request("prosba", RodzajDecyzji::Inna));

    // A writer outside the client can leave out the signature that `rozstrzygnij_decyzje`
    // requires. The table refuses the settlement without one.
    let settlement_without_signature = room.execute(
        "UPDATE sprawa_decyzje_wpisy
        SET stan = 'rozstrzygnieta',
          rozstrzygniecie = :rozstrzygniecie,
          warunek = :warunek,
          uzasadnienie = :uzasadnienie,
          podpisal = :podpisal,
          za_kogo = :za_kogo,
          podpis = :podpis,
          podpisano = :podpisano
        WHERE id = :id",
        &[
            ("rozstrzygniecie", text("zgoda")),
            ("podpisal", text("tozsamosc-posrednika")),
            ("podpis", SqlValue::Null),
            ("podpisano", text("2026-11-03T12:00:00Z")),
            ("id", text("prosba")),
        ],
    );
    expect::refused(settlement_without_signature, "podpis_w_komplecie");
    expect::refused(
        room.execute(
            "UPDATE sprawa_decyzje_wpisy
            SET rozstrzygniecie = 'zgoda', podpisal = 'czlowiek', podpis = 'podpis',
              podpisano = '2026-11-03T12:00:00Z'
            WHERE id = 'prosba'",
            &[],
        ),
        "podpis_tylko_przy_rozstrzygnieciu",
    );

    settle_decision(
        &room,
        &RozstrzygnijDecyzjeParams {
            za_kogo: Some("tozsamosc-nieobecnego-posrednika".to_string()),
            ..settlement("prosba", Rozstrzygniecie::Zgoda, "2026-11-03T12:00:00Z")
        },
    );
    let decisions = sprawa_decyzje::decyzje(&room).expect("read the decisions");
    assert_eq!(
        decisions[0].za_kogo.as_deref(),
        Some("tozsamosc-nieobecnego-posrednika")
    );
}

/// Decyzja: rozstrzygnięcie zapada po złożeniu prośby
#[test]
fn decision_not_before_request() {
    let room = new_room();
    ask_for_decision(&room, &decision_request("prosba", RodzajDecyzji::Inna));

    let refusal_before_request =
        settlement("prosba", Rozstrzygniecie::Odmowa, "2026-11-03T09:00:00Z");
    expect::refused(
        sprawa_decyzje::rozstrzygnij_decyzje(&room, &refusal_before_request),
        "rozstrzygniecie_nie_przed_zlozeniem",
    );

    settle_decision(
        &room,
        &settlement("prosba", Rozstrzygniecie::Odmowa, "2026-11-03T10:00:00Z"),
    );
}

/// Decyzja: rozstrzygniętej i zbędnej się nie zmienia; nowa zastępuje poprzednią
#[test]
fn decision_is_final() {
    let room = new_room();
    approved_decision(
        &room,
        &decision_request("pierwsza", RodzajDecyzji::Inna),
        "2026-11-03T12:00:00Z",
    );

    let first_unneeded = UznajProsbeZaZbednaParams {
        powod_zbednosci: "sprawa się zmieniła".to_string(),
        id: "pierwsza".to_string(),
    };
    expect::refused(
        sprawa_decyzje::uznaj_prosbe_za_zbedna(&room, &first_unneeded),
        "rozstrzygniętej ani zbędnej decyzji się nie zmienia",
    );
    expect::refused(
        room.execute(
            "UPDATE sprawa_decyzje_wpisy SET uzasadnienie = 'dopisek' WHERE id = 'pierwsza'",
            &[],
        ),
        "rozstrzygniętej ani zbędnej decyzji się nie zmienia",
    );

    ask_for_decision(&room, &decision_request("zbedna", RodzajDecyzji::Inna));
    sprawa_decyzje::uznaj_prosbe_za_zbedna(
        &room,
        &UznajProsbeZaZbednaParams {
            powod_zbednosci: "klientka wycofała prośbę".to_string(),
            id: "zbedna".to_string(),
        },
    )
    .expect("find the request unneeded");

    let unneeded_settled = settlement("zbedna", Rozstrzygniecie::Zgoda, "2026-11-03T12:00:00Z");
    expect::refused(
        sprawa_decyzje::rozstrzygnij_decyzje(&room, &unneeded_settled),
        "rozstrzygniętej ani zbędnej decyzji się nie zmienia",
    );
}

/// Decyzja: nowa zastępuje obowiązującą decyzję tego samego rodzaju o tym samym wierszu
#[test]
fn decision_replaced_by_newer() {
    let room = new_room();
    approved_decision(
        &room,
        &decision_request_about(
            "pierwsza",
            RodzajDecyzji::Inna,
            "sprawa_terminy_wpisy",
            "termin",
        ),
        "2026-11-03T12:00:00Z",
    );
    let refusal = "nowa decyzja zastępuje obowiązującą rozstrzygniętą decyzję tego samego rodzaju";

    // The requests the rule refuses, by an agent to a human on 2026-11-04.
    let replacing_request =
        |decision_id: &str, rodzaj: RodzajDecyzji, dotyczy_id: &str, zastepuje_decyzja_id: &str| {
            PoprosODecyzjeParams {
                prosi: "agent".to_string(),
                decyduje: "czlowiek".to_string(),
                zlozono: "2026-11-04T10:00:00Z".to_string(),
                zastepuje_decyzja_id: Some(zastepuje_decyzja_id.to_string()),
                ..decision_request_about(decision_id, rodzaj, "sprawa_terminy_wpisy", dotyczy_id)
            }
        };

    let of_another_kind = replacing_request(
        "innego-rodzaju",
        RodzajDecyzji::PotwierdzenieTerminu,
        "termin",
        "pierwsza",
    );
    expect::refused(
        sprawa_decyzje::popros_o_decyzje(&room, &of_another_kind),
        refusal,
    );
    let about_another_row = replacing_request(
        "o-innym-wierszu",
        RodzajDecyzji::Inna,
        "inny-termin",
        "pierwsza",
    );
    expect::refused(
        sprawa_decyzje::popros_o_decyzje(&room, &about_another_row),
        refusal,
    );
    ask_for_decision(
        &room,
        &decision_request_about(
            "czekajaca",
            RodzajDecyzji::Inna,
            "sprawa_terminy_wpisy",
            "termin",
        ),
    );
    let replacing_waiting = replacing_request(
        "zastepuje-czekajaca",
        RodzajDecyzji::Inna,
        "termin",
        "czekajaca",
    );
    expect::refused(
        sprawa_decyzje::popros_o_decyzje(&room, &replacing_waiting),
        refusal,
    );

    // Without a newer decision a settled one does not turn into a replaced one.
    expect::refused(
        room.execute(
            "UPDATE sprawa_decyzje_wpisy SET stan = 'zastapiona' WHERE id = 'pierwsza'",
            &[],
        ),
        "rozstrzygniętej ani zbędnej decyzji się nie zmienia",
    );

    // Two requests replace the same decision; only the first one settled takes its place.
    for decision_id in ["druga", "rownolegla"] {
        ask_for_decision(
            &room,
            &PoprosODecyzjeParams {
                zastepuje_decyzja_id: Some("pierwsza".to_string()),
                ..decision_request_about(
                    decision_id,
                    RodzajDecyzji::Inna,
                    "sprawa_terminy_wpisy",
                    "termin",
                )
            },
        );
    }
    let first_state = room
        .value(
            "SELECT stan FROM sprawa_decyzje_wpisy WHERE id = 'pierwsza'",
            &[],
        )
        .expect("read the state of the first decision");
    assert_eq!(
        String::from_sql_value(&first_state).expect("the state is a text"),
        "rozstrzygnieta"
    );
    settle_decision(
        &room,
        &settlement("druga", Rozstrzygniecie::Odmowa, "2026-11-03T12:00:00Z"),
    );
    let decisions_of_deadline = sprawa_decyzje::decyzje_wiersza(
        &room,
        &DecyzjeWierszaParams {
            dotyczy_tabela: "sprawa_terminy_wpisy".to_string(),
            dotyczy_id: "termin".to_string(),
        },
    )
    .expect("read the decisions of the deadline");
    let replaced = decisions_of_deadline
        .iter()
        .find(|row| row.id == "pierwsza")
        .expect("the first decision is among them");
    assert!(
        replaced.stan == StanDecyzji::Zastapiona
            && replaced.rozstrzygniecie == Some(Rozstrzygniecie::Zgoda),
        "{replaced:?}"
    );

    let parallel_settled = settlement("rownolegla", Rozstrzygniecie::Zgoda, "2026-11-03T12:00:00Z");
    expect::refused(
        sprawa_decyzje::rozstrzygnij_decyzje(&room, &parallel_settled),
        refusal,
    );
    expect::refused(
        room.execute(
            "UPDATE sprawa_decyzje_wpisy SET stan = 'rozstrzygnieta' WHERE id = 'pierwsza'",
            &[],
        ),
        "rozstrzygniętej ani zbędnej decyzji się nie zmienia",
    );
}

/// Decyzja: zgoda z warunkiem ma warunek, a prośba zbędna powód
#[test]
fn decision_condition_and_reason() {
    let room = new_room();
    ask_for_decision(&room, &decision_request("z-warunkiem", RodzajDecyzji::Inna));
    ask_for_decision(&room, &decision_request("odmowa", RodzajDecyzji::Inna));

    let consent_without_condition = settlement(
        "z-warunkiem",
        Rozstrzygniecie::ZgodaZWarunkiem,
        "2026-11-03T12:00:00Z",
    );
    expect::refused(
        sprawa_decyzje::rozstrzygnij_decyzje(&room, &consent_without_condition),
        "zgoda_z_warunkiem_ma_warunek",
    );
    let refusal_with_condition = RozstrzygnijDecyzjeParams {
        warunek: Some("po obniżce ceny".to_string()),
        ..settlement("odmowa", Rozstrzygniecie::Odmowa, "2026-11-03T12:00:00Z")
    };
    expect::refused(
        sprawa_decyzje::rozstrzygnij_decyzje(&room, &refusal_with_condition),
        "zgoda_z_warunkiem_ma_warunek",
    );
    settle_decision(
        &room,
        &RozstrzygnijDecyzjeParams {
            warunek: Some("ogłoszenie dopiero po sesji zdjęciowej".to_string()),
            uzasadnienie: Some("klientka chce zdjęć".to_string()),
            ..settlement(
                "z-warunkiem",
                Rozstrzygniecie::ZgodaZWarunkiem,
                "2026-11-03T12:00:00Z",
            )
        },
    );

    ask_for_decision(&room, &decision_request("zbedna", RodzajDecyzji::Inna));
    expect::refused(
        room.execute(
            "UPDATE sprawa_decyzje_wpisy SET stan = 'zbedna' WHERE id = 'zbedna'",
            &[],
        ),
        "zbedna_ma_powod",
    );
    sprawa_decyzje::uznaj_prosbe_za_zbedna(
        &room,
        &UznajProsbeZaZbednaParams {
            powod_zbednosci: "termin się zmienił".to_string(),
            id: "zbedna".to_string(),
        },
    )
    .expect("find the request unneeded");
}

/// Decyzja o zdjęciu wątpliwości nie niesie treści ani uzasadnienia (art. 54 AML)
#[test]
fn decision_aml_without_content() {
    let room = new_room();

    let request_with_content = PoprosODecyzjeParams {
        prosi: "kontroler".to_string(),
        prosi_stanowisko: ProsiStanowisko::Kontroler,
        decyduje: "czlowiek".to_string(),
        tresc: Some("beneficjent wyjaśniony".to_string()),
        ..decision_request_about(
            "z-trescia",
            RodzajDecyzji::ZdjecieWatpliwosci,
            "sprawa_metryka_watpliwosci",
            "watpliwosc",
        )
    };
    expect::refused(
        sprawa_decyzje::popros_o_decyzje(&room, &request_with_content),
        "decyzja_aml_bez_tresci",
    );

    ask_for_decision(
        &room,
        &decision_request_about(
            "bez-tresci",
            RodzajDecyzji::ZdjecieWatpliwosci,
            "sprawa_metryka_watpliwosci",
            "watpliwosc",
        ),
    );
    let settlement_with_reason = RozstrzygnijDecyzjeParams {
        uzasadnienie: Some("źródło środków potwierdzone".to_string()),
        ..settlement("bez-tresci", Rozstrzygniecie::Zgoda, "2026-11-03T12:00:00Z")
    };
    expect::refused(
        sprawa_decyzje::rozstrzygnij_decyzje(&room, &settlement_with_reason),
        "decyzja_aml_bez_tresci",
    );
    approve(&room, "bez-tresci", "2026-11-03T12:00:00Z");
}

/// Decyzja: wskazany wiersz to tabela z identyfikatorem, a skrót tylko przy wierszu
#[test]
fn decision_row_reference() {
    let room = new_room();

    let table_without_id = PoprosODecyzjeParams {
        prosi: "agent".to_string(),
        decyduje: "czlowiek".to_string(),
        dotyczy_tabela: Some("sprawa_rachunki_wpisy".to_string()),
        ..decision_request("bez-id", RodzajDecyzji::Inna)
    };
    expect::refused(
        sprawa_decyzje::popros_o_decyzje(&room, &table_without_id),
        "dotyczy_tabela_razem_z_id",
    );
    let hash_without_row = PoprosODecyzjeParams {
        prosi: "agent".to_string(),
        decyduje: "czlowiek".to_string(),
        dotyczy_skrot: Some("sha256-czegos".to_string()),
        ..decision_request("skrot-bez-wiersza", RodzajDecyzji::Inna)
    };
    expect::refused(
        sprawa_decyzje::popros_o_decyzje(&room, &hash_without_row),
        "skrot_tylko_przy_wierszu",
    );

    ask_for_decision(
        &room,
        &PoprosODecyzjeParams {
            dotyczy_skrot: Some("sha256-cena-1".to_string()),
            ..decision_request_about("o-wierszu", RodzajDecyzji::Inna, "oferta_ceny", "cena-1")
        },
    );
    let decisions_of_price = sprawa_decyzje::decyzje_wiersza(
        &room,
        &DecyzjeWierszaParams {
            dotyczy_tabela: "oferta_ceny".to_string(),
            dotyczy_id: "cena-1".to_string(),
        },
    )
    .expect("read the decisions of the price");
    assert!(!decisions_of_price.is_empty());
}

// Case record, stages and doubts ------------------------------------------------------------------

/// Metryka: typ pokoju jest nazwą folderu, a metryki się nie zmienia
#[test]
fn record_room_type() {
    let room = new_room();

    let capitalised_room_type = ZalozMetrykeParams {
        oznaczenie: "Najem 2026/017".to_string(),
        typ_pokoju: "Najem".to_string(),
        przyjal: "czlowiek".to_string(),
        przyjeto: "2026-11-02T09:00:00Z".to_string(),
    };
    expect::refused(
        sprawa_metryka::zaloz_metryke(&room, &capitalised_room_type),
        "typ_pokoju",
    );

    open_case(&room, "najem");
    let changes = [
        ("typ_pokoju", "oferta"),
        ("oznaczenie", "Najem 2026/017, ul. Polna 7 m. 14"),
        ("przyjal", "ktos-inny"),
        ("przyjeto", "2026-11-01T09:00:00Z"),
    ];
    for (column, value) in changes {
        let sql = format!("UPDATE sprawa_metryka_wpis SET {column} = :value");

        expect::refused(
            room.execute(&sql, &[("value", text(value))]),
            "metryki teczki się nie zmienia",
        );
    }
}

/// Etapy: pierwszy to przyjęta, a dziennika się nie zmienia
#[test]
fn stages_start_accepted() {
    let room = new_room();

    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &stage_change("umowa", "2026-11-02T09:00:00Z")),
        "pierwszy etap sprawy to przyjeta",
    );

    open_case(&room, "najem");
    enter_stage(&room, &stage_change("umowa", "2026-11-02T12:00:00Z"));
    let current_stage = sprawa_metryka::biezacy_etap(&room)
        .expect("read the current stage")
        .expect("the case has a stage");
    assert_eq!(current_stage.etap, "umowa");

    expect::refused(
        room.execute(
            "UPDATE sprawa_metryka_etapy SET etap = 'przyjeta' WHERE etap = 'umowa'",
            &[],
        ),
        "wpisu w dzienniku etapów się nie zmienia",
    );
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &stage_change("Umowa", "2026-11-02T13:00:00Z")),
        "etap",
    );
}

/// Etapy: przerwanie wymaga zatwierdzonej decyzji przerwanie_sprawy
#[test]
fn stage_interrupted_needs_decision() {
    let room = new_room();
    open_case(&room, "najem");
    let refusal = "etap przerwana wymaga zatwierdzonej decyzji przerwanie_sprawy";
    let interruption_by = |decyzja_id: Option<&str>| ZmienEtapSprawyParams {
        decyzja_id: decyzja_id.map(str::to_string),
        ..stage_change("przerwana", "2026-11-05T09:00:00Z")
    };

    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &interruption_by(None)),
        refusal,
    );

    ask_for_decision(
        &room,
        &decision_request("odmowa-przerwania", RodzajDecyzji::PrzerwanieSprawy),
    );
    settle_decision(
        &room,
        &settlement(
            "odmowa-przerwania",
            Rozstrzygniecie::Odmowa,
            "2026-11-03T12:00:00Z",
        ),
    );
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &interruption_by(Some("odmowa-przerwania"))),
        refusal,
    );

    approved_decision(
        &room,
        &decision_request("inna-decyzja", RodzajDecyzji::Inna),
        "2026-11-03T12:00:00Z",
    );
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &interruption_by(Some("inna-decyzja"))),
        refusal,
    );

    approved_decision(
        &room,
        &decision_request("przerwanie", RodzajDecyzji::PrzerwanieSprawy),
        "2026-11-03T12:00:00Z",
    );
    enter_stage(&room, &interruption_by(Some("przerwanie")));
}

/// Etapy: przerwana sprawa wraca do etapu sprzed przerwania za decyzją wznowienie_sprawy albo się
/// zamyka (T1)
#[test]
fn stage_resumed_after_interruption() {
    let room = new_room();
    open_case(&room, "najem");
    enter_stage(&room, &stage_change("umowa", "2026-11-03T09:00:00Z"));
    approved_decision(
        &room,
        &decision_request("przerwanie", RodzajDecyzji::PrzerwanieSprawy),
        "2026-11-04T12:00:00Z",
    );
    // A consent to resume given before the interruption concerns another situation.
    approved_decision(
        &room,
        &decision_request("wczesne-wznowienie", RodzajDecyzji::WznowienieSprawy),
        "2026-11-04T13:00:00Z",
    );
    let stage_by = |etap: &str, od: &str, decyzja_id: Option<&str>| ZmienEtapSprawyParams {
        decyzja_id: decyzja_id.map(str::to_string),
        ..stage_change(etap, od)
    };
    enter_stage(
        &room,
        &stage_by("przerwana", "2026-11-05T09:00:00Z", Some("przerwanie")),
    );
    let decision = "powrót z etapu przerwana wymaga zatwierdzonej decyzji wznowienie_sprawy";
    let other_stage =
        "po etapie przerwana przychodzi zamknieta albo powrót do etapu sprzed przerwania";

    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &stage_by("umowa", "2026-11-06T09:00:00Z", None)),
        decision,
    );
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_by("umowa", "2026-11-06T09:00:00Z", Some("wczesne-wznowienie")),
        ),
        decision,
    );
    ask_for_decision(
        &room,
        &decision_request("odmowa-wznowienia", RodzajDecyzji::WznowienieSprawy),
    );
    settle_decision(
        &room,
        &settlement(
            "odmowa-wznowienia",
            Rozstrzygniecie::Odmowa,
            "2026-11-06T08:00:00Z",
        ),
    );
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_by("umowa", "2026-11-06T09:00:00Z", Some("odmowa-wznowienia")),
        ),
        decision,
    );

    approved_decision(
        &room,
        &decision_request("wznowienie", RodzajDecyzji::WznowienieSprawy),
        "2026-11-06T08:00:00Z",
    );
    for etap in ["przyjeta", "inny_etap"] {
        expect::refused(
            sprawa_metryka::zmien_etap_sprawy(
                &room,
                &stage_by(etap, "2026-11-06T09:00:00Z", Some("wznowienie")),
            ),
            other_stage,
        );
    }
    enter_stage(
        &room,
        &stage_by("umowa", "2026-11-06T09:00:00Z", Some("wznowienie")),
    );

    // The second interruption needs its own consent, and so does the second resumption.
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_by("przerwana", "2026-11-07T09:00:00Z", Some("przerwanie")),
        ),
        "jedna decyzja stoi za jednym wejściem w etap",
    );
    approved_decision(
        &room,
        &decision_request("drugie-przerwanie", RodzajDecyzji::PrzerwanieSprawy),
        "2026-11-06T09:30:00Z",
    );
    // The second interruption is entered with an earlier time, so the first consent to resume
    // passes the time check and only the rule of one entry per decision refuses it.
    enter_stage(
        &room,
        &stage_by(
            "przerwana",
            "2026-11-06T07:00:00Z",
            Some("drugie-przerwanie"),
        ),
    );
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_by("umowa", "2026-11-07T09:00:00Z", Some("wznowienie")),
        ),
        "jedna decyzja stoi za jednym wejściem w etap",
    );

    enter_stage(&room, &stage_change("zamknieta", "2026-11-08T09:00:00Z"));
    let stages = sprawa_metryka::etapy(&room).expect("read the stages");
    let etapy: Vec<&str> = stages.iter().map(|row| row.etap.as_str()).collect();
    assert_eq!(
        etapy,
        [
            "przyjeta",
            "umowa",
            "przerwana",
            "umowa",
            "przerwana",
            "zamknieta"
        ]
    );
}

/// Etapy: po zamknięciu teczki nie ma kolejnych etapów
#[test]
fn no_stage_after_closing() {
    let room = new_room();
    open_case(&room, "najem");

    enter_stage(&room, &stage_change("zamknieta", "2026-11-30T09:00:00Z"));
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &stage_change("umowa", "2026-12-01T09:00:00Z")),
        "po etapie zamknieta sprawa nie ma kolejnych etapów",
    );
}

/// Wątpliwość: powstaje stojąca, zdejmuje ją decyzja, która ją wskazuje i zapadła po niej
#[test]
fn doubt_lifted_by_decision() {
    let room = new_room();
    open_case(&room, "najem");

    expect::refused(
        room.execute(
            "INSERT INTO sprawa_metryka_watpliwosci (id, postawil, postawiono, zdjecie_decyzja_id)
            VALUES ('od-razu-zdjeta', 'kontroler', '2026-11-03T09:00:00Z', 'decyzja')",
            &[],
        ),
        "znacznik wątpliwości powstaje stojący",
    );

    let lifting_request = |decision_id: &str, doubt_id: &str, zlozono: &str| PoprosODecyzjeParams {
        zlozono: zlozono.to_string(),
        ..decision_request_about(
            decision_id,
            RodzajDecyzji::ZdjecieWatpliwosci,
            "sprawa_metryka_watpliwosci",
            doubt_id,
        )
    };
    let lifting = |decision_id: &str| ZdejmijWatpliwoscParams {
        zdjecie_decyzja_id: decision_id.to_string(),
        id: "watpliwosc".to_string(),
    };

    // A decision signed before the doubt was raised does not lift it.
    ask_for_decision(
        &room,
        &lifting_request("wczesniejsza", "watpliwosc", "2026-11-03T08:00:00Z"),
    );
    approve(&room, "wczesniejsza", "2026-11-03T08:30:00Z");
    for (kontroler, postawiono) in [
        ("kontroler-sprzedajacej", "09"),
        ("kontroler-kupujacego", "10"),
    ] {
        let doubt_id = if kontroler == "kontroler-sprzedajacej" {
            "watpliwosc"
        } else {
            "druga"
        };
        let doubt = PostawWatpliwoscParams {
            id: doubt_id.to_string(),
            postawil: kontroler.to_string(),
            postawiono: format!("2026-11-03T{postawiono}:00:00Z"),
        };

        sprawa_metryka::postaw_watpliwosc(&room, &doubt).expect("raise a doubt");
    }
    let standing = sprawa_metryka::watpliwosci_stojace(&room).expect("read the doubts");
    assert_eq!(standing.len(), 2);

    expect::refused(
        sprawa_metryka::zdejmij_watpliwosc(&room, &lifting("wczesniejsza")),
        "znacznik zdejmuje tylko zatwierdzona decyzja zdjecie_watpliwosci",
    );

    // A decision on another doubt does not lift this one.
    ask_for_decision(
        &room,
        &lifting_request("o-drugiej", "druga", "2026-11-03T11:00:00Z"),
    );
    approve(&room, "o-drugiej", "2026-11-03T12:00:00Z");
    expect::refused(
        sprawa_metryka::zdejmij_watpliwosc(&room, &lifting("o-drugiej")),
        "znacznik zdejmuje tylko zatwierdzona decyzja zdjecie_watpliwosci",
    );

    ask_for_decision(
        &room,
        &lifting_request("o-pierwszej", "watpliwosc", "2026-11-03T11:00:00Z"),
    );
    approve(&room, "o-pierwszej", "2026-11-03T12:00:00Z");
    sprawa_metryka::zdejmij_watpliwosc(&room, &lifting("o-pierwszej")).expect("lift the doubt");
    let standing = sprawa_metryka::watpliwosci_stojace(&room).expect("read the doubts");
    let standing_ids: Vec<&str> = standing.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(standing_ids, ["druga"]);

    expect::refused(
        sprawa_metryka::zdejmij_watpliwosc(&room, &lifting("o-drugiej")),
        "znacznika wątpliwości się nie zmienia",
    );
    expect::refused(
        room.execute(
            "UPDATE sprawa_metryka_watpliwosci SET postawil = 'ktos-inny' WHERE id = 'druga'",
            &[],
        ),
        "znacznika wątpliwości się nie zmienia",
    );
}

// Participants ------------------------------------------------------------------------------------

/// The change of the identifying data of the participant of `params` to the values in it.
fn data_change_of(params: WpiszUczestnikaParams) -> ZmienDaneUczestnikaParams {
    ZmienDaneUczestnikaParams {
        imiona: params.imiona,
        nazwisko: params.nazwisko,
        imie_ojca: params.imie_ojca,
        imie_matki: params.imie_matki,
        pesel: params.pesel,
        pesel_nie_nadano: params.pesel_nie_nadano,
        konsument: params.konsument,
        nazwa: params.nazwa,
        forma_prawna: params.forma_prawna,
        krs: params.krs,
        ceidg: params.ceidg,
        nip: params.nip,
        regon: params.regon,
        id: params.id,
    }
}

/// Uczestnik: osoba ma imiona i nazwisko, podmiot nazwę i nie ma danych osoby
#[test]
fn participant_names() {
    let room = new_room();

    let person_without_surname = WpiszUczestnikaParams {
        nazwisko: None,
        ..person("osoba")
    };
    expect::refused(
        sprawa_uczestnicy::wpisz_uczestnika(&room, &person_without_surname),
        "osoba_ma_imie_i_nazwisko",
    );
    let company_without_name = WpiszUczestnikaParams {
        rodzaj_podmiotu: RodzajPodmiotu::OsobaPrawna,
        imiona: None,
        nazwisko: None,
        ..person("osoba")
    };
    expect::refused(
        sprawa_uczestnicy::wpisz_uczestnika(&room, &company_without_name),
        "podmiot_ma_nazwe",
    );
    let company_with_person_data = WpiszUczestnikaParams {
        rodzaj_podmiotu: RodzajPodmiotu::OsobaPrawna,
        nazwa: Some("Polna sp. z o.o.".to_string()),
        ..person("osoba")
    };
    expect::refused(
        sprawa_uczestnicy::wpisz_uczestnika(&room, &company_with_person_data),
        "podmiot_bez_danych_osoby",
    );

    add_person(&room, &person("osoba"));
    add_company(&room, "spolka");
}

/// Uczestnik: PESEL ma 11 cyfr, a „PESEL nie nadano” go wyklucza
#[test]
fn participant_pesel() {
    let room = new_room();

    for pesel in ["8001011234", "8001011234X"] {
        let wrong_pesel = WpiszUczestnikaParams {
            pesel: Some(pesel.to_string()),
            ..person("osoba")
        };

        expect::refused(
            sprawa_uczestnicy::wpisz_uczestnika(&room, &wrong_pesel),
            "pesel_ma_11_cyfr",
        );
    }
    let pesel_not_given_but_present = WpiszUczestnikaParams {
        pesel: Some("80010112345".to_string()),
        pesel_nie_nadano: true,
        ..person("osoba")
    };
    expect::refused(
        sprawa_uczestnicy::wpisz_uczestnika(&room, &pesel_not_given_but_present),
        "pesel_nie_nadano_wyklucza_pesel",
    );

    add_person(
        &room,
        &WpiszUczestnikaParams {
            pesel: Some("80010112345".to_string()),
            ..person("z-peselem")
        },
    );
    add_person(
        &room,
        &WpiszUczestnikaParams {
            pesel_nie_nadano: true,
            ..person("cudzoziemiec")
        },
    );
}

/// Uczestnik: rodzaj podmiotu się nie zmienia
#[test]
fn participant_kind_fixed() {
    let room = new_room();
    add_person(&room, &person("malzonek"));
    add_person(
        &room,
        &WpiszUczestnikaParams {
            imiona: Some("Ewa".to_string()),
            ..person("zona")
        },
    );
    sprawa_uczestnicy::wpisz_adres_uczestnika(&room, &address("zona", AdresyRodzaj::Zamieszkanie))
        .expect("add the home address");
    sprawa_uczestnicy::wpisz_obywatelstwo_uczestnika(
        &room,
        &WpiszObywatelstwoUczestnikaParams {
            uczestnik_id: "zona".to_string(),
            panstwo: "PL".to_string(),
        },
    )
    .expect("add the citizenship");
    sprawa_uczestnicy::nadaj_role_uczestnikowi(
        &room,
        &NadajRoleUczestnikowiParams {
            uczestnik_id: "zona".to_string(),
            rola: "malzonek".to_string(),
            dotyczy_uczestnik_id: Some("malzonek".to_string()),
        },
    )
    .expect("give the role of a spouse");

    // The workaround from the audit: a person with a home address, a citizenship and a spouse
    // turns into a company.
    expect::refused(
        room.execute(
            "UPDATE sprawa_uczestnicy_wpisy
            SET rodzaj_podmiotu = 'osoba_prawna', nazwa = 'X', imiona = NULL, nazwisko = NULL
            WHERE id = 'zona'",
            &[],
        ),
        "rodzaj podmiotu uczestnika się nie zmienia",
    );
    let new_surname = WpiszUczestnikaParams {
        nazwisko: Some("Nowak".to_string()),
        ..person("zona")
    };
    sprawa_uczestnicy::zmien_dane_uczestnika(&room, &data_change_of(new_surname))
        .expect("change the participant's data");
}

/// Uczestnik: ustrój majątkowy tylko przy małżeństwie, konsument tylko osoba fizyczna
#[test]
fn participant_family_and_consumer() {
    let room = new_room();
    add_person(&room, &person("osoba"));

    let property_regime_after_divorce = UstalSytuacjeRodzinnaUczestnikaParams {
        stan_cywilny: Some(StanCywilny::Rozwiedziony),
        ustroj_majatkowy: Some(UstrojMajatkowy::WspolnoscUstawowa),
        id: "osoba".to_string(),
    };
    expect::refused(
        sprawa_uczestnicy::ustal_sytuacje_rodzinna_uczestnika(
            &room,
            &property_regime_after_divorce,
        ),
        "ustroj_tylko_w_malzenstwie",
    );
    sprawa_uczestnicy::ustal_sytuacje_rodzinna_uczestnika(
        &room,
        &UstalSytuacjeRodzinnaUczestnikaParams {
            stan_cywilny: Some(StanCywilny::WSeparacji),
            ustroj_majatkowy: Some(UstrojMajatkowy::RozdzielnoscZMocyPrawa),
            id: "osoba".to_string(),
        },
    )
    .expect("set the family situation");

    let consumer_company = WpiszUczestnikaParams {
        rodzaj_podmiotu: RodzajPodmiotu::OsobaPrawna,
        imiona: None,
        nazwisko: None,
        nazwa: Some("Polna sp. z o.o.".to_string()),
        konsument: true,
        ..person("spolka")
    };
    expect::refused(
        sprawa_uczestnicy::wpisz_uczestnika(&room, &consumer_company),
        "konsument_tylko_osoba_fizyczna",
    );
    add_person(
        &room,
        &WpiszUczestnikaParams {
            konsument: true,
            ..person("konsumentka")
        },
    );

    let entrepreneur_in_both_registers = WpiszUczestnikaParams {
        nazwa: Some("Kowalska Nieruchomości".to_string()),
        krs: Some("1".to_string()),
        ceidg: true,
        ..person("przedsiebiorca")
    };
    expect::refused(
        sprawa_uczestnicy::wpisz_uczestnika(&room, &entrepreneur_in_both_registers),
        "krs_albo_ceidg",
    );
}

/// Rola: małżonek wskazuje osobę fizyczną; pełnomocnika i reprezentanta daje umocowanie; rola
/// kończy się raz, z powodem
#[test]
fn participant_roles() {
    let room = new_room();
    add_person(&room, &person("zona"));
    add_person(
        &room,
        &WpiszUczestnikaParams {
            imiona: Some("Piotr".to_string()),
            ..person("maz")
        },
    );
    add_company(&room, "spolka");

    let role = |uczestnik_id: &str, rola: &str, dotyczy_uczestnik_id: Option<&str>| {
        NadajRoleUczestnikowiParams {
            uczestnik_id: uczestnik_id.to_string(),
            rola: rola.to_string(),
            dotyczy_uczestnik_id: dotyczy_uczestnik_id.map(str::to_string),
        }
    };

    expect::refused(
        sprawa_uczestnicy::nadaj_role_uczestnikowi(&room, &role("maz", "malzonek", None)),
        "malzonek_wskazuje_osobe",
    );
    expect::refused(
        sprawa_uczestnicy::nadaj_role_uczestnikowi(&room, &role("maz", "malzonek", Some("spolka"))),
        "małżonek i osoba, której jest małżonkiem, to osoby fizyczne",
    );
    expect::refused(
        sprawa_uczestnicy::nadaj_role_uczestnikowi(&room, &role("maz", "malzonek", Some("maz"))),
        "rola_nie_dotyczy_siebie",
    );
    sprawa_uczestnicy::nadaj_role_uczestnikowi(&room, &role("maz", "malzonek", Some("zona")))
        .expect("give the role of a spouse");

    for rola in ["pelnomocnik", "reprezentant"] {
        expect::refused(
            sprawa_uczestnicy::nadaj_role_uczestnikowi(&room, &role("zona", rola, None)),
            "pelnomocnik_i_reprezentant_z_umocowan",
        );
    }
    expect::refused(
        sprawa_uczestnicy::nadaj_role_uczestnikowi(&room, &role("zona", "Klientka", None)),
        "rola",
    );
    sprawa_uczestnicy::nadaj_role_uczestnikowi(&room, &role("zona", "klient", None))
        .expect("give the role of a client");

    expect::refused(
        room.execute(
            "UPDATE sprawa_uczestnicy_role SET rola = 'druga_strona' WHERE rola = 'klient'",
            &[],
        ),
        "roli w teczce się nie zmienia",
    );

    let role_id = integer_value(
        &room,
        "SELECT id FROM sprawa_uczestnicy_role WHERE rola = 'klient'",
    );
    expect::refused(
        room.execute(
            "INSERT INTO sprawa_uczestnicy_role (uczestnik_id, rola, zakonczono, powod_zakonczenia)
            VALUES ('maz', 'klient', '2026-11-17', 'pomyłka')",
            &[],
        ),
        "rola powstaje jako obowiązująca",
    );
    let end_without_reason = ZakonczRoleUczestnikaParams {
        zakonczono: Some("2026-11-17".to_string()),
        powod_zakonczenia: None,
        id: role_id,
    };
    expect::refused(
        sprawa_uczestnicy::zakoncz_role_uczestnika(&room, &end_without_reason),
        "zakonczenie_roli_z_powodem",
    );
    sprawa_uczestnicy::zakoncz_role_uczestnika(
        &room,
        &ZakonczRoleUczestnikaParams {
            zakonczono: Some("2026-11-17".to_string()),
            powod_zakonczenia: Some("pomyłka: klientem jest mąż".to_string()),
            id: role_id,
        },
    )
    .expect("end the role");
    let second_end = ZakonczRoleUczestnikaParams {
        zakonczono: Some("2026-11-18".to_string()),
        powod_zakonczenia: Some("inny powód".to_string()),
        id: role_id,
    };
    expect::refused(
        sprawa_uczestnicy::zakoncz_role_uczestnika(&room, &second_end),
        "rola może się tylko raz zakończyć",
    );
}

/// The address of the kind `rodzaj` of the participant in Wrocław, Poland.
fn address(uczestnik_id: &str, rodzaj: AdresyRodzaj) -> WpiszAdresUczestnikaParams {
    WpiszAdresUczestnikaParams {
        uczestnik_id: uczestnik_id.to_string(),
        rodzaj,
        kraj: "PL".to_string(),
        miejscowosc: "Wrocław".to_string(),
        ulica: None,
        numer_budynku: None,
        numer_lokalu: None,
        kod_pocztowy: None,
    }
}

/// Adres i obywatelstwo: zamieszkanie i obywatelstwo ma osoba, siedzibę podmiot
#[test]
fn participant_addresses() {
    let room = new_room();
    add_person(&room, &person("osoba"));
    add_company(&room, "spolka");

    expect::refused(
        sprawa_uczestnicy::wpisz_adres_uczestnika(
            &room,
            &address("spolka", AdresyRodzaj::Zamieszkanie),
        ),
        "miejsce zamieszkania ma tylko osoba fizyczna",
    );
    expect::refused(
        sprawa_uczestnicy::wpisz_adres_uczestnika(&room, &address("osoba", AdresyRodzaj::Siedziba)),
        "miejsce zamieszkania ma tylko osoba fizyczna",
    );
    let allowed_addresses = [
        address("osoba", AdresyRodzaj::Zamieszkanie),
        address("spolka", AdresyRodzaj::Siedziba),
        address("spolka", AdresyRodzaj::DoKorespondencji),
    ];
    for allowed_address in &allowed_addresses {
        sprawa_uczestnicy::wpisz_adres_uczestnika(&room, allowed_address).expect("add an address");
    }
    expect::refused(
        room.execute(
            "UPDATE sprawa_uczestnicy_adresy SET rodzaj = 'zamieszkanie'
            WHERE uczestnik_id = 'spolka' AND rodzaj = 'do_korespondencji'",
            &[],
        ),
        "miejsce zamieszkania ma tylko osoba fizyczna",
    );

    let citizenship_of_company = WpiszObywatelstwoUczestnikaParams {
        uczestnik_id: "spolka".to_string(),
        panstwo: "PL".to_string(),
    };
    expect::refused(
        sprawa_uczestnicy::wpisz_obywatelstwo_uczestnika(&room, &citizenship_of_company),
        "obywatelstwo ma tylko osoba fizyczna",
    );
    sprawa_uczestnicy::wpisz_obywatelstwo_uczestnika(
        &room,
        &WpiszObywatelstwoUczestnikaParams {
            uczestnik_id: "osoba".to_string(),
            panstwo: "UA".to_string(),
        },
    )
    .expect("add the citizenship");
    expect::refused(
        room.execute(
            "UPDATE sprawa_uczestnicy_obywatelstwa SET uczestnik_id = 'spolka'",
            &[],
        ),
        "obywatelstwo ma tylko osoba fizyczna",
    );
}

/// Uczestnik: zmiana danych identyfikujących podnosi wersję, kontakt i rodzina nie (D6)
#[test]
fn participant_version() {
    let room = new_room();
    let with_pesel = WpiszUczestnikaParams {
        pesel: Some("80010112345".to_string()),
        ..person("osoba")
    };
    add_person(&room, &with_pesel);
    assert_eq!(version_of(&room, "osoba"), 1);

    sprawa_uczestnicy::zmien_kontakt_uczestnika(
        &room,
        &ZmienKontaktUczestnikaParams {
            telefon: Some("+48 600 000 000".to_string()),
            email: None,
            kanal_kontaktu: Some(KanalKontaktu::Sms),
            id: "osoba".to_string(),
        },
    )
    .expect("change the contact");
    sprawa_uczestnicy::ustal_sytuacje_rodzinna_uczestnika(
        &room,
        &UstalSytuacjeRodzinnaUczestnikaParams {
            stan_cywilny: Some(StanCywilny::WMalzenstwie),
            ustroj_majatkowy: Some(UstrojMajatkowy::WspolnoscUstawowa),
            id: "osoba".to_string(),
        },
    )
    .expect("set the family situation");
    sprawa_uczestnicy::wpisz_adres_uczestnika(
        &room,
        &WpiszAdresUczestnikaParams {
            miejscowosc: "Opole".to_string(),
            ..address("osoba", AdresyRodzaj::DoKorespondencji)
        },
    )
    .expect("add the address for correspondence");
    assert_eq!(version_of(&room, "osoba"), 1);

    let new_surname = WpiszUczestnikaParams {
        nazwisko: Some("Nowak".to_string()),
        ..with_pesel
    };
    sprawa_uczestnicy::zmien_dane_uczestnika(&room, &data_change_of(new_surname))
        .expect("change the participant's data");
    assert_eq!(version_of(&room, "osoba"), 2);

    sprawa_uczestnicy::wpisz_adres_uczestnika(&room, &address("osoba", AdresyRodzaj::Zamieszkanie))
        .expect("add the home address");
    sprawa_uczestnicy::wpisz_obywatelstwo_uczestnika(
        &room,
        &WpiszObywatelstwoUczestnikaParams {
            uczestnik_id: "osoba".to_string(),
            panstwo: "PL".to_string(),
        },
    )
    .expect("add the citizenship");
    assert_eq!(version_of(&room, "osoba"), 4);

    // D14: a mistaken address or citizenship may be deleted, and that changes the version too.
    let address_id = integer_value(
        &room,
        "SELECT id FROM sprawa_uczestnicy_adresy WHERE rodzaj = 'zamieszkanie'",
    );
    sprawa_uczestnicy::wykresl_adres_uczestnika(
        &room,
        &WykreslAdresUczestnikaParams { id: address_id },
    )
    .expect("delete the address");
    let citizenship_id = integer_value(&room, "SELECT id FROM sprawa_uczestnicy_obywatelstwa");
    sprawa_uczestnicy::wykresl_obywatelstwo_uczestnika(
        &room,
        &WykreslObywatelstwoUczestnikaParams { id: citizenship_id },
    )
    .expect("delete the citizenship");
    assert_eq!(version_of(&room, "osoba"), 6);
}

// Properties --------------------------------------------------------------------------------------

/// The parameters of a flat in Wrocław with its land register and no address in the town.
fn flat_fields(flat_id: &str) -> WpiszNieruchomoscParams {
    WpiszNieruchomoscParams {
        id: flat_id.to_string(),
        rodzaj: RodzajNieruchomosci::LokalMieszkalny,
        prawo: PrawoDoNieruchomosci::OdrebnaWlasnoscLokalu,
        miejscowosc: "Wrocław".to_string(),
        gmina: "Wrocław".to_string(),
        ulica: None,
        numer_budynku: None,
        numer_lokalu: None,
        kod_pocztowy: None,
        numer_kw: Some("WR1K/00012345/6".to_string()),
        kw_brak: false,
        powierzchnia_uzytkowa: None,
        liczba_pokoi: None,
        pietro: None,
        nadrzedna_id: None,
    }
}

/// The change of the data of the property of `params` to the values in it; its kind stays.
fn property_change_of(params: WpiszNieruchomoscParams) -> ZmienDaneNieruchomosciParams {
    ZmienDaneNieruchomosciParams {
        prawo: params.prawo,
        miejscowosc: params.miejscowosc,
        gmina: params.gmina,
        ulica: params.ulica,
        numer_budynku: params.numer_budynku,
        numer_lokalu: params.numer_lokalu,
        kod_pocztowy: params.kod_pocztowy,
        numer_kw: params.numer_kw,
        kw_brak: params.kw_brak,
        powierzchnia_uzytkowa: params.powierzchnia_uzytkowa,
        liczba_pokoi: params.liczba_pokoi,
        pietro: params.pietro,
        nadrzedna_id: params.nadrzedna_id,
        id: params.id,
    }
}

/// Nieruchomość: numer księgi ma format XXXX/NNNNNNNN/N albo nieruchomość nie ma księgi
#[test]
fn property_land_register() {
    let room = new_room();

    let wrong_register_numbers = [
        WpiszNieruchomoscParams {
            numer_kw: Some("WR1K/123/6".to_string()),
            ..flat_fields("lokal")
        },
        WpiszNieruchomoscParams {
            numer_kw: None,
            ..flat_fields("lokal")
        },
        WpiszNieruchomoscParams {
            kw_brak: true,
            ..flat_fields("lokal")
        },
    ];
    for wrong_register_number in &wrong_register_numbers {
        expect::refused(
            sprawa_nieruchomosci::wpisz_nieruchomosc(&room, wrong_register_number),
            "numer_kw_albo_brak_ksiegi",
        );
    }

    add_flat(&room, &flat("lokal"));
    add_flat(
        &room,
        &WpiszNieruchomoscParams {
            prawo: PrawoDoNieruchomosci::SpoldzielczeWlasnosciowePrawoDoLokalu,
            numer_kw: None,
            kw_brak: true,
            ..flat("spoldzielcze")
        },
    );
    let register_dropped = WpiszNieruchomoscParams {
        kw_brak: false,
        numer_kw: None,
        ..flat_fields("spoldzielcze")
    };
    expect::refused(
        sprawa_nieruchomosci::zmien_dane_nieruchomosci(
            &room,
            &property_change_of(register_dropped),
        ),
        "numer_kw_albo_brak_ksiegi",
    );
}

/// Nieruchomość: pomieszczenie przynależne wskazuje lokal; działkę wolno wykreślić (D14)
#[test]
fn property_appurtenant_room() {
    let room = new_room();
    add_flat(&room, &flat("lokal"));
    add_flat(
        &room,
        &WpiszNieruchomoscParams {
            rodzaj: RodzajNieruchomosci::Dom,
            prawo: PrawoDoNieruchomosci::Wlasnosc,
            numer_kw: Some("WR1K/00054321/0".to_string()),
            ..flat("dom")
        },
    );

    let appurtenant_room =
        |flat_id: &str, rodzaj: RodzajNieruchomosci, nadrzedna_id: Option<&str>| {
            WpiszNieruchomoscParams {
                rodzaj,
                nadrzedna_id: nadrzedna_id.map(str::to_string),
                ..flat_fields(flat_id)
            }
        };

    expect::refused(
        sprawa_nieruchomosci::wpisz_nieruchomosc(
            &room,
            &appurtenant_room(
                "piwnica",
                RodzajNieruchomosci::PomieszczeniePrzynalezne,
                None,
            ),
        ),
        "pomieszczenie_przynalezne_wskazuje_lokal",
    );
    expect::refused(
        sprawa_nieruchomosci::wpisz_nieruchomosc(
            &room,
            &appurtenant_room("garaz", RodzajNieruchomosci::Garaz, Some("lokal")),
        ),
        "pomieszczenie_przynalezne_wskazuje_lokal",
    );
    let cellar_of_house = appurtenant_room(
        "piwnica",
        RodzajNieruchomosci::PomieszczeniePrzynalezne,
        Some("dom"),
    );
    expect::refused(
        sprawa_nieruchomosci::wpisz_nieruchomosc(&room, &cellar_of_house),
        "pomieszczenie przynależne wskazuje lokal",
    );
    sprawa_nieruchomosci::wpisz_nieruchomosc(
        &room,
        &appurtenant_room(
            "piwnica",
            RodzajNieruchomosci::PomieszczeniePrzynalezne,
            Some("lokal"),
        ),
    )
    .expect("add the cellar of the flat");
    expect::refused(
        sprawa_nieruchomosci::zmien_dane_nieruchomosci(&room, &property_change_of(cellar_of_house)),
        "pomieszczenie przynależne wskazuje lokal",
    );

    // The workaround from the audit: the flat of an appurtenant room turns into a house.
    expect::refused(
        room.execute(
            "UPDATE sprawa_nieruchomosci_wpisy SET rodzaj = 'dom' WHERE id = 'lokal'",
            &[],
        ),
        "rodzaj nieruchomości się nie zmienia",
    );
    let new_flat_number = WpiszNieruchomoscParams {
        numer_lokalu: Some("12A".to_string()),
        ..flat_fields("lokal")
    };
    sprawa_nieruchomosci::zmien_dane_nieruchomosci(&room, &property_change_of(new_flat_number))
        .expect("change the number of the flat");

    sprawa_nieruchomosci::wpisz_dzialke(
        &room,
        &WpiszDzialkeParams {
            nieruchomosc_id: "dom".to_string(),
            obreb: "0012".to_string(),
            numer_dzialki: "45/7".to_string(),
            powierzchnia: None,
        },
    )
    .expect("add the plot");
    let plot_id = integer_value(&room, "SELECT id FROM sprawa_nieruchomosci_dzialki");
    sprawa_nieruchomosci::zmien_dzialke(
        &room,
        &ZmienDzialkeParams {
            obreb: "0012".to_string(),
            numer_dzialki: "45/8".to_string(),
            powierzchnia: Some(812),
            id: plot_id,
        },
    )
    .expect("change the plot");
    sprawa_nieruchomosci::wykresl_dzialke(&room, &WykreslDzialkeParams { id: plot_id })
        .expect("delete the plot");

    let plots = sprawa_nieruchomosci::dzialki(&room).expect("read the plots");
    assert!(plots.is_empty());
}

// Documents ---------------------------------------------------------------------------------------

/// Dokument: powstaje wymagany, zamówiony albo otrzymany; otrzymany ma plik ze skrótem
#[test]
fn document_states_and_file() {
    let room = new_room();

    expect::refused(
        room.execute(
            "INSERT INTO sprawa_dokumenty_wpisy (id, rodzaj, nazwa, stan, sciezka, skrot)
            VALUES ('od-razu', 'inny', 'Dokument', 'sprawdzony', 'd.pdf', 'sha256-d')",
            &[],
        ),
        "dokument powstaje jako wymagany, zamówiony albo otrzymany",
    );

    sprawa_dokumenty::wpisz_wymagany_dokument(
        &room,
        &WpiszWymaganyDokumentParams {
            id: "swiadectwo".to_string(),
            rodzaj: RodzajDokumentu::SwiadectwoEnergetyczne,
            nazwa: "Świadectwo charakterystyki energetycznej".to_string(),
            dotyczy_uczestnik_id: None,
            dotyczy_nieruchomosc_id: None,
            wymaga: Some(WymagaDokumentu::Przepis),
        },
    )
    .expect("require the certificate");

    // The statement of `odnotuj_otrzymanie_dokumentu`; the values left out bind NULL.
    let receipt = "UPDATE sprawa_dokumenty_wpisy
        SET stan = 'otrzymany',
          wystawca = :wystawca,
          data_wystawienia = :data_wystawienia,
          waznosc_do = :waznosc_do,
          waznosc_zrodlo = :waznosc_zrodlo,
          pochodzenie = :pochodzenie,
          kanal = :kanal,
          postac = :postac,
          sciezka = :sciezka,
          skrot = :skrot
        WHERE id = :id";

    // A writer outside the client can leave out the file that `odnotuj_otrzymanie_dokumentu`
    // requires. The table refuses the receipt without one.
    let receipt_without_file = room.execute(
        receipt,
        &[
            ("sciezka", SqlValue::Null),
            ("skrot", SqlValue::Null),
            ("id", text("swiadectwo")),
        ],
    );
    expect::refused(receipt_without_file, "otrzymany_ma_plik");
    // A writer outside the client can leave out the hash that `odnotuj_otrzymanie_dokumentu`
    // requires. The table refuses a file without one.
    let receipt_without_hash = room.execute(
        receipt,
        &[
            ("sciezka", text("dokumenty/swiadectwo.pdf")),
            ("skrot", SqlValue::Null),
            ("id", text("swiadectwo")),
        ],
    );
    expect::refused(receipt_without_hash, "plik_ze_skrotem");
    sprawa_dokumenty::odnotuj_otrzymanie_dokumentu(
        &room,
        &OdnotujOtrzymanieDokumentuParams {
            wystawca: Some("inż. Jan Audytor".to_string()),
            data_wystawienia: Some("2020-03-01".to_string()),
            waznosc_do: Some("2030-03-01".to_string()),
            waznosc_zrodlo: Some(WaznoscZrodlo::Przepis),
            pochodzenie: None,
            kanal: None,
            postac: None,
            sciezka: "dokumenty/swiadectwo.pdf".to_string(),
            skrot: "sha256-swiadectwo".to_string(),
            id: "swiadectwo".to_string(),
        },
    )
    .expect("record the receipt of the certificate");
    sprawa_dokumenty::oznacz_dokument_jako_sprawdzony(
        &room,
        &OznaczDokumentJakoSprawdzonyParams {
            id: "swiadectwo".to_string(),
        },
    )
    .expect("mark the certificate as checked");
}

/// Dokument: odrzucony ma powód, ważność ma źródło i nie jest wcześniejsza niż wystawienie
#[test]
fn document_rejection_and_validity() {
    let room = new_room();
    add_received_document(
        &room,
        &received_document("zaswiadczenie", RodzajDokumentu::Zaswiadczenie),
    );

    expect::refused(
        room.execute(
            "UPDATE sprawa_dokumenty_wpisy SET stan = 'odrzucony' WHERE id = 'zaswiadczenie'",
            &[],
        ),
        "odrzucony_ma_powod",
    );
    sprawa_dokumenty::odrzuc_dokument(
        &room,
        &OdrzucDokumentParams {
            powod_odrzucenia: "nieczytelna pieczęć".to_string(),
            id: "zaswiadczenie".to_string(),
        },
    )
    .expect("reject the certificate");

    // The certificates of the cooperative written directly, without the defaults of a received
    // document.
    let certificate = |document_id: &str| WpiszOtrzymanyDokumentParams {
        id: document_id.to_string(),
        rodzaj: RodzajDokumentu::Zaswiadczenie,
        nazwa: "Zaświadczenie spółdzielni".to_string(),
        dotyczy_uczestnik_id: None,
        dotyczy_nieruchomosc_id: None,
        wystawca: None,
        data_wystawienia: None,
        waznosc_do: None,
        waznosc_zrodlo: None,
        wymaga: None,
        pochodzenie: None,
        kanal: None,
        postac: None,
        sciezka: "dokumenty/z.pdf".to_string(),
        skrot: "sha256-z".to_string(),
    };
    let expired = WpiszOtrzymanyDokumentParams {
        data_wystawienia: Some("2026-11-03".to_string()),
        waznosc_do: Some("2026-11-01".to_string()),
        waznosc_zrodlo: Some(WaznoscZrodlo::Instytucja),
        ..certificate("przeterminowane")
    };
    expect::refused(
        sprawa_dokumenty::wpisz_otrzymany_dokument(&room, &expired),
        "waznosc_nie_przed_wystawieniem",
    );
    let validity_without_source = WpiszOtrzymanyDokumentParams {
        waznosc_do: Some("2026-12-03".to_string()),
        ..certificate("bez-zrodla")
    };
    expect::refused(
        sprawa_dokumenty::wpisz_otrzymany_dokument(&room, &validity_without_source),
        "waznosc_ze_zrodlem",
    );
    add_received_document(
        &room,
        &WpiszOtrzymanyDokumentParams {
            data_wystawienia: Some("2026-11-03".to_string()),
            waznosc_do: Some("2026-12-03".to_string()),
            waznosc_zrodlo: Some(WaznoscZrodlo::Instytucja),
            ..received_document("wazne", RodzajDokumentu::Zaswiadczenie)
        },
    );
}

/// Dokument: plik się nie zmienia, także skan dowodu (D16)
#[test]
fn document_file_immutable() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    add_received_document(
        &room,
        &WpiszOtrzymanyDokumentParams {
            dotyczy_uczestnik_id: Some("klientka".to_string()),
            ..received_document("skan-dowodu", RodzajDokumentu::Tozsamosc)
        },
    );

    let other_files = [
        (Some("dokumenty/inny.pdf"), Some("sha256-skan-dowodu")),
        (Some("dokumenty/skan-dowodu.pdf"), Some("sha256-inny")),
        (None, None),
    ];
    for (sciezka, skrot) in other_files {
        expect::refused(
            room.execute(
                "UPDATE sprawa_dokumenty_wpisy SET sciezka = :sciezka, skrot = :skrot",
                &[
                    ("sciezka", sciezka.to_sql_value()),
                    ("skrot", skrot.to_sql_value()),
                ],
            ),
            "plik dokumentu się nie zmienia",
        );
    }

    // The first file of a required document is not a change.
    sprawa_dokumenty::wpisz_wymagany_dokument(
        &room,
        &WpiszWymaganyDokumentParams {
            id: "odpis".to_string(),
            rodzaj: RodzajDokumentu::OdpisRejestru,
            nazwa: "Odpis z KRS".to_string(),
            dotyczy_uczestnik_id: None,
            dotyczy_nieruchomosc_id: None,
            wymaga: None,
        },
    )
    .expect("require the extract");
    sprawa_dokumenty::odnotuj_otrzymanie_dokumentu(
        &room,
        &OdnotujOtrzymanieDokumentuParams {
            wystawca: None,
            data_wystawienia: None,
            waznosc_do: None,
            waznosc_zrodlo: None,
            pochodzenie: None,
            kanal: None,
            postac: None,
            sciezka: "dokumenty/odpis.pdf".to_string(),
            skrot: "sha256-o".to_string(),
            id: "odpis".to_string(),
        },
    )
    .expect("record the receipt of the extract");
}

/// Dokument: regulamin, standard, instrukcja i uchwała mają własne rodzaje
#[test]
fn document_kinds_of_a_flat() {
    let room = new_room();

    let kinds = [
        ("regulamin", RodzajDokumentu::Regulamin),
        ("standard", RodzajDokumentu::Standard),
        ("instrukcja", RodzajDokumentu::Instrukcja),
        ("uchwala", RodzajDokumentu::Uchwala),
    ];
    for (document_id, rodzaj) in kinds {
        add_received_document(&room, &received_document(document_id, rodzaj));
    }

    // A writer outside the client can give the kind „uchwala_wspolnoty”, which
    // `sprawa_dokumenty::WpisyRodzaj` leaves out. The table refuses it.
    let unknown_kind = room.execute(
        "INSERT INTO sprawa_dokumenty_wpisy (id, rodzaj, nazwa, dotyczy_uczestnik_id,
          dotyczy_nieruchomosc_id, wymaga, stan)
        VALUES (:id, :rodzaj, :nazwa, :dotyczy_uczestnik_id, :dotyczy_nieruchomosc_id, :wymaga,
          'wymagany')",
        &[
            ("id", text("nieznany")),
            ("rodzaj", text("uchwala_wspolnoty")),
            ("nazwa", text("Uchwała wspólnoty")),
        ],
    );
    expect::refused(unknown_kind, "rodzaj IN");
}

/// Dokument: zbędny staje się tylko wymagany albo zamówiony
#[test]
fn document_unneeded() {
    let room = new_room();
    let required = |document_id: &str, nazwa: &str| WpiszWymaganyDokumentParams {
        id: document_id.to_string(),
        rodzaj: RodzajDokumentu::Zaswiadczenie,
        nazwa: nazwa.to_string(),
        dotyczy_uczestnik_id: None,
        dotyczy_nieruchomosc_id: None,
        wymaga: None,
    };
    sprawa_dokumenty::wpisz_wymagany_dokument(&room, &required("wymagany", "Zaświadczenie"))
        .expect("require a document");
    sprawa_dokumenty::wpisz_wymagany_dokument(&room, &required("zamowiony", "Odpis"))
        .expect("require a document");
    sprawa_dokumenty::odnotuj_zamowienie_dokumentu(
        &room,
        &OdnotujZamowienieDokumentuParams {
            id: "zamowiony".to_string(),
        },
    )
    .expect("record the order");
    add_received_document(
        &room,
        &received_document("otrzymany", RodzajDokumentu::Inny),
    );

    let unneeded = |document_id: &str| UznajDokumentZaZbednyParams {
        id: document_id.to_string(),
    };
    for document_id in ["wymagany", "zamowiony"] {
        sprawa_dokumenty::uznaj_dokument_za_zbedny(&room, &unneeded(document_id))
            .expect("find the document unneeded");
    }
    expect::refused(
        sprawa_dokumenty::uznaj_dokument_za_zbedny(&room, &unneeded("otrzymany")),
        "zbędny staje się tylko dokument wymagany albo zamówiony",
    );
}

/// Dokument dotyczy uczestnika albo nieruchomości; źródło wskazuje dokument albo osobę
#[test]
fn document_subject_and_source() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    add_flat(&room, &flat("lokal"));

    let about_person_and_property = WpiszWymaganyDokumentParams {
        id: "dokument".to_string(),
        rodzaj: RodzajDokumentu::Inny,
        nazwa: "Dokument".to_string(),
        dotyczy_uczestnik_id: Some("klientka".to_string()),
        dotyczy_nieruchomosc_id: Some("lokal".to_string()),
        wymaga: None,
    };
    expect::refused(
        sprawa_dokumenty::wpisz_wymagany_dokument(&room, &about_person_and_property),
        "dotyczy_uczestnika_albo_nieruchomosci",
    );
    add_received_document(
        &room,
        &WpiszOtrzymanyDokumentParams {
            dotyczy_uczestnik_id: Some("klientka".to_string()),
            ..received_document("akt-malzenstwa", RodzajDokumentu::AktStanuCywilnego)
        },
    );

    let source = WpiszZrodloParams {
        dotyczy_tabela: "sprawa_uczestnicy_wpisy".to_string(),
        dotyczy_id: "klientka".to_string(),
        pole: "stan_cywilny".to_string(),
        dokument_id: None,
        od_uczestnik_id: None,
        od_kogo: None,
        potwierdzone: false,
        wpisano: "2026-11-03T09:00:00Z".to_string(),
    };
    expect::refused(
        sprawa_dokumenty::wpisz_zrodlo(&room, &source),
        "zrodlo_wskazuje_dokument_albo_osobe",
    );
    let document_and_person = WpiszZrodloParams {
        dokument_id: Some("akt-malzenstwa".to_string()),
        od_uczestnik_id: Some("klientka".to_string()),
        ..source.clone()
    };
    expect::refused(
        sprawa_dokumenty::wpisz_zrodlo(&room, &document_and_person),
        "zrodlo_wskazuje_dokument_albo_osobe",
    );

    let allowed_sources = [
        WpiszZrodloParams {
            od_uczestnik_id: Some("klientka".to_string()),
            ..source.clone()
        },
        WpiszZrodloParams {
            dokument_id: Some("akt-malzenstwa".to_string()),
            ..source.clone()
        },
        WpiszZrodloParams {
            pole: "telefon".to_string(),
            od_kogo: Some("zarządca budynku".to_string()),
            ..source
        },
    ];
    for allowed_source in &allowed_sources {
        sprawa_dokumenty::wpisz_zrodlo(&room, allowed_source).expect("add a source");
    }

    let oral_source_id = integer_value(
        &room,
        "SELECT id FROM sprawa_dokumenty_zrodla WHERE od_uczestnik_id IS NOT NULL",
    );
    sprawa_dokumenty::potwierdz_zrodlo(&room, &PotwierdzZrodloParams { id: oral_source_id })
        .expect("confirm the oral source");
    let sources_of_client = sprawa_dokumenty::zrodla_wiersza(
        &room,
        &ZrodlaWierszaParams {
            dotyczy_tabela: "sprawa_uczestnicy_wpisy".to_string(),
            dotyczy_id: "klientka".to_string(),
        },
    )
    .expect("read the sources of the client");
    assert_eq!(sources_of_client.len(), 3);
}
