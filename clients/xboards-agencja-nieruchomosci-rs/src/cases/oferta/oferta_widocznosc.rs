//! The kind `oferta-widocznosc` of the room `cases/oferta`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "oferta-widocznosc",
    requires: &[&super::sprawa_umowy::KIND],
    migrations: &[
        r#"-- Rodzaj oferty i jej widoczność z umowy pośrednictwa, w wersjach. Pierwszą wersję wpisuje Opiekun
-- oferty przy zawartej umowie, a każdą następną przy aneksie, który ją zmienia. Członkowie teczki
-- ją czytają, a Kierownik pośrednictwa przepisuje ją do katalogu ofert (katalog wskazuje wersję
-- tekstem, więc wersja ma UUID).
--
-- Oferta niepubliczna, czyli poza rynkiem, ma granice zgody: komu i co biuro może pokazać, np.
-- „tylko klientom z potwierdzonym finansowaniem, bez zdjęć wnętrz”. Ogłasza się tylko ofertę
-- publiczną (reguły w oferta-ogloszenia).
CREATE TABLE oferta_widocznosc_wersje (
  id TEXT PRIMARY KEY,
  umowa_id TEXT NOT NULL REFERENCES sprawa_umowy_wpisy (id),
  rodzaj_oferty TEXT NOT NULL CHECK (rodzaj_oferty IN ('sprzedaz', 'najem', 'sprzedaz_albo_najem')),
  widocznosc TEXT NOT NULL CHECK (widocznosc IN ('publiczna', 'niepubliczna')),
  granice_zgody TEXT,
  -- Zdarzenie aneks umowy, które zmieniło widoczność; pierwsza wersja wynika z samej umowy.
  aneks_zdarzenie_id TEXT REFERENCES sprawa_umowy_zdarzenia (id),
  -- Od kiedy wersja obowiązuje: chwila zawarcia umowy albo chwila aneksu.
  od TEXT NOT NULL,

  CONSTRAINT niepubliczna_ma_granice_zgody CHECK ((widocznosc = 'niepubliczna') = (granice_zgody IS NOT NULL))
) STRICT;

-- Bieżąca widoczność: najnowsza wersja. Teczka oferty ma jedną umowę pośrednictwa.
CREATE VIEW oferta_widocznosc_biezaca AS
SELECT *
FROM oferta_widocznosc_wersje
WHERE od = (
  SELECT max(od)
  FROM oferta_widocznosc_wersje
);

-- Pierwsza wersja umowy obowiązuje od jej zawarcia i nie wskazuje aneksu.
CREATE TRIGGER oferta_widocznosc_pierwsza_z_zawartej_umowy
BEFORE INSERT ON oferta_widocznosc_wersje
WHEN NOT EXISTS (
    SELECT 1
    FROM oferta_widocznosc_wersje
    WHERE umowa_id = NEW.umowa_id
  )
  AND (
    NEW.aneks_zdarzenie_id IS NOT NULL
    OR NOT EXISTS (
      SELECT 1
      FROM sprawa_umowy_wpisy
      WHERE id = NEW.umowa_id
        AND stan = 'zawarta'
        AND zawarto = NEW.od
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'pierwsza wersja widoczności powstaje przy zawartej umowie i obowiązuje od jej zawarcia');
END;

-- Widoczność i rodzaj oferty zmienia tylko aneks podpisany przez klienta: każda następna wersja
-- wskazuje zdarzenie aneks tej umowy, obowiązuje od jego chwili i jest późniejsza niż poprzednia.
CREATE TRIGGER oferta_widocznosc_kolejna_z_aneksu
BEFORE INSERT ON oferta_widocznosc_wersje
WHEN EXISTS (
    SELECT 1
    FROM oferta_widocznosc_wersje
    WHERE umowa_id = NEW.umowa_id
  )
  AND (
    NOT EXISTS (
      SELECT 1
      FROM sprawa_umowy_zdarzenia
      WHERE id = NEW.aneks_zdarzenie_id
        AND umowa_id = NEW.umowa_id
        AND rodzaj = 'aneks'
        AND chwila = NEW.od
    )
    OR NOT EXISTS (
      SELECT 1
      FROM sprawa_umowy_wpisy
      WHERE id = NEW.umowa_id
        AND stan = 'zawarta'
    )
    OR EXISTS (
      SELECT 1
      FROM oferta_widocznosc_wersje
      WHERE umowa_id = NEW.umowa_id
        AND od >= NEW.od
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'kolejna wersja widoczności wskazuje aneks zawartej umowy, obowiązuje od jego chwili i jest późniejsza niż poprzednia');
END;

CREATE TRIGGER oferta_widocznosc_wersje_bez_zmian
BEFORE UPDATE ON oferta_widocznosc_wersje
BEGIN
  SELECT RAISE(ABORT, 'wersji widoczności się nie zmienia; zmianę z aneksu wpisuje się jako nową wersję');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER oferta_widocznosc_wersje_bez_zastepowania
BEFORE INSERT ON oferta_widocznosc_wersje
WHEN EXISTS (
  SELECT 1
  FROM oferta_widocznosc_wersje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_widocznosc_wersje_bez_usuwania
BEFORE DELETE ON oferta_widocznosc_wersje
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
    ],
};

/// The values of column `wersje.rodzaj_oferty`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WersjeRodzajOferty {
    Sprzedaz,
    Najem,
    SprzedazAlboNajem,
}

impl WersjeRodzajOferty {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WersjeRodzajOferty] = &[
        WersjeRodzajOferty::Sprzedaz,
        WersjeRodzajOferty::Najem,
        WersjeRodzajOferty::SprzedazAlboNajem,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WersjeRodzajOferty::Sprzedaz => "sprzedaz",
            WersjeRodzajOferty::Najem => "najem",
            WersjeRodzajOferty::SprzedazAlboNajem => "sprzedaz_albo_najem",
        }
    }
}

impl xmeta_room::FromSqlValue for WersjeRodzajOferty {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "sprzedaz" => Ok(WersjeRodzajOferty::Sprzedaz),
                    "najem" => Ok(WersjeRodzajOferty::Najem),
                    "sprzedaz_albo_najem" => Ok(WersjeRodzajOferty::SprzedazAlboNajem),
                    other => Err(format!("{other} is no value of WersjeRodzajOferty")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WersjeRodzajOferty {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wersje.widocznosc`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WersjeWidocznosc {
    Publiczna,
    Niepubliczna,
}

impl WersjeWidocznosc {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WersjeWidocznosc] = &[
        WersjeWidocznosc::Publiczna,
        WersjeWidocznosc::Niepubliczna,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WersjeWidocznosc::Publiczna => "publiczna",
            WersjeWidocznosc::Niepubliczna => "niepubliczna",
        }
    }
}

impl xmeta_room::FromSqlValue for WersjeWidocznosc {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "publiczna" => Ok(WersjeWidocznosc::Publiczna),
                    "niepubliczna" => Ok(WersjeWidocznosc::Niepubliczna),
                    other => Err(format!("{other} is no value of WersjeWidocznosc")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WersjeWidocznosc {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `wersje_widocznosci`.
#[derive(Debug, Clone, PartialEq)]
pub struct WersjeWidocznosciRow {
    pub id: String,
    pub umowa_id: String,
    pub rodzaj_oferty: WersjeRodzajOferty,
    pub widocznosc: WersjeWidocznosc,
    pub granice_zgody: Option<String>,
    pub aneks_zdarzenie_id: Option<String>,
    pub od: String,
}

impl WersjeWidocznosciRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WersjeWidocznosciRow {
            id: row.get("id")?,
            umowa_id: row.get("umowa_id")?,
            rodzaj_oferty: row.get("rodzaj_oferty")?,
            widocznosc: row.get("widocznosc")?,
            granice_zgody: row.get("granice_zgody")?,
            aneks_zdarzenie_id: row.get("aneks_zdarzenie_id")?,
            od: row.get("od")?,
        })
    }
}

/// Wersje rodzaju i widoczności oferty od najstarszej.
pub fn wersje_widocznosci(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WersjeWidocznosciRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_widocznosc_wersje
ORDER BY od, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(WersjeWidocznosciRow::from_row).collect()
}

/// A row of `biezaca_widocznosc`.
#[derive(Debug, Clone, PartialEq)]
pub struct BiezacaWidocznoscRow {
    pub id: String,
    pub umowa_id: String,
    pub rodzaj_oferty: WersjeRodzajOferty,
    pub widocznosc: WersjeWidocznosc,
    pub granice_zgody: Option<String>,
    pub aneks_zdarzenie_id: Option<String>,
    pub od: String,
}

impl BiezacaWidocznoscRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(BiezacaWidocznoscRow {
            id: row.get("id")?,
            umowa_id: row.get("umowa_id")?,
            rodzaj_oferty: row.get("rodzaj_oferty")?,
            widocznosc: row.get("widocznosc")?,
            granice_zgody: row.get("granice_zgody")?,
            aneks_zdarzenie_id: row.get("aneks_zdarzenie_id")?,
            od: row.get("od")?,
        })
    }
}

/// Rodzaj i widoczność oferty, które obowiązują teraz.
pub fn biezaca_widocznosc(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Option<BiezacaWidocznoscRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_widocznosc_biezaca"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.first().map(BiezacaWidocznoscRow::from_row).transpose()
}

/// The parameters of `wpisz_wersje_widocznosci`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszWersjeWidocznosciParams {
    pub id: String,
    pub umowa_id: String,
    pub rodzaj_oferty: WersjeRodzajOferty,
    pub widocznosc: WersjeWidocznosc,
    pub granice_zgody: Option<String>,
    pub aneks_zdarzenie_id: Option<String>,
    pub od: String,
}

/// Rodzaj i widoczność oferty z zawartej umowy (od chwili zawarcia, bez aneksu) albo z aneksu,
/// który je zmienił (od chwili aneksu); identyfikator (UUID) nadaje aplikacja, bo wskazuje go
/// katalog ofert.
pub fn wpisz_wersje_widocznosci(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszWersjeWidocznosciParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO oferta_widocznosc_wersje (id, umowa_id, rodzaj_oferty, widocznosc, granice_zgody,
  aneks_zdarzenie_id, od)
VALUES (:id, :umowa_id, :rodzaj_oferty, :widocznosc, :granice_zgody, :aneks_zdarzenie_id, :od)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("umowa_id", xmeta_room::ToSqlValue::to_sql_value(&params.umowa_id)),
                    (
                        "rodzaj_oferty",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj_oferty),
                    ),
                    (
                        "widocznosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.widocznosc),
                    ),
                    (
                        "granice_zgody",
                        xmeta_room::ToSqlValue::to_sql_value(&params.granice_zgody),
                    ),
                    (
                        "aneks_zdarzenie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.aneks_zdarzenie_id),
                    ),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                ]),
            },
        ],
    )
}
