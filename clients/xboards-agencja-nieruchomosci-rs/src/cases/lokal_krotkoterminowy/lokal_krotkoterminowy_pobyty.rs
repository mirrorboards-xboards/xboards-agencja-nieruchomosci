//! The kind `lokal-krotkoterminowy-pobyty` of the room `cases/lokal_krotkoterminowy`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "lokal-krotkoterminowy-pobyty",
    requires: &[
        &super::sprawa_decyzje::KIND,
        &super::sprawa_korespondencja::KIND,
        &super::sprawa_metryka::KIND,
        &super::sprawa_uczestnicy::KIND,
    ],
    migrations: &[
        r#"-- Pobyty gości: rezerwacja z kanału, pieniądze pobytu, opłacenie, przebieg, zmiany, zgłoszenia,
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
"#,
    ],
};

/// The values of column `pieniadze.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieniadzeRodzaj {
    Zwrot,
    Rekompensata,
    Roszczenie,
    WniosekKanalu,
    Odzysk,
}

impl PieniadzeRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[PieniadzeRodzaj] = &[
        PieniadzeRodzaj::Zwrot,
        PieniadzeRodzaj::Rekompensata,
        PieniadzeRodzaj::Roszczenie,
        PieniadzeRodzaj::WniosekKanalu,
        PieniadzeRodzaj::Odzysk,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            PieniadzeRodzaj::Zwrot => "zwrot",
            PieniadzeRodzaj::Rekompensata => "rekompensata",
            PieniadzeRodzaj::Roszczenie => "roszczenie",
            PieniadzeRodzaj::WniosekKanalu => "wniosek_kanalu",
            PieniadzeRodzaj::Odzysk => "odzysk",
        }
    }
}

impl xmeta_room::FromSqlValue for PieniadzeRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "zwrot" => Ok(PieniadzeRodzaj::Zwrot),
                    "rekompensata" => Ok(PieniadzeRodzaj::Rekompensata),
                    "roszczenie" => Ok(PieniadzeRodzaj::Roszczenie),
                    "wniosek_kanalu" => Ok(PieniadzeRodzaj::WniosekKanalu),
                    "odzysk" => Ok(PieniadzeRodzaj::Odzysk),
                    other => Err(format!("{other} is no value of PieniadzeRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for PieniadzeRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyStan {
    Potwierdzona,
    Kolizja,
    Trwa,
    Zakonczona,
    Odwolana,
}

impl WpisyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyStan] = &[
        WpisyStan::Potwierdzona,
        WpisyStan::Kolizja,
        WpisyStan::Trwa,
        WpisyStan::Zakonczona,
        WpisyStan::Odwolana,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyStan::Potwierdzona => "potwierdzona",
            WpisyStan::Kolizja => "kolizja",
            WpisyStan::Trwa => "trwa",
            WpisyStan::Zakonczona => "zakonczona",
            WpisyStan::Odwolana => "odwolana",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "potwierdzona" => Ok(WpisyStan::Potwierdzona),
                    "kolizja" => Ok(WpisyStan::Kolizja),
                    "trwa" => Ok(WpisyStan::Trwa),
                    "zakonczona" => Ok(WpisyStan::Zakonczona),
                    "odwolana" => Ok(WpisyStan::Odwolana),
                    other => Err(format!("{other} is no value of WpisyStan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyStan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `zmiany.co`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZmianyCo {
    Termin,
    LiczbaGosci,
    Cena,
    Odwolanie,
}

impl ZmianyCo {
    /// Every value, in the order of the catalog.
    pub const ALL: &[ZmianyCo] = &[
        ZmianyCo::Termin,
        ZmianyCo::LiczbaGosci,
        ZmianyCo::Cena,
        ZmianyCo::Odwolanie,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            ZmianyCo::Termin => "termin",
            ZmianyCo::LiczbaGosci => "liczba_gosci",
            ZmianyCo::Cena => "cena",
            ZmianyCo::Odwolanie => "odwolanie",
        }
    }
}

impl xmeta_room::FromSqlValue for ZmianyCo {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "termin" => Ok(ZmianyCo::Termin),
                    "liczba_gosci" => Ok(ZmianyCo::LiczbaGosci),
                    "cena" => Ok(ZmianyCo::Cena),
                    "odwolanie" => Ok(ZmianyCo::Odwolanie),
                    other => Err(format!("{other} is no value of ZmianyCo")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for ZmianyCo {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of field `stan` of `zmien_stan_pobytu`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stan {
    Potwierdzona,
    Kolizja,
    Trwa,
    Odwolana,
}

impl Stan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[Stan] = &[
        Stan::Potwierdzona,
        Stan::Kolizja,
        Stan::Trwa,
        Stan::Odwolana,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            Stan::Potwierdzona => "potwierdzona",
            Stan::Kolizja => "kolizja",
            Stan::Trwa => "trwa",
            Stan::Odwolana => "odwolana",
        }
    }
}

impl xmeta_room::FromSqlValue for Stan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "potwierdzona" => Ok(Stan::Potwierdzona),
                    "kolizja" => Ok(Stan::Kolizja),
                    "trwa" => Ok(Stan::Trwa),
                    "odwolana" => Ok(Stan::Odwolana),
                    other => Err(format!("{other} is no value of Stan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for Stan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `opinie_gosci`.
#[derive(Debug, Clone, PartialEq)]
pub struct OpinieGosciRow {
    pub id: String,
    pub pobyt_id: String,
    pub ocena: i64,
    pub opublikowano: String,
    pub odpowiedz_wysylka_id: Option<String>,
}

impl OpinieGosciRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(OpinieGosciRow {
            id: row.get("id")?,
            pobyt_id: row.get("pobyt_id")?,
            ocena: row.get("ocena")?,
            opublikowano: row.get("opublikowano")?,
            odpowiedz_wysylka_id: row.get("odpowiedz_wysylka_id")?,
        })
    }
}

/// Opinie gości w kolejności publikacji.
pub fn opinie_gosci(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<OpinieGosciRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_pobyty_opinie
ORDER BY opublikowano, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(OpinieGosciRow::from_row).collect()
}

/// The parameters of `wpisz_opinie_goscia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszOpinieGosciaParams {
    pub id: String,
    pub pobyt_id: String,
    pub ocena: i64,
    pub opublikowano: String,
}

/// Opinia gościa z oceną 1–5 i dniem publikacji.
pub fn wpisz_opinie_goscia(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszOpinieGosciaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_pobyty_opinie (id, pobyt_id, ocena, opublikowano)
VALUES (:id, :pobyt_id, :ocena, :opublikowano)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("pobyt_id", xmeta_room::ToSqlValue::to_sql_value(&params.pobyt_id)),
                    ("ocena", xmeta_room::ToSqlValue::to_sql_value(&params.ocena)),
                    (
                        "opublikowano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.opublikowano),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `wskaz_odpowiedz_na_opinie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WskazOdpowiedzNaOpinieParams {
    pub odpowiedz_wysylka_id: String,
    pub id: String,
}

/// Projekt odpowiedzi na opinię, zanim wyjdzie; przy ocenie 3 i niższej czeka on na zgodę.
pub fn wskaz_odpowiedz_na_opinie(
    room: &impl xmeta_room::MetaRoom,
    params: &WskazOdpowiedzNaOpinieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_pobyty_opinie
SET odpowiedz_wysylka_id = :odpowiedz_wysylka_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "odpowiedz_wysylka_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.odpowiedz_wysylka_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `pieniadze_pobytow`.
#[derive(Debug, Clone, PartialEq)]
pub struct PieniadzePobytowRow {
    pub id: String,
    pub pobyt_id: String,
    pub rodzaj: PieniadzeRodzaj,
    pub wedlug_zasad_kanalu: bool,
    pub kwota: i64,
    pub chwila: String,
    pub opis: Option<String>,
    pub decyzja_id: Option<String>,
}

impl PieniadzePobytowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PieniadzePobytowRow {
            id: row.get("id")?,
            pobyt_id: row.get("pobyt_id")?,
            rodzaj: row.get("rodzaj")?,
            wedlug_zasad_kanalu: row.get("wedlug_zasad_kanalu")?,
            kwota: row.get("kwota")?,
            chwila: row.get("chwila")?,
            opis: row.get("opis")?,
            decyzja_id: row.get("decyzja_id")?,
        })
    }
}

/// Zwroty, rekompensaty, roszczenia, wnioski kanału i odzyskane kwoty w kolejności czasu.
pub fn pieniadze_pobytow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PieniadzePobytowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_pobyty_pieniadze
ORDER BY chwila, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(PieniadzePobytowRow::from_row).collect()
}

/// The parameters of `wpisz_pieniadze_pobytu`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszPieniadzePobytuParams {
    pub id: String,
    pub pobyt_id: String,
    pub rodzaj: PieniadzeRodzaj,
    pub wedlug_zasad_kanalu: bool,
    pub kwota: i64,
    pub chwila: String,
    pub opis: Option<String>,
    pub decyzja_id: Option<String>,
}

/// Kwota poza rezerwacją; bez zgody człowieka tylko zwrot według zasad kanału i kwota odzyskana.
/// Identyfikator (UUID) nadaje aplikacja, bo wskazują go decyzja i pozycja zestawienia.
pub fn wpisz_pieniadze_pobytu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszPieniadzePobytuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_pobyty_pieniadze (id, pobyt_id, rodzaj, wedlug_zasad_kanalu, kwota,
  chwila, opis, decyzja_id)
VALUES (:id, :pobyt_id, :rodzaj, :wedlug_zasad_kanalu, :kwota, :chwila, :opis, :decyzja_id)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("pobyt_id", xmeta_room::ToSqlValue::to_sql_value(&params.pobyt_id)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    (
                        "wedlug_zasad_kanalu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wedlug_zasad_kanalu),
                    ),
                    ("kwota", xmeta_room::ToSqlValue::to_sql_value(&params.kwota)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    (
                        "decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_id),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `skargi_sasiadow`.
#[derive(Debug, Clone, PartialEq)]
pub struct SkargiSasiadowRow {
    pub id: i64,
    pub chwila: String,
    pub kto: String,
    pub tresc: String,
    pub pobyt_id: Option<String>,
    pub odpowiedz_wysylka_id: Option<String>,
}

impl SkargiSasiadowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(SkargiSasiadowRow {
            id: row.get("id")?,
            chwila: row.get("chwila")?,
            kto: row.get("kto")?,
            tresc: row.get("tresc")?,
            pobyt_id: row.get("pobyt_id")?,
            odpowiedz_wysylka_id: row.get("odpowiedz_wysylka_id")?,
        })
    }
}

/// Skargi sąsiadów i wspólnoty w kolejności czasu.
pub fn skargi_sasiadow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<SkargiSasiadowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_pobyty_skargi
ORDER BY chwila, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(SkargiSasiadowRow::from_row).collect()
}

/// The parameters of `wpisz_skarge`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszSkargeParams {
    pub chwila: String,
    pub kto: String,
    pub tresc: String,
    pub pobyt_id: Option<String>,
}

/// Skarga: kto, czego dotyczy i którego pobytu, jeśli wiadomo.
pub fn wpisz_skarge(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszSkargeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_pobyty_skargi (chwila, kto, tresc, pobyt_id)
VALUES (:chwila, :kto, :tresc, :pobyt_id)"#,
                parameters: Vec::from([
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    ("kto", xmeta_room::ToSqlValue::to_sql_value(&params.kto)),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                    ("pobyt_id", xmeta_room::ToSqlValue::to_sql_value(&params.pobyt_id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wskaz_odpowiedz_na_skarge`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WskazOdpowiedzNaSkargeParams {
    pub odpowiedz_wysylka_id: String,
    pub id: i64,
}

/// Projekt odpowiedzi na skargę, zanim wyjdzie; czeka on na zgodę człowieka.
pub fn wskaz_odpowiedz_na_skarge(
    room: &impl xmeta_room::MetaRoom,
    params: &WskazOdpowiedzNaSkargeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_pobyty_skargi
SET odpowiedz_wysylka_id = :odpowiedz_wysylka_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "odpowiedz_wysylka_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.odpowiedz_wysylka_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `pobyty`.
#[derive(Debug, Clone, PartialEq)]
pub struct PobytyRow {
    pub id: String,
    pub kanal: String,
    pub numer_rezerwacji: String,
    pub przyjeto: String,
    pub przyjazd: String,
    pub wyjazd: String,
    pub gosci: i64,
    pub gosc_uczestnik_id: String,
    pub kwota_goscia: i64,
    pub oplata_sprzatanie: i64,
    pub prowizja_kanalu: Option<i64>,
    pub wyplata_kanalu: Option<i64>,
    pub wyplacono: Option<String>,
    pub rozliczono: Option<String>,
    pub oplacona: bool,
    pub instrukcja_wyslano: Option<String>,
    pub stan: WpisyStan,
    pub po_koncu_decyzja_id: Option<String>,
    pub rzeczy_pozostawione: Option<String>,
}

impl PobytyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PobytyRow {
            id: row.get("id")?,
            kanal: row.get("kanal")?,
            numer_rezerwacji: row.get("numer_rezerwacji")?,
            przyjeto: row.get("przyjeto")?,
            przyjazd: row.get("przyjazd")?,
            wyjazd: row.get("wyjazd")?,
            gosci: row.get("gosci")?,
            gosc_uczestnik_id: row.get("gosc_uczestnik_id")?,
            kwota_goscia: row.get("kwota_goscia")?,
            oplata_sprzatanie: row.get("oplata_sprzatanie")?,
            prowizja_kanalu: row.get("prowizja_kanalu")?,
            wyplata_kanalu: row.get("wyplata_kanalu")?,
            wyplacono: row.get("wyplacono")?,
            rozliczono: row.get("rozliczono")?,
            oplacona: row.get("oplacona")?,
            instrukcja_wyslano: row.get("instrukcja_wyslano")?,
            stan: row.get("stan")?,
            po_koncu_decyzja_id: row.get("po_koncu_decyzja_id")?,
            rzeczy_pozostawione: row.get("rzeczy_pozostawione")?,
        })
    }
}

/// Pobyty w kolejności przyjazdu.
pub fn pobyty(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PobytyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_pobyty_wpisy
ORDER BY przyjazd, przyjeto"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(PobytyRow::from_row).collect()
}

/// A row of `pobyty_w_toku`.
#[derive(Debug, Clone, PartialEq)]
pub struct PobytyWTokuRow {
    pub id: String,
    pub kanal: String,
    pub numer_rezerwacji: String,
    pub przyjeto: String,
    pub przyjazd: String,
    pub wyjazd: String,
    pub gosci: i64,
    pub gosc_uczestnik_id: String,
    pub kwota_goscia: i64,
    pub oplata_sprzatanie: i64,
    pub prowizja_kanalu: Option<i64>,
    pub wyplata_kanalu: Option<i64>,
    pub wyplacono: Option<String>,
    pub rozliczono: Option<String>,
    pub oplacona: bool,
    pub instrukcja_wyslano: Option<String>,
    pub stan: WpisyStan,
    pub po_koncu_decyzja_id: Option<String>,
    pub rzeczy_pozostawione: Option<String>,
}

impl PobytyWTokuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PobytyWTokuRow {
            id: row.get("id")?,
            kanal: row.get("kanal")?,
            numer_rezerwacji: row.get("numer_rezerwacji")?,
            przyjeto: row.get("przyjeto")?,
            przyjazd: row.get("przyjazd")?,
            wyjazd: row.get("wyjazd")?,
            gosci: row.get("gosci")?,
            gosc_uczestnik_id: row.get("gosc_uczestnik_id")?,
            kwota_goscia: row.get("kwota_goscia")?,
            oplata_sprzatanie: row.get("oplata_sprzatanie")?,
            prowizja_kanalu: row.get("prowizja_kanalu")?,
            wyplata_kanalu: row.get("wyplata_kanalu")?,
            wyplacono: row.get("wyplacono")?,
            rozliczono: row.get("rozliczono")?,
            oplacona: row.get("oplacona")?,
            instrukcja_wyslano: row.get("instrukcja_wyslano")?,
            stan: row.get("stan")?,
            po_koncu_decyzja_id: row.get("po_koncu_decyzja_id")?,
            rzeczy_pozostawione: row.get("rzeczy_pozostawione")?,
        })
    }
}

/// Pobyty potwierdzone, w kolizji i trwające, w kolejności przyjazdu.
pub fn pobyty_w_toku(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PobytyWTokuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_pobyty_wpisy
WHERE stan IN ('potwierdzona', 'kolizja', 'trwa')
ORDER BY przyjazd, przyjeto"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(PobytyWTokuRow::from_row).collect()
}

/// The parameters of `przyjmij_rezerwacje`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrzyjmijRezerwacjeParams {
    pub id: String,
    pub kanal: String,
    pub numer_rezerwacji: String,
    pub przyjeto: String,
    pub przyjazd: String,
    pub wyjazd: String,
    pub gosci: i64,
    pub gosc_uczestnik_id: String,
    pub kwota_goscia: i64,
    pub oplata_sprzatanie: i64,
    pub prowizja_kanalu: Option<i64>,
    pub oplacona: bool,
    pub po_koncu_decyzja_id: Option<String>,
}

/// Rezerwacja sprawdzona na koncie kanału: kanał, numer, termin, goście i pieniądze. Noce po końcu
/// umowy wskazują zgodę człowieka. Identyfikator (UUID) nadaje aplikacja, bo wskazują go noce,
/// zlecenia, dostępy i pozycje zestawień.
pub fn przyjmij_rezerwacje(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzyjmijRezerwacjeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_pobyty_wpisy (id, kanal, numer_rezerwacji, przyjeto, przyjazd,
  wyjazd, gosci, gosc_uczestnik_id, kwota_goscia, oplata_sprzatanie, prowizja_kanalu, oplacona,
  po_koncu_decyzja_id, stan)
VALUES (:id, :kanal, :numer_rezerwacji, :przyjeto, :przyjazd, :wyjazd, :gosci, :gosc_uczestnik_id,
  :kwota_goscia, :oplata_sprzatanie, :prowizja_kanalu, :oplacona, :po_koncu_decyzja_id,
  'potwierdzona')"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("kanal", xmeta_room::ToSqlValue::to_sql_value(&params.kanal)),
                    (
                        "numer_rezerwacji",
                        xmeta_room::ToSqlValue::to_sql_value(&params.numer_rezerwacji),
                    ),
                    ("przyjeto", xmeta_room::ToSqlValue::to_sql_value(&params.przyjeto)),
                    ("przyjazd", xmeta_room::ToSqlValue::to_sql_value(&params.przyjazd)),
                    ("wyjazd", xmeta_room::ToSqlValue::to_sql_value(&params.wyjazd)),
                    ("gosci", xmeta_room::ToSqlValue::to_sql_value(&params.gosci)),
                    (
                        "gosc_uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.gosc_uczestnik_id),
                    ),
                    (
                        "kwota_goscia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kwota_goscia),
                    ),
                    (
                        "oplata_sprzatanie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.oplata_sprzatanie),
                    ),
                    (
                        "prowizja_kanalu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.prowizja_kanalu),
                    ),
                    ("oplacona", xmeta_room::ToSqlValue::to_sql_value(&params.oplacona)),
                    (
                        "po_koncu_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.po_koncu_decyzja_id),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_rezerwacje`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZmienRezerwacjeParams {
    pub przyjazd: String,
    pub wyjazd: String,
    pub gosci: i64,
    pub kwota_goscia: i64,
    pub oplata_sprzatanie: i64,
    pub po_koncu_decyzja_id: Option<String>,
    pub id: String,
}

/// Nowy termin, liczba gości albo cena po zmianie wpisanej w zmianach pobytu. Noce po końcu umowy
/// wskazują zgodę człowieka.
pub fn zmien_rezerwacje(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienRezerwacjeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_pobyty_wpisy
SET przyjazd = :przyjazd,
  wyjazd = :wyjazd,
  gosci = :gosci,
  kwota_goscia = :kwota_goscia,
  oplata_sprzatanie = :oplata_sprzatanie,
  po_koncu_decyzja_id = :po_koncu_decyzja_id
WHERE id = :id"#,
                parameters: Vec::from([
                    ("przyjazd", xmeta_room::ToSqlValue::to_sql_value(&params.przyjazd)),
                    ("wyjazd", xmeta_room::ToSqlValue::to_sql_value(&params.wyjazd)),
                    ("gosci", xmeta_room::ToSqlValue::to_sql_value(&params.gosci)),
                    (
                        "kwota_goscia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kwota_goscia),
                    ),
                    (
                        "oplata_sprzatanie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.oplata_sprzatanie),
                    ),
                    (
                        "po_koncu_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.po_koncu_decyzja_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_oplacenie_pobytu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujOplaceniePobytuParams {
    pub id: String,
}

/// Rezerwacja opłacona według konta kanału albo rachunku biura.
pub fn odnotuj_oplacenie_pobytu(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujOplaceniePobytuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_pobyty_wpisy
SET oplacona = 1
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_instrukcje_wejscia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujInstrukcjeWejsciaParams {
    pub instrukcja_wyslano: String,
    pub id: String,
}

/// Konsjerż wysłał instrukcję wejścia w kanale rezerwacji; tylko do opłaconej rezerwacji.
pub fn odnotuj_instrukcje_wejscia(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujInstrukcjeWejsciaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_pobyty_wpisy
SET instrukcja_wyslano = :instrukcja_wyslano
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "instrukcja_wyslano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.instrukcja_wyslano),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_stan_pobytu`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZmienStanPobytuParams {
    pub stan: Stan,
    pub id: String,
}

/// Kolizja, powrót z kolizji, początek pobytu, odwołanie.
pub fn zmien_stan_pobytu(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienStanPobytuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_pobyty_wpisy
SET stan = :stan
WHERE id = :id"#,
                parameters: Vec::from([
                    ("stan", xmeta_room::ToSqlValue::to_sql_value(&params.stan)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `zakoncz_pobyt`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZakonczPobytParams {
    pub rzeczy_pozostawione: Option<String>,
    pub id: String,
}

/// Gość wyjechał; rzeczy pozostawione, jeśli są.
pub fn zakoncz_pobyt(
    room: &impl xmeta_room::MetaRoom,
    params: &ZakonczPobytParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_pobyty_wpisy
SET stan = 'zakonczona',
  rzeczy_pozostawione = :rzeczy_pozostawione
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "rzeczy_pozostawione",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rzeczy_pozostawione),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_rozliczenie_pobytu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujRozliczeniePobytuParams {
    pub prowizja_kanalu: i64,
    pub wyplata_kanalu: i64,
    pub wyplacono: String,
    pub rozliczono: String,
    pub id: String,
}

/// Rozliczenie z konta kanału: prowizja, wypłata i jej dzień; wpisuje się raz.
pub fn odnotuj_rozliczenie_pobytu(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujRozliczeniePobytuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_pobyty_wpisy
SET prowizja_kanalu = :prowizja_kanalu,
  wyplata_kanalu = :wyplata_kanalu,
  wyplacono = :wyplacono,
  rozliczono = :rozliczono
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "prowizja_kanalu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.prowizja_kanalu),
                    ),
                    (
                        "wyplata_kanalu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wyplata_kanalu),
                    ),
                    (
                        "wyplacono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wyplacono),
                    ),
                    (
                        "rozliczono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rozliczono),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `wyjatki_pobytow`.
#[derive(Debug, Clone, PartialEq)]
pub struct WyjatkiPobytowRow {
    pub id: String,
    pub pobyt_id: String,
    pub opis: String,
    pub decyzja_id: String,
}

impl WyjatkiPobytowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WyjatkiPobytowRow {
            id: row.get("id")?,
            pobyt_id: row.get("pobyt_id")?,
            opis: row.get("opis")?,
            decyzja_id: row.get("decyzja_id")?,
        })
    }
}

/// Wyjątki od regulaminu w kolejności wpisania.
pub fn wyjatki_pobytow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WyjatkiPobytowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_pobyty_wyjatki
ORDER BY rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(WyjatkiPobytowRow::from_row).collect()
}

/// The parameters of `wpisz_wyjatek_pobytu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszWyjatekPobytuParams {
    pub id: String,
    pub pobyt_id: String,
    pub opis: String,
    pub decyzja_id: String,
}

/// Wyjątek od regulaminu za zgodą człowieka. Identyfikator (UUID) nadaje aplikacja, bo wskazuje go
/// decyzja.
pub fn wpisz_wyjatek_pobytu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszWyjatekPobytuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_pobyty_wyjatki (id, pobyt_id, opis, decyzja_id)
VALUES (:id, :pobyt_id, :opis, :decyzja_id)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("pobyt_id", xmeta_room::ToSqlValue::to_sql_value(&params.pobyt_id)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    (
                        "decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_id),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `zgloszenia_gosci`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZgloszeniaGosciRow {
    pub id: String,
    pub pobyt_id: String,
    pub chwila: String,
    pub opis: String,
    pub przekazano: Option<String>,
    pub zagrozenie: bool,
    pub zawiadomiono: Option<String>,
}

impl ZgloszeniaGosciRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZgloszeniaGosciRow {
            id: row.get("id")?,
            pobyt_id: row.get("pobyt_id")?,
            chwila: row.get("chwila")?,
            opis: row.get("opis")?,
            przekazano: row.get("przekazano")?,
            zagrozenie: row.get("zagrozenie")?,
            zawiadomiono: row.get("zawiadomiono")?,
        })
    }
}

/// Zgłoszenia gości w kolejności czasu.
pub fn zgloszenia_gosci(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZgloszeniaGosciRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_pobyty_zgloszenia
ORDER BY chwila, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZgloszeniaGosciRow::from_row).collect()
}

/// The parameters of `wpisz_zgloszenie_goscia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszZgloszenieGosciaParams {
    pub id: String,
    pub pobyt_id: String,
    pub chwila: String,
    pub opis: String,
    pub przekazano: Option<String>,
    pub zagrozenie: bool,
    pub zawiadomiono: Option<String>,
}

/// Problem zgłoszony przez gościa z godziną i opisem; przy zagrożeniu także kogo zawiadomiono.
/// Identyfikator (UUID) nadaje aplikacja, bo wskazuje go zlecenie naprawy.
pub fn wpisz_zgloszenie_goscia(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszZgloszenieGosciaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_pobyty_zgloszenia (id, pobyt_id, chwila, opis, przekazano,
  zagrozenie, zawiadomiono)
VALUES (:id, :pobyt_id, :chwila, :opis, :przekazano, :zagrozenie, :zawiadomiono)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("pobyt_id", xmeta_room::ToSqlValue::to_sql_value(&params.pobyt_id)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    (
                        "przekazano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przekazano),
                    ),
                    (
                        "zagrozenie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zagrozenie),
                    ),
                    (
                        "zawiadomiono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zawiadomiono),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_przekazanie_zgloszenia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPrzekazanieZgloszeniaParams {
    pub przekazano: String,
    pub id: String,
}

/// Komu Konsjerż przekazał zgłoszenie; wpisuje się raz.
pub fn odnotuj_przekazanie_zgloszenia(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPrzekazanieZgloszeniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_pobyty_zgloszenia
SET przekazano = :przekazano
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "przekazano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przekazano),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `zmiany_pobytow`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZmianyPobytowRow {
    pub id: i64,
    pub pobyt_id: String,
    pub chwila: String,
    pub co: ZmianyCo,
    pub przyczyna: String,
    pub zdecydowal: String,
}

impl ZmianyPobytowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZmianyPobytowRow {
            id: row.get("id")?,
            pobyt_id: row.get("pobyt_id")?,
            chwila: row.get("chwila")?,
            co: row.get("co")?,
            przyczyna: row.get("przyczyna")?,
            zdecydowal: row.get("zdecydowal")?,
        })
    }
}

/// Zmiany pobytów w kolejności czasu.
pub fn zmiany_pobytow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZmianyPobytowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_pobyty_zmiany
ORDER BY chwila, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZmianyPobytowRow::from_row).collect()
}

/// The parameters of `wpisz_zmiane_pobytu`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszZmianePobytuParams {
    pub pobyt_id: String,
    pub chwila: String,
    pub co: ZmianyCo,
    pub przyczyna: String,
    pub zdecydowal: String,
}

/// Zmiana terminu, liczby gości albo ceny, albo odwołanie, z przyczyną i decyzją Zarządcy.
pub fn wpisz_zmiane_pobytu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszZmianePobytuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_pobyty_zmiany (pobyt_id, chwila, co, przyczyna, zdecydowal)
VALUES (:pobyt_id, :chwila, :co, :przyczyna, :zdecydowal)"#,
                parameters: Vec::from([
                    ("pobyt_id", xmeta_room::ToSqlValue::to_sql_value(&params.pobyt_id)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    ("co", xmeta_room::ToSqlValue::to_sql_value(&params.co)),
                    (
                        "przyczyna",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przyczyna),
                    ),
                    (
                        "zdecydowal",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zdecydowal),
                    ),
                ]),
            },
        ],
    )
}
