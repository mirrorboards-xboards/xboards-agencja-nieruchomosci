//! The kind `kierownik-dzialu-standardy` of the room `positions/posrednictwo/kierownik_posrednictwa`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "kierownik-dzialu-standardy",
    requires: &[],
    migrations: &[
        r#"-- Standardy działu: kolejne wersje standardów, wzorów i parametrów programu, które zaproponował
-- kierownik, i rozstrzygnięcie człowieka. Dział ma kilka standardów, np. wzór umowy pośrednictwa
-- i standard ogłoszenia; z każdego obowiązuje najnowsza zatwierdzona wersja.
--
-- Tekst wersji leży w repozytorium XGit pokoju pamięci; wiersz trzyma ścieżkę i skrót, a podpis
-- rozstrzygnięcia obejmuje ten skrót. Rozstrzyga właściciel kierownika, a standardy Kontroli
-- właściciel biura, którego podpis przychodzi wiadomością. Baza wymaga kompletu kolumn podpisu,
-- a sam podpis sprawdza aplikacja.
CREATE TABLE kierownik_dzialu_standardy_wersje (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_dzialu_standardy_wersje_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Np. „wzor_umowy_posrednictwa”, „standard_ogloszenia”, „parametry_programu”.
  standard TEXT NOT NULL CHECK (standard <> ''),
  numer INTEGER NOT NULL CHECK (numer > 0),
  sciezka TEXT NOT NULL,
  skrot TEXT NOT NULL,
  opis_zmian TEXT NOT NULL,
  zaproponowano TEXT NOT NULL,
  rozstrzygniecie TEXT CHECK (rozstrzygniecie IN ('zatwierdzona', 'odrzucona')),
  podpisal TEXT,
  podpisal_jako TEXT CHECK (podpisal_jako IN ('wlasciciel', 'wlasciciel_biura')),
  podpis TEXT,
  podpisano TEXT,

  CONSTRAINT numer_wersji_raz_w_standardzie UNIQUE (standard, numer),
  CONSTRAINT podpis_w_komplecie CHECK (
    (podpisal IS NULL) = (podpisal_jako IS NULL)
    AND (podpisal IS NULL) = (podpis IS NULL)
    AND (podpisal IS NULL) = (podpisano IS NULL)
  ),
  CONSTRAINT rozstrzygniecie_z_podpisem CHECK ((rozstrzygniecie IS NULL) = (podpisal IS NULL)),
  CONSTRAINT rozstrzygniecie_nie_przed_propozycja CHECK (podpisano IS NULL OR podpisano >= zaproponowano)
) STRICT;

CREATE TRIGGER kierownik_dzialu_standardy_powstaje_jako_propozycja
BEFORE INSERT ON kierownik_dzialu_standardy_wersje
WHEN NEW.rozstrzygniecie IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'wersja standardu powstaje jako propozycja; rozstrzyga ją człowiek');
END;

-- Podpis obejmuje skrót tekstu, więc rozstrzygniętej wersji nie zmienia się wcale, a propozycja
-- zmienia się tylko rozstrzygnięciem.
CREATE TRIGGER kierownik_dzialu_standardy_rozstrzygniecie_jest_ostateczne
BEFORE UPDATE ON kierownik_dzialu_standardy_wersje
WHEN OLD.rozstrzygniecie IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.standard IS NOT OLD.standard
  OR NEW.numer IS NOT OLD.numer
  OR NEW.sciezka IS NOT OLD.sciezka
  OR NEW.skrot IS NOT OLD.skrot
  OR NEW.opis_zmian IS NOT OLD.opis_zmian
  OR NEW.zaproponowano IS NOT OLD.zaproponowano
BEGIN
  SELECT RAISE(ABORT, 'rozstrzygniętej wersji standardu się nie zmienia, a propozycję zmienia tylko rozstrzygnięcie');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers; także wiersz o tym samym
-- numerze wersji standardu. Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego
-- wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_dzialu_standardy_wersje_bez_zastepowania
BEFORE INSERT ON kierownik_dzialu_standardy_wersje
WHEN EXISTS (
  SELECT 1
  FROM kierownik_dzialu_standardy_wersje
  WHERE id = NEW.id
)
  OR EXISTS (
    SELECT 1
    FROM kierownik_dzialu_standardy_wersje
    WHERE standard = NEW.standard
      AND numer = NEW.numer
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: odrzucona i dawna wersja zostają jako historia standardu.
CREATE TRIGGER kierownik_dzialu_standardy_wersje_bez_usuwania
BEFORE DELETE ON kierownik_dzialu_standardy_wersje
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
    ],
};

/// The values of column `wersje.rozstrzygniecie`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WersjeRozstrzygniecie {
    Zatwierdzona,
    Odrzucona,
}

impl WersjeRozstrzygniecie {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WersjeRozstrzygniecie] = &[
        WersjeRozstrzygniecie::Zatwierdzona,
        WersjeRozstrzygniecie::Odrzucona,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WersjeRozstrzygniecie::Zatwierdzona => "zatwierdzona",
            WersjeRozstrzygniecie::Odrzucona => "odrzucona",
        }
    }
}

impl xmeta_room::FromSqlValue for WersjeRozstrzygniecie {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "zatwierdzona" => Ok(WersjeRozstrzygniecie::Zatwierdzona),
                    "odrzucona" => Ok(WersjeRozstrzygniecie::Odrzucona),
                    other => Err(format!("{other} is no value of WersjeRozstrzygniecie")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WersjeRozstrzygniecie {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wersje.podpisal_jako`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WersjePodpisalJako {
    Wlasciciel,
    WlascicielBiura,
}

impl WersjePodpisalJako {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WersjePodpisalJako] = &[
        WersjePodpisalJako::Wlasciciel,
        WersjePodpisalJako::WlascicielBiura,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WersjePodpisalJako::Wlasciciel => "wlasciciel",
            WersjePodpisalJako::WlascicielBiura => "wlasciciel_biura",
        }
    }
}

impl xmeta_room::FromSqlValue for WersjePodpisalJako {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "wlasciciel" => Ok(WersjePodpisalJako::Wlasciciel),
                    "wlasciciel_biura" => Ok(WersjePodpisalJako::WlascicielBiura),
                    other => Err(format!("{other} is no value of WersjePodpisalJako")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WersjePodpisalJako {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The parameters of `wersje_standardu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WersjeStandarduParams {
    pub standard: String,
}

/// A row of `wersje_standardu`.
#[derive(Debug, Clone, PartialEq)]
pub struct WersjeStandarduRow {
    pub id: i64,
    pub standard: String,
    pub numer: i64,
    pub sciezka: String,
    pub skrot: String,
    pub opis_zmian: String,
    pub zaproponowano: String,
    pub rozstrzygniecie: Option<WersjeRozstrzygniecie>,
    pub podpisal: Option<String>,
    pub podpisal_jako: Option<WersjePodpisalJako>,
    pub podpis: Option<String>,
    pub podpisano: Option<String>,
}

impl WersjeStandarduRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WersjeStandarduRow {
            id: row.get("id")?,
            standard: row.get("standard")?,
            numer: row.get("numer")?,
            sciezka: row.get("sciezka")?,
            skrot: row.get("skrot")?,
            opis_zmian: row.get("opis_zmian")?,
            zaproponowano: row.get("zaproponowano")?,
            rozstrzygniecie: row.get("rozstrzygniecie")?,
            podpisal: row.get("podpisal")?,
            podpisal_jako: row.get("podpisal_jako")?,
            podpis: row.get("podpis")?,
            podpisano: row.get("podpisano")?,
        })
    }
}

/// Wszystkie wersje jednego standardu działu, od najnowszej.
pub fn wersje_standardu(
    room: &impl xmeta_room::MetaRoom,
    params: &WersjeStandarduParams,
) -> Result<Vec<WersjeStandarduRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_dzialu_standardy_wersje
WHERE standard = :standard
ORDER BY numer DESC"#,
        parameters: Vec::from([
            ("standard", xmeta_room::ToSqlValue::to_sql_value(&params.standard)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(WersjeStandarduRow::from_row).collect()
}

/// A row of `obowiazujace_standardy`.
#[derive(Debug, Clone, PartialEq)]
pub struct ObowiazujaceStandardyRow {
    pub id: i64,
    pub standard: String,
    pub numer: i64,
    pub sciezka: String,
    pub skrot: String,
    pub opis_zmian: String,
    pub zaproponowano: String,
    pub rozstrzygniecie: Option<WersjeRozstrzygniecie>,
    pub podpisal: Option<String>,
    pub podpisal_jako: Option<WersjePodpisalJako>,
    pub podpis: Option<String>,
    pub podpisano: Option<String>,
}

impl ObowiazujaceStandardyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ObowiazujaceStandardyRow {
            id: row.get("id")?,
            standard: row.get("standard")?,
            numer: row.get("numer")?,
            sciezka: row.get("sciezka")?,
            skrot: row.get("skrot")?,
            opis_zmian: row.get("opis_zmian")?,
            zaproponowano: row.get("zaproponowano")?,
            rozstrzygniecie: row.get("rozstrzygniecie")?,
            podpisal: row.get("podpisal")?,
            podpisal_jako: row.get("podpisal_jako")?,
            podpis: row.get("podpis")?,
            podpisano: row.get("podpisano")?,
        })
    }
}

/// Najnowsza zatwierdzona wersja każdego standardu działu; to one obowiązują.
pub fn obowiazujace_standardy(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ObowiazujaceStandardyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_dzialu_standardy_wersje AS wersja
WHERE wersja.rozstrzygniecie = 'zatwierdzona'
  AND NOT EXISTS (
    SELECT 1
    FROM kierownik_dzialu_standardy_wersje AS nowsza
    WHERE nowsza.standard = wersja.standard
      AND nowsza.rozstrzygniecie = 'zatwierdzona'
      AND nowsza.numer > wersja.numer
  )
ORDER BY wersja.standard"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ObowiazujaceStandardyRow::from_row).collect()
}

/// The parameters of `zaproponuj_standard`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZaproponujStandardParams {
    pub standard: String,
    pub numer: i64,
    pub sciezka: String,
    pub skrot: String,
    pub opis_zmian: String,
    pub zaproponowano: String,
}

/// Nowa wersja standardu z plikiem w XGit pokoju pamięci, czekająca na rozstrzygnięcie człowieka.
pub fn zaproponuj_standard(
    room: &impl xmeta_room::MetaRoom,
    params: &ZaproponujStandardParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_dzialu_standardy_wersje (standard, numer, sciezka, skrot, opis_zmian, zaproponowano)
VALUES (:standard, :numer, :sciezka, :skrot, :opis_zmian, :zaproponowano)"#,
                parameters: Vec::from([
                    ("standard", xmeta_room::ToSqlValue::to_sql_value(&params.standard)),
                    ("numer", xmeta_room::ToSqlValue::to_sql_value(&params.numer)),
                    ("sciezka", xmeta_room::ToSqlValue::to_sql_value(&params.sciezka)),
                    ("skrot", xmeta_room::ToSqlValue::to_sql_value(&params.skrot)),
                    (
                        "opis_zmian",
                        xmeta_room::ToSqlValue::to_sql_value(&params.opis_zmian),
                    ),
                    (
                        "zaproponowano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zaproponowano),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `rozstrzygnij_standard`.
#[derive(Debug, Clone, PartialEq)]
pub struct RozstrzygnijStandardParams {
    pub rozstrzygniecie: WersjeRozstrzygniecie,
    pub podpisal: String,
    pub podpisal_jako: WersjePodpisalJako,
    pub podpis: String,
    pub podpisano: String,
    pub id: i64,
}

/// Zatwierdzenie albo odrzucenie wersji przez właściciela kierownika albo, w Kontroli, właściciela
/// biura, z podpisem obejmującym skrót pliku.
pub fn rozstrzygnij_standard(
    room: &impl xmeta_room::MetaRoom,
    params: &RozstrzygnijStandardParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_dzialu_standardy_wersje
SET rozstrzygniecie = :rozstrzygniecie,
  podpisal = :podpisal,
  podpisal_jako = :podpisal_jako,
  podpis = :podpis,
  podpisano = :podpisano
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "rozstrzygniecie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rozstrzygniecie),
                    ),
                    ("podpisal", xmeta_room::ToSqlValue::to_sql_value(&params.podpisal)),
                    (
                        "podpisal_jako",
                        xmeta_room::ToSqlValue::to_sql_value(&params.podpisal_jako),
                    ),
                    ("podpis", xmeta_room::ToSqlValue::to_sql_value(&params.podpis)),
                    (
                        "podpisano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.podpisano),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
