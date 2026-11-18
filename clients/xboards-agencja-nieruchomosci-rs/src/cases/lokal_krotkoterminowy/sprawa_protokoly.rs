//! The kind `sprawa-protokoly` of the room `cases/lokal_krotkoterminowy`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "sprawa-protokoly",
    requires: &[&super::sprawa_dokumenty::KIND, &super::sprawa_nieruchomosci::KIND],
    migrations: &[
        r#"-- Protokoły zdawczo-odbiorcze nieruchomości: przejęcie i wydanie lokalu, zwrot, przekazanie
-- po akcie, przyjęcie i zwrot kluczy klienta. Jeden model służy transakcji, najmowi, lokalowi
-- krótkoterminowemu i kluczom oferty, więc porównanie wydania ze zwrotem robi program na tych
-- samych pozycjach. Typ sprawy wskazuje protokół kluczem obcym, np. etap najmu wymaga podpisanego
-- protokołu wydania.
--
-- Protokół przygotowuje agent sprawy jako projekt i dopisuje pozycje: liczniki z odczytem,
-- wyposażenie, klucze, pomieszczenia i uwagi. Podpisany protokół ma dokument z podpisami stron
-- i od podpisu nie zmienia się ani on, ani jego pozycje. Pomyłkę po podpisie poprawia nowy
-- protokół.
CREATE TABLE sprawa_protokoly_wpisy (
  id TEXT PRIMARY KEY,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('przejecie', 'wydanie', 'zwrot', 'przekazanie', 'przyjecie_kluczy', 'zwrot_kluczy')),
  nieruchomosc_id TEXT NOT NULL REFERENCES sprawa_nieruchomosci_wpisy (id),
  -- Chwila przekazania, w której spisano pozycje.
  chwila TEXT NOT NULL,
  stan TEXT NOT NULL CHECK (stan IN ('projekt', 'podpisany')),
  -- Podpisany protokół jako dokument teczki rodzaju protokol.
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),

  CONSTRAINT podpisany_ma_dokument CHECK ((stan = 'podpisany') = (dokument_id IS NOT NULL))
) STRICT;

-- Pozycje protokołu, każda z opisem stanu i zdjęciem, jeśli jest.
CREATE TABLE sprawa_protokoly_pozycje (
  -- Klucz nadany przez bazę jest dodatni; zob. sprawa_protokoly_pozycje_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  protokol_id TEXT NOT NULL REFERENCES sprawa_protokoly_wpisy (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('licznik', 'wyposazenie', 'klucze', 'pomieszczenie', 'uwaga')),
  -- Np. „licznik energii elektrycznej”, „klucz do drzwi wejściowych”, „pralka”.
  nazwa TEXT NOT NULL CHECK (nazwa <> ''),
  -- Numer licznika albo oznaczenie rzeczy, np. numer seryjny.
  numer TEXT,
  -- Odczyt licznika, liczba kluczy albo stan rzeczy, np. „12 345,6 kWh”, „3 sztuki”, „rysa na blacie”.
  opis_stanu TEXT,
  zdjecie_sciezka TEXT,
  zdjecie_skrot TEXT,

  CONSTRAINT zdjecie_ze_skrotem CHECK ((zdjecie_sciezka IS NULL) = (zdjecie_skrot IS NULL)),
  -- Licznik bez numeru i odczytu nie da się porównać przy zwrocie.
  CONSTRAINT licznik_ma_numer_i_odczyt CHECK (rodzaj <> 'licznik' OR (numer IS NOT NULL AND opis_stanu IS NOT NULL))
) STRICT;

-- Pozycje dopisuje się do projektu, a podpis sprawdza zmianę stanu, więc protokół nie może
-- powstać od razu podpisany.
CREATE TRIGGER sprawa_protokoly_powstaje_jako_projekt
BEFORE INSERT ON sprawa_protokoly_wpisy
WHEN NEW.stan <> 'projekt'
BEGIN
  SELECT RAISE(ABORT, 'protokół powstaje jako projekt');
END;

CREATE TRIGGER sprawa_protokoly_podpisany_z_dokumentem_protokolu
BEFORE UPDATE ON sprawa_protokoly_wpisy
WHEN NEW.stan = 'podpisany'
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_dokumenty_wpisy
    WHERE id = NEW.dokument_id
      AND rodzaj = 'protokol'
      AND stan IN ('otrzymany', 'sprawdzony')
  )
BEGIN
  SELECT RAISE(ABORT, 'podpisany protokół wskazuje otrzymany dokument rodzaju protokol');
END;

-- Podpis obejmuje cały protokół: rodzaj, nieruchomość, chwilę i dokument.
CREATE TRIGGER sprawa_protokoly_podpisany_niezmienny
BEFORE UPDATE ON sprawa_protokoly_wpisy
WHEN OLD.stan = 'podpisany'
BEGIN
  SELECT RAISE(ABORT, 'podpisanego protokołu się nie zmienia; pomyłkę poprawia nowy protokół');
END;

-- Dokument podpisanego protokołu jest jego podstawą: nie zmienia rodzaju ani tego, czego dotyczy,
-- i nie staje się odrzucony ani zbędny.
CREATE TRIGGER sprawa_protokoly_dokument_podpisanego_zostaje
BEFORE UPDATE ON sprawa_dokumenty_wpisy
WHEN (
    NEW.rodzaj IS NOT OLD.rodzaj
    OR NEW.dotyczy_uczestnik_id IS NOT OLD.dotyczy_uczestnik_id
    OR NEW.dotyczy_nieruchomosc_id IS NOT OLD.dotyczy_nieruchomosc_id
    OR (NEW.stan IN ('odrzucony', 'zbedny') AND NEW.stan IS NOT OLD.stan)
  )
  AND EXISTS (
    SELECT 1
    FROM sprawa_protokoly_wpisy
    WHERE stan = 'podpisany'
      AND dokument_id = OLD.id
  )
BEGIN
  SELECT RAISE(ABORT, 'dokument podpisanego protokołu nie zmienia rodzaju ani tego, czego dotyczy, i nie jest odrzucany');
END;

CREATE TRIGGER sprawa_protokoly_pozycje_tylko_w_projekcie
BEFORE INSERT ON sprawa_protokoly_pozycje
WHEN (
  SELECT stan <> 'projekt'
  FROM sprawa_protokoly_wpisy
  WHERE id = NEW.protokol_id
)
BEGIN
  SELECT RAISE(ABORT, 'pozycje dopisuje się tylko do projektu protokołu');
END;

-- W projekcie pozycję się poprawia, ale nie przenosi do innego protokołu.
CREATE TRIGGER sprawa_protokoly_pozycja_zostaje_w_protokole
BEFORE UPDATE ON sprawa_protokoly_pozycje
WHEN NEW.id IS NOT OLD.id
  OR NEW.protokol_id IS NOT OLD.protokol_id
BEGIN
  SELECT RAISE(ABORT, 'pozycja zostaje w swoim protokole');
END;

CREATE TRIGGER sprawa_protokoly_pozycje_po_podpisie_bez_zmian
BEFORE UPDATE ON sprawa_protokoly_pozycje
WHEN (
  SELECT stan <> 'projekt'
  FROM sprawa_protokoly_wpisy
  WHERE id = OLD.protokol_id
)
BEGIN
  SELECT RAISE(ABORT, 'pozycji podpisanego protokołu się nie zmienia');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_protokoly_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_protokoly_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_protokoly_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_protokoly_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_protokoly_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER sprawa_protokoly_pozycje_bez_zastepowania
BEFORE INSERT ON sprawa_protokoly_pozycje
WHEN EXISTS (
  SELECT 1
  FROM sprawa_protokoly_pozycje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_protokoly_pozycje_bez_usuwania
BEFORE DELETE ON sprawa_protokoly_pozycje
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
    ],
};

/// The values of column `pozycje.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PozycjeRodzaj {
    Licznik,
    Wyposazenie,
    Klucze,
    Pomieszczenie,
    Uwaga,
}

impl PozycjeRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[PozycjeRodzaj] = &[
        PozycjeRodzaj::Licznik,
        PozycjeRodzaj::Wyposazenie,
        PozycjeRodzaj::Klucze,
        PozycjeRodzaj::Pomieszczenie,
        PozycjeRodzaj::Uwaga,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            PozycjeRodzaj::Licznik => "licznik",
            PozycjeRodzaj::Wyposazenie => "wyposazenie",
            PozycjeRodzaj::Klucze => "klucze",
            PozycjeRodzaj::Pomieszczenie => "pomieszczenie",
            PozycjeRodzaj::Uwaga => "uwaga",
        }
    }
}

impl xmeta_room::FromSqlValue for PozycjeRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "licznik" => Ok(PozycjeRodzaj::Licznik),
                    "wyposazenie" => Ok(PozycjeRodzaj::Wyposazenie),
                    "klucze" => Ok(PozycjeRodzaj::Klucze),
                    "pomieszczenie" => Ok(PozycjeRodzaj::Pomieszczenie),
                    "uwaga" => Ok(PozycjeRodzaj::Uwaga),
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

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    Przejecie,
    Wydanie,
    Zwrot,
    Przekazanie,
    PrzyjecieKluczy,
    ZwrotKluczy,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::Przejecie,
        WpisyRodzaj::Wydanie,
        WpisyRodzaj::Zwrot,
        WpisyRodzaj::Przekazanie,
        WpisyRodzaj::PrzyjecieKluczy,
        WpisyRodzaj::ZwrotKluczy,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::Przejecie => "przejecie",
            WpisyRodzaj::Wydanie => "wydanie",
            WpisyRodzaj::Zwrot => "zwrot",
            WpisyRodzaj::Przekazanie => "przekazanie",
            WpisyRodzaj::PrzyjecieKluczy => "przyjecie_kluczy",
            WpisyRodzaj::ZwrotKluczy => "zwrot_kluczy",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "przejecie" => Ok(WpisyRodzaj::Przejecie),
                    "wydanie" => Ok(WpisyRodzaj::Wydanie),
                    "zwrot" => Ok(WpisyRodzaj::Zwrot),
                    "przekazanie" => Ok(WpisyRodzaj::Przekazanie),
                    "przyjecie_kluczy" => Ok(WpisyRodzaj::PrzyjecieKluczy),
                    "zwrot_kluczy" => Ok(WpisyRodzaj::ZwrotKluczy),
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

/// The values of column `wpisy.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyStan {
    Projekt,
    Podpisany,
}

impl WpisyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyStan] = &[WpisyStan::Projekt, WpisyStan::Podpisany];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyStan::Projekt => "projekt",
            WpisyStan::Podpisany => "podpisany",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "projekt" => Ok(WpisyStan::Projekt),
                    "podpisany" => Ok(WpisyStan::Podpisany),
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

/// A row of `pozycje_protokolow`.
#[derive(Debug, Clone, PartialEq)]
pub struct PozycjeProtokolowRow {
    pub id: i64,
    pub protokol_id: String,
    pub rodzaj: PozycjeRodzaj,
    pub nazwa: String,
    pub numer: Option<String>,
    pub opis_stanu: Option<String>,
    pub zdjecie_sciezka: Option<String>,
    pub zdjecie_skrot: Option<String>,
}

impl PozycjeProtokolowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PozycjeProtokolowRow {
            id: row.get("id")?,
            protokol_id: row.get("protokol_id")?,
            rodzaj: row.get("rodzaj")?,
            nazwa: row.get("nazwa")?,
            numer: row.get("numer")?,
            opis_stanu: row.get("opis_stanu")?,
            zdjecie_sciezka: row.get("zdjecie_sciezka")?,
            zdjecie_skrot: row.get("zdjecie_skrot")?,
        })
    }
}

/// Pozycje wszystkich protokołów teczki w kolejności wpisania.
pub fn pozycje_protokolow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PozycjeProtokolowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_protokoly_pozycje
ORDER BY protokol_id, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(PozycjeProtokolowRow::from_row).collect()
}

/// The parameters of `dopisz_pozycje_protokolu`.
#[derive(Debug, Clone, PartialEq)]
pub struct DopiszPozycjeProtokoluParams {
    pub protokol_id: String,
    pub rodzaj: PozycjeRodzaj,
    pub nazwa: String,
    pub numer: Option<String>,
    pub opis_stanu: Option<String>,
    pub zdjecie_sciezka: Option<String>,
    pub zdjecie_skrot: Option<String>,
}

/// Licznik, rzecz, klucze, pomieszczenie albo uwaga w projekcie protokołu.
pub fn dopisz_pozycje_protokolu(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszPozycjeProtokoluParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_protokoly_pozycje (protokol_id, rodzaj, nazwa, numer, opis_stanu, zdjecie_sciezka,
  zdjecie_skrot)
VALUES (:protokol_id, :rodzaj, :nazwa, :numer, :opis_stanu, :zdjecie_sciezka, :zdjecie_skrot)"#,
                parameters: Vec::from([
                    (
                        "protokol_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.protokol_id),
                    ),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("nazwa", xmeta_room::ToSqlValue::to_sql_value(&params.nazwa)),
                    ("numer", xmeta_room::ToSqlValue::to_sql_value(&params.numer)),
                    (
                        "opis_stanu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.opis_stanu),
                    ),
                    (
                        "zdjecie_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zdjecie_sciezka),
                    ),
                    (
                        "zdjecie_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zdjecie_skrot),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `popraw_pozycje_protokolu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PoprawPozycjeProtokoluParams {
    pub nazwa: String,
    pub numer: Option<String>,
    pub opis_stanu: Option<String>,
    pub zdjecie_sciezka: Option<String>,
    pub zdjecie_skrot: Option<String>,
    pub id: i64,
}

/// Poprawka pozycji, dopóki protokół jest projektem.
pub fn popraw_pozycje_protokolu(
    room: &impl xmeta_room::MetaRoom,
    params: &PoprawPozycjeProtokoluParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_protokoly_pozycje
SET nazwa = :nazwa,
  numer = :numer,
  opis_stanu = :opis_stanu,
  zdjecie_sciezka = :zdjecie_sciezka,
  zdjecie_skrot = :zdjecie_skrot
WHERE id = :id"#,
                parameters: Vec::from([
                    ("nazwa", xmeta_room::ToSqlValue::to_sql_value(&params.nazwa)),
                    ("numer", xmeta_room::ToSqlValue::to_sql_value(&params.numer)),
                    (
                        "opis_stanu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.opis_stanu),
                    ),
                    (
                        "zdjecie_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zdjecie_sciezka),
                    ),
                    (
                        "zdjecie_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zdjecie_skrot),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `protokoly`.
#[derive(Debug, Clone, PartialEq)]
pub struct ProtokolyRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub nieruchomosc_id: String,
    pub chwila: String,
    pub stan: WpisyStan,
    pub dokument_id: Option<String>,
}

impl ProtokolyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ProtokolyRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            nieruchomosc_id: row.get("nieruchomosc_id")?,
            chwila: row.get("chwila")?,
            stan: row.get("stan")?,
            dokument_id: row.get("dokument_id")?,
        })
    }
}

/// Protokoły teczki w kolejności chwili przekazania.
pub fn protokoly(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ProtokolyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_protokoly_wpisy
ORDER BY chwila, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ProtokolyRow::from_row).collect()
}

/// The parameters of `przygotuj_protokol`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzygotujProtokolParams {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub nieruchomosc_id: String,
    pub chwila: String,
}

/// Projekt protokołu przekazania nieruchomości; identyfikator (UUID) nadaje aplikacja, bo wskazują
/// go pozycje i wiersze typu sprawy.
pub fn przygotuj_protokol(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzygotujProtokolParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_protokoly_wpisy (id, rodzaj, nieruchomosc_id, chwila, stan)
VALUES (:id, :rodzaj, :nieruchomosc_id, :chwila, 'projekt')"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    (
                        "nieruchomosc_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.nieruchomosc_id),
                    ),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                ]),
            },
        ],
    )
}

/// The parameters of `podpisz_protokol`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PodpiszProtokolParams {
    pub dokument_id: String,
    pub id: String,
}

/// Protokół podpisany przez strony; dokument to otrzymany plik protokołu z podpisami.
pub fn podpisz_protokol(
    room: &impl xmeta_room::MetaRoom,
    params: &PodpiszProtokolParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_protokoly_wpisy
SET stan = 'podpisany',
  dokument_id = :dokument_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
