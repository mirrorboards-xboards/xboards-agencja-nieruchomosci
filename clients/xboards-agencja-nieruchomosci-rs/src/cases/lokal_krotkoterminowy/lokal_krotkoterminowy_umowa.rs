//! The kind `lokal-krotkoterminowy-umowa` of the room `cases/lokal_krotkoterminowy`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "lokal-krotkoterminowy-umowa",
    requires: &[&super::sprawa_umowy::KIND],
    migrations: &[
        r#"-- Warunki umowy o zarządzanie lokalem, z których korzystają agenci i program: największa liczba
-- gości, prowizja, sprzątanie, kwota napraw, koszty właściciela, terminy właściciela, wypłaty,
-- rezerwa i rezerwacje po końcu umowy, a do tego kanały sprzedaży i widełki cen. Datę zawarcia,
-- koniec, okres wypowiedzenia i polisę trzyma rdzeń (sprawa-umowy). Pisze Zarządca obiektu.
--
-- Warunki są w wersjach. Pierwsza obowiązuje od zawarcia umowy, każda następna od aneksu tej umowy
-- (rdzeń: zdarzenie aneks). Wersja powstaje jako projekt, dostaje kanały i widełki, a potem się ją
-- ustala; ustalonej wersji ani jej kanałów i widełek się nie zmienia. Bieżące warunki to ustalona
-- wersja z najpóźniejszą chwilą od (widok lokal_krotkoterminowy_umowa_warunki_biezace).
CREATE TABLE lokal_krotkoterminowy_umowa_warunki (
  id TEXT PRIMARY KEY,
  umowa_id TEXT NOT NULL REFERENCES sprawa_umowy_wpisy (id),
  max_gosci INTEGER NOT NULL CHECK (max_gosci > 0),
  -- Prowizja biura: procent w setnych częściach (15% to 1500) albo kwota w groszach, i podstawa,
  -- od której się ją liczy, np. „przychód po prowizji kanału”.
  prowizja_rodzaj TEXT NOT NULL CHECK (prowizja_rodzaj IN ('procent', 'kwota')),
  prowizja INTEGER NOT NULL CHECK (prowizja >= 0),
  prowizja_podstawa TEXT NOT NULL,
  -- Opłata za sprzątanie, którą płaci gość, w groszach, i czy pokrywa koszt sprzątania z rejestru.
  oplata_sprzatanie INTEGER NOT NULL CHECK (oplata_sprzatanie >= 0),
  oplata_sprzatanie_pokrywa_koszt INTEGER NOT NULL CHECK (oplata_sprzatanie_pokrywa_koszt IN (0, 1)),
  -- Do tej kwoty (w groszach) biuro zleca naprawę bez pytania właściciela.
  kwota_napraw INTEGER NOT NULL CHECK (kwota_napraw >= 0),
  -- Które koszty biuro płaci z przychodu, a które właściciel sam.
  koszty_wlasciciela TEXT NOT NULL,
  -- Z jakim wyprzedzeniem w dniach właściciel zgłasza noce dla siebie.
  zgloszenie_nocy_dni INTEGER NOT NULL CHECK (zgloszenie_nocy_dni >= 0),
  -- Dzień miesiąca wypłaty dla właściciela i rezerwa na koszty w groszach.
  wyplata_dzien INTEGER NOT NULL CHECK (wyplata_dzien BETWEEN 1 AND 31),
  rezerwa INTEGER NOT NULL CHECK (rezerwa >= 0),
  -- Co dzieje się z rezerwacjami na noce po końcu umowy.
  po_koncu_rezerwacje TEXT NOT NULL,
  -- Aneks, od którego obowiązuje wersja; pierwsza wersja go nie ma.
  aneks_zdarzenie_id TEXT REFERENCES sprawa_umowy_zdarzenia (id),
  od TEXT NOT NULL,
  stan TEXT NOT NULL CHECK (stan IN ('projekt', 'obowiazuje')),

  CONSTRAINT procent_najwyzej_100 CHECK (prowizja_rodzaj <> 'procent' OR prowizja <= 10000),

  UNIQUE (aneks_zdarzenie_id)
) STRICT;

-- Kanały sprzedaży z umowy i to, na czyje konto prowadzi się w nich ofertę.
CREATE TABLE lokal_krotkoterminowy_umowa_kanaly (
  -- Klucz nadany przez bazę jest dodatni; zob. lokal_krotkoterminowy_umowa_kanaly_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  warunki_id TEXT NOT NULL REFERENCES lokal_krotkoterminowy_umowa_warunki (id),
  -- Np. airbnb, booking, strona_biura.
  kanal TEXT NOT NULL CHECK (kanal <> '' AND kanal NOT GLOB '*[^a-z_]*'),
  konto TEXT NOT NULL CHECK (konto IN ('biura', 'wlasciciela')),

  UNIQUE (warunki_id, kanal)
) STRICT;

-- Widełki cen nocy w okresach, np. sezon i poza sezonem, z najkrótszym pobytem. Ceny w groszach,
-- okresy jako dni włącznie.
CREATE TABLE lokal_krotkoterminowy_umowa_widelki (
  -- Klucz nadany przez bazę jest dodatni; zob. lokal_krotkoterminowy_umowa_widelki_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  warunki_id TEXT NOT NULL REFERENCES lokal_krotkoterminowy_umowa_warunki (id),
  sezon TEXT NOT NULL CHECK (sezon <> ''),
  od_dnia TEXT NOT NULL CHECK (od_dnia = date(od_dnia)),
  do_dnia TEXT NOT NULL CHECK (do_dnia = date(do_dnia)),
  cena_min INTEGER NOT NULL CHECK (cena_min > 0),
  cena_max INTEGER NOT NULL,
  pobyt_min INTEGER NOT NULL CHECK (pobyt_min >= 1),

  CONSTRAINT cena_min_nie_wieksza_niz_max CHECK (cena_min <= cena_max),
  CONSTRAINT okres_widelek_od_do CHECK (do_dnia >= od_dnia)
) STRICT;

-- Umowa o zarządzanie lokalem: teczka ma najwyżej jedną zawartą (lokal-krotkoterminowy-reguly),
-- z chwilą, od której biuro może według niej pracować (rdzeń: sprawa_umowy_praca_od).
CREATE VIEW lokal_krotkoterminowy_umowa_lokalu AS
SELECT
  umowa.id AS umowa_id,
  umowa.stan,
  umowa.koniec,
  praca.od AS praca_od
FROM sprawa_umowy_wpisy AS umowa
JOIN sprawa_umowy_praca_od AS praca ON praca.umowa_id = umowa.id
WHERE umowa.rodzaj = 'zarzadzanie';

-- Bieżące warunki: ustalona wersja z najpóźniejszą chwilą od. To jedyne miejsce tej reguły;
-- czytają je reguły pobytów, zleceń i kalendarza.
CREATE VIEW lokal_krotkoterminowy_umowa_warunki_biezace AS
SELECT *
FROM lokal_krotkoterminowy_umowa_warunki
WHERE stan = 'obowiazuje'
ORDER BY od DESC, rowid DESC
LIMIT 1;

-- Ustalenie sprawdza zmianę stanu, a kanały i widełki dopisuje się do projektu, więc wersja
-- powstaje jako projekt.
CREATE TRIGGER lokal_krotkoterminowy_umowa_warunki_powstaja_jako_projekt
BEFORE INSERT ON lokal_krotkoterminowy_umowa_warunki
WHEN NEW.stan <> 'projekt'
BEGIN
  SELECT RAISE(ABORT, 'warunki umowy powstają jako projekt');
END;

-- Ustalenie z właścicielem, które zmienia umowę, obowiązuje dopiero po podpisaniu aneksu
-- („Aneksy”). Pierwsza wersja obowiązuje od zawarcia, każda następna wskazuje aneks tej umowy,
-- obowiązuje od jego chwili i jest późniejsza niż poprzednie. Porównanie z pustą wartością daje
-- NULL, więc coalesce zamienia je na odmowę.
CREATE TRIGGER lokal_krotkoterminowy_umowa_warunki_od_zawarcia_albo_aneksu
BEFORE INSERT ON lokal_krotkoterminowy_umowa_warunki
WHEN NOT coalesce(
  CASE
    WHEN EXISTS (
      SELECT 1
      FROM lokal_krotkoterminowy_umowa_warunki
      WHERE umowa_id = NEW.umowa_id
    )
    THEN EXISTS (
        SELECT 1
        FROM sprawa_umowy_zdarzenia
        WHERE id = NEW.aneks_zdarzenie_id
          AND umowa_id = NEW.umowa_id
          AND rodzaj = 'aneks'
          AND chwila = NEW.od
      )
      AND NEW.od > (
        SELECT max(od)
        FROM lokal_krotkoterminowy_umowa_warunki
        WHERE umowa_id = NEW.umowa_id
      )
    ELSE NEW.aneks_zdarzenie_id IS NULL
      AND NEW.od = (
        SELECT zawarto
        FROM sprawa_umowy_wpisy
        WHERE id = NEW.umowa_id
      )
  END,
  0
)
BEGIN
  SELECT RAISE(ABORT, 'pierwsze warunki obowiązują od zawarcia umowy, bez aneksu; każde następne wskazują aneks tej umowy, obowiązują od jego chwili i są późniejsze niż poprzednie');
END;

-- Stoi po regule chwili od: SQLite wywołuje najpierw trigger utworzony później, a bez zawartej umowy
-- nie ma chwili zawarcia, więc odmowa podaje tę regułę.
CREATE TRIGGER lokal_krotkoterminowy_umowa_warunki_zawartej_umowy
BEFORE INSERT ON lokal_krotkoterminowy_umowa_warunki
WHEN NOT EXISTS (
  SELECT 1
  FROM sprawa_umowy_wpisy
  WHERE id = NEW.umowa_id
    AND rodzaj = 'zarzadzanie'
    AND stan = 'zawarta'
)
BEGIN
  SELECT RAISE(ABORT, 'warunki wpisuje się do zawartej umowy o zarządzanie');
END;

-- Na warunkach stoją reguły pobytów, zleceń i cen, a ustalenie odpowiada podpisanej umowie albo
-- aneksowi. Zmienia się tylko stan projektu na obowiazuje; inne warunki to nowa wersja.
CREATE TRIGGER lokal_krotkoterminowy_umowa_warunki_zmienia_tylko_ustalenie
BEFORE UPDATE ON lokal_krotkoterminowy_umowa_warunki
WHEN OLD.stan <> 'projekt'
  OR NEW.stan <> 'obowiazuje'
  OR NEW.id IS NOT OLD.id
  OR NEW.umowa_id IS NOT OLD.umowa_id
  OR NEW.max_gosci IS NOT OLD.max_gosci
  OR NEW.prowizja_rodzaj IS NOT OLD.prowizja_rodzaj
  OR NEW.prowizja IS NOT OLD.prowizja
  OR NEW.prowizja_podstawa IS NOT OLD.prowizja_podstawa
  OR NEW.oplata_sprzatanie IS NOT OLD.oplata_sprzatanie
  OR NEW.oplata_sprzatanie_pokrywa_koszt IS NOT OLD.oplata_sprzatanie_pokrywa_koszt
  OR NEW.kwota_napraw IS NOT OLD.kwota_napraw
  OR NEW.koszty_wlasciciela IS NOT OLD.koszty_wlasciciela
  OR NEW.zgloszenie_nocy_dni IS NOT OLD.zgloszenie_nocy_dni
  OR NEW.wyplata_dzien IS NOT OLD.wyplata_dzien
  OR NEW.rezerwa IS NOT OLD.rezerwa
  OR NEW.po_koncu_rezerwacje IS NOT OLD.po_koncu_rezerwacje
  OR NEW.aneks_zdarzenie_id IS NOT OLD.aneks_zdarzenie_id
  OR NEW.od IS NOT OLD.od
BEGIN
  SELECT RAISE(ABORT, 'warunków umowy się nie zmienia; projekt się tylko ustala, a inne warunki to nowa wersja');
END;

-- Lokal sprzedaje się tylko w kanałach z umowy albo za zgodą właściciela i człowieka, więc
-- ustalona wersja ma co najmniej jeden kanał.
CREATE TRIGGER lokal_krotkoterminowy_umowa_warunki_ustalone_z_kanalem
BEFORE UPDATE OF stan ON lokal_krotkoterminowy_umowa_warunki
WHEN NEW.stan = 'obowiazuje'
  AND NOT EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_umowa_kanaly
    WHERE warunki_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'ustalone warunki mają co najmniej jeden kanał sprzedaży');
END;

CREATE TRIGGER lokal_krotkoterminowy_umowa_kanaly_tylko_w_projekcie
BEFORE INSERT ON lokal_krotkoterminowy_umowa_kanaly
WHEN (
  SELECT stan <> 'projekt'
  FROM lokal_krotkoterminowy_umowa_warunki
  WHERE id = NEW.warunki_id
)
BEGIN
  SELECT RAISE(ABORT, 'kanały dopisuje się tylko do projektu warunków');
END;

CREATE TRIGGER lokal_krotkoterminowy_umowa_kanaly_bez_zmian
BEFORE UPDATE ON lokal_krotkoterminowy_umowa_kanaly
BEGIN
  SELECT RAISE(ABORT, 'kanału warunków się nie zmienia; inne kanały to nowa wersja warunków');
END;

CREATE TRIGGER lokal_krotkoterminowy_umowa_widelki_tylko_w_projekcie
BEFORE INSERT ON lokal_krotkoterminowy_umowa_widelki
WHEN (
  SELECT stan <> 'projekt'
  FROM lokal_krotkoterminowy_umowa_warunki
  WHERE id = NEW.warunki_id
)
BEGIN
  SELECT RAISE(ABORT, 'widełki dopisuje się tylko do projektu warunków');
END;

-- Cena nocy porównuje się z widełkami okresu, w którym leży noc, więc okresy jednej wersji się
-- nie nakładają.
CREATE TRIGGER lokal_krotkoterminowy_umowa_widelki_bez_nakladania
BEFORE INSERT ON lokal_krotkoterminowy_umowa_widelki
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_umowa_widelki
  WHERE warunki_id = NEW.warunki_id
    AND od_dnia <= NEW.do_dnia
    AND do_dnia >= NEW.od_dnia
)
BEGIN
  SELECT RAISE(ABORT, 'okresy widełek jednej wersji warunków się nie nakładają');
END;

CREATE TRIGGER lokal_krotkoterminowy_umowa_widelki_bez_zmian
BEFORE UPDATE ON lokal_krotkoterminowy_umowa_widelki
BEGIN
  SELECT RAISE(ABORT, 'widełek się nie zmienia; inne widełki to nowa wersja warunków');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER lokal_krotkoterminowy_umowa_warunki_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_umowa_warunki
WHEN EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_umowa_warunki
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_umowa_warunki
    WHERE aneks_zdarzenie_id = NEW.aneks_zdarzenie_id
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_umowa_warunki_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_umowa_warunki
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER lokal_krotkoterminowy_umowa_kanaly_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_umowa_kanaly
WHEN EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_umowa_kanaly
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_umowa_kanaly
    WHERE warunki_id = NEW.warunki_id
      AND kanal = NEW.kanal
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_umowa_kanaly_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_umowa_kanaly
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER lokal_krotkoterminowy_umowa_widelki_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_umowa_widelki
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_umowa_widelki
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_umowa_widelki_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_umowa_widelki
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
    ],
};

/// The values of column `kanaly.konto`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KanalyKonto {
    Biura,
    Wlasciciela,
}

impl KanalyKonto {
    /// Every value, in the order of the catalog.
    pub const ALL: &[KanalyKonto] = &[KanalyKonto::Biura, KanalyKonto::Wlasciciela];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            KanalyKonto::Biura => "biura",
            KanalyKonto::Wlasciciela => "wlasciciela",
        }
    }
}

impl xmeta_room::FromSqlValue for KanalyKonto {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "biura" => Ok(KanalyKonto::Biura),
                    "wlasciciela" => Ok(KanalyKonto::Wlasciciela),
                    other => Err(format!("{other} is no value of KanalyKonto")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for KanalyKonto {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `warunki.prowizja_rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WarunkiProwizjaRodzaj {
    Procent,
    Kwota,
}

impl WarunkiProwizjaRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WarunkiProwizjaRodzaj] = &[
        WarunkiProwizjaRodzaj::Procent,
        WarunkiProwizjaRodzaj::Kwota,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WarunkiProwizjaRodzaj::Procent => "procent",
            WarunkiProwizjaRodzaj::Kwota => "kwota",
        }
    }
}

impl xmeta_room::FromSqlValue for WarunkiProwizjaRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "procent" => Ok(WarunkiProwizjaRodzaj::Procent),
                    "kwota" => Ok(WarunkiProwizjaRodzaj::Kwota),
                    other => Err(format!("{other} is no value of WarunkiProwizjaRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WarunkiProwizjaRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `warunki.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WarunkiStan {
    Projekt,
    Obowiazuje,
}

impl WarunkiStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WarunkiStan] = &[WarunkiStan::Projekt, WarunkiStan::Obowiazuje];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WarunkiStan::Projekt => "projekt",
            WarunkiStan::Obowiazuje => "obowiazuje",
        }
    }
}

impl xmeta_room::FromSqlValue for WarunkiStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "projekt" => Ok(WarunkiStan::Projekt),
                    "obowiazuje" => Ok(WarunkiStan::Obowiazuje),
                    other => Err(format!("{other} is no value of WarunkiStan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WarunkiStan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of field `stan` of `umowa_lokalu`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stan {
    Projekt,
    DoPodpisu,
    Zawarta,
    Zakonczona,
}

impl Stan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[Stan] = &[
        Stan::Projekt,
        Stan::DoPodpisu,
        Stan::Zawarta,
        Stan::Zakonczona,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            Stan::Projekt => "projekt",
            Stan::DoPodpisu => "do_podpisu",
            Stan::Zawarta => "zawarta",
            Stan::Zakonczona => "zakonczona",
        }
    }
}

impl xmeta_room::FromSqlValue for Stan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "projekt" => Ok(Stan::Projekt),
                    "do_podpisu" => Ok(Stan::DoPodpisu),
                    "zawarta" => Ok(Stan::Zawarta),
                    "zakonczona" => Ok(Stan::Zakonczona),
                    other => Err(format!("{other} is no value of Stan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for Stan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `kanaly_warunkow`.
#[derive(Debug, Clone, PartialEq)]
pub struct KanalyWarunkowRow {
    pub id: i64,
    pub warunki_id: String,
    pub kanal: String,
    pub konto: KanalyKonto,
}

impl KanalyWarunkowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(KanalyWarunkowRow {
            id: row.get("id")?,
            warunki_id: row.get("warunki_id")?,
            kanal: row.get("kanal")?,
            konto: row.get("konto")?,
        })
    }
}

/// Kanały sprzedaży każdej wersji warunków.
pub fn kanaly_warunkow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<KanalyWarunkowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_umowa_kanaly
ORDER BY warunki_id, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(KanalyWarunkowRow::from_row).collect()
}

/// The parameters of `dopisz_kanal_warunkow`.
#[derive(Debug, Clone, PartialEq)]
pub struct DopiszKanalWarunkowParams {
    pub warunki_id: String,
    pub kanal: String,
    pub konto: KanalyKonto,
}

/// Kanał sprzedaży z kontem biura albo właściciela w projekcie warunków.
pub fn dopisz_kanal_warunkow(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszKanalWarunkowParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_umowa_kanaly (warunki_id, kanal, konto)
VALUES (:warunki_id, :kanal, :konto)"#,
                parameters: Vec::from([
                    (
                        "warunki_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.warunki_id),
                    ),
                    ("kanal", xmeta_room::ToSqlValue::to_sql_value(&params.kanal)),
                    ("konto", xmeta_room::ToSqlValue::to_sql_value(&params.konto)),
                ]),
            },
        ],
    )
}

/// A row of `warunki_umowy_lokalu`.
#[derive(Debug, Clone, PartialEq)]
pub struct WarunkiUmowyLokaluRow {
    pub id: String,
    pub umowa_id: String,
    pub max_gosci: i64,
    pub prowizja_rodzaj: WarunkiProwizjaRodzaj,
    pub prowizja: i64,
    pub prowizja_podstawa: String,
    pub oplata_sprzatanie: i64,
    pub oplata_sprzatanie_pokrywa_koszt: bool,
    pub kwota_napraw: i64,
    pub koszty_wlasciciela: String,
    pub zgloszenie_nocy_dni: i64,
    pub wyplata_dzien: i64,
    pub rezerwa: i64,
    pub po_koncu_rezerwacje: String,
    pub aneks_zdarzenie_id: Option<String>,
    pub od: String,
    pub stan: WarunkiStan,
}

impl WarunkiUmowyLokaluRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WarunkiUmowyLokaluRow {
            id: row.get("id")?,
            umowa_id: row.get("umowa_id")?,
            max_gosci: row.get("max_gosci")?,
            prowizja_rodzaj: row.get("prowizja_rodzaj")?,
            prowizja: row.get("prowizja")?,
            prowizja_podstawa: row.get("prowizja_podstawa")?,
            oplata_sprzatanie: row.get("oplata_sprzatanie")?,
            oplata_sprzatanie_pokrywa_koszt: row.get("oplata_sprzatanie_pokrywa_koszt")?,
            kwota_napraw: row.get("kwota_napraw")?,
            koszty_wlasciciela: row.get("koszty_wlasciciela")?,
            zgloszenie_nocy_dni: row.get("zgloszenie_nocy_dni")?,
            wyplata_dzien: row.get("wyplata_dzien")?,
            rezerwa: row.get("rezerwa")?,
            po_koncu_rezerwacje: row.get("po_koncu_rezerwacje")?,
            aneks_zdarzenie_id: row.get("aneks_zdarzenie_id")?,
            od: row.get("od")?,
            stan: row.get("stan")?,
        })
    }
}

/// Wersje warunków umowy o zarządzanie w kolejności chwili, od której obowiązują.
pub fn warunki_umowy_lokalu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WarunkiUmowyLokaluRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_umowa_warunki
ORDER BY od, rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(WarunkiUmowyLokaluRow::from_row).collect()
}

/// A row of `biezace_warunki_umowy_lokalu`.
#[derive(Debug, Clone, PartialEq)]
pub struct BiezaceWarunkiUmowyLokaluRow {
    pub id: String,
    pub umowa_id: String,
    pub max_gosci: i64,
    pub prowizja_rodzaj: WarunkiProwizjaRodzaj,
    pub prowizja: i64,
    pub prowizja_podstawa: String,
    pub oplata_sprzatanie: i64,
    pub oplata_sprzatanie_pokrywa_koszt: bool,
    pub kwota_napraw: i64,
    pub koszty_wlasciciela: String,
    pub zgloszenie_nocy_dni: i64,
    pub wyplata_dzien: i64,
    pub rezerwa: i64,
    pub po_koncu_rezerwacje: String,
    pub aneks_zdarzenie_id: Option<String>,
    pub od: String,
    pub stan: WarunkiStan,
}

impl BiezaceWarunkiUmowyLokaluRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(BiezaceWarunkiUmowyLokaluRow {
            id: row.get("id")?,
            umowa_id: row.get("umowa_id")?,
            max_gosci: row.get("max_gosci")?,
            prowizja_rodzaj: row.get("prowizja_rodzaj")?,
            prowizja: row.get("prowizja")?,
            prowizja_podstawa: row.get("prowizja_podstawa")?,
            oplata_sprzatanie: row.get("oplata_sprzatanie")?,
            oplata_sprzatanie_pokrywa_koszt: row.get("oplata_sprzatanie_pokrywa_koszt")?,
            kwota_napraw: row.get("kwota_napraw")?,
            koszty_wlasciciela: row.get("koszty_wlasciciela")?,
            zgloszenie_nocy_dni: row.get("zgloszenie_nocy_dni")?,
            wyplata_dzien: row.get("wyplata_dzien")?,
            rezerwa: row.get("rezerwa")?,
            po_koncu_rezerwacje: row.get("po_koncu_rezerwacje")?,
            aneks_zdarzenie_id: row.get("aneks_zdarzenie_id")?,
            od: row.get("od")?,
            stan: row.get("stan")?,
        })
    }
}

/// Ustalona wersja warunków z najpóźniejszą chwilą od; z nią porównują się pobyty, zlecenia i ceny.
pub fn biezace_warunki_umowy_lokalu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Option<BiezaceWarunkiUmowyLokaluRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_umowa_warunki_biezace"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.first().map(BiezaceWarunkiUmowyLokaluRow::from_row).transpose()
}

/// A row of `umowa_lokalu`.
#[derive(Debug, Clone, PartialEq)]
pub struct UmowaLokaluRow {
    pub umowa_id: String,
    pub stan: Stan,
    pub koniec: Option<String>,
    pub praca_od: String,
}

impl UmowaLokaluRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(UmowaLokaluRow {
            umowa_id: row.get("umowa_id")?,
            stan: row.get("stan")?,
            koniec: row.get("koniec")?,
            praca_od: row.get("praca_od")?,
        })
    }
}

/// Zawarta albo zakończona umowa o zarządzanie lokalem z datą końca i chwilą praca_od.
pub fn umowa_lokalu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Option<UmowaLokaluRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_umowa_lokalu"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.first().map(UmowaLokaluRow::from_row).transpose()
}

/// The parameters of `wpisz_warunki_umowy_lokalu`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszWarunkiUmowyLokaluParams {
    pub id: String,
    pub umowa_id: String,
    pub max_gosci: i64,
    pub prowizja_rodzaj: WarunkiProwizjaRodzaj,
    pub prowizja: i64,
    pub prowizja_podstawa: String,
    pub oplata_sprzatanie: i64,
    pub oplata_sprzatanie_pokrywa_koszt: bool,
    pub kwota_napraw: i64,
    pub koszty_wlasciciela: String,
    pub zgloszenie_nocy_dni: i64,
    pub wyplata_dzien: i64,
    pub rezerwa: i64,
    pub po_koncu_rezerwacje: String,
    pub aneks_zdarzenie_id: Option<String>,
    pub od: String,
}

/// Projekt wersji warunków: pierwsza od zawarcia umowy, następna od aneksu. Identyfikator (UUID)
/// nadaje aplikacja, bo wskazują go kanały i widełki.
pub fn wpisz_warunki_umowy_lokalu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszWarunkiUmowyLokaluParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_umowa_warunki (id, umowa_id, max_gosci, prowizja_rodzaj, prowizja,
  prowizja_podstawa, oplata_sprzatanie, oplata_sprzatanie_pokrywa_koszt, kwota_napraw,
  koszty_wlasciciela, zgloszenie_nocy_dni, wyplata_dzien, rezerwa, po_koncu_rezerwacje,
  aneks_zdarzenie_id, od, stan)
VALUES (:id, :umowa_id, :max_gosci, :prowizja_rodzaj, :prowizja, :prowizja_podstawa,
  :oplata_sprzatanie, :oplata_sprzatanie_pokrywa_koszt, :kwota_napraw, :koszty_wlasciciela,
  :zgloszenie_nocy_dni, :wyplata_dzien, :rezerwa, :po_koncu_rezerwacje, :aneks_zdarzenie_id, :od,
  'projekt')"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("umowa_id", xmeta_room::ToSqlValue::to_sql_value(&params.umowa_id)),
                    (
                        "max_gosci",
                        xmeta_room::ToSqlValue::to_sql_value(&params.max_gosci),
                    ),
                    (
                        "prowizja_rodzaj",
                        xmeta_room::ToSqlValue::to_sql_value(&params.prowizja_rodzaj),
                    ),
                    ("prowizja", xmeta_room::ToSqlValue::to_sql_value(&params.prowizja)),
                    (
                        "prowizja_podstawa",
                        xmeta_room::ToSqlValue::to_sql_value(&params.prowizja_podstawa),
                    ),
                    (
                        "oplata_sprzatanie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.oplata_sprzatanie),
                    ),
                    (
                        "oplata_sprzatanie_pokrywa_koszt",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.oplata_sprzatanie_pokrywa_koszt,
                        ),
                    ),
                    (
                        "kwota_napraw",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kwota_napraw),
                    ),
                    (
                        "koszty_wlasciciela",
                        xmeta_room::ToSqlValue::to_sql_value(&params.koszty_wlasciciela),
                    ),
                    (
                        "zgloszenie_nocy_dni",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zgloszenie_nocy_dni),
                    ),
                    (
                        "wyplata_dzien",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wyplata_dzien),
                    ),
                    ("rezerwa", xmeta_room::ToSqlValue::to_sql_value(&params.rezerwa)),
                    (
                        "po_koncu_rezerwacje",
                        xmeta_room::ToSqlValue::to_sql_value(&params.po_koncu_rezerwacje),
                    ),
                    (
                        "aneks_zdarzenie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.aneks_zdarzenie_id),
                    ),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                ]),
            },
        ],
    )
}

/// The parameters of `ustal_warunki_umowy_lokalu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UstalWarunkiUmowyLokaluParams {
    pub id: String,
}

/// Projekt z kanałami i widełkami zaczyna obowiązywać; potem się go nie zmienia.
pub fn ustal_warunki_umowy_lokalu(
    room: &impl xmeta_room::MetaRoom,
    params: &UstalWarunkiUmowyLokaluParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_umowa_warunki
SET stan = 'obowiazuje'
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `widelki_warunkow`.
#[derive(Debug, Clone, PartialEq)]
pub struct WidelkiWarunkowRow {
    pub id: i64,
    pub warunki_id: String,
    pub sezon: String,
    pub od_dnia: String,
    pub do_dnia: String,
    pub cena_min: i64,
    pub cena_max: i64,
    pub pobyt_min: i64,
}

impl WidelkiWarunkowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WidelkiWarunkowRow {
            id: row.get("id")?,
            warunki_id: row.get("warunki_id")?,
            sezon: row.get("sezon")?,
            od_dnia: row.get("od_dnia")?,
            do_dnia: row.get("do_dnia")?,
            cena_min: row.get("cena_min")?,
            cena_max: row.get("cena_max")?,
            pobyt_min: row.get("pobyt_min")?,
        })
    }
}

/// Widełki cen każdej wersji warunków w kolejności okresów.
pub fn widelki_warunkow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WidelkiWarunkowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_umowa_widelki
ORDER BY warunki_id, od_dnia"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(WidelkiWarunkowRow::from_row).collect()
}

/// The parameters of `dopisz_widelki_warunkow`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DopiszWidelkiWarunkowParams {
    pub warunki_id: String,
    pub sezon: String,
    pub od_dnia: String,
    pub do_dnia: String,
    pub cena_min: i64,
    pub cena_max: i64,
    pub pobyt_min: i64,
}

/// Najniższa i najwyższa cena nocy oraz najkrótszy pobyt w okresie, w projekcie warunków.
pub fn dopisz_widelki_warunkow(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszWidelkiWarunkowParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_umowa_widelki (warunki_id, sezon, od_dnia, do_dnia, cena_min,
  cena_max, pobyt_min)
VALUES (:warunki_id, :sezon, :od_dnia, :do_dnia, :cena_min, :cena_max, :pobyt_min)"#,
                parameters: Vec::from([
                    (
                        "warunki_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.warunki_id),
                    ),
                    ("sezon", xmeta_room::ToSqlValue::to_sql_value(&params.sezon)),
                    ("od_dnia", xmeta_room::ToSqlValue::to_sql_value(&params.od_dnia)),
                    ("do_dnia", xmeta_room::ToSqlValue::to_sql_value(&params.do_dnia)),
                    ("cena_min", xmeta_room::ToSqlValue::to_sql_value(&params.cena_min)),
                    ("cena_max", xmeta_room::ToSqlValue::to_sql_value(&params.cena_max)),
                    (
                        "pobyt_min",
                        xmeta_room::ToSqlValue::to_sql_value(&params.pobyt_min),
                    ),
                ]),
            },
        ],
    )
}
