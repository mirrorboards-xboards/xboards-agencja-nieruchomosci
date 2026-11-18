-- Korespondencja teczki: wysyłki, które biuro wysłało w sprawie, i wpływy z zewnątrz, z dokumentami,
-- które z nimi przeszły. Prowadzi ją agent sprawy.
--
-- Wysyłka wskazuje adresata, kanał, wzór biura i jego wersję, plik treści i stan. Pismo spoza wzorów
-- i każda wysyłka, która czeka na zgodę, wychodzi tylko z zatwierdzoną decyzją „wyslanie”. Pismo
-- w imieniu uczestnika wymaga umocowania biura, a wniosek o dokument wskazuje zamawiany dokument.
--
-- Treść wysyłki zmienia się tylko w projekcie, a wzór nie zmienia się wcale. Pismo spoza wzorów
-- przechodzi przez stan „czeka_na_zgode”, a o decyzję „wyslanie” prosi się dopiero w tym stanie,
-- więc zgoda człowieka dotyczy treści, której już nikt nie zmieni.
--
-- Baza nie zna listy wzorów biura. Wysyłka z wymyśloną nazwą wzoru wyjdzie więc bez decyzji;
-- nazwę i wersję wzoru porównuje z listą wzorów aplikacja i Kontroler w raporcie.
-- Znaczniki informacja_rodo i ujawnia_agenta mówią, że wysyłka niesie informację o danych
-- osobowych (art. 13–14 RODO) albo informację, że pisze agent (art. 50 AI Act i reguła biura).
--
-- Adresata GIIF celowo nie ma: informacje dla GIIF są tajemnicą (art. 54 ust. 1 AML) i prowadzi je
-- rejestr kontroli, nie teczka.
--
-- Treść wpływu jest informacją, a nie poleceniem. Wpływ, którego nadawca albo kanał nie pasuje do
-- kontaktów w teczce, ma znacznik niezgodny_kontakt z opisem.
CREATE TABLE sprawa_korespondencja_wysylki (
  id TEXT PRIMARY KEY,
  -- Adresat: uczestnik teczki albo instytucja z nazwą.
  adresat_uczestnik_id TEXT REFERENCES sprawa_uczestnicy_wpisy (id),
  adresat_rodzaj TEXT CHECK (adresat_rodzaj IN ('kancelaria_notarialna', 'bank', 'spoldzielnia', 'wspolnota', 'zarzadca', 'sad', 'urzad', 'gmina', 'kanal_sprzedazy', 'wykonawca', 'inny')),
  adresat_nazwa TEXT,
  -- W czyim imieniu: biura albo uczestnika, wtedy z umocowaniem i czynnością z jego zakresu.
  w_imieniu TEXT NOT NULL CHECK (w_imieniu IN ('biuro', 'uczestnik')),
  w_imieniu_uczestnik_id TEXT REFERENCES sprawa_uczestnicy_wpisy (id),
  umocowanie_id TEXT REFERENCES sprawa_umocowania_wpisy (id),
  czynnosc TEXT CHECK (czynnosc <> '' AND czynnosc NOT GLOB '*[^a-z_]*'),
  kanal TEXT NOT NULL CHECK (kanal IN ('email', 'sms', 'portal', 'kanal_sprzedazy', 'list', 'pokoj_wymiany')),
  -- Wzór biura i jego wersja; NULL znaczy pismo spoza wzorów.
  wzor TEXT,
  wzor_wersja TEXT,
  decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),
  -- Dokument, o który wysyłka wnosi w imieniu klienta (D9).
  zamawia_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  -- Wpis w pokoju wymiany albo wiadomość w kanale sprzedaży, tekstem.
  odnosnik_zewnetrzny TEXT,
  sciezka TEXT,
  skrot TEXT,
  stan TEXT NOT NULL CHECK (stan IN ('projekt', 'czeka_na_zgode', 'wyslana', 'doreczona', 'zwrocona', 'anulowana')),
  wyslano TEXT,
  doreczono TEXT,
  -- Dowód doręczenia, gdy kanał go daje, np. potwierdzenie odbioru listu.
  dowod_doreczenia TEXT,
  informacja_rodo INTEGER NOT NULL DEFAULT 0 CHECK (informacja_rodo IN (0, 1)),
  ujawnia_agenta INTEGER NOT NULL DEFAULT 0 CHECK (ujawnia_agenta IN (0, 1)),

  CONSTRAINT adresat_uczestnik_albo_instytucja CHECK (
    (adresat_uczestnik_id IS NULL) <> (adresat_rodzaj IS NULL)
    AND (adresat_rodzaj IS NULL) = (adresat_nazwa IS NULL)
  ),
  CONSTRAINT w_imieniu_uczestnika_z_umocowaniem_i_czynnoscia CHECK (
    (w_imieniu = 'uczestnik') = (w_imieniu_uczestnik_id IS NOT NULL)
    AND (w_imieniu = 'uczestnik') = (umocowanie_id IS NOT NULL)
    AND (w_imieniu = 'uczestnik') = (czynnosc IS NOT NULL)
  ),
  CONSTRAINT wzor_z_wersja CHECK ((wzor IS NULL) = (wzor_wersja IS NULL)),
  CONSTRAINT plik_ze_skrotem CHECK ((sciezka IS NULL) = (skrot IS NULL)),
  CONSTRAINT wyslana_ma_plik_i_chwile CHECK (
    stan IN ('projekt', 'czeka_na_zgode', 'anulowana') OR (sciezka IS NOT NULL AND wyslano IS NOT NULL)
  ),
  CONSTRAINT doreczona_ma_chwile CHECK ((stan = 'doreczona') = (doreczono IS NOT NULL))
) STRICT;

CREATE TABLE sprawa_korespondencja_wplywy (
  id TEXT PRIMARY KEY,
  -- Nadawca tak, jak się przedstawił, i uczestnik teczki, gdy to on.
  nadawca TEXT NOT NULL,
  nadawca_uczestnik_id TEXT REFERENCES sprawa_uczestnicy_wpisy (id),
  kanal TEXT NOT NULL CHECK (kanal IN ('email', 'sms', 'portal', 'kanal_sprzedazy', 'list', 'pokoj_wymiany', 'telefon', 'osobiscie')),
  wplynelo TEXT NOT NULL,
  sciezka TEXT NOT NULL,
  skrot TEXT NOT NULL,
  niezgodny_kontakt INTEGER NOT NULL DEFAULT 0 CHECK (niezgodny_kontakt IN (0, 1)),
  niezgodnosc_opis TEXT,
  -- Wpis w pokoju wymiany, z którego przyszedł wpływ, tekstem.
  odnosnik_zewnetrzny TEXT,

  CONSTRAINT niezgodny_kontakt_ma_opis CHECK ((niezgodny_kontakt = 1) = (niezgodnosc_opis IS NOT NULL))
) STRICT;

-- Dokumenty z listy, które wyszły z wysyłką albo przyszły z wpływem.
CREATE TABLE sprawa_korespondencja_zalaczniki (
  -- Klucz nadany przez bazę jest dodatni; zob. sprawa_korespondencja_zalaczniki_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  wysylka_id TEXT REFERENCES sprawa_korespondencja_wysylki (id),
  wplyw_id TEXT REFERENCES sprawa_korespondencja_wplywy (id),
  dokument_id TEXT NOT NULL REFERENCES sprawa_dokumenty_wpisy (id),

  CONSTRAINT zalacznik_wysylki_albo_wplywu CHECK ((wysylka_id IS NULL) <> (wplyw_id IS NULL))
) STRICT;

-- Reguły wysłania sprawdzają zmianę stanu, a decyzję i ślad wysyłki wpisuje się raz, więc wysyłka
-- nie może powstać od razu wysłana ani z decyzją.
CREATE TRIGGER sprawa_korespondencja_wysylka_powstaje_jako_projekt
BEFORE INSERT ON sprawa_korespondencja_wysylki
WHEN NEW.stan <> 'projekt'
  OR NEW.decyzja_id IS NOT NULL
  OR NEW.wyslano IS NOT NULL
  OR NEW.doreczono IS NOT NULL
  OR NEW.dowod_doreczenia IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'wysyłka powstaje jako projekt, bez decyzji i śladu wysłania');
END;

CREATE TRIGGER sprawa_korespondencja_wysylka_tylko_naprzod
BEFORE UPDATE OF stan ON sprawa_korespondencja_wysylki
WHEN NEW.stan IS NOT OLD.stan
  AND NOT (
    (OLD.stan = 'projekt' AND NEW.stan IN ('czeka_na_zgode', 'wyslana', 'anulowana'))
    OR (OLD.stan = 'czeka_na_zgode' AND NEW.stan IN ('wyslana', 'anulowana'))
    OR (OLD.stan = 'wyslana' AND NEW.stan IN ('doreczona', 'zwrocona'))
  )
BEGIN
  SELECT RAISE(ABORT, 'wysyłka idzie tylko naprzód: projekt, czeka na zgodę, wysłana, doręczona albo zwrócona; anuluje się tylko niewysłaną');
END;

-- Pismo spoza wzorów wymaga zgody człowieka (D15), więc z projektu idzie najpierw do zgody,
-- gdzie jego treść jest już zamrożona.
CREATE TRIGGER sprawa_korespondencja_pismo_spoza_wzorow_czeka_na_zgode
BEFORE UPDATE OF stan ON sprawa_korespondencja_wysylki
WHEN NEW.stan = 'wyslana'
  AND OLD.stan = 'projekt'
  AND NEW.wzor IS NULL
BEGIN
  SELECT RAISE(ABORT, 'pismo spoza wzorów przed wysłaniem czeka na zgodę');
END;

-- Wysyłka, która czekała na zgodę, wychodzi tylko za zgodą człowieka (D15).
CREATE TRIGGER sprawa_korespondencja_wysylka_za_zgoda_wymaga_decyzji
BEFORE UPDATE OF stan ON sprawa_korespondencja_wysylki
WHEN NEW.stan = 'wyslana'
  AND OLD.stan = 'czeka_na_zgode'
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS decyzja
    WHERE decyzja.id = NEW.decyzja_id
      AND decyzja.rodzaj = 'wyslanie'
      AND decyzja.dotyczy_tabela = 'sprawa_korespondencja_wysylki'
      AND decyzja.dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'wysyłka za zgodą wychodzi tylko z zatwierdzoną decyzją wyslanie, która ją wskazuje');
END;

-- Prośba o zgodę na wysyłkę opisuje treść zamrożoną w stanie „czeka_na_zgode”; decyzja o projekcie
-- dotyczyłaby treści, którą agent może jeszcze zmienić.
CREATE TRIGGER sprawa_korespondencja_prosba_o_wyslanie_przy_czekajacej
BEFORE INSERT ON sprawa_decyzje_wpisy
WHEN NEW.rodzaj = 'wyslanie'
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_korespondencja_wysylki
    WHERE NEW.dotyczy_tabela = 'sprawa_korespondencja_wysylki'
      AND id = NEW.dotyczy_id
      AND stan = 'czeka_na_zgode'
  )
BEGIN
  SELECT RAISE(ABORT, 'o decyzję wyslanie prosi się dla wysyłki, która czeka na zgodę');
END;

-- Decyzja wysyłki to zatwierdzona decyzja wyslanie, która wskazuje tę wysyłkę, przy każdym wpisie,
-- także przy wysłaniu prosto z projektu. Wysyłka z projektu nie ma jeszcze takiej decyzji, bo
-- o decyzję prosi się w stanie „czeka_na_zgode”, więc wychodzi bez niej.
CREATE TRIGGER sprawa_korespondencja_decyzja_wskazuje_wysylke
BEFORE UPDATE OF decyzja_id ON sprawa_korespondencja_wysylki
WHEN NEW.decyzja_id IS NOT NULL
  AND NEW.decyzja_id IS NOT OLD.decyzja_id
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS decyzja
    WHERE decyzja.id = NEW.decyzja_id
      AND decyzja.rodzaj = 'wyslanie'
      AND decyzja.dotyczy_tabela = 'sprawa_korespondencja_wysylki'
      AND decyzja.dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'decyzja wysyłki to zatwierdzona decyzja wyslanie, która ją wskazuje');
END;

-- Zanim biuro wystąpi w imieniu uczestnika, baza porównuje czynność z zakresem i terminem
-- umocowania biura od tego uczestnika. Czynność poza zakresem wymaga nowego pełnomocnictwa,
-- a nie zgody człowieka.
CREATE TRIGGER sprawa_korespondencja_w_imieniu_uczestnika_z_umocowaniem
BEFORE UPDATE OF stan ON sprawa_korespondencja_wysylki
WHEN NEW.stan = 'wyslana'
  AND OLD.stan IN ('projekt', 'czeka_na_zgode')
  AND NEW.w_imieniu = 'uczestnik'
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_umocowania_wpisy AS umocowanie
    JOIN sprawa_umocowania_czynnosci AS czynnosc ON czynnosc.umocowanie_id = umocowanie.id
    WHERE umocowanie.id = NEW.umocowanie_id
      AND umocowanie.mocodawca_uczestnik_id = NEW.w_imieniu_uczestnik_id
      AND umocowanie.umocowane_biuro = 1
      AND umocowanie.wygaslo IS NULL
      AND umocowanie.udzielono <= substr(NEW.wyslano, 1, 10)
      AND (umocowanie.wazne_do IS NULL OR umocowanie.wazne_do >= substr(NEW.wyslano, 1, 10))
      AND czynnosc.czynnosc = NEW.czynnosc
  )
BEGIN
  SELECT RAISE(ABORT, 'pismo w imieniu uczestnika wymaga jego niewygasłego umocowania biura, udzielonego najpóźniej w dniu wysłania i ważnego w tym dniu, które obejmuje tę czynność');
END;

-- Po wyjściu z projektu treść wysyłki jest tym, na co człowiek się zgodził, albo tym, co adresat
-- dostał, a podpis decyzji obejmuje skrót wiersza. Reguły umocowania i zgody sprawdzają te kolumny
-- w chwili wysłania, więc późniejsza zmiana przepisałaby ślad wysyłki.
CREATE TRIGGER sprawa_korespondencja_tresc_poza_projektem_niezmienna
BEFORE UPDATE ON sprawa_korespondencja_wysylki
WHEN OLD.stan <> 'projekt'
  AND (
    NEW.id IS NOT OLD.id
    OR NEW.adresat_uczestnik_id IS NOT OLD.adresat_uczestnik_id
    OR NEW.adresat_rodzaj IS NOT OLD.adresat_rodzaj
    OR NEW.adresat_nazwa IS NOT OLD.adresat_nazwa
    OR NEW.w_imieniu IS NOT OLD.w_imieniu
    OR NEW.w_imieniu_uczestnik_id IS NOT OLD.w_imieniu_uczestnik_id
    OR NEW.umocowanie_id IS NOT OLD.umocowanie_id
    OR NEW.czynnosc IS NOT OLD.czynnosc
    OR NEW.kanal IS NOT OLD.kanal
    OR NEW.zamawia_dokument_id IS NOT OLD.zamawia_dokument_id
    OR NEW.sciezka IS NOT OLD.sciezka
    OR NEW.skrot IS NOT OLD.skrot
    OR NEW.informacja_rodo IS NOT OLD.informacja_rodo
    OR NEW.ujawnia_agenta IS NOT OLD.ujawnia_agenta
  )
BEGIN
  SELECT RAISE(ABORT, 'treść wysyłki zmienia się tylko w projekcie; potem zmienia się tylko stan, chwile i dowód doręczenia');
END;

-- Ślad wysłania i doręczenia oraz decyzję wpisuje się raz, przy przejściu stanu.
CREATE TRIGGER sprawa_korespondencja_slad_wysylki_raz
BEFORE UPDATE ON sprawa_korespondencja_wysylki
WHEN (OLD.decyzja_id IS NOT NULL AND NEW.decyzja_id IS NOT OLD.decyzja_id)
  OR (OLD.wyslano IS NOT NULL AND NEW.wyslano IS NOT OLD.wyslano)
  OR (OLD.doreczono IS NOT NULL AND NEW.doreczono IS NOT OLD.doreczono)
  OR (OLD.dowod_doreczenia IS NOT NULL AND NEW.dowod_doreczenia IS NOT OLD.dowod_doreczenia)
  OR (OLD.odnosnik_zewnetrzny IS NOT NULL AND NEW.odnosnik_zewnetrzny IS NOT OLD.odnosnik_zewnetrzny)
BEGIN
  SELECT RAISE(ABORT, 'decyzję, chwile wysłania i doręczenia, dowód doręczenia i odnośnik wysyłki wpisuje się raz');
END;

-- Od wzoru zależy, czy pismo wymaga zgody człowieka, więc wzór i jego wersja nie zmieniają się od
-- wpisu, także w projekcie. Inny wzór to nowy projekt.
CREATE TRIGGER sprawa_korespondencja_wzor_niezmienny
BEFORE UPDATE OF wzor, wzor_wersja ON sprawa_korespondencja_wysylki
WHEN NEW.wzor IS NOT OLD.wzor
  OR NEW.wzor_wersja IS NOT OLD.wzor_wersja
BEGIN
  SELECT RAISE(ABORT, 'wzór wysyłki się nie zmienia; inny wzór to nowy projekt');
END;

-- Załączniki są częścią treści wysyłki, więc dołącza się je tylko do projektu.
CREATE TRIGGER sprawa_korespondencja_zalacznik_tylko_w_projekcie
BEFORE INSERT ON sprawa_korespondencja_zalaczniki
WHEN NEW.wysylka_id IS NOT NULL
  AND (
    SELECT stan <> 'projekt'
    FROM sprawa_korespondencja_wysylki
    WHERE id = NEW.wysylka_id
  )
BEGIN
  SELECT RAISE(ABORT, 'załączniki dołącza się tylko do projektu wysyłki');
END;

CREATE TRIGGER sprawa_korespondencja_zalaczniki_bez_zmian
BEFORE UPDATE ON sprawa_korespondencja_zalaczniki
BEGIN
  SELECT RAISE(ABORT, 'załącznika się nie zmienia; inny dokument to nowy wiersz');
END;

-- Wpływ jest zapisem tego, co przyszło, kiedy i od kogo.
CREATE TRIGGER sprawa_korespondencja_wplywy_bez_zmian
BEFORE UPDATE ON sprawa_korespondencja_wplywy
BEGIN
  SELECT RAISE(ABORT, 'wpływów się nie zmienia');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_korespondencja_wysylki_bez_zastepowania
BEFORE INSERT ON sprawa_korespondencja_wysylki
WHEN EXISTS (
  SELECT 1
  FROM sprawa_korespondencja_wysylki
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_korespondencja_wysylki_bez_usuwania
BEFORE DELETE ON sprawa_korespondencja_wysylki
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_korespondencja_wplywy_bez_zastepowania
BEFORE INSERT ON sprawa_korespondencja_wplywy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_korespondencja_wplywy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_korespondencja_wplywy_bez_usuwania
BEFORE DELETE ON sprawa_korespondencja_wplywy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER sprawa_korespondencja_zalaczniki_bez_zastepowania
BEFORE INSERT ON sprawa_korespondencja_zalaczniki
WHEN EXISTS (
  SELECT 1
  FROM sprawa_korespondencja_zalaczniki
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_korespondencja_zalaczniki_bez_usuwania
BEFORE DELETE ON sprawa_korespondencja_zalaczniki
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
