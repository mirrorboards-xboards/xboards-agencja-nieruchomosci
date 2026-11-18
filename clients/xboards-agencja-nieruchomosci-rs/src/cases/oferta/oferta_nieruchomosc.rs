//! The kind `oferta-nieruchomosc` of the room `cases/oferta`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "oferta-nieruchomosc",
    requires: &[&super::sprawa_dokumenty::KIND, &super::sprawa_nieruchomosci::KIND],
    migrations: &[
        r#"-- Opis nieruchomości oferty ponad oznaczenie z rdzenia (sprawa_nieruchomosci_wpisy) i wyniki
-- przeglądania księgi wieczystej. Pisze Opiekun oferty. Źródło każdej danej (dokument, rejestr
-- z datą, oględziny pośrednika albo słowa klienta) wpisuje się w sprawa_dokumenty_zrodla rdzenia.
--
-- Wydruk z przeglądania księgi nie ma mocy dokumentu sądowego (art. 36⁴ ust. 7 ukwh), więc teczka
-- trzyma przeglądanie jako własną notatkę, a nie jako odpis wśród dokumentów. Opiekun spisuje
-- wpisy z działów III i IV bez oceny; ocenia je Analityk stanu prawnego w transakcji.

-- Opis jednej nieruchomości: budynek i lokal, działka, koszty utrzymania, świadectwo
-- charakterystyki energetycznej i osoby w lokalu. Kwoty w groszach.
CREATE TABLE oferta_nieruchomosc_opis (
  nieruchomosc_id TEXT PRIMARY KEY REFERENCES sprawa_nieruchomosci_wpisy (id),

  -- Budynek i lokal; NULL znaczy, że dana nie jest znana.
  liczba_pieter INTEGER CHECK (liczba_pieter >= 0),
  winda INTEGER CHECK (winda IN (0, 1)),
  rok_budowy INTEGER CHECK (rok_budowy > 0),
  material TEXT,
  ogrzewanie TEXT,
  stan_lokalu TEXT,
  remonty TEXT,

  -- Działka: przeznaczenie z planu miejscowego albo z decyzji o warunkach zabudowy. Powierzchnia
  -- użytków rolnych z ewidencji, w metrach kwadratowych, bo od niej zależą ograniczenia obrotu
  -- (art. 1a pkt 1 lit. b ukur).
  przeznaczenie_dzialki TEXT,
  dostep_do_drogi TEXT,
  media TEXT,
  uzytki_rolne_m2 INTEGER CHECK (uzytki_rolne_m2 >= 0),

  -- Koszty utrzymania miesięcznie (opłata dla wspólnoty albo spółdzielni, średnie media) i podatek
  -- od nieruchomości za rok.
  oplata_miesieczna INTEGER CHECK (oplata_miesieczna >= 0),
  media_srednio INTEGER CHECK (media_srednio >= 0),
  podatek INTEGER CHECK (podatek >= 0),

  -- Świadectwo charakterystyki energetycznej (art. 13 i 14 uceb): jest, nie sporządzono albo
  -- nie wiadomo. Wskaźniki idą do ogłoszenia, a data końca ważności leży przy dokumencie.
  swiadectwo TEXT NOT NULL CHECK (swiadectwo IN ('jest', 'brak', 'nieznane')),
  swiadectwo_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  wskazniki_energetyczne TEXT,

  -- Kto mieszka albo jest zameldowany w lokalu i do kiedy trwa najem; dane od klienta.
  osoby_w_lokalu TEXT,
  wynajety_do TEXT,
  -- Przeznaczenie lokalu użytkowego według dokumentów, np. usługi albo biuro.
  przeznaczenie_lokalu_uzytkowego TEXT,

  CONSTRAINT swiadectwo_jest_z_dokumentem_i_wskaznikami CHECK (
    (swiadectwo = 'jest') = (swiadectwo_dokument_id IS NOT NULL)
    AND (swiadectwo = 'jest') = (wskazniki_energetyczne IS NOT NULL)
  )
) STRICT;

-- Przeglądanie księgi wieczystej: kto jest wpisany jako właściciel w dziale II i jakie wpisy są
-- w działach III i IV, z datą. Decyzja o publikacji mimo rozbieżności wskazuje ten wiersz, więc
-- ma UUID.
CREATE TABLE oferta_nieruchomosc_przegladania (
  id TEXT PRIMARY KEY,
  nieruchomosc_id TEXT NOT NULL REFERENCES sprawa_nieruchomosci_wpisy (id),
  numer_kw TEXT NOT NULL CHECK (numer_kw GLOB '[A-Z][A-Z][0-9][A-Z]/[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]/[0-9]'),
  przegladano TEXT NOT NULL,
  wlasciciele_dzial_ii TEXT NOT NULL,
  -- Wpisy spisane bez oceny; NULL znaczy, że dział jest pusty.
  wpisy_dzial_iii TEXT,
  wpisy_dzial_iv TEXT,
  -- Rozbieżność z tym, co mówi klient, np. inny właściciel w księdze albo spadkodawca.
  rozbieznosc INTEGER NOT NULL CHECK (rozbieznosc IN (0, 1)),
  rozbieznosc_opis TEXT,
  wydruk_sciezka TEXT,
  wydruk_skrot TEXT,

  CONSTRAINT rozbieznosc_ma_opis CHECK ((rozbieznosc = 1) = (rozbieznosc_opis IS NOT NULL)),
  CONSTRAINT wydruk_ze_skrotem CHECK ((wydruk_sciezka IS NULL) = (wydruk_skrot IS NULL))
) STRICT;

-- Świadectwo, które jest, ma w teczce plik dokumentu swiadectwo_energetyczne z datą końca
-- ważności; od tej daty zależy, czy ogłoszenie może wyjść.
CREATE TRIGGER oferta_nieruchomosc_swiadectwo_ma_dokument
BEFORE INSERT ON oferta_nieruchomosc_opis
WHEN NEW.swiadectwo_dokument_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_dokumenty_wpisy
    WHERE id = NEW.swiadectwo_dokument_id
      AND rodzaj = 'swiadectwo_energetyczne'
      AND stan IN ('otrzymany', 'sprawdzony')
      AND waznosc_do IS NOT NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'świadectwo wskazuje otrzymany dokument swiadectwo_energetyczne z datą końca ważności');
END;

CREATE TRIGGER oferta_nieruchomosc_zmienione_swiadectwo_ma_dokument
BEFORE UPDATE OF swiadectwo_dokument_id ON oferta_nieruchomosc_opis
WHEN NEW.swiadectwo_dokument_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_dokumenty_wpisy
    WHERE id = NEW.swiadectwo_dokument_id
      AND rodzaj = 'swiadectwo_energetyczne'
      AND stan IN ('otrzymany', 'sprawdzony')
      AND waznosc_do IS NOT NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'świadectwo wskazuje otrzymany dokument swiadectwo_energetyczne z datą końca ważności');
END;

-- Dokument świadectwa, na którym stoi opis, nie zmienia rodzaju ani ważności i nie staje się
-- odrzucony ani zbędny; inną wersję świadectwa wpisuje się jako nowy dokument.
CREATE TRIGGER oferta_nieruchomosc_dokument_swiadectwa_zostaje
BEFORE UPDATE ON sprawa_dokumenty_wpisy
WHEN (
    NEW.rodzaj IS NOT OLD.rodzaj
    OR NEW.waznosc_do IS NOT OLD.waznosc_do
    OR (NEW.stan IN ('odrzucony', 'zbedny') AND NEW.stan IS NOT OLD.stan)
  )
  AND EXISTS (
    SELECT 1
    FROM oferta_nieruchomosc_opis
    WHERE swiadectwo_dokument_id = OLD.id
  )
BEGIN
  SELECT RAISE(ABORT, 'dokument świadectwa z opisu nieruchomości nie zmienia rodzaju ani ważności i nie jest odrzucany');
END;

-- Przeglądanie jest zapisem stanu księgi w dniu przeglądania; nowy stan to nowe przeglądanie.
CREATE TRIGGER oferta_nieruchomosc_przegladania_bez_zmian
BEFORE UPDATE ON oferta_nieruchomosc_przegladania
BEGIN
  SELECT RAISE(ABORT, 'przeglądania księgi się nie zmienia; nowy stan księgi to nowe przeglądanie');
END;

-- Opis jednej nieruchomości jest jeden; zmienia się go przez UPDATE.
CREATE TRIGGER oferta_nieruchomosc_opis_niezmienna_nieruchomosc
BEFORE UPDATE OF nieruchomosc_id ON oferta_nieruchomosc_opis
WHEN NEW.nieruchomosc_id IS NOT OLD.nieruchomosc_id
BEGIN
  SELECT RAISE(ABORT, 'opis nie przechodzi na inną nieruchomość');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER oferta_nieruchomosc_opis_bez_zastepowania
BEFORE INSERT ON oferta_nieruchomosc_opis
WHEN EXISTS (
  SELECT 1
  FROM oferta_nieruchomosc_opis
  WHERE nieruchomosc_id = NEW.nieruchomosc_id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_nieruchomosc_opis_bez_usuwania
BEFORE DELETE ON oferta_nieruchomosc_opis
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER oferta_nieruchomosc_przegladania_bez_zastepowania
BEFORE INSERT ON oferta_nieruchomosc_przegladania
WHEN EXISTS (
  SELECT 1
  FROM oferta_nieruchomosc_przegladania
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_nieruchomosc_przegladania_bez_usuwania
BEFORE DELETE ON oferta_nieruchomosc_przegladania
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
    ],
};

/// The values of column `opis.swiadectwo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpisSwiadectwo {
    Jest,
    Brak,
    Nieznane,
}

impl OpisSwiadectwo {
    /// Every value, in the order of the catalog.
    pub const ALL: &[OpisSwiadectwo] = &[
        OpisSwiadectwo::Jest,
        OpisSwiadectwo::Brak,
        OpisSwiadectwo::Nieznane,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            OpisSwiadectwo::Jest => "jest",
            OpisSwiadectwo::Brak => "brak",
            OpisSwiadectwo::Nieznane => "nieznane",
        }
    }
}

impl xmeta_room::FromSqlValue for OpisSwiadectwo {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "jest" => Ok(OpisSwiadectwo::Jest),
                    "brak" => Ok(OpisSwiadectwo::Brak),
                    "nieznane" => Ok(OpisSwiadectwo::Nieznane),
                    other => Err(format!("{other} is no value of OpisSwiadectwo")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for OpisSwiadectwo {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `opisy_nieruchomosci`.
#[derive(Debug, Clone, PartialEq)]
pub struct OpisyNieruchomosciRow {
    pub nieruchomosc_id: String,
    pub liczba_pieter: Option<i64>,
    pub winda: Option<bool>,
    pub rok_budowy: Option<i64>,
    pub material: Option<String>,
    pub ogrzewanie: Option<String>,
    pub stan_lokalu: Option<String>,
    pub remonty: Option<String>,
    pub przeznaczenie_dzialki: Option<String>,
    pub dostep_do_drogi: Option<String>,
    pub media: Option<String>,
    pub uzytki_rolne_m2: Option<i64>,
    pub oplata_miesieczna: Option<i64>,
    pub media_srednio: Option<i64>,
    pub podatek: Option<i64>,
    pub swiadectwo: OpisSwiadectwo,
    pub swiadectwo_dokument_id: Option<String>,
    pub wskazniki_energetyczne: Option<String>,
    pub osoby_w_lokalu: Option<String>,
    pub wynajety_do: Option<String>,
    pub przeznaczenie_lokalu_uzytkowego: Option<String>,
}

impl OpisyNieruchomosciRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(OpisyNieruchomosciRow {
            nieruchomosc_id: row.get("nieruchomosc_id")?,
            liczba_pieter: row.get("liczba_pieter")?,
            winda: row.get("winda")?,
            rok_budowy: row.get("rok_budowy")?,
            material: row.get("material")?,
            ogrzewanie: row.get("ogrzewanie")?,
            stan_lokalu: row.get("stan_lokalu")?,
            remonty: row.get("remonty")?,
            przeznaczenie_dzialki: row.get("przeznaczenie_dzialki")?,
            dostep_do_drogi: row.get("dostep_do_drogi")?,
            media: row.get("media")?,
            uzytki_rolne_m2: row.get("uzytki_rolne_m2")?,
            oplata_miesieczna: row.get("oplata_miesieczna")?,
            media_srednio: row.get("media_srednio")?,
            podatek: row.get("podatek")?,
            swiadectwo: row.get("swiadectwo")?,
            swiadectwo_dokument_id: row.get("swiadectwo_dokument_id")?,
            wskazniki_energetyczne: row.get("wskazniki_energetyczne")?,
            osoby_w_lokalu: row.get("osoby_w_lokalu")?,
            wynajety_do: row.get("wynajety_do")?,
            przeznaczenie_lokalu_uzytkowego: row.get("przeznaczenie_lokalu_uzytkowego")?,
        })
    }
}

/// Opisy nieruchomości oferty: budynek, działka, koszty, świadectwo i osoby w lokalu.
pub fn opisy_nieruchomosci(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<OpisyNieruchomosciRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_nieruchomosc_opis
ORDER BY nieruchomosc_id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(OpisyNieruchomosciRow::from_row).collect()
}

/// The parameters of `wpisz_opis_nieruchomosci`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszOpisNieruchomosciParams {
    pub nieruchomosc_id: String,
    pub liczba_pieter: Option<i64>,
    pub winda: Option<bool>,
    pub rok_budowy: Option<i64>,
    pub material: Option<String>,
    pub ogrzewanie: Option<String>,
    pub stan_lokalu: Option<String>,
    pub remonty: Option<String>,
    pub przeznaczenie_dzialki: Option<String>,
    pub dostep_do_drogi: Option<String>,
    pub media: Option<String>,
    pub uzytki_rolne_m2: Option<i64>,
    pub oplata_miesieczna: Option<i64>,
    pub media_srednio: Option<i64>,
    pub podatek: Option<i64>,
    pub swiadectwo: OpisSwiadectwo,
    pub swiadectwo_dokument_id: Option<String>,
    pub wskazniki_energetyczne: Option<String>,
    pub osoby_w_lokalu: Option<String>,
    pub wynajety_do: Option<String>,
    pub przeznaczenie_lokalu_uzytkowego: Option<String>,
}

/// Opis nieruchomości z rdzenia; źródło każdej danej wpisuje się w sprawa_dokumenty_zrodla.
pub fn wpisz_opis_nieruchomosci(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszOpisNieruchomosciParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO oferta_nieruchomosc_opis (nieruchomosc_id, liczba_pieter, winda, rok_budowy, material,
  ogrzewanie, stan_lokalu, remonty, przeznaczenie_dzialki, dostep_do_drogi, media, uzytki_rolne_m2,
  oplata_miesieczna, media_srednio, podatek, swiadectwo, swiadectwo_dokument_id,
  wskazniki_energetyczne, osoby_w_lokalu, wynajety_do, przeznaczenie_lokalu_uzytkowego)
VALUES (:nieruchomosc_id, :liczba_pieter, :winda, :rok_budowy, :material, :ogrzewanie, :stan_lokalu,
  :remonty, :przeznaczenie_dzialki, :dostep_do_drogi, :media, :uzytki_rolne_m2, :oplata_miesieczna,
  :media_srednio, :podatek, :swiadectwo, :swiadectwo_dokument_id, :wskazniki_energetyczne,
  :osoby_w_lokalu, :wynajety_do, :przeznaczenie_lokalu_uzytkowego)"#,
                parameters: Vec::from([
                    (
                        "nieruchomosc_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.nieruchomosc_id),
                    ),
                    (
                        "liczba_pieter",
                        xmeta_room::ToSqlValue::to_sql_value(&params.liczba_pieter),
                    ),
                    ("winda", xmeta_room::ToSqlValue::to_sql_value(&params.winda)),
                    (
                        "rok_budowy",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rok_budowy),
                    ),
                    ("material", xmeta_room::ToSqlValue::to_sql_value(&params.material)),
                    (
                        "ogrzewanie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.ogrzewanie),
                    ),
                    (
                        "stan_lokalu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.stan_lokalu),
                    ),
                    ("remonty", xmeta_room::ToSqlValue::to_sql_value(&params.remonty)),
                    (
                        "przeznaczenie_dzialki",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.przeznaczenie_dzialki,
                        ),
                    ),
                    (
                        "dostep_do_drogi",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dostep_do_drogi),
                    ),
                    ("media", xmeta_room::ToSqlValue::to_sql_value(&params.media)),
                    (
                        "uzytki_rolne_m2",
                        xmeta_room::ToSqlValue::to_sql_value(&params.uzytki_rolne_m2),
                    ),
                    (
                        "oplata_miesieczna",
                        xmeta_room::ToSqlValue::to_sql_value(&params.oplata_miesieczna),
                    ),
                    (
                        "media_srednio",
                        xmeta_room::ToSqlValue::to_sql_value(&params.media_srednio),
                    ),
                    ("podatek", xmeta_room::ToSqlValue::to_sql_value(&params.podatek)),
                    (
                        "swiadectwo",
                        xmeta_room::ToSqlValue::to_sql_value(&params.swiadectwo),
                    ),
                    (
                        "swiadectwo_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.swiadectwo_dokument_id,
                        ),
                    ),
                    (
                        "wskazniki_energetyczne",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.wskazniki_energetyczne,
                        ),
                    ),
                    (
                        "osoby_w_lokalu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.osoby_w_lokalu),
                    ),
                    (
                        "wynajety_do",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wynajety_do),
                    ),
                    (
                        "przeznaczenie_lokalu_uzytkowego",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.przeznaczenie_lokalu_uzytkowego,
                        ),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_opis_nieruchomosci`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZmienOpisNieruchomosciParams {
    pub liczba_pieter: Option<i64>,
    pub winda: Option<bool>,
    pub rok_budowy: Option<i64>,
    pub material: Option<String>,
    pub ogrzewanie: Option<String>,
    pub stan_lokalu: Option<String>,
    pub remonty: Option<String>,
    pub przeznaczenie_dzialki: Option<String>,
    pub dostep_do_drogi: Option<String>,
    pub media: Option<String>,
    pub uzytki_rolne_m2: Option<i64>,
    pub oplata_miesieczna: Option<i64>,
    pub media_srednio: Option<i64>,
    pub podatek: Option<i64>,
    pub swiadectwo: OpisSwiadectwo,
    pub swiadectwo_dokument_id: Option<String>,
    pub wskazniki_energetyczne: Option<String>,
    pub osoby_w_lokalu: Option<String>,
    pub wynajety_do: Option<String>,
    pub przeznaczenie_lokalu_uzytkowego: Option<String>,
    pub nieruchomosc_id: String,
}

/// Nowy stan opisu, np. po sprawdzeniu danej od klienta z dokumentem albo po nowym świadectwie.
pub fn zmien_opis_nieruchomosci(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienOpisNieruchomosciParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE oferta_nieruchomosc_opis
SET liczba_pieter = :liczba_pieter,
  winda = :winda,
  rok_budowy = :rok_budowy,
  material = :material,
  ogrzewanie = :ogrzewanie,
  stan_lokalu = :stan_lokalu,
  remonty = :remonty,
  przeznaczenie_dzialki = :przeznaczenie_dzialki,
  dostep_do_drogi = :dostep_do_drogi,
  media = :media,
  uzytki_rolne_m2 = :uzytki_rolne_m2,
  oplata_miesieczna = :oplata_miesieczna,
  media_srednio = :media_srednio,
  podatek = :podatek,
  swiadectwo = :swiadectwo,
  swiadectwo_dokument_id = :swiadectwo_dokument_id,
  wskazniki_energetyczne = :wskazniki_energetyczne,
  osoby_w_lokalu = :osoby_w_lokalu,
  wynajety_do = :wynajety_do,
  przeznaczenie_lokalu_uzytkowego = :przeznaczenie_lokalu_uzytkowego
WHERE nieruchomosc_id = :nieruchomosc_id"#,
                parameters: Vec::from([
                    (
                        "liczba_pieter",
                        xmeta_room::ToSqlValue::to_sql_value(&params.liczba_pieter),
                    ),
                    ("winda", xmeta_room::ToSqlValue::to_sql_value(&params.winda)),
                    (
                        "rok_budowy",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rok_budowy),
                    ),
                    ("material", xmeta_room::ToSqlValue::to_sql_value(&params.material)),
                    (
                        "ogrzewanie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.ogrzewanie),
                    ),
                    (
                        "stan_lokalu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.stan_lokalu),
                    ),
                    ("remonty", xmeta_room::ToSqlValue::to_sql_value(&params.remonty)),
                    (
                        "przeznaczenie_dzialki",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.przeznaczenie_dzialki,
                        ),
                    ),
                    (
                        "dostep_do_drogi",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dostep_do_drogi),
                    ),
                    ("media", xmeta_room::ToSqlValue::to_sql_value(&params.media)),
                    (
                        "uzytki_rolne_m2",
                        xmeta_room::ToSqlValue::to_sql_value(&params.uzytki_rolne_m2),
                    ),
                    (
                        "oplata_miesieczna",
                        xmeta_room::ToSqlValue::to_sql_value(&params.oplata_miesieczna),
                    ),
                    (
                        "media_srednio",
                        xmeta_room::ToSqlValue::to_sql_value(&params.media_srednio),
                    ),
                    ("podatek", xmeta_room::ToSqlValue::to_sql_value(&params.podatek)),
                    (
                        "swiadectwo",
                        xmeta_room::ToSqlValue::to_sql_value(&params.swiadectwo),
                    ),
                    (
                        "swiadectwo_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.swiadectwo_dokument_id,
                        ),
                    ),
                    (
                        "wskazniki_energetyczne",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.wskazniki_energetyczne,
                        ),
                    ),
                    (
                        "osoby_w_lokalu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.osoby_w_lokalu),
                    ),
                    (
                        "wynajety_do",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wynajety_do),
                    ),
                    (
                        "przeznaczenie_lokalu_uzytkowego",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.przeznaczenie_lokalu_uzytkowego,
                        ),
                    ),
                    (
                        "nieruchomosc_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.nieruchomosc_id),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `przegladania_ksiag`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzegladaniaKsiagRow {
    pub id: String,
    pub nieruchomosc_id: String,
    pub numer_kw: String,
    pub przegladano: String,
    pub wlasciciele_dzial_ii: String,
    pub wpisy_dzial_iii: Option<String>,
    pub wpisy_dzial_iv: Option<String>,
    pub rozbieznosc: bool,
    pub rozbieznosc_opis: Option<String>,
    pub wydruk_sciezka: Option<String>,
    pub wydruk_skrot: Option<String>,
}

impl PrzegladaniaKsiagRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PrzegladaniaKsiagRow {
            id: row.get("id")?,
            nieruchomosc_id: row.get("nieruchomosc_id")?,
            numer_kw: row.get("numer_kw")?,
            przegladano: row.get("przegladano")?,
            wlasciciele_dzial_ii: row.get("wlasciciele_dzial_ii")?,
            wpisy_dzial_iii: row.get("wpisy_dzial_iii")?,
            wpisy_dzial_iv: row.get("wpisy_dzial_iv")?,
            rozbieznosc: row.get("rozbieznosc")?,
            rozbieznosc_opis: row.get("rozbieznosc_opis")?,
            wydruk_sciezka: row.get("wydruk_sciezka")?,
            wydruk_skrot: row.get("wydruk_skrot")?,
        })
    }
}

/// Przeglądania ksiąg wieczystych, najnowsze pierwsze.
pub fn przegladania_ksiag(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PrzegladaniaKsiagRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_nieruchomosc_przegladania
ORDER BY przegladano DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(PrzegladaniaKsiagRow::from_row).collect()
}

/// The parameters of `wpisz_przegladanie_ksiegi`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszPrzegladanieKsiegiParams {
    pub id: String,
    pub nieruchomosc_id: String,
    pub numer_kw: String,
    pub przegladano: String,
    pub wlasciciele_dzial_ii: String,
    pub wpisy_dzial_iii: Option<String>,
    pub wpisy_dzial_iv: Option<String>,
    pub rozbieznosc: bool,
    pub rozbieznosc_opis: Option<String>,
    pub wydruk_sciezka: Option<String>,
    pub wydruk_skrot: Option<String>,
}

/// Wynik przeglądania księgi z datą: właściciele z działu II, wpisy z działów III i IV
/// i rozbieżność z tym, co mówi klient. Identyfikator (UUID) nadaje aplikacja, bo wskazuje go
/// decyzja o publikacji mimo rozbieżności.
pub fn wpisz_przegladanie_ksiegi(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszPrzegladanieKsiegiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO oferta_nieruchomosc_przegladania (id, nieruchomosc_id, numer_kw, przegladano,
  wlasciciele_dzial_ii, wpisy_dzial_iii, wpisy_dzial_iv, rozbieznosc, rozbieznosc_opis,
  wydruk_sciezka, wydruk_skrot)
VALUES (:id, :nieruchomosc_id, :numer_kw, :przegladano, :wlasciciele_dzial_ii, :wpisy_dzial_iii,
  :wpisy_dzial_iv, :rozbieznosc, :rozbieznosc_opis, :wydruk_sciezka, :wydruk_skrot)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "nieruchomosc_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.nieruchomosc_id),
                    ),
                    ("numer_kw", xmeta_room::ToSqlValue::to_sql_value(&params.numer_kw)),
                    (
                        "przegladano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przegladano),
                    ),
                    (
                        "wlasciciele_dzial_ii",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.wlasciciele_dzial_ii,
                        ),
                    ),
                    (
                        "wpisy_dzial_iii",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wpisy_dzial_iii),
                    ),
                    (
                        "wpisy_dzial_iv",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wpisy_dzial_iv),
                    ),
                    (
                        "rozbieznosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rozbieznosc),
                    ),
                    (
                        "rozbieznosc_opis",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rozbieznosc_opis),
                    ),
                    (
                        "wydruk_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wydruk_sciezka),
                    ),
                    (
                        "wydruk_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wydruk_skrot),
                    ),
                ]),
            },
        ],
    )
}
