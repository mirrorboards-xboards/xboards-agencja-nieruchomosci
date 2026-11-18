//! The kind `kierownik-dzialu-agenci` of the room `positions/kierownik_dzialu`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "kierownik-dzialu-agenci";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Agenci, których powołał kierownik działu, i każde wpuszczenie agenta do pokoju, które kierownik
-- zrobił albo, w Kontroli, o które poprosił.
--
-- Agent sprawy pracuje w jednej teczce. Koordynator operacji nie ma teczki, bo obsługuje lokale
-- obszaru (obszar wskazuje go w pamięci Kierownika najmu). Kontroler nie ma teczki, bo pracuje dla
-- klienta, a do teczek klienta wpuszczają go kierownicy działów.
--
-- Wymiana jest ostateczna i wskazuje następcę. Następca przejmuje tę samą pracę: to samo
-- stanowisko, tę samą teczkę, a u Kontrolera tego samego klienta.
--
-- W okresie wdrożenia człowiek zatwierdza każde powołanie i każde wpuszczenie. Jego podpis
-- przychodzi wiadomością, a kierownik przenosi go do wiersza; sprawdza go aplikacja. Kiedy okres
-- wdrożenia trwa, ustala właściciel, więc baza wymaga tylko kompletu kolumn zatwierdzenia.
CREATE TABLE kierownik_dzialu_agenci_wpisy (
  id TEXT PRIMARY KEY,
  stanowisko TEXT NOT NULL CHECK (stanowisko IN ('opiekun_oferty', 'opiekun_poszukujacego', 'analityk_kredytowy', 'analityk_stanu_prawnego', 'koordynator_transakcji', 'opiekun_najmu', 'zarzadca_obiektu', 'konsjerz', 'koordynator_operacji', 'kontroler')),
  tozsamosc TEXT NOT NULL,
  -- Człowiek, który jest właścicielem agenta, np. pośrednik, który założył teczkę oferty.
  wlasciciel TEXT NOT NULL,
  -- Pokój teczki, tekstem.
  teczka_odnosnik TEXT,
  -- Klient Kontrolera: wpis w kartotece klientów, tekstem.
  klient_odnosnik TEXT,
  powolano TEXT NOT NULL,
  stan TEXT NOT NULL CHECK (stan IN ('pracuje', 'spi', 'wymieniony')),
  wymieniono TEXT,
  powod_wymiany TEXT,
  nastepca_id TEXT REFERENCES kierownik_dzialu_agenci_wpisy (id),
  zatwierdzil TEXT,
  podpis TEXT,
  zatwierdzono TEXT,

  CONSTRAINT teczka_poza_koordynatorem_operacji_i_kontrolerem CHECK ((stanowisko IN ('koordynator_operacji', 'kontroler')) = (teczka_odnosnik IS NULL)),
  CONSTRAINT klient_tylko_u_kontrolera CHECK ((stanowisko = 'kontroler') = (klient_odnosnik IS NOT NULL)),
  CONSTRAINT wymiana_z_powodem_data_i_nastepca CHECK (
    (stan = 'wymieniony') = (wymieniono IS NOT NULL)
    AND (stan = 'wymieniony') = (powod_wymiany IS NOT NULL)
    AND (stan = 'wymieniony') = (nastepca_id IS NOT NULL)
  ),
  CONSTRAINT zatwierdzenie_w_komplecie CHECK (
    (zatwierdzil IS NULL) = (podpis IS NULL)
    AND (zatwierdzil IS NULL) = (zatwierdzono IS NULL)
  )
) STRICT;

-- Każde wpuszczenie agenta do teczki albo pokoju wymiany, także agenta innego działu (np. Kontrolera
-- na wskazanie Kierownika kontroli albo Analityka kredytowego na prośbę Kierownika transakcji).
-- Agent tego kierownika ma wskazany swój wiersz. Kto poprosił, to kierownik, który wskazał agenta,
-- a jego prośba to wiadomość; kto wpuścił, to kierownik, który dodał agenta do pokoju.
CREATE TABLE kierownik_dzialu_agenci_wpuszczenia (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_dzialu_agenci_wpuszczenia_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  tozsamosc TEXT NOT NULL,
  stanowisko TEXT NOT NULL CHECK (stanowisko IN ('opiekun_oferty', 'opiekun_poszukujacego', 'analityk_kredytowy', 'analityk_stanu_prawnego', 'koordynator_transakcji', 'opiekun_najmu', 'zarzadca_obiektu', 'konsjerz', 'koordynator_operacji', 'kontroler')),
  agent_id TEXT REFERENCES kierownik_dzialu_agenci_wpisy (id),
  -- Pokój teczki albo pokój wymiany, tekstem.
  pokoj_odnosnik TEXT NOT NULL CHECK (pokoj_odnosnik <> ''),
  -- Klient Kontrolera: wpis w kartotece klientów, tekstem.
  klient_odnosnik TEXT,
  prosil TEXT,
  prosba_odnosnik TEXT,
  wpuscil TEXT NOT NULL,
  wpuszczono TEXT NOT NULL,
  usunieto TEXT,
  zatwierdzil TEXT,
  podpis TEXT,
  zatwierdzono TEXT,

  CONSTRAINT klient_tylko_u_kontrolera CHECK ((stanowisko = 'kontroler') = (klient_odnosnik IS NOT NULL)),
  CONSTRAINT prosba_razem_z_proszacym CHECK ((prosil IS NULL) = (prosba_odnosnik IS NULL)),
  CONSTRAINT usuniecie_nie_przed_wpuszczeniem CHECK (usunieto IS NULL OR usunieto >= wpuszczono),
  CONSTRAINT zatwierdzenie_w_komplecie CHECK (
    (zatwierdzil IS NULL) = (podpis IS NULL)
    AND (zatwierdzil IS NULL) = (zatwierdzono IS NULL)
  )
) STRICT;

-- Tożsamość jest w pokoju raz, więc otwarte wpuszczenie jednej tożsamości do jednego pokoju jest
-- jedno; ponowne wpuszczenie przychodzi po usunięciu.
CREATE UNIQUE INDEX kierownik_dzialu_agenci_jedno_otwarte_wpuszczenie
ON kierownik_dzialu_agenci_wpuszczenia (tozsamosc, pokoj_odnosnik)
WHERE usunieto IS NULL;

-- Drugie otwarte wpuszczenie z innym kluczem zderzyłoby się z indeksem, a INSERT OR REPLACE usunąłby
-- wtedy pierwsze z ominięciem blokady usuwania; ten trigger odmawia wcześniej.
CREATE TRIGGER kierownik_dzialu_agenci_jedno_otwarte_wpuszczenie_do_pokoju
BEFORE INSERT ON kierownik_dzialu_agenci_wpuszczenia
WHEN NEW.usunieto IS NULL
  AND EXISTS (
    SELECT 1
    FROM kierownik_dzialu_agenci_wpuszczenia
    WHERE tozsamosc = NEW.tozsamosc
      AND pokoj_odnosnik = NEW.pokoj_odnosnik
      AND usunieto IS NULL
      AND id IS NOT NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'ta tożsamość ma już otwarte wpuszczenie do tego pokoju; nowe przychodzi po usunięciu');
END;

-- Agenci -----------------------------------------------------------------------------------------

-- Reguła następcy sprawdza wymianę, więc agent nie może powstać od razu wymieniony.
CREATE TRIGGER kierownik_dzialu_agenci_powstaje_pracujacy
BEFORE INSERT ON kierownik_dzialu_agenci_wpisy
WHEN NEW.stan <> 'pracuje'
BEGIN
  SELECT RAISE(ABORT, 'agent powstaje jako pracujący');
END;

CREATE TRIGGER kierownik_dzialu_agenci_wymiana_jest_ostateczna
BEFORE UPDATE ON kierownik_dzialu_agenci_wpisy
WHEN OLD.stan = 'wymieniony'
BEGIN
  SELECT RAISE(ABORT, 'wymienionego agenta się nie zmienia');
END;

-- Na stanowisku, teczce i kliencie stoi reguła następcy, a zatwierdzenie obejmuje powołanie.
CREATE TRIGGER kierownik_dzialu_agenci_powolanie_niezmienne
BEFORE UPDATE ON kierownik_dzialu_agenci_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.stanowisko IS NOT OLD.stanowisko
  OR NEW.tozsamosc IS NOT OLD.tozsamosc
  OR NEW.teczka_odnosnik IS NOT OLD.teczka_odnosnik
  OR NEW.klient_odnosnik IS NOT OLD.klient_odnosnik
  OR NEW.powolano IS NOT OLD.powolano
  OR (OLD.zatwierdzil IS NOT NULL AND (
    NEW.zatwierdzil IS NOT OLD.zatwierdzil
    OR NEW.podpis IS NOT OLD.podpis
    OR NEW.zatwierdzono IS NOT OLD.zatwierdzono
  ))
BEGIN
  SELECT RAISE(ABORT, 'stanowiska, tożsamości, teczki, klienta, daty powołania ani zatwierdzenia agenta się nie zmienia');
END;

CREATE TRIGGER kierownik_dzialu_agenci_nastepca_przejmuje_te_sama_prace
BEFORE UPDATE OF nastepca_id ON kierownik_dzialu_agenci_wpisy
WHEN NEW.nastepca_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM kierownik_dzialu_agenci_wpisy AS nastepca
    WHERE nastepca.id = NEW.nastepca_id
      AND nastepca.id <> NEW.id
      AND nastepca.stanowisko = NEW.stanowisko
      AND nastepca.teczka_odnosnik IS NEW.teczka_odnosnik
      AND nastepca.klient_odnosnik IS NEW.klient_odnosnik
  )
BEGIN
  SELECT RAISE(ABORT, 'następca ma to samo stanowisko i tę samą teczkę, a Kontroler tego samego klienta');
END;

-- Wpuszczenia ------------------------------------------------------------------------------------

CREATE TRIGGER kierownik_dzialu_agenci_wpuszczenie_powstaje_otwarte
BEFORE INSERT ON kierownik_dzialu_agenci_wpuszczenia
WHEN NEW.usunieto IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'wpuszczenie powstaje otwarte; usunięcie odnotowuje się później');
END;

-- Wpuszczenie własnego agenta powtarza jego tożsamość, stanowisko i klienta, bo te kolumny wpuszczeń
-- agentów innych działów są jedynym zapisem, kogo kierownik wpuścił.
CREATE TRIGGER kierownik_dzialu_agenci_wpuszczenie_zgodne_z_agentem
BEFORE INSERT ON kierownik_dzialu_agenci_wpuszczenia
WHEN NEW.agent_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM kierownik_dzialu_agenci_wpisy AS agent
    WHERE agent.id = NEW.agent_id
      AND agent.tozsamosc = NEW.tozsamosc
      AND agent.stanowisko = NEW.stanowisko
      AND agent.klient_odnosnik IS NEW.klient_odnosnik
  )
BEGIN
  SELECT RAISE(ABORT, 'wpuszczenie własnego agenta ma jego tożsamość, stanowisko i klienta');
END;

-- Wpuszczenie wolno raz zamknąć usunięciem i raz zatwierdzić; reszta jest śladem.
CREATE TRIGGER kierownik_dzialu_agenci_wpuszczenia_bez_zmian
BEFORE UPDATE ON kierownik_dzialu_agenci_wpuszczenia
WHEN NEW.id IS NOT OLD.id
  OR NEW.tozsamosc IS NOT OLD.tozsamosc
  OR NEW.stanowisko IS NOT OLD.stanowisko
  OR NEW.agent_id IS NOT OLD.agent_id
  OR NEW.pokoj_odnosnik IS NOT OLD.pokoj_odnosnik
  OR NEW.klient_odnosnik IS NOT OLD.klient_odnosnik
  OR NEW.prosil IS NOT OLD.prosil
  OR NEW.prosba_odnosnik IS NOT OLD.prosba_odnosnik
  OR NEW.wpuscil IS NOT OLD.wpuscil
  OR NEW.wpuszczono IS NOT OLD.wpuszczono
  OR (OLD.usunieto IS NOT NULL AND NEW.usunieto IS NOT OLD.usunieto)
  OR (OLD.zatwierdzil IS NOT NULL AND (
    NEW.zatwierdzil IS NOT OLD.zatwierdzil
    OR NEW.podpis IS NOT OLD.podpis
    OR NEW.zatwierdzono IS NOT OLD.zatwierdzono
  ))
BEGIN
  SELECT RAISE(ABORT, 'wpuszczenia się nie zmienia; wolno raz odnotować usunięcie i raz zatwierdzenie');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabeli z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER kierownik_dzialu_agenci_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_dzialu_agenci_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_dzialu_agenci_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER kierownik_dzialu_agenci_wpuszczenia_bez_zastepowania
BEFORE INSERT ON kierownik_dzialu_agenci_wpuszczenia
WHEN EXISTS (
  SELECT 1
  FROM kierownik_dzialu_agenci_wpuszczenia
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: wymieniony agent i zamknięte wpuszczenie zostają jako ślad.
CREATE TRIGGER kierownik_dzialu_agenci_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_dzialu_agenci_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER kierownik_dzialu_agenci_wpuszczenia_bez_usuwania
BEFORE DELETE ON kierownik_dzialu_agenci_wpuszczenia
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
];

/// The values of column `wpisy.stanowisko`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyStanowisko {
    OpiekunOferty,
    OpiekunPoszukujacego,
    AnalitykKredytowy,
    AnalitykStanuPrawnego,
    KoordynatorTransakcji,
    OpiekunNajmu,
    ZarzadcaObiektu,
    Konsjerz,
    KoordynatorOperacji,
    Kontroler,
}

impl WpisyStanowisko {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyStanowisko] = &[
        WpisyStanowisko::OpiekunOferty,
        WpisyStanowisko::OpiekunPoszukujacego,
        WpisyStanowisko::AnalitykKredytowy,
        WpisyStanowisko::AnalitykStanuPrawnego,
        WpisyStanowisko::KoordynatorTransakcji,
        WpisyStanowisko::OpiekunNajmu,
        WpisyStanowisko::ZarzadcaObiektu,
        WpisyStanowisko::Konsjerz,
        WpisyStanowisko::KoordynatorOperacji,
        WpisyStanowisko::Kontroler,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyStanowisko::OpiekunOferty => "opiekun_oferty",
            WpisyStanowisko::OpiekunPoszukujacego => "opiekun_poszukujacego",
            WpisyStanowisko::AnalitykKredytowy => "analityk_kredytowy",
            WpisyStanowisko::AnalitykStanuPrawnego => "analityk_stanu_prawnego",
            WpisyStanowisko::KoordynatorTransakcji => "koordynator_transakcji",
            WpisyStanowisko::OpiekunNajmu => "opiekun_najmu",
            WpisyStanowisko::ZarzadcaObiektu => "zarzadca_obiektu",
            WpisyStanowisko::Konsjerz => "konsjerz",
            WpisyStanowisko::KoordynatorOperacji => "koordynator_operacji",
            WpisyStanowisko::Kontroler => "kontroler",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyStanowisko {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "opiekun_oferty" => Ok(WpisyStanowisko::OpiekunOferty),
                    "opiekun_poszukujacego" => Ok(WpisyStanowisko::OpiekunPoszukujacego),
                    "analityk_kredytowy" => Ok(WpisyStanowisko::AnalitykKredytowy),
                    "analityk_stanu_prawnego" => {
                        Ok(WpisyStanowisko::AnalitykStanuPrawnego)
                    }
                    "koordynator_transakcji" => {
                        Ok(WpisyStanowisko::KoordynatorTransakcji)
                    }
                    "opiekun_najmu" => Ok(WpisyStanowisko::OpiekunNajmu),
                    "zarzadca_obiektu" => Ok(WpisyStanowisko::ZarzadcaObiektu),
                    "konsjerz" => Ok(WpisyStanowisko::Konsjerz),
                    "koordynator_operacji" => Ok(WpisyStanowisko::KoordynatorOperacji),
                    "kontroler" => Ok(WpisyStanowisko::Kontroler),
                    other => Err(format!("{other} is no value of WpisyStanowisko")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyStanowisko {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyStan {
    Pracuje,
    Spi,
    Wymieniony,
}

impl WpisyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyStan] = &[
        WpisyStan::Pracuje,
        WpisyStan::Spi,
        WpisyStan::Wymieniony,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyStan::Pracuje => "pracuje",
            WpisyStan::Spi => "spi",
            WpisyStan::Wymieniony => "wymieniony",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "pracuje" => Ok(WpisyStan::Pracuje),
                    "spi" => Ok(WpisyStan::Spi),
                    "wymieniony" => Ok(WpisyStan::Wymieniony),
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

/// A row of `agenci`.
#[derive(Debug, Clone, PartialEq)]
pub struct AgenciRow {
    pub id: String,
    pub stanowisko: WpisyStanowisko,
    pub tozsamosc: String,
    pub wlasciciel: String,
    pub teczka_odnosnik: Option<String>,
    pub klient_odnosnik: Option<String>,
    pub powolano: String,
    pub stan: WpisyStan,
    pub wymieniono: Option<String>,
    pub powod_wymiany: Option<String>,
    pub nastepca_id: Option<String>,
    pub zatwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub zatwierdzono: Option<String>,
}

impl AgenciRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(AgenciRow {
            id: row.get("id")?,
            stanowisko: row.get("stanowisko")?,
            tozsamosc: row.get("tozsamosc")?,
            wlasciciel: row.get("wlasciciel")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            klient_odnosnik: row.get("klient_odnosnik")?,
            powolano: row.get("powolano")?,
            stan: row.get("stan")?,
            wymieniono: row.get("wymieniono")?,
            powod_wymiany: row.get("powod_wymiany")?,
            nastepca_id: row.get("nastepca_id")?,
            zatwierdzil: row.get("zatwierdzil")?,
            podpis: row.get("podpis")?,
            zatwierdzono: row.get("zatwierdzono")?,
        })
    }
}

/// Wszyscy agenci powołani przez kierownika; najpierw pracujący, potem śpiący i wymienieni.
pub fn agenci(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<AgenciRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_dzialu_agenci_wpisy
ORDER BY stan = 'wymieniony', stan = 'spi', powolano, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(AgenciRow::from_row).collect()
}

/// The parameters of `agenci_teczki`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AgenciTeczkiParams {
    pub teczka_odnosnik: String,
}

/// A row of `agenci_teczki`.
#[derive(Debug, Clone, PartialEq)]
pub struct AgenciTeczkiRow {
    pub id: String,
    pub stanowisko: WpisyStanowisko,
    pub tozsamosc: String,
    pub wlasciciel: String,
    pub teczka_odnosnik: Option<String>,
    pub klient_odnosnik: Option<String>,
    pub powolano: String,
    pub stan: WpisyStan,
    pub wymieniono: Option<String>,
    pub powod_wymiany: Option<String>,
    pub nastepca_id: Option<String>,
    pub zatwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub zatwierdzono: Option<String>,
}

impl AgenciTeczkiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(AgenciTeczkiRow {
            id: row.get("id")?,
            stanowisko: row.get("stanowisko")?,
            tozsamosc: row.get("tozsamosc")?,
            wlasciciel: row.get("wlasciciel")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            klient_odnosnik: row.get("klient_odnosnik")?,
            powolano: row.get("powolano")?,
            stan: row.get("stan")?,
            wymieniono: row.get("wymieniono")?,
            powod_wymiany: row.get("powod_wymiany")?,
            nastepca_id: row.get("nastepca_id")?,
            zatwierdzil: row.get("zatwierdzil")?,
            podpis: row.get("podpis")?,
            zatwierdzono: row.get("zatwierdzono")?,
        })
    }
}

/// Agenci jednej teczki, także wymienieni, od pierwszego powołanego.
pub fn agenci_teczki(
    room: &impl xmeta_room::MetaRoom,
    params: &AgenciTeczkiParams,
) -> Result<Vec<AgenciTeczkiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_dzialu_agenci_wpisy
WHERE teczka_odnosnik = :teczka_odnosnik
ORDER BY powolano, id"#,
        parameters: Vec::from([
            (
                "teczka_odnosnik",
                xmeta_room::ToSqlValue::to_sql_value(&params.teczka_odnosnik),
            ),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(AgenciTeczkiRow::from_row).collect()
}

/// The parameters of `agenci_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AgenciKlientaParams {
    pub klient_odnosnik: String,
}

/// A row of `agenci_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct AgenciKlientaRow {
    pub id: String,
    pub stanowisko: WpisyStanowisko,
    pub tozsamosc: String,
    pub wlasciciel: String,
    pub teczka_odnosnik: Option<String>,
    pub klient_odnosnik: Option<String>,
    pub powolano: String,
    pub stan: WpisyStan,
    pub wymieniono: Option<String>,
    pub powod_wymiany: Option<String>,
    pub nastepca_id: Option<String>,
    pub zatwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub zatwierdzono: Option<String>,
}

impl AgenciKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(AgenciKlientaRow {
            id: row.get("id")?,
            stanowisko: row.get("stanowisko")?,
            tozsamosc: row.get("tozsamosc")?,
            wlasciciel: row.get("wlasciciel")?,
            teczka_odnosnik: row.get("teczka_odnosnik")?,
            klient_odnosnik: row.get("klient_odnosnik")?,
            powolano: row.get("powolano")?,
            stan: row.get("stan")?,
            wymieniono: row.get("wymieniono")?,
            powod_wymiany: row.get("powod_wymiany")?,
            nastepca_id: row.get("nastepca_id")?,
            zatwierdzil: row.get("zatwierdzil")?,
            podpis: row.get("podpis")?,
            zatwierdzono: row.get("zatwierdzono")?,
        })
    }
}

/// Kontrolerzy jednego klienta, także wymienieni, od pierwszego powołanego.
pub fn agenci_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &AgenciKlientaParams,
) -> Result<Vec<AgenciKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_dzialu_agenci_wpisy
WHERE klient_odnosnik = :klient_odnosnik
ORDER BY powolano, id"#,
        parameters: Vec::from([
            (
                "klient_odnosnik",
                xmeta_room::ToSqlValue::to_sql_value(&params.klient_odnosnik),
            ),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(AgenciKlientaRow::from_row).collect()
}

/// The parameters of `powolaj_agenta`.
#[derive(Debug, Clone, PartialEq)]
pub struct PowolajAgentaParams {
    pub id: String,
    pub stanowisko: WpisyStanowisko,
    pub tozsamosc: String,
    pub wlasciciel: String,
    pub teczka_odnosnik: Option<String>,
    pub klient_odnosnik: Option<String>,
    pub powolano: String,
    pub zatwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub zatwierdzono: Option<String>,
}

/// Nowy agent do teczki, a Koordynator operacji do obszaru, a Kontroler do klienta; identyfikator
/// (UUID) nadaje aplikacja, żeby raporty, wpuszczenie i wymiana mogły go od razu wskazać.
/// Zatwierdzenie człowieka przychodzi z powołaniem w okresie wdrożenia.
pub fn powolaj_agenta(
    room: &impl xmeta_room::MetaRoom,
    params: &PowolajAgentaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_dzialu_agenci_wpisy (id, stanowisko, tozsamosc, wlasciciel, teczka_odnosnik,
  klient_odnosnik, powolano, stan, zatwierdzil, podpis, zatwierdzono)
VALUES (:id, :stanowisko, :tozsamosc, :wlasciciel, :teczka_odnosnik, :klient_odnosnik, :powolano,
  'pracuje', :zatwierdzil, :podpis, :zatwierdzono)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "stanowisko",
                        xmeta_room::ToSqlValue::to_sql_value(&params.stanowisko),
                    ),
                    (
                        "tozsamosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.tozsamosc),
                    ),
                    (
                        "wlasciciel",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wlasciciel),
                    ),
                    (
                        "teczka_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.teczka_odnosnik),
                    ),
                    (
                        "klient_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_odnosnik),
                    ),
                    ("powolano", xmeta_room::ToSqlValue::to_sql_value(&params.powolano)),
                    (
                        "zatwierdzil",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zatwierdzil),
                    ),
                    ("podpis", xmeta_room::ToSqlValue::to_sql_value(&params.podpis)),
                    (
                        "zatwierdzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zatwierdzono),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zatwierdz_powolanie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZatwierdzPowolanieParams {
    pub zatwierdzil: String,
    pub podpis: String,
    pub zatwierdzono: String,
    pub id: String,
}

/// Człowiek zatwierdził powołanie podpisaną wiadomością, gdy przyszła po powołaniu.
pub fn zatwierdz_powolanie(
    room: &impl xmeta_room::MetaRoom,
    params: &ZatwierdzPowolanieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_dzialu_agenci_wpisy
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

/// The parameters of `uspij_agenta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UspijAgentaParams {
    pub id: String,
}

/// Agent czeka, aż w jego sprawie coś się wydarzy.
pub fn uspij_agenta(
    room: &impl xmeta_room::MetaRoom,
    params: &UspijAgentaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_dzialu_agenci_wpisy
SET stan = 'spi'
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `obudz_agenta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ObudzAgentaParams {
    pub id: String,
}

/// Agent wraca do pracy, np. bo do jego sprawy przyszło pismo.
pub fn obudz_agenta(
    room: &impl xmeta_room::MetaRoom,
    params: &ObudzAgentaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_dzialu_agenci_wpisy
SET stan = 'pracuje'
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wymien_agenta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WymienAgentaParams {
    pub wymieniono: String,
    pub powod_wymiany: String,
    pub nastepca_id: String,
    pub id: String,
}

/// Wymiana agenta na następcę, którego kierownik powołał wcześniej do tej samej pracy.
pub fn wymien_agenta(
    room: &impl xmeta_room::MetaRoom,
    params: &WymienAgentaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_dzialu_agenci_wpisy
SET stan = 'wymieniony',
  wymieniono = :wymieniono,
  powod_wymiany = :powod_wymiany,
  nastepca_id = :nastepca_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "wymieniono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wymieniono),
                    ),
                    (
                        "powod_wymiany",
                        xmeta_room::ToSqlValue::to_sql_value(&params.powod_wymiany),
                    ),
                    (
                        "nastepca_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.nastepca_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `wpuszczenia_otwarte`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpuszczeniaOtwarteRow {
    pub id: i64,
    pub tozsamosc: String,
    pub stanowisko: WpisyStanowisko,
    pub agent_id: Option<String>,
    pub pokoj_odnosnik: String,
    pub klient_odnosnik: Option<String>,
    pub prosil: Option<String>,
    pub prosba_odnosnik: Option<String>,
    pub wpuscil: String,
    pub wpuszczono: String,
    pub usunieto: Option<String>,
    pub zatwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub zatwierdzono: Option<String>,
}

impl WpuszczeniaOtwarteRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WpuszczeniaOtwarteRow {
            id: row.get("id")?,
            tozsamosc: row.get("tozsamosc")?,
            stanowisko: row.get("stanowisko")?,
            agent_id: row.get("agent_id")?,
            pokoj_odnosnik: row.get("pokoj_odnosnik")?,
            klient_odnosnik: row.get("klient_odnosnik")?,
            prosil: row.get("prosil")?,
            prosba_odnosnik: row.get("prosba_odnosnik")?,
            wpuscil: row.get("wpuscil")?,
            wpuszczono: row.get("wpuszczono")?,
            usunieto: row.get("usunieto")?,
            zatwierdzil: row.get("zatwierdzil")?,
            podpis: row.get("podpis")?,
            zatwierdzono: row.get("zatwierdzono")?,
        })
    }
}

/// Agenci, którzy są teraz w pokojach działu, według pokoju.
pub fn wpuszczenia_otwarte(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WpuszczeniaOtwarteRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_dzialu_agenci_wpuszczenia
WHERE usunieto IS NULL
ORDER BY pokoj_odnosnik, wpuszczono, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(WpuszczeniaOtwarteRow::from_row).collect()
}

/// The parameters of `wpuszczenia_pokoju`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpuszczeniaPokojuParams {
    pub pokoj_odnosnik: String,
}

/// A row of `wpuszczenia_pokoju`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpuszczeniaPokojuRow {
    pub id: i64,
    pub tozsamosc: String,
    pub stanowisko: WpisyStanowisko,
    pub agent_id: Option<String>,
    pub pokoj_odnosnik: String,
    pub klient_odnosnik: Option<String>,
    pub prosil: Option<String>,
    pub prosba_odnosnik: Option<String>,
    pub wpuscil: String,
    pub wpuszczono: String,
    pub usunieto: Option<String>,
    pub zatwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub zatwierdzono: Option<String>,
}

impl WpuszczeniaPokojuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WpuszczeniaPokojuRow {
            id: row.get("id")?,
            tozsamosc: row.get("tozsamosc")?,
            stanowisko: row.get("stanowisko")?,
            agent_id: row.get("agent_id")?,
            pokoj_odnosnik: row.get("pokoj_odnosnik")?,
            klient_odnosnik: row.get("klient_odnosnik")?,
            prosil: row.get("prosil")?,
            prosba_odnosnik: row.get("prosba_odnosnik")?,
            wpuscil: row.get("wpuscil")?,
            wpuszczono: row.get("wpuszczono")?,
            usunieto: row.get("usunieto")?,
            zatwierdzil: row.get("zatwierdzil")?,
            podpis: row.get("podpis")?,
            zatwierdzono: row.get("zatwierdzono")?,
        })
    }
}

/// Wszystkie wpuszczenia do jednego pokoju, także zamknięte, od najstarszego.
pub fn wpuszczenia_pokoju(
    room: &impl xmeta_room::MetaRoom,
    params: &WpuszczeniaPokojuParams,
) -> Result<Vec<WpuszczeniaPokojuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_dzialu_agenci_wpuszczenia
WHERE pokoj_odnosnik = :pokoj_odnosnik
ORDER BY wpuszczono, id"#,
        parameters: Vec::from([
            (
                "pokoj_odnosnik",
                xmeta_room::ToSqlValue::to_sql_value(&params.pokoj_odnosnik),
            ),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(WpuszczeniaPokojuRow::from_row).collect()
}

/// The parameters of `odnotuj_wpuszczenie`.
#[derive(Debug, Clone, PartialEq)]
pub struct OdnotujWpuszczenieParams {
    pub tozsamosc: String,
    pub stanowisko: WpisyStanowisko,
    pub agent_id: Option<String>,
    pub pokoj_odnosnik: String,
    pub klient_odnosnik: Option<String>,
    pub prosil: Option<String>,
    pub prosba_odnosnik: Option<String>,
    pub wpuscil: String,
    pub wpuszczono: String,
    pub zatwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub zatwierdzono: Option<String>,
}

/// Agent wszedł do teczki albo pokoju wymiany: własny agent kierownika (agent_id) albo agent innego
/// działu na prośbę jego kierownika (prosil, prosba_odnosnik).
pub fn odnotuj_wpuszczenie(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWpuszczenieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_dzialu_agenci_wpuszczenia (tozsamosc, stanowisko, agent_id, pokoj_odnosnik,
  klient_odnosnik, prosil, prosba_odnosnik, wpuscil, wpuszczono, zatwierdzil, podpis, zatwierdzono)
VALUES (:tozsamosc, :stanowisko, :agent_id, :pokoj_odnosnik, :klient_odnosnik, :prosil,
  :prosba_odnosnik, :wpuscil, :wpuszczono, :zatwierdzil, :podpis, :zatwierdzono)"#,
                parameters: Vec::from([
                    (
                        "tozsamosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.tozsamosc),
                    ),
                    (
                        "stanowisko",
                        xmeta_room::ToSqlValue::to_sql_value(&params.stanowisko),
                    ),
                    ("agent_id", xmeta_room::ToSqlValue::to_sql_value(&params.agent_id)),
                    (
                        "pokoj_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.pokoj_odnosnik),
                    ),
                    (
                        "klient_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.klient_odnosnik),
                    ),
                    ("prosil", xmeta_room::ToSqlValue::to_sql_value(&params.prosil)),
                    (
                        "prosba_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.prosba_odnosnik),
                    ),
                    ("wpuscil", xmeta_room::ToSqlValue::to_sql_value(&params.wpuscil)),
                    (
                        "wpuszczono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wpuszczono),
                    ),
                    (
                        "zatwierdzil",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zatwierdzil),
                    ),
                    ("podpis", xmeta_room::ToSqlValue::to_sql_value(&params.podpis)),
                    (
                        "zatwierdzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zatwierdzono),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zatwierdz_wpuszczenie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZatwierdzWpuszczenieParams {
    pub zatwierdzil: String,
    pub podpis: String,
    pub zatwierdzono: String,
    pub id: i64,
}

/// Człowiek zatwierdził wpuszczenie podpisaną wiadomością, gdy przyszła po wpuszczeniu.
pub fn zatwierdz_wpuszczenie(
    room: &impl xmeta_room::MetaRoom,
    params: &ZatwierdzWpuszczenieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_dzialu_agenci_wpuszczenia
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

/// The parameters of `odnotuj_usuniecie_z_pokoju`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujUsuniecieZPokojuParams {
    pub usunieto: String,
    pub id: i64,
}

/// Agent wyszedł z pokoju, np. bo jego praca się skończyła albo kierownik go wymienił.
pub fn odnotuj_usuniecie_z_pokoju(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujUsuniecieZPokojuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_dzialu_agenci_wpuszczenia
SET usunieto = :usunieto
WHERE id = :id"#,
                parameters: Vec::from([
                    ("usunieto", xmeta_room::ToSqlValue::to_sql_value(&params.usunieto)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
