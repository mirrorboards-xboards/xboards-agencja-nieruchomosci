//! The kind `wymiana-dokumenty` of the room `cases/wymiana`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "wymiana-dokumenty";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Dokumenty pokoju wymiany: lista dokumentów kancelarii, dokumenty z tej listy, pełnomocnictwa
-- i zgody stron od biura, projekt aktu w wersjach i koszty od kancelarii oraz uwagi stron do
-- projektu. Każdy wiersz jest szczegółem jednego wpisu z wymiana_wpisy_wpisy, a stronę, autora,
-- czas i plik ma wpis. Wiersz dopisuje się razem z jego wpisem i potem się nie zmienia.
--
-- Biuro pisze w imieniu strony tylko z jej pełnomocnictwem (art. 98 KC; zasada „Pisanie bez
-- pełnomocnictwa” w profilu pokoju), więc dokument i zgoda strony wymagają wcześniejszego
-- pełnomocnictwa tej strony.

-- Lista dokumentów, których notariusz wymaga do aktu. Zmiana listy jest nowym wpisem z całą
-- listą; pozycja zachowuje identyfikator między wersjami listy, a dokument go wskazuje.
CREATE TABLE wymiana_dokumenty_lista (
  wpis_id TEXT NOT NULL REFERENCES wymiana_wpisy_wpisy (id),
  -- Identyfikator pozycji nadany przez kancelarię, np. „odpis-kw”.
  pozycja_id TEXT NOT NULL CHECK (pozycja_id <> ''),
  nazwa TEXT NOT NULL CHECK (nazwa <> ''),
  -- Strona, od której dokument ma pochodzić; NULL znaczy, że od którejkolwiek.
  od_strony_id TEXT REFERENCES wymiana_metryka_strony (id),
  postac TEXT NOT NULL CHECK (postac IN ('oryginal', 'odpis', 'skan')),
  -- Ile dni przed aktem dokument może najwyżej być wystawiony, gdy kancelaria tego wymaga.
  swiezosc_dni INTEGER CHECK (swiezosc_dni > 0),
  -- Dzień, do którego kancelaria chce dostać dokument.
  do_kiedy TEXT,

  PRIMARY KEY (wpis_id, pozycja_id)
) STRICT;

-- Dokument przekazany w pokoju: dokument z listy, pełnomocnictwo albo zgoda strony od biura,
-- projekt aktu albo koszty od kancelarii. Plik ma wpis.
CREATE TABLE wymiana_dokumenty_wpisy (
  wpis_id TEXT PRIMARY KEY REFERENCES wymiana_wpisy_wpisy (id),
  -- Taki sam jak rodzaj wpisu; pozwala nazwać wymagane kolumny każdego rodzaju.
  rodzaj TEXT NOT NULL CHECK (
    rodzaj IN ('dokument', 'pelnomocnictwo', 'zgoda_stron', 'projekt_aktu', 'koszty')
  ),
  -- Pozycja listy kancelarii, którą dokument wypełnia.
  pozycja_id TEXT CHECK (pozycja_id <> ''),
  -- Strona, w której imieniu biuro przekazuje dokument, której pełnomocnictwo albo zgodę.
  strona_id TEXT REFERENCES wymiana_metryka_strony (id),
  wystawca TEXT CHECK (wystawca <> ''),
  data_wystawienia TEXT,
  waznosc_do TEXT,
  -- Czy oryginał będzie w dniu aktu i kto go przyniesie.
  oryginal_w_dniu_aktu INTEGER CHECK (oryginal_w_dniu_aktu IN (0, 1)),
  kto_przyniesie TEXT CHECK (kto_przyniesie <> ''),
  -- Zakres pełnomocnictwa albo zgody, np. „uzgadnianie terminu aktu i przekazywanie dokumentów
  -- notariuszowi” albo „biuro dostaje projekt aktu i informację po akcie”.
  zakres TEXT CHECK (zakres <> ''),
  -- Wersja projektu aktu: kolejne wersje stoją obok siebie z rosnącym numerem.
  numer_wersji INTEGER CHECK (numer_wersji > 0),
  -- Kancelaria wskazuje, że to wersja, którą strony mają podpisać.
  ostateczna INTEGER CHECK (ostateczna IN (0, 1)),

  CONSTRAINT dokument_wypelnia_pozycje CHECK ((rodzaj = 'dokument') = (pozycja_id IS NOT NULL)),
  CONSTRAINT dokument_ma_wystawce_i_oryginal CHECK (
    rodzaj <> 'dokument'
    OR (wystawca IS NOT NULL AND data_wystawienia IS NOT NULL AND oryginal_w_dniu_aktu IS NOT NULL)
  ),
  CONSTRAINT oryginal_ma_kto_przyniesie CHECK ((oryginal_w_dniu_aktu IS 1) = (kto_przyniesie IS NOT NULL)),
  CONSTRAINT dokument_biura_wskazuje_strone CHECK (
    (rodzaj IN ('dokument', 'pelnomocnictwo', 'zgoda_stron')) = (strona_id IS NOT NULL)
  ),
  CONSTRAINT pelnomocnictwo_i_zgoda_z_zakresem CHECK (
    (rodzaj IN ('pelnomocnictwo', 'zgoda_stron')) = (zakres IS NOT NULL)
  ),
  CONSTRAINT projekt_ma_wersje CHECK (
    (rodzaj = 'projekt_aktu') = (numer_wersji IS NOT NULL)
    AND (rodzaj = 'projekt_aktu') = (ostateczna IS NOT NULL)
  )
) STRICT;

-- Uwagi stron do wersji projektu, które biuro przekazuje za zgodą człowieka. Decyzja leży
-- w teczce transakcji (decyzja wyslanie), więc wpis wskazuje ją tekstem.
CREATE TABLE wymiana_dokumenty_uwagi (
  wpis_id TEXT PRIMARY KEY REFERENCES wymiana_wpisy_wpisy (id),
  projekt_id TEXT NOT NULL REFERENCES wymiana_dokumenty_wpisy (wpis_id),
  -- Decyzja z teczki transakcji, która pozwoliła przekazać uwagi: identyfikator pokoju i decyzji.
  decyzja_odnosnik TEXT NOT NULL CHECK (decyzja_odnosnik <> '')
) STRICT;

-- Szczegół dopisuje się w tej samej operacji co jego wpis. Późniejszy dopisek zmieniałby wpis,
-- który druga strona już przeczytała, także w zamkniętym pokoju.
CREATE TRIGGER wymiana_dokumenty_lista_do_swojego_wpisu
BEFORE INSERT ON wymiana_dokumenty_lista
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
    AND rodzaj = 'lista_dokumentow'
    AND rowid = (
      SELECT max(rowid)
      FROM wymiana_wpisy_wpisy
    )
)
BEGIN
  SELECT RAISE(ABORT, 'pozycja listy dokumentów należy do najnowszego wpisu lista_dokumentow');
END;

CREATE TRIGGER wymiana_dokumenty_wpisy_do_swojego_wpisu
BEFORE INSERT ON wymiana_dokumenty_wpisy
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
    AND rodzaj = NEW.rodzaj
    AND rowid = (
      SELECT max(rowid)
      FROM wymiana_wpisy_wpisy
    )
)
BEGIN
  SELECT RAISE(ABORT, 'dokument należy do najnowszego wpisu tego samego rodzaju');
END;

CREATE TRIGGER wymiana_dokumenty_uwagi_do_swojego_wpisu
BEFORE INSERT ON wymiana_dokumenty_uwagi
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
    AND rodzaj = 'uwagi_do_projektu'
    AND rowid = (
      SELECT max(rowid)
      FROM wymiana_wpisy_wpisy
    )
)
BEGIN
  SELECT RAISE(ABORT, 'uwagi należą do najnowszego wpisu uwagi_do_projektu');
END;

-- Nowa wersja projektu nie zastępuje starej, tylko staje obok niej z wyższym numerem.
CREATE TRIGGER wymiana_dokumenty_wersja_projektu_rosnie
BEFORE INSERT ON wymiana_dokumenty_wpisy
WHEN NEW.rodzaj = 'projekt_aktu'
  AND NEW.numer_wersji <= (
    SELECT coalesce(max(numer_wersji), 0)
    FROM wymiana_dokumenty_wpisy
    WHERE rodzaj = 'projekt_aktu'
  )
BEGIN
  SELECT RAISE(ABORT, 'numer wersji projektu aktu rośnie');
END;

-- Pełnomocnictwo daje biuru strona, którą biuro reprezentuje według metryki pokoju.
CREATE TRIGGER wymiana_dokumenty_pelnomocnictwo_strony_biura
BEFORE INSERT ON wymiana_dokumenty_wpisy
WHEN NEW.rodzaj = 'pelnomocnictwo'
  AND NOT EXISTS (
    SELECT 1
    FROM wymiana_metryka_strony
    WHERE id = NEW.strona_id
      AND reprezentowana_przez_biuro = 1
  )
BEGIN
  SELECT RAISE(ABORT, 'pełnomocnictwo dotyczy strony, którą biuro reprezentuje według metryki');
END;

-- Pisanie bez pełnomocnictwa: dokument i zgodę strony biuro przekazuje dopiero po jej
-- pełnomocnictwie. Druga strona, która nie jest klientem biura, przekazuje swoje sama.
CREATE TRIGGER wymiana_dokumenty_wymagaja_pelnomocnictwa
BEFORE INSERT ON wymiana_dokumenty_wpisy
WHEN NEW.rodzaj IN ('dokument', 'zgoda_stron')
  AND NOT EXISTS (
    SELECT 1
    FROM wymiana_dokumenty_wpisy
    WHERE rodzaj = 'pelnomocnictwo'
      AND strona_id = NEW.strona_id
  )
BEGIN
  SELECT RAISE(ABORT, 'dokument biura dla strony wymaga wpisanego wcześniej pełnomocnictwa tej strony');
END;

CREATE TRIGGER wymiana_dokumenty_dokument_z_listy
BEFORE INSERT ON wymiana_dokumenty_wpisy
WHEN NEW.rodzaj = 'dokument'
  AND NOT EXISTS (
    SELECT 1
    FROM wymiana_dokumenty_lista
    WHERE pozycja_id = NEW.pozycja_id
  )
BEGIN
  SELECT RAISE(ABORT, 'dokument wypełnia pozycję z listy dokumentów kancelarii');
END;

-- Uwaga wskazuje wersję projektu, a nie inny dokument, żeby było widać, której wersji dotyczy.
CREATE TRIGGER wymiana_dokumenty_uwagi_do_projektu
BEFORE INSERT ON wymiana_dokumenty_uwagi
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_dokumenty_wpisy
  WHERE wpis_id = NEW.projekt_id
    AND rodzaj = 'projekt_aktu'
)
BEGIN
  SELECT RAISE(ABORT, 'uwagi wskazują wersję projektu aktu');
END;

CREATE TRIGGER wymiana_dokumenty_lista_bez_zmian
BEFORE UPDATE ON wymiana_dokumenty_lista
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

CREATE TRIGGER wymiana_dokumenty_wpisy_bez_zmian
BEFORE UPDATE ON wymiana_dokumenty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

CREATE TRIGGER wymiana_dokumenty_uwagi_bez_zmian
BEFORE UPDATE ON wymiana_dokumenty_uwagi
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_dokumenty_lista_bez_zastepowania
BEFORE INSERT ON wymiana_dokumenty_lista
WHEN EXISTS (
  SELECT 1
  FROM wymiana_dokumenty_lista
  WHERE wpis_id = NEW.wpis_id
    AND pozycja_id = NEW.pozycja_id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_dokumenty_lista_bez_usuwania
BEFORE DELETE ON wymiana_dokumenty_lista
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_dokumenty_wpisy_bez_zastepowania
BEFORE INSERT ON wymiana_dokumenty_wpisy
WHEN EXISTS (
  SELECT 1
  FROM wymiana_dokumenty_wpisy
  WHERE wpis_id = NEW.wpis_id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_dokumenty_wpisy_bez_usuwania
BEFORE DELETE ON wymiana_dokumenty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_dokumenty_uwagi_bez_zastepowania
BEFORE INSERT ON wymiana_dokumenty_uwagi
WHEN EXISTS (
  SELECT 1
  FROM wymiana_dokumenty_uwagi
  WHERE wpis_id = NEW.wpis_id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_dokumenty_uwagi_bez_usuwania
BEFORE DELETE ON wymiana_dokumenty_uwagi
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;
"#,
];

/// The values of column `lista.postac`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ListaPostac {
    Oryginal,
    Odpis,
    Skan,
}

impl ListaPostac {
    /// Every value, in the order of the catalog.
    pub const ALL: &[ListaPostac] = &[
        ListaPostac::Oryginal,
        ListaPostac::Odpis,
        ListaPostac::Skan,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            ListaPostac::Oryginal => "oryginal",
            ListaPostac::Odpis => "odpis",
            ListaPostac::Skan => "skan",
        }
    }
}

impl xmeta_room::FromSqlValue for ListaPostac {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "oryginal" => Ok(ListaPostac::Oryginal),
                    "odpis" => Ok(ListaPostac::Odpis),
                    "skan" => Ok(ListaPostac::Skan),
                    other => Err(format!("{other} is no value of ListaPostac")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for ListaPostac {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    Dokument,
    Pelnomocnictwo,
    ZgodaStron,
    ProjektAktu,
    Koszty,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::Dokument,
        WpisyRodzaj::Pelnomocnictwo,
        WpisyRodzaj::ZgodaStron,
        WpisyRodzaj::ProjektAktu,
        WpisyRodzaj::Koszty,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::Dokument => "dokument",
            WpisyRodzaj::Pelnomocnictwo => "pelnomocnictwo",
            WpisyRodzaj::ZgodaStron => "zgoda_stron",
            WpisyRodzaj::ProjektAktu => "projekt_aktu",
            WpisyRodzaj::Koszty => "koszty",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "dokument" => Ok(WpisyRodzaj::Dokument),
                    "pelnomocnictwo" => Ok(WpisyRodzaj::Pelnomocnictwo),
                    "zgoda_stron" => Ok(WpisyRodzaj::ZgodaStron),
                    "projekt_aktu" => Ok(WpisyRodzaj::ProjektAktu),
                    "koszty" => Ok(WpisyRodzaj::Koszty),
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

/// The values of field `strona` of `dokumenty_pokoju`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Strona {
    Biuro,
    Kancelaria,
}

impl Strona {
    /// Every value, in the order of the catalog.
    pub const ALL: &[Strona] = &[Strona::Biuro, Strona::Kancelaria];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            Strona::Biuro => "biuro",
            Strona::Kancelaria => "kancelaria",
        }
    }
}

impl xmeta_room::FromSqlValue for Strona {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "biuro" => Ok(Strona::Biuro),
                    "kancelaria" => Ok(Strona::Kancelaria),
                    other => Err(format!("{other} is no value of Strona")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for Strona {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `dokumenty_pokoju`.
#[derive(Debug, Clone, PartialEq)]
pub struct DokumentyPokojuRow {
    pub wpis_id: String,
    pub rodzaj: WpisyRodzaj,
    pub pozycja_id: Option<String>,
    pub strona_id: Option<String>,
    pub wystawca: Option<String>,
    pub data_wystawienia: Option<String>,
    pub waznosc_do: Option<String>,
    pub oryginal_w_dniu_aktu: Option<bool>,
    pub kto_przyniesie: Option<String>,
    pub zakres: Option<String>,
    pub numer_wersji: Option<i64>,
    pub ostateczna: Option<bool>,
    pub strona: Strona,
    pub autor: String,
    pub chwila: String,
    pub poprawia_id: Option<String>,
    pub tresc: Option<String>,
    pub sciezka: Option<String>,
    pub skrot: Option<String>,
}

impl DokumentyPokojuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(DokumentyPokojuRow {
            wpis_id: row.get("wpis_id")?,
            rodzaj: row.get("rodzaj")?,
            pozycja_id: row.get("pozycja_id")?,
            strona_id: row.get("strona_id")?,
            wystawca: row.get("wystawca")?,
            data_wystawienia: row.get("data_wystawienia")?,
            waznosc_do: row.get("waznosc_do")?,
            oryginal_w_dniu_aktu: row.get("oryginal_w_dniu_aktu")?,
            kto_przyniesie: row.get("kto_przyniesie")?,
            zakres: row.get("zakres")?,
            numer_wersji: row.get("numer_wersji")?,
            ostateczna: row.get("ostateczna")?,
            strona: row.get("strona")?,
            autor: row.get("autor")?,
            chwila: row.get("chwila")?,
            poprawia_id: row.get("poprawia_id")?,
            tresc: row.get("tresc")?,
            sciezka: row.get("sciezka")?,
            skrot: row.get("skrot")?,
        })
    }
}

/// Dokumenty, pełnomocnictwa, zgody, wersje projektu i koszty z wpisami, które je przyniosły.
pub fn dokumenty_pokoju(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<DokumentyPokojuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT dokument.*, wpis.strona, wpis.autor, wpis.chwila, wpis.poprawia_id, wpis.tresc,
  wpis.sciezka, wpis.skrot
FROM wymiana_dokumenty_wpisy AS dokument
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = dokument.wpis_id
ORDER BY wpis.rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(DokumentyPokojuRow::from_row).collect()
}

/// A row of `wersje_projektu_aktu`.
#[derive(Debug, Clone, PartialEq)]
pub struct WersjeProjektuAktuRow {
    pub wpis_id: String,
    pub numer_wersji: Option<i64>,
    pub ostateczna: Option<bool>,
    pub chwila: String,
    pub tresc: Option<String>,
    pub sciezka: Option<String>,
    pub skrot: Option<String>,
}

impl WersjeProjektuAktuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WersjeProjektuAktuRow {
            wpis_id: row.get("wpis_id")?,
            numer_wersji: row.get("numer_wersji")?,
            ostateczna: row.get("ostateczna")?,
            chwila: row.get("chwila")?,
            tresc: row.get("tresc")?,
            sciezka: row.get("sciezka")?,
            skrot: row.get("skrot")?,
        })
    }
}

/// Wersje projektu aktu od najnowszej, z plikiem i tym, czy kancelaria wskazała ją jako ostateczną.
pub fn wersje_projektu_aktu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WersjeProjektuAktuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT dokument.wpis_id, dokument.numer_wersji, dokument.ostateczna, wpis.chwila, wpis.tresc,
  wpis.sciezka, wpis.skrot
FROM wymiana_dokumenty_wpisy AS dokument
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = dokument.wpis_id
WHERE dokument.rodzaj = 'projekt_aktu'
ORDER BY dokument.numer_wersji DESC"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(WersjeProjektuAktuRow::from_row).collect()
}

/// The parameters of `przekaz_dokument_z_listy`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzekazDokumentZListyParams {
    pub wpis_id: String,
    pub strona: Strona,
    pub autor: String,
    pub chwila: String,
    pub odpowiada_na_id: Option<String>,
    pub poprawia_id: Option<String>,
    pub tresc: Option<String>,
    pub sciezka: String,
    pub skrot: String,
    pub pozycja_id: Option<String>,
    pub strona_id: Option<String>,
    pub wystawca: Option<String>,
    pub data_wystawienia: Option<String>,
    pub waznosc_do: Option<String>,
    pub oryginal_w_dniu_aktu: Option<bool>,
    pub kto_przyniesie: Option<String>,
}

/// Biuro przekazuje w imieniu strony dokument z listy kancelarii: wystawca, data, ważność, plik
/// i to, czy oryginał będzie w dniu aktu.
pub fn przekaz_dokument_z_listy(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzekazDokumentZListyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id, poprawia_id,
  tresc, sciezka, skrot)
VALUES (:wpis_id, :strona, :autor, 'dokument', :chwila, :odpowiada_na_id, :poprawia_id, :tresc,
  :sciezka, :skrot)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
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
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_dokumenty_wpisy (wpis_id, rodzaj, pozycja_id, strona_id, wystawca,
  data_wystawienia, waznosc_do, oryginal_w_dniu_aktu, kto_przyniesie)
VALUES (:wpis_id, 'dokument', :pozycja_id, :strona_id, :wystawca, :data_wystawienia, :waznosc_do,
  :oryginal_w_dniu_aktu, :kto_przyniesie)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    (
                        "pozycja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.pozycja_id),
                    ),
                    (
                        "strona_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.strona_id),
                    ),
                    ("wystawca", xmeta_room::ToSqlValue::to_sql_value(&params.wystawca)),
                    (
                        "data_wystawienia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.data_wystawienia),
                    ),
                    (
                        "waznosc_do",
                        xmeta_room::ToSqlValue::to_sql_value(&params.waznosc_do),
                    ),
                    (
                        "oryginal_w_dniu_aktu",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.oryginal_w_dniu_aktu,
                        ),
                    ),
                    (
                        "kto_przyniesie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kto_przyniesie),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `przekaz_pelnomocnictwo`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzekazPelnomocnictwoParams {
    pub wpis_id: String,
    pub strona: Strona,
    pub autor: String,
    pub chwila: String,
    pub poprawia_id: Option<String>,
    pub tresc: Option<String>,
    pub sciezka: String,
    pub skrot: String,
    pub strona_id: Option<String>,
    pub wystawca: Option<String>,
    pub data_wystawienia: Option<String>,
    pub waznosc_do: Option<String>,
    pub zakres: Option<String>,
}

/// Biuro przekazuje pełnomocnictwo strony z zakresem; dopiero po nim pisze w jej imieniu.
pub fn przekaz_pelnomocnictwo(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzekazPelnomocnictwoParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, poprawia_id, tresc, sciezka,
  skrot)
VALUES (:wpis_id, :strona, :autor, 'pelnomocnictwo', :chwila, :poprawia_id, :tresc, :sciezka,
  :skrot)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    ("strona", xmeta_room::ToSqlValue::to_sql_value(&params.strona)),
                    ("autor", xmeta_room::ToSqlValue::to_sql_value(&params.autor)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    (
                        "poprawia_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprawia_id),
                    ),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                    ("sciezka", xmeta_room::ToSqlValue::to_sql_value(&params.sciezka)),
                    ("skrot", xmeta_room::ToSqlValue::to_sql_value(&params.skrot)),
                ]),
            },
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_dokumenty_wpisy (wpis_id, rodzaj, strona_id, wystawca, data_wystawienia,
  waznosc_do, zakres)
VALUES (:wpis_id, 'pelnomocnictwo', :strona_id, :wystawca, :data_wystawienia, :waznosc_do, :zakres)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    (
                        "strona_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.strona_id),
                    ),
                    ("wystawca", xmeta_room::ToSqlValue::to_sql_value(&params.wystawca)),
                    (
                        "data_wystawienia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.data_wystawienia),
                    ),
                    (
                        "waznosc_do",
                        xmeta_room::ToSqlValue::to_sql_value(&params.waznosc_do),
                    ),
                    ("zakres", xmeta_room::ToSqlValue::to_sql_value(&params.zakres)),
                ]),
            },
        ],
    )
}

/// The parameters of `przekaz_zgode_stron`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzekazZgodeStronParams {
    pub wpis_id: String,
    pub strona: Strona,
    pub autor: String,
    pub chwila: String,
    pub poprawia_id: Option<String>,
    pub tresc: Option<String>,
    pub sciezka: String,
    pub skrot: String,
    pub strona_id: Option<String>,
    pub data_wystawienia: Option<String>,
    pub zakres: Option<String>,
}

/// Biuro przekazuje zgodę strony, np. na to, żeby biuro dostawało projekt aktu i informację po akcie.
pub fn przekaz_zgode_stron(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzekazZgodeStronParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, poprawia_id, tresc, sciezka,
  skrot)
VALUES (:wpis_id, :strona, :autor, 'zgoda_stron', :chwila, :poprawia_id, :tresc, :sciezka, :skrot)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    ("strona", xmeta_room::ToSqlValue::to_sql_value(&params.strona)),
                    ("autor", xmeta_room::ToSqlValue::to_sql_value(&params.autor)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    (
                        "poprawia_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprawia_id),
                    ),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                    ("sciezka", xmeta_room::ToSqlValue::to_sql_value(&params.sciezka)),
                    ("skrot", xmeta_room::ToSqlValue::to_sql_value(&params.skrot)),
                ]),
            },
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_dokumenty_wpisy (wpis_id, rodzaj, strona_id, data_wystawienia, zakres)
VALUES (:wpis_id, 'zgoda_stron', :strona_id, :data_wystawienia, :zakres)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    (
                        "strona_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.strona_id),
                    ),
                    (
                        "data_wystawienia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.data_wystawienia),
                    ),
                    ("zakres", xmeta_room::ToSqlValue::to_sql_value(&params.zakres)),
                ]),
            },
        ],
    )
}

/// The parameters of `przekaz_projekt_aktu`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzekazProjektAktuParams {
    pub wpis_id: String,
    pub strona: Strona,
    pub autor: String,
    pub chwila: String,
    pub odpowiada_na_id: Option<String>,
    pub tresc: Option<String>,
    pub sciezka: String,
    pub skrot: String,
    pub numer_wersji: Option<i64>,
    pub ostateczna: Option<bool>,
}

/// Kancelaria zostawia kolejną wersję projektu aktu, za zgodą stron na jego przekazanie biuru.
pub fn przekaz_projekt_aktu(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzekazProjektAktuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id, tresc,
  sciezka, skrot)
VALUES (:wpis_id, :strona, :autor, 'projekt_aktu', :chwila, :odpowiada_na_id, :tresc, :sciezka,
  :skrot)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    ("strona", xmeta_room::ToSqlValue::to_sql_value(&params.strona)),
                    ("autor", xmeta_room::ToSqlValue::to_sql_value(&params.autor)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    (
                        "odpowiada_na_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odpowiada_na_id),
                    ),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                    ("sciezka", xmeta_room::ToSqlValue::to_sql_value(&params.sciezka)),
                    ("skrot", xmeta_room::ToSqlValue::to_sql_value(&params.skrot)),
                ]),
            },
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_dokumenty_wpisy (wpis_id, rodzaj, numer_wersji, ostateczna)
VALUES (:wpis_id, 'projekt_aktu', :numer_wersji, :ostateczna)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    (
                        "numer_wersji",
                        xmeta_room::ToSqlValue::to_sql_value(&params.numer_wersji),
                    ),
                    (
                        "ostateczna",
                        xmeta_room::ToSqlValue::to_sql_value(&params.ostateczna),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `podaj_koszty_aktu`.
#[derive(Debug, Clone, PartialEq)]
pub struct PodajKosztyAktuParams {
    pub wpis_id: String,
    pub strona: Strona,
    pub autor: String,
    pub chwila: String,
    pub poprawia_id: Option<String>,
    pub tresc: Option<String>,
    pub sciezka: Option<String>,
    pub skrot: Option<String>,
}

/// Kancelaria podaje taksę, opłaty i podatki, które pobierze; biuro przekazuje je stronom.
pub fn podaj_koszty_aktu(
    room: &impl xmeta_room::MetaRoom,
    params: &PodajKosztyAktuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, poprawia_id, tresc, sciezka,
  skrot)
VALUES (:wpis_id, :strona, :autor, 'koszty', :chwila, :poprawia_id, :tresc, :sciezka, :skrot)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    ("strona", xmeta_room::ToSqlValue::to_sql_value(&params.strona)),
                    ("autor", xmeta_room::ToSqlValue::to_sql_value(&params.autor)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    (
                        "poprawia_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprawia_id),
                    ),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                    ("sciezka", xmeta_room::ToSqlValue::to_sql_value(&params.sciezka)),
                    ("skrot", xmeta_room::ToSqlValue::to_sql_value(&params.skrot)),
                ]),
            },
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_dokumenty_wpisy (wpis_id, rodzaj)
VALUES (:wpis_id, 'koszty')"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                ]),
            },
        ],
    )
}

/// A row of `listy_dokumentow`.
#[derive(Debug, Clone, PartialEq)]
pub struct ListyDokumentowRow {
    pub wpis_id: String,
    pub pozycja_id: String,
    pub nazwa: String,
    pub od_strony_id: Option<String>,
    pub postac: ListaPostac,
    pub swiezosc_dni: Option<i64>,
    pub do_kiedy: Option<String>,
    pub chwila: String,
}

impl ListyDokumentowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ListyDokumentowRow {
            wpis_id: row.get("wpis_id")?,
            pozycja_id: row.get("pozycja_id")?,
            nazwa: row.get("nazwa")?,
            od_strony_id: row.get("od_strony_id")?,
            postac: row.get("postac")?,
            swiezosc_dni: row.get("swiezosc_dni")?,
            do_kiedy: row.get("do_kiedy")?,
            chwila: row.get("chwila")?,
        })
    }
}

/// Pozycje wszystkich wersji listy dokumentów kancelarii, z chwilą wpisu.
pub fn listy_dokumentow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ListyDokumentowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT lista.*, wpis.chwila
FROM wymiana_dokumenty_lista AS lista
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = lista.wpis_id
ORDER BY wpis.rowid, lista.rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(ListyDokumentowRow::from_row).collect()
}

/// A row of `biezaca_lista_dokumentow`.
#[derive(Debug, Clone, PartialEq)]
pub struct BiezacaListaDokumentowRow {
    pub wpis_id: String,
    pub pozycja_id: String,
    pub nazwa: String,
    pub od_strony_id: Option<String>,
    pub postac: ListaPostac,
    pub swiezosc_dni: Option<i64>,
    pub do_kiedy: Option<String>,
}

impl BiezacaListaDokumentowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(BiezacaListaDokumentowRow {
            wpis_id: row.get("wpis_id")?,
            pozycja_id: row.get("pozycja_id")?,
            nazwa: row.get("nazwa")?,
            od_strony_id: row.get("od_strony_id")?,
            postac: row.get("postac")?,
            swiezosc_dni: row.get("swiezosc_dni")?,
            do_kiedy: row.get("do_kiedy")?,
        })
    }
}

/// Pozycje najnowszej listy dokumentów; zmiana listy jest nowym wpisem z całą listą.
pub fn biezaca_lista_dokumentow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<BiezacaListaDokumentowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM wymiana_dokumenty_lista
WHERE wpis_id = (
  SELECT id
  FROM wymiana_wpisy_wpisy
  WHERE rodzaj = 'lista_dokumentow'
  ORDER BY rowid DESC
  LIMIT 1
)
ORDER BY rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(BiezacaListaDokumentowRow::from_row).collect()
}

/// The parameters of `podaj_liste_dokumentow`.
#[derive(Debug, Clone, PartialEq)]
pub struct PodajListeDokumentowParams {
    pub id: String,
    pub strona: Strona,
    pub autor: String,
    pub chwila: String,
    pub poprawia_id: Option<String>,
    pub tresc: Option<String>,
}

/// Kancelaria zaczyna listę dokumentów do aktu; pozycje dopisuje dopisz_pozycje_listy w tej samej
/// transakcji (useMetaTransaction). Nowa lista poprawia poprzednią.
pub fn podaj_liste_dokumentow(
    room: &impl xmeta_room::MetaRoom,
    params: &PodajListeDokumentowParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, poprawia_id, tresc)
VALUES (:id, :strona, :autor, 'lista_dokumentow', :chwila, :poprawia_id, :tresc)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("strona", xmeta_room::ToSqlValue::to_sql_value(&params.strona)),
                    ("autor", xmeta_room::ToSqlValue::to_sql_value(&params.autor)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    (
                        "poprawia_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprawia_id),
                    ),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                ]),
            },
        ],
    )
}

/// The parameters of `dopisz_pozycje_listy`.
#[derive(Debug, Clone, PartialEq)]
pub struct DopiszPozycjeListyParams {
    pub wpis_id: String,
    pub pozycja_id: String,
    pub nazwa: String,
    pub od_strony_id: Option<String>,
    pub postac: ListaPostac,
    pub swiezosc_dni: Option<i64>,
    pub do_kiedy: Option<String>,
}

/// Pozycja listy: dokument, od której strony, w jakiej postaci, jak świeży i do kiedy.
pub fn dopisz_pozycje_listy(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszPozycjeListyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_dokumenty_lista (wpis_id, pozycja_id, nazwa, od_strony_id, postac,
  swiezosc_dni, do_kiedy)
VALUES (:wpis_id, :pozycja_id, :nazwa, :od_strony_id, :postac, :swiezosc_dni, :do_kiedy)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    (
                        "pozycja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.pozycja_id),
                    ),
                    ("nazwa", xmeta_room::ToSqlValue::to_sql_value(&params.nazwa)),
                    (
                        "od_strony_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.od_strony_id),
                    ),
                    ("postac", xmeta_room::ToSqlValue::to_sql_value(&params.postac)),
                    (
                        "swiezosc_dni",
                        xmeta_room::ToSqlValue::to_sql_value(&params.swiezosc_dni),
                    ),
                    ("do_kiedy", xmeta_room::ToSqlValue::to_sql_value(&params.do_kiedy)),
                ]),
            },
        ],
    )
}

/// A row of `uwagi_do_projektu`.
#[derive(Debug, Clone, PartialEq)]
pub struct UwagiDoProjektuRow {
    pub wpis_id: String,
    pub projekt_id: String,
    pub decyzja_odnosnik: String,
    pub numer_wersji: Option<i64>,
    pub chwila: String,
    pub tresc: Option<String>,
    pub sciezka: Option<String>,
    pub skrot: Option<String>,
}

impl UwagiDoProjektuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(UwagiDoProjektuRow {
            wpis_id: row.get("wpis_id")?,
            projekt_id: row.get("projekt_id")?,
            decyzja_odnosnik: row.get("decyzja_odnosnik")?,
            numer_wersji: row.get("numer_wersji")?,
            chwila: row.get("chwila")?,
            tresc: row.get("tresc")?,
            sciezka: row.get("sciezka")?,
            skrot: row.get("skrot")?,
        })
    }
}

/// Uwagi stron do wersji projektu, z decyzją, która pozwoliła je przekazać.
pub fn uwagi_do_projektu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<UwagiDoProjektuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT uwagi.*, projekt.numer_wersji, wpis.chwila, wpis.tresc, wpis.sciezka, wpis.skrot
FROM wymiana_dokumenty_uwagi AS uwagi
JOIN wymiana_dokumenty_wpisy AS projekt ON projekt.wpis_id = uwagi.projekt_id
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = uwagi.wpis_id
ORDER BY wpis.rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(UwagiDoProjektuRow::from_row).collect()
}

/// The parameters of `przekaz_uwagi_do_projektu`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzekazUwagiDoProjektuParams {
    pub wpis_id: String,
    pub strona: Strona,
    pub autor: String,
    pub chwila: String,
    pub projekt_id: String,
    pub tresc: Option<String>,
    pub sciezka: Option<String>,
    pub skrot: Option<String>,
    pub decyzja_odnosnik: String,
}

/// Biuro przekazuje uwagi stron do wskazanej wersji projektu za zgodą człowieka z teczki.
pub fn przekaz_uwagi_do_projektu(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzekazUwagiDoProjektuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id, tresc,
  sciezka, skrot)
VALUES (:wpis_id, :strona, :autor, 'uwagi_do_projektu', :chwila, :projekt_id, :tresc, :sciezka,
  :skrot)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    ("strona", xmeta_room::ToSqlValue::to_sql_value(&params.strona)),
                    ("autor", xmeta_room::ToSqlValue::to_sql_value(&params.autor)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    (
                        "projekt_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.projekt_id),
                    ),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                    ("sciezka", xmeta_room::ToSqlValue::to_sql_value(&params.sciezka)),
                    ("skrot", xmeta_room::ToSqlValue::to_sql_value(&params.skrot)),
                ]),
            },
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_dokumenty_uwagi (wpis_id, projekt_id, decyzja_odnosnik)
VALUES (:wpis_id, :projekt_id, :decyzja_odnosnik)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    (
                        "projekt_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.projekt_id),
                    ),
                    (
                        "decyzja_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_odnosnik),
                    ),
                ]),
            },
        ],
    )
}
