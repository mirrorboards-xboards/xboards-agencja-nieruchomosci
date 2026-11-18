-- Sprawdzenia: każde sprawdzenie stanu prawnego osobno, np. dział III księgi, pierwokup gminy albo
-- zgodność projektu aktu z umową przedwstępną. Analiza w teczce transakcji zbiera wnioski dla stron;
-- pamięć trzyma pojedyncze sprawdzenia z dokumentem, przepisem i wynikiem, na których analiza stoi.
--
-- Sprawdzenie działu księgi stoi na stanie księgi z chwili odpisu albo przeglądania
-- (analityk-stanu-prawnego-ksiega). Sprawdzenia się nie zmienia; wolno raz odnotować analizę,
-- do której weszło. Ponowne sprawdzenie, np. po nowym odpisie, jest nowym wierszem, a poprzednie
-- wskazuje je jako zastępujące.
CREATE TABLE analityk_stanu_prawnego_sprawdzenia_wpisy (
  id TEXT PRIMARY KEY,
  przedmiot TEXT NOT NULL CHECK (przedmiot IN ('dzial_i', 'dzial_ii', 'dzial_iii', 'dzial_iv', 'pierwokup', 'ograniczenia', 'tytul_zbywcy', 'projekt_umowy')),
  -- Sprawdzany dokument z teczki, np. odpis księgi albo projekt umowy, i jego data.
  dokument_odnosnik TEXT NOT NULL CHECK (dokument_odnosnik <> ''),
  dokument_data TEXT NOT NULL,
  -- Np. „art. 109 ust. 1 pkt 1 ugn”.
  przepis TEXT NOT NULL CHECK (przepis <> ''),
  wynik TEXT NOT NULL CHECK (wynik IN ('bez_uwag', 'ryzyko', 'nie_mozna_ustalic')),
  opis TEXT,
  -- Analiza w teczce transakcji, do której weszło sprawdzenie, tekstem.
  analiza_odnosnik TEXT,
  ksiega_stan_id INTEGER REFERENCES analityk_stanu_prawnego_ksiega_stany (id),
  sprawdzono TEXT NOT NULL,
  zastapione_przez_id TEXT REFERENCES analityk_stanu_prawnego_sprawdzenia_wpisy (id),

  CONSTRAINT ryzyko_z_opisem CHECK (wynik = 'bez_uwag' OR opis IS NOT NULL),
  CONSTRAINT dzial_ksiegi_ze_stanem CHECK (
    przedmiot NOT IN ('dzial_i', 'dzial_ii', 'dzial_iii', 'dzial_iv')
    OR ksiega_stan_id IS NOT NULL
  ),
  CONSTRAINT dokument_nie_po_sprawdzeniu CHECK (dokument_data <= sprawdzono),
  CONSTRAINT zastapione_nie_przez_siebie CHECK (zastapione_przez_id IS NULL OR zastapione_przez_id <> id)
) STRICT;

-- Analiza i zastąpienie przychodzą później, więc sprawdzenie powstaje aktualne i bez analizy.
CREATE TRIGGER analityk_stanu_prawnego_sprawdzenia_powstaje_aktualne
BEFORE INSERT ON analityk_stanu_prawnego_sprawdzenia_wpisy
WHEN NEW.zastapione_przez_id IS NOT NULL
  OR NEW.analiza_odnosnik IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'sprawdzenie powstaje aktualne i bez analizy; obie rzeczy odnotowuje się później');
END;

-- Sprawdzenie jest zapisem tego, co Analityk wiedział w chwili sprawdzenia; zmienia się tylko raz
-- odnotowana analiza i raz odnotowane zastąpienie.
CREATE TRIGGER analityk_stanu_prawnego_sprawdzenia_bez_zmian
BEFORE UPDATE ON analityk_stanu_prawnego_sprawdzenia_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.przedmiot IS NOT OLD.przedmiot
  OR NEW.dokument_odnosnik IS NOT OLD.dokument_odnosnik
  OR NEW.dokument_data IS NOT OLD.dokument_data
  OR NEW.przepis IS NOT OLD.przepis
  OR NEW.wynik IS NOT OLD.wynik
  OR NEW.opis IS NOT OLD.opis
  OR NEW.ksiega_stan_id IS NOT OLD.ksiega_stan_id
  OR NEW.sprawdzono IS NOT OLD.sprawdzono
  OR (OLD.analiza_odnosnik IS NOT NULL AND NEW.analiza_odnosnik IS NOT OLD.analiza_odnosnik)
  OR (OLD.zastapione_przez_id IS NOT NULL AND NEW.zastapione_przez_id IS NOT OLD.zastapione_przez_id)
BEGIN
  SELECT RAISE(ABORT, 'sprawdzenia się nie zmienia; wolno raz odnotować analizę i raz zastąpienie');
END;

-- Ponowne sprawdzenie dotyczy tego samego przedmiotu i nie jest starsze od tego, które zastępuje.
CREATE TRIGGER analityk_stanu_prawnego_sprawdzenia_zastepuje_ponowne
BEFORE UPDATE OF zastapione_przez_id ON analityk_stanu_prawnego_sprawdzenia_wpisy
WHEN NEW.zastapione_przez_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM analityk_stanu_prawnego_sprawdzenia_wpisy AS ponowne
    WHERE ponowne.id = NEW.zastapione_przez_id
      AND ponowne.przedmiot = OLD.przedmiot
      AND ponowne.sprawdzono >= OLD.sprawdzono
  )
BEGIN
  SELECT RAISE(ABORT, 'sprawdzenie zastępuje ponowne sprawdzenie tego samego przedmiotu, nie starsze');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER analityk_stanu_prawnego_sprawdzenia_wpisy_bez_zastepowania
BEFORE INSERT ON analityk_stanu_prawnego_sprawdzenia_wpisy
WHEN EXISTS (
  SELECT 1
  FROM analityk_stanu_prawnego_sprawdzenia_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: sprawdzenie zostaje, bo po latach strona może zapytać, jaki był stan prawny w chwili
-- transakcji.
CREATE TRIGGER analityk_stanu_prawnego_sprawdzenia_wpisy_bez_usuwania
BEFORE DELETE ON analityk_stanu_prawnego_sprawdzenia_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
