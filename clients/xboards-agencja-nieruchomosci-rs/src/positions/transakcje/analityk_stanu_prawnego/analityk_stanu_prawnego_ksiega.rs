//! The kind `analityk-stanu-prawnego-ksiega` of the room `positions/transakcje/analityk_stanu_prawnego`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "analityk-stanu-prawnego-ksiega",
    requires: &[],
    migrations: &[
        r#"-- Stan księgi: data i godzina każdego odpisu albo przeglądania księgi wieczystej, na którym stoi
-- analiza, i lista wpisów, wzmianek i ostrzeżeń z tej chwili. Dzięki temu Analityk widzi, co się
-- zmieniło przed aktem (operacja zmiany_ksiegi).
--
-- Sam odpis leży w teczce transakcji; pamięć wskazuje go odnośnikiem. Wpis księgi trzyma tylko
-- oznaczenie i przedmiot, np. „hipoteka umowna na rzecz banku” albo „wzmianka o wniosku”, bez
-- danych osób, bo te są w odpisie w teczce. Tego baza nie sprawdza: pilnuje norma stanowiska.
CREATE TABLE analityk_stanu_prawnego_ksiega_stany (
  -- Klucz nadany przez bazę jest dodatni; zob. analityk_stanu_prawnego_ksiega_stany_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  numer_kw TEXT NOT NULL CHECK (numer_kw GLOB '[A-Z][A-Z][0-9][A-Z]/[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]/[0-9]'),
  -- Data i godzina, z której odpis albo przeglądanie pokazuje księgę.
  chwila TEXT NOT NULL,
  sposob TEXT NOT NULL CHECK (sposob IN ('odpis', 'przegladanie')),
  -- Odpis w teczce transakcji: pokój i wiersz dokumentu, tekstem.
  dokument_odnosnik TEXT,

  -- Wydruk z przeglądania nie ma mocy dokumentu sądowego, ale odpis jest dokumentem w teczce.
  CONSTRAINT odpis_z_dokumentem CHECK (sposob <> 'odpis' OR dokument_odnosnik IS NOT NULL),
  CONSTRAINT jeden_stan_na_chwile UNIQUE (numer_kw, chwila)
) STRICT;

CREATE TABLE analityk_stanu_prawnego_ksiega_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. analityk_stanu_prawnego_ksiega_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  stan_id INTEGER NOT NULL REFERENCES analityk_stanu_prawnego_ksiega_stany (id),
  -- Dział I-O, I-Sp, II, III albo IV.
  dzial TEXT NOT NULL CHECK (dzial IN ('i_o', 'i_sp', 'ii', 'iii', 'iv')),
  -- Numer wpisu albo wzmianki w dziale, np. „1” albo „Dz.Kw./12345/26”.
  numer TEXT NOT NULL CHECK (numer <> ''),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('wpis', 'wzmianka', 'ostrzezenie')),
  przedmiot TEXT NOT NULL CHECK (przedmiot <> ''),

  CONSTRAINT wpis_raz_w_stanie UNIQUE (stan_id, dzial, rodzaj, numer)
) STRICT;

-- Nowy stan pokazuje księgę później niż poprzedni, więc ostatni stan jest zawsze najnowszy.
CREATE TRIGGER analityk_stanu_prawnego_ksiega_stan_po_poprzednim
BEFORE INSERT ON analityk_stanu_prawnego_ksiega_stany
WHEN EXISTS (
  SELECT 1
  FROM analityk_stanu_prawnego_ksiega_stany AS poprzedni
  WHERE poprzedni.numer_kw = NEW.numer_kw
    AND poprzedni.chwila >= NEW.chwila
)
BEGIN
  SELECT RAISE(ABORT, 'nowy stan księgi pokazuje ją później niż poprzedni');
END;

-- Wpisy zapisuje się razem ze stanem, więc stan, po którym przyszedł nowszy, jest zamknięty.
CREATE TRIGGER analityk_stanu_prawnego_ksiega_wpis_do_ostatniego_stanu
BEFORE INSERT ON analityk_stanu_prawnego_ksiega_wpisy
WHEN EXISTS (
  SELECT 1
  FROM analityk_stanu_prawnego_ksiega_stany AS stan
  JOIN analityk_stanu_prawnego_ksiega_stany AS nowszy
    ON nowszy.numer_kw = stan.numer_kw
    AND nowszy.chwila > stan.chwila
  WHERE stan.id = NEW.stan_id
)
BEGIN
  SELECT RAISE(ABORT, 'wpis księgi dopisuje się tylko do ostatniego stanu księgi');
END;

-- Stan księgi to zapis jednej chwili, więc się go nie zmienia.
CREATE TRIGGER analityk_stanu_prawnego_ksiega_stany_bez_zmian
BEFORE UPDATE ON analityk_stanu_prawnego_ksiega_stany
BEGIN
  SELECT RAISE(ABORT, 'stanu księgi ani jego wpisów się nie zmienia; nowy odpis to nowy stan');
END;

CREATE TRIGGER analityk_stanu_prawnego_ksiega_wpisy_bez_zmian
BEFORE UPDATE ON analityk_stanu_prawnego_ksiega_wpisy
BEGIN
  SELECT RAISE(ABORT, 'stanu księgi ani jego wpisów się nie zmienia; nowy odpis to nowy stan');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER analityk_stanu_prawnego_ksiega_stany_bez_zastepowania
BEFORE INSERT ON analityk_stanu_prawnego_ksiega_stany
WHEN EXISTS (
  SELECT 1
  FROM analityk_stanu_prawnego_ksiega_stany
  WHERE id = NEW.id
    OR (numer_kw = NEW.numer_kw AND chwila = NEW.chwila)
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER analityk_stanu_prawnego_ksiega_wpisy_bez_zastepowania
BEFORE INSERT ON analityk_stanu_prawnego_ksiega_wpisy
WHEN EXISTS (
  SELECT 1
  FROM analityk_stanu_prawnego_ksiega_wpisy
  WHERE id = NEW.id
    OR (stan_id = NEW.stan_id AND dzial = NEW.dzial AND rodzaj = NEW.rodzaj AND numer = NEW.numer)
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: stan księgi zostaje, bo po latach strona może zapytać, jaki był stan prawny w chwili
-- transakcji.
CREATE TRIGGER analityk_stanu_prawnego_ksiega_stany_bez_usuwania
BEFORE DELETE ON analityk_stanu_prawnego_ksiega_stany
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER analityk_stanu_prawnego_ksiega_wpisy_bez_usuwania
BEFORE DELETE ON analityk_stanu_prawnego_ksiega_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
    ],
};

/// The values of column `stany.sposob`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StanySposob {
    Odpis,
    Przegladanie,
}

impl StanySposob {
    /// Every value, in the order of the catalog.
    pub const ALL: &[StanySposob] = &[StanySposob::Odpis, StanySposob::Przegladanie];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            StanySposob::Odpis => "odpis",
            StanySposob::Przegladanie => "przegladanie",
        }
    }
}

impl xmeta_room::FromSqlValue for StanySposob {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "odpis" => Ok(StanySposob::Odpis),
                    "przegladanie" => Ok(StanySposob::Przegladanie),
                    other => Err(format!("{other} is no value of StanySposob")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for StanySposob {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.dzial`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyDzial {
    IO,
    ISp,
    Ii,
    Iii,
    Iv,
}

impl WpisyDzial {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyDzial] = &[
        WpisyDzial::IO,
        WpisyDzial::ISp,
        WpisyDzial::Ii,
        WpisyDzial::Iii,
        WpisyDzial::Iv,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyDzial::IO => "i_o",
            WpisyDzial::ISp => "i_sp",
            WpisyDzial::Ii => "ii",
            WpisyDzial::Iii => "iii",
            WpisyDzial::Iv => "iv",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyDzial {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "i_o" => Ok(WpisyDzial::IO),
                    "i_sp" => Ok(WpisyDzial::ISp),
                    "ii" => Ok(WpisyDzial::Ii),
                    "iii" => Ok(WpisyDzial::Iii),
                    "iv" => Ok(WpisyDzial::Iv),
                    other => Err(format!("{other} is no value of WpisyDzial")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyDzial {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    Wpis,
    Wzmianka,
    Ostrzezenie,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::Wpis,
        WpisyRodzaj::Wzmianka,
        WpisyRodzaj::Ostrzezenie,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::Wpis => "wpis",
            WpisyRodzaj::Wzmianka => "wzmianka",
            WpisyRodzaj::Ostrzezenie => "ostrzezenie",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "wpis" => Ok(WpisyRodzaj::Wpis),
                    "wzmianka" => Ok(WpisyRodzaj::Wzmianka),
                    "ostrzezenie" => Ok(WpisyRodzaj::Ostrzezenie),
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

/// The values of field `zmiana` of `zmiany_ksiegi`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Zmiana {
    Dodany,
    Usuniety,
}

impl Zmiana {
    /// Every value, in the order of the catalog.
    pub const ALL: &[Zmiana] = &[Zmiana::Dodany, Zmiana::Usuniety];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            Zmiana::Dodany => "dodany",
            Zmiana::Usuniety => "usuniety",
        }
    }
}

impl xmeta_room::FromSqlValue for Zmiana {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "dodany" => Ok(Zmiana::Dodany),
                    "usuniety" => Ok(Zmiana::Usuniety),
                    other => Err(format!("{other} is no value of Zmiana")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for Zmiana {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The parameters of `stany_ksiegi`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StanyKsiegiParams {
    pub numer_kw: String,
}

/// A row of `stany_ksiegi`.
#[derive(Debug, Clone, PartialEq)]
pub struct StanyKsiegiRow {
    pub id: i64,
    pub numer_kw: String,
    pub chwila: String,
    pub sposob: StanySposob,
    pub dokument_odnosnik: Option<String>,
}

impl StanyKsiegiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(StanyKsiegiRow {
            id: row.get("id")?,
            numer_kw: row.get("numer_kw")?,
            chwila: row.get("chwila")?,
            sposob: row.get("sposob")?,
            dokument_odnosnik: row.get("dokument_odnosnik")?,
        })
    }
}

/// Stany jednej księgi, od najnowszego.
pub fn stany_ksiegi(
    room: &impl xmeta_room::MetaRoom,
    params: &StanyKsiegiParams,
) -> Result<Vec<StanyKsiegiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM analityk_stanu_prawnego_ksiega_stany
WHERE numer_kw = :numer_kw
ORDER BY chwila DESC, id"#,
        parameters: Vec::from([
            ("numer_kw", xmeta_room::ToSqlValue::to_sql_value(&params.numer_kw)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(StanyKsiegiRow::from_row).collect()
}

/// The parameters of `zapisz_stan_ksiegi`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZapiszStanKsiegiParams {
    pub numer_kw: String,
    pub chwila: String,
    pub sposob: StanySposob,
    pub dokument_odnosnik: Option<String>,
}

/// Nowy odpis albo przeglądanie księgi; wpisy z tej chwili dopisuje zapisz_wpis_ksiegi.
pub fn zapisz_stan_ksiegi(
    room: &impl xmeta_room::MetaRoom,
    params: &ZapiszStanKsiegiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO analityk_stanu_prawnego_ksiega_stany (numer_kw, chwila, sposob, dokument_odnosnik)
VALUES (:numer_kw, :chwila, :sposob, :dokument_odnosnik)"#,
                parameters: Vec::from([
                    ("numer_kw", xmeta_room::ToSqlValue::to_sql_value(&params.numer_kw)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    ("sposob", xmeta_room::ToSqlValue::to_sql_value(&params.sposob)),
                    (
                        "dokument_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_odnosnik),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `wpisy_stanu_ksiegi`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpisyStanuKsiegiParams {
    pub stan_id: i64,
}

/// A row of `wpisy_stanu_ksiegi`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpisyStanuKsiegiRow {
    pub id: i64,
    pub stan_id: i64,
    pub dzial: WpisyDzial,
    pub numer: String,
    pub rodzaj: WpisyRodzaj,
    pub przedmiot: String,
}

impl WpisyStanuKsiegiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WpisyStanuKsiegiRow {
            id: row.get("id")?,
            stan_id: row.get("stan_id")?,
            dzial: row.get("dzial")?,
            numer: row.get("numer")?,
            rodzaj: row.get("rodzaj")?,
            przedmiot: row.get("przedmiot")?,
        })
    }
}

/// Wpisy, wzmianki i ostrzeżenia jednego stanu księgi według działów.
pub fn wpisy_stanu_ksiegi(
    room: &impl xmeta_room::MetaRoom,
    params: &WpisyStanuKsiegiParams,
) -> Result<Vec<WpisyStanuKsiegiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM analityk_stanu_prawnego_ksiega_wpisy
WHERE stan_id = :stan_id
ORDER BY dzial, numer, rodzaj, id"#,
        parameters: Vec::from([
            ("stan_id", xmeta_room::ToSqlValue::to_sql_value(&params.stan_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(WpisyStanuKsiegiRow::from_row).collect()
}

/// The parameters of `zapisz_wpis_ksiegi`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZapiszWpisKsiegiParams {
    pub stan_id: i64,
    pub dzial: WpisyDzial,
    pub numer: String,
    pub rodzaj: WpisyRodzaj,
    pub przedmiot: String,
}

/// Wpis, wzmianka albo ostrzeżenie z ostatniego stanu księgi, bez danych osób.
pub fn zapisz_wpis_ksiegi(
    room: &impl xmeta_room::MetaRoom,
    params: &ZapiszWpisKsiegiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO analityk_stanu_prawnego_ksiega_wpisy (stan_id, dzial, numer, rodzaj, przedmiot)
VALUES (:stan_id, :dzial, :numer, :rodzaj, :przedmiot)"#,
                parameters: Vec::from([
                    ("stan_id", xmeta_room::ToSqlValue::to_sql_value(&params.stan_id)),
                    ("dzial", xmeta_room::ToSqlValue::to_sql_value(&params.dzial)),
                    ("numer", xmeta_room::ToSqlValue::to_sql_value(&params.numer)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    (
                        "przedmiot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przedmiot),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zmiany_ksiegi`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZmianyKsiegiParams {
    pub numer_kw: String,
}

/// A row of `zmiany_ksiegi`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZmianyKsiegiRow {
    pub zmiana: Zmiana,
    pub dzial: WpisyDzial,
    pub numer: String,
    pub rodzaj: WpisyRodzaj,
    pub przedmiot: String,
}

impl ZmianyKsiegiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZmianyKsiegiRow {
            zmiana: row.get("zmiana")?,
            dzial: row.get("dzial")?,
            numer: row.get("numer")?,
            rodzaj: row.get("rodzaj")?,
            przedmiot: row.get("przedmiot")?,
        })
    }
}

/// Co się zmieniło w księdze między dwoma ostatnimi stanami: wpis, którego wcześniej nie było
/// (dodany), i wpis, którego już nie ma (usuniety). Zmieniona treść wpisu o tym samym numerze to
/// jeden usunięty i jeden dodany. Przy jednym stanie nie ma z czym porównać, więc zmian nie ma.
pub fn zmiany_ksiegi(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmianyKsiegiParams,
) -> Result<Vec<ZmianyKsiegiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"WITH ostatnie AS (
  SELECT id,
    chwila
  FROM analityk_stanu_prawnego_ksiega_stany
  WHERE numer_kw = :numer_kw
  ORDER BY chwila DESC
  LIMIT 2
),
nowy AS (
  SELECT id
  FROM ostatnie
  ORDER BY chwila DESC
  LIMIT 1
),
stary AS (
  SELECT id
  FROM ostatnie
  ORDER BY chwila DESC
  LIMIT 1 OFFSET 1
)
SELECT 'dodany' AS zmiana,
  wpis.dzial,
  wpis.numer,
  wpis.rodzaj,
  wpis.przedmiot
FROM analityk_stanu_prawnego_ksiega_wpisy AS wpis
WHERE wpis.stan_id = (SELECT id FROM nowy)
  AND EXISTS (SELECT 1 FROM stary)
  AND NOT EXISTS (
    SELECT 1
    FROM analityk_stanu_prawnego_ksiega_wpisy AS dawny
    WHERE dawny.stan_id = (SELECT id FROM stary)
      AND dawny.dzial = wpis.dzial
      AND dawny.numer = wpis.numer
      AND dawny.rodzaj = wpis.rodzaj
      AND dawny.przedmiot = wpis.przedmiot
  )
UNION ALL
SELECT 'usuniety' AS zmiana,
  wpis.dzial,
  wpis.numer,
  wpis.rodzaj,
  wpis.przedmiot
FROM analityk_stanu_prawnego_ksiega_wpisy AS wpis
WHERE wpis.stan_id = (SELECT id FROM stary)
  AND NOT EXISTS (
    SELECT 1
    FROM analityk_stanu_prawnego_ksiega_wpisy AS obecny
    WHERE obecny.stan_id = (SELECT id FROM nowy)
      AND obecny.dzial = wpis.dzial
      AND obecny.numer = wpis.numer
      AND obecny.rodzaj = wpis.rodzaj
      AND obecny.przedmiot = wpis.przedmiot
  )
ORDER BY dzial, numer, zmiana"#,
        parameters: Vec::from([
            ("numer_kw", xmeta_room::ToSqlValue::to_sql_value(&params.numer_kw)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZmianyKsiegiRow::from_row).collect()
}
