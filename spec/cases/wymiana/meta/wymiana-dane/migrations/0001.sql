-- Dane do aktu w pokoju wymiany: dane osób, które biorą udział w akcie, dane nieruchomości,
-- rachunki do aktu i kredyt kupującego. Każdy wiersz jest szczegółem jednego wpisu
-- z wymiana_wpisy_wpisy, dopisuje się razem z nim i potem się nie zmienia; poprawka to nowy wpis.
--
-- Dane osób, nieruchomości i kredytu biuro przekazuje w imieniu strony, więc wymagają jej
-- wcześniejszego pełnomocnictwa (wymiana-dokumenty). Pokój nie trzyma danych, których akt nie
-- wymaga, np. dochodów kupującego ani numeru dokumentu tożsamości (R1).

-- Osoba albo podmiot, który bierze udział w akcie, z danymi z art. 92 § 1 pkt 4 i 4a PoN i innymi,
-- o które prosi kancelaria, np. stanem cywilnym. Każda dana pochodzi z dokumentu w teczce.
CREATE TABLE wymiana_dane_osoby (
  wpis_id TEXT PRIMARY KEY REFERENCES wymiana_wpisy_wpisy (id),
  -- Strona aktu, do której osoba należy i w której imieniu biuro pisze.
  strona_id TEXT NOT NULL REFERENCES wymiana_metryka_strony (id),
  -- Sama strona albo ktoś, kto działa w jej imieniu.
  udzial TEXT NOT NULL CHECK (udzial IN ('strona', 'przedstawiciel', 'pelnomocnik')),
  rodzaj_podmiotu TEXT NOT NULL CHECK (rodzaj_podmiotu IN ('osoba_fizyczna', 'podmiot')),
  imiona TEXT CHECK (imiona <> ''),
  nazwisko TEXT CHECK (nazwisko <> ''),
  imiona_rodzicow TEXT CHECK (imiona_rodzicow <> ''),
  miejsce_zamieszkania TEXT CHECK (miejsce_zamieszkania <> ''),
  -- Nazwa i siedziba osoby prawnej albo innego podmiotu.
  nazwa TEXT CHECK (nazwa <> ''),
  siedziba TEXT CHECK (siedziba <> ''),
  -- Obywatelstwo cudzoziemca, który nabywa nieruchomość (art. 92 § 1 pkt 4a PoN).
  obywatelstwo TEXT CHECK (obywatelstwo <> ''),
  -- Np. „zamężna, wspólność ustawowa”, gdy kancelaria o to prosi.
  stan_cywilny TEXT CHECK (stan_cywilny <> ''),
  -- Dokument w teczce transakcji, z którego pochodzą dane: identyfikator pokoju i dokumentu.
  zrodlo_dokument_odnosnik TEXT NOT NULL CHECK (zrodlo_dokument_odnosnik <> ''),

  CONSTRAINT osoba_ma_imiona_nazwisko_i_zamieszkanie CHECK (
    rodzaj_podmiotu <> 'osoba_fizyczna'
    OR (imiona IS NOT NULL AND nazwisko IS NOT NULL AND miejsce_zamieszkania IS NOT NULL)
  ),
  -- Imiona rodziców akt podaje przy osobach fizycznych, które biorą udział w akcie, a nie przy
  -- tych, które działają w cudzym imieniu.
  CONSTRAINT strona_fizyczna_ma_imiona_rodzicow CHECK (
    (rodzaj_podmiotu = 'osoba_fizyczna' AND udzial = 'strona') = (imiona_rodzicow IS NOT NULL)
  ),
  CONSTRAINT podmiot_ma_nazwe_i_siedzibe CHECK (
    (rodzaj_podmiotu = 'podmiot') = (nazwa IS NOT NULL AND siedziba IS NOT NULL)
  ),
  CONSTRAINT podmiot_bez_danych_osoby CHECK (
    rodzaj_podmiotu = 'osoba_fizyczna'
    OR (
      imiona IS NULL
      AND nazwisko IS NULL
      AND miejsce_zamieszkania IS NULL
      AND obywatelstwo IS NULL
      AND stan_cywilny IS NULL
    )
  ),
  CONSTRAINT podmiot_tylko_strona CHECK (rodzaj_podmiotu = 'osoba_fizyczna' OR udzial = 'strona')
) STRICT;

-- Oznaczenie nieruchomości z odpisu księgi wieczystej i z ewidencji gruntów i budynków.
CREATE TABLE wymiana_dane_nieruchomosci (
  wpis_id TEXT PRIMARY KEY REFERENCES wymiana_wpisy_wpisy (id),
  -- Strona, w której imieniu biuro przekazuje dane, zwykle sprzedający.
  strona_id TEXT NOT NULL REFERENCES wymiana_metryka_strony (id),
  numer_kw TEXT CHECK (numer_kw <> ''),
  -- Np. „lokal mieszkalny nr 12 w budynku przy ul. Polnej 7, pow. 54,20 m²”.
  oznaczenie TEXT NOT NULL CHECK (oznaczenie <> ''),
  -- Numery działek z obrębem, gdy nieruchomość je ma.
  dzialki TEXT CHECK (dzialki <> ''),
  -- Skąd oznaczenie, np. „odpis zupełny KW z 2026-11-03, wypis z rejestru gruntów z 2026-11-04”.
  zrodlo TEXT NOT NULL CHECK (zrodlo <> '')
) STRICT;

-- Rachunek, który ma trafić do aktu. Od biura wpis mówi tylko, że rachunek sprawdzono i że
-- człowiek go zatwierdził (R4), i wskazuje tę decyzję w teczce transakcji. Od kancelarii,
-- np. rachunek do zapłaty podatku, jest informacją: Koordynator potwierdza go innym kanałem
-- i oddaje Kontrolerowi.
CREATE TABLE wymiana_dane_rachunki (
  wpis_id TEXT PRIMARY KEY REFERENCES wymiana_wpisy_wpisy (id),
  numer TEXT NOT NULL CHECK (numer <> ''),
  posiadacz TEXT NOT NULL CHECK (posiadacz <> ''),
  cel TEXT NOT NULL CHECK (
    cel IN ('zaplata_ceny', 'zadatek', 'splata_hipoteki', 'depozyt_notarialny', 'podatki_i_oplaty', 'inny')
  ),
  -- Decyzja dopuszczenie_rachunku z teczki transakcji: identyfikator pokoju i decyzji.
  decyzja_odnosnik TEXT CHECK (decyzja_odnosnik <> '')
) STRICT;

-- Kredyt kupującego: tylko to, czego potrzebuje akt, gdy kancelaria o to prosi.
CREATE TABLE wymiana_dane_kredyty (
  wpis_id TEXT PRIMARY KEY REFERENCES wymiana_wpisy_wpisy (id),
  -- Kredytobiorca, w którego imieniu biuro przekazuje dane.
  strona_id TEXT NOT NULL REFERENCES wymiana_metryka_strony (id),
  bank TEXT NOT NULL CHECK (bank <> ''),
  -- Kwota kredytu z decyzji banku, w groszach.
  kwota INTEGER NOT NULL CHECK (kwota > 0),
  warunki_wyplaty TEXT CHECK (warunki_wyplaty <> '')
) STRICT;

-- Szczegół dopisuje się w tej samej operacji co jego wpis. Późniejszy dopisek zmieniałby wpis,
-- który druga strona już przeczytała, także w zamkniętym pokoju.
CREATE TRIGGER wymiana_dane_osoby_do_swojego_wpisu
BEFORE INSERT ON wymiana_dane_osoby
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
    AND rodzaj = 'dane_osoby'
    AND rowid = (
      SELECT max(rowid)
      FROM wymiana_wpisy_wpisy
    )
)
BEGIN
  SELECT RAISE(ABORT, 'dane osoby należą do najnowszego wpisu dane_osoby');
END;

CREATE TRIGGER wymiana_dane_nieruchomosci_do_swojego_wpisu
BEFORE INSERT ON wymiana_dane_nieruchomosci
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
    AND rodzaj = 'dane_nieruchomosci'
    AND rowid = (
      SELECT max(rowid)
      FROM wymiana_wpisy_wpisy
    )
)
BEGIN
  SELECT RAISE(ABORT, 'dane nieruchomości należą do najnowszego wpisu dane_nieruchomosci');
END;

CREATE TRIGGER wymiana_dane_rachunki_do_swojego_wpisu
BEFORE INSERT ON wymiana_dane_rachunki
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
    AND rodzaj = 'rachunek'
    AND rowid = (
      SELECT max(rowid)
      FROM wymiana_wpisy_wpisy
    )
)
BEGIN
  SELECT RAISE(ABORT, 'rachunek należy do najnowszego wpisu rachunek');
END;

CREATE TRIGGER wymiana_dane_kredyty_do_swojego_wpisu
BEFORE INSERT ON wymiana_dane_kredyty
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
    AND rodzaj = 'kredyt'
    AND rowid = (
      SELECT max(rowid)
      FROM wymiana_wpisy_wpisy
    )
)
BEGIN
  SELECT RAISE(ABORT, 'kredyt należy do najnowszego wpisu kredyt');
END;

-- Pisanie bez pełnomocnictwa: dane strony biuro przekazuje dopiero po jej pełnomocnictwie.
CREATE TRIGGER wymiana_dane_osoby_wymagaja_pelnomocnictwa
BEFORE INSERT ON wymiana_dane_osoby
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_dokumenty_wpisy
  WHERE rodzaj = 'pelnomocnictwo'
    AND strona_id = NEW.strona_id
)
BEGIN
  SELECT RAISE(ABORT, 'dane strony wymagają wpisanego wcześniej pełnomocnictwa tej strony');
END;

CREATE TRIGGER wymiana_dane_nieruchomosci_wymagaja_pelnomocnictwa
BEFORE INSERT ON wymiana_dane_nieruchomosci
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_dokumenty_wpisy
  WHERE rodzaj = 'pelnomocnictwo'
    AND strona_id = NEW.strona_id
)
BEGIN
  SELECT RAISE(ABORT, 'dane strony wymagają wpisanego wcześniej pełnomocnictwa tej strony');
END;

CREATE TRIGGER wymiana_dane_kredyty_wymagaja_pelnomocnictwa
BEFORE INSERT ON wymiana_dane_kredyty
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_dokumenty_wpisy
  WHERE rodzaj = 'pelnomocnictwo'
    AND strona_id = NEW.strona_id
)
BEGIN
  SELECT RAISE(ABORT, 'dane strony wymagają wpisanego wcześniej pełnomocnictwa tej strony');
END;

-- Rachunek od biura stoi na decyzji człowieka z teczki; decyzje biura nie są sprawą kancelarii.
CREATE TRIGGER wymiana_dane_rachunek_biura_z_decyzja
BEFORE INSERT ON wymiana_dane_rachunki
WHEN (
  SELECT strona = 'biuro'
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
) IS NOT (NEW.decyzja_odnosnik IS NOT NULL)
BEGIN
  SELECT RAISE(ABORT, 'rachunek od biura ma odnośnik do decyzji, a rachunek od kancelarii jest informacją bez decyzji');
END;

CREATE TRIGGER wymiana_dane_osoby_bez_zmian
BEFORE UPDATE ON wymiana_dane_osoby
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

CREATE TRIGGER wymiana_dane_nieruchomosci_bez_zmian
BEFORE UPDATE ON wymiana_dane_nieruchomosci
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

CREATE TRIGGER wymiana_dane_rachunki_bez_zmian
BEFORE UPDATE ON wymiana_dane_rachunki
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

CREATE TRIGGER wymiana_dane_kredyty_bez_zmian
BEFORE UPDATE ON wymiana_dane_kredyty
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_dane_osoby_bez_zastepowania
BEFORE INSERT ON wymiana_dane_osoby
WHEN EXISTS (
  SELECT 1
  FROM wymiana_dane_osoby
  WHERE wpis_id = NEW.wpis_id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_dane_osoby_bez_usuwania
BEFORE DELETE ON wymiana_dane_osoby
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_dane_nieruchomosci_bez_zastepowania
BEFORE INSERT ON wymiana_dane_nieruchomosci
WHEN EXISTS (
  SELECT 1
  FROM wymiana_dane_nieruchomosci
  WHERE wpis_id = NEW.wpis_id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_dane_nieruchomosci_bez_usuwania
BEFORE DELETE ON wymiana_dane_nieruchomosci
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_dane_rachunki_bez_zastepowania
BEFORE INSERT ON wymiana_dane_rachunki
WHEN EXISTS (
  SELECT 1
  FROM wymiana_dane_rachunki
  WHERE wpis_id = NEW.wpis_id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_dane_rachunki_bez_usuwania
BEFORE DELETE ON wymiana_dane_rachunki
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_dane_kredyty_bez_zastepowania
BEFORE INSERT ON wymiana_dane_kredyty
WHEN EXISTS (
  SELECT 1
  FROM wymiana_dane_kredyty
  WHERE wpis_id = NEW.wpis_id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_dane_kredyty_bez_usuwania
BEFORE DELETE ON wymiana_dane_kredyty
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;
