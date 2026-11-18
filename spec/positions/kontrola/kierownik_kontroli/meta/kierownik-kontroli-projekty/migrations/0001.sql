-- Projekty zawiadomień do Generalnego Inspektora Informacji Finansowej (art. 74, 86 i 90 AML)
-- i do prokuratora (art. 89 AML), z terminem, stanem i odwołaniem do wpisów w rejestrze kontroli,
-- na których stoją.
--
-- Projekt leży w pamięci, a nie w rejestrze, dopóki właściciel go nie przekaże, bo pamięć czyta
-- tylko on: krąg osób, które wiedzą o projekcie, jest najmniejszy z możliwych (art. 54 ust. 1 AML).
-- Tekst leży w repozytorium XGit pokoju pamięci; wiersz trzyma ścieżkę i skrót. Po przekazaniu
-- ślad zostaje w rejestrze kontroli, a wiersz traci ścieżkę i skrót i wskazuje tylko tamten wpis.
-- Projekt, którego właściciel nie przekaże, zamyka odstąpienie z jego podpisem, bo wierszy się nie
-- usuwa (R9).
CREATE TABLE kierownik_kontroli_projekty_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_kontroli_projekty_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Jak w rejestrze kontroli: art. 74, 86 i 90 AML do Generalnego Inspektora, art. 89 do prokuratora.
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('art74', 'art86', 'art90', 'art89_prokurator')),
  -- Pokój teczki, której projekt dotyczy, tekstem.
  teczka_odnosnik TEXT,
  -- Termin, który policzył program; projekt ma być u właściciela wcześniej.
  termin TEXT NOT NULL,
  utworzono TEXT NOT NULL,
  stan TEXT NOT NULL CHECK (stan IN ('projekt', 'u_wlasciciela', 'przekazany', 'odstapiono')),
  sciezka TEXT,
  skrot TEXT,
  u_wlasciciela_od TEXT,
  przekazano TEXT,
  -- Ślad zawiadomienia w rejestrze kontroli, tekstem.
  rejestr_odnosnik TEXT,
  -- Właściciel, który postanowił projektu nie przekazywać, jego podpis i chwila.
  odstapil TEXT,
  odstapienie_podpis TEXT,
  odstapiono TEXT,

  CONSTRAINT sciezka_razem_ze_skrotem CHECK ((sciezka IS NULL) = (skrot IS NULL)),
  CONSTRAINT projekt_z_plikiem CHECK (stan NOT IN ('projekt', 'u_wlasciciela') OR sciezka IS NOT NULL),
  CONSTRAINT u_wlasciciela_z_data CHECK (stan <> 'u_wlasciciela' OR u_wlasciciela_od IS NOT NULL),
  CONSTRAINT przekazany_z_wpisem_w_rejestrze CHECK (
    (stan = 'przekazany') = (rejestr_odnosnik IS NOT NULL)
    AND (stan = 'przekazany') = (przekazano IS NOT NULL)
  ),
  CONSTRAINT przekazany_bez_tresci CHECK (stan <> 'przekazany' OR sciezka IS NULL),
  CONSTRAINT odstapienie_z_podpisem_wlasciciela CHECK (
    (stan = 'odstapiono') = (odstapil IS NOT NULL)
    AND (odstapil IS NULL) = (odstapienie_podpis IS NULL)
    AND (odstapil IS NULL) = (odstapiono IS NULL)
  ),
  CONSTRAINT przekazanie_nie_przed_utworzeniem CHECK (przekazano IS NULL OR przekazano >= utworzono)
) STRICT;

-- Wpisy w rejestrze kontroli, na których stoi projekt, np. sprawdzenia i decyzja potwierdzenia
-- podejrzenia.
CREATE TABLE kierownik_kontroli_projekty_podstawy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_kontroli_projekty_podstawy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  projekt_id INTEGER NOT NULL REFERENCES kierownik_kontroli_projekty_wpisy (id),
  rejestr_odnosnik TEXT NOT NULL CHECK (rejestr_odnosnik <> ''),

  CONSTRAINT podstawa_raz_w_projekcie UNIQUE (projekt_id, rejestr_odnosnik)
) STRICT;

-- Projekty ---------------------------------------------------------------------------------------

-- Reguły przekazania i odstąpienia sprawdzają zmianę stanu, więc projekt nie może powstać od razu
-- przekazany.
CREATE TRIGGER kierownik_kontroli_projekty_powstaje_jako_projekt
BEFORE INSERT ON kierownik_kontroli_projekty_wpisy
WHEN NEW.stan <> 'projekt'
BEGIN
  SELECT RAISE(ABORT, 'projekt zawiadomienia powstaje w stanie projekt');
END;

CREATE TRIGGER kierownik_kontroli_projekty_przekazany_i_odstapiony_ostateczne
BEFORE UPDATE ON kierownik_kontroli_projekty_wpisy
WHEN OLD.stan IN ('przekazany', 'odstapiono')
  OR NEW.id IS NOT OLD.id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.teczka_odnosnik IS NOT OLD.teczka_odnosnik
  OR NEW.utworzono IS NOT OLD.utworzono
BEGIN
  SELECT RAISE(ABORT, 'rodzaju, teczki ani daty projektu się nie zmienia, a przekazanego i odstąpionego wcale');
END;

-- Podstawy ---------------------------------------------------------------------------------------

CREATE TRIGGER kierownik_kontroli_projekty_podstawa_projektu_w_toku
BEFORE INSERT ON kierownik_kontroli_projekty_podstawy
WHEN NOT EXISTS (
  SELECT 1
  FROM kierownik_kontroli_projekty_wpisy AS projekt
  WHERE projekt.id = NEW.projekt_id
    AND projekt.stan IN ('projekt', 'u_wlasciciela')
)
BEGIN
  SELECT RAISE(ABORT, 'podstawę dopisuje się tylko do projektu w toku');
END;

CREATE TRIGGER kierownik_kontroli_projekty_podstawy_bez_zmian
BEFORE UPDATE ON kierownik_kontroli_projekty_podstawy
BEGIN
  SELECT RAISE(ABORT, 'podstawy projektu się nie zmienia; inna podstawa to nowy wpis');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers; także podstawę o tym
-- samym wpisie rejestru. Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego
-- wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_kontroli_projekty_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_kontroli_projekty_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_kontroli_projekty_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER kierownik_kontroli_projekty_podstawy_bez_zastepowania
BEFORE INSERT ON kierownik_kontroli_projekty_podstawy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_kontroli_projekty_podstawy
  WHERE id = NEW.id
    OR (projekt_id = NEW.projekt_id AND rejestr_odnosnik = NEW.rejestr_odnosnik)
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: przekazany projekt wskazuje rejestr, a odstąpiony zostaje z podpisem właściciela.
CREATE TRIGGER kierownik_kontroli_projekty_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_kontroli_projekty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); niewysłany projekt zamyka odstąpienie');
END;

CREATE TRIGGER kierownik_kontroli_projekty_podstawy_bez_usuwania
BEFORE DELETE ON kierownik_kontroli_projekty_podstawy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
