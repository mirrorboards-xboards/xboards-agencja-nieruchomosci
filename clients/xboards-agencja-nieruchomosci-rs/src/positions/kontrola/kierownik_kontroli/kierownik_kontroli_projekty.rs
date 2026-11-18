//! The kind `kierownik-kontroli-projekty` of the room `positions/kontrola/kierownik_kontroli`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "kierownik-kontroli-projekty",
    requires: &[],
    migrations: &[
        r#"-- Projekty zawiadomień do Generalnego Inspektora Informacji Finansowej (art. 74, 86 i 90 AML)
-- i do prokuratora (art. 89 AML), z terminem, stanem i odwołaniem do wpisów w rejestrze kontroli,
-- na których stoją.
--
-- Projekt leży w pamięci, a nie w rejestrze, dopóki właściciel go nie przekaże, bo pamięć czyta
-- tylko on: krąg osób, które wiedzą o projekcie, jest najmniejszy z możliwych (art. 54 ust. 1 AML).
-- Tekst leży w repozytorium XGit pokoju pamięci; wiersz trzyma ścieżkę i skrót. Po przekazaniu
-- ślad zostaje w rejestrze kontroli, a wiersz traci ścieżkę i skrót i wskazuje tylko tamten wpis.
-- Projekt, którego właściciel nie przekaże, zamyka odstąpienie z jego podpisem, bo wierszy się nie
-- usuwa (R9).
CREATE TABLE kierownik_kontroli_projekty_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_kontroli_projekty_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Jak w rejestrze kontroli: art. 74, 86 i 90 AML do Generalnego Inspektora, art. 89 do prokuratora.
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('art74', 'art86', 'art90', 'art89_prokurator')),
  -- Pokój teczki, której projekt dotyczy, tekstem.
  teczka_odnosnik TEXT,
  -- Termin, który policzył program; projekt ma być u właściciela wcześniej.
  termin TEXT NOT NULL,
  utworzono TEXT NOT NULL,
  stan TEXT NOT NULL CHECK (stan IN ('projekt', 'u_wlasciciela', 'przekazany', 'odstapiono')),
  sciezka TEXT,
  skrot TEXT,
  u_wlasciciela_od TEXT,
  przekazano TEXT,
  -- Ślad zawiadomienia w rejestrze kontroli, tekstem.
  rejestr_odnosnik TEXT,
  -- Właściciel, który postanowił projektu nie przekazywać, jego podpis i chwila.
  odstapil TEXT,
  odstapienie_podpis TEXT,
  odstapiono TEXT,

  CONSTRAINT sciezka_razem_ze_skrotem CHECK ((sciezka IS NULL) = (skrot IS NULL)),
  CONSTRAINT projekt_z_plikiem CHECK (stan NOT IN ('projekt', 'u_wlasciciela') OR sciezka IS NOT NULL),
  CONSTRAINT u_wlasciciela_z_data CHECK (stan <> 'u_wlasciciela' OR u_wlasciciela_od IS NOT NULL),
  CONSTRAINT przekazany_z_wpisem_w_rejestrze CHECK (
    (stan = 'przekazany') = (rejestr_odnosnik IS NOT NULL)
    AND (stan = 'przekazany') = (przekazano IS NOT NULL)
  ),
  CONSTRAINT przekazany_bez_tresci CHECK (stan <> 'przekazany' OR sciezka IS NULL),
  CONSTRAINT odstapienie_z_podpisem_wlasciciela CHECK (
    (stan = 'odstapiono') = (odstapil IS NOT NULL)
    AND (odstapil IS NULL) = (odstapienie_podpis IS NULL)
    AND (odstapil IS NULL) = (odstapiono IS NULL)
  ),
  CONSTRAINT przekazanie_nie_przed_utworzeniem CHECK (przekazano IS NULL OR przekazano >= utworzono)
) STRICT;

-- Wpisy w rejestrze kontroli, na których stoi projekt, np. sprawdzenia i decyzja potwierdzenia
-- podejrzenia.
CREATE TABLE kierownik_kontroli_projekty_podstawy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_kontroli_projekty_podstawy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  projekt_id INTEGER NOT NULL REFERENCES kierownik_kontroli_projekty_wpisy (id),
  rejestr_odnosnik TEXT NOT NULL CHECK (rejestr_odnosnik <> ''),

  CONSTRAINT podstawa_raz_w_projekcie UNIQUE (projekt_id, rejestr_odnosnik)
) STRICT;

-- Projekty ---------------------------------------------------------------------------------------

-- Reguły przekazania i odstąpienia sprawdzają zmianę stanu, więc projekt nie może powstać od razu
-- przekazany.
CREATE TRIGGER kierownik_kontroli_projekty_powstaje_jako_projekt
BEFORE INSERT ON kierownik_kontroli_projekty_wpisy
WHEN NEW.stan <> 'projekt'
BEGIN
  SELECT RAISE(ABORT, 'projekt zawiadomienia powstaje w stanie projekt');
END;

CREATE TRIGGER kierownik_kontroli_projekty_przekazany_i_odstapiony_ostateczne
BEFORE UPDATE ON kierownik_kontroli_projekty_wpisy
WHEN OLD.stan IN ('przekazany', 'odstapiono')
  OR NEW.id IS NOT OLD.id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.teczka_odnosnik IS NOT OLD.teczka_odnosnik
  OR NEW.utworzono IS NOT OLD.utworzono
BEGIN
  SELECT RAISE(ABORT, 'rodzaju, teczki ani daty projektu się nie zmienia, a przekazanego i odstąpionego wcale');
END;

-- Podstawy ---------------------------------------------------------------------------------------

CREATE TRIGGER kierownik_kontroli_projekty_podstawa_projektu_w_toku
BEFORE INSERT ON kierownik_kontroli_projekty_podstawy
WHEN NOT EXISTS (
  SELECT 1
  FROM kierownik_kontroli_projekty_wpisy AS projekt
  WHERE projekt.id = NEW.projekt_id
    AND projekt.stan IN ('projekt', 'u_wlasciciela')
)
BEGIN
  SELECT RAISE(ABORT, 'podstawę dopisuje się tylko do projektu w toku');
END;

CREATE TRIGGER kierownik_kontroli_projekty_podstawy_bez_zmian
BEFORE UPDATE ON kierownik_kontroli_projekty_podstawy
BEGIN
  SELECT RAISE(ABORT, 'podstawy projektu się nie zmienia; inna podstawa to nowy wpis');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers; także podstawę o tym
-- samym wpisie rejestru. Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego
-- wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_kontroli_projekty_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_kontroli_projekty_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_kontroli_projekty_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER kierownik_kontroli_projekty_podstawy_bez_zastepowania
BEFORE INSERT ON kierownik_kontroli_projekty_podstawy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_kontroli_projekty_podstawy
  WHERE id = NEW.id
    OR (projekt_id = NEW.projekt_id AND rejestr_odnosnik = NEW.rejestr_odnosnik)
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: przekazany projekt wskazuje rejestr, a odstąpiony zostaje z podpisem właściciela.
CREATE TRIGGER kierownik_kontroli_projekty_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_kontroli_projekty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); niewysłany projekt zamyka odstąpienie');
END;

CREATE TRIGGER kierownik_kontroli_projekty_podstawy_bez_usuwania
BEFORE DELETE ON kierownik_kontroli_projekty_podstawy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
    ],
};

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    Art74,
    Art86,
    Art90,
    Art89Prokurator,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::Art74,
        WpisyRodzaj::Art86,
        WpisyRodzaj::Art90,
        WpisyRodzaj::Art89Prokurator,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::Art74 => "art74",
            WpisyRodzaj::Art86 => "art86",
            WpisyRodzaj::Art90 => "art90",
            WpisyRodzaj::Art89Prokurator => "art89_prokurator",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "art74" => Ok(WpisyRodzaj::Art74),
                    "art86" => Ok(WpisyRodzaj::Art86),
                    "art90" => Ok(WpisyRodzaj::Art90),
                    "art89_prokurator" => Ok(WpisyRodzaj::Art89Prokurator),
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
    Projekt,
    UWlasciciela,
    Przekazany,
    Odstapiono,
}

impl WpisyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyStan] = &[
        WpisyStan::Projekt,
        WpisyStan::UWlasciciela,
        WpisyStan::Przekazany,
        WpisyStan::Odstapiono,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyStan::Projekt => "projekt",
            WpisyStan::UWlasciciela => "u_wlasciciela",
            WpisyStan::Przekazany => "przekazany",
            WpisyStan::Odstapiono => "odstapiono",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "projekt" => Ok(WpisyStan::Projekt),
                    "u_wlasciciela" => Ok(WpisyStan::UWlasciciela),
                    "przekazany" => Ok(WpisyStan::Przekazany),
                    "odstapiono" => Ok(WpisyStan::Odstapiono),
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

/// The parameters of `podstawy_projektu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PodstawyProjektuParams {
    pub projekt_id: i64,
}

/// A row of `podstawy_projektu`.
#[derive(Debug, Clone, PartialEq)]
pub struct PodstawyProjektuRow {
    pub id: i64,
    pub projekt_id: i64,
    pub rejestr_odnosnik: String,
}

impl PodstawyProjektuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PodstawyProjektuRow {
            id: row.get("id")?,
            projekt_id: row.get("projekt_id")?,
            rejestr_odnosnik: row.get("rejestr_odnosnik")?,
        })
    }
}

/// Wpisy w rejestrze kontroli, na których stoi projekt.
pub fn podstawy_projektu(
    room: &impl xmeta_room::MetaRoom,
    params: &PodstawyProjektuParams,
) -> Result<Vec<PodstawyProjektuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_kontroli_projekty_podstawy
WHERE projekt_id = :projekt_id
ORDER BY id"#,
        parameters: Vec::from([
            ("projekt_id", xmeta_room::ToSqlValue::to_sql_value(&params.projekt_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(PodstawyProjektuRow::from_row).collect()
}

/// The parameters of `dopisz_podstawe`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DopiszPodstaweParams {
    pub projekt_id: i64,
    pub rejestr_odnosnik: String,
}

/// Wpis w rejestrze kontroli, na którym stoi projekt w toku.
pub fn dopisz_podstawe(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszPodstaweParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_kontroli_projekty_podstawy (projekt_id, rejestr_odnosnik)
VALUES (:projekt_id, :rejestr_odnosnik)"#,
                parameters: Vec::from([
                    (
                        "projekt_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.projekt_id),
                    ),
                    (
                        "rejestr_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rejestr_odnosnik),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `projekty_w_toku`.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjektyWTokuRow {
    pub id: i64,
    pub rodzaj: WpisyRodzaj,
    pub teczka_odnosnik: Option<String>,
    pub termin: String,
    pub utworzono: String,
    pub stan: WpisyStan,
    pub sciezka: Option<String>,
    pub skrot: Option<String>,
    pub u_wlasciciela_od: Option<String>,
    pub przekazano: Option<String>,
    pub rejestr_odnosnik: Option<String>,
    pub odstapil: Option<String>,
    pub odstapienie_podpis: Option<String>,
    pub odstapiono: Option<String>,
}

impl ProjektyWTokuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ProjektyWTokuRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            termin: row.get("termin")?,
            utworzono: row.get("utworzono")?,
            stan: row.get("stan")?,
            sciezka: row.get("sciezka")?,
            skrot: row.get("skrot")?,
            u_wlasciciela_od: row.get("u_wlasciciela_od")?,
            przekazano: row.get("przekazano")?,
            rejestr_odnosnik: row.get("rejestr_odnosnik")?,
            odstapil: row.get("odstapil")?,
            odstapienie_podpis: row.get("odstapienie_podpis")?,
            odstapiono: row.get("odstapiono")?,
        })
    }
}

/// Projekty, które jeszcze nie zostały przekazane ani odstąpione, od najbliższego terminu.
pub fn projekty_w_toku(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ProjektyWTokuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_kontroli_projekty_wpisy
WHERE stan IN ('projekt', 'u_wlasciciela')
ORDER BY termin, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ProjektyWTokuRow::from_row).collect()
}

/// A row of `projekty`.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjektyRow {
    pub id: i64,
    pub rodzaj: WpisyRodzaj,
    pub teczka_odnosnik: Option<String>,
    pub termin: String,
    pub utworzono: String,
    pub stan: WpisyStan,
    pub sciezka: Option<String>,
    pub skrot: Option<String>,
    pub u_wlasciciela_od: Option<String>,
    pub przekazano: Option<String>,
    pub rejestr_odnosnik: Option<String>,
    pub odstapil: Option<String>,
    pub odstapienie_podpis: Option<String>,
    pub odstapiono: Option<String>,
}

impl ProjektyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ProjektyRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            termin: row.get("termin")?,
            utworzono: row.get("utworzono")?,
            stan: row.get("stan")?,
            sciezka: row.get("sciezka")?,
            skrot: row.get("skrot")?,
            u_wlasciciela_od: row.get("u_wlasciciela_od")?,
            przekazano: row.get("przekazano")?,
            rejestr_odnosnik: row.get("rejestr_odnosnik")?,
            odstapil: row.get("odstapil")?,
            odstapienie_podpis: row.get("odstapienie_podpis")?,
            odstapiono: row.get("odstapiono")?,
        })
    }
}

/// Wszystkie projekty, najnowsze pierwsze.
pub fn projekty(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ProjektyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_kontroli_projekty_wpisy
ORDER BY utworzono DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ProjektyRow::from_row).collect()
}

/// The parameters of `utworz_projekt`.
#[derive(Debug, Clone, PartialEq)]
pub struct UtworzProjektParams {
    pub rodzaj: WpisyRodzaj,
    pub teczka_odnosnik: Option<String>,
    pub termin: String,
    pub utworzono: String,
    pub sciezka: Option<String>,
    pub skrot: Option<String>,
}

/// Nowy projekt zawiadomienia z plikiem w XGit pokoju pamięci.
pub fn utworz_projekt(
    room: &impl xmeta_room::MetaRoom,
    params: &UtworzProjektParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_kontroli_projekty_wpisy (rodzaj, teczka_odnosnik, termin, utworzono, stan,
  sciezka, skrot)
VALUES (:rodzaj, :teczka_odnosnik, :termin, :utworzono, 'projekt', :sciezka, :skrot)"#,
                parameters: Vec::from([
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    (
                        "teczka_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.teczka_odnosnik),
                    ),
                    ("termin", xmeta_room::ToSqlValue::to_sql_value(&params.termin)),
                    (
                        "utworzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.utworzono),
                    ),
                    ("sciezka", xmeta_room::ToSqlValue::to_sql_value(&params.sciezka)),
                    ("skrot", xmeta_room::ToSqlValue::to_sql_value(&params.skrot)),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_projekt`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZmienProjektParams {
    pub termin: String,
    pub sciezka: String,
    pub skrot: String,
    pub id: i64,
}

/// Nowa wersja pliku projektu albo nowy termin, póki projekt jest w toku.
pub fn zmien_projekt(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienProjektParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_kontroli_projekty_wpisy
SET termin = :termin,
  sciezka = :sciezka,
  skrot = :skrot
WHERE id = :id"#,
                parameters: Vec::from([
                    ("termin", xmeta_room::ToSqlValue::to_sql_value(&params.termin)),
                    ("sciezka", xmeta_room::ToSqlValue::to_sql_value(&params.sciezka)),
                    ("skrot", xmeta_room::ToSqlValue::to_sql_value(&params.skrot)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `daj_projekt_wlascicielowi`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DajProjektWlascicielowiParams {
    pub u_wlasciciela_od: String,
    pub id: i64,
}

/// Projekt trafił do właściciela, który potwierdza podejrzenie i przekazuje zawiadomienie.
pub fn daj_projekt_wlascicielowi(
    room: &impl xmeta_room::MetaRoom,
    params: &DajProjektWlascicielowiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_kontroli_projekty_wpisy
SET stan = 'u_wlasciciela',
  u_wlasciciela_od = :u_wlasciciela_od
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "u_wlasciciela_od",
                        xmeta_room::ToSqlValue::to_sql_value(&params.u_wlasciciela_od),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `cofnij_projekt_do_poprawy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CofnijProjektDoPoprawyParams {
    pub id: i64,
}

/// Właściciel oddał projekt do poprawy.
pub fn cofnij_projekt_do_poprawy(
    room: &impl xmeta_room::MetaRoom,
    params: &CofnijProjektDoPoprawyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_kontroli_projekty_wpisy
SET stan = 'projekt'
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_przekazanie_zawiadomienia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPrzekazanieZawiadomieniaParams {
    pub przekazano: String,
    pub rejestr_odnosnik: String,
    pub id: i64,
}

/// Właściciel przekazał zawiadomienie; ślad jest w rejestrze kontroli, a projekt traci plik i tylko
/// go wskazuje. Plik w XGit pokoju usuwa aplikacja.
pub fn odnotuj_przekazanie_zawiadomienia(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPrzekazanieZawiadomieniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_kontroli_projekty_wpisy
SET stan = 'przekazany',
  przekazano = :przekazano,
  rejestr_odnosnik = :rejestr_odnosnik,
  sciezka = NULL,
  skrot = NULL
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "przekazano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przekazano),
                    ),
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

/// The parameters of `odnotuj_odstapienie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujOdstapienieParams {
    pub odstapil: String,
    pub odstapienie_podpis: String,
    pub odstapiono: String,
    pub id: i64,
}

/// Właściciel postanowił projektu nie przekazywać i podpisał to.
pub fn odnotuj_odstapienie(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujOdstapienieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_kontroli_projekty_wpisy
SET stan = 'odstapiono',
  odstapil = :odstapil,
  odstapienie_podpis = :odstapienie_podpis,
  odstapiono = :odstapiono
WHERE id = :id"#,
                parameters: Vec::from([
                    ("odstapil", xmeta_room::ToSqlValue::to_sql_value(&params.odstapil)),
                    (
                        "odstapienie_podpis",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odstapienie_podpis),
                    ),
                    (
                        "odstapiono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odstapiono),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
