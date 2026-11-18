-- Protokoły zdawczo-odbiorcze nieruchomości: przejęcie i wydanie lokalu, zwrot, przekazanie
-- po akcie, przyjęcie i zwrot kluczy klienta. Jeden model służy transakcji, najmowi, lokalowi
-- krótkoterminowemu i kluczom oferty, więc porównanie wydania ze zwrotem robi program na tych
-- samych pozycjach. Typ sprawy wskazuje protokół kluczem obcym, np. etap najmu wymaga podpisanego
-- protokołu wydania.
--
-- Protokół przygotowuje agent sprawy jako projekt i dopisuje pozycje: liczniki z odczytem,
-- wyposażenie, klucze, pomieszczenia i uwagi. Podpisany protokół ma dokument z podpisami stron
-- i od podpisu nie zmienia się ani on, ani jego pozycje. Pomyłkę po podpisie poprawia nowy
-- protokół.
CREATE TABLE sprawa_protokoly_wpisy (
  id TEXT PRIMARY KEY,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('przejecie', 'wydanie', 'zwrot', 'przekazanie', 'przyjecie_kluczy', 'zwrot_kluczy')),
  nieruchomosc_id TEXT NOT NULL REFERENCES sprawa_nieruchomosci_wpisy (id),
  -- Chwila przekazania, w której spisano pozycje.
  chwila TEXT NOT NULL,
  stan TEXT NOT NULL CHECK (stan IN ('projekt', 'podpisany')),
  -- Podpisany protokół jako dokument teczki rodzaju protokol.
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),

  CONSTRAINT podpisany_ma_dokument CHECK ((stan = 'podpisany') = (dokument_id IS NOT NULL))
) STRICT;

-- Pozycje protokołu, każda z opisem stanu i zdjęciem, jeśli jest.
CREATE TABLE sprawa_protokoly_pozycje (
  -- Klucz nadany przez bazę jest dodatni; zob. sprawa_protokoly_pozycje_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  protokol_id TEXT NOT NULL REFERENCES sprawa_protokoly_wpisy (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('licznik', 'wyposazenie', 'klucze', 'pomieszczenie', 'uwaga')),
  -- Np. „licznik energii elektrycznej”, „klucz do drzwi wejściowych”, „pralka”.
  nazwa TEXT NOT NULL CHECK (nazwa <> ''),
  -- Numer licznika albo oznaczenie rzeczy, np. numer seryjny.
  numer TEXT,
  -- Odczyt licznika, liczba kluczy albo stan rzeczy, np. „12 345,6 kWh”, „3 sztuki”, „rysa na blacie”.
  opis_stanu TEXT,
  zdjecie_sciezka TEXT,
  zdjecie_skrot TEXT,

  CONSTRAINT zdjecie_ze_skrotem CHECK ((zdjecie_sciezka IS NULL) = (zdjecie_skrot IS NULL)),
  -- Licznik bez numeru i odczytu nie da się porównać przy zwrocie.
  CONSTRAINT licznik_ma_numer_i_odczyt CHECK (rodzaj <> 'licznik' OR (numer IS NOT NULL AND opis_stanu IS NOT NULL))
) STRICT;

-- Pozycje dopisuje się do projektu, a podpis sprawdza zmianę stanu, więc protokół nie może
-- powstać od razu podpisany.
CREATE TRIGGER sprawa_protokoly_powstaje_jako_projekt
BEFORE INSERT ON sprawa_protokoly_wpisy
WHEN NEW.stan <> 'projekt'
BEGIN
  SELECT RAISE(ABORT, 'protokół powstaje jako projekt');
END;

CREATE TRIGGER sprawa_protokoly_podpisany_z_dokumentem_protokolu
BEFORE UPDATE ON sprawa_protokoly_wpisy
WHEN NEW.stan = 'podpisany'
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_dokumenty_wpisy
    WHERE id = NEW.dokument_id
      AND rodzaj = 'protokol'
      AND stan IN ('otrzymany', 'sprawdzony')
  )
BEGIN
  SELECT RAISE(ABORT, 'podpisany protokół wskazuje otrzymany dokument rodzaju protokol');
END;

-- Podpis obejmuje cały protokół: rodzaj, nieruchomość, chwilę i dokument.
CREATE TRIGGER sprawa_protokoly_podpisany_niezmienny
BEFORE UPDATE ON sprawa_protokoly_wpisy
WHEN OLD.stan = 'podpisany'
BEGIN
  SELECT RAISE(ABORT, 'podpisanego protokołu się nie zmienia; pomyłkę poprawia nowy protokół');
END;

-- Dokument podpisanego protokołu jest jego podstawą: nie zmienia rodzaju ani tego, czego dotyczy,
-- i nie staje się odrzucony ani zbędny.
CREATE TRIGGER sprawa_protokoly_dokument_podpisanego_zostaje
BEFORE UPDATE ON sprawa_dokumenty_wpisy
WHEN (
    NEW.rodzaj IS NOT OLD.rodzaj
    OR NEW.dotyczy_uczestnik_id IS NOT OLD.dotyczy_uczestnik_id
    OR NEW.dotyczy_nieruchomosc_id IS NOT OLD.dotyczy_nieruchomosc_id
    OR (NEW.stan IN ('odrzucony', 'zbedny') AND NEW.stan IS NOT OLD.stan)
  )
  AND EXISTS (
    SELECT 1
    FROM sprawa_protokoly_wpisy
    WHERE stan = 'podpisany'
      AND dokument_id = OLD.id
  )
BEGIN
  SELECT RAISE(ABORT, 'dokument podpisanego protokołu nie zmienia rodzaju ani tego, czego dotyczy, i nie jest odrzucany');
END;

CREATE TRIGGER sprawa_protokoly_pozycje_tylko_w_projekcie
BEFORE INSERT ON sprawa_protokoly_pozycje
WHEN (
  SELECT stan <> 'projekt'
  FROM sprawa_protokoly_wpisy
  WHERE id = NEW.protokol_id
)
BEGIN
  SELECT RAISE(ABORT, 'pozycje dopisuje się tylko do projektu protokołu');
END;

-- W projekcie pozycję się poprawia, ale nie przenosi do innego protokołu.
CREATE TRIGGER sprawa_protokoly_pozycja_zostaje_w_protokole
BEFORE UPDATE ON sprawa_protokoly_pozycje
WHEN NEW.id IS NOT OLD.id
  OR NEW.protokol_id IS NOT OLD.protokol_id
BEGIN
  SELECT RAISE(ABORT, 'pozycja zostaje w swoim protokole');
END;

CREATE TRIGGER sprawa_protokoly_pozycje_po_podpisie_bez_zmian
BEFORE UPDATE ON sprawa_protokoly_pozycje
WHEN (
  SELECT stan <> 'projekt'
  FROM sprawa_protokoly_wpisy
  WHERE id = OLD.protokol_id
)
BEGIN
  SELECT RAISE(ABORT, 'pozycji podpisanego protokołu się nie zmienia');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_protokoly_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_protokoly_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_protokoly_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_protokoly_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_protokoly_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER sprawa_protokoly_pozycje_bez_zastepowania
BEFORE INSERT ON sprawa_protokoly_pozycje
WHEN EXISTS (
  SELECT 1
  FROM sprawa_protokoly_pozycje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_protokoly_pozycje_bez_usuwania
BEFORE DELETE ON sprawa_protokoly_pozycje
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
