//! The kind `rejestr-kontroli-klienci` of the room `registers/rejestr_kontroli`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "rejestr-kontroli-klienci";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Klienci biura z Kontrolerem, ich stosunki gospodarcze i kwalifikacja każdej sprawy: czy jest
-- objęta ustawą o przeciwdziałaniu praniu pieniędzy. Wpis stosunków powstaje przy weryfikacji
-- przed pierwszą umową; klient, który wraca po zakończeniu stosunków, dostaje nowy wpis, a stary
-- zostaje razem z jego terminem przechowywania.
--
-- Pisze Kierownik kontroli z raportów Kontrolerów. Wpisu się nie zmienia; jedyne zmiany to
-- jednorazowe uzupełnienie odnośnika do kartoteki, dnia nawiązania i dnia zakończenia stosunków.
-- Kwalifikacji sprawy się nie zmienia: nowa kwalifikacja tej samej teczki to nowy wiersz.
CREATE TABLE rejestr_kontroli_klienci_wpisy (
  id TEXT PRIMARY KEY,
  -- Imię i nazwisko albo nazwa klienta; dane identyfikacyjne są w rodzaju sprawdzenia.
  klient TEXT NOT NULL CHECK (klient <> ''),
  -- Wpis w kartotece klientów (klienci_kartoteka_wpisy). Kartoteka dostaje klienta dopiero
  -- w dniu podpisania umowy, a rejestr przy weryfikacji przed nią, więc odnośnik przychodzi później.
  kartoteka_odnosnik TEXT,
  -- Kontroler, który sprawdza klienta (R7).
  kontroler TEXT NOT NULL CHECK (kontroler <> ''),
  -- Dzień podpisania pierwszej umowy z biurem i dzień zakończenia stosunków z raportu końcowego
  -- Kontrolera; od niego program liczy 5 lat przechowywania (art. 49 ust. 1 AML).
  stosunki_od TEXT,
  stosunki_do TEXT,
  -- Raport Kontrolera, z którego wpis pochodzi.
  zrodlo TEXT NOT NULL CHECK (zrodlo <> ''),
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,

  CONSTRAINT stosunki_konczy_sie_po_nawiazaniu CHECK (
    stosunki_do IS NULL OR (stosunki_od IS NOT NULL AND stosunki_do >= stosunki_od)
  )
) STRICT;

-- Kwalifikacja sprawy klienta. Przy samej umowie o zarządzanie wynik brzmi „do rozstrzygnięcia”,
-- dopóki biuro tego nie ustali; baza tego nie wymusza.
CREATE TABLE rejestr_kontroli_klienci_sprawy (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_klienci_sprawy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  klient_id TEXT NOT NULL REFERENCES rejestr_kontroli_klienci_wpisy (id),
  -- Teczka klienta; odnośnik nie daje do niej dostępu.
  teczka_odnosnik TEXT NOT NULL CHECK (teczka_odnosnik <> ''),
  rodzaj_sprawy TEXT NOT NULL CHECK (rodzaj_sprawy IN ('sprzedaz_albo_kupno', 'posrednictwo_najmu', 'zarzadzanie')),
  objeta TEXT NOT NULL CHECK (objeta IN ('tak', 'nie', 'do_rozstrzygniecia')),
  -- Dlaczego tak, np. „art. 2 ust. 1 pkt 18 AML, czynsz poniżej progu”.
  podstawa TEXT NOT NULL CHECK (podstawa <> ''),
  -- Czynsz miesięczny w groszach, średni kurs euro NBP w złotych i dzień kursu (art. 5 AML).
  czynsz INTEGER CHECK (czynsz > 0),
  kurs_euro REAL CHECK (kurs_euro > 0),
  dzien_kursu TEXT,
  zakwalifikowano TEXT NOT NULL,
  wpisal TEXT NOT NULL,

  CONSTRAINT czynsz_z_kursem_i_dniem CHECK (
    (czynsz IS NULL) = (kurs_euro IS NULL)
    AND (czynsz IS NULL) = (dzien_kursu IS NULL)
  ),
  CONSTRAINT najem_ma_czynsz CHECK (rodzaj_sprawy <> 'posrednictwo_najmu' OR czynsz IS NOT NULL),
  -- Art. 2 ust. 1 pkt 18 AML: pośrednictwo przy najmie z czynszem niższym niż równowartość
  -- 10 000 euro nie jest objęte ustawą, a z wyższym jest. 10 000 euro to kurs × 1 000 000 groszy.
  CONSTRAINT prog_najmu CHECK (
    rodzaj_sprawy <> 'posrednictwo_najmu'
    OR objeta = CASE
      WHEN czynsz < kurs_euro * 1000000 THEN 'nie'
      ELSE 'tak'
    END
  )
) STRICT;

-- Klient, który wraca, nawiązuje stosunki na nowo, ale dopiero po zakończeniu poprzednich.
CREATE TRIGGER rejestr_kontroli_klienci_jedne_trwajace_stosunki
BEFORE INSERT ON rejestr_kontroli_klienci_wpisy
WHEN NEW.kartoteka_odnosnik IS NOT NULL
  AND EXISTS (
    SELECT 1
    FROM rejestr_kontroli_klienci_wpisy
    WHERE kartoteka_odnosnik = NEW.kartoteka_odnosnik
      AND stosunki_do IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'klient ma już trwające stosunki; nowe stosunki wpisuje się po zakończeniu poprzednich');
END;

CREATE TRIGGER rejestr_kontroli_klienci_jedne_trwajace_stosunki_przy_odnosniku
BEFORE UPDATE OF kartoteka_odnosnik ON rejestr_kontroli_klienci_wpisy
WHEN NEW.kartoteka_odnosnik IS NOT NULL
  AND EXISTS (
    SELECT 1
    FROM rejestr_kontroli_klienci_wpisy
    WHERE kartoteka_odnosnik = NEW.kartoteka_odnosnik
      AND id <> NEW.id
      AND stosunki_do IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'klient ma już trwające stosunki; nowe stosunki wpisuje się po zakończeniu poprzednich');
END;

-- Organ kontroli ma zobaczyć, co biuro wiedziało w danym dniu, więc wpisu się nie poprawia.
-- Odnośnik do kartoteki i daty stosunków wpisuje się raz, gdy są znane.
CREATE TRIGGER rejestr_kontroli_klienci_wpisy_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_klienci_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.klient IS NOT OLD.klient
  OR NEW.kontroler IS NOT OLD.kontroler
  OR NEW.zrodlo IS NOT OLD.zrodlo
  OR NEW.wpisal IS NOT OLD.wpisal
  OR NEW.wpisano IS NOT OLD.wpisano
  OR (OLD.kartoteka_odnosnik IS NOT NULL AND NEW.kartoteka_odnosnik IS NOT OLD.kartoteka_odnosnik)
  OR (OLD.stosunki_od IS NOT NULL AND NEW.stosunki_od IS NOT OLD.stosunki_od)
  OR (OLD.stosunki_do IS NOT NULL AND NEW.stosunki_do IS NOT OLD.stosunki_do)
BEGIN
  SELECT RAISE(ABORT, 'wpisu klienta się nie zmienia; odnośnik do kartoteki i daty stosunków wpisuje się raz');
END;

CREATE TRIGGER rejestr_kontroli_klienci_sprawy_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_klienci_sprawy
BEGIN
  SELECT RAISE(ABORT, 'kwalifikacji sprawy się nie zmienia; nowa kwalifikacja to nowy wiersz');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabel z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER rejestr_kontroli_klienci_wpisy_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_klienci_wpisy
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_klienci_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_klienci_sprawy_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_klienci_sprawy
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_klienci_sprawy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

-- Art. 49 AML i R9: dokumentację przechowuje się co najmniej 5 lat, a na razie bez terminu.
CREATE TRIGGER rejestr_kontroli_klienci_wpisy_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_klienci_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_klienci_sprawy_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_klienci_sprawy
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;
"#,
];

/// The values of column `sprawy.rodzaj_sprawy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SprawyRodzajSprawy {
    SprzedazAlboKupno,
    PosrednictwoNajmu,
    Zarzadzanie,
}

impl SprawyRodzajSprawy {
    /// Every value, in the order of the catalog.
    pub const ALL: &[SprawyRodzajSprawy] = &[
        SprawyRodzajSprawy::SprzedazAlboKupno,
        SprawyRodzajSprawy::PosrednictwoNajmu,
        SprawyRodzajSprawy::Zarzadzanie,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            SprawyRodzajSprawy::SprzedazAlboKupno => "sprzedaz_albo_kupno",
            SprawyRodzajSprawy::PosrednictwoNajmu => "posrednictwo_najmu",
            SprawyRodzajSprawy::Zarzadzanie => "zarzadzanie",
        }
    }
}

impl xmeta_room::FromSqlValue for SprawyRodzajSprawy {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "sprzedaz_albo_kupno" => Ok(SprawyRodzajSprawy::SprzedazAlboKupno),
                    "posrednictwo_najmu" => Ok(SprawyRodzajSprawy::PosrednictwoNajmu),
                    "zarzadzanie" => Ok(SprawyRodzajSprawy::Zarzadzanie),
                    other => Err(format!("{other} is no value of SprawyRodzajSprawy")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for SprawyRodzajSprawy {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `sprawy.objeta`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SprawyObjeta {
    Tak,
    Nie,
    DoRozstrzygniecia,
}

impl SprawyObjeta {
    /// Every value, in the order of the catalog.
    pub const ALL: &[SprawyObjeta] = &[
        SprawyObjeta::Tak,
        SprawyObjeta::Nie,
        SprawyObjeta::DoRozstrzygniecia,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            SprawyObjeta::Tak => "tak",
            SprawyObjeta::Nie => "nie",
            SprawyObjeta::DoRozstrzygniecia => "do_rozstrzygniecia",
        }
    }
}

impl xmeta_room::FromSqlValue for SprawyObjeta {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "tak" => Ok(SprawyObjeta::Tak),
                    "nie" => Ok(SprawyObjeta::Nie),
                    "do_rozstrzygniecia" => Ok(SprawyObjeta::DoRozstrzygniecia),
                    other => Err(format!("{other} is no value of SprawyObjeta")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for SprawyObjeta {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The parameters of `sprawy_klienta_kontroli`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SprawyKlientaKontroliParams {
    pub klient_id: String,
}

/// A row of `sprawy_klienta_kontroli`.
#[derive(Debug, Clone, PartialEq)]
pub struct SprawyKlientaKontroliRow {
    pub id: i64,
    pub klient_id: String,
    pub teczka_odnosnik: String,
    pub rodzaj_sprawy: SprawyRodzajSprawy,
    pub objeta: SprawyObjeta,
    pub podstawa: String,
    pub czynsz: Option<i64>,
    pub kurs_euro: Option<f64>,
    pub dzien_kursu: Option<String>,
    pub zakwalifikowano: String,
    pub wpisal: String,
}

impl SprawyKlientaKontroliRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(SprawyKlientaKontroliRow {
            id: row.get("id")?,
            klient_id: row.get("klient_id")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            rodzaj_sprawy: row.get("rodzaj_sprawy")?,
            objeta: row.get("objeta")?,
            podstawa: row.get("podstawa")?,
            czynsz: row.get("czynsz")?,
            kurs_euro: row.get("kurs_euro")?,
            dzien_kursu: row.get("dzien_kursu")?,
            zakwalifikowano: row.get("zakwalifikowano")?,
            wpisal: row.get("wpisal")?,
        })
    }
}

/// Kwalifikacje spraw klienta, najnowsze pierwsze; bieżąca kwalifikacja teczki to najnowsza.
pub fn sprawy_klienta_kontroli(
    room: &impl xmeta_room::MetaRoom,
    params: &SprawyKlientaKontroliParams,
) -> Result<Vec<SprawyKlientaKontroliRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_klienci_sprawy
WHERE klient_id = :klient_id
ORDER BY id DESC"#,
        parameters: Vec::from([
            ("klient_id", xmeta_room::ToSqlValue::to_sql_value(&params.klient_id)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(SprawyKlientaKontroliRow::from_row).collect()
}

/// The parameters of `zakwalifikuj_sprawe`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZakwalifikujSpraweParams {
    pub klient_id: String,
    pub teczka_odnosnik: String,
    pub rodzaj_sprawy: SprawyRodzajSprawy,
    pub objeta: SprawyObjeta,
    pub podstawa: String,
    pub czynsz: Option<i64>,
    pub kurs_euro: Option<f64>,
    pub dzien_kursu: Option<String>,
    pub zakwalifikowano: String,
    pub wpisal: String,
}

/// Czy sprawa jest objęta ustawą, z podstawą; przy pośrednictwie najmu z czynszem, kursem euro
/// i dniem kursu.
pub fn zakwalifikuj_sprawe(
    room: &impl xmeta_room::MetaRoom,
    params: &ZakwalifikujSpraweParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_klienci_sprawy (klient_id, teczka_odnosnik, rodzaj_sprawy, objeta,
  podstawa, czynsz, kurs_euro, dzien_kursu, zakwalifikowano, wpisal)
VALUES (:klient_id, :teczka_odnosnik, :rodzaj_sprawy, :objeta, :podstawa, :czynsz, :kurs_euro,
  :dzien_kursu, :zakwalifikowano, :wpisal)"#,
                parameters: Vec::from([
                    (
                        "klient_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_id),
                    ),
                    (
                        "teczka_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.teczka_odnosnik),
                    ),
                    (
                        "rodzaj_sprawy",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj_sprawy),
                    ),
                    ("objeta", xmeta_room::ToSqlValue::to_sql_value(&params.objeta)),
                    ("podstawa", xmeta_room::ToSqlValue::to_sql_value(&params.podstawa)),
                    ("czynsz", xmeta_room::ToSqlValue::to_sql_value(&params.czynsz)),
                    (
                        "kurs_euro",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kurs_euro),
                    ),
                    (
                        "dzien_kursu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dzien_kursu),
                    ),
                    (
                        "zakwalifikowano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zakwalifikowano),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                ]),
            },
        ],
    )
}

/// A row of `klienci_kontroli`.
#[derive(Debug, Clone, PartialEq)]
pub struct KlienciKontroliRow {
    pub id: String,
    pub klient: String,
    pub kartoteka_odnosnik: Option<String>,
    pub kontroler: String,
    pub stosunki_od: Option<String>,
    pub stosunki_do: Option<String>,
    pub zrodlo: String,
    pub wpisal: String,
    pub wpisano: String,
}

impl KlienciKontroliRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(KlienciKontroliRow {
            id: row.get("id")?,
            klient: row.get("klient")?,
            kartoteka_odnosnik: row.get("kartoteka_odnosnik")?,
            kontroler: row.get("kontroler")?,
            stosunki_od: row.get("stosunki_od")?,
            stosunki_do: row.get("stosunki_do")?,
            zrodlo: row.get("zrodlo")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Stosunki gospodarcze z klientami, najnowsze pierwsze; klient, który wrócił, ma kilka wpisów.
pub fn klienci_kontroli(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<KlienciKontroliRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_klienci_wpisy
ORDER BY wpisano DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(KlienciKontroliRow::from_row).collect()
}

/// The parameters of `stosunki_klienta_z_kartoteki`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StosunkiKlientaZKartotekiParams {
    pub kartoteka_odnosnik: String,
}

/// A row of `stosunki_klienta_z_kartoteki`.
#[derive(Debug, Clone, PartialEq)]
pub struct StosunkiKlientaZKartotekiRow {
    pub id: String,
    pub klient: String,
    pub kartoteka_odnosnik: Option<String>,
    pub kontroler: String,
    pub stosunki_od: Option<String>,
    pub stosunki_do: Option<String>,
    pub zrodlo: String,
    pub wpisal: String,
    pub wpisano: String,
}

impl StosunkiKlientaZKartotekiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(StosunkiKlientaZKartotekiRow {
            id: row.get("id")?,
            klient: row.get("klient")?,
            kartoteka_odnosnik: row.get("kartoteka_odnosnik")?,
            kontroler: row.get("kontroler")?,
            stosunki_od: row.get("stosunki_od")?,
            stosunki_do: row.get("stosunki_do")?,
            zrodlo: row.get("zrodlo")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Wszystkie stosunki klienta z kartoteki; Kierownik kontroli wskazuje dotychczasowego Kontrolera
/// klienta, który wraca (R7).
pub fn stosunki_klienta_z_kartoteki(
    room: &impl xmeta_room::MetaRoom,
    params: &StosunkiKlientaZKartotekiParams,
) -> Result<Vec<StosunkiKlientaZKartotekiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_klienci_wpisy
WHERE kartoteka_odnosnik = :kartoteka_odnosnik
ORDER BY wpisano, id"#,
        parameters: Vec::from([
            (
                "kartoteka_odnosnik",
                xmeta_room::ToSqlValue::to_sql_value(&params.kartoteka_odnosnik),
            ),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(StosunkiKlientaZKartotekiRow::from_row).collect()
}

/// The parameters of `wpisz_klienta_kontroli`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszKlientaKontroliParams {
    pub id: String,
    pub klient: String,
    pub kartoteka_odnosnik: Option<String>,
    pub kontroler: String,
    pub stosunki_od: Option<String>,
    pub stosunki_do: Option<String>,
    pub zrodlo: String,
    pub wpisal: String,
    pub wpisano: String,
}

/// Klient z Kontrolerem przy weryfikacji przed pierwszą umową. Identyfikator (UUID) nadaje
/// aplikacja.
pub fn wpisz_klienta_kontroli(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszKlientaKontroliParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_klienci_wpisy (id, klient, kartoteka_odnosnik, kontroler, stosunki_od,
  stosunki_do, zrodlo, wpisal, wpisano)
VALUES (:id, :klient, :kartoteka_odnosnik, :kontroler, :stosunki_od, :stosunki_do, :zrodlo, :wpisal,
  :wpisano)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("klient", xmeta_room::ToSqlValue::to_sql_value(&params.klient)),
                    (
                        "kartoteka_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kartoteka_odnosnik),
                    ),
                    (
                        "kontroler",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kontroler),
                    ),
                    (
                        "stosunki_od",
                        xmeta_room::ToSqlValue::to_sql_value(&params.stosunki_od),
                    ),
                    (
                        "stosunki_do",
                        xmeta_room::ToSqlValue::to_sql_value(&params.stosunki_do),
                    ),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
        ],
    )
}

/// The parameters of `wpisz_odnosnik_do_kartoteki`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszOdnosnikDoKartotekiParams {
    pub kartoteka_odnosnik: String,
    pub id: String,
}

/// Wpis klienta w kartotece, gdy powstał w dniu podpisania umowy.
pub fn wpisz_odnosnik_do_kartoteki(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszOdnosnikDoKartotekiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE rejestr_kontroli_klienci_wpisy
SET kartoteka_odnosnik = :kartoteka_odnosnik
WHERE id = :id"#,
                parameters: Vec::from([
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

/// The parameters of `wpisz_nawiazanie_stosunkow`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszNawiazanieStosunkowParams {
    pub stosunki_od: String,
    pub id: String,
}

/// Dzień podpisania pierwszej umowy z biurem.
pub fn wpisz_nawiazanie_stosunkow(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszNawiazanieStosunkowParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE rejestr_kontroli_klienci_wpisy
SET stosunki_od = :stosunki_od
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "stosunki_od",
                        xmeta_room::ToSqlValue::to_sql_value(&params.stosunki_od),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wpisz_zakonczenie_stosunkow`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszZakonczenieStosunkowParams {
    pub stosunki_do: String,
    pub id: String,
}

/// Dzień zakończenia stosunków z raportu końcowego Kontrolera; od niego biegnie 5 lat
/// przechowywania.
pub fn wpisz_zakonczenie_stosunkow(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszZakonczenieStosunkowParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE rejestr_kontroli_klienci_wpisy
SET stosunki_do = :stosunki_do
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "stosunki_do",
                        xmeta_room::ToSqlValue::to_sql_value(&params.stosunki_do),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
