//! The kind `poszukiwanie-kredyt` of the room `cases/poszukiwanie`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "poszukiwanie-kredyt",
    requires: &[
        &super::sprawa_decyzje::KIND,
        &super::sprawa_dokumenty::KIND,
        &super::sprawa_korespondencja::KIND,
    ],
    migrations: &[
        r#"-- Analiza kredytowa klienta: dane wejściowe, wyniki programu, bank wybrany przez klienta
-- i harmonogram do wypłaty kredytu. Pisze Analityk kredytowy. Kwoty w groszach.
--
-- Analiza jest orientacyjna: nie jest ofertą ani rekomendacją kredytu i mówi to wprost (R2).
-- Zdolność liczy program biura, a Analityk dobiera dane i tłumaczy wynik; wyniku nie poprawia.
-- Do programu idą tylko dane z dokumentów, a deklaracja klienta zostaje w teczce jako dana
-- niepotwierdzona. Analiza poza standardem działu (np. inny okres kredytowania) wymaga
-- zatwierdzonej decyzji „inna”, która ją wskazuje. Nowa analiza odbiera aktualność poprzednim.
--
-- Listę dokumentów dla banku z ważnością trzyma rdzeń: dokumenty z wymaga = 'bank'. Banku nie
-- poleca żaden agent (R2); bank wybiera klient, a teczka zapisuje, skąd to wiadomo.

-- Dana wejściowa: dochód, zobowiązanie, gospodarstwo, wkład własny, cena albo okres kredytowania,
-- z dokumentem albo jako deklaracja klienta.
CREATE TABLE poszukiwanie_kredyt_dane (
  id TEXT PRIMARY KEY,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('dochod', 'zobowiazanie', 'gospodarstwo', 'wklad_wlasny', 'cena', 'okres')),
  -- Np. „umowa o pracę, netto” albo „rata za samochód”.
  opis TEXT NOT NULL,
  -- Kwota w groszach (dochód i zobowiązanie miesięcznie), liczba osób gospodarstwa albo liczba lat
  -- okresu kredytowania.
  wartosc INTEGER NOT NULL CHECK (wartosc >= 0),
  -- Okres, z którego wzięto kwotę, np. „średnia z 6 miesięcy”.
  okres_zrodla TEXT,
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  deklaracja INTEGER NOT NULL CHECK (deklaracja IN (0, 1)),
  wpisano TEXT NOT NULL,

  CONSTRAINT dana_z_dokumentu_albo_deklaracja CHECK ((deklaracja = 1) = (dokument_id IS NULL))
) STRICT;

-- Analiza z wersją parametrów programu, okresem, wynikiem i wyjaśnieniem dla klienta.
CREATE TABLE poszukiwanie_kredyt_analizy (
  id TEXT PRIMARY KEY,
  -- Wersja parametrów programu ze standardu działu, którą zatwierdził człowiek.
  wersja_parametrow TEXT NOT NULL,
  okres_lat INTEGER NOT NULL CHECK (okres_lat > 0),
  poza_standardem INTEGER NOT NULL CHECK (poza_standardem IN (0, 1)),
  poza_standardem_decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),
  -- Orientacyjny zakres kwoty kredytu i raty oraz udział wkładu własnego w cenie w setnych
  -- procenta (18 % to 1800); wszystko z programu.
  kwota_od INTEGER NOT NULL CHECK (kwota_od >= 0),
  kwota_do INTEGER NOT NULL CHECK (kwota_do >= 0),
  rata_od INTEGER NOT NULL CHECK (rata_od >= 0),
  rata_do INTEGER NOT NULL CHECK (rata_do >= 0),
  udzial_wkladu INTEGER NOT NULL CHECK (udzial_wkladu BETWEEN 0 AND 10000),
  wynik_sciezka TEXT NOT NULL,
  wynik_skrot TEXT NOT NULL,
  wyjasnienie TEXT NOT NULL,
  -- Zdanie, że analiza nie jest ofertą ani rekomendacją kredytu (R2).
  zastrzezenie TEXT NOT NULL CHECK (zastrzezenie <> ''),
  aktualna INTEGER NOT NULL CHECK (aktualna IN (0, 1)),
  utworzono TEXT NOT NULL,

  CONSTRAINT poza_standardem_z_decyzja CHECK ((poza_standardem = 1) = (poza_standardem_decyzja_id IS NOT NULL)),
  CONSTRAINT kwota_od_do CHECK (kwota_od <= kwota_do),
  CONSTRAINT rata_od_do CHECK (rata_od <= rata_do)
) STRICT;

-- Dane, które Analityk wprowadził do programu w tej analizie.
CREATE TABLE poszukiwanie_kredyt_analizy_dane (
  -- Klucz nadany przez bazę jest dodatni; zob. poszukiwanie_kredyt_analizy_dane_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  analiza_id TEXT NOT NULL REFERENCES poszukiwanie_kredyt_analizy (id),
  dana_id TEXT NOT NULL REFERENCES poszukiwanie_kredyt_dane (id),

  UNIQUE (analiza_id, dana_id)
) STRICT;

-- Bank, który wybrał klient, z wpływem, z którego to wiadomo, i jego wymaganiami wobec
-- dokumentów ze źródłem i datą.
CREATE TABLE poszukiwanie_kredyt_banki (
  -- Klucz nadany przez bazę jest dodatni; zob. poszukiwanie_kredyt_banki_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  bank TEXT NOT NULL,
  wybrano TEXT NOT NULL,
  zrodlo_wplyw_id TEXT NOT NULL REFERENCES sprawa_korespondencja_wplywy (id),
  -- Np. strona banku albo pismo banku do klienta.
  wymagania_zrodlo TEXT,
  wymagania_data TEXT,

  CONSTRAINT wymagania_ze_zrodlem_i_data CHECK ((wymagania_zrodlo IS NULL) = (wymagania_data IS NULL))
) STRICT;

-- Droga do wypłaty kredytu: data przewidywana z programu i data rzeczywista po fakcie.
CREATE TABLE poszukiwanie_kredyt_harmonogram (
  -- Klucz nadany przez bazę jest dodatni; zob. poszukiwanie_kredyt_harmonogram_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  krok TEXT NOT NULL CHECK (krok IN ('wniosek', 'wycena', 'decyzja_kredytowa', 'umowa_kredytu', 'wyplata')),
  przewidywana TEXT NOT NULL,
  rzeczywista TEXT
) STRICT;

-- Dana z dokumentu wskazuje dokument z plikiem.
CREATE TRIGGER poszukiwanie_kredyt_dana_z_plikiem
BEFORE INSERT ON poszukiwanie_kredyt_dane
WHEN NEW.dokument_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_dokumenty_wpisy
    WHERE id = NEW.dokument_id
      AND stan IN ('otrzymany', 'sprawdzony')
  )
BEGIN
  SELECT RAISE(ABORT, 'dana wejściowa wskazuje dokument z plikiem: otrzymany albo sprawdzony');
END;

CREATE TRIGGER poszukiwanie_kredyt_dane_bez_zmian
BEFORE UPDATE ON poszukiwanie_kredyt_dane
BEGIN
  SELECT RAISE(ABORT, 'danej wejściowej się nie zmienia; nowa wartość to nowy wpis');
END;

-- Analiza poza standardem stoi na zatwierdzonej decyzji „inna”, która ją wskazuje.
CREATE TRIGGER poszukiwanie_kredyt_poza_standardem_za_zgoda
BEFORE INSERT ON poszukiwanie_kredyt_analizy
WHEN NEW.poza_standardem = 1
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS decyzja
    WHERE decyzja.id = NEW.poza_standardem_decyzja_id
      AND decyzja.rodzaj = 'inna'
      AND decyzja.dotyczy_tabela = 'poszukiwanie_kredyt_analizy'
      AND decyzja.dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'analiza poza standardem wymaga zatwierdzonej decyzji inna, która ją wskazuje');
END;

-- Aktualność odbiera tylko nowa analiza, więc analiza powstaje aktualna.
CREATE TRIGGER poszukiwanie_kredyt_analiza_powstaje_aktualna
BEFORE INSERT ON poszukiwanie_kredyt_analizy
WHEN NEW.aktualna <> 1
BEGIN
  SELECT RAISE(ABORT, 'analiza powstaje jako aktualna');
END;

CREATE TRIGGER poszukiwanie_kredyt_nowa_analiza_zeruje_poprzednie
AFTER INSERT ON poszukiwanie_kredyt_analizy
BEGIN
  UPDATE poszukiwanie_kredyt_analizy
  SET aktualna = 0
  WHERE id <> NEW.id
    AND aktualna = 1;
END;

-- Wynik pochodzi z programu i Analityk go nie poprawia; jedyną zmianą jest utrata aktualności.
CREATE TRIGGER poszukiwanie_kredyt_analizy_bez_zmian
BEFORE UPDATE ON poszukiwanie_kredyt_analizy
WHEN NEW.id IS NOT OLD.id
  OR NEW.wersja_parametrow IS NOT OLD.wersja_parametrow
  OR NEW.okres_lat IS NOT OLD.okres_lat
  OR NEW.poza_standardem IS NOT OLD.poza_standardem
  OR NEW.poza_standardem_decyzja_id IS NOT OLD.poza_standardem_decyzja_id
  OR NEW.kwota_od IS NOT OLD.kwota_od
  OR NEW.kwota_do IS NOT OLD.kwota_do
  OR NEW.rata_od IS NOT OLD.rata_od
  OR NEW.rata_do IS NOT OLD.rata_do
  OR NEW.udzial_wkladu IS NOT OLD.udzial_wkladu
  OR NEW.wynik_sciezka IS NOT OLD.wynik_sciezka
  OR NEW.wynik_skrot IS NOT OLD.wynik_skrot
  OR NEW.wyjasnienie IS NOT OLD.wyjasnienie
  OR NEW.zastrzezenie IS NOT OLD.zastrzezenie
  OR NEW.utworzono IS NOT OLD.utworzono
  OR (OLD.aktualna = 0 AND NEW.aktualna = 1)
BEGIN
  SELECT RAISE(ABORT, 'wyniku analizy się nie zmienia; nowe obliczenie to nowa analiza');
END;

-- Do programu idzie wartość z dokumentu, a nie z deklaracji, i tylko dana wpisana przed analizą.
CREATE TRIGGER poszukiwanie_kredyt_do_programu_dane_z_dokumentow
BEFORE INSERT ON poszukiwanie_kredyt_analizy_dane
WHEN NOT EXISTS (
  SELECT 1
  FROM poszukiwanie_kredyt_dane AS dana
  JOIN poszukiwanie_kredyt_analizy AS analiza ON analiza.id = NEW.analiza_id
  WHERE dana.id = NEW.dana_id
    AND dana.deklaracja = 0
    AND dana.wpisano <= analiza.utworzono
)
BEGIN
  SELECT RAISE(ABORT, 'do programu idą tylko dane z dokumentów wpisane przed analizą, nie deklaracje klienta');
END;

CREATE TRIGGER poszukiwanie_kredyt_analizy_dane_bez_zmian
BEFORE UPDATE ON poszukiwanie_kredyt_analizy_dane
BEGIN
  SELECT RAISE(ABORT, 'danych analizy się nie zmienia; inne dane to nowa analiza');
END;

-- Wybór banku jest zapisem tego, co powiedział klient; wymagania banku wpisuje się raz.
CREATE TRIGGER poszukiwanie_kredyt_banki_wymagania_raz
BEFORE UPDATE ON poszukiwanie_kredyt_banki
WHEN NEW.id IS NOT OLD.id
  OR NEW.bank IS NOT OLD.bank
  OR NEW.wybrano IS NOT OLD.wybrano
  OR NEW.zrodlo_wplyw_id IS NOT OLD.zrodlo_wplyw_id
  OR (OLD.wymagania_zrodlo IS NOT NULL AND NEW.wymagania_zrodlo IS NOT OLD.wymagania_zrodlo)
  OR (OLD.wymagania_data IS NOT NULL AND NEW.wymagania_data IS NOT OLD.wymagania_data)
BEGIN
  SELECT RAISE(ABORT, 'wyboru banku się nie zmienia; wymagania banku wpisuje się raz');
END;

-- Datę przewidywaną program przelicza, dopóki krok się nie odbył; datę rzeczywistą wpisuje się
-- raz, a potem kroku się nie zmienia.
CREATE TRIGGER poszukiwanie_kredyt_harmonogram_po_fakcie_bez_zmian
BEFORE UPDATE ON poszukiwanie_kredyt_harmonogram
WHEN OLD.rzeczywista IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.krok IS NOT OLD.krok
BEGIN
  SELECT RAISE(ABORT, 'kroku harmonogramu po fakcie się nie zmienia; przewidywaną datę przelicza się do chwili kroku');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER poszukiwanie_kredyt_dane_bez_zastepowania
BEFORE INSERT ON poszukiwanie_kredyt_dane
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_kredyt_dane
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_kredyt_dane_bez_usuwania
BEFORE DELETE ON poszukiwanie_kredyt_dane
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER poszukiwanie_kredyt_analizy_bez_zastepowania
BEFORE INSERT ON poszukiwanie_kredyt_analizy
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_kredyt_analizy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_kredyt_analizy_bez_usuwania
BEFORE DELETE ON poszukiwanie_kredyt_analizy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER poszukiwanie_kredyt_analizy_dane_bez_zastepowania
BEFORE INSERT ON poszukiwanie_kredyt_analizy_dane
WHEN EXISTS (
    SELECT 1
    FROM poszukiwanie_kredyt_analizy_dane
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM poszukiwanie_kredyt_analizy_dane
    WHERE analiza_id = NEW.analiza_id
      AND dana_id = NEW.dana_id
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_kredyt_analizy_dane_bez_usuwania
BEFORE DELETE ON poszukiwanie_kredyt_analizy_dane
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER poszukiwanie_kredyt_banki_bez_zastepowania
BEFORE INSERT ON poszukiwanie_kredyt_banki
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_kredyt_banki
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_kredyt_banki_bez_usuwania
BEFORE DELETE ON poszukiwanie_kredyt_banki
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER poszukiwanie_kredyt_harmonogram_bez_zastepowania
BEFORE INSERT ON poszukiwanie_kredyt_harmonogram
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_kredyt_harmonogram
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_kredyt_harmonogram_bez_usuwania
BEFORE DELETE ON poszukiwanie_kredyt_harmonogram
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
    ],
};

/// The values of column `dane.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DaneRodzaj {
    Dochod,
    Zobowiazanie,
    Gospodarstwo,
    WkladWlasny,
    Cena,
    Okres,
}

impl DaneRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[DaneRodzaj] = &[
        DaneRodzaj::Dochod,
        DaneRodzaj::Zobowiazanie,
        DaneRodzaj::Gospodarstwo,
        DaneRodzaj::WkladWlasny,
        DaneRodzaj::Cena,
        DaneRodzaj::Okres,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            DaneRodzaj::Dochod => "dochod",
            DaneRodzaj::Zobowiazanie => "zobowiazanie",
            DaneRodzaj::Gospodarstwo => "gospodarstwo",
            DaneRodzaj::WkladWlasny => "wklad_wlasny",
            DaneRodzaj::Cena => "cena",
            DaneRodzaj::Okres => "okres",
        }
    }
}

impl xmeta_room::FromSqlValue for DaneRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "dochod" => Ok(DaneRodzaj::Dochod),
                    "zobowiazanie" => Ok(DaneRodzaj::Zobowiazanie),
                    "gospodarstwo" => Ok(DaneRodzaj::Gospodarstwo),
                    "wklad_wlasny" => Ok(DaneRodzaj::WkladWlasny),
                    "cena" => Ok(DaneRodzaj::Cena),
                    "okres" => Ok(DaneRodzaj::Okres),
                    other => Err(format!("{other} is no value of DaneRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for DaneRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `harmonogram.krok`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HarmonogramKrok {
    Wniosek,
    Wycena,
    DecyzjaKredytowa,
    UmowaKredytu,
    Wyplata,
}

impl HarmonogramKrok {
    /// Every value, in the order of the catalog.
    pub const ALL: &[HarmonogramKrok] = &[
        HarmonogramKrok::Wniosek,
        HarmonogramKrok::Wycena,
        HarmonogramKrok::DecyzjaKredytowa,
        HarmonogramKrok::UmowaKredytu,
        HarmonogramKrok::Wyplata,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            HarmonogramKrok::Wniosek => "wniosek",
            HarmonogramKrok::Wycena => "wycena",
            HarmonogramKrok::DecyzjaKredytowa => "decyzja_kredytowa",
            HarmonogramKrok::UmowaKredytu => "umowa_kredytu",
            HarmonogramKrok::Wyplata => "wyplata",
        }
    }
}

impl xmeta_room::FromSqlValue for HarmonogramKrok {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "wniosek" => Ok(HarmonogramKrok::Wniosek),
                    "wycena" => Ok(HarmonogramKrok::Wycena),
                    "decyzja_kredytowa" => Ok(HarmonogramKrok::DecyzjaKredytowa),
                    "umowa_kredytu" => Ok(HarmonogramKrok::UmowaKredytu),
                    "wyplata" => Ok(HarmonogramKrok::Wyplata),
                    other => Err(format!("{other} is no value of HarmonogramKrok")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for HarmonogramKrok {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `analizy_kredytowe`.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalizyKredytoweRow {
    pub id: String,
    pub wersja_parametrow: String,
    pub okres_lat: i64,
    pub poza_standardem: bool,
    pub poza_standardem_decyzja_id: Option<String>,
    pub kwota_od: i64,
    pub kwota_do: i64,
    pub rata_od: i64,
    pub rata_do: i64,
    pub udzial_wkladu: i64,
    pub wynik_sciezka: String,
    pub wynik_skrot: String,
    pub wyjasnienie: String,
    pub zastrzezenie: String,
    pub aktualna: bool,
    pub utworzono: String,
}

impl AnalizyKredytoweRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(AnalizyKredytoweRow {
            id: row.get("id")?,
            wersja_parametrow: row.get("wersja_parametrow")?,
            okres_lat: row.get("okres_lat")?,
            poza_standardem: row.get("poza_standardem")?,
            poza_standardem_decyzja_id: row.get("poza_standardem_decyzja_id")?,
            kwota_od: row.get("kwota_od")?,
            kwota_do: row.get("kwota_do")?,
            rata_od: row.get("rata_od")?,
            rata_do: row.get("rata_do")?,
            udzial_wkladu: row.get("udzial_wkladu")?,
            wynik_sciezka: row.get("wynik_sciezka")?,
            wynik_skrot: row.get("wynik_skrot")?,
            wyjasnienie: row.get("wyjasnienie")?,
            zastrzezenie: row.get("zastrzezenie")?,
            aktualna: row.get("aktualna")?,
            utworzono: row.get("utworzono")?,
        })
    }
}

/// Analizy kredytowe, najnowsze pierwsze.
pub fn analizy_kredytowe(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<AnalizyKredytoweRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM poszukiwanie_kredyt_analizy
ORDER BY utworzono DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(AnalizyKredytoweRow::from_row).collect()
}

/// A row of `analiza_aktualna`.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalizaAktualnaRow {
    pub id: String,
    pub wersja_parametrow: String,
    pub okres_lat: i64,
    pub poza_standardem: bool,
    pub poza_standardem_decyzja_id: Option<String>,
    pub kwota_od: i64,
    pub kwota_do: i64,
    pub rata_od: i64,
    pub rata_do: i64,
    pub udzial_wkladu: i64,
    pub wynik_sciezka: String,
    pub wynik_skrot: String,
    pub wyjasnienie: String,
    pub zastrzezenie: String,
    pub aktualna: bool,
    pub utworzono: String,
}

impl AnalizaAktualnaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(AnalizaAktualnaRow {
            id: row.get("id")?,
            wersja_parametrow: row.get("wersja_parametrow")?,
            okres_lat: row.get("okres_lat")?,
            poza_standardem: row.get("poza_standardem")?,
            poza_standardem_decyzja_id: row.get("poza_standardem_decyzja_id")?,
            kwota_od: row.get("kwota_od")?,
            kwota_do: row.get("kwota_do")?,
            rata_od: row.get("rata_od")?,
            rata_do: row.get("rata_do")?,
            udzial_wkladu: row.get("udzial_wkladu")?,
            wynik_sciezka: row.get("wynik_sciezka")?,
            wynik_skrot: row.get("wynik_skrot")?,
            wyjasnienie: row.get("wyjasnienie")?,
            zastrzezenie: row.get("zastrzezenie")?,
            aktualna: row.get("aktualna")?,
            utworzono: row.get("utworzono")?,
        })
    }
}

/// Analiza, która obowiązuje; poprzednie straciły aktualność.
pub fn analiza_aktualna(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Option<AnalizaAktualnaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM poszukiwanie_kredyt_analizy
WHERE aktualna = 1"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.first().map(AnalizaAktualnaRow::from_row).transpose()
}

/// A row of `dane_analiz`.
#[derive(Debug, Clone, PartialEq)]
pub struct DaneAnalizRow {
    pub id: i64,
    pub analiza_id: String,
    pub dana_id: String,
}

impl DaneAnalizRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(DaneAnalizRow {
            id: row.get("id")?,
            analiza_id: row.get("analiza_id")?,
            dana_id: row.get("dana_id")?,
        })
    }
}

/// Dane wejściowe wprowadzone do programu w każdej analizie.
pub fn dane_analiz(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<DaneAnalizRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM poszukiwanie_kredyt_analizy_dane
ORDER BY analiza_id, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(DaneAnalizRow::from_row).collect()
}

/// The parameters of `wpisz_analize_kredytowa`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszAnalizeKredytowaParams {
    pub id: String,
    pub wersja_parametrow: String,
    pub okres_lat: i64,
    pub poza_standardem: bool,
    pub poza_standardem_decyzja_id: Option<String>,
    pub kwota_od: i64,
    pub kwota_do: i64,
    pub rata_od: i64,
    pub rata_do: i64,
    pub udzial_wkladu: i64,
    pub wynik_sciezka: String,
    pub wynik_skrot: String,
    pub wyjasnienie: String,
    pub zastrzezenie: String,
    pub utworzono: String,
}

/// Wynik programu z wersją parametrów, wyjaśnieniem i zastrzeżeniem (R2); poprzednie analizy
/// tracą aktualność. Analiza poza standardem wskazuje zatwierdzoną decyzję człowieka.
pub fn wpisz_analize_kredytowa(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszAnalizeKredytowaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO poszukiwanie_kredyt_analizy (id, wersja_parametrow, okres_lat, poza_standardem,
  poza_standardem_decyzja_id, kwota_od, kwota_do, rata_od, rata_do, udzial_wkladu, wynik_sciezka,
  wynik_skrot, wyjasnienie, zastrzezenie, aktualna, utworzono)
VALUES (:id, :wersja_parametrow, :okres_lat, :poza_standardem, :poza_standardem_decyzja_id,
  :kwota_od, :kwota_do, :rata_od, :rata_do, :udzial_wkladu, :wynik_sciezka, :wynik_skrot,
  :wyjasnienie, :zastrzezenie, 1, :utworzono)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "wersja_parametrow",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wersja_parametrow),
                    ),
                    (
                        "okres_lat",
                        xmeta_room::ToSqlValue::to_sql_value(&params.okres_lat),
                    ),
                    (
                        "poza_standardem",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poza_standardem),
                    ),
                    (
                        "poza_standardem_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.poza_standardem_decyzja_id,
                        ),
                    ),
                    ("kwota_od", xmeta_room::ToSqlValue::to_sql_value(&params.kwota_od)),
                    ("kwota_do", xmeta_room::ToSqlValue::to_sql_value(&params.kwota_do)),
                    ("rata_od", xmeta_room::ToSqlValue::to_sql_value(&params.rata_od)),
                    ("rata_do", xmeta_room::ToSqlValue::to_sql_value(&params.rata_do)),
                    (
                        "udzial_wkladu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.udzial_wkladu),
                    ),
                    (
                        "wynik_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wynik_sciezka),
                    ),
                    (
                        "wynik_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wynik_skrot),
                    ),
                    (
                        "wyjasnienie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wyjasnienie),
                    ),
                    (
                        "zastrzezenie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zastrzezenie),
                    ),
                    (
                        "utworzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.utworzono),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `dopisz_dana_do_analizy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DopiszDanaDoAnalizyParams {
    pub analiza_id: String,
    pub dana_id: String,
}

/// Dana z dokumentu, którą Analityk wprowadził do programu w tej analizie.
pub fn dopisz_dana_do_analizy(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszDanaDoAnalizyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO poszukiwanie_kredyt_analizy_dane (analiza_id, dana_id)
VALUES (:analiza_id, :dana_id)"#,
                parameters: Vec::from([
                    (
                        "analiza_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.analiza_id),
                    ),
                    ("dana_id", xmeta_room::ToSqlValue::to_sql_value(&params.dana_id)),
                ]),
            },
        ],
    )
}

/// A row of `banki_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct BankiKlientaRow {
    pub id: i64,
    pub bank: String,
    pub wybrano: String,
    pub zrodlo_wplyw_id: String,
    pub wymagania_zrodlo: Option<String>,
    pub wymagania_data: Option<String>,
}

impl BankiKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(BankiKlientaRow {
            id: row.get("id")?,
            bank: row.get("bank")?,
            wybrano: row.get("wybrano")?,
            zrodlo_wplyw_id: row.get("zrodlo_wplyw_id")?,
            wymagania_zrodlo: row.get("wymagania_zrodlo")?,
            wymagania_data: row.get("wymagania_data")?,
        })
    }
}

/// Banki, które wybrał klient, z jego wiadomością i wymaganiami banku.
pub fn banki_klienta(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<BankiKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM poszukiwanie_kredyt_banki
ORDER BY wybrano, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(BankiKlientaRow::from_row).collect()
}

/// The parameters of `wpisz_bank_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszBankKlientaParams {
    pub bank: String,
    pub wybrano: String,
    pub zrodlo_wplyw_id: String,
}

/// Bank wybrany przez klienta z wpływem, z którego to wiadomo; biuro banków nie poleca (R2).
pub fn wpisz_bank_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszBankKlientaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO poszukiwanie_kredyt_banki (bank, wybrano, zrodlo_wplyw_id)
VALUES (:bank, :wybrano, :zrodlo_wplyw_id)"#,
                parameters: Vec::from([
                    ("bank", xmeta_room::ToSqlValue::to_sql_value(&params.bank)),
                    ("wybrano", xmeta_room::ToSqlValue::to_sql_value(&params.wybrano)),
                    (
                        "zrodlo_wplyw_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_wplyw_id),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_wymagania_banku`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujWymaganiaBankuParams {
    pub wymagania_zrodlo: String,
    pub wymagania_data: String,
    pub id: i64,
}

/// Skąd teczka zna wymagania banku wobec dokumentów i z kiedy; wpisuje się je raz.
pub fn odnotuj_wymagania_banku(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWymaganiaBankuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE poszukiwanie_kredyt_banki
SET wymagania_zrodlo = :wymagania_zrodlo,
  wymagania_data = :wymagania_data
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "wymagania_zrodlo",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wymagania_zrodlo),
                    ),
                    (
                        "wymagania_data",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wymagania_data),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `dane_kredytowe`.
#[derive(Debug, Clone, PartialEq)]
pub struct DaneKredytoweRow {
    pub id: String,
    pub rodzaj: DaneRodzaj,
    pub opis: String,
    pub wartosc: i64,
    pub okres_zrodla: Option<String>,
    pub dokument_id: Option<String>,
    pub deklaracja: bool,
    pub wpisano: String,
}

impl DaneKredytoweRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(DaneKredytoweRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            opis: row.get("opis")?,
            wartosc: row.get("wartosc")?,
            okres_zrodla: row.get("okres_zrodla")?,
            dokument_id: row.get("dokument_id")?,
            deklaracja: row.get("deklaracja")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Dane wejściowe analizy kredytowej w kolejności wpisania.
pub fn dane_kredytowe(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<DaneKredytoweRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM poszukiwanie_kredyt_dane
ORDER BY wpisano, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(DaneKredytoweRow::from_row).collect()
}

/// The parameters of `wpisz_dana_kredytowa`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszDanaKredytowaParams {
    pub id: String,
    pub rodzaj: DaneRodzaj,
    pub opis: String,
    pub wartosc: i64,
    pub okres_zrodla: Option<String>,
    pub dokument_id: Option<String>,
    pub deklaracja: bool,
    pub wpisano: String,
}

/// Dana wejściowa z dokumentem albo deklaracja klienta oznaczona jako niepotwierdzona;
/// identyfikator (UUID) nadaje aplikacja, bo wskazują go analizy.
pub fn wpisz_dana_kredytowa(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszDanaKredytowaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO poszukiwanie_kredyt_dane (id, rodzaj, opis, wartosc, okres_zrodla, dokument_id,
  deklaracja, wpisano)
VALUES (:id, :rodzaj, :opis, :wartosc, :okres_zrodla, :dokument_id, :deklaracja, :wpisano)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    ("wartosc", xmeta_room::ToSqlValue::to_sql_value(&params.wartosc)),
                    (
                        "okres_zrodla",
                        xmeta_room::ToSqlValue::to_sql_value(&params.okres_zrodla),
                    ),
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                    (
                        "deklaracja",
                        xmeta_room::ToSqlValue::to_sql_value(&params.deklaracja),
                    ),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
        ],
    )
}

/// A row of `harmonogram_kredytu`.
#[derive(Debug, Clone, PartialEq)]
pub struct HarmonogramKredytuRow {
    pub id: i64,
    pub krok: HarmonogramKrok,
    pub przewidywana: String,
    pub rzeczywista: Option<String>,
}

impl HarmonogramKredytuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(HarmonogramKredytuRow {
            id: row.get("id")?,
            krok: row.get("krok")?,
            przewidywana: row.get("przewidywana")?,
            rzeczywista: row.get("rzeczywista")?,
        })
    }
}

/// Kroki do wypłaty kredytu z datą przewidywaną i rzeczywistą.
pub fn harmonogram_kredytu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<HarmonogramKredytuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM poszukiwanie_kredyt_harmonogram
ORDER BY przewidywana, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(HarmonogramKredytuRow::from_row).collect()
}

/// The parameters of `wpisz_krok_harmonogramu`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszKrokHarmonogramuParams {
    pub krok: HarmonogramKrok,
    pub przewidywana: String,
}

/// Krok drogi do wypłaty z datą przewidywaną, którą liczy program.
pub fn wpisz_krok_harmonogramu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszKrokHarmonogramuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO poszukiwanie_kredyt_harmonogram (krok, przewidywana)
VALUES (:krok, :przewidywana)"#,
                parameters: Vec::from([
                    ("krok", xmeta_room::ToSqlValue::to_sql_value(&params.krok)),
                    (
                        "przewidywana",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przewidywana),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `przelicz_krok_harmonogramu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrzeliczKrokHarmonogramuParams {
    pub przewidywana: String,
    pub id: i64,
}

/// Nowa data przewidywana z programu, dopóki krok się nie odbył.
pub fn przelicz_krok_harmonogramu(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzeliczKrokHarmonogramuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE poszukiwanie_kredyt_harmonogram
SET przewidywana = :przewidywana
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "przewidywana",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przewidywana),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_krok_harmonogramu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujKrokHarmonogramuParams {
    pub rzeczywista: String,
    pub id: i64,
}

/// Data, w której krok się odbył; potem kroku się nie zmienia.
pub fn odnotuj_krok_harmonogramu(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujKrokHarmonogramuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE poszukiwanie_kredyt_harmonogram
SET rzeczywista = :rzeczywista
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "rzeczywista",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rzeczywista),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
