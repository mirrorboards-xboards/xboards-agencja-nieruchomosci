-- Notatki przekazania: jedyny sposób, w jaki agenci rozmawiają ze sobą (zasada „Teczka jest
-- wspólnym stanem sprawy”). Notatka jest prośbą od stanowiska do stanowiska; trafia też do
-- następcy agenta. Gdy w teczce pracuje dwóch Kontrolerów (R1), notatka do Kontrolera wskazuje
-- klienta, którego Kontrolera dotyczy. Pisze ją każdy członek teczki, który ma prawo pisać.
--
-- Treść jest zapisem tego, kto komu co przekazał, więc się nie zmienia; sprostowanie to nowa
-- notatka w odpowiedzi. Stan zmienia się raz: notatka zostaje wykonana albo odrzucona z powodem.
CREATE TABLE sprawa_notatki_wpisy (
  id TEXT PRIMARY KEY,
  od TEXT NOT NULL,
  -- Stanowiska: nazwy ich folderów pod positions/.
  od_stanowiska TEXT NOT NULL CHECK (od_stanowiska IN ('analityk_kredytowy', 'analityk_stanu_prawnego', 'kierownik_biura', 'kierownik_kontroli', 'kierownik_najmu', 'kierownik_posrednictwa', 'kierownik_transakcji', 'konsjerz', 'kontroler', 'koordynator_operacji', 'koordynator_transakcji', 'opiekun_najmu', 'opiekun_oferty', 'opiekun_poszukujacego', 'recepcja', 'zarzadca_obiektu')),
  do_stanowiska TEXT NOT NULL CHECK (do_stanowiska IN ('analityk_kredytowy', 'analityk_stanu_prawnego', 'kierownik_biura', 'kierownik_kontroli', 'kierownik_najmu', 'kierownik_posrednictwa', 'kierownik_transakcji', 'konsjerz', 'kontroler', 'koordynator_operacji', 'koordynator_transakcji', 'opiekun_najmu', 'opiekun_oferty', 'opiekun_poszukujacego', 'recepcja', 'zarzadca_obiektu')),
  -- Klient, którego Kontrolera dotyczy notatka, gdy w teczce jest dwóch.
  do_klienta_uczestnik_id TEXT REFERENCES sprawa_uczestnicy_wpisy (id),
  tresc TEXT NOT NULL,
  utworzono TEXT NOT NULL,
  do_kiedy TEXT,
  w_odpowiedzi_na TEXT REFERENCES sprawa_notatki_wpisy (id),
  stan TEXT NOT NULL CHECK (stan IN ('otwarta', 'wykonana', 'odrzucona')),
  powod TEXT,

  CONSTRAINT klient_tylko_przy_kontrolerze CHECK (do_klienta_uczestnik_id IS NULL OR do_stanowiska = 'kontroler'),
  CONSTRAINT odrzucona_ma_powod CHECK ((stan = 'odrzucona') = (powod IS NOT NULL))
) STRICT;

-- Wiersze teczki (rdzenia albo typu sprawy), których notatka dotyczy; bez klucza obcego.
CREATE TABLE sprawa_notatki_wiersze (
  -- Klucz nadany przez bazę jest dodatni; zob. sprawa_notatki_wiersze_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  notatka_id TEXT NOT NULL REFERENCES sprawa_notatki_wpisy (id),
  dotyczy_tabela TEXT NOT NULL,
  dotyczy_id TEXT NOT NULL,

  UNIQUE (notatka_id, dotyczy_tabela, dotyczy_id)
) STRICT;

CREATE TRIGGER sprawa_notatki_powstaje_otwarta
BEFORE INSERT ON sprawa_notatki_wpisy
WHEN NEW.stan <> 'otwarta'
BEGIN
  SELECT RAISE(ABORT, 'notatka powstaje jako otwarta');
END;

CREATE TRIGGER sprawa_notatki_tresc_niezmienna
BEFORE UPDATE ON sprawa_notatki_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.od IS NOT OLD.od
  OR NEW.od_stanowiska IS NOT OLD.od_stanowiska
  OR NEW.do_stanowiska IS NOT OLD.do_stanowiska
  OR NEW.do_klienta_uczestnik_id IS NOT OLD.do_klienta_uczestnik_id
  OR NEW.tresc IS NOT OLD.tresc
  OR NEW.utworzono IS NOT OLD.utworzono
  OR NEW.do_kiedy IS NOT OLD.do_kiedy
  OR NEW.w_odpowiedzi_na IS NOT OLD.w_odpowiedzi_na
BEGIN
  SELECT RAISE(ABORT, 'treść notatki się nie zmienia; sprostowanie to nowa notatka w odpowiedzi');
END;

CREATE TRIGGER sprawa_notatki_stan_zmienia_sie_raz
BEFORE UPDATE ON sprawa_notatki_wpisy
WHEN OLD.stan <> 'otwarta'
BEGIN
  SELECT RAISE(ABORT, 'stan notatki zmienia się raz: z otwartej na wykonaną albo odrzuconą');
END;

CREATE TRIGGER sprawa_notatki_wiersze_bez_zmian
BEFORE UPDATE ON sprawa_notatki_wiersze
BEGIN
  SELECT RAISE(ABORT, 'wiersza notatki się nie zmienia; inny wiersz to nowe wskazanie');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_notatki_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_notatki_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_notatki_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_notatki_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_notatki_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER sprawa_notatki_wiersze_bez_zastepowania
BEFORE INSERT ON sprawa_notatki_wiersze
WHEN EXISTS (
    SELECT 1
    FROM sprawa_notatki_wiersze
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM sprawa_notatki_wiersze
    WHERE notatka_id = NEW.notatka_id
      AND dotyczy_tabela = NEW.dotyczy_tabela
      AND dotyczy_id = NEW.dotyczy_id
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_notatki_wiersze_bez_usuwania
BEFORE DELETE ON sprawa_notatki_wiersze
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
