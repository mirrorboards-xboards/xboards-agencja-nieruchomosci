-- Rachunki bankowe uczestników, które mają trafić do umowy albo przyjąć wypłatę. Podmiana rachunku
-- to częste oszustwo (R4), więc nowy numer nigdy nie zmienia istniejącego wiersza: powstaje nowy
-- wiersz, a stary zostaje z oznaczeniem, kto i kiedy go zastąpił.
--
-- Rachunek wpisuje agent sprawy, a potwierdzenie posiadacza innym kanałem ten, kto potwierdzał.
-- Wynik sprawdzenia Kontrolera leży w sprawa-sprawdzenia, a zgoda człowieka w sprawa-decyzje;
-- regułę, że zgoda wymaga sprawdzenia i potwierdzenia, zapisuje migracja sprawa-sprawdzenia
-- (sprawa_sprawdzenia_dopuszczenie_rachunku_wymaga_sprawdzenia). Rachunek, na który wolno płacić,
-- to wiersz widoku sprawa_rachunki_dopuszczone.
CREATE TABLE sprawa_rachunki_wpisy (
  id TEXT PRIMARY KEY,
  uczestnik_id TEXT NOT NULL REFERENCES sprawa_uczestnicy_wpisy (id),
  numer TEXT NOT NULL CHECK (numer <> ''),
  -- Czyj jest rachunek według dokumentu albo wiadomości.
  posiadacz TEXT NOT NULL,
  cel TEXT NOT NULL CHECK (cel IN ('zaplata_ceny', 'zadatek', 'oplata_rezerwacyjna', 'kaucja', 'zwrot_kaucji', 'czynsz', 'wyplata_dla_wlasciciela', 'splata_hipoteki', 'wyplata_w_kanale', 'inny')),
  -- Skąd numer: dokument z listy albo wiadomość z wpływów.
  zrodlo_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  zrodlo_wplyw_id TEXT REFERENCES sprawa_korespondencja_wplywy (id),
  wpisano TEXT NOT NULL,
  -- Kto, kiedy i jak potwierdził posiadacza innym kanałem, np. „telefonicznie pod numerem z umowy
  -- przedwstępnej”.
  potwierdzil TEXT,
  potwierdzono TEXT,
  potwierdzenie_sposob TEXT,
  zastapiony_przez_id TEXT REFERENCES sprawa_rachunki_wpisy (id),
  zastapil TEXT,
  zastapiono TEXT,

  CONSTRAINT rachunek_ma_zrodlo CHECK ((zrodlo_dokument_id IS NULL) <> (zrodlo_wplyw_id IS NULL)),
  CONSTRAINT potwierdzenie_w_komplecie CHECK (
    (potwierdzil IS NULL) = (potwierdzono IS NULL)
    AND (potwierdzil IS NULL) = (potwierdzenie_sposob IS NULL)
  ),
  CONSTRAINT zastapienie_ma_autora_i_date CHECK (
    (zastapiony_przez_id IS NULL) = (zastapil IS NULL)
    AND (zastapiony_przez_id IS NULL) = (zastapiono IS NULL)
  ),
  CONSTRAINT nie_zastepuje_samego_siebie CHECK (zastapiony_przez_id IS NOT id)
) STRICT;

-- Rachunki, na które wolno płacić (R4): niezastąpione i dopuszczone zatwierdzoną decyzją
-- dopuszczenie_rachunku. To jedyne miejsce tej reguły; korzystają z niego wypłaty rdzenia
-- i płatności typów spraw. Sprawdzenie i potwierdzenie posiadacza wymaga już sama zgoda na rachunek.
CREATE VIEW sprawa_rachunki_dopuszczone AS
SELECT rachunek.*
FROM sprawa_rachunki_wpisy AS rachunek
WHERE rachunek.zastapiony_przez_id IS NULL
  AND EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS dopuszczenie
    WHERE dopuszczenie.rodzaj = 'dopuszczenie_rachunku'
      AND dopuszczenie.dotyczy_tabela = 'sprawa_rachunki_wpisy'
      AND dopuszczenie.dotyczy_id = rachunek.id
  );

CREATE TRIGGER sprawa_rachunki_dane_rachunku_niezmienne
BEFORE UPDATE OF uczestnik_id, numer, posiadacz, cel, zrodlo_dokument_id, zrodlo_wplyw_id ON sprawa_rachunki_wpisy
WHEN NEW.uczestnik_id IS NOT OLD.uczestnik_id
  OR NEW.numer IS NOT OLD.numer
  OR NEW.posiadacz IS NOT OLD.posiadacz
  OR NEW.cel IS NOT OLD.cel
  OR NEW.zrodlo_dokument_id IS NOT OLD.zrodlo_dokument_id
  OR NEW.zrodlo_wplyw_id IS NOT OLD.zrodlo_wplyw_id
BEGIN
  SELECT RAISE(ABORT, 'numer, posiadacz, uczestnik, cel i źródło rachunku się nie zmieniają; nowy numer to nowy wiersz');
END;

CREATE TRIGGER sprawa_rachunki_zastapienie_ostateczne
BEFORE UPDATE OF zastapiony_przez_id, zastapil, zastapiono ON sprawa_rachunki_wpisy
WHEN OLD.zastapiony_przez_id IS NOT NULL
  AND (
    NEW.zastapiony_przez_id IS NOT OLD.zastapiony_przez_id
    OR NEW.zastapil IS NOT OLD.zastapil
    OR NEW.zastapiono IS NOT OLD.zastapiono
  )
BEGIN
  SELECT RAISE(ABORT, 'zastąpionego rachunku się nie przywraca ani nie zastępuje drugi raz');
END;

-- Na potwierdzeniu posiadacza stoi zgoda na rachunek, więc potwierdzenie wpisuje się raz.
-- Nowe potwierdzenie to nowy rachunek.
CREATE TRIGGER sprawa_rachunki_potwierdzenie_raz
BEFORE UPDATE OF potwierdzil, potwierdzono, potwierdzenie_sposob ON sprawa_rachunki_wpisy
WHEN OLD.potwierdzil IS NOT NULL
  AND (
    NEW.potwierdzil IS NOT OLD.potwierdzil
    OR NEW.potwierdzono IS NOT OLD.potwierdzono
    OR NEW.potwierdzenie_sposob IS NOT OLD.potwierdzenie_sposob
  )
BEGIN
  SELECT RAISE(ABORT, 'potwierdzenie posiadacza rachunku wpisuje się raz i się go nie zmienia');
END;

-- Nowy numer zastępuje stary tylko u tego samego uczestnika i w tym samym celu: inaczej
-- zastąpienie podmieniłoby rachunek innej osoby albo innej płatności (R4).
CREATE TRIGGER sprawa_rachunki_zastepca_tego_samego_uczestnika_i_celu
BEFORE UPDATE OF zastapiony_przez_id ON sprawa_rachunki_wpisy
WHEN NEW.zastapiony_przez_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_rachunki_wpisy AS zastepca
    WHERE zastepca.id = NEW.zastapiony_przez_id
      AND zastepca.uczestnik_id = NEW.uczestnik_id
      AND zastepca.cel = NEW.cel
  )
BEGIN
  SELECT RAISE(ABORT, 'rachunek zastępujący ma tego samego uczestnika i ten sam cel');
END;

CREATE TRIGGER sprawa_rachunki_wpisany_zastepca_tego_samego_uczestnika_i_celu
BEFORE INSERT ON sprawa_rachunki_wpisy
WHEN NEW.zastapiony_przez_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_rachunki_wpisy AS zastepca
    WHERE zastepca.id = NEW.zastapiony_przez_id
      AND zastepca.uczestnik_id = NEW.uczestnik_id
      AND zastepca.cel = NEW.cel
  )
BEGIN
  SELECT RAISE(ABORT, 'rachunek zastępujący ma tego samego uczestnika i ten sam cel');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_rachunki_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_rachunki_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_rachunki_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_rachunki_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_rachunki_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
