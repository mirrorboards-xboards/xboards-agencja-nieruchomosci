//! The kind `kierownik-biura-nieobecnosci` of the room `positions/biuro/kierownik_biura`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "kierownik-biura-nieobecnosci",
    requires: &[],
    migrations: &[
        r#"-- Nieobecności i zastępcy: kto jest nieobecny, od kiedy do kiedy i kto dostaje w tym czasie jego
-- listę decyzji, z odwołaniem do wiadomości, którą to zgłoszono.
--
-- Kierownik przyjmuje nieobecność tylko z podpisem człowieka, którego dotyczy, albo właściciela
-- biura, bo niepodpisana wiadomość mogłaby przekierować czyjąś listę (podszywanie się). Podpis
-- sprawdza aplikacja, a baza wymaga jego obecności. Odwołanie też przychodzi podpisane.
CREATE TABLE kierownik_biura_nieobecnosci_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_biura_nieobecnosci_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Tożsamość człowieka, który będzie nieobecny.
  czlowiek TEXT NOT NULL CHECK (czlowiek <> ''),
  -- Daty RRRR-MM-DD, obie włącznie.
  od_dnia TEXT NOT NULL,
  do_dnia TEXT NOT NULL,
  -- Tożsamość człowieka, który dostaje listę.
  zastepca TEXT NOT NULL CHECK (zastepca <> ''),
  podpisal TEXT NOT NULL,
  podpisal_jako TEXT NOT NULL CHECK (podpisal_jako IN ('czlowiek', 'wlasciciel_biura')),
  podpis TEXT NOT NULL CHECK (podpis <> ''),
  -- Wiadomość, którą zgłoszono nieobecność, tekstem.
  wiadomosc_odnosnik TEXT NOT NULL CHECK (wiadomosc_odnosnik <> ''),
  zapisano TEXT NOT NULL,
  odwolal TEXT,
  odwolanie_podpis TEXT,
  odwolano TEXT,

  CONSTRAINT nieobecnosc_od_przed_do CHECK (od_dnia <= do_dnia),
  CONSTRAINT zastepca_to_ktos_inny CHECK (zastepca <> czlowiek),
  CONSTRAINT podpis_czlowieka_ktorego_dotyczy CHECK (podpisal_jako <> 'czlowiek' OR podpisal = czlowiek),
  CONSTRAINT odwolanie_z_podpisem CHECK (
    (odwolano IS NULL) = (odwolal IS NULL)
    AND (odwolano IS NULL) = (odwolanie_podpis IS NULL)
  )
) STRICT;

CREATE TRIGGER kierownik_biura_nieobecnosci_powstaje_nieodwolana
BEFORE INSERT ON kierownik_biura_nieobecnosci_wpisy
WHEN NEW.odwolano IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'nieobecność powstaje nieodwołana; odwołanie przychodzi później');
END;

-- Podpis obejmuje nieobecność, więc zmienia się tylko raz odnotowane odwołanie; inna nieobecność
-- to nowy wpis.
CREATE TRIGGER kierownik_biura_nieobecnosci_bez_zmian
BEFORE UPDATE ON kierownik_biura_nieobecnosci_wpisy
WHEN OLD.odwolano IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.czlowiek IS NOT OLD.czlowiek
  OR NEW.od_dnia IS NOT OLD.od_dnia
  OR NEW.do_dnia IS NOT OLD.do_dnia
  OR NEW.zastepca IS NOT OLD.zastepca
  OR NEW.podpisal IS NOT OLD.podpisal
  OR NEW.podpisal_jako IS NOT OLD.podpisal_jako
  OR NEW.podpis IS NOT OLD.podpis
  OR NEW.wiadomosc_odnosnik IS NOT OLD.wiadomosc_odnosnik
  OR NEW.zapisano IS NOT OLD.zapisano
BEGIN
  SELECT RAISE(ABORT, 'nieobecności się nie zmienia; wolno ją raz odwołać, a inna nieobecność to nowy wpis');
END;

-- Odwołuje ten sam człowiek albo właściciel biura, bo inaczej odwołanie byłoby podszyciem.
CREATE TRIGGER kierownik_biura_nieobecnosci_odwoluje_czlowiek_albo_wlasciciel_biura
BEFORE UPDATE OF odwolal ON kierownik_biura_nieobecnosci_wpisy
WHEN NEW.odwolal IS NOT NULL
  AND NEW.odwolal IS NOT NEW.czlowiek
  AND NOT (NEW.podpisal_jako = 'wlasciciel_biura' AND NEW.odwolal = NEW.podpisal)
BEGIN
  SELECT RAISE(ABORT, 'nieobecność odwołuje człowiek, którego dotyczy, albo właściciel biura, który ją zgłosił');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_biura_nieobecnosci_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_biura_nieobecnosci_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_biura_nieobecnosci_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: dawne nieobecności zostają w sprawozdaniach.
CREATE TRIGGER kierownik_biura_nieobecnosci_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_biura_nieobecnosci_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
    ],
};

/// The values of column `wpisy.podpisal_jako`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyPodpisalJako {
    Czlowiek,
    WlascicielBiura,
}

impl WpisyPodpisalJako {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyPodpisalJako] = &[
        WpisyPodpisalJako::Czlowiek,
        WpisyPodpisalJako::WlascicielBiura,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyPodpisalJako::Czlowiek => "czlowiek",
            WpisyPodpisalJako::WlascicielBiura => "wlasciciel_biura",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyPodpisalJako {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "czlowiek" => Ok(WpisyPodpisalJako::Czlowiek),
                    "wlasciciel_biura" => Ok(WpisyPodpisalJako::WlascicielBiura),
                    other => Err(format!("{other} is no value of WpisyPodpisalJako")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyPodpisalJako {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The parameters of `nieobecnosci_w_dniu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NieobecnosciWDniuParams {
    pub dzien: String,
}

/// A row of `nieobecnosci_w_dniu`.
#[derive(Debug, Clone, PartialEq)]
pub struct NieobecnosciWDniuRow {
    pub id: i64,
    pub czlowiek: String,
    pub od_dnia: String,
    pub do_dnia: String,
    pub zastepca: String,
    pub podpisal: String,
    pub podpisal_jako: WpisyPodpisalJako,
    pub podpis: String,
    pub wiadomosc_odnosnik: String,
    pub zapisano: String,
    pub odwolal: Option<String>,
    pub odwolanie_podpis: Option<String>,
    pub odwolano: Option<String>,
}

impl NieobecnosciWDniuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(NieobecnosciWDniuRow {
            id: row.get("id")?,
            czlowiek: row.get("czlowiek")?,
            od_dnia: row.get("od_dnia")?,
            do_dnia: row.get("do_dnia")?,
            zastepca: row.get("zastepca")?,
            podpisal: row.get("podpisal")?,
            podpisal_jako: row.get("podpisal_jako")?,
            podpis: row.get("podpis")?,
            wiadomosc_odnosnik: row.get("wiadomosc_odnosnik")?,
            zapisano: row.get("zapisano")?,
            odwolal: row.get("odwolal")?,
            odwolanie_podpis: row.get("odwolanie_podpis")?,
            odwolano: row.get("odwolano")?,
        })
    }
}

/// Nieodwołane nieobecności, które obejmują jeden dzień (RRRR-MM-DD).
pub fn nieobecnosci_w_dniu(
    room: &impl xmeta_room::MetaRoom,
    params: &NieobecnosciWDniuParams,
) -> Result<Vec<NieobecnosciWDniuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_biura_nieobecnosci_wpisy
WHERE odwolano IS NULL
  AND od_dnia <= :dzien
  AND do_dnia >= :dzien
ORDER BY czlowiek, id"#,
        parameters: Vec::from([
            ("dzien", xmeta_room::ToSqlValue::to_sql_value(&params.dzien)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(NieobecnosciWDniuRow::from_row).collect()
}

/// The parameters of `nieobecnosci_od`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NieobecnosciOdParams {
    pub od: String,
}

/// A row of `nieobecnosci_od`.
#[derive(Debug, Clone, PartialEq)]
pub struct NieobecnosciOdRow {
    pub id: i64,
    pub czlowiek: String,
    pub od_dnia: String,
    pub do_dnia: String,
    pub zastepca: String,
    pub podpisal: String,
    pub podpisal_jako: WpisyPodpisalJako,
    pub podpis: String,
    pub wiadomosc_odnosnik: String,
    pub zapisano: String,
    pub odwolal: Option<String>,
    pub odwolanie_podpis: Option<String>,
    pub odwolano: Option<String>,
}

impl NieobecnosciOdRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(NieobecnosciOdRow {
            id: row.get("id")?,
            czlowiek: row.get("czlowiek")?,
            od_dnia: row.get("od_dnia")?,
            do_dnia: row.get("do_dnia")?,
            zastepca: row.get("zastepca")?,
            podpisal: row.get("podpisal")?,
            podpisal_jako: row.get("podpisal_jako")?,
            podpis: row.get("podpis")?,
            wiadomosc_odnosnik: row.get("wiadomosc_odnosnik")?,
            zapisano: row.get("zapisano")?,
            odwolal: row.get("odwolal")?,
            odwolanie_podpis: row.get("odwolanie_podpis")?,
            odwolano: row.get("odwolano")?,
        })
    }
}

/// Nieobecności, które trwały albo trwają od jednego dnia, także odwołane, do sprawozdania.
pub fn nieobecnosci_od(
    room: &impl xmeta_room::MetaRoom,
    params: &NieobecnosciOdParams,
) -> Result<Vec<NieobecnosciOdRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_biura_nieobecnosci_wpisy
WHERE do_dnia >= :od
ORDER BY od_dnia, id"#,
        parameters: Vec::from([("od", xmeta_room::ToSqlValue::to_sql_value(&params.od))]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(NieobecnosciOdRow::from_row).collect()
}

/// The parameters of `zapisz_nieobecnosc`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZapiszNieobecnoscParams {
    pub czlowiek: String,
    pub od_dnia: String,
    pub do_dnia: String,
    pub zastepca: String,
    pub podpisal: String,
    pub podpisal_jako: WpisyPodpisalJako,
    pub podpis: String,
    pub wiadomosc_odnosnik: String,
    pub zapisano: String,
}

/// Nieobecność ze zastępcą z podpisanej wiadomości człowieka albo właściciela biura.
pub fn zapisz_nieobecnosc(
    room: &impl xmeta_room::MetaRoom,
    params: &ZapiszNieobecnoscParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_biura_nieobecnosci_wpisy (czlowiek, od_dnia, do_dnia, zastepca, podpisal,
  podpisal_jako, podpis, wiadomosc_odnosnik, zapisano)
VALUES (:czlowiek, :od_dnia, :do_dnia, :zastepca, :podpisal, :podpisal_jako, :podpis,
  :wiadomosc_odnosnik, :zapisano)"#,
                parameters: Vec::from([
                    ("czlowiek", xmeta_room::ToSqlValue::to_sql_value(&params.czlowiek)),
                    ("od_dnia", xmeta_room::ToSqlValue::to_sql_value(&params.od_dnia)),
                    ("do_dnia", xmeta_room::ToSqlValue::to_sql_value(&params.do_dnia)),
                    ("zastepca", xmeta_room::ToSqlValue::to_sql_value(&params.zastepca)),
                    ("podpisal", xmeta_room::ToSqlValue::to_sql_value(&params.podpisal)),
                    (
                        "podpisal_jako",
                        xmeta_room::ToSqlValue::to_sql_value(&params.podpisal_jako),
                    ),
                    ("podpis", xmeta_room::ToSqlValue::to_sql_value(&params.podpis)),
                    (
                        "wiadomosc_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wiadomosc_odnosnik),
                    ),
                    ("zapisano", xmeta_room::ToSqlValue::to_sql_value(&params.zapisano)),
                ]),
            },
        ],
    )
}

/// The parameters of `odwolaj_nieobecnosc`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdwolajNieobecnoscParams {
    pub odwolal: String,
    pub odwolanie_podpis: String,
    pub odwolano: String,
    pub id: i64,
}

/// Podpisane odwołanie nieobecności.
pub fn odwolaj_nieobecnosc(
    room: &impl xmeta_room::MetaRoom,
    params: &OdwolajNieobecnoscParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_biura_nieobecnosci_wpisy
SET odwolal = :odwolal,
  odwolanie_podpis = :odwolanie_podpis,
  odwolano = :odwolano
WHERE id = :id"#,
                parameters: Vec::from([
                    ("odwolal", xmeta_room::ToSqlValue::to_sql_value(&params.odwolal)),
                    (
                        "odwolanie_podpis",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odwolanie_podpis),
                    ),
                    ("odwolano", xmeta_room::ToSqlValue::to_sql_value(&params.odwolano)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
