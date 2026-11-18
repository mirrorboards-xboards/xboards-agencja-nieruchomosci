-- Klienci biura z Kontrolerem, ich stosunki gospodarcze i kwalifikacja każdej sprawy: czy jest
-- objęta ustawą o przeciwdziałaniu praniu pieniędzy. Wpis stosunków powstaje przy weryfikacji
-- przed pierwszą umową; klient, który wraca po zakończeniu stosunków, dostaje nowy wpis, a stary
-- zostaje razem z jego terminem przechowywania.
--
-- Pisze Kierownik kontroli z raportów Kontrolerów. Wpisu się nie zmienia; jedyne zmiany to
-- jednorazowe uzupełnienie odnośnika do kartoteki, dnia nawiązania i dnia zakończenia stosunków.
-- Kwalifikacji sprawy się nie zmienia: nowa kwalifikacja tej samej teczki to nowy wiersz.
CREATE TABLE rejestr_kontroli_klienci_wpisy (
  id TEXT PRIMARY KEY,
  -- Imię i nazwisko albo nazwa klienta; dane identyfikacyjne są w rodzaju sprawdzenia.
  klient TEXT NOT NULL CHECK (klient <> ''),
  -- Wpis w kartotece klientów (klienci_kartoteka_wpisy). Kartoteka dostaje klienta dopiero
  -- w dniu podpisania umowy, a rejestr przy weryfikacji przed nią, więc odnośnik przychodzi później.
  kartoteka_odnosnik TEXT,
  -- Kontroler, który sprawdza klienta (R7).
  kontroler TEXT NOT NULL CHECK (kontroler <> ''),
  -- Dzień podpisania pierwszej umowy z biurem i dzień zakończenia stosunków z raportu końcowego
  -- Kontrolera; od niego program liczy 5 lat przechowywania (art. 49 ust. 1 AML).
  stosunki_od TEXT,
  stosunki_do TEXT,
  -- Raport Kontrolera, z którego wpis pochodzi.
  zrodlo TEXT NOT NULL CHECK (zrodlo <> ''),
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,

  CONSTRAINT stosunki_konczy_sie_po_nawiazaniu CHECK (
    stosunki_do IS NULL OR (stosunki_od IS NOT NULL AND stosunki_do >= stosunki_od)
  )
) STRICT;

-- Kwalifikacja sprawy klienta. Przy samej umowie o zarządzanie wynik brzmi „do rozstrzygnięcia”,
-- dopóki biuro tego nie ustali; baza tego nie wymusza.
CREATE TABLE rejestr_kontroli_klienci_sprawy (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_klienci_sprawy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  klient_id TEXT NOT NULL REFERENCES rejestr_kontroli_klienci_wpisy (id),
  -- Teczka klienta; odnośnik nie daje do niej dostępu.
  teczka_odnosnik TEXT NOT NULL CHECK (teczka_odnosnik <> ''),
  rodzaj_sprawy TEXT NOT NULL CHECK (rodzaj_sprawy IN ('sprzedaz_albo_kupno', 'posrednictwo_najmu', 'zarzadzanie')),
  objeta TEXT NOT NULL CHECK (objeta IN ('tak', 'nie', 'do_rozstrzygniecia')),
  -- Dlaczego tak, np. „art. 2 ust. 1 pkt 18 AML, czynsz poniżej progu”.
  podstawa TEXT NOT NULL CHECK (podstawa <> ''),
  -- Czynsz miesięczny w groszach, średni kurs euro NBP w złotych i dzień kursu (art. 5 AML).
  czynsz INTEGER CHECK (czynsz > 0),
  kurs_euro REAL CHECK (kurs_euro > 0),
  dzien_kursu TEXT,
  zakwalifikowano TEXT NOT NULL,
  wpisal TEXT NOT NULL,

  CONSTRAINT czynsz_z_kursem_i_dniem CHECK (
    (czynsz IS NULL) = (kurs_euro IS NULL)
    AND (czynsz IS NULL) = (dzien_kursu IS NULL)
  ),
  CONSTRAINT najem_ma_czynsz CHECK (rodzaj_sprawy <> 'posrednictwo_najmu' OR czynsz IS NOT NULL),
  -- Art. 2 ust. 1 pkt 18 AML: pośrednictwo przy najmie z czynszem niższym niż równowartość
  -- 10 000 euro nie jest objęte ustawą, a z wyższym jest. 10 000 euro to kurs × 1 000 000 groszy.
  CONSTRAINT prog_najmu CHECK (
    rodzaj_sprawy <> 'posrednictwo_najmu'
    OR objeta = CASE
      WHEN czynsz < kurs_euro * 1000000 THEN 'nie'
      ELSE 'tak'
    END
  )
) STRICT;

-- Klient, który wraca, nawiązuje stosunki na nowo, ale dopiero po zakończeniu poprzednich.
CREATE TRIGGER rejestr_kontroli_klienci_jedne_trwajace_stosunki
BEFORE INSERT ON rejestr_kontroli_klienci_wpisy
WHEN NEW.kartoteka_odnosnik IS NOT NULL
  AND EXISTS (
    SELECT 1
    FROM rejestr_kontroli_klienci_wpisy
    WHERE kartoteka_odnosnik = NEW.kartoteka_odnosnik
      AND stosunki_do IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'klient ma już trwające stosunki; nowe stosunki wpisuje się po zakończeniu poprzednich');
END;

CREATE TRIGGER rejestr_kontroli_klienci_jedne_trwajace_stosunki_przy_odnosniku
BEFORE UPDATE OF kartoteka_odnosnik ON rejestr_kontroli_klienci_wpisy
WHEN NEW.kartoteka_odnosnik IS NOT NULL
  AND EXISTS (
    SELECT 1
    FROM rejestr_kontroli_klienci_wpisy
    WHERE kartoteka_odnosnik = NEW.kartoteka_odnosnik
      AND id <> NEW.id
      AND stosunki_do IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'klient ma już trwające stosunki; nowe stosunki wpisuje się po zakończeniu poprzednich');
END;

-- Organ kontroli ma zobaczyć, co biuro wiedziało w danym dniu, więc wpisu się nie poprawia.
-- Odnośnik do kartoteki i daty stosunków wpisuje się raz, gdy są znane.
CREATE TRIGGER rejestr_kontroli_klienci_wpisy_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_klienci_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.klient IS NOT OLD.klient
  OR NEW.kontroler IS NOT OLD.kontroler
  OR NEW.zrodlo IS NOT OLD.zrodlo
  OR NEW.wpisal IS NOT OLD.wpisal
  OR NEW.wpisano IS NOT OLD.wpisano
  OR (OLD.kartoteka_odnosnik IS NOT NULL AND NEW.kartoteka_odnosnik IS NOT OLD.kartoteka_odnosnik)
  OR (OLD.stosunki_od IS NOT NULL AND NEW.stosunki_od IS NOT OLD.stosunki_od)
  OR (OLD.stosunki_do IS NOT NULL AND NEW.stosunki_do IS NOT OLD.stosunki_do)
BEGIN
  SELECT RAISE(ABORT, 'wpisu klienta się nie zmienia; odnośnik do kartoteki i daty stosunków wpisuje się raz');
END;

CREATE TRIGGER rejestr_kontroli_klienci_sprawy_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_klienci_sprawy
BEGIN
  SELECT RAISE(ABORT, 'kwalifikacji sprawy się nie zmienia; nowa kwalifikacja to nowy wiersz');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabel z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER rejestr_kontroli_klienci_wpisy_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_klienci_wpisy
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_klienci_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_klienci_sprawy_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_klienci_sprawy
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_klienci_sprawy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

-- Art. 49 AML i R9: dokumentację przechowuje się co najmniej 5 lat, a na razie bez terminu.
CREATE TRIGGER rejestr_kontroli_klienci_wpisy_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_klienci_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_klienci_sprawy_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_klienci_sprawy
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;
