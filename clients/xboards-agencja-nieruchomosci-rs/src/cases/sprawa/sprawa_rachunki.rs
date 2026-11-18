//! The kind `sprawa-rachunki` of the room `cases/sprawa`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "sprawa-rachunki",
    requires: &[
        &super::sprawa_dokumenty::KIND,
        &super::sprawa_korespondencja::KIND,
        &super::sprawa_uczestnicy::KIND,
    ],
    migrations: &[
        r#"-- Rachunki bankowe uczestników, które mają trafić do umowy albo przyjąć wypłatę. Podmiana rachunku
-- to częste oszustwo (R4), więc nowy numer nigdy nie zmienia istniejącego wiersza: powstaje nowy
-- wiersz, a stary zostaje z oznaczeniem, kto i kiedy go zastąpił.
--
-- Rachunek wpisuje agent sprawy, a potwierdzenie posiadacza innym kanałem ten, kto potwierdzał.
-- Wynik sprawdzenia Kontrolera leży w sprawa-sprawdzenia, a zgoda człowieka w sprawa-decyzje;
-- regułę, że zgoda wymaga sprawdzenia i potwierdzenia, zapisuje migracja sprawa-sprawdzenia
-- (sprawa_sprawdzenia_dopuszczenie_rachunku_wymaga_sprawdzenia). Rachunek, na który wolno płacić,
-- to wiersz widoku sprawa_rachunki_dopuszczone.
CREATE TABLE sprawa_rachunki_wpisy (
  id TEXT PRIMARY KEY,
  uczestnik_id TEXT NOT NULL REFERENCES sprawa_uczestnicy_wpisy (id),
  numer TEXT NOT NULL CHECK (numer <> ''),
  -- Czyj jest rachunek według dokumentu albo wiadomości.
  posiadacz TEXT NOT NULL,
  cel TEXT NOT NULL CHECK (cel IN ('zaplata_ceny', 'zadatek', 'oplata_rezerwacyjna', 'kaucja', 'zwrot_kaucji', 'czynsz', 'wyplata_dla_wlasciciela', 'splata_hipoteki', 'wyplata_w_kanale', 'inny')),
  -- Skąd numer: dokument z listy albo wiadomość z wpływów.
  zrodlo_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  zrodlo_wplyw_id TEXT REFERENCES sprawa_korespondencja_wplywy (id),
  wpisano TEXT NOT NULL,
  -- Kto, kiedy i jak potwierdził posiadacza innym kanałem, np. „telefonicznie pod numerem z umowy
  -- przedwstępnej”.
  potwierdzil TEXT,
  potwierdzono TEXT,
  potwierdzenie_sposob TEXT,
  zastapiony_przez_id TEXT REFERENCES sprawa_rachunki_wpisy (id),
  zastapil TEXT,
  zastapiono TEXT,

  CONSTRAINT rachunek_ma_zrodlo CHECK ((zrodlo_dokument_id IS NULL) <> (zrodlo_wplyw_id IS NULL)),
  CONSTRAINT potwierdzenie_w_komplecie CHECK (
    (potwierdzil IS NULL) = (potwierdzono IS NULL)
    AND (potwierdzil IS NULL) = (potwierdzenie_sposob IS NULL)
  ),
  CONSTRAINT zastapienie_ma_autora_i_date CHECK (
    (zastapiony_przez_id IS NULL) = (zastapil IS NULL)
    AND (zastapiony_przez_id IS NULL) = (zastapiono IS NULL)
  ),
  CONSTRAINT nie_zastepuje_samego_siebie CHECK (zastapiony_przez_id IS NOT id)
) STRICT;

-- Rachunki, na które wolno płacić (R4): niezastąpione i dopuszczone zatwierdzoną decyzją
-- dopuszczenie_rachunku. To jedyne miejsce tej reguły; korzystają z niego wypłaty rdzenia
-- i płatności typów spraw. Sprawdzenie i potwierdzenie posiadacza wymaga już sama zgoda na rachunek.
CREATE VIEW sprawa_rachunki_dopuszczone AS
SELECT rachunek.*
FROM sprawa_rachunki_wpisy AS rachunek
WHERE rachunek.zastapiony_przez_id IS NULL
  AND EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS dopuszczenie
    WHERE dopuszczenie.rodzaj = 'dopuszczenie_rachunku'
      AND dopuszczenie.dotyczy_tabela = 'sprawa_rachunki_wpisy'
      AND dopuszczenie.dotyczy_id = rachunek.id
  );

CREATE TRIGGER sprawa_rachunki_dane_rachunku_niezmienne
BEFORE UPDATE OF uczestnik_id, numer, posiadacz, cel, zrodlo_dokument_id, zrodlo_wplyw_id ON sprawa_rachunki_wpisy
WHEN NEW.uczestnik_id IS NOT OLD.uczestnik_id
  OR NEW.numer IS NOT OLD.numer
  OR NEW.posiadacz IS NOT OLD.posiadacz
  OR NEW.cel IS NOT OLD.cel
  OR NEW.zrodlo_dokument_id IS NOT OLD.zrodlo_dokument_id
  OR NEW.zrodlo_wplyw_id IS NOT OLD.zrodlo_wplyw_id
BEGIN
  SELECT RAISE(ABORT, 'numer, posiadacz, uczestnik, cel i źródło rachunku się nie zmieniają; nowy numer to nowy wiersz');
END;

CREATE TRIGGER sprawa_rachunki_zastapienie_ostateczne
BEFORE UPDATE OF zastapiony_przez_id, zastapil, zastapiono ON sprawa_rachunki_wpisy
WHEN OLD.zastapiony_przez_id IS NOT NULL
  AND (
    NEW.zastapiony_przez_id IS NOT OLD.zastapiony_przez_id
    OR NEW.zastapil IS NOT OLD.zastapil
    OR NEW.zastapiono IS NOT OLD.zastapiono
  )
BEGIN
  SELECT RAISE(ABORT, 'zastąpionego rachunku się nie przywraca ani nie zastępuje drugi raz');
END;

-- Na potwierdzeniu posiadacza stoi zgoda na rachunek, więc potwierdzenie wpisuje się raz.
-- Nowe potwierdzenie to nowy rachunek.
CREATE TRIGGER sprawa_rachunki_potwierdzenie_raz
BEFORE UPDATE OF potwierdzil, potwierdzono, potwierdzenie_sposob ON sprawa_rachunki_wpisy
WHEN OLD.potwierdzil IS NOT NULL
  AND (
    NEW.potwierdzil IS NOT OLD.potwierdzil
    OR NEW.potwierdzono IS NOT OLD.potwierdzono
    OR NEW.potwierdzenie_sposob IS NOT OLD.potwierdzenie_sposob
  )
BEGIN
  SELECT RAISE(ABORT, 'potwierdzenie posiadacza rachunku wpisuje się raz i się go nie zmienia');
END;

-- Nowy numer zastępuje stary tylko u tego samego uczestnika i w tym samym celu: inaczej
-- zastąpienie podmieniłoby rachunek innej osoby albo innej płatności (R4).
CREATE TRIGGER sprawa_rachunki_zastepca_tego_samego_uczestnika_i_celu
BEFORE UPDATE OF zastapiony_przez_id ON sprawa_rachunki_wpisy
WHEN NEW.zastapiony_przez_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_rachunki_wpisy AS zastepca
    WHERE zastepca.id = NEW.zastapiony_przez_id
      AND zastepca.uczestnik_id = NEW.uczestnik_id
      AND zastepca.cel = NEW.cel
  )
BEGIN
  SELECT RAISE(ABORT, 'rachunek zastępujący ma tego samego uczestnika i ten sam cel');
END;

CREATE TRIGGER sprawa_rachunki_wpisany_zastepca_tego_samego_uczestnika_i_celu
BEFORE INSERT ON sprawa_rachunki_wpisy
WHEN NEW.zastapiony_przez_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_rachunki_wpisy AS zastepca
    WHERE zastepca.id = NEW.zastapiony_przez_id
      AND zastepca.uczestnik_id = NEW.uczestnik_id
      AND zastepca.cel = NEW.cel
  )
BEGIN
  SELECT RAISE(ABORT, 'rachunek zastępujący ma tego samego uczestnika i ten sam cel');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_rachunki_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_rachunki_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_rachunki_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_rachunki_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_rachunki_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
    ],
};

/// The values of column `wpisy.cel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyCel {
    ZaplataCeny,
    Zadatek,
    OplataRezerwacyjna,
    Kaucja,
    ZwrotKaucji,
    Czynsz,
    WyplataDlaWlasciciela,
    SplataHipoteki,
    WyplataWKanale,
    Inny,
}

impl WpisyCel {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyCel] = &[
        WpisyCel::ZaplataCeny,
        WpisyCel::Zadatek,
        WpisyCel::OplataRezerwacyjna,
        WpisyCel::Kaucja,
        WpisyCel::ZwrotKaucji,
        WpisyCel::Czynsz,
        WpisyCel::WyplataDlaWlasciciela,
        WpisyCel::SplataHipoteki,
        WpisyCel::WyplataWKanale,
        WpisyCel::Inny,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyCel::ZaplataCeny => "zaplata_ceny",
            WpisyCel::Zadatek => "zadatek",
            WpisyCel::OplataRezerwacyjna => "oplata_rezerwacyjna",
            WpisyCel::Kaucja => "kaucja",
            WpisyCel::ZwrotKaucji => "zwrot_kaucji",
            WpisyCel::Czynsz => "czynsz",
            WpisyCel::WyplataDlaWlasciciela => "wyplata_dla_wlasciciela",
            WpisyCel::SplataHipoteki => "splata_hipoteki",
            WpisyCel::WyplataWKanale => "wyplata_w_kanale",
            WpisyCel::Inny => "inny",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyCel {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "zaplata_ceny" => Ok(WpisyCel::ZaplataCeny),
                    "zadatek" => Ok(WpisyCel::Zadatek),
                    "oplata_rezerwacyjna" => Ok(WpisyCel::OplataRezerwacyjna),
                    "kaucja" => Ok(WpisyCel::Kaucja),
                    "zwrot_kaucji" => Ok(WpisyCel::ZwrotKaucji),
                    "czynsz" => Ok(WpisyCel::Czynsz),
                    "wyplata_dla_wlasciciela" => Ok(WpisyCel::WyplataDlaWlasciciela),
                    "splata_hipoteki" => Ok(WpisyCel::SplataHipoteki),
                    "wyplata_w_kanale" => Ok(WpisyCel::WyplataWKanale),
                    "inny" => Ok(WpisyCel::Inny),
                    other => Err(format!("{other} is no value of WpisyCel")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyCel {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `rachunki`.
#[derive(Debug, Clone, PartialEq)]
pub struct RachunkiRow {
    pub id: String,
    pub uczestnik_id: String,
    pub numer: String,
    pub posiadacz: String,
    pub cel: WpisyCel,
    pub zrodlo_dokument_id: Option<String>,
    pub zrodlo_wplyw_id: Option<String>,
    pub wpisano: String,
    pub potwierdzil: Option<String>,
    pub potwierdzono: Option<String>,
    pub potwierdzenie_sposob: Option<String>,
    pub zastapiony_przez_id: Option<String>,
    pub zastapil: Option<String>,
    pub zastapiono: Option<String>,
}

impl RachunkiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(RachunkiRow {
            id: row.get("id")?,
            uczestnik_id: row.get("uczestnik_id")?,
            numer: row.get("numer")?,
            posiadacz: row.get("posiadacz")?,
            cel: row.get("cel")?,
            zrodlo_dokument_id: row.get("zrodlo_dokument_id")?,
            zrodlo_wplyw_id: row.get("zrodlo_wplyw_id")?,
            wpisano: row.get("wpisano")?,
            potwierdzil: row.get("potwierdzil")?,
            potwierdzono: row.get("potwierdzono")?,
            potwierdzenie_sposob: row.get("potwierdzenie_sposob")?,
            zastapiony_przez_id: row.get("zastapiony_przez_id")?,
            zastapil: row.get("zastapil")?,
            zastapiono: row.get("zastapiono")?,
        })
    }
}

/// Rachunki uczestników, także zastąpione, w kolejności wpisania.
pub fn rachunki(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<RachunkiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_rachunki_wpisy
ORDER BY wpisano, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(RachunkiRow::from_row).collect()
}

/// A row of `rachunki_biezace`.
#[derive(Debug, Clone, PartialEq)]
pub struct RachunkiBiezaceRow {
    pub id: String,
    pub uczestnik_id: String,
    pub numer: String,
    pub posiadacz: String,
    pub cel: WpisyCel,
    pub zrodlo_dokument_id: Option<String>,
    pub zrodlo_wplyw_id: Option<String>,
    pub wpisano: String,
    pub potwierdzil: Option<String>,
    pub potwierdzono: Option<String>,
    pub potwierdzenie_sposob: Option<String>,
    pub zastapiony_przez_id: Option<String>,
    pub zastapil: Option<String>,
    pub zastapiono: Option<String>,
}

impl RachunkiBiezaceRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(RachunkiBiezaceRow {
            id: row.get("id")?,
            uczestnik_id: row.get("uczestnik_id")?,
            numer: row.get("numer")?,
            posiadacz: row.get("posiadacz")?,
            cel: row.get("cel")?,
            zrodlo_dokument_id: row.get("zrodlo_dokument_id")?,
            zrodlo_wplyw_id: row.get("zrodlo_wplyw_id")?,
            wpisano: row.get("wpisano")?,
            potwierdzil: row.get("potwierdzil")?,
            potwierdzono: row.get("potwierdzono")?,
            potwierdzenie_sposob: row.get("potwierdzenie_sposob")?,
            zastapiony_przez_id: row.get("zastapiony_przez_id")?,
            zastapil: row.get("zastapil")?,
            zastapiono: row.get("zastapiono")?,
        })
    }
}

/// Rachunki, których nikt nie zastąpił.
pub fn rachunki_biezace(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<RachunkiBiezaceRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_rachunki_wpisy
WHERE zastapiony_przez_id IS NULL
ORDER BY wpisano, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(RachunkiBiezaceRow::from_row).collect()
}

/// A row of `rachunki_dopuszczone`.
#[derive(Debug, Clone, PartialEq)]
pub struct RachunkiDopuszczoneRow {
    pub id: String,
    pub uczestnik_id: String,
    pub numer: String,
    pub posiadacz: String,
    pub cel: WpisyCel,
    pub zrodlo_dokument_id: Option<String>,
    pub zrodlo_wplyw_id: Option<String>,
    pub wpisano: String,
    pub potwierdzil: Option<String>,
    pub potwierdzono: Option<String>,
    pub potwierdzenie_sposob: Option<String>,
    pub zastapiony_przez_id: Option<String>,
    pub zastapil: Option<String>,
    pub zastapiono: Option<String>,
}

impl RachunkiDopuszczoneRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(RachunkiDopuszczoneRow {
            id: row.get("id")?,
            uczestnik_id: row.get("uczestnik_id")?,
            numer: row.get("numer")?,
            posiadacz: row.get("posiadacz")?,
            cel: row.get("cel")?,
            zrodlo_dokument_id: row.get("zrodlo_dokument_id")?,
            zrodlo_wplyw_id: row.get("zrodlo_wplyw_id")?,
            wpisano: row.get("wpisano")?,
            potwierdzil: row.get("potwierdzil")?,
            potwierdzono: row.get("potwierdzono")?,
            potwierdzenie_sposob: row.get("potwierdzenie_sposob")?,
            zastapiony_przez_id: row.get("zastapiony_przez_id")?,
            zastapil: row.get("zastapil")?,
            zastapiono: row.get("zastapiono")?,
        })
    }
}

/// Rachunki, na które wolno płacić: niezastąpione i dopuszczone zatwierdzoną decyzją.
pub fn rachunki_dopuszczone(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<RachunkiDopuszczoneRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_rachunki_dopuszczone
ORDER BY wpisano, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(RachunkiDopuszczoneRow::from_row).collect()
}

/// The parameters of `wpisz_rachunek`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszRachunekParams {
    pub id: String,
    pub uczestnik_id: String,
    pub numer: String,
    pub posiadacz: String,
    pub cel: WpisyCel,
    pub zrodlo_dokument_id: Option<String>,
    pub zrodlo_wplyw_id: Option<String>,
    pub wpisano: String,
}

/// Nowy numer rachunku z jego źródłem; czeka na potwierdzenie, sprawdzenie i zgodę.
pub fn wpisz_rachunek(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszRachunekParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_rachunki_wpisy (id, uczestnik_id, numer, posiadacz, cel, zrodlo_dokument_id,
  zrodlo_wplyw_id, wpisano)
VALUES (:id, :uczestnik_id, :numer, :posiadacz, :cel, :zrodlo_dokument_id, :zrodlo_wplyw_id,
  :wpisano)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.uczestnik_id),
                    ),
                    ("numer", xmeta_room::ToSqlValue::to_sql_value(&params.numer)),
                    (
                        "posiadacz",
                        xmeta_room::ToSqlValue::to_sql_value(&params.posiadacz),
                    ),
                    ("cel", xmeta_room::ToSqlValue::to_sql_value(&params.cel)),
                    (
                        "zrodlo_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_dokument_id),
                    ),
                    (
                        "zrodlo_wplyw_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_wplyw_id),
                    ),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_potwierdzenie_rachunku`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPotwierdzenieRachunkuParams {
    pub potwierdzil: String,
    pub potwierdzono: String,
    pub potwierdzenie_sposob: String,
    pub id: String,
}

/// Posiadacz potwierdzony innym kanałem niż ten, którym przyszedł numer.
pub fn odnotuj_potwierdzenie_rachunku(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPotwierdzenieRachunkuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_rachunki_wpisy
SET potwierdzil = :potwierdzil,
  potwierdzono = :potwierdzono,
  potwierdzenie_sposob = :potwierdzenie_sposob
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "potwierdzil",
                        xmeta_room::ToSqlValue::to_sql_value(&params.potwierdzil),
                    ),
                    (
                        "potwierdzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.potwierdzono),
                    ),
                    (
                        "potwierdzenie_sposob",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.potwierdzenie_sposob,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `zastap_rachunek`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZastapRachunekParams {
    pub zastapiony_przez_id: String,
    pub zastapil: String,
    pub zastapiono: String,
    pub id: String,
}

/// Stary rachunek zastąpiony nowym wierszem, z tym, kto i kiedy go zastąpił.
pub fn zastap_rachunek(
    room: &impl xmeta_room::MetaRoom,
    params: &ZastapRachunekParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_rachunki_wpisy
SET zastapiony_przez_id = :zastapiony_przez_id,
  zastapil = :zastapil,
  zastapiono = :zastapiono
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zastapiony_przez_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zastapiony_przez_id),
                    ),
                    ("zastapil", xmeta_room::ToSqlValue::to_sql_value(&params.zastapil)),
                    (
                        "zastapiono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zastapiono),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
