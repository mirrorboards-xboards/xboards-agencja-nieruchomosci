//! The kind `rejestr-kontroli-watpliwosci` of the room `registers/rejestr_kontroli`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "rejestr-kontroli-watpliwosci";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Wątpliwości Kontrolerów z powodem, którego nie ma w teczce, i decyzja człowieka, która wątpliwość
-- zdjęła, oraz niemożność zastosowania środka bezpieczeństwa finansowego. Teczka ma tylko znacznik
-- (art. 54 ust. 1 AML); powód i uzasadnienie zdjęcia są wyłącznie tutaj.
--
-- Kierownik kontroli wpisuje zgłoszenie w dniu raportu. Zdjęcie wpisuje się raz: wskazuje
-- zatwierdzoną decyzję zdjecie_watpliwosci osoby wyznaczonej o tej wątpliwości, a decyzja ma
-- uzasadnienie. Niemożność Kierownik wpisuje, gdy zapadła decyzja o nienawiązaniu albo rozwiązaniu
-- stosunków i ocena, czy zawiadomić Generalnego Inspektora.
CREATE TABLE rejestr_kontroli_watpliwosci_wpisy (
  id TEXT PRIMARY KEY,
  -- Teczka i znacznik w niej (sprawa_metryka_watpliwosci); teczka ma sam znacznik bez powodu.
  teczka_odnosnik TEXT NOT NULL CHECK (teczka_odnosnik <> ''),
  znacznik_odnosnik TEXT NOT NULL CHECK (znacznik_odnosnik <> ''),
  -- Klient, którego wątpliwość dotyczy; NULL przy drugiej stronie, która nie jest klientem.
  klient_id TEXT REFERENCES rejestr_kontroli_klienci_wpisy (id),
  -- Kontroler, który zgłosił wątpliwość, i kiedy.
  zglosil TEXT NOT NULL CHECK (zglosil <> ''),
  zgloszono TEXT NOT NULL,
  powod TEXT NOT NULL CHECK (powod <> ''),
  raport_odnosnik TEXT NOT NULL CHECK (raport_odnosnik <> ''),
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,
  -- Decyzja zdjecie_watpliwosci osoby wyznaczonej z uzasadnieniem; NULL znaczy, że wątpliwość stoi.
  zdjecie_decyzja_id TEXT REFERENCES rejestr_kontroli_decyzje_wpisy (id),

  UNIQUE (teczka_odnosnik, znacznik_odnosnik)
) STRICT;

CREATE TABLE rejestr_kontroli_watpliwosci_niemoznosci (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_watpliwosci_niemoznosci_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  klient_id TEXT NOT NULL REFERENCES rejestr_kontroli_klienci_wpisy (id),
  -- Środek z art. 34 ust. 1 AML, którego nie dało się zastosować.
  srodek TEXT NOT NULL CHECK (srodek IN ('identyfikacja', 'weryfikacja', 'beneficjent', 'cel_i_charakter', 'monitorowanie')),
  opis TEXT NOT NULL CHECK (opis <> ''),
  -- Decyzja o nienawiązaniu albo rozwiązaniu stosunków (art. 41 ust. 1 AML).
  decyzja_id TEXT NOT NULL REFERENCES rejestr_kontroli_decyzje_wpisy (id),
  -- Czy to podstawa do zawiadomienia Generalnego Inspektora (art. 41 ust. 2 AML) i dlaczego.
  ocena_zawiadomienia TEXT NOT NULL CHECK (ocena_zawiadomienia <> ''),
  raport_odnosnik TEXT NOT NULL CHECK (raport_odnosnik <> ''),
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL
) STRICT;

CREATE TRIGGER rejestr_kontroli_watpliwosci_wpisy_powstaje_niezdjeta
BEFORE INSERT ON rejestr_kontroli_watpliwosci_wpisy
WHEN NEW.zdjecie_decyzja_id IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'wątpliwość powstaje jako stojąca; zdjęcie wpisuje się po decyzji');
END;

CREATE TRIGGER rejestr_kontroli_watpliwosci_wpisy_zdjecie_wskazuje_decyzje
BEFORE UPDATE OF zdjecie_decyzja_id ON rejestr_kontroli_watpliwosci_wpisy
WHEN NEW.zdjecie_decyzja_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM rejestr_kontroli_decyzje_zatwierdzone
    WHERE id = NEW.zdjecie_decyzja_id
      AND rodzaj = 'zdjecie_watpliwosci'
      AND dotyczy_tabela = 'rejestr_kontroli_watpliwosci_wpisy'
      AND dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'zdjęcie wątpliwości wskazuje zatwierdzoną decyzję zdjecie_watpliwosci o tej wątpliwości');
END;

CREATE TRIGGER rejestr_kontroli_watpliwosci_wpisy_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_watpliwosci_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.teczka_odnosnik IS NOT OLD.teczka_odnosnik
  OR NEW.znacznik_odnosnik IS NOT OLD.znacznik_odnosnik
  OR NEW.klient_id IS NOT OLD.klient_id
  OR NEW.zglosil IS NOT OLD.zglosil
  OR NEW.zgloszono IS NOT OLD.zgloszono
  OR NEW.powod IS NOT OLD.powod
  OR NEW.raport_odnosnik IS NOT OLD.raport_odnosnik
  OR NEW.wpisal IS NOT OLD.wpisal
  OR NEW.wpisano IS NOT OLD.wpisano
  OR (OLD.zdjecie_decyzja_id IS NOT NULL AND NEW.zdjecie_decyzja_id IS NOT OLD.zdjecie_decyzja_id)
BEGIN
  SELECT RAISE(ABORT, 'wątpliwości się nie zmienia; zdjęcie wpisuje się raz');
END;

CREATE TRIGGER rejestr_kontroli_watpliwosci_niemoznosci_po_decyzji_o_nienawiazaniu
BEFORE INSERT ON rejestr_kontroli_watpliwosci_niemoznosci
WHEN NOT EXISTS (
  SELECT 1
  FROM rejestr_kontroli_decyzje_zatwierdzone
  WHERE id = NEW.decyzja_id
    AND rodzaj = 'nienawiazanie_stosunkow'
    AND dotyczy_tabela = 'rejestr_kontroli_klienci_wpisy'
    AND dotyczy_id = NEW.klient_id
)
BEGIN
  SELECT RAISE(ABORT, 'niemożność środka wskazuje zatwierdzoną decyzję nienawiazanie_stosunkow o tym kliencie');
END;

CREATE TRIGGER rejestr_kontroli_watpliwosci_niemoznosci_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_watpliwosci_niemoznosci
BEGIN
  SELECT RAISE(ABORT, 'wpisu niemożności się nie zmienia; błąd poprawia nowy wpis');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabel z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER rejestr_kontroli_watpliwosci_wpisy_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_watpliwosci_wpisy
WHEN EXISTS (
    SELECT 1
    FROM rejestr_kontroli_watpliwosci_wpisy
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM rejestr_kontroli_watpliwosci_wpisy
    WHERE teczka_odnosnik = NEW.teczka_odnosnik
      AND znacznik_odnosnik = NEW.znacznik_odnosnik
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_watpliwosci_niemoznosci_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_watpliwosci_niemoznosci
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_watpliwosci_niemoznosci
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

-- Art. 49 AML i R9: dokumentację przechowuje się co najmniej 5 lat, a na razie bez terminu.
CREATE TRIGGER rejestr_kontroli_watpliwosci_wpisy_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_watpliwosci_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_watpliwosci_niemoznosci_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_watpliwosci_niemoznosci
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;
"#,
];

/// The values of column `niemoznosci.srodek`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NiemoznosciSrodek {
    Identyfikacja,
    Weryfikacja,
    Beneficjent,
    CelICharakter,
    Monitorowanie,
}

impl NiemoznosciSrodek {
    /// Every value, in the order of the catalog.
    pub const ALL: &[NiemoznosciSrodek] = &[
        NiemoznosciSrodek::Identyfikacja,
        NiemoznosciSrodek::Weryfikacja,
        NiemoznosciSrodek::Beneficjent,
        NiemoznosciSrodek::CelICharakter,
        NiemoznosciSrodek::Monitorowanie,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            NiemoznosciSrodek::Identyfikacja => "identyfikacja",
            NiemoznosciSrodek::Weryfikacja => "weryfikacja",
            NiemoznosciSrodek::Beneficjent => "beneficjent",
            NiemoznosciSrodek::CelICharakter => "cel_i_charakter",
            NiemoznosciSrodek::Monitorowanie => "monitorowanie",
        }
    }
}

impl xmeta_room::FromSqlValue for NiemoznosciSrodek {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "identyfikacja" => Ok(NiemoznosciSrodek::Identyfikacja),
                    "weryfikacja" => Ok(NiemoznosciSrodek::Weryfikacja),
                    "beneficjent" => Ok(NiemoznosciSrodek::Beneficjent),
                    "cel_i_charakter" => Ok(NiemoznosciSrodek::CelICharakter),
                    "monitorowanie" => Ok(NiemoznosciSrodek::Monitorowanie),
                    other => Err(format!("{other} is no value of NiemoznosciSrodek")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for NiemoznosciSrodek {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The parameters of `niemoznosci_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NiemoznosciKlientaParams {
    pub klient_id: String,
}

/// A row of `niemoznosci_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct NiemoznosciKlientaRow {
    pub id: i64,
    pub klient_id: String,
    pub srodek: NiemoznosciSrodek,
    pub opis: String,
    pub decyzja_id: String,
    pub ocena_zawiadomienia: String,
    pub raport_odnosnik: String,
    pub wpisal: String,
    pub wpisano: String,
}

impl NiemoznosciKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(NiemoznosciKlientaRow {
            id: row.get("id")?,
            klient_id: row.get("klient_id")?,
            srodek: row.get("srodek")?,
            opis: row.get("opis")?,
            decyzja_id: row.get("decyzja_id")?,
            ocena_zawiadomienia: row.get("ocena_zawiadomienia")?,
            raport_odnosnik: row.get("raport_odnosnik")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Środki, których nie dało się zastosować wobec klienta.
pub fn niemoznosci_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &NiemoznosciKlientaParams,
) -> Result<Vec<NiemoznosciKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_watpliwosci_niemoznosci
WHERE klient_id = :klient_id
ORDER BY id DESC"#,
        parameters: Vec::from([
            ("klient_id", xmeta_room::ToSqlValue::to_sql_value(&params.klient_id)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(NiemoznosciKlientaRow::from_row).collect()
}

/// The parameters of `wpisz_niemoznosc`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszNiemoznoscParams {
    pub klient_id: String,
    pub srodek: NiemoznosciSrodek,
    pub opis: String,
    pub decyzja_id: String,
    pub ocena_zawiadomienia: String,
    pub raport_odnosnik: String,
    pub wpisal: String,
    pub wpisano: String,
}

/// Niemożność zastosowania środka z decyzją o nienawiązaniu albo rozwiązaniu stosunków i oceną,
/// czy zawiadomić Generalnego Inspektora.
pub fn wpisz_niemoznosc(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszNiemoznoscParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_watpliwosci_niemoznosci (klient_id, srodek, opis, decyzja_id,
  ocena_zawiadomienia, raport_odnosnik, wpisal, wpisano)
VALUES (:klient_id, :srodek, :opis, :decyzja_id, :ocena_zawiadomienia, :raport_odnosnik, :wpisal,
  :wpisano)"#,
                parameters: Vec::from([
                    (
                        "klient_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_id),
                    ),
                    ("srodek", xmeta_room::ToSqlValue::to_sql_value(&params.srodek)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    (
                        "decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_id),
                    ),
                    (
                        "ocena_zawiadomienia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.ocena_zawiadomienia),
                    ),
                    (
                        "raport_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.raport_odnosnik),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
        ],
    )
}

/// A row of `watpliwosci`.
#[derive(Debug, Clone, PartialEq)]
pub struct WatpliwosciRow {
    pub id: String,
    pub teczka_odnosnik: String,
    pub znacznik_odnosnik: String,
    pub klient_id: Option<String>,
    pub zglosil: String,
    pub zgloszono: String,
    pub powod: String,
    pub raport_odnosnik: String,
    pub wpisal: String,
    pub wpisano: String,
    pub zdjecie_decyzja_id: Option<String>,
}

impl WatpliwosciRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WatpliwosciRow {
            id: row.get("id")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            znacznik_odnosnik: row.get("znacznik_odnosnik")?,
            klient_id: row.get("klient_id")?,
            zglosil: row.get("zglosil")?,
            zgloszono: row.get("zgloszono")?,
            powod: row.get("powod")?,
            raport_odnosnik: row.get("raport_odnosnik")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
            zdjecie_decyzja_id: row.get("zdjecie_decyzja_id")?,
        })
    }
}

/// Wątpliwości Kontrolerów, najnowsze pierwsze; stojąca nie ma decyzji o zdjęciu.
pub fn watpliwosci(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WatpliwosciRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_watpliwosci_wpisy
ORDER BY zgloszono DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(WatpliwosciRow::from_row).collect()
}

/// The parameters of `wpisz_watpliwosc`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszWatpliwoscParams {
    pub id: String,
    pub teczka_odnosnik: String,
    pub znacznik_odnosnik: String,
    pub klient_id: Option<String>,
    pub zglosil: String,
    pub zgloszono: String,
    pub powod: String,
    pub raport_odnosnik: String,
    pub wpisal: String,
    pub wpisano: String,
}

/// Zgłoszenie wątpliwości z powodem w dniu raportu. Identyfikator (UUID) nadaje aplikacja, bo
/// wskazuje go prośba o zdjęcie.
pub fn wpisz_watpliwosc(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszWatpliwoscParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_watpliwosci_wpisy (id, teczka_odnosnik, znacznik_odnosnik, klient_id,
  zglosil, zgloszono, powod, raport_odnosnik, wpisal, wpisano)
VALUES (:id, :teczka_odnosnik, :znacznik_odnosnik, :klient_id, :zglosil, :zgloszono, :powod,
  :raport_odnosnik, :wpisal, :wpisano)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "teczka_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.teczka_odnosnik),
                    ),
                    (
                        "znacznik_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.znacznik_odnosnik),
                    ),
                    (
                        "klient_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_id),
                    ),
                    ("zglosil", xmeta_room::ToSqlValue::to_sql_value(&params.zglosil)),
                    (
                        "zgloszono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zgloszono),
                    ),
                    ("powod", xmeta_room::ToSqlValue::to_sql_value(&params.powod)),
                    (
                        "raport_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.raport_odnosnik),
                    ),
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
        ],
    )
}

/// The parameters of `wpisz_zdjecie_watpliwosci`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszZdjecieWatpliwosciParams {
    pub zdjecie_decyzja_id: String,
    pub id: String,
}

/// Zdjęcie wątpliwości decyzją osoby wyznaczonej z uzasadnieniem; wpisuje się raz.
pub fn wpisz_zdjecie_watpliwosci(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszZdjecieWatpliwosciParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE rejestr_kontroli_watpliwosci_wpisy
SET zdjecie_decyzja_id = :zdjecie_decyzja_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zdjecie_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zdjecie_decyzja_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
