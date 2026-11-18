-- Polecenia: co agentowi polecił właściciel albo przełożony, zapisane osobno od ustaleń.
--
-- Agent przyjmuje tylko polecenie podpisane kluczem wydającego, bo niepodpisana wiadomość może
-- udawać właściciela. Podpis sprawdza aplikacja, a baza wymaga jego obecności.
CREATE TABLE stanowisko_polecenia_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. stanowisko_polecenia_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  wydal TEXT NOT NULL,
  wydal_jako TEXT NOT NULL CHECK (wydal_jako IN ('wlasciciel', 'przelozony')),
  otrzymano TEXT NOT NULL,
  tresc TEXT NOT NULL,
  podpis TEXT NOT NULL,
  stan TEXT NOT NULL CHECK (stan IN ('przyjete', 'wykonane', 'odmowione')),
  zamknieto TEXT,
  uwagi TEXT,

  CONSTRAINT zamkniete_z_data CHECK ((stan = 'przyjete') = (zamknieto IS NULL)),
  CONSTRAINT odmowa_z_uwagami CHECK (stan <> 'odmowione' OR uwagi IS NOT NULL)
) STRICT;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- triggerów UPDATE i DELETE, bo pokój nie włącza recursive_triggers; zmianę wiersza robi UPDATE.
CREATE TRIGGER stanowisko_polecenia_wpisy_bez_zastepowania
BEFORE INSERT ON stanowisko_polecenia_wpisy
WHEN EXISTS (
  SELECT 1
  FROM stanowisko_polecenia_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER stanowisko_polecenia_powstaje_przyjete
BEFORE INSERT ON stanowisko_polecenia_wpisy
WHEN NEW.stan <> 'przyjete'
BEGIN
  SELECT RAISE(ABORT, 'polecenie powstaje jako przyjęte');
END;

-- Podpis obejmuje treść, więc zmieniona treść nie byłaby już poleceniem wydającego.
CREATE TRIGGER stanowisko_polecenia_tresc_niezmienna
BEFORE UPDATE ON stanowisko_polecenia_wpisy
WHEN NEW.wydal IS NOT OLD.wydal
  OR NEW.wydal_jako IS NOT OLD.wydal_jako
  OR NEW.otrzymano IS NOT OLD.otrzymano
  OR NEW.tresc IS NOT OLD.tresc
  OR NEW.podpis IS NOT OLD.podpis
BEGIN
  SELECT RAISE(ABORT, 'treści ani podpisu polecenia się nie zmienia');
END;

CREATE TRIGGER stanowisko_polecenia_stan_tylko_naprzod
BEFORE UPDATE ON stanowisko_polecenia_wpisy
WHEN OLD.stan <> 'przyjete'
BEGIN
  SELECT RAISE(ABORT, 'wykonanego ani odmówionego polecenia się nie zmienia');
END;
