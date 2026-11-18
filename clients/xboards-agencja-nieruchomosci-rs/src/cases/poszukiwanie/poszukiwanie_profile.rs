//! The kind `poszukiwanie-profile` of the room `cases/poszukiwanie`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "poszukiwanie-profile";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Profile poszukiwania: czego klient szuka, ile zapłaci i skąd weźmie pieniądze, w wersjach
-- z potwierdzeniem klienta. Pisze Opiekun poszukującego. Kwoty w groszach; budżet przy najmie to
-- najwyższy czynsz miesięczny.
--
-- Opiekun zapisuje wersję z cechami i finansowaniem, a klient ją potwierdza zgodą
-- potwierdzenie_profilu, która dotyczy tej wersji. Potwierdzona wersja się nie zmienia i nie
-- dostaje nowych cech; zmiana profilu to nowa wersja. Propozycje porównuje się z bieżącą
-- potwierdzoną wersją, a etap propozycje zaczyna się dopiero przy potwierdzonym profilu.
--
-- Budżet i finansowanie nie wychodzą poza teczkę poszukiwania (R1); pilnuje tego aplikacja.

-- Profil jednej szukanej nieruchomości; kilka profili ma klient, który kupuje kilka nieruchomości
-- na jednej umowie.
CREATE TABLE poszukiwanie_profile_wpisy (
  id TEXT PRIMARY KEY,
  nazwa TEXT NOT NULL
) STRICT;

CREATE TABLE poszukiwanie_profile_wersje (
  id TEXT PRIMARY KEY,
  profil_id TEXT NOT NULL REFERENCES poszukiwanie_profile_wpisy (id),

  -- Czego klient szuka. Rynek wtórny; zakup od dewelopera nie należy do tej teczki.
  cel TEXT NOT NULL CHECK (cel IN ('kupno', 'najem')),
  -- Kupno na wynajem, a nie dla siebie.
  na_wynajem INTEGER NOT NULL CHECK (na_wynajem IN (0, 1)),
  -- Np. „mieszkanie albo dom”.
  rodzaj_nieruchomosci TEXT NOT NULL,
  -- Przeznaczenie działki albo lokalu użytkowego, np. zabudowa jednorodzinna albo biuro.
  przeznaczenie TEXT,
  polozenie TEXT NOT NULL,
  -- Czas dojazdu do wskazanego miejsca, np. „do 30 minut do szkoły przy ul. Lipowej”.
  dojazd TEXT,
  -- Powierzchnia w setnych metra kwadratowego, jak w rdzeniu.
  pow_min INTEGER CHECK (pow_min > 0),
  pow_max INTEGER CHECK (pow_max > 0),
  pokoje_min INTEGER CHECK (pokoje_min >= 0),
  pokoje_max INTEGER CHECK (pokoje_max >= 0),
  pietro TEXT,

  -- Kiedy i za ile: dzień, w którym klient musi się wprowadzić, i najwyższa kwota.
  termin TEXT,
  termin_powod TEXT,
  budzet INTEGER NOT NULL CHECK (budzet > 0),
  -- Czy budżet obejmuje koszty dodatkowe: podatek, taksę notarialną albo prowizję.
  budzet_z_kosztami INTEGER NOT NULL CHECK (budzet_z_kosztami IN (0, 1)),

  -- Najem: od kiedy i na jak długo, gospodarstwo, najem okazjonalny (art. 19a ust. 2 UOPL),
  -- najwyższa kaucja i umeblowanie (NULL: obojętne).
  najem_od TEXT,
  okres TEXT,
  osoby INTEGER CHECK (osoby > 0),
  dzieci INTEGER CHECK (dzieci >= 0),
  zwierzeta TEXT,
  okazjonalny_dopuszcza INTEGER CHECK (okazjonalny_dopuszcza IN (0, 1)),
  -- Czy klient ma inny lokal, w którym zamieszka po egzekucji, ze zgodą jego właściciela.
  inny_lokal_ma INTEGER CHECK (inny_lokal_ma IN (0, 1)),
  kaucja_max INTEGER CHECK (kaucja_max >= 0),
  umeblowany INTEGER CHECK (umeblowany IN (0, 1)),

  -- Potwierdzenie klienta: zgoda potwierdzenie_profilu, która dotyczy tej wersji; NULL znaczy
  -- wersję, której klient jeszcze nie potwierdził.
  zgoda_id TEXT REFERENCES sprawa_zgody_wpisy (id),
  od TEXT NOT NULL,

  CONSTRAINT na_wynajem_przy_kupnie CHECK (cel = 'kupno' OR na_wynajem = 0),
  CONSTRAINT powierzchnia_od_do CHECK (pow_min IS NULL OR pow_max IS NULL OR pow_min <= pow_max),
  CONSTRAINT pokoje_od_do CHECK (pokoje_min IS NULL OR pokoje_max IS NULL OR pokoje_min <= pokoje_max),
  CONSTRAINT powod_przy_terminie CHECK (termin_powod IS NULL OR termin IS NOT NULL),
  CONSTRAINT najem_ma_okres_osoby_i_rodzaj CHECK (
    cel <> 'najem'
    OR (najem_od IS NOT NULL AND osoby IS NOT NULL AND okazjonalny_dopuszcza IS NOT NULL)
  ),
  CONSTRAINT dane_najmu_tylko_przy_najmie CHECK (
    cel = 'najem'
    OR (
      najem_od IS NULL AND okres IS NULL AND osoby IS NULL AND dzieci IS NULL AND zwierzeta IS NULL
      AND okazjonalny_dopuszcza IS NULL AND inny_lokal_ma IS NULL AND kaucja_max IS NULL
      AND umeblowany IS NULL
    )
  ),
  -- Przed propozycją najmu okazjonalnego Opiekun pyta, czy klient ma inny lokal.
  CONSTRAINT okazjonalny_z_odpowiedzia_o_innym_lokalu CHECK (
    okazjonalny_dopuszcza IS NOT 1 OR inny_lokal_ma IS NOT NULL
  )
) STRICT;

-- Cechy z oznaczeniem, czy są warunkiem, czy życzeniem, np. „balkon” jako warunek.
CREATE TABLE poszukiwanie_profile_cechy (
  -- Klucz nadany przez bazę jest dodatni; zob. poszukiwanie_profile_cechy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  wersja_id TEXT NOT NULL REFERENCES poszukiwanie_profile_wersje (id),
  cecha TEXT NOT NULL,
  waga TEXT NOT NULL CHECK (waga IN ('warunek', 'zyczenie')),

  UNIQUE (wersja_id, cecha)
) STRICT;

-- Źródła pieniędzy z kwotą każdego według deklaracji klienta.
CREATE TABLE poszukiwanie_profile_finansowanie (
  -- Klucz nadany przez bazę jest dodatni; zob. poszukiwanie_profile_finansowanie_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  wersja_id TEXT NOT NULL REFERENCES poszukiwanie_profile_wersje (id),
  zrodlo TEXT NOT NULL CHECK (zrodlo IN ('oszczednosci', 'kredyt', 'sprzedaz_nieruchomosci', 'inne')),
  kwota INTEGER NOT NULL CHECK (kwota > 0),

  UNIQUE (wersja_id, zrodlo)
) STRICT;

-- Bieżący profil: najnowsza potwierdzona wersja każdego profilu.
CREATE VIEW poszukiwanie_profile_biezace AS
SELECT wersja.*
FROM poszukiwanie_profile_wersje AS wersja
WHERE wersja.zgoda_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM poszukiwanie_profile_wersje AS nowsza
    WHERE nowsza.profil_id = wersja.profil_id
      AND nowsza.zgoda_id IS NOT NULL
      AND nowsza.od > wersja.od
  );

-- Potwierdzenie sprawdza zmianę, więc wersja powstaje niepotwierdzona; jest późniejsza niż każda
-- wcześniejsza wersja tego profilu.
CREATE TRIGGER poszukiwanie_profile_wersja_powstaje_niepotwierdzona
BEFORE INSERT ON poszukiwanie_profile_wersje
WHEN NEW.zgoda_id IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'wersja profilu powstaje niepotwierdzona; potwierdza ją potem klient');
END;

CREATE TRIGGER poszukiwanie_profile_wersje_kolejno
BEFORE INSERT ON poszukiwanie_profile_wersje
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_profile_wersje
  WHERE profil_id = NEW.profil_id
    AND od >= NEW.od
)
BEGIN
  SELECT RAISE(ABORT, 'nowa wersja profilu jest późniejsza niż poprzednie wersje tego profilu');
END;

-- Profil potwierdza klient: zgoda potwierdzenie_profilu uczestnika z obowiązującą rolą klient,
-- która dotyczy tej wersji.
CREATE TRIGGER poszukiwanie_profile_potwierdzenie_ze_zgoda_klienta
BEFORE UPDATE OF zgoda_id ON poszukiwanie_profile_wersje
WHEN NEW.zgoda_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_zgody_wpisy AS zgoda
    JOIN sprawa_uczestnicy_role AS rola ON rola.uczestnik_id = zgoda.uczestnik_id
    WHERE zgoda.id = NEW.zgoda_id
      AND zgoda.rodzaj = 'potwierdzenie_profilu'
      AND zgoda.dotyczy_tabela = 'poszukiwanie_profile_wersje'
      AND zgoda.dotyczy_id = NEW.id
      AND rola.rola = 'klient'
      AND rola.zakonczono IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'wersja profilu ma zgodę potwierdzenie_profilu klienta, która jej dotyczy');
END;

-- Wersji się nie zmienia; raz wpisuje się tylko potwierdzenie klienta.
CREATE TRIGGER poszukiwanie_profile_wersje_bez_zmian
BEFORE UPDATE ON poszukiwanie_profile_wersje
WHEN OLD.zgoda_id IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.profil_id IS NOT OLD.profil_id
  OR NEW.cel IS NOT OLD.cel
  OR NEW.na_wynajem IS NOT OLD.na_wynajem
  OR NEW.rodzaj_nieruchomosci IS NOT OLD.rodzaj_nieruchomosci
  OR NEW.przeznaczenie IS NOT OLD.przeznaczenie
  OR NEW.polozenie IS NOT OLD.polozenie
  OR NEW.dojazd IS NOT OLD.dojazd
  OR NEW.pow_min IS NOT OLD.pow_min
  OR NEW.pow_max IS NOT OLD.pow_max
  OR NEW.pokoje_min IS NOT OLD.pokoje_min
  OR NEW.pokoje_max IS NOT OLD.pokoje_max
  OR NEW.pietro IS NOT OLD.pietro
  OR NEW.termin IS NOT OLD.termin
  OR NEW.termin_powod IS NOT OLD.termin_powod
  OR NEW.budzet IS NOT OLD.budzet
  OR NEW.budzet_z_kosztami IS NOT OLD.budzet_z_kosztami
  OR NEW.najem_od IS NOT OLD.najem_od
  OR NEW.okres IS NOT OLD.okres
  OR NEW.osoby IS NOT OLD.osoby
  OR NEW.dzieci IS NOT OLD.dzieci
  OR NEW.zwierzeta IS NOT OLD.zwierzeta
  OR NEW.okazjonalny_dopuszcza IS NOT OLD.okazjonalny_dopuszcza
  OR NEW.inny_lokal_ma IS NOT OLD.inny_lokal_ma
  OR NEW.kaucja_max IS NOT OLD.kaucja_max
  OR NEW.umeblowany IS NOT OLD.umeblowany
  OR NEW.od IS NOT OLD.od
BEGIN
  SELECT RAISE(ABORT, 'wersji profilu się nie zmienia; raz wpisuje się potwierdzenie klienta, a zmiana to nowa wersja');
END;

-- Cechy i finansowanie są częścią tego, co klient potwierdza, więc dopisuje się je tylko do
-- niepotwierdzonej wersji.
CREATE TRIGGER poszukiwanie_profile_cechy_przed_potwierdzeniem
BEFORE INSERT ON poszukiwanie_profile_cechy
WHEN (
  SELECT zgoda_id IS NOT NULL
  FROM poszukiwanie_profile_wersje
  WHERE id = NEW.wersja_id
)
BEGIN
  SELECT RAISE(ABORT, 'cechy i finansowanie dopisuje się tylko do niepotwierdzonej wersji profilu');
END;

CREATE TRIGGER poszukiwanie_profile_finansowanie_przed_potwierdzeniem
BEFORE INSERT ON poszukiwanie_profile_finansowanie
WHEN (
  SELECT zgoda_id IS NOT NULL
  FROM poszukiwanie_profile_wersje
  WHERE id = NEW.wersja_id
)
BEGIN
  SELECT RAISE(ABORT, 'cechy i finansowanie dopisuje się tylko do niepotwierdzonej wersji profilu');
END;

CREATE TRIGGER poszukiwanie_profile_cechy_bez_zmian
BEFORE UPDATE ON poszukiwanie_profile_cechy
BEGIN
  SELECT RAISE(ABORT, 'cechy profilu się nie zmienia; zmiana profilu to nowa wersja');
END;

CREATE TRIGGER poszukiwanie_profile_finansowanie_bez_zmian
BEFORE UPDATE ON poszukiwanie_profile_finansowanie
BEGIN
  SELECT RAISE(ABORT, 'finansowania profilu się nie zmienia; zmiana profilu to nowa wersja');
END;

CREATE TRIGGER poszukiwanie_profile_wpisy_bez_zmian
BEFORE UPDATE OF id ON poszukiwanie_profile_wpisy
WHEN NEW.id IS NOT OLD.id
BEGIN
  SELECT RAISE(ABORT, 'identyfikator profilu się nie zmienia');
END;

-- Propozycje zaczynają się przy profilu, który klient potwierdził.
CREATE TRIGGER poszukiwanie_profile_propozycje_po_potwierdzeniu
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap = 'propozycje'
  AND NOT EXISTS (
    SELECT 1
    FROM poszukiwanie_profile_wersje
    WHERE zgoda_id IS NOT NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'etap propozycje wymaga profilu poszukiwania potwierdzonego przez klienta');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER poszukiwanie_profile_wpisy_bez_zastepowania
BEFORE INSERT ON poszukiwanie_profile_wpisy
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_profile_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_profile_wpisy_bez_usuwania
BEFORE DELETE ON poszukiwanie_profile_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER poszukiwanie_profile_wersje_bez_zastepowania
BEFORE INSERT ON poszukiwanie_profile_wersje
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_profile_wersje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_profile_wersje_bez_usuwania
BEFORE DELETE ON poszukiwanie_profile_wersje
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER poszukiwanie_profile_cechy_bez_zastepowania
BEFORE INSERT ON poszukiwanie_profile_cechy
WHEN EXISTS (
    SELECT 1
    FROM poszukiwanie_profile_cechy
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM poszukiwanie_profile_cechy
    WHERE wersja_id = NEW.wersja_id
      AND cecha = NEW.cecha
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_profile_cechy_bez_usuwania
BEFORE DELETE ON poszukiwanie_profile_cechy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER poszukiwanie_profile_finansowanie_bez_zastepowania
BEFORE INSERT ON poszukiwanie_profile_finansowanie
WHEN EXISTS (
    SELECT 1
    FROM poszukiwanie_profile_finansowanie
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM poszukiwanie_profile_finansowanie
    WHERE wersja_id = NEW.wersja_id
      AND zrodlo = NEW.zrodlo
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_profile_finansowanie_bez_usuwania
BEFORE DELETE ON poszukiwanie_profile_finansowanie
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
];

/// The values of column `cechy.waga`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CechyWaga {
    Warunek,
    Zyczenie,
}

impl CechyWaga {
    /// Every value, in the order of the catalog.
    pub const ALL: &[CechyWaga] = &[CechyWaga::Warunek, CechyWaga::Zyczenie];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            CechyWaga::Warunek => "warunek",
            CechyWaga::Zyczenie => "zyczenie",
        }
    }
}

impl xmeta_room::FromSqlValue for CechyWaga {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "warunek" => Ok(CechyWaga::Warunek),
                    "zyczenie" => Ok(CechyWaga::Zyczenie),
                    other => Err(format!("{other} is no value of CechyWaga")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for CechyWaga {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `finansowanie.zrodlo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FinansowanieZrodlo {
    Oszczednosci,
    Kredyt,
    SprzedazNieruchomosci,
    Inne,
}

impl FinansowanieZrodlo {
    /// Every value, in the order of the catalog.
    pub const ALL: &[FinansowanieZrodlo] = &[
        FinansowanieZrodlo::Oszczednosci,
        FinansowanieZrodlo::Kredyt,
        FinansowanieZrodlo::SprzedazNieruchomosci,
        FinansowanieZrodlo::Inne,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            FinansowanieZrodlo::Oszczednosci => "oszczednosci",
            FinansowanieZrodlo::Kredyt => "kredyt",
            FinansowanieZrodlo::SprzedazNieruchomosci => "sprzedaz_nieruchomosci",
            FinansowanieZrodlo::Inne => "inne",
        }
    }
}

impl xmeta_room::FromSqlValue for FinansowanieZrodlo {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "oszczednosci" => Ok(FinansowanieZrodlo::Oszczednosci),
                    "kredyt" => Ok(FinansowanieZrodlo::Kredyt),
                    "sprzedaz_nieruchomosci" => {
                        Ok(FinansowanieZrodlo::SprzedazNieruchomosci)
                    }
                    "inne" => Ok(FinansowanieZrodlo::Inne),
                    other => Err(format!("{other} is no value of FinansowanieZrodlo")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for FinansowanieZrodlo {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wersje.cel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WersjeCel {
    Kupno,
    Najem,
}

impl WersjeCel {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WersjeCel] = &[WersjeCel::Kupno, WersjeCel::Najem];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WersjeCel::Kupno => "kupno",
            WersjeCel::Najem => "najem",
        }
    }
}

impl xmeta_room::FromSqlValue for WersjeCel {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "kupno" => Ok(WersjeCel::Kupno),
                    "najem" => Ok(WersjeCel::Najem),
                    other => Err(format!("{other} is no value of WersjeCel")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WersjeCel {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `wersje_profili`.
#[derive(Debug, Clone, PartialEq)]
pub struct WersjeProfiliRow {
    pub id: String,
    pub profil_id: String,
    pub cel: WersjeCel,
    pub na_wynajem: bool,
    pub rodzaj_nieruchomosci: String,
    pub przeznaczenie: Option<String>,
    pub polozenie: String,
    pub dojazd: Option<String>,
    pub pow_min: Option<i64>,
    pub pow_max: Option<i64>,
    pub pokoje_min: Option<i64>,
    pub pokoje_max: Option<i64>,
    pub pietro: Option<String>,
    pub termin: Option<String>,
    pub termin_powod: Option<String>,
    pub budzet: i64,
    pub budzet_z_kosztami: bool,
    pub najem_od: Option<String>,
    pub okres: Option<String>,
    pub osoby: Option<i64>,
    pub dzieci: Option<i64>,
    pub zwierzeta: Option<String>,
    pub okazjonalny_dopuszcza: Option<bool>,
    pub inny_lokal_ma: Option<bool>,
    pub kaucja_max: Option<i64>,
    pub umeblowany: Option<bool>,
    pub zgoda_id: Option<String>,
    pub od: String,
}

impl WersjeProfiliRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WersjeProfiliRow {
            id: row.get("id")?,
            profil_id: row.get("profil_id")?,
            cel: row.get("cel")?,
            na_wynajem: row.get("na_wynajem")?,
            rodzaj_nieruchomosci: row.get("rodzaj_nieruchomosci")?,
            przeznaczenie: row.get("przeznaczenie")?,
            polozenie: row.get("polozenie")?,
            dojazd: row.get("dojazd")?,
            pow_min: row.get("pow_min")?,
            pow_max: row.get("pow_max")?,
            pokoje_min: row.get("pokoje_min")?,
            pokoje_max: row.get("pokoje_max")?,
            pietro: row.get("pietro")?,
            termin: row.get("termin")?,
            termin_powod: row.get("termin_powod")?,
            budzet: row.get("budzet")?,
            budzet_z_kosztami: row.get("budzet_z_kosztami")?,
            najem_od: row.get("najem_od")?,
            okres: row.get("okres")?,
            osoby: row.get("osoby")?,
            dzieci: row.get("dzieci")?,
            zwierzeta: row.get("zwierzeta")?,
            okazjonalny_dopuszcza: row.get("okazjonalny_dopuszcza")?,
            inny_lokal_ma: row.get("inny_lokal_ma")?,
            kaucja_max: row.get("kaucja_max")?,
            umeblowany: row.get("umeblowany")?,
            zgoda_id: row.get("zgoda_id")?,
            od: row.get("od")?,
        })
    }
}

/// Wersje profili od najstarszej, także niepotwierdzone.
pub fn wersje_profili(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WersjeProfiliRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM poszukiwanie_profile_wersje
ORDER BY profil_id, od"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(WersjeProfiliRow::from_row).collect()
}

/// A row of `profile_biezace`.
#[derive(Debug, Clone, PartialEq)]
pub struct ProfileBiezaceRow {
    pub id: String,
    pub profil_id: String,
    pub cel: WersjeCel,
    pub na_wynajem: bool,
    pub rodzaj_nieruchomosci: String,
    pub przeznaczenie: Option<String>,
    pub polozenie: String,
    pub dojazd: Option<String>,
    pub pow_min: Option<i64>,
    pub pow_max: Option<i64>,
    pub pokoje_min: Option<i64>,
    pub pokoje_max: Option<i64>,
    pub pietro: Option<String>,
    pub termin: Option<String>,
    pub termin_powod: Option<String>,
    pub budzet: i64,
    pub budzet_z_kosztami: bool,
    pub najem_od: Option<String>,
    pub okres: Option<String>,
    pub osoby: Option<i64>,
    pub dzieci: Option<i64>,
    pub zwierzeta: Option<String>,
    pub okazjonalny_dopuszcza: Option<bool>,
    pub inny_lokal_ma: Option<bool>,
    pub kaucja_max: Option<i64>,
    pub umeblowany: Option<bool>,
    pub zgoda_id: Option<String>,
    pub od: String,
}

impl ProfileBiezaceRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ProfileBiezaceRow {
            id: row.get("id")?,
            profil_id: row.get("profil_id")?,
            cel: row.get("cel")?,
            na_wynajem: row.get("na_wynajem")?,
            rodzaj_nieruchomosci: row.get("rodzaj_nieruchomosci")?,
            przeznaczenie: row.get("przeznaczenie")?,
            polozenie: row.get("polozenie")?,
            dojazd: row.get("dojazd")?,
            pow_min: row.get("pow_min")?,
            pow_max: row.get("pow_max")?,
            pokoje_min: row.get("pokoje_min")?,
            pokoje_max: row.get("pokoje_max")?,
            pietro: row.get("pietro")?,
            termin: row.get("termin")?,
            termin_powod: row.get("termin_powod")?,
            budzet: row.get("budzet")?,
            budzet_z_kosztami: row.get("budzet_z_kosztami")?,
            najem_od: row.get("najem_od")?,
            okres: row.get("okres")?,
            osoby: row.get("osoby")?,
            dzieci: row.get("dzieci")?,
            zwierzeta: row.get("zwierzeta")?,
            okazjonalny_dopuszcza: row.get("okazjonalny_dopuszcza")?,
            inny_lokal_ma: row.get("inny_lokal_ma")?,
            kaucja_max: row.get("kaucja_max")?,
            umeblowany: row.get("umeblowany")?,
            zgoda_id: row.get("zgoda_id")?,
            od: row.get("od")?,
        })
    }
}

/// Najnowsza potwierdzona wersja każdego profilu; z nią porównuje się propozycje.
pub fn profile_biezace(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ProfileBiezaceRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM poszukiwanie_profile_biezace
ORDER BY profil_id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(ProfileBiezaceRow::from_row).collect()
}

/// A row of `cechy_profili`.
#[derive(Debug, Clone, PartialEq)]
pub struct CechyProfiliRow {
    pub id: i64,
    pub wersja_id: String,
    pub cecha: String,
    pub waga: CechyWaga,
}

impl CechyProfiliRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(CechyProfiliRow {
            id: row.get("id")?,
            wersja_id: row.get("wersja_id")?,
            cecha: row.get("cecha")?,
            waga: row.get("waga")?,
        })
    }
}

/// Cechy wersji profili z oznaczeniem warunku albo życzenia.
pub fn cechy_profili(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<CechyProfiliRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM poszukiwanie_profile_cechy
ORDER BY wersja_id, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(CechyProfiliRow::from_row).collect()
}

/// A row of `finansowanie_profili`.
#[derive(Debug, Clone, PartialEq)]
pub struct FinansowanieProfiliRow {
    pub id: i64,
    pub wersja_id: String,
    pub zrodlo: FinansowanieZrodlo,
    pub kwota: i64,
}

impl FinansowanieProfiliRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(FinansowanieProfiliRow {
            id: row.get("id")?,
            wersja_id: row.get("wersja_id")?,
            zrodlo: row.get("zrodlo")?,
            kwota: row.get("kwota")?,
        })
    }
}

/// Źródła pieniędzy wersji profili według deklaracji klienta.
pub fn finansowanie_profili(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<FinansowanieProfiliRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM poszukiwanie_profile_finansowanie
ORDER BY wersja_id, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(FinansowanieProfiliRow::from_row).collect()
}

/// The parameters of `wpisz_wersje_profilu`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszWersjeProfiluParams {
    pub id: String,
    pub profil_id: String,
    pub cel: WersjeCel,
    pub na_wynajem: bool,
    pub rodzaj_nieruchomosci: String,
    pub przeznaczenie: Option<String>,
    pub polozenie: String,
    pub dojazd: Option<String>,
    pub pow_min: Option<i64>,
    pub pow_max: Option<i64>,
    pub pokoje_min: Option<i64>,
    pub pokoje_max: Option<i64>,
    pub pietro: Option<String>,
    pub termin: Option<String>,
    pub termin_powod: Option<String>,
    pub budzet: i64,
    pub budzet_z_kosztami: bool,
    pub najem_od: Option<String>,
    pub okres: Option<String>,
    pub osoby: Option<i64>,
    pub dzieci: Option<i64>,
    pub zwierzeta: Option<String>,
    pub okazjonalny_dopuszcza: Option<bool>,
    pub inny_lokal_ma: Option<bool>,
    pub kaucja_max: Option<i64>,
    pub umeblowany: Option<bool>,
    pub od: String,
}

/// Nowa, jeszcze niepotwierdzona wersja profilu; identyfikator (UUID) nadaje aplikacja, bo
/// wskazują go cechy, finansowanie, zgoda klienta i propozycje.
pub fn wpisz_wersje_profilu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszWersjeProfiluParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO poszukiwanie_profile_wersje (id, profil_id, cel, na_wynajem, rodzaj_nieruchomosci,
  przeznaczenie, polozenie, dojazd, pow_min, pow_max, pokoje_min, pokoje_max, pietro, termin,
  termin_powod, budzet, budzet_z_kosztami, najem_od, okres, osoby, dzieci, zwierzeta,
  okazjonalny_dopuszcza, inny_lokal_ma, kaucja_max, umeblowany, od)
VALUES (:id, :profil_id, :cel, :na_wynajem, :rodzaj_nieruchomosci, :przeznaczenie, :polozenie,
  :dojazd, :pow_min, :pow_max, :pokoje_min, :pokoje_max, :pietro, :termin, :termin_powod, :budzet,
  :budzet_z_kosztami, :najem_od, :okres, :osoby, :dzieci, :zwierzeta, :okazjonalny_dopuszcza,
  :inny_lokal_ma, :kaucja_max, :umeblowany, :od)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "profil_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.profil_id),
                    ),
                    ("cel", xmeta_room::ToSqlValue::to_sql_value(&params.cel)),
                    (
                        "na_wynajem",
                        xmeta_room::ToSqlValue::to_sql_value(&params.na_wynajem),
                    ),
                    (
                        "rodzaj_nieruchomosci",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.rodzaj_nieruchomosci,
                        ),
                    ),
                    (
                        "przeznaczenie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przeznaczenie),
                    ),
                    (
                        "polozenie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.polozenie),
                    ),
                    ("dojazd", xmeta_room::ToSqlValue::to_sql_value(&params.dojazd)),
                    ("pow_min", xmeta_room::ToSqlValue::to_sql_value(&params.pow_min)),
                    ("pow_max", xmeta_room::ToSqlValue::to_sql_value(&params.pow_max)),
                    (
                        "pokoje_min",
                        xmeta_room::ToSqlValue::to_sql_value(&params.pokoje_min),
                    ),
                    (
                        "pokoje_max",
                        xmeta_room::ToSqlValue::to_sql_value(&params.pokoje_max),
                    ),
                    ("pietro", xmeta_room::ToSqlValue::to_sql_value(&params.pietro)),
                    ("termin", xmeta_room::ToSqlValue::to_sql_value(&params.termin)),
                    (
                        "termin_powod",
                        xmeta_room::ToSqlValue::to_sql_value(&params.termin_powod),
                    ),
                    ("budzet", xmeta_room::ToSqlValue::to_sql_value(&params.budzet)),
                    (
                        "budzet_z_kosztami",
                        xmeta_room::ToSqlValue::to_sql_value(&params.budzet_z_kosztami),
                    ),
                    ("najem_od", xmeta_room::ToSqlValue::to_sql_value(&params.najem_od)),
                    ("okres", xmeta_room::ToSqlValue::to_sql_value(&params.okres)),
                    ("osoby", xmeta_room::ToSqlValue::to_sql_value(&params.osoby)),
                    ("dzieci", xmeta_room::ToSqlValue::to_sql_value(&params.dzieci)),
                    (
                        "zwierzeta",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zwierzeta),
                    ),
                    (
                        "okazjonalny_dopuszcza",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.okazjonalny_dopuszcza,
                        ),
                    ),
                    (
                        "inny_lokal_ma",
                        xmeta_room::ToSqlValue::to_sql_value(&params.inny_lokal_ma),
                    ),
                    (
                        "kaucja_max",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kaucja_max),
                    ),
                    (
                        "umeblowany",
                        xmeta_room::ToSqlValue::to_sql_value(&params.umeblowany),
                    ),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                ]),
            },
        ],
    )
}

/// The parameters of `dopisz_ceche_profilu`.
#[derive(Debug, Clone, PartialEq)]
pub struct DopiszCecheProfiluParams {
    pub wersja_id: String,
    pub cecha: String,
    pub waga: CechyWaga,
}

/// Cecha niepotwierdzonej wersji: warunek albo życzenie.
pub fn dopisz_ceche_profilu(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszCecheProfiluParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO poszukiwanie_profile_cechy (wersja_id, cecha, waga)
VALUES (:wersja_id, :cecha, :waga)"#,
                parameters: Vec::from([
                    (
                        "wersja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wersja_id),
                    ),
                    ("cecha", xmeta_room::ToSqlValue::to_sql_value(&params.cecha)),
                    ("waga", xmeta_room::ToSqlValue::to_sql_value(&params.waga)),
                ]),
            },
        ],
    )
}

/// The parameters of `dopisz_finansowanie_profilu`.
#[derive(Debug, Clone, PartialEq)]
pub struct DopiszFinansowanieProfiluParams {
    pub wersja_id: String,
    pub zrodlo: FinansowanieZrodlo,
    pub kwota: i64,
}

/// Źródło pieniędzy niepotwierdzonej wersji z kwotą według deklaracji klienta.
pub fn dopisz_finansowanie_profilu(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszFinansowanieProfiluParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO poszukiwanie_profile_finansowanie (wersja_id, zrodlo, kwota)
VALUES (:wersja_id, :zrodlo, :kwota)"#,
                parameters: Vec::from([
                    (
                        "wersja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wersja_id),
                    ),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                    ("kwota", xmeta_room::ToSqlValue::to_sql_value(&params.kwota)),
                ]),
            },
        ],
    )
}

/// The parameters of `potwierdz_wersje_profilu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PotwierdzWersjeProfiluParams {
    pub zgoda_id: String,
    pub id: String,
}

/// Klient potwierdził wersję profilu zgodą potwierdzenie_profilu; od tej chwili wersja się nie
/// zmienia.
pub fn potwierdz_wersje_profilu(
    room: &impl xmeta_room::MetaRoom,
    params: &PotwierdzWersjeProfiluParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE poszukiwanie_profile_wersje
SET zgoda_id = :zgoda_id
WHERE id = :id"#,
                parameters: Vec::from([
                    ("zgoda_id", xmeta_room::ToSqlValue::to_sql_value(&params.zgoda_id)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `profile_poszukiwania`.
#[derive(Debug, Clone, PartialEq)]
pub struct ProfilePoszukiwaniaRow {
    pub id: String,
    pub nazwa: String,
}

impl ProfilePoszukiwaniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ProfilePoszukiwaniaRow {
            id: row.get("id")?,
            nazwa: row.get("nazwa")?,
        })
    }
}

/// Profile poszukiwania w teczce; kilka, gdy klient kupuje kilka nieruchomości.
pub fn profile_poszukiwania(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ProfilePoszukiwaniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM poszukiwanie_profile_wpisy
ORDER BY rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(ProfilePoszukiwaniaRow::from_row).collect()
}

/// The parameters of `zaloz_profil_poszukiwania`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZalozProfilPoszukiwaniaParams {
    pub id: String,
    pub nazwa: String,
}

/// Nowy profil szukanej nieruchomości; identyfikator (UUID) nadaje aplikacja, bo wskazują go
/// wersje.
pub fn zaloz_profil_poszukiwania(
    room: &impl xmeta_room::MetaRoom,
    params: &ZalozProfilPoszukiwaniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO poszukiwanie_profile_wpisy (id, nazwa)
VALUES (:id, :nazwa)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("nazwa", xmeta_room::ToSqlValue::to_sql_value(&params.nazwa)),
                ]),
            },
        ],
    )
}
