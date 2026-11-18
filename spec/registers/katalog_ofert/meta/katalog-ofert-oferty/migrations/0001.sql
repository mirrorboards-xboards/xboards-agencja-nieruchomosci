-- Katalog ofert biura: każda oferta sprzedaży i najmu długoterminowego z teczki oferty, tylko
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
