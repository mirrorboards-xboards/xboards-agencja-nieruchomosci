//! The kind `sprawa-powiazania` of the room `cases/sprawa`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "sprawa-powiazania";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Odnośniki do innych teczek i pokoi, np. z teczki transakcji do teczki oferty sprzedającego albo
-- do pokoju wymiany z kancelarią. Inny pokój wskazuje się tekstem, bo klucz obcy działa tylko
-- w obrębie jednej bazy. Odnośnik nie daje dostępu do tamtej teczki (R1).
--
-- Wpisuje je agent albo kierownik, który zakłada związek. Powiązanie jest zapisem tego, kto
-- i kiedy połączył sprawy, więc się go nie zmienia.
CREATE TABLE sprawa_powiazania_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. sprawa_powiazania_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('sprawa_zrodlowa', 'sprawa_wynikowa', 'poprzednia_sprawa_klienta', 'wspolna_umowa', 'pokoj_wymiany', 'inna')),
  -- Typ i oznaczenie tamtej teczki, np. oferta i „Oferta 2026/041”; pokój wymiany ich nie ma.
  typ_teczki TEXT CHECK (typ_teczki <> '' AND typ_teczki NOT GLOB '*[^a-z_]*'),
  oznaczenie TEXT,
  -- Pokój i wiersz, tekstem.
  odnosnik TEXT NOT NULL,
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL
) STRICT;

CREATE TRIGGER sprawa_powiazania_bez_zmian
BEFORE UPDATE ON sprawa_powiazania_wpisy
BEGIN
  SELECT RAISE(ABORT, 'powiązania się nie zmienia; nowe powiązanie to nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER sprawa_powiazania_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_powiazania_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_powiazania_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_powiazania_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_powiazania_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
];

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    SprawaZrodlowa,
    SprawaWynikowa,
    PoprzedniaSprawaKlienta,
    WspolnaUmowa,
    PokojWymiany,
    Inna,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::SprawaZrodlowa,
        WpisyRodzaj::SprawaWynikowa,
        WpisyRodzaj::PoprzedniaSprawaKlienta,
        WpisyRodzaj::WspolnaUmowa,
        WpisyRodzaj::PokojWymiany,
        WpisyRodzaj::Inna,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::SprawaZrodlowa => "sprawa_zrodlowa",
            WpisyRodzaj::SprawaWynikowa => "sprawa_wynikowa",
            WpisyRodzaj::PoprzedniaSprawaKlienta => "poprzednia_sprawa_klienta",
            WpisyRodzaj::WspolnaUmowa => "wspolna_umowa",
            WpisyRodzaj::PokojWymiany => "pokoj_wymiany",
            WpisyRodzaj::Inna => "inna",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "sprawa_zrodlowa" => Ok(WpisyRodzaj::SprawaZrodlowa),
                    "sprawa_wynikowa" => Ok(WpisyRodzaj::SprawaWynikowa),
                    "poprzednia_sprawa_klienta" => {
                        Ok(WpisyRodzaj::PoprzedniaSprawaKlienta)
                    }
                    "wspolna_umowa" => Ok(WpisyRodzaj::WspolnaUmowa),
                    "pokoj_wymiany" => Ok(WpisyRodzaj::PokojWymiany),
                    "inna" => Ok(WpisyRodzaj::Inna),
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

/// A row of `powiazania`.
#[derive(Debug, Clone, PartialEq)]
pub struct PowiazaniaRow {
    pub id: i64,
    pub rodzaj: WpisyRodzaj,
    pub typ_teczki: Option<String>,
    pub oznaczenie: Option<String>,
    pub odnosnik: String,
    pub wpisal: String,
    pub wpisano: String,
}

impl PowiazaniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PowiazaniaRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            typ_teczki: row.get("typ_teczki")?,
            oznaczenie: row.get("oznaczenie")?,
            odnosnik: row.get("odnosnik")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Powiązania teczki w kolejności wpisania.
pub fn powiazania(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PowiazaniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_powiazania_wpisy
ORDER BY id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(PowiazaniaRow::from_row).collect()
}

/// The parameters of `wpisz_powiazanie`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszPowiazanieParams {
    pub rodzaj: WpisyRodzaj,
    pub typ_teczki: Option<String>,
    pub oznaczenie: Option<String>,
    pub odnosnik: String,
    pub wpisal: String,
    pub wpisano: String,
}

/// Odnośnik do innej teczki albo pokoju, z rodzajem związku.
pub fn wpisz_powiazanie(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszPowiazanieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_powiazania_wpisy (rodzaj, typ_teczki, oznaczenie, odnosnik, wpisal, wpisano)
VALUES (:rodzaj, :typ_teczki, :oznaczenie, :odnosnik, :wpisal, :wpisano)"#,
                parameters: Vec::from([
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    (
                        "typ_teczki",
                        xmeta_room::ToSqlValue::to_sql_value(&params.typ_teczki),
                    ),
                    (
                        "oznaczenie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.oznaczenie),
                    ),
                    ("odnosnik", xmeta_room::ToSqlValue::to_sql_value(&params.odnosnik)),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
        ],
    )
}
