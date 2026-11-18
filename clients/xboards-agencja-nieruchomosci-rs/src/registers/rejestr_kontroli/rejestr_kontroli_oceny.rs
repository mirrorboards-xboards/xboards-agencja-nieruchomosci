//! The kind `rejestr-kontroli-oceny` of the room `registers/rejestr_kontroli`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "rejestr-kontroli-oceny";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Ocena ryzyka klienta (art. 33 AML) i ocena ryzyka całego biura (art. 27 AML). Ocenę klienta
-- proponuje Kontroler, Kierownik kontroli wpisuje ją do zatwierdzenia, a zatwierdza osoba
-- wyznaczona do obowiązków z ustawy decyzją podpisaną kluczem. Gdy osoba wyznaczona ocenę zmienia,
-- Kierownik wpisuje nową ocenę z jej poziomem i środkami.
--
-- Ponowna ocena jest nowym wpisem i odbiera aktualność poprzedniej. Wersję oceny biura Kierownik
-- wpisuje dopiero po akceptacji właściciela biura. Ocen się nie zmienia; jedyne zmiany to
-- utrata aktualności oceny klienta i jednorazowy wpis jej zatwierdzenia.
CREATE TABLE rejestr_kontroli_oceny_klientow (
  id TEXT PRIMARY KEY,
  klient_id TEXT NOT NULL REFERENCES rejestr_kontroli_klienci_wpisy (id),
  poziom TEXT NOT NULL CHECK (poziom IN ('niskie', 'srednie', 'wysokie')),
  -- Czynniki z art. 33 ust. 3 AML: rodzaj klienta, obszar geograficzny, wartość transakcji, cel
  -- i czas trwania stosunków.
  czynniki TEXT NOT NULL CHECK (czynniki <> ''),
  srodki TEXT NOT NULL CHECK (srodki IN ('zwykle', 'uproszczone', 'wzmozone')),
  -- Kontroler, który zaproponował ocenę, i jego raport.
  zaproponowal TEXT NOT NULL CHECK (zaproponowal <> ''),
  raport_odnosnik TEXT NOT NULL CHECK (raport_odnosnik <> ''),
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,
  aktualna INTEGER NOT NULL CHECK (aktualna IN (0, 1)),
  -- Decyzja zatwierdzenie_oceny osoby wyznaczonej o tej ocenie, przed umową z klientem.
  zatwierdzenie_decyzja_id TEXT REFERENCES rejestr_kontroli_decyzje_wpisy (id),

  -- Art. 42 ust. 1 AML: uproszczone środki tylko przy niższym ryzyku; art. 43 ust. 1 AML: przy
  -- wyższym ryzyku środki wzmożone.
  CONSTRAINT uproszczone_tylko_przy_niskim CHECK (srodki <> 'uproszczone' OR poziom = 'niskie'),
  CONSTRAINT wzmozone_przy_wysokim CHECK (poziom <> 'wysokie' OR srodki = 'wzmozone')
) STRICT;

CREATE TABLE rejestr_kontroli_oceny_biura (
  id TEXT PRIMARY KEY,
  -- Kolejne numery od 1.
  wersja INTEGER NOT NULL UNIQUE CHECK (wersja >= 1),
  -- Czynniki z art. 27 ust. 1 AML: klienci, państwa i obszary, produkty, usługi, transakcje
  -- i kanały dostaw.
  czynniki TEXT NOT NULL CHECK (czynniki <> ''),
  sporzadzono TEXT NOT NULL,
  -- Dzień, do którego trzeba ocenę zaktualizować, nie później niż po 2 latach (art. 27 ust. 3 AML).
  aktualizacja_do TEXT NOT NULL,
  dokument_sciezka TEXT NOT NULL CHECK (dokument_sciezka <> ''),
  dokument_skrot TEXT NOT NULL CHECK (dokument_skrot <> ''),
  -- Akceptacja właściciela biura; wymaga jej reguła biura, a nie ustawa.
  akceptacja_decyzja_id TEXT NOT NULL REFERENCES rejestr_kontroli_decyzje_wpisy (id),
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,

  CONSTRAINT aktualizacja_najpozniej_po_2_latach CHECK (
    aktualizacja_do > sporzadzono
    AND aktualizacja_do <= date(sporzadzono, '+2 years')
  )
) STRICT;

-- Ocena klienta ------------------------------------------------------------------------------------

CREATE TRIGGER rejestr_kontroli_oceny_klientow_powstaje_aktualna
BEFORE INSERT ON rejestr_kontroli_oceny_klientow
WHEN NEW.aktualna <> 1
BEGIN
  SELECT RAISE(ABORT, 'ocena ryzyka klienta powstaje jako aktualna');
END;

CREATE TRIGGER rejestr_kontroli_oceny_klientow_nowa_zeruje_poprzednia
AFTER INSERT ON rejestr_kontroli_oceny_klientow
BEGIN
  UPDATE rejestr_kontroli_oceny_klientow
  SET aktualna = 0
  WHERE klient_id = NEW.klient_id
    AND id <> NEW.id
    AND aktualna = 1;
END;

CREATE TRIGGER rejestr_kontroli_oceny_klientow_zatwierdzenie_wskazuje_decyzje
BEFORE INSERT ON rejestr_kontroli_oceny_klientow
WHEN NEW.zatwierdzenie_decyzja_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM rejestr_kontroli_decyzje_zatwierdzone
    WHERE id = NEW.zatwierdzenie_decyzja_id
      AND rodzaj = 'zatwierdzenie_oceny'
      AND dotyczy_tabela = 'rejestr_kontroli_oceny_klientow'
      AND dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'zatwierdzenie oceny wskazuje zatwierdzoną decyzję zatwierdzenie_oceny o tej ocenie');
END;

CREATE TRIGGER rejestr_kontroli_oceny_klientow_wpisane_zatwierdzenie_wskazuje_decyzje
BEFORE UPDATE OF zatwierdzenie_decyzja_id ON rejestr_kontroli_oceny_klientow
WHEN NEW.zatwierdzenie_decyzja_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM rejestr_kontroli_decyzje_zatwierdzone
    WHERE id = NEW.zatwierdzenie_decyzja_id
      AND rodzaj = 'zatwierdzenie_oceny'
      AND dotyczy_tabela = 'rejestr_kontroli_oceny_klientow'
      AND dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'zatwierdzenie oceny wskazuje zatwierdzoną decyzję zatwierdzenie_oceny o tej ocenie');
END;

-- Zatwierdza się aktualną ocenę, raz. Aktualność traci ocena, którą zastąpiła nowsza.
CREATE TRIGGER rejestr_kontroli_oceny_klientow_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_oceny_klientow
WHEN NEW.id IS NOT OLD.id
  OR NEW.klient_id IS NOT OLD.klient_id
  OR NEW.poziom IS NOT OLD.poziom
  OR NEW.czynniki IS NOT OLD.czynniki
  OR NEW.srodki IS NOT OLD.srodki
  OR NEW.zaproponowal IS NOT OLD.zaproponowal
  OR NEW.raport_odnosnik IS NOT OLD.raport_odnosnik
  OR NEW.wpisal IS NOT OLD.wpisal
  OR NEW.wpisano IS NOT OLD.wpisano
  OR (
    NEW.zatwierdzenie_decyzja_id IS NOT OLD.zatwierdzenie_decyzja_id
    AND (OLD.zatwierdzenie_decyzja_id IS NOT NULL OR OLD.aktualna = 0)
  )
  OR (
    NEW.aktualna IS NOT OLD.aktualna
    AND NOT (
      OLD.aktualna = 1
      AND NEW.aktualna = 0
      AND EXISTS (
        SELECT 1
        FROM rejestr_kontroli_oceny_klientow
        WHERE klient_id = OLD.klient_id
          AND id <> OLD.id
          AND aktualna = 1
      )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'oceny ryzyka klienta się nie zmienia; zatwierdza się raz aktualną ocenę, a ponowna ocena to nowy wpis');
END;

-- Ocena biura --------------------------------------------------------------------------------------

CREATE TRIGGER rejestr_kontroli_oceny_biura_wersja_kolejna
BEFORE INSERT ON rejestr_kontroli_oceny_biura
WHEN NEW.wersja IS NOT (
  SELECT coalesce(max(wersja), 0) + 1
  FROM rejestr_kontroli_oceny_biura
)
BEGIN
  SELECT RAISE(ABORT, 'wersja oceny ryzyka biura ma kolejny numer');
END;

CREATE TRIGGER rejestr_kontroli_oceny_biura_akceptacja_wskazuje_decyzje
BEFORE INSERT ON rejestr_kontroli_oceny_biura
WHEN NOT EXISTS (
  SELECT 1
  FROM rejestr_kontroli_decyzje_zatwierdzone
  WHERE id = NEW.akceptacja_decyzja_id
    AND rodzaj = 'akceptacja_oceny_biura'
    AND dotyczy_tabela = 'rejestr_kontroli_oceny_biura'
    AND dotyczy_id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'ocena ryzyka biura wskazuje zatwierdzoną decyzję akceptacja_oceny_biura o tej wersji');
END;

CREATE TRIGGER rejestr_kontroli_oceny_biura_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_oceny_biura
BEGIN
  SELECT RAISE(ABORT, 'oceny ryzyka biura się nie zmienia; aktualizacja to nowa wersja');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabel z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER rejestr_kontroli_oceny_klientow_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_oceny_klientow
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_oceny_klientow
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_oceny_biura_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_oceny_biura
WHEN EXISTS (
    SELECT 1
    FROM rejestr_kontroli_oceny_biura
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM rejestr_kontroli_oceny_biura
    WHERE wersja = NEW.wersja
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

-- Art. 49 AML i R9: dokumentację przechowuje się co najmniej 5 lat, a na razie bez terminu.
CREATE TRIGGER rejestr_kontroli_oceny_klientow_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_oceny_klientow
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_oceny_biura_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_oceny_biura
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;
"#,
];

/// The values of column `klientow.poziom`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KlientowPoziom {
    Niskie,
    Srednie,
    Wysokie,
}

impl KlientowPoziom {
    /// Every value, in the order of the catalog.
    pub const ALL: &[KlientowPoziom] = &[
        KlientowPoziom::Niskie,
        KlientowPoziom::Srednie,
        KlientowPoziom::Wysokie,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            KlientowPoziom::Niskie => "niskie",
            KlientowPoziom::Srednie => "srednie",
            KlientowPoziom::Wysokie => "wysokie",
        }
    }
}

impl xmeta_room::FromSqlValue for KlientowPoziom {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "niskie" => Ok(KlientowPoziom::Niskie),
                    "srednie" => Ok(KlientowPoziom::Srednie),
                    "wysokie" => Ok(KlientowPoziom::Wysokie),
                    other => Err(format!("{other} is no value of KlientowPoziom")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for KlientowPoziom {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `klientow.srodki`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KlientowSrodki {
    Zwykle,
    Uproszczone,
    Wzmozone,
}

impl KlientowSrodki {
    /// Every value, in the order of the catalog.
    pub const ALL: &[KlientowSrodki] = &[
        KlientowSrodki::Zwykle,
        KlientowSrodki::Uproszczone,
        KlientowSrodki::Wzmozone,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            KlientowSrodki::Zwykle => "zwykle",
            KlientowSrodki::Uproszczone => "uproszczone",
            KlientowSrodki::Wzmozone => "wzmozone",
        }
    }
}

impl xmeta_room::FromSqlValue for KlientowSrodki {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "zwykle" => Ok(KlientowSrodki::Zwykle),
                    "uproszczone" => Ok(KlientowSrodki::Uproszczone),
                    "wzmozone" => Ok(KlientowSrodki::Wzmozone),
                    other => Err(format!("{other} is no value of KlientowSrodki")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for KlientowSrodki {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `oceny_ryzyka_biura`.
#[derive(Debug, Clone, PartialEq)]
pub struct OcenyRyzykaBiuraRow {
    pub id: String,
    pub wersja: i64,
    pub czynniki: String,
    pub sporzadzono: String,
    pub aktualizacja_do: String,
    pub dokument_sciezka: String,
    pub dokument_skrot: String,
    pub akceptacja_decyzja_id: String,
    pub wpisal: String,
    pub wpisano: String,
}

impl OcenyRyzykaBiuraRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(OcenyRyzykaBiuraRow {
            id: row.get("id")?,
            wersja: row.get("wersja")?,
            czynniki: row.get("czynniki")?,
            sporzadzono: row.get("sporzadzono")?,
            aktualizacja_do: row.get("aktualizacja_do")?,
            dokument_sciezka: row.get("dokument_sciezka")?,
            dokument_skrot: row.get("dokument_skrot")?,
            akceptacja_decyzja_id: row.get("akceptacja_decyzja_id")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Wersje oceny ryzyka biura, najnowsza pierwsza.
pub fn oceny_ryzyka_biura(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<OcenyRyzykaBiuraRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_oceny_biura
ORDER BY wersja DESC"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(OcenyRyzykaBiuraRow::from_row).collect()
}

/// The parameters of `wpisz_ocene_ryzyka_biura`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszOceneRyzykaBiuraParams {
    pub id: String,
    pub wersja: i64,
    pub czynniki: String,
    pub sporzadzono: String,
    pub aktualizacja_do: String,
    pub dokument_sciezka: String,
    pub dokument_skrot: String,
    pub akceptacja_decyzja_id: String,
    pub wpisal: String,
    pub wpisano: String,
}

/// Nowa wersja oceny ryzyka biura po akceptacji właściciela biura. Identyfikator (UUID) nadaje
/// aplikacja, zanim powstanie prośba o akceptację.
pub fn wpisz_ocene_ryzyka_biura(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszOceneRyzykaBiuraParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_oceny_biura (id, wersja, czynniki, sporzadzono, aktualizacja_do,
  dokument_sciezka, dokument_skrot, akceptacja_decyzja_id, wpisal, wpisano)
VALUES (:id, :wersja, :czynniki, :sporzadzono, :aktualizacja_do, :dokument_sciezka, :dokument_skrot,
  :akceptacja_decyzja_id, :wpisal, :wpisano)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("wersja", xmeta_room::ToSqlValue::to_sql_value(&params.wersja)),
                    ("czynniki", xmeta_room::ToSqlValue::to_sql_value(&params.czynniki)),
                    (
                        "sporzadzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sporzadzono),
                    ),
                    (
                        "aktualizacja_do",
                        xmeta_room::ToSqlValue::to_sql_value(&params.aktualizacja_do),
                    ),
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
                    ("wpisal", xmeta_room::ToSqlValue::to_sql_value(&params.wpisal)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
        ],
    )
}

/// The parameters of `oceny_ryzyka_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OcenyRyzykaKlientaParams {
    pub klient_id: String,
}

/// A row of `oceny_ryzyka_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct OcenyRyzykaKlientaRow {
    pub id: String,
    pub klient_id: String,
    pub poziom: KlientowPoziom,
    pub czynniki: String,
    pub srodki: KlientowSrodki,
    pub zaproponowal: String,
    pub raport_odnosnik: String,
    pub wpisal: String,
    pub wpisano: String,
    pub aktualna: bool,
    pub zatwierdzenie_decyzja_id: Option<String>,
}

impl OcenyRyzykaKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(OcenyRyzykaKlientaRow {
            id: row.get("id")?,
            klient_id: row.get("klient_id")?,
            poziom: row.get("poziom")?,
            czynniki: row.get("czynniki")?,
            srodki: row.get("srodki")?,
            zaproponowal: row.get("zaproponowal")?,
            raport_odnosnik: row.get("raport_odnosnik")?,
            wpisal: row.get("wpisal")?,
            wpisano: row.get("wpisano")?,
            aktualna: row.get("aktualna")?,
            zatwierdzenie_decyzja_id: row.get("zatwierdzenie_decyzja_id")?,
        })
    }
}

/// Oceny ryzyka klienta, najnowsza pierwsza; bieżąca ma aktualna = 1.
pub fn oceny_ryzyka_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &OcenyRyzykaKlientaParams,
) -> Result<Vec<OcenyRyzykaKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM rejestr_kontroli_oceny_klientow
WHERE klient_id = :klient_id
ORDER BY wpisano DESC, id"#,
        parameters: Vec::from([
            ("klient_id", xmeta_room::ToSqlValue::to_sql_value(&params.klient_id)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(OcenyRyzykaKlientaRow::from_row).collect()
}

/// The parameters of `wpisz_ocene_ryzyka_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszOceneRyzykaKlientaParams {
    pub id: String,
    pub klient_id: String,
    pub poziom: KlientowPoziom,
    pub czynniki: String,
    pub srodki: KlientowSrodki,
    pub zaproponowal: String,
    pub raport_odnosnik: String,
    pub wpisal: String,
    pub wpisano: String,
}

/// Propozycja Kontrolera do zatwierdzenia; poprzednia ocena klienta traci aktualność.
/// Identyfikator (UUID) nadaje aplikacja, bo wskazuje go prośba o zatwierdzenie.
pub fn wpisz_ocene_ryzyka_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszOceneRyzykaKlientaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO rejestr_kontroli_oceny_klientow (id, klient_id, poziom, czynniki, srodki, zaproponowal,
  raport_odnosnik, wpisal, wpisano, aktualna)
VALUES (:id, :klient_id, :poziom, :czynniki, :srodki, :zaproponowal, :raport_odnosnik, :wpisal,
  :wpisano, 1)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "klient_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_id),
                    ),
                    ("poziom", xmeta_room::ToSqlValue::to_sql_value(&params.poziom)),
                    ("czynniki", xmeta_room::ToSqlValue::to_sql_value(&params.czynniki)),
                    ("srodki", xmeta_room::ToSqlValue::to_sql_value(&params.srodki)),
                    (
                        "zaproponowal",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zaproponowal),
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

/// The parameters of `wpisz_zatwierdzenie_oceny`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszZatwierdzenieOcenyParams {
    pub zatwierdzenie_decyzja_id: String,
    pub id: String,
}

/// Zatwierdzenie aktualnej oceny decyzją osoby wyznaczonej; wpisuje się raz.
pub fn wpisz_zatwierdzenie_oceny(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszZatwierdzenieOcenyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE rejestr_kontroli_oceny_klientow
SET zatwierdzenie_decyzja_id = :zatwierdzenie_decyzja_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zatwierdzenie_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.zatwierdzenie_decyzja_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
