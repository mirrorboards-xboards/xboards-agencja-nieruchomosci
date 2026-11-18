-- Projekt oceny ryzyka biura (art. 27 ust. 1 AML): ocena robocza z czynnikami ryzyka dotyczącymi
-- klientów, państw i obszarów geograficznych, produktów, usług, transakcji i kanałów dostaw, każdy
-- z poziomem, środkami ograniczającymi i źródłem.
--
-- Projekt zatwierdza właściciel biura. Zatwierdzona ocena przechodzi do rejestru kontroli; od tej
-- chwili projekt w pamięci tylko wskazuje tamten wpis i się nie zmienia. Kolejny przegląd to nowy
-- projekt.
CREATE TABLE kierownik_kontroli_oceny_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_kontroli_oceny_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  rozpoczeto TEXT NOT NULL,
  -- Ostatni przegląd czynników projektu (przegląd coroczny).
  przejrzano TEXT,
  przekazano_do_zatwierdzenia TEXT,
  -- Zatwierdzona ocena biura w rejestrze kontroli, tekstem.
  rejestr_odnosnik TEXT,

  CONSTRAINT przeglad_nie_przed_rozpoczeciem CHECK (przejrzano IS NULL OR przejrzano >= rozpoczeto),
  CONSTRAINT przekazanie_nie_przed_rozpoczeciem CHECK (
    przekazano_do_zatwierdzenia IS NULL
    OR przekazano_do_zatwierdzenia >= rozpoczeto
  ),
  CONSTRAINT zatwierdzona_po_przekazaniu CHECK (rejestr_odnosnik IS NULL OR przekazano_do_zatwierdzenia IS NOT NULL)
) STRICT;

CREATE TABLE kierownik_kontroli_oceny_czynniki (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_kontroli_oceny_czynniki_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  ocena_id INTEGER NOT NULL REFERENCES kierownik_kontroli_oceny_wpisy (id),
  obszar TEXT NOT NULL CHECK (obszar IN ('klienci', 'obszary_geograficzne', 'produkty', 'uslugi', 'transakcje', 'kanaly_dostaw')),
  czynnik TEXT NOT NULL CHECK (czynnik <> ''),
  poziom TEXT NOT NULL CHECK (poziom IN ('niskie', 'srednie', 'wysokie')),
  srodki TEXT,
  -- Skąd kierownik wie o czynniku, np. krajowa ocena ryzyka albo raport Kontrolera.
  zrodlo TEXT NOT NULL CHECK (zrodlo <> '')
) STRICT;

CREATE TRIGGER kierownik_kontroli_oceny_powstaje_robocza
BEFORE INSERT ON kierownik_kontroli_oceny_wpisy
WHEN NEW.przekazano_do_zatwierdzenia IS NOT NULL
  OR NEW.rejestr_odnosnik IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'projekt oceny powstaje roboczy; przekazanie i zatwierdzenie przychodzą później');
END;

-- Zatwierdzony projekt tylko wskazuje wpis w rejestrze, więc się go nie zmienia, także jego
-- czynników.
CREATE TRIGGER kierownik_kontroli_oceny_zatwierdzona_bez_zmian
BEFORE UPDATE ON kierownik_kontroli_oceny_wpisy
WHEN OLD.rejestr_odnosnik IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.rozpoczeto IS NOT OLD.rozpoczeto
BEGIN
  SELECT RAISE(ABORT, 'projektu oceny zatwierdzonej w rejestrze kontroli się nie zmienia');
END;

CREATE TRIGGER kierownik_kontroli_oceny_czynnik_dopisany_do_zatwierdzonej
BEFORE INSERT ON kierownik_kontroli_oceny_czynniki
WHEN EXISTS (
  SELECT 1
  FROM kierownik_kontroli_oceny_wpisy AS ocena
  WHERE ocena.id = NEW.ocena_id
    AND ocena.rejestr_odnosnik IS NOT NULL
)
BEGIN
  SELECT RAISE(ABORT, 'projektu oceny zatwierdzonej w rejestrze kontroli się nie zmienia');
END;

CREATE TRIGGER kierownik_kontroli_oceny_czynnik_zatwierdzonej_bez_zmian
BEFORE UPDATE ON kierownik_kontroli_oceny_czynniki
WHEN NEW.id IS NOT OLD.id
  OR NEW.ocena_id IS NOT OLD.ocena_id
  OR EXISTS (
    SELECT 1
    FROM kierownik_kontroli_oceny_wpisy AS ocena
    WHERE ocena.id = OLD.ocena_id
      AND ocena.rejestr_odnosnik IS NOT NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'projektu oceny zatwierdzonej w rejestrze kontroli się nie zmienia, a czynnik nie przechodzi do innego projektu');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_kontroli_oceny_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_kontroli_oceny_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_kontroli_oceny_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER kierownik_kontroli_oceny_czynniki_bez_zastepowania
BEFORE INSERT ON kierownik_kontroli_oceny_czynniki
WHEN EXISTS (
  SELECT 1
  FROM kierownik_kontroli_oceny_czynniki
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: dawne projekty zostają jako historia oceny ryzyka biura.
CREATE TRIGGER kierownik_kontroli_oceny_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_kontroli_oceny_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER kierownik_kontroli_oceny_czynniki_bez_usuwania
BEFORE DELETE ON kierownik_kontroli_oceny_czynniki
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
