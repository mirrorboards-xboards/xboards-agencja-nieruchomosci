//! The kind `sprawa-nieruchomosci` of the room `cases/najem`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "sprawa-nieruchomosci",
    requires: &[],
    migrations: &[
        r#"-- Oznaczenie nieruchomości teczki: lokal, dom albo działka, z księgą wieczystą i działkami.
-- Korzystają z niego typy oferty, transakcji, najmu i lokalu, pokój wymiany i dokumenty („czego
-- dotyczy”). Poszukiwanie go nie używa: obejrzane nieruchomości to jego propozycje.
--
-- Nieruchomość wpisuje agent sprawy. Ocenę stanu prawnego robi Analityk stanu prawnego w typie
-- sprawy; rdzeń trzyma tylko oznaczenie.
CREATE TABLE sprawa_nieruchomosci_wpisy (
  id TEXT PRIMARY KEY,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('lokal_mieszkalny', 'lokal_uzytkowy', 'dom', 'dzialka', 'garaz', 'miejsce_postojowe', 'pomieszczenie_przynalezne')),
  prawo TEXT NOT NULL CHECK (prawo IN ('wlasnosc', 'odrebna_wlasnosc_lokalu', 'spoldzielcze_wlasnosciowe_prawo_do_lokalu', 'uzytkowanie_wieczyste', 'udzial_we_wspolwlasnosci', 'inne')),
  miejscowosc TEXT NOT NULL,
  gmina TEXT NOT NULL,
  ulica TEXT,
  numer_budynku TEXT,
  numer_lokalu TEXT,
  kod_pocztowy TEXT,
  -- Numer księgi wieczystej, np. WA1M/00012345/6. Lokal bez księgi, np. część spółdzielczych
  -- praw do lokalu, ma kw_brak = 1.
  numer_kw TEXT,
  kw_brak INTEGER NOT NULL DEFAULT 0 CHECK (kw_brak IN (0, 1)),
  -- Powierzchnia użytkowa w setnych metra kwadratowego (48,20 m² to 4820).
  powierzchnia_uzytkowa INTEGER CHECK (powierzchnia_uzytkowa > 0),
  liczba_pokoi INTEGER CHECK (liczba_pokoi >= 0),
  pietro INTEGER,
  -- Lokal, do którego należy pomieszczenie przynależne, np. piwnica.
  nadrzedna_id TEXT REFERENCES sprawa_nieruchomosci_wpisy (id),

  CONSTRAINT numer_kw_albo_brak_ksiegi CHECK (
    (kw_brak = 1 AND numer_kw IS NULL)
    OR (kw_brak = 0 AND numer_kw IS NOT NULL AND numer_kw GLOB '[A-Z][A-Z][0-9][A-Z]/[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]/[0-9]')
  ),
  CONSTRAINT pomieszczenie_przynalezne_wskazuje_lokal CHECK (
    (rodzaj = 'pomieszczenie_przynalezne') = (nadrzedna_id IS NOT NULL)
  )
) STRICT;

-- Działki ewidencyjne nieruchomości: obręb, numer i powierzchnia.
CREATE TABLE sprawa_nieruchomosci_dzialki (
  id INTEGER PRIMARY KEY,
  nieruchomosc_id TEXT NOT NULL REFERENCES sprawa_nieruchomosci_wpisy (id),
  obreb TEXT NOT NULL,
  numer_dzialki TEXT NOT NULL,
  -- Powierzchnia w metrach kwadratowych, jak w ewidencji gruntów.
  powierzchnia INTEGER CHECK (powierzchnia > 0),

  UNIQUE (nieruchomosc_id, obreb, numer_dzialki)
) STRICT;

-- Pomieszczenie przynależne należy do lokalu, nie do domu ani do innej piwnicy.
CREATE TRIGGER sprawa_nieruchomosci_nadrzedna_to_lokal
BEFORE INSERT ON sprawa_nieruchomosci_wpisy
WHEN NEW.nadrzedna_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_nieruchomosci_wpisy
    WHERE id = NEW.nadrzedna_id
      AND rodzaj IN ('lokal_mieszkalny', 'lokal_uzytkowy')
  )
BEGIN
  SELECT RAISE(ABORT, 'pomieszczenie przynależne wskazuje lokal');
END;

CREATE TRIGGER sprawa_nieruchomosci_zmieniona_nadrzedna_to_lokal
BEFORE UPDATE OF nadrzedna_id ON sprawa_nieruchomosci_wpisy
WHEN NEW.nadrzedna_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_nieruchomosci_wpisy
    WHERE id = NEW.nadrzedna_id
      AND rodzaj IN ('lokal_mieszkalny', 'lokal_uzytkowy')
  )
BEGIN
  SELECT RAISE(ABORT, 'pomieszczenie przynależne wskazuje lokal');
END;

-- Od rodzaju zależą reguły pomieszczenia przynależnego i to, co typ sprawy wie o nieruchomości;
-- zmiana lokalu w dom ominęłaby regułę, że pomieszczenie przynależne wskazuje lokal.
CREATE TRIGGER sprawa_nieruchomosci_rodzaj_niezmienny
BEFORE UPDATE OF rodzaj ON sprawa_nieruchomosci_wpisy
WHEN NEW.rodzaj IS NOT OLD.rodzaj
BEGIN
  SELECT RAISE(ABORT, 'rodzaj nieruchomości się nie zmienia; nieruchomość innego rodzaju to nowy wiersz');
END;

-- Działkę wolno usunąć jako poprawkę pomyłki (D14); nieruchomość zostaje.
-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_nieruchomosci_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_nieruchomosci_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_nieruchomosci_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_nieruchomosci_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_nieruchomosci_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
    ],
};

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    LokalMieszkalny,
    LokalUzytkowy,
    Dom,
    Dzialka,
    Garaz,
    MiejscePostojowe,
    PomieszczeniePrzynalezne,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::LokalMieszkalny,
        WpisyRodzaj::LokalUzytkowy,
        WpisyRodzaj::Dom,
        WpisyRodzaj::Dzialka,
        WpisyRodzaj::Garaz,
        WpisyRodzaj::MiejscePostojowe,
        WpisyRodzaj::PomieszczeniePrzynalezne,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::LokalMieszkalny => "lokal_mieszkalny",
            WpisyRodzaj::LokalUzytkowy => "lokal_uzytkowy",
            WpisyRodzaj::Dom => "dom",
            WpisyRodzaj::Dzialka => "dzialka",
            WpisyRodzaj::Garaz => "garaz",
            WpisyRodzaj::MiejscePostojowe => "miejsce_postojowe",
            WpisyRodzaj::PomieszczeniePrzynalezne => "pomieszczenie_przynalezne",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "lokal_mieszkalny" => Ok(WpisyRodzaj::LokalMieszkalny),
                    "lokal_uzytkowy" => Ok(WpisyRodzaj::LokalUzytkowy),
                    "dom" => Ok(WpisyRodzaj::Dom),
                    "dzialka" => Ok(WpisyRodzaj::Dzialka),
                    "garaz" => Ok(WpisyRodzaj::Garaz),
                    "miejsce_postojowe" => Ok(WpisyRodzaj::MiejscePostojowe),
                    "pomieszczenie_przynalezne" => {
                        Ok(WpisyRodzaj::PomieszczeniePrzynalezne)
                    }
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

/// The values of column `wpisy.prawo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyPrawo {
    Wlasnosc,
    OdrebnaWlasnoscLokalu,
    SpoldzielczeWlasnosciowePrawoDoLokalu,
    UzytkowanieWieczyste,
    UdzialWeWspolwlasnosci,
    Inne,
}

impl WpisyPrawo {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyPrawo] = &[
        WpisyPrawo::Wlasnosc,
        WpisyPrawo::OdrebnaWlasnoscLokalu,
        WpisyPrawo::SpoldzielczeWlasnosciowePrawoDoLokalu,
        WpisyPrawo::UzytkowanieWieczyste,
        WpisyPrawo::UdzialWeWspolwlasnosci,
        WpisyPrawo::Inne,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyPrawo::Wlasnosc => "wlasnosc",
            WpisyPrawo::OdrebnaWlasnoscLokalu => "odrebna_wlasnosc_lokalu",
            WpisyPrawo::SpoldzielczeWlasnosciowePrawoDoLokalu => {
                "spoldzielcze_wlasnosciowe_prawo_do_lokalu"
            }
            WpisyPrawo::UzytkowanieWieczyste => "uzytkowanie_wieczyste",
            WpisyPrawo::UdzialWeWspolwlasnosci => "udzial_we_wspolwlasnosci",
            WpisyPrawo::Inne => "inne",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyPrawo {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "wlasnosc" => Ok(WpisyPrawo::Wlasnosc),
                    "odrebna_wlasnosc_lokalu" => Ok(WpisyPrawo::OdrebnaWlasnoscLokalu),
                    "spoldzielcze_wlasnosciowe_prawo_do_lokalu" => {
                        Ok(WpisyPrawo::SpoldzielczeWlasnosciowePrawoDoLokalu)
                    }
                    "uzytkowanie_wieczyste" => Ok(WpisyPrawo::UzytkowanieWieczyste),
                    "udzial_we_wspolwlasnosci" => Ok(WpisyPrawo::UdzialWeWspolwlasnosci),
                    "inne" => Ok(WpisyPrawo::Inne),
                    other => Err(format!("{other} is no value of WpisyPrawo")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyPrawo {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `dzialki`.
#[derive(Debug, Clone, PartialEq)]
pub struct DzialkiRow {
    pub id: i64,
    pub nieruchomosc_id: String,
    pub obreb: String,
    pub numer_dzialki: String,
    pub powierzchnia: Option<i64>,
}

impl DzialkiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(DzialkiRow {
            id: row.get("id")?,
            nieruchomosc_id: row.get("nieruchomosc_id")?,
            obreb: row.get("obreb")?,
            numer_dzialki: row.get("numer_dzialki")?,
            powierzchnia: row.get("powierzchnia")?,
        })
    }
}

/// Działki ewidencyjne wszystkich nieruchomości teczki.
pub fn dzialki(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<DzialkiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_nieruchomosci_dzialki
ORDER BY nieruchomosc_id, obreb, numer_dzialki"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(DzialkiRow::from_row).collect()
}

/// The parameters of `wpisz_dzialke`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszDzialkeParams {
    pub nieruchomosc_id: String,
    pub obreb: String,
    pub numer_dzialki: String,
    pub powierzchnia: Option<i64>,
}

/// Działka ewidencyjna nieruchomości.
pub fn wpisz_dzialke(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszDzialkeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_nieruchomosci_dzialki (nieruchomosc_id, obreb, numer_dzialki, powierzchnia)
VALUES (:nieruchomosc_id, :obreb, :numer_dzialki, :powierzchnia)"#,
                parameters: Vec::from([
                    (
                        "nieruchomosc_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.nieruchomosc_id),
                    ),
                    ("obreb", xmeta_room::ToSqlValue::to_sql_value(&params.obreb)),
                    (
                        "numer_dzialki",
                        xmeta_room::ToSqlValue::to_sql_value(&params.numer_dzialki),
                    ),
                    (
                        "powierzchnia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.powierzchnia),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_dzialke`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZmienDzialkeParams {
    pub obreb: String,
    pub numer_dzialki: String,
    pub powierzchnia: Option<i64>,
    pub id: i64,
}

/// Poprawka obrębu, numeru albo powierzchni działki.
pub fn zmien_dzialke(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienDzialkeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_nieruchomosci_dzialki
SET obreb = :obreb,
  numer_dzialki = :numer_dzialki,
  powierzchnia = :powierzchnia
WHERE id = :id"#,
                parameters: Vec::from([
                    ("obreb", xmeta_room::ToSqlValue::to_sql_value(&params.obreb)),
                    (
                        "numer_dzialki",
                        xmeta_room::ToSqlValue::to_sql_value(&params.numer_dzialki),
                    ),
                    (
                        "powierzchnia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.powierzchnia),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wykresl_dzialke`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WykreslDzialkeParams {
    pub id: i64,
}

/// Poprawka pomyłki: działka, która nie należy do nieruchomości (D14).
pub fn wykresl_dzialke(
    room: &impl xmeta_room::MetaRoom,
    params: &WykreslDzialkeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"DELETE FROM sprawa_nieruchomosci_dzialki
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `nieruchomosci`.
#[derive(Debug, Clone, PartialEq)]
pub struct NieruchomosciRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub prawo: WpisyPrawo,
    pub miejscowosc: String,
    pub gmina: String,
    pub ulica: Option<String>,
    pub numer_budynku: Option<String>,
    pub numer_lokalu: Option<String>,
    pub kod_pocztowy: Option<String>,
    pub numer_kw: Option<String>,
    pub kw_brak: bool,
    pub powierzchnia_uzytkowa: Option<i64>,
    pub liczba_pokoi: Option<i64>,
    pub pietro: Option<i64>,
    pub nadrzedna_id: Option<String>,
}

impl NieruchomosciRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(NieruchomosciRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            prawo: row.get("prawo")?,
            miejscowosc: row.get("miejscowosc")?,
            gmina: row.get("gmina")?,
            ulica: row.get("ulica")?,
            numer_budynku: row.get("numer_budynku")?,
            numer_lokalu: row.get("numer_lokalu")?,
            kod_pocztowy: row.get("kod_pocztowy")?,
            numer_kw: row.get("numer_kw")?,
            kw_brak: row.get("kw_brak")?,
            powierzchnia_uzytkowa: row.get("powierzchnia_uzytkowa")?,
            liczba_pokoi: row.get("liczba_pokoi")?,
            pietro: row.get("pietro")?,
            nadrzedna_id: row.get("nadrzedna_id")?,
        })
    }
}

/// Nieruchomości teczki w kolejności wpisania.
pub fn nieruchomosci(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<NieruchomosciRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_nieruchomosci_wpisy
ORDER BY rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(NieruchomosciRow::from_row).collect()
}

/// The parameters of `wpisz_nieruchomosc`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszNieruchomoscParams {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub prawo: WpisyPrawo,
    pub miejscowosc: String,
    pub gmina: String,
    pub ulica: Option<String>,
    pub numer_budynku: Option<String>,
    pub numer_lokalu: Option<String>,
    pub kod_pocztowy: Option<String>,
    pub numer_kw: Option<String>,
    pub kw_brak: bool,
    pub powierzchnia_uzytkowa: Option<i64>,
    pub liczba_pokoi: Option<i64>,
    pub pietro: Option<i64>,
    pub nadrzedna_id: Option<String>,
}

/// Oznaczenie nieruchomości; identyfikator (UUID) nadaje aplikacja, bo wskazują go dokumenty,
/// umowy i pokój wymiany.
pub fn wpisz_nieruchomosc(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszNieruchomoscParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_nieruchomosci_wpisy (id, rodzaj, prawo, miejscowosc, gmina, ulica, numer_budynku,
  numer_lokalu, kod_pocztowy, numer_kw, kw_brak, powierzchnia_uzytkowa, liczba_pokoi, pietro,
  nadrzedna_id)
VALUES (:id, :rodzaj, :prawo, :miejscowosc, :gmina, :ulica, :numer_budynku, :numer_lokalu,
  :kod_pocztowy, :numer_kw, :kw_brak, :powierzchnia_uzytkowa, :liczba_pokoi, :pietro, :nadrzedna_id)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("prawo", xmeta_room::ToSqlValue::to_sql_value(&params.prawo)),
                    (
                        "miejscowosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.miejscowosc),
                    ),
                    ("gmina", xmeta_room::ToSqlValue::to_sql_value(&params.gmina)),
                    ("ulica", xmeta_room::ToSqlValue::to_sql_value(&params.ulica)),
                    (
                        "numer_budynku",
                        xmeta_room::ToSqlValue::to_sql_value(&params.numer_budynku),
                    ),
                    (
                        "numer_lokalu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.numer_lokalu),
                    ),
                    (
                        "kod_pocztowy",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kod_pocztowy),
                    ),
                    ("numer_kw", xmeta_room::ToSqlValue::to_sql_value(&params.numer_kw)),
                    ("kw_brak", xmeta_room::ToSqlValue::to_sql_value(&params.kw_brak)),
                    (
                        "powierzchnia_uzytkowa",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.powierzchnia_uzytkowa,
                        ),
                    ),
                    (
                        "liczba_pokoi",
                        xmeta_room::ToSqlValue::to_sql_value(&params.liczba_pokoi),
                    ),
                    ("pietro", xmeta_room::ToSqlValue::to_sql_value(&params.pietro)),
                    (
                        "nadrzedna_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.nadrzedna_id),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_dane_nieruchomosci`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZmienDaneNieruchomosciParams {
    pub prawo: WpisyPrawo,
    pub miejscowosc: String,
    pub gmina: String,
    pub ulica: Option<String>,
    pub numer_budynku: Option<String>,
    pub numer_lokalu: Option<String>,
    pub kod_pocztowy: Option<String>,
    pub numer_kw: Option<String>,
    pub kw_brak: bool,
    pub powierzchnia_uzytkowa: Option<i64>,
    pub liczba_pokoi: Option<i64>,
    pub pietro: Option<i64>,
    pub nadrzedna_id: Option<String>,
    pub id: String,
}

/// Poprawka oznaczenia, np. po odczytaniu odpisu księgi. Rodzaj się nie zmienia; nieruchomość
/// innego rodzaju to nowy wiersz.
pub fn zmien_dane_nieruchomosci(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienDaneNieruchomosciParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_nieruchomosci_wpisy
SET prawo = :prawo,
  miejscowosc = :miejscowosc,
  gmina = :gmina,
  ulica = :ulica,
  numer_budynku = :numer_budynku,
  numer_lokalu = :numer_lokalu,
  kod_pocztowy = :kod_pocztowy,
  numer_kw = :numer_kw,
  kw_brak = :kw_brak,
  powierzchnia_uzytkowa = :powierzchnia_uzytkowa,
  liczba_pokoi = :liczba_pokoi,
  pietro = :pietro,
  nadrzedna_id = :nadrzedna_id
WHERE id = :id"#,
                parameters: Vec::from([
                    ("prawo", xmeta_room::ToSqlValue::to_sql_value(&params.prawo)),
                    (
                        "miejscowosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.miejscowosc),
                    ),
                    ("gmina", xmeta_room::ToSqlValue::to_sql_value(&params.gmina)),
                    ("ulica", xmeta_room::ToSqlValue::to_sql_value(&params.ulica)),
                    (
                        "numer_budynku",
                        xmeta_room::ToSqlValue::to_sql_value(&params.numer_budynku),
                    ),
                    (
                        "numer_lokalu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.numer_lokalu),
                    ),
                    (
                        "kod_pocztowy",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kod_pocztowy),
                    ),
                    ("numer_kw", xmeta_room::ToSqlValue::to_sql_value(&params.numer_kw)),
                    ("kw_brak", xmeta_room::ToSqlValue::to_sql_value(&params.kw_brak)),
                    (
                        "powierzchnia_uzytkowa",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.powierzchnia_uzytkowa,
                        ),
                    ),
                    (
                        "liczba_pokoi",
                        xmeta_room::ToSqlValue::to_sql_value(&params.liczba_pokoi),
                    ),
                    ("pietro", xmeta_room::ToSqlValue::to_sql_value(&params.pietro)),
                    (
                        "nadrzedna_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.nadrzedna_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
