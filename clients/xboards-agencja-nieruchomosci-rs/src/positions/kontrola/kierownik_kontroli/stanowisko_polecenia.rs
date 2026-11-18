//! The kind `stanowisko-polecenia` of the room `positions/kontrola/kierownik_kontroli`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "stanowisko-polecenia",
    requires: &[],
    migrations: &[
        r#"-- Polecenia: co agentowi polecił właściciel albo ktoś, komu właściciel na to pozwolił (np. wspólnik
-- u Kierownika biura), zapisane osobno od ustaleń.
--
-- Agent przyjmuje tylko polecenie podpisane kluczem wydającego, bo niepodpisana wiadomość może
-- udawać właściciela. Podpis sprawdza aplikacja, a baza wymaga jego obecności.
CREATE TABLE stanowisko_polecenia_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. stanowisko_polecenia_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  wydal TEXT NOT NULL,
  wydal_jako TEXT NOT NULL CHECK (wydal_jako IN ('wlasciciel', 'upowazniony')),
  otrzymano TEXT NOT NULL,
  tresc TEXT NOT NULL,
  podpis TEXT NOT NULL,
  stan TEXT NOT NULL CHECK (stan IN ('przyjete', 'wykonane', 'odmowione')),
  zamknieto TEXT,
  uwagi TEXT,

  CONSTRAINT zamkniete_z_data CHECK ((stan = 'przyjete') = (zamknieto IS NULL)),
  CONSTRAINT odmowa_z_uwagami CHECK (stan <> 'odmowione' OR uwagi IS NOT NULL)
) STRICT;

CREATE TRIGGER stanowisko_polecenia_powstaje_przyjete
BEFORE INSERT ON stanowisko_polecenia_wpisy
WHEN NEW.stan <> 'przyjete'
BEGIN
  SELECT RAISE(ABORT, 'polecenie powstaje jako przyjęte');
END;

-- Podpis obejmuje treść, więc zmieniona treść nie byłaby już poleceniem wydającego.
CREATE TRIGGER stanowisko_polecenia_tresc_niezmienna
BEFORE UPDATE ON stanowisko_polecenia_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.wydal IS NOT OLD.wydal
  OR NEW.wydal_jako IS NOT OLD.wydal_jako
  OR NEW.otrzymano IS NOT OLD.otrzymano
  OR NEW.tresc IS NOT OLD.tresc
  OR NEW.podpis IS NOT OLD.podpis
BEGIN
  SELECT RAISE(ABORT, 'treści ani podpisu polecenia się nie zmienia');
END;

CREATE TRIGGER stanowisko_polecenia_stan_tylko_naprzod
BEFORE UPDATE ON stanowisko_polecenia_wpisy
WHEN OLD.stan <> 'przyjete'
BEGIN
  SELECT RAISE(ABORT, 'wykonanego ani odmówionego polecenia się nie zmienia');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER stanowisko_polecenia_wpisy_bez_zastepowania
BEFORE INSERT ON stanowisko_polecenia_wpisy
WHEN EXISTS (
  SELECT 1
  FROM stanowisko_polecenia_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: polecenie zostaje jako ślad tego, co agentowi kazano.
CREATE TRIGGER stanowisko_polecenia_wpisy_bez_usuwania
BEFORE DELETE ON stanowisko_polecenia_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
    ],
};

/// The values of column `wpisy.wydal_jako`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyWydalJako {
    Wlasciciel,
    Upowazniony,
}

impl WpisyWydalJako {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyWydalJako] = &[
        WpisyWydalJako::Wlasciciel,
        WpisyWydalJako::Upowazniony,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyWydalJako::Wlasciciel => "wlasciciel",
            WpisyWydalJako::Upowazniony => "upowazniony",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyWydalJako {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "wlasciciel" => Ok(WpisyWydalJako::Wlasciciel),
                    "upowazniony" => Ok(WpisyWydalJako::Upowazniony),
                    other => Err(format!("{other} is no value of WpisyWydalJako")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyWydalJako {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyStan {
    Przyjete,
    Wykonane,
    Odmowione,
}

impl WpisyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyStan] = &[
        WpisyStan::Przyjete,
        WpisyStan::Wykonane,
        WpisyStan::Odmowione,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyStan::Przyjete => "przyjete",
            WpisyStan::Wykonane => "wykonane",
            WpisyStan::Odmowione => "odmowione",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "przyjete" => Ok(WpisyStan::Przyjete),
                    "wykonane" => Ok(WpisyStan::Wykonane),
                    "odmowione" => Ok(WpisyStan::Odmowione),
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

/// A row of `polecenia`.
#[derive(Debug, Clone, PartialEq)]
pub struct PoleceniaRow {
    pub id: i64,
    pub wydal: String,
    pub wydal_jako: WpisyWydalJako,
    pub otrzymano: String,
    pub tresc: String,
    pub podpis: String,
    pub stan: WpisyStan,
    pub zamknieto: Option<String>,
    pub uwagi: Option<String>,
}

impl PoleceniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PoleceniaRow {
            id: row.get("id")?,
            wydal: row.get("wydal")?,
            wydal_jako: row.get("wydal_jako")?,
            otrzymano: row.get("otrzymano")?,
            tresc: row.get("tresc")?,
            podpis: row.get("podpis")?,
            stan: row.get("stan")?,
            zamknieto: row.get("zamknieto")?,
            uwagi: row.get("uwagi")?,
        })
    }
}

/// Wszystkie polecenia agenta, najnowsze pierwsze.
pub fn polecenia(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PoleceniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM stanowisko_polecenia_wpisy
ORDER BY otrzymano DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(PoleceniaRow::from_row).collect()
}

/// A row of `polecenia_otwarte`.
#[derive(Debug, Clone, PartialEq)]
pub struct PoleceniaOtwarteRow {
    pub id: i64,
    pub wydal: String,
    pub wydal_jako: WpisyWydalJako,
    pub otrzymano: String,
    pub tresc: String,
    pub podpis: String,
    pub stan: WpisyStan,
    pub zamknieto: Option<String>,
    pub uwagi: Option<String>,
}

impl PoleceniaOtwarteRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PoleceniaOtwarteRow {
            id: row.get("id")?,
            wydal: row.get("wydal")?,
            wydal_jako: row.get("wydal_jako")?,
            otrzymano: row.get("otrzymano")?,
            tresc: row.get("tresc")?,
            podpis: row.get("podpis")?,
            stan: row.get("stan")?,
            zamknieto: row.get("zamknieto")?,
            uwagi: row.get("uwagi")?,
        })
    }
}

/// Polecenia przyjęte, ale jeszcze niewykonane, od najstarszego.
pub fn polecenia_otwarte(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PoleceniaOtwarteRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM stanowisko_polecenia_wpisy
WHERE stan = 'przyjete'
ORDER BY otrzymano, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(PoleceniaOtwarteRow::from_row).collect()
}

/// The parameters of `przyjmij_polecenie`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzyjmijPolecenieParams {
    pub wydal: String,
    pub wydal_jako: WpisyWydalJako,
    pub otrzymano: String,
    pub tresc: String,
    pub podpis: String,
}

/// Podpisane polecenie właściciela albo upoważnionego przez niego człowieka; podpis sprawdziła
/// aplikacja.
pub fn przyjmij_polecenie(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzyjmijPolecenieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO stanowisko_polecenia_wpisy (wydal, wydal_jako, otrzymano, tresc, podpis, stan)
VALUES (:wydal, :wydal_jako, :otrzymano, :tresc, :podpis, 'przyjete')"#,
                parameters: Vec::from([
                    ("wydal", xmeta_room::ToSqlValue::to_sql_value(&params.wydal)),
                    (
                        "wydal_jako",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wydal_jako),
                    ),
                    (
                        "otrzymano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.otrzymano),
                    ),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                    ("podpis", xmeta_room::ToSqlValue::to_sql_value(&params.podpis)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_wykonanie_polecenia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujWykonaniePoleceniaParams {
    pub zamknieto: String,
    pub uwagi: Option<String>,
    pub id: i64,
}

/// Polecenie wykonane.
pub fn odnotuj_wykonanie_polecenia(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWykonaniePoleceniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE stanowisko_polecenia_wpisy
SET stan = 'wykonane',
  zamknieto = :zamknieto,
  uwagi = :uwagi
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zamknieto",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zamknieto),
                    ),
                    ("uwagi", xmeta_room::ToSqlValue::to_sql_value(&params.uwagi)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odmow_polecenia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdmowPoleceniaParams {
    pub zamknieto: String,
    pub uwagi: String,
    pub id: i64,
}

/// Agent odmawia polecenia, np. sprzecznego z normą stanowiska, i pisze dlaczego.
pub fn odmow_polecenia(
    room: &impl xmeta_room::MetaRoom,
    params: &OdmowPoleceniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE stanowisko_polecenia_wpisy
SET stan = 'odmowione',
  zamknieto = :zamknieto,
  uwagi = :uwagi
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zamknieto",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zamknieto),
                    ),
                    ("uwagi", xmeta_room::ToSqlValue::to_sql_value(&params.uwagi)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
