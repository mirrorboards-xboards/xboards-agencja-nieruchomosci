//! The kind `sprawa-korespondencja` of the room `cases/poszukiwanie`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "sprawa-korespondencja";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Korespondencja teczki: wysyłki, które biuro wysłało w sprawie, i wpływy z zewnątrz, z dokumentami,
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
"#,
];

/// The values of column `wplywy.kanal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WplywyKanal {
    Email,
    Sms,
    Portal,
    KanalSprzedazy,
    List,
    PokojWymiany,
    Telefon,
    Osobiscie,
}

impl WplywyKanal {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WplywyKanal] = &[
        WplywyKanal::Email,
        WplywyKanal::Sms,
        WplywyKanal::Portal,
        WplywyKanal::KanalSprzedazy,
        WplywyKanal::List,
        WplywyKanal::PokojWymiany,
        WplywyKanal::Telefon,
        WplywyKanal::Osobiscie,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WplywyKanal::Email => "email",
            WplywyKanal::Sms => "sms",
            WplywyKanal::Portal => "portal",
            WplywyKanal::KanalSprzedazy => "kanal_sprzedazy",
            WplywyKanal::List => "list",
            WplywyKanal::PokojWymiany => "pokoj_wymiany",
            WplywyKanal::Telefon => "telefon",
            WplywyKanal::Osobiscie => "osobiscie",
        }
    }
}

impl xmeta_room::FromSqlValue for WplywyKanal {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "email" => Ok(WplywyKanal::Email),
                    "sms" => Ok(WplywyKanal::Sms),
                    "portal" => Ok(WplywyKanal::Portal),
                    "kanal_sprzedazy" => Ok(WplywyKanal::KanalSprzedazy),
                    "list" => Ok(WplywyKanal::List),
                    "pokoj_wymiany" => Ok(WplywyKanal::PokojWymiany),
                    "telefon" => Ok(WplywyKanal::Telefon),
                    "osobiscie" => Ok(WplywyKanal::Osobiscie),
                    other => Err(format!("{other} is no value of WplywyKanal")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WplywyKanal {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wysylki.adresat_rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WysylkiAdresatRodzaj {
    KancelariaNotarialna,
    Bank,
    Spoldzielnia,
    Wspolnota,
    Zarzadca,
    Sad,
    Urzad,
    Gmina,
    KanalSprzedazy,
    Wykonawca,
    Inny,
}

impl WysylkiAdresatRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WysylkiAdresatRodzaj] = &[
        WysylkiAdresatRodzaj::KancelariaNotarialna,
        WysylkiAdresatRodzaj::Bank,
        WysylkiAdresatRodzaj::Spoldzielnia,
        WysylkiAdresatRodzaj::Wspolnota,
        WysylkiAdresatRodzaj::Zarzadca,
        WysylkiAdresatRodzaj::Sad,
        WysylkiAdresatRodzaj::Urzad,
        WysylkiAdresatRodzaj::Gmina,
        WysylkiAdresatRodzaj::KanalSprzedazy,
        WysylkiAdresatRodzaj::Wykonawca,
        WysylkiAdresatRodzaj::Inny,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WysylkiAdresatRodzaj::KancelariaNotarialna => "kancelaria_notarialna",
            WysylkiAdresatRodzaj::Bank => "bank",
            WysylkiAdresatRodzaj::Spoldzielnia => "spoldzielnia",
            WysylkiAdresatRodzaj::Wspolnota => "wspolnota",
            WysylkiAdresatRodzaj::Zarzadca => "zarzadca",
            WysylkiAdresatRodzaj::Sad => "sad",
            WysylkiAdresatRodzaj::Urzad => "urzad",
            WysylkiAdresatRodzaj::Gmina => "gmina",
            WysylkiAdresatRodzaj::KanalSprzedazy => "kanal_sprzedazy",
            WysylkiAdresatRodzaj::Wykonawca => "wykonawca",
            WysylkiAdresatRodzaj::Inny => "inny",
        }
    }
}

impl xmeta_room::FromSqlValue for WysylkiAdresatRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "kancelaria_notarialna" => {
                        Ok(WysylkiAdresatRodzaj::KancelariaNotarialna)
                    }
                    "bank" => Ok(WysylkiAdresatRodzaj::Bank),
                    "spoldzielnia" => Ok(WysylkiAdresatRodzaj::Spoldzielnia),
                    "wspolnota" => Ok(WysylkiAdresatRodzaj::Wspolnota),
                    "zarzadca" => Ok(WysylkiAdresatRodzaj::Zarzadca),
                    "sad" => Ok(WysylkiAdresatRodzaj::Sad),
                    "urzad" => Ok(WysylkiAdresatRodzaj::Urzad),
                    "gmina" => Ok(WysylkiAdresatRodzaj::Gmina),
                    "kanal_sprzedazy" => Ok(WysylkiAdresatRodzaj::KanalSprzedazy),
                    "wykonawca" => Ok(WysylkiAdresatRodzaj::Wykonawca),
                    "inny" => Ok(WysylkiAdresatRodzaj::Inny),
                    other => Err(format!("{other} is no value of WysylkiAdresatRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WysylkiAdresatRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wysylki.w_imieniu`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WysylkiWImieniu {
    Biuro,
    Uczestnik,
}

impl WysylkiWImieniu {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WysylkiWImieniu] = &[
        WysylkiWImieniu::Biuro,
        WysylkiWImieniu::Uczestnik,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WysylkiWImieniu::Biuro => "biuro",
            WysylkiWImieniu::Uczestnik => "uczestnik",
        }
    }
}

impl xmeta_room::FromSqlValue for WysylkiWImieniu {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "biuro" => Ok(WysylkiWImieniu::Biuro),
                    "uczestnik" => Ok(WysylkiWImieniu::Uczestnik),
                    other => Err(format!("{other} is no value of WysylkiWImieniu")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WysylkiWImieniu {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wysylki.kanal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WysylkiKanal {
    Email,
    Sms,
    Portal,
    KanalSprzedazy,
    List,
    PokojWymiany,
}

impl WysylkiKanal {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WysylkiKanal] = &[
        WysylkiKanal::Email,
        WysylkiKanal::Sms,
        WysylkiKanal::Portal,
        WysylkiKanal::KanalSprzedazy,
        WysylkiKanal::List,
        WysylkiKanal::PokojWymiany,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WysylkiKanal::Email => "email",
            WysylkiKanal::Sms => "sms",
            WysylkiKanal::Portal => "portal",
            WysylkiKanal::KanalSprzedazy => "kanal_sprzedazy",
            WysylkiKanal::List => "list",
            WysylkiKanal::PokojWymiany => "pokoj_wymiany",
        }
    }
}

impl xmeta_room::FromSqlValue for WysylkiKanal {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "email" => Ok(WysylkiKanal::Email),
                    "sms" => Ok(WysylkiKanal::Sms),
                    "portal" => Ok(WysylkiKanal::Portal),
                    "kanal_sprzedazy" => Ok(WysylkiKanal::KanalSprzedazy),
                    "list" => Ok(WysylkiKanal::List),
                    "pokoj_wymiany" => Ok(WysylkiKanal::PokojWymiany),
                    other => Err(format!("{other} is no value of WysylkiKanal")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WysylkiKanal {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wysylki.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WysylkiStan {
    Projekt,
    CzekaNaZgode,
    Wyslana,
    Doreczona,
    Zwrocona,
    Anulowana,
}

impl WysylkiStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WysylkiStan] = &[
        WysylkiStan::Projekt,
        WysylkiStan::CzekaNaZgode,
        WysylkiStan::Wyslana,
        WysylkiStan::Doreczona,
        WysylkiStan::Zwrocona,
        WysylkiStan::Anulowana,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WysylkiStan::Projekt => "projekt",
            WysylkiStan::CzekaNaZgode => "czeka_na_zgode",
            WysylkiStan::Wyslana => "wyslana",
            WysylkiStan::Doreczona => "doreczona",
            WysylkiStan::Zwrocona => "zwrocona",
            WysylkiStan::Anulowana => "anulowana",
        }
    }
}

impl xmeta_room::FromSqlValue for WysylkiStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "projekt" => Ok(WysylkiStan::Projekt),
                    "czeka_na_zgode" => Ok(WysylkiStan::CzekaNaZgode),
                    "wyslana" => Ok(WysylkiStan::Wyslana),
                    "doreczona" => Ok(WysylkiStan::Doreczona),
                    "zwrocona" => Ok(WysylkiStan::Zwrocona),
                    "anulowana" => Ok(WysylkiStan::Anulowana),
                    other => Err(format!("{other} is no value of WysylkiStan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WysylkiStan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `wplywy`.
#[derive(Debug, Clone, PartialEq)]
pub struct WplywyRow {
    pub id: String,
    pub nadawca: String,
    pub nadawca_uczestnik_id: Option<String>,
    pub kanal: WplywyKanal,
    pub wplynelo: String,
    pub sciezka: String,
    pub skrot: String,
    pub niezgodny_kontakt: bool,
    pub niezgodnosc_opis: Option<String>,
    pub odnosnik_zewnetrzny: Option<String>,
}

impl WplywyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WplywyRow {
            id: row.get("id")?,
            nadawca: row.get("nadawca")?,
            nadawca_uczestnik_id: row.get("nadawca_uczestnik_id")?,
            kanal: row.get("kanal")?,
            wplynelo: row.get("wplynelo")?,
            sciezka: row.get("sciezka")?,
            skrot: row.get("skrot")?,
            niezgodny_kontakt: row.get("niezgodny_kontakt")?,
            niezgodnosc_opis: row.get("niezgodnosc_opis")?,
            odnosnik_zewnetrzny: row.get("odnosnik_zewnetrzny")?,
        })
    }
}

/// Wpływy teczki, najnowsze pierwsze.
pub fn wplywy(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WplywyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_korespondencja_wplywy
ORDER BY wplynelo DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(WplywyRow::from_row).collect()
}

/// The parameters of `wpisz_wplyw`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszWplywParams {
    pub id: String,
    pub nadawca: String,
    pub nadawca_uczestnik_id: Option<String>,
    pub kanal: WplywyKanal,
    pub wplynelo: String,
    pub sciezka: String,
    pub skrot: String,
    pub niezgodny_kontakt: bool,
    pub niezgodnosc_opis: Option<String>,
    pub odnosnik_zewnetrzny: Option<String>,
}

/// Wiadomość z zewnątrz z nadawcą, kanałem, czasem i plikiem; gdy nadawca albo kanał nie pasuje do
/// kontaktów w teczce, agent zaznacza to z opisem.
pub fn wpisz_wplyw(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszWplywParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_korespondencja_wplywy (id, nadawca, nadawca_uczestnik_id, kanal, wplynelo,
  sciezka, skrot, niezgodny_kontakt, niezgodnosc_opis, odnosnik_zewnetrzny)
VALUES (:id, :nadawca, :nadawca_uczestnik_id, :kanal, :wplynelo, :sciezka, :skrot,
  :niezgodny_kontakt, :niezgodnosc_opis, :odnosnik_zewnetrzny)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("nadawca", xmeta_room::ToSqlValue::to_sql_value(&params.nadawca)),
                    (
                        "nadawca_uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.nadawca_uczestnik_id,
                        ),
                    ),
                    ("kanal", xmeta_room::ToSqlValue::to_sql_value(&params.kanal)),
                    ("wplynelo", xmeta_room::ToSqlValue::to_sql_value(&params.wplynelo)),
                    ("sciezka", xmeta_room::ToSqlValue::to_sql_value(&params.sciezka)),
                    ("skrot", xmeta_room::ToSqlValue::to_sql_value(&params.skrot)),
                    (
                        "niezgodny_kontakt",
                        xmeta_room::ToSqlValue::to_sql_value(&params.niezgodny_kontakt),
                    ),
                    (
                        "niezgodnosc_opis",
                        xmeta_room::ToSqlValue::to_sql_value(&params.niezgodnosc_opis),
                    ),
                    (
                        "odnosnik_zewnetrzny",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odnosnik_zewnetrzny),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `wysylki`.
#[derive(Debug, Clone, PartialEq)]
pub struct WysylkiRow {
    pub id: String,
    pub adresat_uczestnik_id: Option<String>,
    pub adresat_rodzaj: Option<WysylkiAdresatRodzaj>,
    pub adresat_nazwa: Option<String>,
    pub w_imieniu: WysylkiWImieniu,
    pub w_imieniu_uczestnik_id: Option<String>,
    pub umocowanie_id: Option<String>,
    pub czynnosc: Option<String>,
    pub kanal: WysylkiKanal,
    pub wzor: Option<String>,
    pub wzor_wersja: Option<String>,
    pub decyzja_id: Option<String>,
    pub zamawia_dokument_id: Option<String>,
    pub odnosnik_zewnetrzny: Option<String>,
    pub sciezka: Option<String>,
    pub skrot: Option<String>,
    pub stan: WysylkiStan,
    pub wyslano: Option<String>,
    pub doreczono: Option<String>,
    pub dowod_doreczenia: Option<String>,
    pub informacja_rodo: bool,
    pub ujawnia_agenta: bool,
}

impl WysylkiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WysylkiRow {
            id: row.get("id")?,
            adresat_uczestnik_id: row.get("adresat_uczestnik_id")?,
            adresat_rodzaj: row.get("adresat_rodzaj")?,
            adresat_nazwa: row.get("adresat_nazwa")?,
            w_imieniu: row.get("w_imieniu")?,
            w_imieniu_uczestnik_id: row.get("w_imieniu_uczestnik_id")?,
            umocowanie_id: row.get("umocowanie_id")?,
            czynnosc: row.get("czynnosc")?,
            kanal: row.get("kanal")?,
            wzor: row.get("wzor")?,
            wzor_wersja: row.get("wzor_wersja")?,
            decyzja_id: row.get("decyzja_id")?,
            zamawia_dokument_id: row.get("zamawia_dokument_id")?,
            odnosnik_zewnetrzny: row.get("odnosnik_zewnetrzny")?,
            sciezka: row.get("sciezka")?,
            skrot: row.get("skrot")?,
            stan: row.get("stan")?,
            wyslano: row.get("wyslano")?,
            doreczono: row.get("doreczono")?,
            dowod_doreczenia: row.get("dowod_doreczenia")?,
            informacja_rodo: row.get("informacja_rodo")?,
            ujawnia_agenta: row.get("ujawnia_agenta")?,
        })
    }
}

/// Wysyłki teczki, najnowsze pierwsze.
pub fn wysylki(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WysylkiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_korespondencja_wysylki
ORDER BY rowid DESC"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(WysylkiRow::from_row).collect()
}

/// The parameters of `wysylki_do_uczestnika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WysylkiDoUczestnikaParams {
    pub adresat_uczestnik_id: String,
}

/// A row of `wysylki_do_uczestnika`.
#[derive(Debug, Clone, PartialEq)]
pub struct WysylkiDoUczestnikaRow {
    pub id: String,
    pub adresat_uczestnik_id: Option<String>,
    pub adresat_rodzaj: Option<WysylkiAdresatRodzaj>,
    pub adresat_nazwa: Option<String>,
    pub w_imieniu: WysylkiWImieniu,
    pub w_imieniu_uczestnik_id: Option<String>,
    pub umocowanie_id: Option<String>,
    pub czynnosc: Option<String>,
    pub kanal: WysylkiKanal,
    pub wzor: Option<String>,
    pub wzor_wersja: Option<String>,
    pub decyzja_id: Option<String>,
    pub zamawia_dokument_id: Option<String>,
    pub odnosnik_zewnetrzny: Option<String>,
    pub sciezka: Option<String>,
    pub skrot: Option<String>,
    pub stan: WysylkiStan,
    pub wyslano: Option<String>,
    pub doreczono: Option<String>,
    pub dowod_doreczenia: Option<String>,
    pub informacja_rodo: bool,
    pub ujawnia_agenta: bool,
}

impl WysylkiDoUczestnikaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WysylkiDoUczestnikaRow {
            id: row.get("id")?,
            adresat_uczestnik_id: row.get("adresat_uczestnik_id")?,
            adresat_rodzaj: row.get("adresat_rodzaj")?,
            adresat_nazwa: row.get("adresat_nazwa")?,
            w_imieniu: row.get("w_imieniu")?,
            w_imieniu_uczestnik_id: row.get("w_imieniu_uczestnik_id")?,
            umocowanie_id: row.get("umocowanie_id")?,
            czynnosc: row.get("czynnosc")?,
            kanal: row.get("kanal")?,
            wzor: row.get("wzor")?,
            wzor_wersja: row.get("wzor_wersja")?,
            decyzja_id: row.get("decyzja_id")?,
            zamawia_dokument_id: row.get("zamawia_dokument_id")?,
            odnosnik_zewnetrzny: row.get("odnosnik_zewnetrzny")?,
            sciezka: row.get("sciezka")?,
            skrot: row.get("skrot")?,
            stan: row.get("stan")?,
            wyslano: row.get("wyslano")?,
            doreczono: row.get("doreczono")?,
            dowod_doreczenia: row.get("dowod_doreczenia")?,
            informacja_rodo: row.get("informacja_rodo")?,
            ujawnia_agenta: row.get("ujawnia_agenta")?,
        })
    }
}

/// Wysyłki do jednego uczestnika; z nich nowy agent wie, czy rozmówca dostał już informację RODO
/// i informację, że pisze agent.
pub fn wysylki_do_uczestnika(
    room: &impl xmeta_room::MetaRoom,
    params: &WysylkiDoUczestnikaParams,
) -> Result<Vec<WysylkiDoUczestnikaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_korespondencja_wysylki
WHERE adresat_uczestnik_id = :adresat_uczestnik_id
ORDER BY rowid"#,
        parameters: Vec::from([
            (
                "adresat_uczestnik_id",
                xmeta_room::ToSqlValue::to_sql_value(&params.adresat_uczestnik_id),
            ),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(WysylkiDoUczestnikaRow::from_row).collect()
}

/// The parameters of `przygotuj_wysylke`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzygotujWysylkeParams {
    pub id: String,
    pub adresat_uczestnik_id: Option<String>,
    pub adresat_rodzaj: Option<WysylkiAdresatRodzaj>,
    pub adresat_nazwa: Option<String>,
    pub w_imieniu: WysylkiWImieniu,
    pub w_imieniu_uczestnik_id: Option<String>,
    pub umocowanie_id: Option<String>,
    pub czynnosc: Option<String>,
    pub kanal: WysylkiKanal,
    pub wzor: Option<String>,
    pub wzor_wersja: Option<String>,
    pub zamawia_dokument_id: Option<String>,
    pub sciezka: Option<String>,
    pub skrot: Option<String>,
    pub informacja_rodo: bool,
    pub ujawnia_agenta: bool,
}

/// Projekt wysyłki z adresatem, kanałem, wzorem i plikiem treści. Wzór się potem nie zmienia;
/// inny wzór to nowy projekt. Identyfikator (UUID) nadaje aplikacja, bo wskazują go decyzja
/// i załączniki.
pub fn przygotuj_wysylke(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzygotujWysylkeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_korespondencja_wysylki (id, adresat_uczestnik_id, adresat_rodzaj, adresat_nazwa,
  w_imieniu, w_imieniu_uczestnik_id, umocowanie_id, czynnosc, kanal, wzor, wzor_wersja,
  zamawia_dokument_id, sciezka, skrot, informacja_rodo, ujawnia_agenta, stan)
VALUES (:id, :adresat_uczestnik_id, :adresat_rodzaj, :adresat_nazwa, :w_imieniu,
  :w_imieniu_uczestnik_id, :umocowanie_id, :czynnosc, :kanal, :wzor, :wzor_wersja,
  :zamawia_dokument_id, :sciezka, :skrot, :informacja_rodo, :ujawnia_agenta, 'projekt')"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "adresat_uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.adresat_uczestnik_id,
                        ),
                    ),
                    (
                        "adresat_rodzaj",
                        xmeta_room::ToSqlValue::to_sql_value(&params.adresat_rodzaj),
                    ),
                    (
                        "adresat_nazwa",
                        xmeta_room::ToSqlValue::to_sql_value(&params.adresat_nazwa),
                    ),
                    (
                        "w_imieniu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.w_imieniu),
                    ),
                    (
                        "w_imieniu_uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.w_imieniu_uczestnik_id,
                        ),
                    ),
                    (
                        "umocowanie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.umocowanie_id),
                    ),
                    ("czynnosc", xmeta_room::ToSqlValue::to_sql_value(&params.czynnosc)),
                    ("kanal", xmeta_room::ToSqlValue::to_sql_value(&params.kanal)),
                    ("wzor", xmeta_room::ToSqlValue::to_sql_value(&params.wzor)),
                    (
                        "wzor_wersja",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wzor_wersja),
                    ),
                    (
                        "zamawia_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zamawia_dokument_id),
                    ),
                    ("sciezka", xmeta_room::ToSqlValue::to_sql_value(&params.sciezka)),
                    ("skrot", xmeta_room::ToSqlValue::to_sql_value(&params.skrot)),
                    (
                        "informacja_rodo",
                        xmeta_room::ToSqlValue::to_sql_value(&params.informacja_rodo),
                    ),
                    (
                        "ujawnia_agenta",
                        xmeta_room::ToSqlValue::to_sql_value(&params.ujawnia_agenta),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_tresc_wysylki`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZmienTrescWysylkiParams {
    pub sciezka: Option<String>,
    pub skrot: Option<String>,
    pub id: String,
}

/// Nowa wersja pliku treści; zmienia się tylko w projekcie, przed prośbą o zgodę i wysłaniem.
pub fn zmien_tresc_wysylki(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienTrescWysylkiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_korespondencja_wysylki
SET sciezka = :sciezka,
  skrot = :skrot
WHERE id = :id"#,
                parameters: Vec::from([
                    ("sciezka", xmeta_room::ToSqlValue::to_sql_value(&params.sciezka)),
                    ("skrot", xmeta_room::ToSqlValue::to_sql_value(&params.skrot)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `przekaz_wysylke_do_zgody`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrzekazWysylkeDoZgodyParams {
    pub id: String,
}

/// Wysyłka, która według profilu wymaga zgody człowieka, i każde pismo spoza wzorów czeka na
/// decyzję „wyslanie”. Od tej chwili treść się nie zmienia, a o decyzję prosi się dopiero teraz.
pub fn przekaz_wysylke_do_zgody(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzekazWysylkeDoZgodyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_korespondencja_wysylki
SET stan = 'czeka_na_zgode'
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_wyslanie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujWyslanieParams {
    pub wyslano: String,
    pub decyzja_id: Option<String>,
    pub odnosnik_zewnetrzny: Option<String>,
    pub id: String,
}

/// Wysyłka wyszła; pismo za zgodą wskazuje zatwierdzoną decyzję.
pub fn odnotuj_wyslanie(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWyslanieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_korespondencja_wysylki
SET stan = 'wyslana',
  wyslano = :wyslano,
  decyzja_id = :decyzja_id,
  odnosnik_zewnetrzny = :odnosnik_zewnetrzny
WHERE id = :id"#,
                parameters: Vec::from([
                    ("wyslano", xmeta_room::ToSqlValue::to_sql_value(&params.wyslano)),
                    (
                        "decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_id),
                    ),
                    (
                        "odnosnik_zewnetrzny",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odnosnik_zewnetrzny),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_doreczenie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujDoreczenieParams {
    pub doreczono: String,
    pub dowod_doreczenia: Option<String>,
    pub id: String,
}

/// Adresat dostał wysyłkę; dowód, gdy kanał go daje.
pub fn odnotuj_doreczenie(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujDoreczenieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_korespondencja_wysylki
SET stan = 'doreczona',
  doreczono = :doreczono,
  dowod_doreczenia = :dowod_doreczenia
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "doreczono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.doreczono),
                    ),
                    (
                        "dowod_doreczenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dowod_doreczenia),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_zwrot`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujZwrotParams {
    pub dowod_doreczenia: Option<String>,
    pub id: String,
}

/// Wysyłka wróciła niedoręczona.
pub fn odnotuj_zwrot(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujZwrotParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_korespondencja_wysylki
SET stan = 'zwrocona',
  dowod_doreczenia = :dowod_doreczenia
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "dowod_doreczenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dowod_doreczenia),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `anuluj_wysylke`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AnulujWysylkeParams {
    pub id: String,
}

/// Projekt, który nie wyjdzie.
pub fn anuluj_wysylke(
    room: &impl xmeta_room::MetaRoom,
    params: &AnulujWysylkeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_korespondencja_wysylki
SET stan = 'anulowana'
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `zalaczniki`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZalacznikiRow {
    pub id: i64,
    pub wysylka_id: Option<String>,
    pub wplyw_id: Option<String>,
    pub dokument_id: String,
}

impl ZalacznikiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZalacznikiRow {
            id: row.get("id")?,
            wysylka_id: row.get("wysylka_id")?,
            wplyw_id: row.get("wplyw_id")?,
            dokument_id: row.get("dokument_id")?,
        })
    }
}

/// Dokumenty, które wyszły z wysyłkami albo przyszły z wpływami.
pub fn zalaczniki(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZalacznikiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_korespondencja_zalaczniki
ORDER BY id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(ZalacznikiRow::from_row).collect()
}

/// The parameters of `dolacz_dokument_do_wysylki`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DolaczDokumentDoWysylkiParams {
    pub wysylka_id: String,
    pub dokument_id: String,
}

/// Dokument z listy, który wychodzi razem z wysyłką, np. kopia polisy dla klienta; dołącza się
/// go tylko do projektu.
pub fn dolacz_dokument_do_wysylki(
    room: &impl xmeta_room::MetaRoom,
    params: &DolaczDokumentDoWysylkiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_korespondencja_zalaczniki (wysylka_id, dokument_id)
VALUES (:wysylka_id, :dokument_id)"#,
                parameters: Vec::from([
                    (
                        "wysylka_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wysylka_id),
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

/// The parameters of `dolacz_dokument_do_wplywu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DolaczDokumentDoWplywuParams {
    pub wplyw_id: String,
    pub dokument_id: String,
}

/// Dokument z listy, który przyszedł z wpływem.
pub fn dolacz_dokument_do_wplywu(
    room: &impl xmeta_room::MetaRoom,
    params: &DolaczDokumentDoWplywuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_korespondencja_zalaczniki (wplyw_id, dokument_id)
VALUES (:wplyw_id, :dokument_id)"#,
                parameters: Vec::from([
                    ("wplyw_id", xmeta_room::ToSqlValue::to_sql_value(&params.wplyw_id)),
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                ]),
            },
        ],
    )
}
