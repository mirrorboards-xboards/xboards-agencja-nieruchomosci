//! Scenarios of the offer case catalog (spec/cases/oferta) on top of the core.
//!
//! An offer room is the core of every case plus the kinds of the offer type. Each scenario starts
//! from an empty room and checks one rule the database keeps: a write the rule allows and a write it
//! refuses. The first scenario follows a whole offer case as the profile describes it, from intake
//! through the brokerage contract, price, advertisement, viewing and the chain of offers to closing.

use xboards_agencja_nieruchomosci::cases::oferta::{
    self, oferta_ceny, oferta_najem, oferta_nieruchomosc, oferta_oferty, oferta_ogloszenia,
    oferta_widocznosc, oferta_zainteresowani,
};
use xboards_agencja_nieruchomosci::cases::sprawa::{
    sprawa_decyzje, sprawa_dokumenty, sprawa_korespondencja, sprawa_metryka, sprawa_powiazania,
    sprawa_rachunki, sprawa_uczestnicy, sprawa_umowy, sprawa_wyplaty, sprawa_zgody,
};
use xboards_agencja_nieruchomosci::xmeta_room::{FromSqlValue, MemoryRoom, MetaRoom, SqlValue};

use oferta_ceny::{
    DopiszPorownanieTransakcjiParams, GraniceDotyczy as DotyczyCeny,
    OdnotujCeneWOgloszeniachParams, WpiszCeneOfertowaParams, WpiszGraniceNegocjacjiParams,
    WpiszPropozycjeCenyParams,
};
use oferta_najem::WpiszWarunkiNajmuParams;
use oferta_nieruchomosc::{
    OpisSwiadectwo, WpiszOpisNieruchomosciParams, WpiszPrzegladanieKsiegiParams,
    ZmienOpisNieruchomosciParams,
};
use oferta_oferty::{
    OdnotujDecyzjeKlientaOOfercieParams, OdnotujOdpowiedzZainteresowanegoParams,
    OdnotujPrzedstawienieOfertyParams, WpisyDecyzjaKlienta as DecyzjaKlienta,
    WpisyDotyczy as DotyczyOferty, WpisyKierunek as KierunekOferty,
    WpisySposobZlozenia as SposobZlozenia, WpiszKontroferteKlientaParams,
    WpiszOferteZainteresowanegoParams, WskazOdpowiedzNaOferteParams,
};
use oferta_ogloszenia::{
    DopiszWersjeOgloszeniaParams, OdnotujNumerOgloszeniaParams, OdnotujPublikacjeOgloszeniaParams,
    PrzygotujOgloszenieParams, WpisyDotyczy as DotyczyOgloszenia, WpisyKanal as KanalOgloszenia,
    WpiszStatystykiOgloszeniaParams, ZdejmijOgloszenieParams,
};
use oferta_widocznosc::{
    WersjeRodzajOferty as RodzajOferty, WersjeWidocznosc as Widocznosc,
    WpiszWersjeWidocznosciParams,
};
use oferta_zainteresowani::{
    DopiszUczestnikaPrezentacjiParams, OdnotujInformacjeDlaZainteresowanegoParams,
    OdnotujObecnoscNaPrezentacjiParams, OdnotujOdpowiedzNaZapytanieParams,
    OdnotujPodanieAdresuParams, OdnotujPrzebiegPrezentacjiParams, OdnotujWynikZapytaniaParams,
    OdwolajPrezentacjeParams, OsobyDzialaJako as DzialaJako, PrezentacjeOtwiera as Otwiera,
    UmowPrezentacjeParams, WpiszZainteresowanegoParams, WpiszZapytanieParams,
    ZmienKontaktZainteresowanegoParams,
};
use sprawa_decyzje::{
    PoprosODecyzjeParams, WpisyProsiStanowisko as ProsiStanowisko, WpisyRodzaj as RodzajDecyzji,
};
use sprawa_dokumenty::{
    OdrzucDokumentParams, WpisyRodzaj as RodzajDokumentu, WpisyWaznoscZrodlo as WaznoscZrodlo,
    WpiszOtrzymanyDokumentParams, WpiszZrodloParams,
};
use sprawa_korespondencja::{
    OdnotujWyslanieParams, PrzekazWysylkeDoZgodyParams, PrzygotujWysylkeParams, WpiszWplywParams,
    WplywyKanal, WysylkiAdresatRodzaj,
};
use sprawa_metryka::{
    PostawWatpliwoscParams, ZalozMetrykeParams, ZdejmijWatpliwoscParams, ZmienEtapSprawyParams,
};
use sprawa_powiazania::{WpisyRodzaj as RodzajPowiazania, WpiszPowiazanieParams};
use sprawa_rachunki::{WpisyCel as CelRachunku, WpiszRachunekParams};
use sprawa_uczestnicy::{NadajRoleUczestnikowiParams, WpiszUczestnikaParams};
use sprawa_umowy::{
    DopiszCzynnoscUmowyParams, PrzekazUmoweDoPodpisuParams, PrzygotujUmoweParams,
    WpisyRodzaj as RodzajUmowy, WpisySposobZawarcia as SposobZawarcia, WpiszZdarzenieUmowyParams,
    ZakonczUmoweParams, ZdarzeniaRodzaj as RodzajZdarzeniaUmowy,
};
use sprawa_wyplaty::PrzygotujWyplateParams;
use sprawa_zgody::WpiszZgodeParams;

use crate::expect;
use crate::sprawa::{
    WITHDRAWAL_ENDS, add_consent, add_flat, add_inflow, add_person, add_received_document, approve,
    approved_decision, ask_for_decision, check_identity, conclude_contract, consent, contract,
    decision_request, decision_request_about, flat, inflow, letter, open_case, person,
    prepare_contract, prepare_letter, received_document, send, sending,
};
use crate::sprawa_porzadek::{first_row_copy, first_row_copy_without_id, unique_keys_of};

/// The moment the brokerage contract of [`brokerage_contract`] is concluded.
const CONCLUDED: &str = "2026-11-04T12:00:00Z";

/// The moment of a stage change when a scenario names none (the Python default of `stage`).
const STAGE_CHANGED: &str = "2026-11-04T15:00:00Z";

/// The moment of the first publication of an ad when a scenario names none (the Python default of
/// `publish`).
const PUBLISHED: &str = "2026-11-05T10:00:00Z";

const REPLACEMENT: &str =
    "wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE";
const DELETION: &str = "wierszy teczki się nie usuwa (R9)";

fn new_room() -> MemoryRoom {
    oferta::open_in_memory().expect("open the offer room")
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

// Building blocks of an offer case ----------------------------------------------------------------

/// The default parameters of a request for a decision „inna” about the row `dotyczy_id` of the
/// table `dotyczy_tabela` (see [`decision_request_about`]).
fn other_decision(
    decision_id: &str,
    dotyczy_tabela: &str,
    dotyczy_id: &str,
) -> PoprosODecyzjeParams {
    decision_request_about(decision_id, RodzajDecyzji::Inna, dotyczy_tabela, dotyczy_id)
}

/// Writes a decision of the broker of `request`, asked by the offer keeper, and approves it.
fn approved_other(room: &impl MetaRoom, request: &PoprosODecyzjeParams) {
    let by_offer_keeper = PoprosODecyzjeParams {
        prosi_stanowisko: ProsiStanowisko::OpiekunOferty,
        ..request.clone()
    };
    ask_for_decision(room, &by_offer_keeper);

    approve(room, &request.id, "2026-11-03T12:00:00Z");
}

/// Writes the client „klientka”: a consumer and a client of the office, with the role „klient”.
fn add_client(room: &impl MetaRoom) {
    add_person(
        room,
        &WpiszUczestnikaParams {
            konsument: true,
            klient_biura: true,
            ..person("klientka")
        },
    );

    let role = NadajRoleUczestnikowiParams {
        uczestnik_id: "klientka".to_string(),
        rola: "klient".to_string(),
        dotyczy_uczestnik_id: None,
    };
    sprawa_uczestnicy::nadaj_role_uczestnikowi(room, &role).expect("give the client her role");
}

/// The terms of a brokerage contract of the client for the flat (see [`brokerage_terms`]).
struct BrokerageTerms {
    contract_id: &'static str,
    czynnosci: &'static [&'static str],
    obie_strony: bool,
    sposob_zawarcia: SposobZawarcia,
    odstapienie_do: Option<&'static str>,
}

/// The default terms of a brokerage contract: „umowa” for publishing ads and viewings, for one
/// side only, concluded at the office.
fn brokerage_terms() -> BrokerageTerms {
    BrokerageTerms {
        contract_id: "umowa",
        czynnosci: &["publikacja_ogloszen", "prezentacje"],
        obie_strony: false,
        sposob_zawarcia: SposobZawarcia::WLokaluBiura,
        odstapienie_do: None,
    }
}

/// Writes a brokerage contract of `terms` for the flat with the client as its party: her identity
/// check `tozsamosc-<id>`, the draft with its acts, the hand-over for signing and the conclusion
/// on 2026-11-04.
fn brokerage_contract(room: &impl MetaRoom, terms: &BrokerageTerms) {
    let contract_id = terms.contract_id;
    check_identity(room, &format!("tozsamosc-{contract_id}"), "klientka");

    let draft = PrzygotujUmoweParams {
        rodzaj: RodzajUmowy::Posrednictwo,
        nieruchomosc_id: Some("lokal".to_string()),
        obie_strony: terms.obie_strony,
        wynagrodzenie_opis: Some("2,5% ceny".to_string()),
        wynagrodzenie_zdarzenie: Some("zawarcie umowy przyrzeczonej".to_string()),
        ..contract(contract_id)
    };
    prepare_contract(room, &draft, &["klientka"]);

    for czynnosc in terms.czynnosci {
        let act = DopiszCzynnoscUmowyParams {
            umowa_id: contract_id.to_string(),
            czynnosc: czynnosc.to_string(),
        };

        sprawa_umowy::dopisz_czynnosc_umowy(room, &act).expect("add an act of the contract");
    }

    let for_signing = PrzekazUmoweDoPodpisuParams {
        id: contract_id.to_string(),
    };
    sprawa_umowy::przekaz_umowe_do_podpisu(room, &for_signing)
        .expect("hand the contract over for signing");

    conclude_contract(
        room,
        contract_id,
        terms.sposob_zawarcia,
        terms.odstapienie_do,
    );
}

/// Writes an offer case with the client, the flat and a concluded brokerage contract of `terms`.
fn offer_case(room: &impl MetaRoom, terms: &BrokerageTerms) {
    open_case(room, "oferta");
    add_client(room);
    add_flat(room, &flat("lokal"));
    brokerage_contract(room, terms);
}

/// The default parameters of a version of visibility: a public sale offer under „umowa”, from the
/// conclusion of the contract.
fn visibility_version(version_id: &str) -> WpiszWersjeWidocznosciParams {
    WpiszWersjeWidocznosciParams {
        id: version_id.to_string(),
        umowa_id: "umowa".to_string(),
        rodzaj_oferty: RodzajOferty::Sprzedaz,
        widocznosc: Widocznosc::Publiczna,
        granice_zgody: None,
        aneks_zdarzenie_id: None,
        od: CONCLUDED.to_string(),
    }
}

/// Writes the version of visibility of `params` (see [`visibility_version`]).
fn add_visibility(room: &impl MetaRoom, params: &WpiszWersjeWidocznosciParams) {
    oferta_widocznosc::wpisz_wersje_widocznosci(room, params).expect("add a version of visibility");
}

/// The default parameters of the client's consent `rodzaj` about the row `dotyczy_id` of the table
/// `dotyczy_tabela` (see [`consent`]).
fn client_consent(
    consent_id: &str,
    rodzaj: &str,
    dotyczy_tabela: &str,
    dotyczy_id: &str,
) -> WpiszZgodeParams {
    WpiszZgodeParams {
        tresc: format!("Zgoda {rodzaj}."),
        dotyczy_tabela: Some(dotyczy_tabela.to_string()),
        dotyczy_id: Some(dotyczy_id.to_string()),
        ..consent(consent_id, "klientka", rodzaj)
    }
}

/// The default parameters of an offer price: 659 000 zł for the sale from 2026-11-04 at 14:00,
/// with the client's consent `zgoda-<id>`.
fn offer_price(price_id: &str) -> WpiszCeneOfertowaParams {
    WpiszCeneOfertowaParams {
        id: price_id.to_string(),
        dotyczy: DotyczyCeny::Sprzedaz,
        kwota: 65_900_000,
        zgoda_id: format!("zgoda-{price_id}"),
        obowiazuje_od: "2026-11-04T14:00:00Z".to_string(),
        poprzednia_id: None,
        powod: None,
    }
}

/// Writes the client's consent `zgoda-<id>` to the price, then the offer price of `params` (see
/// [`offer_price`]).
fn add_offer_price(room: &impl MetaRoom, params: &WpiszCeneOfertowaParams) {
    let price_consent = client_consent(
        &format!("zgoda-{}", params.id),
        "zatwierdzenie_ceny",
        "oferta_ceny_ofertowe",
        &params.id,
    );
    add_consent(room, &price_consent);

    oferta_ceny::wpisz_cene_ofertowa(room, params).expect("add the offer price");
}

/// The default parameters of an ad of the sale on otodom.pl.
fn ad(ad_id: &str) -> PrzygotujOgloszenieParams {
    PrzygotujOgloszenieParams {
        id: ad_id.to_string(),
        dotyczy: DotyczyOgloszenia::Sprzedaz,
        kanal: KanalOgloszenia::Portal,
        portal: Some("otodom.pl".to_string()),
    }
}

/// The default parameters of the version `numer` of the ad `ad_id`: the flat for 659 000 zł from
/// 2026-11-05 at 09:00, with its content `ogloszenia/<id>-<numer>.json`.
fn ad_version(ad_id: &str, numer: i64) -> DopiszWersjeOgloszeniaParams {
    DopiszWersjeOgloszeniaParams {
        ogloszenie_id: ad_id.to_string(),
        numer,
        od: "2026-11-05T09:00:00Z".to_string(),
        tytul: "2 pokoje, 48 m², Krzyki".to_string(),
        cena: 65_900_000,
        lokalizacja: "Wrocław, Krzyki, ul. Polna".to_string(),
        wskazniki_energetyczne: None,
        tresc_sciezka: format!("ogloszenia/{ad_id}-{numer}.json"),
        tresc_skrot: format!("sha256-{ad_id}-{numer}"),
    }
}

/// Writes the version of the ad of `params` (see [`ad_version`]).
fn add_ad_version(room: &impl MetaRoom, params: &DopiszWersjeOgloszeniaParams) {
    oferta_ogloszenia::dopisz_wersje_ogloszenia(room, params).expect("add a version of the ad");
}

/// Writes the draft ad of `ad` (see [`ad`]) and its first version `first_version` (see
/// [`ad_version`]).
fn draft_ad(
    room: &impl MetaRoom,
    ad: &PrzygotujOgloszenieParams,
    first_version: &DopiszWersjeOgloszeniaParams,
) {
    oferta_ogloszenia::przygotuj_ogloszenie(room, ad).expect("prepare the ad");

    add_ad_version(room, first_version);
}

/// The parameters of the publication of the ad `ad_id` at `opublikowano`.
fn publication(ad_id: &str, opublikowano: &str) -> OdnotujPublikacjeOgloszeniaParams {
    OdnotujPublikacjeOgloszeniaParams {
        opublikowano: opublikowano.to_string(),
        id: ad_id.to_string(),
    }
}

/// Records the publication of the ad `ad_id` at `opublikowano` (the Python defaults are
/// „ogloszenie” and [`PUBLISHED`]).
fn publish(room: &impl MetaRoom, ad_id: &str, opublikowano: &str) {
    let params = publication(ad_id, opublikowano);

    oferta_ogloszenia::odnotuj_publikacje_ogloszenia(room, &params).expect("publish the ad");
}

/// Writes an offer case with public visibility, a price and an ad draft the broker approved.
fn ready_to_publish(room: &impl MetaRoom) {
    offer_case(room, &brokerage_terms());
    add_visibility(room, &visibility_version("widocznosc"));
    add_offer_price(room, &offer_price("cena"));
    draft_ad(room, &ad("ogloszenie"), &ad_version("ogloszenie", 1));
    approved_other(
        room,
        &other_decision(
            "pierwsza-publikacja",
            "oferta_ogloszenia_wpisy",
            "ogloszenie",
        ),
    );
}

/// The default parameters of an interested person: Piotr, by phone, acting for himself.
fn interested(person_id: &str) -> WpiszZainteresowanegoParams {
    WpiszZainteresowanegoParams {
        id: person_id.to_string(),
        imie: "Piotr".to_string(),
        nazwisko: None,
        telefon: Some("+48 600 100 200".to_string()),
        email: None,
        dziala_jako: DzialaJako::Sam,
        biuro: None,
        dla_kogo: None,
        dane_pozyskano: "2026-11-06T09:00:00Z".to_string(),
        informacja_wysylka_id: None,
        informacja_od_recepcji: None,
    }
}

/// Writes the interested person of `params` (see [`interested`]).
fn add_interested(room: &impl MetaRoom, params: &WpiszZainteresowanegoParams) {
    oferta_zainteresowani::wpisz_zainteresowanego(room, params).expect("add the interested person");
}

/// Writes a sent letter `letter_id` to Piotr with the information on personal data (art. 13 RODO).
fn send_privacy_notice(room: &impl MetaRoom, letter_id: &str) {
    let notice = PrzygotujWysylkeParams {
        adresat_rodzaj: Some(WysylkiAdresatRodzaj::Inny),
        adresat_nazwa: Some("Piotr".to_string()),
        wzor: Some("informacja-o-danych".to_string()),
        informacja_rodo: true,
        ..letter(letter_id)
    };
    prepare_letter(room, &notice);

    send(
        room,
        &OdnotujWyslanieParams {
            wyslano: "2026-11-06T09:30:00Z".to_string(),
            ..sending(letter_id)
        },
    );
}

/// The default parameters of an offer of 610 000 zł for the sale from „zainteresowany”, given in
/// the inflow `wplyw-<id>` on 2026-11-10 and binding until 2026-11-12.
fn interested_offer(offer_id: &str) -> WpiszOferteZainteresowanegoParams {
    WpiszOferteZainteresowanegoParams {
        id: offer_id.to_string(),
        dotyczy: DotyczyOferty::Sprzedaz,
        osoba_id: "zainteresowany".to_string(),
        odpowiada_na_id: None,
        wplyw_id: Some(format!("wplyw-{offer_id}")),
        dokument_id: None,
        wpisano: "2026-11-10T09:00:00Z".to_string(),
        kwota: 61_000_000,
        zadatek_albo_kaucja: None,
        termin_umowy: None,
        termin_wydania: None,
        finansowanie: Some("kredyt hipoteczny, decyzja wstępna z banku".to_string()),
        warunki_zastrzezone: None,
        wiaze_do: Some("2026-11-12".to_string()),
        sposob_zlozenia: None,
    }
}

/// Writes the inflow `wplyw-<id>` from Piotr, then the offer of `params` (see
/// [`interested_offer`]).
fn add_interested_offer(room: &impl MetaRoom, params: &WpiszOferteZainteresowanegoParams) {
    let carrier = WpiszWplywParams {
        nadawca: "Piotr <piotr@example.pl>".to_string(),
        ..inflow(&format!("wplyw-{}", params.id))
    };
    add_inflow(room, &carrier);

    oferta_oferty::wpisz_oferte_zainteresowanego(room, params)
        .expect("add the offer of the interested person");
}

/// Presents the offer `offer_id` to the client and records her decision with her consent
/// `decyzja-<id>`.
fn client_decision(room: &impl MetaRoom, offer_id: &str, decyzja_klienta: DecyzjaKlienta) {
    let presentation = OdnotujPrzedstawienieOfertyParams {
        przedstawiono: "2026-11-10T10:00:00Z".to_string(),
        obie_strony_decyzja_id: None,
        id: offer_id.to_string(),
    };
    oferta_oferty::odnotuj_przedstawienie_oferty(room, &presentation)
        .expect("present the offer to the client");

    let consent_id = format!("decyzja-{offer_id}");
    let decision_consent = client_consent(
        &consent_id,
        "odpowiedz_na_oferte",
        "oferta_oferty_wpisy",
        offer_id,
    );
    add_consent(room, &decision_consent);

    let decision = OdnotujDecyzjeKlientaOOfercieParams {
        decyzja_klienta,
        zgoda_id: consent_id,
        id: offer_id.to_string(),
    };
    oferta_oferty::odnotuj_decyzje_klienta_o_ofercie(room, &decision)
        .expect("record the client's decision");
}

/// The default parameters of a reply to an offer for Piotr.
fn reply_letter(letter_id: &str) -> PrzygotujWysylkeParams {
    PrzygotujWysylkeParams {
        adresat_rodzaj: Some(WysylkiAdresatRodzaj::Inny),
        adresat_nazwa: Some("Piotr".to_string()),
        wzor: Some("odpowiedz-na-oferte".to_string()),
        ..letter(letter_id)
    }
}

/// Writes the reply `odpowiedz-<id>` to the offer, linked to it and waiting for the human's
/// consent to send it, which the decision `wyslanie-<letter id>` gives; the id of the reply.
fn reply_awaiting_consent(room: &impl MetaRoom, offer_id: &str) -> String {
    let letter_id = format!("odpowiedz-{offer_id}");
    prepare_letter(room, &reply_letter(&letter_id));

    let reply = WskazOdpowiedzNaOferteParams {
        odpowiedz_wysylka_id: letter_id.clone(),
        id: offer_id.to_string(),
    };
    oferta_oferty::wskaz_odpowiedz_na_oferte(room, &reply).expect("link the reply to the offer");

    let for_consent = PrzekazWysylkeDoZgodyParams {
        id: letter_id.clone(),
    };
    sprawa_korespondencja::przekaz_wysylke_do_zgody(room, &for_consent)
        .expect("hand the reply over for consent");

    approved_other(
        room,
        &decision_request_about(
            &format!("wyslanie-{letter_id}"),
            RodzajDecyzji::Wyslanie,
            "sprawa_korespondencja_wysylki",
            &letter_id,
        ),
    );

    letter_id
}

/// The parameters of a change of the stage of the case by the offer agent, with no decision.
fn stage_change(etap: &str, od: &str) -> ZmienEtapSprawyParams {
    ZmienEtapSprawyParams {
        etap: etap.to_string(),
        od: od.to_string(),
        zmienil: "agent-oferty".to_string(),
        decyzja_id: None,
    }
}

/// Moves the case to the stage of `params` (see [`stage_change`]).
fn stage(room: &impl MetaRoom, params: &ZmienEtapSprawyParams) {
    sprawa_metryka::zmien_etap_sprawy(room, params).expect("change the stage of the case");
}

/// The parameters of a stage change by the writer „a”, which the rules of stages check.
fn stage_change_by_a(etap: &str, od: &str) -> ZmienEtapSprawyParams {
    ZmienEtapSprawyParams {
        zmienil: "a".to_string(),
        ..stage_change(etap, od)
    }
}

/// The parameters of a stage change the decision `decision_id` allows.
fn stage_change_by_decision(etap: &str, od: &str, decision_id: &str) -> ZmienEtapSprawyParams {
    ZmienEtapSprawyParams {
        decyzja_id: Some(decision_id.to_string()),
        ..stage_change(etap, od)
    }
}

/// Ends the contract `contract_id` for the reason `koniec_powod`.
fn end_contract(room: &impl MetaRoom, contract_id: &str, koniec_powod: &str) {
    let params = ZakonczUmoweParams {
        koniec_powod: koniec_powod.to_string(),
        id: contract_id.to_string(),
    };

    sprawa_umowy::zakoncz_umowe(room, &params).expect("end the contract");
}

/// The parameters of a description of the flat with nothing but the state of its energy
/// certificate.
fn empty_description(swiadectwo: OpisSwiadectwo) -> WpiszOpisNieruchomosciParams {
    WpiszOpisNieruchomosciParams {
        nieruchomosc_id: "lokal".to_string(),
        liczba_pieter: None,
        winda: None,
        rok_budowy: None,
        material: None,
        ogrzewanie: None,
        stan_lokalu: None,
        remonty: None,
        przeznaczenie_dzialki: None,
        dostep_do_drogi: None,
        media: None,
        uzytki_rolne_m2: None,
        oplata_miesieczna: None,
        media_srednio: None,
        podatek: None,
        swiadectwo,
        swiadectwo_dokument_id: None,
        wskazniki_energetyczne: None,
        osoby_w_lokalu: None,
        wynajety_do: None,
        przeznaczenie_lokalu_uzytkowego: None,
    }
}

/// The parameters of a change of the description of the flat to nothing but the state of its
/// energy certificate.
fn empty_description_change(swiadectwo: OpisSwiadectwo) -> ZmienOpisNieruchomosciParams {
    ZmienOpisNieruchomosciParams {
        liczba_pieter: None,
        winda: None,
        rok_budowy: None,
        material: None,
        ogrzewanie: None,
        stan_lokalu: None,
        remonty: None,
        przeznaczenie_dzialki: None,
        dostep_do_drogi: None,
        media: None,
        uzytki_rolne_m2: None,
        oplata_miesieczna: None,
        media_srednio: None,
        podatek: None,
        swiadectwo,
        swiadectwo_dokument_id: None,
        wskazniki_energetyczne: None,
        osoby_w_lokalu: None,
        wynajety_do: None,
        przeznaczenie_lokalu_uzytkowego: None,
        nieruchomosc_id: "lokal".to_string(),
    }
}

/// The parameters of a description of the flat whose energy certificate is the document
/// `document_id`, with its indicators.
fn certificate_description(document_id: &str) -> WpiszOpisNieruchomosciParams {
    WpiszOpisNieruchomosciParams {
        swiadectwo_dokument_id: Some(document_id.to_string()),
        wskazniki_energetyczne: Some("EP 120 kWh/(m²·rok)".to_string()),
        ..empty_description(OpisSwiadectwo::Jest)
    }
}

/// The parameters of a received energy certificate `document_id` issued on `data_wystawienia` and
/// valid until `waznosc_do` by law.
fn energy_certificate(
    document_id: &str,
    data_wystawienia: &str,
    waznosc_do: &str,
) -> WpiszOtrzymanyDokumentParams {
    WpiszOtrzymanyDokumentParams {
        data_wystawienia: Some(data_wystawienia.to_string()),
        waznosc_do: Some(waznosc_do.to_string()),
        waznosc_zrodlo: Some(WaznoscZrodlo::Przepis),
        ..received_document(document_id, RodzajDokumentu::SwiadectwoEnergetyczne)
    }
}

/// Writes an offer case with a row in every table of the offer type, as the profile goes.
fn full_case(room: &impl MetaRoom) {
    open_case(room, "oferta");
    stage(
        room,
        &stage_change("weryfikacja_klienta", "2026-11-02T10:00:00Z"),
    );

    // The client, her husband and the flat with its land register entries looked up.
    add_client(room);
    add_person(
        room,
        &WpiszUczestnikaParams {
            imiona: Some("Jan".to_string()),
            nazwisko: Some("Kowalski".to_string()),
            ..person("maz")
        },
    );
    sprawa_uczestnicy::nadaj_role_uczestnikowi(
        room,
        &NadajRoleUczestnikowiParams {
            uczestnik_id: "maz".to_string(),
            rola: "malzonek".to_string(),
            dotyczy_uczestnik_id: Some("klientka".to_string()),
        },
    )
    .expect("give the husband his role");
    add_flat(room, &flat("lokal"));
    oferta_nieruchomosc::wpisz_przegladanie_ksiegi(
        room,
        &WpiszPrzegladanieKsiegiParams {
            id: "przegladanie".to_string(),
            nieruchomosc_id: "lokal".to_string(),
            numer_kw: "WR1K/00012345/6".to_string(),
            przegladano: "2026-11-03T09:00:00Z".to_string(),
            wlasciciele_dzial_ii: "Anna Kowalska".to_string(),
            wpisy_dzial_iii: None,
            wpisy_dzial_iv: Some("hipoteka umowna 300 000 zł".to_string()),
            rozbieznosc: false,
            rozbieznosc_opis: None,
            wydruk_sciezka: Some("ksiega/przegladanie.pdf".to_string()),
            wydruk_skrot: Some("sha256-przegladanie".to_string()),
        },
    )
    .expect("add the land register lookup");
    add_received_document(
        room,
        &WpiszOtrzymanyDokumentParams {
            dotyczy_nieruchomosc_id: Some("lokal".to_string()),
            ..energy_certificate("swiadectwo", "2020-05-10", "2030-05-10")
        },
    );
    oferta_nieruchomosc::wpisz_opis_nieruchomosci(
        room,
        &WpiszOpisNieruchomosciParams {
            liczba_pieter: Some(4),
            winda: Some(false),
            rok_budowy: Some(1978),
            ogrzewanie: Some("miejskie".to_string()),
            oplata_miesieczna: Some(48_000),
            swiadectwo_dokument_id: Some("swiadectwo".to_string()),
            wskazniki_energetyczne: Some("EP 120 kWh/(m²·rok)".to_string()),
            osoby_w_lokalu: Some("nikt nie mieszka".to_string()),
            ..empty_description(OpisSwiadectwo::Jest)
        },
    )
    .expect("add the description of the flat");
    sprawa_dokumenty::wpisz_zrodlo(
        room,
        &WpiszZrodloParams {
            dotyczy_tabela: "oferta_nieruchomosc_opis".to_string(),
            dotyczy_id: "lokal".to_string(),
            pole: "oplata_miesieczna".to_string(),
            dokument_id: None,
            od_uczestnik_id: Some("klientka".to_string()),
            od_kogo: None,
            potwierdzone: false,
            wpisano: "2026-11-03T10:00:00Z".to_string(),
        },
    )
    .expect("add the source of the monthly fee");

    // The brokerage contract signed at a distance by a consumer: the office starts work before
    // the end of the withdrawal period only on her demand.
    stage(
        room,
        &stage_change("umowa_posrednictwa", "2026-11-03T12:00:00Z"),
    );
    brokerage_contract(
        room,
        &BrokerageTerms {
            sposob_zawarcia: SposobZawarcia::NaOdleglosc,
            odstapienie_do: Some(WITHDRAWAL_ENDS),
            ..brokerage_terms()
        },
    );
    add_consent(
        room,
        &consent("zadanie", "klientka", "zadanie_rozpoczecia_pracy"),
    );
    sprawa_umowy::wpisz_zdarzenie_umowy(
        room,
        &WpiszZdarzenieUmowyParams {
            id: "zdarzenie-zadanie".to_string(),
            umowa_id: "umowa".to_string(),
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
    stage(
        room,
        &stage_change("przygotowanie_oferty", "2026-11-04T13:30:00Z"),
    );
    add_visibility(room, &visibility_version("widocznosc"));

    // The price: the office's proposal from comparable sales, the price the client approved and
    // her lowest price.
    oferta_ceny::wpisz_propozycje_ceny(
        room,
        &WpiszPropozycjeCenyParams {
            id: "propozycja".to_string(),
            dotyczy: DotyczyCeny::Sprzedaz,
            kwota: 64_000_000,
            uzasadnienie: "pięć transakcji z ostatniego roku na tym samym osiedlu".to_string(),
            utworzono: "2026-11-04T13:40:00Z".to_string(),
            wyliczenie_sciezka: "ceny/propozycja.json".to_string(),
            wyliczenie_skrot: "sha256-propozycja".to_string(),
        },
    )
    .expect("add the price proposal");
    oferta_ceny::dopisz_porownanie_transakcji(
        room,
        &DopiszPorownanieTransakcjiParams {
            propozycja_id: "propozycja".to_string(),
            data_transakcji: "2026-03-14".to_string(),
            cena: 61_500_000,
            powierzchnia: Some(4710),
            roznice: Some("parter".to_string()),
            zrodlo: "rejestr cen nieruchomości, Wrocław".to_string(),
        },
    )
    .expect("add a comparable sale");
    add_offer_price(room, &offer_price("cena"));
    add_consent(
        room,
        &client_consent(
            "zgoda-granice",
            "granice_negocjacji",
            "oferta_ceny_granice",
            "granice",
        ),
    );
    oferta_ceny::wpisz_granice_negocjacji(
        room,
        &WpiszGraniceNegocjacjiParams {
            id: "granice".to_string(),
            dotyczy: DotyczyCeny::Sprzedaz,
            najnizsza_kwota: 63_000_000,
            warunki: Some("wydanie lokalu najpóźniej w lutym".to_string()),
            zgoda_id: "zgoda-granice".to_string(),
            zastepuje_id: None,
        },
    )
    .expect("add the negotiation limits");

    // The advertisement, approved by the broker before its first publication.
    draft_ad(
        room,
        &ad("ogloszenie"),
        &DopiszWersjeOgloszeniaParams {
            wskazniki_energetyczne: Some("EP 120 kWh/(m²·rok)".to_string()),
            ..ad_version("ogloszenie", 1)
        },
    );
    approved_other(
        room,
        &other_decision(
            "pierwsza-publikacja",
            "oferta_ogloszenia_wpisy",
            "ogloszenie",
        ),
    );
    publish(room, "ogloszenie", PUBLISHED);
    oferta_ogloszenia::odnotuj_numer_ogloszenia(
        room,
        &OdnotujNumerOgloszeniaParams {
            numer_ogloszenia: "OT-778812".to_string(),
            id: "ogloszenie".to_string(),
        },
    )
    .expect("record the number of the ad");
    oferta_ogloszenia::wpisz_statystyki_ogloszenia(
        room,
        &WpiszStatystykiOgloszeniaParams {
            ogloszenie_id: "ogloszenie".to_string(),
            odczytano: "2026-11-09T08:00:00Z".to_string(),
            wyswietlenia: 412,
            zapytania: 3,
        },
    )
    .expect("add the statistics of the ad");
    stage(
        room,
        &stage_change("oferta_aktywna", "2026-11-05T10:00:00Z"),
    );

    // An inquiry, the information on personal data, the answer and a viewing.
    add_interested(room, &interested("zainteresowany"));
    add_inflow(
        room,
        &WpiszWplywParams {
            nadawca: "Piotr <piotr@example.pl>".to_string(),
            kanal: WplywyKanal::Portal,
            ..inflow("zapytanie")
        },
    );
    oferta_zainteresowani::wpisz_zapytanie(
        room,
        &WpiszZapytanieParams {
            osoba_id: "zainteresowany".to_string(),
            wplyw_id: "zapytanie".to_string(),
        },
    )
    .expect("add the inquiry");
    send_privacy_notice(room, "informacja-piotr");
    oferta_zainteresowani::odnotuj_informacje_dla_zainteresowanego(
        room,
        &OdnotujInformacjeDlaZainteresowanegoParams {
            informacja_wysylka_id: Some("informacja-piotr".to_string()),
            informacja_od_recepcji: None,
            id: "zainteresowany".to_string(),
        },
    )
    .expect("record the information on personal data");
    oferta_zainteresowani::odnotuj_odpowiedz_na_zapytanie(
        room,
        &OdnotujOdpowiedzNaZapytanieParams {
            odpowiedz_wysylka_id: "informacja-piotr".to_string(),
            id: 1,
        },
    )
    .expect("record the answer to the inquiry");
    oferta_zainteresowani::umow_prezentacje(
        room,
        &UmowPrezentacjeParams {
            id: "prezentacja".to_string(),
            termin: "2026-11-07T17:00:00Z".to_string(),
            posrednik: "tozsamosc-posrednika".to_string(),
            kalendarz_odnosnik: None,
            otwiera: Otwiera::Posrednik,
        },
    )
    .expect("arrange the viewing");
    oferta_zainteresowani::dopisz_uczestnika_prezentacji(
        room,
        &DopiszUczestnikaPrezentacjiParams {
            prezentacja_id: "prezentacja".to_string(),
            osoba_id: "zainteresowany".to_string(),
            adres_podano: Some("2026-11-06T12:00:00Z".to_string()),
        },
    )
    .expect("add a participant of the viewing");
    oferta_zainteresowani::odnotuj_obecnosc_na_prezentacji(
        room,
        &OdnotujObecnoscNaPrezentacjiParams {
            przyszedl: true,
            id: 1,
        },
    )
    .expect("record the presence at the viewing");
    oferta_zainteresowani::odnotuj_przebieg_prezentacji(
        room,
        &OdnotujPrzebiegPrezentacjiParams {
            przebieg: "Pytał o koszty ogrzewania i o miejsce postojowe.".to_string(),
            id: "prezentacja".to_string(),
        },
    )
    .expect("record the course of the viewing");
    oferta_zainteresowani::odnotuj_wynik_zapytania(
        room,
        &OdnotujWynikZapytaniaParams {
            skonczylo_sie_prezentacja: true,
            id: 1,
        },
    )
    .expect("record the result of the inquiry");

    // The client lowers the price; the ad gets a new version with it.
    add_offer_price(
        room,
        &WpiszCeneOfertowaParams {
            kwota: 64_500_000,
            obowiazuje_od: "2026-11-08T09:00:00Z".to_string(),
            poprzednia_id: Some("cena".to_string()),
            powod: Some("dwie prezentacje bez oferty".to_string()),
            ..offer_price("nowa-cena")
        },
    );
    add_ad_version(
        room,
        &DopiszWersjeOgloszeniaParams {
            cena: 64_500_000,
            od: "2026-11-08T10:00:00Z".to_string(),
            wskazniki_energetyczne: Some("EP 120 kWh/(m²·rok)".to_string()),
            ..ad_version("ogloszenie", 2)
        },
    );
    oferta_ceny::odnotuj_cene_w_ogloszeniach(
        room,
        &OdnotujCeneWOgloszeniachParams {
            w_ogloszeniach_od: "2026-11-08".to_string(),
            id: "nowa-cena".to_string(),
        },
    )
    .expect("record the new price in the ads");

    // The chain of offers: the buyer offers, the client counters, the buyer counters again and
    // the client accepts; every reply waits for the human's consent.
    add_interested_offer(room, &interested_offer("oferta-1"));
    client_decision(room, "oferta-1", DecyzjaKlienta::Kontroferta);
    oferta_oferty::wpisz_kontroferte_klienta(
        room,
        &WpiszKontroferteKlientaParams {
            id: "kontroferta".to_string(),
            dotyczy: DotyczyOferty::Sprzedaz,
            osoba_id: "zainteresowany".to_string(),
            odpowiada_na_id: "oferta-1".to_string(),
            wpisano: "2026-11-10T11:00:00Z".to_string(),
            kwota: 64_000_000,
            zadatek_albo_kaucja: None,
            termin_umowy: None,
            termin_wydania: None,
            warunki_zastrzezone: None,
            wiaze_do: Some("2026-11-13".to_string()),
            sposob_zlozenia: None,
            zgoda_id: "decyzja-oferta-1".to_string(),
        },
    )
    .expect("add the client's counteroffer");
    let first_reply = reply_awaiting_consent(room, "oferta-1");
    send(
        room,
        &OdnotujWyslanieParams {
            decyzja_id: Some(format!("wyslanie-{first_reply}")),
            wyslano: "2026-11-10T12:00:00Z".to_string(),
            ..sending(&first_reply)
        },
    );
    add_inflow(
        room,
        &WpiszWplywParams {
            nadawca: "Piotr <piotr@example.pl>".to_string(),
            ..inflow("odpowiedz-piotra")
        },
    );
    oferta_oferty::odnotuj_odpowiedz_zainteresowanego(
        room,
        &OdnotujOdpowiedzZainteresowanegoParams {
            odpowiedz_drugiej_strony: DecyzjaKlienta::Kontroferta,
            odpowiedz_wplyw_id: "odpowiedz-piotra".to_string(),
            id: "kontroferta".to_string(),
        },
    )
    .expect("record the reply of the interested person");
    add_interested_offer(
        room,
        &WpiszOferteZainteresowanegoParams {
            kwota: 62_500_000,
            odpowiada_na_id: Some("kontroferta".to_string()),
            wpisano: "2026-11-11T09:00:00Z".to_string(),
            ..interested_offer("oferta-2")
        },
    );
    client_decision(room, "oferta-2", DecyzjaKlienta::Przyjeta);
    let second_reply = reply_awaiting_consent(room, "oferta-2");
    send(
        room,
        &OdnotujWyslanieParams {
            decyzja_id: Some(format!("wyslanie-{second_reply}")),
            wyslano: "2026-11-11T12:00:00Z".to_string(),
            ..sending(&second_reply)
        },
    );
    stage(
        room,
        &stage_change("oferta_przyjeta", "2026-11-11T12:30:00Z"),
    );

    // The accepted offer goes to a transaction case; the ad comes down.
    oferta_ogloszenia::zdejmij_ogloszenie(
        room,
        &ZdejmijOgloszenieParams {
            zdjeto: "2026-11-11T13:00:00Z".to_string(),
            powod_zdjecia: "przyjęta oferta kupna".to_string(),
            id: "ogloszenie".to_string(),
        },
    )
    .expect("take the ad down");
    sprawa_powiazania::wpisz_powiazanie(
        room,
        &WpiszPowiazanieParams {
            rodzaj: RodzajPowiazania::SprawaZrodlowa,
            typ_teczki: Some("transakcja".to_string()),
            oznaczenie: Some("Transakcja 2026/012".to_string()),
            odnosnik: "pokoj-transakcji-012".to_string(),
            wpisal: "tozsamosc-kierownika-posrednictwa".to_string(),
            wpisano: "2026-11-11T14:00:00Z".to_string(),
        },
    )
    .expect("link the transaction case");
}

// The whole case ----------------------------------------------------------------------------------

/// Przebieg oferty: od przyjęcia przez umowę, cenę, ogłoszenie i prezentację do przyjętej oferty
#[test]
fn whole_case() {
    let room = new_room();

    full_case(&room);
    end_contract(&room, "umowa", "transakcja doszła do skutku");
    stage(
        &room,
        &stage_change("koniec_oferty", "2026-12-20T09:00:00Z"),
    );
    stage(&room, &stage_change("zamknieta", "2026-12-21T09:00:00Z"));

    let stages = sprawa_metryka::etapy(&room).expect("read the stages");
    let etapy: Vec<&str> = stages.iter().map(|row| row.etap.as_str()).collect();
    assert_eq!(
        etapy,
        [
            "przyjeta",
            "weryfikacja_klienta",
            "umowa_posrednictwa",
            "przygotowanie_oferty",
            "oferta_aktywna",
            "oferta_przyjeta",
            "koniec_oferty",
            "zamknieta",
        ]
    );

    let current_prices =
        oferta_ceny::ceny_ofertowe_biezace(&room).expect("read the current prices");
    let current_price_ids: Vec<&str> = current_prices.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(current_price_ids, ["nowa-cena"]);

    let current_visibility = oferta_widocznosc::biezaca_widocznosc(&room)
        .expect("read the current visibility")
        .expect("the offer has a visibility");
    assert_eq!(current_visibility.widocznosc, Widocznosc::Publiczna);

    let newest_versions =
        oferta_ogloszenia::najnowsze_wersje_ogloszen(&room).expect("read the newest versions");
    let newest_numbers: Vec<i64> = newest_versions.iter().map(|row| row.numer).collect();
    assert_eq!(newest_numbers, [2]);

    let offers = oferta_oferty::oferty(&room).expect("read the offers");
    let directions: Vec<KierunekOferty> = offers.iter().map(|row| row.kierunek).collect();
    assert_eq!(
        directions,
        [
            KierunekOferty::OdZainteresowanego,
            KierunekOferty::OdKlienta,
            KierunekOferty::OdZainteresowanego,
        ]
    );

    let to_present =
        oferta_oferty::oferty_do_przedstawienia(&room).expect("read the offers to present");
    assert!(to_present.is_empty(), "{to_present:?}");
    assert_eq!(offers[2].decyzja_klienta, Some(DecyzjaKlienta::Przyjeta));

    let ads = oferta_ogloszenia::ogloszenia(&room).expect("read the ads");
    let taken_down: Vec<bool> = ads.iter().map(|row| row.zdjeto.is_some()).collect();
    assert_eq!(taken_down, [true]);
}

// oferta-reguly -----------------------------------------------------------------------------------

/// The parameters of the record of the offer case Oferta 2026/041 in a room of the type
/// `typ_pokoju`.
fn case_record(typ_pokoju: &str) -> ZalozMetrykeParams {
    ZalozMetrykeParams {
        oznaczenie: "Oferta 2026/041, ul. Polna 7 m. 12".to_string(),
        typ_pokoju: typ_pokoju.to_string(),
        przyjal: "tozsamosc-posrednika".to_string(),
        przyjeto: "2026-11-02T09:00:00Z".to_string(),
    }
}

/// Reguły: teczka oferty ma typ pokoju oferta
#[test]
fn room_type() {
    let room = new_room();
    expect::refused(
        sprawa_metryka::zaloz_metryke(&room, &case_record("najem")),
        "teczka oferty ma typ pokoju oferta",
    );

    sprawa_metryka::zaloz_metryke(&room, &case_record("oferta")).expect("open the offer case");
}

/// Reguły: etapy oferty idą po kolei, a przyjęta oferta może wrócić do aktywnej
#[test]
fn stage_transitions() {
    let room = new_room();
    offer_case(&room, &brokerage_terms());
    let rule = "etap oferty przechodzi";

    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change_by_a("umowa_posrednictwa", "2026-11-04"),
        ),
        rule,
    );
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &stage_change_by_a("umowa", "2026-11-04")),
        rule,
    );
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &stage_change_by_a("koniec_oferty", "2026-11-04")),
        rule,
    );

    for etap in [
        "weryfikacja_klienta",
        "umowa_posrednictwa",
        "przygotowanie_oferty",
    ] {
        stage(&room, &stage_change(etap, STAGE_CHANGED));
    }
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change_by_a("oferta_przyjeta", "2026-11-05"),
        ),
        rule,
    );

    for etap in [
        "oferta_aktywna",
        "oferta_przyjeta",
        "oferta_aktywna",
        "koniec_oferty",
    ] {
        stage(&room, &stage_change(etap, STAGE_CHANGED));
    }
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change_by_a("oferta_aktywna", "2026-11-06"),
        ),
        rule,
    );
    stage(&room, &stage_change("zamknieta", STAGE_CHANGED));
}

/// Reguły: przerwać wolno każdy etap przed zamknięciem; z przerwania nie ma skoku do innego etapu
#[test]
fn stage_interrupted() {
    let room = new_room();
    offer_case(&room, &brokerage_terms());
    stage(&room, &stage_change("weryfikacja_klienta", STAGE_CHANGED));
    approved_other(
        &room,
        &decision_request("przerwanie", RodzajDecyzji::PrzerwanieSprawy),
    );

    stage(
        &room,
        &stage_change_by_decision("przerwana", STAGE_CHANGED, "przerwanie"),
    );
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change_by_a("oferta_aktywna", "2026-11-05"),
        ),
        "etap oferty przechodzi",
    );
    stage(&room, &stage_change("zamknieta", STAGE_CHANGED));
}

/// Reguły: wznowiona oferta wraca do etapu sprzed przerwania, który dalej wymaga zawartej umowy
#[test]
fn stage_resumed() {
    let room = new_room();
    offer_case(&room, &brokerage_terms());
    for etap in [
        "weryfikacja_klienta",
        "umowa_posrednictwa",
        "przygotowanie_oferty",
    ] {
        stage(&room, &stage_change(etap, STAGE_CHANGED));
    }
    approved_decision(
        &room,
        &decision_request("przerwanie", RodzajDecyzji::PrzerwanieSprawy),
        "2026-11-03T12:00:00Z",
    );
    stage(
        &room,
        &stage_change_by_decision("przerwana", STAGE_CHANGED, "przerwanie"),
    );
    approved_decision(
        &room,
        &decision_request("wznowienie", RodzajDecyzji::WznowienieSprawy),
        "2026-11-05T10:00:00Z",
    );

    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change_by_decision("oferta_aktywna", "2026-11-05T11:00:00Z", "wznowienie"),
        ),
        "etap oferty przechodzi",
    );
    stage(
        &room,
        &stage_change_by_decision("przygotowanie_oferty", "2026-11-05T11:00:00Z", "wznowienie"),
    );

    approved_decision(
        &room,
        &decision_request("drugie-przerwanie", RodzajDecyzji::PrzerwanieSprawy),
        "2026-11-03T12:00:00Z",
    );
    stage(
        &room,
        &stage_change_by_decision("przerwana", "2026-11-06T09:00:00Z", "drugie-przerwanie"),
    );
    end_contract(&room, "umowa", "wypowiedzenie przez klientkę");
    approved_decision(
        &room,
        &decision_request("drugie-wznowienie", RodzajDecyzji::WznowienieSprawy),
        "2026-11-06T10:00:00Z",
    );
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change_by_decision(
                "przygotowanie_oferty",
                "2026-11-06T11:00:00Z",
                "drugie-wznowienie",
            ),
        ),
        "wymagają zawartej umowy pośrednictwa",
    );
    stage(&room, &stage_change("zamknieta", "2026-11-06T12:00:00Z"));
}

/// Reguły: przygotowanie oferty i oferta aktywna wymagają zawartej umowy pośrednictwa
#[test]
fn stage_needs_contract() {
    let room = new_room();
    open_case(&room, "oferta");
    add_client(&room);
    add_flat(&room, &flat("lokal"));
    for etap in ["weryfikacja_klienta", "umowa_posrednictwa"] {
        stage(&room, &stage_change(etap, STAGE_CHANGED));
    }
    let rule = "wymagają zawartej umowy pośrednictwa";

    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change_by_a("przygotowanie_oferty", "2026-11-04"),
        ),
        rule,
    );

    brokerage_contract(&room, &brokerage_terms());
    stage(&room, &stage_change("przygotowanie_oferty", STAGE_CHANGED));
    end_contract(&room, "umowa", "wypowiedzenie przez klientkę");
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change_by_a("oferta_aktywna", "2026-11-05"),
        ),
        rule,
    );
}

/// The parameters of the role `rola` of „osoba”.
fn role_of_person(rola: &str) -> NadajRoleUczestnikowiParams {
    NadajRoleUczestnikowiParams {
        uczestnik_id: "osoba".to_string(),
        rola: rola.to_string(),
        dotyczy_uczestnik_id: None,
    }
}

/// Reguły: role w teczce oferty to klient, współwłaściciel i małżonek
#[test]
fn roles() {
    let room = new_room();
    add_person(&room, &person("osoba"));

    expect::refused(
        sprawa_uczestnicy::nadaj_role_uczestnikowi(&room, &role_of_person("druga_strona")),
        "rola w teczce oferty",
    );
    sprawa_uczestnicy::nadaj_role_uczestnikowi(&room, &role_of_person("wspolwlasciciel"))
        .expect("give the role of a co-owner");
}

/// The parameters of the client's consent `rodzaj` signed in the statement „podpis”.
fn consent_fields(rodzaj: &str) -> WpiszZgodeParams {
    WpiszZgodeParams {
        id: format!("zgoda-{rodzaj}"),
        uczestnik_id: "klientka".to_string(),
        rodzaj: rodzaj.to_string(),
        tresc: "Zatwierdzam opis.".to_string(),
        zlozono: "2026-11-04T13:00:00Z".to_string(),
        dokument_id: Some("podpis".to_string()),
        wplyw_id: None,
        trwaly_nosnik: false,
        dotyczy_tabela: None,
        dotyczy_id: None,
        zastepuje_id: None,
    }
}

/// Reguły: zgody w teczce oferty to zgody oferty i zgody rdzenia
#[test]
fn consent_kinds() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    add_received_document(
        &room,
        &received_document("podpis", RodzajDokumentu::Oswiadczenie),
    );

    expect::refused(
        sprawa_zgody::wpisz_zgode(&room, &consent_fields("potwierdzenie_profilu")),
        "zgoda w teczce oferty",
    );
    sprawa_zgody::wpisz_zgode(&room, &consent_fields("zatwierdzenie_opisu"))
        .expect("add the consent to the description");
}

/// The parameters of a draft contract `contract_id` of the kind `rodzaj` with nothing but its
/// required fields.
fn bare_contract(contract_id: &str, rodzaj: RodzajUmowy) -> PrzygotujUmoweParams {
    PrzygotujUmoweParams {
        id: contract_id.to_string(),
        rodzaj,
        nieruchomosc_id: None,
        poczatek: None,
        koniec: None,
        okres_wypowiedzenia: None,
        obie_strony: false,
        wylacznosc: false,
        wynagrodzenie_opis: None,
        wynagrodzenie_zdarzenie: None,
    }
}

/// Reguły: umowa w teczce oferty to tylko umowa pośrednictwa
#[test]
fn contract_kind() {
    let room = new_room();
    let rule = "umowa w teczce oferty to umowa pośrednictwa";

    expect::refused(
        sprawa_umowy::przygotuj_umowe(
            &room,
            &bare_contract("zarzadzanie", RodzajUmowy::Zarzadzanie),
        ),
        rule,
    );
    sprawa_umowy::przygotuj_umowe(&room, &bare_contract("umowa", RodzajUmowy::Posrednictwo))
        .expect("prepare the brokerage contract");
    expect::refused(
        room.execute("UPDATE sprawa_umowy_wpisy SET rodzaj = 'zarzadzanie'", &[]),
        "rodzaj umowy się nie zmienia",
    );
}

/// Reguły: teczka oferty nie ma rachunków ani wypłat
#[test]
fn no_money() {
    let room = new_room();
    add_person(&room, &person("zainteresowany"));
    add_received_document(&room, &received_document("pismo", RodzajDokumentu::Pismo));
    let rule = "teczka oferty nie przyjmuje pieniędzy";

    expect::refused(
        sprawa_rachunki::wpisz_rachunek(
            &room,
            &WpiszRachunekParams {
                id: "rachunek".to_string(),
                uczestnik_id: "zainteresowany".to_string(),
                numer: "PL61109010140000071219812874".to_string(),
                posiadacz: "Piotr".to_string(),
                cel: CelRachunku::ZwrotKaucji,
                zrodlo_dokument_id: Some("pismo".to_string()),
                zrodlo_wplyw_id: None,
                wpisano: "2026-11-03T08:00:00Z".to_string(),
            },
        ),
        rule,
    );
    expect::refused(
        sprawa_wyplaty::przygotuj_wyplate(
            &room,
            &PrzygotujWyplateParams {
                id: "wyplata".to_string(),
                podstawa_tabela: "oferta_oferty_wpisy".to_string(),
                podstawa_id: "oferta".to_string(),
                kwota: 100,
                wyliczenie_sciezka: "w.json".to_string(),
                wyliczenie_skrot: "sha256-w".to_string(),
                odbiorca_uczestnik_id: "zainteresowany".to_string(),
                rachunek_id: "rachunek".to_string(),
                utworzono: "2026-11-03T08:00:00Z".to_string(),
            },
        ),
        rule,
    );
}

// oferta-widocznosc -------------------------------------------------------------------------------

/// Widoczność: pierwsza wersja powstaje przy zawartej umowie i obowiązuje od zawarcia
#[test]
fn visibility_first_version() {
    let room = new_room();
    open_case(&room, "oferta");
    add_client(&room);
    add_flat(&room, &flat("lokal"));
    sprawa_umowy::przygotuj_umowe(&room, &bare_contract("umowa", RodzajUmowy::Posrednictwo))
        .expect("prepare the brokerage contract");
    let rule = "pierwsza wersja widoczności powstaje przy zawartej umowie";

    expect::refused(
        oferta_widocznosc::wpisz_wersje_widocznosci(&room, &visibility_version("widocznosc")),
        rule,
    );

    end_contract(&room, "umowa", "klientka nie podpisała");
    brokerage_contract(
        &room,
        &BrokerageTerms {
            contract_id: "umowa-2",
            ..brokerage_terms()
        },
    );
    expect::refused(
        oferta_widocznosc::wpisz_wersje_widocznosci(
            &room,
            &WpiszWersjeWidocznosciParams {
                umowa_id: "umowa-2".to_string(),
                od: "2026-11-05".to_string(),
                ..visibility_version("widocznosc")
            },
        ),
        rule,
    );
    add_visibility(
        &room,
        &WpiszWersjeWidocznosciParams {
            umowa_id: "umowa-2".to_string(),
            ..visibility_version("widocznosc")
        },
    );
}

/// Widoczność: oferta niepubliczna ma granice zgody, a zmianę wpisuje się z aneksu
#[test]
fn visibility_changed_by_annex() {
    let room = new_room();
    offer_case(&room, &brokerage_terms());
    expect::refused(
        oferta_widocznosc::wpisz_wersje_widocznosci(
            &room,
            &WpiszWersjeWidocznosciParams {
                widocznosc: Widocznosc::Niepubliczna,
                ..visibility_version("widocznosc")
            },
        ),
        "niepubliczna_ma_granice_zgody",
    );
    add_visibility(&room, &visibility_version("widocznosc"));
    let rule = "kolejna wersja widoczności wskazuje aneks";

    expect::refused(
        oferta_widocznosc::wpisz_wersje_widocznosci(
            &room,
            &WpiszWersjeWidocznosciParams {
                od: "2026-11-20".to_string(),
                ..visibility_version("bez-aneksu")
            },
        ),
        rule,
    );

    add_received_document(
        &room,
        &received_document("aneks", RodzajDokumentu::UmowaZBiurem),
    );
    sprawa_umowy::wpisz_zdarzenie_umowy(
        &room,
        &WpiszZdarzenieUmowyParams {
            id: "aneks".to_string(),
            umowa_id: "umowa".to_string(),
            rodzaj: RodzajZdarzeniaUmowy::Aneks,
            chwila: "2026-11-20T10:00:00Z".to_string(),
            opis: Some("oferta niepubliczna".to_string()),
            dokument_id: Some("aneks".to_string()),
            wysylka_id: None,
            wplyw_id: None,
            zgoda_id: None,
        },
    )
    .expect("add the annex");
    expect::refused(
        oferta_widocznosc::wpisz_wersje_widocznosci(
            &room,
            &WpiszWersjeWidocznosciParams {
                aneks_zdarzenie_id: Some("aneks".to_string()),
                od: "2026-11-21T10:00:00Z".to_string(),
                ..visibility_version("inna-chwila")
            },
        ),
        rule,
    );
    add_visibility(
        &room,
        &WpiszWersjeWidocznosciParams {
            widocznosc: Widocznosc::Niepubliczna,
            granice_zgody: Some(
                "tylko klientom z potwierdzonym finansowaniem, bez zdjęć wnętrz".to_string(),
            ),
            aneks_zdarzenie_id: Some("aneks".to_string()),
            od: "2026-11-20T10:00:00Z".to_string(),
            ..visibility_version("niepubliczna")
        },
    );
    let current_visibility = oferta_widocznosc::biezaca_widocznosc(&room)
        .expect("read the current visibility")
        .expect("the offer has a visibility");
    assert_eq!(current_visibility.id, "niepubliczna");

    expect::refused(
        room.execute(
            "UPDATE oferta_widocznosc_wersje SET widocznosc = 'publiczna' WHERE id = 'niepubliczna'",
            &[],
        ),
        "wersji widoczności się nie zmienia",
    );
}

// oferta-nieruchomosc -----------------------------------------------------------------------------

/// Nieruchomość: świadectwo, które jest, ma dokument z ważnością i wskaźniki
#[test]
fn property_certificate() {
    let room = new_room();
    add_flat(&room, &flat("lokal"));
    add_received_document(&room, &received_document("pismo", RodzajDokumentu::Pismo));
    add_received_document(
        &room,
        &received_document("bez-waznosci", RodzajDokumentu::SwiadectwoEnergetyczne),
    );
    add_received_document(
        &room,
        &energy_certificate("swiadectwo", "2020-05-10", "2030-05-10"),
    );
    let rule = "świadectwo wskazuje otrzymany dokument swiadectwo_energetyczne";

    expect::refused(
        oferta_nieruchomosc::wpisz_opis_nieruchomosci(
            &room,
            &WpiszOpisNieruchomosciParams {
                swiadectwo_dokument_id: Some("swiadectwo".to_string()),
                ..empty_description(OpisSwiadectwo::Jest)
            },
        ),
        "swiadectwo_jest_z_dokumentem_i_wskaznikami",
    );
    expect::refused(
        oferta_nieruchomosc::wpisz_opis_nieruchomosci(&room, &certificate_description("pismo")),
        rule,
    );
    expect::refused(
        oferta_nieruchomosc::wpisz_opis_nieruchomosci(
            &room,
            &certificate_description("bez-waznosci"),
        ),
        rule,
    );
    oferta_nieruchomosc::wpisz_opis_nieruchomosci(&room, &certificate_description("swiadectwo"))
        .expect("add the description with the certificate");
    expect::refused(
        oferta_nieruchomosc::zmien_opis_nieruchomosci(
            &room,
            &ZmienOpisNieruchomosciParams {
                swiadectwo_dokument_id: Some("pismo".to_string()),
                wskazniki_energetyczne: Some("EP 120 kWh/(m²·rok)".to_string()),
                ..empty_description_change(OpisSwiadectwo::Jest)
            },
        ),
        rule,
    );

    expect::refused(
        sprawa_dokumenty::odrzuc_dokument(
            &room,
            &OdrzucDokumentParams {
                powod_odrzucenia: "nieczytelny".to_string(),
                id: "swiadectwo".to_string(),
            },
        ),
        "dokument świadectwa z opisu nieruchomości",
    );
    oferta_nieruchomosc::zmien_opis_nieruchomosci(
        &room,
        &empty_description_change(OpisSwiadectwo::Brak),
    )
    .expect("record that there is no certificate");
    expect::refused(
        room.execute(
            "UPDATE oferta_nieruchomosc_opis SET nieruchomosc_id = 'inny-lokal'",
            &[],
        ),
        "opis nie przechodzi na inną nieruchomość",
    );
}

/// Nieruchomość: rozbieżność z księgą ma opis, a przeglądania się nie zmienia
#[test]
fn property_land_register_lookup() {
    let room = new_room();
    add_flat(&room, &flat("lokal"));
    let lookup = WpiszPrzegladanieKsiegiParams {
        id: "przegladanie".to_string(),
        nieruchomosc_id: "lokal".to_string(),
        numer_kw: "WR1K/00012345/6".to_string(),
        przegladano: "2026-11-03T09:00:00Z".to_string(),
        wlasciciele_dzial_ii: "Zofia Kowalska (spadkodawczyni)".to_string(),
        wpisy_dzial_iii: None,
        wpisy_dzial_iv: None,
        rozbieznosc: true,
        rozbieznosc_opis: None,
        wydruk_sciezka: None,
        wydruk_skrot: None,
    };

    expect::refused(
        oferta_nieruchomosc::wpisz_przegladanie_ksiegi(&room, &lookup),
        "rozbieznosc_ma_opis",
    );
    oferta_nieruchomosc::wpisz_przegladanie_ksiegi(
        &room,
        &WpiszPrzegladanieKsiegiParams {
            rozbieznosc_opis: Some("w księdze wpisana spadkodawczyni klientki".to_string()),
            ..lookup
        },
    )
    .expect("add the lookup with the discrepancy");
    expect::refused(
        room.execute(
            "UPDATE oferta_nieruchomosc_przegladania SET rozbieznosc = 0, rozbieznosc_opis = NULL",
            &[],
        ),
        "przeglądania księgi się nie zmienia",
    );
}

// oferta-ceny -------------------------------------------------------------------------------------

/// Cena ofertowa ma zgodę zatwierdzenie_ceny klienta, która jej dotyczy
#[test]
fn price_needs_client_consent() {
    let room = new_room();
    add_client(&room);
    add_person(
        &room,
        &WpiszUczestnikaParams {
            imiona: Some("Jan".to_string()),
            nazwisko: Some("Kowalski".to_string()),
            ..person("maz")
        },
    );
    let rule = "cena ofertowa ma zgodę zatwierdzenie_ceny klienta";
    let price_with_consent = |zgoda_id: &str| WpiszCeneOfertowaParams {
        zgoda_id: zgoda_id.to_string(),
        ..offer_price("cena")
    };

    add_consent(
        &room,
        &WpiszZgodeParams {
            uczestnik_id: "maz".to_string(),
            ..client_consent(
                "zgoda-meza",
                "zatwierdzenie_ceny",
                "oferta_ceny_ofertowe",
                "cena",
            )
        },
    );
    expect::refused(
        oferta_ceny::wpisz_cene_ofertowa(&room, &price_with_consent("zgoda-meza")),
        rule,
    );
    add_consent(
        &room,
        &client_consent(
            "zgoda-opisu",
            "zatwierdzenie_opisu",
            "oferta_ceny_ofertowe",
            "cena",
        ),
    );
    expect::refused(
        oferta_ceny::wpisz_cene_ofertowa(&room, &price_with_consent("zgoda-opisu")),
        rule,
    );
    add_consent(
        &room,
        &client_consent(
            "zgoda-innej",
            "zatwierdzenie_ceny",
            "oferta_ceny_ofertowe",
            "inna-cena",
        ),
    );
    expect::refused(
        oferta_ceny::wpisz_cene_ofertowa(&room, &price_with_consent("zgoda-innej")),
        rule,
    );

    add_consent(
        &room,
        &client_consent(
            "zgoda",
            "zatwierdzenie_ceny",
            "oferta_ceny_ofertowe",
            "cena",
        ),
    );
    oferta_ceny::wpisz_cene_ofertowa(&room, &price_with_consent("zgoda"))
        .expect("add the offer price");
}

/// The parameters of a change of the price „cena” to 645 000 zł from 2026-11-08 at 09:00, with
/// its reason and the client's consent `zgoda-<id>`.
fn price_change(price_id: &str) -> WpiszCeneOfertowaParams {
    WpiszCeneOfertowaParams {
        kwota: 64_500_000,
        obowiazuje_od: "2026-11-08T09:00:00Z".to_string(),
        poprzednia_id: Some("cena".to_string()),
        powod: Some("dwie prezentacje bez oferty".to_string()),
        ..offer_price(price_id)
    }
}

/// Cena ofertowa: zmiana wskazuje obowiązującą cenę, ma powód i późniejszą chwilę
#[test]
fn price_change_chain() {
    let room = new_room();
    add_client(&room);
    add_offer_price(&room, &offer_price("cena"));
    for price_id in [
        "bez-poprzedniej",
        "bez-powodu",
        "wczesniejsza",
        "czynsz",
        "nowa",
        "od-starej",
    ] {
        add_consent(
            &room,
            &client_consent(
                &format!("zgoda-{price_id}"),
                "zatwierdzenie_ceny",
                "oferta_ceny_ofertowe",
                price_id,
            ),
        );
    }
    let rule = "zmiana ceny wskazuje obowiązującą cenę";

    expect::refused(
        oferta_ceny::wpisz_cene_ofertowa(
            &room,
            &WpiszCeneOfertowaParams {
                poprzednia_id: None,
                ..price_change("bez-poprzedniej")
            },
        ),
        rule,
    );
    expect::refused(
        oferta_ceny::wpisz_cene_ofertowa(
            &room,
            &WpiszCeneOfertowaParams {
                powod: None,
                ..price_change("bez-powodu")
            },
        ),
        "zmiana_ceny_ma_powod",
    );
    expect::refused(
        oferta_ceny::wpisz_cene_ofertowa(
            &room,
            &WpiszCeneOfertowaParams {
                obowiazuje_od: "2026-11-04T13:00:00Z".to_string(),
                ..price_change("wczesniejsza")
            },
        ),
        rule,
    );
    expect::refused(
        oferta_ceny::wpisz_cene_ofertowa(
            &room,
            &WpiszCeneOfertowaParams {
                dotyczy: DotyczyCeny::Najem,
                ..price_change("czynsz")
            },
        ),
        rule,
    );

    oferta_ceny::wpisz_cene_ofertowa(&room, &price_change("nowa")).expect("change the price");
    expect::refused(
        oferta_ceny::wpisz_cene_ofertowa(
            &room,
            &WpiszCeneOfertowaParams {
                obowiazuje_od: "2026-11-09".to_string(),
                ..price_change("od-starej")
            },
        ),
        rule,
    );

    let current_prices =
        oferta_ceny::ceny_ofertowe_biezace(&room).expect("read the current prices");
    let current_price_ids: Vec<&str> = current_prices.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(current_price_ids, ["nowa"]);
}

/// The parameters of the entry of the price „cena” into the ads on `w_ogloszeniach_od`.
fn price_in_ads(w_ogloszeniach_od: &str) -> OdnotujCeneWOgloszeniachParams {
    OdnotujCeneWOgloszeniachParams {
        w_ogloszeniach_od: w_ogloszeniach_od.to_string(),
        id: "cena".to_string(),
    }
}

/// Cena ofertowa: wejście do ogłoszeń wpisuje się raz, a reszty się nie zmienia
#[test]
fn price_in_ads_once() {
    let room = new_room();
    add_client(&room);
    add_offer_price(&room, &offer_price("cena"));

    expect::refused(
        oferta_ceny::odnotuj_cene_w_ogloszeniach(&room, &price_in_ads("2026-11-01")),
        "w_ogloszeniach_nie_przed_cena",
    );
    oferta_ceny::odnotuj_cene_w_ogloszeniach(&room, &price_in_ads("2026-11-05"))
        .expect("record the price in the ads");
    let rule = "ceny ofertowej się nie zmienia";
    expect::refused(
        oferta_ceny::odnotuj_cene_w_ogloszeniach(&room, &price_in_ads("2026-11-06")),
        rule,
    );
    add_consent(
        &room,
        &client_consent(
            "zgoda-od-razu",
            "zatwierdzenie_ceny",
            "oferta_ceny_ofertowe",
            "od-razu",
        ),
    );
    expect::refused(
        room.execute(
            "INSERT INTO oferta_ceny_ofertowe (id, dotyczy, kwota, zgoda_id, obowiazuje_od,
              poprzednia_id, powod, w_ogloszeniach_od)
            VALUES ('od-razu', 'sprzedaz', 1, 'zgoda-od-razu', '2026-11-09T09:00:00Z', 'cena',
              'obniżka', '2026-11-09')",
            &[],
        ),
        "cena ofertowa powstaje bez daty wejścia do ogłoszeń",
    );
    expect::refused(
        room.execute("UPDATE oferta_ceny_ofertowe SET kwota = 1", &[]),
        rule,
    );
}

/// The parameters of the client's lowest sale price of 630 000 zł, `limits_id`, with her consent
/// `zgoda_id`.
fn limits(limits_id: &str, zgoda_id: &str) -> WpiszGraniceNegocjacjiParams {
    WpiszGraniceNegocjacjiParams {
        id: limits_id.to_string(),
        dotyczy: DotyczyCeny::Sprzedaz,
        najnizsza_kwota: 63_000_000,
        warunki: None,
        zgoda_id: zgoda_id.to_string(),
        zastepuje_id: None,
    }
}

/// Granice negocjacji mają zgodę klienta i zastępują tylko obowiązujące granice
#[test]
fn negotiation_limits() {
    let room = new_room();
    add_client(&room);

    add_consent(
        &room,
        &client_consent(
            "zgoda-ceny",
            "zatwierdzenie_ceny",
            "oferta_ceny_granice",
            "granice",
        ),
    );
    expect::refused(
        oferta_ceny::wpisz_granice_negocjacji(&room, &limits("granice", "zgoda-ceny")),
        "granice negocjacji mają zgodę granice_negocjacji klienta",
    );
    for limits_id in ["granice", "nowe", "inne"] {
        add_consent(
            &room,
            &client_consent(
                &format!("zgoda-{limits_id}"),
                "granice_negocjacji",
                "oferta_ceny_granice",
                limits_id,
            ),
        );
    }
    oferta_ceny::wpisz_granice_negocjacji(&room, &limits("granice", "zgoda-granice"))
        .expect("add the negotiation limits");

    let rule = "nowe granice negocjacji zastępują obowiązujące";
    expect::refused(
        oferta_ceny::wpisz_granice_negocjacji(&room, &limits("nowe", "zgoda-nowe")),
        rule,
    );
    oferta_ceny::wpisz_granice_negocjacji(
        &room,
        &WpiszGraniceNegocjacjiParams {
            zastepuje_id: Some("granice".to_string()),
            ..limits("nowe", "zgoda-nowe")
        },
    )
    .expect("replace the negotiation limits");
    expect::refused(
        oferta_ceny::wpisz_granice_negocjacji(
            &room,
            &WpiszGraniceNegocjacjiParams {
                zastepuje_id: Some("granice".to_string()),
                ..limits("inne", "zgoda-inne")
            },
        ),
        rule,
    );

    let current_limits =
        oferta_ceny::granice_negocjacji_biezace(&room).expect("read the current limits");
    let current_limit_ids: Vec<&str> = current_limits.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(current_limit_ids, ["nowe"]);
    expect::refused(
        room.execute("UPDATE oferta_ceny_granice SET warunki = 'x'", &[]),
        "granic negocjacji się nie zmienia",
    );
}

/// Propozycji ceny i porównań transakcji się nie zmienia
#[test]
fn price_proposals_fixed() {
    let room = new_room();
    oferta_ceny::wpisz_propozycje_ceny(
        &room,
        &WpiszPropozycjeCenyParams {
            id: "propozycja".to_string(),
            dotyczy: DotyczyCeny::Sprzedaz,
            kwota: 64_000_000,
            uzasadnienie: "pięć transakcji".to_string(),
            utworzono: "2026-11-04T13:40:00Z".to_string(),
            wyliczenie_sciezka: "ceny/propozycja.json".to_string(),
            wyliczenie_skrot: "sha256-propozycja".to_string(),
        },
    )
    .expect("add the price proposal");
    oferta_ceny::dopisz_porownanie_transakcji(
        &room,
        &DopiszPorownanieTransakcjiParams {
            propozycja_id: "propozycja".to_string(),
            data_transakcji: "2026-03-14".to_string(),
            cena: 61_500_000,
            powierzchnia: None,
            roznice: None,
            zrodlo: "rejestr cen nieruchomości".to_string(),
        },
    )
    .expect("add a comparable sale");

    expect::refused(
        room.execute("UPDATE oferta_ceny_propozycje SET kwota = 1", &[]),
        "propozycji ceny się nie zmienia",
    );
    expect::refused(
        room.execute("UPDATE oferta_ceny_porownania SET cena = 1", &[]),
        "porównania transakcji się nie zmienia",
    );
}

// oferta-najem ------------------------------------------------------------------------------------

/// The parameters of the client's rental terms `terms_id`: a deposit of 6000 zł, an ordinary lease
/// for at least 12 months from 2026-12-01, run by the office, with her consent `zgoda-<id>`.
fn rental_terms_of(terms_id: &str) -> WpiszWarunkiNajmuParams {
    WpiszWarunkiNajmuParams {
        id: terms_id.to_string(),
        oplaty_obok_czynszu: None,
        rozliczenie_mediow: None,
        kaucja: Some(600_000),
        dopuszcza_zwykly: true,
        dopuszcza_okazjonalny: false,
        dopuszcza_instytucjonalny: false,
        wolny_od: Some("2026-12-01".to_string()),
        okres_min: Some("12 miesięcy".to_string()),
        okres_max: None,
        wyposazenie: None,
        liczba_osob: None,
        zwierzeta: None,
        palenie: None,
        prowadzenie_najmu: true,
        zgoda_id: format!("zgoda-{terms_id}"),
        zastepuje_id: None,
    }
}

/// Warunki najmu dopuszczają rodzaj najmu, mają zgodę klienta i zastępują obowiązujące
#[test]
fn rental_terms() {
    let room = new_room();
    add_client(&room);
    for terms_id in ["warunki", "nowe", "inne"] {
        add_consent(
            &room,
            &client_consent(
                &format!("zgoda-{terms_id}"),
                "warunki_najmu",
                "oferta_najem_warunki",
                terms_id,
            ),
        );
    }
    add_consent(
        &room,
        &client_consent(
            "zgoda-ceny",
            "zatwierdzenie_ceny",
            "oferta_najem_warunki",
            "warunki",
        ),
    );

    expect::refused(
        oferta_najem::wpisz_warunki_najmu(
            &room,
            &WpiszWarunkiNajmuParams {
                dopuszcza_zwykly: false,
                ..rental_terms_of("warunki")
            },
        ),
        "dopuszcza_rodzaj_najmu",
    );
    expect::refused(
        oferta_najem::wpisz_warunki_najmu(
            &room,
            &WpiszWarunkiNajmuParams {
                zgoda_id: "zgoda-ceny".to_string(),
                ..rental_terms_of("warunki")
            },
        ),
        "warunki najmu mają zgodę warunki_najmu klienta",
    );
    oferta_najem::wpisz_warunki_najmu(&room, &rental_terms_of("warunki"))
        .expect("add the rental terms");

    let rule = "nowe warunki najmu zastępują warunki, które obowiązują";
    expect::refused(
        oferta_najem::wpisz_warunki_najmu(&room, &rental_terms_of("nowe")),
        rule,
    );
    oferta_najem::wpisz_warunki_najmu(
        &room,
        &WpiszWarunkiNajmuParams {
            zastepuje_id: Some("warunki".to_string()),
            ..rental_terms_of("nowe")
        },
    )
    .expect("replace the rental terms");
    expect::refused(
        oferta_najem::wpisz_warunki_najmu(
            &room,
            &WpiszWarunkiNajmuParams {
                zastepuje_id: Some("warunki".to_string()),
                ..rental_terms_of("inne")
            },
        ),
        rule,
    );

    let current_terms = oferta_najem::warunki_najmu_biezace(&room)
        .expect("read the current rental terms")
        .expect("the offer has rental terms");
    assert_eq!(current_terms.id, "nowe");
    expect::refused(
        room.execute("UPDATE oferta_najem_warunki SET kaucja = 1", &[]),
        "warunków najmu się nie zmienia",
    );
}

// oferta-zainteresowani ---------------------------------------------------------------------------

/// Zainteresowany ma kontakt, a działający przez biuro wskazuje biuro
#[test]
fn interested_contact() {
    let room = new_room();

    expect::refused(
        oferta_zainteresowani::wpisz_zainteresowanego(
            &room,
            &WpiszZainteresowanegoParams {
                telefon: None,
                ..interested("zainteresowany")
            },
        ),
        "osoba_ma_kontakt",
    );
    expect::refused(
        oferta_zainteresowani::wpisz_zainteresowanego(
            &room,
            &WpiszZainteresowanegoParams {
                dziala_jako: DzialaJako::PrzezBiuro,
                ..interested("zainteresowany")
            },
        ),
        "przez_biuro_wskazuje_biuro",
    );
    add_interested(
        &room,
        &WpiszZainteresowanegoParams {
            dziala_jako: DzialaJako::PrzezBiuro,
            biuro: Some("Nieruchomości Odra".to_string()),
            dla_kogo: Some("kupujący".to_string()),
            ..interested("zainteresowany")
        },
    );
    oferta_zainteresowani::zmien_kontakt_zainteresowanego(
        &room,
        &ZmienKontaktZainteresowanegoParams {
            telefon: None,
            email: Some("piotr@example.pl".to_string()),
            id: "zainteresowany".to_string(),
        },
    )
    .expect("change the contact of the interested person");
}

/// The parameters of the information on personal data given to „zainteresowany”.
fn privacy_information(
    informacja_wysylka_id: Option<&str>,
    informacja_od_recepcji: Option<&str>,
) -> OdnotujInformacjeDlaZainteresowanegoParams {
    OdnotujInformacjeDlaZainteresowanegoParams {
        informacja_wysylka_id: informacja_wysylka_id.map(str::to_string),
        informacja_od_recepcji: informacja_od_recepcji.map(str::to_string),
        id: "zainteresowany".to_string(),
    }
}

/// Zainteresowany: informacja o danych to wysłana wysyłka RODO albo data od Recepcji, raz
#[test]
fn interested_privacy_notice() {
    let room = new_room();
    add_interested(&room, &interested("zainteresowany"));
    let rule = "informacja o danych to wysłana wysyłka ze znacznikiem informacja_rodo";
    let letter_to_piotr = |letter_id: &str| PrzygotujWysylkeParams {
        adresat_rodzaj: Some(WysylkiAdresatRodzaj::Inny),
        adresat_nazwa: Some("Piotr".to_string()),
        ..letter(letter_id)
    };

    prepare_letter(&room, &letter_to_piotr("bez-rodo"));
    send(&room, &sending("bez-rodo"));
    expect::refused(
        oferta_zainteresowani::odnotuj_informacje_dla_zainteresowanego(
            &room,
            &privacy_information(Some("bez-rodo"), None),
        ),
        rule,
    );
    prepare_letter(
        &room,
        &PrzygotujWysylkeParams {
            informacja_rodo: true,
            ..letter_to_piotr("niewyslana")
        },
    );
    expect::refused(
        oferta_zainteresowani::odnotuj_informacje_dla_zainteresowanego(
            &room,
            &privacy_information(Some("niewyslana"), None),
        ),
        rule,
    );

    send_privacy_notice(&room, "informacja");
    expect::refused(
        oferta_zainteresowani::odnotuj_informacje_dla_zainteresowanego(
            &room,
            &privacy_information(Some("informacja"), Some("2026-11-06")),
        ),
        "informacja_wysylka_albo_od_recepcji",
    );
    oferta_zainteresowani::odnotuj_informacje_dla_zainteresowanego(
        &room,
        &privacy_information(Some("informacja"), None),
    )
    .expect("record the information on personal data");
    expect::refused(
        oferta_zainteresowani::odnotuj_informacje_dla_zainteresowanego(
            &room,
            &privacy_information(None, Some("2026-11-06")),
        ),
        "informację o danych zainteresowanego wpisuje się raz",
    );
    add_interested(
        &room,
        &WpiszZainteresowanegoParams {
            informacja_od_recepcji: Some("2026-11-06".to_string()),
            ..interested("od-recepcji")
        },
    );
}

/// Zapytanie: odpowiedź przychodzi po informacji o danych, a odpowiedź i wynik raz
#[test]
fn inquiry_answer() {
    let room = new_room();
    add_interested(&room, &interested("zainteresowany"));
    add_inflow(
        &room,
        &WpiszWplywParams {
            nadawca: "Piotr".to_string(),
            kanal: WplywyKanal::Portal,
            ..inflow("zapytanie")
        },
    );
    oferta_zainteresowani::wpisz_zapytanie(
        &room,
        &WpiszZapytanieParams {
            osoba_id: "zainteresowany".to_string(),
            wplyw_id: "zapytanie".to_string(),
        },
    )
    .expect("add the inquiry");
    send_privacy_notice(&room, "odpowiedz");
    let answer = OdnotujOdpowiedzNaZapytanieParams {
        odpowiedz_wysylka_id: "odpowiedz".to_string(),
        id: 1,
    };
    let result = |skonczylo_sie_prezentacja: bool| OdnotujWynikZapytaniaParams {
        skonczylo_sie_prezentacja,
        id: 1,
    };

    expect::refused(
        oferta_zainteresowani::odnotuj_odpowiedz_na_zapytanie(&room, &answer),
        "odpowiedź na zapytanie wymaga informacji o danych zainteresowanego",
    );
    oferta_zainteresowani::odnotuj_informacje_dla_zainteresowanego(
        &room,
        &privacy_information(Some("odpowiedz"), None),
    )
    .expect("record the information on personal data");
    oferta_zainteresowani::odnotuj_odpowiedz_na_zapytanie(&room, &answer)
        .expect("record the answer to the inquiry");
    oferta_zainteresowani::odnotuj_wynik_zapytania(&room, &result(false))
        .expect("record the result of the inquiry");

    let rule = "zapytania się nie zmienia";
    expect::refused(
        oferta_zainteresowani::odnotuj_wynik_zapytania(&room, &result(true)),
        rule,
    );
    expect::refused(
        room.execute(
            "UPDATE oferta_zainteresowani_zapytania SET wplyw_id = 'inny'",
            &[],
        ),
        rule,
    );
    expect::refused(
        room.execute(
            "INSERT INTO oferta_zainteresowani_zapytania (osoba_id, wplyw_id,
              skonczylo_sie_prezentacja)
            VALUES ('zainteresowany', 'zapytanie', 1)",
            &[],
        ),
        "zapytanie powstaje bez odpowiedzi i wyniku",
    );
}

/// The parameters of a viewing at `termin` the client opens, `prezentacja-<termin>`.
fn viewing(termin: &str) -> UmowPrezentacjeParams {
    UmowPrezentacjeParams {
        id: format!("prezentacja-{termin}"),
        termin: termin.to_string(),
        posrednik: "tozsamosc-posrednika".to_string(),
        kalendarz_odnosnik: None,
        otwiera: Otwiera::Klient,
    }
}

/// Prezentacja: przy zawartej umowie, nie wcześniej niż praca od (art. 15 ust. 3 upk)
#[test]
fn viewing_after_work_starts() {
    let room = new_room();
    open_case(&room, "oferta");
    add_client(&room);
    add_flat(&room, &flat("lokal"));
    let rule =
        "prezentacja wymaga zawartej umowy pośrednictwa i odbywa się nie wcześniej niż praca od";

    expect::refused(
        oferta_zainteresowani::umow_prezentacje(&room, &viewing("2026-11-07T17:00:00Z")),
        rule,
    );

    // A consumer's contract off premises: work starts the day after the withdrawal period.
    brokerage_contract(
        &room,
        &BrokerageTerms {
            sposob_zawarcia: SposobZawarcia::NaOdleglosc,
            odstapienie_do: Some(WITHDRAWAL_ENDS),
            ..brokerage_terms()
        },
    );
    expect::refused(
        oferta_zainteresowani::umow_prezentacje(&room, &viewing("2026-11-18T17:00:00Z")),
        rule,
    );
    oferta_zainteresowani::umow_prezentacje(&room, &viewing("2026-11-19T17:00:00Z"))
        .expect("arrange the viewing after work starts");
}

/// Prezentacja: przebieg albo odwołanie raz, termin się nie zmienia, uczestnik raz
#[test]
fn viewing_record() {
    let room = new_room();
    offer_case(&room, &brokerage_terms());
    add_interested(&room, &interested("zainteresowany"));
    oferta_zainteresowani::umow_prezentacje(
        &room,
        &UmowPrezentacjeParams {
            id: "prezentacja".to_string(),
            ..viewing("2026-11-07T17:00:00Z")
        },
    )
    .expect("arrange the viewing");
    let participant = DopiszUczestnikaPrezentacjiParams {
        prezentacja_id: "prezentacja".to_string(),
        osoba_id: "zainteresowany".to_string(),
        adres_podano: None,
    };
    oferta_zainteresowani::dopisz_uczestnika_prezentacji(&room, &participant)
        .expect("add a participant of the viewing");
    let presence = |przyszedl: bool| OdnotujObecnoscNaPrezentacjiParams { przyszedl, id: 1 };

    expect::refused(
        oferta_zainteresowani::dopisz_uczestnika_prezentacji(&room, &participant),
        REPLACEMENT,
    );
    oferta_zainteresowani::odnotuj_podanie_adresu(
        &room,
        &OdnotujPodanieAdresuParams {
            adres_podano: "2026-11-06T12:00:00Z".to_string(),
            id: 1,
        },
    )
    .expect("record that the address was given");
    oferta_zainteresowani::odnotuj_obecnosc_na_prezentacji(&room, &presence(false))
        .expect("record the absence at the viewing");
    expect::refused(
        oferta_zainteresowani::odnotuj_obecnosc_na_prezentacji(&room, &presence(true)),
        "uczestnika prezentacji się nie zmienia",
    );

    let rule = "prezentacji się nie zmienia";
    expect::refused(
        room.execute(
            "UPDATE oferta_zainteresowani_prezentacje SET termin = '2026-11-08T17:00:00Z'",
            &[],
        ),
        rule,
    );

    // A writer outside the client can leave out the reason that `odwolaj_prezentacje` requires.
    // The table refuses the cancellation without one.
    let cancellation_without_reason = room.execute(
        "UPDATE oferta_zainteresowani_prezentacje
        SET odwolano = :odwolano,
          odwolal = :odwolal,
          powod_odwolania = :powod_odwolania
        WHERE id = :id",
        &[
            ("odwolano", text("2026-11-07T12:00:00Z")),
            ("odwolal", text("klientka")),
            ("powod_odwolania", SqlValue::Null),
            ("id", text("prezentacja")),
        ],
    );
    expect::refused(cancellation_without_reason, "odwolanie_kto_kiedy_dlaczego");

    oferta_zainteresowani::odnotuj_przebieg_prezentacji(
        &room,
        &OdnotujPrzebiegPrezentacjiParams {
            przebieg: "Nikt nie przyszedł.".to_string(),
            id: "prezentacja".to_string(),
        },
    )
    .expect("record the course of the viewing");
    let cancellation = |viewing_id: &str| OdwolajPrezentacjeParams {
        odwolano: "2026-11-07T12:00:00Z".to_string(),
        odwolal: "klientka".to_string(),
        powod_odwolania: "choroba".to_string(),
        id: viewing_id.to_string(),
    };
    expect::refused(
        oferta_zainteresowani::odwolaj_prezentacje(&room, &cancellation("prezentacja")),
        rule,
    );
    oferta_zainteresowani::umow_prezentacje(
        &room,
        &UmowPrezentacjeParams {
            id: "druga".to_string(),
            ..viewing("2026-11-08T17:00:00Z")
        },
    )
    .expect("arrange the second viewing");
    oferta_zainteresowani::odwolaj_prezentacje(&room, &cancellation("druga"))
        .expect("cancel the second viewing");
    expect::refused(
        oferta_zainteresowani::odnotuj_przebieg_prezentacji(
            &room,
            &OdnotujPrzebiegPrezentacjiParams {
                przebieg: "nikt nie przyszedł".to_string(),
                id: "druga".to_string(),
            },
        ),
        rule,
    );
    expect::refused(
        room.execute(
            "INSERT INTO oferta_zainteresowani_prezentacje (id, termin, posrednik, otwiera,
              przebieg)
            VALUES ('z-przebiegiem', '2026-11-07T17:00:00Z', 'p', 'klient', 'x')",
            &[],
        ),
        "prezentacja powstaje umówiona",
    );
}

// oferta-ogloszenia -------------------------------------------------------------------------------

/// Ogłoszenie powstaje jako projekt i wychodzi tylko z wersją treści sprzed publikacji
#[test]
fn ad_starts_as_draft() {
    let room = new_room();
    ready_to_publish(&room);

    expect::refused(
        room.execute(
            "INSERT INTO oferta_ogloszenia_wpisy (id, dotyczy, kanal, opublikowano)
            VALUES ('od-razu', 'sprzedaz', 'strona_biura', '2026-11-05T10:00:00Z')",
            &[],
        ),
        "ogłoszenie powstaje jako projekt",
    );
    expect::refused(
        oferta_ogloszenia::przygotuj_ogloszenie(
            &room,
            &PrzygotujOgloszenieParams {
                portal: None,
                ..ad("bez-portalu")
            },
        ),
        "portal_przy_kanale_portal",
    );
    expect::refused(
        oferta_ogloszenia::odnotuj_publikacje_ogloszenia(
            &room,
            &publication("ogloszenie", "2026-11-05T08:00:00Z"),
        ),
        "publikacja wymaga wersji treści sprzed publikacji",
    );
    publish(&room, "ogloszenie", PUBLISHED);
}

/// Ogłoszenie: zawarta umowa z czynnością publikacja_ogloszen (art. 180 ust. 3 ugn)
#[test]
fn ad_within_contract_scope() {
    let room = new_room();
    offer_case(
        &room,
        &BrokerageTerms {
            czynnosci: &["prezentacje"],
            ..brokerage_terms()
        },
    );
    add_visibility(&room, &visibility_version("widocznosc"));
    add_offer_price(&room, &offer_price("cena"));
    draft_ad(&room, &ad("ogloszenie"), &ad_version("ogloszenie", 1));
    approved_other(
        &room,
        &other_decision(
            "pierwsza-publikacja",
            "oferta_ogloszenia_wpisy",
            "ogloszenie",
        ),
    );
    let rule =
        "ogłoszenie wymaga zawartej umowy pośrednictwa, której zakres obejmuje publikacja_ogloszen";

    expect::refused(
        oferta_ogloszenia::odnotuj_publikacje_ogloszenia(
            &room,
            &publication("ogloszenie", PUBLISHED),
        ),
        rule,
    );

    end_contract(&room, "umowa", "aneks rozszerza zakres w nowej umowie");
    brokerage_contract(
        &room,
        &BrokerageTerms {
            contract_id: "umowa-2",
            ..brokerage_terms()
        },
    );
    publish(&room, "ogloszenie", PUBLISHED);
}

/// The parameters of the version `numer` of „ogloszenie” written after the first publication:
/// 659 000 zł from 2026-11-20 at 09:00, with its content `ogloszenia/<numer>.json`.
fn later_ad_version(numer: i64) -> DopiszWersjeOgloszeniaParams {
    DopiszWersjeOgloszeniaParams {
        ogloszenie_id: "ogloszenie".to_string(),
        numer,
        od: "2026-11-20T09:00:00Z".to_string(),
        tytul: "2 pokoje, 48 m², Krzyki".to_string(),
        cena: 65_900_000,
        lokalizacja: "Wrocław, Krzyki".to_string(),
        wskazniki_energetyczne: None,
        tresc_sciezka: format!("ogloszenia/{numer}.json"),
        tresc_skrot: format!("sha256-{numer}"),
    }
}

/// Ogłoszenie wychodzi nie wcześniej niż praca od według umowy (art. 15 ust. 3 upk)
#[test]
fn ad_after_work_starts() {
    let room = new_room();
    offer_case(
        &room,
        &BrokerageTerms {
            sposob_zawarcia: SposobZawarcia::NaOdleglosc,
            odstapienie_do: Some(WITHDRAWAL_ENDS),
            ..brokerage_terms()
        },
    );
    add_visibility(&room, &visibility_version("widocznosc"));
    add_offer_price(&room, &offer_price("cena"));
    draft_ad(&room, &ad("ogloszenie"), &ad_version("ogloszenie", 1));
    approved_other(
        &room,
        &other_decision(
            "pierwsza-publikacja",
            "oferta_ogloszenia_wpisy",
            "ogloszenie",
        ),
    );
    let rule = "ogłoszenie wychodzi nie wcześniej niż praca od";

    expect::refused(
        oferta_ogloszenia::odnotuj_publikacje_ogloszenia(
            &room,
            &publication("ogloszenie", "2026-11-18T10:00:00Z"),
        ),
        rule,
    );
    publish(&room, "ogloszenie", "2026-11-19T08:00:00Z");
    expect::refused(
        oferta_ogloszenia::dopisz_wersje_ogloszenia(
            &room,
            &DopiszWersjeOgloszeniaParams {
                od: "2026-11-18T23:00:00Z".to_string(),
                ..later_ad_version(2)
            },
        ),
        rule,
    );
}

/// Ogłoszenie: tylko oferta publiczna, której rodzaj obejmuje ogłoszenie
#[test]
fn ad_only_public_offer() {
    let room = new_room();
    offer_case(&room, &brokerage_terms());
    add_visibility(
        &room,
        &WpiszWersjeWidocznosciParams {
            widocznosc: Widocznosc::Niepubliczna,
            granice_zgody: Some("tylko klientom biura z potwierdzonym finansowaniem".to_string()),
            ..visibility_version("widocznosc")
        },
    );
    add_offer_price(&room, &offer_price("cena"));
    add_offer_price(
        &room,
        &WpiszCeneOfertowaParams {
            kwota: 320_000,
            dotyczy: DotyczyCeny::Najem,
            ..offer_price("czynsz")
        },
    );
    draft_ad(&room, &ad("ogloszenie"), &ad_version("ogloszenie", 1));
    draft_ad(
        &room,
        &PrzygotujOgloszenieParams {
            dotyczy: DotyczyOgloszenia::Najem,
            ..ad("najem")
        },
        &DopiszWersjeOgloszeniaParams {
            cena: 320_000,
            ..ad_version("najem", 1)
        },
    );
    approved_other(
        &room,
        &other_decision(
            "pierwsza-publikacja",
            "oferta_ogloszenia_wpisy",
            "ogloszenie",
        ),
    );
    let rule = "ogłasza się tylko ofertę publiczną";

    expect::refused(
        oferta_ogloszenia::odnotuj_publikacje_ogloszenia(
            &room,
            &publication("ogloszenie", PUBLISHED),
        ),
        rule,
    );

    add_received_document(
        &room,
        &received_document("aneks", RodzajDokumentu::UmowaZBiurem),
    );
    sprawa_umowy::wpisz_zdarzenie_umowy(
        &room,
        &WpiszZdarzenieUmowyParams {
            id: "aneks".to_string(),
            umowa_id: "umowa".to_string(),
            rodzaj: RodzajZdarzeniaUmowy::Aneks,
            chwila: "2026-11-05T09:30:00Z".to_string(),
            opis: None,
            dokument_id: Some("aneks".to_string()),
            wysylka_id: None,
            wplyw_id: None,
            zgoda_id: None,
        },
    )
    .expect("add the annex");
    add_visibility(
        &room,
        &WpiszWersjeWidocznosciParams {
            aneks_zdarzenie_id: Some("aneks".to_string()),
            od: "2026-11-05T09:30:00Z".to_string(),
            ..visibility_version("publiczna")
        },
    );
    publish(&room, "ogloszenie", PUBLISHED);
    expect::refused(
        oferta_ogloszenia::odnotuj_publikacje_ogloszenia(&room, &publication("najem", PUBLISHED)),
        rule,
    );
}

/// Ogłoszenie: cena w wersji to obowiązująca cena ofertowa, przy publikacji i nowej wersji
#[test]
fn ad_price_is_offer_price() {
    let room = new_room();
    offer_case(&room, &brokerage_terms());
    add_visibility(&room, &visibility_version("widocznosc"));
    add_offer_price(&room, &offer_price("cena"));
    draft_ad(
        &room,
        &ad("ogloszenie"),
        &DopiszWersjeOgloszeniaParams {
            cena: 60_000_000,
            ..ad_version("ogloszenie", 1)
        },
    );
    approved_other(
        &room,
        &other_decision(
            "pierwsza-publikacja",
            "oferta_ogloszenia_wpisy",
            "ogloszenie",
        ),
    );
    let rule = "cena w ogłoszeniu to obowiązująca cena ofertowa";

    expect::refused(
        oferta_ogloszenia::odnotuj_publikacje_ogloszenia(
            &room,
            &publication("ogloszenie", PUBLISHED),
        ),
        rule,
    );
    add_ad_version(
        &room,
        &DopiszWersjeOgloszeniaParams {
            od: "2026-11-05T09:30:00Z".to_string(),
            ..ad_version("ogloszenie", 2)
        },
    );
    publish(&room, "ogloszenie", PUBLISHED);

    add_offer_price(
        &room,
        &WpiszCeneOfertowaParams {
            kwota: 64_500_000,
            obowiazuje_od: "2026-11-08T09:00:00Z".to_string(),
            poprzednia_id: Some("cena".to_string()),
            powod: Some("brak zapytań".to_string()),
            ..offer_price("nowa")
        },
    );
    expect::refused(
        oferta_ogloszenia::dopisz_wersje_ogloszenia(&room, &later_ad_version(3)),
        rule,
    );
    add_ad_version(
        &room,
        &DopiszWersjeOgloszeniaParams {
            cena: 64_500_000,
            ..later_ad_version(3)
        },
    );
    expect::refused(
        oferta_ogloszenia::dopisz_wersje_ogloszenia(
            &room,
            &DopiszWersjeOgloszeniaParams {
                cena: 64_500_000,
                ..later_ad_version(5)
            },
        ),
        "wersje ogłoszenia mają kolejne numery",
    );
    expect::refused(
        oferta_ogloszenia::dopisz_wersje_ogloszenia(
            &room,
            &DopiszWersjeOgloszeniaParams {
                cena: 64_500_000,
                od: "2026-11-19T09:00:00Z".to_string(),
                ..later_ad_version(4)
            },
        ),
        "wersje ogłoszenia mają kolejne numery",
    );
    expect::refused(
        room.execute("UPDATE oferta_ogloszenia_wersje SET cena = 1", &[]),
        "wersji ogłoszenia się nie zmienia",
    );
}

/// Ogłoszenie: przy świadectwie ma wskaźniki, a świadectwo jest ważne w dniu publikacji
#[test]
fn ad_energy_certificate() {
    let room = new_room();
    ready_to_publish(&room);
    add_received_document(
        &room,
        &energy_certificate("swiadectwo", "2016-11-01", "2026-11-30"),
    );
    oferta_nieruchomosc::wpisz_opis_nieruchomosci(&room, &certificate_description("swiadectwo"))
        .expect("add the description with the certificate");
    let rule = "przy świadectwie energetycznym ogłoszenie ma jego wskaźniki";

    expect::refused(
        oferta_ogloszenia::odnotuj_publikacje_ogloszenia(
            &room,
            &publication("ogloszenie", PUBLISHED),
        ),
        rule,
    );
    add_ad_version(
        &room,
        &DopiszWersjeOgloszeniaParams {
            wskazniki_energetyczne: Some("EP 120 kWh/(m²·rok)".to_string()),
            ..ad_version("ogloszenie", 2)
        },
    );
    publish(&room, "ogloszenie", PUBLISHED);

    expect::refused(
        oferta_ogloszenia::dopisz_wersje_ogloszenia(
            &room,
            &DopiszWersjeOgloszeniaParams {
                od: "2026-12-01T09:00:00Z".to_string(),
                wskazniki_energetyczne: Some("EP 120".to_string()),
                ..later_ad_version(3)
            },
        ),
        rule,
    );
    add_ad_version(
        &room,
        &DopiszWersjeOgloszeniaParams {
            od: "2026-11-30T09:00:00Z".to_string(),
            wskazniki_energetyczne: Some("EP 120".to_string()),
            ..later_ad_version(3)
        },
    );
}

/// The parameters of a land register lookup `lookup_id` at `przegladano` that found the client's
/// testator in the register instead of her.
fn discrepant_lookup(lookup_id: &str, przegladano: &str) -> WpiszPrzegladanieKsiegiParams {
    WpiszPrzegladanieKsiegiParams {
        id: lookup_id.to_string(),
        nieruchomosc_id: "lokal".to_string(),
        numer_kw: "WR1K/00012345/6".to_string(),
        przegladano: przegladano.to_string(),
        wlasciciele_dzial_ii: "Zofia Kowalska".to_string(),
        wpisy_dzial_iii: None,
        wpisy_dzial_iv: None,
        rozbieznosc: true,
        rozbieznosc_opis: Some("w księdze wpisana spadkodawczyni klientki".to_string()),
        wydruk_sciezka: None,
        wydruk_skrot: None,
    }
}

/// Ogłoszenie: rozbieżność z księgą wymaga decyzji, która wskazuje ostatnie przeglądanie
#[test]
fn ad_land_register_discrepancy() {
    let room = new_room();
    ready_to_publish(&room);
    oferta_nieruchomosc::wpisz_przegladanie_ksiegi(
        &room,
        &discrepant_lookup("stare", "2026-11-01T09:00:00Z"),
    )
    .expect("add the old lookup");
    oferta_nieruchomosc::wpisz_przegladanie_ksiegi(
        &room,
        &discrepant_lookup("przegladanie", "2026-11-03T09:00:00Z"),
    )
    .expect("add the lookup");
    approved_other(
        &room,
        &other_decision(
            "zgoda-na-stare",
            "oferta_nieruchomosc_przegladania",
            "stare",
        ),
    );
    let rule = "przy rozbieżności z księgą ogłoszenie wymaga zatwierdzonej decyzji inna";

    expect::refused(
        oferta_ogloszenia::odnotuj_publikacje_ogloszenia(
            &room,
            &publication("ogloszenie", PUBLISHED),
        ),
        rule,
    );
    approved_other(
        &room,
        &other_decision(
            "zgoda-mimo-rozbieznosci",
            "oferta_nieruchomosc_przegladania",
            "przegladanie",
        ),
    );
    publish(&room, "ogloszenie", PUBLISHED);

    oferta_nieruchomosc::wpisz_przegladanie_ksiegi(
        &room,
        &WpiszPrzegladanieKsiegiParams {
            wlasciciele_dzial_ii: "Zofia Kowalska, Jan Nowak".to_string(),
            ..discrepant_lookup("nowe", "2026-11-10T09:00:00Z")
        },
    )
    .expect("add the new lookup");
    expect::refused(
        oferta_ogloszenia::dopisz_wersje_ogloszenia(&room, &later_ad_version(2)),
        rule,
    );
}

/// Ogłoszenie: pierwszą publikację w teczce zatwierdza pośrednik decyzją, która ją wskazuje
#[test]
fn ad_first_publication_approved() {
    let room = new_room();
    offer_case(&room, &brokerage_terms());
    add_visibility(&room, &visibility_version("widocznosc"));
    add_offer_price(&room, &offer_price("cena"));
    draft_ad(&room, &ad("ogloszenie"), &ad_version("ogloszenie", 1));
    draft_ad(&room, &ad("drugie"), &ad_version("drugie", 1));
    approved_other(
        &room,
        &other_decision("zgoda-na-drugie", "oferta_ogloszenia_wpisy", "drugie"),
    );
    let rule = "pierwsza publikacja w teczce wymaga zatwierdzonej decyzji inna";

    expect::refused(
        oferta_ogloszenia::odnotuj_publikacje_ogloszenia(
            &room,
            &publication("ogloszenie", PUBLISHED),
        ),
        rule,
    );
    publish(&room, "drugie", PUBLISHED);
    publish(&room, "ogloszenie", PUBLISHED);
}

/// The parameters of the number `numer_ogloszenia` the portal gave „ogloszenie”.
fn ad_number(numer_ogloszenia: &str) -> OdnotujNumerOgloszeniaParams {
    OdnotujNumerOgloszeniaParams {
        numer_ogloszenia: numer_ogloszenia.to_string(),
        id: "ogloszenie".to_string(),
    }
}

/// The parameters of the statistics of „ogloszenie” read on 2026-11-06 at 08:00.
fn ad_statistics(wyswietlenia: i64, zapytania: i64) -> WpiszStatystykiOgloszeniaParams {
    WpiszStatystykiOgloszeniaParams {
        ogloszenie_id: "ogloszenie".to_string(),
        odczytano: "2026-11-06T08:00:00Z".to_string(),
        wyswietlenia,
        zapytania,
    }
}

/// Ogłoszenie: zdjęte ma powód i nie wraca; numer i statystyki po publikacji
#[test]
fn ad_taken_down() {
    let room = new_room();
    ready_to_publish(&room);

    expect::refused(
        oferta_ogloszenia::odnotuj_numer_ogloszenia(&room, &ad_number("OT-1")),
        "numer_po_publikacji",
    );
    expect::refused(
        oferta_ogloszenia::wpisz_statystyki_ogloszenia(&room, &ad_statistics(0, 0)),
        "statystyki odczytuje się dla ogłoszenia po jego publikacji",
    );
    publish(&room, "ogloszenie", PUBLISHED);
    oferta_ogloszenia::odnotuj_numer_ogloszenia(&room, &ad_number("OT-1"))
        .expect("record the number of the ad");
    oferta_ogloszenia::wpisz_statystyki_ogloszenia(&room, &ad_statistics(120, 2))
        .expect("add the statistics of the ad");
    expect::refused(
        room.execute("UPDATE oferta_ogloszenia_statystyki SET zapytania = 9", &[]),
        "odczytu statystyk się nie zmienia",
    );

    let rule = "ogłoszenia się nie zmienia";
    expect::refused(
        oferta_ogloszenia::odnotuj_numer_ogloszenia(&room, &ad_number("OT-2")),
        rule,
    );
    expect::refused(
        room.execute(
            "UPDATE oferta_ogloszenia_wpisy SET zdjeto = '2026-11-11T13:00:00Z' WHERE id = 'ogloszenie'",
            &[],
        ),
        "zdjete_ma_powod",
    );
    oferta_ogloszenia::zdejmij_ogloszenie(
        &room,
        &ZdejmijOgloszenieParams {
            zdjeto: "2026-11-11T13:00:00Z".to_string(),
            powod_zdjecia: "koniec umowy pośrednictwa".to_string(),
            id: "ogloszenie".to_string(),
        },
    )
    .expect("take the ad down");
    expect::refused(
        oferta_ogloszenia::dopisz_wersje_ogloszenia(&room, &later_ad_version(2)),
        "zdjęte ogłoszenie nie dostaje nowych wersji",
    );
    expect::refused(
        room.execute(
            "UPDATE oferta_ogloszenia_wpisy SET zdjeto = NULL, powod_zdjecia = NULL",
            &[],
        ),
        rule,
    );
}

// oferta-oferty -----------------------------------------------------------------------------------

/// Writes an offer case (see [`offer_case`]) with the interested person „zainteresowany”.
fn offers_ready(room: &impl MetaRoom, terms: &BrokerageTerms) {
    offer_case(room, terms);
    add_interested(room, &interested("zainteresowany"));
}

/// Oferta zainteresowanego: z wpływu albo notatki, z terminem związania albo sposobem złożenia
#[test]
fn offer_source_and_binding() {
    let room = new_room();
    offers_ready(&room, &brokerage_terms());
    add_inflow(
        &room,
        &WpiszWplywParams {
            nadawca: "Piotr".to_string(),
            ..inflow("wplyw")
        },
    );
    add_received_document(&room, &received_document("notatka", RodzajDokumentu::Pismo));
    let offer = WpiszOferteZainteresowanegoParams {
        id: "oferta".to_string(),
        dotyczy: DotyczyOferty::Sprzedaz,
        osoba_id: "zainteresowany".to_string(),
        odpowiada_na_id: None,
        wplyw_id: None,
        dokument_id: None,
        wpisano: "2026-11-10T09:00:00Z".to_string(),
        kwota: 61_000_000,
        zadatek_albo_kaucja: None,
        termin_umowy: None,
        termin_wydania: None,
        finansowanie: None,
        warunki_zastrzezone: None,
        wiaze_do: None,
        sposob_zlozenia: None,
    };

    expect::refused(
        oferta_oferty::wpisz_oferte_zainteresowanego(
            &room,
            &WpiszOferteZainteresowanegoParams {
                wplyw_id: Some("wplyw".to_string()),
                ..offer.clone()
            },
        ),
        "oferta_ma_termin_albo_sposob_zlozenia",
    );
    expect::refused(
        oferta_oferty::wpisz_oferte_zainteresowanego(
            &room,
            &WpiszOferteZainteresowanegoParams {
                wplyw_id: Some("wplyw".to_string()),
                dokument_id: Some("notatka".to_string()),
                sposob_zlozenia: Some(SposobZlozenia::WRozmowie),
                ..offer.clone()
            },
        ),
        "oferta_zainteresowanego_z_wplywu_albo_notatki",
    );
    oferta_oferty::wpisz_oferte_zainteresowanego(
        &room,
        &WpiszOferteZainteresowanegoParams {
            dokument_id: Some("notatka".to_string()),
            sposob_zlozenia: Some(SposobZlozenia::WRozmowie),
            ..offer
        },
    )
    .expect("add the offer from the note");
    expect::refused(
        room.execute(
            "INSERT INTO oferta_oferty_wpisy (id, dotyczy, kierunek, osoba_id, wplyw_id, wpisano,
              kwota, sposob_zlozenia, przedstawiono)
            VALUES ('od-razu', 'sprzedaz', 'od_zainteresowanego', 'zainteresowany', 'wplyw',
              '2026-11-10', 1, 'na_pismie', '2026-11-10')",
            &[],
        ),
        "oferta powstaje bez przedstawienia, decyzji i odpowiedzi",
    );
    expect::refused(
        room.execute(
            "UPDATE oferta_oferty_wpisy SET kwota = 62000000 WHERE id = 'oferta'",
            &[],
        ),
        "oferty się nie zmienia",
    );
}

/// The parameters of the client's decision `decyzja_klienta` about „oferta” with the consent
/// `zgoda_id`.
fn decision_about_offer(
    decyzja_klienta: DecyzjaKlienta,
    zgoda_id: &str,
) -> OdnotujDecyzjeKlientaOOfercieParams {
    OdnotujDecyzjeKlientaOOfercieParams {
        decyzja_klienta,
        zgoda_id: zgoda_id.to_string(),
        id: "oferta".to_string(),
    }
}

/// The parameters of the presentation of „oferta” to the client at `przedstawiono`.
fn offer_presentation(przedstawiono: &str) -> OdnotujPrzedstawienieOfertyParams {
    OdnotujPrzedstawienieOfertyParams {
        przedstawiono: przedstawiono.to_string(),
        obie_strony_decyzja_id: None,
        id: "oferta".to_string(),
    }
}

/// Decyzja klienta o ofercie: po przedstawieniu, z jego zgodą odpowiedz_na_oferte, raz
#[test]
fn offer_client_decision() {
    let room = new_room();
    offers_ready(&room, &brokerage_terms());
    add_person(
        &room,
        &WpiszUczestnikaParams {
            imiona: Some("Jan".to_string()),
            nazwisko: Some("Kowalski".to_string()),
            ..person("maz")
        },
    );
    add_interested_offer(&room, &interested_offer("oferta"));
    let rule = "decyzja klienta o ofercie wskazuje jego zgodę odpowiedz_na_oferte";

    add_consent(
        &room,
        &client_consent(
            "decyzja-oferta",
            "odpowiedz_na_oferte",
            "oferta_oferty_wpisy",
            "oferta",
        ),
    );
    expect::refused(
        oferta_oferty::odnotuj_decyzje_klienta_o_ofercie(
            &room,
            &decision_about_offer(DecyzjaKlienta::Przyjeta, "decyzja-oferta"),
        ),
        "decyzja_po_przedstawieniu",
    );
    oferta_oferty::odnotuj_przedstawienie_oferty(
        &room,
        &offer_presentation("2026-11-10T10:00:00Z"),
    )
    .expect("present the offer to the client");
    add_consent(
        &room,
        &WpiszZgodeParams {
            uczestnik_id: "maz".to_string(),
            ..client_consent(
                "zgoda-meza",
                "odpowiedz_na_oferte",
                "oferta_oferty_wpisy",
                "oferta",
            )
        },
    );
    expect::refused(
        oferta_oferty::odnotuj_decyzje_klienta_o_ofercie(
            &room,
            &decision_about_offer(DecyzjaKlienta::Przyjeta, "zgoda-meza"),
        ),
        rule,
    );
    add_consent(
        &room,
        &client_consent(
            "zgoda-ceny",
            "zatwierdzenie_ceny",
            "oferta_oferty_wpisy",
            "oferta",
        ),
    );
    expect::refused(
        oferta_oferty::odnotuj_decyzje_klienta_o_ofercie(
            &room,
            &decision_about_offer(DecyzjaKlienta::Przyjeta, "zgoda-ceny"),
        ),
        rule,
    );
    oferta_oferty::odnotuj_decyzje_klienta_o_ofercie(
        &room,
        &decision_about_offer(DecyzjaKlienta::Odrzucona, "decyzja-oferta"),
    )
    .expect("record the client's decision");

    let rule = "oferty się nie zmienia";
    expect::refused(
        oferta_oferty::odnotuj_decyzje_klienta_o_ofercie(
            &room,
            &decision_about_offer(DecyzjaKlienta::Przyjeta, "decyzja-oferta"),
        ),
        rule,
    );
    expect::refused(
        oferta_oferty::odnotuj_przedstawienie_oferty(
            &room,
            &offer_presentation("2026-11-10T11:00:00Z"),
        ),
        rule,
    );
}

/// The parameters of the client's counteroffer `counter_id` of 640 000 zł to `odpowiada_na_id`,
/// binding until 2026-11-13, with her consent „decyzja-oferta-1”.
fn client_counteroffer(counter_id: &str, odpowiada_na_id: &str) -> WpiszKontroferteKlientaParams {
    WpiszKontroferteKlientaParams {
        id: counter_id.to_string(),
        dotyczy: DotyczyOferty::Sprzedaz,
        osoba_id: "zainteresowany".to_string(),
        odpowiada_na_id: odpowiada_na_id.to_string(),
        wpisano: "2026-11-10T11:00:00Z".to_string(),
        kwota: 64_000_000,
        zadatek_albo_kaucja: None,
        termin_umowy: None,
        termin_wydania: None,
        warunki_zastrzezone: None,
        wiaze_do: Some("2026-11-13".to_string()),
        sposob_zlozenia: None,
        zgoda_id: "decyzja-oferta-1".to_string(),
    }
}

/// The parameters of the counteroffer „oferta-2” of 625 000 zł from „zainteresowany” in writing,
/// in the inflow „odpowiedz-piotra”, to `odpowiada_na_id`.
fn counter_from_interested(odpowiada_na_id: &str) -> WpiszOferteZainteresowanegoParams {
    WpiszOferteZainteresowanegoParams {
        id: "oferta-2".to_string(),
        dotyczy: DotyczyOferty::Sprzedaz,
        osoba_id: "zainteresowany".to_string(),
        odpowiada_na_id: Some(odpowiada_na_id.to_string()),
        wplyw_id: Some("odpowiedz-piotra".to_string()),
        dokument_id: None,
        wpisano: "2026-11-11T09:00:00Z".to_string(),
        kwota: 62_500_000,
        zadatek_albo_kaucja: None,
        termin_umowy: None,
        termin_wydania: None,
        finansowanie: None,
        warunki_zastrzezone: None,
        wiaze_do: None,
        sposob_zlozenia: Some(SposobZlozenia::NaPismie),
    }
}

/// The parameters of Piotr's counteroffer in reply to the offer `offer_id`, in the inflow
/// „odpowiedz-piotra”.
fn reply_of_interested(offer_id: &str) -> OdnotujOdpowiedzZainteresowanegoParams {
    OdnotujOdpowiedzZainteresowanegoParams {
        odpowiedz_drugiej_strony: DecyzjaKlienta::Kontroferta,
        odpowiedz_wplyw_id: "odpowiedz-piotra".to_string(),
        id: offer_id.to_string(),
    }
}

/// Kontroferty: klient odpowiada na decyzję kontroferta, zainteresowany na swoją odpowiedź
#[test]
fn offer_counter_chain() {
    let room = new_room();
    offers_ready(&room, &brokerage_terms());
    add_interested_offer(&room, &interested_offer("oferta-1"));
    add_interested_offer(&room, &interested_offer("oferta-bez-decyzji"));
    client_decision(&room, "oferta-1", DecyzjaKlienta::Kontroferta);
    let rule = "kontroferta klienta odpowiada na ofertę zainteresowanego z decyzją kontroferta";

    expect::refused(
        oferta_oferty::wpisz_kontroferte_klienta(
            &room,
            &client_counteroffer("k", "oferta-bez-decyzji"),
        ),
        rule,
    );
    expect::refused(
        oferta_oferty::wpisz_kontroferte_klienta(
            &room,
            &WpiszKontroferteKlientaParams {
                dotyczy: DotyczyOferty::Najem,
                ..client_counteroffer("k", "oferta-1")
            },
        ),
        rule,
    );
    oferta_oferty::wpisz_kontroferte_klienta(
        &room,
        &client_counteroffer("kontroferta", "oferta-1"),
    )
    .expect("add the client's counteroffer");

    let rule = "kontroferta zainteresowanego odpowiada na kontrofertę klienta";
    expect::refused(
        oferta_oferty::wpisz_oferte_zainteresowanego(
            &room,
            &counter_from_interested("kontroferta"),
        ),
        rule,
    );
    add_inflow(
        &room,
        &WpiszWplywParams {
            nadawca: "Piotr".to_string(),
            ..inflow("odpowiedz-piotra")
        },
    );
    expect::refused(
        oferta_oferty::odnotuj_odpowiedz_zainteresowanego(&room, &reply_of_interested("oferta-1")),
        "odpowiedz_drugiej_strony_na_kontroferte",
    );
    oferta_oferty::odnotuj_odpowiedz_zainteresowanego(&room, &reply_of_interested("kontroferta"))
        .expect("record the reply of the interested person");
    expect::refused(
        oferta_oferty::wpisz_oferte_zainteresowanego(&room, &counter_from_interested("oferta-1")),
        rule,
    );
    oferta_oferty::wpisz_oferte_zainteresowanego(&room, &counter_from_interested("kontroferta"))
        .expect("add the counteroffer of the interested person");
}

/// Odpowiedź na ofertę: niewysłana wysyłka, która wychodzi tylko za zgodą człowieka
#[test]
fn offer_reply_needs_consent() {
    let room = new_room();
    offers_ready(&room, &brokerage_terms());
    add_interested_offer(&room, &interested_offer("oferta"));
    client_decision(&room, "oferta", DecyzjaKlienta::Odrzucona);
    let reply_with = |letter_id: &str| WskazOdpowiedzNaOferteParams {
        odpowiedz_wysylka_id: letter_id.to_string(),
        id: "oferta".to_string(),
    };

    prepare_letter(&room, &reply_letter("wyslana"));
    send(&room, &sending("wyslana"));
    expect::refused(
        oferta_oferty::wskaz_odpowiedz_na_oferte(&room, &reply_with("wyslana")),
        "odpowiedź na ofertę wskazuje wysyłkę, która jeszcze nie wyszła",
    );
    prepare_letter(&room, &reply_letter("odpowiedz"));
    oferta_oferty::wskaz_odpowiedz_na_oferte(&room, &reply_with("odpowiedz"))
        .expect("link the reply to the offer");
    expect::refused(
        sprawa_korespondencja::odnotuj_wyslanie(
            &room,
            &OdnotujWyslanieParams {
                wyslano: "2026-11-10T12:00:00Z".to_string(),
                ..sending("odpowiedz")
            },
        ),
        "odpowiedź na ofertę przed wysłaniem czeka na zgodę",
    );

    sprawa_korespondencja::przekaz_wysylke_do_zgody(
        &room,
        &PrzekazWysylkeDoZgodyParams {
            id: "odpowiedz".to_string(),
        },
    )
    .expect("hand the reply over for consent");
    approved_other(
        &room,
        &decision_request_about(
            "wyslanie",
            RodzajDecyzji::Wyslanie,
            "sprawa_korespondencja_wysylki",
            "odpowiedz",
        ),
    );
    send(
        &room,
        &OdnotujWyslanieParams {
            decyzja_id: Some("wyslanie".to_string()),
            ..sending("odpowiedz")
        },
    );
}

/// Przyjęcie oferty nie wychodzi, dopóki stoi znacznik wątpliwości
#[test]
fn offer_acceptance_blocked_by_doubt() {
    let room = new_room();
    offers_ready(&room, &brokerage_terms());
    add_interested_offer(&room, &interested_offer("oferta"));
    client_decision(&room, "oferta", DecyzjaKlienta::Przyjeta);
    let reply = reply_awaiting_consent(&room, "oferta");
    sprawa_metryka::postaw_watpliwosc(
        &room,
        &PostawWatpliwoscParams {
            id: "watpliwosc".to_string(),
            postawil: "tozsamosc-kontrolera".to_string(),
            postawiono: "2026-11-10T13:00:00Z".to_string(),
        },
    )
    .expect("raise the doubt");
    let rule = "dopóki stoi znacznik wątpliwości, przyjęcie oferty nie wychodzi";
    let sending_at = |wyslano: &str| OdnotujWyslanieParams {
        decyzja_id: Some(format!("wyslanie-{reply}")),
        wyslano: wyslano.to_string(),
        ..sending(&reply)
    };

    expect::refused(
        sprawa_korespondencja::odnotuj_wyslanie(&room, &sending_at("2026-11-10T14:00:00Z")),
        rule,
    );

    ask_for_decision(
        &room,
        &PoprosODecyzjeParams {
            prosi_stanowisko: ProsiStanowisko::Kontroler,
            zlozono: "2026-11-10T15:00:00Z".to_string(),
            ..decision_request_about(
                "zdjecie",
                RodzajDecyzji::ZdjecieWatpliwosci,
                "sprawa_metryka_watpliwosci",
                "watpliwosc",
            )
        },
    );
    approve(&room, "zdjecie", "2026-11-10T16:00:00Z");
    sprawa_metryka::zdejmij_watpliwosc(
        &room,
        &ZdejmijWatpliwoscParams {
            zdjecie_decyzja_id: "zdjecie".to_string(),
            id: "watpliwosc".to_string(),
        },
    )
    .expect("lift the doubt");
    send(&room, &sending_at("2026-11-10T17:00:00Z"));
}

/// Oferta klienta biura bez zgody umowy na obie strony idzie do klienta za zgodą człowieka (R1)
#[test]
fn offer_from_office_client() {
    let room = new_room();
    offers_ready(&room, &brokerage_terms());
    add_interested(
        &room,
        &WpiszZainteresowanegoParams {
            dziala_jako: DzialaJako::KlientBiura,
            ..interested("klient-biura")
        },
    );
    add_interested_offer(
        &room,
        &WpiszOferteZainteresowanegoParams {
            osoba_id: "klient-biura".to_string(),
            ..interested_offer("oferta")
        },
    );
    let rule = "ofertę klienta biura bez zgody umowy na obie strony";

    expect::refused(
        oferta_oferty::odnotuj_przedstawienie_oferty(
            &room,
            &offer_presentation("2026-11-10T10:00:00Z"),
        ),
        rule,
    );
    approved_other(
        &room,
        &other_decision("obie-strony", "oferta_oferty_wpisy", "oferta"),
    );
    oferta_oferty::odnotuj_przedstawienie_oferty(
        &room,
        &OdnotujPrzedstawienieOfertyParams {
            obie_strony_decyzja_id: Some("obie-strony".to_string()),
            ..offer_presentation("2026-11-10T10:00:00Z")
        },
    )
    .expect("present the offer with the human's decision");
}

/// Oferta klienta biura przy umowie na obie strony nie wymaga decyzji człowieka
#[test]
fn offer_from_office_client_both_sides() {
    let room = new_room();
    offers_ready(
        &room,
        &BrokerageTerms {
            obie_strony: true,
            ..brokerage_terms()
        },
    );
    add_interested(
        &room,
        &WpiszZainteresowanegoParams {
            dziala_jako: DzialaJako::KlientBiura,
            ..interested("klient-biura")
        },
    );
    add_interested_offer(
        &room,
        &WpiszOferteZainteresowanegoParams {
            osoba_id: "klient-biura".to_string(),
            ..interested_offer("oferta")
        },
    );

    oferta_oferty::odnotuj_przedstawienie_oferty(
        &room,
        &offer_presentation("2026-11-10T10:00:00Z"),
    )
    .expect("present the offer without a decision");
}

// R9 and replacement ------------------------------------------------------------------------------

/// The names of the tables of the offer type, from the schema.
fn offer_tables(room: &MemoryRoom) -> Vec<String> {
    let rows = room
        .execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'oferta\\_%' ESCAPE '\\'",
            &[],
        )
        .expect("read the tables of the offer type");

    rows.iter()
        .map(|row| row.get::<String>("name").expect("a table has a name"))
        .collect()
}

/// Writes the full offer case (see [`full_case`]) and the client's rental terms „warunki”.
fn full_case_with_rental_terms(room: &impl MetaRoom) {
    full_case(room);

    add_consent(
        room,
        &client_consent(
            "zgoda-warunki",
            "warunki_najmu",
            "oferta_najem_warunki",
            "warunki",
        ),
    );
    oferta_najem::wpisz_warunki_najmu(room, &rental_terms_of("warunki"))
        .expect("add the rental terms");
}

/// The names of the columns of the primary key of `table`, from its schema.
fn primary_key_of(room: &MemoryRoom, table: &str) -> Vec<String> {
    let columns = room
        .execute(&format!("PRAGMA table_info({table})"), &[])
        .expect("read the columns of the table");

    let mut key = Vec::new();
    for column in &columns {
        let position_in_key = column
            .get::<i64>("pk")
            .expect("a column says if it is in the key");

        if position_in_key > 0 {
            key.push(column.get::<String>("name").expect("a column has a name"));
        }
    }

    key
}

/// R9: wierszy teczki oferty się nie usuwa
#[test]
fn no_deletion() {
    let room = new_room();
    full_case_with_rental_terms(&room);

    let tables = offer_tables(&room);
    assert_eq!(tables.len(), 16, "{tables:?}");

    for table in &tables {
        let rows = integer_value(&room, &format!("SELECT count(*) FROM {table}"));
        assert!(rows > 0, "{table} has no row to delete");

        let sql = format!("DELETE FROM {table} WHERE rowid = (SELECT min(rowid) FROM {table})");
        expect::refused(room.execute(&sql, &[]), DELETION);
    }
}

/// Wiersza teczki oferty nie zastępuje INSERT OR REPLACE ani upsert
#[test]
fn no_replacement() {
    let room = new_room();
    full_case_with_rental_terms(&room);

    for table in &offer_tables(&room) {
        let key = primary_key_of(&room, table).join(", ");

        let replace = format!("INSERT OR REPLACE {}", first_row_copy(&room, table));
        expect::refused(room.execute(&replace, &[]), REPLACEMENT);

        let upsert = format!(
            "INSERT {} ON CONFLICT ({key}) DO UPDATE SET {key} = excluded.{key}",
            first_row_copy(&room, table)
        );
        expect::refused(room.execute(&upsert, &[]), REPLACEMENT);

        if !unique_keys_of(&room, table).is_empty() {
            let replace_by_unique_key = format!(
                "INSERT OR REPLACE {}",
                first_row_copy_without_id(&room, table)
            );
            expect::refused(room.execute(&replace_by_unique_key, &[]), REPLACEMENT);
        }
    }
}
