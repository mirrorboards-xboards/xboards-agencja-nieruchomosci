-- Listy decyzji: pozycje z próśb kierowników działów o decyzję człowieka, po jednej liście dla
-- każdego, kto decyduje. Samej prośby kierownik nie widzi: pozycja wskazuje jej wiersz w teczce
-- i niesie opis kierownika działu jako jego słowa, a nie rekomendację Kierownika biura. Prośby
-- Kontroli nie trafiają na listę, bo Kontrola przekazuje je właścicielowi sama.
--
-- Ta sama prośba przysłana ponownie odnawia pozycję, a nie ją podwaja (błędne koło). Zdjętych
-- pozycji się nie wykreśla, bo z nich widać, jak długo decyzje czekały.
CREATE TABLE kierownik_biura_decyzje_pozycje (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_biura_decyzje_pozycje_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Prośba o decyzję w teczce, tekstem: pokój i wiersz.
  prosba_odnosnik TEXT NOT NULL CHECK (prosba_odnosnik <> ''),
  dzial TEXT NOT NULL CHECK (dzial IN ('posrednictwo', 'transakcje', 'najem')),
  -- Tożsamość człowieka, który decyduje.
  decyduje TEXT NOT NULL CHECK (decyduje <> ''),
  opis TEXT NOT NULL CHECK (opis <> ''),
  -- Termin podany przez kierownika działu; decyduje o kolejności na liście.
  termin TEXT,
  przyjeto TEXT NOT NULL,
  odnowiono TEXT,
  -- Kiedy pozycja pierwszy raz trafiła do człowieka; kierownik mówi o tym kierownikowi działu.
  trafila_do_czlowieka TEXT,
  -- Kiedy kierownik zgłosił człowiekowi osobno decyzję, która czeka za długo.
  zgloszono_osobno TEXT,
  zdjeto TEXT,
  powod_zdjecia TEXT CHECK (powod_zdjecia IN ('rozstrzygnieta', 'zbedna')),

  CONSTRAINT odnowienie_nie_przed_przyjeciem CHECK (odnowiono IS NULL OR odnowiono >= przyjeto),
  CONSTRAINT do_czlowieka_nie_przed_przyjeciem CHECK (trafila_do_czlowieka IS NULL OR trafila_do_czlowieka >= przyjeto),
  CONSTRAINT zdjecie_z_powodem CHECK ((zdjeto IS NULL) = (powod_zdjecia IS NULL)),
  CONSTRAINT zdjecie_nie_przed_przyjeciem CHECK (zdjeto IS NULL OR zdjeto >= przyjeto)
) STRICT;

-- Jedna niezdjęta pozycja na prośbę; ponowna prośba ją odnawia.
CREATE UNIQUE INDEX kierownik_biura_decyzje_jedna_pozycja_na_prosbe
ON kierownik_biura_decyzje_pozycje (prosba_odnosnik)
WHERE zdjeto IS NULL;

-- Druga niezdjęta pozycja z innym kluczem zderzyłaby się z indeksem, a INSERT OR REPLACE usunąłby
-- wtedy pierwszą z ominięciem blokady usuwania; ten trigger odmawia wcześniej.
CREATE TRIGGER kierownik_biura_decyzje_prosba_raz_na_liscie
BEFORE INSERT ON kierownik_biura_decyzje_pozycje
WHEN EXISTS (
  SELECT 1
  FROM kierownik_biura_decyzje_pozycje
  WHERE prosba_odnosnik = NEW.prosba_odnosnik
    AND zdjeto IS NULL
    AND id IS NOT NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'ta prośba jest już na liście; ponowna prośba odnawia pozycję');
END;

CREATE TRIGGER kierownik_biura_decyzje_powstaje_na_liscie
BEFORE INSERT ON kierownik_biura_decyzje_pozycje
WHEN NEW.zdjeto IS NOT NULL
  OR NEW.odnowiono IS NOT NULL
  OR NEW.trafila_do_czlowieka IS NOT NULL
  OR NEW.zgloszono_osobno IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'pozycja powstaje na liście, jeszcze nieodnowiona i niewysłana');
END;

-- Prośba, dział, człowiek i chwila przyjęcia się nie zmieniają; wysłanie do człowieka i osobne
-- zgłoszenie odnotowuje się raz, a zdjętej pozycji nie zmienia się wcale.
CREATE TRIGGER kierownik_biura_decyzje_bez_zmian
BEFORE UPDATE ON kierownik_biura_decyzje_pozycje
WHEN OLD.zdjeto IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.prosba_odnosnik IS NOT OLD.prosba_odnosnik
  OR NEW.dzial IS NOT OLD.dzial
  OR NEW.decyduje IS NOT OLD.decyduje
  OR NEW.przyjeto IS NOT OLD.przyjeto
  OR (OLD.trafila_do_czlowieka IS NOT NULL AND NEW.trafila_do_czlowieka IS NOT OLD.trafila_do_czlowieka)
  OR (OLD.zgloszono_osobno IS NOT NULL AND NEW.zgloszono_osobno IS NOT OLD.zgloszono_osobno)
BEGIN
  SELECT RAISE(ABORT, 'prośby, działu, człowieka ani przyjęcia pozycji się nie zmienia, a zdjętej pozycji wcale');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_biura_decyzje_pozycje_bez_zastepowania
BEFORE INSERT ON kierownik_biura_decyzje_pozycje
WHEN EXISTS (
  SELECT 1
  FROM kierownik_biura_decyzje_pozycje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: zdjętych pozycji się nie wykreśla.
CREATE TRIGGER kierownik_biura_decyzje_pozycje_bez_usuwania
BEFORE DELETE ON kierownik_biura_decyzje_pozycje
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); zdjętych pozycji się nie wykreśla');
END;
