-- Pełnomocnictwa i reprezentacja podmiotów w jednej tabeli: kto, komu, do czego i w jakiej formie.
-- Agent występuje w imieniu klienta tylko w granicach pełnomocnictwa, które klient dał biuru
-- (zasada „W imieniu klienta tylko z pełnomocnictwem”). Role „pełnomocnik” i „reprezentant”
-- wynikają z tej tabeli, więc nie ma ich w sprawa_uczestnicy_role.
--
-- Umocowania wpisuje agent sprawy, a sprawdza je Kontroler. Wersja umocowania rośnie przy każdej
-- zmianie; sprawdzenie dotyczy jednej wersji, a trigger, który zeruje jego aktualność, leży
-- w migracji sprawa-sprawdzenia (sprawa_sprawdzenia_nowa_wersja_umocowania).
--
-- Czy czynność mieści się w zakresie, rozstrzyga lista sprawa_umocowania_czynnosci. Reguła
-- wysyłki w imieniu uczestnika leży w migracji sprawa-korespondencja
-- (sprawa_korespondencja_w_imieniu_uczestnika_z_umocowaniem).
CREATE TABLE sprawa_umocowania_wpisy (
  id TEXT PRIMARY KEY,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('pelnomocnictwo', 'reprezentacja')),
  mocodawca_uczestnik_id TEXT NOT NULL REFERENCES sprawa_uczestnicy_wpisy (id),
  -- Umocowany: uczestnik teczki albo samo biuro.
  umocowany_uczestnik_id TEXT REFERENCES sprawa_uczestnicy_wpisy (id),
  umocowane_biuro INTEGER NOT NULL DEFAULT 0 CHECK (umocowane_biuro IN (0, 1)),
  -- Art. 98 KC: ogólne obejmuje czynności zwykłego zarządu, rodzajowe określa ich rodzaj,
  -- a do czynności wskazuje tę czynność.
  zakres TEXT NOT NULL CHECK (zakres IN ('ogolne', 'rodzajowe', 'do_czynnosci')),
  -- Art. 99 § 1 KC: forma czynności, do której umocowuje; reprezentacja wynika z rejestru.
  forma TEXT CHECK (forma IN ('pisemna', 'elektroniczna', 'z_podpisem_poswiadczonym', 'akt_notarialny', 'z_rejestru')),
  -- Kto i w jakim składzie działa za podmiot, np. „dwóch członków zarządu łącznie”.
  sposob_reprezentacji TEXT,
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  udzielono TEXT,
  wazne_do TEXT,
  -- Art. 101 KC: odwołanie, śmierć albo upływ terminu, z datą i źródłem wiadomości.
  wygaslo TEXT,
  wygasniecie_powod TEXT CHECK (wygasniecie_powod IN ('odwolanie', 'smierc_mocodawcy', 'smierc_umocowanego', 'uplyw_terminu', 'wykreslenie_z_rejestru', 'inne')),
  wygasniecie_zrodlo TEXT,
  wersja INTEGER NOT NULL DEFAULT 1 CHECK (wersja >= 1),

  CONSTRAINT umocowany_osoba_albo_biuro CHECK ((umocowany_uczestnik_id IS NOT NULL) <> (umocowane_biuro = 1)),
  CONSTRAINT wygasniecie_ma_powod_i_zrodlo CHECK (
    (wygaslo IS NULL) = (wygasniecie_powod IS NULL)
    AND (wygaslo IS NULL) = (wygasniecie_zrodlo IS NULL)
  ),
  CONSTRAINT nie_umocowuje_siebie CHECK (umocowany_uczestnik_id IS NOT mocodawca_uczestnik_id)
) STRICT;

-- Czynności w zakresie umocowania, np. wystapienie_o_zaswiadczenie, uzgodnienie_terminu_aktu.
-- Nazwy ustala typ sprawy; ta sama nazwa wraca w wysyłce w imieniu uczestnika.
CREATE TABLE sprawa_umocowania_czynnosci (
  -- Klucz nadany przez bazę jest dodatni; zob. sprawa_umocowania_czynnosci_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  umocowanie_id TEXT NOT NULL REFERENCES sprawa_umocowania_wpisy (id),
  czynnosc TEXT NOT NULL CHECK (czynnosc <> '' AND czynnosc NOT GLOB '*[^a-z_]*'),

  UNIQUE (umocowanie_id, czynnosc)
) STRICT;

CREATE TRIGGER sprawa_umocowania_reprezentacja_tylko_podmiotu
BEFORE INSERT ON sprawa_umocowania_wpisy
WHEN NEW.rodzaj = 'reprezentacja'
  AND (
    SELECT rodzaj_podmiotu = 'osoba_fizyczna'
    FROM sprawa_uczestnicy_wpisy
    WHERE id = NEW.mocodawca_uczestnik_id
  )
BEGIN
  SELECT RAISE(ABORT, 'reprezentację ma tylko podmiot; osoba fizyczna udziela pełnomocnictwa');
END;

CREATE TRIGGER sprawa_umocowania_zmieniona_reprezentacja_tylko_podmiotu
BEFORE UPDATE OF rodzaj, mocodawca_uczestnik_id ON sprawa_umocowania_wpisy
WHEN NEW.rodzaj = 'reprezentacja'
  AND (
    SELECT rodzaj_podmiotu = 'osoba_fizyczna'
    FROM sprawa_uczestnicy_wpisy
    WHERE id = NEW.mocodawca_uczestnik_id
  )
BEGIN
  SELECT RAISE(ABORT, 'reprezentację ma tylko podmiot; osoba fizyczna udziela pełnomocnictwa');
END;

-- Data udzielenia mówi, od kiedy biuro mogło działać w imieniu mocodawcy; jej zmiana
-- przepisałaby podstawę wcześniejszych wysyłek.
CREATE TRIGGER sprawa_umocowania_udzielono_niezmienne
BEFORE UPDATE OF udzielono ON sprawa_umocowania_wpisy
WHEN NEW.udzielono IS NOT OLD.udzielono
BEGIN
  SELECT RAISE(ABORT, 'data udzielenia umocowania się nie zmienia; inne umocowanie to nowy wiersz');
END;

-- Wygasłe umocowanie nie odżywa (art. 101 KC): wygaśnięcie wpisuje się raz i się go nie zeruje.
-- Ponowne umocowanie to nowy wiersz.
CREATE TRIGGER sprawa_umocowania_wygasniecie_raz
BEFORE UPDATE OF wygaslo, wygasniecie_powod, wygasniecie_zrodlo ON sprawa_umocowania_wpisy
WHEN OLD.wygaslo IS NOT NULL
  AND (
    NEW.wygaslo IS NOT OLD.wygaslo
    OR NEW.wygasniecie_powod IS NOT OLD.wygasniecie_powod
    OR NEW.wygasniecie_zrodlo IS NOT OLD.wygasniecie_zrodlo
  )
BEGIN
  SELECT RAISE(ABORT, 'wygaśnięcie umocowania wpisuje się raz i się go nie cofa; ponowne umocowanie to nowy wiersz');
END;

CREATE TRIGGER sprawa_umocowania_zmiana_podnosi_wersje
AFTER UPDATE ON sprawa_umocowania_wpisy
WHEN NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.mocodawca_uczestnik_id IS NOT OLD.mocodawca_uczestnik_id
  OR NEW.umocowany_uczestnik_id IS NOT OLD.umocowany_uczestnik_id
  OR NEW.umocowane_biuro IS NOT OLD.umocowane_biuro
  OR NEW.zakres IS NOT OLD.zakres
  OR NEW.forma IS NOT OLD.forma
  OR NEW.sposob_reprezentacji IS NOT OLD.sposob_reprezentacji
  OR NEW.dokument_id IS NOT OLD.dokument_id
  OR NEW.udzielono IS NOT OLD.udzielono
  OR NEW.wazne_do IS NOT OLD.wazne_do
  OR NEW.wygaslo IS NOT OLD.wygaslo
  OR NEW.wygasniecie_powod IS NOT OLD.wygasniecie_powod
  OR NEW.wygasniecie_zrodlo IS NOT OLD.wygasniecie_zrodlo
BEGIN
  UPDATE sprawa_umocowania_wpisy
  SET wersja = wersja + 1
  WHERE id = NEW.id;
END;

CREATE TRIGGER sprawa_umocowania_nowa_czynnosc_podnosi_wersje
AFTER INSERT ON sprawa_umocowania_czynnosci
BEGIN
  UPDATE sprawa_umocowania_wpisy
  SET wersja = wersja + 1
  WHERE id = NEW.umocowanie_id;
END;

-- Zakres jest częścią sprawdzonego umocowania; nowa czynność to nowy wiersz.
CREATE TRIGGER sprawa_umocowania_czynnosci_bez_zmian
BEFORE UPDATE ON sprawa_umocowania_czynnosci
BEGIN
  SELECT RAISE(ABORT, 'czynności w zakresie umocowania się nie zmienia; nowa czynność to nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_umocowania_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_umocowania_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_umocowania_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_umocowania_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_umocowania_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER sprawa_umocowania_czynnosci_bez_zastepowania
BEFORE INSERT ON sprawa_umocowania_czynnosci
WHEN EXISTS (
    SELECT 1
    FROM sprawa_umocowania_czynnosci
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM sprawa_umocowania_czynnosci
    WHERE umocowanie_id = NEW.umocowanie_id
      AND czynnosc = NEW.czynnosc
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_umocowania_czynnosci_bez_usuwania
BEFORE DELETE ON sprawa_umocowania_czynnosci
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
