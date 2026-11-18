//! Scenarios of the core additions for the case types (spec/cases/sprawa): the views of approved
//! decisions (sprawa_decyzje_zatwierdzone), admitted accounts (sprawa_rachunki_dopuszczone) and
//! the start of work under a contract (sprawa_umowy_praca_od), the last day to withdraw from a
//! contract (odstapienie_do) and the handover protocols (sprawa-protokoly).
//!
//! Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
//! allows and a write it refuses. The views hold one rule each; the scenarios check both what a view
//! shows and that the rules of the core read it.

use std::collections::BTreeMap;

use xboards_agencja_nieruchomosci::cases::sprawa::{
    sprawa_decyzje, sprawa_dokumenty, sprawa_protokoly, sprawa_rachunki, sprawa_terminy,
    sprawa_uczestnicy, sprawa_umowy, sprawa_wyplaty,
};
use xboards_agencja_nieruchomosci::xmeta_room::{FromSqlValue, MemoryRoom, MetaRoom, SqlValue};

use sprawa_decyzje::{
    PoprosODecyzjeParams, RozstrzygnijDecyzjeParams, UznajProsbeZaZbednaParams,
    WpisyRodzaj as RodzajDecyzji, WpisyRozstrzygniecie as Rozstrzygniecie,
};
use sprawa_dokumenty::{
    OdrzucDokumentParams, WpisyRodzaj as RodzajDokumentu, WpiszWymaganyDokumentParams,
};
use sprawa_protokoly::{
    DopiszPozycjeProtokoluParams, PodpiszProtokolParams, PoprawPozycjeProtokoluParams,
    PozycjeRodzaj as RodzajPozycji, PrzygotujProtokolParams, WpisyRodzaj as RodzajProtokolu,
    WpisyStan as StanProtokolu,
};
use sprawa_rachunki::{WpiszRachunekParams, ZastapRachunekParams};
use sprawa_terminy::{
    OdnotujPotwierdzenieTerminuParams, WpisyPilnujeStanowisko as PilnujeStanowisko,
    WpisyRodzaj as RodzajTerminu, WpiszTerminParams,
};
use sprawa_uczestnicy::WpiszUczestnikaParams;
use sprawa_umowy::{
    OdnotujZawarcieUmowyParams, PrzekazUmoweDoPodpisuParams, WpisyForma as FormaUmowy,
    WpisySposobZawarcia as SposobZawarcia, WpiszZdarzenieUmowyParams,
    ZdarzeniaRodzaj as RodzajZdarzeniaUmowy,
};
use sprawa_wyplaty::{OdnotujWykonanieWyplatyParams, PrzygotujWyplateParams};

use crate::expect;
use crate::sprawa::{
    WITHDRAWAL_ENDS, account, add_account, add_consent, add_flat, add_person,
    add_received_document, admitted_account, approve, approved_decision, ask_for_decision,
    check_identity, checked_and_approved_payout, conclude_contract, consent, contract,
    decision_request_about, flat, new_room, payout, person, prepare_contract, prepare_payout,
    received_document, settle_decision, settlement,
};

/// A text value to bind in raw SQL.
fn text(value: &str) -> SqlValue {
    SqlValue::Text(value.to_string())
}

/// The time the broker signs a decision unless a scenario says otherwise.
const SIGNED_AT: &str = "2026-11-03T12:00:00Z";

/// The ids of the approved decisions, sorted.
fn approved_ids(room: &impl MetaRoom) -> Vec<String> {
    let approved = sprawa_decyzje::decyzje_zatwierdzone(room).expect("read the approved decisions");

    let mut ids: Vec<String> = approved.into_iter().map(|row| row.id).collect();
    ids.sort();

    ids
}

/// The ids of the admitted accounts, in the order of the view.
fn admitted_ids(room: &impl MetaRoom) -> Vec<String> {
    let admitted = sprawa_rachunki::rachunki_dopuszczone(room).expect("read the admitted accounts");

    admitted.into_iter().map(|row| row.id).collect()
}

/// A new decision `decision_id` on the same row that refuses what the decision `replaced_id`
/// allowed, asked on 2026-11-21 at 09:00 and refused at 10:00.
fn withdraw_approval(
    room: &impl MetaRoom,
    decision_id: &str,
    replaced_id: &str,
    rodzaj: RodzajDecyzji,
    dotyczy_tabela: &str,
    dotyczy_id: &str,
) {
    let request = PoprosODecyzjeParams {
        zlozono: "2026-11-21T09:00:00Z".to_string(),
        zastepuje_decyzja_id: Some(replaced_id.to_string()),
        ..decision_request_about(decision_id, rodzaj, dotyczy_tabela, dotyczy_id)
    };
    ask_for_decision(room, &request);

    let refusal = settlement(decision_id, Rozstrzygniecie::Odmowa, "2026-11-21T10:00:00Z");
    settle_decision(room, &refusal);
}

// Approved decisions ------------------------------------------------------------------------------

/// Zatwierdzona decyzja to zgoda albo zgoda z warunkiem; czekająca, odmowa i zastąpiona nie
#[test]
fn approved_decisions_view() {
    let room = new_room();
    let about_deadline = |decision_id: &str, deadline_id: &str| {
        decision_request_about(
            decision_id,
            RodzajDecyzji::Inna,
            "sprawa_terminy_wpisy",
            deadline_id,
        )
    };

    ask_for_decision(&room, &about_deadline("czeka", "termin-1"));
    ask_for_decision(&room, &about_deadline("zbedna", "termin-2"));
    sprawa_decyzje::uznaj_prosbe_za_zbedna(
        &room,
        &UznajProsbeZaZbednaParams {
            powod_zbednosci: "termin przesunięty".to_string(),
            id: "zbedna".to_string(),
        },
    )
    .expect("find the request unneeded");
    ask_for_decision(&room, &about_deadline("odmowa", "termin-3"));
    settle_decision(
        &room,
        &settlement("odmowa", Rozstrzygniecie::Odmowa, SIGNED_AT),
    );
    approved_decision(&room, &about_deadline("zgoda", "termin-4"), SIGNED_AT);
    ask_for_decision(&room, &about_deadline("z-warunkiem", "termin-5"));
    settle_decision(
        &room,
        &RozstrzygnijDecyzjeParams {
            warunek: Some("po okazaniu odpisu z księgi".to_string()),
            ..settlement("z-warunkiem", Rozstrzygniecie::ZgodaZWarunkiem, SIGNED_AT)
        },
    );
    assert_eq!(approved_ids(&room), ["z-warunkiem", "zgoda"]);

    // A refusal that replaces an approval takes it out of the view.
    withdraw_approval(
        &room,
        "cofniecie",
        "zgoda",
        RodzajDecyzji::Inna,
        "sprawa_terminy_wpisy",
        "termin-4",
    );
    assert_eq!(approved_ids(&room), ["z-warunkiem"]);
}

/// Reguły rdzenia czytają widok: zastąpiona zgoda nie potwierdza terminu
#[test]
fn rules_read_approved_decisions() {
    let room = new_room();
    sprawa_terminy::wpisz_termin(
        &room,
        &WpiszTerminParams {
            id: "termin".to_string(),
            rodzaj: RodzajTerminu::ZUmowy,
            co: "Przekazanie lokalu najemcy".to_string(),
            zdarzenie_tabela: None,
            zdarzenie_id: None,
            zdarzenie_chwila: None,
            koniec: "2026-11-30T10:00:00Z".to_string(),
            podstawa: Some("§ 3 umowy najmu".to_string()),
            pilnuje_stanowisko: PilnujeStanowisko::OpiekunNajmu,
            kalendarz_odnosnik: None,
        },
    )
    .expect("add the deadline");
    let confirmation = decision_request_about(
        "zgoda",
        RodzajDecyzji::PotwierdzenieTerminu,
        "sprawa_terminy_wpisy",
        "termin",
    );
    approved_decision(&room, &confirmation, SIGNED_AT);
    withdraw_approval(
        &room,
        "cofniecie",
        "zgoda",
        RodzajDecyzji::PotwierdzenieTerminu,
        "sprawa_terminy_wpisy",
        "termin",
    );

    let confirmation_by = |decision_id: &str| OdnotujPotwierdzenieTerminuParams {
        potwierdzenie_decyzja_id: decision_id.to_string(),
        id: "termin".to_string(),
    };
    expect::refused(
        sprawa_terminy::odnotuj_potwierdzenie_terminu(&room, &confirmation_by("zgoda")),
        "termin potwierdza tylko zatwierdzona decyzja potwierdzenie_terminu",
    );

    let new_confirmation = PoprosODecyzjeParams {
        zlozono: "2026-11-22T09:00:00Z".to_string(),
        ..decision_request_about(
            "nowa-zgoda",
            RodzajDecyzji::PotwierdzenieTerminu,
            "sprawa_terminy_wpisy",
            "termin",
        )
    };
    approved_decision(&room, &new_confirmation, "2026-11-22T10:00:00Z");
    sprawa_terminy::odnotuj_potwierdzenie_terminu(&room, &confirmation_by("nowa-zgoda"))
        .expect("confirm the deadline by the new approval");
}

// Admitted accounts -------------------------------------------------------------------------------

/// Rachunek dopuszczony: zatwierdzona decyzja dopuszczenie_rachunku i brak zastępcy
#[test]
fn admitted_accounts_view() {
    let room = new_room();
    add_person(&room, &person("najemca"));
    add_account(&room, &account("bez-decyzji", "najemca"));
    admitted_account(&room, "dopuszczony", "najemca");
    admitted_account(&room, "zastapiony", "najemca");
    assert_eq!(admitted_ids(&room), ["dopuszczony", "zastapiony"]);

    add_account(
        &room,
        &WpiszRachunekParams {
            numer: "PL27114020040000300201355387".to_string(),
            ..account("nowy-numer", "najemca")
        },
    );
    sprawa_rachunki::zastap_rachunek(
        &room,
        &ZastapRachunekParams {
            zastapiony_przez_id: "nowy-numer".to_string(),
            zastapil: "tozsamosc-agenta".to_string(),
            zastapiono: "2026-11-08T09:00:00Z".to_string(),
            id: "zastapiony".to_string(),
        },
    )
    .expect("replace the account");
    assert_eq!(admitted_ids(&room), ["dopuszczony"]);

    withdraw_approval(
        &room,
        "cofniecie",
        "dopuszczenie-dopuszczony",
        RodzajDecyzji::DopuszczenieRachunku,
        "sprawa_rachunki_wpisy",
        "dopuszczony",
    );
    assert!(admitted_ids(&room).is_empty());
}

/// Wypłata czyta rachunki dopuszczone: po cofnięciu dopuszczenia nie powstaje ani nie wychodzi
#[test]
fn payouts_read_admitted_accounts() {
    let room = new_room();
    add_person(&room, &person("najemca"));
    admitted_account(&room, "rachunek", "najemca");
    checked_and_approved_payout(&room, "zwrot-kaucji", "najemca", "rachunek");
    add_received_document(
        &room,
        &received_document(
            "potwierdzenie-przelewu",
            RodzajDokumentu::PotwierdzenieBankowe,
        ),
    );

    withdraw_approval(
        &room,
        "cofniecie",
        "dopuszczenie-rachunek",
        RodzajDecyzji::DopuszczenieRachunku,
        "sprawa_rachunki_wpisy",
        "rachunek",
    );

    let execution = OdnotujWykonanieWyplatyParams {
        wykonal: "tozsamosc-posrednika".to_string(),
        wykonano: "2026-11-21T13:00:00Z".to_string(),
        potwierdzenie_dokument_id: "potwierdzenie-przelewu".to_string(),
        id: "zwrot-kaucji".to_string(),
    };
    expect::refused(
        sprawa_wyplaty::odnotuj_wykonanie_wyplaty(&room, &execution),
        "wykonanie wymaga zgody na wypłatę po aktualnym sprawdzeniu niezastąpionego i dopuszczonego",
    );
    let second_payout = PrzygotujWyplateParams {
        utworzono: "2026-11-21T11:00:00Z".to_string(),
        ..payout("druga-wyplata", "najemca", "rachunek")
    };
    expect::refused(
        sprawa_wyplaty::przygotuj_wyplate(&room, &second_payout),
        "wypłata idzie na rachunek odbiorcy dopuszczony decyzją i niezastąpiony",
    );

    // A new admission of the same account lets a new payout through.
    let new_admission = PoprosODecyzjeParams {
        zlozono: "2026-11-22T09:00:00Z".to_string(),
        zastepuje_decyzja_id: Some("cofniecie".to_string()),
        ..decision_request_about(
            "ponowne-dopuszczenie",
            RodzajDecyzji::DopuszczenieRachunku,
            "sprawa_rachunki_wpisy",
            "rachunek",
        )
    };
    ask_for_decision(&room, &new_admission);
    approve(&room, "ponowne-dopuszczenie", "2026-11-22T10:00:00Z");
    prepare_payout(
        &room,
        &PrzygotujWyplateParams {
            utworzono: "2026-11-22T11:00:00Z".to_string(),
            ..payout("druga-wyplata", "najemca", "rachunek")
        },
    );
}

// The last day to withdraw and the start of work --------------------------------------------------

/// A contract (see [`contract`]) of a party Kontroler checked (`tozsamosc-<id>`), handed over to
/// signing.
fn contract_for(room: &impl MetaRoom, contract_id: &str, party_id: &str) {
    check_identity(room, &format!("tozsamosc-{contract_id}"), party_id);
    prepare_contract(room, &contract(contract_id), &[party_id]);

    let for_signing = PrzekazUmoweDoPodpisuParams {
        id: contract_id.to_string(),
    };
    sprawa_umowy::przekaz_umowe_do_podpisu(room, &for_signing)
        .expect("hand the contract over for signing");
}

/// A try to conclude a contract electronically on 2026-11-04, with its own signed document
/// `proba-umowy-<id>` and policy `proba-polisy-<id>`, which it writes unless they exist.
fn conclusion_attempt(
    room: &MemoryRoom,
    contract_id: &str,
    sposob_zawarcia: SposobZawarcia,
    odstapienie_do: Option<&str>,
) -> OdnotujZawarcieUmowyParams {
    let signed_id = format!("proba-umowy-{contract_id}");
    let policy_id = format!("proba-polisy-{contract_id}");

    for (document_id, rodzaj) in [
        (&signed_id, RodzajDokumentu::UmowaZBiurem),
        (&policy_id, RodzajDokumentu::Polisa),
    ] {
        let count = room
            .value(
                "SELECT count(*) FROM sprawa_dokumenty_wpisy WHERE id = :id",
                &[("id", text(document_id))],
            )
            .expect("count the documents");
        let exists = i64::from_sql_value(&count).expect("a count is an integer") > 0;
        if exists {
            continue;
        }

        add_received_document(room, &received_document(document_id, rodzaj));
    }

    OdnotujZawarcieUmowyParams {
        forma: FormaUmowy::Elektroniczna,
        sposob_zawarcia,
        zawarto: "2026-11-04T12:00:00Z".to_string(),
        podpisal_za_biuro: "tozsamosc-posrednika".to_string(),
        dokument_id: signed_id,
        polisa_dokument_id: policy_id,
        odstapienie_do: odstapienie_do.map(str::to_string),
        id: contract_id.to_string(),
    }
}

/// Umowa z konsumentem poza lokalem albo na odległość ma odstapienie_do (art. 27 upk); inna nie
#[test]
fn withdrawal_last_day() {
    let room = new_room();
    add_person(
        &room,
        &WpiszUczestnikaParams {
            konsument: true,
            ..person("konsumentka")
        },
    );
    add_person(
        &room,
        &WpiszUczestnikaParams {
            imiona: Some("Piotr".to_string()),
            nazwa: Some("Nowak Inwestycje".to_string()),
            ceidg: true,
            ..person("przedsiebiorca")
        },
    );
    let refusal =
        "ma odstapienie_do nie wcześniej niż koniec terminu z art. 27 upk; inna umowa go nie ma";

    // Fourteen days from the day of conclusion, which is not counted (art. 111 § 2 KC).
    contract_for(&room, "na-odleglosc", "konsumentka");
    for odstapienie_do in [None, Some("2026-11-17")] {
        let attempt = conclusion_attempt(
            &room,
            "na-odleglosc",
            SposobZawarcia::NaOdleglosc,
            odstapienie_do,
        );
        expect::refused(
            sprawa_umowy::odnotuj_zawarcie_umowy(&room, &attempt),
            refusal,
        );
    }
    let attempt_with_time = conclusion_attempt(
        &room,
        "na-odleglosc",
        SposobZawarcia::NaOdleglosc,
        Some("2026-11-18T23:59:00Z"),
    );
    expect::refused(
        sprawa_umowy::odnotuj_zawarcie_umowy(&room, &attempt_with_time),
        "odstapienie_do_jest_dniem_po_zawarciu",
    );
    conclude_contract(
        &room,
        "na-odleglosc",
        SposobZawarcia::NaOdleglosc,
        Some(WITHDRAWAL_ENDS),
    );

    // After an unannounced visit the consumer has 30 days (art. 27 ust. 2 upk).
    contract_for(&room, "wizyta", "konsumentka");
    let attempt = conclusion_attempt(
        &room,
        "wizyta",
        SposobZawarcia::NieumowionaWizyta,
        Some(WITHDRAWAL_ENDS),
    );
    expect::refused(
        sprawa_umowy::odnotuj_zawarcie_umowy(&room, &attempt),
        refusal,
    );
    conclude_contract(
        &room,
        "wizyta",
        SposobZawarcia::NieumowionaWizyta,
        Some("2026-12-04"),
    );

    // Without the information on withdrawal the right lasts 12 months longer (art. 29 ust. 1 upk).
    contract_for(&room, "bez-informacji", "konsumentka");
    conclude_contract(
        &room,
        "bez-informacji",
        SposobZawarcia::PozaLokalem,
        Some("2027-11-18"),
    );

    // No right to withdraw: at the office, or no consumer among the parties.
    contract_for(&room, "w-lokalu", "konsumentka");
    let attempt = conclusion_attempt(
        &room,
        "w-lokalu",
        SposobZawarcia::WLokaluBiura,
        Some(WITHDRAWAL_ENDS),
    );
    expect::refused(
        sprawa_umowy::odnotuj_zawarcie_umowy(&room, &attempt),
        refusal,
    );
    conclude_contract(&room, "w-lokalu", SposobZawarcia::WLokaluBiura, None);
    contract_for(&room, "przedsiebiorcy", "przedsiebiorca");
    let attempt = conclusion_attempt(
        &room,
        "przedsiebiorcy",
        SposobZawarcia::NaOdleglosc,
        Some(WITHDRAWAL_ENDS),
    );
    expect::refused(
        sprawa_umowy::odnotuj_zawarcie_umowy(&room, &attempt),
        refusal,
    );
    conclude_contract(&room, "przedsiebiorcy", SposobZawarcia::NaOdleglosc, None);

    expect::refused(
        room.execute(
            "UPDATE sprawa_umowy_wpisy SET odstapienie_do = '2026-12-31' WHERE id = 'na-odleglosc'",
            &[],
        ),
        "dane zawarcia umowy się nie zmieniają",
    );
}

/// Praca od: zawarcie, dzień po odstapienie_do albo wcześniejsze żądanie konsumenta
#[test]
fn work_start() {
    let room = new_room();
    add_person(
        &room,
        &WpiszUczestnikaParams {
            konsument: true,
            ..person("konsumentka")
        },
    );

    contract_for(&room, "w-lokalu", "konsumentka");
    conclude_contract(&room, "w-lokalu", SposobZawarcia::WLokaluBiura, None);
    let at_a_distance = [
        "bez-zadania",
        "z-zadaniem",
        "zadanie-w-umowie",
        "zadanie-po-terminie",
    ];
    for contract_id in at_a_distance {
        contract_for(&room, contract_id, "konsumentka");
        conclude_contract(
            &room,
            contract_id,
            SposobZawarcia::NaOdleglosc,
            Some(WITHDRAWAL_ENDS),
        );
    }
    prepare_contract(&room, &contract("projekt"), &[]);

    add_consent(
        &room,
        &consent("zadanie", "konsumentka", "zadanie_rozpoczecia_pracy"),
    );
    let demands = [
        ("zadanie-1", "z-zadaniem", "2026-11-06T09:00:00Z"),
        ("zadanie-2", "z-zadaniem", "2026-11-08T09:00:00Z"),
        // A request signed with the contract, dated a moment before the office signed it.
        ("zadanie-3", "zadanie-w-umowie", "2026-11-04T11:00:00Z"),
        ("zadanie-4", "zadanie-po-terminie", "2026-11-25T09:00:00Z"),
    ];
    for (event_id, umowa_id, chwila) in demands {
        let event = WpiszZdarzenieUmowyParams {
            id: event_id.to_string(),
            umowa_id: umowa_id.to_string(),
            rodzaj: RodzajZdarzeniaUmowy::ZadanieRozpoczeciaPracy,
            chwila: chwila.to_string(),
            opis: None,
            dokument_id: None,
            wysylka_id: None,
            wplyw_id: None,
            zgoda_id: Some("zadanie".to_string()),
        };

        sprawa_umowy::wpisz_zdarzenie_umowy(&room, &event).expect("add the demand to start work");
    }

    let starts = sprawa_umowy::praca_od_umow(&room).expect("read the start of work");
    let work_from: BTreeMap<&str, &str> = starts
        .iter()
        .map(|row| (row.umowa_id.as_str(), row.od.as_str()))
        .collect();
    assert_eq!(
        work_from,
        BTreeMap::from([
            ("w-lokalu", "2026-11-04T12:00:00Z"),
            ("bez-zadania", "2026-11-19"),
            ("z-zadaniem", "2026-11-06T09:00:00Z"),
            ("zadanie-w-umowie", "2026-11-04T12:00:00Z"),
            ("zadanie-po-terminie", "2026-11-19"),
        ])
    );
}

// Handover protocols ------------------------------------------------------------------------------

/// Writes the draft protocol `protocol_id` of the kind `rodzaj` for the flat „lokal”, made on
/// 2026-11-10 at 10:00 (the Python defaults are „protokol” and „wydanie”).
fn draft_protocol(room: &impl MetaRoom, protocol_id: &str, rodzaj: RodzajProtokolu) {
    let params = PrzygotujProtokolParams {
        id: protocol_id.to_string(),
        rodzaj,
        nieruchomosc_id: "lokal".to_string(),
        chwila: "2026-11-10T10:00:00Z".to_string(),
    };

    sprawa_protokoly::przygotuj_protokol(room, &params).expect("prepare the protocol");
}

/// The default parameters of an item of the protocol „protokol”: a working washing machine with a
/// scratch.
fn item() -> DopiszPozycjeProtokoluParams {
    DopiszPozycjeProtokoluParams {
        protokol_id: "protokol".to_string(),
        rodzaj: RodzajPozycji::Wyposazenie,
        nazwa: "pralka".to_string(),
        numer: None,
        opis_stanu: Some("sprawna, rysa na obudowie".to_string()),
        zdjecie_sciezka: None,
        zdjecie_skrot: None,
    }
}

/// Writes the item of `params` (see [`item`]).
fn add_item(room: &impl MetaRoom, params: &DopiszPozycjeProtokoluParams) {
    sprawa_protokoly::dopisz_pozycje_protokolu(room, params).expect("add an item to the protocol");
}

/// The parameters of signing the protocol „protokol” with the document `dokument_id`.
fn signing(dokument_id: &str) -> PodpiszProtokolParams {
    PodpiszProtokolParams {
        dokument_id: dokument_id.to_string(),
        id: "protokol".to_string(),
    }
}

/// Protokół powstaje jako projekt; podpisany ma otrzymany dokument rodzaju protokol
#[test]
fn protocol_signed_with_document() {
    let room = new_room();
    add_flat(&room, &flat("lokal"));
    expect::refused(
        room.execute(
            "INSERT INTO sprawa_protokoly_wpisy (id, rodzaj, nieruchomosc_id, chwila, stan, dokument_id)
            VALUES ('od-razu', 'wydanie', 'lokal', '2026-11-10T10:00:00Z', 'podpisany', NULL)",
            &[],
        ),
        "protokół powstaje jako projekt",
    );
    draft_protocol(&room, "protokol", RodzajProtokolu::Wydanie);

    add_received_document(&room, &received_document("pismo", RodzajDokumentu::Pismo));
    sprawa_dokumenty::wpisz_wymagany_dokument(
        &room,
        &WpiszWymaganyDokumentParams {
            id: "protokol-w-drodze".to_string(),
            rodzaj: RodzajDokumentu::Protokol,
            nazwa: "Protokół".to_string(),
            dotyczy_uczestnik_id: None,
            dotyczy_nieruchomosc_id: None,
            wymaga: None,
        },
    )
    .expect("add the required protocol");
    let refusal = "podpisany protokół wskazuje otrzymany dokument rodzaju protokol";

    // A writer outside the client can leave out the document that `podpisz_protokol` requires.
    // The table refuses the signature without one.
    let signature_without_document = room.execute(
        "UPDATE sprawa_protokoly_wpisy
        SET stan = 'podpisany',
          dokument_id = :dokument_id
        WHERE id = :id",
        &[("dokument_id", SqlValue::Null), ("id", text("protokol"))],
    );
    expect::refused(signature_without_document, refusal);
    for dokument_id in ["pismo", "protokol-w-drodze"] {
        expect::refused(
            sprawa_protokoly::podpisz_protokol(&room, &signing(dokument_id)),
            refusal,
        );
    }

    add_received_document(
        &room,
        &received_document("podpisany", RodzajDokumentu::Protokol),
    );
    expect::refused(
        room.execute(
            "UPDATE sprawa_protokoly_wpisy SET dokument_id = 'podpisany' WHERE id = 'protokol'",
            &[],
        ),
        "podpisany_ma_dokument",
    );
    sprawa_protokoly::podpisz_protokol(&room, &signing("podpisany")).expect("sign the protocol");
    let protocols = sprawa_protokoly::protokoly(&room).expect("read the protocols");
    assert_eq!(protocols[0].stan, StanProtokolu::Podpisany);
}

/// Pozycje protokołu: licznik z numerem i odczytem, zdjęcie ze skrótem
#[test]
fn protocol_items() {
    let room = new_room();
    add_flat(&room, &flat("lokal"));
    draft_protocol(&room, "protokol", RodzajProtokolu::Wydanie);

    let meter_without_reading = DopiszPozycjeProtokoluParams {
        rodzaj: RodzajPozycji::Licznik,
        nazwa: "licznik wody zimnej".to_string(),
        numer: Some("W-1203".to_string()),
        opis_stanu: None,
        ..item()
    };
    expect::refused(
        sprawa_protokoly::dopisz_pozycje_protokolu(&room, &meter_without_reading),
        "licznik_ma_numer_i_odczyt",
    );
    let photo_without_hash = DopiszPozycjeProtokoluParams {
        rodzaj: RodzajPozycji::Pomieszczenie,
        nazwa: "łazienka".to_string(),
        opis_stanu: None,
        zdjecie_sciezka: Some("protokoly/lazienka.jpg".to_string()),
        ..item()
    };
    expect::refused(
        sprawa_protokoly::dopisz_pozycje_protokolu(&room, &photo_without_hash),
        "zdjecie_ze_skrotem",
    );
    add_item(
        &room,
        &DopiszPozycjeProtokoluParams {
            opis_stanu: Some("45,210 m³".to_string()),
            ..meter_without_reading
        },
    );
    add_item(
        &room,
        &DopiszPozycjeProtokoluParams {
            rodzaj: RodzajPozycji::Klucze,
            nazwa: "klucze do skrzynki pocztowej".to_string(),
            opis_stanu: Some("2 sztuki".to_string()),
            ..item()
        },
    );

    let items = sprawa_protokoly::pozycje_protokolow(&room).expect("read the items");
    assert_eq!(items.len(), 2);
}

/// Po podpisie protokołu i jego pozycji się nie zmienia; dokumentu się nie odrzuca
#[test]
fn protocol_frozen_after_signing() {
    let room = new_room();
    add_flat(&room, &flat("lokal"));
    draft_protocol(&room, "protokol", RodzajProtokolu::Wydanie);
    draft_protocol(&room, "drugi", RodzajProtokolu::Zwrot);
    add_item(&room, &item());

    // In the draft an item is corrected, but it stays in its protocol.
    let correction_of = |opis_stanu: &str| PoprawPozycjeProtokoluParams {
        nazwa: "pralka".to_string(),
        numer: None,
        opis_stanu: Some(opis_stanu.to_string()),
        zdjecie_sciezka: None,
        zdjecie_skrot: None,
        id: 1,
    };
    sprawa_protokoly::popraw_pozycje_protokolu(
        &room,
        &correction_of("sprawna, rysa na obudowie, brak węża"),
    )
    .expect("correct the item in the draft");
    expect::refused(
        room.execute(
            "UPDATE sprawa_protokoly_pozycje SET protokol_id = 'drugi' WHERE id = 1",
            &[],
        ),
        "pozycja zostaje w swoim protokole",
    );

    add_received_document(
        &room,
        &received_document("podpisany", RodzajDokumentu::Protokol),
    );
    sprawa_protokoly::podpisz_protokol(&room, &signing("podpisany")).expect("sign the protocol");

    let note_after_signing = DopiszPozycjeProtokoluParams {
        rodzaj: RodzajPozycji::Uwaga,
        nazwa: "dopisek po podpisie".to_string(),
        opis_stanu: None,
        ..item()
    };
    expect::refused(
        sprawa_protokoly::dopisz_pozycje_protokolu(&room, &note_after_signing),
        "pozycje dopisuje się tylko do projektu protokołu",
    );
    expect::refused(
        sprawa_protokoly::popraw_pozycje_protokolu(&room, &correction_of("nowa")),
        "pozycji podpisanego protokołu się nie zmienia",
    );
    expect::refused(
        room.execute(
            "UPDATE sprawa_protokoly_wpisy SET chwila = '2026-11-11T10:00:00Z' WHERE id = 'protokol'",
            &[],
        ),
        "podpisanego protokołu się nie zmienia",
    );
    let rejection = OdrzucDokumentParams {
        powod_odrzucenia: "nieczytelny podpis".to_string(),
        id: "podpisany".to_string(),
    };
    expect::refused(
        sprawa_dokumenty::odrzuc_dokument(&room, &rejection),
        "dokument podpisanego protokołu nie zmienia rodzaju",
    );
    expect::refused(
        room.execute(
            "UPDATE sprawa_dokumenty_wpisy SET rodzaj = 'inny' WHERE id = 'podpisany'",
            &[],
        ),
        "dokument podpisanego protokołu nie zmienia rodzaju",
    );
}

/// Protokołów i pozycji się nie usuwa ani nie zastępuje przez REPLACE albo upsert
#[test]
fn protocol_no_deletion_or_replacement() {
    let room = new_room();
    add_flat(&room, &flat("lokal"));
    draft_protocol(&room, "protokol", RodzajProtokolu::Wydanie);
    add_item(&room, &item());
    let replacement = "wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE";

    let tables = [
        (
            "sprawa_protokoly_wpisy",
            "id, rodzaj, nieruchomosc_id, chwila, stan, dokument_id",
        ),
        (
            "sprawa_protokoly_pozycje",
            "id, protokol_id, rodzaj, nazwa, numer, opis_stanu, zdjecie_sciezka, zdjecie_skrot",
        ),
    ];
    for (table, columns) in tables {
        let copy = format!("INTO {table} ({columns}) SELECT {columns} FROM {table}");

        let replace = format!("INSERT OR REPLACE {copy}");
        expect::refused(room.execute(&replace, &[]), replacement);

        let upsert =
            format!("INSERT {copy} WHERE true ON CONFLICT (id) DO UPDATE SET id = excluded.id");
        expect::refused(room.execute(&upsert, &[]), replacement);

        let delete = format!("DELETE FROM {table}");
        expect::refused(
            room.execute(&delete, &[]),
            "wierszy teczki się nie usuwa (R9)",
        );
    }
}
