//! The kind `sprawa-notatki` of the room `cases/oferta`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "sprawa-notatki";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Notatki przekazania: jedyny sposób, w jaki agenci rozmawiają ze sobą (zasada „Teczka jest
-- wspólnym stanem sprawy”). Notatka jest prośbą od stanowiska do stanowiska; trafia też do
-- następcy agenta. Gdy w teczce pracuje dwóch Kontrolerów (R1), notatka do Kontrolera wskazuje
-- klienta, którego Kontrolera dotyczy. Pisze ją każdy członek teczki, który ma prawo pisać.
--
-- Treść jest zapisem tego, kto komu co przekazał, więc się nie zmienia; sprostowanie to nowa
-- notatka w odpowiedzi. Stan zmienia się raz: notatka zostaje wykonana albo odrzucona z powodem.
CREATE TABLE sprawa_notatki_wpisy (
  id TEXT PRIMARY KEY,
  od TEXT NOT NULL,
  -- Stanowiska: nazwy ich folderów pod positions/.
  od_stanowiska TEXT NOT NULL CHECK (od_stanowiska IN ('analityk_kredytowy', 'analityk_stanu_prawnego', 'kierownik_biura', 'kierownik_kontroli', 'kierownik_najmu', 'kierownik_posrednictwa', 'kierownik_transakcji', 'konsjerz', 'kontroler', 'koordynator_operacji', 'koordynator_transakcji', 'opiekun_najmu', 'opiekun_oferty', 'opiekun_poszukujacego', 'recepcja', 'zarzadca_obiektu')),
  do_stanowiska TEXT NOT NULL CHECK (do_stanowiska IN ('analityk_kredytowy', 'analityk_stanu_prawnego', 'kierownik_biura', 'kierownik_kontroli', 'kierownik_najmu', 'kierownik_posrednictwa', 'kierownik_transakcji', 'konsjerz', 'kontroler', 'koordynator_operacji', 'koordynator_transakcji', 'opiekun_najmu', 'opiekun_oferty', 'opiekun_poszukujacego', 'recepcja', 'zarzadca_obiektu')),
  -- Klient, którego Kontrolera dotyczy notatka, gdy w teczce jest dwóch.
  do_klienta_uczestnik_id TEXT REFERENCES sprawa_uczestnicy_wpisy (id),
  tresc TEXT NOT NULL,
  utworzono TEXT NOT NULL,
  do_kiedy TEXT,
  w_odpowiedzi_na TEXT REFERENCES sprawa_notatki_wpisy (id),
  stan TEXT NOT NULL CHECK (stan IN ('otwarta', 'wykonana', 'odrzucona')),
  powod TEXT,

  CONSTRAINT klient_tylko_przy_kontrolerze CHECK (do_klienta_uczestnik_id IS NULL OR do_stanowiska = 'kontroler'),
  CONSTRAINT odrzucona_ma_powod CHECK ((stan = 'odrzucona') = (powod IS NOT NULL))
) STRICT;

-- Wiersze teczki (rdzenia albo typu sprawy), których notatka dotyczy; bez klucza obcego.
CREATE TABLE sprawa_notatki_wiersze (
  -- Klucz nadany przez bazę jest dodatni; zob. sprawa_notatki_wiersze_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  notatka_id TEXT NOT NULL REFERENCES sprawa_notatki_wpisy (id),
  dotyczy_tabela TEXT NOT NULL,
  dotyczy_id TEXT NOT NULL,

  UNIQUE (notatka_id, dotyczy_tabela, dotyczy_id)
) STRICT;

CREATE TRIGGER sprawa_notatki_powstaje_otwarta
BEFORE INSERT ON sprawa_notatki_wpisy
WHEN NEW.stan <> 'otwarta'
BEGIN
  SELECT RAISE(ABORT, 'notatka powstaje jako otwarta');
END;

CREATE TRIGGER sprawa_notatki_tresc_niezmienna
BEFORE UPDATE ON sprawa_notatki_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.od IS NOT OLD.od
  OR NEW.od_stanowiska IS NOT OLD.od_stanowiska
  OR NEW.do_stanowiska IS NOT OLD.do_stanowiska
  OR NEW.do_klienta_uczestnik_id IS NOT OLD.do_klienta_uczestnik_id
  OR NEW.tresc IS NOT OLD.tresc
  OR NEW.utworzono IS NOT OLD.utworzono
  OR NEW.do_kiedy IS NOT OLD.do_kiedy
  OR NEW.w_odpowiedzi_na IS NOT OLD.w_odpowiedzi_na
BEGIN
  SELECT RAISE(ABORT, 'treść notatki się nie zmienia; sprostowanie to nowa notatka w odpowiedzi');
END;

CREATE TRIGGER sprawa_notatki_stan_zmienia_sie_raz
BEFORE UPDATE ON sprawa_notatki_wpisy
WHEN OLD.stan <> 'otwarta'
BEGIN
  SELECT RAISE(ABORT, 'stan notatki zmienia się raz: z otwartej na wykonaną albo odrzuconą');
END;

CREATE TRIGGER sprawa_notatki_wiersze_bez_zmian
BEFORE UPDATE ON sprawa_notatki_wiersze
BEGIN
  SELECT RAISE(ABORT, 'wiersza notatki się nie zmienia; inny wiersz to nowe wskazanie');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_notatki_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_notatki_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_notatki_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_notatki_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_notatki_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER sprawa_notatki_wiersze_bez_zastepowania
BEFORE INSERT ON sprawa_notatki_wiersze
WHEN EXISTS (
    SELECT 1
    FROM sprawa_notatki_wiersze
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM sprawa_notatki_wiersze
    WHERE notatka_id = NEW.notatka_id
      AND dotyczy_tabela = NEW.dotyczy_tabela
      AND dotyczy_id = NEW.dotyczy_id
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_notatki_wiersze_bez_usuwania
BEFORE DELETE ON sprawa_notatki_wiersze
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
];

/// The values of column `wpisy.od_stanowiska`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyOdStanowiska {
    AnalitykKredytowy,
    AnalitykStanuPrawnego,
    KierownikBiura,
    KierownikKontroli,
    KierownikNajmu,
    KierownikPosrednictwa,
    KierownikTransakcji,
    Konsjerz,
    Kontroler,
    KoordynatorOperacji,
    KoordynatorTransakcji,
    OpiekunNajmu,
    OpiekunOferty,
    OpiekunPoszukujacego,
    Recepcja,
    ZarzadcaObiektu,
}

impl WpisyOdStanowiska {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyOdStanowiska] = &[
        WpisyOdStanowiska::AnalitykKredytowy,
        WpisyOdStanowiska::AnalitykStanuPrawnego,
        WpisyOdStanowiska::KierownikBiura,
        WpisyOdStanowiska::KierownikKontroli,
        WpisyOdStanowiska::KierownikNajmu,
        WpisyOdStanowiska::KierownikPosrednictwa,
        WpisyOdStanowiska::KierownikTransakcji,
        WpisyOdStanowiska::Konsjerz,
        WpisyOdStanowiska::Kontroler,
        WpisyOdStanowiska::KoordynatorOperacji,
        WpisyOdStanowiska::KoordynatorTransakcji,
        WpisyOdStanowiska::OpiekunNajmu,
        WpisyOdStanowiska::OpiekunOferty,
        WpisyOdStanowiska::OpiekunPoszukujacego,
        WpisyOdStanowiska::Recepcja,
        WpisyOdStanowiska::ZarzadcaObiektu,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyOdStanowiska::AnalitykKredytowy => "analityk_kredytowy",
            WpisyOdStanowiska::AnalitykStanuPrawnego => "analityk_stanu_prawnego",
            WpisyOdStanowiska::KierownikBiura => "kierownik_biura",
            WpisyOdStanowiska::KierownikKontroli => "kierownik_kontroli",
            WpisyOdStanowiska::KierownikNajmu => "kierownik_najmu",
            WpisyOdStanowiska::KierownikPosrednictwa => "kierownik_posrednictwa",
            WpisyOdStanowiska::KierownikTransakcji => "kierownik_transakcji",
            WpisyOdStanowiska::Konsjerz => "konsjerz",
            WpisyOdStanowiska::Kontroler => "kontroler",
            WpisyOdStanowiska::KoordynatorOperacji => "koordynator_operacji",
            WpisyOdStanowiska::KoordynatorTransakcji => "koordynator_transakcji",
            WpisyOdStanowiska::OpiekunNajmu => "opiekun_najmu",
            WpisyOdStanowiska::OpiekunOferty => "opiekun_oferty",
            WpisyOdStanowiska::OpiekunPoszukujacego => "opiekun_poszukujacego",
            WpisyOdStanowiska::Recepcja => "recepcja",
            WpisyOdStanowiska::ZarzadcaObiektu => "zarzadca_obiektu",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyOdStanowiska {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "analityk_kredytowy" => Ok(WpisyOdStanowiska::AnalitykKredytowy),
                    "analityk_stanu_prawnego" => {
                        Ok(WpisyOdStanowiska::AnalitykStanuPrawnego)
                    }
                    "kierownik_biura" => Ok(WpisyOdStanowiska::KierownikBiura),
                    "kierownik_kontroli" => Ok(WpisyOdStanowiska::KierownikKontroli),
                    "kierownik_najmu" => Ok(WpisyOdStanowiska::KierownikNajmu),
                    "kierownik_posrednictwa" => {
                        Ok(WpisyOdStanowiska::KierownikPosrednictwa)
                    }
                    "kierownik_transakcji" => Ok(WpisyOdStanowiska::KierownikTransakcji),
                    "konsjerz" => Ok(WpisyOdStanowiska::Konsjerz),
                    "kontroler" => Ok(WpisyOdStanowiska::Kontroler),
                    "koordynator_operacji" => Ok(WpisyOdStanowiska::KoordynatorOperacji),
                    "koordynator_transakcji" => {
                        Ok(WpisyOdStanowiska::KoordynatorTransakcji)
                    }
                    "opiekun_najmu" => Ok(WpisyOdStanowiska::OpiekunNajmu),
                    "opiekun_oferty" => Ok(WpisyOdStanowiska::OpiekunOferty),
                    "opiekun_poszukujacego" => {
                        Ok(WpisyOdStanowiska::OpiekunPoszukujacego)
                    }
                    "recepcja" => Ok(WpisyOdStanowiska::Recepcja),
                    "zarzadca_obiektu" => Ok(WpisyOdStanowiska::ZarzadcaObiektu),
                    other => Err(format!("{other} is no value of WpisyOdStanowiska")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyOdStanowiska {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyStan {
    Otwarta,
    Wykonana,
    Odrzucona,
}

impl WpisyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyStan] = &[
        WpisyStan::Otwarta,
        WpisyStan::Wykonana,
        WpisyStan::Odrzucona,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyStan::Otwarta => "otwarta",
            WpisyStan::Wykonana => "wykonana",
            WpisyStan::Odrzucona => "odrzucona",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "otwarta" => Ok(WpisyStan::Otwarta),
                    "wykonana" => Ok(WpisyStan::Wykonana),
                    "odrzucona" => Ok(WpisyStan::Odrzucona),
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

/// A row of `wiersze_notatek`.
#[derive(Debug, Clone, PartialEq)]
pub struct WierszeNotatekRow {
    pub id: i64,
    pub notatka_id: String,
    pub dotyczy_tabela: String,
    pub dotyczy_id: String,
}

impl WierszeNotatekRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WierszeNotatekRow {
            id: row.get("id")?,
            notatka_id: row.get("notatka_id")?,
            dotyczy_tabela: row.get("dotyczy_tabela")?,
            dotyczy_id: row.get("dotyczy_id")?,
        })
    }
}

/// Wiersze teczki, których dotyczą notatki.
pub fn wiersze_notatek(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WierszeNotatekRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_notatki_wiersze
ORDER BY notatka_id, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(WierszeNotatekRow::from_row).collect()
}

/// The parameters of `wskaz_wiersz_notatki`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WskazWierszNotatkiParams {
    pub notatka_id: String,
    pub dotyczy_tabela: String,
    pub dotyczy_id: String,
}

/// Wiersz teczki, którego dotyczy notatka.
pub fn wskaz_wiersz_notatki(
    room: &impl xmeta_room::MetaRoom,
    params: &WskazWierszNotatkiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_notatki_wiersze (notatka_id, dotyczy_tabela, dotyczy_id)
VALUES (:notatka_id, :dotyczy_tabela, :dotyczy_id)"#,
                parameters: Vec::from([
                    (
                        "notatka_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.notatka_id),
                    ),
                    (
                        "dotyczy_tabela",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_tabela),
                    ),
                    (
                        "dotyczy_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_id),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `notatki`.
#[derive(Debug, Clone, PartialEq)]
pub struct NotatkiRow {
    pub id: String,
    pub od: String,
    pub od_stanowiska: WpisyOdStanowiska,
    pub do_stanowiska: WpisyOdStanowiska,
    pub do_klienta_uczestnik_id: Option<String>,
    pub tresc: String,
    pub utworzono: String,
    pub do_kiedy: Option<String>,
    pub w_odpowiedzi_na: Option<String>,
    pub stan: WpisyStan,
    pub powod: Option<String>,
}

impl NotatkiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(NotatkiRow {
            id: row.get("id")?,
            od: row.get("od")?,
            od_stanowiska: row.get("od_stanowiska")?,
            do_stanowiska: row.get("do_stanowiska")?,
            do_klienta_uczestnik_id: row.get("do_klienta_uczestnik_id")?,
            tresc: row.get("tresc")?,
            utworzono: row.get("utworzono")?,
            do_kiedy: row.get("do_kiedy")?,
            w_odpowiedzi_na: row.get("w_odpowiedzi_na")?,
            stan: row.get("stan")?,
            powod: row.get("powod")?,
        })
    }
}

/// Notatki przekazania w teczce, najnowsze pierwsze.
pub fn notatki(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<NotatkiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_notatki_wpisy
ORDER BY utworzono DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(NotatkiRow::from_row).collect()
}

/// The parameters of `notatki_otwarte_do_stanowiska`.
#[derive(Debug, Clone, PartialEq)]
pub struct NotatkiOtwarteDoStanowiskaParams {
    pub do_stanowiska: WpisyOdStanowiska,
}

/// A row of `notatki_otwarte_do_stanowiska`.
#[derive(Debug, Clone, PartialEq)]
pub struct NotatkiOtwarteDoStanowiskaRow {
    pub id: String,
    pub od: String,
    pub od_stanowiska: WpisyOdStanowiska,
    pub do_stanowiska: WpisyOdStanowiska,
    pub do_klienta_uczestnik_id: Option<String>,
    pub tresc: String,
    pub utworzono: String,
    pub do_kiedy: Option<String>,
    pub w_odpowiedzi_na: Option<String>,
    pub stan: WpisyStan,
    pub powod: Option<String>,
}

impl NotatkiOtwarteDoStanowiskaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(NotatkiOtwarteDoStanowiskaRow {
            id: row.get("id")?,
            od: row.get("od")?,
            od_stanowiska: row.get("od_stanowiska")?,
            do_stanowiska: row.get("do_stanowiska")?,
            do_klienta_uczestnik_id: row.get("do_klienta_uczestnik_id")?,
            tresc: row.get("tresc")?,
            utworzono: row.get("utworzono")?,
            do_kiedy: row.get("do_kiedy")?,
            w_odpowiedzi_na: row.get("w_odpowiedzi_na")?,
            stan: row.get("stan")?,
            powod: row.get("powod")?,
        })
    }
}

/// Otwarte notatki dla jednego stanowiska, najstarsze pierwsze.
pub fn notatki_otwarte_do_stanowiska(
    room: &impl xmeta_room::MetaRoom,
    params: &NotatkiOtwarteDoStanowiskaParams,
) -> Result<Vec<NotatkiOtwarteDoStanowiskaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_notatki_wpisy
WHERE do_stanowiska = :do_stanowiska
  AND stan = 'otwarta'
ORDER BY utworzono, id"#,
        parameters: Vec::from([
            (
                "do_stanowiska",
                xmeta_room::ToSqlValue::to_sql_value(&params.do_stanowiska),
            ),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(NotatkiOtwarteDoStanowiskaRow::from_row).collect()
}

/// The parameters of `dopisz_notatke`.
#[derive(Debug, Clone, PartialEq)]
pub struct DopiszNotatkeParams {
    pub id: String,
    pub od: String,
    pub od_stanowiska: WpisyOdStanowiska,
    pub do_stanowiska: WpisyOdStanowiska,
    pub do_klienta_uczestnik_id: Option<String>,
    pub tresc: String,
    pub utworzono: String,
    pub do_kiedy: Option<String>,
    pub w_odpowiedzi_na: Option<String>,
}

/// Notatka przekazania: co się stało, czego potrzeba i do kiedy; identyfikator (UUID) nadaje
/// aplikacja, żeby od razu wskazać wiersze, których dotyczy.
pub fn dopisz_notatke(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszNotatkeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_notatki_wpisy (id, od, od_stanowiska, do_stanowiska, do_klienta_uczestnik_id,
  tresc, utworzono, do_kiedy, w_odpowiedzi_na, stan)
VALUES (:id, :od, :od_stanowiska, :do_stanowiska, :do_klienta_uczestnik_id, :tresc, :utworzono,
  :do_kiedy, :w_odpowiedzi_na, 'otwarta')"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    (
                        "od_stanowiska",
                        xmeta_room::ToSqlValue::to_sql_value(&params.od_stanowiska),
                    ),
                    (
                        "do_stanowiska",
                        xmeta_room::ToSqlValue::to_sql_value(&params.do_stanowiska),
                    ),
                    (
                        "do_klienta_uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.do_klienta_uczestnik_id,
                        ),
                    ),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                    (
                        "utworzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.utworzono),
                    ),
                    ("do_kiedy", xmeta_room::ToSqlValue::to_sql_value(&params.do_kiedy)),
                    (
                        "w_odpowiedzi_na",
                        xmeta_room::ToSqlValue::to_sql_value(&params.w_odpowiedzi_na),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `oznacz_notatke_jako_wykonana`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OznaczNotatkeJakoWykonanaParams {
    pub id: String,
}

/// Adresat wykonał prośbę z notatki.
pub fn oznacz_notatke_jako_wykonana(
    room: &impl xmeta_room::MetaRoom,
    params: &OznaczNotatkeJakoWykonanaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_notatki_wpisy
SET stan = 'wykonana'
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odrzuc_notatke`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdrzucNotatkeParams {
    pub powod: String,
    pub id: String,
}

/// Adresat nie wykona prośby, bo wykracza poza jego normy; powód wraca do autora.
pub fn odrzuc_notatke(
    room: &impl xmeta_room::MetaRoom,
    params: &OdrzucNotatkeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_notatki_wpisy
SET stan = 'odrzucona',
  powod = :powod
WHERE id = :id"#,
                parameters: Vec::from([
                    ("powod", xmeta_room::ToSqlValue::to_sql_value(&params.powod)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
