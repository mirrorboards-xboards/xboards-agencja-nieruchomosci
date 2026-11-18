-- Zajrzenia: każde zajrzenie kierownika do teczki, z powodem. Kierownik działu zagląda do teczki
-- tylko z powodem, a zajrzenia są śladem dla właściciela i dla kontroli, więc się ich nie zmienia.
--
-- Wpis nie ma terminu przechowania: dopóki obowiązuje rozstrzygnięcie biura R9, biuro niczego nie
-- usuwa z upływem czasu. Rejestr kontroli jest jeden dla biura, więc wpis nie wskazuje, czyj to
-- rejestr.
CREATE TABLE kierownik_dzialu_zajrzenia_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_dzialu_zajrzenia_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Pokój teczki, tekstem.
  teczka_odnosnik TEXT NOT NULL CHECK (teczka_odnosnik <> ''),
  zajrzano TEXT NOT NULL,
  powod TEXT NOT NULL CHECK (powod IN ('raport_sygnalizuje_problem', 'prosba_czlowieka', 'przejecie_po_agencie', 'sprawdzenie_jakosci')),
  -- Co skłoniło kierownika do zajrzenia.
  opis TEXT NOT NULL CHECK (opis <> ''),
  raport_id TEXT REFERENCES kierownik_dzialu_raporty_wpisy (id)
) STRICT;

CREATE TRIGGER kierownik_dzialu_zajrzenia_bez_zmian
BEFORE UPDATE ON kierownik_dzialu_zajrzenia_wpisy
BEGIN
  SELECT RAISE(ABORT, 'zajrzenia się nie zmienia; jest śladem dla właściciela i kontroli');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_dzialu_zajrzenia_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_dzialu_zajrzenia_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_dzialu_zajrzenia_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: zajrzeń się nie wykreśla.
CREATE TRIGGER kierownik_dzialu_zajrzenia_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_dzialu_zajrzenia_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
