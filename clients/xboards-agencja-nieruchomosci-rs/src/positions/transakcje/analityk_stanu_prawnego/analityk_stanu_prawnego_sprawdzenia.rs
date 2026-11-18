//! The kind `analityk-stanu-prawnego-sprawdzenia` of the room `positions/transakcje/analityk_stanu_prawnego`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "analityk-stanu-prawnego-sprawdzenia",
    requires: &[&super::analityk_stanu_prawnego_ksiega::KIND],
    migrations: &[
        r#"-- Sprawdzenia: każde sprawdzenie stanu prawnego osobno, np. dział III księgi, pierwokup gminy albo
-- zgodność projektu aktu z umową przedwstępną. Analiza w teczce transakcji zbiera wnioski dla stron;
-- pamięć trzyma pojedyncze sprawdzenia z dokumentem, przepisem i wynikiem, na których analiza stoi.
--
-- Sprawdzenie działu księgi stoi na stanie księgi z chwili odpisu albo przeglądania
-- (analityk-stanu-prawnego-ksiega). Sprawdzenia się nie zmienia; wolno raz odnotować analizę,
-- do której weszło. Ponowne sprawdzenie, np. po nowym odpisie, jest nowym wierszem, a poprzednie
-- wskazuje je jako zastępujące.
CREATE TABLE analityk_stanu_prawnego_sprawdzenia_wpisy (
  id TEXT PRIMARY KEY,
  przedmiot TEXT NOT NULL CHECK (przedmiot IN ('dzial_i', 'dzial_ii', 'dzial_iii', 'dzial_iv', 'pierwokup', 'ograniczenia', 'tytul_zbywcy', 'projekt_umowy')),
  -- Sprawdzany dokument z teczki, np. odpis księgi albo projekt umowy, i jego data.
  dokument_odnosnik TEXT NOT NULL CHECK (dokument_odnosnik <> ''),
  dokument_data TEXT NOT NULL,
  -- Np. „art. 109 ust. 1 pkt 1 ugn”.
  przepis TEXT NOT NULL CHECK (przepis <> ''),
  wynik TEXT NOT NULL CHECK (wynik IN ('bez_uwag', 'ryzyko', 'nie_mozna_ustalic')),
  opis TEXT,
  -- Analiza w teczce transakcji, do której weszło sprawdzenie, tekstem.
  analiza_odnosnik TEXT,
  ksiega_stan_id INTEGER REFERENCES analityk_stanu_prawnego_ksiega_stany (id),
  sprawdzono TEXT NOT NULL,
  zastapione_przez_id TEXT REFERENCES analityk_stanu_prawnego_sprawdzenia_wpisy (id),

  CONSTRAINT ryzyko_z_opisem CHECK (wynik = 'bez_uwag' OR opis IS NOT NULL),
  CONSTRAINT dzial_ksiegi_ze_stanem CHECK (
    przedmiot NOT IN ('dzial_i', 'dzial_ii', 'dzial_iii', 'dzial_iv')
    OR ksiega_stan_id IS NOT NULL
  ),
  CONSTRAINT dokument_nie_po_sprawdzeniu CHECK (dokument_data <= sprawdzono),
  CONSTRAINT zastapione_nie_przez_siebie CHECK (zastapione_przez_id IS NULL OR zastapione_przez_id <> id)
) STRICT;

-- Analiza i zastąpienie przychodzą później, więc sprawdzenie powstaje aktualne i bez analizy.
CREATE TRIGGER analityk_stanu_prawnego_sprawdzenia_powstaje_aktualne
BEFORE INSERT ON analityk_stanu_prawnego_sprawdzenia_wpisy
WHEN NEW.zastapione_przez_id IS NOT NULL
  OR NEW.analiza_odnosnik IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'sprawdzenie powstaje aktualne i bez analizy; obie rzeczy odnotowuje się później');
END;

-- Sprawdzenie jest zapisem tego, co Analityk wiedział w chwili sprawdzenia; zmienia się tylko raz
-- odnotowana analiza i raz odnotowane zastąpienie.
CREATE TRIGGER analityk_stanu_prawnego_sprawdzenia_bez_zmian
BEFORE UPDATE ON analityk_stanu_prawnego_sprawdzenia_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.przedmiot IS NOT OLD.przedmiot
  OR NEW.dokument_odnosnik IS NOT OLD.dokument_odnosnik
  OR NEW.dokument_data IS NOT OLD.dokument_data
  OR NEW.przepis IS NOT OLD.przepis
  OR NEW.wynik IS NOT OLD.wynik
  OR NEW.opis IS NOT OLD.opis
  OR NEW.ksiega_stan_id IS NOT OLD.ksiega_stan_id
  OR NEW.sprawdzono IS NOT OLD.sprawdzono
  OR (OLD.analiza_odnosnik IS NOT NULL AND NEW.analiza_odnosnik IS NOT OLD.analiza_odnosnik)
  OR (OLD.zastapione_przez_id IS NOT NULL AND NEW.zastapione_przez_id IS NOT OLD.zastapione_przez_id)
BEGIN
  SELECT RAISE(ABORT, 'sprawdzenia się nie zmienia; wolno raz odnotować analizę i raz zastąpienie');
END;

-- Ponowne sprawdzenie dotyczy tego samego przedmiotu i nie jest starsze od tego, które zastępuje.
CREATE TRIGGER analityk_stanu_prawnego_sprawdzenia_zastepuje_ponowne
BEFORE UPDATE OF zastapione_przez_id ON analityk_stanu_prawnego_sprawdzenia_wpisy
WHEN NEW.zastapione_przez_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM analityk_stanu_prawnego_sprawdzenia_wpisy AS ponowne
    WHERE ponowne.id = NEW.zastapione_przez_id
      AND ponowne.przedmiot = OLD.przedmiot
      AND ponowne.sprawdzono >= OLD.sprawdzono
  )
BEGIN
  SELECT RAISE(ABORT, 'sprawdzenie zastępuje ponowne sprawdzenie tego samego przedmiotu, nie starsze');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER analityk_stanu_prawnego_sprawdzenia_wpisy_bez_zastepowania
BEFORE INSERT ON analityk_stanu_prawnego_sprawdzenia_wpisy
WHEN EXISTS (
  SELECT 1
  FROM analityk_stanu_prawnego_sprawdzenia_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: sprawdzenie zostaje, bo po latach strona może zapytać, jaki był stan prawny w chwili
-- transakcji.
CREATE TRIGGER analityk_stanu_prawnego_sprawdzenia_wpisy_bez_usuwania
BEFORE DELETE ON analityk_stanu_prawnego_sprawdzenia_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
    ],
};

/// The values of column `wpisy.przedmiot`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyPrzedmiot {
    DzialI,
    DzialIi,
    DzialIii,
    DzialIv,
    Pierwokup,
    Ograniczenia,
    TytulZbywcy,
    ProjektUmowy,
}

impl WpisyPrzedmiot {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyPrzedmiot] = &[
        WpisyPrzedmiot::DzialI,
        WpisyPrzedmiot::DzialIi,
        WpisyPrzedmiot::DzialIii,
        WpisyPrzedmiot::DzialIv,
        WpisyPrzedmiot::Pierwokup,
        WpisyPrzedmiot::Ograniczenia,
        WpisyPrzedmiot::TytulZbywcy,
        WpisyPrzedmiot::ProjektUmowy,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyPrzedmiot::DzialI => "dzial_i",
            WpisyPrzedmiot::DzialIi => "dzial_ii",
            WpisyPrzedmiot::DzialIii => "dzial_iii",
            WpisyPrzedmiot::DzialIv => "dzial_iv",
            WpisyPrzedmiot::Pierwokup => "pierwokup",
            WpisyPrzedmiot::Ograniczenia => "ograniczenia",
            WpisyPrzedmiot::TytulZbywcy => "tytul_zbywcy",
            WpisyPrzedmiot::ProjektUmowy => "projekt_umowy",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyPrzedmiot {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "dzial_i" => Ok(WpisyPrzedmiot::DzialI),
                    "dzial_ii" => Ok(WpisyPrzedmiot::DzialIi),
                    "dzial_iii" => Ok(WpisyPrzedmiot::DzialIii),
                    "dzial_iv" => Ok(WpisyPrzedmiot::DzialIv),
                    "pierwokup" => Ok(WpisyPrzedmiot::Pierwokup),
                    "ograniczenia" => Ok(WpisyPrzedmiot::Ograniczenia),
                    "tytul_zbywcy" => Ok(WpisyPrzedmiot::TytulZbywcy),
                    "projekt_umowy" => Ok(WpisyPrzedmiot::ProjektUmowy),
                    other => Err(format!("{other} is no value of WpisyPrzedmiot")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyPrzedmiot {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.wynik`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyWynik {
    BezUwag,
    Ryzyko,
    NieMoznaUstalic,
}

impl WpisyWynik {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyWynik] = &[
        WpisyWynik::BezUwag,
        WpisyWynik::Ryzyko,
        WpisyWynik::NieMoznaUstalic,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyWynik::BezUwag => "bez_uwag",
            WpisyWynik::Ryzyko => "ryzyko",
            WpisyWynik::NieMoznaUstalic => "nie_mozna_ustalic",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyWynik {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "bez_uwag" => Ok(WpisyWynik::BezUwag),
                    "ryzyko" => Ok(WpisyWynik::Ryzyko),
                    "nie_mozna_ustalic" => Ok(WpisyWynik::NieMoznaUstalic),
                    other => Err(format!("{other} is no value of WpisyWynik")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyWynik {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `sprawdzenia_stanu_prawnego`.
#[derive(Debug, Clone, PartialEq)]
pub struct SprawdzeniaStanuPrawnegoRow {
    pub id: String,
    pub przedmiot: WpisyPrzedmiot,
    pub dokument_odnosnik: String,
    pub dokument_data: String,
    pub przepis: String,
    pub wynik: WpisyWynik,
    pub opis: Option<String>,
    pub analiza_odnosnik: Option<String>,
    pub ksiega_stan_id: Option<i64>,
    pub sprawdzono: String,
    pub zastapione_przez_id: Option<String>,
}

impl SprawdzeniaStanuPrawnegoRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(SprawdzeniaStanuPrawnegoRow {
            id: row.get("id")?,
            przedmiot: row.get("przedmiot")?,
            dokument_odnosnik: row.get("dokument_odnosnik")?,
            dokument_data: row.get("dokument_data")?,
            przepis: row.get("przepis")?,
            wynik: row.get("wynik")?,
            opis: row.get("opis")?,
            analiza_odnosnik: row.get("analiza_odnosnik")?,
            ksiega_stan_id: row.get("ksiega_stan_id")?,
            sprawdzono: row.get("sprawdzono")?,
            zastapione_przez_id: row.get("zastapione_przez_id")?,
        })
    }
}

/// Aktualne sprawdzenia (niezastąpione); najpierw ryzyka i to, czego nie można ustalić.
pub fn sprawdzenia_stanu_prawnego(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<SprawdzeniaStanuPrawnegoRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM analityk_stanu_prawnego_sprawdzenia_wpisy
WHERE zastapione_przez_id IS NULL
ORDER BY wynik = 'bez_uwag', sprawdzono DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(SprawdzeniaStanuPrawnegoRow::from_row).collect()
}

/// The parameters of `sprawdzenia_dokumentu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SprawdzeniaDokumentuParams {
    pub dokument_odnosnik: String,
}

/// A row of `sprawdzenia_dokumentu`.
#[derive(Debug, Clone, PartialEq)]
pub struct SprawdzeniaDokumentuRow {
    pub id: String,
    pub przedmiot: WpisyPrzedmiot,
    pub dokument_odnosnik: String,
    pub dokument_data: String,
    pub przepis: String,
    pub wynik: WpisyWynik,
    pub opis: Option<String>,
    pub analiza_odnosnik: Option<String>,
    pub ksiega_stan_id: Option<i64>,
    pub sprawdzono: String,
    pub zastapione_przez_id: Option<String>,
}

impl SprawdzeniaDokumentuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(SprawdzeniaDokumentuRow {
            id: row.get("id")?,
            przedmiot: row.get("przedmiot")?,
            dokument_odnosnik: row.get("dokument_odnosnik")?,
            dokument_data: row.get("dokument_data")?,
            przepis: row.get("przepis")?,
            wynik: row.get("wynik")?,
            opis: row.get("opis")?,
            analiza_odnosnik: row.get("analiza_odnosnik")?,
            ksiega_stan_id: row.get("ksiega_stan_id")?,
            sprawdzono: row.get("sprawdzono")?,
            zastapione_przez_id: row.get("zastapione_przez_id")?,
        })
    }
}

/// Aktualne sprawdzenia, które stoją na jednym dokumencie; po nowej wersji dokumentu Analityk
/// sprawdza je ponownie.
pub fn sprawdzenia_dokumentu(
    room: &impl xmeta_room::MetaRoom,
    params: &SprawdzeniaDokumentuParams,
) -> Result<Vec<SprawdzeniaDokumentuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM analityk_stanu_prawnego_sprawdzenia_wpisy
WHERE dokument_odnosnik = :dokument_odnosnik
  AND zastapione_przez_id IS NULL
ORDER BY sprawdzono, id"#,
        parameters: Vec::from([
            (
                "dokument_odnosnik",
                xmeta_room::ToSqlValue::to_sql_value(&params.dokument_odnosnik),
            ),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(SprawdzeniaDokumentuRow::from_row).collect()
}

/// A row of `sprawdzenia_bez_analizy`.
#[derive(Debug, Clone, PartialEq)]
pub struct SprawdzeniaBezAnalizyRow {
    pub id: String,
    pub przedmiot: WpisyPrzedmiot,
    pub dokument_odnosnik: String,
    pub dokument_data: String,
    pub przepis: String,
    pub wynik: WpisyWynik,
    pub opis: Option<String>,
    pub analiza_odnosnik: Option<String>,
    pub ksiega_stan_id: Option<i64>,
    pub sprawdzono: String,
    pub zastapione_przez_id: Option<String>,
}

impl SprawdzeniaBezAnalizyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(SprawdzeniaBezAnalizyRow {
            id: row.get("id")?,
            przedmiot: row.get("przedmiot")?,
            dokument_odnosnik: row.get("dokument_odnosnik")?,
            dokument_data: row.get("dokument_data")?,
            przepis: row.get("przepis")?,
            wynik: row.get("wynik")?,
            opis: row.get("opis")?,
            analiza_odnosnik: row.get("analiza_odnosnik")?,
            ksiega_stan_id: row.get("ksiega_stan_id")?,
            sprawdzono: row.get("sprawdzono")?,
            zastapione_przez_id: row.get("zastapione_przez_id")?,
        })
    }
}

/// Aktualne sprawdzenia, które jeszcze nie weszły do analizy w teczce.
pub fn sprawdzenia_bez_analizy(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<SprawdzeniaBezAnalizyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM analityk_stanu_prawnego_sprawdzenia_wpisy
WHERE analiza_odnosnik IS NULL
  AND zastapione_przez_id IS NULL
ORDER BY sprawdzono, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(SprawdzeniaBezAnalizyRow::from_row).collect()
}

/// The parameters of `zapisz_sprawdzenie_stanu_prawnego`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZapiszSprawdzenieStanuPrawnegoParams {
    pub id: String,
    pub przedmiot: WpisyPrzedmiot,
    pub dokument_odnosnik: String,
    pub dokument_data: String,
    pub przepis: String,
    pub wynik: WpisyWynik,
    pub opis: Option<String>,
    pub ksiega_stan_id: Option<i64>,
    pub sprawdzono: String,
}

/// Sprawdzenie z dokumentem, przepisem i wynikiem; identyfikator (UUID) nadaje aplikacja, żeby
/// poprzednie sprawdzenie mogło go wskazać jako zastępujące.
pub fn zapisz_sprawdzenie_stanu_prawnego(
    room: &impl xmeta_room::MetaRoom,
    params: &ZapiszSprawdzenieStanuPrawnegoParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO analityk_stanu_prawnego_sprawdzenia_wpisy (id, przedmiot, dokument_odnosnik,
  dokument_data, przepis, wynik, opis, ksiega_stan_id, sprawdzono)
VALUES (:id, :przedmiot, :dokument_odnosnik, :dokument_data, :przepis, :wynik, :opis,
  :ksiega_stan_id, :sprawdzono)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "przedmiot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przedmiot),
                    ),
                    (
                        "dokument_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_odnosnik),
                    ),
                    (
                        "dokument_data",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_data),
                    ),
                    ("przepis", xmeta_room::ToSqlValue::to_sql_value(&params.przepis)),
                    ("wynik", xmeta_room::ToSqlValue::to_sql_value(&params.wynik)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    (
                        "ksiega_stan_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.ksiega_stan_id),
                    ),
                    (
                        "sprawdzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzono),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_analize_sprawdzenia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujAnalizeSprawdzeniaParams {
    pub analiza_odnosnik: String,
    pub id: String,
}

/// Sprawdzenie weszło do analizy w teczce transakcji.
pub fn odnotuj_analize_sprawdzenia(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujAnalizeSprawdzeniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE analityk_stanu_prawnego_sprawdzenia_wpisy
SET analiza_odnosnik = :analiza_odnosnik
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "analiza_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.analiza_odnosnik),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `zastap_sprawdzenie_stanu_prawnego`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZastapSprawdzenieStanuPrawnegoParams {
    pub zastapione_przez_id: String,
    pub id: String,
}

/// Poprzednie sprawdzenie staje się nieaktualne, bo zastępuje je ponowne, zapisane wcześniej.
pub fn zastap_sprawdzenie_stanu_prawnego(
    room: &impl xmeta_room::MetaRoom,
    params: &ZastapSprawdzenieStanuPrawnegoParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE analityk_stanu_prawnego_sprawdzenia_wpisy
SET zastapione_przez_id = :zastapione_przez_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zastapione_przez_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zastapione_przez_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
