//! The kind `stanowisko-zrodla` of the room `positions/biuro/kierownik_biura`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "stanowisko-zrodla",
    requires: &[],
    migrations: &[
        r#"-- Źródła: jak rzetelne okazały się osoby, instytucje, portale, rejestry, dokumenty i agenci, od
-- których agent coś wie. Po nieprawdzie źródła agent ocenia na nowo wszystko, co od niego wie
-- (operacja ustalenia_od_zrodla w stanowisko-ustalenia, a u kierownika działu także zwyczaje_od_zrodla).
--
-- Osobę z teczki, wykonawcę z rejestru wykonawców albo klienta z kartoteki wskazuje odnośnik do
-- jego wpisu, a nie jego dane: pamięć nie powtarza danych, które trzyma inny pokój (R1).
CREATE TABLE stanowisko_zrodla_wpisy (
  id TEXT PRIMARY KEY,
  nazwa TEXT,
  -- Rodzaj zgłoszenia (np. „pytanie o parking”) jest źródłem u Konsjerża, który ocenia zgłoszenia,
  -- a nie gości.
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('osoba', 'instytucja', 'firma', 'portal', 'kanal_sprzedazy', 'rejestr_publiczny', 'dokument', 'dane_rynkowe', 'rodzaj_zgloszenia', 'agent_zewnetrzny', 'agent_biura', 'czlowiek_biura')),
  -- Pokój i wiersz uczestnika teczki albo wpisu rejestru, tekstem.
  wpis_odnosnik TEXT,
  -- Tożsamość z XAuth/xkeys agenta albo człowieka biura.
  tozsamosc TEXT,
  ocena TEXT NOT NULL CHECK (ocena IN ('nieoceniona', 'rzetelne', 'zastrzezenia', 'nierzetelne')),
  oceniono TEXT,

  -- Wpis innego pokoju wskazuje odnośnik, więc jego nazwy pamięć nie powtarza.
  CONSTRAINT zrodlo_nazwane_albo_wskazane CHECK ((nazwa IS NULL) <> (wpis_odnosnik IS NULL)),
  CONSTRAINT ocena_z_data CHECK ((ocena = 'nieoceniona') = (oceniono IS NULL))
) STRICT;

-- Zdarzenia, na których stoi ocena źródła. To ślad, więc się ich nie zmienia; jedyną zmianą jest
-- odnotowanie, że zdarzenie weszło do raportu.
CREATE TABLE stanowisko_zrodla_zdarzenia (
  -- Klucz nadany przez bazę jest dodatni; zob. stanowisko_zrodla_zdarzenia_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  zrodlo_id TEXT NOT NULL REFERENCES stanowisko_zrodla_wpisy (id),
  zaszlo TEXT NOT NULL,
  -- Fakty (potwierdzenie, rozbieżność, nieprawda, spóźnienie, pytanie o drugą stronę), zakłócenia
  -- z profili stanowisk i rozstrzygnięcie zarzutu.
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('potwierdzenie', 'rozbieznosc', 'nieprawda', 'spoznienie', 'druga_strona', 'manipulacja', 'podszywanie', 'nacechowanie', 'dryf', 'przeciazenie', 'bledne_kolo', 'rozpoznanie', 'poroznienie', 'zarzut_potwierdzony', 'zarzut_odparty')),
  opis TEXT NOT NULL,
  -- Wiersz, który zdarzenie pokazuje, np. raport w teczce albo wpis dziennika wpływów: tabela
  -- i pokój z wierszem, tekstem.
  dotyczy_tabela TEXT,
  dotyczy_odnosnik TEXT,
  -- Kiedy agent podał zdarzenie w raporcie.
  zgloszono TEXT,

  CONSTRAINT dotyczy_tabela_razem_z_odnosnikiem CHECK ((dotyczy_tabela IS NULL) = (dotyczy_odnosnik IS NULL)),
  CONSTRAINT zgloszenie_nie_przed_zdarzeniem CHECK (zgloszono IS NULL OR zgloszono >= zaszlo)
) STRICT;

-- Obniżenie oceny sprawdza zdarzenia źródła, więc źródło nie może powstać od razu ocenione.
CREATE TRIGGER stanowisko_zrodla_powstaje_nieocenione
BEFORE INSERT ON stanowisko_zrodla_wpisy
WHEN NEW.ocena <> 'nieoceniona'
BEGIN
  SELECT RAISE(ABORT, 'źródło powstaje nieocenione; ocenę daje się po zdarzeniu');
END;

-- To, czym źródło jest, się nie zmienia; inne źródło to nowy wpis.
CREATE TRIGGER stanowisko_zrodla_tozsamosc_zrodla_niezmienna
BEFORE UPDATE ON stanowisko_zrodla_wpisy
WHEN NEW.nazwa IS NOT OLD.nazwa
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.wpis_odnosnik IS NOT OLD.wpis_odnosnik
  OR NEW.tozsamosc IS NOT OLD.tozsamosc
BEGIN
  SELECT RAISE(ABORT, 'nazwy, rodzaju ani wskazania źródła się nie zmienia; inne źródło to nowy wpis');
END;

-- Ocenę obniża tylko stwierdzony fakt. Sam zarzut (poróżnienie) jej nie obniża, dopóki agent go nie
-- potwierdzi, bo poróżnienie polega właśnie na zarzutach bez faktów. Pytanie o drugą stronę jest
-- faktem: naprawdę padło, a rozdział stron (R1) zabrania na nie odpowiadać.
CREATE TRIGGER stanowisko_zrodla_ocene_obniza_tylko_fakt
BEFORE UPDATE OF ocena ON stanowisko_zrodla_wpisy
WHEN NEW.ocena IN ('zastrzezenia', 'nierzetelne')
  AND NOT EXISTS (
    SELECT 1
    FROM stanowisko_zrodla_zdarzenia AS zdarzenie
    WHERE zdarzenie.zrodlo_id = NEW.id
      AND zdarzenie.rodzaj IN ('nieprawda', 'rozbieznosc', 'spoznienie', 'druga_strona', 'manipulacja', 'podszywanie', 'zarzut_potwierdzony')
  )
BEGIN
  SELECT RAISE(ABORT, 'ocenę źródła obniża tylko stwierdzony fakt, a nie sam zarzut');
END;

CREATE TRIGGER stanowisko_zrodla_zdarzenia_powstaje_niezgloszone
BEFORE INSERT ON stanowisko_zrodla_zdarzenia
WHEN NEW.zgloszono IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'zdarzenie źródła powstaje niezgłoszone; zgłoszenie odnotowuje się po raporcie');
END;

CREATE TRIGGER stanowisko_zrodla_zdarzenia_bez_zmian
BEFORE UPDATE ON stanowisko_zrodla_zdarzenia
WHEN OLD.zgloszono IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.zrodlo_id IS NOT OLD.zrodlo_id
  OR NEW.zaszlo IS NOT OLD.zaszlo
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.opis IS NOT OLD.opis
  OR NEW.dotyczy_tabela IS NOT OLD.dotyczy_tabela
  OR NEW.dotyczy_odnosnik IS NOT OLD.dotyczy_odnosnik
BEGIN
  SELECT RAISE(ABORT, 'zdarzenia źródła tylko się dopisuje; wolno raz odnotować zgłoszenie');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabeli z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER stanowisko_zrodla_wpisy_bez_zastepowania
BEFORE INSERT ON stanowisko_zrodla_wpisy
WHEN EXISTS (
  SELECT 1
  FROM stanowisko_zrodla_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER stanowisko_zrodla_zdarzenia_bez_zastepowania
BEFORE INSERT ON stanowisko_zrodla_zdarzenia
WHEN EXISTS (
  SELECT 1
  FROM stanowisko_zrodla_zdarzenia
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: pamięć trwa bez terminu; pomyłkę poprawia nowy wpis albo nowa ocena.
CREATE TRIGGER stanowisko_zrodla_wpisy_bez_usuwania
BEFORE DELETE ON stanowisko_zrodla_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER stanowisko_zrodla_zdarzenia_bez_usuwania
BEFORE DELETE ON stanowisko_zrodla_zdarzenia
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
    ],
};

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    Osoba,
    Instytucja,
    Firma,
    Portal,
    KanalSprzedazy,
    RejestrPubliczny,
    Dokument,
    DaneRynkowe,
    RodzajZgloszenia,
    AgentZewnetrzny,
    AgentBiura,
    CzlowiekBiura,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::Osoba,
        WpisyRodzaj::Instytucja,
        WpisyRodzaj::Firma,
        WpisyRodzaj::Portal,
        WpisyRodzaj::KanalSprzedazy,
        WpisyRodzaj::RejestrPubliczny,
        WpisyRodzaj::Dokument,
        WpisyRodzaj::DaneRynkowe,
        WpisyRodzaj::RodzajZgloszenia,
        WpisyRodzaj::AgentZewnetrzny,
        WpisyRodzaj::AgentBiura,
        WpisyRodzaj::CzlowiekBiura,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::Osoba => "osoba",
            WpisyRodzaj::Instytucja => "instytucja",
            WpisyRodzaj::Firma => "firma",
            WpisyRodzaj::Portal => "portal",
            WpisyRodzaj::KanalSprzedazy => "kanal_sprzedazy",
            WpisyRodzaj::RejestrPubliczny => "rejestr_publiczny",
            WpisyRodzaj::Dokument => "dokument",
            WpisyRodzaj::DaneRynkowe => "dane_rynkowe",
            WpisyRodzaj::RodzajZgloszenia => "rodzaj_zgloszenia",
            WpisyRodzaj::AgentZewnetrzny => "agent_zewnetrzny",
            WpisyRodzaj::AgentBiura => "agent_biura",
            WpisyRodzaj::CzlowiekBiura => "czlowiek_biura",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "osoba" => Ok(WpisyRodzaj::Osoba),
                    "instytucja" => Ok(WpisyRodzaj::Instytucja),
                    "firma" => Ok(WpisyRodzaj::Firma),
                    "portal" => Ok(WpisyRodzaj::Portal),
                    "kanal_sprzedazy" => Ok(WpisyRodzaj::KanalSprzedazy),
                    "rejestr_publiczny" => Ok(WpisyRodzaj::RejestrPubliczny),
                    "dokument" => Ok(WpisyRodzaj::Dokument),
                    "dane_rynkowe" => Ok(WpisyRodzaj::DaneRynkowe),
                    "rodzaj_zgloszenia" => Ok(WpisyRodzaj::RodzajZgloszenia),
                    "agent_zewnetrzny" => Ok(WpisyRodzaj::AgentZewnetrzny),
                    "agent_biura" => Ok(WpisyRodzaj::AgentBiura),
                    "czlowiek_biura" => Ok(WpisyRodzaj::CzlowiekBiura),
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

/// The values of column `wpisy.ocena`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyOcena {
    Nieoceniona,
    Rzetelne,
    Zastrzezenia,
    Nierzetelne,
}

impl WpisyOcena {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyOcena] = &[
        WpisyOcena::Nieoceniona,
        WpisyOcena::Rzetelne,
        WpisyOcena::Zastrzezenia,
        WpisyOcena::Nierzetelne,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyOcena::Nieoceniona => "nieoceniona",
            WpisyOcena::Rzetelne => "rzetelne",
            WpisyOcena::Zastrzezenia => "zastrzezenia",
            WpisyOcena::Nierzetelne => "nierzetelne",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyOcena {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "nieoceniona" => Ok(WpisyOcena::Nieoceniona),
                    "rzetelne" => Ok(WpisyOcena::Rzetelne),
                    "zastrzezenia" => Ok(WpisyOcena::Zastrzezenia),
                    "nierzetelne" => Ok(WpisyOcena::Nierzetelne),
                    other => Err(format!("{other} is no value of WpisyOcena")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyOcena {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `zdarzenia.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZdarzeniaRodzaj {
    Potwierdzenie,
    Rozbieznosc,
    Nieprawda,
    Spoznienie,
    DrugaStrona,
    Manipulacja,
    Podszywanie,
    Nacechowanie,
    Dryf,
    Przeciazenie,
    BledneKolo,
    Rozpoznanie,
    Poroznienie,
    ZarzutPotwierdzony,
    ZarzutOdparty,
}

impl ZdarzeniaRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[ZdarzeniaRodzaj] = &[
        ZdarzeniaRodzaj::Potwierdzenie,
        ZdarzeniaRodzaj::Rozbieznosc,
        ZdarzeniaRodzaj::Nieprawda,
        ZdarzeniaRodzaj::Spoznienie,
        ZdarzeniaRodzaj::DrugaStrona,
        ZdarzeniaRodzaj::Manipulacja,
        ZdarzeniaRodzaj::Podszywanie,
        ZdarzeniaRodzaj::Nacechowanie,
        ZdarzeniaRodzaj::Dryf,
        ZdarzeniaRodzaj::Przeciazenie,
        ZdarzeniaRodzaj::BledneKolo,
        ZdarzeniaRodzaj::Rozpoznanie,
        ZdarzeniaRodzaj::Poroznienie,
        ZdarzeniaRodzaj::ZarzutPotwierdzony,
        ZdarzeniaRodzaj::ZarzutOdparty,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            ZdarzeniaRodzaj::Potwierdzenie => "potwierdzenie",
            ZdarzeniaRodzaj::Rozbieznosc => "rozbieznosc",
            ZdarzeniaRodzaj::Nieprawda => "nieprawda",
            ZdarzeniaRodzaj::Spoznienie => "spoznienie",
            ZdarzeniaRodzaj::DrugaStrona => "druga_strona",
            ZdarzeniaRodzaj::Manipulacja => "manipulacja",
            ZdarzeniaRodzaj::Podszywanie => "podszywanie",
            ZdarzeniaRodzaj::Nacechowanie => "nacechowanie",
            ZdarzeniaRodzaj::Dryf => "dryf",
            ZdarzeniaRodzaj::Przeciazenie => "przeciazenie",
            ZdarzeniaRodzaj::BledneKolo => "bledne_kolo",
            ZdarzeniaRodzaj::Rozpoznanie => "rozpoznanie",
            ZdarzeniaRodzaj::Poroznienie => "poroznienie",
            ZdarzeniaRodzaj::ZarzutPotwierdzony => "zarzut_potwierdzony",
            ZdarzeniaRodzaj::ZarzutOdparty => "zarzut_odparty",
        }
    }
}

impl xmeta_room::FromSqlValue for ZdarzeniaRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "potwierdzenie" => Ok(ZdarzeniaRodzaj::Potwierdzenie),
                    "rozbieznosc" => Ok(ZdarzeniaRodzaj::Rozbieznosc),
                    "nieprawda" => Ok(ZdarzeniaRodzaj::Nieprawda),
                    "spoznienie" => Ok(ZdarzeniaRodzaj::Spoznienie),
                    "druga_strona" => Ok(ZdarzeniaRodzaj::DrugaStrona),
                    "manipulacja" => Ok(ZdarzeniaRodzaj::Manipulacja),
                    "podszywanie" => Ok(ZdarzeniaRodzaj::Podszywanie),
                    "nacechowanie" => Ok(ZdarzeniaRodzaj::Nacechowanie),
                    "dryf" => Ok(ZdarzeniaRodzaj::Dryf),
                    "przeciazenie" => Ok(ZdarzeniaRodzaj::Przeciazenie),
                    "bledne_kolo" => Ok(ZdarzeniaRodzaj::BledneKolo),
                    "rozpoznanie" => Ok(ZdarzeniaRodzaj::Rozpoznanie),
                    "poroznienie" => Ok(ZdarzeniaRodzaj::Poroznienie),
                    "zarzut_potwierdzony" => Ok(ZdarzeniaRodzaj::ZarzutPotwierdzony),
                    "zarzut_odparty" => Ok(ZdarzeniaRodzaj::ZarzutOdparty),
                    other => Err(format!("{other} is no value of ZdarzeniaRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for ZdarzeniaRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of field `ocena` of `ocen_zrodlo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ocena {
    Rzetelne,
    Zastrzezenia,
    Nierzetelne,
}

impl Ocena {
    /// Every value, in the order of the catalog.
    pub const ALL: &[Ocena] = &[
        Ocena::Rzetelne,
        Ocena::Zastrzezenia,
        Ocena::Nierzetelne,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            Ocena::Rzetelne => "rzetelne",
            Ocena::Zastrzezenia => "zastrzezenia",
            Ocena::Nierzetelne => "nierzetelne",
        }
    }
}

impl xmeta_room::FromSqlValue for Ocena {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "rzetelne" => Ok(Ocena::Rzetelne),
                    "zastrzezenia" => Ok(Ocena::Zastrzezenia),
                    "nierzetelne" => Ok(Ocena::Nierzetelne),
                    other => Err(format!("{other} is no value of Ocena")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for Ocena {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `zrodla`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZrodlaRow {
    pub id: String,
    pub nazwa: Option<String>,
    pub rodzaj: WpisyRodzaj,
    pub wpis_odnosnik: Option<String>,
    pub tozsamosc: Option<String>,
    pub ocena: WpisyOcena,
    pub oceniono: Option<String>,
}

impl ZrodlaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZrodlaRow {
            id: row.get("id")?,
            nazwa: row.get("nazwa")?,
            rodzaj: row.get("rodzaj")?,
            wpis_odnosnik: row.get("wpis_odnosnik")?,
            tozsamosc: row.get("tozsamosc")?,
            ocena: row.get("ocena")?,
            oceniono: row.get("oceniono")?,
        })
    }
}

/// Wszystkie źródła agenta; najpierw te, którym nie można ufać.
pub fn zrodla(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZrodlaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM stanowisko_zrodla_wpisy
ORDER BY ocena = 'nierzetelne' DESC, ocena = 'zastrzezenia' DESC, nazwa, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZrodlaRow::from_row).collect()
}

/// The parameters of `wpisz_zrodlo`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszZrodloParams {
    pub id: String,
    pub nazwa: Option<String>,
    pub rodzaj: WpisyRodzaj,
    pub wpis_odnosnik: Option<String>,
    pub tozsamosc: Option<String>,
}

/// Nowe źródło, jeszcze nieocenione; identyfikator (UUID) nadaje aplikacja, żeby ustalenie mogło go
/// od razu wskazać. Uczestnika teczki albo wpis rejestru wskazuje odnośnik zamiast nazwy.
pub fn wpisz_zrodlo(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszZrodloParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO stanowisko_zrodla_wpisy (id, nazwa, rodzaj, wpis_odnosnik, tozsamosc, ocena)
VALUES (:id, :nazwa, :rodzaj, :wpis_odnosnik, :tozsamosc, 'nieoceniona')"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("nazwa", xmeta_room::ToSqlValue::to_sql_value(&params.nazwa)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    (
                        "wpis_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wpis_odnosnik),
                    ),
                    (
                        "tozsamosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.tozsamosc),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `ocen_zrodlo`.
#[derive(Debug, Clone, PartialEq)]
pub struct OcenZrodloParams {
    pub ocena: Ocena,
    pub oceniono: String,
    pub id: String,
}

/// Nowa ocena źródła; obniżyć ją może tylko zdarzenie z faktem.
pub fn ocen_zrodlo(
    room: &impl xmeta_room::MetaRoom,
    params: &OcenZrodloParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE stanowisko_zrodla_wpisy
SET ocena = :ocena,
  oceniono = :oceniono
WHERE id = :id"#,
                parameters: Vec::from([
                    ("ocena", xmeta_room::ToSqlValue::to_sql_value(&params.ocena)),
                    ("oceniono", xmeta_room::ToSqlValue::to_sql_value(&params.oceniono)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `zdarzenia_zrodla`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZdarzeniaZrodlaParams {
    pub zrodlo_id: String,
}

/// A row of `zdarzenia_zrodla`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZdarzeniaZrodlaRow {
    pub id: i64,
    pub zrodlo_id: String,
    pub zaszlo: String,
    pub rodzaj: ZdarzeniaRodzaj,
    pub opis: String,
    pub dotyczy_tabela: Option<String>,
    pub dotyczy_odnosnik: Option<String>,
    pub zgloszono: Option<String>,
}

impl ZdarzeniaZrodlaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZdarzeniaZrodlaRow {
            id: row.get("id")?,
            zrodlo_id: row.get("zrodlo_id")?,
            zaszlo: row.get("zaszlo")?,
            rodzaj: row.get("rodzaj")?,
            opis: row.get("opis")?,
            dotyczy_tabela: row.get("dotyczy_tabela")?,
            dotyczy_odnosnik: row.get("dotyczy_odnosnik")?,
            zgloszono: row.get("zgloszono")?,
        })
    }
}

/// Zdarzenia jednego źródła, od najstarszego; na nich stoi jego ocena.
pub fn zdarzenia_zrodla(
    room: &impl xmeta_room::MetaRoom,
    params: &ZdarzeniaZrodlaParams,
) -> Result<Vec<ZdarzeniaZrodlaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM stanowisko_zrodla_zdarzenia
WHERE zrodlo_id = :zrodlo_id
ORDER BY zaszlo, id"#,
        parameters: Vec::from([
            ("zrodlo_id", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZdarzeniaZrodlaRow::from_row).collect()
}

/// A row of `zdarzenia_do_zgloszenia`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZdarzeniaDoZgloszeniaRow {
    pub id: i64,
    pub zrodlo_id: String,
    pub zaszlo: String,
    pub rodzaj: ZdarzeniaRodzaj,
    pub opis: String,
    pub dotyczy_tabela: Option<String>,
    pub dotyczy_odnosnik: Option<String>,
    pub zgloszono: Option<String>,
}

impl ZdarzeniaDoZgloszeniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZdarzeniaDoZgloszeniaRow {
            id: row.get("id")?,
            zrodlo_id: row.get("zrodlo_id")?,
            zaszlo: row.get("zaszlo")?,
            rodzaj: row.get("rodzaj")?,
            opis: row.get("opis")?,
            dotyczy_tabela: row.get("dotyczy_tabela")?,
            dotyczy_odnosnik: row.get("dotyczy_odnosnik")?,
            zgloszono: row.get("zgloszono")?,
        })
    }
}

/// Zdarzenia, których agent nie podał jeszcze w raporcie; z nich składa część raportu o źródłach.
pub fn zdarzenia_do_zgloszenia(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZdarzeniaDoZgloszeniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM stanowisko_zrodla_zdarzenia
WHERE zgloszono IS NULL
ORDER BY zaszlo, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZdarzeniaDoZgloszeniaRow::from_row).collect()
}

/// A row of `zdarzenia_zrodel`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZdarzeniaZrodelRow {
    pub zrodlo_id: String,
    pub rodzaj: ZdarzeniaRodzaj,
    pub liczba: i64,
    pub ostatnio: String,
}

impl ZdarzeniaZrodelRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZdarzeniaZrodelRow {
            zrodlo_id: row.get("zrodlo_id")?,
            rodzaj: row.get("rodzaj")?,
            liczba: row.get("liczba")?,
            ostatnio: row.get("ostatnio")?,
        })
    }
}

/// Liczba zdarzeń każdego źródła według rodzaju, np. spóźnienia i poprawki wykonawców u Koordynatora
/// operacji albo manipulacje według nadawców w meldunku Recepcji.
pub fn zdarzenia_zrodel(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZdarzeniaZrodelRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT zrodlo_id,
  rodzaj,
  count(*) AS liczba,
  max(zaszlo) AS ostatnio
FROM stanowisko_zrodla_zdarzenia
GROUP BY zrodlo_id, rodzaj
ORDER BY zrodlo_id, rodzaj"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZdarzeniaZrodelRow::from_row).collect()
}

/// The parameters of `dopisz_zdarzenie_zrodla`.
#[derive(Debug, Clone, PartialEq)]
pub struct DopiszZdarzenieZrodlaParams {
    pub zrodlo_id: String,
    pub zaszlo: String,
    pub rodzaj: ZdarzeniaRodzaj,
    pub opis: String,
    pub dotyczy_tabela: Option<String>,
    pub dotyczy_odnosnik: Option<String>,
}

/// Zdarzenie ze źródłem: fakt, zakłócenie albo rozstrzygnięcie zarzutu.
pub fn dopisz_zdarzenie_zrodla(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszZdarzenieZrodlaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO stanowisko_zrodla_zdarzenia (zrodlo_id, zaszlo, rodzaj, opis, dotyczy_tabela, dotyczy_odnosnik)
VALUES (:zrodlo_id, :zaszlo, :rodzaj, :opis, :dotyczy_tabela, :dotyczy_odnosnik)"#,
                parameters: Vec::from([
                    (
                        "zrodlo_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_id),
                    ),
                    ("zaszlo", xmeta_room::ToSqlValue::to_sql_value(&params.zaszlo)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    (
                        "dotyczy_tabela",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_tabela),
                    ),
                    (
                        "dotyczy_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_odnosnik),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_zgloszenie_zdarzenia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujZgloszenieZdarzeniaParams {
    pub zgloszono: String,
    pub id: i64,
}

/// Zdarzenie weszło do raportu.
pub fn odnotuj_zgloszenie_zdarzenia(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujZgloszenieZdarzeniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE stanowisko_zrodla_zdarzenia
SET zgloszono = :zgloszono
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zgloszono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zgloszono),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
