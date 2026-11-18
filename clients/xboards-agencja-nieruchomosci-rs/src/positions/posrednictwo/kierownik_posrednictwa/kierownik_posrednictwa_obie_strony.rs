//! The kind `kierownik-posrednictwa-obie-strony` of the room `positions/posrednictwo/kierownik_posrednictwa`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "kierownik-posrednictwa-obie-strony",
    requires: &[],
    migrations: &[
        r#"-- Obie strony (rozstrzygnięcie biura R1): pary teczki oferty i teczki poszukiwania, które spotkały
-- się w jednej transakcji, i to, czy umowy pośrednictwa obu stron pozwalają biuru działać dla
-- drugiej strony.
--
-- Wiersz trzyma tylko odnośniki do teczek i umów, bez cen, budżetów ani danych klientów, bo
-- kierownik nie zestawia danych obu stron. Para powstaje do wyjaśnienia, a „tak” wymaga wskazania
-- obu umów. Ofertę przekazuje się dalej do Transakcji bez zgody obu umów tylko za zgodą człowieka;
-- jego podpis kierownik przenosi do wiersza. Po przekazaniu ocena umów się nie zmienia.
CREATE TABLE kierownik_posrednictwa_obie_strony_pary (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_posrednictwa_obie_strony_pary_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Pokoje teczek obu stron, tekstem.
  teczka_oferty_odnosnik TEXT NOT NULL CHECK (teczka_oferty_odnosnik <> ''),
  teczka_poszukiwania_odnosnik TEXT NOT NULL CHECK (teczka_poszukiwania_odnosnik <> ''),
  stwierdzono TEXT NOT NULL,
  umowy_pozwalaja TEXT NOT NULL CHECK (umowy_pozwalaja IN ('tak', 'nie', 'do_wyjasnienia')),
  -- Umowy pośrednictwa w obu teczkach, tekstem: pokój i wiersz.
  umowa_oferty_odnosnik TEXT,
  umowa_poszukiwania_odnosnik TEXT,
  -- Kiedy kierownik przekazał przyjętą ofertę Kierownikowi transakcji, i teczka transakcji.
  przekazano TEXT,
  transakcja_odnosnik TEXT,
  zatwierdzil TEXT,
  podpis TEXT,
  zatwierdzono TEXT,

  CONSTRAINT para_teczek_raz UNIQUE (teczka_oferty_odnosnik, teczka_poszukiwania_odnosnik),
  CONSTRAINT zgoda_umow_wskazuje_obie_umowy CHECK (
    umowy_pozwalaja <> 'tak'
    OR (umowa_oferty_odnosnik IS NOT NULL AND umowa_poszukiwania_odnosnik IS NOT NULL)
  ),
  CONSTRAINT przekazanie_za_zgoda_umow_albo_czlowieka CHECK (
    przekazano IS NULL
    OR umowy_pozwalaja = 'tak'
    OR zatwierdzil IS NOT NULL
  ),
  CONSTRAINT transakcja_po_przekazaniu CHECK (transakcja_odnosnik IS NULL OR przekazano IS NOT NULL),
  CONSTRAINT przekazanie_nie_przed_stwierdzeniem CHECK (przekazano IS NULL OR przekazano >= stwierdzono),
  CONSTRAINT zatwierdzenie_w_komplecie CHECK (
    (zatwierdzil IS NULL) = (podpis IS NULL)
    AND (zatwierdzil IS NULL) = (zatwierdzono IS NULL)
  )
) STRICT;

CREATE TRIGGER kierownik_posrednictwa_obie_strony_para_powstaje_do_wyjasnienia
BEFORE INSERT ON kierownik_posrednictwa_obie_strony_pary
WHEN NEW.umowy_pozwalaja <> 'do_wyjasnienia'
  OR NEW.przekazano IS NOT NULL
  OR NEW.transakcja_odnosnik IS NOT NULL
  OR NEW.zatwierdzil IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'para powstaje do wyjaśnienia i nieprzekazana');
END;

-- Teczki pary się nie zmieniają; po przekazaniu nie zmienia się też ocena umów ani zgoda
-- człowieka, a wolno raz wpisać teczkę transakcji.
CREATE TRIGGER kierownik_posrednictwa_obie_strony_bez_zmian
BEFORE UPDATE ON kierownik_posrednictwa_obie_strony_pary
WHEN NEW.id IS NOT OLD.id
  OR NEW.teczka_oferty_odnosnik IS NOT OLD.teczka_oferty_odnosnik
  OR NEW.teczka_poszukiwania_odnosnik IS NOT OLD.teczka_poszukiwania_odnosnik
  OR NEW.stwierdzono IS NOT OLD.stwierdzono
  OR (OLD.przekazano IS NOT NULL AND (
    NEW.umowy_pozwalaja IS NOT OLD.umowy_pozwalaja
    OR NEW.umowa_oferty_odnosnik IS NOT OLD.umowa_oferty_odnosnik
    OR NEW.umowa_poszukiwania_odnosnik IS NOT OLD.umowa_poszukiwania_odnosnik
    OR NEW.przekazano IS NOT OLD.przekazano
    OR NEW.zatwierdzil IS NOT OLD.zatwierdzil
    OR NEW.podpis IS NOT OLD.podpis
    OR NEW.zatwierdzono IS NOT OLD.zatwierdzono
  ))
  OR (OLD.transakcja_odnosnik IS NOT NULL AND NEW.transakcja_odnosnik IS NOT OLD.transakcja_odnosnik)
BEGIN
  SELECT RAISE(ABORT, 'teczek pary się nie zmienia, a po przekazaniu także oceny umów; teczkę transakcji wpisuje się raz');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers; także parę tych samych
-- teczek. Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER kierownik_posrednictwa_obie_strony_pary_bez_zastepowania
BEFORE INSERT ON kierownik_posrednictwa_obie_strony_pary
WHEN EXISTS (
  SELECT 1
  FROM kierownik_posrednictwa_obie_strony_pary
  WHERE id = NEW.id
    OR (
      teczka_oferty_odnosnik = NEW.teczka_oferty_odnosnik
      AND teczka_poszukiwania_odnosnik = NEW.teczka_poszukiwania_odnosnik
    )
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: pary zostają jako ślad transakcji prowadzonych dla obu stron.
CREATE TRIGGER kierownik_posrednictwa_obie_strony_pary_bez_usuwania
BEFORE DELETE ON kierownik_posrednictwa_obie_strony_pary
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
    ],
};

/// The values of column `pary.umowy_pozwalaja`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParyUmowyPozwalaja {
    Tak,
    Nie,
    DoWyjasnienia,
}

impl ParyUmowyPozwalaja {
    /// Every value, in the order of the catalog.
    pub const ALL: &[ParyUmowyPozwalaja] = &[
        ParyUmowyPozwalaja::Tak,
        ParyUmowyPozwalaja::Nie,
        ParyUmowyPozwalaja::DoWyjasnienia,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            ParyUmowyPozwalaja::Tak => "tak",
            ParyUmowyPozwalaja::Nie => "nie",
            ParyUmowyPozwalaja::DoWyjasnienia => "do_wyjasnienia",
        }
    }
}

impl xmeta_room::FromSqlValue for ParyUmowyPozwalaja {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "tak" => Ok(ParyUmowyPozwalaja::Tak),
                    "nie" => Ok(ParyUmowyPozwalaja::Nie),
                    "do_wyjasnienia" => Ok(ParyUmowyPozwalaja::DoWyjasnienia),
                    other => Err(format!("{other} is no value of ParyUmowyPozwalaja")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for ParyUmowyPozwalaja {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `pary`.
#[derive(Debug, Clone, PartialEq)]
pub struct ParyRow {
    pub id: i64,
    pub teczka_oferty_odnosnik: String,
    pub teczka_poszukiwania_odnosnik: String,
    pub stwierdzono: String,
    pub umowy_pozwalaja: ParyUmowyPozwalaja,
    pub umowa_oferty_odnosnik: Option<String>,
    pub umowa_poszukiwania_odnosnik: Option<String>,
    pub przekazano: Option<String>,
    pub transakcja_odnosnik: Option<String>,
    pub zatwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub zatwierdzono: Option<String>,
}

impl ParyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ParyRow {
            id: row.get("id")?,
            teczka_oferty_odnosnik: row.get("teczka_oferty_odnosnik")?,
            teczka_poszukiwania_odnosnik: row.get("teczka_poszukiwania_odnosnik")?,
            stwierdzono: row.get("stwierdzono")?,
            umowy_pozwalaja: row.get("umowy_pozwalaja")?,
            umowa_oferty_odnosnik: row.get("umowa_oferty_odnosnik")?,
            umowa_poszukiwania_odnosnik: row.get("umowa_poszukiwania_odnosnik")?,
            przekazano: row.get("przekazano")?,
            transakcja_odnosnik: row.get("transakcja_odnosnik")?,
            zatwierdzil: row.get("zatwierdzil")?,
            podpis: row.get("podpis")?,
            zatwierdzono: row.get("zatwierdzono")?,
        })
    }
}

/// Wszystkie pary obu stron, najnowsze pierwsze.
pub fn pary(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ParyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_posrednictwa_obie_strony_pary
ORDER BY stwierdzono DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ParyRow::from_row).collect()
}

/// A row of `pary_do_wyjasnienia`.
#[derive(Debug, Clone, PartialEq)]
pub struct ParyDoWyjasnieniaRow {
    pub id: i64,
    pub teczka_oferty_odnosnik: String,
    pub teczka_poszukiwania_odnosnik: String,
    pub stwierdzono: String,
    pub umowy_pozwalaja: ParyUmowyPozwalaja,
    pub umowa_oferty_odnosnik: Option<String>,
    pub umowa_poszukiwania_odnosnik: Option<String>,
    pub przekazano: Option<String>,
    pub transakcja_odnosnik: Option<String>,
    pub zatwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub zatwierdzono: Option<String>,
}

impl ParyDoWyjasnieniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ParyDoWyjasnieniaRow {
            id: row.get("id")?,
            teczka_oferty_odnosnik: row.get("teczka_oferty_odnosnik")?,
            teczka_poszukiwania_odnosnik: row.get("teczka_poszukiwania_odnosnik")?,
            stwierdzono: row.get("stwierdzono")?,
            umowy_pozwalaja: row.get("umowy_pozwalaja")?,
            umowa_oferty_odnosnik: row.get("umowa_oferty_odnosnik")?,
            umowa_poszukiwania_odnosnik: row.get("umowa_poszukiwania_odnosnik")?,
            przekazano: row.get("przekazano")?,
            transakcja_odnosnik: row.get("transakcja_odnosnik")?,
            zatwierdzil: row.get("zatwierdzil")?,
            podpis: row.get("podpis")?,
            zatwierdzono: row.get("zatwierdzono")?,
        })
    }
}

/// Pary, w których nie wiadomo jeszcze, czy umowy obu stron pozwalają biuru działać dla drugiej
/// strony, od najstarszej.
pub fn pary_do_wyjasnienia(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ParyDoWyjasnieniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_posrednictwa_obie_strony_pary
WHERE umowy_pozwalaja = 'do_wyjasnienia'
ORDER BY stwierdzono, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ParyDoWyjasnieniaRow::from_row).collect()
}

/// The parameters of `zapisz_pare`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZapiszPareParams {
    pub teczka_oferty_odnosnik: String,
    pub teczka_poszukiwania_odnosnik: String,
    pub stwierdzono: String,
}

/// Oferta i poszukiwanie spotkały się w jednej transakcji.
pub fn zapisz_pare(
    room: &impl xmeta_room::MetaRoom,
    params: &ZapiszPareParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_posrednictwa_obie_strony_pary (teczka_oferty_odnosnik,
  teczka_poszukiwania_odnosnik, stwierdzono, umowy_pozwalaja)
VALUES (:teczka_oferty_odnosnik, :teczka_poszukiwania_odnosnik, :stwierdzono, 'do_wyjasnienia')"#,
                parameters: Vec::from([
                    (
                        "teczka_oferty_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.teczka_oferty_odnosnik,
                        ),
                    ),
                    (
                        "teczka_poszukiwania_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.teczka_poszukiwania_odnosnik,
                        ),
                    ),
                    (
                        "stwierdzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.stwierdzono),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `ocen_umowy_pary`.
#[derive(Debug, Clone, PartialEq)]
pub struct OcenUmowyParyParams {
    pub umowy_pozwalaja: ParyUmowyPozwalaja,
    pub umowa_oferty_odnosnik: Option<String>,
    pub umowa_poszukiwania_odnosnik: Option<String>,
    pub id: i64,
}

/// Czy umowy pośrednictwa obu stron pozwalają biuru działać dla drugiej strony; „tak” wskazuje obie
/// umowy.
pub fn ocen_umowy_pary(
    room: &impl xmeta_room::MetaRoom,
    params: &OcenUmowyParyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_posrednictwa_obie_strony_pary
SET umowy_pozwalaja = :umowy_pozwalaja,
  umowa_oferty_odnosnik = :umowa_oferty_odnosnik,
  umowa_poszukiwania_odnosnik = :umowa_poszukiwania_odnosnik
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "umowy_pozwalaja",
                        xmeta_room::ToSqlValue::to_sql_value(&params.umowy_pozwalaja),
                    ),
                    (
                        "umowa_oferty_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.umowa_oferty_odnosnik,
                        ),
                    ),
                    (
                        "umowa_poszukiwania_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.umowa_poszukiwania_odnosnik,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_przekazanie_pary`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPrzekazanieParyParams {
    pub przekazano: String,
    pub zatwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub zatwierdzono: Option<String>,
    pub id: i64,
}

/// Przyjęta oferta przekazana Kierownikowi transakcji; bez zgody obu umów tylko za zgodą człowieka.
pub fn odnotuj_przekazanie_pary(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPrzekazanieParyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_posrednictwa_obie_strony_pary
SET przekazano = :przekazano,
  zatwierdzil = :zatwierdzil,
  podpis = :podpis,
  zatwierdzono = :zatwierdzono
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "przekazano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przekazano),
                    ),
                    (
                        "zatwierdzil",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zatwierdzil),
                    ),
                    ("podpis", xmeta_room::ToSqlValue::to_sql_value(&params.podpis)),
                    (
                        "zatwierdzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zatwierdzono),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_teczke_transakcji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujTeczkeTransakcjiParams {
    pub transakcja_odnosnik: String,
    pub id: i64,
}

/// Kierownik transakcji założył teczkę transakcji dla pary.
pub fn odnotuj_teczke_transakcji(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujTeczkeTransakcjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_posrednictwa_obie_strony_pary
SET transakcja_odnosnik = :transakcja_odnosnik
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "transakcja_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.transakcja_odnosnik),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
