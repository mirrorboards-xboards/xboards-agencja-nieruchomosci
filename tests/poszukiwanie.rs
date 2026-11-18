//! Scenarios of the search case catalog (spec/cases/poszukiwanie) on top of the core.
//!
//! A search room is the core of every case plus the kinds of the search type. Each scenario starts
//! from an empty room and checks one rule the database keeps: a write the rule allows and a write
//! it refuses. The first scenario follows a whole search case as the profile describes it, from
//! intake through the brokerage contract, the search profile, the credit analysis, proposals and
//! viewings to the client's accepted offer.

use xboards_agencja_nieruchomosci::cases::poszukiwanie::{
    self, poszukiwanie_kredyt, poszukiwanie_oferty, poszukiwanie_profile, poszukiwanie_propozycje,
};
use xboards_agencja_nieruchomosci::cases::sprawa::{
    sprawa_decyzje, sprawa_dokumenty, sprawa_korespondencja, sprawa_metryka, sprawa_powiazania,
    sprawa_rachunki, sprawa_uczestnicy, sprawa_umowy, sprawa_wyplaty, sprawa_zgody,
};
use xboards_agencja_nieruchomosci::xmeta_room::{FromSqlValue, MemoryRoom, MetaRoom, SqlValue};

use poszukiwanie_kredyt::{
    DaneRodzaj, DopiszDanaDoAnalizyParams, HarmonogramKrok, OdnotujKrokHarmonogramuParams,
    OdnotujWymaganiaBankuParams, PrzeliczKrokHarmonogramuParams, WpiszAnalizeKredytowaParams,
    WpiszBankKlientaParams, WpiszDanaKredytowaParams, WpiszKrokHarmonogramuParams,
};
use poszukiwanie_oferty::{
    OdnotujDecyzjeKlientaOKontrofercieParams, OdnotujOdpowiedzDrugiejStronyParams,
    OdnotujPrzedstawienieKontrofertyParams, PotwierdzOferteKlientaParams,
    PrzygotujOferteKlientaParams, WpisyDotyczy as Dotyczy, WpisyKierunek as Kierunek,
    WpisyOdpowiedzDrugiejStrony as OdpowiedzDrugiejStrony, WpisySposobZlozenia as SposobZlozenia,
    WpiszKontroferteDrugiejStronyParams, WskazOdpowiedzNaKontroferteParams,
    WskazWysylkeOfertyKlientaParams,
};
use poszukiwanie_profile::{
    CechyWaga, DopiszCecheProfiluParams, DopiszFinansowanieProfiluParams, FinansowanieZrodlo,
    PotwierdzWersjeProfiluParams, WersjeCel, WpiszWersjeProfiluParams,
    ZalozProfilPoszukiwaniaParams,
};
use poszukiwanie_propozycje::{
    DopiszPytanieDoPrezentacjiParams, OdnotujCenyTransakcjiPropozycjiParams,
    OdnotujNotatkePosrednikaParams, OdnotujOdpowiedzNaPytanieParams, OdnotujOpinieKlientaParams,
    OdnotujPrzedstawieniePropozycjiParams, OdnotujPrzegladanieKsiegiPropozycjiParams,
    OdnotujReakcjeNaPropozycjeParams, OdwolajPrezentacjePropozycjiParams,
    PrezentacjeKtoPokazuje as KtoPokazuje, UmowPrezentacjePropozycjiParams,
    WpisyKwOgloszajacyWpisany as OgloszajacyWpisany, WpisyProwadzi as Prowadzi,
    WpisyReakcja as Reakcja, WpisyZrodlo as ZrodloPropozycji, WpiszPropozycjeParams,
};
use sprawa_decyzje::{
    PoprosODecyzjeParams, WpisyProsiStanowisko as ProsiStanowisko, WpisyRodzaj as RodzajDecyzji,
};
use sprawa_dokumenty::{
    WpisyRodzaj as RodzajDokumentu, WpisyWymaga as WymagaDokumentu, WpiszOtrzymanyDokumentParams,
    WpiszWymaganyDokumentParams,
};
use sprawa_korespondencja::{
    OdnotujWyslanieParams, PrzekazWysylkeDoZgodyParams, PrzygotujWysylkeParams, WpiszWplywParams,
    WysylkiAdresatRodzaj,
};
use sprawa_metryka::{
    PostawWatpliwoscParams, ZalozMetrykeParams, ZdejmijWatpliwoscParams, ZmienEtapSprawyParams,
};
use sprawa_powiazania::{WpisyRodzaj as RodzajPowiazania, WpiszPowiazanieParams};
use sprawa_rachunki::WpiszRachunekParams;
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
    WITHDRAWAL_ENDS, account, add_consent, add_inflow, add_person, add_received_document, approve,
    approved_decision, ask_for_decision, check_identity, conclude_contract, consent, contract,
    decision_request, decision_request_about, inflow, letter, open_case, person, prepare_contract,
    prepare_letter, received_document, send, sending,
};
use crate::sprawa_porzadek::{first_row_copy, first_row_copy_without_id, unique_keys_of};

const REPLACEMENT: &str =
    "wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE";
const DELETION: &str = "wierszy teczki się nie usuwa (R9)";
const STANDARD_DISCLAIMER: &str =
    "Analiza jest orientacyjna; nie jest ofertą ani rekomendacją kredytu.";

fn new_room() -> MemoryRoom {
    poszukiwanie::open_in_memory().expect("open the search room")
}

/// A text value to bind in raw SQL.
fn text(value: &str) -> SqlValue {
    SqlValue::Text(value.to_string())
}

// Building blocks of a search case ----------------------------------------------------------------

/// Writes the request of `request` asked by the searcher's keeper, and the broker's consent to it.
fn approved_other(room: &impl MetaRoom, request: &PoprosODecyzjeParams) {
    let asked_by_keeper = PoprosODecyzjeParams {
        prosi_stanowisko: ProsiStanowisko::OpiekunPoszukujacego,
        ..request.clone()
    };

    ask_for_decision(room, &asked_by_keeper);
    approve(room, &request.id, "2026-11-03T12:00:00Z");
}

fn give_role(room: &impl MetaRoom, uczestnik_id: &str, rola: &str) {
    let params = NadajRoleUczestnikowiParams {
        uczestnik_id: uczestnik_id.to_string(),
        rola: rola.to_string(),
        dotyczy_uczestnik_id: None,
    };

    sprawa_uczestnicy::nadaj_role_uczestnikowi(room, &params).expect("give the role");
}

/// Writes the client `klient`: Marek Wiśniewski, a consumer, with the role of the client.
fn client(room: &impl MetaRoom) {
    add_person(
        room,
        &WpiszUczestnikaParams {
            imiona: Some("Marek".to_string()),
            nazwisko: Some("Wiśniewski".to_string()),
            konsument: true,
            ..person("klient")
        },
    );
    give_role(room, "klient", "klient");
}

/// Writes Ewa Lis, the client's partner, as the participant `partnerka`.
fn add_partner(room: &impl MetaRoom) {
    add_person(
        room,
        &WpiszUczestnikaParams {
            imiona: Some("Ewa".to_string()),
            nazwisko: Some("Lis".to_string()),
            ..person("partnerka")
        },
    );
}

/// A brokerage contract of the buyer, checked, signed and concluded on 2026-11-04.
fn brokerage_contract(
    room: &impl MetaRoom,
    sposob_zawarcia: SposobZawarcia,
    odstapienie_do: Option<&str>,
) {
    check_identity(room, "tozsamosc-klienta", "klient");
    prepare_contract(
        room,
        &PrzygotujUmoweParams {
            rodzaj: RodzajUmowy::Posrednictwo,
            wylacznosc: false,
            wynagrodzenie_opis: Some("2% ceny".to_string()),
            wynagrodzenie_zdarzenie: Some("zawarcie umowy przyrzeczonej".to_string()),
            ..contract("umowa")
        },
        &["klient"],
    );

    for czynnosc in ["wyszukiwanie_ofert", "prezentacje", "wysylanie_ofert"] {
        let act = DopiszCzynnoscUmowyParams {
            umowa_id: "umowa".to_string(),
            czynnosc: czynnosc.to_string(),
        };

        sprawa_umowy::dopisz_czynnosc_umowy(room, &act).expect("add an act of the contract");
    }

    let for_signing = PrzekazUmoweDoPodpisuParams {
        id: "umowa".to_string(),
    };
    sprawa_umowy::przekaz_umowe_do_podpisu(room, &for_signing)
        .expect("hand the contract over for signing");

    conclude_contract(room, "umowa", sposob_zawarcia, odstapienie_do);
}

/// A search case with the client and a concluded brokerage contract (the Python defaults are
/// „w_lokalu_biura” and no end of the withdrawal period).
fn search_case(
    room: &impl MetaRoom,
    sposob_zawarcia: SposobZawarcia,
    odstapienie_do: Option<&str>,
) {
    open_case(room, "poszukiwanie");
    client(room);
    brokerage_contract(room, sposob_zawarcia, odstapienie_do);
}

fn end_contract(room: &impl MetaRoom, koniec_powod: &str) {
    let params = ZakonczUmoweParams {
        koniec_powod: koniec_powod.to_string(),
        id: "umowa".to_string(),
    };

    sprawa_umowy::zakoncz_umowe(room, &params).expect("end the contract");
}

/// The parameters of the client's consent of `rodzaj` about the row `dotyczy_id` of
/// `dotyczy_tabela`; [`add_consent`] writes it with its inflow.
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
        ..consent(consent_id, "klient", rodzaj)
    }
}

fn open_profile(room: &impl MetaRoom, nazwa: &str) {
    let params = ZalozProfilPoszukiwaniaParams {
        id: "profil".to_string(),
        nazwa: nazwa.to_string(),
    };

    poszukiwanie_profile::zaloz_profil_poszukiwania(room, &params)
        .expect("open the search profile");
}

/// The default parameters of a version of the profile: a flat to buy in Wrocław, Krzyki or
/// Fabryczna, with at least three rooms, for 750 000 zł.
fn profile_version(version_id: &str, od: &str) -> WpiszWersjeProfiluParams {
    WpiszWersjeProfiluParams {
        id: version_id.to_string(),
        profil_id: "profil".to_string(),
        cel: WersjeCel::Kupno,
        na_wynajem: false,
        rodzaj_nieruchomosci: "mieszkanie".to_string(),
        przeznaczenie: None,
        polozenie: "Wrocław, Krzyki albo Fabryczna".to_string(),
        dojazd: None,
        pow_min: None,
        pow_max: None,
        pokoje_min: Some(3),
        pokoje_max: None,
        pietro: None,
        termin: None,
        termin_powod: None,
        budzet: 75_000_000,
        budzet_z_kosztami: false,
        najem_od: None,
        okres: None,
        osoby: None,
        dzieci: None,
        zwierzeta: None,
        okazjonalny_dopuszcza: None,
        inny_lokal_ma: None,
        kaucja_max: None,
        umeblowany: None,
        od: od.to_string(),
    }
}

/// The parameters of a version of the profile the rule scenarios vary: a flat to buy anywhere in
/// Wrocław, with no number of rooms, from 2026-11-05 at 09:00.
fn minimal_profile_version(version_id: &str) -> WpiszWersjeProfiluParams {
    WpiszWersjeProfiluParams {
        polozenie: "Wrocław".to_string(),
        pokoje_min: None,
        ..profile_version(version_id, "2026-11-05T09:00:00Z")
    }
}

fn add_profile_version(room: &impl MetaRoom, params: &WpiszWersjeProfiluParams) {
    poszukiwanie_profile::wpisz_wersje_profilu(room, params).expect("add a version of the profile");
}

fn feature(wersja_id: &str, cecha: &str, waga: CechyWaga) -> DopiszCecheProfiluParams {
    DopiszCecheProfiluParams {
        wersja_id: wersja_id.to_string(),
        cecha: cecha.to_string(),
        waga,
    }
}

fn financing(
    wersja_id: &str,
    zrodlo: FinansowanieZrodlo,
    kwota: i64,
) -> DopiszFinansowanieProfiluParams {
    DopiszFinansowanieProfiluParams {
        wersja_id: wersja_id.to_string(),
        zrodlo,
        kwota,
    }
}

/// Writes the client's consent `zgoda-<version>` and confirms the version of the profile with it.
fn confirm_profile(room: &impl MetaRoom, version_id: &str) {
    let consent_id = format!("zgoda-{version_id}");
    add_consent(
        room,
        &client_consent(
            &consent_id,
            "potwierdzenie_profilu",
            "poszukiwanie_profile_wersje",
            version_id,
        ),
    );

    let confirmation = PotwierdzWersjeProfiluParams {
        zgoda_id: consent_id,
        id: version_id.to_string(),
    };
    poszukiwanie_profile::potwierdz_wersje_profilu(room, &confirmation)
        .expect("confirm the version of the profile");
}

/// Writes the profile, unless there is one, and its version `wersja-1` from 2026-11-05 at 09:00
/// (see [`profile_version`]), confirmed by the client.
fn confirmed_profile(room: &impl MetaRoom) {
    let profiles = poszukiwanie_profile::profile_poszukiwania(room).expect("read the profiles");
    if profiles.is_empty() {
        open_profile(room, "Mieszkanie dla rodziny");
    }

    add_profile_version(room, &profile_version("wersja-1", "2026-11-05T09:00:00Z"));
    confirm_profile(room, "wersja-1");
}

/// The default parameters of a proposal from otodom.pl for the version `wersja-1`, listed by the
/// owner for 660 000 zł.
fn proposal(proposal_id: &str) -> WpiszPropozycjeParams {
    WpiszPropozycjeParams {
        id: proposal_id.to_string(),
        profil_wersja_id: "wersja-1".to_string(),
        zrodlo: ZrodloPropozycji::Portal,
        ogloszenie_sciezka: Some(format!("propozycje/{proposal_id}.html")),
        ogloszenie_skrot: Some(format!("sha256-{proposal_id}")),
        portal: Some("otodom.pl".to_string()),
        numer: Some("OT-445566".to_string()),
        katalog_odnosnik: None,
        katalog_wersja_od: None,
        prowadzi: Prowadzi::Wlasciciel,
        niepubliczna: false,
        zgodnosc: "3 pokoje, Krzyki, balkon".to_string(),
        niespelnione_zyczenia: None,
        rozbieznosci: None,
        numer_kw: None,
        cena_ogloszenia: 66_000_000,
        ceny_rcn_dokument_id: None,
        utworzono: "2026-11-06T09:00:00Z".to_string(),
    }
}

/// The parameters of the proposal `propozycja` the rule scenarios vary: [`proposal`] with a
/// shorter match.
fn proposal_with_short_match() -> WpiszPropozycjeParams {
    WpiszPropozycjeParams {
        zgodnosc: "3 pokoje, Krzyki".to_string(),
        ..proposal("propozycja")
    }
}

/// The parameters of the proposal `propozycja` from the office's catalog, not public, led by the
/// offer's keeper.
fn catalog_proposal() -> WpiszPropozycjeParams {
    WpiszPropozycjeParams {
        zrodlo: ZrodloPropozycji::KatalogBiura,
        ogloszenie_sciezka: None,
        ogloszenie_skrot: None,
        portal: None,
        numer: None,
        katalog_odnosnik: Some("katalog-ofert/oferta-041".to_string()),
        katalog_wersja_od: Some("2026-11-05T10:00:00Z".to_string()),
        prowadzi: Prowadzi::OpiekunOferty,
        niepubliczna: true,
        ..proposal_with_short_match()
    }
}

fn add_proposal(room: &impl MetaRoom, params: &WpiszPropozycjeParams) {
    poszukiwanie_propozycje::wpisz_propozycje(room, params).expect("add the proposal");
}

/// The parameters of presenting the proposal `propozycja` to the client at `przedstawiono`.
fn presentation(przedstawiono: &str) -> OdnotujPrzedstawieniePropozycjiParams {
    OdnotujPrzedstawieniePropozycjiParams {
        przedstawiono: przedstawiono.to_string(),
        id: "propozycja".to_string(),
    }
}

/// Writes the proposal of `params` and presents it to the client on 2026-11-06 at 10:00.
fn presented_proposal(room: &impl MetaRoom, params: &WpiszPropozycjeParams) {
    add_proposal(room, params);

    let presented = OdnotujPrzedstawieniePropozycjiParams {
        przedstawiono: "2026-11-06T10:00:00Z".to_string(),
        id: params.id.clone(),
    };
    poszukiwanie_propozycje::odnotuj_przedstawienie_propozycji(room, &presented)
        .expect("present the proposal");
}

/// A search case with a confirmed profile and a proposal the client got.
fn offer_ready(room: &impl MetaRoom) {
    search_case(room, SposobZawarcia::WLokaluBiura, None);
    confirmed_profile(room);
    presented_proposal(room, &proposal("propozycja"));
}

/// The parameters of the client's offer `oferta` the rule scenarios vary: 610 000 zł for the
/// proposal `propozycja`, binding until 2026-11-12, with nothing on financing.
fn offer_without_financing() -> PrzygotujOferteKlientaParams {
    PrzygotujOferteKlientaParams {
        id: "oferta".to_string(),
        dotyczy: Dotyczy::Kupno,
        propozycja_id: "propozycja".to_string(),
        adresat: "Jan Nowak, właściciel, bez biura".to_string(),
        odpowiada_na_id: None,
        wpisano: "2026-11-08T09:00:00Z".to_string(),
        kwota: 61_000_000,
        zadatek_albo_kaucja: None,
        termin_umowy: None,
        termin_wydania: None,
        finansowanie_w_ofercie: None,
        warunki_zastrzezone: None,
        wiaze_do: Some("2026-11-12".to_string()),
        sposob_zlozenia: None,
    }
}

/// The default parameters of the client's offer: [`offer_without_financing`] with a part of the
/// price from a mortgage.
fn client_offer(offer_id: &str) -> PrzygotujOferteKlientaParams {
    PrzygotujOferteKlientaParams {
        id: offer_id.to_string(),
        finansowanie_w_ofercie: Some("część ceny z kredytu hipotecznego".to_string()),
        ..offer_without_financing()
    }
}

fn prepare_client_offer(room: &impl MetaRoom, params: &PrzygotujOferteKlientaParams) {
    poszukiwanie_oferty::przygotuj_oferte_klienta(room, params)
        .expect("prepare the client's offer");
}

/// Writes the client's consent `zgoda-<offer>` and confirms the offer with it.
fn confirm_offer(room: &impl MetaRoom, offer_id: &str) {
    let consent_id = format!("zgoda-{offer_id}");
    add_consent(
        room,
        &client_consent(
            &consent_id,
            "potwierdzenie_oferty",
            "poszukiwanie_oferty_wpisy",
            offer_id,
        ),
    );

    let confirmation = PotwierdzOferteKlientaParams {
        zgoda_id: consent_id,
        id: offer_id.to_string(),
    };
    poszukiwanie_oferty::potwierdz_oferte_klienta(room, &confirmation)
        .expect("confirm the client's offer");
}

fn letter_of_offer(room: &impl MetaRoom, offer_id: &str, letter_id: &str) {
    let params = WskazWysylkeOfertyKlientaParams {
        wysylka_id: letter_id.to_string(),
        id: offer_id.to_string(),
    };

    poszukiwanie_oferty::wskaz_wysylke_oferty_klienta(room, &params)
        .expect("name the letter of the offer");
}

/// An offer letter to the other side, waiting for the human's consent `wyslanie-<letter>` to send
/// it.
fn letter_awaiting_consent(room: &impl MetaRoom, letter_id: &str) {
    let for_consent = PrzekazWysylkeDoZgodyParams {
        id: letter_id.to_string(),
    };
    sprawa_korespondencja::przekaz_wysylke_do_zgody(room, &for_consent)
        .expect("hand the letter over for consent");

    approved_other(
        room,
        &decision_request_about(
            &format!("wyslanie-{letter_id}"),
            RodzajDecyzji::Wyslanie,
            "sprawa_korespondencja_wysylki",
            letter_id,
        ),
    );
}

/// Writes the draft of an offer letter to Jan Nowak, the owner.
fn prepare_offer_letter(room: &impl MetaRoom, letter_id: &str) {
    prepare_letter(
        room,
        &PrzygotujWysylkeParams {
            adresat_rodzaj: Some(WysylkiAdresatRodzaj::Inny),
            adresat_nazwa: Some("Jan Nowak".to_string()),
            wzor: Some("oferta-zakupu".to_string()),
            ..letter(letter_id)
        },
    );
}

/// Sends the letter `letter_id` at `wyslano` with the human's consent `wyslanie-<letter>`.
fn send_with_consent(room: &impl MetaRoom, letter_id: &str, wyslano: &str) {
    send(
        room,
        &OdnotujWyslanieParams {
            decyzja_id: Some(format!("wyslanie-{letter_id}")),
            wyslano: wyslano.to_string(),
            ..sending(letter_id)
        },
    );
}

/// A client offer of `params` the client confirmed and a human allowed to send; its letter
/// `wysylka-<offer>` goes out at `wyslano` (the Python default is 2026-11-08 at 12:00).
fn sent_client_offer(room: &impl MetaRoom, params: &PrzygotujOferteKlientaParams, wyslano: &str) {
    prepare_client_offer(room, params);
    confirm_offer(room, &params.id);

    let letter_id = format!("wysylka-{}", params.id);
    prepare_offer_letter(room, &letter_id);
    letter_of_offer(room, &params.id, &letter_id);
    letter_awaiting_consent(room, &letter_id);
    send_with_consent(room, &letter_id, wyslano);
}

/// The parameters of an inflow `inflow_id` from `nadawca`, with no participant named as its sender.
fn inflow_from(inflow_id: &str, nadawca: &str) -> WpiszWplywParams {
    WpiszWplywParams {
        nadawca: nadawca.to_string(),
        ..inflow(inflow_id)
    }
}

/// The parameters of the other side's counteroffer `kontroferta` of 660 000 zł, in writing, which
/// answers `odpowiada_na_id` and came in the inflow `wplyw`.
fn counteroffer(odpowiada_na_id: &str) -> WpiszKontroferteDrugiejStronyParams {
    WpiszKontroferteDrugiejStronyParams {
        id: "kontroferta".to_string(),
        dotyczy: Dotyczy::Kupno,
        propozycja_id: "propozycja".to_string(),
        adresat: "Jan Nowak".to_string(),
        odpowiada_na_id: odpowiada_na_id.to_string(),
        wplyw_id: "wplyw".to_string(),
        wpisano: "2026-11-09T09:00:00Z".to_string(),
        kwota: 66_000_000,
        zadatek_albo_kaucja: None,
        termin_umowy: None,
        termin_wydania: None,
        finansowanie_w_ofercie: None,
        warunki_zastrzezone: None,
        wiaze_do: None,
        sposob_zlozenia: Some(SposobZlozenia::NaPismie),
    }
}

/// Writes the owner's answer to the offer `answers_id` with a counteroffer `counter_id` of
/// 660 000 zł, which came in the inflow `wplyw-<counter>`.
fn counteroffer_of_other_side(room: &impl MetaRoom, counter_id: &str, answers_id: &str) {
    let inflow_id = format!("wplyw-{counter_id}");
    add_inflow(room, &inflow_from(&inflow_id, "Jan Nowak <jan@example.pl>"));

    let answer = OdnotujOdpowiedzDrugiejStronyParams {
        odpowiedz_drugiej_strony: OdpowiedzDrugiejStrony::Kontroferta,
        odpowiedz_wplyw_id: inflow_id.clone(),
        odpowiedz_przedstawiono: Some("2026-11-09T10:00:00Z".to_string()),
        id: answers_id.to_string(),
    };
    poszukiwanie_oferty::odnotuj_odpowiedz_drugiej_strony(room, &answer)
        .expect("record the other side's answer");

    let counter = WpiszKontroferteDrugiejStronyParams {
        id: counter_id.to_string(),
        adresat: "Jan Nowak, właściciel, bez biura".to_string(),
        wplyw_id: inflow_id,
        ..counteroffer(answers_id)
    };
    poszukiwanie_oferty::wpisz_kontroferte_drugiej_strony(room, &counter)
        .expect("add the other side's counteroffer");
}

fn present_counteroffer(room: &impl MetaRoom, counter_id: &str) {
    let params = OdnotujPrzedstawienieKontrofertyParams {
        przedstawiono: "2026-11-09T10:00:00Z".to_string(),
        id: counter_id.to_string(),
    };

    poszukiwanie_oferty::odnotuj_przedstawienie_kontroferty(room, &params)
        .expect("present the counteroffer");
}

/// The parameters of the agent's change of the stage of the case at `od`, with no decision.
fn stage_change(etap: &str, od: &str) -> ZmienEtapSprawyParams {
    ZmienEtapSprawyParams {
        etap: etap.to_string(),
        od: od.to_string(),
        zmienil: "agent-poszukiwania".to_string(),
        decyzja_id: None,
    }
}

/// The parameters of a change of the stage by someone else, which the rule scenarios expect to be
/// refused.
fn stage_change_by_another(etap: &str, od: &str) -> ZmienEtapSprawyParams {
    ZmienEtapSprawyParams {
        zmienil: "a".to_string(),
        ..stage_change(etap, od)
    }
}

fn enter_stage(room: &impl MetaRoom, params: &ZmienEtapSprawyParams) {
    sprawa_metryka::zmien_etap_sprawy(room, params).expect("change the stage of the case");
}

/// Moves the case to `etap` on 2026-11-05 at 10:00 (the Python default of `stage`).
fn stage(room: &impl MetaRoom, etap: &str) {
    enter_stage(room, &stage_change(etap, "2026-11-05T10:00:00Z"));
}

/// The default parameters of a credit analysis made at 2026-11-07 10:00 by the standard
/// parameters: 520–580 thousand zł of credit for 25 years.
fn credit_analysis(analysis_id: &str) -> WpiszAnalizeKredytowaParams {
    WpiszAnalizeKredytowaParams {
        id: analysis_id.to_string(),
        wersja_parametrow: "standard-2026-10".to_string(),
        okres_lat: 25,
        poza_standardem: false,
        poza_standardem_decyzja_id: None,
        kwota_od: 52_000_000,
        kwota_do: 58_000_000,
        rata_od: 340_000,
        rata_do: 380_000,
        udzial_wkladu: 1800,
        wynik_sciezka: format!("analizy/{analysis_id}.json"),
        wynik_skrot: format!("sha256-{analysis_id}"),
        wyjasnienie: "Przy cenie 650 000 zł wkład wynosi około 18% ceny.".to_string(),
        zastrzezenie: STANDARD_DISCLAIMER.to_string(),
        utworzono: "2026-11-07T10:00:00Z".to_string(),
    }
}

/// The parameters of a credit analysis the rule scenarios vary: [`credit_analysis`] with a shorter
/// explanation.
fn credit_analysis_briefly_explained(analysis_id: &str) -> WpiszAnalizeKredytowaParams {
    WpiszAnalizeKredytowaParams {
        wyjasnienie: "Wkład około 18% ceny.".to_string(),
        ..credit_analysis(analysis_id)
    }
}

fn add_credit_analysis(room: &impl MetaRoom, params: &WpiszAnalizeKredytowaParams) {
    poszukiwanie_kredyt::wpisz_analize_kredytowa(room, params).expect("add the credit analysis");
}

fn analysis_input(analiza_id: &str, dana_id: &str) -> DopiszDanaDoAnalizyParams {
    DopiszDanaDoAnalizyParams {
        analiza_id: analiza_id.to_string(),
        dana_id: dana_id.to_string(),
    }
}

/// Writes the client's net income from a certificate the bank requires (`dokument-<data>`),
/// entered at `wpisano`.
fn income_from_document(room: &impl MetaRoom, data_id: &str, wpisano: &str) {
    let document_id = format!("dokument-{data_id}");
    add_received_document(
        room,
        &WpiszOtrzymanyDokumentParams {
            wymaga: Some(WymagaDokumentu::Bank),
            ..received_document(&document_id, RodzajDokumentu::Zaswiadczenie)
        },
    );

    let income = WpiszDanaKredytowaParams {
        id: data_id.to_string(),
        rodzaj: DaneRodzaj::Dochod,
        opis: "umowa o pracę, netto".to_string(),
        wartosc: 900_000,
        okres_zrodla: Some("średnia z 6 miesięcy".to_string()),
        dokument_id: Some(document_id),
        deklaracja: false,
        wpisano: wpisano.to_string(),
    };
    poszukiwanie_kredyt::wpisz_dana_kredytowa(room, &income).expect("add the income");
}

/// The default parameters of a viewing `viewing_id` of the proposal `propozycja` on 2026-11-07 at
/// 17:00, shown by the owner, who gets the client's name and phone.
fn viewing(viewing_id: &str) -> UmowPrezentacjePropozycjiParams {
    UmowPrezentacjePropozycjiParams {
        id: viewing_id.to_string(),
        propozycja_id: "propozycja".to_string(),
        termin: "2026-11-07T17:00:00Z".to_string(),
        kto_pokazuje: KtoPokazuje::Wlasciciel,
        dane_klienta_podane: "imię, nazwisko, telefon".to_string(),
        szerszy_zakres: false,
        szerszy_zakres_decyzja_id: None,
        uczestnicy: None,
    }
}

fn arrange_viewing(room: &impl MetaRoom, params: &UmowPrezentacjePropozycjiParams) {
    poszukiwanie_propozycje::umow_prezentacje_propozycji(room, params)
        .expect("arrange the viewing");
}

/// Writes a search case with a row in every table of the search type, as the profile goes.
fn full_case(room: &impl MetaRoom) {
    open_case(room, "poszukiwanie");
    enter_stage(
        room,
        &stage_change("weryfikacja_klienta", "2026-11-02T10:00:00Z"),
    );

    // The client and his partner, who buys with him but is not a party to the contract.
    client(room);
    add_partner(room);
    give_role(room, "partnerka", "wspolnabywca");

    // The brokerage contract signed at a distance by a consumer: the office starts work before
    // the end of the withdrawal period only on his demand.
    enter_stage(
        room,
        &stage_change("umowa_posrednictwa", "2026-11-03T12:00:00Z"),
    );
    brokerage_contract(room, SposobZawarcia::NaOdleglosc, Some(WITHDRAWAL_ENDS));
    add_consent(
        room,
        &consent("zadanie", "klient", "zadanie_rozpoczecia_pracy"),
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

    // The search profile with its features and financing, confirmed by the client.
    enter_stage(
        room,
        &stage_change("profil_poszukiwania", "2026-11-04T14:00:00Z"),
    );
    open_profile(room, "Mieszkanie dla rodziny");
    add_profile_version(
        room,
        &WpiszWersjeProfiluParams {
            termin: Some("2027-02-28".to_string()),
            termin_powod: Some("koniec obecnego najmu".to_string()),
            ..profile_version("wersja-1", "2026-11-05T09:00:00Z")
        },
    );
    poszukiwanie_profile::dopisz_ceche_profilu(
        room,
        &feature("wersja-1", "balkon", CechyWaga::Warunek),
    )
    .expect("add the balcony");
    poszukiwanie_profile::dopisz_ceche_profilu(
        room,
        &feature("wersja-1", "garaż", CechyWaga::Zyczenie),
    )
    .expect("add the garage");
    poszukiwanie_profile::dopisz_finansowanie_profilu(
        room,
        &financing("wersja-1", FinansowanieZrodlo::Kredyt, 60_000_000),
    )
    .expect("add the credit");
    poszukiwanie_profile::dopisz_finansowanie_profilu(
        room,
        &financing("wersja-1", FinansowanieZrodlo::Oszczednosci, 15_000_000),
    )
    .expect("add the savings");
    confirm_profile(room, "wersja-1");
    enter_stage(room, &stage_change("propozycje", "2026-11-05T10:00:00Z"));

    // The credit analysis from documents, with the bank the client chose and the schedule.
    income_from_document(room, "dochod", "2026-11-07T09:00:00Z");
    poszukiwanie_kredyt::wpisz_dana_kredytowa(
        room,
        &WpiszDanaKredytowaParams {
            id: "rata-samochodu".to_string(),
            rodzaj: DaneRodzaj::Zobowiazanie,
            opis: "rata za samochód według klienta".to_string(),
            wartosc: 80_000,
            okres_zrodla: None,
            dokument_id: None,
            deklaracja: true,
            wpisano: "2026-11-07T09:10:00Z".to_string(),
        },
    )
    .expect("add the car loan the client declared");
    add_credit_analysis(room, &credit_analysis("analiza-1"));
    poszukiwanie_kredyt::dopisz_dana_do_analizy(room, &analysis_input("analiza-1", "dochod"))
        .expect("add the income to the analysis");
    add_inflow(
        room,
        &WpiszWplywParams {
            nadawca_uczestnik_id: Some("klient".to_string()),
            ..inflow_from("wybor-banku", "Marek Wiśniewski")
        },
    );
    poszukiwanie_kredyt::wpisz_bank_klienta(
        room,
        &WpiszBankKlientaParams {
            bank: "Bank Przykładowy S.A.".to_string(),
            wybrano: "2026-11-07".to_string(),
            zrodlo_wplyw_id: "wybor-banku".to_string(),
        },
    )
    .expect("add the client's bank");
    poszukiwanie_kredyt::odnotuj_wymagania_banku(
        room,
        &OdnotujWymaganiaBankuParams {
            wymagania_zrodlo: "pismo banku do klienta".to_string(),
            wymagania_data: "2026-11-07".to_string(),
            id: 1,
        },
    )
    .expect("record the bank's requirements");
    poszukiwanie_kredyt::wpisz_krok_harmonogramu(
        room,
        &WpiszKrokHarmonogramuParams {
            krok: HarmonogramKrok::Wniosek,
            przewidywana: "2026-11-15".to_string(),
        },
    )
    .expect("add the step of the application");
    poszukiwanie_kredyt::odnotuj_krok_harmonogramu(
        room,
        &OdnotujKrokHarmonogramuParams {
            rzeczywista: "2026-11-14".to_string(),
            id: 1,
        },
    )
    .expect("record the step of the application");

    // After three viewings the client allows another district: a new confirmed version.
    add_profile_version(
        room,
        &WpiszWersjeProfiluParams {
            polozenie: "Wrocław, Krzyki albo Fabryczna albo Psie Pole".to_string(),
            ..profile_version("wersja-2", "2026-11-06T08:00:00Z")
        },
    );
    poszukiwanie_profile::dopisz_ceche_profilu(
        room,
        &feature("wersja-2", "balkon", CechyWaga::Zyczenie),
    )
    .expect("add the balcony as a wish");
    confirm_profile(room, "wersja-2");

    // A proposal from a portal with the land register lookup and comparable sales, a viewing
    // with the client's opinion and an open question.
    presented_proposal(
        room,
        &WpiszPropozycjeParams {
            profil_wersja_id: "wersja-2".to_string(),
            numer_kw: Some("WR1K/00098765/4".to_string()),
            ..proposal("propozycja")
        },
    );
    poszukiwanie_propozycje::odnotuj_przegladanie_ksiegi_propozycji(
        room,
        &OdnotujPrzegladanieKsiegiPropozycjiParams {
            numer_kw: "WR1K/00098765/4".to_string(),
            kw_przegladano: "2026-11-06T11:00:00Z".to_string(),
            kw_ogloszajacy_wpisany: OgloszajacyWpisany::Tak,
            kw_wpisy_iii_iv: Some("hipoteka umowna".to_string()),
            id: "propozycja".to_string(),
        },
    )
    .expect("record the land register lookup");
    add_received_document(
        room,
        &WpiszOtrzymanyDokumentParams {
            pochodzenie: Some("rejestr cen nieruchomości".to_string()),
            ..received_document("ceny-rcn", RodzajDokumentu::Inny)
        },
    );
    poszukiwanie_propozycje::odnotuj_ceny_transakcji_propozycji(
        room,
        &OdnotujCenyTransakcjiPropozycjiParams {
            ceny_rcn_dokument_id: "ceny-rcn".to_string(),
            id: "propozycja".to_string(),
        },
    )
    .expect("record the comparable sales");
    poszukiwanie_propozycje::odnotuj_reakcje_na_propozycje(
        room,
        &OdnotujReakcjeNaPropozycjeParams {
            reakcja: Reakcja::ChceObejrzec,
            reakcja_powod: Some("blisko szkoły".to_string()),
            id: "propozycja".to_string(),
        },
    )
    .expect("record the client's reaction");
    arrange_viewing(
        room,
        &UmowPrezentacjePropozycjiParams {
            uczestnicy: Some("klient i partnerka, pośrednik biura".to_string()),
            ..viewing("prezentacja")
        },
    );
    poszukiwanie_propozycje::odnotuj_opinie_klienta(
        room,
        &OdnotujOpinieKlientaParams {
            opinia_klienta: "Podoba się kuchnia, chce złożyć ofertę.".to_string(),
            id: "prezentacja".to_string(),
        },
    )
    .expect("record the client's opinion");
    poszukiwanie_propozycje::odnotuj_notatke_posrednika(
        room,
        &OdnotujNotatkePosrednikaParams {
            notatka_posrednika: "Zawilgocenie w łazience przy oknie.".to_string(),
            id: "prezentacja".to_string(),
        },
    )
    .expect("record the broker's note");
    poszukiwanie_propozycje::dopisz_pytanie_do_prezentacji(
        room,
        &DopiszPytanieDoPrezentacjiParams {
            prezentacja_id: "prezentacja".to_string(),
            pytanie: "Ile wynosi zaliczka do wspólnoty?".to_string(),
        },
    )
    .expect("add the question");
    add_inflow(
        room,
        &inflow_from("odpowiedz-wlasciciela", "Jan Nowak <jan@example.pl>"),
    );
    poszukiwanie_propozycje::odnotuj_odpowiedz_na_pytanie(
        room,
        &OdnotujOdpowiedzNaPytanieParams {
            odpowiedz: "420 zł miesięcznie (twierdzenie właściciela)".to_string(),
            odpowiedz_wplyw_id: "odpowiedz-wlasciciela".to_string(),
            id: 1,
        },
    )
    .expect("record the owner's answer");

    // The chain of offers: the client offers, the owner counters, the client counters again and
    // the owner accepts; every offer of the client waits for the human's consent.
    sent_client_offer(room, &client_offer("oferta"), "2026-11-08T12:00:00Z");
    enter_stage(room, &stage_change("negocjacje", "2026-11-08T12:30:00Z"));
    counteroffer_of_other_side(room, "kontroferta-wlasciciela", "oferta");
    present_counteroffer(room, "kontroferta-wlasciciela");
    poszukiwanie_oferty::odnotuj_decyzje_klienta_o_kontrofercie(
        room,
        &OdnotujDecyzjeKlientaOKontrofercieParams {
            decyzja_klienta: OdpowiedzDrugiejStrony::Kontroferta,
            zgoda_id: None,
            id: "kontroferta-wlasciciela".to_string(),
        },
    )
    .expect("record the client's decision to counter");
    sent_client_offer(
        room,
        &PrzygotujOferteKlientaParams {
            kwota: 64_000_000,
            odpowiada_na_id: Some("kontroferta-wlasciciela".to_string()),
            wpisano: "2026-11-09T11:00:00Z".to_string(),
            ..client_offer("kontroferta-klienta")
        },
        "2026-11-09T12:00:00Z",
    );
    add_inflow(
        room,
        &inflow_from("przyjecie", "Jan Nowak <jan@example.pl>"),
    );
    poszukiwanie_oferty::odnotuj_odpowiedz_drugiej_strony(
        room,
        &OdnotujOdpowiedzDrugiejStronyParams {
            odpowiedz_drugiej_strony: OdpowiedzDrugiejStrony::Przyjeta,
            odpowiedz_wplyw_id: "przyjecie".to_string(),
            odpowiedz_przedstawiono: Some("2026-11-10T09:00:00Z".to_string()),
            id: "kontroferta-klienta".to_string(),
        },
    )
    .expect("record the owner's acceptance");
    enter_stage(
        room,
        &stage_change("oferta_przyjeta", "2026-11-10T10:00:00Z"),
    );

    // The accepted offer goes to a transaction case.
    sprawa_powiazania::wpisz_powiazanie(
        room,
        &WpiszPowiazanieParams {
            rodzaj: RodzajPowiazania::SprawaZrodlowa,
            typ_teczki: Some("transakcja".to_string()),
            oznaczenie: Some("Transakcja 2026/014".to_string()),
            odnosnik: "pokoj-transakcji-014".to_string(),
            wpisal: "tozsamosc-kierownika-posrednictwa".to_string(),
            wpisano: "2026-11-10T11:00:00Z".to_string(),
        },
    )
    .expect("link the transaction case");
}

// The whole case ----------------------------------------------------------------------------------

/// Przebieg poszukiwania: od umowy przez profil, analizę i prezentację do przyjętej oferty
#[test]
fn whole_case() {
    let room = new_room();

    full_case(&room);
    end_contract(&room, "transakcja doszła do skutku");
    enter_stage(
        &room,
        &stage_change("koniec_poszukiwania", "2027-01-20T09:00:00Z"),
    );
    enter_stage(&room, &stage_change("zamknieta", "2027-01-21T09:00:00Z"));

    let stages = sprawa_metryka::etapy(&room).expect("read the stages");
    let stage_names: Vec<&str> = stages.iter().map(|row| row.etap.as_str()).collect();
    assert_eq!(
        stage_names,
        [
            "przyjeta",
            "weryfikacja_klienta",
            "umowa_posrednictwa",
            "profil_poszukiwania",
            "propozycje",
            "negocjacje",
            "oferta_przyjeta",
            "koniec_poszukiwania",
            "zamknieta",
        ]
    );

    let current_profiles =
        poszukiwanie_profile::profile_biezace(&room).expect("read the current profiles");
    let current_profile_ids: Vec<&str> =
        current_profiles.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(current_profile_ids, ["wersja-2"]);

    let current_analysis = poszukiwanie_kredyt::analiza_aktualna(&room)
        .expect("read the current analysis")
        .expect("there is a current analysis");
    assert_eq!(current_analysis.id, "analiza-1");

    let offers = poszukiwanie_oferty::oferty_klienta(&room).expect("read the offers");
    let directions: Vec<Kierunek> = offers.iter().map(|row| row.kierunek).collect();
    assert_eq!(
        directions,
        [
            Kierunek::OdKlienta,
            Kierunek::OdDrugiejStrony,
            Kierunek::OdKlienta,
        ]
    );
    assert_eq!(
        offers[2].odpowiedz_drugiej_strony,
        Some(OdpowiedzDrugiejStrony::Przyjeta)
    );

    let open_questions =
        poszukiwanie_propozycje::pytania_otwarte(&room).expect("read the open questions");
    assert!(open_questions.is_empty());
}

// poszukiwanie-reguly -----------------------------------------------------------------------------

/// Reguły: teczka poszukiwania ma typ pokoju poszukiwanie
#[test]
fn room_type() {
    let room = new_room();
    let record = |typ_pokoju: &str| ZalozMetrykeParams {
        oznaczenie: "Poszukiwanie 2026/017".to_string(),
        typ_pokoju: typ_pokoju.to_string(),
        przyjal: "tozsamosc-posrednika".to_string(),
        przyjeto: "2026-11-02T09:00:00Z".to_string(),
    };

    expect::refused(
        sprawa_metryka::zaloz_metryke(&room, &record("oferta")),
        "teczka poszukiwania ma typ pokoju poszukiwanie",
    );
    sprawa_metryka::zaloz_metryke(&room, &record("poszukiwanie")).expect("open the search case");
}

/// Reguły: etapy poszukiwania idą po kolei; z negocjacji i przyjętej oferty wraca się do propozycji
#[test]
fn stage_transitions() {
    let room = new_room();
    search_case(&room, SposobZawarcia::WLokaluBiura, None);
    confirmed_profile(&room);
    let rule = "etap poszukiwania przechodzi";

    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change_by_another("profil_poszukiwania", "2026-11-05"),
        ),
        rule,
    );
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change_by_another("negocjacje", "2026-11-05"),
        ),
        rule,
    );

    for etap in [
        "weryfikacja_klienta",
        "umowa_posrednictwa",
        "profil_poszukiwania",
        "propozycje",
    ] {
        stage(&room, etap);
    }
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change_by_another("oferta_przyjeta", "2026-11-06"),
        ),
        rule,
    );

    for etap in [
        "negocjacje",
        "propozycje",
        "negocjacje",
        "oferta_przyjeta",
        "propozycje",
    ] {
        stage(&room, etap);
    }
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change_by_another("zamknieta", "2026-11-07"),
        ),
        rule,
    );
    stage(&room, "koniec_poszukiwania");
    stage(&room, "zamknieta");
}

/// Reguły: przerwać wolno każdy etap przed zamknięciem; z przerwania nie ma skoku do innego etapu
#[test]
fn stage_interrupted() {
    let room = new_room();
    search_case(&room, SposobZawarcia::WLokaluBiura, None);
    stage(&room, "weryfikacja_klienta");
    approved_other(
        &room,
        &decision_request("przerwanie", RodzajDecyzji::PrzerwanieSprawy),
    );

    enter_stage(
        &room,
        &ZmienEtapSprawyParams {
            decyzja_id: Some("przerwanie".to_string()),
            ..stage_change("przerwana", "2026-11-05T10:00:00Z")
        },
    );
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change_by_another("koniec_poszukiwania", "2026-11-06"),
        ),
        "etap poszukiwania przechodzi",
    );
    stage(&room, "zamknieta");
}

/// Reguły: wznowione poszukiwanie wraca do etapu sprzed przerwania, który dalej wymaga umowy
#[test]
fn stage_resumed() {
    let room = new_room();
    search_case(&room, SposobZawarcia::WLokaluBiura, None);
    for etap in [
        "weryfikacja_klienta",
        "umowa_posrednictwa",
        "profil_poszukiwania",
    ] {
        stage(&room, etap);
    }
    approved_decision(
        &room,
        &decision_request("przerwanie", RodzajDecyzji::PrzerwanieSprawy),
        "2026-11-03T12:00:00Z",
    );
    enter_stage(
        &room,
        &ZmienEtapSprawyParams {
            decyzja_id: Some("przerwanie".to_string()),
            ..stage_change("przerwana", "2026-11-05T10:00:00Z")
        },
    );
    approved_decision(
        &room,
        &decision_request("wznowienie", RodzajDecyzji::WznowienieSprawy),
        "2026-11-05T12:00:00Z",
    );

    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &ZmienEtapSprawyParams {
                decyzja_id: Some("wznowienie".to_string()),
                ..stage_change("koniec_poszukiwania", "2026-11-05T13:00:00Z")
            },
        ),
        "etap poszukiwania przechodzi",
    );
    enter_stage(
        &room,
        &ZmienEtapSprawyParams {
            decyzja_id: Some("wznowienie".to_string()),
            ..stage_change("profil_poszukiwania", "2026-11-05T13:00:00Z")
        },
    );

    approved_decision(
        &room,
        &decision_request("drugie-przerwanie", RodzajDecyzji::PrzerwanieSprawy),
        "2026-11-03T12:00:00Z",
    );
    enter_stage(
        &room,
        &ZmienEtapSprawyParams {
            decyzja_id: Some("drugie-przerwanie".to_string()),
            ..stage_change("przerwana", "2026-11-06T09:00:00Z")
        },
    );
    end_contract(&room, "wypowiedzenie przez klienta");
    approved_decision(
        &room,
        &decision_request("drugie-wznowienie", RodzajDecyzji::WznowienieSprawy),
        "2026-11-06T10:00:00Z",
    );
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &ZmienEtapSprawyParams {
                decyzja_id: Some("drugie-wznowienie".to_string()),
                ..stage_change("profil_poszukiwania", "2026-11-06T11:00:00Z")
            },
        ),
        "etap profil_poszukiwania wymaga zawartej umowy pośrednictwa",
    );
    enter_stage(&room, &stage_change("zamknieta", "2026-11-06T12:00:00Z"));
}

/// Reguły: profil po zawarciu umowy, propozycje po potwierdzeniu profilu przez klienta
#[test]
fn stages_need_contract_and_profile() {
    let room = new_room();
    open_case(&room, "poszukiwanie");
    client(&room);
    for etap in ["weryfikacja_klienta", "umowa_posrednictwa"] {
        stage(&room, etap);
    }

    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change_by_another("profil_poszukiwania", "2026-11-04"),
        ),
        "etap profil_poszukiwania wymaga zawartej umowy pośrednictwa",
    );
    brokerage_contract(&room, SposobZawarcia::WLokaluBiura, None);
    stage(&room, "profil_poszukiwania");

    open_profile(&room, "Mieszkanie");
    add_profile_version(&room, &profile_version("wersja-1", "2026-11-05T09:00:00Z"));
    let rule = "etap propozycje wymaga profilu poszukiwania potwierdzonego przez klienta";
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(
            &room,
            &stage_change_by_another("propozycje", "2026-11-05"),
        ),
        rule,
    );
    confirm_profile(&room, "wersja-1");
    stage(&room, "propozycje");
}

/// Reguły: role w teczce poszukiwania to klient, współnabywca i małżonek
#[test]
fn roles() {
    let room = new_room();
    add_person(&room, &person("osoba"));

    let co_owner = NadajRoleUczestnikowiParams {
        uczestnik_id: "osoba".to_string(),
        rola: "wspolwlasciciel".to_string(),
        dotyczy_uczestnik_id: None,
    };
    expect::refused(
        sprawa_uczestnicy::nadaj_role_uczestnikowi(&room, &co_owner),
        "rola w teczce poszukiwania",
    );
    give_role(&room, "osoba", "wspolnabywca");
}

/// The parameters of the client's consent `zgoda-<kind>` of `rodzaj`, given on 2026-11-05 in the
/// inflow `wiadomosc`, about no row.
fn consent_of_kind(rodzaj: &str) -> WpiszZgodeParams {
    WpiszZgodeParams {
        id: format!("zgoda-{rodzaj}"),
        uczestnik_id: "klient".to_string(),
        rodzaj: rodzaj.to_string(),
        tresc: "Potwierdzam.".to_string(),
        zlozono: "2026-11-05T09:00:00Z".to_string(),
        dokument_id: None,
        wplyw_id: Some("wiadomosc".to_string()),
        trwaly_nosnik: false,
        dotyczy_tabela: None,
        dotyczy_id: None,
        zastepuje_id: None,
    }
}

/// Reguły: zgody w teczce poszukiwania to potwierdzenia profilu i oferty oraz zgody rdzenia
#[test]
fn consent_kinds() {
    let room = new_room();
    client(&room);
    add_inflow(
        &room,
        &WpiszWplywParams {
            nadawca_uczestnik_id: Some("klient".to_string()),
            ..inflow("wiadomosc")
        },
    );

    expect::refused(
        sprawa_zgody::wpisz_zgode(&room, &consent_of_kind("zatwierdzenie_ceny")),
        "zgoda w teczce poszukiwania",
    );
    sprawa_zgody::wpisz_zgode(&room, &consent_of_kind("potwierdzenie_profilu"))
        .expect("add the confirmation of the profile");
}

/// Reguły: umowa w teczce poszukiwania to umowa pośrednictwa; rachunków i wypłat nie ma
#[test]
fn contract_kind_and_no_money() {
    let room = new_room();
    client(&room);
    add_received_document(&room, &received_document("pismo", RodzajDokumentu::Pismo));
    let rule = "umowa w teczce poszukiwania to umowa pośrednictwa";
    let contract_of = |contract_id: &str, rodzaj: RodzajUmowy| PrzygotujUmoweParams {
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
    };

    expect::refused(
        sprawa_umowy::przygotuj_umowe(&room, &contract_of("zarzadzanie", RodzajUmowy::Zarzadzanie)),
        rule,
    );
    sprawa_umowy::przygotuj_umowe(&room, &contract_of("umowa", RodzajUmowy::Posrednictwo))
        .expect("prepare the brokerage contract");
    expect::refused(
        room.execute("UPDATE sprawa_umowy_wpisy SET rodzaj = 'zarzadzanie'", &[]),
        "rodzaj umowy się nie zmienia",
    );

    let money = "teczka poszukiwania nie przyjmuje pieniędzy";
    expect::refused(
        sprawa_rachunki::wpisz_rachunek(
            &room,
            &WpiszRachunekParams {
                zrodlo_dokument_id: Some("pismo".to_string()),
                ..account("rachunek", "klient")
            },
        ),
        money,
    );
    expect::refused(
        sprawa_wyplaty::przygotuj_wyplate(
            &room,
            &PrzygotujWyplateParams {
                id: "wyplata".to_string(),
                podstawa_tabela: "poszukiwanie_oferty_wpisy".to_string(),
                podstawa_id: "oferta".to_string(),
                kwota: 100,
                wyliczenie_sciezka: "w.json".to_string(),
                wyliczenie_skrot: "sha256-w".to_string(),
                odbiorca_uczestnik_id: "klient".to_string(),
                rachunek_id: "rachunek".to_string(),
                utworzono: "2026-11-03T08:00:00Z".to_string(),
            },
        ),
        money,
    );
}

// poszukiwanie-profile ----------------------------------------------------------------------------

/// Profil: wersja powstaje niepotwierdzona i jest późniejsza niż poprzednie wersje
#[test]
fn profile_version_order() {
    let room = new_room();
    client(&room);
    open_profile(&room, "Mieszkanie");
    add_consent(
        &room,
        &client_consent(
            "zgoda",
            "potwierdzenie_profilu",
            "poszukiwanie_profile_wersje",
            "od-razu",
        ),
    );

    expect::refused(
        room.execute(
            "INSERT INTO poszukiwanie_profile_wersje (id, profil_id, cel, rodzaj_nieruchomosci,
              polozenie, budzet, budzet_z_kosztami, zgoda_id, od)
            VALUES ('od-razu', 'profil', 'kupno', 'mieszkanie', 'Wrocław', 1, 0, 'zgoda',
              '2026-11-05')",
            &[],
        ),
        "wersja profilu powstaje niepotwierdzona",
    );
    add_profile_version(&room, &profile_version("wersja-1", "2026-11-05T09:00:00Z"));
    expect::refused(
        poszukiwanie_profile::wpisz_wersje_profilu(
            &room,
            &WpiszWersjeProfiluParams {
                od: "2026-11-05T09:00:00Z".to_string(),
                ..minimal_profile_version("wersja-2")
            },
        ),
        "nowa wersja profilu jest późniejsza",
    );
    add_profile_version(
        &room,
        &WpiszWersjeProfiluParams {
            od: "2026-11-06T09:00:00Z".to_string(),
            ..minimal_profile_version("wersja-2")
        },
    );
}

/// Profil potwierdza klient zgodą potwierdzenie_profilu, która dotyczy tej wersji
#[test]
fn profile_confirmed_by_client() {
    let room = new_room();
    client(&room);
    add_partner(&room);
    open_profile(&room, "Mieszkanie");
    add_profile_version(&room, &profile_version("wersja-1", "2026-11-05T09:00:00Z"));
    let rule = "wersja profilu ma zgodę potwierdzenie_profilu klienta";
    let confirmation_by = |consent_id: &str| PotwierdzWersjeProfiluParams {
        zgoda_id: consent_id.to_string(),
        id: "wersja-1".to_string(),
    };

    add_consent(
        &room,
        &WpiszZgodeParams {
            uczestnik_id: "partnerka".to_string(),
            ..client_consent(
                "zgoda-partnerki",
                "potwierdzenie_profilu",
                "poszukiwanie_profile_wersje",
                "wersja-1",
            )
        },
    );
    expect::refused(
        poszukiwanie_profile::potwierdz_wersje_profilu(&room, &confirmation_by("zgoda-partnerki")),
        rule,
    );
    add_consent(
        &room,
        &client_consent(
            "zgoda-oferty",
            "potwierdzenie_oferty",
            "poszukiwanie_profile_wersje",
            "wersja-1",
        ),
    );
    expect::refused(
        poszukiwanie_profile::potwierdz_wersje_profilu(&room, &confirmation_by("zgoda-oferty")),
        rule,
    );
    add_consent(
        &room,
        &client_consent(
            "zgoda-innej",
            "potwierdzenie_profilu",
            "poszukiwanie_profile_wersje",
            "inna",
        ),
    );
    expect::refused(
        poszukiwanie_profile::potwierdz_wersje_profilu(&room, &confirmation_by("zgoda-innej")),
        rule,
    );

    confirm_profile(&room, "wersja-1");

    let current_profiles =
        poszukiwanie_profile::profile_biezace(&room).expect("read the current profiles");
    let current_profile_ids: Vec<&str> =
        current_profiles.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(current_profile_ids, ["wersja-1"]);
}

/// Profil: potwierdzonej wersji się nie zmienia i nie dostaje nowych cech ani finansowania
#[test]
fn profile_confirmed_is_fixed() {
    let room = new_room();
    client(&room);
    open_profile(&room, "Mieszkanie");
    add_profile_version(&room, &profile_version("wersja-1", "2026-11-05T09:00:00Z"));
    poszukiwanie_profile::dopisz_ceche_profilu(
        &room,
        &feature("wersja-1", "balkon", CechyWaga::Warunek),
    )
    .expect("add the balcony");
    expect::refused(
        poszukiwanie_profile::dopisz_ceche_profilu(
            &room,
            &feature("wersja-1", "balkon", CechyWaga::Zyczenie),
        ),
        REPLACEMENT,
    );
    confirm_profile(&room, "wersja-1");
    let rule = "cechy i finansowanie dopisuje się tylko do niepotwierdzonej wersji profilu";

    expect::refused(
        poszukiwanie_profile::dopisz_ceche_profilu(
            &room,
            &feature("wersja-1", "garaż", CechyWaga::Zyczenie),
        ),
        rule,
    );
    expect::refused(
        poszukiwanie_profile::dopisz_finansowanie_profilu(
            &room,
            &financing("wersja-1", FinansowanieZrodlo::Kredyt, 1),
        ),
        rule,
    );
    expect::refused(
        room.execute(
            "UPDATE poszukiwanie_profile_wersje SET budzet = 80000000",
            &[],
        ),
        "wersji profilu się nie zmienia",
    );
    expect::refused(
        room.execute(
            "UPDATE poszukiwanie_profile_cechy SET waga = 'zyczenie'",
            &[],
        ),
        "cechy profilu się nie zmienia",
    );
}

/// Profil: dane najmu tylko przy najmie, z okresem, osobami i odpowiedzią o innym lokalu
#[test]
fn profile_rental_fields() {
    let room = new_room();
    open_profile(&room, "Mieszkanie do wynajęcia");
    let rental = WpiszWersjeProfiluParams {
        cel: WersjeCel::Najem,
        budzet: 320_000,
        najem_od: Some("2026-12-01".to_string()),
        osoby: Some(2),
        okazjonalny_dopuszcza: Some(false),
        ..minimal_profile_version("wersja-1")
    };

    expect::refused(
        poszukiwanie_profile::wpisz_wersje_profilu(
            &room,
            &WpiszWersjeProfiluParams {
                osoby: Some(2),
                ..minimal_profile_version("wersja-1")
            },
        ),
        "dane_najmu_tylko_przy_najmie",
    );
    expect::refused(
        poszukiwanie_profile::wpisz_wersje_profilu(
            &room,
            &WpiszWersjeProfiluParams {
                osoby: None,
                ..rental.clone()
            },
        ),
        "najem_ma_okres_osoby_i_rodzaj",
    );
    expect::refused(
        poszukiwanie_profile::wpisz_wersje_profilu(
            &room,
            &WpiszWersjeProfiluParams {
                okazjonalny_dopuszcza: Some(true),
                ..rental.clone()
            },
        ),
        "okazjonalny_z_odpowiedzia_o_innym_lokalu",
    );
    expect::refused(
        poszukiwanie_profile::wpisz_wersje_profilu(
            &room,
            &WpiszWersjeProfiluParams {
                na_wynajem: true,
                ..rental.clone()
            },
        ),
        "na_wynajem_przy_kupnie",
    );
    expect::refused(
        poszukiwanie_profile::wpisz_wersje_profilu(
            &room,
            &WpiszWersjeProfiluParams {
                pow_min: Some(6000),
                pow_max: Some(4000),
                ..minimal_profile_version("wersja-1")
            },
        ),
        "powierzchnia_od_do",
    );
    add_profile_version(
        &room,
        &WpiszWersjeProfiluParams {
            okazjonalny_dopuszcza: Some(true),
            inny_lokal_ma: Some(true),
            ..rental
        },
    );
}

// poszukiwanie-kredyt -----------------------------------------------------------------------------

/// Kredyt: dana z dokumentem z plikiem albo deklaracja klienta; danej się nie zmienia
#[test]
fn credit_data_source() {
    let room = new_room();
    sprawa_dokumenty::wpisz_wymagany_dokument(
        &room,
        &WpiszWymaganyDokumentParams {
            id: "wymagany".to_string(),
            rodzaj: RodzajDokumentu::Zaswiadczenie,
            nazwa: "Zaświadczenie o dochodach".to_string(),
            dotyczy_uczestnik_id: None,
            dotyczy_nieruchomosc_id: None,
            wymaga: Some(WymagaDokumentu::Bank),
        },
    )
    .expect("require the certificate");
    let income_without_source = WpiszDanaKredytowaParams {
        id: "dochod".to_string(),
        rodzaj: DaneRodzaj::Dochod,
        opis: "umowa o pracę, netto".to_string(),
        wartosc: 900_000,
        okres_zrodla: None,
        dokument_id: None,
        deklaracja: false,
        wpisano: "2026-11-07T09:00:00Z".to_string(),
    };

    expect::refused(
        poszukiwanie_kredyt::wpisz_dana_kredytowa(&room, &income_without_source),
        "dana_z_dokumentu_albo_deklaracja",
    );
    expect::refused(
        poszukiwanie_kredyt::wpisz_dana_kredytowa(
            &room,
            &WpiszDanaKredytowaParams {
                dokument_id: Some("wymagany".to_string()),
                ..income_without_source.clone()
            },
        ),
        "dana wejściowa wskazuje dokument z plikiem",
    );
    poszukiwanie_kredyt::wpisz_dana_kredytowa(
        &room,
        &WpiszDanaKredytowaParams {
            deklaracja: true,
            ..income_without_source
        },
    )
    .expect("add the income the client declared");
    expect::refused(
        room.execute("UPDATE poszukiwanie_kredyt_dane SET wartosc = 1", &[]),
        "danej wejściowej się nie zmienia",
    );
}

/// Kredyt: do programu idą tylko dane z dokumentów wpisane przed analizą
#[test]
fn credit_program_inputs() {
    let room = new_room();
    income_from_document(&room, "dochod", "2026-11-07T09:00:00Z");
    income_from_document(&room, "pozniejszy", "2026-11-08T09:00:00Z");
    poszukiwanie_kredyt::wpisz_dana_kredytowa(
        &room,
        &WpiszDanaKredytowaParams {
            id: "deklaracja".to_string(),
            rodzaj: DaneRodzaj::Zobowiazanie,
            opis: "rata według klienta".to_string(),
            wartosc: 80_000,
            okres_zrodla: None,
            dokument_id: None,
            deklaracja: true,
            wpisano: "2026-11-07T09:10:00Z".to_string(),
        },
    )
    .expect("add the loan the client declared");
    add_credit_analysis(&room, &credit_analysis("analiza"));
    let rule = "do programu idą tylko dane z dokumentów wpisane przed analizą";

    expect::refused(
        poszukiwanie_kredyt::dopisz_dana_do_analizy(
            &room,
            &analysis_input("analiza", "deklaracja"),
        ),
        rule,
    );
    expect::refused(
        poszukiwanie_kredyt::dopisz_dana_do_analizy(
            &room,
            &analysis_input("analiza", "pozniejszy"),
        ),
        rule,
    );
    poszukiwanie_kredyt::dopisz_dana_do_analizy(&room, &analysis_input("analiza", "dochod"))
        .expect("add the income to the analysis");
    expect::refused(
        poszukiwanie_kredyt::dopisz_dana_do_analizy(&room, &analysis_input("analiza", "dochod")),
        REPLACEMENT,
    );
}

/// Kredyt: analiza poza standardem ma zatwierdzoną decyzję, która ją wskazuje (R2)
#[test]
fn credit_outside_standard() {
    let room = new_room();
    let rule = "analiza poza standardem wymaga zatwierdzonej decyzji inna";
    let outside_standard_with = |decision_id: &str| WpiszAnalizeKredytowaParams {
        okres_lat: 35,
        poza_standardem: true,
        poza_standardem_decyzja_id: Some(decision_id.to_string()),
        ..credit_analysis_briefly_explained("analiza")
    };

    ask_for_decision(
        &room,
        &PoprosODecyzjeParams {
            prosi_stanowisko: ProsiStanowisko::AnalitykKredytowy,
            ..decision_request_about(
                "czeka",
                RodzajDecyzji::Inna,
                "poszukiwanie_kredyt_analizy",
                "analiza",
            )
        },
    );
    expect::refused(
        poszukiwanie_kredyt::wpisz_analize_kredytowa(&room, &outside_standard_with("czeka")),
        rule,
    );
    approved_other(
        &room,
        &decision_request_about(
            "inna-analiza",
            RodzajDecyzji::Inna,
            "poszukiwanie_kredyt_analizy",
            "inna",
        ),
    );
    expect::refused(
        poszukiwanie_kredyt::wpisz_analize_kredytowa(&room, &outside_standard_with("inna-analiza")),
        rule,
    );
    expect::refused(
        poszukiwanie_kredyt::wpisz_analize_kredytowa(
            &room,
            &WpiszAnalizeKredytowaParams {
                poza_standardem_decyzja_id: Some("inna-analiza".to_string()),
                ..credit_analysis_briefly_explained("analiza")
            },
        ),
        "poza_standardem_z_decyzja",
    );
    approved_other(
        &room,
        &decision_request_about(
            "zgoda",
            RodzajDecyzji::Inna,
            "poszukiwanie_kredyt_analizy",
            "analiza",
        ),
    );
    add_credit_analysis(&room, &outside_standard_with("zgoda"));
}

/// Kredyt: analiza ma zastrzeżenie, nowa odbiera aktualność poprzedniej, wyniku się nie zmienia
#[test]
fn credit_analysis_current() {
    let room = new_room();

    expect::refused(
        poszukiwanie_kredyt::wpisz_analize_kredytowa(
            &room,
            &WpiszAnalizeKredytowaParams {
                zastrzezenie: String::new(),
                ..credit_analysis_briefly_explained("pusta")
            },
        ),
        "zastrzezenie",
    );
    expect::refused(
        poszukiwanie_kredyt::wpisz_analize_kredytowa(
            &room,
            &WpiszAnalizeKredytowaParams {
                kwota_od: 60_000_000,
                ..credit_analysis_briefly_explained("odwrotna")
            },
        ),
        "kwota_od_do",
    );
    expect::refused(
        room.execute(
            "INSERT INTO poszukiwanie_kredyt_analizy (id, wersja_parametrow, okres_lat,
              poza_standardem, kwota_od, kwota_do, rata_od, rata_do, udzial_wkladu, wynik_sciezka,
              wynik_skrot, wyjasnienie, zastrzezenie, aktualna, utworzono)
            VALUES ('stara', 'p', 25, 0, 1, 2, 1, 2, 1800, 'a.json', 'sha256-a', 'w', 'z', 0,
              '2026-11-07')",
            &[],
        ),
        "analiza powstaje jako aktualna",
    );
    add_credit_analysis(&room, &credit_analysis("analiza-1"));
    add_credit_analysis(
        &room,
        &WpiszAnalizeKredytowaParams {
            utworzono: "2026-11-20T10:00:00Z".to_string(),
            ..credit_analysis("analiza-2")
        },
    );

    let current_analysis = poszukiwanie_kredyt::analiza_aktualna(&room)
        .expect("read the current analysis")
        .expect("there is a current analysis");
    assert_eq!(current_analysis.id, "analiza-2");

    let rule = "wyniku analizy się nie zmienia";
    expect::refused(
        room.execute(
            "UPDATE poszukiwanie_kredyt_analizy SET kwota_do = 1 WHERE id = 'analiza-2'",
            &[],
        ),
        rule,
    );
    expect::refused(
        room.execute(
            "UPDATE poszukiwanie_kredyt_analizy SET aktualna = 1 WHERE id = 'analiza-1'",
            &[],
        ),
        rule,
    );
}

/// Kredyt: bank wybiera klient wiadomością; wymagania banku i kroki harmonogramu raz
#[test]
fn credit_bank_and_schedule() {
    let room = new_room();
    client(&room);
    add_inflow(
        &room,
        &WpiszWplywParams {
            nadawca_uczestnik_id: Some("klient".to_string()),
            ..inflow_from("wybor-banku", "Marek")
        },
    );

    // A writer outside the client can leave out the client's message that `wpisz_bank_klienta`
    // requires. The table refuses the bank without one.
    let bank_without_message = room.execute(
        "INSERT INTO poszukiwanie_kredyt_banki (bank, wybrano, zrodlo_wplyw_id)
        VALUES (:bank, :wybrano, :zrodlo_wplyw_id)",
        &[
            ("bank", text("Bank Przykładowy S.A.")),
            ("wybrano", text("2026-11-07")),
            ("zrodlo_wplyw_id", SqlValue::Null),
        ],
    );
    expect::refused(bank_without_message, "NOT NULL constraint failed");
    poszukiwanie_kredyt::wpisz_bank_klienta(
        &room,
        &WpiszBankKlientaParams {
            bank: "Bank Przykładowy S.A.".to_string(),
            wybrano: "2026-11-07".to_string(),
            zrodlo_wplyw_id: "wybor-banku".to_string(),
        },
    )
    .expect("add the client's bank");

    // A writer outside the client can leave out the date that `odnotuj_wymagania_banku` requires.
    // The table refuses the requirements without one.
    let requirements_without_date = room.execute(
        "UPDATE poszukiwanie_kredyt_banki
        SET wymagania_zrodlo = :wymagania_zrodlo,
          wymagania_data = :wymagania_data
        WHERE id = :id",
        &[
            ("wymagania_zrodlo", text("strona banku")),
            ("wymagania_data", SqlValue::Null),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(requirements_without_date, "wymagania_ze_zrodlem_i_data");
    let requirements_from =
        |wymagania_zrodlo: &str, wymagania_data: &str| OdnotujWymaganiaBankuParams {
            wymagania_zrodlo: wymagania_zrodlo.to_string(),
            wymagania_data: wymagania_data.to_string(),
            id: 1,
        };
    poszukiwanie_kredyt::odnotuj_wymagania_banku(
        &room,
        &requirements_from("strona banku", "2026-11-07"),
    )
    .expect("record the bank's requirements");
    let rule = "wyboru banku się nie zmienia";
    expect::refused(
        poszukiwanie_kredyt::odnotuj_wymagania_banku(
            &room,
            &requirements_from("pismo", "2026-11-08"),
        ),
        rule,
    );
    expect::refused(
        room.execute(
            "UPDATE poszukiwanie_kredyt_banki SET bank = 'Inny Bank'",
            &[],
        ),
        rule,
    );

    poszukiwanie_kredyt::wpisz_krok_harmonogramu(
        &room,
        &WpiszKrokHarmonogramuParams {
            krok: HarmonogramKrok::DecyzjaKredytowa,
            przewidywana: "2026-12-01".to_string(),
        },
    )
    .expect("add the step of the credit decision");
    let recalculation_to = |przewidywana: &str| PrzeliczKrokHarmonogramuParams {
        przewidywana: przewidywana.to_string(),
        id: 1,
    };
    let record_of = |rzeczywista: &str| OdnotujKrokHarmonogramuParams {
        rzeczywista: rzeczywista.to_string(),
        id: 1,
    };
    poszukiwanie_kredyt::przelicz_krok_harmonogramu(&room, &recalculation_to("2026-12-05"))
        .expect("recalculate the step");
    poszukiwanie_kredyt::odnotuj_krok_harmonogramu(&room, &record_of("2026-12-04"))
        .expect("record the step");
    let rule = "kroku harmonogramu po fakcie się nie zmienia";
    expect::refused(
        poszukiwanie_kredyt::przelicz_krok_harmonogramu(&room, &recalculation_to("2026-12-06")),
        rule,
    );
    expect::refused(
        poszukiwanie_kredyt::odnotuj_krok_harmonogramu(&room, &record_of("2026-12-03")),
        rule,
    );
}

// poszukiwanie-propozycje -------------------------------------------------------------------------

/// Propozycja wskazuje bieżącą wersję profilu, którą klient potwierdził
#[test]
fn proposal_current_profile() {
    let room = new_room();
    search_case(&room, SposobZawarcia::WLokaluBiura, None);
    confirmed_profile(&room);
    add_profile_version(&room, &profile_version("wersja-2", "2026-11-06T08:00:00Z"));
    let rule = "propozycja wskazuje bieżącą wersję profilu potwierdzoną przez klienta";
    let proposal_for = |profil_wersja_id: &str| WpiszPropozycjeParams {
        profil_wersja_id: profil_wersja_id.to_string(),
        ..proposal_with_short_match()
    };

    expect::refused(
        poszukiwanie_propozycje::wpisz_propozycje(&room, &proposal_for("wersja-2")),
        rule,
    );
    confirm_profile(&room, "wersja-2");
    expect::refused(
        poszukiwanie_propozycje::wpisz_propozycje(&room, &proposal_for("wersja-1")),
        rule,
    );
    add_proposal(&room, &proposal_for("wersja-2"));
}

/// Propozycja: z portalu ma kopię i numer, z katalogu odnośnik i datę wersji
#[test]
fn proposal_source() {
    let room = new_room();
    search_case(&room, SposobZawarcia::WLokaluBiura, None);
    confirmed_profile(&room);

    expect::refused(
        poszukiwanie_propozycje::wpisz_propozycje(
            &room,
            &WpiszPropozycjeParams {
                numer: None,
                ..proposal_with_short_match()
            },
        ),
        "z_portalu_z_kopia_i_numerem",
    );
    expect::refused(
        poszukiwanie_propozycje::wpisz_propozycje(
            &room,
            &WpiszPropozycjeParams {
                katalog_wersja_od: None,
                ..catalog_proposal()
            },
        ),
        "z_katalogu_z_odnosnikiem_i_wersja",
    );
    expect::refused(
        poszukiwanie_propozycje::wpisz_propozycje(
            &room,
            &WpiszPropozycjeParams {
                niepubliczna: true,
                ..proposal_with_short_match()
            },
        ),
        "niepubliczna_tylko_z_katalogu",
    );
    expect::refused(
        poszukiwanie_propozycje::wpisz_propozycje(
            &room,
            &WpiszPropozycjeParams {
                prowadzi: Prowadzi::Wlasciciel,
                ..catalog_proposal()
            },
        ),
        "z_katalogu_prowadzi_opiekun_oferty",
    );
    add_proposal(&room, &catalog_proposal());
}

/// Propozycja: przedstawienie przy zawartej umowie, nie wcześniej niż praca od (art. 15 upk)
#[test]
fn proposal_presented_after_work_starts() {
    let room = new_room();
    search_case(&room, SposobZawarcia::NaOdleglosc, Some(WITHDRAWAL_ENDS));
    confirmed_profile(&room);
    add_proposal(&room, &proposal("propozycja"));
    let rule =
        "propozycję przedstawia się przy zawartej umowie pośrednictwa, nie wcześniej niż praca od";

    expect::refused(
        poszukiwanie_propozycje::odnotuj_przedstawienie_propozycji(
            &room,
            &presentation("2026-11-18T10:00:00Z"),
        ),
        rule,
    );
    poszukiwanie_propozycje::odnotuj_przedstawienie_propozycji(
        &room,
        &presentation("2026-11-19T10:00:00Z"),
    )
    .expect("present the proposal");
    expect::refused(
        poszukiwanie_propozycje::odnotuj_przedstawienie_propozycji(
            &room,
            &presentation("2026-11-20T10:00:00Z"),
        ),
        "propozycji się nie zmienia",
    );
}

/// Propozycja: reakcja po przedstawieniu; po „czeka” klient może odpowiedzieć jeszcze raz
#[test]
fn proposal_reaction() {
    let room = new_room();
    search_case(&room, SposobZawarcia::WLokaluBiura, None);
    confirmed_profile(&room);
    add_proposal(&room, &proposal("propozycja"));
    let reaction =
        |reakcja: Reakcja, reakcja_powod: Option<&str>| OdnotujReakcjeNaPropozycjeParams {
            reakcja,
            reakcja_powod: reakcja_powod.map(str::to_string),
            id: "propozycja".to_string(),
        };

    expect::refused(
        poszukiwanie_propozycje::odnotuj_reakcje_na_propozycje(
            &room,
            &reaction(Reakcja::Czeka, None),
        ),
        "reakcja_po_przedstawieniu",
    );
    poszukiwanie_propozycje::odnotuj_przedstawienie_propozycji(
        &room,
        &presentation("2026-11-06T10:00:00Z"),
    )
    .expect("present the proposal");
    poszukiwanie_propozycje::odnotuj_reakcje_na_propozycje(&room, &reaction(Reakcja::Czeka, None))
        .expect("record that the client waits");
    poszukiwanie_propozycje::odnotuj_reakcje_na_propozycje(
        &room,
        &reaction(Reakcja::Odrzuca, Some("za daleko od szkoły")),
    )
    .expect("record that the client rejects the proposal");
    expect::refused(
        poszukiwanie_propozycje::odnotuj_reakcje_na_propozycje(
            &room,
            &reaction(Reakcja::ChceObejrzec, None),
        ),
        "propozycji się nie zmienia",
    );
}

/// Propozycja: wynik przeglądania księgi z numerem, raz; źródła i ceny się nie zmienia
#[test]
fn proposal_land_register() {
    let room = new_room();
    search_case(&room, SposobZawarcia::WLokaluBiura, None);
    confirmed_profile(&room);
    add_proposal(&room, &proposal("propozycja"));

    // A writer outside the client can leave out the result that
    // `odnotuj_przegladanie_ksiegi_propozycji` requires. The table refuses the lookup without one.
    let lookup_without_result = room.execute(
        "UPDATE poszukiwanie_propozycje_wpisy
        SET numer_kw = :numer_kw,
          kw_przegladano = :kw_przegladano,
          kw_ogloszajacy_wpisany = :kw_ogloszajacy_wpisany,
          kw_wpisy_iii_iv = :kw_wpisy_iii_iv
        WHERE id = :id",
        &[
            ("numer_kw", text("WR1K/00098765/4")),
            ("kw_przegladano", text("2026-11-06T11:00:00Z")),
            ("kw_ogloszajacy_wpisany", SqlValue::Null),
            ("kw_wpisy_iii_iv", SqlValue::Null),
            ("id", text("propozycja")),
        ],
    );
    expect::refused(
        lookup_without_result,
        "przegladanie_ksiegi_z_numerem_i_wynikiem",
    );
    let lookup = |kw_przegladano: &str, kw_ogloszajacy_wpisany: OgloszajacyWpisany| {
        OdnotujPrzegladanieKsiegiPropozycjiParams {
            numer_kw: "WR1K/00098765/4".to_string(),
            kw_przegladano: kw_przegladano.to_string(),
            kw_ogloszajacy_wpisany,
            kw_wpisy_iii_iv: None,
            id: "propozycja".to_string(),
        }
    };
    poszukiwanie_propozycje::odnotuj_przegladanie_ksiegi_propozycji(
        &room,
        &lookup("2026-11-06T11:00:00Z", OgloszajacyWpisany::Nie),
    )
    .expect("record the land register lookup");
    let rule = "propozycji się nie zmienia";
    expect::refused(
        poszukiwanie_propozycje::odnotuj_przegladanie_ksiegi_propozycji(
            &room,
            &lookup("2026-11-07T11:00:00Z", OgloszajacyWpisany::Tak),
        ),
        rule,
    );
    expect::refused(
        room.execute(
            "UPDATE poszukiwanie_propozycje_wpisy SET cena_ogloszenia = 1",
            &[],
        ),
        rule,
    );

    sprawa_dokumenty::wpisz_wymagany_dokument(
        &room,
        &WpiszWymaganyDokumentParams {
            id: "wymagany".to_string(),
            rodzaj: RodzajDokumentu::Inny,
            nazwa: "Ceny transakcji".to_string(),
            dotyczy_uczestnik_id: None,
            dotyczy_nieruchomosc_id: None,
            wymaga: None,
        },
    )
    .expect("require the transaction prices");
    expect::refused(
        poszukiwanie_propozycje::odnotuj_ceny_transakcji_propozycji(
            &room,
            &OdnotujCenyTransakcjiPropozycjiParams {
                ceny_rcn_dokument_id: "wymagany".to_string(),
                id: "propozycja".to_string(),
            },
        ),
        "ceny transakcji z rejestru cen to dokument z plikiem",
    );
}

/// Prezentacja: tylko przedstawionej propozycji; szerszy zakres danych klienta za zgodą
#[test]
fn proposal_viewing() {
    let room = new_room();
    search_case(&room, SposobZawarcia::WLokaluBiura, None);
    confirmed_profile(&room);
    add_proposal(&room, &proposal("propozycja"));

    expect::refused(
        poszukiwanie_propozycje::umow_prezentacje_propozycji(&room, &viewing("prezentacja")),
        "prezentacja dotyczy propozycji przedstawionej klientowi",
    );
    poszukiwanie_propozycje::odnotuj_przedstawienie_propozycji(
        &room,
        &presentation("2026-11-06T10:00:00Z"),
    )
    .expect("present the proposal");
    let rule = "szerszy zakres danych klienta wymaga zatwierdzonej decyzji inna";
    approved_other(
        &room,
        &decision_request_about(
            "inna-prezentacja",
            RodzajDecyzji::Inna,
            "poszukiwanie_propozycje_prezentacje",
            "inna",
        ),
    );
    expect::refused(
        poszukiwanie_propozycje::umow_prezentacje_propozycji(
            &room,
            &UmowPrezentacjePropozycjiParams {
                szerszy_zakres: true,
                szerszy_zakres_decyzja_id: Some("inna-prezentacja".to_string()),
                ..viewing("prezentacja")
            },
        ),
        rule,
    );
    expect::refused(
        poszukiwanie_propozycje::umow_prezentacje_propozycji(
            &room,
            &UmowPrezentacjePropozycjiParams {
                szerszy_zakres_decyzja_id: Some("inna-prezentacja".to_string()),
                ..viewing("prezentacja")
            },
        ),
        "szerszy_zakres_z_decyzja",
    );
    approved_other(
        &room,
        &decision_request_about(
            "szerszy-zakres",
            RodzajDecyzji::Inna,
            "poszukiwanie_propozycje_prezentacje",
            "prezentacja",
        ),
    );
    arrange_viewing(
        &room,
        &UmowPrezentacjePropozycjiParams {
            dane_klienta_podane: "imię, nazwisko, telefon, adres e-mail i pracodawca".to_string(),
            szerszy_zakres: true,
            szerszy_zakres_decyzja_id: Some("szerszy-zakres".to_string()),
            ..viewing("prezentacja")
        },
    );
}

/// Prezentacja: opinia i notatka raz, odwołana bez opinii; pytanie otwarte, odpowiedź raz
#[test]
fn proposal_viewing_record() {
    let room = new_room();
    offer_ready(&room);
    arrange_viewing(&room, &viewing("prezentacja"));
    arrange_viewing(&room, &viewing("odwolana"));
    let rule = "prezentacji się nie zmienia";
    let opinion = |opinia_klienta: &str| OdnotujOpinieKlientaParams {
        opinia_klienta: opinia_klienta.to_string(),
        id: "prezentacja".to_string(),
    };
    let cancellation_of = |viewing_id: &str| OdwolajPrezentacjePropozycjiParams {
        odwolano: "2026-11-07T12:00:00Z".to_string(),
        odwolal: "właściciel".to_string(),
        powod_odwolania: "wyjazd".to_string(),
        id: viewing_id.to_string(),
    };

    poszukiwanie_propozycje::odnotuj_opinie_klienta(&room, &opinion("Za ciemno."))
        .expect("record the client's opinion");
    expect::refused(
        poszukiwanie_propozycje::odnotuj_opinie_klienta(&room, &opinion("Jasno.")),
        rule,
    );
    expect::refused(
        room.execute(
            "UPDATE poszukiwanie_propozycje_prezentacje SET termin = '2026-11-08'",
            &[],
        ),
        rule,
    );
    expect::refused(
        poszukiwanie_propozycje::odwolaj_prezentacje_propozycji(
            &room,
            &cancellation_of("prezentacja"),
        ),
        "odwolana_bez_opinii_i_notatki",
    );
    poszukiwanie_propozycje::odwolaj_prezentacje_propozycji(&room, &cancellation_of("odwolana"))
        .expect("cancel the viewing");
    expect::refused(
        poszukiwanie_propozycje::odnotuj_notatke_posrednika(
            &room,
            &OdnotujNotatkePosrednikaParams {
                notatka_posrednika: "x".to_string(),
                id: "odwolana".to_string(),
            },
        ),
        rule,
    );

    poszukiwanie_propozycje::dopisz_pytanie_do_prezentacji(
        &room,
        &DopiszPytanieDoPrezentacjiParams {
            prezentacja_id: "prezentacja".to_string(),
            pytanie: "Ile czynszu?".to_string(),
        },
    )
    .expect("add the question");
    add_inflow(&room, &inflow_from("odpowiedz", "Jan Nowak"));
    expect::refused(
        room.execute(
            "UPDATE poszukiwanie_propozycje_pytania SET odpowiedz = '420 zł' WHERE id = 1",
            &[],
        ),
        "odpowiedz_z_wplywem",
    );
    let answer = |odpowiedz: &str| OdnotujOdpowiedzNaPytanieParams {
        odpowiedz: odpowiedz.to_string(),
        odpowiedz_wplyw_id: "odpowiedz".to_string(),
        id: 1,
    };
    poszukiwanie_propozycje::odnotuj_odpowiedz_na_pytanie(&room, &answer("420 zł"))
        .expect("record the answer");
    expect::refused(
        poszukiwanie_propozycje::odnotuj_odpowiedz_na_pytanie(&room, &answer("450 zł")),
        "pytania się nie zmienia",
    );

    expect::refused(
        room.execute(
            "INSERT INTO poszukiwanie_propozycje_pytania (prezentacja_id, pytanie, odpowiedz,
              odpowiedz_wplyw_id)
            VALUES ('prezentacja', 'Czy jest piwnica?', 'tak', 'odpowiedz')",
            &[],
        ),
        "pytanie powstaje otwarte",
    );
    expect::refused(
        room.execute(
            "INSERT INTO poszukiwanie_propozycje_prezentacje (id, propozycja_id, termin,
              kto_pokazuje, dane_klienta_podane, szerszy_zakres, opinia_klienta)
            VALUES ('z-opinia', 'propozycja', '2026-11-07', 'wlasciciel', 'imię', 0, 'dobra')",
            &[],
        ),
        "prezentacja powstaje umówiona",
    );
    expect::refused(
        room.execute(
            "INSERT INTO poszukiwanie_propozycje_wpisy (id, profil_wersja_id, zrodlo, prowadzi,
              niepubliczna, zgodnosc, cena_ogloszenia, utworzono, przedstawiono)
            VALUES ('od-razu', 'wersja-1', 'inne_biuro', 'inne_biuro', 0, 'tak', 1, '2026-11-06',
              '2026-11-06')",
            &[],
        ),
        "propozycja powstaje nieprzedstawiona",
    );
}

// poszukiwanie-oferty -----------------------------------------------------------------------------

/// Oferta klienta dotyczy przedstawionej propozycji i celu jej profilu
#[test]
fn offer_concerns_presented_proposal() {
    let room = new_room();
    search_case(&room, SposobZawarcia::WLokaluBiura, None);
    confirmed_profile(&room);
    add_proposal(&room, &proposal("propozycja"));
    let rule = "oferta dotyczy propozycji przedstawionej klientowi i celu jej profilu";

    expect::refused(
        poszukiwanie_oferty::przygotuj_oferte_klienta(&room, &offer_without_financing()),
        rule,
    );
    poszukiwanie_propozycje::odnotuj_przedstawienie_propozycji(
        &room,
        &presentation("2026-11-06T10:00:00Z"),
    )
    .expect("present the proposal");
    expect::refused(
        poszukiwanie_oferty::przygotuj_oferte_klienta(
            &room,
            &PrzygotujOferteKlientaParams {
                dotyczy: Dotyczy::Najem,
                ..offer_without_financing()
            },
        ),
        rule,
    );
    expect::refused(
        poszukiwanie_oferty::przygotuj_oferte_klienta(
            &room,
            &PrzygotujOferteKlientaParams {
                wiaze_do: None,
                ..offer_without_financing()
            },
        ),
        "oferta_ma_termin_albo_sposob_zlozenia",
    );
    prepare_client_offer(&room, &offer_without_financing());
    expect::refused(
        room.execute("UPDATE poszukiwanie_oferty_wpisy SET kwota = 62000000", &[]),
        "oferty się nie zmienia",
    );
}

/// Oferta klienta: potwierdza ją klient zgodą potwierdzenie_oferty przed wskazaniem wysyłki
#[test]
fn offer_confirmed_by_client() {
    let room = new_room();
    offer_ready(&room);
    add_partner(&room);
    prepare_client_offer(&room, &client_offer("oferta"));
    prepare_offer_letter(&room, "wysylka");
    let rule = "oferta klienta i jego decyzja mają zgodę potwierdzenie_oferty klienta";
    let confirmation_by = |consent_id: &str| PotwierdzOferteKlientaParams {
        zgoda_id: consent_id.to_string(),
        id: "oferta".to_string(),
    };
    let letter_of_offer_params = WskazWysylkeOfertyKlientaParams {
        wysylka_id: "wysylka".to_string(),
        id: "oferta".to_string(),
    };

    expect::refused(
        poszukiwanie_oferty::wskaz_wysylke_oferty_klienta(&room, &letter_of_offer_params),
        "wysylka_po_potwierdzeniu",
    );
    add_consent(
        &room,
        &WpiszZgodeParams {
            uczestnik_id: "partnerka".to_string(),
            ..client_consent(
                "zgoda-partnerki",
                "potwierdzenie_oferty",
                "poszukiwanie_oferty_wpisy",
                "oferta",
            )
        },
    );
    expect::refused(
        poszukiwanie_oferty::potwierdz_oferte_klienta(&room, &confirmation_by("zgoda-partnerki")),
        rule,
    );
    add_consent(
        &room,
        &client_consent(
            "zgoda-profilu",
            "potwierdzenie_profilu",
            "poszukiwanie_oferty_wpisy",
            "oferta",
        ),
    );
    expect::refused(
        poszukiwanie_oferty::potwierdz_oferte_klienta(&room, &confirmation_by("zgoda-profilu")),
        rule,
    );
    confirm_offer(&room, "oferta");
    letter_of_offer(&room, "oferta", "wysylka");
    expect::refused(
        poszukiwanie_oferty::potwierdz_oferte_klienta(&room, &confirmation_by("zgoda-profilu")),
        "oferty się nie zmienia",
    );
}

/// Oferta klienta wychodzi tylko za zgodą człowieka, przy niewysłanej wysyłce
#[test]
fn offer_sent_with_consent() {
    let room = new_room();
    offer_ready(&room);
    prepare_client_offer(&room, &client_offer("oferta"));
    confirm_offer(&room, "oferta");
    prepare_offer_letter(&room, "wyslana");
    send(&room, &sending("wyslana"));

    expect::refused(
        poszukiwanie_oferty::wskaz_wysylke_oferty_klienta(
            &room,
            &WskazWysylkeOfertyKlientaParams {
                wysylka_id: "wyslana".to_string(),
                id: "oferta".to_string(),
            },
        ),
        "oferta i odpowiedź wskazują wysyłkę, która jeszcze nie wyszła",
    );
    prepare_offer_letter(&room, "wysylka");
    letter_of_offer(&room, "oferta", "wysylka");
    expect::refused(
        sprawa_korespondencja::odnotuj_wyslanie(
            &room,
            &OdnotujWyslanieParams {
                wyslano: "2026-11-08T12:00:00Z".to_string(),
                ..sending("wysylka")
            },
        ),
        "oferta klienta i odpowiedź przed wysłaniem czekają na zgodę",
    );
    letter_awaiting_consent(&room, "wysylka");
    send_with_consent(&room, "wysylka", "2026-11-08T12:00:00Z");
}

/// Oferta klienta nie wychodzi przed praca od ani przy znaczniku wątpliwości
#[test]
fn offer_not_before_work_nor_with_doubt() {
    let room = new_room();
    // A consumer's contract at a distance without his demand: work starts on 2026-11-19. The office
    // may prepare the offer earlier, but it goes out only from that day.
    search_case(&room, SposobZawarcia::NaOdleglosc, Some(WITHDRAWAL_ENDS));
    confirmed_profile(&room);
    add_proposal(&room, &proposal("propozycja"));
    poszukiwanie_propozycje::odnotuj_przedstawienie_propozycji(
        &room,
        &presentation("2026-11-19T08:00:00Z"),
    )
    .expect("present the proposal");
    prepare_client_offer(&room, &client_offer("oferta"));
    confirm_offer(&room, "oferta");
    prepare_offer_letter(&room, "wysylka");
    letter_of_offer(&room, "oferta", "wysylka");
    letter_awaiting_consent(&room, "wysylka");
    let sending_at = |wyslano: &str| OdnotujWyslanieParams {
        decyzja_id: Some("wyslanie-wysylka".to_string()),
        wyslano: wyslano.to_string(),
        ..sending("wysylka")
    };

    expect::refused(
        sprawa_korespondencja::odnotuj_wyslanie(&room, &sending_at("2026-11-18T12:00:00Z")),
        "oferta klienta wychodzi przy zawartej umowie pośrednictwa, nie wcześniej niż praca od",
    );
    sprawa_metryka::postaw_watpliwosc(
        &room,
        &PostawWatpliwoscParams {
            id: "watpliwosc".to_string(),
            postawil: "tozsamosc-kontrolera".to_string(),
            postawiono: "2026-11-18T13:00:00Z".to_string(),
        },
    )
    .expect("raise the doubt");
    expect::refused(
        sprawa_korespondencja::odnotuj_wyslanie(&room, &sending_at("2026-11-19T12:00:00Z")),
        "dopóki stoi znacznik wątpliwości, oferta klienta i odpowiedź nie wychodzą",
    );
    ask_for_decision(
        &room,
        &PoprosODecyzjeParams {
            prosi_stanowisko: ProsiStanowisko::Kontroler,
            zlozono: "2026-11-18T14:00:00Z".to_string(),
            ..decision_request_about(
                "zdjecie",
                RodzajDecyzji::ZdjecieWatpliwosci,
                "sprawa_metryka_watpliwosci",
                "watpliwosc",
            )
        },
    );
    approve(&room, "zdjecie", "2026-11-18T15:00:00Z");
    sprawa_metryka::zdejmij_watpliwosc(
        &room,
        &ZdejmijWatpliwoscParams {
            zdjecie_decyzja_id: "zdjecie".to_string(),
            id: "watpliwosc".to_string(),
        },
    )
    .expect("lift the doubt");
    send_with_consent(&room, "wysylka", "2026-11-19T12:00:00Z");
}

/// Kontroferty: druga strona odpowiada na wysłaną ofertę, klient na jej kontrofertę
#[test]
fn offer_counter_chain() {
    let room = new_room();
    offer_ready(&room);
    prepare_client_offer(&room, &client_offer("niewyslana"));
    sent_client_offer(&room, &client_offer("oferta"), "2026-11-08T12:00:00Z");
    add_inflow(&room, &inflow_from("wplyw", "Jan Nowak"));

    expect::refused(
        poszukiwanie_oferty::odnotuj_odpowiedz_drugiej_strony(
            &room,
            &OdnotujOdpowiedzDrugiejStronyParams {
                odpowiedz_drugiej_strony: OdpowiedzDrugiejStrony::Odrzucona,
                odpowiedz_wplyw_id: "wplyw".to_string(),
                odpowiedz_przedstawiono: None,
                id: "niewyslana".to_string(),
            },
        ),
        "odpowiedz_drugiej_strony_z_wplywem",
    );
    let rule = "kontroferta odpowiada na ofertę tej samej propozycji";
    expect::refused(
        poszukiwanie_oferty::wpisz_kontroferte_drugiej_strony(&room, &counteroffer("oferta")),
        rule,
    );
    counteroffer_of_other_side(&room, "kontroferta", "oferta");
    let counter_of_client = PrzygotujOferteKlientaParams {
        id: "kontra".to_string(),
        odpowiada_na_id: Some("kontroferta".to_string()),
        ..offer_without_financing()
    };
    expect::refused(
        poszukiwanie_oferty::przygotuj_oferte_klienta(&room, &counter_of_client),
        rule,
    );
    present_counteroffer(&room, "kontroferta");
    poszukiwanie_oferty::odnotuj_decyzje_klienta_o_kontrofercie(
        &room,
        &OdnotujDecyzjeKlientaOKontrofercieParams {
            decyzja_klienta: OdpowiedzDrugiejStrony::Kontroferta,
            zgoda_id: None,
            id: "kontroferta".to_string(),
        },
    )
    .expect("record the client's decision to counter");
    prepare_client_offer(&room, &counter_of_client);
}

/// Kontroferta drugiej strony: przyjęcie i odrzucenie ze zgodą klienta, odpowiedź za zgodą człowieka
#[test]
fn counteroffer_decision() {
    let room = new_room();
    offer_ready(&room);
    sent_client_offer(&room, &client_offer("oferta"), "2026-11-08T12:00:00Z");
    counteroffer_of_other_side(&room, "kontroferta", "oferta");
    let decision = |decyzja_klienta: OdpowiedzDrugiejStrony, zgoda_id: Option<&str>| {
        OdnotujDecyzjeKlientaOKontrofercieParams {
            decyzja_klienta,
            zgoda_id: zgoda_id.map(str::to_string),
            id: "kontroferta".to_string(),
        }
    };

    expect::refused(
        poszukiwanie_oferty::odnotuj_decyzje_klienta_o_kontrofercie(
            &room,
            &decision(OdpowiedzDrugiejStrony::Kontroferta, None),
        ),
        "decyzja_po_przedstawieniu",
    );
    present_counteroffer(&room, "kontroferta");
    expect::refused(
        poszukiwanie_oferty::odnotuj_decyzje_klienta_o_kontrofercie(
            &room,
            &decision(OdpowiedzDrugiejStrony::Przyjeta, None),
        ),
        "decyzja_klienta_ze_zgoda",
    );
    add_consent(
        &room,
        &client_consent(
            "zgoda-kontroferty",
            "potwierdzenie_oferty",
            "poszukiwanie_oferty_wpisy",
            "kontroferta",
        ),
    );
    expect::refused(
        poszukiwanie_oferty::odnotuj_decyzje_klienta_o_kontrofercie(
            &room,
            &decision(
                OdpowiedzDrugiejStrony::Kontroferta,
                Some("zgoda-kontroferty"),
            ),
        ),
        "decyzja_klienta_ze_zgoda",
    );
    poszukiwanie_oferty::odnotuj_decyzje_klienta_o_kontrofercie(
        &room,
        &decision(OdpowiedzDrugiejStrony::Przyjeta, Some("zgoda-kontroferty")),
    )
    .expect("record the client's acceptance");

    prepare_offer_letter(&room, "przyjecie");
    poszukiwanie_oferty::wskaz_odpowiedz_na_kontroferte(
        &room,
        &WskazOdpowiedzNaKontroferteParams {
            odpowiedz_wysylka_id: "przyjecie".to_string(),
            id: "kontroferta".to_string(),
        },
    )
    .expect("name the letter of the answer");
    expect::refused(
        sprawa_korespondencja::odnotuj_wyslanie(
            &room,
            &OdnotujWyslanieParams {
                wyslano: "2026-11-09T12:00:00Z".to_string(),
                ..sending("przyjecie")
            },
        ),
        "oferta klienta i odpowiedź przed wysłaniem czekają na zgodę",
    );
    letter_awaiting_consent(&room, "przyjecie");
    send_with_consent(&room, "przyjecie", "2026-11-09T12:00:00Z");
    expect::refused(
        poszukiwanie_oferty::odnotuj_decyzje_klienta_o_kontrofercie(
            &room,
            &decision(OdpowiedzDrugiejStrony::Odrzucona, Some("zgoda-kontroferty")),
        ),
        "oferty się nie zmienia",
    );
}

/// Oferta powstaje bez potwierdzenia, wysyłki, odpowiedzi i decyzji
#[test]
fn offer_starts_without_course() {
    let room = new_room();
    offer_ready(&room);
    add_consent(
        &room,
        &client_consent(
            "zgoda",
            "potwierdzenie_oferty",
            "poszukiwanie_oferty_wpisy",
            "od-razu",
        ),
    );

    expect::refused(
        room.execute(
            "INSERT INTO poszukiwanie_oferty_wpisy (id, dotyczy, kierunek, propozycja_id, adresat,
              wpisano, kwota, wiaze_do, zgoda_id)
            VALUES ('od-razu', 'kupno', 'od_klienta', 'propozycja', 'Jan Nowak', '2026-11-08', 1,
              '2026-11-12', 'zgoda')",
            &[],
        ),
        "oferta powstaje bez potwierdzenia, wysyłki, odpowiedzi i decyzji",
    );
    expect::refused(
        room.execute(
            "INSERT INTO poszukiwanie_oferty_wpisy (id, dotyczy, kierunek, propozycja_id, adresat,
              wpisano, kwota, wiaze_do)
            VALUES ('bez-wplywu', 'kupno', 'od_drugiej_strony', 'propozycja', 'Jan Nowak',
              '2026-11-08', 1, '2026-11-12')",
            &[],
        ),
        "kontroferta_drugiej_strony_z_wplywem_w_lancuchu",
    );
}

// R9 and replacement ------------------------------------------------------------------------------

/// The tables of the search type, whose names start with `poszukiwanie_`.
fn search_tables(room: &MemoryRoom) -> Vec<String> {
    let rows = room
        .execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' \
            AND name LIKE 'poszukiwanie\\_%' ESCAPE '\\'",
            &[],
        )
        .expect("read the tables");

    rows.iter()
        .map(|row| row.get::<String>("name").expect("a table has a name"))
        .collect()
}

/// R9: wierszy teczki poszukiwania się nie usuwa
#[test]
fn no_deletion() {
    let room = new_room();
    full_case(&room);

    let tables = search_tables(&room);
    assert_eq!(tables.len(), 13, "{tables:?}");

    for table in &tables {
        let count = room
            .value(&format!("SELECT count(*) FROM {table}"), &[])
            .expect("count the rows");
        let rows = i64::from_sql_value(&count).expect("the count is an integer");
        assert!(rows > 0, "{table} has no row to delete");

        let sql = format!("DELETE FROM {table} WHERE rowid = (SELECT min(rowid) FROM {table})");
        expect::refused(room.execute(&sql, &[]), DELETION);
    }
}

/// Wiersza teczki poszukiwania nie zastępuje INSERT OR REPLACE ani upsert
#[test]
fn no_replacement() {
    let room = new_room();
    full_case(&room);

    for table in &search_tables(&room) {
        let replace = format!("INSERT OR REPLACE {}", first_row_copy(&room, table));
        expect::refused(room.execute(&replace, &[]), REPLACEMENT);

        let upsert = format!(
            "INSERT {} ON CONFLICT (id) DO UPDATE SET id = excluded.id",
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
