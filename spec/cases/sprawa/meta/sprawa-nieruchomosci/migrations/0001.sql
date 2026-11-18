-- Oznaczenie nieruchomości teczki: lokal, dom albo działka, z księgą wieczystą i działkami.
-- Korzystają z niego typy oferty, transakcji, najmu i lokalu, pokój wymiany i dokumenty („czego
-- dotyczy”). Poszukiwanie go nie używa: obejrzane nieruchomości to jego propozycje.
--
-- Nieruchomość wpisuje agent sprawy. Ocenę stanu prawnego robi Analityk stanu prawnego w typie
-- sprawy; rdzeń trzyma tylko oznaczenie.
CREATE TABLE sprawa_nieruchomosci_wpisy (
  id TEXT PRIMARY KEY,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('lokal_mieszkalny', 'lokal_uzytkowy', 'dom', 'dzialka', 'garaz', 'miejsce_postojowe', 'pomieszczenie_przynalezne')),
  prawo TEXT NOT NULL CHECK (prawo IN ('wlasnosc', 'odrebna_wlasnosc_lokalu', 'spoldzielcze_wlasnosciowe_prawo_do_lokalu', 'uzytkowanie_wieczyste', 'udzial_we_wspolwlasnosci', 'inne')),
  miejscowosc TEXT NOT NULL,
  gmina TEXT NOT NULL,
  ulica TEXT,
  numer_budynku TEXT,
  numer_lokalu TEXT,
  kod_pocztowy TEXT,
  -- Numer księgi wieczystej, np. WA1M/00012345/6. Lokal bez księgi, np. część spółdzielczych
  -- praw do lokalu, ma kw_brak = 1.
  numer_kw TEXT,
  kw_brak INTEGER NOT NULL DEFAULT 0 CHECK (kw_brak IN (0, 1)),
  -- Powierzchnia użytkowa w setnych metra kwadratowego (48,20 m² to 4820).
  powierzchnia_uzytkowa INTEGER CHECK (powierzchnia_uzytkowa > 0),
  liczba_pokoi INTEGER CHECK (liczba_pokoi >= 0),
  pietro INTEGER,
  -- Lokal, do którego należy pomieszczenie przynależne, np. piwnica.
  nadrzedna_id TEXT REFERENCES sprawa_nieruchomosci_wpisy (id),

  CONSTRAINT numer_kw_albo_brak_ksiegi CHECK (
    (kw_brak = 1 AND numer_kw IS NULL)
    OR (kw_brak = 0 AND numer_kw IS NOT NULL AND numer_kw GLOB '[A-Z][A-Z][0-9][A-Z]/[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]/[0-9]')
  ),
  CONSTRAINT pomieszczenie_przynalezne_wskazuje_lokal CHECK (
    (rodzaj = 'pomieszczenie_przynalezne') = (nadrzedna_id IS NOT NULL)
  )
) STRICT;

-- Działki ewidencyjne nieruchomości: obręb, numer i powierzchnia.
CREATE TABLE sprawa_nieruchomosci_dzialki (
  id INTEGER PRIMARY KEY,
  nieruchomosc_id TEXT NOT NULL REFERENCES sprawa_nieruchomosci_wpisy (id),
  obreb TEXT NOT NULL,
  numer_dzialki TEXT NOT NULL,
  -- Powierzchnia w metrach kwadratowych, jak w ewidencji gruntów.
  powierzchnia INTEGER CHECK (powierzchnia > 0),

  UNIQUE (nieruchomosc_id, obreb, numer_dzialki)
) STRICT;

-- Pomieszczenie przynależne należy do lokalu, nie do domu ani do innej piwnicy.
CREATE TRIGGER sprawa_nieruchomosci_nadrzedna_to_lokal
BEFORE INSERT ON sprawa_nieruchomosci_wpisy
WHEN NEW.nadrzedna_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_nieruchomosci_wpisy
    WHERE id = NEW.nadrzedna_id
      AND rodzaj IN ('lokal_mieszkalny', 'lokal_uzytkowy')
  )
BEGIN
  SELECT RAISE(ABORT, 'pomieszczenie przynależne wskazuje lokal');
END;

CREATE TRIGGER sprawa_nieruchomosci_zmieniona_nadrzedna_to_lokal
BEFORE UPDATE OF nadrzedna_id ON sprawa_nieruchomosci_wpisy
WHEN NEW.nadrzedna_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_nieruchomosci_wpisy
    WHERE id = NEW.nadrzedna_id
      AND rodzaj IN ('lokal_mieszkalny', 'lokal_uzytkowy')
  )
BEGIN
  SELECT RAISE(ABORT, 'pomieszczenie przynależne wskazuje lokal');
END;

-- Od rodzaju zależą reguły pomieszczenia przynależnego i to, co typ sprawy wie o nieruchomości;
-- zmiana lokalu w dom ominęłaby regułę, że pomieszczenie przynależne wskazuje lokal.
CREATE TRIGGER sprawa_nieruchomosci_rodzaj_niezmienny
BEFORE UPDATE OF rodzaj ON sprawa_nieruchomosci_wpisy
WHEN NEW.rodzaj IS NOT OLD.rodzaj
BEGIN
  SELECT RAISE(ABORT, 'rodzaj nieruchomości się nie zmienia; nieruchomość innego rodzaju to nowy wiersz');
END;

-- Działkę wolno usunąć jako poprawkę pomyłki (D14); nieruchomość zostaje.
-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_nieruchomosci_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_nieruchomosci_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_nieruchomosci_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_nieruchomosci_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_nieruchomosci_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
