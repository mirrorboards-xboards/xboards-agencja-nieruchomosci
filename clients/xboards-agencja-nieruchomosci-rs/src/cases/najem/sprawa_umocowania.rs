//! The kind `sprawa-umocowania` of the room `cases/najem`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "sprawa-umocowania";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Pełnomocnictwa i reprezentacja podmiotów w jednej tabeli: kto, komu, do czego i w jakiej formie.
-- Agent występuje w imieniu klienta tylko w granicach pełnomocnictwa, które klient dał biuru
-- (zasada „W imieniu klienta tylko z pełnomocnictwem”). Role „pełnomocnik” i „reprezentant”
-- wynikają z tej tabeli, więc nie ma ich w sprawa_uczestnicy_role.
--
-- Umocowania wpisuje agent sprawy, a sprawdza je Kontroler. Wersja umocowania rośnie przy każdej
-- zmianie; sprawdzenie dotyczy jednej wersji, a trigger, który zeruje jego aktualność, leży
-- w migracji sprawa-sprawdzenia (sprawa_sprawdzenia_nowa_wersja_umocowania).
--
-- Czy czynność mieści się w zakresie, rozstrzyga lista sprawa_umocowania_czynnosci. Reguła
-- wysyłki w imieniu uczestnika leży w migracji sprawa-korespondencja
-- (sprawa_korespondencja_w_imieniu_uczestnika_z_umocowaniem).
CREATE TABLE sprawa_umocowania_wpisy (
  id TEXT PRIMARY KEY,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('pelnomocnictwo', 'reprezentacja')),
  mocodawca_uczestnik_id TEXT NOT NULL REFERENCES sprawa_uczestnicy_wpisy (id),
  -- Umocowany: uczestnik teczki albo samo biuro.
  umocowany_uczestnik_id TEXT REFERENCES sprawa_uczestnicy_wpisy (id),
  umocowane_biuro INTEGER NOT NULL DEFAULT 0 CHECK (umocowane_biuro IN (0, 1)),
  -- Art. 98 KC: ogólne obejmuje czynności zwykłego zarządu, rodzajowe określa ich rodzaj,
  -- a do czynności wskazuje tę czynność.
  zakres TEXT NOT NULL CHECK (zakres IN ('ogolne', 'rodzajowe', 'do_czynnosci')),
  -- Art. 99 § 1 KC: forma czynności, do której umocowuje; reprezentacja wynika z rejestru.
  forma TEXT CHECK (forma IN ('pisemna', 'elektroniczna', 'z_podpisem_poswiadczonym', 'akt_notarialny', 'z_rejestru')),
  -- Kto i w jakim składzie działa za podmiot, np. „dwóch członków zarządu łącznie”.
  sposob_reprezentacji TEXT,
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  udzielono TEXT,
  wazne_do TEXT,
  -- Art. 101 KC: odwołanie, śmierć albo upływ terminu, z datą i źródłem wiadomości.
  wygaslo TEXT,
  wygasniecie_powod TEXT CHECK (wygasniecie_powod IN ('odwolanie', 'smierc_mocodawcy', 'smierc_umocowanego', 'uplyw_terminu', 'wykreslenie_z_rejestru', 'inne')),
  wygasniecie_zrodlo TEXT,
  wersja INTEGER NOT NULL DEFAULT 1 CHECK (wersja >= 1),

  CONSTRAINT umocowany_osoba_albo_biuro CHECK ((umocowany_uczestnik_id IS NOT NULL) <> (umocowane_biuro = 1)),
  CONSTRAINT wygasniecie_ma_powod_i_zrodlo CHECK (
    (wygaslo IS NULL) = (wygasniecie_powod IS NULL)
    AND (wygaslo IS NULL) = (wygasniecie_zrodlo IS NULL)
  ),
  CONSTRAINT nie_umocowuje_siebie CHECK (umocowany_uczestnik_id IS NOT mocodawca_uczestnik_id)
) STRICT;

-- Czynności w zakresie umocowania, np. wystapienie_o_zaswiadczenie, uzgodnienie_terminu_aktu.
-- Nazwy ustala typ sprawy; ta sama nazwa wraca w wysyłce w imieniu uczestnika.
CREATE TABLE sprawa_umocowania_czynnosci (
  -- Klucz nadany przez bazę jest dodatni; zob. sprawa_umocowania_czynnosci_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  umocowanie_id TEXT NOT NULL REFERENCES sprawa_umocowania_wpisy (id),
  czynnosc TEXT NOT NULL CHECK (czynnosc <> '' AND czynnosc NOT GLOB '*[^a-z_]*'),

  UNIQUE (umocowanie_id, czynnosc)
) STRICT;

CREATE TRIGGER sprawa_umocowania_reprezentacja_tylko_podmiotu
BEFORE INSERT ON sprawa_umocowania_wpisy
WHEN NEW.rodzaj = 'reprezentacja'
  AND (
    SELECT rodzaj_podmiotu = 'osoba_fizyczna'
    FROM sprawa_uczestnicy_wpisy
    WHERE id = NEW.mocodawca_uczestnik_id
  )
BEGIN
  SELECT RAISE(ABORT, 'reprezentację ma tylko podmiot; osoba fizyczna udziela pełnomocnictwa');
END;

CREATE TRIGGER sprawa_umocowania_zmieniona_reprezentacja_tylko_podmiotu
BEFORE UPDATE OF rodzaj, mocodawca_uczestnik_id ON sprawa_umocowania_wpisy
WHEN NEW.rodzaj = 'reprezentacja'
  AND (
    SELECT rodzaj_podmiotu = 'osoba_fizyczna'
    FROM sprawa_uczestnicy_wpisy
    WHERE id = NEW.mocodawca_uczestnik_id
  )
BEGIN
  SELECT RAISE(ABORT, 'reprezentację ma tylko podmiot; osoba fizyczna udziela pełnomocnictwa');
END;

-- Data udzielenia mówi, od kiedy biuro mogło działać w imieniu mocodawcy; jej zmiana
-- przepisałaby podstawę wcześniejszych wysyłek.
CREATE TRIGGER sprawa_umocowania_udzielono_niezmienne
BEFORE UPDATE OF udzielono ON sprawa_umocowania_wpisy
WHEN NEW.udzielono IS NOT OLD.udzielono
BEGIN
  SELECT RAISE(ABORT, 'data udzielenia umocowania się nie zmienia; inne umocowanie to nowy wiersz');
END;

-- Wygasłe umocowanie nie odżywa (art. 101 KC): wygaśnięcie wpisuje się raz i się go nie zeruje.
-- Ponowne umocowanie to nowy wiersz.
CREATE TRIGGER sprawa_umocowania_wygasniecie_raz
BEFORE UPDATE OF wygaslo, wygasniecie_powod, wygasniecie_zrodlo ON sprawa_umocowania_wpisy
WHEN OLD.wygaslo IS NOT NULL
  AND (
    NEW.wygaslo IS NOT OLD.wygaslo
    OR NEW.wygasniecie_powod IS NOT OLD.wygasniecie_powod
    OR NEW.wygasniecie_zrodlo IS NOT OLD.wygasniecie_zrodlo
  )
BEGIN
  SELECT RAISE(ABORT, 'wygaśnięcie umocowania wpisuje się raz i się go nie cofa; ponowne umocowanie to nowy wiersz');
END;

CREATE TRIGGER sprawa_umocowania_zmiana_podnosi_wersje
AFTER UPDATE ON sprawa_umocowania_wpisy
WHEN NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.mocodawca_uczestnik_id IS NOT OLD.mocodawca_uczestnik_id
  OR NEW.umocowany_uczestnik_id IS NOT OLD.umocowany_uczestnik_id
  OR NEW.umocowane_biuro IS NOT OLD.umocowane_biuro
  OR NEW.zakres IS NOT OLD.zakres
  OR NEW.forma IS NOT OLD.forma
  OR NEW.sposob_reprezentacji IS NOT OLD.sposob_reprezentacji
  OR NEW.dokument_id IS NOT OLD.dokument_id
  OR NEW.udzielono IS NOT OLD.udzielono
  OR NEW.wazne_do IS NOT OLD.wazne_do
  OR NEW.wygaslo IS NOT OLD.wygaslo
  OR NEW.wygasniecie_powod IS NOT OLD.wygasniecie_powod
  OR NEW.wygasniecie_zrodlo IS NOT OLD.wygasniecie_zrodlo
BEGIN
  UPDATE sprawa_umocowania_wpisy
  SET wersja = wersja + 1
  WHERE id = NEW.id;
END;

CREATE TRIGGER sprawa_umocowania_nowa_czynnosc_podnosi_wersje
AFTER INSERT ON sprawa_umocowania_czynnosci
BEGIN
  UPDATE sprawa_umocowania_wpisy
  SET wersja = wersja + 1
  WHERE id = NEW.umocowanie_id;
END;

-- Zakres jest częścią sprawdzonego umocowania; nowa czynność to nowy wiersz.
CREATE TRIGGER sprawa_umocowania_czynnosci_bez_zmian
BEFORE UPDATE ON sprawa_umocowania_czynnosci
BEGIN
  SELECT RAISE(ABORT, 'czynności w zakresie umocowania się nie zmienia; nowa czynność to nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_umocowania_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_umocowania_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_umocowania_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_umocowania_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_umocowania_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER sprawa_umocowania_czynnosci_bez_zastepowania
BEFORE INSERT ON sprawa_umocowania_czynnosci
WHEN EXISTS (
    SELECT 1
    FROM sprawa_umocowania_czynnosci
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM sprawa_umocowania_czynnosci
    WHERE umocowanie_id = NEW.umocowanie_id
      AND czynnosc = NEW.czynnosc
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_umocowania_czynnosci_bez_usuwania
BEFORE DELETE ON sprawa_umocowania_czynnosci
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
];

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    Pelnomocnictwo,
    Reprezentacja,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::Pelnomocnictwo,
        WpisyRodzaj::Reprezentacja,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::Pelnomocnictwo => "pelnomocnictwo",
            WpisyRodzaj::Reprezentacja => "reprezentacja",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "pelnomocnictwo" => Ok(WpisyRodzaj::Pelnomocnictwo),
                    "reprezentacja" => Ok(WpisyRodzaj::Reprezentacja),
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

/// The values of column `wpisy.zakres`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyZakres {
    Ogolne,
    Rodzajowe,
    DoCzynnosci,
}

impl WpisyZakres {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyZakres] = &[
        WpisyZakres::Ogolne,
        WpisyZakres::Rodzajowe,
        WpisyZakres::DoCzynnosci,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyZakres::Ogolne => "ogolne",
            WpisyZakres::Rodzajowe => "rodzajowe",
            WpisyZakres::DoCzynnosci => "do_czynnosci",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyZakres {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "ogolne" => Ok(WpisyZakres::Ogolne),
                    "rodzajowe" => Ok(WpisyZakres::Rodzajowe),
                    "do_czynnosci" => Ok(WpisyZakres::DoCzynnosci),
                    other => Err(format!("{other} is no value of WpisyZakres")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyZakres {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.forma`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyForma {
    Pisemna,
    Elektroniczna,
    ZPodpisemPoswiadczonym,
    AktNotarialny,
    ZRejestru,
}

impl WpisyForma {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyForma] = &[
        WpisyForma::Pisemna,
        WpisyForma::Elektroniczna,
        WpisyForma::ZPodpisemPoswiadczonym,
        WpisyForma::AktNotarialny,
        WpisyForma::ZRejestru,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyForma::Pisemna => "pisemna",
            WpisyForma::Elektroniczna => "elektroniczna",
            WpisyForma::ZPodpisemPoswiadczonym => "z_podpisem_poswiadczonym",
            WpisyForma::AktNotarialny => "akt_notarialny",
            WpisyForma::ZRejestru => "z_rejestru",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyForma {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "pisemna" => Ok(WpisyForma::Pisemna),
                    "elektroniczna" => Ok(WpisyForma::Elektroniczna),
                    "z_podpisem_poswiadczonym" => Ok(WpisyForma::ZPodpisemPoswiadczonym),
                    "akt_notarialny" => Ok(WpisyForma::AktNotarialny),
                    "z_rejestru" => Ok(WpisyForma::ZRejestru),
                    other => Err(format!("{other} is no value of WpisyForma")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyForma {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.wygasniecie_powod`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyWygasnieciePowod {
    Odwolanie,
    SmiercMocodawcy,
    SmiercUmocowanego,
    UplywTerminu,
    WykreslenieZRejestru,
    Inne,
}

impl WpisyWygasnieciePowod {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyWygasnieciePowod] = &[
        WpisyWygasnieciePowod::Odwolanie,
        WpisyWygasnieciePowod::SmiercMocodawcy,
        WpisyWygasnieciePowod::SmiercUmocowanego,
        WpisyWygasnieciePowod::UplywTerminu,
        WpisyWygasnieciePowod::WykreslenieZRejestru,
        WpisyWygasnieciePowod::Inne,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyWygasnieciePowod::Odwolanie => "odwolanie",
            WpisyWygasnieciePowod::SmiercMocodawcy => "smierc_mocodawcy",
            WpisyWygasnieciePowod::SmiercUmocowanego => "smierc_umocowanego",
            WpisyWygasnieciePowod::UplywTerminu => "uplyw_terminu",
            WpisyWygasnieciePowod::WykreslenieZRejestru => "wykreslenie_z_rejestru",
            WpisyWygasnieciePowod::Inne => "inne",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyWygasnieciePowod {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "odwolanie" => Ok(WpisyWygasnieciePowod::Odwolanie),
                    "smierc_mocodawcy" => Ok(WpisyWygasnieciePowod::SmiercMocodawcy),
                    "smierc_umocowanego" => Ok(WpisyWygasnieciePowod::SmiercUmocowanego),
                    "uplyw_terminu" => Ok(WpisyWygasnieciePowod::UplywTerminu),
                    "wykreslenie_z_rejestru" => {
                        Ok(WpisyWygasnieciePowod::WykreslenieZRejestru)
                    }
                    "inne" => Ok(WpisyWygasnieciePowod::Inne),
                    other => Err(format!("{other} is no value of WpisyWygasnieciePowod")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyWygasnieciePowod {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `czynnosci_umocowan`.
#[derive(Debug, Clone, PartialEq)]
pub struct CzynnosciUmocowanRow {
    pub id: i64,
    pub umocowanie_id: String,
    pub czynnosc: String,
}

impl CzynnosciUmocowanRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(CzynnosciUmocowanRow {
            id: row.get("id")?,
            umocowanie_id: row.get("umocowanie_id")?,
            czynnosc: row.get("czynnosc")?,
        })
    }
}

/// Czynności w zakresie wszystkich umocowań teczki.
pub fn czynnosci_umocowan(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<CzynnosciUmocowanRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_umocowania_czynnosci
ORDER BY umocowanie_id, czynnosc"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(CzynnosciUmocowanRow::from_row).collect()
}

/// The parameters of `dopisz_czynnosc_umocowania`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DopiszCzynnoscUmocowaniaParams {
    pub umocowanie_id: String,
    pub czynnosc: String,
}

/// Czynność, do której umocowanie upoważnia, nazwana jak w typie sprawy.
pub fn dopisz_czynnosc_umocowania(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszCzynnoscUmocowaniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_umocowania_czynnosci (umocowanie_id, czynnosc)
VALUES (:umocowanie_id, :czynnosc)"#,
                parameters: Vec::from([
                    (
                        "umocowanie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.umocowanie_id),
                    ),
                    ("czynnosc", xmeta_room::ToSqlValue::to_sql_value(&params.czynnosc)),
                ]),
            },
        ],
    )
}

/// A row of `umocowania`.
#[derive(Debug, Clone, PartialEq)]
pub struct UmocowaniaRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub mocodawca_uczestnik_id: String,
    pub umocowany_uczestnik_id: Option<String>,
    pub umocowane_biuro: bool,
    pub zakres: WpisyZakres,
    pub forma: Option<WpisyForma>,
    pub sposob_reprezentacji: Option<String>,
    pub dokument_id: Option<String>,
    pub udzielono: Option<String>,
    pub wazne_do: Option<String>,
    pub wygaslo: Option<String>,
    pub wygasniecie_powod: Option<WpisyWygasnieciePowod>,
    pub wygasniecie_zrodlo: Option<String>,
    pub wersja: i64,
}

impl UmocowaniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(UmocowaniaRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            mocodawca_uczestnik_id: row.get("mocodawca_uczestnik_id")?,
            umocowany_uczestnik_id: row.get("umocowany_uczestnik_id")?,
            umocowane_biuro: row.get("umocowane_biuro")?,
            zakres: row.get("zakres")?,
            forma: row.get("forma")?,
            sposob_reprezentacji: row.get("sposob_reprezentacji")?,
            dokument_id: row.get("dokument_id")?,
            udzielono: row.get("udzielono")?,
            wazne_do: row.get("wazne_do")?,
            wygaslo: row.get("wygaslo")?,
            wygasniecie_powod: row.get("wygasniecie_powod")?,
            wygasniecie_zrodlo: row.get("wygasniecie_zrodlo")?,
            wersja: row.get("wersja")?,
        })
    }
}

/// Pełnomocnictwa i reprezentacje w teczce, w kolejności wpisania.
pub fn umocowania(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<UmocowaniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_umocowania_wpisy
ORDER BY rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(UmocowaniaRow::from_row).collect()
}

/// The parameters of `wpisz_umocowanie`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszUmocowanieParams {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub mocodawca_uczestnik_id: String,
    pub umocowany_uczestnik_id: Option<String>,
    pub umocowane_biuro: bool,
    pub zakres: WpisyZakres,
    pub forma: Option<WpisyForma>,
    pub sposob_reprezentacji: Option<String>,
    pub dokument_id: Option<String>,
    pub udzielono: Option<String>,
    pub wazne_do: Option<String>,
}

/// Pełnomocnictwo albo reprezentacja podmiotu, z dokumentem, na którym stoi.
pub fn wpisz_umocowanie(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszUmocowanieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_umocowania_wpisy (id, rodzaj, mocodawca_uczestnik_id, umocowany_uczestnik_id,
  umocowane_biuro, zakres, forma, sposob_reprezentacji, dokument_id, udzielono, wazne_do)
VALUES (:id, :rodzaj, :mocodawca_uczestnik_id, :umocowany_uczestnik_id, :umocowane_biuro, :zakres,
  :forma, :sposob_reprezentacji, :dokument_id, :udzielono, :wazne_do)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    (
                        "mocodawca_uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.mocodawca_uczestnik_id,
                        ),
                    ),
                    (
                        "umocowany_uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.umocowany_uczestnik_id,
                        ),
                    ),
                    (
                        "umocowane_biuro",
                        xmeta_room::ToSqlValue::to_sql_value(&params.umocowane_biuro),
                    ),
                    ("zakres", xmeta_room::ToSqlValue::to_sql_value(&params.zakres)),
                    ("forma", xmeta_room::ToSqlValue::to_sql_value(&params.forma)),
                    (
                        "sposob_reprezentacji",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.sposob_reprezentacji,
                        ),
                    ),
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                    (
                        "udzielono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.udzielono),
                    ),
                    ("wazne_do", xmeta_room::ToSqlValue::to_sql_value(&params.wazne_do)),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_dane_umocowania`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZmienDaneUmocowaniaParams {
    pub rodzaj: WpisyRodzaj,
    pub mocodawca_uczestnik_id: String,
    pub umocowany_uczestnik_id: Option<String>,
    pub umocowane_biuro: bool,
    pub zakres: WpisyZakres,
    pub forma: Option<WpisyForma>,
    pub sposob_reprezentacji: Option<String>,
    pub dokument_id: Option<String>,
    pub wazne_do: Option<String>,
    pub id: String,
}

/// Poprawka umocowania; podnosi jego wersję, więc Kontroler sprawdza je od nowa. Data udzielenia
/// się nie zmienia, a wygaśnięcie wpisuje odnotuj_wygasniecie_umocowania.
pub fn zmien_dane_umocowania(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienDaneUmocowaniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_umocowania_wpisy
SET rodzaj = :rodzaj,
  mocodawca_uczestnik_id = :mocodawca_uczestnik_id,
  umocowany_uczestnik_id = :umocowany_uczestnik_id,
  umocowane_biuro = :umocowane_biuro,
  zakres = :zakres,
  forma = :forma,
  sposob_reprezentacji = :sposob_reprezentacji,
  dokument_id = :dokument_id,
  wazne_do = :wazne_do
WHERE id = :id"#,
                parameters: Vec::from([
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    (
                        "mocodawca_uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.mocodawca_uczestnik_id,
                        ),
                    ),
                    (
                        "umocowany_uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.umocowany_uczestnik_id,
                        ),
                    ),
                    (
                        "umocowane_biuro",
                        xmeta_room::ToSqlValue::to_sql_value(&params.umocowane_biuro),
                    ),
                    ("zakres", xmeta_room::ToSqlValue::to_sql_value(&params.zakres)),
                    ("forma", xmeta_room::ToSqlValue::to_sql_value(&params.forma)),
                    (
                        "sposob_reprezentacji",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.sposob_reprezentacji,
                        ),
                    ),
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                    ("wazne_do", xmeta_room::ToSqlValue::to_sql_value(&params.wazne_do)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_wygasniecie_umocowania`.
#[derive(Debug, Clone, PartialEq)]
pub struct OdnotujWygasniecieUmocowaniaParams {
    pub wygaslo: String,
    pub wygasniecie_powod: WpisyWygasnieciePowod,
    pub wygasniecie_zrodlo: String,
    pub id: String,
}

/// Odwołanie albo wygaśnięcie umocowania, z datą i źródłem wiadomości; wpisuje się je raz.
pub fn odnotuj_wygasniecie_umocowania(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWygasniecieUmocowaniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_umocowania_wpisy
SET wygaslo = :wygaslo,
  wygasniecie_powod = :wygasniecie_powod,
  wygasniecie_zrodlo = :wygasniecie_zrodlo
WHERE id = :id"#,
                parameters: Vec::from([
                    ("wygaslo", xmeta_room::ToSqlValue::to_sql_value(&params.wygaslo)),
                    (
                        "wygasniecie_powod",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wygasniecie_powod),
                    ),
                    (
                        "wygasniecie_zrodlo",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wygasniecie_zrodlo),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
