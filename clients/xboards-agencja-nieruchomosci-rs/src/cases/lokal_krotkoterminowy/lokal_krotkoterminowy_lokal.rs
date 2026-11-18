//! The kind `lokal-krotkoterminowy-lokal` of the room `cases/lokal_krotkoterminowy`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "lokal-krotkoterminowy-lokal";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Dane lokalu ponad oznaczenie nieruchomości z rdzenia: rejestracja, łóżka, czas sprzątania,
-- standard i instrukcja wejścia, a do tego regulaminy lokalu, wyposażenie biura, przeglądy
-- i uchwały wspólnoty. Spis wyposażenia przy przejęciu i zwrocie trzyma rdzeń (sprawa-protokoly).
-- Pisze Zarządca obiektu; wyposażenie i przeglądy także Koordynator operacji.
--
-- Kod do zamka nie jest częścią instrukcji wejścia: ustawia się go dla każdego pobytu i zlecenia
-- (lokal-krotkoterminowy-zlecenia).
CREATE TABLE lokal_krotkoterminowy_lokal_dane (
  -- Jeden wiersz na teczkę, bo teczka prowadzi jeden lokal.
  id INTEGER PRIMARY KEY CHECK (id = 1),
  nieruchomosc_id TEXT NOT NULL REFERENCES sprawa_nieruchomosci_wpisy (id),
  -- Rejestracja lokalu, gdy biuro ustali, że jest wymagana (sprawa otwarta w „Agencji nieruchomości”).
  rejestracja_numer TEXT,
  rejestracja_data TEXT,
  lozka INTEGER NOT NULL CHECK (lozka > 0),
  -- Czas sprzątania ze standardu lokalu w minutach; z nim porównuje się okno między pobytami.
  czas_sprzatania_min INTEGER CHECK (czas_sprzatania_min > 0),
  -- Liczba kompletów kluczy, gdy lokal je ma.
  klucze_liczba INTEGER CHECK (klucze_liczba >= 0),
  standard_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  instrukcja_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),

  CONSTRAINT rejestracja_numer_z_data CHECK ((rejestracja_numer IS NULL) = (rejestracja_data IS NULL))
) STRICT;

-- Regulaminy lokalu w kolejności obowiązywania. Bieżący to ostatni.
CREATE TABLE lokal_krotkoterminowy_lokal_regulaminy (
  id TEXT PRIMARY KEY,
  dokument_id TEXT NOT NULL REFERENCES sprawa_dokumenty_wpisy (id),
  od TEXT NOT NULL,
  -- Zgoda człowieka na zmianę regulaminu; pierwszy regulamin jej nie wymaga.
  decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id)
) STRICT;

-- Wyposażenie, które biuro kupiło albo wymieniło, z kosztem i datą; wycofana rzecz dostaje datę
-- końca i powód.
CREATE TABLE lokal_krotkoterminowy_lokal_wyposazenie (
  -- Klucz nadany przez bazę jest dodatni; zob. lokal_krotkoterminowy_lokal_wyposazenie_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  rzecz TEXT NOT NULL CHECK (rzecz <> ''),
  -- Dzień zakupu albo wstawienia.
  od TEXT,
  -- Koszt zakupu w groszach, gdy kupiło biuro na koszt właściciela.
  koszt INTEGER CHECK (koszt >= 0),
  -- Zgoda właściciela na uzupełnienie wyposażenia, gdy jej wymagało.
  zgoda_id TEXT REFERENCES sprawa_zgody_wpisy (id),
  do_dnia TEXT,
  powod TEXT,

  CONSTRAINT koszt_z_data CHECK (koszt IS NULL OR od IS NOT NULL),
  CONSTRAINT wycofanie_z_powodem CHECK ((do_dnia IS NULL) = (powod IS NULL)),
  CONSTRAINT wycofanie_nie_przed_zakupem CHECK (do_dnia IS NULL OR od IS NULL OR do_dnia >= od)
) STRICT;

-- Przeglądy instalacji, czujników i gaśnicy z datą następnego. Które są obowiązkowe, mówi
-- standard lokalu.
CREATE TABLE lokal_krotkoterminowy_lokal_przeglady (
  -- Klucz nadany przez bazę jest dodatni; zob. lokal_krotkoterminowy_lokal_przeglady_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Np. instalacja_elektryczna, instalacja_gazowa, czujnik_dymu, czujnik_co, gasnica.
  rodzaj TEXT NOT NULL CHECK (rodzaj <> '' AND rodzaj NOT GLOB '*[^a-z_]*'),
  data TEXT NOT NULL CHECK (data = date(data)),
  nastepny TEXT CHECK (nastepny = date(nastepny)),
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),

  CONSTRAINT nastepny_po_przegladzie CHECK (nastepny IS NULL OR nastepny > data)
) STRICT;

-- Uchwały wspólnoty albo spółdzielni o najmie krótkoterminowym, z dniem podjęcia i dniem,
-- w którym biuro się o nich dowiedziało (Zarządca zgłasza je w raporcie tego dnia).
CREATE TABLE lokal_krotkoterminowy_lokal_uchwaly (
  -- Klucz nadany przez bazę jest dodatni; zob. lokal_krotkoterminowy_lokal_uchwaly_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  data TEXT NOT NULL CHECK (data = date(data)),
  tresc TEXT NOT NULL CHECK (tresc <> ''),
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  dowiedziano TEXT NOT NULL,

  CONSTRAINT dowiedziano_po_podjeciu CHECK (dowiedziano >= data)
) STRICT;

-- Zmiana regulaminu wymaga zgody człowieka („Sprawy za zgodą”): każdy regulamin po pierwszym
-- wskazuje zatwierdzoną decyzję, która go wskazuje.
CREATE TRIGGER lokal_krotkoterminowy_lokal_zmiana_regulaminu_za_zgoda
BEFORE INSERT ON lokal_krotkoterminowy_lokal_regulaminy
WHEN EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_lokal_regulaminy
  )
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone
    WHERE id = NEW.decyzja_id
      AND rodzaj = 'inna'
      AND dotyczy_tabela = 'lokal_krotkoterminowy_lokal_regulaminy'
      AND dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'każdy regulamin po pierwszym wskazuje zatwierdzoną decyzję inna, która go wskazuje');
END;

-- Regulamin obowiązywał gości od swojej chwili; nowy regulamin to nowy wiersz.
CREATE TRIGGER lokal_krotkoterminowy_lokal_regulaminy_bez_zmian
BEFORE UPDATE ON lokal_krotkoterminowy_lokal_regulaminy
BEGIN
  SELECT RAISE(ABORT, 'regulaminu się nie zmienia; nowy regulamin to nowy wiersz');
END;

CREATE TRIGGER lokal_krotkoterminowy_lokal_zgoda_na_wyposazenie
BEFORE INSERT ON lokal_krotkoterminowy_lokal_wyposazenie
WHEN NEW.zgoda_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_zgody_wpisy
    WHERE id = NEW.zgoda_id
      AND rodzaj = 'zgoda_na_wyposazenie'
  )
BEGIN
  SELECT RAISE(ABORT, 'wyposażenie wskazuje zgodę zgoda_na_wyposazenie');
END;

-- Koszt wyposażenia trafia do zestawienia dla właściciela, więc rzecz, koszt i data się nie
-- zmieniają. Wycofanie wpisuje się raz.
CREATE TRIGGER lokal_krotkoterminowy_lokal_wyposazenie_zmienia_tylko_wycofanie
BEFORE UPDATE ON lokal_krotkoterminowy_lokal_wyposazenie
WHEN OLD.do_dnia IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.rzecz IS NOT OLD.rzecz
  OR NEW.od IS NOT OLD.od
  OR NEW.koszt IS NOT OLD.koszt
  OR NEW.zgoda_id IS NOT OLD.zgoda_id
BEGIN
  SELECT RAISE(ABORT, 'wyposażenia się nie zmienia; wpisuje się tylko jego wycofanie, raz');
END;

CREATE TRIGGER lokal_krotkoterminowy_lokal_przeglady_bez_zmian
BEFORE UPDATE ON lokal_krotkoterminowy_lokal_przeglady
BEGIN
  SELECT RAISE(ABORT, 'przeglądu się nie zmienia; kolejny przegląd to nowy wiersz');
END;

CREATE TRIGGER lokal_krotkoterminowy_lokal_uchwaly_bez_zmian
BEFORE UPDATE ON lokal_krotkoterminowy_lokal_uchwaly
BEGIN
  SELECT RAISE(ABORT, 'uchwały się nie zmienia; nowa uchwała to nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER lokal_krotkoterminowy_lokal_dane_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_lokal_dane
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_lokal_dane
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_lokal_dane_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_lokal_dane
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER lokal_krotkoterminowy_lokal_regulaminy_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_lokal_regulaminy
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_lokal_regulaminy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_lokal_regulaminy_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_lokal_regulaminy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER lokal_krotkoterminowy_lokal_wyposazenie_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_lokal_wyposazenie
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_lokal_wyposazenie
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_lokal_wyposazenie_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_lokal_wyposazenie
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER lokal_krotkoterminowy_lokal_przeglady_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_lokal_przeglady
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_lokal_przeglady
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_lokal_przeglady_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_lokal_przeglady
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER lokal_krotkoterminowy_lokal_uchwaly_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_lokal_uchwaly
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_lokal_uchwaly
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_lokal_uchwaly_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_lokal_uchwaly
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
];

/// A row of `dane_lokalu`.
#[derive(Debug, Clone, PartialEq)]
pub struct DaneLokaluRow {
    pub id: i64,
    pub nieruchomosc_id: String,
    pub rejestracja_numer: Option<String>,
    pub rejestracja_data: Option<String>,
    pub lozka: i64,
    pub czas_sprzatania_min: Option<i64>,
    pub klucze_liczba: Option<i64>,
    pub standard_dokument_id: Option<String>,
    pub instrukcja_dokument_id: Option<String>,
}

impl DaneLokaluRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(DaneLokaluRow {
            id: row.get("id")?,
            nieruchomosc_id: row.get("nieruchomosc_id")?,
            rejestracja_numer: row.get("rejestracja_numer")?,
            rejestracja_data: row.get("rejestracja_data")?,
            lozka: row.get("lozka")?,
            czas_sprzatania_min: row.get("czas_sprzatania_min")?,
            klucze_liczba: row.get("klucze_liczba")?,
            standard_dokument_id: row.get("standard_dokument_id")?,
            instrukcja_dokument_id: row.get("instrukcja_dokument_id")?,
        })
    }
}

/// Dane lokalu: nieruchomość, rejestracja, łóżka, czas sprzątania, klucze, standard i instrukcja.
pub fn dane_lokalu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Option<DaneLokaluRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_lokal_dane"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.first().map(DaneLokaluRow::from_row).transpose()
}

/// The parameters of `wpisz_dane_lokalu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszDaneLokaluParams {
    pub nieruchomosc_id: String,
    pub rejestracja_numer: Option<String>,
    pub rejestracja_data: Option<String>,
    pub lozka: i64,
    pub czas_sprzatania_min: Option<i64>,
    pub klucze_liczba: Option<i64>,
    pub standard_dokument_id: Option<String>,
    pub instrukcja_dokument_id: Option<String>,
}

/// Dane lokalu przy przejęciu w zarząd; teczka ma jeden wiersz.
pub fn wpisz_dane_lokalu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszDaneLokaluParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_lokal_dane (id, nieruchomosc_id, rejestracja_numer,
  rejestracja_data, lozka, czas_sprzatania_min, klucze_liczba, standard_dokument_id,
  instrukcja_dokument_id)
VALUES (1, :nieruchomosc_id, :rejestracja_numer, :rejestracja_data, :lozka, :czas_sprzatania_min,
  :klucze_liczba, :standard_dokument_id, :instrukcja_dokument_id)"#,
                parameters: Vec::from([
                    (
                        "nieruchomosc_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.nieruchomosc_id),
                    ),
                    (
                        "rejestracja_numer",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rejestracja_numer),
                    ),
                    (
                        "rejestracja_data",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rejestracja_data),
                    ),
                    ("lozka", xmeta_room::ToSqlValue::to_sql_value(&params.lozka)),
                    (
                        "czas_sprzatania_min",
                        xmeta_room::ToSqlValue::to_sql_value(&params.czas_sprzatania_min),
                    ),
                    (
                        "klucze_liczba",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klucze_liczba),
                    ),
                    (
                        "standard_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.standard_dokument_id,
                        ),
                    ),
                    (
                        "instrukcja_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.instrukcja_dokument_id,
                        ),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_dane_lokalu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZmienDaneLokaluParams {
    pub rejestracja_numer: Option<String>,
    pub rejestracja_data: Option<String>,
    pub lozka: i64,
    pub czas_sprzatania_min: Option<i64>,
    pub klucze_liczba: Option<i64>,
    pub standard_dokument_id: Option<String>,
    pub instrukcja_dokument_id: Option<String>,
}

/// Nowe dane lokalu, np. numer rejestracji albo nowa wersja standardu czy instrukcji.
pub fn zmien_dane_lokalu(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienDaneLokaluParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_lokal_dane
SET rejestracja_numer = :rejestracja_numer,
  rejestracja_data = :rejestracja_data,
  lozka = :lozka,
  czas_sprzatania_min = :czas_sprzatania_min,
  klucze_liczba = :klucze_liczba,
  standard_dokument_id = :standard_dokument_id,
  instrukcja_dokument_id = :instrukcja_dokument_id
WHERE id = 1"#,
                parameters: Vec::from([
                    (
                        "rejestracja_numer",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rejestracja_numer),
                    ),
                    (
                        "rejestracja_data",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rejestracja_data),
                    ),
                    ("lozka", xmeta_room::ToSqlValue::to_sql_value(&params.lozka)),
                    (
                        "czas_sprzatania_min",
                        xmeta_room::ToSqlValue::to_sql_value(&params.czas_sprzatania_min),
                    ),
                    (
                        "klucze_liczba",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klucze_liczba),
                    ),
                    (
                        "standard_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.standard_dokument_id,
                        ),
                    ),
                    (
                        "instrukcja_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.instrukcja_dokument_id,
                        ),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `przeglady_lokalu`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzegladyLokaluRow {
    pub id: i64,
    pub rodzaj: String,
    pub data: String,
    pub nastepny: Option<String>,
    pub dokument_id: Option<String>,
}

impl PrzegladyLokaluRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PrzegladyLokaluRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            data: row.get("data")?,
            nastepny: row.get("nastepny")?,
            dokument_id: row.get("dokument_id")?,
        })
    }
}

/// Przeglądy lokalu w kolejności dat.
pub fn przeglady_lokalu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PrzegladyLokaluRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_lokal_przeglady
ORDER BY data, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(PrzegladyLokaluRow::from_row).collect()
}

/// The parameters of `wpisz_przeglad_lokalu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszPrzegladLokaluParams {
    pub rodzaj: String,
    pub data: String,
    pub nastepny: Option<String>,
    pub dokument_id: Option<String>,
}

/// Przegląd instalacji, czujnika albo gaśnicy z datą następnego i dokumentem.
pub fn wpisz_przeglad_lokalu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszPrzegladLokaluParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_lokal_przeglady (rodzaj, data, nastepny, dokument_id)
VALUES (:rodzaj, :data, :nastepny, :dokument_id)"#,
                parameters: Vec::from([
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("data", xmeta_room::ToSqlValue::to_sql_value(&params.data)),
                    ("nastepny", xmeta_room::ToSqlValue::to_sql_value(&params.nastepny)),
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `regulaminy_lokalu`.
#[derive(Debug, Clone, PartialEq)]
pub struct RegulaminyLokaluRow {
    pub id: String,
    pub dokument_id: String,
    pub od: String,
    pub decyzja_id: Option<String>,
}

impl RegulaminyLokaluRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(RegulaminyLokaluRow {
            id: row.get("id")?,
            dokument_id: row.get("dokument_id")?,
            od: row.get("od")?,
            decyzja_id: row.get("decyzja_id")?,
        })
    }
}

/// Regulaminy lokalu w kolejności obowiązywania; ostatni jest bieżący.
pub fn regulaminy_lokalu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<RegulaminyLokaluRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_lokal_regulaminy
ORDER BY od, rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(RegulaminyLokaluRow::from_row).collect()
}

/// The parameters of `wpisz_regulamin_lokalu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszRegulaminLokaluParams {
    pub id: String,
    pub dokument_id: String,
    pub od: String,
    pub decyzja_id: Option<String>,
}

/// Regulamin lokalu; każdy po pierwszym wskazuje zatwierdzoną decyzję o zmianie. Identyfikator
/// (UUID) nadaje aplikacja, bo wskazuje go decyzja.
pub fn wpisz_regulamin_lokalu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszRegulaminLokaluParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_lokal_regulaminy (id, dokument_id, od, decyzja_id)
VALUES (:id, :dokument_id, :od, :decyzja_id)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
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

/// A row of `uchwaly_wspolnoty`.
#[derive(Debug, Clone, PartialEq)]
pub struct UchwalyWspolnotyRow {
    pub id: i64,
    pub data: String,
    pub tresc: String,
    pub dokument_id: Option<String>,
    pub dowiedziano: String,
}

impl UchwalyWspolnotyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(UchwalyWspolnotyRow {
            id: row.get("id")?,
            data: row.get("data")?,
            tresc: row.get("tresc")?,
            dokument_id: row.get("dokument_id")?,
            dowiedziano: row.get("dowiedziano")?,
        })
    }
}

/// Uchwały wspólnoty albo spółdzielni o najmie krótkoterminowym w kolejności podjęcia.
pub fn uchwaly_wspolnoty(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<UchwalyWspolnotyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_lokal_uchwaly
ORDER BY data, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(UchwalyWspolnotyRow::from_row).collect()
}

/// The parameters of `wpisz_uchwale_wspolnoty`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszUchwaleWspolnotyParams {
    pub data: String,
    pub tresc: String,
    pub dokument_id: Option<String>,
    pub dowiedziano: String,
}

/// Uchwała z dniem podjęcia, treścią i dniem, w którym biuro się o niej dowiedziało.
pub fn wpisz_uchwale_wspolnoty(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszUchwaleWspolnotyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_lokal_uchwaly (data, tresc, dokument_id, dowiedziano)
VALUES (:data, :tresc, :dokument_id, :dowiedziano)"#,
                parameters: Vec::from([
                    ("data", xmeta_room::ToSqlValue::to_sql_value(&params.data)),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                    (
                        "dowiedziano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dowiedziano),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `wyposazenie_lokalu`.
#[derive(Debug, Clone, PartialEq)]
pub struct WyposazenieLokaluRow {
    pub id: i64,
    pub rzecz: String,
    pub od: Option<String>,
    pub koszt: Option<i64>,
    pub zgoda_id: Option<String>,
    pub do_dnia: Option<String>,
    pub powod: Option<String>,
}

impl WyposazenieLokaluRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WyposazenieLokaluRow {
            id: row.get("id")?,
            rzecz: row.get("rzecz")?,
            od: row.get("od")?,
            koszt: row.get("koszt")?,
            zgoda_id: row.get("zgoda_id")?,
            do_dnia: row.get("do_dnia")?,
            powod: row.get("powod")?,
        })
    }
}

/// Wyposażenie biura w lokalu w kolejności wpisania, także wycofane.
pub fn wyposazenie_lokalu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WyposazenieLokaluRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_lokal_wyposazenie
ORDER BY id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(WyposazenieLokaluRow::from_row).collect()
}

/// The parameters of `wpisz_wyposazenie_lokalu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszWyposazenieLokaluParams {
    pub rzecz: String,
    pub od: Option<String>,
    pub koszt: Option<i64>,
    pub zgoda_id: Option<String>,
}

/// Rzecz kupiona albo wymieniona przez biuro, z kosztem i datą.
pub fn wpisz_wyposazenie_lokalu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszWyposazenieLokaluParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_lokal_wyposazenie (rzecz, od, koszt, zgoda_id)
VALUES (:rzecz, :od, :koszt, :zgoda_id)"#,
                parameters: Vec::from([
                    ("rzecz", xmeta_room::ToSqlValue::to_sql_value(&params.rzecz)),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    ("koszt", xmeta_room::ToSqlValue::to_sql_value(&params.koszt)),
                    ("zgoda_id", xmeta_room::ToSqlValue::to_sql_value(&params.zgoda_id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wycofaj_wyposazenie_lokalu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WycofajWyposazenieLokaluParams {
    pub do_dnia: String,
    pub powod: String,
    pub id: i64,
}

/// Rzecz zużyta, zniszczona albo wymieniona: dzień końca i powód.
pub fn wycofaj_wyposazenie_lokalu(
    room: &impl xmeta_room::MetaRoom,
    params: &WycofajWyposazenieLokaluParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_lokal_wyposazenie
SET do_dnia = :do_dnia,
  powod = :powod
WHERE id = :id"#,
                parameters: Vec::from([
                    ("do_dnia", xmeta_room::ToSqlValue::to_sql_value(&params.do_dnia)),
                    ("powod", xmeta_room::ToSqlValue::to_sql_value(&params.powod)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
