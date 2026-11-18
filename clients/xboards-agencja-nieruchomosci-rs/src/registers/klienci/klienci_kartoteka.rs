//! The kind `klienci-kartoteka` of the room `registers/klienci`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "klienci-kartoteka";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Kartoteka klientów biura: osoby i podmioty, które zawarły z biurem umowę pośrednictwa albo
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
"#,
];

/// The values of column `adresy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdresyRodzaj {
    Zamieszkanie,
    DoKorespondencji,
    Siedziba,
}

impl AdresyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[AdresyRodzaj] = &[
        AdresyRodzaj::Zamieszkanie,
        AdresyRodzaj::DoKorespondencji,
        AdresyRodzaj::Siedziba,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            AdresyRodzaj::Zamieszkanie => "zamieszkanie",
            AdresyRodzaj::DoKorespondencji => "do_korespondencji",
            AdresyRodzaj::Siedziba => "siedziba",
        }
    }
}

impl xmeta_room::FromSqlValue for AdresyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "zamieszkanie" => Ok(AdresyRodzaj::Zamieszkanie),
                    "do_korespondencji" => Ok(AdresyRodzaj::DoKorespondencji),
                    "siedziba" => Ok(AdresyRodzaj::Siedziba),
                    other => Err(format!("{other} is no value of AdresyRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for AdresyRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `informacje.sposob`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InformacjeSposob {
    Email,
    Sms,
    Portal,
    List,
    Osobiscie,
}

impl InformacjeSposob {
    /// Every value, in the order of the catalog.
    pub const ALL: &[InformacjeSposob] = &[
        InformacjeSposob::Email,
        InformacjeSposob::Sms,
        InformacjeSposob::Portal,
        InformacjeSposob::List,
        InformacjeSposob::Osobiscie,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            InformacjeSposob::Email => "email",
            InformacjeSposob::Sms => "sms",
            InformacjeSposob::Portal => "portal",
            InformacjeSposob::List => "list",
            InformacjeSposob::Osobiscie => "osobiscie",
        }
    }
}

impl xmeta_room::FromSqlValue for InformacjeSposob {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "email" => Ok(InformacjeSposob::Email),
                    "sms" => Ok(InformacjeSposob::Sms),
                    "portal" => Ok(InformacjeSposob::Portal),
                    "list" => Ok(InformacjeSposob::List),
                    "osobiscie" => Ok(InformacjeSposob::Osobiscie),
                    other => Err(format!("{other} is no value of InformacjeSposob")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for InformacjeSposob {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `kontakty.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KontaktyRodzaj {
    Telefon,
    Email,
}

impl KontaktyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[KontaktyRodzaj] = &[KontaktyRodzaj::Telefon, KontaktyRodzaj::Email];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            KontaktyRodzaj::Telefon => "telefon",
            KontaktyRodzaj::Email => "email",
        }
    }
}

impl xmeta_room::FromSqlValue for KontaktyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "telefon" => Ok(KontaktyRodzaj::Telefon),
                    "email" => Ok(KontaktyRodzaj::Email),
                    other => Err(format!("{other} is no value of KontaktyRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for KontaktyRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `osoby.funkcja`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OsobyFunkcja {
    DzialaZaPodmiot,
    Kontaktowa,
}

impl OsobyFunkcja {
    /// Every value, in the order of the catalog.
    pub const ALL: &[OsobyFunkcja] = &[
        OsobyFunkcja::DzialaZaPodmiot,
        OsobyFunkcja::Kontaktowa,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            OsobyFunkcja::DzialaZaPodmiot => "dziala_za_podmiot",
            OsobyFunkcja::Kontaktowa => "kontaktowa",
        }
    }
}

impl xmeta_room::FromSqlValue for OsobyFunkcja {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "dziala_za_podmiot" => Ok(OsobyFunkcja::DzialaZaPodmiot),
                    "kontaktowa" => Ok(OsobyFunkcja::Kontaktowa),
                    other => Err(format!("{other} is no value of OsobyFunkcja")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for OsobyFunkcja {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `umowy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UmowyRodzaj {
    PosrednictwoSprzedazy,
    PosrednictwoKupna,
    PosrednictwoWynajmu,
    PosrednictwoNajmu,
    ZarzadzanieNajemDlugoterminowy,
    ZarzadzanieNajemKrotkoterminowy,
}

impl UmowyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[UmowyRodzaj] = &[
        UmowyRodzaj::PosrednictwoSprzedazy,
        UmowyRodzaj::PosrednictwoKupna,
        UmowyRodzaj::PosrednictwoWynajmu,
        UmowyRodzaj::PosrednictwoNajmu,
        UmowyRodzaj::ZarzadzanieNajemDlugoterminowy,
        UmowyRodzaj::ZarzadzanieNajemKrotkoterminowy,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            UmowyRodzaj::PosrednictwoSprzedazy => "posrednictwo_sprzedazy",
            UmowyRodzaj::PosrednictwoKupna => "posrednictwo_kupna",
            UmowyRodzaj::PosrednictwoWynajmu => "posrednictwo_wynajmu",
            UmowyRodzaj::PosrednictwoNajmu => "posrednictwo_najmu",
            UmowyRodzaj::ZarzadzanieNajemDlugoterminowy => {
                "zarzadzanie_najem_dlugoterminowy"
            }
            UmowyRodzaj::ZarzadzanieNajemKrotkoterminowy => {
                "zarzadzanie_najem_krotkoterminowy"
            }
        }
    }
}

impl xmeta_room::FromSqlValue for UmowyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "posrednictwo_sprzedazy" => Ok(UmowyRodzaj::PosrednictwoSprzedazy),
                    "posrednictwo_kupna" => Ok(UmowyRodzaj::PosrednictwoKupna),
                    "posrednictwo_wynajmu" => Ok(UmowyRodzaj::PosrednictwoWynajmu),
                    "posrednictwo_najmu" => Ok(UmowyRodzaj::PosrednictwoNajmu),
                    "zarzadzanie_najem_dlugoterminowy" => {
                        Ok(UmowyRodzaj::ZarzadzanieNajemDlugoterminowy)
                    }
                    "zarzadzanie_najem_krotkoterminowy" => {
                        Ok(UmowyRodzaj::ZarzadzanieNajemKrotkoterminowy)
                    }
                    other => Err(format!("{other} is no value of UmowyRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for UmowyRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `umowy.dzial`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UmowyDzial {
    Posrednictwo,
    Najem,
}

impl UmowyDzial {
    /// Every value, in the order of the catalog.
    pub const ALL: &[UmowyDzial] = &[UmowyDzial::Posrednictwo, UmowyDzial::Najem];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            UmowyDzial::Posrednictwo => "posrednictwo",
            UmowyDzial::Najem => "najem",
        }
    }
}

impl xmeta_room::FromSqlValue for UmowyDzial {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "posrednictwo" => Ok(UmowyDzial::Posrednictwo),
                    "najem" => Ok(UmowyDzial::Najem),
                    other => Err(format!("{other} is no value of UmowyDzial")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for UmowyDzial {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `umowy.typ_teczki`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UmowyTypTeczki {
    Oferta,
    Poszukiwanie,
    Najem,
    LokalKrotkoterminowy,
}

impl UmowyTypTeczki {
    /// Every value, in the order of the catalog.
    pub const ALL: &[UmowyTypTeczki] = &[
        UmowyTypTeczki::Oferta,
        UmowyTypTeczki::Poszukiwanie,
        UmowyTypTeczki::Najem,
        UmowyTypTeczki::LokalKrotkoterminowy,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            UmowyTypTeczki::Oferta => "oferta",
            UmowyTypTeczki::Poszukiwanie => "poszukiwanie",
            UmowyTypTeczki::Najem => "najem",
            UmowyTypTeczki::LokalKrotkoterminowy => "lokal_krotkoterminowy",
        }
    }
}

impl xmeta_room::FromSqlValue for UmowyTypTeczki {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "oferta" => Ok(UmowyTypTeczki::Oferta),
                    "poszukiwanie" => Ok(UmowyTypTeczki::Poszukiwanie),
                    "najem" => Ok(UmowyTypTeczki::Najem),
                    "lokal_krotkoterminowy" => Ok(UmowyTypTeczki::LokalKrotkoterminowy),
                    other => Err(format!("{other} is no value of UmowyTypTeczki")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for UmowyTypTeczki {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `umowy_stany.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UmowyStanyStan {
    Obowiazuje,
    Odstapiono,
    Wypowiedziana,
    Wygasla,
    Wykonana,
}

impl UmowyStanyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[UmowyStanyStan] = &[
        UmowyStanyStan::Obowiazuje,
        UmowyStanyStan::Odstapiono,
        UmowyStanyStan::Wypowiedziana,
        UmowyStanyStan::Wygasla,
        UmowyStanyStan::Wykonana,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            UmowyStanyStan::Obowiazuje => "obowiazuje",
            UmowyStanyStan::Odstapiono => "odstapiono",
            UmowyStanyStan::Wypowiedziana => "wypowiedziana",
            UmowyStanyStan::Wygasla => "wygasla",
            UmowyStanyStan::Wykonana => "wykonana",
        }
    }
}

impl xmeta_room::FromSqlValue for UmowyStanyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "obowiazuje" => Ok(UmowyStanyStan::Obowiazuje),
                    "odstapiono" => Ok(UmowyStanyStan::Odstapiono),
                    "wypowiedziana" => Ok(UmowyStanyStan::Wypowiedziana),
                    "wygasla" => Ok(UmowyStanyStan::Wygasla),
                    "wykonana" => Ok(UmowyStanyStan::Wykonana),
                    other => Err(format!("{other} is no value of UmowyStanyStan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for UmowyStanyStan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.rodzaj_podmiotu`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzajPodmiotu {
    OsobaFizyczna,
    OsobaPrawna,
    JednostkaBezOsobowosci,
}

impl WpisyRodzajPodmiotu {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzajPodmiotu] = &[
        WpisyRodzajPodmiotu::OsobaFizyczna,
        WpisyRodzajPodmiotu::OsobaPrawna,
        WpisyRodzajPodmiotu::JednostkaBezOsobowosci,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzajPodmiotu::OsobaFizyczna => "osoba_fizyczna",
            WpisyRodzajPodmiotu::OsobaPrawna => "osoba_prawna",
            WpisyRodzajPodmiotu::JednostkaBezOsobowosci => "jednostka_bez_osobowosci",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzajPodmiotu {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "osoba_fizyczna" => Ok(WpisyRodzajPodmiotu::OsobaFizyczna),
                    "osoba_prawna" => Ok(WpisyRodzajPodmiotu::OsobaPrawna),
                    "jednostka_bez_osobowosci" => {
                        Ok(WpisyRodzajPodmiotu::JednostkaBezOsobowosci)
                    }
                    other => Err(format!("{other} is no value of WpisyRodzajPodmiotu")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyRodzajPodmiotu {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The parameters of `adresy_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AdresyKlientaParams {
    pub klient_id: String,
}

/// A row of `adresy_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct AdresyKlientaRow {
    pub id: i64,
    pub klient_id: String,
    pub rodzaj: AdresyRodzaj,
    pub adres: String,
    pub od: String,
    pub r#do: Option<String>,
    pub zrodlo_odnosnik: String,
}

impl AdresyKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(AdresyKlientaRow {
            id: row.get("id")?,
            klient_id: row.get("klient_id")?,
            rodzaj: row.get("rodzaj")?,
            adres: row.get("adres")?,
            od: row.get("od")?,
            r#do: row.get("do")?,
            zrodlo_odnosnik: row.get("zrodlo_odnosnik")?,
        })
    }
}

/// Adresy klienta, bieżące i zakończone, najnowsze pierwsze.
pub fn adresy_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &AdresyKlientaParams,
) -> Result<Vec<AdresyKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM klienci_kartoteka_adresy
WHERE klient_id = :klient_id
ORDER BY id DESC"#,
        parameters: Vec::from([
            ("klient_id", xmeta_room::ToSqlValue::to_sql_value(&params.klient_id)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(AdresyKlientaRow::from_row).collect()
}

/// The parameters of `wpisz_adres_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszAdresKlientaParams {
    pub od: String,
    pub klient_id: String,
    pub rodzaj: AdresyRodzaj,
    pub adres: String,
    pub zrodlo_odnosnik: String,
}

/// Nowy adres klienta ze źródłem w teczce. Bieżący adres tego samego rodzaju kończy się w dniu,
/// od którego obowiązuje nowy.
pub fn wpisz_adres_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszAdresKlientaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE klienci_kartoteka_adresy
SET do = :od
WHERE klient_id = :klient_id
  AND rodzaj = :rodzaj
  AND do IS NULL"#,
                parameters: Vec::from([
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    (
                        "klient_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_id),
                    ),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                ]),
            },
            xmeta_room::Statement {
                sql: r#"INSERT INTO klienci_kartoteka_adresy (klient_id, rodzaj, adres, od, zrodlo_odnosnik)
VALUES (:klient_id, :rodzaj, :adres, :od, :zrodlo_odnosnik)"#,
                parameters: Vec::from([
                    (
                        "klient_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_id),
                    ),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("adres", xmeta_room::ToSqlValue::to_sql_value(&params.adres)),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    (
                        "zrodlo_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_odnosnik),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zakoncz_adres_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZakonczAdresKlientaParams {
    pub r#do: Option<String>,
    pub id: i64,
}

/// Adres, który przestał obowiązywać bez następcy, np. adres do korespondencji, gdy klient chce
/// dostawać pisma na adres zamieszkania.
pub fn zakoncz_adres_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &ZakonczAdresKlientaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE klienci_kartoteka_adresy
SET do = :do
WHERE id = :id"#,
                parameters: Vec::from([
                    ("do", xmeta_room::ToSqlValue::to_sql_value(&params.r#do)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `dane_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DaneKlientaParams {
    pub klient_id: String,
}

/// A row of `dane_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct DaneKlientaRow {
    pub id: i64,
    pub klient_id: String,
    pub od: String,
    pub aktualne: bool,
    pub imiona: Option<String>,
    pub nazwisko: Option<String>,
    pub pesel: Option<String>,
    pub nazwa: Option<String>,
    pub forma_prawna: Option<String>,
    pub krs: Option<String>,
    pub ceidg: bool,
    pub nip: Option<String>,
    pub regon: Option<String>,
    pub firma: Option<String>,
    pub zrodlo_odnosnik: String,
    pub zrodlo_opis: Option<String>,
    pub wpisal: String,
}

impl DaneKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(DaneKlientaRow {
            id: row.get("id")?,
            klient_id: row.get("klient_id")?,
            od: row.get("od")?,
            aktualne: row.get("aktualne")?,
            imiona: row.get("imiona")?,
            nazwisko: row.get("nazwisko")?,
            pesel: row.get("pesel")?,
            nazwa: row.get("nazwa")?,
            forma_prawna: row.get("forma_prawna")?,
            krs: row.get("krs")?,
            ceidg: row.get("ceidg")?,
            nip: row.get("nip")?,
            regon: row.get("regon")?,
            firma: row.get("firma")?,
            zrodlo_odnosnik: row.get("zrodlo_odnosnik")?,
            zrodlo_opis: row.get("zrodlo_opis")?,
            wpisal: row.get("wpisal")?,
        })
    }
}

/// Wszystkie wersje danych klienta, najnowsza pierwsza; aktualna ma aktualne = 1.
pub fn dane_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &DaneKlientaParams,
) -> Result<Vec<DaneKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM klienci_kartoteka_dane
WHERE klient_id = :klient_id
ORDER BY id DESC"#,
        parameters: Vec::from([
            ("klient_id", xmeta_room::ToSqlValue::to_sql_value(&params.klient_id)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(DaneKlientaRow::from_row).collect()
}

/// The parameters of `wpisz_dane_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszDaneKlientaParams {
    pub klient_id: String,
    pub od: String,
    pub imiona: Option<String>,
    pub nazwisko: Option<String>,
    pub pesel: Option<String>,
    pub nazwa: Option<String>,
    pub forma_prawna: Option<String>,
    pub krs: Option<String>,
    pub ceidg: bool,
    pub nip: Option<String>,
    pub regon: Option<String>,
    pub firma: Option<String>,
    pub zrodlo_odnosnik: String,
    pub zrodlo_opis: Option<String>,
    pub wpisal: String,
}

/// Nowa wersja danych klienta ze źródłem w teczce, np. nowe nazwisko z aneksu. Poprzednia wersja
/// traci aktualność, ale zostaje.
pub fn wpisz_dane_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszDaneKlientaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO klienci_kartoteka_dane (klient_id, od, aktualne, imiona, nazwisko, pesel, nazwa,
  forma_prawna, krs, ceidg, nip, regon, firma, zrodlo_odnosnik, zrodlo_opis, wpisal)
VALUES (:klient_id, :od, 1, :imiona, :nazwisko, :pesel, :nazwa, :forma_prawna, :krs, :ceidg, :nip,
  :regon, :firma, :zrodlo_odnosnik, :zrodlo_opis, :wpisal)"#,
                parameters: Vec::from([
                    (
                        "klient_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_id),
                    ),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    ("imiona", xmeta_room::ToSqlValue::to_sql_value(&params.imiona)),
                    ("nazwisko", xmeta_room::ToSqlValue::to_sql_value(&params.nazwisko)),
                    ("pesel", xmeta_room::ToSqlValue::to_sql_value(&params.pesel)),
                    ("nazwa", xmeta_room::ToSqlValue::to_sql_value(&params.nazwa)),
                    (
                        "forma_prawna",
                        xmeta_room::ToSqlValue::to_sql_value(&params.forma_prawna),
                    ),
                    ("krs", xmeta_room::ToSqlValue::to_sql_value(&params.krs)),
                    ("ceidg", xmeta_room::ToSqlValue::to_sql_value(&params.ceidg)),
                    ("nip", xmeta_room::ToSqlValue::to_sql_value(&params.nip)),
                    ("regon", xmeta_room::ToSqlValue::to_sql_value(&params.regon)),
                    ("firma", xmeta_room::ToSqlValue::to_sql_value(&params.firma)),
                    (
                        "zrodlo_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_odnosnik),
                    ),
                    (
                        "zrodlo_opis",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_opis),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                ]),
            },
        ],
    )
}

/// The parameters of `informacje_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct InformacjeKlientaParams {
    pub klient_id: String,
}

/// A row of `informacje_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct InformacjeKlientaRow {
    pub id: i64,
    pub klient_id: String,
    pub poinformowano: String,
    pub sposob: InformacjeSposob,
    pub wysylka_odnosnik: String,
}

impl InformacjeKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(InformacjeKlientaRow {
            id: row.get("id")?,
            klient_id: row.get("klient_id")?,
            poinformowano: row.get("poinformowano")?,
            sposob: row.get("sposob")?,
            wysylka_odnosnik: row.get("wysylka_odnosnik")?,
        })
    }
}

/// Kiedy i jak biuro poinformowało klienta o danych w kartotece.
pub fn informacje_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &InformacjeKlientaParams,
) -> Result<Vec<InformacjeKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM klienci_kartoteka_informacje
WHERE klient_id = :klient_id
ORDER BY poinformowano, id"#,
        parameters: Vec::from([
            ("klient_id", xmeta_room::ToSqlValue::to_sql_value(&params.klient_id)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(InformacjeKlientaRow::from_row).collect()
}

/// The parameters of `wpisz_informacje_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszInformacjeKlientaParams {
    pub klient_id: String,
    pub poinformowano: String,
    pub sposob: InformacjeSposob,
    pub wysylka_odnosnik: String,
}

/// Informacja o kartotece wysłana klientowi przez agenta sprawy (art. 13 RODO).
pub fn wpisz_informacje_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszInformacjeKlientaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO klienci_kartoteka_informacje (klient_id, poinformowano, sposob, wysylka_odnosnik)
VALUES (:klient_id, :poinformowano, :sposob, :wysylka_odnosnik)"#,
                parameters: Vec::from([
                    (
                        "klient_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_id),
                    ),
                    (
                        "poinformowano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poinformowano),
                    ),
                    ("sposob", xmeta_room::ToSqlValue::to_sql_value(&params.sposob)),
                    (
                        "wysylka_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wysylka_odnosnik),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `kontakty_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct KontaktyKlientaParams {
    pub klient_id: String,
}

/// A row of `kontakty_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct KontaktyKlientaRow {
    pub id: i64,
    pub klient_id: String,
    pub rodzaj: KontaktyRodzaj,
    pub wartosc: String,
    pub preferowany: bool,
    pub od: String,
    pub r#do: Option<String>,
    pub zrodlo_odnosnik: String,
}

impl KontaktyKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(KontaktyKlientaRow {
            id: row.get("id")?,
            klient_id: row.get("klient_id")?,
            rodzaj: row.get("rodzaj")?,
            wartosc: row.get("wartosc")?,
            preferowany: row.get("preferowany")?,
            od: row.get("od")?,
            r#do: row.get("do")?,
            zrodlo_odnosnik: row.get("zrodlo_odnosnik")?,
        })
    }
}

/// Kontakty klienta, bieżące i zakończone, najnowsze pierwsze.
pub fn kontakty_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &KontaktyKlientaParams,
) -> Result<Vec<KontaktyKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM klienci_kartoteka_kontakty
WHERE klient_id = :klient_id
ORDER BY id DESC"#,
        parameters: Vec::from([
            ("klient_id", xmeta_room::ToSqlValue::to_sql_value(&params.klient_id)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(KontaktyKlientaRow::from_row).collect()
}

/// The parameters of `klienci_po_kontakcie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct KlienciPoKontakcieParams {
    pub wartosc: String,
}

/// A row of `klienci_po_kontakcie`.
#[derive(Debug, Clone, PartialEq)]
pub struct KlienciPoKontakcieRow {
    pub id: i64,
    pub klient_id: String,
    pub rodzaj: KontaktyRodzaj,
    pub wartosc: String,
    pub preferowany: bool,
    pub od: String,
    pub r#do: Option<String>,
    pub zrodlo_odnosnik: String,
}

impl KlienciPoKontakcieRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(KlienciPoKontakcieRow {
            id: row.get("id")?,
            klient_id: row.get("klient_id")?,
            rodzaj: row.get("rodzaj")?,
            wartosc: row.get("wartosc")?,
            preferowany: row.get("preferowany")?,
            od: row.get("od")?,
            r#do: row.get("do")?,
            zrodlo_odnosnik: row.get("zrodlo_odnosnik")?,
        })
    }
}

/// Klienci z tym telefonem albo adresem poczty, także dawnym: stary numer z karty Recepcji nadal
/// wskazuje klienta.
pub fn klienci_po_kontakcie(
    room: &impl xmeta_room::MetaRoom,
    params: &KlienciPoKontakcieParams,
) -> Result<Vec<KlienciPoKontakcieRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM klienci_kartoteka_kontakty
WHERE wartosc = :wartosc
ORDER BY klient_id, id"#,
        parameters: Vec::from([
            ("wartosc", xmeta_room::ToSqlValue::to_sql_value(&params.wartosc)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(KlienciPoKontakcieRow::from_row).collect()
}

/// The parameters of `wpisz_kontakt_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszKontaktKlientaParams {
    pub klient_id: String,
    pub rodzaj: KontaktyRodzaj,
    pub wartosc: String,
    pub preferowany: bool,
    pub od: String,
    pub zrodlo_odnosnik: String,
}

/// Nowy telefon albo adres poczty klienta ze źródłem w teczce. Wiadomość „zmieniłam numer” nie
/// jest źródłem; agent sprawy potwierdza zmianę innym kanałem.
pub fn wpisz_kontakt_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszKontaktKlientaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO klienci_kartoteka_kontakty (klient_id, rodzaj, wartosc, preferowany, od, zrodlo_odnosnik)
VALUES (:klient_id, :rodzaj, :wartosc, :preferowany, :od, :zrodlo_odnosnik)"#,
                parameters: Vec::from([
                    (
                        "klient_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_id),
                    ),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("wartosc", xmeta_room::ToSqlValue::to_sql_value(&params.wartosc)),
                    (
                        "preferowany",
                        xmeta_room::ToSqlValue::to_sql_value(&params.preferowany),
                    ),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    (
                        "zrodlo_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_odnosnik),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zakoncz_kontakt_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZakonczKontaktKlientaParams {
    pub r#do: Option<String>,
    pub id: i64,
}

/// Kontakt, którego klient już nie używa; zostaje w kartotece jako historia.
pub fn zakoncz_kontakt_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &ZakonczKontaktKlientaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE klienci_kartoteka_kontakty
SET do = :do
WHERE id = :id"#,
                parameters: Vec::from([
                    ("do", xmeta_room::ToSqlValue::to_sql_value(&params.r#do)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `osoby_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OsobyKlientaParams {
    pub klient_id: String,
}

/// A row of `osoby_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct OsobyKlientaRow {
    pub id: i64,
    pub klient_id: String,
    pub funkcja: OsobyFunkcja,
    pub imiona: String,
    pub nazwisko: String,
    pub kontakt: Option<String>,
    pub podstawa_odnosnik: Option<String>,
    pub od: String,
    pub r#do: Option<String>,
    pub zrodlo_odnosnik: String,
}

impl OsobyKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(OsobyKlientaRow {
            id: row.get("id")?,
            klient_id: row.get("klient_id")?,
            funkcja: row.get("funkcja")?,
            imiona: row.get("imiona")?,
            nazwisko: row.get("nazwisko")?,
            kontakt: row.get("kontakt")?,
            podstawa_odnosnik: row.get("podstawa_odnosnik")?,
            od: row.get("od")?,
            r#do: row.get("do")?,
            zrodlo_odnosnik: row.get("zrodlo_odnosnik")?,
        })
    }
}

/// Osoby działające za podmiot i osoby kontaktowe, bieżące i zakończone.
pub fn osoby_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &OsobyKlientaParams,
) -> Result<Vec<OsobyKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM klienci_kartoteka_osoby
WHERE klient_id = :klient_id
ORDER BY id DESC"#,
        parameters: Vec::from([
            ("klient_id", xmeta_room::ToSqlValue::to_sql_value(&params.klient_id)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(OsobyKlientaRow::from_row).collect()
}

/// The parameters of `wpisz_osobe_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszOsobeKlientaParams {
    pub klient_id: String,
    pub funkcja: OsobyFunkcja,
    pub imiona: String,
    pub nazwisko: String,
    pub kontakt: Option<String>,
    pub podstawa_odnosnik: Option<String>,
    pub od: String,
    pub zrodlo_odnosnik: String,
}

/// Osoba, która podpisała umowę za podmiot (z podstawą umocowania), albo osoba kontaktowa.
pub fn wpisz_osobe_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszOsobeKlientaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO klienci_kartoteka_osoby (klient_id, funkcja, imiona, nazwisko, kontakt,
  podstawa_odnosnik, od, zrodlo_odnosnik)
VALUES (:klient_id, :funkcja, :imiona, :nazwisko, :kontakt, :podstawa_odnosnik, :od,
  :zrodlo_odnosnik)"#,
                parameters: Vec::from([
                    (
                        "klient_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_id),
                    ),
                    ("funkcja", xmeta_room::ToSqlValue::to_sql_value(&params.funkcja)),
                    ("imiona", xmeta_room::ToSqlValue::to_sql_value(&params.imiona)),
                    ("nazwisko", xmeta_room::ToSqlValue::to_sql_value(&params.nazwisko)),
                    ("kontakt", xmeta_room::ToSqlValue::to_sql_value(&params.kontakt)),
                    (
                        "podstawa_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.podstawa_odnosnik),
                    ),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    (
                        "zrodlo_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_odnosnik),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zakoncz_osobe_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZakonczOsobeKlientaParams {
    pub r#do: Option<String>,
    pub id: i64,
}

/// Osoba, która przestała działać za podmiot albo rozmawiać z biurem.
pub fn zakoncz_osobe_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &ZakonczOsobeKlientaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE klienci_kartoteka_osoby
SET do = :do
WHERE id = :id"#,
                parameters: Vec::from([
                    ("do", xmeta_room::ToSqlValue::to_sql_value(&params.r#do)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `umowy_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UmowyKlientaParams {
    pub klient_id: String,
}

/// A row of `umowy_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct UmowyKlientaRow {
    pub id: String,
    pub klient_id: String,
    pub rodzaj: UmowyRodzaj,
    pub dzial: UmowyDzial,
    pub nieruchomosc_adres: Option<String>,
    pub numer_kw: Option<String>,
    pub teczka_odnosnik: String,
    pub typ_teczki: UmowyTypTeczki,
    pub oznaczenie: String,
    pub umowa_odnosnik: String,
    pub podpisal_za_biuro: String,
    pub zawarto: String,
    pub koniec: Option<String>,
    pub konsument: bool,
    pub obie_strony: bool,
    pub wpisal: String,
    pub wpisano: String,
    pub stan: UmowyStanyStan,
    pub stan_od: String,
}

impl UmowyKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(UmowyKlientaRow {
            id: row.get("id")?,
            klient_id: row.get("klient_id")?,
            rodzaj: row.get("rodzaj")?,
            dzial: row.get("dzial")?,
            nieruchomosc_adres: row.get("nieruchomosc_adres")?,
            numer_kw: row.get("numer_kw")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            typ_teczki: row.get("typ_teczki")?,
            oznaczenie: row.get("oznaczenie")?,
            umowa_odnosnik: row.get("umowa_odnosnik")?,
            podpisal_za_biuro: row.get("podpisal_za_biuro")?,
            zawarto: row.get("zawarto")?,
            koniec: row.get("koniec")?,
            konsument: row.get("konsument")?,
            obie_strony: row.get("obie_strony")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
            stan: row.get("stan")?,
            stan_od: row.get("stan_od")?,
        })
    }
}

/// Umowy klienta z biurem z bieżącym stanem, najnowsze pierwsze. Kierownik pośrednictwa sprawdza
/// tu obie_strony przed transakcją dla obu stron (R1).
pub fn umowy_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &UmowyKlientaParams,
) -> Result<Vec<UmowyKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT
  umowy.*,
  stany.stan,
  stany.od AS stan_od
FROM klienci_kartoteka_umowy AS umowy
JOIN klienci_kartoteka_umowy_stany AS stany ON stany.umowa_id = umowy.id
  AND stany.id = (
    SELECT max(id)
    FROM klienci_kartoteka_umowy_stany
    WHERE umowa_id = umowy.id
  )
WHERE umowy.klient_id = :klient_id
ORDER BY umowy.zawarto DESC, umowy.rowid DESC"#,
        parameters: Vec::from([
            ("klient_id", xmeta_room::ToSqlValue::to_sql_value(&params.klient_id)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(UmowyKlientaRow::from_row).collect()
}

/// The parameters of `stany_umowy_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StanyUmowyKlientaParams {
    pub umowa_id: String,
}

/// A row of `stany_umowy_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct StanyUmowyKlientaRow {
    pub id: i64,
    pub umowa_id: String,
    pub stan: UmowyStanyStan,
    pub od: String,
    pub zrodlo_odnosnik: String,
    pub wpisal: String,
}

impl StanyUmowyKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(StanyUmowyKlientaRow {
            id: row.get("id")?,
            umowa_id: row.get("umowa_id")?,
            stan: row.get("stan")?,
            od: row.get("od")?,
            zrodlo_odnosnik: row.get("zrodlo_odnosnik")?,
            wpisal: row.get("wpisal")?,
        })
    }
}

/// Historia stanów jednej umowy ze źródłami.
pub fn stany_umowy_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &StanyUmowyKlientaParams,
) -> Result<Vec<StanyUmowyKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM klienci_kartoteka_umowy_stany
WHERE umowa_id = :umowa_id
ORDER BY id"#,
        parameters: Vec::from([
            ("umowa_id", xmeta_room::ToSqlValue::to_sql_value(&params.umowa_id)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(StanyUmowyKlientaRow::from_row).collect()
}

/// The parameters of `koniec_ostatniej_umowy_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct KoniecOstatniejUmowyKlientaParams {
    pub klient_id: String,
}

/// A row of `koniec_ostatniej_umowy_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct KoniecOstatniejUmowyKlientaRow {
    pub koniec: Option<String>,
}

impl KoniecOstatniejUmowyKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(KoniecOstatniejUmowyKlientaRow {
            koniec: row.get("koniec")?,
        })
    }
}

/// Dzień końca ostatniej umowy klienta; null, gdy któraś umowa jeszcze obowiązuje albo klient nie
/// ma umów.
pub fn koniec_ostatniej_umowy_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &KoniecOstatniejUmowyKlientaParams,
) -> Result<Option<KoniecOstatniejUmowyKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT
  CASE
    WHEN count(*) = 0 OR sum(stany.stan = 'obowiazuje') > 0 THEN NULL
    ELSE max(stany.od)
  END AS koniec
FROM klienci_kartoteka_umowy AS umowy
JOIN klienci_kartoteka_umowy_stany AS stany ON stany.umowa_id = umowy.id
  AND stany.id = (
    SELECT max(id)
    FROM klienci_kartoteka_umowy_stany
    WHERE umowa_id = umowy.id
  )
WHERE umowy.klient_id = :klient_id"#,
        parameters: Vec::from([
            ("klient_id", xmeta_room::ToSqlValue::to_sql_value(&params.klient_id)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.first().map(KoniecOstatniejUmowyKlientaRow::from_row).transpose()
}

/// The parameters of `wpisz_umowe_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszUmoweKlientaParams {
    pub id: String,
    pub klient_id: String,
    pub rodzaj: UmowyRodzaj,
    pub dzial: UmowyDzial,
    pub nieruchomosc_adres: Option<String>,
    pub numer_kw: Option<String>,
    pub teczka_odnosnik: String,
    pub typ_teczki: UmowyTypTeczki,
    pub oznaczenie: String,
    pub umowa_odnosnik: String,
    pub podpisal_za_biuro: String,
    pub zawarto: String,
    pub koniec: Option<String>,
    pub konsument: bool,
    pub obie_strony: bool,
    pub wpisal: String,
    pub wpisano: String,
}

/// Umowa klienta z biurem w dniu jej podpisania; obowiązuje od zawarcia, a źródłem stanu jest
/// umowa w teczce. Identyfikator (UUID) nadaje aplikacja.
pub fn wpisz_umowe_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszUmoweKlientaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO klienci_kartoteka_umowy (id, klient_id, rodzaj, dzial, nieruchomosc_adres, numer_kw,
  teczka_odnosnik, typ_teczki, oznaczenie, umowa_odnosnik, podpisal_za_biuro, zawarto, koniec,
  konsument, obie_strony, wpisal, wpisano)
VALUES (:id, :klient_id, :rodzaj, :dzial, :nieruchomosc_adres, :numer_kw, :teczka_odnosnik,
  :typ_teczki, :oznaczenie, :umowa_odnosnik, :podpisal_za_biuro, :zawarto, :koniec, :konsument,
  :obie_strony, :wpisal, :wpisano)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "klient_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_id),
                    ),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("dzial", xmeta_room::ToSqlValue::to_sql_value(&params.dzial)),
                    (
                        "nieruchomosc_adres",
                        xmeta_room::ToSqlValue::to_sql_value(&params.nieruchomosc_adres),
                    ),
                    ("numer_kw", xmeta_room::ToSqlValue::to_sql_value(&params.numer_kw)),
                    (
                        "teczka_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.teczka_odnosnik),
                    ),
                    (
                        "typ_teczki",
                        xmeta_room::ToSqlValue::to_sql_value(&params.typ_teczki),
                    ),
                    (
                        "oznaczenie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.oznaczenie),
                    ),
                    (
                        "umowa_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.umowa_odnosnik),
                    ),
                    (
                        "podpisal_za_biuro",
                        xmeta_room::ToSqlValue::to_sql_value(&params.podpisal_za_biuro),
                    ),
                    ("zawarto", xmeta_room::ToSqlValue::to_sql_value(&params.zawarto)),
                    ("koniec", xmeta_room::ToSqlValue::to_sql_value(&params.koniec)),
                    (
                        "konsument",
                        xmeta_room::ToSqlValue::to_sql_value(&params.konsument),
                    ),
                    (
                        "obie_strony",
                        xmeta_room::ToSqlValue::to_sql_value(&params.obie_strony),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
            xmeta_room::Statement {
                sql: r#"INSERT INTO klienci_kartoteka_umowy_stany (umowa_id, stan, od, zrodlo_odnosnik, wpisal)
VALUES (:id, 'obowiazuje', :zawarto, :umowa_odnosnik, :wpisal)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("zawarto", xmeta_room::ToSqlValue::to_sql_value(&params.zawarto)),
                    (
                        "umowa_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.umowa_odnosnik),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_stan_umowy_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZmienStanUmowyKlientaParams {
    pub umowa_id: String,
    pub stan: UmowyStanyStan,
    pub od: String,
    pub zrodlo_odnosnik: String,
    pub wpisal: String,
}

/// Koniec umowy po raporcie agenta: odstąpienie, wypowiedzenie, wygaśnięcie albo wykonanie, ze
/// źródłem w teczce.
pub fn zmien_stan_umowy_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienStanUmowyKlientaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO klienci_kartoteka_umowy_stany (umowa_id, stan, od, zrodlo_odnosnik, wpisal)
VALUES (:umowa_id, :stan, :od, :zrodlo_odnosnik, :wpisal)"#,
                parameters: Vec::from([
                    ("umowa_id", xmeta_room::ToSqlValue::to_sql_value(&params.umowa_id)),
                    ("stan", xmeta_room::ToSqlValue::to_sql_value(&params.stan)),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    (
                        "zrodlo_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_odnosnik),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                ]),
            },
        ],
    )
}

/// A row of `klienci`.
#[derive(Debug, Clone, PartialEq)]
pub struct KlienciRow {
    pub id: String,
    pub rodzaj_podmiotu: WpisyRodzajPodmiotu,
    pub wpisano: String,
    pub imiona: Option<String>,
    pub nazwisko: Option<String>,
    pub nazwa: Option<String>,
    pub firma: Option<String>,
}

impl KlienciRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(KlienciRow {
            id: row.get("id")?,
            rodzaj_podmiotu: row.get("rodzaj_podmiotu")?,
            wpisano: row.get("wpisano")?,
            imiona: row.get("imiona")?,
            nazwisko: row.get("nazwisko")?,
            nazwa: row.get("nazwa")?,
            firma: row.get("firma")?,
        })
    }
}

/// Klienci kartoteki z aktualną wersją danych, w kolejności wpisania.
pub fn klienci(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<KlienciRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT
  wpisy.id,
  wpisy.rodzaj_podmiotu,
  wpisy.wpisano,
  dane.imiona,
  dane.nazwisko,
  dane.nazwa,
  dane.firma
FROM klienci_kartoteka_wpisy AS wpisy
JOIN klienci_kartoteka_dane AS dane ON dane.klient_id = wpisy.id
  AND dane.aktualne = 1
ORDER BY wpisy.rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(KlienciRow::from_row).collect()
}

/// The parameters of `klienci_po_nazwisku_albo_nazwie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct KlienciPoNazwiskuAlboNazwieParams {
    pub szukane: String,
}

/// A row of `klienci_po_nazwisku_albo_nazwie`.
#[derive(Debug, Clone, PartialEq)]
pub struct KlienciPoNazwiskuAlboNazwieRow {
    pub klient_id: String,
    pub imiona: Option<String>,
    pub nazwisko: Option<String>,
    pub nazwa: Option<String>,
    pub od: String,
    pub aktualne: bool,
}

impl KlienciPoNazwiskuAlboNazwieRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(KlienciPoNazwiskuAlboNazwieRow {
            klient_id: row.get("klient_id")?,
            imiona: row.get("imiona")?,
            nazwisko: row.get("nazwisko")?,
            nazwa: row.get("nazwa")?,
            od: row.get("od")?,
            aktualne: row.get("aktualne")?,
        })
    }
}

/// Klienci, których którakolwiek wersja danych ma to nazwisko albo tę nazwę. Kierownik biura
/// porównuje kartę Recepcji także z danymi sprzed zmiany, np. z nazwiskiem sprzed ślubu.
pub fn klienci_po_nazwisku_albo_nazwie(
    room: &impl xmeta_room::MetaRoom,
    params: &KlienciPoNazwiskuAlboNazwieParams,
) -> Result<Vec<KlienciPoNazwiskuAlboNazwieRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT DISTINCT
  dane.klient_id,
  dane.imiona,
  dane.nazwisko,
  dane.nazwa,
  dane.od,
  dane.aktualne
FROM klienci_kartoteka_dane AS dane
WHERE dane.nazwisko = :szukane
  OR dane.nazwa = :szukane
ORDER BY dane.klient_id, dane.id"#,
        parameters: Vec::from([
            ("szukane", xmeta_room::ToSqlValue::to_sql_value(&params.szukane)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(KlienciPoNazwiskuAlboNazwieRow::from_row).collect()
}

/// The parameters of `wpisz_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszKlientaParams {
    pub id: String,
    pub rodzaj_podmiotu: WpisyRodzajPodmiotu,
    pub wpisal: String,
    pub wpisano: String,
    pub od: String,
    pub imiona: Option<String>,
    pub nazwisko: Option<String>,
    pub pesel: Option<String>,
    pub nazwa: Option<String>,
    pub forma_prawna: Option<String>,
    pub krs: Option<String>,
    pub ceidg: bool,
    pub nip: Option<String>,
    pub regon: Option<String>,
    pub firma: Option<String>,
    pub zrodlo_odnosnik: String,
    pub zrodlo_opis: Option<String>,
}

/// Nowy klient z pierwszą wersją danych, w dniu podpisania pierwszej umowy. Identyfikator (UUID)
/// nadaje aplikacja, bo wskazują go umowy, adresy i kontakty klienta.
pub fn wpisz_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszKlientaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO klienci_kartoteka_wpisy (id, rodzaj_podmiotu, wpisal, wpisano)
VALUES (:id, :rodzaj_podmiotu, :wpisal, :wpisano)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "rodzaj_podmiotu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj_podmiotu),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
            xmeta_room::Statement {
                sql: r#"INSERT INTO klienci_kartoteka_dane (klient_id, od, aktualne, imiona, nazwisko, pesel, nazwa,
  forma_prawna, krs, ceidg, nip, regon, firma, zrodlo_odnosnik, zrodlo_opis, wpisal)
VALUES (:id, :od, 1, :imiona, :nazwisko, :pesel, :nazwa, :forma_prawna, :krs, :ceidg, :nip, :regon,
  :firma, :zrodlo_odnosnik, :zrodlo_opis, :wpisal)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    ("imiona", xmeta_room::ToSqlValue::to_sql_value(&params.imiona)),
                    ("nazwisko", xmeta_room::ToSqlValue::to_sql_value(&params.nazwisko)),
                    ("pesel", xmeta_room::ToSqlValue::to_sql_value(&params.pesel)),
                    ("nazwa", xmeta_room::ToSqlValue::to_sql_value(&params.nazwa)),
                    (
                        "forma_prawna",
                        xmeta_room::ToSqlValue::to_sql_value(&params.forma_prawna),
                    ),
                    ("krs", xmeta_room::ToSqlValue::to_sql_value(&params.krs)),
                    ("ceidg", xmeta_room::ToSqlValue::to_sql_value(&params.ceidg)),
                    ("nip", xmeta_room::ToSqlValue::to_sql_value(&params.nip)),
                    ("regon", xmeta_room::ToSqlValue::to_sql_value(&params.regon)),
                    ("firma", xmeta_room::ToSqlValue::to_sql_value(&params.firma)),
                    (
                        "zrodlo_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_odnosnik),
                    ),
                    (
                        "zrodlo_opis",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_opis),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                ]),
            },
        ],
    )
}
