//! The kind `sprawa-zgody` of the room `cases/najem`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "sprawa-zgody";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Zgody i oświadczenia klientów i innych osób spoza biura, np. zatwierdzenie ceny oferty albo
-- żądanie konsumenta, żeby biuro zaczęło pracę przed upływem terminu na odstąpienie. Dowodem jest
-- dokument albo wiadomość w teczce, a nie klucz. Zgody ustnej się nie wpisuje: jest ustaleniem
-- w pamięci agenta, dopóki klient jej nie potwierdzi.
--
-- Rodzaj zgody ustala typ sprawy tekstem. Rdzeń zna zadanie_rozpoczecia_pracy i odstapienie;
-- ich reguły przy umowie z biurem leżą w migracji sprawa-umowy. Zgody wpisuje agent sprawy.
CREATE TABLE sprawa_zgody_wpisy (
  id TEXT PRIMARY KEY,
  uczestnik_id TEXT NOT NULL REFERENCES sprawa_uczestnicy_wpisy (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj <> '' AND rodzaj NOT GLOB '*[^a-z_]*'),
  tresc TEXT NOT NULL,
  zlozono TEXT NOT NULL,
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  wplyw_id TEXT REFERENCES sprawa_korespondencja_wplywy (id),
  -- Oświadczenie na trwałym nośniku (art. 15 ust. 3 upk), np. podpisany plik albo wiadomość
  -- poczty, którą klient zachowuje.
  trwaly_nosnik INTEGER NOT NULL CHECK (trwaly_nosnik IN (0, 1)),
  -- Wiersz, którego zgoda dotyczy, np. cena w typie oferty; bez klucza obcego.
  dotyczy_tabela TEXT,
  dotyczy_id TEXT,
  -- Wcześniejsza zgoda, którą ta zastępuje.
  zastepuje_id TEXT REFERENCES sprawa_zgody_wpisy (id),

  CONSTRAINT zgoda_ma_dokument_albo_wiadomosc CHECK (dokument_id IS NOT NULL OR wplyw_id IS NOT NULL),
  CONSTRAINT dotyczy_tabela_razem_z_id CHECK ((dotyczy_tabela IS NULL) = (dotyczy_id IS NULL)),
  CONSTRAINT nie_zastepuje_samej_siebie CHECK (zastepuje_id IS NOT id)
) STRICT;

-- Dowodem zgody jest plik: dokument bez pliku (wymagany albo zamówiony) niczego nie dowodzi.
CREATE TRIGGER sprawa_zgody_dokument_z_plikiem
BEFORE INSERT ON sprawa_zgody_wpisy
WHEN NEW.dokument_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_dokumenty_wpisy
    WHERE id = NEW.dokument_id
      AND stan IN ('otrzymany', 'sprawdzony')
  )
BEGIN
  SELECT RAISE(ABORT, 'zgoda wskazuje dokument z plikiem: otrzymany albo sprawdzony');
END;

-- Dokument, który dowodzi zgody, nie zmienia rodzaju ani tego, czego dotyczy, i nie staje się
-- odrzucony ani zbędny; inaczej zgoda straciłaby dowód.
CREATE TRIGGER sprawa_zgody_dokument_zgody_zostaje
BEFORE UPDATE ON sprawa_dokumenty_wpisy
WHEN (
    NEW.rodzaj IS NOT OLD.rodzaj
    OR NEW.dotyczy_uczestnik_id IS NOT OLD.dotyczy_uczestnik_id
    OR NEW.dotyczy_nieruchomosc_id IS NOT OLD.dotyczy_nieruchomosc_id
    OR (NEW.stan IN ('odrzucony', 'zbedny') AND NEW.stan IS NOT OLD.stan)
  )
  AND EXISTS (
    SELECT 1
    FROM sprawa_zgody_wpisy
    WHERE dokument_id = OLD.id
  )
BEGIN
  SELECT RAISE(ABORT, 'dokument, który dowodzi zgody, nie zmienia rodzaju ani tego, czego dotyczy, i nie jest odrzucany');
END;

CREATE TRIGGER sprawa_zgody_bez_zmian
BEFORE UPDATE ON sprawa_zgody_wpisy
BEGIN
  SELECT RAISE(ABORT, 'zgody się nie zmienia; nowa zgoda zastępuje poprzednią');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_zgody_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_zgody_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_zgody_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_zgody_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_zgody_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
];

/// A row of `zgody`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZgodyRow {
    pub id: String,
    pub uczestnik_id: String,
    pub rodzaj: String,
    pub tresc: String,
    pub zlozono: String,
    pub dokument_id: Option<String>,
    pub wplyw_id: Option<String>,
    pub trwaly_nosnik: bool,
    pub dotyczy_tabela: Option<String>,
    pub dotyczy_id: Option<String>,
    pub zastepuje_id: Option<String>,
}

impl ZgodyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZgodyRow {
            id: row.get("id")?,
            uczestnik_id: row.get("uczestnik_id")?,
            rodzaj: row.get("rodzaj")?,
            tresc: row.get("tresc")?,
            zlozono: row.get("zlozono")?,
            dokument_id: row.get("dokument_id")?,
            wplyw_id: row.get("wplyw_id")?,
            trwaly_nosnik: row.get("trwaly_nosnik")?,
            dotyczy_tabela: row.get("dotyczy_tabela")?,
            dotyczy_id: row.get("dotyczy_id")?,
            zastepuje_id: row.get("zastepuje_id")?,
        })
    }
}

/// Zgody i oświadczenia w teczce, najnowsze pierwsze.
pub fn zgody(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZgodyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_zgody_wpisy
ORDER BY zlozono DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(ZgodyRow::from_row).collect()
}

/// A row of `zgody_biezace`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZgodyBiezaceRow {
    pub id: String,
    pub uczestnik_id: String,
    pub rodzaj: String,
    pub tresc: String,
    pub zlozono: String,
    pub dokument_id: Option<String>,
    pub wplyw_id: Option<String>,
    pub trwaly_nosnik: bool,
    pub dotyczy_tabela: Option<String>,
    pub dotyczy_id: Option<String>,
    pub zastepuje_id: Option<String>,
}

impl ZgodyBiezaceRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZgodyBiezaceRow {
            id: row.get("id")?,
            uczestnik_id: row.get("uczestnik_id")?,
            rodzaj: row.get("rodzaj")?,
            tresc: row.get("tresc")?,
            zlozono: row.get("zlozono")?,
            dokument_id: row.get("dokument_id")?,
            wplyw_id: row.get("wplyw_id")?,
            trwaly_nosnik: row.get("trwaly_nosnik")?,
            dotyczy_tabela: row.get("dotyczy_tabela")?,
            dotyczy_id: row.get("dotyczy_id")?,
            zastepuje_id: row.get("zastepuje_id")?,
        })
    }
}

/// Zgody, których żadna nowsza nie zastąpiła.
pub fn zgody_biezace(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZgodyBiezaceRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_zgody_wpisy AS zgoda
WHERE NOT EXISTS (
  SELECT 1
  FROM sprawa_zgody_wpisy AS nowsza
  WHERE nowsza.zastepuje_id = zgoda.id
)
ORDER BY zgoda.zlozono DESC, zgoda.id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(ZgodyBiezaceRow::from_row).collect()
}

/// The parameters of `wpisz_zgode`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszZgodeParams {
    pub id: String,
    pub uczestnik_id: String,
    pub rodzaj: String,
    pub tresc: String,
    pub zlozono: String,
    pub dokument_id: Option<String>,
    pub wplyw_id: Option<String>,
    pub trwaly_nosnik: bool,
    pub dotyczy_tabela: Option<String>,
    pub dotyczy_id: Option<String>,
    pub zastepuje_id: Option<String>,
}

/// Zgoda albo oświadczenie z datą, treścią i dokumentem albo wiadomością, w której je złożono;
/// nowa zgoda w tej samej sprawie wskazuje tę, którą zastępuje.
pub fn wpisz_zgode(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszZgodeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_zgody_wpisy (id, uczestnik_id, rodzaj, tresc, zlozono, dokument_id, wplyw_id,
  trwaly_nosnik, dotyczy_tabela, dotyczy_id, zastepuje_id)
VALUES (:id, :uczestnik_id, :rodzaj, :tresc, :zlozono, :dokument_id, :wplyw_id, :trwaly_nosnik,
  :dotyczy_tabela, :dotyczy_id, :zastepuje_id)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.uczestnik_id),
                    ),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                    ("zlozono", xmeta_room::ToSqlValue::to_sql_value(&params.zlozono)),
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                    ("wplyw_id", xmeta_room::ToSqlValue::to_sql_value(&params.wplyw_id)),
                    (
                        "trwaly_nosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.trwaly_nosnik),
                    ),
                    (
                        "dotyczy_tabela",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_tabela),
                    ),
                    (
                        "dotyczy_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_id),
                    ),
                    (
                        "zastepuje_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zastepuje_id),
                    ),
                ]),
            },
        ],
    )
}
