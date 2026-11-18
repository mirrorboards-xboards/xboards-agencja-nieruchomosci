-- Stan księgi: data i godzina każdego odpisu albo przeglądania księgi wieczystej, na którym stoi
-- analiza, i lista wpisów, wzmianek i ostrzeżeń z tej chwili. Dzięki temu Analityk widzi, co się
-- zmieniło przed aktem (operacja zmiany_ksiegi).
--
-- Sam odpis leży w teczce transakcji; pamięć wskazuje go odnośnikiem. Wpis księgi trzyma tylko
-- oznaczenie i przedmiot, np. „hipoteka umowna na rzecz banku” albo „wzmianka o wniosku”, bez
-- danych osób, bo te są w odpisie w teczce. Tego baza nie sprawdza: pilnuje norma stanowiska.
CREATE TABLE analityk_stanu_prawnego_ksiega_stany (
  -- Klucz nadany przez bazę jest dodatni; zob. analityk_stanu_prawnego_ksiega_stany_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  numer_kw TEXT NOT NULL CHECK (numer_kw GLOB '[A-Z][A-Z][0-9][A-Z]/[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]/[0-9]'),
  -- Data i godzina, z której odpis albo przeglądanie pokazuje księgę.
  chwila TEXT NOT NULL,
  sposob TEXT NOT NULL CHECK (sposob IN ('odpis', 'przegladanie')),
  -- Odpis w teczce transakcji: pokój i wiersz dokumentu, tekstem.
  dokument_odnosnik TEXT,

  -- Wydruk z przeglądania nie ma mocy dokumentu sądowego, ale odpis jest dokumentem w teczce.
  CONSTRAINT odpis_z_dokumentem CHECK (sposob <> 'odpis' OR dokument_odnosnik IS NOT NULL),
  CONSTRAINT jeden_stan_na_chwile UNIQUE (numer_kw, chwila)
) STRICT;

CREATE TABLE analityk_stanu_prawnego_ksiega_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. analityk_stanu_prawnego_ksiega_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  stan_id INTEGER NOT NULL REFERENCES analityk_stanu_prawnego_ksiega_stany (id),
  -- Dział I-O, I-Sp, II, III albo IV.
  dzial TEXT NOT NULL CHECK (dzial IN ('i_o', 'i_sp', 'ii', 'iii', 'iv')),
  -- Numer wpisu albo wzmianki w dziale, np. „1” albo „Dz.Kw./12345/26”.
  numer TEXT NOT NULL CHECK (numer <> ''),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('wpis', 'wzmianka', 'ostrzezenie')),
  przedmiot TEXT NOT NULL CHECK (przedmiot <> ''),

  CONSTRAINT wpis_raz_w_stanie UNIQUE (stan_id, dzial, rodzaj, numer)
) STRICT;

-- Nowy stan pokazuje księgę później niż poprzedni, więc ostatni stan jest zawsze najnowszy.
CREATE TRIGGER analityk_stanu_prawnego_ksiega_stan_po_poprzednim
BEFORE INSERT ON analityk_stanu_prawnego_ksiega_stany
WHEN EXISTS (
  SELECT 1
  FROM analityk_stanu_prawnego_ksiega_stany AS poprzedni
  WHERE poprzedni.numer_kw = NEW.numer_kw
    AND poprzedni.chwila >= NEW.chwila
)
BEGIN
  SELECT RAISE(ABORT, 'nowy stan księgi pokazuje ją później niż poprzedni');
END;

-- Wpisy zapisuje się razem ze stanem, więc stan, po którym przyszedł nowszy, jest zamknięty.
CREATE TRIGGER analityk_stanu_prawnego_ksiega_wpis_do_ostatniego_stanu
BEFORE INSERT ON analityk_stanu_prawnego_ksiega_wpisy
WHEN EXISTS (
  SELECT 1
  FROM analityk_stanu_prawnego_ksiega_stany AS stan
  JOIN analityk_stanu_prawnego_ksiega_stany AS nowszy
    ON nowszy.numer_kw = stan.numer_kw
    AND nowszy.chwila > stan.chwila
  WHERE stan.id = NEW.stan_id
)
BEGIN
  SELECT RAISE(ABORT, 'wpis księgi dopisuje się tylko do ostatniego stanu księgi');
END;

-- Stan księgi to zapis jednej chwili, więc się go nie zmienia.
CREATE TRIGGER analityk_stanu_prawnego_ksiega_stany_bez_zmian
BEFORE UPDATE ON analityk_stanu_prawnego_ksiega_stany
BEGIN
  SELECT RAISE(ABORT, 'stanu księgi ani jego wpisów się nie zmienia; nowy odpis to nowy stan');
END;

CREATE TRIGGER analityk_stanu_prawnego_ksiega_wpisy_bez_zmian
BEFORE UPDATE ON analityk_stanu_prawnego_ksiega_wpisy
BEGIN
  SELECT RAISE(ABORT, 'stanu księgi ani jego wpisów się nie zmienia; nowy odpis to nowy stan');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER analityk_stanu_prawnego_ksiega_stany_bez_zastepowania
BEFORE INSERT ON analityk_stanu_prawnego_ksiega_stany
WHEN EXISTS (
  SELECT 1
  FROM analityk_stanu_prawnego_ksiega_stany
  WHERE id = NEW.id
    OR (numer_kw = NEW.numer_kw AND chwila = NEW.chwila)
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER analityk_stanu_prawnego_ksiega_wpisy_bez_zastepowania
BEFORE INSERT ON analityk_stanu_prawnego_ksiega_wpisy
WHEN EXISTS (
  SELECT 1
  FROM analityk_stanu_prawnego_ksiega_wpisy
  WHERE id = NEW.id
    OR (stan_id = NEW.stan_id AND dzial = NEW.dzial AND rodzaj = NEW.rodzaj AND numer = NEW.numer)
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: stan księgi zostaje, bo po latach strona może zapytać, jaki był stan prawny w chwili
-- transakcji.
CREATE TRIGGER analityk_stanu_prawnego_ksiega_stany_bez_usuwania
BEFORE DELETE ON analityk_stanu_prawnego_ksiega_stany
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER analityk_stanu_prawnego_ksiega_wpisy_bez_usuwania
BEFORE DELETE ON analityk_stanu_prawnego_ksiega_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
