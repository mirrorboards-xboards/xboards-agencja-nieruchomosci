//! The kind `kierownik-dzialu-spis` of the room `positions/transakcje/kierownik_transakcji`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "kierownik-dzialu-spis";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Spis spraw działu i ich znaki rozpoznawcze, które kierownik przekazuje Recepcji. Dzięki nim
-- Recepcja przypisuje wiadomość do sprawy bez czytania teczek. Recepcja zmienia swój spis tylko na
-- podstawie tego, co przekazał kierownik, więc kierownik pamięta, co i kiedy przekazał.
--
-- Wiersz mówi, w jakim stanie jest sprawa w spisie, a kolumny „przekazano” mówią, czy Recepcja ten
-- stan już dostała. Zmiana oznaczenia, agenta albo kancelarii czeka na ponowne przekazanie.
-- Numerów ofert i ogłoszeń spis nie trzyma, bo Recepcja czyta je w katalogu ofert.
CREATE TABLE kierownik_dzialu_spis_sprawy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_dzialu_spis_sprawy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Pokój teczki, tekstem.
  teczka_odnosnik TEXT NOT NULL UNIQUE CHECK (teczka_odnosnik <> ''),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('oferta', 'poszukiwanie', 'transakcja', 'najem', 'lokal_krotkoterminowy')),
  oznaczenie TEXT NOT NULL CHECK (oznaczenie <> ''),
  -- Tożsamość agenta, któremu Recepcja oddaje wiadomości w sprawie.
  agent TEXT NOT NULL,
  -- Dzień przyjęcia sprawy z metryki teczki; Recepcja pokazuje go w swoim spisie.
  otwarto TEXT NOT NULL CHECK (otwarto <> ''),
  -- Czy kancelaria transakcji pracuje na XNotariuszu; wtedy pisze do biura tylko w pokoju wymiany
  -- (rozstrzygnięcie biura R5).
  kancelaria_xnotariusz INTEGER CHECK (kancelaria_xnotariusz IN (0, 1)),
  przekazano TEXT,
  zamknieto TEXT,
  zamkniecie_przekazano TEXT,

  CONSTRAINT kancelaria_tylko_przy_transakcji CHECK ((rodzaj = 'transakcja') = (kancelaria_xnotariusz IS NOT NULL)),
  CONSTRAINT zamkniecie_przekazane_po_zamknieciu CHECK (
    zamkniecie_przekazano IS NULL
    OR (zamknieto IS NOT NULL AND zamkniecie_przekazano >= zamknieto)
  )
) STRICT;

CREATE TABLE kierownik_dzialu_spis_znaki (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_dzialu_spis_znaki_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  sprawa_id INTEGER NOT NULL REFERENCES kierownik_dzialu_spis_sprawy (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('adres_nieruchomosci', 'oznaczenie_transakcji', 'adres_poczty_klienta')),
  wartosc TEXT NOT NULL CHECK (wartosc <> ''),
  dodano TEXT NOT NULL,
  przekazano TEXT,
  wycofano TEXT,
  wycofanie_przekazano TEXT,

  CONSTRAINT przekazanie_nie_przed_dodaniem CHECK (przekazano IS NULL OR przekazano >= dodano),
  CONSTRAINT wycofanie_nie_przed_dodaniem CHECK (wycofano IS NULL OR wycofano >= dodano),
  -- Wycofanie przekazuje się tylko wtedy, gdy Recepcja dostała znak.
  CONSTRAINT wycofanie_przekazane_po_wycofaniu_przekazanego_znaku CHECK (
    wycofanie_przekazano IS NULL
    OR (wycofano IS NOT NULL AND przekazano IS NOT NULL AND wycofanie_przekazano >= wycofano)
  )
) STRICT;

-- Sprawy -----------------------------------------------------------------------------------------

CREATE TRIGGER kierownik_dzialu_spis_sprawa_powstaje_otwarta
BEFORE INSERT ON kierownik_dzialu_spis_sprawy
WHEN NEW.przekazano IS NOT NULL
  OR NEW.zamknieto IS NOT NULL
  OR NEW.zamkniecie_przekazano IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'sprawa powstaje w spisie otwarta i jeszcze nieprzekazana');
END;

-- Zamknięta sprawa zostaje w spisie, bo po latach może w niej przyjść pismo; zmienia się w niej już
-- tylko przekazanie zamknięcia, raz.
CREATE TRIGGER kierownik_dzialu_spis_sprawy_bez_zmian
BEFORE UPDATE ON kierownik_dzialu_spis_sprawy
WHEN NEW.id IS NOT OLD.id
  OR NEW.teczka_odnosnik IS NOT OLD.teczka_odnosnik
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.otwarto IS NOT OLD.otwarto
  OR (OLD.zamknieto IS NOT NULL AND (
    NEW.oznaczenie IS NOT OLD.oznaczenie
    OR NEW.agent IS NOT OLD.agent
    OR NEW.kancelaria_xnotariusz IS NOT OLD.kancelaria_xnotariusz
    OR NEW.przekazano IS NOT OLD.przekazano
    OR NEW.zamknieto IS NOT OLD.zamknieto
    OR OLD.zamkniecie_przekazano IS NOT NULL
  ))
BEGIN
  SELECT RAISE(ABORT, 'teczki ani rodzaju sprawy w spisie się nie zmienia, a zamkniętej sprawy tylko raz przekazuje się zamknięcie');
END;

-- Znaki ------------------------------------------------------------------------------------------

CREATE TRIGGER kierownik_dzialu_spis_znak_powstaje_nieprzekazany
BEFORE INSERT ON kierownik_dzialu_spis_znaki
WHEN NEW.przekazano IS NOT NULL
  OR NEW.wycofano IS NOT NULL
  OR NEW.wycofanie_przekazano IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'znak powstaje niewycofany i jeszcze nieprzekazany');
END;

-- Znaku się nie zmienia; wolno raz odnotować przekazanie, wycofanie i przekazanie wycofania.
CREATE TRIGGER kierownik_dzialu_spis_znaki_bez_zmian
BEFORE UPDATE ON kierownik_dzialu_spis_znaki
WHEN NEW.id IS NOT OLD.id
  OR NEW.sprawa_id IS NOT OLD.sprawa_id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.wartosc IS NOT OLD.wartosc
  OR NEW.dodano IS NOT OLD.dodano
  OR (OLD.przekazano IS NOT NULL AND NEW.przekazano IS NOT OLD.przekazano)
  OR (OLD.wycofano IS NOT NULL AND NEW.wycofano IS NOT OLD.wycofano)
  OR (OLD.wycofanie_przekazano IS NOT NULL AND NEW.wycofanie_przekazano IS NOT OLD.wycofanie_przekazano)
BEGIN
  SELECT RAISE(ABORT, 'znaku się nie zmienia; wolno raz odnotować przekazanie, wycofanie i przekazanie wycofania');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers; także sprawę tej samej
-- teczki. Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER kierownik_dzialu_spis_sprawy_bez_zastepowania
BEFORE INSERT ON kierownik_dzialu_spis_sprawy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_dzialu_spis_sprawy
  WHERE id = NEW.id
    OR teczka_odnosnik = NEW.teczka_odnosnik
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER kierownik_dzialu_spis_znaki_bez_zastepowania
BEFORE INSERT ON kierownik_dzialu_spis_znaki
WHEN EXISTS (
  SELECT 1
  FROM kierownik_dzialu_spis_znaki
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: zamknięta sprawa i wycofany znak zostają w spisie.
CREATE TRIGGER kierownik_dzialu_spis_sprawy_bez_usuwania
BEFORE DELETE ON kierownik_dzialu_spis_sprawy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER kierownik_dzialu_spis_znaki_bez_usuwania
BEFORE DELETE ON kierownik_dzialu_spis_znaki
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); znaku się nie usuwa, tylko wycofuje');
END;
"#,
];

/// The values of column `sprawy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SprawyRodzaj {
    Oferta,
    Poszukiwanie,
    Transakcja,
    Najem,
    LokalKrotkoterminowy,
}

impl SprawyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[SprawyRodzaj] = &[
        SprawyRodzaj::Oferta,
        SprawyRodzaj::Poszukiwanie,
        SprawyRodzaj::Transakcja,
        SprawyRodzaj::Najem,
        SprawyRodzaj::LokalKrotkoterminowy,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            SprawyRodzaj::Oferta => "oferta",
            SprawyRodzaj::Poszukiwanie => "poszukiwanie",
            SprawyRodzaj::Transakcja => "transakcja",
            SprawyRodzaj::Najem => "najem",
            SprawyRodzaj::LokalKrotkoterminowy => "lokal_krotkoterminowy",
        }
    }
}

impl xmeta_room::FromSqlValue for SprawyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "oferta" => Ok(SprawyRodzaj::Oferta),
                    "poszukiwanie" => Ok(SprawyRodzaj::Poszukiwanie),
                    "transakcja" => Ok(SprawyRodzaj::Transakcja),
                    "najem" => Ok(SprawyRodzaj::Najem),
                    "lokal_krotkoterminowy" => Ok(SprawyRodzaj::LokalKrotkoterminowy),
                    other => Err(format!("{other} is no value of SprawyRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for SprawyRodzaj {
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

/// A row of `sprawy_w_spisie`.
#[derive(Debug, Clone, PartialEq)]
pub struct SprawyWSpisieRow {
    pub id: i64,
    pub teczka_odnosnik: String,
    pub rodzaj: SprawyRodzaj,
    pub oznaczenie: String,
    pub agent: String,
    pub otwarto: String,
    pub kancelaria_xnotariusz: Option<bool>,
    pub przekazano: Option<String>,
    pub zamknieto: Option<String>,
    pub zamkniecie_przekazano: Option<String>,
}

impl SprawyWSpisieRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(SprawyWSpisieRow {
            id: row.get("id")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            rodzaj: row.get("rodzaj")?,
            oznaczenie: row.get("oznaczenie")?,
            agent: row.get("agent")?,
            otwarto: row.get("otwarto")?,
            kancelaria_xnotariusz: row.get("kancelaria_xnotariusz")?,
            przekazano: row.get("przekazano")?,
            zamknieto: row.get("zamknieto")?,
            zamkniecie_przekazano: row.get("zamkniecie_przekazano")?,
        })
    }
}

/// Wszystkie sprawy w spisie; najpierw otwarte, potem zamknięte.
pub fn sprawy_w_spisie(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<SprawyWSpisieRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_dzialu_spis_sprawy
ORDER BY zamknieto IS NOT NULL, oznaczenie, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(SprawyWSpisieRow::from_row).collect()
}

/// A row of `do_przekazania_recepcji`.
#[derive(Debug, Clone, PartialEq)]
pub struct DoPrzekazaniaRecepcjiRow {
    pub id: i64,
    pub teczka_odnosnik: String,
    pub rodzaj: SprawyRodzaj,
    pub oznaczenie: String,
    pub agent: String,
    pub otwarto: String,
    pub kancelaria_xnotariusz: Option<bool>,
    pub przekazano: Option<String>,
    pub zamknieto: Option<String>,
    pub zamkniecie_przekazano: Option<String>,
}

impl DoPrzekazaniaRecepcjiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(DoPrzekazaniaRecepcjiRow {
            id: row.get("id")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            rodzaj: row.get("rodzaj")?,
            oznaczenie: row.get("oznaczenie")?,
            agent: row.get("agent")?,
            otwarto: row.get("otwarto")?,
            kancelaria_xnotariusz: row.get("kancelaria_xnotariusz")?,
            przekazano: row.get("przekazano")?,
            zamknieto: row.get("zamknieto")?,
            zamkniecie_przekazano: row.get("zamkniecie_przekazano")?,
        })
    }
}

/// Sprawy, których stanu Recepcja jeszcze nie dostała: nowe, zmienione albo zamknięte. Znaki do
/// przekazania podaje znaki_do_przekazania_recepcji.
pub fn do_przekazania_recepcji(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<DoPrzekazaniaRecepcjiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_dzialu_spis_sprawy
WHERE przekazano IS NULL
  OR (zamknieto IS NOT NULL AND zamkniecie_przekazano IS NULL)
ORDER BY id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(DoPrzekazaniaRecepcjiRow::from_row).collect()
}

/// The parameters of `wpisz_sprawe_do_spisu`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszSpraweDoSpisuParams {
    pub teczka_odnosnik: String,
    pub rodzaj: SprawyRodzaj,
    pub oznaczenie: String,
    pub agent: String,
    pub otwarto: String,
    pub kancelaria_xnotariusz: Option<bool>,
}

/// Nowa sprawa działu w spisie, gdy kierownik powołał jej agenta; przy transakcji z informacją, czy
/// kancelaria pracuje na XNotariuszu.
pub fn wpisz_sprawe_do_spisu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszSpraweDoSpisuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_dzialu_spis_sprawy (teczka_odnosnik, rodzaj, oznaczenie, agent, otwarto, kancelaria_xnotariusz)
VALUES (:teczka_odnosnik, :rodzaj, :oznaczenie, :agent, :otwarto, :kancelaria_xnotariusz)"#,
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
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_sprawe_w_spisie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZmienSpraweWSpisieParams {
    pub oznaczenie: String,
    pub agent: String,
    pub kancelaria_xnotariusz: Option<bool>,
    pub id: i64,
}

/// Sprawa się zmieniła, np. ma nowe oznaczenie albo agenta po wymianie; nowy stan czeka na
/// przekazanie Recepcji.
pub fn zmien_sprawe_w_spisie(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienSpraweWSpisieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_dzialu_spis_sprawy
SET oznaczenie = :oznaczenie,
  agent = :agent,
  kancelaria_xnotariusz = :kancelaria_xnotariusz,
  przekazano = NULL
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
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_przekazanie_sprawy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPrzekazanieSprawyParams {
    pub przekazano: String,
    pub id: i64,
}

/// Recepcja dostała bieżący stan sprawy.
pub fn odnotuj_przekazanie_sprawy(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPrzekazanieSprawyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_dzialu_spis_sprawy
SET przekazano = :przekazano
WHERE id = :id"#,
                parameters: Vec::from([
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

/// The parameters of `zamknij_sprawe_w_spisie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZamknijSpraweWSpisieParams {
    pub zamknieto: String,
    pub id: i64,
}

/// Sprawa się zamknęła; zostaje w spisie, a zamknięcie czeka na przekazanie Recepcji.
pub fn zamknij_sprawe_w_spisie(
    room: &impl xmeta_room::MetaRoom,
    params: &ZamknijSpraweWSpisieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_dzialu_spis_sprawy
SET zamknieto = :zamknieto
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zamknieto",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zamknieto),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_przekazanie_zamkniecia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPrzekazanieZamknieciaParams {
    pub zamkniecie_przekazano: String,
    pub id: i64,
}

/// Recepcja dostała zamknięcie sprawy.
pub fn odnotuj_przekazanie_zamkniecia(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPrzekazanieZamknieciaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_dzialu_spis_sprawy
SET zamkniecie_przekazano = :zamkniecie_przekazano
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zamkniecie_przekazano",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.zamkniecie_przekazano,
                        ),
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
    pub dodano: String,
    pub przekazano: Option<String>,
    pub wycofano: Option<String>,
    pub wycofanie_przekazano: Option<String>,
}

impl ZnakiSprawyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZnakiSprawyRow {
            id: row.get("id")?,
            sprawa_id: row.get("sprawa_id")?,
            rodzaj: row.get("rodzaj")?,
            wartosc: row.get("wartosc")?,
            dodano: row.get("dodano")?,
            przekazano: row.get("przekazano")?,
            wycofano: row.get("wycofano")?,
            wycofanie_przekazano: row.get("wycofanie_przekazano")?,
        })
    }
}

/// Znaki jednej sprawy, także wycofane, od najstarszego.
pub fn znaki_sprawy(
    room: &impl xmeta_room::MetaRoom,
    params: &ZnakiSprawyParams,
) -> Result<Vec<ZnakiSprawyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_dzialu_spis_znaki
WHERE sprawa_id = :sprawa_id
ORDER BY dodano, id"#,
        parameters: Vec::from([
            ("sprawa_id", xmeta_room::ToSqlValue::to_sql_value(&params.sprawa_id)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(ZnakiSprawyRow::from_row).collect()
}

/// A row of `znaki_do_przekazania_recepcji`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZnakiDoPrzekazaniaRecepcjiRow {
    pub id: i64,
    pub sprawa_id: i64,
    pub rodzaj: ZnakiRodzaj,
    pub wartosc: String,
    pub dodano: String,
    pub przekazano: Option<String>,
    pub wycofano: Option<String>,
    pub wycofanie_przekazano: Option<String>,
}

impl ZnakiDoPrzekazaniaRecepcjiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZnakiDoPrzekazaniaRecepcjiRow {
            id: row.get("id")?,
            sprawa_id: row.get("sprawa_id")?,
            rodzaj: row.get("rodzaj")?,
            wartosc: row.get("wartosc")?,
            dodano: row.get("dodano")?,
            przekazano: row.get("przekazano")?,
            wycofano: row.get("wycofano")?,
            wycofanie_przekazano: row.get("wycofanie_przekazano")?,
        })
    }
}

/// Znaki, których Recepcja jeszcze nie dostała, i wycofania znaków, które już dostała.
pub fn znaki_do_przekazania_recepcji(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZnakiDoPrzekazaniaRecepcjiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_dzialu_spis_znaki
WHERE (przekazano IS NULL AND wycofano IS NULL)
  OR (przekazano IS NOT NULL AND wycofano IS NOT NULL AND wycofanie_przekazano IS NULL)
ORDER BY sprawa_id, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(ZnakiDoPrzekazaniaRecepcjiRow::from_row).collect()
}

/// The parameters of `dodaj_znak`.
#[derive(Debug, Clone, PartialEq)]
pub struct DodajZnakParams {
    pub sprawa_id: i64,
    pub rodzaj: ZnakiRodzaj,
    pub wartosc: String,
    pub dodano: String,
}

/// Nowy znak rozpoznawczy sprawy, np. adres nieruchomości.
pub fn dodaj_znak(
    room: &impl xmeta_room::MetaRoom,
    params: &DodajZnakParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_dzialu_spis_znaki (sprawa_id, rodzaj, wartosc, dodano)
VALUES (:sprawa_id, :rodzaj, :wartosc, :dodano)"#,
                parameters: Vec::from([
                    (
                        "sprawa_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawa_id),
                    ),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("wartosc", xmeta_room::ToSqlValue::to_sql_value(&params.wartosc)),
                    ("dodano", xmeta_room::ToSqlValue::to_sql_value(&params.dodano)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_przekazanie_znaku`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPrzekazanieZnakuParams {
    pub przekazano: String,
    pub id: i64,
}

/// Recepcja dostała znak.
pub fn odnotuj_przekazanie_znaku(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPrzekazanieZnakuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_dzialu_spis_znaki
SET przekazano = :przekazano
WHERE id = :id"#,
                parameters: Vec::from([
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

/// The parameters of `wycofaj_znak`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WycofajZnakParams {
    pub wycofano: String,
    pub id: i64,
}

/// Znak przestał wskazywać sprawę, np. klient zmienił adres poczty; zostaje w spisie jako wycofany.
pub fn wycofaj_znak(
    room: &impl xmeta_room::MetaRoom,
    params: &WycofajZnakParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_dzialu_spis_znaki
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

/// The parameters of `odnotuj_przekazanie_wycofania`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPrzekazanieWycofaniaParams {
    pub wycofanie_przekazano: String,
    pub id: i64,
}

/// Recepcja dostała wycofanie znaku.
pub fn odnotuj_przekazanie_wycofania(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPrzekazanieWycofaniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_dzialu_spis_znaki
SET wycofanie_przekazano = :wycofanie_przekazano
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "wycofanie_przekazano",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.wycofanie_przekazano,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
