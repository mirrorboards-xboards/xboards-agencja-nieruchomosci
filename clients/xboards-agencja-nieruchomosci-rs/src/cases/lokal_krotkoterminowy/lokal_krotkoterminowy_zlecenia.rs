//! The kind `lokal-krotkoterminowy-zlecenia` of the room `cases/lokal_krotkoterminowy`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "lokal-krotkoterminowy-zlecenia",
    requires: &[
        &super::lokal_krotkoterminowy_pobyty::KIND,
        &super::sprawa_decyzje::KIND,
        &super::sprawa_uczestnicy::KIND,
        &super::sprawa_zgody::KIND,
    ],
    migrations: &[
        r#"-- Zlecenia dla wykonawców (sprzątanie, pranie, serwis, naprawy, przeglądy, zapasy, klucze,
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
"#,
    ],
};

/// The values of column `dostepy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DostepyRodzaj {
    Kod,
    Klucze,
}

impl DostepyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[DostepyRodzaj] = &[DostepyRodzaj::Kod, DostepyRodzaj::Klucze];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            DostepyRodzaj::Kod => "kod",
            DostepyRodzaj::Klucze => "klucze",
        }
    }
}

impl xmeta_room::FromSqlValue for DostepyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "kod" => Ok(DostepyRodzaj::Kod),
                    "klucze" => Ok(DostepyRodzaj::Klucze),
                    other => Err(format!("{other} is no value of DostepyRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for DostepyRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    Sprzatanie,
    Pranie,
    Serwis,
    Naprawa,
    Przeglad,
    DostawaZapasow,
    WydanieKluczy,
    OdbiorKluczy,
    PrzyjecieGosci,
    WymianaZamka,
    PracePozaLokalem,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::Sprzatanie,
        WpisyRodzaj::Pranie,
        WpisyRodzaj::Serwis,
        WpisyRodzaj::Naprawa,
        WpisyRodzaj::Przeglad,
        WpisyRodzaj::DostawaZapasow,
        WpisyRodzaj::WydanieKluczy,
        WpisyRodzaj::OdbiorKluczy,
        WpisyRodzaj::PrzyjecieGosci,
        WpisyRodzaj::WymianaZamka,
        WpisyRodzaj::PracePozaLokalem,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::Sprzatanie => "sprzatanie",
            WpisyRodzaj::Pranie => "pranie",
            WpisyRodzaj::Serwis => "serwis",
            WpisyRodzaj::Naprawa => "naprawa",
            WpisyRodzaj::Przeglad => "przeglad",
            WpisyRodzaj::DostawaZapasow => "dostawa_zapasow",
            WpisyRodzaj::WydanieKluczy => "wydanie_kluczy",
            WpisyRodzaj::OdbiorKluczy => "odbior_kluczy",
            WpisyRodzaj::PrzyjecieGosci => "przyjecie_gosci",
            WpisyRodzaj::WymianaZamka => "wymiana_zamka",
            WpisyRodzaj::PracePozaLokalem => "prace_poza_lokalem",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "sprzatanie" => Ok(WpisyRodzaj::Sprzatanie),
                    "pranie" => Ok(WpisyRodzaj::Pranie),
                    "serwis" => Ok(WpisyRodzaj::Serwis),
                    "naprawa" => Ok(WpisyRodzaj::Naprawa),
                    "przeglad" => Ok(WpisyRodzaj::Przeglad),
                    "dostawa_zapasow" => Ok(WpisyRodzaj::DostawaZapasow),
                    "wydanie_kluczy" => Ok(WpisyRodzaj::WydanieKluczy),
                    "odbior_kluczy" => Ok(WpisyRodzaj::OdbiorKluczy),
                    "przyjecie_gosci" => Ok(WpisyRodzaj::PrzyjecieGosci),
                    "wymiana_zamka" => Ok(WpisyRodzaj::WymianaZamka),
                    "prace_poza_lokalem" => Ok(WpisyRodzaj::PracePozaLokalem),
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

/// The values of column `wpisy.przyczyna`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyPrzyczyna {
    WyjazdGoscia,
    PrzyjazdGoscia,
    Zgloszenie,
    RaportWykonawcy,
    StandardLokalu,
    PrzygotowanieLokalu,
    ZamkniecieLokalu,
}

impl WpisyPrzyczyna {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyPrzyczyna] = &[
        WpisyPrzyczyna::WyjazdGoscia,
        WpisyPrzyczyna::PrzyjazdGoscia,
        WpisyPrzyczyna::Zgloszenie,
        WpisyPrzyczyna::RaportWykonawcy,
        WpisyPrzyczyna::StandardLokalu,
        WpisyPrzyczyna::PrzygotowanieLokalu,
        WpisyPrzyczyna::ZamkniecieLokalu,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyPrzyczyna::WyjazdGoscia => "wyjazd_goscia",
            WpisyPrzyczyna::PrzyjazdGoscia => "przyjazd_goscia",
            WpisyPrzyczyna::Zgloszenie => "zgloszenie",
            WpisyPrzyczyna::RaportWykonawcy => "raport_wykonawcy",
            WpisyPrzyczyna::StandardLokalu => "standard_lokalu",
            WpisyPrzyczyna::PrzygotowanieLokalu => "przygotowanie_lokalu",
            WpisyPrzyczyna::ZamkniecieLokalu => "zamkniecie_lokalu",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyPrzyczyna {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "wyjazd_goscia" => Ok(WpisyPrzyczyna::WyjazdGoscia),
                    "przyjazd_goscia" => Ok(WpisyPrzyczyna::PrzyjazdGoscia),
                    "zgloszenie" => Ok(WpisyPrzyczyna::Zgloszenie),
                    "raport_wykonawcy" => Ok(WpisyPrzyczyna::RaportWykonawcy),
                    "standard_lokalu" => Ok(WpisyPrzyczyna::StandardLokalu),
                    "przygotowanie_lokalu" => Ok(WpisyPrzyczyna::PrzygotowanieLokalu),
                    "zamkniecie_lokalu" => Ok(WpisyPrzyczyna::ZamkniecieLokalu),
                    other => Err(format!("{other} is no value of WpisyPrzyczyna")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyPrzyczyna {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.cena_zrodlo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyCenaZrodlo {
    Cennik,
    Wycena,
}

impl WpisyCenaZrodlo {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyCenaZrodlo] = &[
        WpisyCenaZrodlo::Cennik,
        WpisyCenaZrodlo::Wycena,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyCenaZrodlo::Cennik => "cennik",
            WpisyCenaZrodlo::Wycena => "wycena",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyCenaZrodlo {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "cennik" => Ok(WpisyCenaZrodlo::Cennik),
                    "wycena" => Ok(WpisyCenaZrodlo::Wycena),
                    other => Err(format!("{other} is no value of WpisyCenaZrodlo")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyCenaZrodlo {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyStan {
    Zlecone,
    Wstrzymane,
    Potwierdzone,
    Anulowane,
}

impl WpisyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyStan] = &[
        WpisyStan::Zlecone,
        WpisyStan::Wstrzymane,
        WpisyStan::Potwierdzone,
        WpisyStan::Anulowane,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyStan::Zlecone => "zlecone",
            WpisyStan::Wstrzymane => "wstrzymane",
            WpisyStan::Potwierdzone => "potwierdzone",
            WpisyStan::Anulowane => "anulowane",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "zlecone" => Ok(WpisyStan::Zlecone),
                    "wstrzymane" => Ok(WpisyStan::Wstrzymane),
                    "potwierdzone" => Ok(WpisyStan::Potwierdzone),
                    "anulowane" => Ok(WpisyStan::Anulowane),
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

/// The values of field `stan` of `zmien_stan_zlecenia`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stan {
    Zlecone,
    Wstrzymane,
    Anulowane,
}

impl Stan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[Stan] = &[Stan::Zlecone, Stan::Wstrzymane, Stan::Anulowane];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            Stan::Zlecone => "zlecone",
            Stan::Wstrzymane => "wstrzymane",
            Stan::Anulowane => "anulowane",
        }
    }
}

impl xmeta_room::FromSqlValue for Stan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "zlecone" => Ok(Stan::Zlecone),
                    "wstrzymane" => Ok(Stan::Wstrzymane),
                    "anulowane" => Ok(Stan::Anulowane),
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

/// A row of `dostepy_do_lokalu`.
#[derive(Debug, Clone, PartialEq)]
pub struct DostepyDoLokaluRow {
    pub id: i64,
    pub pobyt_id: Option<String>,
    pub zlecenie_id: Option<String>,
    pub rodzaj: DostepyRodzaj,
    pub kod: Option<String>,
    pub klucze_liczba: Option<i64>,
    pub zamek_odnosnik: Option<String>,
    pub wazny_od: String,
    pub wazny_do: String,
    pub kto: String,
    pub ustawiono: String,
    pub wydano: Option<String>,
    pub odebrano: Option<String>,
}

impl DostepyDoLokaluRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(DostepyDoLokaluRow {
            id: row.get("id")?,
            pobyt_id: row.get("pobyt_id")?,
            zlecenie_id: row.get("zlecenie_id")?,
            rodzaj: row.get("rodzaj")?,
            kod: row.get("kod")?,
            klucze_liczba: row.get("klucze_liczba")?,
            zamek_odnosnik: row.get("zamek_odnosnik")?,
            wazny_od: row.get("wazny_od")?,
            wazny_do: row.get("wazny_do")?,
            kto: row.get("kto")?,
            ustawiono: row.get("ustawiono")?,
            wydano: row.get("wydano")?,
            odebrano: row.get("odebrano")?,
        })
    }
}

/// Kody i klucze w kolejności początku ważności.
pub fn dostepy_do_lokalu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<DostepyDoLokaluRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_zlecenia_dostepy
ORDER BY wazny_od, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(DostepyDoLokaluRow::from_row).collect()
}

/// The parameters of `ustaw_dostep`.
#[derive(Debug, Clone, PartialEq)]
pub struct UstawDostepParams {
    pub pobyt_id: Option<String>,
    pub zlecenie_id: Option<String>,
    pub rodzaj: DostepyRodzaj,
    pub kod: Option<String>,
    pub klucze_liczba: Option<i64>,
    pub zamek_odnosnik: Option<String>,
    pub wazny_od: String,
    pub wazny_do: String,
    pub kto: String,
    pub ustawiono: String,
}

/// Kod do zamka albo klucze dla pobytu albo zlecenia, z czasem ważności i adresatem.
pub fn ustaw_dostep(
    room: &impl xmeta_room::MetaRoom,
    params: &UstawDostepParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_zlecenia_dostepy (pobyt_id, zlecenie_id, rodzaj, kod, klucze_liczba,
  zamek_odnosnik, wazny_od, wazny_do, kto, ustawiono)
VALUES (:pobyt_id, :zlecenie_id, :rodzaj, :kod, :klucze_liczba, :zamek_odnosnik, :wazny_od,
  :wazny_do, :kto, :ustawiono)"#,
                parameters: Vec::from([
                    ("pobyt_id", xmeta_room::ToSqlValue::to_sql_value(&params.pobyt_id)),
                    (
                        "zlecenie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zlecenie_id),
                    ),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("kod", xmeta_room::ToSqlValue::to_sql_value(&params.kod)),
                    (
                        "klucze_liczba",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klucze_liczba),
                    ),
                    (
                        "zamek_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zamek_odnosnik),
                    ),
                    ("wazny_od", xmeta_room::ToSqlValue::to_sql_value(&params.wazny_od)),
                    ("wazny_do", xmeta_room::ToSqlValue::to_sql_value(&params.wazny_do)),
                    ("kto", xmeta_room::ToSqlValue::to_sql_value(&params.kto)),
                    (
                        "ustawiono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.ustawiono),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_wydanie_dostepu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujWydanieDostepuParams {
    pub wydano: String,
    pub id: i64,
}

/// Kod wysłany gościowi albo wykonawcy, albo klucze wydane; wpisuje się raz.
pub fn odnotuj_wydanie_dostepu(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWydanieDostepuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_zlecenia_dostepy
SET wydano = :wydano
WHERE id = :id"#,
                parameters: Vec::from([
                    ("wydano", xmeta_room::ToSqlValue::to_sql_value(&params.wydano)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_odebranie_dostepu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujOdebranieDostepuParams {
    pub odebrano: String,
    pub id: i64,
}

/// Kod wygaszony albo klucze odebrane; wpisuje się raz.
pub fn odnotuj_odebranie_dostepu(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujOdebranieDostepuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_zlecenia_dostepy
SET odebrano = :odebrano
WHERE id = :id"#,
                parameters: Vec::from([
                    ("odebrano", xmeta_room::ToSqlValue::to_sql_value(&params.odebrano)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `zlecenia`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZleceniaRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub przyczyna: WpisyPrzyczyna,
    pub pobyt_id: Option<String>,
    pub zgloszenie_id: Option<String>,
    pub wykonawca_odnosnik: String,
    pub spoza_rejestru: bool,
    pub cennik_wersja_odnosnik: Option<String>,
    pub poza_standardem: bool,
    pub okno_od: String,
    pub okno_do: String,
    pub cena: i64,
    pub cena_zrodlo: WpisyCenaZrodlo,
    pub zgoda_id: Option<String>,
    pub decyzja_id: Option<String>,
    pub zlecil: String,
    pub zlecono: String,
    pub raport_sciezka: Option<String>,
    pub raport_skrot: Option<String>,
    pub potwierdzil: Option<String>,
    pub potwierdzono: Option<String>,
    pub koszt: Option<i64>,
    pub koszt_decyzja_id: Option<String>,
    pub stan: WpisyStan,
}

impl ZleceniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZleceniaRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            przyczyna: row.get("przyczyna")?,
            pobyt_id: row.get("pobyt_id")?,
            zgloszenie_id: row.get("zgloszenie_id")?,
            wykonawca_odnosnik: row.get("wykonawca_odnosnik")?,
            spoza_rejestru: row.get("spoza_rejestru")?,
            cennik_wersja_odnosnik: row.get("cennik_wersja_odnosnik")?,
            poza_standardem: row.get("poza_standardem")?,
            okno_od: row.get("okno_od")?,
            okno_do: row.get("okno_do")?,
            cena: row.get("cena")?,
            cena_zrodlo: row.get("cena_zrodlo")?,
            zgoda_id: row.get("zgoda_id")?,
            decyzja_id: row.get("decyzja_id")?,
            zlecil: row.get("zlecil")?,
            zlecono: row.get("zlecono")?,
            raport_sciezka: row.get("raport_sciezka")?,
            raport_skrot: row.get("raport_skrot")?,
            potwierdzil: row.get("potwierdzil")?,
            potwierdzono: row.get("potwierdzono")?,
            koszt: row.get("koszt")?,
            koszt_decyzja_id: row.get("koszt_decyzja_id")?,
            stan: row.get("stan")?,
        })
    }
}

/// Zlecenia w kolejności okna pracy.
pub fn zlecenia(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZleceniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_zlecenia_wpisy
ORDER BY okno_od, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZleceniaRow::from_row).collect()
}

/// A row of `zlecenia_otwarte`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZleceniaOtwarteRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub przyczyna: WpisyPrzyczyna,
    pub pobyt_id: Option<String>,
    pub zgloszenie_id: Option<String>,
    pub wykonawca_odnosnik: String,
    pub spoza_rejestru: bool,
    pub cennik_wersja_odnosnik: Option<String>,
    pub poza_standardem: bool,
    pub okno_od: String,
    pub okno_do: String,
    pub cena: i64,
    pub cena_zrodlo: WpisyCenaZrodlo,
    pub zgoda_id: Option<String>,
    pub decyzja_id: Option<String>,
    pub zlecil: String,
    pub zlecono: String,
    pub raport_sciezka: Option<String>,
    pub raport_skrot: Option<String>,
    pub potwierdzil: Option<String>,
    pub potwierdzono: Option<String>,
    pub koszt: Option<i64>,
    pub koszt_decyzja_id: Option<String>,
    pub stan: WpisyStan,
}

impl ZleceniaOtwarteRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZleceniaOtwarteRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            przyczyna: row.get("przyczyna")?,
            pobyt_id: row.get("pobyt_id")?,
            zgloszenie_id: row.get("zgloszenie_id")?,
            wykonawca_odnosnik: row.get("wykonawca_odnosnik")?,
            spoza_rejestru: row.get("spoza_rejestru")?,
            cennik_wersja_odnosnik: row.get("cennik_wersja_odnosnik")?,
            poza_standardem: row.get("poza_standardem")?,
            okno_od: row.get("okno_od")?,
            okno_do: row.get("okno_do")?,
            cena: row.get("cena")?,
            cena_zrodlo: row.get("cena_zrodlo")?,
            zgoda_id: row.get("zgoda_id")?,
            decyzja_id: row.get("decyzja_id")?,
            zlecil: row.get("zlecil")?,
            zlecono: row.get("zlecono")?,
            raport_sciezka: row.get("raport_sciezka")?,
            raport_skrot: row.get("raport_skrot")?,
            potwierdzil: row.get("potwierdzil")?,
            potwierdzono: row.get("potwierdzono")?,
            koszt: row.get("koszt")?,
            koszt_decyzja_id: row.get("koszt_decyzja_id")?,
            stan: row.get("stan")?,
        })
    }
}

/// Zlecenia zlecone i wstrzymane w kolejności okna pracy.
pub fn zlecenia_otwarte(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZleceniaOtwarteRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_zlecenia_wpisy
WHERE stan IN ('zlecone', 'wstrzymane')
ORDER BY okno_od, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZleceniaOtwarteRow::from_row).collect()
}

/// The parameters of `zlec_prace`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZlecPraceParams {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub przyczyna: WpisyPrzyczyna,
    pub pobyt_id: Option<String>,
    pub zgloszenie_id: Option<String>,
    pub wykonawca_odnosnik: String,
    pub spoza_rejestru: bool,
    pub cennik_wersja_odnosnik: Option<String>,
    pub poza_standardem: bool,
    pub okno_od: String,
    pub okno_do: String,
    pub cena: i64,
    pub cena_zrodlo: WpisyCenaZrodlo,
    pub zgoda_id: Option<String>,
    pub decyzja_id: Option<String>,
    pub zlecil: String,
    pub zlecono: String,
}

/// Zlecenie dla wykonawcy z rejestru albo spoza niego, z oknem i ceną; gdy ma powód zgody,
/// wskazuje decyzję człowieka i ewentualnie zgodę właściciela. Identyfikator (UUID) nadaje
/// aplikacja, bo wskazują go decyzja, zgoda, noce, dostępy i pozycje zestawień.
pub fn zlec_prace(
    room: &impl xmeta_room::MetaRoom,
    params: &ZlecPraceParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_zlecenia_wpisy (id, rodzaj, przyczyna, pobyt_id, zgloszenie_id,
  wykonawca_odnosnik, spoza_rejestru, cennik_wersja_odnosnik, poza_standardem, okno_od, okno_do, cena,
  cena_zrodlo, zgoda_id, decyzja_id, zlecil, zlecono, stan)
VALUES (:id, :rodzaj, :przyczyna, :pobyt_id, :zgloszenie_id, :wykonawca_odnosnik, :spoza_rejestru,
  :cennik_wersja_odnosnik, :poza_standardem, :okno_od, :okno_do, :cena, :cena_zrodlo, :zgoda_id,
  :decyzja_id, :zlecil, :zlecono, 'zlecone')"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    (
                        "przyczyna",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przyczyna),
                    ),
                    ("pobyt_id", xmeta_room::ToSqlValue::to_sql_value(&params.pobyt_id)),
                    (
                        "zgloszenie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zgloszenie_id),
                    ),
                    (
                        "wykonawca_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wykonawca_odnosnik),
                    ),
                    (
                        "spoza_rejestru",
                        xmeta_room::ToSqlValue::to_sql_value(&params.spoza_rejestru),
                    ),
                    (
                        "cennik_wersja_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.cennik_wersja_odnosnik,
                        ),
                    ),
                    (
                        "poza_standardem",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poza_standardem),
                    ),
                    ("okno_od", xmeta_room::ToSqlValue::to_sql_value(&params.okno_od)),
                    ("okno_do", xmeta_room::ToSqlValue::to_sql_value(&params.okno_do)),
                    ("cena", xmeta_room::ToSqlValue::to_sql_value(&params.cena)),
                    (
                        "cena_zrodlo",
                        xmeta_room::ToSqlValue::to_sql_value(&params.cena_zrodlo),
                    ),
                    ("zgoda_id", xmeta_room::ToSqlValue::to_sql_value(&params.zgoda_id)),
                    (
                        "decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_id),
                    ),
                    ("zlecil", xmeta_room::ToSqlValue::to_sql_value(&params.zlecil)),
                    ("zlecono", xmeta_room::ToSqlValue::to_sql_value(&params.zlecono)),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_stan_zlecenia`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZmienStanZleceniaParams {
    pub stan: Stan,
    pub id: String,
}

/// Wstrzymanie, wznowienie albo anulowanie zlecenia.
pub fn zmien_stan_zlecenia(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienStanZleceniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_zlecenia_wpisy
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

/// The parameters of `potwierdz_wykonanie_zlecenia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PotwierdzWykonanieZleceniaParams {
    pub raport_sciezka: String,
    pub raport_skrot: String,
    pub potwierdzil: String,
    pub potwierdzono: String,
    pub id: String,
}

/// Raport wykonawcy ze zdjęciami porównany z listą kontrolną i potwierdzenie wykonania.
pub fn potwierdz_wykonanie_zlecenia(
    room: &impl xmeta_room::MetaRoom,
    params: &PotwierdzWykonanieZleceniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_zlecenia_wpisy
SET stan = 'potwierdzone',
  raport_sciezka = :raport_sciezka,
  raport_skrot = :raport_skrot,
  potwierdzil = :potwierdzil,
  potwierdzono = :potwierdzono
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "raport_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.raport_sciezka),
                    ),
                    (
                        "raport_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.raport_skrot),
                    ),
                    (
                        "potwierdzil",
                        xmeta_room::ToSqlValue::to_sql_value(&params.potwierdzil),
                    ),
                    (
                        "potwierdzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.potwierdzono),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wpisz_koszt_zlecenia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszKosztZleceniaParams {
    pub koszt: i64,
    pub koszt_decyzja_id: Option<String>,
    pub id: String,
}

/// Koszt po potwierdzeniu wykonania; inny niż cena wskazuje decyzję człowieka.
pub fn wpisz_koszt_zlecenia(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszKosztZleceniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_zlecenia_wpisy
SET koszt = :koszt,
  koszt_decyzja_id = :koszt_decyzja_id
WHERE id = :id"#,
                parameters: Vec::from([
                    ("koszt", xmeta_room::ToSqlValue::to_sql_value(&params.koszt)),
                    (
                        "koszt_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.koszt_decyzja_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
