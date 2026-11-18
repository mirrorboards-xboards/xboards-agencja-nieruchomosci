-- Standardy działu: kolejne wersje standardów, wzorów i parametrów programu, które zaproponował
-- kierownik, i rozstrzygnięcie człowieka. Dział ma kilka standardów, np. wzór umowy pośrednictwa
-- i standard ogłoszenia; z każdego obowiązuje najnowsza zatwierdzona wersja.
--
-- Tekst wersji leży w repozytorium XGit pokoju pamięci; wiersz trzyma ścieżkę i skrót, a podpis
-- rozstrzygnięcia obejmuje ten skrót. Rozstrzyga właściciel kierownika, a standardy Kontroli
-- właściciel biura, którego podpis przychodzi wiadomością. Baza wymaga kompletu kolumn podpisu,
-- a sam podpis sprawdza aplikacja.
CREATE TABLE kierownik_dzialu_standardy_wersje (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_dzialu_standardy_wersje_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Np. „wzor_umowy_posrednictwa”, „standard_ogloszenia”, „parametry_programu”.
  standard TEXT NOT NULL CHECK (standard <> ''),
  numer INTEGER NOT NULL CHECK (numer > 0),
  sciezka TEXT NOT NULL,
  skrot TEXT NOT NULL,
  opis_zmian TEXT NOT NULL,
  zaproponowano TEXT NOT NULL,
  rozstrzygniecie TEXT CHECK (rozstrzygniecie IN ('zatwierdzona', 'odrzucona')),
  podpisal TEXT,
  podpisal_jako TEXT CHECK (podpisal_jako IN ('wlasciciel', 'wlasciciel_biura')),
  podpis TEXT,
  podpisano TEXT,

  CONSTRAINT numer_wersji_raz_w_standardzie UNIQUE (standard, numer),
  CONSTRAINT podpis_w_komplecie CHECK (
    (podpisal IS NULL) = (podpisal_jako IS NULL)
    AND (podpisal IS NULL) = (podpis IS NULL)
    AND (podpisal IS NULL) = (podpisano IS NULL)
  ),
  CONSTRAINT rozstrzygniecie_z_podpisem CHECK ((rozstrzygniecie IS NULL) = (podpisal IS NULL)),
  CONSTRAINT rozstrzygniecie_nie_przed_propozycja CHECK (podpisano IS NULL OR podpisano >= zaproponowano)
) STRICT;

CREATE TRIGGER kierownik_dzialu_standardy_powstaje_jako_propozycja
BEFORE INSERT ON kierownik_dzialu_standardy_wersje
WHEN NEW.rozstrzygniecie IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'wersja standardu powstaje jako propozycja; rozstrzyga ją człowiek');
END;

-- Podpis obejmuje skrót tekstu, więc rozstrzygniętej wersji nie zmienia się wcale, a propozycja
-- zmienia się tylko rozstrzygnięciem.
CREATE TRIGGER kierownik_dzialu_standardy_rozstrzygniecie_jest_ostateczne
BEFORE UPDATE ON kierownik_dzialu_standardy_wersje
WHEN OLD.rozstrzygniecie IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.standard IS NOT OLD.standard
  OR NEW.numer IS NOT OLD.numer
  OR NEW.sciezka IS NOT OLD.sciezka
  OR NEW.skrot IS NOT OLD.skrot
  OR NEW.opis_zmian IS NOT OLD.opis_zmian
  OR NEW.zaproponowano IS NOT OLD.zaproponowano
BEGIN
  SELECT RAISE(ABORT, 'rozstrzygniętej wersji standardu się nie zmienia, a propozycję zmienia tylko rozstrzygnięcie');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers; także wiersz o tym samym
-- numerze wersji standardu. Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego
-- wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_dzialu_standardy_wersje_bez_zastepowania
BEFORE INSERT ON kierownik_dzialu_standardy_wersje
WHEN EXISTS (
  SELECT 1
  FROM kierownik_dzialu_standardy_wersje
  WHERE id = NEW.id
)
  OR EXISTS (
    SELECT 1
    FROM kierownik_dzialu_standardy_wersje
    WHERE standard = NEW.standard
      AND numer = NEW.numer
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: odrzucona i dawna wersja zostają jako historia standardu.
CREATE TRIGGER kierownik_dzialu_standardy_wersje_bez_usuwania
BEFORE DELETE ON kierownik_dzialu_standardy_wersje
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
