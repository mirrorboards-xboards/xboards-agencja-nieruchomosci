//! The kind `rejestr-kontroli-sprawdzenia` of the room `registers/rejestr_kontroli`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "rejestr-kontroli-sprawdzenia",
    requires: &[
        &super::rejestr_kontroli_decyzje::KIND,
        &super::rejestr_kontroli_klienci::KIND,
    ],
    migrations: &[
        r#"-- Środki bezpieczeństwa finansowego i sprawdzenia spoza ustawy, przeniesione przez Kierownika
-- kontroli z raportów Kontrolerów w dniu raportu: identyfikacja i weryfikacja, kopie dokumentów,
-- Centralny Rejestr Beneficjentów Rzeczywistych, eksponowane stanowiska polityczne, listy
-- sankcyjne, cel stosunków, źródło środków, bieżąca analiza i informacja dla klienta.
--
-- Każdy wpis ma źródło, raport i autora, a wpis sprawdzenia także Kontrolera i datę. Wpisu się nie
-- zmienia: ponowne sprawdzenie to nowy wpis, który wskazuje poprzedni i odbiera mu aktualność.
-- Organ kontroli ma zobaczyć, co biuro wiedziało w danym dniu (art. 34 ust. 3 AML). Wyjątkiem są
-- zdarzenia, które przychodzą później: akceptacja przy osobie zajmującej eksponowane stanowisko
-- polityczne, zamrożenie i informacja o nim; każde wpisuje się raz.

-- Dane identyfikacyjne z art. 36 AML klienta, beneficjenta rzeczywistego, osoby upoważnionej do
-- działania za klienta i osoby, która reprezentuje podmiot. Seria i numer dokumentu tożsamości są
-- tylko tutaj, nie w teczce.
CREATE TABLE rejestr_kontroli_sprawdzenia_identyfikacje (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_sprawdzenia_identyfikacje_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  klient_id TEXT NOT NULL REFERENCES rejestr_kontroli_klienci_wpisy (id),
  kogo TEXT NOT NULL CHECK (kogo IN ('klient', 'beneficjent', 'upowazniony', 'reprezentant')),
  rodzaj_podmiotu TEXT NOT NULL CHECK (rodzaj_podmiotu IN ('osoba_fizyczna', 'podmiot')),
  -- Osoba fizyczna (art. 36 ust. 1 pkt 1 i ust. 3 AML).
  imiona TEXT,
  nazwisko TEXT,
  -- Kod państwa ISO 3166-1 alfa-2.
  obywatelstwo TEXT CHECK (obywatelstwo IS NULL OR (length(obywatelstwo) = 2 AND obywatelstwo NOT GLOB '*[^A-Z]*')),
  pesel TEXT,
  -- Gdy PESEL nie nadano.
  data_urodzenia TEXT,
  panstwo_urodzenia TEXT,
  -- Seria i numer dokumentu tożsamości.
  dokument_tozsamosci TEXT,
  adres_zamieszkania TEXT,
  -- Osoba fizyczna przedsiębiorca.
  firma TEXT,
  nip TEXT,
  adres_dzialalnosci TEXT,
  -- Podmiot (art. 36 ust. 1 pkt 2 AML); bez NIP państwo, nazwa i numer rejestru z datą rejestracji.
  nazwa TEXT,
  forma_organizacyjna TEXT,
  adres_siedziby TEXT,
  rejestr_panstwo TEXT,
  rejestr_nazwa TEXT,
  rejestr_numer TEXT,
  rejestr_data TEXT,
  -- Struktura własności i kontroli spółki przy beneficjencie (art. 34 ust. 1 pkt 2 lit. b AML).
  struktura_wlasnosci TEXT,
  -- Dokument, z którego wynika umocowanie osoby upoważnionej (art. 34 ust. 2 AML).
  umocowanie_dokument TEXT,
  -- Źródło danej, np. dowód osobisty, odpis z KRS, wykaz stanowisk; raport, z którego Kierownik
  -- kontroli przeniósł wpis, i kto wpisał.
  zrodlo TEXT NOT NULL CHECK (zrodlo <> ''),
  raport_odnosnik TEXT NOT NULL CHECK (raport_odnosnik <> ''),
  -- Kontroler, który sprawdzał, i kiedy.
  sprawdzil TEXT NOT NULL CHECK (sprawdzil <> ''),
  sprawdzono TEXT NOT NULL,
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,
  -- Ponowne sprawdzenie wskazuje wpis, który zastępuje, i odbiera mu aktualność.
  aktualne INTEGER NOT NULL CHECK (aktualne IN (0, 1)),
  poprzedni_id INTEGER REFERENCES rejestr_kontroli_sprawdzenia_identyfikacje (id),

  -- Beneficjent, osoba upoważniona i reprezentant są osobami fizycznymi; podmiotem bywa tylko klient.
  CONSTRAINT podmiotem_bywa_tylko_klient CHECK (kogo = 'klient' OR rodzaj_podmiotu = 'osoba_fizyczna'),
  CONSTRAINT osoba_ma_imie_i_nazwisko CHECK (rodzaj_podmiotu <> 'osoba_fizyczna' OR (imiona IS NOT NULL AND nazwisko IS NOT NULL)),
  -- Art. 36 ust. 1 pkt 1 i ust. 3 AML; przy beneficjencie wystarczy imię i nazwisko (ust. 2).
  CONSTRAINT osoba_ma_obywatelstwo_pesel_albo_urodzenie_i_dokument CHECK (
    rodzaj_podmiotu <> 'osoba_fizyczna'
    OR kogo = 'beneficjent'
    OR (
      obywatelstwo IS NOT NULL
      AND (pesel IS NOT NULL OR (data_urodzenia IS NOT NULL AND panstwo_urodzenia IS NOT NULL))
      AND dokument_tozsamosci IS NOT NULL
    )
  ),
  CONSTRAINT podmiot_ma_nazwe_forme_i_siedzibe CHECK (
    rodzaj_podmiotu <> 'podmiot'
    OR (nazwa IS NOT NULL AND forma_organizacyjna IS NOT NULL AND adres_siedziby IS NOT NULL)
  ),
  CONSTRAINT podmiot_ma_nip_albo_rejestr CHECK (
    rodzaj_podmiotu <> 'podmiot'
    OR nip IS NOT NULL
    OR (rejestr_panstwo IS NOT NULL AND rejestr_nazwa IS NOT NULL AND rejestr_numer IS NOT NULL AND rejestr_data IS NOT NULL)
  ),
  CONSTRAINT upowazniony_ma_dokument_umocowania CHECK (kogo <> 'upowazniony' OR umocowanie_dokument IS NOT NULL),
  CONSTRAINT pesel_ma_11_cyfr CHECK (pesel IS NULL OR (length(pesel) = 11 AND pesel NOT GLOB '*[^0-9]*'))
) STRICT;

-- Weryfikacja danych identyfikacyjnych (art. 37 ust. 1 AML): na jakiej podstawie Kontroler je
-- potwierdził, kiedy i z jakim wynikiem.
CREATE TABLE rejestr_kontroli_sprawdzenia_weryfikacje (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_sprawdzenia_weryfikacje_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  identyfikacja_id INTEGER NOT NULL REFERENCES rejestr_kontroli_sprawdzenia_identyfikacje (id),
  podstawa TEXT NOT NULL CHECK (podstawa IN ('dowod_osobisty', 'paszport', 'inny_dokument', 'odpis_krs', 'srodek_identyfikacji_elektronicznej', 'inna')),
  wynik TEXT NOT NULL CHECK (wynik IN ('zgodne', 'uwagi', 'nie_mozna_ustalic')),
  opis TEXT,
  -- Źródło danej, np. dowód osobisty, odpis z KRS, wykaz stanowisk; raport, z którego Kierownik
  -- kontroli przeniósł wpis, i kto wpisał.
  zrodlo TEXT NOT NULL CHECK (zrodlo <> ''),
  raport_odnosnik TEXT NOT NULL CHECK (raport_odnosnik <> ''),
  -- Kontroler, który sprawdzał, i kiedy.
  sprawdzil TEXT NOT NULL CHECK (sprawdzil <> ''),
  sprawdzono TEXT NOT NULL,
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,
  -- Ponowne sprawdzenie wskazuje wpis, który zastępuje, i odbiera mu aktualność.
  aktualne INTEGER NOT NULL CHECK (aktualne IN (0, 1)),
  poprzedni_id INTEGER REFERENCES rejestr_kontroli_sprawdzenia_weryfikacje (id),

  CONSTRAINT wynik_z_uwagami_ma_opis CHECK (wynik = 'zgodne' OR opis IS NOT NULL)
) STRICT;

-- Kopie dokumentów tożsamości ze skrótem (art. 34 ust. 4 AML). Skan zostaje też w teczce.
CREATE TABLE rejestr_kontroli_sprawdzenia_kopie (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_sprawdzenia_kopie_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  identyfikacja_id INTEGER NOT NULL REFERENCES rejestr_kontroli_sprawdzenia_identyfikacje (id),
  plik_sciezka TEXT NOT NULL CHECK (plik_sciezka <> ''),
  plik_skrot TEXT NOT NULL CHECK (plik_skrot <> ''),
  -- Źródło danej, np. dowód osobisty, odpis z KRS, wykaz stanowisk; raport, z którego Kierownik
  -- kontroli przeniósł wpis, i kto wpisał.
  zrodlo TEXT NOT NULL CHECK (zrodlo <> ''),
  raport_odnosnik TEXT NOT NULL CHECK (raport_odnosnik <> ''),
  -- Kontroler, który sprawdzał, i kiedy.
  sprawdzil TEXT NOT NULL CHECK (sprawdzil <> ''),
  sprawdzono TEXT NOT NULL,
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL
) STRICT;

-- Odpis z Centralnego Rejestru Beneficjentów Rzeczywistych albo potwierdzenie zgłoszenia
-- (art. 39 ust. 1a AML) i rozbieżność z tym, co ustalił Kontroler, z jej wyjaśnieniem
-- (art. 61a ust. 1 AML).
CREATE TABLE rejestr_kontroli_sprawdzenia_crbr (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_sprawdzenia_crbr_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  klient_id TEXT NOT NULL REFERENCES rejestr_kontroli_klienci_wpisy (id),
  odpis_sciezka TEXT NOT NULL CHECK (odpis_sciezka <> ''),
  odpis_skrot TEXT NOT NULL CHECK (odpis_skrot <> ''),
  rozbieznosc INTEGER NOT NULL CHECK (rozbieznosc IN (0, 1)),
  rozbieznosc_opis TEXT,
  wyjasnienie TEXT,
  -- Źródło danej, np. dowód osobisty, odpis z KRS, wykaz stanowisk; raport, z którego Kierownik
  -- kontroli przeniósł wpis, i kto wpisał.
  zrodlo TEXT NOT NULL CHECK (zrodlo <> ''),
  raport_odnosnik TEXT NOT NULL CHECK (raport_odnosnik <> ''),
  -- Kontroler, który sprawdzał, i kiedy.
  sprawdzil TEXT NOT NULL CHECK (sprawdzil <> ''),
  sprawdzono TEXT NOT NULL,
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,
  -- Ponowne sprawdzenie wskazuje wpis, który zastępuje, i odbiera mu aktualność.
  aktualne INTEGER NOT NULL CHECK (aktualne IN (0, 1)),
  poprzedni_id INTEGER REFERENCES rejestr_kontroli_sprawdzenia_crbr (id),

  CONSTRAINT rozbieznosc_ma_opis CHECK ((rozbieznosc = 1) = (rozbieznosc_opis IS NOT NULL)),
  CONSTRAINT wyjasnienie_tylko_przy_rozbieznosci CHECK (wyjasnienie IS NULL OR rozbieznosc = 1)
) STRICT;

-- Eksponowane stanowisko polityczne klienta albo beneficjenta: oświadczenie z klauzulą
-- (art. 46 ust. 1 AML), wynik sprawdzenia wykazu stanowisk z datą, a przy takiej osobie, członku
-- jej rodziny albo bliskim współpracowniku źródło majątku i akceptacja właściciela biura na
-- stosunki (art. 46 ust. 2 i 6 AML). Akceptację wpisuje się raz, gdy zapadnie.
CREATE TABLE rejestr_kontroli_sprawdzenia_pep (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_sprawdzenia_pep_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  klient_id TEXT NOT NULL REFERENCES rejestr_kontroli_klienci_wpisy (id),
  -- Czy sprawdzenie dotyczy klienta, czy beneficjenta, i imię i nazwisko tej osoby.
  dotyczy TEXT NOT NULL CHECK (dotyczy IN ('klient', 'beneficjent')),
  osoba TEXT NOT NULL CHECK (osoba <> ''),
  oswiadczenie_sciezka TEXT,
  oswiadczenie_skrot TEXT,
  -- Oświadczenie zawiera klauzulę „Jestem świadomy odpowiedzialności karnej za złożenie
  -- fałszywego oświadczenia.”
  oswiadczenie_z_klauzula INTEGER NOT NULL DEFAULT 0 CHECK (oswiadczenie_z_klauzula IN (0, 1)),
  wykaz_sprawdzono TEXT NOT NULL,
  wynik TEXT NOT NULL CHECK (wynik IN ('nie_pep', 'pep', 'czlonek_rodziny', 'bliski_wspolpracownik')),
  -- Dzień końca pełnienia stanowiska; przez co najmniej 12 miesięcy po nim biuro uwzględnia to
  -- ryzyko (art. 46 ust. 5 AML). Termin liczy program.
  stanowisko_do TEXT,
  zrodlo_majatku TEXT,
  akceptacja_decyzja_id TEXT REFERENCES rejestr_kontroli_decyzje_wpisy (id),
  -- Źródło danej, np. dowód osobisty, odpis z KRS, wykaz stanowisk; raport, z którego Kierownik
  -- kontroli przeniósł wpis, i kto wpisał.
  zrodlo TEXT NOT NULL CHECK (zrodlo <> ''),
  raport_odnosnik TEXT NOT NULL CHECK (raport_odnosnik <> ''),
  -- Kontroler, który sprawdzał, i kiedy.
  sprawdzil TEXT NOT NULL CHECK (sprawdzil <> ''),
  sprawdzono TEXT NOT NULL,
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,
  -- Ponowne sprawdzenie wskazuje wpis, który zastępuje, i odbiera mu aktualność.
  aktualne INTEGER NOT NULL CHECK (aktualne IN (0, 1)),
  poprzedni_id INTEGER REFERENCES rejestr_kontroli_sprawdzenia_pep (id),

  CONSTRAINT oswiadczenie_z_klauzula CHECK (
    (oswiadczenie_sciezka IS NULL) = (oswiadczenie_skrot IS NULL)
    AND (oswiadczenie_sciezka IS NULL OR oswiadczenie_z_klauzula = 1)
  ),
  CONSTRAINT pep_ma_zrodlo_majatku CHECK (wynik = 'nie_pep' OR zrodlo_majatku IS NOT NULL),
  CONSTRAINT akceptacja_tylko_przy_pep CHECK (wynik <> 'nie_pep' OR akceptacja_decyzja_id IS NULL)
) STRICT;

-- Sprawdzenie list sankcyjnych (art. 118 ust. 1 AML), a przy trafieniu zamrożenie wartości
-- majątkowych klienta i chwila przekazania informacji Generalnemu Inspektorowi (art. 119 AML).
-- Zamrożenie i informację wpisuje się raz, gdy nastąpią; termin 2 dni roboczych liczy program.
CREATE TABLE rejestr_kontroli_sprawdzenia_sankcje (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_sprawdzenia_sankcje_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  klient_id TEXT NOT NULL REFERENCES rejestr_kontroli_klienci_wpisy (id),
  -- Które listy, np. „ONZ, UE, lista GIIF”.
  listy TEXT NOT NULL CHECK (listy <> ''),
  wynik TEXT NOT NULL CHECK (wynik IN ('brak_trafien', 'trafienie')),
  zamrozono TEXT,
  zamrozenie_informacja TEXT,
  -- Źródło danej, np. dowód osobisty, odpis z KRS, wykaz stanowisk; raport, z którego Kierownik
  -- kontroli przeniósł wpis, i kto wpisał.
  zrodlo TEXT NOT NULL CHECK (zrodlo <> ''),
  raport_odnosnik TEXT NOT NULL CHECK (raport_odnosnik <> ''),
  -- Kontroler, który sprawdzał, i kiedy.
  sprawdzil TEXT NOT NULL CHECK (sprawdzil <> ''),
  sprawdzono TEXT NOT NULL,
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,
  -- Ponowne sprawdzenie wskazuje wpis, który zastępuje, i odbiera mu aktualność.
  aktualne INTEGER NOT NULL CHECK (aktualne IN (0, 1)),
  poprzedni_id INTEGER REFERENCES rejestr_kontroli_sprawdzenia_sankcje (id),

  CONSTRAINT zamrozenie_tylko_przy_trafieniu CHECK (zamrozono IS NULL OR wynik = 'trafienie'),
  CONSTRAINT informacja_po_zamrozeniu CHECK (zamrozenie_informacja IS NULL OR (zamrozono IS NOT NULL AND zamrozenie_informacja >= zamrozono))
) STRICT;

-- Cel i zamierzony charakter stosunków gospodarczych (art. 34 ust. 1 pkt 3 AML).
CREATE TABLE rejestr_kontroli_sprawdzenia_cele (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_sprawdzenia_cele_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  klient_id TEXT NOT NULL REFERENCES rejestr_kontroli_klienci_wpisy (id),
  -- Np. „sprzedaż mieszkania odziedziczonego po matce”.
  cel TEXT NOT NULL CHECK (cel <> ''),
  charakter TEXT NOT NULL CHECK (charakter <> ''),
  -- Źródło danej, np. dowód osobisty, odpis z KRS, wykaz stanowisk; raport, z którego Kierownik
  -- kontroli przeniósł wpis, i kto wpisał.
  zrodlo TEXT NOT NULL CHECK (zrodlo <> ''),
  raport_odnosnik TEXT NOT NULL CHECK (raport_odnosnik <> ''),
  -- Kontroler, który sprawdzał, i kiedy.
  sprawdzil TEXT NOT NULL CHECK (sprawdzil <> ''),
  sprawdzono TEXT NOT NULL,
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,
  -- Ponowne sprawdzenie wskazuje wpis, który zastępuje, i odbiera mu aktualność.
  aktualne INTEGER NOT NULL CHECK (aktualne IN (0, 1)),
  poprzedni_id INTEGER REFERENCES rejestr_kontroli_sprawdzenia_cele (id)
) STRICT;

-- Pochodzenie pieniędzy klienta z dokumentem, gdy uzasadniają to okoliczności
-- (art. 34 ust. 1 pkt 4 lit. b AML).
CREATE TABLE rejestr_kontroli_sprawdzenia_zrodla_srodkow (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_sprawdzenia_zrodla_srodkow_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  klient_id TEXT NOT NULL REFERENCES rejestr_kontroli_klienci_wpisy (id),
  pochodzenie_srodkow TEXT NOT NULL CHECK (pochodzenie_srodkow <> ''),
  dokument_sciezka TEXT NOT NULL CHECK (dokument_sciezka <> ''),
  dokument_skrot TEXT NOT NULL CHECK (dokument_skrot <> ''),
  -- Źródło danej, np. dowód osobisty, odpis z KRS, wykaz stanowisk; raport, z którego Kierownik
  -- kontroli przeniósł wpis, i kto wpisał.
  zrodlo TEXT NOT NULL CHECK (zrodlo <> ''),
  raport_odnosnik TEXT NOT NULL CHECK (raport_odnosnik <> ''),
  -- Kontroler, który sprawdzał, i kiedy.
  sprawdzil TEXT NOT NULL CHECK (sprawdzil <> ''),
  sprawdzono TEXT NOT NULL,
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,
  -- Ponowne sprawdzenie wskazuje wpis, który zastępuje, i odbiera mu aktualność.
  aktualne INTEGER NOT NULL CHECK (aktualne IN (0, 1)),
  poprzedni_id INTEGER REFERENCES rejestr_kontroli_sprawdzenia_zrodla_srodkow (id)
) STRICT;

-- Wyniki bieżącej analizy transakcji klienta (art. 34 ust. 1 pkt 4 lit. a AML), np. porównanie
-- ceny i sposobu zapłaty z tym, co biuro wie o kliencie. Każda analiza stoi sama; 5 lat
-- przechowywania liczy się od niej (art. 49 ust. 2 AML).
CREATE TABLE rejestr_kontroli_sprawdzenia_monitorowanie (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_sprawdzenia_monitorowanie_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  klient_id TEXT NOT NULL REFERENCES rejestr_kontroli_klienci_wpisy (id),
  teczka_odnosnik TEXT NOT NULL CHECK (teczka_odnosnik <> ''),
  analiza TEXT NOT NULL CHECK (analiza <> ''),
  wynik TEXT NOT NULL CHECK (wynik IN ('zgodne', 'uwagi')),
  opis TEXT,
  -- Źródło danej, np. dowód osobisty, odpis z KRS, wykaz stanowisk; raport, z którego Kierownik
  -- kontroli przeniósł wpis, i kto wpisał.
  zrodlo TEXT NOT NULL CHECK (zrodlo <> ''),
  raport_odnosnik TEXT NOT NULL CHECK (raport_odnosnik <> ''),
  -- Kontroler, który sprawdzał, i kiedy.
  sprawdzil TEXT NOT NULL CHECK (sprawdzil <> ''),
  sprawdzono TEXT NOT NULL,
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,

  CONSTRAINT wynik_z_uwagami_ma_opis CHECK (wynik = 'zgodne' OR opis IS NOT NULL)
) STRICT;

-- Sprawdzenia spoza ustawy: tożsamość przed umową, tytuł zbywcy albo wynajmującego,
-- pełnomocnictwo i rachunek (R4, R7). Kolumny jak w teczce (sprawa_sprawdzenia_wpisy), do tego
-- odnośniki do teczki, sprawdzanego wiersza, wpisu sprawdzenia i wypłaty przy rachunku. Trafiają tu
-- także sprawdzenia drugiej strony, która nie jest klientem biura.
CREATE TABLE rejestr_kontroli_sprawdzenia_poza_ustawa (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_sprawdzenia_poza_ustawa_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Klient biura; NULL przy drugiej stronie, która nie jest klientem (R7).
  klient_id TEXT REFERENCES rejestr_kontroli_klienci_wpisy (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('tozsamosc', 'tytul', 'umocowanie', 'rachunek')),
  teczka_odnosnik TEXT NOT NULL CHECK (teczka_odnosnik <> ''),
  -- Sprawdzany wiersz teczki i jego wersja; rachunek nie ma wersji.
  wiersz_odnosnik TEXT NOT NULL CHECK (wiersz_odnosnik <> ''),
  wersja_wiersza INTEGER,
  -- Wpis w sprawa_sprawdzenia_wpisy i wypłata (sprawa_wyplaty_wpisy) przy rachunku przed wypłatą.
  sprawdzenie_odnosnik TEXT NOT NULL CHECK (sprawdzenie_odnosnik <> ''),
  wyplata_odnosnik TEXT,
  zrodlo_data TEXT,
  wynik TEXT NOT NULL CHECK (wynik IN ('zgodne', 'uwagi', 'nie_mozna_ustalic')),
  opis TEXT,
  -- Źródło danej, np. dowód osobisty, odpis z KRS, wykaz stanowisk; raport, z którego Kierownik
  -- kontroli przeniósł wpis, i kto wpisał.
  zrodlo TEXT NOT NULL CHECK (zrodlo <> ''),
  raport_odnosnik TEXT NOT NULL CHECK (raport_odnosnik <> ''),
  -- Kontroler, który sprawdzał, i kiedy.
  sprawdzil TEXT NOT NULL CHECK (sprawdzil <> ''),
  sprawdzono TEXT NOT NULL,
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,
  -- Ponowne sprawdzenie wskazuje wpis, który zastępuje, i odbiera mu aktualność.
  aktualne INTEGER NOT NULL CHECK (aktualne IN (0, 1)),
  poprzedni_id INTEGER REFERENCES rejestr_kontroli_sprawdzenia_poza_ustawa (id),

  CONSTRAINT wersja_wiersza_przy_wierszu_z_wersja CHECK ((wersja_wiersza IS NOT NULL) = (rodzaj <> 'rachunek')),
  CONSTRAINT wyplata_tylko_przy_rachunku CHECK (wyplata_odnosnik IS NULL OR rodzaj = 'rachunek'),
  CONSTRAINT wynik_z_uwagami_ma_opis CHECK (wynik = 'zgodne' OR opis IS NOT NULL)
) STRICT;

-- Kiedy i jak biuro poinformowało klienta przed nawiązaniem stosunków o przetwarzaniu jego danych
-- i o swoich obowiązkach z ustawy (art. 34 ust. 5 AML).
CREATE TABLE rejestr_kontroli_sprawdzenia_informacje (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_sprawdzenia_informacje_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  klient_id TEXT NOT NULL REFERENCES rejestr_kontroli_klienci_wpisy (id),
  poinformowano TEXT NOT NULL,
  sposob TEXT NOT NULL CHECK (sposob IN ('email', 'sms', 'portal', 'list', 'osobiscie')),
  -- Źródło danej, np. dowód osobisty, odpis z KRS, wykaz stanowisk; raport, z którego Kierownik
  -- kontroli przeniósł wpis, i kto wpisał.
  zrodlo TEXT NOT NULL CHECK (zrodlo <> ''),
  raport_odnosnik TEXT NOT NULL CHECK (raport_odnosnik <> ''),
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL
) STRICT;

-- Ponowne sprawdzenie ------------------------------------------------------------------------

-- Sprawdzenie powstaje aktualne; aktualność odbiera tylko ponowne sprawdzenie.
CREATE TRIGGER rejestr_kontroli_sprawdzenia_identyfikacje_powstaje_aktualne
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_identyfikacje
WHEN NEW.aktualne <> 1
BEGIN
  SELECT RAISE(ABORT, 'wpis sprawdzenia powstaje jako aktualny');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_identyfikacje_ponowne_wskazuje_aktualny
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_identyfikacje
WHEN NEW.poprzedni_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM rejestr_kontroli_sprawdzenia_identyfikacje
    WHERE id = NEW.poprzedni_id
      AND aktualne = 1
      AND klient_id IS NEW.klient_id
  )
BEGIN
  SELECT RAISE(ABORT, 'ponowne sprawdzenie wskazuje aktualny wpis o tym samym kliencie albo wierszu');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_identyfikacje_ponowne_zeruje_poprzedni
AFTER INSERT ON rejestr_kontroli_sprawdzenia_identyfikacje
WHEN NEW.poprzedni_id IS NOT NULL
BEGIN
  UPDATE rejestr_kontroli_sprawdzenia_identyfikacje
  SET aktualne = 0
  WHERE id = NEW.poprzedni_id;
END;

-- Sprawdzenie powstaje aktualne; aktualność odbiera tylko ponowne sprawdzenie.
CREATE TRIGGER rejestr_kontroli_sprawdzenia_weryfikacje_powstaje_aktualne
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_weryfikacje
WHEN NEW.aktualne <> 1
BEGIN
  SELECT RAISE(ABORT, 'wpis sprawdzenia powstaje jako aktualny');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_weryfikacje_ponowne_wskazuje_aktualny
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_weryfikacje
WHEN NEW.poprzedni_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM rejestr_kontroli_sprawdzenia_weryfikacje
    WHERE id = NEW.poprzedni_id
      AND aktualne = 1
      AND identyfikacja_id IS NEW.identyfikacja_id
  )
BEGIN
  SELECT RAISE(ABORT, 'ponowne sprawdzenie wskazuje aktualny wpis o tym samym kliencie albo wierszu');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_weryfikacje_ponowne_zeruje_poprzedni
AFTER INSERT ON rejestr_kontroli_sprawdzenia_weryfikacje
WHEN NEW.poprzedni_id IS NOT NULL
BEGIN
  UPDATE rejestr_kontroli_sprawdzenia_weryfikacje
  SET aktualne = 0
  WHERE id = NEW.poprzedni_id;
END;

-- Sprawdzenie powstaje aktualne; aktualność odbiera tylko ponowne sprawdzenie.
CREATE TRIGGER rejestr_kontroli_sprawdzenia_crbr_powstaje_aktualne
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_crbr
WHEN NEW.aktualne <> 1
BEGIN
  SELECT RAISE(ABORT, 'wpis sprawdzenia powstaje jako aktualny');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_crbr_ponowne_wskazuje_aktualny
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_crbr
WHEN NEW.poprzedni_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM rejestr_kontroli_sprawdzenia_crbr
    WHERE id = NEW.poprzedni_id
      AND aktualne = 1
      AND klient_id IS NEW.klient_id
  )
BEGIN
  SELECT RAISE(ABORT, 'ponowne sprawdzenie wskazuje aktualny wpis o tym samym kliencie albo wierszu');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_crbr_ponowne_zeruje_poprzedni
AFTER INSERT ON rejestr_kontroli_sprawdzenia_crbr
WHEN NEW.poprzedni_id IS NOT NULL
BEGIN
  UPDATE rejestr_kontroli_sprawdzenia_crbr
  SET aktualne = 0
  WHERE id = NEW.poprzedni_id;
END;

-- Sprawdzenie powstaje aktualne; aktualność odbiera tylko ponowne sprawdzenie.
CREATE TRIGGER rejestr_kontroli_sprawdzenia_pep_powstaje_aktualne
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_pep
WHEN NEW.aktualne <> 1
BEGIN
  SELECT RAISE(ABORT, 'wpis sprawdzenia powstaje jako aktualny');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_pep_ponowne_wskazuje_aktualny
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_pep
WHEN NEW.poprzedni_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM rejestr_kontroli_sprawdzenia_pep
    WHERE id = NEW.poprzedni_id
      AND aktualne = 1
      AND klient_id IS NEW.klient_id
  )
BEGIN
  SELECT RAISE(ABORT, 'ponowne sprawdzenie wskazuje aktualny wpis o tym samym kliencie albo wierszu');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_pep_ponowne_zeruje_poprzedni
AFTER INSERT ON rejestr_kontroli_sprawdzenia_pep
WHEN NEW.poprzedni_id IS NOT NULL
BEGIN
  UPDATE rejestr_kontroli_sprawdzenia_pep
  SET aktualne = 0
  WHERE id = NEW.poprzedni_id;
END;

-- Sprawdzenie powstaje aktualne; aktualność odbiera tylko ponowne sprawdzenie.
CREATE TRIGGER rejestr_kontroli_sprawdzenia_sankcje_powstaje_aktualne
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_sankcje
WHEN NEW.aktualne <> 1
BEGIN
  SELECT RAISE(ABORT, 'wpis sprawdzenia powstaje jako aktualny');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_sankcje_ponowne_wskazuje_aktualny
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_sankcje
WHEN NEW.poprzedni_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM rejestr_kontroli_sprawdzenia_sankcje
    WHERE id = NEW.poprzedni_id
      AND aktualne = 1
      AND klient_id IS NEW.klient_id
  )
BEGIN
  SELECT RAISE(ABORT, 'ponowne sprawdzenie wskazuje aktualny wpis o tym samym kliencie albo wierszu');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_sankcje_ponowne_zeruje_poprzedni
AFTER INSERT ON rejestr_kontroli_sprawdzenia_sankcje
WHEN NEW.poprzedni_id IS NOT NULL
BEGIN
  UPDATE rejestr_kontroli_sprawdzenia_sankcje
  SET aktualne = 0
  WHERE id = NEW.poprzedni_id;
END;

-- Sprawdzenie powstaje aktualne; aktualność odbiera tylko ponowne sprawdzenie.
CREATE TRIGGER rejestr_kontroli_sprawdzenia_cele_powstaje_aktualne
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_cele
WHEN NEW.aktualne <> 1
BEGIN
  SELECT RAISE(ABORT, 'wpis sprawdzenia powstaje jako aktualny');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_cele_ponowne_wskazuje_aktualny
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_cele
WHEN NEW.poprzedni_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM rejestr_kontroli_sprawdzenia_cele
    WHERE id = NEW.poprzedni_id
      AND aktualne = 1
      AND klient_id IS NEW.klient_id
  )
BEGIN
  SELECT RAISE(ABORT, 'ponowne sprawdzenie wskazuje aktualny wpis o tym samym kliencie albo wierszu');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_cele_ponowne_zeruje_poprzedni
AFTER INSERT ON rejestr_kontroli_sprawdzenia_cele
WHEN NEW.poprzedni_id IS NOT NULL
BEGIN
  UPDATE rejestr_kontroli_sprawdzenia_cele
  SET aktualne = 0
  WHERE id = NEW.poprzedni_id;
END;

-- Sprawdzenie powstaje aktualne; aktualność odbiera tylko ponowne sprawdzenie.
CREATE TRIGGER rejestr_kontroli_sprawdzenia_zrodla_srodkow_powstaje_aktualne
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_zrodla_srodkow
WHEN NEW.aktualne <> 1
BEGIN
  SELECT RAISE(ABORT, 'wpis sprawdzenia powstaje jako aktualny');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_zrodla_srodkow_ponowne_wskazuje_aktualny
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_zrodla_srodkow
WHEN NEW.poprzedni_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM rejestr_kontroli_sprawdzenia_zrodla_srodkow
    WHERE id = NEW.poprzedni_id
      AND aktualne = 1
      AND klient_id IS NEW.klient_id
  )
BEGIN
  SELECT RAISE(ABORT, 'ponowne sprawdzenie wskazuje aktualny wpis o tym samym kliencie albo wierszu');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_zrodla_srodkow_ponowne_zeruje_poprzedni
AFTER INSERT ON rejestr_kontroli_sprawdzenia_zrodla_srodkow
WHEN NEW.poprzedni_id IS NOT NULL
BEGIN
  UPDATE rejestr_kontroli_sprawdzenia_zrodla_srodkow
  SET aktualne = 0
  WHERE id = NEW.poprzedni_id;
END;

-- Sprawdzenie powstaje aktualne; aktualność odbiera tylko ponowne sprawdzenie.
CREATE TRIGGER rejestr_kontroli_sprawdzenia_poza_ustawa_powstaje_aktualne
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_poza_ustawa
WHEN NEW.aktualne <> 1
BEGIN
  SELECT RAISE(ABORT, 'wpis sprawdzenia powstaje jako aktualny');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_poza_ustawa_ponowne_wskazuje_aktualny
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_poza_ustawa
WHEN NEW.poprzedni_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM rejestr_kontroli_sprawdzenia_poza_ustawa
    WHERE id = NEW.poprzedni_id
      AND aktualne = 1
      AND wiersz_odnosnik IS NEW.wiersz_odnosnik
  )
BEGIN
  SELECT RAISE(ABORT, 'ponowne sprawdzenie wskazuje aktualny wpis o tym samym kliencie albo wierszu');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_poza_ustawa_ponowne_zeruje_poprzedni
AFTER INSERT ON rejestr_kontroli_sprawdzenia_poza_ustawa
WHEN NEW.poprzedni_id IS NOT NULL
BEGIN
  UPDATE rejestr_kontroli_sprawdzenia_poza_ustawa
  SET aktualne = 0
  WHERE id = NEW.poprzedni_id;
END;

-- Akceptacja przy eksponowanym stanowisku politycznym ---------------------------------------------

-- Akceptacja to zatwierdzona decyzja akceptacja_pep właściciela biura o stosunkach z tym klientem.
CREATE TRIGGER rejestr_kontroli_sprawdzenia_pep_akceptacja_wskazuje_decyzje
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_pep
WHEN NEW.akceptacja_decyzja_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM rejestr_kontroli_decyzje_zatwierdzone
    WHERE id = NEW.akceptacja_decyzja_id
      AND rodzaj = 'akceptacja_pep'
      AND dotyczy_tabela = 'rejestr_kontroli_klienci_wpisy'
      AND dotyczy_id = NEW.klient_id
  )
BEGIN
  SELECT RAISE(ABORT, 'akceptacja przy eksponowanym stanowisku wskazuje zatwierdzoną decyzję akceptacja_pep o tym kliencie');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_pep_wpisana_akceptacja_wskazuje_decyzje
BEFORE UPDATE OF akceptacja_decyzja_id ON rejestr_kontroli_sprawdzenia_pep
WHEN NEW.akceptacja_decyzja_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM rejestr_kontroli_decyzje_zatwierdzone
    WHERE id = NEW.akceptacja_decyzja_id
      AND rodzaj = 'akceptacja_pep'
      AND dotyczy_tabela = 'rejestr_kontroli_klienci_wpisy'
      AND dotyczy_id = NEW.klient_id
  )
BEGIN
  SELECT RAISE(ABORT, 'akceptacja przy eksponowanym stanowisku wskazuje zatwierdzoną decyzję akceptacja_pep o tym kliencie');
END;

-- Wpisy bez zmian ---------------------------------------------------------------------------

-- Jedyną zmianą jest utrata aktualności po ponownym sprawdzeniu.
CREATE TRIGGER rejestr_kontroli_sprawdzenia_identyfikacje_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_sprawdzenia_identyfikacje
WHEN NEW.id IS NOT OLD.id
  OR NEW.klient_id IS NOT OLD.klient_id
  OR NEW.kogo IS NOT OLD.kogo
  OR NEW.rodzaj_podmiotu IS NOT OLD.rodzaj_podmiotu
  OR NEW.imiona IS NOT OLD.imiona
  OR NEW.nazwisko IS NOT OLD.nazwisko
  OR NEW.obywatelstwo IS NOT OLD.obywatelstwo
  OR NEW.pesel IS NOT OLD.pesel
  OR NEW.data_urodzenia IS NOT OLD.data_urodzenia
  OR NEW.panstwo_urodzenia IS NOT OLD.panstwo_urodzenia
  OR NEW.dokument_tozsamosci IS NOT OLD.dokument_tozsamosci
  OR NEW.adres_zamieszkania IS NOT OLD.adres_zamieszkania
  OR NEW.firma IS NOT OLD.firma
  OR NEW.nip IS NOT OLD.nip
  OR NEW.adres_dzialalnosci IS NOT OLD.adres_dzialalnosci
  OR NEW.nazwa IS NOT OLD.nazwa
  OR NEW.forma_organizacyjna IS NOT OLD.forma_organizacyjna
  OR NEW.adres_siedziby IS NOT OLD.adres_siedziby
  OR NEW.rejestr_panstwo IS NOT OLD.rejestr_panstwo
  OR NEW.rejestr_nazwa IS NOT OLD.rejestr_nazwa
  OR NEW.rejestr_numer IS NOT OLD.rejestr_numer
  OR NEW.rejestr_data IS NOT OLD.rejestr_data
  OR NEW.struktura_wlasnosci IS NOT OLD.struktura_wlasnosci
  OR NEW.umocowanie_dokument IS NOT OLD.umocowanie_dokument
  OR NEW.zrodlo IS NOT OLD.zrodlo
  OR NEW.raport_odnosnik IS NOT OLD.raport_odnosnik
  OR NEW.sprawdzil IS NOT OLD.sprawdzil
  OR NEW.sprawdzono IS NOT OLD.sprawdzono
  OR NEW.wpisal IS NOT OLD.wpisal
  OR NEW.wpisano IS NOT OLD.wpisano
  OR NEW.poprzedni_id IS NOT OLD.poprzedni_id
  OR (
    NEW.aktualne IS NOT OLD.aktualne
    AND NOT (
      OLD.aktualne = 1
      AND NEW.aktualne = 0
      AND EXISTS (
        SELECT 1
        FROM rejestr_kontroli_sprawdzenia_identyfikacje
        WHERE poprzedni_id = OLD.id
      )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'wpisu sprawdzenia się nie zmienia; ponowne sprawdzenie to nowy wpis');
END;

-- Jedyną zmianą jest utrata aktualności po ponownym sprawdzeniu.
CREATE TRIGGER rejestr_kontroli_sprawdzenia_weryfikacje_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_sprawdzenia_weryfikacje
WHEN NEW.id IS NOT OLD.id
  OR NEW.identyfikacja_id IS NOT OLD.identyfikacja_id
  OR NEW.podstawa IS NOT OLD.podstawa
  OR NEW.wynik IS NOT OLD.wynik
  OR NEW.opis IS NOT OLD.opis
  OR NEW.zrodlo IS NOT OLD.zrodlo
  OR NEW.raport_odnosnik IS NOT OLD.raport_odnosnik
  OR NEW.sprawdzil IS NOT OLD.sprawdzil
  OR NEW.sprawdzono IS NOT OLD.sprawdzono
  OR NEW.wpisal IS NOT OLD.wpisal
  OR NEW.wpisano IS NOT OLD.wpisano
  OR NEW.poprzedni_id IS NOT OLD.poprzedni_id
  OR (
    NEW.aktualne IS NOT OLD.aktualne
    AND NOT (
      OLD.aktualne = 1
      AND NEW.aktualne = 0
      AND EXISTS (
        SELECT 1
        FROM rejestr_kontroli_sprawdzenia_weryfikacje
        WHERE poprzedni_id = OLD.id
      )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'wpisu sprawdzenia się nie zmienia; ponowne sprawdzenie to nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_kopie_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_sprawdzenia_kopie
BEGIN
  SELECT RAISE(ABORT, 'wpisu sprawdzenia się nie zmienia; ponowne sprawdzenie to nowy wpis');
END;

-- Jedyną zmianą jest utrata aktualności po ponownym sprawdzeniu.
CREATE TRIGGER rejestr_kontroli_sprawdzenia_crbr_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_sprawdzenia_crbr
WHEN NEW.id IS NOT OLD.id
  OR NEW.klient_id IS NOT OLD.klient_id
  OR NEW.odpis_sciezka IS NOT OLD.odpis_sciezka
  OR NEW.odpis_skrot IS NOT OLD.odpis_skrot
  OR NEW.rozbieznosc IS NOT OLD.rozbieznosc
  OR NEW.rozbieznosc_opis IS NOT OLD.rozbieznosc_opis
  OR NEW.wyjasnienie IS NOT OLD.wyjasnienie
  OR NEW.zrodlo IS NOT OLD.zrodlo
  OR NEW.raport_odnosnik IS NOT OLD.raport_odnosnik
  OR NEW.sprawdzil IS NOT OLD.sprawdzil
  OR NEW.sprawdzono IS NOT OLD.sprawdzono
  OR NEW.wpisal IS NOT OLD.wpisal
  OR NEW.wpisano IS NOT OLD.wpisano
  OR NEW.poprzedni_id IS NOT OLD.poprzedni_id
  OR (
    NEW.aktualne IS NOT OLD.aktualne
    AND NOT (
      OLD.aktualne = 1
      AND NEW.aktualne = 0
      AND EXISTS (
        SELECT 1
        FROM rejestr_kontroli_sprawdzenia_crbr
        WHERE poprzedni_id = OLD.id
      )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'wpisu sprawdzenia się nie zmienia; ponowne sprawdzenie to nowy wpis');
END;

-- Jedyne zmiany to utrata aktualności po ponownym sprawdzeniu i jednorazowy wpis zdarzenia,
-- które przyszło później.
CREATE TRIGGER rejestr_kontroli_sprawdzenia_pep_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_sprawdzenia_pep
WHEN NEW.id IS NOT OLD.id
  OR NEW.klient_id IS NOT OLD.klient_id
  OR NEW.dotyczy IS NOT OLD.dotyczy
  OR NEW.osoba IS NOT OLD.osoba
  OR NEW.oswiadczenie_sciezka IS NOT OLD.oswiadczenie_sciezka
  OR NEW.oswiadczenie_skrot IS NOT OLD.oswiadczenie_skrot
  OR NEW.oswiadczenie_z_klauzula IS NOT OLD.oswiadczenie_z_klauzula
  OR NEW.wykaz_sprawdzono IS NOT OLD.wykaz_sprawdzono
  OR NEW.wynik IS NOT OLD.wynik
  OR NEW.stanowisko_do IS NOT OLD.stanowisko_do
  OR NEW.zrodlo_majatku IS NOT OLD.zrodlo_majatku
  OR NEW.zrodlo IS NOT OLD.zrodlo
  OR NEW.raport_odnosnik IS NOT OLD.raport_odnosnik
  OR NEW.sprawdzil IS NOT OLD.sprawdzil
  OR NEW.sprawdzono IS NOT OLD.sprawdzono
  OR NEW.wpisal IS NOT OLD.wpisal
  OR NEW.wpisano IS NOT OLD.wpisano
  OR NEW.poprzedni_id IS NOT OLD.poprzedni_id
  OR (OLD.akceptacja_decyzja_id IS NOT NULL AND NEW.akceptacja_decyzja_id IS NOT OLD.akceptacja_decyzja_id)
  OR (
    NEW.aktualne IS NOT OLD.aktualne
    AND NOT (
      OLD.aktualne = 1
      AND NEW.aktualne = 0
      AND EXISTS (
        SELECT 1
        FROM rejestr_kontroli_sprawdzenia_pep
        WHERE poprzedni_id = OLD.id
      )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'wpisu sprawdzenia się nie zmienia; ponowne sprawdzenie to nowy wpis');
END;

-- Jedyne zmiany to utrata aktualności po ponownym sprawdzeniu i jednorazowy wpis zdarzenia,
-- które przyszło później.
CREATE TRIGGER rejestr_kontroli_sprawdzenia_sankcje_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_sprawdzenia_sankcje
WHEN NEW.id IS NOT OLD.id
  OR NEW.klient_id IS NOT OLD.klient_id
  OR NEW.listy IS NOT OLD.listy
  OR NEW.wynik IS NOT OLD.wynik
  OR NEW.zrodlo IS NOT OLD.zrodlo
  OR NEW.raport_odnosnik IS NOT OLD.raport_odnosnik
  OR NEW.sprawdzil IS NOT OLD.sprawdzil
  OR NEW.sprawdzono IS NOT OLD.sprawdzono
  OR NEW.wpisal IS NOT OLD.wpisal
  OR NEW.wpisano IS NOT OLD.wpisano
  OR NEW.poprzedni_id IS NOT OLD.poprzedni_id
  OR (OLD.zamrozono IS NOT NULL AND NEW.zamrozono IS NOT OLD.zamrozono)
  OR (OLD.zamrozenie_informacja IS NOT NULL AND NEW.zamrozenie_informacja IS NOT OLD.zamrozenie_informacja)
  OR (
    NEW.aktualne IS NOT OLD.aktualne
    AND NOT (
      OLD.aktualne = 1
      AND NEW.aktualne = 0
      AND EXISTS (
        SELECT 1
        FROM rejestr_kontroli_sprawdzenia_sankcje
        WHERE poprzedni_id = OLD.id
      )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'wpisu sprawdzenia się nie zmienia; ponowne sprawdzenie to nowy wpis');
END;

-- Jedyną zmianą jest utrata aktualności po ponownym sprawdzeniu.
CREATE TRIGGER rejestr_kontroli_sprawdzenia_cele_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_sprawdzenia_cele
WHEN NEW.id IS NOT OLD.id
  OR NEW.klient_id IS NOT OLD.klient_id
  OR NEW.cel IS NOT OLD.cel
  OR NEW.charakter IS NOT OLD.charakter
  OR NEW.zrodlo IS NOT OLD.zrodlo
  OR NEW.raport_odnosnik IS NOT OLD.raport_odnosnik
  OR NEW.sprawdzil IS NOT OLD.sprawdzil
  OR NEW.sprawdzono IS NOT OLD.sprawdzono
  OR NEW.wpisal IS NOT OLD.wpisal
  OR NEW.wpisano IS NOT OLD.wpisano
  OR NEW.poprzedni_id IS NOT OLD.poprzedni_id
  OR (
    NEW.aktualne IS NOT OLD.aktualne
    AND NOT (
      OLD.aktualne = 1
      AND NEW.aktualne = 0
      AND EXISTS (
        SELECT 1
        FROM rejestr_kontroli_sprawdzenia_cele
        WHERE poprzedni_id = OLD.id
      )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'wpisu sprawdzenia się nie zmienia; ponowne sprawdzenie to nowy wpis');
END;

-- Jedyną zmianą jest utrata aktualności po ponownym sprawdzeniu.
CREATE TRIGGER rejestr_kontroli_sprawdzenia_zrodla_srodkow_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_sprawdzenia_zrodla_srodkow
WHEN NEW.id IS NOT OLD.id
  OR NEW.klient_id IS NOT OLD.klient_id
  OR NEW.pochodzenie_srodkow IS NOT OLD.pochodzenie_srodkow
  OR NEW.dokument_sciezka IS NOT OLD.dokument_sciezka
  OR NEW.dokument_skrot IS NOT OLD.dokument_skrot
  OR NEW.zrodlo IS NOT OLD.zrodlo
  OR NEW.raport_odnosnik IS NOT OLD.raport_odnosnik
  OR NEW.sprawdzil IS NOT OLD.sprawdzil
  OR NEW.sprawdzono IS NOT OLD.sprawdzono
  OR NEW.wpisal IS NOT OLD.wpisal
  OR NEW.wpisano IS NOT OLD.wpisano
  OR NEW.poprzedni_id IS NOT OLD.poprzedni_id
  OR (
    NEW.aktualne IS NOT OLD.aktualne
    AND NOT (
      OLD.aktualne = 1
      AND NEW.aktualne = 0
      AND EXISTS (
        SELECT 1
        FROM rejestr_kontroli_sprawdzenia_zrodla_srodkow
        WHERE poprzedni_id = OLD.id
      )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'wpisu sprawdzenia się nie zmienia; ponowne sprawdzenie to nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_monitorowanie_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_sprawdzenia_monitorowanie
BEGIN
  SELECT RAISE(ABORT, 'wpisu sprawdzenia się nie zmienia; ponowne sprawdzenie to nowy wpis');
END;

-- Jedyną zmianą jest utrata aktualności po ponownym sprawdzeniu.
CREATE TRIGGER rejestr_kontroli_sprawdzenia_poza_ustawa_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_sprawdzenia_poza_ustawa
WHEN NEW.id IS NOT OLD.id
  OR NEW.klient_id IS NOT OLD.klient_id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.teczka_odnosnik IS NOT OLD.teczka_odnosnik
  OR NEW.wiersz_odnosnik IS NOT OLD.wiersz_odnosnik
  OR NEW.wersja_wiersza IS NOT OLD.wersja_wiersza
  OR NEW.sprawdzenie_odnosnik IS NOT OLD.sprawdzenie_odnosnik
  OR NEW.wyplata_odnosnik IS NOT OLD.wyplata_odnosnik
  OR NEW.zrodlo_data IS NOT OLD.zrodlo_data
  OR NEW.wynik IS NOT OLD.wynik
  OR NEW.opis IS NOT OLD.opis
  OR NEW.zrodlo IS NOT OLD.zrodlo
  OR NEW.raport_odnosnik IS NOT OLD.raport_odnosnik
  OR NEW.sprawdzil IS NOT OLD.sprawdzil
  OR NEW.sprawdzono IS NOT OLD.sprawdzono
  OR NEW.wpisal IS NOT OLD.wpisal
  OR NEW.wpisano IS NOT OLD.wpisano
  OR NEW.poprzedni_id IS NOT OLD.poprzedni_id
  OR (
    NEW.aktualne IS NOT OLD.aktualne
    AND NOT (
      OLD.aktualne = 1
      AND NEW.aktualne = 0
      AND EXISTS (
        SELECT 1
        FROM rejestr_kontroli_sprawdzenia_poza_ustawa
        WHERE poprzedni_id = OLD.id
      )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'wpisu sprawdzenia się nie zmienia; ponowne sprawdzenie to nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_informacje_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_sprawdzenia_informacje
BEGIN
  SELECT RAISE(ABORT, 'wpisu sprawdzenia się nie zmienia; ponowne sprawdzenie to nowy wpis');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabel z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER rejestr_kontroli_sprawdzenia_identyfikacje_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_identyfikacje
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_sprawdzenia_identyfikacje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_weryfikacje_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_weryfikacje
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_sprawdzenia_weryfikacje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_kopie_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_kopie
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_sprawdzenia_kopie
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_crbr_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_crbr
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_sprawdzenia_crbr
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_pep_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_pep
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_sprawdzenia_pep
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_sankcje_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_sankcje
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_sprawdzenia_sankcje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_cele_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_cele
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_sprawdzenia_cele
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_zrodla_srodkow_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_zrodla_srodkow
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_sprawdzenia_zrodla_srodkow
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_monitorowanie_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_monitorowanie
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_sprawdzenia_monitorowanie
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_poza_ustawa_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_poza_ustawa
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_sprawdzenia_poza_ustawa
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_informacje_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_sprawdzenia_informacje
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_sprawdzenia_informacje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

-- Art. 49 AML i R9: dokumentację przechowuje się co najmniej 5 lat, a na razie bez terminu.
CREATE TRIGGER rejestr_kontroli_sprawdzenia_identyfikacje_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_sprawdzenia_identyfikacje
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_weryfikacje_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_sprawdzenia_weryfikacje
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_kopie_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_sprawdzenia_kopie
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_crbr_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_sprawdzenia_crbr
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_pep_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_sprawdzenia_pep
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_sankcje_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_sprawdzenia_sankcje
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_cele_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_sprawdzenia_cele
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_zrodla_srodkow_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_sprawdzenia_zrodla_srodkow
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_monitorowanie_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_sprawdzenia_monitorowanie
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_poza_ustawa_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_sprawdzenia_poza_ustawa
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_sprawdzenia_informacje_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_sprawdzenia_informacje
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;
"#,
    ],
};

/// The values of column `identyfikacje.kogo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentyfikacjeKogo {
    Klient,
    Beneficjent,
    Upowazniony,
    Reprezentant,
}

impl IdentyfikacjeKogo {
    /// Every value, in the order of the catalog.
    pub const ALL: &[IdentyfikacjeKogo] = &[
        IdentyfikacjeKogo::Klient,
        IdentyfikacjeKogo::Beneficjent,
        IdentyfikacjeKogo::Upowazniony,
        IdentyfikacjeKogo::Reprezentant,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            IdentyfikacjeKogo::Klient => "klient",
            IdentyfikacjeKogo::Beneficjent => "beneficjent",
            IdentyfikacjeKogo::Upowazniony => "upowazniony",
            IdentyfikacjeKogo::Reprezentant => "reprezentant",
        }
    }
}

impl xmeta_room::FromSqlValue for IdentyfikacjeKogo {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "klient" => Ok(IdentyfikacjeKogo::Klient),
                    "beneficjent" => Ok(IdentyfikacjeKogo::Beneficjent),
                    "upowazniony" => Ok(IdentyfikacjeKogo::Upowazniony),
                    "reprezentant" => Ok(IdentyfikacjeKogo::Reprezentant),
                    other => Err(format!("{other} is no value of IdentyfikacjeKogo")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for IdentyfikacjeKogo {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `identyfikacje.rodzaj_podmiotu`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentyfikacjeRodzajPodmiotu {
    OsobaFizyczna,
    Podmiot,
}

impl IdentyfikacjeRodzajPodmiotu {
    /// Every value, in the order of the catalog.
    pub const ALL: &[IdentyfikacjeRodzajPodmiotu] = &[
        IdentyfikacjeRodzajPodmiotu::OsobaFizyczna,
        IdentyfikacjeRodzajPodmiotu::Podmiot,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            IdentyfikacjeRodzajPodmiotu::OsobaFizyczna => "osoba_fizyczna",
            IdentyfikacjeRodzajPodmiotu::Podmiot => "podmiot",
        }
    }
}

impl xmeta_room::FromSqlValue for IdentyfikacjeRodzajPodmiotu {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "osoba_fizyczna" => Ok(IdentyfikacjeRodzajPodmiotu::OsobaFizyczna),
                    "podmiot" => Ok(IdentyfikacjeRodzajPodmiotu::Podmiot),
                    other => {
                        Err(
                            format!("{other} is no value of IdentyfikacjeRodzajPodmiotu"),
                        )
                    }
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for IdentyfikacjeRodzajPodmiotu {
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

/// The values of column `monitorowanie.wynik`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MonitorowanieWynik {
    Zgodne,
    Uwagi,
}

impl MonitorowanieWynik {
    /// Every value, in the order of the catalog.
    pub const ALL: &[MonitorowanieWynik] = &[
        MonitorowanieWynik::Zgodne,
        MonitorowanieWynik::Uwagi,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            MonitorowanieWynik::Zgodne => "zgodne",
            MonitorowanieWynik::Uwagi => "uwagi",
        }
    }
}

impl xmeta_room::FromSqlValue for MonitorowanieWynik {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "zgodne" => Ok(MonitorowanieWynik::Zgodne),
                    "uwagi" => Ok(MonitorowanieWynik::Uwagi),
                    other => Err(format!("{other} is no value of MonitorowanieWynik")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for MonitorowanieWynik {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `pep.dotyczy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PepDotyczy {
    Klient,
    Beneficjent,
}

impl PepDotyczy {
    /// Every value, in the order of the catalog.
    pub const ALL: &[PepDotyczy] = &[PepDotyczy::Klient, PepDotyczy::Beneficjent];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            PepDotyczy::Klient => "klient",
            PepDotyczy::Beneficjent => "beneficjent",
        }
    }
}

impl xmeta_room::FromSqlValue for PepDotyczy {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "klient" => Ok(PepDotyczy::Klient),
                    "beneficjent" => Ok(PepDotyczy::Beneficjent),
                    other => Err(format!("{other} is no value of PepDotyczy")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for PepDotyczy {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `pep.wynik`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PepWynik {
    NiePep,
    Pep,
    CzlonekRodziny,
    BliskiWspolpracownik,
}

impl PepWynik {
    /// Every value, in the order of the catalog.
    pub const ALL: &[PepWynik] = &[
        PepWynik::NiePep,
        PepWynik::Pep,
        PepWynik::CzlonekRodziny,
        PepWynik::BliskiWspolpracownik,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            PepWynik::NiePep => "nie_pep",
            PepWynik::Pep => "pep",
            PepWynik::CzlonekRodziny => "czlonek_rodziny",
            PepWynik::BliskiWspolpracownik => "bliski_wspolpracownik",
        }
    }
}

impl xmeta_room::FromSqlValue for PepWynik {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "nie_pep" => Ok(PepWynik::NiePep),
                    "pep" => Ok(PepWynik::Pep),
                    "czlonek_rodziny" => Ok(PepWynik::CzlonekRodziny),
                    "bliski_wspolpracownik" => Ok(PepWynik::BliskiWspolpracownik),
                    other => Err(format!("{other} is no value of PepWynik")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for PepWynik {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `poza_ustawa.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PozaUstawaRodzaj {
    Tozsamosc,
    Tytul,
    Umocowanie,
    Rachunek,
}

impl PozaUstawaRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[PozaUstawaRodzaj] = &[
        PozaUstawaRodzaj::Tozsamosc,
        PozaUstawaRodzaj::Tytul,
        PozaUstawaRodzaj::Umocowanie,
        PozaUstawaRodzaj::Rachunek,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            PozaUstawaRodzaj::Tozsamosc => "tozsamosc",
            PozaUstawaRodzaj::Tytul => "tytul",
            PozaUstawaRodzaj::Umocowanie => "umocowanie",
            PozaUstawaRodzaj::Rachunek => "rachunek",
        }
    }
}

impl xmeta_room::FromSqlValue for PozaUstawaRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "tozsamosc" => Ok(PozaUstawaRodzaj::Tozsamosc),
                    "tytul" => Ok(PozaUstawaRodzaj::Tytul),
                    "umocowanie" => Ok(PozaUstawaRodzaj::Umocowanie),
                    "rachunek" => Ok(PozaUstawaRodzaj::Rachunek),
                    other => Err(format!("{other} is no value of PozaUstawaRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for PozaUstawaRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `poza_ustawa.wynik`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PozaUstawaWynik {
    Zgodne,
    Uwagi,
    NieMoznaUstalic,
}

impl PozaUstawaWynik {
    /// Every value, in the order of the catalog.
    pub const ALL: &[PozaUstawaWynik] = &[
        PozaUstawaWynik::Zgodne,
        PozaUstawaWynik::Uwagi,
        PozaUstawaWynik::NieMoznaUstalic,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            PozaUstawaWynik::Zgodne => "zgodne",
            PozaUstawaWynik::Uwagi => "uwagi",
            PozaUstawaWynik::NieMoznaUstalic => "nie_mozna_ustalic",
        }
    }
}

impl xmeta_room::FromSqlValue for PozaUstawaWynik {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "zgodne" => Ok(PozaUstawaWynik::Zgodne),
                    "uwagi" => Ok(PozaUstawaWynik::Uwagi),
                    "nie_mozna_ustalic" => Ok(PozaUstawaWynik::NieMoznaUstalic),
                    other => Err(format!("{other} is no value of PozaUstawaWynik")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for PozaUstawaWynik {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `sankcje.wynik`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SankcjeWynik {
    BrakTrafien,
    Trafienie,
}

impl SankcjeWynik {
    /// Every value, in the order of the catalog.
    pub const ALL: &[SankcjeWynik] = &[
        SankcjeWynik::BrakTrafien,
        SankcjeWynik::Trafienie,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            SankcjeWynik::BrakTrafien => "brak_trafien",
            SankcjeWynik::Trafienie => "trafienie",
        }
    }
}

impl xmeta_room::FromSqlValue for SankcjeWynik {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "brak_trafien" => Ok(SankcjeWynik::BrakTrafien),
                    "trafienie" => Ok(SankcjeWynik::Trafienie),
                    other => Err(format!("{other} is no value of SankcjeWynik")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for SankcjeWynik {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `weryfikacje.podstawa`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeryfikacjePodstawa {
    DowodOsobisty,
    Paszport,
    InnyDokument,
    OdpisKrs,
    SrodekIdentyfikacjiElektronicznej,
    Inna,
}

impl WeryfikacjePodstawa {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WeryfikacjePodstawa] = &[
        WeryfikacjePodstawa::DowodOsobisty,
        WeryfikacjePodstawa::Paszport,
        WeryfikacjePodstawa::InnyDokument,
        WeryfikacjePodstawa::OdpisKrs,
        WeryfikacjePodstawa::SrodekIdentyfikacjiElektronicznej,
        WeryfikacjePodstawa::Inna,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WeryfikacjePodstawa::DowodOsobisty => "dowod_osobisty",
            WeryfikacjePodstawa::Paszport => "paszport",
            WeryfikacjePodstawa::InnyDokument => "inny_dokument",
            WeryfikacjePodstawa::OdpisKrs => "odpis_krs",
            WeryfikacjePodstawa::SrodekIdentyfikacjiElektronicznej => {
                "srodek_identyfikacji_elektronicznej"
            }
            WeryfikacjePodstawa::Inna => "inna",
        }
    }
}

impl xmeta_room::FromSqlValue for WeryfikacjePodstawa {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "dowod_osobisty" => Ok(WeryfikacjePodstawa::DowodOsobisty),
                    "paszport" => Ok(WeryfikacjePodstawa::Paszport),
                    "inny_dokument" => Ok(WeryfikacjePodstawa::InnyDokument),
                    "odpis_krs" => Ok(WeryfikacjePodstawa::OdpisKrs),
                    "srodek_identyfikacji_elektronicznej" => {
                        Ok(WeryfikacjePodstawa::SrodekIdentyfikacjiElektronicznej)
                    }
                    "inna" => Ok(WeryfikacjePodstawa::Inna),
                    other => Err(format!("{other} is no value of WeryfikacjePodstawa")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WeryfikacjePodstawa {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The parameters of `cele_stosunkow_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CeleStosunkowKlientaParams {
    pub klient_id: String,
}

/// A row of `cele_stosunkow_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct CeleStosunkowKlientaRow {
    pub id: i64,
    pub klient_id: String,
    pub cel: String,
    pub charakter: String,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub wpisal: String,
    pub wpisano: String,
    pub aktualne: bool,
    pub poprzedni_id: Option<i64>,
}

impl CeleStosunkowKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(CeleStosunkowKlientaRow {
            id: row.get("id")?,
            klient_id: row.get("klient_id")?,
            cel: row.get("cel")?,
            charakter: row.get("charakter")?,
            zrodlo: row.get("zrodlo")?,
            raport_odnosnik: row.get("raport_odnosnik")?,
            sprawdzil: row.get("sprawdzil")?,
            sprawdzono: row.get("sprawdzono")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
            aktualne: row.get("aktualne")?,
            poprzedni_id: row.get("poprzedni_id")?,
        })
    }
}

/// Cel i charakter stosunków z klientem, najnowsze pierwsze.
pub fn cele_stosunkow_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &CeleStosunkowKlientaParams,
) -> Result<Vec<CeleStosunkowKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_sprawdzenia_cele
WHERE klient_id = :klient_id
ORDER BY id DESC"#,
        parameters: Vec::from([
            ("klient_id", xmeta_room::ToSqlValue::to_sql_value(&params.klient_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(CeleStosunkowKlientaRow::from_row).collect()
}

/// The parameters of `wpisz_cel_stosunkow`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszCelStosunkowParams {
    pub klient_id: String,
    pub cel: String,
    pub charakter: String,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub wpisal: String,
    pub wpisano: String,
    pub poprzedni_id: Option<i64>,
}

/// Cel i zamierzony charakter stosunków gospodarczych.
pub fn wpisz_cel_stosunkow(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszCelStosunkowParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_sprawdzenia_cele (klient_id, cel, charakter, zrodlo, raport_odnosnik,
  sprawdzil, sprawdzono, wpisal, wpisano, aktualne, poprzedni_id)
VALUES (:klient_id, :cel, :charakter, :zrodlo, :raport_odnosnik, :sprawdzil, :sprawdzono, :wpisal,
  :wpisano, 1, :poprzedni_id)"#,
                parameters: Vec::from([
                    (
                        "klient_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_id),
                    ),
                    ("cel", xmeta_room::ToSqlValue::to_sql_value(&params.cel)),
                    (
                        "charakter",
                        xmeta_room::ToSqlValue::to_sql_value(&params.charakter),
                    ),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                    (
                        "raport_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.raport_odnosnik),
                    ),
                    (
                        "sprawdzil",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzil),
                    ),
                    (
                        "sprawdzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzono),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                    (
                        "poprzedni_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprzedni_id),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `odpisy_crbr_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdpisyCrbrKlientaParams {
    pub klient_id: String,
}

/// A row of `odpisy_crbr_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct OdpisyCrbrKlientaRow {
    pub id: i64,
    pub klient_id: String,
    pub odpis_sciezka: String,
    pub odpis_skrot: String,
    pub rozbieznosc: bool,
    pub rozbieznosc_opis: Option<String>,
    pub wyjasnienie: Option<String>,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub wpisal: String,
    pub wpisano: String,
    pub aktualne: bool,
    pub poprzedni_id: Option<i64>,
}

impl OdpisyCrbrKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(OdpisyCrbrKlientaRow {
            id: row.get("id")?,
            klient_id: row.get("klient_id")?,
            odpis_sciezka: row.get("odpis_sciezka")?,
            odpis_skrot: row.get("odpis_skrot")?,
            rozbieznosc: row.get("rozbieznosc")?,
            rozbieznosc_opis: row.get("rozbieznosc_opis")?,
            wyjasnienie: row.get("wyjasnienie")?,
            zrodlo: row.get("zrodlo")?,
            raport_odnosnik: row.get("raport_odnosnik")?,
            sprawdzil: row.get("sprawdzil")?,
            sprawdzono: row.get("sprawdzono")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
            aktualne: row.get("aktualne")?,
            poprzedni_id: row.get("poprzedni_id")?,
        })
    }
}

/// Odpisy z Centralnego Rejestru Beneficjentów Rzeczywistych i rozbieżności, najnowsze pierwsze.
pub fn odpisy_crbr_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &OdpisyCrbrKlientaParams,
) -> Result<Vec<OdpisyCrbrKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_sprawdzenia_crbr
WHERE klient_id = :klient_id
ORDER BY id DESC"#,
        parameters: Vec::from([
            ("klient_id", xmeta_room::ToSqlValue::to_sql_value(&params.klient_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(OdpisyCrbrKlientaRow::from_row).collect()
}

/// The parameters of `wpisz_odpis_crbr`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszOdpisCrbrParams {
    pub klient_id: String,
    pub odpis_sciezka: String,
    pub odpis_skrot: String,
    pub rozbieznosc: bool,
    pub rozbieznosc_opis: Option<String>,
    pub wyjasnienie: Option<String>,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub wpisal: String,
    pub wpisano: String,
    pub poprzedni_id: Option<i64>,
}

/// Odpis albo potwierdzenie zgłoszenia z rozbieżnością i jej wyjaśnieniem.
pub fn wpisz_odpis_crbr(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszOdpisCrbrParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_sprawdzenia_crbr (klient_id, odpis_sciezka, odpis_skrot, rozbieznosc,
  rozbieznosc_opis, wyjasnienie, zrodlo, raport_odnosnik, sprawdzil, sprawdzono, wpisal, wpisano,
  aktualne, poprzedni_id)
VALUES (:klient_id, :odpis_sciezka, :odpis_skrot, :rozbieznosc, :rozbieznosc_opis, :wyjasnienie,
  :zrodlo, :raport_odnosnik, :sprawdzil, :sprawdzono, :wpisal, :wpisano, 1, :poprzedni_id)"#,
                parameters: Vec::from([
                    (
                        "klient_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_id),
                    ),
                    (
                        "odpis_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odpis_sciezka),
                    ),
                    (
                        "odpis_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odpis_skrot),
                    ),
                    (
                        "rozbieznosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rozbieznosc),
                    ),
                    (
                        "rozbieznosc_opis",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rozbieznosc_opis),
                    ),
                    (
                        "wyjasnienie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wyjasnienie),
                    ),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                    (
                        "raport_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.raport_odnosnik),
                    ),
                    (
                        "sprawdzil",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzil),
                    ),
                    (
                        "sprawdzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzono),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                    (
                        "poprzedni_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprzedni_id),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `identyfikacje_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct IdentyfikacjeKlientaParams {
    pub klient_id: String,
}

/// A row of `identyfikacje_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentyfikacjeKlientaRow {
    pub id: i64,
    pub klient_id: String,
    pub kogo: IdentyfikacjeKogo,
    pub rodzaj_podmiotu: IdentyfikacjeRodzajPodmiotu,
    pub imiona: Option<String>,
    pub nazwisko: Option<String>,
    pub obywatelstwo: Option<String>,
    pub pesel: Option<String>,
    pub data_urodzenia: Option<String>,
    pub panstwo_urodzenia: Option<String>,
    pub dokument_tozsamosci: Option<String>,
    pub adres_zamieszkania: Option<String>,
    pub firma: Option<String>,
    pub nip: Option<String>,
    pub adres_dzialalnosci: Option<String>,
    pub nazwa: Option<String>,
    pub forma_organizacyjna: Option<String>,
    pub adres_siedziby: Option<String>,
    pub rejestr_panstwo: Option<String>,
    pub rejestr_nazwa: Option<String>,
    pub rejestr_numer: Option<String>,
    pub rejestr_data: Option<String>,
    pub struktura_wlasnosci: Option<String>,
    pub umocowanie_dokument: Option<String>,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub wpisal: String,
    pub wpisano: String,
    pub aktualne: bool,
    pub poprzedni_id: Option<i64>,
}

impl IdentyfikacjeKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(IdentyfikacjeKlientaRow {
            id: row.get("id")?,
            klient_id: row.get("klient_id")?,
            kogo: row.get("kogo")?,
            rodzaj_podmiotu: row.get("rodzaj_podmiotu")?,
            imiona: row.get("imiona")?,
            nazwisko: row.get("nazwisko")?,
            obywatelstwo: row.get("obywatelstwo")?,
            pesel: row.get("pesel")?,
            data_urodzenia: row.get("data_urodzenia")?,
            panstwo_urodzenia: row.get("panstwo_urodzenia")?,
            dokument_tozsamosci: row.get("dokument_tozsamosci")?,
            adres_zamieszkania: row.get("adres_zamieszkania")?,
            firma: row.get("firma")?,
            nip: row.get("nip")?,
            adres_dzialalnosci: row.get("adres_dzialalnosci")?,
            nazwa: row.get("nazwa")?,
            forma_organizacyjna: row.get("forma_organizacyjna")?,
            adres_siedziby: row.get("adres_siedziby")?,
            rejestr_panstwo: row.get("rejestr_panstwo")?,
            rejestr_nazwa: row.get("rejestr_nazwa")?,
            rejestr_numer: row.get("rejestr_numer")?,
            rejestr_data: row.get("rejestr_data")?,
            struktura_wlasnosci: row.get("struktura_wlasnosci")?,
            umocowanie_dokument: row.get("umocowanie_dokument")?,
            zrodlo: row.get("zrodlo")?,
            raport_odnosnik: row.get("raport_odnosnik")?,
            sprawdzil: row.get("sprawdzil")?,
            sprawdzono: row.get("sprawdzono")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
            aktualne: row.get("aktualne")?,
            poprzedni_id: row.get("poprzedni_id")?,
        })
    }
}

/// Dane identyfikacyjne klienta, jego beneficjentów i osób działających za niego, najnowsze pierwsze.
pub fn identyfikacje_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &IdentyfikacjeKlientaParams,
) -> Result<Vec<IdentyfikacjeKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_sprawdzenia_identyfikacje
WHERE klient_id = :klient_id
ORDER BY id DESC"#,
        parameters: Vec::from([
            ("klient_id", xmeta_room::ToSqlValue::to_sql_value(&params.klient_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(IdentyfikacjeKlientaRow::from_row).collect()
}

/// The parameters of `wpisz_identyfikacje`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszIdentyfikacjeParams {
    pub klient_id: String,
    pub kogo: IdentyfikacjeKogo,
    pub rodzaj_podmiotu: IdentyfikacjeRodzajPodmiotu,
    pub imiona: Option<String>,
    pub nazwisko: Option<String>,
    pub obywatelstwo: Option<String>,
    pub pesel: Option<String>,
    pub data_urodzenia: Option<String>,
    pub panstwo_urodzenia: Option<String>,
    pub dokument_tozsamosci: Option<String>,
    pub adres_zamieszkania: Option<String>,
    pub firma: Option<String>,
    pub nip: Option<String>,
    pub adres_dzialalnosci: Option<String>,
    pub nazwa: Option<String>,
    pub forma_organizacyjna: Option<String>,
    pub adres_siedziby: Option<String>,
    pub rejestr_panstwo: Option<String>,
    pub rejestr_nazwa: Option<String>,
    pub rejestr_numer: Option<String>,
    pub rejestr_data: Option<String>,
    pub struktura_wlasnosci: Option<String>,
    pub umocowanie_dokument: Option<String>,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub wpisal: String,
    pub wpisano: String,
    pub poprzedni_id: Option<i64>,
}

/// Dane identyfikacyjne z raportu Kontrolera; ponowna identyfikacja wskazuje poprzedni wpis.
pub fn wpisz_identyfikacje(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszIdentyfikacjeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_sprawdzenia_identyfikacje (klient_id, kogo, rodzaj_podmiotu, imiona,
  nazwisko, obywatelstwo, pesel, data_urodzenia, panstwo_urodzenia, dokument_tozsamosci,
  adres_zamieszkania, firma, nip, adres_dzialalnosci, nazwa, forma_organizacyjna, adres_siedziby,
  rejestr_panstwo, rejestr_nazwa, rejestr_numer, rejestr_data, struktura_wlasnosci,
  umocowanie_dokument, zrodlo, raport_odnosnik, sprawdzil, sprawdzono, wpisal, wpisano, aktualne,
  poprzedni_id)
VALUES (:klient_id, :kogo, :rodzaj_podmiotu, :imiona, :nazwisko, :obywatelstwo, :pesel,
  :data_urodzenia, :panstwo_urodzenia, :dokument_tozsamosci, :adres_zamieszkania, :firma, :nip,
  :adres_dzialalnosci, :nazwa, :forma_organizacyjna, :adres_siedziby, :rejestr_panstwo,
  :rejestr_nazwa, :rejestr_numer, :rejestr_data, :struktura_wlasnosci, :umocowanie_dokument,
  :zrodlo, :raport_odnosnik, :sprawdzil, :sprawdzono, :wpisal, :wpisano, 1, :poprzedni_id)"#,
                parameters: Vec::from([
                    (
                        "klient_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_id),
                    ),
                    ("kogo", xmeta_room::ToSqlValue::to_sql_value(&params.kogo)),
                    (
                        "rodzaj_podmiotu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj_podmiotu),
                    ),
                    ("imiona", xmeta_room::ToSqlValue::to_sql_value(&params.imiona)),
                    ("nazwisko", xmeta_room::ToSqlValue::to_sql_value(&params.nazwisko)),
                    (
                        "obywatelstwo",
                        xmeta_room::ToSqlValue::to_sql_value(&params.obywatelstwo),
                    ),
                    ("pesel", xmeta_room::ToSqlValue::to_sql_value(&params.pesel)),
                    (
                        "data_urodzenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.data_urodzenia),
                    ),
                    (
                        "panstwo_urodzenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.panstwo_urodzenia),
                    ),
                    (
                        "dokument_tozsamosci",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_tozsamosci),
                    ),
                    (
                        "adres_zamieszkania",
                        xmeta_room::ToSqlValue::to_sql_value(&params.adres_zamieszkania),
                    ),
                    ("firma", xmeta_room::ToSqlValue::to_sql_value(&params.firma)),
                    ("nip", xmeta_room::ToSqlValue::to_sql_value(&params.nip)),
                    (
                        "adres_dzialalnosci",
                        xmeta_room::ToSqlValue::to_sql_value(&params.adres_dzialalnosci),
                    ),
                    ("nazwa", xmeta_room::ToSqlValue::to_sql_value(&params.nazwa)),
                    (
                        "forma_organizacyjna",
                        xmeta_room::ToSqlValue::to_sql_value(&params.forma_organizacyjna),
                    ),
                    (
                        "adres_siedziby",
                        xmeta_room::ToSqlValue::to_sql_value(&params.adres_siedziby),
                    ),
                    (
                        "rejestr_panstwo",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rejestr_panstwo),
                    ),
                    (
                        "rejestr_nazwa",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rejestr_nazwa),
                    ),
                    (
                        "rejestr_numer",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rejestr_numer),
                    ),
                    (
                        "rejestr_data",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rejestr_data),
                    ),
                    (
                        "struktura_wlasnosci",
                        xmeta_room::ToSqlValue::to_sql_value(&params.struktura_wlasnosci),
                    ),
                    (
                        "umocowanie_dokument",
                        xmeta_room::ToSqlValue::to_sql_value(&params.umocowanie_dokument),
                    ),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                    (
                        "raport_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.raport_odnosnik),
                    ),
                    (
                        "sprawdzil",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzil),
                    ),
                    (
                        "sprawdzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzono),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                    (
                        "poprzedni_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprzedni_id),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `informacje_aml_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct InformacjeAmlKlientaParams {
    pub klient_id: String,
}

/// A row of `informacje_aml_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct InformacjeAmlKlientaRow {
    pub id: i64,
    pub klient_id: String,
    pub poinformowano: String,
    pub sposob: InformacjeSposob,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub wpisal: String,
    pub wpisano: String,
}

impl InformacjeAmlKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(InformacjeAmlKlientaRow {
            id: row.get("id")?,
            klient_id: row.get("klient_id")?,
            poinformowano: row.get("poinformowano")?,
            sposob: row.get("sposob")?,
            zrodlo: row.get("zrodlo")?,
            raport_odnosnik: row.get("raport_odnosnik")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Informacje dla klienta o przetwarzaniu danych i obowiązkach biura.
pub fn informacje_aml_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &InformacjeAmlKlientaParams,
) -> Result<Vec<InformacjeAmlKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_sprawdzenia_informacje
WHERE klient_id = :klient_id
ORDER BY id DESC"#,
        parameters: Vec::from([
            ("klient_id", xmeta_room::ToSqlValue::to_sql_value(&params.klient_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(InformacjeAmlKlientaRow::from_row).collect()
}

/// The parameters of `wpisz_informacje_aml`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszInformacjeAmlParams {
    pub klient_id: String,
    pub poinformowano: String,
    pub sposob: InformacjeSposob,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub wpisal: String,
    pub wpisano: String,
}

/// Kiedy i jak biuro poinformowało klienta przed nawiązaniem stosunków (art. 34 ust. 5 AML).
pub fn wpisz_informacje_aml(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszInformacjeAmlParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_sprawdzenia_informacje (klient_id, poinformowano, sposob, zrodlo,
  raport_odnosnik, wpisal, wpisano)
VALUES (:klient_id, :poinformowano, :sposob, :zrodlo, :raport_odnosnik, :wpisal, :wpisano)"#,
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
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                    (
                        "raport_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.raport_odnosnik),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
        ],
    )
}

/// The parameters of `kopie_identyfikacji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct KopieIdentyfikacjiParams {
    pub identyfikacja_id: i64,
}

/// A row of `kopie_identyfikacji`.
#[derive(Debug, Clone, PartialEq)]
pub struct KopieIdentyfikacjiRow {
    pub id: i64,
    pub identyfikacja_id: i64,
    pub plik_sciezka: String,
    pub plik_skrot: String,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub wpisal: String,
    pub wpisano: String,
}

impl KopieIdentyfikacjiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(KopieIdentyfikacjiRow {
            id: row.get("id")?,
            identyfikacja_id: row.get("identyfikacja_id")?,
            plik_sciezka: row.get("plik_sciezka")?,
            plik_skrot: row.get("plik_skrot")?,
            zrodlo: row.get("zrodlo")?,
            raport_odnosnik: row.get("raport_odnosnik")?,
            sprawdzil: row.get("sprawdzil")?,
            sprawdzono: row.get("sprawdzono")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Kopie dokumentów tożsamości jednej identyfikacji.
pub fn kopie_identyfikacji(
    room: &impl xmeta_room::MetaRoom,
    params: &KopieIdentyfikacjiParams,
) -> Result<Vec<KopieIdentyfikacjiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_sprawdzenia_kopie
WHERE identyfikacja_id = :identyfikacja_id
ORDER BY id DESC"#,
        parameters: Vec::from([
            (
                "identyfikacja_id",
                xmeta_room::ToSqlValue::to_sql_value(&params.identyfikacja_id),
            ),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(KopieIdentyfikacjiRow::from_row).collect()
}

/// The parameters of `wpisz_kopie_dokumentu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszKopieDokumentuParams {
    pub identyfikacja_id: i64,
    pub plik_sciezka: String,
    pub plik_skrot: String,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub wpisal: String,
    pub wpisano: String,
}

/// Kopia dokumentu tożsamości ze skrótem, dołączona przez Kontrolera do raportu.
pub fn wpisz_kopie_dokumentu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszKopieDokumentuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_sprawdzenia_kopie (identyfikacja_id, plik_sciezka, plik_skrot, zrodlo,
  raport_odnosnik, sprawdzil, sprawdzono, wpisal, wpisano)
VALUES (:identyfikacja_id, :plik_sciezka, :plik_skrot, :zrodlo, :raport_odnosnik, :sprawdzil,
  :sprawdzono, :wpisal, :wpisano)"#,
                parameters: Vec::from([
                    (
                        "identyfikacja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.identyfikacja_id),
                    ),
                    (
                        "plik_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.plik_sciezka),
                    ),
                    (
                        "plik_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.plik_skrot),
                    ),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                    (
                        "raport_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.raport_odnosnik),
                    ),
                    (
                        "sprawdzil",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzil),
                    ),
                    (
                        "sprawdzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzono),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
        ],
    )
}

/// The parameters of `analizy_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AnalizyKlientaParams {
    pub klient_id: String,
}

/// A row of `analizy_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalizyKlientaRow {
    pub id: i64,
    pub klient_id: String,
    pub teczka_odnosnik: String,
    pub analiza: String,
    pub wynik: MonitorowanieWynik,
    pub opis: Option<String>,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub wpisal: String,
    pub wpisano: String,
}

impl AnalizyKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(AnalizyKlientaRow {
            id: row.get("id")?,
            klient_id: row.get("klient_id")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            analiza: row.get("analiza")?,
            wynik: row.get("wynik")?,
            opis: row.get("opis")?,
            zrodlo: row.get("zrodlo")?,
            raport_odnosnik: row.get("raport_odnosnik")?,
            sprawdzil: row.get("sprawdzil")?,
            sprawdzono: row.get("sprawdzono")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Wyniki bieżącej analizy transakcji klienta, najnowsze pierwsze.
pub fn analizy_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &AnalizyKlientaParams,
) -> Result<Vec<AnalizyKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_sprawdzenia_monitorowanie
WHERE klient_id = :klient_id
ORDER BY id DESC"#,
        parameters: Vec::from([
            ("klient_id", xmeta_room::ToSqlValue::to_sql_value(&params.klient_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(AnalizyKlientaRow::from_row).collect()
}

/// The parameters of `wpisz_analize`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszAnalizeParams {
    pub klient_id: String,
    pub teczka_odnosnik: String,
    pub analiza: String,
    pub wynik: MonitorowanieWynik,
    pub opis: Option<String>,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub wpisal: String,
    pub wpisano: String,
}

/// Wynik bieżącej analizy transakcji klienta w jednej teczce.
pub fn wpisz_analize(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszAnalizeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_sprawdzenia_monitorowanie (klient_id, teczka_odnosnik, analiza, wynik,
  opis, zrodlo, raport_odnosnik, sprawdzil, sprawdzono, wpisal, wpisano)
VALUES (:klient_id, :teczka_odnosnik, :analiza, :wynik, :opis, :zrodlo, :raport_odnosnik,
  :sprawdzil, :sprawdzono, :wpisal, :wpisano)"#,
                parameters: Vec::from([
                    (
                        "klient_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_id),
                    ),
                    (
                        "teczka_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.teczka_odnosnik),
                    ),
                    ("analiza", xmeta_room::ToSqlValue::to_sql_value(&params.analiza)),
                    ("wynik", xmeta_room::ToSqlValue::to_sql_value(&params.wynik)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                    (
                        "raport_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.raport_odnosnik),
                    ),
                    (
                        "sprawdzil",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzil),
                    ),
                    (
                        "sprawdzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzono),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
        ],
    )
}

/// The parameters of `sprawdzenia_pep_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SprawdzeniaPepKlientaParams {
    pub klient_id: String,
}

/// A row of `sprawdzenia_pep_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct SprawdzeniaPepKlientaRow {
    pub id: i64,
    pub klient_id: String,
    pub dotyczy: PepDotyczy,
    pub osoba: String,
    pub oswiadczenie_sciezka: Option<String>,
    pub oswiadczenie_skrot: Option<String>,
    pub oswiadczenie_z_klauzula: bool,
    pub wykaz_sprawdzono: String,
    pub wynik: PepWynik,
    pub stanowisko_do: Option<String>,
    pub zrodlo_majatku: Option<String>,
    pub akceptacja_decyzja_id: Option<String>,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub wpisal: String,
    pub wpisano: String,
    pub aktualne: bool,
    pub poprzedni_id: Option<i64>,
}

impl SprawdzeniaPepKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(SprawdzeniaPepKlientaRow {
            id: row.get("id")?,
            klient_id: row.get("klient_id")?,
            dotyczy: row.get("dotyczy")?,
            osoba: row.get("osoba")?,
            oswiadczenie_sciezka: row.get("oswiadczenie_sciezka")?,
            oswiadczenie_skrot: row.get("oswiadczenie_skrot")?,
            oswiadczenie_z_klauzula: row.get("oswiadczenie_z_klauzula")?,
            wykaz_sprawdzono: row.get("wykaz_sprawdzono")?,
            wynik: row.get("wynik")?,
            stanowisko_do: row.get("stanowisko_do")?,
            zrodlo_majatku: row.get("zrodlo_majatku")?,
            akceptacja_decyzja_id: row.get("akceptacja_decyzja_id")?,
            zrodlo: row.get("zrodlo")?,
            raport_odnosnik: row.get("raport_odnosnik")?,
            sprawdzil: row.get("sprawdzil")?,
            sprawdzono: row.get("sprawdzono")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
            aktualne: row.get("aktualne")?,
            poprzedni_id: row.get("poprzedni_id")?,
        })
    }
}

/// Sprawdzenia eksponowanego stanowiska politycznego klienta i jego beneficjentów.
pub fn sprawdzenia_pep_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &SprawdzeniaPepKlientaParams,
) -> Result<Vec<SprawdzeniaPepKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_sprawdzenia_pep
WHERE klient_id = :klient_id
ORDER BY id DESC"#,
        parameters: Vec::from([
            ("klient_id", xmeta_room::ToSqlValue::to_sql_value(&params.klient_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(SprawdzeniaPepKlientaRow::from_row).collect()
}

/// The parameters of `wpisz_sprawdzenie_pep`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszSprawdzeniePepParams {
    pub klient_id: String,
    pub dotyczy: PepDotyczy,
    pub osoba: String,
    pub oswiadczenie_sciezka: Option<String>,
    pub oswiadczenie_skrot: Option<String>,
    pub oswiadczenie_z_klauzula: bool,
    pub wykaz_sprawdzono: String,
    pub wynik: PepWynik,
    pub stanowisko_do: Option<String>,
    pub zrodlo_majatku: Option<String>,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub wpisal: String,
    pub wpisano: String,
    pub poprzedni_id: Option<i64>,
}

/// Oświadczenie i wynik sprawdzenia wykazu stanowisk; akceptację wpisuje wpisz_akceptacje_pep.
pub fn wpisz_sprawdzenie_pep(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszSprawdzeniePepParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_sprawdzenia_pep (klient_id, dotyczy, osoba, oswiadczenie_sciezka,
  oswiadczenie_skrot, oswiadczenie_z_klauzula, wykaz_sprawdzono, wynik, stanowisko_do,
  zrodlo_majatku, zrodlo, raport_odnosnik, sprawdzil, sprawdzono, wpisal, wpisano, aktualne,
  poprzedni_id)
VALUES (:klient_id, :dotyczy, :osoba, :oswiadczenie_sciezka, :oswiadczenie_skrot,
  :oswiadczenie_z_klauzula, :wykaz_sprawdzono, :wynik, :stanowisko_do, :zrodlo_majatku, :zrodlo,
  :raport_odnosnik, :sprawdzil, :sprawdzono, :wpisal, :wpisano, 1, :poprzedni_id)"#,
                parameters: Vec::from([
                    (
                        "klient_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_id),
                    ),
                    ("dotyczy", xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy)),
                    ("osoba", xmeta_room::ToSqlValue::to_sql_value(&params.osoba)),
                    (
                        "oswiadczenie_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.oswiadczenie_sciezka,
                        ),
                    ),
                    (
                        "oswiadczenie_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.oswiadczenie_skrot),
                    ),
                    (
                        "oswiadczenie_z_klauzula",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.oswiadczenie_z_klauzula,
                        ),
                    ),
                    (
                        "wykaz_sprawdzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wykaz_sprawdzono),
                    ),
                    ("wynik", xmeta_room::ToSqlValue::to_sql_value(&params.wynik)),
                    (
                        "stanowisko_do",
                        xmeta_room::ToSqlValue::to_sql_value(&params.stanowisko_do),
                    ),
                    (
                        "zrodlo_majatku",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_majatku),
                    ),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                    (
                        "raport_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.raport_odnosnik),
                    ),
                    (
                        "sprawdzil",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzil),
                    ),
                    (
                        "sprawdzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzono),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                    (
                        "poprzedni_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprzedni_id),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `wpisz_akceptacje_pep`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszAkceptacjePepParams {
    pub akceptacja_decyzja_id: String,
    pub id: i64,
}

/// Akceptacja właściciela biura na stosunki z osobą zajmującą eksponowane stanowisko polityczne;
/// wpisuje się raz.
pub fn wpisz_akceptacje_pep(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszAkceptacjePepParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE rejestr_kontroli_sprawdzenia_pep
SET akceptacja_decyzja_id = :akceptacja_decyzja_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "akceptacja_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.akceptacja_decyzja_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `sprawdzenia_poza_ustawa_teczki`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SprawdzeniaPozaUstawaTeczkiParams {
    pub teczka_odnosnik: String,
}

/// A row of `sprawdzenia_poza_ustawa_teczki`.
#[derive(Debug, Clone, PartialEq)]
pub struct SprawdzeniaPozaUstawaTeczkiRow {
    pub id: i64,
    pub klient_id: Option<String>,
    pub rodzaj: PozaUstawaRodzaj,
    pub teczka_odnosnik: String,
    pub wiersz_odnosnik: String,
    pub wersja_wiersza: Option<i64>,
    pub sprawdzenie_odnosnik: String,
    pub wyplata_odnosnik: Option<String>,
    pub zrodlo_data: Option<String>,
    pub wynik: PozaUstawaWynik,
    pub opis: Option<String>,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub wpisal: String,
    pub wpisano: String,
    pub aktualne: bool,
    pub poprzedni_id: Option<i64>,
}

impl SprawdzeniaPozaUstawaTeczkiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(SprawdzeniaPozaUstawaTeczkiRow {
            id: row.get("id")?,
            klient_id: row.get("klient_id")?,
            rodzaj: row.get("rodzaj")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            wiersz_odnosnik: row.get("wiersz_odnosnik")?,
            wersja_wiersza: row.get("wersja_wiersza")?,
            sprawdzenie_odnosnik: row.get("sprawdzenie_odnosnik")?,
            wyplata_odnosnik: row.get("wyplata_odnosnik")?,
            zrodlo_data: row.get("zrodlo_data")?,
            wynik: row.get("wynik")?,
            opis: row.get("opis")?,
            zrodlo: row.get("zrodlo")?,
            raport_odnosnik: row.get("raport_odnosnik")?,
            sprawdzil: row.get("sprawdzil")?,
            sprawdzono: row.get("sprawdzono")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
            aktualne: row.get("aktualne")?,
            poprzedni_id: row.get("poprzedni_id")?,
        })
    }
}

/// Sprawdzenia spoza ustawy w jednej teczce, najnowsze pierwsze.
pub fn sprawdzenia_poza_ustawa_teczki(
    room: &impl xmeta_room::MetaRoom,
    params: &SprawdzeniaPozaUstawaTeczkiParams,
) -> Result<Vec<SprawdzeniaPozaUstawaTeczkiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_sprawdzenia_poza_ustawa
WHERE teczka_odnosnik = :teczka_odnosnik
ORDER BY id DESC"#,
        parameters: Vec::from([
            (
                "teczka_odnosnik",
                xmeta_room::ToSqlValue::to_sql_value(&params.teczka_odnosnik),
            ),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(SprawdzeniaPozaUstawaTeczkiRow::from_row).collect()
}

/// The parameters of `wpisz_sprawdzenie_poza_ustawa`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszSprawdzeniePozaUstawaParams {
    pub klient_id: Option<String>,
    pub rodzaj: PozaUstawaRodzaj,
    pub teczka_odnosnik: String,
    pub wiersz_odnosnik: String,
    pub wersja_wiersza: Option<i64>,
    pub sprawdzenie_odnosnik: String,
    pub wyplata_odnosnik: Option<String>,
    pub zrodlo_data: Option<String>,
    pub wynik: PozaUstawaWynik,
    pub opis: Option<String>,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub wpisal: String,
    pub wpisano: String,
    pub poprzedni_id: Option<i64>,
}

/// Sprawdzenie tożsamości, tytułu, pełnomocnictwa albo rachunku z teczki; przy rachunku przed
/// wypłatą także wypłata. Ponowne sprawdzenie tego samego wiersza wskazuje poprzedni wpis.
pub fn wpisz_sprawdzenie_poza_ustawa(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszSprawdzeniePozaUstawaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_sprawdzenia_poza_ustawa (klient_id, rodzaj, teczka_odnosnik,
  wiersz_odnosnik, wersja_wiersza, sprawdzenie_odnosnik, wyplata_odnosnik, zrodlo_data, wynik, opis,
  zrodlo, raport_odnosnik, sprawdzil, sprawdzono, wpisal, wpisano, aktualne, poprzedni_id)
VALUES (:klient_id, :rodzaj, :teczka_odnosnik, :wiersz_odnosnik, :wersja_wiersza,
  :sprawdzenie_odnosnik, :wyplata_odnosnik, :zrodlo_data, :wynik, :opis, :zrodlo, :raport_odnosnik,
  :sprawdzil, :sprawdzono, :wpisal, :wpisano, 1, :poprzedni_id)"#,
                parameters: Vec::from([
                    (
                        "klient_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_id),
                    ),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    (
                        "teczka_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.teczka_odnosnik),
                    ),
                    (
                        "wiersz_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wiersz_odnosnik),
                    ),
                    (
                        "wersja_wiersza",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wersja_wiersza),
                    ),
                    (
                        "sprawdzenie_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.sprawdzenie_odnosnik,
                        ),
                    ),
                    (
                        "wyplata_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wyplata_odnosnik),
                    ),
                    (
                        "zrodlo_data",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_data),
                    ),
                    ("wynik", xmeta_room::ToSqlValue::to_sql_value(&params.wynik)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                    (
                        "raport_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.raport_odnosnik),
                    ),
                    (
                        "sprawdzil",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzil),
                    ),
                    (
                        "sprawdzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzono),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                    (
                        "poprzedni_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprzedni_id),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `sprawdzenia_sankcji_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SprawdzeniaSankcjiKlientaParams {
    pub klient_id: String,
}

/// A row of `sprawdzenia_sankcji_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct SprawdzeniaSankcjiKlientaRow {
    pub id: i64,
    pub klient_id: String,
    pub listy: String,
    pub wynik: SankcjeWynik,
    pub zamrozono: Option<String>,
    pub zamrozenie_informacja: Option<String>,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub wpisal: String,
    pub wpisano: String,
    pub aktualne: bool,
    pub poprzedni_id: Option<i64>,
}

impl SprawdzeniaSankcjiKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(SprawdzeniaSankcjiKlientaRow {
            id: row.get("id")?,
            klient_id: row.get("klient_id")?,
            listy: row.get("listy")?,
            wynik: row.get("wynik")?,
            zamrozono: row.get("zamrozono")?,
            zamrozenie_informacja: row.get("zamrozenie_informacja")?,
            zrodlo: row.get("zrodlo")?,
            raport_odnosnik: row.get("raport_odnosnik")?,
            sprawdzil: row.get("sprawdzil")?,
            sprawdzono: row.get("sprawdzono")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
            aktualne: row.get("aktualne")?,
            poprzedni_id: row.get("poprzedni_id")?,
        })
    }
}

/// Sprawdzenia list sankcyjnych klienta, najnowsze pierwsze.
pub fn sprawdzenia_sankcji_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &SprawdzeniaSankcjiKlientaParams,
) -> Result<Vec<SprawdzeniaSankcjiKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_sprawdzenia_sankcje
WHERE klient_id = :klient_id
ORDER BY id DESC"#,
        parameters: Vec::from([
            ("klient_id", xmeta_room::ToSqlValue::to_sql_value(&params.klient_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(SprawdzeniaSankcjiKlientaRow::from_row).collect()
}

/// The parameters of `wpisz_sprawdzenie_sankcji`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszSprawdzenieSankcjiParams {
    pub klient_id: String,
    pub listy: String,
    pub wynik: SankcjeWynik,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub wpisal: String,
    pub wpisano: String,
    pub poprzedni_id: Option<i64>,
}

/// Sprawdzenie list sankcyjnych z wynikiem; zamrożenie wpisuje wpisz_zamrozenie.
pub fn wpisz_sprawdzenie_sankcji(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszSprawdzenieSankcjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_sprawdzenia_sankcje (klient_id, listy, wynik, zrodlo, raport_odnosnik,
  sprawdzil, sprawdzono, wpisal, wpisano, aktualne, poprzedni_id)
VALUES (:klient_id, :listy, :wynik, :zrodlo, :raport_odnosnik, :sprawdzil, :sprawdzono, :wpisal,
  :wpisano, 1, :poprzedni_id)"#,
                parameters: Vec::from([
                    (
                        "klient_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_id),
                    ),
                    ("listy", xmeta_room::ToSqlValue::to_sql_value(&params.listy)),
                    ("wynik", xmeta_room::ToSqlValue::to_sql_value(&params.wynik)),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                    (
                        "raport_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.raport_odnosnik),
                    ),
                    (
                        "sprawdzil",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzil),
                    ),
                    (
                        "sprawdzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzono),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                    (
                        "poprzedni_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprzedni_id),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `wpisz_zamrozenie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszZamrozenieParams {
    pub zamrozono: String,
    pub id: i64,
}

/// Chwila zamrożenia wartości majątkowych klienta po trafieniu na liście; wpisuje się raz.
pub fn wpisz_zamrozenie(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszZamrozenieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE rejestr_kontroli_sprawdzenia_sankcje
SET zamrozono = :zamrozono
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zamrozono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zamrozono),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wpisz_informacje_o_zamrozeniu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszInformacjeOZamrozeniuParams {
    pub zamrozenie_informacja: String,
    pub id: i64,
}

/// Chwila przekazania informacji o zamrożeniu Generalnemu Inspektorowi; wpisuje się raz.
pub fn wpisz_informacje_o_zamrozeniu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszInformacjeOZamrozeniuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE rejestr_kontroli_sprawdzenia_sankcje
SET zamrozenie_informacja = :zamrozenie_informacja
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zamrozenie_informacja",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.zamrozenie_informacja,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `weryfikacje_identyfikacji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WeryfikacjeIdentyfikacjiParams {
    pub identyfikacja_id: i64,
}

/// A row of `weryfikacje_identyfikacji`.
#[derive(Debug, Clone, PartialEq)]
pub struct WeryfikacjeIdentyfikacjiRow {
    pub id: i64,
    pub identyfikacja_id: i64,
    pub podstawa: WeryfikacjePodstawa,
    pub wynik: PozaUstawaWynik,
    pub opis: Option<String>,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub wpisal: String,
    pub wpisano: String,
    pub aktualne: bool,
    pub poprzedni_id: Option<i64>,
}

impl WeryfikacjeIdentyfikacjiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WeryfikacjeIdentyfikacjiRow {
            id: row.get("id")?,
            identyfikacja_id: row.get("identyfikacja_id")?,
            podstawa: row.get("podstawa")?,
            wynik: row.get("wynik")?,
            opis: row.get("opis")?,
            zrodlo: row.get("zrodlo")?,
            raport_odnosnik: row.get("raport_odnosnik")?,
            sprawdzil: row.get("sprawdzil")?,
            sprawdzono: row.get("sprawdzono")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
            aktualne: row.get("aktualne")?,
            poprzedni_id: row.get("poprzedni_id")?,
        })
    }
}

/// Weryfikacje jednej identyfikacji, najnowsze pierwsze.
pub fn weryfikacje_identyfikacji(
    room: &impl xmeta_room::MetaRoom,
    params: &WeryfikacjeIdentyfikacjiParams,
) -> Result<Vec<WeryfikacjeIdentyfikacjiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_sprawdzenia_weryfikacje
WHERE identyfikacja_id = :identyfikacja_id
ORDER BY id DESC"#,
        parameters: Vec::from([
            (
                "identyfikacja_id",
                xmeta_room::ToSqlValue::to_sql_value(&params.identyfikacja_id),
            ),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(WeryfikacjeIdentyfikacjiRow::from_row).collect()
}

/// The parameters of `wpisz_weryfikacje`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszWeryfikacjeParams {
    pub identyfikacja_id: i64,
    pub podstawa: WeryfikacjePodstawa,
    pub wynik: PozaUstawaWynik,
    pub opis: Option<String>,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub wpisal: String,
    pub wpisano: String,
    pub poprzedni_id: Option<i64>,
}

/// Weryfikacja danych identyfikacyjnych z podstawą i wynikiem.
pub fn wpisz_weryfikacje(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszWeryfikacjeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_sprawdzenia_weryfikacje (identyfikacja_id, podstawa, wynik, opis,
  zrodlo, raport_odnosnik, sprawdzil, sprawdzono, wpisal, wpisano, aktualne, poprzedni_id)
VALUES (:identyfikacja_id, :podstawa, :wynik, :opis, :zrodlo, :raport_odnosnik, :sprawdzil,
  :sprawdzono, :wpisal, :wpisano, 1, :poprzedni_id)"#,
                parameters: Vec::from([
                    (
                        "identyfikacja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.identyfikacja_id),
                    ),
                    ("podstawa", xmeta_room::ToSqlValue::to_sql_value(&params.podstawa)),
                    ("wynik", xmeta_room::ToSqlValue::to_sql_value(&params.wynik)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                    (
                        "raport_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.raport_odnosnik),
                    ),
                    (
                        "sprawdzil",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzil),
                    ),
                    (
                        "sprawdzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzono),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                    (
                        "poprzedni_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprzedni_id),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zrodla_srodkow_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZrodlaSrodkowKlientaParams {
    pub klient_id: String,
}

/// A row of `zrodla_srodkow_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZrodlaSrodkowKlientaRow {
    pub id: i64,
    pub klient_id: String,
    pub pochodzenie_srodkow: String,
    pub dokument_sciezka: String,
    pub dokument_skrot: String,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub wpisal: String,
    pub wpisano: String,
    pub aktualne: bool,
    pub poprzedni_id: Option<i64>,
}

impl ZrodlaSrodkowKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZrodlaSrodkowKlientaRow {
            id: row.get("id")?,
            klient_id: row.get("klient_id")?,
            pochodzenie_srodkow: row.get("pochodzenie_srodkow")?,
            dokument_sciezka: row.get("dokument_sciezka")?,
            dokument_skrot: row.get("dokument_skrot")?,
            zrodlo: row.get("zrodlo")?,
            raport_odnosnik: row.get("raport_odnosnik")?,
            sprawdzil: row.get("sprawdzil")?,
            sprawdzono: row.get("sprawdzono")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
            aktualne: row.get("aktualne")?,
            poprzedni_id: row.get("poprzedni_id")?,
        })
    }
}

/// Pochodzenie pieniędzy klienta z dokumentami, najnowsze pierwsze.
pub fn zrodla_srodkow_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &ZrodlaSrodkowKlientaParams,
) -> Result<Vec<ZrodlaSrodkowKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_sprawdzenia_zrodla_srodkow
WHERE klient_id = :klient_id
ORDER BY id DESC"#,
        parameters: Vec::from([
            ("klient_id", xmeta_room::ToSqlValue::to_sql_value(&params.klient_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZrodlaSrodkowKlientaRow::from_row).collect()
}

/// The parameters of `wpisz_zrodlo_srodkow`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszZrodloSrodkowParams {
    pub klient_id: String,
    pub pochodzenie_srodkow: String,
    pub dokument_sciezka: String,
    pub dokument_skrot: String,
    pub zrodlo: String,
    pub raport_odnosnik: String,
    pub sprawdzil: String,
    pub sprawdzono: String,
    pub wpisal: String,
    pub wpisano: String,
    pub poprzedni_id: Option<i64>,
}

/// Pochodzenie pieniędzy klienta z dokumentem.
pub fn wpisz_zrodlo_srodkow(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszZrodloSrodkowParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_sprawdzenia_zrodla_srodkow (klient_id, pochodzenie_srodkow,
  dokument_sciezka, dokument_skrot, zrodlo, raport_odnosnik, sprawdzil, sprawdzono, wpisal, wpisano,
  aktualne, poprzedni_id)
VALUES (:klient_id, :pochodzenie_srodkow, :dokument_sciezka, :dokument_skrot, :zrodlo,
  :raport_odnosnik, :sprawdzil, :sprawdzono, :wpisal, :wpisano, 1, :poprzedni_id)"#,
                parameters: Vec::from([
                    (
                        "klient_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_id),
                    ),
                    (
                        "pochodzenie_srodkow",
                        xmeta_room::ToSqlValue::to_sql_value(&params.pochodzenie_srodkow),
                    ),
                    (
                        "dokument_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_sciezka),
                    ),
                    (
                        "dokument_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_skrot),
                    ),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                    (
                        "raport_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.raport_odnosnik),
                    ),
                    (
                        "sprawdzil",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzil),
                    ),
                    (
                        "sprawdzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzono),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                    (
                        "poprzedni_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprzedni_id),
                    ),
                ]),
            },
        ],
    )
}
