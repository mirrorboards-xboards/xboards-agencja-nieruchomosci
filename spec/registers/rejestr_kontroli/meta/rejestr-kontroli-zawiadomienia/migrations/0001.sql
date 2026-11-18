-- Ślad zawiadomień, żądania organów i kontrole. Projekt zawiadomienia leży w pamięci Kierownika
-- kontroli, dopóki człowiek go nie przekaże; do rejestru trafia ślad w dniu przekazania. Żaden
-- agent nie przekazuje niczego organowi; robi to osoba wyznaczona do obowiązków z ustawy.
--
-- Zawiadomienie wskazuje dwie zatwierdzone decyzje: potwierdzenie podejrzenia i przekazanie, a jego
-- przekazanie nie jest wcześniejsze niż potwierdzenie. Termin 2 dni roboczych (art. 74 ust. 2 AML)
-- liczy program. Wierszy się nie zmienia; to, co przychodzi od organu później, wpisuje się raz.
CREATE TABLE rejestr_kontroli_zawiadomienia_wpisy (
  id TEXT PRIMARY KEY,
  -- Art. 74, 86 i 90 AML do Generalnego Inspektora, art. 89 AML do prokuratora.
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('art74', 'art86', 'art90', 'art89_prokurator')),
  potwierdzenie_decyzja_id TEXT NOT NULL REFERENCES rejestr_kontroli_decyzje_wpisy (id),
  przekazanie_decyzja_id TEXT NOT NULL REFERENCES rejestr_kontroli_decyzje_wpisy (id),
  -- Osoba wyznaczona, która przekazała zawiadomienie (art. 8 AML), i chwila przekazania.
  przekazal TEXT NOT NULL CHECK (przekazal <> ''),
  przekazano TEXT NOT NULL,
  -- Odnośniki do wpisów rejestru, na których zawiadomienie stoi.
  podstawy TEXT NOT NULL CHECK (podstawy <> ''),
  -- Urzędowe poświadczenie odbioru (art. 86 ust. 3 AML) i odpowiedź organu, np. żądanie
  -- wstrzymania transakcji (art. 86 ust. 5 AML); każde wpisuje się raz, gdy przyjdzie.
  potwierdzenie_przyjecia TEXT,
  odpowiedz_organu TEXT,
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL
) STRICT;

-- Żądania organów i to, co biuro przekazało.
CREATE TABLE rejestr_kontroli_zawiadomienia_zadania (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_zawiadomienia_zadania_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Np. Generalny Inspektor Informacji Finansowej (art. 76 ust. 1 AML).
  organ TEXT NOT NULL CHECK (organ <> ''),
  zadano TEXT NOT NULL,
  tresc TEXT NOT NULL CHECK (tresc <> ''),
  -- Właściciel Kierownika kontroli, który odebrał żądanie.
  odebral TEXT NOT NULL CHECK (odebral <> ''),
  -- Kiedy i co biuro przekazało; wpisuje się raz.
  przekazano TEXT,
  co_przekazano TEXT,
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,

  CONSTRAINT przekazanie_w_komplecie CHECK ((przekazano IS NULL) = (co_przekazano IS NULL)),
  CONSTRAINT przekazanie_po_zadaniu CHECK (przekazano IS NULL OR przekazano >= zadano)
) STRICT;

-- Kontrole wykonywania obowiązków z protokołem i zaleceniami, wpisane po protokole.
CREATE TABLE rejestr_kontroli_zawiadomienia_kontrole (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_zawiadomienia_kontrole_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Kto kontrolował wykonywanie obowiązków (art. 130 ust. 1 AML).
  organ TEXT NOT NULL CHECK (organ <> ''),
  od TEXT NOT NULL,
  do TEXT NOT NULL,
  protokol_sciezka TEXT NOT NULL CHECK (protokol_sciezka <> ''),
  protokol_skrot TEXT NOT NULL CHECK (protokol_skrot <> ''),
  zalecenia TEXT,
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,

  CONSTRAINT koniec_nie_przed_poczatkiem CHECK (do >= od)
) STRICT;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_wpisy_po_potwierdzeniu_podejrzenia
BEFORE INSERT ON rejestr_kontroli_zawiadomienia_wpisy
WHEN NOT EXISTS (
  SELECT 1
  FROM rejestr_kontroli_decyzje_zatwierdzone
  WHERE id = NEW.potwierdzenie_decyzja_id
    AND rodzaj = 'potwierdzenie_podejrzenia'
)
BEGIN
  SELECT RAISE(ABORT, 'zawiadomienie wskazuje zatwierdzoną decyzję potwierdzenie_podejrzenia');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_wpisy_po_decyzji_o_przekazaniu
BEFORE INSERT ON rejestr_kontroli_zawiadomienia_wpisy
WHEN NOT EXISTS (
  SELECT 1
  FROM rejestr_kontroli_decyzje_zatwierdzone
  WHERE id = NEW.przekazanie_decyzja_id
    AND rodzaj = 'przekazanie_zawiadomienia'
)
BEGIN
  SELECT RAISE(ABORT, 'zawiadomienie wskazuje zatwierdzoną decyzję przekazanie_zawiadomienia');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_wpisy_przekazanie_po_potwierdzeniu
BEFORE INSERT ON rejestr_kontroli_zawiadomienia_wpisy
WHEN NEW.przekazano < (
  SELECT podpisano
  FROM rejestr_kontroli_decyzje_wpisy
  WHERE id = NEW.potwierdzenie_decyzja_id
)
BEGIN
  SELECT RAISE(ABORT, 'przekazanie zawiadomienia nie jest wcześniejsze niż potwierdzenie podejrzenia');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_wpisy_odpowiedzi_przychodza_pozniej
BEFORE INSERT ON rejestr_kontroli_zawiadomienia_wpisy
WHEN NEW.potwierdzenie_przyjecia IS NOT NULL
  OR NEW.odpowiedz_organu IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'zawiadomienie powstaje bez potwierdzenia przyjęcia i odpowiedzi organu; wpisuje się je, gdy przyjdą');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_wpisy_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_zawiadomienia_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.potwierdzenie_decyzja_id IS NOT OLD.potwierdzenie_decyzja_id
  OR NEW.przekazanie_decyzja_id IS NOT OLD.przekazanie_decyzja_id
  OR NEW.przekazal IS NOT OLD.przekazal
  OR NEW.przekazano IS NOT OLD.przekazano
  OR NEW.podstawy IS NOT OLD.podstawy
  OR NEW.wpisal IS NOT OLD.wpisal
  OR NEW.wpisano IS NOT OLD.wpisano
  OR (OLD.potwierdzenie_przyjecia IS NOT NULL AND NEW.potwierdzenie_przyjecia IS NOT OLD.potwierdzenie_przyjecia)
  OR (OLD.odpowiedz_organu IS NOT NULL AND NEW.odpowiedz_organu IS NOT OLD.odpowiedz_organu)
BEGIN
  SELECT RAISE(ABORT, 'śladu zawiadomienia się nie zmienia; potwierdzenie przyjęcia i odpowiedź organu wpisuje się raz');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_zadania_powstaje_bez_przekazania
BEFORE INSERT ON rejestr_kontroli_zawiadomienia_zadania
WHEN NEW.przekazano IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'żądanie organu powstaje bez przekazania; to, co biuro przekazało, wpisuje się raz');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_zadania_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_zawiadomienia_zadania
WHEN NEW.id IS NOT OLD.id
  OR NEW.organ IS NOT OLD.organ
  OR NEW.zadano IS NOT OLD.zadano
  OR NEW.tresc IS NOT OLD.tresc
  OR NEW.odebral IS NOT OLD.odebral
  OR NEW.wpisal IS NOT OLD.wpisal
  OR NEW.wpisano IS NOT OLD.wpisano
  OR (OLD.przekazano IS NOT NULL AND (NEW.przekazano IS NOT OLD.przekazano OR NEW.co_przekazano IS NOT OLD.co_przekazano))
BEGIN
  SELECT RAISE(ABORT, 'żądania organu się nie zmienia; to, co biuro przekazało, wpisuje się raz');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_kontrole_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_zawiadomienia_kontrole
BEGIN
  SELECT RAISE(ABORT, 'wpisu kontroli się nie zmienia; błąd poprawia nowy wpis');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabel z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER rejestr_kontroli_zawiadomienia_wpisy_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_zawiadomienia_wpisy
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_zawiadomienia_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_zadania_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_zawiadomienia_zadania
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_zawiadomienia_zadania
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_kontrole_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_zawiadomienia_kontrole
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_zawiadomienia_kontrole
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

-- Art. 49 AML i R9: dokumentację przechowuje się co najmniej 5 lat, a na razie bez terminu.
CREATE TRIGGER rejestr_kontroli_zawiadomienia_wpisy_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_zawiadomienia_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_zadania_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_zawiadomienia_zadania
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_kontrole_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_zawiadomienia_kontrole
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;
