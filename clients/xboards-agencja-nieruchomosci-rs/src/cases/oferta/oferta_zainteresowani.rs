//! The kind `oferta-zainteresowani` of the room `cases/oferta`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "oferta-zainteresowani";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Zainteresowani ofertą: osoby, które pytają i oglądają, ich zapytania i prezentacje. Pisze
-- Opiekun oferty.
--
-- Zainteresowany nie jest uczestnikiem rdzenia: rdzeń wymaga nazwiska i danych osoby, a profil
-- tylko imienia i kontaktu (minimalizacja danych, art. 5 ust. 1 lit. c RODO). Gdy zainteresowany
-- staje się stroną, wpisuje go teczka transakcji albo najmu.
--
-- Pełny adres z numerem lokalu dostaje tylko osoba umówiona na prezentację, więc jedyna kolumna
-- z chwilą podania adresu leży przy uczestniku prezentacji. Prezentacja to praca według umowy
-- pośrednictwa, więc odbywa się nie wcześniej niż sprawa_umowy_praca_od.

-- Osoba, która pyta albo ogląda, z kontaktem i informacją o przetwarzaniu jej danych
-- (art. 13 ust. 1 RODO): wysyłką z teczki albo datą, w której dostała ją od Recepcji
-- (art. 13 ust. 4 RODO).
CREATE TABLE oferta_zainteresowani_osoby (
  id TEXT PRIMARY KEY,
  imie TEXT NOT NULL,
  nazwisko TEXT,
  telefon TEXT,
  email TEXT,
  -- Sama, przez inne biuro albo jako klient tego biura prowadzony przez Opiekuna poszukującego.
  dziala_jako TEXT NOT NULL CHECK (dziala_jako IN ('sam', 'przez_biuro', 'klient_biura')),
  biuro TEXT,
  -- Dla kogo działa biuro, które przysłało zapytanie, jeśli to podało.
  dla_kogo TEXT,
  dane_pozyskano TEXT NOT NULL,
  informacja_wysylka_id TEXT REFERENCES sprawa_korespondencja_wysylki (id),
  informacja_od_recepcji TEXT,

  CONSTRAINT osoba_ma_kontakt CHECK (telefon IS NOT NULL OR email IS NOT NULL),
  CONSTRAINT przez_biuro_wskazuje_biuro CHECK (dziala_jako <> 'przez_biuro' OR biuro IS NOT NULL),
  CONSTRAINT informacja_wysylka_albo_od_recepcji CHECK (
    informacja_wysylka_id IS NULL OR informacja_od_recepcji IS NULL
  )
) STRICT;

-- Zapytanie o ofertę: wpływ, odpowiedź i to, czy skończyło się prezentacją.
CREATE TABLE oferta_zainteresowani_zapytania (
  -- Klucz nadany przez bazę jest dodatni; zob. oferta_zainteresowani_zapytania_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  osoba_id TEXT NOT NULL REFERENCES oferta_zainteresowani_osoby (id),
  wplyw_id TEXT NOT NULL REFERENCES sprawa_korespondencja_wplywy (id),
  odpowiedz_wysylka_id TEXT REFERENCES sprawa_korespondencja_wysylki (id),
  skonczylo_sie_prezentacja INTEGER CHECK (skonczylo_sie_prezentacja IN (0, 1))
) STRICT;

-- Prezentacja w kalendarzu pośrednika; dzień otwarty to jedna prezentacja z wieloma
-- uczestnikami.
CREATE TABLE oferta_zainteresowani_prezentacje (
  id TEXT PRIMARY KEY,
  termin TEXT NOT NULL,
  posrednik TEXT NOT NULL,
  kalendarz_odnosnik TEXT,
  -- Kto otwiera lokal: klient, pośrednik z kluczami biura albo najemca, który w nim mieszka.
  otwiera TEXT NOT NULL CHECK (otwiera IN ('klient', 'posrednik', 'najemca')),
  -- Notatka po prezentacji: kto przyszedł, co oglądał, o co pytał i jak ocenił nieruchomość.
  przebieg TEXT,
  odwolano TEXT,
  odwolal TEXT,
  powod_odwolania TEXT,

  CONSTRAINT odwolanie_kto_kiedy_dlaczego CHECK (
    (odwolano IS NULL) = (odwolal IS NULL)
    AND (odwolano IS NULL) = (powod_odwolania IS NULL)
  ),
  CONSTRAINT odwolana_bez_przebiegu CHECK (odwolano IS NULL OR przebieg IS NULL)
) STRICT;

CREATE TABLE oferta_zainteresowani_prezentacje_uczestnicy (
  -- Klucz nadany przez bazę jest dodatni; zob.
  -- oferta_zainteresowani_prezentacje_uczestnicy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  prezentacja_id TEXT NOT NULL REFERENCES oferta_zainteresowani_prezentacje (id),
  osoba_id TEXT NOT NULL REFERENCES oferta_zainteresowani_osoby (id),
  -- NULL, dopóki prezentacja się nie odbyła; 0 znaczy, że uczestnik nie przyszedł.
  przyszedl INTEGER CHECK (przyszedl IN (0, 1)),
  -- Chwila, w której Opiekun podał uczestnikowi pełny adres.
  adres_podano TEXT,

  UNIQUE (prezentacja_id, osoba_id)
) STRICT;

-- Informacja z teczki to wysłana wysyłka ze znacznikiem informacja_rodo.
CREATE TRIGGER oferta_zainteresowani_informacja_to_wysylka_rodo
BEFORE INSERT ON oferta_zainteresowani_osoby
WHEN NEW.informacja_wysylka_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_korespondencja_wysylki
    WHERE id = NEW.informacja_wysylka_id
      AND informacja_rodo = 1
      AND stan IN ('wyslana', 'doreczona')
  )
BEGIN
  SELECT RAISE(ABORT, 'informacja o danych to wysłana wysyłka ze znacznikiem informacja_rodo');
END;

CREATE TRIGGER oferta_zainteresowani_zmieniona_informacja_to_wysylka_rodo
BEFORE UPDATE OF informacja_wysylka_id ON oferta_zainteresowani_osoby
WHEN NEW.informacja_wysylka_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_korespondencja_wysylki
    WHERE id = NEW.informacja_wysylka_id
      AND informacja_rodo = 1
      AND stan IN ('wyslana', 'doreczona')
  )
BEGIN
  SELECT RAISE(ABORT, 'informacja o danych to wysłana wysyłka ze znacznikiem informacja_rodo');
END;

-- Informację o danych wpisuje się raz; kontakt i opis osoby wolno poprawić.
CREATE TRIGGER oferta_zainteresowani_informacja_raz
BEFORE UPDATE ON oferta_zainteresowani_osoby
WHEN NEW.id IS NOT OLD.id
  OR (OLD.informacja_wysylka_id IS NOT NULL AND NEW.informacja_wysylka_id IS NOT OLD.informacja_wysylka_id)
  OR (OLD.informacja_od_recepcji IS NOT NULL AND NEW.informacja_od_recepcji IS NOT OLD.informacja_od_recepcji)
BEGIN
  SELECT RAISE(ABORT, 'informację o danych zainteresowanego wpisuje się raz');
END;

-- Odpowiedź na zapytanie jest wiadomością do osoby, więc przychodzi po informacji o jej danych
-- albo razem z nią (art. 13 ust. 1 i art. 14 ust. 3 lit. b RODO).
CREATE TRIGGER oferta_zainteresowani_odpowiedz_po_informacji
BEFORE UPDATE OF odpowiedz_wysylka_id ON oferta_zainteresowani_zapytania
WHEN NEW.odpowiedz_wysylka_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM oferta_zainteresowani_osoby
    WHERE id = NEW.osoba_id
      AND (informacja_wysylka_id IS NOT NULL OR informacja_od_recepcji IS NOT NULL)
  )
BEGIN
  SELECT RAISE(ABORT, 'odpowiedź na zapytanie wymaga informacji o danych zainteresowanego');
END;

-- Zapytanie powstaje bez odpowiedzi i wyniku; oba wpisuje się potem raz.
CREATE TRIGGER oferta_zainteresowani_zapytanie_powstaje_bez_odpowiedzi
BEFORE INSERT ON oferta_zainteresowani_zapytania
WHEN NEW.odpowiedz_wysylka_id IS NOT NULL
  OR NEW.skonczylo_sie_prezentacja IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'zapytanie powstaje bez odpowiedzi i wyniku');
END;

CREATE TRIGGER oferta_zainteresowani_zapytania_odpowiedz_i_wynik_raz
BEFORE UPDATE ON oferta_zainteresowani_zapytania
WHEN NEW.id IS NOT OLD.id
  OR NEW.osoba_id IS NOT OLD.osoba_id
  OR NEW.wplyw_id IS NOT OLD.wplyw_id
  OR (OLD.odpowiedz_wysylka_id IS NOT NULL AND NEW.odpowiedz_wysylka_id IS NOT OLD.odpowiedz_wysylka_id)
  OR (OLD.skonczylo_sie_prezentacja IS NOT NULL AND NEW.skonczylo_sie_prezentacja IS NOT OLD.skonczylo_sie_prezentacja)
BEGIN
  SELECT RAISE(ABORT, 'zapytania się nie zmienia; odpowiedź i wynik wpisuje się raz');
END;

-- Prezentacja to praca według zawartej umowy pośrednictwa, więc jej termin nie jest wcześniejszy
-- niż chwila, od której biuro może pracować (art. 15 ust. 3 i art. 21 ust. 2 upk).
CREATE TRIGGER oferta_zainteresowani_prezentacja_od_pracy
BEFORE INSERT ON oferta_zainteresowani_prezentacje
WHEN NOT EXISTS (
  SELECT 1
  FROM sprawa_umowy_wpisy AS umowa
  JOIN sprawa_umowy_praca_od AS praca ON praca.umowa_id = umowa.id
  WHERE umowa.rodzaj = 'posrednictwo'
    AND umowa.stan = 'zawarta'
    AND NEW.termin >= praca.od
)
BEGIN
  SELECT RAISE(ABORT, 'prezentacja wymaga zawartej umowy pośrednictwa i odbywa się nie wcześniej niż praca od');
END;

-- Prezentacja powstaje umówiona; przebieg albo odwołanie wpisuje się potem raz. Inny termin to
-- odwołanie tej prezentacji i nowa prezentacja.
CREATE TRIGGER oferta_zainteresowani_prezentacja_powstaje_umowiona
BEFORE INSERT ON oferta_zainteresowani_prezentacje
WHEN NEW.przebieg IS NOT NULL
  OR NEW.odwolano IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'prezentacja powstaje umówiona, bez przebiegu i odwołania');
END;

CREATE TRIGGER oferta_zainteresowani_prezentacje_bez_zmian
BEFORE UPDATE ON oferta_zainteresowani_prezentacje
WHEN OLD.przebieg IS NOT NULL
  OR OLD.odwolano IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.termin IS NOT OLD.termin
  OR NEW.posrednik IS NOT OLD.posrednik
  OR NEW.kalendarz_odnosnik IS NOT OLD.kalendarz_odnosnik
  OR NEW.otwiera IS NOT OLD.otwiera
BEGIN
  SELECT RAISE(ABORT, 'prezentacji się nie zmienia; raz wpisuje się przebieg albo odwołanie');
END;

-- Obecność i chwilę podania adresu wpisuje się raz.
CREATE TRIGGER oferta_zainteresowani_uczestnicy_raz
BEFORE UPDATE ON oferta_zainteresowani_prezentacje_uczestnicy
WHEN NEW.id IS NOT OLD.id
  OR NEW.prezentacja_id IS NOT OLD.prezentacja_id
  OR NEW.osoba_id IS NOT OLD.osoba_id
  OR (OLD.przyszedl IS NOT NULL AND NEW.przyszedl IS NOT OLD.przyszedl)
  OR (OLD.adres_podano IS NOT NULL AND NEW.adres_podano IS NOT OLD.adres_podano)
BEGIN
  SELECT RAISE(ABORT, 'uczestnika prezentacji się nie zmienia; obecność i podanie adresu wpisuje się raz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER oferta_zainteresowani_osoby_bez_zastepowania
BEFORE INSERT ON oferta_zainteresowani_osoby
WHEN EXISTS (
  SELECT 1
  FROM oferta_zainteresowani_osoby
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_zainteresowani_osoby_bez_usuwania
BEFORE DELETE ON oferta_zainteresowani_osoby
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER oferta_zainteresowani_zapytania_bez_zastepowania
BEFORE INSERT ON oferta_zainteresowani_zapytania
WHEN EXISTS (
  SELECT 1
  FROM oferta_zainteresowani_zapytania
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_zainteresowani_zapytania_bez_usuwania
BEFORE DELETE ON oferta_zainteresowani_zapytania
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER oferta_zainteresowani_prezentacje_bez_zastepowania
BEFORE INSERT ON oferta_zainteresowani_prezentacje
WHEN EXISTS (
  SELECT 1
  FROM oferta_zainteresowani_prezentacje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_zainteresowani_prezentacje_bez_usuwania
BEFORE DELETE ON oferta_zainteresowani_prezentacje
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER oferta_zainteresowani_prezentacje_uczestnicy_bez_zastepowania
BEFORE INSERT ON oferta_zainteresowani_prezentacje_uczestnicy
WHEN EXISTS (
    SELECT 1
    FROM oferta_zainteresowani_prezentacje_uczestnicy
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM oferta_zainteresowani_prezentacje_uczestnicy
    WHERE prezentacja_id = NEW.prezentacja_id
      AND osoba_id = NEW.osoba_id
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_zainteresowani_prezentacje_uczestnicy_bez_usuwania
BEFORE DELETE ON oferta_zainteresowani_prezentacje_uczestnicy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
];

/// The values of column `osoby.dziala_jako`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OsobyDzialaJako {
    Sam,
    PrzezBiuro,
    KlientBiura,
}

impl OsobyDzialaJako {
    /// Every value, in the order of the catalog.
    pub const ALL: &[OsobyDzialaJako] = &[
        OsobyDzialaJako::Sam,
        OsobyDzialaJako::PrzezBiuro,
        OsobyDzialaJako::KlientBiura,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            OsobyDzialaJako::Sam => "sam",
            OsobyDzialaJako::PrzezBiuro => "przez_biuro",
            OsobyDzialaJako::KlientBiura => "klient_biura",
        }
    }
}

impl xmeta_room::FromSqlValue for OsobyDzialaJako {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "sam" => Ok(OsobyDzialaJako::Sam),
                    "przez_biuro" => Ok(OsobyDzialaJako::PrzezBiuro),
                    "klient_biura" => Ok(OsobyDzialaJako::KlientBiura),
                    other => Err(format!("{other} is no value of OsobyDzialaJako")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for OsobyDzialaJako {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `prezentacje.otwiera`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrezentacjeOtwiera {
    Klient,
    Posrednik,
    Najemca,
}

impl PrezentacjeOtwiera {
    /// Every value, in the order of the catalog.
    pub const ALL: &[PrezentacjeOtwiera] = &[
        PrezentacjeOtwiera::Klient,
        PrezentacjeOtwiera::Posrednik,
        PrezentacjeOtwiera::Najemca,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            PrezentacjeOtwiera::Klient => "klient",
            PrezentacjeOtwiera::Posrednik => "posrednik",
            PrezentacjeOtwiera::Najemca => "najemca",
        }
    }
}

impl xmeta_room::FromSqlValue for PrezentacjeOtwiera {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "klient" => Ok(PrezentacjeOtwiera::Klient),
                    "posrednik" => Ok(PrezentacjeOtwiera::Posrednik),
                    "najemca" => Ok(PrezentacjeOtwiera::Najemca),
                    other => Err(format!("{other} is no value of PrezentacjeOtwiera")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for PrezentacjeOtwiera {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `zainteresowani`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZainteresowaniRow {
    pub id: String,
    pub imie: String,
    pub nazwisko: Option<String>,
    pub telefon: Option<String>,
    pub email: Option<String>,
    pub dziala_jako: OsobyDzialaJako,
    pub biuro: Option<String>,
    pub dla_kogo: Option<String>,
    pub dane_pozyskano: String,
    pub informacja_wysylka_id: Option<String>,
    pub informacja_od_recepcji: Option<String>,
}

impl ZainteresowaniRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZainteresowaniRow {
            id: row.get("id")?,
            imie: row.get("imie")?,
            nazwisko: row.get("nazwisko")?,
            telefon: row.get("telefon")?,
            email: row.get("email")?,
            dziala_jako: row.get("dziala_jako")?,
            biuro: row.get("biuro")?,
            dla_kogo: row.get("dla_kogo")?,
            dane_pozyskano: row.get("dane_pozyskano")?,
            informacja_wysylka_id: row.get("informacja_wysylka_id")?,
            informacja_od_recepcji: row.get("informacja_od_recepcji")?,
        })
    }
}

/// Osoby, które pytały o ofertę albo ją oglądały, w kolejności pozyskania danych.
pub fn zainteresowani(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZainteresowaniRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_zainteresowani_osoby
ORDER BY dane_pozyskano, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(ZainteresowaniRow::from_row).collect()
}

/// The parameters of `wpisz_zainteresowanego`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszZainteresowanegoParams {
    pub id: String,
    pub imie: String,
    pub nazwisko: Option<String>,
    pub telefon: Option<String>,
    pub email: Option<String>,
    pub dziala_jako: OsobyDzialaJako,
    pub biuro: Option<String>,
    pub dla_kogo: Option<String>,
    pub dane_pozyskano: String,
    pub informacja_wysylka_id: Option<String>,
    pub informacja_od_recepcji: Option<String>,
}

/// Osoba, która pyta albo ogląda: imię, kontakt i to, czy działa sama, przez inne biuro albo jako
/// klient biura. Identyfikator (UUID) nadaje aplikacja, bo wskazują go zapytania, prezentacje
/// i oferty.
pub fn wpisz_zainteresowanego(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszZainteresowanegoParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO oferta_zainteresowani_osoby (id, imie, nazwisko, telefon, email, dziala_jako, biuro,
  dla_kogo, dane_pozyskano, informacja_wysylka_id, informacja_od_recepcji)
VALUES (:id, :imie, :nazwisko, :telefon, :email, :dziala_jako, :biuro, :dla_kogo, :dane_pozyskano,
  :informacja_wysylka_id, :informacja_od_recepcji)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("imie", xmeta_room::ToSqlValue::to_sql_value(&params.imie)),
                    ("nazwisko", xmeta_room::ToSqlValue::to_sql_value(&params.nazwisko)),
                    ("telefon", xmeta_room::ToSqlValue::to_sql_value(&params.telefon)),
                    ("email", xmeta_room::ToSqlValue::to_sql_value(&params.email)),
                    (
                        "dziala_jako",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dziala_jako),
                    ),
                    ("biuro", xmeta_room::ToSqlValue::to_sql_value(&params.biuro)),
                    ("dla_kogo", xmeta_room::ToSqlValue::to_sql_value(&params.dla_kogo)),
                    (
                        "dane_pozyskano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dane_pozyskano),
                    ),
                    (
                        "informacja_wysylka_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.informacja_wysylka_id,
                        ),
                    ),
                    (
                        "informacja_od_recepcji",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.informacja_od_recepcji,
                        ),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_kontakt_zainteresowanego`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZmienKontaktZainteresowanegoParams {
    pub telefon: Option<String>,
    pub email: Option<String>,
    pub id: String,
}

/// Poprawiony kontakt osoby, np. nowy numer telefonu.
pub fn zmien_kontakt_zainteresowanego(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienKontaktZainteresowanegoParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE oferta_zainteresowani_osoby
SET telefon = :telefon,
  email = :email
WHERE id = :id"#,
                parameters: Vec::from([
                    ("telefon", xmeta_room::ToSqlValue::to_sql_value(&params.telefon)),
                    ("email", xmeta_room::ToSqlValue::to_sql_value(&params.email)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_informacje_dla_zainteresowanego`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujInformacjeDlaZainteresowanegoParams {
    pub informacja_wysylka_id: Option<String>,
    pub informacja_od_recepcji: Option<String>,
    pub id: String,
}

/// Informacja o przetwarzaniu danych: wysłana wysyłka z informacja_rodo albo dzień, w którym osoba
/// dostała ją od Recepcji. Wpisuje się ją raz.
pub fn odnotuj_informacje_dla_zainteresowanego(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujInformacjeDlaZainteresowanegoParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE oferta_zainteresowani_osoby
SET informacja_wysylka_id = :informacja_wysylka_id,
  informacja_od_recepcji = :informacja_od_recepcji
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "informacja_wysylka_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.informacja_wysylka_id,
                        ),
                    ),
                    (
                        "informacja_od_recepcji",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.informacja_od_recepcji,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `prezentacje`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrezentacjeRow {
    pub id: String,
    pub termin: String,
    pub posrednik: String,
    pub kalendarz_odnosnik: Option<String>,
    pub otwiera: PrezentacjeOtwiera,
    pub przebieg: Option<String>,
    pub odwolano: Option<String>,
    pub odwolal: Option<String>,
    pub powod_odwolania: Option<String>,
}

impl PrezentacjeRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PrezentacjeRow {
            id: row.get("id")?,
            termin: row.get("termin")?,
            posrednik: row.get("posrednik")?,
            kalendarz_odnosnik: row.get("kalendarz_odnosnik")?,
            otwiera: row.get("otwiera")?,
            przebieg: row.get("przebieg")?,
            odwolano: row.get("odwolano")?,
            odwolal: row.get("odwolal")?,
            powod_odwolania: row.get("powod_odwolania")?,
        })
    }
}

/// Prezentacje oferty w kolejności terminów.
pub fn prezentacje(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PrezentacjeRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_zainteresowani_prezentacje
ORDER BY termin, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(PrezentacjeRow::from_row).collect()
}

/// A row of `uczestnicy_prezentacji`.
#[derive(Debug, Clone, PartialEq)]
pub struct UczestnicyPrezentacjiRow {
    pub id: i64,
    pub prezentacja_id: String,
    pub osoba_id: String,
    pub przyszedl: Option<bool>,
    pub adres_podano: Option<String>,
}

impl UczestnicyPrezentacjiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(UczestnicyPrezentacjiRow {
            id: row.get("id")?,
            prezentacja_id: row.get("prezentacja_id")?,
            osoba_id: row.get("osoba_id")?,
            przyszedl: row.get("przyszedl")?,
            adres_podano: row.get("adres_podano")?,
        })
    }
}

/// Uczestnicy prezentacji z obecnością i chwilą podania pełnego adresu.
pub fn uczestnicy_prezentacji(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<UczestnicyPrezentacjiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_zainteresowani_prezentacje_uczestnicy
ORDER BY prezentacja_id, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(UczestnicyPrezentacjiRow::from_row).collect()
}

/// The parameters of `umow_prezentacje`.
#[derive(Debug, Clone, PartialEq)]
pub struct UmowPrezentacjeParams {
    pub id: String,
    pub termin: String,
    pub posrednik: String,
    pub kalendarz_odnosnik: Option<String>,
    pub otwiera: PrezentacjeOtwiera,
}

/// Prezentacja w kalendarzu pośrednika, nie wcześniej niż praca według umowy pośrednictwa;
/// identyfikator (UUID) nadaje aplikacja, bo wskazują go uczestnicy.
pub fn umow_prezentacje(
    room: &impl xmeta_room::MetaRoom,
    params: &UmowPrezentacjeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO oferta_zainteresowani_prezentacje (id, termin, posrednik, kalendarz_odnosnik, otwiera)
VALUES (:id, :termin, :posrednik, :kalendarz_odnosnik, :otwiera)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("termin", xmeta_room::ToSqlValue::to_sql_value(&params.termin)),
                    (
                        "posrednik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.posrednik),
                    ),
                    (
                        "kalendarz_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kalendarz_odnosnik),
                    ),
                    ("otwiera", xmeta_room::ToSqlValue::to_sql_value(&params.otwiera)),
                ]),
            },
        ],
    )
}

/// The parameters of `dopisz_uczestnika_prezentacji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DopiszUczestnikaPrezentacjiParams {
    pub prezentacja_id: String,
    pub osoba_id: String,
    pub adres_podano: Option<String>,
}

/// Osoba umówiona na prezentację; chwila podania pełnego adresu, gdy Opiekun już go podał.
pub fn dopisz_uczestnika_prezentacji(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszUczestnikaPrezentacjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO oferta_zainteresowani_prezentacje_uczestnicy (prezentacja_id, osoba_id, adres_podano)
VALUES (:prezentacja_id, :osoba_id, :adres_podano)"#,
                parameters: Vec::from([
                    (
                        "prezentacja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.prezentacja_id),
                    ),
                    ("osoba_id", xmeta_room::ToSqlValue::to_sql_value(&params.osoba_id)),
                    (
                        "adres_podano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.adres_podano),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_podanie_adresu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPodanieAdresuParams {
    pub adres_podano: String,
    pub id: i64,
}

/// Chwila, w której Opiekun podał uczestnikowi pełny adres; wpisuje się ją raz.
pub fn odnotuj_podanie_adresu(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPodanieAdresuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE oferta_zainteresowani_prezentacje_uczestnicy
SET adres_podano = :adres_podano
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "adres_podano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.adres_podano),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_obecnosc_na_prezentacji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujObecnoscNaPrezentacjiParams {
    pub przyszedl: bool,
    pub id: i64,
}

/// Czy uczestnik przyszedł na prezentację.
pub fn odnotuj_obecnosc_na_prezentacji(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujObecnoscNaPrezentacjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE oferta_zainteresowani_prezentacje_uczestnicy
SET przyszedl = :przyszedl
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "przyszedl",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przyszedl),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_przebieg_prezentacji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPrzebiegPrezentacjiParams {
    pub przebieg: String,
    pub id: String,
}

/// Notatka pośrednika po prezentacji, spisana przez Opiekuna.
pub fn odnotuj_przebieg_prezentacji(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPrzebiegPrezentacjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE oferta_zainteresowani_prezentacje
SET przebieg = :przebieg
WHERE id = :id"#,
                parameters: Vec::from([
                    ("przebieg", xmeta_room::ToSqlValue::to_sql_value(&params.przebieg)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odwolaj_prezentacje`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdwolajPrezentacjeParams {
    pub odwolano: String,
    pub odwolal: String,
    pub powod_odwolania: String,
    pub id: String,
}

/// Kto, kiedy i dlaczego odwołał prezentację.
pub fn odwolaj_prezentacje(
    room: &impl xmeta_room::MetaRoom,
    params: &OdwolajPrezentacjeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE oferta_zainteresowani_prezentacje
SET odwolano = :odwolano,
  odwolal = :odwolal,
  powod_odwolania = :powod_odwolania
WHERE id = :id"#,
                parameters: Vec::from([
                    ("odwolano", xmeta_room::ToSqlValue::to_sql_value(&params.odwolano)),
                    ("odwolal", xmeta_room::ToSqlValue::to_sql_value(&params.odwolal)),
                    (
                        "powod_odwolania",
                        xmeta_room::ToSqlValue::to_sql_value(&params.powod_odwolania),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `zapytania`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZapytaniaRow {
    pub id: i64,
    pub osoba_id: String,
    pub wplyw_id: String,
    pub odpowiedz_wysylka_id: Option<String>,
    pub skonczylo_sie_prezentacja: Option<bool>,
}

impl ZapytaniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZapytaniaRow {
            id: row.get("id")?,
            osoba_id: row.get("osoba_id")?,
            wplyw_id: row.get("wplyw_id")?,
            odpowiedz_wysylka_id: row.get("odpowiedz_wysylka_id")?,
            skonczylo_sie_prezentacja: row.get("skonczylo_sie_prezentacja")?,
        })
    }
}

/// Zapytania o ofertę w kolejności wpisania.
pub fn zapytania(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZapytaniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_zainteresowani_zapytania
ORDER BY id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(ZapytaniaRow::from_row).collect()
}

/// The parameters of `wpisz_zapytanie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszZapytanieParams {
    pub osoba_id: String,
    pub wplyw_id: String,
}

/// Zapytanie zainteresowanego z wpływem, w którym przyszło.
pub fn wpisz_zapytanie(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszZapytanieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO oferta_zainteresowani_zapytania (osoba_id, wplyw_id)
VALUES (:osoba_id, :wplyw_id)"#,
                parameters: Vec::from([
                    ("osoba_id", xmeta_room::ToSqlValue::to_sql_value(&params.osoba_id)),
                    ("wplyw_id", xmeta_room::ToSqlValue::to_sql_value(&params.wplyw_id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_odpowiedz_na_zapytanie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujOdpowiedzNaZapytanieParams {
    pub odpowiedz_wysylka_id: String,
    pub id: i64,
}

/// Wysyłka z odpowiedzią na zapytanie; zainteresowany ma już informację o swoich danych.
pub fn odnotuj_odpowiedz_na_zapytanie(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujOdpowiedzNaZapytanieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE oferta_zainteresowani_zapytania
SET odpowiedz_wysylka_id = :odpowiedz_wysylka_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "odpowiedz_wysylka_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.odpowiedz_wysylka_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_wynik_zapytania`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujWynikZapytaniaParams {
    pub skonczylo_sie_prezentacja: bool,
    pub id: i64,
}

/// Czy zapytanie skończyło się prezentacją.
pub fn odnotuj_wynik_zapytania(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWynikZapytaniaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE oferta_zainteresowani_zapytania
SET skonczylo_sie_prezentacja = :skonczylo_sie_prezentacja
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "skonczylo_sie_prezentacja",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.skonczylo_sie_prezentacja,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
