-- Sprawozdania dla właściciela biura, robocze i przekazane. Tekst leży w repozytorium XGit pokoju
-- pamięci; wiersz trzyma okres, ścieżkę i skrót. Przekazanego sprawozdania się nie zmienia, bo
-- właściciel biura opiera na nim decyzje; poprawka to nowe sprawozdanie.
CREATE TABLE kierownik_biura_sprawozdania_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_biura_sprawozdania_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Miesięczne, na żądanie właściciela biura albo od razu, np. po przekroczeniu limitu spraw.
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('miesieczne', 'na_zadanie', 'natychmiastowe')),
  okres_od TEXT NOT NULL,
  okres_do TEXT NOT NULL,
  sciezka TEXT NOT NULL CHECK (sciezka <> ''),
  skrot TEXT NOT NULL CHECK (skrot <> ''),
  sporzadzono TEXT NOT NULL,
  przekazano TEXT,

  CONSTRAINT okres_od_nie_po_do CHECK (okres_od <= okres_do),
  CONSTRAINT przekazanie_nie_przed_sporzadzeniem CHECK (przekazano IS NULL OR przekazano >= sporzadzono)
) STRICT;

CREATE TRIGGER kierownik_biura_sprawozdania_powstaje_robocze
BEFORE INSERT ON kierownik_biura_sprawozdania_wpisy
WHEN NEW.przekazano IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'sprawozdanie powstaje robocze; przekazanie odnotowuje się później');
END;

CREATE TRIGGER kierownik_biura_sprawozdania_przekazane_bez_zmian
BEFORE UPDATE ON kierownik_biura_sprawozdania_wpisy
WHEN OLD.przekazano IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.rodzaj IS NOT OLD.rodzaj
BEGIN
  SELECT RAISE(ABORT, 'przekazanego sprawozdania się nie zmienia; poprawka to nowe sprawozdanie');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_biura_sprawozdania_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_biura_sprawozdania_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_biura_sprawozdania_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: przekazane sprawozdania zostają.
CREATE TRIGGER kierownik_biura_sprawozdania_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_biura_sprawozdania_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
