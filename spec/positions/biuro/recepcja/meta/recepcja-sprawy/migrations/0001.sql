-- Spis spraw i znaki rozpoznawcze, po których Recepcja przypisuje wiadomości do spraw. Spis i znaki
-- Recepcja zmienia tylko na podstawie tego, co przekazał kierownik działu (jego kierownik-dzialu-spis), a nie
-- na podstawie treści wiadomości, bo ciąg wiadomości mógłby ją w ten sposób przekierować (dryf).
-- Dlatego każdy wiersz wskazuje, kto i kiedy go przekazał, a każda zmiana sprawy przychodzi
-- z nowym przekazaniem.
--
-- Zamknięta sprawa zostaje w spisie, bo po latach może w niej przyjść pismo, które ma obudzić jej
-- agenta. Znak, który przestał pasować, wycofuje się, a nie usuwa. Numerów ofert i ogłoszeń spis
-- nie trzyma, bo Recepcja czyta je w katalogu ofert. Etapu sprawy ani tego, czy jest wstrzymana,
-- spis nie zna.
CREATE TABLE recepcja_sprawy_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. recepcja_sprawy_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Pokój teczki, tekstem.
  teczka_odnosnik TEXT NOT NULL UNIQUE CHECK (teczka_odnosnik <> ''),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('oferta', 'poszukiwanie', 'transakcja', 'najem', 'lokal_krotkoterminowy')),
  oznaczenie TEXT NOT NULL CHECK (oznaczenie <> ''),
  -- Tożsamość agenta, któremu Recepcja oddaje wiadomości w sprawie.
  agent TEXT NOT NULL CHECK (agent <> ''),
  stan TEXT NOT NULL CHECK (stan IN ('otwarta', 'zamknieta')),
  otwarto TEXT NOT NULL,
  zamknieto TEXT,
  -- Czy kancelaria transakcji pracuje na XNotariuszu; wtedy pisze do biura tylko w pokoju wymiany
  -- (rozstrzygnięcie biura R5), a jej wiadomość spoza niego jest podejrzana.
  kancelaria_xnotariusz INTEGER CHECK (kancelaria_xnotariusz IN (0, 1)),
  -- Kierownik działu, który przekazał ostatni stan sprawy, i chwila przekazania.
  przekazal TEXT NOT NULL CHECK (przekazal <> ''),
  przekazano TEXT NOT NULL,

  CONSTRAINT zamknieta_z_data_zamkniecia CHECK ((stan = 'zamknieta') = (zamknieto IS NOT NULL)),
  CONSTRAINT zamkniecie_nie_przed_otwarciem CHECK (zamknieto IS NULL OR zamknieto >= otwarto),
  CONSTRAINT kancelaria_tylko_przy_transakcji CHECK ((rodzaj = 'transakcja') = (kancelaria_xnotariusz IS NOT NULL))
) STRICT;

CREATE TABLE recepcja_sprawy_znaki (
  -- Klucz nadany przez bazę jest dodatni; zob. recepcja_sprawy_znaki_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  sprawa_id INTEGER NOT NULL REFERENCES recepcja_sprawy_wpisy (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('adres_nieruchomosci', 'oznaczenie_transakcji', 'adres_poczty_klienta')),
  wartosc TEXT NOT NULL CHECK (wartosc <> ''),
  przekazal TEXT NOT NULL CHECK (przekazal <> ''),
  dodano TEXT NOT NULL,
  wycofano TEXT,

  CONSTRAINT wycofanie_nie_przed_dodaniem CHECK (wycofano IS NULL OR wycofano >= dodano)
) STRICT;

-- Sprawy -----------------------------------------------------------------------------------------

CREATE TRIGGER recepcja_sprawy_powstaje_otwarta
BEFORE INSERT ON recepcja_sprawy_wpisy
WHEN NEW.stan <> 'otwarta'
BEGIN
  SELECT RAISE(ABORT, 'sprawa powstaje w spisie otwarta; zamknięcie przekazuje kierownik działu');
END;

-- Każda zmiana sprawy to nowe przekazanie od kierownika działu, a nie wniosek z treści wiadomości.
CREATE TRIGGER recepcja_sprawy_zmiana_z_przekazania
BEFORE UPDATE ON recepcja_sprawy_wpisy
WHEN NEW.przekazano <= OLD.przekazano
BEGIN
  SELECT RAISE(ABORT, 'sprawę w spisie zmienia tylko nowe przekazanie kierownika działu');
END;

-- Zamknięta sprawa zostaje w spisie bez zmian.
CREATE TRIGGER recepcja_sprawy_bez_zmian
BEFORE UPDATE ON recepcja_sprawy_wpisy
WHEN OLD.stan = 'zamknieta'
  OR NEW.id IS NOT OLD.id
  OR NEW.teczka_odnosnik IS NOT OLD.teczka_odnosnik
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.otwarto IS NOT OLD.otwarto
BEGIN
  SELECT RAISE(ABORT, 'teczki, rodzaju ani daty otwarcia sprawy się nie zmienia, a zamkniętej sprawy wcale');
END;

-- Znaki ------------------------------------------------------------------------------------------

CREATE TRIGGER recepcja_sprawy_znak_powstaje_niewycofany
BEFORE INSERT ON recepcja_sprawy_znaki
WHEN NEW.wycofano IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'znak powstaje niewycofany; wycofanie przekazuje kierownik działu');
END;

CREATE TRIGGER recepcja_sprawy_znaki_bez_zmian
BEFORE UPDATE ON recepcja_sprawy_znaki
WHEN OLD.wycofano IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.sprawa_id IS NOT OLD.sprawa_id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.wartosc IS NOT OLD.wartosc
  OR NEW.przekazal IS NOT OLD.przekazal
  OR NEW.dodano IS NOT OLD.dodano
BEGIN
  SELECT RAISE(ABORT, 'znaku się nie zmienia; wolno go raz wycofać');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers; także sprawę tej samej
-- teczki. Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER recepcja_sprawy_wpisy_bez_zastepowania
BEFORE INSERT ON recepcja_sprawy_wpisy
WHEN EXISTS (
  SELECT 1
  FROM recepcja_sprawy_wpisy
  WHERE id = NEW.id
    OR teczka_odnosnik = NEW.teczka_odnosnik
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER recepcja_sprawy_znaki_bez_zastepowania
BEFORE INSERT ON recepcja_sprawy_znaki
WHEN EXISTS (
  SELECT 1
  FROM recepcja_sprawy_znaki
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: zamknięta sprawa i wycofany znak zostają w spisie.
CREATE TRIGGER recepcja_sprawy_wpisy_bez_usuwania
BEFORE DELETE ON recepcja_sprawy_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); zamknięta sprawa zostaje w spisie');
END;

CREATE TRIGGER recepcja_sprawy_znaki_bez_usuwania
BEFORE DELETE ON recepcja_sprawy_znaki
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); znaku się nie usuwa, tylko wycofuje');
END;
