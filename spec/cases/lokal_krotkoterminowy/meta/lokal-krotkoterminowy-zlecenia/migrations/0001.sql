-- Zlecenia dla wykonawców (sprzątanie, pranie, serwis, naprawy, przeglądy, zapasy, klucze,
-- przyjęcie gości, wymiana zamka) i dostępy do lokalu: kody do zamków i klucze. Pisze Koordynator
-- operacji obszaru. Wykonawcy i cenniki leżą w rejestrze wykonawców, więc zlecenie wskazuje je
-- tekstem; wykonawcom płaci człowiek poza teczką. Wykonawca nie dostaje danych gości: pilnuje tego
-- aplikacja.
--
-- Zlecenie wykonawcy spoza rejestru, zlecenie poza standardem lokalu, wymiana zamka i cena
-- z wyceny ponad kwotę napraw z umowy wymagają zgody człowieka przed zleceniem; ta ostatnia także
-- zgody właściciela. Jedna decyzja obejmuje całe zlecenie ze wszystkimi powodami. Koszt wpisuje
-- się po potwierdzeniu wykonania; koszt inny niż cena wymaga osobnej decyzji.
--
-- Kody do zamków leżą w teczce (T7): każdy członek teczki je widzi. Reguła dotyczy wydania:
-- dostęp dla pobytu wymaga opłaconej rezerwacji i mieści się w dniach pobytu, a dostęp dla
-- zlecenia mieści się w jego oknie. Godziny zameldowania i wymeldowania z regulaminu porównuje
-- aplikacja.
CREATE TABLE lokal_krotkoterminowy_zlecenia_wpisy (
  id TEXT PRIMARY KEY,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('sprzatanie', 'pranie', 'serwis', 'naprawa', 'przeglad', 'dostawa_zapasow', 'wydanie_kluczy', 'odbior_kluczy', 'przyjecie_gosci', 'wymiana_zamka', 'prace_poza_lokalem')),
  -- Dlaczego: wyjazd albo przyjazd gościa (wskazuje pobyt), zgłoszenie (wskazuje zgłoszenie),
  -- raport wykonawcy, przegląd ze standardu lokalu, przygotowanie albo zamknięcie lokalu.
  przyczyna TEXT NOT NULL CHECK (przyczyna IN ('wyjazd_goscia', 'przyjazd_goscia', 'zgloszenie', 'raport_wykonawcy', 'standard_lokalu', 'przygotowanie_lokalu', 'zamkniecie_lokalu')),
  pobyt_id TEXT REFERENCES lokal_krotkoterminowy_pobyty_wpisy (id),
  zgloszenie_id TEXT REFERENCES lokal_krotkoterminowy_pobyty_zgloszenia (id),
  -- Wykonawca i wersja cennika w rejestrze wykonawców, tekstem (wykonawcy_spis_wpisy,
  -- wykonawcy_cenniki_wersje).
  wykonawca_odnosnik TEXT NOT NULL CHECK (wykonawca_odnosnik <> ''),
  spoza_rejestru INTEGER NOT NULL CHECK (spoza_rejestru IN (0, 1)),
  cennik_wersja_odnosnik TEXT,
  poza_standardem INTEGER NOT NULL CHECK (poza_standardem IN (0, 1)),
  -- Okno pracy: np. od wymeldowania do zameldowania następnego gościa.
  okno_od TEXT NOT NULL,
  okno_do TEXT NOT NULL,
  -- Cena w groszach z cennika rejestru albo z wyceny wykonawcy.
  cena INTEGER NOT NULL CHECK (cena >= 0),
  cena_zrodlo TEXT NOT NULL CHECK (cena_zrodlo IN ('cennik', 'wycena')),
  -- Zgoda właściciela zgoda_na_naprawe przy cenie z wyceny ponad kwotę napraw.
  zgoda_id TEXT REFERENCES sprawa_zgody_wpisy (id),
  -- Zgoda człowieka na zlecenie, gdy ma ono powód, który jej wymaga.
  decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),
  zlecil TEXT NOT NULL,
  zlecono TEXT NOT NULL,
  -- Wykonanie: raport wykonawcy ze zdjęciami i potwierdzenie po porównaniu z listą kontrolną.
  raport_sciezka TEXT,
  raport_skrot TEXT,
  potwierdzil TEXT,
  potwierdzono TEXT,
  -- Koszt w groszach po potwierdzeniu; inny niż cena wskazuje zgodę człowieka.
  koszt INTEGER CHECK (koszt >= 0),
  koszt_decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),
  stan TEXT NOT NULL CHECK (stan IN ('zlecone', 'wstrzymane', 'potwierdzone', 'anulowane')),

  CONSTRAINT przyczyna_wskazuje_pobyt_albo_zgloszenie CHECK (
    (przyczyna NOT IN ('wyjazd_goscia', 'przyjazd_goscia') OR pobyt_id IS NOT NULL)
    AND (przyczyna <> 'zgloszenie' OR zgloszenie_id IS NOT NULL)
  ),
  CONSTRAINT cena_z_cennika_rejestru CHECK (
    cena_zrodlo <> 'cennik' OR (spoza_rejestru = 0 AND cennik_wersja_odnosnik IS NOT NULL)
  ),
  CONSTRAINT okno_od_do CHECK (okno_do > okno_od),
  CONSTRAINT raport_ze_skrotem CHECK ((raport_sciezka IS NULL) = (raport_skrot IS NULL)),
  CONSTRAINT potwierdzone_z_raportem CHECK (
    (stan = 'potwierdzone') = (potwierdzono IS NOT NULL)
    AND (stan = 'potwierdzone') = (potwierdzil IS NOT NULL)
    AND (stan <> 'potwierdzone' OR raport_sciezka IS NOT NULL)
  ),
  CONSTRAINT koszt_po_potwierdzeniu CHECK (koszt IS NULL OR stan = 'potwierdzone'),
  CONSTRAINT decyzja_kosztu_przy_koszcie CHECK (koszt_decyzja_id IS NULL OR koszt IS NOT NULL)
) STRICT;

-- Dostępy do lokalu: kod do zamka albo klucze, dla pobytu albo dla zlecenia, z czasem ważności,
-- adresatem i chwilami wydania oraz odebrania (przy kodzie: wygaszenia).
CREATE TABLE lokal_krotkoterminowy_zlecenia_dostepy (
  -- Klucz nadany przez bazę jest dodatni; zob. lokal_krotkoterminowy_zlecenia_dostepy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  pobyt_id TEXT REFERENCES lokal_krotkoterminowy_pobyty_wpisy (id),
  zlecenie_id TEXT REFERENCES lokal_krotkoterminowy_zlecenia_wpisy (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('kod', 'klucze')),
  -- Kod do zamka (T7: w teczce) albo liczba wydanych kluczy.
  kod TEXT CHECK (kod <> ''),
  klucze_liczba INTEGER CHECK (klucze_liczba > 0),
  -- Zamek w systemie zamków, tekstem.
  zamek_odnosnik TEXT,
  wazny_od TEXT NOT NULL,
  wazny_do TEXT NOT NULL,
  -- Komu: gość z kanału, wykonawca albo gospodarz na miejscu.
  kto TEXT NOT NULL CHECK (kto <> ''),
  ustawiono TEXT NOT NULL,
  wydano TEXT,
  odebrano TEXT,

  CONSTRAINT dostep_dla_pobytu_albo_zlecenia CHECK ((pobyt_id IS NULL) <> (zlecenie_id IS NULL)),
  CONSTRAINT kod_albo_klucze CHECK (
    (rodzaj = 'kod') = (kod IS NOT NULL)
    AND (rodzaj = 'klucze') = (klucze_liczba IS NOT NULL)
  ),
  CONSTRAINT waznosc_od_do CHECK (wazny_do > wazny_od),
  CONSTRAINT klucze_odebrane_po_wydaniu CHECK (
    odebrano IS NULL
    OR (rodzaj = 'kod' AND wydano IS NULL)
    OR coalesce(odebrano >= wydano, 0)
  )
) STRICT;

-- Potwierdzenie i koszt sprawdzają zmianę, więc zlecenie powstaje zlecone, bez wyniku.
CREATE TRIGGER lokal_krotkoterminowy_zlecenia_powstaje_zlecone
BEFORE INSERT ON lokal_krotkoterminowy_zlecenia_wpisy
WHEN NEW.stan <> 'zlecone'
  OR NEW.raport_sciezka IS NOT NULL
  OR NEW.potwierdzono IS NOT NULL
  OR NEW.koszt IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'zlecenie powstaje zlecone, bez raportu, potwierdzenia i kosztu');
END;

-- Zlecenie spoza rejestru, poza standardem, wymiana zamka i cena z wyceny ponad kwotę napraw
-- z bieżących warunków umowy wychodzą tylko za zgodą człowieka. Bez ustalonych warunków nie ma
-- kwoty napraw, więc każda wycena wymaga zgody.
CREATE TRIGGER lokal_krotkoterminowy_zlecenia_zlecenie_za_zgoda
BEFORE INSERT ON lokal_krotkoterminowy_zlecenia_wpisy
WHEN (
    NEW.spoza_rejestru = 1
    OR NEW.poza_standardem = 1
    OR NEW.rodzaj = 'wymiana_zamka'
    OR (
      NEW.cena_zrodlo = 'wycena'
      AND coalesce(
        NEW.cena > (
          SELECT kwota_napraw
          FROM lokal_krotkoterminowy_umowa_warunki_biezace
        ),
        1
      )
    )
  )
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone
    WHERE id = NEW.decyzja_id
      AND rodzaj = 'inna'
      AND dotyczy_tabela = 'lokal_krotkoterminowy_zlecenia_wpisy'
      AND dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'zlecenie spoza rejestru, poza standardem, wymiana zamka i wycena ponad kwotę napraw wskazują zatwierdzoną decyzję inna, która je wskazuje');
END;

-- Cenę z wyceny ponad kwotę napraw zatwierdza też właściciel lokalu: zgoda_na_naprawe właściciela
-- albo współwłaściciela, która wskazuje to zlecenie.
CREATE TRIGGER lokal_krotkoterminowy_zlecenia_wycena_ponad_kwote_za_zgoda_wlasciciela
BEFORE INSERT ON lokal_krotkoterminowy_zlecenia_wpisy
WHEN NEW.cena_zrodlo = 'wycena'
  AND coalesce(
    NEW.cena > (
      SELECT kwota_napraw
      FROM lokal_krotkoterminowy_umowa_warunki_biezace
    ),
    1
  )
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_zgody_wpisy AS zgoda
    JOIN sprawa_uczestnicy_role AS rola ON rola.uczestnik_id = zgoda.uczestnik_id
    WHERE zgoda.id = NEW.zgoda_id
      AND zgoda.rodzaj = 'zgoda_na_naprawe'
      AND zgoda.dotyczy_tabela = 'lokal_krotkoterminowy_zlecenia_wpisy'
      AND zgoda.dotyczy_id = NEW.id
      AND rola.rola IN ('wlasciciel_lokalu', 'wspolwlasciciel')
      AND rola.zakonczono IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'cena z wyceny ponad kwotę napraw wskazuje zgodę zgoda_na_naprawe właściciela lokalu, która wskazuje zlecenie');
END;

-- Zlecenie czeka, gdy Koordynator wstrzymał prace (np. kody mogły wyciec), i wraca po decyzji
-- człowieka. Potwierdzone i anulowane są ostateczne.
CREATE TRIGGER lokal_krotkoterminowy_zlecenia_przejscia_stanu
BEFORE UPDATE OF stan ON lokal_krotkoterminowy_zlecenia_wpisy
WHEN NEW.stan IS NOT OLD.stan
  AND NOT (
    (OLD.stan = 'zlecone' AND NEW.stan IN ('wstrzymane', 'potwierdzone', 'anulowane'))
    OR (OLD.stan = 'wstrzymane' AND NEW.stan IN ('zlecone', 'anulowane'))
  )
BEGIN
  SELECT RAISE(ABORT, 'zlecenie przechodzi: zlecone, wstrzymane i z powrotem, potwierdzone albo anulowane');
END;

-- Na treści zlecenia stoją zgody człowieka i właściciela oraz dostępy w jego oknie. Inne zlecenie
-- to anulowanie i nowy wpis.
CREATE TRIGGER lokal_krotkoterminowy_zlecenia_tresc_niezmienna
BEFORE UPDATE ON lokal_krotkoterminowy_zlecenia_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.przyczyna IS NOT OLD.przyczyna
  OR NEW.pobyt_id IS NOT OLD.pobyt_id
  OR NEW.zgloszenie_id IS NOT OLD.zgloszenie_id
  OR NEW.wykonawca_odnosnik IS NOT OLD.wykonawca_odnosnik
  OR NEW.spoza_rejestru IS NOT OLD.spoza_rejestru
  OR NEW.cennik_wersja_odnosnik IS NOT OLD.cennik_wersja_odnosnik
  OR NEW.poza_standardem IS NOT OLD.poza_standardem
  OR NEW.okno_od IS NOT OLD.okno_od
  OR NEW.okno_do IS NOT OLD.okno_do
  OR NEW.cena IS NOT OLD.cena
  OR NEW.cena_zrodlo IS NOT OLD.cena_zrodlo
  OR NEW.zgoda_id IS NOT OLD.zgoda_id
  OR NEW.decyzja_id IS NOT OLD.decyzja_id
  OR NEW.zlecil IS NOT OLD.zlecil
  OR NEW.zlecono IS NOT OLD.zlecono
BEGIN
  SELECT RAISE(ABORT, 'treść zlecenia się nie zmienia; inne zlecenie to anulowanie i nowy wpis');
END;

-- Potwierdzenie z raportem wpisuje się raz, a koszt i jego decyzję też raz: na nich stoi pozycja
-- zestawienia dla właściciela. Anulowanego zlecenia się nie zmienia.
CREATE TRIGGER lokal_krotkoterminowy_zlecenia_wynik_raz
BEFORE UPDATE ON lokal_krotkoterminowy_zlecenia_wpisy
WHEN OLD.stan = 'anulowane'
  OR (
    OLD.stan = 'potwierdzone'
    AND (
      NEW.raport_sciezka IS NOT OLD.raport_sciezka
      OR NEW.raport_skrot IS NOT OLD.raport_skrot
      OR NEW.potwierdzil IS NOT OLD.potwierdzil
      OR NEW.potwierdzono IS NOT OLD.potwierdzono
    )
  )
  OR (
    OLD.koszt IS NOT NULL
    AND (NEW.koszt IS NOT OLD.koszt OR NEW.koszt_decyzja_id IS NOT OLD.koszt_decyzja_id)
  )
BEGIN
  SELECT RAISE(ABORT, 'potwierdzenie i koszt zlecenia wpisuje się raz, a anulowanego zlecenia się nie zmienia');
END;

-- Gdy wykonawca podaje inną kwotę niż cena, Koordynator prosi człowieka o decyzję po
-- potwierdzeniu wykonania. Decyzja o samym zleceniu jej nie zastępuje.
CREATE TRIGGER lokal_krotkoterminowy_zlecenia_koszt_inny_niz_cena_za_zgoda
BEFORE UPDATE OF koszt, koszt_decyzja_id ON lokal_krotkoterminowy_zlecenia_wpisy
WHEN NEW.koszt IS NOT NULL
  AND NEW.koszt <> NEW.cena
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone
    WHERE id = NEW.koszt_decyzja_id
      AND id IS NOT NEW.decyzja_id
      AND rodzaj = 'inna'
      AND dotyczy_tabela = 'lokal_krotkoterminowy_zlecenia_wpisy'
      AND dotyczy_id = NEW.id
      AND zlozono >= NEW.potwierdzono
  )
BEGIN
  SELECT RAISE(ABORT, 'koszt inny niż cena wskazuje zatwierdzoną decyzję inna o tym zleceniu, o którą poproszono po potwierdzeniu wykonania');
END;

-- Gość dostaje dostęp tylko do opłaconego pobytu, który się nie skończył, i tylko na jego dni.
CREATE TRIGGER lokal_krotkoterminowy_zlecenia_dostep_dla_oplaconego_pobytu
BEFORE INSERT ON lokal_krotkoterminowy_zlecenia_dostepy
WHEN NEW.pobyt_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_pobyty_wpisy
    WHERE id = NEW.pobyt_id
      AND oplacona = 1
      AND stan IN ('potwierdzona', 'trwa')
      AND substr(NEW.wazny_od, 1, 10) >= przyjazd
      AND substr(NEW.wazny_do, 1, 10) <= wyjazd
  )
BEGIN
  SELECT RAISE(ABORT, 'dostęp dla pobytu wymaga opłaconej rezerwacji, która trwa albo czeka, i mieści się w dniach pobytu');
END;

-- Wykonawca dostaje dostęp tylko na okno zlecenia, które nie jest wstrzymane ani zamknięte.
CREATE TRIGGER lokal_krotkoterminowy_zlecenia_dostep_w_oknie_zlecenia
BEFORE INSERT ON lokal_krotkoterminowy_zlecenia_dostepy
WHEN NEW.zlecenie_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_zlecenia_wpisy
    WHERE id = NEW.zlecenie_id
      AND stan = 'zlecone'
      AND NEW.wazny_od >= okno_od
      AND NEW.wazny_do <= okno_do
  )
BEGIN
  SELECT RAISE(ABORT, 'dostęp dla zlecenia mieści się w oknie zlecenia, które jest zlecone');
END;

-- Wydanie kodu albo kluczy sprawdza warunki jeszcze raz: pobyt mógł zostać odwołany, a zlecenie
-- wstrzymane.
CREATE TRIGGER lokal_krotkoterminowy_zlecenia_wydanie_dostepu
BEFORE UPDATE OF wydano ON lokal_krotkoterminowy_zlecenia_dostepy
WHEN NEW.wydano IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_pobyty_wpisy
    WHERE id = NEW.pobyt_id
      AND oplacona = 1
      AND stan IN ('potwierdzona', 'trwa')
  )
  AND NOT EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_zlecenia_wpisy
    WHERE id = NEW.zlecenie_id
      AND stan = 'zlecone'
  )
BEGIN
  SELECT RAISE(ABORT, 'kod albo klucze wydaje się tylko dla opłaconego pobytu, który trwa albo czeka, albo dla zlecenia, które jest zlecone');
END;

-- Dostęp jest zapisem tego, kto, czym i na jaki czas wchodzi do lokalu; wydanie i odebranie
-- wpisuje się raz. Inny czas albo inny kod to nowy dostęp.
CREATE TRIGGER lokal_krotkoterminowy_zlecenia_dostep_zmienia_tylko_wydanie_i_odebranie
BEFORE UPDATE ON lokal_krotkoterminowy_zlecenia_dostepy
WHEN NEW.id IS NOT OLD.id
  OR NEW.pobyt_id IS NOT OLD.pobyt_id
  OR NEW.zlecenie_id IS NOT OLD.zlecenie_id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.kod IS NOT OLD.kod
  OR NEW.klucze_liczba IS NOT OLD.klucze_liczba
  OR NEW.zamek_odnosnik IS NOT OLD.zamek_odnosnik
  OR NEW.wazny_od IS NOT OLD.wazny_od
  OR NEW.wazny_do IS NOT OLD.wazny_do
  OR NEW.kto IS NOT OLD.kto
  OR NEW.ustawiono IS NOT OLD.ustawiono
  OR (OLD.wydano IS NOT NULL AND NEW.wydano IS NOT OLD.wydano)
  OR (OLD.odebrano IS NOT NULL AND NEW.odebrano IS NOT OLD.odebrano)
BEGIN
  SELECT RAISE(ABORT, 'dostępu się nie zmienia; wydanie i odebranie wpisuje się raz, a inny czas albo kod to nowy dostęp');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER lokal_krotkoterminowy_zlecenia_wpisy_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_zlecenia_wpisy
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_zlecenia_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_zlecenia_wpisy_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_zlecenia_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER lokal_krotkoterminowy_zlecenia_dostepy_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_zlecenia_dostepy
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_zlecenia_dostepy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_zlecenia_dostepy_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_zlecenia_dostepy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
