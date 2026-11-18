//! The kind `kierownik-dzialu-raporty` of the room `positions/kontrola/kierownik_kontroli`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "kierownik-dzialu-raporty";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Raporty agentów działu o stałych polach i prośby o decyzje, które kierownik przekazał dalej.
--
-- Raport nie powtarza treści prośby: wskazuje jej wiersz w teczce. Kierownik kontroli przekazuje
-- prośby właścicielowi sam, z pominięciem listy decyzji Kierownika biura, bo ich treść, a nawet to,
-- że są, może dotyczyć przeciwdziałania praniu pieniędzy; pozostali kierownicy przekazują je
-- Kierownikowi biura.
CREATE TABLE kierownik_dzialu_raporty_wpisy (
  id TEXT PRIMARY KEY,
  agent_id TEXT NOT NULL REFERENCES kierownik_dzialu_agenci_wpisy (id),
  otrzymano TEXT NOT NULL,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('biezacy', 'koncowy', 'okresowy', 'natychmiastowy')),
  etap_albo_okres TEXT NOT NULL,
  blokada TEXT,
  najblizszy_termin TEXT,
  -- Prośba o decyzję w teczce, tekstem: pokój i wiersz.
  decyzja_odnosnik TEXT,
  -- To, co raport mówi poza stałymi polami, np. jak skończyła się oferta.
  uwagi TEXT
) STRICT;

CREATE TABLE kierownik_dzialu_raporty_prosby (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_dzialu_raporty_prosby_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  decyzja_odnosnik TEXT NOT NULL,
  raport_id TEXT NOT NULL REFERENCES kierownik_dzialu_raporty_wpisy (id),
  przekazano_do TEXT NOT NULL CHECK (przekazano_do IN ('kierownik_biura', 'wlasciciel')),
  przekazano TEXT NOT NULL,
  -- Kiedy kierownik zgłosił, że prośbę można zdjąć z listy, i dlaczego: rozstrzygnięta albo zbędna.
  zgloszono_zdjecie TEXT,
  powod_zdjecia TEXT CHECK (powod_zdjecia IN ('rozstrzygnieta', 'zbedna')),

  CONSTRAINT zdjecie_z_powodem CHECK ((zgloszono_zdjecie IS NULL) = (powod_zdjecia IS NULL)),
  CONSTRAINT zdjecie_nie_przed_przekazaniem CHECK (zgloszono_zdjecie IS NULL OR zgloszono_zdjecie >= przekazano)
) STRICT;

-- Tej samej prośby nie przekazuje się drugi raz, dopóki nie zgłoszono jej zdjęcia: powtórzona prośba
-- naciska na człowieka (błędne koło).
CREATE UNIQUE INDEX kierownik_dzialu_raporty_prosba_przekazana_raz
ON kierownik_dzialu_raporty_prosby (decyzja_odnosnik)
WHERE zgloszono_zdjecie IS NULL;

-- Druga otwarta prośba z innym kluczem zderzyłaby się z indeksem, a INSERT OR REPLACE usunąłby wtedy
-- pierwszą z ominięciem blokady usuwania; ten trigger odmawia wcześniej.
CREATE TRIGGER kierownik_dzialu_raporty_prosba_przekazana_raz_przed_zdjeciem
BEFORE INSERT ON kierownik_dzialu_raporty_prosby
WHEN NEW.zgloszono_zdjecie IS NULL
  AND EXISTS (
    SELECT 1
    FROM kierownik_dzialu_raporty_prosby
    WHERE decyzja_odnosnik = NEW.decyzja_odnosnik
      AND zgloszono_zdjecie IS NULL
      AND id IS NOT NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'tej prośby nie przekazuje się drugi raz, dopóki nie zgłoszono jej zdjęcia');
END;

CREATE TRIGGER kierownik_dzialu_raporty_prosba_powstaje_na_liscie
BEFORE INSERT ON kierownik_dzialu_raporty_prosby
WHEN NEW.zgloszono_zdjecie IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'przekazana prośba powstaje na liście; zdjęcie zgłasza się później');
END;

CREATE TRIGGER kierownik_dzialu_raporty_bez_zmian
BEFORE UPDATE ON kierownik_dzialu_raporty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'raporty tylko się dopisuje; zmiany nie są dozwolone');
END;

CREATE TRIGGER kierownik_dzialu_raporty_prosby_bez_zmian
BEFORE UPDATE ON kierownik_dzialu_raporty_prosby
WHEN OLD.zgloszono_zdjecie IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.decyzja_odnosnik IS NOT OLD.decyzja_odnosnik
  OR NEW.raport_id IS NOT OLD.raport_id
  OR NEW.przekazano_do IS NOT OLD.przekazano_do
  OR NEW.przekazano IS NOT OLD.przekazano
BEGIN
  SELECT RAISE(ABORT, 'przekazanej prośby się nie zmienia; wolno raz zgłosić jej zdjęcie');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabeli z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER kierownik_dzialu_raporty_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_dzialu_raporty_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_dzialu_raporty_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER kierownik_dzialu_raporty_prosby_bez_zastepowania
BEFORE INSERT ON kierownik_dzialu_raporty_prosby
WHEN EXISTS (
  SELECT 1
  FROM kierownik_dzialu_raporty_prosby
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: raporty i prośby są śladem pracy działu.
CREATE TRIGGER kierownik_dzialu_raporty_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_dzialu_raporty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER kierownik_dzialu_raporty_prosby_bez_usuwania
BEFORE DELETE ON kierownik_dzialu_raporty_prosby
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
];

/// The values of column `prosby.przekazano_do`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProsbyPrzekazanoDo {
    KierownikBiura,
    Wlasciciel,
}

impl ProsbyPrzekazanoDo {
    /// Every value, in the order of the catalog.
    pub const ALL: &[ProsbyPrzekazanoDo] = &[
        ProsbyPrzekazanoDo::KierownikBiura,
        ProsbyPrzekazanoDo::Wlasciciel,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            ProsbyPrzekazanoDo::KierownikBiura => "kierownik_biura",
            ProsbyPrzekazanoDo::Wlasciciel => "wlasciciel",
        }
    }
}

impl xmeta_room::FromSqlValue for ProsbyPrzekazanoDo {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "kierownik_biura" => Ok(ProsbyPrzekazanoDo::KierownikBiura),
                    "wlasciciel" => Ok(ProsbyPrzekazanoDo::Wlasciciel),
                    other => Err(format!("{other} is no value of ProsbyPrzekazanoDo")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for ProsbyPrzekazanoDo {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `prosby.powod_zdjecia`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProsbyPowodZdjecia {
    Rozstrzygnieta,
    Zbedna,
}

impl ProsbyPowodZdjecia {
    /// Every value, in the order of the catalog.
    pub const ALL: &[ProsbyPowodZdjecia] = &[
        ProsbyPowodZdjecia::Rozstrzygnieta,
        ProsbyPowodZdjecia::Zbedna,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            ProsbyPowodZdjecia::Rozstrzygnieta => "rozstrzygnieta",
            ProsbyPowodZdjecia::Zbedna => "zbedna",
        }
    }
}

impl xmeta_room::FromSqlValue for ProsbyPowodZdjecia {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "rozstrzygnieta" => Ok(ProsbyPowodZdjecia::Rozstrzygnieta),
                    "zbedna" => Ok(ProsbyPowodZdjecia::Zbedna),
                    other => Err(format!("{other} is no value of ProsbyPowodZdjecia")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for ProsbyPowodZdjecia {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    Biezacy,
    Koncowy,
    Okresowy,
    Natychmiastowy,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::Biezacy,
        WpisyRodzaj::Koncowy,
        WpisyRodzaj::Okresowy,
        WpisyRodzaj::Natychmiastowy,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::Biezacy => "biezacy",
            WpisyRodzaj::Koncowy => "koncowy",
            WpisyRodzaj::Okresowy => "okresowy",
            WpisyRodzaj::Natychmiastowy => "natychmiastowy",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "biezacy" => Ok(WpisyRodzaj::Biezacy),
                    "koncowy" => Ok(WpisyRodzaj::Koncowy),
                    "okresowy" => Ok(WpisyRodzaj::Okresowy),
                    "natychmiastowy" => Ok(WpisyRodzaj::Natychmiastowy),
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

/// A row of `prosby_na_liscie`.
#[derive(Debug, Clone, PartialEq)]
pub struct ProsbyNaLiscieRow {
    pub id: i64,
    pub decyzja_odnosnik: String,
    pub raport_id: String,
    pub przekazano_do: ProsbyPrzekazanoDo,
    pub przekazano: String,
    pub zgloszono_zdjecie: Option<String>,
    pub powod_zdjecia: Option<ProsbyPowodZdjecia>,
}

impl ProsbyNaLiscieRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ProsbyNaLiscieRow {
            id: row.get("id")?,
            decyzja_odnosnik: row.get("decyzja_odnosnik")?,
            raport_id: row.get("raport_id")?,
            przekazano_do: row.get("przekazano_do")?,
            przekazano: row.get("przekazano")?,
            zgloszono_zdjecie: row.get("zgloszono_zdjecie")?,
            powod_zdjecia: row.get("powod_zdjecia")?,
        })
    }
}

/// Prośby przekazane dalej, których zdjęcia kierownik jeszcze nie zgłosił, od najstarszej.
pub fn prosby_na_liscie(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ProsbyNaLiscieRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_dzialu_raporty_prosby
WHERE zgloszono_zdjecie IS NULL
ORDER BY przekazano, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(ProsbyNaLiscieRow::from_row).collect()
}

/// The parameters of `przekaz_prosbe`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzekazProsbeParams {
    pub decyzja_odnosnik: String,
    pub raport_id: String,
    pub przekazano_do: ProsbyPrzekazanoDo,
    pub przekazano: String,
}

/// Prośba z raportu przekazana Kierownikowi biura albo, w Kontroli, właścicielowi.
pub fn przekaz_prosbe(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzekazProsbeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_dzialu_raporty_prosby (decyzja_odnosnik, raport_id, przekazano_do, przekazano)
VALUES (:decyzja_odnosnik, :raport_id, :przekazano_do, :przekazano)"#,
                parameters: Vec::from([
                    (
                        "decyzja_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_odnosnik),
                    ),
                    (
                        "raport_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.raport_id),
                    ),
                    (
                        "przekazano_do",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przekazano_do),
                    ),
                    (
                        "przekazano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przekazano),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zglos_zdjecie_prosby`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZglosZdjecieProsbyParams {
    pub zgloszono_zdjecie: String,
    pub powod_zdjecia: ProsbyPowodZdjecia,
    pub id: i64,
}

/// Prośbę rozstrzygnięto albo przestała być potrzebna, więc kierownik zgłasza, że można ją zdjąć
/// z listy.
pub fn zglos_zdjecie_prosby(
    room: &impl xmeta_room::MetaRoom,
    params: &ZglosZdjecieProsbyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_dzialu_raporty_prosby
SET zgloszono_zdjecie = :zgloszono_zdjecie,
  powod_zdjecia = :powod_zdjecia
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zgloszono_zdjecie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zgloszono_zdjecie),
                    ),
                    (
                        "powod_zdjecia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.powod_zdjecia),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `raporty`.
#[derive(Debug, Clone, PartialEq)]
pub struct RaportyRow {
    pub id: String,
    pub agent_id: String,
    pub otrzymano: String,
    pub rodzaj: WpisyRodzaj,
    pub etap_albo_okres: String,
    pub blokada: Option<String>,
    pub najblizszy_termin: Option<String>,
    pub decyzja_odnosnik: Option<String>,
    pub uwagi: Option<String>,
}

impl RaportyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(RaportyRow {
            id: row.get("id")?,
            agent_id: row.get("agent_id")?,
            otrzymano: row.get("otrzymano")?,
            rodzaj: row.get("rodzaj")?,
            etap_albo_okres: row.get("etap_albo_okres")?,
            blokada: row.get("blokada")?,
            najblizszy_termin: row.get("najblizszy_termin")?,
            decyzja_odnosnik: row.get("decyzja_odnosnik")?,
            uwagi: row.get("uwagi")?,
        })
    }
}

/// Raporty agentów działu, najnowsze pierwsze.
pub fn raporty(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<RaportyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_dzialu_raporty_wpisy
ORDER BY otrzymano DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(RaportyRow::from_row).collect()
}

/// The parameters of `raporty_agenta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RaportyAgentaParams {
    pub agent_id: String,
}

/// A row of `raporty_agenta`.
#[derive(Debug, Clone, PartialEq)]
pub struct RaportyAgentaRow {
    pub id: String,
    pub agent_id: String,
    pub otrzymano: String,
    pub rodzaj: WpisyRodzaj,
    pub etap_albo_okres: String,
    pub blokada: Option<String>,
    pub najblizszy_termin: Option<String>,
    pub decyzja_odnosnik: Option<String>,
    pub uwagi: Option<String>,
}

impl RaportyAgentaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(RaportyAgentaRow {
            id: row.get("id")?,
            agent_id: row.get("agent_id")?,
            otrzymano: row.get("otrzymano")?,
            rodzaj: row.get("rodzaj")?,
            etap_albo_okres: row.get("etap_albo_okres")?,
            blokada: row.get("blokada")?,
            najblizszy_termin: row.get("najblizszy_termin")?,
            decyzja_odnosnik: row.get("decyzja_odnosnik")?,
            uwagi: row.get("uwagi")?,
        })
    }
}

/// Raporty jednego agenta, od najstarszego.
pub fn raporty_agenta(
    room: &impl xmeta_room::MetaRoom,
    params: &RaportyAgentaParams,
) -> Result<Vec<RaportyAgentaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_dzialu_raporty_wpisy
WHERE agent_id = :agent_id
ORDER BY otrzymano, id"#,
        parameters: Vec::from([
            ("agent_id", xmeta_room::ToSqlValue::to_sql_value(&params.agent_id)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(RaportyAgentaRow::from_row).collect()
}

/// The parameters of `przyjmij_raport`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzyjmijRaportParams {
    pub id: String,
    pub agent_id: String,
    pub otrzymano: String,
    pub rodzaj: WpisyRodzaj,
    pub etap_albo_okres: String,
    pub blokada: Option<String>,
    pub najblizszy_termin: Option<String>,
    pub decyzja_odnosnik: Option<String>,
    pub uwagi: Option<String>,
}

/// Raport agenta o stałych polach; identyfikator (UUID) nadaje aplikacja, żeby prośba, zajrzenie
/// i rozesłanie polisy mogły go od razu wskazać.
pub fn przyjmij_raport(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzyjmijRaportParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_dzialu_raporty_wpisy (id, agent_id, otrzymano, rodzaj, etap_albo_okres, blokada,
  najblizszy_termin, decyzja_odnosnik, uwagi)
VALUES (:id, :agent_id, :otrzymano, :rodzaj, :etap_albo_okres, :blokada, :najblizszy_termin,
  :decyzja_odnosnik, :uwagi)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("agent_id", xmeta_room::ToSqlValue::to_sql_value(&params.agent_id)),
                    (
                        "otrzymano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.otrzymano),
                    ),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    (
                        "etap_albo_okres",
                        xmeta_room::ToSqlValue::to_sql_value(&params.etap_albo_okres),
                    ),
                    ("blokada", xmeta_room::ToSqlValue::to_sql_value(&params.blokada)),
                    (
                        "najblizszy_termin",
                        xmeta_room::ToSqlValue::to_sql_value(&params.najblizszy_termin),
                    ),
                    (
                        "decyzja_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_odnosnik),
                    ),
                    ("uwagi", xmeta_room::ToSqlValue::to_sql_value(&params.uwagi)),
                ]),
            },
        ],
    )
}
