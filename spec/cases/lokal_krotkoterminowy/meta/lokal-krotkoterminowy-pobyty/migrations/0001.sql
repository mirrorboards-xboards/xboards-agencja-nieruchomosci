-- Pobyty gości: rezerwacja z kanału, pieniądze pobytu, opłacenie, przebieg, zmiany, zgłoszenia,
-- wyjątki od regulaminu, opinie, zwroty, rekompensaty i szkody, a także skargi sąsiadów.
-- Rezerwacje i pieniądze pisze Zarządca obiektu, zgłoszenia, wyjątki i opinie Konsjerż. Rozmowy
-- z gośćmi zostają w kanale; teczka zapisuje ich skutki.
--
-- Gość jest uczestnikiem rdzenia z rolą gosc i danymi z kanału. Innych danych gościa teczka nie
-- zbiera, dopóki biuro nie ustali obowiązków, które w „Agencji nieruchomości” są sprawami otwartymi (opłata
-- miejscowa, meldunek); miejscem na nie są liczba gości i noce pobytu.
--
-- Reguły, które zależą od późniejszych rodzajów, leżą w ich migracjach: kanał rezerwacji ma ofertę
-- w kalendarzu (lokal-krotkoterminowy-kalendarz), a pieniądze pobytu ujęte w wysłanym zestawieniu
-- się nie zmieniają (lokal-krotkoterminowy-zestawienia).
CREATE TABLE lokal_krotkoterminowy_pobyty_wpisy (
  id TEXT PRIMARY KEY,
  -- Rezerwacja: kanał, numer w kanale, chwila przyjęcia, dni przyjazdu i wyjazdu, liczba gości.
  kanal TEXT NOT NULL CHECK (kanal <> '' AND kanal NOT GLOB '*[^a-z_]*'),
  numer_rezerwacji TEXT NOT NULL CHECK (numer_rezerwacji <> ''),
  przyjeto TEXT NOT NULL,
  przyjazd TEXT NOT NULL CHECK (przyjazd = date(przyjazd)),
  wyjazd TEXT NOT NULL CHECK (wyjazd = date(wyjazd)),
  gosci INTEGER NOT NULL CHECK (gosci > 0),
  gosc_uczestnik_id TEXT NOT NULL REFERENCES sprawa_uczestnicy_wpisy (id),
  -- Pieniądze w groszach z rozliczenia na koncie kanału: zapłata gościa, opłata za sprzątanie,
  -- prowizja kanału, wypłata kanału z dniem wypłaty i chwila rozliczenia w teczce. Przy sprzedaży
  -- przez stronę biura wypłata to wpływ na rachunek biura.
  kwota_goscia INTEGER NOT NULL CHECK (kwota_goscia >= 0),
  oplata_sprzatanie INTEGER NOT NULL CHECK (oplata_sprzatanie >= 0),
  prowizja_kanalu INTEGER CHECK (prowizja_kanalu >= 0),
  wyplata_kanalu INTEGER CHECK (wyplata_kanalu >= 0),
  wyplacono TEXT,
  rozliczono TEXT,
  oplacona INTEGER NOT NULL CHECK (oplacona IN (0, 1)),
  -- Przebieg: kiedy Konsjerż wysłał instrukcję wejścia, stan i rzeczy pozostawione.
  instrukcja_wyslano TEXT,
  stan TEXT NOT NULL CHECK (stan IN ('potwierdzona', 'kolizja', 'trwa', 'zakonczona', 'odwolana')),
  -- Zgoda człowieka na noce po dacie końca umowy o zarządzanie.
  po_koncu_decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),
  rzeczy_pozostawione TEXT,

  CONSTRAINT wyjazd_po_przyjezdzie CHECK (wyjazd > przyjazd),
  -- Konsjerż wysyła instrukcję wejścia tylko do opłaconej rezerwacji.
  CONSTRAINT instrukcja_tylko_do_oplaconej CHECK (instrukcja_wyslano IS NULL OR oplacona = 1),
  CONSTRAINT rozliczenie_w_komplecie CHECK (
    rozliczono IS NULL
    OR (prowizja_kanalu IS NOT NULL AND wyplata_kanalu IS NOT NULL AND wyplacono IS NOT NULL)
  ),

  UNIQUE (kanal, numer_rezerwacji)
) STRICT;

-- Zmiany pobytu: termin, liczba gości, cena albo odwołanie, z przyczyną i tym, kto zdecydował.
CREATE TABLE lokal_krotkoterminowy_pobyty_zmiany (
  -- Klucz nadany przez bazę jest dodatni; zob. lokal_krotkoterminowy_pobyty_zmiany_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  pobyt_id TEXT NOT NULL REFERENCES lokal_krotkoterminowy_pobyty_wpisy (id),
  chwila TEXT NOT NULL,
  co TEXT NOT NULL CHECK (co IN ('termin', 'liczba_gosci', 'cena', 'odwolanie')),
  przyczyna TEXT NOT NULL CHECK (przyczyna <> ''),
  zdecydowal TEXT NOT NULL
) STRICT;

-- Zgłoszenia gościa: problem z godziną, opisem i tym, komu Konsjerż go przekazał. Zagrożenie
-- (pożar, gaz, zdrowie, przemoc) ma zapisane, kogo Konsjerż zawiadomił po podaniu numeru 112.
-- Zlecenie naprawy wskazuje zgłoszenie, więc ma UUID.
CREATE TABLE lokal_krotkoterminowy_pobyty_zgloszenia (
  id TEXT PRIMARY KEY,
  pobyt_id TEXT NOT NULL REFERENCES lokal_krotkoterminowy_pobyty_wpisy (id),
  chwila TEXT NOT NULL,
  opis TEXT NOT NULL CHECK (opis <> ''),
  -- Komu Konsjerż przekazał zgłoszenie, np. „Koordynator operacji, 14:05”.
  przekazano TEXT,
  zagrozenie INTEGER NOT NULL CHECK (zagrozenie IN (0, 1)),
  zawiadomiono TEXT,

  CONSTRAINT zagrozenie_ma_zawiadomienie CHECK (zagrozenie = 0 OR zawiadomiono IS NOT NULL)
) STRICT;

-- Wyjątki od regulaminu, np. późne wymeldowanie albo zwierzę, każdy za zgodą człowieka.
CREATE TABLE lokal_krotkoterminowy_pobyty_wyjatki (
  id TEXT PRIMARY KEY,
  pobyt_id TEXT NOT NULL REFERENCES lokal_krotkoterminowy_pobyty_wpisy (id),
  opis TEXT NOT NULL CHECK (opis <> ''),
  decyzja_id TEXT NOT NULL REFERENCES sprawa_decyzje_wpisy (id)
) STRICT;

-- Opinia gościa z oceną w skali 1–5 i odpowiedzią wysłaną z teczki.
CREATE TABLE lokal_krotkoterminowy_pobyty_opinie (
  id TEXT PRIMARY KEY,
  pobyt_id TEXT NOT NULL REFERENCES lokal_krotkoterminowy_pobyty_wpisy (id),
  ocena INTEGER NOT NULL CHECK (ocena BETWEEN 1 AND 5),
  opublikowano TEXT NOT NULL,
  odpowiedz_wysylka_id TEXT REFERENCES sprawa_korespondencja_wysylki (id),

  UNIQUE (pobyt_id)
) STRICT;

-- Pieniądze pobytu poza rezerwacją, w groszach: zwrot dla gościa, rekompensata, roszczenie wobec
-- gościa, wniosek w procedurze kanału i kwota odzyskana. Zestawienie wskazuje je jako źródło
-- pozycji, więc mają UUID.
CREATE TABLE lokal_krotkoterminowy_pobyty_pieniadze (
  id TEXT PRIMARY KEY,
  pobyt_id TEXT NOT NULL REFERENCES lokal_krotkoterminowy_pobyty_wpisy (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('zwrot', 'rekompensata', 'roszczenie', 'wniosek_kanalu', 'odzysk')),
  -- Zwrot według zasad odwołania w kanale i regulaminu lokalu nie wymaga zgody człowieka.
  wedlug_zasad_kanalu INTEGER NOT NULL CHECK (wedlug_zasad_kanalu IN (0, 1)),
  kwota INTEGER NOT NULL CHECK (kwota > 0),
  chwila TEXT NOT NULL,
  opis TEXT,
  decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),

  CONSTRAINT wedlug_zasad_kanalu_tylko_zwrot CHECK (wedlug_zasad_kanalu = 0 OR rodzaj = 'zwrot')
) STRICT;

-- Skargi sąsiadów albo wspólnoty: kto, czego dotyczy, którego pobytu, i odpowiedź z teczki.
CREATE TABLE lokal_krotkoterminowy_pobyty_skargi (
  -- Klucz nadany przez bazę jest dodatni; zob. lokal_krotkoterminowy_pobyty_skargi_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  chwila TEXT NOT NULL,
  kto TEXT NOT NULL CHECK (kto <> ''),
  tresc TEXT NOT NULL CHECK (tresc <> ''),
  pobyt_id TEXT REFERENCES lokal_krotkoterminowy_pobyty_wpisy (id),
  odpowiedz_wysylka_id TEXT REFERENCES sprawa_korespondencja_wysylki (id)
) STRICT;

-- Reguły przebiegu sprawdzają zmianę stanu, a instrukcję i rozliczenie wpisuje się później, więc
-- pobyt powstaje potwierdzony, bez nich.
CREATE TRIGGER lokal_krotkoterminowy_pobyty_powstaje_potwierdzony
BEFORE INSERT ON lokal_krotkoterminowy_pobyty_wpisy
WHEN NEW.stan <> 'potwierdzona'
  OR NEW.instrukcja_wyslano IS NOT NULL
  OR NEW.rozliczono IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'pobyt powstaje potwierdzony, bez instrukcji wejścia i rozliczenia');
END;

-- Rezerwację przyjmuje się według obowiązującej umowy o zarządzanie i nie przed praca_od: przed
-- końcem terminu na odstąpienie tylko na żądanie właściciela konsumenta (R8).
CREATE TRIGGER lokal_krotkoterminowy_pobyty_przy_umowie_od_pracy_od
BEFORE INSERT ON lokal_krotkoterminowy_pobyty_wpisy
WHEN NOT coalesce(
  (
    SELECT
      umowa.stan = 'zawarta'
      AND NEW.przyjeto >= umowa.praca_od
    FROM lokal_krotkoterminowy_umowa_lokalu AS umowa
  ),
  0
)
BEGIN
  SELECT RAISE(ABORT, 'rezerwację przyjmuje się przy zawartej umowie o zarządzanie, nie wcześniej niż praca_od');
END;

CREATE TRIGGER lokal_krotkoterminowy_pobyty_gosci_do_max
BEFORE INSERT ON lokal_krotkoterminowy_pobyty_wpisy
WHEN NOT coalesce(
  NEW.gosci <= (
    SELECT max_gosci
    FROM lokal_krotkoterminowy_umowa_warunki_biezace
  ),
  0
)
BEGIN
  SELECT RAISE(ABORT, 'liczba gości nie przekracza największej liczby gości z bieżących warunków umowy');
END;

CREATE TRIGGER lokal_krotkoterminowy_pobyty_zmienieni_gosci_do_max
BEFORE UPDATE OF gosci ON lokal_krotkoterminowy_pobyty_wpisy
WHEN NEW.gosci > OLD.gosci
  AND NOT coalesce(
    NEW.gosci <= (
      SELECT max_gosci
      FROM lokal_krotkoterminowy_umowa_warunki_biezace
    ),
    0
  )
BEGIN
  SELECT RAISE(ABORT, 'liczba gości nie przekracza największej liczby gości z bieżących warunków umowy');
END;

CREATE TRIGGER lokal_krotkoterminowy_pobyty_gosc_z_rola
BEFORE INSERT ON lokal_krotkoterminowy_pobyty_wpisy
WHEN NOT EXISTS (
  SELECT 1
  FROM sprawa_uczestnicy_role
  WHERE uczestnik_id = NEW.gosc_uczestnik_id
    AND rola = 'gosc'
    AND zakonczono IS NULL
)
BEGIN
  SELECT RAISE(ABORT, 'pobyt wskazuje uczestnika z rolą gosc');
END;

-- Każda rezerwacja na noce po dacie końca umowy wymaga zgody człowieka. Ostatnia noc pobytu to
-- dzień przed wyjazdem; umowa bez daty końca nie ma takich nocy.
CREATE TRIGGER lokal_krotkoterminowy_pobyty_po_koncu_umowy_za_zgoda
BEFORE INSERT ON lokal_krotkoterminowy_pobyty_wpisy
WHEN date(NEW.wyjazd, '-1 day') > (
    SELECT koniec
    FROM lokal_krotkoterminowy_umowa_lokalu
  )
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone
    WHERE id = NEW.po_koncu_decyzja_id
      AND rodzaj = 'inna'
      AND dotyczy_tabela = 'lokal_krotkoterminowy_pobyty_wpisy'
      AND dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'pobyt z nocami po dacie końca umowy wskazuje zatwierdzoną decyzję inna, która go wskazuje');
END;

CREATE TRIGGER lokal_krotkoterminowy_pobyty_zmieniony_po_koncu_umowy_za_zgoda
BEFORE UPDATE OF przyjazd, wyjazd, po_koncu_decyzja_id ON lokal_krotkoterminowy_pobyty_wpisy
WHEN date(NEW.wyjazd, '-1 day') > (
    SELECT koniec
    FROM lokal_krotkoterminowy_umowa_lokalu
  )
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone
    WHERE id = NEW.po_koncu_decyzja_id
      AND rodzaj = 'inna'
      AND dotyczy_tabela = 'lokal_krotkoterminowy_pobyty_wpisy'
      AND dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'pobyt z nocami po dacie końca umowy wskazuje zatwierdzoną decyzję inna, która go wskazuje');
END;

-- Pobyt: potwierdzony, w kolizji (dwa kanały sprzedały tę samą noc; który zostaje, decyduje
-- człowiek), trwa, zakończony albo odwołany.
CREATE TRIGGER lokal_krotkoterminowy_pobyty_przejscia_stanu
BEFORE UPDATE OF stan ON lokal_krotkoterminowy_pobyty_wpisy
WHEN NEW.stan IS NOT OLD.stan
  AND NOT (
    (OLD.stan = 'potwierdzona' AND NEW.stan IN ('kolizja', 'trwa', 'odwolana'))
    OR (OLD.stan = 'kolizja' AND NEW.stan IN ('potwierdzona', 'odwolana'))
    OR (OLD.stan = 'trwa' AND NEW.stan = 'zakonczona')
  )
BEGIN
  SELECT RAISE(ABORT, 'pobyt przechodzi: potwierdzona, trwa, zakończona; kolizja wraca do potwierdzonej albo kończy się odwołaniem');
END;

-- Rezerwację wskazuje kanał i jego numer, a gościa dane z kanału; nowa rezerwacja to nowy wiersz.
CREATE TRIGGER lokal_krotkoterminowy_pobyty_rezerwacja_niezmienna
BEFORE UPDATE ON lokal_krotkoterminowy_pobyty_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.kanal IS NOT OLD.kanal
  OR NEW.numer_rezerwacji IS NOT OLD.numer_rezerwacji
  OR NEW.przyjeto IS NOT OLD.przyjeto
  OR NEW.gosc_uczestnik_id IS NOT OLD.gosc_uczestnik_id
BEGIN
  SELECT RAISE(ABORT, 'kanał, numer rezerwacji, chwila przyjęcia i gość się nie zmieniają; nowa rezerwacja to nowy wiersz');
END;

-- Po zakończeniu albo odwołaniu termin, goście i instrukcja są już historią pobytu.
CREATE TRIGGER lokal_krotkoterminowy_pobyty_przebieg_po_koncu_niezmienny
BEFORE UPDATE ON lokal_krotkoterminowy_pobyty_wpisy
WHEN OLD.stan IN ('zakonczona', 'odwolana')
  AND (
    NEW.przyjazd IS NOT OLD.przyjazd
    OR NEW.wyjazd IS NOT OLD.wyjazd
    OR NEW.gosci IS NOT OLD.gosci
    OR NEW.oplacona IS NOT OLD.oplacona
    OR NEW.instrukcja_wyslano IS NOT OLD.instrukcja_wyslano
    OR NEW.po_koncu_decyzja_id IS NOT OLD.po_koncu_decyzja_id
  )
BEGIN
  SELECT RAISE(ABORT, 'termin, goście, opłacenie i instrukcja zakończonego albo odwołanego pobytu się nie zmieniają');
END;

-- Instrukcję wysłaną gościowi i rozliczenie z konta kanału wpisuje się raz; na rozliczeniu stoją
-- pozycje zestawienia dla właściciela.
CREATE TRIGGER lokal_krotkoterminowy_pobyty_instrukcja_i_rozliczenie_raz
BEFORE UPDATE ON lokal_krotkoterminowy_pobyty_wpisy
WHEN (OLD.instrukcja_wyslano IS NOT NULL AND NEW.instrukcja_wyslano IS NOT OLD.instrukcja_wyslano)
  OR (
    OLD.rozliczono IS NOT NULL
    AND (
      NEW.rozliczono IS NOT OLD.rozliczono
      OR NEW.kwota_goscia IS NOT OLD.kwota_goscia
      OR NEW.oplata_sprzatanie IS NOT OLD.oplata_sprzatanie
      OR NEW.prowizja_kanalu IS NOT OLD.prowizja_kanalu
      OR NEW.wyplata_kanalu IS NOT OLD.wyplata_kanalu
      OR NEW.wyplacono IS NOT OLD.wyplacono
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'instrukcję wejścia i rozliczenie pobytu wpisuje się raz; potem pieniądze pobytu się nie zmieniają');
END;

-- Ostatnie rozliczenie przychodzi po ostatnim pobycie: żaden pobyt nie czeka ani nie trwa.
CREATE TRIGGER lokal_krotkoterminowy_pobyty_rozliczenie_po_ostatnim_pobycie
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap = 'ostatnie_rozliczenie'
  AND EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_pobyty_wpisy
    WHERE stan IN ('potwierdzona', 'kolizja', 'trwa')
  )
BEGIN
  SELECT RAISE(ABORT, 'etap ostatnie_rozliczenie przychodzi, gdy żaden pobyt nie jest potwierdzony, w kolizji ani w toku');
END;

CREATE TRIGGER lokal_krotkoterminowy_pobyty_zmiany_bez_zmian
BEFORE UPDATE ON lokal_krotkoterminowy_pobyty_zmiany
BEGIN
  SELECT RAISE(ABORT, 'zmiany pobytu się nie zmienia; sprostowanie to nowa zmiana');
END;

-- Zgłoszenie jest zapisem tego, co gość powiedział; dopisuje się tylko, komu je przekazano, raz.
CREATE TRIGGER lokal_krotkoterminowy_pobyty_zgloszenie_zmienia_tylko_przekazanie
BEFORE UPDATE ON lokal_krotkoterminowy_pobyty_zgloszenia
WHEN OLD.przekazano IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.pobyt_id IS NOT OLD.pobyt_id
  OR NEW.chwila IS NOT OLD.chwila
  OR NEW.opis IS NOT OLD.opis
  OR NEW.zagrozenie IS NOT OLD.zagrozenie
  OR NEW.zawiadomiono IS NOT OLD.zawiadomiono
BEGIN
  SELECT RAISE(ABORT, 'zgłoszenia się nie zmienia; dopisuje się tylko, komu je przekazano, raz');
END;

-- Wyjątek od regulaminu wymaga zgody człowieka; Konsjerż go nie obiecuje.
CREATE TRIGGER lokal_krotkoterminowy_pobyty_wyjatek_za_zgoda
BEFORE INSERT ON lokal_krotkoterminowy_pobyty_wyjatki
WHEN NOT EXISTS (
  SELECT 1
  FROM sprawa_decyzje_zatwierdzone
  WHERE id = NEW.decyzja_id
    AND rodzaj = 'inna'
    AND dotyczy_tabela = 'lokal_krotkoterminowy_pobyty_wyjatki'
    AND dotyczy_id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wyjątek od regulaminu wskazuje zatwierdzoną decyzję inna, która go wskazuje');
END;

CREATE TRIGGER lokal_krotkoterminowy_pobyty_wyjatki_bez_zmian
BEFORE UPDATE ON lokal_krotkoterminowy_pobyty_wyjatki
BEGIN
  SELECT RAISE(ABORT, 'wyjątku się nie zmienia; inny wyjątek to nowy wiersz za nową zgodą');
END;

-- Rekompensata, roszczenie, wniosek w procedurze kanału i zwrot ponad zasady kanału wymagają zgody
-- człowieka. Zwrot według zasad kanału i kwota odzyskana jej nie wymagają.
CREATE TRIGGER lokal_krotkoterminowy_pobyty_pieniadze_za_zgoda
BEFORE INSERT ON lokal_krotkoterminowy_pobyty_pieniadze
WHEN NEW.rodzaj <> 'odzysk'
  AND NOT (NEW.rodzaj = 'zwrot' AND NEW.wedlug_zasad_kanalu = 1)
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone
    WHERE id = NEW.decyzja_id
      AND rodzaj = 'inna'
      AND dotyczy_tabela = 'lokal_krotkoterminowy_pobyty_pieniadze'
      AND dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'rekompensata, roszczenie, wniosek kanału i zwrot ponad zasady kanału wskazują zatwierdzoną decyzję inna, która je wskazuje');
END;

CREATE TRIGGER lokal_krotkoterminowy_pobyty_pieniadze_bez_zmian
BEFORE UPDATE ON lokal_krotkoterminowy_pobyty_pieniadze
BEGIN
  SELECT RAISE(ABORT, 'pieniędzy pobytu się nie zmienia; korekta to nowy wiersz');
END;

-- Odpowiedź na opinię i na skargę wskazuje się, zanim wyjdzie: tylko wtedy reguła zgody może jej
-- dopilnować. Odpowiedź wskazuje się raz.
CREATE TRIGGER lokal_krotkoterminowy_pobyty_opinia_wskazuje_odpowiedz_przed_wyslaniem
BEFORE INSERT ON lokal_krotkoterminowy_pobyty_opinie
WHEN NEW.odpowiedz_wysylka_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_korespondencja_wysylki
    WHERE id = NEW.odpowiedz_wysylka_id
      AND stan IN ('projekt', 'czeka_na_zgode')
  )
BEGIN
  SELECT RAISE(ABORT, 'odpowiedź na opinię albo skargę wskazuje wysyłkę, zanim wyjdzie');
END;

CREATE TRIGGER lokal_krotkoterminowy_pobyty_opinia_zmienia_tylko_odpowiedz
BEFORE UPDATE ON lokal_krotkoterminowy_pobyty_opinie
WHEN OLD.odpowiedz_wysylka_id IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.pobyt_id IS NOT OLD.pobyt_id
  OR NEW.ocena IS NOT OLD.ocena
  OR NEW.opublikowano IS NOT OLD.opublikowano
  OR NOT EXISTS (
    SELECT 1
    FROM sprawa_korespondencja_wysylki
    WHERE id = NEW.odpowiedz_wysylka_id
      AND stan IN ('projekt', 'czeka_na_zgode')
  )
BEGIN
  SELECT RAISE(ABORT, 'odpowiedź na opinię albo skargę wskazuje wysyłkę, zanim wyjdzie; opinii się nie zmienia, a odpowiedź wskazuje się raz');
END;

CREATE TRIGGER lokal_krotkoterminowy_pobyty_skarga_wskazuje_odpowiedz_przed_wyslaniem
BEFORE INSERT ON lokal_krotkoterminowy_pobyty_skargi
WHEN NEW.odpowiedz_wysylka_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_korespondencja_wysylki
    WHERE id = NEW.odpowiedz_wysylka_id
      AND stan IN ('projekt', 'czeka_na_zgode')
  )
BEGIN
  SELECT RAISE(ABORT, 'odpowiedź na opinię albo skargę wskazuje wysyłkę, zanim wyjdzie');
END;

CREATE TRIGGER lokal_krotkoterminowy_pobyty_skarga_zmienia_tylko_odpowiedz
BEFORE UPDATE ON lokal_krotkoterminowy_pobyty_skargi
WHEN OLD.odpowiedz_wysylka_id IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.chwila IS NOT OLD.chwila
  OR NEW.kto IS NOT OLD.kto
  OR NEW.tresc IS NOT OLD.tresc
  OR NEW.pobyt_id IS NOT OLD.pobyt_id
  OR NOT EXISTS (
    SELECT 1
    FROM sprawa_korespondencja_wysylki
    WHERE id = NEW.odpowiedz_wysylka_id
      AND stan IN ('projekt', 'czeka_na_zgode')
  )
BEGIN
  SELECT RAISE(ABORT, 'odpowiedź na opinię albo skargę wskazuje wysyłkę, zanim wyjdzie; skargi się nie zmienia, a odpowiedź wskazuje się raz');
END;

-- Odpowiedź na opinię z oceną 3 albo niższą i odpowiedź na skargę wychodzą za zgodą człowieka,
-- więc przed wysłaniem czekają na zgodę (rdzeń wymaga wtedy decyzji wyslanie).
CREATE TRIGGER lokal_krotkoterminowy_pobyty_odpowiedz_za_zgoda
BEFORE UPDATE OF stan ON sprawa_korespondencja_wysylki
WHEN NEW.stan = 'wyslana'
  AND OLD.stan = 'projekt'
  AND (
    EXISTS (
      SELECT 1
      FROM lokal_krotkoterminowy_pobyty_opinie
      WHERE odpowiedz_wysylka_id = NEW.id
        AND ocena <= 3
    )
    OR EXISTS (
      SELECT 1
      FROM lokal_krotkoterminowy_pobyty_skargi
      WHERE odpowiedz_wysylka_id = NEW.id
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'odpowiedź na opinię z oceną 3 albo niższą i odpowiedź na skargę przed wysłaniem czekają na zgodę');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER lokal_krotkoterminowy_pobyty_wpisy_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_pobyty_wpisy
WHEN EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_pobyty_wpisy
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_pobyty_wpisy
    WHERE kanal = NEW.kanal
      AND numer_rezerwacji = NEW.numer_rezerwacji
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_pobyty_wpisy_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_pobyty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER lokal_krotkoterminowy_pobyty_zmiany_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_pobyty_zmiany
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_pobyty_zmiany
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_pobyty_zmiany_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_pobyty_zmiany
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER lokal_krotkoterminowy_pobyty_zgloszenia_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_pobyty_zgloszenia
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_pobyty_zgloszenia
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_pobyty_zgloszenia_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_pobyty_zgloszenia
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER lokal_krotkoterminowy_pobyty_wyjatki_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_pobyty_wyjatki
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_pobyty_wyjatki
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_pobyty_wyjatki_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_pobyty_wyjatki
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER lokal_krotkoterminowy_pobyty_opinie_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_pobyty_opinie
WHEN EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_pobyty_opinie
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_pobyty_opinie
    WHERE pobyt_id = NEW.pobyt_id
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_pobyty_opinie_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_pobyty_opinie
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER lokal_krotkoterminowy_pobyty_pieniadze_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_pobyty_pieniadze
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_pobyty_pieniadze
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_pobyty_pieniadze_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_pobyty_pieniadze
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER lokal_krotkoterminowy_pobyty_skargi_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_pobyty_skargi
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_pobyty_skargi
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_pobyty_skargi_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_pobyty_skargi
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
