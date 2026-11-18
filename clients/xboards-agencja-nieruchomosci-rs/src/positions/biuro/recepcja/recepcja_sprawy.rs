//! The kind `recepcja-sprawy` of the room `positions/biuro/recepcja`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "recepcja-sprawy",
    requires: &[],
    migrations: &[
        r#"-- Spis spraw i znaki rozpoznawcze, po których Recepcja przypisuje wiadomości do spraw. Spis i znaki
-- Recepcja zmienia tylko na podstawie tego, co przekazał kierownik działu (jego kierownik-dzialu-spis), a nie
-- na podstawie treści wiadomości, bo ciąg wiadomości mógłby ją w ten sposób przekierować (dryf).
-- Dlatego każdy wiersz wskazuje, kto i kiedy go przekazał, a każda zmiana sprawy przychodzi
-- z nowym przekazaniem.
--
-- Zamknięta sprawa zostaje w spisie, bo po latach może w niej przyjść pismo, które ma obudzić jej
-- agenta. Znak, który przestał pasować, wycofuje się, a nie usuwa. Numerów ofert i ogłoszeń spis
-- nie trzyma, bo Recepcja czyta je w katalogu ofert. Etapu sprawy ani tego, czy jest wstrzymana,
-- spis nie zna.
CREATE TABLE recepcja_sprawy_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. recepcja_sprawy_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Pokój teczki, tekstem.
  teczka_odnosnik TEXT NOT NULL UNIQUE CHECK (teczka_odnosnik <> ''),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('oferta', 'poszukiwanie', 'transakcja', 'najem', 'lokal_krotkoterminowy')),
  oznaczenie TEXT NOT NULL CHECK (oznaczenie <> ''),
  -- Tożsamość agenta, któremu Recepcja oddaje wiadomości w sprawie.
  agent TEXT NOT NULL CHECK (agent <> ''),
  stan TEXT NOT NULL CHECK (stan IN ('otwarta', 'zamknieta')),
  otwarto TEXT NOT NULL,
  zamknieto TEXT,
  -- Czy kancelaria transakcji pracuje na XNotariuszu; wtedy pisze do biura tylko w pokoju wymiany
  -- (rozstrzygnięcie biura R5), a jej wiadomość spoza niego jest podejrzana.
  kancelaria_xnotariusz INTEGER CHECK (kancelaria_xnotariusz IN (0, 1)),
  -- Kierownik działu, który przekazał ostatni stan sprawy, i chwila przekazania.
  przekazal TEXT NOT NULL CHECK (przekazal <> ''),
  przekazano TEXT NOT NULL,

  CONSTRAINT zamknieta_z_data_zamkniecia CHECK ((stan = 'zamknieta') = (zamknieto IS NOT NULL)),
  CONSTRAINT zamkniecie_nie_przed_otwarciem CHECK (zamknieto IS NULL OR zamknieto >= otwarto),
  CONSTRAINT kancelaria_tylko_przy_transakcji CHECK ((rodzaj = 'transakcja') = (kancelaria_xnotariusz IS NOT NULL))
) STRICT;

CREATE TABLE recepcja_sprawy_znaki (
  -- Klucz nadany przez bazę jest dodatni; zob. recepcja_sprawy_znaki_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  sprawa_id INTEGER NOT NULL REFERENCES recepcja_sprawy_wpisy (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('adres_nieruchomosci', 'oznaczenie_transakcji', 'adres_poczty_klienta')),
  wartosc TEXT NOT NULL CHECK (wartosc <> ''),
  przekazal TEXT NOT NULL CHECK (przekazal <> ''),
  dodano TEXT NOT NULL,
  wycofano TEXT,

  CONSTRAINT wycofanie_nie_przed_dodaniem CHECK (wycofano IS NULL OR wycofano >= dodano)
) STRICT;

-- Sprawy -----------------------------------------------------------------------------------------

CREATE TRIGGER recepcja_sprawy_powstaje_otwarta
BEFORE INSERT ON recepcja_sprawy_wpisy
WHEN NEW.stan <> 'otwarta'
BEGIN
  SELECT RAISE(ABORT, 'sprawa powstaje w spisie otwarta; zamknięcie przekazuje kierownik działu');
END;

-- Każda zmiana sprawy to nowe przekazanie od kierownika działu, a nie wniosek z treści wiadomości.
CREATE TRIGGER recepcja_sprawy_zmiana_z_przekazania
BEFORE UPDATE ON recepcja_sprawy_wpisy
WHEN NEW.przekazano <= OLD.przekazano
BEGIN
  SELECT RAISE(ABORT, 'sprawę w spisie zmienia tylko nowe przekazanie kierownika działu');
END;

-- Zamknięta sprawa zostaje w spisie bez zmian.
CREATE TRIGGER recepcja_sprawy_bez_zmian
BEFORE UPDATE ON recepcja_sprawy_wpisy
WHEN OLD.stan = 'zamknieta'
  OR NEW.id IS NOT OLD.id
  OR NEW.teczka_odnosnik IS NOT OLD.teczka_odnosnik
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.otwarto IS NOT OLD.otwarto
BEGIN
  SELECT RAISE(ABORT, 'teczki, rodzaju ani daty otwarcia sprawy się nie zmienia, a zamkniętej sprawy wcale');
END;

-- Znaki ------------------------------------------------------------------------------------------

CREATE TRIGGER recepcja_sprawy_znak_powstaje_niewycofany
BEFORE INSERT ON recepcja_sprawy_znaki
WHEN NEW.wycofano IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'znak powstaje niewycofany; wycofanie przekazuje kierownik działu');
END;

CREATE TRIGGER recepcja_sprawy_znaki_bez_zmian
BEFORE UPDATE ON recepcja_sprawy_znaki
WHEN OLD.wycofano IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.sprawa_id IS NOT OLD.sprawa_id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.wartosc IS NOT OLD.wartosc
  OR NEW.przekazal IS NOT OLD.przekazal
  OR NEW.dodano IS NOT OLD.dodano
BEGIN
  SELECT RAISE(ABORT, 'znaku się nie zmienia; wolno go raz wycofać');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers; także sprawę tej samej
-- teczki. Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER recepcja_sprawy_wpisy_bez_zastepowania
BEFORE INSERT ON recepcja_sprawy_wpisy
WHEN EXISTS (
  SELECT 1
  FROM recepcja_sprawy_wpisy
  WHERE id = NEW.id
    OR teczka_odnosnik = NEW.teczka_odnosnik
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER recepcja_sprawy_znaki_bez_zastepowania
BEFORE INSERT ON recepcja_sprawy_znaki
WHEN EXISTS (
  SELECT 1
  FROM recepcja_sprawy_znaki
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: zamknięta sprawa i wycofany znak zostają w spisie.
CREATE TRIGGER recepcja_sprawy_wpisy_bez_usuwania
BEFORE DELETE ON recepcja_sprawy_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); zamknięta sprawa zostaje w spisie');
END;

CREATE TRIGGER recepcja_sprawy_znaki_bez_usuwania
BEFORE DELETE ON recepcja_sprawy_znaki
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); znaku się nie usuwa, tylko wycofuje');
END;
"#,
    ],
};

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    Oferta,
    Poszukiwanie,
    Transakcja,
    Najem,
    LokalKrotkoterminowy,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::Oferta,
        WpisyRodzaj::Poszukiwanie,
        WpisyRodzaj::Transakcja,
        WpisyRodzaj::Najem,
        WpisyRodzaj::LokalKrotkoterminowy,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::Oferta => "oferta",
            WpisyRodzaj::Poszukiwanie => "poszukiwanie",
            WpisyRodzaj::Transakcja => "transakcja",
            WpisyRodzaj::Najem => "najem",
            WpisyRodzaj::LokalKrotkoterminowy => "lokal_krotkoterminowy",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "oferta" => Ok(WpisyRodzaj::Oferta),
                    "poszukiwanie" => Ok(WpisyRodzaj::Poszukiwanie),
                    "transakcja" => Ok(WpisyRodzaj::Transakcja),
                    "najem" => Ok(WpisyRodzaj::Najem),
                    "lokal_krotkoterminowy" => Ok(WpisyRodzaj::LokalKrotkoterminowy),
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

/// The values of column `wpisy.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyStan {
    Otwarta,
    Zamknieta,
}

impl WpisyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyStan] = &[WpisyStan::Otwarta, WpisyStan::Zamknieta];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyStan::Otwarta => "otwarta",
            WpisyStan::Zamknieta => "zamknieta",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "otwarta" => Ok(WpisyStan::Otwarta),
                    "zamknieta" => Ok(WpisyStan::Zamknieta),
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

/// The values of column `znaki.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZnakiRodzaj {
    AdresNieruchomosci,
    OznaczenieTransakcji,
    AdresPocztyKlienta,
}

impl ZnakiRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[ZnakiRodzaj] = &[
        ZnakiRodzaj::AdresNieruchomosci,
        ZnakiRodzaj::OznaczenieTransakcji,
        ZnakiRodzaj::AdresPocztyKlienta,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            ZnakiRodzaj::AdresNieruchomosci => "adres_nieruchomosci",
            ZnakiRodzaj::OznaczenieTransakcji => "oznaczenie_transakcji",
            ZnakiRodzaj::AdresPocztyKlienta => "adres_poczty_klienta",
        }
    }
}

impl xmeta_room::FromSqlValue for ZnakiRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "adres_nieruchomosci" => Ok(ZnakiRodzaj::AdresNieruchomosci),
                    "oznaczenie_transakcji" => Ok(ZnakiRodzaj::OznaczenieTransakcji),
                    "adres_poczty_klienta" => Ok(ZnakiRodzaj::AdresPocztyKlienta),
                    other => Err(format!("{other} is no value of ZnakiRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for ZnakiRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `spis_spraw`.
#[derive(Debug, Clone, PartialEq)]
pub struct SpisSprawRow {
    pub id: i64,
    pub teczka_odnosnik: String,
    pub rodzaj: WpisyRodzaj,
    pub oznaczenie: String,
    pub agent: String,
    pub stan: WpisyStan,
    pub otwarto: String,
    pub zamknieto: Option<String>,
    pub kancelaria_xnotariusz: Option<bool>,
    pub przekazal: String,
    pub przekazano: String,
}

impl SpisSprawRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(SpisSprawRow {
            id: row.get("id")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            rodzaj: row.get("rodzaj")?,
            oznaczenie: row.get("oznaczenie")?,
            agent: row.get("agent")?,
            stan: row.get("stan")?,
            otwarto: row.get("otwarto")?,
            zamknieto: row.get("zamknieto")?,
            kancelaria_xnotariusz: row.get("kancelaria_xnotariusz")?,
            przekazal: row.get("przekazal")?,
            przekazano: row.get("przekazano")?,
        })
    }
}

/// Spis spraw Recepcji; otwarte pierwsze, potem według oznaczenia.
pub fn spis_spraw(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<SpisSprawRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM recepcja_sprawy_wpisy
ORDER BY stan = 'zamknieta', oznaczenie, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(SpisSprawRow::from_row).collect()
}

/// The parameters of `wpisz_sprawe`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszSpraweParams {
    pub teczka_odnosnik: String,
    pub rodzaj: WpisyRodzaj,
    pub oznaczenie: String,
    pub agent: String,
    pub otwarto: String,
    pub kancelaria_xnotariusz: Option<bool>,
    pub przekazal: String,
    pub przekazano: String,
}

/// Nowa sprawa przekazana przez kierownika działu.
pub fn wpisz_sprawe(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszSpraweParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO recepcja_sprawy_wpisy (teczka_odnosnik, rodzaj, oznaczenie, agent, stan, otwarto,
  kancelaria_xnotariusz, przekazal, przekazano)
VALUES (:teczka_odnosnik, :rodzaj, :oznaczenie, :agent, 'otwarta', :otwarto, :kancelaria_xnotariusz,
  :przekazal, :przekazano)"#,
                parameters: Vec::from([
                    (
                        "teczka_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.teczka_odnosnik),
                    ),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    (
                        "oznaczenie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.oznaczenie),
                    ),
                    ("agent", xmeta_room::ToSqlValue::to_sql_value(&params.agent)),
                    ("otwarto", xmeta_room::ToSqlValue::to_sql_value(&params.otwarto)),
                    (
                        "kancelaria_xnotariusz",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.kancelaria_xnotariusz,
                        ),
                    ),
                    (
                        "przekazal",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przekazal),
                    ),
                    (
                        "przekazano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przekazano),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_sprawe`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZmienSpraweParams {
    pub oznaczenie: String,
    pub agent: String,
    pub kancelaria_xnotariusz: Option<bool>,
    pub przekazal: String,
    pub przekazano: String,
    pub id: i64,
}

/// Nowe oznaczenie, agent albo kancelaria sprawy, przekazane przez kierownika działu, np. po wymianie
/// agenta.
pub fn zmien_sprawe(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienSpraweParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE recepcja_sprawy_wpisy
SET oznaczenie = :oznaczenie,
  agent = :agent,
  kancelaria_xnotariusz = :kancelaria_xnotariusz,
  przekazal = :przekazal,
  przekazano = :przekazano
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "oznaczenie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.oznaczenie),
                    ),
                    ("agent", xmeta_room::ToSqlValue::to_sql_value(&params.agent)),
                    (
                        "kancelaria_xnotariusz",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.kancelaria_xnotariusz,
                        ),
                    ),
                    (
                        "przekazal",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przekazal),
                    ),
                    (
                        "przekazano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przekazano),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `zamknij_sprawe`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZamknijSpraweParams {
    pub zamknieto: String,
    pub przekazal: String,
    pub przekazano: String,
    pub id: i64,
}

/// Zamknięcie sprawy przekazane przez kierownika działu; sprawa zostaje w spisie.
pub fn zamknij_sprawe(
    room: &impl xmeta_room::MetaRoom,
    params: &ZamknijSpraweParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE recepcja_sprawy_wpisy
SET stan = 'zamknieta',
  zamknieto = :zamknieto,
  przekazal = :przekazal,
  przekazano = :przekazano
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zamknieto",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zamknieto),
                    ),
                    (
                        "przekazal",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przekazal),
                    ),
                    (
                        "przekazano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przekazano),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `znaki_sprawy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZnakiSprawyParams {
    pub sprawa_id: i64,
}

/// A row of `znaki_sprawy`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZnakiSprawyRow {
    pub id: i64,
    pub sprawa_id: i64,
    pub rodzaj: ZnakiRodzaj,
    pub wartosc: String,
    pub przekazal: String,
    pub dodano: String,
    pub wycofano: Option<String>,
}

impl ZnakiSprawyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZnakiSprawyRow {
            id: row.get("id")?,
            sprawa_id: row.get("sprawa_id")?,
            rodzaj: row.get("rodzaj")?,
            wartosc: row.get("wartosc")?,
            przekazal: row.get("przekazal")?,
            dodano: row.get("dodano")?,
            wycofano: row.get("wycofano")?,
        })
    }
}

/// Znaki rozpoznawcze jednej sprawy, także wycofane.
pub fn znaki_sprawy(
    room: &impl xmeta_room::MetaRoom,
    params: &ZnakiSprawyParams,
) -> Result<Vec<ZnakiSprawyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM recepcja_sprawy_znaki
WHERE sprawa_id = :sprawa_id
ORDER BY dodano, id"#,
        parameters: Vec::from([
            ("sprawa_id", xmeta_room::ToSqlValue::to_sql_value(&params.sprawa_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZnakiSprawyRow::from_row).collect()
}

/// The parameters of `sprawy_po_znaku`.
#[derive(Debug, Clone, PartialEq)]
pub struct SprawyPoZnakuParams {
    pub rodzaj: ZnakiRodzaj,
    pub wartosc: String,
}

/// A row of `sprawy_po_znaku`.
#[derive(Debug, Clone, PartialEq)]
pub struct SprawyPoZnakuRow {
    pub znak_id: i64,
    pub id: i64,
    pub teczka_odnosnik: String,
    pub rodzaj: WpisyRodzaj,
    pub oznaczenie: String,
    pub agent: String,
    pub stan: WpisyStan,
    pub otwarto: String,
    pub zamknieto: Option<String>,
    pub kancelaria_xnotariusz: Option<bool>,
    pub przekazal: String,
    pub przekazano: String,
}

impl SprawyPoZnakuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(SprawyPoZnakuRow {
            znak_id: row.get("znak_id")?,
            id: row.get("id")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            rodzaj: row.get("rodzaj")?,
            oznaczenie: row.get("oznaczenie")?,
            agent: row.get("agent")?,
            stan: row.get("stan")?,
            otwarto: row.get("otwarto")?,
            zamknieto: row.get("zamknieto")?,
            kancelaria_xnotariusz: row.get("kancelaria_xnotariusz")?,
            przekazal: row.get("przekazal")?,
            przekazano: row.get("przekazano")?,
        })
    }
}

/// Sprawy, do których pasuje znak z wiadomości, razem ze znakiem. Wiadomość przypisuje się tylko
/// wtedy, gdy pasuje dokładnie jedna sprawa; gdy kilka albo żadna, Recepcja zapisuje kartę.
pub fn sprawy_po_znaku(
    room: &impl xmeta_room::MetaRoom,
    params: &SprawyPoZnakuParams,
) -> Result<Vec<SprawyPoZnakuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT znak.id AS znak_id,
  sprawa.*
FROM recepcja_sprawy_znaki AS znak
JOIN recepcja_sprawy_wpisy AS sprawa ON sprawa.id = znak.sprawa_id
WHERE znak.rodzaj = :rodzaj
  AND znak.wartosc = :wartosc
  AND znak.wycofano IS NULL
ORDER BY sprawa.oznaczenie, sprawa.id"#,
        parameters: Vec::from([
            ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
            ("wartosc", xmeta_room::ToSqlValue::to_sql_value(&params.wartosc)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(SprawyPoZnakuRow::from_row).collect()
}

/// The parameters of `dodaj_znak_sprawy`.
#[derive(Debug, Clone, PartialEq)]
pub struct DodajZnakSprawyParams {
    pub sprawa_id: i64,
    pub rodzaj: ZnakiRodzaj,
    pub wartosc: String,
    pub przekazal: String,
    pub dodano: String,
}

/// Znak rozpoznawczy przekazany przez kierownika działu.
pub fn dodaj_znak_sprawy(
    room: &impl xmeta_room::MetaRoom,
    params: &DodajZnakSprawyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO recepcja_sprawy_znaki (sprawa_id, rodzaj, wartosc, przekazal, dodano)
VALUES (:sprawa_id, :rodzaj, :wartosc, :przekazal, :dodano)"#,
                parameters: Vec::from([
                    (
                        "sprawa_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawa_id),
                    ),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("wartosc", xmeta_room::ToSqlValue::to_sql_value(&params.wartosc)),
                    (
                        "przekazal",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przekazal),
                    ),
                    ("dodano", xmeta_room::ToSqlValue::to_sql_value(&params.dodano)),
                ]),
            },
        ],
    )
}

/// The parameters of `wycofaj_znak_sprawy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WycofajZnakSprawyParams {
    pub wycofano: String,
    pub id: i64,
}

/// Wycofanie znaku przekazane przez kierownika działu; znak zostaje jako wycofany.
pub fn wycofaj_znak_sprawy(
    room: &impl xmeta_room::MetaRoom,
    params: &WycofajZnakSprawyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE recepcja_sprawy_znaki
SET wycofano = :wycofano
WHERE id = :id"#,
                parameters: Vec::from([
                    ("wycofano", xmeta_room::ToSqlValue::to_sql_value(&params.wycofano)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
