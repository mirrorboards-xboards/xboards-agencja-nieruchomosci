-- Spis spraw działu i ich znaki rozpoznawcze, które kierownik przekazuje Recepcji. Dzięki nim
-- Recepcja przypisuje wiadomość do sprawy bez czytania teczek. Recepcja zmienia swój spis tylko na
-- podstawie tego, co przekazał kierownik, więc kierownik pamięta, co i kiedy przekazał.
--
-- Wiersz mówi, w jakim stanie jest sprawa w spisie, a kolumny „przekazano” mówią, czy Recepcja ten
-- stan już dostała. Zmiana oznaczenia, agenta albo kancelarii czeka na ponowne przekazanie.
-- Numerów ofert i ogłoszeń spis nie trzyma, bo Recepcja czyta je w katalogu ofert.
CREATE TABLE kierownik_dzialu_spis_sprawy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_dzialu_spis_sprawy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Pokój teczki, tekstem.
  teczka_odnosnik TEXT NOT NULL UNIQUE CHECK (teczka_odnosnik <> ''),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('oferta', 'poszukiwanie', 'transakcja', 'najem', 'lokal_krotkoterminowy')),
  oznaczenie TEXT NOT NULL CHECK (oznaczenie <> ''),
  -- Tożsamość agenta, któremu Recepcja oddaje wiadomości w sprawie.
  agent TEXT NOT NULL,
  -- Dzień przyjęcia sprawy z metryki teczki; Recepcja pokazuje go w swoim spisie.
  otwarto TEXT NOT NULL CHECK (otwarto <> ''),
  -- Czy kancelaria transakcji pracuje na XNotariuszu; wtedy pisze do biura tylko w pokoju wymiany
  -- (rozstrzygnięcie biura R5).
  kancelaria_xnotariusz INTEGER CHECK (kancelaria_xnotariusz IN (0, 1)),
  przekazano TEXT,
  zamknieto TEXT,
  zamkniecie_przekazano TEXT,

  CONSTRAINT kancelaria_tylko_przy_transakcji CHECK ((rodzaj = 'transakcja') = (kancelaria_xnotariusz IS NOT NULL)),
  CONSTRAINT zamkniecie_przekazane_po_zamknieciu CHECK (
    zamkniecie_przekazano IS NULL
    OR (zamknieto IS NOT NULL AND zamkniecie_przekazano >= zamknieto)
  )
) STRICT;

CREATE TABLE kierownik_dzialu_spis_znaki (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_dzialu_spis_znaki_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  sprawa_id INTEGER NOT NULL REFERENCES kierownik_dzialu_spis_sprawy (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('adres_nieruchomosci', 'oznaczenie_transakcji', 'adres_poczty_klienta')),
  wartosc TEXT NOT NULL CHECK (wartosc <> ''),
  dodano TEXT NOT NULL,
  przekazano TEXT,
  wycofano TEXT,
  wycofanie_przekazano TEXT,

  CONSTRAINT przekazanie_nie_przed_dodaniem CHECK (przekazano IS NULL OR przekazano >= dodano),
  CONSTRAINT wycofanie_nie_przed_dodaniem CHECK (wycofano IS NULL OR wycofano >= dodano),
  -- Wycofanie przekazuje się tylko wtedy, gdy Recepcja dostała znak.
  CONSTRAINT wycofanie_przekazane_po_wycofaniu_przekazanego_znaku CHECK (
    wycofanie_przekazano IS NULL
    OR (wycofano IS NOT NULL AND przekazano IS NOT NULL AND wycofanie_przekazano >= wycofano)
  )
) STRICT;

-- Sprawy -----------------------------------------------------------------------------------------

CREATE TRIGGER kierownik_dzialu_spis_sprawa_powstaje_otwarta
BEFORE INSERT ON kierownik_dzialu_spis_sprawy
WHEN NEW.przekazano IS NOT NULL
  OR NEW.zamknieto IS NOT NULL
  OR NEW.zamkniecie_przekazano IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'sprawa powstaje w spisie otwarta i jeszcze nieprzekazana');
END;

-- Zamknięta sprawa zostaje w spisie, bo po latach może w niej przyjść pismo; zmienia się w niej już
-- tylko przekazanie zamknięcia, raz.
CREATE TRIGGER kierownik_dzialu_spis_sprawy_bez_zmian
BEFORE UPDATE ON kierownik_dzialu_spis_sprawy
WHEN NEW.id IS NOT OLD.id
  OR NEW.teczka_odnosnik IS NOT OLD.teczka_odnosnik
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.otwarto IS NOT OLD.otwarto
  OR (OLD.zamknieto IS NOT NULL AND (
    NEW.oznaczenie IS NOT OLD.oznaczenie
    OR NEW.agent IS NOT OLD.agent
    OR NEW.kancelaria_xnotariusz IS NOT OLD.kancelaria_xnotariusz
    OR NEW.przekazano IS NOT OLD.przekazano
    OR NEW.zamknieto IS NOT OLD.zamknieto
    OR OLD.zamkniecie_przekazano IS NOT NULL
  ))
BEGIN
  SELECT RAISE(ABORT, 'teczki ani rodzaju sprawy w spisie się nie zmienia, a zamkniętej sprawy tylko raz przekazuje się zamknięcie');
END;

-- Znaki ------------------------------------------------------------------------------------------

CREATE TRIGGER kierownik_dzialu_spis_znak_powstaje_nieprzekazany
BEFORE INSERT ON kierownik_dzialu_spis_znaki
WHEN NEW.przekazano IS NOT NULL
  OR NEW.wycofano IS NOT NULL
  OR NEW.wycofanie_przekazano IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'znak powstaje niewycofany i jeszcze nieprzekazany');
END;

-- Znaku się nie zmienia; wolno raz odnotować przekazanie, wycofanie i przekazanie wycofania.
CREATE TRIGGER kierownik_dzialu_spis_znaki_bez_zmian
BEFORE UPDATE ON kierownik_dzialu_spis_znaki
WHEN NEW.id IS NOT OLD.id
  OR NEW.sprawa_id IS NOT OLD.sprawa_id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.wartosc IS NOT OLD.wartosc
  OR NEW.dodano IS NOT OLD.dodano
  OR (OLD.przekazano IS NOT NULL AND NEW.przekazano IS NOT OLD.przekazano)
  OR (OLD.wycofano IS NOT NULL AND NEW.wycofano IS NOT OLD.wycofano)
  OR (OLD.wycofanie_przekazano IS NOT NULL AND NEW.wycofanie_przekazano IS NOT OLD.wycofanie_przekazano)
BEGIN
  SELECT RAISE(ABORT, 'znaku się nie zmienia; wolno raz odnotować przekazanie, wycofanie i przekazanie wycofania');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers; także sprawę tej samej
-- teczki. Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER kierownik_dzialu_spis_sprawy_bez_zastepowania
BEFORE INSERT ON kierownik_dzialu_spis_sprawy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_dzialu_spis_sprawy
  WHERE id = NEW.id
    OR teczka_odnosnik = NEW.teczka_odnosnik
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER kierownik_dzialu_spis_znaki_bez_zastepowania
BEFORE INSERT ON kierownik_dzialu_spis_znaki
WHEN EXISTS (
  SELECT 1
  FROM kierownik_dzialu_spis_znaki
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: zamknięta sprawa i wycofany znak zostają w spisie.
CREATE TRIGGER kierownik_dzialu_spis_sprawy_bez_usuwania
BEFORE DELETE ON kierownik_dzialu_spis_sprawy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER kierownik_dzialu_spis_znaki_bez_usuwania
BEFORE DELETE ON kierownik_dzialu_spis_znaki
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); znaku się nie usuwa, tylko wycofuje');
END;
