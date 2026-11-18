-- Kartoteka klientów biura: osoby i podmioty, które zawarły z biurem umowę pośrednictwa albo
-- o zarządzanie, z historią ich danych i listą umów z biurem. Służy do rozpoznania klienta, który
-- wraca, i do ciągłości danych między sprawami. Pokój jest jeden na biuro.
--
-- Piszą Kierownik pośrednictwa i Kierownik najmu, każdy ze źródłem w teczce: umową, aneksem albo
-- dokumentem. Czytają Kierownik biura i Kierownik kontroli. Kto co zmienia, pilnują uprawnienia
-- pokoju i aplikacja; baza nie wie, kto pisze.
--
-- Zmiana danych nie usuwa poprzedniej wartości: dane klienta mają wersje, z których jedna jest
-- aktualna, a adresy, kontakty i osoby mają daty od i do. Dzięki temu stary numer telefonu z karty
-- Recepcji nadal wskazuje klienta. Wierszy się nie zmienia; wyjątkiem są koniec (do) adresu,
-- kontaktu i osoby oraz utrata aktualności wersji danych.
--
-- Kartoteka nie ma kolumn na ślad przeciwdziałania praniu pieniędzy, oceny ani dokumenty
-- tożsamości (art. 54 ust. 1 AML): to wynika z budowy tabel. Inne pokoje wskazuje tekstem
-- (odnośniki do teczek), bez klucza obcego.
CREATE TABLE klienci_kartoteka_wpisy (
  id TEXT PRIMARY KEY,
  rodzaj_podmiotu TEXT NOT NULL CHECK (rodzaj_podmiotu IN ('osoba_fizyczna', 'osoba_prawna', 'jednostka_bez_osobowosci')),
  -- Kierownik działu, który wpisał klienta, i dzień podpisania pierwszej umowy.
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL
) STRICT;

-- Dane klienta w wersjach (migawkach). Nowa wersja powstaje przy każdej zmianie danych
-- z dokumentem i odbiera aktualność poprzedniej, więc aktualna wersja jest jedna.
CREATE TABLE klienci_kartoteka_dane (
  -- Klucz nadany przez bazę jest dodatni; zob. klienci_kartoteka_dane_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  klient_id TEXT NOT NULL REFERENCES klienci_kartoteka_wpisy (id),
  -- Dzień, od którego dane obowiązują, np. dzień aneksu z nowym nazwiskiem.
  od TEXT NOT NULL,
  aktualne INTEGER NOT NULL CHECK (aktualne IN (0, 1)),

  -- Osoba fizyczna. PESEL odróżnia dwie osoby o tym samym imieniu i nazwisku.
  imiona TEXT,
  nazwisko TEXT,
  pesel TEXT,

  -- Podmiot, a także osoba fizyczna, która zawarła umowę jako przedsiębiorca (firma, NIP, wpis
  -- w CEIDG).
  nazwa TEXT,
  forma_prawna TEXT,
  krs TEXT,
  ceidg INTEGER NOT NULL DEFAULT 0 CHECK (ceidg IN (0, 1)),
  nip TEXT,
  regon TEXT,
  firma TEXT,

  -- Dokument w teczce, z którego kierownik przepisał dane, i krótki opis, np. „aneks nr 1”.
  zrodlo_odnosnik TEXT NOT NULL CHECK (zrodlo_odnosnik <> ''),
  zrodlo_opis TEXT,
  wpisal TEXT NOT NULL,

  CONSTRAINT pesel_ma_11_cyfr CHECK (pesel IS NULL OR (length(pesel) = 11 AND pesel NOT GLOB '*[^0-9]*')),
  CONSTRAINT krs_albo_ceidg CHECK (krs IS NULL OR ceidg = 0)
) STRICT;

-- Adres zamieszkania i adres do korespondencji osoby albo siedziba podmiotu. Nowy adres tego
-- samego rodzaju zamyka poprzedni datą do.
CREATE TABLE klienci_kartoteka_adresy (
  -- Klucz nadany przez bazę jest dodatni; zob. klienci_kartoteka_adresy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  klient_id TEXT NOT NULL REFERENCES klienci_kartoteka_wpisy (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('zamieszkanie', 'do_korespondencji', 'siedziba')),
  -- Adres tak jak w umowie, np. „ul. Polna 7 m. 12, 50-001 Wrocław”.
  adres TEXT NOT NULL CHECK (adres <> ''),
  od TEXT NOT NULL,
  do TEXT,
  zrodlo_odnosnik TEXT NOT NULL CHECK (zrodlo_odnosnik <> ''),

  CONSTRAINT koniec_nie_przed_poczatkiem CHECK (do IS NULL OR do >= od)
) STRICT;

-- Telefony i adresy poczty elektronicznej, po których Kierownik biura rozpoznaje klienta na karcie
-- Recepcji. Preferowany kontakt to kanał, którym klient chce rozmawiać z biurem.
CREATE TABLE klienci_kartoteka_kontakty (
  -- Klucz nadany przez bazę jest dodatni; zob. klienci_kartoteka_kontakty_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  klient_id TEXT NOT NULL REFERENCES klienci_kartoteka_wpisy (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('telefon', 'email')),
  wartosc TEXT NOT NULL CHECK (wartosc <> ''),
  preferowany INTEGER NOT NULL DEFAULT 0 CHECK (preferowany IN (0, 1)),
  od TEXT NOT NULL,
  do TEXT,
  zrodlo_odnosnik TEXT NOT NULL CHECK (zrodlo_odnosnik <> ''),

  CONSTRAINT koniec_nie_przed_poczatkiem CHECK (do IS NULL OR do >= od)
) STRICT;

-- Osoby, które działają za podmiot (podpisały umowę), z podstawą umocowania, i osoby kontaktowe
-- podmiotu.
CREATE TABLE klienci_kartoteka_osoby (
  -- Klucz nadany przez bazę jest dodatni; zob. klienci_kartoteka_osoby_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  klient_id TEXT NOT NULL REFERENCES klienci_kartoteka_wpisy (id),
  funkcja TEXT NOT NULL CHECK (funkcja IN ('dziala_za_podmiot', 'kontaktowa')),
  imiona TEXT NOT NULL CHECK (imiona <> ''),
  nazwisko TEXT NOT NULL CHECK (nazwisko <> ''),
  -- Telefon albo adres poczty elektronicznej osoby kontaktowej.
  kontakt TEXT,
  -- Odpis z KRS albo pełnomocnictwo w teczce.
  podstawa_odnosnik TEXT,
  od TEXT NOT NULL,
  do TEXT,
  zrodlo_odnosnik TEXT NOT NULL CHECK (zrodlo_odnosnik <> ''),

  CONSTRAINT dzialajacy_za_podmiot_ma_podstawe CHECK (funkcja <> 'dziala_za_podmiot' OR podstawa_odnosnik IS NOT NULL),
  CONSTRAINT koniec_nie_przed_poczatkiem CHECK (do IS NULL OR do >= od)
) STRICT;

-- Umowy klienta z biurem, każda osobnym wpisem. Odnośnik do teczki nie daje do niej dostępu.
-- Umowa pośrednictwa kupna albo najmu nie ma jeszcze nieruchomości.
CREATE TABLE klienci_kartoteka_umowy (
  id TEXT PRIMARY KEY,
  klient_id TEXT NOT NULL REFERENCES klienci_kartoteka_wpisy (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('posrednictwo_sprzedazy', 'posrednictwo_kupna', 'posrednictwo_wynajmu', 'posrednictwo_najmu', 'zarzadzanie_najem_dlugoterminowy', 'zarzadzanie_najem_krotkoterminowy')),
  -- Dział, którego kierownik wpisał umowę i zmienia jej stan.
  dzial TEXT NOT NULL CHECK (dzial IN ('posrednictwo', 'najem')),
  nieruchomosc_adres TEXT,
  numer_kw TEXT,
  -- Teczka, w której umowę zawarto (typ_pokoju z jej metryki), i jej oznaczenie.
  teczka_odnosnik TEXT NOT NULL CHECK (teczka_odnosnik <> ''),
  typ_teczki TEXT NOT NULL CHECK (typ_teczki IN ('oferta', 'poszukiwanie', 'najem', 'lokal_krotkoterminowy')),
  oznaczenie TEXT NOT NULL CHECK (oznaczenie <> ''),
  -- Umowa w teczce (sprawa_umowy_wpisy).
  umowa_odnosnik TEXT NOT NULL CHECK (umowa_odnosnik <> ''),
  podpisal_za_biuro TEXT NOT NULL,
  -- Dzień zawarcia i, przy umowie na czas oznaczony, dzień jej końca.
  zawarto TEXT NOT NULL,
  koniec TEXT,
  -- Art. 22¹ KC; od tego zależy prawo odstąpienia (R8).
  konsument INTEGER NOT NULL CHECK (konsument IN (0, 1)),
  -- Umowa pozwala biuru pośredniczyć także dla drugiej strony (R1).
  obie_strony INTEGER NOT NULL CHECK (obie_strony IN (0, 1)),
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,

  CONSTRAINT dzial_pasuje_do_rodzaju CHECK (
    dzial = CASE
      WHEN rodzaj LIKE 'posrednictwo%' THEN 'posrednictwo'
      ELSE 'najem'
    END
  ),
  -- Umowę pośrednictwa zawiera się w teczce oferty albo poszukiwania, a umowę o zarządzanie
  -- w teczce najmu albo lokalu krótkoterminowego.
  CONSTRAINT typ_teczki_pasuje_do_rodzaju CHECK (
    typ_teczki = CASE rodzaj
      WHEN 'posrednictwo_sprzedazy' THEN 'oferta'
      WHEN 'posrednictwo_wynajmu' THEN 'oferta'
      WHEN 'posrednictwo_kupna' THEN 'poszukiwanie'
      WHEN 'posrednictwo_najmu' THEN 'poszukiwanie'
      WHEN 'zarzadzanie_najem_dlugoterminowy' THEN 'najem'
      WHEN 'zarzadzanie_najem_krotkoterminowy' THEN 'lokal_krotkoterminowy'
    END
  ),
  CONSTRAINT nieruchomosc_tylko_przy_umowie_o_nieruchomosc CHECK (
    CASE
      WHEN rodzaj IN ('posrednictwo_kupna', 'posrednictwo_najmu') THEN nieruchomosc_adres IS NULL AND numer_kw IS NULL
      ELSE nieruchomosc_adres IS NOT NULL
    END
  ),
  CONSTRAINT obie_strony_tylko_przy_posrednictwie CHECK (obie_strony = 0 OR dzial = 'posrednictwo'),
  CONSTRAINT koniec_nie_przed_zawarciem CHECK (koniec IS NULL OR koniec >= zawarto)
) STRICT;

-- Stany umowy z datą i źródłem, np. oświadczeniem konsumenta o odstąpieniu z teczki. Umowa
-- obowiązuje od zawarcia, a potem przechodzi do jednego stanu końcowego.
CREATE TABLE klienci_kartoteka_umowy_stany (
  -- Klucz nadany przez bazę jest dodatni; zob. klienci_kartoteka_umowy_stany_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  umowa_id TEXT NOT NULL REFERENCES klienci_kartoteka_umowy (id),
  stan TEXT NOT NULL CHECK (stan IN ('obowiazuje', 'odstapiono', 'wypowiedziana', 'wygasla', 'wykonana')),
  od TEXT NOT NULL,
  -- Umowa w teczce przy wpisie, zdarzenie umowy (sprawa_umowy_zdarzenia) albo raport końcowy.
  zrodlo_odnosnik TEXT NOT NULL CHECK (zrodlo_odnosnik <> ''),
  wpisal TEXT NOT NULL
) STRICT;

-- Kiedy i jak biuro poinformowało klienta, że trzyma jego dane w kartotece, w jakim celu i jak
-- długo (art. 13 RODO). Informację wysłał agent sprawy, więc wpis wskazuje wysyłkę w teczce.
CREATE TABLE klienci_kartoteka_informacje (
  -- Klucz nadany przez bazę jest dodatni; zob. klienci_kartoteka_informacje_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  klient_id TEXT NOT NULL REFERENCES klienci_kartoteka_wpisy (id),
  poinformowano TEXT NOT NULL,
  sposob TEXT NOT NULL CHECK (sposob IN ('email', 'sms', 'portal', 'list', 'osobiscie')),
  -- Wysyłka w teczce (sprawa_korespondencja_wysylki).
  wysylka_odnosnik TEXT NOT NULL CHECK (wysylka_odnosnik <> '')
) STRICT;

-- Dane osoby fizycznej i dane podmiotu --------------------------------------------------------

CREATE TRIGGER klienci_kartoteka_dane_osoby_maja_imie_i_nazwisko
BEFORE INSERT ON klienci_kartoteka_dane
WHEN (
  SELECT rodzaj_podmiotu = 'osoba_fizyczna'
  FROM klienci_kartoteka_wpisy
  WHERE id = NEW.klient_id
)
  AND (NEW.imiona IS NULL OR NEW.nazwisko IS NULL)
BEGIN
  SELECT RAISE(ABORT, 'dane osoby fizycznej mają imiona i nazwisko');
END;

CREATE TRIGGER klienci_kartoteka_dane_podmiotu_bez_danych_osoby
BEFORE INSERT ON klienci_kartoteka_dane
WHEN (
  SELECT rodzaj_podmiotu <> 'osoba_fizyczna'
  FROM klienci_kartoteka_wpisy
  WHERE id = NEW.klient_id
)
  AND (NEW.nazwa IS NULL OR NEW.imiona IS NOT NULL OR NEW.nazwisko IS NOT NULL OR NEW.pesel IS NOT NULL)
BEGIN
  SELECT RAISE(ABORT, 'dane podmiotu mają nazwę i nie mają imion, nazwiska ani PESEL');
END;

-- Kolejna umowa trafia do istniejącego klienta; ten sam PESEL u drugiego klienta to drugi wpis tej
-- samej osoby.
CREATE TRIGGER klienci_kartoteka_pesel_jednego_klienta
BEFORE INSERT ON klienci_kartoteka_dane
WHEN NEW.pesel IS NOT NULL
  AND EXISTS (
    SELECT 1
    FROM klienci_kartoteka_dane
    WHERE pesel = NEW.pesel
      AND klient_id <> NEW.klient_id
  )
BEGIN
  SELECT RAISE(ABORT, 'ten PESEL ma już inny klient kartoteki; kolejna umowa trafia do istniejącego klienta');
END;

-- Wersje danych ------------------------------------------------------------------------------------

-- Aktualność odbiera tylko nowa wersja, więc wersja powstaje aktualna.
CREATE TRIGGER klienci_kartoteka_dane_powstaja_aktualne
BEFORE INSERT ON klienci_kartoteka_dane
WHEN NEW.aktualne <> 1
BEGIN
  SELECT RAISE(ABORT, 'wersja danych klienta powstaje jako aktualna');
END;

CREATE TRIGGER klienci_kartoteka_dane_nie_starsze_niz_aktualne
BEFORE INSERT ON klienci_kartoteka_dane
WHEN NEW.od < (
  SELECT od
  FROM klienci_kartoteka_dane
  WHERE klient_id = NEW.klient_id
    AND aktualne = 1
)
BEGIN
  SELECT RAISE(ABORT, 'nowa wersja danych klienta nie obowiązuje wcześniej niż aktualna');
END;

CREATE TRIGGER klienci_kartoteka_nowe_dane_zeruja_poprzednie
AFTER INSERT ON klienci_kartoteka_dane
BEGIN
  UPDATE klienci_kartoteka_dane
  SET aktualne = 0
  WHERE klient_id = NEW.klient_id
    AND id <> NEW.id
    AND aktualne = 1;
END;

-- Wersji się nie zmienia. Jedyną zmianą jest utrata aktualności, gdy klient ma nowszą aktualną
-- wersję; inaczej klient zostałby bez aktualnych danych.
CREATE TRIGGER klienci_kartoteka_dane_bez_zmian
BEFORE UPDATE ON klienci_kartoteka_dane
WHEN NEW.id IS NOT OLD.id
  OR NEW.klient_id IS NOT OLD.klient_id
  OR NEW.od IS NOT OLD.od
  OR NEW.imiona IS NOT OLD.imiona
  OR NEW.nazwisko IS NOT OLD.nazwisko
  OR NEW.pesel IS NOT OLD.pesel
  OR NEW.nazwa IS NOT OLD.nazwa
  OR NEW.forma_prawna IS NOT OLD.forma_prawna
  OR NEW.krs IS NOT OLD.krs
  OR NEW.ceidg IS NOT OLD.ceidg
  OR NEW.nip IS NOT OLD.nip
  OR NEW.regon IS NOT OLD.regon
  OR NEW.firma IS NOT OLD.firma
  OR NEW.zrodlo_odnosnik IS NOT OLD.zrodlo_odnosnik
  OR NEW.zrodlo_opis IS NOT OLD.zrodlo_opis
  OR NEW.wpisal IS NOT OLD.wpisal
  OR (
    NEW.aktualne IS NOT OLD.aktualne
    AND NOT (
      OLD.aktualne = 1
      AND NEW.aktualne = 0
      AND EXISTS (
        SELECT 1
        FROM klienci_kartoteka_dane
        WHERE klient_id = OLD.klient_id
          AND id <> OLD.id
          AND aktualne = 1
      )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'danych klienta się nie zmienia; zmiana danych to nowa wersja');
END;

-- Adresy, kontakty i osoby ----------------------------------------------------------------------

-- Miejsce zamieszkania ma osoba fizyczna, siedzibę podmiot.
CREATE TRIGGER klienci_kartoteka_rodzaj_adresu_pasuje_do_podmiotu
BEFORE INSERT ON klienci_kartoteka_adresy
WHEN (
  SELECT
    (NEW.rodzaj = 'zamieszkanie' AND rodzaj_podmiotu <> 'osoba_fizyczna')
    OR (NEW.rodzaj = 'siedziba' AND rodzaj_podmiotu = 'osoba_fizyczna')
  FROM klienci_kartoteka_wpisy
  WHERE id = NEW.klient_id
)
BEGIN
  SELECT RAISE(ABORT, 'miejsce zamieszkania ma tylko osoba fizyczna, a siedzibę tylko podmiot');
END;

-- Bieżący adres danego rodzaju jest jeden; nowy adres wpisuje się po zamknięciu poprzedniego.
CREATE TRIGGER klienci_kartoteka_jeden_biezacy_adres_rodzaju
BEFORE INSERT ON klienci_kartoteka_adresy
WHEN EXISTS (
  SELECT 1
  FROM klienci_kartoteka_adresy
  WHERE klient_id = NEW.klient_id
    AND rodzaj = NEW.rodzaj
    AND do IS NULL
)
BEGIN
  SELECT RAISE(ABORT, 'klient ma już bieżący adres tego rodzaju; nowy adres wpisuje się po zamknięciu poprzedniego');
END;

-- Adres, kontakt i osoba powstają bieżące, a koniec przechodzi przez regułę zmiany.
CREATE TRIGGER klienci_kartoteka_adres_powstaje_biezacy
BEFORE INSERT ON klienci_kartoteka_adresy
WHEN NEW.do IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'adres, kontakt i osoba powstają jako bieżące');
END;

CREATE TRIGGER klienci_kartoteka_kontakt_powstaje_biezacy
BEFORE INSERT ON klienci_kartoteka_kontakty
WHEN NEW.do IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'adres, kontakt i osoba powstają jako bieżące');
END;

CREATE TRIGGER klienci_kartoteka_osoba_powstaje_biezaca
BEFORE INSERT ON klienci_kartoteka_osoby
WHEN NEW.do IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'adres, kontakt i osoba powstają jako bieżące');
END;

CREATE TRIGGER klienci_kartoteka_osoby_tylko_podmiotu
BEFORE INSERT ON klienci_kartoteka_osoby
WHEN (
  SELECT rodzaj_podmiotu = 'osoba_fizyczna'
  FROM klienci_kartoteka_wpisy
  WHERE id = NEW.klient_id
)
BEGIN
  SELECT RAISE(ABORT, 'osoby działające za klienta i osoby kontaktowe ma tylko podmiot');
END;

-- Adres, kontakt i osoba mogą się tylko raz zakończyć; poprzednia wartość zostaje jako historia.
CREATE TRIGGER klienci_kartoteka_adresy_bez_zmian
BEFORE UPDATE ON klienci_kartoteka_adresy
WHEN NEW.id IS NOT OLD.id
  OR NEW.klient_id IS NOT OLD.klient_id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.adres IS NOT OLD.adres
  OR NEW.od IS NOT OLD.od
  OR NEW.zrodlo_odnosnik IS NOT OLD.zrodlo_odnosnik
  OR (OLD.do IS NOT NULL AND NEW.do IS NOT OLD.do)
BEGIN
  SELECT RAISE(ABORT, 'adresu, kontaktu ani osoby się nie zmienia; mogą się tylko raz zakończyć');
END;

CREATE TRIGGER klienci_kartoteka_kontakty_bez_zmian
BEFORE UPDATE ON klienci_kartoteka_kontakty
WHEN NEW.id IS NOT OLD.id
  OR NEW.klient_id IS NOT OLD.klient_id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.wartosc IS NOT OLD.wartosc
  OR NEW.preferowany IS NOT OLD.preferowany
  OR NEW.od IS NOT OLD.od
  OR NEW.zrodlo_odnosnik IS NOT OLD.zrodlo_odnosnik
  OR (OLD.do IS NOT NULL AND NEW.do IS NOT OLD.do)
BEGIN
  SELECT RAISE(ABORT, 'adresu, kontaktu ani osoby się nie zmienia; mogą się tylko raz zakończyć');
END;

CREATE TRIGGER klienci_kartoteka_osoby_bez_zmian
BEFORE UPDATE ON klienci_kartoteka_osoby
WHEN NEW.id IS NOT OLD.id
  OR NEW.klient_id IS NOT OLD.klient_id
  OR NEW.funkcja IS NOT OLD.funkcja
  OR NEW.imiona IS NOT OLD.imiona
  OR NEW.nazwisko IS NOT OLD.nazwisko
  OR NEW.kontakt IS NOT OLD.kontakt
  OR NEW.podstawa_odnosnik IS NOT OLD.podstawa_odnosnik
  OR NEW.od IS NOT OLD.od
  OR NEW.zrodlo_odnosnik IS NOT OLD.zrodlo_odnosnik
  OR (OLD.do IS NOT NULL AND NEW.do IS NOT OLD.do)
BEGIN
  SELECT RAISE(ABORT, 'adresu, kontaktu ani osoby się nie zmienia; mogą się tylko raz zakończyć');
END;

-- Umowy i ich stany ----------------------------------------------------------------------------

-- Art. 22¹ KC: konsumentem jest tylko osoba fizyczna.
CREATE TRIGGER klienci_kartoteka_konsument_tylko_osoba_fizyczna
BEFORE INSERT ON klienci_kartoteka_umowy
WHEN NEW.konsument = 1
  AND (
    SELECT rodzaj_podmiotu <> 'osoba_fizyczna'
    FROM klienci_kartoteka_wpisy
    WHERE id = NEW.klient_id
  )
BEGIN
  SELECT RAISE(ABORT, 'umowę jako konsument zawiera tylko osoba fizyczna');
END;

-- Umowa obowiązuje od zawarcia, potem przechodzi do jednego stanu końcowego, a stan końcowy jest
-- ostateczny.
CREATE TRIGGER klienci_kartoteka_stan_umowy_tylko_naprzod
BEFORE INSERT ON klienci_kartoteka_umowy_stany
WHEN NOT (
  (
    NEW.stan = 'obowiazuje'
    AND NOT EXISTS (
      SELECT 1
      FROM klienci_kartoteka_umowy_stany
      WHERE umowa_id = NEW.umowa_id
    )
  )
  OR (
    NEW.stan <> 'obowiazuje'
    AND (
      SELECT stan
      FROM klienci_kartoteka_umowy_stany
      WHERE umowa_id = NEW.umowa_id
      ORDER BY id DESC
      LIMIT 1
    ) IS 'obowiazuje'
  )
)
BEGIN
  SELECT RAISE(ABORT, 'stan umowy przechodzi tylko naprzód: od „obowiązuje” do jednego stanu końcowego');
END;

CREATE TRIGGER klienci_kartoteka_stan_umowy_nie_przed_poprzednim
BEFORE INSERT ON klienci_kartoteka_umowy_stany
WHEN NEW.od < (
    SELECT zawarto
    FROM klienci_kartoteka_umowy
    WHERE id = NEW.umowa_id
  )
  OR NEW.od < (
    SELECT max(od)
    FROM klienci_kartoteka_umowy_stany
    WHERE umowa_id = NEW.umowa_id
  )
BEGIN
  SELECT RAISE(ABORT, 'stan umowy nie zaczyna się przed jej zawarciem ani przed poprzednim stanem');
END;

-- Wiersze bez zmian ----------------------------------------------------------------------------

CREATE TRIGGER klienci_kartoteka_wpisy_bez_zmian
BEFORE UPDATE ON klienci_kartoteka_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wpisu klienta, umowy, jej stanu ani informacji się nie zmienia');
END;

CREATE TRIGGER klienci_kartoteka_umowy_bez_zmian
BEFORE UPDATE ON klienci_kartoteka_umowy
BEGIN
  SELECT RAISE(ABORT, 'wpisu klienta, umowy, jej stanu ani informacji się nie zmienia');
END;

CREATE TRIGGER klienci_kartoteka_umowy_stany_bez_zmian
BEFORE UPDATE ON klienci_kartoteka_umowy_stany
BEGIN
  SELECT RAISE(ABORT, 'wpisu klienta, umowy, jej stanu ani informacji się nie zmienia');
END;

CREATE TRIGGER klienci_kartoteka_informacje_bez_zmian
BEFORE UPDATE ON klienci_kartoteka_informacje
BEGIN
  SELECT RAISE(ABORT, 'wpisu klienta, umowy, jej stanu ani informacji się nie zmienia');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabel z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER klienci_kartoteka_wpisy_bez_zastepowania
BEFORE INSERT ON klienci_kartoteka_wpisy
WHEN EXISTS (
  SELECT 1
  FROM klienci_kartoteka_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza kartoteki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER klienci_kartoteka_dane_bez_zastepowania
BEFORE INSERT ON klienci_kartoteka_dane
WHEN EXISTS (
  SELECT 1
  FROM klienci_kartoteka_dane
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza kartoteki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER klienci_kartoteka_adresy_bez_zastepowania
BEFORE INSERT ON klienci_kartoteka_adresy
WHEN EXISTS (
  SELECT 1
  FROM klienci_kartoteka_adresy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza kartoteki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER klienci_kartoteka_kontakty_bez_zastepowania
BEFORE INSERT ON klienci_kartoteka_kontakty
WHEN EXISTS (
  SELECT 1
  FROM klienci_kartoteka_kontakty
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza kartoteki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER klienci_kartoteka_osoby_bez_zastepowania
BEFORE INSERT ON klienci_kartoteka_osoby
WHEN EXISTS (
  SELECT 1
  FROM klienci_kartoteka_osoby
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza kartoteki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER klienci_kartoteka_umowy_bez_zastepowania
BEFORE INSERT ON klienci_kartoteka_umowy
WHEN EXISTS (
  SELECT 1
  FROM klienci_kartoteka_umowy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza kartoteki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER klienci_kartoteka_umowy_stany_bez_zastepowania
BEFORE INSERT ON klienci_kartoteka_umowy_stany
WHEN EXISTS (
  SELECT 1
  FROM klienci_kartoteka_umowy_stany
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza kartoteki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER klienci_kartoteka_informacje_bez_zastepowania
BEFORE INSERT ON klienci_kartoteka_informacje
WHEN EXISTS (
  SELECT 1
  FROM klienci_kartoteka_informacje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza kartoteki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

-- R9: biuro trzyma wpisy klientów bez terminu; pomyłkę poprawia nowy wpis.
CREATE TRIGGER klienci_kartoteka_wpisy_bez_usuwania
BEFORE DELETE ON klienci_kartoteka_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy kartoteki się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER klienci_kartoteka_dane_bez_usuwania
BEFORE DELETE ON klienci_kartoteka_dane
BEGIN
  SELECT RAISE(ABORT, 'wierszy kartoteki się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER klienci_kartoteka_adresy_bez_usuwania
BEFORE DELETE ON klienci_kartoteka_adresy
BEGIN
  SELECT RAISE(ABORT, 'wierszy kartoteki się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER klienci_kartoteka_kontakty_bez_usuwania
BEFORE DELETE ON klienci_kartoteka_kontakty
BEGIN
  SELECT RAISE(ABORT, 'wierszy kartoteki się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER klienci_kartoteka_osoby_bez_usuwania
BEFORE DELETE ON klienci_kartoteka_osoby
BEGIN
  SELECT RAISE(ABORT, 'wierszy kartoteki się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER klienci_kartoteka_umowy_bez_usuwania
BEFORE DELETE ON klienci_kartoteka_umowy
BEGIN
  SELECT RAISE(ABORT, 'wierszy kartoteki się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER klienci_kartoteka_umowy_stany_bez_usuwania
BEFORE DELETE ON klienci_kartoteka_umowy_stany
BEGIN
  SELECT RAISE(ABORT, 'wierszy kartoteki się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER klienci_kartoteka_informacje_bez_usuwania
BEFORE DELETE ON klienci_kartoteka_informacje
BEGIN
  SELECT RAISE(ABORT, 'wierszy kartoteki się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
