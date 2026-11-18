//! The kind `rejestr-kontroli-zawiadomienia` of the room `registers/rejestr_kontroli`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "rejestr-kontroli-zawiadomienia",
    requires: &[&super::rejestr_kontroli_decyzje::KIND],
    migrations: &[
        r#"-- Ślad zawiadomień, żądania organów i kontrole. Projekt zawiadomienia leży w pamięci Kierownika
-- kontroli, dopóki człowiek go nie przekaże; do rejestru trafia ślad w dniu przekazania. Żaden
-- agent nie przekazuje niczego organowi; robi to osoba wyznaczona do obowiązków z ustawy.
--
-- Zawiadomienie wskazuje dwie zatwierdzone decyzje: potwierdzenie podejrzenia i przekazanie, a jego
-- przekazanie nie jest wcześniejsze niż potwierdzenie. Termin 2 dni roboczych (art. 74 ust. 2 AML)
-- liczy program. Wierszy się nie zmienia; to, co przychodzi od organu później, wpisuje się raz.
CREATE TABLE rejestr_kontroli_zawiadomienia_wpisy (
  id TEXT PRIMARY KEY,
  -- Art. 74, 86 i 90 AML do Generalnego Inspektora, art. 89 AML do prokuratora.
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('art74', 'art86', 'art90', 'art89_prokurator')),
  potwierdzenie_decyzja_id TEXT NOT NULL REFERENCES rejestr_kontroli_decyzje_wpisy (id),
  przekazanie_decyzja_id TEXT NOT NULL REFERENCES rejestr_kontroli_decyzje_wpisy (id),
  -- Osoba wyznaczona, która przekazała zawiadomienie (art. 8 AML), i chwila przekazania.
  przekazal TEXT NOT NULL CHECK (przekazal <> ''),
  przekazano TEXT NOT NULL,
  -- Odnośniki do wpisów rejestru, na których zawiadomienie stoi.
  podstawy TEXT NOT NULL CHECK (podstawy <> ''),
  -- Urzędowe poświadczenie odbioru (art. 86 ust. 3 AML) i odpowiedź organu, np. żądanie
  -- wstrzymania transakcji (art. 86 ust. 5 AML); każde wpisuje się raz, gdy przyjdzie.
  potwierdzenie_przyjecia TEXT,
  odpowiedz_organu TEXT,
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL
) STRICT;

-- Żądania organów i to, co biuro przekazało.
CREATE TABLE rejestr_kontroli_zawiadomienia_zadania (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_zawiadomienia_zadania_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Np. Generalny Inspektor Informacji Finansowej (art. 76 ust. 1 AML).
  organ TEXT NOT NULL CHECK (organ <> ''),
  zadano TEXT NOT NULL,
  tresc TEXT NOT NULL CHECK (tresc <> ''),
  -- Właściciel Kierownika kontroli, który odebrał żądanie.
  odebral TEXT NOT NULL CHECK (odebral <> ''),
  -- Kiedy i co biuro przekazało; wpisuje się raz.
  przekazano TEXT,
  co_przekazano TEXT,
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,

  CONSTRAINT przekazanie_w_komplecie CHECK ((przekazano IS NULL) = (co_przekazano IS NULL)),
  CONSTRAINT przekazanie_po_zadaniu CHECK (przekazano IS NULL OR przekazano >= zadano)
) STRICT;

-- Kontrole wykonywania obowiązków z protokołem i zaleceniami, wpisane po protokole.
CREATE TABLE rejestr_kontroli_zawiadomienia_kontrole (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_zawiadomienia_kontrole_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Kto kontrolował wykonywanie obowiązków (art. 130 ust. 1 AML).
  organ TEXT NOT NULL CHECK (organ <> ''),
  od TEXT NOT NULL,
  do TEXT NOT NULL,
  protokol_sciezka TEXT NOT NULL CHECK (protokol_sciezka <> ''),
  protokol_skrot TEXT NOT NULL CHECK (protokol_skrot <> ''),
  zalecenia TEXT,
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,

  CONSTRAINT koniec_nie_przed_poczatkiem CHECK (do >= od)
) STRICT;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_wpisy_po_potwierdzeniu_podejrzenia
BEFORE INSERT ON rejestr_kontroli_zawiadomienia_wpisy
WHEN NOT EXISTS (
  SELECT 1
  FROM rejestr_kontroli_decyzje_zatwierdzone
  WHERE id = NEW.potwierdzenie_decyzja_id
    AND rodzaj = 'potwierdzenie_podejrzenia'
)
BEGIN
  SELECT RAISE(ABORT, 'zawiadomienie wskazuje zatwierdzoną decyzję potwierdzenie_podejrzenia');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_wpisy_po_decyzji_o_przekazaniu
BEFORE INSERT ON rejestr_kontroli_zawiadomienia_wpisy
WHEN NOT EXISTS (
  SELECT 1
  FROM rejestr_kontroli_decyzje_zatwierdzone
  WHERE id = NEW.przekazanie_decyzja_id
    AND rodzaj = 'przekazanie_zawiadomienia'
)
BEGIN
  SELECT RAISE(ABORT, 'zawiadomienie wskazuje zatwierdzoną decyzję przekazanie_zawiadomienia');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_wpisy_przekazanie_po_potwierdzeniu
BEFORE INSERT ON rejestr_kontroli_zawiadomienia_wpisy
WHEN NEW.przekazano < (
  SELECT podpisano
  FROM rejestr_kontroli_decyzje_wpisy
  WHERE id = NEW.potwierdzenie_decyzja_id
)
BEGIN
  SELECT RAISE(ABORT, 'przekazanie zawiadomienia nie jest wcześniejsze niż potwierdzenie podejrzenia');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_wpisy_odpowiedzi_przychodza_pozniej
BEFORE INSERT ON rejestr_kontroli_zawiadomienia_wpisy
WHEN NEW.potwierdzenie_przyjecia IS NOT NULL
  OR NEW.odpowiedz_organu IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'zawiadomienie powstaje bez potwierdzenia przyjęcia i odpowiedzi organu; wpisuje się je, gdy przyjdą');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_wpisy_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_zawiadomienia_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.potwierdzenie_decyzja_id IS NOT OLD.potwierdzenie_decyzja_id
  OR NEW.przekazanie_decyzja_id IS NOT OLD.przekazanie_decyzja_id
  OR NEW.przekazal IS NOT OLD.przekazal
  OR NEW.przekazano IS NOT OLD.przekazano
  OR NEW.podstawy IS NOT OLD.podstawy
  OR NEW.wpisal IS NOT OLD.wpisal
  OR NEW.wpisano IS NOT OLD.wpisano
  OR (OLD.potwierdzenie_przyjecia IS NOT NULL AND NEW.potwierdzenie_przyjecia IS NOT OLD.potwierdzenie_przyjecia)
  OR (OLD.odpowiedz_organu IS NOT NULL AND NEW.odpowiedz_organu IS NOT OLD.odpowiedz_organu)
BEGIN
  SELECT RAISE(ABORT, 'śladu zawiadomienia się nie zmienia; potwierdzenie przyjęcia i odpowiedź organu wpisuje się raz');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_zadania_powstaje_bez_przekazania
BEFORE INSERT ON rejestr_kontroli_zawiadomienia_zadania
WHEN NEW.przekazano IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'żądanie organu powstaje bez przekazania; to, co biuro przekazało, wpisuje się raz');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_zadania_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_zawiadomienia_zadania
WHEN NEW.id IS NOT OLD.id
  OR NEW.organ IS NOT OLD.organ
  OR NEW.zadano IS NOT OLD.zadano
  OR NEW.tresc IS NOT OLD.tresc
  OR NEW.odebral IS NOT OLD.odebral
  OR NEW.wpisal IS NOT OLD.wpisal
  OR NEW.wpisano IS NOT OLD.wpisano
  OR (OLD.przekazano IS NOT NULL AND (NEW.przekazano IS NOT OLD.przekazano OR NEW.co_przekazano IS NOT OLD.co_przekazano))
BEGIN
  SELECT RAISE(ABORT, 'żądania organu się nie zmienia; to, co biuro przekazało, wpisuje się raz');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_kontrole_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_zawiadomienia_kontrole
BEGIN
  SELECT RAISE(ABORT, 'wpisu kontroli się nie zmienia; błąd poprawia nowy wpis');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabel z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER rejestr_kontroli_zawiadomienia_wpisy_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_zawiadomienia_wpisy
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_zawiadomienia_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_zadania_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_zawiadomienia_zadania
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_zawiadomienia_zadania
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_kontrole_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_zawiadomienia_kontrole
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_zawiadomienia_kontrole
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

-- Art. 49 AML i R9: dokumentację przechowuje się co najmniej 5 lat, a na razie bez terminu.
CREATE TRIGGER rejestr_kontroli_zawiadomienia_wpisy_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_zawiadomienia_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_zadania_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_zawiadomienia_zadania
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_zawiadomienia_kontrole_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_zawiadomienia_kontrole
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;
"#,
    ],
};

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    Art74,
    Art86,
    Art90,
    Art89Prokurator,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::Art74,
        WpisyRodzaj::Art86,
        WpisyRodzaj::Art90,
        WpisyRodzaj::Art89Prokurator,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::Art74 => "art74",
            WpisyRodzaj::Art86 => "art86",
            WpisyRodzaj::Art90 => "art90",
            WpisyRodzaj::Art89Prokurator => "art89_prokurator",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "art74" => Ok(WpisyRodzaj::Art74),
                    "art86" => Ok(WpisyRodzaj::Art86),
                    "art90" => Ok(WpisyRodzaj::Art90),
                    "art89_prokurator" => Ok(WpisyRodzaj::Art89Prokurator),
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

/// A row of `kontrole_organow`.
#[derive(Debug, Clone, PartialEq)]
pub struct KontroleOrganowRow {
    pub id: i64,
    pub organ: String,
    pub od: String,
    pub r#do: String,
    pub protokol_sciezka: String,
    pub protokol_skrot: String,
    pub zalecenia: Option<String>,
    pub wpisal: String,
    pub wpisano: String,
}

impl KontroleOrganowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(KontroleOrganowRow {
            id: row.get("id")?,
            organ: row.get("organ")?,
            od: row.get("od")?,
            r#do: row.get("do")?,
            protokol_sciezka: row.get("protokol_sciezka")?,
            protokol_skrot: row.get("protokol_skrot")?,
            zalecenia: row.get("zalecenia")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Kontrole wykonywania obowiązków, najnowsze pierwsze.
pub fn kontrole_organow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<KontroleOrganowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_zawiadomienia_kontrole
ORDER BY od DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(KontroleOrganowRow::from_row).collect()
}

/// The parameters of `wpisz_kontrole_organu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszKontroleOrganuParams {
    pub organ: String,
    pub od: String,
    pub r#do: String,
    pub protokol_sciezka: String,
    pub protokol_skrot: String,
    pub zalecenia: Option<String>,
    pub wpisal: String,
    pub wpisano: String,
}

/// Kontrola z protokołem i zaleceniami.
pub fn wpisz_kontrole_organu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszKontroleOrganuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_zawiadomienia_kontrole (organ, od, do, protokol_sciezka, protokol_skrot,
  zalecenia, wpisal, wpisano)
VALUES (:organ, :od, :do, :protokol_sciezka, :protokol_skrot, :zalecenia, :wpisal, :wpisano)"#,
                parameters: Vec::from([
                    ("organ", xmeta_room::ToSqlValue::to_sql_value(&params.organ)),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    ("do", xmeta_room::ToSqlValue::to_sql_value(&params.r#do)),
                    (
                        "protokol_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.protokol_sciezka),
                    ),
                    (
                        "protokol_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.protokol_skrot),
                    ),
                    (
                        "zalecenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zalecenia),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
        ],
    )
}

/// A row of `zawiadomienia`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZawiadomieniaRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub potwierdzenie_decyzja_id: String,
    pub przekazanie_decyzja_id: String,
    pub przekazal: String,
    pub przekazano: String,
    pub podstawy: String,
    pub potwierdzenie_przyjecia: Option<String>,
    pub odpowiedz_organu: Option<String>,
    pub wpisal: String,
    pub wpisano: String,
}

impl ZawiadomieniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZawiadomieniaRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            potwierdzenie_decyzja_id: row.get("potwierdzenie_decyzja_id")?,
            przekazanie_decyzja_id: row.get("przekazanie_decyzja_id")?,
            przekazal: row.get("przekazal")?,
            przekazano: row.get("przekazano")?,
            podstawy: row.get("podstawy")?,
            potwierdzenie_przyjecia: row.get("potwierdzenie_przyjecia")?,
            odpowiedz_organu: row.get("odpowiedz_organu")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Ślad zawiadomień, najnowsze pierwsze.
pub fn zawiadomienia(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZawiadomieniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_zawiadomienia_wpisy
ORDER BY przekazano DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZawiadomieniaRow::from_row).collect()
}

/// The parameters of `wpisz_zawiadomienie`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszZawiadomienieParams {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub potwierdzenie_decyzja_id: String,
    pub przekazanie_decyzja_id: String,
    pub przekazal: String,
    pub przekazano: String,
    pub podstawy: String,
    pub wpisal: String,
    pub wpisano: String,
}

/// Ślad zawiadomienia w dniu, w którym osoba wyznaczona je przekazała. Identyfikator (UUID) nadaje
/// aplikacja.
pub fn wpisz_zawiadomienie(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszZawiadomienieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_zawiadomienia_wpisy (id, rodzaj, potwierdzenie_decyzja_id,
  przekazanie_decyzja_id, przekazal, przekazano, podstawy, wpisal, wpisano)
VALUES (:id, :rodzaj, :potwierdzenie_decyzja_id, :przekazanie_decyzja_id, :przekazal, :przekazano,
  :podstawy, :wpisal, :wpisano)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    (
                        "potwierdzenie_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.potwierdzenie_decyzja_id,
                        ),
                    ),
                    (
                        "przekazanie_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.przekazanie_decyzja_id,
                        ),
                    ),
                    (
                        "przekazal",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przekazal),
                    ),
                    (
                        "przekazano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przekazano),
                    ),
                    ("podstawy", xmeta_room::ToSqlValue::to_sql_value(&params.podstawy)),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
        ],
    )
}

/// The parameters of `wpisz_potwierdzenie_przyjecia`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszPotwierdzeniePrzyjeciaParams {
    pub potwierdzenie_przyjecia: String,
    pub id: String,
}

/// Urzędowe poświadczenie odbioru zawiadomienia; wpisuje się raz.
pub fn wpisz_potwierdzenie_przyjecia(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszPotwierdzeniePrzyjeciaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE rejestr_kontroli_zawiadomienia_wpisy
SET potwierdzenie_przyjecia = :potwierdzenie_przyjecia
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "potwierdzenie_przyjecia",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.potwierdzenie_przyjecia,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wpisz_odpowiedz_organu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszOdpowiedzOrganuParams {
    pub odpowiedz_organu: String,
    pub id: String,
}

/// Odpowiedź organu, np. żądanie wstrzymania transakcji; wpisuje się raz.
pub fn wpisz_odpowiedz_organu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszOdpowiedzOrganuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE rejestr_kontroli_zawiadomienia_wpisy
SET odpowiedz_organu = :odpowiedz_organu
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "odpowiedz_organu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odpowiedz_organu),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `zadania_organow`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZadaniaOrganowRow {
    pub id: i64,
    pub organ: String,
    pub zadano: String,
    pub tresc: String,
    pub odebral: String,
    pub przekazano: Option<String>,
    pub co_przekazano: Option<String>,
    pub wpisal: String,
    pub wpisano: String,
}

impl ZadaniaOrganowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZadaniaOrganowRow {
            id: row.get("id")?,
            organ: row.get("organ")?,
            zadano: row.get("zadano")?,
            tresc: row.get("tresc")?,
            odebral: row.get("odebral")?,
            przekazano: row.get("przekazano")?,
            co_przekazano: row.get("co_przekazano")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Żądania organów, najnowsze pierwsze.
pub fn zadania_organow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZadaniaOrganowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_zawiadomienia_zadania
ORDER BY zadano DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZadaniaOrganowRow::from_row).collect()
}

/// The parameters of `wpisz_zadanie_organu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszZadanieOrganuParams {
    pub organ: String,
    pub zadano: String,
    pub tresc: String,
    pub odebral: String,
    pub wpisal: String,
    pub wpisano: String,
}

/// Żądanie organu, które odebrał właściciel Kierownika kontroli.
pub fn wpisz_zadanie_organu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszZadanieOrganuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_zawiadomienia_zadania (organ, zadano, tresc, odebral, wpisal, wpisano)
VALUES (:organ, :zadano, :tresc, :odebral, :wpisal, :wpisano)"#,
                parameters: Vec::from([
                    ("organ", xmeta_room::ToSqlValue::to_sql_value(&params.organ)),
                    ("zadano", xmeta_room::ToSqlValue::to_sql_value(&params.zadano)),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                    ("odebral", xmeta_room::ToSqlValue::to_sql_value(&params.odebral)),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
        ],
    )
}

/// The parameters of `wpisz_odpowiedz_na_zadanie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszOdpowiedzNaZadanieParams {
    pub przekazano: String,
    pub co_przekazano: String,
    pub id: i64,
}

/// Kiedy i co biuro przekazało organowi; wpisuje się raz.
pub fn wpisz_odpowiedz_na_zadanie(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszOdpowiedzNaZadanieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE rejestr_kontroli_zawiadomienia_zadania
SET przekazano = :przekazano,
  co_przekazano = :co_przekazano
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "przekazano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przekazano),
                    ),
                    (
                        "co_przekazano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.co_przekazano),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
