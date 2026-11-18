-- Rodzaj oferty i jej widoczność z umowy pośrednictwa, w wersjach. Pierwszą wersję wpisuje Opiekun
-- oferty przy zawartej umowie, a każdą następną przy aneksie, który ją zmienia. Członkowie teczki
-- ją czytają, a Kierownik pośrednictwa przepisuje ją do katalogu ofert (katalog wskazuje wersję
-- tekstem, więc wersja ma UUID).
--
-- Oferta niepubliczna, czyli poza rynkiem, ma granice zgody: komu i co biuro może pokazać, np.
-- „tylko klientom z potwierdzonym finansowaniem, bez zdjęć wnętrz”. Ogłasza się tylko ofertę
-- publiczną (reguły w oferta-ogloszenia).
CREATE TABLE oferta_widocznosc_wersje (
  id TEXT PRIMARY KEY,
  umowa_id TEXT NOT NULL REFERENCES sprawa_umowy_wpisy (id),
  rodzaj_oferty TEXT NOT NULL CHECK (rodzaj_oferty IN ('sprzedaz', 'najem', 'sprzedaz_albo_najem')),
  widocznosc TEXT NOT NULL CHECK (widocznosc IN ('publiczna', 'niepubliczna')),
  granice_zgody TEXT,
  -- Zdarzenie aneks umowy, które zmieniło widoczność; pierwsza wersja wynika z samej umowy.
  aneks_zdarzenie_id TEXT REFERENCES sprawa_umowy_zdarzenia (id),
  -- Od kiedy wersja obowiązuje: chwila zawarcia umowy albo chwila aneksu.
  od TEXT NOT NULL,

  CONSTRAINT niepubliczna_ma_granice_zgody CHECK ((widocznosc = 'niepubliczna') = (granice_zgody IS NOT NULL))
) STRICT;

-- Bieżąca widoczność: najnowsza wersja. Teczka oferty ma jedną umowę pośrednictwa.
CREATE VIEW oferta_widocznosc_biezaca AS
SELECT *
FROM oferta_widocznosc_wersje
WHERE od = (
  SELECT max(od)
  FROM oferta_widocznosc_wersje
);

-- Pierwsza wersja umowy obowiązuje od jej zawarcia i nie wskazuje aneksu.
CREATE TRIGGER oferta_widocznosc_pierwsza_z_zawartej_umowy
BEFORE INSERT ON oferta_widocznosc_wersje
WHEN NOT EXISTS (
    SELECT 1
    FROM oferta_widocznosc_wersje
    WHERE umowa_id = NEW.umowa_id
  )
  AND (
    NEW.aneks_zdarzenie_id IS NOT NULL
    OR NOT EXISTS (
      SELECT 1
      FROM sprawa_umowy_wpisy
      WHERE id = NEW.umowa_id
        AND stan = 'zawarta'
        AND zawarto = NEW.od
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'pierwsza wersja widoczności powstaje przy zawartej umowie i obowiązuje od jej zawarcia');
END;

-- Widoczność i rodzaj oferty zmienia tylko aneks podpisany przez klienta: każda następna wersja
-- wskazuje zdarzenie aneks tej umowy, obowiązuje od jego chwili i jest późniejsza niż poprzednia.
CREATE TRIGGER oferta_widocznosc_kolejna_z_aneksu
BEFORE INSERT ON oferta_widocznosc_wersje
WHEN EXISTS (
    SELECT 1
    FROM oferta_widocznosc_wersje
    WHERE umowa_id = NEW.umowa_id
  )
  AND (
    NOT EXISTS (
      SELECT 1
      FROM sprawa_umowy_zdarzenia
      WHERE id = NEW.aneks_zdarzenie_id
        AND umowa_id = NEW.umowa_id
        AND rodzaj = 'aneks'
        AND chwila = NEW.od
    )
    OR NOT EXISTS (
      SELECT 1
      FROM sprawa_umowy_wpisy
      WHERE id = NEW.umowa_id
        AND stan = 'zawarta'
    )
    OR EXISTS (
      SELECT 1
      FROM oferta_widocznosc_wersje
      WHERE umowa_id = NEW.umowa_id
        AND od >= NEW.od
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'kolejna wersja widoczności wskazuje aneks zawartej umowy, obowiązuje od jego chwili i jest późniejsza niż poprzednia');
END;

CREATE TRIGGER oferta_widocznosc_wersje_bez_zmian
BEFORE UPDATE ON oferta_widocznosc_wersje
BEGIN
  SELECT RAISE(ABORT, 'wersji widoczności się nie zmienia; zmianę z aneksu wpisuje się jako nową wersję');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER oferta_widocznosc_wersje_bez_zastepowania
BEFORE INSERT ON oferta_widocznosc_wersje
WHEN EXISTS (
  SELECT 1
  FROM oferta_widocznosc_wersje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_widocznosc_wersje_bez_usuwania
BEFORE DELETE ON oferta_widocznosc_wersje
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
