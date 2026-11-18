//! The kind `kierownik-posrednictwa-przydzial` of the room `positions/posrednictwo/kierownik_posrednictwa`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "kierownik-posrednictwa-przydzial";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Przydział pośredników: reguły każdego pośrednika (obszar, rodzaje nieruchomości, limit spraw
-- i czy przyjmuje nowe) i karty od Kierownika biura z pośrednikiem, którego kierownik zaproponował.
--
-- Reguły pośrednika zmienia tylko jego wiadomość podpisana jego kluczem (podszywanie się); podpis
-- sprawdza aplikacja, a baza wymaga jego obecności. Zmiana to nowy wiersz, a poprzedni dostaje datę
-- zastąpienia, więc historia zostaje (R9). Pośrednik ma naraz jedną regułę.
CREATE TABLE kierownik_posrednictwa_przydzial_posrednicy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_posrednictwa_przydzial_posrednicy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Tożsamość pośrednika.
  posrednik TEXT NOT NULL CHECK (posrednik <> ''),
  -- Np. „Wrocław, Krzyki i Fabryczna”.
  obszar TEXT NOT NULL CHECK (obszar <> ''),
  -- Np. „mieszkania, domy”.
  rodzaje_nieruchomosci TEXT NOT NULL CHECK (rodzaje_nieruchomosci <> ''),
  -- Ile spraw (ofert i poszukiwań) pośrednik prowadzi naraz najwyżej.
  limit_spraw INTEGER NOT NULL CHECK (limit_spraw > 0),
  przyjmuje_nowe INTEGER NOT NULL CHECK (przyjmuje_nowe IN (0, 1)),
  od TEXT NOT NULL,
  -- Podpisana wiadomość pośrednika, która ustaliła reguły, tekstem.
  wiadomosc_odnosnik TEXT NOT NULL CHECK (wiadomosc_odnosnik <> ''),
  podpis TEXT NOT NULL CHECK (podpis <> ''),
  zastapione_od TEXT,

  CONSTRAINT zastapienie_nie_przed_poczatkiem CHECK (zastapione_od IS NULL OR zastapione_od >= od)
) STRICT;

-- Karta nowej sprawy od Kierownika biura. Kontakt leży na karcie, dopóki pośrednik nie założy
-- teczki albo karta nie zostanie odrzucona; potem kierownik go usuwa, bo trafił do teczki.
-- Pośrednik spoza reguł wymaga zgody człowieka; jego podpis kierownik przenosi do wiersza.
CREATE TABLE kierownik_posrednictwa_przydzial_karty (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_posrednictwa_przydzial_karty_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Karta w pamięci Kierownika biura, tekstem.
  karta_odnosnik TEXT NOT NULL UNIQUE CHECK (karta_odnosnik <> ''),
  otrzymano TEXT NOT NULL,
  kontakt TEXT,
  stan TEXT NOT NULL CHECK (stan IN ('czeka', 'zaproponowana', 'przyjeta', 'odrzucona')),
  -- Tożsamość zaproponowanego pośrednika i reguła, według której go zaproponowano.
  posrednik TEXT,
  regula_id INTEGER REFERENCES kierownik_posrednictwa_przydzial_posrednicy (id),
  zaproponowano TEXT,
  -- Teczka oferty albo poszukiwania, którą pośrednik założył, tekstem.
  teczka_odnosnik TEXT,
  powod TEXT,
  rozstrzygnieto TEXT,
  zatwierdzil TEXT,
  podpis TEXT,
  zatwierdzono TEXT,

  CONSTRAINT propozycja_z_posrednikiem CHECK (
    (stan IN ('zaproponowana', 'przyjeta')) = (posrednik IS NOT NULL)
    AND (posrednik IS NULL) = (zaproponowano IS NULL)
  ),
  CONSTRAINT regula_przy_posredniku CHECK (regula_id IS NULL OR posrednik IS NOT NULL),
  CONSTRAINT posrednik_poza_regulami_za_zgoda_czlowieka CHECK (
    posrednik IS NULL
    OR regula_id IS NOT NULL
    OR zatwierdzil IS NOT NULL
  ),
  CONSTRAINT przyjeta_z_teczka CHECK ((stan = 'przyjeta') = (teczka_odnosnik IS NOT NULL)),
  CONSTRAINT odrzucona_z_powodem CHECK ((stan = 'odrzucona') = (powod IS NOT NULL)),
  CONSTRAINT rozstrzygnieta_z_data CHECK ((stan IN ('przyjeta', 'odrzucona')) = (rozstrzygnieto IS NOT NULL)),
  CONSTRAINT rozstrzygnieta_bez_kontaktu CHECK (stan NOT IN ('przyjeta', 'odrzucona') OR kontakt IS NULL),
  CONSTRAINT zatwierdzenie_w_komplecie CHECK (
    (zatwierdzil IS NULL) = (podpis IS NULL)
    AND (zatwierdzil IS NULL) = (zatwierdzono IS NULL)
  ),
  CONSTRAINT rozstrzygniecie_nie_przed_otrzymaniem CHECK (rozstrzygnieto IS NULL OR rozstrzygnieto >= otrzymano)
) STRICT;

-- Pośrednik ma naraz jedną regułę; nowa przychodzi po zastąpieniu starej.
CREATE UNIQUE INDEX kierownik_posrednictwa_przydzial_jedna_regula_posrednika
ON kierownik_posrednictwa_przydzial_posrednicy (posrednik)
WHERE zastapione_od IS NULL;

-- Pośrednicy ------------------------------------------------------------------------------------

CREATE TRIGGER kierownik_posrednictwa_przydzial_regula_powstaje_aktualna
BEFORE INSERT ON kierownik_posrednictwa_przydzial_posrednicy
WHEN NEW.zastapione_od IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'reguła pośrednika powstaje aktualna; zastąpienie odnotowuje się później');
END;

-- Druga aktualna reguła z innym kluczem zderzyłaby się z indeksem, a INSERT OR REPLACE usunąłby
-- wtedy pierwszą z ominięciem blokady usuwania; ten trigger odmawia wcześniej.
CREATE TRIGGER kierownik_posrednictwa_przydzial_jedna_regula_przed_zastapieniem
BEFORE INSERT ON kierownik_posrednictwa_przydzial_posrednicy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_posrednictwa_przydzial_posrednicy
  WHERE posrednik = NEW.posrednik
    AND zastapione_od IS NULL
    AND id IS NOT NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'ten pośrednik ma już aktualną regułę; nowa przychodzi po zastąpieniu starej');
END;

-- Podpis obejmuje regułę, więc zmienia się tylko raz odnotowane zastąpienie.
CREATE TRIGGER kierownik_posrednictwa_przydzial_regula_bez_zmian
BEFORE UPDATE ON kierownik_posrednictwa_przydzial_posrednicy
WHEN OLD.zastapione_od IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.posrednik IS NOT OLD.posrednik
  OR NEW.obszar IS NOT OLD.obszar
  OR NEW.rodzaje_nieruchomosci IS NOT OLD.rodzaje_nieruchomosci
  OR NEW.limit_spraw IS NOT OLD.limit_spraw
  OR NEW.przyjmuje_nowe IS NOT OLD.przyjmuje_nowe
  OR NEW.od IS NOT OLD.od
  OR NEW.wiadomosc_odnosnik IS NOT OLD.wiadomosc_odnosnik
  OR NEW.podpis IS NOT OLD.podpis
BEGIN
  SELECT RAISE(ABORT, 'reguły pośrednika się nie zmienia; zmiana to nowa reguła z podpisem pośrednika');
END;

-- Karty ------------------------------------------------------------------------------------------

CREATE TRIGGER kierownik_posrednictwa_przydzial_karta_powstaje_czekajaca
BEFORE INSERT ON kierownik_posrednictwa_przydzial_karty
WHEN NEW.stan <> 'czeka'
BEGIN
  SELECT RAISE(ABORT, 'karta powstaje jako czekająca; propozycja pośrednika przychodzi później');
END;

-- Propozycja według reguły wskazuje pośrednika tej reguły, gdy reguła jest aktualna i pośrednik
-- przyjmuje nowe sprawy.
CREATE TRIGGER kierownik_posrednictwa_przydzial_propozycja_zgodna_z_regula
BEFORE UPDATE OF regula_id, posrednik ON kierownik_posrednictwa_przydzial_karty
WHEN NEW.regula_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM kierownik_posrednictwa_przydzial_posrednicy AS regula
    WHERE regula.id = NEW.regula_id
      AND regula.posrednik = NEW.posrednik
      AND regula.zastapione_od IS NULL
      AND regula.przyjmuje_nowe = 1
  )
BEGIN
  SELECT RAISE(ABORT, 'propozycja według reguły wskazuje pośrednika aktualnej reguły, który przyjmuje nowe sprawy');
END;

-- Karta od Kierownika biura się nie zmienia; przyjętej ani odrzuconej karty nie zmienia się wcale.
CREATE TRIGGER kierownik_posrednictwa_przydzial_karta_bez_zmian
BEFORE UPDATE ON kierownik_posrednictwa_przydzial_karty
WHEN OLD.stan IN ('przyjeta', 'odrzucona')
  OR NEW.id IS NOT OLD.id
  OR NEW.karta_odnosnik IS NOT OLD.karta_odnosnik
  OR NEW.otrzymano IS NOT OLD.otrzymano
BEGIN
  SELECT RAISE(ABORT, 'karty od Kierownika biura się nie zmienia, a przyjętej ani odrzuconej wcale');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers; także kartę o tym samym
-- odnośniku. Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo
-- klucze są dodatnie.
CREATE TRIGGER kierownik_posrednictwa_przydzial_posrednicy_bez_zastepowania
BEFORE INSERT ON kierownik_posrednictwa_przydzial_posrednicy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_posrednictwa_przydzial_posrednicy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER kierownik_posrednictwa_przydzial_karty_bez_zastepowania
BEFORE INSERT ON kierownik_posrednictwa_przydzial_karty
WHEN EXISTS (
  SELECT 1
  FROM kierownik_posrednictwa_przydzial_karty
  WHERE id = NEW.id
    OR karta_odnosnik = NEW.karta_odnosnik
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: zastąpione reguły i rozstrzygnięte karty zostają jako historia przydziałów.
CREATE TRIGGER kierownik_posrednictwa_przydzial_posrednicy_bez_usuwania
BEFORE DELETE ON kierownik_posrednictwa_przydzial_posrednicy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER kierownik_posrednictwa_przydzial_karty_bez_usuwania
BEFORE DELETE ON kierownik_posrednictwa_przydzial_karty
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
];

/// The values of column `karty.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KartyStan {
    Czeka,
    Zaproponowana,
    Przyjeta,
    Odrzucona,
}

impl KartyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[KartyStan] = &[
        KartyStan::Czeka,
        KartyStan::Zaproponowana,
        KartyStan::Przyjeta,
        KartyStan::Odrzucona,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            KartyStan::Czeka => "czeka",
            KartyStan::Zaproponowana => "zaproponowana",
            KartyStan::Przyjeta => "przyjeta",
            KartyStan::Odrzucona => "odrzucona",
        }
    }
}

impl xmeta_room::FromSqlValue for KartyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "czeka" => Ok(KartyStan::Czeka),
                    "zaproponowana" => Ok(KartyStan::Zaproponowana),
                    "przyjeta" => Ok(KartyStan::Przyjeta),
                    "odrzucona" => Ok(KartyStan::Odrzucona),
                    other => Err(format!("{other} is no value of KartyStan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for KartyStan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `karty_otwarte`.
#[derive(Debug, Clone, PartialEq)]
pub struct KartyOtwarteRow {
    pub id: i64,
    pub karta_odnosnik: String,
    pub otrzymano: String,
    pub kontakt: Option<String>,
    pub stan: KartyStan,
    pub posrednik: Option<String>,
    pub regula_id: Option<i64>,
    pub zaproponowano: Option<String>,
    pub teczka_odnosnik: Option<String>,
    pub powod: Option<String>,
    pub rozstrzygnieto: Option<String>,
    pub zatwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub zatwierdzono: Option<String>,
}

impl KartyOtwarteRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(KartyOtwarteRow {
            id: row.get("id")?,
            karta_odnosnik: row.get("karta_odnosnik")?,
            otrzymano: row.get("otrzymano")?,
            kontakt: row.get("kontakt")?,
            stan: row.get("stan")?,
            posrednik: row.get("posrednik")?,
            regula_id: row.get("regula_id")?,
            zaproponowano: row.get("zaproponowano")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            powod: row.get("powod")?,
            rozstrzygnieto: row.get("rozstrzygnieto")?,
            zatwierdzil: row.get("zatwierdzil")?,
            podpis: row.get("podpis")?,
            zatwierdzono: row.get("zatwierdzono")?,
        })
    }
}

/// Karty, które czekają na propozycję albo na pośrednika, od najstarszej.
pub fn karty_otwarte(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<KartyOtwarteRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_posrednictwa_przydzial_karty
WHERE stan IN ('czeka', 'zaproponowana')
ORDER BY otrzymano, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(KartyOtwarteRow::from_row).collect()
}

/// The parameters of `przyjmij_karte`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrzyjmijKarteParams {
    pub karta_odnosnik: String,
    pub otrzymano: String,
    pub kontakt: Option<String>,
}

/// Karta nowej sprawy od Kierownika biura.
pub fn przyjmij_karte(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzyjmijKarteParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_posrednictwa_przydzial_karty (karta_odnosnik, otrzymano, kontakt, stan)
VALUES (:karta_odnosnik, :otrzymano, :kontakt, 'czeka')"#,
                parameters: Vec::from([
                    (
                        "karta_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.karta_odnosnik),
                    ),
                    (
                        "otrzymano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.otrzymano),
                    ),
                    ("kontakt", xmeta_room::ToSqlValue::to_sql_value(&params.kontakt)),
                ]),
            },
        ],
    )
}

/// The parameters of `zaproponuj_posrednika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZaproponujPosrednikaParams {
    pub posrednik: String,
    pub regula_id: Option<i64>,
    pub zaproponowano: String,
    pub zatwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub zatwierdzono: Option<String>,
    pub id: i64,
}

/// Propozycja pośrednika według reguły albo, za zgodą człowieka, poza regułami.
pub fn zaproponuj_posrednika(
    room: &impl xmeta_room::MetaRoom,
    params: &ZaproponujPosrednikaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_posrednictwa_przydzial_karty
SET stan = 'zaproponowana',
  posrednik = :posrednik,
  regula_id = :regula_id,
  zaproponowano = :zaproponowano,
  zatwierdzil = :zatwierdzil,
  podpis = :podpis,
  zatwierdzono = :zatwierdzono
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "posrednik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.posrednik),
                    ),
                    (
                        "regula_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.regula_id),
                    ),
                    (
                        "zaproponowano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zaproponowano),
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

/// The parameters of `cofnij_propozycje`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CofnijPropozycjeParams {
    pub id: i64,
}

/// Pośrednik nie przyjął sprawy, więc karta czeka na nową propozycję.
pub fn cofnij_propozycje(
    room: &impl xmeta_room::MetaRoom,
    params: &CofnijPropozycjeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_posrednictwa_przydzial_karty
SET stan = 'czeka',
  posrednik = NULL,
  regula_id = NULL,
  zaproponowano = NULL,
  zatwierdzil = NULL,
  podpis = NULL,
  zatwierdzono = NULL
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_zalozenie_teczki`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujZalozenieTeczkiParams {
    pub teczka_odnosnik: String,
    pub rozstrzygnieto: String,
    pub id: i64,
}

/// Pośrednik przyjął sprawę i założył teczkę; kontakt trafił do teczki, więc znika z karty.
pub fn odnotuj_zalozenie_teczki(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujZalozenieTeczkiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_posrednictwa_przydzial_karty
SET stan = 'przyjeta',
  teczka_odnosnik = :teczka_odnosnik,
  rozstrzygnieto = :rozstrzygnieto,
  kontakt = NULL
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "teczka_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.teczka_odnosnik),
                    ),
                    (
                        "rozstrzygnieto",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rozstrzygnieto),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odrzuc_karte`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdrzucKarteParams {
    pub powod: String,
    pub rozstrzygnieto: String,
    pub id: i64,
}

/// Karta odrzucona z powodem; kontakt znika z karty.
pub fn odrzuc_karte(
    room: &impl xmeta_room::MetaRoom,
    params: &OdrzucKarteParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_posrednictwa_przydzial_karty
SET stan = 'odrzucona',
  posrednik = NULL,
  regula_id = NULL,
  zaproponowano = NULL,
  powod = :powod,
  rozstrzygnieto = :rozstrzygnieto,
  kontakt = NULL
WHERE id = :id"#,
                parameters: Vec::from([
                    ("powod", xmeta_room::ToSqlValue::to_sql_value(&params.powod)),
                    (
                        "rozstrzygnieto",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rozstrzygnieto),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `posrednicy`.
#[derive(Debug, Clone, PartialEq)]
pub struct PosrednicyRow {
    pub id: i64,
    pub posrednik: String,
    pub obszar: String,
    pub rodzaje_nieruchomosci: String,
    pub limit_spraw: i64,
    pub przyjmuje_nowe: bool,
    pub od: String,
    pub wiadomosc_odnosnik: String,
    pub podpis: String,
    pub zastapione_od: Option<String>,
}

impl PosrednicyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PosrednicyRow {
            id: row.get("id")?,
            posrednik: row.get("posrednik")?,
            obszar: row.get("obszar")?,
            rodzaje_nieruchomosci: row.get("rodzaje_nieruchomosci")?,
            limit_spraw: row.get("limit_spraw")?,
            przyjmuje_nowe: row.get("przyjmuje_nowe")?,
            od: row.get("od")?,
            wiadomosc_odnosnik: row.get("wiadomosc_odnosnik")?,
            podpis: row.get("podpis")?,
            zastapione_od: row.get("zastapione_od")?,
        })
    }
}

/// Aktualne reguły pośredników.
pub fn posrednicy(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PosrednicyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_posrednictwa_przydzial_posrednicy
WHERE zastapione_od IS NULL
ORDER BY posrednik, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(PosrednicyRow::from_row).collect()
}

/// The parameters of `historia_regul_posrednika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HistoriaRegulPosrednikaParams {
    pub posrednik: String,
}

/// A row of `historia_regul_posrednika`.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoriaRegulPosrednikaRow {
    pub id: i64,
    pub posrednik: String,
    pub obszar: String,
    pub rodzaje_nieruchomosci: String,
    pub limit_spraw: i64,
    pub przyjmuje_nowe: bool,
    pub od: String,
    pub wiadomosc_odnosnik: String,
    pub podpis: String,
    pub zastapione_od: Option<String>,
}

impl HistoriaRegulPosrednikaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(HistoriaRegulPosrednikaRow {
            id: row.get("id")?,
            posrednik: row.get("posrednik")?,
            obszar: row.get("obszar")?,
            rodzaje_nieruchomosci: row.get("rodzaje_nieruchomosci")?,
            limit_spraw: row.get("limit_spraw")?,
            przyjmuje_nowe: row.get("przyjmuje_nowe")?,
            od: row.get("od")?,
            wiadomosc_odnosnik: row.get("wiadomosc_odnosnik")?,
            podpis: row.get("podpis")?,
            zastapione_od: row.get("zastapione_od")?,
        })
    }
}

/// Wszystkie reguły jednego pośrednika, także zastąpione, od najstarszej.
pub fn historia_regul_posrednika(
    room: &impl xmeta_room::MetaRoom,
    params: &HistoriaRegulPosrednikaParams,
) -> Result<Vec<HistoriaRegulPosrednikaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_posrednictwa_przydzial_posrednicy
WHERE posrednik = :posrednik
ORDER BY od, id"#,
        parameters: Vec::from([
            ("posrednik", xmeta_room::ToSqlValue::to_sql_value(&params.posrednik)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(HistoriaRegulPosrednikaRow::from_row).collect()
}

/// A row of `obciazenie_posrednikow`.
#[derive(Debug, Clone, PartialEq)]
pub struct ObciazeniePosrednikowRow {
    pub regula_id: i64,
    pub posrednik: String,
    pub limit_spraw: i64,
    pub przyjmuje_nowe: bool,
    pub sprawy: i64,
}

impl ObciazeniePosrednikowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ObciazeniePosrednikowRow {
            regula_id: row.get("regula_id")?,
            posrednik: row.get("posrednik")?,
            limit_spraw: row.get("limit_spraw")?,
            przyjmuje_nowe: row.get("przyjmuje_nowe")?,
            sprawy: row.get("sprawy")?,
        })
    }
}

/// Ile spraw prowadzi każdy pośrednik z aktualną regułą wobec swojego limitu: tyle, ilu niewymienionych
/// opiekunów ma jako właściciel. Według tego kierownik proponuje pośrednika i melduje obłożenie.
pub fn obciazenie_posrednikow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ObciazeniePosrednikowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT regula.id AS regula_id,
  regula.posrednik,
  regula.limit_spraw,
  regula.przyjmuje_nowe,
  (
    SELECT count(*)
    FROM kierownik_dzialu_agenci_wpisy AS agent
    WHERE agent.wlasciciel = regula.posrednik
      AND agent.stanowisko IN ('opiekun_oferty', 'opiekun_poszukujacego')
      AND agent.stan <> 'wymieniony'
  ) AS sprawy
FROM kierownik_posrednictwa_przydzial_posrednicy AS regula
WHERE regula.zastapione_od IS NULL
ORDER BY regula.posrednik, regula.id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(ObciazeniePosrednikowRow::from_row).collect()
}

/// The parameters of `zapisz_regule_posrednika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZapiszRegulePosrednikaParams {
    pub posrednik: String,
    pub obszar: String,
    pub rodzaje_nieruchomosci: String,
    pub limit_spraw: i64,
    pub przyjmuje_nowe: bool,
    pub od: String,
    pub wiadomosc_odnosnik: String,
    pub podpis: String,
}

/// Nowa reguła z podpisanej wiadomości pośrednika; poprzednią trzeba najpierw zastąpić.
pub fn zapisz_regule_posrednika(
    room: &impl xmeta_room::MetaRoom,
    params: &ZapiszRegulePosrednikaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_posrednictwa_przydzial_posrednicy (posrednik, obszar, rodzaje_nieruchomosci,
  limit_spraw, przyjmuje_nowe, od, wiadomosc_odnosnik, podpis)
VALUES (:posrednik, :obszar, :rodzaje_nieruchomosci, :limit_spraw, :przyjmuje_nowe, :od,
  :wiadomosc_odnosnik, :podpis)"#,
                parameters: Vec::from([
                    (
                        "posrednik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.posrednik),
                    ),
                    ("obszar", xmeta_room::ToSqlValue::to_sql_value(&params.obszar)),
                    (
                        "rodzaje_nieruchomosci",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.rodzaje_nieruchomosci,
                        ),
                    ),
                    (
                        "limit_spraw",
                        xmeta_room::ToSqlValue::to_sql_value(&params.limit_spraw),
                    ),
                    (
                        "przyjmuje_nowe",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przyjmuje_nowe),
                    ),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    (
                        "wiadomosc_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wiadomosc_odnosnik),
                    ),
                    ("podpis", xmeta_room::ToSqlValue::to_sql_value(&params.podpis)),
                ]),
            },
        ],
    )
}

/// The parameters of `zastap_regule_posrednika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZastapRegulePosrednikaParams {
    pub zastapione_od: String,
    pub id: i64,
}

/// Reguła przestaje obowiązywać, bo pośrednik podpisał nową.
pub fn zastap_regule_posrednika(
    room: &impl xmeta_room::MetaRoom,
    params: &ZastapRegulePosrednikaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_posrednictwa_przydzial_posrednicy
SET zastapione_od = :zastapione_od
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zastapione_od",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zastapione_od),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
