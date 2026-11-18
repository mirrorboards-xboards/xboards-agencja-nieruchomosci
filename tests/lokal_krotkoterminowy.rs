//! Scenarios of the short-term rental case (spec/cases/lokal_krotkoterminowy).
//!
//! A case room is the core of every case plus the kinds of the type. Each scenario starts from an
//! empty room and checks one rule the database keeps: a write the rule allows and a write it
//! refuses. The first scenario follows a whole case as the profile describes it: the management
//! contract, the flat, the offer, a guest's stay with its access code, the orders for contractors,
//! the monthly statement, the payout to the owner and the end of management.
//!
//! The scenarios call the operations of the core kinds through `cases::sprawa`, like the building
//! blocks of `crate::sprawa` they share, so one set of core types serves both.

use xboards_agencja_nieruchomosci::cases::lokal_krotkoterminowy::{
    self, lokal_krotkoterminowy_kalendarz, lokal_krotkoterminowy_lokal,
    lokal_krotkoterminowy_pobyty, lokal_krotkoterminowy_umowa, lokal_krotkoterminowy_zestawienia,
    lokal_krotkoterminowy_zlecenia,
};
use xboards_agencja_nieruchomosci::cases::sprawa::{
    sprawa_decyzje, sprawa_dokumenty, sprawa_korespondencja, sprawa_metryka, sprawa_protokoly,
    sprawa_rachunki, sprawa_sprawdzenia, sprawa_uczestnicy, sprawa_umowy, sprawa_wyplaty,
    sprawa_zgody,
};
use xboards_agencja_nieruchomosci::xmeta_room::{FromSqlValue, MemoryRoom, SqlValue};

use lokal_krotkoterminowy_kalendarz::{
    KanalyKonto as KontoOferty, NoceStan as StanNocy, OpublikujOferteWKanaleParams,
    PrzygotujOferteWKanaleParams, Stan as StanOferty, UstawCeneNocyParams, WpiszNocParams,
    WpiszWyliczenieCenParams, WskazRachunekOfertyWKanaleParams, ZmienNocParams,
    ZmienStanOfertyWKanaleParams,
};
use lokal_krotkoterminowy_lokal::{
    WpiszDaneLokaluParams, WpiszPrzegladLokaluParams, WpiszRegulaminLokaluParams,
    WpiszUchwaleWspolnotyParams, WpiszWyposazenieLokaluParams, WycofajWyposazenieLokaluParams,
    ZmienDaneLokaluParams,
};
use lokal_krotkoterminowy_pobyty::{
    OdnotujInstrukcjeWejsciaParams, OdnotujOplaceniePobytuParams,
    OdnotujPrzekazanieZgloszeniaParams, OdnotujRozliczeniePobytuParams,
    PieniadzeRodzaj as RodzajPieniedzy, PrzyjmijRezerwacjeParams, Stan as StanPobytu,
    WpiszOpinieGosciaParams, WpiszPieniadzePobytuParams, WpiszSkargeParams,
    WpiszWyjatekPobytuParams, WpiszZgloszenieGosciaParams, WpiszZmianePobytuParams,
    WskazOdpowiedzNaOpinieParams, WskazOdpowiedzNaSkargeParams, ZakonczPobytParams,
    ZmianyCo as CoSieZmienia, ZmienRezerwacjeParams, ZmienStanPobytuParams,
};
use lokal_krotkoterminowy_umowa::{
    DopiszKanalWarunkowParams, DopiszWidelkiWarunkowParams, KanalyKonto as KontoWarunkow,
    UstalWarunkiUmowyLokaluParams, WarunkiProwizjaRodzaj as RodzajProwizji,
    WarunkiStan as StanWarunkow, WpiszWarunkiUmowyLokaluParams,
};
use lokal_krotkoterminowy_zestawienia::{
    CofnijZestawienieDoProjektuParams, DopiszPozycjeZestawieniaParams,
    OdnotujWyslanieZestawieniaParams, PozycjeRodzaj as RodzajPozycji, PrzygotujZestawienieParams,
    WpisyStan as StanZestawienia, WskazWysylkeZestawieniaParams, ZamknijZestawienieParams,
};
use lokal_krotkoterminowy_zlecenia::{
    DostepyRodzaj as RodzajDostepu, OdnotujOdebranieDostepuParams, OdnotujWydanieDostepuParams,
    PotwierdzWykonanieZleceniaParams, Stan as StanZlecenia, UstawDostepParams,
    WpisyCenaZrodlo as ZrodloCeny, WpisyPrzyczyna as PrzyczynaZlecenia,
    WpisyRodzaj as RodzajZlecenia, WpiszKosztZleceniaParams, ZlecPraceParams,
    ZmienStanZleceniaParams,
};
use sprawa_decyzje::{PoprosODecyzjeParams, WpisyRodzaj as RodzajDecyzji};
use sprawa_dokumenty::{WpisyRodzaj as RodzajDokumentu, WpiszOtrzymanyDokumentParams};
use sprawa_korespondencja::{
    OdnotujWyslanieParams, PrzekazWysylkeDoZgodyParams, PrzygotujWysylkeParams,
    WysylkiAdresatRodzaj as RodzajAdresata, WysylkiKanal as KanalWysylki,
};
use sprawa_metryka::{ZalozMetrykeParams, ZmienEtapSprawyParams};
use sprawa_protokoly::{
    DopiszPozycjeProtokoluParams, PodpiszProtokolParams, PozycjeRodzaj as RodzajPozycjiProtokolu,
    PrzygotujProtokolParams, WpisyRodzaj as RodzajProtokolu,
};
use sprawa_rachunki::{WpisyCel as CelRachunku, WpiszRachunekParams};
use sprawa_sprawdzenia::{
    WpisyRodzaj as RodzajSprawdzenia, WpisyWynik as WynikSprawdzenia, WpiszSprawdzenieParams,
};
use sprawa_uczestnicy::{
    NadajRoleUczestnikowiParams, WpisyDaneOd as DaneOd, WpisyKanalKontaktu as KanalKontaktu,
    WpiszUczestnikaParams,
};
use sprawa_umowy::{
    OdnotujAneksUmowyParams, OdnotujZawarcieUmowyParams, PrzekazUmoweDoPodpisuParams,
    PrzygotujUmoweParams, WpisyForma as FormaUmowy, WpisyRodzaj as RodzajUmowy,
    WpisySposobZawarcia as SposobZawarcia, WpiszZdarzenieUmowyParams, ZakonczUmoweParams,
    ZdarzeniaRodzaj as RodzajZdarzeniaUmowy,
};
use sprawa_wyplaty::{
    OdnotujWykonanieWyplatyParams, PrzygotujWyplateParams, WpisyStan as StanWyplaty,
    WskazSprawdzenieWyplatyParams, WskazZgodeNaWyplateParams,
};
use sprawa_zgody::WpiszZgodeParams;

use crate::expect;
use crate::sprawa::{
    WITHDRAWAL_ENDS, account, add_account, add_consent, add_flat, add_person,
    add_received_document, approved_decision, check, check_identity, check_of, conclude_contract,
    confirm_account, consent, contract, decision_request, decision_request_about, flat, letter,
    open_case, person, prepare_contract, prepare_letter, prepare_payout, received_document, send,
    sending,
};
use crate::sprawa_porzadek::{first_row_copy, first_row_copy_without_id};

/// The room type of a short-term rental case.
const TYPE: &str = "lokal_krotkoterminowy";

/// The moment the management contract is concluded ([`conclude_contract`] writes it).
const CONCLUDED: &str = "2026-11-04T12:00:00Z";

/// The end date of the management contract unless a scenario says otherwise.
const CONTRACT_ENDS: &str = "2027-01-31";

/// The moment the human signs a decision unless a scenario says otherwise.
const APPROVED: &str = "2026-11-03T12:00:00Z";

/// A new room of a short-term rental case with no record yet.
fn empty_room() -> MemoryRoom {
    lokal_krotkoterminowy::open_in_memory().expect("open the room of a short-term rental case")
}

/// A short-term rental case room right after intake.
fn new_room() -> MemoryRoom {
    let room = empty_room();
    open_case(&room, TYPE);

    room
}

/// A text value to bind in raw SQL.
fn text(value: &str) -> SqlValue {
    SqlValue::Text(value.to_string())
}

/// The single integer a read in raw SQL returns, e.g. a count.
fn integer_value(room: &MemoryRoom, sql: &str) -> i64 {
    let value = room.value(sql, &[]).expect("read the value");

    i64::from_sql_value(&value).expect("the value is an integer")
}

// Building blocks of a short-term rental case -----------------------------------------------------

/// The parameters of a change of the stage of the case by the agent, with no decision.
fn stage_change(etap: &str, od: &str) -> ZmienEtapSprawyParams {
    ZmienEtapSprawyParams {
        etap: etap.to_string(),
        od: od.to_string(),
        zmienil: "agent".to_string(),
        decyzja_id: None,
    }
}

/// Moves the case to the stage `etap` from `od`, as the manager's agent.
fn stage(room: &MemoryRoom, etap: &str, od: &str) {
    let params = ZmienEtapSprawyParams {
        zmienil: "agent-zarzadcy".to_string(),
        ..stage_change(etap, od)
    };

    sprawa_metryka::zmien_etap_sprawy(room, &params).expect("change the stage of the case");
}

/// [`stage`] by the decision `decyzja_id` that interrupts or resumes the case.
fn stage_by_decision(room: &MemoryRoom, etap: &str, od: &str, decyzja_id: &str) {
    let params = ZmienEtapSprawyParams {
        zmienil: "agent-zarzadcy".to_string(),
        decyzja_id: Some(decyzja_id.to_string()),
        ..stage_change(etap, od)
    };

    sprawa_metryka::zmien_etap_sprawy(room, &params).expect("change the stage of the case");
}

/// Writes the request for a decision `rodzaj` about the row `dotyczy_id` of `dotyczy_tabela`,
/// filed and signed at the defaults of [`decision_request`] and [`APPROVED`].
fn approved_decision_about(
    room: &MemoryRoom,
    decision_id: &str,
    rodzaj: RodzajDecyzji,
    dotyczy_tabela: &str,
    dotyczy_id: &str,
) {
    let request = decision_request_about(decision_id, rodzaj, dotyczy_tabela, dotyczy_id);

    approved_decision(room, &request, APPROVED);
}

/// Gives the participant `uczestnik_id` the role `rola`.
fn give_role(room: &MemoryRoom, uczestnik_id: &str, rola: &str) {
    let params = NadajRoleUczestnikowiParams {
        uczestnik_id: uczestnik_id.to_string(),
        rola: rola.to_string(),
        dotyczy_uczestnik_id: None,
    };

    sprawa_uczestnicy::nadaj_role_uczestnikowi(room, &params).expect("give the role");
}

/// The default parameters of the owner of the flat: Anna Kowalska (see [`person`]), a client of
/// the office contacted by e-mail.
fn owner() -> WpiszUczestnikaParams {
    WpiszUczestnikaParams {
        klient_biura: true,
        kanal_kontaktu: Some(KanalKontaktu::Email),
        ..person("wlascicielka")
    }
}

/// Writes the owner of `params` (see [`owner`]) with the role wlasciciel_lokalu.
fn add_owner(room: &MemoryRoom, params: &WpiszUczestnikaParams) {
    add_person(room, params);

    give_role(room, &params.id, "wlasciciel_lokalu");
}

/// Writes the guest Jan Nowak, whose data came from the sales channel, with the role gosc.
fn add_guest(room: &MemoryRoom) {
    let guest = WpiszUczestnikaParams {
        imiona: Some("Jan".to_string()),
        nazwisko: Some("Nowak".to_string()),
        kanal_kontaktu: Some(KanalKontaktu::KanalSprzedazy),
        dane_od: DaneOd::ZInnegoZrodla,
        dane_pozyskano: "2026-11-07T10:00:00Z".to_string(),
        ..person("gosc")
    };
    add_person(room, &guest);

    give_role(room, "gosc", "gosc");
}

/// Writes the received document `document_id` of the kind `rodzaj` named `nazwa` (see
/// [`received_document`]).
fn add_named_document(room: &MemoryRoom, document_id: &str, rodzaj: RodzajDokumentu, nazwa: &str) {
    let params = WpiszOtrzymanyDokumentParams {
        nazwa: nazwa.to_string(),
        ..received_document(document_id, rodzaj)
    };

    add_received_document(room, &params);
}

/// Hands the draft contract `contract_id` over for signing.
fn hand_over_for_signing(room: &MemoryRoom, contract_id: &str) {
    let params = PrzekazUmoweDoPodpisuParams {
        id: contract_id.to_string(),
    };

    sprawa_umowy::przekaz_umowe_do_podpisu(room, &params)
        .expect("hand the contract over for signing");
}

/// Writes Kontroler's check of the owner's identity and the management contract „umowa” about
/// the flat, ending on `koniec`, and hands it over for signing.
fn management_contract_for_signing(room: &MemoryRoom, koniec: Option<&str>) {
    check_identity(room, "tozsamosc-wlascicielki", "wlascicielka");

    let draft = PrzygotujUmoweParams {
        nieruchomosc_id: Some("lokal".to_string()),
        koniec: koniec.map(str::to_string),
        ..contract("umowa")
    };
    prepare_contract(room, &draft, &["wlascicielka"]);

    hand_over_for_signing(room, "umowa");
}

/// The owner checked by Kontroler signs the management contract, ending on `koniec`, in the
/// office.
fn sign_management_contract(room: &MemoryRoom, koniec: Option<&str>) {
    management_contract_for_signing(room, koniec);

    conclude_contract(room, "umowa", SposobZawarcia::WLokaluBiura, None);
}

/// The consumer owner checked by Kontroler signs the management contract, ending on
/// [`CONTRACT_ENDS`], at a distance; she may withdraw until [`WITHDRAWAL_ENDS`].
fn sign_management_contract_at_a_distance(room: &MemoryRoom) {
    management_contract_for_signing(room, Some(CONTRACT_ENDS));

    conclude_contract(
        room,
        "umowa",
        SposobZawarcia::NaOdleglosc,
        Some(WITHDRAWAL_ENDS),
    );
}

/// The parameters of the event `rodzaj` of the contract „umowa” at `chwila`, with no document.
fn contract_event(
    event_id: &str,
    rodzaj: RodzajZdarzeniaUmowy,
    chwila: &str,
) -> WpiszZdarzenieUmowyParams {
    WpiszZdarzenieUmowyParams {
        id: event_id.to_string(),
        umowa_id: "umowa".to_string(),
        rodzaj,
        chwila: chwila.to_string(),
        opis: None,
        dokument_id: None,
        wysylka_id: None,
        wplyw_id: None,
        zgoda_id: None,
    }
}

/// Writes the event of the contract of `params` (see [`contract_event`]).
fn add_contract_event(room: &MemoryRoom, params: &WpiszZdarzenieUmowyParams) {
    sprawa_umowy::wpisz_zdarzenie_umowy(room, params).expect("add the event of the contract");
}

/// Ends the contract `contract_id` for the reason `koniec_powod`.
fn end_contract(room: &MemoryRoom, contract_id: &str, koniec_powod: &str) {
    let params = ZakonczUmoweParams {
        koniec_powod: koniec_powod.to_string(),
        id: contract_id.to_string(),
    };

    sprawa_umowy::zakoncz_umowe(room, &params).expect("end the contract");
}

/// The consumer's explicit request to start before the withdrawal period ends (R8), made on
/// 2026-11-04 at 13:00.
fn ask_to_start_work(room: &MemoryRoom) {
    add_consent(
        room,
        &consent("zadanie", "wlascicielka", "zadanie_rozpoczecia_pracy"),
    );

    let request = WpiszZdarzenieUmowyParams {
        zgoda_id: Some("zadanie".to_string()),
        ..contract_event(
            "zdarzenie-zadanie",
            RodzajZdarzeniaUmowy::ZadanieRozpoczeciaPracy,
            "2026-11-04T13:00:00Z",
        )
    };
    add_contract_event(room, &request);
}

/// The default parameters of the terms `terms_id` of the contract „umowa” from its conclusion: up
/// to 4 guests, 15% of the revenue after the channel's fee, cleaning 150 zł that covers its cost,
/// repairs up to 500 zł, the owner's nights reported 14 days ahead, the payout on the 15th and a
/// reserve of 500 zł.
fn terms(terms_id: &str) -> WpiszWarunkiUmowyLokaluParams {
    WpiszWarunkiUmowyLokaluParams {
        id: terms_id.to_string(),
        umowa_id: "umowa".to_string(),
        max_gosci: 4,
        prowizja_rodzaj: RodzajProwizji::Procent,
        prowizja: 1500,
        prowizja_podstawa: "przychód po prowizji kanału".to_string(),
        oplata_sprzatanie: 15_000,
        oplata_sprzatanie_pokrywa_koszt: true,
        kwota_napraw: 50_000,
        koszty_wlasciciela: "Biuro płaci z przychodu opłaty dla wspólnoty i internet.".to_string(),
        zgloszenie_nocy_dni: 14,
        wyplata_dzien: 15,
        rezerwa: 50_000,
        po_koncu_rezerwacje: "Biuro odwołuje rezerwacje po końcu umowy za zgodą człowieka."
            .to_string(),
        aneks_zdarzenie_id: None,
        od: CONCLUDED.to_string(),
    }
}

/// [`terms`] „warunki” with the shorter descriptions of the owner's costs and of the bookings
/// after the end that the scenarios of refused terms give.
fn terms_fields() -> WpiszWarunkiUmowyLokaluParams {
    WpiszWarunkiUmowyLokaluParams {
        koszty_wlasciciela: "opłaty wspólnoty z przychodu".to_string(),
        po_koncu_rezerwacje: "odwołanie za zgodą człowieka".to_string(),
        ..terms("warunki")
    }
}

/// Writes the draft terms of `params` (see [`terms`]).
fn draft_terms(room: &MemoryRoom, params: &WpiszWarunkiUmowyLokaluParams) {
    lokal_krotkoterminowy_umowa::wpisz_warunki_umowy_lokalu(room, params)
        .expect("write the draft terms");
}

/// The parameters of the sales channel `kanal` of the terms `terms_id` on the account `konto`.
fn terms_channel(terms_id: &str, kanal: &str, konto: KontoWarunkow) -> DopiszKanalWarunkowParams {
    DopiszKanalWarunkowParams {
        warunki_id: terms_id.to_string(),
        kanal: kanal.to_string(),
        konto,
    }
}

/// Writes the channel of the terms of `params` (see [`terms_channel`]).
fn add_terms_channel(room: &MemoryRoom, params: &DopiszKanalWarunkowParams) {
    lokal_krotkoterminowy_umowa::dopisz_kanal_warunkow(room, params)
        .expect("add a channel of the terms");
}

/// The parameters of establishing the draft terms `terms_id`.
fn establishment(terms_id: &str) -> UstalWarunkiUmowyLokaluParams {
    UstalWarunkiUmowyLokaluParams {
        id: terms_id.to_string(),
    }
}

/// Terms of `params` (see [`terms`]) with Airbnb on the office's account, Booking on the owner's
/// and winter price limits, established.
fn established_terms(room: &MemoryRoom, params: &WpiszWarunkiUmowyLokaluParams) {
    draft_terms(room, params);

    add_terms_channel(
        room,
        &terms_channel(&params.id, "airbnb", KontoWarunkow::Biura),
    );
    add_terms_channel(
        room,
        &terms_channel(&params.id, "booking", KontoWarunkow::Wlasciciela),
    );

    let winter_limits = DopiszWidelkiWarunkowParams {
        warunki_id: params.id.clone(),
        sezon: "zima 2026/2027".to_string(),
        od_dnia: "2026-11-01".to_string(),
        do_dnia: "2027-03-31".to_string(),
        cena_min: 25_000,
        cena_max: 150_000,
        pobyt_min: 2,
    };
    lokal_krotkoterminowy_umowa::dopisz_widelki_warunkow(room, &winter_limits)
        .expect("add the price limits of the terms");

    lokal_krotkoterminowy_umowa::ustal_warunki_umowy_lokalu(room, &establishment(&params.id))
        .expect("establish the terms");
}

/// Writes the draft protocol `protocol_id` of the kind `rodzaj` about the flat at `chwila`.
fn prepare_protocol(room: &MemoryRoom, protocol_id: &str, rodzaj: RodzajProtokolu, chwila: &str) {
    let params = PrzygotujProtokolParams {
        id: protocol_id.to_string(),
        rodzaj,
        nieruchomosc_id: "lokal".to_string(),
        chwila: chwila.to_string(),
    };

    sprawa_protokoly::przygotuj_protokol(room, &params).expect("prepare the protocol");
}

/// Signs the protocol `protocol_id` with the received document `dokument_id`.
fn sign_protocol(room: &MemoryRoom, protocol_id: &str, dokument_id: &str) {
    let params = PodpiszProtokolParams {
        dokument_id: dokument_id.to_string(),
        id: protocol_id.to_string(),
    };

    sprawa_protokoly::podpisz_protokol(room, &params).expect("sign the protocol");
}

/// Writes the protocol `protocol_id` of the kind `rodzaj` with the two keys of the front door,
/// signed with the received document `podpisany-<id>`.
fn signed_protocol(room: &MemoryRoom, protocol_id: &str, rodzaj: RodzajProtokolu, chwila: &str) {
    prepare_protocol(room, protocol_id, rodzaj, chwila);

    let keys = DopiszPozycjeProtokoluParams {
        protokol_id: protocol_id.to_string(),
        rodzaj: RodzajPozycjiProtokolu::Klucze,
        nazwa: "klucze do drzwi wejściowych".to_string(),
        numer: None,
        opis_stanu: Some("2 sztuki".to_string()),
        zdjecie_sciezka: None,
        zdjecie_skrot: None,
    };
    sprawa_protokoly::dopisz_pozycje_protokolu(room, &keys).expect("add the keys to the protocol");

    let signed_id = format!("podpisany-{protocol_id}");
    add_received_document(
        room,
        &received_document(&signed_id, RodzajDokumentu::Protokol),
    );
    sign_protocol(room, protocol_id, &signed_id);
}

/// The default parameters of the offer `offer_id` in the channel `kanal` on the account `konto`,
/// with no account for payouts and no consent.
fn offer(offer_id: &str, kanal: &str, konto: KontoOferty) -> PrzygotujOferteWKanaleParams {
    PrzygotujOferteWKanaleParams {
        id: offer_id.to_string(),
        kanal: kanal.to_string(),
        konto,
        nazwa_konta: format!("Biuro – {kanal}"),
        rachunek_id: None,
        zgoda_id: None,
    }
}

/// The default parameters of the offer „oferta-airbnb” on the office's Airbnb account.
fn airbnb_offer() -> PrzygotujOferteWKanaleParams {
    offer("oferta-airbnb", "airbnb", KontoOferty::Biura)
}

/// Writes the draft offer of `params` (see [`offer`]).
fn draft_offer(room: &MemoryRoom, params: &PrzygotujOferteWKanaleParams) {
    lokal_krotkoterminowy_kalendarz::przygotuj_oferte_w_kanale(room, params)
        .expect("prepare the offer");
}

/// The default parameters of publishing the offer `offer_id` as number `nr-<id>` on 2026-11-06
/// at 10:00, with no decision about a new channel.
fn publication(offer_id: &str) -> OpublikujOferteWKanaleParams {
    OpublikujOferteWKanaleParams {
        numer_oferty: format!("nr-{offer_id}"),
        opublikowano: "2026-11-06T10:00:00Z".to_string(),
        nowy_kanal_decyzja_id: None,
        id: offer_id.to_string(),
    }
}

/// Publishes the offer of `params` (see [`publication`]).
fn publish(room: &MemoryRoom, params: &OpublikujOferteWKanaleParams) {
    lokal_krotkoterminowy_kalendarz::opublikuj_oferte_w_kanale(room, params)
        .expect("publish the offer");
}

/// The parameters of a change of the state of the offer `offer_id` to `stan`.
fn offer_state(offer_id: &str, stan: StanOferty) -> ZmienStanOfertyWKanaleParams {
    ZmienStanOfertyWKanaleParams {
        stan,
        id: offer_id.to_string(),
    }
}

/// Changes the state of the offer `offer_id` to `stan`.
fn set_offer_state(room: &MemoryRoom, offer_id: &str, stan: StanOferty) {
    lokal_krotkoterminowy_kalendarz::zmien_stan_oferty_w_kanale(room, &offer_state(offer_id, stan))
        .expect("change the state of the offer");
}

/// An owner, her flat, a concluded contract with established terms and a published offer.
fn managed_flat(room: &MemoryRoom) {
    add_owner(room, &owner());
    add_flat(room, &flat("lokal"));
    sign_management_contract(room, Some(CONTRACT_ENDS));
    established_terms(room, &terms("warunki"));
    draft_offer(room, &airbnb_offer());
    publish(room, &publication("oferta-airbnb"));
}

/// The default parameters of the guest's booking `stay_id` on Airbnb, number `HM-<id>`, taken on
/// 2026-11-07 at 10:00: two guests paying 3000 zł with a cleaning fee of 150 zł, the channel
/// taking 450 zł, not yet paid.
fn booking(stay_id: &str, przyjazd: &str, wyjazd: &str) -> PrzyjmijRezerwacjeParams {
    PrzyjmijRezerwacjeParams {
        id: stay_id.to_string(),
        kanal: "airbnb".to_string(),
        numer_rezerwacji: format!("HM-{stay_id}"),
        przyjeto: "2026-11-07T10:00:00Z".to_string(),
        przyjazd: przyjazd.to_string(),
        wyjazd: wyjazd.to_string(),
        gosci: 2,
        gosc_uczestnik_id: "gosc".to_string(),
        kwota_goscia: 300_000,
        oplata_sprzatanie: 15_000,
        prowizja_kanalu: Some(45_000),
        oplacona: false,
        po_koncu_decyzja_id: None,
    }
}

/// [`booking`] „pobyt” from 10 to 13 December without the channel's fee, the booking the refusals
/// of the scenarios start from.
fn booking_fields() -> PrzyjmijRezerwacjeParams {
    PrzyjmijRezerwacjeParams {
        prowizja_kanalu: None,
        ..booking("pobyt", "2026-12-10", "2026-12-13")
    }
}

/// Takes the booking of `params` (see [`booking`]).
fn book(room: &MemoryRoom, params: &PrzyjmijRezerwacjeParams) {
    lokal_krotkoterminowy_pobyty::przyjmij_rezerwacje(room, params).expect("take the booking");
}

/// A managed flat with a guest's stay from 10 to 13 December.
fn booked_flat(room: &MemoryRoom) {
    managed_flat(room);
    add_guest(room);
    book(room, &booking("pobyt", "2026-12-10", "2026-12-13"));
}

/// The default parameters of a change of the booking `stay_id` to the dates `przyjazd` and
/// `wyjazd`, for two guests paying 3000 zł with a cleaning fee of 150 zł.
fn rebooking(stay_id: &str, przyjazd: &str, wyjazd: &str) -> ZmienRezerwacjeParams {
    ZmienRezerwacjeParams {
        przyjazd: przyjazd.to_string(),
        wyjazd: wyjazd.to_string(),
        gosci: 2,
        kwota_goscia: 300_000,
        oplata_sprzatanie: 15_000,
        po_koncu_decyzja_id: None,
        id: stay_id.to_string(),
    }
}

/// The parameters of recording that the stay `stay_id` is paid.
fn stay_payment(stay_id: &str) -> OdnotujOplaceniePobytuParams {
    OdnotujOplaceniePobytuParams {
        id: stay_id.to_string(),
    }
}

/// Records that the stay `stay_id` is paid.
fn record_stay_paid(room: &MemoryRoom, stay_id: &str) {
    lokal_krotkoterminowy_pobyty::odnotuj_oplacenie_pobytu(room, &stay_payment(stay_id))
        .expect("record the payment of the stay");
}

/// The parameters of a change of the state of the stay `stay_id` to `stan`.
fn stay_state(stay_id: &str, stan: StanPobytu) -> ZmienStanPobytuParams {
    ZmienStanPobytuParams {
        stan,
        id: stay_id.to_string(),
    }
}

/// Changes the state of the stay `stay_id` to `stan`.
fn set_stay_state(room: &MemoryRoom, stay_id: &str, stan: StanPobytu) {
    lokal_krotkoterminowy_pobyty::zmien_stan_pobytu(room, &stay_state(stay_id, stan))
        .expect("change the state of the stay");
}

/// The parameters of ending the stay `stay_id` with nothing left behind.
fn stay_end(stay_id: &str) -> ZakonczPobytParams {
    ZakonczPobytParams {
        rzeczy_pozostawione: None,
        id: stay_id.to_string(),
    }
}

/// The parameters of recording that the entry instruction of the stay `stay_id` went out at
/// `instrukcja_wyslano`.
fn entry_instruction(stay_id: &str, instrukcja_wyslano: &str) -> OdnotujInstrukcjeWejsciaParams {
    OdnotujInstrukcjeWejsciaParams {
        instrukcja_wyslano: instrukcja_wyslano.to_string(),
        id: stay_id.to_string(),
    }
}

/// An order from the register's price list, after the guest leaves: cleaning for 150 zł on 13
/// December from 10:00 to 15:00, ordered by the coordinator's agent on 1 December.
fn order(order_id: &str) -> ZlecPraceParams {
    ZlecPraceParams {
        id: order_id.to_string(),
        rodzaj: RodzajZlecenia::Sprzatanie,
        przyczyna: PrzyczynaZlecenia::WyjazdGoscia,
        pobyt_id: Some("pobyt".to_string()),
        zgloszenie_id: None,
        wykonawca_odnosnik: "rejestr-wykonawcow/czysty-dom".to_string(),
        spoza_rejestru: false,
        cennik_wersja_odnosnik: Some("rejestr-wykonawcow/czysty-dom/cennik-3".to_string()),
        poza_standardem: false,
        okno_od: "2026-12-13T10:00:00Z".to_string(),
        okno_do: "2026-12-13T15:00:00Z".to_string(),
        cena: 15_000,
        cena_zrodlo: ZrodloCeny::Cennik,
        zgoda_id: None,
        decyzja_id: None,
        zlecil: "agent-koordynatora".to_string(),
        zlecono: "2026-12-01T09:00:00Z".to_string(),
    }
}

/// Writes the order of `params` (see [`order`]).
fn place_order(room: &MemoryRoom, params: &ZlecPraceParams) {
    lokal_krotkoterminowy_zlecenia::zlec_prace(room, params).expect("place the order");
}

/// The coordinator's agent confirms the order `order_id` at `potwierdzono` with the report
/// `raporty/<id>.pdf`.
fn confirm_order(room: &MemoryRoom, order_id: &str, potwierdzono: &str) {
    let params = PotwierdzWykonanieZleceniaParams {
        raport_sciezka: format!("raporty/{order_id}.pdf"),
        raport_skrot: format!("sha256-{order_id}"),
        potwierdzil: "agent-koordynatora".to_string(),
        potwierdzono: potwierdzono.to_string(),
        id: order_id.to_string(),
    };

    lokal_krotkoterminowy_zlecenia::potwierdz_wykonanie_zlecenia(room, &params)
        .expect("confirm the order");
}

/// The parameters of the cost `koszt` of the order `order_id`, with no decision.
fn cost_of_order(order_id: &str, koszt: i64) -> WpiszKosztZleceniaParams {
    WpiszKosztZleceniaParams {
        koszt,
        koszt_decyzja_id: None,
        id: order_id.to_string(),
    }
}

/// Writes the cost of an order of `params` (see [`cost_of_order`]).
fn record_cost_of_order(room: &MemoryRoom, params: &WpiszKosztZleceniaParams) {
    lokal_krotkoterminowy_zlecenia::wpisz_koszt_zlecenia(room, params)
        .expect("write the cost of the order");
}

/// The parameters of a change of the state of the order `order_id` to `stan`.
fn order_state(order_id: &str, stan: StanZlecenia) -> ZmienStanZleceniaParams {
    ZmienStanZleceniaParams {
        stan,
        id: order_id.to_string(),
    }
}

/// Changes the state of the order `order_id` to `stan`.
fn set_order_state(room: &MemoryRoom, order_id: &str, stan: StanZlecenia) {
    lokal_krotkoterminowy_zlecenia::zmien_stan_zlecenia(room, &order_state(order_id, stan))
        .expect("change the state of the order");
}

/// The default parameters of the guest's access to the stay „pobyt”: the code 4821 from 10
/// December at 15:00 to 13 December at 11:00, set on 9 December.
fn access() -> UstawDostepParams {
    UstawDostepParams {
        pobyt_id: Some("pobyt".to_string()),
        zlecenie_id: None,
        rodzaj: RodzajDostepu::Kod,
        kod: Some("4821".to_string()),
        klucze_liczba: None,
        zamek_odnosnik: None,
        wazny_od: "2026-12-10T15:00:00Z".to_string(),
        wazny_do: "2026-12-13T11:00:00Z".to_string(),
        kto: "gość".to_string(),
        ustawiono: "2026-12-09T09:00:00Z".to_string(),
    }
}

/// Sets the access of `params` (see [`access`]).
fn set_access(room: &MemoryRoom, params: &UstawDostepParams) {
    lokal_krotkoterminowy_zlecenia::ustaw_dostep(room, params).expect("set the access");
}

/// The parameters of recording that the access `access_id` was given at `wydano`.
fn access_given(access_id: i64, wydano: &str) -> OdnotujWydanieDostepuParams {
    OdnotujWydanieDostepuParams {
        wydano: wydano.to_string(),
        id: access_id,
    }
}

/// The parameters of recording that the access `access_id` was taken back at `odebrano`.
fn access_taken_back(access_id: i64, odebrano: &str) -> OdnotujOdebranieDostepuParams {
    OdnotujOdebranieDostepuParams {
        odebrano: odebrano.to_string(),
        id: access_id,
    }
}

/// The parameters of the night `data` in the state `stan`, changed at `zmieniono`, of no stay,
/// order or consent.
fn night(data: &str, stan: StanNocy, zmieniono: &str) -> WpiszNocParams {
    WpiszNocParams {
        data: data.to_string(),
        stan,
        pobyt_id: None,
        zlecenie_id: None,
        zgoda_id: None,
        zmieniono: zmieniono.to_string(),
    }
}

/// Writes the night of `params` (see [`night`]).
fn add_night(room: &MemoryRoom, params: &WpiszNocParams) {
    lokal_krotkoterminowy_kalendarz::wpisz_noc(room, params).expect("write the night");
}

/// The parameters of a change of the night `data` to the state `stan` at `zmieniono`, of no stay,
/// order or consent.
fn night_change(data: &str, stan: StanNocy, zmieniono: &str) -> ZmienNocParams {
    ZmienNocParams {
        stan,
        pobyt_id: None,
        zlecenie_id: None,
        zgoda_id: None,
        zmieniono: zmieniono.to_string(),
        data: data.to_string(),
    }
}

/// The parameters of the night `data` of the stay `stay_id`, booked at `zmieniono`.
fn booked_night(data: &str, stay_id: &str, zmieniono: &str) -> WpiszNocParams {
    WpiszNocParams {
        pobyt_id: Some(stay_id.to_string()),
        ..night(data, StanNocy::Zarezerwowana, zmieniono)
    }
}

/// A letter that waits for the human's consent and goes out with it: the request filed at 09:00
/// of `day`, signed at 10:00, the letter sent at 11:00.
fn send_with_consent(room: &MemoryRoom, letter_id: &str, day: &str) {
    let for_consent = PrzekazWysylkeDoZgodyParams {
        id: letter_id.to_string(),
    };
    sprawa_korespondencja::przekaz_wysylke_do_zgody(room, &for_consent)
        .expect("hand the letter over for consent");

    let decision_id = format!("wyslanie-{letter_id}");
    let request = PoprosODecyzjeParams {
        zlozono: format!("{day}T09:00:00Z"),
        ..decision_request_about(
            &decision_id,
            RodzajDecyzji::Wyslanie,
            "sprawa_korespondencja_wysylki",
            letter_id,
        )
    };
    approved_decision(room, &request, &format!("{day}T10:00:00Z"));

    let sending_with_consent = OdnotujWyslanieParams {
        decyzja_id: Some(decision_id),
        wyslano: format!("{day}T11:00:00Z"),
        ..sending(letter_id)
    };
    send(room, &sending_with_consent);
}

/// Writes the draft letter `letter_id` to the owner from the template of the monthly statement.
fn letter_to_owner(room: &MemoryRoom, letter_id: &str) {
    let params = PrzygotujWysylkeParams {
        adresat_rodzaj: None,
        adresat_nazwa: None,
        adresat_uczestnik_id: Some("wlascicielka".to_string()),
        wzor: Some("zestawienie-miesieczne".to_string()),
        ..letter(letter_id)
    };

    prepare_letter(room, &params);
}

/// Writes the draft letter `letter_id` to the guest in the sales channel, an answer to a review.
fn letter_to_guest(room: &MemoryRoom, letter_id: &str) {
    let params = PrzygotujWysylkeParams {
        adresat_rodzaj: None,
        adresat_nazwa: None,
        adresat_uczestnik_id: Some("gosc".to_string()),
        kanal: KanalWysylki::KanalSprzedazy,
        wzor: Some("odpowiedz-na-opinie".to_string()),
        ..letter(letter_id)
    };

    prepare_letter(room, &params);
}

/// The parameters of the letter `letter_id` to the neighbour from flat 4 (see [`letter`]).
fn letter_to_neighbour(letter_id: &str) -> PrzygotujWysylkeParams {
    PrzygotujWysylkeParams {
        adresat_rodzaj: Some(RodzajAdresata::Inny),
        adresat_nazwa: Some("Sąsiad z lokalu 4".to_string()),
        ..letter(letter_id)
    }
}

/// The parameters of Kontroler's compliant check `check_id` of the account `account_id` at
/// `sprawdzono`.
fn account_check(check_id: &str, account_id: &str, sprawdzono: &str) -> WpiszSprawdzenieParams {
    WpiszSprawdzenieParams {
        rachunek_id: Some(account_id.to_string()),
        ..check_of(
            check_id,
            RodzajSprawdzenia::Rachunek,
            sprawdzono,
            WynikSprawdzenia::Zgodne,
        )
    }
}

/// An account of the owner for `cel`, confirmed by another channel, checked
/// (`sprawdzenie-<id>`) and admitted (`dopuszczenie-<id>`) (R4).
fn owner_account(room: &MemoryRoom, account_id: &str, cel: CelRachunku) {
    let owners_account = WpiszRachunekParams {
        cel,
        posiadacz: "Anna Kowalska".to_string(),
        ..account(account_id, "wlascicielka")
    };
    add_account(room, &owners_account);
    confirm_account(room, account_id);

    let check_id = format!("sprawdzenie-{account_id}");
    check(
        room,
        &account_check(&check_id, account_id, "2026-11-03T09:00:00Z"),
    );

    approved_decision_about(
        room,
        &format!("dopuszczenie-{account_id}"),
        RodzajDecyzji::DopuszczenieRachunku,
        "sprawa_rachunki_wpisy",
        account_id,
    );
}

/// The default parameters of the draft statement `statement_id` for the month `miesiac`, made on
/// 2027-01-05, which corrects nothing.
fn statement(statement_id: &str, miesiac: &str) -> PrzygotujZestawienieParams {
    PrzygotujZestawienieParams {
        id: statement_id.to_string(),
        miesiac: miesiac.to_string(),
        utworzono: "2027-01-05T09:00:00Z".to_string(),
        korekta_do_id: None,
        korekta_decyzja_id: None,
    }
}

/// Writes the draft statement of `params` (see [`statement`]).
fn draft_statement(room: &MemoryRoom, params: &PrzygotujZestawienieParams) {
    lokal_krotkoterminowy_zestawienia::przygotuj_zestawienie(room, params)
        .expect("prepare the statement");
}

/// The parameters of the line `rodzaj` of `kwota` in the statement `statement_id`, with no
/// source yet; set its source with struct update.
fn line(statement_id: &str, rodzaj: RodzajPozycji, kwota: i64) -> DopiszPozycjeZestawieniaParams {
    DopiszPozycjeZestawieniaParams {
        zestawienie_id: statement_id.to_string(),
        rodzaj,
        kwota,
        opis: None,
        pobyt_id: None,
        zlecenie_id: None,
        pieniadze_id: None,
        warunki_id: None,
        dokument_id: None,
    }
}

/// [`line`] whose source is the stay „pobyt”.
fn stay_line(
    statement_id: &str,
    rodzaj: RodzajPozycji,
    kwota: i64,
) -> DopiszPozycjeZestawieniaParams {
    DopiszPozycjeZestawieniaParams {
        pobyt_id: Some("pobyt".to_string()),
        ..line(statement_id, rodzaj, kwota)
    }
}

/// Writes the line of `params` (see [`line`]).
fn add_line(room: &MemoryRoom, params: &DopiszPozycjeZestawieniaParams) {
    lokal_krotkoterminowy_zestawienia::dopisz_pozycje_zestawienia(room, params)
        .expect("add a line of the statement");
}

/// The parameters of closing the statement `statement_id` with its file, the amount to pay out
/// and the reserve after it.
fn statement_closing(
    statement_id: &str,
    kwota_do_wyplaty: i64,
    rezerwa_po: i64,
) -> ZamknijZestawienieParams {
    ZamknijZestawienieParams {
        sciezka: format!("zestawienia/{statement_id}.pdf"),
        skrot: format!("sha256-{statement_id}"),
        kwota_do_wyplaty,
        rezerwa_po,
        id: statement_id.to_string(),
    }
}

/// Closes the statement `statement_id` (see [`statement_closing`]).
fn close_statement(room: &MemoryRoom, statement_id: &str, kwota_do_wyplaty: i64, rezerwa_po: i64) {
    let params = statement_closing(statement_id, kwota_do_wyplaty, rezerwa_po);

    lokal_krotkoterminowy_zestawienia::zamknij_zestawienie(room, &params)
        .expect("close the statement");
}

/// The parameters of naming the letter `letter_id` that sends the statement `statement_id`.
fn statement_letter(statement_id: &str, letter_id: &str) -> WskazWysylkeZestawieniaParams {
    WskazWysylkeZestawieniaParams {
        wysylka_id: letter_id.to_string(),
        id: statement_id.to_string(),
    }
}

/// The parameters of recording that the statement `statement_id` went out.
fn statement_sent(statement_id: &str) -> OdnotujWyslanieZestawieniaParams {
    OdnotujWyslanieZestawieniaParams {
        id: statement_id.to_string(),
    }
}

/// Sends the ready statement `statement_id` to the owner in the letter `wysylka-<id>`, with the
/// human's consent on `day`.
fn send_statement(room: &MemoryRoom, statement_id: &str, day: &str) {
    let letter_id = format!("wysylka-{statement_id}");
    letter_to_owner(room, &letter_id);

    lokal_krotkoterminowy_zestawienia::wskaz_wysylke_zestawienia(
        room,
        &statement_letter(statement_id, &letter_id),
    )
    .expect("name the letter of the statement");
    send_with_consent(room, &letter_id, day);

    lokal_krotkoterminowy_zestawienia::odnotuj_wyslanie_zestawienia(
        room,
        &statement_sent(statement_id),
    )
    .expect("record the sending of the statement");
}

/// The parameters of the payout `payout_id` of `kwota` to the owner on the account
/// „rachunek-wyplat”, based on the statement `statement_id`, prepared at 09:00 of `day`.
fn owner_payout(
    payout_id: &str,
    statement_id: &str,
    kwota: i64,
    day: &str,
) -> PrzygotujWyplateParams {
    PrzygotujWyplateParams {
        id: payout_id.to_string(),
        podstawa_tabela: "lokal_krotkoterminowy_zestawienia_wpisy".to_string(),
        podstawa_id: statement_id.to_string(),
        kwota,
        wyliczenie_sciezka: format!("wyliczenia/{payout_id}.json"),
        wyliczenie_skrot: format!("sha256-{payout_id}"),
        odbiorca_uczestnik_id: "wlascicielka".to_string(),
        rachunek_id: "rachunek-wyplat".to_string(),
        utworzono: format!("{day}T09:00:00Z"),
    }
}

/// [`owner_payout`] „wyplata” of `kwota` from the statement „zestawienie” on 2027-01-08, the
/// payout the refusals of the scenarios start from.
fn payout_fields(kwota: i64) -> PrzygotujWyplateParams {
    owner_payout("wyplata", "zestawienie", kwota, "2027-01-08")
}

/// The payout of a sent statement: the account checked anew, the human's consent, the transfer,
/// all on `day`.
fn pay_owner(room: &MemoryRoom, payout_id: &str, statement_id: &str, kwota: i64, day: &str) {
    prepare_payout(room, &owner_payout(payout_id, statement_id, kwota, day));

    let check_id = format!("sprawdzenie-{payout_id}");
    check(
        room,
        &account_check(&check_id, "rachunek-wyplat", &format!("{day}T10:00:00Z")),
    );
    let check_of_payout = WskazSprawdzenieWyplatyParams {
        sprawdzenie_id: check_id,
        id: payout_id.to_string(),
    };
    sprawa_wyplaty::wskaz_sprawdzenie_wyplaty(room, &check_of_payout)
        .expect("name the check of the payout");

    let decision_id = format!("zgoda-{payout_id}");
    let consent_request = PoprosODecyzjeParams {
        zlozono: format!("{day}T11:00:00Z"),
        ..decision_request_about(
            &decision_id,
            RodzajDecyzji::Wyplata,
            "sprawa_wyplaty_wpisy",
            payout_id,
        )
    };
    approved_decision(room, &consent_request, &format!("{day}T12:00:00Z"));
    let consent_to_payout = WskazZgodeNaWyplateParams {
        decyzja_id: decision_id,
        id: payout_id.to_string(),
    };
    sprawa_wyplaty::wskaz_zgode_na_wyplate(room, &consent_to_payout)
        .expect("name the consent to the payout");

    let transfer_id = format!("przelew-{payout_id}");
    add_received_document(
        room,
        &received_document(&transfer_id, RodzajDokumentu::PotwierdzenieBankowe),
    );
    let execution = OdnotujWykonanieWyplatyParams {
        wykonal: "tozsamosc-posrednika".to_string(),
        wykonano: format!("{day}T13:00:00Z"),
        potwierdzenie_dokument_id: transfer_id,
        id: payout_id.to_string(),
    };
    sprawa_wyplaty::odnotuj_wykonanie_wyplaty(room, &execution).expect("execute the payout");
}

/// The parameters of the settlement of the stay `stay_id` by the channel: 450 zł of its fee, 2550
/// zł paid out on 14 December, recorded on 15 December.
fn stay_settlement(stay_id: &str) -> OdnotujRozliczeniePobytuParams {
    OdnotujRozliczeniePobytuParams {
        prowizja_kanalu: 45_000,
        wyplata_kanalu: 255_000,
        wyplacono: "2026-12-14".to_string(),
        rozliczono: "2026-12-15T09:00:00Z".to_string(),
        id: stay_id.to_string(),
    }
}

/// Records the channel's settlement of the stay „pobyt” (see [`stay_settlement`]).
fn settle_stay(room: &MemoryRoom) {
    lokal_krotkoterminowy_pobyty::odnotuj_rozliczenie_pobytu(room, &stay_settlement("pobyt"))
        .expect("settle the stay");
}

/// The stay „pobyt” is paid, goes on and ends.
fn finished_stay(room: &MemoryRoom) {
    record_stay_paid(room, "pobyt");
    set_stay_state(room, "pobyt", StanPobytu::Trwa);

    lokal_krotkoterminowy_pobyty::zakoncz_pobyt(room, &stay_end("pobyt")).expect("end the stay");
}

/// Writes a short-term rental case with a row in every table of the type, as the profile goes.
fn full_case(room: &MemoryRoom) {
    // Intake, the owner's identity, the contract with a consumer at a distance (R7, R8).
    stage(room, "weryfikacja_wlasciciela", "2026-11-02T12:00:00Z");
    add_owner(
        room,
        &WpiszUczestnikaParams {
            konsument: true,
            ..owner()
        },
    );
    add_flat(room, &flat("lokal"));
    stage(room, "umowa_o_zarzadzanie", "2026-11-03T09:00:00Z");
    sign_management_contract_at_a_distance(room);
    ask_to_start_work(room);
    established_terms(room, &terms("warunki"));

    // The flat: data, house rules, equipment bought by the office, a check-up and a resolution.
    stage(room, "przygotowanie_lokalu", "2026-11-05T09:00:00Z");
    add_named_document(
        room,
        "standard",
        RodzajDokumentu::Standard,
        "Standard lokalu",
    );
    add_named_document(
        room,
        "instrukcja",
        RodzajDokumentu::Instrukcja,
        "Instrukcja wejścia",
    );
    lokal_krotkoterminowy_lokal::wpisz_dane_lokalu(
        room,
        &WpiszDaneLokaluParams {
            nieruchomosc_id: "lokal".to_string(),
            rejestracja_numer: None,
            rejestracja_data: None,
            lozka: 3,
            czas_sprzatania_min: Some(120),
            klucze_liczba: Some(2),
            standard_dokument_id: Some("standard".to_string()),
            instrukcja_dokument_id: Some("instrukcja".to_string()),
        },
    )
    .expect("write the data of the flat");
    add_named_document(
        room,
        "regulamin-1",
        RodzajDokumentu::Regulamin,
        "Regulamin lokalu",
    );
    lokal_krotkoterminowy_lokal::wpisz_regulamin_lokalu(
        room,
        &WpiszRegulaminLokaluParams {
            id: "regulamin-1".to_string(),
            dokument_id: "regulamin-1".to_string(),
            od: CONCLUDED.to_string(),
            decyzja_id: None,
        },
    )
    .expect("write the house rules");
    add_consent(
        room,
        &WpiszZgodeParams {
            tresc: "Pościel.".to_string(),
            ..consent("zgoda-poscel", "wlascicielka", "zgoda_na_wyposazenie")
        },
    );
    lokal_krotkoterminowy_lokal::wpisz_wyposazenie_lokalu(
        room,
        &WpiszWyposazenieLokaluParams {
            rzecz: "pościel, 3 komplety".to_string(),
            od: Some("2026-11-05".to_string()),
            koszt: Some(45_000),
            zgoda_id: Some("zgoda-poscel".to_string()),
        },
    )
    .expect("write the equipment");
    add_named_document(
        room,
        "protokol-gazowy",
        RodzajDokumentu::Inny,
        "Przegląd instalacji gazowej",
    );
    lokal_krotkoterminowy_lokal::wpisz_przeglad_lokalu(
        room,
        &WpiszPrzegladLokaluParams {
            rodzaj: "instalacja_gazowa".to_string(),
            data: "2026-10-20".to_string(),
            nastepny: Some("2027-10-20".to_string()),
            dokument_id: Some("protokol-gazowy".to_string()),
        },
    )
    .expect("write the check-up");
    add_named_document(
        room,
        "uchwala",
        RodzajDokumentu::Uchwala,
        "Uchwała wspólnoty nr 4/2026",
    );
    lokal_krotkoterminowy_lokal::wpisz_uchwale_wspolnoty(
        room,
        &WpiszUchwaleWspolnotyParams {
            data: "2026-10-01".to_string(),
            tresc: "Najem krótkoterminowy tylko z ciszą nocną od 22.".to_string(),
            dokument_id: Some("uchwala".to_string()),
            dowiedziano: "2026-11-05".to_string(),
        },
    )
    .expect("write the resolution");
    signed_protocol(
        room,
        "przejecie",
        RodzajProtokolu::Przejecie,
        "2026-11-05T10:00:00Z",
    );
    place_order(
        room,
        &ZlecPraceParams {
            przyczyna: PrzyczynaZlecenia::PrzygotowanieLokalu,
            pobyt_id: None,
            okno_od: "2026-11-05T12:00:00Z".to_string(),
            okno_do: "2026-11-05T16:00:00Z".to_string(),
            cena: 20_000,
            zlecono: "2026-11-05T09:00:00Z".to_string(),
            ..order("pierwsze-sprzatanie")
        },
    );
    confirm_order(room, "pierwsze-sprzatanie", "2026-11-05T17:00:00Z");
    record_cost_of_order(room, &cost_of_order("pierwsze-sprzatanie", 20_000));

    // The offer in the channel with the account the channel pays to, and a price within limits.
    owner_account(room, "rachunek-kanalu", CelRachunku::WyplataWKanale);
    stage(room, "oferta_w_kanalach", "2026-11-06T09:00:00Z");
    draft_offer(
        room,
        &PrzygotujOferteWKanaleParams {
            rachunek_id: Some("rachunek-kanalu".to_string()),
            ..airbnb_offer()
        },
    );
    publish(room, &publication("oferta-airbnb"));
    lokal_krotkoterminowy_kalendarz::wpisz_wyliczenie_cen(
        room,
        &WpiszWyliczenieCenParams {
            id: "wyliczenie".to_string(),
            chwila: "2026-11-06T11:00:00Z".to_string(),
            dane_wejsciowe: "sezon zimowy, jarmark świąteczny, obłożenie 60%".to_string(),
            zrodla: "kalendarz miasta z 2026-11-06; ceny 5 podobnych lokali z 2026-11-06"
                .to_string(),
            sciezka: "wyliczenia/ceny-grudzien.json".to_string(),
            skrot: "sha256-ceny-grudzien".to_string(),
        },
    )
    .expect("write the price calculation");
    lokal_krotkoterminowy_kalendarz::ustaw_cene_nocy(
        room,
        &UstawCeneNocyParams {
            id: "cena-10-12".to_string(),
            data: "2026-12-10".to_string(),
            kanal: "airbnb".to_string(),
            cena: 100_000,
            ustawiono: "2026-11-06T11:05:00Z".to_string(),
            wyliczenie_id: Some("wyliczenie".to_string()),
            decyzja_id: None,
        },
    )
    .expect("set the price of the night");
    stage(room, "prowadzenie_lokalu", "2026-11-07T09:00:00Z");

    // The calendar: a free night, the owner's night and the guest's stay.
    add_night(
        room,
        &night("2026-12-01", StanNocy::Wolna, "2026-11-07T09:00:00Z"),
    );
    add_consent(
        room,
        &WpiszZgodeParams {
            tresc: "Wigilia dla mnie.".to_string(),
            zlozono: "2026-11-08T09:00:00Z".to_string(),
            ..consent("noce-swieta", "wlascicielka", "noce_wlasciciela")
        },
    );
    add_night(
        room,
        &WpiszNocParams {
            zgoda_id: Some("noce-swieta".to_string()),
            ..night("2026-12-24", StanNocy::Wlasciciel, "2026-11-08T10:00:00Z")
        },
    );
    add_guest(room);
    book(room, &booking("pobyt", "2026-12-10", "2026-12-13"));
    for data in ["2026-12-10", "2026-12-11", "2026-12-12"] {
        add_night(room, &booked_night(data, "pobyt", "2026-11-07T10:05:00Z"));
    }
    lokal_krotkoterminowy_pobyty::wpisz_zmiane_pobytu(
        room,
        &WpiszZmianePobytuParams {
            pobyt_id: "pobyt".to_string(),
            chwila: "2026-11-20T10:00:00Z".to_string(),
            co: CoSieZmienia::LiczbaGosci,
            przyczyna: "gość przyjeżdża z partnerką".to_string(),
            zdecydowal: "agent-zarzadcy".to_string(),
        },
    )
    .expect("write the change of the stay");
    lokal_krotkoterminowy_pobyty::zmien_rezerwacje(
        room,
        &ZmienRezerwacjeParams {
            gosci: 3,
            ..rebooking("pobyt", "2026-12-10", "2026-12-13")
        },
    )
    .expect("change the booking");

    // Before arrival: payment, a late check-out by exception, the code and the entry instruction.
    record_stay_paid(room, "pobyt");
    approved_decision_about(
        room,
        "pozne-wymeldowanie",
        RodzajDecyzji::Inna,
        "lokal_krotkoterminowy_pobyty_wyjatki",
        "wyjatek",
    );
    lokal_krotkoterminowy_pobyty::wpisz_wyjatek_pobytu(
        room,
        &WpiszWyjatekPobytuParams {
            id: "wyjatek".to_string(),
            pobyt_id: "pobyt".to_string(),
            opis: "wymeldowanie o 12:00".to_string(),
            decyzja_id: "pozne-wymeldowanie".to_string(),
        },
    )
    .expect("write the exception of the stay");
    set_access(
        room,
        &UstawDostepParams {
            zamek_odnosnik: Some("zamki/dluga-7-3".to_string()),
            wazny_do: "2026-12-13T12:00:00Z".to_string(),
            kto: "gość z rezerwacji HM-pobyt".to_string(),
            ..access()
        },
    );
    lokal_krotkoterminowy_zlecenia::odnotuj_wydanie_dostepu(
        room,
        &access_given(1, "2026-12-10T10:00:00Z"),
    )
    .expect("give the code");
    lokal_krotkoterminowy_pobyty::odnotuj_instrukcje_wejscia(
        room,
        &entry_instruction("pobyt", "2026-12-10T10:00:00Z"),
    )
    .expect("send the entry instruction");
    set_stay_state(room, "pobyt", StanPobytu::Trwa);

    // During the stay: a broken boiler, a repair from a quote within the repair amount, at a cost
    // other than the quote, and a compensation for the night without hot water.
    lokal_krotkoterminowy_pobyty::wpisz_zgloszenie_goscia(
        room,
        &WpiszZgloszenieGosciaParams {
            id: "zgloszenie".to_string(),
            pobyt_id: "pobyt".to_string(),
            chwila: "2026-12-11T20:00:00Z".to_string(),
            opis: "brak ciepłej wody".to_string(),
            przekazano: None,
            zagrozenie: false,
            zawiadomiono: None,
        },
    )
    .expect("write the guest's report");
    lokal_krotkoterminowy_pobyty::odnotuj_przekazanie_zgloszenia(
        room,
        &OdnotujPrzekazanieZgloszeniaParams {
            przekazano: "Koordynator operacji, 20:05".to_string(),
            id: "zgloszenie".to_string(),
        },
    )
    .expect("pass the report on");
    place_order(
        room,
        &ZlecPraceParams {
            rodzaj: RodzajZlecenia::Naprawa,
            przyczyna: PrzyczynaZlecenia::Zgloszenie,
            zgloszenie_id: Some("zgloszenie".to_string()),
            wykonawca_odnosnik: "rejestr-wykonawcow/hydraulik".to_string(),
            cennik_wersja_odnosnik: None,
            okno_od: "2026-12-20T08:00:00Z".to_string(),
            okno_do: "2026-12-20T18:00:00Z".to_string(),
            cena: 30_000,
            cena_zrodlo: ZrodloCeny::Wycena,
            zlecono: "2026-12-11T20:30:00Z".to_string(),
            ..order("naprawa")
        },
    );
    add_night(
        room,
        &WpiszNocParams {
            zlecenie_id: Some("naprawa".to_string()),
            ..night("2026-12-20", StanNocy::Prace, "2026-12-11T20:35:00Z")
        },
    );
    set_access(
        room,
        &UstawDostepParams {
            pobyt_id: None,
            zlecenie_id: Some("naprawa".to_string()),
            rodzaj: RodzajDostepu::Klucze,
            kod: None,
            klucze_liczba: Some(1),
            wazny_od: "2026-12-20T08:00:00Z".to_string(),
            wazny_do: "2026-12-20T18:00:00Z".to_string(),
            kto: "hydraulik z rejestru".to_string(),
            ustawiono: "2026-12-19T09:00:00Z".to_string(),
            ..access()
        },
    );
    approved_decision_about(
        room,
        "rekompensata-decyzja",
        RodzajDecyzji::Inna,
        "lokal_krotkoterminowy_pobyty_pieniadze",
        "rekompensata",
    );
    lokal_krotkoterminowy_pobyty::wpisz_pieniadze_pobytu(
        room,
        &WpiszPieniadzePobytuParams {
            decyzja_id: Some("rekompensata-decyzja".to_string()),
            ..stay_money("rekompensata", RodzajPieniedzy::Rekompensata)
        },
    )
    .expect("write the compensation");
    lokal_krotkoterminowy_pobyty::wpisz_skarge(
        room,
        &WpiszSkargeParams {
            chwila: "2026-12-12T23:30:00Z".to_string(),
            kto: "sąsiad z lokalu 4".to_string(),
            tresc: "hałas".to_string(),
            pobyt_id: None,
        },
    )
    .expect("write the complaint");
    prepare_letter(
        room,
        &PrzygotujWysylkeParams {
            wzor: Some("odpowiedz-na-skarge".to_string()),
            ..letter_to_neighbour("odpowiedz-sasiadowi")
        },
    );
    lokal_krotkoterminowy_pobyty::wskaz_odpowiedz_na_skarge(
        room,
        &WskazOdpowiedzNaSkargeParams {
            odpowiedz_wysylka_id: "odpowiedz-sasiadowi".to_string(),
            id: 1,
        },
    )
    .expect("name the answer to the complaint");
    send_with_consent(room, "odpowiedz-sasiadowi", "2026-12-13");

    // After departure: cleaning in the window, the code expires, a low review answered with
    // consent, the channel's settlement.
    lokal_krotkoterminowy_pobyty::zakoncz_pobyt(
        room,
        &ZakonczPobytParams {
            rzeczy_pozostawione: Some("ładowarka do telefonu".to_string()),
            ..stay_end("pobyt")
        },
    )
    .expect("end the stay");
    lokal_krotkoterminowy_zlecenia::odnotuj_odebranie_dostepu(
        room,
        &access_taken_back(1, "2026-12-13T12:00:00Z"),
    )
    .expect("take the code back");
    place_order(room, &order("sprzatanie"));
    set_access(
        room,
        &UstawDostepParams {
            pobyt_id: None,
            zlecenie_id: Some("sprzatanie".to_string()),
            kod: Some("9930".to_string()),
            wazny_od: "2026-12-13T10:00:00Z".to_string(),
            wazny_do: "2026-12-13T15:00:00Z".to_string(),
            kto: "ekipa sprzątająca".to_string(),
            ustawiono: "2026-12-12T09:00:00Z".to_string(),
            ..access()
        },
    );
    confirm_order(room, "sprzatanie", "2026-12-13T16:00:00Z");
    record_cost_of_order(room, &cost_of_order("sprzatanie", 15_000));
    confirm_order(room, "naprawa", "2026-12-20T17:00:00Z");
    approved_decision(
        room,
        &PoprosODecyzjeParams {
            zlozono: "2026-12-20T17:30:00Z".to_string(),
            ..decision_request_about(
                "koszt-naprawy",
                RodzajDecyzji::Inna,
                "lokal_krotkoterminowy_zlecenia_wpisy",
                "naprawa",
            )
        },
        "2026-12-20T18:00:00Z",
    );
    record_cost_of_order(
        room,
        &WpiszKosztZleceniaParams {
            koszt_decyzja_id: Some("koszt-naprawy".to_string()),
            ..cost_of_order("naprawa", 32_000)
        },
    );
    lokal_krotkoterminowy_pobyty::wpisz_opinie_goscia(
        room,
        &review("opinia", "pobyt", 2, "2026-12-14T09:00:00Z"),
    )
    .expect("write the review");
    letter_to_guest(room, "odpowiedz-na-opinie");
    lokal_krotkoterminowy_pobyty::wskaz_odpowiedz_na_opinie(
        room,
        &review_answer("opinia", "odpowiedz-na-opinie"),
    )
    .expect("name the answer to the review");
    send_with_consent(room, "odpowiedz-na-opinie", "2026-12-15");
    settle_stay(room);

    // The statement for December: every line with its source, the amount is their sum.
    add_named_document(
        room,
        "naliczenie-wspolnoty",
        RodzajDokumentu::Pismo,
        "Naliczenie za grudzień",
    );
    draft_statement(room, &statement("zestawienie", "2026-12"));
    let december_lines = [
        stay_line("zestawienie", RodzajPozycji::Przychod, 300_000),
        stay_line("zestawienie", RodzajPozycji::ProwizjaKanalu, -45_000),
        DopiszPozycjeZestawieniaParams {
            warunki_id: Some("warunki".to_string()),
            ..stay_line("zestawienie", RodzajPozycji::ProwizjaBiura, -38_250)
        },
        DopiszPozycjeZestawieniaParams {
            zlecenie_id: Some("pierwsze-sprzatanie".to_string()),
            ..line("zestawienie", RodzajPozycji::Koszt, -20_000)
        },
        DopiszPozycjeZestawieniaParams {
            zlecenie_id: Some("sprzatanie".to_string()),
            ..line("zestawienie", RodzajPozycji::Koszt, -15_000)
        },
        DopiszPozycjeZestawieniaParams {
            zlecenie_id: Some("naprawa".to_string()),
            ..line("zestawienie", RodzajPozycji::Koszt, -32_000)
        },
        DopiszPozycjeZestawieniaParams {
            pieniadze_id: Some("rekompensata".to_string()),
            ..line("zestawienie", RodzajPozycji::Zwrot, -10_000)
        },
        DopiszPozycjeZestawieniaParams {
            dokument_id: Some("naliczenie-wspolnoty".to_string()),
            ..line("zestawienie", RodzajPozycji::OplataLokalu, -5_000)
        },
        DopiszPozycjeZestawieniaParams {
            warunki_id: Some("warunki".to_string()),
            ..line("zestawienie", RodzajPozycji::Rezerwa, -50_000)
        },
    ];
    for december_line in &december_lines {
        add_line(room, december_line);
    }
    close_statement(room, "zestawienie", 84_750, 50_000);
    send_statement(room, "zestawienie", "2027-01-06");

    // The payout after the account is checked anew (R4, D13).
    owner_account(room, "rachunek-wyplat", CelRachunku::WyplataDlaWlasciciela);
    pay_owner(
        room,
        "wyplata-grudzien",
        "zestawienie",
        84_750,
        "2027-01-08",
    );

    // The end: no nights after the end date, the offer closed, the flat and keys returned.
    stage(room, "koniec_umowy", "2027-01-10T09:00:00Z");
    add_night(
        room,
        &night("2027-02-01", StanNocy::PoKoncu, "2027-01-10T09:05:00Z"),
    );
    stage(room, "zamkniecie_lokalu", "2027-02-01T09:00:00Z");
    set_offer_state(room, "oferta-airbnb", StanOferty::Zamknieta);
    signed_protocol(
        room,
        "zwrot",
        RodzajProtokolu::Zwrot,
        "2027-02-01T10:00:00Z",
    );
    stage(room, "ostatnie_rozliczenie", "2027-02-02T09:00:00Z");

    // The last statement corrects December with the human's consent and releases the reserve.
    add_named_document(
        room,
        "korekta-wspolnoty",
        RodzajDokumentu::Pismo,
        "Korekta naliczenia",
    );
    approved_decision_about(
        room,
        "korekta",
        RodzajDecyzji::Inna,
        "lokal_krotkoterminowy_zestawienia_wpisy",
        "zestawienie-ostatnie",
    );
    draft_statement(
        room,
        &PrzygotujZestawienieParams {
            korekta_do_id: Some("zestawienie".to_string()),
            korekta_decyzja_id: Some("korekta".to_string()),
            ..statement("zestawienie-ostatnie", "2027-01")
        },
    );
    add_line(
        room,
        &DopiszPozycjeZestawieniaParams {
            dokument_id: Some("korekta-wspolnoty".to_string()),
            ..line("zestawienie-ostatnie", RodzajPozycji::Korekta, 5_000)
        },
    );
    add_line(
        room,
        &DopiszPozycjeZestawieniaParams {
            warunki_id: Some("warunki".to_string()),
            ..line("zestawienie-ostatnie", RodzajPozycji::Rezerwa, 50_000)
        },
    );
    close_statement(room, "zestawienie-ostatnie", 55_000, 0);
    send_statement(room, "zestawienie-ostatnie", "2027-02-03");
    pay_owner(
        room,
        "wyplata-ostatnia",
        "zestawienie-ostatnie",
        55_000,
        "2027-02-04",
    );
    stage(room, "po_zarzadzaniu", "2027-02-05T09:00:00Z");
}

/// The default parameters of the money `money_id` of the kind `rodzaj` of the stay „pobyt”: 100
/// zł on 12 December, outside the channel's rules, with no decision.
fn stay_money(money_id: &str, rodzaj: RodzajPieniedzy) -> WpiszPieniadzePobytuParams {
    WpiszPieniadzePobytuParams {
        id: money_id.to_string(),
        pobyt_id: "pobyt".to_string(),
        rodzaj,
        wedlug_zasad_kanalu: false,
        kwota: 10_000,
        chwila: "2026-12-12T09:00:00Z".to_string(),
        opis: None,
        decyzja_id: None,
    }
}

/// The parameters of the guest's review `review_id` of the stay `stay_id` with the rating `ocena`.
fn review(
    review_id: &str,
    stay_id: &str,
    ocena: i64,
    opublikowano: &str,
) -> WpiszOpinieGosciaParams {
    WpiszOpinieGosciaParams {
        id: review_id.to_string(),
        pobyt_id: stay_id.to_string(),
        ocena,
        opublikowano: opublikowano.to_string(),
    }
}

/// The parameters of naming the letter `letter_id` as the answer to the review `review_id`.
fn review_answer(review_id: &str, letter_id: &str) -> WskazOdpowiedzNaOpinieParams {
    WskazOdpowiedzNaOpinieParams {
        odpowiedz_wysylka_id: letter_id.to_string(),
        id: review_id.to_string(),
    }
}

// The whole case ----------------------------------------------------------------------------------

/// Przebieg lokalu: umowa, oferta, pobyt z kodem, zlecenia, zestawienia i wypłaty
#[test]
fn whole_case() {
    let room = new_room();

    full_case(&room);
    stage(&room, "zamknieta", "2027-02-06T09:00:00Z");

    let stages = sprawa_metryka::etapy(&room).expect("read the stages");
    let etapy: Vec<&str> = stages.iter().map(|row| row.etap.as_str()).collect();
    assert_eq!(
        etapy,
        [
            "przyjeta",
            "weryfikacja_wlasciciela",
            "umowa_o_zarzadzanie",
            "przygotowanie_lokalu",
            "oferta_w_kanalach",
            "prowadzenie_lokalu",
            "koniec_umowy",
            "zamkniecie_lokalu",
            "ostatnie_rozliczenie",
            "po_zarzadzaniu",
            "zamknieta",
        ]
    );

    let payouts = sprawa_wyplaty::wyplaty(&room).expect("read the payouts");
    let payout_states: Vec<StanWyplaty> = payouts.iter().map(|row| row.stan).collect();
    assert_eq!(
        payout_states,
        [StanWyplaty::Wykonana, StanWyplaty::Wykonana]
    );

    let statements =
        lokal_krotkoterminowy_zestawienia::zestawienia(&room).expect("read the statements");
    let statement_states: Vec<StanZestawienia> = statements.iter().map(|row| row.stan).collect();
    assert_eq!(
        statement_states,
        [StanZestawienia::Wyslane, StanZestawienia::Wyslane]
    );

    let stays_in_progress =
        lokal_krotkoterminowy_pobyty::pobyty_w_toku(&room).expect("read the stays in progress");
    assert!(stays_in_progress.is_empty());

    let open_orders =
        lokal_krotkoterminowy_zlecenia::zlecenia_otwarte(&room).expect("read the open orders");
    assert!(open_orders.is_empty());

    let contract_of_flat = lokal_krotkoterminowy_umowa::umowa_lokalu(&room)
        .expect("read the contract of the flat")
        .expect("the flat has a contract");
    assert_eq!(contract_of_flat.praca_od, "2026-11-04T13:00:00Z");

    let current_terms = lokal_krotkoterminowy_umowa::biezace_warunki_umowy_lokalu(&room)
        .expect("read the current terms")
        .expect("the contract has current terms");
    assert_eq!(current_terms.id, "warunki");

    let prices = lokal_krotkoterminowy_kalendarz::ceny_nocy(&room).expect("read the prices");
    assert_eq!(prices[0].cena, 100_000);

    let accesses =
        lokal_krotkoterminowy_zlecenia::dostepy_do_lokalu(&room).expect("read the accesses");
    let guest_access = &accesses[0];
    assert!(
        guest_access.kod.as_deref() == Some("4821")
            && guest_access.odebrano.as_deref() == Some("2026-12-13T12:00:00Z"),
        "{guest_access:?}"
    );
}

// Rules of the case on the core -------------------------------------------------------------------

/// Reguły: teczka lokalu ma typ pokoju lokal_krotkoterminowy
#[test]
fn room_type() {
    let room = empty_room();

    let rental_record = ZalozMetrykeParams {
        oznaczenie: "Najem 2026/017".to_string(),
        typ_pokoju: "najem".to_string(),
        przyjal: "tozsamosc-posrednika".to_string(),
        przyjeto: "2026-11-02T09:00:00Z".to_string(),
    };
    expect::refused(
        sprawa_metryka::zaloz_metryke(&room, &rental_record),
        "teczka lokalu krótkoterminowego ma typ pokoju lokal_krotkoterminowy",
    );

    open_case(&room, TYPE);
    let record = sprawa_metryka::metryka(&room)
        .expect("read the record")
        .expect("the case has a record");
    assert_eq!(record.typ_pokoju, TYPE);
}

/// Reguły: etapy lokalu idą po kolei, a przerwać można każdy przed zamknięciem
#[test]
fn stages_in_order() {
    let room = new_room();

    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &stage_change("umowa", "2026-11-02T10:00:00Z")),
        "etap spoza etapów lokalu krótkoterminowego",
    );
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change("umowa_o_zarzadzanie", "2026-11-02T10:00:00Z"),
        ),
        "etapy lokalu idą po kolei",
    );
    stage(&room, "weryfikacja_wlasciciela", "2026-11-02T10:00:00Z");
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change("weryfikacja_wlasciciela", "2026-11-02T11:00:00Z"),
        ),
        "etapy lokalu idą po kolei",
    );
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change("zamknieta", "2026-11-02T11:00:00Z"),
        ),
        "etapy lokalu idą po kolei",
    );

    approved_decision(
        &room,
        &decision_request("przerwanie", RodzajDecyzji::PrzerwanieSprawy),
        APPROVED,
    );
    stage_by_decision(&room, "przerwana", "2026-11-03T13:00:00Z", "przerwanie");
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change("umowa_o_zarzadzanie", "2026-11-03T14:00:00Z"),
        ),
        "etapy lokalu idą po kolei",
    );
    stage(&room, "zamknieta", "2026-11-03T14:00:00Z");
}

/// Reguły: wznowiony lokal wraca do etapu sprzed przerwania, który dalej wymaga zawartej umowy
#[test]
fn stages_resumed() {
    let room = new_room();
    add_owner(&room, &owner());
    add_flat(&room, &flat("lokal"));
    to_contract_stage(&room);
    sign_management_contract(&room, Some(CONTRACT_ENDS));
    stage(&room, "przygotowanie_lokalu", "2026-11-05T09:00:00Z");
    approved_decision(
        &room,
        &decision_request("przerwanie", RodzajDecyzji::PrzerwanieSprawy),
        APPROVED,
    );
    stage_by_decision(&room, "przerwana", "2026-11-06T09:00:00Z", "przerwanie");
    approved_decision(
        &room,
        &decision_request("wznowienie", RodzajDecyzji::WznowienieSprawy),
        "2026-11-06T10:00:00Z",
    );

    let resumed_at_contract = ZmienEtapSprawyParams {
        decyzja_id: Some("wznowienie".to_string()),
        ..stage_change("umowa_o_zarzadzanie", "2026-11-06T11:00:00Z")
    };
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &resumed_at_contract),
        "etapy lokalu idą po kolei",
    );
    stage_by_decision(
        &room,
        "przygotowanie_lokalu",
        "2026-11-06T11:00:00Z",
        "wznowienie",
    );

    approved_decision(
        &room,
        &decision_request("drugie-przerwanie", RodzajDecyzji::PrzerwanieSprawy),
        APPROVED,
    );
    stage_by_decision(
        &room,
        "przerwana",
        "2026-11-07T09:00:00Z",
        "drugie-przerwanie",
    );
    end_contract(&room, "umowa", "wypowiedzenie przez właścicielkę");
    approved_decision(
        &room,
        &decision_request("drugie-wznowienie", RodzajDecyzji::WznowienieSprawy),
        "2026-11-07T10:00:00Z",
    );
    let resumed_without_contract = ZmienEtapSprawyParams {
        decyzja_id: Some("drugie-wznowienie".to_string()),
        ..stage_change("przygotowanie_lokalu", "2026-11-07T11:00:00Z")
    };
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &resumed_without_contract),
        "etap przygotowanie_lokalu wymaga zawartej umowy o zarządzanie",
    );
    stage(&room, "zamknieta", "2026-11-07T12:00:00Z");
}

/// Moves a new case through the owner's verification to the stage of the contract.
fn to_contract_stage(room: &MemoryRoom) {
    stage(room, "weryfikacja_wlasciciela", "2026-11-02T10:00:00Z");
    stage(room, "umowa_o_zarzadzanie", "2026-11-03T09:00:00Z");
}

/// Reguły: przygotowanie lokalu wymaga zawartej umowy o zarządzanie
#[test]
fn preparation_after_contract() {
    let room = new_room();
    add_owner(&room, &owner());
    add_flat(&room, &flat("lokal"));
    to_contract_stage(&room);

    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change("przygotowanie_lokalu", "2026-11-04T09:00:00Z"),
        ),
        "etap przygotowanie_lokalu wymaga zawartej umowy o zarządzanie",
    );

    sign_management_contract(&room, Some(CONTRACT_ENDS));
    stage(&room, "przygotowanie_lokalu", "2026-11-05T09:00:00Z");
}

/// Reguły: oferta w kanałach od praca_od konsumenta i po protokole przejęcia
#[test]
fn offer_stage_after_work_start() {
    let room = new_room();
    add_owner(
        &room,
        &WpiszUczestnikaParams {
            konsument: true,
            ..owner()
        },
    );
    add_flat(&room, &flat("lokal"));
    to_contract_stage(&room);
    sign_management_contract_at_a_distance(&room);
    stage(&room, "przygotowanie_lokalu", "2026-11-05T09:00:00Z");
    signed_protocol(
        &room,
        "przejecie",
        RodzajProtokolu::Przejecie,
        "2026-11-05T10:00:00Z",
    );
    let refusal = "etap oferta_w_kanalach wymaga zawartej umowy, od której biuro już może pracować";

    // Without the consumer's request the office works from the day after the withdrawal period.
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change("oferta_w_kanalach", "2026-11-06T09:00:00Z"),
        ),
        refusal,
    );

    ask_to_start_work(&room);
    stage(&room, "oferta_w_kanalach", "2026-11-06T09:00:00Z");
}

/// Reguły: oferta w kanałach wymaga podpisanego protokołu przejęcia
#[test]
fn offer_stage_after_takeover() {
    let room = new_room();
    add_owner(&room, &owner());
    add_flat(&room, &flat("lokal"));
    to_contract_stage(&room);
    sign_management_contract(&room, Some(CONTRACT_ENDS));
    stage(&room, "przygotowanie_lokalu", "2026-11-05T09:00:00Z");
    prepare_protocol(
        &room,
        "przejecie",
        RodzajProtokolu::Przejecie,
        "2026-11-05T10:00:00Z",
    );

    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change("oferta_w_kanalach", "2026-11-06T09:00:00Z"),
        ),
        "i podpisanego protokołu przejęcia",
    );

    add_received_document(
        &room,
        &received_document("podpisany", RodzajDokumentu::Protokol),
    );
    sign_protocol(&room, "przejecie", "podpisany");
    stage(&room, "oferta_w_kanalach", "2026-11-06T09:00:00Z");
}

/// A case in the offer stage, with the contract without or with an end date `koniec`.
fn offering_flat(room: &MemoryRoom, koniec: Option<&str>) {
    add_owner(room, &owner());
    add_flat(room, &flat("lokal"));
    to_contract_stage(room);
    sign_management_contract(room, koniec);
    stage(room, "przygotowanie_lokalu", "2026-11-05T09:00:00Z");
    signed_protocol(
        room,
        "przejecie",
        RodzajProtokolu::Przejecie,
        "2026-11-05T10:00:00Z",
    );
    stage(room, "oferta_w_kanalach", "2026-11-06T09:00:00Z");
}

/// Reguły: koniec umowy ma podstawę: datę końca albo wypowiedzenie
#[test]
fn contract_end_needs_ground() {
    let refusal = "etap koniec_umowy wymaga daty końca umowy o zarządzanie, jej wypowiedzenia";

    // A contract for a fixed term ends on its end date, set here by an annex.
    let room = new_room();
    offering_flat(&room, None);
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change("koniec_umowy", "2026-12-01T09:00:00Z"),
        ),
        refusal,
    );
    add_contract_event(
        &room,
        &WpiszZdarzenieUmowyParams {
            dokument_id: Some("podpisana-umowa".to_string()),
            ..contract_event(
                "aneks-koniec",
                RodzajZdarzeniaUmowy::Aneks,
                "2026-11-30T12:00:00Z",
            )
        },
    );
    sprawa_umowy::odnotuj_aneks_umowy(
        &room,
        &OdnotujAneksUmowyParams {
            koniec: Some("2027-01-31".to_string()),
            okres_wypowiedzenia: None,
            wylacznosc: true,
            wynagrodzenie_opis: Some("10% czynszu miesięcznie".to_string()),
            wynagrodzenie_zdarzenie: None,
            warunki_od: "2026-11-30T12:00:00Z".to_string(),
            id: "umowa".to_string(),
        },
    )
    .expect("record the annex");
    stage(&room, "koniec_umowy", "2026-12-01T09:00:00Z");

    // A contract for an indefinite term ends by notice; the core changes its end date only by
    // annex.
    let room = new_room();
    offering_flat(&room, None);
    add_received_document(
        &room,
        &received_document("wypowiedzenie", RodzajDokumentu::Pismo),
    );
    add_contract_event(
        &room,
        &WpiszZdarzenieUmowyParams {
            dokument_id: Some("wypowiedzenie".to_string()),
            ..contract_event(
                "wypowiedzenie",
                RodzajZdarzeniaUmowy::Wypowiedzenie,
                "2026-11-30T12:00:00Z",
            )
        },
    );
    stage(&room, "koniec_umowy", "2026-12-01T09:00:00Z");
}

/// Reguły: ostatnie rozliczenie po podpisanym protokole zwrotu lokalu
#[test]
fn last_settlement_after_return() {
    let room = new_room();
    offering_flat(&room, Some(CONTRACT_ENDS));
    stage(&room, "koniec_umowy", "2027-01-10T09:00:00Z");
    stage(&room, "zamkniecie_lokalu", "2027-02-01T09:00:00Z");

    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change("ostatnie_rozliczenie", "2027-02-02T09:00:00Z"),
        ),
        "etap ostatnie_rozliczenie wymaga podpisanego protokołu zwrotu lokalu",
    );

    signed_protocol(
        &room,
        "zwrot",
        RodzajProtokolu::Zwrot,
        "2027-02-01T10:00:00Z",
    );
    stage(&room, "ostatnie_rozliczenie", "2027-02-02T09:00:00Z");
}

/// Reguły: role, zgody i umowa o zarządzanie z listy lokalu
#[test]
fn roles_consents_contract_kind() {
    let room = new_room();
    add_person(&room, &person("osoba"));

    let client_role = NadajRoleUczestnikowiParams {
        uczestnik_id: "osoba".to_string(),
        rola: "klient".to_string(),
        dotyczy_uczestnik_id: None,
    };
    expect::refused(
        sprawa_uczestnicy::nadaj_role_uczestnikowi(&room, &client_role),
        "rola spoza ról lokalu",
    );
    give_role(&room, "osoba", "zarzadca_budynku");
    expect::refused(
        room.execute("UPDATE sprawa_uczestnicy_role SET rola = 'najemca'", &[]),
        "rola spoza ról lokalu",
    );

    let price_approval = WpiszZgodeParams {
        tresc: "Cena 500 zł".to_string(),
        wplyw_id: None,
        ..consent("cena", "osoba", "zatwierdzenie_ceny")
    };
    expect::refused(
        sprawa_zgody::wpisz_zgode(&room, &price_approval),
        "zgoda spoza zgód lokalu",
    );
    add_consent(
        &room,
        &WpiszZgodeParams {
            tresc: "Zgadzam się na Vrbo.".to_string(),
            ..consent("kanal", "osoba", "zgoda_na_kanal")
        },
    );

    let brokerage = PrzygotujUmoweParams {
        rodzaj: RodzajUmowy::Posrednictwo,
        wylacznosc: false,
        wynagrodzenie_opis: None,
        ..contract("posrednictwo")
    };
    expect::refused(
        sprawa_umowy::przygotuj_umowe(&room, &brokerage),
        "teczka lokalu ma tylko umowę o zarządzanie",
    );
    let management = PrzygotujUmoweParams {
        wylacznosc: false,
        wynagrodzenie_opis: None,
        ..contract("umowa")
    };
    sprawa_umowy::przygotuj_umowe(&room, &management).expect("prepare the management contract");
    expect::refused(
        room.execute("UPDATE sprawa_umowy_wpisy SET rodzaj = 'posrednictwo'", &[]),
        "rodzaj umowy się nie zmienia",
    );
}

/// Reguły: teczka lokalu ma jedną zawartą umowę; nowa umowa to nowa teczka
#[test]
fn one_concluded_contract() {
    let room = new_room();
    add_owner(&room, &owner());
    add_flat(&room, &flat("lokal"));
    sign_management_contract(&room, Some(CONTRACT_ENDS));
    prepare_contract(&room, &contract("druga"), &["wlascicielka"]);
    hand_over_for_signing(&room, "druga");

    let second_conclusion = OdnotujZawarcieUmowyParams {
        forma: FormaUmowy::Pisemna,
        sposob_zawarcia: SposobZawarcia::WLokaluBiura,
        zawarto: "2026-11-10T12:00:00Z".to_string(),
        podpisal_za_biuro: "tozsamosc-posrednika".to_string(),
        dokument_id: "podpisana-umowa".to_string(),
        polisa_dokument_id: "polisa-umowa".to_string(),
        odstapienie_do: None,
        id: "druga".to_string(),
    };
    expect::refused(
        sprawa_umowy::odnotuj_zawarcie_umowy(&room, &second_conclusion),
        "teczka lokalu ma najwyżej jedną zawartą umowę o zarządzanie",
    );

    end_contract(&room, "druga", "klientka podpisała pierwszą umowę");
}

/// Reguły: każde pismo do wspólnoty i zarządcy budynku wychodzi za zgodą
#[test]
fn letters_to_association_with_consent() {
    let room = new_room();
    add_person(
        &room,
        &WpiszUczestnikaParams {
            imiona: Some("Piotr".to_string()),
            nazwisko: Some("Zieliński".to_string()),
            ..person("zarzadca")
        },
    );
    give_role(&room, "zarzadca", "zarzadca_budynku");
    prepare_letter(&room, &letter("do-wspolnoty"));
    prepare_letter(
        &room,
        &PrzygotujWysylkeParams {
            adresat_rodzaj: None,
            adresat_nazwa: None,
            adresat_uczestnik_id: Some("zarzadca".to_string()),
            ..letter("do-zarzadcy")
        },
    );
    prepare_letter(
        &room,
        &PrzygotujWysylkeParams {
            adresat_rodzaj: Some(RodzajAdresata::Urzad),
            adresat_nazwa: Some("Urząd Miasta".to_string()),
            ..letter("do-urzedu")
        },
    );
    let refusal =
        "pismo do wspólnoty, spółdzielni albo zarządcy budynku przed wysłaniem czeka na zgodę";

    for letter_id in ["do-wspolnoty", "do-zarzadcy"] {
        expect::refused(
            sprawa_korespondencja::odnotuj_wyslanie(&room, &sending(letter_id)),
            refusal,
        );
    }

    send_with_consent(&room, "do-wspolnoty", "2026-11-05");
    send(&room, &sending("do-urzedu"));
}

// Terms of the management contract ----------------------------------------------------------------

/// Umowa: warunki powstają jako projekt przy zawartej umowie i obowiązują od zawarcia
#[test]
fn terms_start_with_contract() {
    let room = new_room();
    add_owner(&room, &owner());
    add_flat(&room, &flat("lokal"));
    prepare_contract(&room, &contract("umowa"), &["wlascicielka"]);

    expect::refused(
        lokal_krotkoterminowy_umowa::wpisz_warunki_umowy_lokalu(&room, &terms_fields()),
        "warunki wpisuje się do zawartej umowy o zarządzanie",
    );

    check_identity(&room, "tozsamosc", "wlascicielka");
    hand_over_for_signing(&room, "umowa");
    conclude_contract(&room, "umowa", SposobZawarcia::WLokaluBiura, None);
    let terms_after_conclusion = WpiszWarunkiUmowyLokaluParams {
        od: "2026-11-05T09:00:00Z".to_string(),
        ..terms_fields()
    };
    expect::refused(
        lokal_krotkoterminowy_umowa::wpisz_warunki_umowy_lokalu(&room, &terms_after_conclusion),
        "pierwsze warunki obowiązują od zawarcia umowy",
    );
    expect::refused(
        room.execute(
            "INSERT INTO lokal_krotkoterminowy_umowa_warunki (id, umowa_id, max_gosci,
              prowizja_rodzaj, prowizja, prowizja_podstawa, oplata_sprzatanie,
              oplata_sprzatanie_pokrywa_koszt, kwota_napraw, koszty_wlasciciela,
              zgloszenie_nocy_dni, wyplata_dzien, rezerwa, po_koncu_rezerwacje, od, stan)
            VALUES ('od-razu', 'umowa', 4, 'procent', 1500, 'przychód', 0, 1, 0, 'brak', 14, 15,
              0, 'brak', :od, 'obowiazuje')",
            &[("od", text(CONCLUDED))],
        ),
        "warunki umowy powstają jako projekt",
    );

    established_terms(&room, &terms("warunki"));
    let current_terms = lokal_krotkoterminowy_umowa::biezace_warunki_umowy_lokalu(&room)
        .expect("read the current terms")
        .expect("the contract has current terms");
    assert_eq!(current_terms.stan, StanWarunkow::Obowiazuje);
}

/// Umowa: następne warunki wskazują aneks i obowiązują od jego chwili
#[test]
fn terms_change_by_annex() {
    let room = new_room();
    add_owner(&room, &owner());
    add_flat(&room, &flat("lokal"));
    sign_management_contract(&room, Some(CONTRACT_ENDS));
    established_terms(&room, &terms("warunki"));

    let terms_without_annex = WpiszWarunkiUmowyLokaluParams {
        id: "bez-aneksu".to_string(),
        od: "2026-12-01T09:00:00Z".to_string(),
        ..terms_fields()
    };
    expect::refused(
        lokal_krotkoterminowy_umowa::wpisz_warunki_umowy_lokalu(&room, &terms_without_annex),
        "każde następne wskazują aneks tej umowy",
    );

    add_contract_event(
        &room,
        &WpiszZdarzenieUmowyParams {
            dokument_id: Some("podpisana-umowa".to_string()),
            ..contract_event("aneks", RodzajZdarzeniaUmowy::Aneks, "2026-12-01T09:00:00Z")
        },
    );
    let terms_at_another_moment = WpiszWarunkiUmowyLokaluParams {
        id: "zla-chwila".to_string(),
        od: "2026-12-02T09:00:00Z".to_string(),
        aneks_zdarzenie_id: Some("aneks".to_string()),
        ..terms_fields()
    };
    expect::refused(
        lokal_krotkoterminowy_umowa::wpisz_warunki_umowy_lokalu(&room, &terms_at_another_moment),
        "każde następne wskazują aneks tej umowy",
    );

    established_terms(
        &room,
        &WpiszWarunkiUmowyLokaluParams {
            od: "2026-12-01T09:00:00Z".to_string(),
            aneks_zdarzenie_id: Some("aneks".to_string()),
            ..terms("warunki-2")
        },
    );
    let current_terms = lokal_krotkoterminowy_umowa::biezace_warunki_umowy_lokalu(&room)
        .expect("read the current terms")
        .expect("the contract has current terms");
    assert_eq!(current_terms.id, "warunki-2");
}

/// Umowa: ustalone warunki mają kanał i się nie zmieniają, a kanały i widełki tylko w projekcie
#[test]
fn terms_frozen() {
    let room = new_room();
    add_owner(&room, &owner());
    add_flat(&room, &flat("lokal"));
    sign_management_contract(&room, Some(CONTRACT_ENDS));
    let commission_above_100_percent = WpiszWarunkiUmowyLokaluParams {
        prowizja: 12_000,
        ..terms_fields()
    };
    expect::refused(
        lokal_krotkoterminowy_umowa::wpisz_warunki_umowy_lokalu(
            &room,
            &commission_above_100_percent,
        ),
        "procent_najwyzej_100",
    );
    draft_terms(&room, &terms("warunki"));

    expect::refused(
        lokal_krotkoterminowy_umowa::ustal_warunki_umowy_lokalu(&room, &establishment("warunki")),
        "ustalone warunki mają co najmniej jeden kanał sprzedaży",
    );

    add_terms_channel(
        &room,
        &terms_channel("warunki", "airbnb", KontoWarunkow::Biura),
    );
    lokal_krotkoterminowy_umowa::ustal_warunki_umowy_lokalu(&room, &establishment("warunki"))
        .expect("establish the terms");

    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_umowa_warunki SET kwota_napraw = 90000",
            &[],
        ),
        "warunków umowy się nie zmienia",
    );
    expect::refused(
        lokal_krotkoterminowy_umowa::dopisz_kanal_warunkow(
            &room,
            &terms_channel("warunki", "booking", KontoWarunkow::Biura),
        ),
        "kanały dopisuje się tylko do projektu warunków",
    );
    let summer_limits = DopiszWidelkiWarunkowParams {
        warunki_id: "warunki".to_string(),
        sezon: "lato".to_string(),
        od_dnia: "2027-06-01".to_string(),
        do_dnia: "2027-08-31".to_string(),
        cena_min: 30_000,
        cena_max: 90_000,
        pobyt_min: 2,
    };
    expect::refused(
        lokal_krotkoterminowy_umowa::dopisz_widelki_warunkow(&room, &summer_limits),
        "widełki dopisuje się tylko do projektu warunków",
    );
    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_umowa_kanaly SET konto = 'wlasciciela'",
            &[],
        ),
        "kanału warunków się nie zmienia",
    );
}

/// Umowa: widełki mają cenę najniższą nie większą niż najwyższą, a okresy się nie nakładają
#[test]
fn price_limits() {
    let room = new_room();
    add_owner(&room, &owner());
    add_flat(&room, &flat("lokal"));
    sign_management_contract(&room, Some(CONTRACT_ENDS));
    draft_terms(&room, &terms("warunki"));
    let limits = DopiszWidelkiWarunkowParams {
        warunki_id: "warunki".to_string(),
        sezon: "zima".to_string(),
        od_dnia: "2026-11-01".to_string(),
        do_dnia: "2027-03-31".to_string(),
        cena_min: 25_000,
        cena_max: 90_000,
        pobyt_min: 2,
    };

    let lowest_above_highest = DopiszWidelkiWarunkowParams {
        cena_max: 20_000,
        ..limits.clone()
    };
    expect::refused(
        lokal_krotkoterminowy_umowa::dopisz_widelki_warunkow(&room, &lowest_above_highest),
        "cena_min_nie_wieksza_niz_max",
    );
    lokal_krotkoterminowy_umowa::dopisz_widelki_warunkow(&room, &limits)
        .expect("add the winter limits");
    let overlapping = DopiszWidelkiWarunkowParams {
        sezon: "ferie".to_string(),
        od_dnia: "2027-02-01".to_string(),
        do_dnia: "2027-02-28".to_string(),
        ..limits.clone()
    };
    expect::refused(
        lokal_krotkoterminowy_umowa::dopisz_widelki_warunkow(&room, &overlapping),
        "okresy widełek jednej wersji warunków się nie nakładają",
    );
    let spring = DopiszWidelkiWarunkowParams {
        sezon: "wiosna".to_string(),
        od_dnia: "2027-04-01".to_string(),
        do_dnia: "2027-05-31".to_string(),
        ..limits
    };
    lokal_krotkoterminowy_umowa::dopisz_widelki_warunkow(&room, &spring)
        .expect("add the spring limits");
    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_umowa_widelki SET cena_max = 99000",
            &[],
        ),
        "widełek się nie zmienia",
    );
}

// The flat ----------------------------------------------------------------------------------------

/// The parameters of the data of the flat with `lozka` beds and nothing else.
fn flat_data_with_beds(lozka: i64) -> WpiszDaneLokaluParams {
    WpiszDaneLokaluParams {
        nieruchomosc_id: "lokal".to_string(),
        rejestracja_numer: None,
        rejestracja_data: None,
        lozka,
        czas_sprzatania_min: None,
        klucze_liczba: None,
        standard_dokument_id: None,
        instrukcja_dokument_id: None,
    }
}

/// Lokal: dane lokalu w jednym wierszu, rejestracja z numerem i datą
#[test]
fn flat_data() {
    let room = new_room();
    add_flat(&room, &flat("lokal"));

    let registration_without_date = WpiszDaneLokaluParams {
        rejestracja_numer: Some("REJ/1".to_string()),
        ..flat_data_with_beds(2)
    };
    expect::refused(
        lokal_krotkoterminowy_lokal::wpisz_dane_lokalu(&room, &registration_without_date),
        "rejestracja_numer_z_data",
    );
    lokal_krotkoterminowy_lokal::wpisz_dane_lokalu(&room, &flat_data_with_beds(2))
        .expect("write the data of the flat");
    expect::refused(
        lokal_krotkoterminowy_lokal::wpisz_dane_lokalu(&room, &flat_data_with_beds(3)),
        "wiersza teczki nie zastępuje się nowym wpisem",
    );
    lokal_krotkoterminowy_lokal::zmien_dane_lokalu(
        &room,
        &ZmienDaneLokaluParams {
            rejestracja_numer: Some("REJ/1".to_string()),
            rejestracja_data: Some("2026-12-01".to_string()),
            lozka: 3,
            czas_sprzatania_min: None,
            klucze_liczba: None,
            standard_dokument_id: None,
            instrukcja_dokument_id: None,
        },
    )
    .expect("change the data of the flat");

    let data_of_flat = lokal_krotkoterminowy_lokal::dane_lokalu(&room)
        .expect("read the data of the flat")
        .expect("the flat has its data");
    assert_eq!(data_of_flat.rejestracja_numer.as_deref(), Some("REJ/1"));
}

/// The parameters of the house rules `rules_id` from its received document of the same id, from
/// `od`, with no decision.
fn house_rules(rules_id: &str, od: &str) -> WpiszRegulaminLokaluParams {
    WpiszRegulaminLokaluParams {
        id: rules_id.to_string(),
        dokument_id: rules_id.to_string(),
        od: od.to_string(),
        decyzja_id: None,
    }
}

/// Lokal: każdy regulamin po pierwszym ma zgodę człowieka i się nie zmienia
#[test]
fn house_rules_change_with_consent() {
    let room = new_room();
    for rules_id in ["regulamin-1", "regulamin-2"] {
        add_received_document(
            &room,
            &received_document(rules_id, RodzajDokumentu::Regulamin),
        );
    }
    lokal_krotkoterminowy_lokal::wpisz_regulamin_lokalu(
        &room,
        &house_rules("regulamin-1", "2026-11-05"),
    )
    .expect("write the first house rules");
    let refusal = "każdy regulamin po pierwszym wskazuje zatwierdzoną decyzję inna";

    expect::refused(
        lokal_krotkoterminowy_lokal::wpisz_regulamin_lokalu(
            &room,
            &house_rules("regulamin-2", "2026-12-01"),
        ),
        refusal,
    );
    approved_decision_about(
        &room,
        "inny-wiersz",
        RodzajDecyzji::Inna,
        "lokal_krotkoterminowy_lokal_regulaminy",
        "regulamin-1",
    );
    let by_decision_about_other_row = WpiszRegulaminLokaluParams {
        decyzja_id: Some("inny-wiersz".to_string()),
        ..house_rules("regulamin-2", "2026-12-01")
    };
    expect::refused(
        lokal_krotkoterminowy_lokal::wpisz_regulamin_lokalu(&room, &by_decision_about_other_row),
        refusal,
    );

    approved_decision_about(
        &room,
        "zmiana",
        RodzajDecyzji::Inna,
        "lokal_krotkoterminowy_lokal_regulaminy",
        "regulamin-2",
    );
    lokal_krotkoterminowy_lokal::wpisz_regulamin_lokalu(
        &room,
        &WpiszRegulaminLokaluParams {
            decyzja_id: Some("zmiana".to_string()),
            ..house_rules("regulamin-2", "2026-12-01")
        },
    )
    .expect("write the second house rules");
    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_lokal_regulaminy SET od = '2026-12-02'",
            &[],
        ),
        "regulaminu się nie zmienia",
    );
}

/// The parameters of the kettle for 120 zł bought on `od` with the consent `zgoda_id`.
fn kettle(od: Option<&str>, zgoda_id: Option<&str>) -> WpiszWyposazenieLokaluParams {
    WpiszWyposazenieLokaluParams {
        rzecz: "czajnik".to_string(),
        od: od.map(str::to_string),
        koszt: Some(12_000),
        zgoda_id: zgoda_id.map(str::to_string),
    }
}

/// The parameters of withdrawing the equipment `equipment_id` on `do_dnia` for `powod`.
fn withdrawal(equipment_id: i64, do_dnia: &str, powod: &str) -> WycofajWyposazenieLokaluParams {
    WycofajWyposazenieLokaluParams {
        do_dnia: do_dnia.to_string(),
        powod: powod.to_string(),
        id: equipment_id,
    }
}

/// Lokal: wyposażenie z kosztem ma datę, zgoda to zgoda_na_wyposazenie, wycofanie raz
#[test]
fn equipment() {
    let room = new_room();
    add_owner(&room, &owner());
    add_consent(
        &room,
        &consent("naprawa", "wlascicielka", "zgoda_na_naprawe"),
    );
    add_consent(
        &room,
        &consent("wyposazenie", "wlascicielka", "zgoda_na_wyposazenie"),
    );

    expect::refused(
        lokal_krotkoterminowy_lokal::wpisz_wyposazenie_lokalu(&room, &kettle(None, None)),
        "koszt_z_data",
    );
    expect::refused(
        lokal_krotkoterminowy_lokal::wpisz_wyposazenie_lokalu(
            &room,
            &kettle(Some("2026-11-05"), Some("naprawa")),
        ),
        "wyposażenie wskazuje zgodę zgoda_na_wyposazenie",
    );
    lokal_krotkoterminowy_lokal::wpisz_wyposazenie_lokalu(
        &room,
        &kettle(Some("2026-11-05"), Some("wyposazenie")),
    )
    .expect("write the kettle");

    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_lokal_wyposazenie SET koszt = 9000",
            &[],
        ),
        "wyposażenia się nie zmienia",
    );
    expect::refused(
        lokal_krotkoterminowy_lokal::wycofaj_wyposazenie_lokalu(
            &room,
            &withdrawal(1, "2026-11-01", "zepsuty"),
        ),
        "wycofanie_nie_przed_zakupem",
    );
    lokal_krotkoterminowy_lokal::wycofaj_wyposazenie_lokalu(
        &room,
        &withdrawal(1, "2027-01-10", "zepsuty"),
    )
    .expect("withdraw the kettle");
    expect::refused(
        lokal_krotkoterminowy_lokal::wycofaj_wyposazenie_lokalu(
            &room,
            &withdrawal(1, "2027-01-11", "zgubiony"),
        ),
        "wyposażenia się nie zmienia",
    );
}

/// The parameters of the check-up of the carbon monoxide detector on `data`, next due on
/// `nastepny`, with no document.
fn detector_checkup(data: &str, nastepny: &str) -> WpiszPrzegladLokaluParams {
    WpiszPrzegladLokaluParams {
        rodzaj: "czujnik_co".to_string(),
        data: data.to_string(),
        nastepny: Some(nastepny.to_string()),
        dokument_id: None,
    }
}

/// The parameters of the association's resolution of 1 October „Zakaz”, learnt of on
/// `dowiedziano`, with no document.
fn ban_resolution(dowiedziano: &str) -> WpiszUchwaleWspolnotyParams {
    WpiszUchwaleWspolnotyParams {
        data: "2026-10-01".to_string(),
        tresc: "Zakaz".to_string(),
        dokument_id: None,
        dowiedziano: dowiedziano.to_string(),
    }
}

/// Lokal: przegląd ma następny termin po sobie, uchwała jest znana po podjęciu; obu się nie zmienia
#[test]
fn checkups_and_resolutions() {
    let room = new_room();

    expect::refused(
        lokal_krotkoterminowy_lokal::wpisz_przeglad_lokalu(
            &room,
            &detector_checkup("2026-11-05", "2026-11-05"),
        ),
        "nastepny_po_przegladzie",
    );
    lokal_krotkoterminowy_lokal::wpisz_przeglad_lokalu(
        &room,
        &detector_checkup("2026-11-05", "2027-11-05"),
    )
    .expect("write the check-up");
    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_lokal_przeglady SET nastepny = '2028-11-05'",
            &[],
        ),
        "przeglądu się nie zmienia",
    );

    expect::refused(
        lokal_krotkoterminowy_lokal::wpisz_uchwale_wspolnoty(&room, &ban_resolution("2026-09-30")),
        "dowiedziano_po_podjeciu",
    );
    lokal_krotkoterminowy_lokal::wpisz_uchwale_wspolnoty(&room, &ban_resolution("2026-11-05"))
        .expect("write the resolution");
    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_lokal_uchwaly SET tresc = 'Zgoda'",
            &[],
        ),
        "uchwały się nie zmienia",
    );
}

// Stays -------------------------------------------------------------------------------------------

/// Pobyty: rezerwacja przy zawartej umowie, nie przed praca_od konsumenta
#[test]
fn booking_after_work_start() {
    let room = new_room();
    add_owner(
        &room,
        &WpiszUczestnikaParams {
            konsument: true,
            ..owner()
        },
    );
    add_flat(&room, &flat("lokal"));
    sign_management_contract_at_a_distance(&room);
    established_terms(&room, &terms("warunki"));
    draft_offer(&room, &airbnb_offer());
    // The publication waits for the end of the withdrawal period, the day after WITHDRAWAL_ENDS.
    let early_publication = OpublikujOferteWKanaleParams {
        numer_oferty: "nr-1".to_string(),
        ..publication("oferta-airbnb")
    };
    expect::refused(
        lokal_krotkoterminowy_kalendarz::opublikuj_oferte_w_kanale(&room, &early_publication),
        "ofertę publikuje się przy zawartej umowie o zarządzanie, nie wcześniej niż praca_od",
    );
    publish(
        &room,
        &OpublikujOferteWKanaleParams {
            opublikowano: "2026-11-19T08:00:00Z".to_string(),
            ..publication("oferta-airbnb")
        },
    );
    add_guest(&room);

    // A booking comes after the publication, so the publication already keeps praca_od.
    book(
        &room,
        &PrzyjmijRezerwacjeParams {
            przyjeto: "2026-11-19T09:00:00Z".to_string(),
            ..booking("pobyt", "2026-12-10", "2026-12-13")
        },
    );

    end_contract(&room, "umowa", "wypowiedzenie");
    let booking_after_end = PrzyjmijRezerwacjeParams {
        id: "po-koncu".to_string(),
        numer_rezerwacji: "HM-2".to_string(),
        przyjeto: "2026-11-20T09:00:00Z".to_string(),
        ..booking_fields()
    };
    expect::refused(
        lokal_krotkoterminowy_pobyty::przyjmij_rezerwacje(&room, &booking_after_end),
        "rezerwację przyjmuje się przy zawartej umowie o zarządzanie",
    );
}

/// Pobyty: rezerwacja z kanału z opublikowaną ofertą, nie sprzed publikacji
#[test]
fn booking_from_channel_with_offer() {
    let room = new_room();
    managed_flat(&room);
    add_guest(&room);
    let refusal = "rezerwacja przychodzi z kanału, w którym lokal ma opublikowaną ofertę";

    let from_channel_without_offer = PrzyjmijRezerwacjeParams {
        kanal: "booking".to_string(),
        ..booking_fields()
    };
    expect::refused(
        lokal_krotkoterminowy_pobyty::przyjmij_rezerwacje(&room, &from_channel_without_offer),
        refusal,
    );
    let before_publication = PrzyjmijRezerwacjeParams {
        przyjeto: "2026-11-06T09:00:00Z".to_string(),
        ..booking_fields()
    };
    expect::refused(
        lokal_krotkoterminowy_pobyty::przyjmij_rezerwacje(&room, &before_publication),
        refusal,
    );
    book(&room, &booking("pobyt", "2026-12-10", "2026-12-13"));
}

/// Pobyty: liczba gości do największej z umowy, gość z rolą gosc, wyjazd po przyjeździe
#[test]
fn booking_guests() {
    let room = new_room();
    managed_flat(&room);
    add_person(
        &room,
        &WpiszUczestnikaParams {
            imiona: Some("Ewa".to_string()),
            nazwisko: Some("Lis".to_string()),
            ..person("bez-roli")
        },
    );
    add_guest(&room);
    let too_many_guests = "liczba gości nie przekracza największej liczby gości";

    let five_guests = PrzyjmijRezerwacjeParams {
        gosci: 5,
        ..booking_fields()
    };
    expect::refused(
        lokal_krotkoterminowy_pobyty::przyjmij_rezerwacje(&room, &five_guests),
        too_many_guests,
    );
    let guest_without_role = PrzyjmijRezerwacjeParams {
        gosc_uczestnik_id: "bez-roli".to_string(),
        ..booking_fields()
    };
    expect::refused(
        lokal_krotkoterminowy_pobyty::przyjmij_rezerwacje(&room, &guest_without_role),
        "pobyt wskazuje uczestnika z rolą gosc",
    );
    let leaving_on_arrival = PrzyjmijRezerwacjeParams {
        wyjazd: "2026-12-10".to_string(),
        ..booking_fields()
    };
    expect::refused(
        lokal_krotkoterminowy_pobyty::przyjmij_rezerwacje(&room, &leaving_on_arrival),
        "wyjazd_po_przyjezdzie",
    );
    book(&room, &booking("pobyt", "2026-12-10", "2026-12-13"));

    let rebooking_for_five = ZmienRezerwacjeParams {
        gosci: 5,
        ..rebooking("pobyt", "2026-12-10", "2026-12-13")
    };
    expect::refused(
        lokal_krotkoterminowy_pobyty::zmien_rezerwacje(&room, &rebooking_for_five),
        too_many_guests,
    );
    lokal_krotkoterminowy_pobyty::zmien_rezerwacje(
        &room,
        &ZmienRezerwacjeParams {
            gosci: 4,
            kwota_goscia: 400_000,
            ..rebooking("pobyt", "2026-12-10", "2026-12-14")
        },
    )
    .expect("change the booking");
    let second_booking = PrzyjmijRezerwacjeParams {
        id: "drugi".to_string(),
        ..booking_fields()
    };
    expect::refused(
        lokal_krotkoterminowy_pobyty::przyjmij_rezerwacje(&room, &second_booking),
        "wiersza teczki nie zastępuje się nowym wpisem",
    );
}

/// Pobyty: noce po dacie końca umowy wymagają zgody człowieka, także po zmianie terminu
#[test]
fn booking_after_contract_end() {
    let room = new_room();
    add_owner(&room, &owner());
    add_flat(&room, &flat("lokal"));
    sign_management_contract(&room, Some(CONTRACT_ENDS));
    established_terms(&room, &terms("warunki"));
    draft_offer(&room, &airbnb_offer());
    publish(&room, &publication("oferta-airbnb"));
    add_guest(&room);
    let refusal = "pobyt z nocami po dacie końca umowy wskazuje zatwierdzoną decyzję inna";

    // The last night of 30 January to 1 February is 31 January, the end date itself.
    book(&room, &booking("na-koniec", "2027-01-30", "2027-02-01"));
    let past_the_end = PrzyjmijRezerwacjeParams {
        id: "po-koncu".to_string(),
        numer_rezerwacji: "HM-2".to_string(),
        przyjazd: "2027-01-31".to_string(),
        wyjazd: "2027-02-02".to_string(),
        ..booking_fields()
    };
    expect::refused(
        lokal_krotkoterminowy_pobyty::przyjmij_rezerwacje(&room, &past_the_end),
        refusal,
    );
    expect::refused(
        lokal_krotkoterminowy_pobyty::zmien_rezerwacje(
            &room,
            &rebooking("na-koniec", "2027-01-30", "2027-02-02"),
        ),
        refusal,
    );

    approved_decision_about(
        &room,
        "po-koncu-decyzja",
        RodzajDecyzji::Inna,
        "lokal_krotkoterminowy_pobyty_wpisy",
        "po-koncu",
    );
    book(
        &room,
        &PrzyjmijRezerwacjeParams {
            numer_rezerwacji: "HM-2".to_string(),
            po_koncu_decyzja_id: Some("po-koncu-decyzja".to_string()),
            ..booking("po-koncu", "2027-01-31", "2027-02-02")
        },
    );
}

/// Pobyty: przebieg pobytu, instrukcja tylko do opłaconej, rozliczenie raz
#[test]
fn stay_course() {
    let room = new_room();
    booked_flat(&room);

    expect::refused(
        lokal_krotkoterminowy_pobyty::odnotuj_instrukcje_wejscia(
            &room,
            &entry_instruction("pobyt", "2026-12-10T10:00:00Z"),
        ),
        "instrukcja_tylko_do_oplaconej",
    );
    expect::refused(
        lokal_krotkoterminowy_pobyty::zakoncz_pobyt(&room, &stay_end("pobyt")),
        "pobyt przechodzi",
    );
    expect::refused(
        room.execute(
            "INSERT INTO lokal_krotkoterminowy_pobyty_wpisy (id, kanal, numer_rezerwacji,
              przyjeto, przyjazd, wyjazd, gosci, gosc_uczestnik_id, kwota_goscia,
              oplata_sprzatanie, oplacona, stan)
            VALUES ('trwa', 'airbnb', 'HM-3', '2026-11-08T10:00:00Z', '2026-12-20', '2026-12-22',
              2, 'gosc', 1, 1, 1, 'trwa')",
            &[],
        ),
        "pobyt powstaje potwierdzony",
    );

    record_stay_paid(&room, "pobyt");
    lokal_krotkoterminowy_pobyty::odnotuj_instrukcje_wejscia(
        &room,
        &entry_instruction("pobyt", "2026-12-10T10:00:00Z"),
    )
    .expect("send the entry instruction");
    expect::refused(
        lokal_krotkoterminowy_pobyty::odnotuj_instrukcje_wejscia(
            &room,
            &entry_instruction("pobyt", "2026-12-10T11:00:00Z"),
        ),
        "instrukcję wejścia i rozliczenie pobytu wpisuje się raz",
    );
    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_pobyty_wpisy SET kanal = 'booking'",
            &[],
        ),
        "kanał, numer rezerwacji, chwila przyjęcia i gość się nie zmieniają",
    );

    set_stay_state(&room, "pobyt", StanPobytu::Trwa);
    lokal_krotkoterminowy_pobyty::zakoncz_pobyt(&room, &stay_end("pobyt")).expect("end the stay");
    expect::refused(
        lokal_krotkoterminowy_pobyty::zmien_rezerwacje(
            &room,
            &rebooking("pobyt", "2026-12-10", "2026-12-14"),
        ),
        "termin, goście, opłacenie i instrukcja zakończonego albo odwołanego pobytu się nie zmieniają",
    );

    // A writer outside the client can leave out the channel's payout and its date that
    // `odnotuj_rozliczenie_pobytu` requires. The table refuses the settlement without them.
    let settlement_without_payout = room.execute(
        "UPDATE lokal_krotkoterminowy_pobyty_wpisy
        SET prowizja_kanalu = :prowizja_kanalu,
          wyplata_kanalu = :wyplata_kanalu,
          wyplacono = :wyplacono,
          rozliczono = :rozliczono
        WHERE id = :id",
        &[
            ("prowizja_kanalu", SqlValue::Integer(45_000)),
            ("rozliczono", text("2026-12-15T09:00:00Z")),
            ("id", text("pobyt")),
        ],
    );
    expect::refused(settlement_without_payout, "rozliczenie_w_komplecie");
    settle_stay(&room);
    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_pobyty_wpisy SET wyplata_kanalu = 1",
            &[],
        ),
        "instrukcję wejścia i rozliczenie pobytu wpisuje się raz",
    );
}

/// The parameters of the guest's report of a smell of gas in the stay „pobyt”, a danger, with no
/// notification and not passed on yet.
fn gas_report() -> WpiszZgloszenieGosciaParams {
    WpiszZgloszenieGosciaParams {
        id: "zgloszenie".to_string(),
        pobyt_id: "pobyt".to_string(),
        chwila: "2026-12-11T20:00:00Z".to_string(),
        opis: "zapach gazu".to_string(),
        przekazano: None,
        zagrozenie: true,
        zawiadomiono: None,
    }
}

/// The parameters of recording that the report „zgloszenie” was passed on to `przekazano`.
fn report_passed_on(przekazano: &str) -> OdnotujPrzekazanieZgloszeniaParams {
    OdnotujPrzekazanieZgloszeniaParams {
        przekazano: przekazano.to_string(),
        id: "zgloszenie".to_string(),
    }
}

/// Pobyty: zgłoszenie zagrożenia ma zawiadomienie, a przekazanie wpisuje się raz
#[test]
fn guest_reports() {
    let room = new_room();
    booked_flat(&room);

    expect::refused(
        lokal_krotkoterminowy_pobyty::wpisz_zgloszenie_goscia(&room, &gas_report()),
        "zagrozenie_ma_zawiadomienie",
    );
    lokal_krotkoterminowy_pobyty::wpisz_zgloszenie_goscia(
        &room,
        &WpiszZgloszenieGosciaParams {
            zawiadomiono: Some(
                "gość zadzwonił pod 112; zawiadomiony człowiek prowadzący lokal".to_string(),
            ),
            ..gas_report()
        },
    )
    .expect("write the guest's report");
    lokal_krotkoterminowy_pobyty::odnotuj_przekazanie_zgloszenia(
        &room,
        &report_passed_on("Koordynator operacji, 20:02"),
    )
    .expect("pass the report on");

    expect::refused(
        lokal_krotkoterminowy_pobyty::odnotuj_przekazanie_zgloszenia(
            &room,
            &report_passed_on("Zarządca"),
        ),
        "zgłoszenia się nie zmienia",
    );
    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_pobyty_zgloszenia SET opis = 'nic'",
            &[],
        ),
        "zgłoszenia się nie zmienia",
    );
    lokal_krotkoterminowy_pobyty::wpisz_zmiane_pobytu(
        &room,
        &WpiszZmianePobytuParams {
            pobyt_id: "pobyt".to_string(),
            chwila: "2026-11-20T10:00:00Z".to_string(),
            co: CoSieZmienia::Cena,
            przyczyna: "rabat".to_string(),
            zdecydowal: "agent-zarzadcy".to_string(),
        },
    )
    .expect("write the change of the stay");
    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_pobyty_zmiany SET przyczyna = 'inna'",
            &[],
        ),
        "zmiany pobytu się nie zmienia",
    );
}

/// Pobyty: wyjątek, rekompensata, roszczenie i zwrot ponad zasady kanału mają zgodę człowieka
#[test]
fn stay_exceptions_and_money() {
    let room = new_room();
    booked_flat(&room);
    let refusal = "rekompensata, roszczenie, wniosek kanału i zwrot ponad zasady kanału wskazują zatwierdzoną decyzję";

    // A writer outside the client can leave out the decision that `wpisz_wyjatek_pobytu`
    // requires. The table refuses the exception without one.
    let exception_without_decision = room.execute(
        "INSERT INTO lokal_krotkoterminowy_pobyty_wyjatki (id, pobyt_id, opis, decyzja_id)
        VALUES (:id, :pobyt_id, :opis, :decyzja_id)",
        &[
            ("id", text("zwierze")),
            ("pobyt_id", text("pobyt")),
            ("opis", text("pies")),
        ],
    );
    expect::refused(
        exception_without_decision,
        "wyjątek od regulaminu wskazuje zatwierdzoną decyzję inna",
    );
    approved_decision_about(
        &room,
        "pies",
        RodzajDecyzji::Inna,
        "lokal_krotkoterminowy_pobyty_wyjatki",
        "zwierze",
    );
    lokal_krotkoterminowy_pobyty::wpisz_wyjatek_pobytu(
        &room,
        &WpiszWyjatekPobytuParams {
            id: "zwierze".to_string(),
            pobyt_id: "pobyt".to_string(),
            opis: "pies".to_string(),
            decyzja_id: "pies".to_string(),
        },
    )
    .expect("write the exception of the stay");
    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_pobyty_wyjatki SET opis = 'dwa psy'",
            &[],
        ),
        "wyjątku się nie zmienia",
    );

    let kinds_needing_decision = [
        ("rekompensata", RodzajPieniedzy::Rekompensata),
        ("roszczenie", RodzajPieniedzy::Roszczenie),
        ("wniosek_kanalu", RodzajPieniedzy::WniosekKanalu),
        ("zwrot", RodzajPieniedzy::Zwrot),
    ];
    for (money_id, rodzaj) in kinds_needing_decision {
        expect::refused(
            lokal_krotkoterminowy_pobyty::wpisz_pieniadze_pobytu(
                &room,
                &stay_money(money_id, rodzaj),
            ),
            refusal,
        );
    }
    let recovery_by_channel_rules = WpiszPieniadzePobytuParams {
        wedlug_zasad_kanalu: true,
        ..stay_money("r", RodzajPieniedzy::Odzysk)
    };
    expect::refused(
        lokal_krotkoterminowy_pobyty::wpisz_pieniadze_pobytu(&room, &recovery_by_channel_rules),
        "wedlug_zasad_kanalu_tylko_zwrot",
    );

    lokal_krotkoterminowy_pobyty::wpisz_pieniadze_pobytu(
        &room,
        &WpiszPieniadzePobytuParams {
            wedlug_zasad_kanalu: true,
            ..stay_money("zwrot", RodzajPieniedzy::Zwrot)
        },
    )
    .expect("write the refund by the channel's rules");
    lokal_krotkoterminowy_pobyty::wpisz_pieniadze_pobytu(
        &room,
        &stay_money("odzysk", RodzajPieniedzy::Odzysk),
    )
    .expect("write the recovery");
    approved_decision_about(
        &room,
        "szkoda",
        RodzajDecyzji::Inna,
        "lokal_krotkoterminowy_pobyty_pieniadze",
        "roszczenie",
    );
    lokal_krotkoterminowy_pobyty::wpisz_pieniadze_pobytu(
        &room,
        &WpiszPieniadzePobytuParams {
            decyzja_id: Some("szkoda".to_string()),
            ..stay_money("roszczenie", RodzajPieniedzy::Roszczenie)
        },
    )
    .expect("write the claim");
    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_pobyty_pieniadze SET kwota = 1",
            &[],
        ),
        "pieniędzy pobytu się nie zmienia",
    );
}

/// The parameters of sending the letter `letter_id` on 15 December at 10:00, with no decision.
fn sending_on_15_december(letter_id: &str) -> OdnotujWyslanieParams {
    OdnotujWyslanieParams {
        wyslano: "2026-12-15T10:00:00Z".to_string(),
        ..sending(letter_id)
    }
}

/// Pobyty: odpowiedź na opinię 3 i niżej oraz na skargę czeka na zgodę; wskazuje się ją przed wysłaniem
#[test]
fn answers_with_consent() {
    let room = new_room();
    booked_flat(&room);
    lokal_krotkoterminowy_pobyty::wpisz_opinie_goscia(
        &room,
        &review("slaba", "pobyt", 3, "2026-12-14T09:00:00Z"),
    )
    .expect("write the low review");
    lokal_krotkoterminowy_pobyty::wpisz_skarge(
        &room,
        &WpiszSkargeParams {
            chwila: "2026-12-12T23:30:00Z".to_string(),
            kto: "sąsiad z lokalu 4".to_string(),
            tresc: "hałas".to_string(),
            pobyt_id: Some("pobyt".to_string()),
        },
    )
    .expect("write the complaint");
    let refusal = "odpowiedź na opinię z oceną 3 albo niższą i odpowiedź na skargę przed wysłaniem czekają na zgodę";

    letter_to_guest(&room, "na-slaba");
    lokal_krotkoterminowy_pobyty::wskaz_odpowiedz_na_opinie(
        &room,
        &review_answer("slaba", "na-slaba"),
    )
    .expect("name the answer to the low review");
    expect::refused(
        sprawa_korespondencja::odnotuj_wyslanie(&room, &sending_on_15_december("na-slaba")),
        refusal,
    );
    send_with_consent(&room, "na-slaba", "2026-12-15");

    prepare_letter(&room, &letter_to_neighbour("na-skarge"));
    lokal_krotkoterminowy_pobyty::wskaz_odpowiedz_na_skarge(
        &room,
        &WskazOdpowiedzNaSkargeParams {
            odpowiedz_wysylka_id: "na-skarge".to_string(),
            id: 1,
        },
    )
    .expect("name the answer to the complaint");
    expect::refused(
        sprawa_korespondencja::odnotuj_wyslanie(&room, &sending_on_15_december("na-skarge")),
        refusal,
    );

    // A good review gets its answer without consent; the answer is named before it goes out.
    book(
        &room,
        &PrzyjmijRezerwacjeParams {
            numer_rezerwacji: "HM-2".to_string(),
            ..booking("drugi", "2026-12-20", "2026-12-22")
        },
    );
    lokal_krotkoterminowy_pobyty::wpisz_opinie_goscia(
        &room,
        &review("dobra", "drugi", 5, "2026-12-23T09:00:00Z"),
    )
    .expect("write the good review");
    letter_to_guest(&room, "na-dobra");
    send(&room, &sending("na-dobra"));
    expect::refused(
        lokal_krotkoterminowy_pobyty::wskaz_odpowiedz_na_opinie(
            &room,
            &review_answer("dobra", "na-dobra"),
        ),
        "odpowiedź na opinię albo skargę wskazuje wysyłkę, zanim wyjdzie",
    );
    letter_to_guest(&room, "na-dobra-2");
    lokal_krotkoterminowy_pobyty::wskaz_odpowiedz_na_opinie(
        &room,
        &review_answer("dobra", "na-dobra-2"),
    )
    .expect("name the answer to the good review");
    send(&room, &sending("na-dobra-2"));
    expect::refused(
        lokal_krotkoterminowy_pobyty::wskaz_odpowiedz_na_opinie(
            &room,
            &review_answer("dobra", "na-slaba"),
        ),
        "odpowiedź wskazuje się raz",
    );
}

/// Pobyty: ostatnie rozliczenie dopiero, gdy żaden pobyt nie czeka ani nie trwa
#[test]
fn last_settlement_after_last_stay() {
    let room = new_room();
    offering_flat(&room, Some(CONTRACT_ENDS));
    established_terms(&room, &terms("warunki"));
    draft_offer(&room, &airbnb_offer());
    publish(&room, &publication("oferta-airbnb"));
    add_guest(&room);
    book(&room, &booking("pobyt", "2026-12-10", "2026-12-13"));
    stage(&room, "prowadzenie_lokalu", "2026-11-07T09:00:00Z");
    stage(&room, "koniec_umowy", "2027-01-10T09:00:00Z");
    stage(&room, "zamkniecie_lokalu", "2027-02-01T09:00:00Z");
    set_offer_state(&room, "oferta-airbnb", StanOferty::Zamknieta);
    signed_protocol(
        &room,
        "zwrot",
        RodzajProtokolu::Zwrot,
        "2027-02-01T10:00:00Z",
    );

    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change("ostatnie_rozliczenie", "2027-02-02T09:00:00Z"),
        ),
        "etap ostatnie_rozliczenie przychodzi, gdy żaden pobyt nie jest potwierdzony",
    );

    finished_stay(&room);
    stage(&room, "ostatnie_rozliczenie", "2027-02-02T09:00:00Z");
}

// Orders and access -------------------------------------------------------------------------------

/// [`order`] `order_id` of a contractor outside the register, at the price of a quote.
fn order_outside_register(order_id: &str) -> ZlecPraceParams {
    ZlecPraceParams {
        spoza_rejestru: true,
        cennik_wersja_odnosnik: None,
        cena_zrodlo: ZrodloCeny::Wycena,
        ..order(order_id)
    }
}

/// Zlecenia: spoza rejestru, poza standardem i wymiana zamka mają zgodę człowieka
#[test]
fn orders_with_consent() {
    let room = new_room();
    booked_flat(&room);
    let refusal = "zlecenie spoza rejestru, poza standardem, wymiana zamka i wycena ponad kwotę napraw wskazują";

    let orders_needing_decision = [
        order_outside_register("spoza"),
        ZlecPraceParams {
            poza_standardem: true,
            ..order("poza-standardem")
        },
        ZlecPraceParams {
            rodzaj: RodzajZlecenia::WymianaZamka,
            ..order("zamek")
        },
    ];
    for order_needing_decision in &orders_needing_decision {
        expect::refused(
            lokal_krotkoterminowy_zlecenia::zlec_prace(&room, order_needing_decision),
            refusal,
        );
    }
    let without_price_list = ZlecPraceParams {
        cennik_wersja_odnosnik: None,
        ..order("bez-cennika")
    };
    expect::refused(
        lokal_krotkoterminowy_zlecenia::zlec_prace(&room, &without_price_list),
        "cena_z_cennika_rejestru",
    );

    // A decision about another order does not cover this one.
    approved_decision_about(
        &room,
        "decyzja-zamek",
        RodzajDecyzji::Inna,
        "lokal_krotkoterminowy_zlecenia_wpisy",
        "zamek",
    );
    let by_decision_about_lock = ZlecPraceParams {
        decyzja_id: Some("decyzja-zamek".to_string()),
        ..order_outside_register("spoza")
    };
    expect::refused(
        lokal_krotkoterminowy_zlecenia::zlec_prace(&room, &by_decision_about_lock),
        refusal,
    );
    place_order(
        &room,
        &ZlecPraceParams {
            rodzaj: RodzajZlecenia::WymianaZamka,
            decyzja_id: Some("decyzja-zamek".to_string()),
            ..order("zamek")
        },
    );
    approved_decision_about(
        &room,
        "decyzja-spoza",
        RodzajDecyzji::Inna,
        "lokal_krotkoterminowy_zlecenia_wpisy",
        "spoza",
    );
    place_order(
        &room,
        &ZlecPraceParams {
            decyzja_id: Some("decyzja-spoza".to_string()),
            ..order_outside_register("spoza")
        },
    );
    place_order(&room, &order("sprzatanie"));
}

/// [`order`] `order_id` of a repair to keep the standard of the flat, for `cena` from a quote,
/// of no stay.
fn repair(order_id: &str, cena: i64) -> ZlecPraceParams {
    ZlecPraceParams {
        rodzaj: RodzajZlecenia::Naprawa,
        przyczyna: PrzyczynaZlecenia::StandardLokalu,
        pobyt_id: None,
        cennik_wersja_odnosnik: None,
        cena,
        cena_zrodlo: ZrodloCeny::Wycena,
        ..order(order_id)
    }
}

/// The parameters of the owner's consent `consent_id` to the repair of the order `order_id`.
fn repair_consent(consent_id: &str, tresc: &str, order_id: &str) -> WpiszZgodeParams {
    WpiszZgodeParams {
        tresc: tresc.to_string(),
        dotyczy_tabela: Some("lokal_krotkoterminowy_zlecenia_wpisy".to_string()),
        dotyczy_id: Some(order_id.to_string()),
        ..consent(consent_id, "wlascicielka", "zgoda_na_naprawe")
    }
}

/// Zlecenia: wycena ponad kwotę napraw ma zgodę właściciela i człowieka
#[test]
fn quote_above_repair_amount() {
    let room = new_room();
    booked_flat(&room);
    let refusal = "cena z wyceny ponad kwotę napraw wskazuje zgodę zgoda_na_naprawe właściciela";

    place_order(&room, &repair("w-kwocie", 50_000));

    approved_decision_about(
        &room,
        "decyzja-boiler",
        RodzajDecyzji::Inna,
        "lokal_krotkoterminowy_zlecenia_wpisy",
        "boiler",
    );
    let without_consent = ZlecPraceParams {
        decyzja_id: Some("decyzja-boiler".to_string()),
        ..repair("boiler", 80_000)
    };
    expect::refused(
        lokal_krotkoterminowy_zlecenia::zlec_prace(&room, &without_consent),
        refusal,
    );
    add_consent(
        &room,
        &repair_consent("zgoda-inna", "Naprawa pralki", "pralka"),
    );
    let with_consent_about_other_order = ZlecPraceParams {
        zgoda_id: Some("zgoda-inna".to_string()),
        ..without_consent.clone()
    };
    expect::refused(
        lokal_krotkoterminowy_zlecenia::zlec_prace(&room, &with_consent_about_other_order),
        refusal,
    );
    add_consent(
        &room,
        &repair_consent("zgoda-boiler", "Wymiana bojlera do 800 zł", "boiler"),
    );
    let without_decision = ZlecPraceParams {
        zgoda_id: Some("zgoda-boiler".to_string()),
        ..repair("boiler", 80_000)
    };
    expect::refused(
        lokal_krotkoterminowy_zlecenia::zlec_prace(&room, &without_decision),
        "wycena ponad kwotę napraw wskazują zatwierdzoną decyzję",
    );

    place_order(
        &room,
        &ZlecPraceParams {
            zgoda_id: Some("zgoda-boiler".to_string()),
            decyzja_id: Some("decyzja-boiler".to_string()),
            ..repair("boiler", 80_000)
        },
    );
}

/// Zlecenia: koszt po potwierdzeniu, inny niż cena z osobną zgodą, wpisany raz
#[test]
fn order_cost() {
    let room = new_room();
    booked_flat(&room);
    place_order(&room, &order("sprzatanie"));
    let other_cost_needs_decision = "koszt inny niż cena wskazuje zatwierdzoną decyzję";

    expect::refused(
        lokal_krotkoterminowy_zlecenia::wpisz_koszt_zlecenia(
            &room,
            &cost_of_order("sprzatanie", 15_000),
        ),
        "koszt_po_potwierdzeniu",
    );
    // A writer outside the client can leave out the report that `potwierdz_wykonanie_zlecenia`
    // requires. The table refuses the confirmation without one.
    let confirmation_without_report = room.execute(
        "UPDATE lokal_krotkoterminowy_zlecenia_wpisy
        SET stan = 'potwierdzone',
          raport_sciezka = :raport_sciezka,
          raport_skrot = :raport_skrot,
          potwierdzil = :potwierdzil,
          potwierdzono = :potwierdzono
        WHERE id = :id",
        &[
            ("potwierdzil", text("agent")),
            ("potwierdzono", text("2026-12-13T16:00:00Z")),
            ("id", text("sprzatanie")),
        ],
    );
    expect::refused(confirmation_without_report, "potwierdzone_z_raportem");
    confirm_order(&room, "sprzatanie", "2026-12-13T16:00:00Z");
    expect::refused(
        lokal_krotkoterminowy_zlecenia::wpisz_koszt_zlecenia(
            &room,
            &cost_of_order("sprzatanie", 18_000),
        ),
        other_cost_needs_decision,
    );

    // A decision asked before the work was confirmed is about something else.
    approved_decision(
        &room,
        &PoprosODecyzjeParams {
            zlozono: "2026-12-13T15:00:00Z".to_string(),
            ..decision_request_about(
                "za-wczesna",
                RodzajDecyzji::Inna,
                "lokal_krotkoterminowy_zlecenia_wpisy",
                "sprzatanie",
            )
        },
        "2026-12-13T15:30:00Z",
    );
    let by_early_decision = WpiszKosztZleceniaParams {
        koszt_decyzja_id: Some("za-wczesna".to_string()),
        ..cost_of_order("sprzatanie", 18_000)
    };
    expect::refused(
        lokal_krotkoterminowy_zlecenia::wpisz_koszt_zlecenia(&room, &by_early_decision),
        other_cost_needs_decision,
    );
    approved_decision(
        &room,
        &PoprosODecyzjeParams {
            zlozono: "2026-12-13T17:00:00Z".to_string(),
            ..decision_request_about(
                "koszt",
                RodzajDecyzji::Inna,
                "lokal_krotkoterminowy_zlecenia_wpisy",
                "sprzatanie",
            )
        },
        "2026-12-13T18:00:00Z",
    );
    record_cost_of_order(
        &room,
        &WpiszKosztZleceniaParams {
            koszt_decyzja_id: Some("koszt".to_string()),
            ..cost_of_order("sprzatanie", 18_000)
        },
    );

    expect::refused(
        lokal_krotkoterminowy_zlecenia::wpisz_koszt_zlecenia(
            &room,
            &cost_of_order("sprzatanie", 15_000),
        ),
        "potwierdzenie i koszt zlecenia wpisuje się raz",
    );
    expect::refused(
        lokal_krotkoterminowy_zlecenia::zmien_stan_zlecenia(
            &room,
            &order_state("sprzatanie", StanZlecenia::Anulowane),
        ),
        "zlecenie przechodzi",
    );
}

/// Zlecenia: wstrzymane wraca po decyzji, anulowane jest ostateczne, treść się nie zmienia
#[test]
fn order_states() {
    let room = new_room();
    booked_flat(&room);
    place_order(&room, &order("sprzatanie"));

    set_order_state(&room, "sprzatanie", StanZlecenia::Wstrzymane);
    let confirmation = PotwierdzWykonanieZleceniaParams {
        raport_sciezka: "r.pdf".to_string(),
        raport_skrot: "s".to_string(),
        potwierdzil: "agent".to_string(),
        potwierdzono: "2026-12-13T16:00:00Z".to_string(),
        id: "sprzatanie".to_string(),
    };
    expect::refused(
        lokal_krotkoterminowy_zlecenia::potwierdz_wykonanie_zlecenia(&room, &confirmation),
        "zlecenie przechodzi",
    );
    set_order_state(&room, "sprzatanie", StanZlecenia::Zlecone);
    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_zlecenia_wpisy SET cena = 1",
            &[],
        ),
        "treść zlecenia się nie zmienia",
    );
    let after_departure_of_no_stay = ZlecPraceParams {
        pobyt_id: None,
        ..order("bez-pobytu")
    };
    expect::refused(
        lokal_krotkoterminowy_zlecenia::zlec_prace(&room, &after_departure_of_no_stay),
        "przyczyna_wskazuje_pobyt_albo_zgloszenie",
    );

    set_order_state(&room, "sprzatanie", StanZlecenia::Anulowane);
    expect::refused(
        lokal_krotkoterminowy_zlecenia::zmien_stan_zlecenia(
            &room,
            &order_state("sprzatanie", StanZlecenia::Zlecone),
        ),
        "anulowanego zlecenia się nie zmienia",
    );
}

/// Dostępy: kod dla gościa tylko do opłaconego pobytu i w jego dniach (T7)
#[test]
fn access_for_paid_stay() {
    let room = new_room();
    booked_flat(&room);
    let refusal = "dostęp dla pobytu wymaga opłaconej rezerwacji";

    expect::refused(
        lokal_krotkoterminowy_zlecenia::ustaw_dostep(&room, &access()),
        refusal,
    );
    record_stay_paid(&room, "pobyt");
    let accesses_outside_stay = [
        UstawDostepParams {
            wazny_od: "2026-12-09T15:00:00Z".to_string(),
            ..access()
        },
        UstawDostepParams {
            wazny_do: "2026-12-14T11:00:00Z".to_string(),
            ..access()
        },
    ];
    for access_outside_stay in &accesses_outside_stay {
        expect::refused(
            lokal_krotkoterminowy_zlecenia::ustaw_dostep(&room, access_outside_stay),
            refusal,
        );
    }
    let without_code = UstawDostepParams {
        kod: None,
        ..access()
    };
    expect::refused(
        lokal_krotkoterminowy_zlecenia::ustaw_dostep(&room, &without_code),
        "kod_albo_klucze",
    );
    let for_nobody = UstawDostepParams {
        pobyt_id: None,
        ..access()
    };
    expect::refused(
        lokal_krotkoterminowy_zlecenia::ustaw_dostep(&room, &for_nobody),
        "dostep_dla_pobytu_albo_zlecenia",
    );
    set_access(&room, &access());

    let accesses =
        lokal_krotkoterminowy_zlecenia::dostepy_do_lokalu(&room).expect("read the accesses");
    assert_eq!(accesses[0].kod.as_deref(), Some("4821"));
    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_zlecenia_dostepy SET wazny_do = '2026-12-20T11:00:00Z'",
            &[],
        ),
        "dostępu się nie zmienia",
    );

    // The stay is cancelled before the code went out: the code is not given.
    set_stay_state(&room, "pobyt", StanPobytu::Odwolana);
    expect::refused(
        lokal_krotkoterminowy_zlecenia::odnotuj_wydanie_dostepu(
            &room,
            &access_given(1, "2026-12-10T10:00:00Z"),
        ),
        "kod albo klucze wydaje się tylko dla opłaconego pobytu",
    );
    lokal_krotkoterminowy_zlecenia::odnotuj_odebranie_dostepu(
        &room,
        &access_taken_back(1, "2026-12-09T12:00:00Z"),
    )
    .expect("take the code back");
}

/// Dostępy: kod i klucze dla wykonawcy tylko w oknie zlecenia; wydanie i odebranie raz
#[test]
fn access_for_order() {
    let room = new_room();
    booked_flat(&room);
    place_order(&room, &order("sprzatanie"));
    let keys_for_cleaning = UstawDostepParams {
        pobyt_id: None,
        zlecenie_id: Some("sprzatanie".to_string()),
        rodzaj: RodzajDostepu::Klucze,
        kod: None,
        klucze_liczba: Some(1),
        wazny_od: "2026-12-13T10:00:00Z".to_string(),
        wazny_do: "2026-12-13T15:00:00Z".to_string(),
        kto: "ekipa sprzątająca".to_string(),
        ..access()
    };
    let refusal = "dostęp dla zlecenia mieści się w oknie zlecenia";

    let past_the_window = UstawDostepParams {
        wazny_do: "2026-12-13T16:00:00Z".to_string(),
        ..keys_for_cleaning.clone()
    };
    expect::refused(
        lokal_krotkoterminowy_zlecenia::ustaw_dostep(&room, &past_the_window),
        refusal,
    );
    set_access(&room, &keys_for_cleaning);
    expect::refused(
        lokal_krotkoterminowy_zlecenia::odnotuj_odebranie_dostepu(
            &room,
            &access_taken_back(1, "2026-12-13T15:00:00Z"),
        ),
        "klucze_odebrane_po_wydaniu",
    );
    lokal_krotkoterminowy_zlecenia::odnotuj_wydanie_dostepu(
        &room,
        &access_given(1, "2026-12-13T09:50:00Z"),
    )
    .expect("give the keys");
    expect::refused(
        lokal_krotkoterminowy_zlecenia::odnotuj_wydanie_dostepu(
            &room,
            &access_given(1, "2026-12-13T09:55:00Z"),
        ),
        "dostępu się nie zmienia",
    );
    lokal_krotkoterminowy_zlecenia::odnotuj_odebranie_dostepu(
        &room,
        &access_taken_back(1, "2026-12-13T15:00:00Z"),
    )
    .expect("take the keys back");

    set_order_state(&room, "sprzatanie", StanZlecenia::Wstrzymane);
    expect::refused(
        lokal_krotkoterminowy_zlecenia::ustaw_dostep(&room, &keys_for_cleaning),
        refusal,
    );
}

// Calendar, prices and offers ---------------------------------------------------------------------

/// The parameters of a change of the night `data` to a night of the stay `stay_id` at
/// `zmieniono`.
fn booked_night_change(data: &str, stay_id: &str, zmieniono: &str) -> ZmienNocParams {
    ZmienNocParams {
        pobyt_id: Some(stay_id.to_string()),
        ..night_change(data, StanNocy::Zarezerwowana, zmieniono)
    }
}

/// Kalendarz: noc zarezerwowana leży w dniach pobytu, a dzień wyjazdu jest wolny
#[test]
fn nights_of_a_stay() {
    let room = new_room();
    booked_flat(&room);
    let refusal = "noc zarezerwowana należy do potwierdzonego, trwającego albo zakończonego pobytu";

    expect::refused(
        lokal_krotkoterminowy_kalendarz::wpisz_noc(
            &room,
            &booked_night("2026-12-13", "pobyt", "2026-11-07T10:05:00Z"),
        ),
        refusal,
    );
    let free_night_of_stay = WpiszNocParams {
        pobyt_id: Some("pobyt".to_string()),
        ..night("2026-12-10", StanNocy::Wolna, "2026-11-07T10:05:00Z")
    };
    expect::refused(
        lokal_krotkoterminowy_kalendarz::wpisz_noc(&room, &free_night_of_stay),
        "stan_wskazuje_pobyt_zlecenie_albo_zgode",
    );
    add_night(
        &room,
        &booked_night("2026-12-10", "pobyt", "2026-11-07T10:05:00Z"),
    );
    add_night(
        &room,
        &night("2026-12-13", StanNocy::Wolna, "2026-11-07T10:05:00Z"),
    );
    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_kalendarz_noce SET data = '2026-12-14'
            WHERE data = '2026-12-13'",
            &[],
        ),
        "data nocy się nie zmienia",
    );
}

/// Kalendarz: tej samej nocy nie sprzeda się dwa razy; druga rezerwacja to kolizja
#[test]
fn double_booking_is_a_collision() {
    let room = new_room();
    booked_flat(&room);
    add_night(
        &room,
        &booked_night("2026-12-11", "pobyt", "2026-11-07T10:05:00Z"),
    );
    draft_offer(
        &room,
        &offer("oferta-booking", "booking", KontoOferty::Wlasciciela),
    );
    publish(&room, &publication("oferta-booking"));
    book(
        &room,
        &PrzyjmijRezerwacjeParams {
            kanal: "booking".to_string(),
            numer_rezerwacji: "BK-1".to_string(),
            ..booking("z-bookingu", "2026-12-11", "2026-12-12")
        },
    );

    // The second channel's guest cannot take the night: the row exists and is taken.
    expect::refused(
        lokal_krotkoterminowy_kalendarz::wpisz_noc(
            &room,
            &booked_night("2026-12-11", "z-bookingu", "2026-11-07T11:00:00Z"),
        ),
        "wiersza teczki nie zastępuje się nowym wpisem",
    );
    expect::refused(
        lokal_krotkoterminowy_kalendarz::zmien_noc(
            &room,
            &booked_night_change("2026-12-11", "z-bookingu", "2026-11-07T11:00:00Z"),
        ),
        "zajęta noc najpierw się zwalnia",
    );
    expect::refused(
        lokal_krotkoterminowy_kalendarz::zmien_noc(
            &room,
            &night_change("2026-12-11", StanNocy::Wolna, "2026-11-07T11:00:00Z"),
        ),
        "noc pobytu zwalnia się, gdy pobyt jest odwołany",
    );
    set_stay_state(&room, "z-bookingu", StanPobytu::Kolizja);

    // The human keeps the second stay: the first is cancelled, its night freed and taken.
    set_stay_state(&room, "pobyt", StanPobytu::Odwolana);
    lokal_krotkoterminowy_kalendarz::zmien_noc(
        &room,
        &night_change("2026-12-11", StanNocy::Wolna, "2026-11-07T12:00:00Z"),
    )
    .expect("free the night");
    expect::refused(
        lokal_krotkoterminowy_kalendarz::zmien_noc(
            &room,
            &booked_night_change("2026-12-11", "z-bookingu", "2026-11-07T12:05:00Z"),
        ),
        "noc zarezerwowana należy do potwierdzonego",
    );
    set_stay_state(&room, "z-bookingu", StanPobytu::Potwierdzona);
    lokal_krotkoterminowy_kalendarz::zmien_noc(
        &room,
        &booked_night_change("2026-12-11", "z-bookingu", "2026-11-07T12:05:00Z"),
    )
    .expect("book the night for the second stay");
}

/// The parameters of the owner's night `data` by the consent `zgoda_id`, changed at `zmieniono`.
fn owners_night(data: &str, zgoda_id: &str, zmieniono: &str) -> WpiszNocParams {
    WpiszNocParams {
        zgoda_id: Some(zgoda_id.to_string()),
        ..night(data, StanNocy::Wlasciciel, zmieniono)
    }
}

/// The parameters of the night `data` blocked for the works of the order „naprawa”.
fn works_night(data: &str) -> WpiszNocParams {
    WpiszNocParams {
        zlecenie_id: Some("naprawa".to_string()),
        ..night(data, StanNocy::Prace, "2026-12-01T09:00:00Z")
    }
}

/// The parameters of the owner's consent `consent_id` to her nights on Christmas Eve, given at
/// `zlozono`.
fn christmas_eve_consent(consent_id: &str, zlozono: &str) -> WpiszZgodeParams {
    WpiszZgodeParams {
        tresc: "Wigilia".to_string(),
        zlozono: zlozono.to_string(),
        ..consent(consent_id, "wlascicielka", "noce_wlasciciela")
    }
}

/// Kalendarz: noc na prace w oknie zlecenia, noc właściciela zgłoszona na czas
#[test]
fn blocked_nights() {
    let room = new_room();
    booked_flat(&room);
    place_order(
        &room,
        &ZlecPraceParams {
            przyczyna: PrzyczynaZlecenia::StandardLokalu,
            pobyt_id: None,
            rodzaj: RodzajZlecenia::Naprawa,
            okno_od: "2026-12-20T08:00:00Z".to_string(),
            okno_do: "2026-12-20T18:00:00Z".to_string(),
            ..order("naprawa")
        },
    );

    expect::refused(
        lokal_krotkoterminowy_kalendarz::wpisz_noc(&room, &works_night("2026-12-21")),
        "noc na prace leży w oknie zlecenia",
    );
    add_night(&room, &works_night("2026-12-20"));

    // Terms say 14 days ahead: a request of 15 December is late for the 24th.
    add_consent(
        &room,
        &christmas_eve_consent("spoznione", "2026-12-15T09:00:00Z"),
    );
    add_consent(
        &room,
        &christmas_eve_consent("na-czas", "2026-12-01T09:00:00Z"),
    );
    add_consent(
        &room,
        &consent("inna-zgoda", "wlascicielka", "zgoda_na_naprawe"),
    );
    let refusal = "noc właściciela wskazuje zgodę noce_wlasciciela złożoną z wyprzedzeniem";
    for zgoda_id in ["spoznione", "inna-zgoda"] {
        expect::refused(
            lokal_krotkoterminowy_kalendarz::wpisz_noc(
                &room,
                &owners_night("2026-12-24", zgoda_id, "2026-12-15T10:00:00Z"),
            ),
            refusal,
        );
    }
    add_night(
        &room,
        &owners_night("2026-12-24", "na-czas", "2026-12-01T10:00:00Z"),
    );

    // A blocked night is freed before anything else takes it.
    let owners_instead_of_works = ZmienNocParams {
        zgoda_id: Some("na-czas".to_string()),
        ..night_change("2026-12-20", StanNocy::Wlasciciel, "2026-12-02T10:00:00Z")
    };
    expect::refused(
        lokal_krotkoterminowy_kalendarz::zmien_noc(&room, &owners_instead_of_works),
        "zajęta noc najpierw się zwalnia",
    );
    lokal_krotkoterminowy_kalendarz::zmien_noc(
        &room,
        &night_change("2026-12-20", StanNocy::Wolna, "2026-12-02T10:00:00Z"),
    )
    .expect("free the night of the works");
}

/// Kalendarz: noc po dacie końca umowy nie jest wolna, a stan po_koncu ma tylko taka noc
#[test]
fn nights_after_contract_end() {
    let room = new_room();
    offering_flat(&room, Some(CONTRACT_ENDS));
    let refusal = "noc po dacie końca umowy nie jest wolna";

    expect::refused(
        lokal_krotkoterminowy_kalendarz::wpisz_noc(
            &room,
            &night("2027-02-01", StanNocy::Wolna, "2026-11-07T09:00:00Z"),
        ),
        refusal,
    );
    expect::refused(
        lokal_krotkoterminowy_kalendarz::wpisz_noc(
            &room,
            &night("2027-01-31", StanNocy::PoKoncu, "2026-11-07T09:00:00Z"),
        ),
        refusal,
    );
    add_night(
        &room,
        &night("2027-01-31", StanNocy::Wolna, "2026-11-07T09:00:00Z"),
    );
    add_night(
        &room,
        &night("2027-02-01", StanNocy::PoKoncu, "2026-11-07T09:00:00Z"),
    );
    expect::refused(
        lokal_krotkoterminowy_kalendarz::zmien_noc(
            &room,
            &night_change("2027-02-01", StanNocy::Wolna, "2026-11-08T09:00:00Z"),
        ),
        refusal,
    );
}

/// The parameters of the price `price_id` of `cena` for the night of 10 December on Airbnb, set
/// on 6 November at 11:00 with no calculation and no decision.
fn night_price(price_id: &str, cena: i64) -> UstawCeneNocyParams {
    UstawCeneNocyParams {
        id: price_id.to_string(),
        data: "2026-12-10".to_string(),
        kanal: "airbnb".to_string(),
        cena,
        ustawiono: "2026-11-06T11:00:00Z".to_string(),
        wyliczenie_id: None,
        decyzja_id: None,
    }
}

/// Kalendarz: cena poza widełkami ma zgodę człowieka, w kanale z umowy albo z ofertą
#[test]
fn night_prices() {
    let room = new_room();
    managed_flat(&room);
    let refusal = "cena poza widełkami z bieżących warunków umowy wskazuje zatwierdzoną decyzję";

    lokal_krotkoterminowy_kalendarz::ustaw_cene_nocy(&room, &night_price("w-widelkach", 100_000))
        .expect("set a price within the limits");
    expect::refused(
        lokal_krotkoterminowy_kalendarz::ustaw_cene_nocy(&room, &night_price("za-wysoka", 200_000)),
        refusal,
    );
    let outside_any_limits = UstawCeneNocyParams {
        data: "2027-06-10".to_string(),
        ..night_price("bez-widelek", 100_000)
    };
    expect::refused(
        lokal_krotkoterminowy_kalendarz::ustaw_cene_nocy(&room, &outside_any_limits),
        refusal,
    );
    approved_decision_about(
        &room,
        "sylwester",
        RodzajDecyzji::Inna,
        "lokal_krotkoterminowy_kalendarz_ceny",
        "za-wysoka",
    );
    lokal_krotkoterminowy_kalendarz::ustaw_cene_nocy(
        &room,
        &UstawCeneNocyParams {
            decyzja_id: Some("sylwester".to_string()),
            ustawiono: "2026-11-06T12:00:00Z".to_string(),
            ..night_price("za-wysoka", 200_000)
        },
    )
    .expect("set a price above the limits by decision");
    let channel_without_offer = UstawCeneNocyParams {
        kanal: "vrbo".to_string(),
        ..night_price("vrbo", 100_000)
    };
    expect::refused(
        lokal_krotkoterminowy_kalendarz::ustaw_cene_nocy(&room, &channel_without_offer),
        "cenę ustawia się w kanale z umowy albo w kanale, w którym lokal ma ofertę",
    );

    let prices = lokal_krotkoterminowy_kalendarz::ceny_nocy(&room).expect("read the prices");
    let price_ids: Vec<&str> = prices.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(price_ids, ["za-wysoka"]);
    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_kalendarz_ceny SET cena = 1",
            &[],
        ),
        "ceny się nie zmienia",
    );
    lokal_krotkoterminowy_kalendarz::wpisz_wyliczenie_cen(
        &room,
        &WpiszWyliczenieCenParams {
            id: "w".to_string(),
            chwila: "2026-11-06T11:00:00Z".to_string(),
            dane_wejsciowe: "sezon".to_string(),
            zrodla: "ceny podobnych lokali z 2026-11-06".to_string(),
            sciezka: "w.json".to_string(),
            skrot: "sha256-w".to_string(),
        },
    )
    .expect("write the price calculation");
    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_kalendarz_wyliczenia SET zrodla = 'inne'",
            &[],
        ),
        "wyliczenia się nie zmienia",
    );
}

/// Kalendarz: publikacja w kanale z umowy z tym samym kontem albo za zgodą właściciela i człowieka
#[test]
fn publication_channel() {
    let room = new_room();
    add_owner(&room, &owner());
    add_flat(&room, &flat("lokal"));
    sign_management_contract(&room, Some(CONTRACT_ENDS));
    established_terms(&room, &terms("warunki"));
    let refusal = "ofertę publikuje się w kanale z bieżących warunków umowy z tym samym kontem";

    draft_offer(
        &room,
        &offer("booking-na-biuro", "booking", KontoOferty::Biura),
    );
    let booking_on_office_account = OpublikujOferteWKanaleParams {
        numer_oferty: "1".to_string(),
        ..publication("booking-na-biuro")
    };
    expect::refused(
        lokal_krotkoterminowy_kalendarz::opublikuj_oferte_w_kanale(
            &room,
            &booking_on_office_account,
        ),
        refusal,
    );

    add_consent(
        &room,
        &WpiszZgodeParams {
            tresc: "Zgadzam się na Vrbo.".to_string(),
            ..consent("vrbo", "wlascicielka", "zgoda_na_kanal")
        },
    );
    draft_offer(
        &room,
        &PrzygotujOferteWKanaleParams {
            zgoda_id: Some("vrbo".to_string()),
            ..offer("oferta-vrbo", "vrbo", KontoOferty::Biura)
        },
    );
    let vrbo_without_decision = OpublikujOferteWKanaleParams {
        numer_oferty: "1".to_string(),
        ..publication("oferta-vrbo")
    };
    expect::refused(
        lokal_krotkoterminowy_kalendarz::opublikuj_oferte_w_kanale(&room, &vrbo_without_decision),
        refusal,
    );
    approved_decision_about(
        &room,
        "nowy-kanal",
        RodzajDecyzji::Inna,
        "lokal_krotkoterminowy_kalendarz_kanaly",
        "oferta-vrbo",
    );
    publish(
        &room,
        &OpublikujOferteWKanaleParams {
            nowy_kanal_decyzja_id: Some("nowy-kanal".to_string()),
            ..publication("oferta-vrbo")
        },
    );

    draft_offer(&room, &airbnb_offer());
    expect::refused(
        lokal_krotkoterminowy_kalendarz::zmien_stan_oferty_w_kanale(
            &room,
            &offer_state("oferta-airbnb", StanOferty::Aktywna),
        ),
        "nie wcześniej niż praca_od",
    );
    publish(&room, &publication("oferta-airbnb"));
    // A writer outside the client can give the state „projekt”, which
    // `lokal_krotkoterminowy_kalendarz::Stan` leaves out. The table refuses it.
    let back_to_draft = room.execute(
        "UPDATE lokal_krotkoterminowy_kalendarz_kanaly
        SET stan = :stan
        WHERE id = :id",
        &[("stan", text("projekt")), ("id", text("oferta-airbnb"))],
    );
    expect::refused(back_to_draft, "oferta w kanale przechodzi");
    set_offer_state(&room, "oferta-airbnb", StanOferty::Zawieszona);
    set_offer_state(&room, "oferta-airbnb", StanOferty::Zamknieta);
    let no_account = WskazRachunekOfertyWKanaleParams {
        rachunek_id: None,
        id: "oferta-airbnb".to_string(),
    };
    expect::refused(
        lokal_krotkoterminowy_kalendarz::wskaz_rachunek_oferty_w_kanale(&room, &no_account),
        "zamkniętej oferty się nie zmienia",
    );
    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_kalendarz_kanaly SET konto = 'wlasciciela'
            WHERE id = 'oferta-vrbo'",
            &[],
        ),
        "kanał, konto i publikację oferty wpisuje się raz",
    );
}

/// The parameters of naming the account `account_id` for the payouts of the Airbnb offer.
fn airbnb_offer_account(account_id: &str) -> WskazRachunekOfertyWKanaleParams {
    WskazRachunekOfertyWKanaleParams {
        rachunek_id: Some(account_id.to_string()),
        id: "oferta-airbnb".to_string(),
    }
}

/// Kalendarz: kanał wypłaca na rachunek dopuszczony po sprawdzeniu Kontrolera (R4)
#[test]
fn channel_account_admitted() {
    let room = new_room();
    managed_flat(&room);
    let refusal = "rachunek do wypłat w kanale jest dopuszczony decyzją, niezastąpiony i ma cel wyplata_w_kanale";
    add_account(
        &room,
        &WpiszRachunekParams {
            cel: CelRachunku::WyplataWKanale,
            ..account("niesprawdzony", "wlascicielka")
        },
    );
    owner_account(&room, "na-wyplaty", CelRachunku::WyplataDlaWlasciciela);
    owner_account(&room, "kanalu", CelRachunku::WyplataWKanale);

    for account_id in ["niesprawdzony", "na-wyplaty"] {
        expect::refused(
            lokal_krotkoterminowy_kalendarz::wskaz_rachunek_oferty_w_kanale(
                &room,
                &airbnb_offer_account(account_id),
            ),
            refusal,
        );
    }
    let offer_with_unchecked_account = PrzygotujOferteWKanaleParams {
        nazwa_konta: "Właścicielka".to_string(),
        rachunek_id: Some("niesprawdzony".to_string()),
        ..offer("druga", "booking", KontoOferty::Wlasciciela)
    };
    expect::refused(
        lokal_krotkoterminowy_kalendarz::przygotuj_oferte_w_kanale(
            &room,
            &offer_with_unchecked_account,
        ),
        refusal,
    );
    lokal_krotkoterminowy_kalendarz::wskaz_rachunek_oferty_w_kanale(
        &room,
        &airbnb_offer_account("kanalu"),
    )
    .expect("name the admitted account of the channel");
}

/// Kalendarz: ostatnie rozliczenie po zamknięciu ofert we wszystkich kanałach
#[test]
fn last_settlement_after_offers_closed() {
    let room = new_room();
    offering_flat(&room, Some(CONTRACT_ENDS));
    established_terms(&room, &terms("warunki"));
    draft_offer(&room, &airbnb_offer());
    publish(&room, &publication("oferta-airbnb"));
    stage(&room, "prowadzenie_lokalu", "2026-11-07T09:00:00Z");
    stage(&room, "koniec_umowy", "2027-01-10T09:00:00Z");
    stage(&room, "zamkniecie_lokalu", "2027-02-01T09:00:00Z");
    signed_protocol(
        &room,
        "zwrot",
        RodzajProtokolu::Zwrot,
        "2027-02-01T10:00:00Z",
    );
    set_offer_state(&room, "oferta-airbnb", StanOferty::Zawieszona);

    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change("ostatnie_rozliczenie", "2027-02-02T09:00:00Z"),
        ),
        "etap ostatnie_rozliczenie przychodzi po zamknięciu ofert",
    );

    set_offer_state(&room, "oferta-airbnb", StanOferty::Zamknieta);
    stage(&room, "ostatnie_rozliczenie", "2027-02-02T09:00:00Z");
}

// Statements --------------------------------------------------------------------------------------

/// A booked flat whose stay ended and was settled on 15 December, with a confirmed cleaning.
fn settled_flat(room: &MemoryRoom) {
    booked_flat(room);
    finished_stay(room);
    settle_stay(room);
    place_order(room, &order("sprzatanie"));
    confirm_order(room, "sprzatanie", "2026-12-13T16:00:00Z");
    record_cost_of_order(room, &cost_of_order("sprzatanie", 15_000));
}

/// Zestawienia: pozycja ma źródło i znak, przychód z rozliczonego pobytu z jego kwotą
#[test]
fn statement_lines() {
    let room = new_room();
    settled_flat(&room);
    draft_statement(&room, &statement("listopad", "2026-11"));
    draft_statement(&room, &statement("zestawienie", "2026-12"));

    expect::refused(
        lokal_krotkoterminowy_zestawienia::dopisz_pozycje_zestawienia(
            &room,
            &line("zestawienie", RodzajPozycji::OplataLokalu, -5_000),
        ),
        "pozycja_ma_zrodlo",
    );
    expect::refused(
        lokal_krotkoterminowy_zestawienia::dopisz_pozycje_zestawienia(
            &room,
            &stay_line("zestawienie", RodzajPozycji::ProwizjaBiura, 1_000),
        ),
        "znak_kwoty_wedlug_rodzaju",
    );
    let refusal =
        "przychód i prowizja kanału wskazują pobyt rozliczony najpóźniej w miesiącu zestawienia";
    let lines_not_matching_settlement = [
        stay_line("listopad", RodzajPozycji::Przychod, 300_000),
        stay_line("zestawienie", RodzajPozycji::Przychod, 290_000),
        stay_line("zestawienie", RodzajPozycji::ProwizjaKanalu, -40_000),
    ];
    for line_not_matching_settlement in &lines_not_matching_settlement {
        expect::refused(
            lokal_krotkoterminowy_zestawienia::dopisz_pozycje_zestawienia(
                &room,
                line_not_matching_settlement,
            ),
            refusal,
        );
    }

    add_line(
        &room,
        &stay_line("zestawienie", RodzajPozycji::Przychod, 300_000),
    );
    add_line(
        &room,
        &stay_line("zestawienie", RodzajPozycji::ProwizjaKanalu, -45_000),
    );
    expect::refused(
        lokal_krotkoterminowy_zestawienia::dopisz_pozycje_zestawienia(
            &room,
            &stay_line("listopad", RodzajPozycji::Przychod, 300_000),
        ),
        "przychód, prowizja kanału, koszt zlecenia i pieniądze pobytu trafiają do jednego zestawienia",
    );
    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_zestawienia_pozycje SET kwota = 1",
            &[],
        ),
        "pozycji zestawienia się nie zmienia",
    );
}

/// [`line`] of the kind „koszt” of `kwota` whose source is the order `order_id`.
fn order_line(statement_id: &str, kwota: i64, order_id: &str) -> DopiszPozycjeZestawieniaParams {
    DopiszPozycjeZestawieniaParams {
        zlecenie_id: Some(order_id.to_string()),
        ..line(statement_id, RodzajPozycji::Koszt, kwota)
    }
}

/// Zestawienia: koszt z potwierdzonego zlecenia, zwrot z pieniędzy pobytu, korekta tylko w korekcie
#[test]
fn statement_costs_and_refunds() {
    let room = new_room();
    settled_flat(&room);
    place_order(
        &room,
        &ZlecPraceParams {
            okno_od: "2026-12-14T10:00:00Z".to_string(),
            okno_do: "2026-12-14T15:00:00Z".to_string(),
            ..order("otwarte")
        },
    );
    lokal_krotkoterminowy_pobyty::wpisz_pieniadze_pobytu(
        &room,
        &WpiszPieniadzePobytuParams {
            wedlug_zasad_kanalu: true,
            ..stay_money("zwrot", RodzajPieniedzy::Zwrot)
        },
    )
    .expect("write the refund");
    add_received_document(&room, &received_document("pismo", RodzajDokumentu::Inny));
    draft_statement(&room, &statement("zestawienie", "2026-12"));
    let cost_needs_confirmed_order = "koszt wskazuje potwierdzone zlecenie z wpisanym kosztem";

    expect::refused(
        lokal_krotkoterminowy_zestawienia::dopisz_pozycje_zestawienia(
            &room,
            &order_line("zestawienie", -15_000, "otwarte"),
        ),
        cost_needs_confirmed_order,
    );
    expect::refused(
        lokal_krotkoterminowy_zestawienia::dopisz_pozycje_zestawienia(
            &room,
            &order_line("zestawienie", -14_000, "sprzatanie"),
        ),
        cost_needs_confirmed_order,
    );
    add_line(&room, &order_line("zestawienie", -15_000, "sprzatanie"));
    expect::refused(
        lokal_krotkoterminowy_zestawienia::dopisz_pozycje_zestawienia(
            &room,
            &order_line("zestawienie", -15_000, "sprzatanie"),
        ),
        "przychód, prowizja kanału, koszt zlecenia i pieniądze pobytu trafiają do jednego zestawienia",
    );

    let recovery_of_refund = DopiszPozycjeZestawieniaParams {
        pieniadze_id: Some("zwrot".to_string()),
        ..line("zestawienie", RodzajPozycji::Odzysk, 10_000)
    };
    expect::refused(
        lokal_krotkoterminowy_zestawienia::dopisz_pozycje_zestawienia(&room, &recovery_of_refund),
        "zwrot wskazuje zwrot albo rekompensatę",
    );
    add_line(
        &room,
        &DopiszPozycjeZestawieniaParams {
            pieniadze_id: Some("zwrot".to_string()),
            ..line("zestawienie", RodzajPozycji::Zwrot, -10_000)
        },
    );
    let correction_outside_correction = DopiszPozycjeZestawieniaParams {
        dokument_id: Some("pismo".to_string()),
        ..line("zestawienie", RodzajPozycji::Korekta, 5_000)
    };
    expect::refused(
        lokal_krotkoterminowy_zestawienia::dopisz_pozycje_zestawienia(
            &room,
            &correction_outside_correction,
        ),
        "pozycja korekty leży w zestawieniu, które wskazuje korygowane zestawienie",
    );
}

/// Zestawienia: kwota do wypłaty to suma pozycji; gotowe zestawienie zamraża plik, kwoty i pozycje
#[test]
fn statement_ready() {
    let room = new_room();
    settled_flat(&room);
    draft_statement(&room, &statement("zestawienie", "2026-12"));
    add_line(
        &room,
        &stay_line("zestawienie", RodzajPozycji::Przychod, 300_000),
    );
    add_line(&room, &order_line("zestawienie", -15_000, "sprzatanie"));

    let closing_with_other_sum = ZamknijZestawienieParams {
        sciezka: "z.pdf".to_string(),
        skrot: "s".to_string(),
        ..statement_closing("zestawienie", 300_000, 0)
    };
    expect::refused(
        lokal_krotkoterminowy_zestawienia::zamknij_zestawienie(&room, &closing_with_other_sum),
        "kwota do wypłaty to suma pozycji zestawienia",
    );
    close_statement(&room, "zestawienie", 285_000, 50_000);

    expect::refused(
        lokal_krotkoterminowy_zestawienia::dopisz_pozycje_zestawienia(
            &room,
            &stay_line("zestawienie", RodzajPozycji::ProwizjaKanalu, -45_000),
        ),
        "pozycje dopisuje się tylko do projektu zestawienia",
    );
    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_zestawienia_wpisy SET kwota_do_wyplaty = 1",
            &[],
        ),
        "plik i kwoty zestawienia zmieniają się tylko w projekcie",
    );
    expect::refused(
        lokal_krotkoterminowy_zestawienia::odnotuj_wyslanie_zestawienia(
            &room,
            &statement_sent("zestawienie"),
        ),
        "zestawienie jest wysłane, gdy jego wysyłka wyszła",
    );

    // Back to draft for a missing line, then ready again with the new sum.
    lokal_krotkoterminowy_zestawienia::cofnij_zestawienie_do_projektu(
        &room,
        &statement_back_to_draft("zestawienie"),
    )
    .expect("take the statement back to draft");
    add_line(
        &room,
        &stay_line("zestawienie", RodzajPozycji::ProwizjaKanalu, -45_000),
    );
    close_statement(&room, "zestawienie", 240_000, 50_000);
    expect::refused(
        room.execute(
            "INSERT INTO lokal_krotkoterminowy_zestawienia_wpisy (id, miesiac, utworzono, stan)
            VALUES ('drugie', '2026-12', '2027-01-05', 'projekt')",
            &[],
        ),
        "wiersza teczki nie zastępuje się nowym wpisem",
    );
}

/// The parameters of taking the statement `statement_id` back to draft.
fn statement_back_to_draft(statement_id: &str) -> CofnijZestawienieDoProjektuParams {
    CofnijZestawienieDoProjektuParams {
        id: statement_id.to_string(),
    }
}

/// Zestawienia: wysłanie za zgodą człowieka; wysłanego zestawienia się nie zmienia
#[test]
fn statement_sent_with_consent() {
    let room = new_room();
    settled_flat(&room);
    draft_statement(&room, &statement("zestawienie", "2026-12"));
    add_line(
        &room,
        &stay_line("zestawienie", RodzajPozycji::Przychod, 300_000),
    );
    letter_to_owner(&room, "wysylka");

    expect::refused(
        lokal_krotkoterminowy_zestawienia::wskaz_wysylke_zestawienia(
            &room,
            &statement_letter("zestawienie", "wysylka"),
        ),
        "gotowe zestawienie wskazuje wysyłkę raz, zanim wyjdzie",
    );
    close_statement(&room, "zestawienie", 300_000, 50_000);
    lokal_krotkoterminowy_zestawienia::wskaz_wysylke_zestawienia(
        &room,
        &statement_letter("zestawienie", "wysylka"),
    )
    .expect("name the letter of the statement");
    let sending_without_consent = OdnotujWyslanieParams {
        wyslano: "2027-01-06T11:00:00Z".to_string(),
        ..sending("wysylka")
    };
    expect::refused(
        sprawa_korespondencja::odnotuj_wyslanie(&room, &sending_without_consent),
        "zestawienie dla właściciela przed wysłaniem czeka na zgodę",
    );
    expect::refused(
        lokal_krotkoterminowy_zestawienia::odnotuj_wyslanie_zestawienia(
            &room,
            &statement_sent("zestawienie"),
        ),
        "zestawienie jest wysłane, gdy jego wysyłka wyszła",
    );
    expect::refused(
        lokal_krotkoterminowy_zestawienia::cofnij_zestawienie_do_projektu(
            &room,
            &statement_back_to_draft("zestawienie"),
        ),
        "do projektu wraca tylko bez wysyłki albo z anulowaną",
    );

    send_with_consent(&room, "wysylka", "2027-01-06");
    lokal_krotkoterminowy_zestawienia::odnotuj_wyslanie_zestawienia(
        &room,
        &statement_sent("zestawienie"),
    )
    .expect("record the sending of the statement");
    expect::refused(
        room.execute(
            "UPDATE lokal_krotkoterminowy_zestawienia_wpisy SET rezerwa_po = 0",
            &[],
        ),
        "wysłanego zestawienia się nie zmienia",
    );
}

/// Zestawienia: korekta wskazuje wcześniejsze wysłane zestawienie i zgodę człowieka
#[test]
fn statement_correction() {
    let room = new_room();
    settled_flat(&room);
    draft_statement(&room, &statement("zestawienie", "2026-12"));
    add_line(
        &room,
        &stay_line("zestawienie", RodzajPozycji::Przychod, 300_000),
    );
    close_statement(&room, "zestawienie", 300_000, 50_000);
    let refusal = "korekta wskazuje wcześniejsze wysłane zestawienie i zatwierdzoną decyzję inna";
    approved_decision_about(
        &room,
        "korekta",
        RodzajDecyzji::Inna,
        "lokal_krotkoterminowy_zestawienia_wpisy",
        "styczen",
    );
    let correction = PrzygotujZestawienieParams {
        utworzono: "2027-02-05T09:00:00Z".to_string(),
        korekta_do_id: Some("zestawienie".to_string()),
        korekta_decyzja_id: Some("korekta".to_string()),
        ..statement("styczen", "2027-01")
    };

    expect::refused(
        lokal_krotkoterminowy_zestawienia::przygotuj_zestawienie(&room, &correction),
        refusal,
    );
    send_statement(&room, "zestawienie", "2027-01-06");
    let correction_without_decision = PrzygotujZestawienieParams {
        korekta_decyzja_id: None,
        ..correction.clone()
    };
    expect::refused(
        lokal_krotkoterminowy_zestawienia::przygotuj_zestawienie(
            &room,
            &correction_without_decision,
        ),
        refusal,
    );
    let correction_of_other_row = PrzygotujZestawienieParams {
        id: "inne".to_string(),
        ..correction.clone()
    };
    expect::refused(
        lokal_krotkoterminowy_zestawienia::przygotuj_zestawienie(&room, &correction_of_other_row),
        refusal,
    );
    draft_statement(&room, &correction);
    add_received_document(&room, &received_document("pismo", RodzajDokumentu::Inny));
    add_line(
        &room,
        &DopiszPozycjeZestawieniaParams {
            dokument_id: Some("pismo".to_string()),
            ..line("styczen", RodzajPozycji::Korekta, -5_000)
        },
    );
}

/// Zestawienia: wypłata wskazuje wysłane zestawienie z jego kwotą, jedna na zestawienie
#[test]
fn payout_from_statement() {
    let room = new_room();
    settled_flat(&room);
    owner_account(&room, "rachunek-wyplat", CelRachunku::WyplataDlaWlasciciela);
    draft_statement(&room, &statement("zestawienie", "2026-12"));
    add_line(
        &room,
        &stay_line("zestawienie", RodzajPozycji::Przychod, 300_000),
    );
    close_statement(&room, "zestawienie", 300_000, 50_000);
    let refusal = "wypłata wskazuje wysłane zestawienie i ma jego kwotę do wypłaty";

    expect::refused(
        sprawa_wyplaty::przygotuj_wyplate(&room, &payout_fields(300_000)),
        refusal,
    );
    send_statement(&room, "zestawienie", "2027-01-06");
    expect::refused(
        sprawa_wyplaty::przygotuj_wyplate(&room, &payout_fields(299_000)),
        refusal,
    );
    let payout_of_deposit = PrzygotujWyplateParams {
        podstawa_tabela: "najem_kaucje".to_string(),
        ..payout_fields(300_000)
    };
    expect::refused(
        sprawa_wyplaty::przygotuj_wyplate(&room, &payout_of_deposit),
        refusal,
    );

    pay_owner(&room, "wyplata", "zestawienie", 300_000, "2027-01-08");
    let second_payout = PrzygotujWyplateParams {
        id: "druga".to_string(),
        ..payout_fields(300_000)
    };
    expect::refused(
        sprawa_wyplaty::przygotuj_wyplate(&room, &second_payout),
        "zestawienie albo zwrot ma jedną wypłatę",
    );
}

/// Zestawienia: po zarządzaniu wszystko wysłane, a każdy rozliczony pobyt i koszt w zestawieniu
#[test]
fn management_ends_with_everything_settled() {
    let room = new_room();
    offering_flat(&room, Some(CONTRACT_ENDS));
    established_terms(&room, &terms("warunki"));
    draft_offer(&room, &airbnb_offer());
    publish(&room, &publication("oferta-airbnb"));
    add_guest(&room);
    book(&room, &booking("pobyt", "2026-12-10", "2026-12-13"));
    stage(&room, "prowadzenie_lokalu", "2026-11-07T09:00:00Z");
    finished_stay(&room);
    settle_stay(&room);
    place_order(&room, &order("sprzatanie"));
    confirm_order(&room, "sprzatanie", "2026-12-13T16:00:00Z");
    record_cost_of_order(&room, &cost_of_order("sprzatanie", 15_000));
    stage(&room, "koniec_umowy", "2027-01-10T09:00:00Z");
    stage(&room, "zamkniecie_lokalu", "2027-02-01T09:00:00Z");
    set_offer_state(&room, "oferta-airbnb", StanOferty::Zamknieta);
    signed_protocol(
        &room,
        "zwrot",
        RodzajProtokolu::Zwrot,
        "2027-02-01T10:00:00Z",
    );
    stage(&room, "ostatnie_rozliczenie", "2027-02-02T09:00:00Z");
    let refusal = "etap po_zarzadzaniu wymaga wysłanych zestawień";

    draft_statement(&room, &statement("zestawienie", "2026-12"));
    add_line(
        &room,
        &stay_line("zestawienie", RodzajPozycji::Przychod, 300_000),
    );
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change("po_zarzadzaniu", "2027-02-05T09:00:00Z"),
        ),
        refusal,
    );
    close_statement(&room, "zestawienie", 300_000, 50_000);
    send_statement(&room, "zestawienie", "2027-01-06");
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change("po_zarzadzaniu", "2027-02-05T09:00:00Z"),
        ),
        refusal,
    );

    draft_statement(&room, &statement("styczen", "2027-01"));
    add_line(&room, &order_line("styczen", -15_000, "sprzatanie"));
    add_line(
        &room,
        &DopiszPozycjeZestawieniaParams {
            warunki_id: Some("warunki".to_string()),
            ..line("styczen", RodzajPozycji::Rezerwa, 15_000)
        },
    );
    close_statement(&room, "styczen", 0, 35_000);
    send_statement(&room, "styczen", "2027-02-03");
    stage(&room, "po_zarzadzaniu", "2027-02-05T09:00:00Z");
}

// R9 and replacement ------------------------------------------------------------------------------

/// The tables of the case type, each with its key and the columns of its UNIQUE keys other than
/// the key.
const TYPE_TABLES: &[(&str, &str, &[&str])] = &[
    ("lokal_krotkoterminowy_umowa_warunki", "id", &[]),
    (
        "lokal_krotkoterminowy_umowa_kanaly",
        "id",
        &["warunki_id, kanal"],
    ),
    ("lokal_krotkoterminowy_umowa_widelki", "id", &[]),
    ("lokal_krotkoterminowy_lokal_dane", "id", &[]),
    ("lokal_krotkoterminowy_lokal_regulaminy", "id", &[]),
    ("lokal_krotkoterminowy_lokal_wyposazenie", "id", &[]),
    ("lokal_krotkoterminowy_lokal_przeglady", "id", &[]),
    ("lokal_krotkoterminowy_lokal_uchwaly", "id", &[]),
    (
        "lokal_krotkoterminowy_pobyty_wpisy",
        "id",
        &["kanal, numer_rezerwacji"],
    ),
    ("lokal_krotkoterminowy_pobyty_zmiany", "id", &[]),
    ("lokal_krotkoterminowy_pobyty_zgloszenia", "id", &[]),
    ("lokal_krotkoterminowy_pobyty_wyjatki", "id", &[]),
    ("lokal_krotkoterminowy_pobyty_opinie", "id", &["pobyt_id"]),
    ("lokal_krotkoterminowy_pobyty_pieniadze", "id", &[]),
    ("lokal_krotkoterminowy_pobyty_skargi", "id", &[]),
    ("lokal_krotkoterminowy_zlecenia_wpisy", "id", &[]),
    ("lokal_krotkoterminowy_zlecenia_dostepy", "id", &[]),
    ("lokal_krotkoterminowy_kalendarz_noce", "data", &[]),
    ("lokal_krotkoterminowy_kalendarz_wyliczenia", "id", &[]),
    ("lokal_krotkoterminowy_kalendarz_ceny", "id", &[]),
    ("lokal_krotkoterminowy_kalendarz_kanaly", "id", &[]),
    (
        "lokal_krotkoterminowy_zestawienia_wpisy",
        "id",
        &["miesiac"],
    ),
    ("lokal_krotkoterminowy_zestawienia_pozycje", "id", &[]),
];

/// The names of the tables of the case type in the room's schema, sorted.
fn type_tables(room: &MemoryRoom) -> Vec<String> {
    let rows = room
        .execute(
            "SELECT name FROM sqlite_schema
            WHERE type = 'table' AND name LIKE 'lokal\\_krotkoterminowy\\_%' ESCAPE '\\'",
            &[],
        )
        .expect("read the tables of the type");

    let mut tables: Vec<String> = rows
        .iter()
        .map(|row| row.get::<String>("name").expect("a table has a name"))
        .collect();
    tables.sort();

    tables
}

/// R9: wierszy typu lokalu się nie usuwa
#[test]
fn no_deletion() {
    let room = new_room();
    full_case(&room);

    let mut expected_tables: Vec<&str> = TYPE_TABLES.iter().map(|(table, _, _)| *table).collect();
    expected_tables.sort();
    let tables = type_tables(&room);
    assert_eq!(tables, expected_tables, "{tables:?}");

    for (table, _, _) in TYPE_TABLES {
        let rows = integer_value(&room, &format!("SELECT count(*) FROM {table}"));
        assert!(rows > 0, "{table} has no row to delete");

        let sql = format!("DELETE FROM {table} WHERE rowid = (SELECT min(rowid) FROM {table})");
        expect::refused(room.execute(&sql, &[]), "wierszy teczki się nie usuwa (R9)");
    }
}

/// Wiersza typu lokalu nie zastępuje INSERT OR REPLACE ani upsert
#[test]
fn no_replacement() {
    let room = new_room();
    full_case(&room);
    let refusal = "wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE";

    for (table, key, unique_keys) in TYPE_TABLES {
        let rows = integer_value(&room, &format!("SELECT count(*) FROM {table}"));
        assert!(rows > 0, "{table} has no row to replace");

        let replace = format!("INSERT OR REPLACE {}", first_row_copy(&room, table));
        expect::refused(room.execute(&replace, &[]), refusal);

        let upsert = format!(
            "INSERT {} ON CONFLICT ({key}) DO UPDATE SET {key} = excluded.{key}",
            first_row_copy(&room, table)
        );
        expect::refused(room.execute(&upsert, &[]), refusal);

        for columns in *unique_keys {
            let replace_by_unique_key = format!(
                "INSERT OR REPLACE {}",
                first_row_copy_without_id(&room, table)
            );
            expect::refused(room.execute(&replace_by_unique_key, &[]), refusal);

            let upsert_by_unique_key = format!(
                "INSERT {} ON CONFLICT ({columns}) DO UPDATE SET {key} = excluded.{key}",
                first_row_copy_without_id(&room, table)
            );
            expect::refused(room.execute(&upsert_by_unique_key, &[]), refusal);
        }
    }

    // The lines of a statement have partial unique keys: a second revenue line of the same stay.
    expect::refused(
        room.execute(
            "INSERT OR REPLACE INTO lokal_krotkoterminowy_zestawienia_pozycje
              (zestawienie_id, rodzaj, kwota, pobyt_id)
            VALUES ('zestawienie', 'przychod', 300000, 'pobyt')",
            &[],
        ),
        "przychód, prowizja kanału, koszt zlecenia i pieniądze pobytu trafiają do jednego zestawienia",
    );
}
