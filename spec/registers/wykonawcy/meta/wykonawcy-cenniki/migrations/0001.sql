-- Cenniki wykonawców w wersjach i ich przypisanie do wykonawców. Jeden cennik może obowiązywać
-- kilku wykonawców; wtedy wszyscy pracują na tych samych warunkach. Zlecenie w teczce lokalu
-- wskazuje wersję cennika tekstem, więc koszt w starym zestawieniu nie zmienia się po nowej
-- wersji.
--
-- Wersję zatwierdza decyzja człowieka rodzaju „cennik” (wykonawcy_spis_decyzje), której podpis
-- obejmuje skrót całej wersji z pozycjami; skrót liczy i sprawdza aplikacja. Kierownik najmu
-- przepisuje wersję jako projekt, dopisuje pozycje i zamyka ją. Od zamknięcia wersji ani jej pozycji
-- się nie zmienia, a Koordynator stosuje tylko wersję zamkniętą. Wersja nie obowiązuje wstecz.
CREATE TABLE wykonawcy_cenniki_wpisy (
  id TEXT PRIMARY KEY,
  -- Np. „Sprzątanie Stare Miasto”.
  nazwa TEXT NOT NULL CHECK (nazwa <> ''),
  wpisano TEXT NOT NULL
) STRICT;

CREATE TABLE wykonawcy_cenniki_wersje (
  id TEXT PRIMARY KEY,
  cennik_id TEXT NOT NULL REFERENCES wykonawcy_cenniki_wpisy (id),
  -- Kolejne numery od 1.
  numer INTEGER NOT NULL CHECK (numer >= 1),
  -- Dzień, od którego wersja obowiązuje.
  obowiazuje_od TEXT NOT NULL,
  -- Chwila wpisu do rejestru.
  wpisano TEXT NOT NULL,
  stan TEXT NOT NULL CHECK (stan IN ('projekt', 'zamknieta')),
  decyzja_id TEXT NOT NULL REFERENCES wykonawcy_spis_decyzje (id),

  -- Wersja nie obowiązuje wstecz: dzień obowiązywania nie jest wcześniejszy niż dzień wpisu.
  CONSTRAINT obowiazuje_od_nie_przed_wpisem CHECK (
    obowiazuje_od = date(obowiazuje_od)
    AND obowiazuje_od >= date(wpisano)
  ),

  UNIQUE (cennik_id, numer)
) STRICT;

-- Pozycje wersji: usługa albo dopłata, jednostka i cena w groszach.
CREATE TABLE wykonawcy_cenniki_pozycje (
  -- Klucz nadany przez bazę jest dodatni; zob. wykonawcy_cenniki_pozycje_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  wersja_id TEXT NOT NULL REFERENCES wykonawcy_cenniki_wersje (id),
  -- Np. „sprzątanie po wyjeździe gościa”, „praca w święto”.
  usluga TEXT NOT NULL CHECK (usluga <> ''),
  -- Dopłata do usługi, np. za okno krótsze niż trzy godziny albo dojazd poza obszar.
  doplata INTEGER NOT NULL CHECK (doplata IN (0, 1)),
  -- Np. „lokal do 40 m²”, „komplet pościeli”, „godzina”, „wizyta”.
  jednostka TEXT NOT NULL CHECK (jednostka <> ''),
  cena INTEGER NOT NULL CHECK (cena >= 0),
  -- Cena zawiera podatek VAT.
  z_vat INTEGER NOT NULL CHECK (z_vat IN (0, 1)),
  -- Np. „najkrótsza wizyta 1 godzina”, „odwołanie bez opłaty do 24 godzin przed”.
  warunki TEXT
) STRICT;

-- Wykonawcy, którzy pracują według cennika, od dnia do dnia, z decyzją przypisania i decyzją
-- jego końca.
CREATE TABLE wykonawcy_cenniki_przypisania (
  -- Klucz nadany przez bazę jest dodatni; zob. wykonawcy_cenniki_przypisania_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  cennik_id TEXT NOT NULL REFERENCES wykonawcy_cenniki_wpisy (id),
  wykonawca_id TEXT NOT NULL REFERENCES wykonawcy_spis_wpisy (id),
  od TEXT NOT NULL,
  do TEXT,
  decyzja_id TEXT NOT NULL REFERENCES wykonawcy_spis_decyzje (id),
  zakonczenie_decyzja_id TEXT REFERENCES wykonawcy_spis_decyzje (id),

  CONSTRAINT koniec_z_decyzja CHECK ((do IS NULL) = (zakonczenie_decyzja_id IS NULL)),
  CONSTRAINT koniec_nie_przed_poczatkiem CHECK (do IS NULL OR do >= od)
) STRICT;

-- Wersje ---------------------------------------------------------------------------------------

-- Pozycje dopisuje się do projektu, a zamknięcie sprawdza zmianę stanu, więc wersja nie może
-- powstać od razu zamknięta.
CREATE TRIGGER wykonawcy_cenniki_wersja_powstaje_jako_projekt
BEFORE INSERT ON wykonawcy_cenniki_wersje
WHEN NEW.stan <> 'projekt'
BEGIN
  SELECT RAISE(ABORT, 'wersja cennika powstaje jako projekt');
END;

CREATE TRIGGER wykonawcy_cenniki_wersja_po_decyzji_o_cenniku
BEFORE INSERT ON wykonawcy_cenniki_wersje
WHEN (
  SELECT rodzaj
  FROM wykonawcy_spis_decyzje
  WHERE id = NEW.decyzja_id
) IS NOT 'cennik'
BEGIN
  SELECT RAISE(ABORT, 'wersję cennika wpisuje się po decyzji o cenniku');
END;

-- Kolejna wersja ma kolejny numer i nie obowiązuje wcześniej niż poprzednia, więc obowiązującą
-- wersję wyznacza dzień.
CREATE TRIGGER wykonawcy_cenniki_wersja_kolejna
BEFORE INSERT ON wykonawcy_cenniki_wersje
WHEN NEW.numer IS NOT (
    SELECT coalesce(max(numer), 0) + 1
    FROM wykonawcy_cenniki_wersje
    WHERE cennik_id = NEW.cennik_id
  )
  OR NEW.obowiazuje_od < (
    SELECT max(obowiazuje_od)
    FROM wykonawcy_cenniki_wersje
    WHERE cennik_id = NEW.cennik_id
  )
BEGIN
  SELECT RAISE(ABORT, 'wersja cennika ma kolejny numer i nie obowiązuje wcześniej niż poprzednia');
END;

CREATE TRIGGER wykonawcy_cenniki_zamknieta_ma_pozycje
BEFORE UPDATE OF stan ON wykonawcy_cenniki_wersje
WHEN NEW.stan = 'zamknieta'
  AND NOT EXISTS (
    SELECT 1
    FROM wykonawcy_cenniki_pozycje
    WHERE wersja_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'zamknięta wersja cennika ma co najmniej jedną pozycję');
END;

-- Jedyną zmianą wersji jest zamknięcie projektu; podpis decyzji obejmuje całą wersję.
CREATE TRIGGER wykonawcy_cenniki_wersje_bez_zmian
BEFORE UPDATE ON wykonawcy_cenniki_wersje
WHEN OLD.stan <> 'projekt'
  OR NEW.id IS NOT OLD.id
  OR NEW.cennik_id IS NOT OLD.cennik_id
  OR NEW.numer IS NOT OLD.numer
  OR NEW.obowiazuje_od IS NOT OLD.obowiazuje_od
  OR NEW.wpisano IS NOT OLD.wpisano
  OR NEW.decyzja_id IS NOT OLD.decyzja_id
BEGIN
  SELECT RAISE(ABORT, 'wersji cennika się nie zmienia; nowa decyzja to nowa wersja');
END;

-- Pozycje --------------------------------------------------------------------------------------

CREATE TRIGGER wykonawcy_cenniki_pozycje_tylko_w_projekcie
BEFORE INSERT ON wykonawcy_cenniki_pozycje
WHEN (
  SELECT stan
  FROM wykonawcy_cenniki_wersje
  WHERE id = NEW.wersja_id
) IS NOT 'projekt'
BEGIN
  SELECT RAISE(ABORT, 'pozycje dopisuje się tylko do projektu wersji cennika');
END;

-- Pomyłkę w pozycji projektu poprawia się przed zamknięciem; potem pozycji się nie zmienia.
CREATE TRIGGER wykonawcy_cenniki_pozycje_bez_zmian
BEFORE UPDATE ON wykonawcy_cenniki_pozycje
WHEN NEW.id IS NOT OLD.id
  OR NEW.wersja_id IS NOT OLD.wersja_id
  OR (
    SELECT stan
    FROM wykonawcy_cenniki_wersje
    WHERE id = OLD.wersja_id
  ) IS NOT 'projekt'
BEGIN
  SELECT RAISE(ABORT, 'pozycji zamkniętej wersji cennika się nie zmienia');
END;

-- Przypisania ----------------------------------------------------------------------------------

-- Wykonawca pracuje według cennika, który ma zamkniętą wersję, i tylko po decyzji o przypisaniu.
CREATE TRIGGER wykonawcy_cenniki_przypisanie_po_decyzji
BEFORE INSERT ON wykonawcy_cenniki_przypisania
WHEN (
  SELECT rodzaj
  FROM wykonawcy_spis_decyzje
  WHERE id = NEW.decyzja_id
) IS NOT 'przypisanie'
BEGIN
  SELECT RAISE(ABORT, 'przypisanie cennika i jego koniec wpisuje się po decyzji o przypisaniu');
END;

CREATE TRIGGER wykonawcy_cenniki_koniec_przypisania_po_decyzji
BEFORE UPDATE OF zakonczenie_decyzja_id ON wykonawcy_cenniki_przypisania
WHEN NEW.zakonczenie_decyzja_id IS NOT NULL
  AND (
    SELECT rodzaj
    FROM wykonawcy_spis_decyzje
    WHERE id = NEW.zakonczenie_decyzja_id
  ) IS NOT 'przypisanie'
BEGIN
  SELECT RAISE(ABORT, 'przypisanie cennika i jego koniec wpisuje się po decyzji o przypisaniu');
END;

CREATE TRIGGER wykonawcy_cenniki_przypisanie_cennika_z_wersja
BEFORE INSERT ON wykonawcy_cenniki_przypisania
WHEN NOT EXISTS (
  SELECT 1
  FROM wykonawcy_cenniki_wersje
  WHERE cennik_id = NEW.cennik_id
    AND stan = 'zamknieta'
)
BEGIN
  SELECT RAISE(ABORT, 'przypisuje się cennik, który ma zamkniętą wersję');
END;

-- Wstrzymany i wykreślony wykonawca nie dostaje zleceń, więc nie dostaje też nowego cennika.
CREATE TRIGGER wykonawcy_cenniki_przypisanie_zatwierdzonego
BEFORE INSERT ON wykonawcy_cenniki_przypisania
WHEN (
  SELECT stan
  FROM wykonawcy_spis_stany
  WHERE wykonawca_id = NEW.wykonawca_id
  ORDER BY id DESC
  LIMIT 1
) IS NOT 'zatwierdzony'
BEGIN
  SELECT RAISE(ABORT, 'cennik przypisuje się zatwierdzonemu wykonawcy');
END;

CREATE TRIGGER wykonawcy_cenniki_przypisanie_powstaje_biezace
BEFORE INSERT ON wykonawcy_cenniki_przypisania
WHEN NEW.do IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'przypisanie cennika powstaje jako bieżące');
END;

CREATE TRIGGER wykonawcy_cenniki_przypisania_bez_zmian
BEFORE UPDATE ON wykonawcy_cenniki_przypisania
WHEN NEW.id IS NOT OLD.id
  OR NEW.cennik_id IS NOT OLD.cennik_id
  OR NEW.wykonawca_id IS NOT OLD.wykonawca_id
  OR NEW.od IS NOT OLD.od
  OR NEW.decyzja_id IS NOT OLD.decyzja_id
  OR (OLD.do IS NOT NULL AND (NEW.do IS NOT OLD.do OR NEW.zakonczenie_decyzja_id IS NOT OLD.zakonczenie_decyzja_id))
BEGIN
  SELECT RAISE(ABORT, 'przypisania cennika się nie zmienia; może się tylko raz zakończyć decyzją');
END;

-- Wiersze bez zmian ----------------------------------------------------------------------------

CREATE TRIGGER wykonawcy_cenniki_wpisy_bez_zmian
BEFORE UPDATE ON wykonawcy_cenniki_wpisy
BEGIN
  SELECT RAISE(ABORT, 'cennika się nie zmienia; zmiana stawek to nowa wersja');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabel z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER wykonawcy_cenniki_wpisy_bez_zastepowania
BEFORE INSERT ON wykonawcy_cenniki_wpisy
WHEN EXISTS (
  SELECT 1
  FROM wykonawcy_cenniki_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER wykonawcy_cenniki_wersje_bez_zastepowania
BEFORE INSERT ON wykonawcy_cenniki_wersje
WHEN EXISTS (
    SELECT 1
    FROM wykonawcy_cenniki_wersje
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM wykonawcy_cenniki_wersje
    WHERE cennik_id = NEW.cennik_id
      AND numer = NEW.numer
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER wykonawcy_cenniki_pozycje_bez_zastepowania
BEFORE INSERT ON wykonawcy_cenniki_pozycje
WHEN EXISTS (
  SELECT 1
  FROM wykonawcy_cenniki_pozycje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER wykonawcy_cenniki_przypisania_bez_zastepowania
BEFORE INSERT ON wykonawcy_cenniki_przypisania
WHEN EXISTS (
  SELECT 1
  FROM wykonawcy_cenniki_przypisania
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER wykonawcy_cenniki_wpisy_bez_usuwania
BEFORE DELETE ON wykonawcy_cenniki_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER wykonawcy_cenniki_wersje_bez_usuwania
BEFORE DELETE ON wykonawcy_cenniki_wersje
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER wykonawcy_cenniki_pozycje_bez_usuwania
BEFORE DELETE ON wykonawcy_cenniki_pozycje
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER wykonawcy_cenniki_przypisania_bez_usuwania
BEFORE DELETE ON wykonawcy_cenniki_przypisania
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;
