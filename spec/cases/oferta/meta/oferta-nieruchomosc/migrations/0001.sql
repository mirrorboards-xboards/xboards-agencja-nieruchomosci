-- Opis nieruchomości oferty ponad oznaczenie z rdzenia (sprawa_nieruchomosci_wpisy) i wyniki
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
