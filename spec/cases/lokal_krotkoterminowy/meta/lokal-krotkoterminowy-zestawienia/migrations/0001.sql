-- Miesięczne zestawienia dla właściciela lokalu (R3): pozycje ze źródłem w teczce, kwota do
-- wypłaty i rezerwa. Zestawienie liczy program, a Zarządca obiektu dobiera dane i je wysyła; nie
-- jest ono fakturą ani dokumentem księgowym.
--
-- Pozycje mają znak: przychód i kwota odzyskana dodają, prowizje, koszty, opłaty lokalu i zwroty
-- odejmują, a zmiana rezerwy i korekta mogą jedno i drugie. Kwota do wypłaty to suma pozycji.
-- Zestawienie obejmuje pobyty, które kanał rozliczył do końca jego miesiąca; przychód pobytu
-- i koszt zlecenia trafiają do jednego zestawienia, a przed zakończeniem zarządzania żaden nie
-- może z nich wypaść.
--
-- Zestawienie idzie: projekt (pozycje się dopisuje), gotowe (plik i kwoty zamrożone, wysyłka
-- czeka na zgodę człowieka), wysłane (już się nie zmienia). Korektę wysłanego zestawienia wprowadza
-- następne zestawienie za zgodą człowieka. Wypłata z rdzenia wskazuje wysłane zestawienie i ma
-- jego kwotę; rachunek sprawdza Kontroler po jej przygotowaniu (rdzeń, R4).
CREATE TABLE lokal_krotkoterminowy_zestawienia_wpisy (
  id TEXT PRIMARY KEY,
  -- Miesiąc zestawienia, np. 2027-01.
  miesiac TEXT NOT NULL CHECK (miesiac GLOB '[0-9][0-9][0-9][0-9]-[01][0-9]'),
  stan TEXT NOT NULL CHECK (stan IN ('projekt', 'gotowe', 'wyslane')),
  utworzono TEXT NOT NULL,
  -- Plik zestawienia z programu, ze skrótem.
  sciezka TEXT,
  skrot TEXT,
  -- Kwota do wypłaty i rezerwa po zestawieniu, w groszach.
  kwota_do_wyplaty INTEGER,
  rezerwa_po INTEGER CHECK (rezerwa_po >= 0),
  wysylka_id TEXT REFERENCES sprawa_korespondencja_wysylki (id),
  -- Wysłane zestawienie, które to zestawienie koryguje, i zgoda człowieka na korektę.
  korekta_do_id TEXT REFERENCES lokal_krotkoterminowy_zestawienia_wpisy (id),
  korekta_decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),

  CONSTRAINT plik_ze_skrotem CHECK ((sciezka IS NULL) = (skrot IS NULL)),
  CONSTRAINT gotowe_ma_plik_i_kwoty CHECK (
    stan = 'projekt' OR (sciezka IS NOT NULL AND kwota_do_wyplaty IS NOT NULL AND rezerwa_po IS NOT NULL)
  ),
  CONSTRAINT wyslane_ma_wysylke CHECK (stan <> 'wyslane' OR wysylka_id IS NOT NULL),
  CONSTRAINT korekta_z_decyzja CHECK ((korekta_do_id IS NULL) = (korekta_decyzja_id IS NULL)),

  UNIQUE (miesiac)
) STRICT;

-- Pozycje zestawienia, każda ze źródłem w teczce: pobyt (rozliczenie kanału, prowizja biura),
-- zlecenie (koszt), pieniądze pobytu (zwrot, odzysk), warunki umowy (prowizja biura, rezerwa) albo
-- dokument (np. naliczenie wspólnoty).
CREATE TABLE lokal_krotkoterminowy_zestawienia_pozycje (
  -- Klucz nadany przez bazę jest dodatni; zob. lokal_krotkoterminowy_zestawienia_pozycje_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  zestawienie_id TEXT NOT NULL REFERENCES lokal_krotkoterminowy_zestawienia_wpisy (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('przychod', 'prowizja_kanalu', 'prowizja_biura', 'koszt', 'oplata_lokalu', 'zwrot', 'odzysk', 'rezerwa', 'korekta')),
  -- Kwota w groszach ze znakiem: dodatnia dodaje się do wypłaty, ujemna odejmuje.
  kwota INTEGER NOT NULL,
  opis TEXT,
  pobyt_id TEXT REFERENCES lokal_krotkoterminowy_pobyty_wpisy (id),
  zlecenie_id TEXT REFERENCES lokal_krotkoterminowy_zlecenia_wpisy (id),
  pieniadze_id TEXT REFERENCES lokal_krotkoterminowy_pobyty_pieniadze (id),
  warunki_id TEXT REFERENCES lokal_krotkoterminowy_umowa_warunki (id),
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),

  -- Program nie przyjmuje pozycji bez źródła w teczce.
  CONSTRAINT pozycja_ma_zrodlo CHECK (
    pobyt_id IS NOT NULL
    OR zlecenie_id IS NOT NULL
    OR pieniadze_id IS NOT NULL
    OR warunki_id IS NOT NULL
    OR dokument_id IS NOT NULL
  ),
  CONSTRAINT znak_kwoty_wedlug_rodzaju CHECK (
    CASE
      WHEN rodzaj IN ('przychod', 'odzysk') THEN kwota > 0
      WHEN rodzaj IN ('prowizja_kanalu', 'prowizja_biura', 'koszt', 'oplata_lokalu', 'zwrot') THEN kwota < 0
      ELSE kwota <> 0
    END
  )
) STRICT;

-- Przychód i prowizja kanału pobytu, koszt zlecenia i pieniądze pobytu trafiają do jednego
-- zestawienia (zob. też lokal_krotkoterminowy_zestawienia_pozycja_raz).
CREATE UNIQUE INDEX lokal_krotkoterminowy_zestawienia_przychod_pobytu_raz
ON lokal_krotkoterminowy_zestawienia_pozycje (pobyt_id)
WHERE rodzaj = 'przychod';

CREATE UNIQUE INDEX lokal_krotkoterminowy_zestawienia_prowizja_kanalu_raz
ON lokal_krotkoterminowy_zestawienia_pozycje (pobyt_id)
WHERE rodzaj = 'prowizja_kanalu';

CREATE UNIQUE INDEX lokal_krotkoterminowy_zestawienia_koszt_zlecenia_raz
ON lokal_krotkoterminowy_zestawienia_pozycje (zlecenie_id)
WHERE rodzaj = 'koszt';

CREATE UNIQUE INDEX lokal_krotkoterminowy_zestawienia_pieniadze_raz
ON lokal_krotkoterminowy_zestawienia_pozycje (pieniadze_id)
WHERE rodzaj IN ('zwrot', 'odzysk');

-- Pliki, kwoty i pozycje sprawdza się przy przejściach stanu, więc zestawienie powstaje jako
-- projekt bez wysyłki.
CREATE TRIGGER lokal_krotkoterminowy_zestawienia_powstaje_jako_projekt
BEFORE INSERT ON lokal_krotkoterminowy_zestawienia_wpisy
WHEN NEW.stan <> 'projekt'
  OR NEW.wysylka_id IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'zestawienie powstaje jako projekt, bez wysyłki');
END;

-- Wysłanego zestawienia się nie zmienia; korektę wprowadza następne zestawienie za zgodą
-- człowieka, które wskazuje korygowane zestawienie.
CREATE TRIGGER lokal_krotkoterminowy_zestawienia_korekta_za_zgoda
BEFORE INSERT ON lokal_krotkoterminowy_zestawienia_wpisy
WHEN NEW.korekta_do_id IS NOT NULL
  AND (
    NOT EXISTS (
      SELECT 1
      FROM lokal_krotkoterminowy_zestawienia_wpisy
      WHERE id = NEW.korekta_do_id
        AND stan = 'wyslane'
        AND miesiac < NEW.miesiac
    )
    OR NOT EXISTS (
      SELECT 1
      FROM sprawa_decyzje_zatwierdzone
      WHERE id = NEW.korekta_decyzja_id
        AND rodzaj = 'inna'
        AND dotyczy_tabela = 'lokal_krotkoterminowy_zestawienia_wpisy'
        AND dotyczy_id = NEW.id
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'korekta wskazuje wcześniejsze wysłane zestawienie i zatwierdzoną decyzję inna, która ją wskazuje');
END;

-- Projekt staje się gotowy, gotowe wraca do projektu tylko bez wysyłki albo z anulowaną wysyłką,
-- a wychodzi jako wysłane.
CREATE TRIGGER lokal_krotkoterminowy_zestawienia_przejscia_stanu
BEFORE UPDATE OF stan ON lokal_krotkoterminowy_zestawienia_wpisy
WHEN NEW.stan IS NOT OLD.stan
  AND NOT (
    (OLD.stan = 'projekt' AND NEW.stan = 'gotowe')
    OR (OLD.stan = 'gotowe' AND NEW.stan = 'wyslane')
    OR (
      OLD.stan = 'gotowe'
      AND NEW.stan = 'projekt'
      AND NEW.wysylka_id IS NULL
      AND NOT EXISTS (
        SELECT 1
        FROM sprawa_korespondencja_wysylki
        WHERE id = OLD.wysylka_id
          AND stan <> 'anulowana'
      )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'zestawienie przechodzi: projekt, gotowe, wysłane; do projektu wraca tylko bez wysyłki albo z anulowaną');
END;

-- Kwota do wypłaty to przychody minus prowizje, koszty, opłaty i zmiana rezerwy: suma pozycji.
CREATE TRIGGER lokal_krotkoterminowy_zestawienia_kwota_to_suma_pozycji
BEFORE UPDATE OF stan ON lokal_krotkoterminowy_zestawienia_wpisy
WHEN NEW.stan = 'gotowe'
  AND OLD.stan = 'projekt'
  AND NEW.kwota_do_wyplaty IS NOT (
    SELECT coalesce(sum(kwota), 0)
    FROM lokal_krotkoterminowy_zestawienia_pozycje
    WHERE zestawienie_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'kwota do wypłaty to suma pozycji zestawienia');
END;

-- Gotowe zestawienie wskazuje wysyłkę, zanim wyjdzie, żeby reguła zgody mogła jej dopilnować.
CREATE TRIGGER lokal_krotkoterminowy_zestawienia_wysylka_przed_wyslaniem
BEFORE UPDATE OF wysylka_id ON lokal_krotkoterminowy_zestawienia_wpisy
WHEN NEW.wysylka_id IS NOT NULL
  AND NEW.wysylka_id IS NOT OLD.wysylka_id
  AND (
    OLD.wysylka_id IS NOT NULL
    OR OLD.stan <> 'gotowe'
    OR NOT EXISTS (
      SELECT 1
      FROM sprawa_korespondencja_wysylki
      WHERE id = NEW.wysylka_id
        AND stan IN ('projekt', 'czeka_na_zgode')
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'gotowe zestawienie wskazuje wysyłkę raz, zanim wyjdzie');
END;

-- Wysłanie zestawienia właścicielowi wymaga zgody człowieka: wysyłka przed wysłaniem czeka na
-- zgodę (rdzeń wymaga wtedy decyzji wyslanie).
CREATE TRIGGER lokal_krotkoterminowy_zestawienia_wysylka_za_zgoda
BEFORE UPDATE OF stan ON sprawa_korespondencja_wysylki
WHEN NEW.stan = 'wyslana'
  AND OLD.stan = 'projekt'
  AND EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_zestawienia_wpisy
    WHERE wysylka_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'zestawienie dla właściciela przed wysłaniem czeka na zgodę');
END;

CREATE TRIGGER lokal_krotkoterminowy_zestawienia_wyslane_po_wysylce
BEFORE UPDATE OF stan ON lokal_krotkoterminowy_zestawienia_wpisy
WHEN NEW.stan = 'wyslane'
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_korespondencja_wysylki
    WHERE id = NEW.wysylka_id
      AND stan IN ('wyslana', 'doreczona')
  )
BEGIN
  SELECT RAISE(ABORT, 'zestawienie jest wysłane, gdy jego wysyłka wyszła');
END;

-- Plik i kwoty zmieniają się tylko w projekcie, a miesiąc i korekta wcale. Wysłanego zestawienia
-- się nie zmienia.
CREATE TRIGGER lokal_krotkoterminowy_zestawienia_tresc_tylko_w_projekcie
BEFORE UPDATE ON lokal_krotkoterminowy_zestawienia_wpisy
WHEN OLD.stan = 'wyslane'
  OR NEW.id IS NOT OLD.id
  OR NEW.miesiac IS NOT OLD.miesiac
  OR NEW.utworzono IS NOT OLD.utworzono
  OR NEW.korekta_do_id IS NOT OLD.korekta_do_id
  OR NEW.korekta_decyzja_id IS NOT OLD.korekta_decyzja_id
  OR (
    OLD.stan <> 'projekt'
    AND (
      NEW.sciezka IS NOT OLD.sciezka
      OR NEW.skrot IS NOT OLD.skrot
      OR NEW.kwota_do_wyplaty IS NOT OLD.kwota_do_wyplaty
      OR NEW.rezerwa_po IS NOT OLD.rezerwa_po
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'plik i kwoty zestawienia zmieniają się tylko w projekcie, a wysłanego zestawienia się nie zmienia');
END;

CREATE TRIGGER lokal_krotkoterminowy_zestawienia_pozycje_tylko_w_projekcie
BEFORE INSERT ON lokal_krotkoterminowy_zestawienia_pozycje
WHEN (
  SELECT stan <> 'projekt'
  FROM lokal_krotkoterminowy_zestawienia_wpisy
  WHERE id = NEW.zestawienie_id
)
BEGIN
  SELECT RAISE(ABORT, 'pozycje dopisuje się tylko do projektu zestawienia');
END;

CREATE TRIGGER lokal_krotkoterminowy_zestawienia_pozycje_bez_zmian
BEFORE UPDATE ON lokal_krotkoterminowy_zestawienia_pozycje
BEGIN
  SELECT RAISE(ABORT, 'pozycji zestawienia się nie zmienia; pomyłkę w projekcie poprawia pozycja korekty, a po wysłaniu następne zestawienie');
END;

-- Przychód i prowizja kanału pochodzą z rozliczenia pobytu na koncie kanału, z tymi samymi
-- kwotami, a pobyt rozliczono najpóźniej w miesiącu zestawienia. Pobyt nierozliczony przechodzi
-- do następnego zestawienia.
CREATE TRIGGER lokal_krotkoterminowy_zestawienia_przychod_z_rozliczenia_pobytu
BEFORE INSERT ON lokal_krotkoterminowy_zestawienia_pozycje
WHEN NEW.rodzaj IN ('przychod', 'prowizja_kanalu')
  AND NOT EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_pobyty_wpisy AS pobyt
    JOIN lokal_krotkoterminowy_zestawienia_wpisy AS zestawienie ON zestawienie.id = NEW.zestawienie_id
    WHERE pobyt.id = NEW.pobyt_id
      AND pobyt.rozliczono IS NOT NULL
      AND substr(pobyt.rozliczono, 1, 7) <= zestawienie.miesiac
      AND NEW.kwota = CASE NEW.rodzaj WHEN 'przychod' THEN pobyt.kwota_goscia ELSE -pobyt.prowizja_kanalu END
  )
BEGIN
  SELECT RAISE(ABORT, 'przychód i prowizja kanału wskazują pobyt rozliczony najpóźniej w miesiącu zestawienia i mają kwoty z rozliczenia');
END;

-- Koszt wskazuje zlecenie z potwierdzonym wykonaniem i wpisanym kosztem, z tą samą kwotą.
CREATE TRIGGER lokal_krotkoterminowy_zestawienia_koszt_z_potwierdzonego_zlecenia
BEFORE INSERT ON lokal_krotkoterminowy_zestawienia_pozycje
WHEN NEW.rodzaj = 'koszt'
  AND NOT EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_zlecenia_wpisy
    WHERE id = NEW.zlecenie_id
      AND stan = 'potwierdzone'
      AND koszt IS NOT NULL
      AND NEW.kwota = -koszt
  )
BEGIN
  SELECT RAISE(ABORT, 'koszt wskazuje potwierdzone zlecenie z wpisanym kosztem i ma jego kwotę');
END;

-- Zwrot dla gościa i rekompensata odejmują, kwota odzyskana od gościa albo z procedury kanału
-- dodaje, każda z kwotą z pieniędzy pobytu.
CREATE TRIGGER lokal_krotkoterminowy_zestawienia_zwrot_i_odzysk_z_pieniedzy_pobytu
BEFORE INSERT ON lokal_krotkoterminowy_zestawienia_pozycje
WHEN NEW.rodzaj IN ('zwrot', 'odzysk')
  AND NOT EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_pobyty_pieniadze
    WHERE id = NEW.pieniadze_id
      AND (
        (NEW.rodzaj = 'zwrot' AND rodzaj IN ('zwrot', 'rekompensata') AND NEW.kwota = -kwota)
        OR (NEW.rodzaj = 'odzysk' AND rodzaj = 'odzysk' AND NEW.kwota = kwota)
      )
  )
BEGIN
  SELECT RAISE(ABORT, 'zwrot wskazuje zwrot albo rekompensatę, a odzysk kwotę odzyskaną z pieniędzy pobytu, z tą samą kwotą');
END;

CREATE TRIGGER lokal_krotkoterminowy_zestawienia_korekta_w_zestawieniu_korygujacym
BEFORE INSERT ON lokal_krotkoterminowy_zestawienia_pozycje
WHEN NEW.rodzaj = 'korekta'
  AND NOT EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_zestawienia_wpisy
    WHERE id = NEW.zestawienie_id
      AND korekta_do_id IS NOT NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'pozycja korekty leży w zestawieniu, które wskazuje korygowane zestawienie za zgodą człowieka');
END;

-- Ta sama reguła co indeksy *_raz, z nazwą w odmowie: przychód, prowizja kanału, koszt i pieniądze
-- pobytu nie trafiają do dwóch zestawień.
CREATE TRIGGER lokal_krotkoterminowy_zestawienia_pozycja_raz
BEFORE INSERT ON lokal_krotkoterminowy_zestawienia_pozycje
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_zestawienia_pozycje
  WHERE (rodzaj = 'przychod' AND NEW.rodzaj = 'przychod' AND pobyt_id = NEW.pobyt_id)
    OR (rodzaj = 'prowizja_kanalu' AND NEW.rodzaj = 'prowizja_kanalu' AND pobyt_id = NEW.pobyt_id)
    OR (rodzaj = 'koszt' AND NEW.rodzaj = 'koszt' AND zlecenie_id = NEW.zlecenie_id)
    OR (
      rodzaj IN ('zwrot', 'odzysk')
      AND NEW.rodzaj IN ('zwrot', 'odzysk')
      AND pieniadze_id = NEW.pieniadze_id
    )
)
BEGIN
  SELECT RAISE(ABORT, 'przychód, prowizja kanału, koszt zlecenia i pieniądze pobytu trafiają do jednego zestawienia');
END;

-- Pieniądze właściciela wychodzą z teczki lokalu na podstawie wysłanego zestawienia, z jego kwotą
-- do wypłaty, albo jako zwrot lub rekompensata dla gościa, z kwotą z pieniędzy pobytu.
CREATE TRIGGER lokal_krotkoterminowy_zestawienia_wyplata_z_zestawienia
BEFORE INSERT ON sprawa_wyplaty_wpisy
WHEN NOT (
    (
      NEW.podstawa_tabela = 'lokal_krotkoterminowy_zestawienia_wpisy'
      AND EXISTS (
        SELECT 1
        FROM lokal_krotkoterminowy_zestawienia_wpisy
        WHERE id = NEW.podstawa_id
          AND stan = 'wyslane'
          AND kwota_do_wyplaty = NEW.kwota
      )
    )
    OR (
      NEW.podstawa_tabela = 'lokal_krotkoterminowy_pobyty_pieniadze'
      AND EXISTS (
        SELECT 1
        FROM lokal_krotkoterminowy_pobyty_pieniadze
        WHERE id = NEW.podstawa_id
          AND rodzaj IN ('zwrot', 'rekompensata')
          AND kwota = NEW.kwota
      )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'wypłata wskazuje wysłane zestawienie i ma jego kwotę do wypłaty albo wskazuje zwrot lub rekompensatę dla gościa i ma jej kwotę');
END;

-- Jedno zestawienie to jedna wypłata; nowa dopiero po anulowaniu poprzedniej.
CREATE TRIGGER lokal_krotkoterminowy_zestawienia_jedna_wyplata
BEFORE INSERT ON sprawa_wyplaty_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_wyplaty_wpisy
  WHERE podstawa_tabela = NEW.podstawa_tabela
    AND podstawa_id = NEW.podstawa_id
    AND stan IN ('przygotowana', 'wykonana')
)
BEGIN
  SELECT RAISE(ABORT, 'zestawienie albo zwrot ma jedną wypłatę; nowa przychodzi dopiero po anulowaniu poprzedniej');
END;

-- Zarządzanie kończy się, gdy każde zestawienie jest wysłane, a każdy rozliczony pobyt i każdy
-- koszt zlecenia jest w zestawieniu: żaden nie wypadł z obu.
CREATE TRIGGER lokal_krotkoterminowy_zestawienia_po_zarzadzaniu_wszystko_rozliczone
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap = 'po_zarzadzaniu'
  AND (
    EXISTS (
      SELECT 1
      FROM lokal_krotkoterminowy_zestawienia_wpisy
      WHERE stan <> 'wyslane'
    )
    OR EXISTS (
      SELECT 1
      FROM lokal_krotkoterminowy_pobyty_wpisy AS pobyt
      WHERE pobyt.rozliczono IS NOT NULL
        AND NOT EXISTS (
          SELECT 1
          FROM lokal_krotkoterminowy_zestawienia_pozycje
          WHERE rodzaj = 'przychod'
            AND pobyt_id = pobyt.id
        )
    )
    OR EXISTS (
      SELECT 1
      FROM lokal_krotkoterminowy_zlecenia_wpisy AS zlecenie
      WHERE zlecenie.koszt IS NOT NULL
        AND NOT EXISTS (
          SELECT 1
          FROM lokal_krotkoterminowy_zestawienia_pozycje
          WHERE rodzaj = 'koszt'
            AND zlecenie_id = zlecenie.id
        )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'etap po_zarzadzaniu wymaga wysłanych zestawień, w których jest każdy rozliczony pobyt i każdy koszt zlecenia');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER lokal_krotkoterminowy_zestawienia_wpisy_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_zestawienia_wpisy
WHEN EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_zestawienia_wpisy
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_zestawienia_wpisy
    WHERE miesiac = NEW.miesiac
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_zestawienia_wpisy_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_zestawienia_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER lokal_krotkoterminowy_zestawienia_pozycje_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_zestawienia_pozycje
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_zestawienia_pozycje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_zestawienia_pozycje_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_zestawienia_pozycje
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
