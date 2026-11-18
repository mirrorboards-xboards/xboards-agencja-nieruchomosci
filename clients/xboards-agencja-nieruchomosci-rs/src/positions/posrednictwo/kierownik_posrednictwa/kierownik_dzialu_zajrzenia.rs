//! The kind `kierownik-dzialu-zajrzenia` of the room `positions/posrednictwo/kierownik_posrednictwa`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "kierownik-dzialu-zajrzenia",
    requires: &[&super::kierownik_dzialu_raporty::KIND],
    migrations: &[
        r#"-- Zajrzenia: każde zajrzenie kierownika do teczki, z powodem. Kierownik działu zagląda do teczki
-- tylko z powodem, a zajrzenia są śladem dla właściciela i dla kontroli, więc się ich nie zmienia.
--
-- Wpis nie ma terminu przechowania: dopóki obowiązuje rozstrzygnięcie biura R9, biuro niczego nie
-- usuwa z upływem czasu. Rejestr kontroli jest jeden dla biura, więc wpis nie wskazuje, czyj to
-- rejestr.
CREATE TABLE kierownik_dzialu_zajrzenia_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_dzialu_zajrzenia_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Pokój teczki, tekstem.
  teczka_odnosnik TEXT NOT NULL CHECK (teczka_odnosnik <> ''),
  zajrzano TEXT NOT NULL,
  powod TEXT NOT NULL CHECK (powod IN ('raport_sygnalizuje_problem', 'prosba_czlowieka', 'przejecie_po_agencie', 'sprawdzenie_jakosci')),
  -- Co skłoniło kierownika do zajrzenia.
  opis TEXT NOT NULL CHECK (opis <> ''),
  raport_id TEXT REFERENCES kierownik_dzialu_raporty_wpisy (id)
) STRICT;

CREATE TRIGGER kierownik_dzialu_zajrzenia_bez_zmian
BEFORE UPDATE ON kierownik_dzialu_zajrzenia_wpisy
BEGIN
  SELECT RAISE(ABORT, 'zajrzenia się nie zmienia; jest śladem dla właściciela i kontroli');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_dzialu_zajrzenia_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_dzialu_zajrzenia_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_dzialu_zajrzenia_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: zajrzeń się nie wykreśla.
CREATE TRIGGER kierownik_dzialu_zajrzenia_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_dzialu_zajrzenia_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
    ],
};

/// The values of column `wpisy.powod`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyPowod {
    RaportSygnalizujeProblem,
    ProsbaCzlowieka,
    PrzejeciePoAgencie,
    SprawdzenieJakosci,
}

impl WpisyPowod {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyPowod] = &[
        WpisyPowod::RaportSygnalizujeProblem,
        WpisyPowod::ProsbaCzlowieka,
        WpisyPowod::PrzejeciePoAgencie,
        WpisyPowod::SprawdzenieJakosci,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyPowod::RaportSygnalizujeProblem => "raport_sygnalizuje_problem",
            WpisyPowod::ProsbaCzlowieka => "prosba_czlowieka",
            WpisyPowod::PrzejeciePoAgencie => "przejecie_po_agencie",
            WpisyPowod::SprawdzenieJakosci => "sprawdzenie_jakosci",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyPowod {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "raport_sygnalizuje_problem" => {
                        Ok(WpisyPowod::RaportSygnalizujeProblem)
                    }
                    "prosba_czlowieka" => Ok(WpisyPowod::ProsbaCzlowieka),
                    "przejecie_po_agencie" => Ok(WpisyPowod::PrzejeciePoAgencie),
                    "sprawdzenie_jakosci" => Ok(WpisyPowod::SprawdzenieJakosci),
                    other => Err(format!("{other} is no value of WpisyPowod")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyPowod {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `zajrzenia`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZajrzeniaRow {
    pub id: i64,
    pub teczka_odnosnik: String,
    pub zajrzano: String,
    pub powod: WpisyPowod,
    pub opis: String,
    pub raport_id: Option<String>,
}

impl ZajrzeniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZajrzeniaRow {
            id: row.get("id")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            zajrzano: row.get("zajrzano")?,
            powod: row.get("powod")?,
            opis: row.get("opis")?,
            raport_id: row.get("raport_id")?,
        })
    }
}

/// Wszystkie zajrzenia kierownika do teczek, najnowsze pierwsze.
pub fn zajrzenia(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZajrzeniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_dzialu_zajrzenia_wpisy
ORDER BY zajrzano DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZajrzeniaRow::from_row).collect()
}

/// The parameters of `zajrzenia_od`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZajrzeniaOdParams {
    pub zajrzano: String,
}

/// A row of `zajrzenia_od`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZajrzeniaOdRow {
    pub id: i64,
    pub teczka_odnosnik: String,
    pub zajrzano: String,
    pub powod: WpisyPowod,
    pub opis: String,
    pub raport_id: Option<String>,
}

impl ZajrzeniaOdRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZajrzeniaOdRow {
            id: row.get("id")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            zajrzano: row.get("zajrzano")?,
            powod: row.get("powod")?,
            opis: row.get("opis")?,
            raport_id: row.get("raport_id")?,
        })
    }
}

/// Zajrzenia od podanej chwili, np. do raportu dla właściciela.
pub fn zajrzenia_od(
    room: &impl xmeta_room::MetaRoom,
    params: &ZajrzeniaOdParams,
) -> Result<Vec<ZajrzeniaOdRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_dzialu_zajrzenia_wpisy
WHERE zajrzano >= :zajrzano
ORDER BY zajrzano, id"#,
        parameters: Vec::from([
            ("zajrzano", xmeta_room::ToSqlValue::to_sql_value(&params.zajrzano)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZajrzeniaOdRow::from_row).collect()
}

/// The parameters of `zapisz_zajrzenie`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZapiszZajrzenieParams {
    pub teczka_odnosnik: String,
    pub zajrzano: String,
    pub powod: WpisyPowod,
    pub opis: String,
    pub raport_id: Option<String>,
}

/// Zajrzenie do teczki z powodem i opisem; raport, jeśli to on sygnalizował problem.
pub fn zapisz_zajrzenie(
    room: &impl xmeta_room::MetaRoom,
    params: &ZapiszZajrzenieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_dzialu_zajrzenia_wpisy (teczka_odnosnik, zajrzano, powod, opis, raport_id)
VALUES (:teczka_odnosnik, :zajrzano, :powod, :opis, :raport_id)"#,
                parameters: Vec::from([
                    (
                        "teczka_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.teczka_odnosnik),
                    ),
                    ("zajrzano", xmeta_room::ToSqlValue::to_sql_value(&params.zajrzano)),
                    ("powod", xmeta_room::ToSqlValue::to_sql_value(&params.powod)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    (
                        "raport_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.raport_id),
                    ),
                ]),
            },
        ],
    )
}
