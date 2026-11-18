//! The kind `sprawa-wyplaty` of the room `cases/najem`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "sprawa-wyplaty",
    requires: &[
        &super::sprawa_decyzje::KIND,
        &super::sprawa_dokumenty::KIND,
        &super::sprawa_metryka::KIND,
        &super::sprawa_rachunki::KIND,
        &super::sprawa_sprawdzenia::KIND,
        &super::sprawa_uczestnicy::KIND,
    ],
    migrations: &[
        r#"-- Wypłaty pieniędzy klienta, które przechodzą przez biuro, np. zwrot kaucji albo miesięczna
-- wypłata dla właściciela lokalu. Wypłatę przygotowuje agent sprawy z podstawą z typu sprawy
-- i wyliczeniem z programu; agent nie zleca przelewów. Wykonanie wpisuje człowiek, który zlecił
-- przelew, z potwierdzeniem z banku.
--
-- Kolejność jest stała (R4, D13): wypłata powstaje, potem Kontroler sprawdza rachunek odbiorcy
-- na nowo, potem człowiek zgadza się na wypłatę, a dopiero potem przelew. Rachunek musi być
-- dopuszczony decyzją i niezastąpiony (widok sprawa_rachunki_dopuszczone). Limitu dni na
-- świeżość sprawdzenia nie ma.
--
-- Treść wypłaty nie zmienia się od wpisu, bo na niej stoją sprawdzenie i zgoda. Inna kwota, inny
-- odbiorca albo inny rachunek to anulowanie tej wypłaty i nowy wpis.
CREATE TABLE sprawa_wyplaty_wpisy (
  id TEXT PRIMARY KEY,
  -- Wiersz typu sprawy, z którego wynika wypłata, np. rozliczenie kaucji; bez klucza obcego.
  podstawa_tabela TEXT NOT NULL,
  podstawa_id TEXT NOT NULL,
  kwota INTEGER NOT NULL CHECK (kwota > 0),
  -- Plik wyliczenia z programu, ze skrótem.
  wyliczenie_sciezka TEXT NOT NULL,
  wyliczenie_skrot TEXT NOT NULL,
  odbiorca_uczestnik_id TEXT NOT NULL REFERENCES sprawa_uczestnicy_wpisy (id),
  rachunek_id TEXT NOT NULL REFERENCES sprawa_rachunki_wpisy (id),
  sprawdzenie_id TEXT REFERENCES sprawa_sprawdzenia_wpisy (id),
  decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),
  utworzono TEXT NOT NULL,
  stan TEXT NOT NULL CHECK (stan IN ('przygotowana', 'wykonana', 'anulowana')),
  wykonal TEXT,
  wykonano TEXT,
  potwierdzenie_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),

  CONSTRAINT wykonana_ma_wykonawce_i_potwierdzenie CHECK (
    (stan = 'wykonana') = (wykonal IS NOT NULL)
    AND (stan = 'wykonana') = (wykonano IS NOT NULL)
    AND (stan = 'wykonana') = (potwierdzenie_dokument_id IS NOT NULL)
  )
) STRICT;

-- Sprawdzenie i zgoda przychodzą po przygotowaniu wypłaty, więc wiersz powstaje bez nich.
CREATE TRIGGER sprawa_wyplaty_powstaje_przygotowana
BEFORE INSERT ON sprawa_wyplaty_wpisy
WHEN NEW.stan <> 'przygotowana' OR NEW.sprawdzenie_id IS NOT NULL OR NEW.decyzja_id IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'wypłata powstaje jako przygotowana, bez sprawdzenia i zgody');
END;

CREATE TRIGGER sprawa_wyplaty_na_dopuszczony_rachunek
BEFORE INSERT ON sprawa_wyplaty_wpisy
WHEN NOT EXISTS (
  SELECT 1
  FROM sprawa_rachunki_dopuszczone AS rachunek
  WHERE rachunek.id = NEW.rachunek_id
    AND rachunek.uczestnik_id = NEW.odbiorca_uczestnik_id
)
BEGIN
  SELECT RAISE(ABORT, 'wypłata idzie na rachunek odbiorcy dopuszczony decyzją i niezastąpiony');
END;

-- Sprawdzenie Kontrolera i zgoda człowieka dotyczą tej treści wypłaty, a reguła D13 porównuje
-- sprawdzenie z chwilą utworzenia, więc żadnej z tych kolumn się nie zmienia.
CREATE TRIGGER sprawa_wyplaty_tresc_niezmienna
BEFORE UPDATE ON sprawa_wyplaty_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.podstawa_tabela IS NOT OLD.podstawa_tabela
  OR NEW.podstawa_id IS NOT OLD.podstawa_id
  OR NEW.kwota IS NOT OLD.kwota
  OR NEW.wyliczenie_sciezka IS NOT OLD.wyliczenie_sciezka
  OR NEW.wyliczenie_skrot IS NOT OLD.wyliczenie_skrot
  OR NEW.odbiorca_uczestnik_id IS NOT OLD.odbiorca_uczestnik_id
  OR NEW.rachunek_id IS NOT OLD.rachunek_id
  OR NEW.utworzono IS NOT OLD.utworzono
BEGIN
  SELECT RAISE(ABORT, 'treść wypłaty się nie zmienia; inna wypłata to anulowanie i nowy wpis');
END;

-- Wypłata wskazuje sprawdzenie i zgodę raz: podmiana na inne ominęłaby kolejność D13.
CREATE TRIGGER sprawa_wyplaty_sprawdzenie_i_zgoda_raz
BEFORE UPDATE OF sprawdzenie_id, decyzja_id ON sprawa_wyplaty_wpisy
WHEN (OLD.sprawdzenie_id IS NOT NULL AND NEW.sprawdzenie_id IS NOT OLD.sprawdzenie_id)
  OR (OLD.decyzja_id IS NOT NULL AND NEW.decyzja_id IS NOT OLD.decyzja_id)
BEGIN
  SELECT RAISE(ABORT, 'wypłata wskazuje sprawdzenie i zgodę raz; inna wypłata to anulowanie i nowy wpis');
END;

-- D13: Kontroler sprawdza rachunek przed każdą wypłatą, po jej przygotowaniu.
CREATE TRIGGER sprawa_wyplaty_sprawdzenie_po_przygotowaniu
BEFORE UPDATE OF sprawdzenie_id ON sprawa_wyplaty_wpisy
WHEN NEW.sprawdzenie_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_sprawdzenia_wpisy AS sprawdzenie
    WHERE sprawdzenie.id = NEW.sprawdzenie_id
      AND sprawdzenie.rodzaj = 'rachunek'
      AND sprawdzenie.rachunek_id = NEW.rachunek_id
      AND sprawdzenie.aktualne = 1
      AND sprawdzenie.wynik = 'zgodne'
      AND sprawdzenie.sprawdzono >= NEW.utworzono
  )
BEGIN
  SELECT RAISE(ABORT, 'wypłata wskazuje aktualne sprawdzenie „zgodne” jej rachunku zrobione po przygotowaniu wypłaty');
END;

-- Zgoda na wypłatę zapada po sprawdzeniu rachunku, które wypłata wskazuje.
CREATE TRIGGER sprawa_wyplaty_zgoda_po_sprawdzeniu_rachunku
BEFORE UPDATE OF stan ON sprawa_decyzje_wpisy
WHEN NEW.stan = 'rozstrzygnieta'
  AND NEW.rodzaj = 'wyplata'
  AND NEW.rozstrzygniecie IN ('zgoda', 'zgoda_z_warunkiem')
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_wyplaty_wpisy AS wyplata
    JOIN sprawa_sprawdzenia_wpisy AS sprawdzenie ON sprawdzenie.id = wyplata.sprawdzenie_id
    WHERE NEW.dotyczy_tabela = 'sprawa_wyplaty_wpisy'
      AND wyplata.id = NEW.dotyczy_id
      AND sprawdzenie.sprawdzono <= NEW.podpisano
  )
BEGIN
  SELECT RAISE(ABORT, 'zgoda na wypłatę zapada po sprawdzeniu rachunku, które wypłata wskazuje');
END;

CREATE TRIGGER sprawa_wyplaty_decyzja_to_zgoda_na_te_wyplate
BEFORE UPDATE OF decyzja_id ON sprawa_wyplaty_wpisy
WHEN NEW.decyzja_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS decyzja
    WHERE decyzja.id = NEW.decyzja_id
      AND decyzja.rodzaj = 'wyplata'
      AND decyzja.dotyczy_tabela = 'sprawa_wyplaty_wpisy'
      AND decyzja.dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'wypłata wskazuje zatwierdzoną decyzję wyplata, która ją wskazuje');
END;

-- Przy wykonaniu baza sprawdza cały łańcuch jeszcze raz: rachunek mógł zostać zastąpiony,
-- sprawdzenie zastąpione nowszym, a zgoda na wypłatę albo na rachunek zastąpiona nowszą decyzją.
CREATE TRIGGER sprawa_wyplaty_wykonanie_po_zgodzie
BEFORE UPDATE OF stan ON sprawa_wyplaty_wpisy
WHEN NEW.stan = 'wykonana'
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS decyzja
    JOIN sprawa_sprawdzenia_wpisy AS sprawdzenie ON sprawdzenie.id = NEW.sprawdzenie_id
    JOIN sprawa_rachunki_dopuszczone AS rachunek ON rachunek.id = NEW.rachunek_id
    WHERE decyzja.id = NEW.decyzja_id
      AND decyzja.rodzaj = 'wyplata'
      AND decyzja.dotyczy_tabela = 'sprawa_wyplaty_wpisy'
      AND decyzja.dotyczy_id = NEW.id
      AND sprawdzenie.rachunek_id = NEW.rachunek_id
      AND sprawdzenie.aktualne = 1
      AND sprawdzenie.wynik = 'zgodne'
      AND sprawdzenie.sprawdzono >= NEW.utworzono
      AND decyzja.podpisano >= sprawdzenie.sprawdzono
  )
BEGIN
  SELECT RAISE(ABORT, 'wykonanie wymaga zgody na wypłatę po aktualnym sprawdzeniu niezastąpionego i dopuszczonego rachunku');
END;

CREATE TRIGGER sprawa_wyplaty_wykonana_albo_anulowana_ostateczna
BEFORE UPDATE ON sprawa_wyplaty_wpisy
WHEN OLD.stan <> 'przygotowana'
BEGIN
  SELECT RAISE(ABORT, 'wykonanej ani anulowanej wypłaty się nie zmienia');
END;

-- D19: teczki nie zamyka się, dopóki jakaś wypłata czeka na wykonanie albo anulowanie.
CREATE TRIGGER sprawa_wyplaty_zamkniecie_po_wyplatach
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap = 'zamknieta'
  AND EXISTS (
    SELECT 1
    FROM sprawa_wyplaty_wpisy
    WHERE stan = 'przygotowana'
  )
BEGIN
  SELECT RAISE(ABORT, 'teczki nie zamyka się, dopóki jakaś wypłata jest przygotowana');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_wyplaty_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_wyplaty_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_wyplaty_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_wyplaty_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_wyplaty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
    ],
};

/// The values of column `wpisy.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyStan {
    Przygotowana,
    Wykonana,
    Anulowana,
}

impl WpisyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyStan] = &[
        WpisyStan::Przygotowana,
        WpisyStan::Wykonana,
        WpisyStan::Anulowana,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyStan::Przygotowana => "przygotowana",
            WpisyStan::Wykonana => "wykonana",
            WpisyStan::Anulowana => "anulowana",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "przygotowana" => Ok(WpisyStan::Przygotowana),
                    "wykonana" => Ok(WpisyStan::Wykonana),
                    "anulowana" => Ok(WpisyStan::Anulowana),
                    other => Err(format!("{other} is no value of WpisyStan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyStan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `wyplaty`.
#[derive(Debug, Clone, PartialEq)]
pub struct WyplatyRow {
    pub id: String,
    pub podstawa_tabela: String,
    pub podstawa_id: String,
    pub kwota: i64,
    pub wyliczenie_sciezka: String,
    pub wyliczenie_skrot: String,
    pub odbiorca_uczestnik_id: String,
    pub rachunek_id: String,
    pub sprawdzenie_id: Option<String>,
    pub decyzja_id: Option<String>,
    pub utworzono: String,
    pub stan: WpisyStan,
    pub wykonal: Option<String>,
    pub wykonano: Option<String>,
    pub potwierdzenie_dokument_id: Option<String>,
}

impl WyplatyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WyplatyRow {
            id: row.get("id")?,
            podstawa_tabela: row.get("podstawa_tabela")?,
            podstawa_id: row.get("podstawa_id")?,
            kwota: row.get("kwota")?,
            wyliczenie_sciezka: row.get("wyliczenie_sciezka")?,
            wyliczenie_skrot: row.get("wyliczenie_skrot")?,
            odbiorca_uczestnik_id: row.get("odbiorca_uczestnik_id")?,
            rachunek_id: row.get("rachunek_id")?,
            sprawdzenie_id: row.get("sprawdzenie_id")?,
            decyzja_id: row.get("decyzja_id")?,
            utworzono: row.get("utworzono")?,
            stan: row.get("stan")?,
            wykonal: row.get("wykonal")?,
            wykonano: row.get("wykonano")?,
            potwierdzenie_dokument_id: row.get("potwierdzenie_dokument_id")?,
        })
    }
}

/// Wypłaty pieniędzy klienta, najnowsze pierwsze.
pub fn wyplaty(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WyplatyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_wyplaty_wpisy
ORDER BY utworzono DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(WyplatyRow::from_row).collect()
}

/// A row of `wyplaty_przygotowane`.
#[derive(Debug, Clone, PartialEq)]
pub struct WyplatyPrzygotowaneRow {
    pub id: String,
    pub podstawa_tabela: String,
    pub podstawa_id: String,
    pub kwota: i64,
    pub wyliczenie_sciezka: String,
    pub wyliczenie_skrot: String,
    pub odbiorca_uczestnik_id: String,
    pub rachunek_id: String,
    pub sprawdzenie_id: Option<String>,
    pub decyzja_id: Option<String>,
    pub utworzono: String,
    pub stan: WpisyStan,
    pub wykonal: Option<String>,
    pub wykonano: Option<String>,
    pub potwierdzenie_dokument_id: Option<String>,
}

impl WyplatyPrzygotowaneRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WyplatyPrzygotowaneRow {
            id: row.get("id")?,
            podstawa_tabela: row.get("podstawa_tabela")?,
            podstawa_id: row.get("podstawa_id")?,
            kwota: row.get("kwota")?,
            wyliczenie_sciezka: row.get("wyliczenie_sciezka")?,
            wyliczenie_skrot: row.get("wyliczenie_skrot")?,
            odbiorca_uczestnik_id: row.get("odbiorca_uczestnik_id")?,
            rachunek_id: row.get("rachunek_id")?,
            sprawdzenie_id: row.get("sprawdzenie_id")?,
            decyzja_id: row.get("decyzja_id")?,
            utworzono: row.get("utworzono")?,
            stan: row.get("stan")?,
            wykonal: row.get("wykonal")?,
            wykonano: row.get("wykonano")?,
            potwierdzenie_dokument_id: row.get("potwierdzenie_dokument_id")?,
        })
    }
}

/// Wypłaty, które czekają na sprawdzenie, zgodę albo wykonanie.
pub fn wyplaty_przygotowane(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WyplatyPrzygotowaneRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_wyplaty_wpisy
WHERE stan = 'przygotowana'
ORDER BY utworzono, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(WyplatyPrzygotowaneRow::from_row).collect()
}

/// The parameters of `przygotuj_wyplate`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrzygotujWyplateParams {
    pub id: String,
    pub podstawa_tabela: String,
    pub podstawa_id: String,
    pub kwota: i64,
    pub wyliczenie_sciezka: String,
    pub wyliczenie_skrot: String,
    pub odbiorca_uczestnik_id: String,
    pub rachunek_id: String,
    pub utworzono: String,
}

/// Wypłata z podstawą z typu sprawy, kwotą w groszach, wyliczeniem i dopuszczonym rachunkiem
/// odbiorcy; identyfikator (UUID) nadaje aplikacja, bo wskazują go decyzja i rejestr kontroli.
pub fn przygotuj_wyplate(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzygotujWyplateParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_wyplaty_wpisy (id, podstawa_tabela, podstawa_id, kwota, wyliczenie_sciezka,
  wyliczenie_skrot, odbiorca_uczestnik_id, rachunek_id, utworzono, stan)
VALUES (:id, :podstawa_tabela, :podstawa_id, :kwota, :wyliczenie_sciezka, :wyliczenie_skrot,
  :odbiorca_uczestnik_id, :rachunek_id, :utworzono, 'przygotowana')"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "podstawa_tabela",
                        xmeta_room::ToSqlValue::to_sql_value(&params.podstawa_tabela),
                    ),
                    (
                        "podstawa_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.podstawa_id),
                    ),
                    ("kwota", xmeta_room::ToSqlValue::to_sql_value(&params.kwota)),
                    (
                        "wyliczenie_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wyliczenie_sciezka),
                    ),
                    (
                        "wyliczenie_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wyliczenie_skrot),
                    ),
                    (
                        "odbiorca_uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.odbiorca_uczestnik_id,
                        ),
                    ),
                    (
                        "rachunek_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rachunek_id),
                    ),
                    (
                        "utworzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.utworzono),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `wskaz_sprawdzenie_wyplaty`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WskazSprawdzenieWyplatyParams {
    pub sprawdzenie_id: String,
    pub id: String,
}

/// Sprawdzenie rachunku, które Kontroler zrobił po przygotowaniu wypłaty.
pub fn wskaz_sprawdzenie_wyplaty(
    room: &impl xmeta_room::MetaRoom,
    params: &WskazSprawdzenieWyplatyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_wyplaty_wpisy
SET sprawdzenie_id = :sprawdzenie_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "sprawdzenie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sprawdzenie_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wskaz_zgode_na_wyplate`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WskazZgodeNaWyplateParams {
    pub decyzja_id: String,
    pub id: String,
}

/// Zatwierdzona decyzja wyplata, która wskazuje tę wypłatę.
pub fn wskaz_zgode_na_wyplate(
    room: &impl xmeta_room::MetaRoom,
    params: &WskazZgodeNaWyplateParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_wyplaty_wpisy
SET decyzja_id = :decyzja_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_wykonanie_wyplaty`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujWykonanieWyplatyParams {
    pub wykonal: String,
    pub wykonano: String,
    pub potwierdzenie_dokument_id: String,
    pub id: String,
}

/// Człowiek, który zlecił przelew, wpisuje wykonanie z potwierdzeniem z banku.
pub fn odnotuj_wykonanie_wyplaty(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWykonanieWyplatyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_wyplaty_wpisy
SET stan = 'wykonana',
  wykonal = :wykonal,
  wykonano = :wykonano,
  potwierdzenie_dokument_id = :potwierdzenie_dokument_id
WHERE id = :id"#,
                parameters: Vec::from([
                    ("wykonal", xmeta_room::ToSqlValue::to_sql_value(&params.wykonal)),
                    ("wykonano", xmeta_room::ToSqlValue::to_sql_value(&params.wykonano)),
                    (
                        "potwierdzenie_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.potwierdzenie_dokument_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `anuluj_wyplate`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AnulujWyplateParams {
    pub id: String,
}

/// Wypłata, która nie zostanie wykonana, np. po zmianie rozliczenia; nowa to nowy wiersz.
pub fn anuluj_wyplate(
    room: &impl xmeta_room::MetaRoom,
    params: &AnulujWyplateParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_wyplaty_wpisy
SET stan = 'anulowana'
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
