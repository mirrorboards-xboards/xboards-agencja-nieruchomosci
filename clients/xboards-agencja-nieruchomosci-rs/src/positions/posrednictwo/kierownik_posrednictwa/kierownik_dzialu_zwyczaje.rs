//! The kind `kierownik-dzialu-zwyczaje` of the room `positions/posrednictwo/kierownik_posrednictwa`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "kierownik-dzialu-zwyczaje",
    requires: &[&super::stanowisko_zrodla::KIND],
    migrations: &[
        r#"-- Zwyczaje kontrahentów: jak portal, kanał sprzedaży, kancelaria, bank, spółdzielnia, wspólnota,
-- urząd, wykonawca albo deweloper zwykle postępuje, z tematem, opisem, źródłem, pewnością i datą.
-- To ustalenia kierownika o kontrahentach; polecenia leżą w stanowisko-polecenia.
--
-- Źródło zwyczaju to wiersz w stanowisko-zrodla, bo gdy źródło poda nieprawdę, kierownik ocenia na nowo
-- wszystkie zwyczaje od niego (operacja zwyczaje_od_zrodla). Raport albo wiadomość, z której zwyczaj
-- pochodzi, wskazuje odnośnik.
--
-- Zwyczaj jest pewny tylko z potwierdzeniem: regulaminem, potwierdzeniem kontrahenta albo podpisem
-- człowieka biura. Podpis przychodzi w wiadomości, a kierownik przenosi go do wiersza; sprawdza go
-- aplikacja. Zwyczaj, który okazał się nieprawdziwy, traci pewność albo zostaje wykreślony;
-- wykreślony zostaje w pamięci jako ślad.
CREATE TABLE kierownik_dzialu_zwyczaje_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_dzialu_zwyczaje_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  kontrahent_rodzaj TEXT NOT NULL CHECK (kontrahent_rodzaj IN ('portal', 'kanal_sprzedazy', 'kancelaria', 'bank', 'spoldzielnia', 'wspolnota', 'urzad', 'wykonawca', 'deweloper')),
  -- Np. „Spółdzielnia Mieszkaniowa Leśna” albo nazwa portalu.
  kontrahent TEXT NOT NULL CHECK (kontrahent <> ''),
  temat TEXT NOT NULL,
  opis TEXT NOT NULL,
  zrodlo_id TEXT REFERENCES stanowisko_zrodla_wpisy (id),
  -- Raport, wiadomość albo wiersz teczki, z którego zwyczaj pochodzi, tekstem.
  zrodlo_odnosnik TEXT,
  ustalono TEXT NOT NULL,
  pewnosc TEXT NOT NULL CHECK (pewnosc IN ('pewny', 'prawdopodobny', 'niepewny')),
  potwierdzenie TEXT CHECK (potwierdzenie IN ('regulamin', 'kontrahent', 'czlowiek_biura')),
  -- Regulamin albo wiadomość kontrahenta, tekstem.
  potwierdzenie_odnosnik TEXT,
  potwierdzil TEXT,
  podpis TEXT,
  potwierdzono TEXT,
  wykreslono TEXT,
  powod_wykreslenia TEXT,

  CONSTRAINT zwyczaj_ze_zrodlem CHECK (zrodlo_id IS NOT NULL OR zrodlo_odnosnik IS NOT NULL),
  CONSTRAINT pewny_z_potwierdzeniem CHECK ((pewnosc = 'pewny') = (potwierdzenie IS NOT NULL)),
  CONSTRAINT potwierdzenie_z_data CHECK ((potwierdzenie IS NULL) = (potwierdzono IS NULL)),
  CONSTRAINT potwierdzenie_regulaminem_albo_kontrahenta_wskazane CHECK (
    (potwierdzenie IS NOT NULL AND potwierdzenie IN ('regulamin', 'kontrahent')) = (potwierdzenie_odnosnik IS NOT NULL)
  ),
  CONSTRAINT potwierdzenie_czlowieka_biura_z_podpisem CHECK (
    (potwierdzenie IS 'czlowiek_biura') = (potwierdzil IS NOT NULL)
    AND (potwierdzil IS NULL) = (podpis IS NULL)
  ),
  CONSTRAINT wykreslenie_z_powodem CHECK ((wykreslono IS NULL) = (powod_wykreslenia IS NULL))
) STRICT;

-- Potwierdzenie i wykreślenie przychodzą później, więc zwyczaj powstaje bez nich, a przez to nie
-- jest jeszcze pewny.
CREATE TRIGGER kierownik_dzialu_zwyczaje_powstaje_niepotwierdzony
BEFORE INSERT ON kierownik_dzialu_zwyczaje_wpisy
WHEN NEW.potwierdzenie IS NOT NULL
  OR NEW.wykreslono IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'zwyczaj powstaje niepotwierdzony i niewykreślony');
END;

CREATE TRIGGER kierownik_dzialu_zwyczaje_wykreslony_bez_zmian
BEFORE UPDATE ON kierownik_dzialu_zwyczaje_wpisy
WHEN OLD.wykreslono IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'wykreślonego zwyczaju się nie zmienia');
END;

-- Na tym, kogo zwyczaj dotyczy i skąd pochodzi, stoi ponowna ocena od źródła; inny zwyczaj to nowy
-- wpis.
CREATE TRIGGER kierownik_dzialu_zwyczaje_tresc_niezmienna
BEFORE UPDATE ON kierownik_dzialu_zwyczaje_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.kontrahent_rodzaj IS NOT OLD.kontrahent_rodzaj
  OR NEW.kontrahent IS NOT OLD.kontrahent
  OR NEW.temat IS NOT OLD.temat
  OR NEW.opis IS NOT OLD.opis
  OR NEW.zrodlo_id IS NOT OLD.zrodlo_id
  OR NEW.zrodlo_odnosnik IS NOT OLD.zrodlo_odnosnik
  OR NEW.ustalono IS NOT OLD.ustalono
BEGIN
  SELECT RAISE(ABORT, 'kontrahenta, treści, źródła ani daty zwyczaju się nie zmienia; inny zwyczaj to nowy wpis');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_dzialu_zwyczaje_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_dzialu_zwyczaje_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_dzialu_zwyczaje_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: zwyczaj, który przestał obowiązywać, wykreśla się, a nie usuwa.
CREATE TRIGGER kierownik_dzialu_zwyczaje_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_dzialu_zwyczaje_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
    ],
};

/// The values of column `wpisy.kontrahent_rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyKontrahentRodzaj {
    Portal,
    KanalSprzedazy,
    Kancelaria,
    Bank,
    Spoldzielnia,
    Wspolnota,
    Urzad,
    Wykonawca,
    Deweloper,
}

impl WpisyKontrahentRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyKontrahentRodzaj] = &[
        WpisyKontrahentRodzaj::Portal,
        WpisyKontrahentRodzaj::KanalSprzedazy,
        WpisyKontrahentRodzaj::Kancelaria,
        WpisyKontrahentRodzaj::Bank,
        WpisyKontrahentRodzaj::Spoldzielnia,
        WpisyKontrahentRodzaj::Wspolnota,
        WpisyKontrahentRodzaj::Urzad,
        WpisyKontrahentRodzaj::Wykonawca,
        WpisyKontrahentRodzaj::Deweloper,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyKontrahentRodzaj::Portal => "portal",
            WpisyKontrahentRodzaj::KanalSprzedazy => "kanal_sprzedazy",
            WpisyKontrahentRodzaj::Kancelaria => "kancelaria",
            WpisyKontrahentRodzaj::Bank => "bank",
            WpisyKontrahentRodzaj::Spoldzielnia => "spoldzielnia",
            WpisyKontrahentRodzaj::Wspolnota => "wspolnota",
            WpisyKontrahentRodzaj::Urzad => "urzad",
            WpisyKontrahentRodzaj::Wykonawca => "wykonawca",
            WpisyKontrahentRodzaj::Deweloper => "deweloper",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyKontrahentRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "portal" => Ok(WpisyKontrahentRodzaj::Portal),
                    "kanal_sprzedazy" => Ok(WpisyKontrahentRodzaj::KanalSprzedazy),
                    "kancelaria" => Ok(WpisyKontrahentRodzaj::Kancelaria),
                    "bank" => Ok(WpisyKontrahentRodzaj::Bank),
                    "spoldzielnia" => Ok(WpisyKontrahentRodzaj::Spoldzielnia),
                    "wspolnota" => Ok(WpisyKontrahentRodzaj::Wspolnota),
                    "urzad" => Ok(WpisyKontrahentRodzaj::Urzad),
                    "wykonawca" => Ok(WpisyKontrahentRodzaj::Wykonawca),
                    "deweloper" => Ok(WpisyKontrahentRodzaj::Deweloper),
                    other => Err(format!("{other} is no value of WpisyKontrahentRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyKontrahentRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.pewnosc`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyPewnosc {
    Pewny,
    Prawdopodobny,
    Niepewny,
}

impl WpisyPewnosc {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyPewnosc] = &[
        WpisyPewnosc::Pewny,
        WpisyPewnosc::Prawdopodobny,
        WpisyPewnosc::Niepewny,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyPewnosc::Pewny => "pewny",
            WpisyPewnosc::Prawdopodobny => "prawdopodobny",
            WpisyPewnosc::Niepewny => "niepewny",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyPewnosc {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "pewny" => Ok(WpisyPewnosc::Pewny),
                    "prawdopodobny" => Ok(WpisyPewnosc::Prawdopodobny),
                    "niepewny" => Ok(WpisyPewnosc::Niepewny),
                    other => Err(format!("{other} is no value of WpisyPewnosc")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyPewnosc {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.potwierdzenie`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyPotwierdzenie {
    Regulamin,
    Kontrahent,
    CzlowiekBiura,
}

impl WpisyPotwierdzenie {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyPotwierdzenie] = &[
        WpisyPotwierdzenie::Regulamin,
        WpisyPotwierdzenie::Kontrahent,
        WpisyPotwierdzenie::CzlowiekBiura,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyPotwierdzenie::Regulamin => "regulamin",
            WpisyPotwierdzenie::Kontrahent => "kontrahent",
            WpisyPotwierdzenie::CzlowiekBiura => "czlowiek_biura",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyPotwierdzenie {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "regulamin" => Ok(WpisyPotwierdzenie::Regulamin),
                    "kontrahent" => Ok(WpisyPotwierdzenie::Kontrahent),
                    "czlowiek_biura" => Ok(WpisyPotwierdzenie::CzlowiekBiura),
                    other => Err(format!("{other} is no value of WpisyPotwierdzenie")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyPotwierdzenie {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of field `pewnosc` of `wpisz_zwyczaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pewnosc {
    Prawdopodobny,
    Niepewny,
}

impl Pewnosc {
    /// Every value, in the order of the catalog.
    pub const ALL: &[Pewnosc] = &[Pewnosc::Prawdopodobny, Pewnosc::Niepewny];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            Pewnosc::Prawdopodobny => "prawdopodobny",
            Pewnosc::Niepewny => "niepewny",
        }
    }
}

impl xmeta_room::FromSqlValue for Pewnosc {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "prawdopodobny" => Ok(Pewnosc::Prawdopodobny),
                    "niepewny" => Ok(Pewnosc::Niepewny),
                    other => Err(format!("{other} is no value of Pewnosc")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for Pewnosc {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The parameters of `zwyczaje_kontrahenta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZwyczajeKontrahentaParams {
    pub kontrahent: String,
}

/// A row of `zwyczaje_kontrahenta`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZwyczajeKontrahentaRow {
    pub id: i64,
    pub kontrahent_rodzaj: WpisyKontrahentRodzaj,
    pub kontrahent: String,
    pub temat: String,
    pub opis: String,
    pub zrodlo_id: Option<String>,
    pub zrodlo_odnosnik: Option<String>,
    pub ustalono: String,
    pub pewnosc: WpisyPewnosc,
    pub potwierdzenie: Option<WpisyPotwierdzenie>,
    pub potwierdzenie_odnosnik: Option<String>,
    pub potwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub potwierdzono: Option<String>,
    pub wykreslono: Option<String>,
    pub powod_wykreslenia: Option<String>,
}

impl ZwyczajeKontrahentaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZwyczajeKontrahentaRow {
            id: row.get("id")?,
            kontrahent_rodzaj: row.get("kontrahent_rodzaj")?,
            kontrahent: row.get("kontrahent")?,
            temat: row.get("temat")?,
            opis: row.get("opis")?,
            zrodlo_id: row.get("zrodlo_id")?,
            zrodlo_odnosnik: row.get("zrodlo_odnosnik")?,
            ustalono: row.get("ustalono")?,
            pewnosc: row.get("pewnosc")?,
            potwierdzenie: row.get("potwierdzenie")?,
            potwierdzenie_odnosnik: row.get("potwierdzenie_odnosnik")?,
            potwierdzil: row.get("potwierdzil")?,
            podpis: row.get("podpis")?,
            potwierdzono: row.get("potwierdzono")?,
            wykreslono: row.get("wykreslono")?,
            powod_wykreslenia: row.get("powod_wykreslenia")?,
        })
    }
}

/// Niewykreślone zwyczaje jednego kontrahenta; pewne pierwsze.
pub fn zwyczaje_kontrahenta(
    room: &impl xmeta_room::MetaRoom,
    params: &ZwyczajeKontrahentaParams,
) -> Result<Vec<ZwyczajeKontrahentaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_dzialu_zwyczaje_wpisy
WHERE kontrahent = :kontrahent
  AND wykreslono IS NULL
ORDER BY pewnosc = 'niepewny', pewnosc = 'prawdopodobny', temat, id"#,
        parameters: Vec::from([
            ("kontrahent", xmeta_room::ToSqlValue::to_sql_value(&params.kontrahent)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZwyczajeKontrahentaRow::from_row).collect()
}

/// A row of `zwyczaje`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZwyczajeRow {
    pub id: i64,
    pub kontrahent_rodzaj: WpisyKontrahentRodzaj,
    pub kontrahent: String,
    pub temat: String,
    pub opis: String,
    pub zrodlo_id: Option<String>,
    pub zrodlo_odnosnik: Option<String>,
    pub ustalono: String,
    pub pewnosc: WpisyPewnosc,
    pub potwierdzenie: Option<WpisyPotwierdzenie>,
    pub potwierdzenie_odnosnik: Option<String>,
    pub potwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub potwierdzono: Option<String>,
    pub wykreslono: Option<String>,
    pub powod_wykreslenia: Option<String>,
}

impl ZwyczajeRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZwyczajeRow {
            id: row.get("id")?,
            kontrahent_rodzaj: row.get("kontrahent_rodzaj")?,
            kontrahent: row.get("kontrahent")?,
            temat: row.get("temat")?,
            opis: row.get("opis")?,
            zrodlo_id: row.get("zrodlo_id")?,
            zrodlo_odnosnik: row.get("zrodlo_odnosnik")?,
            ustalono: row.get("ustalono")?,
            pewnosc: row.get("pewnosc")?,
            potwierdzenie: row.get("potwierdzenie")?,
            potwierdzenie_odnosnik: row.get("potwierdzenie_odnosnik")?,
            potwierdzil: row.get("potwierdzil")?,
            podpis: row.get("podpis")?,
            potwierdzono: row.get("potwierdzono")?,
            wykreslono: row.get("wykreslono")?,
            powod_wykreslenia: row.get("powod_wykreslenia")?,
        })
    }
}

/// Wszystkie zwyczaje, także wykreślone, według rodzaju kontrahenta, kontrahenta i tematu.
pub fn zwyczaje(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZwyczajeRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_dzialu_zwyczaje_wpisy
ORDER BY kontrahent_rodzaj, kontrahent, temat, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZwyczajeRow::from_row).collect()
}

/// The parameters of `zwyczaje_od_zrodla`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZwyczajeOdZrodlaParams {
    pub zrodlo_id: String,
}

/// A row of `zwyczaje_od_zrodla`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZwyczajeOdZrodlaRow {
    pub id: i64,
    pub kontrahent_rodzaj: WpisyKontrahentRodzaj,
    pub kontrahent: String,
    pub temat: String,
    pub opis: String,
    pub zrodlo_id: Option<String>,
    pub zrodlo_odnosnik: Option<String>,
    pub ustalono: String,
    pub pewnosc: WpisyPewnosc,
    pub potwierdzenie: Option<WpisyPotwierdzenie>,
    pub potwierdzenie_odnosnik: Option<String>,
    pub potwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub potwierdzono: Option<String>,
    pub wykreslono: Option<String>,
    pub powod_wykreslenia: Option<String>,
}

impl ZwyczajeOdZrodlaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZwyczajeOdZrodlaRow {
            id: row.get("id")?,
            kontrahent_rodzaj: row.get("kontrahent_rodzaj")?,
            kontrahent: row.get("kontrahent")?,
            temat: row.get("temat")?,
            opis: row.get("opis")?,
            zrodlo_id: row.get("zrodlo_id")?,
            zrodlo_odnosnik: row.get("zrodlo_odnosnik")?,
            ustalono: row.get("ustalono")?,
            pewnosc: row.get("pewnosc")?,
            potwierdzenie: row.get("potwierdzenie")?,
            potwierdzenie_odnosnik: row.get("potwierdzenie_odnosnik")?,
            potwierdzil: row.get("potwierdzil")?,
            podpis: row.get("podpis")?,
            potwierdzono: row.get("potwierdzono")?,
            wykreslono: row.get("wykreslono")?,
            powod_wykreslenia: row.get("powod_wykreslenia")?,
        })
    }
}

/// Niewykreślone zwyczaje od jednego źródła; po jego nieprawdzie kierownik ocenia je na nowo.
pub fn zwyczaje_od_zrodla(
    room: &impl xmeta_room::MetaRoom,
    params: &ZwyczajeOdZrodlaParams,
) -> Result<Vec<ZwyczajeOdZrodlaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_dzialu_zwyczaje_wpisy
WHERE zrodlo_id = :zrodlo_id
  AND wykreslono IS NULL
ORDER BY ustalono, id"#,
        parameters: Vec::from([
            ("zrodlo_id", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZwyczajeOdZrodlaRow::from_row).collect()
}

/// The parameters of `wpisz_zwyczaj`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszZwyczajParams {
    pub kontrahent_rodzaj: WpisyKontrahentRodzaj,
    pub kontrahent: String,
    pub temat: String,
    pub opis: String,
    pub zrodlo_id: Option<String>,
    pub zrodlo_odnosnik: Option<String>,
    pub ustalono: String,
    pub pewnosc: Pewnosc,
}

/// Zwyczaj zauważony przez kierownika, jeszcze bez potwierdzenia: prawdopodobny albo niepewny.
pub fn wpisz_zwyczaj(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszZwyczajParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_dzialu_zwyczaje_wpisy (kontrahent_rodzaj, kontrahent, temat, opis, zrodlo_id,
  zrodlo_odnosnik, ustalono, pewnosc)
VALUES (:kontrahent_rodzaj, :kontrahent, :temat, :opis, :zrodlo_id, :zrodlo_odnosnik, :ustalono,
  :pewnosc)"#,
                parameters: Vec::from([
                    (
                        "kontrahent_rodzaj",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kontrahent_rodzaj),
                    ),
                    (
                        "kontrahent",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kontrahent),
                    ),
                    ("temat", xmeta_room::ToSqlValue::to_sql_value(&params.temat)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    (
                        "zrodlo_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_id),
                    ),
                    (
                        "zrodlo_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_odnosnik),
                    ),
                    ("ustalono", xmeta_room::ToSqlValue::to_sql_value(&params.ustalono)),
                    ("pewnosc", xmeta_room::ToSqlValue::to_sql_value(&params.pewnosc)),
                ]),
            },
        ],
    )
}

/// The parameters of `potwierdz_zwyczaj`.
#[derive(Debug, Clone, PartialEq)]
pub struct PotwierdzZwyczajParams {
    pub potwierdzenie: WpisyPotwierdzenie,
    pub potwierdzenie_odnosnik: Option<String>,
    pub potwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub potwierdzono: String,
    pub id: i64,
}

/// Zwyczaj potwierdził regulamin albo kontrahent (odnośnik) albo człowiek biura podpisaną
/// wiadomością (potwierdził i podpis), więc zwyczaj jest pewny.
pub fn potwierdz_zwyczaj(
    room: &impl xmeta_room::MetaRoom,
    params: &PotwierdzZwyczajParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_dzialu_zwyczaje_wpisy
SET pewnosc = 'pewny',
  potwierdzenie = :potwierdzenie,
  potwierdzenie_odnosnik = :potwierdzenie_odnosnik,
  potwierdzil = :potwierdzil,
  podpis = :podpis,
  potwierdzono = :potwierdzono
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "potwierdzenie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.potwierdzenie),
                    ),
                    (
                        "potwierdzenie_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.potwierdzenie_odnosnik,
                        ),
                    ),
                    (
                        "potwierdzil",
                        xmeta_room::ToSqlValue::to_sql_value(&params.potwierdzil),
                    ),
                    ("podpis", xmeta_room::ToSqlValue::to_sql_value(&params.podpis)),
                    (
                        "potwierdzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.potwierdzono),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `zakwestionuj_zwyczaj`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZakwestionujZwyczajParams {
    pub pewnosc: Pewnosc,
    pub id: i64,
}

/// Zwyczaj okazał się wątpliwy, np. dwa raporty mówią co innego albo źródło podało nieprawdę, więc
/// traci potwierdzenie i dostaje nową pewność.
pub fn zakwestionuj_zwyczaj(
    room: &impl xmeta_room::MetaRoom,
    params: &ZakwestionujZwyczajParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_dzialu_zwyczaje_wpisy
SET pewnosc = :pewnosc,
  potwierdzenie = NULL,
  potwierdzenie_odnosnik = NULL,
  potwierdzil = NULL,
  podpis = NULL,
  potwierdzono = NULL
WHERE id = :id"#,
                parameters: Vec::from([
                    ("pewnosc", xmeta_room::ToSqlValue::to_sql_value(&params.pewnosc)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wykresl_zwyczaj`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WykreslZwyczajParams {
    pub wykreslono: String,
    pub powod_wykreslenia: String,
    pub id: i64,
}

/// Zwyczaj okazał się nieprawdziwy albo przestał obowiązywać; zostaje w pamięci jako wykreślony.
pub fn wykresl_zwyczaj(
    room: &impl xmeta_room::MetaRoom,
    params: &WykreslZwyczajParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_dzialu_zwyczaje_wpisy
SET wykreslono = :wykreslono,
  powod_wykreslenia = :powod_wykreslenia
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "wykreslono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wykreslono),
                    ),
                    (
                        "powod_wykreslenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.powod_wykreslenia),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
