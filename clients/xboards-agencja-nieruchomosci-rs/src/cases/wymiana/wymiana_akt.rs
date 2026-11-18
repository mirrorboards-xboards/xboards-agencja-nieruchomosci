//! The kind `wymiana-akt` of the room `cases/wymiana`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "wymiana-akt";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Akt w pokoju wymiany: terminy aktu ze stanami obu stron, pierwokup i informacja po akcie.
-- Każdy wiersz jest szczegółem jednego wpisu z wymiana_wpisy_wpisy, dopisuje się razem z nim
-- i potem się nie zmienia.
--
-- Termin ma jeden wpis z propozycją, a jego stan zmieniają kolejne wpisy obu stron. Termin jest
-- potwierdzony dopiero wtedy, gdy potwierdziły go obie strony pokoju. Potwierdzenie biura stoi
-- na decyzji potwierdzenie_terminu z teczki transakcji, którą wpis wskazuje tekstem. Pierwokup
-- i to, co dzieje się po akcie, zna kancelaria, więc tylko ona o nich pisze (wymiana-wpisy).

-- Propozycja terminu aktu od którejkolwiek strony: dzień, godzina, miejsce i czynność.
CREATE TABLE wymiana_akt_terminy (
  id TEXT PRIMARY KEY,
  wpis_id TEXT NOT NULL UNIQUE REFERENCES wymiana_wpisy_wpisy (id),
  dzien TEXT NOT NULL,
  -- Godzina w czasie lokalnym kancelarii, np. „10:30”.
  godzina TEXT NOT NULL CHECK (godzina GLOB '[0-2][0-9]:[0-5][0-9]'),
  miejsce TEXT NOT NULL CHECK (miejsce <> ''),
  czynnosc TEXT NOT NULL CHECK (
    czynnosc IN ('przedwstepna_akt', 'sprzedaz_warunkowa', 'przenoszaca', 'oswiadczenie_najemcy')
  )
) STRICT;

-- Stan terminu ogłoszony przez jedną stronę pokoju: potwierdzenie, przesunięcie, odwołanie albo
-- to, że akt się odbył. Przesunięty termin zastępuje nowa propozycja.
CREATE TABLE wymiana_akt_terminy_stany (
  wpis_id TEXT PRIMARY KEY REFERENCES wymiana_wpisy_wpisy (id),
  termin_id TEXT NOT NULL REFERENCES wymiana_akt_terminy (id),
  stan TEXT NOT NULL CHECK (stan IN ('potwierdzony', 'przesuniety', 'odwolany', 'odbyl_sie')),
  -- Decyzja potwierdzenie_terminu z teczki transakcji: identyfikator pokoju i decyzji.
  decyzja_odnosnik TEXT CHECK (decyzja_odnosnik <> '')
) STRICT;

-- Pierwokup: kiedy uprawniony dostał zawiadomienie i czy wykonał prawo (art. 110 ugn,
-- art. 3 ust. 2 ukur). Bez tego Koordynator nie ustali terminu umowy przenoszącej.
CREATE TABLE wymiana_akt_pierwokupy (
  wpis_id TEXT PRIMARY KEY REFERENCES wymiana_wpisy_wpisy (id),
  -- Np. „Gmina Miasto Kraków” albo „Krajowy Ośrodek Wsparcia Rolnictwa”.
  uprawniony TEXT NOT NULL CHECK (uprawniony <> ''),
  zawiadomienie_doreczono TEXT,
  -- Wykonał prawo, oświadczył, że go nie wykona, albo termin upłynął bez oświadczenia; NULL znaczy,
  -- że wyniku jeszcze nie ma.
  wynik TEXT CHECK (wynik IN ('wykonal', 'nie_wykonal', 'termin_uplynal')),
  -- Dzień, w którym zapadł wynik.
  data TEXT,

  CONSTRAINT wynik_z_data CHECK ((wynik IS NULL) = (data IS NULL)),
  CONSTRAINT wynik_po_zawiadomieniu CHECK (wynik IS NULL OR zawiadomienie_doreczono IS NOT NULL)
) STRICT;

-- To, co kancelaria zdecyduje przekazać po akcie, jeśli strony na to pozwoliły.
CREATE TABLE wymiana_akt_po_akcie (
  wpis_id TEXT PRIMARY KEY REFERENCES wymiana_wpisy_wpisy (id),
  akt_sporzadzono TEXT,
  -- Dzień złożenia wniosku o wpis do księgi wieczystej (art. 92 § 4 PoN).
  wniosek_kw_zlozono TEXT,

  CONSTRAINT po_akcie_z_data CHECK (akt_sporzadzono IS NOT NULL OR wniosek_kw_zlozono IS NOT NULL)
) STRICT;

-- Termin ze stanami obu stron. Bieżący stan: odbył się, odwołany albo przesunięty, gdy którakolwiek
-- strona tak ogłosiła; potwierdzony, gdy potwierdziły go obie strony; inaczej proponowany.
CREATE VIEW wymiana_akt_terminy_biezace AS
SELECT
  termin.*,
  ogloszone.potwierdzila_biuro,
  ogloszone.potwierdzila_kancelaria,
  CASE
    WHEN ogloszone.odbyl_sie THEN 'odbyl_sie'
    WHEN ogloszone.odwolany THEN 'odwolany'
    WHEN ogloszone.przesuniety THEN 'przesuniety'
    WHEN ogloszone.potwierdzila_biuro AND ogloszone.potwierdzila_kancelaria THEN 'potwierdzony'
    ELSE 'proponowany'
  END AS stan
FROM wymiana_akt_terminy AS termin
JOIN (
  SELECT
    termin.id AS termin_id,
    count(CASE WHEN stan.stan = 'potwierdzony' AND wpis.strona = 'biuro' THEN 1 END) > 0 AS potwierdzila_biuro,
    count(CASE WHEN stan.stan = 'potwierdzony' AND wpis.strona = 'kancelaria' THEN 1 END) > 0 AS potwierdzila_kancelaria,
    count(CASE WHEN stan.stan = 'przesuniety' THEN 1 END) > 0 AS przesuniety,
    count(CASE WHEN stan.stan = 'odwolany' THEN 1 END) > 0 AS odwolany,
    count(CASE WHEN stan.stan = 'odbyl_sie' THEN 1 END) > 0 AS odbyl_sie
  FROM wymiana_akt_terminy AS termin
  LEFT JOIN wymiana_akt_terminy_stany AS stan ON stan.termin_id = termin.id
  LEFT JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = stan.wpis_id
  GROUP BY termin.id
) AS ogloszone ON ogloszone.termin_id = termin.id;

-- Szczegół dopisuje się w tej samej operacji co jego wpis. Późniejszy dopisek zmieniałby wpis,
-- który druga strona już przeczytała, także w zamkniętym pokoju.
CREATE TRIGGER wymiana_akt_terminy_do_swojego_wpisu
BEFORE INSERT ON wymiana_akt_terminy
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
    AND rodzaj = 'propozycja_terminu'
    AND rowid = (
      SELECT max(rowid)
      FROM wymiana_wpisy_wpisy
    )
)
BEGIN
  SELECT RAISE(ABORT, 'termin należy do najnowszego wpisu propozycja_terminu');
END;

CREATE TRIGGER wymiana_akt_terminy_stany_do_swojego_wpisu
BEFORE INSERT ON wymiana_akt_terminy_stany
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
    AND rodzaj = 'stan_terminu'
    AND rowid = (
      SELECT max(rowid)
      FROM wymiana_wpisy_wpisy
    )
)
BEGIN
  SELECT RAISE(ABORT, 'stan terminu należy do najnowszego wpisu stan_terminu');
END;

CREATE TRIGGER wymiana_akt_pierwokupy_do_swojego_wpisu
BEFORE INSERT ON wymiana_akt_pierwokupy
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
    AND rodzaj = 'pierwokup'
    AND rowid = (
      SELECT max(rowid)
      FROM wymiana_wpisy_wpisy
    )
)
BEGIN
  SELECT RAISE(ABORT, 'pierwokup należy do najnowszego wpisu pierwokup');
END;

CREATE TRIGGER wymiana_akt_po_akcie_do_swojego_wpisu
BEFORE INSERT ON wymiana_akt_po_akcie
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
    AND rodzaj = 'po_akcie'
    AND rowid = (
      SELECT max(rowid)
      FROM wymiana_wpisy_wpisy
    )
)
BEGIN
  SELECT RAISE(ABORT, 'informacja po akcie należy do najnowszego wpisu po_akcie');
END;

-- Potwierdzenie biura stoi na decyzji człowieka z teczki transakcji (profil Koordynatora
-- transakcji). Inne stany biura i wszystkie stany kancelarii decyzji biura nie wskazują.
CREATE TRIGGER wymiana_akt_potwierdzenie_biura_z_decyzja
BEFORE INSERT ON wymiana_akt_terminy_stany
WHEN (
  SELECT strona = 'biuro' AND NEW.stan = 'potwierdzony'
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.wpis_id
) IS NOT (NEW.decyzja_odnosnik IS NOT NULL)
BEGIN
  SELECT RAISE(ABORT, 'potwierdzenie terminu przez biuro wskazuje decyzję potwierdzenie_terminu, a inne stany jej nie mają');
END;

-- Że akt się odbył, wie kancelaria, w której go sporządzono.
CREATE TRIGGER wymiana_akt_odbycie_od_kancelarii
BEFORE INSERT ON wymiana_akt_terminy_stany
WHEN NEW.stan = 'odbyl_sie'
  AND NOT EXISTS (
    SELECT 1
    FROM wymiana_wpisy_wpisy
    WHERE id = NEW.wpis_id
      AND strona = 'kancelaria'
  )
BEGIN
  SELECT RAISE(ABORT, 'że akt się odbył, ogłasza kancelaria');
END;

CREATE TRIGGER wymiana_akt_odbywa_sie_potwierdzony
BEFORE INSERT ON wymiana_akt_terminy_stany
WHEN NEW.stan = 'odbyl_sie'
  AND NOT EXISTS (
    SELECT 1
    FROM wymiana_akt_terminy_biezace
    WHERE id = NEW.termin_id
      AND stan = 'potwierdzony'
  )
BEGIN
  SELECT RAISE(ABORT, 'odbywa się tylko termin potwierdzony przez obie strony');
END;

-- Odwołany, przesunięty albo odbyty termin się nie zmienia; nowy termin to nowa propozycja.
CREATE TRIGGER wymiana_akt_termin_zakonczony
BEFORE INSERT ON wymiana_akt_terminy_stany
WHEN EXISTS (
  SELECT 1
  FROM wymiana_akt_terminy_stany
  WHERE termin_id = NEW.termin_id
    AND stan IN ('przesuniety', 'odwolany', 'odbyl_sie')
)
BEGIN
  SELECT RAISE(ABORT, 'odwołany, przesunięty albo odbyty termin nie ma kolejnych stanów; nowy termin to nowa propozycja');
END;

CREATE TRIGGER wymiana_akt_terminy_bez_zmian
BEFORE UPDATE ON wymiana_akt_terminy
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

CREATE TRIGGER wymiana_akt_terminy_stany_bez_zmian
BEFORE UPDATE ON wymiana_akt_terminy_stany
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

CREATE TRIGGER wymiana_akt_pierwokupy_bez_zmian
BEFORE UPDATE ON wymiana_akt_pierwokupy
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

CREATE TRIGGER wymiana_akt_po_akcie_bez_zmian
BEFORE UPDATE ON wymiana_akt_po_akcie
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_akt_terminy_bez_zastepowania
BEFORE INSERT ON wymiana_akt_terminy
WHEN EXISTS (
    SELECT 1
    FROM wymiana_akt_terminy
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM wymiana_akt_terminy
    WHERE wpis_id = NEW.wpis_id
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_akt_terminy_bez_usuwania
BEFORE DELETE ON wymiana_akt_terminy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_akt_terminy_stany_bez_zastepowania
BEFORE INSERT ON wymiana_akt_terminy_stany
WHEN EXISTS (
  SELECT 1
  FROM wymiana_akt_terminy_stany
  WHERE wpis_id = NEW.wpis_id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_akt_terminy_stany_bez_usuwania
BEFORE DELETE ON wymiana_akt_terminy_stany
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_akt_pierwokupy_bez_zastepowania
BEFORE INSERT ON wymiana_akt_pierwokupy
WHEN EXISTS (
  SELECT 1
  FROM wymiana_akt_pierwokupy
  WHERE wpis_id = NEW.wpis_id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_akt_pierwokupy_bez_usuwania
BEFORE DELETE ON wymiana_akt_pierwokupy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_akt_po_akcie_bez_zastepowania
BEFORE INSERT ON wymiana_akt_po_akcie
WHEN EXISTS (
  SELECT 1
  FROM wymiana_akt_po_akcie
  WHERE wpis_id = NEW.wpis_id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_akt_po_akcie_bez_usuwania
BEFORE DELETE ON wymiana_akt_po_akcie
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;
"#,
];

/// The values of column `pierwokupy.wynik`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PierwokupyWynik {
    Wykonal,
    NieWykonal,
    TerminUplynal,
}

impl PierwokupyWynik {
    /// Every value, in the order of the catalog.
    pub const ALL: &[PierwokupyWynik] = &[
        PierwokupyWynik::Wykonal,
        PierwokupyWynik::NieWykonal,
        PierwokupyWynik::TerminUplynal,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            PierwokupyWynik::Wykonal => "wykonal",
            PierwokupyWynik::NieWykonal => "nie_wykonal",
            PierwokupyWynik::TerminUplynal => "termin_uplynal",
        }
    }
}

impl xmeta_room::FromSqlValue for PierwokupyWynik {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "wykonal" => Ok(PierwokupyWynik::Wykonal),
                    "nie_wykonal" => Ok(PierwokupyWynik::NieWykonal),
                    "termin_uplynal" => Ok(PierwokupyWynik::TerminUplynal),
                    other => Err(format!("{other} is no value of PierwokupyWynik")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for PierwokupyWynik {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `terminy.czynnosc`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerminyCzynnosc {
    PrzedwstepnaAkt,
    SprzedazWarunkowa,
    Przenoszaca,
    OswiadczenieNajemcy,
}

impl TerminyCzynnosc {
    /// Every value, in the order of the catalog.
    pub const ALL: &[TerminyCzynnosc] = &[
        TerminyCzynnosc::PrzedwstepnaAkt,
        TerminyCzynnosc::SprzedazWarunkowa,
        TerminyCzynnosc::Przenoszaca,
        TerminyCzynnosc::OswiadczenieNajemcy,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            TerminyCzynnosc::PrzedwstepnaAkt => "przedwstepna_akt",
            TerminyCzynnosc::SprzedazWarunkowa => "sprzedaz_warunkowa",
            TerminyCzynnosc::Przenoszaca => "przenoszaca",
            TerminyCzynnosc::OswiadczenieNajemcy => "oswiadczenie_najemcy",
        }
    }
}

impl xmeta_room::FromSqlValue for TerminyCzynnosc {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "przedwstepna_akt" => Ok(TerminyCzynnosc::PrzedwstepnaAkt),
                    "sprzedaz_warunkowa" => Ok(TerminyCzynnosc::SprzedazWarunkowa),
                    "przenoszaca" => Ok(TerminyCzynnosc::Przenoszaca),
                    "oswiadczenie_najemcy" => Ok(TerminyCzynnosc::OswiadczenieNajemcy),
                    other => Err(format!("{other} is no value of TerminyCzynnosc")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for TerminyCzynnosc {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `terminy_stany.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerminyStanyStan {
    Potwierdzony,
    Przesuniety,
    Odwolany,
    OdbylSie,
}

impl TerminyStanyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[TerminyStanyStan] = &[
        TerminyStanyStan::Potwierdzony,
        TerminyStanyStan::Przesuniety,
        TerminyStanyStan::Odwolany,
        TerminyStanyStan::OdbylSie,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            TerminyStanyStan::Potwierdzony => "potwierdzony",
            TerminyStanyStan::Przesuniety => "przesuniety",
            TerminyStanyStan::Odwolany => "odwolany",
            TerminyStanyStan::OdbylSie => "odbyl_sie",
        }
    }
}

impl xmeta_room::FromSqlValue for TerminyStanyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "potwierdzony" => Ok(TerminyStanyStan::Potwierdzony),
                    "przesuniety" => Ok(TerminyStanyStan::Przesuniety),
                    "odwolany" => Ok(TerminyStanyStan::Odwolany),
                    "odbyl_sie" => Ok(TerminyStanyStan::OdbylSie),
                    other => Err(format!("{other} is no value of TerminyStanyStan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for TerminyStanyStan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of field `strona` of `podaj_pierwokup`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Strona {
    Biuro,
    Kancelaria,
}

impl Strona {
    /// Every value, in the order of the catalog.
    pub const ALL: &[Strona] = &[Strona::Biuro, Strona::Kancelaria];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            Strona::Biuro => "biuro",
            Strona::Kancelaria => "kancelaria",
        }
    }
}

impl xmeta_room::FromSqlValue for Strona {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "biuro" => Ok(Strona::Biuro),
                    "kancelaria" => Ok(Strona::Kancelaria),
                    other => Err(format!("{other} is no value of Strona")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for Strona {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of field `stan` of `terminy_aktu`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stan {
    Proponowany,
    Potwierdzony,
    Przesuniety,
    Odwolany,
    OdbylSie,
}

impl Stan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[Stan] = &[
        Stan::Proponowany,
        Stan::Potwierdzony,
        Stan::Przesuniety,
        Stan::Odwolany,
        Stan::OdbylSie,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            Stan::Proponowany => "proponowany",
            Stan::Potwierdzony => "potwierdzony",
            Stan::Przesuniety => "przesuniety",
            Stan::Odwolany => "odwolany",
            Stan::OdbylSie => "odbyl_sie",
        }
    }
}

impl xmeta_room::FromSqlValue for Stan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "proponowany" => Ok(Stan::Proponowany),
                    "potwierdzony" => Ok(Stan::Potwierdzony),
                    "przesuniety" => Ok(Stan::Przesuniety),
                    "odwolany" => Ok(Stan::Odwolany),
                    "odbyl_sie" => Ok(Stan::OdbylSie),
                    other => Err(format!("{other} is no value of Stan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for Stan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `pierwokupy`.
#[derive(Debug, Clone, PartialEq)]
pub struct PierwokupyRow {
    pub wpis_id: String,
    pub uprawniony: String,
    pub zawiadomienie_doreczono: Option<String>,
    pub wynik: Option<PierwokupyWynik>,
    pub data: Option<String>,
    pub chwila: String,
    pub poprawia_id: Option<String>,
    pub tresc: Option<String>,
}

impl PierwokupyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PierwokupyRow {
            wpis_id: row.get("wpis_id")?,
            uprawniony: row.get("uprawniony")?,
            zawiadomienie_doreczono: row.get("zawiadomienie_doreczono")?,
            wynik: row.get("wynik")?,
            data: row.get("data")?,
            chwila: row.get("chwila")?,
            poprawia_id: row.get("poprawia_id")?,
            tresc: row.get("tresc")?,
        })
    }
}

/// Wiadomości kancelarii o pierwokupie: doręczenie zawiadomienia i wynik, od najstarszej.
pub fn pierwokupy(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PierwokupyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT pierwokup.*, wpis.chwila, wpis.poprawia_id, wpis.tresc
FROM wymiana_akt_pierwokupy AS pierwokup
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = pierwokup.wpis_id
ORDER BY wpis.rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(PierwokupyRow::from_row).collect()
}

/// The parameters of `podaj_pierwokup`.
#[derive(Debug, Clone, PartialEq)]
pub struct PodajPierwokupParams {
    pub wpis_id: String,
    pub strona: Strona,
    pub autor: String,
    pub chwila: String,
    pub poprawia_id: Option<String>,
    pub tresc: Option<String>,
    pub uprawniony: String,
    pub zawiadomienie_doreczono: Option<String>,
    pub wynik: Option<PierwokupyWynik>,
    pub data: Option<String>,
}

/// Kancelaria podaje, kiedy uprawniony dostał zawiadomienie, a potem wynik; wynik poprawia
/// wcześniejszy wpis.
pub fn podaj_pierwokup(
    room: &impl xmeta_room::MetaRoom,
    params: &PodajPierwokupParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, poprawia_id, tresc)
VALUES (:wpis_id, :strona, :autor, 'pierwokup', :chwila, :poprawia_id, :tresc)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    ("strona", xmeta_room::ToSqlValue::to_sql_value(&params.strona)),
                    ("autor", xmeta_room::ToSqlValue::to_sql_value(&params.autor)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    (
                        "poprawia_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprawia_id),
                    ),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                ]),
            },
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_akt_pierwokupy (wpis_id, uprawniony, zawiadomienie_doreczono, wynik, data)
VALUES (:wpis_id, :uprawniony, :zawiadomienie_doreczono, :wynik, :data)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    (
                        "uprawniony",
                        xmeta_room::ToSqlValue::to_sql_value(&params.uprawniony),
                    ),
                    (
                        "zawiadomienie_doreczono",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.zawiadomienie_doreczono,
                        ),
                    ),
                    ("wynik", xmeta_room::ToSqlValue::to_sql_value(&params.wynik)),
                    ("data", xmeta_room::ToSqlValue::to_sql_value(&params.data)),
                ]),
            },
        ],
    )
}

/// A row of `informacje_po_akcie`.
#[derive(Debug, Clone, PartialEq)]
pub struct InformacjePoAkcieRow {
    pub wpis_id: String,
    pub akt_sporzadzono: Option<String>,
    pub wniosek_kw_zlozono: Option<String>,
    pub chwila: String,
    pub poprawia_id: Option<String>,
    pub tresc: Option<String>,
}

impl InformacjePoAkcieRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(InformacjePoAkcieRow {
            wpis_id: row.get("wpis_id")?,
            akt_sporzadzono: row.get("akt_sporzadzono")?,
            wniosek_kw_zlozono: row.get("wniosek_kw_zlozono")?,
            chwila: row.get("chwila")?,
            poprawia_id: row.get("poprawia_id")?,
            tresc: row.get("tresc")?,
        })
    }
}

/// To, co kancelaria przekazała po akcie: sporządzenie aktu i złożenie wniosku do księgi.
pub fn informacje_po_akcie(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<InformacjePoAkcieRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT po_akcie.*, wpis.chwila, wpis.poprawia_id, wpis.tresc
FROM wymiana_akt_po_akcie AS po_akcie
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = po_akcie.wpis_id
ORDER BY wpis.rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(InformacjePoAkcieRow::from_row).collect()
}

/// The parameters of `podaj_informacje_po_akcie`.
#[derive(Debug, Clone, PartialEq)]
pub struct PodajInformacjePoAkcieParams {
    pub wpis_id: String,
    pub strona: Strona,
    pub autor: String,
    pub chwila: String,
    pub poprawia_id: Option<String>,
    pub tresc: Option<String>,
    pub akt_sporzadzono: Option<String>,
    pub wniosek_kw_zlozono: Option<String>,
}

/// Kancelaria przekazuje, że akt sporządzono i kiedy złożyła wniosek do księgi wieczystej.
pub fn podaj_informacje_po_akcie(
    room: &impl xmeta_room::MetaRoom,
    params: &PodajInformacjePoAkcieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, poprawia_id, tresc)
VALUES (:wpis_id, :strona, :autor, 'po_akcie', :chwila, :poprawia_id, :tresc)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    ("strona", xmeta_room::ToSqlValue::to_sql_value(&params.strona)),
                    ("autor", xmeta_room::ToSqlValue::to_sql_value(&params.autor)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    (
                        "poprawia_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.poprawia_id),
                    ),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                ]),
            },
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_akt_po_akcie (wpis_id, akt_sporzadzono, wniosek_kw_zlozono)
VALUES (:wpis_id, :akt_sporzadzono, :wniosek_kw_zlozono)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    (
                        "akt_sporzadzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.akt_sporzadzono),
                    ),
                    (
                        "wniosek_kw_zlozono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wniosek_kw_zlozono),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `terminy_aktu`.
#[derive(Debug, Clone, PartialEq)]
pub struct TerminyAktuRow {
    pub id: String,
    pub wpis_id: String,
    pub dzien: String,
    pub godzina: String,
    pub miejsce: String,
    pub czynnosc: TerminyCzynnosc,
    pub potwierdzila_biuro: bool,
    pub potwierdzila_kancelaria: bool,
    pub stan: Stan,
}

impl TerminyAktuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(TerminyAktuRow {
            id: row.get("id")?,
            wpis_id: row.get("wpis_id")?,
            dzien: row.get("dzien")?,
            godzina: row.get("godzina")?,
            miejsce: row.get("miejsce")?,
            czynnosc: row.get("czynnosc")?,
            potwierdzila_biuro: row.get("potwierdzila_biuro")?,
            potwierdzila_kancelaria: row.get("potwierdzila_kancelaria")?,
            stan: row.get("stan")?,
        })
    }
}

/// Terminy aktu z bieżącym stanem i tym, która strona już je potwierdziła.
pub fn terminy_aktu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<TerminyAktuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM wymiana_akt_terminy_biezace
ORDER BY dzien, godzina"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(TerminyAktuRow::from_row).collect()
}

/// A row of `stany_terminow`.
#[derive(Debug, Clone, PartialEq)]
pub struct StanyTerminowRow {
    pub wpis_id: String,
    pub termin_id: String,
    pub stan: TerminyStanyStan,
    pub decyzja_odnosnik: Option<String>,
    pub strona: Strona,
    pub autor: String,
    pub chwila: String,
    pub tresc: Option<String>,
}

impl StanyTerminowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(StanyTerminowRow {
            wpis_id: row.get("wpis_id")?,
            termin_id: row.get("termin_id")?,
            stan: row.get("stan")?,
            decyzja_odnosnik: row.get("decyzja_odnosnik")?,
            strona: row.get("strona")?,
            autor: row.get("autor")?,
            chwila: row.get("chwila")?,
            tresc: row.get("tresc")?,
        })
    }
}

/// Ogłoszenia stanów terminów obu stron, z decyzją przy potwierdzeniu biura.
pub fn stany_terminow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<StanyTerminowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT stan.*, wpis.strona, wpis.autor, wpis.chwila, wpis.tresc
FROM wymiana_akt_terminy_stany AS stan
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = stan.wpis_id
ORDER BY wpis.rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(StanyTerminowRow::from_row).collect()
}

/// The parameters of `zaproponuj_termin_aktu`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZaproponujTerminAktuParams {
    pub wpis_id: String,
    pub strona: Strona,
    pub autor: String,
    pub chwila: String,
    pub odpowiada_na_id: Option<String>,
    pub tresc: Option<String>,
    pub id: String,
    pub dzien: String,
    pub godzina: String,
    pub miejsce: String,
    pub czynnosc: TerminyCzynnosc,
}

/// Strona proponuje dzień, godzinę i miejsce aktu; termin czeka na potwierdzenie obu stron.
pub fn zaproponuj_termin_aktu(
    room: &impl xmeta_room::MetaRoom,
    params: &ZaproponujTerminAktuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id, tresc)
VALUES (:wpis_id, :strona, :autor, 'propozycja_terminu', :chwila, :odpowiada_na_id, :tresc)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    ("strona", xmeta_room::ToSqlValue::to_sql_value(&params.strona)),
                    ("autor", xmeta_room::ToSqlValue::to_sql_value(&params.autor)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    (
                        "odpowiada_na_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odpowiada_na_id),
                    ),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                ]),
            },
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_akt_terminy (id, wpis_id, dzien, godzina, miejsce, czynnosc)
VALUES (:id, :wpis_id, :dzien, :godzina, :miejsce, :czynnosc)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    ("dzien", xmeta_room::ToSqlValue::to_sql_value(&params.dzien)),
                    ("godzina", xmeta_room::ToSqlValue::to_sql_value(&params.godzina)),
                    ("miejsce", xmeta_room::ToSqlValue::to_sql_value(&params.miejsce)),
                    ("czynnosc", xmeta_room::ToSqlValue::to_sql_value(&params.czynnosc)),
                ]),
            },
        ],
    )
}

/// The parameters of `oglos_stan_terminu`.
#[derive(Debug, Clone, PartialEq)]
pub struct OglosStanTerminuParams {
    pub wpis_id: String,
    pub strona: Strona,
    pub autor: String,
    pub chwila: String,
    pub odpowiada_na_id: Option<String>,
    pub tresc: Option<String>,
    pub termin_id: String,
    pub stan: TerminyStanyStan,
    pub decyzja_odnosnik: Option<String>,
}

/// Strona potwierdza, przesuwa albo odwołuje termin, a kancelaria ogłasza, że akt się odbył.
/// Biuro potwierdza termin tylko z decyzją potwierdzenie_terminu z teczki.
pub fn oglos_stan_terminu(
    room: &impl xmeta_room::MetaRoom,
    params: &OglosStanTerminuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id, tresc)
VALUES (:wpis_id, :strona, :autor, 'stan_terminu', :chwila, :odpowiada_na_id, :tresc)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    ("strona", xmeta_room::ToSqlValue::to_sql_value(&params.strona)),
                    ("autor", xmeta_room::ToSqlValue::to_sql_value(&params.autor)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    (
                        "odpowiada_na_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odpowiada_na_id),
                    ),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                ]),
            },
            xmeta_room::Statement {
                sql: r#"INSERT INTO wymiana_akt_terminy_stany (wpis_id, termin_id, stan, decyzja_odnosnik)
VALUES (:wpis_id, :termin_id, :stan, :decyzja_odnosnik)"#,
                parameters: Vec::from([
                    ("wpis_id", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_id)),
                    (
                        "termin_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.termin_id),
                    ),
                    ("stan", xmeta_room::ToSqlValue::to_sql_value(&params.stan)),
                    (
                        "decyzja_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_odnosnik),
                    ),
                ]),
            },
        ],
    )
}
