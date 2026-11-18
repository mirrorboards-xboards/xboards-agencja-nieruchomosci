//! The kind `katalog-ofert-oferty` of the room `registers/katalog_ofert`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "katalog-ofert-oferty",
    requires: &[],
    migrations: &[
        r#"-- Katalog ofert biura: każda oferta sprzedaży i najmu długoterminowego z teczki oferty, tylko
-- w zakresie, na który pozwolił klient, z wersjami, ogłoszeniami i statusem. Opiekun
-- poszukującego dobiera z niego oferty biura, a Recepcja rozpoznaje numer oferty albo ogłoszenia.
-- Pokój jest jeden na biuro.
--
-- Pisze Kierownik pośrednictwa, zawsze ze źródłem w teczce oferty (tekstem, bez klucza obcego).
-- Czytają Opiekunowie poszukujących, Recepcja i Kierownik biura. Wierszy się nie zmienia: zmiana
-- ceny, opisu, zdjęć, widoczności albo wyłączności to nowa wersja, a bieżąca jest najnowsza. Dzięki
-- temu wiadomo, którą wersję Opiekun pokazał klientowi. Jedyną zmianą jest koniec ogłoszenia.
--
-- Katalog nie ma kolumn na dane właściciela, dokładny adres, numer księgi wieczystej ani granice
-- negocjacji (R1, art. 4 pkt 1 RODO); to wynika z budowy tabel. Lokali krótkoterminowych tu nie ma.
CREATE TABLE katalog_ofert_oferty_wpisy (
  id TEXT PRIMARY KEY,
  -- Numer nadany przez biuro, który podają ogłoszenia i wiadomości o ofercie.
  numer_oferty TEXT NOT NULL UNIQUE CHECK (numer_oferty <> ''),
  -- Teczka oferty; odnośnik nie daje do niej dostępu.
  teczka_odnosnik TEXT NOT NULL CHECK (teczka_odnosnik <> ''),
  rodzaj_oferty TEXT NOT NULL CHECK (rodzaj_oferty IN ('sprzedaz', 'najem', 'sprzedaz_i_najem')),
  rodzaj_nieruchomosci TEXT NOT NULL CHECK (rodzaj_nieruchomosci IN ('lokal', 'dom_z_dzialka', 'dzialka', 'spoldzielcze_prawo_do_lokalu')),
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL
) STRICT;

-- Wersje tego, co klient pozwolił pokazać. Kwoty są w groszach, takie, jakie zatwierdził klient.
CREATE TABLE katalog_ofert_oferty_wersje (
  -- Klucz nadany przez bazę jest dodatni; zob. katalog_ofert_oferty_wersje_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  oferta_id TEXT NOT NULL REFERENCES katalog_ofert_oferty_wpisy (id),
  od TEXT NOT NULL,
  -- Powierzchnia użytkowa, a przy działce jej powierzchnia, w metrach kwadratowych.
  powierzchnia REAL NOT NULL CHECK (powierzchnia > 0),
  -- Miejscowość, dzielnica albo okolica, bez numeru budynku i lokalu.
  lokalizacja_przyblizona TEXT NOT NULL CHECK (lokalizacja_przyblizona <> ''),
  cena_sprzedazy INTEGER CHECK (cena_sprzedazy > 0),
  czynsz INTEGER CHECK (czynsz > 0),
  -- Opis i zdjęcia bez osób, dokumentów i rzeczy, po których można rozpoznać klienta.
  opis TEXT NOT NULL CHECK (opis <> ''),
  zdjecia_sciezka TEXT,
  zdjecia_skrot TEXT,
  widocznosc TEXT NOT NULL CHECK (widocznosc IN ('publiczna', 'niepubliczna')),
  -- Przy ofercie niepublicznej: komu i co biuro może pokazać.
  granice_zgody TEXT,
  wylacznosc INTEGER NOT NULL CHECK (wylacznosc IN (0, 1)),
  -- Zgoda klienta (sprawa_zgody_wpisy), wersja widoczności w teczce oferty albo umowa.
  zrodlo_odnosnik TEXT NOT NULL CHECK (zrodlo_odnosnik <> ''),
  -- Aneks do umowy pośrednictwa (sprawa_umowy_zdarzenia), gdy wersja zmienia widoczność albo
  -- wyłączność.
  aneks_odnosnik TEXT,
  -- Wersja, którą ta poprawia, gdy poprzednia była błędna.
  poprawia_id INTEGER REFERENCES katalog_ofert_oferty_wersje (id),
  wpisal TEXT NOT NULL,

  CONSTRAINT granice_zgody_tylko_przy_niepublicznej CHECK ((widocznosc = 'niepubliczna') = (granice_zgody IS NOT NULL)),
  CONSTRAINT zdjecia_ze_skrotem CHECK ((zdjecia_sciezka IS NULL) = (zdjecia_skrot IS NULL))
) STRICT;

-- Ogłoszenia oferty publicznej z numerami nadanymi przez portale; po nich Recepcja rozpoznaje
-- ofertę w wiadomości z portalu.
CREATE TABLE katalog_ofert_oferty_ogloszenia (
  -- Klucz nadany przez bazę jest dodatni; zob. katalog_ofert_oferty_ogloszenia_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  oferta_id TEXT NOT NULL REFERENCES katalog_ofert_oferty_wpisy (id),
  portal TEXT NOT NULL CHECK (portal <> ''),
  numer_ogloszenia TEXT NOT NULL CHECK (numer_ogloszenia <> ''),
  od TEXT NOT NULL,
  do TEXT,

  CONSTRAINT koniec_nie_przed_poczatkiem CHECK (do IS NULL OR do >= od),

  UNIQUE (portal, numer_ogloszenia)
) STRICT;

-- Status oferty z datą i źródłem w teczce oferty (oferta zainteresowanego albo umowa).
CREATE TABLE katalog_ofert_oferty_statusy (
  -- Klucz nadany przez bazę jest dodatni; zob. katalog_ofert_oferty_statusy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  oferta_id TEXT NOT NULL REFERENCES katalog_ofert_oferty_wpisy (id),
  status TEXT NOT NULL CHECK (status IN ('aktywna', 'zarezerwowana', 'sprzedana_albo_wynajeta', 'wycofana')),
  od TEXT NOT NULL,
  zrodlo_odnosnik TEXT NOT NULL CHECK (zrodlo_odnosnik <> ''),
  wpisal TEXT NOT NULL
) STRICT;

-- Wersje ---------------------------------------------------------------------------------------

-- Przy sprzedaży jest cena, przy najmie czynsz, a przy ofercie na oba sposoby obie kwoty.
CREATE TRIGGER katalog_ofert_oferty_kwoty_pasuja_do_rodzaju
BEFORE INSERT ON katalog_ofert_oferty_wersje
WHEN (
  SELECT
    (rodzaj_oferty IN ('sprzedaz', 'sprzedaz_i_najem')) IS NOT (NEW.cena_sprzedazy IS NOT NULL)
    OR (rodzaj_oferty IN ('najem', 'sprzedaz_i_najem')) IS NOT (NEW.czynsz IS NOT NULL)
  FROM katalog_ofert_oferty_wpisy
  WHERE id = NEW.oferta_id
)
BEGIN
  SELECT RAISE(ABORT, 'oferta sprzedaży ma cenę, oferta najmu czynsz, a oferta na oba sposoby obie kwoty');
END;

-- Tryb widoczności i wyłączność wybiera klient w umowie pośrednictwa, więc ich zmiana wymaga
-- aneksu podpisanego przez klienta.
CREATE TRIGGER katalog_ofert_oferty_zmiana_widocznosci_z_aneksem
BEFORE INSERT ON katalog_ofert_oferty_wersje
WHEN NEW.aneks_odnosnik IS NULL
  AND EXISTS (
    SELECT 1
    FROM katalog_ofert_oferty_wersje AS poprzednia
    WHERE poprzednia.oferta_id = NEW.oferta_id
      AND poprzednia.id = (
        SELECT max(id)
        FROM katalog_ofert_oferty_wersje
        WHERE oferta_id = NEW.oferta_id
      )
      AND (
        poprzednia.widocznosc IS NOT NEW.widocznosc
        OR poprzednia.wylacznosc IS NOT NEW.wylacznosc
      )
  )
BEGIN
  SELECT RAISE(ABORT, 'zmiana widoczności albo wyłączności wymaga aneksu do umowy pośrednictwa');
END;

CREATE TRIGGER katalog_ofert_oferty_wersja_nie_starsza_niz_poprzednia
BEFORE INSERT ON katalog_ofert_oferty_wersje
WHEN NEW.od < (
  SELECT max(od)
  FROM katalog_ofert_oferty_wersje
  WHERE oferta_id = NEW.oferta_id
)
BEGIN
  SELECT RAISE(ABORT, 'nowa wersja oferty nie obowiązuje wcześniej niż poprzednia');
END;

CREATE TRIGGER katalog_ofert_oferty_poprawka_tej_samej_oferty
BEFORE INSERT ON katalog_ofert_oferty_wersje
WHEN NEW.poprawia_id IS NOT NULL
  AND (
    SELECT oferta_id
    FROM katalog_ofert_oferty_wersje
    WHERE id = NEW.poprawia_id
  ) IS NOT NEW.oferta_id
BEGIN
  SELECT RAISE(ABORT, 'wersja poprawia wcześniejszą wersję tej samej oferty');
END;

-- Oferty niepublicznej biuro nie publikuje, więc nie przechodzi na nią oferta z trwającym
-- ogłoszeniem.
CREATE TRIGGER katalog_ofert_oferty_niepubliczna_bez_trwajacych_ogloszen
BEFORE INSERT ON katalog_ofert_oferty_wersje
WHEN NEW.widocznosc = 'niepubliczna'
  AND EXISTS (
    SELECT 1
    FROM katalog_ofert_oferty_ogloszenia
    WHERE oferta_id = NEW.oferta_id
      AND do IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'oferta niepubliczna nie ma ogłoszeń; najpierw kończy się ogłoszenia');
END;

-- Ogłoszenia -----------------------------------------------------------------------------------

CREATE TRIGGER katalog_ofert_oferty_ogloszenie_tylko_publicznej
BEFORE INSERT ON katalog_ofert_oferty_ogloszenia
WHEN (
  SELECT widocznosc
  FROM katalog_ofert_oferty_wersje
  WHERE oferta_id = NEW.oferta_id
  ORDER BY id DESC
  LIMIT 1
) IS NOT 'publiczna'
BEGIN
  SELECT RAISE(ABORT, 'oferta niepubliczna nie ma ogłoszeń; najpierw kończy się ogłoszenia');
END;

CREATE TRIGGER katalog_ofert_oferty_ogloszenie_powstaje_trwajace
BEFORE INSERT ON katalog_ofert_oferty_ogloszenia
WHEN NEW.do IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'ogłoszenie powstaje jako trwające');
END;

-- Ogłoszenie może się tylko raz zakończyć.
CREATE TRIGGER katalog_ofert_oferty_ogloszenia_bez_zmian
BEFORE UPDATE ON katalog_ofert_oferty_ogloszenia
WHEN NEW.id IS NOT OLD.id
  OR NEW.oferta_id IS NOT OLD.oferta_id
  OR NEW.portal IS NOT OLD.portal
  OR NEW.numer_ogloszenia IS NOT OLD.numer_ogloszenia
  OR NEW.od IS NOT OLD.od
  OR (OLD.do IS NOT NULL AND NEW.do IS NOT OLD.do)
BEGIN
  SELECT RAISE(ABORT, 'ogłoszenia się nie zmienia; może się tylko raz zakończyć');
END;

-- Status ---------------------------------------------------------------------------------------

-- Oferta zaczyna jako aktywna; aktywna ⇄ zarezerwowana → sprzedana albo wynajęta, a z każdego
-- stanu do wycofanej. Stany końcowe są ostateczne.
CREATE TRIGGER katalog_ofert_oferty_status_z_przejsciem
BEFORE INSERT ON katalog_ofert_oferty_statusy
WHEN NOT (
  CASE (
    SELECT status
    FROM katalog_ofert_oferty_statusy
    WHERE oferta_id = NEW.oferta_id
    ORDER BY id DESC
    LIMIT 1
  )
    WHEN 'aktywna' THEN NEW.status IN ('zarezerwowana', 'wycofana')
    WHEN 'zarezerwowana' THEN NEW.status IN ('aktywna', 'sprzedana_albo_wynajeta', 'wycofana')
    WHEN 'sprzedana_albo_wynajeta' THEN 0
    WHEN 'wycofana' THEN 0
    ELSE NEW.status = 'aktywna'
  END
)
BEGIN
  SELECT RAISE(ABORT, 'status oferty: aktywna ⇄ zarezerwowana → sprzedana albo wynajęta, z każdego do wycofanej, a stany końcowe są ostateczne');
END;

CREATE TRIGGER katalog_ofert_oferty_status_nie_przed_poprzednim
BEFORE INSERT ON katalog_ofert_oferty_statusy
WHEN NEW.od < (
  SELECT max(od)
  FROM katalog_ofert_oferty_statusy
  WHERE oferta_id = NEW.oferta_id
)
BEGIN
  SELECT RAISE(ABORT, 'status oferty nie zmienia się przed poprzednim');
END;

-- Zakończona oferta zostaje w katalogu, bo wskazują ją propozycje w teczkach poszukiwania, ale nie
-- dostaje nowych wersji ani ogłoszeń.
CREATE TRIGGER katalog_ofert_oferty_zakonczona_bez_nowej_wersji
BEFORE INSERT ON katalog_ofert_oferty_wersje
WHEN (
  SELECT status
  FROM katalog_ofert_oferty_statusy
  WHERE oferta_id = NEW.oferta_id
  ORDER BY id DESC
  LIMIT 1
) IN ('sprzedana_albo_wynajeta', 'wycofana')
BEGIN
  SELECT RAISE(ABORT, 'zakończona oferta nie dostaje nowych wersji ani ogłoszeń');
END;

CREATE TRIGGER katalog_ofert_oferty_zakonczona_bez_nowego_ogloszenia
BEFORE INSERT ON katalog_ofert_oferty_ogloszenia
WHEN (
  SELECT status
  FROM katalog_ofert_oferty_statusy
  WHERE oferta_id = NEW.oferta_id
  ORDER BY id DESC
  LIMIT 1
) IN ('sprzedana_albo_wynajeta', 'wycofana')
BEGIN
  SELECT RAISE(ABORT, 'zakończona oferta nie dostaje nowych wersji ani ogłoszeń');
END;

-- Wiersze bez zmian ----------------------------------------------------------------------------

CREATE TRIGGER katalog_ofert_oferty_wpisy_bez_zmian
BEFORE UPDATE ON katalog_ofert_oferty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wpisu oferty, wersji ani statusu się nie zmienia; zmiana to nowa wersja albo nowy status');
END;

CREATE TRIGGER katalog_ofert_oferty_wersje_bez_zmian
BEFORE UPDATE ON katalog_ofert_oferty_wersje
BEGIN
  SELECT RAISE(ABORT, 'wpisu oferty, wersji ani statusu się nie zmienia; zmiana to nowa wersja albo nowy status');
END;

CREATE TRIGGER katalog_ofert_oferty_statusy_bez_zmian
BEFORE UPDATE ON katalog_ofert_oferty_statusy
BEGIN
  SELECT RAISE(ABORT, 'wpisu oferty, wersji ani statusu się nie zmienia; zmiana to nowa wersja albo nowy status');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabel z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER katalog_ofert_oferty_wpisy_bez_zastepowania
BEFORE INSERT ON katalog_ofert_oferty_wpisy
WHEN EXISTS (
    SELECT 1
    FROM katalog_ofert_oferty_wpisy
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM katalog_ofert_oferty_wpisy
    WHERE numer_oferty = NEW.numer_oferty
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza katalogu nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER katalog_ofert_oferty_wersje_bez_zastepowania
BEFORE INSERT ON katalog_ofert_oferty_wersje
WHEN EXISTS (
  SELECT 1
  FROM katalog_ofert_oferty_wersje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza katalogu nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER katalog_ofert_oferty_ogloszenia_bez_zastepowania
BEFORE INSERT ON katalog_ofert_oferty_ogloszenia
WHEN EXISTS (
    SELECT 1
    FROM katalog_ofert_oferty_ogloszenia
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM katalog_ofert_oferty_ogloszenia
    WHERE portal = NEW.portal
      AND numer_ogloszenia = NEW.numer_ogloszenia
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza katalogu nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER katalog_ofert_oferty_statusy_bez_zastepowania
BEFORE INSERT ON katalog_ofert_oferty_statusy
WHEN EXISTS (
  SELECT 1
  FROM katalog_ofert_oferty_statusy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza katalogu nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

-- R9: biuro trzyma zdjęcia i opis zakończonej oferty bez terminu.
CREATE TRIGGER katalog_ofert_oferty_wpisy_bez_usuwania
BEFORE DELETE ON katalog_ofert_oferty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy katalogu się nie usuwa (R9); błąd poprawia nowa wersja');
END;

CREATE TRIGGER katalog_ofert_oferty_wersje_bez_usuwania
BEFORE DELETE ON katalog_ofert_oferty_wersje
BEGIN
  SELECT RAISE(ABORT, 'wierszy katalogu się nie usuwa (R9); błąd poprawia nowa wersja');
END;

CREATE TRIGGER katalog_ofert_oferty_ogloszenia_bez_usuwania
BEFORE DELETE ON katalog_ofert_oferty_ogloszenia
BEGIN
  SELECT RAISE(ABORT, 'wierszy katalogu się nie usuwa (R9); błąd poprawia nowa wersja');
END;

CREATE TRIGGER katalog_ofert_oferty_statusy_bez_usuwania
BEFORE DELETE ON katalog_ofert_oferty_statusy
BEGIN
  SELECT RAISE(ABORT, 'wierszy katalogu się nie usuwa (R9); błąd poprawia nowa wersja');
END;
"#,
    ],
};

/// The values of column `statusy.status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusyStatus {
    Aktywna,
    Zarezerwowana,
    SprzedanaAlboWynajeta,
    Wycofana,
}

impl StatusyStatus {
    /// Every value, in the order of the catalog.
    pub const ALL: &[StatusyStatus] = &[
        StatusyStatus::Aktywna,
        StatusyStatus::Zarezerwowana,
        StatusyStatus::SprzedanaAlboWynajeta,
        StatusyStatus::Wycofana,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            StatusyStatus::Aktywna => "aktywna",
            StatusyStatus::Zarezerwowana => "zarezerwowana",
            StatusyStatus::SprzedanaAlboWynajeta => "sprzedana_albo_wynajeta",
            StatusyStatus::Wycofana => "wycofana",
        }
    }
}

impl xmeta_room::FromSqlValue for StatusyStatus {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "aktywna" => Ok(StatusyStatus::Aktywna),
                    "zarezerwowana" => Ok(StatusyStatus::Zarezerwowana),
                    "sprzedana_albo_wynajeta" => Ok(StatusyStatus::SprzedanaAlboWynajeta),
                    "wycofana" => Ok(StatusyStatus::Wycofana),
                    other => Err(format!("{other} is no value of StatusyStatus")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for StatusyStatus {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wersje.widocznosc`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WersjeWidocznosc {
    Publiczna,
    Niepubliczna,
}

impl WersjeWidocznosc {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WersjeWidocznosc] = &[
        WersjeWidocznosc::Publiczna,
        WersjeWidocznosc::Niepubliczna,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WersjeWidocznosc::Publiczna => "publiczna",
            WersjeWidocznosc::Niepubliczna => "niepubliczna",
        }
    }
}

impl xmeta_room::FromSqlValue for WersjeWidocznosc {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "publiczna" => Ok(WersjeWidocznosc::Publiczna),
                    "niepubliczna" => Ok(WersjeWidocznosc::Niepubliczna),
                    other => Err(format!("{other} is no value of WersjeWidocznosc")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WersjeWidocznosc {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.rodzaj_oferty`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzajOferty {
    Sprzedaz,
    Najem,
    SprzedazINajem,
}

impl WpisyRodzajOferty {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzajOferty] = &[
        WpisyRodzajOferty::Sprzedaz,
        WpisyRodzajOferty::Najem,
        WpisyRodzajOferty::SprzedazINajem,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzajOferty::Sprzedaz => "sprzedaz",
            WpisyRodzajOferty::Najem => "najem",
            WpisyRodzajOferty::SprzedazINajem => "sprzedaz_i_najem",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzajOferty {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "sprzedaz" => Ok(WpisyRodzajOferty::Sprzedaz),
                    "najem" => Ok(WpisyRodzajOferty::Najem),
                    "sprzedaz_i_najem" => Ok(WpisyRodzajOferty::SprzedazINajem),
                    other => Err(format!("{other} is no value of WpisyRodzajOferty")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyRodzajOferty {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.rodzaj_nieruchomosci`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzajNieruchomosci {
    Lokal,
    DomZDzialka,
    Dzialka,
    SpoldzielczePrawoDoLokalu,
}

impl WpisyRodzajNieruchomosci {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzajNieruchomosci] = &[
        WpisyRodzajNieruchomosci::Lokal,
        WpisyRodzajNieruchomosci::DomZDzialka,
        WpisyRodzajNieruchomosci::Dzialka,
        WpisyRodzajNieruchomosci::SpoldzielczePrawoDoLokalu,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzajNieruchomosci::Lokal => "lokal",
            WpisyRodzajNieruchomosci::DomZDzialka => "dom_z_dzialka",
            WpisyRodzajNieruchomosci::Dzialka => "dzialka",
            WpisyRodzajNieruchomosci::SpoldzielczePrawoDoLokalu => {
                "spoldzielcze_prawo_do_lokalu"
            }
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzajNieruchomosci {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "lokal" => Ok(WpisyRodzajNieruchomosci::Lokal),
                    "dom_z_dzialka" => Ok(WpisyRodzajNieruchomosci::DomZDzialka),
                    "dzialka" => Ok(WpisyRodzajNieruchomosci::Dzialka),
                    "spoldzielcze_prawo_do_lokalu" => {
                        Ok(WpisyRodzajNieruchomosci::SpoldzielczePrawoDoLokalu)
                    }
                    other => {
                        Err(format!("{other} is no value of WpisyRodzajNieruchomosci"))
                    }
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyRodzajNieruchomosci {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `ogloszenia_ofert`.
#[derive(Debug, Clone, PartialEq)]
pub struct OgloszeniaOfertRow {
    pub id: i64,
    pub oferta_id: String,
    pub portal: String,
    pub numer_ogloszenia: String,
    pub od: String,
    pub r#do: Option<String>,
}

impl OgloszeniaOfertRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(OgloszeniaOfertRow {
            id: row.get("id")?,
            oferta_id: row.get("oferta_id")?,
            portal: row.get("portal")?,
            numer_ogloszenia: row.get("numer_ogloszenia")?,
            od: row.get("od")?,
            r#do: row.get("do")?,
        })
    }
}

/// Ogłoszenia wszystkich ofert, trwające i zakończone.
pub fn ogloszenia_ofert(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<OgloszeniaOfertRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM katalog_ofert_oferty_ogloszenia
ORDER BY oferta_id, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(OgloszeniaOfertRow::from_row).collect()
}

/// The parameters of `oferta_po_ogloszeniu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OfertaPoOgloszeniuParams {
    pub portal: String,
    pub numer_ogloszenia: String,
}

/// A row of `oferta_po_ogloszeniu`.
#[derive(Debug, Clone, PartialEq)]
pub struct OfertaPoOgloszeniuRow {
    pub id: i64,
    pub oferta_id: String,
    pub portal: String,
    pub numer_ogloszenia: String,
    pub od: String,
    pub r#do: Option<String>,
}

impl OfertaPoOgloszeniuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(OfertaPoOgloszeniuRow {
            id: row.get("id")?,
            oferta_id: row.get("oferta_id")?,
            portal: row.get("portal")?,
            numer_ogloszenia: row.get("numer_ogloszenia")?,
            od: row.get("od")?,
            r#do: row.get("do")?,
        })
    }
}

/// Ogłoszenie o numerze z wiadomości portalu; wskazuje ofertę.
pub fn oferta_po_ogloszeniu(
    room: &impl xmeta_room::MetaRoom,
    params: &OfertaPoOgloszeniuParams,
) -> Result<Option<OfertaPoOgloszeniuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM katalog_ofert_oferty_ogloszenia
WHERE portal = :portal
  AND numer_ogloszenia = :numer_ogloszenia"#,
        parameters: Vec::from([
            ("portal", xmeta_room::ToSqlValue::to_sql_value(&params.portal)),
            (
                "numer_ogloszenia",
                xmeta_room::ToSqlValue::to_sql_value(&params.numer_ogloszenia),
            ),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.first().map(OfertaPoOgloszeniuRow::from_row).transpose()
}

/// The parameters of `wpisz_ogloszenie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszOgloszenieParams {
    pub oferta_id: String,
    pub portal: String,
    pub numer_ogloszenia: String,
    pub od: String,
}

/// Ogłoszenie oferty publicznej z numerem nadanym przez portal.
pub fn wpisz_ogloszenie(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszOgloszenieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO katalog_ofert_oferty_ogloszenia (oferta_id, portal, numer_ogloszenia, od)
VALUES (:oferta_id, :portal, :numer_ogloszenia, :od)"#,
                parameters: Vec::from([
                    (
                        "oferta_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.oferta_id),
                    ),
                    ("portal", xmeta_room::ToSqlValue::to_sql_value(&params.portal)),
                    (
                        "numer_ogloszenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.numer_ogloszenia),
                    ),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                ]),
            },
        ],
    )
}

/// The parameters of `zakoncz_ogloszenie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZakonczOgloszenieParams {
    pub r#do: Option<String>,
    pub id: i64,
}

/// Ogłoszenie zdjęte z portalu.
pub fn zakoncz_ogloszenie(
    room: &impl xmeta_room::MetaRoom,
    params: &ZakonczOgloszenieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE katalog_ofert_oferty_ogloszenia
SET do = :do
WHERE id = :id"#,
                parameters: Vec::from([
                    ("do", xmeta_room::ToSqlValue::to_sql_value(&params.r#do)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `statusy_oferty`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StatusyOfertyParams {
    pub oferta_id: String,
}

/// A row of `statusy_oferty`.
#[derive(Debug, Clone, PartialEq)]
pub struct StatusyOfertyRow {
    pub id: i64,
    pub oferta_id: String,
    pub status: StatusyStatus,
    pub od: String,
    pub zrodlo_odnosnik: String,
    pub wpisal: String,
}

impl StatusyOfertyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(StatusyOfertyRow {
            id: row.get("id")?,
            oferta_id: row.get("oferta_id")?,
            status: row.get("status")?,
            od: row.get("od")?,
            zrodlo_odnosnik: row.get("zrodlo_odnosnik")?,
            wpisal: row.get("wpisal")?,
        })
    }
}

/// Historia statusu oferty ze źródłami.
pub fn statusy_oferty(
    room: &impl xmeta_room::MetaRoom,
    params: &StatusyOfertyParams,
) -> Result<Vec<StatusyOfertyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM katalog_ofert_oferty_statusy
WHERE oferta_id = :oferta_id
ORDER BY id"#,
        parameters: Vec::from([
            ("oferta_id", xmeta_room::ToSqlValue::to_sql_value(&params.oferta_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(StatusyOfertyRow::from_row).collect()
}

/// The parameters of `zmien_status_oferty`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZmienStatusOfertyParams {
    pub oferta_id: String,
    pub status: StatusyStatus,
    pub od: String,
    pub zrodlo_odnosnik: String,
    pub wpisal: String,
}

/// Rezerwacja, powrót do aktywnych, sprzedaż albo najem, wycofanie; ze źródłem w teczce oferty.
pub fn zmien_status_oferty(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienStatusOfertyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO katalog_ofert_oferty_statusy (oferta_id, status, od, zrodlo_odnosnik, wpisal)
VALUES (:oferta_id, :status, :od, :zrodlo_odnosnik, :wpisal)"#,
                parameters: Vec::from([
                    (
                        "oferta_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.oferta_id),
                    ),
                    ("status", xmeta_room::ToSqlValue::to_sql_value(&params.status)),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    (
                        "zrodlo_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_odnosnik),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                ]),
            },
        ],
    )
}

/// The parameters of `wersje_oferty`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WersjeOfertyParams {
    pub oferta_id: String,
}

/// A row of `wersje_oferty`.
#[derive(Debug, Clone, PartialEq)]
pub struct WersjeOfertyRow {
    pub id: i64,
    pub oferta_id: String,
    pub od: String,
    pub powierzchnia: f64,
    pub lokalizacja_przyblizona: String,
    pub cena_sprzedazy: Option<i64>,
    pub czynsz: Option<i64>,
    pub opis: String,
    pub zdjecia_sciezka: Option<String>,
    pub zdjecia_skrot: Option<String>,
    pub widocznosc: WersjeWidocznosc,
    pub granice_zgody: Option<String>,
    pub wylacznosc: bool,
    pub zrodlo_odnosnik: String,
    pub aneks_odnosnik: Option<String>,
    pub poprawia_id: Option<i64>,
    pub wpisal: String,
}

impl WersjeOfertyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WersjeOfertyRow {
            id: row.get("id")?,
            oferta_id: row.get("oferta_id")?,
            od: row.get("od")?,
            powierzchnia: row.get("powierzchnia")?,
            lokalizacja_przyblizona: row.get("lokalizacja_przyblizona")?,
            cena_sprzedazy: row.get("cena_sprzedazy")?,
            czynsz: row.get("czynsz")?,
            opis: row.get("opis")?,
            zdjecia_sciezka: row.get("zdjecia_sciezka")?,
            zdjecia_skrot: row.get("zdjecia_skrot")?,
            widocznosc: row.get("widocznosc")?,
            granice_zgody: row.get("granice_zgody")?,
            wylacznosc: row.get("wylacznosc")?,
            zrodlo_odnosnik: row.get("zrodlo_odnosnik")?,
            aneks_odnosnik: row.get("aneks_odnosnik")?,
            poprawia_id: row.get("poprawia_id")?,
            wpisal: row.get("wpisal")?,
        })
    }
}

/// Wszystkie wersje oferty, najnowsza pierwsza; Opiekun poszukującego widzi, którą pokazał
/// klientowi.
pub fn wersje_oferty(
    room: &impl xmeta_room::MetaRoom,
    params: &WersjeOfertyParams,
) -> Result<Vec<WersjeOfertyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM katalog_ofert_oferty_wersje
WHERE oferta_id = :oferta_id
ORDER BY id DESC"#,
        parameters: Vec::from([
            ("oferta_id", xmeta_room::ToSqlValue::to_sql_value(&params.oferta_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(WersjeOfertyRow::from_row).collect()
}

/// The parameters of `wpisz_wersje_oferty`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszWersjeOfertyParams {
    pub oferta_id: String,
    pub od: String,
    pub powierzchnia: f64,
    pub lokalizacja_przyblizona: String,
    pub cena_sprzedazy: Option<i64>,
    pub czynsz: Option<i64>,
    pub opis: String,
    pub zdjecia_sciezka: Option<String>,
    pub zdjecia_skrot: Option<String>,
    pub widocznosc: WersjeWidocznosc,
    pub granice_zgody: Option<String>,
    pub wylacznosc: bool,
    pub zrodlo_odnosnik: String,
    pub aneks_odnosnik: Option<String>,
    pub poprawia_id: Option<i64>,
    pub wpisal: String,
}

/// Nowa cena, opis, zdjęcia, widoczność albo wyłączność ze źródłem w teczce; zmiana widoczności
/// albo wyłączności ma aneks. Błąd poprzedniej wersji poprawia nowa, która ją wskazuje.
pub fn wpisz_wersje_oferty(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszWersjeOfertyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO katalog_ofert_oferty_wersje (oferta_id, od, powierzchnia, lokalizacja_przyblizona,
  cena_sprzedazy, czynsz, opis, zdjecia_sciezka, zdjecia_skrot, widocznosc, granice_zgody,
  wylacznosc, zrodlo_odnosnik, aneks_odnosnik, poprawia_id, wpisal)
VALUES (:oferta_id, :od, :powierzchnia, :lokalizacja_przyblizona, :cena_sprzedazy, :czynsz, :opis,
  :zdjecia_sciezka, :zdjecia_skrot, :widocznosc, :granice_zgody, :wylacznosc, :zrodlo_odnosnik,
  :aneks_odnosnik, :poprawia_id, :wpisal)"#,
                parameters: Vec::from([
                    (
                        "oferta_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.oferta_id),
                    ),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    (
                        "powierzchnia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.powierzchnia),
                    ),
                    (
                        "lokalizacja_przyblizona",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.lokalizacja_przyblizona,
                        ),
                    ),
                    (
                        "cena_sprzedazy",
                        xmeta_room::ToSqlValue::to_sql_value(&params.cena_sprzedazy),
                    ),
                    ("czynsz", xmeta_room::ToSqlValue::to_sql_value(&params.czynsz)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    (
                        "zdjecia_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zdjecia_sciezka),
                    ),
                    (
                        "zdjecia_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zdjecia_skrot),
                    ),
                    (
                        "widocznosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.widocznosc),
                    ),
                    (
                        "granice_zgody",
                        xmeta_room::ToSqlValue::to_sql_value(&params.granice_zgody),
                    ),
                    (
                        "wylacznosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wylacznosc),
                    ),
                    (
                        "zrodlo_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_odnosnik),
                    ),
                    (
                        "aneks_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.aneks_odnosnik),
                    ),
                    (
                        "poprawia_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprawia_id),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                ]),
            },
        ],
    )
}

/// A row of `oferty`.
#[derive(Debug, Clone, PartialEq)]
pub struct OfertyRow {
    pub id: String,
    pub numer_oferty: String,
    pub teczka_odnosnik: String,
    pub rodzaj_oferty: WpisyRodzajOferty,
    pub rodzaj_nieruchomosci: WpisyRodzajNieruchomosci,
    pub wersja_id: i64,
    pub wersja_od: String,
    pub powierzchnia: f64,
    pub lokalizacja_przyblizona: String,
    pub cena_sprzedazy: Option<i64>,
    pub czynsz: Option<i64>,
    pub opis: String,
    pub zdjecia_sciezka: Option<String>,
    pub widocznosc: WersjeWidocznosc,
    pub granice_zgody: Option<String>,
    pub wylacznosc: bool,
    pub status: StatusyStatus,
    pub status_od: String,
}

impl OfertyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(OfertyRow {
            id: row.get("id")?,
            numer_oferty: row.get("numer_oferty")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            rodzaj_oferty: row.get("rodzaj_oferty")?,
            rodzaj_nieruchomosci: row.get("rodzaj_nieruchomosci")?,
            wersja_id: row.get("wersja_id")?,
            wersja_od: row.get("wersja_od")?,
            powierzchnia: row.get("powierzchnia")?,
            lokalizacja_przyblizona: row.get("lokalizacja_przyblizona")?,
            cena_sprzedazy: row.get("cena_sprzedazy")?,
            czynsz: row.get("czynsz")?,
            opis: row.get("opis")?,
            zdjecia_sciezka: row.get("zdjecia_sciezka")?,
            widocznosc: row.get("widocznosc")?,
            granice_zgody: row.get("granice_zgody")?,
            wylacznosc: row.get("wylacznosc")?,
            status: row.get("status")?,
            status_od: row.get("status_od")?,
        })
    }
}

/// Oferty katalogu z bieżącą wersją i bieżącym statusem, w kolejności wpisania. Opiekun
/// poszukującego proponuje tylko aktywne, a zarezerwowane z informacją o rezerwacji; niepubliczną
/// pokazuje tylko w granicach zgody.
pub fn oferty(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<OfertyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT
  wpisy.id,
  wpisy.numer_oferty,
  wpisy.teczka_odnosnik,
  wpisy.rodzaj_oferty,
  wpisy.rodzaj_nieruchomosci,
  wersje.id AS wersja_id,
  wersje.od AS wersja_od,
  wersje.powierzchnia,
  wersje.lokalizacja_przyblizona,
  wersje.cena_sprzedazy,
  wersje.czynsz,
  wersje.opis,
  wersje.zdjecia_sciezka,
  wersje.widocznosc,
  wersje.granice_zgody,
  wersje.wylacznosc,
  statusy.status,
  statusy.od AS status_od
FROM katalog_ofert_oferty_wpisy AS wpisy
JOIN katalog_ofert_oferty_wersje AS wersje ON wersje.oferta_id = wpisy.id
  AND wersje.id = (
    SELECT max(id)
    FROM katalog_ofert_oferty_wersje
    WHERE oferta_id = wpisy.id
  )
JOIN katalog_ofert_oferty_statusy AS statusy ON statusy.oferta_id = wpisy.id
  AND statusy.id = (
    SELECT max(id)
    FROM katalog_ofert_oferty_statusy
    WHERE oferta_id = wpisy.id
  )
ORDER BY wpisy.rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(OfertyRow::from_row).collect()
}

/// The parameters of `oferta_po_numerze`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OfertaPoNumerzeParams {
    pub numer_oferty: String,
}

/// A row of `oferta_po_numerze`.
#[derive(Debug, Clone, PartialEq)]
pub struct OfertaPoNumerzeRow {
    pub id: String,
    pub numer_oferty: String,
    pub teczka_odnosnik: String,
    pub rodzaj_oferty: WpisyRodzajOferty,
    pub rodzaj_nieruchomosci: WpisyRodzajNieruchomosci,
    pub wpisal: String,
    pub wpisano: String,
}

impl OfertaPoNumerzeRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(OfertaPoNumerzeRow {
            id: row.get("id")?,
            numer_oferty: row.get("numer_oferty")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            rodzaj_oferty: row.get("rodzaj_oferty")?,
            rodzaj_nieruchomosci: row.get("rodzaj_nieruchomosci")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Oferta o numerze z wiadomości; wpis wskazuje teczkę, której Opiekunowi oferty Recepcja oddaje
/// wiadomość.
pub fn oferta_po_numerze(
    room: &impl xmeta_room::MetaRoom,
    params: &OfertaPoNumerzeParams,
) -> Result<Option<OfertaPoNumerzeRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM katalog_ofert_oferty_wpisy
WHERE numer_oferty = :numer_oferty"#,
        parameters: Vec::from([
            ("numer_oferty", xmeta_room::ToSqlValue::to_sql_value(&params.numer_oferty)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.first().map(OfertaPoNumerzeRow::from_row).transpose()
}

/// The parameters of `wpisz_oferte`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszOferteParams {
    pub id: String,
    pub numer_oferty: String,
    pub teczka_odnosnik: String,
    pub rodzaj_oferty: WpisyRodzajOferty,
    pub rodzaj_nieruchomosci: WpisyRodzajNieruchomosci,
    pub wpisal: String,
    pub wpisano: String,
    pub od: String,
    pub powierzchnia: f64,
    pub lokalizacja_przyblizona: String,
    pub cena_sprzedazy: Option<i64>,
    pub czynsz: Option<i64>,
    pub opis: String,
    pub zdjecia_sciezka: Option<String>,
    pub zdjecia_skrot: Option<String>,
    pub widocznosc: WersjeWidocznosc,
    pub granice_zgody: Option<String>,
    pub wylacznosc: bool,
    pub zrodlo_odnosnik: String,
}

/// Nowa oferta po raporcie Opiekuna oferty: wpis, pierwsza wersja i status „aktywna” ze źródłem
/// w teczce. Identyfikator (UUID) nadaje aplikacja.
pub fn wpisz_oferte(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszOferteParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO katalog_ofert_oferty_wpisy (id, numer_oferty, teczka_odnosnik, rodzaj_oferty,
  rodzaj_nieruchomosci, wpisal, wpisano)
VALUES (:id, :numer_oferty, :teczka_odnosnik, :rodzaj_oferty, :rodzaj_nieruchomosci, :wpisal,
  :wpisano)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "numer_oferty",
                        xmeta_room::ToSqlValue::to_sql_value(&params.numer_oferty),
                    ),
                    (
                        "teczka_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.teczka_odnosnik),
                    ),
                    (
                        "rodzaj_oferty",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj_oferty),
                    ),
                    (
                        "rodzaj_nieruchomosci",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.rodzaj_nieruchomosci,
                        ),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
            xmeta_room::Statement {
                sql: r#"INSERT INTO katalog_ofert_oferty_wersje (oferta_id, od, powierzchnia, lokalizacja_przyblizona,
  cena_sprzedazy, czynsz, opis, zdjecia_sciezka, zdjecia_skrot, widocznosc, granice_zgody,
  wylacznosc, zrodlo_odnosnik, wpisal)
VALUES (:id, :od, :powierzchnia, :lokalizacja_przyblizona, :cena_sprzedazy, :czynsz, :opis,
  :zdjecia_sciezka, :zdjecia_skrot, :widocznosc, :granice_zgody, :wylacznosc, :zrodlo_odnosnik,
  :wpisal)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    (
                        "powierzchnia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.powierzchnia),
                    ),
                    (
                        "lokalizacja_przyblizona",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.lokalizacja_przyblizona,
                        ),
                    ),
                    (
                        "cena_sprzedazy",
                        xmeta_room::ToSqlValue::to_sql_value(&params.cena_sprzedazy),
                    ),
                    ("czynsz", xmeta_room::ToSqlValue::to_sql_value(&params.czynsz)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    (
                        "zdjecia_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zdjecia_sciezka),
                    ),
                    (
                        "zdjecia_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zdjecia_skrot),
                    ),
                    (
                        "widocznosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.widocznosc),
                    ),
                    (
                        "granice_zgody",
                        xmeta_room::ToSqlValue::to_sql_value(&params.granice_zgody),
                    ),
                    (
                        "wylacznosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wylacznosc),
                    ),
                    (
                        "zrodlo_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_odnosnik),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                ]),
            },
            xmeta_room::Statement {
                sql: r#"INSERT INTO katalog_ofert_oferty_statusy (oferta_id, status, od, zrodlo_odnosnik, wpisal)
VALUES (:id, 'aktywna', :od, :zrodlo_odnosnik, :wpisal)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    (
                        "zrodlo_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_odnosnik),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                ]),
            },
        ],
    )
}
