//! The kind `lokal-krotkoterminowy-zestawienia` of the room `cases/lokal_krotkoterminowy`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "lokal-krotkoterminowy-zestawienia",
    requires: &[
        &super::lokal_krotkoterminowy_pobyty::KIND,
        &super::lokal_krotkoterminowy_umowa::KIND,
        &super::lokal_krotkoterminowy_zlecenia::KIND,
        &super::sprawa_decyzje::KIND,
        &super::sprawa_dokumenty::KIND,
        &super::sprawa_korespondencja::KIND,
        &super::sprawa_metryka::KIND,
        &super::sprawa_wyplaty::KIND,
    ],
    migrations: &[
        r#"-- Miesięczne zestawienia dla właściciela lokalu (R3): pozycje ze źródłem w teczce, kwota do
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
"#,
    ],
};

/// The values of column `pozycje.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PozycjeRodzaj {
    Przychod,
    ProwizjaKanalu,
    ProwizjaBiura,
    Koszt,
    OplataLokalu,
    Zwrot,
    Odzysk,
    Rezerwa,
    Korekta,
}

impl PozycjeRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[PozycjeRodzaj] = &[
        PozycjeRodzaj::Przychod,
        PozycjeRodzaj::ProwizjaKanalu,
        PozycjeRodzaj::ProwizjaBiura,
        PozycjeRodzaj::Koszt,
        PozycjeRodzaj::OplataLokalu,
        PozycjeRodzaj::Zwrot,
        PozycjeRodzaj::Odzysk,
        PozycjeRodzaj::Rezerwa,
        PozycjeRodzaj::Korekta,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            PozycjeRodzaj::Przychod => "przychod",
            PozycjeRodzaj::ProwizjaKanalu => "prowizja_kanalu",
            PozycjeRodzaj::ProwizjaBiura => "prowizja_biura",
            PozycjeRodzaj::Koszt => "koszt",
            PozycjeRodzaj::OplataLokalu => "oplata_lokalu",
            PozycjeRodzaj::Zwrot => "zwrot",
            PozycjeRodzaj::Odzysk => "odzysk",
            PozycjeRodzaj::Rezerwa => "rezerwa",
            PozycjeRodzaj::Korekta => "korekta",
        }
    }
}

impl xmeta_room::FromSqlValue for PozycjeRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "przychod" => Ok(PozycjeRodzaj::Przychod),
                    "prowizja_kanalu" => Ok(PozycjeRodzaj::ProwizjaKanalu),
                    "prowizja_biura" => Ok(PozycjeRodzaj::ProwizjaBiura),
                    "koszt" => Ok(PozycjeRodzaj::Koszt),
                    "oplata_lokalu" => Ok(PozycjeRodzaj::OplataLokalu),
                    "zwrot" => Ok(PozycjeRodzaj::Zwrot),
                    "odzysk" => Ok(PozycjeRodzaj::Odzysk),
                    "rezerwa" => Ok(PozycjeRodzaj::Rezerwa),
                    "korekta" => Ok(PozycjeRodzaj::Korekta),
                    other => Err(format!("{other} is no value of PozycjeRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for PozycjeRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyStan {
    Projekt,
    Gotowe,
    Wyslane,
}

impl WpisyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyStan] = &[
        WpisyStan::Projekt,
        WpisyStan::Gotowe,
        WpisyStan::Wyslane,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyStan::Projekt => "projekt",
            WpisyStan::Gotowe => "gotowe",
            WpisyStan::Wyslane => "wyslane",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "projekt" => Ok(WpisyStan::Projekt),
                    "gotowe" => Ok(WpisyStan::Gotowe),
                    "wyslane" => Ok(WpisyStan::Wyslane),
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

/// A row of `pozycje_zestawien`.
#[derive(Debug, Clone, PartialEq)]
pub struct PozycjeZestawienRow {
    pub id: i64,
    pub zestawienie_id: String,
    pub rodzaj: PozycjeRodzaj,
    pub kwota: i64,
    pub opis: Option<String>,
    pub pobyt_id: Option<String>,
    pub zlecenie_id: Option<String>,
    pub pieniadze_id: Option<String>,
    pub warunki_id: Option<String>,
    pub dokument_id: Option<String>,
}

impl PozycjeZestawienRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PozycjeZestawienRow {
            id: row.get("id")?,
            zestawienie_id: row.get("zestawienie_id")?,
            rodzaj: row.get("rodzaj")?,
            kwota: row.get("kwota")?,
            opis: row.get("opis")?,
            pobyt_id: row.get("pobyt_id")?,
            zlecenie_id: row.get("zlecenie_id")?,
            pieniadze_id: row.get("pieniadze_id")?,
            warunki_id: row.get("warunki_id")?,
            dokument_id: row.get("dokument_id")?,
        })
    }
}

/// Pozycje wszystkich zestawień w kolejności wpisania.
pub fn pozycje_zestawien(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PozycjeZestawienRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_zestawienia_pozycje
ORDER BY zestawienie_id, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(PozycjeZestawienRow::from_row).collect()
}

/// The parameters of `dopisz_pozycje_zestawienia`.
#[derive(Debug, Clone, PartialEq)]
pub struct DopiszPozycjeZestawieniaParams {
    pub zestawienie_id: String,
    pub rodzaj: PozycjeRodzaj,
    pub kwota: i64,
    pub opis: Option<String>,
    pub pobyt_id: Option<String>,
    pub zlecenie_id: Option<String>,
    pub pieniadze_id: Option<String>,
    pub warunki_id: Option<String>,
    pub dokument_id: Option<String>,
}

/// Pozycja ze znakiem i źródłem w teczce, w projekcie zestawienia.
pub fn dopisz_pozycje_zestawienia(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszPozycjeZestawieniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_zestawienia_pozycje (zestawienie_id, rodzaj, kwota, opis, pobyt_id,
  zlecenie_id, pieniadze_id, warunki_id, dokument_id)
VALUES (:zestawienie_id, :rodzaj, :kwota, :opis, :pobyt_id, :zlecenie_id, :pieniadze_id, :warunki_id,
  :dokument_id)"#,
                parameters: Vec::from([
                    (
                        "zestawienie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zestawienie_id),
                    ),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("kwota", xmeta_room::ToSqlValue::to_sql_value(&params.kwota)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    ("pobyt_id", xmeta_room::ToSqlValue::to_sql_value(&params.pobyt_id)),
                    (
                        "zlecenie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zlecenie_id),
                    ),
                    (
                        "pieniadze_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.pieniadze_id),
                    ),
                    (
                        "warunki_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.warunki_id),
                    ),
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `zestawienia`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZestawieniaRow {
    pub id: String,
    pub miesiac: String,
    pub stan: WpisyStan,
    pub utworzono: String,
    pub sciezka: Option<String>,
    pub skrot: Option<String>,
    pub kwota_do_wyplaty: Option<i64>,
    pub rezerwa_po: Option<i64>,
    pub wysylka_id: Option<String>,
    pub korekta_do_id: Option<String>,
    pub korekta_decyzja_id: Option<String>,
}

impl ZestawieniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZestawieniaRow {
            id: row.get("id")?,
            miesiac: row.get("miesiac")?,
            stan: row.get("stan")?,
            utworzono: row.get("utworzono")?,
            sciezka: row.get("sciezka")?,
            skrot: row.get("skrot")?,
            kwota_do_wyplaty: row.get("kwota_do_wyplaty")?,
            rezerwa_po: row.get("rezerwa_po")?,
            wysylka_id: row.get("wysylka_id")?,
            korekta_do_id: row.get("korekta_do_id")?,
            korekta_decyzja_id: row.get("korekta_decyzja_id")?,
        })
    }
}

/// Zestawienia dla właściciela w kolejności miesięcy.
pub fn zestawienia(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZestawieniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_zestawienia_wpisy
ORDER BY miesiac"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZestawieniaRow::from_row).collect()
}

/// The parameters of `przygotuj_zestawienie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrzygotujZestawienieParams {
    pub id: String,
    pub miesiac: String,
    pub utworzono: String,
    pub korekta_do_id: Option<String>,
    pub korekta_decyzja_id: Option<String>,
}

/// Projekt zestawienia za miesiąc; korekta wcześniejszego zestawienia wskazuje je i zgodę
/// człowieka. Identyfikator (UUID) nadaje aplikacja, bo wskazują go pozycje, decyzja i wypłata.
pub fn przygotuj_zestawienie(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzygotujZestawienieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_zestawienia_wpisy (id, miesiac, utworzono, korekta_do_id,
  korekta_decyzja_id, stan)
VALUES (:id, :miesiac, :utworzono, :korekta_do_id, :korekta_decyzja_id, 'projekt')"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("miesiac", xmeta_room::ToSqlValue::to_sql_value(&params.miesiac)),
                    (
                        "utworzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.utworzono),
                    ),
                    (
                        "korekta_do_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.korekta_do_id),
                    ),
                    (
                        "korekta_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.korekta_decyzja_id),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zamknij_zestawienie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZamknijZestawienieParams {
    pub sciezka: String,
    pub skrot: String,
    pub kwota_do_wyplaty: i64,
    pub rezerwa_po: i64,
    pub id: String,
}

/// Projekt gotowy do zgody: plik z programu, kwota do wypłaty (suma pozycji) i rezerwa po
/// zestawieniu.
pub fn zamknij_zestawienie(
    room: &impl xmeta_room::MetaRoom,
    params: &ZamknijZestawienieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_zestawienia_wpisy
SET stan = 'gotowe',
  sciezka = :sciezka,
  skrot = :skrot,
  kwota_do_wyplaty = :kwota_do_wyplaty,
  rezerwa_po = :rezerwa_po
WHERE id = :id"#,
                parameters: Vec::from([
                    ("sciezka", xmeta_room::ToSqlValue::to_sql_value(&params.sciezka)),
                    ("skrot", xmeta_room::ToSqlValue::to_sql_value(&params.skrot)),
                    (
                        "kwota_do_wyplaty",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kwota_do_wyplaty),
                    ),
                    (
                        "rezerwa_po",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rezerwa_po),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wskaz_wysylke_zestawienia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WskazWysylkeZestawieniaParams {
    pub wysylka_id: String,
    pub id: String,
}

/// Wysyłka gotowego zestawienia, zanim wyjdzie; czeka ona na zgodę człowieka.
pub fn wskaz_wysylke_zestawienia(
    room: &impl xmeta_room::MetaRoom,
    params: &WskazWysylkeZestawieniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_zestawienia_wpisy
SET wysylka_id = :wysylka_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "wysylka_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wysylka_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `cofnij_zestawienie_do_projektu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CofnijZestawienieDoProjektuParams {
    pub id: String,
}

/// Gotowe zestawienie wraca do poprawki, gdy nie ma wysyłki albo jego wysyłkę anulowano.
pub fn cofnij_zestawienie_do_projektu(
    room: &impl xmeta_room::MetaRoom,
    params: &CofnijZestawienieDoProjektuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_zestawienia_wpisy
SET stan = 'projekt',
  wysylka_id = NULL
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_wyslanie_zestawienia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujWyslanieZestawieniaParams {
    pub id: String,
}

/// Zestawienie wyszło do właściciela; od tej chwili się nie zmienia.
pub fn odnotuj_wyslanie_zestawienia(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWyslanieZestawieniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_zestawienia_wpisy
SET stan = 'wyslane'
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
