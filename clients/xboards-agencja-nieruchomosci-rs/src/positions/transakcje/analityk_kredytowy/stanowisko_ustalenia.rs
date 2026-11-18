//! The kind `stanowisko-ustalenia` of the room `positions/transakcje/analityk_kredytowy`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "stanowisko-ustalenia",
    requires: &[&super::stanowisko_zrodla::KIND],
    migrations: &[
        r#"-- Ustalenia: co agent ustalił o świecie, ze źródłem, pewnością, sposobem sprawdzenia i datą.
-- Polecenia leżą osobno (stanowisko-polecenia), bo to, co agentowi kazano, nie jest faktem o świecie.
--
-- Ustalenie jest zapisem tego, co agent wiedział i skąd. Nowy fakt to nowe ustalenie, a poprzednie
-- staje się nieaktualne. Tabela nie ma kolumny autora: autora każdego zapisu zapisuje dziennik
-- pokoju, więc następca agenta odróżnia w nim ustalenia poprzednika i może ocenić je na nowo.
CREATE TABLE stanowisko_ustalenia_wpisy (
  id TEXT PRIMARY KEY,
  tresc TEXT NOT NULL,
  zrodlo_id TEXT REFERENCES stanowisko_zrodla_wpisy (id),
  -- Wiersz teczki, z którego ustalenie wynika: tabela i pokój z wierszem, tekstem.
  wynika_z_tabela TEXT,
  wynika_z_odnosnik TEXT,
  -- Czego ustalenie dotyczy, gdy agent pracuje w wielu teczkach, np. teczka lokalu (jej metryka),
  -- propozycja w teczce poszukiwania albo pobyt gościa: tabela i pokój z wierszem, tekstem.
  dotyczy_tabela TEXT,
  dotyczy_odnosnik TEXT,
  pewnosc TEXT NOT NULL CHECK (pewnosc IN ('potwierdzone', 'prawdopodobne', 'niepewne')),
  sposob_sprawdzenia TEXT,
  ustalono TEXT NOT NULL,
  -- Gdzie ustalenie weszło, np. notatka w teczce albo raport.
  wykorzystano_tabela TEXT,
  wykorzystano_odnosnik TEXT,
  nieaktualne_od TEXT,
  zastapione_przez_id TEXT REFERENCES stanowisko_ustalenia_wpisy (id),

  CONSTRAINT ustalenie_ze_zrodlem CHECK (zrodlo_id IS NOT NULL OR wynika_z_odnosnik IS NOT NULL),
  CONSTRAINT wynika_z_tabela_razem_z_odnosnikiem CHECK ((wynika_z_tabela IS NULL) = (wynika_z_odnosnik IS NULL)),
  CONSTRAINT dotyczy_tabela_razem_z_odnosnikiem CHECK ((dotyczy_tabela IS NULL) = (dotyczy_odnosnik IS NULL)),
  CONSTRAINT wykorzystano_tabela_razem_z_odnosnikiem CHECK ((wykorzystano_tabela IS NULL) = (wykorzystano_odnosnik IS NULL)),
  CONSTRAINT potwierdzone_ze_sposobem_sprawdzenia CHECK (pewnosc <> 'potwierdzone' OR sposob_sprawdzenia IS NOT NULL),
  CONSTRAINT zastapione_jest_nieaktualne CHECK (zastapione_przez_id IS NULL OR nieaktualne_od IS NOT NULL)
) STRICT;

-- Nieaktualność przychodzi po ustaleniu, więc ustalenie powstaje aktualne.
CREATE TRIGGER stanowisko_ustalenia_powstaje_aktualne
BEFORE INSERT ON stanowisko_ustalenia_wpisy
WHEN NEW.nieaktualne_od IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'ustalenie powstaje aktualne; nieaktualność odnotowuje się później');
END;

-- Pewność, wykorzystanie i nieaktualność się zmieniają; to, co, skąd i o czym agent ustalił, już nie.
CREATE TRIGGER stanowisko_ustalenia_tresc_niezmienna
BEFORE UPDATE ON stanowisko_ustalenia_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.tresc IS NOT OLD.tresc
  OR NEW.zrodlo_id IS NOT OLD.zrodlo_id
  OR NEW.wynika_z_tabela IS NOT OLD.wynika_z_tabela
  OR NEW.wynika_z_odnosnik IS NOT OLD.wynika_z_odnosnik
  OR NEW.dotyczy_tabela IS NOT OLD.dotyczy_tabela
  OR NEW.dotyczy_odnosnik IS NOT OLD.dotyczy_odnosnik
  OR NEW.ustalono IS NOT OLD.ustalono
BEGIN
  SELECT RAISE(ABORT, 'treści, źródła, przedmiotu ani daty ustalenia się nie zmienia; nowy fakt to nowe ustalenie');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER stanowisko_ustalenia_wpisy_bez_zastepowania
BEFORE INSERT ON stanowisko_ustalenia_wpisy
WHEN EXISTS (
  SELECT 1
  FROM stanowisko_ustalenia_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: pamięć trwa bez terminu; ustalenie, które przestało być prawdą, staje się nieaktualne.
CREATE TRIGGER stanowisko_ustalenia_wpisy_bez_usuwania
BEFORE DELETE ON stanowisko_ustalenia_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
    ],
};

/// The values of column `wpisy.pewnosc`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyPewnosc {
    Potwierdzone,
    Prawdopodobne,
    Niepewne,
}

impl WpisyPewnosc {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyPewnosc] = &[
        WpisyPewnosc::Potwierdzone,
        WpisyPewnosc::Prawdopodobne,
        WpisyPewnosc::Niepewne,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyPewnosc::Potwierdzone => "potwierdzone",
            WpisyPewnosc::Prawdopodobne => "prawdopodobne",
            WpisyPewnosc::Niepewne => "niepewne",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyPewnosc {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "potwierdzone" => Ok(WpisyPewnosc::Potwierdzone),
                    "prawdopodobne" => Ok(WpisyPewnosc::Prawdopodobne),
                    "niepewne" => Ok(WpisyPewnosc::Niepewne),
                    other => Err(format!("{other} is no value of WpisyPewnosc")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyPewnosc {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `ustalenia`.
#[derive(Debug, Clone, PartialEq)]
pub struct UstaleniaRow {
    pub id: String,
    pub tresc: String,
    pub zrodlo_id: Option<String>,
    pub wynika_z_tabela: Option<String>,
    pub wynika_z_odnosnik: Option<String>,
    pub dotyczy_tabela: Option<String>,
    pub dotyczy_odnosnik: Option<String>,
    pub pewnosc: WpisyPewnosc,
    pub sposob_sprawdzenia: Option<String>,
    pub ustalono: String,
    pub wykorzystano_tabela: Option<String>,
    pub wykorzystano_odnosnik: Option<String>,
    pub nieaktualne_od: Option<String>,
    pub zastapione_przez_id: Option<String>,
}

impl UstaleniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(UstaleniaRow {
            id: row.get("id")?,
            tresc: row.get("tresc")?,
            zrodlo_id: row.get("zrodlo_id")?,
            wynika_z_tabela: row.get("wynika_z_tabela")?,
            wynika_z_odnosnik: row.get("wynika_z_odnosnik")?,
            dotyczy_tabela: row.get("dotyczy_tabela")?,
            dotyczy_odnosnik: row.get("dotyczy_odnosnik")?,
            pewnosc: row.get("pewnosc")?,
            sposob_sprawdzenia: row.get("sposob_sprawdzenia")?,
            ustalono: row.get("ustalono")?,
            wykorzystano_tabela: row.get("wykorzystano_tabela")?,
            wykorzystano_odnosnik: row.get("wykorzystano_odnosnik")?,
            nieaktualne_od: row.get("nieaktualne_od")?,
            zastapione_przez_id: row.get("zastapione_przez_id")?,
        })
    }
}

/// Aktualne ustalenia agenta, najnowsze pierwsze.
pub fn ustalenia(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<UstaleniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM stanowisko_ustalenia_wpisy
WHERE nieaktualne_od IS NULL
ORDER BY ustalono DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(UstaleniaRow::from_row).collect()
}

/// The parameters of `ustalenia_od_zrodla`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UstaleniaOdZrodlaParams {
    pub zrodlo_id: String,
}

/// A row of `ustalenia_od_zrodla`.
#[derive(Debug, Clone, PartialEq)]
pub struct UstaleniaOdZrodlaRow {
    pub id: String,
    pub tresc: String,
    pub zrodlo_id: Option<String>,
    pub wynika_z_tabela: Option<String>,
    pub wynika_z_odnosnik: Option<String>,
    pub dotyczy_tabela: Option<String>,
    pub dotyczy_odnosnik: Option<String>,
    pub pewnosc: WpisyPewnosc,
    pub sposob_sprawdzenia: Option<String>,
    pub ustalono: String,
    pub wykorzystano_tabela: Option<String>,
    pub wykorzystano_odnosnik: Option<String>,
    pub nieaktualne_od: Option<String>,
    pub zastapione_przez_id: Option<String>,
}

impl UstaleniaOdZrodlaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(UstaleniaOdZrodlaRow {
            id: row.get("id")?,
            tresc: row.get("tresc")?,
            zrodlo_id: row.get("zrodlo_id")?,
            wynika_z_tabela: row.get("wynika_z_tabela")?,
            wynika_z_odnosnik: row.get("wynika_z_odnosnik")?,
            dotyczy_tabela: row.get("dotyczy_tabela")?,
            dotyczy_odnosnik: row.get("dotyczy_odnosnik")?,
            pewnosc: row.get("pewnosc")?,
            sposob_sprawdzenia: row.get("sposob_sprawdzenia")?,
            ustalono: row.get("ustalono")?,
            wykorzystano_tabela: row.get("wykorzystano_tabela")?,
            wykorzystano_odnosnik: row.get("wykorzystano_odnosnik")?,
            nieaktualne_od: row.get("nieaktualne_od")?,
            zastapione_przez_id: row.get("zastapione_przez_id")?,
        })
    }
}

/// Aktualne ustalenia, które stoją na jednym źródle; po nieprawdzie tego źródła agent ocenia je na
/// nowo i oznacza jako niepewne to, czego nie potwierdził niezależnie.
pub fn ustalenia_od_zrodla(
    room: &impl xmeta_room::MetaRoom,
    params: &UstaleniaOdZrodlaParams,
) -> Result<Vec<UstaleniaOdZrodlaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM stanowisko_ustalenia_wpisy
WHERE zrodlo_id = :zrodlo_id
  AND nieaktualne_od IS NULL
ORDER BY ustalono, id"#,
        parameters: Vec::from([
            ("zrodlo_id", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(UstaleniaOdZrodlaRow::from_row).collect()
}

/// The parameters of `ustalenia_z_wiersza`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UstaleniaZWierszaParams {
    pub wynika_z_odnosnik: String,
}

/// A row of `ustalenia_z_wiersza`.
#[derive(Debug, Clone, PartialEq)]
pub struct UstaleniaZWierszaRow {
    pub id: String,
    pub tresc: String,
    pub zrodlo_id: Option<String>,
    pub wynika_z_tabela: Option<String>,
    pub wynika_z_odnosnik: Option<String>,
    pub dotyczy_tabela: Option<String>,
    pub dotyczy_odnosnik: Option<String>,
    pub pewnosc: WpisyPewnosc,
    pub sposob_sprawdzenia: Option<String>,
    pub ustalono: String,
    pub wykorzystano_tabela: Option<String>,
    pub wykorzystano_odnosnik: Option<String>,
    pub nieaktualne_od: Option<String>,
    pub zastapione_przez_id: Option<String>,
}

impl UstaleniaZWierszaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(UstaleniaZWierszaRow {
            id: row.get("id")?,
            tresc: row.get("tresc")?,
            zrodlo_id: row.get("zrodlo_id")?,
            wynika_z_tabela: row.get("wynika_z_tabela")?,
            wynika_z_odnosnik: row.get("wynika_z_odnosnik")?,
            dotyczy_tabela: row.get("dotyczy_tabela")?,
            dotyczy_odnosnik: row.get("dotyczy_odnosnik")?,
            pewnosc: row.get("pewnosc")?,
            sposob_sprawdzenia: row.get("sposob_sprawdzenia")?,
            ustalono: row.get("ustalono")?,
            wykorzystano_tabela: row.get("wykorzystano_tabela")?,
            wykorzystano_odnosnik: row.get("wykorzystano_odnosnik")?,
            nieaktualne_od: row.get("nieaktualne_od")?,
            zastapione_przez_id: row.get("zastapione_przez_id")?,
        })
    }
}

/// Aktualne ustalenia, które wynikają z jednego wiersza teczki; po zmianie dokumentu stają się
/// nieaktualne.
pub fn ustalenia_z_wiersza(
    room: &impl xmeta_room::MetaRoom,
    params: &UstaleniaZWierszaParams,
) -> Result<Vec<UstaleniaZWierszaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM stanowisko_ustalenia_wpisy
WHERE wynika_z_odnosnik = :wynika_z_odnosnik
  AND nieaktualne_od IS NULL
ORDER BY ustalono, id"#,
        parameters: Vec::from([
            (
                "wynika_z_odnosnik",
                xmeta_room::ToSqlValue::to_sql_value(&params.wynika_z_odnosnik),
            ),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(UstaleniaZWierszaRow::from_row).collect()
}

/// The parameters of `ustalenia_o`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UstaleniaOParams {
    pub dotyczy_odnosnik: String,
}

/// A row of `ustalenia_o`.
#[derive(Debug, Clone, PartialEq)]
pub struct UstaleniaORow {
    pub id: String,
    pub tresc: String,
    pub zrodlo_id: Option<String>,
    pub wynika_z_tabela: Option<String>,
    pub wynika_z_odnosnik: Option<String>,
    pub dotyczy_tabela: Option<String>,
    pub dotyczy_odnosnik: Option<String>,
    pub pewnosc: WpisyPewnosc,
    pub sposob_sprawdzenia: Option<String>,
    pub ustalono: String,
    pub wykorzystano_tabela: Option<String>,
    pub wykorzystano_odnosnik: Option<String>,
    pub nieaktualne_od: Option<String>,
    pub zastapione_przez_id: Option<String>,
}

impl UstaleniaORow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(UstaleniaORow {
            id: row.get("id")?,
            tresc: row.get("tresc")?,
            zrodlo_id: row.get("zrodlo_id")?,
            wynika_z_tabela: row.get("wynika_z_tabela")?,
            wynika_z_odnosnik: row.get("wynika_z_odnosnik")?,
            dotyczy_tabela: row.get("dotyczy_tabela")?,
            dotyczy_odnosnik: row.get("dotyczy_odnosnik")?,
            pewnosc: row.get("pewnosc")?,
            sposob_sprawdzenia: row.get("sposob_sprawdzenia")?,
            ustalono: row.get("ustalono")?,
            wykorzystano_tabela: row.get("wykorzystano_tabela")?,
            wykorzystano_odnosnik: row.get("wykorzystano_odnosnik")?,
            nieaktualne_od: row.get("nieaktualne_od")?,
            zastapione_przez_id: row.get("zastapione_przez_id")?,
        })
    }
}

/// Aktualne ustalenia o jednej rzeczy, np. o lokalu, propozycji albo pobycie.
pub fn ustalenia_o(
    room: &impl xmeta_room::MetaRoom,
    params: &UstaleniaOParams,
) -> Result<Vec<UstaleniaORow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM stanowisko_ustalenia_wpisy
WHERE dotyczy_odnosnik = :dotyczy_odnosnik
  AND nieaktualne_od IS NULL
ORDER BY ustalono, id"#,
        parameters: Vec::from([
            (
                "dotyczy_odnosnik",
                xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_odnosnik),
            ),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(UstaleniaORow::from_row).collect()
}

/// The parameters of `zapisz_ustalenie`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZapiszUstalenieParams {
    pub id: String,
    pub tresc: String,
    pub zrodlo_id: Option<String>,
    pub wynika_z_tabela: Option<String>,
    pub wynika_z_odnosnik: Option<String>,
    pub dotyczy_tabela: Option<String>,
    pub dotyczy_odnosnik: Option<String>,
    pub pewnosc: WpisyPewnosc,
    pub sposob_sprawdzenia: Option<String>,
    pub ustalono: String,
}

/// Nowe ustalenie ze źródłem albo wierszem teczki; identyfikator (UUID) nadaje aplikacja, żeby
/// nowsze ustalenie mogło go od razu wskazać.
pub fn zapisz_ustalenie(
    room: &impl xmeta_room::MetaRoom,
    params: &ZapiszUstalenieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO stanowisko_ustalenia_wpisy (id, tresc, zrodlo_id, wynika_z_tabela, wynika_z_odnosnik,
  dotyczy_tabela, dotyczy_odnosnik, pewnosc, sposob_sprawdzenia, ustalono)
VALUES (:id, :tresc, :zrodlo_id, :wynika_z_tabela, :wynika_z_odnosnik, :dotyczy_tabela,
  :dotyczy_odnosnik, :pewnosc, :sposob_sprawdzenia, :ustalono)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                    (
                        "zrodlo_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_id),
                    ),
                    (
                        "wynika_z_tabela",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wynika_z_tabela),
                    ),
                    (
                        "wynika_z_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wynika_z_odnosnik),
                    ),
                    (
                        "dotyczy_tabela",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_tabela),
                    ),
                    (
                        "dotyczy_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_odnosnik),
                    ),
                    ("pewnosc", xmeta_room::ToSqlValue::to_sql_value(&params.pewnosc)),
                    (
                        "sposob_sprawdzenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sposob_sprawdzenia),
                    ),
                    ("ustalono", xmeta_room::ToSqlValue::to_sql_value(&params.ustalono)),
                ]),
            },
        ],
    )
}

/// The parameters of `ocen_ustalenie_na_nowo`.
#[derive(Debug, Clone, PartialEq)]
pub struct OcenUstalenieNaNowoParams {
    pub pewnosc: WpisyPewnosc,
    pub sposob_sprawdzenia: Option<String>,
    pub id: String,
}

/// Nowa pewność ustalenia, np. po nieprawdzie źródła; potwierdzenie wymaga sposobu sprawdzenia.
pub fn ocen_ustalenie_na_nowo(
    room: &impl xmeta_room::MetaRoom,
    params: &OcenUstalenieNaNowoParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE stanowisko_ustalenia_wpisy
SET pewnosc = :pewnosc,
  sposob_sprawdzenia = :sposob_sprawdzenia
WHERE id = :id"#,
                parameters: Vec::from([
                    ("pewnosc", xmeta_room::ToSqlValue::to_sql_value(&params.pewnosc)),
                    (
                        "sposob_sprawdzenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sposob_sprawdzenia),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_wykorzystanie_ustalenia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujWykorzystanieUstaleniaParams {
    pub wykorzystano_tabela: String,
    pub wykorzystano_odnosnik: String,
    pub id: String,
}

/// Ustalenie weszło do wiersza teczki albo raportu.
pub fn odnotuj_wykorzystanie_ustalenia(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWykorzystanieUstaleniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE stanowisko_ustalenia_wpisy
SET wykorzystano_tabela = :wykorzystano_tabela,
  wykorzystano_odnosnik = :wykorzystano_odnosnik
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "wykorzystano_tabela",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wykorzystano_tabela),
                    ),
                    (
                        "wykorzystano_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.wykorzystano_odnosnik,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `uniewaznij_ustalenie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UniewaznijUstalenieParams {
    pub nieaktualne_od: String,
    pub zastapione_przez_id: Option<String>,
    pub id: String,
}

/// Ustalenie przestało być aktualne, np. bo dokument się zmienił; nowsze ustalenie, jeśli jest,
/// wskazuje się jako zastępujące.
pub fn uniewaznij_ustalenie(
    room: &impl xmeta_room::MetaRoom,
    params: &UniewaznijUstalenieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE stanowisko_ustalenia_wpisy
SET nieaktualne_od = :nieaktualne_od,
  zastapione_przez_id = :zastapione_przez_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "nieaktualne_od",
                        xmeta_room::ToSqlValue::to_sql_value(&params.nieaktualne_od),
                    ),
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
