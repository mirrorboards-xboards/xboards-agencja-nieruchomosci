//! Scenarios of the control head's memory room (spec/positions/kontrola/kierownik_kontroli).
//!
//! The room has the head's own kinds, three kinds of the agent memory and five kinds of the
//! department memory (no case list for Reception and no policy copies, D11); the rules of the
//! shared kinds are checked in stanowisko.rs and kierownik_dzialu.rs. Each scenario starts from an
//! empty room and checks one rule the database keeps: a write the rule allows and a write it
//! refuses. The first scenario follows the office's risk assessment to the control register, a
//! notification draft that the owner hands over and a person's request about their data.

// The building blocks of kierownik_dzialu.rs take the types of the shared catalog's room.
use xboards_agencja_nieruchomosci::positions::kierownik_dzialu::kierownik_dzialu_raporty as shared_raporty;
use xboards_agencja_nieruchomosci::positions::kontrola::kierownik_kontroli::{
    self, kierownik_dzialu_raporty, kierownik_kontroli_dane_osobowe, kierownik_kontroli_oceny,
    kierownik_kontroli_projekty,
};
use xboards_agencja_nieruchomosci::xmeta_room::{FromSqlValue, MemoryRoom, SqlValue};

use kierownik_dzialu_raporty::ProsbyPrzekazanoDo;
use kierownik_kontroli_dane_osobowe::{
    DajSpraweWlascicielowiParams, OdnotujZalatwienieParams, WpisyRodzaj as RodzajSprawyDanych,
    WpisyStan as StanSprawyDanych, ZapiszSpraweDanychOsobowychParams,
};
use kierownik_kontroli_oceny::{
    CzynnikiObszar, CzynnikiOcenyParams, CzynnikiPoziom, DopiszCzynnikParams,
    OdnotujPrzegladOcenyParams, OdnotujZatwierdzenieOcenyParams, PrzekazOceneDoZatwierdzeniaParams,
    RozpocznijProjektOcenyParams, ZmienCzynnikParams,
};
use kierownik_kontroli_projekty::{
    CofnijProjektDoPoprawyParams, DajProjektWlascicielowiParams, DopiszPodstaweParams,
    OdnotujOdstapienieParams, OdnotujPrzekazanieZawiadomieniaParams, PodstawyProjektuParams,
    UtworzProjektParams, WpisyRodzaj as RodzajProjektu, WpisyStan as StanProjektu,
    ZmienProjektParams,
};

use crate::expect;
use crate::kierownik_dzialu::{
    appoint_agent, controller, forward_request, receive_report, report, request,
};
use crate::stanowisko::{DELETION, REPLACEMENT, first_row_copy};

fn new_room() -> MemoryRoom {
    kierownik_kontroli::open_in_memory().expect("open the room of the Head of control")
}

/// A text value to bind in raw SQL.
fn text(value: &str) -> SqlValue {
    SqlValue::Text(value.to_string())
}

// Building blocks ---------------------------------------------------------------------------------

/// The default parameters of the start of a draft of the office's risk assessment.
fn assessment_start() -> RozpocznijProjektOcenyParams {
    RozpocznijProjektOcenyParams {
        rozpoczeto: "2026-11-02T09:00:00Z".to_string(),
    }
}

fn start_assessment(room: &MemoryRoom, params: &RozpocznijProjektOcenyParams) {
    kierownik_kontroli_oceny::rozpocznij_projekt_oceny(room, params)
        .expect("start the draft of the risk assessment");
}

/// The default parameters of a risk factor of the first assessment: clients from outside the EEA.
fn factor() -> DopiszCzynnikParams {
    DopiszCzynnikParams {
        ocena_id: 1,
        obszar: CzynnikiObszar::Klienci,
        czynnik: "Klienci spoza Europejskiego Obszaru Gospodarczego".to_string(),
        poziom: CzynnikiPoziom::Srednie,
        srodki: Some("Wzmożona weryfikacja źródła środków".to_string()),
        zrodlo: "Krajowa ocena ryzyka".to_string(),
    }
}

/// The parameters of a risk factor of services without measures, for the writes a rule refuses.
fn services_factor() -> DopiszCzynnikParams {
    DopiszCzynnikParams {
        ocena_id: 1,
        obszar: CzynnikiObszar::Uslugi,
        czynnik: "Zarządzanie najmem krótkoterminowym".to_string(),
        poziom: CzynnikiPoziom::Niskie,
        srodki: None,
        zrodlo: "Raport Kontrolera".to_string(),
    }
}

fn add_factor(room: &MemoryRoom, params: &DopiszCzynnikParams) {
    kierownik_kontroli_oceny::dopisz_czynnik(room, params).expect("add the risk factor");
}

fn hand_assessment_for_approval(room: &MemoryRoom, przekazano_do_zatwierdzenia: &str) {
    let params = PrzekazOceneDoZatwierdzeniaParams {
        przekazano_do_zatwierdzenia: przekazano_do_zatwierdzenia.to_string(),
        id: 1,
    };

    kierownik_kontroli_oceny::przekaz_ocene_do_zatwierdzenia(room, &params)
        .expect("hand the assessment over for approval");
}

fn record_assessment_approval(room: &MemoryRoom) {
    let params = OdnotujZatwierdzenieOcenyParams {
        rejestr_odnosnik: "pokoj-rejestru/ocena-biura-1".to_string(),
        id: 1,
    };

    kierownik_kontroli_oceny::odnotuj_zatwierdzenie_oceny(room, &params)
        .expect("record the approval of the assessment");
}

/// The default parameters of a draft of a notification under art. 74 for a transaction.
fn draft() -> UtworzProjektParams {
    UtworzProjektParams {
        rodzaj: RodzajProjektu::Art74,
        teczka_odnosnik: Some("pokoj-transakcji-1".to_string()),
        termin: "2026-11-06".to_string(),
        utworzono: "2026-11-04T10:00:00Z".to_string(),
        sciezka: Some("projekty/art74-1.typ".to_string()),
        skrot: Some("sha256-projekt-1".to_string()),
    }
}

/// The parameters of a draft under art. 86 without a case file, for the writes a rule refuses.
fn bare_draft() -> UtworzProjektParams {
    UtworzProjektParams {
        rodzaj: RodzajProjektu::Art86,
        teczka_odnosnik: None,
        termin: "2026-11-06".to_string(),
        utworzono: "2026-11-04T10:00:00Z".to_string(),
        sciezka: Some("projekty/art86-1.typ".to_string()),
        skrot: Some("sha256".to_string()),
    }
}

fn create_draft(room: &MemoryRoom, params: &UtworzProjektParams) {
    kierownik_kontroli_projekty::utworz_projekt(room, params).expect("create the draft");
}

fn add_basis(room: &MemoryRoom, projekt_id: i64, rejestr_odnosnik: &str) {
    let params = DopiszPodstaweParams {
        projekt_id,
        rejestr_odnosnik: rejestr_odnosnik.to_string(),
    };

    kierownik_kontroli_projekty::dopisz_podstawe(room, &params).expect("add the basis");
}

fn give_draft_to_owner(room: &MemoryRoom, draft_id: i64) {
    let params = DajProjektWlascicielowiParams {
        u_wlasciciela_od: "2026-11-04T11:00:00Z".to_string(),
        id: draft_id,
    };

    kierownik_kontroli_projekty::daj_projekt_wlascicielowi(room, &params)
        .expect("give the draft to the owner");
}

fn record_notification_handed_over(room: &MemoryRoom, draft_id: i64) {
    let params = OdnotujPrzekazanieZawiadomieniaParams {
        przekazano: "2026-11-05T09:00:00Z".to_string(),
        rejestr_odnosnik: "pokoj-rejestru/zawiadomienie-1".to_string(),
        id: draft_id,
    };

    kierownik_kontroli_projekty::odnotuj_przekazanie_zawiadomienia(room, &params)
        .expect("record the notification handed over");
}

/// The owner's signed withdrawal from the draft.
fn owner_withdrawal(draft_id: i64) -> OdnotujOdstapienieParams {
    OdnotujOdstapienieParams {
        odstapil: "tozsamosc-wlasciciela".to_string(),
        odstapienie_podpis: "podpis-kluczem-wlasciciela".to_string(),
        odstapiono: "2026-11-05T10:00:00Z".to_string(),
        id: draft_id,
    }
}

fn withdraw_draft(room: &MemoryRoom, draft_id: i64) {
    kierownik_kontroli_projekty::odnotuj_odstapienie(room, &owner_withdrawal(draft_id))
        .expect("record the owner's withdrawal");
}

/// The default parameters of a person's request for access to their data (art. 15).
fn data_request() -> ZapiszSpraweDanychOsobowychParams {
    ZapiszSpraweDanychOsobowychParams {
        rodzaj: RodzajSprawyDanych::DostepArt15,
        wplynelo: "2026-11-04T09:00:00Z".to_string(),
        wplyw_odnosnik: "wiadomosc-prosba-o-dane".to_string(),
        termin: "2026-12-04".to_string(),
    }
}

fn add_data_request(room: &MemoryRoom, params: &ZapiszSpraweDanychOsobowychParams) {
    kierownik_kontroli_dane_osobowe::zapisz_sprawe_danych_osobowych(room, params)
        .expect("add the request about personal data");
}

/// The parameters of settling the first request about personal data with the owner's answer.
fn settlement(odpowiedz_odnosnik: &str, zalatwiono: &str) -> OdnotujZalatwienieParams {
    OdnotujZalatwienieParams {
        odpowiedz_odnosnik: odpowiedz_odnosnik.to_string(),
        zalatwiono: zalatwiono.to_string(),
        id: 1,
    }
}

/// One row in every table of the head's own kinds, for the checks of R9 and replacement.
fn full_memory(room: &MemoryRoom) {
    start_assessment(room, &assessment_start());
    add_factor(room, &factor());
    create_draft(room, &draft());
    add_basis(room, 1, "pokoj-rejestru/sprawdzenie-1");
    add_data_request(room, &data_request());
}

// A whole day -------------------------------------------------------------------------------------

/// ocena ryzyka trafia do rejestru, właściciel przekazuje zawiadomienie i załatwia prośbę
#[test]
fn whole_day() {
    let room = new_room();

    appoint_agent(&room, &controller("kontroler-1"));
    receive_report(
        &room,
        &shared_raporty::PrzyjmijRaportParams {
            etap_albo_okres: "weryfikacja".to_string(),
            decyzja_odnosnik: Some("pokoj-transakcji-1/decyzja-1".to_string()),
            ..report("raport-1", "kontroler-1")
        },
    );
    forward_request(
        &room,
        &shared_raporty::PrzekazProsbeParams {
            decyzja_odnosnik: "pokoj-transakcji-1/decyzja-1".to_string(),
            przekazano_do: shared_raporty::ProsbyPrzekazanoDo::Wlasciciel,
            ..request()
        },
    );

    start_assessment(&room, &assessment_start());
    add_factor(&room, &factor());
    add_factor(
        &room,
        &DopiszCzynnikParams {
            obszar: CzynnikiObszar::KanalyDostaw,
            czynnik: "Umowy zawierane na odległość".to_string(),
            poziom: CzynnikiPoziom::Wysokie,
            ..factor()
        },
    );
    kierownik_kontroli_oceny::odnotuj_przeglad_oceny(
        &room,
        &OdnotujPrzegladOcenyParams {
            przejrzano: "2026-11-03T09:00:00Z".to_string(),
            id: 1,
        },
    )
    .expect("record the review of the assessment");
    hand_assessment_for_approval(&room, "2026-11-03T10:00:00Z");
    record_assessment_approval(&room);

    create_draft(&room, &draft());
    add_basis(&room, 1, "pokoj-rejestru/sprawdzenie-1");
    add_basis(&room, 1, "pokoj-rejestru/decyzja-potwierdzenia");
    give_draft_to_owner(&room, 1);
    record_notification_handed_over(&room, 1);

    add_data_request(&room, &data_request());
    kierownik_kontroli_dane_osobowe::daj_sprawe_wlascicielowi(
        &room,
        &DajSpraweWlascicielowiParams { id: 1 },
    )
    .expect("give the request to the owner");
    kierownik_kontroli_dane_osobowe::odnotuj_zalatwienie(
        &room,
        &settlement("wiadomosc-odpowiedz-wlasciciela", "2026-11-20T09:00:00Z"),
    )
    .expect("record the request settled");

    let requests = kierownik_dzialu_raporty::prosby_na_liscie(&room).expect("read the requests");
    assert_eq!(requests[0].przekazano_do, ProsbyPrzekazanoDo::Wlasciciel);

    let factors =
        kierownik_kontroli_oceny::czynniki_oceny(&room, &CzynnikiOcenyParams { ocena_id: 1 })
            .expect("read the factors of the assessment");
    assert_eq!(factors.len(), 2);

    let drafts_in_progress =
        kierownik_kontroli_projekty::projekty_w_toku(&room).expect("read the drafts in progress");
    assert!(drafts_in_progress.is_empty());

    let drafts = kierownik_kontroli_projekty::projekty(&room).expect("read the drafts");
    let handed_over = &drafts[0];
    assert_eq!(
        (
            handed_over.stan,
            handed_over.sciezka.as_deref(),
            handed_over.skrot.as_deref(),
        ),
        (StanProjektu::Przekazany, None, None)
    );

    let bases = kierownik_kontroli_projekty::podstawy_projektu(
        &room,
        &PodstawyProjektuParams { projekt_id: 1 },
    )
    .expect("read the bases of the draft");
    assert_eq!(bases.len(), 2);

    let data_requests = kierownik_kontroli_dane_osobowe::sprawy_danych_osobowych(&room)
        .expect("read the requests about personal data");
    assert_eq!(data_requests[0].stan, StanSprawyDanych::Zalatwiona);
}

// Risk assessment of the office -------------------------------------------------------------------

/// projekt oceny powstaje roboczy, a zatwierdzenie przychodzi po przekazaniu
#[test]
fn assessment_is_approved_after_handover() {
    let room = new_room();
    start_assessment(&room, &assessment_start());

    expect::refused(
        room.execute(
            "INSERT INTO kierownik_kontroli_oceny_wpisy (rozpoczeto, przekazano_do_zatwierdzenia)
            VALUES ('2026-11-02', '2026-11-03')",
            &[],
        ),
        "projekt oceny powstaje roboczy",
    );

    let review_before_start = OdnotujPrzegladOcenyParams {
        przejrzano: "2026-11-01T09:00:00Z".to_string(),
        id: 1,
    };
    expect::refused(
        kierownik_kontroli_oceny::odnotuj_przeglad_oceny(&room, &review_before_start),
        "przeglad_nie_przed_rozpoczeciem",
    );

    let handover_before_start = PrzekazOceneDoZatwierdzeniaParams {
        przekazano_do_zatwierdzenia: "2026-11-01T09:00:00Z".to_string(),
        id: 1,
    };
    expect::refused(
        kierownik_kontroli_oceny::przekaz_ocene_do_zatwierdzenia(&room, &handover_before_start),
        "przekazanie_nie_przed_rozpoczeciem",
    );

    let approval_before_handover = OdnotujZatwierdzenieOcenyParams {
        rejestr_odnosnik: "pokoj-rejestru/ocena-biura-1".to_string(),
        id: 1,
    };
    expect::refused(
        kierownik_kontroli_oceny::odnotuj_zatwierdzenie_oceny(&room, &approval_before_handover),
        "zatwierdzona_po_przekazaniu",
    );
    hand_assessment_for_approval(&room, "2026-11-03T10:00:00Z");
    record_assessment_approval(&room);

    let assessments =
        kierownik_kontroli_oceny::projekty_ocen(&room).expect("read the drafts of assessments");
    assert_eq!(
        assessments[0].rejestr_odnosnik.as_deref(),
        Some("pokoj-rejestru/ocena-biura-1")
    );
}

/// zatwierdzonego projektu oceny ani jego czynników się nie zmienia
#[test]
fn approved_assessment_does_not_change() {
    let room = new_room();
    start_assessment(&room, &assessment_start());
    start_assessment(
        &room,
        &RozpocznijProjektOcenyParams {
            rozpoczeto: "2026-11-03T09:00:00Z".to_string(),
        },
    );
    add_factor(&room, &factor());
    kierownik_kontroli_oceny::zmien_czynnik(
        &room,
        &ZmienCzynnikParams {
            obszar: CzynnikiObszar::Klienci,
            czynnik: "Klienci spoza EOG".to_string(),
            poziom: CzynnikiPoziom::Wysokie,
            srodki: None,
            zrodlo: "Krajowa ocena ryzyka".to_string(),
            id: 1,
        },
    )
    .expect("change the factor");
    expect::refused(
        room.execute(
            "UPDATE kierownik_kontroli_oceny_czynniki SET ocena_id = 2 WHERE id = 1",
            &[],
        ),
        "czynnik nie przechodzi do innego projektu",
    );

    hand_assessment_for_approval(&room, "2026-11-03T10:00:00Z");
    record_assessment_approval(&room);

    let rule = "projektu oceny zatwierdzonej w rejestrze kontroli się nie zmienia";
    let review_after_approval = OdnotujPrzegladOcenyParams {
        przejrzano: "2026-11-04T09:00:00Z".to_string(),
        id: 1,
    };
    expect::refused(
        kierownik_kontroli_oceny::odnotuj_przeglad_oceny(&room, &review_after_approval),
        rule,
    );
    expect::refused(
        kierownik_kontroli_oceny::dopisz_czynnik(&room, &services_factor()),
        rule,
    );

    let services = services_factor();
    let change_after_approval = ZmienCzynnikParams {
        obszar: services.obszar,
        czynnik: services.czynnik,
        poziom: services.poziom,
        srodki: services.srodki,
        zrodlo: services.zrodlo,
        id: 1,
    };
    expect::refused(
        kierownik_kontroli_oceny::zmien_czynnik(&room, &change_after_approval),
        rule,
    );
    add_factor(
        &room,
        &DopiszCzynnikParams {
            ocena_id: 2,
            ..services_factor()
        },
    );
}

// Notification drafts -----------------------------------------------------------------------------

/// projekt zawiadomienia powstaje jako projekt z plikiem, a u właściciela ma datę
#[test]
fn draft_starts_with_file() {
    let room = new_room();
    create_draft(&room, &draft());

    expect::refused(
        room.execute(
            "INSERT INTO kierownik_kontroli_projekty_wpisy (rodzaj, termin, utworzono, stan,
              przekazano, rejestr_odnosnik)
            VALUES ('art86', '2026-11-06', '2026-11-04', 'przekazany', '2026-11-05', 'r')",
            &[],
        ),
        "projekt zawiadomienia powstaje w stanie projekt",
    );

    let path_without_hash = UtworzProjektParams {
        skrot: None,
        ..bare_draft()
    };
    expect::refused(
        kierownik_kontroli_projekty::utworz_projekt(&room, &path_without_hash),
        "sciezka_razem_ze_skrotem",
    );

    let draft_without_file = UtworzProjektParams {
        sciezka: None,
        skrot: None,
        ..bare_draft()
    };
    expect::refused(
        kierownik_kontroli_projekty::utworz_projekt(&room, &draft_without_file),
        "projekt_z_plikiem",
    );

    // A writer outside the client can leave out the date that `daj_projekt_wlascicielowi`
    // requires. The table refuses the draft at the owner's without one.
    let at_owner_without_date = room.execute(
        "UPDATE kierownik_kontroli_projekty_wpisy
        SET stan = 'u_wlasciciela',
          u_wlasciciela_od = :u_wlasciciela_od
        WHERE id = :id",
        &[
            ("u_wlasciciela_od", SqlValue::Null),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(at_owner_without_date, "u_wlasciciela_z_data");
    give_draft_to_owner(&room, 1);
    kierownik_kontroli_projekty::cofnij_projekt_do_poprawy(
        &room,
        &CofnijProjektDoPoprawyParams { id: 1 },
    )
    .expect("send the draft back for correction");
    kierownik_kontroli_projekty::zmien_projekt(
        &room,
        &ZmienProjektParams {
            termin: "2026-11-06".to_string(),
            sciezka: "projekty/art74-1-v2.typ".to_string(),
            skrot: "sha256-v2".to_string(),
            id: 1,
        },
    )
    .expect("change the draft");

    let drafts_in_progress =
        kierownik_kontroli_projekty::projekty_w_toku(&room).expect("read the drafts in progress");
    assert_eq!(drafts_in_progress[0].skrot.as_deref(), Some("sha256-v2"));
}

/// przekazany projekt wskazuje rejestr i nie ma pliku; przekazanego się nie zmienia
#[test]
fn handed_over_draft_points_to_register() {
    let room = new_room();
    create_draft(&room, &draft());
    give_draft_to_owner(&room, 1);

    // A writer outside the client can leave out the register entry that
    // `odnotuj_przekazanie_zawiadomienia` requires. The table refuses the handover without one.
    let handover_without_entry = room.execute(
        "UPDATE kierownik_kontroli_projekty_wpisy
        SET stan = 'przekazany',
          przekazano = :przekazano,
          rejestr_odnosnik = :rejestr_odnosnik,
          sciezka = NULL,
          skrot = NULL
        WHERE id = :id",
        &[
            ("przekazano", text("2026-11-05T09:00:00Z")),
            ("rejestr_odnosnik", SqlValue::Null),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(handover_without_entry, "przekazany_z_wpisem_w_rejestrze");
    expect::refused(
        room.execute(
            "UPDATE kierownik_kontroli_projekty_wpisy
            SET stan = 'przekazany', przekazano = '2026-11-05T09:00:00Z', rejestr_odnosnik = 'r'
            WHERE id = 1",
            &[],
        ),
        "przekazany_bez_tresci",
    );

    let handover_before_creation = OdnotujPrzekazanieZawiadomieniaParams {
        przekazano: "2026-11-03T09:00:00Z".to_string(),
        rejestr_odnosnik: "pokoj-rejestru/zawiadomienie-1".to_string(),
        id: 1,
    };
    expect::refused(
        kierownik_kontroli_projekty::odnotuj_przekazanie_zawiadomienia(
            &room,
            &handover_before_creation,
        ),
        "przekazanie_nie_przed_utworzeniem",
    );
    record_notification_handed_over(&room, 1);

    let rule = "a przekazanego i odstąpionego wcale";
    expect::refused(
        kierownik_kontroli_projekty::cofnij_projekt_do_poprawy(
            &room,
            &CofnijProjektDoPoprawyParams { id: 1 },
        ),
        rule,
    );
    expect::refused(
        kierownik_kontroli_projekty::odnotuj_odstapienie(&room, &owner_withdrawal(1)),
        rule,
    );
}

/// od projektu odstępuje właściciel z podpisem, a odstąpionego się nie zmienia
#[test]
fn withdrawal_is_signed_by_owner() {
    let room = new_room();
    create_draft(&room, &draft());

    let rule = "odstapienie_z_podpisem_wlasciciela";

    // A writer outside the client can leave out the signature that `odnotuj_odstapienie`
    // requires. The table refuses the withdrawal without one.
    let withdrawal_without_signature = room.execute(
        "UPDATE kierownik_kontroli_projekty_wpisy
        SET stan = 'odstapiono',
          odstapil = :odstapil,
          odstapienie_podpis = :odstapienie_podpis,
          odstapiono = :odstapiono
        WHERE id = :id",
        &[
            ("odstapil", text("tozsamosc-wlasciciela")),
            ("odstapienie_podpis", SqlValue::Null),
            ("odstapiono", text("2026-11-05T10:00:00Z")),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(withdrawal_without_signature, rule);
    expect::refused(
        room.execute(
            "UPDATE kierownik_kontroli_projekty_wpisy SET stan = 'odstapiono' WHERE id = 1",
            &[],
        ),
        rule,
    );
    withdraw_draft(&room, 1);

    let change_after_withdrawal = ZmienProjektParams {
        termin: "2026-11-07".to_string(),
        sciezka: "s".to_string(),
        skrot: "k".to_string(),
        id: 1,
    };
    expect::refused(
        kierownik_kontroli_projekty::zmien_projekt(&room, &change_after_withdrawal),
        "a przekazanego i odstąpionego wcale",
    );
    expect::refused(
        room.execute(
            "UPDATE kierownik_kontroli_projekty_wpisy SET rodzaj = 'art90' WHERE id = 1",
            &[],
        ),
        "rodzaju, teczki ani daty projektu się nie zmienia",
    );
}

/// podstawę dopisuje się raz i tylko do projektu w toku
#[test]
fn basis_only_for_draft_in_progress() {
    let room = new_room();
    create_draft(&room, &draft());
    create_draft(
        &room,
        &UtworzProjektParams {
            utworzono: "2026-11-04T12:00:00Z".to_string(),
            ..draft()
        },
    );
    add_basis(&room, 1, "pokoj-rejestru/sprawdzenie-1");
    withdraw_draft(&room, 2);

    let basis_of_withdrawn = DopiszPodstaweParams {
        projekt_id: 2,
        rejestr_odnosnik: "pokoj-rejestru/sprawdzenie-1".to_string(),
    };
    expect::refused(
        kierownik_kontroli_projekty::dopisz_podstawe(&room, &basis_of_withdrawn),
        "podstawę dopisuje się tylko do projektu w toku",
    );

    let basis_again = DopiszPodstaweParams {
        projekt_id: 1,
        rejestr_odnosnik: "pokoj-rejestru/sprawdzenie-1".to_string(),
    };
    expect::refused(
        kierownik_kontroli_projekty::dopisz_podstawe(&room, &basis_again),
        REPLACEMENT,
    );
    expect::refused(
        room.execute(
            "UPDATE kierownik_kontroli_projekty_podstawy SET rejestr_odnosnik = 'inny' WHERE id = 1",
            &[],
        ),
        "podstawy projektu się nie zmienia",
    );
}

// Personal data -----------------------------------------------------------------------------------

/// sprawa danych osobowych powstaje otwarta, a załatwia ją odpowiedź właściciela
#[test]
fn data_request_needs_owner_answer() {
    let room = new_room();
    add_data_request(&room, &data_request());
    add_data_request(
        &room,
        &ZapiszSpraweDanychOsobowychParams {
            rodzaj: RodzajSprawyDanych::Naruszenie,
            termin: "2026-11-07T09:00:00Z".to_string(),
            ..data_request()
        },
    );

    expect::refused(
        room.execute(
            "INSERT INTO kierownik_kontroli_dane_osobowe_wpisy (rodzaj, wplynelo, wplyw_odnosnik,
              termin, stan, odpowiedz_odnosnik, zalatwiono)
            VALUES ('sprzeciw_art21', '2026-11-04', 'w', '2026-12-04', 'zalatwiona', 'o',
              '2026-11-05')",
            &[],
        ),
        "sprawa danych osobowych powstaje otwarta",
    );

    let deadline_before_arrival = ZapiszSpraweDanychOsobowychParams {
        rodzaj: RodzajSprawyDanych::UsuniecieArt17,
        wplynelo: "2026-11-04T09:00:00Z".to_string(),
        wplyw_odnosnik: "w".to_string(),
        termin: "2026-11-01".to_string(),
    };
    expect::refused(
        kierownik_kontroli_dane_osobowe::zapisz_sprawe_danych_osobowych(
            &room,
            &deadline_before_arrival,
        ),
        "termin_nie_przed_wplywem",
    );

    // A writer outside the client can leave out the owner's answer that `odnotuj_zalatwienie`
    // requires. The table refuses the settlement without one.
    let settlement_without_answer = room.execute(
        "UPDATE kierownik_kontroli_dane_osobowe_wpisy
        SET stan = 'zalatwiona',
          odpowiedz_odnosnik = :odpowiedz_odnosnik,
          zalatwiono = :zalatwiono
        WHERE id = :id",
        &[
            ("odpowiedz_odnosnik", SqlValue::Null),
            ("zalatwiono", text("2026-11-20T09:00:00Z")),
            ("id", SqlValue::Integer(1)),
        ],
    );
    expect::refused(
        settlement_without_answer,
        "zalatwiona_z_odpowiedzia_wlasciciela",
    );
    expect::refused(
        kierownik_kontroli_dane_osobowe::odnotuj_zalatwienie(
            &room,
            &settlement("o", "2026-11-01T09:00:00Z"),
        ),
        "zalatwienie_nie_przed_wplywem",
    );
    kierownik_kontroli_dane_osobowe::odnotuj_zalatwienie(
        &room,
        &settlement("o", "2026-11-20T09:00:00Z"),
    )
    .expect("record the request settled");

    let rule = "a załatwionej wcale";
    expect::refused(
        kierownik_kontroli_dane_osobowe::daj_sprawe_wlascicielowi(
            &room,
            &DajSpraweWlascicielowiParams { id: 1 },
        ),
        rule,
    );
    expect::refused(
        room.execute(
            "UPDATE kierownik_kontroli_dane_osobowe_wpisy SET rodzaj = 'dostep_art15' WHERE id = 2",
            &[],
        ),
        "rodzaju ani wpływu sprawy danych osobowych się nie zmienia",
    );

    let data_requests = kierownik_kontroli_dane_osobowe::sprawy_danych_osobowych(&room)
        .expect("read the requests about personal data");
    let kinds: Vec<RodzajSprawyDanych> = data_requests.iter().map(|row| row.rodzaj).collect();
    assert_eq!(
        kinds,
        [
            RodzajSprawyDanych::Naruszenie,
            RodzajSprawyDanych::DostepArt15,
        ]
    );
}

// R9 and replacement ------------------------------------------------------------------------------

/// The head's own tables in the room.
fn own_tables_of(room: &MemoryRoom) -> Vec<String> {
    let rows = room
        .execute(
            r"SELECT name FROM sqlite_schema WHERE type = 'table'
            AND name LIKE 'kierownik\_kontroli\_%' ESCAPE '\'",
            &[],
        )
        .expect("read the tables");

    rows.iter()
        .map(|row| row.get::<String>("name").expect("a table has a name"))
        .collect()
}

/// R9: wierszy pamięci się nie usuwa ani nie zastępuje przez REPLACE albo upsert
#[test]
fn no_deletion_or_replacement() {
    let room = new_room();
    full_memory(&room);

    let tables = own_tables_of(&room);
    assert_eq!(tables.len(), 5, "{tables:?}");

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

/// REPLACE nie usuwa podstawy projektu przez klucz UNIQUE
#[test]
fn no_replacement_through_unique_basis() {
    let room = new_room();
    full_memory(&room);

    let table = "kierownik_kontroli_projekty_podstawy";
    let replace = format!(
        "INSERT OR REPLACE INTO {table} (id, projekt_id, rejestr_odnosnik) \
        SELECT 99, projekt_id, rejestr_odnosnik FROM {table}"
    );
    expect::refused(room.execute(&replace, &[]), REPLACEMENT);

    // `WHERE true` keeps SQLite from reading ON CONFLICT as a join constraint.
    let upsert = format!(
        "INSERT INTO {table} (id, projekt_id, rejestr_odnosnik) \
        SELECT 99, projekt_id, rejestr_odnosnik FROM {table} \
        WHERE true ON CONFLICT (projekt_id, rejestr_odnosnik) DO UPDATE SET id = excluded.id"
    );
    expect::refused(room.execute(&upsert, &[]), REPLACEMENT);
}
