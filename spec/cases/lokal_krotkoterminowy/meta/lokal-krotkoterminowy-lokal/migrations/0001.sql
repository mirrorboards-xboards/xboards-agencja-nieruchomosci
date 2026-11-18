-- Dane lokalu ponad oznaczenie nieruchomości z rdzenia: rejestracja, łóżka, czas sprzątania,
-- standard i instrukcja wejścia, a do tego regulaminy lokalu, wyposażenie biura, przeglądy
-- i uchwały wspólnoty. Spis wyposażenia przy przejęciu i zwrocie trzyma rdzeń (sprawa-protokoly).
-- Pisze Zarządca obiektu; wyposażenie i przeglądy także Koordynator operacji.
--
-- Kod do zamka nie jest częścią instrukcji wejścia: ustawia się go dla każdego pobytu i zlecenia
-- (lokal-krotkoterminowy-zlecenia).
CREATE TABLE lokal_krotkoterminowy_lokal_dane (
  -- Jeden wiersz na teczkę, bo teczka prowadzi jeden lokal.
  id INTEGER PRIMARY KEY CHECK (id = 1),
  nieruchomosc_id TEXT NOT NULL REFERENCES sprawa_nieruchomosci_wpisy (id),
  -- Rejestracja lokalu, gdy biuro ustali, że jest wymagana (sprawa otwarta w „Agencji nieruchomości”).
  rejestracja_numer TEXT,
  rejestracja_data TEXT,
  lozka INTEGER NOT NULL CHECK (lozka > 0),
  -- Czas sprzątania ze standardu lokalu w minutach; z nim porównuje się okno między pobytami.
  czas_sprzatania_min INTEGER CHECK (czas_sprzatania_min > 0),
  -- Liczba kompletów kluczy, gdy lokal je ma.
  klucze_liczba INTEGER CHECK (klucze_liczba >= 0),
  standard_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  instrukcja_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),

  CONSTRAINT rejestracja_numer_z_data CHECK ((rejestracja_numer IS NULL) = (rejestracja_data IS NULL))
) STRICT;

-- Regulaminy lokalu w kolejności obowiązywania. Bieżący to ostatni.
CREATE TABLE lokal_krotkoterminowy_lokal_regulaminy (
  id TEXT PRIMARY KEY,
  dokument_id TEXT NOT NULL REFERENCES sprawa_dokumenty_wpisy (id),
  od TEXT NOT NULL,
  -- Zgoda człowieka na zmianę regulaminu; pierwszy regulamin jej nie wymaga.
  decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id)
) STRICT;

-- Wyposażenie, które biuro kupiło albo wymieniło, z kosztem i datą; wycofana rzecz dostaje datę
-- końca i powód.
CREATE TABLE lokal_krotkoterminowy_lokal_wyposazenie (
  -- Klucz nadany przez bazę jest dodatni; zob. lokal_krotkoterminowy_lokal_wyposazenie_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  rzecz TEXT NOT NULL CHECK (rzecz <> ''),
  -- Dzień zakupu albo wstawienia.
  od TEXT,
  -- Koszt zakupu w groszach, gdy kupiło biuro na koszt właściciela.
  koszt INTEGER CHECK (koszt >= 0),
  -- Zgoda właściciela na uzupełnienie wyposażenia, gdy jej wymagało.
  zgoda_id TEXT REFERENCES sprawa_zgody_wpisy (id),
  do_dnia TEXT,
  powod TEXT,

  CONSTRAINT koszt_z_data CHECK (koszt IS NULL OR od IS NOT NULL),
  CONSTRAINT wycofanie_z_powodem CHECK ((do_dnia IS NULL) = (powod IS NULL)),
  CONSTRAINT wycofanie_nie_przed_zakupem CHECK (do_dnia IS NULL OR od IS NULL OR do_dnia >= od)
) STRICT;

-- Przeglądy instalacji, czujników i gaśnicy z datą następnego. Które są obowiązkowe, mówi
-- standard lokalu.
CREATE TABLE lokal_krotkoterminowy_lokal_przeglady (
  -- Klucz nadany przez bazę jest dodatni; zob. lokal_krotkoterminowy_lokal_przeglady_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Np. instalacja_elektryczna, instalacja_gazowa, czujnik_dymu, czujnik_co, gasnica.
  rodzaj TEXT NOT NULL CHECK (rodzaj <> '' AND rodzaj NOT GLOB '*[^a-z_]*'),
  data TEXT NOT NULL CHECK (data = date(data)),
  nastepny TEXT CHECK (nastepny = date(nastepny)),
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),

  CONSTRAINT nastepny_po_przegladzie CHECK (nastepny IS NULL OR nastepny > data)
) STRICT;

-- Uchwały wspólnoty albo spółdzielni o najmie krótkoterminowym, z dniem podjęcia i dniem,
-- w którym biuro się o nich dowiedziało (Zarządca zgłasza je w raporcie tego dnia).
CREATE TABLE lokal_krotkoterminowy_lokal_uchwaly (
  -- Klucz nadany przez bazę jest dodatni; zob. lokal_krotkoterminowy_lokal_uchwaly_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  data TEXT NOT NULL CHECK (data = date(data)),
  tresc TEXT NOT NULL CHECK (tresc <> ''),
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  dowiedziano TEXT NOT NULL,

  CONSTRAINT dowiedziano_po_podjeciu CHECK (dowiedziano >= data)
) STRICT;

-- Zmiana regulaminu wymaga zgody człowieka („Sprawy za zgodą”): każdy regulamin po pierwszym
-- wskazuje zatwierdzoną decyzję, która go wskazuje.
CREATE TRIGGER lokal_krotkoterminowy_lokal_zmiana_regulaminu_za_zgoda
BEFORE INSERT ON lokal_krotkoterminowy_lokal_regulaminy
WHEN EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_lokal_regulaminy
  )
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone
    WHERE id = NEW.decyzja_id
      AND rodzaj = 'inna'
      AND dotyczy_tabela = 'lokal_krotkoterminowy_lokal_regulaminy'
      AND dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'każdy regulamin po pierwszym wskazuje zatwierdzoną decyzję inna, która go wskazuje');
END;

-- Regulamin obowiązywał gości od swojej chwili; nowy regulamin to nowy wiersz.
CREATE TRIGGER lokal_krotkoterminowy_lokal_regulaminy_bez_zmian
BEFORE UPDATE ON lokal_krotkoterminowy_lokal_regulaminy
BEGIN
  SELECT RAISE(ABORT, 'regulaminu się nie zmienia; nowy regulamin to nowy wiersz');
END;

CREATE TRIGGER lokal_krotkoterminowy_lokal_zgoda_na_wyposazenie
BEFORE INSERT ON lokal_krotkoterminowy_lokal_wyposazenie
WHEN NEW.zgoda_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_zgody_wpisy
    WHERE id = NEW.zgoda_id
      AND rodzaj = 'zgoda_na_wyposazenie'
  )
BEGIN
  SELECT RAISE(ABORT, 'wyposażenie wskazuje zgodę zgoda_na_wyposazenie');
END;

-- Koszt wyposażenia trafia do zestawienia dla właściciela, więc rzecz, koszt i data się nie
-- zmieniają. Wycofanie wpisuje się raz.
CREATE TRIGGER lokal_krotkoterminowy_lokal_wyposazenie_zmienia_tylko_wycofanie
BEFORE UPDATE ON lokal_krotkoterminowy_lokal_wyposazenie
WHEN OLD.do_dnia IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.rzecz IS NOT OLD.rzecz
  OR NEW.od IS NOT OLD.od
  OR NEW.koszt IS NOT OLD.koszt
  OR NEW.zgoda_id IS NOT OLD.zgoda_id
BEGIN
  SELECT RAISE(ABORT, 'wyposażenia się nie zmienia; wpisuje się tylko jego wycofanie, raz');
END;

CREATE TRIGGER lokal_krotkoterminowy_lokal_przeglady_bez_zmian
BEFORE UPDATE ON lokal_krotkoterminowy_lokal_przeglady
BEGIN
  SELECT RAISE(ABORT, 'przeglądu się nie zmienia; kolejny przegląd to nowy wiersz');
END;

CREATE TRIGGER lokal_krotkoterminowy_lokal_uchwaly_bez_zmian
BEFORE UPDATE ON lokal_krotkoterminowy_lokal_uchwaly
BEGIN
  SELECT RAISE(ABORT, 'uchwały się nie zmienia; nowa uchwała to nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER lokal_krotkoterminowy_lokal_dane_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_lokal_dane
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_lokal_dane
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_lokal_dane_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_lokal_dane
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER lokal_krotkoterminowy_lokal_regulaminy_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_lokal_regulaminy
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_lokal_regulaminy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_lokal_regulaminy_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_lokal_regulaminy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER lokal_krotkoterminowy_lokal_wyposazenie_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_lokal_wyposazenie
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_lokal_wyposazenie
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_lokal_wyposazenie_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_lokal_wyposazenie
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER lokal_krotkoterminowy_lokal_przeglady_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_lokal_przeglady
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_lokal_przeglady
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_lokal_przeglady_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_lokal_przeglady
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER lokal_krotkoterminowy_lokal_uchwaly_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_lokal_uchwaly
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_lokal_uchwaly
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_lokal_uchwaly_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_lokal_uchwaly
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
