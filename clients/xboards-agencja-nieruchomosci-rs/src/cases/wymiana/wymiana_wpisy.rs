//! The kind `wymiana-wpisy` of the room `cases/wymiana`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "wymiana-wpisy",
    requires: &[&super::wymiana_metryka::KIND],
    migrations: &[
        r#"-- Dziennik wpisów pokoju wymiany: każdy wpis obu stron, także wiadomości, z autorem, stroną
-- i czasem. Pola typowe wpisu leżą w tabelach szczegółów innych rodzajów, które go wskazują
-- (wymiana-dokumenty, wymiana-dane, wymiana-akt; decyzja T11). Podpis autora daje serwer pokoju.
--
-- Wpisu się nie zmienia ani nie usuwa. Poprawka jest nowym wpisem, który wskazuje poprzedni.
-- Teczka transakcji w biurze i teczka notariusza wskazują wpis jego identyfikatorem, np. wpływ
-- z kanału pokoj_wymiany, więc wpis ma UUID.
--
-- Do pokoju nie trafia nic z przeciwdziałania praniu pieniędzy ani dane jednej strony, których
-- akt nie wymaga. Nie ma na to kolumn, a treść wiadomości pilnuje aplikacja.
CREATE TABLE wymiana_wpisy_wpisy (
  id TEXT PRIMARY KEY,
  -- Strona pokoju, w której imieniu pisze autor; deklarację porównuje aplikacja (decyzja T10).
  strona TEXT NOT NULL CHECK (strona IN ('biuro', 'kancelaria')),
  -- Tożsamość członka pokoju, który podpisał wpis.
  autor TEXT NOT NULL CHECK (autor <> ''),
  rodzaj TEXT NOT NULL CHECK (
    rodzaj IN (
      'lista_dokumentow',
      'dokument',
      'pelnomocnictwo',
      'zgoda_stron',
      'projekt_aktu',
      'koszty',
      'uwagi_do_projektu',
      'dane_osoby',
      'dane_nieruchomosci',
      'rachunek',
      'kredyt',
      'propozycja_terminu',
      'stan_terminu',
      'pierwokup',
      'po_akcie',
      'wiadomosc'
    )
  ),
  chwila TEXT NOT NULL,
  -- Wpis, na który ten odpowiada, np. odpowiedź na pytanie o ważność zaświadczenia.
  odpowiada_na_id TEXT REFERENCES wymiana_wpisy_wpisy (id),
  -- Wpis, który ten poprawia; poprawiony wpis zostaje w pokoju.
  poprawia_id TEXT REFERENCES wymiana_wpisy_wpisy (id),
  tresc TEXT CHECK (tresc <> ''),
  -- Plik w repozytorium plików pokoju i skrót jego treści, który obejmuje podpis wpisu.
  sciezka TEXT CHECK (sciezka <> ''),
  skrot TEXT CHECK (skrot <> ''),

  CONSTRAINT plik_ze_skrotem CHECK ((sciezka IS NULL) = (skrot IS NULL)),
  CONSTRAINT dokument_z_plikiem CHECK (
    rodzaj NOT IN ('dokument', 'pelnomocnictwo', 'zgoda_stron', 'projekt_aktu') OR sciezka IS NOT NULL
  ),
  CONSTRAINT wiadomosc_z_trescia CHECK (rodzaj <> 'wiadomosc' OR tresc IS NOT NULL),
  -- Klucz obcy przepuściłby wpis, który wskazuje sam siebie; poprawkę pilnuje trigger niżej.
  CONSTRAINT odpowiedz_nie_wskazuje_siebie CHECK (odpowiada_na_id IS NOT id),
  -- Listę dokumentów, projekt aktu, koszty, pierwokup i informację po akcie zna i wybiera
  -- kancelaria (art. 18 § 1 PoN).
  CONSTRAINT wpis_tylko_od_kancelarii CHECK (
    rodzaj NOT IN ('lista_dokumentow', 'projekt_aktu', 'koszty', 'pierwokup', 'po_akcie')
    OR strona = 'kancelaria'
  ),
  -- Dokumenty, pełnomocnictwa, zgody, uwagi i dane stron biuro przekazuje w imieniu stron.
  CONSTRAINT wpis_tylko_od_biura CHECK (
    rodzaj NOT IN (
      'dokument',
      'pelnomocnictwo',
      'zgoda_stron',
      'uwagi_do_projektu',
      'dane_osoby',
      'dane_nieruchomosci',
      'kredyt'
    )
    OR strona = 'biuro'
  )
) STRICT;

CREATE TRIGGER wymiana_wpisy_wymagaja_metryki
BEFORE INSERT ON wymiana_wpisy_wpisy
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_metryka_wpis
)
BEGIN
  SELECT RAISE(ABORT, 'wpis powstaje dopiero w pokoju z metryką');
END;

-- Po zamknięciu nikt w pokoju nie pisze, a członkowie mogą go tylko czytać.
CREATE TRIGGER wymiana_wpisy_zamkniety_pokoj_bez_wpisow
BEFORE INSERT ON wymiana_wpisy_wpisy
WHEN (
  SELECT stan
  FROM wymiana_metryka_stan
) = 'zamkniety'
BEGIN
  SELECT RAISE(ABORT, 'w zamkniętym pokoju nie powstaje nowy wpis');
END;

-- Poprawka zastępuje dane tego samego rodzaju i tej samej strony: kancelaria nie poprawia
-- pełnomocnictwa biura, a nowe dane osoby nie poprawiają rachunku.
CREATE TRIGGER wymiana_wpisy_poprawka_tego_samego_rodzaju
BEFORE INSERT ON wymiana_wpisy_wpisy
WHEN NEW.poprawia_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM wymiana_wpisy_wpisy AS poprawiany
    WHERE poprawiany.id = NEW.poprawia_id
      AND poprawiany.rodzaj = NEW.rodzaj
      AND poprawiany.strona = NEW.strona
  )
BEGIN
  SELECT RAISE(ABORT, 'poprawka wskazuje wpis tego samego rodzaju i tej samej strony');
END;

CREATE TRIGGER wymiana_wpisy_wpisy_bez_zmian
BEFORE UPDATE ON wymiana_wpisy_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_wpisy_wpisy_bez_zastepowania
BEFORE INSERT ON wymiana_wpisy_wpisy
WHEN EXISTS (
  SELECT 1
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_wpisy_wpisy_bez_usuwania
BEFORE DELETE ON wymiana_wpisy_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;
"#,
    ],
};

/// The values of column `wpisy.strona`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyStrona {
    Biuro,
    Kancelaria,
}

impl WpisyStrona {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyStrona] = &[WpisyStrona::Biuro, WpisyStrona::Kancelaria];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyStrona::Biuro => "biuro",
            WpisyStrona::Kancelaria => "kancelaria",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyStrona {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "biuro" => Ok(WpisyStrona::Biuro),
                    "kancelaria" => Ok(WpisyStrona::Kancelaria),
                    other => Err(format!("{other} is no value of WpisyStrona")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyStrona {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    ListaDokumentow,
    Dokument,
    Pelnomocnictwo,
    ZgodaStron,
    ProjektAktu,
    Koszty,
    UwagiDoProjektu,
    DaneOsoby,
    DaneNieruchomosci,
    Rachunek,
    Kredyt,
    PropozycjaTerminu,
    StanTerminu,
    Pierwokup,
    PoAkcie,
    Wiadomosc,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::ListaDokumentow,
        WpisyRodzaj::Dokument,
        WpisyRodzaj::Pelnomocnictwo,
        WpisyRodzaj::ZgodaStron,
        WpisyRodzaj::ProjektAktu,
        WpisyRodzaj::Koszty,
        WpisyRodzaj::UwagiDoProjektu,
        WpisyRodzaj::DaneOsoby,
        WpisyRodzaj::DaneNieruchomosci,
        WpisyRodzaj::Rachunek,
        WpisyRodzaj::Kredyt,
        WpisyRodzaj::PropozycjaTerminu,
        WpisyRodzaj::StanTerminu,
        WpisyRodzaj::Pierwokup,
        WpisyRodzaj::PoAkcie,
        WpisyRodzaj::Wiadomosc,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::ListaDokumentow => "lista_dokumentow",
            WpisyRodzaj::Dokument => "dokument",
            WpisyRodzaj::Pelnomocnictwo => "pelnomocnictwo",
            WpisyRodzaj::ZgodaStron => "zgoda_stron",
            WpisyRodzaj::ProjektAktu => "projekt_aktu",
            WpisyRodzaj::Koszty => "koszty",
            WpisyRodzaj::UwagiDoProjektu => "uwagi_do_projektu",
            WpisyRodzaj::DaneOsoby => "dane_osoby",
            WpisyRodzaj::DaneNieruchomosci => "dane_nieruchomosci",
            WpisyRodzaj::Rachunek => "rachunek",
            WpisyRodzaj::Kredyt => "kredyt",
            WpisyRodzaj::PropozycjaTerminu => "propozycja_terminu",
            WpisyRodzaj::StanTerminu => "stan_terminu",
            WpisyRodzaj::Pierwokup => "pierwokup",
            WpisyRodzaj::PoAkcie => "po_akcie",
            WpisyRodzaj::Wiadomosc => "wiadomosc",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "lista_dokumentow" => Ok(WpisyRodzaj::ListaDokumentow),
                    "dokument" => Ok(WpisyRodzaj::Dokument),
                    "pelnomocnictwo" => Ok(WpisyRodzaj::Pelnomocnictwo),
                    "zgoda_stron" => Ok(WpisyRodzaj::ZgodaStron),
                    "projekt_aktu" => Ok(WpisyRodzaj::ProjektAktu),
                    "koszty" => Ok(WpisyRodzaj::Koszty),
                    "uwagi_do_projektu" => Ok(WpisyRodzaj::UwagiDoProjektu),
                    "dane_osoby" => Ok(WpisyRodzaj::DaneOsoby),
                    "dane_nieruchomosci" => Ok(WpisyRodzaj::DaneNieruchomosci),
                    "rachunek" => Ok(WpisyRodzaj::Rachunek),
                    "kredyt" => Ok(WpisyRodzaj::Kredyt),
                    "propozycja_terminu" => Ok(WpisyRodzaj::PropozycjaTerminu),
                    "stan_terminu" => Ok(WpisyRodzaj::StanTerminu),
                    "pierwokup" => Ok(WpisyRodzaj::Pierwokup),
                    "po_akcie" => Ok(WpisyRodzaj::PoAkcie),
                    "wiadomosc" => Ok(WpisyRodzaj::Wiadomosc),
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

/// A row of `wpisy`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpisyRow {
    pub id: String,
    pub strona: WpisyStrona,
    pub autor: String,
    pub rodzaj: WpisyRodzaj,
    pub chwila: String,
    pub odpowiada_na_id: Option<String>,
    pub poprawia_id: Option<String>,
    pub tresc: Option<String>,
    pub sciezka: Option<String>,
    pub skrot: Option<String>,
}

impl WpisyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WpisyRow {
            id: row.get("id")?,
            strona: row.get("strona")?,
            autor: row.get("autor")?,
            rodzaj: row.get("rodzaj")?,
            chwila: row.get("chwila")?,
            odpowiada_na_id: row.get("odpowiada_na_id")?,
            poprawia_id: row.get("poprawia_id")?,
            tresc: row.get("tresc")?,
            sciezka: row.get("sciezka")?,
            skrot: row.get("skrot")?,
        })
    }
}

/// Wszystkie wpisy pokoju w kolejności powstania; poprawione zostają obok poprawek.
pub fn wpisy(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WpisyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM wymiana_wpisy_wpisy
ORDER BY rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(WpisyRow::from_row).collect()
}

/// The parameters of `wpis`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpisParams {
    pub id: String,
}

/// A row of `wpis`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpisRow {
    pub id: String,
    pub strona: WpisyStrona,
    pub autor: String,
    pub rodzaj: WpisyRodzaj,
    pub chwila: String,
    pub odpowiada_na_id: Option<String>,
    pub poprawia_id: Option<String>,
    pub tresc: Option<String>,
    pub sciezka: Option<String>,
    pub skrot: Option<String>,
}

impl WpisRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WpisRow {
            id: row.get("id")?,
            strona: row.get("strona")?,
            autor: row.get("autor")?,
            rodzaj: row.get("rodzaj")?,
            chwila: row.get("chwila")?,
            odpowiada_na_id: row.get("odpowiada_na_id")?,
            poprawia_id: row.get("poprawia_id")?,
            tresc: row.get("tresc")?,
            sciezka: row.get("sciezka")?,
            skrot: row.get("skrot")?,
        })
    }
}

/// Jeden wpis, np. ten, na który odpowiada wiadomość albo który wskazuje teczka.
pub fn wpis(
    room: &impl xmeta_room::MetaRoom,
    params: &WpisParams,
) -> Result<Option<WpisRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM wymiana_wpisy_wpisy
WHERE id = :id"#,
        parameters: Vec::from([("id", xmeta_room::ToSqlValue::to_sql_value(&params.id))]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.first().map(WpisRow::from_row).transpose()
}

/// The parameters of `wpisy_od_chwili`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpisyOdChwiliParams {
    pub od: String,
}

/// A row of `wpisy_od_chwili`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpisyOdChwiliRow {
    pub id: String,
    pub strona: WpisyStrona,
    pub autor: String,
    pub rodzaj: WpisyRodzaj,
    pub chwila: String,
    pub odpowiada_na_id: Option<String>,
    pub poprawia_id: Option<String>,
    pub tresc: Option<String>,
    pub sciezka: Option<String>,
    pub skrot: Option<String>,
}

impl WpisyOdChwiliRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WpisyOdChwiliRow {
            id: row.get("id")?,
            strona: row.get("strona")?,
            autor: row.get("autor")?,
            rodzaj: row.get("rodzaj")?,
            chwila: row.get("chwila")?,
            odpowiada_na_id: row.get("odpowiada_na_id")?,
            poprawia_id: row.get("poprawia_id")?,
            tresc: row.get("tresc")?,
            sciezka: row.get("sciezka")?,
            skrot: row.get("skrot")?,
        })
    }
}

/// Wpisy, które powstały od danej chwili, np. od ostatniego przeniesienia do teczki.
pub fn wpisy_od_chwili(
    room: &impl xmeta_room::MetaRoom,
    params: &WpisyOdChwiliParams,
) -> Result<Vec<WpisyOdChwiliRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM wymiana_wpisy_wpisy
WHERE chwila >= :od
ORDER BY rowid"#,
        parameters: Vec::from([("od", xmeta_room::ToSqlValue::to_sql_value(&params.od))]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(WpisyOdChwiliRow::from_row).collect()
}

/// The parameters of `wpisz_wiadomosc`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszWiadomoscParams {
    pub id: String,
    pub strona: WpisyStrona,
    pub autor: String,
    pub chwila: String,
    pub odpowiada_na_id: Option<String>,
    pub poprawia_id: Option<String>,
    pub tresc: Option<String>,
    pub sciezka: Option<String>,
    pub skrot: Option<String>,
}

/// Pytanie albo odpowiedź, która nie mieści się w polach innych wpisów; wskazuje wpis, którego
/// dotyczy. Może mieć plik.
pub fn wpisz_wiadomosc(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszWiadomoscParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id, poprawia_id,
  tresc, sciezka, skrot)
VALUES (:id, :strona, :autor, 'wiadomosc', :chwila, :odpowiada_na_id, :poprawia_id, :tresc,
  :sciezka, :skrot)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("strona", xmeta_room::ToSqlValue::to_sql_value(&params.strona)),
                    ("autor", xmeta_room::ToSqlValue::to_sql_value(&params.autor)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    (
                        "odpowiada_na_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odpowiada_na_id),
                    ),
                    (
                        "poprawia_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprawia_id),
                    ),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                    ("sciezka", xmeta_room::ToSqlValue::to_sql_value(&params.sciezka)),
                    ("skrot", xmeta_room::ToSqlValue::to_sql_value(&params.skrot)),
                ]),
            },
        ],
    )
}
