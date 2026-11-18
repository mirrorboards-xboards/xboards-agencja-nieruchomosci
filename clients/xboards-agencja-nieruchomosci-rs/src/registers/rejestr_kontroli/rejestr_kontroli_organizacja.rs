//! The kind `rejestr-kontroli-organizacja` of the room `registers/rejestr_kontroli`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "rejestr-kontroli-organizacja",
    requires: &[&super::rejestr_kontroli_decyzje::KIND],
    migrations: &[
        r#"-- Organizacja Kontroli w biurze: kolejne wersje procedury wewnętrznej z akceptacją właściciela
-- biura (art. 50 AML), wyznaczenia osób z art. 6 i 8 AML i szkolenia (art. 52 AML). Biuro robi to
-- raz dla całej działalności, więc rejestr jest jeden na biuro.
--
-- Wersję procedury Kierownik kontroli wpisuje po akceptacji, a wprowadza się ją nie wcześniej niż
-- w dniu akceptacji. Osoba z art. 8 AML jest w danej chwili jedna; wyznaczenie kończy się raz.
-- Szkolenie Kierownik wpisuje po potwierdzeniu udziału przez człowieka.
CREATE TABLE rejestr_kontroli_organizacja_procedury (
  id TEXT PRIMARY KEY,
  -- Kolejne numery od 1.
  wersja INTEGER NOT NULL UNIQUE CHECK (wersja >= 1),
  dokument_sciezka TEXT NOT NULL CHECK (dokument_sciezka <> ''),
  dokument_skrot TEXT NOT NULL CHECK (dokument_skrot <> ''),
  -- Akceptacja właściciela biura przed wprowadzeniem (art. 50 ust. 3 AML).
  akceptacja_decyzja_id TEXT NOT NULL REFERENCES rejestr_kontroli_decyzje_wpisy (id),
  -- Dzień wprowadzenia.
  wprowadzono TEXT NOT NULL,
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL
) STRICT;

CREATE TABLE rejestr_kontroli_organizacja_wyznaczenia (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_organizacja_wyznaczenia_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Kadra kierownicza odpowiedzialna za obowiązki z ustawy (art. 6 AML) albo osoba odpowiedzialna
  -- za zgodność i zawiadomienia (art. 8 AML).
  funkcja TEXT NOT NULL CHECK (funkcja IN ('art6', 'art8')),
  osoba TEXT NOT NULL CHECK (osoba <> ''),
  od TEXT NOT NULL,
  do TEXT,
  -- Dokument wyznaczenia, np. uchwała albo pismo właściciela.
  podstawa TEXT,
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,

  CONSTRAINT koniec_nie_przed_poczatkiem CHECK (do IS NULL OR do >= od)
) STRICT;

CREATE TABLE rejestr_kontroli_organizacja_szkolenia (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_organizacja_szkolenia_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  uczestnik TEXT NOT NULL CHECK (uczestnik <> ''),
  odbyto TEXT NOT NULL,
  -- Z czego, np. wersja standardu Kontroli zatwierdzona przez właściciela biura.
  temat TEXT NOT NULL CHECK (temat <> ''),
  -- Człowiek, który potwierdził udział.
  potwierdzil TEXT NOT NULL CHECK (potwierdzil <> ''),
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL
) STRICT;

CREATE TRIGGER rejestr_kontroli_organizacja_procedury_wersja_kolejna
BEFORE INSERT ON rejestr_kontroli_organizacja_procedury
WHEN NEW.wersja IS NOT (
  SELECT coalesce(max(wersja), 0) + 1
  FROM rejestr_kontroli_organizacja_procedury
)
BEGIN
  SELECT RAISE(ABORT, 'wersja procedury ma kolejny numer');
END;

CREATE TRIGGER rejestr_kontroli_organizacja_procedury_akceptacja_wskazuje_decyzje
BEFORE INSERT ON rejestr_kontroli_organizacja_procedury
WHEN NOT EXISTS (
  SELECT 1
  FROM rejestr_kontroli_decyzje_zatwierdzone
  WHERE id = NEW.akceptacja_decyzja_id
    AND rodzaj = 'akceptacja_procedury'
    AND dotyczy_tabela = 'rejestr_kontroli_organizacja_procedury'
    AND dotyczy_id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'procedura wskazuje zatwierdzoną decyzję akceptacja_procedury o tej wersji');
END;

CREATE TRIGGER rejestr_kontroli_organizacja_procedury_wprowadzona_po_akceptacji
BEFORE INSERT ON rejestr_kontroli_organizacja_procedury
WHEN NEW.wprowadzono < (
  SELECT date(podpisano)
  FROM rejestr_kontroli_decyzje_wpisy
  WHERE id = NEW.akceptacja_decyzja_id
)
BEGIN
  SELECT RAISE(ABORT, 'procedurę wprowadza się nie wcześniej niż w dniu akceptacji');
END;

CREATE TRIGGER rejestr_kontroli_organizacja_procedury_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_organizacja_procedury
BEGIN
  SELECT RAISE(ABORT, 'wersji procedury się nie zmienia; zmiana procedury to nowa wersja');
END;

CREATE TRIGGER rejestr_kontroli_organizacja_wyznaczenia_powstaje_biezace
BEFORE INSERT ON rejestr_kontroli_organizacja_wyznaczenia
WHEN NEW.do IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'wyznaczenie powstaje jako bieżące');
END;

-- Osoba z art. 8 AML jest jedna; następcę wpisuje się po zakończeniu wyznaczenia poprzednika.
CREATE TRIGGER rejestr_kontroli_organizacja_wyznaczenia_jedna_osoba_art8
BEFORE INSERT ON rejestr_kontroli_organizacja_wyznaczenia
WHEN NEW.funkcja = 'art8'
  AND EXISTS (
    SELECT 1
    FROM rejestr_kontroli_organizacja_wyznaczenia
    WHERE funkcja = 'art8'
      AND do IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'osoba z art. 8 AML jest jedna; następcę wpisuje się po zakończeniu wyznaczenia poprzednika');
END;

CREATE TRIGGER rejestr_kontroli_organizacja_wyznaczenia_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_organizacja_wyznaczenia
WHEN NEW.id IS NOT OLD.id
  OR NEW.funkcja IS NOT OLD.funkcja
  OR NEW.osoba IS NOT OLD.osoba
  OR NEW.od IS NOT OLD.od
  OR NEW.podstawa IS NOT OLD.podstawa
  OR NEW.wpisal IS NOT OLD.wpisal
  OR NEW.wpisano IS NOT OLD.wpisano
  OR (OLD.do IS NOT NULL AND NEW.do IS NOT OLD.do)
BEGIN
  SELECT RAISE(ABORT, 'wyznaczenia się nie zmienia; może się tylko raz zakończyć');
END;

CREATE TRIGGER rejestr_kontroli_organizacja_szkolenia_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_organizacja_szkolenia
BEGIN
  SELECT RAISE(ABORT, 'wpisu szkolenia się nie zmienia; błąd poprawia nowy wpis');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabel z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER rejestr_kontroli_organizacja_procedury_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_organizacja_procedury
WHEN EXISTS (
    SELECT 1
    FROM rejestr_kontroli_organizacja_procedury
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM rejestr_kontroli_organizacja_procedury
    WHERE wersja = NEW.wersja
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_organizacja_wyznaczenia_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_organizacja_wyznaczenia
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_organizacja_wyznaczenia
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_organizacja_szkolenia_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_organizacja_szkolenia
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_organizacja_szkolenia
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

-- Art. 49 AML i R9: dokumentację przechowuje się co najmniej 5 lat, a na razie bez terminu.
CREATE TRIGGER rejestr_kontroli_organizacja_procedury_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_organizacja_procedury
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_organizacja_wyznaczenia_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_organizacja_wyznaczenia
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_organizacja_szkolenia_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_organizacja_szkolenia
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;
"#,
    ],
};

/// The values of column `wyznaczenia.funkcja`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WyznaczeniaFunkcja {
    Art6,
    Art8,
}

impl WyznaczeniaFunkcja {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WyznaczeniaFunkcja] = &[
        WyznaczeniaFunkcja::Art6,
        WyznaczeniaFunkcja::Art8,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WyznaczeniaFunkcja::Art6 => "art6",
            WyznaczeniaFunkcja::Art8 => "art8",
        }
    }
}

impl xmeta_room::FromSqlValue for WyznaczeniaFunkcja {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "art6" => Ok(WyznaczeniaFunkcja::Art6),
                    "art8" => Ok(WyznaczeniaFunkcja::Art8),
                    other => Err(format!("{other} is no value of WyznaczeniaFunkcja")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WyznaczeniaFunkcja {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `procedury`.
#[derive(Debug, Clone, PartialEq)]
pub struct ProceduryRow {
    pub id: String,
    pub wersja: i64,
    pub dokument_sciezka: String,
    pub dokument_skrot: String,
    pub akceptacja_decyzja_id: String,
    pub wprowadzono: String,
    pub wpisal: String,
    pub wpisano: String,
}

impl ProceduryRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ProceduryRow {
            id: row.get("id")?,
            wersja: row.get("wersja")?,
            dokument_sciezka: row.get("dokument_sciezka")?,
            dokument_skrot: row.get("dokument_skrot")?,
            akceptacja_decyzja_id: row.get("akceptacja_decyzja_id")?,
            wprowadzono: row.get("wprowadzono")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Wersje procedury wewnętrznej, najnowsza pierwsza.
pub fn procedury(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ProceduryRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_organizacja_procedury
ORDER BY wersja DESC"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ProceduryRow::from_row).collect()
}

/// The parameters of `wpisz_procedure`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszProcedureParams {
    pub id: String,
    pub wersja: i64,
    pub dokument_sciezka: String,
    pub dokument_skrot: String,
    pub akceptacja_decyzja_id: String,
    pub wprowadzono: String,
    pub wpisal: String,
    pub wpisano: String,
}

/// Nowa wersja procedury po akceptacji właściciela biura. Identyfikator (UUID) nadaje aplikacja,
/// zanim powstanie prośba o akceptację.
pub fn wpisz_procedure(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszProcedureParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_organizacja_procedury (id, wersja, dokument_sciezka, dokument_skrot,
  akceptacja_decyzja_id, wprowadzono, wpisal, wpisano)
VALUES (:id, :wersja, :dokument_sciezka, :dokument_skrot, :akceptacja_decyzja_id, :wprowadzono,
  :wpisal, :wpisano)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("wersja", xmeta_room::ToSqlValue::to_sql_value(&params.wersja)),
                    (
                        "dokument_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_sciezka),
                    ),
                    (
                        "dokument_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_skrot),
                    ),
                    (
                        "akceptacja_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.akceptacja_decyzja_id,
                        ),
                    ),
                    (
                        "wprowadzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wprowadzono),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
        ],
    )
}

/// A row of `szkolenia`.
#[derive(Debug, Clone, PartialEq)]
pub struct SzkoleniaRow {
    pub id: i64,
    pub uczestnik: String,
    pub odbyto: String,
    pub temat: String,
    pub potwierdzil: String,
    pub wpisal: String,
    pub wpisano: String,
}

impl SzkoleniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(SzkoleniaRow {
            id: row.get("id")?,
            uczestnik: row.get("uczestnik")?,
            odbyto: row.get("odbyto")?,
            temat: row.get("temat")?,
            potwierdzil: row.get("potwierdzil")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Szkolenia, najnowsze pierwsze.
pub fn szkolenia(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<SzkoleniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_organizacja_szkolenia
ORDER BY odbyto DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(SzkoleniaRow::from_row).collect()
}

/// The parameters of `wpisz_szkolenie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszSzkolenieParams {
    pub uczestnik: String,
    pub odbyto: String,
    pub temat: String,
    pub potwierdzil: String,
    pub wpisal: String,
    pub wpisano: String,
}

/// Szkolenie po potwierdzeniu udziału przez człowieka.
pub fn wpisz_szkolenie(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszSzkolenieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_organizacja_szkolenia (uczestnik, odbyto, temat, potwierdzil, wpisal,
  wpisano)
VALUES (:uczestnik, :odbyto, :temat, :potwierdzil, :wpisal, :wpisano)"#,
                parameters: Vec::from([
                    (
                        "uczestnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.uczestnik),
                    ),
                    ("odbyto", xmeta_room::ToSqlValue::to_sql_value(&params.odbyto)),
                    ("temat", xmeta_room::ToSqlValue::to_sql_value(&params.temat)),
                    (
                        "potwierdzil",
                        xmeta_room::ToSqlValue::to_sql_value(&params.potwierdzil),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
        ],
    )
}

/// A row of `wyznaczenia`.
#[derive(Debug, Clone, PartialEq)]
pub struct WyznaczeniaRow {
    pub id: i64,
    pub funkcja: WyznaczeniaFunkcja,
    pub osoba: String,
    pub od: String,
    pub r#do: Option<String>,
    pub podstawa: Option<String>,
    pub wpisal: String,
    pub wpisano: String,
}

impl WyznaczeniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WyznaczeniaRow {
            id: row.get("id")?,
            funkcja: row.get("funkcja")?,
            osoba: row.get("osoba")?,
            od: row.get("od")?,
            r#do: row.get("do")?,
            podstawa: row.get("podstawa")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Wyznaczenia osób z art. 6 i 8 AML, bieżące i zakończone.
pub fn wyznaczenia(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WyznaczeniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_organizacja_wyznaczenia
ORDER BY funkcja, od DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(WyznaczeniaRow::from_row).collect()
}

/// The parameters of `wpisz_wyznaczenie`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszWyznaczenieParams {
    pub funkcja: WyznaczeniaFunkcja,
    pub osoba: String,
    pub od: String,
    pub podstawa: Option<String>,
    pub wpisal: String,
    pub wpisano: String,
}

/// Osoba wyznaczona do funkcji od dnia.
pub fn wpisz_wyznaczenie(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszWyznaczenieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_organizacja_wyznaczenia (funkcja, osoba, od, podstawa, wpisal, wpisano)
VALUES (:funkcja, :osoba, :od, :podstawa, :wpisal, :wpisano)"#,
                parameters: Vec::from([
                    ("funkcja", xmeta_room::ToSqlValue::to_sql_value(&params.funkcja)),
                    ("osoba", xmeta_room::ToSqlValue::to_sql_value(&params.osoba)),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    ("podstawa", xmeta_room::ToSqlValue::to_sql_value(&params.podstawa)),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
        ],
    )
}

/// The parameters of `zakoncz_wyznaczenie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZakonczWyznaczenieParams {
    pub r#do: String,
    pub id: i64,
}

/// Koniec wyznaczenia; wpis zostaje.
pub fn zakoncz_wyznaczenie(
    room: &impl xmeta_room::MetaRoom,
    params: &ZakonczWyznaczenieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE rejestr_kontroli_organizacja_wyznaczenia
SET do = :do
WHERE id = :id"#,
                parameters: Vec::from([
                    ("do", xmeta_room::ToSqlValue::to_sql_value(&params.r#do)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
