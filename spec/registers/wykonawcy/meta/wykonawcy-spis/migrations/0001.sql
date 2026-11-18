-- Wykonawcy, którym biuro zleca pracę w lokalach krótkoterminowych: sprzątanie, gospodarze na
-- miejscu oraz naprawy i serwis. Rejestr trzyma ich dane do umowy, osoby, kanały zleceń, obszary,
-- stan i podpisane decyzje człowieka. Pokój jest jeden na biuro.
--
-- Pisze Kierownik najmu, a czytają Koordynatorzy operacji. Kierownik wpisuje każdą zmianę dopiero
-- po decyzji człowieka podpisanej jego kluczem, więc każdy wiersz wskazuje decyzję właściwego
-- rodzaju. Podpis obejmuje skrót wpisu (dotyczy_skrot); baza wymaga kompletu kolumn podpisu,
-- a sam podpis sprawdza aplikacja, zanim Koordynator zastosuje nowego wykonawcę albo kontakt.
--
-- Wpisów się nie poprawia: dane wykonawcy mają wersje, a osoby, kanały i obszary daty od i do,
-- z decyzją zakończenia. Wykreślony wykonawca zostaje w rejestrze, bo wskazują go zlecenia
-- w teczkach lokali (tekstem, bez klucza obcego). Rejestr nie ma rachunków bankowych wykonawców
-- ani danych gości; to wynika z budowy tabel.
CREATE TABLE wykonawcy_spis_decyzje (
  id TEXT PRIMARY KEY,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('przyjecie', 'zmiana_danych', 'cennik', 'przypisanie', 'wstrzymanie', 'przywrocenie', 'wykreslenie')),
  -- Człowiek, który zdecydował, np. właściciel Kierownika najmu.
  decyduje TEXT NOT NULL CHECK (decyduje <> ''),
  -- Co zatwierdził, krótko, np. „przyjęcie Czysto sp. c. do obszaru Stare Miasto”.
  tresc TEXT NOT NULL CHECK (tresc <> ''),
  -- Skrót wpisu, a przy cenniku skrót całej wersji z pozycjami, który liczy aplikacja.
  dotyczy_skrot TEXT NOT NULL CHECK (dotyczy_skrot <> ''),
  podpis TEXT NOT NULL CHECK (podpis <> ''),
  podpisano TEXT NOT NULL,
  -- Kierownik najmu, który przepisał decyzję do rejestru, i kiedy.
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,

  CONSTRAINT wpis_nie_przed_podpisem CHECK (wpisano >= podpisano)
) STRICT;

CREATE TABLE wykonawcy_spis_wpisy (
  id TEXT PRIMARY KEY,
  -- Np. „Czysto sp. c.” albo „Jan Kowalski, hydraulik”.
  nazwa TEXT NOT NULL CHECK (nazwa <> ''),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('sprzatanie', 'gospodarz', 'naprawy_i_serwis')),
  -- Specjalność wykonawcy napraw, np. hydraulik, elektryk, serwis klimatyzacji.
  specjalnosc TEXT,
  decyzja_id TEXT NOT NULL REFERENCES wykonawcy_spis_decyzje (id),
  wpisano TEXT NOT NULL,

  CONSTRAINT specjalnosc_przy_naprawach CHECK ((rodzaj = 'naprawy_i_serwis') = (specjalnosc IS NOT NULL))
) STRICT;

-- Dane do umowy, zakres usług, umowa z biurem i polisa wykonawcy w wersjach. Nowa wersja, np. po
-- przedłużeniu umowy albo nowej polisie, odbiera aktualność poprzedniej.
CREATE TABLE wykonawcy_spis_dane (
  -- Klucz nadany przez bazę jest dodatni; zob. wykonawcy_spis_dane_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  wykonawca_id TEXT NOT NULL REFERENCES wykonawcy_spis_wpisy (id),
  aktualne INTEGER NOT NULL CHECK (aktualne IN (0, 1)),
  firma TEXT NOT NULL CHECK (firma <> ''),
  nip TEXT,
  krs TEXT,
  ceidg INTEGER NOT NULL DEFAULT 0 CHECK (ceidg IN (0, 1)),
  adres TEXT NOT NULL CHECK (adres <> ''),
  -- Co wykonawca robi, a czego nie, np. „sprzątanie i pranie; bez mycia okien na wysokości”.
  zakres_uslug TEXT NOT NULL CHECK (zakres_uslug <> ''),
  -- Wykonawca przywozi własne środki czystości.
  srodki_wlasne INTEGER NOT NULL CHECK (srodki_wlasne IN (0, 1)),
  umowa_od TEXT NOT NULL,
  umowa_do TEXT,
  -- Człowiek, który podpisał umowę za biuro.
  umowa_podpisal TEXT NOT NULL CHECK (umowa_podpisal <> ''),
  umowa_sciezka TEXT NOT NULL,
  umowa_skrot TEXT NOT NULL,
  -- Polisa OC wykonawcy, gdy biuro jej wymaga.
  polisa_do TEXT,
  polisa_sciezka TEXT,
  polisa_skrot TEXT,
  decyzja_id TEXT NOT NULL REFERENCES wykonawcy_spis_decyzje (id),
  wpisano TEXT NOT NULL,

  CONSTRAINT krs_albo_ceidg CHECK (krs IS NULL OR ceidg = 0),
  CONSTRAINT koniec_umowy_nie_przed_poczatkiem CHECK (umowa_do IS NULL OR umowa_do >= umowa_od),
  CONSTRAINT polisa_w_komplecie CHECK (
    (polisa_do IS NULL) = (polisa_sciezka IS NULL)
    AND (polisa_do IS NULL) = (polisa_skrot IS NULL)
  )
) STRICT;

-- Pracownicy wykonawcy, którzy przyjmują zlecenia i przychodzą do lokali. Koordynator wysyła kod
-- do zamka tylko osobie z prawem do kodów, której zlecił pracę w tym lokalu i w tym czasie.
CREATE TABLE wykonawcy_spis_osoby (
  -- Klucz nadany przez bazę jest dodatni; zob. wykonawcy_spis_osoby_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  wykonawca_id TEXT NOT NULL REFERENCES wykonawcy_spis_wpisy (id),
  imiona TEXT NOT NULL CHECK (imiona <> ''),
  nazwisko TEXT NOT NULL CHECK (nazwisko <> ''),
  telefon TEXT NOT NULL CHECK (telefon <> ''),
  kody_do_zamkow INTEGER NOT NULL CHECK (kody_do_zamkow IN (0, 1)),
  od TEXT NOT NULL,
  do TEXT,
  decyzja_id TEXT NOT NULL REFERENCES wykonawcy_spis_decyzje (id),
  zakonczenie_decyzja_id TEXT REFERENCES wykonawcy_spis_decyzje (id),

  CONSTRAINT koniec_z_decyzja CHECK ((do IS NULL) = (zakonczenie_decyzja_id IS NULL)),
  CONSTRAINT koniec_nie_przed_poczatkiem CHECK (do IS NULL OR do >= od)
) STRICT;

-- Kanały zleceń z numerami i adresami, z których wykonawca pisze. Koordynator porównuje z nimi
-- każdą wiadomość „od wykonawcy”.
CREATE TABLE wykonawcy_spis_kanaly (
  -- Klucz nadany przez bazę jest dodatni; zob. wykonawcy_spis_kanaly_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  wykonawca_id TEXT NOT NULL REFERENCES wykonawcy_spis_wpisy (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('aplikacja', 'sms', 'email')),
  -- Numer telefonu, adres poczty albo konto w aplikacji biura.
  adres TEXT NOT NULL CHECK (adres <> ''),
  od TEXT NOT NULL,
  do TEXT,
  decyzja_id TEXT NOT NULL REFERENCES wykonawcy_spis_decyzje (id),
  zakonczenie_decyzja_id TEXT REFERENCES wykonawcy_spis_decyzje (id),

  CONSTRAINT koniec_z_decyzja CHECK ((do IS NULL) = (zakonczenie_decyzja_id IS NULL)),
  CONSTRAINT koniec_nie_przed_poczatkiem CHECK (do IS NULL OR do >= od)
) STRICT;

-- Obszary biura, w których wykonawca pracuje, z dniami i godzinami pracy, a u wykonawcy napraw
-- z czasem dojazdu do pilnej usterki.
CREATE TABLE wykonawcy_spis_obszary (
  -- Klucz nadany przez bazę jest dodatni; zob. wykonawcy_spis_obszary_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  wykonawca_id TEXT NOT NULL REFERENCES wykonawcy_spis_wpisy (id),
  obszar TEXT NOT NULL CHECK (obszar <> ''),
  -- Np. „pon.–sob. 8–20”.
  dostepnosc TEXT NOT NULL CHECK (dostepnosc <> ''),
  -- Np. „do 2 godzin”.
  czas_dojazdu_do_usterki TEXT,
  od TEXT NOT NULL,
  do TEXT,
  decyzja_id TEXT NOT NULL REFERENCES wykonawcy_spis_decyzje (id),
  zakonczenie_decyzja_id TEXT REFERENCES wykonawcy_spis_decyzje (id),

  CONSTRAINT koniec_z_decyzja CHECK ((do IS NULL) = (zakonczenie_decyzja_id IS NULL)),
  CONSTRAINT koniec_nie_przed_poczatkiem CHECK (do IS NULL OR do >= od)
) STRICT;

-- Stan wykonawcy z powodem i decyzją: zatwierdzony przy przyjęciu, wstrzymany do decyzji
-- o przywróceniu, wykreślony na zawsze.
CREATE TABLE wykonawcy_spis_stany (
  -- Klucz nadany przez bazę jest dodatni; zob. wykonawcy_spis_stany_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  wykonawca_id TEXT NOT NULL REFERENCES wykonawcy_spis_wpisy (id),
  stan TEXT NOT NULL CHECK (stan IN ('zatwierdzony', 'wstrzymany', 'wykreslony')),
  od TEXT NOT NULL,
  powod TEXT,
  decyzja_id TEXT NOT NULL REFERENCES wykonawcy_spis_decyzje (id),

  CONSTRAINT wstrzymanie_i_wykreslenie_z_powodem CHECK (stan = 'zatwierdzony' OR powod IS NOT NULL)
) STRICT;

-- Decyzje właściwego rodzaju -----------------------------------------------------------------------

CREATE TRIGGER wykonawcy_spis_wpis_po_decyzji_o_przyjeciu
BEFORE INSERT ON wykonawcy_spis_wpisy
WHEN (
  SELECT rodzaj
  FROM wykonawcy_spis_decyzje
  WHERE id = NEW.decyzja_id
) IS NOT 'przyjecie'
BEGIN
  SELECT RAISE(ABORT, 'wykonawcę wpisuje się po decyzji o przyjęciu');
END;

CREATE TRIGGER wykonawcy_spis_dane_po_decyzji
BEFORE INSERT ON wykonawcy_spis_dane
WHEN (
  SELECT rodzaj
  FROM wykonawcy_spis_decyzje
  WHERE id = NEW.decyzja_id
) NOT IN ('przyjecie', 'zmiana_danych')
BEGIN
  SELECT RAISE(ABORT, 'dane, osobę, kanał i obszar wpisuje się po decyzji o przyjęciu albo zmianie danych');
END;

CREATE TRIGGER wykonawcy_spis_osoba_po_decyzji
BEFORE INSERT ON wykonawcy_spis_osoby
WHEN (
  SELECT rodzaj
  FROM wykonawcy_spis_decyzje
  WHERE id = NEW.decyzja_id
) NOT IN ('przyjecie', 'zmiana_danych')
BEGIN
  SELECT RAISE(ABORT, 'dane, osobę, kanał i obszar wpisuje się po decyzji o przyjęciu albo zmianie danych');
END;

CREATE TRIGGER wykonawcy_spis_kanal_po_decyzji
BEFORE INSERT ON wykonawcy_spis_kanaly
WHEN (
  SELECT rodzaj
  FROM wykonawcy_spis_decyzje
  WHERE id = NEW.decyzja_id
) NOT IN ('przyjecie', 'zmiana_danych')
BEGIN
  SELECT RAISE(ABORT, 'dane, osobę, kanał i obszar wpisuje się po decyzji o przyjęciu albo zmianie danych');
END;

CREATE TRIGGER wykonawcy_spis_obszar_po_decyzji
BEFORE INSERT ON wykonawcy_spis_obszary
WHEN (
  SELECT rodzaj
  FROM wykonawcy_spis_decyzje
  WHERE id = NEW.decyzja_id
) NOT IN ('przyjecie', 'zmiana_danych')
BEGIN
  SELECT RAISE(ABORT, 'dane, osobę, kanał i obszar wpisuje się po decyzji o przyjęciu albo zmianie danych');
END;

-- Zmiana telefonu albo pracownika zmienia to, kto dostaje kody do zamków, więc koniec osoby,
-- kanału i obszaru też ma decyzję człowieka.
CREATE TRIGGER wykonawcy_spis_koniec_osoby_po_decyzji
BEFORE UPDATE OF zakonczenie_decyzja_id ON wykonawcy_spis_osoby
WHEN NEW.zakonczenie_decyzja_id IS NOT NULL
  AND (
    SELECT rodzaj
    FROM wykonawcy_spis_decyzje
    WHERE id = NEW.zakonczenie_decyzja_id
  ) NOT IN ('zmiana_danych', 'wykreslenie')
BEGIN
  SELECT RAISE(ABORT, 'koniec osoby, kanału i obszaru wpisuje się po decyzji o zmianie danych albo wykreśleniu');
END;

CREATE TRIGGER wykonawcy_spis_koniec_kanalu_po_decyzji
BEFORE UPDATE OF zakonczenie_decyzja_id ON wykonawcy_spis_kanaly
WHEN NEW.zakonczenie_decyzja_id IS NOT NULL
  AND (
    SELECT rodzaj
    FROM wykonawcy_spis_decyzje
    WHERE id = NEW.zakonczenie_decyzja_id
  ) NOT IN ('zmiana_danych', 'wykreslenie')
BEGIN
  SELECT RAISE(ABORT, 'koniec osoby, kanału i obszaru wpisuje się po decyzji o zmianie danych albo wykreśleniu');
END;

CREATE TRIGGER wykonawcy_spis_koniec_obszaru_po_decyzji
BEFORE UPDATE OF zakonczenie_decyzja_id ON wykonawcy_spis_obszary
WHEN NEW.zakonczenie_decyzja_id IS NOT NULL
  AND (
    SELECT rodzaj
    FROM wykonawcy_spis_decyzje
    WHERE id = NEW.zakonczenie_decyzja_id
  ) NOT IN ('zmiana_danych', 'wykreslenie')
BEGIN
  SELECT RAISE(ABORT, 'koniec osoby, kanału i obszaru wpisuje się po decyzji o zmianie danych albo wykreśleniu');
END;

-- Stan wykonawcy -----------------------------------------------------------------------------------

-- Wykonawca jest zatwierdzony od przyjęcia; wstrzymany wraca do zatwierdzonych po decyzji
-- o przywróceniu; wykreślenie jest ostateczne. Każdy stan ma decyzję swojego rodzaju.
CREATE TRIGGER wykonawcy_spis_stan_z_przejsciem
BEFORE INSERT ON wykonawcy_spis_stany
WHEN NOT (
  CASE (
    SELECT stan
    FROM wykonawcy_spis_stany
    WHERE wykonawca_id = NEW.wykonawca_id
    ORDER BY id DESC
    LIMIT 1
  )
    WHEN 'zatwierdzony' THEN NEW.stan IN ('wstrzymany', 'wykreslony')
    WHEN 'wstrzymany' THEN NEW.stan IN ('zatwierdzony', 'wykreslony')
    WHEN 'wykreslony' THEN 0
    ELSE NEW.stan = 'zatwierdzony'
  END
)
BEGIN
  SELECT RAISE(ABORT, 'stan wykonawcy: zatwierdzony przy przyjęciu, wstrzymany ⇄ zatwierdzony, wykreślony na zawsze');
END;

CREATE TRIGGER wykonawcy_spis_stan_z_decyzja_swojego_rodzaju
BEFORE INSERT ON wykonawcy_spis_stany
WHEN NOT (
  SELECT
    CASE NEW.stan
      WHEN 'zatwierdzony' THEN rodzaj IN ('przyjecie', 'przywrocenie')
      WHEN 'wstrzymany' THEN rodzaj = 'wstrzymanie'
      WHEN 'wykreslony' THEN rodzaj = 'wykreslenie'
    END
  FROM wykonawcy_spis_decyzje
  WHERE id = NEW.decyzja_id
)
BEGIN
  SELECT RAISE(ABORT, 'zatwierdzenie ma decyzję o przyjęciu albo przywróceniu, wstrzymanie o wstrzymaniu, a wykreślenie o wykreśleniu');
END;

-- Wykreślony wykonawca nie dostaje nowych osób, kanałów ani obszarów; jego pracownicy tracą prawo
-- do kodów do zamków.
CREATE TRIGGER wykonawcy_spis_osoba_nie_u_wykreslonego
BEFORE INSERT ON wykonawcy_spis_osoby
WHEN (
  SELECT stan
  FROM wykonawcy_spis_stany
  WHERE wykonawca_id = NEW.wykonawca_id
  ORDER BY id DESC
  LIMIT 1
) IS 'wykreslony'
BEGIN
  SELECT RAISE(ABORT, 'wykreślony wykonawca nie dostaje nowych danych, osób, kanałów ani obszarów');
END;

CREATE TRIGGER wykonawcy_spis_kanal_nie_u_wykreslonego
BEFORE INSERT ON wykonawcy_spis_kanaly
WHEN (
  SELECT stan
  FROM wykonawcy_spis_stany
  WHERE wykonawca_id = NEW.wykonawca_id
  ORDER BY id DESC
  LIMIT 1
) IS 'wykreslony'
BEGIN
  SELECT RAISE(ABORT, 'wykreślony wykonawca nie dostaje nowych danych, osób, kanałów ani obszarów');
END;

CREATE TRIGGER wykonawcy_spis_obszar_nie_u_wykreslonego
BEFORE INSERT ON wykonawcy_spis_obszary
WHEN (
  SELECT stan
  FROM wykonawcy_spis_stany
  WHERE wykonawca_id = NEW.wykonawca_id
  ORDER BY id DESC
  LIMIT 1
) IS 'wykreslony'
BEGIN
  SELECT RAISE(ABORT, 'wykreślony wykonawca nie dostaje nowych danych, osób, kanałów ani obszarów');
END;

CREATE TRIGGER wykonawcy_spis_dane_nie_u_wykreslonego
BEFORE INSERT ON wykonawcy_spis_dane
WHEN (
  SELECT stan
  FROM wykonawcy_spis_stany
  WHERE wykonawca_id = NEW.wykonawca_id
  ORDER BY id DESC
  LIMIT 1
) IS 'wykreslony'
BEGIN
  SELECT RAISE(ABORT, 'wykreślony wykonawca nie dostaje nowych danych, osób, kanałów ani obszarów');
END;

-- Wersje danych ------------------------------------------------------------------------------------

CREATE TRIGGER wykonawcy_spis_dane_powstaja_aktualne
BEFORE INSERT ON wykonawcy_spis_dane
WHEN NEW.aktualne <> 1
BEGIN
  SELECT RAISE(ABORT, 'wersja danych wykonawcy powstaje jako aktualna');
END;

CREATE TRIGGER wykonawcy_spis_nowe_dane_zeruja_poprzednie
AFTER INSERT ON wykonawcy_spis_dane
BEGIN
  UPDATE wykonawcy_spis_dane
  SET aktualne = 0
  WHERE wykonawca_id = NEW.wykonawca_id
    AND id <> NEW.id
    AND aktualne = 1;
END;

CREATE TRIGGER wykonawcy_spis_dane_bez_zmian
BEFORE UPDATE ON wykonawcy_spis_dane
WHEN NEW.id IS NOT OLD.id
  OR NEW.wykonawca_id IS NOT OLD.wykonawca_id
  OR NEW.firma IS NOT OLD.firma
  OR NEW.nip IS NOT OLD.nip
  OR NEW.krs IS NOT OLD.krs
  OR NEW.ceidg IS NOT OLD.ceidg
  OR NEW.adres IS NOT OLD.adres
  OR NEW.zakres_uslug IS NOT OLD.zakres_uslug
  OR NEW.srodki_wlasne IS NOT OLD.srodki_wlasne
  OR NEW.umowa_od IS NOT OLD.umowa_od
  OR NEW.umowa_do IS NOT OLD.umowa_do
  OR NEW.umowa_podpisal IS NOT OLD.umowa_podpisal
  OR NEW.umowa_sciezka IS NOT OLD.umowa_sciezka
  OR NEW.umowa_skrot IS NOT OLD.umowa_skrot
  OR NEW.polisa_do IS NOT OLD.polisa_do
  OR NEW.polisa_sciezka IS NOT OLD.polisa_sciezka
  OR NEW.polisa_skrot IS NOT OLD.polisa_skrot
  OR NEW.decyzja_id IS NOT OLD.decyzja_id
  OR NEW.wpisano IS NOT OLD.wpisano
  OR (
    NEW.aktualne IS NOT OLD.aktualne
    AND NOT (
      OLD.aktualne = 1
      AND NEW.aktualne = 0
      AND EXISTS (
        SELECT 1
        FROM wykonawcy_spis_dane
        WHERE wykonawca_id = OLD.wykonawca_id
          AND id <> OLD.id
          AND aktualne = 1
      )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'danych wykonawcy się nie poprawia; zmiana to nowa wersja po decyzji');
END;

-- Osoby, kanały i obszary --------------------------------------------------------------------------

-- Osoba, kanał i obszar powstają bieżące, a koniec przechodzi przez regułę zmiany.
CREATE TRIGGER wykonawcy_spis_osoba_powstaje_biezaca
BEFORE INSERT ON wykonawcy_spis_osoby
WHEN NEW.do IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'osoba, kanał i obszar powstają jako bieżące');
END;

CREATE TRIGGER wykonawcy_spis_kanal_powstaje_biezacy
BEFORE INSERT ON wykonawcy_spis_kanaly
WHEN NEW.do IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'osoba, kanał i obszar powstają jako bieżące');
END;

CREATE TRIGGER wykonawcy_spis_obszar_powstaje_biezacy
BEFORE INSERT ON wykonawcy_spis_obszary
WHEN NEW.do IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'osoba, kanał i obszar powstają jako bieżące');
END;

-- Wierszy się nie zmienia poza jednorazowym końcem z decyzją.
CREATE TRIGGER wykonawcy_spis_osoby_bez_zmian
BEFORE UPDATE ON wykonawcy_spis_osoby
WHEN NEW.id IS NOT OLD.id
  OR NEW.wykonawca_id IS NOT OLD.wykonawca_id
  OR NEW.imiona IS NOT OLD.imiona
  OR NEW.nazwisko IS NOT OLD.nazwisko
  OR NEW.telefon IS NOT OLD.telefon
  OR NEW.kody_do_zamkow IS NOT OLD.kody_do_zamkow
  OR NEW.od IS NOT OLD.od
  OR NEW.decyzja_id IS NOT OLD.decyzja_id
  OR (OLD.do IS NOT NULL AND (NEW.do IS NOT OLD.do OR NEW.zakonczenie_decyzja_id IS NOT OLD.zakonczenie_decyzja_id))
BEGIN
  SELECT RAISE(ABORT, 'osoby, kanału ani obszaru się nie zmienia; mogą się tylko raz zakończyć decyzją');
END;

CREATE TRIGGER wykonawcy_spis_kanaly_bez_zmian
BEFORE UPDATE ON wykonawcy_spis_kanaly
WHEN NEW.id IS NOT OLD.id
  OR NEW.wykonawca_id IS NOT OLD.wykonawca_id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.adres IS NOT OLD.adres
  OR NEW.od IS NOT OLD.od
  OR NEW.decyzja_id IS NOT OLD.decyzja_id
  OR (OLD.do IS NOT NULL AND (NEW.do IS NOT OLD.do OR NEW.zakonczenie_decyzja_id IS NOT OLD.zakonczenie_decyzja_id))
BEGIN
  SELECT RAISE(ABORT, 'osoby, kanału ani obszaru się nie zmienia; mogą się tylko raz zakończyć decyzją');
END;

CREATE TRIGGER wykonawcy_spis_obszary_bez_zmian
BEFORE UPDATE ON wykonawcy_spis_obszary
WHEN NEW.id IS NOT OLD.id
  OR NEW.wykonawca_id IS NOT OLD.wykonawca_id
  OR NEW.obszar IS NOT OLD.obszar
  OR NEW.dostepnosc IS NOT OLD.dostepnosc
  OR NEW.czas_dojazdu_do_usterki IS NOT OLD.czas_dojazdu_do_usterki
  OR NEW.od IS NOT OLD.od
  OR NEW.decyzja_id IS NOT OLD.decyzja_id
  OR (OLD.do IS NOT NULL AND (NEW.do IS NOT OLD.do OR NEW.zakonczenie_decyzja_id IS NOT OLD.zakonczenie_decyzja_id))
BEGIN
  SELECT RAISE(ABORT, 'osoby, kanału ani obszaru się nie zmienia; mogą się tylko raz zakończyć decyzją');
END;

-- Wiersze bez zmian ----------------------------------------------------------------------------

CREATE TRIGGER wykonawcy_spis_decyzje_bez_zmian
BEFORE UPDATE ON wykonawcy_spis_decyzje
BEGIN
  SELECT RAISE(ABORT, 'decyzji, wpisu wykonawcy ani jego stanu się nie zmienia; błąd poprawia nowy wpis');
END;

CREATE TRIGGER wykonawcy_spis_wpisy_bez_zmian
BEFORE UPDATE ON wykonawcy_spis_wpisy
BEGIN
  SELECT RAISE(ABORT, 'decyzji, wpisu wykonawcy ani jego stanu się nie zmienia; błąd poprawia nowy wpis');
END;

CREATE TRIGGER wykonawcy_spis_stany_bez_zmian
BEFORE UPDATE ON wykonawcy_spis_stany
BEGIN
  SELECT RAISE(ABORT, 'decyzji, wpisu wykonawcy ani jego stanu się nie zmienia; błąd poprawia nowy wpis');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabel z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER wykonawcy_spis_decyzje_bez_zastepowania
BEFORE INSERT ON wykonawcy_spis_decyzje
WHEN EXISTS (
  SELECT 1
  FROM wykonawcy_spis_decyzje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER wykonawcy_spis_wpisy_bez_zastepowania
BEFORE INSERT ON wykonawcy_spis_wpisy
WHEN EXISTS (
  SELECT 1
  FROM wykonawcy_spis_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER wykonawcy_spis_dane_bez_zastepowania
BEFORE INSERT ON wykonawcy_spis_dane
WHEN EXISTS (
  SELECT 1
  FROM wykonawcy_spis_dane
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER wykonawcy_spis_osoby_bez_zastepowania
BEFORE INSERT ON wykonawcy_spis_osoby
WHEN EXISTS (
  SELECT 1
  FROM wykonawcy_spis_osoby
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER wykonawcy_spis_kanaly_bez_zastepowania
BEFORE INSERT ON wykonawcy_spis_kanaly
WHEN EXISTS (
  SELECT 1
  FROM wykonawcy_spis_kanaly
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER wykonawcy_spis_obszary_bez_zastepowania
BEFORE INSERT ON wykonawcy_spis_obszary
WHEN EXISTS (
  SELECT 1
  FROM wykonawcy_spis_obszary
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER wykonawcy_spis_stany_bez_zastepowania
BEFORE INSERT ON wykonawcy_spis_stany
WHEN EXISTS (
  SELECT 1
  FROM wykonawcy_spis_stany
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

-- Wykreślony wykonawca zostaje, bo wskazują go zlecenia i koszty w teczkach lokali (R9).
CREATE TRIGGER wykonawcy_spis_decyzje_bez_usuwania
BEFORE DELETE ON wykonawcy_spis_decyzje
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER wykonawcy_spis_wpisy_bez_usuwania
BEFORE DELETE ON wykonawcy_spis_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER wykonawcy_spis_dane_bez_usuwania
BEFORE DELETE ON wykonawcy_spis_dane
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER wykonawcy_spis_osoby_bez_usuwania
BEFORE DELETE ON wykonawcy_spis_osoby
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER wykonawcy_spis_kanaly_bez_usuwania
BEFORE DELETE ON wykonawcy_spis_kanaly
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER wykonawcy_spis_obszary_bez_usuwania
BEFORE DELETE ON wykonawcy_spis_obszary
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER wykonawcy_spis_stany_bez_usuwania
BEFORE DELETE ON wykonawcy_spis_stany
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;
