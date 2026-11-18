-- Zgody i oświadczenia klientów i innych osób spoza biura, np. zatwierdzenie ceny oferty albo
-- żądanie konsumenta, żeby biuro zaczęło pracę przed upływem terminu na odstąpienie. Dowodem jest
-- dokument albo wiadomość w teczce, a nie klucz. Zgody ustnej się nie wpisuje: jest ustaleniem
-- w pamięci agenta, dopóki klient jej nie potwierdzi.
--
-- Rodzaj zgody ustala typ sprawy tekstem. Rdzeń zna zadanie_rozpoczecia_pracy i odstapienie;
-- ich reguły przy umowie z biurem leżą w migracji sprawa-umowy. Zgody wpisuje agent sprawy.
CREATE TABLE sprawa_zgody_wpisy (
  id TEXT PRIMARY KEY,
  uczestnik_id TEXT NOT NULL REFERENCES sprawa_uczestnicy_wpisy (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj <> '' AND rodzaj NOT GLOB '*[^a-z_]*'),
  tresc TEXT NOT NULL,
  zlozono TEXT NOT NULL,
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  wplyw_id TEXT REFERENCES sprawa_korespondencja_wplywy (id),
  -- Oświadczenie na trwałym nośniku (art. 15 ust. 3 upk), np. podpisany plik albo wiadomość
  -- poczty, którą klient zachowuje.
  trwaly_nosnik INTEGER NOT NULL CHECK (trwaly_nosnik IN (0, 1)),
  -- Wiersz, którego zgoda dotyczy, np. cena w typie oferty; bez klucza obcego.
  dotyczy_tabela TEXT,
  dotyczy_id TEXT,
  -- Wcześniejsza zgoda, którą ta zastępuje.
  zastepuje_id TEXT REFERENCES sprawa_zgody_wpisy (id),

  CONSTRAINT zgoda_ma_dokument_albo_wiadomosc CHECK (dokument_id IS NOT NULL OR wplyw_id IS NOT NULL),
  CONSTRAINT dotyczy_tabela_razem_z_id CHECK ((dotyczy_tabela IS NULL) = (dotyczy_id IS NULL)),
  CONSTRAINT nie_zastepuje_samej_siebie CHECK (zastepuje_id IS NOT id)
) STRICT;

-- Dowodem zgody jest plik: dokument bez pliku (wymagany albo zamówiony) niczego nie dowodzi.
CREATE TRIGGER sprawa_zgody_dokument_z_plikiem
BEFORE INSERT ON sprawa_zgody_wpisy
WHEN NEW.dokument_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_dokumenty_wpisy
    WHERE id = NEW.dokument_id
      AND stan IN ('otrzymany', 'sprawdzony')
  )
BEGIN
  SELECT RAISE(ABORT, 'zgoda wskazuje dokument z plikiem: otrzymany albo sprawdzony');
END;

-- Dokument, który dowodzi zgody, nie zmienia rodzaju ani tego, czego dotyczy, i nie staje się
-- odrzucony ani zbędny; inaczej zgoda straciłaby dowód.
CREATE TRIGGER sprawa_zgody_dokument_zgody_zostaje
BEFORE UPDATE ON sprawa_dokumenty_wpisy
WHEN (
    NEW.rodzaj IS NOT OLD.rodzaj
    OR NEW.dotyczy_uczestnik_id IS NOT OLD.dotyczy_uczestnik_id
    OR NEW.dotyczy_nieruchomosc_id IS NOT OLD.dotyczy_nieruchomosc_id
    OR (NEW.stan IN ('odrzucony', 'zbedny') AND NEW.stan IS NOT OLD.stan)
  )
  AND EXISTS (
    SELECT 1
    FROM sprawa_zgody_wpisy
    WHERE dokument_id = OLD.id
  )
BEGIN
  SELECT RAISE(ABORT, 'dokument, który dowodzi zgody, nie zmienia rodzaju ani tego, czego dotyczy, i nie jest odrzucany');
END;

CREATE TRIGGER sprawa_zgody_bez_zmian
BEFORE UPDATE ON sprawa_zgody_wpisy
BEGIN
  SELECT RAISE(ABORT, 'zgody się nie zmienia; nowa zgoda zastępuje poprzednią');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_zgody_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_zgody_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_zgody_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_zgody_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_zgody_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
