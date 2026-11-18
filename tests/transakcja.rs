//! Scenarios of the sale case room (spec/cases/transakcja on the core).
//!
//! Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
//! allows and a write it refuses. The first two scenarios follow a whole sale as the profile
//! describes it: a flat sold with a preliminary agreement and a mortgage, and a plot sold in two
//! notarial deeds because the municipality has a right of pre-emption.
//!
//! The kinds of the core run through the operations of `cases::sprawa`, the kinds of the sale
//! through those of `cases::transakcja`.

use std::collections::HashSet;

use xboards_agencja_nieruchomosci::cases::sprawa::{
    sprawa_decyzje, sprawa_dokumenty, sprawa_korespondencja, sprawa_metryka, sprawa_nieruchomosci,
    sprawa_powiazania, sprawa_protokoly, sprawa_rachunki, sprawa_sprawdzenia, sprawa_terminy,
    sprawa_uczestnicy, sprawa_umocowania, sprawa_umowy, sprawa_zgody,
};
use xboards_agencja_nieruchomosci::cases::transakcja::{
    self, transakcja_analizy, transakcja_umowy,
};
use xboards_agencja_nieruchomosci::xmeta_room::{FromSqlValue, MemoryRoom, MetaRoom, SqlValue};

use sprawa_decyzje::WpisyRodzaj as RodzajDecyzji;
use sprawa_dokumenty::{WpisyRodzaj as RodzajDokumentu, WpiszOtrzymanyDokumentParams};
use sprawa_korespondencja::{WpiszWplywParams, WplywyKanal};
use sprawa_metryka::{ZalozMetrykeParams, ZmienEtapSprawyParams};
use sprawa_nieruchomosci::{
    WpisyPrawo as PrawoDoNieruchomosci, WpisyRodzaj as RodzajNieruchomosci, WpiszNieruchomoscParams,
};
use sprawa_powiazania::{WpisyRodzaj as RodzajPowiazania, WpiszPowiazanieParams};
use sprawa_protokoly::{
    DopiszPozycjeProtokoluParams, PodpiszProtokolParams, PozycjeRodzaj as RodzajPozycjiProtokolu,
    PrzygotujProtokolParams, WpisyRodzaj as RodzajProtokolu,
};
use sprawa_rachunki::{WpisyCel as CelRachunku, WpiszRachunekParams, ZastapRachunekParams};
use sprawa_sprawdzenia::{
    WpisyRodzaj as RodzajSprawdzenia, WpisyWynik as WynikSprawdzenia, WpiszSprawdzenieParams,
};
use sprawa_terminy::{
    OdnotujPotwierdzenieTerminuParams, WpisyPilnujeStanowisko as PilnujeStanowisko,
    WpisyRodzaj as RodzajTerminu, WpiszTerminParams,
};
use sprawa_uczestnicy::{NadajRoleUczestnikowiParams, WpiszUczestnikaParams};
use sprawa_umocowania::{
    WpisyForma as FormaUmocowania, WpisyRodzaj as RodzajUmocowania,
    WpisyZakres as ZakresUmocowania, WpiszUmocowanieParams, ZmienDaneUmocowaniaParams,
};
use sprawa_umowy::{PrzygotujUmoweParams, WpisyRodzaj as RodzajUmowyZBiurem};
use sprawa_zgody::WpiszZgodeParams;
use transakcja_analizy::{
    DopiszRyzykoTransakcjiParams, WpisyPrzedmiot as PrzedmiotAnalizy, WpiszAnalizeTransakcjiParams,
};
use transakcja_umowy::{
    DopiszStroneUmowyTransakcjiParams, OdnotujWniosekDoKsiegiParams, OdnotujWpisDoKsiegiParams,
    OdnotujWplatePlatnosciParams, OdnotujWynikPierwokupuParams,
    OdnotujZawarcieUmowyTransakcjiParams, OdnotujZawiadomienieOPierwokupieParams,
    OdnotujZeUmowaTransakcjiNieDoszlaParams, OtworzPokojWymianyParams, PierwokupyWynik,
    PlatnosciRodzaj as RodzajPlatnosci, PlatnosciZrodlo as ZrodloPlatnosci,
    PrzygotujUmoweTransakcjiParams, RozwiazUmoweTransakcjiParams, StronyStrona as Strona,
    WpisyForma as FormaUmowy, WpisyRodzaj as RodzajUmowy, WpisyStan as StanUmowy,
    WpiszKancelarieParams, WpiszPierwokupParams, WpiszPlatnoscTransakcjiParams,
    WskazRachunekPlatnosciParams, Wynik as WynikPierwokupu, ZakonczKancelarieParams,
    ZmienPlatnoscTransakcjiParams, ZmienProjektUmowyTransakcjiParams,
};

use crate::expect;
use crate::sprawa::{
    account, add_account, add_flat, add_inflow, add_person, add_received_document,
    approved_decision, check, check_of, confirm_account, consent, decision_request,
    decision_request_about, flat, inflow, open_case, person, received_document,
};
use crate::sprawa_porzadek::{first_row_copy, first_row_copy_without_id, unique_keys_of};

fn new_room() -> MemoryRoom {
    transakcja::open_in_memory().expect("open the sale case room")
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

/// The time the broker signs a decision unless a scenario says otherwise.
const SIGNED_AT: &str = "2026-11-03T12:00:00Z";

// Building blocks of a sale -----------------------------------------------------------------------

fn open_sale(room: &impl MetaRoom) {
    open_case(room, "transakcja");
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

/// Moves the case through the stages `etapy`, one after another, as the transaction coordinator.
fn move_to(room: &impl MetaRoom, etapy: &[&str]) {
    for etap in etapy {
        move_by_decision(room, etap, None);
    }
}

/// Moves the case to the stage `etap` as the transaction coordinator, on the decision `decyzja_id`.
fn move_by_decision(room: &impl MetaRoom, etap: &str, decyzja_id: Option<&str>) {
    let params = ZmienEtapSprawyParams {
        zmienil: "tozsamosc-koordynatora".to_string(),
        decyzja_id: decyzja_id.map(str::to_string),
        ..stage_change(etap, "2026-11-05T09:00:00Z")
    };

    sprawa_metryka::zmien_etap_sprawy(room, &params).expect("change the stage of the case");
}

/// The seller, a client of the office, and the buyer with his wife, who consents.
fn add_parties(room: &impl MetaRoom) {
    add_person(
        room,
        &WpiszUczestnikaParams {
            konsument: true,
            klient_biura: true,
            ..person("sprzedajaca")
        },
    );
    add_person(
        room,
        &WpiszUczestnikaParams {
            imiona: Some("Jan".to_string()),
            nazwisko: Some("Nowak".to_string()),
            ..person("kupujacy")
        },
    );
    add_person(
        room,
        &WpiszUczestnikaParams {
            imiona: Some("Ewa".to_string()),
            nazwisko: Some("Nowak".to_string()),
            ..person("zona-kupujacego")
        },
    );

    let roles = [
        ("sprzedajaca", "sprzedajacy", None),
        ("kupujacy", "kupujacy", None),
        ("zona-kupujacego", "malzonek", Some("kupujacy")),
    ];
    for (uczestnik_id, rola, dotyczy_uczestnik_id) in roles {
        let role = NadajRoleUczestnikowiParams {
            uczestnik_id: uczestnik_id.to_string(),
            rola: rola.to_string(),
            dotyczy_uczestnik_id: dotyczy_uczestnik_id.map(str::to_string),
        };

        sprawa_uczestnicy::nadaj_role_uczestnikowi(room, &role).expect("give a role");
    }
}

/// The default parameters of the notary office of notary Maria Wiśniewska, in XNotariusz since
/// 2026-11-04 (the Python default id is „kancelaria”).
fn notary_office(office_id: &str) -> WpiszKancelarieParams {
    WpiszKancelarieParams {
        id: office_id.to_string(),
        nazwa: "Kancelaria Notarialna Maria Wiśniewska".to_string(),
        adres: Some("ul. Rynek 1, Wrocław".to_string()),
        notariusz: Some("notariusz Maria Wiśniewska".to_string()),
        xnotariusz: true,
        od: "2026-11-04".to_string(),
    }
}

/// Writes the notary office of `params` (see [`notary_office`]).
fn add_notary_office(room: &impl MetaRoom, params: &WpiszKancelarieParams) {
    transakcja_umowy::wpisz_kancelarie(room, params).expect("add the notary office");
}

/// A human approves the exchange room with the notary office, then it opens.
fn open_exchange_room(room: &impl MetaRoom, office_id: &str) {
    let decision_id = format!("otwarcie-{office_id}");
    let opening = decision_request_about(
        &decision_id,
        RodzajDecyzji::Inna,
        "transakcja_umowy_kancelarie",
        office_id,
    );
    approved_decision(room, &opening, SIGNED_AT);

    let params = OtworzPokojWymianyParams {
        otwarcie_decyzja_id: decision_id,
        pokoj_odnosnik: format!("pokoj-wymiany-{office_id}"),
        id: office_id.to_string(),
    };
    transakcja_umowy::otworz_pokoj_wymiany(room, &params).expect("open the exchange room");
}

/// An account of Anna Kowalska for the payment `cel`, confirmed by another channel, checked and
/// admitted (R4).
fn admitted_account_for(room: &impl MetaRoom, account_id: &str, person_id: &str, cel: CelRachunku) {
    add_account(
        room,
        &WpiszRachunekParams {
            cel,
            posiadacz: "Anna Kowalska".to_string(),
            ..account(account_id, person_id)
        },
    );
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
    approved_decision(room, &admission, SIGNED_AT);
}

/// The default parameters of a draft agreement for 620 000.00 zł with the draft document
/// `projekt-<id>` (the Python defaults are „przedwstepna” and „pisemna”).
fn agreement(
    agreement_id: &str,
    rodzaj: RodzajUmowy,
    forma: FormaUmowy,
) -> PrzygotujUmoweTransakcjiParams {
    PrzygotujUmoweTransakcjiParams {
        id: agreement_id.to_string(),
        rodzaj,
        forma,
        oparta_na_id: None,
        kancelaria_id: None,
        projekt_dokument_id: Some(format!("projekt-{agreement_id}")),
        cena: Some(62_000_000),
        termin_przyrzeczonej: None,
        termin_zaplaty: None,
        termin_wydania: Some("14 dni od zapłaty całej ceny".to_string()),
        warunki: None,
    }
}

/// Writes the received draft `projekt-<id>`, then the draft agreement of `params` (see
/// [`agreement`]).
fn prepare_agreement(room: &impl MetaRoom, params: &PrzygotujUmoweTransakcjiParams) {
    let draft_id = format!("projekt-{}", params.id);
    add_received_document(room, &received_document(&draft_id, RodzajDokumentu::Umowa));

    transakcja_umowy::przygotuj_umowe_transakcji(room, params).expect("prepare the agreement");
}

/// The parameters of the legal analyst's analysis `analysis_id` of the draft `dokument_id` of
/// the agreement `agreement_id`.
fn draft_analysis(
    analysis_id: &str,
    agreement_id: &str,
    dokument_id: &str,
) -> WpiszAnalizeTransakcjiParams {
    WpiszAnalizeTransakcjiParams {
        id: analysis_id.to_string(),
        przedmiot: PrzedmiotAnalizy::ProjektUmowy,
        umowa_id: Some(agreement_id.to_string()),
        dokument_id: Some(dokument_id.to_string()),
        poprzedni_odpis_dokument_id: None,
        sporzadzono: "2026-11-06T09:00:00Z".to_string(),
        wnioski: "Projekt zgodny z księgą; zadatek w rozumieniu art. 394 KC.".to_string(),
        sciezka: format!("analizy/{analysis_id}.typ"),
        skrot: format!("sha256-{analysis_id}"),
    }
}

/// Writes the analysis of `params` (see [`draft_analysis`]).
fn add_analysis(room: &impl MetaRoom, params: &WpiszAnalizeTransakcjiParams) {
    transakcja_analizy::wpisz_analize_transakcji(room, params).expect("add the analysis");
}

/// The legal analyst's analysis `analiza-<id>` of the agreement's current draft `projekt-<id>`.
fn analyse_draft(room: &impl MetaRoom, agreement_id: &str) {
    let analysis = draft_analysis(
        &format!("analiza-{agreement_id}"),
        agreement_id,
        &format!("projekt-{agreement_id}"),
    );

    add_analysis(room, &analysis);
}

/// Writes the signed agreement `podpisana-<id>` and concludes the agreement at `zawarto`; a deed
/// also gets its repertory number (the Python defaults are „pisemna” and 2026-11-10 at 12:00).
fn conclude_agreement(room: &impl MetaRoom, agreement_id: &str, forma: FormaUmowy, zawarto: &str) {
    let is_deed = forma == FormaUmowy::AktNotarialny;
    let signed_kind = if is_deed {
        RodzajDokumentu::AktNotarialny
    } else {
        RodzajDokumentu::Umowa
    };
    let repertorium = if is_deed {
        Some("Rep. A nr 1234/2026".to_string())
    } else {
        None
    };

    let signed_id = format!("podpisana-{agreement_id}");
    add_received_document(room, &received_document(&signed_id, signed_kind));

    let conclusion = OdnotujZawarcieUmowyTransakcjiParams {
        dokument_id: signed_id,
        zawarto: zawarto.to_string(),
        repertorium,
        id: agreement_id.to_string(),
    };
    transakcja_umowy::odnotuj_zawarcie_umowy_transakcji(room, &conclusion)
        .expect("conclude the agreement");
}

/// An agreement of `params` (see [`agreement`]) whose draft the analyst assessed and the parties
/// signed on 2026-11-10 at 12:00.
fn concluded_agreement(room: &impl MetaRoom, params: &PrzygotujUmoweTransakcjiParams) {
    prepare_agreement(room, params);
    analyse_draft(room, &params.id);
    conclude_agreement(room, &params.id, params.forma, "2026-11-10T12:00:00Z");
}

/// A notarial deed of the kind `rodzaj` signed at the notary office of the case (the Python
/// default kind is „przenoszaca”).
fn concluded_deed(room: &impl MetaRoom, agreement_id: &str, rodzaj: RodzajUmowy) {
    let deed = PrzygotujUmoweTransakcjiParams {
        kancelaria_id: Some("kancelaria".to_string()),
        ..agreement(agreement_id, rodzaj, FormaUmowy::AktNotarialny)
    };

    concluded_agreement(room, &deed);
}

/// The default parameters of a payment of 60 000.00 zł of own money, due on 2026-11-12, with no
/// account yet (the Python default kind is „zadatek”).
fn payment(
    payment_id: &str,
    agreement_id: &str,
    rodzaj: RodzajPlatnosci,
) -> WpiszPlatnoscTransakcjiParams {
    WpiszPlatnoscTransakcjiParams {
        id: payment_id.to_string(),
        umowa_id: agreement_id.to_string(),
        rodzaj,
        charakter: None,
        kwota: 6_000_000,
        termin: Some("2026-11-12".to_string()),
        zrodlo: ZrodloPlatnosci::Wlasne,
        bank: None,
        warunki_wyplaty: None,
        rachunek_id: None,
    }
}

/// Writes the payment of `params` (see [`payment`]).
fn add_payment(room: &impl MetaRoom, params: &WpiszPlatnoscTransakcjiParams) {
    transakcja_umowy::wpisz_platnosc_transakcji(room, params).expect("add the payment");
}

/// Writes the bank's confirmation `potwierdzenie-<id>` and records that the money came on
/// 2026-11-12 at 10:00.
fn record_payment(room: &impl MetaRoom, payment_id: &str) {
    let proof_id = format!("potwierdzenie-{payment_id}");
    add_received_document(
        room,
        &received_document(&proof_id, RodzajDokumentu::PotwierdzenieBankowe),
    );

    let params = OdnotujWplatePlatnosciParams {
        wplacono: "2026-11-12T10:00:00Z".to_string(),
        potwierdzenie_dokument_id: proof_id,
        id: payment_id.to_string(),
    };
    transakcja_umowy::odnotuj_wplate_platnosci(room, &params).expect("record the payment");
}

/// Writes the municipality's right of pre-emption „pierwokup-gminy”.
fn add_right_of_preemption(room: &impl MetaRoom) {
    let params = WpiszPierwokupParams {
        id: "pierwokup-gminy".to_string(),
        uprawniony: "Gmina Wrocław".to_string(),
        podstawa: "art. 109 ust. 1 pkt 3 ugn".to_string(),
        zawiadomiono: None,
    };

    transakcja_umowy::wpisz_pierwokup(room, &params).expect("add the right of pre-emption");
}

/// Writes the notary office's message `wiadomosc-pierwokup-gminy` from the exchange room, the
/// notification of the municipality and the result `wynik` of its right of pre-emption.
fn record_preemption_result(room: &impl MetaRoom, wynik: WynikPierwokupu) {
    let message_id = "wiadomosc-pierwokup-gminy";
    add_inflow(
        room,
        &WpiszWplywParams {
            nadawca: "Kancelaria Notarialna".to_string(),
            kanal: WplywyKanal::PokojWymiany,
            ..inflow(message_id)
        },
    );

    let notification = OdnotujZawiadomienieOPierwokupieParams {
        zawiadomiono: "2026-11-11".to_string(),
        id: "pierwokup-gminy".to_string(),
    };
    transakcja_umowy::odnotuj_zawiadomienie_o_pierwokupie(room, &notification)
        .expect("record the notification of the municipality");

    let result = OdnotujWynikPierwokupuParams {
        wynik,
        zrodlo_wplyw_id: message_id.to_string(),
        id: "pierwokup-gminy".to_string(),
    };
    transakcja_umowy::odnotuj_wynik_pierwokupu(room, &result)
        .expect("record the result of the pre-emption");
}

/// The buyer's power of attorney `power_id` in the form `forma` for his brother, who signs the
/// deed for him (the Python defaults are „pelnomocnictwo-kupujacego” and „akt_notarialny”).
fn add_deed_power(room: &impl MetaRoom, power_id: &str, forma: FormaUmocowania) {
    add_person(
        room,
        &WpiszUczestnikaParams {
            imiona: Some("Piotr".to_string()),
            nazwisko: Some("Nowak".to_string()),
            ..person("brat-kupujacego")
        },
    );

    let document_id = format!("dokument-{power_id}");
    add_received_document(
        room,
        &received_document(&document_id, RodzajDokumentu::Pelnomocnictwo),
    );

    let power = WpiszUmocowanieParams {
        id: power_id.to_string(),
        rodzaj: RodzajUmocowania::Pelnomocnictwo,
        mocodawca_uczestnik_id: "kupujacy".to_string(),
        umocowany_uczestnik_id: Some("brat-kupujacego".to_string()),
        umocowane_biuro: false,
        zakres: ZakresUmocowania::DoCzynnosci,
        forma: Some(forma),
        sposob_reprezentacji: None,
        dokument_id: Some(document_id),
        udzielono: Some("2026-11-03".to_string()),
        wazne_do: None,
    };
    sprawa_umocowania::wpisz_umocowanie(room, &power).expect("add the power of attorney");
}

/// The parameters of the party `uczestnik_id` of the agreement, acting on no power.
fn party(umowa_id: &str, uczestnik_id: &str, strona: Strona) -> DopiszStroneUmowyTransakcjiParams {
    DopiszStroneUmowyTransakcjiParams {
        umowa_id: umowa_id.to_string(),
        uczestnik_id: uczestnik_id.to_string(),
        strona,
        umocowanie_id: None,
    }
}

/// Writes the party of the agreement of `params` (see [`party`]).
fn add_party(room: &impl MetaRoom, params: &DopiszStroneUmowyTransakcjiParams) {
    transakcja_umowy::dopisz_strone_umowy_transakcji(room, params)
        .expect("add a party of the agreement");
}

/// Writes the handover protocol of the flat „lokal” with its keys and signs it.
fn signed_handover_protocol(room: &impl MetaRoom) {
    let protocol = PrzygotujProtokolParams {
        id: "protokol-przekazania".to_string(),
        rodzaj: RodzajProtokolu::Przekazanie,
        nieruchomosc_id: "lokal".to_string(),
        chwila: "2026-11-20T10:00:00Z".to_string(),
    };
    sprawa_protokoly::przygotuj_protokol(room, &protocol).expect("prepare the protocol");

    let keys = DopiszPozycjeProtokoluParams {
        protokol_id: "protokol-przekazania".to_string(),
        rodzaj: RodzajPozycjiProtokolu::Klucze,
        nazwa: "klucze do drzwi wejściowych".to_string(),
        numer: None,
        opis_stanu: Some("3 sztuki".to_string()),
        zdjecie_sciezka: None,
        zdjecie_skrot: None,
    };
    sprawa_protokoly::dopisz_pozycje_protokolu(room, &keys).expect("add the keys to the protocol");

    add_received_document(
        room,
        &received_document("podpisany-protokol-przekazania", RodzajDokumentu::Protokol),
    );
    let signature = PodpiszProtokolParams {
        dokument_id: "podpisany-protokol-przekazania".to_string(),
        id: "protokol-przekazania".to_string(),
    };
    sprawa_protokoly::podpisz_protokol(room, &signature).expect("sign the protocol");
}

/// A flat sold with a reservation-free preliminary agreement, a deposit and a mortgage.
fn full_sale_of_flat(room: &impl MetaRoom) {
    open_sale(room);
    add_parties(room);
    add_flat(room, &flat("lokal"));
    sprawa_powiazania::wpisz_powiazanie(
        room,
        &WpiszPowiazanieParams {
            rodzaj: RodzajPowiazania::SprawaZrodlowa,
            typ_teczki: Some("oferta".to_string()),
            oznaczenie: Some("Oferta 2026/041".to_string()),
            odnosnik: "pokoj-oferty-041".to_string(),
            wpisal: "tozsamosc-kierownika-transakcji".to_string(),
            wpisano: "2026-11-02T09:15:00Z".to_string(),
        },
    )
    .expect("add the link to the offer");

    // Preparation: the analyst reads the land register; the notary office gets an exchange room.
    move_to(room, &["przygotowanie"]);
    add_received_document(
        room,
        &WpiszOtrzymanyDokumentParams {
            dotyczy_nieruchomosc_id: Some("lokal".to_string()),
            ..received_document("odpis-kw", RodzajDokumentu::OdpisKsiegiWieczystej)
        },
    );
    add_analysis(
        room,
        &WpiszAnalizeTransakcjiParams {
            id: "analiza-ksiegi".to_string(),
            przedmiot: PrzedmiotAnalizy::Ksiega,
            umowa_id: None,
            dokument_id: Some("odpis-kw".to_string()),
            poprzedni_odpis_dokument_id: None,
            sporzadzono: "2026-11-04T09:00:00Z".to_string(),
            wnioski: "Dział III i IV bez wpisów; sprzedająca jedyną właścicielką.".to_string(),
            sciezka: "analizy/ksiega.typ".to_string(),
            skrot: "sha256-ksiega".to_string(),
        },
    );
    transakcja_analizy::dopisz_ryzyko_transakcji(
        room,
        &DopiszRyzykoTransakcjiParams {
            analiza_id: "analiza-ksiegi".to_string(),
            opis: "Wzmianka o wniosku w dziale III".to_string(),
            skutek: Some("Akt dopiero po wyjaśnieniu wzmianki".to_string()),
        },
    )
    .expect("add the risk of the land register");
    add_notary_office(room, &notary_office("kancelaria"));
    open_exchange_room(room, "kancelaria");
    add_inflow(
        room,
        &WpiszWplywParams {
            nadawca_uczestnik_id: Some("kupujacy".to_string()),
            ..inflow("zgoda-na-informacje")
        },
    );
    sprawa_zgody::wpisz_zgode(
        room,
        &WpiszZgodeParams {
            tresc: "Zgadzam się, żeby biuro dostawało projekt aktu od kancelarii.".to_string(),
            zlozono: "2026-11-04T10:00:00Z".to_string(),
            wplyw_id: Some("zgoda-na-informacje".to_string()),
            trwaly_nosnik: true,
            ..consent(
                "informacje-od-kancelarii",
                "kupujacy",
                "informacje_od_kancelarii",
            )
        },
    )
    .expect("add the buyer's consent");

    // The preliminary agreement in writing with a deposit to the seller's admitted account.
    move_to(room, &["umowa_przedwstepna"]);
    prepare_agreement(
        room,
        &PrzygotujUmoweTransakcjiParams {
            termin_przyrzeczonej: Some("2026-12-15".to_string()),
            ..agreement(
                "przedwstepna",
                RodzajUmowy::Przedwstepna,
                FormaUmowy::Pisemna,
            )
        },
    );
    for (party_id, strona) in [
        ("sprzedajaca", Strona::Sprzedajacy),
        ("kupujacy", Strona::Kupujacy),
    ] {
        add_party(room, &party("przedwstepna", party_id, strona));
    }
    admitted_account_for(
        room,
        "rachunek-zadatku",
        "sprzedajaca",
        CelRachunku::Zadatek,
    );
    add_payment(
        room,
        &WpiszPlatnoscTransakcjiParams {
            rachunek_id: Some("rachunek-zadatku".to_string()),
            ..payment("zadatek", "przedwstepna", RodzajPlatnosci::Zadatek)
        },
    );
    analyse_draft(room, "przedwstepna");
    conclude_agreement(
        room,
        "przedwstepna",
        FormaUmowy::Pisemna,
        "2026-11-10T12:00:00Z",
    );
    record_payment(room, "zadatek");

    // Preparing the deed: the date confirmed by a human, the price from the buyer's loan.
    move_to(room, &["przygotowanie_aktu"]);
    sprawa_terminy::wpisz_termin(
        room,
        &WpiszTerminParams {
            id: "termin-aktu".to_string(),
            rodzaj: RodzajTerminu::ZUmowy,
            co: "Akt notarialny umowy sprzedaży".to_string(),
            zdarzenie_tabela: None,
            zdarzenie_id: None,
            zdarzenie_chwila: None,
            koniec: "2026-12-10T10:00:00Z".to_string(),
            podstawa: Some("§ 2 umowy przedwstępnej".to_string()),
            pilnuje_stanowisko: PilnujeStanowisko::KoordynatorTransakcji,
            kalendarz_odnosnik: Some("kalendarz/akt-polna-7".to_string()),
        },
    )
    .expect("add the date of the deed");
    let confirmation = decision_request_about(
        "potwierdzenie-aktu",
        RodzajDecyzji::PotwierdzenieTerminu,
        "sprawa_terminy_wpisy",
        "termin-aktu",
    );
    approved_decision(room, &confirmation, SIGNED_AT);
    sprawa_terminy::odnotuj_potwierdzenie_terminu(
        room,
        &OdnotujPotwierdzenieTerminuParams {
            potwierdzenie_decyzja_id: "potwierdzenie-aktu".to_string(),
            id: "termin-aktu".to_string(),
        },
    )
    .expect("record the confirmation of the date");
    prepare_agreement(
        room,
        &PrzygotujUmoweTransakcjiParams {
            oparta_na_id: Some("przedwstepna".to_string()),
            kancelaria_id: Some("kancelaria".to_string()),
            termin_zaplaty: Some("2026-12-17".to_string()),
            ..agreement("akt", RodzajUmowy::Przenoszaca, FormaUmowy::AktNotarialny)
        },
    );
    add_deed_power(
        room,
        "pelnomocnictwo-kupujacego",
        FormaUmocowania::AktNotarialny,
    );
    add_party(room, &party("akt", "sprzedajaca", Strona::Sprzedajacy));
    add_party(
        room,
        &DopiszStroneUmowyTransakcjiParams {
            umocowanie_id: Some("pelnomocnictwo-kupujacego".to_string()),
            ..party("akt", "kupujacy", Strona::Kupujacy)
        },
    );
    add_party(room, &party("akt", "zona-kupujacego", Strona::Malzonek));
    admitted_account_for(
        room,
        "rachunek-ceny",
        "sprzedajaca",
        CelRachunku::ZaplataCeny,
    );
    add_payment(
        room,
        &WpiszPlatnoscTransakcjiParams {
            kwota: 56_000_000,
            zrodlo: ZrodloPlatnosci::Kredyt,
            bank: Some("Bank Przykładowy S.A.".to_string()),
            warunki_wyplaty: Some(
                "po przedstawieniu wypisu aktu i wniosku o wpis hipoteki".to_string(),
            ),
            rachunek_id: Some("rachunek-ceny".to_string()),
            ..payment("cena-z-kredytu", "akt", RodzajPlatnosci::Cena)
        },
    );
    analyse_draft(room, "akt");

    // The deed: the notary files the land register application on the same day.
    move_to(room, &["akt"]);
    conclude_agreement(
        room,
        "akt",
        FormaUmowy::AktNotarialny,
        "2026-12-10T11:00:00Z",
    );
    transakcja_umowy::odnotuj_wniosek_do_ksiegi(
        room,
        &OdnotujWniosekDoKsiegiParams {
            wniosek_kw_zlozono: "2026-12-10".to_string(),
            id: "akt".to_string(),
        },
    )
    .expect("record the land register application");
    record_payment(room, "cena-z-kredytu");

    // Handover with a signed protocol, then the buyer's entry in the land register.
    move_to(room, &["przekazanie"]);
    signed_handover_protocol(room);
    move_to(room, &["po_przekazaniu"]);
    transakcja_umowy::odnotuj_wpis_do_ksiegi(
        room,
        &OdnotujWpisDoKsiegiParams {
            wpis_kw: "2027-01-20".to_string(),
            id: "akt".to_string(),
        },
    )
    .expect("record the land register entry");
}

/// A plot the municipality may buy first: a conditional deed, then the transfer deed.
fn full_sale_with_preemption(room: &impl MetaRoom) {
    open_sale(room);
    add_parties(room);
    add_flat(
        room,
        &WpiszNieruchomoscParams {
            rodzaj: RodzajNieruchomosci::Dzialka,
            prawo: PrawoDoNieruchomosci::Wlasnosc,
            numer_budynku: None,
            numer_lokalu: None,
            powierzchnia_uzytkowa: None,
            liczba_pokoi: None,
            pietro: None,
            ..flat("dzialka")
        },
    );
    add_notary_office(room, &notary_office("kancelaria"));
    add_right_of_preemption(room);
    add_analysis(
        room,
        &WpiszAnalizeTransakcjiParams {
            id: "analiza-ograniczen".to_string(),
            przedmiot: PrzedmiotAnalizy::Ograniczenia,
            umowa_id: None,
            dokument_id: None,
            poprzedni_odpis_dokument_id: None,
            sporzadzono: "2026-11-04T09:00:00Z".to_string(),
            wnioski: "Działka w planie na cele publiczne; gminie przysługuje pierwokup."
                .to_string(),
            sciezka: "analizy/ograniczenia.typ".to_string(),
            skrot: "sha256-ograniczenia".to_string(),
        },
    );
    transakcja_analizy::dopisz_ryzyko_transakcji(
        room,
        &DopiszRyzykoTransakcjiParams {
            analiza_id: "analiza-ograniczen".to_string(),
            opis: "Gmina może wykonać pierwokup".to_string(),
            skutek: Some(
                "Dwa akty; sprzedaż bezwarunkowa byłaby nieważna (art. 599 § 2 KC)".to_string(),
            ),
        },
    )
    .expect("add the risk of the pre-emption");
    move_to(
        room,
        &["przygotowanie", "przygotowanie_aktu", "akt_warunkowy"],
    );

    concluded_deed(room, "warunkowa", RodzajUmowy::Warunkowa);
    record_preemption_result(room, WynikPierwokupu::NieWykonano);

    move_to(room, &["akt"]);
    prepare_agreement(
        room,
        &PrzygotujUmoweTransakcjiParams {
            oparta_na_id: Some("warunkowa".to_string()),
            kancelaria_id: Some("kancelaria".to_string()),
            ..agreement("akt", RodzajUmowy::Przenoszaca, FormaUmowy::AktNotarialny)
        },
    );
    add_party(room, &party("akt", "sprzedajaca", Strona::Sprzedajacy));
    add_party(room, &party("akt", "kupujacy", Strona::Kupujacy));
    admitted_account_for(
        room,
        "rachunek-ceny",
        "sprzedajaca",
        CelRachunku::ZaplataCeny,
    );
    add_payment(
        room,
        &WpiszPlatnoscTransakcjiParams {
            kwota: 62_000_000,
            rachunek_id: Some("rachunek-ceny".to_string()),
            ..payment("cena", "akt", RodzajPlatnosci::Cena)
        },
    );
    analyse_draft(room, "akt");
    conclude_agreement(
        room,
        "akt",
        FormaUmowy::AktNotarialny,
        "2026-12-20T11:00:00Z",
    );
    record_payment(room, "cena");
    transakcja_umowy::odnotuj_wniosek_do_ksiegi(
        room,
        &OdnotujWniosekDoKsiegiParams {
            wniosek_kw_zlozono: "2026-12-20".to_string(),
            id: "akt".to_string(),
        },
    )
    .expect("record the land register application");
}

// The whole case ----------------------------------------------------------------------------------

/// Przebieg transakcji lokalu: przedwstępna z zadatkiem, akt z kredytem, przekazanie, księga
#[test]
fn whole_sale_of_flat() {
    let room = new_room();

    full_sale_of_flat(&room);
    move_to(&room, &["zamknieta"]);

    let stages = sprawa_metryka::etapy(&room).expect("read the stages");
    let etapy: Vec<&str> = stages.iter().map(|row| row.etap.as_str()).collect();
    assert_eq!(
        etapy,
        [
            "przyjeta",
            "przygotowanie",
            "umowa_przedwstepna",
            "przygotowanie_aktu",
            "akt",
            "przekazanie",
            "po_przekazaniu",
            "zamknieta",
        ]
    );

    let agreements = transakcja_umowy::umowy_transakcji(&room).expect("read the agreements");
    let preliminary = agreements
        .iter()
        .find(|row| row.id == "przedwstepna")
        .expect("the preliminary agreement exists");
    assert_eq!(preliminary.stan, StanUmowy::Zawarta, "{agreements:?}");
    let deed = agreements
        .iter()
        .find(|row| row.id == "akt")
        .expect("the deed exists");
    assert_eq!(
        deed.wpis_kw.as_deref(),
        Some("2027-01-20"),
        "{agreements:?}"
    );

    let payments = transakcja_umowy::platnosci_transakcji(&room).expect("read the payments");
    assert!(
        payments.iter().all(|row| row.wplacono.is_some()),
        "{payments:?}"
    );

    let offices = transakcja_umowy::kancelarie(&room).expect("read the notary offices");
    assert_eq!(
        offices[0].pokoj_odnosnik.as_deref(),
        Some("pokoj-wymiany-kancelaria")
    );

    let current_analyses =
        transakcja_analizy::analizy_aktualne_transakcji(&room).expect("read the current analyses");
    let current: HashSet<PrzedmiotAnalizy> =
        current_analyses.iter().map(|row| row.przedmiot).collect();
    assert_eq!(
        current,
        HashSet::from([PrzedmiotAnalizy::Ksiega, PrzedmiotAnalizy::ProjektUmowy])
    );
}

/// Przebieg transakcji z pierwokupem gminy: akt warunkowy, wynik pierwokupu, akt przenoszący
#[test]
fn whole_sale_with_preemption() {
    let room = new_room();

    full_sale_with_preemption(&room);

    let stages = sprawa_metryka::etapy(&room).expect("read the stages");
    let etapy: Vec<&str> = stages.iter().map(|row| row.etap.as_str()).collect();
    assert_eq!(
        etapy[etapy.len() - 2..],
        ["akt_warunkowy", "akt"],
        "{etapy:?}"
    );

    let preemptions = transakcja_umowy::pierwokupy(&room).expect("read the rights of pre-emption");
    assert_eq!(preemptions[0].wynik, PierwokupyWynik::NieWykonano);

    let agreements = transakcja_umowy::umowy_transakcji(&room).expect("read the agreements");
    let kinds: Vec<RodzajUmowy> = agreements.iter().map(|row| row.rodzaj).collect();
    assert_eq!(kinds, [RodzajUmowy::Warunkowa, RodzajUmowy::Przenoszaca]);
}

// Rules on the core -------------------------------------------------------------------------------

/// Reguły: teczka transakcji ma typ pokoju transakcja (T2)
#[test]
fn room_type() {
    let room = new_room();

    let rental_record = ZalozMetrykeParams {
        oznaczenie: "Najem 2026/017".to_string(),
        typ_pokoju: "najem".to_string(),
        przyjal: "tozsamosc-posrednika".to_string(),
        przyjeto: "2026-11-02T09:00:00Z".to_string(),
    };
    expect::refused(
        sprawa_metryka::zaloz_metryke(&room, &rental_record),
        "teczka transakcji ma typ pokoju transakcja",
    );

    open_sale(&room);
    let record = sprawa_metryka::metryka(&room)
        .expect("read the record")
        .expect("the case has a record");
    assert_eq!(record.typ_pokoju, "transakcja");
}

/// Reguły: etapy transakcji są fazami i idą po kolei (T8)
#[test]
fn stages_in_order() {
    let room = new_room();
    open_sale(&room);
    let refusal = "etapy transakcji idą po kolei";

    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &stage_change("umowa", "2026-11-05")),
        refusal,
    );
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &stage_change("akt", "2026-11-05")),
        refusal,
    );

    // Without a preliminary agreement the case goes straight to preparing the deed.
    move_to(&room, &["przygotowanie", "przygotowanie_aktu"]);
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &stage_change("umowa_przedwstepna", "2026-11-05")),
        refusal,
    );
    move_to(&room, &["akt"]);
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &stage_change("zamknieta", "2026-11-05")),
        refusal,
    );
}

/// Reguły: każdą fazę można przerwać decyzją, a po przerwaniu przychodzi zamknięcie albo powrót
#[test]
fn stages_interrupted() {
    let room = new_room();
    open_sale(&room);
    move_to(&room, &["przygotowanie", "umowa_przedwstepna"]);

    let interruption = decision_request("przerwanie", RodzajDecyzji::PrzerwanieSprawy);
    approved_decision(&room, &interruption, SIGNED_AT);
    move_by_decision(&room, "przerwana", Some("przerwanie"));
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &stage_change("przygotowanie_aktu", "2026-11-06")),
        "etapy transakcji idą po kolei",
    );
    move_to(&room, &["zamknieta"]);
}

/// Reguły: wznowiona transakcja wraca do fazy sprzed przerwania i idzie dalej po kolei
#[test]
fn stages_resumed() {
    let room = new_room();
    open_sale(&room);
    move_to(&room, &["przygotowanie", "umowa_przedwstepna"]);
    let interruption = decision_request("przerwanie", RodzajDecyzji::PrzerwanieSprawy);
    approved_decision(&room, &interruption, SIGNED_AT);
    move_by_decision(&room, "przerwana", Some("przerwanie"));
    let resumption = decision_request("wznowienie", RodzajDecyzji::WznowienieSprawy);
    approved_decision(&room, &resumption, "2026-11-05T12:00:00Z");

    let skipping_back = ZmienEtapSprawyParams {
        decyzja_id: Some("wznowienie".to_string()),
        ..stage_change("przygotowanie_aktu", "2026-11-06")
    };
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &skipping_back),
        "etapy transakcji idą po kolei",
    );
    move_by_decision(&room, "umowa_przedwstepna", Some("wznowienie"));
    move_to(&room, &["przygotowanie_aktu"]);
}

/// Reguły: faza przekazania wymaga zawartej umowy przenoszącej, a po przekazaniu protokołu
#[test]
fn handover_stage_rules() {
    let room = new_room();
    open_sale(&room);
    add_parties(&room);
    add_flat(&room, &flat("lokal"));
    add_notary_office(&room, &notary_office("kancelaria"));
    move_to(&room, &["przygotowanie", "przygotowanie_aktu", "akt"]);

    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &stage_change("przekazanie", "2026-12-10")),
        "faza przekazania wymaga zawartej umowy przenoszącej własność",
    );
    concluded_deed(&room, "akt", RodzajUmowy::Przenoszaca);
    move_to(&room, &["przekazanie"]);

    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &stage_change("po_przekazaniu", "2026-12-20")),
        "faza po przekazaniu wymaga podpisanego protokołu przekazania",
    );
    signed_handover_protocol(&room);
    move_to(&room, &["po_przekazaniu"]);
}

/// Reguły: role, zgody i brak umów z biurem w teczce transakcji
#[test]
fn roles_consents_and_office_contracts() {
    let room = new_room();
    open_sale(&room);
    add_parties(&room);

    let client_role = NadajRoleUczestnikowiParams {
        uczestnik_id: "kupujacy".to_string(),
        rola: "klient".to_string(),
        dotyczy_uczestnik_id: None,
    };
    expect::refused(
        sprawa_uczestnicy::nadaj_role_uczestnikowi(&room, &client_role),
        "rola w teczce transakcji to sprzedajacy, kupujacy albo malzonek",
    );

    add_inflow(
        &room,
        &WpiszWplywParams {
            nadawca_uczestnik_id: Some("sprzedajaca".to_string()),
            ..inflow("zadanie")
        },
    );
    let consent_of = |consent_id: &str, rodzaj: &str| WpiszZgodeParams {
        tresc: "Treść oświadczenia".to_string(),
        zlozono: "2026-11-04T10:00:00Z".to_string(),
        wplyw_id: Some("zadanie".to_string()),
        trwaly_nosnik: true,
        ..consent(consent_id, "sprzedajaca", rodzaj)
    };
    expect::refused(
        sprawa_zgody::wpisz_zgode(
            &room,
            &consent_of("zadanie-pracy", "zadanie_rozpoczecia_pracy"),
        ),
        "zgoda w teczce transakcji to informacje_od_kancelarii",
    );
    sprawa_zgody::wpisz_zgode(&room, &consent_of("informacje", "informacje_od_kancelarii"))
        .expect("add the consent to information from the notary office");

    let brokerage_contract = PrzygotujUmoweParams {
        id: "umowa-posrednictwa".to_string(),
        rodzaj: RodzajUmowyZBiurem::Posrednictwo,
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
        sprawa_umowy::przygotuj_umowe(&room, &brokerage_contract),
        "w teczce transakcji nie ma umów z biurem",
    );
}

// Notary office -----------------------------------------------------------------------------------

/// Kancelaria: pokój wymiany otwiera zatwierdzona decyzja o tej kancelarii w XNotariuszu
#[test]
fn exchange_room_needs_decision() {
    let room = new_room();
    open_sale(&room);
    expect::refused(
        room.execute(
            "INSERT INTO transakcja_umowy_kancelarie (id, nazwa, xnotariusz, od, do, powod_zmiany)
            VALUES ('od-razu', 'Kancelaria', 1, '2026-11-04', '2026-11-05', 'pomyłka')",
            &[],
        ),
        "kancelaria powstaje bez pokoju wymiany i bez końca",
    );

    add_notary_office(&room, &notary_office("kancelaria"));

    let opening_by = |decyzja_id: &str, office_id: &str| OtworzPokojWymianyParams {
        otwarcie_decyzja_id: decyzja_id.to_string(),
        pokoj_odnosnik: "pokoj".to_string(),
        id: office_id.to_string(),
    };
    expect::refused(
        transakcja_umowy::otworz_pokoj_wymiany(&room, &opening_by("brak", "kancelaria")),
        "pokój wymiany otwiera tylko zatwierdzona decyzja inna, która wskazuje tę kancelarię",
    );
    let about_another_office = decision_request_about(
        "o-innej",
        RodzajDecyzji::Inna,
        "transakcja_umowy_kancelarie",
        "inna-kancelaria",
    );
    approved_decision(&room, &about_another_office, SIGNED_AT);
    expect::refused(
        transakcja_umowy::otworz_pokoj_wymiany(&room, &opening_by("o-innej", "kancelaria")),
        "pokój wymiany otwiera tylko zatwierdzona decyzja inna",
    );

    open_exchange_room(&room, "kancelaria");
    let offices = transakcja_umowy::kancelarie(&room).expect("read the notary offices");
    assert_eq!(
        offices[0].pokoj_odnosnik.as_deref(),
        Some("pokoj-wymiany-kancelaria")
    );

    // Outside XNotariusz there is no exchange room, even with a decision.
    transakcja_umowy::zakoncz_kancelarie(
        &room,
        &ZakonczKancelarieParams {
            r#do: "2026-11-06".to_string(),
            powod_zmiany: "strony wybrały inną".to_string(),
            id: "kancelaria".to_string(),
        },
    )
    .expect("end the notary office");
    add_notary_office(
        &room,
        &WpiszKancelarieParams {
            xnotariusz: false,
            od: "2026-11-06".to_string(),
            ..notary_office("poza-xnotariuszem")
        },
    );
    let opening_outside = decision_request_about(
        "otwarcie-poza",
        RodzajDecyzji::Inna,
        "transakcja_umowy_kancelarie",
        "poza-xnotariuszem",
    );
    approved_decision(&room, &opening_outside, SIGNED_AT);
    expect::refused(
        transakcja_umowy::otworz_pokoj_wymiany(
            &room,
            &opening_by("otwarcie-poza", "poza-xnotariuszem"),
        ),
        "pokoj_tylko_z_xnotariuszem",
    );
}

/// Kancelaria: jedna bieżąca, zmiana kończy poprzednią z powodem, otwartej się nie zmienia
#[test]
fn one_current_office() {
    let room = new_room();
    open_sale(&room);
    add_notary_office(&room, &notary_office("kancelaria"));

    let second_office = WpiszKancelarieParams {
        id: "druga".to_string(),
        nazwa: "Druga kancelaria".to_string(),
        adres: None,
        notariusz: None,
        xnotariusz: true,
        od: "2026-11-05".to_string(),
    };
    expect::refused(
        transakcja_umowy::wpisz_kancelarie(&room, &second_office),
        "teczka ma jedną bieżącą kancelarię",
    );
    open_exchange_room(&room, "kancelaria");
    expect::refused(
        room.execute(
            "UPDATE transakcja_umowy_kancelarie SET notariusz = 'inny notariusz' WHERE id = 'kancelaria'",
            &[],
        ),
        "kancelarii z pokojem wymiany ani zakończonej się nie zmienia",
    );

    // A writer outside the client can leave out the reason that `zakoncz_kancelarie` requires.
    // The table refuses the end without one.
    let end_without_reason = room.execute(
        "UPDATE transakcja_umowy_kancelarie
        SET do = :do,
          powod_zmiany = :powod_zmiany
        WHERE id = :id",
        &[
            ("do", text("2026-11-06")),
            ("powod_zmiany", SqlValue::Null),
            ("id", text("kancelaria")),
        ],
    );
    expect::refused(end_without_reason, "koniec_z_powodem");

    transakcja_umowy::zakoncz_kancelarie(
        &room,
        &ZakonczKancelarieParams {
            r#do: "2026-11-06".to_string(),
            powod_zmiany: "notariusz chory".to_string(),
            id: "kancelaria".to_string(),
        },
    )
    .expect("end the notary office");
    expect::refused(
        room.execute(
            "UPDATE transakcja_umowy_kancelarie SET powod_zmiany = 'inny powód' WHERE id = 'kancelaria'",
            &[],
        ),
        "kancelarii z pokojem wymiany ani zakończonej się nie zmienia",
    );
    add_notary_office(
        &room,
        &WpiszKancelarieParams {
            od: "2026-11-06".to_string(),
            ..notary_office("druga")
        },
    );
}

// Agreements --------------------------------------------------------------------------------------

/// The parameters of a draft agreement with no draft document, price or terms.
fn bare_agreement(
    agreement_id: &str,
    rodzaj: RodzajUmowy,
    forma: FormaUmowy,
) -> PrzygotujUmoweTransakcjiParams {
    PrzygotujUmoweTransakcjiParams {
        projekt_dokument_id: None,
        cena: None,
        termin_wydania: None,
        ..agreement(agreement_id, rodzaj, forma)
    }
}

/// Umowa: forma według rodzaju; warunkowa i przenosząca tylko w akcie (art. 158 KC)
#[test]
fn agreement_form_by_kind() {
    let room = new_room();
    open_sale(&room);

    let wrong_forms = [
        ("a", RodzajUmowy::Przenoszaca, FormaUmowy::Pisemna),
        ("b", RodzajUmowy::Warunkowa, FormaUmowy::Pisemna),
        ("c", RodzajUmowy::Rezerwacyjna, FormaUmowy::AktNotarialny),
    ];
    for (agreement_id, rodzaj, forma) in wrong_forms {
        expect::refused(
            transakcja_umowy::przygotuj_umowe_transakcji(
                &room,
                &bare_agreement(agreement_id, rodzaj, forma),
            ),
            "forma_wedlug_rodzaju",
        );
    }

    let allowed_forms = [
        (
            "przedwstepna-w-akcie",
            RodzajUmowy::Przedwstepna,
            FormaUmowy::AktNotarialny,
        ),
        (
            "rezerwacyjna",
            RodzajUmowy::Rezerwacyjna,
            FormaUmowy::Pisemna,
        ),
    ];
    for (agreement_id, rodzaj, forma) in allowed_forms {
        transakcja_umowy::przygotuj_umowe_transakcji(
            &room,
            &bare_agreement(agreement_id, rodzaj, forma),
        )
        .expect("prepare an agreement in the form of its kind");
    }
}

/// Umowa: powstaje jako projekt i przechodzi do zawartej albo niedoszłej, zawarta do rozwiązanej
#[test]
fn agreement_states() {
    let room = new_room();
    open_sale(&room);

    expect::refused(
        room.execute(
            "INSERT INTO transakcja_umowy_wpisy (id, rodzaj, forma, stan, koniec_powod)
            VALUES ('od-razu', 'przedwstepna', 'pisemna', 'rozwiazana', 'pomyłka')",
            &[],
        ),
        "umowa transakcji powstaje jako projekt",
    );

    prepare_agreement(
        &room,
        &agreement(
            "nie-dojdzie",
            RodzajUmowy::Przedwstepna,
            FormaUmowy::Pisemna,
        ),
    );

    // A writer outside the client can leave out the reason that
    // `odnotuj_ze_umowa_transakcji_nie_doszla` requires. The table refuses the end without one.
    let failure_without_reason = room.execute(
        "UPDATE transakcja_umowy_wpisy
        SET stan = 'nie_doszla',
          koniec_powod = :koniec_powod
        WHERE id = :id",
        &[
            ("koniec_powod", SqlValue::Null),
            ("id", text("nie-dojdzie")),
        ],
    );
    expect::refused(failure_without_reason, "koniec_ma_powod");

    transakcja_umowy::odnotuj_ze_umowa_transakcji_nie_doszla(
        &room,
        &OdnotujZeUmowaTransakcjiNieDoszlaParams {
            koniec_powod: "bank odmówił kredytu".to_string(),
            id: "nie-dojdzie".to_string(),
        },
    )
    .expect("record that the agreement did not come about");
    expect::refused(
        room.execute(
            "UPDATE transakcja_umowy_wpisy SET stan = 'zawarta' WHERE id = 'nie-dojdzie'",
            &[],
        ),
        "umowa transakcji przechodzi z projektu do zawartej albo niedoszłej",
    );

    concluded_agreement(
        &room,
        &agreement(
            "przedwstepna",
            RodzajUmowy::Przedwstepna,
            FormaUmowy::Pisemna,
        ),
    );
    transakcja_umowy::rozwiaz_umowe_transakcji(
        &room,
        &RozwiazUmoweTransakcjiParams {
            koniec_powod: "odstąpienie z zadatkiem (art. 394 KC)".to_string(),
            id: "przedwstepna".to_string(),
        },
    )
    .expect("dissolve the agreement");
    let second_dissolution = RozwiazUmoweTransakcjiParams {
        koniec_powod: "inny powód".to_string(),
        id: "przedwstepna".to_string(),
    };
    expect::refused(
        transakcja_umowy::rozwiaz_umowe_transakcji(&room, &second_dissolution),
        "treść umowy transakcji zmienia się tylko w projekcie",
    );
}

/// Umowa: zawarta ma podpisany dokument, a akt także kancelarię i numer repertorium
#[test]
fn agreement_concluded_with_document() {
    let room = new_room();
    open_sale(&room);
    add_notary_office(&room, &notary_office("kancelaria"));

    prepare_agreement(
        &room,
        &agreement(
            "przedwstepna",
            RodzajUmowy::Przedwstepna,
            FormaUmowy::Pisemna,
        ),
    );
    analyse_draft(&room, "przedwstepna");
    add_received_document(
        &room,
        &received_document("zly-dokument", RodzajDokumentu::Pismo),
    );
    let conclusion_with_letter = OdnotujZawarcieUmowyTransakcjiParams {
        dokument_id: "zly-dokument".to_string(),
        zawarto: "2026-11-10T12:00:00Z".to_string(),
        repertorium: None,
        id: "przedwstepna".to_string(),
    };
    expect::refused(
        transakcja_umowy::odnotuj_zawarcie_umowy_transakcji(&room, &conclusion_with_letter),
        "zawarta umowa wskazuje otrzymany dokument",
    );
    conclude_agreement(
        &room,
        "przedwstepna",
        FormaUmowy::Pisemna,
        "2026-11-10T12:00:00Z",
    );

    prepare_agreement(
        &room,
        &PrzygotujUmoweTransakcjiParams {
            kancelaria_id: Some("kancelaria".to_string()),
            ..agreement("akt", RodzajUmowy::Przenoszaca, FormaUmowy::AktNotarialny)
        },
    );
    analyse_draft(&room, "akt");
    add_received_document(
        &room,
        &received_document("wypis-aktu", RodzajDokumentu::AktNotarialny),
    );
    let deed_without_repertory = OdnotujZawarcieUmowyTransakcjiParams {
        dokument_id: "wypis-aktu".to_string(),
        zawarto: "2026-12-10T11:00:00Z".to_string(),
        repertorium: None,
        id: "akt".to_string(),
    };
    expect::refused(
        transakcja_umowy::odnotuj_zawarcie_umowy_transakcji(&room, &deed_without_repertory),
        "akt_ma_kancelarie_i_repertorium",
    );
    transakcja_umowy::odnotuj_zawarcie_umowy_transakcji(
        &room,
        &OdnotujZawarcieUmowyTransakcjiParams {
            repertorium: Some("Rep. A nr 77/2026".to_string()),
            ..deed_without_repertory
        },
    )
    .expect("conclude the deed");
}

/// Umowa: zawiera się ją tylko z aktualną analizą jej bieżącego projektu
#[test]
fn agreement_needs_current_analysis() {
    let room = new_room();
    open_sale(&room);
    prepare_agreement(
        &room,
        &agreement(
            "przedwstepna",
            RodzajUmowy::Przedwstepna,
            FormaUmowy::Pisemna,
        ),
    );
    add_received_document(
        &room,
        &received_document("podpisana", RodzajDokumentu::Umowa),
    );

    let refusal = "umowa zostaje zawarta tylko z aktualną analizą swojego projektu";
    let conclusion = OdnotujZawarcieUmowyTransakcjiParams {
        dokument_id: "podpisana".to_string(),
        zawarto: "2026-11-10T12:00:00Z".to_string(),
        repertorium: None,
        id: "przedwstepna".to_string(),
    };
    expect::refused(
        transakcja_umowy::odnotuj_zawarcie_umowy_transakcji(&room, &conclusion),
        refusal,
    );

    // The analysed draft is replaced by a new one, so the analysis no longer covers it.
    analyse_draft(&room, "przedwstepna");
    add_received_document(
        &room,
        &received_document("nowy-projekt", RodzajDokumentu::Umowa),
    );
    transakcja_umowy::zmien_projekt_umowy_transakcji(
        &room,
        &ZmienProjektUmowyTransakcjiParams {
            kancelaria_id: None,
            projekt_dokument_id: Some("nowy-projekt".to_string()),
            cena: Some(61_000_000),
            termin_przyrzeczonej: None,
            termin_zaplaty: None,
            termin_wydania: None,
            warunki: None,
            id: "przedwstepna".to_string(),
        },
    )
    .expect("replace the draft");
    expect::refused(
        transakcja_umowy::odnotuj_zawarcie_umowy_transakcji(&room, &conclusion),
        refusal,
    );

    add_analysis(
        &room,
        &draft_analysis("analiza-nowego", "przedwstepna", "nowy-projekt"),
    );
    transakcja_umowy::odnotuj_zawarcie_umowy_transakcji(&room, &conclusion)
        .expect("conclude the agreement");
}

/// Umowa: po wyjściu z projektu treść stoi, a wniosek i wpis do księgi wpisuje się raz
#[test]
fn agreement_frozen_after_draft() {
    let room = new_room();
    open_sale(&room);
    add_notary_office(&room, &notary_office("kancelaria"));
    concluded_deed(&room, "akt", RodzajUmowy::Przenoszaca);

    let frozen = "treść umowy transakcji zmienia się tylko w projekcie";
    let new_draft = ZmienProjektUmowyTransakcjiParams {
        kancelaria_id: Some("kancelaria".to_string()),
        projekt_dokument_id: Some("projekt-akt".to_string()),
        cena: Some(1),
        termin_przyrzeczonej: None,
        termin_zaplaty: None,
        termin_wydania: None,
        warunki: None,
        id: "akt".to_string(),
    };
    expect::refused(
        transakcja_umowy::zmien_projekt_umowy_transakcji(&room, &new_draft),
        frozen,
    );
    let entry = OdnotujWpisDoKsiegiParams {
        wpis_kw: "2027-01-20".to_string(),
        id: "akt".to_string(),
    };
    expect::refused(
        transakcja_umowy::odnotuj_wpis_do_ksiegi(&room, &entry),
        "wpis_po_wniosku",
    );
    let application_of =
        |agreement_id: &str, wniosek_kw_zlozono: &str| OdnotujWniosekDoKsiegiParams {
            wniosek_kw_zlozono: wniosek_kw_zlozono.to_string(),
            id: agreement_id.to_string(),
        };
    transakcja_umowy::odnotuj_wniosek_do_ksiegi(&room, &application_of("akt", "2026-12-10"))
        .expect("record the land register application");
    expect::refused(
        transakcja_umowy::odnotuj_wniosek_do_ksiegi(&room, &application_of("akt", "2026-12-11")),
        frozen,
    );
    transakcja_umowy::odnotuj_wpis_do_ksiegi(&room, &entry)
        .expect("record the land register entry");

    // Only a concluded transfer deed has a land register entry.
    concluded_agreement(
        &room,
        &agreement(
            "przedwstepna",
            RodzajUmowy::Przedwstepna,
            FormaUmowy::Pisemna,
        ),
    );
    expect::refused(
        transakcja_umowy::odnotuj_wniosek_do_ksiegi(
            &room,
            &application_of("przedwstepna", "2026-12-10"),
        ),
        "ksiega_po_zawartej_umowie_przenoszacej",
    );
}

// Right of pre-emption ----------------------------------------------------------------------------

/// Pierwokup: umowa przenosząca dopiero po umowie warunkowej i wyniku każdego pierwokupu
#[test]
fn preemption_needs_conditional_deed() {
    let room = new_room();
    open_sale(&room);
    add_notary_office(&room, &notary_office("kancelaria"));
    add_right_of_preemption(&room);

    // A transfer deed without the conditional one is void (art. 599 § 2 KC).
    prepare_agreement(
        &room,
        &PrzygotujUmoweTransakcjiParams {
            kancelaria_id: Some("kancelaria".to_string()),
            ..agreement(
                "bezwarunkowa",
                RodzajUmowy::Przenoszaca,
                FormaUmowy::AktNotarialny,
            )
        },
    );
    analyse_draft(&room, "bezwarunkowa");
    add_received_document(
        &room,
        &received_document("wypis", RodzajDokumentu::AktNotarialny),
    );
    let refusal = "przy pierwokupie umowa przenosząca wymaga zawartej umowy warunkowej";
    let unconditional_deed = OdnotujZawarcieUmowyTransakcjiParams {
        dokument_id: "wypis".to_string(),
        zawarto: "2026-12-10T11:00:00Z".to_string(),
        repertorium: Some("Rep. A nr 1/2026".to_string()),
        id: "bezwarunkowa".to_string(),
    };
    expect::refused(
        transakcja_umowy::odnotuj_zawarcie_umowy_transakcji(&room, &unconditional_deed),
        refusal,
    );

    // With the conditional deed, but while the municipality has not answered yet.
    concluded_deed(&room, "warunkowa", RodzajUmowy::Warunkowa);
    prepare_agreement(
        &room,
        &PrzygotujUmoweTransakcjiParams {
            kancelaria_id: Some("kancelaria".to_string()),
            oparta_na_id: Some("warunkowa".to_string()),
            ..agreement(
                "przenoszaca",
                RodzajUmowy::Przenoszaca,
                FormaUmowy::AktNotarialny,
            )
        },
    );
    analyse_draft(&room, "przenoszaca");
    add_received_document(
        &room,
        &received_document("wypis-przenoszacej", RodzajDokumentu::AktNotarialny),
    );
    let deed = OdnotujZawarcieUmowyTransakcjiParams {
        dokument_id: "wypis-przenoszacej".to_string(),
        zawarto: "2027-01-15T11:00:00Z".to_string(),
        repertorium: Some("Rep. A nr 2/2027".to_string()),
        id: "przenoszaca".to_string(),
    };
    expect::refused(
        transakcja_umowy::odnotuj_zawarcie_umowy_transakcji(&room, &deed),
        refusal,
    );

    record_preemption_result(&room, WynikPierwokupu::Uplynal);
    transakcja_umowy::odnotuj_zawarcie_umowy_transakcji(&room, &deed)
        .expect("conclude the transfer deed");

    // A right of pre-emption entered after the transfer deed would go round its rule.
    let later_preemption = WpiszPierwokupParams {
        id: "pozniejszy".to_string(),
        uprawniony: "Krajowy Ośrodek Wsparcia Rolnictwa".to_string(),
        podstawa: "art. 3 ust. 4 ukur".to_string(),
        zawiadomiono: None,
    };
    expect::refused(
        transakcja_umowy::wpisz_pierwokup(&room, &later_preemption),
        "pierwokup wpisuje się przed zawarciem umowy przenoszącej",
    );
}

/// Pierwokup: wykonany wyklucza umowę przenoszącą; wynik jest ostateczny i ma źródło
#[test]
fn preemption_exercised() {
    let room = new_room();
    open_sale(&room);
    add_right_of_preemption(&room);

    expect::refused(
        room.execute(
            "INSERT INTO transakcja_umowy_pierwokupy (id, uprawniony, podstawa, wynik)
            VALUES ('od-razu', 'Gmina', 'art. 109 ugn', 'uplynal')",
            &[],
        ),
        "pierwokup powstaje jako czekający na wynik",
    );

    // A writer outside the client can leave out the message that `odnotuj_wynik_pierwokupu`
    // requires. The table refuses the result without one.
    let result_without_message = room.execute(
        "UPDATE transakcja_umowy_pierwokupy
        SET wynik = :wynik,
          zrodlo_wplyw_id = :zrodlo_wplyw_id
        WHERE id = :id",
        &[
            ("wynik", text("wykonano")),
            ("zrodlo_wplyw_id", SqlValue::Null),
            ("id", text("pierwokup-gminy")),
        ],
    );
    expect::refused(
        result_without_message,
        "wynik_po_zawiadomieniu_z_wiadomoscia",
    );

    record_preemption_result(&room, WynikPierwokupu::Wykonano);
    let changed_result = OdnotujWynikPierwokupuParams {
        wynik: WynikPierwokupu::NieWykonano,
        zrodlo_wplyw_id: "wiadomosc-pierwokup-gminy".to_string(),
        id: "pierwokup-gminy".to_string(),
    };
    expect::refused(
        transakcja_umowy::odnotuj_wynik_pierwokupu(&room, &changed_result),
        "wyniku pierwokupu ani raz wpisanego zawiadomienia się nie zmienia",
    );
    expect::refused(
        transakcja_umowy::przygotuj_umowe_transakcji(
            &room,
            &bare_agreement(
                "przenoszaca",
                RodzajUmowy::Przenoszaca,
                FormaUmowy::AktNotarialny,
            ),
        ),
        "uprawniony wykonał pierwokup, więc umowy przenoszącej nie ma",
    );
    transakcja_umowy::przygotuj_umowe_transakcji(
        &room,
        &bare_agreement(
            "warunkowa",
            RodzajUmowy::Warunkowa,
            FormaUmowy::AktNotarialny,
        ),
    )
    .expect("prepare the conditional deed");
}

// Parties -----------------------------------------------------------------------------------------

/// Strona: pełnomocnik do aktu ma pełnomocnictwo w formie aktu notarialnego (art. 99 § 1 KC)
#[test]
fn deed_power_in_notarial_form() {
    let room = new_room();
    open_sale(&room);
    add_parties(&room);
    add_notary_office(&room, &notary_office("kancelaria"));
    add_deed_power(&room, "pisemne", FormaUmocowania::Pisemna);
    prepare_agreement(
        &room,
        &PrzygotujUmoweTransakcjiParams {
            kancelaria_id: Some("kancelaria".to_string()),
            ..agreement("akt", RodzajUmowy::Przenoszaca, FormaUmowy::AktNotarialny)
        },
    );
    prepare_agreement(
        &room,
        &agreement(
            "przedwstepna",
            RodzajUmowy::Przedwstepna,
            FormaUmowy::Pisemna,
        ),
    );

    let refusal = "pełnomocnik strony działa z jej umocowania";
    let party_on_power = |umowa_id: &str, uczestnik_id: &str, strona: Strona, power_id: &str| {
        DopiszStroneUmowyTransakcjiParams {
            umocowanie_id: Some(power_id.to_string()),
            ..party(umowa_id, uczestnik_id, strona)
        }
    };
    expect::refused(
        transakcja_umowy::dopisz_strone_umowy_transakcji(
            &room,
            &party_on_power("akt", "kupujacy", Strona::Kupujacy, "pisemne"),
        ),
        refusal,
    );
    expect::refused(
        transakcja_umowy::dopisz_strone_umowy_transakcji(
            &room,
            &party_on_power(
                "przedwstepna",
                "sprzedajaca",
                Strona::Sprzedajacy,
                "pisemne",
            ),
        ),
        refusal,
    );

    // A written power is enough for a preliminary agreement in writing.
    add_party(
        &room,
        &party_on_power("przedwstepna", "kupujacy", Strona::Kupujacy, "pisemne"),
    );
    sprawa_umocowania::wpisz_umocowanie(
        &room,
        &WpiszUmocowanieParams {
            id: "notarialne".to_string(),
            rodzaj: RodzajUmocowania::Pelnomocnictwo,
            mocodawca_uczestnik_id: "kupujacy".to_string(),
            umocowany_uczestnik_id: Some("brat-kupujacego".to_string()),
            umocowane_biuro: false,
            zakres: ZakresUmocowania::DoCzynnosci,
            forma: Some(FormaUmocowania::AktNotarialny),
            sposob_reprezentacji: None,
            dokument_id: None,
            udzielono: Some("2026-11-03".to_string()),
            wazne_do: None,
        },
    )
    .expect("add the notarial power of attorney");
    add_party(
        &room,
        &party_on_power("akt", "kupujacy", Strona::Kupujacy, "notarialne"),
    );

    let power_made_written = ZmienDaneUmocowaniaParams {
        rodzaj: RodzajUmocowania::Pelnomocnictwo,
        mocodawca_uczestnik_id: "kupujacy".to_string(),
        umocowany_uczestnik_id: Some("brat-kupujacego".to_string()),
        umocowane_biuro: false,
        zakres: ZakresUmocowania::DoCzynnosci,
        forma: Some(FormaUmocowania::Pisemna),
        sposob_reprezentacji: None,
        dokument_id: None,
        wazne_do: None,
        id: "notarialne".to_string(),
    };
    expect::refused(
        sprawa_umocowania::zmien_dane_umocowania(&room, &power_made_written),
        refusal,
    );
}

/// Strona: dopisuje się ją tylko do projektu i się jej nie zmienia
#[test]
fn parties_only_in_draft() {
    let room = new_room();
    open_sale(&room);
    add_parties(&room);
    prepare_agreement(
        &room,
        &agreement(
            "przedwstepna",
            RodzajUmowy::Przedwstepna,
            FormaUmowy::Pisemna,
        ),
    );
    add_party(
        &room,
        &party("przedwstepna", "sprzedajaca", Strona::Sprzedajacy),
    );

    expect::refused(
        room.execute(
            "UPDATE transakcja_umowy_strony SET strona = 'kupujacy'",
            &[],
        ),
        "strony umowy transakcji się nie zmienia",
    );
    analyse_draft(&room, "przedwstepna");
    conclude_agreement(
        &room,
        "przedwstepna",
        FormaUmowy::Pisemna,
        "2026-11-10T12:00:00Z",
    );
    expect::refused(
        transakcja_umowy::dopisz_strone_umowy_transakcji(
            &room,
            &party("przedwstepna", "kupujacy", Strona::Kupujacy),
        ),
        "strony dopisuje się tylko do projektu umowy transakcji",
    );
}

// Payments ----------------------------------------------------------------------------------------

/// The parameters of the deposit „platnosc” of 60 000.00 zł of own money under the preliminary
/// agreement, with no due date and no account.
fn payment_fields() -> WpiszPlatnoscTransakcjiParams {
    WpiszPlatnoscTransakcjiParams {
        termin: None,
        ..payment("platnosc", "przedwstepna", RodzajPlatnosci::Zadatek)
    }
}

/// The parameters of naming the account `rachunek_id` for the payment `payment_id`.
fn account_for_payment(payment_id: &str, rachunek_id: &str) -> WskazRachunekPlatnosciParams {
    WskazRachunekPlatnosciParams {
        rachunek_id: rachunek_id.to_string(),
        id: payment_id.to_string(),
    }
}

/// Płatność: idzie na rachunek dopuszczony decyzją i niezastąpiony (R4)
#[test]
fn payment_to_admitted_account() {
    let room = new_room();
    open_sale(&room);
    add_parties(&room);
    prepare_agreement(
        &room,
        &agreement(
            "przedwstepna",
            RodzajUmowy::Przedwstepna,
            FormaUmowy::Pisemna,
        ),
    );
    add_account(
        &room,
        &WpiszRachunekParams {
            cel: CelRachunku::Zadatek,
            ..account("niedopuszczony", "sprzedajaca")
        },
    );

    let refusal = "płatność wskazuje rachunek dopuszczony decyzją i niezastąpiony (R4)";
    let payment_to_unadmitted = WpiszPlatnoscTransakcjiParams {
        rachunek_id: Some("niedopuszczony".to_string()),
        ..payment_fields()
    };
    expect::refused(
        transakcja_umowy::wpisz_platnosc_transakcji(&room, &payment_to_unadmitted),
        refusal,
    );
    add_payment(
        &room,
        &payment("zadatek", "przedwstepna", RodzajPlatnosci::Zadatek),
    );
    expect::refused(
        transakcja_umowy::wskaz_rachunek_platnosci(
            &room,
            &account_for_payment("zadatek", "niedopuszczony"),
        ),
        refusal,
    );

    admitted_account_for(&room, "dopuszczony", "sprzedajaca", CelRachunku::Zadatek);
    transakcja_umowy::wskaz_rachunek_platnosci(
        &room,
        &account_for_payment("zadatek", "dopuszczony"),
    )
    .expect("name the admitted account");

    // The account is replaced before the money comes: the payment is checked again.
    add_account(
        &room,
        &WpiszRachunekParams {
            cel: CelRachunku::Zadatek,
            numer: "PL27114020040000300201355387".to_string(),
            ..account("nowy", "sprzedajaca")
        },
    );
    sprawa_rachunki::zastap_rachunek(
        &room,
        &ZastapRachunekParams {
            zastapiony_przez_id: "nowy".to_string(),
            zastapil: "tozsamosc-agenta".to_string(),
            zastapiono: "2026-11-11T09:00:00Z".to_string(),
            id: "dopuszczony".to_string(),
        },
    )
    .expect("replace the account");
    add_received_document(
        &room,
        &received_document("potwierdzenie", RodzajDokumentu::PotwierdzenieBankowe),
    );
    let payment_to_replaced = OdnotujWplatePlatnosciParams {
        wplacono: "2026-11-12T10:00:00Z".to_string(),
        potwierdzenie_dokument_id: "potwierdzenie".to_string(),
        id: "zadatek".to_string(),
    };
    expect::refused(
        transakcja_umowy::odnotuj_wplate_platnosci(&room, &payment_to_replaced),
        refusal,
    );
}

/// Płatność: kredyt ma bank i warunki wypłaty; wpłata ma rachunek i potwierdzenie z banku
#[test]
fn payment_loan_and_proof() {
    let room = new_room();
    open_sale(&room);
    add_parties(&room);
    prepare_agreement(
        &room,
        &agreement(
            "przedwstepna",
            RodzajUmowy::Przedwstepna,
            FormaUmowy::Pisemna,
        ),
    );

    let loan_without_terms = WpiszPlatnoscTransakcjiParams {
        rodzaj: RodzajPlatnosci::Cena,
        zrodlo: ZrodloPlatnosci::Kredyt,
        bank: Some("Bank Przykładowy S.A.".to_string()),
        ..payment_fields()
    };
    expect::refused(
        transakcja_umowy::wpisz_platnosc_transakcji(&room, &loan_without_terms),
        "kredyt_z_bankiem_i_warunkami",
    );
    let own_money_with_bank = WpiszPlatnoscTransakcjiParams {
        bank: Some("Bank Przykładowy S.A.".to_string()),
        ..payment_fields()
    };
    expect::refused(
        transakcja_umowy::wpisz_platnosc_transakcji(&room, &own_money_with_bank),
        "kredyt_z_bankiem_i_warunkami",
    );
    add_payment(
        &room,
        &payment("zadatek", "przedwstepna", RodzajPlatnosci::Zadatek),
    );
    add_received_document(
        &room,
        &received_document(
            "potwierdzenie-bez-rachunku",
            RodzajDokumentu::PotwierdzenieBankowe,
        ),
    );
    let payment_without_account = OdnotujWplatePlatnosciParams {
        wplacono: "2026-11-12T10:00:00Z".to_string(),
        potwierdzenie_dokument_id: "potwierdzenie-bez-rachunku".to_string(),
        id: "zadatek".to_string(),
    };
    expect::refused(
        transakcja_umowy::odnotuj_wplate_platnosci(&room, &payment_without_account),
        "wplata_z_rachunkiem_i_potwierdzeniem",
    );

    admitted_account_for(&room, "rachunek", "sprzedajaca", CelRachunku::Zadatek);
    transakcja_umowy::wskaz_rachunek_platnosci(&room, &account_for_payment("zadatek", "rachunek"))
        .expect("name the admitted account");
    add_received_document(
        &room,
        &received_document("zrzut-ekranu", RodzajDokumentu::Pismo),
    );
    let payment_with_screenshot = OdnotujWplatePlatnosciParams {
        potwierdzenie_dokument_id: "zrzut-ekranu".to_string(),
        ..payment_without_account
    };
    expect::refused(
        transakcja_umowy::odnotuj_wplate_platnosci(&room, &payment_with_screenshot),
        "wpłata wskazuje otrzymane potwierdzenie bankowe",
    );
    record_payment(&room, "zadatek");
}

/// Płatność: dopisuje się ją do projektu; po zawarciu warunki stoją, a zapłaconej nic
#[test]
fn payment_terms_fixed() {
    let room = new_room();
    open_sale(&room);
    add_parties(&room);
    prepare_agreement(
        &room,
        &agreement(
            "przedwstepna",
            RodzajUmowy::Przedwstepna,
            FormaUmowy::Pisemna,
        ),
    );
    add_payment(
        &room,
        &payment("zadatek", "przedwstepna", RodzajPlatnosci::Zadatek),
    );
    let change_of_amount = |kwota: i64| ZmienPlatnoscTransakcjiParams {
        charakter: None,
        kwota,
        termin: None,
        zrodlo: ZrodloPlatnosci::Wlasne,
        bank: None,
        warunki_wyplaty: None,
        id: "zadatek".to_string(),
    };
    transakcja_umowy::zmien_platnosc_transakcji(&room, &change_of_amount(5_000_000))
        .expect("change the payment in the draft");

    expect::refused(
        room.execute(
            "INSERT INTO transakcja_umowy_platnosci (id, umowa_id, rodzaj, kwota, zrodlo, rachunek_id,
              wplacono, potwierdzenie_dokument_id)
            VALUES ('od-razu', 'przedwstepna', 'zaliczka', 100, 'wlasne', NULL, '2026-11-12', NULL)",
            &[],
        ),
        "płatność dopisuje się do projektu umowy transakcji, jeszcze niezapłaconą",
    );

    analyse_draft(&room, "przedwstepna");
    conclude_agreement(
        &room,
        "przedwstepna",
        FormaUmowy::Pisemna,
        "2026-11-10T12:00:00Z",
    );
    expect::refused(
        transakcja_umowy::zmien_platnosc_transakcji(&room, &change_of_amount(4_000_000)),
        "warunki płatności zmieniają się tylko w projekcie umowy",
    );
    let second_payment = WpiszPlatnoscTransakcjiParams {
        id: "druga".to_string(),
        ..payment_fields()
    };
    expect::refused(
        transakcja_umowy::wpisz_platnosc_transakcji(&room, &second_payment),
        "płatność dopisuje się do projektu umowy transakcji",
    );

    // The account may still be named after signing, and then the money comes.
    admitted_account_for(&room, "rachunek", "sprzedajaca", CelRachunku::Zadatek);
    transakcja_umowy::wskaz_rachunek_platnosci(&room, &account_for_payment("zadatek", "rachunek"))
        .expect("name the account after signing");
    record_payment(&room, "zadatek");
    admitted_account_for(&room, "drugi", "sprzedajaca", CelRachunku::Zadatek);
    expect::refused(
        transakcja_umowy::wskaz_rachunek_platnosci(&room, &account_for_payment("zadatek", "drugi")),
        "zapłaconej płatności nie zmienia się wcale",
    );
}

// Analyses ----------------------------------------------------------------------------------------

/// Analiza: nowa odbiera aktualność poprzedniej tego samego przedmiotu i umowy
#[test]
fn analysis_supersedes_previous() {
    let room = new_room();
    open_sale(&room);
    prepare_agreement(
        &room,
        &agreement(
            "przedwstepna",
            RodzajUmowy::Przedwstepna,
            FormaUmowy::Pisemna,
        ),
    );
    prepare_agreement(
        &room,
        &agreement(
            "rezerwacyjna",
            RodzajUmowy::Rezerwacyjna,
            FormaUmowy::Pisemna,
        ),
    );

    add_analysis(
        &room,
        &draft_analysis("pierwsza", "przedwstepna", "projekt-przedwstepna"),
    );
    add_analysis(
        &room,
        &draft_analysis("innej-umowy", "rezerwacyjna", "projekt-rezerwacyjna"),
    );
    add_analysis(
        &room,
        &draft_analysis("druga", "przedwstepna", "projekt-przedwstepna"),
    );

    let current_analyses =
        transakcja_analizy::analizy_aktualne_transakcji(&room).expect("read the current analyses");
    let mut current: Vec<&str> = current_analyses.iter().map(|row| row.id.as_str()).collect();
    current.sort();
    assert_eq!(current, ["druga", "innej-umowy"]);

    expect::refused(
        room.execute(
            "INSERT INTO transakcja_analizy_wpisy (id, przedmiot, sporzadzono, wnioski, sciezka, skrot,
              aktualna)
            VALUES ('stara', 'ksiega', '2026-11-01', 'wnioski', 'a.typ', 'sha', 0)",
            &[],
        ),
        "analiza powstaje jako aktualna",
    );
    let analysis_without_agreement = WpiszAnalizeTransakcjiParams {
        id: "bez-umowy".to_string(),
        przedmiot: PrzedmiotAnalizy::ProjektUmowy,
        umowa_id: None,
        dokument_id: Some("projekt-przedwstepna".to_string()),
        poprzedni_odpis_dokument_id: None,
        sporzadzono: "2026-11-06".to_string(),
        wnioski: "wnioski".to_string(),
        sciezka: "a.typ".to_string(),
        skrot: "sha".to_string(),
    };
    expect::refused(
        transakcja_analizy::wpisz_analize_transakcji(&room, &analysis_without_agreement),
        "umowa_przy_analizie_umowy",
    );
}

/// Analiza: wpisu się nie zmienia, a ryzyko dopisuje się do aktualnej
#[test]
fn analysis_immutable_and_risks() {
    let room = new_room();
    open_sale(&room);
    prepare_agreement(
        &room,
        &agreement(
            "przedwstepna",
            RodzajUmowy::Przedwstepna,
            FormaUmowy::Pisemna,
        ),
    );
    add_analysis(
        &room,
        &draft_analysis("pierwsza", "przedwstepna", "projekt-przedwstepna"),
    );

    expect::refused(
        room.execute(
            "UPDATE transakcja_analizy_wpisy SET wnioski = 'inne' WHERE id = 'pierwsza'",
            &[],
        ),
        "analizy się nie zmienia",
    );

    let risk_of = |opis: &str| DopiszRyzykoTransakcjiParams {
        analiza_id: "pierwsza".to_string(),
        opis: opis.to_string(),
        skutek: None,
    };
    transakcja_analizy::dopisz_ryzyko_transakcji(&room, &risk_of("Zadatek bez terminu"))
        .expect("add a risk to the current analysis");
    add_analysis(
        &room,
        &draft_analysis("druga", "przedwstepna", "projekt-przedwstepna"),
    );
    expect::refused(
        transakcja_analizy::dopisz_ryzyko_transakcji(&room, &risk_of("Spóźnione ryzyko")),
        "ryzyko dopisuje się do aktualnej analizy",
    );
    expect::refused(
        room.execute(
            "UPDATE transakcja_analizy_wpisy SET aktualna = 1 WHERE id = 'pierwsza'",
            &[],
        ),
        "analizy się nie zmienia",
    );
    expect::refused(
        room.execute("UPDATE transakcja_analizy_ryzyka SET skutek = 'inny'", &[]),
        "ryzyka się nie zmienia",
    );
}

// R9: no deletion, no replacement -----------------------------------------------------------------

/// The tables of the sale kinds, whose names start with `transakcja_`.
fn transakcja_tables(room: &MemoryRoom) -> Vec<String> {
    let rows = room
        .execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'transakcja\\_%' ESCAPE '\\'",
            &[],
        )
        .expect("read the tables");

    rows.iter()
        .map(|row| row.get::<String>("name").expect("a table has a name"))
        .collect()
}

/// R9: wierszy typu transakcji się nie usuwa ani nie zastępuje przez REPLACE i upsert
#[test]
fn no_deletion_or_replacement() {
    let room = new_room();
    full_sale_with_preemption(&room);

    let tables = transakcja_tables(&room);
    assert_eq!(tables.len(), 7, "{tables:?}");
    let refusal = "wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE";

    for table in &tables {
        let rows = integer_value(&room, &format!("SELECT count(*) FROM {table}"));
        assert!(rows > 0, "{table} has no row");

        let delete = format!("DELETE FROM {table} WHERE rowid = (SELECT min(rowid) FROM {table})");
        expect::refused(
            room.execute(&delete, &[]),
            "wierszy teczki się nie usuwa (R9)",
        );

        let replace = format!("INSERT OR REPLACE {}", first_row_copy(&room, table));
        expect::refused(room.execute(&replace, &[]), refusal);

        let upsert = format!(
            "INSERT {} ON CONFLICT (id) DO UPDATE SET id = excluded.id",
            first_row_copy(&room, table)
        );
        expect::refused(room.execute(&upsert, &[]), refusal);

        if !unique_keys_of(&room, table).is_empty() {
            let replace_by_unique_key = format!(
                "INSERT OR REPLACE {}",
                first_row_copy_without_id(&room, table)
            );
            expect::refused(room.execute(&replace_by_unique_key, &[]), refusal);
        }
    }
}
