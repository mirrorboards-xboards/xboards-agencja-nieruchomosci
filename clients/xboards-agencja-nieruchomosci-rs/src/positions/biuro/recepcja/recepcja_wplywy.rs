//! The kind `recepcja-wplywy` of the room `positions/biuro/recepcja`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "recepcja-wplywy";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Dziennik wpływów: kiedy wiadomość wpłynęła, którym kanałem, od kogo, jakiego jest rodzaju, do
-- której sprawy należy, po jakim znaku ją przypisano i komu ją oddano. Treści wiadomości tu nie ma:
-- plik czeka w skrzynce, z której przyszedł, a wiersz wskazuje tylko wiadomość.
--
-- Wiadomość trafia do sprawy tylko po znaku: po znaku ze spisu spraw albo po numerze oferty lub
-- ogłoszenia z katalogu ofert, a nie po tym, co sama o sobie twierdzi. Wpis powstaje, gdy Recepcja
-- wiadomość rozdzieliła, i jest tylko dopisywany. Pism od Generalnego Inspektora Informacji
-- Finansowej dziennik nie zna (art. 54 ust. 1 AML).
CREATE TABLE recepcja_wplywy_dziennik (
  -- Klucz nadany przez bazę jest dodatni; zob. recepcja_wplywy_dziennik_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Co najmniej do minuty: RRRR-MM-DDTHH:MM, bo od doręczenia może biec termin.
  wplynelo TEXT NOT NULL,
  kanal TEXT NOT NULL CHECK (kanal IN ('email', 'formularz', 'portal', 'telefon', 'skan_poczty')),
  -- Adres, numer albo konto na portalu, z którego wiadomość przyszła.
  nadawca TEXT NOT NULL CHECK (nadawca <> ''),
  -- Np. „zapytanie o ofertę”, „zaświadczenie spółdzielni”, „wypowiedzenie umowy najmu”.
  rodzaj_pisma TEXT NOT NULL CHECK (rodzaj_pisma <> ''),
  -- Gdzie plik czeka w skrzynce, tekstem.
  wiadomosc_odnosnik TEXT NOT NULL CHECK (wiadomosc_odnosnik <> ''),
  sprawa_id INTEGER REFERENCES recepcja_sprawy_wpisy (id),
  znak_id INTEGER REFERENCES recepcja_sprawy_znaki (id),
  -- Wpis katalogu ofert, którego numer oferty albo ogłoszenia podała wiadomość, tekstem.
  katalog_odnosnik TEXT,
  -- Agent sprawy (także Konsjerż lokalu), karta dla Kierownika biura, pracownik biura, do którego
  -- pismo zaadresowano imiennie, albo nigdzie, np. automatyczna odpowiedź.
  oddano TEXT NOT NULL CHECK (oddano IN ('agent_sprawy', 'karta', 'pracownik', 'nigdzie')),
  -- Tożsamość agenta albo pracownika, któremu oddano wiadomość.
  oddano_komu TEXT,
  oddano_o TEXT,
  -- Wiadomość przyszła innym kanałem, niż ten nadawca zwykle pisze.
  kanal_nie_pasuje INTEGER NOT NULL CHECK (kanal_nie_pasuje IN (0, 1)),

  CONSTRAINT wplynelo_co_do_minuty CHECK (length(wplynelo) >= 16),
  CONSTRAINT sprawa_tylko_po_znaku_albo_katalogu CHECK (
    (sprawa_id IS NULL) = (znak_id IS NULL AND katalog_odnosnik IS NULL)
    AND (znak_id IS NULL OR katalog_odnosnik IS NULL)
  ),
  CONSTRAINT agent_sprawy_dostaje_wiadomosc_ze_sprawa CHECK (oddano <> 'agent_sprawy' OR sprawa_id IS NOT NULL),
  CONSTRAINT oddane_komus CHECK ((oddano IN ('agent_sprawy', 'pracownik')) = (oddano_komu IS NOT NULL)),
  CONSTRAINT oddane_z_czasem CHECK ((oddano = 'nigdzie') = (oddano_o IS NULL)),
  CONSTRAINT oddanie_nie_przed_wplywem CHECK (oddano_o IS NULL OR oddano_o >= wplynelo)
) STRICT;

-- Znak musi należeć do wskazanej sprawy i nie być wycofany, bo inaczej wiadomość trafiłaby do
-- cudzej sprawy.
CREATE TRIGGER recepcja_wplywy_znak_tej_sprawy
BEFORE INSERT ON recepcja_wplywy_dziennik
WHEN NEW.znak_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM recepcja_sprawy_znaki AS znak
    WHERE znak.id = NEW.znak_id
      AND znak.sprawa_id = NEW.sprawa_id
      AND znak.wycofano IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'wiadomość trafia do sprawy po jej niewycofanym znaku');
END;

-- Po numerze z katalogu ofert wiadomość trafia do oferty, a oferta jest sprawą spisu.
CREATE TRIGGER recepcja_wplywy_katalog_wskazuje_oferte
BEFORE INSERT ON recepcja_wplywy_dziennik
WHEN NEW.katalog_odnosnik IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM recepcja_sprawy_wpisy AS sprawa
    WHERE sprawa.id = NEW.sprawa_id
      AND sprawa.rodzaj = 'oferta'
  )
BEGIN
  SELECT RAISE(ABORT, 'po numerze z katalogu ofert wiadomość trafia do sprawy oferty');
END;

CREATE TRIGGER recepcja_wplywy_bez_zmian
BEFORE UPDATE ON recepcja_wplywy_dziennik
BEGIN
  SELECT RAISE(ABORT, 'dziennik wpływów tylko się dopisuje; zmiany nie są dozwolone');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER recepcja_wplywy_dziennik_bez_zastepowania
BEFORE INSERT ON recepcja_wplywy_dziennik
WHEN EXISTS (
  SELECT 1
  FROM recepcja_wplywy_dziennik
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: dziennik jest śladem, że żadna wiadomość nie zginęła.
CREATE TRIGGER recepcja_wplywy_dziennik_bez_usuwania
BEFORE DELETE ON recepcja_wplywy_dziennik
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); dziennik wpływów tylko się dopisuje');
END;
"#,
];

/// The values of column `dziennik.kanal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DziennikKanal {
    Email,
    Formularz,
    Portal,
    Telefon,
    SkanPoczty,
}

impl DziennikKanal {
    /// Every value, in the order of the catalog.
    pub const ALL: &[DziennikKanal] = &[
        DziennikKanal::Email,
        DziennikKanal::Formularz,
        DziennikKanal::Portal,
        DziennikKanal::Telefon,
        DziennikKanal::SkanPoczty,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            DziennikKanal::Email => "email",
            DziennikKanal::Formularz => "formularz",
            DziennikKanal::Portal => "portal",
            DziennikKanal::Telefon => "telefon",
            DziennikKanal::SkanPoczty => "skan_poczty",
        }
    }
}

impl xmeta_room::FromSqlValue for DziennikKanal {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "email" => Ok(DziennikKanal::Email),
                    "formularz" => Ok(DziennikKanal::Formularz),
                    "portal" => Ok(DziennikKanal::Portal),
                    "telefon" => Ok(DziennikKanal::Telefon),
                    "skan_poczty" => Ok(DziennikKanal::SkanPoczty),
                    other => Err(format!("{other} is no value of DziennikKanal")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for DziennikKanal {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `dziennik.oddano`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DziennikOddano {
    AgentSprawy,
    Karta,
    Pracownik,
    Nigdzie,
}

impl DziennikOddano {
    /// Every value, in the order of the catalog.
    pub const ALL: &[DziennikOddano] = &[
        DziennikOddano::AgentSprawy,
        DziennikOddano::Karta,
        DziennikOddano::Pracownik,
        DziennikOddano::Nigdzie,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            DziennikOddano::AgentSprawy => "agent_sprawy",
            DziennikOddano::Karta => "karta",
            DziennikOddano::Pracownik => "pracownik",
            DziennikOddano::Nigdzie => "nigdzie",
        }
    }
}

impl xmeta_room::FromSqlValue for DziennikOddano {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "agent_sprawy" => Ok(DziennikOddano::AgentSprawy),
                    "karta" => Ok(DziennikOddano::Karta),
                    "pracownik" => Ok(DziennikOddano::Pracownik),
                    "nigdzie" => Ok(DziennikOddano::Nigdzie),
                    other => Err(format!("{other} is no value of DziennikOddano")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for DziennikOddano {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The parameters of `dziennik_dnia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DziennikDniaParams {
    pub dzien: String,
}

/// A row of `dziennik_dnia`.
#[derive(Debug, Clone, PartialEq)]
pub struct DziennikDniaRow {
    pub id: i64,
    pub wplynelo: String,
    pub kanal: DziennikKanal,
    pub nadawca: String,
    pub rodzaj_pisma: String,
    pub wiadomosc_odnosnik: String,
    pub sprawa_id: Option<i64>,
    pub znak_id: Option<i64>,
    pub katalog_odnosnik: Option<String>,
    pub oddano: DziennikOddano,
    pub oddano_komu: Option<String>,
    pub oddano_o: Option<String>,
    pub kanal_nie_pasuje: bool,
}

impl DziennikDniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(DziennikDniaRow {
            id: row.get("id")?,
            wplynelo: row.get("wplynelo")?,
            kanal: row.get("kanal")?,
            nadawca: row.get("nadawca")?,
            rodzaj_pisma: row.get("rodzaj_pisma")?,
            wiadomosc_odnosnik: row.get("wiadomosc_odnosnik")?,
            sprawa_id: row.get("sprawa_id")?,
            znak_id: row.get("znak_id")?,
            katalog_odnosnik: row.get("katalog_odnosnik")?,
            oddano: row.get("oddano")?,
            oddano_komu: row.get("oddano_komu")?,
            oddano_o: row.get("oddano_o")?,
            kanal_nie_pasuje: row.get("kanal_nie_pasuje")?,
        })
    }
}

/// Wpisy dziennika z jednego dnia (RRRR-MM-DD), od najwcześniejszego.
pub fn dziennik_dnia(
    room: &impl xmeta_room::MetaRoom,
    params: &DziennikDniaParams,
) -> Result<Vec<DziennikDniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM recepcja_wplywy_dziennik
WHERE substr(wplynelo, 1, 10) = :dzien
ORDER BY wplynelo, id"#,
        parameters: Vec::from([
            ("dzien", xmeta_room::ToSqlValue::to_sql_value(&params.dzien)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(DziennikDniaRow::from_row).collect()
}

/// The parameters of `wplywy_sprawy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WplywySprawyParams {
    pub sprawa_id: i64,
}

/// A row of `wplywy_sprawy`.
#[derive(Debug, Clone, PartialEq)]
pub struct WplywySprawyRow {
    pub id: i64,
    pub wplynelo: String,
    pub kanal: DziennikKanal,
    pub nadawca: String,
    pub rodzaj_pisma: String,
    pub wiadomosc_odnosnik: String,
    pub sprawa_id: Option<i64>,
    pub znak_id: Option<i64>,
    pub katalog_odnosnik: Option<String>,
    pub oddano: DziennikOddano,
    pub oddano_komu: Option<String>,
    pub oddano_o: Option<String>,
    pub kanal_nie_pasuje: bool,
}

impl WplywySprawyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WplywySprawyRow {
            id: row.get("id")?,
            wplynelo: row.get("wplynelo")?,
            kanal: row.get("kanal")?,
            nadawca: row.get("nadawca")?,
            rodzaj_pisma: row.get("rodzaj_pisma")?,
            wiadomosc_odnosnik: row.get("wiadomosc_odnosnik")?,
            sprawa_id: row.get("sprawa_id")?,
            znak_id: row.get("znak_id")?,
            katalog_odnosnik: row.get("katalog_odnosnik")?,
            oddano: row.get("oddano")?,
            oddano_komu: row.get("oddano_komu")?,
            oddano_o: row.get("oddano_o")?,
            kanal_nie_pasuje: row.get("kanal_nie_pasuje")?,
        })
    }
}

/// Wszystkie wiadomości jednej sprawy, od najwcześniejszej.
pub fn wplywy_sprawy(
    room: &impl xmeta_room::MetaRoom,
    params: &WplywySprawyParams,
) -> Result<Vec<WplywySprawyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM recepcja_wplywy_dziennik
WHERE sprawa_id = :sprawa_id
ORDER BY wplynelo, id"#,
        parameters: Vec::from([
            ("sprawa_id", xmeta_room::ToSqlValue::to_sql_value(&params.sprawa_id)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(WplywySprawyRow::from_row).collect()
}

/// The parameters of `wplywy_od_nadawcy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WplywyOdNadawcyParams {
    pub nadawca: String,
}

/// A row of `wplywy_od_nadawcy`.
#[derive(Debug, Clone, PartialEq)]
pub struct WplywyOdNadawcyRow {
    pub id: i64,
    pub wplynelo: String,
    pub kanal: DziennikKanal,
    pub nadawca: String,
    pub rodzaj_pisma: String,
    pub wiadomosc_odnosnik: String,
    pub sprawa_id: Option<i64>,
    pub znak_id: Option<i64>,
    pub katalog_odnosnik: Option<String>,
    pub oddano: DziennikOddano,
    pub oddano_komu: Option<String>,
    pub oddano_o: Option<String>,
    pub kanal_nie_pasuje: bool,
}

impl WplywyOdNadawcyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WplywyOdNadawcyRow {
            id: row.get("id")?,
            wplynelo: row.get("wplynelo")?,
            kanal: row.get("kanal")?,
            nadawca: row.get("nadawca")?,
            rodzaj_pisma: row.get("rodzaj_pisma")?,
            wiadomosc_odnosnik: row.get("wiadomosc_odnosnik")?,
            sprawa_id: row.get("sprawa_id")?,
            znak_id: row.get("znak_id")?,
            katalog_odnosnik: row.get("katalog_odnosnik")?,
            oddano: row.get("oddano")?,
            oddano_komu: row.get("oddano_komu")?,
            oddano_o: row.get("oddano_o")?,
            kanal_nie_pasuje: row.get("kanal_nie_pasuje")?,
        })
    }
}

/// Wiadomości od jednego nadawcy, od najwcześniejszej; z nich widać, czy kanał do niego pasuje.
pub fn wplywy_od_nadawcy(
    room: &impl xmeta_room::MetaRoom,
    params: &WplywyOdNadawcyParams,
) -> Result<Vec<WplywyOdNadawcyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM recepcja_wplywy_dziennik
WHERE nadawca = :nadawca
ORDER BY wplynelo, id"#,
        parameters: Vec::from([
            ("nadawca", xmeta_room::ToSqlValue::to_sql_value(&params.nadawca)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(WplywyOdNadawcyRow::from_row).collect()
}

/// The parameters of `wplywy_wedlug_kanalow`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WplywyWedlugKanalowParams {
    pub dzien: String,
}

/// A row of `wplywy_wedlug_kanalow`.
#[derive(Debug, Clone, PartialEq)]
pub struct WplywyWedlugKanalowRow {
    pub kanal: DziennikKanal,
    pub liczba: i64,
}

impl WplywyWedlugKanalowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WplywyWedlugKanalowRow {
            kanal: row.get("kanal")?,
            liczba: row.get("liczba")?,
        })
    }
}

/// Ile wiadomości wpłynęło jednego dnia (RRRR-MM-DD) każdym kanałem, do meldunku.
pub fn wplywy_wedlug_kanalow(
    room: &impl xmeta_room::MetaRoom,
    params: &WplywyWedlugKanalowParams,
) -> Result<Vec<WplywyWedlugKanalowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT kanal,
  count(*) AS liczba
FROM recepcja_wplywy_dziennik
WHERE substr(wplynelo, 1, 10) = :dzien
GROUP BY kanal
ORDER BY kanal"#,
        parameters: Vec::from([
            ("dzien", xmeta_room::ToSqlValue::to_sql_value(&params.dzien)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(WplywyWedlugKanalowRow::from_row).collect()
}

/// The parameters of `odnotuj_wplyw`.
#[derive(Debug, Clone, PartialEq)]
pub struct OdnotujWplywParams {
    pub wplynelo: String,
    pub kanal: DziennikKanal,
    pub nadawca: String,
    pub rodzaj_pisma: String,
    pub wiadomosc_odnosnik: String,
    pub sprawa_id: Option<i64>,
    pub znak_id: Option<i64>,
    pub katalog_odnosnik: Option<String>,
    pub oddano: DziennikOddano,
    pub oddano_komu: Option<String>,
    pub oddano_o: Option<String>,
    pub kanal_nie_pasuje: bool,
}

/// Wiadomość rozdzielona: sprawa i znak albo wpis katalogu, jeśli pasują, oraz komu ją oddano.
pub fn odnotuj_wplyw(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWplywParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO recepcja_wplywy_dziennik (wplynelo, kanal, nadawca, rodzaj_pisma, wiadomosc_odnosnik,
  sprawa_id, znak_id, katalog_odnosnik, oddano, oddano_komu, oddano_o, kanal_nie_pasuje)
VALUES (:wplynelo, :kanal, :nadawca, :rodzaj_pisma, :wiadomosc_odnosnik, :sprawa_id, :znak_id,
  :katalog_odnosnik, :oddano, :oddano_komu, :oddano_o, :kanal_nie_pasuje)"#,
                parameters: Vec::from([
                    ("wplynelo", xmeta_room::ToSqlValue::to_sql_value(&params.wplynelo)),
                    ("kanal", xmeta_room::ToSqlValue::to_sql_value(&params.kanal)),
                    ("nadawca", xmeta_room::ToSqlValue::to_sql_value(&params.nadawca)),
                    (
                        "rodzaj_pisma",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj_pisma),
                    ),
                    (
                        "wiadomosc_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wiadomosc_odnosnik),
                    ),
                    (
                        "sprawa_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawa_id),
                    ),
                    ("znak_id", xmeta_room::ToSqlValue::to_sql_value(&params.znak_id)),
                    (
                        "katalog_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.katalog_odnosnik),
                    ),
                    ("oddano", xmeta_room::ToSqlValue::to_sql_value(&params.oddano)),
                    (
                        "oddano_komu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.oddano_komu),
                    ),
                    ("oddano_o", xmeta_room::ToSqlValue::to_sql_value(&params.oddano_o)),
                    (
                        "kanal_nie_pasuje",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kanal_nie_pasuje),
                    ),
                ]),
            },
        ],
    )
}
