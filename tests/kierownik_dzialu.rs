//! Scenarios of the memory a department head adds (spec/positions/kierownik_dzialu).
//!
//! The catalog goes into the memory rooms of the four department heads, so its rules are checked
//! once here, in a room of the shared catalog. Its habits name their source in stanowisko-zrodla,
//! so the room imports that kind of the agent memory. Each scenario starts from an empty room and
//! checks one rule the database keeps: a write the rule allows and a write it refuses. The first
//! scenario follows a head who appoints an agent, takes its report, forwards its request, looks
//! into its case file, hands the case to Reception, sends out a new policy and replaces the agent.
//!
//! The building blocks marked `pub(crate)` are shared with the scenarios of the department heads:
//! their rooms hold the same kinds, so the operations of this room write there too.

use std::collections::HashMap;

use xboards_agencja_nieruchomosci::positions::kierownik_dzialu::{
    self, kierownik_dzialu_agenci, kierownik_dzialu_polisy, kierownik_dzialu_raporty,
    kierownik_dzialu_spis, kierownik_dzialu_standardy, kierownik_dzialu_zajrzenia,
    kierownik_dzialu_zwyczaje,
};
use xboards_agencja_nieruchomosci::positions::stanowisko::stanowisko_zrodla::{
    WpisyRodzaj as RodzajZrodla, WpiszZrodloParams, ZdarzeniaRodzaj,
};
use xboards_agencja_nieruchomosci::xmeta_room::{FromSqlValue, MemoryRoom, MetaRoom, SqlValue};

use kierownik_dzialu_agenci::{
    AgenciKlientaParams, ObudzAgentaParams, OdnotujUsuniecieZPokojuParams,
    OdnotujWpuszczenieParams, PowolajAgentaParams, UspijAgentaParams, WpisyStanowisko,
    WpuszczeniaPokojuParams, WymienAgentaParams, ZatwierdzPowolanieParams,
    ZatwierdzWpuszczenieParams,
};
use kierownik_dzialu_polisy::{OdnotujRozeslaniePolisyParams, RozeslaniaPolisyParams};
use kierownik_dzialu_raporty::{
    ProsbyPowodZdjecia, ProsbyPrzekazanoDo, PrzekazProsbeParams, PrzyjmijRaportParams,
    WpisyRodzaj as RodzajRaportu, ZglosZdjecieProsbyParams,
};
use kierownik_dzialu_spis::{
    DodajZnakParams, OdnotujPrzekazanieSprawyParams, OdnotujPrzekazanieWycofaniaParams,
    OdnotujPrzekazanieZamknieciaParams, OdnotujPrzekazanieZnakuParams, SprawyRodzaj,
    WpiszSpraweDoSpisuParams, WycofajZnakParams, ZamknijSpraweWSpisieParams,
    ZmienSpraweWSpisieParams, ZnakiRodzaj, ZnakiSprawyParams,
};
use kierownik_dzialu_standardy::{
    RozstrzygnijStandardParams, WersjePodpisalJako, WersjeRozstrzygniecie, WersjeStandarduParams,
    ZaproponujStandardParams,
};
use kierownik_dzialu_zajrzenia::{WpisyPowod, ZajrzeniaOdParams, ZapiszZajrzenieParams};
use kierownik_dzialu_zwyczaje::{
    Pewnosc, PotwierdzZwyczajParams, WpisyKontrahentRodzaj, WpisyPewnosc as PewnoscZwyczaju,
    WpisyPotwierdzenie, WpiszZwyczajParams, WykreslZwyczajParams, ZakwestionujZwyczajParams,
    ZwyczajeKontrahentaParams, ZwyczajeOdZrodlaParams,
};

use crate::expect;
use crate::stanowisko::{
    DELETION, REPLACEMENT, add_source, add_source_event, first_row_copy, source, source_event,
};

/// Who approves an appointment or a let-in for the owner, and the signature of the approval.
const OWNER: &str = "tozsamosc-wlasciciela";
const OWNER_SIGNATURE: &str = "podpis-kluczem-wlasciciela";

/// When the owner approves unless a scenario says otherwise.
const APPROVED_AT: &str = "2026-11-02T09:30:00Z";

fn new_room() -> MemoryRoom {
    kierownik_dzialu::open_in_memory().expect("open the room of a department head")
}

/// A text value to bind in raw SQL.
fn text(value: &str) -> SqlValue {
    SqlValue::Text(value.to_string())
}

// Building blocks, also for the scenarios of the department heads ---------------------------------

/// The default parameters of an agent of a case: the carer of an offer with its case file, not yet
/// approved by the owner.
pub(crate) fn agent(agent_id: &str) -> PowolajAgentaParams {
    PowolajAgentaParams {
        id: agent_id.to_string(),
        stanowisko: WpisyStanowisko::OpiekunOferty,
        tozsamosc: format!("tozsamosc-{agent_id}"),
        wlasciciel: "tozsamosc-posrednika".to_string(),
        teczka_odnosnik: Some("pokoj-teczki-1".to_string()),
        klient_odnosnik: None,
        powolano: "2026-11-02T09:00:00Z".to_string(),
        zatwierdzil: None,
        podpis: None,
        zatwierdzono: None,
    }
}

/// The default parameters of a Controller: no case file, a client in the client register.
pub(crate) fn controller(agent_id: &str) -> PowolajAgentaParams {
    PowolajAgentaParams {
        stanowisko: WpisyStanowisko::Kontroler,
        wlasciciel: "tozsamosc-wlasciciela-kontroli".to_string(),
        teczka_odnosnik: None,
        klient_odnosnik: Some("pokoj-kartoteki/klient-nowak".to_string()),
        ..agent(agent_id)
    }
}

/// Writes the appointment of an agent into the head's list of agents.
pub(crate) fn appoint_agent(room: &impl MetaRoom, params: &PowolajAgentaParams) {
    kierownik_dzialu_agenci::powolaj_agenta(room, params).expect("appoint the agent");
}

/// The default parameters of letting a Controller into a case file at the request of the Head of
/// control.
pub(crate) fn controller_let_in() -> OdnotujWpuszczenieParams {
    OdnotujWpuszczenieParams {
        prosil: Some("tozsamosc-kierownika-kontroli".to_string()),
        prosba_odnosnik: Some("wiadomosc-wskazanie-kontrolera".to_string()),
        wpuszczono: "2026-11-02T10:00:00Z".to_string(),
        ..unrequested_let_in()
    }
}

/// Writes that the head let an identity into a room.
pub(crate) fn let_in(room: &impl MetaRoom, params: &OdnotujWpuszczenieParams) {
    kierownik_dzialu_agenci::odnotuj_wpuszczenie(room, params).expect("let the identity in");
}

/// The default parameters of letting a Controller into a case file without a request.
fn unrequested_let_in() -> OdnotujWpuszczenieParams {
    OdnotujWpuszczenieParams {
        tozsamosc: "tozsamosc-kontrolera".to_string(),
        stanowisko: WpisyStanowisko::Kontroler,
        agent_id: None,
        pokoj_odnosnik: "pokoj-teczki-1".to_string(),
        klient_odnosnik: Some("pokoj-kartoteki/klient-nowak".to_string()),
        prosil: None,
        prosba_odnosnik: None,
        wpuscil: "tozsamosc-kierownika-posrednictwa".to_string(),
        wpuszczono: "2026-11-03T10:00:00Z".to_string(),
        zatwierdzil: None,
        podpis: None,
        zatwierdzono: None,
    }
}

/// The owner's approval of an agent's appointment.
fn appointment_approval(agent_id: &str, zatwierdzono: &str) -> ZatwierdzPowolanieParams {
    ZatwierdzPowolanieParams {
        zatwierdzil: OWNER.to_string(),
        podpis: OWNER_SIGNATURE.to_string(),
        zatwierdzono: zatwierdzono.to_string(),
        id: agent_id.to_string(),
    }
}

/// The owner's approval of a let-in.
fn let_in_approval(let_in_id: i64, zatwierdzono: &str) -> ZatwierdzWpuszczenieParams {
    ZatwierdzWpuszczenieParams {
        zatwierdzil: OWNER.to_string(),
        podpis: OWNER_SIGNATURE.to_string(),
        zatwierdzono: zatwierdzono.to_string(),
        id: let_in_id,
    }
}

/// The default parameters of a current report of an agent at the stage of publication.
pub(crate) fn report(report_id: &str, agent_id: &str) -> PrzyjmijRaportParams {
    PrzyjmijRaportParams {
        id: report_id.to_string(),
        agent_id: agent_id.to_string(),
        otrzymano: "2026-11-03T16:00:00Z".to_string(),
        rodzaj: RodzajRaportu::Biezacy,
        etap_albo_okres: "publikacja".to_string(),
        blokada: None,
        najblizszy_termin: None,
        decyzja_odnosnik: None,
        uwagi: None,
    }
}

/// Writes a report the head received from an agent.
pub(crate) fn receive_report(room: &impl MetaRoom, params: &PrzyjmijRaportParams) {
    kierownik_dzialu_raporty::przyjmij_raport(room, params).expect("receive the report");
}

/// The default parameters of a request from the report `raport-1`, forwarded to the Office head.
pub(crate) fn request() -> PrzekazProsbeParams {
    PrzekazProsbeParams {
        decyzja_odnosnik: "pokoj-teczki-1/decyzja-1".to_string(),
        raport_id: "raport-1".to_string(),
        przekazano_do: ProsbyPrzekazanoDo::KierownikBiura,
        przekazano: "2026-11-03T17:00:00Z".to_string(),
    }
}

/// Writes that the head forwarded an agent's request onto the list of requests.
pub(crate) fn forward_request(room: &impl MetaRoom, params: &PrzekazProsbeParams) {
    kierownik_dzialu_raporty::przekaz_prosbe(room, params).expect("forward the request");
}

/// The default parameters of a look into a case file after a report signals a problem.
fn look_in() -> ZapiszZajrzenieParams {
    ZapiszZajrzenieParams {
        teczka_odnosnik: "pokoj-teczki-1".to_string(),
        zajrzano: "2026-11-04T10:00:00Z".to_string(),
        powod: WpisyPowod::RaportSygnalizujeProblem,
        opis: "Raport mówi, że portal od tygodnia nie publikuje ogłoszenia".to_string(),
        raport_id: None,
    }
}

fn look_into_case(room: &impl MetaRoom, params: &ZapiszZajrzenieParams) {
    kierownik_dzialu_zajrzenia::zapisz_zajrzenie(room, params).expect("look into the case file");
}

/// The default parameters of a proposed version of a standard.
fn standard_version(standard: &str, numer: i64) -> ZaproponujStandardParams {
    ZaproponujStandardParams {
        standard: standard.to_string(),
        numer,
        sciezka: format!("standardy/{standard}-{numer}.typ"),
        skrot: format!("sha256-{standard}-{numer}"),
        opis_zmian: "Zdjęcia w kolejności pomieszczeń".to_string(),
        zaproponowano: "2026-11-01T09:00:00Z".to_string(),
    }
}

fn propose_standard(room: &impl MetaRoom, params: &ZaproponujStandardParams) {
    kierownik_dzialu_standardy::zaproponuj_standard(room, params).expect("propose the standard");
}

/// The default parameters of a decision on a version of a standard, signed by the owner.
fn standard_decision(
    version_id: i64,
    rozstrzygniecie: WersjeRozstrzygniecie,
) -> RozstrzygnijStandardParams {
    RozstrzygnijStandardParams {
        rozstrzygniecie,
        podpisal: OWNER.to_string(),
        podpisal_jako: WersjePodpisalJako::Wlasciciel,
        podpis: OWNER_SIGNATURE.to_string(),
        podpisano: "2026-11-02T09:00:00Z".to_string(),
        id: version_id,
    }
}

fn decide_standard(room: &impl MetaRoom, params: &RozstrzygnijStandardParams) {
    kierownik_dzialu_standardy::rozstrzygnij_standard(room, params)
        .expect("decide on the standard");
}

/// The default parameters of a probable habit of a portal, learned from a report.
fn habit() -> WpiszZwyczajParams {
    WpiszZwyczajParams {
        kontrahent_rodzaj: WpisyKontrahentRodzaj::Portal,
        kontrahent: "Portal Ogłoszeń".to_string(),
        temat: "odświeżanie".to_string(),
        opis: "Usuwa ogłoszenie po 30 dniach bez odświeżenia".to_string(),
        zrodlo_id: None,
        zrodlo_odnosnik: Some("raport-1".to_string()),
        ustalono: "2026-11-03T09:00:00Z".to_string(),
        pewnosc: Pewnosc::Prawdopodobny,
    }
}

fn add_habit(room: &impl MetaRoom, params: &WpiszZwyczajParams) {
    kierownik_dzialu_zwyczaje::wpisz_zwyczaj(room, params).expect("add the habit");
}

fn confirm_habit_by_terms(room: &MemoryRoom, habit_id: i64) {
    let params = PotwierdzZwyczajParams {
        potwierdzenie: WpisyPotwierdzenie::Regulamin,
        potwierdzenie_odnosnik: Some("regulamin-portalu".to_string()),
        potwierdzil: None,
        podpis: None,
        potwierdzono: "2026-11-04".to_string(),
        id: habit_id,
    };

    kierownik_dzialu_zwyczaje::potwierdz_zwyczaj(room, &params)
        .expect("confirm the habit by the terms");
}

/// The default parameters of an offer's case in the list for Reception.
pub(crate) fn case_in_list() -> WpiszSpraweDoSpisuParams {
    WpiszSpraweDoSpisuParams {
        teczka_odnosnik: "pokoj-teczki-1".to_string(),
        rodzaj: SprawyRodzaj::Oferta,
        oznaczenie: "Oferta 2026/031, ul. Polna 7 m. 12".to_string(),
        agent: "tozsamosc-opiekun-1".to_string(),
        otwarto: "2026-11-02".to_string(),
        kancelaria_xnotariusz: None,
    }
}

/// Writes a case into the head's list of cases for Reception.
pub(crate) fn add_case_to_list(room: &impl MetaRoom, params: &WpiszSpraweDoSpisuParams) {
    kierownik_dzialu_spis::wpisz_sprawe_do_spisu(room, params).expect("add the case to the list");
}

/// The parameters of a case with a short designation, for the writes a rule refuses.
fn bare_case() -> WpiszSpraweDoSpisuParams {
    WpiszSpraweDoSpisuParams {
        oznaczenie: "Oferta".to_string(),
        agent: "tozsamosc-opiekuna".to_string(),
        ..case_in_list()
    }
}

/// The default parameters of the address of a property, a mark of the first case.
fn mark() -> DodajZnakParams {
    DodajZnakParams {
        sprawa_id: 1,
        rodzaj: ZnakiRodzaj::AdresNieruchomosci,
        wartosc: "ul. Polna 7 m. 12, Wrocław".to_string(),
        dodano: "2026-11-02T09:00:00Z".to_string(),
    }
}

fn add_mark(room: &impl MetaRoom, params: &DodajZnakParams) {
    kierownik_dzialu_spis::dodaj_znak(room, params).expect("add the mark");
}

/// The default parameters of a copy of the office's policy sent for an agency contract.
fn policy_copy() -> OdnotujRozeslaniePolisyParams {
    OdnotujRozeslaniePolisyParams {
        polisa_numer: "OC/2027/001".to_string(),
        polisa_odnosnik: Some("pokoj-kierownika-biura/polisa-2027".to_string()),
        umowa_odnosnik: "pokoj-teczki-1/umowa-posrednictwa".to_string(),
        raport_id: "raport-1".to_string(),
        rozeslano: "2026-11-05T10:00:00Z".to_string(),
    }
}

fn record_policy_copy(room: &impl MetaRoom, params: &OdnotujRozeslaniePolisyParams) {
    kierownik_dzialu_polisy::odnotuj_rozeslanie_polisy(room, params)
        .expect("record the copy of the policy");
}

fn replace_agent(room: &MemoryRoom, agent_id: &str, nastepca_id: &str) {
    let params = WymienAgentaParams {
        wymieniono: "2026-11-06".to_string(),
        powod_wymiany: "Zawiódł".to_string(),
        nastepca_id: nastepca_id.to_string(),
        id: agent_id.to_string(),
    };

    kierownik_dzialu_agenci::wymien_agenta(room, &params).expect("replace the agent");
}

fn report_request_withdrawn(
    room: &MemoryRoom,
    zgloszono_zdjecie: &str,
    powod_zdjecia: ProsbyPowodZdjecia,
) {
    let params = ZglosZdjecieProsbyParams {
        zgloszono_zdjecie: zgloszono_zdjecie.to_string(),
        powod_zdjecia,
        id: 1,
    };

    kierownik_dzialu_raporty::zglos_zdjecie_prosby(room, &params)
        .expect("report the request withdrawn");
}

/// One row in every table of the catalog, for the checks of R9 and replacement.
fn full_memory(room: &MemoryRoom) {
    appoint_agent(room, &agent("opiekun-1"));
    let_in(room, &controller_let_in());
    receive_report(
        room,
        &PrzyjmijRaportParams {
            decyzja_odnosnik: Some("pokoj-teczki-1/decyzja-1".to_string()),
            ..report("raport-1", "opiekun-1")
        },
    );
    forward_request(room, &request());
    look_into_case(
        room,
        &ZapiszZajrzenieParams {
            raport_id: Some("raport-1".to_string()),
            ..look_in()
        },
    );
    propose_standard(room, &standard_version("standard_ogloszenia", 1));
    add_source(
        room,
        &WpiszZrodloParams {
            nazwa: Some("Portal Ogłoszeń".to_string()),
            rodzaj: RodzajZrodla::Portal,
            ..source("portal")
        },
    );
    add_source_event(
        room,
        &source_event("portal", ZdarzeniaRodzaj::Potwierdzenie),
    );
    add_habit(
        room,
        &WpiszZwyczajParams {
            zrodlo_id: Some("portal".to_string()),
            ..habit()
        },
    );
    add_case_to_list(room, &case_in_list());
    add_mark(room, &mark());
    record_policy_copy(room, &policy_copy());
}

// A whole day -------------------------------------------------------------------------------------

/// kierownik powołuje agenta, przekazuje prośbę, daje Recepcji spis i wymienia agenta
#[test]
fn whole_day() {
    let room = new_room();

    appoint_agent(
        &room,
        &PowolajAgentaParams {
            zatwierdzil: Some(OWNER.to_string()),
            podpis: Some(OWNER_SIGNATURE.to_string()),
            zatwierdzono: Some(APPROVED_AT.to_string()),
            ..agent("opiekun-1")
        },
    );
    add_case_to_list(&room, &case_in_list());
    add_mark(&room, &mark());

    let cases_for_reception =
        kierownik_dzialu_spis::do_przekazania_recepcji(&room).expect("read the cases to hand");
    for case in cases_for_reception {
        let handover = OdnotujPrzekazanieSprawyParams {
            przekazano: "2026-11-02T09:10:00Z".to_string(),
            id: case.id,
        };
        kierownik_dzialu_spis::odnotuj_przekazanie_sprawy(&room, &handover)
            .expect("record the case handed to Reception");
    }

    let marks_for_reception = kierownik_dzialu_spis::znaki_do_przekazania_recepcji(&room)
        .expect("read the marks to hand");
    for mark in marks_for_reception {
        let handover = OdnotujPrzekazanieZnakuParams {
            przekazano: "2026-11-02T09:10:00Z".to_string(),
            id: mark.id,
        };
        kierownik_dzialu_spis::odnotuj_przekazanie_znaku(&room, &handover)
            .expect("record the mark handed to Reception");
    }
    let_in(&room, &controller_let_in());

    receive_report(
        &room,
        &PrzyjmijRaportParams {
            blokada: Some("Klient nie zatwierdził zdjęć".to_string()),
            najblizszy_termin: Some("2026-11-10".to_string()),
            decyzja_odnosnik: Some("pokoj-teczki-1/decyzja-1".to_string()),
            ..report("raport-1", "opiekun-1")
        },
    );
    forward_request(&room, &request());
    look_into_case(
        &room,
        &ZapiszZajrzenieParams {
            raport_id: Some("raport-1".to_string()),
            ..look_in()
        },
    );
    record_policy_copy(&room, &policy_copy());

    report_request_withdrawn(
        &room,
        "2026-11-04T12:00:00Z",
        ProsbyPowodZdjecia::Rozstrzygnieta,
    );

    appoint_agent(
        &room,
        &PowolajAgentaParams {
            powolano: "2026-11-06T09:00:00Z".to_string(),
            ..agent("opiekun-2")
        },
    );
    kierownik_dzialu_agenci::wymien_agenta(
        &room,
        &WymienAgentaParams {
            wymieniono: "2026-11-06T09:00:00Z".to_string(),
            powod_wymiany: "Opiekun przyjął polecenie z wiadomości klienta".to_string(),
            nastepca_id: "opiekun-2".to_string(),
            id: "opiekun-1".to_string(),
        },
    )
    .expect("replace the agent");
    kierownik_dzialu_spis::zmien_sprawe_w_spisie(
        &room,
        &ZmienSpraweWSpisieParams {
            oznaczenie: "Oferta 2026/031, ul. Polna 7 m. 12".to_string(),
            agent: "tozsamosc-opiekun-2".to_string(),
            kancelaria_xnotariusz: None,
            id: 1,
        },
    )
    .expect("change the case in the list");

    let changed_cases =
        kierownik_dzialu_spis::do_przekazania_recepcji(&room).expect("read the cases to hand");
    let agents_of_changed_cases: Vec<&str> = changed_cases
        .iter()
        .map(|case| case.agent.as_str())
        .collect();
    assert_eq!(agents_of_changed_cases, ["tozsamosc-opiekun-2"]);

    let agents = kierownik_dzialu_agenci::agenci(&room).expect("read the agents");
    let agent_ids: Vec<&str> = agents.iter().map(|agent| agent.id.as_str()).collect();
    assert_eq!(agent_ids, ["opiekun-2", "opiekun-1"]);

    let listed_requests =
        kierownik_dzialu_raporty::prosby_na_liscie(&room).expect("read the listed requests");
    assert!(listed_requests.is_empty());

    let look_ins = kierownik_dzialu_zajrzenia::zajrzenia_od(
        &room,
        &ZajrzeniaOdParams {
            zajrzano: "2026-11-01".to_string(),
        },
    )
    .expect("read the look-ins");
    assert_eq!(look_ins.len(), 1);

    let open_let_ins =
        kierownik_dzialu_agenci::wpuszczenia_otwarte(&room).expect("read the open let-ins");
    let let_in_identities: Vec<&str> = open_let_ins
        .iter()
        .map(|row| row.tozsamosc.as_str())
        .collect();
    assert_eq!(let_in_identities, ["tozsamosc-kontrolera"]);

    let policy_copies = kierownik_dzialu_polisy::rozeslania_polisy(
        &room,
        &RozeslaniaPolisyParams {
            polisa_numer: "OC/2027/001".to_string(),
        },
    )
    .expect("read the copies of the policy");
    assert_eq!(policy_copies.len(), 1);
}

// Agents ------------------------------------------------------------------------------------------

/// agent sprawy ma teczkę, a Koordynator operacji i Kontroler jej nie mają
#[test]
fn case_file_by_position() {
    let room = new_room();
    appoint_agent(&room, &agent("opiekun"));
    appoint_agent(
        &room,
        &PowolajAgentaParams {
            stanowisko: WpisyStanowisko::KoordynatorOperacji,
            teczka_odnosnik: None,
            ..agent("koordynator")
        },
    );
    appoint_agent(&room, &controller("kontroler"));

    let rule = "teczka_poza_koordynatorem_operacji_i_kontrolerem";
    let manager_without_file = PowolajAgentaParams {
        id: "bez-teczki".to_string(),
        stanowisko: WpisyStanowisko::ZarzadcaObiektu,
        tozsamosc: "t".to_string(),
        wlasciciel: "w".to_string(),
        teczka_odnosnik: None,
        klient_odnosnik: None,
        powolano: "2026-11-02".to_string(),
        zatwierdzil: None,
        podpis: None,
        zatwierdzono: None,
    };
    expect::refused(
        kierownik_dzialu_agenci::powolaj_agenta(&room, &manager_without_file),
        rule,
    );

    let coordinator_with_file = PowolajAgentaParams {
        id: "koordynator-z-teczka".to_string(),
        stanowisko: WpisyStanowisko::KoordynatorOperacji,
        teczka_odnosnik: Some("pokoj-lokalu-1".to_string()),
        ..manager_without_file
    };
    expect::refused(
        kierownik_dzialu_agenci::powolaj_agenta(&room, &coordinator_with_file),
        rule,
    );
}

/// Kontroler ma klienta, a inni agenci go nie mają
#[test]
fn controller_has_client() {
    let room = new_room();
    appoint_agent(&room, &controller("kontroler"));

    let controller_without_client = PowolajAgentaParams {
        id: "kontroler-bez-klienta".to_string(),
        stanowisko: WpisyStanowisko::Kontroler,
        tozsamosc: "t".to_string(),
        wlasciciel: "w".to_string(),
        teczka_odnosnik: None,
        klient_odnosnik: None,
        powolano: "2026-11-02".to_string(),
        zatwierdzil: None,
        podpis: None,
        zatwierdzono: None,
    };
    expect::refused(
        kierownik_dzialu_agenci::powolaj_agenta(&room, &controller_without_client),
        "klient_tylko_u_kontrolera",
    );

    let carer_with_client = PowolajAgentaParams {
        id: "opiekun-z-klientem".to_string(),
        stanowisko: WpisyStanowisko::OpiekunOferty,
        teczka_odnosnik: Some("pokoj-teczki-1".to_string()),
        klient_odnosnik: Some("pokoj-kartoteki/klient-nowak".to_string()),
        ..controller_without_client
    };
    expect::refused(
        kierownik_dzialu_agenci::powolaj_agenta(&room, &carer_with_client),
        "klient_tylko_u_kontrolera",
    );

    let client_agents = kierownik_dzialu_agenci::agenci_klienta(
        &room,
        &AgenciKlientaParams {
            klient_odnosnik: "pokoj-kartoteki/klient-nowak".to_string(),
        },
    )
    .expect("read the agents of the client");
    let client_agent_ids: Vec<&str> = client_agents.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(client_agent_ids, ["kontroler"]);
}

/// agent powstaje pracujący, a jego powołania się nie zmienia
#[test]
fn agent_starts_working() {
    let room = new_room();
    appoint_agent(&room, &agent("opiekun"));
    kierownik_dzialu_agenci::uspij_agenta(
        &room,
        &UspijAgentaParams {
            id: "opiekun".to_string(),
        },
    )
    .expect("put the agent to sleep");
    kierownik_dzialu_agenci::obudz_agenta(
        &room,
        &ObudzAgentaParams {
            id: "opiekun".to_string(),
        },
    )
    .expect("wake the agent");

    expect::refused(
        room.execute(
            "INSERT INTO kierownik_dzialu_agenci_wpisy (id, stanowisko, tozsamosc, wlasciciel,
              teczka_odnosnik, powolano, stan)
            VALUES ('spiacy', 'opiekun_oferty', 't', 'w', 'pokoj-teczki-1', '2026-11-02', 'spi')",
            &[],
        ),
        "agent powstaje jako pracujący",
    );

    let changes = [
        ("stanowisko", "'opiekun_najmu'"),
        ("teczka_odnosnik", "'pokoj-teczki-2'"),
        ("tozsamosc", "'inna'"),
    ];
    for (column, value) in changes {
        let change = format!(
            "UPDATE kierownik_dzialu_agenci_wpisy SET {column} = {value} WHERE id = 'opiekun'"
        );
        expect::refused(
            room.execute(&change, &[]),
            "stanowiska, tożsamości, teczki, klienta, daty powołania ani zatwierdzenia agenta się \
            nie zmienia",
        );
    }
}

/// wymiana wymaga powodu, daty i następcy i jest ostateczna
#[test]
fn replacement_needs_reason() {
    let room = new_room();
    appoint_agent(&room, &agent("opiekun-1"));
    appoint_agent(&room, &agent("opiekun-2"));

    // A writer outside the client can leave out the reason that `wymien_agenta` requires. The
    // table refuses the replacement without one.
    let replacement_without_reason = room.execute(
        "UPDATE kierownik_dzialu_agenci_wpisy
        SET stan = 'wymieniony',
          wymieniono = :wymieniono,
          powod_wymiany = :powod_wymiany,
          nastepca_id = :nastepca_id
        WHERE id = :id",
        &[
            ("wymieniono", text("2026-11-06")),
            ("powod_wymiany", SqlValue::Null),
            ("nastepca_id", text("opiekun-2")),
            ("id", text("opiekun-1")),
        ],
    );
    expect::refused(
        replacement_without_reason,
        "wymiana_z_powodem_data_i_nastepca",
    );
    replace_agent(&room, "opiekun-1", "opiekun-2");

    let wake_replaced = ObudzAgentaParams {
        id: "opiekun-1".to_string(),
    };
    expect::refused(
        kierownik_dzialu_agenci::obudz_agenta(&room, &wake_replaced),
        "wymienionego agenta się nie zmienia",
    );
}

/// następca ma to samo stanowisko i tę samą teczkę, a Kontroler tego samego klienta
#[test]
fn successor_takes_same_work() {
    let room = new_room();
    appoint_agent(&room, &agent("opiekun-1"));
    appoint_agent(
        &room,
        &PowolajAgentaParams {
            teczka_odnosnik: Some("pokoj-teczki-2".to_string()),
            ..agent("inna-teczka")
        },
    );
    appoint_agent(
        &room,
        &PowolajAgentaParams {
            stanowisko: WpisyStanowisko::OpiekunPoszukujacego,
            ..agent("inne-stanowisko")
        },
    );
    appoint_agent(&room, &controller("kontroler-1"));
    appoint_agent(
        &room,
        &PowolajAgentaParams {
            klient_odnosnik: Some("pokoj-kartoteki/klient-kowalski".to_string()),
            ..controller("kontroler-innego")
        },
    );
    appoint_agent(&room, &controller("kontroler-2"));

    let rule = "następca ma to samo stanowisko i tę samą teczkę";
    for successor in ["inna-teczka", "inne-stanowisko", "opiekun-1"] {
        let replacement = WymienAgentaParams {
            wymieniono: "2026-11-06".to_string(),
            powod_wymiany: "Zawiódł".to_string(),
            nastepca_id: successor.to_string(),
            id: "opiekun-1".to_string(),
        };
        expect::refused(
            kierownik_dzialu_agenci::wymien_agenta(&room, &replacement),
            rule,
        );
    }

    let controller_of_other_client = WymienAgentaParams {
        wymieniono: "2026-11-06".to_string(),
        powod_wymiany: "Zawiódł".to_string(),
        nastepca_id: "kontroler-innego".to_string(),
        id: "kontroler-1".to_string(),
    };
    expect::refused(
        kierownik_dzialu_agenci::wymien_agenta(&room, &controller_of_other_client),
        rule,
    );
    replace_agent(&room, "kontroler-1", "kontroler-2");
}

/// zatwierdzenie powołania ma komplet kolumn i nie zmienia się po wpisaniu
#[test]
fn appointment_approval_complete() {
    let room = new_room();
    appoint_agent(&room, &agent("opiekun-1"));

    // A writer outside the client can leave out the signature that `zatwierdz_powolanie`
    // requires. The table refuses the approval without one.
    let approval_without_signature = room.execute(
        "UPDATE kierownik_dzialu_agenci_wpisy
        SET zatwierdzil = :zatwierdzil,
          podpis = :podpis,
          zatwierdzono = :zatwierdzono
        WHERE id = :id",
        &[
            ("zatwierdzil", text(OWNER)),
            ("podpis", SqlValue::Null),
            ("zatwierdzono", text("2026-11-02")),
            ("id", text("opiekun-1")),
        ],
    );
    expect::refused(approval_without_signature, "zatwierdzenie_w_komplecie");
    kierownik_dzialu_agenci::zatwierdz_powolanie(
        &room,
        &appointment_approval("opiekun-1", APPROVED_AT),
    )
    .expect("approve the appointment");
    expect::refused(
        kierownik_dzialu_agenci::zatwierdz_powolanie(
            &room,
            &appointment_approval("opiekun-1", "2026-11-03"),
        ),
        "stanowiska, tożsamości, teczki, klienta, daty powołania ani zatwierdzenia agenta się nie \
        zmienia",
    );
}

// Let-ins -----------------------------------------------------------------------------------------

/// jedno otwarte wpuszczenie tożsamości do pokoju; nowe po usunięciu
#[test]
fn one_open_let_in_per_room() {
    let room = new_room();
    let_in(&room, &controller_let_in());
    let_in(
        &room,
        &OdnotujWpuszczenieParams {
            pokoj_odnosnik: "pokoj-teczki-2".to_string(),
            ..controller_let_in()
        },
    );

    let rule = "ta tożsamość ma już otwarte wpuszczenie do tego pokoju";
    expect::refused(
        kierownik_dzialu_agenci::odnotuj_wpuszczenie(&room, &unrequested_let_in()),
        rule,
    );
    kierownik_dzialu_agenci::odnotuj_usuniecie_z_pokoju(
        &room,
        &OdnotujUsuniecieZPokojuParams {
            usunieto: "2026-11-05T10:00:00Z".to_string(),
            id: 1,
        },
    )
    .expect("remove the identity from the room");
    let_in(
        &room,
        &OdnotujWpuszczenieParams {
            wpuszczono: "2026-11-06T10:00:00Z".to_string(),
            ..controller_let_in()
        },
    );

    let room_let_ins = kierownik_dzialu_agenci::wpuszczenia_pokoju(
        &room,
        &WpuszczeniaPokojuParams {
            pokoj_odnosnik: "pokoj-teczki-1".to_string(),
        },
    )
    .expect("read the let-ins of the room");
    assert_eq!(room_let_ins.len(), 2);
}

/// wpuszczenie: Kontroler z klientem, prośba z proszącym, usunięcie po wpuszczeniu
#[test]
fn let_in_fields_go_together() {
    let room = new_room();
    let_in(&room, &controller_let_in());

    let controller_without_client = OdnotujWpuszczenieParams {
        pokoj_odnosnik: "pokoj-teczki-2".to_string(),
        klient_odnosnik: None,
        ..unrequested_let_in()
    };
    expect::refused(
        kierownik_dzialu_agenci::odnotuj_wpuszczenie(&room, &controller_without_client),
        "klient_tylko_u_kontrolera",
    );

    let requester_without_request = OdnotujWpuszczenieParams {
        pokoj_odnosnik: "pokoj-teczki-2".to_string(),
        prosil: Some("tozsamosc-kierownika-kontroli".to_string()),
        ..unrequested_let_in()
    };
    expect::refused(
        kierownik_dzialu_agenci::odnotuj_wpuszczenie(&room, &requester_without_request),
        "prosba_razem_z_proszacym",
    );

    let removal_before_let_in = OdnotujUsuniecieZPokojuParams {
        usunieto: "2026-11-01T00:00:00Z".to_string(),
        id: 1,
    };
    expect::refused(
        kierownik_dzialu_agenci::odnotuj_usuniecie_z_pokoju(&room, &removal_before_let_in),
        "usuniecie_nie_przed_wpuszczeniem",
    );

    // A writer outside the client can leave out the signature that `zatwierdz_wpuszczenie`
    // requires. The table refuses the approval without one.
    let approval_without_signature = room.execute(
        "UPDATE kierownik_dzialu_agenci_wpuszczenia
        SET zatwierdzil = :zatwierdzil,
          podpis = :podpis,
          zatwierdzono = :zatwierdzono
        WHERE id = :id",
        &[
            ("zatwierdzil", text(OWNER)),
            ("podpis", SqlValue::Null),
            ("zatwierdzono", text("2026-11-02")),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(approval_without_signature, "zatwierdzenie_w_komplecie");
}

/// wpuszczenie własnego agenta ma jego tożsamość, stanowisko i klienta
#[test]
fn own_agent_let_in_matches() {
    let room = new_room();
    appoint_agent(
        &room,
        &PowolajAgentaParams {
            stanowisko: WpisyStanowisko::KoordynatorTransakcji,
            ..agent("koordynator")
        },
    );

    let own_agent_let_in = OdnotujWpuszczenieParams {
        tozsamosc: "tozsamosc-koordynator".to_string(),
        stanowisko: WpisyStanowisko::KoordynatorTransakcji,
        agent_id: Some("koordynator".to_string()),
        pokoj_odnosnik: "pokoj-wymiany-1".to_string(),
        klient_odnosnik: None,
        prosil: None,
        prosba_odnosnik: None,
        wpuscil: "tozsamosc-kierownika-transakcji".to_string(),
        wpuszczono: "2026-11-03T10:00:00Z".to_string(),
        zatwierdzil: None,
        podpis: None,
        zatwierdzono: None,
    };
    kierownik_dzialu_agenci::odnotuj_wpuszczenie(&room, &own_agent_let_in)
        .expect("let the own agent in");

    let let_in_of_another_identity = OdnotujWpuszczenieParams {
        tozsamosc: "tozsamosc-kogos-innego".to_string(),
        pokoj_odnosnik: "pokoj-wymiany-2".to_string(),
        ..own_agent_let_in
    };
    expect::refused(
        kierownik_dzialu_agenci::odnotuj_wpuszczenie(&room, &let_in_of_another_identity),
        "wpuszczenie własnego agenta ma jego tożsamość, stanowisko i klienta",
    );
}

/// wpuszczenie powstaje otwarte, a usunięcie i zatwierdzenie odnotowuje się raz
#[test]
fn let_in_is_a_trace() {
    let room = new_room();
    let_in(&room, &controller_let_in());
    kierownik_dzialu_agenci::zatwierdz_wpuszczenie(&room, &let_in_approval(1, APPROVED_AT))
        .expect("approve the let-in");
    kierownik_dzialu_agenci::odnotuj_usuniecie_z_pokoju(
        &room,
        &OdnotujUsuniecieZPokojuParams {
            usunieto: "2026-11-05T10:00:00Z".to_string(),
            id: 1,
        },
    )
    .expect("remove the identity from the room");

    let rule = "wpuszczenia się nie zmienia";
    let second_removal = OdnotujUsuniecieZPokojuParams {
        usunieto: "2026-11-06T10:00:00Z".to_string(),
        id: 1,
    };
    expect::refused(
        kierownik_dzialu_agenci::odnotuj_usuniecie_z_pokoju(&room, &second_removal),
        rule,
    );
    expect::refused(
        kierownik_dzialu_agenci::zatwierdz_wpuszczenie(&room, &let_in_approval(1, "2026-11-03")),
        rule,
    );
    expect::refused(
        room.execute(
            "UPDATE kierownik_dzialu_agenci_wpuszczenia SET wpuscil = 'ktos-inny'",
            &[],
        ),
        rule,
    );
    expect::refused(
        room.execute(
            "INSERT INTO kierownik_dzialu_agenci_wpuszczenia (tozsamosc, stanowisko, pokoj_odnosnik,
              wpuscil, wpuszczono, usunieto)
            VALUES ('t', 'opiekun_najmu', 'pokoj-teczki-3', 'k', '2026-11-03', '2026-11-04')",
            &[],
        ),
        "wpuszczenie powstaje otwarte",
    );
}

// Reports and requests ----------------------------------------------------------------------------

/// raporty tylko się dopisuje
#[test]
fn reports_are_appended() {
    let room = new_room();
    appoint_agent(&room, &agent("opiekun-1"));
    receive_report(&room, &report("raport-1", "opiekun-1"));

    expect::refused(
        room.execute(
            "UPDATE kierownik_dzialu_raporty_wpisy SET etap_albo_okres = 'koniec'",
            &[],
        ),
        "raporty tylko się dopisuje",
    );

    let report_of_stranger = PrzyjmijRaportParams {
        otrzymano: "2026-11-03".to_string(),
        ..report("raport-obcego", "nieznany")
    };
    expect::refused(
        kierownik_dzialu_raporty::przyjmij_raport(&room, &report_of_stranger),
        "FOREIGN KEY constraint failed",
    );
}

/// prośby nie przekazuje się drugi raz, dopóki nie zgłoszono jej zdjęcia z powodem
#[test]
fn request_forwarded_once() {
    let room = new_room();
    appoint_agent(&room, &agent("opiekun-1"));
    receive_report(&room, &report("raport-1", "opiekun-1"));
    forward_request(&room, &request());

    let second_forward = PrzekazProsbeParams {
        przekazano: "2026-11-04T09:00:00Z".to_string(),
        ..request()
    };
    expect::refused(
        kierownik_dzialu_raporty::przekaz_prosbe(&room, &second_forward),
        "tej prośby nie przekazuje się drugi raz",
    );

    // A writer outside the client can leave out the reason that `zglos_zdjecie_prosby` requires.
    // The table refuses the withdrawal without one.
    let withdrawal_without_reason = room.execute(
        "UPDATE kierownik_dzialu_raporty_prosby
        SET zgloszono_zdjecie = :zgloszono_zdjecie,
          powod_zdjecia = :powod_zdjecia
        WHERE id = :id",
        &[
            ("zgloszono_zdjecie", text("2026-11-04T09:00:00Z")),
            ("powod_zdjecia", SqlValue::Null),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(withdrawal_without_reason, "zdjecie_z_powodem");

    let withdrawal_before_forward = ZglosZdjecieProsbyParams {
        zgloszono_zdjecie: "2026-11-01T09:00:00Z".to_string(),
        powod_zdjecia: ProsbyPowodZdjecia::Zbedna,
        id: 1,
    };
    expect::refused(
        kierownik_dzialu_raporty::zglos_zdjecie_prosby(&room, &withdrawal_before_forward),
        "zdjecie_nie_przed_przekazaniem",
    );
    report_request_withdrawn(&room, "2026-11-04T09:00:00Z", ProsbyPowodZdjecia::Zbedna);
    forward_request(
        &room,
        &PrzekazProsbeParams {
            przekazano_do: ProsbyPrzekazanoDo::Wlasciciel,
            przekazano: "2026-11-05T09:00:00Z".to_string(),
            ..request()
        },
    );

    let listed_requests =
        kierownik_dzialu_raporty::prosby_na_liscie(&room).expect("read the listed requests");
    let recipients: Vec<ProsbyPrzekazanoDo> = listed_requests
        .iter()
        .map(|row| row.przekazano_do)
        .collect();
    assert_eq!(recipients, [ProsbyPrzekazanoDo::Wlasciciel]);
}

/// przekazana prośba powstaje na liście, a po zdjęciu się jej nie zmienia
#[test]
fn request_starts_on_list() {
    let room = new_room();
    appoint_agent(&room, &agent("opiekun-1"));
    receive_report(&room, &report("raport-1", "opiekun-1"));
    forward_request(&room, &request());
    report_request_withdrawn(
        &room,
        "2026-11-04T09:00:00Z",
        ProsbyPowodZdjecia::Rozstrzygnieta,
    );

    let second_withdrawal = ZglosZdjecieProsbyParams {
        zgloszono_zdjecie: "2026-11-05T09:00:00Z".to_string(),
        powod_zdjecia: ProsbyPowodZdjecia::Zbedna,
        id: 1,
    };
    expect::refused(
        kierownik_dzialu_raporty::zglos_zdjecie_prosby(&room, &second_withdrawal),
        "przekazanej prośby się nie zmienia",
    );
    expect::refused(
        room.execute(
            "INSERT INTO kierownik_dzialu_raporty_prosby (decyzja_odnosnik, raport_id,
              przekazano_do, przekazano, zgloszono_zdjecie, powod_zdjecia)
            VALUES ('pokoj-teczki-1/decyzja-2', 'raport-1', 'kierownik_biura', '2026-11-03',
              '2026-11-04', 'zbedna')",
            &[],
        ),
        "przekazana prośba powstaje na liście",
    );
}

// Look-ins ----------------------------------------------------------------------------------------

/// zajrzenie ma jeden z czterech powodów i opis, a potem się go nie zmienia
#[test]
fn look_in_is_fixed() {
    let room = new_room();
    look_into_case(&room, &look_in());

    // A writer outside the client can give the reason „ciekawosc”, which `WpisyPowod` leaves out.
    // The table refuses the look-in with it.
    let look_in_out_of_curiosity = room.execute(
        "INSERT INTO kierownik_dzialu_zajrzenia_wpisy (teczka_odnosnik, zajrzano, powod, opis,
          raport_id)
        VALUES (:teczka_odnosnik, :zajrzano, :powod, :opis, :raport_id)",
        &[
            ("teczka_odnosnik", text("pokoj-teczki-1")),
            ("zajrzano", text("2026-11-04")),
            ("powod", text("ciekawosc")),
            ("opis", text("Bez powodu")),
            ("raport_id", SqlValue::Null),
        ],
    );
    expect::refused(
        look_in_out_of_curiosity,
        "CHECK constraint failed: powod IN",
    );

    // A writer outside the client can leave out the description that `zapisz_zajrzenie` requires.
    // The table refuses the look-in without one.
    let look_in_without_description = room.execute(
        "INSERT INTO kierownik_dzialu_zajrzenia_wpisy (teczka_odnosnik, zajrzano, powod, opis,
          raport_id)
        VALUES (:teczka_odnosnik, :zajrzano, :powod, :opis, :raport_id)",
        &[
            ("teczka_odnosnik", text("pokoj-teczki-1")),
            ("zajrzano", text("2026-11-04")),
            ("powod", text("sprawdzenie_jakosci")),
            ("opis", SqlValue::Null),
            ("raport_id", SqlValue::Null),
        ],
    );
    expect::refused(
        look_in_without_description,
        "NOT NULL constraint failed: kierownik_dzialu_zajrzenia_wpisy.opis",
    );
    expect::refused(
        room.execute(
            "UPDATE kierownik_dzialu_zajrzenia_wpisy SET opis = 'Inny powód'",
            &[],
        ),
        "zajrzenia się nie zmienia",
    );
}

// Standards ---------------------------------------------------------------------------------------

/// wersja standardu powstaje jako propozycja, a numer jest jeden w standardzie
#[test]
fn standard_starts_as_proposal() {
    let room = new_room();
    propose_standard(&room, &standard_version("standard_ogloszenia", 1));
    propose_standard(&room, &standard_version("wzor_umowy_posrednictwa", 1));

    let same_number_again = ZaproponujStandardParams {
        standard: "standard_ogloszenia".to_string(),
        numer: 1,
        sciezka: "s".to_string(),
        skrot: "k".to_string(),
        opis_zmian: "o".to_string(),
        zaproponowano: "2026-11-01".to_string(),
    };
    expect::refused(
        kierownik_dzialu_standardy::zaproponuj_standard(&room, &same_number_again),
        REPLACEMENT,
    );
    expect::refused(
        room.execute(
            "INSERT INTO kierownik_dzialu_standardy_wersje (standard, numer, sciezka, skrot,
              opis_zmian, zaproponowano, rozstrzygniecie, podpisal, podpisal_jako, podpis,
              podpisano)
            VALUES ('standard_ogloszenia', 2, 's', 'k', 'o', '2026-11-01', 'zatwierdzona', 'w',
              'wlasciciel', 'p', '2026-11-02')",
            &[],
        ),
        "wersja standardu powstaje jako propozycja",
    );
}

/// rozstrzygnięcie standardu wymaga kompletu podpisu i jest ostateczne
#[test]
fn standard_decision_signed_and_final() {
    let room = new_room();
    propose_standard(&room, &standard_version("standard_ogloszenia", 1));

    // A writer outside the client can leave out the signature that `rozstrzygnij_standard`
    // requires. The table refuses the decision without one.
    let decision_without_signature = room.execute(
        "UPDATE kierownik_dzialu_standardy_wersje
        SET rozstrzygniecie = :rozstrzygniecie,
          podpisal = :podpisal,
          podpisal_jako = :podpisal_jako,
          podpis = :podpis,
          podpisano = :podpisano
        WHERE id = :id",
        &[
            ("rozstrzygniecie", text("zatwierdzona")),
            ("podpisal", text(OWNER)),
            ("podpisal_jako", text("wlasciciel")),
            ("podpis", SqlValue::Null),
            ("podpisano", text("2026-11-02")),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(decision_without_signature, "podpis_w_komplecie");

    let decision_before_proposal = RozstrzygnijStandardParams {
        podpisano: "2026-10-01".to_string(),
        ..standard_decision(1, WersjeRozstrzygniecie::Zatwierdzona)
    };
    expect::refused(
        kierownik_dzialu_standardy::rozstrzygnij_standard(&room, &decision_before_proposal),
        "rozstrzygniecie_nie_przed_propozycja",
    );
    expect::refused(
        room.execute(
            "UPDATE kierownik_dzialu_standardy_wersje SET skrot = 'inny' WHERE id = 1",
            &[],
        ),
        "rozstrzygniętej wersji standardu się nie zmienia",
    );
    decide_standard(
        &room,
        &RozstrzygnijStandardParams {
            podpisal_jako: WersjePodpisalJako::WlascicielBiura,
            ..standard_decision(1, WersjeRozstrzygniecie::Zatwierdzona)
        },
    );
    expect::refused(
        kierownik_dzialu_standardy::rozstrzygnij_standard(
            &room,
            &standard_decision(1, WersjeRozstrzygniecie::Odrzucona),
        ),
        "rozstrzygniętej wersji standardu się nie zmienia",
    );
}

/// obowiązuje najnowsza zatwierdzona wersja każdego standardu
#[test]
fn current_standards() {
    let room = new_room();
    propose_standard(&room, &standard_version("standard_ogloszenia", 1));
    propose_standard(&room, &standard_version("standard_ogloszenia", 2));
    propose_standard(&room, &standard_version("standard_ogloszenia", 3));
    propose_standard(&room, &standard_version("parametry_programu", 1));
    decide_standard(
        &room,
        &standard_decision(1, WersjeRozstrzygniecie::Zatwierdzona),
    );
    decide_standard(
        &room,
        &standard_decision(2, WersjeRozstrzygniecie::Zatwierdzona),
    );
    decide_standard(
        &room,
        &standard_decision(3, WersjeRozstrzygniecie::Odrzucona),
    );
    decide_standard(
        &room,
        &standard_decision(4, WersjeRozstrzygniecie::Zatwierdzona),
    );

    let current_standards =
        kierownik_dzialu_standardy::obowiazujace_standardy(&room).expect("read current standards");
    let current: HashMap<&str, i64> = current_standards
        .iter()
        .map(|row| (row.standard.as_str(), row.numer))
        .collect();
    assert_eq!(
        current,
        HashMap::from([("parametry_programu", 1), ("standard_ogloszenia", 2)])
    );

    let versions = kierownik_dzialu_standardy::wersje_standardu(
        &room,
        &WersjeStandarduParams {
            standard: "standard_ogloszenia".to_string(),
        },
    )
    .expect("read the versions of the standard");
    let version_numbers: Vec<i64> = versions.iter().map(|row| row.numer).collect();
    assert_eq!(version_numbers, [3, 2, 1]);
}

// Habits ------------------------------------------------------------------------------------------

/// zwyczaj ma źródło i powstaje niepotwierdzony, a pewny wymaga potwierdzenia
#[test]
fn habit_needs_source_and_confirmation() {
    let room = new_room();
    add_habit(&room, &habit());

    let habit_without_source = WpiszZwyczajParams {
        kontrahent_rodzaj: WpisyKontrahentRodzaj::Bank,
        kontrahent: "Bank".to_string(),
        temat: "wypłata".to_string(),
        opis: "Wypłaca po wypisie aktu".to_string(),
        zrodlo_id: None,
        zrodlo_odnosnik: None,
        ustalono: "2026-11-03".to_string(),
        pewnosc: Pewnosc::Niepewny,
    };
    expect::refused(
        kierownik_dzialu_zwyczaje::wpisz_zwyczaj(&room, &habit_without_source),
        "zwyczaj_ze_zrodlem",
    );

    // A writer outside the client can give the certainty „pewny”, which `Pewnosc` leaves out. The
    // table refuses a certain habit without a confirmation.
    let certain_habit = room.execute(
        "INSERT INTO kierownik_dzialu_zwyczaje_wpisy (kontrahent_rodzaj, kontrahent, temat, opis,
          zrodlo_id, zrodlo_odnosnik, ustalono, pewnosc)
        VALUES (:kontrahent_rodzaj, :kontrahent, :temat, :opis, :zrodlo_id, :zrodlo_odnosnik,
          :ustalono, :pewnosc)",
        &[
            ("kontrahent_rodzaj", text("bank")),
            ("kontrahent", text("Bank")),
            ("temat", text("wypłata")),
            ("opis", text("Wypłaca po wypisie aktu")),
            ("zrodlo_id", SqlValue::Null),
            ("zrodlo_odnosnik", text("raport-2")),
            ("ustalono", text("2026-11-03")),
            ("pewnosc", text("pewny")),
        ],
    );
    expect::refused(certain_habit, "pewny_z_potwierdzeniem");
    expect::refused(
        room.execute(
            "INSERT INTO kierownik_dzialu_zwyczaje_wpisy (kontrahent_rodzaj, kontrahent, temat,
              opis, zrodlo_odnosnik, ustalono, pewnosc, potwierdzenie, potwierdzenie_odnosnik,
              potwierdzono)
            VALUES ('portal', 'Portal', 't', 'o', 'r', '2026-11-03', 'pewny', 'regulamin',
              'regulamin-portalu', '2026-11-03')",
            &[],
        ),
        "zwyczaj powstaje niepotwierdzony i niewykreślony",
    );
}

/// zwyczaj potwierdza regulamin albo kontrahent z odnośnikiem, a człowiek biura podpisem
#[test]
fn habit_confirmation_kinds() {
    let room = new_room();
    add_habit(&room, &habit());
    add_habit(
        &room,
        &WpiszZwyczajParams {
            kontrahent_rodzaj: WpisyKontrahentRodzaj::Bank,
            kontrahent: "Bank Przykładowy".to_string(),
            temat: "wypłata".to_string(),
            ..habit()
        },
    );

    let terms_without_reference = PotwierdzZwyczajParams {
        potwierdzenie: WpisyPotwierdzenie::Regulamin,
        potwierdzenie_odnosnik: None,
        potwierdzil: None,
        podpis: None,
        potwierdzono: "2026-11-04".to_string(),
        id: 1,
    };
    expect::refused(
        kierownik_dzialu_zwyczaje::potwierdz_zwyczaj(&room, &terms_without_reference),
        "potwierdzenie_regulaminem_albo_kontrahenta_wskazane",
    );

    let human_without_signature = PotwierdzZwyczajParams {
        potwierdzenie: WpisyPotwierdzenie::CzlowiekBiura,
        potwierdzenie_odnosnik: None,
        potwierdzil: Some("tozsamosc-pośrednika".to_string()),
        podpis: None,
        potwierdzono: "2026-11-04".to_string(),
        id: 2,
    };
    expect::refused(
        kierownik_dzialu_zwyczaje::potwierdz_zwyczaj(&room, &human_without_signature),
        "potwierdzenie_czlowieka_biura_z_podpisem",
    );

    let terms_with_signature = PotwierdzZwyczajParams {
        potwierdzenie: WpisyPotwierdzenie::Regulamin,
        potwierdzenie_odnosnik: Some("regulamin-portalu".to_string()),
        potwierdzil: Some("tozsamosc-pośrednika".to_string()),
        podpis: Some("podpis".to_string()),
        potwierdzono: "2026-11-04".to_string(),
        id: 1,
    };
    expect::refused(
        kierownik_dzialu_zwyczaje::potwierdz_zwyczaj(&room, &terms_with_signature),
        "potwierdzenie_czlowieka_biura_z_podpisem",
    );

    // A writer outside the client can leave out the date that `potwierdz_zwyczaj` requires. The
    // table refuses the confirmation without one.
    let confirmation_without_date = room.execute(
        "UPDATE kierownik_dzialu_zwyczaje_wpisy
        SET pewnosc = 'pewny',
          potwierdzenie = :potwierdzenie,
          potwierdzenie_odnosnik = :potwierdzenie_odnosnik,
          potwierdzil = :potwierdzil,
          podpis = :podpis,
          potwierdzono = :potwierdzono
        WHERE id = :id",
        &[
            ("potwierdzenie", text("regulamin")),
            ("potwierdzenie_odnosnik", text("regulamin-portalu")),
            ("potwierdzil", SqlValue::Null),
            ("podpis", SqlValue::Null),
            ("potwierdzono", SqlValue::Null),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(confirmation_without_date, "potwierdzenie_z_data");

    confirm_habit_by_terms(&room, 1);
    kierownik_dzialu_zwyczaje::potwierdz_zwyczaj(
        &room,
        &PotwierdzZwyczajParams {
            potwierdzenie: WpisyPotwierdzenie::CzlowiekBiura,
            potwierdzenie_odnosnik: None,
            potwierdzil: Some(OWNER.to_string()),
            podpis: Some(OWNER_SIGNATURE.to_string()),
            potwierdzono: "2026-11-04".to_string(),
            id: 2,
        },
    )
    .expect("confirm the habit by a human of the office");

    let portal_habits = kierownik_dzialu_zwyczaje::zwyczaje_kontrahenta(
        &room,
        &ZwyczajeKontrahentaParams {
            kontrahent: "Portal Ogłoszeń".to_string(),
        },
    )
    .expect("read the habits of the portal");
    assert_eq!(portal_habits[0].pewnosc, PewnoscZwyczaju::Pewny);
}

/// zakwestionowany zwyczaj traci potwierdzenie, a wykreślonego się nie zmienia
#[test]
fn habit_questioned_and_struck() {
    let room = new_room();
    add_habit(&room, &habit());
    confirm_habit_by_terms(&room, 1);
    kierownik_dzialu_zwyczaje::zakwestionuj_zwyczaj(
        &room,
        &ZakwestionujZwyczajParams {
            pewnosc: Pewnosc::Niepewny,
            id: 1,
        },
    )
    .expect("question the habit");

    let confirmation = room
        .value(
            "SELECT potwierdzenie FROM kierownik_dzialu_zwyczaje_wpisy WHERE id = 1",
            &[],
        )
        .expect("read the confirmation");
    let confirmation =
        Option::<String>::from_sql_value(&confirmation).expect("the confirmation is text or NULL");
    assert_eq!(confirmation, None);

    expect::refused(
        room.execute(
            "UPDATE kierownik_dzialu_zwyczaje_wpisy SET opis = 'Usuwa po 60 dniach'",
            &[],
        ),
        "kontrahenta, treści, źródła ani daty zwyczaju się nie zmienia",
    );

    // A writer outside the client can leave out the reason that `wykresl_zwyczaj` requires. The
    // table refuses the striking without one.
    let striking_without_reason = room.execute(
        "UPDATE kierownik_dzialu_zwyczaje_wpisy
        SET wykreslono = :wykreslono,
          powod_wykreslenia = :powod_wykreslenia
        WHERE id = :id",
        &[
            ("wykreslono", text("2026-11-05")),
            ("powod_wykreslenia", SqlValue::Null),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(striking_without_reason, "wykreslenie_z_powodem");
    kierownik_dzialu_zwyczaje::wykresl_zwyczaj(
        &room,
        &WykreslZwyczajParams {
            wykreslono: "2026-11-05".to_string(),
            powod_wykreslenia: "Portal zmienił regulamin".to_string(),
            id: 1,
        },
    )
    .expect("strike the habit");

    let question_struck = ZakwestionujZwyczajParams {
        pewnosc: Pewnosc::Prawdopodobny,
        id: 1,
    };
    expect::refused(
        kierownik_dzialu_zwyczaje::zakwestionuj_zwyczaj(&room, &question_struck),
        "wykreślonego zwyczaju się nie zmienia",
    );

    let portal_habits = kierownik_dzialu_zwyczaje::zwyczaje_kontrahenta(
        &room,
        &ZwyczajeKontrahentaParams {
            kontrahent: "Portal Ogłoszeń".to_string(),
        },
    )
    .expect("read the habits of the portal");
    assert!(portal_habits.is_empty());

    let habits = kierownik_dzialu_zwyczaje::zwyczaje(&room).expect("read the habits");
    assert_eq!(habits.len(), 1);
}

/// zwyczaje od źródła, które podało nieprawdę, wracają do ponownej oceny
#[test]
fn habits_from_one_source() {
    let room = new_room();
    add_source(
        &room,
        &WpiszZrodloParams {
            nazwa: Some("Doradca z banku".to_string()),
            rodzaj: RodzajZrodla::Osoba,
            ..source("doradca")
        },
    );
    add_source(
        &room,
        &WpiszZrodloParams {
            nazwa: Some("Regulamin portalu".to_string()),
            rodzaj: RodzajZrodla::Dokument,
            ..source("regulamin")
        },
    );
    add_habit(
        &room,
        &WpiszZwyczajParams {
            zrodlo_id: Some("doradca".to_string()),
            zrodlo_odnosnik: None,
            kontrahent_rodzaj: WpisyKontrahentRodzaj::Bank,
            kontrahent: "Bank".to_string(),
            ..habit()
        },
    );
    add_habit(
        &room,
        &WpiszZwyczajParams {
            zrodlo_id: Some("regulamin".to_string()),
            ..habit()
        },
    );
    add_habit(
        &room,
        &WpiszZwyczajParams {
            zrodlo_id: Some("doradca".to_string()),
            kontrahent_rodzaj: WpisyKontrahentRodzaj::Bank,
            kontrahent: "Bank".to_string(),
            temat: "termin".to_string(),
            ..habit()
        },
    );
    kierownik_dzialu_zwyczaje::wykresl_zwyczaj(
        &room,
        &WykreslZwyczajParams {
            wykreslono: "2026-11-04".to_string(),
            powod_wykreslenia: "Nieaktualny".to_string(),
            id: 3,
        },
    )
    .expect("strike the habit");

    let advisor_habits = kierownik_dzialu_zwyczaje::zwyczaje_od_zrodla(
        &room,
        &ZwyczajeOdZrodlaParams {
            zrodlo_id: "doradca".to_string(),
        },
    )
    .expect("read the habits from the source");
    let advisor_habit_ids: Vec<i64> = advisor_habits.iter().map(|row| row.id).collect();
    assert_eq!(advisor_habit_ids, [1]);

    let habit_of_unknown_source = WpiszZwyczajParams {
        kontrahent_rodzaj: WpisyKontrahentRodzaj::Bank,
        kontrahent: "Bank".to_string(),
        temat: "t".to_string(),
        opis: "o".to_string(),
        zrodlo_id: Some("nieznane-zrodlo".to_string()),
        zrodlo_odnosnik: None,
        ustalono: "2026-11-03".to_string(),
        pewnosc: Pewnosc::Niepewny,
    };
    expect::refused(
        kierownik_dzialu_zwyczaje::wpisz_zwyczaj(&room, &habit_of_unknown_source),
        "FOREIGN KEY constraint failed",
    );
}

// Reception list ----------------------------------------------------------------------------------

/// sprawa jest w spisie raz, a kancelaria na XNotariuszu tylko przy transakcji
#[test]
fn case_in_list_once() {
    let room = new_room();
    add_case_to_list(&room, &case_in_list());
    add_case_to_list(
        &room,
        &WpiszSpraweDoSpisuParams {
            teczka_odnosnik: "pokoj-transakcji-1".to_string(),
            rodzaj: SprawyRodzaj::Transakcja,
            oznaczenie: "Transakcja 2026/007".to_string(),
            kancelaria_xnotariusz: Some(true),
            ..case_in_list()
        },
    );

    expect::refused(
        kierownik_dzialu_spis::wpisz_sprawe_do_spisu(&room, &bare_case()),
        REPLACEMENT,
    );

    let offer_with_office = WpiszSpraweDoSpisuParams {
        teczka_odnosnik: "pokoj-teczki-2".to_string(),
        kancelaria_xnotariusz: Some(false),
        ..bare_case()
    };
    expect::refused(
        kierownik_dzialu_spis::wpisz_sprawe_do_spisu(&room, &offer_with_office),
        "kancelaria_tylko_przy_transakcji",
    );

    let transaction_without_office = WpiszSpraweDoSpisuParams {
        teczka_odnosnik: "pokoj-transakcji-2".to_string(),
        rodzaj: SprawyRodzaj::Transakcja,
        ..bare_case()
    };
    expect::refused(
        kierownik_dzialu_spis::wpisz_sprawe_do_spisu(&room, &transaction_without_office),
        "kancelaria_tylko_przy_transakcji",
    );
}

/// sprawa powstaje otwarta, zmiana czeka na przekazanie, a zamkniętej się nie zmienia
#[test]
fn case_changes_wait_for_reception() {
    let room = new_room();

    let case_without_date = WpiszSpraweDoSpisuParams {
        teczka_odnosnik: "bez-daty".to_string(),
        otwarto: String::new(),
        ..bare_case()
    };
    expect::refused(
        kierownik_dzialu_spis::wpisz_sprawe_do_spisu(&room, &case_without_date),
        "otwarto",
    );
    add_case_to_list(&room, &case_in_list());
    record_case_handed_over(&room, "2026-11-02T10:00:00Z");

    let cases_for_reception =
        kierownik_dzialu_spis::do_przekazania_recepcji(&room).expect("read the cases to hand");
    assert!(cases_for_reception.is_empty());

    kierownik_dzialu_spis::zmien_sprawe_w_spisie(
        &room,
        &ZmienSpraweWSpisieParams {
            oznaczenie: "Oferta 2026/031".to_string(),
            agent: "tozsamosc-nastepcy".to_string(),
            kancelaria_xnotariusz: None,
            id: 1,
        },
    )
    .expect("change the case in the list");

    let changed_cases =
        kierownik_dzialu_spis::do_przekazania_recepcji(&room).expect("read the cases to hand");
    assert_eq!(changed_cases.len(), 1);
    record_case_handed_over(&room, "2026-11-03T10:00:00Z");

    let closing_handed_before_closing = OdnotujPrzekazanieZamknieciaParams {
        zamkniecie_przekazano: "2026-11-04".to_string(),
        id: 1,
    };
    expect::refused(
        kierownik_dzialu_spis::odnotuj_przekazanie_zamkniecia(
            &room,
            &closing_handed_before_closing,
        ),
        "zamkniecie_przekazane_po_zamknieciu",
    );
    kierownik_dzialu_spis::zamknij_sprawe_w_spisie(
        &room,
        &ZamknijSpraweWSpisieParams {
            zamknieto: "2026-11-20".to_string(),
            id: 1,
        },
    )
    .expect("close the case in the list");

    let closed_cases =
        kierownik_dzialu_spis::do_przekazania_recepcji(&room).expect("read the cases to hand");
    assert_eq!(closed_cases.len(), 1);
    kierownik_dzialu_spis::odnotuj_przekazanie_zamkniecia(
        &room,
        &OdnotujPrzekazanieZamknieciaParams {
            zamkniecie_przekazano: "2026-11-20".to_string(),
            id: 1,
        },
    )
    .expect("record the closing handed to Reception");

    let rule = "teczki ani rodzaju sprawy w spisie się nie zmienia";
    let change_of_closed = ZmienSpraweWSpisieParams {
        oznaczenie: "Inna".to_string(),
        agent: "inny".to_string(),
        kancelaria_xnotariusz: None,
        id: 1,
    };
    expect::refused(
        kierownik_dzialu_spis::zmien_sprawe_w_spisie(&room, &change_of_closed),
        rule,
    );

    let second_closing_handover = OdnotujPrzekazanieZamknieciaParams {
        zamkniecie_przekazano: "2026-11-21".to_string(),
        id: 1,
    };
    expect::refused(
        kierownik_dzialu_spis::odnotuj_przekazanie_zamkniecia(&room, &second_closing_handover),
        rule,
    );
    expect::refused(
        room.execute(
            "UPDATE kierownik_dzialu_spis_sprawy SET rodzaj = 'najem'",
            &[],
        ),
        rule,
    );
    expect::refused(
        room.execute(
            "INSERT INTO kierownik_dzialu_spis_sprawy (teczka_odnosnik, rodzaj, oznaczenie, agent,
              zamknieto)
            VALUES ('pokoj-teczki-9', 'oferta', 'Oferta', 'agent', '2026-11-02')",
            &[],
        ),
        "sprawa powstaje w spisie otwarta",
    );

    let listed_cases = kierownik_dzialu_spis::sprawy_w_spisie(&room).expect("read the list");
    assert_eq!(listed_cases[0].zamknieto.as_deref(), Some("2026-11-20"));
}

fn record_case_handed_over(room: &MemoryRoom, przekazano: &str) {
    let params = OdnotujPrzekazanieSprawyParams {
        przekazano: przekazano.to_string(),
        id: 1,
    };

    kierownik_dzialu_spis::odnotuj_przekazanie_sprawy(room, &params)
        .expect("record the case handed to Reception");
}

/// znaku się nie usuwa ani nie zmienia, tylko wycofuje, a wycofanie przekazuje się raz
#[test]
fn marks_are_withdrawn() {
    let room = new_room();
    add_case_to_list(&room, &case_in_list());
    add_mark(&room, &mark());
    add_mark(
        &room,
        &DodajZnakParams {
            rodzaj: ZnakiRodzaj::AdresPocztyKlienta,
            wartosc: "nowak@example.com".to_string(),
            ..mark()
        },
    );

    let withdrawal_handed_before_withdrawal = OdnotujPrzekazanieWycofaniaParams {
        wycofanie_przekazano: "2026-11-03".to_string(),
        id: 1,
    };
    expect::refused(
        kierownik_dzialu_spis::odnotuj_przekazanie_wycofania(
            &room,
            &withdrawal_handed_before_withdrawal,
        ),
        "wycofanie_przekazane_po_wycofaniu_przekazanego_znaku",
    );
    kierownik_dzialu_spis::odnotuj_przekazanie_znaku(
        &room,
        &OdnotujPrzekazanieZnakuParams {
            przekazano: "2026-11-02T10:00:00Z".to_string(),
            id: 1,
        },
    )
    .expect("record the mark handed to Reception");
    for mark_id in [1, 2] {
        let withdrawal = WycofajZnakParams {
            wycofano: "2026-11-05T10:00:00Z".to_string(),
            id: mark_id,
        };
        kierownik_dzialu_spis::wycofaj_znak(&room, &withdrawal).expect("withdraw the mark");
    }

    let marks_for_reception = kierownik_dzialu_spis::znaki_do_przekazania_recepcji(&room)
        .expect("read the marks to hand");
    let mark_ids: Vec<i64> = marks_for_reception.iter().map(|row| row.id).collect();
    assert_eq!(mark_ids, [1]);
    kierownik_dzialu_spis::odnotuj_przekazanie_wycofania(
        &room,
        &OdnotujPrzekazanieWycofaniaParams {
            wycofanie_przekazano: "2026-11-05T11:00:00Z".to_string(),
            id: 1,
        },
    )
    .expect("record the withdrawal handed to Reception");

    let marks_after_handover = kierownik_dzialu_spis::znaki_do_przekazania_recepcji(&room)
        .expect("read the marks to hand");
    assert!(marks_after_handover.is_empty());

    let rule = "znaku się nie zmienia";
    let second_withdrawal = WycofajZnakParams {
        wycofano: "2026-11-06".to_string(),
        id: 1,
    };
    expect::refused(
        kierownik_dzialu_spis::wycofaj_znak(&room, &second_withdrawal),
        rule,
    );

    let second_handover = OdnotujPrzekazanieZnakuParams {
        przekazano: "2026-11-06".to_string(),
        id: 1,
    };
    expect::refused(
        kierownik_dzialu_spis::odnotuj_przekazanie_znaku(&room, &second_handover),
        rule,
    );
    expect::refused(
        room.execute(
            "UPDATE kierownik_dzialu_spis_znaki SET wartosc = 'ul. Leśna 1' WHERE id = 2",
            &[],
        ),
        rule,
    );
    expect::refused(
        room.execute("DELETE FROM kierownik_dzialu_spis_znaki WHERE id = 2", &[]),
        "znaku się nie usuwa, tylko wycofuje",
    );

    add_mark(
        &room,
        &DodajZnakParams {
            rodzaj: ZnakiRodzaj::OznaczenieTransakcji,
            wartosc: "Transakcja 2026/007".to_string(),
            ..mark()
        },
    );

    let withdrawal_before_adding = WycofajZnakParams {
        wycofano: "2026-11-01".to_string(),
        id: 3,
    };
    expect::refused(
        kierownik_dzialu_spis::wycofaj_znak(&room, &withdrawal_before_adding),
        "wycofanie_nie_przed_dodaniem",
    );

    let handover_before_adding = OdnotujPrzekazanieZnakuParams {
        przekazano: "2026-11-01".to_string(),
        id: 3,
    };
    expect::refused(
        kierownik_dzialu_spis::odnotuj_przekazanie_znaku(&room, &handover_before_adding),
        "przekazanie_nie_przed_dodaniem",
    );
    expect::refused(
        room.execute(
            "INSERT INTO kierownik_dzialu_spis_znaki (sprawa_id, rodzaj, wartosc, dodano,
              przekazano)
            VALUES (1, 'oznaczenie_transakcji', 'T/1', '2026-11-02', '2026-11-02')",
            &[],
        ),
        "znak powstaje niewycofany i jeszcze nieprzekazany",
    );

    let case_marks =
        kierownik_dzialu_spis::znaki_sprawy(&room, &ZnakiSprawyParams { sprawa_id: 1 })
            .expect("read the marks of the case");
    assert_eq!(case_marks.len(), 3);
}

// Policies ----------------------------------------------------------------------------------------

/// umowa dostaje kopię jednej polisy raz, a rozesłania się nie zmienia
#[test]
fn policy_copy_once_per_contract() {
    let room = new_room();
    appoint_agent(&room, &agent("opiekun-1"));
    receive_report(&room, &report("raport-1", "opiekun-1"));
    record_policy_copy(&room, &policy_copy());
    record_policy_copy(
        &room,
        &OdnotujRozeslaniePolisyParams {
            umowa_odnosnik: "pokoj-teczki-2/umowa-posrednictwa".to_string(),
            ..policy_copy()
        },
    );
    record_policy_copy(
        &room,
        &OdnotujRozeslaniePolisyParams {
            polisa_numer: "OC/2028/001".to_string(),
            ..policy_copy()
        },
    );

    let second_copy_for_contract = OdnotujRozeslaniePolisyParams {
        polisa_numer: "OC/2027/001".to_string(),
        polisa_odnosnik: None,
        umowa_odnosnik: "pokoj-teczki-1/umowa-posrednictwa".to_string(),
        raport_id: "raport-1".to_string(),
        rozeslano: "2026-11-06".to_string(),
    };
    expect::refused(
        kierownik_dzialu_polisy::odnotuj_rozeslanie_polisy(&room, &second_copy_for_contract),
        REPLACEMENT,
    );
    expect::refused(
        room.execute(
            "UPDATE kierownik_dzialu_polisy_rozeslania SET rozeslano = '2026-11-07'",
            &[],
        ),
        "rozesłania polisy się nie zmienia",
    );

    // A writer outside the client can leave out the report that `odnotuj_rozeslanie_polisy`
    // requires. The table refuses the copy without one.
    let copy_without_report = room.execute(
        "INSERT INTO kierownik_dzialu_polisy_rozeslania (polisa_numer, polisa_odnosnik,
          umowa_odnosnik, raport_id, rozeslano)
        VALUES (:polisa_numer, :polisa_odnosnik, :umowa_odnosnik, :raport_id, :rozeslano)",
        &[
            ("polisa_numer", text("OC/2027/001")),
            ("polisa_odnosnik", SqlValue::Null),
            ("umowa_odnosnik", text("pokoj-teczki-3/umowa")),
            ("raport_id", SqlValue::Null),
            ("rozeslano", text("2026-11-06")),
        ],
    );
    expect::refused(
        copy_without_report,
        "NOT NULL constraint failed: kierownik_dzialu_polisy_rozeslania.raport_id",
    );

    let policy_copies = kierownik_dzialu_polisy::rozeslania_polisy(
        &room,
        &RozeslaniaPolisyParams {
            polisa_numer: "OC/2027/001".to_string(),
        },
    )
    .expect("read the copies of the policy");
    assert_eq!(policy_copies.len(), 2);
}

// R9 and replacement ------------------------------------------------------------------------------

/// The tables of the catalog in the room.
fn tables_of(room: &MemoryRoom) -> Vec<String> {
    let rows = room
        .execute(
            r"SELECT name FROM sqlite_schema WHERE type = 'table' AND name LIKE 'kierownik\_dzialu\_%' ESCAPE '\'",
            &[],
        )
        .expect("read the tables");

    rows.iter()
        .map(|row| row.get::<String>("name").expect("a table has a name"))
        .collect()
}

/// The names of the table's columns, in the order of the schema.
fn columns_of(room: &MemoryRoom, table: &str) -> Vec<String> {
    let rows = room
        .execute(&format!("PRAGMA table_info({table})"), &[])
        .expect("read the columns of the table");

    rows.iter()
        .map(|row| row.get::<String>("name").expect("a column has a name"))
        .collect()
}

/// R9: wierszy pamięci się nie usuwa ani nie zastępuje przez REPLACE albo upsert
#[test]
fn no_deletion_or_replacement() {
    let room = new_room();
    full_memory(&room);

    let tables = tables_of(&room);
    assert_eq!(tables.len(), 10, "{tables:?}");

    for table in &tables {
        let row_count = room
            .value(&format!("SELECT count(*) FROM {table}"), &[])
            .expect("count the rows");
        let row_count = i64::from_sql_value(&row_count).expect("the count is a number");
        assert!(row_count > 0, "{table} has no row");

        let replace = format!("INSERT OR REPLACE {}", first_row_copy(&room, table));
        expect::refused(room.execute(&replace, &[]), REPLACEMENT);

        let upsert = format!(
            "INSERT {} ON CONFLICT (id) DO UPDATE SET id = excluded.id",
            first_row_copy(&room, table)
        );
        expect::refused(room.execute(&upsert, &[]), REPLACEMENT);

        let delete = format!("DELETE FROM {table} WHERE rowid = (SELECT min(rowid) FROM {table})");
        expect::refused(room.execute(&delete, &[]), DELETION);
    }
}

/// REPLACE nie usuwa otwartego wpuszczenia, prośby, sprawy ani rozesłania przez klucz UNIQUE
#[test]
fn no_replacement_through_unique_keys() {
    let room = new_room();
    full_memory(&room);

    expect::refused(
        room.execute(
            "INSERT OR REPLACE INTO kierownik_dzialu_agenci_wpuszczenia (id, tozsamosc, stanowisko,
              pokoj_odnosnik, klient_odnosnik, wpuscil, wpuszczono)
            SELECT 99, tozsamosc, stanowisko, pokoj_odnosnik, klient_odnosnik, wpuscil, wpuszczono
            FROM kierownik_dzialu_agenci_wpuszczenia",
            &[],
        ),
        "ta tożsamość ma już otwarte wpuszczenie do tego pokoju",
    );
    expect::refused(
        room.execute(
            "INSERT OR REPLACE INTO kierownik_dzialu_raporty_prosby (id, decyzja_odnosnik,
              raport_id, przekazano_do, przekazano)
            SELECT 99, decyzja_odnosnik, raport_id, przekazano_do, przekazano
            FROM kierownik_dzialu_raporty_prosby",
            &[],
        ),
        "tej prośby nie przekazuje się drugi raz",
    );

    let unique_keys = [
        ("kierownik_dzialu_standardy_wersje", "standard, numer"),
        ("kierownik_dzialu_spis_sprawy", "teczka_odnosnik"),
        (
            "kierownik_dzialu_polisy_rozeslania",
            "polisa_numer, umowa_odnosnik",
        ),
    ];
    for (table, key) in unique_keys {
        let columns = columns_of(&room, table);
        let selected: Vec<&str> = columns
            .iter()
            .map(|name| if name == "id" { "99" } else { name.as_str() })
            .collect();
        let columns = columns.join(", ");
        let selected = selected.join(", ");

        let replace =
            format!("INSERT OR REPLACE INTO {table} ({columns}) SELECT {selected} FROM {table}");
        expect::refused(room.execute(&replace, &[]), REPLACEMENT);

        // `WHERE true` keeps SQLite from reading ON CONFLICT as a join constraint.
        let upsert = format!(
            "INSERT INTO {table} ({columns}) SELECT {selected} FROM {table} \
            WHERE true ON CONFLICT ({key}) DO UPDATE SET id = excluded.id"
        );
        expect::refused(room.execute(&upsert, &[]), REPLACEMENT);
    }
}
