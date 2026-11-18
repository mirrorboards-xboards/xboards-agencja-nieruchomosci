-- Ustalenia: co agent ustalił o świecie, ze źródłem, pewnością, sposobem sprawdzenia i datą.
-- Polecenia leżą osobno (stanowisko-polecenia), bo to, co agentowi kazano, nie jest faktem o świecie.
--
-- Ustalenie jest zapisem tego, co agent wiedział i skąd. Nowy fakt to nowe ustalenie, a poprzednie
-- staje się nieaktualne. Tabela nie ma kolumny autora: autora każdego zapisu zapisuje dziennik
-- pokoju, więc następca agenta odróżnia w nim ustalenia poprzednika i może ocenić je na nowo.
CREATE TABLE stanowisko_ustalenia_wpisy (
  id TEXT PRIMARY KEY,
  tresc TEXT NOT NULL,
  zrodlo_id TEXT REFERENCES stanowisko_zrodla_wpisy (id),
  -- Wiersz teczki, z którego ustalenie wynika: tabela i pokój z wierszem, tekstem.
  wynika_z_tabela TEXT,
  wynika_z_odnosnik TEXT,
  -- Czego ustalenie dotyczy, gdy agent pracuje w wielu teczkach, np. teczka lokalu (jej metryka),
  -- propozycja w teczce poszukiwania albo pobyt gościa: tabela i pokój z wierszem, tekstem.
  dotyczy_tabela TEXT,
  dotyczy_odnosnik TEXT,
  pewnosc TEXT NOT NULL CHECK (pewnosc IN ('potwierdzone', 'prawdopodobne', 'niepewne')),
  sposob_sprawdzenia TEXT,
  ustalono TEXT NOT NULL,
  -- Gdzie ustalenie weszło, np. notatka w teczce albo raport.
  wykorzystano_tabela TEXT,
  wykorzystano_odnosnik TEXT,
  nieaktualne_od TEXT,
  zastapione_przez_id TEXT REFERENCES stanowisko_ustalenia_wpisy (id),

  CONSTRAINT ustalenie_ze_zrodlem CHECK (zrodlo_id IS NOT NULL OR wynika_z_odnosnik IS NOT NULL),
  CONSTRAINT wynika_z_tabela_razem_z_odnosnikiem CHECK ((wynika_z_tabela IS NULL) = (wynika_z_odnosnik IS NULL)),
  CONSTRAINT dotyczy_tabela_razem_z_odnosnikiem CHECK ((dotyczy_tabela IS NULL) = (dotyczy_odnosnik IS NULL)),
  CONSTRAINT wykorzystano_tabela_razem_z_odnosnikiem CHECK ((wykorzystano_tabela IS NULL) = (wykorzystano_odnosnik IS NULL)),
  CONSTRAINT potwierdzone_ze_sposobem_sprawdzenia CHECK (pewnosc <> 'potwierdzone' OR sposob_sprawdzenia IS NOT NULL),
  CONSTRAINT zastapione_jest_nieaktualne CHECK (zastapione_przez_id IS NULL OR nieaktualne_od IS NOT NULL)
) STRICT;

-- Nieaktualność przychodzi po ustaleniu, więc ustalenie powstaje aktualne.
CREATE TRIGGER stanowisko_ustalenia_powstaje_aktualne
BEFORE INSERT ON stanowisko_ustalenia_wpisy
WHEN NEW.nieaktualne_od IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'ustalenie powstaje aktualne; nieaktualność odnotowuje się później');
END;

-- Pewność, wykorzystanie i nieaktualność się zmieniają; to, co, skąd i o czym agent ustalił, już nie.
CREATE TRIGGER stanowisko_ustalenia_tresc_niezmienna
BEFORE UPDATE ON stanowisko_ustalenia_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.tresc IS NOT OLD.tresc
  OR NEW.zrodlo_id IS NOT OLD.zrodlo_id
  OR NEW.wynika_z_tabela IS NOT OLD.wynika_z_tabela
  OR NEW.wynika_z_odnosnik IS NOT OLD.wynika_z_odnosnik
  OR NEW.dotyczy_tabela IS NOT OLD.dotyczy_tabela
  OR NEW.dotyczy_odnosnik IS NOT OLD.dotyczy_odnosnik
  OR NEW.ustalono IS NOT OLD.ustalono
BEGIN
  SELECT RAISE(ABORT, 'treści, źródła, przedmiotu ani daty ustalenia się nie zmienia; nowy fakt to nowe ustalenie');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER stanowisko_ustalenia_wpisy_bez_zastepowania
BEFORE INSERT ON stanowisko_ustalenia_wpisy
WHEN EXISTS (
  SELECT 1
  FROM stanowisko_ustalenia_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: pamięć trwa bez terminu; ustalenie, które przestało być prawdą, staje się nieaktualne.
CREATE TRIGGER stanowisko_ustalenia_wpisy_bez_usuwania
BEFORE DELETE ON stanowisko_ustalenia_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
