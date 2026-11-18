//! The kind `kierownik-kontroli-oceny` of the room `positions/kontrola/kierownik_kontroli`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "kierownik-kontroli-oceny";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Projekt oceny ryzyka biura (art. 27 ust. 1 AML): ocena robocza z czynnikami ryzyka dotyczącymi
-- klientów, państw i obszarów geograficznych, produktów, usług, transakcji i kanałów dostaw, każdy
-- z poziomem, środkami ograniczającymi i źródłem.
--
-- Projekt zatwierdza właściciel biura. Zatwierdzona ocena przechodzi do rejestru kontroli; od tej
-- chwili projekt w pamięci tylko wskazuje tamten wpis i się nie zmienia. Kolejny przegląd to nowy
-- projekt.
CREATE TABLE kierownik_kontroli_oceny_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_kontroli_oceny_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  rozpoczeto TEXT NOT NULL,
  -- Ostatni przegląd czynników projektu (przegląd coroczny).
  przejrzano TEXT,
  przekazano_do_zatwierdzenia TEXT,
  -- Zatwierdzona ocena biura w rejestrze kontroli, tekstem.
  rejestr_odnosnik TEXT,

  CONSTRAINT przeglad_nie_przed_rozpoczeciem CHECK (przejrzano IS NULL OR przejrzano >= rozpoczeto),
  CONSTRAINT przekazanie_nie_przed_rozpoczeciem CHECK (
    przekazano_do_zatwierdzenia IS NULL
    OR przekazano_do_zatwierdzenia >= rozpoczeto
  ),
  CONSTRAINT zatwierdzona_po_przekazaniu CHECK (rejestr_odnosnik IS NULL OR przekazano_do_zatwierdzenia IS NOT NULL)
) STRICT;

CREATE TABLE kierownik_kontroli_oceny_czynniki (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_kontroli_oceny_czynniki_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  ocena_id INTEGER NOT NULL REFERENCES kierownik_kontroli_oceny_wpisy (id),
  obszar TEXT NOT NULL CHECK (obszar IN ('klienci', 'obszary_geograficzne', 'produkty', 'uslugi', 'transakcje', 'kanaly_dostaw')),
  czynnik TEXT NOT NULL CHECK (czynnik <> ''),
  poziom TEXT NOT NULL CHECK (poziom IN ('niskie', 'srednie', 'wysokie')),
  srodki TEXT,
  -- Skąd kierownik wie o czynniku, np. krajowa ocena ryzyka albo raport Kontrolera.
  zrodlo TEXT NOT NULL CHECK (zrodlo <> '')
) STRICT;

CREATE TRIGGER kierownik_kontroli_oceny_powstaje_robocza
BEFORE INSERT ON kierownik_kontroli_oceny_wpisy
WHEN NEW.przekazano_do_zatwierdzenia IS NOT NULL
  OR NEW.rejestr_odnosnik IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'projekt oceny powstaje roboczy; przekazanie i zatwierdzenie przychodzą później');
END;

-- Zatwierdzony projekt tylko wskazuje wpis w rejestrze, więc się go nie zmienia, także jego
-- czynników.
CREATE TRIGGER kierownik_kontroli_oceny_zatwierdzona_bez_zmian
BEFORE UPDATE ON kierownik_kontroli_oceny_wpisy
WHEN OLD.rejestr_odnosnik IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.rozpoczeto IS NOT OLD.rozpoczeto
BEGIN
  SELECT RAISE(ABORT, 'projektu oceny zatwierdzonej w rejestrze kontroli się nie zmienia');
END;

CREATE TRIGGER kierownik_kontroli_oceny_czynnik_dopisany_do_zatwierdzonej
BEFORE INSERT ON kierownik_kontroli_oceny_czynniki
WHEN EXISTS (
  SELECT 1
  FROM kierownik_kontroli_oceny_wpisy AS ocena
  WHERE ocena.id = NEW.ocena_id
    AND ocena.rejestr_odnosnik IS NOT NULL
)
BEGIN
  SELECT RAISE(ABORT, 'projektu oceny zatwierdzonej w rejestrze kontroli się nie zmienia');
END;

CREATE TRIGGER kierownik_kontroli_oceny_czynnik_zatwierdzonej_bez_zmian
BEFORE UPDATE ON kierownik_kontroli_oceny_czynniki
WHEN NEW.id IS NOT OLD.id
  OR NEW.ocena_id IS NOT OLD.ocena_id
  OR EXISTS (
    SELECT 1
    FROM kierownik_kontroli_oceny_wpisy AS ocena
    WHERE ocena.id = OLD.ocena_id
      AND ocena.rejestr_odnosnik IS NOT NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'projektu oceny zatwierdzonej w rejestrze kontroli się nie zmienia, a czynnik nie przechodzi do innego projektu');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_kontroli_oceny_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_kontroli_oceny_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_kontroli_oceny_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER kierownik_kontroli_oceny_czynniki_bez_zastepowania
BEFORE INSERT ON kierownik_kontroli_oceny_czynniki
WHEN EXISTS (
  SELECT 1
  FROM kierownik_kontroli_oceny_czynniki
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: dawne projekty zostają jako historia oceny ryzyka biura.
CREATE TRIGGER kierownik_kontroli_oceny_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_kontroli_oceny_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER kierownik_kontroli_oceny_czynniki_bez_usuwania
BEFORE DELETE ON kierownik_kontroli_oceny_czynniki
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
];

/// The values of column `czynniki.obszar`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CzynnikiObszar {
    Klienci,
    ObszaryGeograficzne,
    Produkty,
    Uslugi,
    Transakcje,
    KanalyDostaw,
}

impl CzynnikiObszar {
    /// Every value, in the order of the catalog.
    pub const ALL: &[CzynnikiObszar] = &[
        CzynnikiObszar::Klienci,
        CzynnikiObszar::ObszaryGeograficzne,
        CzynnikiObszar::Produkty,
        CzynnikiObszar::Uslugi,
        CzynnikiObszar::Transakcje,
        CzynnikiObszar::KanalyDostaw,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            CzynnikiObszar::Klienci => "klienci",
            CzynnikiObszar::ObszaryGeograficzne => "obszary_geograficzne",
            CzynnikiObszar::Produkty => "produkty",
            CzynnikiObszar::Uslugi => "uslugi",
            CzynnikiObszar::Transakcje => "transakcje",
            CzynnikiObszar::KanalyDostaw => "kanaly_dostaw",
        }
    }
}

impl xmeta_room::FromSqlValue for CzynnikiObszar {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "klienci" => Ok(CzynnikiObszar::Klienci),
                    "obszary_geograficzne" => Ok(CzynnikiObszar::ObszaryGeograficzne),
                    "produkty" => Ok(CzynnikiObszar::Produkty),
                    "uslugi" => Ok(CzynnikiObszar::Uslugi),
                    "transakcje" => Ok(CzynnikiObszar::Transakcje),
                    "kanaly_dostaw" => Ok(CzynnikiObszar::KanalyDostaw),
                    other => Err(format!("{other} is no value of CzynnikiObszar")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for CzynnikiObszar {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `czynniki.poziom`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CzynnikiPoziom {
    Niskie,
    Srednie,
    Wysokie,
}

impl CzynnikiPoziom {
    /// Every value, in the order of the catalog.
    pub const ALL: &[CzynnikiPoziom] = &[
        CzynnikiPoziom::Niskie,
        CzynnikiPoziom::Srednie,
        CzynnikiPoziom::Wysokie,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            CzynnikiPoziom::Niskie => "niskie",
            CzynnikiPoziom::Srednie => "srednie",
            CzynnikiPoziom::Wysokie => "wysokie",
        }
    }
}

impl xmeta_room::FromSqlValue for CzynnikiPoziom {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "niskie" => Ok(CzynnikiPoziom::Niskie),
                    "srednie" => Ok(CzynnikiPoziom::Srednie),
                    "wysokie" => Ok(CzynnikiPoziom::Wysokie),
                    other => Err(format!("{other} is no value of CzynnikiPoziom")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for CzynnikiPoziom {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The parameters of `czynniki_oceny`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CzynnikiOcenyParams {
    pub ocena_id: i64,
}

/// A row of `czynniki_oceny`.
#[derive(Debug, Clone, PartialEq)]
pub struct CzynnikiOcenyRow {
    pub id: i64,
    pub ocena_id: i64,
    pub obszar: CzynnikiObszar,
    pub czynnik: String,
    pub poziom: CzynnikiPoziom,
    pub srodki: Option<String>,
    pub zrodlo: String,
}

impl CzynnikiOcenyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(CzynnikiOcenyRow {
            id: row.get("id")?,
            ocena_id: row.get("ocena_id")?,
            obszar: row.get("obszar")?,
            czynnik: row.get("czynnik")?,
            poziom: row.get("poziom")?,
            srodki: row.get("srodki")?,
            zrodlo: row.get("zrodlo")?,
        })
    }
}

/// Czynniki jednego projektu według obszaru.
pub fn czynniki_oceny(
    room: &impl xmeta_room::MetaRoom,
    params: &CzynnikiOcenyParams,
) -> Result<Vec<CzynnikiOcenyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_kontroli_oceny_czynniki
WHERE ocena_id = :ocena_id
ORDER BY obszar, id"#,
        parameters: Vec::from([
            ("ocena_id", xmeta_room::ToSqlValue::to_sql_value(&params.ocena_id)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(CzynnikiOcenyRow::from_row).collect()
}

/// The parameters of `dopisz_czynnik`.
#[derive(Debug, Clone, PartialEq)]
pub struct DopiszCzynnikParams {
    pub ocena_id: i64,
    pub obszar: CzynnikiObszar,
    pub czynnik: String,
    pub poziom: CzynnikiPoziom,
    pub srodki: Option<String>,
    pub zrodlo: String,
}

/// Nowy czynnik ryzyka w projekcie roboczym.
pub fn dopisz_czynnik(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszCzynnikParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_kontroli_oceny_czynniki (ocena_id, obszar, czynnik, poziom, srodki, zrodlo)
VALUES (:ocena_id, :obszar, :czynnik, :poziom, :srodki, :zrodlo)"#,
                parameters: Vec::from([
                    ("ocena_id", xmeta_room::ToSqlValue::to_sql_value(&params.ocena_id)),
                    ("obszar", xmeta_room::ToSqlValue::to_sql_value(&params.obszar)),
                    ("czynnik", xmeta_room::ToSqlValue::to_sql_value(&params.czynnik)),
                    ("poziom", xmeta_room::ToSqlValue::to_sql_value(&params.poziom)),
                    ("srodki", xmeta_room::ToSqlValue::to_sql_value(&params.srodki)),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_czynnik`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZmienCzynnikParams {
    pub obszar: CzynnikiObszar,
    pub czynnik: String,
    pub poziom: CzynnikiPoziom,
    pub srodki: Option<String>,
    pub zrodlo: String,
    pub id: i64,
}

/// Nowy opis, poziom, środki albo źródło czynnika w projekcie roboczym.
pub fn zmien_czynnik(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienCzynnikParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_kontroli_oceny_czynniki
SET obszar = :obszar,
  czynnik = :czynnik,
  poziom = :poziom,
  srodki = :srodki,
  zrodlo = :zrodlo
WHERE id = :id"#,
                parameters: Vec::from([
                    ("obszar", xmeta_room::ToSqlValue::to_sql_value(&params.obszar)),
                    ("czynnik", xmeta_room::ToSqlValue::to_sql_value(&params.czynnik)),
                    ("poziom", xmeta_room::ToSqlValue::to_sql_value(&params.poziom)),
                    ("srodki", xmeta_room::ToSqlValue::to_sql_value(&params.srodki)),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `projekty_ocen`.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjektyOcenRow {
    pub id: i64,
    pub rozpoczeto: String,
    pub przejrzano: Option<String>,
    pub przekazano_do_zatwierdzenia: Option<String>,
    pub rejestr_odnosnik: Option<String>,
}

impl ProjektyOcenRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ProjektyOcenRow {
            id: row.get("id")?,
            rozpoczeto: row.get("rozpoczeto")?,
            przejrzano: row.get("przejrzano")?,
            przekazano_do_zatwierdzenia: row.get("przekazano_do_zatwierdzenia")?,
            rejestr_odnosnik: row.get("rejestr_odnosnik")?,
        })
    }
}

/// Projekty oceny ryzyka biura, najnowsze pierwsze.
pub fn projekty_ocen(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ProjektyOcenRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_kontroli_oceny_wpisy
ORDER BY rozpoczeto DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(ProjektyOcenRow::from_row).collect()
}

/// The parameters of `rozpocznij_projekt_oceny`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RozpocznijProjektOcenyParams {
    pub rozpoczeto: String,
}

/// Nowy projekt oceny ryzyka biura, np. przy corocznym przeglądzie.
pub fn rozpocznij_projekt_oceny(
    room: &impl xmeta_room::MetaRoom,
    params: &RozpocznijProjektOcenyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_kontroli_oceny_wpisy (rozpoczeto)
VALUES (:rozpoczeto)"#,
                parameters: Vec::from([
                    (
                        "rozpoczeto",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rozpoczeto),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_przeglad_oceny`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPrzegladOcenyParams {
    pub przejrzano: String,
    pub id: i64,
}

/// Kierownik przejrzał czynniki projektu.
pub fn odnotuj_przeglad_oceny(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPrzegladOcenyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_kontroli_oceny_wpisy
SET przejrzano = :przejrzano
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "przejrzano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przejrzano),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `przekaz_ocene_do_zatwierdzenia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrzekazOceneDoZatwierdzeniaParams {
    pub przekazano_do_zatwierdzenia: String,
    pub id: i64,
}

/// Projekt trafił do właściciela biura do zatwierdzenia.
pub fn przekaz_ocene_do_zatwierdzenia(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzekazOceneDoZatwierdzeniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_kontroli_oceny_wpisy
SET przekazano_do_zatwierdzenia = :przekazano_do_zatwierdzenia
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "przekazano_do_zatwierdzenia",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.przekazano_do_zatwierdzenia,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_zatwierdzenie_oceny`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujZatwierdzenieOcenyParams {
    pub rejestr_odnosnik: String,
    pub id: i64,
}

/// Zatwierdzona ocena jest w rejestrze kontroli; projekt odtąd tylko ją wskazuje.
pub fn odnotuj_zatwierdzenie_oceny(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujZatwierdzenieOcenyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_kontroli_oceny_wpisy
SET rejestr_odnosnik = :rejestr_odnosnik
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "rejestr_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rejestr_odnosnik),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
