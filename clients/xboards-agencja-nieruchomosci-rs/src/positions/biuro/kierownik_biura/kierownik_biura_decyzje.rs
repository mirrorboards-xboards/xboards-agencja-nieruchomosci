//! The kind `kierownik-biura-decyzje` of the room `positions/biuro/kierownik_biura`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "kierownik-biura-decyzje";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Listy decyzji: pozycje z próśb kierowników działów o decyzję człowieka, po jednej liście dla
-- każdego, kto decyduje. Samej prośby kierownik nie widzi: pozycja wskazuje jej wiersz w teczce
-- i niesie opis kierownika działu jako jego słowa, a nie rekomendację Kierownika biura. Prośby
-- Kontroli nie trafiają na listę, bo Kontrola przekazuje je właścicielowi sama.
--
-- Ta sama prośba przysłana ponownie odnawia pozycję, a nie ją podwaja (błędne koło). Zdjętych
-- pozycji się nie wykreśla, bo z nich widać, jak długo decyzje czekały.
CREATE TABLE kierownik_biura_decyzje_pozycje (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_biura_decyzje_pozycje_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Prośba o decyzję w teczce, tekstem: pokój i wiersz.
  prosba_odnosnik TEXT NOT NULL CHECK (prosba_odnosnik <> ''),
  dzial TEXT NOT NULL CHECK (dzial IN ('posrednictwo', 'transakcje', 'najem')),
  -- Tożsamość człowieka, który decyduje.
  decyduje TEXT NOT NULL CHECK (decyduje <> ''),
  opis TEXT NOT NULL CHECK (opis <> ''),
  -- Termin podany przez kierownika działu; decyduje o kolejności na liście.
  termin TEXT,
  przyjeto TEXT NOT NULL,
  odnowiono TEXT,
  -- Kiedy pozycja pierwszy raz trafiła do człowieka; kierownik mówi o tym kierownikowi działu.
  trafila_do_czlowieka TEXT,
  -- Kiedy kierownik zgłosił człowiekowi osobno decyzję, która czeka za długo.
  zgloszono_osobno TEXT,
  zdjeto TEXT,
  powod_zdjecia TEXT CHECK (powod_zdjecia IN ('rozstrzygnieta', 'zbedna')),

  CONSTRAINT odnowienie_nie_przed_przyjeciem CHECK (odnowiono IS NULL OR odnowiono >= przyjeto),
  CONSTRAINT do_czlowieka_nie_przed_przyjeciem CHECK (trafila_do_czlowieka IS NULL OR trafila_do_czlowieka >= przyjeto),
  CONSTRAINT zdjecie_z_powodem CHECK ((zdjeto IS NULL) = (powod_zdjecia IS NULL)),
  CONSTRAINT zdjecie_nie_przed_przyjeciem CHECK (zdjeto IS NULL OR zdjeto >= przyjeto)
) STRICT;

-- Jedna niezdjęta pozycja na prośbę; ponowna prośba ją odnawia.
CREATE UNIQUE INDEX kierownik_biura_decyzje_jedna_pozycja_na_prosbe
ON kierownik_biura_decyzje_pozycje (prosba_odnosnik)
WHERE zdjeto IS NULL;

-- Druga niezdjęta pozycja z innym kluczem zderzyłaby się z indeksem, a INSERT OR REPLACE usunąłby
-- wtedy pierwszą z ominięciem blokady usuwania; ten trigger odmawia wcześniej.
CREATE TRIGGER kierownik_biura_decyzje_prosba_raz_na_liscie
BEFORE INSERT ON kierownik_biura_decyzje_pozycje
WHEN EXISTS (
  SELECT 1
  FROM kierownik_biura_decyzje_pozycje
  WHERE prosba_odnosnik = NEW.prosba_odnosnik
    AND zdjeto IS NULL
    AND id IS NOT NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'ta prośba jest już na liście; ponowna prośba odnawia pozycję');
END;

CREATE TRIGGER kierownik_biura_decyzje_powstaje_na_liscie
BEFORE INSERT ON kierownik_biura_decyzje_pozycje
WHEN NEW.zdjeto IS NOT NULL
  OR NEW.odnowiono IS NOT NULL
  OR NEW.trafila_do_czlowieka IS NOT NULL
  OR NEW.zgloszono_osobno IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'pozycja powstaje na liście, jeszcze nieodnowiona i niewysłana');
END;

-- Prośba, dział, człowiek i chwila przyjęcia się nie zmieniają; wysłanie do człowieka i osobne
-- zgłoszenie odnotowuje się raz, a zdjętej pozycji nie zmienia się wcale.
CREATE TRIGGER kierownik_biura_decyzje_bez_zmian
BEFORE UPDATE ON kierownik_biura_decyzje_pozycje
WHEN OLD.zdjeto IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.prosba_odnosnik IS NOT OLD.prosba_odnosnik
  OR NEW.dzial IS NOT OLD.dzial
  OR NEW.decyduje IS NOT OLD.decyduje
  OR NEW.przyjeto IS NOT OLD.przyjeto
  OR (OLD.trafila_do_czlowieka IS NOT NULL AND NEW.trafila_do_czlowieka IS NOT OLD.trafila_do_czlowieka)
  OR (OLD.zgloszono_osobno IS NOT NULL AND NEW.zgloszono_osobno IS NOT OLD.zgloszono_osobno)
BEGIN
  SELECT RAISE(ABORT, 'prośby, działu, człowieka ani przyjęcia pozycji się nie zmienia, a zdjętej pozycji wcale');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_biura_decyzje_pozycje_bez_zastepowania
BEFORE INSERT ON kierownik_biura_decyzje_pozycje
WHEN EXISTS (
  SELECT 1
  FROM kierownik_biura_decyzje_pozycje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: zdjętych pozycji się nie wykreśla.
CREATE TRIGGER kierownik_biura_decyzje_pozycje_bez_usuwania
BEFORE DELETE ON kierownik_biura_decyzje_pozycje
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); zdjętych pozycji się nie wykreśla');
END;
"#,
];

/// The values of column `pozycje.dzial`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PozycjeDzial {
    Posrednictwo,
    Transakcje,
    Najem,
}

impl PozycjeDzial {
    /// Every value, in the order of the catalog.
    pub const ALL: &[PozycjeDzial] = &[
        PozycjeDzial::Posrednictwo,
        PozycjeDzial::Transakcje,
        PozycjeDzial::Najem,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            PozycjeDzial::Posrednictwo => "posrednictwo",
            PozycjeDzial::Transakcje => "transakcje",
            PozycjeDzial::Najem => "najem",
        }
    }
}

impl xmeta_room::FromSqlValue for PozycjeDzial {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "posrednictwo" => Ok(PozycjeDzial::Posrednictwo),
                    "transakcje" => Ok(PozycjeDzial::Transakcje),
                    "najem" => Ok(PozycjeDzial::Najem),
                    other => Err(format!("{other} is no value of PozycjeDzial")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for PozycjeDzial {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `pozycje.powod_zdjecia`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PozycjePowodZdjecia {
    Rozstrzygnieta,
    Zbedna,
}

impl PozycjePowodZdjecia {
    /// Every value, in the order of the catalog.
    pub const ALL: &[PozycjePowodZdjecia] = &[
        PozycjePowodZdjecia::Rozstrzygnieta,
        PozycjePowodZdjecia::Zbedna,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            PozycjePowodZdjecia::Rozstrzygnieta => "rozstrzygnieta",
            PozycjePowodZdjecia::Zbedna => "zbedna",
        }
    }
}

impl xmeta_room::FromSqlValue for PozycjePowodZdjecia {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "rozstrzygnieta" => Ok(PozycjePowodZdjecia::Rozstrzygnieta),
                    "zbedna" => Ok(PozycjePowodZdjecia::Zbedna),
                    other => Err(format!("{other} is no value of PozycjePowodZdjecia")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for PozycjePowodZdjecia {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The parameters of `lista_decyzji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ListaDecyzjiParams {
    pub decyduje: String,
    pub bliski_termin: String,
}

/// A row of `lista_decyzji`.
#[derive(Debug, Clone, PartialEq)]
pub struct ListaDecyzjiRow {
    pub id: i64,
    pub prosba_odnosnik: String,
    pub dzial: PozycjeDzial,
    pub decyduje: String,
    pub opis: String,
    pub termin: Option<String>,
    pub przyjeto: String,
    pub odnowiono: Option<String>,
    pub trafila_do_czlowieka: Option<String>,
    pub zgloszono_osobno: Option<String>,
    pub zdjeto: Option<String>,
    pub powod_zdjecia: Option<PozycjePowodZdjecia>,
}

impl ListaDecyzjiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ListaDecyzjiRow {
            id: row.get("id")?,
            prosba_odnosnik: row.get("prosba_odnosnik")?,
            dzial: row.get("dzial")?,
            decyduje: row.get("decyduje")?,
            opis: row.get("opis")?,
            termin: row.get("termin")?,
            przyjeto: row.get("przyjeto")?,
            odnowiono: row.get("odnowiono")?,
            trafila_do_czlowieka: row.get("trafila_do_czlowieka")?,
            zgloszono_osobno: row.get("zgloszono_osobno")?,
            zdjeto: row.get("zdjeto")?,
            powod_zdjecia: row.get("powod_zdjecia")?,
        })
    }
}

/// Lista jednego człowieka: najpierw pozycje z terminem nie późniejszym niż `bliski_termin`, według
/// terminu, potem pozostałe od najstarszej.
pub fn lista_decyzji(
    room: &impl xmeta_room::MetaRoom,
    params: &ListaDecyzjiParams,
) -> Result<Vec<ListaDecyzjiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_biura_decyzje_pozycje
WHERE decyduje = :decyduje
  AND zdjeto IS NULL
ORDER BY (termin IS NOT NULL AND termin <= :bliski_termin) DESC,
  CASE
    WHEN termin IS NOT NULL AND termin <= :bliski_termin THEN termin
  END,
  przyjeto,
  id"#,
        parameters: Vec::from([
            ("decyduje", xmeta_room::ToSqlValue::to_sql_value(&params.decyduje)),
            (
                "bliski_termin",
                xmeta_room::ToSqlValue::to_sql_value(&params.bliski_termin),
            ),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(ListaDecyzjiRow::from_row).collect()
}

/// The parameters of `pozycje_czekajace_od`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PozycjeCzekajaceOdParams {
    pub przed: String,
}

/// A row of `pozycje_czekajace_od`.
#[derive(Debug, Clone, PartialEq)]
pub struct PozycjeCzekajaceOdRow {
    pub id: i64,
    pub prosba_odnosnik: String,
    pub dzial: PozycjeDzial,
    pub decyduje: String,
    pub opis: String,
    pub termin: Option<String>,
    pub przyjeto: String,
    pub odnowiono: Option<String>,
    pub trafila_do_czlowieka: Option<String>,
    pub zgloszono_osobno: Option<String>,
    pub zdjeto: Option<String>,
    pub powod_zdjecia: Option<PozycjePowodZdjecia>,
}

impl PozycjeCzekajaceOdRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PozycjeCzekajaceOdRow {
            id: row.get("id")?,
            prosba_odnosnik: row.get("prosba_odnosnik")?,
            dzial: row.get("dzial")?,
            decyduje: row.get("decyduje")?,
            opis: row.get("opis")?,
            termin: row.get("termin")?,
            przyjeto: row.get("przyjeto")?,
            odnowiono: row.get("odnowiono")?,
            trafila_do_czlowieka: row.get("trafila_do_czlowieka")?,
            zgloszono_osobno: row.get("zgloszono_osobno")?,
            zdjeto: row.get("zdjeto")?,
            powod_zdjecia: row.get("powod_zdjecia")?,
        })
    }
}

/// Niezdjęte pozycje przyjęte przed jedną chwilą, od najstarszej; to decyzje, które czekają za długo.
pub fn pozycje_czekajace_od(
    room: &impl xmeta_room::MetaRoom,
    params: &PozycjeCzekajaceOdParams,
) -> Result<Vec<PozycjeCzekajaceOdRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_biura_decyzje_pozycje
WHERE zdjeto IS NULL
  AND przyjeto < :przed
ORDER BY przyjeto, id"#,
        parameters: Vec::from([
            ("przed", xmeta_room::ToSqlValue::to_sql_value(&params.przed)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(PozycjeCzekajaceOdRow::from_row).collect()
}

/// The parameters of `pozycje_zdjete_od`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PozycjeZdjeteOdParams {
    pub od: String,
}

/// A row of `pozycje_zdjete_od`.
#[derive(Debug, Clone, PartialEq)]
pub struct PozycjeZdjeteOdRow {
    pub id: i64,
    pub prosba_odnosnik: String,
    pub dzial: PozycjeDzial,
    pub decyduje: String,
    pub opis: String,
    pub termin: Option<String>,
    pub przyjeto: String,
    pub odnowiono: Option<String>,
    pub trafila_do_czlowieka: Option<String>,
    pub zgloszono_osobno: Option<String>,
    pub zdjeto: Option<String>,
    pub powod_zdjecia: Option<PozycjePowodZdjecia>,
}

impl PozycjeZdjeteOdRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PozycjeZdjeteOdRow {
            id: row.get("id")?,
            prosba_odnosnik: row.get("prosba_odnosnik")?,
            dzial: row.get("dzial")?,
            decyduje: row.get("decyduje")?,
            opis: row.get("opis")?,
            termin: row.get("termin")?,
            przyjeto: row.get("przyjeto")?,
            odnowiono: row.get("odnowiono")?,
            trafila_do_czlowieka: row.get("trafila_do_czlowieka")?,
            zgloszono_osobno: row.get("zgloszono_osobno")?,
            zdjeto: row.get("zdjeto")?,
            powod_zdjecia: row.get("powod_zdjecia")?,
        })
    }
}

/// Pozycje zdjęte od jednego dnia; z nich widać czas oczekiwania decyzji w sprawozdaniu.
pub fn pozycje_zdjete_od(
    room: &impl xmeta_room::MetaRoom,
    params: &PozycjeZdjeteOdParams,
) -> Result<Vec<PozycjeZdjeteOdRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_biura_decyzje_pozycje
WHERE zdjeto >= :od
ORDER BY zdjeto, id"#,
        parameters: Vec::from([("od", xmeta_room::ToSqlValue::to_sql_value(&params.od))]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(PozycjeZdjeteOdRow::from_row).collect()
}

/// The parameters of `przyjmij_pozycje`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzyjmijPozycjeParams {
    pub prosba_odnosnik: String,
    pub dzial: PozycjeDzial,
    pub decyduje: String,
    pub opis: String,
    pub termin: Option<String>,
    pub przyjeto: String,
}

/// Nowa pozycja z prośby kierownika działu.
pub fn przyjmij_pozycje(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzyjmijPozycjeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_biura_decyzje_pozycje (prosba_odnosnik, dzial, decyduje, opis, termin,
  przyjeto)
VALUES (:prosba_odnosnik, :dzial, :decyduje, :opis, :termin, :przyjeto)"#,
                parameters: Vec::from([
                    (
                        "prosba_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.prosba_odnosnik),
                    ),
                    ("dzial", xmeta_room::ToSqlValue::to_sql_value(&params.dzial)),
                    ("decyduje", xmeta_room::ToSqlValue::to_sql_value(&params.decyduje)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    ("termin", xmeta_room::ToSqlValue::to_sql_value(&params.termin)),
                    ("przyjeto", xmeta_room::ToSqlValue::to_sql_value(&params.przyjeto)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnow_pozycje`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnowPozycjeParams {
    pub opis: String,
    pub termin: Option<String>,
    pub odnowiono: String,
    pub id: i64,
}

/// Ta sama prośba przyszła ponownie: pozycja dostaje nowy opis i termin, ale się nie podwaja.
pub fn odnow_pozycje(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnowPozycjeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_biura_decyzje_pozycje
SET opis = :opis,
  termin = :termin,
  odnowiono = :odnowiono
WHERE id = :id"#,
                parameters: Vec::from([
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    ("termin", xmeta_room::ToSqlValue::to_sql_value(&params.termin)),
                    (
                        "odnowiono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odnowiono),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_wyslanie_pozycji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujWyslaniePozycjiParams {
    pub trafila_do_czlowieka: String,
    pub id: i64,
}

/// Pozycja pierwszy raz trafiła do człowieka na jego liście.
pub fn odnotuj_wyslanie_pozycji(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWyslaniePozycjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_biura_decyzje_pozycje
SET trafila_do_czlowieka = :trafila_do_czlowieka
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "trafila_do_czlowieka",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.trafila_do_czlowieka,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_osobne_zgloszenie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujOsobneZgloszenieParams {
    pub zgloszono_osobno: String,
    pub id: i64,
}

/// Decyzję, która czeka za długo, kierownik zgłosił człowiekowi osobno.
pub fn odnotuj_osobne_zgloszenie(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujOsobneZgloszenieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_biura_decyzje_pozycje
SET zgloszono_osobno = :zgloszono_osobno
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zgloszono_osobno",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zgloszono_osobno),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `zdejmij_pozycje`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZdejmijPozycjeParams {
    pub zdjeto: String,
    pub powod_zdjecia: PozycjePowodZdjecia,
    pub id: i64,
}

/// Kierownik działu zgłosił, że prośba jest rozstrzygnięta albo zbędna.
pub fn zdejmij_pozycje(
    room: &impl xmeta_room::MetaRoom,
    params: &ZdejmijPozycjeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_biura_decyzje_pozycje
SET zdjeto = :zdjeto,
  powod_zdjecia = :powod_zdjecia
WHERE id = :id"#,
                parameters: Vec::from([
                    ("zdjeto", xmeta_room::ToSqlValue::to_sql_value(&params.zdjeto)),
                    (
                        "powod_zdjecia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.powod_zdjecia),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
