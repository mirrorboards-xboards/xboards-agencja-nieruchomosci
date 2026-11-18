//! The kind `najem-czynsz` of the room `cases/najem`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "najem-czynsz";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Pieniądze i media najmu: należności z harmonogramu, wpłaty, ich rozliczenie na należności, zmiany
-- czynszu, zaległości, odczyty liczników i dane o czynszu dla gminy. Kwoty, terminy, saldo
-- i zaległości liczy program; Opiekun najmu wpisuje dane z umowy i wpłaty. Kwoty w groszach.
--
-- Gdy czynsz wpływa przez biuro, biuro przekazuje wynajmującemu każdą wpłatę osobno (decyzja T13):
-- wypłata z rdzenia wskazuje jedną wpłatę (podstawa_tabela = najem_czynsz_wplaty).
--
-- Rachunek biura nie jest rachunkiem uczestnika teczki, a rdzeń zna tylko rachunki uczestników,
-- więc wpłata na rachunek biura nie wskazuje rachunku z rdzenia.

-- Należności z harmonogramu: czynsz, media i opłaty dla wspólnoty, każda z okresem i terminem.
CREATE TABLE najem_czynsz_naleznosci (
  id TEXT PRIMARY KEY,
  najem_id TEXT NOT NULL REFERENCES najem_umowy_najmu (id),
  tytul TEXT NOT NULL CHECK (tytul IN ('czynsz', 'media', 'wspolnota', 'inna')),
  -- Okres płatności, np. 2026-12.
  okres TEXT NOT NULL,
  kwota INTEGER NOT NULL CHECK (kwota > 0),
  termin TEXT NOT NULL
) STRICT;

-- Wpłaty z płacącym, rachunkiem i źródłem potwierdzenia. Wyciąg z rachunku biura albo potwierdzenie
-- od wynajmującego jest pewniejsze niż zrzut ekranu od najemcy.
CREATE TABLE najem_czynsz_wplaty (
  id TEXT PRIMARY KEY,
  najem_id TEXT NOT NULL REFERENCES najem_umowy_najmu (id),
  chwila TEXT NOT NULL,
  kwota INTEGER NOT NULL CHECK (kwota > 0),
  placacy_uczestnik_id TEXT REFERENCES sprawa_uczestnicy_wpisy (id),
  -- Gdzie wpłynęły pieniądze: na rachunek biura (przepływ przez biuro) albo wprost na rachunek
  -- wynajmującego, dopuszczony decyzją (R4).
  na_rachunek TEXT NOT NULL CHECK (na_rachunek IN ('biura', 'wynajmujacego')),
  rachunek_id TEXT REFERENCES sprawa_rachunki_wpisy (id),
  potwierdzenie TEXT NOT NULL CHECK (potwierdzenie IN ('wyciag_biura', 'od_wynajmujacego', 'od_najemcy')),
  dokument_id TEXT NOT NULL REFERENCES sprawa_dokumenty_wpisy (id),

  CONSTRAINT rachunek_wynajmujacego_wskazany CHECK ((na_rachunek = 'wynajmujacego') = (rachunek_id IS NOT NULL))
) STRICT;

-- Które należności pokrywa wpłata i jaką częścią. Wypełnia program; z tego liczy saldo okresów.
CREATE TABLE najem_czynsz_rozliczenia (
  -- Klucz nadany przez bazę jest dodatni; zob. najem_czynsz_rozliczenia_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  wplata_id TEXT NOT NULL REFERENCES najem_czynsz_wplaty (id),
  naleznosc_id TEXT NOT NULL REFERENCES najem_czynsz_naleznosci (id),
  kwota INTEGER NOT NULL CHECK (kwota > 0),

  UNIQUE (wplata_id, naleznosc_id)
) STRICT;

-- Zmiany czynszu: stara i nowa kwota, pismo i jego doręczenie, dzień, od którego obowiązuje nowa,
-- podstawa i aneks, gdy zmiana wynika z porozumienia stron.
CREATE TABLE najem_czynsz_zmiany (
  -- Klucz nadany przez bazę jest dodatni; zob. najem_czynsz_zmiany_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  najem_id TEXT NOT NULL REFERENCES najem_umowy_najmu (id),
  stara INTEGER NOT NULL CHECK (stara > 0),
  nowa INTEGER NOT NULL CHECK (nowa > 0),
  pismo_wysylka_id TEXT REFERENCES sprawa_korespondencja_wysylki (id),
  doreczono TEXT,
  obowiazuje_od TEXT NOT NULL,
  -- Np. „art. 8a UOPL, wzrost kosztów utrzymania” albo „§ 4 umowy najmu okazjonalnego”.
  podstawa TEXT NOT NULL CHECK (podstawa <> ''),
  aneks_id TEXT REFERENCES najem_umowy_aneksy (id),

  CONSTRAINT zmiana_zmienia_kwote CHECK (nowa <> stara),
  CONSTRAINT doreczenie_pisma CHECK (doreczono IS NULL OR pismo_wysylka_id IS NOT NULL)
) STRICT;

-- Zaległość za co najmniej trzy pełne okresy płatności: od kiedy, pisemne uprzedzenie o zamiarze
-- wypowiedzenia i dodatkowy termin na zapłatę (art. 11 ust. 2 pkt 2 UOPL). Kaucja najmu
-- instytucjonalnego wskazuje zaległość, którą pokryła, więc wiersz ma UUID.
CREATE TABLE najem_czynsz_zaleglosci (
  id TEXT PRIMARY KEY,
  najem_id TEXT NOT NULL REFERENCES najem_umowy_najmu (id),
  trzy_okresy_od TEXT NOT NULL,
  uprzedzenie_wysylka_id TEXT REFERENCES sprawa_korespondencja_wysylki (id),
  dodatkowy_termin_do TEXT,
  wynik TEXT NOT NULL CHECK (wynik IN ('czeka', 'splacona', 'wypowiedzenie')),

  CONSTRAINT dodatkowy_termin_po_uprzedzeniu CHECK (dodatkowy_termin_do IS NULL OR uprzedzenie_wysylka_id IS NOT NULL),
  CONSTRAINT wypowiedzenie_po_dodatkowym_terminie CHECK (wynik <> 'wypowiedzenie' OR dodatkowy_termin_do IS NOT NULL)
) STRICT;

-- Odczyty liczników w czasie najmu, każdy ze zdjęciem; stan przy wydaniu i zwrocie leży
-- w protokołach rdzenia.
CREATE TABLE najem_czynsz_odczyty (
  -- Klucz nadany przez bazę jest dodatni; zob. najem_czynsz_odczyty_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  najem_id TEXT NOT NULL REFERENCES najem_umowy_najmu (id),
  licznik TEXT NOT NULL CHECK (licznik IN ('energia', 'gaz', 'woda_zimna', 'woda_ciepla', 'cieplo')),
  numer TEXT NOT NULL CHECK (numer <> ''),
  chwila TEXT NOT NULL,
  -- Odczyt z jednostką, np. „12 512,0 kWh”.
  stan TEXT NOT NULL CHECK (stan <> ''),
  zdjecie_sciezka TEXT NOT NULL,
  zdjecie_skrot TEXT NOT NULL
) STRICT;

-- Dane o czynszu najmu, które biuro jako zarządca przekazuje gminie (art. 186a ugn): czynsz,
-- lokalizacja, budynek i lokal. Opiekun oddaje je Kierownikowi najmu, a gminie wysyła człowiek.
-- Powierzchnia użytkowa w setnych metra kwadratowego według art. 2 ust. 1 pkt 7 UOPL.
CREATE TABLE najem_czynsz_dla_gminy (
  -- Klucz nadany przez bazę jest dodatni; zob. najem_czynsz_dla_gminy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  najem_id TEXT NOT NULL REFERENCES najem_umowy_najmu (id),
  okres TEXT NOT NULL,
  czynsz INTEGER NOT NULL CHECK (czynsz > 0),
  gmina TEXT NOT NULL CHECK (gmina <> ''),
  rok_budowy INTEGER,
  stan_techniczny TEXT,
  powierzchnia INTEGER CHECK (powierzchnia > 0),
  -- Do czasu, gdy przepisy albo biuro ustalą opis standardu, wyposażenie z protokołu wydania.
  standard TEXT,
  oddano TEXT,
  wyslano TEXT,
  -- Człowiek, który wysłał zestawienie gminie.
  wyslal TEXT,

  CONSTRAINT wyslanie_ma_wysylajacego CHECK ((wyslano IS NULL) = (wyslal IS NULL)),
  CONSTRAINT wyslanie_po_oddaniu CHECK (wyslano IS NULL OR oddano IS NOT NULL)
) STRICT;

-- Należności i wpłaty -------------------------------------------------------------------------------

-- Należność poprawia się, dopóki nie pokrywa jej żadna wpłata; potem saldo stoi na niej.
CREATE TRIGGER najem_czynsz_naleznosc_zmienia_sie_przed_wplata
BEFORE UPDATE ON najem_czynsz_naleznosci
WHEN NEW.id IS NOT OLD.id
  OR NEW.najem_id IS NOT OLD.najem_id
  OR EXISTS (
    SELECT 1
    FROM najem_czynsz_rozliczenia
    WHERE naleznosc_id = OLD.id
  )
BEGIN
  SELECT RAISE(ABORT, 'należność zmienia się tylko, dopóki nie pokrywa jej żadna wpłata');
END;

-- R4: wpłata wprost do wynajmującego idzie na jego rachunek dopuszczony decyzją i niezastąpiony.
CREATE TRIGGER najem_czynsz_wplata_na_dopuszczony_rachunek
BEFORE INSERT ON najem_czynsz_wplaty
WHEN NEW.rachunek_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_rachunki_dopuszczone
    WHERE id = NEW.rachunek_id
  )
BEGIN
  SELECT RAISE(ABORT, 'wpłata idzie na rachunek dopuszczony decyzją i niezastąpiony (R4)');
END;

-- Na rachunek biura czynsz wpływa tylko wtedy, gdy bieżące warunki zarządzania mówią, że pieniądze
-- idą przez biuro.
CREATE TRIGGER najem_czynsz_wplata_do_biura_przy_przeplywie_przez_biuro
BEFORE INSERT ON najem_czynsz_wplaty
WHEN NEW.na_rachunek = 'biura'
  AND NOT coalesce(
    (
      SELECT przeplyw = 'przez_biuro'
      FROM najem_umowy_zarzadzanie
      ORDER BY id DESC
      LIMIT 1
    ),
    0
  )
BEGIN
  SELECT RAISE(ABORT, 'na rachunek biura czynsz wpływa tylko przy przepływie przez biuro');
END;

CREATE TRIGGER najem_czynsz_wplaty_bez_zmian
BEFORE UPDATE ON najem_czynsz_wplaty
BEGIN
  SELECT RAISE(ABORT, 'wpłaty się nie zmienia; pomyłkę poprawia rozliczenie albo nowy wpis');
END;

-- Rozliczenie nie przekracza ani wpłaty, ani należności; rozliczenia się nie zmienia.
CREATE TRIGGER najem_czynsz_rozliczenie_w_granicach_wplaty_i_naleznosci
BEFORE INSERT ON najem_czynsz_rozliczenia
WHEN coalesce(
    (
      SELECT sum(kwota)
      FROM najem_czynsz_rozliczenia
      WHERE wplata_id = NEW.wplata_id
    ),
    0
  ) + NEW.kwota > (
    SELECT kwota
    FROM najem_czynsz_wplaty
    WHERE id = NEW.wplata_id
  )
  OR coalesce(
    (
      SELECT sum(kwota)
      FROM najem_czynsz_rozliczenia
      WHERE naleznosc_id = NEW.naleznosc_id
    ),
    0
  ) + NEW.kwota > (
    SELECT kwota
    FROM najem_czynsz_naleznosci
    WHERE id = NEW.naleznosc_id
  )
BEGIN
  SELECT RAISE(ABORT, 'rozliczenie nie przekracza kwoty wpłaty ani kwoty należności');
END;

CREATE TRIGGER najem_czynsz_rozliczenia_bez_zmian
BEFORE UPDATE ON najem_czynsz_rozliczenia
BEGIN
  SELECT RAISE(ABORT, 'rozliczenia wpłaty się nie zmienia');
END;

-- Decyzja T13: biuro przekazuje wynajmującemu każdą wpłatę czynszu osobno. Wypłata wskazuje jedną
-- wpłatę, która wpłynęła na rachunek biura, idzie do wynajmującego, nie przekracza wpłaty i jest
-- jedyną nieanulowaną wypłatą tej wpłaty. Pieniądze idą przez biuro tylko przy takim przepływie.
CREATE TRIGGER najem_czynsz_przekazanie_wplaty_wynajmujacemu
BEFORE INSERT ON sprawa_wyplaty_wpisy
WHEN NEW.podstawa_tabela = 'najem_czynsz_wplaty'
  AND (
    NOT EXISTS (
      SELECT 1
      FROM najem_czynsz_wplaty
      WHERE id = NEW.podstawa_id
        AND na_rachunek = 'biura'
        AND kwota >= NEW.kwota
    )
    OR NOT coalesce(
      (
        SELECT przeplyw = 'przez_biuro'
        FROM najem_umowy_zarzadzanie
        ORDER BY id DESC
        LIMIT 1
      ),
      0
    )
    OR NOT EXISTS (
      SELECT 1
      FROM sprawa_uczestnicy_role
      WHERE uczestnik_id = NEW.odbiorca_uczestnik_id
        AND rola = 'wynajmujacy'
        AND zakonczono IS NULL
    )
    OR EXISTS (
      SELECT 1
      FROM sprawa_wyplaty_wpisy
      WHERE podstawa_tabela = 'najem_czynsz_wplaty'
        AND podstawa_id = NEW.podstawa_id
        AND stan <> 'anulowana'
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'przekazanie czynszu wskazuje jedną wpłatę na rachunek biura przy przepływie przez biuro, idzie do wynajmującego, nie przekracza wpłaty i jest jedyne (T13)');
END;

-- Zmiany czynszu ------------------------------------------------------------------------------------

-- Zmiana zaczyna się od czynszu, który obowiązuje: z umowy albo z ostatniej zmiany. Zmiana
-- z porozumienia stron wskazuje aneks tej umowy, który już obowiązuje.
CREATE TRIGGER najem_czynsz_zmiana_od_biezacego_czynszu
BEFORE INSERT ON najem_czynsz_zmiany
WHEN NEW.stara IS NOT coalesce(
    (
      SELECT nowa
      FROM najem_czynsz_zmiany
      WHERE najem_id = NEW.najem_id
      ORDER BY id DESC
      LIMIT 1
    ),
    (
      SELECT czynsz
      FROM najem_umowy_najmu
      WHERE id = NEW.najem_id
    )
  )
  OR (
    NEW.aneks_id IS NOT NULL
    AND NOT EXISTS (
      SELECT 1
      FROM najem_umowy_aneksy
      WHERE id = NEW.aneks_id
        AND najem_id = NEW.najem_id
        AND obowiazuje_od IS NOT NULL
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'zmiana czynszu zaczyna się od obowiązującego czynszu, a z porozumienia wskazuje obowiązujący aneks tej umowy');
END;

-- Art. 8a ust. 1–3 UOPL: przy najmie zwykłym właściciel podwyższa czynsz, wypowiadając jego
-- wysokość na piśmie z trzymiesięcznym terminem, na koniec miesiąca. Nowy czynsz obowiązuje więc od
-- pierwszego dnia miesiąca, najwcześniej po miesiącu, w którym kończy się termin liczony od
-- doręczenia (miesiąc doręczenia + 4). Dłuższy termin z umowy sprawdza Opiekun.
CREATE TRIGGER najem_czynsz_podwyzka_przy_zwyklym_po_wypowiedzeniu
BEFORE INSERT ON najem_czynsz_zmiany
WHEN NEW.nowa > NEW.stara
  AND NEW.aneks_id IS NULL
  AND (
    SELECT rodzaj = 'zwykly'
    FROM najem_umowy_najmu
    WHERE id = NEW.najem_id
  )
  AND NOT coalesce(
    NEW.doreczono IS NOT NULL
    AND NEW.obowiazuje_od = date(NEW.obowiazuje_od, 'start of month')
    AND NEW.obowiazuje_od >= date(NEW.doreczono, 'start of month', '+4 months'),
    0
  )
BEGIN
  SELECT RAISE(ABORT, 'podwyżka przy najmie zwykłym obowiązuje od pierwszego dnia miesiąca po trzymiesięcznym wypowiedzeniu doręczonym na piśmie (art. 8a UOPL)');
END;

-- Art. 9 ust. 1b UOPL: czynszu nie podwyższa się częściej niż co 6 miesięcy, licząc od dnia,
-- w którym poprzednia podwyżka zaczęła obowiązywać.
CREATE TRIGGER najem_czynsz_podwyzka_przy_zwyklym_co_6_miesiecy
BEFORE INSERT ON najem_czynsz_zmiany
WHEN NEW.nowa > NEW.stara
  AND NEW.aneks_id IS NULL
  AND (
    SELECT rodzaj = 'zwykly'
    FROM najem_umowy_najmu
    WHERE id = NEW.najem_id
  )
  AND EXISTS (
    SELECT 1
    FROM najem_czynsz_zmiany
    WHERE najem_id = NEW.najem_id
      AND nowa > stara
      AND NEW.obowiazuje_od < date(obowiazuje_od, '+6 months')
  )
BEGIN
  SELECT RAISE(ABORT, 'przy najmie zwykłym czynszu nie podwyższa się częściej niż co 6 miesięcy (art. 9 ust. 1b UOPL)');
END;

CREATE TRIGGER najem_czynsz_zmiany_bez_zmian
BEFORE UPDATE ON najem_czynsz_zmiany
BEGIN
  SELECT RAISE(ABORT, 'zmiany czynszu się nie zmienia; kolejna zmiana to nowy wpis');
END;

-- Zaległości ----------------------------------------------------------------------------------------

CREATE TRIGGER najem_czynsz_zaleglosc_powstaje_czekajaca
BEFORE INSERT ON najem_czynsz_zaleglosci
WHEN NEW.wynik <> 'czeka'
  OR NEW.uprzedzenie_wysylka_id IS NOT NULL
  OR NEW.dodatkowy_termin_do IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'zaległość powstaje jako czekająca, bez uprzedzenia i dodatkowego terminu');
END;

-- Uprzedzenie o zamiarze wypowiedzenia wychodzi za zgodą człowieka, więc zaległość wskazuje pismo,
-- które jeszcze nie wyszło; reguła wysyłki niżej puszcza je tylko przez czeka_na_zgode.
CREATE TRIGGER najem_czynsz_uprzedzenie_wskazuje_niewyslane_pismo
BEFORE UPDATE OF uprzedzenie_wysylka_id ON najem_czynsz_zaleglosci
WHEN NEW.uprzedzenie_wysylka_id IS NOT NULL
  AND NEW.uprzedzenie_wysylka_id IS NOT OLD.uprzedzenie_wysylka_id
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_korespondencja_wysylki
    WHERE id = NEW.uprzedzenie_wysylka_id
      AND stan IN ('projekt', 'czeka_na_zgode')
  )
BEGIN
  SELECT RAISE(ABORT, 'zaległość wskazuje uprzedzenie, które jeszcze nie wyszło; wychodzi ono za zgodą człowieka');
END;

CREATE TRIGGER najem_czynsz_uprzedzenie_za_zgoda
BEFORE UPDATE OF stan ON sprawa_korespondencja_wysylki
WHEN NEW.stan = 'wyslana'
  AND OLD.stan = 'projekt'
  AND EXISTS (
    SELECT 1
    FROM najem_czynsz_zaleglosci
    WHERE uprzedzenie_wysylka_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'uprzedzenie o zamiarze wypowiedzenia wychodzi przez czeka_na_zgode, za zgodą człowieka');
END;

-- Art. 11 ust. 2 pkt 2 UOPL: dodatkowy termin na zapłatę trwa co najmniej miesiąc od doręczenia
-- uprzedzenia.
CREATE TRIGGER najem_czynsz_dodatkowy_termin_co_najmniej_miesiac
BEFORE UPDATE OF dodatkowy_termin_do ON najem_czynsz_zaleglosci
WHEN NEW.dodatkowy_termin_do IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_korespondencja_wysylki
    WHERE id = NEW.uprzedzenie_wysylka_id
      AND stan = 'doreczona'
      AND NEW.dodatkowy_termin_do >= date(doreczono, '+1 month')
  )
BEGIN
  SELECT RAISE(ABORT, 'dodatkowy termin trwa co najmniej miesiąc od doręczenia uprzedzenia (art. 11 ust. 2 pkt 2 UOPL)');
END;

CREATE TRIGGER najem_czynsz_zaleglosc_bez_zmian
BEFORE UPDATE ON najem_czynsz_zaleglosci
WHEN OLD.wynik <> 'czeka'
  OR NEW.id IS NOT OLD.id
  OR NEW.najem_id IS NOT OLD.najem_id
  OR NEW.trzy_okresy_od IS NOT OLD.trzy_okresy_od
  OR (OLD.uprzedzenie_wysylka_id IS NOT NULL AND NEW.uprzedzenie_wysylka_id IS NOT OLD.uprzedzenie_wysylka_id)
  OR (OLD.dodatkowy_termin_do IS NOT NULL AND NEW.dodatkowy_termin_do IS NOT OLD.dodatkowy_termin_do)
BEGIN
  SELECT RAISE(ABORT, 'zaległości się nie zmienia; uprzedzenie, dodatkowy termin i wynik wpisuje się raz');
END;

-- Odczyty i dane dla gminy --------------------------------------------------------------------------

CREATE TRIGGER najem_czynsz_odczyty_bez_zmian
BEFORE UPDATE ON najem_czynsz_odczyty
BEGIN
  SELECT RAISE(ABORT, 'odczytu licznika się nie zmienia; poprawka to nowy odczyt');
END;

-- Art. 186a ugn i R6: dane o czynszu przekazuje zarządca, więc tylko przy zawartej umowie
-- o zarządzanie, i tylko o lokalach mieszkalnych (najem lokalu użytkowego tego nie obejmuje).
CREATE TRIGGER najem_czynsz_dane_dla_gminy_przy_zarzadzaniu_lokalem_mieszkalnym
BEFORE INSERT ON najem_czynsz_dla_gminy
WHEN NOT EXISTS (
    SELECT 1
    FROM sprawa_umowy_wpisy
    WHERE rodzaj = 'zarzadzanie'
      AND stan = 'zawarta'
  )
  OR (
    SELECT nieruchomosc.rodzaj = 'lokal_uzytkowy'
    FROM najem_umowy_najmu AS najem
    JOIN sprawa_nieruchomosci_wpisy AS nieruchomosc ON nieruchomosc.id = najem.nieruchomosc_id
    WHERE najem.id = NEW.najem_id
  )
BEGIN
  SELECT RAISE(ABORT, 'dane dla gminy przygotowuje się przy zawartej umowie o zarządzanie i nie o lokalu użytkowym');
END;

CREATE TRIGGER najem_czynsz_dane_dla_gminy_powstaja_nieoddane
BEFORE INSERT ON najem_czynsz_dla_gminy
WHEN NEW.oddano IS NOT NULL
  OR NEW.wyslano IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'dane dla gminy powstają nieoddane i niewysłane');
END;

-- Dane poprawia się, dopóki Opiekun ich nie oddał; oddanie i wysłanie wpisuje się raz.
CREATE TRIGGER najem_czynsz_dane_dla_gminy_po_oddaniu_niezmienne
BEFORE UPDATE ON najem_czynsz_dla_gminy
WHEN NEW.id IS NOT OLD.id
  OR NEW.najem_id IS NOT OLD.najem_id
  OR (OLD.oddano IS NOT NULL AND (
    NEW.oddano IS NOT OLD.oddano
    OR NEW.okres IS NOT OLD.okres
    OR NEW.czynsz IS NOT OLD.czynsz
    OR NEW.gmina IS NOT OLD.gmina
    OR NEW.rok_budowy IS NOT OLD.rok_budowy
    OR NEW.stan_techniczny IS NOT OLD.stan_techniczny
    OR NEW.powierzchnia IS NOT OLD.powierzchnia
    OR NEW.standard IS NOT OLD.standard
  ))
  OR (OLD.wyslano IS NOT NULL AND (
    NEW.wyslano IS NOT OLD.wyslano
    OR NEW.wyslal IS NOT OLD.wyslal
  ))
BEGIN
  SELECT RAISE(ABORT, 'oddanych danych dla gminy się nie zmienia; oddanie i wysłanie wpisuje się raz');
END;

-- Bez usuwania i zastępowania ---------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER najem_czynsz_naleznosci_bez_zastepowania
BEFORE INSERT ON najem_czynsz_naleznosci
WHEN EXISTS (
  SELECT 1
  FROM najem_czynsz_naleznosci
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER najem_czynsz_naleznosci_bez_usuwania
BEFORE DELETE ON najem_czynsz_naleznosci
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER najem_czynsz_wplaty_bez_zastepowania
BEFORE INSERT ON najem_czynsz_wplaty
WHEN EXISTS (
  SELECT 1
  FROM najem_czynsz_wplaty
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER najem_czynsz_wplaty_bez_usuwania
BEFORE DELETE ON najem_czynsz_wplaty
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER najem_czynsz_rozliczenia_bez_zastepowania
BEFORE INSERT ON najem_czynsz_rozliczenia
WHEN EXISTS (
    SELECT 1
    FROM najem_czynsz_rozliczenia
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM najem_czynsz_rozliczenia
    WHERE wplata_id = NEW.wplata_id
      AND naleznosc_id = NEW.naleznosc_id
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER najem_czynsz_rozliczenia_bez_usuwania
BEFORE DELETE ON najem_czynsz_rozliczenia
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER najem_czynsz_zmiany_bez_zastepowania
BEFORE INSERT ON najem_czynsz_zmiany
WHEN EXISTS (
  SELECT 1
  FROM najem_czynsz_zmiany
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER najem_czynsz_zmiany_bez_usuwania
BEFORE DELETE ON najem_czynsz_zmiany
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER najem_czynsz_zaleglosci_bez_zastepowania
BEFORE INSERT ON najem_czynsz_zaleglosci
WHEN EXISTS (
  SELECT 1
  FROM najem_czynsz_zaleglosci
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER najem_czynsz_zaleglosci_bez_usuwania
BEFORE DELETE ON najem_czynsz_zaleglosci
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER najem_czynsz_odczyty_bez_zastepowania
BEFORE INSERT ON najem_czynsz_odczyty
WHEN EXISTS (
  SELECT 1
  FROM najem_czynsz_odczyty
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER najem_czynsz_odczyty_bez_usuwania
BEFORE DELETE ON najem_czynsz_odczyty
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER najem_czynsz_dla_gminy_bez_zastepowania
BEFORE INSERT ON najem_czynsz_dla_gminy
WHEN EXISTS (
  SELECT 1
  FROM najem_czynsz_dla_gminy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER najem_czynsz_dla_gminy_bez_usuwania
BEFORE DELETE ON najem_czynsz_dla_gminy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
];

/// The values of column `naleznosci.tytul`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NaleznosciTytul {
    Czynsz,
    Media,
    Wspolnota,
    Inna,
}

impl NaleznosciTytul {
    /// Every value, in the order of the catalog.
    pub const ALL: &[NaleznosciTytul] = &[
        NaleznosciTytul::Czynsz,
        NaleznosciTytul::Media,
        NaleznosciTytul::Wspolnota,
        NaleznosciTytul::Inna,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            NaleznosciTytul::Czynsz => "czynsz",
            NaleznosciTytul::Media => "media",
            NaleznosciTytul::Wspolnota => "wspolnota",
            NaleznosciTytul::Inna => "inna",
        }
    }
}

impl xmeta_room::FromSqlValue for NaleznosciTytul {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "czynsz" => Ok(NaleznosciTytul::Czynsz),
                    "media" => Ok(NaleznosciTytul::Media),
                    "wspolnota" => Ok(NaleznosciTytul::Wspolnota),
                    "inna" => Ok(NaleznosciTytul::Inna),
                    other => Err(format!("{other} is no value of NaleznosciTytul")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for NaleznosciTytul {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `odczyty.licznik`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OdczytyLicznik {
    Energia,
    Gaz,
    WodaZimna,
    WodaCiepla,
    Cieplo,
}

impl OdczytyLicznik {
    /// Every value, in the order of the catalog.
    pub const ALL: &[OdczytyLicznik] = &[
        OdczytyLicznik::Energia,
        OdczytyLicznik::Gaz,
        OdczytyLicznik::WodaZimna,
        OdczytyLicznik::WodaCiepla,
        OdczytyLicznik::Cieplo,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            OdczytyLicznik::Energia => "energia",
            OdczytyLicznik::Gaz => "gaz",
            OdczytyLicznik::WodaZimna => "woda_zimna",
            OdczytyLicznik::WodaCiepla => "woda_ciepla",
            OdczytyLicznik::Cieplo => "cieplo",
        }
    }
}

impl xmeta_room::FromSqlValue for OdczytyLicznik {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "energia" => Ok(OdczytyLicznik::Energia),
                    "gaz" => Ok(OdczytyLicznik::Gaz),
                    "woda_zimna" => Ok(OdczytyLicznik::WodaZimna),
                    "woda_ciepla" => Ok(OdczytyLicznik::WodaCiepla),
                    "cieplo" => Ok(OdczytyLicznik::Cieplo),
                    other => Err(format!("{other} is no value of OdczytyLicznik")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for OdczytyLicznik {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wplaty.na_rachunek`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WplatyNaRachunek {
    Biura,
    Wynajmujacego,
}

impl WplatyNaRachunek {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WplatyNaRachunek] = &[
        WplatyNaRachunek::Biura,
        WplatyNaRachunek::Wynajmujacego,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WplatyNaRachunek::Biura => "biura",
            WplatyNaRachunek::Wynajmujacego => "wynajmujacego",
        }
    }
}

impl xmeta_room::FromSqlValue for WplatyNaRachunek {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "biura" => Ok(WplatyNaRachunek::Biura),
                    "wynajmujacego" => Ok(WplatyNaRachunek::Wynajmujacego),
                    other => Err(format!("{other} is no value of WplatyNaRachunek")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WplatyNaRachunek {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wplaty.potwierdzenie`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WplatyPotwierdzenie {
    WyciagBiura,
    OdWynajmujacego,
    OdNajemcy,
}

impl WplatyPotwierdzenie {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WplatyPotwierdzenie] = &[
        WplatyPotwierdzenie::WyciagBiura,
        WplatyPotwierdzenie::OdWynajmujacego,
        WplatyPotwierdzenie::OdNajemcy,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WplatyPotwierdzenie::WyciagBiura => "wyciag_biura",
            WplatyPotwierdzenie::OdWynajmujacego => "od_wynajmujacego",
            WplatyPotwierdzenie::OdNajemcy => "od_najemcy",
        }
    }
}

impl xmeta_room::FromSqlValue for WplatyPotwierdzenie {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "wyciag_biura" => Ok(WplatyPotwierdzenie::WyciagBiura),
                    "od_wynajmujacego" => Ok(WplatyPotwierdzenie::OdWynajmujacego),
                    "od_najemcy" => Ok(WplatyPotwierdzenie::OdNajemcy),
                    other => Err(format!("{other} is no value of WplatyPotwierdzenie")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WplatyPotwierdzenie {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `zaleglosci.wynik`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZaleglosciWynik {
    Czeka,
    Splacona,
    Wypowiedzenie,
}

impl ZaleglosciWynik {
    /// Every value, in the order of the catalog.
    pub const ALL: &[ZaleglosciWynik] = &[
        ZaleglosciWynik::Czeka,
        ZaleglosciWynik::Splacona,
        ZaleglosciWynik::Wypowiedzenie,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            ZaleglosciWynik::Czeka => "czeka",
            ZaleglosciWynik::Splacona => "splacona",
            ZaleglosciWynik::Wypowiedzenie => "wypowiedzenie",
        }
    }
}

impl xmeta_room::FromSqlValue for ZaleglosciWynik {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "czeka" => Ok(ZaleglosciWynik::Czeka),
                    "splacona" => Ok(ZaleglosciWynik::Splacona),
                    "wypowiedzenie" => Ok(ZaleglosciWynik::Wypowiedzenie),
                    other => Err(format!("{other} is no value of ZaleglosciWynik")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for ZaleglosciWynik {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of field `wynik` of `odnotuj_wynik_zaleglosci`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Wynik {
    Splacona,
    Wypowiedzenie,
}

impl Wynik {
    /// Every value, in the order of the catalog.
    pub const ALL: &[Wynik] = &[Wynik::Splacona, Wynik::Wypowiedzenie];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            Wynik::Splacona => "splacona",
            Wynik::Wypowiedzenie => "wypowiedzenie",
        }
    }
}

impl xmeta_room::FromSqlValue for Wynik {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "splacona" => Ok(Wynik::Splacona),
                    "wypowiedzenie" => Ok(Wynik::Wypowiedzenie),
                    other => Err(format!("{other} is no value of Wynik")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for Wynik {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `dane_dla_gminy`.
#[derive(Debug, Clone, PartialEq)]
pub struct DaneDlaGminyRow {
    pub id: i64,
    pub najem_id: String,
    pub okres: String,
    pub czynsz: i64,
    pub gmina: String,
    pub rok_budowy: Option<i64>,
    pub stan_techniczny: Option<String>,
    pub powierzchnia: Option<i64>,
    pub standard: Option<String>,
    pub oddano: Option<String>,
    pub wyslano: Option<String>,
    pub wyslal: Option<String>,
}

impl DaneDlaGminyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(DaneDlaGminyRow {
            id: row.get("id")?,
            najem_id: row.get("najem_id")?,
            okres: row.get("okres")?,
            czynsz: row.get("czynsz")?,
            gmina: row.get("gmina")?,
            rok_budowy: row.get("rok_budowy")?,
            stan_techniczny: row.get("stan_techniczny")?,
            powierzchnia: row.get("powierzchnia")?,
            standard: row.get("standard")?,
            oddano: row.get("oddano")?,
            wyslano: row.get("wyslano")?,
            wyslal: row.get("wyslal")?,
        })
    }
}

/// Dane o czynszu dla gminy w kolejności okresów.
pub fn dane_dla_gminy(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<DaneDlaGminyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM najem_czynsz_dla_gminy
ORDER BY okres, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(DaneDlaGminyRow::from_row).collect()
}

/// The parameters of `przygotuj_dane_dla_gminy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrzygotujDaneDlaGminyParams {
    pub najem_id: String,
    pub okres: String,
    pub czynsz: i64,
    pub gmina: String,
    pub rok_budowy: Option<i64>,
    pub stan_techniczny: Option<String>,
    pub powierzchnia: Option<i64>,
    pub standard: Option<String>,
}

/// Dane tego najmu za okres, z umowy i z dokumentów budynku.
pub fn przygotuj_dane_dla_gminy(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzygotujDaneDlaGminyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO najem_czynsz_dla_gminy (najem_id, okres, czynsz, gmina, rok_budowy, stan_techniczny,
  powierzchnia, standard)
VALUES (:najem_id, :okres, :czynsz, :gmina, :rok_budowy, :stan_techniczny, :powierzchnia,
  :standard)"#,
                parameters: Vec::from([
                    ("najem_id", xmeta_room::ToSqlValue::to_sql_value(&params.najem_id)),
                    ("okres", xmeta_room::ToSqlValue::to_sql_value(&params.okres)),
                    ("czynsz", xmeta_room::ToSqlValue::to_sql_value(&params.czynsz)),
                    ("gmina", xmeta_room::ToSqlValue::to_sql_value(&params.gmina)),
                    (
                        "rok_budowy",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rok_budowy),
                    ),
                    (
                        "stan_techniczny",
                        xmeta_room::ToSqlValue::to_sql_value(&params.stan_techniczny),
                    ),
                    (
                        "powierzchnia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.powierzchnia),
                    ),
                    ("standard", xmeta_room::ToSqlValue::to_sql_value(&params.standard)),
                ]),
            },
        ],
    )
}

/// The parameters of `popraw_dane_dla_gminy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PoprawDaneDlaGminyParams {
    pub czynsz: i64,
    pub rok_budowy: Option<i64>,
    pub stan_techniczny: Option<String>,
    pub powierzchnia: Option<i64>,
    pub standard: Option<String>,
    pub id: i64,
}

/// Poprawka danych, dopóki Opiekun ich nie oddał.
pub fn popraw_dane_dla_gminy(
    room: &impl xmeta_room::MetaRoom,
    params: &PoprawDaneDlaGminyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_czynsz_dla_gminy
SET czynsz = :czynsz,
  rok_budowy = :rok_budowy,
  stan_techniczny = :stan_techniczny,
  powierzchnia = :powierzchnia,
  standard = :standard
WHERE id = :id"#,
                parameters: Vec::from([
                    ("czynsz", xmeta_room::ToSqlValue::to_sql_value(&params.czynsz)),
                    (
                        "rok_budowy",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rok_budowy),
                    ),
                    (
                        "stan_techniczny",
                        xmeta_room::ToSqlValue::to_sql_value(&params.stan_techniczny),
                    ),
                    (
                        "powierzchnia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.powierzchnia),
                    ),
                    ("standard", xmeta_room::ToSqlValue::to_sql_value(&params.standard)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_oddanie_danych_dla_gminy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujOddanieDanychDlaGminyParams {
    pub oddano: String,
    pub id: i64,
}

/// Opiekun oddał dane Kierownikowi najmu.
pub fn odnotuj_oddanie_danych_dla_gminy(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujOddanieDanychDlaGminyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_czynsz_dla_gminy
SET oddano = :oddano
WHERE id = :id"#,
                parameters: Vec::from([
                    ("oddano", xmeta_room::ToSqlValue::to_sql_value(&params.oddano)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_wyslanie_danych_dla_gminy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujWyslanieDanychDlaGminyParams {
    pub wyslano: String,
    pub wyslal: String,
    pub id: i64,
}

/// Człowiek wysłał zestawienie gminie.
pub fn odnotuj_wyslanie_danych_dla_gminy(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWyslanieDanychDlaGminyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_czynsz_dla_gminy
SET wyslano = :wyslano,
  wyslal = :wyslal
WHERE id = :id"#,
                parameters: Vec::from([
                    ("wyslano", xmeta_room::ToSqlValue::to_sql_value(&params.wyslano)),
                    ("wyslal", xmeta_room::ToSqlValue::to_sql_value(&params.wyslal)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `naleznosci_najmu`.
#[derive(Debug, Clone, PartialEq)]
pub struct NaleznosciNajmuRow {
    pub id: String,
    pub najem_id: String,
    pub tytul: NaleznosciTytul,
    pub okres: String,
    pub kwota: i64,
    pub termin: String,
}

impl NaleznosciNajmuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(NaleznosciNajmuRow {
            id: row.get("id")?,
            najem_id: row.get("najem_id")?,
            tytul: row.get("tytul")?,
            okres: row.get("okres")?,
            kwota: row.get("kwota")?,
            termin: row.get("termin")?,
        })
    }
}

/// Harmonogram: należności w kolejności terminu.
pub fn naleznosci_najmu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<NaleznosciNajmuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM najem_czynsz_naleznosci
ORDER BY termin, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(NaleznosciNajmuRow::from_row).collect()
}

/// A row of `saldo_naleznosci`.
#[derive(Debug, Clone, PartialEq)]
pub struct SaldoNaleznosciRow {
    pub id: String,
    pub najem_id: String,
    pub tytul: NaleznosciTytul,
    pub okres: String,
    pub kwota: i64,
    pub termin: String,
    pub zaplacono: i64,
    pub zostalo: i64,
}

impl SaldoNaleznosciRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(SaldoNaleznosciRow {
            id: row.get("id")?,
            najem_id: row.get("najem_id")?,
            tytul: row.get("tytul")?,
            okres: row.get("okres")?,
            kwota: row.get("kwota")?,
            termin: row.get("termin")?,
            zaplacono: row.get("zaplacono")?,
            zostalo: row.get("zostalo")?,
        })
    }
}

/// Saldo każdej należności: ile pokryły wpłaty i ile zostało; z niego program liczy zaległość.
pub fn saldo_naleznosci(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<SaldoNaleznosciRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT
  naleznosc.id,
  naleznosc.najem_id,
  naleznosc.tytul,
  naleznosc.okres,
  naleznosc.kwota,
  naleznosc.termin,
  coalesce(sum(rozliczenie.kwota), 0) AS zaplacono,
  naleznosc.kwota - coalesce(sum(rozliczenie.kwota), 0) AS zostalo
FROM najem_czynsz_naleznosci AS naleznosc
LEFT JOIN najem_czynsz_rozliczenia AS rozliczenie ON rozliczenie.naleznosc_id = naleznosc.id
GROUP BY naleznosc.id
ORDER BY naleznosc.termin, naleznosc.id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(SaldoNaleznosciRow::from_row).collect()
}

/// The parameters of `wpisz_naleznosc`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszNaleznoscParams {
    pub id: String,
    pub najem_id: String,
    pub tytul: NaleznosciTytul,
    pub okres: String,
    pub kwota: i64,
    pub termin: String,
}

/// Należność z harmonogramu, który liczy program; identyfikator (UUID) nadaje aplikacja.
pub fn wpisz_naleznosc(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszNaleznoscParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO najem_czynsz_naleznosci (id, najem_id, tytul, okres, kwota, termin)
VALUES (:id, :najem_id, :tytul, :okres, :kwota, :termin)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("najem_id", xmeta_room::ToSqlValue::to_sql_value(&params.najem_id)),
                    ("tytul", xmeta_room::ToSqlValue::to_sql_value(&params.tytul)),
                    ("okres", xmeta_room::ToSqlValue::to_sql_value(&params.okres)),
                    ("kwota", xmeta_room::ToSqlValue::to_sql_value(&params.kwota)),
                    ("termin", xmeta_room::ToSqlValue::to_sql_value(&params.termin)),
                ]),
            },
        ],
    )
}

/// The parameters of `popraw_naleznosc`.
#[derive(Debug, Clone, PartialEq)]
pub struct PoprawNaleznoscParams {
    pub tytul: NaleznosciTytul,
    pub okres: String,
    pub kwota: i64,
    pub termin: String,
    pub id: String,
}

/// Poprawka należności, dopóki nie pokrywa jej żadna wpłata.
pub fn popraw_naleznosc(
    room: &impl xmeta_room::MetaRoom,
    params: &PoprawNaleznoscParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_czynsz_naleznosci
SET tytul = :tytul,
  okres = :okres,
  kwota = :kwota,
  termin = :termin
WHERE id = :id"#,
                parameters: Vec::from([
                    ("tytul", xmeta_room::ToSqlValue::to_sql_value(&params.tytul)),
                    ("okres", xmeta_room::ToSqlValue::to_sql_value(&params.okres)),
                    ("kwota", xmeta_room::ToSqlValue::to_sql_value(&params.kwota)),
                    ("termin", xmeta_room::ToSqlValue::to_sql_value(&params.termin)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `odczyty_licznikow`.
#[derive(Debug, Clone, PartialEq)]
pub struct OdczytyLicznikowRow {
    pub id: i64,
    pub najem_id: String,
    pub licznik: OdczytyLicznik,
    pub numer: String,
    pub chwila: String,
    pub stan: String,
    pub zdjecie_sciezka: String,
    pub zdjecie_skrot: String,
}

impl OdczytyLicznikowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(OdczytyLicznikowRow {
            id: row.get("id")?,
            najem_id: row.get("najem_id")?,
            licznik: row.get("licznik")?,
            numer: row.get("numer")?,
            chwila: row.get("chwila")?,
            stan: row.get("stan")?,
            zdjecie_sciezka: row.get("zdjecie_sciezka")?,
            zdjecie_skrot: row.get("zdjecie_skrot")?,
        })
    }
}

/// Odczyty liczników w kolejności chwili.
pub fn odczyty_licznikow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<OdczytyLicznikowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM najem_czynsz_odczyty
ORDER BY licznik, chwila, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(OdczytyLicznikowRow::from_row).collect()
}

/// The parameters of `wpisz_odczyt_licznika`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszOdczytLicznikaParams {
    pub najem_id: String,
    pub licznik: OdczytyLicznik,
    pub numer: String,
    pub chwila: String,
    pub stan: String,
    pub zdjecie_sciezka: String,
    pub zdjecie_skrot: String,
}

/// Odczyt licznika ze zdjęciem, porównany przez Opiekuna z poprzednim.
pub fn wpisz_odczyt_licznika(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszOdczytLicznikaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO najem_czynsz_odczyty (najem_id, licznik, numer, chwila, stan, zdjecie_sciezka,
  zdjecie_skrot)
VALUES (:najem_id, :licznik, :numer, :chwila, :stan, :zdjecie_sciezka, :zdjecie_skrot)"#,
                parameters: Vec::from([
                    ("najem_id", xmeta_room::ToSqlValue::to_sql_value(&params.najem_id)),
                    ("licznik", xmeta_room::ToSqlValue::to_sql_value(&params.licznik)),
                    ("numer", xmeta_room::ToSqlValue::to_sql_value(&params.numer)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    ("stan", xmeta_room::ToSqlValue::to_sql_value(&params.stan)),
                    (
                        "zdjecie_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zdjecie_sciezka),
                    ),
                    (
                        "zdjecie_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zdjecie_skrot),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `rozliczenia_wplat`.
#[derive(Debug, Clone, PartialEq)]
pub struct RozliczeniaWplatRow {
    pub id: i64,
    pub wplata_id: String,
    pub naleznosc_id: String,
    pub kwota: i64,
}

impl RozliczeniaWplatRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(RozliczeniaWplatRow {
            id: row.get("id")?,
            wplata_id: row.get("wplata_id")?,
            naleznosc_id: row.get("naleznosc_id")?,
            kwota: row.get("kwota")?,
        })
    }
}

/// Które należności pokrywa każda wpłata.
pub fn rozliczenia_wplat(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<RozliczeniaWplatRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM najem_czynsz_rozliczenia
ORDER BY wplata_id, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(RozliczeniaWplatRow::from_row).collect()
}

/// The parameters of `rozlicz_wplate`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RozliczWplateParams {
    pub wplata_id: String,
    pub naleznosc_id: String,
    pub kwota: i64,
}

/// Część wpłaty na jedną należność, wyliczona przez program.
pub fn rozlicz_wplate(
    room: &impl xmeta_room::MetaRoom,
    params: &RozliczWplateParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO najem_czynsz_rozliczenia (wplata_id, naleznosc_id, kwota)
VALUES (:wplata_id, :naleznosc_id, :kwota)"#,
                parameters: Vec::from([
                    (
                        "wplata_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wplata_id),
                    ),
                    (
                        "naleznosc_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.naleznosc_id),
                    ),
                    ("kwota", xmeta_room::ToSqlValue::to_sql_value(&params.kwota)),
                ]),
            },
        ],
    )
}

/// A row of `wplaty_czynszu`.
#[derive(Debug, Clone, PartialEq)]
pub struct WplatyCzynszuRow {
    pub id: String,
    pub najem_id: String,
    pub chwila: String,
    pub kwota: i64,
    pub placacy_uczestnik_id: Option<String>,
    pub na_rachunek: WplatyNaRachunek,
    pub rachunek_id: Option<String>,
    pub potwierdzenie: WplatyPotwierdzenie,
    pub dokument_id: String,
}

impl WplatyCzynszuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WplatyCzynszuRow {
            id: row.get("id")?,
            najem_id: row.get("najem_id")?,
            chwila: row.get("chwila")?,
            kwota: row.get("kwota")?,
            placacy_uczestnik_id: row.get("placacy_uczestnik_id")?,
            na_rachunek: row.get("na_rachunek")?,
            rachunek_id: row.get("rachunek_id")?,
            potwierdzenie: row.get("potwierdzenie")?,
            dokument_id: row.get("dokument_id")?,
        })
    }
}

/// Wpłaty w kolejności chwili wpływu.
pub fn wplaty_czynszu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WplatyCzynszuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM najem_czynsz_wplaty
ORDER BY chwila, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(WplatyCzynszuRow::from_row).collect()
}

/// The parameters of `wpisz_wplate_czynszu`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszWplateCzynszuParams {
    pub id: String,
    pub najem_id: String,
    pub chwila: String,
    pub kwota: i64,
    pub placacy_uczestnik_id: Option<String>,
    pub na_rachunek: WplatyNaRachunek,
    pub rachunek_id: Option<String>,
    pub potwierdzenie: WplatyPotwierdzenie,
    pub dokument_id: String,
}

/// Wpłata z płacącym, rachunkiem i potwierdzeniem; identyfikator (UUID) nadaje aplikacja, bo
/// wskazuje go przekazanie wynajmującemu.
pub fn wpisz_wplate_czynszu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszWplateCzynszuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO najem_czynsz_wplaty (id, najem_id, chwila, kwota, placacy_uczestnik_id, na_rachunek,
  rachunek_id, potwierdzenie, dokument_id)
VALUES (:id, :najem_id, :chwila, :kwota, :placacy_uczestnik_id, :na_rachunek, :rachunek_id,
  :potwierdzenie, :dokument_id)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("najem_id", xmeta_room::ToSqlValue::to_sql_value(&params.najem_id)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    ("kwota", xmeta_room::ToSqlValue::to_sql_value(&params.kwota)),
                    (
                        "placacy_uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.placacy_uczestnik_id,
                        ),
                    ),
                    (
                        "na_rachunek",
                        xmeta_room::ToSqlValue::to_sql_value(&params.na_rachunek),
                    ),
                    (
                        "rachunek_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rachunek_id),
                    ),
                    (
                        "potwierdzenie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.potwierdzenie),
                    ),
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `zaleglosci`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZaleglosciRow {
    pub id: String,
    pub najem_id: String,
    pub trzy_okresy_od: String,
    pub uprzedzenie_wysylka_id: Option<String>,
    pub dodatkowy_termin_do: Option<String>,
    pub wynik: ZaleglosciWynik,
}

impl ZaleglosciRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZaleglosciRow {
            id: row.get("id")?,
            najem_id: row.get("najem_id")?,
            trzy_okresy_od: row.get("trzy_okresy_od")?,
            uprzedzenie_wysylka_id: row.get("uprzedzenie_wysylka_id")?,
            dodatkowy_termin_do: row.get("dodatkowy_termin_do")?,
            wynik: row.get("wynik")?,
        })
    }
}

/// Zaległości za co najmniej trzy pełne okresy z uprzedzeniem, terminem i wynikiem.
pub fn zaleglosci(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZaleglosciRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM najem_czynsz_zaleglosci
ORDER BY trzy_okresy_od, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(ZaleglosciRow::from_row).collect()
}

/// The parameters of `wpisz_zaleglosc`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszZalegloscParams {
    pub id: String,
    pub najem_id: String,
    pub trzy_okresy_od: String,
}

/// Dzień, w którym zaległość doszła do trzech pełnych okresów płatności; liczy program.
pub fn wpisz_zaleglosc(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszZalegloscParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO najem_czynsz_zaleglosci (id, najem_id, trzy_okresy_od, wynik)
VALUES (:id, :najem_id, :trzy_okresy_od, 'czeka')"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("najem_id", xmeta_room::ToSqlValue::to_sql_value(&params.najem_id)),
                    (
                        "trzy_okresy_od",
                        xmeta_room::ToSqlValue::to_sql_value(&params.trzy_okresy_od),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `wskaz_uprzedzenie_o_zaleglosci`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WskazUprzedzenieOZaleglosciParams {
    pub uprzedzenie_wysylka_id: String,
    pub id: String,
}

/// Pismo o zamiarze wypowiedzenia, jeszcze przed wysłaniem; wychodzi za zgodą człowieka.
pub fn wskaz_uprzedzenie_o_zaleglosci(
    room: &impl xmeta_room::MetaRoom,
    params: &WskazUprzedzenieOZaleglosciParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_czynsz_zaleglosci
SET uprzedzenie_wysylka_id = :uprzedzenie_wysylka_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "uprzedzenie_wysylka_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.uprzedzenie_wysylka_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `ustal_dodatkowy_termin_zaleglosci`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UstalDodatkowyTerminZaleglosciParams {
    pub dodatkowy_termin_do: String,
    pub id: String,
}

/// Koniec dodatkowego terminu na zapłatę, co najmniej miesiąc od doręczenia uprzedzenia.
pub fn ustal_dodatkowy_termin_zaleglosci(
    room: &impl xmeta_room::MetaRoom,
    params: &UstalDodatkowyTerminZaleglosciParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_czynsz_zaleglosci
SET dodatkowy_termin_do = :dodatkowy_termin_do
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "dodatkowy_termin_do",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dodatkowy_termin_do),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_wynik_zaleglosci`.
#[derive(Debug, Clone, PartialEq)]
pub struct OdnotujWynikZaleglosciParams {
    pub wynik: Wynik,
    pub id: String,
}

/// Najemca spłacił zaległość albo wynajmujący wypowiada najem po dodatkowym terminie.
pub fn odnotuj_wynik_zaleglosci(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWynikZaleglosciParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_czynsz_zaleglosci
SET wynik = :wynik
WHERE id = :id"#,
                parameters: Vec::from([
                    ("wynik", xmeta_room::ToSqlValue::to_sql_value(&params.wynik)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `zmiany_czynszu`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZmianyCzynszuRow {
    pub id: i64,
    pub najem_id: String,
    pub stara: i64,
    pub nowa: i64,
    pub pismo_wysylka_id: Option<String>,
    pub doreczono: Option<String>,
    pub obowiazuje_od: String,
    pub podstawa: String,
    pub aneks_id: Option<String>,
}

impl ZmianyCzynszuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZmianyCzynszuRow {
            id: row.get("id")?,
            najem_id: row.get("najem_id")?,
            stara: row.get("stara")?,
            nowa: row.get("nowa")?,
            pismo_wysylka_id: row.get("pismo_wysylka_id")?,
            doreczono: row.get("doreczono")?,
            obowiazuje_od: row.get("obowiazuje_od")?,
            podstawa: row.get("podstawa")?,
            aneks_id: row.get("aneks_id")?,
        })
    }
}

/// Zmiany czynszu w kolejności wpisania; bieżący czynsz to nowa kwota ostatniej.
pub fn zmiany_czynszu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZmianyCzynszuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM najem_czynsz_zmiany
ORDER BY najem_id, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(ZmianyCzynszuRow::from_row).collect()
}

/// The parameters of `wpisz_zmiane_czynszu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszZmianeCzynszuParams {
    pub najem_id: String,
    pub stara: i64,
    pub nowa: i64,
    pub pismo_wysylka_id: Option<String>,
    pub doreczono: Option<String>,
    pub obowiazuje_od: String,
    pub podstawa: String,
    pub aneks_id: Option<String>,
}

/// Zmiana czynszu: wypowiedzenie wysokości z doręczeniem albo obowiązujący aneks, z podstawą.
pub fn wpisz_zmiane_czynszu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszZmianeCzynszuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO najem_czynsz_zmiany (najem_id, stara, nowa, pismo_wysylka_id, doreczono, obowiazuje_od,
  podstawa, aneks_id)
VALUES (:najem_id, :stara, :nowa, :pismo_wysylka_id, :doreczono, :obowiazuje_od, :podstawa,
  :aneks_id)"#,
                parameters: Vec::from([
                    ("najem_id", xmeta_room::ToSqlValue::to_sql_value(&params.najem_id)),
                    ("stara", xmeta_room::ToSqlValue::to_sql_value(&params.stara)),
                    ("nowa", xmeta_room::ToSqlValue::to_sql_value(&params.nowa)),
                    (
                        "pismo_wysylka_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.pismo_wysylka_id),
                    ),
                    (
                        "doreczono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.doreczono),
                    ),
                    (
                        "obowiazuje_od",
                        xmeta_room::ToSqlValue::to_sql_value(&params.obowiazuje_od),
                    ),
                    ("podstawa", xmeta_room::ToSqlValue::to_sql_value(&params.podstawa)),
                    ("aneks_id", xmeta_room::ToSqlValue::to_sql_value(&params.aneks_id)),
                ]),
            },
        ],
    )
}
