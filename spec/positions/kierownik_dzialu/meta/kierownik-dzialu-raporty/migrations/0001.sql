-- Raporty agentów działu o stałych polach i prośby o decyzje, które kierownik przekazał dalej.
--
-- Raport nie powtarza treści prośby: wskazuje jej wiersz w teczce. Kierownik kontroli przekazuje
-- prośby właścicielowi sam, z pominięciem listy decyzji Kierownika biura, bo ich treść, a nawet to,
-- że są, może dotyczyć przeciwdziałania praniu pieniędzy; pozostali kierownicy przekazują je
-- Kierownikowi biura.
CREATE TABLE kierownik_dzialu_raporty_wpisy (
  id TEXT PRIMARY KEY,
  agent_id TEXT NOT NULL REFERENCES kierownik_dzialu_agenci_wpisy (id),
  otrzymano TEXT NOT NULL,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('biezacy', 'koncowy', 'okresowy', 'natychmiastowy')),
  etap_albo_okres TEXT NOT NULL,
  blokada TEXT,
  najblizszy_termin TEXT,
  -- Prośba o decyzję w teczce, tekstem: pokój i wiersz.
  decyzja_odnosnik TEXT,
  -- To, co raport mówi poza stałymi polami, np. jak skończyła się oferta.
  uwagi TEXT
) STRICT;

CREATE TABLE kierownik_dzialu_raporty_prosby (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_dzialu_raporty_prosby_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  decyzja_odnosnik TEXT NOT NULL,
  raport_id TEXT NOT NULL REFERENCES kierownik_dzialu_raporty_wpisy (id),
  przekazano_do TEXT NOT NULL CHECK (przekazano_do IN ('kierownik_biura', 'wlasciciel')),
  przekazano TEXT NOT NULL,
  -- Kiedy kierownik zgłosił, że prośbę można zdjąć z listy, i dlaczego: rozstrzygnięta albo zbędna.
  zgloszono_zdjecie TEXT,
  powod_zdjecia TEXT CHECK (powod_zdjecia IN ('rozstrzygnieta', 'zbedna')),

  CONSTRAINT zdjecie_z_powodem CHECK ((zgloszono_zdjecie IS NULL) = (powod_zdjecia IS NULL)),
  CONSTRAINT zdjecie_nie_przed_przekazaniem CHECK (zgloszono_zdjecie IS NULL OR zgloszono_zdjecie >= przekazano)
) STRICT;

-- Tej samej prośby nie przekazuje się drugi raz, dopóki nie zgłoszono jej zdjęcia: powtórzona prośba
-- naciska na człowieka (błędne koło).
CREATE UNIQUE INDEX kierownik_dzialu_raporty_prosba_przekazana_raz
ON kierownik_dzialu_raporty_prosby (decyzja_odnosnik)
WHERE zgloszono_zdjecie IS NULL;

-- Druga otwarta prośba z innym kluczem zderzyłaby się z indeksem, a INSERT OR REPLACE usunąłby wtedy
-- pierwszą z ominięciem blokady usuwania; ten trigger odmawia wcześniej.
CREATE TRIGGER kierownik_dzialu_raporty_prosba_przekazana_raz_przed_zdjeciem
BEFORE INSERT ON kierownik_dzialu_raporty_prosby
WHEN NEW.zgloszono_zdjecie IS NULL
  AND EXISTS (
    SELECT 1
    FROM kierownik_dzialu_raporty_prosby
    WHERE decyzja_odnosnik = NEW.decyzja_odnosnik
      AND zgloszono_zdjecie IS NULL
      AND id IS NOT NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'tej prośby nie przekazuje się drugi raz, dopóki nie zgłoszono jej zdjęcia');
END;

CREATE TRIGGER kierownik_dzialu_raporty_prosba_powstaje_na_liscie
BEFORE INSERT ON kierownik_dzialu_raporty_prosby
WHEN NEW.zgloszono_zdjecie IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'przekazana prośba powstaje na liście; zdjęcie zgłasza się później');
END;

CREATE TRIGGER kierownik_dzialu_raporty_bez_zmian
BEFORE UPDATE ON kierownik_dzialu_raporty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'raporty tylko się dopisuje; zmiany nie są dozwolone');
END;

CREATE TRIGGER kierownik_dzialu_raporty_prosby_bez_zmian
BEFORE UPDATE ON kierownik_dzialu_raporty_prosby
WHEN OLD.zgloszono_zdjecie IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.decyzja_odnosnik IS NOT OLD.decyzja_odnosnik
  OR NEW.raport_id IS NOT OLD.raport_id
  OR NEW.przekazano_do IS NOT OLD.przekazano_do
  OR NEW.przekazano IS NOT OLD.przekazano
BEGIN
  SELECT RAISE(ABORT, 'przekazanej prośby się nie zmienia; wolno raz zgłosić jej zdjęcie');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabeli z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER kierownik_dzialu_raporty_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_dzialu_raporty_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_dzialu_raporty_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER kierownik_dzialu_raporty_prosby_bez_zastepowania
BEFORE INSERT ON kierownik_dzialu_raporty_prosby
WHEN EXISTS (
  SELECT 1
  FROM kierownik_dzialu_raporty_prosby
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: raporty i prośby są śladem pracy działu.
CREATE TRIGGER kierownik_dzialu_raporty_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_dzialu_raporty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER kierownik_dzialu_raporty_prosby_bez_usuwania
BEFORE DELETE ON kierownik_dzialu_raporty_prosby
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
