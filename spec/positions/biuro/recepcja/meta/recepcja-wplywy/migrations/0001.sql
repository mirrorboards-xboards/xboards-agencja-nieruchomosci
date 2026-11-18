-- Dziennik wpływów: kiedy wiadomość wpłynęła, którym kanałem, od kogo, jakiego jest rodzaju, do
-- której sprawy należy, po jakim znaku ją przypisano i komu ją oddano. Treści wiadomości tu nie ma:
-- plik czeka w skrzynce, z której przyszedł, a wiersz wskazuje tylko wiadomość.
--
-- Wiadomość trafia do sprawy tylko po znaku: po znaku ze spisu spraw albo po numerze oferty lub
-- ogłoszenia z katalogu ofert, a nie po tym, co sama o sobie twierdzi. Wpis powstaje, gdy Recepcja
-- wiadomość rozdzieliła, i jest tylko dopisywany. Pism od Generalnego Inspektora Informacji
-- Finansowej dziennik nie zna (art. 54 ust. 1 AML).
CREATE TABLE recepcja_wplywy_dziennik (
  -- Klucz nadany przez bazę jest dodatni; zob. recepcja_wplywy_dziennik_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Co najmniej do minuty: RRRR-MM-DDTHH:MM, bo od doręczenia może biec termin.
  wplynelo TEXT NOT NULL,
  kanal TEXT NOT NULL CHECK (kanal IN ('email', 'formularz', 'portal', 'telefon', 'skan_poczty')),
  -- Adres, numer albo konto na portalu, z którego wiadomość przyszła.
  nadawca TEXT NOT NULL CHECK (nadawca <> ''),
  -- Np. „zapytanie o ofertę”, „zaświadczenie spółdzielni”, „wypowiedzenie umowy najmu”.
  rodzaj_pisma TEXT NOT NULL CHECK (rodzaj_pisma <> ''),
  -- Gdzie plik czeka w skrzynce, tekstem.
  wiadomosc_odnosnik TEXT NOT NULL CHECK (wiadomosc_odnosnik <> ''),
  sprawa_id INTEGER REFERENCES recepcja_sprawy_wpisy (id),
  znak_id INTEGER REFERENCES recepcja_sprawy_znaki (id),
  -- Wpis katalogu ofert, którego numer oferty albo ogłoszenia podała wiadomość, tekstem.
  katalog_odnosnik TEXT,
  -- Agent sprawy (także Konsjerż lokalu), karta dla Kierownika biura, pracownik biura, do którego
  -- pismo zaadresowano imiennie, albo nigdzie, np. automatyczna odpowiedź.
  oddano TEXT NOT NULL CHECK (oddano IN ('agent_sprawy', 'karta', 'pracownik', 'nigdzie')),
  -- Tożsamość agenta albo pracownika, któremu oddano wiadomość.
  oddano_komu TEXT,
  oddano_o TEXT,
  -- Wiadomość przyszła innym kanałem, niż ten nadawca zwykle pisze.
  kanal_nie_pasuje INTEGER NOT NULL CHECK (kanal_nie_pasuje IN (0, 1)),

  CONSTRAINT wplynelo_co_do_minuty CHECK (length(wplynelo) >= 16),
  CONSTRAINT sprawa_tylko_po_znaku_albo_katalogu CHECK (
    (sprawa_id IS NULL) = (znak_id IS NULL AND katalog_odnosnik IS NULL)
    AND (znak_id IS NULL OR katalog_odnosnik IS NULL)
  ),
  CONSTRAINT agent_sprawy_dostaje_wiadomosc_ze_sprawa CHECK (oddano <> 'agent_sprawy' OR sprawa_id IS NOT NULL),
  CONSTRAINT oddane_komus CHECK ((oddano IN ('agent_sprawy', 'pracownik')) = (oddano_komu IS NOT NULL)),
  CONSTRAINT oddane_z_czasem CHECK ((oddano = 'nigdzie') = (oddano_o IS NULL)),
  CONSTRAINT oddanie_nie_przed_wplywem CHECK (oddano_o IS NULL OR oddano_o >= wplynelo)
) STRICT;

-- Znak musi należeć do wskazanej sprawy i nie być wycofany, bo inaczej wiadomość trafiłaby do
-- cudzej sprawy.
CREATE TRIGGER recepcja_wplywy_znak_tej_sprawy
BEFORE INSERT ON recepcja_wplywy_dziennik
WHEN NEW.znak_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM recepcja_sprawy_znaki AS znak
    WHERE znak.id = NEW.znak_id
      AND znak.sprawa_id = NEW.sprawa_id
      AND znak.wycofano IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'wiadomość trafia do sprawy po jej niewycofanym znaku');
END;

-- Po numerze z katalogu ofert wiadomość trafia do oferty, a oferta jest sprawą spisu.
CREATE TRIGGER recepcja_wplywy_katalog_wskazuje_oferte
BEFORE INSERT ON recepcja_wplywy_dziennik
WHEN NEW.katalog_odnosnik IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM recepcja_sprawy_wpisy AS sprawa
    WHERE sprawa.id = NEW.sprawa_id
      AND sprawa.rodzaj = 'oferta'
  )
BEGIN
  SELECT RAISE(ABORT, 'po numerze z katalogu ofert wiadomość trafia do sprawy oferty');
END;

CREATE TRIGGER recepcja_wplywy_bez_zmian
BEFORE UPDATE ON recepcja_wplywy_dziennik
BEGIN
  SELECT RAISE(ABORT, 'dziennik wpływów tylko się dopisuje; zmiany nie są dozwolone');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER recepcja_wplywy_dziennik_bez_zastepowania
BEFORE INSERT ON recepcja_wplywy_dziennik
WHEN EXISTS (
  SELECT 1
  FROM recepcja_wplywy_dziennik
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: dziennik jest śladem, że żadna wiadomość nie zginęła.
CREATE TRIGGER recepcja_wplywy_dziennik_bez_usuwania
BEFORE DELETE ON recepcja_wplywy_dziennik
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); dziennik wpływów tylko się dopisuje');
END;
