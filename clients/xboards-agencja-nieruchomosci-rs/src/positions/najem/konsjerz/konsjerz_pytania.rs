//! The kind `konsjerz-pytania` of the room `positions/najem/konsjerz`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "konsjerz-pytania";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Pytania gości: o co goście tego lokalu pytają i jaka odpowiedź im pomogła, np. że wjazd do garażu
-- jest od ul. Bocznej. Z tych wpisów Konsjerż proponuje Zarządcy zmiany w opisie lokalu; propozycja
-- idzie notatką w teczce lokalu, a wprowadza ją Zarządca.
--
-- Żadna kolumna nie wskazuje gościa ani rozmowy: pamięć Konsjerża nie może się stać kartoteką
-- gości, a rozmowy leżą w kanale i w teczce. Lepsza odpowiedź na to samo pytanie to nowy wiersz,
-- który zastępuje poprzedni, więc wystąpienia zostają przy odpowiedzi, której dotyczyły.
CREATE TABLE konsjerz_pytania_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. konsjerz_pytania_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Np. „wjazd do garażu”.
  temat TEXT NOT NULL CHECK (temat <> ''),
  odpowiedz TEXT NOT NULL CHECK (odpowiedz <> ''),
  zapisano TEXT NOT NULL,
  -- Kiedy i którą notatką w teczce lokalu Konsjerż zaproponował Zarządcy zmianę opisu lokalu.
  propozycja_przekazano TEXT,
  notatka_odnosnik TEXT,
  zastapione_przez_id INTEGER REFERENCES konsjerz_pytania_wpisy (id),

  CONSTRAINT propozycja_z_notatka CHECK ((propozycja_przekazano IS NULL) = (notatka_odnosnik IS NULL)),
  CONSTRAINT propozycja_nie_przed_zapisem CHECK (propozycja_przekazano IS NULL OR propozycja_przekazano >= zapisano),
  CONSTRAINT zastapione_nie_przez_siebie CHECK (zastapione_przez_id IS NULL OR zastapione_przez_id <> id)
) STRICT;

-- Każde zadanie pytania i to, czy odpowiedź pomogła. Wystąpienie nie wskazuje gościa.
CREATE TABLE konsjerz_pytania_wystapienia (
  -- Klucz nadany przez bazę jest dodatni; zob. konsjerz_pytania_wystapienia_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  pytanie_id INTEGER NOT NULL REFERENCES konsjerz_pytania_wpisy (id),
  zadano TEXT NOT NULL,
  -- 1, gdy odpowiedź pomogła, 0, gdy nie; NULL, dopóki Konsjerż tego nie wie.
  pomogla INTEGER CHECK (pomogla IN (0, 1))
) STRICT;

-- Propozycja i zastąpienie przychodzą później, więc pytanie powstaje aktualne i bez propozycji.
CREATE TRIGGER konsjerz_pytania_powstaje_aktualne
BEFORE INSERT ON konsjerz_pytania_wpisy
WHEN NEW.propozycja_przekazano IS NOT NULL
  OR NEW.zastapione_przez_id IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'pytanie powstaje aktualne i bez propozycji; obie rzeczy odnotowuje się później');
END;

-- Pytanie z odpowiedzią się nie zmienia; wolno raz odnotować propozycję i raz zastąpienie.
CREATE TRIGGER konsjerz_pytania_bez_zmian
BEFORE UPDATE ON konsjerz_pytania_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.temat IS NOT OLD.temat
  OR NEW.odpowiedz IS NOT OLD.odpowiedz
  OR NEW.zapisano IS NOT OLD.zapisano
  OR (OLD.propozycja_przekazano IS NOT NULL AND (
    NEW.propozycja_przekazano IS NOT OLD.propozycja_przekazano
    OR NEW.notatka_odnosnik IS NOT OLD.notatka_odnosnik
  ))
  OR (OLD.zastapione_przez_id IS NOT NULL AND NEW.zastapione_przez_id IS NOT OLD.zastapione_przez_id)
BEGIN
  SELECT RAISE(ABORT, 'pytania ani odpowiedzi się nie zmienia; lepsza odpowiedź to nowy wiersz');
END;

-- Nowa odpowiedź zastępuje odpowiedź na ten sam temat i nie jest starsza od niej.
CREATE TRIGGER konsjerz_pytania_zastepuje_ten_sam_temat
BEFORE UPDATE OF zastapione_przez_id ON konsjerz_pytania_wpisy
WHEN NEW.zastapione_przez_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM konsjerz_pytania_wpisy AS nowe
    WHERE nowe.id = NEW.zastapione_przez_id
      AND nowe.temat = OLD.temat
      AND nowe.zapisano >= OLD.zapisano
  )
BEGIN
  SELECT RAISE(ABORT, 'pytanie zastępuje nowsza odpowiedź na ten sam temat');
END;

-- Wystąpienie dotyczy aktualnej odpowiedzi i nie poprzedza jej zapisania.
CREATE TRIGGER konsjerz_pytania_wystapienie_aktualnej_odpowiedzi
BEFORE INSERT ON konsjerz_pytania_wystapienia
WHEN NOT EXISTS (
  SELECT 1
  FROM konsjerz_pytania_wpisy AS pytanie
  WHERE pytanie.id = NEW.pytanie_id
    AND pytanie.zastapione_przez_id IS NULL
    AND pytanie.zapisano <= NEW.zadano
)
BEGIN
  SELECT RAISE(ABORT, 'wystąpienie dotyczy aktualnej odpowiedzi i nie poprzedza jej zapisania');
END;

CREATE TRIGGER konsjerz_pytania_wystapienie_powstaje_bez_oceny
BEFORE INSERT ON konsjerz_pytania_wystapienia
WHEN NEW.pomogla IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'wystąpienie powstaje bez oceny; to, czy odpowiedź pomogła, odnotowuje się później');
END;

CREATE TRIGGER konsjerz_pytania_wystapienia_bez_zmian
BEFORE UPDATE ON konsjerz_pytania_wystapienia
WHEN OLD.pomogla IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.pytanie_id IS NOT OLD.pytanie_id
  OR NEW.zadano IS NOT OLD.zadano
BEGIN
  SELECT RAISE(ABORT, 'wystąpienia tylko się dopisuje; wolno raz odnotować, czy odpowiedź pomogła');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER konsjerz_pytania_wpisy_bez_zastepowania
BEFORE INSERT ON konsjerz_pytania_wpisy
WHEN EXISTS (
  SELECT 1
  FROM konsjerz_pytania_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER konsjerz_pytania_wystapienia_bez_zastepowania
BEFORE INSERT ON konsjerz_pytania_wystapienia
WHEN EXISTS (
  SELECT 1
  FROM konsjerz_pytania_wystapienia
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: pamięć trwa bez terminu.
CREATE TRIGGER konsjerz_pytania_wpisy_bez_usuwania
BEFORE DELETE ON konsjerz_pytania_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER konsjerz_pytania_wystapienia_bez_usuwania
BEFORE DELETE ON konsjerz_pytania_wystapienia
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
];

/// A row of `pytania_gosci`.
#[derive(Debug, Clone, PartialEq)]
pub struct PytaniaGosciRow {
    pub id: i64,
    pub temat: String,
    pub odpowiedz: String,
    pub zapisano: String,
    pub propozycja_przekazano: Option<String>,
    pub notatka_odnosnik: Option<String>,
    pub zastapione_przez_id: Option<i64>,
    pub wystapienia: i64,
    pub pomocne: i64,
}

impl PytaniaGosciRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PytaniaGosciRow {
            id: row.get("id")?,
            temat: row.get("temat")?,
            odpowiedz: row.get("odpowiedz")?,
            zapisano: row.get("zapisano")?,
            propozycja_przekazano: row.get("propozycja_przekazano")?,
            notatka_odnosnik: row.get("notatka_odnosnik")?,
            zastapione_przez_id: row.get("zastapione_przez_id")?,
            wystapienia: row.get("wystapienia")?,
            pomocne: row.get("pomocne")?,
        })
    }
}

/// Aktualne odpowiedzi z liczbą wystąpień i liczbą tych, w których pomogły; najczęstsze pierwsze.
pub fn pytania_gosci(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PytaniaGosciRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT pytanie.*,
  count(wystapienie.id) AS wystapienia,
  coalesce(sum(wystapienie.pomogla), 0) AS pomocne
FROM konsjerz_pytania_wpisy AS pytanie
LEFT JOIN konsjerz_pytania_wystapienia AS wystapienie
  ON wystapienie.pytanie_id = pytanie.id
WHERE pytanie.zastapione_przez_id IS NULL
GROUP BY pytanie.id
ORDER BY wystapienia DESC, pytanie.temat, pytanie.id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(PytaniaGosciRow::from_row).collect()
}

/// The parameters of `zapisz_pytanie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZapiszPytanieParams {
    pub temat: String,
    pub odpowiedz: String,
    pub zapisano: String,
}

/// Nowe pytanie z odpowiedzią, bez danych gościa.
pub fn zapisz_pytanie(
    room: &impl xmeta_room::MetaRoom,
    params: &ZapiszPytanieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO konsjerz_pytania_wpisy (temat, odpowiedz, zapisano)
VALUES (:temat, :odpowiedz, :zapisano)"#,
                parameters: Vec::from([
                    ("temat", xmeta_room::ToSqlValue::to_sql_value(&params.temat)),
                    (
                        "odpowiedz",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odpowiedz),
                    ),
                    ("zapisano", xmeta_room::ToSqlValue::to_sql_value(&params.zapisano)),
                ]),
            },
        ],
    )
}

/// The parameters of `zastap_odpowiedz`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZastapOdpowiedzParams {
    pub zastapione_przez_id: i64,
    pub id: i64,
}

/// Poprzednia odpowiedź staje się nieaktualna, bo zastępuje ją lepsza na ten sam temat.
pub fn zastap_odpowiedz(
    room: &impl xmeta_room::MetaRoom,
    params: &ZastapOdpowiedzParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE konsjerz_pytania_wpisy
SET zastapione_przez_id = :zastapione_przez_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zastapione_przez_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zastapione_przez_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_propozycje_opisu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPropozycjeOpisuParams {
    pub propozycja_przekazano: String,
    pub notatka_odnosnik: String,
    pub id: i64,
}

/// Konsjerż zaproponował Zarządcy zmianę opisu lokalu notatką w teczce lokalu.
pub fn odnotuj_propozycje_opisu(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPropozycjeOpisuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE konsjerz_pytania_wpisy
SET propozycja_przekazano = :propozycja_przekazano,
  notatka_odnosnik = :notatka_odnosnik
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "propozycja_przekazano",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.propozycja_przekazano,
                        ),
                    ),
                    (
                        "notatka_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.notatka_odnosnik),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `dopisz_wystapienie_pytania`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DopiszWystapieniePytaniaParams {
    pub pytanie_id: i64,
    pub zadano: String,
}

/// Gość zadał pytanie, na które Konsjerż ma odpowiedź.
pub fn dopisz_wystapienie_pytania(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszWystapieniePytaniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO konsjerz_pytania_wystapienia (pytanie_id, zadano)
VALUES (:pytanie_id, :zadano)"#,
                parameters: Vec::from([
                    (
                        "pytanie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.pytanie_id),
                    ),
                    ("zadano", xmeta_room::ToSqlValue::to_sql_value(&params.zadano)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_czy_odpowiedz_pomogla`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujCzyOdpowiedzPomoglaParams {
    pub pomogla: bool,
    pub id: i64,
}

/// Konsjerż wie już, czy odpowiedź pomogła gościowi.
pub fn odnotuj_czy_odpowiedz_pomogla(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujCzyOdpowiedzPomoglaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE konsjerz_pytania_wystapienia
SET pomogla = :pomogla
WHERE id = :id"#,
                parameters: Vec::from([
                    ("pomogla", xmeta_room::ToSqlValue::to_sql_value(&params.pomogla)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
