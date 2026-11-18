-- Obie strony (rozstrzygnięcie biura R1): pary teczki oferty i teczki poszukiwania, które spotkały
-- się w jednej transakcji, i to, czy umowy pośrednictwa obu stron pozwalają biuru działać dla
-- drugiej strony.
--
-- Wiersz trzyma tylko odnośniki do teczek i umów, bez cen, budżetów ani danych klientów, bo
-- kierownik nie zestawia danych obu stron. Para powstaje do wyjaśnienia, a „tak” wymaga wskazania
-- obu umów. Ofertę przekazuje się dalej do Transakcji bez zgody obu umów tylko za zgodą człowieka;
-- jego podpis kierownik przenosi do wiersza. Po przekazaniu ocena umów się nie zmienia.
CREATE TABLE kierownik_posrednictwa_obie_strony_pary (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_posrednictwa_obie_strony_pary_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Pokoje teczek obu stron, tekstem.
  teczka_oferty_odnosnik TEXT NOT NULL CHECK (teczka_oferty_odnosnik <> ''),
  teczka_poszukiwania_odnosnik TEXT NOT NULL CHECK (teczka_poszukiwania_odnosnik <> ''),
  stwierdzono TEXT NOT NULL,
  umowy_pozwalaja TEXT NOT NULL CHECK (umowy_pozwalaja IN ('tak', 'nie', 'do_wyjasnienia')),
  -- Umowy pośrednictwa w obu teczkach, tekstem: pokój i wiersz.
  umowa_oferty_odnosnik TEXT,
  umowa_poszukiwania_odnosnik TEXT,
  -- Kiedy kierownik przekazał przyjętą ofertę Kierownikowi transakcji, i teczka transakcji.
  przekazano TEXT,
  transakcja_odnosnik TEXT,
  zatwierdzil TEXT,
  podpis TEXT,
  zatwierdzono TEXT,

  CONSTRAINT para_teczek_raz UNIQUE (teczka_oferty_odnosnik, teczka_poszukiwania_odnosnik),
  CONSTRAINT zgoda_umow_wskazuje_obie_umowy CHECK (
    umowy_pozwalaja <> 'tak'
    OR (umowa_oferty_odnosnik IS NOT NULL AND umowa_poszukiwania_odnosnik IS NOT NULL)
  ),
  CONSTRAINT przekazanie_za_zgoda_umow_albo_czlowieka CHECK (
    przekazano IS NULL
    OR umowy_pozwalaja = 'tak'
    OR zatwierdzil IS NOT NULL
  ),
  CONSTRAINT transakcja_po_przekazaniu CHECK (transakcja_odnosnik IS NULL OR przekazano IS NOT NULL),
  CONSTRAINT przekazanie_nie_przed_stwierdzeniem CHECK (przekazano IS NULL OR przekazano >= stwierdzono),
  CONSTRAINT zatwierdzenie_w_komplecie CHECK (
    (zatwierdzil IS NULL) = (podpis IS NULL)
    AND (zatwierdzil IS NULL) = (zatwierdzono IS NULL)
  )
) STRICT;

CREATE TRIGGER kierownik_posrednictwa_obie_strony_para_powstaje_do_wyjasnienia
BEFORE INSERT ON kierownik_posrednictwa_obie_strony_pary
WHEN NEW.umowy_pozwalaja <> 'do_wyjasnienia'
  OR NEW.przekazano IS NOT NULL
  OR NEW.transakcja_odnosnik IS NOT NULL
  OR NEW.zatwierdzil IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'para powstaje do wyjaśnienia i nieprzekazana');
END;

-- Teczki pary się nie zmieniają; po przekazaniu nie zmienia się też ocena umów ani zgoda
-- człowieka, a wolno raz wpisać teczkę transakcji.
CREATE TRIGGER kierownik_posrednictwa_obie_strony_bez_zmian
BEFORE UPDATE ON kierownik_posrednictwa_obie_strony_pary
WHEN NEW.id IS NOT OLD.id
  OR NEW.teczka_oferty_odnosnik IS NOT OLD.teczka_oferty_odnosnik
  OR NEW.teczka_poszukiwania_odnosnik IS NOT OLD.teczka_poszukiwania_odnosnik
  OR NEW.stwierdzono IS NOT OLD.stwierdzono
  OR (OLD.przekazano IS NOT NULL AND (
    NEW.umowy_pozwalaja IS NOT OLD.umowy_pozwalaja
    OR NEW.umowa_oferty_odnosnik IS NOT OLD.umowa_oferty_odnosnik
    OR NEW.umowa_poszukiwania_odnosnik IS NOT OLD.umowa_poszukiwania_odnosnik
    OR NEW.przekazano IS NOT OLD.przekazano
    OR NEW.zatwierdzil IS NOT OLD.zatwierdzil
    OR NEW.podpis IS NOT OLD.podpis
    OR NEW.zatwierdzono IS NOT OLD.zatwierdzono
  ))
  OR (OLD.transakcja_odnosnik IS NOT NULL AND NEW.transakcja_odnosnik IS NOT OLD.transakcja_odnosnik)
BEGIN
  SELECT RAISE(ABORT, 'teczek pary się nie zmienia, a po przekazaniu także oceny umów; teczkę transakcji wpisuje się raz');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers; także parę tych samych
-- teczek. Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER kierownik_posrednictwa_obie_strony_pary_bez_zastepowania
BEFORE INSERT ON kierownik_posrednictwa_obie_strony_pary
WHEN EXISTS (
  SELECT 1
  FROM kierownik_posrednictwa_obie_strony_pary
  WHERE id = NEW.id
    OR (
      teczka_oferty_odnosnik = NEW.teczka_oferty_odnosnik
      AND teczka_poszukiwania_odnosnik = NEW.teczka_poszukiwania_odnosnik
    )
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: pary zostają jako ślad transakcji prowadzonych dla obu stron.
CREATE TRIGGER kierownik_posrednictwa_obie_strony_pary_bez_usuwania
BEFORE DELETE ON kierownik_posrednictwa_obie_strony_pary
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
