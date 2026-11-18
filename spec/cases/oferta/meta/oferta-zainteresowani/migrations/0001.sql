-- Zainteresowani ofertą: osoby, które pytają i oglądają, ich zapytania i prezentacje. Pisze
-- Opiekun oferty.
--
-- Zainteresowany nie jest uczestnikiem rdzenia: rdzeń wymaga nazwiska i danych osoby, a profil
-- tylko imienia i kontaktu (minimalizacja danych, art. 5 ust. 1 lit. c RODO). Gdy zainteresowany
-- staje się stroną, wpisuje go teczka transakcji albo najmu.
--
-- Pełny adres z numerem lokalu dostaje tylko osoba umówiona na prezentację, więc jedyna kolumna
-- z chwilą podania adresu leży przy uczestniku prezentacji. Prezentacja to praca według umowy
-- pośrednictwa, więc odbywa się nie wcześniej niż sprawa_umowy_praca_od.

-- Osoba, która pyta albo ogląda, z kontaktem i informacją o przetwarzaniu jej danych
-- (art. 13 ust. 1 RODO): wysyłką z teczki albo datą, w której dostała ją od Recepcji
-- (art. 13 ust. 4 RODO).
CREATE TABLE oferta_zainteresowani_osoby (
  id TEXT PRIMARY KEY,
  imie TEXT NOT NULL,
  nazwisko TEXT,
  telefon TEXT,
  email TEXT,
  -- Sama, przez inne biuro albo jako klient tego biura prowadzony przez Opiekuna poszukującego.
  dziala_jako TEXT NOT NULL CHECK (dziala_jako IN ('sam', 'przez_biuro', 'klient_biura')),
  biuro TEXT,
  -- Dla kogo działa biuro, które przysłało zapytanie, jeśli to podało.
  dla_kogo TEXT,
  dane_pozyskano TEXT NOT NULL,
  informacja_wysylka_id TEXT REFERENCES sprawa_korespondencja_wysylki (id),
  informacja_od_recepcji TEXT,

  CONSTRAINT osoba_ma_kontakt CHECK (telefon IS NOT NULL OR email IS NOT NULL),
  CONSTRAINT przez_biuro_wskazuje_biuro CHECK (dziala_jako <> 'przez_biuro' OR biuro IS NOT NULL),
  CONSTRAINT informacja_wysylka_albo_od_recepcji CHECK (
    informacja_wysylka_id IS NULL OR informacja_od_recepcji IS NULL
  )
) STRICT;

-- Zapytanie o ofertę: wpływ, odpowiedź i to, czy skończyło się prezentacją.
CREATE TABLE oferta_zainteresowani_zapytania (
  -- Klucz nadany przez bazę jest dodatni; zob. oferta_zainteresowani_zapytania_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  osoba_id TEXT NOT NULL REFERENCES oferta_zainteresowani_osoby (id),
  wplyw_id TEXT NOT NULL REFERENCES sprawa_korespondencja_wplywy (id),
  odpowiedz_wysylka_id TEXT REFERENCES sprawa_korespondencja_wysylki (id),
  skonczylo_sie_prezentacja INTEGER CHECK (skonczylo_sie_prezentacja IN (0, 1))
) STRICT;

-- Prezentacja w kalendarzu pośrednika; dzień otwarty to jedna prezentacja z wieloma
-- uczestnikami.
CREATE TABLE oferta_zainteresowani_prezentacje (
  id TEXT PRIMARY KEY,
  termin TEXT NOT NULL,
  posrednik TEXT NOT NULL,
  kalendarz_odnosnik TEXT,
  -- Kto otwiera lokal: klient, pośrednik z kluczami biura albo najemca, który w nim mieszka.
  otwiera TEXT NOT NULL CHECK (otwiera IN ('klient', 'posrednik', 'najemca')),
  -- Notatka po prezentacji: kto przyszedł, co oglądał, o co pytał i jak ocenił nieruchomość.
  przebieg TEXT,
  odwolano TEXT,
  odwolal TEXT,
  powod_odwolania TEXT,

  CONSTRAINT odwolanie_kto_kiedy_dlaczego CHECK (
    (odwolano IS NULL) = (odwolal IS NULL)
    AND (odwolano IS NULL) = (powod_odwolania IS NULL)
  ),
  CONSTRAINT odwolana_bez_przebiegu CHECK (odwolano IS NULL OR przebieg IS NULL)
) STRICT;

CREATE TABLE oferta_zainteresowani_prezentacje_uczestnicy (
  -- Klucz nadany przez bazę jest dodatni; zob.
  -- oferta_zainteresowani_prezentacje_uczestnicy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  prezentacja_id TEXT NOT NULL REFERENCES oferta_zainteresowani_prezentacje (id),
  osoba_id TEXT NOT NULL REFERENCES oferta_zainteresowani_osoby (id),
  -- NULL, dopóki prezentacja się nie odbyła; 0 znaczy, że uczestnik nie przyszedł.
  przyszedl INTEGER CHECK (przyszedl IN (0, 1)),
  -- Chwila, w której Opiekun podał uczestnikowi pełny adres.
  adres_podano TEXT,

  UNIQUE (prezentacja_id, osoba_id)
) STRICT;

-- Informacja z teczki to wysłana wysyłka ze znacznikiem informacja_rodo.
CREATE TRIGGER oferta_zainteresowani_informacja_to_wysylka_rodo
BEFORE INSERT ON oferta_zainteresowani_osoby
WHEN NEW.informacja_wysylka_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_korespondencja_wysylki
    WHERE id = NEW.informacja_wysylka_id
      AND informacja_rodo = 1
      AND stan IN ('wyslana', 'doreczona')
  )
BEGIN
  SELECT RAISE(ABORT, 'informacja o danych to wysłana wysyłka ze znacznikiem informacja_rodo');
END;

CREATE TRIGGER oferta_zainteresowani_zmieniona_informacja_to_wysylka_rodo
BEFORE UPDATE OF informacja_wysylka_id ON oferta_zainteresowani_osoby
WHEN NEW.informacja_wysylka_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_korespondencja_wysylki
    WHERE id = NEW.informacja_wysylka_id
      AND informacja_rodo = 1
      AND stan IN ('wyslana', 'doreczona')
  )
BEGIN
  SELECT RAISE(ABORT, 'informacja o danych to wysłana wysyłka ze znacznikiem informacja_rodo');
END;

-- Informację o danych wpisuje się raz; kontakt i opis osoby wolno poprawić.
CREATE TRIGGER oferta_zainteresowani_informacja_raz
BEFORE UPDATE ON oferta_zainteresowani_osoby
WHEN NEW.id IS NOT OLD.id
  OR (OLD.informacja_wysylka_id IS NOT NULL AND NEW.informacja_wysylka_id IS NOT OLD.informacja_wysylka_id)
  OR (OLD.informacja_od_recepcji IS NOT NULL AND NEW.informacja_od_recepcji IS NOT OLD.informacja_od_recepcji)
BEGIN
  SELECT RAISE(ABORT, 'informację o danych zainteresowanego wpisuje się raz');
END;

-- Odpowiedź na zapytanie jest wiadomością do osoby, więc przychodzi po informacji o jej danych
-- albo razem z nią (art. 13 ust. 1 i art. 14 ust. 3 lit. b RODO).
CREATE TRIGGER oferta_zainteresowani_odpowiedz_po_informacji
BEFORE UPDATE OF odpowiedz_wysylka_id ON oferta_zainteresowani_zapytania
WHEN NEW.odpowiedz_wysylka_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM oferta_zainteresowani_osoby
    WHERE id = NEW.osoba_id
      AND (informacja_wysylka_id IS NOT NULL OR informacja_od_recepcji IS NOT NULL)
  )
BEGIN
  SELECT RAISE(ABORT, 'odpowiedź na zapytanie wymaga informacji o danych zainteresowanego');
END;

-- Zapytanie powstaje bez odpowiedzi i wyniku; oba wpisuje się potem raz.
CREATE TRIGGER oferta_zainteresowani_zapytanie_powstaje_bez_odpowiedzi
BEFORE INSERT ON oferta_zainteresowani_zapytania
WHEN NEW.odpowiedz_wysylka_id IS NOT NULL
  OR NEW.skonczylo_sie_prezentacja IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'zapytanie powstaje bez odpowiedzi i wyniku');
END;

CREATE TRIGGER oferta_zainteresowani_zapytania_odpowiedz_i_wynik_raz
BEFORE UPDATE ON oferta_zainteresowani_zapytania
WHEN NEW.id IS NOT OLD.id
  OR NEW.osoba_id IS NOT OLD.osoba_id
  OR NEW.wplyw_id IS NOT OLD.wplyw_id
  OR (OLD.odpowiedz_wysylka_id IS NOT NULL AND NEW.odpowiedz_wysylka_id IS NOT OLD.odpowiedz_wysylka_id)
  OR (OLD.skonczylo_sie_prezentacja IS NOT NULL AND NEW.skonczylo_sie_prezentacja IS NOT OLD.skonczylo_sie_prezentacja)
BEGIN
  SELECT RAISE(ABORT, 'zapytania się nie zmienia; odpowiedź i wynik wpisuje się raz');
END;

-- Prezentacja to praca według zawartej umowy pośrednictwa, więc jej termin nie jest wcześniejszy
-- niż chwila, od której biuro może pracować (art. 15 ust. 3 i art. 21 ust. 2 upk).
CREATE TRIGGER oferta_zainteresowani_prezentacja_od_pracy
BEFORE INSERT ON oferta_zainteresowani_prezentacje
WHEN NOT EXISTS (
  SELECT 1
  FROM sprawa_umowy_wpisy AS umowa
  JOIN sprawa_umowy_praca_od AS praca ON praca.umowa_id = umowa.id
  WHERE umowa.rodzaj = 'posrednictwo'
    AND umowa.stan = 'zawarta'
    AND NEW.termin >= praca.od
)
BEGIN
  SELECT RAISE(ABORT, 'prezentacja wymaga zawartej umowy pośrednictwa i odbywa się nie wcześniej niż praca od');
END;

-- Prezentacja powstaje umówiona; przebieg albo odwołanie wpisuje się potem raz. Inny termin to
-- odwołanie tej prezentacji i nowa prezentacja.
CREATE TRIGGER oferta_zainteresowani_prezentacja_powstaje_umowiona
BEFORE INSERT ON oferta_zainteresowani_prezentacje
WHEN NEW.przebieg IS NOT NULL
  OR NEW.odwolano IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'prezentacja powstaje umówiona, bez przebiegu i odwołania');
END;

CREATE TRIGGER oferta_zainteresowani_prezentacje_bez_zmian
BEFORE UPDATE ON oferta_zainteresowani_prezentacje
WHEN OLD.przebieg IS NOT NULL
  OR OLD.odwolano IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.termin IS NOT OLD.termin
  OR NEW.posrednik IS NOT OLD.posrednik
  OR NEW.kalendarz_odnosnik IS NOT OLD.kalendarz_odnosnik
  OR NEW.otwiera IS NOT OLD.otwiera
BEGIN
  SELECT RAISE(ABORT, 'prezentacji się nie zmienia; raz wpisuje się przebieg albo odwołanie');
END;

-- Obecność i chwilę podania adresu wpisuje się raz.
CREATE TRIGGER oferta_zainteresowani_uczestnicy_raz
BEFORE UPDATE ON oferta_zainteresowani_prezentacje_uczestnicy
WHEN NEW.id IS NOT OLD.id
  OR NEW.prezentacja_id IS NOT OLD.prezentacja_id
  OR NEW.osoba_id IS NOT OLD.osoba_id
  OR (OLD.przyszedl IS NOT NULL AND NEW.przyszedl IS NOT OLD.przyszedl)
  OR (OLD.adres_podano IS NOT NULL AND NEW.adres_podano IS NOT OLD.adres_podano)
BEGIN
  SELECT RAISE(ABORT, 'uczestnika prezentacji się nie zmienia; obecność i podanie adresu wpisuje się raz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER oferta_zainteresowani_osoby_bez_zastepowania
BEFORE INSERT ON oferta_zainteresowani_osoby
WHEN EXISTS (
  SELECT 1
  FROM oferta_zainteresowani_osoby
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_zainteresowani_osoby_bez_usuwania
BEFORE DELETE ON oferta_zainteresowani_osoby
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER oferta_zainteresowani_zapytania_bez_zastepowania
BEFORE INSERT ON oferta_zainteresowani_zapytania
WHEN EXISTS (
  SELECT 1
  FROM oferta_zainteresowani_zapytania
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_zainteresowani_zapytania_bez_usuwania
BEFORE DELETE ON oferta_zainteresowani_zapytania
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER oferta_zainteresowani_prezentacje_bez_zastepowania
BEFORE INSERT ON oferta_zainteresowani_prezentacje
WHEN EXISTS (
  SELECT 1
  FROM oferta_zainteresowani_prezentacje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_zainteresowani_prezentacje_bez_usuwania
BEFORE DELETE ON oferta_zainteresowani_prezentacje
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER oferta_zainteresowani_prezentacje_uczestnicy_bez_zastepowania
BEFORE INSERT ON oferta_zainteresowani_prezentacje_uczestnicy
WHEN EXISTS (
    SELECT 1
    FROM oferta_zainteresowani_prezentacje_uczestnicy
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM oferta_zainteresowani_prezentacje_uczestnicy
    WHERE prezentacja_id = NEW.prezentacja_id
      AND osoba_id = NEW.osoba_id
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_zainteresowani_prezentacje_uczestnicy_bez_usuwania
BEFORE DELETE ON oferta_zainteresowani_prezentacje_uczestnicy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
