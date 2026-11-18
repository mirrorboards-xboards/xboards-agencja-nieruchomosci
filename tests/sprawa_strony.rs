//! Scenarios of the core kinds between the office and the parties (spec/cases/sprawa):
//! powers of attorney (sprawa-umocowania), correspondence (sprawa-korespondencja), consents of
//! clients (sprawa-zgody) and contracts of clients with the office (sprawa-umowy).
//!
//! Each scenario starts from an empty room and checks one rule the database keeps: a write the rule
//! allows and a write it refuses.

use xboards_agencja_nieruchomosci::cases::sprawa::{
    sprawa_decyzje, sprawa_dokumenty, sprawa_korespondencja, sprawa_metryka, sprawa_sprawdzenia,
    sprawa_uczestnicy, sprawa_umocowania, sprawa_umowy, sprawa_zgody,
};
use xboards_agencja_nieruchomosci::xmeta_room::{
    FromSqlValue, MemoryRoom, MetaRoom, SqlValue, ToSqlValue,
};

use sprawa_decyzje::{
    PoprosODecyzjeParams, WpisyRodzaj as RodzajDecyzji, WpisyRozstrzygniecie as Rozstrzygniecie,
};
use sprawa_dokumenty::{
    OdnotujOtrzymanieDokumentuParams, OdrzucDokumentParams, OznaczDokumentJakoSprawdzonyParams,
    WpisyKanal as KanalDokumentu, WpisyPostac as PostacDokumentu, WpisyRodzaj as RodzajDokumentu,
    WpisyWymaga as WymagaDokumentu, WpiszOtrzymanyDokumentParams, WpiszWymaganyDokumentParams,
};
use sprawa_korespondencja::{
    AnulujWysylkeParams, DolaczDokumentDoWplywuParams, DolaczDokumentDoWysylkiParams,
    OdnotujDoreczenieParams, OdnotujWyslanieParams, OdnotujZwrotParams,
    PrzekazWysylkeDoZgodyParams, PrzygotujWysylkeParams, WpiszWplywParams, WplywyKanal,
    WysylkiAdresatRodzaj, WysylkiWImieniu, ZmienTrescWysylkiParams,
};
use sprawa_metryka::{PostawWatpliwoscParams, ZdejmijWatpliwoscParams};
use sprawa_sprawdzenia::{
    WpisyRodzaj as RodzajSprawdzenia, WpisyWynik as WynikSprawdzenia, WpiszSprawdzenieParams,
};
use sprawa_uczestnicy::{WpiszUczestnikaParams, ZmienDaneUczestnikaParams};
use sprawa_umocowania::{
    DopiszCzynnoscUmocowaniaParams, OdnotujWygasniecieUmocowaniaParams,
    WpisyForma as FormaUmocowania, WpisyRodzaj as RodzajUmocowania,
    WpisyWygasnieciePowod as PowodWygasniecia, WpisyZakres as ZakresUmocowania,
    WpiszUmocowanieParams, ZmienDaneUmocowaniaParams,
};
use sprawa_umowy::{
    CofnijUmoweDoProjektuParams, DopiszCzynnoscUmowyParams, DopiszStroneUmowyParams,
    OdnotujAneksUmowyParams, OdnotujInformacjeOOdstapieniuParams,
    OdnotujKoniecPoWypowiedzeniuParams, OdnotujNowaPoliseUmowyParams, OdnotujZawarcieUmowyParams,
    PrzekazUmoweDoPodpisuParams, PrzygotujUmoweParams, WpisyForma as FormaUmowy,
    WpisyRodzaj as RodzajUmowy, WpisySposobZawarcia as SposobZawarcia, WpiszZdarzenieUmowyParams,
    ZakonczUmoweParams, ZdarzeniaRodzaj as RodzajZdarzeniaUmowy, ZmienWarunkiUmowyParams,
};
use sprawa_zgody::WpiszZgodeParams;

use crate::expect;
use crate::sprawa::{
    WITHDRAWAL_ENDS, add_company, add_consent, add_inflow, add_person, add_power_for_office,
    add_received_document, approved_decision, ask_for_decision, check, check_identity, check_of,
    conclude_contract, consent, contract, decision_request_about, inflow, letter, new_room, person,
    power_version_of, prepare_contract, prepare_letter, received_document, send, sending,
    settle_decision, settlement,
};

/// A text value to bind in raw SQL.
fn text(value: &str) -> SqlValue {
    SqlValue::Text(value.to_string())
}

/// The default parameters of a participant (see [`person`]) with the first name `imiona`.
fn person_named(person_id: &str, imiona: &str) -> WpiszUczestnikaParams {
    WpiszUczestnikaParams {
        imiona: Some(imiona.to_string()),
        ..person(person_id)
    }
}

/// Hands the draft contract `contract_id` over for signing.
fn hand_over_for_signing(room: &impl MetaRoom, contract_id: &str) {
    let params = PrzekazUmoweDoPodpisuParams {
        id: contract_id.to_string(),
    };

    sprawa_umowy::przekaz_umowe_do_podpisu(room, &params)
        .expect("hand the contract over for signing");
}

/// Ends the contract `contract_id` for the reason `koniec_powod`.
fn end_contract(room: &impl MetaRoom, contract_id: &str, koniec_powod: &str) {
    let params = ZakonczUmoweParams {
        koniec_powod: koniec_powod.to_string(),
        id: contract_id.to_string(),
    };

    sprawa_umowy::zakoncz_umowe(room, &params).expect("end the contract");
}

/// Moves the draft letter `letter_id` to waiting for the human's consent.
fn hand_over_for_consent(room: &impl MetaRoom, letter_id: &str) {
    let params = PrzekazWysylkeDoZgodyParams {
        id: letter_id.to_string(),
    };

    sprawa_korespondencja::przekaz_wysylke_do_zgody(room, &params)
        .expect("hand the letter over for consent");
}

/// The default parameters of the sending of `letter_id` (see [`sending`]) with the decision
/// `decyzja_id`.
fn sending_with_decision(letter_id: &str, decyzja_id: &str) -> OdnotujWyslanieParams {
    OdnotujWyslanieParams {
        decyzja_id: Some(decyzja_id.to_string()),
        ..sending(letter_id)
    }
}

/// The default parameters of a letter (see [`letter`]) outside the templates.
fn letter_outside_template(letter_id: &str) -> PrzygotujWysylkeParams {
    PrzygotujWysylkeParams {
        wzor: None,
        wzor_wersja: None,
        ..letter(letter_id)
    }
}

/// Writes the approved decision `decision_id` of the kind „wyslanie” about the letter `letter_id`.
fn approved_sending(room: &impl MetaRoom, decision_id: &str, letter_id: &str) {
    let request = decision_request_about(
        decision_id,
        RodzajDecyzji::Wyslanie,
        "sprawa_korespondencja_wysylki",
        letter_id,
    );

    approved_decision(room, &request, "2026-11-03T12:00:00Z");
}

/// Writes the approved decision `zdjecie` to lift the doubt `watpliwosc`.
fn approved_lifting_of_doubt(room: &impl MetaRoom) {
    let request = decision_request_about(
        "zdjecie",
        RodzajDecyzji::ZdjecieWatpliwosci,
        "sprawa_metryka_watpliwosci",
        "watpliwosc",
    );

    approved_decision(room, &request, "2026-11-03T12:00:00Z");
}

/// Kontroler raises the doubt `watpliwosc` on 2026-11-03 at 09:00.
fn raise_doubt(room: &impl MetaRoom) {
    let params = PostawWatpliwoscParams {
        id: "watpliwosc".to_string(),
        postawil: "tozsamosc-kontrolera".to_string(),
        postawiono: "2026-11-03T09:00:00Z".to_string(),
    };

    sprawa_metryka::postaw_watpliwosc(room, &params).expect("raise the doubt");
}

/// Lifts the doubt `watpliwosc` with the decision `zdjecie`.
fn lift_doubt(room: &impl MetaRoom) {
    let params = ZdejmijWatpliwoscParams {
        zdjecie_decyzja_id: "zdjecie".to_string(),
        id: "watpliwosc".to_string(),
    };

    sprawa_metryka::zdejmij_watpliwosc(room, &params).expect("lift the doubt");
}

// Powers of attorney ------------------------------------------------------------------------------

/// The default parameters of the client's power of attorney `pelnomocnictwo` of a kind, in writing,
/// that names no holder yet.
fn power_fields() -> WpiszUmocowanieParams {
    WpiszUmocowanieParams {
        id: "pelnomocnictwo".to_string(),
        rodzaj: RodzajUmocowania::Pelnomocnictwo,
        mocodawca_uczestnik_id: "klientka".to_string(),
        umocowany_uczestnik_id: None,
        umocowane_biuro: false,
        zakres: ZakresUmocowania::Rodzajowe,
        forma: Some(FormaUmocowania::Pisemna),
        sposob_reprezentacji: None,
        dokument_id: None,
        udzielono: None,
        wazne_do: None,
    }
}

/// The change of a power to the data of `params`; the date of the grant does not change.
fn power_change_of(params: WpiszUmocowanieParams) -> ZmienDaneUmocowaniaParams {
    ZmienDaneUmocowaniaParams {
        rodzaj: params.rodzaj,
        mocodawca_uczestnik_id: params.mocodawca_uczestnik_id,
        umocowany_uczestnik_id: params.umocowany_uczestnik_id,
        umocowane_biuro: params.umocowane_biuro,
        zakres: params.zakres,
        forma: params.forma,
        sposob_reprezentacji: params.sposob_reprezentacji,
        dokument_id: params.dokument_id,
        wazne_do: params.wazne_do,
        id: params.id,
    }
}

/// The client's revocation of a power on 2026-11-10, from her message.
fn revocation(power_id: &str) -> OdnotujWygasniecieUmocowaniaParams {
    OdnotujWygasniecieUmocowaniaParams {
        wygaslo: "2026-11-10".to_string(),
        wygasniecie_powod: PowodWygasniecia::Odwolanie,
        wygasniecie_zrodlo: "wiadomość klientki z 10 listopada".to_string(),
        id: power_id.to_string(),
    }
}

/// Umocowanie: umocowany jest osobą albo biurem, nie oba naraz
#[test]
fn power_holder() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    add_person(&room, &person_named("syn", "Piotr"));

    expect::refused(
        sprawa_umocowania::wpisz_umocowanie(&room, &power_fields()),
        "umocowany_osoba_albo_biuro",
    );
    expect::refused(
        sprawa_umocowania::wpisz_umocowanie(
            &room,
            &WpiszUmocowanieParams {
                umocowany_uczestnik_id: Some("syn".to_string()),
                umocowane_biuro: true,
                ..power_fields()
            },
        ),
        "umocowany_osoba_albo_biuro",
    );
    expect::refused(
        sprawa_umocowania::wpisz_umocowanie(
            &room,
            &WpiszUmocowanieParams {
                umocowany_uczestnik_id: Some("klientka".to_string()),
                ..power_fields()
            },
        ),
        "nie_umocowuje_siebie",
    );

    sprawa_umocowania::wpisz_umocowanie(
        &room,
        &WpiszUmocowanieParams {
            umocowany_uczestnik_id: Some("syn".to_string()),
            ..power_fields()
        },
    )
    .expect("add the power for the son");
    sprawa_umocowania::wpisz_umocowanie(
        &room,
        &WpiszUmocowanieParams {
            id: "dla-biura".to_string(),
            umocowane_biuro: true,
            ..power_fields()
        },
    )
    .expect("add the power for the office");
}

/// Umocowanie: reprezentację ma tylko podmiot
#[test]
fn representation_of_entity() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    add_person(&room, &person_named("prezes", "Piotr"));
    add_company(&room, "spolka");
    add_received_document(
        &room,
        &received_document("odpis-krs", RodzajDokumentu::OdpisRejestru),
    );

    let representation_of_person = WpiszUmocowanieParams {
        rodzaj: RodzajUmocowania::Reprezentacja,
        umocowany_uczestnik_id: Some("prezes".to_string()),
        ..power_fields()
    };
    expect::refused(
        sprawa_umocowania::wpisz_umocowanie(&room, &representation_of_person),
        "reprezentację ma tylko podmiot",
    );

    sprawa_umocowania::wpisz_umocowanie(
        &room,
        &WpiszUmocowanieParams {
            id: "reprezentacja".to_string(),
            rodzaj: RodzajUmocowania::Reprezentacja,
            mocodawca_uczestnik_id: "spolka".to_string(),
            umocowany_uczestnik_id: Some("prezes".to_string()),
            zakres: ZakresUmocowania::Ogolne,
            forma: Some(FormaUmocowania::ZRejestru),
            sposob_reprezentacji: Some("prezes zarządu samodzielnie".to_string()),
            dokument_id: Some("odpis-krs".to_string()),
            ..power_fields()
        },
    )
    .expect("add the representation of the company");

    let representation_moved_to_person = WpiszUmocowanieParams {
        id: "reprezentacja".to_string(),
        rodzaj: RodzajUmocowania::Reprezentacja,
        mocodawca_uczestnik_id: "klientka".to_string(),
        umocowany_uczestnik_id: Some("prezes".to_string()),
        ..power_fields()
    };
    expect::refused(
        sprawa_umocowania::zmien_dane_umocowania(
            &room,
            &power_change_of(representation_moved_to_person),
        ),
        "reprezentację ma tylko podmiot",
    );
}

/// Umocowanie: wygaśnięcie ma powód i źródło wiadomości
#[test]
fn power_expiry() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    add_power_for_office(
        &room,
        "pelnomocnictwo",
        "klientka",
        &["wystapienie_do_wspolnoty"],
        "2026-11-03",
    );

    expect::refused(
        room.execute(
            "UPDATE sprawa_umocowania_wpisy SET wygaslo = '2026-11-10'",
            &[],
        ),
        "wygasniecie_ma_powod_i_zrodlo",
    );

    // A writer outside the client can leave out the source that `odnotuj_wygasniecie_umocowania`
    // requires. The table refuses the expiry without one.
    let expiry_without_source = room.execute(
        "UPDATE sprawa_umocowania_wpisy
        SET wygaslo = :wygaslo,
          wygasniecie_powod = :wygasniecie_powod,
          wygasniecie_zrodlo = :wygasniecie_zrodlo
        WHERE id = :id",
        &[
            ("wygaslo", text("2026-11-10")),
            ("wygasniecie_powod", text("odwolanie")),
            ("wygasniecie_zrodlo", SqlValue::Null),
            ("id", text("pelnomocnictwo")),
        ],
    );
    expect::refused(expiry_without_source, "wygasniecie_ma_powod_i_zrodlo");

    sprawa_umocowania::odnotuj_wygasniecie_umocowania(&room, &revocation("pelnomocnictwo"))
        .expect("record the revocation");
}

/// Umocowanie: każda zmiana i nowa czynność podnosi wersję; czynności się nie zmienia (D6)
#[test]
fn power_version() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    add_power_for_office(&room, "pelnomocnictwo", "klientka", &[], "2026-11-03");
    assert_eq!(power_version_of(&room, "pelnomocnictwo"), 1);

    sprawa_umocowania::dopisz_czynnosc_umocowania(
        &room,
        &DopiszCzynnoscUmocowaniaParams {
            umocowanie_id: "pelnomocnictwo".to_string(),
            czynnosc: "odbior_odpisu".to_string(),
        },
    )
    .expect("add an act of the power");
    assert_eq!(power_version_of(&room, "pelnomocnictwo"), 2);

    let power_until_january = WpiszUmocowanieParams {
        umocowane_biuro: true,
        zakres: ZakresUmocowania::DoCzynnosci,
        wazne_do: Some("2027-01-31".to_string()),
        ..power_fields()
    };
    sprawa_umocowania::zmien_dane_umocowania(&room, &power_change_of(power_until_january))
        .expect("change the power");
    assert_eq!(power_version_of(&room, "pelnomocnictwo"), 3);

    expect::refused(
        sprawa_umocowania::dopisz_czynnosc_umocowania(
            &room,
            &DopiszCzynnoscUmocowaniaParams {
                umocowanie_id: "pelnomocnictwo".to_string(),
                czynnosc: "Odbiór odpisu".to_string(),
            },
        ),
        "czynnosc",
    );
    expect::refused(
        room.execute(
            "UPDATE sprawa_umocowania_czynnosci SET czynnosc = 'uzgodnienie_terminu_aktu'",
            &[],
        ),
        "czynności w zakresie umocowania się nie zmienia",
    );
}

// Correspondence ----------------------------------------------------------------------------------

/// Wysyłka: adresat to uczestnik albo instytucja z nazwą; adresata GIIF nie ma
#[test]
fn letter_addressee() {
    let room = new_room();
    add_person(&room, &person("klientka"));

    let two_addressees = PrzygotujWysylkeParams {
        adresat_uczestnik_id: Some("klientka".to_string()),
        adresat_rodzaj: Some(WysylkiAdresatRodzaj::Bank),
        adresat_nazwa: Some("Bank".to_string()),
        wzor: None,
        wzor_wersja: None,
        sciezka: None,
        skrot: None,
        ..letter("dwoch-adresatow")
    };
    expect::refused(
        sprawa_korespondencja::przygotuj_wysylke(&room, &two_addressees),
        "adresat_uczestnik_albo_instytucja",
    );
    let without_name = PrzygotujWysylkeParams {
        adresat_rodzaj: Some(WysylkiAdresatRodzaj::Bank),
        adresat_nazwa: None,
        wzor: None,
        wzor_wersja: None,
        sciezka: None,
        skrot: None,
        ..letter("bez-nazwy")
    };
    expect::refused(
        sprawa_korespondencja::przygotuj_wysylke(&room, &without_name),
        "adresat_uczestnik_albo_instytucja",
    );

    // A writer outside the client can give the addressee „giif”, which `WysylkiAdresatRodzaj`
    // leaves out. The table refuses it.
    let letter_to_giif = room.execute(
        "INSERT INTO sprawa_korespondencja_wysylki (id, adresat_uczestnik_id, adresat_rodzaj,
          adresat_nazwa, w_imieniu, w_imieniu_uczestnik_id, umocowanie_id, czynnosc, kanal, wzor,
          wzor_wersja, zamawia_dokument_id, sciezka, skrot, informacja_rodo, ujawnia_agenta, stan)
        VALUES (:id, :adresat_uczestnik_id, :adresat_rodzaj, :adresat_nazwa, :w_imieniu,
          :w_imieniu_uczestnik_id, :umocowanie_id, :czynnosc, :kanal, :wzor, :wzor_wersja,
          :zamawia_dokument_id, :sciezka, :skrot, :informacja_rodo, :ujawnia_agenta, 'projekt')",
        &[
            ("id", text("giif")),
            ("adresat_rodzaj", text("giif")),
            (
                "adresat_nazwa",
                text("Generalny Inspektor Informacji Finansowej"),
            ),
            ("w_imieniu", text("biuro")),
            ("kanal", text("list")),
            ("informacja_rodo", false.to_sql_value()),
            ("ujawnia_agenta", false.to_sql_value()),
        ],
    );
    expect::refused(letter_to_giif, "adresat_rodzaj");

    prepare_letter(&room, &letter("do-wspolnoty"));
    prepare_letter(
        &room,
        &PrzygotujWysylkeParams {
            adresat_rodzaj: None,
            adresat_nazwa: None,
            adresat_uczestnik_id: Some("klientka".to_string()),
            ..letter("do-klientki")
        },
    );
}

/// Wysyłka: powstaje jako projekt i idzie tylko naprzód
#[test]
fn letter_moves_forward() {
    let room = new_room();

    expect::refused(
        room.execute(
            "INSERT INTO sprawa_korespondencja_wysylki (id, adresat_rodzaj, adresat_nazwa, w_imieniu,
              kanal, wzor, wzor_wersja, sciezka, skrot, stan, wyslano)
            VALUES ('od-razu', 'bank', 'Bank', 'biuro', 'email', 'w', '1', 'w.pdf', 'sha256-w',
              'wyslana', '2026-11-05T10:00:00Z')",
            &[],
        ),
        "wysyłka powstaje jako projekt",
    );

    prepare_letter(&room, &letter("pismo"));
    send(&room, &sending("pismo"));
    expect::refused(
        sprawa_korespondencja::anuluj_wysylke(
            &room,
            &AnulujWysylkeParams {
                id: "pismo".to_string(),
            },
        ),
        "wysyłka idzie tylko naprzód",
    );
    sprawa_korespondencja::odnotuj_doreczenie(
        &room,
        &OdnotujDoreczenieParams {
            doreczono: "2026-11-06T10:00:00Z".to_string(),
            dowod_doreczenia: None,
            id: "pismo".to_string(),
        },
    )
    .expect("record the delivery");
    expect::refused(
        sprawa_korespondencja::odnotuj_zwrot(
            &room,
            &OdnotujZwrotParams {
                dowod_doreczenia: None,
                id: "pismo".to_string(),
            },
        ),
        "wysyłka idzie tylko naprzód",
    );
    expect::refused(
        room.execute(
            "UPDATE sprawa_korespondencja_wysylki SET stan = 'wyslana' WHERE id = 'pismo'",
            &[],
        ),
        "wysyłka idzie tylko naprzód",
    );

    prepare_letter(&room, &letter("niepotrzebne"));
    sprawa_korespondencja::anuluj_wysylke(
        &room,
        &AnulujWysylkeParams {
            id: "niepotrzebne".to_string(),
        },
    )
    .expect("cancel the letter");
    expect::refused(
        sprawa_korespondencja::przekaz_wysylke_do_zgody(
            &room,
            &PrzekazWysylkeDoZgodyParams {
                id: "niepotrzebne".to_string(),
            },
        ),
        "wysyłka idzie tylko naprzód",
    );
}

/// Wysyłka: pismo spoza wzorów czeka na zgodę i wychodzi z zatwierdzoną decyzją, która je wskazuje
#[test]
fn letter_outside_templates() {
    let room = new_room();
    prepare_letter(&room, &letter_outside_template("pismo-wlasne"));
    prepare_letter(&room, &letter_outside_template("inne-pismo"));
    let refusal = "wysyłka za zgodą wychodzi tylko z zatwierdzoną decyzją wyslanie";

    expect::refused(
        sprawa_korespondencja::odnotuj_wyslanie(&room, &sending("pismo-wlasne")),
        "pismo spoza wzorów przed wysłaniem czeka na zgodę",
    );

    // A request for consent describes content that no longer changes.
    let too_early = PoprosODecyzjeParams {
        prosi: "agent".to_string(),
        decyduje: "czlowiek".to_string(),
        ..decision_request_about(
            "za-wczesnie",
            RodzajDecyzji::Wyslanie,
            "sprawa_korespondencja_wysylki",
            "pismo-wlasne",
        )
    };
    expect::refused(
        sprawa_decyzje::popros_o_decyzje(&room, &too_early),
        "o decyzję wyslanie prosi się dla wysyłki, która czeka na zgodę",
    );

    hand_over_for_consent(&room, "pismo-wlasne");
    hand_over_for_consent(&room, "inne-pismo");
    expect::refused(
        sprawa_korespondencja::odnotuj_wyslanie(&room, &sending("pismo-wlasne")),
        refusal,
    );

    // A decision the letter points at is checked as it is written (decyzja wysyłki to …).
    let wrong_decision = "decyzja wysyłki to zatwierdzona decyzja wyslanie, która ją wskazuje";
    approved_sending(&room, "zgoda-na-inne", "inne-pismo");
    expect::refused(
        sprawa_korespondencja::odnotuj_wyslanie(
            &room,
            &sending_with_decision("pismo-wlasne", "zgoda-na-inne"),
        ),
        wrong_decision,
    );

    ask_for_decision(
        &room,
        &decision_request_about(
            "odmowa",
            RodzajDecyzji::Wyslanie,
            "sprawa_korespondencja_wysylki",
            "pismo-wlasne",
        ),
    );
    settle_decision(
        &room,
        &settlement("odmowa", Rozstrzygniecie::Odmowa, "2026-11-03T12:00:00Z"),
    );
    expect::refused(
        sprawa_korespondencja::odnotuj_wyslanie(
            &room,
            &sending_with_decision("pismo-wlasne", "odmowa"),
        ),
        wrong_decision,
    );

    // The workaround from the audit: after the consent the agent swaps the file and sends.
    approved_sending(&room, "zgoda", "pismo-wlasne");
    expect::refused(
        sprawa_korespondencja::zmien_tresc_wysylki(
            &room,
            &ZmienTrescWysylkiParams {
                sciezka: Some("wysylki/b.pdf".to_string()),
                skrot: Some("sha256-b".to_string()),
                id: "pismo-wlasne".to_string(),
            },
        ),
        "treść wysyłki zmienia się tylko w projekcie",
    );
    send(&room, &sending_with_decision("pismo-wlasne", "zgoda"));
}

/// Wysyłka ze wzoru, która czeka na zgodę, wychodzi tylko z decyzją; bez zgody od razu (D15)
#[test]
fn letter_waiting_for_consent() {
    let room = new_room();
    prepare_letter(&room, &letter("ze-wzoru"));
    prepare_letter(&room, &letter("za-zgoda"));
    hand_over_for_consent(&room, "za-zgoda");

    send(&room, &sending("ze-wzoru"));
    expect::refused(
        sprawa_korespondencja::odnotuj_wyslanie(&room, &sending("za-zgoda")),
        "wysyłka za zgodą wychodzi tylko z zatwierdzoną decyzją wyslanie",
    );

    approved_sending(&room, "zgoda", "za-zgoda");
    send(&room, &sending_with_decision("za-zgoda", "zgoda"));
}

/// Wysyłka: decyzja to zatwierdzona decyzja wyslanie, która ją wskazuje, także przy wysłaniu z projektu
#[test]
fn letter_decision_points_at_letter() {
    let room = new_room();
    prepare_letter(&room, &letter("ze-wzoru"));
    prepare_letter(&room, &letter("za-zgoda"));
    hand_over_for_consent(&room, "za-zgoda");
    approved_sending(&room, "zgoda", "za-zgoda");
    let other_decision = decision_request_about(
        "inna",
        RodzajDecyzji::Inna,
        "sprawa_korespondencja_wysylki",
        "za-zgoda",
    );
    approved_decision(&room, &other_decision, "2026-11-03T12:00:00Z");
    let rule = "decyzja wysyłki to zatwierdzona decyzja wyslanie, która ją wskazuje";

    // The workaround from the builders: a letter from a draft sent with someone else's consent.
    expect::refused(
        sprawa_korespondencja::odnotuj_wyslanie(&room, &sending_with_decision("ze-wzoru", "zgoda")),
        rule,
    );
    expect::refused(
        room.execute(
            "UPDATE sprawa_korespondencja_wysylki SET decyzja_id = 'inna' WHERE id = 'za-zgoda'",
            &[],
        ),
        rule,
    );

    send(&room, &sending("ze-wzoru"));
    send(&room, &sending_with_decision("za-zgoda", "zgoda"));
}

/// Wysyłka: wzór się nie zmienia, a pismo spoza wzorów nie staje się pismem ze wzoru
#[test]
fn letter_template_fixed() {
    let room = new_room();
    prepare_letter(&room, &letter("ze-wzoru"));
    prepare_letter(&room, &letter_outside_template("spoza-wzorow"));
    let refusal = "wzór wysyłki się nie zmienia";

    expect::refused(
        room.execute(
            "UPDATE sprawa_korespondencja_wysylki SET wzor_wersja = '4' WHERE id = 'ze-wzoru'",
            &[],
        ),
        refusal,
    );

    // The workaround from the audit: a letter outside the templates names a template and goes
    // out without a decision in one write.
    expect::refused(
        room.execute(
            "UPDATE sprawa_korespondencja_wysylki
            SET wzor = 'cokolwiek', wzor_wersja = '9', stan = 'wyslana',
              wyslano = '2026-11-05T10:00:00Z'
            WHERE id = 'spoza-wzorow'",
            &[],
        ),
        refusal,
    );

    // Another template is a new draft; the old one is cancelled.
    sprawa_korespondencja::anuluj_wysylke(
        &room,
        &AnulujWysylkeParams {
            id: "ze-wzoru".to_string(),
        },
    )
    .expect("cancel the letter");
    prepare_letter(
        &room,
        &PrzygotujWysylkeParams {
            wzor_wersja: Some("4".to_string()),
            ..letter("ze-wzoru-4")
        },
    );
    send(&room, &sending("ze-wzoru-4"));
}

/// Wysyłka w imieniu uczestnika wymaga jego ważnego umocowania biura, które obejmuje czynność
#[test]
fn letter_on_behalf_of_participant() {
    let room = new_room();
    let for_the_association = ["wystapienie_do_wspolnoty"];
    add_person(&room, &person("klientka"));
    add_person(&room, &person_named("sasiad", "Piotr"));
    add_power_for_office(
        &room,
        "pelnomocnictwo",
        "klientka",
        &for_the_association,
        "2026-11-03",
    );
    add_power_for_office(
        &room,
        "od-sasiada",
        "sasiad",
        &for_the_association,
        "2026-11-03",
    );
    add_power_for_office(
        &room,
        "przeterminowane",
        "klientka",
        &for_the_association,
        "2026-11-03",
    );
    add_power_for_office(
        &room,
        "pozniejsze",
        "klientka",
        &for_the_association,
        "2026-11-10",
    );
    let expired_power = WpiszUmocowanieParams {
        id: "przeterminowane".to_string(),
        umocowane_biuro: true,
        zakres: ZakresUmocowania::DoCzynnosci,
        wazne_do: Some("2026-11-01".to_string()),
        ..power_fields()
    };
    sprawa_umocowania::zmien_dane_umocowania(&room, &power_change_of(expired_power))
        .expect("let the power expire on 2026-11-01");
    add_power_for_office(
        &room,
        "odwolane",
        "klientka",
        &for_the_association,
        "2026-11-03",
    );
    sprawa_umocowania::odnotuj_wygasniecie_umocowania(
        &room,
        &OdnotujWygasniecieUmocowaniaParams {
            wygaslo: "2026-11-04".to_string(),
            wygasniecie_zrodlo: "wiadomość klientki".to_string(),
            ..revocation("odwolane")
        },
    )
    .expect("record the revocation");
    sprawa_umocowania::wpisz_umocowanie(
        &room,
        &WpiszUmocowanieParams {
            id: "dla-syna".to_string(),
            umocowany_uczestnik_id: Some("sasiad".to_string()),
            zakres: ZakresUmocowania::DoCzynnosci,
            ..power_fields()
        },
    )
    .expect("add the power for the son");
    sprawa_umocowania::dopisz_czynnosc_umocowania(
        &room,
        &DopiszCzynnoscUmocowaniaParams {
            umocowanie_id: "dla-syna".to_string(),
            czynnosc: "wystapienie_do_wspolnoty".to_string(),
        },
    )
    .expect("add an act of the power");

    let without_power = PrzygotujWysylkeParams {
        adresat_nazwa: Some("Wspólnota".to_string()),
        w_imieniu: WysylkiWImieniu::Uczestnik,
        w_imieniu_uczestnik_id: Some("klientka".to_string()),
        wzor: None,
        wzor_wersja: None,
        sciezka: None,
        skrot: None,
        ..letter("bez-umocowania")
    };
    expect::refused(
        sprawa_korespondencja::przygotuj_wysylke(&room, &without_power),
        "w_imieniu_uczestnika_z_umocowaniem_i_czynnoscia",
    );

    let on_behalf = |letter_id: &str, umocowanie_id: &str, czynnosc: &str| {
        let params = PrzygotujWysylkeParams {
            w_imieniu: WysylkiWImieniu::Uczestnik,
            w_imieniu_uczestnik_id: Some("klientka".to_string()),
            umocowanie_id: Some(umocowanie_id.to_string()),
            czynnosc: Some(czynnosc.to_string()),
            ..letter(letter_id)
        };

        prepare_letter(&room, &params);
    };

    let refusal = "pismo w imieniu uczestnika wymaga jego niewygasłego umocowania biura";
    let letters_without_valid_power = [
        (
            "poza-zakresem",
            "pelnomocnictwo",
            "uzgodnienie_terminu_aktu",
        ),
        ("cudze", "od-sasiada", "wystapienie_do_wspolnoty"),
        ("po-terminie", "przeterminowane", "wystapienie_do_wspolnoty"),
        ("po-odwolaniu", "odwolane", "wystapienie_do_wspolnoty"),
        (
            "przed-udzieleniem",
            "pozniejsze",
            "wystapienie_do_wspolnoty",
        ),
        ("dla-osoby", "dla-syna", "wystapienie_do_wspolnoty"),
    ];
    for (letter_id, umocowanie_id, czynnosc) in letters_without_valid_power {
        on_behalf(letter_id, umocowanie_id, czynnosc);

        expect::refused(
            sprawa_korespondencja::odnotuj_wyslanie(&room, &sending(letter_id)),
            refusal,
        );
    }

    on_behalf("w-zakresie", "pelnomocnictwo", "wystapienie_do_wspolnoty");
    send(&room, &sending("w-zakresie"));
}

/// Wysyłka: treść i załączniki zmieniają się tylko w projekcie, a ślad wysłania raz
#[test]
fn letter_content_frozen_outside_draft() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    add_power_for_office(
        &room,
        "pelnomocnictwo",
        "klientka",
        &["wystapienie_do_wspolnoty"],
        "2026-11-03",
    );
    add_power_for_office(&room, "inne", "klientka", &["inna_czynnosc"], "2026-11-03");
    add_received_document(&room, &received_document("polisa", RodzajDokumentu::Polisa));
    add_received_document(&room, &received_document("aneks", RodzajDokumentu::Umowa));
    prepare_letter(
        &room,
        &PrzygotujWysylkeParams {
            w_imieniu: WysylkiWImieniu::Uczestnik,
            w_imieniu_uczestnik_id: Some("klientka".to_string()),
            umocowanie_id: Some("pelnomocnictwo".to_string()),
            czynnosc: Some("wystapienie_do_wspolnoty".to_string()),
            ..letter("pismo")
        },
    );

    sprawa_korespondencja::zmien_tresc_wysylki(
        &room,
        &ZmienTrescWysylkiParams {
            sciezka: Some("wysylki/pismo-2.pdf".to_string()),
            skrot: Some("sha256-2".to_string()),
            id: "pismo".to_string(),
        },
    )
    .expect("change the content of the draft");
    sprawa_korespondencja::dolacz_dokument_do_wysylki(
        &room,
        &DolaczDokumentDoWysylkiParams {
            wysylka_id: "pismo".to_string(),
            dokument_id: "polisa".to_string(),
        },
    )
    .expect("attach the policy");
    send(&room, &sending("pismo"));

    let refusal = "treść wysyłki zmienia się tylko w projekcie";
    expect::refused(
        sprawa_korespondencja::zmien_tresc_wysylki(
            &room,
            &ZmienTrescWysylkiParams {
                sciezka: Some("wysylki/pismo-3.pdf".to_string()),
                skrot: Some("sha256-3".to_string()),
                id: "pismo".to_string(),
            },
        ),
        refusal,
    );

    // The workaround from the audit: after sending, the trail is rewritten with another power,
    // another act, another addressee and an earlier moment.
    expect::refused(
        room.execute(
            "UPDATE sprawa_korespondencja_wysylki
            SET umocowanie_id = 'inne', czynnosc = 'inna_czynnosc', adresat_nazwa = 'Urząd'
            WHERE id = 'pismo'",
            &[],
        ),
        refusal,
    );
    expect::refused(
        room.execute(
            "UPDATE sprawa_korespondencja_wysylki SET wyslano = '2025-01-01' WHERE id = 'pismo'",
            &[],
        ),
        "decyzję, chwile wysłania i doręczenia, dowód doręczenia i odnośnik wysyłki wpisuje się raz",
    );
    expect::refused(
        sprawa_korespondencja::dolacz_dokument_do_wysylki(
            &room,
            &DolaczDokumentDoWysylkiParams {
                wysylka_id: "pismo".to_string(),
                dokument_id: "aneks".to_string(),
            },
        ),
        "załączniki dołącza się tylko do projektu wysyłki",
    );
    expect::refused(
        room.execute(
            "UPDATE sprawa_korespondencja_zalaczniki SET dokument_id = 'aneks'",
            &[],
        ),
        "załącznika się nie zmienia",
    );

    // The state, the moment and the proof of delivery still come in.
    sprawa_korespondencja::odnotuj_doreczenie(
        &room,
        &OdnotujDoreczenieParams {
            doreczono: "2026-11-05T12:00:00Z".to_string(),
            dowod_doreczenia: Some("potwierdzenie odbioru".to_string()),
            id: "pismo".to_string(),
        },
    )
    .expect("record the delivery");

    // A draft waiting for consent is frozen too, attachments included.
    prepare_letter(&room, &letter("czeka"));
    hand_over_for_consent(&room, "czeka");
    expect::refused(
        room.execute(
            "UPDATE sprawa_korespondencja_wysylki SET kanal = 'list' WHERE id = 'czeka'",
            &[],
        ),
        refusal,
    );
    expect::refused(
        sprawa_korespondencja::dolacz_dokument_do_wysylki(
            &room,
            &DolaczDokumentDoWysylkiParams {
                wysylka_id: "czeka".to_string(),
                dokument_id: "aneks".to_string(),
            },
        ),
        "załączniki dołącza się tylko do projektu wysyłki",
    );
    expect::refused(
        room.execute(
            "INSERT INTO sprawa_korespondencja_wysylki (id, adresat_rodzaj, adresat_nazwa, w_imieniu,
              kanal, wzor, wzor_wersja, stan, wyslano)
            VALUES ('z-chwila', 'urzad', 'Urząd', 'biuro', 'email', 'w', '1', 'projekt',
              '2026-11-05T10:00:00Z')",
            &[],
        ),
        "wysyłka powstaje jako projekt, bez decyzji i śladu wysłania",
    );
}

/// Wysyłka: wysłana ma plik i chwilę wysłania, doręczona chwilę doręczenia
#[test]
fn letter_sent_has_file() {
    let room = new_room();
    prepare_letter(
        &room,
        &PrzygotujWysylkeParams {
            sciezka: None,
            skrot: None,
            ..letter("bez-pliku")
        },
    );

    expect::refused(
        sprawa_korespondencja::odnotuj_wyslanie(&room, &sending("bez-pliku")),
        "wyslana_ma_plik_i_chwile",
    );

    prepare_letter(&room, &letter("pismo"));
    send(&room, &sending("pismo"));
    expect::refused(
        room.execute(
            "UPDATE sprawa_korespondencja_wysylki SET stan = 'doreczona' WHERE id = 'pismo'",
            &[],
        ),
        "doreczona_ma_chwile",
    );
}

/// Wpływ: niezgodny kontakt ma opis, a wpływów się nie zmienia
#[test]
fn inflow_rules() {
    let room = new_room();

    expect::refused(
        sprawa_korespondencja::wpisz_wplyw(
            &room,
            &WpiszWplywParams {
                niezgodny_kontakt: true,
                ..inflow_fields()
            },
        ),
        "niezgodny_kontakt_ma_opis",
    );
    sprawa_korespondencja::wpisz_wplyw(
        &room,
        &WpiszWplywParams {
            niezgodny_kontakt: true,
            niezgodnosc_opis: Some(
                "„kancelaria” pisze z adresu spoza kontaktów teczki".to_string(),
            ),
            ..inflow_fields()
        },
    )
    .expect("add the inflow from an unknown address");

    expect::refused(
        room.execute(
            "UPDATE sprawa_korespondencja_wplywy SET niezgodny_kontakt = 0, niezgodnosc_opis = NULL",
            &[],
        ),
        "wpływów się nie zmienia",
    );
}

/// The default parameters of an e-mail from a law firm, from a contact of the folder.
fn inflow_fields() -> WpiszWplywParams {
    WpiszWplywParams {
        id: "wplyw".to_string(),
        nadawca: "kancelaria@example.pl".to_string(),
        nadawca_uczestnik_id: None,
        kanal: WplywyKanal::Email,
        wplynelo: "2026-11-03T08:00:00Z".to_string(),
        sciezka: "wplywy/wplyw.eml".to_string(),
        skrot: "sha256-wplyw".to_string(),
        niezgodny_kontakt: false,
        niezgodnosc_opis: None,
        odnosnik_zewnetrzny: None,
    }
}

/// Załącznik należy do wysyłki albo do wpływu
#[test]
fn attachment_of_letter_or_inflow() {
    let room = new_room();
    add_received_document(
        &room,
        &received_document("zaswiadczenie", RodzajDokumentu::Zaswiadczenie),
    );
    prepare_letter(&room, &letter("pismo"));
    add_inflow(&room, &inflow("wplyw"));

    expect::refused(
        room.execute(
            "INSERT INTO sprawa_korespondencja_zalaczniki (wysylka_id, wplyw_id, dokument_id)
            VALUES ('pismo', 'wplyw', 'zaswiadczenie')",
            &[],
        ),
        "zalacznik_wysylki_albo_wplywu",
    );
    expect::refused(
        room.execute(
            "INSERT INTO sprawa_korespondencja_zalaczniki (dokument_id) VALUES ('zaswiadczenie')",
            &[],
        ),
        "zalacznik_wysylki_albo_wplywu",
    );

    sprawa_korespondencja::dolacz_dokument_do_wplywu(
        &room,
        &DolaczDokumentDoWplywuParams {
            wplyw_id: "wplyw".to_string(),
            dokument_id: "zaswiadczenie".to_string(),
        },
    )
    .expect("attach the certificate to the inflow");
    sprawa_korespondencja::dolacz_dokument_do_wysylki(
        &room,
        &DolaczDokumentDoWysylkiParams {
            wysylka_id: "pismo".to_string(),
            dokument_id: "zaswiadczenie".to_string(),
        },
    )
    .expect("attach the certificate to the letter");

    let attachments = sprawa_korespondencja::zalaczniki(&room).expect("read the attachments");
    assert_eq!(attachments.len(), 2);
}

// Consents ----------------------------------------------------------------------------------------

/// Zgoda ma dokument albo wiadomość, nie zmienia się, a nowa zastępuje poprzednią
#[test]
fn consent_rules() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    add_received_document(
        &room,
        &received_document("oswiadczenie", RodzajDokumentu::Oswiadczenie),
    );

    let price_consent = |consent_id: &str| WpiszZgodeParams {
        id: consent_id.to_string(),
        uczestnik_id: "klientka".to_string(),
        rodzaj: "zatwierdzenie_ceny".to_string(),
        tresc: "Zatwierdzam cenę ofertową 650 000 zł.".to_string(),
        zlozono: "2026-11-04T10:00:00Z".to_string(),
        dokument_id: None,
        wplyw_id: None,
        trwaly_nosnik: false,
        dotyczy_tabela: None,
        dotyczy_id: None,
        zastepuje_id: None,
    };
    expect::refused(
        sprawa_zgody::wpisz_zgode(&room, &price_consent("ustna")),
        "zgoda_ma_dokument_albo_wiadomosc",
    );
    expect::refused(
        sprawa_zgody::wpisz_zgode(
            &room,
            &WpiszZgodeParams {
                rodzaj: "Zatwierdzenie ceny".to_string(),
                ..price_consent("zly-rodzaj")
            },
        ),
        "rodzaj",
    );
    sprawa_zgody::wpisz_zgode(
        &room,
        &WpiszZgodeParams {
            dokument_id: Some("oswiadczenie".to_string()),
            ..price_consent("cena-1")
        },
    )
    .expect("add the consent to the price");

    expect::refused(
        room.execute(
            "UPDATE sprawa_zgody_wpisy SET tresc = 'Zatwierdzam cenę 600 000 zł.'",
            &[],
        ),
        "zgody się nie zmienia",
    );

    add_inflow(
        &room,
        &WpiszWplywParams {
            nadawca_uczestnik_id: Some("klientka".to_string()),
            ..inflow("nowa-cena")
        },
    );
    sprawa_zgody::wpisz_zgode(
        &room,
        &WpiszZgodeParams {
            wplyw_id: Some("nowa-cena".to_string()),
            zastepuje_id: Some("cena-1".to_string()),
            tresc: "Zatwierdzam cenę 640 000 zł.".to_string(),
            zlozono: "2026-11-06T10:00:00Z".to_string(),
            ..price_consent("cena-2")
        },
    )
    .expect("add the consent to the new price");

    let current_consents = sprawa_zgody::zgody_biezace(&room).expect("read the consents");
    let consent_ids: Vec<&str> = current_consents.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(consent_ids, ["cena-2"]);

    expect::refused(
        sprawa_zgody::wpisz_zgode(
            &room,
            &WpiszZgodeParams {
                wplyw_id: Some("nowa-cena".to_string()),
                zastepuje_id: Some("cena-3".to_string()),
                ..price_consent("cena-3")
            },
        ),
        "nie_zastepuje_samej_siebie",
    );
}

// Contracts with the office -----------------------------------------------------------------------

/// Umowa: powstaje jako projekt i przechodzi tylko dozwolonymi krokami
#[test]
fn contract_states() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    check_identity(&room, "tozsamosc", "klientka");

    expect::refused(
        room.execute(
            "INSERT INTO sprawa_umowy_wpisy (id, rodzaj, stan, obie_strony, wylacznosc)
            VALUES ('od-razu', 'posrednictwo', 'zawarta', 0, 0)",
            &[],
        ),
        "umowa powstaje jako projekt",
    );

    prepare_contract(&room, &contract("umowa"), &["klientka"]);
    add_received_document(
        &room,
        &received_document("podpisana-wczesniej", RodzajDokumentu::UmowaZBiurem),
    );
    add_received_document(
        &room,
        &received_document("polisa-wczesniej", RodzajDokumentu::Polisa),
    );
    let conclusion_of_draft = OdnotujZawarcieUmowyParams {
        forma: FormaUmowy::Pisemna,
        sposob_zawarcia: SposobZawarcia::WLokaluBiura,
        zawarto: "2026-11-04T12:00:00Z".to_string(),
        podpisal_za_biuro: "tozsamosc-posrednika".to_string(),
        dokument_id: "podpisana-wczesniej".to_string(),
        polisa_dokument_id: "polisa-wczesniej".to_string(),
        odstapienie_do: None,
        id: "umowa".to_string(),
    };
    expect::refused(
        sprawa_umowy::odnotuj_zawarcie_umowy(&room, &conclusion_of_draft),
        "umowa przechodzi",
    );
    let back_to_draft = CofnijUmoweDoProjektuParams {
        id: "umowa".to_string(),
    };
    hand_over_for_signing(&room, "umowa");
    sprawa_umowy::cofnij_umowe_do_projektu(&room, &back_to_draft)
        .expect("take the contract back to the draft");
    hand_over_for_signing(&room, "umowa");
    conclude_contract(&room, "umowa", SposobZawarcia::WLokaluBiura, None);
    expect::refused(
        sprawa_umowy::cofnij_umowe_do_projektu(&room, &back_to_draft),
        "umowa przechodzi",
    );

    expect::refused(
        room.execute(
            "UPDATE sprawa_umowy_wpisy SET stan = 'zakonczona' WHERE id = 'umowa'",
            &[],
        ),
        "zakonczona_ma_powod",
    );
    end_contract(&room, "umowa", "wypowiedzenie przez klientkę");
    expect::refused(
        sprawa_umowy::przekaz_umowe_do_podpisu(
            &room,
            &PrzekazUmoweDoPodpisuParams {
                id: "umowa".to_string(),
            },
        ),
        "umowa przechodzi",
    );
}

/// The change of the identifying data of the natural person `person_id` to `imiona` `nazwisko`.
fn identity_change(person_id: &str, imiona: &str, nazwisko: &str) -> ZmienDaneUczestnikaParams {
    ZmienDaneUczestnikaParams {
        imiona: Some(imiona.to_string()),
        nazwisko: Some(nazwisko.to_string()),
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
        id: person_id.to_string(),
    }
}

/// Umowa idzie do podpisu, gdy każda strona ma aktualne sprawdzenie tożsamości „zgodne” (R7)
#[test]
fn contract_to_signing_after_identity_checks() {
    let room = new_room();
    add_person(&room, &person("zona"));
    add_person(&room, &person_named("maz", "Piotr"));
    let refusal =
        "umowa idzie do podpisu, gdy ma strony, a każda ma aktualne sprawdzenie tożsamości";
    let for_signing = |contract_id: &str| {
        let params = PrzekazUmoweDoPodpisuParams {
            id: contract_id.to_string(),
        };

        sprawa_umowy::przekaz_umowe_do_podpisu(&room, &params)
    };

    prepare_contract(&room, &contract("bez-stron"), &[]);
    expect::refused(for_signing("bez-stron"), refusal);

    prepare_contract(&room, &contract("umowa"), &["zona", "maz"]);
    check_identity(&room, "tozsamosc-zony", "zona");
    expect::refused(for_signing("umowa"), refusal);

    let check_with_remarks = WpiszSprawdzenieParams {
        uczestnik_id: Some("maz".to_string()),
        ..check_of(
            "tozsamosc-meza",
            RodzajSprawdzenia::Tozsamosc,
            "2026-11-03T09:00:00Z",
            WynikSprawdzenia::Uwagi,
        )
    };
    check(&room, &check_with_remarks);
    expect::refused(for_signing("umowa"), refusal);

    check_identity(&room, "tozsamosc-meza-2", "maz");
    sprawa_uczestnicy::zmien_dane_uczestnika(&room, &identity_change("zona", "Anna", "Nowak"))
        .expect("change the wife's surname");
    expect::refused(for_signing("umowa"), refusal);

    check_identity(&room, "tozsamosc-zony-2", "zona");
    for_signing("umowa").expect("hand the contract over for signing");

    add_person(&room, &person_named("corka", "Ewa"));
    expect::refused(
        sprawa_umowy::dopisz_strone_umowy(
            &room,
            &DopiszStroneUmowyParams {
                umowa_id: "umowa".to_string(),
                uczestnik_id: "corka".to_string(),
            },
        ),
        "strony dopisuje się tylko do projektu umowy",
    );
    expect::refused(
        room.execute("UPDATE sprawa_umowy_strony SET uczestnik_id = 'zona'", &[]),
        "strony umowy się nie zmienia",
    );
}

/// Umowa: dopóki stoi znacznik wątpliwości, nie idzie do podpisu
#[test]
fn contract_blocked_by_doubt() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    check_identity(&room, "tozsamosc", "klientka");
    prepare_contract(&room, &contract("umowa"), &["klientka"]);
    raise_doubt(&room);

    expect::refused(
        sprawa_umowy::przekaz_umowe_do_podpisu(
            &room,
            &PrzekazUmoweDoPodpisuParams {
                id: "umowa".to_string(),
            },
        ),
        "dopóki stoi znacznik wątpliwości, umowa nie idzie do podpisu",
    );

    approved_lifting_of_doubt(&room);
    lift_doubt(&room);
    hand_over_for_signing(&room, "umowa");
}

/// The values of the statement of `odnotuj_zawarcie_umowy` for the conclusion `concluded`, with
/// the value `left_out` bound NULL.
fn conclusion_without(
    concluded: &OdnotujZawarcieUmowyParams,
    left_out: &str,
) -> Vec<(&'static str, SqlValue)> {
    let values = [
        ("forma", concluded.forma.to_sql_value()),
        ("sposob_zawarcia", concluded.sposob_zawarcia.to_sql_value()),
        ("zawarto", concluded.zawarto.to_sql_value()),
        (
            "podpisal_za_biuro",
            concluded.podpisal_za_biuro.to_sql_value(),
        ),
        ("dokument_id", concluded.dokument_id.to_sql_value()),
        (
            "polisa_dokument_id",
            concluded.polisa_dokument_id.to_sql_value(),
        ),
        ("odstapienie_do", concluded.odstapienie_do.to_sql_value()),
        ("id", concluded.id.to_sql_value()),
    ];

    values
        .into_iter()
        .map(|(name, value)| {
            if name == left_out {
                (name, SqlValue::Null)
            } else {
                (name, value)
            }
        })
        .collect()
}

/// The statement of `odnotuj_zawarcie_umowy`.
const CONCLUSION: &str = "UPDATE sprawa_umowy_wpisy
    SET stan = 'zawarta',
      forma = :forma,
      sposob_zawarcia = :sposob_zawarcia,
      zawarto = :zawarto,
      podpisal_za_biuro = :podpisal_za_biuro,
      dokument_id = :dokument_id,
      polisa_dokument_id = :polisa_dokument_id,
      warunki_od = :zawarto,
      odstapienie_do = :odstapienie_do
    WHERE id = :id";

/// Umowa zawarta ma podpisany dokument umowy z biurem, kopię polisy, formę i sposób zawarcia
#[test]
fn contract_concluded_with_policy() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    check_identity(&room, "tozsamosc", "klientka");
    prepare_contract(&room, &contract("umowa"), &["klientka"]);
    hand_over_for_signing(&room, "umowa");
    add_received_document(
        &room,
        &received_document("podpisana", RodzajDokumentu::UmowaZBiurem),
    );
    add_received_document(&room, &received_document("polisa", RodzajDokumentu::Polisa));
    add_received_document(
        &room,
        &received_document("nie-polisa", RodzajDokumentu::Zaswiadczenie),
    );
    sprawa_dokumenty::wpisz_wymagany_dokument(
        &room,
        &WpiszWymaganyDokumentParams {
            id: "polisa-w-drodze".to_string(),
            rodzaj: RodzajDokumentu::Polisa,
            nazwa: "Polisa OC".to_string(),
            dotyczy_uczestnik_id: None,
            dotyczy_nieruchomosc_id: None,
            wymaga: None,
        },
    )
    .expect("require the policy");

    let concluded = OdnotujZawarcieUmowyParams {
        forma: FormaUmowy::Elektroniczna,
        sposob_zawarcia: SposobZawarcia::NaOdleglosc,
        zawarto: "2026-11-04T12:00:00Z".to_string(),
        podpisal_za_biuro: "tozsamosc-posrednika".to_string(),
        dokument_id: "podpisana".to_string(),
        polisa_dokument_id: "polisa".to_string(),
        odstapienie_do: None,
        id: "umowa".to_string(),
    };
    add_received_document(&room, &received_document("pismo", RodzajDokumentu::Pismo));
    let refusal =
        "umowa zawarta ma podpisany dokument umowy z biurem, kopię polisy, formę i sposób";

    expect::refused(
        sprawa_umowy::odnotuj_zawarcie_umowy(
            &room,
            &OdnotujZawarcieUmowyParams {
                dokument_id: "pismo".to_string(),
                ..concluded.clone()
            },
        ),
        refusal,
    );
    // A writer outside the client can leave out the policy that `odnotuj_zawarcie_umowy`
    // requires. The table refuses the conclusion without one.
    expect::refused(
        room.execute(
            CONCLUSION,
            &conclusion_without(&concluded, "polisa_dokument_id"),
        ),
        refusal,
    );
    expect::refused(
        sprawa_umowy::odnotuj_zawarcie_umowy(
            &room,
            &OdnotujZawarcieUmowyParams {
                polisa_dokument_id: "nie-polisa".to_string(),
                ..concluded.clone()
            },
        ),
        refusal,
    );
    expect::refused(
        sprawa_umowy::odnotuj_zawarcie_umowy(
            &room,
            &OdnotujZawarcieUmowyParams {
                polisa_dokument_id: "polisa-w-drodze".to_string(),
                ..concluded.clone()
            },
        ),
        refusal,
    );
    // A writer outside the client can leave out the signed document that
    // `odnotuj_zawarcie_umowy` requires. The table refuses the conclusion without one.
    expect::refused(
        room.execute(CONCLUSION, &conclusion_without(&concluded, "dokument_id")),
        refusal,
    );
    // A writer outside the client can leave out the way of conclusion that
    // `odnotuj_zawarcie_umowy` requires. The table refuses the conclusion without one.
    expect::refused(
        room.execute(
            CONCLUSION,
            &conclusion_without(&concluded, "sposob_zawarcia"),
        ),
        refusal,
    );

    sprawa_umowy::odnotuj_zawarcie_umowy(&room, &concluded).expect("conclude the contract");
    let contracts = sprawa_umowy::umowy(&room).expect("read the contracts");
    assert_eq!(
        contracts[0].warunki_od.as_deref(),
        Some("2026-11-04T12:00:00Z")
    );
    expect::refused(
        room.execute(
            "UPDATE sprawa_umowy_wpisy SET forma = NULL WHERE id = 'umowa'",
            &[],
        ),
        "dane zawarcia umowy się nie zmieniają",
    );
}

/// Żądanie rozpoczęcia pracy wskazuje zgodę strony, a przy umowie poza lokalem trwały nośnik
#[test]
fn contract_early_start_request() {
    let room = new_room();
    add_person(
        &room,
        &WpiszUczestnikaParams {
            konsument: true,
            ..person("klientka")
        },
    );
    add_person(&room, &person_named("sasiad", "Piotr"));
    check_identity(&room, "tozsamosc", "klientka");
    prepare_contract(&room, &contract("poza-lokalem"), &["klientka"]);
    hand_over_for_signing(&room, "poza-lokalem");
    conclude_contract(
        &room,
        "poza-lokalem",
        SposobZawarcia::PozaLokalem,
        Some(WITHDRAWAL_ENDS),
    );
    prepare_contract(&room, &contract("na-odleglosc"), &["klientka"]);
    hand_over_for_signing(&room, "na-odleglosc");
    conclude_contract(
        &room,
        "na-odleglosc",
        SposobZawarcia::NaOdleglosc,
        Some(WITHDRAWAL_ENDS),
    );

    add_consent(
        &room,
        &consent("zadanie-sasiada", "sasiad", "zadanie_rozpoczecia_pracy"),
    );
    add_consent(
        &room,
        &consent("zadanie-w-mailu", "klientka", "zadanie_rozpoczecia_pracy"),
    );
    add_consent(&room, &consent("cena", "klientka", "zatwierdzenie_ceny"));
    add_received_document(
        &room,
        &received_document("podpisane-zadanie", RodzajDokumentu::Oswiadczenie),
    );
    sprawa_zgody::wpisz_zgode(
        &room,
        &WpiszZgodeParams {
            id: "zadanie-na-nosniku".to_string(),
            uczestnik_id: "klientka".to_string(),
            rodzaj: "zadanie_rozpoczecia_pracy".to_string(),
            tresc:
                "Żądam rozpoczęcia pracy i wiem, że tracę prawo odstąpienia po pełnym wykonaniu."
                    .to_string(),
            zlozono: "2026-11-04T13:00:00Z".to_string(),
            dokument_id: Some("podpisane-zadanie".to_string()),
            wplyw_id: None,
            trwaly_nosnik: true,
            dotyczy_tabela: None,
            dotyczy_id: None,
            zastepuje_id: None,
        },
    )
    .expect("add the demand on a durable medium");

    let request =
        |event_id: &str, umowa_id: &str, zgoda_id: Option<&str>| WpiszZdarzenieUmowyParams {
            id: event_id.to_string(),
            umowa_id: umowa_id.to_string(),
            rodzaj: RodzajZdarzeniaUmowy::ZadanieRozpoczeciaPracy,
            chwila: "2026-11-04T13:00:00Z".to_string(),
            opis: None,
            dokument_id: None,
            wysylka_id: None,
            wplyw_id: None,
            zgoda_id: zgoda_id.map(str::to_string),
        };

    let refusal = "żądanie rozpoczęcia pracy wskazuje zgodę zadanie_rozpoczecia_pracy strony umowy";
    expect::refused(
        sprawa_umowy::wpisz_zdarzenie_umowy(
            &room,
            &WpiszZdarzenieUmowyParams {
                rodzaj: RodzajZdarzeniaUmowy::Aneks,
                ..request("aneks-bez-dowodu", "na-odleglosc", None)
            },
        ),
        "zdarzenie_ma_dowod",
    );
    expect::refused(
        sprawa_umowy::wpisz_zdarzenie_umowy(
            &room,
            &request("cudze", "na-odleglosc", Some("zadanie-sasiada")),
        ),
        refusal,
    );
    expect::refused(
        sprawa_umowy::wpisz_zdarzenie_umowy(&room, &request("cena", "na-odleglosc", Some("cena"))),
        refusal,
    );
    sprawa_umowy::wpisz_zdarzenie_umowy(
        &room,
        &request("zadanie-1", "na-odleglosc", Some("zadanie-w-mailu")),
    )
    .expect("record the demand to start work");

    expect::refused(
        sprawa_umowy::wpisz_zdarzenie_umowy(
            &room,
            &request("zadanie-2", "poza-lokalem", Some("zadanie-w-mailu")),
        ),
        "przy umowie poza lokalem żądanie rozpoczęcia pracy jest na trwałym nośniku",
    );
    sprawa_umowy::wpisz_zdarzenie_umowy(
        &room,
        &request("zadanie-3", "poza-lokalem", Some("zadanie-na-nosniku")),
    )
    .expect("record the demand on a durable medium");

    expect::refused(
        room.execute(
            "UPDATE sprawa_umowy_zdarzenia SET chwila = '2026-11-03T13:00:00Z'",
            &[],
        ),
        "zdarzenia umowy się nie zmienia",
    );
}

/// Odstąpienie od umowy wskazuje oświadczenie konsumenta, który jest jej stroną
#[test]
fn contract_withdrawal() {
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
    for party_id in ["konsumentka", "przedsiebiorca"] {
        let contract_id = format!("umowa-{party_id}");
        check_identity(&room, &format!("tozsamosc-{party_id}"), party_id);
        prepare_contract(&room, &contract(&contract_id), &[party_id]);
        hand_over_for_signing(&room, &contract_id);

        // Only the consumer may withdraw, so only her contract has the last day to do it.
        let odstapienie_do = if party_id == "konsumentka" {
            Some(WITHDRAWAL_ENDS)
        } else {
            None
        };
        conclude_contract(
            &room,
            &contract_id,
            SposobZawarcia::NaOdleglosc,
            odstapienie_do,
        );

        add_consent(
            &room,
            &WpiszZgodeParams {
                tresc: "Odstępuję od umowy pośrednictwa.".to_string(),
                ..consent(&format!("odstapienie-{party_id}"), party_id, "odstapienie")
            },
        );
    }

    let withdrawal = |party_id: &str| WpiszZdarzenieUmowyParams {
        id: format!("zdarzenie-{party_id}"),
        umowa_id: format!("umowa-{party_id}"),
        rodzaj: RodzajZdarzeniaUmowy::Odstapienie,
        chwila: "2026-11-10T09:00:00Z".to_string(),
        opis: None,
        dokument_id: None,
        wysylka_id: None,
        wplyw_id: None,
        zgoda_id: Some(format!("odstapienie-{party_id}")),
    };

    expect::refused(
        sprawa_umowy::wpisz_zdarzenie_umowy(&room, &withdrawal("przedsiebiorca")),
        "odstąpienie wskazuje oświadczenie odstapienie konsumenta",
    );
    sprawa_umowy::wpisz_zdarzenie_umowy(&room, &withdrawal("konsumentka"))
        .expect("record the withdrawal");
    // The contract had no end; the withdrawal gives it one without an annex.
    sprawa_umowy::odnotuj_koniec_po_wypowiedzeniu(
        &room,
        &OdnotujKoniecPoWypowiedzeniuParams {
            koniec: "2026-11-10".to_string(),
            id: "umowa-konsumentka".to_string(),
        },
    )
    .expect("record the end after the withdrawal");
    end_contract(&room, "umowa-konsumentka", "odstąpienie konsumentki");
}

/// Writes the contract `contract_id` (see [`contract`]) until 2027-11-04 whose one party `party_id`
/// Kontroler checked (`tozsamosc-<id>`), hands it over for signing and concludes it at the office.
fn concluded_contract_for(room: &impl MetaRoom, contract_id: &str, party_id: &str) {
    check_identity(room, &format!("tozsamosc-{contract_id}"), party_id);

    let until_november = PrzygotujUmoweParams {
        koniec: Some("2027-11-04".to_string()),
        ..contract(contract_id)
    };
    prepare_contract(room, &until_november, &[party_id]);

    hand_over_for_signing(room, contract_id);
    conclude_contract(room, contract_id, SposobZawarcia::WLokaluBiura, None);
}

/// Zdarzenia umowy powstają tylko przy umowie zawartej albo zakończonej
#[test]
fn contract_events_only_after_conclusion() {
    let room = new_room();
    add_person(
        &room,
        &WpiszUczestnikaParams {
            konsument: true,
            ..person("klientka")
        },
    );
    add_consent(
        &room,
        &consent("zadanie-w-mailu", "klientka", "zadanie_rozpoczecia_pracy"),
    );
    check_identity(&room, "tozsamosc", "klientka");
    prepare_contract(&room, &contract("projekt"), &["klientka"]);
    prepare_contract(&room, &contract("bez-podpisu"), &["klientka"]);
    end_contract(&room, "bez-podpisu", "klientka nie podpisała");
    let refusal = "zdarzenie powstaje tylko przy umowie zawartej albo zakończonej";
    let request = |umowa_id: &str| WpiszZdarzenieUmowyParams {
        id: format!("zadanie-{umowa_id}"),
        umowa_id: umowa_id.to_string(),
        rodzaj: RodzajZdarzeniaUmowy::ZadanieRozpoczeciaPracy,
        chwila: "2026-11-04T13:00:00Z".to_string(),
        opis: None,
        dokument_id: None,
        wysylka_id: None,
        wplyw_id: None,
        zgoda_id: Some("zadanie-w-mailu".to_string()),
    };

    // The workaround from the audit: a request without a durable medium on the draft, before the
    // way of conclusion is known.
    for umowa_id in ["projekt", "bez-podpisu"] {
        expect::refused(
            sprawa_umowy::wpisz_zdarzenie_umowy(&room, &request(umowa_id)),
            refusal,
        );
    }

    hand_over_for_signing(&room, "projekt");
    conclude_contract(
        &room,
        "projekt",
        SposobZawarcia::NaOdleglosc,
        Some(WITHDRAWAL_ENDS),
    );
    sprawa_umowy::wpisz_zdarzenie_umowy(
        &room,
        &WpiszZdarzenieUmowyParams {
            id: "zadanie".to_string(),
            ..request("projekt")
        },
    )
    .expect("record the demand to start work");
}

/// The event `event_id` of the kind `rodzaj` of the contract `umowa`, at `chwila`, proven by the
/// document `dokument_id`.
fn event_with_document(
    event_id: &str,
    rodzaj: RodzajZdarzeniaUmowy,
    chwila: &str,
    dokument_id: &str,
) -> WpiszZdarzenieUmowyParams {
    WpiszZdarzenieUmowyParams {
        id: event_id.to_string(),
        umowa_id: "umowa".to_string(),
        rodzaj,
        chwila: chwila.to_string(),
        opis: None,
        dokument_id: Some(dokument_id.to_string()),
        wysylka_id: None,
        wplyw_id: None,
        zgoda_id: None,
    }
}

/// Umowa po zawarciu: dane zawarcia stoją, warunki zmienia aneks, a polisę nowa polisa
#[test]
fn contract_frozen_after_conclusion() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    add_received_document(
        &room,
        &received_document("podpisany-aneks", RodzajDokumentu::UmowaZBiurem),
    );
    add_received_document(
        &room,
        &received_document("nowa-polisa", RodzajDokumentu::Polisa),
    );
    concluded_contract_for(&room, "umowa", "klientka");
    let zawarcia = "dane zawarcia umowy się nie zmieniają";

    // The workaround from the audit: the conclusion rewritten after the fact.
    expect::refused(
        room.execute(
            "UPDATE sprawa_umowy_wpisy
            SET zawarto = '2020-01-01', sposob_zawarcia = 'poza_lokalem',
              forma = 'elektroniczna', podpisal_za_biuro = 'ktos'
            WHERE id = 'umowa'",
            &[],
        ),
        zawarcia,
    );
    prepare_letter(
        &room,
        &PrzygotujWysylkeParams {
            wzor: Some("informacja-o-odstapieniu".to_string()),
            ..letter("spozniona-informacja")
        },
    );
    expect::refused(
        sprawa_umowy::odnotuj_informacje_o_odstapieniu(
            &room,
            &OdnotujInformacjeOOdstapieniuParams {
                informacja_o_odstapieniu_wysylka_id: "spozniona-informacja".to_string(),
                id: "umowa".to_string(),
            },
        ),
        zawarcia,
    );

    let aneks = "warunki zawartej umowy zmienia tylko aneks";
    let annex = |warunki_od: &str| OdnotujAneksUmowyParams {
        koniec: Some("2028-11-04".to_string()),
        okres_wypowiedzenia: Some("3 miesiące".to_string()),
        wylacznosc: false,
        wynagrodzenie_opis: Some("8% czynszu miesięcznie".to_string()),
        wynagrodzenie_zdarzenie: None,
        warunki_od: warunki_od.to_string(),
        id: "umowa".to_string(),
    };
    // The change of terms keeps `obie_strony` as the contract has it („false”), so only the rule of
    // annexes refuses it.
    let terms_change = ZmienWarunkiUmowyParams {
        nieruchomosc_id: None,
        poczatek: None,
        koniec: Some("2028-11-04".to_string()),
        okres_wypowiedzenia: Some("3 miesiące".to_string()),
        obie_strony: false,
        wylacznosc: false,
        wynagrodzenie_opis: Some("8% czynszu miesięcznie".to_string()),
        wynagrodzenie_zdarzenie: None,
        id: "umowa".to_string(),
    };
    expect::refused(
        room.execute(
            "UPDATE sprawa_umowy_wpisy SET wylacznosc = 0 WHERE id = 'umowa'",
            &[],
        ),
        aneks,
    );
    expect::refused(
        sprawa_umowy::odnotuj_aneks_umowy(&room, &annex("2026-12-01T10:00:00Z")),
        aneks,
    );
    expect::refused(
        sprawa_umowy::zmien_warunki_umowy(&room, &terms_change),
        aneks,
    );

    sprawa_umowy::wpisz_zdarzenie_umowy(
        &room,
        &event_with_document(
            "aneks-1",
            RodzajZdarzeniaUmowy::Aneks,
            "2026-12-01T10:00:00Z",
            "podpisany-aneks",
        ),
    )
    .expect("record the annex");
    expect::refused(
        sprawa_umowy::odnotuj_aneks_umowy(&room, &annex("2026-12-02T10:00:00Z")),
        aneks,
    );
    sprawa_umowy::odnotuj_aneks_umowy(&room, &annex("2026-12-01T10:00:00Z"))
        .expect("change the terms by the annex");
    let contracts = sprawa_umowy::umowy(&room).expect("read the contracts");
    assert!(!contracts[0].wylacznosc);

    // The same annex does not change the terms a second time.
    expect::refused(
        sprawa_umowy::odnotuj_aneks_umowy(
            &room,
            &OdnotujAneksUmowyParams {
                wylacznosc: true,
                ..annex("2026-12-01T10:00:00Z")
            },
        ),
        aneks,
    );

    let polisa = "polisę zawartej umowy zmienia tylko zdarzenie nowa_polisa";
    let new_policy = OdnotujNowaPoliseUmowyParams {
        polisa_dokument_id: "nowa-polisa".to_string(),
        id: "umowa".to_string(),
    };
    expect::refused(
        sprawa_umowy::odnotuj_nowa_polise_umowy(&room, &new_policy),
        polisa,
    );
    sprawa_umowy::wpisz_zdarzenie_umowy(
        &room,
        &event_with_document(
            "polisa-2027",
            RodzajZdarzeniaUmowy::NowaPolisa,
            "2027-01-02T10:00:00Z",
            "nowa-polisa",
        ),
    )
    .expect("record the new policy");
    sprawa_umowy::odnotuj_nowa_polise_umowy(&room, &new_policy)
        .expect("point the contract to the new policy");

    end_contract(&room, "umowa", "wypowiedzenie");
    expect::refused(
        room.execute("UPDATE sprawa_umowy_wpisy SET poczatek = '2026-01-01'", &[]),
        zawarcia,
    );
}

/// Umowa po wypowiedzeniu albo odstąpieniu: koniec wpisuje się albo skraca bez aneksu, nie przed dniem zdarzenia
#[test]
fn contract_end_after_notice() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    add_received_document(
        &room,
        &received_document("pismo-wypowiedzenia", RodzajDokumentu::Pismo),
    );
    add_received_document(
        &room,
        &received_document("podpisany-aneks", RodzajDokumentu::UmowaZBiurem),
    );
    concluded_contract_for(&room, "umowa", "klientka");
    let aneks = "warunki zawartej umowy zmienia tylko aneks";
    let rule = "po wypowiedzeniu albo odstąpieniu koniec umowy wpisuje się albo skraca";
    let end_on = |koniec: &str| OdnotujKoniecPoWypowiedzeniuParams {
        koniec: koniec.to_string(),
        id: "umowa".to_string(),
    };

    // Without a notice the end is a term like any other.
    expect::refused(
        sprawa_umowy::odnotuj_koniec_po_wypowiedzeniu(&room, &end_on("2027-01-31")),
        aneks,
    );

    sprawa_umowy::wpisz_zdarzenie_umowy(
        &room,
        &event_with_document(
            "wypowiedzenie",
            RodzajZdarzeniaUmowy::Wypowiedzenie,
            "2026-12-01T10:00:00Z",
            "pismo-wypowiedzenia",
        ),
    )
    .expect("record the notice");
    expect::refused(
        sprawa_umowy::odnotuj_koniec_po_wypowiedzeniu(&room, &end_on("2026-11-30")),
        rule,
    );
    expect::refused(
        sprawa_umowy::odnotuj_koniec_po_wypowiedzeniu(&room, &end_on("2028-01-31")),
        rule,
    );
    // A writer outside the client can leave out the end that `odnotuj_koniec_po_wypowiedzeniu`
    // requires. The database refuses to clear the end.
    let end_left_out = room.execute(
        "UPDATE sprawa_umowy_wpisy SET koniec = :koniec WHERE id = :id",
        &[("koniec", SqlValue::Null), ("id", text("umowa"))],
    );
    expect::refused(end_left_out, rule);
    // The other terms still change only by an annex.
    expect::refused(
        room.execute(
            "UPDATE sprawa_umowy_wpisy SET koniec = '2027-01-31', wylacznosc = 0 WHERE id = 'umowa'",
            &[],
        ),
        aneks,
    );

    sprawa_umowy::odnotuj_koniec_po_wypowiedzeniu(&room, &end_on("2027-01-31"))
        .expect("record the end after the notice");
    sprawa_umowy::odnotuj_koniec_po_wypowiedzeniu(&room, &end_on("2026-12-01"))
        .expect("shorten the end after the notice");
    let contracts = sprawa_umowy::umowy(&room).expect("read the contracts");
    assert_eq!(contracts[0].koniec.as_deref(), Some("2026-12-01"));

    // An annex after the notice sets new terms; the earlier notice no longer shortens them.
    sprawa_umowy::wpisz_zdarzenie_umowy(
        &room,
        &event_with_document(
            "aneks",
            RodzajZdarzeniaUmowy::Aneks,
            "2026-12-01T12:00:00Z",
            "podpisany-aneks",
        ),
    )
    .expect("record the annex");
    sprawa_umowy::odnotuj_aneks_umowy(
        &room,
        &OdnotujAneksUmowyParams {
            koniec: Some("2027-11-04".to_string()),
            okres_wypowiedzenia: None,
            wylacznosc: true,
            wynagrodzenie_opis: Some("10% czynszu miesięcznie".to_string()),
            wynagrodzenie_zdarzenie: None,
            warunki_od: "2026-12-01T12:00:00Z".to_string(),
            id: "umowa".to_string(),
        },
    )
    .expect("change the terms by the annex");
    expect::refused(
        sprawa_umowy::odnotuj_koniec_po_wypowiedzeniu(&room, &end_on("2027-01-31")),
        aneks,
    );
}

/// Umowa: rodzaj nie zmienia się od wpisu, także w projekcie; inny rodzaj to nowy projekt
#[test]
fn contract_kind_fixed() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    prepare_contract(&room, &contract("umowa"), &["klientka"]);
    let rule = "rodzaj umowy się nie zmienia";

    expect::refused(
        room.execute(
            "UPDATE sprawa_umowy_wpisy SET rodzaj = 'posrednictwo' WHERE id = 'umowa'",
            &[],
        ),
        rule,
    );

    sprawa_umowy::zmien_warunki_umowy(
        &room,
        &ZmienWarunkiUmowyParams {
            nieruchomosc_id: None,
            poczatek: None,
            koniec: None,
            okres_wypowiedzenia: None,
            obie_strony: false,
            wylacznosc: false,
            wynagrodzenie_opis: Some("8% czynszu miesięcznie".to_string()),
            wynagrodzenie_zdarzenie: None,
            id: "umowa".to_string(),
        },
    )
    .expect("change the terms of the draft");
    prepare_contract(
        &room,
        &PrzygotujUmoweParams {
            rodzaj: RodzajUmowy::Posrednictwo,
            ..contract("posrednictwo")
        },
        &["klientka"],
    );

    let contracts = sprawa_umowy::umowy(&room).expect("read the contracts");
    let kinds: Vec<RodzajUmowy> = contracts.iter().map(|row| row.rodzaj).collect();
    assert_eq!(kinds, [RodzajUmowy::Zarzadzanie, RodzajUmowy::Posrednictwo]);
}

/// Dokument zawartej umowy i dowód zgody nie zmieniają rodzaju i nie są odrzucane
#[test]
fn basis_documents_stay() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    add_person(&room, &person_named("sasiad", "Piotr"));
    concluded_contract_for(&room, "umowa", "klientka");
    add_received_document(
        &room,
        &WpiszOtrzymanyDokumentParams {
            dotyczy_uczestnik_id: Some("klientka".to_string()),
            ..received_document("oswiadczenie", RodzajDokumentu::Oswiadczenie)
        },
    );
    sprawa_zgody::wpisz_zgode(
        &room,
        &WpiszZgodeParams {
            id: "cena".to_string(),
            uczestnik_id: "klientka".to_string(),
            rodzaj: "zatwierdzenie_ceny".to_string(),
            tresc: "Zatwierdzam cenę.".to_string(),
            zlozono: "2026-11-04T10:00:00Z".to_string(),
            dokument_id: Some("oswiadczenie".to_string()),
            wplyw_id: None,
            trwaly_nosnik: true,
            dotyczy_tabela: None,
            dotyczy_id: None,
            zastepuje_id: None,
        },
    )
    .expect("add the consent to the price");

    let umowy = "dokument, na którym stoi zawarta umowa, nie zmienia rodzaju";
    let zgody = "dokument, który dowodzi zgody, nie zmienia rodzaju";
    let basis_documents = [
        ("podpisana-umowa", umowy),
        ("polisa-umowa", umowy),
        ("oswiadczenie", zgody),
    ];
    for (document_id, refusal) in basis_documents {
        expect::refused(
            sprawa_dokumenty::odrzuc_dokument(
                &room,
                &OdrzucDokumentParams {
                    powod_odrzucenia: "nieczytelny".to_string(),
                    id: document_id.to_string(),
                },
            ),
            refusal,
        );
        expect::refused(
            room.execute(
                "UPDATE sprawa_dokumenty_wpisy SET rodzaj = 'inny' WHERE id = :id",
                &[("id", text(document_id))],
            ),
            refusal,
        );
        expect::refused(
            room.execute(
                "UPDATE sprawa_dokumenty_wpisy SET dotyczy_uczestnik_id = 'sasiad' WHERE id = :id",
                &[("id", text(document_id))],
            ),
            refusal,
        );

        sprawa_dokumenty::oznacz_dokument_jako_sprawdzony(
            &room,
            &OznaczDokumentJakoSprawdzonyParams {
                id: document_id.to_string(),
            },
        )
        .expect("mark the document as checked");
    }

    // A document no contract or consent stands on is still rejected as before.
    add_received_document(&room, &received_document("luzny", RodzajDokumentu::Pismo));
    sprawa_dokumenty::odrzuc_dokument(
        &room,
        &OdrzucDokumentParams {
            powod_odrzucenia: "nieczytelny".to_string(),
            id: "luzny".to_string(),
        },
    )
    .expect("reject the loose document");
}

/// Zgoda wskazuje dokument z plikiem: otrzymany albo sprawdzony
#[test]
fn consent_document_has_file() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    sprawa_dokumenty::wpisz_wymagany_dokument(
        &room,
        &WpiszWymaganyDokumentParams {
            id: "oswiadczenie".to_string(),
            rodzaj: RodzajDokumentu::Oswiadczenie,
            nazwa: "Oświadczenie o cenie".to_string(),
            dotyczy_uczestnik_id: None,
            dotyczy_nieruchomosc_id: None,
            wymaga: Some(WymagaDokumentu::Umowa),
        },
    )
    .expect("require the statement");
    let price_consent = WpiszZgodeParams {
        id: "cena".to_string(),
        uczestnik_id: "klientka".to_string(),
        rodzaj: "zatwierdzenie_ceny".to_string(),
        tresc: "Zatwierdzam cenę.".to_string(),
        zlozono: "2026-11-04T10:00:00Z".to_string(),
        dokument_id: Some("oswiadczenie".to_string()),
        wplyw_id: None,
        trwaly_nosnik: false,
        dotyczy_tabela: None,
        dotyczy_id: None,
        zastepuje_id: None,
    };

    // The workaround from the audit: a consent standing on a required document without a file.
    expect::refused(
        sprawa_zgody::wpisz_zgode(&room, &price_consent),
        "zgoda wskazuje dokument z plikiem: otrzymany albo sprawdzony",
    );

    sprawa_dokumenty::odnotuj_otrzymanie_dokumentu(
        &room,
        &OdnotujOtrzymanieDokumentuParams {
            wystawca: None,
            data_wystawienia: None,
            waznosc_do: None,
            waznosc_zrodlo: None,
            pochodzenie: Some("od klientki".to_string()),
            kanal: Some(KanalDokumentu::Email),
            postac: Some(PostacDokumentu::Skan),
            sciezka: "dokumenty/oswiadczenie.pdf".to_string(),
            skrot: "sha256-oswiadczenie".to_string(),
            id: "oswiadczenie".to_string(),
        },
    )
    .expect("record the receipt of the statement");
    sprawa_zgody::wpisz_zgode(&room, &price_consent).expect("add the consent to the price");
}

/// Umocowanie: data udzielenia się nie zmienia, a wygaśnięcie wpisuje się raz
#[test]
fn power_granted_and_expiry_fixed() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    add_power_for_office(
        &room,
        "pelnomocnictwo",
        "klientka",
        &["wystapienie_do_wspolnoty"],
        "2026-11-03",
    );
    sprawa_umocowania::dopisz_czynnosc_umocowania(
        &room,
        &DopiszCzynnoscUmocowaniaParams {
            umocowanie_id: "pelnomocnictwo".to_string(),
            czynnosc: "inna_czynnosc".to_string(),
        },
    )
    .expect("add an act of the power");

    // The workaround from the audit: a grant moved after the sending date.
    expect::refused(
        room.execute(
            "UPDATE sprawa_umocowania_wpisy SET udzielono = '2027-01-01' WHERE id = 'pelnomocnictwo'",
            &[],
        ),
        "data udzielenia umocowania się nie zmienia",
    );

    sprawa_umocowania::odnotuj_wygasniecie_umocowania(&room, &revocation("pelnomocnictwo"))
        .expect("record the revocation");

    // The workaround from the audit: a revoked power revived by clearing its expiry.
    let once = "wygaśnięcie umocowania wpisuje się raz i się go nie cofa";
    expect::refused(
        room.execute(
            "UPDATE sprawa_umocowania_wpisy
            SET wygaslo = NULL, wygasniecie_powod = NULL, wygasniecie_zrodlo = NULL
            WHERE id = 'pelnomocnictwo'",
            &[],
        ),
        once,
    );
    expect::refused(
        sprawa_umocowania::odnotuj_wygasniecie_umocowania(
            &room,
            &OdnotujWygasniecieUmocowaniaParams {
                wygaslo: "2026-12-01".to_string(),
                ..revocation("pelnomocnictwo")
            },
        ),
        once,
    );
}

/// Zakres umowy dopisuje się tylko do projektu
#[test]
fn contract_scope_only_in_draft() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    check_identity(&room, "tozsamosc", "klientka");
    prepare_contract(&room, &contract("umowa"), &["klientka"]);
    sprawa_umowy::dopisz_czynnosc_umowy(
        &room,
        &DopiszCzynnoscUmowyParams {
            umowa_id: "umowa".to_string(),
            czynnosc: "publikacja_ogloszenia".to_string(),
        },
    )
    .expect("add an act to the scope");
    hand_over_for_signing(&room, "umowa");

    let negotiation = DopiszCzynnoscUmowyParams {
        umowa_id: "umowa".to_string(),
        czynnosc: "negocjacje_ceny".to_string(),
    };
    expect::refused(
        sprawa_umowy::dopisz_czynnosc_umowy(&room, &negotiation),
        "zakres dopisuje się tylko do projektu umowy",
    );
    conclude_contract(&room, "umowa", SposobZawarcia::WLokaluBiura, None);
    expect::refused(
        sprawa_umowy::dopisz_czynnosc_umowy(&room, &negotiation),
        "zakres dopisuje się tylko do projektu umowy",
    );
}

/// Zawarcie umowy sprawdza jeszcze raz tożsamość stron i znacznik wątpliwości
#[test]
fn contract_conclusion_rechecks_signing_conditions() {
    let room = new_room();
    add_person(&room, &person("klientka"));
    check_identity(&room, "tozsamosc", "klientka");
    prepare_contract(&room, &contract("umowa"), &["klientka"]);
    hand_over_for_signing(&room, "umowa");
    let refusal = "umowa zostaje zawarta, gdy każda strona ma aktualne sprawdzenie tożsamości";

    // The workaround from the audit: identity data changed after the hand-over to signing.
    sprawa_uczestnicy::zmien_dane_uczestnika(
        &room,
        &identity_change("klientka", "Anna", "Zmieniona"),
    )
    .expect("change the client's surname");
    expect::refused(
        sprawa_umowy::odnotuj_zawarcie_umowy(&room, &concluded_fields(&room, "umowa")),
        refusal,
    );

    check_identity(&room, "tozsamosc-2", "klientka");
    raise_doubt(&room);
    expect::refused(
        sprawa_umowy::odnotuj_zawarcie_umowy(&room, &concluded_fields(&room, "umowa")),
        refusal,
    );

    approved_lifting_of_doubt(&room);
    lift_doubt(&room);
    sprawa_umowy::odnotuj_zawarcie_umowy(&room, &concluded_fields(&room, "umowa"))
        .expect("conclude the contract");
}

/// The conclusion of a contract with its signed document and policy, added when missing.
fn concluded_fields(room: &MemoryRoom, contract_id: &str) -> OdnotujZawarcieUmowyParams {
    let signed_id = format!("podpisana-{contract_id}");
    let basis_documents = [
        (signed_id.as_str(), RodzajDokumentu::UmowaZBiurem),
        ("polisa", RodzajDokumentu::Polisa),
    ];

    for (document_id, rodzaj) in basis_documents {
        let count = room
            .value(
                "SELECT count(*) FROM sprawa_dokumenty_wpisy WHERE id = :id",
                &[("id", text(document_id))],
            )
            .expect("count the documents");
        let exists = i64::from_sql_value(&count).expect("the count is an integer") > 0;
        if exists {
            continue;
        }

        add_received_document(room, &received_document(document_id, rodzaj));
    }

    OdnotujZawarcieUmowyParams {
        forma: FormaUmowy::Pisemna,
        sposob_zawarcia: SposobZawarcia::WLokaluBiura,
        zawarto: "2026-11-04T12:00:00Z".to_string(),
        podpisal_za_biuro: "tozsamosc-posrednika".to_string(),
        dokument_id: signed_id,
        polisa_dokument_id: "polisa".to_string(),
        odstapienie_do: None,
        id: contract_id.to_string(),
    }
}

/// Znacznik konsumenta strony umowy, która wyszła z projektu, się nie zmienia
#[test]
fn consumer_flag_of_party_fixed() {
    let room = new_room();
    add_person(
        &room,
        &WpiszUczestnikaParams {
            konsument: true,
            ..person("klientka")
        },
    );
    check_identity(&room, "tozsamosc", "klientka");
    prepare_contract(&room, &contract("umowa"), &["klientka"]);
    let refusal = "znacznik konsumenta strony umowy, która wyszła z projektu, się nie zmienia";

    // In the draft the flag is still corrected from the client's statement.
    room.execute(
        "UPDATE sprawa_uczestnicy_wpisy SET konsument = 0 WHERE id = 'klientka'",
        &[],
    )
    .expect("clear the consumer flag in the draft");
    room.execute(
        "UPDATE sprawa_uczestnicy_wpisy SET konsument = 1 WHERE id = 'klientka'",
        &[],
    )
    .expect("set the consumer flag in the draft");

    hand_over_for_signing(&room, "umowa");
    expect::refused(
        room.execute(
            "UPDATE sprawa_uczestnicy_wpisy SET konsument = 0 WHERE id = 'klientka'",
            &[],
        ),
        refusal,
    );

    // The workaround from the audit: the flag cleared after the withdrawal of a concluded contract.
    conclude_contract(
        &room,
        "umowa",
        SposobZawarcia::NaOdleglosc,
        Some(WITHDRAWAL_ENDS),
    );
    add_consent(
        &room,
        &WpiszZgodeParams {
            tresc: "Odstępuję od umowy.".to_string(),
            ..consent("odstapienie", "klientka", "odstapienie")
        },
    );
    sprawa_umowy::wpisz_zdarzenie_umowy(
        &room,
        &WpiszZdarzenieUmowyParams {
            id: "zdarzenie-odstapienia".to_string(),
            umowa_id: "umowa".to_string(),
            rodzaj: RodzajZdarzeniaUmowy::Odstapienie,
            chwila: "2026-11-06T10:00:00Z".to_string(),
            opis: None,
            dokument_id: None,
            wysylka_id: None,
            wplyw_id: None,
            zgoda_id: Some("odstapienie".to_string()),
        },
    )
    .expect("record the withdrawal");
    end_contract(&room, "umowa", "odstąpienie konsumentki");
    expect::refused(
        room.execute(
            "UPDATE sprawa_uczestnicy_wpisy SET konsument = 0 WHERE id = 'klientka'",
            &[],
        ),
        refusal,
    );
}
