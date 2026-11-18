-- Analiza kredytowa klienta: dane wejściowe, wyniki programu, bank wybrany przez klienta
-- i harmonogram do wypłaty kredytu. Pisze Analityk kredytowy. Kwoty w groszach.
--
-- Analiza jest orientacyjna: nie jest ofertą ani rekomendacją kredytu i mówi to wprost (R2).
-- Zdolność liczy program biura, a Analityk dobiera dane i tłumaczy wynik; wyniku nie poprawia.
-- Do programu idą tylko dane z dokumentów, a deklaracja klienta zostaje w teczce jako dana
-- niepotwierdzona. Analiza poza standardem działu (np. inny okres kredytowania) wymaga
-- zatwierdzonej decyzji „inna”, która ją wskazuje. Nowa analiza odbiera aktualność poprzednim.
--
-- Listę dokumentów dla banku z ważnością trzyma rdzeń: dokumenty z wymaga = 'bank'. Banku nie
-- poleca żaden agent (R2); bank wybiera klient, a teczka zapisuje, skąd to wiadomo.

-- Dana wejściowa: dochód, zobowiązanie, gospodarstwo, wkład własny, cena albo okres kredytowania,
-- z dokumentem albo jako deklaracja klienta.
CREATE TABLE poszukiwanie_kredyt_dane (
  id TEXT PRIMARY KEY,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('dochod', 'zobowiazanie', 'gospodarstwo', 'wklad_wlasny', 'cena', 'okres')),
  -- Np. „umowa o pracę, netto” albo „rata za samochód”.
  opis TEXT NOT NULL,
  -- Kwota w groszach (dochód i zobowiązanie miesięcznie), liczba osób gospodarstwa albo liczba lat
  -- okresu kredytowania.
  wartosc INTEGER NOT NULL CHECK (wartosc >= 0),
  -- Okres, z którego wzięto kwotę, np. „średnia z 6 miesięcy”.
  okres_zrodla TEXT,
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  deklaracja INTEGER NOT NULL CHECK (deklaracja IN (0, 1)),
  wpisano TEXT NOT NULL,

  CONSTRAINT dana_z_dokumentu_albo_deklaracja CHECK ((deklaracja = 1) = (dokument_id IS NULL))
) STRICT;

-- Analiza z wersją parametrów programu, okresem, wynikiem i wyjaśnieniem dla klienta.
CREATE TABLE poszukiwanie_kredyt_analizy (
  id TEXT PRIMARY KEY,
  -- Wersja parametrów programu ze standardu działu, którą zatwierdził człowiek.
  wersja_parametrow TEXT NOT NULL,
  okres_lat INTEGER NOT NULL CHECK (okres_lat > 0),
  poza_standardem INTEGER NOT NULL CHECK (poza_standardem IN (0, 1)),
  poza_standardem_decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),
  -- Orientacyjny zakres kwoty kredytu i raty oraz udział wkładu własnego w cenie w setnych
  -- procenta (18 % to 1800); wszystko z programu.
  kwota_od INTEGER NOT NULL CHECK (kwota_od >= 0),
  kwota_do INTEGER NOT NULL CHECK (kwota_do >= 0),
  rata_od INTEGER NOT NULL CHECK (rata_od >= 0),
  rata_do INTEGER NOT NULL CHECK (rata_do >= 0),
  udzial_wkladu INTEGER NOT NULL CHECK (udzial_wkladu BETWEEN 0 AND 10000),
  wynik_sciezka TEXT NOT NULL,
  wynik_skrot TEXT NOT NULL,
  wyjasnienie TEXT NOT NULL,
  -- Zdanie, że analiza nie jest ofertą ani rekomendacją kredytu (R2).
  zastrzezenie TEXT NOT NULL CHECK (zastrzezenie <> ''),
  aktualna INTEGER NOT NULL CHECK (aktualna IN (0, 1)),
  utworzono TEXT NOT NULL,

  CONSTRAINT poza_standardem_z_decyzja CHECK ((poza_standardem = 1) = (poza_standardem_decyzja_id IS NOT NULL)),
  CONSTRAINT kwota_od_do CHECK (kwota_od <= kwota_do),
  CONSTRAINT rata_od_do CHECK (rata_od <= rata_do)
) STRICT;

-- Dane, które Analityk wprowadził do programu w tej analizie.
CREATE TABLE poszukiwanie_kredyt_analizy_dane (
  -- Klucz nadany przez bazę jest dodatni; zob. poszukiwanie_kredyt_analizy_dane_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  analiza_id TEXT NOT NULL REFERENCES poszukiwanie_kredyt_analizy (id),
  dana_id TEXT NOT NULL REFERENCES poszukiwanie_kredyt_dane (id),

  UNIQUE (analiza_id, dana_id)
) STRICT;

-- Bank, który wybrał klient, z wpływem, z którego to wiadomo, i jego wymaganiami wobec
-- dokumentów ze źródłem i datą.
CREATE TABLE poszukiwanie_kredyt_banki (
  -- Klucz nadany przez bazę jest dodatni; zob. poszukiwanie_kredyt_banki_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  bank TEXT NOT NULL,
  wybrano TEXT NOT NULL,
  zrodlo_wplyw_id TEXT NOT NULL REFERENCES sprawa_korespondencja_wplywy (id),
  -- Np. strona banku albo pismo banku do klienta.
  wymagania_zrodlo TEXT,
  wymagania_data TEXT,

  CONSTRAINT wymagania_ze_zrodlem_i_data CHECK ((wymagania_zrodlo IS NULL) = (wymagania_data IS NULL))
) STRICT;

-- Droga do wypłaty kredytu: data przewidywana z programu i data rzeczywista po fakcie.
CREATE TABLE poszukiwanie_kredyt_harmonogram (
  -- Klucz nadany przez bazę jest dodatni; zob. poszukiwanie_kredyt_harmonogram_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  krok TEXT NOT NULL CHECK (krok IN ('wniosek', 'wycena', 'decyzja_kredytowa', 'umowa_kredytu', 'wyplata')),
  przewidywana TEXT NOT NULL,
  rzeczywista TEXT
) STRICT;

-- Dana z dokumentu wskazuje dokument z plikiem.
CREATE TRIGGER poszukiwanie_kredyt_dana_z_plikiem
BEFORE INSERT ON poszukiwanie_kredyt_dane
WHEN NEW.dokument_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_dokumenty_wpisy
    WHERE id = NEW.dokument_id
      AND stan IN ('otrzymany', 'sprawdzony')
  )
BEGIN
  SELECT RAISE(ABORT, 'dana wejściowa wskazuje dokument z plikiem: otrzymany albo sprawdzony');
END;

CREATE TRIGGER poszukiwanie_kredyt_dane_bez_zmian
BEFORE UPDATE ON poszukiwanie_kredyt_dane
BEGIN
  SELECT RAISE(ABORT, 'danej wejściowej się nie zmienia; nowa wartość to nowy wpis');
END;

-- Analiza poza standardem stoi na zatwierdzonej decyzji „inna”, która ją wskazuje.
CREATE TRIGGER poszukiwanie_kredyt_poza_standardem_za_zgoda
BEFORE INSERT ON poszukiwanie_kredyt_analizy
WHEN NEW.poza_standardem = 1
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS decyzja
    WHERE decyzja.id = NEW.poza_standardem_decyzja_id
      AND decyzja.rodzaj = 'inna'
      AND decyzja.dotyczy_tabela = 'poszukiwanie_kredyt_analizy'
      AND decyzja.dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'analiza poza standardem wymaga zatwierdzonej decyzji inna, która ją wskazuje');
END;

-- Aktualność odbiera tylko nowa analiza, więc analiza powstaje aktualna.
CREATE TRIGGER poszukiwanie_kredyt_analiza_powstaje_aktualna
BEFORE INSERT ON poszukiwanie_kredyt_analizy
WHEN NEW.aktualna <> 1
BEGIN
  SELECT RAISE(ABORT, 'analiza powstaje jako aktualna');
END;

CREATE TRIGGER poszukiwanie_kredyt_nowa_analiza_zeruje_poprzednie
AFTER INSERT ON poszukiwanie_kredyt_analizy
BEGIN
  UPDATE poszukiwanie_kredyt_analizy
  SET aktualna = 0
  WHERE id <> NEW.id
    AND aktualna = 1;
END;

-- Wynik pochodzi z programu i Analityk go nie poprawia; jedyną zmianą jest utrata aktualności.
CREATE TRIGGER poszukiwanie_kredyt_analizy_bez_zmian
BEFORE UPDATE ON poszukiwanie_kredyt_analizy
WHEN NEW.id IS NOT OLD.id
  OR NEW.wersja_parametrow IS NOT OLD.wersja_parametrow
  OR NEW.okres_lat IS NOT OLD.okres_lat
  OR NEW.poza_standardem IS NOT OLD.poza_standardem
  OR NEW.poza_standardem_decyzja_id IS NOT OLD.poza_standardem_decyzja_id
  OR NEW.kwota_od IS NOT OLD.kwota_od
  OR NEW.kwota_do IS NOT OLD.kwota_do
  OR NEW.rata_od IS NOT OLD.rata_od
  OR NEW.rata_do IS NOT OLD.rata_do
  OR NEW.udzial_wkladu IS NOT OLD.udzial_wkladu
  OR NEW.wynik_sciezka IS NOT OLD.wynik_sciezka
  OR NEW.wynik_skrot IS NOT OLD.wynik_skrot
  OR NEW.wyjasnienie IS NOT OLD.wyjasnienie
  OR NEW.zastrzezenie IS NOT OLD.zastrzezenie
  OR NEW.utworzono IS NOT OLD.utworzono
  OR (OLD.aktualna = 0 AND NEW.aktualna = 1)
BEGIN
  SELECT RAISE(ABORT, 'wyniku analizy się nie zmienia; nowe obliczenie to nowa analiza');
END;

-- Do programu idzie wartość z dokumentu, a nie z deklaracji, i tylko dana wpisana przed analizą.
CREATE TRIGGER poszukiwanie_kredyt_do_programu_dane_z_dokumentow
BEFORE INSERT ON poszukiwanie_kredyt_analizy_dane
WHEN NOT EXISTS (
  SELECT 1
  FROM poszukiwanie_kredyt_dane AS dana
  JOIN poszukiwanie_kredyt_analizy AS analiza ON analiza.id = NEW.analiza_id
  WHERE dana.id = NEW.dana_id
    AND dana.deklaracja = 0
    AND dana.wpisano <= analiza.utworzono
)
BEGIN
  SELECT RAISE(ABORT, 'do programu idą tylko dane z dokumentów wpisane przed analizą, nie deklaracje klienta');
END;

CREATE TRIGGER poszukiwanie_kredyt_analizy_dane_bez_zmian
BEFORE UPDATE ON poszukiwanie_kredyt_analizy_dane
BEGIN
  SELECT RAISE(ABORT, 'danych analizy się nie zmienia; inne dane to nowa analiza');
END;

-- Wybór banku jest zapisem tego, co powiedział klient; wymagania banku wpisuje się raz.
CREATE TRIGGER poszukiwanie_kredyt_banki_wymagania_raz
BEFORE UPDATE ON poszukiwanie_kredyt_banki
WHEN NEW.id IS NOT OLD.id
  OR NEW.bank IS NOT OLD.bank
  OR NEW.wybrano IS NOT OLD.wybrano
  OR NEW.zrodlo_wplyw_id IS NOT OLD.zrodlo_wplyw_id
  OR (OLD.wymagania_zrodlo IS NOT NULL AND NEW.wymagania_zrodlo IS NOT OLD.wymagania_zrodlo)
  OR (OLD.wymagania_data IS NOT NULL AND NEW.wymagania_data IS NOT OLD.wymagania_data)
BEGIN
  SELECT RAISE(ABORT, 'wyboru banku się nie zmienia; wymagania banku wpisuje się raz');
END;

-- Datę przewidywaną program przelicza, dopóki krok się nie odbył; datę rzeczywistą wpisuje się
-- raz, a potem kroku się nie zmienia.
CREATE TRIGGER poszukiwanie_kredyt_harmonogram_po_fakcie_bez_zmian
BEFORE UPDATE ON poszukiwanie_kredyt_harmonogram
WHEN OLD.rzeczywista IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.krok IS NOT OLD.krok
BEGIN
  SELECT RAISE(ABORT, 'kroku harmonogramu po fakcie się nie zmienia; przewidywaną datę przelicza się do chwili kroku');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER poszukiwanie_kredyt_dane_bez_zastepowania
BEFORE INSERT ON poszukiwanie_kredyt_dane
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_kredyt_dane
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_kredyt_dane_bez_usuwania
BEFORE DELETE ON poszukiwanie_kredyt_dane
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER poszukiwanie_kredyt_analizy_bez_zastepowania
BEFORE INSERT ON poszukiwanie_kredyt_analizy
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_kredyt_analizy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_kredyt_analizy_bez_usuwania
BEFORE DELETE ON poszukiwanie_kredyt_analizy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER poszukiwanie_kredyt_analizy_dane_bez_zastepowania
BEFORE INSERT ON poszukiwanie_kredyt_analizy_dane
WHEN EXISTS (
    SELECT 1
    FROM poszukiwanie_kredyt_analizy_dane
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM poszukiwanie_kredyt_analizy_dane
    WHERE analiza_id = NEW.analiza_id
      AND dana_id = NEW.dana_id
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_kredyt_analizy_dane_bez_usuwania
BEFORE DELETE ON poszukiwanie_kredyt_analizy_dane
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER poszukiwanie_kredyt_banki_bez_zastepowania
BEFORE INSERT ON poszukiwanie_kredyt_banki
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_kredyt_banki
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_kredyt_banki_bez_usuwania
BEFORE DELETE ON poszukiwanie_kredyt_banki
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER poszukiwanie_kredyt_harmonogram_bez_zastepowania
BEFORE INSERT ON poszukiwanie_kredyt_harmonogram
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_kredyt_harmonogram
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_kredyt_harmonogram_bez_usuwania
BEFORE DELETE ON poszukiwanie_kredyt_harmonogram
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
