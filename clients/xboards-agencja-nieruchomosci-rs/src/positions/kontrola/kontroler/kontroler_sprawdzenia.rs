//! The kind `kontroler-sprawdzenia` of the room `positions/kontrola/kontroler`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "kontroler-sprawdzenia",
    requires: &[&super::stanowisko_zrodla::KIND],
    migrations: &[
        r#"-- Sprawdzenia: każde sprawdzenie Kontrolera z przeciwdziałania praniu pieniędzy osobno, np. tożsamość
-- klientki przed umową pośrednictwa, beneficjenci spółki kupującej albo lista sankcyjna. Leżą
-- w pamięci, a nie w teczce, bo teczkę czytają agenci, którzy rozmawiają z klientami. Sprawdzenia
-- spoza przeciwdziałania praniu pieniędzy (rachunek, tytuł, pełnomocnictwo) leżą w teczce.
--
-- Sprawdzenie robocze czeka na przekazanie Kierownikowi kontroli. Przekazanego sprawdzenia się nie
-- zmienia. Gdy Kierownik kontroli przeniesie je do rejestru kontroli i poda wpis, Kontroler czyści
-- opis, czynniki i proponowany poziom (D13): zostają rodzaj, źródło z datą, wynik i odnośnik do
-- rejestru, a szczegóły leżą już tylko w rejestrze. Ponowne sprawdzenie jest nowym wierszem,
-- a poprzednie wskazuje je jako zastępujące.
CREATE TABLE kontroler_sprawdzenia_wpisy (
  id TEXT PRIMARY KEY,
  -- Rodzaje sprawdzeń rejestru kontroli i ocena ryzyka klienta.
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('identyfikacja', 'weryfikacja', 'beneficjenci', 'crbr', 'pep', 'sankcje', 'cel_stosunkow', 'zrodlo_srodkow', 'monitorowanie', 'ocena_ryzyka')),
  -- Teczka klienta, w której zaszło zdarzenie ze sprawdzeniem, tekstem.
  teczka_odnosnik TEXT NOT NULL CHECK (teczka_odnosnik <> ''),
  -- Sprawdzany wiersz teczki, np. uczestnik albo pełnomocnik: tabela i pokój z wierszem, tekstem.
  dotyczy_tabela TEXT,
  dotyczy_odnosnik TEXT,
  -- Źródło spoza teczki (np. RDO, KRS, CRBR, lista sankcyjna) z datą, w której je sprawdzono.
  zrodlo TEXT,
  zrodlo_data TEXT,
  -- Osoba albo instytucja, której oświadczenie sprawdzenie porównuje ze źródłem.
  zrodlo_id TEXT REFERENCES stanowisko_zrodla_wpisy (id),
  wynik TEXT NOT NULL CHECK (wynik IN ('zgodne', 'uwagi', 'nie_mozna_ustalic')),
  opis TEXT,
  -- Poziomy jak w ocenach rejestru kontroli.
  proponowany_poziom TEXT CHECK (proponowany_poziom IN ('niskie', 'srednie', 'wysokie')),
  czynniki TEXT,
  -- 1, gdy sprawdzenie jest powodem wątpliwości (znacznika w teczce).
  powod_watpliwosci INTEGER NOT NULL CHECK (powod_watpliwosci IN (0, 1)),
  sprawdzono TEXT NOT NULL,
  przekazano TEXT,
  -- Wpis w rejestrze kontroli, tekstem.
  rejestr_odnosnik TEXT,
  zastapione_przez_id TEXT REFERENCES kontroler_sprawdzenia_wpisy (id),

  CONSTRAINT dotyczy_tabela_razem_z_odnosnikiem CHECK ((dotyczy_tabela IS NULL) = (dotyczy_odnosnik IS NULL)),
  CONSTRAINT zrodlo_z_data CHECK ((zrodlo IS NULL) = (zrodlo_data IS NULL)),
  -- Przy identyfikacji, weryfikacji i beneficjentach zapisuje się źródło i datę.
  CONSTRAINT identyfikacja_ze_zrodlem_i_data CHECK (
    rodzaj NOT IN ('identyfikacja', 'weryfikacja', 'beneficjenci')
    OR zrodlo IS NOT NULL
  ),
  CONSTRAINT poziom_i_czynniki_tylko_przy_ocenie_ryzyka CHECK (
    rodzaj = 'ocena_ryzyka'
    OR (proponowany_poziom IS NULL AND czynniki IS NULL)
  ),
  -- Do wpisu w rejestrze ocena ryzyka niesie poziom i czynniki, a uwagi i brak ustalenia opis.
  CONSTRAINT ocena_ryzyka_z_poziomem_i_czynnikami CHECK (
    rodzaj <> 'ocena_ryzyka'
    OR rejestr_odnosnik IS NOT NULL
    OR (proponowany_poziom IS NOT NULL AND czynniki IS NOT NULL)
  ),
  CONSTRAINT uwagi_z_opisem CHECK (wynik = 'zgodne' OR rejestr_odnosnik IS NOT NULL OR opis IS NOT NULL),
  CONSTRAINT wpis_w_rejestrze_po_przekazaniu CHECK (rejestr_odnosnik IS NULL OR przekazano IS NOT NULL),
  CONSTRAINT przekazanie_nie_przed_sprawdzeniem CHECK (przekazano IS NULL OR przekazano >= sprawdzono),
  -- D13: szczegóły sprawdzenia po wpisie w rejestrze leżą tylko w rejestrze.
  CONSTRAINT po_wpisie_w_rejestrze_bez_szczegolow CHECK (
    rejestr_odnosnik IS NULL
    OR (opis IS NULL AND czynniki IS NULL AND proponowany_poziom IS NULL)
  ),
  CONSTRAINT zastapione_nie_przez_siebie CHECK (zastapione_przez_id IS NULL OR zastapione_przez_id <> id)
) STRICT;

-- Przekazanie, wpis w rejestrze i zastąpienie przychodzą później, więc sprawdzenie powstaje robocze.
CREATE TRIGGER kontroler_sprawdzenia_powstaje_robocze
BEFORE INSERT ON kontroler_sprawdzenia_wpisy
WHEN NEW.przekazano IS NOT NULL
  OR NEW.rejestr_odnosnik IS NOT NULL
  OR NEW.zastapione_przez_id IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'sprawdzenie powstaje robocze: bez przekazania, wpisu w rejestrze i zastąpienia');
END;

-- Po przekazaniu zmienia się tylko to, co przynosi rejestr: odnośnik do wpisu (raz) i razem z nim
-- wyczyszczone szczegóły, a poza tym zastąpienie przez ponowne sprawdzenie.
CREATE TRIGGER kontroler_sprawdzenia_przekazane_bez_zmian
BEFORE UPDATE ON kontroler_sprawdzenia_wpisy
WHEN OLD.przekazano IS NOT NULL
  AND (
    NEW.id IS NOT OLD.id
    OR NEW.rodzaj IS NOT OLD.rodzaj
    OR NEW.teczka_odnosnik IS NOT OLD.teczka_odnosnik
    OR NEW.dotyczy_tabela IS NOT OLD.dotyczy_tabela
    OR NEW.dotyczy_odnosnik IS NOT OLD.dotyczy_odnosnik
    OR NEW.zrodlo IS NOT OLD.zrodlo
    OR NEW.zrodlo_data IS NOT OLD.zrodlo_data
    OR NEW.zrodlo_id IS NOT OLD.zrodlo_id
    OR NEW.wynik IS NOT OLD.wynik
    OR NEW.powod_watpliwosci IS NOT OLD.powod_watpliwosci
    OR NEW.sprawdzono IS NOT OLD.sprawdzono
    OR NEW.przekazano IS NOT OLD.przekazano
    OR (OLD.rejestr_odnosnik IS NOT NULL AND NEW.rejestr_odnosnik IS NOT OLD.rejestr_odnosnik)
    OR (
      NEW.rejestr_odnosnik IS NULL
      AND (
        NEW.opis IS NOT OLD.opis
        OR NEW.czynniki IS NOT OLD.czynniki
        OR NEW.proponowany_poziom IS NOT OLD.proponowany_poziom
      )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'przekazanego sprawdzenia się nie zmienia; ponowne sprawdzenie to nowy wiersz');
END;

CREATE TRIGGER kontroler_sprawdzenia_zastapione_raz
BEFORE UPDATE OF zastapione_przez_id ON kontroler_sprawdzenia_wpisy
WHEN OLD.zastapione_przez_id IS NOT NULL
  AND NEW.zastapione_przez_id IS NOT OLD.zastapione_przez_id
BEGIN
  SELECT RAISE(ABORT, 'zastąpionego sprawdzenia nie zastępuje się drugi raz');
END;

-- Ponowne sprawdzenie dotyczy tego samego rodzaju i nie jest starsze od tego, które zastępuje.
CREATE TRIGGER kontroler_sprawdzenia_zastepuje_ponowne
BEFORE UPDATE OF zastapione_przez_id ON kontroler_sprawdzenia_wpisy
WHEN NEW.zastapione_przez_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM kontroler_sprawdzenia_wpisy AS ponowne
    WHERE ponowne.id = NEW.zastapione_przez_id
      AND ponowne.rodzaj = OLD.rodzaj
      AND ponowne.sprawdzono >= OLD.sprawdzono
  )
BEGIN
  SELECT RAISE(ABORT, 'sprawdzenie zastępuje ponowne sprawdzenie tego samego rodzaju, nie starsze');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER kontroler_sprawdzenia_wpisy_bez_zastepowania
BEFORE INSERT ON kontroler_sprawdzenia_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kontroler_sprawdzenia_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: sprawdzenie zostaje jako ślad tego, co i w jakim źródle Kontroler sprawdził.
CREATE TRIGGER kontroler_sprawdzenia_wpisy_bez_usuwania
BEFORE DELETE ON kontroler_sprawdzenia_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
    ],
};

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    Identyfikacja,
    Weryfikacja,
    Beneficjenci,
    Crbr,
    Pep,
    Sankcje,
    CelStosunkow,
    ZrodloSrodkow,
    Monitorowanie,
    OcenaRyzyka,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::Identyfikacja,
        WpisyRodzaj::Weryfikacja,
        WpisyRodzaj::Beneficjenci,
        WpisyRodzaj::Crbr,
        WpisyRodzaj::Pep,
        WpisyRodzaj::Sankcje,
        WpisyRodzaj::CelStosunkow,
        WpisyRodzaj::ZrodloSrodkow,
        WpisyRodzaj::Monitorowanie,
        WpisyRodzaj::OcenaRyzyka,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::Identyfikacja => "identyfikacja",
            WpisyRodzaj::Weryfikacja => "weryfikacja",
            WpisyRodzaj::Beneficjenci => "beneficjenci",
            WpisyRodzaj::Crbr => "crbr",
            WpisyRodzaj::Pep => "pep",
            WpisyRodzaj::Sankcje => "sankcje",
            WpisyRodzaj::CelStosunkow => "cel_stosunkow",
            WpisyRodzaj::ZrodloSrodkow => "zrodlo_srodkow",
            WpisyRodzaj::Monitorowanie => "monitorowanie",
            WpisyRodzaj::OcenaRyzyka => "ocena_ryzyka",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "identyfikacja" => Ok(WpisyRodzaj::Identyfikacja),
                    "weryfikacja" => Ok(WpisyRodzaj::Weryfikacja),
                    "beneficjenci" => Ok(WpisyRodzaj::Beneficjenci),
                    "crbr" => Ok(WpisyRodzaj::Crbr),
                    "pep" => Ok(WpisyRodzaj::Pep),
                    "sankcje" => Ok(WpisyRodzaj::Sankcje),
                    "cel_stosunkow" => Ok(WpisyRodzaj::CelStosunkow),
                    "zrodlo_srodkow" => Ok(WpisyRodzaj::ZrodloSrodkow),
                    "monitorowanie" => Ok(WpisyRodzaj::Monitorowanie),
                    "ocena_ryzyka" => Ok(WpisyRodzaj::OcenaRyzyka),
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

/// The values of column `wpisy.wynik`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyWynik {
    Zgodne,
    Uwagi,
    NieMoznaUstalic,
}

impl WpisyWynik {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyWynik] = &[
        WpisyWynik::Zgodne,
        WpisyWynik::Uwagi,
        WpisyWynik::NieMoznaUstalic,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyWynik::Zgodne => "zgodne",
            WpisyWynik::Uwagi => "uwagi",
            WpisyWynik::NieMoznaUstalic => "nie_mozna_ustalic",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyWynik {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "zgodne" => Ok(WpisyWynik::Zgodne),
                    "uwagi" => Ok(WpisyWynik::Uwagi),
                    "nie_mozna_ustalic" => Ok(WpisyWynik::NieMoznaUstalic),
                    other => Err(format!("{other} is no value of WpisyWynik")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyWynik {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.proponowany_poziom`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyProponowanyPoziom {
    Niskie,
    Srednie,
    Wysokie,
}

impl WpisyProponowanyPoziom {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyProponowanyPoziom] = &[
        WpisyProponowanyPoziom::Niskie,
        WpisyProponowanyPoziom::Srednie,
        WpisyProponowanyPoziom::Wysokie,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyProponowanyPoziom::Niskie => "niskie",
            WpisyProponowanyPoziom::Srednie => "srednie",
            WpisyProponowanyPoziom::Wysokie => "wysokie",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyProponowanyPoziom {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "niskie" => Ok(WpisyProponowanyPoziom::Niskie),
                    "srednie" => Ok(WpisyProponowanyPoziom::Srednie),
                    "wysokie" => Ok(WpisyProponowanyPoziom::Wysokie),
                    other => {
                        Err(format!("{other} is no value of WpisyProponowanyPoziom"))
                    }
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyProponowanyPoziom {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `sprawdzenia`.
#[derive(Debug, Clone, PartialEq)]
pub struct SprawdzeniaRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub teczka_odnosnik: String,
    pub dotyczy_tabela: Option<String>,
    pub dotyczy_odnosnik: Option<String>,
    pub zrodlo: Option<String>,
    pub zrodlo_data: Option<String>,
    pub zrodlo_id: Option<String>,
    pub wynik: WpisyWynik,
    pub opis: Option<String>,
    pub proponowany_poziom: Option<WpisyProponowanyPoziom>,
    pub czynniki: Option<String>,
    pub powod_watpliwosci: bool,
    pub sprawdzono: String,
    pub przekazano: Option<String>,
    pub rejestr_odnosnik: Option<String>,
    pub zastapione_przez_id: Option<String>,
}

impl SprawdzeniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(SprawdzeniaRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            dotyczy_tabela: row.get("dotyczy_tabela")?,
            dotyczy_odnosnik: row.get("dotyczy_odnosnik")?,
            zrodlo: row.get("zrodlo")?,
            zrodlo_data: row.get("zrodlo_data")?,
            zrodlo_id: row.get("zrodlo_id")?,
            wynik: row.get("wynik")?,
            opis: row.get("opis")?,
            proponowany_poziom: row.get("proponowany_poziom")?,
            czynniki: row.get("czynniki")?,
            powod_watpliwosci: row.get("powod_watpliwosci")?,
            sprawdzono: row.get("sprawdzono")?,
            przekazano: row.get("przekazano")?,
            rejestr_odnosnik: row.get("rejestr_odnosnik")?,
            zastapione_przez_id: row.get("zastapione_przez_id")?,
        })
    }
}

/// Aktualne sprawdzenia (niezastąpione), od najnowszego.
pub fn sprawdzenia(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<SprawdzeniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kontroler_sprawdzenia_wpisy
WHERE zastapione_przez_id IS NULL
ORDER BY sprawdzono DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(SprawdzeniaRow::from_row).collect()
}

/// The parameters of `sprawdzenia_teczki`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SprawdzeniaTeczkiParams {
    pub teczka_odnosnik: String,
}

/// A row of `sprawdzenia_teczki`.
#[derive(Debug, Clone, PartialEq)]
pub struct SprawdzeniaTeczkiRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub teczka_odnosnik: String,
    pub dotyczy_tabela: Option<String>,
    pub dotyczy_odnosnik: Option<String>,
    pub zrodlo: Option<String>,
    pub zrodlo_data: Option<String>,
    pub zrodlo_id: Option<String>,
    pub wynik: WpisyWynik,
    pub opis: Option<String>,
    pub proponowany_poziom: Option<WpisyProponowanyPoziom>,
    pub czynniki: Option<String>,
    pub powod_watpliwosci: bool,
    pub sprawdzono: String,
    pub przekazano: Option<String>,
    pub rejestr_odnosnik: Option<String>,
    pub zastapione_przez_id: Option<String>,
}

impl SprawdzeniaTeczkiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(SprawdzeniaTeczkiRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            dotyczy_tabela: row.get("dotyczy_tabela")?,
            dotyczy_odnosnik: row.get("dotyczy_odnosnik")?,
            zrodlo: row.get("zrodlo")?,
            zrodlo_data: row.get("zrodlo_data")?,
            zrodlo_id: row.get("zrodlo_id")?,
            wynik: row.get("wynik")?,
            opis: row.get("opis")?,
            proponowany_poziom: row.get("proponowany_poziom")?,
            czynniki: row.get("czynniki")?,
            powod_watpliwosci: row.get("powod_watpliwosci")?,
            sprawdzono: row.get("sprawdzono")?,
            przekazano: row.get("przekazano")?,
            rejestr_odnosnik: row.get("rejestr_odnosnik")?,
            zastapione_przez_id: row.get("zastapione_przez_id")?,
        })
    }
}

/// Aktualne sprawdzenia jednej teczki klienta, od najstarszego.
pub fn sprawdzenia_teczki(
    room: &impl xmeta_room::MetaRoom,
    params: &SprawdzeniaTeczkiParams,
) -> Result<Vec<SprawdzeniaTeczkiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kontroler_sprawdzenia_wpisy
WHERE teczka_odnosnik = :teczka_odnosnik
  AND zastapione_przez_id IS NULL
ORDER BY sprawdzono, id"#,
        parameters: Vec::from([
            (
                "teczka_odnosnik",
                xmeta_room::ToSqlValue::to_sql_value(&params.teczka_odnosnik),
            ),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(SprawdzeniaTeczkiRow::from_row).collect()
}

/// A row of `sprawdzenia_do_przekazania`.
#[derive(Debug, Clone, PartialEq)]
pub struct SprawdzeniaDoPrzekazaniaRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub teczka_odnosnik: String,
    pub dotyczy_tabela: Option<String>,
    pub dotyczy_odnosnik: Option<String>,
    pub zrodlo: Option<String>,
    pub zrodlo_data: Option<String>,
    pub zrodlo_id: Option<String>,
    pub wynik: WpisyWynik,
    pub opis: Option<String>,
    pub proponowany_poziom: Option<WpisyProponowanyPoziom>,
    pub czynniki: Option<String>,
    pub powod_watpliwosci: bool,
    pub sprawdzono: String,
    pub przekazano: Option<String>,
    pub rejestr_odnosnik: Option<String>,
    pub zastapione_przez_id: Option<String>,
}

impl SprawdzeniaDoPrzekazaniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(SprawdzeniaDoPrzekazaniaRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            dotyczy_tabela: row.get("dotyczy_tabela")?,
            dotyczy_odnosnik: row.get("dotyczy_odnosnik")?,
            zrodlo: row.get("zrodlo")?,
            zrodlo_data: row.get("zrodlo_data")?,
            zrodlo_id: row.get("zrodlo_id")?,
            wynik: row.get("wynik")?,
            opis: row.get("opis")?,
            proponowany_poziom: row.get("proponowany_poziom")?,
            czynniki: row.get("czynniki")?,
            powod_watpliwosci: row.get("powod_watpliwosci")?,
            sprawdzono: row.get("sprawdzono")?,
            przekazano: row.get("przekazano")?,
            rejestr_odnosnik: row.get("rejestr_odnosnik")?,
            zastapione_przez_id: row.get("zastapione_przez_id")?,
        })
    }
}

/// Aktualne sprawdzenia robocze, które czekają na przekazanie Kierownikowi kontroli.
pub fn sprawdzenia_do_przekazania(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<SprawdzeniaDoPrzekazaniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kontroler_sprawdzenia_wpisy
WHERE przekazano IS NULL
  AND zastapione_przez_id IS NULL
ORDER BY sprawdzono, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(SprawdzeniaDoPrzekazaniaRow::from_row).collect()
}

/// The parameters of `zapisz_sprawdzenie`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZapiszSprawdzenieParams {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub teczka_odnosnik: String,
    pub dotyczy_tabela: Option<String>,
    pub dotyczy_odnosnik: Option<String>,
    pub zrodlo: Option<String>,
    pub zrodlo_data: Option<String>,
    pub zrodlo_id: Option<String>,
    pub wynik: WpisyWynik,
    pub opis: Option<String>,
    pub proponowany_poziom: Option<WpisyProponowanyPoziom>,
    pub czynniki: Option<String>,
    pub powod_watpliwosci: bool,
    pub sprawdzono: String,
}

/// Sprawdzenie z wynikiem; identyfikator (UUID) nadaje aplikacja, żeby poprzednie sprawdzenie mogło
/// go wskazać jako zastępujące.
pub fn zapisz_sprawdzenie(
    room: &impl xmeta_room::MetaRoom,
    params: &ZapiszSprawdzenieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kontroler_sprawdzenia_wpisy (id, rodzaj, teczka_odnosnik, dotyczy_tabela,
  dotyczy_odnosnik, zrodlo, zrodlo_data, zrodlo_id, wynik, opis, proponowany_poziom, czynniki,
  powod_watpliwosci, sprawdzono)
VALUES (:id, :rodzaj, :teczka_odnosnik, :dotyczy_tabela, :dotyczy_odnosnik, :zrodlo, :zrodlo_data,
  :zrodlo_id, :wynik, :opis, :proponowany_poziom, :czynniki, :powod_watpliwosci, :sprawdzono)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    (
                        "teczka_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.teczka_odnosnik),
                    ),
                    (
                        "dotyczy_tabela",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_tabela),
                    ),
                    (
                        "dotyczy_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_odnosnik),
                    ),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                    (
                        "zrodlo_data",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_data),
                    ),
                    (
                        "zrodlo_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_id),
                    ),
                    ("wynik", xmeta_room::ToSqlValue::to_sql_value(&params.wynik)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    (
                        "proponowany_poziom",
                        xmeta_room::ToSqlValue::to_sql_value(&params.proponowany_poziom),
                    ),
                    ("czynniki", xmeta_room::ToSqlValue::to_sql_value(&params.czynniki)),
                    (
                        "powod_watpliwosci",
                        xmeta_room::ToSqlValue::to_sql_value(&params.powod_watpliwosci),
                    ),
                    (
                        "sprawdzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzono),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_przekazanie_sprawdzenia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPrzekazanieSprawdzeniaParams {
    pub przekazano: String,
    pub id: String,
}

/// Sprawdzenie przekazane Kierownikowi kontroli; od tej chwili się nie zmienia.
pub fn odnotuj_przekazanie_sprawdzenia(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPrzekazanieSprawdzeniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kontroler_sprawdzenia_wpisy
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

/// The parameters of `odnotuj_wpis_sprawdzenia_w_rejestrze`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujWpisSprawdzeniaWRejestrzeParams {
    pub rejestr_odnosnik: String,
    pub id: String,
}

/// Kierownik kontroli podał wpis w rejestrze kontroli, który powstał z przekazanego sprawdzenia;
/// opis, czynniki i proponowany poziom leżą odtąd tylko w rejestrze (D13).
pub fn odnotuj_wpis_sprawdzenia_w_rejestrze(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWpisSprawdzeniaWRejestrzeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kontroler_sprawdzenia_wpisy
SET rejestr_odnosnik = :rejestr_odnosnik,
  opis = NULL,
  czynniki = NULL,
  proponowany_poziom = NULL
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

/// The parameters of `zastap_sprawdzenie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZastapSprawdzenieParams {
    pub zastapione_przez_id: String,
    pub id: String,
}

/// Poprzednie sprawdzenie staje się nieaktualne, bo zastępuje je ponowne, zapisane wcześniej.
pub fn zastap_sprawdzenie(
    room: &impl xmeta_room::MetaRoom,
    params: &ZastapSprawdzenieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kontroler_sprawdzenia_wpisy
SET zastapione_przez_id = :zastapione_przez_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zastapione_przez_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zastapione_przez_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
