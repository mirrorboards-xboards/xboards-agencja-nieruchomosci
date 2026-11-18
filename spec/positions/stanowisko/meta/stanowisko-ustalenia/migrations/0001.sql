-- Ustalenia: co agent ustalił o świecie, ze źródłem, pewnością, sposobem sprawdzenia i datą.
-- Polecenia leżą osobno (stanowisko-polecenia), bo to, co agentowi kazano, nie jest faktem o świecie.
--
-- Ustalenie jest zapisem tego, co agent wiedział i skąd. Nowy fakt to nowe ustalenie, a poprzednie
-- staje się nieaktualne. Następca agenta widzi w kolumnie `zapisal`, które ustalenia są cudze, i może
-- ocenić je na nowo.
CREATE TABLE stanowisko_ustalenia_wpisy (
  id TEXT PRIMARY KEY,
  tresc TEXT NOT NULL,
  zrodlo_id TEXT REFERENCES stanowisko_zrodla_wpisy (id),
  -- Wiersz teczki, z którego ustalenie wynika: tabela i pokój z wierszem, tekstem.
  wynika_z_tabela TEXT,
  wynika_z_odnosnik TEXT,
  pewnosc TEXT NOT NULL CHECK (pewnosc IN ('potwierdzone', 'prawdopodobne', 'watpliwe')),
  sposob_sprawdzenia TEXT,
  ustalono TEXT NOT NULL,
  zapisal TEXT NOT NULL,
  -- Gdzie ustalenie weszło, np. wersja projektu albo notatka w teczce.
  wykorzystano_tabela TEXT,
  wykorzystano_odnosnik TEXT,
  nieaktualne_od TEXT,
  zastapione_przez_id TEXT REFERENCES stanowisko_ustalenia_wpisy (id),

  CONSTRAINT ustalenie_ze_zrodlem CHECK (zrodlo_id IS NOT NULL OR wynika_z_odnosnik IS NOT NULL),
  CONSTRAINT wynika_z_tabela_razem_z_odnosnikiem CHECK ((wynika_z_tabela IS NULL) = (wynika_z_odnosnik IS NULL)),
  CONSTRAINT wykorzystano_tabela_razem_z_odnosnikiem CHECK ((wykorzystano_tabela IS NULL) = (wykorzystano_odnosnik IS NULL)),
  CONSTRAINT potwierdzone_ze_sposobem_sprawdzenia CHECK (pewnosc <> 'potwierdzone' OR sposob_sprawdzenia IS NOT NULL),
  CONSTRAINT zastapione_jest_nieaktualne CHECK (zastapione_przez_id IS NULL OR nieaktualne_od IS NOT NULL)
) STRICT;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- triggerów UPDATE i DELETE, bo pokój nie włącza recursive_triggers; zmianę wiersza robi UPDATE.
CREATE TRIGGER stanowisko_ustalenia_wpisy_bez_zastepowania
BEFORE INSERT ON stanowisko_ustalenia_wpisy
WHEN EXISTS (
  SELECT 1
  FROM stanowisko_ustalenia_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- Pewność, wykorzystanie i nieaktualność się zmieniają; to, co i skąd agent ustalił, już nie.
CREATE TRIGGER stanowisko_ustalenia_tresc_niezmienna
BEFORE UPDATE ON stanowisko_ustalenia_wpisy
WHEN NEW.tresc IS NOT OLD.tresc
  OR NEW.zrodlo_id IS NOT OLD.zrodlo_id
  OR NEW.wynika_z_tabela IS NOT OLD.wynika_z_tabela
  OR NEW.wynika_z_odnosnik IS NOT OLD.wynika_z_odnosnik
  OR NEW.ustalono IS NOT OLD.ustalono
  OR NEW.zapisal IS NOT OLD.zapisal
BEGIN
  SELECT RAISE(ABORT, 'treści, źródła, daty ani autora ustalenia się nie zmienia; nowy fakt to nowe ustalenie');
END;
