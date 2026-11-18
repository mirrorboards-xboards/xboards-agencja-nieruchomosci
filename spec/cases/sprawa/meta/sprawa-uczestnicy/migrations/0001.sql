-- Osoby i podmioty w teczce z ich danymi oraz ich role w teczce. Rola należy do udziału w teczce,
-- a nie do osoby: ta sama osoba bywa w jednej teczce klientką, a w innej pełnomocniczką. Listę
-- ról ustala typ sprawy tekstem; rdzeń sprawdza tylko format i reguły małżonka.
--
-- Uczestników wpisuje agent sprawy, a Kontroler ich czyta. Teczka trzyma tylko dane, których
-- potrzebuje sprawa (art. 5 ust. 1 lit. c RODO). Seria i numer dokumentu tożsamości zebrane jako
-- środek bezpieczeństwa finansowego leżą w rejestrze kontroli, nie tutaj.
--
-- Wersja uczestnika rośnie przy każdej zmianie danych identyfikujących, także adresu
-- i obywatelstwa. Sprawdzenie Kontrolera dotyczy jednej wersji; triggery, które zerują jego
-- aktualność, leżą w migracji sprawa-sprawdzenia (sprawa_sprawdzenia_nowa_wersja_uczestnika).
CREATE TABLE sprawa_uczestnicy_wpisy (
  id TEXT PRIMARY KEY,
  rodzaj_podmiotu TEXT NOT NULL CHECK (rodzaj_podmiotu IN ('osoba_fizyczna', 'osoba_prawna', 'jednostka_bez_osobowosci')),

  -- Osoba fizyczna. Imiona rodziców i PESEL podaje akt notarialny i wpis w księdze wieczystej.
  imiona TEXT,
  nazwisko TEXT,
  imie_ojca TEXT,
  imie_matki TEXT,
  pesel TEXT,
  -- Osoba, której PESEL nie nadano, np. część cudzoziemców (art. 25 ust. 4 ukwh).
  pesel_nie_nadano INTEGER NOT NULL DEFAULT 0 CHECK (pesel_nie_nadano IN (0, 1)),
  stan_cywilny TEXT CHECK (stan_cywilny IN ('stanu_wolnego', 'w_malzenstwie', 'w_separacji', 'rozwiedziony', 'owdowialy')),
  ustroj_majatkowy TEXT CHECK (ustroj_majatkowy IN ('wspolnosc_ustawowa', 'wspolnosc_umowna', 'rozdzielnosc_umowna', 'rozdzielnosc_sadowa', 'rozdzielnosc_z_mocy_prawa')),
  -- Art. 22¹ KC; od znacznika zależy prawo odstąpienia od umowy z biurem.
  konsument INTEGER NOT NULL DEFAULT 0 CHECK (konsument IN (0, 1)),

  -- Podmiot, a także osoba fizyczna prowadząca działalność (firma, NIP, REGON, wpis w CEIDG).
  nazwa TEXT,
  forma_prawna TEXT,
  krs TEXT,
  ceidg INTEGER NOT NULL DEFAULT 0 CHECK (ceidg IN (0, 1)),
  nip TEXT,
  regon TEXT,

  -- Kontakt jest daną roboczą biura, a nie wymogiem przepisu.
  telefon TEXT,
  email TEXT,
  kanal_kontaktu TEXT CHECK (kanal_kontaktu IN ('telefon', 'email', 'sms', 'portal', 'kanal_sprzedazy', 'list')),

  -- Klient biura ma umowę z biurem i własnego Kontrolera (R7); odnośnik do kartoteki klientów,
  -- tekstem, bo to inny pokój.
  klient_biura INTEGER NOT NULL DEFAULT 0 CHECK (klient_biura IN (0, 1)),
  kartoteka_odnosnik TEXT,
  wersja INTEGER NOT NULL DEFAULT 1 CHECK (wersja >= 1),

  -- Skąd biuro ma dane osoby: od niej samej (art. 13 RODO) albo z innego źródła (art. 14 RODO),
  -- i kiedy je pozyskało. Osoba poinformowana poza teczką ma tu opis, skąd to wiadomo
  -- (art. 14 ust. 5 lit. a RODO). Informację wysłaną z teczki wskazuje wysyłka z informacja_rodo.
  dane_od TEXT NOT NULL CHECK (dane_od IN ('od_osoby', 'z_innego_zrodla')),
  dane_pozyskano TEXT NOT NULL,
  poinformowana_poza_teczka TEXT,

  CONSTRAINT osoba_ma_imie_i_nazwisko CHECK (
    rodzaj_podmiotu <> 'osoba_fizyczna' OR (imiona IS NOT NULL AND nazwisko IS NOT NULL)
  ),
  CONSTRAINT podmiot_ma_nazwe CHECK (rodzaj_podmiotu = 'osoba_fizyczna' OR nazwa IS NOT NULL),
  CONSTRAINT podmiot_bez_danych_osoby CHECK (
    rodzaj_podmiotu = 'osoba_fizyczna'
    OR (
      imiona IS NULL AND nazwisko IS NULL AND imie_ojca IS NULL AND imie_matki IS NULL
      AND pesel IS NULL AND pesel_nie_nadano = 0 AND stan_cywilny IS NULL
      AND ustroj_majatkowy IS NULL
    )
  ),
  CONSTRAINT konsument_tylko_osoba_fizyczna CHECK (konsument = 0 OR rodzaj_podmiotu = 'osoba_fizyczna'),
  CONSTRAINT pesel_nie_nadano_wyklucza_pesel CHECK (pesel_nie_nadano = 0 OR pesel IS NULL),
  CONSTRAINT pesel_ma_11_cyfr CHECK (pesel IS NULL OR (length(pesel) = 11 AND pesel NOT GLOB '*[^0-9]*')),
  -- W separacji małżeństwo trwa, a ustrojem jest rozdzielność z mocy prawa (art. 54 KRO).
  CONSTRAINT ustroj_tylko_w_malzenstwie CHECK (
    ustroj_majatkowy IS NULL OR stan_cywilny IN ('w_malzenstwie', 'w_separacji')
  ),
  CONSTRAINT krs_albo_ceidg CHECK (krs IS NULL OR ceidg = 0)
) STRICT;

-- Miejsce zamieszkania, siedziba i adres do korespondencji, po jednym każdego rodzaju.
CREATE TABLE sprawa_uczestnicy_adresy (
  id INTEGER PRIMARY KEY,
  uczestnik_id TEXT NOT NULL REFERENCES sprawa_uczestnicy_wpisy (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('zamieszkanie', 'siedziba', 'do_korespondencji')),
  -- Kod państwa ISO 3166-1 alfa-2.
  kraj TEXT NOT NULL CHECK (length(kraj) = 2 AND kraj NOT GLOB '*[^A-Z]*'),
  miejscowosc TEXT NOT NULL,
  ulica TEXT,
  numer_budynku TEXT,
  numer_lokalu TEXT,
  kod_pocztowy TEXT,

  UNIQUE (uczestnik_id, rodzaj)
) STRICT;

-- Obywatelstwa osoby fizycznej, kilka na osobę; potrzebne, gdy nabywca może być cudzoziemcem.
CREATE TABLE sprawa_uczestnicy_obywatelstwa (
  id INTEGER PRIMARY KEY,
  uczestnik_id TEXT NOT NULL REFERENCES sprawa_uczestnicy_wpisy (id),
  -- Kod państwa ISO 3166-1 alfa-2.
  panstwo TEXT NOT NULL CHECK (length(panstwo) = 2 AND panstwo NOT GLOB '*[^A-Z]*'),

  UNIQUE (uczestnik_id, panstwo)
) STRICT;

-- Role uczestnika w teczce, np. klient, druga_strona, kupujacy, malzonek, gosc. Pełnomocnik
-- i reprezentant wynikają z umocowań (sprawa-umocowania), więc nie wpisuje się ich tutaj.
CREATE TABLE sprawa_uczestnicy_role (
  -- Klucz nadany przez bazę jest dodatni; zob. sprawa_uczestnicy_role_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  uczestnik_id TEXT NOT NULL REFERENCES sprawa_uczestnicy_wpisy (id),
  rola TEXT NOT NULL CHECK (rola <> '' AND rola NOT GLOB '*[^a-z_]*'),
  -- Uczestnik, którego rola dotyczy, np. osoba, której małżonkiem jest ten uczestnik.
  dotyczy_uczestnik_id TEXT REFERENCES sprawa_uczestnicy_wpisy (id),
  -- Rola, która przestała obowiązywać albo była pomyłką, dostaje datę końca i powód. Zostaje
  -- w teczce jako ślad sprawy (R9).
  zakonczono TEXT,
  powod_zakonczenia TEXT,

  CONSTRAINT zakonczenie_roli_z_powodem CHECK (
    (zakonczono IS NULL) = (powod_zakonczenia IS NULL)
    AND (powod_zakonczenia IS NULL OR powod_zakonczenia <> '')
  ),
  CONSTRAINT pelnomocnik_i_reprezentant_z_umocowan CHECK (rola NOT IN ('pelnomocnik', 'reprezentant')),
  CONSTRAINT malzonek_wskazuje_osobe CHECK (rola <> 'malzonek' OR dotyczy_uczestnik_id IS NOT NULL),
  CONSTRAINT rola_nie_dotyczy_siebie CHECK (dotyczy_uczestnik_id IS NOT uczestnik_id)
) STRICT;

-- Miejsce zamieszkania ma osoba fizyczna, siedzibę podmiot.
CREATE TRIGGER sprawa_uczestnicy_rodzaj_adresu_pasuje_do_podmiotu
BEFORE INSERT ON sprawa_uczestnicy_adresy
WHEN (
  SELECT
    (NEW.rodzaj = 'zamieszkanie' AND rodzaj_podmiotu <> 'osoba_fizyczna')
    OR (NEW.rodzaj = 'siedziba' AND rodzaj_podmiotu = 'osoba_fizyczna')
  FROM sprawa_uczestnicy_wpisy
  WHERE id = NEW.uczestnik_id
)
BEGIN
  SELECT RAISE(ABORT, 'miejsce zamieszkania ma tylko osoba fizyczna, a siedzibę tylko podmiot');
END;

CREATE TRIGGER sprawa_uczestnicy_rodzaj_zmienionego_adresu_pasuje_do_podmiotu
BEFORE UPDATE OF uczestnik_id, rodzaj ON sprawa_uczestnicy_adresy
WHEN (
  SELECT
    (NEW.rodzaj = 'zamieszkanie' AND rodzaj_podmiotu <> 'osoba_fizyczna')
    OR (NEW.rodzaj = 'siedziba' AND rodzaj_podmiotu = 'osoba_fizyczna')
  FROM sprawa_uczestnicy_wpisy
  WHERE id = NEW.uczestnik_id
)
BEGIN
  SELECT RAISE(ABORT, 'miejsce zamieszkania ma tylko osoba fizyczna, a siedzibę tylko podmiot');
END;

CREATE TRIGGER sprawa_uczestnicy_obywatelstwo_ma_osoba_fizyczna
BEFORE INSERT ON sprawa_uczestnicy_obywatelstwa
WHEN (
  SELECT rodzaj_podmiotu <> 'osoba_fizyczna'
  FROM sprawa_uczestnicy_wpisy
  WHERE id = NEW.uczestnik_id
)
BEGIN
  SELECT RAISE(ABORT, 'obywatelstwo ma tylko osoba fizyczna');
END;

CREATE TRIGGER sprawa_uczestnicy_zmienione_obywatelstwo_ma_osoba_fizyczna
BEFORE UPDATE OF uczestnik_id ON sprawa_uczestnicy_obywatelstwa
WHEN (
  SELECT rodzaj_podmiotu <> 'osoba_fizyczna'
  FROM sprawa_uczestnicy_wpisy
  WHERE id = NEW.uczestnik_id
)
BEGIN
  SELECT RAISE(ABORT, 'obywatelstwo ma tylko osoba fizyczna');
END;

-- Art. 37 § 1 pkt 1 KRO: zgody małżonka wymaga zbycie i nabycie z majątku wspólnego, więc
-- małżonek i osoba, której jest małżonkiem, to osoby fizyczne.
CREATE TRIGGER sprawa_uczestnicy_malzonek_osoby_fizycznej
BEFORE INSERT ON sprawa_uczestnicy_role
WHEN NEW.rola = 'malzonek'
  AND EXISTS (
    SELECT 1
    FROM sprawa_uczestnicy_wpisy
    WHERE id IN (NEW.uczestnik_id, NEW.dotyczy_uczestnik_id)
      AND rodzaj_podmiotu <> 'osoba_fizyczna'
  )
BEGIN
  SELECT RAISE(ABORT, 'małżonek i osoba, której jest małżonkiem, to osoby fizyczne');
END;

-- Rola powstaje jako obowiązująca, żeby jej koniec zawsze przeszedł przez regułę zmiany.
CREATE TRIGGER sprawa_uczestnicy_rola_powstaje_obowiazujaca
BEFORE INSERT ON sprawa_uczestnicy_role
WHEN NEW.zakonczono IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'rola powstaje jako obowiązująca');
END;

-- Roli się nie zmienia. Może się tylko raz zakończyć: rola, która przestała obowiązywać albo
-- była pomyłką, zostaje jako ślad sprawy (R9). Nowa rola to nowy wiersz.
CREATE TRIGGER sprawa_uczestnicy_role_bez_zmian
BEFORE UPDATE ON sprawa_uczestnicy_role
WHEN OLD.zakonczono IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.uczestnik_id IS NOT OLD.uczestnik_id
  OR NEW.rola IS NOT OLD.rola
  OR NEW.dotyczy_uczestnik_id IS NOT OLD.dotyczy_uczestnik_id
BEGIN
  SELECT RAISE(ABORT, 'roli w teczce się nie zmienia; rola może się tylko raz zakończyć');
END;

-- Reguły adresów, obywatelstw i małżonka sprawdzają rodzaj podmiotu przy zapisie tych wierszy;
-- zmiana rodzaju później by je ominęła.
CREATE TRIGGER sprawa_uczestnicy_rodzaj_podmiotu_niezmienny
BEFORE UPDATE OF rodzaj_podmiotu ON sprawa_uczestnicy_wpisy
WHEN NEW.rodzaj_podmiotu IS NOT OLD.rodzaj_podmiotu
BEGIN
  SELECT RAISE(ABORT, 'rodzaj podmiotu uczestnika się nie zmienia; osoba w miejsce podmiotu to nowy uczestnik');
END;

-- Dane identyfikujące to dane osoby albo podmiotu, adres zamieszkania albo siedziby
-- i obywatelstwo. Kontakt, adres do korespondencji i sytuacja rodzinna osoby nie identyfikują.
CREATE TRIGGER sprawa_uczestnicy_zmiana_danych_podnosi_wersje
AFTER UPDATE OF imiona, nazwisko, imie_ojca, imie_matki, pesel, pesel_nie_nadano, nazwa,
  forma_prawna, krs, ceidg, nip, regon
ON sprawa_uczestnicy_wpisy
WHEN NEW.imiona IS NOT OLD.imiona
  OR NEW.nazwisko IS NOT OLD.nazwisko
  OR NEW.imie_ojca IS NOT OLD.imie_ojca
  OR NEW.imie_matki IS NOT OLD.imie_matki
  OR NEW.pesel IS NOT OLD.pesel
  OR NEW.pesel_nie_nadano IS NOT OLD.pesel_nie_nadano
  OR NEW.nazwa IS NOT OLD.nazwa
  OR NEW.forma_prawna IS NOT OLD.forma_prawna
  OR NEW.krs IS NOT OLD.krs
  OR NEW.ceidg IS NOT OLD.ceidg
  OR NEW.nip IS NOT OLD.nip
  OR NEW.regon IS NOT OLD.regon
BEGIN
  UPDATE sprawa_uczestnicy_wpisy
  SET wersja = wersja + 1
  WHERE id = NEW.id;
END;

CREATE TRIGGER sprawa_uczestnicy_nowy_adres_podnosi_wersje
AFTER INSERT ON sprawa_uczestnicy_adresy
WHEN NEW.rodzaj IN ('zamieszkanie', 'siedziba')
BEGIN
  UPDATE sprawa_uczestnicy_wpisy
  SET wersja = wersja + 1
  WHERE id = NEW.uczestnik_id;
END;

CREATE TRIGGER sprawa_uczestnicy_zmiana_adresu_podnosi_wersje
AFTER UPDATE ON sprawa_uczestnicy_adresy
WHEN NEW.rodzaj IN ('zamieszkanie', 'siedziba')
  OR OLD.rodzaj IN ('zamieszkanie', 'siedziba')
BEGIN
  UPDATE sprawa_uczestnicy_wpisy
  SET wersja = wersja + 1
  WHERE id IN (OLD.uczestnik_id, NEW.uczestnik_id);
END;

-- Adres i obywatelstwo wolno usunąć jako poprawkę pomyłki (D14): dziennik pokoju XSql zachowuje
-- poprzedni stan z autorem.
CREATE TRIGGER sprawa_uczestnicy_usuniecie_adresu_podnosi_wersje
AFTER DELETE ON sprawa_uczestnicy_adresy
WHEN OLD.rodzaj IN ('zamieszkanie', 'siedziba')
BEGIN
  UPDATE sprawa_uczestnicy_wpisy
  SET wersja = wersja + 1
  WHERE id = OLD.uczestnik_id;
END;

CREATE TRIGGER sprawa_uczestnicy_nowe_obywatelstwo_podnosi_wersje
AFTER INSERT ON sprawa_uczestnicy_obywatelstwa
BEGIN
  UPDATE sprawa_uczestnicy_wpisy
  SET wersja = wersja + 1
  WHERE id = NEW.uczestnik_id;
END;

CREATE TRIGGER sprawa_uczestnicy_zmiana_obywatelstwa_podnosi_wersje
AFTER UPDATE ON sprawa_uczestnicy_obywatelstwa
BEGIN
  UPDATE sprawa_uczestnicy_wpisy
  SET wersja = wersja + 1
  WHERE id IN (OLD.uczestnik_id, NEW.uczestnik_id);
END;

CREATE TRIGGER sprawa_uczestnicy_usuniecie_obywatelstwa_podnosi_wersje
AFTER DELETE ON sprawa_uczestnicy_obywatelstwa
BEGIN
  UPDATE sprawa_uczestnicy_wpisy
  SET wersja = wersja + 1
  WHERE id = OLD.uczestnik_id;
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_uczestnicy_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_uczestnicy_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_uczestnicy_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_uczestnicy_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_uczestnicy_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER sprawa_uczestnicy_role_bez_zastepowania
BEFORE INSERT ON sprawa_uczestnicy_role
WHEN EXISTS (
  SELECT 1
  FROM sprawa_uczestnicy_role
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_uczestnicy_role_bez_usuwania
BEFORE DELETE ON sprawa_uczestnicy_role
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
