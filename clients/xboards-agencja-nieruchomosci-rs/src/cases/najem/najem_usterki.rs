//! The kind `najem-usterki` of the room `cases/najem`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "najem-usterki";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Usterki i przeglądy lokalu w czasie najmu. Zgłoszenie przychodzi do Opiekuna najmu wprost albo
-- przez Recepcję. Opiekun proponuje, kogo obciąża naprawa, według umowy i przepisów (art. 662,
-- 663 i 681 KC, art. 6a i 6b UOPL); gdy strony się spierają, rozstrzyga człowiek. Naprawę zleca
-- człowiek z biura: do kwoty napraw z bieżących warunków zarządzania bez pytania wynajmującego,
-- a wyższą za jego zgodą i za zgodą człowieka. Usunięcie potwierdza najemca.
--
-- Wejście do lokalu i jego otwarcie należą do człowieka; agent ich nie zleca. Pilnuje tego
-- aplikacja, bo baza nie wie, kto pisze.

-- Zgłoszenia usterek. Decyzje wskazują zgłoszenie, więc wiersz ma UUID.
CREATE TABLE najem_usterki_zgloszenia (
  id TEXT PRIMARY KEY,
  chwila TEXT NOT NULL,
  zglosil_uczestnik_id TEXT REFERENCES sprawa_uczestnicy_wpisy (id),
  wplyw_id TEXT REFERENCES sprawa_korespondencja_wplywy (id),
  opis TEXT NOT NULL CHECK (opis <> ''),
  -- Katalog zdjęć w repozytorium XGit pokoju i skrót jego zawartości.
  zdjecia_sciezka TEXT,
  zdjecia_skrot TEXT,
  -- Usterka uniemożliwia korzystanie z lokalu (art. 663 KC).
  uniemozliwia INTEGER NOT NULL CHECK (uniemozliwia IN (0, 1)),
  -- Kogo obciąża naprawa: propozycja Opiekuna, a przy sporze decyzja człowieka.
  obciaza TEXT CHECK (obciaza IN ('wynajmujacy', 'najemca', 'spor')),
  spor_decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),
  -- Zlecenie naprawy: koszt w groszach, wykonawca i człowiek, który zlecił. Ponad kwotę napraw
  -- także zgoda wynajmującego i decyzja człowieka.
  koszt INTEGER CHECK (koszt >= 0),
  zgoda_wynajmujacego_id TEXT REFERENCES sprawa_zgody_wpisy (id),
  zlecenie_decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),
  zlecil TEXT,
  zlecono TEXT,
  wykonawca TEXT,
  usunieto TEXT,
  -- Wiadomość, w której najemca potwierdził usunięcie.
  potwierdzenie_wplyw_id TEXT REFERENCES sprawa_korespondencja_wplywy (id),

  CONSTRAINT zdjecia_ze_skrotem CHECK ((zdjecia_sciezka IS NULL) = (zdjecia_skrot IS NULL)),
  CONSTRAINT zlecenie_w_komplecie CHECK (
    (zlecil IS NULL) = (zlecono IS NULL)
    AND (zlecil IS NULL) = (wykonawca IS NULL)
    AND (zlecil IS NULL) = (koszt IS NULL)
  ),
  CONSTRAINT zlecenie_obciaza_wynajmujacego CHECK (zlecil IS NULL OR obciaza = 'wynajmujacy'),
  CONSTRAINT usuniecie_z_potwierdzeniem_najemcy CHECK ((usunieto IS NULL) = (potwierdzenie_wplyw_id IS NULL))
) STRICT;

-- Okresowe przeglądy lokalu po uzgodnieniu terminu z najemcą (art. 10 ust. 3 UOPL).
CREATE TABLE najem_usterki_przeglady (
  -- Klucz nadany przez bazę jest dodatni; zob. najem_usterki_przeglady_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  data TEXT NOT NULL,
  wynik TEXT NOT NULL CHECK (wynik <> ''),
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id)
) STRICT;

-- Decyzje, zlecenie i usunięcie przychodzą po zgłoszeniu.
CREATE TRIGGER najem_usterki_zgloszenie_powstaje_bez_zlecenia
BEFORE INSERT ON najem_usterki_zgloszenia
WHEN NEW.spor_decyzja_id IS NOT NULL
  OR NEW.zlecenie_decyzja_id IS NOT NULL
  OR NEW.zgoda_wynajmujacego_id IS NOT NULL
  OR NEW.zlecil IS NOT NULL
  OR NEW.usunieto IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'zgłoszenie usterki powstaje bez decyzji, zlecenia i usunięcia');
END;

-- Spór o to, kogo obciąża naprawa, rozstrzyga człowiek: obciążenie zmienia się ze sporu na stronę
-- tylko z zatwierdzoną decyzją inna, która wskazuje to zgłoszenie.
CREATE TRIGGER najem_usterki_spor_rozstrzyga_decyzja
BEFORE UPDATE OF obciaza ON najem_usterki_zgloszenia
WHEN OLD.obciaza = 'spor'
  AND NEW.obciaza IS NOT 'spor'
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS decyzja
    WHERE decyzja.id = NEW.spor_decyzja_id
      AND decyzja.rodzaj = 'inna'
      AND decyzja.dotyczy_tabela = 'najem_usterki_zgloszenia'
      AND decyzja.dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'spór o naprawę rozstrzyga zatwierdzona decyzja inna, która wskazuje zgłoszenie');
END;

-- Naprawa ponad kwotę napraw z bieżących warunków zarządzania (bez nich każda) wymaga zgody
-- zgoda_na_naprawe wynajmującego o tym zgłoszeniu i zatwierdzonej decyzji inna o nim.
CREATE TRIGGER najem_usterki_naprawa_ponad_kwote_za_zgoda
BEFORE UPDATE OF zlecil ON najem_usterki_zgloszenia
WHEN NEW.zlecil IS NOT NULL
  AND NEW.koszt > coalesce(
    (
      SELECT kwota_napraw
      FROM najem_umowy_zarzadzanie
      ORDER BY id DESC
      LIMIT 1
    ),
    0
  )
  AND (
    NOT EXISTS (
      SELECT 1
      FROM sprawa_zgody_wpisy AS zgoda
      JOIN sprawa_uczestnicy_role AS rola ON rola.uczestnik_id = zgoda.uczestnik_id
      WHERE zgoda.id = NEW.zgoda_wynajmujacego_id
        AND zgoda.rodzaj = 'zgoda_na_naprawe'
        AND zgoda.dotyczy_tabela = 'najem_usterki_zgloszenia'
        AND zgoda.dotyczy_id = NEW.id
        AND rola.rola = 'wynajmujacy'
        AND rola.zakonczono IS NULL
    )
    OR NOT EXISTS (
      SELECT 1
      FROM sprawa_decyzje_zatwierdzone AS decyzja
      WHERE decyzja.id = NEW.zlecenie_decyzja_id
        AND decyzja.rodzaj = 'inna'
        AND decyzja.dotyczy_tabela = 'najem_usterki_zgloszenia'
        AND decyzja.dotyczy_id = NEW.id
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'naprawa ponad kwotę napraw z umowy o zarządzanie wymaga zgody wynajmującego i zatwierdzonej decyzji o tym zgłoszeniu');
END;

-- Usunięcie potwierdza najemca wiadomością od siebie.
CREATE TRIGGER najem_usterki_usuniecie_potwierdza_najemca
BEFORE UPDATE OF potwierdzenie_wplyw_id ON najem_usterki_zgloszenia
WHEN NEW.potwierdzenie_wplyw_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_korespondencja_wplywy AS wplyw
    JOIN sprawa_uczestnicy_role AS rola ON rola.uczestnik_id = wplyw.nadawca_uczestnik_id
    WHERE wplyw.id = NEW.potwierdzenie_wplyw_id
      AND rola.rola = 'najemca'
      AND rola.zakonczono IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'usunięcie usterki potwierdza najemca wiadomością od siebie');
END;

-- Zgłoszenie jest zapisem tego, co przyszło; obciążenie zmienia się do zlecenia, a decyzje,
-- zlecenie i usunięcie wpisuje się raz.
CREATE TRIGGER najem_usterki_zgloszenie_bez_zmian
BEFORE UPDATE ON najem_usterki_zgloszenia
WHEN NEW.id IS NOT OLD.id
  OR NEW.chwila IS NOT OLD.chwila
  OR NEW.zglosil_uczestnik_id IS NOT OLD.zglosil_uczestnik_id
  OR NEW.wplyw_id IS NOT OLD.wplyw_id
  OR NEW.opis IS NOT OLD.opis
  OR NEW.zdjecia_sciezka IS NOT OLD.zdjecia_sciezka
  OR NEW.zdjecia_skrot IS NOT OLD.zdjecia_skrot
  OR NEW.uniemozliwia IS NOT OLD.uniemozliwia
  OR (OLD.spor_decyzja_id IS NOT NULL AND NEW.spor_decyzja_id IS NOT OLD.spor_decyzja_id)
  OR (OLD.zlecil IS NOT NULL AND (
    NEW.zlecil IS NOT OLD.zlecil
    OR NEW.zlecono IS NOT OLD.zlecono
    OR NEW.wykonawca IS NOT OLD.wykonawca
    OR NEW.koszt IS NOT OLD.koszt
    OR NEW.obciaza IS NOT OLD.obciaza
    OR NEW.zgoda_wynajmujacego_id IS NOT OLD.zgoda_wynajmujacego_id
    OR NEW.zlecenie_decyzja_id IS NOT OLD.zlecenie_decyzja_id
  ))
  OR (OLD.usunieto IS NOT NULL AND (
    NEW.usunieto IS NOT OLD.usunieto
    OR NEW.potwierdzenie_wplyw_id IS NOT OLD.potwierdzenie_wplyw_id
  ))
BEGIN
  SELECT RAISE(ABORT, 'zgłoszenia usterki się nie zmienia; decyzję, zlecenie i usunięcie wpisuje się raz');
END;

CREATE TRIGGER najem_usterki_przeglady_bez_zmian
BEFORE UPDATE ON najem_usterki_przeglady
BEGIN
  SELECT RAISE(ABORT, 'przeglądu się nie zmienia; kolejny przegląd to nowy wpis');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER najem_usterki_zgloszenia_bez_zastepowania
BEFORE INSERT ON najem_usterki_zgloszenia
WHEN EXISTS (
  SELECT 1
  FROM najem_usterki_zgloszenia
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER najem_usterki_zgloszenia_bez_usuwania
BEFORE DELETE ON najem_usterki_zgloszenia
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER najem_usterki_przeglady_bez_zastepowania
BEFORE INSERT ON najem_usterki_przeglady
WHEN EXISTS (
  SELECT 1
  FROM najem_usterki_przeglady
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER najem_usterki_przeglady_bez_usuwania
BEFORE DELETE ON najem_usterki_przeglady
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
];

/// The values of column `zgloszenia.obciaza`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZgloszeniaObciaza {
    Wynajmujacy,
    Najemca,
    Spor,
}

impl ZgloszeniaObciaza {
    /// Every value, in the order of the catalog.
    pub const ALL: &[ZgloszeniaObciaza] = &[
        ZgloszeniaObciaza::Wynajmujacy,
        ZgloszeniaObciaza::Najemca,
        ZgloszeniaObciaza::Spor,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            ZgloszeniaObciaza::Wynajmujacy => "wynajmujacy",
            ZgloszeniaObciaza::Najemca => "najemca",
            ZgloszeniaObciaza::Spor => "spor",
        }
    }
}

impl xmeta_room::FromSqlValue for ZgloszeniaObciaza {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "wynajmujacy" => Ok(ZgloszeniaObciaza::Wynajmujacy),
                    "najemca" => Ok(ZgloszeniaObciaza::Najemca),
                    "spor" => Ok(ZgloszeniaObciaza::Spor),
                    other => Err(format!("{other} is no value of ZgloszeniaObciaza")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for ZgloszeniaObciaza {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `przeglady`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzegladyRow {
    pub id: i64,
    pub data: String,
    pub wynik: String,
    pub dokument_id: Option<String>,
}

impl PrzegladyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PrzegladyRow {
            id: row.get("id")?,
            data: row.get("data")?,
            wynik: row.get("wynik")?,
            dokument_id: row.get("dokument_id")?,
        })
    }
}

/// Przeglądy lokalu w kolejności dat.
pub fn przeglady(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PrzegladyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM najem_usterki_przeglady
ORDER BY data, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(PrzegladyRow::from_row).collect()
}

/// The parameters of `wpisz_przeglad`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszPrzegladParams {
    pub data: String,
    pub wynik: String,
    pub dokument_id: Option<String>,
}

/// Okresowy przegląd z wynikiem i protokołem, jeśli jest.
pub fn wpisz_przeglad(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszPrzegladParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO najem_usterki_przeglady (data, wynik, dokument_id)
VALUES (:data, :wynik, :dokument_id)"#,
                parameters: Vec::from([
                    ("data", xmeta_room::ToSqlValue::to_sql_value(&params.data)),
                    ("wynik", xmeta_room::ToSqlValue::to_sql_value(&params.wynik)),
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `usterki`.
#[derive(Debug, Clone, PartialEq)]
pub struct UsterkiRow {
    pub id: String,
    pub chwila: String,
    pub zglosil_uczestnik_id: Option<String>,
    pub wplyw_id: Option<String>,
    pub opis: String,
    pub zdjecia_sciezka: Option<String>,
    pub zdjecia_skrot: Option<String>,
    pub uniemozliwia: bool,
    pub obciaza: Option<ZgloszeniaObciaza>,
    pub spor_decyzja_id: Option<String>,
    pub koszt: Option<i64>,
    pub zgoda_wynajmujacego_id: Option<String>,
    pub zlecenie_decyzja_id: Option<String>,
    pub zlecil: Option<String>,
    pub zlecono: Option<String>,
    pub wykonawca: Option<String>,
    pub usunieto: Option<String>,
    pub potwierdzenie_wplyw_id: Option<String>,
}

impl UsterkiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(UsterkiRow {
            id: row.get("id")?,
            chwila: row.get("chwila")?,
            zglosil_uczestnik_id: row.get("zglosil_uczestnik_id")?,
            wplyw_id: row.get("wplyw_id")?,
            opis: row.get("opis")?,
            zdjecia_sciezka: row.get("zdjecia_sciezka")?,
            zdjecia_skrot: row.get("zdjecia_skrot")?,
            uniemozliwia: row.get("uniemozliwia")?,
            obciaza: row.get("obciaza")?,
            spor_decyzja_id: row.get("spor_decyzja_id")?,
            koszt: row.get("koszt")?,
            zgoda_wynajmujacego_id: row.get("zgoda_wynajmujacego_id")?,
            zlecenie_decyzja_id: row.get("zlecenie_decyzja_id")?,
            zlecil: row.get("zlecil")?,
            zlecono: row.get("zlecono")?,
            wykonawca: row.get("wykonawca")?,
            usunieto: row.get("usunieto")?,
            potwierdzenie_wplyw_id: row.get("potwierdzenie_wplyw_id")?,
        })
    }
}

/// Zgłoszenia usterek, najnowsze pierwsze.
pub fn usterki(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<UsterkiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM najem_usterki_zgloszenia
ORDER BY chwila DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(UsterkiRow::from_row).collect()
}

/// The parameters of `zglos_usterke`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZglosUsterkeParams {
    pub id: String,
    pub chwila: String,
    pub zglosil_uczestnik_id: Option<String>,
    pub wplyw_id: Option<String>,
    pub opis: String,
    pub zdjecia_sciezka: Option<String>,
    pub zdjecia_skrot: Option<String>,
    pub uniemozliwia: bool,
    pub obciaza: Option<ZgloszeniaObciaza>,
}

/// Zgłoszenie od najemcy, wprost albo przez Recepcję, z propozycją, kogo obciąża naprawa;
/// identyfikator (UUID) nadaje aplikacja, bo wskazują go decyzje i zgoda wynajmującego.
pub fn zglos_usterke(
    room: &impl xmeta_room::MetaRoom,
    params: &ZglosUsterkeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO najem_usterki_zgloszenia (id, chwila, zglosil_uczestnik_id, wplyw_id, opis,
  zdjecia_sciezka, zdjecia_skrot, uniemozliwia, obciaza)
VALUES (:id, :chwila, :zglosil_uczestnik_id, :wplyw_id, :opis, :zdjecia_sciezka, :zdjecia_skrot,
  :uniemozliwia, :obciaza)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    (
                        "zglosil_uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.zglosil_uczestnik_id,
                        ),
                    ),
                    ("wplyw_id", xmeta_room::ToSqlValue::to_sql_value(&params.wplyw_id)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    (
                        "zdjecia_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zdjecia_sciezka),
                    ),
                    (
                        "zdjecia_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zdjecia_skrot),
                    ),
                    (
                        "uniemozliwia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.uniemozliwia),
                    ),
                    ("obciaza", xmeta_room::ToSqlValue::to_sql_value(&params.obciaza)),
                ]),
            },
        ],
    )
}

/// The parameters of `ustal_kogo_obciaza_usterka`.
#[derive(Debug, Clone, PartialEq)]
pub struct UstalKogoObciazaUsterkaParams {
    pub obciaza: ZgloszeniaObciaza,
    pub spor_decyzja_id: Option<String>,
    pub id: String,
}

/// Kogo obciąża naprawa; spór rozstrzyga decyzja człowieka.
pub fn ustal_kogo_obciaza_usterka(
    room: &impl xmeta_room::MetaRoom,
    params: &UstalKogoObciazaUsterkaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_usterki_zgloszenia
SET obciaza = :obciaza,
  spor_decyzja_id = :spor_decyzja_id
WHERE id = :id"#,
                parameters: Vec::from([
                    ("obciaza", xmeta_room::ToSqlValue::to_sql_value(&params.obciaza)),
                    (
                        "spor_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.spor_decyzja_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `zlec_naprawe_usterki`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZlecNapraweUsterkiParams {
    pub koszt: i64,
    pub zlecil: String,
    pub zlecono: String,
    pub wykonawca: String,
    pub zgoda_wynajmujacego_id: Option<String>,
    pub zlecenie_decyzja_id: Option<String>,
    pub id: String,
}

/// Człowiek z biura zleca naprawę obciążającą wynajmującego: wykonawca i koszt, a ponad kwotę
/// napraw także zgoda wynajmującego i decyzja.
pub fn zlec_naprawe_usterki(
    room: &impl xmeta_room::MetaRoom,
    params: &ZlecNapraweUsterkiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_usterki_zgloszenia
SET koszt = :koszt,
  zlecil = :zlecil,
  zlecono = :zlecono,
  wykonawca = :wykonawca,
  zgoda_wynajmujacego_id = :zgoda_wynajmujacego_id,
  zlecenie_decyzja_id = :zlecenie_decyzja_id
WHERE id = :id"#,
                parameters: Vec::from([
                    ("koszt", xmeta_room::ToSqlValue::to_sql_value(&params.koszt)),
                    ("zlecil", xmeta_room::ToSqlValue::to_sql_value(&params.zlecil)),
                    ("zlecono", xmeta_room::ToSqlValue::to_sql_value(&params.zlecono)),
                    (
                        "wykonawca",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wykonawca),
                    ),
                    (
                        "zgoda_wynajmujacego_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.zgoda_wynajmujacego_id,
                        ),
                    ),
                    (
                        "zlecenie_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zlecenie_decyzja_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_usuniecie_usterki`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujUsuniecieUsterkiParams {
    pub usunieto: String,
    pub potwierdzenie_wplyw_id: String,
    pub id: String,
}

/// Usterka usunięta, co najemca potwierdził wiadomością.
pub fn odnotuj_usuniecie_usterki(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujUsuniecieUsterkiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_usterki_zgloszenia
SET usunieto = :usunieto,
  potwierdzenie_wplyw_id = :potwierdzenie_wplyw_id
WHERE id = :id"#,
                parameters: Vec::from([
                    ("usunieto", xmeta_room::ToSqlValue::to_sql_value(&params.usunieto)),
                    (
                        "potwierdzenie_wplyw_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.potwierdzenie_wplyw_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
