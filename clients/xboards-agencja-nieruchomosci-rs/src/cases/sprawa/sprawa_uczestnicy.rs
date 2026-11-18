//! The kind `sprawa-uczestnicy` of the room `cases/sprawa`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "sprawa-uczestnicy";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Osoby i podmioty w teczce z ich danymi oraz ich role w teczce. Rola należy do udziału w teczce,
-- a nie do osoby: ta sama osoba bywa w jednej teczce klientką, a w innej pełnomocniczką. Listę
-- ról ustala typ sprawy tekstem; rdzeń sprawdza tylko format i reguły małżonka.
--
-- Uczestników wpisuje agent sprawy, a Kontroler ich czyta. Teczka trzyma tylko dane, których
-- potrzebuje sprawa (art. 5 ust. 1 lit. c RODO). Seria i numer dokumentu tożsamości zebrane jako
-- środek bezpieczeństwa finansowego leżą w rejestrze kontroli, nie tutaj.
--
-- Wersja uczestnika rośnie przy każdej zmianie danych identyfikujących, także adresu
-- i obywatelstwa. Sprawdzenie Kontrolera dotyczy jednej wersji; triggery, które zerują jego
-- aktualność, leżą w migracji sprawa-sprawdzenia (sprawa_sprawdzenia_nowa_wersja_uczestnika).
CREATE TABLE sprawa_uczestnicy_wpisy (
  id TEXT PRIMARY KEY,
  rodzaj_podmiotu TEXT NOT NULL CHECK (rodzaj_podmiotu IN ('osoba_fizyczna', 'osoba_prawna', 'jednostka_bez_osobowosci')),

  -- Osoba fizyczna. Imiona rodziców i PESEL podaje akt notarialny i wpis w księdze wieczystej.
  imiona TEXT,
  nazwisko TEXT,
  imie_ojca TEXT,
  imie_matki TEXT,
  pesel TEXT,
  -- Osoba, której PESEL nie nadano, np. część cudzoziemców (art. 25 ust. 4 ukwh).
  pesel_nie_nadano INTEGER NOT NULL DEFAULT 0 CHECK (pesel_nie_nadano IN (0, 1)),
  stan_cywilny TEXT CHECK (stan_cywilny IN ('stanu_wolnego', 'w_malzenstwie', 'w_separacji', 'rozwiedziony', 'owdowialy')),
  ustroj_majatkowy TEXT CHECK (ustroj_majatkowy IN ('wspolnosc_ustawowa', 'wspolnosc_umowna', 'rozdzielnosc_umowna', 'rozdzielnosc_sadowa', 'rozdzielnosc_z_mocy_prawa')),
  -- Art. 22¹ KC; od znacznika zależy prawo odstąpienia od umowy z biurem.
  konsument INTEGER NOT NULL DEFAULT 0 CHECK (konsument IN (0, 1)),

  -- Podmiot, a także osoba fizyczna prowadząca działalność (firma, NIP, REGON, wpis w CEIDG).
  nazwa TEXT,
  forma_prawna TEXT,
  krs TEXT,
  ceidg INTEGER NOT NULL DEFAULT 0 CHECK (ceidg IN (0, 1)),
  nip TEXT,
  regon TEXT,

  -- Kontakt jest daną roboczą biura, a nie wymogiem przepisu.
  telefon TEXT,
  email TEXT,
  kanal_kontaktu TEXT CHECK (kanal_kontaktu IN ('telefon', 'email', 'sms', 'portal', 'kanal_sprzedazy', 'list')),

  -- Klient biura ma umowę z biurem i własnego Kontrolera (R7); odnośnik do kartoteki klientów,
  -- tekstem, bo to inny pokój.
  klient_biura INTEGER NOT NULL DEFAULT 0 CHECK (klient_biura IN (0, 1)),
  kartoteka_odnosnik TEXT,
  wersja INTEGER NOT NULL DEFAULT 1 CHECK (wersja >= 1),

  -- Skąd biuro ma dane osoby: od niej samej (art. 13 RODO) albo z innego źródła (art. 14 RODO),
  -- i kiedy je pozyskało. Osoba poinformowana poza teczką ma tu opis, skąd to wiadomo
  -- (art. 14 ust. 5 lit. a RODO). Informację wysłaną z teczki wskazuje wysyłka z informacja_rodo.
  dane_od TEXT NOT NULL CHECK (dane_od IN ('od_osoby', 'z_innego_zrodla')),
  dane_pozyskano TEXT NOT NULL,
  poinformowana_poza_teczka TEXT,

  CONSTRAINT osoba_ma_imie_i_nazwisko CHECK (
    rodzaj_podmiotu <> 'osoba_fizyczna' OR (imiona IS NOT NULL AND nazwisko IS NOT NULL)
  ),
  CONSTRAINT podmiot_ma_nazwe CHECK (rodzaj_podmiotu = 'osoba_fizyczna' OR nazwa IS NOT NULL),
  CONSTRAINT podmiot_bez_danych_osoby CHECK (
    rodzaj_podmiotu = 'osoba_fizyczna'
    OR (
      imiona IS NULL AND nazwisko IS NULL AND imie_ojca IS NULL AND imie_matki IS NULL
      AND pesel IS NULL AND pesel_nie_nadano = 0 AND stan_cywilny IS NULL
      AND ustroj_majatkowy IS NULL
    )
  ),
  CONSTRAINT konsument_tylko_osoba_fizyczna CHECK (konsument = 0 OR rodzaj_podmiotu = 'osoba_fizyczna'),
  CONSTRAINT pesel_nie_nadano_wyklucza_pesel CHECK (pesel_nie_nadano = 0 OR pesel IS NULL),
  CONSTRAINT pesel_ma_11_cyfr CHECK (pesel IS NULL OR (length(pesel) = 11 AND pesel NOT GLOB '*[^0-9]*')),
  -- W separacji małżeństwo trwa, a ustrojem jest rozdzielność z mocy prawa (art. 54 KRO).
  CONSTRAINT ustroj_tylko_w_malzenstwie CHECK (
    ustroj_majatkowy IS NULL OR stan_cywilny IN ('w_malzenstwie', 'w_separacji')
  ),
  CONSTRAINT krs_albo_ceidg CHECK (krs IS NULL OR ceidg = 0)
) STRICT;

-- Miejsce zamieszkania, siedziba i adres do korespondencji, po jednym każdego rodzaju.
CREATE TABLE sprawa_uczestnicy_adresy (
  id INTEGER PRIMARY KEY,
  uczestnik_id TEXT NOT NULL REFERENCES sprawa_uczestnicy_wpisy (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('zamieszkanie', 'siedziba', 'do_korespondencji')),
  -- Kod państwa ISO 3166-1 alfa-2.
  kraj TEXT NOT NULL CHECK (length(kraj) = 2 AND kraj NOT GLOB '*[^A-Z]*'),
  miejscowosc TEXT NOT NULL,
  ulica TEXT,
  numer_budynku TEXT,
  numer_lokalu TEXT,
  kod_pocztowy TEXT,

  UNIQUE (uczestnik_id, rodzaj)
) STRICT;

-- Obywatelstwa osoby fizycznej, kilka na osobę; potrzebne, gdy nabywca może być cudzoziemcem.
CREATE TABLE sprawa_uczestnicy_obywatelstwa (
  id INTEGER PRIMARY KEY,
  uczestnik_id TEXT NOT NULL REFERENCES sprawa_uczestnicy_wpisy (id),
  -- Kod państwa ISO 3166-1 alfa-2.
  panstwo TEXT NOT NULL CHECK (length(panstwo) = 2 AND panstwo NOT GLOB '*[^A-Z]*'),

  UNIQUE (uczestnik_id, panstwo)
) STRICT;

-- Role uczestnika w teczce, np. klient, druga_strona, kupujacy, malzonek, gosc. Pełnomocnik
-- i reprezentant wynikają z umocowań (sprawa-umocowania), więc nie wpisuje się ich tutaj.
CREATE TABLE sprawa_uczestnicy_role (
  -- Klucz nadany przez bazę jest dodatni; zob. sprawa_uczestnicy_role_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  uczestnik_id TEXT NOT NULL REFERENCES sprawa_uczestnicy_wpisy (id),
  rola TEXT NOT NULL CHECK (rola <> '' AND rola NOT GLOB '*[^a-z_]*'),
  -- Uczestnik, którego rola dotyczy, np. osoba, której małżonkiem jest ten uczestnik.
  dotyczy_uczestnik_id TEXT REFERENCES sprawa_uczestnicy_wpisy (id),
  -- Rola, która przestała obowiązywać albo była pomyłką, dostaje datę końca i powód. Zostaje
  -- w teczce jako ślad sprawy (R9).
  zakonczono TEXT,
  powod_zakonczenia TEXT,

  CONSTRAINT zakonczenie_roli_z_powodem CHECK (
    (zakonczono IS NULL) = (powod_zakonczenia IS NULL)
    AND (powod_zakonczenia IS NULL OR powod_zakonczenia <> '')
  ),
  CONSTRAINT pelnomocnik_i_reprezentant_z_umocowan CHECK (rola NOT IN ('pelnomocnik', 'reprezentant')),
  CONSTRAINT malzonek_wskazuje_osobe CHECK (rola <> 'malzonek' OR dotyczy_uczestnik_id IS NOT NULL),
  CONSTRAINT rola_nie_dotyczy_siebie CHECK (dotyczy_uczestnik_id IS NOT uczestnik_id)
) STRICT;

-- Miejsce zamieszkania ma osoba fizyczna, siedzibę podmiot.
CREATE TRIGGER sprawa_uczestnicy_rodzaj_adresu_pasuje_do_podmiotu
BEFORE INSERT ON sprawa_uczestnicy_adresy
WHEN (
  SELECT
    (NEW.rodzaj = 'zamieszkanie' AND rodzaj_podmiotu <> 'osoba_fizyczna')
    OR (NEW.rodzaj = 'siedziba' AND rodzaj_podmiotu = 'osoba_fizyczna')
  FROM sprawa_uczestnicy_wpisy
  WHERE id = NEW.uczestnik_id
)
BEGIN
  SELECT RAISE(ABORT, 'miejsce zamieszkania ma tylko osoba fizyczna, a siedzibę tylko podmiot');
END;

CREATE TRIGGER sprawa_uczestnicy_rodzaj_zmienionego_adresu_pasuje_do_podmiotu
BEFORE UPDATE OF uczestnik_id, rodzaj ON sprawa_uczestnicy_adresy
WHEN (
  SELECT
    (NEW.rodzaj = 'zamieszkanie' AND rodzaj_podmiotu <> 'osoba_fizyczna')
    OR (NEW.rodzaj = 'siedziba' AND rodzaj_podmiotu = 'osoba_fizyczna')
  FROM sprawa_uczestnicy_wpisy
  WHERE id = NEW.uczestnik_id
)
BEGIN
  SELECT RAISE(ABORT, 'miejsce zamieszkania ma tylko osoba fizyczna, a siedzibę tylko podmiot');
END;

CREATE TRIGGER sprawa_uczestnicy_obywatelstwo_ma_osoba_fizyczna
BEFORE INSERT ON sprawa_uczestnicy_obywatelstwa
WHEN (
  SELECT rodzaj_podmiotu <> 'osoba_fizyczna'
  FROM sprawa_uczestnicy_wpisy
  WHERE id = NEW.uczestnik_id
)
BEGIN
  SELECT RAISE(ABORT, 'obywatelstwo ma tylko osoba fizyczna');
END;

CREATE TRIGGER sprawa_uczestnicy_zmienione_obywatelstwo_ma_osoba_fizyczna
BEFORE UPDATE OF uczestnik_id ON sprawa_uczestnicy_obywatelstwa
WHEN (
  SELECT rodzaj_podmiotu <> 'osoba_fizyczna'
  FROM sprawa_uczestnicy_wpisy
  WHERE id = NEW.uczestnik_id
)
BEGIN
  SELECT RAISE(ABORT, 'obywatelstwo ma tylko osoba fizyczna');
END;

-- Art. 37 § 1 pkt 1 KRO: zgody małżonka wymaga zbycie i nabycie z majątku wspólnego, więc
-- małżonek i osoba, której jest małżonkiem, to osoby fizyczne.
CREATE TRIGGER sprawa_uczestnicy_malzonek_osoby_fizycznej
BEFORE INSERT ON sprawa_uczestnicy_role
WHEN NEW.rola = 'malzonek'
  AND EXISTS (
    SELECT 1
    FROM sprawa_uczestnicy_wpisy
    WHERE id IN (NEW.uczestnik_id, NEW.dotyczy_uczestnik_id)
      AND rodzaj_podmiotu <> 'osoba_fizyczna'
  )
BEGIN
  SELECT RAISE(ABORT, 'małżonek i osoba, której jest małżonkiem, to osoby fizyczne');
END;

-- Rola powstaje jako obowiązująca, żeby jej koniec zawsze przeszedł przez regułę zmiany.
CREATE TRIGGER sprawa_uczestnicy_rola_powstaje_obowiazujaca
BEFORE INSERT ON sprawa_uczestnicy_role
WHEN NEW.zakonczono IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'rola powstaje jako obowiązująca');
END;

-- Roli się nie zmienia. Może się tylko raz zakończyć: rola, która przestała obowiązywać albo
-- była pomyłką, zostaje jako ślad sprawy (R9). Nowa rola to nowy wiersz.
CREATE TRIGGER sprawa_uczestnicy_role_bez_zmian
BEFORE UPDATE ON sprawa_uczestnicy_role
WHEN OLD.zakonczono IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.uczestnik_id IS NOT OLD.uczestnik_id
  OR NEW.rola IS NOT OLD.rola
  OR NEW.dotyczy_uczestnik_id IS NOT OLD.dotyczy_uczestnik_id
BEGIN
  SELECT RAISE(ABORT, 'roli w teczce się nie zmienia; rola może się tylko raz zakończyć');
END;

-- Reguły adresów, obywatelstw i małżonka sprawdzają rodzaj podmiotu przy zapisie tych wierszy;
-- zmiana rodzaju później by je ominęła.
CREATE TRIGGER sprawa_uczestnicy_rodzaj_podmiotu_niezmienny
BEFORE UPDATE OF rodzaj_podmiotu ON sprawa_uczestnicy_wpisy
WHEN NEW.rodzaj_podmiotu IS NOT OLD.rodzaj_podmiotu
BEGIN
  SELECT RAISE(ABORT, 'rodzaj podmiotu uczestnika się nie zmienia; osoba w miejsce podmiotu to nowy uczestnik');
END;

-- Dane identyfikujące to dane osoby albo podmiotu, adres zamieszkania albo siedziby
-- i obywatelstwo. Kontakt, adres do korespondencji i sytuacja rodzinna osoby nie identyfikują.
CREATE TRIGGER sprawa_uczestnicy_zmiana_danych_podnosi_wersje
AFTER UPDATE OF imiona, nazwisko, imie_ojca, imie_matki, pesel, pesel_nie_nadano, nazwa,
  forma_prawna, krs, ceidg, nip, regon
ON sprawa_uczestnicy_wpisy
WHEN NEW.imiona IS NOT OLD.imiona
  OR NEW.nazwisko IS NOT OLD.nazwisko
  OR NEW.imie_ojca IS NOT OLD.imie_ojca
  OR NEW.imie_matki IS NOT OLD.imie_matki
  OR NEW.pesel IS NOT OLD.pesel
  OR NEW.pesel_nie_nadano IS NOT OLD.pesel_nie_nadano
  OR NEW.nazwa IS NOT OLD.nazwa
  OR NEW.forma_prawna IS NOT OLD.forma_prawna
  OR NEW.krs IS NOT OLD.krs
  OR NEW.ceidg IS NOT OLD.ceidg
  OR NEW.nip IS NOT OLD.nip
  OR NEW.regon IS NOT OLD.regon
BEGIN
  UPDATE sprawa_uczestnicy_wpisy
  SET wersja = wersja + 1
  WHERE id = NEW.id;
END;

CREATE TRIGGER sprawa_uczestnicy_nowy_adres_podnosi_wersje
AFTER INSERT ON sprawa_uczestnicy_adresy
WHEN NEW.rodzaj IN ('zamieszkanie', 'siedziba')
BEGIN
  UPDATE sprawa_uczestnicy_wpisy
  SET wersja = wersja + 1
  WHERE id = NEW.uczestnik_id;
END;

CREATE TRIGGER sprawa_uczestnicy_zmiana_adresu_podnosi_wersje
AFTER UPDATE ON sprawa_uczestnicy_adresy
WHEN NEW.rodzaj IN ('zamieszkanie', 'siedziba')
  OR OLD.rodzaj IN ('zamieszkanie', 'siedziba')
BEGIN
  UPDATE sprawa_uczestnicy_wpisy
  SET wersja = wersja + 1
  WHERE id IN (OLD.uczestnik_id, NEW.uczestnik_id);
END;

-- Adres i obywatelstwo wolno usunąć jako poprawkę pomyłki (D14): dziennik pokoju XSql zachowuje
-- poprzedni stan z autorem.
CREATE TRIGGER sprawa_uczestnicy_usuniecie_adresu_podnosi_wersje
AFTER DELETE ON sprawa_uczestnicy_adresy
WHEN OLD.rodzaj IN ('zamieszkanie', 'siedziba')
BEGIN
  UPDATE sprawa_uczestnicy_wpisy
  SET wersja = wersja + 1
  WHERE id = OLD.uczestnik_id;
END;

CREATE TRIGGER sprawa_uczestnicy_nowe_obywatelstwo_podnosi_wersje
AFTER INSERT ON sprawa_uczestnicy_obywatelstwa
BEGIN
  UPDATE sprawa_uczestnicy_wpisy
  SET wersja = wersja + 1
  WHERE id = NEW.uczestnik_id;
END;

CREATE TRIGGER sprawa_uczestnicy_zmiana_obywatelstwa_podnosi_wersje
AFTER UPDATE ON sprawa_uczestnicy_obywatelstwa
BEGIN
  UPDATE sprawa_uczestnicy_wpisy
  SET wersja = wersja + 1
  WHERE id IN (OLD.uczestnik_id, NEW.uczestnik_id);
END;

CREATE TRIGGER sprawa_uczestnicy_usuniecie_obywatelstwa_podnosi_wersje
AFTER DELETE ON sprawa_uczestnicy_obywatelstwa
BEGIN
  UPDATE sprawa_uczestnicy_wpisy
  SET wersja = wersja + 1
  WHERE id = OLD.uczestnik_id;
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_uczestnicy_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_uczestnicy_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_uczestnicy_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_uczestnicy_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_uczestnicy_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER sprawa_uczestnicy_role_bez_zastepowania
BEFORE INSERT ON sprawa_uczestnicy_role
WHEN EXISTS (
  SELECT 1
  FROM sprawa_uczestnicy_role
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_uczestnicy_role_bez_usuwania
BEFORE DELETE ON sprawa_uczestnicy_role
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
];

/// The values of column `adresy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdresyRodzaj {
    Zamieszkanie,
    Siedziba,
    DoKorespondencji,
}

impl AdresyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[AdresyRodzaj] = &[
        AdresyRodzaj::Zamieszkanie,
        AdresyRodzaj::Siedziba,
        AdresyRodzaj::DoKorespondencji,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            AdresyRodzaj::Zamieszkanie => "zamieszkanie",
            AdresyRodzaj::Siedziba => "siedziba",
            AdresyRodzaj::DoKorespondencji => "do_korespondencji",
        }
    }
}

impl xmeta_room::FromSqlValue for AdresyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "zamieszkanie" => Ok(AdresyRodzaj::Zamieszkanie),
                    "siedziba" => Ok(AdresyRodzaj::Siedziba),
                    "do_korespondencji" => Ok(AdresyRodzaj::DoKorespondencji),
                    other => Err(format!("{other} is no value of AdresyRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for AdresyRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.rodzaj_podmiotu`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzajPodmiotu {
    OsobaFizyczna,
    OsobaPrawna,
    JednostkaBezOsobowosci,
}

impl WpisyRodzajPodmiotu {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzajPodmiotu] = &[
        WpisyRodzajPodmiotu::OsobaFizyczna,
        WpisyRodzajPodmiotu::OsobaPrawna,
        WpisyRodzajPodmiotu::JednostkaBezOsobowosci,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzajPodmiotu::OsobaFizyczna => "osoba_fizyczna",
            WpisyRodzajPodmiotu::OsobaPrawna => "osoba_prawna",
            WpisyRodzajPodmiotu::JednostkaBezOsobowosci => "jednostka_bez_osobowosci",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzajPodmiotu {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "osoba_fizyczna" => Ok(WpisyRodzajPodmiotu::OsobaFizyczna),
                    "osoba_prawna" => Ok(WpisyRodzajPodmiotu::OsobaPrawna),
                    "jednostka_bez_osobowosci" => {
                        Ok(WpisyRodzajPodmiotu::JednostkaBezOsobowosci)
                    }
                    other => Err(format!("{other} is no value of WpisyRodzajPodmiotu")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyRodzajPodmiotu {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.stan_cywilny`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyStanCywilny {
    StanuWolnego,
    WMalzenstwie,
    WSeparacji,
    Rozwiedziony,
    Owdowialy,
}

impl WpisyStanCywilny {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyStanCywilny] = &[
        WpisyStanCywilny::StanuWolnego,
        WpisyStanCywilny::WMalzenstwie,
        WpisyStanCywilny::WSeparacji,
        WpisyStanCywilny::Rozwiedziony,
        WpisyStanCywilny::Owdowialy,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyStanCywilny::StanuWolnego => "stanu_wolnego",
            WpisyStanCywilny::WMalzenstwie => "w_malzenstwie",
            WpisyStanCywilny::WSeparacji => "w_separacji",
            WpisyStanCywilny::Rozwiedziony => "rozwiedziony",
            WpisyStanCywilny::Owdowialy => "owdowialy",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyStanCywilny {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "stanu_wolnego" => Ok(WpisyStanCywilny::StanuWolnego),
                    "w_malzenstwie" => Ok(WpisyStanCywilny::WMalzenstwie),
                    "w_separacji" => Ok(WpisyStanCywilny::WSeparacji),
                    "rozwiedziony" => Ok(WpisyStanCywilny::Rozwiedziony),
                    "owdowialy" => Ok(WpisyStanCywilny::Owdowialy),
                    other => Err(format!("{other} is no value of WpisyStanCywilny")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyStanCywilny {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.ustroj_majatkowy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyUstrojMajatkowy {
    WspolnoscUstawowa,
    WspolnoscUmowna,
    RozdzielnoscUmowna,
    RozdzielnoscSadowa,
    RozdzielnoscZMocyPrawa,
}

impl WpisyUstrojMajatkowy {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyUstrojMajatkowy] = &[
        WpisyUstrojMajatkowy::WspolnoscUstawowa,
        WpisyUstrojMajatkowy::WspolnoscUmowna,
        WpisyUstrojMajatkowy::RozdzielnoscUmowna,
        WpisyUstrojMajatkowy::RozdzielnoscSadowa,
        WpisyUstrojMajatkowy::RozdzielnoscZMocyPrawa,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyUstrojMajatkowy::WspolnoscUstawowa => "wspolnosc_ustawowa",
            WpisyUstrojMajatkowy::WspolnoscUmowna => "wspolnosc_umowna",
            WpisyUstrojMajatkowy::RozdzielnoscUmowna => "rozdzielnosc_umowna",
            WpisyUstrojMajatkowy::RozdzielnoscSadowa => "rozdzielnosc_sadowa",
            WpisyUstrojMajatkowy::RozdzielnoscZMocyPrawa => "rozdzielnosc_z_mocy_prawa",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyUstrojMajatkowy {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "wspolnosc_ustawowa" => Ok(WpisyUstrojMajatkowy::WspolnoscUstawowa),
                    "wspolnosc_umowna" => Ok(WpisyUstrojMajatkowy::WspolnoscUmowna),
                    "rozdzielnosc_umowna" => Ok(WpisyUstrojMajatkowy::RozdzielnoscUmowna),
                    "rozdzielnosc_sadowa" => Ok(WpisyUstrojMajatkowy::RozdzielnoscSadowa),
                    "rozdzielnosc_z_mocy_prawa" => {
                        Ok(WpisyUstrojMajatkowy::RozdzielnoscZMocyPrawa)
                    }
                    other => Err(format!("{other} is no value of WpisyUstrojMajatkowy")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyUstrojMajatkowy {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.kanal_kontaktu`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyKanalKontaktu {
    Telefon,
    Email,
    Sms,
    Portal,
    KanalSprzedazy,
    List,
}

impl WpisyKanalKontaktu {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyKanalKontaktu] = &[
        WpisyKanalKontaktu::Telefon,
        WpisyKanalKontaktu::Email,
        WpisyKanalKontaktu::Sms,
        WpisyKanalKontaktu::Portal,
        WpisyKanalKontaktu::KanalSprzedazy,
        WpisyKanalKontaktu::List,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyKanalKontaktu::Telefon => "telefon",
            WpisyKanalKontaktu::Email => "email",
            WpisyKanalKontaktu::Sms => "sms",
            WpisyKanalKontaktu::Portal => "portal",
            WpisyKanalKontaktu::KanalSprzedazy => "kanal_sprzedazy",
            WpisyKanalKontaktu::List => "list",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyKanalKontaktu {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "telefon" => Ok(WpisyKanalKontaktu::Telefon),
                    "email" => Ok(WpisyKanalKontaktu::Email),
                    "sms" => Ok(WpisyKanalKontaktu::Sms),
                    "portal" => Ok(WpisyKanalKontaktu::Portal),
                    "kanal_sprzedazy" => Ok(WpisyKanalKontaktu::KanalSprzedazy),
                    "list" => Ok(WpisyKanalKontaktu::List),
                    other => Err(format!("{other} is no value of WpisyKanalKontaktu")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyKanalKontaktu {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.dane_od`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyDaneOd {
    OdOsoby,
    ZInnegoZrodla,
}

impl WpisyDaneOd {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyDaneOd] = &[WpisyDaneOd::OdOsoby, WpisyDaneOd::ZInnegoZrodla];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyDaneOd::OdOsoby => "od_osoby",
            WpisyDaneOd::ZInnegoZrodla => "z_innego_zrodla",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyDaneOd {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "od_osoby" => Ok(WpisyDaneOd::OdOsoby),
                    "z_innego_zrodla" => Ok(WpisyDaneOd::ZInnegoZrodla),
                    other => Err(format!("{other} is no value of WpisyDaneOd")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyDaneOd {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `adresy_uczestnikow`.
#[derive(Debug, Clone, PartialEq)]
pub struct AdresyUczestnikowRow {
    pub id: i64,
    pub uczestnik_id: String,
    pub rodzaj: AdresyRodzaj,
    pub kraj: String,
    pub miejscowosc: String,
    pub ulica: Option<String>,
    pub numer_budynku: Option<String>,
    pub numer_lokalu: Option<String>,
    pub kod_pocztowy: Option<String>,
}

impl AdresyUczestnikowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(AdresyUczestnikowRow {
            id: row.get("id")?,
            uczestnik_id: row.get("uczestnik_id")?,
            rodzaj: row.get("rodzaj")?,
            kraj: row.get("kraj")?,
            miejscowosc: row.get("miejscowosc")?,
            ulica: row.get("ulica")?,
            numer_budynku: row.get("numer_budynku")?,
            numer_lokalu: row.get("numer_lokalu")?,
            kod_pocztowy: row.get("kod_pocztowy")?,
        })
    }
}

/// Adresy wszystkich uczestników teczki.
pub fn adresy_uczestnikow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<AdresyUczestnikowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_uczestnicy_adresy
ORDER BY uczestnik_id, rodzaj"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(AdresyUczestnikowRow::from_row).collect()
}

/// The parameters of `wpisz_adres_uczestnika`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszAdresUczestnikaParams {
    pub uczestnik_id: String,
    pub rodzaj: AdresyRodzaj,
    pub kraj: String,
    pub miejscowosc: String,
    pub ulica: Option<String>,
    pub numer_budynku: Option<String>,
    pub numer_lokalu: Option<String>,
    pub kod_pocztowy: Option<String>,
}

/// Adres uczestnika; każdy rodzaj adresu raz na uczestnika.
pub fn wpisz_adres_uczestnika(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszAdresUczestnikaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_uczestnicy_adresy (uczestnik_id, rodzaj, kraj, miejscowosc, ulica, numer_budynku,
  numer_lokalu, kod_pocztowy)
VALUES (:uczestnik_id, :rodzaj, :kraj, :miejscowosc, :ulica, :numer_budynku, :numer_lokalu,
  :kod_pocztowy)"#,
                parameters: Vec::from([
                    (
                        "uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.uczestnik_id),
                    ),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("kraj", xmeta_room::ToSqlValue::to_sql_value(&params.kraj)),
                    (
                        "miejscowosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.miejscowosc),
                    ),
                    ("ulica", xmeta_room::ToSqlValue::to_sql_value(&params.ulica)),
                    (
                        "numer_budynku",
                        xmeta_room::ToSqlValue::to_sql_value(&params.numer_budynku),
                    ),
                    (
                        "numer_lokalu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.numer_lokalu),
                    ),
                    (
                        "kod_pocztowy",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kod_pocztowy),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_adres_uczestnika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZmienAdresUczestnikaParams {
    pub kraj: String,
    pub miejscowosc: String,
    pub ulica: Option<String>,
    pub numer_budynku: Option<String>,
    pub numer_lokalu: Option<String>,
    pub kod_pocztowy: Option<String>,
    pub id: i64,
}

/// Nowy adres tego samego rodzaju, np. po przeprowadzce.
pub fn zmien_adres_uczestnika(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienAdresUczestnikaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_uczestnicy_adresy
SET kraj = :kraj,
  miejscowosc = :miejscowosc,
  ulica = :ulica,
  numer_budynku = :numer_budynku,
  numer_lokalu = :numer_lokalu,
  kod_pocztowy = :kod_pocztowy
WHERE id = :id"#,
                parameters: Vec::from([
                    ("kraj", xmeta_room::ToSqlValue::to_sql_value(&params.kraj)),
                    (
                        "miejscowosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.miejscowosc),
                    ),
                    ("ulica", xmeta_room::ToSqlValue::to_sql_value(&params.ulica)),
                    (
                        "numer_budynku",
                        xmeta_room::ToSqlValue::to_sql_value(&params.numer_budynku),
                    ),
                    (
                        "numer_lokalu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.numer_lokalu),
                    ),
                    (
                        "kod_pocztowy",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kod_pocztowy),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wykresl_adres_uczestnika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WykreslAdresUczestnikaParams {
    pub id: i64,
}

/// Poprawka pomyłki: adres wpisany niepotrzebnie albo nie tej osobie (D14).
pub fn wykresl_adres_uczestnika(
    room: &impl xmeta_room::MetaRoom,
    params: &WykreslAdresUczestnikaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"DELETE FROM sprawa_uczestnicy_adresy
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `obywatelstwa_uczestnikow`.
#[derive(Debug, Clone, PartialEq)]
pub struct ObywatelstwaUczestnikowRow {
    pub id: i64,
    pub uczestnik_id: String,
    pub panstwo: String,
}

impl ObywatelstwaUczestnikowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ObywatelstwaUczestnikowRow {
            id: row.get("id")?,
            uczestnik_id: row.get("uczestnik_id")?,
            panstwo: row.get("panstwo")?,
        })
    }
}

/// Obywatelstwa wszystkich uczestników teczki.
pub fn obywatelstwa_uczestnikow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ObywatelstwaUczestnikowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_uczestnicy_obywatelstwa
ORDER BY uczestnik_id, panstwo"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(ObywatelstwaUczestnikowRow::from_row).collect()
}

/// The parameters of `wpisz_obywatelstwo_uczestnika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszObywatelstwoUczestnikaParams {
    pub uczestnik_id: String,
    pub panstwo: String,
}

/// Obywatelstwo osoby fizycznej, gdy nabywca może być cudzoziemcem.
pub fn wpisz_obywatelstwo_uczestnika(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszObywatelstwoUczestnikaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_uczestnicy_obywatelstwa (uczestnik_id, panstwo)
VALUES (:uczestnik_id, :panstwo)"#,
                parameters: Vec::from([
                    (
                        "uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.uczestnik_id),
                    ),
                    ("panstwo", xmeta_room::ToSqlValue::to_sql_value(&params.panstwo)),
                ]),
            },
        ],
    )
}

/// The parameters of `wykresl_obywatelstwo_uczestnika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WykreslObywatelstwoUczestnikaParams {
    pub id: i64,
}

/// Poprawka pomyłki w obywatelstwie (D14).
pub fn wykresl_obywatelstwo_uczestnika(
    room: &impl xmeta_room::MetaRoom,
    params: &WykreslObywatelstwoUczestnikaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"DELETE FROM sprawa_uczestnicy_obywatelstwa
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `role_uczestnikow`.
#[derive(Debug, Clone, PartialEq)]
pub struct RoleUczestnikowRow {
    pub id: i64,
    pub uczestnik_id: String,
    pub rola: String,
    pub dotyczy_uczestnik_id: Option<String>,
    pub zakonczono: Option<String>,
    pub powod_zakonczenia: Option<String>,
}

impl RoleUczestnikowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(RoleUczestnikowRow {
            id: row.get("id")?,
            uczestnik_id: row.get("uczestnik_id")?,
            rola: row.get("rola")?,
            dotyczy_uczestnik_id: row.get("dotyczy_uczestnik_id")?,
            zakonczono: row.get("zakonczono")?,
            powod_zakonczenia: row.get("powod_zakonczenia")?,
        })
    }
}

/// Role uczestników w teczce, w kolejności nadania.
pub fn role_uczestnikow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<RoleUczestnikowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_uczestnicy_role
ORDER BY id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(RoleUczestnikowRow::from_row).collect()
}

/// The parameters of `nadaj_role_uczestnikowi`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NadajRoleUczestnikowiParams {
    pub uczestnik_id: String,
    pub rola: String,
    pub dotyczy_uczestnik_id: Option<String>,
}

/// Rola uczestnika w teczce, z listy ról typu sprawy; małżonek wskazuje osobę, której jest
/// małżonkiem.
pub fn nadaj_role_uczestnikowi(
    room: &impl xmeta_room::MetaRoom,
    params: &NadajRoleUczestnikowiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_uczestnicy_role (uczestnik_id, rola, dotyczy_uczestnik_id)
VALUES (:uczestnik_id, :rola, :dotyczy_uczestnik_id)"#,
                parameters: Vec::from([
                    (
                        "uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.uczestnik_id),
                    ),
                    ("rola", xmeta_room::ToSqlValue::to_sql_value(&params.rola)),
                    (
                        "dotyczy_uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.dotyczy_uczestnik_id,
                        ),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zakoncz_role_uczestnika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZakonczRoleUczestnikaParams {
    pub zakonczono: Option<String>,
    pub powod_zakonczenia: Option<String>,
    pub id: i64,
}

/// Koniec roli, która przestała obowiązywać albo była pomyłką; rola zostaje w teczce z datą
/// i powodem (R9).
pub fn zakoncz_role_uczestnika(
    room: &impl xmeta_room::MetaRoom,
    params: &ZakonczRoleUczestnikaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_uczestnicy_role
SET zakonczono = :zakonczono, powod_zakonczenia = :powod_zakonczenia
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zakonczono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zakonczono),
                    ),
                    (
                        "powod_zakonczenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.powod_zakonczenia),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `uczestnicy`.
#[derive(Debug, Clone, PartialEq)]
pub struct UczestnicyRow {
    pub id: String,
    pub rodzaj_podmiotu: WpisyRodzajPodmiotu,
    pub imiona: Option<String>,
    pub nazwisko: Option<String>,
    pub imie_ojca: Option<String>,
    pub imie_matki: Option<String>,
    pub pesel: Option<String>,
    pub pesel_nie_nadano: bool,
    pub stan_cywilny: Option<WpisyStanCywilny>,
    pub ustroj_majatkowy: Option<WpisyUstrojMajatkowy>,
    pub konsument: bool,
    pub nazwa: Option<String>,
    pub forma_prawna: Option<String>,
    pub krs: Option<String>,
    pub ceidg: bool,
    pub nip: Option<String>,
    pub regon: Option<String>,
    pub telefon: Option<String>,
    pub email: Option<String>,
    pub kanal_kontaktu: Option<WpisyKanalKontaktu>,
    pub klient_biura: bool,
    pub kartoteka_odnosnik: Option<String>,
    pub wersja: i64,
    pub dane_od: WpisyDaneOd,
    pub dane_pozyskano: String,
    pub poinformowana_poza_teczka: Option<String>,
}

impl UczestnicyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(UczestnicyRow {
            id: row.get("id")?,
            rodzaj_podmiotu: row.get("rodzaj_podmiotu")?,
            imiona: row.get("imiona")?,
            nazwisko: row.get("nazwisko")?,
            imie_ojca: row.get("imie_ojca")?,
            imie_matki: row.get("imie_matki")?,
            pesel: row.get("pesel")?,
            pesel_nie_nadano: row.get("pesel_nie_nadano")?,
            stan_cywilny: row.get("stan_cywilny")?,
            ustroj_majatkowy: row.get("ustroj_majatkowy")?,
            konsument: row.get("konsument")?,
            nazwa: row.get("nazwa")?,
            forma_prawna: row.get("forma_prawna")?,
            krs: row.get("krs")?,
            ceidg: row.get("ceidg")?,
            nip: row.get("nip")?,
            regon: row.get("regon")?,
            telefon: row.get("telefon")?,
            email: row.get("email")?,
            kanal_kontaktu: row.get("kanal_kontaktu")?,
            klient_biura: row.get("klient_biura")?,
            kartoteka_odnosnik: row.get("kartoteka_odnosnik")?,
            wersja: row.get("wersja")?,
            dane_od: row.get("dane_od")?,
            dane_pozyskano: row.get("dane_pozyskano")?,
            poinformowana_poza_teczka: row.get("poinformowana_poza_teczka")?,
        })
    }
}

/// Osoby i podmioty w teczce, w kolejności wpisania.
pub fn uczestnicy(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<UczestnicyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_uczestnicy_wpisy
ORDER BY rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(UczestnicyRow::from_row).collect()
}

/// The parameters of `uczestnik`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UczestnikParams {
    pub id: String,
}

/// A row of `uczestnik`.
#[derive(Debug, Clone, PartialEq)]
pub struct UczestnikRow {
    pub id: String,
    pub rodzaj_podmiotu: WpisyRodzajPodmiotu,
    pub imiona: Option<String>,
    pub nazwisko: Option<String>,
    pub imie_ojca: Option<String>,
    pub imie_matki: Option<String>,
    pub pesel: Option<String>,
    pub pesel_nie_nadano: bool,
    pub stan_cywilny: Option<WpisyStanCywilny>,
    pub ustroj_majatkowy: Option<WpisyUstrojMajatkowy>,
    pub konsument: bool,
    pub nazwa: Option<String>,
    pub forma_prawna: Option<String>,
    pub krs: Option<String>,
    pub ceidg: bool,
    pub nip: Option<String>,
    pub regon: Option<String>,
    pub telefon: Option<String>,
    pub email: Option<String>,
    pub kanal_kontaktu: Option<WpisyKanalKontaktu>,
    pub klient_biura: bool,
    pub kartoteka_odnosnik: Option<String>,
    pub wersja: i64,
    pub dane_od: WpisyDaneOd,
    pub dane_pozyskano: String,
    pub poinformowana_poza_teczka: Option<String>,
}

impl UczestnikRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(UczestnikRow {
            id: row.get("id")?,
            rodzaj_podmiotu: row.get("rodzaj_podmiotu")?,
            imiona: row.get("imiona")?,
            nazwisko: row.get("nazwisko")?,
            imie_ojca: row.get("imie_ojca")?,
            imie_matki: row.get("imie_matki")?,
            pesel: row.get("pesel")?,
            pesel_nie_nadano: row.get("pesel_nie_nadano")?,
            stan_cywilny: row.get("stan_cywilny")?,
            ustroj_majatkowy: row.get("ustroj_majatkowy")?,
            konsument: row.get("konsument")?,
            nazwa: row.get("nazwa")?,
            forma_prawna: row.get("forma_prawna")?,
            krs: row.get("krs")?,
            ceidg: row.get("ceidg")?,
            nip: row.get("nip")?,
            regon: row.get("regon")?,
            telefon: row.get("telefon")?,
            email: row.get("email")?,
            kanal_kontaktu: row.get("kanal_kontaktu")?,
            klient_biura: row.get("klient_biura")?,
            kartoteka_odnosnik: row.get("kartoteka_odnosnik")?,
            wersja: row.get("wersja")?,
            dane_od: row.get("dane_od")?,
            dane_pozyskano: row.get("dane_pozyskano")?,
            poinformowana_poza_teczka: row.get("poinformowana_poza_teczka")?,
        })
    }
}

/// Jedna osoba albo jeden podmiot z teczki.
pub fn uczestnik(
    room: &impl xmeta_room::MetaRoom,
    params: &UczestnikParams,
) -> Result<Option<UczestnikRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_uczestnicy_wpisy
WHERE id = :id"#,
        parameters: Vec::from([("id", xmeta_room::ToSqlValue::to_sql_value(&params.id))]),
    };
    let rows = room.read(&statement)?;
    rows.first().map(UczestnikRow::from_row).transpose()
}

/// The parameters of `wpisz_uczestnika`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszUczestnikaParams {
    pub id: String,
    pub rodzaj_podmiotu: WpisyRodzajPodmiotu,
    pub imiona: Option<String>,
    pub nazwisko: Option<String>,
    pub imie_ojca: Option<String>,
    pub imie_matki: Option<String>,
    pub pesel: Option<String>,
    pub pesel_nie_nadano: bool,
    pub konsument: bool,
    pub nazwa: Option<String>,
    pub forma_prawna: Option<String>,
    pub krs: Option<String>,
    pub ceidg: bool,
    pub nip: Option<String>,
    pub regon: Option<String>,
    pub telefon: Option<String>,
    pub email: Option<String>,
    pub kanal_kontaktu: Option<WpisyKanalKontaktu>,
    pub klient_biura: bool,
    pub kartoteka_odnosnik: Option<String>,
    pub dane_od: WpisyDaneOd,
    pub dane_pozyskano: String,
}

/// Nowa osoba albo podmiot w teczce, z tym, skąd biuro ma jego dane. Identyfikator (UUID) nadaje
/// aplikacja, żeby od razu powiązać z nim adres, rolę albo dokument.
pub fn wpisz_uczestnika(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszUczestnikaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_uczestnicy_wpisy (id, rodzaj_podmiotu, imiona, nazwisko, imie_ojca, imie_matki,
  pesel, pesel_nie_nadano, konsument, nazwa, forma_prawna, krs, ceidg, nip, regon, telefon, email,
  kanal_kontaktu, klient_biura, kartoteka_odnosnik, dane_od, dane_pozyskano)
VALUES (:id, :rodzaj_podmiotu, :imiona, :nazwisko, :imie_ojca, :imie_matki, :pesel,
  :pesel_nie_nadano, :konsument, :nazwa, :forma_prawna, :krs, :ceidg, :nip, :regon, :telefon, :email,
  :kanal_kontaktu, :klient_biura, :kartoteka_odnosnik, :dane_od, :dane_pozyskano)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "rodzaj_podmiotu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj_podmiotu),
                    ),
                    ("imiona", xmeta_room::ToSqlValue::to_sql_value(&params.imiona)),
                    ("nazwisko", xmeta_room::ToSqlValue::to_sql_value(&params.nazwisko)),
                    (
                        "imie_ojca",
                        xmeta_room::ToSqlValue::to_sql_value(&params.imie_ojca),
                    ),
                    (
                        "imie_matki",
                        xmeta_room::ToSqlValue::to_sql_value(&params.imie_matki),
                    ),
                    ("pesel", xmeta_room::ToSqlValue::to_sql_value(&params.pesel)),
                    (
                        "pesel_nie_nadano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.pesel_nie_nadano),
                    ),
                    (
                        "konsument",
                        xmeta_room::ToSqlValue::to_sql_value(&params.konsument),
                    ),
                    ("nazwa", xmeta_room::ToSqlValue::to_sql_value(&params.nazwa)),
                    (
                        "forma_prawna",
                        xmeta_room::ToSqlValue::to_sql_value(&params.forma_prawna),
                    ),
                    ("krs", xmeta_room::ToSqlValue::to_sql_value(&params.krs)),
                    ("ceidg", xmeta_room::ToSqlValue::to_sql_value(&params.ceidg)),
                    ("nip", xmeta_room::ToSqlValue::to_sql_value(&params.nip)),
                    ("regon", xmeta_room::ToSqlValue::to_sql_value(&params.regon)),
                    ("telefon", xmeta_room::ToSqlValue::to_sql_value(&params.telefon)),
                    ("email", xmeta_room::ToSqlValue::to_sql_value(&params.email)),
                    (
                        "kanal_kontaktu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kanal_kontaktu),
                    ),
                    (
                        "klient_biura",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_biura),
                    ),
                    (
                        "kartoteka_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kartoteka_odnosnik),
                    ),
                    ("dane_od", xmeta_room::ToSqlValue::to_sql_value(&params.dane_od)),
                    (
                        "dane_pozyskano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dane_pozyskano),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_dane_uczestnika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZmienDaneUczestnikaParams {
    pub imiona: Option<String>,
    pub nazwisko: Option<String>,
    pub imie_ojca: Option<String>,
    pub imie_matki: Option<String>,
    pub pesel: Option<String>,
    pub pesel_nie_nadano: bool,
    pub konsument: bool,
    pub nazwa: Option<String>,
    pub forma_prawna: Option<String>,
    pub krs: Option<String>,
    pub ceidg: bool,
    pub nip: Option<String>,
    pub regon: Option<String>,
    pub id: String,
}

/// Poprawka danych osoby albo podmiotu. Zmiana danych identyfikujących podnosi wersję uczestnika,
/// a z nią traci aktualność sprawdzenie Kontrolera (triggery w migracjach). Rodzaj podmiotu się
/// nie zmienia; osoba w miejsce podmiotu to nowy uczestnik.
pub fn zmien_dane_uczestnika(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienDaneUczestnikaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_uczestnicy_wpisy
SET imiona = :imiona,
  nazwisko = :nazwisko,
  imie_ojca = :imie_ojca,
  imie_matki = :imie_matki,
  pesel = :pesel,
  pesel_nie_nadano = :pesel_nie_nadano,
  konsument = :konsument,
  nazwa = :nazwa,
  forma_prawna = :forma_prawna,
  krs = :krs,
  ceidg = :ceidg,
  nip = :nip,
  regon = :regon
WHERE id = :id"#,
                parameters: Vec::from([
                    ("imiona", xmeta_room::ToSqlValue::to_sql_value(&params.imiona)),
                    ("nazwisko", xmeta_room::ToSqlValue::to_sql_value(&params.nazwisko)),
                    (
                        "imie_ojca",
                        xmeta_room::ToSqlValue::to_sql_value(&params.imie_ojca),
                    ),
                    (
                        "imie_matki",
                        xmeta_room::ToSqlValue::to_sql_value(&params.imie_matki),
                    ),
                    ("pesel", xmeta_room::ToSqlValue::to_sql_value(&params.pesel)),
                    (
                        "pesel_nie_nadano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.pesel_nie_nadano),
                    ),
                    (
                        "konsument",
                        xmeta_room::ToSqlValue::to_sql_value(&params.konsument),
                    ),
                    ("nazwa", xmeta_room::ToSqlValue::to_sql_value(&params.nazwa)),
                    (
                        "forma_prawna",
                        xmeta_room::ToSqlValue::to_sql_value(&params.forma_prawna),
                    ),
                    ("krs", xmeta_room::ToSqlValue::to_sql_value(&params.krs)),
                    ("ceidg", xmeta_room::ToSqlValue::to_sql_value(&params.ceidg)),
                    ("nip", xmeta_room::ToSqlValue::to_sql_value(&params.nip)),
                    ("regon", xmeta_room::ToSqlValue::to_sql_value(&params.regon)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_kontakt_uczestnika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZmienKontaktUczestnikaParams {
    pub telefon: Option<String>,
    pub email: Option<String>,
    pub kanal_kontaktu: Option<WpisyKanalKontaktu>,
    pub id: String,
}

/// Telefon, adres poczty i kanał, którym osoba chce rozmawiać z biurem.
pub fn zmien_kontakt_uczestnika(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienKontaktUczestnikaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_uczestnicy_wpisy
SET telefon = :telefon,
  email = :email,
  kanal_kontaktu = :kanal_kontaktu
WHERE id = :id"#,
                parameters: Vec::from([
                    ("telefon", xmeta_room::ToSqlValue::to_sql_value(&params.telefon)),
                    ("email", xmeta_room::ToSqlValue::to_sql_value(&params.email)),
                    (
                        "kanal_kontaktu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kanal_kontaktu),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `ustal_sytuacje_rodzinna_uczestnika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UstalSytuacjeRodzinnaUczestnikaParams {
    pub stan_cywilny: Option<WpisyStanCywilny>,
    pub ustroj_majatkowy: Option<WpisyUstrojMajatkowy>,
    pub id: String,
}

/// Stan cywilny i ustrój majątkowy z oświadczenia strony; dokument, na którym stoją, wskazuje
/// źródło w sprawa_dokumenty_zrodla.
pub fn ustal_sytuacje_rodzinna_uczestnika(
    room: &impl xmeta_room::MetaRoom,
    params: &UstalSytuacjeRodzinnaUczestnikaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_uczestnicy_wpisy
SET stan_cywilny = :stan_cywilny,
  ustroj_majatkowy = :ustroj_majatkowy
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "stan_cywilny",
                        xmeta_room::ToSqlValue::to_sql_value(&params.stan_cywilny),
                    ),
                    (
                        "ustroj_majatkowy",
                        xmeta_room::ToSqlValue::to_sql_value(&params.ustroj_majatkowy),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `polacz_uczestnika_z_kartoteka`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PolaczUczestnikaZKartotekaParams {
    pub klient_biura: bool,
    pub kartoteka_odnosnik: Option<String>,
    pub id: String,
}

/// Odnośnik do wpisu klienta w kartotece klientów, który podaje kierownik działu.
pub fn polacz_uczestnika_z_kartoteka(
    room: &impl xmeta_room::MetaRoom,
    params: &PolaczUczestnikaZKartotekaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_uczestnicy_wpisy
SET klient_biura = :klient_biura,
  kartoteka_odnosnik = :kartoteka_odnosnik
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "klient_biura",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_biura),
                    ),
                    (
                        "kartoteka_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kartoteka_odnosnik),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_poinformowanie_poza_teczka`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPoinformowaniePozaTeczkaParams {
    pub poinformowana_poza_teczka: String,
    pub id: String,
}

/// Osoba ma już informację o przetwarzaniu danych; opis mówi, skąd to wiadomo.
pub fn odnotuj_poinformowanie_poza_teczka(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPoinformowaniePozaTeczkaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_uczestnicy_wpisy
SET poinformowana_poza_teczka = :poinformowana_poza_teczka
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "poinformowana_poza_teczka",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.poinformowana_poza_teczka,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
