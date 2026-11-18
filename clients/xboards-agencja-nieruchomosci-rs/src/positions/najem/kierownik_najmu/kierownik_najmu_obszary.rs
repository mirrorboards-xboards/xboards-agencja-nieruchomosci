//! The kind `kierownik-najmu-obszary` of the room `positions/najem/kierownik_najmu`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "kierownik-najmu-obszary",
    requires: &[&super::kierownik_dzialu_agenci::KIND],
    migrations: &[
        r#"-- Obszary lokali krótkoterminowych: granice każdego obszaru, zatwierdzone przez człowieka, jego
-- Koordynator operacji i lokale, które do niego należą.
--
-- Granice proponuje kierownik, a zatwierdza człowiek podpisem w wiadomości; kierownik przenosi
-- podpis do wiersza, a sprawdza go aplikacja. Zatwierdzonych granic się nie zmienia: inne granice
-- to nowy obszar, a stary się znosi. Koordynatora operacji kierownik powołuje w kierownik-dzialu-agenci,
-- a obszar wskazuje jego wiersz, bo Koordynator nie ma teczki.
CREATE TABLE kierownik_najmu_obszary_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_najmu_obszary_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  nazwa TEXT NOT NULL CHECK (nazwa <> ''),
  -- Np. „Wrocław, Stare Miasto w granicach ul. Kazimierza Wielkiego i fosy”.
  granice TEXT NOT NULL CHECK (granice <> ''),
  zaproponowano TEXT NOT NULL,
  zatwierdzil TEXT,
  podpis TEXT,
  zatwierdzono TEXT,
  koordynator_id TEXT REFERENCES kierownik_dzialu_agenci_wpisy (id),
  zniesiono TEXT,

  CONSTRAINT zatwierdzenie_w_komplecie CHECK (
    (zatwierdzil IS NULL) = (podpis IS NULL)
    AND (zatwierdzil IS NULL) = (zatwierdzono IS NULL)
  ),
  CONSTRAINT zatwierdzenie_nie_przed_propozycja CHECK (zatwierdzono IS NULL OR zatwierdzono >= zaproponowano),
  CONSTRAINT koordynator_obszaru_zatwierdzonego CHECK (koordynator_id IS NULL OR zatwierdzil IS NOT NULL),
  CONSTRAINT zniesienie_nie_przed_propozycja CHECK (zniesiono IS NULL OR zniesiono >= zaproponowano)
) STRICT;

-- Lokal (jego teczka) należy do obszaru od dnia do dnia; naraz do jednego obszaru.
CREATE TABLE kierownik_najmu_obszary_lokale (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_najmu_obszary_lokale_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  obszar_id INTEGER NOT NULL REFERENCES kierownik_najmu_obszary_wpisy (id),
  -- Pokój teczki lokalu krótkoterminowego, tekstem.
  teczka_odnosnik TEXT NOT NULL CHECK (teczka_odnosnik <> ''),
  od TEXT NOT NULL,
  do_dnia TEXT,

  CONSTRAINT koniec_nie_przed_poczatkiem CHECK (do_dnia IS NULL OR do_dnia >= od)
) STRICT;

CREATE UNIQUE INDEX kierownik_najmu_obszary_lokal_w_jednym_obszarze
ON kierownik_najmu_obszary_lokale (teczka_odnosnik)
WHERE do_dnia IS NULL;

-- Obszary ----------------------------------------------------------------------------------------

CREATE TRIGGER kierownik_najmu_obszary_powstaje_jako_propozycja
BEFORE INSERT ON kierownik_najmu_obszary_wpisy
WHEN NEW.zatwierdzil IS NOT NULL
  OR NEW.koordynator_id IS NOT NULL
  OR NEW.zniesiono IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'obszar powstaje jako propozycja kierownika; granice zatwierdza człowiek');
END;

-- Koordynatorem obszaru jest agent na stanowisku Koordynatora operacji, który nie został wymieniony.
CREATE TRIGGER kierownik_najmu_obszary_koordynatorem_koordynator_operacji
BEFORE UPDATE OF koordynator_id ON kierownik_najmu_obszary_wpisy
WHEN NEW.koordynator_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM kierownik_dzialu_agenci_wpisy AS agent
    WHERE agent.id = NEW.koordynator_id
      AND agent.stanowisko = 'koordynator_operacji'
      AND agent.stan <> 'wymieniony'
  )
BEGIN
  SELECT RAISE(ABORT, 'koordynatorem obszaru jest niewymieniony agent na stanowisku Koordynatora operacji');
END;

-- Podpis obejmuje granice, więc zatwierdzonego obszaru nie zmienia się poza Koordynatorem i raz
-- odnotowanym zniesieniem; zniesionego nie zmienia się wcale.
CREATE TRIGGER kierownik_najmu_obszary_bez_zmian
BEFORE UPDATE ON kierownik_najmu_obszary_wpisy
WHEN OLD.zniesiono IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.zaproponowano IS NOT OLD.zaproponowano
  OR (OLD.zatwierdzil IS NOT NULL AND (
    NEW.nazwa IS NOT OLD.nazwa
    OR NEW.granice IS NOT OLD.granice
    OR NEW.zatwierdzil IS NOT OLD.zatwierdzil
    OR NEW.podpis IS NOT OLD.podpis
    OR NEW.zatwierdzono IS NOT OLD.zatwierdzono
  ))
BEGIN
  SELECT RAISE(ABORT, 'zatwierdzonych granic obszaru się nie zmienia, a zniesionego obszaru wcale; inne granice to nowy obszar');
END;

-- Obszar znosi się dopiero, gdy nie ma już w nim lokali.
CREATE TRIGGER kierownik_najmu_obszary_zniesiony_bez_lokali
BEFORE UPDATE OF zniesiono ON kierownik_najmu_obszary_wpisy
WHEN NEW.zniesiono IS NOT NULL
  AND EXISTS (
    SELECT 1
    FROM kierownik_najmu_obszary_lokale AS lokal
    WHERE lokal.obszar_id = NEW.id
      AND lokal.do_dnia IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'obszar znosi się dopiero, gdy nie ma w nim lokali');
END;

-- Lokale -----------------------------------------------------------------------------------------

CREATE TRIGGER kierownik_najmu_obszary_lokal_powstaje_w_obszarze
BEFORE INSERT ON kierownik_najmu_obszary_lokale
WHEN NEW.do_dnia IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'lokal powstaje w obszarze; koniec przynależności odnotowuje się później');
END;

CREATE TRIGGER kierownik_najmu_obszary_lokal_do_zatwierdzonego_obszaru
BEFORE INSERT ON kierownik_najmu_obszary_lokale
WHEN NOT EXISTS (
  SELECT 1
  FROM kierownik_najmu_obszary_wpisy AS obszar
  WHERE obszar.id = NEW.obszar_id
    AND obszar.zatwierdzil IS NOT NULL
    AND obszar.zniesiono IS NULL
)
BEGIN
  SELECT RAISE(ABORT, 'lokal należy tylko do obszaru zatwierdzonego i niezniesionego');
END;

-- Drugie przypisanie lokalu z innym kluczem zderzyłoby się z indeksem, a INSERT OR REPLACE usunąłby
-- wtedy pierwsze z ominięciem blokady usuwania; ten trigger odmawia wcześniej.
CREATE TRIGGER kierownik_najmu_obszary_lokal_w_jednym_obszarze_naraz
BEFORE INSERT ON kierownik_najmu_obszary_lokale
WHEN EXISTS (
  SELECT 1
  FROM kierownik_najmu_obszary_lokale
  WHERE teczka_odnosnik = NEW.teczka_odnosnik
    AND do_dnia IS NULL
    AND id IS NOT NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'lokal należy naraz do jednego obszaru; nowy obszar przychodzi po końcu starego');
END;

CREATE TRIGGER kierownik_najmu_obszary_lokale_bez_zmian
BEFORE UPDATE ON kierownik_najmu_obszary_lokale
WHEN OLD.do_dnia IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.obszar_id IS NOT OLD.obszar_id
  OR NEW.teczka_odnosnik IS NOT OLD.teczka_odnosnik
  OR NEW.od IS NOT OLD.od
BEGIN
  SELECT RAISE(ABORT, 'przynależności lokalu do obszaru się nie zmienia; wolno raz odnotować jej koniec');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_najmu_obszary_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_najmu_obszary_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_najmu_obszary_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER kierownik_najmu_obszary_lokale_bez_zastepowania
BEFORE INSERT ON kierownik_najmu_obszary_lokale
WHEN EXISTS (
  SELECT 1
  FROM kierownik_najmu_obszary_lokale
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: zniesiony obszar i dawna przynależność lokalu zostają jako historia.
CREATE TRIGGER kierownik_najmu_obszary_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_najmu_obszary_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER kierownik_najmu_obszary_lokale_bez_usuwania
BEFORE DELETE ON kierownik_najmu_obszary_lokale
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
    ],
};

/// The parameters of `lokale_obszaru`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LokaleObszaruParams {
    pub obszar_id: i64,
}

/// A row of `lokale_obszaru`.
#[derive(Debug, Clone, PartialEq)]
pub struct LokaleObszaruRow {
    pub id: i64,
    pub obszar_id: i64,
    pub teczka_odnosnik: String,
    pub od: String,
    pub do_dnia: Option<String>,
}

impl LokaleObszaruRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(LokaleObszaruRow {
            id: row.get("id")?,
            obszar_id: row.get("obszar_id")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            od: row.get("od")?,
            do_dnia: row.get("do_dnia")?,
        })
    }
}

/// Lokale, które teraz należą do jednego obszaru.
pub fn lokale_obszaru(
    room: &impl xmeta_room::MetaRoom,
    params: &LokaleObszaruParams,
) -> Result<Vec<LokaleObszaruRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_najmu_obszary_lokale
WHERE obszar_id = :obszar_id
  AND do_dnia IS NULL
ORDER BY od, id"#,
        parameters: Vec::from([
            ("obszar_id", xmeta_room::ToSqlValue::to_sql_value(&params.obszar_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(LokaleObszaruRow::from_row).collect()
}

/// The parameters of `obszar_lokalu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ObszarLokaluParams {
    pub teczka_odnosnik: String,
}

/// A row of `obszar_lokalu`.
#[derive(Debug, Clone, PartialEq)]
pub struct ObszarLokaluRow {
    pub id: i64,
    pub nazwa: String,
    pub granice: String,
    pub zaproponowano: String,
    pub zatwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub zatwierdzono: Option<String>,
    pub koordynator_id: Option<String>,
    pub zniesiono: Option<String>,
}

impl ObszarLokaluRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ObszarLokaluRow {
            id: row.get("id")?,
            nazwa: row.get("nazwa")?,
            granice: row.get("granice")?,
            zaproponowano: row.get("zaproponowano")?,
            zatwierdzil: row.get("zatwierdzil")?,
            podpis: row.get("podpis")?,
            zatwierdzono: row.get("zatwierdzono")?,
            koordynator_id: row.get("koordynator_id")?,
            zniesiono: row.get("zniesiono")?,
        })
    }
}

/// Obszar, do którego teraz należy lokal, z jego Koordynatorem operacji.
pub fn obszar_lokalu(
    room: &impl xmeta_room::MetaRoom,
    params: &ObszarLokaluParams,
) -> Result<Vec<ObszarLokaluRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT obszar.*
FROM kierownik_najmu_obszary_lokale AS lokal
JOIN kierownik_najmu_obszary_wpisy AS obszar ON obszar.id = lokal.obszar_id
WHERE lokal.teczka_odnosnik = :teczka_odnosnik
  AND lokal.do_dnia IS NULL"#,
        parameters: Vec::from([
            (
                "teczka_odnosnik",
                xmeta_room::ToSqlValue::to_sql_value(&params.teczka_odnosnik),
            ),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ObszarLokaluRow::from_row).collect()
}

/// A row of `lokale_bez_koordynatora`.
#[derive(Debug, Clone, PartialEq)]
pub struct LokaleBezKoordynatoraRow {
    pub id: i64,
    pub obszar_id: i64,
    pub teczka_odnosnik: String,
    pub od: String,
    pub do_dnia: Option<String>,
}

impl LokaleBezKoordynatoraRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(LokaleBezKoordynatoraRow {
            id: row.get("id")?,
            obszar_id: row.get("obszar_id")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            od: row.get("od")?,
            do_dnia: row.get("do_dnia")?,
        })
    }
}

/// Lokale w obszarach, które nie mają Koordynatora operacji; każdy lokal powinien go mieć.
pub fn lokale_bez_koordynatora(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<LokaleBezKoordynatoraRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT lokal.*
FROM kierownik_najmu_obszary_lokale AS lokal
JOIN kierownik_najmu_obszary_wpisy AS obszar ON obszar.id = lokal.obszar_id
WHERE lokal.do_dnia IS NULL
  AND obszar.koordynator_id IS NULL
ORDER BY lokal.od, lokal.id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(LokaleBezKoordynatoraRow::from_row).collect()
}

/// The parameters of `przypisz_lokal_do_obszaru`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrzypiszLokalDoObszaruParams {
    pub obszar_id: i64,
    pub teczka_odnosnik: String,
    pub od: String,
}

/// Lokal należy od dziś do obszaru; poprzednią przynależność trzeba najpierw zakończyć.
pub fn przypisz_lokal_do_obszaru(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzypiszLokalDoObszaruParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_najmu_obszary_lokale (obszar_id, teczka_odnosnik, od)
VALUES (:obszar_id, :teczka_odnosnik, :od)"#,
                parameters: Vec::from([
                    (
                        "obszar_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.obszar_id),
                    ),
                    (
                        "teczka_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.teczka_odnosnik),
                    ),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                ]),
            },
        ],
    )
}

/// The parameters of `zakoncz_przynaleznosc_lokalu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZakonczPrzynaleznoscLokaluParams {
    pub do_dnia: String,
    pub id: i64,
}

/// Lokal przestaje należeć do obszaru, np. bo przechodzi do innego albo biuro przestało nim zarządzać.
pub fn zakoncz_przynaleznosc_lokalu(
    room: &impl xmeta_room::MetaRoom,
    params: &ZakonczPrzynaleznoscLokaluParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_najmu_obszary_lokale
SET do_dnia = :do_dnia
WHERE id = :id"#,
                parameters: Vec::from([
                    ("do_dnia", xmeta_room::ToSqlValue::to_sql_value(&params.do_dnia)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `obszary`.
#[derive(Debug, Clone, PartialEq)]
pub struct ObszaryRow {
    pub id: i64,
    pub nazwa: String,
    pub granice: String,
    pub zaproponowano: String,
    pub zatwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub zatwierdzono: Option<String>,
    pub koordynator_id: Option<String>,
    pub zniesiono: Option<String>,
}

impl ObszaryRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ObszaryRow {
            id: row.get("id")?,
            nazwa: row.get("nazwa")?,
            granice: row.get("granice")?,
            zaproponowano: row.get("zaproponowano")?,
            zatwierdzil: row.get("zatwierdzil")?,
            podpis: row.get("podpis")?,
            zatwierdzono: row.get("zatwierdzono")?,
            koordynator_id: row.get("koordynator_id")?,
            zniesiono: row.get("zniesiono")?,
        })
    }
}

/// Wszystkie obszary; najpierw niezniesione, według nazwy.
pub fn obszary(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ObszaryRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_najmu_obszary_wpisy
ORDER BY zniesiono IS NOT NULL, nazwa, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ObszaryRow::from_row).collect()
}

/// The parameters of `obszary_koordynatora`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ObszaryKoordynatoraParams {
    pub koordynator_id: String,
}

/// A row of `obszary_koordynatora`.
#[derive(Debug, Clone, PartialEq)]
pub struct ObszaryKoordynatoraRow {
    pub id: i64,
    pub nazwa: String,
    pub granice: String,
    pub zaproponowano: String,
    pub zatwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub zatwierdzono: Option<String>,
    pub koordynator_id: Option<String>,
    pub zniesiono: Option<String>,
}

impl ObszaryKoordynatoraRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ObszaryKoordynatoraRow {
            id: row.get("id")?,
            nazwa: row.get("nazwa")?,
            granice: row.get("granice")?,
            zaproponowano: row.get("zaproponowano")?,
            zatwierdzil: row.get("zatwierdzil")?,
            podpis: row.get("podpis")?,
            zatwierdzono: row.get("zatwierdzono")?,
            koordynator_id: row.get("koordynator_id")?,
            zniesiono: row.get("zniesiono")?,
        })
    }
}

/// Niezniesione obszary jednego Koordynatora operacji.
pub fn obszary_koordynatora(
    room: &impl xmeta_room::MetaRoom,
    params: &ObszaryKoordynatoraParams,
) -> Result<Vec<ObszaryKoordynatoraRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_najmu_obszary_wpisy
WHERE koordynator_id = :koordynator_id
  AND zniesiono IS NULL
ORDER BY nazwa, id"#,
        parameters: Vec::from([
            (
                "koordynator_id",
                xmeta_room::ToSqlValue::to_sql_value(&params.koordynator_id),
            ),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ObszaryKoordynatoraRow::from_row).collect()
}

/// The parameters of `zaproponuj_obszar`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZaproponujObszarParams {
    pub nazwa: String,
    pub granice: String,
    pub zaproponowano: String,
}

/// Nowy obszar z granicami, które kierownik proponuje człowiekowi.
pub fn zaproponuj_obszar(
    room: &impl xmeta_room::MetaRoom,
    params: &ZaproponujObszarParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_najmu_obszary_wpisy (nazwa, granice, zaproponowano)
VALUES (:nazwa, :granice, :zaproponowano)"#,
                parameters: Vec::from([
                    ("nazwa", xmeta_room::ToSqlValue::to_sql_value(&params.nazwa)),
                    ("granice", xmeta_room::ToSqlValue::to_sql_value(&params.granice)),
                    (
                        "zaproponowano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zaproponowano),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_propozycje_obszaru`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZmienPropozycjeObszaruParams {
    pub nazwa: String,
    pub granice: String,
    pub id: i64,
}

/// Kierownik poprawia granice, których człowiek jeszcze nie zatwierdził.
pub fn zmien_propozycje_obszaru(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienPropozycjeObszaruParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_najmu_obszary_wpisy
SET nazwa = :nazwa,
  granice = :granice
WHERE id = :id"#,
                parameters: Vec::from([
                    ("nazwa", xmeta_room::ToSqlValue::to_sql_value(&params.nazwa)),
                    ("granice", xmeta_room::ToSqlValue::to_sql_value(&params.granice)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `zatwierdz_obszar`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZatwierdzObszarParams {
    pub zatwierdzil: String,
    pub podpis: String,
    pub zatwierdzono: String,
    pub id: i64,
}

/// Człowiek zatwierdził granice podpisaną wiadomością.
pub fn zatwierdz_obszar(
    room: &impl xmeta_room::MetaRoom,
    params: &ZatwierdzObszarParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_najmu_obszary_wpisy
SET zatwierdzil = :zatwierdzil,
  podpis = :podpis,
  zatwierdzono = :zatwierdzono
WHERE id = :id"#,
                parameters: Vec::from([
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

/// The parameters of `wskaz_koordynatora_obszaru`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WskazKoordynatoraObszaruParams {
    pub koordynator_id: String,
    pub id: i64,
}

/// Koordynator operacji obszaru, także następca po wymianie poprzedniego.
pub fn wskaz_koordynatora_obszaru(
    room: &impl xmeta_room::MetaRoom,
    params: &WskazKoordynatoraObszaruParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_najmu_obszary_wpisy
SET koordynator_id = :koordynator_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "koordynator_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.koordynator_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `znies_obszar`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZniesObszarParams {
    pub zniesiono: String,
    pub id: i64,
}

/// Obszar przestaje istnieć, np. bo człowiek zatwierdził nowe granice jako nowy obszar.
pub fn znies_obszar(
    room: &impl xmeta_room::MetaRoom,
    params: &ZniesObszarParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_najmu_obszary_wpisy
SET zniesiono = :zniesiono
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zniesiono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zniesiono),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
