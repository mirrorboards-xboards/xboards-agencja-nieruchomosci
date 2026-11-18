//! The kind `stanowisko-kontakty` of the room `positions/transakcje/analityk_kredytowy`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "stanowisko-kontakty";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Kontakty: kto poza teczką zajmuje się sprawą, którym kanałem i o jakiej porze odpowiada, np.
-- administrator wspólnoty, kancelaria, bank, spółdzielnia albo wykonawca.
--
-- Uczestnika teczki albo wpis rejestru (np. wykonawcę z rejestru wykonawców) wskazuje odnośnik, bez
-- nazwy i adresu, bo te dane trzyma tamten pokój. Zmiana kontaktu to nowy wiersz, a stary dostaje
-- datę nieaktualności i zostaje jako historia.
CREATE TABLE stanowisko_kontakty_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. stanowisko_kontakty_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Teczka albo teczka lokalu, której kontakt dotyczy, tekstem. Brak oznacza kontakt całego obszaru
  -- albo biura, np. dostawcę zapasów albo administrację budynku z wieloma lokalami.
  dotyczy_odnosnik TEXT CHECK (dotyczy_odnosnik IS NULL OR dotyczy_odnosnik <> ''),
  -- Np. „Wspólnota Mieszkaniowa Polna 7” albo „Bank Przykładowy”.
  instytucja TEXT,
  -- Np. „administrator”, „doradca kredytowy”, „sekretariat kancelarii”.
  rola TEXT NOT NULL CHECK (rola <> ''),
  -- Pokój i wiersz uczestnika teczki albo wpisu rejestru, tekstem.
  wpis_odnosnik TEXT,
  nazwa TEXT,
  -- Adres w kanale: adres poczty, numer telefonu albo adres pocztowy.
  adres TEXT,
  kanal TEXT NOT NULL CHECK (kanal IN ('email', 'telefon', 'sms', 'portal', 'kanal_sprzedazy', 'list', 'osobiscie', 'pokoj_wymiany')),
  -- Np. „dni robocze 8–12”.
  pory TEXT,
  zapisano TEXT NOT NULL,
  nieaktualny_od TEXT,

  CONSTRAINT kontakt_wskazany_albo_nazwany CHECK ((wpis_odnosnik IS NULL) <> (nazwa IS NULL)),
  CONSTRAINT nazwa_razem_z_adresem CHECK ((nazwa IS NULL) = (adres IS NULL)),
  CONSTRAINT nieaktualny_nie_przed_zapisem CHECK (nieaktualny_od IS NULL OR nieaktualny_od >= zapisano)
) STRICT;

-- Nieaktualność przychodzi przy zmianie kontaktu, więc kontakt powstaje aktualny.
CREATE TRIGGER stanowisko_kontakty_powstaje_aktualny
BEFORE INSERT ON stanowisko_kontakty_wpisy
WHEN NEW.nieaktualny_od IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'kontakt powstaje aktualny; zmiana kontaktu to nowy wiersz');
END;

CREATE TRIGGER stanowisko_kontakty_bez_zmian
BEFORE UPDATE ON stanowisko_kontakty_wpisy
WHEN OLD.nieaktualny_od IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.dotyczy_odnosnik IS NOT OLD.dotyczy_odnosnik
  OR NEW.instytucja IS NOT OLD.instytucja
  OR NEW.rola IS NOT OLD.rola
  OR NEW.wpis_odnosnik IS NOT OLD.wpis_odnosnik
  OR NEW.nazwa IS NOT OLD.nazwa
  OR NEW.adres IS NOT OLD.adres
  OR NEW.kanal IS NOT OLD.kanal
  OR NEW.pory IS NOT OLD.pory
  OR NEW.zapisano IS NOT OLD.zapisano
BEGIN
  SELECT RAISE(ABORT, 'kontaktu się nie zmienia; zmiana kontaktu to nowy wiersz, a stary staje się nieaktualny');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER stanowisko_kontakty_wpisy_bez_zastepowania
BEFORE INSERT ON stanowisko_kontakty_wpisy
WHEN EXISTS (
  SELECT 1
  FROM stanowisko_kontakty_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: stary kontakt zostaje jako historia.
CREATE TRIGGER stanowisko_kontakty_wpisy_bez_usuwania
BEFORE DELETE ON stanowisko_kontakty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
];

/// The values of column `wpisy.kanal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyKanal {
    Email,
    Telefon,
    Sms,
    Portal,
    KanalSprzedazy,
    List,
    Osobiscie,
    PokojWymiany,
}

impl WpisyKanal {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyKanal] = &[
        WpisyKanal::Email,
        WpisyKanal::Telefon,
        WpisyKanal::Sms,
        WpisyKanal::Portal,
        WpisyKanal::KanalSprzedazy,
        WpisyKanal::List,
        WpisyKanal::Osobiscie,
        WpisyKanal::PokojWymiany,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyKanal::Email => "email",
            WpisyKanal::Telefon => "telefon",
            WpisyKanal::Sms => "sms",
            WpisyKanal::Portal => "portal",
            WpisyKanal::KanalSprzedazy => "kanal_sprzedazy",
            WpisyKanal::List => "list",
            WpisyKanal::Osobiscie => "osobiscie",
            WpisyKanal::PokojWymiany => "pokoj_wymiany",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyKanal {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "email" => Ok(WpisyKanal::Email),
                    "telefon" => Ok(WpisyKanal::Telefon),
                    "sms" => Ok(WpisyKanal::Sms),
                    "portal" => Ok(WpisyKanal::Portal),
                    "kanal_sprzedazy" => Ok(WpisyKanal::KanalSprzedazy),
                    "list" => Ok(WpisyKanal::List),
                    "osobiscie" => Ok(WpisyKanal::Osobiscie),
                    "pokoj_wymiany" => Ok(WpisyKanal::PokojWymiany),
                    other => Err(format!("{other} is no value of WpisyKanal")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyKanal {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `kontakty`.
#[derive(Debug, Clone, PartialEq)]
pub struct KontaktyRow {
    pub id: i64,
    pub dotyczy_odnosnik: Option<String>,
    pub instytucja: Option<String>,
    pub rola: String,
    pub wpis_odnosnik: Option<String>,
    pub nazwa: Option<String>,
    pub adres: Option<String>,
    pub kanal: WpisyKanal,
    pub pory: Option<String>,
    pub zapisano: String,
    pub nieaktualny_od: Option<String>,
}

impl KontaktyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(KontaktyRow {
            id: row.get("id")?,
            dotyczy_odnosnik: row.get("dotyczy_odnosnik")?,
            instytucja: row.get("instytucja")?,
            rola: row.get("rola")?,
            wpis_odnosnik: row.get("wpis_odnosnik")?,
            nazwa: row.get("nazwa")?,
            adres: row.get("adres")?,
            kanal: row.get("kanal")?,
            pory: row.get("pory")?,
            zapisano: row.get("zapisano")?,
            nieaktualny_od: row.get("nieaktualny_od")?,
        })
    }
}

/// Aktualne kontakty agenta według teczki i roli.
pub fn kontakty(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<KontaktyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM stanowisko_kontakty_wpisy
WHERE nieaktualny_od IS NULL
ORDER BY dotyczy_odnosnik, rola, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(KontaktyRow::from_row).collect()
}

/// The parameters of `kontakty_sprawy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct KontaktySprawyParams {
    pub dotyczy_odnosnik: String,
}

/// A row of `kontakty_sprawy`.
#[derive(Debug, Clone, PartialEq)]
pub struct KontaktySprawyRow {
    pub id: i64,
    pub dotyczy_odnosnik: Option<String>,
    pub instytucja: Option<String>,
    pub rola: String,
    pub wpis_odnosnik: Option<String>,
    pub nazwa: Option<String>,
    pub adres: Option<String>,
    pub kanal: WpisyKanal,
    pub pory: Option<String>,
    pub zapisano: String,
    pub nieaktualny_od: Option<String>,
}

impl KontaktySprawyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(KontaktySprawyRow {
            id: row.get("id")?,
            dotyczy_odnosnik: row.get("dotyczy_odnosnik")?,
            instytucja: row.get("instytucja")?,
            rola: row.get("rola")?,
            wpis_odnosnik: row.get("wpis_odnosnik")?,
            nazwa: row.get("nazwa")?,
            adres: row.get("adres")?,
            kanal: row.get("kanal")?,
            pory: row.get("pory")?,
            zapisano: row.get("zapisano")?,
            nieaktualny_od: row.get("nieaktualny_od")?,
        })
    }
}

/// Aktualne kontakty jednej teczki albo jednego lokalu.
pub fn kontakty_sprawy(
    room: &impl xmeta_room::MetaRoom,
    params: &KontaktySprawyParams,
) -> Result<Vec<KontaktySprawyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM stanowisko_kontakty_wpisy
WHERE dotyczy_odnosnik = :dotyczy_odnosnik
  AND nieaktualny_od IS NULL
ORDER BY rola, id"#,
        parameters: Vec::from([
            (
                "dotyczy_odnosnik",
                xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_odnosnik),
            ),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(KontaktySprawyRow::from_row).collect()
}

/// The parameters of `zapisz_kontakt`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZapiszKontaktParams {
    pub dotyczy_odnosnik: Option<String>,
    pub instytucja: Option<String>,
    pub rola: String,
    pub wpis_odnosnik: Option<String>,
    pub nazwa: Option<String>,
    pub adres: Option<String>,
    pub kanal: WpisyKanal,
    pub pory: Option<String>,
    pub zapisano: String,
}

/// Nowy kontakt; uczestnika teczki albo wpis rejestru wskazuje odnośnik zamiast nazwy i adresu.
pub fn zapisz_kontakt(
    room: &impl xmeta_room::MetaRoom,
    params: &ZapiszKontaktParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO stanowisko_kontakty_wpisy (dotyczy_odnosnik, instytucja, rola, wpis_odnosnik, nazwa, adres,
  kanal, pory, zapisano)
VALUES (:dotyczy_odnosnik, :instytucja, :rola, :wpis_odnosnik, :nazwa, :adres, :kanal, :pory,
  :zapisano)"#,
                parameters: Vec::from([
                    (
                        "dotyczy_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_odnosnik),
                    ),
                    (
                        "instytucja",
                        xmeta_room::ToSqlValue::to_sql_value(&params.instytucja),
                    ),
                    ("rola", xmeta_room::ToSqlValue::to_sql_value(&params.rola)),
                    (
                        "wpis_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wpis_odnosnik),
                    ),
                    ("nazwa", xmeta_room::ToSqlValue::to_sql_value(&params.nazwa)),
                    ("adres", xmeta_room::ToSqlValue::to_sql_value(&params.adres)),
                    ("kanal", xmeta_room::ToSqlValue::to_sql_value(&params.kanal)),
                    ("pory", xmeta_room::ToSqlValue::to_sql_value(&params.pory)),
                    ("zapisano", xmeta_room::ToSqlValue::to_sql_value(&params.zapisano)),
                ]),
            },
        ],
    )
}

/// The parameters of `uniewaznij_kontakt`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UniewaznijKontaktParams {
    pub nieaktualny_od: String,
    pub id: i64,
}

/// Kontakt przestał być aktualny, np. bo zmienił się administrator albo numer; nowy kontakt to nowy
/// wiersz.
pub fn uniewaznij_kontakt(
    room: &impl xmeta_room::MetaRoom,
    params: &UniewaznijKontaktParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE stanowisko_kontakty_wpisy
SET nieaktualny_od = :nieaktualny_od
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "nieaktualny_od",
                        xmeta_room::ToSqlValue::to_sql_value(&params.nieaktualny_od),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
