-- Dokumenty teczki, od wymaganego do sprawdzonego, i źródła danych. Typ sprawy mówi, jakie
-- dokumenty są w nim potrzebne. Plik leży w repozytorium XGit pokoju, a tabela trzyma jego ścieżkę
-- i skrót treści, który obejmuje podpis decyzji. Dokumenty prowadzi agent sprawy.
--
-- Skan dowodu osobistego zostaje w teczce jak każdy inny plik. Kopię skanu jako dokumentację
-- środków bezpieczeństwa finansowego Kontroler dołącza do raportu, a Kierownik kontroli przenosi
-- ją do rejestru kontroli; w teczce nikt pliku nie usuwa ani nie czyści.
--
-- Wniosek biura o dokument w imieniu strony to wysyłka z zamawia_dokument_id i umocowaniem
-- (sprawa-korespondencja).
--
-- Dokument, na którym stoi zawarta umowa, zgoda albo podpisany protokół, nie zmienia rodzaju ani
-- tego, czego dotyczy, i nie staje się odrzucony ani zbędny. Te reguły leżą w migracjach
-- sprawa-umowy (sprawa_umowy_dokument_zawartej_umowy_zostaje), sprawa-zgody
-- (sprawa_zgody_dokument_zgody_zostaje) i sprawa-protokoly
-- (sprawa_protokoly_dokument_podpisanego_zostaje).
CREATE TABLE sprawa_dokumenty_wpisy (
  id TEXT PRIMARY KEY,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('tozsamosc', 'odpis_ksiegi_wieczystej', 'odpis_rejestru', 'zaswiadczenie', 'akt_notarialny', 'umowa', 'umowa_z_biurem', 'pelnomocnictwo', 'polisa', 'oswiadczenie', 'protokol', 'akt_stanu_cywilnego', 'decyzja_kredytowa', 'swiadectwo_energetyczne', 'potwierdzenie_bankowe', 'regulamin', 'standard', 'instrukcja', 'uchwala', 'pismo', 'inny')),
  nazwa TEXT NOT NULL,
  -- Kogo albo czego dokument dotyczy: uczestnika albo nieruchomości.
  dotyczy_uczestnik_id TEXT REFERENCES sprawa_uczestnicy_wpisy (id),
  dotyczy_nieruchomosc_id TEXT REFERENCES sprawa_nieruchomosci_wpisy (id),
  wystawca TEXT,
  data_wystawienia TEXT,
  waznosc_do TEXT,
  -- Skąd wiadomo, do kiedy dokument jest ważny: od wystawcy, z przepisu albo od instytucji, która
  -- go wymaga (np. notariusz przyjmuje zaświadczenie spółdzielni przez 30 dni).
  waznosc_zrodlo TEXT CHECK (waznosc_zrodlo IN ('wystawca', 'przepis', 'instytucja')),
  -- Kto wymaga dokumentu. Agent nie dokłada do listy własnych wymagań.
  wymaga TEXT CHECK (wymaga IN ('przepis', 'notariusz', 'bank', 'umowa')),
  -- Od kogo dokument przyszedł, np. „od klientki”, „pobrany z centralnej bazy ksiąg wieczystych”.
  pochodzenie TEXT,
  kanal TEXT CHECK (kanal IN ('email', 'sms', 'portal', 'kanal_sprzedazy', 'list', 'pokoj_wymiany', 'osobiscie', 'rejestr_elektroniczny')),
  postac TEXT CHECK (postac IN ('oryginal', 'kopia', 'skan', 'odpis', 'wydruk')),
  stan TEXT NOT NULL CHECK (stan IN ('wymagany', 'zamowiony', 'otrzymany', 'sprawdzony', 'odrzucony', 'zbedny')),
  powod_odrzucenia TEXT,
  sciezka TEXT,
  skrot TEXT,

  CONSTRAINT dotyczy_uczestnika_albo_nieruchomosci CHECK (
    dotyczy_uczestnik_id IS NULL OR dotyczy_nieruchomosc_id IS NULL
  ),
  CONSTRAINT plik_ze_skrotem CHECK ((sciezka IS NULL) = (skrot IS NULL)),
  CONSTRAINT otrzymany_ma_plik CHECK (stan NOT IN ('otrzymany', 'sprawdzony') OR sciezka IS NOT NULL),
  CONSTRAINT odrzucony_ma_powod CHECK ((stan = 'odrzucony') = (powod_odrzucenia IS NOT NULL)),
  CONSTRAINT waznosc_ze_zrodlem CHECK (waznosc_do IS NULL OR waznosc_zrodlo IS NOT NULL),
  CONSTRAINT waznosc_nie_przed_wystawieniem CHECK (
    waznosc_do IS NULL OR data_wystawienia IS NULL OR waznosc_do >= data_wystawienia
  )
) STRICT;

-- Źródło jednej danej teczki (zasada „Każda wartość ma źródło”): pole wiersza rdzenia albo typu
-- sprawy i dokument, z którego pochodzi, albo osoba, od której biuro je ma, np. „od klienta,
-- niepotwierdzone”. Służy też typom spraw.
CREATE TABLE sprawa_dokumenty_zrodla (
  -- Klucz nadany przez bazę jest dodatni; zob. sprawa_dokumenty_zrodla_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Wiersz i pole, bez klucza obcego, bo może to być tabela typu sprawy.
  dotyczy_tabela TEXT NOT NULL,
  dotyczy_id TEXT NOT NULL,
  pole TEXT NOT NULL,
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  od_uczestnik_id TEXT REFERENCES sprawa_uczestnicy_wpisy (id),
  -- Osoba spoza teczki, od której dana pochodzi, np. „zarządca budynku, telefonicznie”.
  od_kogo TEXT,
  potwierdzone INTEGER NOT NULL CHECK (potwierdzone IN (0, 1)),
  wpisano TEXT NOT NULL,

  CONSTRAINT zrodlo_wskazuje_dokument_albo_osobe CHECK (
    (dokument_id IS NOT NULL) + (od_uczestnik_id IS NOT NULL) + (od_kogo IS NOT NULL) = 1
  )
) STRICT;

-- Wymagany, zamówiony albo otrzymany: reguły dalszych stanów sprawdzają zmianę.
CREATE TRIGGER sprawa_dokumenty_powstaje_w_stanie_poczatkowym
BEFORE INSERT ON sprawa_dokumenty_wpisy
WHEN NEW.stan NOT IN ('wymagany', 'zamowiony', 'otrzymany')
BEGIN
  SELECT RAISE(ABORT, 'dokument powstaje jako wymagany, zamówiony albo otrzymany');
END;

-- Na plik powołują się podpisy decyzji, wysyłki i źródła, więc plik się nie zmienia; nowa kopia
-- dokumentu to nowy wiersz.
CREATE TRIGGER sprawa_dokumenty_plik_niezmienny
BEFORE UPDATE OF sciezka, skrot ON sprawa_dokumenty_wpisy
WHEN OLD.sciezka IS NOT NULL
  AND (NEW.sciezka IS NOT OLD.sciezka OR NEW.skrot IS NOT OLD.skrot)
BEGIN
  SELECT RAISE(ABORT, 'plik dokumentu się nie zmienia; nowa kopia to nowy wiersz');
END;

CREATE TRIGGER sprawa_dokumenty_zbedny_tylko_z_wymaganego
BEFORE UPDATE OF stan ON sprawa_dokumenty_wpisy
WHEN NEW.stan = 'zbedny' AND OLD.stan NOT IN ('wymagany', 'zamowiony', 'zbedny')
BEGIN
  SELECT RAISE(ABORT, 'zbędny staje się tylko dokument wymagany albo zamówiony');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_dokumenty_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_dokumenty_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_dokumenty_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_dokumenty_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_dokumenty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER sprawa_dokumenty_zrodla_bez_zastepowania
BEFORE INSERT ON sprawa_dokumenty_zrodla
WHEN EXISTS (
  SELECT 1
  FROM sprawa_dokumenty_zrodla
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_dokumenty_zrodla_bez_usuwania
BEFORE DELETE ON sprawa_dokumenty_zrodla
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
