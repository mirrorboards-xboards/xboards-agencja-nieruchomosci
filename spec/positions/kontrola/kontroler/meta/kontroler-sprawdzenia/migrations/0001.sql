-- Sprawdzenia: każde sprawdzenie Kontrolera z przeciwdziałania praniu pieniędzy osobno, np. tożsamość
-- klientki przed umową pośrednictwa, beneficjenci spółki kupującej albo lista sankcyjna. Leżą
-- w pamięci, a nie w teczce, bo teczkę czytają agenci, którzy rozmawiają z klientami. Sprawdzenia
-- spoza przeciwdziałania praniu pieniędzy (rachunek, tytuł, pełnomocnictwo) leżą w teczce.
--
-- Sprawdzenie robocze czeka na przekazanie Kierownikowi kontroli. Przekazanego sprawdzenia się nie
-- zmienia. Gdy Kierownik kontroli przeniesie je do rejestru kontroli i poda wpis, Kontroler czyści
-- opis, czynniki i proponowany poziom (D13): zostają rodzaj, źródło z datą, wynik i odnośnik do
-- rejestru, a szczegóły leżą już tylko w rejestrze. Ponowne sprawdzenie jest nowym wierszem,
-- a poprzednie wskazuje je jako zastępujące.
CREATE TABLE kontroler_sprawdzenia_wpisy (
  id TEXT PRIMARY KEY,
  -- Rodzaje sprawdzeń rejestru kontroli i ocena ryzyka klienta.
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('identyfikacja', 'weryfikacja', 'beneficjenci', 'crbr', 'pep', 'sankcje', 'cel_stosunkow', 'zrodlo_srodkow', 'monitorowanie', 'ocena_ryzyka')),
  -- Teczka klienta, w której zaszło zdarzenie ze sprawdzeniem, tekstem.
  teczka_odnosnik TEXT NOT NULL CHECK (teczka_odnosnik <> ''),
  -- Sprawdzany wiersz teczki, np. uczestnik albo pełnomocnik: tabela i pokój z wierszem, tekstem.
  dotyczy_tabela TEXT,
  dotyczy_odnosnik TEXT,
  -- Źródło spoza teczki (np. RDO, KRS, CRBR, lista sankcyjna) z datą, w której je sprawdzono.
  zrodlo TEXT,
  zrodlo_data TEXT,
  -- Osoba albo instytucja, której oświadczenie sprawdzenie porównuje ze źródłem.
  zrodlo_id TEXT REFERENCES stanowisko_zrodla_wpisy (id),
  wynik TEXT NOT NULL CHECK (wynik IN ('zgodne', 'uwagi', 'nie_mozna_ustalic')),
  opis TEXT,
  -- Poziomy jak w ocenach rejestru kontroli.
  proponowany_poziom TEXT CHECK (proponowany_poziom IN ('niskie', 'srednie', 'wysokie')),
  czynniki TEXT,
  -- 1, gdy sprawdzenie jest powodem wątpliwości (znacznika w teczce).
  powod_watpliwosci INTEGER NOT NULL CHECK (powod_watpliwosci IN (0, 1)),
  sprawdzono TEXT NOT NULL,
  przekazano TEXT,
  -- Wpis w rejestrze kontroli, tekstem.
  rejestr_odnosnik TEXT,
  zastapione_przez_id TEXT REFERENCES kontroler_sprawdzenia_wpisy (id),

  CONSTRAINT dotyczy_tabela_razem_z_odnosnikiem CHECK ((dotyczy_tabela IS NULL) = (dotyczy_odnosnik IS NULL)),
  CONSTRAINT zrodlo_z_data CHECK ((zrodlo IS NULL) = (zrodlo_data IS NULL)),
  -- Przy identyfikacji, weryfikacji i beneficjentach zapisuje się źródło i datę.
  CONSTRAINT identyfikacja_ze_zrodlem_i_data CHECK (
    rodzaj NOT IN ('identyfikacja', 'weryfikacja', 'beneficjenci')
    OR zrodlo IS NOT NULL
  ),
  CONSTRAINT poziom_i_czynniki_tylko_przy_ocenie_ryzyka CHECK (
    rodzaj = 'ocena_ryzyka'
    OR (proponowany_poziom IS NULL AND czynniki IS NULL)
  ),
  -- Do wpisu w rejestrze ocena ryzyka niesie poziom i czynniki, a uwagi i brak ustalenia opis.
  CONSTRAINT ocena_ryzyka_z_poziomem_i_czynnikami CHECK (
    rodzaj <> 'ocena_ryzyka'
    OR rejestr_odnosnik IS NOT NULL
    OR (proponowany_poziom IS NOT NULL AND czynniki IS NOT NULL)
  ),
  CONSTRAINT uwagi_z_opisem CHECK (wynik = 'zgodne' OR rejestr_odnosnik IS NOT NULL OR opis IS NOT NULL),
  CONSTRAINT wpis_w_rejestrze_po_przekazaniu CHECK (rejestr_odnosnik IS NULL OR przekazano IS NOT NULL),
  CONSTRAINT przekazanie_nie_przed_sprawdzeniem CHECK (przekazano IS NULL OR przekazano >= sprawdzono),
  -- D13: szczegóły sprawdzenia po wpisie w rejestrze leżą tylko w rejestrze.
  CONSTRAINT po_wpisie_w_rejestrze_bez_szczegolow CHECK (
    rejestr_odnosnik IS NULL
    OR (opis IS NULL AND czynniki IS NULL AND proponowany_poziom IS NULL)
  ),
  CONSTRAINT zastapione_nie_przez_siebie CHECK (zastapione_przez_id IS NULL OR zastapione_przez_id <> id)
) STRICT;

-- Przekazanie, wpis w rejestrze i zastąpienie przychodzą później, więc sprawdzenie powstaje robocze.
CREATE TRIGGER kontroler_sprawdzenia_powstaje_robocze
BEFORE INSERT ON kontroler_sprawdzenia_wpisy
WHEN NEW.przekazano IS NOT NULL
  OR NEW.rejestr_odnosnik IS NOT NULL
  OR NEW.zastapione_przez_id IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'sprawdzenie powstaje robocze: bez przekazania, wpisu w rejestrze i zastąpienia');
END;

-- Po przekazaniu zmienia się tylko to, co przynosi rejestr: odnośnik do wpisu (raz) i razem z nim
-- wyczyszczone szczegóły, a poza tym zastąpienie przez ponowne sprawdzenie.
CREATE TRIGGER kontroler_sprawdzenia_przekazane_bez_zmian
BEFORE UPDATE ON kontroler_sprawdzenia_wpisy
WHEN OLD.przekazano IS NOT NULL
  AND (
    NEW.id IS NOT OLD.id
    OR NEW.rodzaj IS NOT OLD.rodzaj
    OR NEW.teczka_odnosnik IS NOT OLD.teczka_odnosnik
    OR NEW.dotyczy_tabela IS NOT OLD.dotyczy_tabela
    OR NEW.dotyczy_odnosnik IS NOT OLD.dotyczy_odnosnik
    OR NEW.zrodlo IS NOT OLD.zrodlo
    OR NEW.zrodlo_data IS NOT OLD.zrodlo_data
    OR NEW.zrodlo_id IS NOT OLD.zrodlo_id
    OR NEW.wynik IS NOT OLD.wynik
    OR NEW.powod_watpliwosci IS NOT OLD.powod_watpliwosci
    OR NEW.sprawdzono IS NOT OLD.sprawdzono
    OR NEW.przekazano IS NOT OLD.przekazano
    OR (OLD.rejestr_odnosnik IS NOT NULL AND NEW.rejestr_odnosnik IS NOT OLD.rejestr_odnosnik)
    OR (
      NEW.rejestr_odnosnik IS NULL
      AND (
        NEW.opis IS NOT OLD.opis
        OR NEW.czynniki IS NOT OLD.czynniki
        OR NEW.proponowany_poziom IS NOT OLD.proponowany_poziom
      )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'przekazanego sprawdzenia się nie zmienia; ponowne sprawdzenie to nowy wiersz');
END;

CREATE TRIGGER kontroler_sprawdzenia_zastapione_raz
BEFORE UPDATE OF zastapione_przez_id ON kontroler_sprawdzenia_wpisy
WHEN OLD.zastapione_przez_id IS NOT NULL
  AND NEW.zastapione_przez_id IS NOT OLD.zastapione_przez_id
BEGIN
  SELECT RAISE(ABORT, 'zastąpionego sprawdzenia nie zastępuje się drugi raz');
END;

-- Ponowne sprawdzenie dotyczy tego samego rodzaju i nie jest starsze od tego, które zastępuje.
CREATE TRIGGER kontroler_sprawdzenia_zastepuje_ponowne
BEFORE UPDATE OF zastapione_przez_id ON kontroler_sprawdzenia_wpisy
WHEN NEW.zastapione_przez_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM kontroler_sprawdzenia_wpisy AS ponowne
    WHERE ponowne.id = NEW.zastapione_przez_id
      AND ponowne.rodzaj = OLD.rodzaj
      AND ponowne.sprawdzono >= OLD.sprawdzono
  )
BEGIN
  SELECT RAISE(ABORT, 'sprawdzenie zastępuje ponowne sprawdzenie tego samego rodzaju, nie starsze');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER kontroler_sprawdzenia_wpisy_bez_zastepowania
BEFORE INSERT ON kontroler_sprawdzenia_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kontroler_sprawdzenia_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: sprawdzenie zostaje jako ślad tego, co i w jakim źródle Kontroler sprawdził.
CREATE TRIGGER kontroler_sprawdzenia_wpisy_bez_usuwania
BEFORE DELETE ON kontroler_sprawdzenia_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
