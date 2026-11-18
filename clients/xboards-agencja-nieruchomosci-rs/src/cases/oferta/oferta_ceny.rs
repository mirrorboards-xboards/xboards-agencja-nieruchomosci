//! The kind `oferta-ceny` of the room `cases/oferta`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "oferta-ceny";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Ceny oferty: propozycja biura z porównanymi transakcjami, cena ofertowa zatwierdzona przez
-- klienta z historią zmian i granice negocjacji, osobno dla sprzedaży i dla najmu (dotyczy).
-- Przy najmie kwota to miesięczny czynsz. Kwoty w groszach. Pisze Opiekun oferty.
--
-- Cena oferty to propozycja biura, a nie operat szacunkowy. Cenę ofertową i granice wyznacza
-- klient: każdy wpis wskazuje jego zgodę, która dotyczy tego wpisu. Żadnego wpisu się nie zmienia
-- ani nie usuwa, więc widać, czy cena nie przesuwa się po trochu bez decyzji klienta. Granice
-- negocjacji nie wychodzą poza teczkę oferty (R1); pilnuje tego aplikacja.

-- Propozycja ceny albo czynszu z uzasadnieniem i plikiem wyliczenia z programu (np. średniej ceny
-- za metr z porównanych transakcji).
CREATE TABLE oferta_ceny_propozycje (
  id TEXT PRIMARY KEY,
  dotyczy TEXT NOT NULL CHECK (dotyczy IN ('sprzedaz', 'najem')),
  kwota INTEGER NOT NULL CHECK (kwota > 0),
  uzasadnienie TEXT NOT NULL,
  utworzono TEXT NOT NULL,
  wyliczenie_sciezka TEXT NOT NULL,
  wyliczenie_skrot TEXT NOT NULL
) STRICT;

-- Transakcje z rejestru cen nieruchomości (art. 181a pkt 6 ugn), z którymi Opiekun porównał
-- nieruchomość; cenę za metr liczy program.
CREATE TABLE oferta_ceny_porownania (
  -- Klucz nadany przez bazę jest dodatni; zob. oferta_ceny_porownania_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  propozycja_id TEXT NOT NULL REFERENCES oferta_ceny_propozycje (id),
  data_transakcji TEXT NOT NULL,
  cena INTEGER NOT NULL CHECK (cena > 0),
  -- Powierzchnia w setnych metra kwadratowego, jak w rdzeniu.
  powierzchnia INTEGER CHECK (powierzchnia > 0),
  -- Różnice wobec oferty, np. piętro albo stan.
  roznice TEXT,
  zrodlo TEXT NOT NULL
) STRICT;

-- Cena ofertowa, którą klient zatwierdził; każda zmiana to nowy wpis, który wskazuje poprzednią
-- cenę i ma powód. Ta cena trafia do ogłoszenia (w_ogloszeniach_od).
CREATE TABLE oferta_ceny_ofertowe (
  id TEXT PRIMARY KEY,
  dotyczy TEXT NOT NULL CHECK (dotyczy IN ('sprzedaz', 'najem')),
  kwota INTEGER NOT NULL CHECK (kwota > 0),
  zgoda_id TEXT NOT NULL REFERENCES sprawa_zgody_wpisy (id),
  obowiazuje_od TEXT NOT NULL,
  poprzednia_id TEXT REFERENCES oferta_ceny_ofertowe (id),
  powod TEXT,
  -- Dzień (bez godziny), w którym nowa cena weszła do ogłoszeń; wpisuje się go raz, po zmianie
  -- ogłoszeń.
  w_ogloszeniach_od TEXT,

  CONSTRAINT zmiana_ceny_ma_powod CHECK ((poprzednia_id IS NULL) = (powod IS NULL)),
  CONSTRAINT w_ogloszeniach_nie_przed_cena CHECK (
    w_ogloszeniach_od IS NULL OR w_ogloszeniach_od >= substr(obowiazuje_od, 1, 10)
  )
) STRICT;

-- Granice negocjacji: najniższa cena albo czynsz, które klient przyjmie, i warunki, od których to
-- zależy. Nowe granice zastępują poprzednie.
CREATE TABLE oferta_ceny_granice (
  id TEXT PRIMARY KEY,
  dotyczy TEXT NOT NULL CHECK (dotyczy IN ('sprzedaz', 'najem')),
  najnizsza_kwota INTEGER NOT NULL CHECK (najnizsza_kwota > 0),
  warunki TEXT,
  zgoda_id TEXT NOT NULL REFERENCES sprawa_zgody_wpisy (id),
  zastepuje_id TEXT REFERENCES oferta_ceny_granice (id),

  CONSTRAINT nie_zastepuje_samych_siebie CHECK (zastepuje_id IS NOT id)
) STRICT;

-- Cena ofertowa, która obowiązuje: żadna nowsza cena jej nie wskazuje.
CREATE VIEW oferta_ceny_biezace AS
SELECT *
FROM oferta_ceny_ofertowe AS cena
WHERE NOT EXISTS (
  SELECT 1
  FROM oferta_ceny_ofertowe AS nowsza
  WHERE nowsza.poprzednia_id = cena.id
);

-- Cenę ofertową zatwierdza klient: zgoda zatwierdzenie_ceny uczestnika z obowiązującą rolą
-- klient, która dotyczy tej ceny.
CREATE TRIGGER oferta_ceny_ofertowa_ze_zgoda_klienta
BEFORE INSERT ON oferta_ceny_ofertowe
WHEN NOT EXISTS (
  SELECT 1
  FROM sprawa_zgody_wpisy AS zgoda
  JOIN sprawa_uczestnicy_role AS rola ON rola.uczestnik_id = zgoda.uczestnik_id
  WHERE zgoda.id = NEW.zgoda_id
    AND zgoda.rodzaj = 'zatwierdzenie_ceny'
    AND zgoda.dotyczy_tabela = 'oferta_ceny_ofertowe'
    AND zgoda.dotyczy_id = NEW.id
    AND rola.rola = 'klient'
    AND rola.zakonczono IS NULL
)
BEGIN
  SELECT RAISE(ABORT, 'cena ofertowa ma zgodę zatwierdzenie_ceny klienta, która jej dotyczy');
END;

-- Pierwsza cena (sprzedaży albo najmu) nie ma poprzedniej; każda następna wskazuje cenę, która
-- obowiązuje, i obowiązuje od chwili późniejszej niż ona.
CREATE TRIGGER oferta_ceny_zmiana_wskazuje_obowiazujaca
BEFORE INSERT ON oferta_ceny_ofertowe
WHEN (
    NEW.poprzednia_id IS NULL
    AND EXISTS (
      SELECT 1
      FROM oferta_ceny_ofertowe
      WHERE dotyczy = NEW.dotyczy
    )
  )
  OR (
    NEW.poprzednia_id IS NOT NULL
    AND NOT EXISTS (
      SELECT 1
      FROM oferta_ceny_biezace
      WHERE id = NEW.poprzednia_id
        AND dotyczy = NEW.dotyczy
        AND obowiazuje_od < NEW.obowiazuje_od
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'zmiana ceny wskazuje obowiązującą cenę tego samego rodzaju i obowiązuje od chwili późniejszej niż ona');
END;

-- Cena powstaje bez daty wejścia do ogłoszeń, a potem dostaje ją raz; poza tym jej się nie
-- zmienia.
CREATE TRIGGER oferta_ceny_ofertowa_powstaje_przed_ogloszeniami
BEFORE INSERT ON oferta_ceny_ofertowe
WHEN NEW.w_ogloszeniach_od IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'cena ofertowa powstaje bez daty wejścia do ogłoszeń');
END;

CREATE TRIGGER oferta_ceny_ofertowe_bez_zmian
BEFORE UPDATE ON oferta_ceny_ofertowe
WHEN OLD.w_ogloszeniach_od IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.dotyczy IS NOT OLD.dotyczy
  OR NEW.kwota IS NOT OLD.kwota
  OR NEW.zgoda_id IS NOT OLD.zgoda_id
  OR NEW.obowiazuje_od IS NOT OLD.obowiazuje_od
  OR NEW.poprzednia_id IS NOT OLD.poprzednia_id
  OR NEW.powod IS NOT OLD.powod
BEGIN
  SELECT RAISE(ABORT, 'ceny ofertowej się nie zmienia; raz wpisuje się tylko dzień wejścia do ogłoszeń');
END;

-- Granice wyznacza klient: zgoda granice_negocjacji uczestnika z obowiązującą rolą klient,
-- która dotyczy tych granic.
CREATE TRIGGER oferta_ceny_granice_ze_zgoda_klienta
BEFORE INSERT ON oferta_ceny_granice
WHEN NOT EXISTS (
  SELECT 1
  FROM sprawa_zgody_wpisy AS zgoda
  JOIN sprawa_uczestnicy_role AS rola ON rola.uczestnik_id = zgoda.uczestnik_id
  WHERE zgoda.id = NEW.zgoda_id
    AND zgoda.rodzaj = 'granice_negocjacji'
    AND zgoda.dotyczy_tabela = 'oferta_ceny_granice'
    AND zgoda.dotyczy_id = NEW.id
    AND rola.rola = 'klient'
    AND rola.zakonczono IS NULL
)
BEGIN
  SELECT RAISE(ABORT, 'granice negocjacji mają zgodę granice_negocjacji klienta, która ich dotyczy');
END;

-- Pierwsze granice nie zastępują żadnych; następne zastępują granice tego samego rodzaju, które
-- obowiązują.
CREATE TRIGGER oferta_ceny_granice_zastepuja_obowiazujace
BEFORE INSERT ON oferta_ceny_granice
WHEN (
    NEW.zastepuje_id IS NULL
    AND EXISTS (
      SELECT 1
      FROM oferta_ceny_granice
      WHERE dotyczy = NEW.dotyczy
    )
  )
  OR (
    NEW.zastepuje_id IS NOT NULL
    AND NOT EXISTS (
      SELECT 1
      FROM oferta_ceny_granice AS poprzednie
      WHERE poprzednie.id = NEW.zastepuje_id
        AND poprzednie.dotyczy = NEW.dotyczy
        AND NOT EXISTS (
          SELECT 1
          FROM oferta_ceny_granice AS nowsze
          WHERE nowsze.zastepuje_id = poprzednie.id
        )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'nowe granice negocjacji zastępują obowiązujące granice tego samego rodzaju');
END;

CREATE TRIGGER oferta_ceny_propozycje_bez_zmian
BEFORE UPDATE ON oferta_ceny_propozycje
BEGIN
  SELECT RAISE(ABORT, 'propozycji ceny się nie zmienia; nowa propozycja to nowy wpis');
END;

CREATE TRIGGER oferta_ceny_porownania_bez_zmian
BEFORE UPDATE ON oferta_ceny_porownania
BEGIN
  SELECT RAISE(ABORT, 'porównania transakcji się nie zmienia; nowe porównanie to nowy wpis');
END;

CREATE TRIGGER oferta_ceny_granice_bez_zmian
BEFORE UPDATE ON oferta_ceny_granice
BEGIN
  SELECT RAISE(ABORT, 'granic negocjacji się nie zmienia; nowe granice zastępują poprzednie');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER oferta_ceny_propozycje_bez_zastepowania
BEFORE INSERT ON oferta_ceny_propozycje
WHEN EXISTS (
  SELECT 1
  FROM oferta_ceny_propozycje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_ceny_propozycje_bez_usuwania
BEFORE DELETE ON oferta_ceny_propozycje
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER oferta_ceny_porownania_bez_zastepowania
BEFORE INSERT ON oferta_ceny_porownania
WHEN EXISTS (
  SELECT 1
  FROM oferta_ceny_porownania
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_ceny_porownania_bez_usuwania
BEFORE DELETE ON oferta_ceny_porownania
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER oferta_ceny_ofertowe_bez_zastepowania
BEFORE INSERT ON oferta_ceny_ofertowe
WHEN EXISTS (
  SELECT 1
  FROM oferta_ceny_ofertowe
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_ceny_ofertowe_bez_usuwania
BEFORE DELETE ON oferta_ceny_ofertowe
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER oferta_ceny_granice_bez_zastepowania
BEFORE INSERT ON oferta_ceny_granice
WHEN EXISTS (
  SELECT 1
  FROM oferta_ceny_granice
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_ceny_granice_bez_usuwania
BEFORE DELETE ON oferta_ceny_granice
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
];

/// The values of column `granice.dotyczy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraniceDotyczy {
    Sprzedaz,
    Najem,
}

impl GraniceDotyczy {
    /// Every value, in the order of the catalog.
    pub const ALL: &[GraniceDotyczy] = &[
        GraniceDotyczy::Sprzedaz,
        GraniceDotyczy::Najem,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            GraniceDotyczy::Sprzedaz => "sprzedaz",
            GraniceDotyczy::Najem => "najem",
        }
    }
}

impl xmeta_room::FromSqlValue for GraniceDotyczy {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "sprzedaz" => Ok(GraniceDotyczy::Sprzedaz),
                    "najem" => Ok(GraniceDotyczy::Najem),
                    other => Err(format!("{other} is no value of GraniceDotyczy")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for GraniceDotyczy {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `granice_negocjacji`.
#[derive(Debug, Clone, PartialEq)]
pub struct GraniceNegocjacjiRow {
    pub id: String,
    pub dotyczy: GraniceDotyczy,
    pub najnizsza_kwota: i64,
    pub warunki: Option<String>,
    pub zgoda_id: String,
    pub zastepuje_id: Option<String>,
}

impl GraniceNegocjacjiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(GraniceNegocjacjiRow {
            id: row.get("id")?,
            dotyczy: row.get("dotyczy")?,
            najnizsza_kwota: row.get("najnizsza_kwota")?,
            warunki: row.get("warunki")?,
            zgoda_id: row.get("zgoda_id")?,
            zastepuje_id: row.get("zastepuje_id")?,
        })
    }
}

/// Granice negocjacji od najstarszych; nie wychodzą poza teczkę oferty (R1).
pub fn granice_negocjacji(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<GraniceNegocjacjiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_ceny_granice
ORDER BY dotyczy, rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(GraniceNegocjacjiRow::from_row).collect()
}

/// A row of `granice_negocjacji_biezace`.
#[derive(Debug, Clone, PartialEq)]
pub struct GraniceNegocjacjiBiezaceRow {
    pub id: String,
    pub dotyczy: GraniceDotyczy,
    pub najnizsza_kwota: i64,
    pub warunki: Option<String>,
    pub zgoda_id: String,
    pub zastepuje_id: Option<String>,
}

impl GraniceNegocjacjiBiezaceRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(GraniceNegocjacjiBiezaceRow {
            id: row.get("id")?,
            dotyczy: row.get("dotyczy")?,
            najnizsza_kwota: row.get("najnizsza_kwota")?,
            warunki: row.get("warunki")?,
            zgoda_id: row.get("zgoda_id")?,
            zastepuje_id: row.get("zastepuje_id")?,
        })
    }
}

/// Granice negocjacji, których nie zastąpiły nowsze.
pub fn granice_negocjacji_biezace(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<GraniceNegocjacjiBiezaceRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_ceny_granice AS granice
WHERE NOT EXISTS (
  SELECT 1
  FROM oferta_ceny_granice AS nowsze
  WHERE nowsze.zastepuje_id = granice.id
)
ORDER BY granice.dotyczy"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(GraniceNegocjacjiBiezaceRow::from_row).collect()
}

/// The parameters of `wpisz_granice_negocjacji`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszGraniceNegocjacjiParams {
    pub id: String,
    pub dotyczy: GraniceDotyczy,
    pub najnizsza_kwota: i64,
    pub warunki: Option<String>,
    pub zgoda_id: String,
    pub zastepuje_id: Option<String>,
}

/// Najniższa cena albo czynsz, które klient przyjmie, z warunkami i jego zgodą; nowe granice
/// zastępują obowiązujące.
pub fn wpisz_granice_negocjacji(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszGraniceNegocjacjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO oferta_ceny_granice (id, dotyczy, najnizsza_kwota, warunki, zgoda_id, zastepuje_id)
VALUES (:id, :dotyczy, :najnizsza_kwota, :warunki, :zgoda_id, :zastepuje_id)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("dotyczy", xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy)),
                    (
                        "najnizsza_kwota",
                        xmeta_room::ToSqlValue::to_sql_value(&params.najnizsza_kwota),
                    ),
                    ("warunki", xmeta_room::ToSqlValue::to_sql_value(&params.warunki)),
                    ("zgoda_id", xmeta_room::ToSqlValue::to_sql_value(&params.zgoda_id)),
                    (
                        "zastepuje_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zastepuje_id),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `ceny_ofertowe`.
#[derive(Debug, Clone, PartialEq)]
pub struct CenyOfertoweRow {
    pub id: String,
    pub dotyczy: GraniceDotyczy,
    pub kwota: i64,
    pub zgoda_id: String,
    pub obowiazuje_od: String,
    pub poprzednia_id: Option<String>,
    pub powod: Option<String>,
    pub w_ogloszeniach_od: Option<String>,
}

impl CenyOfertoweRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(CenyOfertoweRow {
            id: row.get("id")?,
            dotyczy: row.get("dotyczy")?,
            kwota: row.get("kwota")?,
            zgoda_id: row.get("zgoda_id")?,
            obowiazuje_od: row.get("obowiazuje_od")?,
            poprzednia_id: row.get("poprzednia_id")?,
            powod: row.get("powod")?,
            w_ogloszeniach_od: row.get("w_ogloszeniach_od")?,
        })
    }
}

/// Historia cen ofertowych od najstarszej: każda zmiana wskazuje poprzednią cenę.
pub fn ceny_ofertowe(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<CenyOfertoweRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_ceny_ofertowe
ORDER BY dotyczy, obowiazuje_od, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(CenyOfertoweRow::from_row).collect()
}

/// A row of `ceny_ofertowe_biezace`.
#[derive(Debug, Clone, PartialEq)]
pub struct CenyOfertoweBiezaceRow {
    pub id: String,
    pub dotyczy: GraniceDotyczy,
    pub kwota: i64,
    pub zgoda_id: String,
    pub obowiazuje_od: String,
    pub poprzednia_id: Option<String>,
    pub powod: Option<String>,
    pub w_ogloszeniach_od: Option<String>,
}

impl CenyOfertoweBiezaceRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(CenyOfertoweBiezaceRow {
            id: row.get("id")?,
            dotyczy: row.get("dotyczy")?,
            kwota: row.get("kwota")?,
            zgoda_id: row.get("zgoda_id")?,
            obowiazuje_od: row.get("obowiazuje_od")?,
            poprzednia_id: row.get("poprzednia_id")?,
            powod: row.get("powod")?,
            w_ogloszeniach_od: row.get("w_ogloszeniach_od")?,
        })
    }
}

/// Ceny ofertowe, które obowiązują: sprzedaży, najmu albo obie.
pub fn ceny_ofertowe_biezace(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<CenyOfertoweBiezaceRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_ceny_biezace
ORDER BY dotyczy"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(CenyOfertoweBiezaceRow::from_row).collect()
}

/// The parameters of `wpisz_cene_ofertowa`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszCeneOfertowaParams {
    pub id: String,
    pub dotyczy: GraniceDotyczy,
    pub kwota: i64,
    pub zgoda_id: String,
    pub obowiazuje_od: String,
    pub poprzednia_id: Option<String>,
    pub powod: Option<String>,
}

/// Cena ofertowa zatwierdzona przez klienta. Zmiana ceny wskazuje cenę, która obowiązywała,
/// i ma powód; identyfikator (UUID) nadaje aplikacja, bo wskazuje go zgoda klienta.
pub fn wpisz_cene_ofertowa(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszCeneOfertowaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO oferta_ceny_ofertowe (id, dotyczy, kwota, zgoda_id, obowiazuje_od, poprzednia_id, powod)
VALUES (:id, :dotyczy, :kwota, :zgoda_id, :obowiazuje_od, :poprzednia_id, :powod)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("dotyczy", xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy)),
                    ("kwota", xmeta_room::ToSqlValue::to_sql_value(&params.kwota)),
                    ("zgoda_id", xmeta_room::ToSqlValue::to_sql_value(&params.zgoda_id)),
                    (
                        "obowiazuje_od",
                        xmeta_room::ToSqlValue::to_sql_value(&params.obowiazuje_od),
                    ),
                    (
                        "poprzednia_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprzednia_id),
                    ),
                    ("powod", xmeta_room::ToSqlValue::to_sql_value(&params.powod)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_cene_w_ogloszeniach`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujCeneWOgloszeniachParams {
    pub w_ogloszeniach_od: String,
    pub id: String,
}

/// Dzień, w którym cena weszła do ogłoszeń; wpisuje się go raz.
pub fn odnotuj_cene_w_ogloszeniach(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujCeneWOgloszeniachParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE oferta_ceny_ofertowe
SET w_ogloszeniach_od = :w_ogloszeniach_od
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "w_ogloszeniach_od",
                        xmeta_room::ToSqlValue::to_sql_value(&params.w_ogloszeniach_od),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `propozycje_cen`.
#[derive(Debug, Clone, PartialEq)]
pub struct PropozycjeCenRow {
    pub id: String,
    pub dotyczy: GraniceDotyczy,
    pub kwota: i64,
    pub uzasadnienie: String,
    pub utworzono: String,
    pub wyliczenie_sciezka: String,
    pub wyliczenie_skrot: String,
}

impl PropozycjeCenRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PropozycjeCenRow {
            id: row.get("id")?,
            dotyczy: row.get("dotyczy")?,
            kwota: row.get("kwota")?,
            uzasadnienie: row.get("uzasadnienie")?,
            utworzono: row.get("utworzono")?,
            wyliczenie_sciezka: row.get("wyliczenie_sciezka")?,
            wyliczenie_skrot: row.get("wyliczenie_skrot")?,
        })
    }
}

/// Propozycje ceny i czynszu biura, najnowsze pierwsze.
pub fn propozycje_cen(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PropozycjeCenRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_ceny_propozycje
ORDER BY utworzono DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(PropozycjeCenRow::from_row).collect()
}

/// A row of `porownania_transakcji`.
#[derive(Debug, Clone, PartialEq)]
pub struct PorownaniaTransakcjiRow {
    pub id: i64,
    pub propozycja_id: String,
    pub data_transakcji: String,
    pub cena: i64,
    pub powierzchnia: Option<i64>,
    pub roznice: Option<String>,
    pub zrodlo: String,
}

impl PorownaniaTransakcjiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PorownaniaTransakcjiRow {
            id: row.get("id")?,
            propozycja_id: row.get("propozycja_id")?,
            data_transakcji: row.get("data_transakcji")?,
            cena: row.get("cena")?,
            powierzchnia: row.get("powierzchnia")?,
            roznice: row.get("roznice")?,
            zrodlo: row.get("zrodlo")?,
        })
    }
}

/// Transakcje porównane w propozycjach ceny, w kolejności wpisania.
pub fn porownania_transakcji(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PorownaniaTransakcjiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_ceny_porownania
ORDER BY propozycja_id, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(PorownaniaTransakcjiRow::from_row).collect()
}

/// The parameters of `wpisz_propozycje_ceny`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszPropozycjeCenyParams {
    pub id: String,
    pub dotyczy: GraniceDotyczy,
    pub kwota: i64,
    pub uzasadnienie: String,
    pub utworzono: String,
    pub wyliczenie_sciezka: String,
    pub wyliczenie_skrot: String,
}

/// Propozycja ceny albo czynszu z uzasadnieniem i plikiem wyliczenia z programu; identyfikator
/// (UUID) nadaje aplikacja, bo wskazują go porównania.
pub fn wpisz_propozycje_ceny(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszPropozycjeCenyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO oferta_ceny_propozycje (id, dotyczy, kwota, uzasadnienie, utworzono, wyliczenie_sciezka,
  wyliczenie_skrot)
VALUES (:id, :dotyczy, :kwota, :uzasadnienie, :utworzono, :wyliczenie_sciezka, :wyliczenie_skrot)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("dotyczy", xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy)),
                    ("kwota", xmeta_room::ToSqlValue::to_sql_value(&params.kwota)),
                    (
                        "uzasadnienie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.uzasadnienie),
                    ),
                    (
                        "utworzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.utworzono),
                    ),
                    (
                        "wyliczenie_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wyliczenie_sciezka),
                    ),
                    (
                        "wyliczenie_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wyliczenie_skrot),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `dopisz_porownanie_transakcji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DopiszPorownanieTransakcjiParams {
    pub propozycja_id: String,
    pub data_transakcji: String,
    pub cena: i64,
    pub powierzchnia: Option<i64>,
    pub roznice: Option<String>,
    pub zrodlo: String,
}

/// Transakcja z rejestru cen nieruchomości porównana w propozycji, z różnicami wobec oferty.
pub fn dopisz_porownanie_transakcji(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszPorownanieTransakcjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO oferta_ceny_porownania (propozycja_id, data_transakcji, cena, powierzchnia, roznice,
  zrodlo)
VALUES (:propozycja_id, :data_transakcji, :cena, :powierzchnia, :roznice, :zrodlo)"#,
                parameters: Vec::from([
                    (
                        "propozycja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.propozycja_id),
                    ),
                    (
                        "data_transakcji",
                        xmeta_room::ToSqlValue::to_sql_value(&params.data_transakcji),
                    ),
                    ("cena", xmeta_room::ToSqlValue::to_sql_value(&params.cena)),
                    (
                        "powierzchnia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.powierzchnia),
                    ),
                    ("roznice", xmeta_room::ToSqlValue::to_sql_value(&params.roznice)),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                ]),
            },
        ],
    )
}
