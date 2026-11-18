//! The kind `kierownik-kontroli-dane-osobowe` of the room `positions/kontrola/kierownik_kontroli`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "kierownik-kontroli-dane-osobowe";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Dane osobowe: prośby osób o ich dane (art. 15–18, 20 i 21 RODO) i zgłoszone naruszenia ochrony
-- danych, z datą, terminem, stanem i odwołaniem do odpowiedzi właściciela. Kierownik przygotowuje
-- odpowiedź albo opis naruszenia, a załatwia sprawę właściciel.
--
-- Treść prośby leży w wiadomości, którą wpłynęła; wiersz ją tylko wskazuje. Prośby o usunięcie
-- danych (art. 17 RODO) pamięć nie wykona na własnych wierszach, dopóki obowiązuje rozstrzygnięcie
-- biura R9.
CREATE TABLE kierownik_kontroli_dane_osobowe_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_kontroli_dane_osobowe_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('dostep_art15', 'sprostowanie_art16', 'usuniecie_art17', 'ograniczenie_art18', 'przenoszenie_art20', 'sprzeciw_art21', 'naruszenie')),
  wplynelo TEXT NOT NULL,
  -- Wiadomość z prośbą albo zgłoszenie naruszenia, tekstem.
  wplyw_odnosnik TEXT NOT NULL CHECK (wplyw_odnosnik <> ''),
  -- Termin, który policzył program, np. miesiąc na odpowiedź (art. 12 ust. 3 RODO) albo 72 godziny
  -- na zgłoszenie naruszenia (art. 33 ust. 1 RODO).
  termin TEXT NOT NULL,
  stan TEXT NOT NULL CHECK (stan IN ('otwarta', 'u_wlasciciela', 'zalatwiona')),
  -- Odpowiedź albo zgłoszenie, które wysłał właściciel, tekstem.
  odpowiedz_odnosnik TEXT,
  zalatwiono TEXT,

  CONSTRAINT termin_nie_przed_wplywem CHECK (termin >= wplynelo),
  CONSTRAINT zalatwiona_z_odpowiedzia_wlasciciela CHECK (
    (stan = 'zalatwiona') = (odpowiedz_odnosnik IS NOT NULL)
    AND (stan = 'zalatwiona') = (zalatwiono IS NOT NULL)
  ),
  CONSTRAINT zalatwienie_nie_przed_wplywem CHECK (zalatwiono IS NULL OR zalatwiono >= wplynelo)
) STRICT;

CREATE TRIGGER kierownik_kontroli_dane_osobowe_powstaje_otwarta
BEFORE INSERT ON kierownik_kontroli_dane_osobowe_wpisy
WHEN NEW.stan <> 'otwarta'
BEGIN
  SELECT RAISE(ABORT, 'sprawa danych osobowych powstaje otwarta');
END;

CREATE TRIGGER kierownik_kontroli_dane_osobowe_bez_zmian
BEFORE UPDATE ON kierownik_kontroli_dane_osobowe_wpisy
WHEN OLD.stan = 'zalatwiona'
  OR NEW.id IS NOT OLD.id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.wplynelo IS NOT OLD.wplynelo
  OR NEW.wplyw_odnosnik IS NOT OLD.wplyw_odnosnik
BEGIN
  SELECT RAISE(ABORT, 'rodzaju ani wpływu sprawy danych osobowych się nie zmienia, a załatwionej wcale');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_kontroli_dane_osobowe_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_kontroli_dane_osobowe_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_kontroli_dane_osobowe_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: załatwione sprawy zostają jako ślad dla organu nadzorczego.
CREATE TRIGGER kierownik_kontroli_dane_osobowe_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_kontroli_dane_osobowe_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
];

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    DostepArt15,
    SprostowanieArt16,
    UsuniecieArt17,
    OgraniczenieArt18,
    PrzenoszenieArt20,
    SprzeciwArt21,
    Naruszenie,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::DostepArt15,
        WpisyRodzaj::SprostowanieArt16,
        WpisyRodzaj::UsuniecieArt17,
        WpisyRodzaj::OgraniczenieArt18,
        WpisyRodzaj::PrzenoszenieArt20,
        WpisyRodzaj::SprzeciwArt21,
        WpisyRodzaj::Naruszenie,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::DostepArt15 => "dostep_art15",
            WpisyRodzaj::SprostowanieArt16 => "sprostowanie_art16",
            WpisyRodzaj::UsuniecieArt17 => "usuniecie_art17",
            WpisyRodzaj::OgraniczenieArt18 => "ograniczenie_art18",
            WpisyRodzaj::PrzenoszenieArt20 => "przenoszenie_art20",
            WpisyRodzaj::SprzeciwArt21 => "sprzeciw_art21",
            WpisyRodzaj::Naruszenie => "naruszenie",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "dostep_art15" => Ok(WpisyRodzaj::DostepArt15),
                    "sprostowanie_art16" => Ok(WpisyRodzaj::SprostowanieArt16),
                    "usuniecie_art17" => Ok(WpisyRodzaj::UsuniecieArt17),
                    "ograniczenie_art18" => Ok(WpisyRodzaj::OgraniczenieArt18),
                    "przenoszenie_art20" => Ok(WpisyRodzaj::PrzenoszenieArt20),
                    "sprzeciw_art21" => Ok(WpisyRodzaj::SprzeciwArt21),
                    "naruszenie" => Ok(WpisyRodzaj::Naruszenie),
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

/// The values of column `wpisy.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyStan {
    Otwarta,
    UWlasciciela,
    Zalatwiona,
}

impl WpisyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyStan] = &[
        WpisyStan::Otwarta,
        WpisyStan::UWlasciciela,
        WpisyStan::Zalatwiona,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyStan::Otwarta => "otwarta",
            WpisyStan::UWlasciciela => "u_wlasciciela",
            WpisyStan::Zalatwiona => "zalatwiona",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "otwarta" => Ok(WpisyStan::Otwarta),
                    "u_wlasciciela" => Ok(WpisyStan::UWlasciciela),
                    "zalatwiona" => Ok(WpisyStan::Zalatwiona),
                    other => Err(format!("{other} is no value of WpisyStan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyStan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `sprawy_danych_osobowych`.
#[derive(Debug, Clone, PartialEq)]
pub struct SprawyDanychOsobowychRow {
    pub id: i64,
    pub rodzaj: WpisyRodzaj,
    pub wplynelo: String,
    pub wplyw_odnosnik: String,
    pub termin: String,
    pub stan: WpisyStan,
    pub odpowiedz_odnosnik: Option<String>,
    pub zalatwiono: Option<String>,
}

impl SprawyDanychOsobowychRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(SprawyDanychOsobowychRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            wplynelo: row.get("wplynelo")?,
            wplyw_odnosnik: row.get("wplyw_odnosnik")?,
            termin: row.get("termin")?,
            stan: row.get("stan")?,
            odpowiedz_odnosnik: row.get("odpowiedz_odnosnik")?,
            zalatwiono: row.get("zalatwiono")?,
        })
    }
}

/// Sprawy niezałatwione od najbliższego terminu, potem załatwione od najnowszej.
pub fn sprawy_danych_osobowych(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<SprawyDanychOsobowychRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_kontroli_dane_osobowe_wpisy
ORDER BY stan = 'zalatwiona', CASE WHEN stan <> 'zalatwiona' THEN termin END, wplynelo DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(SprawyDanychOsobowychRow::from_row).collect()
}

/// The parameters of `zapisz_sprawe_danych_osobowych`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZapiszSpraweDanychOsobowychParams {
    pub rodzaj: WpisyRodzaj,
    pub wplynelo: String,
    pub wplyw_odnosnik: String,
    pub termin: String,
}

/// Nowa prośba osoby o jej dane albo zgłoszone naruszenie, z terminem z programu.
pub fn zapisz_sprawe_danych_osobowych(
    room: &impl xmeta_room::MetaRoom,
    params: &ZapiszSpraweDanychOsobowychParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_kontroli_dane_osobowe_wpisy (rodzaj, wplynelo, wplyw_odnosnik, termin, stan)
VALUES (:rodzaj, :wplynelo, :wplyw_odnosnik, :termin, 'otwarta')"#,
                parameters: Vec::from([
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("wplynelo", xmeta_room::ToSqlValue::to_sql_value(&params.wplynelo)),
                    (
                        "wplyw_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wplyw_odnosnik),
                    ),
                    ("termin", xmeta_room::ToSqlValue::to_sql_value(&params.termin)),
                ]),
            },
        ],
    )
}

/// The parameters of `daj_sprawe_wlascicielowi`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DajSpraweWlascicielowiParams {
    pub id: i64,
}

/// Odpowiedź albo opis naruszenia czeka u właściciela.
pub fn daj_sprawe_wlascicielowi(
    room: &impl xmeta_room::MetaRoom,
    params: &DajSpraweWlascicielowiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_kontroli_dane_osobowe_wpisy
SET stan = 'u_wlasciciela'
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_zalatwienie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujZalatwienieParams {
    pub odpowiedz_odnosnik: String,
    pub zalatwiono: String,
    pub id: i64,
}

/// Właściciel wysłał odpowiedź albo zgłoszenie naruszenia.
pub fn odnotuj_zalatwienie(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujZalatwienieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_kontroli_dane_osobowe_wpisy
SET stan = 'zalatwiona',
  odpowiedz_odnosnik = :odpowiedz_odnosnik,
  zalatwiono = :zalatwiono
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "odpowiedz_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odpowiedz_odnosnik),
                    ),
                    (
                        "zalatwiono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zalatwiono),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
