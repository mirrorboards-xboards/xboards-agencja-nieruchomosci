-- Profile poszukiwania: czego klient szuka, ile zapłaci i skąd weźmie pieniądze, w wersjach
-- z potwierdzeniem klienta. Pisze Opiekun poszukującego. Kwoty w groszach; budżet przy najmie to
-- najwyższy czynsz miesięczny.
--
-- Opiekun zapisuje wersję z cechami i finansowaniem, a klient ją potwierdza zgodą
-- potwierdzenie_profilu, która dotyczy tej wersji. Potwierdzona wersja się nie zmienia i nie
-- dostaje nowych cech; zmiana profilu to nowa wersja. Propozycje porównuje się z bieżącą
-- potwierdzoną wersją, a etap propozycje zaczyna się dopiero przy potwierdzonym profilu.
--
-- Budżet i finansowanie nie wychodzą poza teczkę poszukiwania (R1); pilnuje tego aplikacja.

-- Profil jednej szukanej nieruchomości; kilka profili ma klient, który kupuje kilka nieruchomości
-- na jednej umowie.
CREATE TABLE poszukiwanie_profile_wpisy (
  id TEXT PRIMARY KEY,
  nazwa TEXT NOT NULL
) STRICT;

CREATE TABLE poszukiwanie_profile_wersje (
  id TEXT PRIMARY KEY,
  profil_id TEXT NOT NULL REFERENCES poszukiwanie_profile_wpisy (id),

  -- Czego klient szuka. Rynek wtórny; zakup od dewelopera nie należy do tej teczki.
  cel TEXT NOT NULL CHECK (cel IN ('kupno', 'najem')),
  -- Kupno na wynajem, a nie dla siebie.
  na_wynajem INTEGER NOT NULL CHECK (na_wynajem IN (0, 1)),
  -- Np. „mieszkanie albo dom”.
  rodzaj_nieruchomosci TEXT NOT NULL,
  -- Przeznaczenie działki albo lokalu użytkowego, np. zabudowa jednorodzinna albo biuro.
  przeznaczenie TEXT,
  polozenie TEXT NOT NULL,
  -- Czas dojazdu do wskazanego miejsca, np. „do 30 minut do szkoły przy ul. Lipowej”.
  dojazd TEXT,
  -- Powierzchnia w setnych metra kwadratowego, jak w rdzeniu.
  pow_min INTEGER CHECK (pow_min > 0),
  pow_max INTEGER CHECK (pow_max > 0),
  pokoje_min INTEGER CHECK (pokoje_min >= 0),
  pokoje_max INTEGER CHECK (pokoje_max >= 0),
  pietro TEXT,

  -- Kiedy i za ile: dzień, w którym klient musi się wprowadzić, i najwyższa kwota.
  termin TEXT,
  termin_powod TEXT,
  budzet INTEGER NOT NULL CHECK (budzet > 0),
  -- Czy budżet obejmuje koszty dodatkowe: podatek, taksę notarialną albo prowizję.
  budzet_z_kosztami INTEGER NOT NULL CHECK (budzet_z_kosztami IN (0, 1)),

  -- Najem: od kiedy i na jak długo, gospodarstwo, najem okazjonalny (art. 19a ust. 2 UOPL),
  -- najwyższa kaucja i umeblowanie (NULL: obojętne).
  najem_od TEXT,
  okres TEXT,
  osoby INTEGER CHECK (osoby > 0),
  dzieci INTEGER CHECK (dzieci >= 0),
  zwierzeta TEXT,
  okazjonalny_dopuszcza INTEGER CHECK (okazjonalny_dopuszcza IN (0, 1)),
  -- Czy klient ma inny lokal, w którym zamieszka po egzekucji, ze zgodą jego właściciela.
  inny_lokal_ma INTEGER CHECK (inny_lokal_ma IN (0, 1)),
  kaucja_max INTEGER CHECK (kaucja_max >= 0),
  umeblowany INTEGER CHECK (umeblowany IN (0, 1)),

  -- Potwierdzenie klienta: zgoda potwierdzenie_profilu, która dotyczy tej wersji; NULL znaczy
  -- wersję, której klient jeszcze nie potwierdził.
  zgoda_id TEXT REFERENCES sprawa_zgody_wpisy (id),
  od TEXT NOT NULL,

  CONSTRAINT na_wynajem_przy_kupnie CHECK (cel = 'kupno' OR na_wynajem = 0),
  CONSTRAINT powierzchnia_od_do CHECK (pow_min IS NULL OR pow_max IS NULL OR pow_min <= pow_max),
  CONSTRAINT pokoje_od_do CHECK (pokoje_min IS NULL OR pokoje_max IS NULL OR pokoje_min <= pokoje_max),
  CONSTRAINT powod_przy_terminie CHECK (termin_powod IS NULL OR termin IS NOT NULL),
  CONSTRAINT najem_ma_okres_osoby_i_rodzaj CHECK (
    cel <> 'najem'
    OR (najem_od IS NOT NULL AND osoby IS NOT NULL AND okazjonalny_dopuszcza IS NOT NULL)
  ),
  CONSTRAINT dane_najmu_tylko_przy_najmie CHECK (
    cel = 'najem'
    OR (
      najem_od IS NULL AND okres IS NULL AND osoby IS NULL AND dzieci IS NULL AND zwierzeta IS NULL
      AND okazjonalny_dopuszcza IS NULL AND inny_lokal_ma IS NULL AND kaucja_max IS NULL
      AND umeblowany IS NULL
    )
  ),
  -- Przed propozycją najmu okazjonalnego Opiekun pyta, czy klient ma inny lokal.
  CONSTRAINT okazjonalny_z_odpowiedzia_o_innym_lokalu CHECK (
    okazjonalny_dopuszcza IS NOT 1 OR inny_lokal_ma IS NOT NULL
  )
) STRICT;

-- Cechy z oznaczeniem, czy są warunkiem, czy życzeniem, np. „balkon” jako warunek.
CREATE TABLE poszukiwanie_profile_cechy (
  -- Klucz nadany przez bazę jest dodatni; zob. poszukiwanie_profile_cechy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  wersja_id TEXT NOT NULL REFERENCES poszukiwanie_profile_wersje (id),
  cecha TEXT NOT NULL,
  waga TEXT NOT NULL CHECK (waga IN ('warunek', 'zyczenie')),

  UNIQUE (wersja_id, cecha)
) STRICT;

-- Źródła pieniędzy z kwotą każdego według deklaracji klienta.
CREATE TABLE poszukiwanie_profile_finansowanie (
  -- Klucz nadany przez bazę jest dodatni; zob. poszukiwanie_profile_finansowanie_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  wersja_id TEXT NOT NULL REFERENCES poszukiwanie_profile_wersje (id),
  zrodlo TEXT NOT NULL CHECK (zrodlo IN ('oszczednosci', 'kredyt', 'sprzedaz_nieruchomosci', 'inne')),
  kwota INTEGER NOT NULL CHECK (kwota > 0),

  UNIQUE (wersja_id, zrodlo)
) STRICT;

-- Bieżący profil: najnowsza potwierdzona wersja każdego profilu.
CREATE VIEW poszukiwanie_profile_biezace AS
SELECT wersja.*
FROM poszukiwanie_profile_wersje AS wersja
WHERE wersja.zgoda_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM poszukiwanie_profile_wersje AS nowsza
    WHERE nowsza.profil_id = wersja.profil_id
      AND nowsza.zgoda_id IS NOT NULL
      AND nowsza.od > wersja.od
  );

-- Potwierdzenie sprawdza zmianę, więc wersja powstaje niepotwierdzona; jest późniejsza niż każda
-- wcześniejsza wersja tego profilu.
CREATE TRIGGER poszukiwanie_profile_wersja_powstaje_niepotwierdzona
BEFORE INSERT ON poszukiwanie_profile_wersje
WHEN NEW.zgoda_id IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'wersja profilu powstaje niepotwierdzona; potwierdza ją potem klient');
END;

CREATE TRIGGER poszukiwanie_profile_wersje_kolejno
BEFORE INSERT ON poszukiwanie_profile_wersje
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_profile_wersje
  WHERE profil_id = NEW.profil_id
    AND od >= NEW.od
)
BEGIN
  SELECT RAISE(ABORT, 'nowa wersja profilu jest późniejsza niż poprzednie wersje tego profilu');
END;

-- Profil potwierdza klient: zgoda potwierdzenie_profilu uczestnika z obowiązującą rolą klient,
-- która dotyczy tej wersji.
CREATE TRIGGER poszukiwanie_profile_potwierdzenie_ze_zgoda_klienta
BEFORE UPDATE OF zgoda_id ON poszukiwanie_profile_wersje
WHEN NEW.zgoda_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_zgody_wpisy AS zgoda
    JOIN sprawa_uczestnicy_role AS rola ON rola.uczestnik_id = zgoda.uczestnik_id
    WHERE zgoda.id = NEW.zgoda_id
      AND zgoda.rodzaj = 'potwierdzenie_profilu'
      AND zgoda.dotyczy_tabela = 'poszukiwanie_profile_wersje'
      AND zgoda.dotyczy_id = NEW.id
      AND rola.rola = 'klient'
      AND rola.zakonczono IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'wersja profilu ma zgodę potwierdzenie_profilu klienta, która jej dotyczy');
END;

-- Wersji się nie zmienia; raz wpisuje się tylko potwierdzenie klienta.
CREATE TRIGGER poszukiwanie_profile_wersje_bez_zmian
BEFORE UPDATE ON poszukiwanie_profile_wersje
WHEN OLD.zgoda_id IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.profil_id IS NOT OLD.profil_id
  OR NEW.cel IS NOT OLD.cel
  OR NEW.na_wynajem IS NOT OLD.na_wynajem
  OR NEW.rodzaj_nieruchomosci IS NOT OLD.rodzaj_nieruchomosci
  OR NEW.przeznaczenie IS NOT OLD.przeznaczenie
  OR NEW.polozenie IS NOT OLD.polozenie
  OR NEW.dojazd IS NOT OLD.dojazd
  OR NEW.pow_min IS NOT OLD.pow_min
  OR NEW.pow_max IS NOT OLD.pow_max
  OR NEW.pokoje_min IS NOT OLD.pokoje_min
  OR NEW.pokoje_max IS NOT OLD.pokoje_max
  OR NEW.pietro IS NOT OLD.pietro
  OR NEW.termin IS NOT OLD.termin
  OR NEW.termin_powod IS NOT OLD.termin_powod
  OR NEW.budzet IS NOT OLD.budzet
  OR NEW.budzet_z_kosztami IS NOT OLD.budzet_z_kosztami
  OR NEW.najem_od IS NOT OLD.najem_od
  OR NEW.okres IS NOT OLD.okres
  OR NEW.osoby IS NOT OLD.osoby
  OR NEW.dzieci IS NOT OLD.dzieci
  OR NEW.zwierzeta IS NOT OLD.zwierzeta
  OR NEW.okazjonalny_dopuszcza IS NOT OLD.okazjonalny_dopuszcza
  OR NEW.inny_lokal_ma IS NOT OLD.inny_lokal_ma
  OR NEW.kaucja_max IS NOT OLD.kaucja_max
  OR NEW.umeblowany IS NOT OLD.umeblowany
  OR NEW.od IS NOT OLD.od
BEGIN
  SELECT RAISE(ABORT, 'wersji profilu się nie zmienia; raz wpisuje się potwierdzenie klienta, a zmiana to nowa wersja');
END;

-- Cechy i finansowanie są częścią tego, co klient potwierdza, więc dopisuje się je tylko do
-- niepotwierdzonej wersji.
CREATE TRIGGER poszukiwanie_profile_cechy_przed_potwierdzeniem
BEFORE INSERT ON poszukiwanie_profile_cechy
WHEN (
  SELECT zgoda_id IS NOT NULL
  FROM poszukiwanie_profile_wersje
  WHERE id = NEW.wersja_id
)
BEGIN
  SELECT RAISE(ABORT, 'cechy i finansowanie dopisuje się tylko do niepotwierdzonej wersji profilu');
END;

CREATE TRIGGER poszukiwanie_profile_finansowanie_przed_potwierdzeniem
BEFORE INSERT ON poszukiwanie_profile_finansowanie
WHEN (
  SELECT zgoda_id IS NOT NULL
  FROM poszukiwanie_profile_wersje
  WHERE id = NEW.wersja_id
)
BEGIN
  SELECT RAISE(ABORT, 'cechy i finansowanie dopisuje się tylko do niepotwierdzonej wersji profilu');
END;

CREATE TRIGGER poszukiwanie_profile_cechy_bez_zmian
BEFORE UPDATE ON poszukiwanie_profile_cechy
BEGIN
  SELECT RAISE(ABORT, 'cechy profilu się nie zmienia; zmiana profilu to nowa wersja');
END;

CREATE TRIGGER poszukiwanie_profile_finansowanie_bez_zmian
BEFORE UPDATE ON poszukiwanie_profile_finansowanie
BEGIN
  SELECT RAISE(ABORT, 'finansowania profilu się nie zmienia; zmiana profilu to nowa wersja');
END;

CREATE TRIGGER poszukiwanie_profile_wpisy_bez_zmian
BEFORE UPDATE OF id ON poszukiwanie_profile_wpisy
WHEN NEW.id IS NOT OLD.id
BEGIN
  SELECT RAISE(ABORT, 'identyfikator profilu się nie zmienia');
END;

-- Propozycje zaczynają się przy profilu, który klient potwierdził.
CREATE TRIGGER poszukiwanie_profile_propozycje_po_potwierdzeniu
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap = 'propozycje'
  AND NOT EXISTS (
    SELECT 1
    FROM poszukiwanie_profile_wersje
    WHERE zgoda_id IS NOT NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'etap propozycje wymaga profilu poszukiwania potwierdzonego przez klienta');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER poszukiwanie_profile_wpisy_bez_zastepowania
BEFORE INSERT ON poszukiwanie_profile_wpisy
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_profile_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_profile_wpisy_bez_usuwania
BEFORE DELETE ON poszukiwanie_profile_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER poszukiwanie_profile_wersje_bez_zastepowania
BEFORE INSERT ON poszukiwanie_profile_wersje
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_profile_wersje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_profile_wersje_bez_usuwania
BEFORE DELETE ON poszukiwanie_profile_wersje
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER poszukiwanie_profile_cechy_bez_zastepowania
BEFORE INSERT ON poszukiwanie_profile_cechy
WHEN EXISTS (
    SELECT 1
    FROM poszukiwanie_profile_cechy
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM poszukiwanie_profile_cechy
    WHERE wersja_id = NEW.wersja_id
      AND cecha = NEW.cecha
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_profile_cechy_bez_usuwania
BEFORE DELETE ON poszukiwanie_profile_cechy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER poszukiwanie_profile_finansowanie_bez_zastepowania
BEFORE INSERT ON poszukiwanie_profile_finansowanie
WHEN EXISTS (
    SELECT 1
    FROM poszukiwanie_profile_finansowanie
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM poszukiwanie_profile_finansowanie
    WHERE wersja_id = NEW.wersja_id
      AND zrodlo = NEW.zrodlo
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_profile_finansowanie_bez_usuwania
BEFORE DELETE ON poszukiwanie_profile_finansowanie
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
