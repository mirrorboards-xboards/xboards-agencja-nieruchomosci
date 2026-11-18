-- Metryka pokoju wymiany: jeden wiersz na pokój, strony aktu i dziennik zamknięć.
-- Pokój wymiany jest wspólny dla biura na XEstates i kancelarii notarialnej na XNotariuszu,
-- jeden na transakcję i kancelarię. Metrykę zakłada biuro, gdy człowiek, który przyjął
-- transakcję, zgodzi się na otwarcie pokoju z kancelarią wybraną przez strony. Pokój nie ma
-- rdzenia teczki: teczka transakcji zostaje w biurze, a pokój wskazuje ją tekstem.
--
-- Kolumna strona_pokoju jest deklaracją piszącego. Organizację autora poświadcza serwer pokoju,
-- a porównuje ją aplikacja (decyzja T10); baza tego nie umie.
CREATE TABLE wymiana_metryka_wpis (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  -- Oznaczenie transakcji w biurze, np. „Transakcja 2026/017, ul. Polna 7 m. 12”.
  oznaczenie TEXT NOT NULL CHECK (oznaczenie <> ''),
  -- Teczka transakcji (albo najmu przy oświadczeniu najemcy) w biurze: identyfikator pokoju.
  transakcja_odnosnik TEXT NOT NULL CHECK (transakcja_odnosnik <> ''),
  -- Czynność, dla której biuro otworzyło pokój. Kolejne czynności tej transakcji u tej samej
  -- kancelarii, np. umowa przenosząca po warunkowej, wskazują swoje terminy aktu.
  czynnosc TEXT NOT NULL CHECK (
    czynnosc IN ('przedwstepna_akt', 'sprzedaz_warunkowa', 'przenoszaca', 'oswiadczenie_najemcy')
  ),
  kancelaria_nazwa TEXT NOT NULL CHECK (kancelaria_nazwa <> ''),
  kancelaria_adres TEXT NOT NULL CHECK (kancelaria_adres <> ''),
  -- Imię i nazwisko notariusza.
  notariusz TEXT NOT NULL CHECK (notariusz <> ''),
  -- Sygnatura sprawy w kancelarii; podaje ją kancelaria, gdy zechce.
  sygnatura_kancelarii TEXT CHECK (sygnatura_kancelarii <> ''),
  nieruchomosc_adres TEXT NOT NULL CHECK (nieruchomosc_adres <> ''),
  -- Lokal bez księgi wieczystej, np. przy oświadczeniu najemcy, nie ma numeru.
  numer_kw TEXT CHECK (numer_kw <> ''),
  otwarto TEXT NOT NULL
) STRICT;

-- Strony aktu: kto zawiera umowę albo składa oświadczenie, i czy biuro ją reprezentuje.
-- Biuro pisze w imieniu strony tylko z jej pełnomocnictwem (wymiana-dokumenty).
CREATE TABLE wymiana_metryka_strony (
  id TEXT PRIMARY KEY,
  -- Imiona i nazwisko albo nazwa strony.
  nazwa TEXT NOT NULL CHECK (nazwa <> ''),
  rola TEXT NOT NULL CHECK (rola IN ('sprzedajacy', 'kupujacy', 'wynajmujacy', 'najemca', 'inna')),
  reprezentowana_przez_biuro INTEGER NOT NULL CHECK (reprezentowana_przez_biuro IN (0, 1))
) STRICT;

-- Dziennik zamknięć: pokój zamyka się dopiero, gdy zamkną go obie strony, i otwiera ponownie
-- dopiero, gdy obie strony go otworzą (profil pokoju, rozdział „Cykl istnienia”).
CREATE TABLE wymiana_metryka_zamkniecia (
  -- Klucz nadany przez bazę jest dodatni; zob. wymiana_metryka_zamkniecia_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  strona_pokoju TEXT NOT NULL CHECK (strona_pokoju IN ('biuro', 'kancelaria')),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('zamkniecie', 'ponowne_otwarcie')),
  chwila TEXT NOT NULL
) STRICT;

-- Stan pokoju. Pełne zamknięcie to zamknięcie obu stron, a pełne otwarcie to ponowne otwarcie
-- obu stron. Każda strona zamyka i otwiera na przemian, zamyka tylko otwarty pokój i otwiera
-- tylko zamknięty, więc pokój jest zamknięty, gdy pełnych zamknięć jest więcej niż pełnych
-- otwarć. Jedno zamknięcie strony zostawia pokój otwarty, a jedno ponowne otwarcie — zamknięty.
CREATE VIEW wymiana_metryka_stan AS
SELECT
  CASE
    WHEN min(zamkniecia_biura, zamkniecia_kancelarii) > min(otwarcia_biura, otwarcia_kancelarii)
      THEN 'zamkniety'
    ELSE 'otwarty'
  END AS stan
FROM (
  SELECT
    count(CASE WHEN strona_pokoju = 'biuro' AND rodzaj = 'zamkniecie' THEN 1 END) AS zamkniecia_biura,
    count(CASE WHEN strona_pokoju = 'kancelaria' AND rodzaj = 'zamkniecie' THEN 1 END) AS zamkniecia_kancelarii,
    count(CASE WHEN strona_pokoju = 'biuro' AND rodzaj = 'ponowne_otwarcie' THEN 1 END) AS otwarcia_biura,
    count(CASE WHEN strona_pokoju = 'kancelaria' AND rodzaj = 'ponowne_otwarcie' THEN 1 END) AS otwarcia_kancelarii
  FROM wymiana_metryka_zamkniecia
);

-- Metryka mówi kancelarii, z kim i o jakiej transakcji rozmawia; po niej człowiek po stronie
-- biura rozpoznaje kancelarię. Zmienia się tylko raz: kancelaria podaje swoją sygnaturę.
CREATE TRIGGER wymiana_metryka_wpis_zmienia_tylko_sygnature
BEFORE UPDATE ON wymiana_metryka_wpis
WHEN OLD.sygnatura_kancelarii IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.oznaczenie IS NOT OLD.oznaczenie
  OR NEW.transakcja_odnosnik IS NOT OLD.transakcja_odnosnik
  OR NEW.czynnosc IS NOT OLD.czynnosc
  OR NEW.kancelaria_nazwa IS NOT OLD.kancelaria_nazwa
  OR NEW.kancelaria_adres IS NOT OLD.kancelaria_adres
  OR NEW.notariusz IS NOT OLD.notariusz
  OR NEW.nieruchomosc_adres IS NOT OLD.nieruchomosc_adres
  OR NEW.numer_kw IS NOT OLD.numer_kw
  OR NEW.otwarto IS NOT OLD.otwarto
BEGIN
  SELECT RAISE(ABORT, 'metryki pokoju się nie zmienia; kancelaria raz podaje tylko swoją sygnaturę');
END;

CREATE TRIGGER wymiana_metryka_zamkniecie_po_otwarciu
BEFORE INSERT ON wymiana_metryka_zamkniecia
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_metryka_wpis
)
BEGIN
  SELECT RAISE(ABORT, 'pokój bez metryki nie ma czego zamykać');
END;

-- Strona zamyka i otwiera na przemian, a zaczyna od zamknięcia.
CREATE TRIGGER wymiana_metryka_zamkniecia_na_przemian
BEFORE INSERT ON wymiana_metryka_zamkniecia
WHEN NEW.rodzaj = coalesce(
  (
    SELECT rodzaj
    FROM wymiana_metryka_zamkniecia
    WHERE strona_pokoju = NEW.strona_pokoju
    ORDER BY id DESC
    LIMIT 1
  ),
  'ponowne_otwarcie'
)
BEGIN
  SELECT RAISE(ABORT, 'strona pokoju zamyka i otwiera go na przemian, zaczynając od zamknięcia');
END;

-- Ponowne otwarcie wymaga zgody obu stron, więc strona, która już otworzyła zamknięty pokój,
-- czeka na drugą i nie zamyka go od nowa. Zamknięcie, na które druga strona nie odpowiada,
-- zostaje; o tym, czego jeszcze trzeba, strony piszą w wiadomościach.
CREATE TRIGGER wymiana_metryka_zamyka_otwarty_otwiera_zamkniety
BEFORE INSERT ON wymiana_metryka_zamkniecia
WHEN (
  SELECT stan
  FROM wymiana_metryka_stan
) = CASE NEW.rodzaj WHEN 'zamkniecie' THEN 'zamkniety' ELSE 'otwarty' END
BEGIN
  SELECT RAISE(ABORT, 'strona zamyka tylko otwarty pokój, a ponownie otwiera tylko zamknięty');
END;

CREATE TRIGGER wymiana_metryka_strony_bez_zmian
BEFORE UPDATE ON wymiana_metryka_strony
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

CREATE TRIGGER wymiana_metryka_zamkniecia_bez_zmian
BEFORE UPDATE ON wymiana_metryka_zamkniecia
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_metryka_wpis_bez_zastepowania
BEFORE INSERT ON wymiana_metryka_wpis
WHEN EXISTS (
  SELECT 1
  FROM wymiana_metryka_wpis
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_metryka_wpis_bez_usuwania
BEFORE DELETE ON wymiana_metryka_wpis
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_metryka_strony_bez_zastepowania
BEFORE INSERT ON wymiana_metryka_strony
WHEN EXISTS (
  SELECT 1
  FROM wymiana_metryka_strony
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_metryka_strony_bez_usuwania
BEFORE DELETE ON wymiana_metryka_strony
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER wymiana_metryka_zamkniecia_bez_zastepowania
BEFORE INSERT ON wymiana_metryka_zamkniecia
WHEN EXISTS (
  SELECT 1
  FROM wymiana_metryka_zamkniecia
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_metryka_zamkniecia_bez_usuwania
BEFORE DELETE ON wymiana_metryka_zamkniecia
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;
