//! Scenarios of the rental case room (spec/cases/najem on the core).
//!
//! Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
//! allows and a write it refuses. The first two scenarios follow a whole rental as the profile
//! describes it: an ordinary lease the office manages from the contract to the return of the deposit,
//! and an occasional lease the office only brokers, which ends after the flat is handed over.

use xboards_agencja_nieruchomosci::cases::najem::{
    self, najem_czynsz, najem_kaucja, najem_umowy, najem_usterki,
};
use xboards_agencja_nieruchomosci::cases::sprawa::{
    sprawa_decyzje, sprawa_dokumenty, sprawa_korespondencja, sprawa_metryka, sprawa_nieruchomosci,
    sprawa_powiazania, sprawa_protokoly, sprawa_rachunki, sprawa_sprawdzenia, sprawa_uczestnicy,
    sprawa_umowy, sprawa_wyplaty, sprawa_zgody,
};
use xboards_agencja_nieruchomosci::xmeta_room::{FromSqlValue, MemoryRoom, SqlValue};

use najem_czynsz::{
    NaleznosciTytul as TytulNaleznosci, OdczytyLicznik as Licznik,
    OdnotujOddanieDanychDlaGminyParams, OdnotujWynikZaleglosciParams,
    OdnotujWyslanieDanychDlaGminyParams, PoprawDaneDlaGminyParams, PoprawNaleznoscParams,
    PrzygotujDaneDlaGminyParams, RozliczWplateParams, UstalDodatkowyTerminZaleglosciParams,
    WpiszNaleznoscParams, WpiszOdczytLicznikaParams, WpiszWplateCzynszuParams,
    WpiszZalegloscParams, WpiszZmianeCzynszuParams, WplatyNaRachunek as NaRachunek,
    WplatyPotwierdzenie as PotwierdzenieWplaty, WskazUprzedzenieOZaleglosciParams,
    Wynik as WynikZaleglosci,
};
use najem_kaucja::{
    DopiszPotracenieZKaucjiParams, OdnotujUzupelnienieKaucjiParams,
    OdnotujWyslanieRozliczeniaKaucjiParams, PobraniaLezy as KaucjaLezy,
    PrzygotujRozliczenieKaucjiParams, RozliczeniaStan as StanRozliczeniaKaucji,
    WpiszPobranieKaucjiParams, WpiszWykorzystanieKaucjiParams, WskazZgodeNaPotracenieParams,
    ZatwierdzRozliczenieKaucjiParams,
};
use najem_umowy::{
    DopiszOplateNajmuParams, KonceSposob as SposobKonca, NajmuForma as FormaNajmu,
    NajmuRodzaj as RodzajNajmu, OdnotujDecyzjePoSmierciNajemcyParams,
    OdnotujDoreczenieKoncaNajmuParams, OdnotujDoreczenieZadaniaOproznieniaParams,
    OdnotujOproznienieLokaluParams, OdnotujPodpisanieAneksuNajmuParams,
    OdnotujPrzekazanieSwiadectwaParams, OdnotujUtrateLokaluZastepczegoParams,
    OdnotujWyprowadzkeZamieszkujacegoParams, OdnotujZawarcieUmowyNajmuParams,
    OdnotujZgloszenieNajmuDoUrzeduParams, OplatyRodzaj as RodzajOplaty,
    OplatyUmowaZDostawcaMa as UmowaZDostawcaMa, PrzekazUmoweNajmuDoPodpisuParams,
    PrzygotujAneksNajmuParams, PrzygotujUmoweNajmuParams, WpiszKoniecNajmuParams,
    WpiszLokalZastepczyParams, WpiszWarunkiZarzadzaniaParams, WpiszZamieszkujacegoParams,
    WskazOswiadczenieNajemcyParams, WskazZadanieOproznieniaParams, WskazZgodeNaAneksNajmuParams,
    WskazZgodeNaLokalZastepczyParams, ZakonczUmoweNajmuParams, ZarzadzaniePrzeplyw as Przeplyw,
    ZmienProjektUmowyNajmuParams,
};
use najem_usterki::{
    OdnotujUsuniecieUsterkiParams, UstalKogoObciazaUsterkaParams, WpiszPrzegladParams,
    ZglosUsterkeParams, ZgloszeniaObciaza as ObciazaUsterka, ZlecNapraweUsterkiParams,
};
use sprawa_decyzje::{PoprosODecyzjeParams, WpisyRodzaj as RodzajDecyzji};
use sprawa_dokumenty::WpisyRodzaj as RodzajDokumentu;
use sprawa_korespondencja::{
    OdnotujDoreczenieParams, OdnotujWyslanieParams, PrzekazWysylkeDoZgodyParams,
    PrzygotujWysylkeParams, WpiszWplywParams,
};
use sprawa_metryka::{PostawWatpliwoscParams, ZalozMetrykeParams, ZmienEtapSprawyParams};
use sprawa_nieruchomosci::{WpisyRodzaj as RodzajNieruchomosci, WpiszNieruchomoscParams};
use sprawa_powiazania::{WpisyRodzaj as RodzajPowiazania, WpiszPowiazanieParams};
use sprawa_protokoly::{
    DopiszPozycjeProtokoluParams, PodpiszProtokolParams, PozycjeRodzaj as RodzajPozycjiProtokolu,
    PrzygotujProtokolParams, WpisyRodzaj as RodzajProtokolu,
};
use sprawa_rachunki::{WpisyCel as CelRachunku, WpiszRachunekParams};
use sprawa_sprawdzenia::{
    WpisyRodzaj as RodzajSprawdzenia, WpisyWynik as WynikSprawdzenia, WpiszSprawdzenieParams,
};
use sprawa_uczestnicy::{NadajRoleUczestnikowiParams, WpiszUczestnikaParams};
use sprawa_umowy::{
    PrzygotujUmoweParams, WpisyRodzaj as RodzajUmowy, WpisySposobZawarcia as SposobZawarcia,
    WpiszZdarzenieUmowyParams, ZakonczUmoweParams, ZdarzeniaRodzaj as RodzajZdarzeniaUmowy,
};
use sprawa_wyplaty::{
    AnulujWyplateParams, OdnotujWykonanieWyplatyParams, PrzygotujWyplateParams,
    WpisyStan as StanWyplaty, WskazSprawdzenieWyplatyParams, WskazZgodeNaWyplateParams,
};
use sprawa_zgody::WpiszZgodeParams;

use crate::expect;
use crate::sprawa::{
    account, add_account, add_company, add_flat, add_inflow, add_person, add_received_document,
    approve, approved_decision, ask_for_decision, check, check_of, concluded_contract,
    confirm_account, consent, contract, decision_request, decision_request_about, flat, inflow,
    letter, open_case, payout, person, prepare_letter, prepare_payout, received_document, send,
    sending,
};
use crate::sprawa_porzadek::{first_row_copy, first_row_copy_without_id, unique_keys_of};

fn new_room() -> MemoryRoom {
    najem::open_in_memory().expect("open the room of a rental")
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

// Building blocks of a rental ----------------------------------------------------------------------

/// The management contract of `crate::sprawa::concluded_contract` is concluded at this moment.
const MANAGEMENT_CONCLUDED: &str = "2026-11-04T12:00:00Z";

fn open_rental(room: &MemoryRoom) {
    open_case(room, "najem");
}

/// The parameters of the rental agent's move of the case to `etap`, with no decision.
fn stage_change(etap: &str) -> ZmienEtapSprawyParams {
    ZmienEtapSprawyParams {
        etap: etap.to_string(),
        od: "2026-11-05T09:00:00Z".to_string(),
        zmienil: "tozsamosc-opiekuna".to_string(),
        decyzja_id: None,
    }
}

/// Moves the case through the `etapy`, one after another.
fn move_to(room: &MemoryRoom, etapy: &[&str]) {
    for etap in etapy {
        sprawa_metryka::zmien_etap_sprawy(room, &stage_change(etap))
            .expect("change the stage of the case");
    }
}

/// Moves the case to `etap` on the strength of the decision `decision_id`.
fn move_by_decision(room: &MemoryRoom, etap: &str, decision_id: &str) {
    let params = ZmienEtapSprawyParams {
        decyzja_id: Some(decision_id.to_string()),
        ..stage_change(etap)
    };

    sprawa_metryka::zmien_etap_sprawy(room, &params).expect("change the stage by the decision");
}

/// Expects the database to refuse the agent's move of the case to `etap` with `rule`.
#[track_caller]
fn refused_stage(room: &MemoryRoom, rule: &str, etap: &str) {
    let params = ZmienEtapSprawyParams {
        etap: etap.to_string(),
        od: "2026-11-05".to_string(),
        zmienil: "agent".to_string(),
        decyzja_id: None,
    };

    expect::refused(sprawa_metryka::zmien_etap_sprawy(room, &params), rule);
}

/// The default parameters of a participant (see [`person`]) with other names.
fn person_named(person_id: &str, imiona: &str, nazwisko: &str) -> WpiszUczestnikaParams {
    WpiszUczestnikaParams {
        imiona: Some(imiona.to_string()),
        nazwisko: Some(nazwisko.to_string()),
        ..person(person_id)
    }
}

fn give_role(room: &MemoryRoom, person_id: &str, rola: &str) {
    let params = NadajRoleUczestnikowiParams {
        uczestnik_id: person_id.to_string(),
        rola: rola.to_string(),
        dotyczy_uczestnik_id: None,
    };

    sprawa_uczestnicy::nadaj_role_uczestnikowi(room, &params).expect("give the participant a role");
}

/// The owner, a consumer and a client of the office, and the tenant found for her.
fn add_rental_parties(room: &MemoryRoom) {
    add_person(
        room,
        &WpiszUczestnikaParams {
            konsument: true,
            klient_biura: true,
            ..person("wlascicielka")
        },
    );
    add_person(room, &person_named("najemca", "Jan", "Nowak"));

    give_role(room, "wlascicielka", "wynajmujacy");
    give_role(room, "najemca", "najemca");
}

/// Writes the inflow `inflow_id` (see [`inflow`]) sent by the participant `person_id`.
fn add_inflow_from(room: &MemoryRoom, inflow_id: &str, person_id: &str) {
    let params = WpiszWplywParams {
        nadawca_uczestnik_id: Some(person_id.to_string()),
        ..inflow(inflow_id)
    };

    add_inflow(room, &params);
}

/// A decision of the kind `rodzaj` about the row `dotyczy_id` of `dotyczy_tabela`, asked for and
/// approved by the broker on 2026-11-03.
fn approved_decision_about(
    room: &MemoryRoom,
    decision_id: &str,
    rodzaj: RodzajDecyzji,
    dotyczy_tabela: &str,
    dotyczy_id: &str,
) {
    let request = decision_request_about(decision_id, rodzaj, dotyczy_tabela, dotyczy_id);

    approved_decision(room, &request, "2026-11-03T12:00:00Z");
}

/// The default parameters of the first terms of the management contract „zarzadzanie”, valid from
/// `od`: repairs up to 500 zł without asking, money through the office.
fn management_terms(od: &str) -> WpiszWarunkiZarzadzaniaParams {
    WpiszWarunkiZarzadzaniaParams {
        umowa_id: "zarzadzanie".to_string(),
        kwota_napraw: 50_000,
        przeplyw: Przeplyw::PrzezBiuro,
        aneks_zdarzenie_id: None,
        od: od.to_string(),
    }
}

/// The management contract with the owner and its first terms with the money flow `przeplyw`.
fn management(room: &MemoryRoom, przeplyw: Przeplyw) {
    concluded_contract(
        room,
        "zarzadzanie",
        "wlascicielka",
        SposobZawarcia::WLokaluBiura,
    );

    let terms = WpiszWarunkiZarzadzaniaParams {
        przeplyw,
        ..management_terms(MANAGEMENT_CONCLUDED)
    };
    najem_umowy::wpisz_warunki_zarzadzania(room, &terms).expect("write the management terms");
}

/// Writes the received annex `aneks-zarzadzania` of the management contract and its event
/// „aneks” at `chwila`.
fn add_management_annex(room: &MemoryRoom, chwila: &str) {
    add_received_document(
        room,
        &received_document("aneks-zarzadzania", RodzajDokumentu::UmowaZBiurem),
    );

    let annex = WpiszZdarzenieUmowyParams {
        id: "aneks".to_string(),
        umowa_id: "zarzadzanie".to_string(),
        rodzaj: RodzajZdarzeniaUmowy::Aneks,
        chwila: chwila.to_string(),
        opis: None,
        dokument_id: Some("aneks-zarzadzania".to_string()),
        wysylka_id: None,
        wplyw_id: None,
        zgoda_id: None,
    };
    sprawa_umowy::wpisz_zdarzenie_umowy(room, &annex).expect("write the annex of the contract");
}

/// The default parameters of a lease of the flat „lokal” for two years from 2026-12-01: 3000 zł
/// rent, a deposit of two rents, in writing, from the draft `projekt-<id>`.
fn lease(lease_id: &str, rodzaj: RodzajNajmu) -> PrzygotujUmoweNajmuParams {
    PrzygotujUmoweNajmuParams {
        id: lease_id.to_string(),
        rodzaj,
        nieruchomosc_id: "lokal".to_string(),
        poczatek: "2026-12-01".to_string(),
        koniec: Some("2028-11-30".to_string()),
        forma: FormaNajmu::Pisemna,
        projekt_dokument_id: Some(format!("projekt-{lease_id}")),
        czynsz: 300_000,
        termin_platnosci: Some("z góry do 10. dnia miesiąca".to_string()),
        kaucja: 600_000,
        zwierzeta: None,
        podnajem: None,
        zmiany_w_lokalu: None,
        naprawy_podzial: None,
        wypowiedzenie_przypadki: None,
    }
}

/// The default parameters of a lease (see [`lease`]) without a deposit.
fn lease_without_deposit(lease_id: &str, rodzaj: RodzajNajmu) -> PrzygotujUmoweNajmuParams {
    PrzygotujUmoweNajmuParams {
        kaucja: 0,
        ..lease(lease_id, rodzaj)
    }
}

/// The default parameters of an ordinary lease „najem” (see [`lease`]) with no draft document
/// and no payment term.
fn lease_fields() -> PrzygotujUmoweNajmuParams {
    PrzygotujUmoweNajmuParams {
        projekt_dokument_id: None,
        termin_platnosci: None,
        ..lease("najem", RodzajNajmu::Zwykly)
    }
}

/// Writes the received draft `projekt-<id>`, then the draft lease of `params` (see [`lease`]).
fn prepare_lease(room: &MemoryRoom, params: &PrzygotujUmoweNajmuParams) {
    let draft_id = format!("projekt-{}", params.id);
    add_received_document(room, &received_document(&draft_id, RodzajDokumentu::Umowa));

    najem_umowy::przygotuj_umowe_najmu(room, params).expect("prepare the lease");
}

/// The tenant's declaration as a notarial deed, an annex of an occasional lease.
fn add_tenant_declaration(room: &MemoryRoom, lease_id: &str) {
    let declaration_id = format!("oswiadczenie-{lease_id}");
    add_received_document(
        room,
        &received_document(&declaration_id, RodzajDokumentu::AktNotarialny),
    );

    let params = WskazOswiadczenieNajemcyParams {
        oswiadczenie_dokument_id: declaration_id,
        id: lease_id.to_string(),
    };
    najem_umowy::wskaz_oswiadczenie_najemcy(room, &params)
        .expect("point the lease to the tenant's declaration");
}

/// Writes the substitute flat at ul. Lipowa 4 m. 2 with its owner's consent; returns its id.
fn add_substitute_flat(room: &MemoryRoom, lease_id: &str) -> i64 {
    let substitute = WpiszLokalZastepczyParams {
        najem_id: lease_id.to_string(),
        adres: "ul. Lipowa 4 m. 2, Wrocław".to_string(),
        wskazano: "2026-11-20".to_string(),
    };
    najem_umowy::wpisz_lokal_zastepczy(room, &substitute).expect("write the substitute flat");
    let substitute_id = integer_value(room, "SELECT max(id) FROM najem_umowy_lokale_zastepcze");

    let consent_id = format!("zgoda-lokal-{substitute_id}");
    add_received_document(
        room,
        &received_document(&consent_id, RodzajDokumentu::Oswiadczenie),
    );
    let consent_of_owner = WskazZgodeNaLokalZastepczyParams {
        zgoda_dokument_id: consent_id,
        id: substitute_id,
    };
    najem_umowy::wskaz_zgode_na_lokal_zastepczy(room, &consent_of_owner)
        .expect("point the substitute flat to its owner's consent");

    substitute_id
}

/// The parameters of handing the lease over for signing on the decision `do-podpisu-<id>`.
fn lease_for_signing(lease_id: &str) -> PrzekazUmoweNajmuDoPodpisuParams {
    PrzekazUmoweNajmuDoPodpisuParams {
        do_podpisu_decyzja_id: format!("do-podpisu-{lease_id}"),
        id: lease_id.to_string(),
    }
}

/// A human's consent to the lease `do-podpisu-<id>`, then the lease handed over for signing.
fn lease_to_signing(room: &MemoryRoom, lease_id: &str) {
    approved_decision_about(
        room,
        &format!("do-podpisu-{lease_id}"),
        RodzajDecyzji::Inna,
        "najem_umowy_najmu",
        lease_id,
    );

    najem_umowy::przekaz_umowe_najmu_do_podpisu(room, &lease_for_signing(lease_id))
        .expect("hand the lease over for signing");
}

/// Writes the received signed lease `podpisana-<id>` and records the lease concluded with it.
fn conclude_lease(room: &MemoryRoom, lease_id: &str) {
    let signed_id = format!("podpisana-{lease_id}");
    add_received_document(room, &received_document(&signed_id, RodzajDokumentu::Umowa));

    let conclusion = OdnotujZawarcieUmowyNajmuParams {
        dokument_id: signed_id,
        zawarto: "2026-11-25T12:00:00Z".to_string(),
        id: lease_id.to_string(),
    };
    najem_umowy::odnotuj_zawarcie_umowy_najmu(room, &conclusion).expect("conclude the lease");
}

/// A lease of `params` (see [`lease`]) signed by the parties, with the annexes its kind requires.
fn concluded_lease(room: &MemoryRoom, params: &PrzygotujUmoweNajmuParams) {
    prepare_lease(room, params);

    if params.rodzaj != RodzajNajmu::Zwykly {
        add_tenant_declaration(room, &params.id);
    }
    if params.rodzaj == RodzajNajmu::Okazjonalny {
        add_substitute_flat(room, &params.id);
    }

    lease_to_signing(room, &params.id);
    conclude_lease(room, &params.id);
}

/// Writes the protocol `protokol-<rodzaj>` of the flat at `chwila` with the electricity meter, and
/// signs it with the received document `podpisany-protokol-<rodzaj>`.
fn signed_protocol(room: &MemoryRoom, rodzaj: RodzajProtokolu, chwila: &str) {
    let protocol_id = format!("protokol-{}", rodzaj.as_str());
    let signed_id = format!("podpisany-protokol-{}", rodzaj.as_str());

    let protocol = PrzygotujProtokolParams {
        id: protocol_id.clone(),
        rodzaj,
        nieruchomosc_id: "lokal".to_string(),
        chwila: chwila.to_string(),
    };
    sprawa_protokoly::przygotuj_protokol(room, &protocol).expect("prepare the protocol");

    let meter = DopiszPozycjeProtokoluParams {
        protokol_id: protocol_id.clone(),
        rodzaj: RodzajPozycjiProtokolu::Licznik,
        nazwa: "licznik energii elektrycznej".to_string(),
        numer: Some("E-778812".to_string()),
        opis_stanu: Some("12 345,6 kWh".to_string()),
        zdjecie_sciezka: None,
        zdjecie_skrot: None,
    };
    sprawa_protokoly::dopisz_pozycje_protokolu(room, &meter).expect("add the meter");

    add_received_document(
        room,
        &received_document(&signed_id, RodzajDokumentu::Protokol),
    );
    let signature = PodpiszProtokolParams {
        dokument_id: signed_id,
        id: protocol_id,
    };
    sprawa_protokoly::podpisz_protokol(room, &signature).expect("sign the protocol");
}

/// An account (see [`account`]) for `cel`, held by `posiadacz`, confirmed by another channel,
/// checked by Kontroler and admitted (R4). The Python default holder is „Anna Kowalska”.
fn admitted_account_for(
    room: &MemoryRoom,
    account_id: &str,
    person_id: &str,
    cel: CelRachunku,
    posiadacz: &str,
) {
    add_account(
        room,
        &WpiszRachunekParams {
            cel,
            posiadacz: posiadacz.to_string(),
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

    approved_decision_about(
        room,
        &format!("dopuszczenie-{account_id}"),
        RodzajDecyzji::DopuszczenieRachunku,
        "sprawa_rachunki_wpisy",
        account_id,
    );
}

/// The default parameters of a payout (see [`payout`]) of `kwota` from the rent payment
/// „wplata-grudzien” to the owner, prepared on 2027-01-12 (T13).
fn rent_transfer(
    payout_id: &str,
    kwota: i64,
    person_id: &str,
    account_id: &str,
) -> PrzygotujWyplateParams {
    PrzygotujWyplateParams {
        podstawa_tabela: "najem_czynsz_wplaty".to_string(),
        podstawa_id: "wplata-grudzien".to_string(),
        kwota,
        utworzono: "2027-01-12T09:00:00Z".to_string(),
        ..payout(payout_id, person_id, account_id)
    }
}

/// The default parameters of a payout (see [`payout`]) of `kwota` by the deposit settlement
/// „rozliczenie”, prepared on 2027-01-12.
fn deposit_return(
    payout_id: &str,
    kwota: i64,
    person_id: &str,
    account_id: &str,
) -> PrzygotujWyplateParams {
    PrzygotujWyplateParams {
        podstawa_tabela: "najem_kaucja_rozliczenia".to_string(),
        podstawa_id: "rozliczenie".to_string(),
        kwota,
        utworzono: "2027-01-12T09:00:00Z".to_string(),
        ..payout(payout_id, person_id, account_id)
    }
}

/// A fresh check of the account, a human's consent and the transfer (D13).
fn execute_payout(room: &MemoryRoom, payout_id: &str, account_id: &str) {
    let check_id = format!("sprawdzenie-{payout_id}");
    let account_check = WpiszSprawdzenieParams {
        rachunek_id: Some(account_id.to_string()),
        ..check_of(
            &check_id,
            RodzajSprawdzenia::Rachunek,
            "2027-01-12T10:00:00Z",
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
        zlozono: "2027-01-12T11:00:00Z".to_string(),
        ..decision_request_about(
            &decision_id,
            RodzajDecyzji::Wyplata,
            "sprawa_wyplaty_wpisy",
            payout_id,
        )
    };
    approved_decision(room, &consent_request, "2027-01-12T12:00:00Z");

    let consent_to_payout = WskazZgodeNaWyplateParams {
        decyzja_id: decision_id,
        id: payout_id.to_string(),
    };
    sprawa_wyplaty::wskaz_zgode_na_wyplate(room, &consent_to_payout)
        .expect("point the payout to its consent");

    let transfer_id = format!("przelew-{payout_id}");
    add_received_document(
        room,
        &received_document(&transfer_id, RodzajDokumentu::PotwierdzenieBankowe),
    );
    let execution = OdnotujWykonanieWyplatyParams {
        wykonal: "tozsamosc-posrednika".to_string(),
        wykonano: "2027-01-12T13:00:00Z".to_string(),
        potwierdzenie_dokument_id: transfer_id,
        id: payout_id.to_string(),
    };
    sprawa_wyplaty::odnotuj_wykonanie_wyplaty(room, &execution).expect("execute the payout");
}

/// Writes a draft letter (see [`letter`]) from the template `wzor` to the tenant.
fn letter_to_tenant(room: &MemoryRoom, letter_id: &str, wzor: &str) {
    let params = PrzygotujWysylkeParams {
        adresat_rodzaj: None,
        adresat_nazwa: None,
        adresat_uczestnik_id: Some("najemca".to_string()),
        wzor: Some(wzor.to_string()),
        ..letter(letter_id)
    };

    prepare_letter(room, &params);
}

/// Records the sending of the letter `letter_id` at `wyslano`, with no decision.
fn send_at(room: &MemoryRoom, letter_id: &str, wyslano: &str) {
    let params = OdnotujWyslanieParams {
        wyslano: wyslano.to_string(),
        ..sending(letter_id)
    };

    send(room, &params);
}

/// A letter that always needs a human: waiting for consent, the decision, then sending at
/// `wyslano` (the Python default is „2027-02-25T10:00:00Z”).
fn send_with_consent(room: &MemoryRoom, letter_id: &str, wyslano: &str) {
    let for_consent = PrzekazWysylkeDoZgodyParams {
        id: letter_id.to_string(),
    };
    sprawa_korespondencja::przekaz_wysylke_do_zgody(room, &for_consent)
        .expect("hand the letter over for consent");

    let decision_id = format!("wyslanie-{letter_id}");
    let request = decision_request_about(
        &decision_id,
        RodzajDecyzji::Wyslanie,
        "sprawa_korespondencja_wysylki",
        letter_id,
    );
    ask_for_decision(room, &request);
    approve(room, &decision_id, "2026-11-03T12:00:00Z");

    let sending_by_decision = OdnotujWyslanieParams {
        decyzja_id: Some(decision_id),
        wyslano: wyslano.to_string(),
        ..sending(letter_id)
    };
    send(room, &sending_by_decision);
}

/// Records the delivery of the letter `letter_id` at `doreczono`.
fn deliver(room: &MemoryRoom, letter_id: &str, doreczono: &str) {
    let params = OdnotujDoreczenieParams {
        doreczono: doreczono.to_string(),
        dowod_doreczenia: None,
        id: letter_id.to_string(),
    };

    sprawa_korespondencja::odnotuj_doreczenie(room, &params).expect("record the delivery");
}

/// The default parameters of the tenant's payment of 3000 zł rent to the office's account on
/// 2026-12-08, shown by the office's statement `wyciag-<id>`.
fn rent_payment(payment_id: &str) -> WpiszWplateCzynszuParams {
    WpiszWplateCzynszuParams {
        id: payment_id.to_string(),
        najem_id: "najem".to_string(),
        chwila: "2026-12-08T10:00:00Z".to_string(),
        kwota: 300_000,
        placacy_uczestnik_id: Some("najemca".to_string()),
        na_rachunek: NaRachunek::Biura,
        rachunek_id: None,
        potwierdzenie: PotwierdzenieWplaty::WyciagBiura,
        dokument_id: format!("wyciag-{payment_id}"),
    }
}

/// Writes the received statement `wyciag-<id>` and the rent payment (see [`rent_payment`]) of
/// `kwota` (the Python defaults are „wplata-grudzien” and 300_000).
fn rent_paid_to_office(room: &MemoryRoom, payment_id: &str, kwota: i64) {
    let statement_id = format!("wyciag-{payment_id}");
    add_received_document(
        room,
        &received_document(&statement_id, RodzajDokumentu::PotwierdzenieBankowe),
    );

    let payment = WpiszWplateCzynszuParams {
        kwota,
        ..rent_payment(payment_id)
    };
    najem_czynsz::wpisz_wplate_czynszu(room, &payment).expect("write the rent payment");
}

/// Writes the rent of December 2026, 3000 zł due on 2026-12-10.
fn add_rent_due(room: &MemoryRoom) {
    let params = WpiszNaleznoscParams {
        id: "czynsz-grudzien".to_string(),
        najem_id: "najem".to_string(),
        tytul: TytulNaleznosci::Czynsz,
        okres: "2026-12".to_string(),
        kwota: 300_000,
        termin: "2026-12-10".to_string(),
    };

    najem_czynsz::wpisz_naleznosc(room, &params).expect("write the rent due");
}

fn settle_payment(room: &MemoryRoom, payment_id: &str, kwota: i64) {
    let params = RozliczWplateParams {
        wplata_id: payment_id.to_string(),
        naleznosc_id: "czynsz-grudzien".to_string(),
        kwota,
    };

    najem_czynsz::rozlicz_wplate(room, &params).expect("settle the payment");
}

/// The default parameters of the deposit „kaucja” of the lease „najem”: two rents of 3000 zł
/// paid on 2026-11-25 and kept by the office.
fn deposit() -> WpiszPobranieKaucjiParams {
    WpiszPobranieKaucjiParams {
        id: "kaucja".to_string(),
        najem_id: "najem".to_string(),
        kwota: 600_000,
        krotnosc: 2.0,
        czynsz_przy_zawarciu: 300_000,
        wplacono: "2026-11-25T13:00:00Z".to_string(),
        lezy: KaucjaLezy::UBiura,
        rachunek_id: None,
    }
}

fn collect_deposit(room: &MemoryRoom, params: &WpiszPobranieKaucjiParams) {
    najem_kaucja::wpisz_pobranie_kaucji(room, params).expect("write the deposit");
}

fn deposit_at_office(room: &MemoryRoom) {
    collect_deposit(room, &deposit());
}

/// The default parameters of the settlement „rozliczenie” of the deposit: the flat emptied on
/// 2027-08-31, the deposit indexed to the rent of 3200 zł on that day.
fn deposit_settlement() -> PrzygotujRozliczenieKaucjiParams {
    PrzygotujRozliczenieKaucjiParams {
        id: "rozliczenie".to_string(),
        pobranie_id: "kaucja".to_string(),
        oproznienie: "2027-08-31".to_string(),
        czynsz_w_dniu_zwrotu: 320_000,
        kwota_zwaloryzowana: 640_000,
        wyliczenie_sciezka: "wyliczenia/rozliczenie.json".to_string(),
        wyliczenie_skrot: "sha256-rozliczenie".to_string(),
    }
}

fn prepare_deposit_settlement(room: &MemoryRoom) {
    najem_kaucja::przygotuj_rozliczenie_kaucji(room, &deposit_settlement())
        .expect("prepare the deposit settlement");
}

fn approve_deposit_settlement(room: &MemoryRoom) {
    let params = ZatwierdzRozliczenieKaucjiParams {
        id: "rozliczenie".to_string(),
    };

    najem_kaucja::zatwierdz_rozliczenie_kaucji(room, &params)
        .expect("approve the deposit settlement");
}

/// The parameters of sending the deposit settlement „rozliczenie” in the letter
/// „rozliczenie-kaucji”.
fn settlement_sent() -> OdnotujWyslanieRozliczeniaKaucjiParams {
    OdnotujWyslanieRozliczeniaKaucjiParams {
        wysylka_id: "rozliczenie-kaucji".to_string(),
        id: "rozliczenie".to_string(),
    }
}

/// The default parameters of a deduction of 400 zł from the settlement „rozliczenie” on the
/// basis `podstawa-<id>`.
fn deduction(deduction_id: &str) -> DopiszPotracenieZKaucjiParams {
    DopiszPotracenieZKaucjiParams {
        id: deduction_id.to_string(),
        rozliczenie_id: "rozliczenie".to_string(),
        tytul: "naprawa blatu z protokołu zwrotu".to_string(),
        kwota: 40_000,
        dokument_id: format!("podstawa-{deduction_id}"),
    }
}

/// Writes the received basis `podstawa-<id>` and the deduction of `params` (see [`deduction`]),
/// with no consent yet.
fn add_deduction(room: &MemoryRoom, params: &DopiszPotracenieZKaucjiParams) {
    add_received_document(
        room,
        &received_document(&params.dokument_id, RodzajDokumentu::Inny),
    );

    najem_kaucja::dopisz_potracenie_z_kaucji(room, params).expect("add the deduction");
}

/// Points the deduction `deduction_id` to the approved decision `decision_id`.
fn consent_to_deduction(room: &MemoryRoom, deduction_id: &str, decision_id: &str) {
    let params = WskazZgodeNaPotracenieParams {
        decyzja_id: decision_id.to_string(),
        id: deduction_id.to_string(),
    };

    najem_kaucja::wskaz_zgode_na_potracenie(room, &params)
        .expect("point the deduction to its consent");
}

/// [`add_deduction`] with a human's consent `zgoda-<id>` (Python: `approved=True`).
fn add_approved_deduction(room: &MemoryRoom, params: &DopiszPotracenieZKaucjiParams) {
    add_deduction(room, params);

    let decision_id = format!("zgoda-{}", params.id);
    approved_decision_about(
        room,
        &decision_id,
        RodzajDecyzji::Inna,
        "najem_kaucja_potracenia",
        &params.id,
    );
    consent_to_deduction(room, &params.id, &decision_id);
}

/// The tenant reports a leaking siphon in the inflow `zgloszenie-usterka`; it burdens `obciaza`
/// (the Python default is „wynajmujacy”).
fn report_defect(room: &MemoryRoom, obciaza: ObciazaUsterka) {
    add_inflow_from(room, "zgloszenie-usterka", "najemca");

    let report = ZglosUsterkeParams {
        id: "usterka".to_string(),
        chwila: "2027-01-20T08:00:00Z".to_string(),
        zglosil_uczestnik_id: Some("najemca".to_string()),
        wplyw_id: Some("zgloszenie-usterka".to_string()),
        opis: "Cieknie syfon pod zlewem".to_string(),
        zdjecia_sciezka: None,
        zdjecia_skrot: None,
        uniemozliwia: false,
        obciaza: Some(obciaza),
    };
    najem_usterki::zglos_usterke(room, &report).expect("report the defect");
}

/// The default parameters of the broker's order of the repair of „usterka” for 300 zł.
fn repair() -> ZlecNapraweUsterkiParams {
    ZlecNapraweUsterkiParams {
        koszt: 30_000,
        zlecil: "tozsamosc-posrednika".to_string(),
        zlecono: "2027-01-21T09:00:00Z".to_string(),
        wykonawca: "Hydraulik Jan Kowalski".to_string(),
        zgoda_wynajmujacego_id: None,
        zlecenie_decyzja_id: None,
        id: "usterka".to_string(),
    }
}

fn order_repair(room: &MemoryRoom, params: &ZlecNapraweUsterkiParams) {
    najem_usterki::zlec_naprawe_usterki(room, params).expect("order the repair");
}

/// The parameters of the end of the lease `najem_id` in the way `sposob`, with no letter.
fn lease_end(end_id: &str, najem_id: &str, sposob: SposobKonca) -> WpiszKoniecNajmuParams {
    WpiszKoniecNajmuParams {
        id: end_id.to_string(),
        najem_id: najem_id.to_string(),
        sposob,
        przyczyna: None,
        wysylka_id: None,
        wplyw_id: None,
        koniec_najmu: None,
    }
}

fn end_lease(room: &MemoryRoom, params: &WpiszKoniecNajmuParams) {
    najem_umowy::wpisz_koniec_najmu(room, params).expect("write the end of the lease");
}

/// The default parameters of the rent data of 2027 for the municipality of Wrocław.
fn municipality_data(najem_id: &str) -> PrzygotujDaneDlaGminyParams {
    PrzygotujDaneDlaGminyParams {
        najem_id: najem_id.to_string(),
        okres: "2027".to_string(),
        czynsz: 300_000,
        gmina: "Wrocław".to_string(),
        rok_budowy: None,
        stan_techniczny: None,
        powierzchnia: None,
        standard: None,
    }
}

/// The default parameters of the reading of a meter of the lease „najem” on 2027-01-05.
fn meter_reading(
    licznik: Licznik,
    numer: &str,
    stan: &str,
    photo: &str,
) -> WpiszOdczytLicznikaParams {
    WpiszOdczytLicznikaParams {
        najem_id: "najem".to_string(),
        licznik,
        numer: numer.to_string(),
        chwila: "2027-01-05T09:00:00Z".to_string(),
        stan: stan.to_string(),
        zdjecie_sciezka: format!("odczyty/{photo}.jpg"),
        zdjecie_skrot: format!("sha256-{photo}"),
    }
}

/// The default parameters of the arrears „zaleglosc” of the lease `najem_id`, three periods
/// overdue from 2027-02-11.
fn arrears(arrears_id: &str, najem_id: &str) -> WpiszZalegloscParams {
    WpiszZalegloscParams {
        id: arrears_id.to_string(),
        najem_id: najem_id.to_string(),
        trzy_okresy_od: "2027-02-11".to_string(),
    }
}

fn add_arrears(room: &MemoryRoom, params: &WpiszZalegloscParams) {
    najem_czynsz::wpisz_zaleglosc(room, params).expect("write the arrears");
}

fn add_fee(room: &MemoryRoom, params: &DopiszOplateNajmuParams) {
    najem_umowy::dopisz_oplate_najmu(room, params).expect("add the fee");
}

fn prepare_annex(room: &MemoryRoom, annex_id: &str, opis: &str) {
    let params = PrzygotujAneksNajmuParams {
        id: annex_id.to_string(),
        najem_id: "najem".to_string(),
        opis: opis.to_string(),
        projekt_dokument_id: None,
    };

    najem_umowy::przygotuj_aneks_najmu(room, &params).expect("prepare the annex");
}

fn consent_to_annex(room: &MemoryRoom, annex_id: &str, decision_id: &str) {
    let params = WskazZgodeNaAneksNajmuParams {
        decyzja_id: decision_id.to_string(),
        id: annex_id.to_string(),
    };

    najem_umowy::wskaz_zgode_na_aneks_najmu(room, &params).expect("point the annex to its consent");
}

/// The parameters of the annex `annex_id` signed in the document `dokument_id`.
fn annex_signing(
    annex_id: &str,
    dokument_id: &str,
    obowiazuje_od: &str,
) -> OdnotujPodpisanieAneksuNajmuParams {
    OdnotujPodpisanieAneksuNajmuParams {
        dokument_id: dokument_id.to_string(),
        obowiazuje_od: obowiazuje_od.to_string(),
        id: annex_id.to_string(),
    }
}

/// An ordinary lease the office manages, from the contract to the return of the deposit.
fn full_managed_rental(room: &MemoryRoom) {
    open_rental(room);
    add_rental_parties(room);
    add_person(room, &person_named("partnerka", "Ewa", "Zielińska"));
    give_role(room, "partnerka", "zamieszkujacy");
    add_flat(room, &flat("lokal"));
    let source_case = WpiszPowiazanieParams {
        rodzaj: RodzajPowiazania::SprawaZrodlowa,
        typ_teczki: Some("oferta".to_string()),
        oznaczenie: Some("Oferta 2026/041".to_string()),
        odnosnik: "pokoj-oferty-041".to_string(),
        wpisal: "tozsamosc-kierownika-najmu".to_string(),
        wpisano: "2026-11-02T09:15:00Z".to_string(),
    };
    sprawa_powiazania::wpisz_powiazanie(room, &source_case).expect("link the source case");

    // The management contract: repairs up to 500 zł without asking, money through the office.
    move_to(room, &["umowa_z_biurem"]);
    management(room, Przeplyw::PrzezBiuro);

    // The lease: an ordinary lease for two years, signed after a human's consent.
    move_to(room, &["projekt_umowy"]);
    prepare_lease(room, &lease("najem", RodzajNajmu::Zwykly));
    add_fee(
        room,
        &DopiszOplateNajmuParams {
            najem_id: "najem".to_string(),
            rodzaj: RodzajOplaty::Energia,
            sposob: "najemca płaci dostawcy według licznika".to_string(),
            umowa_z_dostawca_ma: UmowaZDostawcaMa::Najemca,
        },
    );
    add_fee(
        room,
        &DopiszOplateNajmuParams {
            najem_id: "najem".to_string(),
            rodzaj: RodzajOplaty::Woda,
            sposob: "zaliczka 80 zł miesięcznie, rozliczenie według wodomierza".to_string(),
            umowa_z_dostawca_ma: UmowaZDostawcaMa::Wynajmujacy,
        },
    );
    lease_to_signing(room, "najem");
    move_to(room, &["podpisanie_umowy"]);
    conclude_lease(room, "najem");
    najem_umowy::odnotuj_przekazanie_swiadectwa(
        room,
        &OdnotujPrzekazanieSwiadectwaParams {
            swiadectwo_przekazano: "2026-11-25".to_string(),
            id: "najem".to_string(),
        },
    )
    .expect("record the energy certificate handed over");
    najem_umowy::wpisz_zamieszkujacego(
        room,
        &WpiszZamieszkujacegoParams {
            najem_id: "najem".to_string(),
            uczestnik_id: "partnerka".to_string(),
            od: "2026-12-01".to_string(),
        },
    )
    .expect("write the partner living with the tenant");
    deposit_at_office(room);

    // Handover with a signed protocol; the lease runs.
    move_to(room, &["wydanie_lokalu"]);
    signed_protocol(room, RodzajProtokolu::Wydanie, "2026-11-30T10:00:00Z");
    move_to(room, &["trwanie_najmu"]);

    // The rent of December through the office and its transfer to the owner (T13).
    add_rent_due(room);
    rent_paid_to_office(room, "wplata-grudzien", 300_000);
    settle_payment(room, "wplata-grudzien", 300_000);
    admitted_account_for(
        room,
        "rachunek-wlascicielki",
        "wlascicielka",
        CelRachunku::WyplataDlaWlasciciela,
        "Anna Kowalska",
    );
    prepare_payout(
        room,
        &rent_transfer(
            "przekazanie-grudzien",
            270_000,
            "wlascicielka",
            "rachunek-wlascicielki",
        ),
    );
    execute_payout(room, "przekazanie-grudzien", "rachunek-wlascicielki");

    // Meters, a defect within the repair limit, an inspection.
    najem_czynsz::wpisz_odczyt_licznika(
        room,
        &meter_reading(Licznik::WodaZimna, "W-1201", "231,4 m³", "woda-2027-01"),
    )
    .expect("write the meter reading");
    report_defect(room, ObciazaUsterka::Wynajmujacy);
    order_repair(room, &repair());
    add_inflow_from(room, "potwierdzenie-usuniecia", "najemca");
    najem_usterki::odnotuj_usuniecie_usterki(
        room,
        &OdnotujUsuniecieUsterkiParams {
            usunieto: "2027-01-22".to_string(),
            potwierdzenie_wplyw_id: "potwierdzenie-usuniecia".to_string(),
            id: "usterka".to_string(),
        },
    )
    .expect("record the defect removed");
    najem_usterki::wpisz_przeglad(
        room,
        &WpiszPrzegladParams {
            data: "2027-02-01".to_string(),
            wynik: "Lokal bez uwag".to_string(),
            dokument_id: None,
        },
    )
    .expect("write the inspection");

    // An annex allowing a cat, and a rent increase by notice (art. 8a UOPL).
    prepare_annex(room, "aneks-kot", "Zgoda na kota");
    approved_decision_about(
        room,
        "zgoda-aneks-kot",
        RodzajDecyzji::Inna,
        "najem_umowy_aneksy",
        "aneks-kot",
    );
    consent_to_annex(room, "aneks-kot", "zgoda-aneks-kot");
    add_received_document(
        room,
        &received_document("podpisany-aneks-kot", RodzajDokumentu::Umowa),
    );
    najem_umowy::odnotuj_podpisanie_aneksu_najmu(
        room,
        &annex_signing("aneks-kot", "podpisany-aneks-kot", "2027-01-15"),
    )
    .expect("record the annex signed");
    letter_to_tenant(room, "podwyzka", "wypowiedzenie-wysokosci-czynszu");
    send_at(room, "podwyzka", "2027-01-14T10:00:00Z");
    deliver(room, "podwyzka", "2027-01-15T10:00:00Z");
    najem_czynsz::wpisz_zmiane_czynszu(
        room,
        &WpiszZmianeCzynszuParams {
            najem_id: "najem".to_string(),
            stara: 300_000,
            nowa: 320_000,
            pismo_wysylka_id: Some("podwyzka".to_string()),
            doreczono: Some("2027-01-15".to_string()),
            obowiazuje_od: "2027-05-01".to_string(),
            podstawa: "art. 8a UOPL, wzrost kosztów utrzymania".to_string(),
            aneks_id: None,
        },
    )
    .expect("write the rent increase");

    // Arrears of three periods: a warning sent with a human's consent, an extra month, paid.
    add_arrears(room, &arrears("zaleglosc", "najem"));
    letter_to_tenant(room, "uprzedzenie", "uprzedzenie-o-wypowiedzeniu");
    najem_czynsz::wskaz_uprzedzenie_o_zaleglosci(
        room,
        &WskazUprzedzenieOZaleglosciParams {
            uprzedzenie_wysylka_id: "uprzedzenie".to_string(),
            id: "zaleglosc".to_string(),
        },
    )
    .expect("point the arrears to the warning");
    send_with_consent(room, "uprzedzenie", "2027-02-25T10:00:00Z");
    deliver(room, "uprzedzenie", "2027-02-26T10:00:00Z");
    najem_czynsz::ustal_dodatkowy_termin_zaleglosci(
        room,
        &UstalDodatkowyTerminZaleglosciParams {
            dodatkowy_termin_do: "2027-03-26".to_string(),
            id: "zaleglosc".to_string(),
        },
    )
    .expect("set the extra term");
    najem_czynsz::odnotuj_wynik_zaleglosci(
        room,
        &OdnotujWynikZaleglosciParams {
            wynik: WynikZaleglosci::Splacona,
            id: "zaleglosc".to_string(),
        },
    )
    .expect("record the arrears paid");

    // Rent data for the municipality, sent by a human.
    najem_czynsz::przygotuj_dane_dla_gminy(
        room,
        &PrzygotujDaneDlaGminyParams {
            czynsz: 320_000,
            rok_budowy: Some(1998),
            stan_techniczny: Some("dobry".to_string()),
            powierzchnia: Some(4820),
            standard: Some("wyposażenie z protokołu wydania".to_string()),
            ..municipality_data("najem")
        },
    )
    .expect("prepare the data for the municipality");
    najem_czynsz::odnotuj_oddanie_danych_dla_gminy(
        room,
        &OdnotujOddanieDanychDlaGminyParams {
            oddano: "2027-03-01".to_string(),
            id: 1,
        },
    )
    .expect("record the data handed in");
    najem_czynsz::odnotuj_wyslanie_danych_dla_gminy(
        room,
        &OdnotujWyslanieDanychDlaGminyParams {
            wyslano: "2027-03-05".to_string(),
            wyslal: "tozsamosc-kierownika-najmu".to_string(),
            id: 1,
        },
    )
    .expect("record the data sent");

    // The parties end the lease by agreement; the flat comes back with a protocol.
    move_to(room, &["koniec_najmu"]);
    end_lease(
        room,
        &WpiszKoniecNajmuParams {
            koniec_najmu: Some("2027-08-31".to_string()),
            ..lease_end("koniec", "najem", SposobKonca::Porozumienie)
        },
    );
    najem_umowy::odnotuj_oproznienie_lokalu(
        room,
        &OdnotujOproznienieLokaluParams {
            oproznienie: "2027-08-31".to_string(),
            id: "koniec".to_string(),
        },
    )
    .expect("record the flat emptied");
    najem_umowy::zakoncz_umowe_najmu(
        room,
        &ZakonczUmoweNajmuParams {
            koniec_powod: "porozumienie stron".to_string(),
            id: "najem".to_string(),
        },
    )
    .expect("end the lease");
    move_to(room, &["zwrot_lokalu"]);
    signed_protocol(room, RodzajProtokolu::Zwrot, "2027-08-31T10:00:00Z");

    // The deposit: indexed to the rent on the return day, one deduction, back to the tenant.
    move_to(room, &["rozliczenie_kaucji"]);
    prepare_deposit_settlement(room);
    add_approved_deduction(room, &deduction("potracenie-blat"));
    approve_deposit_settlement(room);
    letter_to_tenant(room, "rozliczenie-kaucji", "rozliczenie-kaucji");
    send_at(room, "rozliczenie-kaucji", "2027-09-10T10:00:00Z");
    najem_kaucja::odnotuj_wyslanie_rozliczenia_kaucji(room, &settlement_sent())
        .expect("record the settlement sent");
    admitted_account_for(
        room,
        "rachunek-najemcy",
        "najemca",
        CelRachunku::ZwrotKaucji,
        "Jan Nowak",
    );
    prepare_payout(
        room,
        &deposit_return("zwrot-kaucji", 600_000, "najemca", "rachunek-najemcy"),
    );
    execute_payout(room, "zwrot-kaucji", "rachunek-najemcy");
    move_to(room, &["po_najmie"]);
}

/// An occasional lease the office only brokers: it ends after the tax office notification.
fn full_brokered_occasional_rental(room: &MemoryRoom) {
    open_rental(room);
    add_rental_parties(room);
    add_person(room, &person_named("wlasciciel-lipowej", "Piotr", "Lis"));
    give_role(room, "wlasciciel-lipowej", "wlasciciel_innego_lokalu");
    add_flat(room, &flat("lokal"));

    move_to(room, &["projekt_umowy"]);
    prepare_lease(
        room,
        &PrzygotujUmoweNajmuParams {
            kaucja: 900_000,
            ..lease("najem", RodzajNajmu::Okazjonalny)
        },
    );
    add_substitute_flat(room, "najem");
    move_to(room, &["oswiadczenie_najemcy"]);
    add_tenant_declaration(room, "najem");
    lease_to_signing(room, "najem");
    move_to(room, &["podpisanie_umowy"]);
    conclude_lease(room, "najem");
    admitted_account_for(
        room,
        "rachunek-kaucji",
        "wlascicielka",
        CelRachunku::Kaucja,
        "Anna Kowalska",
    );
    collect_deposit(
        room,
        &WpiszPobranieKaucjiParams {
            kwota: 900_000,
            krotnosc: 3.0,
            lezy: KaucjaLezy::UWynajmujacego,
            rachunek_id: Some("rachunek-kaucji".to_string()),
            ..deposit()
        },
    );

    move_to(room, &["wydanie_lokalu"]);
    signed_protocol(room, RodzajProtokolu::Wydanie, "2026-11-30T10:00:00Z");
    move_to(room, &["zgloszenie_najmu"]);
    add_received_document(
        room,
        &received_document("zgloszenie-us", RodzajDokumentu::Pismo),
    );
    najem_umowy::odnotuj_zgloszenie_najmu_do_urzedu(
        room,
        &OdnotujZgloszenieNajmuDoUrzeduParams {
            zgloszono_us: "2026-12-10".to_string(),
            zgloszenie_us_dokument_id: "zgloszenie-us".to_string(),
            id: "najem".to_string(),
        },
    )
    .expect("record the tax office notification");
    move_to(room, &["po_najmie", "zamknieta"]);
}

// The whole case ------------------------------------------------------------------------------------

/// Przebieg najmu z zarządzaniem: umowa, wydanie, czynsz przez biuro, koniec, zwrot kaucji
#[test]
fn whole_managed_rental() {
    let room = new_room();

    full_managed_rental(&room);
    move_to(&room, &["zamknieta"]);

    let stages = sprawa_metryka::etapy(&room).expect("read the stages");
    let etapy: Vec<&str> = stages.iter().map(|row| row.etap.as_str()).collect();
    assert_eq!(
        etapy,
        [
            "przyjeta",
            "umowa_z_biurem",
            "projekt_umowy",
            "podpisanie_umowy",
            "wydanie_lokalu",
            "trwanie_najmu",
            "koniec_najmu",
            "zwrot_lokalu",
            "rozliczenie_kaucji",
            "po_najmie",
            "zamknieta",
        ]
    );

    let payouts = sprawa_wyplaty::wyplaty(&room).expect("read the payouts");
    let payout_states: Vec<StanWyplaty> = payouts.iter().map(|row| row.stan).collect();
    assert_eq!(
        payout_states,
        [StanWyplaty::Wykonana, StanWyplaty::Wykonana]
    );

    let balances = najem_czynsz::saldo_naleznosci(&room).expect("read the balance");
    let balance = &balances[0];
    assert!(
        balance.zaplacono == 300_000 && balance.zostalo == 0,
        "{balance:?}"
    );

    let rent_changes = najem_czynsz::zmiany_czynszu(&room).expect("read the rent changes");
    assert_eq!(rent_changes[0].nowa, 320_000);

    let settlements = najem_kaucja::rozliczenia_kaucji(&room).expect("read the settlements");
    assert_eq!(settlements[0].stan, StanRozliczeniaKaucji::Wyslane);

    let data = najem_czynsz::dane_dla_gminy(&room).expect("read the data for the municipality");
    assert_eq!(
        data[0].wyslal.as_deref(),
        Some("tozsamosc-kierownika-najmu")
    );
}

/// Przebieg najmu okazjonalnego przy samym pośrednictwie: załączniki, wydanie, zgłoszenie
#[test]
fn whole_brokered_occasional_rental() {
    let room = new_room();

    full_brokered_occasional_rental(&room);

    let stages = sprawa_metryka::etapy(&room).expect("read the stages");
    let etapy: Vec<&str> = stages.iter().map(|row| row.etap.as_str()).collect();
    assert_eq!(
        etapy,
        [
            "przyjeta",
            "projekt_umowy",
            "oswiadczenie_najemcy",
            "podpisanie_umowy",
            "wydanie_lokalu",
            "zgloszenie_najmu",
            "po_najmie",
            "zamknieta",
        ]
    );

    let leases = najem_umowy::umowy_najmu(&room).expect("read the leases");
    assert_eq!(leases[0].zgloszono_us.as_deref(), Some("2026-12-10"));
}

// Rules on the core -------------------------------------------------------------------------------

/// Reguły: teczka najmu ma typ pokoju najem (T2)
#[test]
fn room_type() {
    let room = new_room();

    let record_of_transaction = ZalozMetrykeParams {
        oznaczenie: "Transakcja 2026/003".to_string(),
        typ_pokoju: "transakcja".to_string(),
        przyjal: "tozsamosc-posrednika".to_string(),
        przyjeto: "2026-11-02T09:00:00Z".to_string(),
    };
    expect::refused(
        sprawa_metryka::zaloz_metryke(&room, &record_of_transaction),
        "teczka najmu ma typ pokoju najem",
    );

    open_rental(&room);
    let record = sprawa_metryka::metryka(&room)
        .expect("read the record")
        .expect("the case has a record");
    assert_eq!(record.typ_pokoju, "najem");
}

/// Reguły: etapy najmu idą po kolei; umowę z biurem pomija się przy samym pośrednictwie
#[test]
fn stages_in_order() {
    let room = new_room();
    open_rental(&room);

    let rule = "etapy najmu idą po kolei";
    refused_stage(&room, rule, "umowa");
    refused_stage(&room, rule, "wydanie_lokalu");
    move_to(&room, &["projekt_umowy"]);
    refused_stage(&room, rule, "umowa_z_biurem");

    approved_decision(
        &room,
        &decision_request("przerwanie", RodzajDecyzji::PrzerwanieSprawy),
        "2026-11-03T12:00:00Z",
    );
    move_by_decision(&room, "przerwana", "przerwanie");
    refused_stage(&room, rule, "podpisanie_umowy");
    move_to(&room, &["zamknieta"]);
}

/// Reguły: wznowiony najem wraca do etapu sprzed przerwania i idzie dalej po kolei
#[test]
fn stages_resumed() {
    let room = new_room();
    open_rental(&room);
    move_to(&room, &["projekt_umowy"]);
    approved_decision(
        &room,
        &decision_request("przerwanie", RodzajDecyzji::PrzerwanieSprawy),
        "2026-11-03T12:00:00Z",
    );
    move_by_decision(&room, "przerwana", "przerwanie");
    approved_decision(
        &room,
        &decision_request("wznowienie", RodzajDecyzji::WznowienieSprawy),
        "2026-11-05T12:00:00Z",
    );

    let back_to_office_contract = ZmienEtapSprawyParams {
        etap: "umowa_z_biurem".to_string(),
        od: "2026-11-06".to_string(),
        zmienil: "agent".to_string(),
        decyzja_id: Some("wznowienie".to_string()),
    };
    expect::refused(
        sprawa_metryka::zmien_etap_sprawy(&room, &back_to_office_contract),
        "etapy najmu idą po kolei",
    );
    move_by_decision(&room, "projekt_umowy", "wznowienie");
    move_to(&room, &["oswiadczenie_najemcy"]);
}

/// Reguły: z wydania lokalu i ze zwrotu wychodzi się z podpisanym protokołem
#[test]
fn stages_need_protocols() {
    let room = new_room();
    open_rental(&room);
    add_rental_parties(&room);
    add_flat(&room, &flat("lokal"));
    concluded_lease(&room, &lease("najem", RodzajNajmu::Zwykly));
    move_to(
        &room,
        &["projekt_umowy", "podpisanie_umowy", "wydanie_lokalu"],
    );

    refused_stage(
        &room,
        "sprawa wychodzi z wydania lokalu tylko z podpisanym protokołem wydania",
        "trwanie_najmu",
    );
    signed_protocol(&room, RodzajProtokolu::Wydanie, "2026-11-30T10:00:00Z");
    move_to(&room, &["trwanie_najmu", "koniec_najmu", "zwrot_lokalu"]);

    refused_stage(
        &room,
        "sprawa wychodzi ze zwrotu lokalu tylko z podpisanym protokołem zwrotu",
        "po_najmie",
    );
    signed_protocol(&room, RodzajProtokolu::Zwrot, "2027-08-31T10:00:00Z");
    move_to(&room, &["po_najmie"]);
}

/// Reguły: przy umowie o zarządzanie najem nie kończy się po wydaniu lokalu (R6)
#[test]
fn management_does_not_end_after_handover() {
    let room = new_room();
    open_rental(&room);
    add_rental_parties(&room);
    add_flat(&room, &flat("lokal"));
    concluded_lease(&room, &lease("najem", RodzajNajmu::Zwykly));
    move_to(
        &room,
        &["projekt_umowy", "podpisanie_umowy", "wydanie_lokalu"],
    );
    signed_protocol(&room, RodzajProtokolu::Wydanie, "2026-11-30T10:00:00Z");
    management(&room, Przeplyw::PrzezBiuro);

    let rule = "przy zawartej umowie o zarządzanie najem nie kończy się po wydaniu lokalu (R6)";
    refused_stage(&room, rule, "po_najmie");
    refused_stage(&room, rule, "zamknieta");

    sprawa_umowy::zakoncz_umowe(
        &room,
        &ZakonczUmoweParams {
            koniec_powod: "wypowiedzenie przez właścicielkę".to_string(),
            id: "zarzadzanie".to_string(),
        },
    )
    .expect("end the management contract");
    move_to(&room, &["po_najmie"]);
}

/// Reguły: role, zgody i tylko umowa o zarządzanie w teczce najmu
#[test]
fn roles_consents_and_office_contract() {
    let room = new_room();
    open_rental(&room);
    add_rental_parties(&room);

    let buyer = NadajRoleUczestnikowiParams {
        uczestnik_id: "najemca".to_string(),
        rola: "kupujacy".to_string(),
        dotyczy_uczestnik_id: None,
    };
    expect::refused(
        sprawa_uczestnicy::nadaj_role_uczestnikowi(&room, &buyer),
        "rola w teczce najmu to wynajmujacy",
    );
    sprawa_uczestnicy::nadaj_role_uczestnikowi(
        &room,
        &NadajRoleUczestnikowiParams {
            uczestnik_id: "najemca".to_string(),
            rola: "malzonek".to_string(),
            dotyczy_uczestnik_id: Some("wlascicielka".to_string()),
        },
    )
    .expect("give the tenant the role of the owner's spouse");

    add_inflow_from(&room, "oswiadczenie", "wlascicielka");
    let statement = |consent_id: &str, rodzaj: &str| WpiszZgodeParams {
        tresc: "Treść oświadczenia".to_string(),
        zlozono: "2026-11-04T10:00:00Z".to_string(),
        wplyw_id: Some("oswiadczenie".to_string()),
        trwaly_nosnik: true,
        ..consent(consent_id, "wlascicielka", rodzaj)
    };
    expect::refused(
        sprawa_zgody::wpisz_zgode(&room, &statement("cena", "zatwierdzenie_ceny")),
        "zgoda w teczce najmu to wybor_rodzaju_najmu",
    );
    sprawa_zgody::wpisz_zgode(&room, &statement("rodzaj-najmu", "wybor_rodzaju_najmu"))
        .expect("write the choice of the kind of lease");
    sprawa_zgody::wpisz_zgode(&room, &statement("zadanie", "zadanie_rozpoczecia_pracy"))
        .expect("write the demand to start work");

    let refusal = "umowa z biurem w teczce najmu to umowa o zarządzanie";
    let brokerage = PrzygotujUmoweParams {
        rodzaj: RodzajUmowy::Posrednictwo,
        wylacznosc: false,
        wynagrodzenie_opis: None,
        ..contract("posrednictwo")
    };
    expect::refused(sprawa_umowy::przygotuj_umowe(&room, &brokerage), refusal);
    sprawa_umowy::przygotuj_umowe(
        &room,
        &PrzygotujUmoweParams {
            wynagrodzenie_opis: None,
            ..contract("zarzadzanie")
        },
    )
    .expect("prepare the management contract");
    expect::refused(
        room.execute(
            "UPDATE sprawa_umowy_wpisy SET rodzaj = 'posrednictwo' WHERE id = 'zarzadzanie'",
            &[],
        ),
        "rodzaj umowy się nie zmienia",
    );
}

// Management terms --------------------------------------------------------------------------------

/// Zarządzanie: pierwsze warunki od zawarcia umowy, kolejne od jej aneksu
#[test]
fn management_terms_versions() {
    let room = new_room();
    open_rental(&room);
    add_rental_parties(&room);
    concluded_contract(
        &room,
        "zarzadzanie",
        "wlascicielka",
        SposobZawarcia::WLokaluBiura,
    );

    let rule = "pierwsze warunki zarządzania obowiązują od zawarcia umowy";
    expect::refused(
        najem_umowy::wpisz_warunki_zarzadzania(&room, &management_terms("2026-11-01T00:00:00Z")),
        rule,
    );
    najem_umowy::wpisz_warunki_zarzadzania(&room, &management_terms(MANAGEMENT_CONCLUDED))
        .expect("write the first terms");

    // The next terms need an annex of the same contract.
    expect::refused(
        najem_umowy::wpisz_warunki_zarzadzania(&room, &management_terms("2026-12-01T00:00:00Z")),
        rule,
    );
    add_management_annex(&room, "2026-12-01T00:00:00Z");
    najem_umowy::wpisz_warunki_zarzadzania(
        &room,
        &WpiszWarunkiZarzadzaniaParams {
            kwota_napraw: 100_000,
            przeplyw: Przeplyw::Wprost,
            aneks_zdarzenie_id: Some("aneks".to_string()),
            ..management_terms("2026-12-01T00:00:00Z")
        },
    )
    .expect("write the terms of the annex");
    let terms = najem_umowy::warunki_zarzadzania(&room).expect("read the management terms");
    let current_terms = terms.last().expect("the contract has terms");
    assert_eq!(current_terms.przeplyw, Przeplyw::Wprost);

    expect::refused(
        room.execute("UPDATE najem_umowy_zarzadzanie SET kwota_napraw = 0", &[]),
        "warunków zarządzania się nie zmienia",
    );
}

// Lease -------------------------------------------------------------------------------------------

/// Umowa najmu: czas, forma i granica kaucji według rodzaju najmu (UOPL)
#[test]
fn lease_kind_limits() {
    let room = new_room();
    open_rental(&room);
    add_flat(&room, &flat("lokal"));

    let occasional_over_ten_years = PrzygotujUmoweNajmuParams {
        rodzaj: RodzajNajmu::Okazjonalny,
        koniec: Some("2036-12-02".to_string()),
        kaucja: 0,
        ..lease_fields()
    };
    expect::refused(
        najem_umowy::przygotuj_umowe_najmu(&room, &occasional_over_ten_years),
        "okazjonalny_najwyzej_10_lat",
    );
    let occasional_indefinite = PrzygotujUmoweNajmuParams {
        rodzaj: RodzajNajmu::Okazjonalny,
        koniec: None,
        kaucja: 0,
        ..lease_fields()
    };
    expect::refused(
        najem_umowy::przygotuj_umowe_najmu(&room, &occasional_indefinite),
        "okazjonalny_najwyzej_10_lat",
    );
    let institutional_indefinite = PrzygotujUmoweNajmuParams {
        rodzaj: RodzajNajmu::Instytucjonalny,
        koniec: None,
        kaucja: 0,
        ..lease_fields()
    };
    expect::refused(
        najem_umowy::przygotuj_umowe_najmu(&room, &institutional_indefinite),
        "instytucjonalny_na_czas_oznaczony",
    );
    let institutional_documentary = PrzygotujUmoweNajmuParams {
        rodzaj: RodzajNajmu::Instytucjonalny,
        forma: FormaNajmu::Dokumentowa,
        kaucja: 0,
        ..lease_fields()
    };
    expect::refused(
        najem_umowy::przygotuj_umowe_najmu(&room, &institutional_documentary),
        "forma_pisemna_przy_okazjonalnym_i_instytucjonalnym",
    );
    let ordinary_deposit_too_high = PrzygotujUmoweNajmuParams {
        kaucja: 3_600_001,
        ..lease_fields()
    };
    expect::refused(
        najem_umowy::przygotuj_umowe_najmu(&room, &ordinary_deposit_too_high),
        "kaucja_w_granicy",
    );
    let occasional_deposit_too_high = PrzygotujUmoweNajmuParams {
        rodzaj: RodzajNajmu::Okazjonalny,
        kaucja: 1_800_001,
        ..lease_fields()
    };
    expect::refused(
        najem_umowy::przygotuj_umowe_najmu(&room, &occasional_deposit_too_high),
        "kaucja_w_granicy",
    );

    najem_umowy::przygotuj_umowe_najmu(
        &room,
        &PrzygotujUmoweNajmuParams {
            id: "zwykly".to_string(),
            kaucja: 3_600_000,
            forma: FormaNajmu::Dokumentowa,
            koniec: None,
            ..lease_fields()
        },
    )
    .expect("prepare the ordinary lease");
    najem_umowy::przygotuj_umowe_najmu(
        &room,
        &PrzygotujUmoweNajmuParams {
            id: "okazjonalny".to_string(),
            rodzaj: RodzajNajmu::Okazjonalny,
            koniec: Some("2036-12-01".to_string()),
            kaucja: 1_800_000,
            ..lease_fields()
        },
    )
    .expect("prepare the occasional lease");

    add_received_document(
        &room,
        &received_document("oswiadczenie", RodzajDokumentu::AktNotarialny),
    );
    let declaration_of = |lease_id: &str| WskazOswiadczenieNajemcyParams {
        oswiadczenie_dokument_id: "oswiadczenie".to_string(),
        id: lease_id.to_string(),
    };
    expect::refused(
        najem_umowy::wskaz_oswiadczenie_najemcy(&room, &declaration_of("zwykly")),
        "oswiadczenie_przy_okazjonalnym_i_instytucjonalnym",
    );
    najem_umowy::wskaz_oswiadczenie_najemcy(&room, &declaration_of("okazjonalny"))
        .expect("point the occasional lease to the declaration");
}

/// Umowa najmu: powstaje jako projekt, z podpisu nie wraca, zawarta ma podpisany egzemplarz
#[test]
fn lease_states() {
    let room = new_room();
    open_rental(&room);
    add_rental_parties(&room);
    add_flat(&room, &flat("lokal"));

    expect::refused(
        room.execute(
            "INSERT INTO najem_umowy_najmu (id, rodzaj, nieruchomosc_id, poczatek, forma, stan,
              czynsz, kaucja, koniec_powod)
            VALUES ('od-razu', 'zwykly', 'lokal', '2026-12-01', 'pisemna', 'zakonczona', 1, 0, 'pomyłka')",
            &[],
        ),
        "umowa najmu powstaje jako projekt",
    );
    prepare_lease(&room, &lease("najem", RodzajNajmu::Zwykly));
    lease_to_signing(&room, "najem");
    expect::refused(
        room.execute(
            "UPDATE najem_umowy_najmu SET stan = 'projekt' WHERE id = 'najem'",
            &[],
        ),
        "z podpisu nie wraca do projektu",
    );

    add_received_document(
        &room,
        &received_document("zly-dokument", RodzajDokumentu::Pismo),
    );
    let conclusion_with_letter = OdnotujZawarcieUmowyNajmuParams {
        dokument_id: "zly-dokument".to_string(),
        zawarto: "2026-11-25T12:00:00Z".to_string(),
        id: "najem".to_string(),
    };
    expect::refused(
        najem_umowy::odnotuj_zawarcie_umowy_najmu(&room, &conclusion_with_letter),
        "zawarta umowa najmu wskazuje otrzymany podpisany egzemplarz",
    );
    conclude_lease(&room, "najem");
}

/// Umowa najmu: do podpisu za zgodą człowieka i nie przy stojącym znaczniku wątpliwości
#[test]
fn lease_signing_needs_consent() {
    let room = new_room();
    open_rental(&room);
    add_rental_parties(&room);
    add_flat(&room, &flat("lokal"));
    prepare_lease(&room, &lease("najem", RodzajNajmu::Zwykly));

    let rule = "umowa najmu idzie do podpisu za zatwierdzoną decyzją inna";
    let signing_on = |decision_id: &str| PrzekazUmoweNajmuDoPodpisuParams {
        do_podpisu_decyzja_id: decision_id.to_string(),
        id: "najem".to_string(),
    };
    expect::refused(
        najem_umowy::przekaz_umowe_najmu_do_podpisu(&room, &signing_on("brak")),
        rule,
    );
    approved_decision_about(
        &room,
        "o-innej",
        RodzajDecyzji::Inna,
        "najem_umowy_najmu",
        "inna-umowa",
    );
    expect::refused(
        najem_umowy::przekaz_umowe_najmu_do_podpisu(&room, &signing_on("o-innej")),
        rule,
    );

    approved_decision_about(
        &room,
        "do-podpisu-najem",
        RodzajDecyzji::Inna,
        "najem_umowy_najmu",
        "najem",
    );
    sprawa_metryka::postaw_watpliwosc(
        &room,
        &PostawWatpliwoscParams {
            id: "watpliwosc".to_string(),
            postawil: "tozsamosc-kontrolera".to_string(),
            postawiono: "2026-11-07T09:00:00Z".to_string(),
        },
    )
    .expect("raise the doubt");
    expect::refused(
        najem_umowy::przekaz_umowe_najmu_do_podpisu(&room, &signing_on("do-podpisu-najem")),
        "dopóki stoi znacznik wątpliwości, umowa najmu nie idzie do podpisu",
    );
}

/// Umowa najmu: strony według rodzaju najmu (art. 19a ust. 1 i art. 19f ust. 1 UOPL)
#[test]
fn lease_parties_by_kind() {
    let room = new_room();
    open_rental(&room);
    add_flat(&room, &flat("lokal"));
    add_company(&room, "spolka");
    give_role(&room, "spolka", "wynajmujacy");
    let rule = "umowa najmu ma wynajmującego i najemcę";

    // Without a tenant there is nobody to sign.
    prepare_lease(&room, &lease("zwykly", RodzajNajmu::Zwykly));
    approved_decision_about(
        &room,
        "do-podpisu-zwykly",
        RodzajDecyzji::Inna,
        "najem_umowy_najmu",
        "zwykly",
    );
    expect::refused(
        najem_umowy::przekaz_umowe_najmu_do_podpisu(&room, &lease_for_signing("zwykly")),
        rule,
    );
    add_person(&room, &person_named("najemca", "Jan", "Nowak"));
    give_role(&room, "najemca", "najemca");
    najem_umowy::przekaz_umowe_najmu_do_podpisu(&room, &lease_for_signing("zwykly"))
        .expect("hand the ordinary lease over for signing");

    // An occasional lease has an individual as the landlord, not a company.
    prepare_lease(
        &room,
        &lease_without_deposit("okazjonalny", RodzajNajmu::Okazjonalny),
    );
    add_tenant_declaration(&room, "okazjonalny");
    add_substitute_flat(&room, "okazjonalny");
    approved_decision_about(
        &room,
        "do-podpisu-okazjonalny",
        RodzajDecyzji::Inna,
        "najem_umowy_najmu",
        "okazjonalny",
    );
    expect::refused(
        najem_umowy::przekaz_umowe_najmu_do_podpisu(&room, &lease_for_signing("okazjonalny")),
        rule,
    );

    // An institutional landlord has a tax number and a register entry; the company has both.
    prepare_lease(
        &room,
        &lease_without_deposit("instytucjonalny", RodzajNajmu::Instytucjonalny),
    );
    add_tenant_declaration(&room, "instytucjonalny");
    lease_to_signing(&room, "instytucjonalny");

    add_person(&room, &person("osoba-bez-nip"));
    give_role(&room, "osoba-bez-nip", "wynajmujacy");
    prepare_lease(
        &room,
        &lease_without_deposit("drugi-instytucjonalny", RodzajNajmu::Instytucjonalny),
    );
    add_tenant_declaration(&room, "drugi-instytucjonalny");
    approved_decision_about(
        &room,
        "do-podpisu-drugi",
        RodzajDecyzji::Inna,
        "najem_umowy_najmu",
        "drugi-instytucjonalny",
    );
    let second_for_signing = PrzekazUmoweNajmuDoPodpisuParams {
        do_podpisu_decyzja_id: "do-podpisu-drugi".to_string(),
        id: "drugi-instytucjonalny".to_string(),
    };
    expect::refused(
        najem_umowy::przekaz_umowe_najmu_do_podpisu(&room, &second_for_signing),
        rule,
    );
}

/// Umowa najmu: okazjonalna i instytucjonalna do podpisu tylko z załącznikami (art. 19a ust. 2)
#[test]
fn lease_signing_needs_annexes() {
    let room = new_room();
    open_rental(&room);
    add_rental_parties(&room);
    add_flat(&room, &flat("lokal"));
    let rule =
        "umowa najmu okazjonalnego i instytucjonalnego idzie do podpisu z oświadczeniem najemcy";

    prepare_lease(
        &room,
        &lease_without_deposit("najem", RodzajNajmu::Okazjonalny),
    );
    approved_decision_about(
        &room,
        "do-podpisu-najem",
        RodzajDecyzji::Inna,
        "najem_umowy_najmu",
        "najem",
    );
    let signing = lease_for_signing("najem");
    expect::refused(
        najem_umowy::przekaz_umowe_najmu_do_podpisu(&room, &signing),
        rule,
    );

    add_tenant_declaration(&room, "najem");
    expect::refused(
        najem_umowy::przekaz_umowe_najmu_do_podpisu(&room, &signing),
        rule,
    );

    // A substitute flat without its owner's consent, then one the tenant can no longer use.
    najem_umowy::wpisz_lokal_zastepczy(
        &room,
        &WpiszLokalZastepczyParams {
            najem_id: "najem".to_string(),
            adres: "ul. Długa 1".to_string(),
            wskazano: "2026-11-20".to_string(),
        },
    )
    .expect("write the substitute flat without consent");
    expect::refused(
        najem_umowy::przekaz_umowe_najmu_do_podpisu(&room, &signing),
        rule,
    );
    let substitute_id = add_substitute_flat(&room, "najem");
    najem_umowy::odnotuj_utrate_lokalu_zastepczego(
        &room,
        &OdnotujUtrateLokaluZastepczegoParams {
            niemozliwy_od: "2026-11-22".to_string(),
            id: substitute_id,
        },
    )
    .expect("record the substitute flat lost");
    expect::refused(
        najem_umowy::przekaz_umowe_najmu_do_podpisu(&room, &signing),
        rule,
    );

    add_substitute_flat(&room, "najem");
    najem_umowy::przekaz_umowe_najmu_do_podpisu(&room, &signing)
        .expect("hand the lease over for signing");
}

/// The default parameters of a change of the draft lease „najem” without a deposit.
fn lease_change() -> ZmienProjektUmowyNajmuParams {
    ZmienProjektUmowyNajmuParams {
        poczatek: "2026-12-01".to_string(),
        koniec: Some("2028-11-30".to_string()),
        forma: FormaNajmu::Pisemna,
        projekt_dokument_id: Some("projekt-najem".to_string()),
        czynsz: 300_000,
        termin_platnosci: None,
        kaucja: 0,
        zwierzeta: None,
        podnajem: None,
        zmiany_w_lokalu: None,
        naprawy_podzial: None,
        wypowiedzenie_przypadki: None,
        id: "najem".to_string(),
    }
}

/// Umowa najmu: po wyjściu z projektu treść stoi, a zgłoszenie i świadectwo wpisuje się raz
#[test]
fn lease_frozen_after_draft() {
    let room = new_room();
    open_rental(&room);
    add_rental_parties(&room);
    add_flat(&room, &flat("lokal"));
    prepare_lease(
        &room,
        &lease_without_deposit("najem", RodzajNajmu::Okazjonalny),
    );
    najem_umowy::zmien_projekt_umowy_najmu(
        &room,
        &ZmienProjektUmowyNajmuParams {
            czynsz: 310_000,
            ..lease_change()
        },
    )
    .expect("change the draft lease");
    add_tenant_declaration(&room, "najem");
    add_substitute_flat(&room, "najem");
    lease_to_signing(&room, "najem");

    let rule = "treść umowy najmu zmienia się tylko w projekcie";
    let higher_rent = ZmienProjektUmowyNajmuParams {
        czynsz: 320_000,
        ..lease_change()
    };
    expect::refused(
        najem_umowy::zmien_projekt_umowy_najmu(&room, &higher_rent),
        rule,
    );
    conclude_lease(&room, "najem");
    expect::refused(
        najem_umowy::zmien_projekt_umowy_najmu(&room, &higher_rent),
        rule,
    );

    let certificate_on = |swiadectwo_przekazano: &str| OdnotujPrzekazanieSwiadectwaParams {
        swiadectwo_przekazano: swiadectwo_przekazano.to_string(),
        id: "najem".to_string(),
    };
    najem_umowy::odnotuj_przekazanie_swiadectwa(&room, &certificate_on("2026-11-25"))
        .expect("record the energy certificate handed over");
    expect::refused(
        najem_umowy::odnotuj_przekazanie_swiadectwa(&room, &certificate_on("2026-11-26")),
        rule,
    );
    add_received_document(
        &room,
        &received_document("zgloszenie", RodzajDokumentu::Pismo),
    );
    let notification_on = |zgloszono_us: &str| OdnotujZgloszenieNajmuDoUrzeduParams {
        zgloszono_us: zgloszono_us.to_string(),
        zgloszenie_us_dokument_id: "zgloszenie".to_string(),
        id: "najem".to_string(),
    };
    najem_umowy::odnotuj_zgloszenie_najmu_do_urzedu(&room, &notification_on("2026-12-10"))
        .expect("record the tax office notification");
    expect::refused(
        najem_umowy::odnotuj_zgloszenie_najmu_do_urzedu(&room, &notification_on("2026-12-11")),
        rule,
    );
}

/// Umowa najmu: zgłoszenie do urzędu skarbowego jest tylko przy najmie okazjonalnym
#[test]
fn tax_office_notification_only_occasional() {
    let room = new_room();
    open_rental(&room);
    add_rental_parties(&room);
    add_flat(&room, &flat("lokal"));
    concluded_lease(&room, &lease("najem", RodzajNajmu::Zwykly));
    add_received_document(
        &room,
        &received_document("zgloszenie", RodzajDokumentu::Pismo),
    );

    let notification = OdnotujZgloszenieNajmuDoUrzeduParams {
        zgloszono_us: "2026-12-10".to_string(),
        zgloszenie_us_dokument_id: "zgloszenie".to_string(),
        id: "najem".to_string(),
    };
    expect::refused(
        najem_umowy::odnotuj_zgloszenie_najmu_do_urzedu(&room, &notification),
        "zgloszenie_us_przy_okazjonalnym",
    );
}

/// Etapy: oświadczenie pomija się tylko przy zwykłym, zgłoszenie tylko przy innym niż okazjonalny
#[test]
fn stages_skipped_by_lease_kind() {
    let room = new_room();
    open_rental(&room);
    add_rental_parties(&room);
    add_flat(&room, &flat("lokal"));
    move_to(&room, &["projekt_umowy"]);

    let rule = "etap oświadczenia najemcy pomija się tylko przy najmie zwykłym";
    refused_stage(&room, rule, "podpisanie_umowy");
    prepare_lease(
        &room,
        &lease_without_deposit("najem", RodzajNajmu::Okazjonalny),
    );
    refused_stage(&room, rule, "podpisanie_umowy");
    move_to(&room, &["oswiadczenie_najemcy"]);
    add_tenant_declaration(&room, "najem");
    add_substitute_flat(&room, "najem");
    lease_to_signing(&room, "najem");
    conclude_lease(&room, "najem");
    move_to(&room, &["podpisanie_umowy", "wydanie_lokalu"]);
    signed_protocol(&room, RodzajProtokolu::Wydanie, "2026-11-30T10:00:00Z");

    let rule = "przy najmie okazjonalnym sprawa idzie z wydania lokalu przez zgłoszenie najmu";
    refused_stage(&room, rule, "trwanie_najmu");
    refused_stage(&room, rule, "po_najmie");
    move_to(&room, &["zgloszenie_najmu", "trwanie_najmu"]);
}

/// Opłaty: dopisuje się je do projektu, a przy zwykłym tylko niezależne od właściciela
#[test]
fn lease_fees() {
    let room = new_room();
    open_rental(&room);
    add_flat(&room, &flat("lokal"));
    prepare_lease(&room, &lease("najem", RodzajNajmu::Zwykly));
    let fee = |rodzaj: RodzajOplaty| DopiszOplateNajmuParams {
        najem_id: "najem".to_string(),
        rodzaj,
        sposob: "zaliczka miesięczna".to_string(),
        umowa_z_dostawca_ma: UmowaZDostawcaMa::Wynajmujacy,
    };

    expect::refused(
        najem_umowy::dopisz_oplate_najmu(&room, &fee(RodzajOplaty::Internet)),
        "przy najmie zwykłym wynajmujący pobiera obok czynszu tylko opłaty niezależne od właściciela",
    );
    add_fee(&room, &fee(RodzajOplaty::Cieplo));
    add_fee(
        &room,
        &DopiszOplateNajmuParams {
            umowa_z_dostawca_ma: UmowaZDostawcaMa::Najemca,
            ..fee(RodzajOplaty::Internet)
        },
    );
    expect::refused(
        room.execute("UPDATE najem_umowy_oplaty SET sposob = 'inny'", &[]),
        "opłaty umowy najmu się nie zmienia",
    );

    // An occasional lease may say otherwise (art. 19c ust. 1 UOPL).
    prepare_lease(
        &room,
        &lease_without_deposit("okazjonalny", RodzajNajmu::Okazjonalny),
    );
    add_fee(
        &room,
        &DopiszOplateNajmuParams {
            najem_id: "okazjonalny".to_string(),
            ..fee(RodzajOplaty::Internet)
        },
    );

    approved_decision_about(
        &room,
        "do-podpisu-najem",
        RodzajDecyzji::Inna,
        "najem_umowy_najmu",
        "najem",
    );
    add_person(&room, &person("wynajmujaca"));
    add_person(&room, &person_named("najemca", "Jan", "Nowak"));
    give_role(&room, "wynajmujaca", "wynajmujacy");
    give_role(&room, "najemca", "najemca");
    najem_umowy::przekaz_umowe_najmu_do_podpisu(&room, &lease_for_signing("najem"))
        .expect("hand the lease over for signing");
    expect::refused(
        najem_umowy::dopisz_oplate_najmu(&room, &fee(RodzajOplaty::Gaz)),
        "opłatę dopisuje się do projektu umowy najmu",
    );
}

/// Zamieszkujący i lokal zastępczy: daty i zgoda wpisuje się raz, lokal tylko przy okazjonalnym
#[test]
fn household_and_substitute_flat() {
    let room = new_room();
    open_rental(&room);
    add_rental_parties(&room);
    add_flat(&room, &flat("lokal"));
    concluded_lease(&room, &lease("najem", RodzajNajmu::Zwykly));

    najem_umowy::wpisz_zamieszkujacego(
        &room,
        &WpiszZamieszkujacegoParams {
            najem_id: "najem".to_string(),
            uczestnik_id: "najemca".to_string(),
            od: "2026-12-01".to_string(),
        },
    )
    .expect("write the person living in the flat");
    let move_out_on = |day: &str| OdnotujWyprowadzkeZamieszkujacegoParams {
        r#do: day.to_string(),
        id: 1,
    };
    expect::refused(
        najem_umowy::odnotuj_wyprowadzke_zamieszkujacego(&room, &move_out_on("2026-11-30")),
        "wyprowadzka_nie_przed_zamieszkaniem",
    );
    najem_umowy::odnotuj_wyprowadzke_zamieszkujacego(&room, &move_out_on("2027-03-31"))
        .expect("record the move-out");
    expect::refused(
        najem_umowy::odnotuj_wyprowadzke_zamieszkujacego(&room, &move_out_on("2027-04-30")),
        "osoby zamieszkującej się nie zmienia",
    );

    let substitute_of = |lease_id: &str| WpiszLokalZastepczyParams {
        najem_id: lease_id.to_string(),
        adres: "ul. Lipowa 4".to_string(),
        wskazano: "2026-11-20".to_string(),
    };
    expect::refused(
        najem_umowy::wpisz_lokal_zastepczy(&room, &substitute_of("najem")),
        "lokal zastępczy wskazuje się tylko przy najmie okazjonalnym",
    );
    prepare_lease(
        &room,
        &lease_without_deposit("okazjonalny", RodzajNajmu::Okazjonalny),
    );
    najem_umowy::wpisz_lokal_zastepczy(&room, &substitute_of("okazjonalny"))
        .expect("write the substitute flat");
    add_received_document(&room, &received_document("pismo", RodzajDokumentu::Pismo));
    let consent_in_letter = WskazZgodeNaLokalZastepczyParams {
        zgoda_dokument_id: "pismo".to_string(),
        id: 1,
    };
    expect::refused(
        najem_umowy::wskaz_zgode_na_lokal_zastepczy(&room, &consent_in_letter),
        "zgoda na lokal zastępczy wskazuje otrzymane oświadczenie",
    );
    expect::refused(
        room.execute(
            "INSERT INTO najem_umowy_lokale_zastepcze (najem_id, adres, wskazano, niemozliwy_od)
            VALUES ('okazjonalny', 'ul. Długa 1', '2026-11-20', '2026-11-21')",
            &[],
        ),
        "lokal zastępczy powstaje bez zgody i bez utraty",
    );
    let lost_on = |niemozliwy_od: &str| OdnotujUtrateLokaluZastepczegoParams {
        niemozliwy_od: niemozliwy_od.to_string(),
        id: 1,
    };
    najem_umowy::odnotuj_utrate_lokalu_zastepczego(&room, &lost_on("2027-01-10"))
        .expect("record the substitute flat lost");
    expect::refused(
        najem_umowy::odnotuj_utrate_lokalu_zastepczego(&room, &lost_on("2027-01-11")),
        "lokalu zastępczego się nie zmienia",
    );
}

/// Aneks: przy zawartej umowie, za zgodą człowieka, obowiązuje z podpisanym egzemplarzem
#[test]
fn lease_annex() {
    let room = new_room();
    open_rental(&room);
    add_rental_parties(&room);
    add_flat(&room, &flat("lokal"));
    prepare_lease(&room, &lease("najem", RodzajNajmu::Zwykly));

    let annex = PrzygotujAneksNajmuParams {
        id: "aneks".to_string(),
        najem_id: "najem".to_string(),
        opis: "Inny termin płatności".to_string(),
        projekt_dokument_id: None,
    };
    expect::refused(
        najem_umowy::przygotuj_aneks_najmu(&room, &annex),
        "aneks powstaje przy zawartej umowie najmu",
    );
    lease_to_signing(&room, "najem");
    conclude_lease(&room, "najem");
    prepare_annex(&room, "aneks", "Inny termin płatności");

    add_received_document(
        &room,
        &received_document("podpisany-aneks", RodzajDokumentu::Umowa),
    );
    expect::refused(
        najem_umowy::odnotuj_podpisanie_aneksu_najmu(
            &room,
            &annex_signing("aneks", "podpisany-aneks", "2027-01-15"),
        ),
        "aneks obowiązuje z podpisanym egzemplarzem i obowiązującą zgodą człowieka",
    );
    let consent_without_decision = WskazZgodeNaAneksNajmuParams {
        decyzja_id: "brak".to_string(),
        id: "aneks".to_string(),
    };
    expect::refused(
        najem_umowy::wskaz_zgode_na_aneks_najmu(&room, &consent_without_decision),
        "aneks idzie do stron za zatwierdzoną decyzją inna",
    );
    approved_decision_about(
        &room,
        "zgoda-aneks",
        RodzajDecyzji::Inna,
        "najem_umowy_aneksy",
        "aneks",
    );
    consent_to_annex(&room, "aneks", "zgoda-aneks");
    expect::refused(
        room.execute(
            "UPDATE najem_umowy_aneksy SET opis = 'Inna treść' WHERE id = 'aneks'",
            &[],
        ),
        "treść aneksu stoi od zgody człowieka",
    );

    add_received_document(&room, &received_document("pismo", RodzajDokumentu::Pismo));
    expect::refused(
        najem_umowy::odnotuj_podpisanie_aneksu_najmu(
            &room,
            &annex_signing("aneks", "pismo", "2027-01-15"),
        ),
        "aneks obowiązuje z podpisanym egzemplarzem",
    );
    najem_umowy::odnotuj_podpisanie_aneksu_najmu(
        &room,
        &annex_signing("aneks", "podpisany-aneks", "2027-01-15"),
    )
    .expect("record the annex signed");
}

/// Koniec najmu: wypowiedzenie na piśmie z przyczyną wychodzi tylko za zgodą człowieka
#[test]
fn lease_notice_with_consent() {
    let room = new_room();
    open_rental(&room);
    add_rental_parties(&room);
    add_flat(&room, &flat("lokal"));
    concluded_lease(&room, &lease("najem", RodzajNajmu::Zwykly));
    let end = WpiszKoniecNajmuParams {
        koniec_najmu: Some("2027-04-30".to_string()),
        ..lease_end("koniec", "najem", SposobKonca::WypowiedzenieWynajmujacego)
    };

    expect::refused(
        najem_umowy::wpisz_koniec_najmu(&room, &end),
        "wypowiedzenie_wynajmujacego_na_pismie_z_przyczyna",
    );

    letter_to_tenant(&room, "wyslane-bez-zgody", "wypowiedzenie");
    send(&room, &sending("wyslane-bez-zgody"));
    let notice_in = |letter_id: &str| WpiszKoniecNajmuParams {
        wysylka_id: Some(letter_id.to_string()),
        przyczyna: Some("zaległość za trzy okresy".to_string()),
        ..end.clone()
    };
    expect::refused(
        najem_umowy::wpisz_koniec_najmu(&room, &notice_in("wyslane-bez-zgody")),
        "koniec najmu wskazuje pismo, które jeszcze nie wyszło",
    );

    letter_to_tenant(&room, "wypowiedzenie", "wypowiedzenie");
    end_lease(&room, &notice_in("wypowiedzenie"));
    let sent_without_consent = OdnotujWyslanieParams {
        wyslano: "2027-03-01T10:00:00Z".to_string(),
        ..sending("wypowiedzenie")
    };
    expect::refused(
        sprawa_korespondencja::odnotuj_wyslanie(&room, &sent_without_consent),
        "wypowiedzenie i żądanie opróżnienia lokalu wychodzą przez czeka_na_zgode",
    );
    send_with_consent(&room, "wypowiedzenie", "2027-02-25T10:00:00Z");
    let delivered_on = |doreczono: &str| OdnotujDoreczenieKoncaNajmuParams {
        doreczono: doreczono.to_string(),
        id: "koniec".to_string(),
    };
    najem_umowy::odnotuj_doreczenie_konca_najmu(&room, &delivered_on("2027-03-03"))
        .expect("record the notice delivered");
    expect::refused(
        najem_umowy::odnotuj_doreczenie_konca_najmu(&room, &delivered_on("2027-03-04")),
        "końca najmu się nie zmienia",
    );
}

/// Koniec najmu: po śmierci najemcy dzień końca przychodzi z decyzją człowieka (art. 691 KC)
#[test]
fn tenant_death_needs_decision() {
    let room = new_room();
    open_rental(&room);
    add_rental_parties(&room);
    add_flat(&room, &flat("lokal"));
    concluded_lease(&room, &lease("najem", RodzajNajmu::Zwykly));
    end_lease(
        &room,
        &lease_end("smierc", "najem", SposobKonca::SmiercNajemcy),
    );

    // A writer outside the client can leave out the decision that
    // `odnotuj_decyzje_po_smierci_najemcy` requires. The table refuses the day of the end without
    // one.
    let end_day_without_decision = room.execute(
        "UPDATE najem_umowy_konce
        SET decyzja_id = :decyzja_id,
          koniec_najmu = :koniec_najmu
        WHERE id = :id",
        &[
            ("decyzja_id", SqlValue::Null),
            ("koniec_najmu", text("2027-02-28")),
            ("id", text("smierc")),
        ],
    );
    expect::refused(end_day_without_decision, "smierc_najemcy_z_decyzja");
    let decision_of = |decision_id: &str| OdnotujDecyzjePoSmierciNajemcyParams {
        decyzja_id: decision_id.to_string(),
        koniec_najmu: Some("2027-02-28".to_string()),
        id: "smierc".to_string(),
    };
    expect::refused(
        najem_umowy::odnotuj_decyzje_po_smierci_najemcy(&room, &decision_of("brak")),
        "koniec najmu wskazuje zatwierdzoną decyzję inna",
    );
    approved_decision_about(
        &room,
        "decyzja-smierc",
        RodzajDecyzji::Inna,
        "najem_umowy_konce",
        "smierc",
    );
    najem_umowy::odnotuj_decyzje_po_smierci_najemcy(&room, &decision_of("decyzja-smierc"))
        .expect("record the decision after the tenant's death");
    expect::refused(
        najem_umowy::wpisz_koniec_najmu(
            &room,
            &lease_end("porozumienie", "najem", SposobKonca::Porozumienie),
        ),
        "koniec_najmu_znany",
    );
}

/// The parameters of the delivery of the eviction demand of the end `end_id` on 2028-12-02 with
/// the term `zadanie_termin`.
fn eviction_demand_delivered(
    end_id: &str,
    zadanie_termin: &str,
) -> OdnotujDoreczenieZadaniaOproznieniaParams {
    OdnotujDoreczenieZadaniaOproznieniaParams {
        zadanie_doreczono: "2028-12-02".to_string(),
        zadanie_termin: zadanie_termin.to_string(),
        id: end_id.to_string(),
    }
}

fn point_to_eviction_demand(room: &MemoryRoom, end_id: &str, letter_id: &str) {
    let params = WskazZadanieOproznieniaParams {
        zadanie_wysylka_id: letter_id.to_string(),
        id: end_id.to_string(),
    };

    najem_umowy::wskaz_zadanie_oproznienia(room, &params)
        .expect("point the end to the eviction demand");
}

/// Żądanie opróżnienia: tylko przy okazjonalnym i instytucjonalnym, z terminem 7 albo 14 dni
#[test]
fn eviction_demand_term() {
    let room = new_room();
    open_rental(&room);
    add_rental_parties(&room);
    add_flat(&room, &flat("lokal"));
    concluded_lease(&room, &lease("zwykly", RodzajNajmu::Zwykly));
    concluded_lease(
        &room,
        &lease_without_deposit("okazjonalny", RodzajNajmu::Okazjonalny),
    );
    let rule = "żądanie opróżnienia jest przy najmie okazjonalnym";

    for lease_id in ["zwykly", "okazjonalny"] {
        end_lease(
            &room,
            &WpiszKoniecNajmuParams {
                koniec_najmu: Some("2028-11-30".to_string()),
                ..lease_end(
                    &format!("koniec-{lease_id}"),
                    lease_id,
                    SposobKonca::UplywCzasu,
                )
            },
        );
        letter_to_tenant(&room, &format!("zadanie-{lease_id}"), "zadanie-oproznienia");
    }

    let demand_of_ordinary = WskazZadanieOproznieniaParams {
        zadanie_wysylka_id: "zadanie-zwykly".to_string(),
        id: "koniec-zwykly".to_string(),
    };
    expect::refused(
        najem_umowy::wskaz_zadanie_oproznienia(&room, &demand_of_ordinary),
        rule,
    );

    point_to_eviction_demand(&room, "koniec-okazjonalny", "zadanie-okazjonalny");
    send_with_consent(&room, "zadanie-okazjonalny", "2028-12-01T10:00:00Z");
    expect::refused(
        najem_umowy::odnotuj_doreczenie_zadania_oproznienia(
            &room,
            &eviction_demand_delivered("koniec-okazjonalny", "2028-12-08"),
        ),
        rule,
    );
    najem_umowy::odnotuj_doreczenie_zadania_oproznienia(
        &room,
        &eviction_demand_delivered("koniec-okazjonalny", "2028-12-09"),
    )
    .expect("record the eviction demand delivered");
}

/// Żądanie opróżnienia przy najmie instytucjonalnym ma termin co najmniej 14 dni
#[test]
fn eviction_demand_institutional() {
    let room = new_room();
    open_rental(&room);
    add_flat(&room, &flat("lokal"));
    add_company(&room, "spolka");
    add_person(&room, &person_named("najemca", "Jan", "Nowak"));
    give_role(&room, "spolka", "wynajmujacy");
    give_role(&room, "najemca", "najemca");
    concluded_lease(
        &room,
        &lease_without_deposit("najem", RodzajNajmu::Instytucjonalny),
    );
    end_lease(
        &room,
        &WpiszKoniecNajmuParams {
            koniec_najmu: Some("2028-11-30".to_string()),
            ..lease_end("koniec", "najem", SposobKonca::UplywCzasu)
        },
    );
    letter_to_tenant(&room, "zadanie", "zadanie-oproznienia");
    point_to_eviction_demand(&room, "koniec", "zadanie");
    send_with_consent(&room, "zadanie", "2028-12-01T10:00:00Z");

    expect::refused(
        najem_umowy::odnotuj_doreczenie_zadania_oproznienia(
            &room,
            &eviction_demand_delivered("koniec", "2028-12-15"),
        ),
        "instytucjonalnym (co najmniej 14 dni)",
    );
    najem_umowy::odnotuj_doreczenie_zadania_oproznienia(
        &room,
        &eviction_demand_delivered("koniec", "2028-12-16"),
    )
    .expect("record the eviction demand delivered");
}

// Rent ----------------------------------------------------------------------------------------------

/// A concluded ordinary lease under a management contract with the money flow `przeplyw` (the
/// Python default is „przez_biuro”).
fn managed_lease(room: &MemoryRoom, przeplyw: Przeplyw) {
    open_rental(room);
    add_rental_parties(room);
    add_flat(room, &flat("lokal"));
    management(room, przeplyw);
    concluded_lease(room, &lease("najem", RodzajNajmu::Zwykly));
}

/// Wpłata: na rachunek biura tylko przy przepływie przez biuro, wprost na rachunek dopuszczony
#[test]
fn rent_payment_account() {
    let room = new_room();
    managed_lease(&room, Przeplyw::Wprost);
    add_received_document(
        &room,
        &received_document("potwierdzenie", RodzajDokumentu::PotwierdzenieBankowe),
    );
    let payment = |payment_id: &str, na_rachunek: NaRachunek, rachunek_id: Option<&str>| {
        WpiszWplateCzynszuParams {
            na_rachunek,
            rachunek_id: rachunek_id.map(str::to_string),
            potwierdzenie: PotwierdzenieWplaty::OdWynajmujacego,
            dokument_id: "potwierdzenie".to_string(),
            ..rent_payment(payment_id)
        }
    };

    expect::refused(
        najem_czynsz::wpisz_wplate_czynszu(&room, &payment("do-biura", NaRachunek::Biura, None)),
        "na rachunek biura czynsz wpływa tylko przy przepływie przez biuro",
    );
    expect::refused(
        najem_czynsz::wpisz_wplate_czynszu(
            &room,
            &payment("bez-rachunku", NaRachunek::Wynajmujacego, None),
        ),
        "rachunek_wynajmujacego_wskazany",
    );
    add_account(
        &room,
        &WpiszRachunekParams {
            cel: CelRachunku::Czynsz,
            ..account("niedopuszczony", "wlascicielka")
        },
    );
    expect::refused(
        najem_czynsz::wpisz_wplate_czynszu(
            &room,
            &payment(
                "na-niedopuszczony",
                NaRachunek::Wynajmujacego,
                Some("niedopuszczony"),
            ),
        ),
        "wpłata idzie na rachunek dopuszczony decyzją i niezastąpiony (R4)",
    );
    admitted_account_for(
        &room,
        "rachunek-czynszu",
        "wlascicielka",
        CelRachunku::Czynsz,
        "Anna Kowalska",
    );
    najem_czynsz::wpisz_wplate_czynszu(
        &room,
        &payment(
            "wprost",
            NaRachunek::Wynajmujacego,
            Some("rachunek-czynszu"),
        ),
    )
    .expect("write the payment to the landlord's account");
    expect::refused(
        room.execute("UPDATE najem_czynsz_wplaty SET kwota = 1", &[]),
        "wpłaty się nie zmienia",
    );
}

/// Należność i rozliczenie: poprawka przed wpłatą, rozliczenie w granicach wpłaty i należności
#[test]
fn rent_due_and_settlement() {
    let room = new_room();
    managed_lease(&room, Przeplyw::PrzezBiuro);
    add_rent_due(&room);
    let correction = |kwota: i64| PoprawNaleznoscParams {
        tytul: TytulNaleznosci::Czynsz,
        okres: "2026-12".to_string(),
        kwota,
        termin: "2026-12-10".to_string(),
        id: "czynsz-grudzien".to_string(),
    };
    najem_czynsz::popraw_naleznosc(&room, &correction(290_000)).expect("correct the rent due");
    rent_paid_to_office(&room, "wplata-grudzien", 200_000);

    let rule = "rozliczenie nie przekracza kwoty wpłaty ani kwoty należności";
    let settlement_of = |payment_id: &str, kwota: i64| RozliczWplateParams {
        wplata_id: payment_id.to_string(),
        naleznosc_id: "czynsz-grudzien".to_string(),
        kwota,
    };
    expect::refused(
        najem_czynsz::rozlicz_wplate(&room, &settlement_of("wplata-grudzien", 200_001)),
        rule,
    );
    settle_payment(&room, "wplata-grudzien", 200_000);
    rent_paid_to_office(&room, "druga-wplata", 200_000);
    expect::refused(
        najem_czynsz::rozlicz_wplate(&room, &settlement_of("druga-wplata", 90_001)),
        rule,
    );
    settle_payment(&room, "druga-wplata", 90_000);

    let balances = najem_czynsz::saldo_naleznosci(&room).expect("read the balance");
    let balance = &balances[0];
    assert!(
        balance.zaplacono == 290_000 && balance.zostalo == 0,
        "{balance:?}"
    );
    expect::refused(
        najem_czynsz::popraw_naleznosc(&room, &correction(300_000)),
        "należność zmienia się tylko, dopóki nie pokrywa jej żadna wpłata",
    );
    expect::refused(
        room.execute("UPDATE najem_czynsz_rozliczenia SET kwota = 1", &[]),
        "rozliczenia wpłaty się nie zmienia",
    );
}

/// Przekazanie czynszu (T13): jedna wpłata na rachunek biura, do wynajmującego, raz
#[test]
fn rent_transfer_to_owner() {
    let room = new_room();
    managed_lease(&room, Przeplyw::PrzezBiuro);
    rent_paid_to_office(&room, "wplata-grudzien", 300_000);
    admitted_account_for(
        &room,
        "rachunek-wlascicielki",
        "wlascicielka",
        CelRachunku::WyplataDlaWlasciciela,
        "Anna Kowalska",
    );
    admitted_account_for(
        &room,
        "rachunek-najemcy",
        "najemca",
        CelRachunku::Inny,
        "Jan Nowak",
    );
    let rule = "przekazanie czynszu wskazuje jedną wpłatę na rachunek biura";
    let to_owner = |payout_id: &str, kwota: i64| {
        rent_transfer(payout_id, kwota, "wlascicielka", "rachunek-wlascicielki")
    };

    expect::refused(
        sprawa_wyplaty::przygotuj_wyplate(&room, &to_owner("ponad-wplate", 300_001)),
        rule,
    );
    expect::refused(
        sprawa_wyplaty::przygotuj_wyplate(
            &room,
            &rent_transfer("do-najemcy", 270_000, "najemca", "rachunek-najemcy"),
        ),
        rule,
    );
    let without_payment = PrzygotujWyplateParams {
        podstawa_id: "brak".to_string(),
        ..to_owner("bez-wplaty", 270_000)
    };
    expect::refused(
        sprawa_wyplaty::przygotuj_wyplate(&room, &without_payment),
        rule,
    );

    prepare_payout(&room, &to_owner("przekazanie", 270_000));
    expect::refused(
        sprawa_wyplaty::przygotuj_wyplate(&room, &to_owner("drugie", 1_000)),
        rule,
    );
    sprawa_wyplaty::anuluj_wyplate(
        &room,
        &AnulujWyplateParams {
            id: "przekazanie".to_string(),
        },
    )
    .expect("cancel the payout");
    prepare_payout(&room, &to_owner("ponowne", 270_000));
}

/// Przekazanie czynszu (T13) wymaga przepływu przez biuro według bieżących warunków
#[test]
fn rent_transfer_needs_flow_through_office() {
    let room = new_room();
    managed_lease(&room, Przeplyw::PrzezBiuro);
    rent_paid_to_office(&room, "wplata-grudzien", 300_000);
    add_management_annex(&room, "2026-12-20T00:00:00Z");
    najem_umowy::wpisz_warunki_zarzadzania(
        &room,
        &WpiszWarunkiZarzadzaniaParams {
            przeplyw: Przeplyw::Wprost,
            aneks_zdarzenie_id: Some("aneks".to_string()),
            ..management_terms("2026-12-20T00:00:00Z")
        },
    )
    .expect("write the terms of the annex");
    admitted_account_for(
        &room,
        "rachunek-wlascicielki",
        "wlascicielka",
        CelRachunku::WyplataDlaWlasciciela,
        "Anna Kowalska",
    );

    let transfer = rent_transfer(
        "przekazanie",
        270_000,
        "wlascicielka",
        "rachunek-wlascicielki",
    );
    expect::refused(
        sprawa_wyplaty::przygotuj_wyplate(&room, &transfer),
        "przekazanie czynszu wskazuje jedną wpłatę na rachunek biura przy przepływie przez biuro",
    );
}

/// The default parameters of an agreed change of the rent of the lease „najem” from `stara` to
/// `nowa`, from 2027-06-01.
fn rent_change(stara: i64, nowa: i64) -> WpiszZmianeCzynszuParams {
    WpiszZmianeCzynszuParams {
        najem_id: "najem".to_string(),
        stara,
        nowa,
        pismo_wysylka_id: None,
        doreczono: None,
        obowiazuje_od: "2027-06-01".to_string(),
        podstawa: "porozumienie stron".to_string(),
        aneks_id: None,
    }
}

/// Zmiana czynszu: od bieżącego czynszu; z porozumienia wskazuje obowiązujący aneks
#[test]
fn rent_change_from_current_rent() {
    let room = new_room();
    managed_lease(&room, Przeplyw::PrzezBiuro);
    let rule = "zmiana czynszu zaczyna się od obowiązującego czynszu";

    expect::refused(
        najem_czynsz::wpisz_zmiane_czynszu(&room, &rent_change(310_000, 290_000)),
        rule,
    );
    expect::refused(
        najem_czynsz::wpisz_zmiane_czynszu(&room, &rent_change(300_000, 300_000)),
        "zmiana_zmienia_kwote",
    );

    prepare_annex(&room, "aneks", "Wyższy czynsz od czerwca");
    let change_by_annex = WpiszZmianeCzynszuParams {
        aneks_id: Some("aneks".to_string()),
        ..rent_change(300_000, 330_000)
    };
    expect::refused(
        najem_czynsz::wpisz_zmiane_czynszu(&room, &change_by_annex),
        rule,
    );
    approved_decision_about(
        &room,
        "zgoda-aneks",
        RodzajDecyzji::Inna,
        "najem_umowy_aneksy",
        "aneks",
    );
    consent_to_annex(&room, "aneks", "zgoda-aneks");
    add_received_document(
        &room,
        &received_document("podpisany-aneks", RodzajDokumentu::Umowa),
    );
    najem_umowy::odnotuj_podpisanie_aneksu_najmu(
        &room,
        &annex_signing("aneks", "podpisany-aneks", "2027-05-20"),
    )
    .expect("record the annex signed");
    najem_czynsz::wpisz_zmiane_czynszu(&room, &change_by_annex)
        .expect("write the change by the annex");

    // An agreed reduction later starts from the new rent.
    expect::refused(
        najem_czynsz::wpisz_zmiane_czynszu(&room, &rent_change(300_000, 280_000)),
        rule,
    );
    najem_czynsz::wpisz_zmiane_czynszu(&room, &rent_change(330_000, 280_000))
        .expect("write the reduction");
    expect::refused(
        room.execute("UPDATE najem_czynsz_zmiany SET podstawa = 'inna'", &[]),
        "zmiany czynszu się nie zmienia",
    );
}

/// Podwyżka przy najmie zwykłym: wypowiedzenie na piśmie, 3 miesiące, od 1. dnia, co 6 miesięcy
#[test]
fn rent_increase_ordinary_lease() {
    let room = new_room();
    managed_lease(&room, Przeplyw::PrzezBiuro);
    letter_to_tenant(&room, "podwyzka", "wypowiedzenie-wysokosci-czynszu");
    let rule = "podwyżka przy najmie zwykłym obowiązuje od pierwszego dnia miesiąca";
    let increase = WpiszZmianeCzynszuParams {
        najem_id: "najem".to_string(),
        stara: 300_000,
        nowa: 320_000,
        pismo_wysylka_id: Some("podwyzka".to_string()),
        doreczono: None,
        obowiazuje_od: "2027-05-01".to_string(),
        podstawa: "art. 8a UOPL".to_string(),
        aneks_id: None,
    };
    let delivered = |params: &WpiszZmianeCzynszuParams, doreczono: &str, obowiazuje_od: &str| {
        WpiszZmianeCzynszuParams {
            doreczono: Some(doreczono.to_string()),
            obowiazuje_od: obowiazuje_od.to_string(),
            ..params.clone()
        }
    };

    let without_letter = WpiszZmianeCzynszuParams {
        pismo_wysylka_id: None,
        ..increase.clone()
    };
    expect::refused(
        najem_czynsz::wpisz_zmiane_czynszu(&room, &without_letter),
        rule,
    );
    // Delivered on 1 February: the three months end on 1 May, so the notice works from 1 June.
    expect::refused(
        najem_czynsz::wpisz_zmiane_czynszu(
            &room,
            &delivered(&increase, "2027-02-01", "2027-05-01"),
        ),
        rule,
    );
    expect::refused(
        najem_czynsz::wpisz_zmiane_czynszu(
            &room,
            &delivered(&increase, "2027-02-01", "2027-06-15"),
        ),
        rule,
    );
    najem_czynsz::wpisz_zmiane_czynszu(&room, &delivered(&increase, "2027-02-01", "2027-06-01"))
        .expect("write the increase");

    // The next increase comes six months after the previous one started.
    letter_to_tenant(&room, "druga-podwyzka", "wypowiedzenie-wysokosci-czynszu");
    let second = WpiszZmianeCzynszuParams {
        stara: 320_000,
        nowa: 340_000,
        pismo_wysylka_id: Some("druga-podwyzka".to_string()),
        ..increase.clone()
    };
    expect::refused(
        najem_czynsz::wpisz_zmiane_czynszu(&room, &delivered(&second, "2027-07-01", "2027-11-01")),
        "przy najmie zwykłym czynszu nie podwyższa się częściej niż co 6 miesięcy",
    );
    najem_czynsz::wpisz_zmiane_czynszu(&room, &delivered(&second, "2027-08-01", "2027-12-01"))
        .expect("write the second increase");
}

/// Podwyżka przy najmie okazjonalnym idzie według umowy, bez reguł art. 8a UOPL
#[test]
fn rent_increase_occasional_lease() {
    let room = new_room();
    open_rental(&room);
    add_rental_parties(&room);
    add_flat(&room, &flat("lokal"));
    concluded_lease(
        &room,
        &lease_without_deposit("najem", RodzajNajmu::Okazjonalny),
    );

    najem_czynsz::wpisz_zmiane_czynszu(
        &room,
        &WpiszZmianeCzynszuParams {
            najem_id: "najem".to_string(),
            stara: 300_000,
            nowa: 315_000,
            pismo_wysylka_id: None,
            doreczono: None,
            obowiazuje_od: "2027-01-15".to_string(),
            podstawa: "§ 4 umowy: waloryzacja o wskaźnik inflacji".to_string(),
            aneks_id: None,
        },
    )
    .expect("write the increase by the lease");
}

/// Zaległość: uprzedzenie za zgodą człowieka, dodatkowy termin co najmniej miesiąc
#[test]
fn arrears_warning_and_extra_month() {
    let room = new_room();
    managed_lease(&room, Przeplyw::PrzezBiuro);
    expect::refused(
        room.execute(
            "INSERT INTO najem_czynsz_zaleglosci (id, najem_id, trzy_okresy_od, wynik)
            VALUES ('od-razu', 'najem', '2027-02-11', 'splacona')",
            &[],
        ),
        "zaległość powstaje jako czekająca",
    );
    add_arrears(&room, &arrears("zaleglosc", "najem"));

    letter_to_tenant(&room, "wyslane-bez-zgody", "uprzedzenie-o-wypowiedzeniu");
    send(&room, &sending("wyslane-bez-zgody"));
    let warning_in = |letter_id: &str| WskazUprzedzenieOZaleglosciParams {
        uprzedzenie_wysylka_id: letter_id.to_string(),
        id: "zaleglosc".to_string(),
    };
    expect::refused(
        najem_czynsz::wskaz_uprzedzenie_o_zaleglosci(&room, &warning_in("wyslane-bez-zgody")),
        "zaległość wskazuje uprzedzenie, które jeszcze nie wyszło",
    );

    letter_to_tenant(&room, "uprzedzenie", "uprzedzenie-o-wypowiedzeniu");
    najem_czynsz::wskaz_uprzedzenie_o_zaleglosci(&room, &warning_in("uprzedzenie"))
        .expect("point the arrears to the warning");
    let sent_without_consent = OdnotujWyslanieParams {
        wyslano: "2027-02-25T10:00:00Z".to_string(),
        ..sending("uprzedzenie")
    };
    expect::refused(
        sprawa_korespondencja::odnotuj_wyslanie(&room, &sent_without_consent),
        "uprzedzenie o zamiarze wypowiedzenia wychodzi przez czeka_na_zgode",
    );
    let outcome = |wynik: WynikZaleglosci| OdnotujWynikZaleglosciParams {
        wynik,
        id: "zaleglosc".to_string(),
    };
    expect::refused(
        najem_czynsz::odnotuj_wynik_zaleglosci(&room, &outcome(WynikZaleglosci::Wypowiedzenie)),
        "wypowiedzenie_po_dodatkowym_terminie",
    );
    send_with_consent(&room, "uprzedzenie", "2027-02-25T10:00:00Z");

    let rule = "dodatkowy termin trwa co najmniej miesiąc od doręczenia uprzedzenia";
    let extra_term_until = |day: &str| UstalDodatkowyTerminZaleglosciParams {
        dodatkowy_termin_do: day.to_string(),
        id: "zaleglosc".to_string(),
    };
    expect::refused(
        najem_czynsz::ustal_dodatkowy_termin_zaleglosci(&room, &extra_term_until("2027-03-26")),
        rule,
    );
    deliver(&room, "uprzedzenie", "2027-02-26T10:00:00Z");
    expect::refused(
        najem_czynsz::ustal_dodatkowy_termin_zaleglosci(&room, &extra_term_until("2027-03-25")),
        rule,
    );
    najem_czynsz::ustal_dodatkowy_termin_zaleglosci(&room, &extra_term_until("2027-03-26"))
        .expect("set the extra term");
    najem_czynsz::odnotuj_wynik_zaleglosci(&room, &outcome(WynikZaleglosci::Wypowiedzenie))
        .expect("record the notice after the extra term");
    expect::refused(
        najem_czynsz::odnotuj_wynik_zaleglosci(&room, &outcome(WynikZaleglosci::Splacona)),
        "zaległości się nie zmienia",
    );
}

/// Odczyty liczników się nie zmienia; dane dla gminy tylko przy zarządzaniu lokalem mieszkalnym
#[test]
fn meter_readings_and_municipality_data() {
    let room = new_room();
    open_rental(&room);
    add_rental_parties(&room);
    add_flat(&room, &flat("lokal"));
    add_flat(
        &room,
        &WpiszNieruchomoscParams {
            rodzaj: RodzajNieruchomosci::LokalUzytkowy,
            numer_lokalu: Some("U1".to_string()),
            numer_kw: Some("WR1K/00054321/0".to_string()),
            ..flat("uzytkowy")
        },
    );
    concluded_lease(&room, &lease("najem", RodzajNajmu::Zwykly));
    prepare_lease(
        &room,
        &PrzygotujUmoweNajmuParams {
            nieruchomosc_id: "uzytkowy".to_string(),
            ..lease_without_deposit("uzytkowego", RodzajNajmu::Zwykly)
        },
    );
    let rule = "dane dla gminy przygotowuje się przy zawartej umowie o zarządzanie";

    expect::refused(
        najem_czynsz::przygotuj_dane_dla_gminy(&room, &municipality_data("najem")),
        rule,
    );
    management(&room, Przeplyw::PrzezBiuro);
    expect::refused(
        najem_czynsz::przygotuj_dane_dla_gminy(&room, &municipality_data("uzytkowego")),
        rule,
    );
    najem_czynsz::przygotuj_dane_dla_gminy(&room, &municipality_data("najem"))
        .expect("prepare the data for the municipality");
    let correction = |czynsz: i64| PoprawDaneDlaGminyParams {
        czynsz,
        rok_budowy: None,
        stan_techniczny: None,
        powierzchnia: Some(4820),
        standard: None,
        id: 1,
    };
    najem_czynsz::popraw_dane_dla_gminy(&room, &correction(310_000))
        .expect("correct the data for the municipality");
    let sent_before_handed_in = OdnotujWyslanieDanychDlaGminyParams {
        wyslano: "2027-03-05".to_string(),
        wyslal: "czlowiek".to_string(),
        id: 1,
    };
    expect::refused(
        najem_czynsz::odnotuj_wyslanie_danych_dla_gminy(&room, &sent_before_handed_in),
        "wyslanie_po_oddaniu",
    );
    najem_czynsz::odnotuj_oddanie_danych_dla_gminy(
        &room,
        &OdnotujOddanieDanychDlaGminyParams {
            oddano: "2027-03-01".to_string(),
            id: 1,
        },
    )
    .expect("record the data handed in");
    expect::refused(
        najem_czynsz::popraw_dane_dla_gminy(&room, &correction(320_000)),
        "oddanych danych dla gminy się nie zmienia",
    );

    najem_czynsz::wpisz_odczyt_licznika(
        &room,
        &meter_reading(Licznik::Gaz, "G-77", "1 024 m³", "gaz"),
    )
    .expect("write the meter reading");
    expect::refused(
        room.execute("UPDATE najem_czynsz_odczyty SET stan = '0 m³'", &[]),
        "odczytu licznika się nie zmienia",
    );
}

// Deposit -------------------------------------------------------------------------------------------

/// Kaucja: pobranie w granicy umowy, z czynszem z dnia zawarcia, na rachunek dopuszczony
#[test]
fn deposit_collection() {
    let room = new_room();
    managed_lease(&room, Przeplyw::PrzezBiuro);
    let rule = "pobrana kaucja mieści się w kaucji z umowy najmu";

    let too_high = WpiszPobranieKaucjiParams {
        id: "za-duza".to_string(),
        kwota: 600_001,
        ..deposit()
    };
    expect::refused(najem_kaucja::wpisz_pobranie_kaucji(&room, &too_high), rule);
    let other_rent = WpiszPobranieKaucjiParams {
        id: "inny-czynsz".to_string(),
        czynsz_przy_zawarciu: 290_000,
        ..deposit()
    };
    expect::refused(
        najem_kaucja::wpisz_pobranie_kaucji(&room, &other_rent),
        rule,
    );
    add_account(
        &room,
        &WpiszRachunekParams {
            cel: CelRachunku::Kaucja,
            ..account("niedopuszczony", "wlascicielka")
        },
    );
    let on_account_not_admitted = WpiszPobranieKaucjiParams {
        id: "na-niedopuszczony".to_string(),
        lezy: KaucjaLezy::UWynajmujacego,
        rachunek_id: Some("niedopuszczony".to_string()),
        ..deposit()
    };
    expect::refused(
        najem_kaucja::wpisz_pobranie_kaucji(&room, &on_account_not_admitted),
        "kaucja idzie na rachunek dopuszczony decyzją i niezastąpiony (R4)",
    );
    collect_deposit(&room, &deposit());
    expect::refused(
        room.execute("UPDATE najem_kaucja_pobrania SET kwota = 1", &[]),
        "pobrania kaucji się nie zmienia",
    );
}

/// Kaucja: rozliczenia nie pomija się, gdy kaucję pobrano
#[test]
fn deposit_settlement_stage() {
    let room = new_room();
    managed_lease(&room, Przeplyw::PrzezBiuro);
    move_to(
        &room,
        &["projekt_umowy", "podpisanie_umowy", "wydanie_lokalu"],
    );
    signed_protocol(&room, RodzajProtokolu::Wydanie, "2026-11-30T10:00:00Z");
    move_to(&room, &["trwanie_najmu", "koniec_najmu", "zwrot_lokalu"]);
    signed_protocol(&room, RodzajProtokolu::Zwrot, "2027-08-31T10:00:00Z");
    deposit_at_office(&room);

    refused_stage(
        &room,
        "po zwrocie lokalu z pobraną kaucją przychodzi rozliczenie kaucji",
        "po_najmie",
    );
    move_to(&room, &["rozliczenie_kaucji", "po_najmie"]);
}

/// Kaucja: wykorzystanie na zaległość tylko przy najmie instytucjonalnym (art. 19f ust. 7)
#[test]
fn deposit_used_only_institutional() {
    let room = new_room();
    managed_lease(&room, Przeplyw::PrzezBiuro);
    deposit_at_office(&room);
    add_arrears(&room, &arrears("zaleglosc", "najem"));

    let use_of_ordinary_deposit = WpiszWykorzystanieKaucjiParams {
        pobranie_id: "kaucja".to_string(),
        kwota: 100_000,
        chwila: "2027-02-12".to_string(),
        zaleglosc_id: "zaleglosc".to_string(),
        uzupelnic_do: None,
    };
    expect::refused(
        najem_kaucja::wpisz_wykorzystanie_kaucji(&room, &use_of_ordinary_deposit),
        "kaucję wykorzystuje się na zaległość tego najmu tylko przy najmie instytucjonalnym",
    );

    institutional_deposit_used(&room);
    let topped_up_on = |uzupelniono: &str| OdnotujUzupelnienieKaucjiParams {
        uzupelniono: uzupelniono.to_string(),
        id: 1,
    };
    najem_kaucja::odnotuj_uzupelnienie_kaucji(&room, &topped_up_on("2027-03-01"))
        .expect("record the deposit topped up");
    expect::refused(
        najem_kaucja::odnotuj_uzupelnienie_kaucji(&room, &topped_up_on("2027-03-02")),
        "wykorzystania kaucji się nie zmienia",
    );
}

/// An institutional lease whose deposit covered arrears; the landlord is a company.
fn institutional_deposit_used(room: &MemoryRoom) {
    add_company(room, "spolka");
    prepare_lease(
        room,
        &PrzygotujUmoweNajmuParams {
            kaucja: 600_000,
            ..lease("instytucjonalny", RodzajNajmu::Instytucjonalny)
        },
    );
    add_arrears(
        room,
        &arrears("zaleglosc-instytucjonalna", "instytucjonalny"),
    );
    collect_deposit(
        room,
        &WpiszPobranieKaucjiParams {
            id: "kaucja-instytucjonalna".to_string(),
            najem_id: "instytucjonalny".to_string(),
            ..deposit()
        },
    );

    let use_of_deposit = WpiszWykorzystanieKaucjiParams {
        pobranie_id: "kaucja-instytucjonalna".to_string(),
        kwota: 100_000,
        chwila: "2027-02-12".to_string(),
        zaleglosc_id: "zaleglosc-instytucjonalna".to_string(),
        uzupelnic_do: Some("2027-03-12".to_string()),
    };
    najem_kaucja::wpisz_wykorzystanie_kaucji(room, &use_of_deposit)
        .expect("use the deposit for the arrears");
}

/// Rozliczenie kaucji: waloryzacja nie mniej niż pobrana; zatwierdzenie po zgodzie na potrącenia
#[test]
fn deposit_settlement_rules() {
    let room = new_room();
    managed_lease(&room, Przeplyw::PrzezBiuro);
    deposit_at_office(&room);

    let indexed_below_collected = PrzygotujRozliczenieKaucjiParams {
        kwota_zwaloryzowana: 599_999,
        ..deposit_settlement()
    };
    expect::refused(
        najem_kaucja::przygotuj_rozliczenie_kaucji(&room, &indexed_below_collected),
        "kaucja zwaloryzowana nie jest mniejsza niż pobrana",
    );
    expect::refused(
        room.execute(
            "INSERT INTO najem_kaucja_rozliczenia (id, pobranie_id, oproznienie, czynsz_w_dniu_zwrotu,
              kwota_zwaloryzowana, wyliczenie_sciezka, wyliczenie_skrot, stan)
            VALUES ('od-razu', 'kaucja', '2027-08-31', 320000, 640000, 'w.json', 'sha', 'zatwierdzone')",
            &[],
        ),
        "rozliczenie kaucji powstaje jako projekt",
    );
    prepare_deposit_settlement(&room);
    expect::refused(
        room.execute(
            "UPDATE najem_kaucja_rozliczenia SET kwota_zwaloryzowana = 1",
            &[],
        ),
        "kaucja zwaloryzowana nie jest mniejsza niż pobrana",
    );

    add_deduction(&room, &deduction("bez-zgody"));
    let rule = "rozliczenie kaucji jest zatwierdzone, gdy każde potrącenie ma zatwierdzoną decyzję";
    let approval = ZatwierdzRozliczenieKaucjiParams {
        id: "rozliczenie".to_string(),
    };
    expect::refused(
        najem_kaucja::zatwierdz_rozliczenie_kaucji(&room, &approval),
        rule,
    );
    let consent_without_decision = WskazZgodeNaPotracenieParams {
        decyzja_id: "brak".to_string(),
        id: "bez-zgody".to_string(),
    };
    expect::refused(
        najem_kaucja::wskaz_zgode_na_potracenie(&room, &consent_without_decision),
        "potrącenie wskazuje zatwierdzoną decyzję inna",
    );
    approved_decision_about(
        &room,
        "zgoda-bez-zgody",
        RodzajDecyzji::Inna,
        "najem_kaucja_potracenia",
        "bez-zgody",
    );
    consent_to_deduction(&room, "bez-zgody", "zgoda-bez-zgody");
    add_approved_deduction(
        &room,
        &DopiszPotracenieZKaucjiParams {
            kwota: 600_001,
            ..deduction("ponad-kaucje")
        },
    );
    expect::refused(
        najem_kaucja::zatwierdz_rozliczenie_kaucji(&room, &approval),
        rule,
    );
}

/// Rozliczenie kaucji: projekt, zatwierdzone, wysłane; po zatwierdzeniu stoi z potrąceniami
#[test]
fn deposit_settlement_states() {
    let room = new_room();
    managed_lease(&room, Przeplyw::PrzezBiuro);
    deposit_at_office(&room);
    prepare_deposit_settlement(&room);
    add_approved_deduction(&room, &deduction("potracenie"));
    letter_to_tenant(&room, "rozliczenie-kaucji", "rozliczenie-kaucji");

    expect::refused(
        najem_kaucja::odnotuj_wyslanie_rozliczenia_kaucji(&room, &settlement_sent()),
        "rozliczenie kaucji przechodzi: projekt, zatwierdzone, wysłane",
    );
    approve_deposit_settlement(&room);
    expect::refused(
        room.execute(
            "UPDATE najem_kaucja_rozliczenia SET czynsz_w_dniu_zwrotu = 1",
            &[],
        ),
        "zatwierdzonego rozliczenia kaucji się nie zmienia",
    );
    let late_deduction = DopiszPotracenieZKaucjiParams {
        tytul: "spóźnione".to_string(),
        kwota: 1,
        dokument_id: "podstawa-potracenie".to_string(),
        ..deduction("pozne")
    };
    expect::refused(
        najem_kaucja::dopisz_potracenie_z_kaucji(&room, &late_deduction),
        "potrącenie dopisuje się do projektu rozliczenia kaucji",
    );
    expect::refused(
        room.execute("UPDATE najem_kaucja_potracenia SET kwota = 1", &[]),
        "potrącenia się nie zmienia",
    );
    najem_kaucja::odnotuj_wyslanie_rozliczenia_kaucji(&room, &settlement_sent())
        .expect("record the settlement sent");
}

/// Zwrot kaucji u biura: wypłata według zatwierdzonego rozliczenia, do najemcy, raz
#[test]
fn deposit_return_payout() {
    let room = new_room();
    managed_lease(&room, Przeplyw::PrzezBiuro);
    deposit_at_office(&room);
    prepare_deposit_settlement(&room);
    add_approved_deduction(&room, &deduction("potracenie"));
    admitted_account_for(
        &room,
        "rachunek-najemcy",
        "najemca",
        CelRachunku::ZwrotKaucji,
        "Jan Nowak",
    );
    admitted_account_for(
        &room,
        "rachunek-wlascicielki",
        "wlascicielka",
        CelRachunku::ZwrotKaucji,
        "Anna Kowalska",
    );
    let rule = "zwrot kaucji u biura wskazuje zatwierdzone rozliczenie";
    let to_tenant = |payout_id: &str, kwota: i64| {
        deposit_return(payout_id, kwota, "najemca", "rachunek-najemcy")
    };

    expect::refused(
        sprawa_wyplaty::przygotuj_wyplate(&room, &to_tenant("przed-zatwierdzeniem", 600_000)),
        rule,
    );
    approve_deposit_settlement(&room);
    expect::refused(
        sprawa_wyplaty::przygotuj_wyplate(&room, &to_tenant("zla-kwota", 640_000)),
        rule,
    );
    let to_owner = deposit_return(
        "do-wlascicielki",
        600_000,
        "wlascicielka",
        "rachunek-wlascicielki",
    );
    expect::refused(sprawa_wyplaty::przygotuj_wyplate(&room, &to_owner), rule);

    prepare_payout(&room, &to_tenant("zwrot", 600_000));
    expect::refused(
        sprawa_wyplaty::przygotuj_wyplate(&room, &to_tenant("drugi", 600_000)),
        rule,
    );
}

// Defects -------------------------------------------------------------------------------------------

/// Usterka: spór rozstrzyga decyzja człowieka; zlecenie tylko naprawy obciążającej wynajmującego
#[test]
fn defect_dispute() {
    let room = new_room();
    managed_lease(&room, Przeplyw::PrzezBiuro);
    report_defect(&room, ObciazaUsterka::Spor);

    let landlord_without_decision = UstalKogoObciazaUsterkaParams {
        obciaza: ObciazaUsterka::Wynajmujacy,
        spor_decyzja_id: None,
        id: "usterka".to_string(),
    };
    expect::refused(
        najem_usterki::ustal_kogo_obciaza_usterka(&room, &landlord_without_decision),
        "spór o naprawę rozstrzyga zatwierdzona decyzja inna",
    );
    expect::refused(
        najem_usterki::zlec_naprawe_usterki(&room, &repair()),
        "zlecenie_obciaza_wynajmujacego",
    );
    approved_decision_about(
        &room,
        "spor",
        RodzajDecyzji::Inna,
        "najem_usterki_zgloszenia",
        "usterka",
    );
    najem_usterki::ustal_kogo_obciaza_usterka(
        &room,
        &UstalKogoObciazaUsterkaParams {
            spor_decyzja_id: Some("spor".to_string()),
            ..landlord_without_decision
        },
    )
    .expect("settle who the defect burdens");
    order_repair(&room, &repair());

    expect::refused(
        room.execute(
            "INSERT INTO najem_usterki_zgloszenia (id, chwila, opis, uniemozliwia, obciaza, zlecil, zlecono,
              wykonawca, koszt)
            VALUES ('od-razu', '2027-01-20', 'opis', 0, 'wynajmujacy', 'c', '2027-01-21', 'w', 1)",
            &[],
        ),
        "zgłoszenie usterki powstaje bez decyzji, zlecenia i usunięcia",
    );
}

/// Usterka: naprawa ponad kwotę z umowy o zarządzanie za zgodą wynajmującego i człowieka
#[test]
fn defect_repair_above_limit() {
    let room = new_room();
    managed_lease(&room, Przeplyw::PrzezBiuro);
    report_defect(&room, ObciazaUsterka::Wynajmujacy);
    let rule = "naprawa ponad kwotę napraw z umowy o zarządzanie wymaga zgody wynajmującego";

    let above_limit = ZlecNapraweUsterkiParams {
        koszt: 50_001,
        ..repair()
    };
    expect::refused(
        najem_usterki::zlec_naprawe_usterki(&room, &above_limit),
        rule,
    );

    add_inflow_from(&room, "zgoda-na-naprawe", "wlascicielka");
    sprawa_zgody::wpisz_zgode(
        &room,
        &WpiszZgodeParams {
            tresc: "Zgadzam się na wymianę syfonu za 800 zł.".to_string(),
            zlozono: "2027-01-20T12:00:00Z".to_string(),
            wplyw_id: Some("zgoda-na-naprawe".to_string()),
            trwaly_nosnik: true,
            dotyczy_tabela: Some("najem_usterki_zgloszenia".to_string()),
            dotyczy_id: Some("usterka".to_string()),
            ..consent("zgoda-wynajmujacej", "wlascicielka", "zgoda_na_naprawe")
        },
    )
    .expect("write the landlord's consent to the repair");
    let with_landlord_consent = ZlecNapraweUsterkiParams {
        koszt: 80_000,
        zgoda_wynajmujacego_id: Some("zgoda-wynajmujacej".to_string()),
        ..repair()
    };
    expect::refused(
        najem_usterki::zlec_naprawe_usterki(&room, &with_landlord_consent),
        rule,
    );
    approved_decision_about(
        &room,
        "zlecenie",
        RodzajDecyzji::Inna,
        "najem_usterki_zgloszenia",
        "usterka",
    );
    order_repair(
        &room,
        &ZlecNapraweUsterkiParams {
            zlecenie_decyzja_id: Some("zlecenie".to_string()),
            ..with_landlord_consent
        },
    );

    let cheaper = ZlecNapraweUsterkiParams {
        koszt: 10_000,
        ..repair()
    };
    expect::refused(
        najem_usterki::zlec_naprawe_usterki(&room, &cheaper),
        "zgłoszenia usterki się nie zmienia",
    );
}

/// Usterka: usunięcie potwierdza najemca wiadomością od siebie; przeglądów się nie zmienia
#[test]
fn defect_removal_confirmed_by_tenant() {
    let room = new_room();
    managed_lease(&room, Przeplyw::PrzezBiuro);
    report_defect(&room, ObciazaUsterka::Wynajmujacy);
    order_repair(&room, &repair());

    add_inflow_from(&room, "od-wlascicielki", "wlascicielka");
    let confirmed_in = |inflow_id: &str| OdnotujUsuniecieUsterkiParams {
        usunieto: "2027-01-22".to_string(),
        potwierdzenie_wplyw_id: inflow_id.to_string(),
        id: "usterka".to_string(),
    };
    expect::refused(
        najem_usterki::odnotuj_usuniecie_usterki(&room, &confirmed_in("od-wlascicielki")),
        "usunięcie usterki potwierdza najemca wiadomością od siebie",
    );
    // A writer outside the client can leave out the confirmation that `odnotuj_usuniecie_usterki`
    // requires. The table refuses the removal without one.
    let removal_without_confirmation = room.execute(
        "UPDATE najem_usterki_zgloszenia
        SET usunieto = :usunieto,
          potwierdzenie_wplyw_id = :potwierdzenie_wplyw_id
        WHERE id = :id",
        &[
            ("usunieto", text("2027-01-22")),
            ("potwierdzenie_wplyw_id", SqlValue::Null),
            ("id", text("usterka")),
        ],
    );
    expect::refused(
        removal_without_confirmation,
        "usuniecie_z_potwierdzeniem_najemcy",
    );
    add_inflow_from(&room, "od-najemcy", "najemca");
    najem_usterki::odnotuj_usuniecie_usterki(&room, &confirmed_in("od-najemcy"))
        .expect("record the defect removed");

    najem_usterki::wpisz_przeglad(
        &room,
        &WpiszPrzegladParams {
            data: "2027-02-01".to_string(),
            wynik: "Lokal bez uwag".to_string(),
            dokument_id: None,
        },
    )
    .expect("write the inspection");
    expect::refused(
        room.execute("UPDATE najem_usterki_przeglady SET wynik = 'inny'", &[]),
        "przeglądu się nie zmienia",
    );
}

// R9: no deletion, no replacement -----------------------------------------------------------------

/// The tables of the rental type, whose names start with `najem_`.
fn najem_tables(room: &MemoryRoom) -> Vec<String> {
    let rows = room
        .execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'najem\\_%' ESCAPE '\\'",
            &[],
        )
        .expect("read the tables");

    rows.iter()
        .map(|row| row.get::<String>("name").expect("a table has a name"))
        .collect()
}

/// R9: wierszy typu najmu się nie usuwa ani nie zastępuje przez REPLACE i upsert
#[test]
fn no_deletion_or_replacement() {
    let room = new_room();
    full_managed_rental(&room);

    // The managed ordinary lease has no substitute flat and no use of the deposit during the lease.
    prepare_lease(
        &room,
        &lease_without_deposit("okazjonalny", RodzajNajmu::Okazjonalny),
    );
    najem_umowy::wpisz_lokal_zastepczy(
        &room,
        &WpiszLokalZastepczyParams {
            najem_id: "okazjonalny".to_string(),
            adres: "ul. Lipowa 4".to_string(),
            wskazano: "2026-11-20".to_string(),
        },
    )
    .expect("write the substitute flat");
    institutional_deposit_used(&room);

    let tables = najem_tables(&room);
    assert_eq!(tables.len(), 20, "{tables:?}");
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
