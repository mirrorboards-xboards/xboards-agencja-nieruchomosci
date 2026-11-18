//! The kind `kierownik-dzialu-polisy` of the room `positions/posrednictwo/kierownik_posrednictwa`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "kierownik-dzialu-polisy",
    requires: &[&super::kierownik_dzialu_raporty::KIND],
    migrations: &[
        r#"-- Rozesłanie polisy: które umowy pośrednictwa albo o zarządzanie dostały kopię nowej polisy i kiedy.
-- Biuro niezwłocznie przekazuje klientowi kopię nowej polisy (art. 181 ust. 3a i art. 186 ust. 3a
-- ugn), a kierownik działu poleca to każdemu agentowi z otwartą umową. Wiersz powstaje z raportu
-- agenta, który kopię wysłał.
--
-- Polisę prowadzi Kierownik biura w swojej pamięci; tu jest jej numer i odnośnik do tamtego wpisu.
CREATE TABLE kierownik_dzialu_polisy_rozeslania (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_dzialu_polisy_rozeslania_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  polisa_numer TEXT NOT NULL CHECK (polisa_numer <> ''),
  -- Wpis polisy w pamięci Kierownika biura, tekstem.
  polisa_odnosnik TEXT,
  -- Umowa z biurem w teczce, tekstem: pokój i wiersz.
  umowa_odnosnik TEXT NOT NULL CHECK (umowa_odnosnik <> ''),
  raport_id TEXT NOT NULL REFERENCES kierownik_dzialu_raporty_wpisy (id),
  rozeslano TEXT NOT NULL,

  -- Umowa dostaje kopię jednej polisy raz.
  CONSTRAINT polisa_raz_do_umowy UNIQUE (polisa_numer, umowa_odnosnik)
) STRICT;

CREATE TRIGGER kierownik_dzialu_polisy_bez_zmian
BEFORE UPDATE ON kierownik_dzialu_polisy_rozeslania
BEGIN
  SELECT RAISE(ABORT, 'rozesłania polisy się nie zmienia; jest śladem wysłanej kopii');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers; także wiersz tej samej
-- polisy i umowy. Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza,
-- bo klucze są dodatnie.
CREATE TRIGGER kierownik_dzialu_polisy_rozeslania_bez_zastepowania
BEFORE INSERT ON kierownik_dzialu_polisy_rozeslania
WHEN EXISTS (
  SELECT 1
  FROM kierownik_dzialu_polisy_rozeslania
  WHERE id = NEW.id
    OR (polisa_numer = NEW.polisa_numer AND umowa_odnosnik = NEW.umowa_odnosnik)
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: ślad wysłanej kopii zostaje.
CREATE TRIGGER kierownik_dzialu_polisy_rozeslania_bez_usuwania
BEFORE DELETE ON kierownik_dzialu_polisy_rozeslania
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
    ],
};

/// The parameters of `rozeslania_polisy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RozeslaniaPolisyParams {
    pub polisa_numer: String,
}

/// A row of `rozeslania_polisy`.
#[derive(Debug, Clone, PartialEq)]
pub struct RozeslaniaPolisyRow {
    pub id: i64,
    pub polisa_numer: String,
    pub polisa_odnosnik: Option<String>,
    pub umowa_odnosnik: String,
    pub raport_id: String,
    pub rozeslano: String,
}

impl RozeslaniaPolisyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(RozeslaniaPolisyRow {
            id: row.get("id")?,
            polisa_numer: row.get("polisa_numer")?,
            polisa_odnosnik: row.get("polisa_odnosnik")?,
            umowa_odnosnik: row.get("umowa_odnosnik")?,
            raport_id: row.get("raport_id")?,
            rozeslano: row.get("rozeslano")?,
        })
    }
}

/// Umowy, które dostały kopię jednej polisy, od najwcześniejszego wysłania.
pub fn rozeslania_polisy(
    room: &impl xmeta_room::MetaRoom,
    params: &RozeslaniaPolisyParams,
) -> Result<Vec<RozeslaniaPolisyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_dzialu_polisy_rozeslania
WHERE polisa_numer = :polisa_numer
ORDER BY rozeslano, id"#,
        parameters: Vec::from([
            ("polisa_numer", xmeta_room::ToSqlValue::to_sql_value(&params.polisa_numer)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(RozeslaniaPolisyRow::from_row).collect()
}

/// The parameters of `odnotuj_rozeslanie_polisy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujRozeslaniePolisyParams {
    pub polisa_numer: String,
    pub polisa_odnosnik: Option<String>,
    pub umowa_odnosnik: String,
    pub raport_id: String,
    pub rozeslano: String,
}

/// Agent wysłał klientowi kopię polisy i podał to w raporcie.
pub fn odnotuj_rozeslanie_polisy(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujRozeslaniePolisyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_dzialu_polisy_rozeslania (polisa_numer, polisa_odnosnik, umowa_odnosnik, raport_id,
  rozeslano)
VALUES (:polisa_numer, :polisa_odnosnik, :umowa_odnosnik, :raport_id, :rozeslano)"#,
                parameters: Vec::from([
                    (
                        "polisa_numer",
                        xmeta_room::ToSqlValue::to_sql_value(&params.polisa_numer),
                    ),
                    (
                        "polisa_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.polisa_odnosnik),
                    ),
                    (
                        "umowa_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.umowa_odnosnik),
                    ),
                    (
                        "raport_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.raport_id),
                    ),
                    (
                        "rozeslano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rozeslano),
                    ),
                ]),
            },
        ],
    )
}
