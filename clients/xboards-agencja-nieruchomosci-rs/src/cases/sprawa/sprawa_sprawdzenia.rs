//! The kind `sprawa-sprawdzenia` of the room `cases/sprawa`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "sprawa-sprawdzenia",
    requires: &[
        &super::sprawa_decyzje::KIND,
        &super::sprawa_dokumenty::KIND,
        &super::sprawa_nieruchomosci::KIND,
        &super::sprawa_rachunki::KIND,
        &super::sprawa_uczestnicy::KIND,
        &super::sprawa_umocowania::KIND,
    ],
    migrations: &[
        r#"-- Wyniki sprawdzeń Kontrolera, które nie należą do przeciwdziałania praniu pieniędzy: tożsamość,
-- tytuł do nieruchomości, umocowanie i rachunek. Typ sprawy mówi, co Kontroler w nim sprawdza.
-- Wpisuje je tylko Kontroler klienta (R7), a wynik ma tylko trzy słowa; ustalenia ze środków
-- bezpieczeństwa finansowego leżą w rejestrze kontroli.
--
-- Wynik dotyczy wersji wiersza, którą Kontroler widział (D6). Nowe sprawdzenie tego samego wiersza
-- i zmiana danych identyfikujących uczestnika albo umocowania odbierają poprzedniemu wynikowi
-- aktualność. Wpisu się nie zmienia; ponowne sprawdzenie to nowy wpis.
--
-- Reguły, które zależą od późniejszych rodzajów, leżą w ich migracjach: sprawdzenie stron przed
-- podpisem umowy w sprawa-umowy (sprawa_umowy_do_podpisu_po_sprawdzeniu_stron), sprawdzenie
-- rachunku przed wypłatą w sprawa-wyplaty (sprawa_wyplaty_sprawdzenie_po_przygotowaniu).
CREATE TABLE sprawa_sprawdzenia_wpisy (
  id TEXT PRIMARY KEY,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('tozsamosc', 'tytul', 'umocowanie', 'rachunek')),
  -- Sprawdzany wiersz: uczestnik (tożsamość), uczestnik z nieruchomością albo dokumentem (tytuł),
  -- umocowanie albo rachunek.
  uczestnik_id TEXT REFERENCES sprawa_uczestnicy_wpisy (id),
  nieruchomosc_id TEXT REFERENCES sprawa_nieruchomosci_wpisy (id),
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  umocowanie_id TEXT REFERENCES sprawa_umocowania_wpisy (id),
  rachunek_id TEXT REFERENCES sprawa_rachunki_wpisy (id),
  -- Wersja uczestnika albo umocowania, którą Kontroler sprawdził; rachunek się nie zmienia.
  wersja_wiersza INTEGER,
  -- Źródło spoza teczki, np. dział II księgi wieczystej, odpis z KRS, telefon do posiadacza.
  zrodlo TEXT NOT NULL,
  zrodlo_data TEXT,
  wynik TEXT NOT NULL CHECK (wynik IN ('zgodne', 'uwagi', 'nie_mozna_ustalic')),
  -- Czego brakuje; tylko to, co wolno przeczytać osobom obsługującym klienta.
  opis TEXT,
  sprawdzil TEXT NOT NULL,
  sprawdzono TEXT NOT NULL,
  aktualne INTEGER NOT NULL CHECK (aktualne IN (0, 1)),

  CONSTRAINT rodzaj_pasuje_do_wiersza CHECK (
    CASE rodzaj
      WHEN 'tozsamosc' THEN uczestnik_id IS NOT NULL AND nieruchomosc_id IS NULL
        AND dokument_id IS NULL AND umocowanie_id IS NULL AND rachunek_id IS NULL
      WHEN 'tytul' THEN uczestnik_id IS NOT NULL
        AND (nieruchomosc_id IS NOT NULL OR dokument_id IS NOT NULL)
        AND umocowanie_id IS NULL AND rachunek_id IS NULL
      WHEN 'umocowanie' THEN umocowanie_id IS NOT NULL AND uczestnik_id IS NULL
        AND nieruchomosc_id IS NULL AND dokument_id IS NULL AND rachunek_id IS NULL
      WHEN 'rachunek' THEN rachunek_id IS NOT NULL AND uczestnik_id IS NULL
        AND nieruchomosc_id IS NULL AND dokument_id IS NULL AND umocowanie_id IS NULL
    END
  ),
  CONSTRAINT wersja_wiersza_przy_wierszu_z_wersja CHECK (
    (wersja_wiersza IS NOT NULL) = (rodzaj IN ('tozsamosc', 'tytul', 'umocowanie'))
  ),
  CONSTRAINT wynik_z_uwagami_ma_opis CHECK (wynik = 'zgodne' OR opis IS NOT NULL)
) STRICT;

-- Aktualność odbierają tylko triggery, więc sprawdzenie powstaje aktualne.
CREATE TRIGGER sprawa_sprawdzenia_powstaje_aktualne
BEFORE INSERT ON sprawa_sprawdzenia_wpisy
WHEN NEW.aktualne <> 1
BEGIN
  SELECT RAISE(ABORT, 'sprawdzenie powstaje jako aktualne');
END;

CREATE TRIGGER sprawa_sprawdzenia_dotyczy_biezacej_wersji
BEFORE INSERT ON sprawa_sprawdzenia_wpisy
WHEN (
    NEW.uczestnik_id IS NOT NULL
    AND NEW.wersja_wiersza IS NOT (
      SELECT wersja
      FROM sprawa_uczestnicy_wpisy
      WHERE id = NEW.uczestnik_id
    )
  )
  OR (
    NEW.umocowanie_id IS NOT NULL
    AND NEW.wersja_wiersza IS NOT (
      SELECT wersja
      FROM sprawa_umocowania_wpisy
      WHERE id = NEW.umocowanie_id
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'sprawdzenie dotyczy bieżącej wersji wiersza');
END;

CREATE TRIGGER sprawa_sprawdzenia_nowe_zeruje_poprzednie
AFTER INSERT ON sprawa_sprawdzenia_wpisy
BEGIN
  UPDATE sprawa_sprawdzenia_wpisy
  SET aktualne = 0
  WHERE id <> NEW.id
    AND aktualne = 1
    AND rodzaj = NEW.rodzaj
    AND uczestnik_id IS NEW.uczestnik_id
    AND nieruchomosc_id IS NEW.nieruchomosc_id
    AND dokument_id IS NEW.dokument_id
    AND umocowanie_id IS NEW.umocowanie_id
    AND rachunek_id IS NEW.rachunek_id;
END;

-- Wynik ma podpis Kontrolera; jedyną zmianą jest utrata aktualności.
CREATE TRIGGER sprawa_sprawdzenia_bez_zmian
BEFORE UPDATE ON sprawa_sprawdzenia_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.uczestnik_id IS NOT OLD.uczestnik_id
  OR NEW.nieruchomosc_id IS NOT OLD.nieruchomosc_id
  OR NEW.dokument_id IS NOT OLD.dokument_id
  OR NEW.umocowanie_id IS NOT OLD.umocowanie_id
  OR NEW.rachunek_id IS NOT OLD.rachunek_id
  OR NEW.wersja_wiersza IS NOT OLD.wersja_wiersza
  OR NEW.zrodlo IS NOT OLD.zrodlo
  OR NEW.zrodlo_data IS NOT OLD.zrodlo_data
  OR NEW.wynik IS NOT OLD.wynik
  OR NEW.opis IS NOT OLD.opis
  OR NEW.sprawdzil IS NOT OLD.sprawdzil
  OR NEW.sprawdzono IS NOT OLD.sprawdzono
  OR (OLD.aktualne = 0 AND NEW.aktualne = 1)
BEGIN
  SELECT RAISE(ABORT, 'wyniku sprawdzenia się nie zmienia; ponowne sprawdzenie to nowy wpis');
END;

CREATE TRIGGER sprawa_sprawdzenia_nowa_wersja_uczestnika
AFTER UPDATE OF wersja ON sprawa_uczestnicy_wpisy
WHEN NEW.wersja IS NOT OLD.wersja
BEGIN
  UPDATE sprawa_sprawdzenia_wpisy
  SET aktualne = 0
  WHERE uczestnik_id = NEW.id
    AND aktualne = 1;
END;

CREATE TRIGGER sprawa_sprawdzenia_nowa_wersja_umocowania
AFTER UPDATE OF wersja ON sprawa_umocowania_wpisy
WHEN NEW.wersja IS NOT OLD.wersja
BEGIN
  UPDATE sprawa_sprawdzenia_wpisy
  SET aktualne = 0
  WHERE umocowanie_id = NEW.id
    AND aktualne = 1;
END;

-- Zgoda na rachunek stoi na aktualnym sprawdzeniu „zgodne” i potwierdzeniu posiadacza innym
-- kanałem, a zastąpiony rachunek nie dostaje zgody (R4). Odmówić wolno zawsze.
CREATE TRIGGER sprawa_sprawdzenia_dopuszczenie_rachunku_wymaga_sprawdzenia
BEFORE UPDATE OF stan ON sprawa_decyzje_wpisy
WHEN NEW.stan = 'rozstrzygnieta'
  AND NEW.rodzaj = 'dopuszczenie_rachunku'
  AND NEW.rozstrzygniecie IN ('zgoda', 'zgoda_z_warunkiem')
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_rachunki_wpisy AS rachunek
    JOIN sprawa_sprawdzenia_wpisy AS sprawdzenie ON sprawdzenie.rachunek_id = rachunek.id
    WHERE NEW.dotyczy_tabela = 'sprawa_rachunki_wpisy'
      AND rachunek.id = NEW.dotyczy_id
      AND rachunek.potwierdzono IS NOT NULL
      AND rachunek.zastapiony_przez_id IS NULL
      AND sprawdzenie.rodzaj = 'rachunek'
      AND sprawdzenie.aktualne = 1
      AND sprawdzenie.wynik = 'zgodne'
  )
BEGIN
  SELECT RAISE(ABORT, 'zgoda na rachunek wymaga aktualnego sprawdzenia „zgodne”, potwierdzenia innym kanałem i niezastąpionego rachunku');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_sprawdzenia_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_sprawdzenia_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_sprawdzenia_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_sprawdzenia_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_sprawdzenia_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
    ],
};

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    Tozsamosc,
    Tytul,
    Umocowanie,
    Rachunek,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::Tozsamosc,
        WpisyRodzaj::Tytul,
        WpisyRodzaj::Umocowanie,
        WpisyRodzaj::Rachunek,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::Tozsamosc => "tozsamosc",
            WpisyRodzaj::Tytul => "tytul",
            WpisyRodzaj::Umocowanie => "umocowanie",
            WpisyRodzaj::Rachunek => "rachunek",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "tozsamosc" => Ok(WpisyRodzaj::Tozsamosc),
                    "tytul" => Ok(WpisyRodzaj::Tytul),
                    "umocowanie" => Ok(WpisyRodzaj::Umocowanie),
                    "rachunek" => Ok(WpisyRodzaj::Rachunek),
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

/// A row of `sprawdzenia`.
#[derive(Debug, Clone, PartialEq)]
pub struct SprawdzeniaRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub uczestnik_id: Option<String>,
    pub nieruchomosc_id: Option<String>,
    pub dokument_id: Option<String>,
    pub umocowanie_id: Option<String>,
    pub rachunek_id: Option<String>,
    pub wersja_wiersza: Option<i64>,
    pub zrodlo: String,
    pub zrodlo_data: Option<String>,
    pub wynik: WpisyWynik,
    pub opis: Option<String>,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub aktualne: bool,
}

impl SprawdzeniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(SprawdzeniaRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            uczestnik_id: row.get("uczestnik_id")?,
            nieruchomosc_id: row.get("nieruchomosc_id")?,
            dokument_id: row.get("dokument_id")?,
            umocowanie_id: row.get("umocowanie_id")?,
            rachunek_id: row.get("rachunek_id")?,
            wersja_wiersza: row.get("wersja_wiersza")?,
            zrodlo: row.get("zrodlo")?,
            zrodlo_data: row.get("zrodlo_data")?,
            wynik: row.get("wynik")?,
            opis: row.get("opis")?,
            sprawdzil: row.get("sprawdzil")?,
            sprawdzono: row.get("sprawdzono")?,
            aktualne: row.get("aktualne")?,
        })
    }
}

/// Wyniki sprawdzeń Kontrolera, najnowsze pierwsze.
pub fn sprawdzenia(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<SprawdzeniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_sprawdzenia_wpisy
ORDER BY sprawdzono DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(SprawdzeniaRow::from_row).collect()
}

/// A row of `sprawdzenia_aktualne`.
#[derive(Debug, Clone, PartialEq)]
pub struct SprawdzeniaAktualneRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub uczestnik_id: Option<String>,
    pub nieruchomosc_id: Option<String>,
    pub dokument_id: Option<String>,
    pub umocowanie_id: Option<String>,
    pub rachunek_id: Option<String>,
    pub wersja_wiersza: Option<i64>,
    pub zrodlo: String,
    pub zrodlo_data: Option<String>,
    pub wynik: WpisyWynik,
    pub opis: Option<String>,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub aktualne: bool,
}

impl SprawdzeniaAktualneRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(SprawdzeniaAktualneRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            uczestnik_id: row.get("uczestnik_id")?,
            nieruchomosc_id: row.get("nieruchomosc_id")?,
            dokument_id: row.get("dokument_id")?,
            umocowanie_id: row.get("umocowanie_id")?,
            rachunek_id: row.get("rachunek_id")?,
            wersja_wiersza: row.get("wersja_wiersza")?,
            zrodlo: row.get("zrodlo")?,
            zrodlo_data: row.get("zrodlo_data")?,
            wynik: row.get("wynik")?,
            opis: row.get("opis")?,
            sprawdzil: row.get("sprawdzil")?,
            sprawdzono: row.get("sprawdzono")?,
            aktualne: row.get("aktualne")?,
        })
    }
}

/// Wyniki, które obejmują bieżącą wersję sprawdzanych wierszy.
pub fn sprawdzenia_aktualne(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<SprawdzeniaAktualneRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_sprawdzenia_wpisy
WHERE aktualne = 1
ORDER BY sprawdzono DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(SprawdzeniaAktualneRow::from_row).collect()
}

/// The parameters of `wpisz_sprawdzenie`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszSprawdzenieParams {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub uczestnik_id: Option<String>,
    pub nieruchomosc_id: Option<String>,
    pub dokument_id: Option<String>,
    pub umocowanie_id: Option<String>,
    pub rachunek_id: Option<String>,
    pub wersja_wiersza: Option<i64>,
    pub zrodlo: String,
    pub zrodlo_data: Option<String>,
    pub wynik: WpisyWynik,
    pub opis: Option<String>,
    pub sprawdzil: String,
    pub sprawdzono: String,
}

/// Wynik sprawdzenia Kontrolera dla wiersza w bieżącej wersji; poprzedni wynik tego wiersza
/// traci aktualność.
pub fn wpisz_sprawdzenie(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszSprawdzenieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_sprawdzenia_wpisy (id, rodzaj, uczestnik_id, nieruchomosc_id, dokument_id,
  umocowanie_id, rachunek_id, wersja_wiersza, zrodlo, zrodlo_data, wynik, opis, sprawdzil,
  sprawdzono, aktualne)
VALUES (:id, :rodzaj, :uczestnik_id, :nieruchomosc_id, :dokument_id, :umocowanie_id, :rachunek_id,
  :wersja_wiersza, :zrodlo, :zrodlo_data, :wynik, :opis, :sprawdzil, :sprawdzono, 1)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    (
                        "uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.uczestnik_id),
                    ),
                    (
                        "nieruchomosc_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.nieruchomosc_id),
                    ),
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                    (
                        "umocowanie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.umocowanie_id),
                    ),
                    (
                        "rachunek_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rachunek_id),
                    ),
                    (
                        "wersja_wiersza",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wersja_wiersza),
                    ),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                    (
                        "zrodlo_data",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_data),
                    ),
                    ("wynik", xmeta_room::ToSqlValue::to_sql_value(&params.wynik)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    (
                        "sprawdzil",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzil),
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
