//! The kind `transakcja-analizy` of the room `cases/transakcja`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "transakcja-analizy";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Analizy stanu prawnego i ryzyka transakcji: księga wieczysta, ograniczenia obrotu, projekty
-- i forma umów, przekazanie i przerwanie. Pisze Analityk stanu prawnego. Analiza ma plik z pełnym
-- wywodem, a w tabeli krótkie wnioski i ryzyka.
--
-- Analizy się nie zmienia: nowa analiza tego samego przedmiotu (i tej samej umowy) odbiera
-- poprzedniej aktualność. Wniosek, że transakcji nie da się przeprowadzić, wymaga zgody człowieka
-- (decyzja przerwanie_sprawy w rdzeniu); ryzyko trafia do teczki od razu.
CREATE TABLE transakcja_analizy_wpisy (
  id TEXT PRIMARY KEY,
  przedmiot TEXT NOT NULL CHECK (przedmiot IN ('ksiega', 'ograniczenia', 'projekt_umowy', 'forma_umowy', 'przekazanie', 'przerwanie')),
  -- Umowa, której dotyczy analiza projektu albo formy.
  umowa_id TEXT REFERENCES transakcja_umowy_wpisy (id),
  -- Analizowany dokument, np. odpis księgi albo projekt umowy, i przy księdze poprzedni odpis,
  -- z którym Analityk porównuje nowy tuż przed aktem.
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  poprzedni_odpis_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  sporzadzono TEXT NOT NULL,
  wnioski TEXT NOT NULL CHECK (wnioski <> ''),
  sciezka TEXT NOT NULL,
  skrot TEXT NOT NULL,
  aktualna INTEGER NOT NULL CHECK (aktualna IN (0, 1)),

  CONSTRAINT umowa_przy_analizie_umowy CHECK (
    (przedmiot IN ('projekt_umowy', 'forma_umowy')) = (umowa_id IS NOT NULL)
  ),
  CONSTRAINT projekt_umowy_wskazuje_dokument CHECK (przedmiot <> 'projekt_umowy' OR dokument_id IS NOT NULL),
  CONSTRAINT poprzedni_odpis_tylko_przy_ksiedze CHECK (poprzedni_odpis_dokument_id IS NULL OR przedmiot = 'ksiega')
) STRICT;

-- Ryzyka, które Analityk wskazał w analizie, ze skutkiem dla stron.
CREATE TABLE transakcja_analizy_ryzyka (
  -- Klucz nadany przez bazę jest dodatni; zob. transakcja_analizy_ryzyka_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  analiza_id TEXT NOT NULL REFERENCES transakcja_analizy_wpisy (id),
  opis TEXT NOT NULL CHECK (opis <> ''),
  skutek TEXT
) STRICT;

-- Aktualność odbiera tylko nowsza analiza, więc analiza powstaje aktualna.
CREATE TRIGGER transakcja_analizy_powstaje_aktualna
BEFORE INSERT ON transakcja_analizy_wpisy
WHEN NEW.aktualna <> 1
BEGIN
  SELECT RAISE(ABORT, 'analiza powstaje jako aktualna');
END;

CREATE TRIGGER transakcja_analizy_nowa_zastepuje_poprzednia
AFTER INSERT ON transakcja_analizy_wpisy
BEGIN
  UPDATE transakcja_analizy_wpisy
  SET aktualna = 0
  WHERE id <> NEW.id
    AND aktualna = 1
    AND przedmiot = NEW.przedmiot
    AND umowa_id IS NEW.umowa_id;
END;

-- Analiza jest zapisem tego, co Analityk ustalił; jedyną zmianą jest utrata aktualności.
CREATE TRIGGER transakcja_analizy_bez_zmian
BEFORE UPDATE ON transakcja_analizy_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.przedmiot IS NOT OLD.przedmiot
  OR NEW.umowa_id IS NOT OLD.umowa_id
  OR NEW.dokument_id IS NOT OLD.dokument_id
  OR NEW.poprzedni_odpis_dokument_id IS NOT OLD.poprzedni_odpis_dokument_id
  OR NEW.sporzadzono IS NOT OLD.sporzadzono
  OR NEW.wnioski IS NOT OLD.wnioski
  OR NEW.sciezka IS NOT OLD.sciezka
  OR NEW.skrot IS NOT OLD.skrot
  OR (OLD.aktualna = 0 AND NEW.aktualna = 1)
BEGIN
  SELECT RAISE(ABORT, 'analizy się nie zmienia; nowa analiza to nowy wpis');
END;

-- Ryzyka należą do bieżącej oceny: dopisuje się je do aktualnej analizy i się ich nie zmienia.
CREATE TRIGGER transakcja_analizy_ryzyko_aktualnej_analizy
BEFORE INSERT ON transakcja_analizy_ryzyka
WHEN NOT EXISTS (
  SELECT 1
  FROM transakcja_analizy_wpisy
  WHERE id = NEW.analiza_id
    AND aktualna = 1
)
BEGIN
  SELECT RAISE(ABORT, 'ryzyko dopisuje się do aktualnej analizy');
END;

CREATE TRIGGER transakcja_analizy_ryzyka_bez_zmian
BEFORE UPDATE ON transakcja_analizy_ryzyka
BEGIN
  SELECT RAISE(ABORT, 'ryzyka się nie zmienia; nowa ocena to nowa analiza');
END;

-- Każdy projekt umowy ocenia Analityk, zanim strony go podpiszą: umowa zostaje zawarta tylko
-- z aktualną analizą właśnie tego projektu. Nowy projekt (inny dokument) wymaga nowej analizy.
CREATE TRIGGER transakcja_analizy_zawarcie_po_analizie_projektu
BEFORE UPDATE OF stan ON transakcja_umowy_wpisy
WHEN NEW.stan = 'zawarta'
  AND OLD.stan = 'projekt'
  AND NOT EXISTS (
    SELECT 1
    FROM transakcja_analizy_wpisy
    WHERE przedmiot = 'projekt_umowy'
      AND umowa_id = NEW.id
      AND dokument_id = NEW.projekt_dokument_id
      AND aktualna = 1
  )
BEGIN
  SELECT RAISE(ABORT, 'umowa zostaje zawarta tylko z aktualną analizą swojego projektu');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER transakcja_analizy_wpisy_bez_zastepowania
BEFORE INSERT ON transakcja_analizy_wpisy
WHEN EXISTS (
  SELECT 1
  FROM transakcja_analizy_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER transakcja_analizy_wpisy_bez_usuwania
BEFORE DELETE ON transakcja_analizy_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER transakcja_analizy_ryzyka_bez_zastepowania
BEFORE INSERT ON transakcja_analizy_ryzyka
WHEN EXISTS (
  SELECT 1
  FROM transakcja_analizy_ryzyka
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER transakcja_analizy_ryzyka_bez_usuwania
BEFORE DELETE ON transakcja_analizy_ryzyka
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
];

/// The values of column `wpisy.przedmiot`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyPrzedmiot {
    Ksiega,
    Ograniczenia,
    ProjektUmowy,
    FormaUmowy,
    Przekazanie,
    Przerwanie,
}

impl WpisyPrzedmiot {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyPrzedmiot] = &[
        WpisyPrzedmiot::Ksiega,
        WpisyPrzedmiot::Ograniczenia,
        WpisyPrzedmiot::ProjektUmowy,
        WpisyPrzedmiot::FormaUmowy,
        WpisyPrzedmiot::Przekazanie,
        WpisyPrzedmiot::Przerwanie,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyPrzedmiot::Ksiega => "ksiega",
            WpisyPrzedmiot::Ograniczenia => "ograniczenia",
            WpisyPrzedmiot::ProjektUmowy => "projekt_umowy",
            WpisyPrzedmiot::FormaUmowy => "forma_umowy",
            WpisyPrzedmiot::Przekazanie => "przekazanie",
            WpisyPrzedmiot::Przerwanie => "przerwanie",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyPrzedmiot {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "ksiega" => Ok(WpisyPrzedmiot::Ksiega),
                    "ograniczenia" => Ok(WpisyPrzedmiot::Ograniczenia),
                    "projekt_umowy" => Ok(WpisyPrzedmiot::ProjektUmowy),
                    "forma_umowy" => Ok(WpisyPrzedmiot::FormaUmowy),
                    "przekazanie" => Ok(WpisyPrzedmiot::Przekazanie),
                    "przerwanie" => Ok(WpisyPrzedmiot::Przerwanie),
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

/// A row of `ryzyka_transakcji`.
#[derive(Debug, Clone, PartialEq)]
pub struct RyzykaTransakcjiRow {
    pub id: i64,
    pub analiza_id: String,
    pub opis: String,
    pub skutek: Option<String>,
}

impl RyzykaTransakcjiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(RyzykaTransakcjiRow {
            id: row.get("id")?,
            analiza_id: row.get("analiza_id")?,
            opis: row.get("opis")?,
            skutek: row.get("skutek")?,
        })
    }
}

/// Ryzyka wszystkich analiz w kolejności wpisania.
pub fn ryzyka_transakcji(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<RyzykaTransakcjiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM transakcja_analizy_ryzyka
ORDER BY analiza_id, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(RyzykaTransakcjiRow::from_row).collect()
}

/// The parameters of `dopisz_ryzyko_transakcji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DopiszRyzykoTransakcjiParams {
    pub analiza_id: String,
    pub opis: String,
    pub skutek: Option<String>,
}

/// Ryzyko z aktualnej analizy, ze skutkiem dla stron.
pub fn dopisz_ryzyko_transakcji(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszRyzykoTransakcjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO transakcja_analizy_ryzyka (analiza_id, opis, skutek)
VALUES (:analiza_id, :opis, :skutek)"#,
                parameters: Vec::from([
                    (
                        "analiza_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.analiza_id),
                    ),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    ("skutek", xmeta_room::ToSqlValue::to_sql_value(&params.skutek)),
                ]),
            },
        ],
    )
}

/// A row of `analizy_transakcji`.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalizyTransakcjiRow {
    pub id: String,
    pub przedmiot: WpisyPrzedmiot,
    pub umowa_id: Option<String>,
    pub dokument_id: Option<String>,
    pub poprzedni_odpis_dokument_id: Option<String>,
    pub sporzadzono: String,
    pub wnioski: String,
    pub sciezka: String,
    pub skrot: String,
    pub aktualna: bool,
}

impl AnalizyTransakcjiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(AnalizyTransakcjiRow {
            id: row.get("id")?,
            przedmiot: row.get("przedmiot")?,
            umowa_id: row.get("umowa_id")?,
            dokument_id: row.get("dokument_id")?,
            poprzedni_odpis_dokument_id: row.get("poprzedni_odpis_dokument_id")?,
            sporzadzono: row.get("sporzadzono")?,
            wnioski: row.get("wnioski")?,
            sciezka: row.get("sciezka")?,
            skrot: row.get("skrot")?,
            aktualna: row.get("aktualna")?,
        })
    }
}

/// Analizy stanu prawnego, najnowsze pierwsze.
pub fn analizy_transakcji(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<AnalizyTransakcjiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM transakcja_analizy_wpisy
ORDER BY sporzadzono DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(AnalizyTransakcjiRow::from_row).collect()
}

/// A row of `analizy_aktualne_transakcji`.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalizyAktualneTransakcjiRow {
    pub id: String,
    pub przedmiot: WpisyPrzedmiot,
    pub umowa_id: Option<String>,
    pub dokument_id: Option<String>,
    pub poprzedni_odpis_dokument_id: Option<String>,
    pub sporzadzono: String,
    pub wnioski: String,
    pub sciezka: String,
    pub skrot: String,
    pub aktualna: bool,
}

impl AnalizyAktualneTransakcjiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(AnalizyAktualneTransakcjiRow {
            id: row.get("id")?,
            przedmiot: row.get("przedmiot")?,
            umowa_id: row.get("umowa_id")?,
            dokument_id: row.get("dokument_id")?,
            poprzedni_odpis_dokument_id: row.get("poprzedni_odpis_dokument_id")?,
            sporzadzono: row.get("sporzadzono")?,
            wnioski: row.get("wnioski")?,
            sciezka: row.get("sciezka")?,
            skrot: row.get("skrot")?,
            aktualna: row.get("aktualna")?,
        })
    }
}

/// Bieżąca ocena: aktualna analiza każdego przedmiotu.
pub fn analizy_aktualne_transakcji(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<AnalizyAktualneTransakcjiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM transakcja_analizy_wpisy
WHERE aktualna = 1
ORDER BY przedmiot, umowa_id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(AnalizyAktualneTransakcjiRow::from_row).collect()
}

/// The parameters of `wpisz_analize_transakcji`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszAnalizeTransakcjiParams {
    pub id: String,
    pub przedmiot: WpisyPrzedmiot,
    pub umowa_id: Option<String>,
    pub dokument_id: Option<String>,
    pub poprzedni_odpis_dokument_id: Option<String>,
    pub sporzadzono: String,
    pub wnioski: String,
    pub sciezka: String,
    pub skrot: String,
}

/// Nowa analiza z wnioskami i plikiem; poprzednia analiza tego przedmiotu traci aktualność.
pub fn wpisz_analize_transakcji(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszAnalizeTransakcjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO transakcja_analizy_wpisy (id, przedmiot, umowa_id, dokument_id,
  poprzedni_odpis_dokument_id, sporzadzono, wnioski, sciezka, skrot, aktualna)
VALUES (:id, :przedmiot, :umowa_id, :dokument_id, :poprzedni_odpis_dokument_id, :sporzadzono,
  :wnioski, :sciezka, :skrot, 1)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "przedmiot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przedmiot),
                    ),
                    ("umowa_id", xmeta_room::ToSqlValue::to_sql_value(&params.umowa_id)),
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                    (
                        "poprzedni_odpis_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.poprzedni_odpis_dokument_id,
                        ),
                    ),
                    (
                        "sporzadzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sporzadzono),
                    ),
                    ("wnioski", xmeta_room::ToSqlValue::to_sql_value(&params.wnioski)),
                    ("sciezka", xmeta_room::ToSqlValue::to_sql_value(&params.sciezka)),
                    ("skrot", xmeta_room::ToSqlValue::to_sql_value(&params.skrot)),
                ]),
            },
        ],
    )
}
