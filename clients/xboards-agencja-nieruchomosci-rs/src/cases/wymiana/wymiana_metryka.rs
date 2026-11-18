//! The kind `wymiana-metryka` of the room `cases/wymiana`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "wymiana-metryka",
    requires: &[],
    migrations: &[
        r#"-- Metryka pokoju wymiany: jeden wiersz na pokój, strony aktu i dziennik zamknięć.
-- Pokój wymiany jest wspólny dla biura na XEstates i kancelarii notarialnej na XNotariuszu,
-- jeden na transakcję i kancelarię. Metrykę zakłada biuro, gdy człowiek, który przyjął
-- transakcję, zgodzi się na otwarcie pokoju z kancelarią wybraną przez strony. Pokój nie ma
-- rdzenia teczki: teczka transakcji zostaje w biurze, a pokój wskazuje ją tekstem.
--
-- Kolumna strona_pokoju jest deklaracją piszącego. Organizację autora poświadcza serwer pokoju,
-- a porównuje ją aplikacja (decyzja T10); baza tego nie umie.
CREATE TABLE wymiana_metryka_wpis (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  -- Oznaczenie transakcji w biurze, np. „Transakcja 2026/017, ul. Polna 7 m. 12”.
  oznaczenie TEXT NOT NULL CHECK (oznaczenie <> ''),
  -- Teczka transakcji (albo najmu przy oświadczeniu najemcy) w biurze: identyfikator pokoju.
  transakcja_odnosnik TEXT NOT NULL CHECK (transakcja_odnosnik <> ''),
  -- Czynność, dla której biuro otworzyło pokój. Kolejne czynności tej transakcji u tej samej
  -- kancelarii, np. umowa przenosząca po warunkowej, wskazują swoje terminy aktu.
  czynnosc TEXT NOT NULL CHECK (
    czynnosc IN ('przedwstepna_akt', 'sprzedaz_warunkowa', 'przenoszaca', 'oswiadczenie_najemcy')
  ),
  kancelaria_nazwa TEXT NOT NULL CHECK (kancelaria_nazwa <> ''),
  kancelaria_adres TEXT NOT NULL CHECK (kancelaria_adres <> ''),
  -- Imię i nazwisko notariusza.
  notariusz TEXT NOT NULL CHECK (notariusz <> ''),
  -- Sygnatura sprawy w kancelarii; podaje ją kancelaria, gdy zechce.
  sygnatura_kancelarii TEXT CHECK (sygnatura_kancelarii <> ''),
  nieruchomosc_adres TEXT NOT NULL CHECK (nieruchomosc_adres <> ''),
  -- Lokal bez księgi wieczystej, np. przy oświadczeniu najemcy, nie ma numeru.
  numer_kw TEXT CHECK (numer_kw <> ''),
  otwarto TEXT NOT NULL
) STRICT;

-- Strony aktu: kto zawiera umowę albo składa oświadczenie, i czy biuro ją reprezentuje.
-- Biuro pisze w imieniu strony tylko z jej pełnomocnictwem (wymiana-dokumenty).
CREATE TABLE wymiana_metryka_strony (
  id TEXT PRIMARY KEY,
  -- Imiona i nazwisko albo nazwa strony.
  nazwa TEXT NOT NULL CHECK (nazwa <> ''),
  rola TEXT NOT NULL CHECK (rola IN ('sprzedajacy', 'kupujacy', 'wynajmujacy', 'najemca', 'inna')),
  reprezentowana_przez_biuro INTEGER NOT NULL CHECK (reprezentowana_przez_biuro IN (0, 1))
) STRICT;

-- Dziennik zamknięć: pokój zamyka się dopiero, gdy zamkną go obie strony, i otwiera ponownie
-- dopiero, gdy obie strony go otworzą (profil pokoju, rozdział „Cykl istnienia”).
CREATE TABLE wymiana_metryka_zamkniecia (
  -- Klucz nadany przez bazę jest dodatni; zob. wymiana_metryka_zamkniecia_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  strona_pokoju TEXT NOT NULL CHECK (strona_pokoju IN ('biuro', 'kancelaria')),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('zamkniecie', 'ponowne_otwarcie')),
  chwila TEXT NOT NULL
) STRICT;

-- Stan pokoju. Pełne zamknięcie to zamknięcie obu stron, a pełne otwarcie to ponowne otwarcie
-- obu stron. Każda strona zamyka i otwiera na przemian, zamyka tylko otwarty pokój i otwiera
-- tylko zamknięty, więc pokój jest zamknięty, gdy pełnych zamknięć jest więcej niż pełnych
-- otwarć. Jedno zamknięcie strony zostawia pokój otwarty, a jedno ponowne otwarcie — zamknięty.
CREATE VIEW wymiana_metryka_stan AS
SELECT
  CASE
    WHEN min(zamkniecia_biura, zamkniecia_kancelarii) > min(otwarcia_biura, otwarcia_kancelarii)
      THEN 'zamkniety'
    ELSE 'otwarty'
  END AS stan
FROM (
  SELECT
    count(CASE WHEN strona_pokoju = 'biuro' AND rodzaj = 'zamkniecie' THEN 1 END) AS zamkniecia_biura,
    count(CASE WHEN strona_pokoju = 'kancelaria' AND rodzaj = 'zamkniecie' THEN 1 END) AS zamkniecia_kancelarii,
    count(CASE WHEN strona_pokoju = 'biuro' AND rodzaj = 'ponowne_otwarcie' THEN 1 END) AS otwarcia_biura,
    count(CASE WHEN strona_pokoju = 'kancelaria' AND rodzaj = 'ponowne_otwarcie' THEN 1 END) AS otwarcia_kancelarii
  FROM wymiana_metryka_zamkniecia
);

-- Metryka mówi kancelarii, z kim i o jakiej transakcji rozmawia; po niej człowiek po stronie
-- biura rozpoznaje kancelarię. Zmienia się tylko raz: kancelaria podaje swoją sygnaturę.
CREATE TRIGGER wymiana_metryka_wpis_zmienia_tylko_sygnature
BEFORE UPDATE ON wymiana_metryka_wpis
WHEN OLD.sygnatura_kancelarii IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.oznaczenie IS NOT OLD.oznaczenie
  OR NEW.transakcja_odnosnik IS NOT OLD.transakcja_odnosnik
  OR NEW.czynnosc IS NOT OLD.czynnosc
  OR NEW.kancelaria_nazwa IS NOT OLD.kancelaria_nazwa
  OR NEW.kancelaria_adres IS NOT OLD.kancelaria_adres
  OR NEW.notariusz IS NOT OLD.notariusz
  OR NEW.nieruchomosc_adres IS NOT OLD.nieruchomosc_adres
  OR NEW.numer_kw IS NOT OLD.numer_kw
  OR NEW.otwarto IS NOT OLD.otwarto
BEGIN
  SELECT RAISE(ABORT, 'metryki pokoju się nie zmienia; kancelaria raz podaje tylko swoją sygnaturę');
END;

CREATE TRIGGER wymiana_metryka_zamkniecie_po_otwarciu
BEFORE INSERT ON wymiana_metryka_zamkniecia
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_metryka_wpis
)
BEGIN
  SELECT RAISE(ABORT, 'pokój bez metryki nie ma czego zamykać');
END;

-- Strona zamyka i otwiera na przemian, a zaczyna od zamknięcia.
CREATE TRIGGER wymiana_metryka_zamkniecia_na_przemian
BEFORE INSERT ON wymiana_metryka_zamkniecia
WHEN NEW.rodzaj = coalesce(
  (
    SELECT rodzaj
    FROM wymiana_metryka_zamkniecia
    WHERE strona_pokoju = NEW.strona_pokoju
    ORDER BY id DESC
    LIMIT 1
  ),
  'ponowne_otwarcie'
)
BEGIN
  SELECT RAISE(ABORT, 'strona pokoju zamyka i otwiera go na przemian, zaczynając od zamknięcia');
END;

-- Ponowne otwarcie wymaga zgody obu stron, więc strona, która już otworzyła zamknięty pokój,
-- czeka na drugą i nie zamyka go od nowa. Zamknięcie, na które druga strona nie odpowiada,
-- zostaje; o tym, czego jeszcze trzeba, strony piszą w wiadomościach.
CREATE TRIGGER wymiana_metryka_zamyka_otwarty_otwiera_zamkniety
BEFORE INSERT ON wymiana_metryka_zamkniecia
WHEN (
  SELECT stan
  FROM wymiana_metryka_stan
) = CASE NEW.rodzaj WHEN 'zamkniecie' THEN 'zamkniety' ELSE 'otwarty' END
BEGIN
  SELECT RAISE(ABORT, 'strona zamyka tylko otwarty pokój, a ponownie otwiera tylko zamknięty');
END;

CREATE TRIGGER wymiana_metryka_strony_bez_zmian
BEFORE UPDATE ON wymiana_metryka_strony
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

CREATE TRIGGER wymiana_metryka_zamkniecia_bez_zmian
BEFORE UPDATE ON wymiana_metryka_zamkniecia
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_metryka_wpis_bez_zastepowania
BEFORE INSERT ON wymiana_metryka_wpis
WHEN EXISTS (
  SELECT 1
  FROM wymiana_metryka_wpis
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_metryka_wpis_bez_usuwania
BEFORE DELETE ON wymiana_metryka_wpis
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_metryka_strony_bez_zastepowania
BEFORE INSERT ON wymiana_metryka_strony
WHEN EXISTS (
  SELECT 1
  FROM wymiana_metryka_strony
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_metryka_strony_bez_usuwania
BEFORE DELETE ON wymiana_metryka_strony
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER wymiana_metryka_zamkniecia_bez_zastepowania
BEFORE INSERT ON wymiana_metryka_zamkniecia
WHEN EXISTS (
  SELECT 1
  FROM wymiana_metryka_zamkniecia
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_metryka_zamkniecia_bez_usuwania
BEFORE DELETE ON wymiana_metryka_zamkniecia
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;
"#,
    ],
};

/// The values of column `strony.rola`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StronyRola {
    Sprzedajacy,
    Kupujacy,
    Wynajmujacy,
    Najemca,
    Inna,
}

impl StronyRola {
    /// Every value, in the order of the catalog.
    pub const ALL: &[StronyRola] = &[
        StronyRola::Sprzedajacy,
        StronyRola::Kupujacy,
        StronyRola::Wynajmujacy,
        StronyRola::Najemca,
        StronyRola::Inna,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            StronyRola::Sprzedajacy => "sprzedajacy",
            StronyRola::Kupujacy => "kupujacy",
            StronyRola::Wynajmujacy => "wynajmujacy",
            StronyRola::Najemca => "najemca",
            StronyRola::Inna => "inna",
        }
    }
}

impl xmeta_room::FromSqlValue for StronyRola {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "sprzedajacy" => Ok(StronyRola::Sprzedajacy),
                    "kupujacy" => Ok(StronyRola::Kupujacy),
                    "wynajmujacy" => Ok(StronyRola::Wynajmujacy),
                    "najemca" => Ok(StronyRola::Najemca),
                    "inna" => Ok(StronyRola::Inna),
                    other => Err(format!("{other} is no value of StronyRola")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for StronyRola {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpis.czynnosc`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisCzynnosc {
    PrzedwstepnaAkt,
    SprzedazWarunkowa,
    Przenoszaca,
    OswiadczenieNajemcy,
}

impl WpisCzynnosc {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisCzynnosc] = &[
        WpisCzynnosc::PrzedwstepnaAkt,
        WpisCzynnosc::SprzedazWarunkowa,
        WpisCzynnosc::Przenoszaca,
        WpisCzynnosc::OswiadczenieNajemcy,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisCzynnosc::PrzedwstepnaAkt => "przedwstepna_akt",
            WpisCzynnosc::SprzedazWarunkowa => "sprzedaz_warunkowa",
            WpisCzynnosc::Przenoszaca => "przenoszaca",
            WpisCzynnosc::OswiadczenieNajemcy => "oswiadczenie_najemcy",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisCzynnosc {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "przedwstepna_akt" => Ok(WpisCzynnosc::PrzedwstepnaAkt),
                    "sprzedaz_warunkowa" => Ok(WpisCzynnosc::SprzedazWarunkowa),
                    "przenoszaca" => Ok(WpisCzynnosc::Przenoszaca),
                    "oswiadczenie_najemcy" => Ok(WpisCzynnosc::OswiadczenieNajemcy),
                    other => Err(format!("{other} is no value of WpisCzynnosc")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisCzynnosc {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `zamkniecia.strona_pokoju`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZamknieciaStronaPokoju {
    Biuro,
    Kancelaria,
}

impl ZamknieciaStronaPokoju {
    /// Every value, in the order of the catalog.
    pub const ALL: &[ZamknieciaStronaPokoju] = &[
        ZamknieciaStronaPokoju::Biuro,
        ZamknieciaStronaPokoju::Kancelaria,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            ZamknieciaStronaPokoju::Biuro => "biuro",
            ZamknieciaStronaPokoju::Kancelaria => "kancelaria",
        }
    }
}

impl xmeta_room::FromSqlValue for ZamknieciaStronaPokoju {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "biuro" => Ok(ZamknieciaStronaPokoju::Biuro),
                    "kancelaria" => Ok(ZamknieciaStronaPokoju::Kancelaria),
                    other => {
                        Err(format!("{other} is no value of ZamknieciaStronaPokoju"))
                    }
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for ZamknieciaStronaPokoju {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `zamkniecia.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZamknieciaRodzaj {
    Zamkniecie,
    PonowneOtwarcie,
}

impl ZamknieciaRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[ZamknieciaRodzaj] = &[
        ZamknieciaRodzaj::Zamkniecie,
        ZamknieciaRodzaj::PonowneOtwarcie,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            ZamknieciaRodzaj::Zamkniecie => "zamkniecie",
            ZamknieciaRodzaj::PonowneOtwarcie => "ponowne_otwarcie",
        }
    }
}

impl xmeta_room::FromSqlValue for ZamknieciaRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "zamkniecie" => Ok(ZamknieciaRodzaj::Zamkniecie),
                    "ponowne_otwarcie" => Ok(ZamknieciaRodzaj::PonowneOtwarcie),
                    other => Err(format!("{other} is no value of ZamknieciaRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for ZamknieciaRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of field `stan` of `stan_pokoju`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stan {
    Otwarty,
    Zamkniety,
}

impl Stan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[Stan] = &[Stan::Otwarty, Stan::Zamkniety];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            Stan::Otwarty => "otwarty",
            Stan::Zamkniety => "zamkniety",
        }
    }
}

impl xmeta_room::FromSqlValue for Stan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "otwarty" => Ok(Stan::Otwarty),
                    "zamkniety" => Ok(Stan::Zamkniety),
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

/// A row of `metryka`.
#[derive(Debug, Clone, PartialEq)]
pub struct MetrykaRow {
    pub id: i64,
    pub oznaczenie: String,
    pub transakcja_odnosnik: String,
    pub czynnosc: WpisCzynnosc,
    pub kancelaria_nazwa: String,
    pub kancelaria_adres: String,
    pub notariusz: String,
    pub sygnatura_kancelarii: Option<String>,
    pub nieruchomosc_adres: String,
    pub numer_kw: Option<String>,
    pub otwarto: String,
}

impl MetrykaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(MetrykaRow {
            id: row.get("id")?,
            oznaczenie: row.get("oznaczenie")?,
            transakcja_odnosnik: row.get("transakcja_odnosnik")?,
            czynnosc: row.get("czynnosc")?,
            kancelaria_nazwa: row.get("kancelaria_nazwa")?,
            kancelaria_adres: row.get("kancelaria_adres")?,
            notariusz: row.get("notariusz")?,
            sygnatura_kancelarii: row.get("sygnatura_kancelarii")?,
            nieruchomosc_adres: row.get("nieruchomosc_adres")?,
            numer_kw: row.get("numer_kw")?,
            otwarto: row.get("otwarto")?,
        })
    }
}

/// Metryka pokoju: transakcja, czynność, kancelaria, notariusz i nieruchomość.
pub fn metryka(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Option<MetrykaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM wymiana_metryka_wpis
WHERE id = 1"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.first().map(MetrykaRow::from_row).transpose()
}

/// The parameters of `otworz_pokoj_wymiany`.
#[derive(Debug, Clone, PartialEq)]
pub struct OtworzPokojWymianyParams {
    pub oznaczenie: String,
    pub transakcja_odnosnik: String,
    pub czynnosc: WpisCzynnosc,
    pub kancelaria_nazwa: String,
    pub kancelaria_adres: String,
    pub notariusz: String,
    pub nieruchomosc_adres: String,
    pub numer_kw: Option<String>,
    pub otwarto: String,
}

/// Biuro zakłada pokój za zgodą człowieka, który przyjął transakcję, i zaprasza kancelarię.
pub fn otworz_pokoj_wymiany(
    room: &impl xmeta_room::MetaRoom,
    params: &OtworzPokojWymianyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_metryka_wpis (id, oznaczenie, transakcja_odnosnik, czynnosc, kancelaria_nazwa,
  kancelaria_adres, notariusz, nieruchomosc_adres, numer_kw, otwarto)
VALUES (1, :oznaczenie, :transakcja_odnosnik, :czynnosc, :kancelaria_nazwa, :kancelaria_adres,
  :notariusz, :nieruchomosc_adres, :numer_kw, :otwarto)"#,
                parameters: Vec::from([
                    (
                        "oznaczenie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.oznaczenie),
                    ),
                    (
                        "transakcja_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.transakcja_odnosnik),
                    ),
                    ("czynnosc", xmeta_room::ToSqlValue::to_sql_value(&params.czynnosc)),
                    (
                        "kancelaria_nazwa",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kancelaria_nazwa),
                    ),
                    (
                        "kancelaria_adres",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kancelaria_adres),
                    ),
                    (
                        "notariusz",
                        xmeta_room::ToSqlValue::to_sql_value(&params.notariusz),
                    ),
                    (
                        "nieruchomosc_adres",
                        xmeta_room::ToSqlValue::to_sql_value(&params.nieruchomosc_adres),
                    ),
                    ("numer_kw", xmeta_room::ToSqlValue::to_sql_value(&params.numer_kw)),
                    ("otwarto", xmeta_room::ToSqlValue::to_sql_value(&params.otwarto)),
                ]),
            },
        ],
    )
}

/// The parameters of `podaj_sygnature_kancelarii`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PodajSygnatureKancelariiParams {
    pub sygnatura_kancelarii: String,
}

/// Kancelaria podaje sygnaturę swojej sprawy; robi to raz.
pub fn podaj_sygnature_kancelarii(
    room: &impl xmeta_room::MetaRoom,
    params: &PodajSygnatureKancelariiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE wymiana_metryka_wpis
SET sygnatura_kancelarii = :sygnatura_kancelarii
WHERE id = 1"#,
                parameters: Vec::from([
                    (
                        "sygnatura_kancelarii",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.sygnatura_kancelarii,
                        ),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `strony_aktu`.
#[derive(Debug, Clone, PartialEq)]
pub struct StronyAktuRow {
    pub id: String,
    pub nazwa: String,
    pub rola: StronyRola,
    pub reprezentowana_przez_biuro: bool,
}

impl StronyAktuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(StronyAktuRow {
            id: row.get("id")?,
            nazwa: row.get("nazwa")?,
            rola: row.get("rola")?,
            reprezentowana_przez_biuro: row.get("reprezentowana_przez_biuro")?,
        })
    }
}

/// Strony aktu w kolejności wpisania, z tym, które reprezentuje biuro.
pub fn strony_aktu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<StronyAktuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM wymiana_metryka_strony
ORDER BY rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(StronyAktuRow::from_row).collect()
}

/// The parameters of `wpisz_strone_aktu`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszStroneAktuParams {
    pub id: String,
    pub nazwa: String,
    pub rola: StronyRola,
    pub reprezentowana_przez_biuro: bool,
}

/// Biuro wpisuje stronę aktu przy otwarciu pokoju albo gdy strona dojdzie później.
pub fn wpisz_strone_aktu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszStroneAktuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_metryka_strony (id, nazwa, rola, reprezentowana_przez_biuro)
VALUES (:id, :nazwa, :rola, :reprezentowana_przez_biuro)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("nazwa", xmeta_room::ToSqlValue::to_sql_value(&params.nazwa)),
                    ("rola", xmeta_room::ToSqlValue::to_sql_value(&params.rola)),
                    (
                        "reprezentowana_przez_biuro",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.reprezentowana_przez_biuro,
                        ),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `stan_pokoju`.
#[derive(Debug, Clone, PartialEq)]
pub struct StanPokojuRow {
    pub stan: Stan,
}

impl StanPokojuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(StanPokojuRow {
            stan: row.get("stan")?,
        })
    }
}

/// Czy pokój jest otwarty, czy zamknięty; zamyka go i otwiera ponownie zgoda obu stron.
pub fn stan_pokoju(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Option<StanPokojuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT stan
FROM wymiana_metryka_stan"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.first().map(StanPokojuRow::from_row).transpose()
}

/// A row of `zamkniecia`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZamknieciaRow {
    pub id: i64,
    pub strona_pokoju: ZamknieciaStronaPokoju,
    pub rodzaj: ZamknieciaRodzaj,
    pub chwila: String,
}

impl ZamknieciaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZamknieciaRow {
            id: row.get("id")?,
            strona_pokoju: row.get("strona_pokoju")?,
            rodzaj: row.get("rodzaj")?,
            chwila: row.get("chwila")?,
        })
    }
}

/// Dziennik zamknięć i ponownych otwarć obu stron, z datami.
pub fn zamkniecia(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZamknieciaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM wymiana_metryka_zamkniecia
ORDER BY id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZamknieciaRow::from_row).collect()
}

/// The parameters of `zamknij_pokoj_wymiany`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZamknijPokojWymianyParams {
    pub strona_pokoju: ZamknieciaStronaPokoju,
    pub chwila: String,
}

/// Strona zgadza się na zamknięcie pokoju; pokój zamyka się, gdy zgodzą się obie.
pub fn zamknij_pokoj_wymiany(
    room: &impl xmeta_room::MetaRoom,
    params: &ZamknijPokojWymianyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_metryka_zamkniecia (strona_pokoju, rodzaj, chwila)
VALUES (:strona_pokoju, 'zamkniecie', :chwila)"#,
                parameters: Vec::from([
                    (
                        "strona_pokoju",
                        xmeta_room::ToSqlValue::to_sql_value(&params.strona_pokoju),
                    ),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                ]),
            },
        ],
    )
}

/// The parameters of `otworz_ponownie_pokoj_wymiany`.
#[derive(Debug, Clone, PartialEq)]
pub struct OtworzPonowniePokojWymianyParams {
    pub strona_pokoju: ZamknieciaStronaPokoju,
    pub chwila: String,
}

/// Strona zgadza się na ponowne otwarcie zamkniętego pokoju; otwiera się, gdy zgodzą się obie.
pub fn otworz_ponownie_pokoj_wymiany(
    room: &impl xmeta_room::MetaRoom,
    params: &OtworzPonowniePokojWymianyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_metryka_zamkniecia (strona_pokoju, rodzaj, chwila)
VALUES (:strona_pokoju, 'ponowne_otwarcie', :chwila)"#,
                parameters: Vec::from([
                    (
                        "strona_pokoju",
                        xmeta_room::ToSqlValue::to_sql_value(&params.strona_pokoju),
                    ),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                ]),
            },
        ],
    )
}
