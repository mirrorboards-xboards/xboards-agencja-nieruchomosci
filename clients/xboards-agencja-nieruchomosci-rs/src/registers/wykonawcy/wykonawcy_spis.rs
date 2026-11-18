//! The kind `wykonawcy-spis` of the room `registers/wykonawcy`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "wykonawcy-spis",
    requires: &[],
    migrations: &[
        r#"-- Wykonawcy, którym biuro zleca pracę w lokalach krótkoterminowych: sprzątanie, gospodarze na
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
"#,
    ],
};

/// The values of column `decyzje.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecyzjeRodzaj {
    Przyjecie,
    ZmianaDanych,
    Cennik,
    Przypisanie,
    Wstrzymanie,
    Przywrocenie,
    Wykreslenie,
}

impl DecyzjeRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[DecyzjeRodzaj] = &[
        DecyzjeRodzaj::Przyjecie,
        DecyzjeRodzaj::ZmianaDanych,
        DecyzjeRodzaj::Cennik,
        DecyzjeRodzaj::Przypisanie,
        DecyzjeRodzaj::Wstrzymanie,
        DecyzjeRodzaj::Przywrocenie,
        DecyzjeRodzaj::Wykreslenie,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            DecyzjeRodzaj::Przyjecie => "przyjecie",
            DecyzjeRodzaj::ZmianaDanych => "zmiana_danych",
            DecyzjeRodzaj::Cennik => "cennik",
            DecyzjeRodzaj::Przypisanie => "przypisanie",
            DecyzjeRodzaj::Wstrzymanie => "wstrzymanie",
            DecyzjeRodzaj::Przywrocenie => "przywrocenie",
            DecyzjeRodzaj::Wykreslenie => "wykreslenie",
        }
    }
}

impl xmeta_room::FromSqlValue for DecyzjeRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "przyjecie" => Ok(DecyzjeRodzaj::Przyjecie),
                    "zmiana_danych" => Ok(DecyzjeRodzaj::ZmianaDanych),
                    "cennik" => Ok(DecyzjeRodzaj::Cennik),
                    "przypisanie" => Ok(DecyzjeRodzaj::Przypisanie),
                    "wstrzymanie" => Ok(DecyzjeRodzaj::Wstrzymanie),
                    "przywrocenie" => Ok(DecyzjeRodzaj::Przywrocenie),
                    "wykreslenie" => Ok(DecyzjeRodzaj::Wykreslenie),
                    other => Err(format!("{other} is no value of DecyzjeRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for DecyzjeRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `kanaly.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KanalyRodzaj {
    Aplikacja,
    Sms,
    Email,
}

impl KanalyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[KanalyRodzaj] = &[
        KanalyRodzaj::Aplikacja,
        KanalyRodzaj::Sms,
        KanalyRodzaj::Email,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            KanalyRodzaj::Aplikacja => "aplikacja",
            KanalyRodzaj::Sms => "sms",
            KanalyRodzaj::Email => "email",
        }
    }
}

impl xmeta_room::FromSqlValue for KanalyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "aplikacja" => Ok(KanalyRodzaj::Aplikacja),
                    "sms" => Ok(KanalyRodzaj::Sms),
                    "email" => Ok(KanalyRodzaj::Email),
                    other => Err(format!("{other} is no value of KanalyRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for KanalyRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `stany.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StanyStan {
    Zatwierdzony,
    Wstrzymany,
    Wykreslony,
}

impl StanyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[StanyStan] = &[
        StanyStan::Zatwierdzony,
        StanyStan::Wstrzymany,
        StanyStan::Wykreslony,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            StanyStan::Zatwierdzony => "zatwierdzony",
            StanyStan::Wstrzymany => "wstrzymany",
            StanyStan::Wykreslony => "wykreslony",
        }
    }
}

impl xmeta_room::FromSqlValue for StanyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "zatwierdzony" => Ok(StanyStan::Zatwierdzony),
                    "wstrzymany" => Ok(StanyStan::Wstrzymany),
                    "wykreslony" => Ok(StanyStan::Wykreslony),
                    other => Err(format!("{other} is no value of StanyStan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for StanyStan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    Sprzatanie,
    Gospodarz,
    NaprawyISerwis,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::Sprzatanie,
        WpisyRodzaj::Gospodarz,
        WpisyRodzaj::NaprawyISerwis,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::Sprzatanie => "sprzatanie",
            WpisyRodzaj::Gospodarz => "gospodarz",
            WpisyRodzaj::NaprawyISerwis => "naprawy_i_serwis",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "sprzatanie" => Ok(WpisyRodzaj::Sprzatanie),
                    "gospodarz" => Ok(WpisyRodzaj::Gospodarz),
                    "naprawy_i_serwis" => Ok(WpisyRodzaj::NaprawyISerwis),
                    other => Err(format!("{other} is no value of WpisyRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The parameters of `dane_wykonawcy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DaneWykonawcyParams {
    pub wykonawca_id: String,
}

/// A row of `dane_wykonawcy`.
#[derive(Debug, Clone, PartialEq)]
pub struct DaneWykonawcyRow {
    pub id: i64,
    pub wykonawca_id: String,
    pub aktualne: bool,
    pub firma: String,
    pub nip: Option<String>,
    pub krs: Option<String>,
    pub ceidg: bool,
    pub adres: String,
    pub zakres_uslug: String,
    pub srodki_wlasne: bool,
    pub umowa_od: String,
    pub umowa_do: Option<String>,
    pub umowa_podpisal: String,
    pub umowa_sciezka: String,
    pub umowa_skrot: String,
    pub polisa_do: Option<String>,
    pub polisa_sciezka: Option<String>,
    pub polisa_skrot: Option<String>,
    pub decyzja_id: String,
    pub wpisano: String,
}

impl DaneWykonawcyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(DaneWykonawcyRow {
            id: row.get("id")?,
            wykonawca_id: row.get("wykonawca_id")?,
            aktualne: row.get("aktualne")?,
            firma: row.get("firma")?,
            nip: row.get("nip")?,
            krs: row.get("krs")?,
            ceidg: row.get("ceidg")?,
            adres: row.get("adres")?,
            zakres_uslug: row.get("zakres_uslug")?,
            srodki_wlasne: row.get("srodki_wlasne")?,
            umowa_od: row.get("umowa_od")?,
            umowa_do: row.get("umowa_do")?,
            umowa_podpisal: row.get("umowa_podpisal")?,
            umowa_sciezka: row.get("umowa_sciezka")?,
            umowa_skrot: row.get("umowa_skrot")?,
            polisa_do: row.get("polisa_do")?,
            polisa_sciezka: row.get("polisa_sciezka")?,
            polisa_skrot: row.get("polisa_skrot")?,
            decyzja_id: row.get("decyzja_id")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Wszystkie wersje danych wykonawcy, najnowsza pierwsza; aktualna ma aktualne = 1.
pub fn dane_wykonawcy(
    room: &impl xmeta_room::MetaRoom,
    params: &DaneWykonawcyParams,
) -> Result<Vec<DaneWykonawcyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM wykonawcy_spis_dane
WHERE wykonawca_id = :wykonawca_id
ORDER BY id DESC"#,
        parameters: Vec::from([
            ("wykonawca_id", xmeta_room::ToSqlValue::to_sql_value(&params.wykonawca_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(DaneWykonawcyRow::from_row).collect()
}

/// The parameters of `wpisz_dane_wykonawcy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszDaneWykonawcyParams {
    pub wykonawca_id: String,
    pub firma: String,
    pub nip: Option<String>,
    pub krs: Option<String>,
    pub ceidg: bool,
    pub adres: String,
    pub zakres_uslug: String,
    pub srodki_wlasne: bool,
    pub umowa_od: String,
    pub umowa_do: Option<String>,
    pub umowa_podpisal: String,
    pub umowa_sciezka: String,
    pub umowa_skrot: String,
    pub polisa_do: Option<String>,
    pub polisa_sciezka: Option<String>,
    pub polisa_skrot: Option<String>,
    pub decyzja_id: String,
    pub wpisano: String,
}

/// Nowa wersja danych po decyzji o zmianie danych, np. przedłużona umowa albo nowa polisa.
pub fn wpisz_dane_wykonawcy(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszDaneWykonawcyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wykonawcy_spis_dane (wykonawca_id, aktualne, firma, nip, krs, ceidg, adres, zakres_uslug,
  srodki_wlasne, umowa_od, umowa_do, umowa_podpisal, umowa_sciezka, umowa_skrot, polisa_do,
  polisa_sciezka, polisa_skrot, decyzja_id, wpisano)
VALUES (:wykonawca_id, 1, :firma, :nip, :krs, :ceidg, :adres, :zakres_uslug, :srodki_wlasne,
  :umowa_od, :umowa_do, :umowa_podpisal, :umowa_sciezka, :umowa_skrot, :polisa_do,
  :polisa_sciezka, :polisa_skrot, :decyzja_id, :wpisano)"#,
                parameters: Vec::from([
                    (
                        "wykonawca_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wykonawca_id),
                    ),
                    ("firma", xmeta_room::ToSqlValue::to_sql_value(&params.firma)),
                    ("nip", xmeta_room::ToSqlValue::to_sql_value(&params.nip)),
                    ("krs", xmeta_room::ToSqlValue::to_sql_value(&params.krs)),
                    ("ceidg", xmeta_room::ToSqlValue::to_sql_value(&params.ceidg)),
                    ("adres", xmeta_room::ToSqlValue::to_sql_value(&params.adres)),
                    (
                        "zakres_uslug",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zakres_uslug),
                    ),
                    (
                        "srodki_wlasne",
                        xmeta_room::ToSqlValue::to_sql_value(&params.srodki_wlasne),
                    ),
                    ("umowa_od", xmeta_room::ToSqlValue::to_sql_value(&params.umowa_od)),
                    ("umowa_do", xmeta_room::ToSqlValue::to_sql_value(&params.umowa_do)),
                    (
                        "umowa_podpisal",
                        xmeta_room::ToSqlValue::to_sql_value(&params.umowa_podpisal),
                    ),
                    (
                        "umowa_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.umowa_sciezka),
                    ),
                    (
                        "umowa_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.umowa_skrot),
                    ),
                    (
                        "polisa_do",
                        xmeta_room::ToSqlValue::to_sql_value(&params.polisa_do),
                    ),
                    (
                        "polisa_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.polisa_sciezka),
                    ),
                    (
                        "polisa_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.polisa_skrot),
                    ),
                    (
                        "decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_id),
                    ),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
        ],
    )
}

/// A row of `decyzje_wykonawcow`.
#[derive(Debug, Clone, PartialEq)]
pub struct DecyzjeWykonawcowRow {
    pub id: String,
    pub rodzaj: DecyzjeRodzaj,
    pub decyduje: String,
    pub tresc: String,
    pub dotyczy_skrot: String,
    pub podpis: String,
    pub podpisano: String,
    pub wpisal: String,
    pub wpisano: String,
}

impl DecyzjeWykonawcowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(DecyzjeWykonawcowRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            decyduje: row.get("decyduje")?,
            tresc: row.get("tresc")?,
            dotyczy_skrot: row.get("dotyczy_skrot")?,
            podpis: row.get("podpis")?,
            podpisano: row.get("podpisano")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Decyzje człowieka przepisane do rejestru, najnowsze pierwsze.
pub fn decyzje_wykonawcow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<DecyzjeWykonawcowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM wykonawcy_spis_decyzje
ORDER BY podpisano DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(DecyzjeWykonawcowRow::from_row).collect()
}

/// The parameters of `wpisz_decyzje_o_wykonawcy`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszDecyzjeOWykonawcyParams {
    pub id: String,
    pub rodzaj: DecyzjeRodzaj,
    pub decyduje: String,
    pub tresc: String,
    pub dotyczy_skrot: String,
    pub podpis: String,
    pub podpisano: String,
    pub wpisal: String,
    pub wpisano: String,
}

/// Decyzja człowieka podpisana jego kluczem, przepisana do rejestru razem z podpisem, zanim
/// Kierownik najmu wpisze zmianę, której dotyczy. Identyfikator (UUID) nadaje aplikacja.
pub fn wpisz_decyzje_o_wykonawcy(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszDecyzjeOWykonawcyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wykonawcy_spis_decyzje (id, rodzaj, decyduje, tresc, dotyczy_skrot, podpis, podpisano,
  wpisal, wpisano)
VALUES (:id, :rodzaj, :decyduje, :tresc, :dotyczy_skrot, :podpis, :podpisano, :wpisal, :wpisano)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("decyduje", xmeta_room::ToSqlValue::to_sql_value(&params.decyduje)),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                    (
                        "dotyczy_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_skrot),
                    ),
                    ("podpis", xmeta_room::ToSqlValue::to_sql_value(&params.podpis)),
                    (
                        "podpisano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.podpisano),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
        ],
    )
}

/// A row of `kanaly_wykonawcow`.
#[derive(Debug, Clone, PartialEq)]
pub struct KanalyWykonawcowRow {
    pub id: i64,
    pub wykonawca_id: String,
    pub rodzaj: KanalyRodzaj,
    pub adres: String,
    pub od: String,
    pub r#do: Option<String>,
    pub decyzja_id: String,
    pub zakonczenie_decyzja_id: Option<String>,
}

impl KanalyWykonawcowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(KanalyWykonawcowRow {
            id: row.get("id")?,
            wykonawca_id: row.get("wykonawca_id")?,
            rodzaj: row.get("rodzaj")?,
            adres: row.get("adres")?,
            od: row.get("od")?,
            r#do: row.get("do")?,
            decyzja_id: row.get("decyzja_id")?,
            zakonczenie_decyzja_id: row.get("zakonczenie_decyzja_id")?,
        })
    }
}

/// Bieżące kanały zleceń wszystkich wykonawców.
pub fn kanaly_wykonawcow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<KanalyWykonawcowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM wykonawcy_spis_kanaly
WHERE do IS NULL
ORDER BY wykonawca_id, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(KanalyWykonawcowRow::from_row).collect()
}

/// The parameters of `wpisz_kanal_wykonawcy`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszKanalWykonawcyParams {
    pub wykonawca_id: String,
    pub rodzaj: KanalyRodzaj,
    pub adres: String,
    pub od: String,
    pub decyzja_id: String,
}

/// Kanał zleceń po decyzji o przyjęciu albo zmianie danych.
pub fn wpisz_kanal_wykonawcy(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszKanalWykonawcyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wykonawcy_spis_kanaly (wykonawca_id, rodzaj, adres, od, decyzja_id)
VALUES (:wykonawca_id, :rodzaj, :adres, :od, :decyzja_id)"#,
                parameters: Vec::from([
                    (
                        "wykonawca_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wykonawca_id),
                    ),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("adres", xmeta_room::ToSqlValue::to_sql_value(&params.adres)),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    (
                        "decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_id),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zakoncz_kanal_wykonawcy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZakonczKanalWykonawcyParams {
    pub r#do: Option<String>,
    pub zakonczenie_decyzja_id: String,
    pub id: i64,
}

/// Kanał, którego wykonawca już nie używa, po decyzji o zmianie danych albo wykreśleniu.
pub fn zakoncz_kanal_wykonawcy(
    room: &impl xmeta_room::MetaRoom,
    params: &ZakonczKanalWykonawcyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE wykonawcy_spis_kanaly
SET do = :do,
  zakonczenie_decyzja_id = :zakonczenie_decyzja_id
WHERE id = :id"#,
                parameters: Vec::from([
                    ("do", xmeta_room::ToSqlValue::to_sql_value(&params.r#do)),
                    (
                        "zakonczenie_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.zakonczenie_decyzja_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `obszary_wykonawcow`.
#[derive(Debug, Clone, PartialEq)]
pub struct ObszaryWykonawcowRow {
    pub id: i64,
    pub wykonawca_id: String,
    pub obszar: String,
    pub dostepnosc: String,
    pub czas_dojazdu_do_usterki: Option<String>,
    pub od: String,
    pub r#do: Option<String>,
    pub decyzja_id: String,
    pub zakonczenie_decyzja_id: Option<String>,
}

impl ObszaryWykonawcowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ObszaryWykonawcowRow {
            id: row.get("id")?,
            wykonawca_id: row.get("wykonawca_id")?,
            obszar: row.get("obszar")?,
            dostepnosc: row.get("dostepnosc")?,
            czas_dojazdu_do_usterki: row.get("czas_dojazdu_do_usterki")?,
            od: row.get("od")?,
            r#do: row.get("do")?,
            decyzja_id: row.get("decyzja_id")?,
            zakonczenie_decyzja_id: row.get("zakonczenie_decyzja_id")?,
        })
    }
}

/// Bieżące obszary pracy wszystkich wykonawców; Koordynator wybiera z nich wykonawcę dla lokalu.
pub fn obszary_wykonawcow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ObszaryWykonawcowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM wykonawcy_spis_obszary
WHERE do IS NULL
ORDER BY obszar, wykonawca_id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ObszaryWykonawcowRow::from_row).collect()
}

/// The parameters of `wpisz_obszar_wykonawcy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszObszarWykonawcyParams {
    pub wykonawca_id: String,
    pub obszar: String,
    pub dostepnosc: String,
    pub czas_dojazdu_do_usterki: Option<String>,
    pub od: String,
    pub decyzja_id: String,
}

/// Obszar pracy po decyzji o przyjęciu albo zmianie danych.
pub fn wpisz_obszar_wykonawcy(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszObszarWykonawcyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wykonawcy_spis_obszary (wykonawca_id, obszar, dostepnosc, czas_dojazdu_do_usterki, od,
  decyzja_id)
VALUES (:wykonawca_id, :obszar, :dostepnosc, :czas_dojazdu_do_usterki, :od, :decyzja_id)"#,
                parameters: Vec::from([
                    (
                        "wykonawca_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wykonawca_id),
                    ),
                    ("obszar", xmeta_room::ToSqlValue::to_sql_value(&params.obszar)),
                    (
                        "dostepnosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dostepnosc),
                    ),
                    (
                        "czas_dojazdu_do_usterki",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.czas_dojazdu_do_usterki,
                        ),
                    ),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    (
                        "decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_id),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zakoncz_obszar_wykonawcy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZakonczObszarWykonawcyParams {
    pub r#do: Option<String>,
    pub zakonczenie_decyzja_id: String,
    pub id: i64,
}

/// Obszar, w którym wykonawca już nie pracuje, po decyzji o zmianie danych albo wykreśleniu.
pub fn zakoncz_obszar_wykonawcy(
    room: &impl xmeta_room::MetaRoom,
    params: &ZakonczObszarWykonawcyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE wykonawcy_spis_obszary
SET do = :do,
  zakonczenie_decyzja_id = :zakonczenie_decyzja_id
WHERE id = :id"#,
                parameters: Vec::from([
                    ("do", xmeta_room::ToSqlValue::to_sql_value(&params.r#do)),
                    (
                        "zakonczenie_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.zakonczenie_decyzja_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `osoby_wykonawcow`.
#[derive(Debug, Clone, PartialEq)]
pub struct OsobyWykonawcowRow {
    pub id: i64,
    pub wykonawca_id: String,
    pub imiona: String,
    pub nazwisko: String,
    pub telefon: String,
    pub kody_do_zamkow: bool,
    pub od: String,
    pub r#do: Option<String>,
    pub decyzja_id: String,
    pub zakonczenie_decyzja_id: Option<String>,
}

impl OsobyWykonawcowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(OsobyWykonawcowRow {
            id: row.get("id")?,
            wykonawca_id: row.get("wykonawca_id")?,
            imiona: row.get("imiona")?,
            nazwisko: row.get("nazwisko")?,
            telefon: row.get("telefon")?,
            kody_do_zamkow: row.get("kody_do_zamkow")?,
            od: row.get("od")?,
            r#do: row.get("do")?,
            decyzja_id: row.get("decyzja_id")?,
            zakonczenie_decyzja_id: row.get("zakonczenie_decyzja_id")?,
        })
    }
}

/// Bieżący pracownicy wykonawców. Koordynator porównuje z nimi nadawcę wiadomości i wysyła kod do
/// zamka tylko osobie z kody_do_zamkow = 1.
pub fn osoby_wykonawcow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<OsobyWykonawcowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM wykonawcy_spis_osoby
WHERE do IS NULL
ORDER BY wykonawca_id, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(OsobyWykonawcowRow::from_row).collect()
}

/// The parameters of `osoby_wykonawcy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OsobyWykonawcyParams {
    pub wykonawca_id: String,
}

/// A row of `osoby_wykonawcy`.
#[derive(Debug, Clone, PartialEq)]
pub struct OsobyWykonawcyRow {
    pub id: i64,
    pub wykonawca_id: String,
    pub imiona: String,
    pub nazwisko: String,
    pub telefon: String,
    pub kody_do_zamkow: bool,
    pub od: String,
    pub r#do: Option<String>,
    pub decyzja_id: String,
    pub zakonczenie_decyzja_id: Option<String>,
}

impl OsobyWykonawcyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(OsobyWykonawcyRow {
            id: row.get("id")?,
            wykonawca_id: row.get("wykonawca_id")?,
            imiona: row.get("imiona")?,
            nazwisko: row.get("nazwisko")?,
            telefon: row.get("telefon")?,
            kody_do_zamkow: row.get("kody_do_zamkow")?,
            od: row.get("od")?,
            r#do: row.get("do")?,
            decyzja_id: row.get("decyzja_id")?,
            zakonczenie_decyzja_id: row.get("zakonczenie_decyzja_id")?,
        })
    }
}

/// Pracownicy jednego wykonawcy, bieżący i zakończeni.
pub fn osoby_wykonawcy(
    room: &impl xmeta_room::MetaRoom,
    params: &OsobyWykonawcyParams,
) -> Result<Vec<OsobyWykonawcyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM wykonawcy_spis_osoby
WHERE wykonawca_id = :wykonawca_id
ORDER BY id DESC"#,
        parameters: Vec::from([
            ("wykonawca_id", xmeta_room::ToSqlValue::to_sql_value(&params.wykonawca_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(OsobyWykonawcyRow::from_row).collect()
}

/// The parameters of `wpisz_osobe_wykonawcy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszOsobeWykonawcyParams {
    pub wykonawca_id: String,
    pub imiona: String,
    pub nazwisko: String,
    pub telefon: String,
    pub kody_do_zamkow: bool,
    pub od: String,
    pub decyzja_id: String,
}

/// Pracownik wykonawcy po decyzji o przyjęciu albo zmianie danych.
pub fn wpisz_osobe_wykonawcy(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszOsobeWykonawcyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wykonawcy_spis_osoby (wykonawca_id, imiona, nazwisko, telefon, kody_do_zamkow, od,
  decyzja_id)
VALUES (:wykonawca_id, :imiona, :nazwisko, :telefon, :kody_do_zamkow, :od, :decyzja_id)"#,
                parameters: Vec::from([
                    (
                        "wykonawca_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wykonawca_id),
                    ),
                    ("imiona", xmeta_room::ToSqlValue::to_sql_value(&params.imiona)),
                    ("nazwisko", xmeta_room::ToSqlValue::to_sql_value(&params.nazwisko)),
                    ("telefon", xmeta_room::ToSqlValue::to_sql_value(&params.telefon)),
                    (
                        "kody_do_zamkow",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kody_do_zamkow),
                    ),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    (
                        "decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_id),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zakoncz_osobe_wykonawcy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZakonczOsobeWykonawcyParams {
    pub r#do: Option<String>,
    pub zakonczenie_decyzja_id: String,
    pub id: i64,
}

/// Pracownik, który przestał pracować dla biura, po decyzji o zmianie danych albo wykreśleniu.
pub fn zakoncz_osobe_wykonawcy(
    room: &impl xmeta_room::MetaRoom,
    params: &ZakonczOsobeWykonawcyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE wykonawcy_spis_osoby
SET do = :do,
  zakonczenie_decyzja_id = :zakonczenie_decyzja_id
WHERE id = :id"#,
                parameters: Vec::from([
                    ("do", xmeta_room::ToSqlValue::to_sql_value(&params.r#do)),
                    (
                        "zakonczenie_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.zakonczenie_decyzja_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `stany_wykonawcy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StanyWykonawcyParams {
    pub wykonawca_id: String,
}

/// A row of `stany_wykonawcy`.
#[derive(Debug, Clone, PartialEq)]
pub struct StanyWykonawcyRow {
    pub id: i64,
    pub wykonawca_id: String,
    pub stan: StanyStan,
    pub od: String,
    pub powod: Option<String>,
    pub decyzja_id: String,
}

impl StanyWykonawcyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(StanyWykonawcyRow {
            id: row.get("id")?,
            wykonawca_id: row.get("wykonawca_id")?,
            stan: row.get("stan")?,
            od: row.get("od")?,
            powod: row.get("powod")?,
            decyzja_id: row.get("decyzja_id")?,
        })
    }
}

/// Historia stanu wykonawcy z powodami i decyzjami.
pub fn stany_wykonawcy(
    room: &impl xmeta_room::MetaRoom,
    params: &StanyWykonawcyParams,
) -> Result<Vec<StanyWykonawcyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM wykonawcy_spis_stany
WHERE wykonawca_id = :wykonawca_id
ORDER BY id"#,
        parameters: Vec::from([
            ("wykonawca_id", xmeta_room::ToSqlValue::to_sql_value(&params.wykonawca_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(StanyWykonawcyRow::from_row).collect()
}

/// The parameters of `zmien_stan_wykonawcy`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZmienStanWykonawcyParams {
    pub wykonawca_id: String,
    pub stan: StanyStan,
    pub od: String,
    pub powod: Option<String>,
    pub decyzja_id: String,
}

/// Wstrzymanie, przywrócenie albo wykreślenie po decyzji człowieka tego rodzaju.
pub fn zmien_stan_wykonawcy(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienStanWykonawcyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wykonawcy_spis_stany (wykonawca_id, stan, od, powod, decyzja_id)
VALUES (:wykonawca_id, :stan, :od, :powod, :decyzja_id)"#,
                parameters: Vec::from([
                    (
                        "wykonawca_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wykonawca_id),
                    ),
                    ("stan", xmeta_room::ToSqlValue::to_sql_value(&params.stan)),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    ("powod", xmeta_room::ToSqlValue::to_sql_value(&params.powod)),
                    (
                        "decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_id),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `wykonawcy`.
#[derive(Debug, Clone, PartialEq)]
pub struct WykonawcyRow {
    pub id: String,
    pub nazwa: String,
    pub rodzaj: WpisyRodzaj,
    pub specjalnosc: Option<String>,
    pub zakres_uslug: String,
    pub umowa_do: Option<String>,
    pub polisa_do: Option<String>,
    pub stan: StanyStan,
    pub stan_od: String,
}

impl WykonawcyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WykonawcyRow {
            id: row.get("id")?,
            nazwa: row.get("nazwa")?,
            rodzaj: row.get("rodzaj")?,
            specjalnosc: row.get("specjalnosc")?,
            zakres_uslug: row.get("zakres_uslug")?,
            umowa_do: row.get("umowa_do")?,
            polisa_do: row.get("polisa_do")?,
            stan: row.get("stan")?,
            stan_od: row.get("stan_od")?,
        })
    }
}

/// Wykonawcy z aktualnymi danymi i bieżącym stanem, w kolejności wpisania.
pub fn wykonawcy(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WykonawcyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT
  wpisy.id,
  wpisy.nazwa,
  wpisy.rodzaj,
  wpisy.specjalnosc,
  dane.zakres_uslug,
  dane.umowa_do,
  dane.polisa_do,
  stany.stan,
  stany.od AS stan_od
FROM wykonawcy_spis_wpisy AS wpisy
JOIN wykonawcy_spis_dane AS dane ON dane.wykonawca_id = wpisy.id
  AND dane.aktualne = 1
JOIN wykonawcy_spis_stany AS stany ON stany.wykonawca_id = wpisy.id
  AND stany.id = (
    SELECT max(id)
    FROM wykonawcy_spis_stany
    WHERE wykonawca_id = wpisy.id
  )
ORDER BY wpisy.rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(WykonawcyRow::from_row).collect()
}

/// The parameters of `wpisz_wykonawce`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszWykonawceParams {
    pub id: String,
    pub nazwa: String,
    pub rodzaj: WpisyRodzaj,
    pub specjalnosc: Option<String>,
    pub decyzja_id: String,
    pub wpisano: String,
    pub firma: String,
    pub nip: Option<String>,
    pub krs: Option<String>,
    pub ceidg: bool,
    pub adres: String,
    pub zakres_uslug: String,
    pub srodki_wlasne: bool,
    pub umowa_od: String,
    pub umowa_do: Option<String>,
    pub umowa_podpisal: String,
    pub umowa_sciezka: String,
    pub umowa_skrot: String,
    pub polisa_do: Option<String>,
    pub polisa_sciezka: Option<String>,
    pub polisa_skrot: Option<String>,
}

/// Nowy wykonawca po decyzji o przyjęciu i podpisaniu umowy: wpis, pierwsza wersja danych i stan
/// „zatwierdzony” z tą samą decyzją. Identyfikator (UUID) nadaje aplikacja.
pub fn wpisz_wykonawce(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszWykonawceParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wykonawcy_spis_wpisy (id, nazwa, rodzaj, specjalnosc, decyzja_id, wpisano)
VALUES (:id, :nazwa, :rodzaj, :specjalnosc, :decyzja_id, :wpisano)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("nazwa", xmeta_room::ToSqlValue::to_sql_value(&params.nazwa)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    (
                        "specjalnosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.specjalnosc),
                    ),
                    (
                        "decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_id),
                    ),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
            xmeta_room::Statement {
                sql: r#"INSERT INTO wykonawcy_spis_dane (wykonawca_id, aktualne, firma, nip, krs, ceidg, adres, zakres_uslug,
  srodki_wlasne, umowa_od, umowa_do, umowa_podpisal, umowa_sciezka, umowa_skrot, polisa_do,
  polisa_sciezka, polisa_skrot, decyzja_id, wpisano)
VALUES (:id, 1, :firma, :nip, :krs, :ceidg, :adres, :zakres_uslug, :srodki_wlasne, :umowa_od,
  :umowa_do, :umowa_podpisal, :umowa_sciezka, :umowa_skrot, :polisa_do, :polisa_sciezka,
  :polisa_skrot, :decyzja_id, :wpisano)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("firma", xmeta_room::ToSqlValue::to_sql_value(&params.firma)),
                    ("nip", xmeta_room::ToSqlValue::to_sql_value(&params.nip)),
                    ("krs", xmeta_room::ToSqlValue::to_sql_value(&params.krs)),
                    ("ceidg", xmeta_room::ToSqlValue::to_sql_value(&params.ceidg)),
                    ("adres", xmeta_room::ToSqlValue::to_sql_value(&params.adres)),
                    (
                        "zakres_uslug",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zakres_uslug),
                    ),
                    (
                        "srodki_wlasne",
                        xmeta_room::ToSqlValue::to_sql_value(&params.srodki_wlasne),
                    ),
                    ("umowa_od", xmeta_room::ToSqlValue::to_sql_value(&params.umowa_od)),
                    ("umowa_do", xmeta_room::ToSqlValue::to_sql_value(&params.umowa_do)),
                    (
                        "umowa_podpisal",
                        xmeta_room::ToSqlValue::to_sql_value(&params.umowa_podpisal),
                    ),
                    (
                        "umowa_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.umowa_sciezka),
                    ),
                    (
                        "umowa_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.umowa_skrot),
                    ),
                    (
                        "polisa_do",
                        xmeta_room::ToSqlValue::to_sql_value(&params.polisa_do),
                    ),
                    (
                        "polisa_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.polisa_sciezka),
                    ),
                    (
                        "polisa_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.polisa_skrot),
                    ),
                    (
                        "decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_id),
                    ),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
            xmeta_room::Statement {
                sql: r#"INSERT INTO wykonawcy_spis_stany (wykonawca_id, stan, od, decyzja_id)
VALUES (:id, 'zatwierdzony', date(:wpisano), :decyzja_id)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                    (
                        "decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_id),
                    ),
                ]),
            },
        ],
    )
}
