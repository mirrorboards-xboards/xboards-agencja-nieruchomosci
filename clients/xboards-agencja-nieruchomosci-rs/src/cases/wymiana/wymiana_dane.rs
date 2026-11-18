//! The kind `wymiana-dane` of the room `cases/wymiana`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "wymiana-dane";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Dane do aktu w pokoju wymiany: dane osób, które biorą udział w akcie, dane nieruchomości,
-- rachunki do aktu i kredyt kupującego. Każdy wiersz jest szczegółem jednego wpisu
-- z wymiana_wpisy_wpisy, dopisuje się razem z nim i potem się nie zmienia; poprawka to nowy wpis.
--
-- Dane osób, nieruchomości i kredytu biuro przekazuje w imieniu strony, więc wymagają jej
-- wcześniejszego pełnomocnictwa (wymiana-dokumenty). Pokój nie trzyma danych, których akt nie
-- wymaga, np. dochodów kupującego ani numeru dokumentu tożsamości (R1).

-- Osoba albo podmiot, który bierze udział w akcie, z danymi z art. 92 § 1 pkt 4 i 4a PoN i innymi,
-- o które prosi kancelaria, np. stanem cywilnym. Każda dana pochodzi z dokumentu w teczce.
CREATE TABLE wymiana_dane_osoby (
  wpis_id TEXT PRIMARY KEY REFERENCES wymiana_wpisy_wpisy (id),
  -- Strona aktu, do której osoba należy i w której imieniu biuro pisze.
  strona_id TEXT NOT NULL REFERENCES wymiana_metryka_strony (id),
  -- Sama strona albo ktoś, kto działa w jej imieniu.
  udzial TEXT NOT NULL CHECK (udzial IN ('strona', 'przedstawiciel', 'pelnomocnik')),
  rodzaj_podmiotu TEXT NOT NULL CHECK (rodzaj_podmiotu IN ('osoba_fizyczna', 'podmiot')),
  imiona TEXT CHECK (imiona <> ''),
  nazwisko TEXT CHECK (nazwisko <> ''),
  imiona_rodzicow TEXT CHECK (imiona_rodzicow <> ''),
  miejsce_zamieszkania TEXT CHECK (miejsce_zamieszkania <> ''),
  -- Nazwa i siedziba osoby prawnej albo innego podmiotu.
  nazwa TEXT CHECK (nazwa <> ''),
  siedziba TEXT CHECK (siedziba <> ''),
  -- Obywatelstwo cudzoziemca, który nabywa nieruchomość (art. 92 § 1 pkt 4a PoN).
  obywatelstwo TEXT CHECK (obywatelstwo <> ''),
  -- Np. „zamężna, wspólność ustawowa”, gdy kancelaria o to prosi.
  stan_cywilny TEXT CHECK (stan_cywilny <> ''),
  -- Dokument w teczce transakcji, z którego pochodzą dane: identyfikator pokoju i dokumentu.
  zrodlo_dokument_odnosnik TEXT NOT NULL CHECK (zrodlo_dokument_odnosnik <> ''),

  CONSTRAINT osoba_ma_imiona_nazwisko_i_zamieszkanie CHECK (
    rodzaj_podmiotu <> 'osoba_fizyczna'
    OR (imiona IS NOT NULL AND nazwisko IS NOT NULL AND miejsce_zamieszkania IS NOT NULL)
  ),
  -- Imiona rodziców akt podaje przy osobach fizycznych, które biorą udział w akcie, a nie przy
  -- tych, które działają w cudzym imieniu.
  CONSTRAINT strona_fizyczna_ma_imiona_rodzicow CHECK (
    (rodzaj_podmiotu = 'osoba_fizyczna' AND udzial = 'strona') = (imiona_rodzicow IS NOT NULL)
  ),
  CONSTRAINT podmiot_ma_nazwe_i_siedzibe CHECK (
    (rodzaj_podmiotu = 'podmiot') = (nazwa IS NOT NULL AND siedziba IS NOT NULL)
  ),
  CONSTRAINT podmiot_bez_danych_osoby CHECK (
    rodzaj_podmiotu = 'osoba_fizyczna'
    OR (
      imiona IS NULL
      AND nazwisko IS NULL
      AND miejsce_zamieszkania IS NULL
      AND obywatelstwo IS NULL
      AND stan_cywilny IS NULL
    )
  ),
  CONSTRAINT podmiot_tylko_strona CHECK (rodzaj_podmiotu = 'osoba_fizyczna' OR udzial = 'strona')
) STRICT;

-- Oznaczenie nieruchomości z odpisu księgi wieczystej i z ewidencji gruntów i budynków.
CREATE TABLE wymiana_dane_nieruchomosci (
  wpis_id TEXT PRIMARY KEY REFERENCES wymiana_wpisy_wpisy (id),
  -- Strona, w której imieniu biuro przekazuje dane, zwykle sprzedający.
  strona_id TEXT NOT NULL REFERENCES wymiana_metryka_strony (id),
  numer_kw TEXT CHECK (numer_kw <> ''),
  -- Np. „lokal mieszkalny nr 12 w budynku przy ul. Polnej 7, pow. 54,20 m²”.
  oznaczenie TEXT NOT NULL CHECK (oznaczenie <> ''),
  -- Numery działek z obrębem, gdy nieruchomość je ma.
  dzialki TEXT CHECK (dzialki <> ''),
  -- Skąd oznaczenie, np. „odpis zupełny KW z 2026-11-03, wypis z rejestru gruntów z 2026-11-04”.
  zrodlo TEXT NOT NULL CHECK (zrodlo <> '')
) STRICT;

-- Rachunek, który ma trafić do aktu. Od biura wpis mówi tylko, że rachunek sprawdzono i że
-- człowiek go zatwierdził (R4), i wskazuje tę decyzję w teczce transakcji. Od kancelarii,
-- np. rachunek do zapłaty podatku, jest informacją: Koordynator potwierdza go innym kanałem
-- i oddaje Kontrolerowi.
CREATE TABLE wymiana_dane_rachunki (
  wpis_id TEXT PRIMARY KEY REFERENCES wymiana_wpisy_wpisy (id),
  numer TEXT NOT NULL CHECK (numer <> ''),
  posiadacz TEXT NOT NULL CHECK (posiadacz <> ''),
  cel TEXT NOT NULL CHECK (
    cel IN ('zaplata_ceny', 'zadatek', 'splata_hipoteki', 'depozyt_notarialny', 'podatki_i_oplaty', 'inny')
  ),
  -- Decyzja dopuszczenie_rachunku z teczki transakcji: identyfikator pokoju i decyzji.
  decyzja_odnosnik TEXT CHECK (decyzja_odnosnik <> '')
) STRICT;

-- Kredyt kupującego: tylko to, czego potrzebuje akt, gdy kancelaria o to prosi.
CREATE TABLE wymiana_dane_kredyty (
  wpis_id TEXT PRIMARY KEY REFERENCES wymiana_wpisy_wpisy (id),
  -- Kredytobiorca, w którego imieniu biuro przekazuje dane.
  strona_id TEXT NOT NULL REFERENCES wymiana_metryka_strony (id),
  bank TEXT NOT NULL CHECK (bank <> ''),
  -- Kwota kredytu z decyzji banku, w groszach.
  kwota INTEGER NOT NULL CHECK (kwota > 0),
  warunki_wyplaty TEXT CHECK (warunki_wyplaty <> '')
) STRICT;

-- Szczegół dopisuje się w tej samej operacji co jego wpis. Późniejszy dopisek zmieniałby wpis,
-- który druga strona już przeczytała, także w zamkniętym pokoju.
CREATE TRIGGER wymiana_dane_osoby_do_swojego_wpisu
BEFORE INSERT ON wymiana_dane_osoby
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
    AND rodzaj = 'dane_osoby'
    AND rowid = (
      SELECT max(rowid)
      FROM wymiana_wpisy_wpisy
    )
)
BEGIN
  SELECT RAISE(ABORT, 'dane osoby należą do najnowszego wpisu dane_osoby');
END;

CREATE TRIGGER wymiana_dane_nieruchomosci_do_swojego_wpisu
BEFORE INSERT ON wymiana_dane_nieruchomosci
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
    AND rodzaj = 'dane_nieruchomosci'
    AND rowid = (
      SELECT max(rowid)
      FROM wymiana_wpisy_wpisy
    )
)
BEGIN
  SELECT RAISE(ABORT, 'dane nieruchomości należą do najnowszego wpisu dane_nieruchomosci');
END;

CREATE TRIGGER wymiana_dane_rachunki_do_swojego_wpisu
BEFORE INSERT ON wymiana_dane_rachunki
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
    AND rodzaj = 'rachunek'
    AND rowid = (
      SELECT max(rowid)
      FROM wymiana_wpisy_wpisy
    )
)
BEGIN
  SELECT RAISE(ABORT, 'rachunek należy do najnowszego wpisu rachunek');
END;

CREATE TRIGGER wymiana_dane_kredyty_do_swojego_wpisu
BEFORE INSERT ON wymiana_dane_kredyty
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
    AND rodzaj = 'kredyt'
    AND rowid = (
      SELECT max(rowid)
      FROM wymiana_wpisy_wpisy
    )
)
BEGIN
  SELECT RAISE(ABORT, 'kredyt należy do najnowszego wpisu kredyt');
END;

-- Pisanie bez pełnomocnictwa: dane strony biuro przekazuje dopiero po jej pełnomocnictwie.
CREATE TRIGGER wymiana_dane_osoby_wymagaja_pelnomocnictwa
BEFORE INSERT ON wymiana_dane_osoby
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_dokumenty_wpisy
  WHERE rodzaj = 'pelnomocnictwo'
    AND strona_id = NEW.strona_id
)
BEGIN
  SELECT RAISE(ABORT, 'dane strony wymagają wpisanego wcześniej pełnomocnictwa tej strony');
END;

CREATE TRIGGER wymiana_dane_nieruchomosci_wymagaja_pelnomocnictwa
BEFORE INSERT ON wymiana_dane_nieruchomosci
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_dokumenty_wpisy
  WHERE rodzaj = 'pelnomocnictwo'
    AND strona_id = NEW.strona_id
)
BEGIN
  SELECT RAISE(ABORT, 'dane strony wymagają wpisanego wcześniej pełnomocnictwa tej strony');
END;

CREATE TRIGGER wymiana_dane_kredyty_wymagaja_pelnomocnictwa
BEFORE INSERT ON wymiana_dane_kredyty
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_dokumenty_wpisy
  WHERE rodzaj = 'pelnomocnictwo'
    AND strona_id = NEW.strona_id
)
BEGIN
  SELECT RAISE(ABORT, 'dane strony wymagają wpisanego wcześniej pełnomocnictwa tej strony');
END;

-- Rachunek od biura stoi na decyzji człowieka z teczki; decyzje biura nie są sprawą kancelarii.
CREATE TRIGGER wymiana_dane_rachunek_biura_z_decyzja
BEFORE INSERT ON wymiana_dane_rachunki
WHEN (
  SELECT strona = 'biuro'
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
) IS NOT (NEW.decyzja_odnosnik IS NOT NULL)
BEGIN
  SELECT RAISE(ABORT, 'rachunek od biura ma odnośnik do decyzji, a rachunek od kancelarii jest informacją bez decyzji');
END;

CREATE TRIGGER wymiana_dane_osoby_bez_zmian
BEFORE UPDATE ON wymiana_dane_osoby
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

CREATE TRIGGER wymiana_dane_nieruchomosci_bez_zmian
BEFORE UPDATE ON wymiana_dane_nieruchomosci
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

CREATE TRIGGER wymiana_dane_rachunki_bez_zmian
BEFORE UPDATE ON wymiana_dane_rachunki
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

CREATE TRIGGER wymiana_dane_kredyty_bez_zmian
BEFORE UPDATE ON wymiana_dane_kredyty
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_dane_osoby_bez_zastepowania
BEFORE INSERT ON wymiana_dane_osoby
WHEN EXISTS (
  SELECT 1
  FROM wymiana_dane_osoby
  WHERE wpis_id = NEW.wpis_id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_dane_osoby_bez_usuwania
BEFORE DELETE ON wymiana_dane_osoby
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_dane_nieruchomosci_bez_zastepowania
BEFORE INSERT ON wymiana_dane_nieruchomosci
WHEN EXISTS (
  SELECT 1
  FROM wymiana_dane_nieruchomosci
  WHERE wpis_id = NEW.wpis_id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_dane_nieruchomosci_bez_usuwania
BEFORE DELETE ON wymiana_dane_nieruchomosci
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_dane_rachunki_bez_zastepowania
BEFORE INSERT ON wymiana_dane_rachunki
WHEN EXISTS (
  SELECT 1
  FROM wymiana_dane_rachunki
  WHERE wpis_id = NEW.wpis_id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_dane_rachunki_bez_usuwania
BEFORE DELETE ON wymiana_dane_rachunki
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_dane_kredyty_bez_zastepowania
BEFORE INSERT ON wymiana_dane_kredyty
WHEN EXISTS (
  SELECT 1
  FROM wymiana_dane_kredyty
  WHERE wpis_id = NEW.wpis_id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_dane_kredyty_bez_usuwania
BEFORE DELETE ON wymiana_dane_kredyty
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;
"#,
];

/// The values of column `osoby.udzial`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OsobyUdzial {
    Strona,
    Przedstawiciel,
    Pelnomocnik,
}

impl OsobyUdzial {
    /// Every value, in the order of the catalog.
    pub const ALL: &[OsobyUdzial] = &[
        OsobyUdzial::Strona,
        OsobyUdzial::Przedstawiciel,
        OsobyUdzial::Pelnomocnik,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            OsobyUdzial::Strona => "strona",
            OsobyUdzial::Przedstawiciel => "przedstawiciel",
            OsobyUdzial::Pelnomocnik => "pelnomocnik",
        }
    }
}

impl xmeta_room::FromSqlValue for OsobyUdzial {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "strona" => Ok(OsobyUdzial::Strona),
                    "przedstawiciel" => Ok(OsobyUdzial::Przedstawiciel),
                    "pelnomocnik" => Ok(OsobyUdzial::Pelnomocnik),
                    other => Err(format!("{other} is no value of OsobyUdzial")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for OsobyUdzial {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `osoby.rodzaj_podmiotu`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OsobyRodzajPodmiotu {
    OsobaFizyczna,
    Podmiot,
}

impl OsobyRodzajPodmiotu {
    /// Every value, in the order of the catalog.
    pub const ALL: &[OsobyRodzajPodmiotu] = &[
        OsobyRodzajPodmiotu::OsobaFizyczna,
        OsobyRodzajPodmiotu::Podmiot,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            OsobyRodzajPodmiotu::OsobaFizyczna => "osoba_fizyczna",
            OsobyRodzajPodmiotu::Podmiot => "podmiot",
        }
    }
}

impl xmeta_room::FromSqlValue for OsobyRodzajPodmiotu {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "osoba_fizyczna" => Ok(OsobyRodzajPodmiotu::OsobaFizyczna),
                    "podmiot" => Ok(OsobyRodzajPodmiotu::Podmiot),
                    other => Err(format!("{other} is no value of OsobyRodzajPodmiotu")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for OsobyRodzajPodmiotu {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `rachunki.cel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RachunkiCel {
    ZaplataCeny,
    Zadatek,
    SplataHipoteki,
    DepozytNotarialny,
    PodatkiIOplaty,
    Inny,
}

impl RachunkiCel {
    /// Every value, in the order of the catalog.
    pub const ALL: &[RachunkiCel] = &[
        RachunkiCel::ZaplataCeny,
        RachunkiCel::Zadatek,
        RachunkiCel::SplataHipoteki,
        RachunkiCel::DepozytNotarialny,
        RachunkiCel::PodatkiIOplaty,
        RachunkiCel::Inny,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            RachunkiCel::ZaplataCeny => "zaplata_ceny",
            RachunkiCel::Zadatek => "zadatek",
            RachunkiCel::SplataHipoteki => "splata_hipoteki",
            RachunkiCel::DepozytNotarialny => "depozyt_notarialny",
            RachunkiCel::PodatkiIOplaty => "podatki_i_oplaty",
            RachunkiCel::Inny => "inny",
        }
    }
}

impl xmeta_room::FromSqlValue for RachunkiCel {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "zaplata_ceny" => Ok(RachunkiCel::ZaplataCeny),
                    "zadatek" => Ok(RachunkiCel::Zadatek),
                    "splata_hipoteki" => Ok(RachunkiCel::SplataHipoteki),
                    "depozyt_notarialny" => Ok(RachunkiCel::DepozytNotarialny),
                    "podatki_i_oplaty" => Ok(RachunkiCel::PodatkiIOplaty),
                    "inny" => Ok(RachunkiCel::Inny),
                    other => Err(format!("{other} is no value of RachunkiCel")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for RachunkiCel {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of field `strona` of `przekaz_kredyt`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Strona {
    Biuro,
    Kancelaria,
}

impl Strona {
    /// Every value, in the order of the catalog.
    pub const ALL: &[Strona] = &[Strona::Biuro, Strona::Kancelaria];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            Strona::Biuro => "biuro",
            Strona::Kancelaria => "kancelaria",
        }
    }
}

impl xmeta_room::FromSqlValue for Strona {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "biuro" => Ok(Strona::Biuro),
                    "kancelaria" => Ok(Strona::Kancelaria),
                    other => Err(format!("{other} is no value of Strona")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for Strona {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `kredyty`.
#[derive(Debug, Clone, PartialEq)]
pub struct KredytyRow {
    pub wpis_id: String,
    pub strona_id: String,
    pub bank: String,
    pub kwota: i64,
    pub warunki_wyplaty: Option<String>,
    pub autor: String,
    pub chwila: String,
    pub poprawia_id: Option<String>,
}

impl KredytyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(KredytyRow {
            wpis_id: row.get("wpis_id")?,
            strona_id: row.get("strona_id")?,
            bank: row.get("bank")?,
            kwota: row.get("kwota")?,
            warunki_wyplaty: row.get("warunki_wyplaty")?,
            autor: row.get("autor")?,
            chwila: row.get("chwila")?,
            poprawia_id: row.get("poprawia_id")?,
        })
    }
}

/// Kredyt kupującego w tym zakresie, którego potrzebuje akt.
pub fn kredyty(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<KredytyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT kredyt.*, wpis.autor, wpis.chwila, wpis.poprawia_id
FROM wymiana_dane_kredyty AS kredyt
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = kredyt.wpis_id
ORDER BY wpis.rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(KredytyRow::from_row).collect()
}

/// The parameters of `przekaz_kredyt`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzekazKredytParams {
    pub wpis_id: String,
    pub strona: Strona,
    pub autor: String,
    pub chwila: String,
    pub odpowiada_na_id: Option<String>,
    pub poprawia_id: Option<String>,
    pub strona_id: String,
    pub bank: String,
    pub kwota: i64,
    pub warunki_wyplaty: Option<String>,
}

/// Biuro przekazuje w imieniu kupującego bank, kwotę z decyzji i warunki wypłaty, gdy kancelaria
/// o nie prosi.
pub fn przekaz_kredyt(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzekazKredytParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id, poprawia_id)
VALUES (:wpis_id, :strona, :autor, 'kredyt', :chwila, :odpowiada_na_id, :poprawia_id)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    ("strona", xmeta_room::ToSqlValue::to_sql_value(&params.strona)),
                    ("autor", xmeta_room::ToSqlValue::to_sql_value(&params.autor)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    (
                        "odpowiada_na_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odpowiada_na_id),
                    ),
                    (
                        "poprawia_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprawia_id),
                    ),
                ]),
            },
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_dane_kredyty (wpis_id, strona_id, bank, kwota, warunki_wyplaty)
VALUES (:wpis_id, :strona_id, :bank, :kwota, :warunki_wyplaty)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    (
                        "strona_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.strona_id),
                    ),
                    ("bank", xmeta_room::ToSqlValue::to_sql_value(&params.bank)),
                    ("kwota", xmeta_room::ToSqlValue::to_sql_value(&params.kwota)),
                    (
                        "warunki_wyplaty",
                        xmeta_room::ToSqlValue::to_sql_value(&params.warunki_wyplaty),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `dane_nieruchomosci`.
#[derive(Debug, Clone, PartialEq)]
pub struct DaneNieruchomosciRow {
    pub wpis_id: String,
    pub strona_id: String,
    pub numer_kw: Option<String>,
    pub oznaczenie: String,
    pub dzialki: Option<String>,
    pub zrodlo: String,
    pub autor: String,
    pub chwila: String,
    pub poprawia_id: Option<String>,
}

impl DaneNieruchomosciRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(DaneNieruchomosciRow {
            wpis_id: row.get("wpis_id")?,
            strona_id: row.get("strona_id")?,
            numer_kw: row.get("numer_kw")?,
            oznaczenie: row.get("oznaczenie")?,
            dzialki: row.get("dzialki")?,
            zrodlo: row.get("zrodlo")?,
            autor: row.get("autor")?,
            chwila: row.get("chwila")?,
            poprawia_id: row.get("poprawia_id")?,
        })
    }
}

/// Oznaczenie nieruchomości ze źródłem, z wpisami, które je przyniosły.
pub fn dane_nieruchomosci(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<DaneNieruchomosciRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT nieruchomosc.*, wpis.autor, wpis.chwila, wpis.poprawia_id
FROM wymiana_dane_nieruchomosci AS nieruchomosc
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = nieruchomosc.wpis_id
ORDER BY wpis.rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(DaneNieruchomosciRow::from_row).collect()
}

/// The parameters of `przekaz_dane_nieruchomosci`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzekazDaneNieruchomosciParams {
    pub wpis_id: String,
    pub strona: Strona,
    pub autor: String,
    pub chwila: String,
    pub odpowiada_na_id: Option<String>,
    pub poprawia_id: Option<String>,
    pub strona_id: String,
    pub numer_kw: Option<String>,
    pub oznaczenie: String,
    pub dzialki: Option<String>,
    pub zrodlo: String,
}

/// Biuro przekazuje w imieniu strony oznaczenie nieruchomości z odpisu księgi i z ewidencji.
pub fn przekaz_dane_nieruchomosci(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzekazDaneNieruchomosciParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id, poprawia_id)
VALUES (:wpis_id, :strona, :autor, 'dane_nieruchomosci', :chwila, :odpowiada_na_id, :poprawia_id)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    ("strona", xmeta_room::ToSqlValue::to_sql_value(&params.strona)),
                    ("autor", xmeta_room::ToSqlValue::to_sql_value(&params.autor)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    (
                        "odpowiada_na_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odpowiada_na_id),
                    ),
                    (
                        "poprawia_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprawia_id),
                    ),
                ]),
            },
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_dane_nieruchomosci (wpis_id, strona_id, numer_kw, oznaczenie, dzialki, zrodlo)
VALUES (:wpis_id, :strona_id, :numer_kw, :oznaczenie, :dzialki, :zrodlo)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    (
                        "strona_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.strona_id),
                    ),
                    ("numer_kw", xmeta_room::ToSqlValue::to_sql_value(&params.numer_kw)),
                    (
                        "oznaczenie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.oznaczenie),
                    ),
                    ("dzialki", xmeta_room::ToSqlValue::to_sql_value(&params.dzialki)),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                ]),
            },
        ],
    )
}

/// A row of `dane_osob`.
#[derive(Debug, Clone, PartialEq)]
pub struct DaneOsobRow {
    pub wpis_id: String,
    pub strona_id: String,
    pub udzial: OsobyUdzial,
    pub rodzaj_podmiotu: OsobyRodzajPodmiotu,
    pub imiona: Option<String>,
    pub nazwisko: Option<String>,
    pub imiona_rodzicow: Option<String>,
    pub miejsce_zamieszkania: Option<String>,
    pub nazwa: Option<String>,
    pub siedziba: Option<String>,
    pub obywatelstwo: Option<String>,
    pub stan_cywilny: Option<String>,
    pub zrodlo_dokument_odnosnik: String,
    pub autor: String,
    pub chwila: String,
    pub poprawia_id: Option<String>,
}

impl DaneOsobRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(DaneOsobRow {
            wpis_id: row.get("wpis_id")?,
            strona_id: row.get("strona_id")?,
            udzial: row.get("udzial")?,
            rodzaj_podmiotu: row.get("rodzaj_podmiotu")?,
            imiona: row.get("imiona")?,
            nazwisko: row.get("nazwisko")?,
            imiona_rodzicow: row.get("imiona_rodzicow")?,
            miejsce_zamieszkania: row.get("miejsce_zamieszkania")?,
            nazwa: row.get("nazwa")?,
            siedziba: row.get("siedziba")?,
            obywatelstwo: row.get("obywatelstwo")?,
            stan_cywilny: row.get("stan_cywilny")?,
            zrodlo_dokument_odnosnik: row.get("zrodlo_dokument_odnosnik")?,
            autor: row.get("autor")?,
            chwila: row.get("chwila")?,
            poprawia_id: row.get("poprawia_id")?,
        })
    }
}

/// Dane osób do aktu z wpisami, które je przyniosły; poprawione zostają obok poprawek.
pub fn dane_osob(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<DaneOsobRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT osoba.*, wpis.autor, wpis.chwila, wpis.poprawia_id
FROM wymiana_dane_osoby AS osoba
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = osoba.wpis_id
ORDER BY wpis.rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(DaneOsobRow::from_row).collect()
}

/// The parameters of `przekaz_dane_osoby`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzekazDaneOsobyParams {
    pub wpis_id: String,
    pub strona: Strona,
    pub autor: String,
    pub chwila: String,
    pub odpowiada_na_id: Option<String>,
    pub poprawia_id: Option<String>,
    pub strona_id: String,
    pub udzial: OsobyUdzial,
    pub rodzaj_podmiotu: OsobyRodzajPodmiotu,
    pub imiona: Option<String>,
    pub nazwisko: Option<String>,
    pub imiona_rodzicow: Option<String>,
    pub miejsce_zamieszkania: Option<String>,
    pub nazwa: Option<String>,
    pub siedziba: Option<String>,
    pub obywatelstwo: Option<String>,
    pub stan_cywilny: Option<String>,
    pub zrodlo_dokument_odnosnik: String,
}

/// Biuro przekazuje w imieniu strony dane osoby, która bierze udział w akcie, ze źródłem w teczce.
pub fn przekaz_dane_osoby(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzekazDaneOsobyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id, poprawia_id)
VALUES (:wpis_id, :strona, :autor, 'dane_osoby', :chwila, :odpowiada_na_id, :poprawia_id)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    ("strona", xmeta_room::ToSqlValue::to_sql_value(&params.strona)),
                    ("autor", xmeta_room::ToSqlValue::to_sql_value(&params.autor)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    (
                        "odpowiada_na_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odpowiada_na_id),
                    ),
                    (
                        "poprawia_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprawia_id),
                    ),
                ]),
            },
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_dane_osoby (wpis_id, strona_id, udzial, rodzaj_podmiotu, imiona, nazwisko,
  imiona_rodzicow, miejsce_zamieszkania, nazwa, siedziba, obywatelstwo, stan_cywilny,
  zrodlo_dokument_odnosnik)
VALUES (:wpis_id, :strona_id, :udzial, :rodzaj_podmiotu, :imiona, :nazwisko, :imiona_rodzicow,
  :miejsce_zamieszkania, :nazwa, :siedziba, :obywatelstwo, :stan_cywilny,
  :zrodlo_dokument_odnosnik)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    (
                        "strona_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.strona_id),
                    ),
                    ("udzial", xmeta_room::ToSqlValue::to_sql_value(&params.udzial)),
                    (
                        "rodzaj_podmiotu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj_podmiotu),
                    ),
                    ("imiona", xmeta_room::ToSqlValue::to_sql_value(&params.imiona)),
                    ("nazwisko", xmeta_room::ToSqlValue::to_sql_value(&params.nazwisko)),
                    (
                        "imiona_rodzicow",
                        xmeta_room::ToSqlValue::to_sql_value(&params.imiona_rodzicow),
                    ),
                    (
                        "miejsce_zamieszkania",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.miejsce_zamieszkania,
                        ),
                    ),
                    ("nazwa", xmeta_room::ToSqlValue::to_sql_value(&params.nazwa)),
                    ("siedziba", xmeta_room::ToSqlValue::to_sql_value(&params.siedziba)),
                    (
                        "obywatelstwo",
                        xmeta_room::ToSqlValue::to_sql_value(&params.obywatelstwo),
                    ),
                    (
                        "stan_cywilny",
                        xmeta_room::ToSqlValue::to_sql_value(&params.stan_cywilny),
                    ),
                    (
                        "zrodlo_dokument_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.zrodlo_dokument_odnosnik,
                        ),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `rachunki_do_aktu`.
#[derive(Debug, Clone, PartialEq)]
pub struct RachunkiDoAktuRow {
    pub wpis_id: String,
    pub numer: String,
    pub posiadacz: String,
    pub cel: RachunkiCel,
    pub decyzja_odnosnik: Option<String>,
    pub strona: Strona,
    pub autor: String,
    pub chwila: String,
    pub poprawia_id: Option<String>,
    pub tresc: Option<String>,
}

impl RachunkiDoAktuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(RachunkiDoAktuRow {
            wpis_id: row.get("wpis_id")?,
            numer: row.get("numer")?,
            posiadacz: row.get("posiadacz")?,
            cel: row.get("cel")?,
            decyzja_odnosnik: row.get("decyzja_odnosnik")?,
            strona: row.get("strona")?,
            autor: row.get("autor")?,
            chwila: row.get("chwila")?,
            poprawia_id: row.get("poprawia_id")?,
            tresc: row.get("tresc")?,
        })
    }
}

/// Rachunki obu stron pokoju z wpisami; od biura z decyzją, od kancelarii jako informacja.
pub fn rachunki_do_aktu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<RachunkiDoAktuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT rachunek.*, wpis.strona, wpis.autor, wpis.chwila, wpis.poprawia_id, wpis.tresc
FROM wymiana_dane_rachunki AS rachunek
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = rachunek.wpis_id
ORDER BY wpis.rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(RachunkiDoAktuRow::from_row).collect()
}

/// The parameters of `podaj_rachunek`.
#[derive(Debug, Clone, PartialEq)]
pub struct PodajRachunekParams {
    pub wpis_id: String,
    pub strona: Strona,
    pub autor: String,
    pub chwila: String,
    pub odpowiada_na_id: Option<String>,
    pub poprawia_id: Option<String>,
    pub tresc: Option<String>,
    pub numer: String,
    pub posiadacz: String,
    pub cel: RachunkiCel,
    pub decyzja_odnosnik: Option<String>,
}

/// Biuro przekazuje rachunek zatwierdzony przez człowieka; kancelaria podaje rachunek jako informację.
pub fn podaj_rachunek(
    room: &impl xmeta_room::MetaRoom,
    params: &PodajRachunekParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id, poprawia_id,
  tresc)
VALUES (:wpis_id, :strona, :autor, 'rachunek', :chwila, :odpowiada_na_id, :poprawia_id, :tresc)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    ("strona", xmeta_room::ToSqlValue::to_sql_value(&params.strona)),
                    ("autor", xmeta_room::ToSqlValue::to_sql_value(&params.autor)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    (
                        "odpowiada_na_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odpowiada_na_id),
                    ),
                    (
                        "poprawia_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprawia_id),
                    ),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                ]),
            },
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_dane_rachunki (wpis_id, numer, posiadacz, cel, decyzja_odnosnik)
VALUES (:wpis_id, :numer, :posiadacz, :cel, :decyzja_odnosnik)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    ("numer", xmeta_room::ToSqlValue::to_sql_value(&params.numer)),
                    (
                        "posiadacz",
                        xmeta_room::ToSqlValue::to_sql_value(&params.posiadacz),
                    ),
                    ("cel", xmeta_room::ToSqlValue::to_sql_value(&params.cel)),
                    (
                        "decyzja_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_odnosnik),
                    ),
                ]),
            },
        ],
    )
}
