//! The kind `kierownik-biura-polisy` of the room `positions/biuro/kierownik_biura`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "kierownik-biura-polisy",
    requires: &[],
    migrations: &[
        r#"-- Polisy ubezpieczenia odpowiedzialności cywilnej biura: pośrednika i zarządcy. Kopia polisy
-- aktualnej na dzień zawarcia umowy jest załącznikiem do każdej umowy pośrednictwa i każdej umowy
-- o zarządzanie (art. 181 ust. 3a i art. 186 ust. 3a ugn, rozstrzygnięcie biura R6).
--
-- Kierownik pilnuje daty końca, przypomina o niej właścicielowi biura 30 dni wcześniej i o nowej
-- polisie zawiadamia tego samego dnia Kierownika pośrednictwa (polisa pośrednika) albo Kierownika
-- najmu (polisa zarządcy). Kopia leży w dokumentach biura, a wiersz trzyma do niej odwołanie. Nowa
-- polisa albo zmiana jej danych to nowy wiersz.
CREATE TABLE kierownik_biura_polisy_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_biura_polisy_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('posrednika', 'zarzadcy')),
  numer TEXT NOT NULL CHECK (numer <> ''),
  ubezpieczyciel TEXT NOT NULL CHECK (ubezpieczyciel <> ''),
  -- Okres ubezpieczenia, RRRR-MM-DD, oba dni włącznie.
  okres_od TEXT NOT NULL,
  okres_do TEXT NOT NULL,
  -- Kopia polisy w dokumentach biura, tekstem.
  dokument_odnosnik TEXT NOT NULL CHECK (dokument_odnosnik <> ''),
  zapisano TEXT NOT NULL,
  -- Kiedy kierownik przypomniał właścicielowi biura o końcu polisy.
  przypomniano TEXT,
  -- Kiedy i którego kierownika działu zawiadomił o tej polisie.
  zawiadomiono TEXT,
  zawiadomiono_kogo TEXT CHECK (zawiadomiono_kogo IN ('kierownik_posrednictwa', 'kierownik_najmu')),

  CONSTRAINT okres_od_przed_do CHECK (okres_od < okres_do),
  CONSTRAINT zawiadomienie_w_komplecie CHECK ((zawiadomiono IS NULL) = (zawiadomiono_kogo IS NULL)),
  -- O polisie pośrednika wie Pośrednictwo, a o polisie zarządcy Najem.
  CONSTRAINT zawiadomiony_kierownik_wlasciwy_dla_polisy CHECK (
    zawiadomiono_kogo IS NULL
    OR (rodzaj = 'posrednika') = (zawiadomiono_kogo = 'kierownik_posrednictwa')
  )
) STRICT;

CREATE TRIGGER kierownik_biura_polisy_powstaje_bez_zawiadomien
BEFORE INSERT ON kierownik_biura_polisy_wpisy
WHEN NEW.przypomniano IS NOT NULL
  OR NEW.zawiadomiono IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'polisa powstaje bez przypomnienia i zawiadomienia; odnotowuje się je później');
END;

-- Dane polisy pochodzą z dokumentu, więc się nie zmieniają; przypomnienie i zawiadomienie
-- odnotowuje się raz.
CREATE TRIGGER kierownik_biura_polisy_bez_zmian
BEFORE UPDATE ON kierownik_biura_polisy_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.numer IS NOT OLD.numer
  OR NEW.ubezpieczyciel IS NOT OLD.ubezpieczyciel
  OR NEW.okres_od IS NOT OLD.okres_od
  OR NEW.okres_do IS NOT OLD.okres_do
  OR NEW.dokument_odnosnik IS NOT OLD.dokument_odnosnik
  OR NEW.zapisano IS NOT OLD.zapisano
  OR (OLD.przypomniano IS NOT NULL AND NEW.przypomniano IS NOT OLD.przypomniano)
  OR (OLD.zawiadomiono IS NOT NULL AND (
    NEW.zawiadomiono IS NOT OLD.zawiadomiono
    OR NEW.zawiadomiono_kogo IS NOT OLD.zawiadomiono_kogo
  ))
BEGIN
  SELECT RAISE(ABORT, 'danych polisy się nie zmienia; nowa polisa to nowy wpis, a przypomnienie i zawiadomienie odnotowuje się raz');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_biura_polisy_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_biura_polisy_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_biura_polisy_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: dawna polisa zostaje, bo była załącznikiem umów zawartych w jej okresie.
CREATE TRIGGER kierownik_biura_polisy_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_biura_polisy_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
    ],
};

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    Posrednika,
    Zarzadcy,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[WpisyRodzaj::Posrednika, WpisyRodzaj::Zarzadcy];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::Posrednika => "posrednika",
            WpisyRodzaj::Zarzadcy => "zarzadcy",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "posrednika" => Ok(WpisyRodzaj::Posrednika),
                    "zarzadcy" => Ok(WpisyRodzaj::Zarzadcy),
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

/// The values of column `wpisy.zawiadomiono_kogo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyZawiadomionoKogo {
    KierownikPosrednictwa,
    KierownikNajmu,
}

impl WpisyZawiadomionoKogo {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyZawiadomionoKogo] = &[
        WpisyZawiadomionoKogo::KierownikPosrednictwa,
        WpisyZawiadomionoKogo::KierownikNajmu,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyZawiadomionoKogo::KierownikPosrednictwa => "kierownik_posrednictwa",
            WpisyZawiadomionoKogo::KierownikNajmu => "kierownik_najmu",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyZawiadomionoKogo {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "kierownik_posrednictwa" => {
                        Ok(WpisyZawiadomionoKogo::KierownikPosrednictwa)
                    }
                    "kierownik_najmu" => Ok(WpisyZawiadomionoKogo::KierownikNajmu),
                    other => Err(format!("{other} is no value of WpisyZawiadomionoKogo")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyZawiadomionoKogo {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `polisy`.
#[derive(Debug, Clone, PartialEq)]
pub struct PolisyRow {
    pub id: i64,
    pub rodzaj: WpisyRodzaj,
    pub numer: String,
    pub ubezpieczyciel: String,
    pub okres_od: String,
    pub okres_do: String,
    pub dokument_odnosnik: String,
    pub zapisano: String,
    pub przypomniano: Option<String>,
    pub zawiadomiono: Option<String>,
    pub zawiadomiono_kogo: Option<WpisyZawiadomionoKogo>,
}

impl PolisyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PolisyRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            numer: row.get("numer")?,
            ubezpieczyciel: row.get("ubezpieczyciel")?,
            okres_od: row.get("okres_od")?,
            okres_do: row.get("okres_do")?,
            dokument_odnosnik: row.get("dokument_odnosnik")?,
            zapisano: row.get("zapisano")?,
            przypomniano: row.get("przypomniano")?,
            zawiadomiono: row.get("zawiadomiono")?,
            zawiadomiono_kogo: row.get("zawiadomiono_kogo")?,
        })
    }
}

/// Wszystkie polisy, najnowsze okresy pierwsze.
pub fn polisy(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PolisyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_biura_polisy_wpisy
ORDER BY okres_do DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(PolisyRow::from_row).collect()
}

/// The parameters of `polisa_w_dniu`.
#[derive(Debug, Clone, PartialEq)]
pub struct PolisaWDniuParams {
    pub rodzaj: WpisyRodzaj,
    pub dzien: String,
}

/// A row of `polisa_w_dniu`.
#[derive(Debug, Clone, PartialEq)]
pub struct PolisaWDniuRow {
    pub id: i64,
    pub rodzaj: WpisyRodzaj,
    pub numer: String,
    pub ubezpieczyciel: String,
    pub okres_od: String,
    pub okres_do: String,
    pub dokument_odnosnik: String,
    pub zapisano: String,
    pub przypomniano: Option<String>,
    pub zawiadomiono: Option<String>,
    pub zawiadomiono_kogo: Option<WpisyZawiadomionoKogo>,
}

impl PolisaWDniuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PolisaWDniuRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            numer: row.get("numer")?,
            ubezpieczyciel: row.get("ubezpieczyciel")?,
            okres_od: row.get("okres_od")?,
            okres_do: row.get("okres_do")?,
            dokument_odnosnik: row.get("dokument_odnosnik")?,
            zapisano: row.get("zapisano")?,
            przypomniano: row.get("przypomniano")?,
            zawiadomiono: row.get("zawiadomiono")?,
            zawiadomiono_kogo: row.get("zawiadomiono_kogo")?,
        })
    }
}

/// Polisy danego rodzaju aktualne w jednym dniu (RRRR-MM-DD), np. w dniu zawarcia umowy.
pub fn polisa_w_dniu(
    room: &impl xmeta_room::MetaRoom,
    params: &PolisaWDniuParams,
) -> Result<Vec<PolisaWDniuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_biura_polisy_wpisy
WHERE rodzaj = :rodzaj
  AND okres_od <= :dzien
  AND okres_do >= :dzien
ORDER BY okres_od, id"#,
        parameters: Vec::from([
            ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
            ("dzien", xmeta_room::ToSqlValue::to_sql_value(&params.dzien)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(PolisaWDniuRow::from_row).collect()
}

/// The parameters of `polisy_do_przypomnienia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PolisyDoPrzypomnieniaParams {
    pub do_dnia: String,
}

/// A row of `polisy_do_przypomnienia`.
#[derive(Debug, Clone, PartialEq)]
pub struct PolisyDoPrzypomnieniaRow {
    pub id: i64,
    pub rodzaj: WpisyRodzaj,
    pub numer: String,
    pub ubezpieczyciel: String,
    pub okres_od: String,
    pub okres_do: String,
    pub dokument_odnosnik: String,
    pub zapisano: String,
    pub przypomniano: Option<String>,
    pub zawiadomiono: Option<String>,
    pub zawiadomiono_kogo: Option<WpisyZawiadomionoKogo>,
}

impl PolisyDoPrzypomnieniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PolisyDoPrzypomnieniaRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            numer: row.get("numer")?,
            ubezpieczyciel: row.get("ubezpieczyciel")?,
            okres_od: row.get("okres_od")?,
            okres_do: row.get("okres_do")?,
            dokument_odnosnik: row.get("dokument_odnosnik")?,
            zapisano: row.get("zapisano")?,
            przypomniano: row.get("przypomniano")?,
            zawiadomiono: row.get("zawiadomiono")?,
            zawiadomiono_kogo: row.get("zawiadomiono_kogo")?,
        })
    }
}

/// Polisy, które kończą się nie później niż `do_dnia`, a kierownik jeszcze o nich nie przypomniał.
pub fn polisy_do_przypomnienia(
    room: &impl xmeta_room::MetaRoom,
    params: &PolisyDoPrzypomnieniaParams,
) -> Result<Vec<PolisyDoPrzypomnieniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_biura_polisy_wpisy
WHERE przypomniano IS NULL
  AND okres_do <= :do_dnia
ORDER BY okres_do, id"#,
        parameters: Vec::from([
            ("do_dnia", xmeta_room::ToSqlValue::to_sql_value(&params.do_dnia)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(PolisyDoPrzypomnieniaRow::from_row).collect()
}

/// The parameters of `zapisz_polise`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZapiszPoliseParams {
    pub rodzaj: WpisyRodzaj,
    pub numer: String,
    pub ubezpieczyciel: String,
    pub okres_od: String,
    pub okres_do: String,
    pub dokument_odnosnik: String,
    pub zapisano: String,
}

/// Nowa polisa albo zmienione dane polisy, z odwołaniem do kopii w dokumentach biura.
pub fn zapisz_polise(
    room: &impl xmeta_room::MetaRoom,
    params: &ZapiszPoliseParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_biura_polisy_wpisy (rodzaj, numer, ubezpieczyciel, okres_od, okres_do,
  dokument_odnosnik, zapisano)
VALUES (:rodzaj, :numer, :ubezpieczyciel, :okres_od, :okres_do, :dokument_odnosnik, :zapisano)"#,
                parameters: Vec::from([
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("numer", xmeta_room::ToSqlValue::to_sql_value(&params.numer)),
                    (
                        "ubezpieczyciel",
                        xmeta_room::ToSqlValue::to_sql_value(&params.ubezpieczyciel),
                    ),
                    ("okres_od", xmeta_room::ToSqlValue::to_sql_value(&params.okres_od)),
                    ("okres_do", xmeta_room::ToSqlValue::to_sql_value(&params.okres_do)),
                    (
                        "dokument_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_odnosnik),
                    ),
                    ("zapisano", xmeta_room::ToSqlValue::to_sql_value(&params.zapisano)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_przypomnienie_o_polisie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPrzypomnienieOPolisieParams {
    pub przypomniano: String,
    pub id: i64,
}

/// Kierownik przypomniał właścicielowi biura o końcu polisy.
pub fn odnotuj_przypomnienie_o_polisie(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPrzypomnienieOPolisieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_biura_polisy_wpisy
SET przypomniano = :przypomniano
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "przypomniano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przypomniano),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_zawiadomienie_o_polisie`.
#[derive(Debug, Clone, PartialEq)]
pub struct OdnotujZawiadomienieOPolisieParams {
    pub zawiadomiono: String,
    pub zawiadomiono_kogo: WpisyZawiadomionoKogo,
    pub id: i64,
}

/// Kierownik zawiadomił kierownika działu o nowej polisie.
pub fn odnotuj_zawiadomienie_o_polisie(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujZawiadomienieOPolisieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_biura_polisy_wpisy
SET zawiadomiono = :zawiadomiono,
  zawiadomiono_kogo = :zawiadomiono_kogo
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zawiadomiono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zawiadomiono),
                    ),
                    (
                        "zawiadomiono_kogo",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zawiadomiono_kogo),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
