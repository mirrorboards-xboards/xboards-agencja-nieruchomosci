-- Ceny oferty: propozycja biura z porównanymi transakcjami, cena ofertowa zatwierdzona przez
-- klienta z historią zmian i granice negocjacji, osobno dla sprzedaży i dla najmu (dotyczy).
-- Przy najmie kwota to miesięczny czynsz. Kwoty w groszach. Pisze Opiekun oferty.
--
-- Cena oferty to propozycja biura, a nie operat szacunkowy. Cenę ofertową i granice wyznacza
-- klient: każdy wpis wskazuje jego zgodę, która dotyczy tego wpisu. Żadnego wpisu się nie zmienia
-- ani nie usuwa, więc widać, czy cena nie przesuwa się po trochu bez decyzji klienta. Granice
-- negocjacji nie wychodzą poza teczkę oferty (R1); pilnuje tego aplikacja.

-- Propozycja ceny albo czynszu z uzasadnieniem i plikiem wyliczenia z programu (np. średniej ceny
-- za metr z porównanych transakcji).
CREATE TABLE oferta_ceny_propozycje (
  id TEXT PRIMARY KEY,
  dotyczy TEXT NOT NULL CHECK (dotyczy IN ('sprzedaz', 'najem')),
  kwota INTEGER NOT NULL CHECK (kwota > 0),
  uzasadnienie TEXT NOT NULL,
  utworzono TEXT NOT NULL,
  wyliczenie_sciezka TEXT NOT NULL,
  wyliczenie_skrot TEXT NOT NULL
) STRICT;

-- Transakcje z rejestru cen nieruchomości (art. 181a pkt 6 ugn), z którymi Opiekun porównał
-- nieruchomość; cenę za metr liczy program.
CREATE TABLE oferta_ceny_porownania (
  -- Klucz nadany przez bazę jest dodatni; zob. oferta_ceny_porownania_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  propozycja_id TEXT NOT NULL REFERENCES oferta_ceny_propozycje (id),
  data_transakcji TEXT NOT NULL,
  cena INTEGER NOT NULL CHECK (cena > 0),
  -- Powierzchnia w setnych metra kwadratowego, jak w rdzeniu.
  powierzchnia INTEGER CHECK (powierzchnia > 0),
  -- Różnice wobec oferty, np. piętro albo stan.
  roznice TEXT,
  zrodlo TEXT NOT NULL
) STRICT;

-- Cena ofertowa, którą klient zatwierdził; każda zmiana to nowy wpis, który wskazuje poprzednią
-- cenę i ma powód. Ta cena trafia do ogłoszenia (w_ogloszeniach_od).
CREATE TABLE oferta_ceny_ofertowe (
  id TEXT PRIMARY KEY,
  dotyczy TEXT NOT NULL CHECK (dotyczy IN ('sprzedaz', 'najem')),
  kwota INTEGER NOT NULL CHECK (kwota > 0),
  zgoda_id TEXT NOT NULL REFERENCES sprawa_zgody_wpisy (id),
  obowiazuje_od TEXT NOT NULL,
  poprzednia_id TEXT REFERENCES oferta_ceny_ofertowe (id),
  powod TEXT,
  -- Dzień (bez godziny), w którym nowa cena weszła do ogłoszeń; wpisuje się go raz, po zmianie
  -- ogłoszeń.
  w_ogloszeniach_od TEXT,

  CONSTRAINT zmiana_ceny_ma_powod CHECK ((poprzednia_id IS NULL) = (powod IS NULL)),
  CONSTRAINT w_ogloszeniach_nie_przed_cena CHECK (
    w_ogloszeniach_od IS NULL OR w_ogloszeniach_od >= substr(obowiazuje_od, 1, 10)
  )
) STRICT;

-- Granice negocjacji: najniższa cena albo czynsz, które klient przyjmie, i warunki, od których to
-- zależy. Nowe granice zastępują poprzednie.
CREATE TABLE oferta_ceny_granice (
  id TEXT PRIMARY KEY,
  dotyczy TEXT NOT NULL CHECK (dotyczy IN ('sprzedaz', 'najem')),
  najnizsza_kwota INTEGER NOT NULL CHECK (najnizsza_kwota > 0),
  warunki TEXT,
  zgoda_id TEXT NOT NULL REFERENCES sprawa_zgody_wpisy (id),
  zastepuje_id TEXT REFERENCES oferta_ceny_granice (id),

  CONSTRAINT nie_zastepuje_samych_siebie CHECK (zastepuje_id IS NOT id)
) STRICT;

-- Cena ofertowa, która obowiązuje: żadna nowsza cena jej nie wskazuje.
CREATE VIEW oferta_ceny_biezace AS
SELECT *
FROM oferta_ceny_ofertowe AS cena
WHERE NOT EXISTS (
  SELECT 1
  FROM oferta_ceny_ofertowe AS nowsza
  WHERE nowsza.poprzednia_id = cena.id
);

-- Cenę ofertową zatwierdza klient: zgoda zatwierdzenie_ceny uczestnika z obowiązującą rolą
-- klient, która dotyczy tej ceny.
CREATE TRIGGER oferta_ceny_ofertowa_ze_zgoda_klienta
BEFORE INSERT ON oferta_ceny_ofertowe
WHEN NOT EXISTS (
  SELECT 1
  FROM sprawa_zgody_wpisy AS zgoda
  JOIN sprawa_uczestnicy_role AS rola ON rola.uczestnik_id = zgoda.uczestnik_id
  WHERE zgoda.id = NEW.zgoda_id
    AND zgoda.rodzaj = 'zatwierdzenie_ceny'
    AND zgoda.dotyczy_tabela = 'oferta_ceny_ofertowe'
    AND zgoda.dotyczy_id = NEW.id
    AND rola.rola = 'klient'
    AND rola.zakonczono IS NULL
)
BEGIN
  SELECT RAISE(ABORT, 'cena ofertowa ma zgodę zatwierdzenie_ceny klienta, która jej dotyczy');
END;

-- Pierwsza cena (sprzedaży albo najmu) nie ma poprzedniej; każda następna wskazuje cenę, która
-- obowiązuje, i obowiązuje od chwili późniejszej niż ona.
CREATE TRIGGER oferta_ceny_zmiana_wskazuje_obowiazujaca
BEFORE INSERT ON oferta_ceny_ofertowe
WHEN (
    NEW.poprzednia_id IS NULL
    AND EXISTS (
      SELECT 1
      FROM oferta_ceny_ofertowe
      WHERE dotyczy = NEW.dotyczy
    )
  )
  OR (
    NEW.poprzednia_id IS NOT NULL
    AND NOT EXISTS (
      SELECT 1
      FROM oferta_ceny_biezace
      WHERE id = NEW.poprzednia_id
        AND dotyczy = NEW.dotyczy
        AND obowiazuje_od < NEW.obowiazuje_od
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'zmiana ceny wskazuje obowiązującą cenę tego samego rodzaju i obowiązuje od chwili późniejszej niż ona');
END;

-- Cena powstaje bez daty wejścia do ogłoszeń, a potem dostaje ją raz; poza tym jej się nie
-- zmienia.
CREATE TRIGGER oferta_ceny_ofertowa_powstaje_przed_ogloszeniami
BEFORE INSERT ON oferta_ceny_ofertowe
WHEN NEW.w_ogloszeniach_od IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'cena ofertowa powstaje bez daty wejścia do ogłoszeń');
END;

CREATE TRIGGER oferta_ceny_ofertowe_bez_zmian
BEFORE UPDATE ON oferta_ceny_ofertowe
WHEN OLD.w_ogloszeniach_od IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.dotyczy IS NOT OLD.dotyczy
  OR NEW.kwota IS NOT OLD.kwota
  OR NEW.zgoda_id IS NOT OLD.zgoda_id
  OR NEW.obowiazuje_od IS NOT OLD.obowiazuje_od
  OR NEW.poprzednia_id IS NOT OLD.poprzednia_id
  OR NEW.powod IS NOT OLD.powod
BEGIN
  SELECT RAISE(ABORT, 'ceny ofertowej się nie zmienia; raz wpisuje się tylko dzień wejścia do ogłoszeń');
END;

-- Granice wyznacza klient: zgoda granice_negocjacji uczestnika z obowiązującą rolą klient,
-- która dotyczy tych granic.
CREATE TRIGGER oferta_ceny_granice_ze_zgoda_klienta
BEFORE INSERT ON oferta_ceny_granice
WHEN NOT EXISTS (
  SELECT 1
  FROM sprawa_zgody_wpisy AS zgoda
  JOIN sprawa_uczestnicy_role AS rola ON rola.uczestnik_id = zgoda.uczestnik_id
  WHERE zgoda.id = NEW.zgoda_id
    AND zgoda.rodzaj = 'granice_negocjacji'
    AND zgoda.dotyczy_tabela = 'oferta_ceny_granice'
    AND zgoda.dotyczy_id = NEW.id
    AND rola.rola = 'klient'
    AND rola.zakonczono IS NULL
)
BEGIN
  SELECT RAISE(ABORT, 'granice negocjacji mają zgodę granice_negocjacji klienta, która ich dotyczy');
END;

-- Pierwsze granice nie zastępują żadnych; następne zastępują granice tego samego rodzaju, które
-- obowiązują.
CREATE TRIGGER oferta_ceny_granice_zastepuja_obowiazujace
BEFORE INSERT ON oferta_ceny_granice
WHEN (
    NEW.zastepuje_id IS NULL
    AND EXISTS (
      SELECT 1
      FROM oferta_ceny_granice
      WHERE dotyczy = NEW.dotyczy
    )
  )
  OR (
    NEW.zastepuje_id IS NOT NULL
    AND NOT EXISTS (
      SELECT 1
      FROM oferta_ceny_granice AS poprzednie
      WHERE poprzednie.id = NEW.zastepuje_id
        AND poprzednie.dotyczy = NEW.dotyczy
        AND NOT EXISTS (
          SELECT 1
          FROM oferta_ceny_granice AS nowsze
          WHERE nowsze.zastepuje_id = poprzednie.id
        )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'nowe granice negocjacji zastępują obowiązujące granice tego samego rodzaju');
END;

CREATE TRIGGER oferta_ceny_propozycje_bez_zmian
BEFORE UPDATE ON oferta_ceny_propozycje
BEGIN
  SELECT RAISE(ABORT, 'propozycji ceny się nie zmienia; nowa propozycja to nowy wpis');
END;

CREATE TRIGGER oferta_ceny_porownania_bez_zmian
BEFORE UPDATE ON oferta_ceny_porownania
BEGIN
  SELECT RAISE(ABORT, 'porównania transakcji się nie zmienia; nowe porównanie to nowy wpis');
END;

CREATE TRIGGER oferta_ceny_granice_bez_zmian
BEFORE UPDATE ON oferta_ceny_granice
BEGIN
  SELECT RAISE(ABORT, 'granic negocjacji się nie zmienia; nowe granice zastępują poprzednie');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER oferta_ceny_propozycje_bez_zastepowania
BEFORE INSERT ON oferta_ceny_propozycje
WHEN EXISTS (
  SELECT 1
  FROM oferta_ceny_propozycje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_ceny_propozycje_bez_usuwania
BEFORE DELETE ON oferta_ceny_propozycje
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER oferta_ceny_porownania_bez_zastepowania
BEFORE INSERT ON oferta_ceny_porownania
WHEN EXISTS (
  SELECT 1
  FROM oferta_ceny_porownania
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_ceny_porownania_bez_usuwania
BEFORE DELETE ON oferta_ceny_porownania
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER oferta_ceny_ofertowe_bez_zastepowania
BEFORE INSERT ON oferta_ceny_ofertowe
WHEN EXISTS (
  SELECT 1
  FROM oferta_ceny_ofertowe
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_ceny_ofertowe_bez_usuwania
BEFORE DELETE ON oferta_ceny_ofertowe
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER oferta_ceny_granice_bez_zastepowania
BEFORE INSERT ON oferta_ceny_granice
WHEN EXISTS (
  SELECT 1
  FROM oferta_ceny_granice
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_ceny_granice_bez_usuwania
BEFORE DELETE ON oferta_ceny_granice
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
