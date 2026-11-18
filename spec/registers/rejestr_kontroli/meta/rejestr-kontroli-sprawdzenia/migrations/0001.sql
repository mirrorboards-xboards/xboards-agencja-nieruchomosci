-- Środki bezpieczeństwa finansowego i sprawdzenia spoza ustawy, przeniesione przez Kierownika
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
