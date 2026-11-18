//! The kind `kierownik-biura-meldunki` of the room `positions/biuro/kierownik_biura`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "kierownik-biura-meldunki";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Meldunki Recepcji i kierowników Pośrednictwa, Transakcji i Najmu. Z nich powstaje przebieg
-- obłożenia w sprawozdaniu, więc kierownik trzyma wszystkie, a nie tylko ostatni. Kontrola nie
-- melduje Kierownikowi biura, bo jej nie podlega.
--
-- Liczby są informacją od nadawcy, a twierdzenia (np. że inny dział opóźnia sprawy) zapisuje się
-- jako jego słowa, które kierownik sprawdza w liczbach. Meldunki tylko się dopisuje.
CREATE TABLE kierownik_biura_meldunki_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_biura_meldunki_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  nadawca TEXT NOT NULL CHECK (nadawca IN ('recepcja', 'posrednictwo', 'transakcje', 'najem')),
  -- Tożsamość agenta, który meldunek wysłał.
  tozsamosc TEXT NOT NULL CHECK (tozsamosc <> ''),
  otrzymano TEXT NOT NULL,
  -- Pola meldunku kierownika działu.
  agenci_pracuja INTEGER CHECK (agenci_pracuja >= 0),
  sprawy_zablokowane INTEGER CHECK (sprawy_zablokowane >= 0),
  decyzje_czekaja INTEGER CHECK (decyzje_czekaja >= 0),
  najblizszy_termin TEXT,
  -- Pola meldunku Recepcji.
  wiadomosci_wplynely INTEGER CHECK (wiadomosci_wplynely >= 0),
  karty_wyslane INTEGER CHECK (karty_wyslane >= 0),
  zaklocenia INTEGER CHECK (zaklocenia >= 0),
  -- To, co meldunek mówi poza liczbami, jako słowa nadawcy.
  twierdzenia TEXT,

  CONSTRAINT pola_meldunku_dzialu CHECK (
    (nadawca <> 'recepcja') = (agenci_pracuja IS NOT NULL)
    AND (nadawca <> 'recepcja') = (sprawy_zablokowane IS NOT NULL)
    AND (nadawca <> 'recepcja') = (decyzje_czekaja IS NOT NULL)
    AND (nadawca <> 'recepcja' OR najblizszy_termin IS NULL)
  ),
  CONSTRAINT pola_meldunku_recepcji CHECK (
    (nadawca = 'recepcja') = (wiadomosci_wplynely IS NOT NULL)
    AND (nadawca = 'recepcja') = (karty_wyslane IS NOT NULL)
    AND (nadawca = 'recepcja') = (zaklocenia IS NOT NULL)
  )
) STRICT;

CREATE TRIGGER kierownik_biura_meldunki_bez_zmian
BEFORE UPDATE ON kierownik_biura_meldunki_wpisy
BEGIN
  SELECT RAISE(ABORT, 'meldunki tylko się dopisuje; zmiany nie są dozwolone');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_biura_meldunki_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_biura_meldunki_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_biura_meldunki_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: z dawnych meldunków powstaje przebieg obłożenia.
CREATE TRIGGER kierownik_biura_meldunki_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_biura_meldunki_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); meldunki tylko się dopisuje');
END;
"#,
];

/// The values of column `wpisy.nadawca`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyNadawca {
    Recepcja,
    Posrednictwo,
    Transakcje,
    Najem,
}

impl WpisyNadawca {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyNadawca] = &[
        WpisyNadawca::Recepcja,
        WpisyNadawca::Posrednictwo,
        WpisyNadawca::Transakcje,
        WpisyNadawca::Najem,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyNadawca::Recepcja => "recepcja",
            WpisyNadawca::Posrednictwo => "posrednictwo",
            WpisyNadawca::Transakcje => "transakcje",
            WpisyNadawca::Najem => "najem",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyNadawca {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "recepcja" => Ok(WpisyNadawca::Recepcja),
                    "posrednictwo" => Ok(WpisyNadawca::Posrednictwo),
                    "transakcje" => Ok(WpisyNadawca::Transakcje),
                    "najem" => Ok(WpisyNadawca::Najem),
                    other => Err(format!("{other} is no value of WpisyNadawca")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyNadawca {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of field `nadawca` of `dopisz_meldunek_dzialu`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nadawca {
    Posrednictwo,
    Transakcje,
    Najem,
}

impl Nadawca {
    /// Every value, in the order of the catalog.
    pub const ALL: &[Nadawca] = &[
        Nadawca::Posrednictwo,
        Nadawca::Transakcje,
        Nadawca::Najem,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            Nadawca::Posrednictwo => "posrednictwo",
            Nadawca::Transakcje => "transakcje",
            Nadawca::Najem => "najem",
        }
    }
}

impl xmeta_room::FromSqlValue for Nadawca {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "posrednictwo" => Ok(Nadawca::Posrednictwo),
                    "transakcje" => Ok(Nadawca::Transakcje),
                    "najem" => Ok(Nadawca::Najem),
                    other => Err(format!("{other} is no value of Nadawca")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for Nadawca {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `ostatnie_meldunki`.
#[derive(Debug, Clone, PartialEq)]
pub struct OstatnieMeldunkiRow {
    pub id: i64,
    pub nadawca: WpisyNadawca,
    pub tozsamosc: String,
    pub otrzymano: String,
    pub agenci_pracuja: Option<i64>,
    pub sprawy_zablokowane: Option<i64>,
    pub decyzje_czekaja: Option<i64>,
    pub najblizszy_termin: Option<String>,
    pub wiadomosci_wplynely: Option<i64>,
    pub karty_wyslane: Option<i64>,
    pub zaklocenia: Option<i64>,
    pub twierdzenia: Option<String>,
}

impl OstatnieMeldunkiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(OstatnieMeldunkiRow {
            id: row.get("id")?,
            nadawca: row.get("nadawca")?,
            tozsamosc: row.get("tozsamosc")?,
            otrzymano: row.get("otrzymano")?,
            agenci_pracuja: row.get("agenci_pracuja")?,
            sprawy_zablokowane: row.get("sprawy_zablokowane")?,
            decyzje_czekaja: row.get("decyzje_czekaja")?,
            najblizszy_termin: row.get("najblizszy_termin")?,
            wiadomosci_wplynely: row.get("wiadomosci_wplynely")?,
            karty_wyslane: row.get("karty_wyslane")?,
            zaklocenia: row.get("zaklocenia")?,
            twierdzenia: row.get("twierdzenia")?,
        })
    }
}

/// Ostatni meldunek każdego nadawcy; z niego kierownik zna bieżące obłożenie.
pub fn ostatnie_meldunki(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<OstatnieMeldunkiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT meldunek.*
FROM kierownik_biura_meldunki_wpisy AS meldunek
WHERE meldunek.id = (
  SELECT ostatni.id
  FROM kierownik_biura_meldunki_wpisy AS ostatni
  WHERE ostatni.nadawca = meldunek.nadawca
  ORDER BY ostatni.otrzymano DESC, ostatni.id DESC
  LIMIT 1
)
ORDER BY meldunek.nadawca"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(OstatnieMeldunkiRow::from_row).collect()
}

/// The parameters of `meldunki_od`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MeldunkiOdParams {
    pub od: String,
}

/// A row of `meldunki_od`.
#[derive(Debug, Clone, PartialEq)]
pub struct MeldunkiOdRow {
    pub id: i64,
    pub nadawca: WpisyNadawca,
    pub tozsamosc: String,
    pub otrzymano: String,
    pub agenci_pracuja: Option<i64>,
    pub sprawy_zablokowane: Option<i64>,
    pub decyzje_czekaja: Option<i64>,
    pub najblizszy_termin: Option<String>,
    pub wiadomosci_wplynely: Option<i64>,
    pub karty_wyslane: Option<i64>,
    pub zaklocenia: Option<i64>,
    pub twierdzenia: Option<String>,
}

impl MeldunkiOdRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(MeldunkiOdRow {
            id: row.get("id")?,
            nadawca: row.get("nadawca")?,
            tozsamosc: row.get("tozsamosc")?,
            otrzymano: row.get("otrzymano")?,
            agenci_pracuja: row.get("agenci_pracuja")?,
            sprawy_zablokowane: row.get("sprawy_zablokowane")?,
            decyzje_czekaja: row.get("decyzje_czekaja")?,
            najblizszy_termin: row.get("najblizszy_termin")?,
            wiadomosci_wplynely: row.get("wiadomosci_wplynely")?,
            karty_wyslane: row.get("karty_wyslane")?,
            zaklocenia: row.get("zaklocenia")?,
            twierdzenia: row.get("twierdzenia")?,
        })
    }
}

/// Meldunki od jednego dnia, od najstarszego; z nich powstaje przebieg obłożenia w sprawozdaniu.
pub fn meldunki_od(
    room: &impl xmeta_room::MetaRoom,
    params: &MeldunkiOdParams,
) -> Result<Vec<MeldunkiOdRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_biura_meldunki_wpisy
WHERE otrzymano >= :od
ORDER BY otrzymano, id"#,
        parameters: Vec::from([("od", xmeta_room::ToSqlValue::to_sql_value(&params.od))]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(MeldunkiOdRow::from_row).collect()
}

/// The parameters of `dopisz_meldunek_dzialu`.
#[derive(Debug, Clone, PartialEq)]
pub struct DopiszMeldunekDzialuParams {
    pub nadawca: Nadawca,
    pub tozsamosc: String,
    pub otrzymano: String,
    pub agenci_pracuja: Option<i64>,
    pub sprawy_zablokowane: Option<i64>,
    pub decyzje_czekaja: Option<i64>,
    pub najblizszy_termin: Option<String>,
    pub twierdzenia: Option<String>,
}

/// Meldunek kierownika Pośrednictwa, Transakcji albo Najmu.
pub fn dopisz_meldunek_dzialu(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszMeldunekDzialuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_biura_meldunki_wpisy (nadawca, tozsamosc, otrzymano, agenci_pracuja,
  sprawy_zablokowane, decyzje_czekaja, najblizszy_termin, twierdzenia)
VALUES (:nadawca, :tozsamosc, :otrzymano, :agenci_pracuja, :sprawy_zablokowane, :decyzje_czekaja,
  :najblizszy_termin, :twierdzenia)"#,
                parameters: Vec::from([
                    ("nadawca", xmeta_room::ToSqlValue::to_sql_value(&params.nadawca)),
                    (
                        "tozsamosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.tozsamosc),
                    ),
                    (
                        "otrzymano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.otrzymano),
                    ),
                    (
                        "agenci_pracuja",
                        xmeta_room::ToSqlValue::to_sql_value(&params.agenci_pracuja),
                    ),
                    (
                        "sprawy_zablokowane",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawy_zablokowane),
                    ),
                    (
                        "decyzje_czekaja",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzje_czekaja),
                    ),
                    (
                        "najblizszy_termin",
                        xmeta_room::ToSqlValue::to_sql_value(&params.najblizszy_termin),
                    ),
                    (
                        "twierdzenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.twierdzenia),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `dopisz_meldunek_recepcji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DopiszMeldunekRecepcjiParams {
    pub tozsamosc: String,
    pub otrzymano: String,
    pub wiadomosci_wplynely: Option<i64>,
    pub karty_wyslane: Option<i64>,
    pub zaklocenia: Option<i64>,
    pub twierdzenia: Option<String>,
}

/// Dzienny meldunek Recepcji.
pub fn dopisz_meldunek_recepcji(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszMeldunekRecepcjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_biura_meldunki_wpisy (nadawca, tozsamosc, otrzymano, wiadomosci_wplynely,
  karty_wyslane, zaklocenia, twierdzenia)
VALUES ('recepcja', :tozsamosc, :otrzymano, :wiadomosci_wplynely, :karty_wyslane, :zaklocenia,
  :twierdzenia)"#,
                parameters: Vec::from([
                    (
                        "tozsamosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.tozsamosc),
                    ),
                    (
                        "otrzymano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.otrzymano),
                    ),
                    (
                        "wiadomosci_wplynely",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wiadomosci_wplynely),
                    ),
                    (
                        "karty_wyslane",
                        xmeta_room::ToSqlValue::to_sql_value(&params.karty_wyslane),
                    ),
                    (
                        "zaklocenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zaklocenia),
                    ),
                    (
                        "twierdzenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.twierdzenia),
                    ),
                ]),
            },
        ],
    )
}
