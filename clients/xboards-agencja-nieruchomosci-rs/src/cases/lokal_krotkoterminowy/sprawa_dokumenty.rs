//! The kind `sprawa-dokumenty` of the room `cases/lokal_krotkoterminowy`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "sprawa-dokumenty";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Dokumenty teczki, od wymaganego do sprawdzonego, i źródła danych. Typ sprawy mówi, jakie
-- dokumenty są w nim potrzebne. Plik leży w repozytorium XGit pokoju, a tabela trzyma jego ścieżkę
-- i skrót treści, który obejmuje podpis decyzji. Dokumenty prowadzi agent sprawy.
--
-- Skan dowodu osobistego zostaje w teczce jak każdy inny plik. Kopię skanu jako dokumentację
-- środków bezpieczeństwa finansowego Kontroler dołącza do raportu, a Kierownik kontroli przenosi
-- ją do rejestru kontroli; w teczce nikt pliku nie usuwa ani nie czyści.
--
-- Wniosek biura o dokument w imieniu strony to wysyłka z zamawia_dokument_id i umocowaniem
-- (sprawa-korespondencja).
--
-- Dokument, na którym stoi zawarta umowa, zgoda albo podpisany protokół, nie zmienia rodzaju ani
-- tego, czego dotyczy, i nie staje się odrzucony ani zbędny. Te reguły leżą w migracjach
-- sprawa-umowy (sprawa_umowy_dokument_zawartej_umowy_zostaje), sprawa-zgody
-- (sprawa_zgody_dokument_zgody_zostaje) i sprawa-protokoly
-- (sprawa_protokoly_dokument_podpisanego_zostaje).
CREATE TABLE sprawa_dokumenty_wpisy (
  id TEXT PRIMARY KEY,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('tozsamosc', 'odpis_ksiegi_wieczystej', 'odpis_rejestru', 'zaswiadczenie', 'akt_notarialny', 'umowa', 'umowa_z_biurem', 'pelnomocnictwo', 'polisa', 'oswiadczenie', 'protokol', 'akt_stanu_cywilnego', 'decyzja_kredytowa', 'swiadectwo_energetyczne', 'potwierdzenie_bankowe', 'regulamin', 'standard', 'instrukcja', 'uchwala', 'pismo', 'inny')),
  nazwa TEXT NOT NULL,
  -- Kogo albo czego dokument dotyczy: uczestnika albo nieruchomości.
  dotyczy_uczestnik_id TEXT REFERENCES sprawa_uczestnicy_wpisy (id),
  dotyczy_nieruchomosc_id TEXT REFERENCES sprawa_nieruchomosci_wpisy (id),
  wystawca TEXT,
  data_wystawienia TEXT,
  waznosc_do TEXT,
  -- Skąd wiadomo, do kiedy dokument jest ważny: od wystawcy, z przepisu albo od instytucji, która
  -- go wymaga (np. notariusz przyjmuje zaświadczenie spółdzielni przez 30 dni).
  waznosc_zrodlo TEXT CHECK (waznosc_zrodlo IN ('wystawca', 'przepis', 'instytucja')),
  -- Kto wymaga dokumentu. Agent nie dokłada do listy własnych wymagań.
  wymaga TEXT CHECK (wymaga IN ('przepis', 'notariusz', 'bank', 'umowa')),
  -- Od kogo dokument przyszedł, np. „od klientki”, „pobrany z centralnej bazy ksiąg wieczystych”.
  pochodzenie TEXT,
  kanal TEXT CHECK (kanal IN ('email', 'sms', 'portal', 'kanal_sprzedazy', 'list', 'pokoj_wymiany', 'osobiscie', 'rejestr_elektroniczny')),
  postac TEXT CHECK (postac IN ('oryginal', 'kopia', 'skan', 'odpis', 'wydruk')),
  stan TEXT NOT NULL CHECK (stan IN ('wymagany', 'zamowiony', 'otrzymany', 'sprawdzony', 'odrzucony', 'zbedny')),
  powod_odrzucenia TEXT,
  sciezka TEXT,
  skrot TEXT,

  CONSTRAINT dotyczy_uczestnika_albo_nieruchomosci CHECK (
    dotyczy_uczestnik_id IS NULL OR dotyczy_nieruchomosc_id IS NULL
  ),
  CONSTRAINT plik_ze_skrotem CHECK ((sciezka IS NULL) = (skrot IS NULL)),
  CONSTRAINT otrzymany_ma_plik CHECK (stan NOT IN ('otrzymany', 'sprawdzony') OR sciezka IS NOT NULL),
  CONSTRAINT odrzucony_ma_powod CHECK ((stan = 'odrzucony') = (powod_odrzucenia IS NOT NULL)),
  CONSTRAINT waznosc_ze_zrodlem CHECK (waznosc_do IS NULL OR waznosc_zrodlo IS NOT NULL),
  CONSTRAINT waznosc_nie_przed_wystawieniem CHECK (
    waznosc_do IS NULL OR data_wystawienia IS NULL OR waznosc_do >= data_wystawienia
  )
) STRICT;

-- Źródło jednej danej teczki (zasada „Każda wartość ma źródło”): pole wiersza rdzenia albo typu
-- sprawy i dokument, z którego pochodzi, albo osoba, od której biuro je ma, np. „od klienta,
-- niepotwierdzone”. Służy też typom spraw.
CREATE TABLE sprawa_dokumenty_zrodla (
  -- Klucz nadany przez bazę jest dodatni; zob. sprawa_dokumenty_zrodla_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Wiersz i pole, bez klucza obcego, bo może to być tabela typu sprawy.
  dotyczy_tabela TEXT NOT NULL,
  dotyczy_id TEXT NOT NULL,
  pole TEXT NOT NULL,
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  od_uczestnik_id TEXT REFERENCES sprawa_uczestnicy_wpisy (id),
  -- Osoba spoza teczki, od której dana pochodzi, np. „zarządca budynku, telefonicznie”.
  od_kogo TEXT,
  potwierdzone INTEGER NOT NULL CHECK (potwierdzone IN (0, 1)),
  wpisano TEXT NOT NULL,

  CONSTRAINT zrodlo_wskazuje_dokument_albo_osobe CHECK (
    (dokument_id IS NOT NULL) + (od_uczestnik_id IS NOT NULL) + (od_kogo IS NOT NULL) = 1
  )
) STRICT;

-- Wymagany, zamówiony albo otrzymany: reguły dalszych stanów sprawdzają zmianę.
CREATE TRIGGER sprawa_dokumenty_powstaje_w_stanie_poczatkowym
BEFORE INSERT ON sprawa_dokumenty_wpisy
WHEN NEW.stan NOT IN ('wymagany', 'zamowiony', 'otrzymany')
BEGIN
  SELECT RAISE(ABORT, 'dokument powstaje jako wymagany, zamówiony albo otrzymany');
END;

-- Na plik powołują się podpisy decyzji, wysyłki i źródła, więc plik się nie zmienia; nowa kopia
-- dokumentu to nowy wiersz.
CREATE TRIGGER sprawa_dokumenty_plik_niezmienny
BEFORE UPDATE OF sciezka, skrot ON sprawa_dokumenty_wpisy
WHEN OLD.sciezka IS NOT NULL
  AND (NEW.sciezka IS NOT OLD.sciezka OR NEW.skrot IS NOT OLD.skrot)
BEGIN
  SELECT RAISE(ABORT, 'plik dokumentu się nie zmienia; nowa kopia to nowy wiersz');
END;

CREATE TRIGGER sprawa_dokumenty_zbedny_tylko_z_wymaganego
BEFORE UPDATE OF stan ON sprawa_dokumenty_wpisy
WHEN NEW.stan = 'zbedny' AND OLD.stan NOT IN ('wymagany', 'zamowiony', 'zbedny')
BEGIN
  SELECT RAISE(ABORT, 'zbędny staje się tylko dokument wymagany albo zamówiony');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_dokumenty_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_dokumenty_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_dokumenty_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_dokumenty_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_dokumenty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER sprawa_dokumenty_zrodla_bez_zastepowania
BEFORE INSERT ON sprawa_dokumenty_zrodla
WHEN EXISTS (
  SELECT 1
  FROM sprawa_dokumenty_zrodla
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_dokumenty_zrodla_bez_usuwania
BEFORE DELETE ON sprawa_dokumenty_zrodla
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
];

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    Tozsamosc,
    OdpisKsiegiWieczystej,
    OdpisRejestru,
    Zaswiadczenie,
    AktNotarialny,
    Umowa,
    UmowaZBiurem,
    Pelnomocnictwo,
    Polisa,
    Oswiadczenie,
    Protokol,
    AktStanuCywilnego,
    DecyzjaKredytowa,
    SwiadectwoEnergetyczne,
    PotwierdzenieBankowe,
    Regulamin,
    Standard,
    Instrukcja,
    Uchwala,
    Pismo,
    Inny,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::Tozsamosc,
        WpisyRodzaj::OdpisKsiegiWieczystej,
        WpisyRodzaj::OdpisRejestru,
        WpisyRodzaj::Zaswiadczenie,
        WpisyRodzaj::AktNotarialny,
        WpisyRodzaj::Umowa,
        WpisyRodzaj::UmowaZBiurem,
        WpisyRodzaj::Pelnomocnictwo,
        WpisyRodzaj::Polisa,
        WpisyRodzaj::Oswiadczenie,
        WpisyRodzaj::Protokol,
        WpisyRodzaj::AktStanuCywilnego,
        WpisyRodzaj::DecyzjaKredytowa,
        WpisyRodzaj::SwiadectwoEnergetyczne,
        WpisyRodzaj::PotwierdzenieBankowe,
        WpisyRodzaj::Regulamin,
        WpisyRodzaj::Standard,
        WpisyRodzaj::Instrukcja,
        WpisyRodzaj::Uchwala,
        WpisyRodzaj::Pismo,
        WpisyRodzaj::Inny,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::Tozsamosc => "tozsamosc",
            WpisyRodzaj::OdpisKsiegiWieczystej => "odpis_ksiegi_wieczystej",
            WpisyRodzaj::OdpisRejestru => "odpis_rejestru",
            WpisyRodzaj::Zaswiadczenie => "zaswiadczenie",
            WpisyRodzaj::AktNotarialny => "akt_notarialny",
            WpisyRodzaj::Umowa => "umowa",
            WpisyRodzaj::UmowaZBiurem => "umowa_z_biurem",
            WpisyRodzaj::Pelnomocnictwo => "pelnomocnictwo",
            WpisyRodzaj::Polisa => "polisa",
            WpisyRodzaj::Oswiadczenie => "oswiadczenie",
            WpisyRodzaj::Protokol => "protokol",
            WpisyRodzaj::AktStanuCywilnego => "akt_stanu_cywilnego",
            WpisyRodzaj::DecyzjaKredytowa => "decyzja_kredytowa",
            WpisyRodzaj::SwiadectwoEnergetyczne => "swiadectwo_energetyczne",
            WpisyRodzaj::PotwierdzenieBankowe => "potwierdzenie_bankowe",
            WpisyRodzaj::Regulamin => "regulamin",
            WpisyRodzaj::Standard => "standard",
            WpisyRodzaj::Instrukcja => "instrukcja",
            WpisyRodzaj::Uchwala => "uchwala",
            WpisyRodzaj::Pismo => "pismo",
            WpisyRodzaj::Inny => "inny",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "tozsamosc" => Ok(WpisyRodzaj::Tozsamosc),
                    "odpis_ksiegi_wieczystej" => Ok(WpisyRodzaj::OdpisKsiegiWieczystej),
                    "odpis_rejestru" => Ok(WpisyRodzaj::OdpisRejestru),
                    "zaswiadczenie" => Ok(WpisyRodzaj::Zaswiadczenie),
                    "akt_notarialny" => Ok(WpisyRodzaj::AktNotarialny),
                    "umowa" => Ok(WpisyRodzaj::Umowa),
                    "umowa_z_biurem" => Ok(WpisyRodzaj::UmowaZBiurem),
                    "pelnomocnictwo" => Ok(WpisyRodzaj::Pelnomocnictwo),
                    "polisa" => Ok(WpisyRodzaj::Polisa),
                    "oswiadczenie" => Ok(WpisyRodzaj::Oswiadczenie),
                    "protokol" => Ok(WpisyRodzaj::Protokol),
                    "akt_stanu_cywilnego" => Ok(WpisyRodzaj::AktStanuCywilnego),
                    "decyzja_kredytowa" => Ok(WpisyRodzaj::DecyzjaKredytowa),
                    "swiadectwo_energetyczne" => Ok(WpisyRodzaj::SwiadectwoEnergetyczne),
                    "potwierdzenie_bankowe" => Ok(WpisyRodzaj::PotwierdzenieBankowe),
                    "regulamin" => Ok(WpisyRodzaj::Regulamin),
                    "standard" => Ok(WpisyRodzaj::Standard),
                    "instrukcja" => Ok(WpisyRodzaj::Instrukcja),
                    "uchwala" => Ok(WpisyRodzaj::Uchwala),
                    "pismo" => Ok(WpisyRodzaj::Pismo),
                    "inny" => Ok(WpisyRodzaj::Inny),
                    other => Err(format!("{other} is no value of WpisyRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.waznosc_zrodlo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyWaznoscZrodlo {
    Wystawca,
    Przepis,
    Instytucja,
}

impl WpisyWaznoscZrodlo {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyWaznoscZrodlo] = &[
        WpisyWaznoscZrodlo::Wystawca,
        WpisyWaznoscZrodlo::Przepis,
        WpisyWaznoscZrodlo::Instytucja,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyWaznoscZrodlo::Wystawca => "wystawca",
            WpisyWaznoscZrodlo::Przepis => "przepis",
            WpisyWaznoscZrodlo::Instytucja => "instytucja",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyWaznoscZrodlo {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "wystawca" => Ok(WpisyWaznoscZrodlo::Wystawca),
                    "przepis" => Ok(WpisyWaznoscZrodlo::Przepis),
                    "instytucja" => Ok(WpisyWaznoscZrodlo::Instytucja),
                    other => Err(format!("{other} is no value of WpisyWaznoscZrodlo")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyWaznoscZrodlo {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.wymaga`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyWymaga {
    Przepis,
    Notariusz,
    Bank,
    Umowa,
}

impl WpisyWymaga {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyWymaga] = &[
        WpisyWymaga::Przepis,
        WpisyWymaga::Notariusz,
        WpisyWymaga::Bank,
        WpisyWymaga::Umowa,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyWymaga::Przepis => "przepis",
            WpisyWymaga::Notariusz => "notariusz",
            WpisyWymaga::Bank => "bank",
            WpisyWymaga::Umowa => "umowa",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyWymaga {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "przepis" => Ok(WpisyWymaga::Przepis),
                    "notariusz" => Ok(WpisyWymaga::Notariusz),
                    "bank" => Ok(WpisyWymaga::Bank),
                    "umowa" => Ok(WpisyWymaga::Umowa),
                    other => Err(format!("{other} is no value of WpisyWymaga")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyWymaga {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.kanal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyKanal {
    Email,
    Sms,
    Portal,
    KanalSprzedazy,
    List,
    PokojWymiany,
    Osobiscie,
    RejestrElektroniczny,
}

impl WpisyKanal {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyKanal] = &[
        WpisyKanal::Email,
        WpisyKanal::Sms,
        WpisyKanal::Portal,
        WpisyKanal::KanalSprzedazy,
        WpisyKanal::List,
        WpisyKanal::PokojWymiany,
        WpisyKanal::Osobiscie,
        WpisyKanal::RejestrElektroniczny,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyKanal::Email => "email",
            WpisyKanal::Sms => "sms",
            WpisyKanal::Portal => "portal",
            WpisyKanal::KanalSprzedazy => "kanal_sprzedazy",
            WpisyKanal::List => "list",
            WpisyKanal::PokojWymiany => "pokoj_wymiany",
            WpisyKanal::Osobiscie => "osobiscie",
            WpisyKanal::RejestrElektroniczny => "rejestr_elektroniczny",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyKanal {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "email" => Ok(WpisyKanal::Email),
                    "sms" => Ok(WpisyKanal::Sms),
                    "portal" => Ok(WpisyKanal::Portal),
                    "kanal_sprzedazy" => Ok(WpisyKanal::KanalSprzedazy),
                    "list" => Ok(WpisyKanal::List),
                    "pokoj_wymiany" => Ok(WpisyKanal::PokojWymiany),
                    "osobiscie" => Ok(WpisyKanal::Osobiscie),
                    "rejestr_elektroniczny" => Ok(WpisyKanal::RejestrElektroniczny),
                    other => Err(format!("{other} is no value of WpisyKanal")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyKanal {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.postac`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyPostac {
    Oryginal,
    Kopia,
    Skan,
    Odpis,
    Wydruk,
}

impl WpisyPostac {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyPostac] = &[
        WpisyPostac::Oryginal,
        WpisyPostac::Kopia,
        WpisyPostac::Skan,
        WpisyPostac::Odpis,
        WpisyPostac::Wydruk,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyPostac::Oryginal => "oryginal",
            WpisyPostac::Kopia => "kopia",
            WpisyPostac::Skan => "skan",
            WpisyPostac::Odpis => "odpis",
            WpisyPostac::Wydruk => "wydruk",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyPostac {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "oryginal" => Ok(WpisyPostac::Oryginal),
                    "kopia" => Ok(WpisyPostac::Kopia),
                    "skan" => Ok(WpisyPostac::Skan),
                    "odpis" => Ok(WpisyPostac::Odpis),
                    "wydruk" => Ok(WpisyPostac::Wydruk),
                    other => Err(format!("{other} is no value of WpisyPostac")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyPostac {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyStan {
    Wymagany,
    Zamowiony,
    Otrzymany,
    Sprawdzony,
    Odrzucony,
    Zbedny,
}

impl WpisyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyStan] = &[
        WpisyStan::Wymagany,
        WpisyStan::Zamowiony,
        WpisyStan::Otrzymany,
        WpisyStan::Sprawdzony,
        WpisyStan::Odrzucony,
        WpisyStan::Zbedny,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyStan::Wymagany => "wymagany",
            WpisyStan::Zamowiony => "zamowiony",
            WpisyStan::Otrzymany => "otrzymany",
            WpisyStan::Sprawdzony => "sprawdzony",
            WpisyStan::Odrzucony => "odrzucony",
            WpisyStan::Zbedny => "zbedny",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "wymagany" => Ok(WpisyStan::Wymagany),
                    "zamowiony" => Ok(WpisyStan::Zamowiony),
                    "otrzymany" => Ok(WpisyStan::Otrzymany),
                    "sprawdzony" => Ok(WpisyStan::Sprawdzony),
                    "odrzucony" => Ok(WpisyStan::Odrzucony),
                    "zbedny" => Ok(WpisyStan::Zbedny),
                    other => Err(format!("{other} is no value of WpisyStan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyStan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `dokumenty`.
#[derive(Debug, Clone, PartialEq)]
pub struct DokumentyRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub nazwa: String,
    pub dotyczy_uczestnik_id: Option<String>,
    pub dotyczy_nieruchomosc_id: Option<String>,
    pub wystawca: Option<String>,
    pub data_wystawienia: Option<String>,
    pub waznosc_do: Option<String>,
    pub waznosc_zrodlo: Option<WpisyWaznoscZrodlo>,
    pub wymaga: Option<WpisyWymaga>,
    pub pochodzenie: Option<String>,
    pub kanal: Option<WpisyKanal>,
    pub postac: Option<WpisyPostac>,
    pub stan: WpisyStan,
    pub powod_odrzucenia: Option<String>,
    pub sciezka: Option<String>,
    pub skrot: Option<String>,
}

impl DokumentyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(DokumentyRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            nazwa: row.get("nazwa")?,
            dotyczy_uczestnik_id: row.get("dotyczy_uczestnik_id")?,
            dotyczy_nieruchomosc_id: row.get("dotyczy_nieruchomosc_id")?,
            wystawca: row.get("wystawca")?,
            data_wystawienia: row.get("data_wystawienia")?,
            waznosc_do: row.get("waznosc_do")?,
            waznosc_zrodlo: row.get("waznosc_zrodlo")?,
            wymaga: row.get("wymaga")?,
            pochodzenie: row.get("pochodzenie")?,
            kanal: row.get("kanal")?,
            postac: row.get("postac")?,
            stan: row.get("stan")?,
            powod_odrzucenia: row.get("powod_odrzucenia")?,
            sciezka: row.get("sciezka")?,
            skrot: row.get("skrot")?,
        })
    }
}

/// Lista dokumentów teczki w kolejności wpisania.
pub fn dokumenty(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<DokumentyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_dokumenty_wpisy
ORDER BY rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(DokumentyRow::from_row).collect()
}

/// A row of `dokumenty_do_zebrania`.
#[derive(Debug, Clone, PartialEq)]
pub struct DokumentyDoZebraniaRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub nazwa: String,
    pub dotyczy_uczestnik_id: Option<String>,
    pub dotyczy_nieruchomosc_id: Option<String>,
    pub wystawca: Option<String>,
    pub data_wystawienia: Option<String>,
    pub waznosc_do: Option<String>,
    pub waznosc_zrodlo: Option<WpisyWaznoscZrodlo>,
    pub wymaga: Option<WpisyWymaga>,
    pub pochodzenie: Option<String>,
    pub kanal: Option<WpisyKanal>,
    pub postac: Option<WpisyPostac>,
    pub stan: WpisyStan,
    pub powod_odrzucenia: Option<String>,
    pub sciezka: Option<String>,
    pub skrot: Option<String>,
}

impl DokumentyDoZebraniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(DokumentyDoZebraniaRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            nazwa: row.get("nazwa")?,
            dotyczy_uczestnik_id: row.get("dotyczy_uczestnik_id")?,
            dotyczy_nieruchomosc_id: row.get("dotyczy_nieruchomosc_id")?,
            wystawca: row.get("wystawca")?,
            data_wystawienia: row.get("data_wystawienia")?,
            waznosc_do: row.get("waznosc_do")?,
            waznosc_zrodlo: row.get("waznosc_zrodlo")?,
            wymaga: row.get("wymaga")?,
            pochodzenie: row.get("pochodzenie")?,
            kanal: row.get("kanal")?,
            postac: row.get("postac")?,
            stan: row.get("stan")?,
            powod_odrzucenia: row.get("powod_odrzucenia")?,
            sciezka: row.get("sciezka")?,
            skrot: row.get("skrot")?,
        })
    }
}

/// Dokumenty, które są wymagane albo zamówione i jeszcze nie przyszły.
pub fn dokumenty_do_zebrania(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<DokumentyDoZebraniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_dokumenty_wpisy
WHERE stan IN ('wymagany', 'zamowiony')
ORDER BY rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(DokumentyDoZebraniaRow::from_row).collect()
}

/// The parameters of `wpisz_wymagany_dokument`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszWymaganyDokumentParams {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub nazwa: String,
    pub dotyczy_uczestnik_id: Option<String>,
    pub dotyczy_nieruchomosc_id: Option<String>,
    pub wymaga: Option<WpisyWymaga>,
}

/// Dokument, którego wymaga przepis, notariusz, bank albo umowa; czeka na zamówienie.
pub fn wpisz_wymagany_dokument(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszWymaganyDokumentParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_dokumenty_wpisy (id, rodzaj, nazwa, dotyczy_uczestnik_id,
  dotyczy_nieruchomosc_id, wymaga, stan)
VALUES (:id, :rodzaj, :nazwa, :dotyczy_uczestnik_id, :dotyczy_nieruchomosc_id, :wymaga, 'wymagany')"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("nazwa", xmeta_room::ToSqlValue::to_sql_value(&params.nazwa)),
                    (
                        "dotyczy_uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.dotyczy_uczestnik_id,
                        ),
                    ),
                    (
                        "dotyczy_nieruchomosc_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.dotyczy_nieruchomosc_id,
                        ),
                    ),
                    ("wymaga", xmeta_room::ToSqlValue::to_sql_value(&params.wymaga)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_zamowienie_dokumentu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujZamowienieDokumentuParams {
    pub id: String,
}

/// Biuro zamówiło dokument, np. wysyłką do spółdzielni w imieniu klienta.
pub fn odnotuj_zamowienie_dokumentu(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujZamowienieDokumentuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_dokumenty_wpisy
SET stan = 'zamowiony'
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wpisz_otrzymany_dokument`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszOtrzymanyDokumentParams {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub nazwa: String,
    pub dotyczy_uczestnik_id: Option<String>,
    pub dotyczy_nieruchomosc_id: Option<String>,
    pub wystawca: Option<String>,
    pub data_wystawienia: Option<String>,
    pub waznosc_do: Option<String>,
    pub waznosc_zrodlo: Option<WpisyWaznoscZrodlo>,
    pub wymaga: Option<WpisyWymaga>,
    pub pochodzenie: Option<String>,
    pub kanal: Option<WpisyKanal>,
    pub postac: Option<WpisyPostac>,
    pub sciezka: String,
    pub skrot: String,
}

/// Dokument, który przyszedł bez wcześniejszego wymagania, od razu z plikiem.
pub fn wpisz_otrzymany_dokument(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszOtrzymanyDokumentParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_dokumenty_wpisy (id, rodzaj, nazwa, dotyczy_uczestnik_id,
  dotyczy_nieruchomosc_id, wystawca, data_wystawienia, waznosc_do, waznosc_zrodlo, wymaga,
  pochodzenie, kanal, postac, sciezka, skrot, stan)
VALUES (:id, :rodzaj, :nazwa, :dotyczy_uczestnik_id, :dotyczy_nieruchomosc_id, :wystawca,
  :data_wystawienia, :waznosc_do, :waznosc_zrodlo, :wymaga, :pochodzenie, :kanal, :postac, :sciezka,
  :skrot, 'otrzymany')"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("nazwa", xmeta_room::ToSqlValue::to_sql_value(&params.nazwa)),
                    (
                        "dotyczy_uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.dotyczy_uczestnik_id,
                        ),
                    ),
                    (
                        "dotyczy_nieruchomosc_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.dotyczy_nieruchomosc_id,
                        ),
                    ),
                    ("wystawca", xmeta_room::ToSqlValue::to_sql_value(&params.wystawca)),
                    (
                        "data_wystawienia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.data_wystawienia),
                    ),
                    (
                        "waznosc_do",
                        xmeta_room::ToSqlValue::to_sql_value(&params.waznosc_do),
                    ),
                    (
                        "waznosc_zrodlo",
                        xmeta_room::ToSqlValue::to_sql_value(&params.waznosc_zrodlo),
                    ),
                    ("wymaga", xmeta_room::ToSqlValue::to_sql_value(&params.wymaga)),
                    (
                        "pochodzenie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.pochodzenie),
                    ),
                    ("kanal", xmeta_room::ToSqlValue::to_sql_value(&params.kanal)),
                    ("postac", xmeta_room::ToSqlValue::to_sql_value(&params.postac)),
                    ("sciezka", xmeta_room::ToSqlValue::to_sql_value(&params.sciezka)),
                    ("skrot", xmeta_room::ToSqlValue::to_sql_value(&params.skrot)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_otrzymanie_dokumentu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujOtrzymanieDokumentuParams {
    pub wystawca: Option<String>,
    pub data_wystawienia: Option<String>,
    pub waznosc_do: Option<String>,
    pub waznosc_zrodlo: Option<WpisyWaznoscZrodlo>,
    pub pochodzenie: Option<String>,
    pub kanal: Option<WpisyKanal>,
    pub postac: Option<WpisyPostac>,
    pub sciezka: String,
    pub skrot: String,
    pub id: String,
}

/// Wymagany albo zamówiony dokument przyszedł: wystawca, ważność, pochodzenie i plik.
pub fn odnotuj_otrzymanie_dokumentu(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujOtrzymanieDokumentuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_dokumenty_wpisy
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
WHERE id = :id"#,
                parameters: Vec::from([
                    ("wystawca", xmeta_room::ToSqlValue::to_sql_value(&params.wystawca)),
                    (
                        "data_wystawienia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.data_wystawienia),
                    ),
                    (
                        "waznosc_do",
                        xmeta_room::ToSqlValue::to_sql_value(&params.waznosc_do),
                    ),
                    (
                        "waznosc_zrodlo",
                        xmeta_room::ToSqlValue::to_sql_value(&params.waznosc_zrodlo),
                    ),
                    (
                        "pochodzenie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.pochodzenie),
                    ),
                    ("kanal", xmeta_room::ToSqlValue::to_sql_value(&params.kanal)),
                    ("postac", xmeta_room::ToSqlValue::to_sql_value(&params.postac)),
                    ("sciezka", xmeta_room::ToSqlValue::to_sql_value(&params.sciezka)),
                    ("skrot", xmeta_room::ToSqlValue::to_sql_value(&params.skrot)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `oznacz_dokument_jako_sprawdzony`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OznaczDokumentJakoSprawdzonyParams {
    pub id: String,
}

/// Agent sprawy porównał dokument z wymaganiem: jest kompletny i czytelny.
pub fn oznacz_dokument_jako_sprawdzony(
    room: &impl xmeta_room::MetaRoom,
    params: &OznaczDokumentJakoSprawdzonyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_dokumenty_wpisy
SET stan = 'sprawdzony'
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odrzuc_dokument`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdrzucDokumentParams {
    pub powod_odrzucenia: String,
    pub id: String,
}

/// Dokument nie nadaje się, np. „nieczytelna pieczęć”; agent zamawia nowy jako nowy wiersz.
pub fn odrzuc_dokument(
    room: &impl xmeta_room::MetaRoom,
    params: &OdrzucDokumentParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_dokumenty_wpisy
SET stan = 'odrzucony',
  powod_odrzucenia = :powod_odrzucenia
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "powod_odrzucenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.powod_odrzucenia),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `uznaj_dokument_za_zbedny`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UznajDokumentZaZbednyParams {
    pub id: String,
}

/// Wymagany albo zamówiony dokument przestał być potrzebny.
pub fn uznaj_dokument_za_zbedny(
    room: &impl xmeta_room::MetaRoom,
    params: &UznajDokumentZaZbednyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_dokumenty_wpisy
SET stan = 'zbedny'
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `zrodla`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZrodlaRow {
    pub id: i64,
    pub dotyczy_tabela: String,
    pub dotyczy_id: String,
    pub pole: String,
    pub dokument_id: Option<String>,
    pub od_uczestnik_id: Option<String>,
    pub od_kogo: Option<String>,
    pub potwierdzone: bool,
    pub wpisano: String,
}

impl ZrodlaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZrodlaRow {
            id: row.get("id")?,
            dotyczy_tabela: row.get("dotyczy_tabela")?,
            dotyczy_id: row.get("dotyczy_id")?,
            pole: row.get("pole")?,
            dokument_id: row.get("dokument_id")?,
            od_uczestnik_id: row.get("od_uczestnik_id")?,
            od_kogo: row.get("od_kogo")?,
            potwierdzone: row.get("potwierdzone")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Źródła wszystkich danych teczki.
pub fn zrodla(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZrodlaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_dokumenty_zrodla
ORDER BY dotyczy_tabela, dotyczy_id, pole, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(ZrodlaRow::from_row).collect()
}

/// The parameters of `zrodla_wiersza`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZrodlaWierszaParams {
    pub dotyczy_tabela: String,
    pub dotyczy_id: String,
}

/// A row of `zrodla_wiersza`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZrodlaWierszaRow {
    pub id: i64,
    pub dotyczy_tabela: String,
    pub dotyczy_id: String,
    pub pole: String,
    pub dokument_id: Option<String>,
    pub od_uczestnik_id: Option<String>,
    pub od_kogo: Option<String>,
    pub potwierdzone: bool,
    pub wpisano: String,
}

impl ZrodlaWierszaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZrodlaWierszaRow {
            id: row.get("id")?,
            dotyczy_tabela: row.get("dotyczy_tabela")?,
            dotyczy_id: row.get("dotyczy_id")?,
            pole: row.get("pole")?,
            dokument_id: row.get("dokument_id")?,
            od_uczestnik_id: row.get("od_uczestnik_id")?,
            od_kogo: row.get("od_kogo")?,
            potwierdzone: row.get("potwierdzone")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Źródła pól jednego wiersza teczki, najnowsze pierwsze.
pub fn zrodla_wiersza(
    room: &impl xmeta_room::MetaRoom,
    params: &ZrodlaWierszaParams,
) -> Result<Vec<ZrodlaWierszaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_dokumenty_zrodla
WHERE dotyczy_tabela = :dotyczy_tabela
  AND dotyczy_id = :dotyczy_id
ORDER BY pole, id DESC"#,
        parameters: Vec::from([
            (
                "dotyczy_tabela",
                xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_tabela),
            ),
            ("dotyczy_id", xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_id)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(ZrodlaWierszaRow::from_row).collect()
}

/// The parameters of `wpisz_zrodlo`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszZrodloParams {
    pub dotyczy_tabela: String,
    pub dotyczy_id: String,
    pub pole: String,
    pub dokument_id: Option<String>,
    pub od_uczestnik_id: Option<String>,
    pub od_kogo: Option<String>,
    pub potwierdzone: bool,
    pub wpisano: String,
}

/// Skąd pochodzi wartość pola: dokument albo osoba, potwierdzona albo nie.
pub fn wpisz_zrodlo(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszZrodloParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_dokumenty_zrodla (dotyczy_tabela, dotyczy_id, pole, dokument_id, od_uczestnik_id,
  od_kogo, potwierdzone, wpisano)
VALUES (:dotyczy_tabela, :dotyczy_id, :pole, :dokument_id, :od_uczestnik_id, :od_kogo,
  :potwierdzone, :wpisano)"#,
                parameters: Vec::from([
                    (
                        "dotyczy_tabela",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_tabela),
                    ),
                    (
                        "dotyczy_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_id),
                    ),
                    ("pole", xmeta_room::ToSqlValue::to_sql_value(&params.pole)),
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                    (
                        "od_uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.od_uczestnik_id),
                    ),
                    ("od_kogo", xmeta_room::ToSqlValue::to_sql_value(&params.od_kogo)),
                    (
                        "potwierdzone",
                        xmeta_room::ToSqlValue::to_sql_value(&params.potwierdzone),
                    ),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
        ],
    )
}

/// The parameters of `potwierdz_zrodlo`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PotwierdzZrodloParams {
    pub id: i64,
}

/// Dana od osoby została potwierdzona, np. dokumentem albo drugim kanałem.
pub fn potwierdz_zrodlo(
    room: &impl xmeta_room::MetaRoom,
    params: &PotwierdzZrodloParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_dokumenty_zrodla
SET potwierdzone = 1
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
