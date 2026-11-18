-- Polisy ubezpieczenia odpowiedzialności cywilnej biura: pośrednika i zarządcy. Kopia polisy
-- aktualnej na dzień zawarcia umowy jest załącznikiem do każdej umowy pośrednictwa i każdej umowy
-- o zarządzanie (art. 181 ust. 3a i art. 186 ust. 3a ugn, rozstrzygnięcie biura R6).
--
-- Kierownik pilnuje daty końca, przypomina o niej właścicielowi biura 30 dni wcześniej i o nowej
-- polisie zawiadamia tego samego dnia Kierownika pośrednictwa (polisa pośrednika) albo Kierownika
-- najmu (polisa zarządcy). Kopia leży w dokumentach biura, a wiersz trzyma do niej odwołanie. Nowa
-- polisa albo zmiana jej danych to nowy wiersz.
CREATE TABLE kierownik_biura_polisy_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_biura_polisy_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('posrednika', 'zarzadcy')),
  numer TEXT NOT NULL CHECK (numer <> ''),
  ubezpieczyciel TEXT NOT NULL CHECK (ubezpieczyciel <> ''),
  -- Okres ubezpieczenia, RRRR-MM-DD, oba dni włącznie.
  okres_od TEXT NOT NULL,
  okres_do TEXT NOT NULL,
  -- Kopia polisy w dokumentach biura, tekstem.
  dokument_odnosnik TEXT NOT NULL CHECK (dokument_odnosnik <> ''),
  zapisano TEXT NOT NULL,
  -- Kiedy kierownik przypomniał właścicielowi biura o końcu polisy.
  przypomniano TEXT,
  -- Kiedy i którego kierownika działu zawiadomił o tej polisie.
  zawiadomiono TEXT,
  zawiadomiono_kogo TEXT CHECK (zawiadomiono_kogo IN ('kierownik_posrednictwa', 'kierownik_najmu')),

  CONSTRAINT okres_od_przed_do CHECK (okres_od < okres_do),
  CONSTRAINT zawiadomienie_w_komplecie CHECK ((zawiadomiono IS NULL) = (zawiadomiono_kogo IS NULL)),
  -- O polisie pośrednika wie Pośrednictwo, a o polisie zarządcy Najem.
  CONSTRAINT zawiadomiony_kierownik_wlasciwy_dla_polisy CHECK (
    zawiadomiono_kogo IS NULL
    OR (rodzaj = 'posrednika') = (zawiadomiono_kogo = 'kierownik_posrednictwa')
  )
) STRICT;

CREATE TRIGGER kierownik_biura_polisy_powstaje_bez_zawiadomien
BEFORE INSERT ON kierownik_biura_polisy_wpisy
WHEN NEW.przypomniano IS NOT NULL
  OR NEW.zawiadomiono IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'polisa powstaje bez przypomnienia i zawiadomienia; odnotowuje się je później');
END;

-- Dane polisy pochodzą z dokumentu, więc się nie zmieniają; przypomnienie i zawiadomienie
-- odnotowuje się raz.
CREATE TRIGGER kierownik_biura_polisy_bez_zmian
BEFORE UPDATE ON kierownik_biura_polisy_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.numer IS NOT OLD.numer
  OR NEW.ubezpieczyciel IS NOT OLD.ubezpieczyciel
  OR NEW.okres_od IS NOT OLD.okres_od
  OR NEW.okres_do IS NOT OLD.okres_do
  OR NEW.dokument_odnosnik IS NOT OLD.dokument_odnosnik
  OR NEW.zapisano IS NOT OLD.zapisano
  OR (OLD.przypomniano IS NOT NULL AND NEW.przypomniano IS NOT OLD.przypomniano)
  OR (OLD.zawiadomiono IS NOT NULL AND (
    NEW.zawiadomiono IS NOT OLD.zawiadomiono
    OR NEW.zawiadomiono_kogo IS NOT OLD.zawiadomiono_kogo
  ))
BEGIN
  SELECT RAISE(ABORT, 'danych polisy się nie zmienia; nowa polisa to nowy wpis, a przypomnienie i zawiadomienie odnotowuje się raz');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_biura_polisy_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_biura_polisy_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_biura_polisy_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: dawna polisa zostaje, bo była załącznikiem umów zawartych w jej okresie.
CREATE TRIGGER kierownik_biura_polisy_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_biura_polisy_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
