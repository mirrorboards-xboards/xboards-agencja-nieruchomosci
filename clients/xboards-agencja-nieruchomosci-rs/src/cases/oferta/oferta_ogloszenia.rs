//! The kind `oferta-ogloszenia` of the room `cases/oferta`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "oferta-ogloszenia",
    requires: &[
        &super::oferta_nieruchomosc::KIND,
        &super::sprawa_dokumenty::KIND,
        &super::sprawa_umowy::KIND,
    ],
    migrations: &[
        r#"-- Ogłoszenia oferty na portalach i na stronie biura, z wersjami treści i statystykami. Pisze
-- Opiekun oferty według standardu działu. Ogłoszenie jest zaproszeniem do składania ofert, a nie
-- ofertą (art. 71 KC).
--
-- Ogłoszenie powstaje jako projekt i dostaje wersje treści. Publikacja i każda nowa wersja
-- opublikowanego ogłoszenia przechodzą te same reguły, sprawdzane na najnowszej wersji i chwili
-- publikacji albo wersji:
-- - umowa pośrednictwa jest zawarta, a jej zakres obejmuje publikacja_ogloszen (art. 180 ust. 3
--   ugn), i biuro może już według niej pracować (sprawa_umowy_praca_od);
-- - bieżąca widoczność oferty jest publiczna i obejmuje to, czego ogłoszenie dotyczy;
-- - cena w wersji równa się cenie ofertowej, która obowiązuje;
-- - gdy świadectwo charakterystyki energetycznej jest, wersja ma jego wskaźniki, a świadectwo jest
--   ważne w dniu publikacji (art. 13 uceb);
-- - gdy ostatnie przeglądanie księgi nieruchomości wykazało rozbieżność, publikacja ma zatwierdzoną
--   decyzję „inna”, która wskazuje to przeglądanie.
-- Pierwsza publikacja w teczce ma też zatwierdzoną decyzję „inna” pośrednika, która wskazuje
-- ogłoszenie. Zdjęte ogłoszenie ma powód i nie wraca; wersji się nie zmienia.
--
-- Adres w ogłoszeniu bez numeru lokalu i zdjęcia bez osób i dokumentów sprawdza aplikacja
-- i standard ogłoszenia; baza tego nie umie.

-- Ogłoszenie w jednym kanale; numer nadaje portal po publikacji.
CREATE TABLE oferta_ogloszenia_wpisy (
  id TEXT PRIMARY KEY,
  dotyczy TEXT NOT NULL CHECK (dotyczy IN ('sprzedaz', 'najem')),
  kanal TEXT NOT NULL CHECK (kanal IN ('portal', 'strona_biura')),
  portal TEXT,
  numer_ogloszenia TEXT,
  opublikowano TEXT,
  zdjeto TEXT,
  -- Np. przyjęta oferta albo koniec umowy pośrednictwa.
  powod_zdjecia TEXT,

  CONSTRAINT portal_przy_kanale_portal CHECK ((kanal = 'portal') = (portal IS NOT NULL)),
  CONSTRAINT numer_po_publikacji CHECK (numer_ogloszenia IS NULL OR opublikowano IS NOT NULL),
  CONSTRAINT zdjete_ma_powod CHECK ((zdjeto IS NULL) = (powod_zdjecia IS NULL)),
  CONSTRAINT zdjete_po_publikacji CHECK (zdjeto IS NULL OR (opublikowano IS NOT NULL AND zdjeto >= opublikowano))
) STRICT;

-- Wersje treści ogłoszenia: tytuł, cena, lokalizacja bez numeru lokalu, wskaźniki energetyczne
-- i plik treści ze zdjęciami i rzutem. Każda zmiana treści to nowa wersja z datą.
CREATE TABLE oferta_ogloszenia_wersje (
  -- Klucz nadany przez bazę jest dodatni; zob. oferta_ogloszenia_wersje_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  ogloszenie_id TEXT NOT NULL REFERENCES oferta_ogloszenia_wpisy (id),
  numer INTEGER NOT NULL CHECK (numer >= 1),
  od TEXT NOT NULL,
  tytul TEXT NOT NULL,
  cena INTEGER NOT NULL CHECK (cena > 0),
  lokalizacja TEXT NOT NULL,
  wskazniki_energetyczne TEXT,
  tresc_sciezka TEXT NOT NULL,
  tresc_skrot TEXT NOT NULL,

  UNIQUE (ogloszenie_id, numer)
) STRICT;

-- Liczby wyświetleń i zapytań podane przez portal, z chwilą odczytu.
CREATE TABLE oferta_ogloszenia_statystyki (
  -- Klucz nadany przez bazę jest dodatni; zob. oferta_ogloszenia_statystyki_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  ogloszenie_id TEXT NOT NULL REFERENCES oferta_ogloszenia_wpisy (id),
  odczytano TEXT NOT NULL,
  wyswietlenia INTEGER NOT NULL CHECK (wyswietlenia >= 0),
  zapytania INTEGER NOT NULL CHECK (zapytania >= 0)
) STRICT;

-- Najnowsza wersja każdego ogłoszenia; ją sprawdza publikacja.
CREATE VIEW oferta_ogloszenia_wersje_najnowsze AS
SELECT wersja.*
FROM oferta_ogloszenia_wersje AS wersja
WHERE wersja.numer = (
  SELECT max(numer)
  FROM oferta_ogloszenia_wersje AS inna
  WHERE inna.ogloszenie_id = wersja.ogloszenie_id
);

-- Publikacja sprawdza zmianę, więc ogłoszenie powstaje jako projekt.
CREATE TRIGGER oferta_ogloszenia_powstaje_jako_projekt
BEFORE INSERT ON oferta_ogloszenia_wpisy
WHEN NEW.opublikowano IS NOT NULL
  OR NEW.numer_ogloszenia IS NOT NULL
  OR NEW.zdjeto IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'ogłoszenie powstaje jako projekt, bez publikacji, numeru i zdjęcia');
END;

-- Kanał i to, czego ogłoszenie dotyczy, się nie zmieniają; publikację, numer i zdjęcie wpisuje
-- się raz, a zdjęte ogłoszenie nie wraca.
CREATE TRIGGER oferta_ogloszenia_wpisy_bez_zmian
BEFORE UPDATE ON oferta_ogloszenia_wpisy
WHEN OLD.zdjeto IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.dotyczy IS NOT OLD.dotyczy
  OR NEW.kanal IS NOT OLD.kanal
  OR NEW.portal IS NOT OLD.portal
  OR (OLD.opublikowano IS NOT NULL AND NEW.opublikowano IS NOT OLD.opublikowano)
  OR (OLD.numer_ogloszenia IS NOT NULL AND NEW.numer_ogloszenia IS NOT OLD.numer_ogloszenia)
BEGIN
  SELECT RAISE(ABORT, 'ogłoszenia się nie zmienia; publikację, numer i zdjęcie wpisuje się raz, a zdjęte nie wraca');
END;

-- Publikacja: najnowsza wersja treści istnieje i nie jest późniejsza niż publikacja.
CREATE TRIGGER oferta_ogloszenia_publikacja_ma_wersje
BEFORE UPDATE OF opublikowano ON oferta_ogloszenia_wpisy
WHEN NEW.opublikowano IS NOT NULL
  AND OLD.opublikowano IS NULL
  AND NOT EXISTS (
    SELECT 1
    FROM oferta_ogloszenia_wersje
    WHERE ogloszenie_id = NEW.id
      AND od <= NEW.opublikowano
  )
BEGIN
  SELECT RAISE(ABORT, 'publikacja wymaga wersji treści sprzed publikacji');
END;

-- Publikacja: zawarta umowa pośrednictwa z czynnością publikacja_ogloszen.
CREATE TRIGGER oferta_ogloszenia_publikacja_w_zakresie_umowy
BEFORE UPDATE OF opublikowano ON oferta_ogloszenia_wpisy
WHEN NEW.opublikowano IS NOT NULL
  AND OLD.opublikowano IS NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_umowy_wpisy AS umowa
    JOIN sprawa_umowy_zakres AS zakres ON zakres.umowa_id = umowa.id
    WHERE umowa.rodzaj = 'posrednictwo'
      AND umowa.stan = 'zawarta'
      AND zakres.czynnosc = 'publikacja_ogloszen'
  )
BEGIN
  SELECT RAISE(ABORT, 'ogłoszenie wymaga zawartej umowy pośrednictwa, której zakres obejmuje publikacja_ogloszen');
END;

-- Publikacja: nie wcześniej niż biuro może pracować według umowy (art. 15 ust. 3 i art. 21
-- ust. 2 upk).
CREATE TRIGGER oferta_ogloszenia_publikacja_od_pracy
BEFORE UPDATE OF opublikowano ON oferta_ogloszenia_wpisy
WHEN NEW.opublikowano IS NOT NULL
  AND OLD.opublikowano IS NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_umowy_wpisy AS umowa
    JOIN sprawa_umowy_praca_od AS praca ON praca.umowa_id = umowa.id
    WHERE umowa.rodzaj = 'posrednictwo'
      AND umowa.stan = 'zawarta'
      AND NEW.opublikowano >= praca.od
  )
BEGIN
  SELECT RAISE(ABORT, 'ogłoszenie wychodzi nie wcześniej niż praca od według umowy pośrednictwa');
END;

-- Publikacja: oferta publiczna, której rodzaj obejmuje to, czego ogłoszenie dotyczy.
CREATE TRIGGER oferta_ogloszenia_publikacja_oferty_publicznej
BEFORE UPDATE OF opublikowano ON oferta_ogloszenia_wpisy
WHEN NEW.opublikowano IS NOT NULL
  AND OLD.opublikowano IS NULL
  AND NOT EXISTS (
    SELECT 1
    FROM oferta_widocznosc_biezaca
    WHERE widocznosc = 'publiczna'
      AND rodzaj_oferty IN (NEW.dotyczy, 'sprzedaz_albo_najem')
  )
BEGIN
  SELECT RAISE(ABORT, 'ogłasza się tylko ofertę publiczną, której rodzaj obejmuje to, czego ogłoszenie dotyczy');
END;

-- Publikacja: cena w najnowszej wersji to obowiązująca cena ofertowa.
CREATE TRIGGER oferta_ogloszenia_publikacja_z_cena_ofertowa
BEFORE UPDATE OF opublikowano ON oferta_ogloszenia_wpisy
WHEN NEW.opublikowano IS NOT NULL
  AND OLD.opublikowano IS NULL
  AND NOT EXISTS (
    SELECT 1
    FROM oferta_ogloszenia_wersje_najnowsze AS wersja
    JOIN oferta_ceny_biezace AS cena ON cena.dotyczy = NEW.dotyczy
    WHERE wersja.ogloszenie_id = NEW.id
      AND wersja.cena = cena.kwota
  )
BEGIN
  SELECT RAISE(ABORT, 'cena w ogłoszeniu to obowiązująca cena ofertowa zatwierdzona przez klienta');
END;

-- Publikacja: przy świadectwie wersja ma wskaźniki, a świadectwo jest ważne w dniu publikacji.
CREATE TRIGGER oferta_ogloszenia_publikacja_ze_swiadectwem
BEFORE UPDATE OF opublikowano ON oferta_ogloszenia_wpisy
WHEN NEW.opublikowano IS NOT NULL
  AND OLD.opublikowano IS NULL
  AND EXISTS (
    SELECT 1
    FROM oferta_nieruchomosc_opis AS opis
    JOIN sprawa_dokumenty_wpisy AS swiadectwo ON swiadectwo.id = opis.swiadectwo_dokument_id
    WHERE opis.swiadectwo = 'jest'
      AND (
        swiadectwo.waznosc_do < substr(NEW.opublikowano, 1, 10)
        OR EXISTS (
          SELECT 1
          FROM oferta_ogloszenia_wersje_najnowsze AS wersja
          WHERE wersja.ogloszenie_id = NEW.id
            AND wersja.wskazniki_energetyczne IS NULL
        )
      )
  )
BEGIN
  SELECT RAISE(ABORT, 'przy świadectwie energetycznym ogłoszenie ma jego wskaźniki, a świadectwo jest ważne w dniu publikacji');
END;

-- Publikacja: rozbieżność w ostatnim przeglądaniu księgi wymaga zatwierdzonej decyzji „inna”,
-- która wskazuje to przeglądanie.
CREATE TRIGGER oferta_ogloszenia_publikacja_mimo_rozbieznosci_za_zgoda
BEFORE UPDATE OF opublikowano ON oferta_ogloszenia_wpisy
WHEN NEW.opublikowano IS NOT NULL
  AND OLD.opublikowano IS NULL
  AND EXISTS (
    SELECT 1
    FROM oferta_nieruchomosc_przegladania AS przegladanie
    WHERE przegladanie.rozbieznosc = 1
      AND NOT EXISTS (
        SELECT 1
        FROM oferta_nieruchomosc_przegladania AS nowsze
        WHERE nowsze.nieruchomosc_id = przegladanie.nieruchomosc_id
          AND nowsze.przegladano > przegladanie.przegladano
      )
      AND NOT EXISTS (
        SELECT 1
        FROM sprawa_decyzje_zatwierdzone AS decyzja
        WHERE decyzja.rodzaj = 'inna'
          AND decyzja.dotyczy_tabela = 'oferta_nieruchomosc_przegladania'
          AND decyzja.dotyczy_id = przegladanie.id
      )
  )
BEGIN
  SELECT RAISE(ABORT, 'przy rozbieżności z księgą ogłoszenie wymaga zatwierdzonej decyzji inna, która wskazuje przeglądanie');
END;

-- Pierwszą publikację w teczce zatwierdza pośrednik decyzją „inna”, która wskazuje ogłoszenie.
CREATE TRIGGER oferta_ogloszenia_pierwsza_publikacja_za_zgoda
BEFORE UPDATE OF opublikowano ON oferta_ogloszenia_wpisy
WHEN NEW.opublikowano IS NOT NULL
  AND OLD.opublikowano IS NULL
  AND NOT EXISTS (
    SELECT 1
    FROM oferta_ogloszenia_wpisy
    WHERE opublikowano IS NOT NULL
  )
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS decyzja
    WHERE decyzja.rodzaj = 'inna'
      AND decyzja.dotyczy_tabela = 'oferta_ogloszenia_wpisy'
      AND decyzja.dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'pierwsza publikacja w teczce wymaga zatwierdzonej decyzji inna, która wskazuje ogłoszenie');
END;

-- Wersje mają kolejne numery od 1 i nie są wcześniejsze niż poprzednia; zdjęte ogłoszenie nie
-- dostaje nowych wersji.
CREATE TRIGGER oferta_ogloszenia_wersje_kolejno
BEFORE INSERT ON oferta_ogloszenia_wersje
WHEN NEW.numer IS NOT (
    SELECT coalesce(max(numer), 0) + 1
    FROM oferta_ogloszenia_wersje
    WHERE ogloszenie_id = NEW.ogloszenie_id
  )
  OR EXISTS (
    SELECT 1
    FROM oferta_ogloszenia_wersje
    WHERE ogloszenie_id = NEW.ogloszenie_id
      AND od > NEW.od
  )
BEGIN
  SELECT RAISE(ABORT, 'wersje ogłoszenia mają kolejne numery od 1 i nie są wcześniejsze niż poprzednia');
END;

CREATE TRIGGER oferta_ogloszenia_zdjete_bez_nowych_wersji
BEFORE INSERT ON oferta_ogloszenia_wersje
WHEN (
  SELECT zdjeto IS NOT NULL
  FROM oferta_ogloszenia_wpisy
  WHERE id = NEW.ogloszenie_id
)
BEGIN
  SELECT RAISE(ABORT, 'zdjęte ogłoszenie nie dostaje nowych wersji; nie wraca');
END;

-- Nowa wersja opublikowanego ogłoszenia przechodzi te same reguły co publikacja, na chwili
-- wersji: umowa i jej zakres, praca od, widoczność, cena, świadectwo i rozbieżność.
CREATE TRIGGER oferta_ogloszenia_wersja_w_zakresie_umowy
BEFORE INSERT ON oferta_ogloszenia_wersje
WHEN (
    SELECT opublikowano IS NOT NULL
    FROM oferta_ogloszenia_wpisy
    WHERE id = NEW.ogloszenie_id
  )
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_umowy_wpisy AS umowa
    JOIN sprawa_umowy_zakres AS zakres ON zakres.umowa_id = umowa.id
    WHERE umowa.rodzaj = 'posrednictwo'
      AND umowa.stan = 'zawarta'
      AND zakres.czynnosc = 'publikacja_ogloszen'
  )
BEGIN
  SELECT RAISE(ABORT, 'ogłoszenie wymaga zawartej umowy pośrednictwa, której zakres obejmuje publikacja_ogloszen');
END;

CREATE TRIGGER oferta_ogloszenia_wersja_od_pracy
BEFORE INSERT ON oferta_ogloszenia_wersje
WHEN (
    SELECT opublikowano IS NOT NULL
    FROM oferta_ogloszenia_wpisy
    WHERE id = NEW.ogloszenie_id
  )
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_umowy_wpisy AS umowa
    JOIN sprawa_umowy_praca_od AS praca ON praca.umowa_id = umowa.id
    WHERE umowa.rodzaj = 'posrednictwo'
      AND umowa.stan = 'zawarta'
      AND NEW.od >= praca.od
  )
BEGIN
  SELECT RAISE(ABORT, 'ogłoszenie wychodzi nie wcześniej niż praca od według umowy pośrednictwa');
END;

CREATE TRIGGER oferta_ogloszenia_wersja_oferty_publicznej
BEFORE INSERT ON oferta_ogloszenia_wersje
WHEN (
    SELECT opublikowano IS NOT NULL
    FROM oferta_ogloszenia_wpisy
    WHERE id = NEW.ogloszenie_id
  )
  AND NOT EXISTS (
    SELECT 1
    FROM oferta_widocznosc_biezaca AS widocznosc
    JOIN oferta_ogloszenia_wpisy AS ogloszenie ON ogloszenie.id = NEW.ogloszenie_id
    WHERE widocznosc.widocznosc = 'publiczna'
      AND widocznosc.rodzaj_oferty IN (ogloszenie.dotyczy, 'sprzedaz_albo_najem')
  )
BEGIN
  SELECT RAISE(ABORT, 'ogłasza się tylko ofertę publiczną, której rodzaj obejmuje to, czego ogłoszenie dotyczy');
END;

CREATE TRIGGER oferta_ogloszenia_wersja_z_cena_ofertowa
BEFORE INSERT ON oferta_ogloszenia_wersje
WHEN (
    SELECT opublikowano IS NOT NULL
    FROM oferta_ogloszenia_wpisy
    WHERE id = NEW.ogloszenie_id
  )
  AND NOT EXISTS (
    SELECT 1
    FROM oferta_ceny_biezace AS cena
    JOIN oferta_ogloszenia_wpisy AS ogloszenie ON ogloszenie.dotyczy = cena.dotyczy
    WHERE ogloszenie.id = NEW.ogloszenie_id
      AND cena.kwota = NEW.cena
  )
BEGIN
  SELECT RAISE(ABORT, 'cena w ogłoszeniu to obowiązująca cena ofertowa zatwierdzona przez klienta');
END;

CREATE TRIGGER oferta_ogloszenia_wersja_ze_swiadectwem
BEFORE INSERT ON oferta_ogloszenia_wersje
WHEN (
    SELECT opublikowano IS NOT NULL
    FROM oferta_ogloszenia_wpisy
    WHERE id = NEW.ogloszenie_id
  )
  AND EXISTS (
    SELECT 1
    FROM oferta_nieruchomosc_opis AS opis
    JOIN sprawa_dokumenty_wpisy AS swiadectwo ON swiadectwo.id = opis.swiadectwo_dokument_id
    WHERE opis.swiadectwo = 'jest'
      AND (
        swiadectwo.waznosc_do < substr(NEW.od, 1, 10)
        OR NEW.wskazniki_energetyczne IS NULL
      )
  )
BEGIN
  SELECT RAISE(ABORT, 'przy świadectwie energetycznym ogłoszenie ma jego wskaźniki, a świadectwo jest ważne w dniu publikacji');
END;

CREATE TRIGGER oferta_ogloszenia_wersja_mimo_rozbieznosci_za_zgoda
BEFORE INSERT ON oferta_ogloszenia_wersje
WHEN (
    SELECT opublikowano IS NOT NULL
    FROM oferta_ogloszenia_wpisy
    WHERE id = NEW.ogloszenie_id
  )
  AND EXISTS (
    SELECT 1
    FROM oferta_nieruchomosc_przegladania AS przegladanie
    WHERE przegladanie.rozbieznosc = 1
      AND NOT EXISTS (
        SELECT 1
        FROM oferta_nieruchomosc_przegladania AS nowsze
        WHERE nowsze.nieruchomosc_id = przegladanie.nieruchomosc_id
          AND nowsze.przegladano > przegladanie.przegladano
      )
      AND NOT EXISTS (
        SELECT 1
        FROM sprawa_decyzje_zatwierdzone AS decyzja
        WHERE decyzja.rodzaj = 'inna'
          AND decyzja.dotyczy_tabela = 'oferta_nieruchomosc_przegladania'
          AND decyzja.dotyczy_id = przegladanie.id
      )
  )
BEGIN
  SELECT RAISE(ABORT, 'przy rozbieżności z księgą ogłoszenie wymaga zatwierdzonej decyzji inna, która wskazuje przeglądanie');
END;

CREATE TRIGGER oferta_ogloszenia_wersje_bez_zmian
BEFORE UPDATE ON oferta_ogloszenia_wersje
BEGIN
  SELECT RAISE(ABORT, 'wersji ogłoszenia się nie zmienia; zmiana treści to nowa wersja');
END;

-- Statystyki podaje portal o opublikowanym ogłoszeniu.
CREATE TRIGGER oferta_ogloszenia_statystyki_po_publikacji
BEFORE INSERT ON oferta_ogloszenia_statystyki
WHEN NOT EXISTS (
  SELECT 1
  FROM oferta_ogloszenia_wpisy
  WHERE id = NEW.ogloszenie_id
    AND opublikowano <= NEW.odczytano
)
BEGIN
  SELECT RAISE(ABORT, 'statystyki odczytuje się dla ogłoszenia po jego publikacji');
END;

CREATE TRIGGER oferta_ogloszenia_statystyki_bez_zmian
BEFORE UPDATE ON oferta_ogloszenia_statystyki
BEGIN
  SELECT RAISE(ABORT, 'odczytu statystyk się nie zmienia; nowy odczyt to nowy wpis');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER oferta_ogloszenia_wpisy_bez_zastepowania
BEFORE INSERT ON oferta_ogloszenia_wpisy
WHEN EXISTS (
  SELECT 1
  FROM oferta_ogloszenia_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_ogloszenia_wpisy_bez_usuwania
BEFORE DELETE ON oferta_ogloszenia_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER oferta_ogloszenia_wersje_bez_zastepowania
BEFORE INSERT ON oferta_ogloszenia_wersje
WHEN EXISTS (
    SELECT 1
    FROM oferta_ogloszenia_wersje
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM oferta_ogloszenia_wersje
    WHERE ogloszenie_id = NEW.ogloszenie_id
      AND numer = NEW.numer
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_ogloszenia_wersje_bez_usuwania
BEFORE DELETE ON oferta_ogloszenia_wersje
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER oferta_ogloszenia_statystyki_bez_zastepowania
BEFORE INSERT ON oferta_ogloszenia_statystyki
WHEN EXISTS (
  SELECT 1
  FROM oferta_ogloszenia_statystyki
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_ogloszenia_statystyki_bez_usuwania
BEFORE DELETE ON oferta_ogloszenia_statystyki
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
    ],
};

/// The values of column `wpisy.dotyczy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyDotyczy {
    Sprzedaz,
    Najem,
}

impl WpisyDotyczy {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyDotyczy] = &[WpisyDotyczy::Sprzedaz, WpisyDotyczy::Najem];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyDotyczy::Sprzedaz => "sprzedaz",
            WpisyDotyczy::Najem => "najem",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyDotyczy {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "sprzedaz" => Ok(WpisyDotyczy::Sprzedaz),
                    "najem" => Ok(WpisyDotyczy::Najem),
                    other => Err(format!("{other} is no value of WpisyDotyczy")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyDotyczy {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.kanal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyKanal {
    Portal,
    StronaBiura,
}

impl WpisyKanal {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyKanal] = &[WpisyKanal::Portal, WpisyKanal::StronaBiura];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyKanal::Portal => "portal",
            WpisyKanal::StronaBiura => "strona_biura",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyKanal {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "portal" => Ok(WpisyKanal::Portal),
                    "strona_biura" => Ok(WpisyKanal::StronaBiura),
                    other => Err(format!("{other} is no value of WpisyKanal")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyKanal {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `statystyki_ogloszen`.
#[derive(Debug, Clone, PartialEq)]
pub struct StatystykiOgloszenRow {
    pub id: i64,
    pub ogloszenie_id: String,
    pub odczytano: String,
    pub wyswietlenia: i64,
    pub zapytania: i64,
}

impl StatystykiOgloszenRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(StatystykiOgloszenRow {
            id: row.get("id")?,
            ogloszenie_id: row.get("ogloszenie_id")?,
            odczytano: row.get("odczytano")?,
            wyswietlenia: row.get("wyswietlenia")?,
            zapytania: row.get("zapytania")?,
        })
    }
}

/// Odczyty statystyk portali, najnowsze pierwsze.
pub fn statystyki_ogloszen(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<StatystykiOgloszenRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_ogloszenia_statystyki
ORDER BY odczytano DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(StatystykiOgloszenRow::from_row).collect()
}

/// The parameters of `wpisz_statystyki_ogloszenia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszStatystykiOgloszeniaParams {
    pub ogloszenie_id: String,
    pub odczytano: String,
    pub wyswietlenia: i64,
    pub zapytania: i64,
}

/// Liczby wyświetleń i zapytań podane przez portal, z chwilą odczytu.
pub fn wpisz_statystyki_ogloszenia(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszStatystykiOgloszeniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO oferta_ogloszenia_statystyki (ogloszenie_id, odczytano, wyswietlenia, zapytania)
VALUES (:ogloszenie_id, :odczytano, :wyswietlenia, :zapytania)"#,
                parameters: Vec::from([
                    (
                        "ogloszenie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.ogloszenie_id),
                    ),
                    (
                        "odczytano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odczytano),
                    ),
                    (
                        "wyswietlenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wyswietlenia),
                    ),
                    (
                        "zapytania",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zapytania),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `wersje_ogloszen`.
#[derive(Debug, Clone, PartialEq)]
pub struct WersjeOgloszenRow {
    pub id: i64,
    pub ogloszenie_id: String,
    pub numer: i64,
    pub od: String,
    pub tytul: String,
    pub cena: i64,
    pub lokalizacja: String,
    pub wskazniki_energetyczne: Option<String>,
    pub tresc_sciezka: String,
    pub tresc_skrot: String,
}

impl WersjeOgloszenRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WersjeOgloszenRow {
            id: row.get("id")?,
            ogloszenie_id: row.get("ogloszenie_id")?,
            numer: row.get("numer")?,
            od: row.get("od")?,
            tytul: row.get("tytul")?,
            cena: row.get("cena")?,
            lokalizacja: row.get("lokalizacja")?,
            wskazniki_energetyczne: row.get("wskazniki_energetyczne")?,
            tresc_sciezka: row.get("tresc_sciezka")?,
            tresc_skrot: row.get("tresc_skrot")?,
        })
    }
}

/// Wersje treści ogłoszeń od najstarszej.
pub fn wersje_ogloszen(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WersjeOgloszenRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_ogloszenia_wersje
ORDER BY ogloszenie_id, numer"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(WersjeOgloszenRow::from_row).collect()
}

/// A row of `najnowsze_wersje_ogloszen`.
#[derive(Debug, Clone, PartialEq)]
pub struct NajnowszeWersjeOgloszenRow {
    pub id: i64,
    pub ogloszenie_id: String,
    pub numer: i64,
    pub od: String,
    pub tytul: String,
    pub cena: i64,
    pub lokalizacja: String,
    pub wskazniki_energetyczne: Option<String>,
    pub tresc_sciezka: String,
    pub tresc_skrot: String,
}

impl NajnowszeWersjeOgloszenRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(NajnowszeWersjeOgloszenRow {
            id: row.get("id")?,
            ogloszenie_id: row.get("ogloszenie_id")?,
            numer: row.get("numer")?,
            od: row.get("od")?,
            tytul: row.get("tytul")?,
            cena: row.get("cena")?,
            lokalizacja: row.get("lokalizacja")?,
            wskazniki_energetyczne: row.get("wskazniki_energetyczne")?,
            tresc_sciezka: row.get("tresc_sciezka")?,
            tresc_skrot: row.get("tresc_skrot")?,
        })
    }
}

/// Najnowsza wersja treści każdego ogłoszenia.
pub fn najnowsze_wersje_ogloszen(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<NajnowszeWersjeOgloszenRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_ogloszenia_wersje_najnowsze
ORDER BY ogloszenie_id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(NajnowszeWersjeOgloszenRow::from_row).collect()
}

/// The parameters of `dopisz_wersje_ogloszenia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DopiszWersjeOgloszeniaParams {
    pub ogloszenie_id: String,
    pub numer: i64,
    pub od: String,
    pub tytul: String,
    pub cena: i64,
    pub lokalizacja: String,
    pub wskazniki_energetyczne: Option<String>,
    pub tresc_sciezka: String,
    pub tresc_skrot: String,
}

/// Nowa wersja treści z kolejnym numerem; wersja opublikowanego ogłoszenia przechodzi reguły
/// publikacji.
pub fn dopisz_wersje_ogloszenia(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszWersjeOgloszeniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO oferta_ogloszenia_wersje (ogloszenie_id, numer, od, tytul, cena, lokalizacja,
  wskazniki_energetyczne, tresc_sciezka, tresc_skrot)
VALUES (:ogloszenie_id, :numer, :od, :tytul, :cena, :lokalizacja, :wskazniki_energetyczne,
  :tresc_sciezka, :tresc_skrot)"#,
                parameters: Vec::from([
                    (
                        "ogloszenie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.ogloszenie_id),
                    ),
                    ("numer", xmeta_room::ToSqlValue::to_sql_value(&params.numer)),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    ("tytul", xmeta_room::ToSqlValue::to_sql_value(&params.tytul)),
                    ("cena", xmeta_room::ToSqlValue::to_sql_value(&params.cena)),
                    (
                        "lokalizacja",
                        xmeta_room::ToSqlValue::to_sql_value(&params.lokalizacja),
                    ),
                    (
                        "wskazniki_energetyczne",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.wskazniki_energetyczne,
                        ),
                    ),
                    (
                        "tresc_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.tresc_sciezka),
                    ),
                    (
                        "tresc_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.tresc_skrot),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `ogloszenia`.
#[derive(Debug, Clone, PartialEq)]
pub struct OgloszeniaRow {
    pub id: String,
    pub dotyczy: WpisyDotyczy,
    pub kanal: WpisyKanal,
    pub portal: Option<String>,
    pub numer_ogloszenia: Option<String>,
    pub opublikowano: Option<String>,
    pub zdjeto: Option<String>,
    pub powod_zdjecia: Option<String>,
}

impl OgloszeniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(OgloszeniaRow {
            id: row.get("id")?,
            dotyczy: row.get("dotyczy")?,
            kanal: row.get("kanal")?,
            portal: row.get("portal")?,
            numer_ogloszenia: row.get("numer_ogloszenia")?,
            opublikowano: row.get("opublikowano")?,
            zdjeto: row.get("zdjeto")?,
            powod_zdjecia: row.get("powod_zdjecia")?,
        })
    }
}

/// Ogłoszenia oferty w kolejności wpisania.
pub fn ogloszenia(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<OgloszeniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_ogloszenia_wpisy
ORDER BY rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(OgloszeniaRow::from_row).collect()
}

/// The parameters of `przygotuj_ogloszenie`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzygotujOgloszenieParams {
    pub id: String,
    pub dotyczy: WpisyDotyczy,
    pub kanal: WpisyKanal,
    pub portal: Option<String>,
}

/// Projekt ogłoszenia w jednym kanale; identyfikator (UUID) nadaje aplikacja, bo wskazują go
/// wersje, decyzja o pierwszej publikacji i katalog ofert.
pub fn przygotuj_ogloszenie(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzygotujOgloszenieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO oferta_ogloszenia_wpisy (id, dotyczy, kanal, portal)
VALUES (:id, :dotyczy, :kanal, :portal)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("dotyczy", xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy)),
                    ("kanal", xmeta_room::ToSqlValue::to_sql_value(&params.kanal)),
                    ("portal", xmeta_room::ToSqlValue::to_sql_value(&params.portal)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_publikacje_ogloszenia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPublikacjeOgloszeniaParams {
    pub opublikowano: String,
    pub id: String,
}

/// Ogłoszenie wyszło: najnowsza wersja przechodzi reguły publikacji, a pierwsza publikacja
/// w teczce ma zatwierdzoną decyzję pośrednika.
pub fn odnotuj_publikacje_ogloszenia(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPublikacjeOgloszeniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE oferta_ogloszenia_wpisy
SET opublikowano = :opublikowano
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "opublikowano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.opublikowano),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_numer_ogloszenia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujNumerOgloszeniaParams {
    pub numer_ogloszenia: String,
    pub id: String,
}

/// Numer nadany przez portal; Kierownik pośrednictwa przepisuje go do katalogu ofert.
pub fn odnotuj_numer_ogloszenia(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujNumerOgloszeniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE oferta_ogloszenia_wpisy
SET numer_ogloszenia = :numer_ogloszenia
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "numer_ogloszenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.numer_ogloszenia),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `zdejmij_ogloszenie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZdejmijOgloszenieParams {
    pub zdjeto: String,
    pub powod_zdjecia: String,
    pub id: String,
}

/// Zdjęcie ogłoszenia z powodem, np. przyjęta oferta albo koniec umowy pośrednictwa; zdjęte
/// ogłoszenie nie wraca.
pub fn zdejmij_ogloszenie(
    room: &impl xmeta_room::MetaRoom,
    params: &ZdejmijOgloszenieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE oferta_ogloszenia_wpisy
SET zdjeto = :zdjeto,
  powod_zdjecia = :powod_zdjecia
WHERE id = :id"#,
                parameters: Vec::from([
                    ("zdjeto", xmeta_room::ToSqlValue::to_sql_value(&params.zdjeto)),
                    (
                        "powod_zdjecia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.powod_zdjecia),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
