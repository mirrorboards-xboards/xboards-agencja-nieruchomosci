-- Analizy stanu prawnego i ryzyka transakcji: księga wieczysta, ograniczenia obrotu, projekty
-- i forma umów, przekazanie i przerwanie. Pisze Analityk stanu prawnego. Analiza ma plik z pełnym
-- wywodem, a w tabeli krótkie wnioski i ryzyka.
--
-- Analizy się nie zmienia: nowa analiza tego samego przedmiotu (i tej samej umowy) odbiera
-- poprzedniej aktualność. Wniosek, że transakcji nie da się przeprowadzić, wymaga zgody człowieka
-- (decyzja przerwanie_sprawy w rdzeniu); ryzyko trafia do teczki od razu.
CREATE TABLE transakcja_analizy_wpisy (
  id TEXT PRIMARY KEY,
  przedmiot TEXT NOT NULL CHECK (przedmiot IN ('ksiega', 'ograniczenia', 'projekt_umowy', 'forma_umowy', 'przekazanie', 'przerwanie')),
  -- Umowa, której dotyczy analiza projektu albo formy.
  umowa_id TEXT REFERENCES transakcja_umowy_wpisy (id),
  -- Analizowany dokument, np. odpis księgi albo projekt umowy, i przy księdze poprzedni odpis,
  -- z którym Analityk porównuje nowy tuż przed aktem.
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  poprzedni_odpis_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  sporzadzono TEXT NOT NULL,
  wnioski TEXT NOT NULL CHECK (wnioski <> ''),
  sciezka TEXT NOT NULL,
  skrot TEXT NOT NULL,
  aktualna INTEGER NOT NULL CHECK (aktualna IN (0, 1)),

  CONSTRAINT umowa_przy_analizie_umowy CHECK (
    (przedmiot IN ('projekt_umowy', 'forma_umowy')) = (umowa_id IS NOT NULL)
  ),
  CONSTRAINT projekt_umowy_wskazuje_dokument CHECK (przedmiot <> 'projekt_umowy' OR dokument_id IS NOT NULL),
  CONSTRAINT poprzedni_odpis_tylko_przy_ksiedze CHECK (poprzedni_odpis_dokument_id IS NULL OR przedmiot = 'ksiega')
) STRICT;

-- Ryzyka, które Analityk wskazał w analizie, ze skutkiem dla stron.
CREATE TABLE transakcja_analizy_ryzyka (
  -- Klucz nadany przez bazę jest dodatni; zob. transakcja_analizy_ryzyka_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  analiza_id TEXT NOT NULL REFERENCES transakcja_analizy_wpisy (id),
  opis TEXT NOT NULL CHECK (opis <> ''),
  skutek TEXT
) STRICT;

-- Aktualność odbiera tylko nowsza analiza, więc analiza powstaje aktualna.
CREATE TRIGGER transakcja_analizy_powstaje_aktualna
BEFORE INSERT ON transakcja_analizy_wpisy
WHEN NEW.aktualna <> 1
BEGIN
  SELECT RAISE(ABORT, 'analiza powstaje jako aktualna');
END;

CREATE TRIGGER transakcja_analizy_nowa_zastepuje_poprzednia
AFTER INSERT ON transakcja_analizy_wpisy
BEGIN
  UPDATE transakcja_analizy_wpisy
  SET aktualna = 0
  WHERE id <> NEW.id
    AND aktualna = 1
    AND przedmiot = NEW.przedmiot
    AND umowa_id IS NEW.umowa_id;
END;

-- Analiza jest zapisem tego, co Analityk ustalił; jedyną zmianą jest utrata aktualności.
CREATE TRIGGER transakcja_analizy_bez_zmian
BEFORE UPDATE ON transakcja_analizy_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.przedmiot IS NOT OLD.przedmiot
  OR NEW.umowa_id IS NOT OLD.umowa_id
  OR NEW.dokument_id IS NOT OLD.dokument_id
  OR NEW.poprzedni_odpis_dokument_id IS NOT OLD.poprzedni_odpis_dokument_id
  OR NEW.sporzadzono IS NOT OLD.sporzadzono
  OR NEW.wnioski IS NOT OLD.wnioski
  OR NEW.sciezka IS NOT OLD.sciezka
  OR NEW.skrot IS NOT OLD.skrot
  OR (OLD.aktualna = 0 AND NEW.aktualna = 1)
BEGIN
  SELECT RAISE(ABORT, 'analizy się nie zmienia; nowa analiza to nowy wpis');
END;

-- Ryzyka należą do bieżącej oceny: dopisuje się je do aktualnej analizy i się ich nie zmienia.
CREATE TRIGGER transakcja_analizy_ryzyko_aktualnej_analizy
BEFORE INSERT ON transakcja_analizy_ryzyka
WHEN NOT EXISTS (
  SELECT 1
  FROM transakcja_analizy_wpisy
  WHERE id = NEW.analiza_id
    AND aktualna = 1
)
BEGIN
  SELECT RAISE(ABORT, 'ryzyko dopisuje się do aktualnej analizy');
END;

CREATE TRIGGER transakcja_analizy_ryzyka_bez_zmian
BEFORE UPDATE ON transakcja_analizy_ryzyka
BEGIN
  SELECT RAISE(ABORT, 'ryzyka się nie zmienia; nowa ocena to nowa analiza');
END;

-- Każdy projekt umowy ocenia Analityk, zanim strony go podpiszą: umowa zostaje zawarta tylko
-- z aktualną analizą właśnie tego projektu. Nowy projekt (inny dokument) wymaga nowej analizy.
CREATE TRIGGER transakcja_analizy_zawarcie_po_analizie_projektu
BEFORE UPDATE OF stan ON transakcja_umowy_wpisy
WHEN NEW.stan = 'zawarta'
  AND OLD.stan = 'projekt'
  AND NOT EXISTS (
    SELECT 1
    FROM transakcja_analizy_wpisy
    WHERE przedmiot = 'projekt_umowy'
      AND umowa_id = NEW.id
      AND dokument_id = NEW.projekt_dokument_id
      AND aktualna = 1
  )
BEGIN
  SELECT RAISE(ABORT, 'umowa zostaje zawarta tylko z aktualną analizą swojego projektu');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER transakcja_analizy_wpisy_bez_zastepowania
BEFORE INSERT ON transakcja_analizy_wpisy
WHEN EXISTS (
  SELECT 1
  FROM transakcja_analizy_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER transakcja_analizy_wpisy_bez_usuwania
BEFORE DELETE ON transakcja_analizy_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER transakcja_analizy_ryzyka_bez_zastepowania
BEFORE INSERT ON transakcja_analizy_ryzyka
WHEN EXISTS (
  SELECT 1
  FROM transakcja_analizy_ryzyka
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER transakcja_analizy_ryzyka_bez_usuwania
BEFORE DELETE ON transakcja_analizy_ryzyka
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
