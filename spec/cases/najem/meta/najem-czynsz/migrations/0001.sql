-- Pieniądze i media najmu: należności z harmonogramu, wpłaty, ich rozliczenie na należności, zmiany
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
