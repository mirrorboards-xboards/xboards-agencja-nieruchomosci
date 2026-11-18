//! The kind `wykonawcy-cenniki` of the room `registers/wykonawcy`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "wykonawcy-cenniki",
    requires: &[&super::wykonawcy_spis::KIND],
    migrations: &[
        r#"-- Cenniki wykonawców w wersjach i ich przypisanie do wykonawców. Jeden cennik może obowiązywać
-- kilku wykonawców; wtedy wszyscy pracują na tych samych warunkach. Zlecenie w teczce lokalu
-- wskazuje wersję cennika tekstem, więc koszt w starym zestawieniu nie zmienia się po nowej
-- wersji.
--
-- Wersję zatwierdza decyzja człowieka rodzaju „cennik” (wykonawcy_spis_decyzje), której podpis
-- obejmuje skrót całej wersji z pozycjami; skrót liczy i sprawdza aplikacja. Kierownik najmu
-- przepisuje wersję jako projekt, dopisuje pozycje i zamyka ją. Od zamknięcia wersji ani jej pozycji
-- się nie zmienia, a Koordynator stosuje tylko wersję zamkniętą. Wersja nie obowiązuje wstecz.
CREATE TABLE wykonawcy_cenniki_wpisy (
  id TEXT PRIMARY KEY,
  -- Np. „Sprzątanie Stare Miasto”.
  nazwa TEXT NOT NULL CHECK (nazwa <> ''),
  wpisano TEXT NOT NULL
) STRICT;

CREATE TABLE wykonawcy_cenniki_wersje (
  id TEXT PRIMARY KEY,
  cennik_id TEXT NOT NULL REFERENCES wykonawcy_cenniki_wpisy (id),
  -- Kolejne numery od 1.
  numer INTEGER NOT NULL CHECK (numer >= 1),
  -- Dzień, od którego wersja obowiązuje.
  obowiazuje_od TEXT NOT NULL,
  -- Chwila wpisu do rejestru.
  wpisano TEXT NOT NULL,
  stan TEXT NOT NULL CHECK (stan IN ('projekt', 'zamknieta')),
  decyzja_id TEXT NOT NULL REFERENCES wykonawcy_spis_decyzje (id),

  -- Wersja nie obowiązuje wstecz: dzień obowiązywania nie jest wcześniejszy niż dzień wpisu.
  CONSTRAINT obowiazuje_od_nie_przed_wpisem CHECK (
    obowiazuje_od = date(obowiazuje_od)
    AND obowiazuje_od >= date(wpisano)
  ),

  UNIQUE (cennik_id, numer)
) STRICT;

-- Pozycje wersji: usługa albo dopłata, jednostka i cena w groszach.
CREATE TABLE wykonawcy_cenniki_pozycje (
  -- Klucz nadany przez bazę jest dodatni; zob. wykonawcy_cenniki_pozycje_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  wersja_id TEXT NOT NULL REFERENCES wykonawcy_cenniki_wersje (id),
  -- Np. „sprzątanie po wyjeździe gościa”, „praca w święto”.
  usluga TEXT NOT NULL CHECK (usluga <> ''),
  -- Dopłata do usługi, np. za okno krótsze niż trzy godziny albo dojazd poza obszar.
  doplata INTEGER NOT NULL CHECK (doplata IN (0, 1)),
  -- Np. „lokal do 40 m²”, „komplet pościeli”, „godzina”, „wizyta”.
  jednostka TEXT NOT NULL CHECK (jednostka <> ''),
  cena INTEGER NOT NULL CHECK (cena >= 0),
  -- Cena zawiera podatek VAT.
  z_vat INTEGER NOT NULL CHECK (z_vat IN (0, 1)),
  -- Np. „najkrótsza wizyta 1 godzina”, „odwołanie bez opłaty do 24 godzin przed”.
  warunki TEXT
) STRICT;

-- Wykonawcy, którzy pracują według cennika, od dnia do dnia, z decyzją przypisania i decyzją
-- jego końca.
CREATE TABLE wykonawcy_cenniki_przypisania (
  -- Klucz nadany przez bazę jest dodatni; zob. wykonawcy_cenniki_przypisania_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  cennik_id TEXT NOT NULL REFERENCES wykonawcy_cenniki_wpisy (id),
  wykonawca_id TEXT NOT NULL REFERENCES wykonawcy_spis_wpisy (id),
  od TEXT NOT NULL,
  do TEXT,
  decyzja_id TEXT NOT NULL REFERENCES wykonawcy_spis_decyzje (id),
  zakonczenie_decyzja_id TEXT REFERENCES wykonawcy_spis_decyzje (id),

  CONSTRAINT koniec_z_decyzja CHECK ((do IS NULL) = (zakonczenie_decyzja_id IS NULL)),
  CONSTRAINT koniec_nie_przed_poczatkiem CHECK (do IS NULL OR do >= od)
) STRICT;

-- Wersje ---------------------------------------------------------------------------------------

-- Pozycje dopisuje się do projektu, a zamknięcie sprawdza zmianę stanu, więc wersja nie może
-- powstać od razu zamknięta.
CREATE TRIGGER wykonawcy_cenniki_wersja_powstaje_jako_projekt
BEFORE INSERT ON wykonawcy_cenniki_wersje
WHEN NEW.stan <> 'projekt'
BEGIN
  SELECT RAISE(ABORT, 'wersja cennika powstaje jako projekt');
END;

CREATE TRIGGER wykonawcy_cenniki_wersja_po_decyzji_o_cenniku
BEFORE INSERT ON wykonawcy_cenniki_wersje
WHEN (
  SELECT rodzaj
  FROM wykonawcy_spis_decyzje
  WHERE id = NEW.decyzja_id
) IS NOT 'cennik'
BEGIN
  SELECT RAISE(ABORT, 'wersję cennika wpisuje się po decyzji o cenniku');
END;

-- Kolejna wersja ma kolejny numer i nie obowiązuje wcześniej niż poprzednia, więc obowiązującą
-- wersję wyznacza dzień.
CREATE TRIGGER wykonawcy_cenniki_wersja_kolejna
BEFORE INSERT ON wykonawcy_cenniki_wersje
WHEN NEW.numer IS NOT (
    SELECT coalesce(max(numer), 0) + 1
    FROM wykonawcy_cenniki_wersje
    WHERE cennik_id = NEW.cennik_id
  )
  OR NEW.obowiazuje_od < (
    SELECT max(obowiazuje_od)
    FROM wykonawcy_cenniki_wersje
    WHERE cennik_id = NEW.cennik_id
  )
BEGIN
  SELECT RAISE(ABORT, 'wersja cennika ma kolejny numer i nie obowiązuje wcześniej niż poprzednia');
END;

CREATE TRIGGER wykonawcy_cenniki_zamknieta_ma_pozycje
BEFORE UPDATE OF stan ON wykonawcy_cenniki_wersje
WHEN NEW.stan = 'zamknieta'
  AND NOT EXISTS (
    SELECT 1
    FROM wykonawcy_cenniki_pozycje
    WHERE wersja_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'zamknięta wersja cennika ma co najmniej jedną pozycję');
END;

-- Jedyną zmianą wersji jest zamknięcie projektu; podpis decyzji obejmuje całą wersję.
CREATE TRIGGER wykonawcy_cenniki_wersje_bez_zmian
BEFORE UPDATE ON wykonawcy_cenniki_wersje
WHEN OLD.stan <> 'projekt'
  OR NEW.id IS NOT OLD.id
  OR NEW.cennik_id IS NOT OLD.cennik_id
  OR NEW.numer IS NOT OLD.numer
  OR NEW.obowiazuje_od IS NOT OLD.obowiazuje_od
  OR NEW.wpisano IS NOT OLD.wpisano
  OR NEW.decyzja_id IS NOT OLD.decyzja_id
BEGIN
  SELECT RAISE(ABORT, 'wersji cennika się nie zmienia; nowa decyzja to nowa wersja');
END;

-- Pozycje --------------------------------------------------------------------------------------

CREATE TRIGGER wykonawcy_cenniki_pozycje_tylko_w_projekcie
BEFORE INSERT ON wykonawcy_cenniki_pozycje
WHEN (
  SELECT stan
  FROM wykonawcy_cenniki_wersje
  WHERE id = NEW.wersja_id
) IS NOT 'projekt'
BEGIN
  SELECT RAISE(ABORT, 'pozycje dopisuje się tylko do projektu wersji cennika');
END;

-- Pomyłkę w pozycji projektu poprawia się przed zamknięciem; potem pozycji się nie zmienia.
CREATE TRIGGER wykonawcy_cenniki_pozycje_bez_zmian
BEFORE UPDATE ON wykonawcy_cenniki_pozycje
WHEN NEW.id IS NOT OLD.id
  OR NEW.wersja_id IS NOT OLD.wersja_id
  OR (
    SELECT stan
    FROM wykonawcy_cenniki_wersje
    WHERE id = OLD.wersja_id
  ) IS NOT 'projekt'
BEGIN
  SELECT RAISE(ABORT, 'pozycji zamkniętej wersji cennika się nie zmienia');
END;

-- Przypisania ----------------------------------------------------------------------------------

-- Wykonawca pracuje według cennika, który ma zamkniętą wersję, i tylko po decyzji o przypisaniu.
CREATE TRIGGER wykonawcy_cenniki_przypisanie_po_decyzji
BEFORE INSERT ON wykonawcy_cenniki_przypisania
WHEN (
  SELECT rodzaj
  FROM wykonawcy_spis_decyzje
  WHERE id = NEW.decyzja_id
) IS NOT 'przypisanie'
BEGIN
  SELECT RAISE(ABORT, 'przypisanie cennika i jego koniec wpisuje się po decyzji o przypisaniu');
END;

CREATE TRIGGER wykonawcy_cenniki_koniec_przypisania_po_decyzji
BEFORE UPDATE OF zakonczenie_decyzja_id ON wykonawcy_cenniki_przypisania
WHEN NEW.zakonczenie_decyzja_id IS NOT NULL
  AND (
    SELECT rodzaj
    FROM wykonawcy_spis_decyzje
    WHERE id = NEW.zakonczenie_decyzja_id
  ) IS NOT 'przypisanie'
BEGIN
  SELECT RAISE(ABORT, 'przypisanie cennika i jego koniec wpisuje się po decyzji o przypisaniu');
END;

CREATE TRIGGER wykonawcy_cenniki_przypisanie_cennika_z_wersja
BEFORE INSERT ON wykonawcy_cenniki_przypisania
WHEN NOT EXISTS (
  SELECT 1
  FROM wykonawcy_cenniki_wersje
  WHERE cennik_id = NEW.cennik_id
    AND stan = 'zamknieta'
)
BEGIN
  SELECT RAISE(ABORT, 'przypisuje się cennik, który ma zamkniętą wersję');
END;

-- Wstrzymany i wykreślony wykonawca nie dostaje zleceń, więc nie dostaje też nowego cennika.
CREATE TRIGGER wykonawcy_cenniki_przypisanie_zatwierdzonego
BEFORE INSERT ON wykonawcy_cenniki_przypisania
WHEN (
  SELECT stan
  FROM wykonawcy_spis_stany
  WHERE wykonawca_id = NEW.wykonawca_id
  ORDER BY id DESC
  LIMIT 1
) IS NOT 'zatwierdzony'
BEGIN
  SELECT RAISE(ABORT, 'cennik przypisuje się zatwierdzonemu wykonawcy');
END;

CREATE TRIGGER wykonawcy_cenniki_przypisanie_powstaje_biezace
BEFORE INSERT ON wykonawcy_cenniki_przypisania
WHEN NEW.do IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'przypisanie cennika powstaje jako bieżące');
END;

CREATE TRIGGER wykonawcy_cenniki_przypisania_bez_zmian
BEFORE UPDATE ON wykonawcy_cenniki_przypisania
WHEN NEW.id IS NOT OLD.id
  OR NEW.cennik_id IS NOT OLD.cennik_id
  OR NEW.wykonawca_id IS NOT OLD.wykonawca_id
  OR NEW.od IS NOT OLD.od
  OR NEW.decyzja_id IS NOT OLD.decyzja_id
  OR (OLD.do IS NOT NULL AND (NEW.do IS NOT OLD.do OR NEW.zakonczenie_decyzja_id IS NOT OLD.zakonczenie_decyzja_id))
BEGIN
  SELECT RAISE(ABORT, 'przypisania cennika się nie zmienia; może się tylko raz zakończyć decyzją');
END;

-- Wiersze bez zmian ----------------------------------------------------------------------------

CREATE TRIGGER wykonawcy_cenniki_wpisy_bez_zmian
BEFORE UPDATE ON wykonawcy_cenniki_wpisy
BEGIN
  SELECT RAISE(ABORT, 'cennika się nie zmienia; zmiana stawek to nowa wersja');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabel z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER wykonawcy_cenniki_wpisy_bez_zastepowania
BEFORE INSERT ON wykonawcy_cenniki_wpisy
WHEN EXISTS (
  SELECT 1
  FROM wykonawcy_cenniki_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER wykonawcy_cenniki_wersje_bez_zastepowania
BEFORE INSERT ON wykonawcy_cenniki_wersje
WHEN EXISTS (
    SELECT 1
    FROM wykonawcy_cenniki_wersje
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM wykonawcy_cenniki_wersje
    WHERE cennik_id = NEW.cennik_id
      AND numer = NEW.numer
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER wykonawcy_cenniki_pozycje_bez_zastepowania
BEFORE INSERT ON wykonawcy_cenniki_pozycje
WHEN EXISTS (
  SELECT 1
  FROM wykonawcy_cenniki_pozycje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER wykonawcy_cenniki_przypisania_bez_zastepowania
BEFORE INSERT ON wykonawcy_cenniki_przypisania
WHEN EXISTS (
  SELECT 1
  FROM wykonawcy_cenniki_przypisania
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER wykonawcy_cenniki_wpisy_bez_usuwania
BEFORE DELETE ON wykonawcy_cenniki_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER wykonawcy_cenniki_wersje_bez_usuwania
BEFORE DELETE ON wykonawcy_cenniki_wersje
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER wykonawcy_cenniki_pozycje_bez_usuwania
BEFORE DELETE ON wykonawcy_cenniki_pozycje
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER wykonawcy_cenniki_przypisania_bez_usuwania
BEFORE DELETE ON wykonawcy_cenniki_przypisania
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;
"#,
    ],
};

/// The values of column `wersje.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WersjeStan {
    Projekt,
    Zamknieta,
}

impl WersjeStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WersjeStan] = &[WersjeStan::Projekt, WersjeStan::Zamknieta];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WersjeStan::Projekt => "projekt",
            WersjeStan::Zamknieta => "zamknieta",
        }
    }
}

impl xmeta_room::FromSqlValue for WersjeStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "projekt" => Ok(WersjeStan::Projekt),
                    "zamknieta" => Ok(WersjeStan::Zamknieta),
                    other => Err(format!("{other} is no value of WersjeStan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WersjeStan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The parameters of `pozycje_cennika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PozycjeCennikaParams {
    pub wersja_id: String,
}

/// A row of `pozycje_cennika`.
#[derive(Debug, Clone, PartialEq)]
pub struct PozycjeCennikaRow {
    pub id: i64,
    pub wersja_id: String,
    pub usluga: String,
    pub doplata: bool,
    pub jednostka: String,
    pub cena: i64,
    pub z_vat: bool,
    pub warunki: Option<String>,
}

impl PozycjeCennikaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PozycjeCennikaRow {
            id: row.get("id")?,
            wersja_id: row.get("wersja_id")?,
            usluga: row.get("usluga")?,
            doplata: row.get("doplata")?,
            jednostka: row.get("jednostka")?,
            cena: row.get("cena")?,
            z_vat: row.get("z_vat")?,
            warunki: row.get("warunki")?,
        })
    }
}

/// Pozycje jednej wersji cennika.
pub fn pozycje_cennika(
    room: &impl xmeta_room::MetaRoom,
    params: &PozycjeCennikaParams,
) -> Result<Vec<PozycjeCennikaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM wykonawcy_cenniki_pozycje
WHERE wersja_id = :wersja_id
ORDER BY id"#,
        parameters: Vec::from([
            ("wersja_id", xmeta_room::ToSqlValue::to_sql_value(&params.wersja_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(PozycjeCennikaRow::from_row).collect()
}

/// The parameters of `dopisz_pozycje_cennika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DopiszPozycjeCennikaParams {
    pub wersja_id: String,
    pub usluga: String,
    pub doplata: bool,
    pub jednostka: String,
    pub cena: i64,
    pub z_vat: bool,
    pub warunki: Option<String>,
}

/// Usługa albo dopłata w projekcie wersji, z ceną w groszach.
pub fn dopisz_pozycje_cennika(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszPozycjeCennikaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wykonawcy_cenniki_pozycje (wersja_id, usluga, doplata, jednostka, cena, z_vat, warunki)
VALUES (:wersja_id, :usluga, :doplata, :jednostka, :cena, :z_vat, :warunki)"#,
                parameters: Vec::from([
                    (
                        "wersja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wersja_id),
                    ),
                    ("usluga", xmeta_room::ToSqlValue::to_sql_value(&params.usluga)),
                    ("doplata", xmeta_room::ToSqlValue::to_sql_value(&params.doplata)),
                    (
                        "jednostka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.jednostka),
                    ),
                    ("cena", xmeta_room::ToSqlValue::to_sql_value(&params.cena)),
                    ("z_vat", xmeta_room::ToSqlValue::to_sql_value(&params.z_vat)),
                    ("warunki", xmeta_room::ToSqlValue::to_sql_value(&params.warunki)),
                ]),
            },
        ],
    )
}

/// The parameters of `popraw_pozycje_cennika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PoprawPozycjeCennikaParams {
    pub usluga: String,
    pub doplata: bool,
    pub jednostka: String,
    pub cena: i64,
    pub z_vat: bool,
    pub warunki: Option<String>,
    pub id: i64,
}

/// Poprawka pozycji, dopóki wersja jest projektem.
pub fn popraw_pozycje_cennika(
    room: &impl xmeta_room::MetaRoom,
    params: &PoprawPozycjeCennikaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE wykonawcy_cenniki_pozycje
SET usluga = :usluga,
  doplata = :doplata,
  jednostka = :jednostka,
  cena = :cena,
  z_vat = :z_vat,
  warunki = :warunki
WHERE id = :id"#,
                parameters: Vec::from([
                    ("usluga", xmeta_room::ToSqlValue::to_sql_value(&params.usluga)),
                    ("doplata", xmeta_room::ToSqlValue::to_sql_value(&params.doplata)),
                    (
                        "jednostka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.jednostka),
                    ),
                    ("cena", xmeta_room::ToSqlValue::to_sql_value(&params.cena)),
                    ("z_vat", xmeta_room::ToSqlValue::to_sql_value(&params.z_vat)),
                    ("warunki", xmeta_room::ToSqlValue::to_sql_value(&params.warunki)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `przypisania_cennikow`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzypisaniaCennikowRow {
    pub id: i64,
    pub cennik_id: String,
    pub wykonawca_id: String,
    pub od: String,
    pub r#do: Option<String>,
    pub decyzja_id: String,
    pub zakonczenie_decyzja_id: Option<String>,
}

impl PrzypisaniaCennikowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PrzypisaniaCennikowRow {
            id: row.get("id")?,
            cennik_id: row.get("cennik_id")?,
            wykonawca_id: row.get("wykonawca_id")?,
            od: row.get("od")?,
            r#do: row.get("do")?,
            decyzja_id: row.get("decyzja_id")?,
            zakonczenie_decyzja_id: row.get("zakonczenie_decyzja_id")?,
        })
    }
}

/// Bieżące przypisania cenników do wykonawców.
pub fn przypisania_cennikow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PrzypisaniaCennikowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM wykonawcy_cenniki_przypisania
WHERE do IS NULL
ORDER BY wykonawca_id, cennik_id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(PrzypisaniaCennikowRow::from_row).collect()
}

/// The parameters of `przypisz_cennik`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrzypiszCennikParams {
    pub cennik_id: String,
    pub wykonawca_id: String,
    pub od: String,
    pub decyzja_id: String,
}

/// Wykonawca pracuje według cennika od dnia, po decyzji o przypisaniu.
pub fn przypisz_cennik(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzypiszCennikParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wykonawcy_cenniki_przypisania (cennik_id, wykonawca_id, od, decyzja_id)
VALUES (:cennik_id, :wykonawca_id, :od, :decyzja_id)"#,
                parameters: Vec::from([
                    (
                        "cennik_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.cennik_id),
                    ),
                    (
                        "wykonawca_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wykonawca_id),
                    ),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    (
                        "decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_id),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zakoncz_przypisanie_cennika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZakonczPrzypisanieCennikaParams {
    pub r#do: Option<String>,
    pub zakonczenie_decyzja_id: String,
    pub id: i64,
}

/// Koniec przypisania po decyzji o przypisaniu, np. gdy wykonawca przechodzi na inny cennik.
pub fn zakoncz_przypisanie_cennika(
    room: &impl xmeta_room::MetaRoom,
    params: &ZakonczPrzypisanieCennikaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE wykonawcy_cenniki_przypisania
SET do = :do,
  zakonczenie_decyzja_id = :zakonczenie_decyzja_id
WHERE id = :id"#,
                parameters: Vec::from([
                    ("do", xmeta_room::ToSqlValue::to_sql_value(&params.r#do)),
                    (
                        "zakonczenie_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.zakonczenie_decyzja_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wersje_cennika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WersjeCennikaParams {
    pub cennik_id: String,
}

/// A row of `wersje_cennika`.
#[derive(Debug, Clone, PartialEq)]
pub struct WersjeCennikaRow {
    pub id: String,
    pub cennik_id: String,
    pub numer: i64,
    pub obowiazuje_od: String,
    pub wpisano: String,
    pub stan: WersjeStan,
    pub decyzja_id: String,
}

impl WersjeCennikaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WersjeCennikaRow {
            id: row.get("id")?,
            cennik_id: row.get("cennik_id")?,
            numer: row.get("numer")?,
            obowiazuje_od: row.get("obowiazuje_od")?,
            wpisano: row.get("wpisano")?,
            stan: row.get("stan")?,
            decyzja_id: row.get("decyzja_id")?,
        })
    }
}

/// Wersje jednego cennika, najnowsza pierwsza.
pub fn wersje_cennika(
    room: &impl xmeta_room::MetaRoom,
    params: &WersjeCennikaParams,
) -> Result<Vec<WersjeCennikaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM wykonawcy_cenniki_wersje
WHERE cennik_id = :cennik_id
ORDER BY numer DESC"#,
        parameters: Vec::from([
            ("cennik_id", xmeta_room::ToSqlValue::to_sql_value(&params.cennik_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(WersjeCennikaRow::from_row).collect()
}

/// The parameters of `obowiazujaca_wersja_cennika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ObowiazujacaWersjaCennikaParams {
    pub cennik_id: String,
    pub dzien: String,
}

/// A row of `obowiazujaca_wersja_cennika`.
#[derive(Debug, Clone, PartialEq)]
pub struct ObowiazujacaWersjaCennikaRow {
    pub id: String,
    pub cennik_id: String,
    pub numer: i64,
    pub obowiazuje_od: String,
    pub wpisano: String,
    pub stan: WersjeStan,
    pub decyzja_id: String,
}

impl ObowiazujacaWersjaCennikaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ObowiazujacaWersjaCennikaRow {
            id: row.get("id")?,
            cennik_id: row.get("cennik_id")?,
            numer: row.get("numer")?,
            obowiazuje_od: row.get("obowiazuje_od")?,
            wpisano: row.get("wpisano")?,
            stan: row.get("stan")?,
            decyzja_id: row.get("decyzja_id")?,
        })
    }
}

/// Zamknięta wersja cennika, która obowiązuje w danym dniu; null przed pierwszą.
pub fn obowiazujaca_wersja_cennika(
    room: &impl xmeta_room::MetaRoom,
    params: &ObowiazujacaWersjaCennikaParams,
) -> Result<Option<ObowiazujacaWersjaCennikaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM wykonawcy_cenniki_wersje
WHERE cennik_id = :cennik_id
  AND stan = 'zamknieta'
  AND obowiazuje_od <= :dzien
ORDER BY numer DESC
LIMIT 1"#,
        parameters: Vec::from([
            ("cennik_id", xmeta_room::ToSqlValue::to_sql_value(&params.cennik_id)),
            ("dzien", xmeta_room::ToSqlValue::to_sql_value(&params.dzien)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.first().map(ObowiazujacaWersjaCennikaRow::from_row).transpose()
}

/// The parameters of `wpisz_wersje_cennika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszWersjeCennikaParams {
    pub id: String,
    pub cennik_id: String,
    pub numer: i64,
    pub obowiazuje_od: String,
    pub wpisano: String,
    pub decyzja_id: String,
}

/// Projekt nowej wersji po decyzji o cenniku; pozycje dopisuje dopisz_pozycje_cennika, a wersję
/// zamyka zamknij_wersje_cennika. Identyfikator (UUID) nadaje aplikacja, bo wskazują go pozycje
/// i zlecenia w teczkach lokali.
pub fn wpisz_wersje_cennika(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszWersjeCennikaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wykonawcy_cenniki_wersje (id, cennik_id, numer, obowiazuje_od, wpisano, stan, decyzja_id)
VALUES (:id, :cennik_id, :numer, :obowiazuje_od, :wpisano, 'projekt', :decyzja_id)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "cennik_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.cennik_id),
                    ),
                    ("numer", xmeta_room::ToSqlValue::to_sql_value(&params.numer)),
                    (
                        "obowiazuje_od",
                        xmeta_room::ToSqlValue::to_sql_value(&params.obowiazuje_od),
                    ),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                    (
                        "decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_id),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zamknij_wersje_cennika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZamknijWersjeCennikaParams {
    pub id: String,
}

/// Wersja ze wszystkimi pozycjami, których skrót objął podpis decyzji; od teraz się nie zmienia.
pub fn zamknij_wersje_cennika(
    room: &impl xmeta_room::MetaRoom,
    params: &ZamknijWersjeCennikaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE wykonawcy_cenniki_wersje
SET stan = 'zamknieta'
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `cenniki`.
#[derive(Debug, Clone, PartialEq)]
pub struct CennikiRow {
    pub id: String,
    pub nazwa: String,
    pub wpisano: String,
}

impl CennikiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(CennikiRow {
            id: row.get("id")?,
            nazwa: row.get("nazwa")?,
            wpisano: row.get("wpisano")?,
        })
    }
}

/// Cenniki rejestru w kolejności wpisania.
pub fn cenniki(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<CennikiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM wykonawcy_cenniki_wpisy
ORDER BY rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(CennikiRow::from_row).collect()
}

/// The parameters of `wpisz_cennik`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszCennikParams {
    pub id: String,
    pub nazwa: String,
    pub wpisano: String,
}

/// Nowy cennik; jego stawki to pierwsza wersja. Identyfikator (UUID) nadaje aplikacja.
pub fn wpisz_cennik(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszCennikParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wykonawcy_cenniki_wpisy (id, nazwa, wpisano)
VALUES (:id, :nazwa, :wpisano)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("nazwa", xmeta_room::ToSqlValue::to_sql_value(&params.nazwa)),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                ]),
            },
        ],
    )
}
