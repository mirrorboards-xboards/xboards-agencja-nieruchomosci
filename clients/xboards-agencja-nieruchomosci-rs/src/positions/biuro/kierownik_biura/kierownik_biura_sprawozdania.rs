//! The kind `kierownik-biura-sprawozdania` of the room `positions/biuro/kierownik_biura`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "kierownik-biura-sprawozdania",
    requires: &[],
    migrations: &[
        r#"-- Sprawozdania dla właściciela biura, robocze i przekazane. Tekst leży w repozytorium XGit pokoju
-- pamięci; wiersz trzyma okres, ścieżkę i skrót. Przekazanego sprawozdania się nie zmienia, bo
-- właściciel biura opiera na nim decyzje; poprawka to nowe sprawozdanie.
CREATE TABLE kierownik_biura_sprawozdania_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_biura_sprawozdania_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Miesięczne, na żądanie właściciela biura albo od razu, np. po przekroczeniu limitu spraw.
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('miesieczne', 'na_zadanie', 'natychmiastowe')),
  okres_od TEXT NOT NULL,
  okres_do TEXT NOT NULL,
  sciezka TEXT NOT NULL CHECK (sciezka <> ''),
  skrot TEXT NOT NULL CHECK (skrot <> ''),
  sporzadzono TEXT NOT NULL,
  przekazano TEXT,

  CONSTRAINT okres_od_nie_po_do CHECK (okres_od <= okres_do),
  CONSTRAINT przekazanie_nie_przed_sporzadzeniem CHECK (przekazano IS NULL OR przekazano >= sporzadzono)
) STRICT;

CREATE TRIGGER kierownik_biura_sprawozdania_powstaje_robocze
BEFORE INSERT ON kierownik_biura_sprawozdania_wpisy
WHEN NEW.przekazano IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'sprawozdanie powstaje robocze; przekazanie odnotowuje się później');
END;

CREATE TRIGGER kierownik_biura_sprawozdania_przekazane_bez_zmian
BEFORE UPDATE ON kierownik_biura_sprawozdania_wpisy
WHEN OLD.przekazano IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.rodzaj IS NOT OLD.rodzaj
BEGIN
  SELECT RAISE(ABORT, 'przekazanego sprawozdania się nie zmienia; poprawka to nowe sprawozdanie');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_biura_sprawozdania_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_biura_sprawozdania_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_biura_sprawozdania_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: przekazane sprawozdania zostają.
CREATE TRIGGER kierownik_biura_sprawozdania_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_biura_sprawozdania_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
    ],
};

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    Miesieczne,
    NaZadanie,
    Natychmiastowe,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::Miesieczne,
        WpisyRodzaj::NaZadanie,
        WpisyRodzaj::Natychmiastowe,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::Miesieczne => "miesieczne",
            WpisyRodzaj::NaZadanie => "na_zadanie",
            WpisyRodzaj::Natychmiastowe => "natychmiastowe",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "miesieczne" => Ok(WpisyRodzaj::Miesieczne),
                    "na_zadanie" => Ok(WpisyRodzaj::NaZadanie),
                    "natychmiastowe" => Ok(WpisyRodzaj::Natychmiastowe),
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

/// A row of `sprawozdania`.
#[derive(Debug, Clone, PartialEq)]
pub struct SprawozdaniaRow {
    pub id: i64,
    pub rodzaj: WpisyRodzaj,
    pub okres_od: String,
    pub okres_do: String,
    pub sciezka: String,
    pub skrot: String,
    pub sporzadzono: String,
    pub przekazano: Option<String>,
}

impl SprawozdaniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(SprawozdaniaRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            okres_od: row.get("okres_od")?,
            okres_do: row.get("okres_do")?,
            sciezka: row.get("sciezka")?,
            skrot: row.get("skrot")?,
            sporzadzono: row.get("sporzadzono")?,
            przekazano: row.get("przekazano")?,
        })
    }
}

/// Wszystkie sprawozdania, najnowsze pierwsze; robocze mają puste przekazanie.
pub fn sprawozdania(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<SprawozdaniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_biura_sprawozdania_wpisy
ORDER BY sporzadzono DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(SprawozdaniaRow::from_row).collect()
}

/// The parameters of `zapisz_sprawozdanie`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZapiszSprawozdanieParams {
    pub rodzaj: WpisyRodzaj,
    pub okres_od: String,
    pub okres_do: String,
    pub sciezka: String,
    pub skrot: String,
    pub sporzadzono: String,
}

/// Nowe sprawozdanie robocze.
pub fn zapisz_sprawozdanie(
    room: &impl xmeta_room::MetaRoom,
    params: &ZapiszSprawozdanieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_biura_sprawozdania_wpisy (rodzaj, okres_od, okres_do, sciezka, skrot,
  sporzadzono)
VALUES (:rodzaj, :okres_od, :okres_do, :sciezka, :skrot, :sporzadzono)"#,
                parameters: Vec::from([
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("okres_od", xmeta_room::ToSqlValue::to_sql_value(&params.okres_od)),
                    ("okres_do", xmeta_room::ToSqlValue::to_sql_value(&params.okres_do)),
                    ("sciezka", xmeta_room::ToSqlValue::to_sql_value(&params.sciezka)),
                    ("skrot", xmeta_room::ToSqlValue::to_sql_value(&params.skrot)),
                    (
                        "sporzadzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sporzadzono),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_sprawozdanie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZmienSprawozdanieParams {
    pub okres_od: String,
    pub okres_do: String,
    pub sciezka: String,
    pub skrot: String,
    pub sporzadzono: String,
    pub id: i64,
}

/// Nowa wersja roboczego sprawozdania w XGit.
pub fn zmien_sprawozdanie(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienSprawozdanieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_biura_sprawozdania_wpisy
SET okres_od = :okres_od,
  okres_do = :okres_do,
  sciezka = :sciezka,
  skrot = :skrot,
  sporzadzono = :sporzadzono
WHERE id = :id"#,
                parameters: Vec::from([
                    ("okres_od", xmeta_room::ToSqlValue::to_sql_value(&params.okres_od)),
                    ("okres_do", xmeta_room::ToSqlValue::to_sql_value(&params.okres_do)),
                    ("sciezka", xmeta_room::ToSqlValue::to_sql_value(&params.sciezka)),
                    ("skrot", xmeta_room::ToSqlValue::to_sql_value(&params.skrot)),
                    (
                        "sporzadzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sporzadzono),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `przekaz_sprawozdanie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrzekazSprawozdanieParams {
    pub przekazano: String,
    pub id: i64,
}

/// Sprawozdanie przekazane właścicielowi biura.
pub fn przekaz_sprawozdanie(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzekazSprawozdanieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_biura_sprawozdania_wpisy
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
