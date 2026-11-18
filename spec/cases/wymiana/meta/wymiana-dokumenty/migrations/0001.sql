-- Dokumenty pokoju wymiany: lista dokumentów kancelarii, dokumenty z tej listy, pełnomocnictwa
-- i zgody stron od biura, projekt aktu w wersjach i koszty od kancelarii oraz uwagi stron do
-- projektu. Każdy wiersz jest szczegółem jednego wpisu z wymiana_wpisy_wpisy, a stronę, autora,
-- czas i plik ma wpis. Wiersz dopisuje się razem z jego wpisem i potem się nie zmienia.
--
-- Biuro pisze w imieniu strony tylko z jej pełnomocnictwem (art. 98 KC; zasada „Pisanie bez
-- pełnomocnictwa” w profilu pokoju), więc dokument i zgoda strony wymagają wcześniejszego
-- pełnomocnictwa tej strony.

-- Lista dokumentów, których notariusz wymaga do aktu. Zmiana listy jest nowym wpisem z całą
-- listą; pozycja zachowuje identyfikator między wersjami listy, a dokument go wskazuje.
CREATE TABLE wymiana_dokumenty_lista (
  wpis_id TEXT NOT NULL REFERENCES wymiana_wpisy_wpisy (id),
  -- Identyfikator pozycji nadany przez kancelarię, np. „odpis-kw”.
  pozycja_id TEXT NOT NULL CHECK (pozycja_id <> ''),
  nazwa TEXT NOT NULL CHECK (nazwa <> ''),
  -- Strona, od której dokument ma pochodzić; NULL znaczy, że od którejkolwiek.
  od_strony_id TEXT REFERENCES wymiana_metryka_strony (id),
  postac TEXT NOT NULL CHECK (postac IN ('oryginal', 'odpis', 'skan')),
  -- Ile dni przed aktem dokument może najwyżej być wystawiony, gdy kancelaria tego wymaga.
  swiezosc_dni INTEGER CHECK (swiezosc_dni > 0),
  -- Dzień, do którego kancelaria chce dostać dokument.
  do_kiedy TEXT,

  PRIMARY KEY (wpis_id, pozycja_id)
) STRICT;

-- Dokument przekazany w pokoju: dokument z listy, pełnomocnictwo albo zgoda strony od biura,
-- projekt aktu albo koszty od kancelarii. Plik ma wpis.
CREATE TABLE wymiana_dokumenty_wpisy (
  wpis_id TEXT PRIMARY KEY REFERENCES wymiana_wpisy_wpisy (id),
  -- Taki sam jak rodzaj wpisu; pozwala nazwać wymagane kolumny każdego rodzaju.
  rodzaj TEXT NOT NULL CHECK (
    rodzaj IN ('dokument', 'pelnomocnictwo', 'zgoda_stron', 'projekt_aktu', 'koszty')
  ),
  -- Pozycja listy kancelarii, którą dokument wypełnia.
  pozycja_id TEXT CHECK (pozycja_id <> ''),
  -- Strona, w której imieniu biuro przekazuje dokument, której pełnomocnictwo albo zgodę.
  strona_id TEXT REFERENCES wymiana_metryka_strony (id),
  wystawca TEXT CHECK (wystawca <> ''),
  data_wystawienia TEXT,
  waznosc_do TEXT,
  -- Czy oryginał będzie w dniu aktu i kto go przyniesie.
  oryginal_w_dniu_aktu INTEGER CHECK (oryginal_w_dniu_aktu IN (0, 1)),
  kto_przyniesie TEXT CHECK (kto_przyniesie <> ''),
  -- Zakres pełnomocnictwa albo zgody, np. „uzgadnianie terminu aktu i przekazywanie dokumentów
  -- notariuszowi” albo „biuro dostaje projekt aktu i informację po akcie”.
  zakres TEXT CHECK (zakres <> ''),
  -- Wersja projektu aktu: kolejne wersje stoją obok siebie z rosnącym numerem.
  numer_wersji INTEGER CHECK (numer_wersji > 0),
  -- Kancelaria wskazuje, że to wersja, którą strony mają podpisać.
  ostateczna INTEGER CHECK (ostateczna IN (0, 1)),

  CONSTRAINT dokument_wypelnia_pozycje CHECK ((rodzaj = 'dokument') = (pozycja_id IS NOT NULL)),
  CONSTRAINT dokument_ma_wystawce_i_oryginal CHECK (
    rodzaj <> 'dokument'
    OR (wystawca IS NOT NULL AND data_wystawienia IS NOT NULL AND oryginal_w_dniu_aktu IS NOT NULL)
  ),
  CONSTRAINT oryginal_ma_kto_przyniesie CHECK ((oryginal_w_dniu_aktu IS 1) = (kto_przyniesie IS NOT NULL)),
  CONSTRAINT dokument_biura_wskazuje_strone CHECK (
    (rodzaj IN ('dokument', 'pelnomocnictwo', 'zgoda_stron')) = (strona_id IS NOT NULL)
  ),
  CONSTRAINT pelnomocnictwo_i_zgoda_z_zakresem CHECK (
    (rodzaj IN ('pelnomocnictwo', 'zgoda_stron')) = (zakres IS NOT NULL)
  ),
  CONSTRAINT projekt_ma_wersje CHECK (
    (rodzaj = 'projekt_aktu') = (numer_wersji IS NOT NULL)
    AND (rodzaj = 'projekt_aktu') = (ostateczna IS NOT NULL)
  )
) STRICT;

-- Uwagi stron do wersji projektu, które biuro przekazuje za zgodą człowieka. Decyzja leży
-- w teczce transakcji (decyzja wyslanie), więc wpis wskazuje ją tekstem.
CREATE TABLE wymiana_dokumenty_uwagi (
  wpis_id TEXT PRIMARY KEY REFERENCES wymiana_wpisy_wpisy (id),
  projekt_id TEXT NOT NULL REFERENCES wymiana_dokumenty_wpisy (wpis_id),
  -- Decyzja z teczki transakcji, która pozwoliła przekazać uwagi: identyfikator pokoju i decyzji.
  decyzja_odnosnik TEXT NOT NULL CHECK (decyzja_odnosnik <> '')
) STRICT;

-- Szczegół dopisuje się w tej samej operacji co jego wpis. Późniejszy dopisek zmieniałby wpis,
-- który druga strona już przeczytała, także w zamkniętym pokoju.
CREATE TRIGGER wymiana_dokumenty_lista_do_swojego_wpisu
BEFORE INSERT ON wymiana_dokumenty_lista
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
    AND rodzaj = 'lista_dokumentow'
    AND rowid = (
      SELECT max(rowid)
      FROM wymiana_wpisy_wpisy
    )
)
BEGIN
  SELECT RAISE(ABORT, 'pozycja listy dokumentów należy do najnowszego wpisu lista_dokumentow');
END;

CREATE TRIGGER wymiana_dokumenty_wpisy_do_swojego_wpisu
BEFORE INSERT ON wymiana_dokumenty_wpisy
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
    AND rodzaj = NEW.rodzaj
    AND rowid = (
      SELECT max(rowid)
      FROM wymiana_wpisy_wpisy
    )
)
BEGIN
  SELECT RAISE(ABORT, 'dokument należy do najnowszego wpisu tego samego rodzaju');
END;

CREATE TRIGGER wymiana_dokumenty_uwagi_do_swojego_wpisu
BEFORE INSERT ON wymiana_dokumenty_uwagi
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
    AND rodzaj = 'uwagi_do_projektu'
    AND rowid = (
      SELECT max(rowid)
      FROM wymiana_wpisy_wpisy
    )
)
BEGIN
  SELECT RAISE(ABORT, 'uwagi należą do najnowszego wpisu uwagi_do_projektu');
END;

-- Nowa wersja projektu nie zastępuje starej, tylko staje obok niej z wyższym numerem.
CREATE TRIGGER wymiana_dokumenty_wersja_projektu_rosnie
BEFORE INSERT ON wymiana_dokumenty_wpisy
WHEN NEW.rodzaj = 'projekt_aktu'
  AND NEW.numer_wersji <= (
    SELECT coalesce(max(numer_wersji), 0)
    FROM wymiana_dokumenty_wpisy
    WHERE rodzaj = 'projekt_aktu'
  )
BEGIN
  SELECT RAISE(ABORT, 'numer wersji projektu aktu rośnie');
END;

-- Pełnomocnictwo daje biuru strona, którą biuro reprezentuje według metryki pokoju.
CREATE TRIGGER wymiana_dokumenty_pelnomocnictwo_strony_biura
BEFORE INSERT ON wymiana_dokumenty_wpisy
WHEN NEW.rodzaj = 'pelnomocnictwo'
  AND NOT EXISTS (
    SELECT 1
    FROM wymiana_metryka_strony
    WHERE id = NEW.strona_id
      AND reprezentowana_przez_biuro = 1
  )
BEGIN
  SELECT RAISE(ABORT, 'pełnomocnictwo dotyczy strony, którą biuro reprezentuje według metryki');
END;

-- Pisanie bez pełnomocnictwa: dokument i zgodę strony biuro przekazuje dopiero po jej
-- pełnomocnictwie. Druga strona, która nie jest klientem biura, przekazuje swoje sama.
CREATE TRIGGER wymiana_dokumenty_wymagaja_pelnomocnictwa
BEFORE INSERT ON wymiana_dokumenty_wpisy
WHEN NEW.rodzaj IN ('dokument', 'zgoda_stron')
  AND NOT EXISTS (
    SELECT 1
    FROM wymiana_dokumenty_wpisy
    WHERE rodzaj = 'pelnomocnictwo'
      AND strona_id = NEW.strona_id
  )
BEGIN
  SELECT RAISE(ABORT, 'dokument biura dla strony wymaga wpisanego wcześniej pełnomocnictwa tej strony');
END;

CREATE TRIGGER wymiana_dokumenty_dokument_z_listy
BEFORE INSERT ON wymiana_dokumenty_wpisy
WHEN NEW.rodzaj = 'dokument'
  AND NOT EXISTS (
    SELECT 1
    FROM wymiana_dokumenty_lista
    WHERE pozycja_id = NEW.pozycja_id
  )
BEGIN
  SELECT RAISE(ABORT, 'dokument wypełnia pozycję z listy dokumentów kancelarii');
END;

-- Uwaga wskazuje wersję projektu, a nie inny dokument, żeby było widać, której wersji dotyczy.
CREATE TRIGGER wymiana_dokumenty_uwagi_do_projektu
BEFORE INSERT ON wymiana_dokumenty_uwagi
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_dokumenty_wpisy
  WHERE wpis_id = NEW.projekt_id
    AND rodzaj = 'projekt_aktu'
)
BEGIN
  SELECT RAISE(ABORT, 'uwagi wskazują wersję projektu aktu');
END;

CREATE TRIGGER wymiana_dokumenty_lista_bez_zmian
BEFORE UPDATE ON wymiana_dokumenty_lista
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

CREATE TRIGGER wymiana_dokumenty_wpisy_bez_zmian
BEFORE UPDATE ON wymiana_dokumenty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

CREATE TRIGGER wymiana_dokumenty_uwagi_bez_zmian
BEFORE UPDATE ON wymiana_dokumenty_uwagi
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_dokumenty_lista_bez_zastepowania
BEFORE INSERT ON wymiana_dokumenty_lista
WHEN EXISTS (
  SELECT 1
  FROM wymiana_dokumenty_lista
  WHERE wpis_id = NEW.wpis_id
    AND pozycja_id = NEW.pozycja_id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_dokumenty_lista_bez_usuwania
BEFORE DELETE ON wymiana_dokumenty_lista
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_dokumenty_wpisy_bez_zastepowania
BEFORE INSERT ON wymiana_dokumenty_wpisy
WHEN EXISTS (
  SELECT 1
  FROM wymiana_dokumenty_wpisy
  WHERE wpis_id = NEW.wpis_id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_dokumenty_wpisy_bez_usuwania
BEFORE DELETE ON wymiana_dokumenty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_dokumenty_uwagi_bez_zastepowania
BEFORE INSERT ON wymiana_dokumenty_uwagi
WHEN EXISTS (
  SELECT 1
  FROM wymiana_dokumenty_uwagi
  WHERE wpis_id = NEW.wpis_id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_dokumenty_uwagi_bez_usuwania
BEFORE DELETE ON wymiana_dokumenty_uwagi
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;
