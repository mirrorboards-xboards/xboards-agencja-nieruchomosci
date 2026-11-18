//! The kind `oferta-najem` of the room `cases/oferta`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "oferta-najem",
    requires: &[&super::sprawa_uczestnicy::KIND, &super::sprawa_zgody::KIND],
    migrations: &[
        r#"-- Warunki, na jakich klient wynajmie nieruchomość, w wersjach. Czynsz leży w oferta-ceny
-- (dotyczy = 'najem'). Pisze Opiekun oferty; każdą wersję zatwierdza klient zgodą warunki_najmu.
-- Kwoty w groszach.
--
-- Rodzaj umowy najmu wybiera klient; Opiekun pokazuje mu różnice ze wzorów biura i nie doradza.
-- Granice kaucji z ustawy (art. 6 ust. 1, art. 19a ust. 4 i art. 19f ust. 5 UOPL) dotyczą umowy
-- najmu, więc pilnuje ich teczka najmu.
CREATE TABLE oferta_najem_warunki (
  id TEXT PRIMARY KEY,
  -- Opłaty, które najemca płaci obok czynszu, i sposób rozliczenia mediów.
  oplaty_obok_czynszu TEXT,
  rozliczenie_mediow TEXT,
  kaucja INTEGER CHECK (kaucja >= 0),
  -- Rodzaje umowy najmu, które klient dopuszcza.
  dopuszcza_zwykly INTEGER NOT NULL CHECK (dopuszcza_zwykly IN (0, 1)),
  dopuszcza_okazjonalny INTEGER NOT NULL CHECK (dopuszcza_okazjonalny IN (0, 1)),
  dopuszcza_instytucjonalny INTEGER NOT NULL CHECK (dopuszcza_instytucjonalny IN (0, 1)),
  -- Od kiedy lokal jest wolny oraz najkrótszy i najdłuższy okres najmu, np. „12 miesięcy”.
  wolny_od TEXT,
  okres_min TEXT,
  okres_max TEXT,
  wyposazenie TEXT,
  liczba_osob INTEGER CHECK (liczba_osob > 0),
  zwierzeta TEXT,
  palenie TEXT,
  -- Czy biuro ma po zawarciu umowy prowadzić najem; wtedy dochodzi umowa o zarządzanie (R6).
  prowadzenie_najmu INTEGER NOT NULL CHECK (prowadzenie_najmu IN (0, 1)),
  zgoda_id TEXT NOT NULL REFERENCES sprawa_zgody_wpisy (id),
  zastepuje_id TEXT REFERENCES oferta_najem_warunki (id),

  CONSTRAINT dopuszcza_rodzaj_najmu CHECK (dopuszcza_zwykly + dopuszcza_okazjonalny + dopuszcza_instytucjonalny >= 1),
  CONSTRAINT nie_zastepuje_samej_siebie CHECK (zastepuje_id IS NOT id)
) STRICT;

-- Warunki najmu zatwierdza klient: zgoda warunki_najmu uczestnika z obowiązującą rolą klient,
-- która dotyczy tej wersji.
CREATE TRIGGER oferta_najem_warunki_ze_zgoda_klienta
BEFORE INSERT ON oferta_najem_warunki
WHEN NOT EXISTS (
  SELECT 1
  FROM sprawa_zgody_wpisy AS zgoda
  JOIN sprawa_uczestnicy_role AS rola ON rola.uczestnik_id = zgoda.uczestnik_id
  WHERE zgoda.id = NEW.zgoda_id
    AND zgoda.rodzaj = 'warunki_najmu'
    AND zgoda.dotyczy_tabela = 'oferta_najem_warunki'
    AND zgoda.dotyczy_id = NEW.id
    AND rola.rola = 'klient'
    AND rola.zakonczono IS NULL
)
BEGIN
  SELECT RAISE(ABORT, 'warunki najmu mają zgodę warunki_najmu klienta, która ich dotyczy');
END;

-- Pierwsza wersja nie zastępuje żadnej; następna zastępuje wersję, która obowiązuje.
CREATE TRIGGER oferta_najem_warunki_zastepuja_obowiazujace
BEFORE INSERT ON oferta_najem_warunki
WHEN (
    NEW.zastepuje_id IS NULL
    AND EXISTS (
      SELECT 1
      FROM oferta_najem_warunki
    )
  )
  OR (
    NEW.zastepuje_id IS NOT NULL
    AND NOT EXISTS (
      SELECT 1
      FROM oferta_najem_warunki AS poprzednie
      WHERE poprzednie.id = NEW.zastepuje_id
        AND NOT EXISTS (
          SELECT 1
          FROM oferta_najem_warunki AS nowsze
          WHERE nowsze.zastepuje_id = poprzednie.id
        )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'nowe warunki najmu zastępują warunki, które obowiązują');
END;

CREATE TRIGGER oferta_najem_warunki_bez_zmian
BEFORE UPDATE ON oferta_najem_warunki
BEGIN
  SELECT RAISE(ABORT, 'warunków najmu się nie zmienia; nowa wersja zastępuje poprzednią');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER oferta_najem_warunki_bez_zastepowania
BEFORE INSERT ON oferta_najem_warunki
WHEN EXISTS (
  SELECT 1
  FROM oferta_najem_warunki
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_najem_warunki_bez_usuwania
BEFORE DELETE ON oferta_najem_warunki
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
    ],
};

/// A row of `warunki_najmu`.
#[derive(Debug, Clone, PartialEq)]
pub struct WarunkiNajmuRow {
    pub id: String,
    pub oplaty_obok_czynszu: Option<String>,
    pub rozliczenie_mediow: Option<String>,
    pub kaucja: Option<i64>,
    pub dopuszcza_zwykly: bool,
    pub dopuszcza_okazjonalny: bool,
    pub dopuszcza_instytucjonalny: bool,
    pub wolny_od: Option<String>,
    pub okres_min: Option<String>,
    pub okres_max: Option<String>,
    pub wyposazenie: Option<String>,
    pub liczba_osob: Option<i64>,
    pub zwierzeta: Option<String>,
    pub palenie: Option<String>,
    pub prowadzenie_najmu: bool,
    pub zgoda_id: String,
    pub zastepuje_id: Option<String>,
}

impl WarunkiNajmuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WarunkiNajmuRow {
            id: row.get("id")?,
            oplaty_obok_czynszu: row.get("oplaty_obok_czynszu")?,
            rozliczenie_mediow: row.get("rozliczenie_mediow")?,
            kaucja: row.get("kaucja")?,
            dopuszcza_zwykly: row.get("dopuszcza_zwykly")?,
            dopuszcza_okazjonalny: row.get("dopuszcza_okazjonalny")?,
            dopuszcza_instytucjonalny: row.get("dopuszcza_instytucjonalny")?,
            wolny_od: row.get("wolny_od")?,
            okres_min: row.get("okres_min")?,
            okres_max: row.get("okres_max")?,
            wyposazenie: row.get("wyposazenie")?,
            liczba_osob: row.get("liczba_osob")?,
            zwierzeta: row.get("zwierzeta")?,
            palenie: row.get("palenie")?,
            prowadzenie_najmu: row.get("prowadzenie_najmu")?,
            zgoda_id: row.get("zgoda_id")?,
            zastepuje_id: row.get("zastepuje_id")?,
        })
    }
}

/// Wersje warunków najmu od najstarszej.
pub fn warunki_najmu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WarunkiNajmuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_najem_warunki
ORDER BY rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(WarunkiNajmuRow::from_row).collect()
}

/// A row of `warunki_najmu_biezace`.
#[derive(Debug, Clone, PartialEq)]
pub struct WarunkiNajmuBiezaceRow {
    pub id: String,
    pub oplaty_obok_czynszu: Option<String>,
    pub rozliczenie_mediow: Option<String>,
    pub kaucja: Option<i64>,
    pub dopuszcza_zwykly: bool,
    pub dopuszcza_okazjonalny: bool,
    pub dopuszcza_instytucjonalny: bool,
    pub wolny_od: Option<String>,
    pub okres_min: Option<String>,
    pub okres_max: Option<String>,
    pub wyposazenie: Option<String>,
    pub liczba_osob: Option<i64>,
    pub zwierzeta: Option<String>,
    pub palenie: Option<String>,
    pub prowadzenie_najmu: bool,
    pub zgoda_id: String,
    pub zastepuje_id: Option<String>,
}

impl WarunkiNajmuBiezaceRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WarunkiNajmuBiezaceRow {
            id: row.get("id")?,
            oplaty_obok_czynszu: row.get("oplaty_obok_czynszu")?,
            rozliczenie_mediow: row.get("rozliczenie_mediow")?,
            kaucja: row.get("kaucja")?,
            dopuszcza_zwykly: row.get("dopuszcza_zwykly")?,
            dopuszcza_okazjonalny: row.get("dopuszcza_okazjonalny")?,
            dopuszcza_instytucjonalny: row.get("dopuszcza_instytucjonalny")?,
            wolny_od: row.get("wolny_od")?,
            okres_min: row.get("okres_min")?,
            okres_max: row.get("okres_max")?,
            wyposazenie: row.get("wyposazenie")?,
            liczba_osob: row.get("liczba_osob")?,
            zwierzeta: row.get("zwierzeta")?,
            palenie: row.get("palenie")?,
            prowadzenie_najmu: row.get("prowadzenie_najmu")?,
            zgoda_id: row.get("zgoda_id")?,
            zastepuje_id: row.get("zastepuje_id")?,
        })
    }
}

/// Warunki najmu, które obowiązują: wersja, której nie zastąpiła nowsza.
pub fn warunki_najmu_biezace(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Option<WarunkiNajmuBiezaceRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_najem_warunki AS warunki
WHERE NOT EXISTS (
  SELECT 1
  FROM oferta_najem_warunki AS nowsze
  WHERE nowsze.zastepuje_id = warunki.id
)"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.first().map(WarunkiNajmuBiezaceRow::from_row).transpose()
}

/// The parameters of `wpisz_warunki_najmu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszWarunkiNajmuParams {
    pub id: String,
    pub oplaty_obok_czynszu: Option<String>,
    pub rozliczenie_mediow: Option<String>,
    pub kaucja: Option<i64>,
    pub dopuszcza_zwykly: bool,
    pub dopuszcza_okazjonalny: bool,
    pub dopuszcza_instytucjonalny: bool,
    pub wolny_od: Option<String>,
    pub okres_min: Option<String>,
    pub okres_max: Option<String>,
    pub wyposazenie: Option<String>,
    pub liczba_osob: Option<i64>,
    pub zwierzeta: Option<String>,
    pub palenie: Option<String>,
    pub prowadzenie_najmu: bool,
    pub zgoda_id: String,
    pub zastepuje_id: Option<String>,
}

/// Warunki najmu zatwierdzone przez klienta; nowa wersja zastępuje obowiązującą. Identyfikator
/// (UUID) nadaje aplikacja, bo wskazuje go zgoda klienta.
pub fn wpisz_warunki_najmu(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszWarunkiNajmuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO oferta_najem_warunki (id, oplaty_obok_czynszu, rozliczenie_mediow, kaucja,
  dopuszcza_zwykly, dopuszcza_okazjonalny, dopuszcza_instytucjonalny, wolny_od, okres_min,
  okres_max, wyposazenie, liczba_osob, zwierzeta, palenie, prowadzenie_najmu, zgoda_id, zastepuje_id)
VALUES (:id, :oplaty_obok_czynszu, :rozliczenie_mediow, :kaucja, :dopuszcza_zwykly,
  :dopuszcza_okazjonalny, :dopuszcza_instytucjonalny, :wolny_od, :okres_min, :okres_max,
  :wyposazenie, :liczba_osob, :zwierzeta, :palenie, :prowadzenie_najmu, :zgoda_id, :zastepuje_id)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "oplaty_obok_czynszu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.oplaty_obok_czynszu),
                    ),
                    (
                        "rozliczenie_mediow",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rozliczenie_mediow),
                    ),
                    ("kaucja", xmeta_room::ToSqlValue::to_sql_value(&params.kaucja)),
                    (
                        "dopuszcza_zwykly",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dopuszcza_zwykly),
                    ),
                    (
                        "dopuszcza_okazjonalny",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.dopuszcza_okazjonalny,
                        ),
                    ),
                    (
                        "dopuszcza_instytucjonalny",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.dopuszcza_instytucjonalny,
                        ),
                    ),
                    ("wolny_od", xmeta_room::ToSqlValue::to_sql_value(&params.wolny_od)),
                    (
                        "okres_min",
                        xmeta_room::ToSqlValue::to_sql_value(&params.okres_min),
                    ),
                    (
                        "okres_max",
                        xmeta_room::ToSqlValue::to_sql_value(&params.okres_max),
                    ),
                    (
                        "wyposazenie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wyposazenie),
                    ),
                    (
                        "liczba_osob",
                        xmeta_room::ToSqlValue::to_sql_value(&params.liczba_osob),
                    ),
                    (
                        "zwierzeta",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zwierzeta),
                    ),
                    ("palenie", xmeta_room::ToSqlValue::to_sql_value(&params.palenie)),
                    (
                        "prowadzenie_najmu",
                        xmeta_room::ToSqlValue::to_sql_value(&params.prowadzenie_najmu),
                    ),
                    ("zgoda_id", xmeta_room::ToSqlValue::to_sql_value(&params.zgoda_id)),
                    (
                        "zastepuje_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zastepuje_id),
                    ),
                ]),
            },
        ],
    )
}
