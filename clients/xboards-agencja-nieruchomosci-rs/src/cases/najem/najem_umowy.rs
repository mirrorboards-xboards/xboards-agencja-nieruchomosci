//! The kind `najem-umowy` of the room `cases/najem`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "najem-umowy",
    requires: &[
        &super::sprawa_decyzje::KIND,
        &super::sprawa_dokumenty::KIND,
        &super::sprawa_korespondencja::KIND,
        &super::sprawa_metryka::KIND,
        &super::sprawa_nieruchomosci::KIND,
        &super::sprawa_uczestnicy::KIND,
        &super::sprawa_umowy::KIND,
    ],
    migrations: &[
        r#"-- Umowy najmu: warunki umowy o zarządzanie, umowa najmu z opłatami i załącznikami, osoby
-- zamieszkujące z najemcą, lokal zastępczy przy najmie okazjonalnym, aneksy i koniec najmu. Biuro
-- nie jest stroną umowy najmu: projekt przygotowuje Opiekun najmu według wzorów biura, a podpisują
-- strony. Pisze Opiekun najmu.
--
-- Umowa najmu powstaje jako projekt. Do podpisu idzie za zgodą człowieka i przy najmie
-- okazjonalnym i instytucjonalnym tylko z załącznikami z ustawy. Od wyjścia z projektu jej treść
-- się nie zmienia: zmienia ją aneks z decyzją i podpisem. Z podpisu umowa nie wraca do projektu,
-- bo decyzja dotyczyła tej treści; poprawka to zakończenie projektu i nowa umowa.
--
-- Wysyłki wypowiedzenia i żądania opróżnienia lokalu wychodzą zawsze za zgodą człowieka (stan
-- czeka_na_zgode w rdzeniu), bo ocena przyczyny końca najmu jest oceną prawną.

-- Warunki umowy o zarządzanie, których potrzebuje najem, w wersjach: do jakiej kwoty biuro zleca
-- naprawy bez pytania wynajmującego i czy pieniądze idą przez biuro. Pierwsza wersja obowiązuje od
-- zawarcia umowy, każda następna od aneksu tej umowy. Bieżąca wersja to ostatnia.
CREATE TABLE najem_umowy_zarzadzanie (
  -- Klucz nadany przez bazę jest dodatni; zob. najem_umowy_zarzadzanie_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  umowa_id TEXT NOT NULL REFERENCES sprawa_umowy_wpisy (id),
  kwota_napraw INTEGER NOT NULL CHECK (kwota_napraw >= 0),
  przeplyw TEXT NOT NULL CHECK (przeplyw IN ('przez_biuro', 'wprost')),
  aneks_zdarzenie_id TEXT REFERENCES sprawa_umowy_zdarzenia (id),
  od TEXT NOT NULL
) STRICT;

-- Umowa najmu lokalu. Czynsz i kaucja to kwoty z dnia zawarcia; późniejsze zmiany czynszu leżą
-- w najem-czynsz.
CREATE TABLE najem_umowy_najmu (
  id TEXT PRIMARY KEY,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('zwykly', 'okazjonalny', 'instytucjonalny')),
  nieruchomosc_id TEXT NOT NULL REFERENCES sprawa_nieruchomosci_wpisy (id),
  poczatek TEXT NOT NULL,
  -- Koniec czasu oznaczonego; NULL to czas nieoznaczony.
  koniec TEXT,
  forma TEXT NOT NULL CHECK (forma IN ('pisemna', 'elektroniczna', 'dokumentowa')),
  stan TEXT NOT NULL CHECK (stan IN ('projekt', 'do_podpisu', 'zawarta', 'zakonczona')),
  koniec_powod TEXT,
  projekt_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  -- Zatwierdzona decyzja inna o przekazaniu stronom projektu do podpisu.
  do_podpisu_decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),
  zawarto TEXT,

  -- Pieniądze w groszach: czynsz miesięczny i kaucja; 0 to najem bez kaucji.
  czynsz INTEGER NOT NULL CHECK (czynsz > 0),
  -- Np. „z góry do 10. dnia miesiąca”.
  termin_platnosci TEXT,
  kaucja INTEGER NOT NULL CHECK (kaucja >= 0),

  -- Na co wynajmujący się zgodził i jak strony podzieliły naprawy, gdy inaczej niż ustawa.
  zwierzeta TEXT,
  podnajem TEXT,
  zmiany_w_lokalu TEXT,
  naprawy_podzial TEXT,
  wypowiedzenie_przypadki TEXT,

  -- Załączniki i zgłoszenia: oświadczenie najemcy w formie aktu notarialnego (art. 19a ust. 2
  -- pkt 1 i art. 19f ust. 3 UOPL), dzień przekazania kopii świadectwa charakterystyki
  -- energetycznej (art. 11 ust. 1 pkt 2 uceb) i zgłoszenie najmu okazjonalnego do urzędu
  -- skarbowego (art. 19b ust. 1 UOPL).
  oswiadczenie_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  swiadectwo_przekazano TEXT,
  zgloszenie_us_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  zgloszono_us TEXT,

  CONSTRAINT koniec_nie_przed_poczatkiem CHECK (koniec IS NULL OR koniec >= poczatek),
  -- Art. 19a ust. 1 UOPL: najem okazjonalny jest na czas oznaczony, nie dłuższy niż 10 lat.
  CONSTRAINT okazjonalny_najwyzej_10_lat CHECK (
    rodzaj <> 'okazjonalny' OR (koniec IS NOT NULL AND koniec <= date(poczatek, '+10 years'))
  ),
  -- Art. 19f ust. 2 UOPL.
  CONSTRAINT instytucjonalny_na_czas_oznaczony CHECK (rodzaj <> 'instytucjonalny' OR koniec IS NOT NULL),
  -- Art. 19a ust. 6 i art. 19g ust. 1 UOPL: forma pisemna pod rygorem nieważności; forma
  -- elektroniczna jest jej równoważna (art. 78¹ § 2 KC).
  CONSTRAINT forma_pisemna_przy_okazjonalnym_i_instytucjonalnym CHECK (
    rodzaj = 'zwykly' OR forma IN ('pisemna', 'elektroniczna')
  ),
  -- Art. 6 ust. 1, art. 19a ust. 4 i art. 19f ust. 5 UOPL: kaucja najwyżej dwunastokrotność
  -- czynszu z dnia zawarcia przy najmie zwykłym, a sześciokrotność przy okazjonalnym
  -- i instytucjonalnym.
  CONSTRAINT kaucja_w_granicy CHECK (
    kaucja <= czynsz * CASE rodzaj WHEN 'zwykly' THEN 12 ELSE 6 END
  ),
  CONSTRAINT zakonczona_ma_powod CHECK ((stan = 'zakonczona') = (koniec_powod IS NOT NULL)),
  CONSTRAINT do_podpisu_z_decyzja CHECK (stan NOT IN ('do_podpisu', 'zawarta') OR do_podpisu_decyzja_id IS NOT NULL),
  CONSTRAINT zawarta_ma_dokument_i_chwile CHECK (
    (zawarto IS NULL) = (dokument_id IS NULL)
    AND (stan <> 'zawarta' OR zawarto IS NOT NULL)
  ),
  CONSTRAINT oswiadczenie_przy_okazjonalnym_i_instytucjonalnym CHECK (
    oswiadczenie_dokument_id IS NULL OR rodzaj <> 'zwykly'
  ),
  CONSTRAINT zgloszenie_us_przy_okazjonalnym CHECK (
    (zgloszono_us IS NULL) = (zgloszenie_us_dokument_id IS NULL)
    AND (zgloszono_us IS NULL OR rodzaj = 'okazjonalny')
  )
) STRICT;

-- Opłaty obok czynszu i to, kto ma umowę z dostawcą. Od tego zależy, czy wynajmujący może pobierać
-- opłatę od najemcy. Opłaty dla wspólnoty albo spółdzielni są składnikiem czynszu.
CREATE TABLE najem_umowy_oplaty (
  -- Klucz nadany przez bazę jest dodatni; zob. najem_umowy_oplaty_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  najem_id TEXT NOT NULL REFERENCES najem_umowy_najmu (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('energia', 'gaz', 'woda', 'scieki', 'odpady', 'nieczystosci_ciekle', 'cieplo', 'internet', 'inna')),
  -- Np. „zaliczka 150 zł miesięcznie, rozliczenie roczne według licznika”.
  sposob TEXT NOT NULL CHECK (sposob <> ''),
  umowa_z_dostawca_ma TEXT NOT NULL CHECK (umowa_z_dostawca_ma IN ('najemca', 'wynajmujacy'))
) STRICT;

-- Osoby, które zamieszkują z najemcą, z datą zamieszkania i wyprowadzki: pełnoletnie odpowiadają
-- solidarnie za czynsz za czas zamieszkiwania (art. 688¹ KC).
CREATE TABLE najem_umowy_zamieszkujacy (
  -- Klucz nadany przez bazę jest dodatni; zob. najem_umowy_zamieszkujacy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  najem_id TEXT NOT NULL REFERENCES najem_umowy_najmu (id),
  uczestnik_id TEXT NOT NULL REFERENCES sprawa_uczestnicy_wpisy (id),
  od TEXT NOT NULL,
  do TEXT,

  CONSTRAINT wyprowadzka_nie_przed_zamieszkaniem CHECK (do IS NULL OR do >= od)
) STRICT;

-- Lokal, w którym najemca okazjonalny zamieszka w razie egzekucji, ze zgodą jego właściciela
-- (art. 19a ust. 2 pkt 2–3 UOPL). Gdy najemca straci tę możliwość, w 21 dni wskazuje nowy lokal
-- (art. 19a ust. 3 UOPL); termin trzyma rdzeń. Bieżący lokal nie ma niemozliwy_od.
CREATE TABLE najem_umowy_lokale_zastepcze (
  -- Klucz nadany przez bazę jest dodatni; zob. najem_umowy_lokale_zastepcze_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  najem_id TEXT NOT NULL REFERENCES najem_umowy_najmu (id),
  adres TEXT NOT NULL CHECK (adres <> ''),
  -- Oświadczenie właściciela tamtego lokalu, na żądanie wynajmującego z podpisem notarialnie
  -- poświadczonym.
  zgoda_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  wskazano TEXT NOT NULL,
  -- Dzień, w którym najemca dowiedział się, że nie może tam zamieszkać.
  niemozliwy_od TEXT
) STRICT;

-- Aneksy umowy najmu: projekt, zgoda człowieka i podpisany egzemplarz. Opiekun nie stosuje
-- ustalenia stron, dopóki aneks nie obowiązuje. Decyzja wskazuje aneks, więc ma UUID.
CREATE TABLE najem_umowy_aneksy (
  id TEXT PRIMARY KEY,
  najem_id TEXT NOT NULL REFERENCES najem_umowy_najmu (id),
  opis TEXT NOT NULL CHECK (opis <> ''),
  projekt_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  obowiazuje_od TEXT,

  CONSTRAINT obowiazuje_z_podpisanym_dokumentem CHECK ((obowiazuje_od IS NULL) = (dokument_id IS NULL))
) STRICT;

-- Koniec najmu: sposób, przyczyna, pismo i jego doręczenie, dzień końca najmu, żądanie opróżnienia
-- lokalu przy najmie okazjonalnym i instytucjonalnym i dzień opróżnienia, od którego biegnie
-- miesiąc na zwrot kaucji. Decyzja przy śmierci najemcy wskazuje ten wiersz, więc ma UUID.
CREATE TABLE najem_umowy_konce (
  id TEXT PRIMARY KEY,
  najem_id TEXT NOT NULL REFERENCES najem_umowy_najmu (id),
  sposob TEXT NOT NULL CHECK (sposob IN ('uplyw_czasu', 'wypowiedzenie_wynajmujacego', 'wypowiedzenie_najemcy', 'porozumienie', 'smierc_najemcy')),
  przyczyna TEXT,
  -- Pismo wynajmującego, które wysyła biuro, albo pismo najemcy, które wpłynęło.
  wysylka_id TEXT REFERENCES sprawa_korespondencja_wysylki (id),
  wplyw_id TEXT REFERENCES sprawa_korespondencja_wplywy (id),
  doreczono TEXT,
  koniec_najmu TEXT,
  -- Przy śmierci najemcy zatwierdzona decyzja inna: kto wstąpił w najem albo że najem wygasł
  -- (art. 691 KC).
  decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),
  zadanie_wysylka_id TEXT REFERENCES sprawa_korespondencja_wysylki (id),
  zadanie_doreczono TEXT,
  zadanie_termin TEXT,
  oproznienie TEXT,

  -- Art. 11 ust. 1 UOPL: wypowiedzenie przez właściciela na piśmie i z przyczyną.
  CONSTRAINT wypowiedzenie_wynajmujacego_na_pismie_z_przyczyna CHECK (
    sposob <> 'wypowiedzenie_wynajmujacego' OR (wysylka_id IS NOT NULL AND przyczyna IS NOT NULL)
  ),
  CONSTRAINT wypowiedzenie_najemcy_z_pisma CHECK (sposob <> 'wypowiedzenie_najemcy' OR wplyw_id IS NOT NULL),
  CONSTRAINT doreczenie_pisma CHECK (doreczono IS NULL OR wysylka_id IS NOT NULL OR wplyw_id IS NOT NULL),
  -- Dzień końca jest znany od razu, poza śmiercią najemcy: tam wynika z decyzji.
  CONSTRAINT koniec_najmu_znany CHECK (sposob = 'smierc_najemcy' OR koniec_najmu IS NOT NULL),
  CONSTRAINT smierc_najemcy_z_decyzja CHECK (
    sposob <> 'smierc_najemcy' OR koniec_najmu IS NULL OR decyzja_id IS NOT NULL
  ),
  CONSTRAINT zadanie_termin_przy_doreczeniu CHECK (
    (zadanie_doreczono IS NULL) = (zadanie_termin IS NULL)
    AND (zadanie_doreczono IS NULL OR zadanie_wysylka_id IS NOT NULL)
  )
) STRICT;

-- Warunki zarządzania -------------------------------------------------------------------------------

-- Pierwsza wersja obowiązuje od zawarcia umowy o zarządzanie, a każda następna wskazuje aneks tej
-- umowy i obowiązuje od jego chwili, później niż poprzednia. Porównanie z pustą wartością daje
-- NULL, więc coalesce zamienia je na odmowę.
CREATE TRIGGER najem_umowy_warunki_zarzadzania_z_zawarcia_albo_aneksu
BEFORE INSERT ON najem_umowy_zarzadzanie
WHEN NOT coalesce(
  CASE
    WHEN EXISTS (
      SELECT 1
      FROM najem_umowy_zarzadzanie
      WHERE umowa_id = NEW.umowa_id
    )
    THEN EXISTS (
      SELECT 1
      FROM sprawa_umowy_zdarzenia AS aneks
      JOIN sprawa_umowy_wpisy AS umowa ON umowa.id = aneks.umowa_id
      WHERE aneks.id = NEW.aneks_zdarzenie_id
        AND aneks.umowa_id = NEW.umowa_id
        AND aneks.rodzaj = 'aneks'
        AND aneks.chwila = NEW.od
        AND umowa.stan = 'zawarta'
    )
    AND NEW.od > (
      SELECT max(od)
      FROM najem_umowy_zarzadzanie
      WHERE umowa_id = NEW.umowa_id
    )
    ELSE NEW.aneks_zdarzenie_id IS NULL
    AND EXISTS (
      SELECT 1
      FROM sprawa_umowy_wpisy
      WHERE id = NEW.umowa_id
        AND stan = 'zawarta'
        AND zawarto = NEW.od
    )
  END,
  0
)
BEGIN
  SELECT RAISE(ABORT, 'pierwsze warunki zarządzania obowiązują od zawarcia umowy, a kolejne wskazują jej aneks i obowiązują od jego chwili');
END;

CREATE TRIGGER najem_umowy_zarzadzanie_bez_zmian
BEFORE UPDATE ON najem_umowy_zarzadzanie
BEGIN
  SELECT RAISE(ABORT, 'warunków zarządzania się nie zmienia; nowe warunki to nowa wersja z aneksem');
END;

-- Umowa najmu --------------------------------------------------------------------------------------

-- Reguły podpisu i zawarcia sprawdzają zmianę stanu, więc umowa powstaje jako projekt bez
-- decyzji, podpisu i zgłoszeń.
CREATE TRIGGER najem_umowy_najmu_powstaje_jako_projekt
BEFORE INSERT ON najem_umowy_najmu
WHEN NEW.stan <> 'projekt'
  OR NEW.do_podpisu_decyzja_id IS NOT NULL
  OR NEW.zawarto IS NOT NULL
  OR NEW.swiadectwo_przekazano IS NOT NULL
  OR NEW.zgloszono_us IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'umowa najmu powstaje jako projekt, bez decyzji, podpisu i zgłoszeń');
END;

CREATE TRIGGER najem_umowy_najmu_przejscia_stanu
BEFORE UPDATE OF stan ON najem_umowy_najmu
WHEN NEW.stan IS NOT OLD.stan
  AND NOT (
    (OLD.stan = 'projekt' AND NEW.stan IN ('do_podpisu', 'zakonczona'))
    OR (OLD.stan = 'do_podpisu' AND NEW.stan IN ('zawarta', 'zakonczona'))
    OR (OLD.stan = 'zawarta' AND NEW.stan = 'zakonczona')
  )
BEGIN
  SELECT RAISE(ABORT, 'umowa najmu przechodzi: projekt, do podpisu, zawarta, zakończona; z podpisu nie wraca do projektu');
END;

-- Projekt trafia do stron za zgodą człowieka, która wskazuje tę umowę, i nie przy stojącym
-- znaczniku wątpliwości Kontrolera.
CREATE TRIGGER najem_umowy_najmu_do_podpisu_za_zgoda
BEFORE UPDATE OF stan ON najem_umowy_najmu
WHEN NEW.stan = 'do_podpisu'
  AND OLD.stan = 'projekt'
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS decyzja
    WHERE decyzja.id = NEW.do_podpisu_decyzja_id
      AND decyzja.rodzaj = 'inna'
      AND decyzja.dotyczy_tabela = 'najem_umowy_najmu'
      AND decyzja.dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'umowa najmu idzie do podpisu za zatwierdzoną decyzją inna, która ją wskazuje');
END;

CREATE TRIGGER najem_umowy_najmu_watpliwosc_blokuje_podpis
BEFORE UPDATE OF stan ON najem_umowy_najmu
WHEN NEW.stan = 'do_podpisu'
  AND OLD.stan = 'projekt'
  AND EXISTS (
    SELECT 1
    FROM sprawa_metryka_watpliwosci
    WHERE zdjecie_decyzja_id IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'dopóki stoi znacznik wątpliwości, umowa najmu nie idzie do podpisu');
END;

-- Umowa ma strony: co najmniej jednego wynajmującego i jednego najemcę. Art. 19a ust. 1 UOPL:
-- przy najmie okazjonalnym wynajmujący jest osobą fizyczną. Art. 19f ust. 1 UOPL: przy
-- instytucjonalnym prowadzi działalność, więc ma NIP i wpis w KRS albo w CEIDG. Że osoba nie
-- prowadzi działalności w zakresie wynajmowania, ocenia człowiek.
CREATE TRIGGER najem_umowy_najmu_strony_wedlug_rodzaju
BEFORE UPDATE OF stan ON najem_umowy_najmu
WHEN NEW.stan = 'do_podpisu'
  AND OLD.stan = 'projekt'
  AND (
    NOT EXISTS (
      SELECT 1
      FROM sprawa_uczestnicy_role
      WHERE rola = 'wynajmujacy'
        AND zakonczono IS NULL
    )
    OR NOT EXISTS (
      SELECT 1
      FROM sprawa_uczestnicy_role
      WHERE rola = 'najemca'
        AND zakonczono IS NULL
    )
    OR EXISTS (
      SELECT 1
      FROM sprawa_uczestnicy_role AS rola
      JOIN sprawa_uczestnicy_wpisy AS wynajmujacy ON wynajmujacy.id = rola.uczestnik_id
      WHERE rola.rola = 'wynajmujacy'
        AND rola.zakonczono IS NULL
        AND (
          (NEW.rodzaj = 'okazjonalny' AND wynajmujacy.rodzaj_podmiotu <> 'osoba_fizyczna')
          OR (
            NEW.rodzaj = 'instytucjonalny'
            AND (wynajmujacy.nip IS NULL OR (wynajmujacy.krs IS NULL AND wynajmujacy.ceidg = 0))
          )
        )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'umowa najmu ma wynajmującego i najemcę; przy okazjonalnym wynajmujący jest osobą fizyczną, a przy instytucjonalnym ma NIP i wpis w KRS albo CEIDG');
END;

-- Art. 19a ust. 2 i art. 19f ust. 3 UOPL: biuro nie przekazuje do podpisu umowy najmu okazjonalnego
-- ani instytucjonalnego bez oświadczenia najemcy w formie aktu notarialnego, a okazjonalnego także
-- bez bieżącego lokalu zastępczego ze zgodą jego właściciela.
CREATE TRIGGER najem_umowy_najmu_do_podpisu_z_zalacznikami
BEFORE UPDATE OF stan ON najem_umowy_najmu
WHEN NEW.stan = 'do_podpisu'
  AND OLD.stan = 'projekt'
  AND NEW.rodzaj <> 'zwykly'
  AND (
    NOT EXISTS (
      SELECT 1
      FROM sprawa_dokumenty_wpisy
      WHERE id = NEW.oswiadczenie_dokument_id
        AND rodzaj = 'akt_notarialny'
        AND stan IN ('otrzymany', 'sprawdzony')
    )
    OR (
      NEW.rodzaj = 'okazjonalny'
      AND NOT EXISTS (
        SELECT 1
        FROM najem_umowy_lokale_zastepcze
        WHERE najem_id = NEW.id
          AND zgoda_dokument_id IS NOT NULL
          AND niemozliwy_od IS NULL
      )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'umowa najmu okazjonalnego i instytucjonalnego idzie do podpisu z oświadczeniem najemcy w formie aktu, a okazjonalnego także z lokalem zastępczym i zgodą jego właściciela');
END;

CREATE TRIGGER najem_umowy_najmu_zawarta_z_dokumentem
BEFORE UPDATE OF stan ON najem_umowy_najmu
WHEN NEW.stan = 'zawarta'
  AND OLD.stan = 'do_podpisu'
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_dokumenty_wpisy
    WHERE id = NEW.dokument_id
      AND rodzaj = 'umowa'
      AND stan IN ('otrzymany', 'sprawdzony')
  )
BEGIN
  SELECT RAISE(ABORT, 'zawarta umowa najmu wskazuje otrzymany podpisany egzemplarz');
END;

-- Po wyjściu z projektu treść umowy jest tym, na co zgodził się człowiek i co podpisały strony;
-- zmienia ją aneks. Zmienia się stan, a dane zawarcia, koniec, przekazanie świadectwa
-- i zgłoszenie do urzędu wpisuje się raz.
CREATE TRIGGER najem_umowy_najmu_tresc_poza_projektem_niezmienna
BEFORE UPDATE ON najem_umowy_najmu
WHEN OLD.stan <> 'projekt'
  AND (
    NEW.id IS NOT OLD.id
    OR NEW.rodzaj IS NOT OLD.rodzaj
    OR NEW.nieruchomosc_id IS NOT OLD.nieruchomosc_id
    OR NEW.poczatek IS NOT OLD.poczatek
    OR NEW.koniec IS NOT OLD.koniec
    OR NEW.forma IS NOT OLD.forma
    OR NEW.projekt_dokument_id IS NOT OLD.projekt_dokument_id
    OR NEW.do_podpisu_decyzja_id IS NOT OLD.do_podpisu_decyzja_id
    OR NEW.czynsz IS NOT OLD.czynsz
    OR NEW.termin_platnosci IS NOT OLD.termin_platnosci
    OR NEW.kaucja IS NOT OLD.kaucja
    OR NEW.zwierzeta IS NOT OLD.zwierzeta
    OR NEW.podnajem IS NOT OLD.podnajem
    OR NEW.zmiany_w_lokalu IS NOT OLD.zmiany_w_lokalu
    OR NEW.naprawy_podzial IS NOT OLD.naprawy_podzial
    OR NEW.wypowiedzenie_przypadki IS NOT OLD.wypowiedzenie_przypadki
    OR NEW.oswiadczenie_dokument_id IS NOT OLD.oswiadczenie_dokument_id
    OR (OLD.dokument_id IS NOT NULL AND NEW.dokument_id IS NOT OLD.dokument_id)
    OR (OLD.zawarto IS NOT NULL AND NEW.zawarto IS NOT OLD.zawarto)
    OR (OLD.koniec_powod IS NOT NULL AND NEW.koniec_powod IS NOT OLD.koniec_powod)
    OR (OLD.swiadectwo_przekazano IS NOT NULL AND NEW.swiadectwo_przekazano IS NOT OLD.swiadectwo_przekazano)
    OR (OLD.zgloszono_us IS NOT NULL AND NEW.zgloszono_us IS NOT OLD.zgloszono_us)
    OR (OLD.zgloszenie_us_dokument_id IS NOT NULL AND NEW.zgloszenie_us_dokument_id IS NOT OLD.zgloszenie_us_dokument_id)
  )
BEGIN
  SELECT RAISE(ABORT, 'treść umowy najmu zmienia się tylko w projekcie; potem zmienia ją aneks, a dane zawarcia i zgłoszenia wpisuje się raz');
END;

-- Etapy, które zależą od umowy najmu ----------------------------------------------------------------

-- Oświadczenie najemcy jest załącznikiem najmu okazjonalnego i instytucjonalnego, więc pominąć
-- ten etap wolno tylko przy bieżącej umowie najmu zwykłego.
CREATE TRIGGER najem_umowy_pominiecie_oswiadczenia_przy_zwyklym
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap = 'podpisanie_umowy'
  AND (
    SELECT etap = 'projekt_umowy'
    FROM sprawa_metryka_etapy
    ORDER BY id DESC
    LIMIT 1
  )
  AND (
    NOT EXISTS (
      SELECT 1
      FROM najem_umowy_najmu
      WHERE stan <> 'zakonczona'
        AND rodzaj = 'zwykly'
    )
    OR EXISTS (
      SELECT 1
      FROM najem_umowy_najmu
      WHERE stan <> 'zakonczona'
        AND rodzaj <> 'zwykly'
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'etap oświadczenia najemcy pomija się tylko przy najmie zwykłym');
END;

-- Art. 19b ust. 1 UOPL: właściciel zgłasza najem okazjonalny do urzędu skarbowego w 14 dni od
-- rozpoczęcia najmu, więc przy najmie okazjonalnym sprawa idzie z wydania lokalu przez zgłoszenie.
CREATE TRIGGER najem_umowy_okazjonalny_przez_zgloszenie
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap IN ('trwanie_najmu', 'po_najmie', 'zamknieta')
  AND (
    SELECT etap = 'wydanie_lokalu'
    FROM sprawa_metryka_etapy
    ORDER BY id DESC
    LIMIT 1
  )
  AND EXISTS (
    SELECT 1
    FROM najem_umowy_najmu
    WHERE stan <> 'zakonczona'
      AND rodzaj = 'okazjonalny'
  )
BEGIN
  SELECT RAISE(ABORT, 'przy najmie okazjonalnym sprawa idzie z wydania lokalu przez zgłoszenie najmu');
END;

-- Opłaty -------------------------------------------------------------------------------------------

-- Opłaty są postanowieniem umowy, więc dopisuje się je do projektu i się ich nie zmienia.
CREATE TRIGGER najem_umowy_oplata_w_projekcie
BEFORE INSERT ON najem_umowy_oplaty
WHEN (
  SELECT stan <> 'projekt'
  FROM najem_umowy_najmu
  WHERE id = NEW.najem_id
)
BEGIN
  SELECT RAISE(ABORT, 'opłatę dopisuje się do projektu umowy najmu');
END;

-- Art. 2 ust. 1 pkt 8 i art. 9 ust. 5–6 UOPL: przy najmie zwykłym wynajmujący pobiera obok czynszu
-- tylko opłaty niezależne od właściciela (energia, gaz, woda, ścieki, odpady, nieczystości ciekłe;
-- ciepło jako energię), gdy umowę z dostawcą ma on, a nie najemca.
CREATE TRIGGER najem_umowy_oplata_przy_zwyklym_niezalezna_od_wlasciciela
BEFORE INSERT ON najem_umowy_oplaty
WHEN NEW.umowa_z_dostawca_ma = 'wynajmujacy'
  AND NEW.rodzaj NOT IN ('energia', 'gaz', 'woda', 'scieki', 'odpady', 'nieczystosci_ciekle', 'cieplo')
  AND (
    SELECT rodzaj = 'zwykly'
    FROM najem_umowy_najmu
    WHERE id = NEW.najem_id
  )
BEGIN
  SELECT RAISE(ABORT, 'przy najmie zwykłym wynajmujący pobiera obok czynszu tylko opłaty niezależne od właściciela');
END;

CREATE TRIGGER najem_umowy_oplaty_bez_zmian
BEFORE UPDATE ON najem_umowy_oplaty
BEGIN
  SELECT RAISE(ABORT, 'opłaty umowy najmu się nie zmienia; zmienia ją aneks');
END;

-- Zamieszkujący ------------------------------------------------------------------------------------

-- Od dat zależy odpowiedzialność za czynsz, więc zmienia się tylko raz wpisana data wyprowadzki.
CREATE TRIGGER najem_umowy_zamieszkujacy_zmienia_tylko_wyprowadzke
BEFORE UPDATE ON najem_umowy_zamieszkujacy
WHEN OLD.do IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.najem_id IS NOT OLD.najem_id
  OR NEW.uczestnik_id IS NOT OLD.uczestnik_id
  OR NEW.od IS NOT OLD.od
BEGIN
  SELECT RAISE(ABORT, 'osoby zamieszkującej się nie zmienia; wpisuje się raz datę jej wyprowadzki');
END;

-- Lokal zastępczy ----------------------------------------------------------------------------------

CREATE TRIGGER najem_umowy_lokal_zastepczy_przy_okazjonalnym
BEFORE INSERT ON najem_umowy_lokale_zastepcze
WHEN (
  SELECT rodzaj <> 'okazjonalny'
  FROM najem_umowy_najmu
  WHERE id = NEW.najem_id
)
BEGIN
  SELECT RAISE(ABORT, 'lokal zastępczy wskazuje się tylko przy najmie okazjonalnym');
END;

-- Zgoda przychodzi po wskazaniu lokalu, a utrata możliwości zamieszkania jeszcze później.
CREATE TRIGGER najem_umowy_lokal_zastepczy_powstaje_bez_zgody
BEFORE INSERT ON najem_umowy_lokale_zastepcze
WHEN NEW.zgoda_dokument_id IS NOT NULL
  OR NEW.niemozliwy_od IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'lokal zastępczy powstaje bez zgody i bez utraty; dopisuje się je później');
END;

CREATE TRIGGER najem_umowy_zgoda_na_lokal_zastepczy_z_plikiem
BEFORE UPDATE OF zgoda_dokument_id ON najem_umowy_lokale_zastepcze
WHEN NEW.zgoda_dokument_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_dokumenty_wpisy
    WHERE id = NEW.zgoda_dokument_id
      AND rodzaj = 'oswiadczenie'
      AND stan IN ('otrzymany', 'sprawdzony')
  )
BEGIN
  SELECT RAISE(ABORT, 'zgoda na lokal zastępczy wskazuje otrzymane oświadczenie właściciela tamtego lokalu');
END;

CREATE TRIGGER najem_umowy_lokal_zastepczy_bez_zmian
BEFORE UPDATE ON najem_umowy_lokale_zastepcze
WHEN NEW.id IS NOT OLD.id
  OR NEW.najem_id IS NOT OLD.najem_id
  OR NEW.adres IS NOT OLD.adres
  OR NEW.wskazano IS NOT OLD.wskazano
  OR (OLD.zgoda_dokument_id IS NOT NULL AND NEW.zgoda_dokument_id IS NOT OLD.zgoda_dokument_id)
  OR (OLD.niemozliwy_od IS NOT NULL AND NEW.niemozliwy_od IS NOT OLD.niemozliwy_od)
BEGIN
  SELECT RAISE(ABORT, 'lokalu zastępczego się nie zmienia; zgodę i utratę wpisuje się raz, a nowy lokal to nowy wiersz');
END;

-- Aneksy -------------------------------------------------------------------------------------------

-- Aneks zmienia zawartą umowę, a zgodę i podpis dopisuje się po kolei.
CREATE TRIGGER najem_umowy_aneks_zawartej_umowy
BEFORE INSERT ON najem_umowy_aneksy
WHEN NEW.decyzja_id IS NOT NULL
  OR NEW.dokument_id IS NOT NULL
  OR NOT EXISTS (
    SELECT 1
    FROM najem_umowy_najmu
    WHERE id = NEW.najem_id
      AND stan = 'zawarta'
  )
BEGIN
  SELECT RAISE(ABORT, 'aneks powstaje przy zawartej umowie najmu, bez zgody i podpisu');
END;

CREATE TRIGGER najem_umowy_aneks_za_zgoda
BEFORE UPDATE OF decyzja_id ON najem_umowy_aneksy
WHEN NEW.decyzja_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS decyzja
    WHERE decyzja.id = NEW.decyzja_id
      AND decyzja.rodzaj = 'inna'
      AND decyzja.dotyczy_tabela = 'najem_umowy_aneksy'
      AND decyzja.dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'aneks idzie do stron za zatwierdzoną decyzją inna, która go wskazuje');
END;

-- Aneks obowiązuje z podpisanym egzemplarzem, a zgoda wciąż obowiązuje (nie została zastąpiona).
CREATE TRIGGER najem_umowy_aneks_obowiazuje_z_podpisem
BEFORE UPDATE OF dokument_id, obowiazuje_od ON najem_umowy_aneksy
WHEN NEW.dokument_id IS NOT NULL
  AND (
    NOT EXISTS (
      SELECT 1
      FROM sprawa_dokumenty_wpisy
      WHERE id = NEW.dokument_id
        AND rodzaj = 'umowa'
        AND stan IN ('otrzymany', 'sprawdzony')
    )
    OR NOT EXISTS (
      SELECT 1
      FROM sprawa_decyzje_zatwierdzone
      WHERE id = NEW.decyzja_id
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'aneks obowiązuje z podpisanym egzemplarzem i obowiązującą zgodą człowieka');
END;

-- Zgoda dotyczy treści aneksu, więc po niej treść stoi; zgodę i podpis wpisuje się raz.
CREATE TRIGGER najem_umowy_aneks_po_zgodzie_niezmienny
BEFORE UPDATE ON najem_umowy_aneksy
WHEN NEW.id IS NOT OLD.id
  OR NEW.najem_id IS NOT OLD.najem_id
  OR (OLD.decyzja_id IS NOT NULL AND (
    NEW.decyzja_id IS NOT OLD.decyzja_id
    OR NEW.opis IS NOT OLD.opis
    OR NEW.projekt_dokument_id IS NOT OLD.projekt_dokument_id
  ))
  OR (OLD.dokument_id IS NOT NULL AND (
    NEW.dokument_id IS NOT OLD.dokument_id
    OR NEW.obowiazuje_od IS NOT OLD.obowiazuje_od
  ))
BEGIN
  SELECT RAISE(ABORT, 'treść aneksu stoi od zgody człowieka, a zgodę i podpis wpisuje się raz');
END;

-- Koniec najmu -------------------------------------------------------------------------------------

-- Doręczenie, decyzję, żądanie opróżnienia i opróżnienie dopisuje się później.
CREATE TRIGGER najem_umowy_koniec_powstaje_bez_dalszych_krokow
BEFORE INSERT ON najem_umowy_konce
WHEN NEW.doreczono IS NOT NULL
  OR NEW.decyzja_id IS NOT NULL
  OR NEW.zadanie_wysylka_id IS NOT NULL
  OR NEW.oproznienie IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'koniec najmu powstaje bez doręczenia, decyzji, żądania i opróżnienia');
END;

-- Wypowiedzenie i żądanie opróżnienia wychodzą za zgodą człowieka, więc koniec wskazuje pismo,
-- które jeszcze nie wyszło; reguła wysyłki niżej puszcza je tylko przez czeka_na_zgode.
CREATE TRIGGER najem_umowy_koniec_wskazuje_niewyslane_pismo
BEFORE INSERT ON najem_umowy_konce
WHEN NEW.wysylka_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_korespondencja_wysylki
    WHERE id = NEW.wysylka_id
      AND stan IN ('projekt', 'czeka_na_zgode')
  )
BEGIN
  SELECT RAISE(ABORT, 'koniec najmu wskazuje pismo, które jeszcze nie wyszło; wychodzi ono za zgodą człowieka');
END;

CREATE TRIGGER najem_umowy_zadanie_wskazuje_niewyslane_pismo
BEFORE UPDATE OF zadanie_wysylka_id ON najem_umowy_konce
WHEN NEW.zadanie_wysylka_id IS NOT NULL
  AND NEW.zadanie_wysylka_id IS NOT OLD.zadanie_wysylka_id
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_korespondencja_wysylki
    WHERE id = NEW.zadanie_wysylka_id
      AND stan IN ('projekt', 'czeka_na_zgode')
  )
BEGIN
  SELECT RAISE(ABORT, 'koniec najmu wskazuje pismo, które jeszcze nie wyszło; wychodzi ono za zgodą człowieka');
END;

CREATE TRIGGER najem_umowy_pismo_konca_najmu_za_zgoda
BEFORE UPDATE OF stan ON sprawa_korespondencja_wysylki
WHEN NEW.stan = 'wyslana'
  AND OLD.stan = 'projekt'
  AND EXISTS (
    SELECT 1
    FROM najem_umowy_konce
    WHERE NEW.id IN (wysylka_id, zadanie_wysylka_id)
  )
BEGIN
  SELECT RAISE(ABORT, 'wypowiedzenie i żądanie opróżnienia lokalu wychodzą przez czeka_na_zgode, za zgodą człowieka');
END;

-- Art. 691 KC: kto wstąpił w najem po śmierci najemcy, ocenia człowiek; dzień końca najmu przychodzi
-- z jego decyzją, która wskazuje ten wiersz.
CREATE TRIGGER najem_umowy_smierc_najemcy_z_decyzja
BEFORE UPDATE OF decyzja_id ON najem_umowy_konce
WHEN NEW.decyzja_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS decyzja
    WHERE decyzja.id = NEW.decyzja_id
      AND decyzja.rodzaj = 'inna'
      AND decyzja.dotyczy_tabela = 'najem_umowy_konce'
      AND decyzja.dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'koniec najmu wskazuje zatwierdzoną decyzję inna, która go wskazuje');
END;

-- Art. 19d ust. 2–3 i art. 19i ust. 2–3 UOPL: żądanie opróżnienia jest tylko przy najmie
-- okazjonalnym i instytucjonalnym i wyznacza termin co najmniej 7 albo 14 dni od doręczenia.
CREATE TRIGGER najem_umowy_zadanie_oproznienia_z_terminem
BEFORE UPDATE OF zadanie_wysylka_id, zadanie_doreczono, zadanie_termin ON najem_umowy_konce
WHEN NEW.zadanie_wysylka_id IS NOT NULL
  AND NOT coalesce(
    (
      SELECT
        CASE rodzaj
          WHEN 'okazjonalny' THEN NEW.zadanie_termin IS NULL OR NEW.zadanie_termin >= date(NEW.zadanie_doreczono, '+7 days')
          WHEN 'instytucjonalny' THEN NEW.zadanie_termin IS NULL OR NEW.zadanie_termin >= date(NEW.zadanie_doreczono, '+14 days')
          ELSE 0
        END
      FROM najem_umowy_najmu
      WHERE id = NEW.najem_id
    ),
    0
  )
BEGIN
  SELECT RAISE(ABORT, 'żądanie opróżnienia jest przy najmie okazjonalnym (termin co najmniej 7 dni od doręczenia) i instytucjonalnym (co najmniej 14 dni)');
END;

-- Sposób, przyczyna i pismo są podstawą końca najmu; pozostałe kroki wpisuje się raz.
CREATE TRIGGER najem_umowy_koniec_bez_zmian
BEFORE UPDATE ON najem_umowy_konce
WHEN NEW.id IS NOT OLD.id
  OR NEW.najem_id IS NOT OLD.najem_id
  OR NEW.sposob IS NOT OLD.sposob
  OR NEW.przyczyna IS NOT OLD.przyczyna
  OR NEW.wysylka_id IS NOT OLD.wysylka_id
  OR NEW.wplyw_id IS NOT OLD.wplyw_id
  OR (OLD.doreczono IS NOT NULL AND NEW.doreczono IS NOT OLD.doreczono)
  OR (OLD.koniec_najmu IS NOT NULL AND NEW.koniec_najmu IS NOT OLD.koniec_najmu)
  OR (OLD.decyzja_id IS NOT NULL AND NEW.decyzja_id IS NOT OLD.decyzja_id)
  OR (OLD.zadanie_wysylka_id IS NOT NULL AND NEW.zadanie_wysylka_id IS NOT OLD.zadanie_wysylka_id)
  OR (OLD.zadanie_doreczono IS NOT NULL AND (
    NEW.zadanie_doreczono IS NOT OLD.zadanie_doreczono
    OR NEW.zadanie_termin IS NOT OLD.zadanie_termin
  ))
  OR (OLD.oproznienie IS NOT NULL AND NEW.oproznienie IS NOT OLD.oproznienie)
BEGIN
  SELECT RAISE(ABORT, 'końca najmu się nie zmienia; doręczenie, decyzję, żądanie i opróżnienie wpisuje się raz');
END;

-- Bez usuwania i zastępowania ---------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER najem_umowy_zarzadzanie_bez_zastepowania
BEFORE INSERT ON najem_umowy_zarzadzanie
WHEN EXISTS (
  SELECT 1
  FROM najem_umowy_zarzadzanie
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER najem_umowy_zarzadzanie_bez_usuwania
BEFORE DELETE ON najem_umowy_zarzadzanie
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER najem_umowy_najmu_bez_zastepowania
BEFORE INSERT ON najem_umowy_najmu
WHEN EXISTS (
  SELECT 1
  FROM najem_umowy_najmu
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER najem_umowy_najmu_bez_usuwania
BEFORE DELETE ON najem_umowy_najmu
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER najem_umowy_oplaty_bez_zastepowania
BEFORE INSERT ON najem_umowy_oplaty
WHEN EXISTS (
  SELECT 1
  FROM najem_umowy_oplaty
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER najem_umowy_oplaty_bez_usuwania
BEFORE DELETE ON najem_umowy_oplaty
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER najem_umowy_zamieszkujacy_bez_zastepowania
BEFORE INSERT ON najem_umowy_zamieszkujacy
WHEN EXISTS (
  SELECT 1
  FROM najem_umowy_zamieszkujacy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER najem_umowy_zamieszkujacy_bez_usuwania
BEFORE DELETE ON najem_umowy_zamieszkujacy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER najem_umowy_lokale_zastepcze_bez_zastepowania
BEFORE INSERT ON najem_umowy_lokale_zastepcze
WHEN EXISTS (
  SELECT 1
  FROM najem_umowy_lokale_zastepcze
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER najem_umowy_lokale_zastepcze_bez_usuwania
BEFORE DELETE ON najem_umowy_lokale_zastepcze
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER najem_umowy_aneksy_bez_zastepowania
BEFORE INSERT ON najem_umowy_aneksy
WHEN EXISTS (
  SELECT 1
  FROM najem_umowy_aneksy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER najem_umowy_aneksy_bez_usuwania
BEFORE DELETE ON najem_umowy_aneksy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER najem_umowy_konce_bez_zastepowania
BEFORE INSERT ON najem_umowy_konce
WHEN EXISTS (
  SELECT 1
  FROM najem_umowy_konce
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER najem_umowy_konce_bez_usuwania
BEFORE DELETE ON najem_umowy_konce
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
    ],
};

/// The values of column `konce.sposob`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KonceSposob {
    UplywCzasu,
    WypowiedzenieWynajmujacego,
    WypowiedzenieNajemcy,
    Porozumienie,
    SmiercNajemcy,
}

impl KonceSposob {
    /// Every value, in the order of the catalog.
    pub const ALL: &[KonceSposob] = &[
        KonceSposob::UplywCzasu,
        KonceSposob::WypowiedzenieWynajmujacego,
        KonceSposob::WypowiedzenieNajemcy,
        KonceSposob::Porozumienie,
        KonceSposob::SmiercNajemcy,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            KonceSposob::UplywCzasu => "uplyw_czasu",
            KonceSposob::WypowiedzenieWynajmujacego => "wypowiedzenie_wynajmujacego",
            KonceSposob::WypowiedzenieNajemcy => "wypowiedzenie_najemcy",
            KonceSposob::Porozumienie => "porozumienie",
            KonceSposob::SmiercNajemcy => "smierc_najemcy",
        }
    }
}

impl xmeta_room::FromSqlValue for KonceSposob {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "uplyw_czasu" => Ok(KonceSposob::UplywCzasu),
                    "wypowiedzenie_wynajmujacego" => {
                        Ok(KonceSposob::WypowiedzenieWynajmujacego)
                    }
                    "wypowiedzenie_najemcy" => Ok(KonceSposob::WypowiedzenieNajemcy),
                    "porozumienie" => Ok(KonceSposob::Porozumienie),
                    "smierc_najemcy" => Ok(KonceSposob::SmiercNajemcy),
                    other => Err(format!("{other} is no value of KonceSposob")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for KonceSposob {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `najmu.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NajmuRodzaj {
    Zwykly,
    Okazjonalny,
    Instytucjonalny,
}

impl NajmuRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[NajmuRodzaj] = &[
        NajmuRodzaj::Zwykly,
        NajmuRodzaj::Okazjonalny,
        NajmuRodzaj::Instytucjonalny,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            NajmuRodzaj::Zwykly => "zwykly",
            NajmuRodzaj::Okazjonalny => "okazjonalny",
            NajmuRodzaj::Instytucjonalny => "instytucjonalny",
        }
    }
}

impl xmeta_room::FromSqlValue for NajmuRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "zwykly" => Ok(NajmuRodzaj::Zwykly),
                    "okazjonalny" => Ok(NajmuRodzaj::Okazjonalny),
                    "instytucjonalny" => Ok(NajmuRodzaj::Instytucjonalny),
                    other => Err(format!("{other} is no value of NajmuRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for NajmuRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `najmu.forma`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NajmuForma {
    Pisemna,
    Elektroniczna,
    Dokumentowa,
}

impl NajmuForma {
    /// Every value, in the order of the catalog.
    pub const ALL: &[NajmuForma] = &[
        NajmuForma::Pisemna,
        NajmuForma::Elektroniczna,
        NajmuForma::Dokumentowa,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            NajmuForma::Pisemna => "pisemna",
            NajmuForma::Elektroniczna => "elektroniczna",
            NajmuForma::Dokumentowa => "dokumentowa",
        }
    }
}

impl xmeta_room::FromSqlValue for NajmuForma {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "pisemna" => Ok(NajmuForma::Pisemna),
                    "elektroniczna" => Ok(NajmuForma::Elektroniczna),
                    "dokumentowa" => Ok(NajmuForma::Dokumentowa),
                    other => Err(format!("{other} is no value of NajmuForma")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for NajmuForma {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `najmu.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NajmuStan {
    Projekt,
    DoPodpisu,
    Zawarta,
    Zakonczona,
}

impl NajmuStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[NajmuStan] = &[
        NajmuStan::Projekt,
        NajmuStan::DoPodpisu,
        NajmuStan::Zawarta,
        NajmuStan::Zakonczona,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            NajmuStan::Projekt => "projekt",
            NajmuStan::DoPodpisu => "do_podpisu",
            NajmuStan::Zawarta => "zawarta",
            NajmuStan::Zakonczona => "zakonczona",
        }
    }
}

impl xmeta_room::FromSqlValue for NajmuStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "projekt" => Ok(NajmuStan::Projekt),
                    "do_podpisu" => Ok(NajmuStan::DoPodpisu),
                    "zawarta" => Ok(NajmuStan::Zawarta),
                    "zakonczona" => Ok(NajmuStan::Zakonczona),
                    other => Err(format!("{other} is no value of NajmuStan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for NajmuStan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `oplaty.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OplatyRodzaj {
    Energia,
    Gaz,
    Woda,
    Scieki,
    Odpady,
    NieczystosciCiekle,
    Cieplo,
    Internet,
    Inna,
}

impl OplatyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[OplatyRodzaj] = &[
        OplatyRodzaj::Energia,
        OplatyRodzaj::Gaz,
        OplatyRodzaj::Woda,
        OplatyRodzaj::Scieki,
        OplatyRodzaj::Odpady,
        OplatyRodzaj::NieczystosciCiekle,
        OplatyRodzaj::Cieplo,
        OplatyRodzaj::Internet,
        OplatyRodzaj::Inna,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            OplatyRodzaj::Energia => "energia",
            OplatyRodzaj::Gaz => "gaz",
            OplatyRodzaj::Woda => "woda",
            OplatyRodzaj::Scieki => "scieki",
            OplatyRodzaj::Odpady => "odpady",
            OplatyRodzaj::NieczystosciCiekle => "nieczystosci_ciekle",
            OplatyRodzaj::Cieplo => "cieplo",
            OplatyRodzaj::Internet => "internet",
            OplatyRodzaj::Inna => "inna",
        }
    }
}

impl xmeta_room::FromSqlValue for OplatyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "energia" => Ok(OplatyRodzaj::Energia),
                    "gaz" => Ok(OplatyRodzaj::Gaz),
                    "woda" => Ok(OplatyRodzaj::Woda),
                    "scieki" => Ok(OplatyRodzaj::Scieki),
                    "odpady" => Ok(OplatyRodzaj::Odpady),
                    "nieczystosci_ciekle" => Ok(OplatyRodzaj::NieczystosciCiekle),
                    "cieplo" => Ok(OplatyRodzaj::Cieplo),
                    "internet" => Ok(OplatyRodzaj::Internet),
                    "inna" => Ok(OplatyRodzaj::Inna),
                    other => Err(format!("{other} is no value of OplatyRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for OplatyRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `oplaty.umowa_z_dostawca_ma`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OplatyUmowaZDostawcaMa {
    Najemca,
    Wynajmujacy,
}

impl OplatyUmowaZDostawcaMa {
    /// Every value, in the order of the catalog.
    pub const ALL: &[OplatyUmowaZDostawcaMa] = &[
        OplatyUmowaZDostawcaMa::Najemca,
        OplatyUmowaZDostawcaMa::Wynajmujacy,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            OplatyUmowaZDostawcaMa::Najemca => "najemca",
            OplatyUmowaZDostawcaMa::Wynajmujacy => "wynajmujacy",
        }
    }
}

impl xmeta_room::FromSqlValue for OplatyUmowaZDostawcaMa {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "najemca" => Ok(OplatyUmowaZDostawcaMa::Najemca),
                    "wynajmujacy" => Ok(OplatyUmowaZDostawcaMa::Wynajmujacy),
                    other => {
                        Err(format!("{other} is no value of OplatyUmowaZDostawcaMa"))
                    }
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for OplatyUmowaZDostawcaMa {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `zarzadzanie.przeplyw`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZarzadzaniePrzeplyw {
    PrzezBiuro,
    Wprost,
}

impl ZarzadzaniePrzeplyw {
    /// Every value, in the order of the catalog.
    pub const ALL: &[ZarzadzaniePrzeplyw] = &[
        ZarzadzaniePrzeplyw::PrzezBiuro,
        ZarzadzaniePrzeplyw::Wprost,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            ZarzadzaniePrzeplyw::PrzezBiuro => "przez_biuro",
            ZarzadzaniePrzeplyw::Wprost => "wprost",
        }
    }
}

impl xmeta_room::FromSqlValue for ZarzadzaniePrzeplyw {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "przez_biuro" => Ok(ZarzadzaniePrzeplyw::PrzezBiuro),
                    "wprost" => Ok(ZarzadzaniePrzeplyw::Wprost),
                    other => Err(format!("{other} is no value of ZarzadzaniePrzeplyw")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for ZarzadzaniePrzeplyw {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `aneksy_najmu`.
#[derive(Debug, Clone, PartialEq)]
pub struct AneksyNajmuRow {
    pub id: String,
    pub najem_id: String,
    pub opis: String,
    pub projekt_dokument_id: Option<String>,
    pub decyzja_id: Option<String>,
    pub dokument_id: Option<String>,
    pub obowiazuje_od: Option<String>,
}

impl AneksyNajmuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(AneksyNajmuRow {
            id: row.get("id")?,
            najem_id: row.get("najem_id")?,
            opis: row.get("opis")?,
            projekt_dokument_id: row.get("projekt_dokument_id")?,
            decyzja_id: row.get("decyzja_id")?,
            dokument_id: row.get("dokument_id")?,
            obowiazuje_od: row.get("obowiazuje_od")?,
        })
    }
}

/// Aneksy umów najmu; obowiązujące mają obowiazuje_od.
pub fn aneksy_najmu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<AneksyNajmuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM najem_umowy_aneksy
ORDER BY rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(AneksyNajmuRow::from_row).collect()
}

/// The parameters of `przygotuj_aneks_najmu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrzygotujAneksNajmuParams {
    pub id: String,
    pub najem_id: String,
    pub opis: String,
    pub projekt_dokument_id: Option<String>,
}

/// Projekt aneksu zawartej umowy najmu; identyfikator (UUID) nadaje aplikacja, bo wskazuje go
/// decyzja.
pub fn przygotuj_aneks_najmu(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzygotujAneksNajmuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO najem_umowy_aneksy (id, najem_id, opis, projekt_dokument_id)
VALUES (:id, :najem_id, :opis, :projekt_dokument_id)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("najem_id", xmeta_room::ToSqlValue::to_sql_value(&params.najem_id)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    (
                        "projekt_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.projekt_dokument_id),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `wskaz_zgode_na_aneks_najmu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WskazZgodeNaAneksNajmuParams {
    pub decyzja_id: String,
    pub id: String,
}

/// Zatwierdzona decyzja inna o przekazaniu aneksu stronom.
pub fn wskaz_zgode_na_aneks_najmu(
    room: &impl xmeta_room::MetaRoom,
    params: &WskazZgodeNaAneksNajmuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_umowy_aneksy
SET decyzja_id = :decyzja_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_podpisanie_aneksu_najmu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPodpisanieAneksuNajmuParams {
    pub dokument_id: String,
    pub obowiazuje_od: String,
    pub id: String,
}

/// Strony podpisały aneks; od tego dnia obowiązuje.
pub fn odnotuj_podpisanie_aneksu_najmu(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPodpisanieAneksuNajmuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_umowy_aneksy
SET dokument_id = :dokument_id,
  obowiazuje_od = :obowiazuje_od
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                    (
                        "obowiazuje_od",
                        xmeta_room::ToSqlValue::to_sql_value(&params.obowiazuje_od),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `konce_najmu`.
#[derive(Debug, Clone, PartialEq)]
pub struct KonceNajmuRow {
    pub id: String,
    pub najem_id: String,
    pub sposob: KonceSposob,
    pub przyczyna: Option<String>,
    pub wysylka_id: Option<String>,
    pub wplyw_id: Option<String>,
    pub doreczono: Option<String>,
    pub koniec_najmu: Option<String>,
    pub decyzja_id: Option<String>,
    pub zadanie_wysylka_id: Option<String>,
    pub zadanie_doreczono: Option<String>,
    pub zadanie_termin: Option<String>,
    pub oproznienie: Option<String>,
}

impl KonceNajmuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(KonceNajmuRow {
            id: row.get("id")?,
            najem_id: row.get("najem_id")?,
            sposob: row.get("sposob")?,
            przyczyna: row.get("przyczyna")?,
            wysylka_id: row.get("wysylka_id")?,
            wplyw_id: row.get("wplyw_id")?,
            doreczono: row.get("doreczono")?,
            koniec_najmu: row.get("koniec_najmu")?,
            decyzja_id: row.get("decyzja_id")?,
            zadanie_wysylka_id: row.get("zadanie_wysylka_id")?,
            zadanie_doreczono: row.get("zadanie_doreczono")?,
            zadanie_termin: row.get("zadanie_termin")?,
            oproznienie: row.get("oproznienie")?,
        })
    }
}

/// Koniec najmu: sposób, pismo, dzień końca, żądanie opróżnienia i opróżnienie.
pub fn konce_najmu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<KonceNajmuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM najem_umowy_konce
ORDER BY rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(KonceNajmuRow::from_row).collect()
}

/// The parameters of `wpisz_koniec_najmu`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszKoniecNajmuParams {
    pub id: String,
    pub najem_id: String,
    pub sposob: KonceSposob,
    pub przyczyna: Option<String>,
    pub wysylka_id: Option<String>,
    pub wplyw_id: Option<String>,
    pub koniec_najmu: Option<String>,
}

/// Jak najem się kończy: upływ czasu, wypowiedzenie (pismo wynajmującego jeszcze przed wysłaniem
/// albo pismo najemcy), porozumienie albo śmierć najemcy; identyfikator (UUID) nadaje aplikacja.
pub fn wpisz_koniec_najmu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszKoniecNajmuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO najem_umowy_konce (id, najem_id, sposob, przyczyna, wysylka_id, wplyw_id, koniec_najmu)
VALUES (:id, :najem_id, :sposob, :przyczyna, :wysylka_id, :wplyw_id, :koniec_najmu)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("najem_id", xmeta_room::ToSqlValue::to_sql_value(&params.najem_id)),
                    ("sposob", xmeta_room::ToSqlValue::to_sql_value(&params.sposob)),
                    (
                        "przyczyna",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przyczyna),
                    ),
                    (
                        "wysylka_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wysylka_id),
                    ),
                    ("wplyw_id", xmeta_room::ToSqlValue::to_sql_value(&params.wplyw_id)),
                    (
                        "koniec_najmu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.koniec_najmu),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_doreczenie_konca_najmu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujDoreczenieKoncaNajmuParams {
    pub doreczono: String,
    pub id: String,
}

/// Pismo o końcu najmu doręczono.
pub fn odnotuj_doreczenie_konca_najmu(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujDoreczenieKoncaNajmuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_umowy_konce
SET doreczono = :doreczono
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "doreczono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.doreczono),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_decyzje_po_smierci_najemcy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujDecyzjePoSmierciNajemcyParams {
    pub decyzja_id: String,
    pub koniec_najmu: Option<String>,
    pub id: String,
}

/// Człowiek ocenił, kto wstąpił w najem; gdy nikt, dzień wygaśnięcia najmu.
pub fn odnotuj_decyzje_po_smierci_najemcy(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujDecyzjePoSmierciNajemcyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_umowy_konce
SET decyzja_id = :decyzja_id,
  koniec_najmu = :koniec_najmu
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_id),
                    ),
                    (
                        "koniec_najmu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.koniec_najmu),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wskaz_zadanie_oproznienia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WskazZadanieOproznieniaParams {
    pub zadanie_wysylka_id: String,
    pub id: String,
}

/// Żądanie opróżnienia lokalu przy najmie okazjonalnym albo instytucjonalnym, jeszcze przed wysłaniem.
pub fn wskaz_zadanie_oproznienia(
    room: &impl xmeta_room::MetaRoom,
    params: &WskazZadanieOproznieniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_umowy_konce
SET zadanie_wysylka_id = :zadanie_wysylka_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zadanie_wysylka_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zadanie_wysylka_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_doreczenie_zadania_oproznienia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujDoreczenieZadaniaOproznieniaParams {
    pub zadanie_doreczono: String,
    pub zadanie_termin: String,
    pub id: String,
}

/// Żądanie doręczono; termin opróżnienia z żądania.
pub fn odnotuj_doreczenie_zadania_oproznienia(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujDoreczenieZadaniaOproznieniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_umowy_konce
SET zadanie_doreczono = :zadanie_doreczono,
  zadanie_termin = :zadanie_termin
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zadanie_doreczono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zadanie_doreczono),
                    ),
                    (
                        "zadanie_termin",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zadanie_termin),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_oproznienie_lokalu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujOproznienieLokaluParams {
    pub oproznienie: String,
    pub id: String,
}

/// Najemca oddał klucze i opuścił lokal; od tego dnia biegnie miesiąc na zwrot kaucji.
pub fn odnotuj_oproznienie_lokalu(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujOproznienieLokaluParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_umowy_konce
SET oproznienie = :oproznienie
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "oproznienie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.oproznienie),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `lokale_zastepcze`.
#[derive(Debug, Clone, PartialEq)]
pub struct LokaleZastepczeRow {
    pub id: i64,
    pub najem_id: String,
    pub adres: String,
    pub zgoda_dokument_id: Option<String>,
    pub wskazano: String,
    pub niemozliwy_od: Option<String>,
}

impl LokaleZastepczeRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(LokaleZastepczeRow {
            id: row.get("id")?,
            najem_id: row.get("najem_id")?,
            adres: row.get("adres")?,
            zgoda_dokument_id: row.get("zgoda_dokument_id")?,
            wskazano: row.get("wskazano")?,
            niemozliwy_od: row.get("niemozliwy_od")?,
        })
    }
}

/// Lokale wskazane na wypadek egzekucji; bieżący nie ma niemozliwy_od.
pub fn lokale_zastepcze(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<LokaleZastepczeRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM najem_umowy_lokale_zastepcze
ORDER BY wskazano, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(LokaleZastepczeRow::from_row).collect()
}

/// The parameters of `wpisz_lokal_zastepczy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszLokalZastepczyParams {
    pub najem_id: String,
    pub adres: String,
    pub wskazano: String,
}

/// Lokal, w którym najemca okazjonalny zamieszka w razie egzekucji.
pub fn wpisz_lokal_zastepczy(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszLokalZastepczyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO najem_umowy_lokale_zastepcze (najem_id, adres, wskazano)
VALUES (:najem_id, :adres, :wskazano)"#,
                parameters: Vec::from([
                    ("najem_id", xmeta_room::ToSqlValue::to_sql_value(&params.najem_id)),
                    ("adres", xmeta_room::ToSqlValue::to_sql_value(&params.adres)),
                    ("wskazano", xmeta_room::ToSqlValue::to_sql_value(&params.wskazano)),
                ]),
            },
        ],
    )
}

/// The parameters of `wskaz_zgode_na_lokal_zastepczy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WskazZgodeNaLokalZastepczyParams {
    pub zgoda_dokument_id: String,
    pub id: i64,
}

/// Oświadczenie właściciela tamtego lokalu o zgodzie na zamieszkanie najemcy.
pub fn wskaz_zgode_na_lokal_zastepczy(
    room: &impl xmeta_room::MetaRoom,
    params: &WskazZgodeNaLokalZastepczyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_umowy_lokale_zastepcze
SET zgoda_dokument_id = :zgoda_dokument_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zgoda_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zgoda_dokument_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_utrate_lokalu_zastepczego`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujUtrateLokaluZastepczegoParams {
    pub niemozliwy_od: String,
    pub id: i64,
}

/// Najemca dowiedział się, że nie może zamieszkać we wskazanym lokalu; ma 21 dni na nowy.
pub fn odnotuj_utrate_lokalu_zastepczego(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujUtrateLokaluZastepczegoParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_umowy_lokale_zastepcze
SET niemozliwy_od = :niemozliwy_od
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "niemozliwy_od",
                        xmeta_room::ToSqlValue::to_sql_value(&params.niemozliwy_od),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `umowy_najmu`.
#[derive(Debug, Clone, PartialEq)]
pub struct UmowyNajmuRow {
    pub id: String,
    pub rodzaj: NajmuRodzaj,
    pub nieruchomosc_id: String,
    pub poczatek: String,
    pub koniec: Option<String>,
    pub forma: NajmuForma,
    pub stan: NajmuStan,
    pub koniec_powod: Option<String>,
    pub projekt_dokument_id: Option<String>,
    pub dokument_id: Option<String>,
    pub do_podpisu_decyzja_id: Option<String>,
    pub zawarto: Option<String>,
    pub czynsz: i64,
    pub termin_platnosci: Option<String>,
    pub kaucja: i64,
    pub zwierzeta: Option<String>,
    pub podnajem: Option<String>,
    pub zmiany_w_lokalu: Option<String>,
    pub naprawy_podzial: Option<String>,
    pub wypowiedzenie_przypadki: Option<String>,
    pub oswiadczenie_dokument_id: Option<String>,
    pub swiadectwo_przekazano: Option<String>,
    pub zgloszenie_us_dokument_id: Option<String>,
    pub zgloszono_us: Option<String>,
}

impl UmowyNajmuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(UmowyNajmuRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            nieruchomosc_id: row.get("nieruchomosc_id")?,
            poczatek: row.get("poczatek")?,
            koniec: row.get("koniec")?,
            forma: row.get("forma")?,
            stan: row.get("stan")?,
            koniec_powod: row.get("koniec_powod")?,
            projekt_dokument_id: row.get("projekt_dokument_id")?,
            dokument_id: row.get("dokument_id")?,
            do_podpisu_decyzja_id: row.get("do_podpisu_decyzja_id")?,
            zawarto: row.get("zawarto")?,
            czynsz: row.get("czynsz")?,
            termin_platnosci: row.get("termin_platnosci")?,
            kaucja: row.get("kaucja")?,
            zwierzeta: row.get("zwierzeta")?,
            podnajem: row.get("podnajem")?,
            zmiany_w_lokalu: row.get("zmiany_w_lokalu")?,
            naprawy_podzial: row.get("naprawy_podzial")?,
            wypowiedzenie_przypadki: row.get("wypowiedzenie_przypadki")?,
            oswiadczenie_dokument_id: row.get("oswiadczenie_dokument_id")?,
            swiadectwo_przekazano: row.get("swiadectwo_przekazano")?,
            zgloszenie_us_dokument_id: row.get("zgloszenie_us_dokument_id")?,
            zgloszono_us: row.get("zgloszono_us")?,
        })
    }
}

/// Umowy najmu teczki w kolejności wpisania.
pub fn umowy_najmu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<UmowyNajmuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM najem_umowy_najmu
ORDER BY rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(UmowyNajmuRow::from_row).collect()
}

/// The parameters of `przygotuj_umowe_najmu`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzygotujUmoweNajmuParams {
    pub id: String,
    pub rodzaj: NajmuRodzaj,
    pub nieruchomosc_id: String,
    pub poczatek: String,
    pub koniec: Option<String>,
    pub forma: NajmuForma,
    pub projekt_dokument_id: Option<String>,
    pub czynsz: i64,
    pub termin_platnosci: Option<String>,
    pub kaucja: i64,
    pub zwierzeta: Option<String>,
    pub podnajem: Option<String>,
    pub zmiany_w_lokalu: Option<String>,
    pub naprawy_podzial: Option<String>,
    pub wypowiedzenie_przypadki: Option<String>,
}

/// Projekt umowy najmu według wzoru dla rodzaju najmu, który wybrał wynajmujący; identyfikator
/// (UUID) nadaje aplikacja, bo wskazują go decyzja, opłaty, aneksy i należności.
pub fn przygotuj_umowe_najmu(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzygotujUmoweNajmuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO najem_umowy_najmu (id, rodzaj, nieruchomosc_id, poczatek, koniec, forma,
  projekt_dokument_id, czynsz, termin_platnosci, kaucja, zwierzeta, podnajem, zmiany_w_lokalu,
  naprawy_podzial, wypowiedzenie_przypadki, stan)
VALUES (:id, :rodzaj, :nieruchomosc_id, :poczatek, :koniec, :forma, :projekt_dokument_id, :czynsz,
  :termin_platnosci, :kaucja, :zwierzeta, :podnajem, :zmiany_w_lokalu, :naprawy_podzial,
  :wypowiedzenie_przypadki, 'projekt')"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    (
                        "nieruchomosc_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.nieruchomosc_id),
                    ),
                    ("poczatek", xmeta_room::ToSqlValue::to_sql_value(&params.poczatek)),
                    ("koniec", xmeta_room::ToSqlValue::to_sql_value(&params.koniec)),
                    ("forma", xmeta_room::ToSqlValue::to_sql_value(&params.forma)),
                    (
                        "projekt_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.projekt_dokument_id),
                    ),
                    ("czynsz", xmeta_room::ToSqlValue::to_sql_value(&params.czynsz)),
                    (
                        "termin_platnosci",
                        xmeta_room::ToSqlValue::to_sql_value(&params.termin_platnosci),
                    ),
                    ("kaucja", xmeta_room::ToSqlValue::to_sql_value(&params.kaucja)),
                    (
                        "zwierzeta",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zwierzeta),
                    ),
                    ("podnajem", xmeta_room::ToSqlValue::to_sql_value(&params.podnajem)),
                    (
                        "zmiany_w_lokalu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zmiany_w_lokalu),
                    ),
                    (
                        "naprawy_podzial",
                        xmeta_room::ToSqlValue::to_sql_value(&params.naprawy_podzial),
                    ),
                    (
                        "wypowiedzenie_przypadki",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.wypowiedzenie_przypadki,
                        ),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_projekt_umowy_najmu`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZmienProjektUmowyNajmuParams {
    pub poczatek: String,
    pub koniec: Option<String>,
    pub forma: NajmuForma,
    pub projekt_dokument_id: Option<String>,
    pub czynsz: i64,
    pub termin_platnosci: Option<String>,
    pub kaucja: i64,
    pub zwierzeta: Option<String>,
    pub podnajem: Option<String>,
    pub zmiany_w_lokalu: Option<String>,
    pub naprawy_podzial: Option<String>,
    pub wypowiedzenie_przypadki: Option<String>,
    pub id: String,
}

/// Poprawka projektu przed podpisem; po wyjściu z projektu umowę zmienia aneks.
pub fn zmien_projekt_umowy_najmu(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienProjektUmowyNajmuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_umowy_najmu
SET poczatek = :poczatek,
  koniec = :koniec,
  forma = :forma,
  projekt_dokument_id = :projekt_dokument_id,
  czynsz = :czynsz,
  termin_platnosci = :termin_platnosci,
  kaucja = :kaucja,
  zwierzeta = :zwierzeta,
  podnajem = :podnajem,
  zmiany_w_lokalu = :zmiany_w_lokalu,
  naprawy_podzial = :naprawy_podzial,
  wypowiedzenie_przypadki = :wypowiedzenie_przypadki
WHERE id = :id"#,
                parameters: Vec::from([
                    ("poczatek", xmeta_room::ToSqlValue::to_sql_value(&params.poczatek)),
                    ("koniec", xmeta_room::ToSqlValue::to_sql_value(&params.koniec)),
                    ("forma", xmeta_room::ToSqlValue::to_sql_value(&params.forma)),
                    (
                        "projekt_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.projekt_dokument_id),
                    ),
                    ("czynsz", xmeta_room::ToSqlValue::to_sql_value(&params.czynsz)),
                    (
                        "termin_platnosci",
                        xmeta_room::ToSqlValue::to_sql_value(&params.termin_platnosci),
                    ),
                    ("kaucja", xmeta_room::ToSqlValue::to_sql_value(&params.kaucja)),
                    (
                        "zwierzeta",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zwierzeta),
                    ),
                    ("podnajem", xmeta_room::ToSqlValue::to_sql_value(&params.podnajem)),
                    (
                        "zmiany_w_lokalu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zmiany_w_lokalu),
                    ),
                    (
                        "naprawy_podzial",
                        xmeta_room::ToSqlValue::to_sql_value(&params.naprawy_podzial),
                    ),
                    (
                        "wypowiedzenie_przypadki",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.wypowiedzenie_przypadki,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wskaz_oswiadczenie_najemcy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WskazOswiadczenieNajemcyParams {
    pub oswiadczenie_dokument_id: String,
    pub id: String,
}

/// Wypis aktu z oświadczeniem najemcy (najem okazjonalny i instytucjonalny), przed podpisem.
pub fn wskaz_oswiadczenie_najemcy(
    room: &impl xmeta_room::MetaRoom,
    params: &WskazOswiadczenieNajemcyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_umowy_najmu
SET oswiadczenie_dokument_id = :oswiadczenie_dokument_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "oswiadczenie_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.oswiadczenie_dokument_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `przekaz_umowe_najmu_do_podpisu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrzekazUmoweNajmuDoPodpisuParams {
    pub do_podpisu_decyzja_id: String,
    pub id: String,
}

/// Projekt trafia do stron za zatwierdzoną decyzją inna, która wskazuje tę umowę.
pub fn przekaz_umowe_najmu_do_podpisu(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzekazUmoweNajmuDoPodpisuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_umowy_najmu
SET stan = 'do_podpisu',
  do_podpisu_decyzja_id = :do_podpisu_decyzja_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "do_podpisu_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.do_podpisu_decyzja_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_zawarcie_umowy_najmu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujZawarcieUmowyNajmuParams {
    pub dokument_id: String,
    pub zawarto: String,
    pub id: String,
}

/// Strony podpisały umowę najmu: podpisany egzemplarz i chwila.
pub fn odnotuj_zawarcie_umowy_najmu(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujZawarcieUmowyNajmuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_umowy_najmu
SET stan = 'zawarta',
  dokument_id = :dokument_id,
  zawarto = :zawarto
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                    ("zawarto", xmeta_room::ToSqlValue::to_sql_value(&params.zawarto)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_przekazanie_swiadectwa`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPrzekazanieSwiadectwaParams {
    pub swiadectwo_przekazano: String,
    pub id: String,
}

/// Wynajmujący przekazał najemcy kopię świadectwa charakterystyki energetycznej.
pub fn odnotuj_przekazanie_swiadectwa(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPrzekazanieSwiadectwaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_umowy_najmu
SET swiadectwo_przekazano = :swiadectwo_przekazano
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "swiadectwo_przekazano",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.swiadectwo_przekazano,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_zgloszenie_najmu_do_urzedu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujZgloszenieNajmuDoUrzeduParams {
    pub zgloszono_us: String,
    pub zgloszenie_us_dokument_id: String,
    pub id: String,
}

/// Właściciel zgłosił najem okazjonalny naczelnikowi urzędu skarbowego; potwierdzenie zgłoszenia.
pub fn odnotuj_zgloszenie_najmu_do_urzedu(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujZgloszenieNajmuDoUrzeduParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_umowy_najmu
SET zgloszono_us = :zgloszono_us,
  zgloszenie_us_dokument_id = :zgloszenie_us_dokument_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zgloszono_us",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zgloszono_us),
                    ),
                    (
                        "zgloszenie_us_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.zgloszenie_us_dokument_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `zakoncz_umowe_najmu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZakonczUmoweNajmuParams {
    pub koniec_powod: String,
    pub id: String,
}

/// Koniec umowy najmu albo projektu, którego strony nie podpiszą.
pub fn zakoncz_umowe_najmu(
    room: &impl xmeta_room::MetaRoom,
    params: &ZakonczUmoweNajmuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_umowy_najmu
SET stan = 'zakonczona',
  koniec_powod = :koniec_powod
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "koniec_powod",
                        xmeta_room::ToSqlValue::to_sql_value(&params.koniec_powod),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `oplaty_najmu`.
#[derive(Debug, Clone, PartialEq)]
pub struct OplatyNajmuRow {
    pub id: i64,
    pub najem_id: String,
    pub rodzaj: OplatyRodzaj,
    pub sposob: String,
    pub umowa_z_dostawca_ma: OplatyUmowaZDostawcaMa,
}

impl OplatyNajmuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(OplatyNajmuRow {
            id: row.get("id")?,
            najem_id: row.get("najem_id")?,
            rodzaj: row.get("rodzaj")?,
            sposob: row.get("sposob")?,
            umowa_z_dostawca_ma: row.get("umowa_z_dostawca_ma")?,
        })
    }
}

/// Opłaty obok czynszu z tym, kto ma umowę z dostawcą.
pub fn oplaty_najmu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<OplatyNajmuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM najem_umowy_oplaty
ORDER BY najem_id, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(OplatyNajmuRow::from_row).collect()
}

/// The parameters of `dopisz_oplate_najmu`.
#[derive(Debug, Clone, PartialEq)]
pub struct DopiszOplateNajmuParams {
    pub najem_id: String,
    pub rodzaj: OplatyRodzaj,
    pub sposob: String,
    pub umowa_z_dostawca_ma: OplatyUmowaZDostawcaMa,
}

/// Opłata z projektu umowy najmu.
pub fn dopisz_oplate_najmu(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszOplateNajmuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO najem_umowy_oplaty (najem_id, rodzaj, sposob, umowa_z_dostawca_ma)
VALUES (:najem_id, :rodzaj, :sposob, :umowa_z_dostawca_ma)"#,
                parameters: Vec::from([
                    ("najem_id", xmeta_room::ToSqlValue::to_sql_value(&params.najem_id)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("sposob", xmeta_room::ToSqlValue::to_sql_value(&params.sposob)),
                    (
                        "umowa_z_dostawca_ma",
                        xmeta_room::ToSqlValue::to_sql_value(&params.umowa_z_dostawca_ma),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `zamieszkujacy`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZamieszkujacyRow {
    pub id: i64,
    pub najem_id: String,
    pub uczestnik_id: String,
    pub od: String,
    pub r#do: Option<String>,
}

impl ZamieszkujacyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZamieszkujacyRow {
            id: row.get("id")?,
            najem_id: row.get("najem_id")?,
            uczestnik_id: row.get("uczestnik_id")?,
            od: row.get("od")?,
            r#do: row.get("do")?,
        })
    }
}

/// Osoby zamieszkujące z najemcą z datą zamieszkania i wyprowadzki.
pub fn zamieszkujacy(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZamieszkujacyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM najem_umowy_zamieszkujacy
ORDER BY od, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZamieszkujacyRow::from_row).collect()
}

/// The parameters of `wpisz_zamieszkujacego`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszZamieszkujacegoParams {
    pub najem_id: String,
    pub uczestnik_id: String,
    pub od: String,
}

/// Osoba, która zamieszkała z najemcą, i od kiedy.
pub fn wpisz_zamieszkujacego(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszZamieszkujacegoParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO najem_umowy_zamieszkujacy (najem_id, uczestnik_id, od)
VALUES (:najem_id, :uczestnik_id, :od)"#,
                parameters: Vec::from([
                    ("najem_id", xmeta_room::ToSqlValue::to_sql_value(&params.najem_id)),
                    (
                        "uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.uczestnik_id),
                    ),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_wyprowadzke_zamieszkujacego`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujWyprowadzkeZamieszkujacegoParams {
    pub r#do: String,
    pub id: i64,
}

/// Osoba przestała mieszkać z najemcą.
pub fn odnotuj_wyprowadzke_zamieszkujacego(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWyprowadzkeZamieszkujacegoParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_umowy_zamieszkujacy
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

/// A row of `warunki_zarzadzania`.
#[derive(Debug, Clone, PartialEq)]
pub struct WarunkiZarzadzaniaRow {
    pub id: i64,
    pub umowa_id: String,
    pub kwota_napraw: i64,
    pub przeplyw: ZarzadzaniePrzeplyw,
    pub aneks_zdarzenie_id: Option<String>,
    pub od: String,
}

impl WarunkiZarzadzaniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WarunkiZarzadzaniaRow {
            id: row.get("id")?,
            umowa_id: row.get("umowa_id")?,
            kwota_napraw: row.get("kwota_napraw")?,
            przeplyw: row.get("przeplyw")?,
            aneks_zdarzenie_id: row.get("aneks_zdarzenie_id")?,
            od: row.get("od")?,
        })
    }
}

/// Wersje warunków umowy o zarządzanie; bieżąca jest ostatnia.
pub fn warunki_zarzadzania(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WarunkiZarzadzaniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM najem_umowy_zarzadzanie
ORDER BY id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(WarunkiZarzadzaniaRow::from_row).collect()
}

/// The parameters of `wpisz_warunki_zarzadzania`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszWarunkiZarzadzaniaParams {
    pub umowa_id: String,
    pub kwota_napraw: i64,
    pub przeplyw: ZarzadzaniePrzeplyw,
    pub aneks_zdarzenie_id: Option<String>,
    pub od: String,
}

/// Kwota napraw i przepływ pieniędzy: pierwsza wersja od zawarcia umowy o zarządzanie, kolejna od
/// aneksu tej umowy.
pub fn wpisz_warunki_zarzadzania(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszWarunkiZarzadzaniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO najem_umowy_zarzadzanie (umowa_id, kwota_napraw, przeplyw, aneks_zdarzenie_id, od)
VALUES (:umowa_id, :kwota_napraw, :przeplyw, :aneks_zdarzenie_id, :od)"#,
                parameters: Vec::from([
                    ("umowa_id", xmeta_room::ToSqlValue::to_sql_value(&params.umowa_id)),
                    (
                        "kwota_napraw",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kwota_napraw),
                    ),
                    ("przeplyw", xmeta_room::ToSqlValue::to_sql_value(&params.przeplyw)),
                    (
                        "aneks_zdarzenie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.aneks_zdarzenie_id),
                    ),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                ]),
            },
        ],
    )
}
