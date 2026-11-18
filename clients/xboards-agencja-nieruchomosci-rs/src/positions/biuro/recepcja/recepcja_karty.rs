//! The kind `recepcja-karty` of the room `positions/biuro/recepcja`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "recepcja-karty";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Karty dla Kierownika biura: nowe zgłoszenie, pismo, którego Recepcja nie umiała przypisać do
-- żadnej sprawy, albo skarga na biuro, i to, jak Kierownik biura kartę rozstrzygnął.
--
-- Karta ma stałe pola, które Recepcja wypełniła z wiadomości albo rozmowy. Wolny tekst
-- zgłaszającego zostaje w skrzynce, a karta ma do niego tylko odnośnik. Dane zgłaszającego leżą na
-- karcie tylko dopóki karta czeka. Po rozstrzygnięciu muszą zniknąć, bo trafiły do działu albo
-- zgłoszenie odrzucono, a Recepcja nie trzyma danych klientów. Poprzedni stan wiersza zostaje
-- w dzienniku pokoju.
CREATE TABLE recepcja_karty_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. recepcja_karty_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('nowe_zgloszenie', 'pismo_bez_sprawy', 'skarga')),
  -- Każda wiadomość ma wpis w dzienniku, także rozmowa telefoniczna.
  wplyw_id INTEGER NOT NULL REFERENCES recepcja_wplywy_dziennik (id),
  wyslano TEXT NOT NULL,
  -- Czego chce zgłaszający: sprzedać, kupić, wynająć albo oddać lokal w zarząd.
  rodzaj_sprawy TEXT CHECK (rodzaj_sprawy IN ('sprzedaz', 'kupno', 'wynajem', 'zarzad')),
  -- Np. „mieszkanie”, „dom”, „lokal użytkowy”.
  rodzaj_nieruchomosci TEXT,
  miejscowosc TEXT,
  -- Kanał, którym zgłaszający chce dostać odpowiedź.
  kanal TEXT CHECK (kanal IN ('email', 'formularz', 'portal', 'telefon', 'skan_poczty')),
  -- Kto się zgłasza i jak się z nim kontaktować; znika po rozstrzygnięciu.
  zglaszajacy TEXT,
  kontakt TEXT,
  -- Wolny tekst zgłaszającego w skrzynce, tekstem.
  wiadomosc_odnosnik TEXT NOT NULL CHECK (wiadomosc_odnosnik <> ''),
  stan TEXT NOT NULL CHECK (stan IN ('czeka', 'rozstrzygnieta')),
  rozstrzygnieto TEXT,
  rozstrzygniecie TEXT CHECK (rozstrzygniecie IN ('przydzielona', 'odrzucona')),
  -- Dział, któremu Kierownik biura przydzielił kartę.
  dzial TEXT CHECK (dzial IN ('posrednictwo', 'transakcje', 'najem')),

  CONSTRAINT czekajace_zgloszenie_z_danymi CHECK (
    stan <> 'czeka'
    OR rodzaj <> 'nowe_zgloszenie'
    OR (rodzaj_sprawy IS NOT NULL AND zglaszajacy IS NOT NULL AND kontakt IS NOT NULL)
  ),
  CONSTRAINT rozstrzygnieta_bez_danych_zglaszajacego CHECK (
    stan <> 'rozstrzygnieta'
    OR (zglaszajacy IS NULL AND kontakt IS NULL)
  ),
  CONSTRAINT rozstrzygnieta_z_data_i_sposobem CHECK (
    (stan = 'rozstrzygnieta') = (rozstrzygnieto IS NOT NULL)
    AND (stan = 'rozstrzygnieta') = (rozstrzygniecie IS NOT NULL)
  ),
  CONSTRAINT dzial_przy_przydziale CHECK ((rozstrzygniecie IS 'przydzielona') = (dzial IS NOT NULL)),
  CONSTRAINT rozstrzygniecie_nie_przed_wyslaniem CHECK (rozstrzygnieto IS NULL OR rozstrzygnieto >= wyslano)
) STRICT;

CREATE TRIGGER recepcja_karty_powstaje_czekajaca
BEFORE INSERT ON recepcja_karty_wpisy
WHEN NEW.stan <> 'czeka'
BEGIN
  SELECT RAISE(ABORT, 'karta powstaje jako czekająca; rozstrzyga ją Kierownik biura');
END;

-- Wpis dziennika mówi, komu oddano wiadomość, więc karta stoi na wpisie oddanym na kartę.
CREATE TRIGGER recepcja_karty_wplyw_oddany_na_karte
BEFORE INSERT ON recepcja_karty_wpisy
WHEN NOT EXISTS (
  SELECT 1
  FROM recepcja_wplywy_dziennik AS wplyw
  WHERE wplyw.id = NEW.wplyw_id
    AND wplyw.oddano = 'karta'
)
BEGIN
  SELECT RAISE(ABORT, 'karta stoi na wpisie dziennika oddanym na kartę');
END;

-- Czekającą kartę zmienia tylko rozstrzygnięcie; rozstrzygniętej nie zmienia się wcale.
CREATE TRIGGER recepcja_karty_bez_zmian
BEFORE UPDATE ON recepcja_karty_wpisy
WHEN OLD.stan = 'rozstrzygnieta'
  OR NEW.id IS NOT OLD.id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.wplyw_id IS NOT OLD.wplyw_id
  OR NEW.wyslano IS NOT OLD.wyslano
  OR NEW.rodzaj_sprawy IS NOT OLD.rodzaj_sprawy
  OR NEW.rodzaj_nieruchomosci IS NOT OLD.rodzaj_nieruchomosci
  OR NEW.miejscowosc IS NOT OLD.miejscowosc
  OR NEW.kanal IS NOT OLD.kanal
  OR NEW.wiadomosc_odnosnik IS NOT OLD.wiadomosc_odnosnik
BEGIN
  SELECT RAISE(ABORT, 'wysłanej karty się nie zmienia; wolno ją raz rozstrzygnąć');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER recepcja_karty_wpisy_bez_zastepowania
BEFORE INSERT ON recepcja_karty_wpisy
WHEN EXISTS (
  SELECT 1
  FROM recepcja_karty_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: rozstrzygnięta karta zostaje bez danych zgłaszającego.
CREATE TRIGGER recepcja_karty_wpisy_bez_usuwania
BEFORE DELETE ON recepcja_karty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
];

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    NoweZgloszenie,
    PismoBezSprawy,
    Skarga,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::NoweZgloszenie,
        WpisyRodzaj::PismoBezSprawy,
        WpisyRodzaj::Skarga,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::NoweZgloszenie => "nowe_zgloszenie",
            WpisyRodzaj::PismoBezSprawy => "pismo_bez_sprawy",
            WpisyRodzaj::Skarga => "skarga",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "nowe_zgloszenie" => Ok(WpisyRodzaj::NoweZgloszenie),
                    "pismo_bez_sprawy" => Ok(WpisyRodzaj::PismoBezSprawy),
                    "skarga" => Ok(WpisyRodzaj::Skarga),
                    other => Err(format!("{other} is no value of WpisyRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.rodzaj_sprawy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzajSprawy {
    Sprzedaz,
    Kupno,
    Wynajem,
    Zarzad,
}

impl WpisyRodzajSprawy {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzajSprawy] = &[
        WpisyRodzajSprawy::Sprzedaz,
        WpisyRodzajSprawy::Kupno,
        WpisyRodzajSprawy::Wynajem,
        WpisyRodzajSprawy::Zarzad,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzajSprawy::Sprzedaz => "sprzedaz",
            WpisyRodzajSprawy::Kupno => "kupno",
            WpisyRodzajSprawy::Wynajem => "wynajem",
            WpisyRodzajSprawy::Zarzad => "zarzad",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzajSprawy {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "sprzedaz" => Ok(WpisyRodzajSprawy::Sprzedaz),
                    "kupno" => Ok(WpisyRodzajSprawy::Kupno),
                    "wynajem" => Ok(WpisyRodzajSprawy::Wynajem),
                    "zarzad" => Ok(WpisyRodzajSprawy::Zarzad),
                    other => Err(format!("{other} is no value of WpisyRodzajSprawy")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyRodzajSprawy {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.kanal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyKanal {
    Email,
    Formularz,
    Portal,
    Telefon,
    SkanPoczty,
}

impl WpisyKanal {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyKanal] = &[
        WpisyKanal::Email,
        WpisyKanal::Formularz,
        WpisyKanal::Portal,
        WpisyKanal::Telefon,
        WpisyKanal::SkanPoczty,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyKanal::Email => "email",
            WpisyKanal::Formularz => "formularz",
            WpisyKanal::Portal => "portal",
            WpisyKanal::Telefon => "telefon",
            WpisyKanal::SkanPoczty => "skan_poczty",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyKanal {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "email" => Ok(WpisyKanal::Email),
                    "formularz" => Ok(WpisyKanal::Formularz),
                    "portal" => Ok(WpisyKanal::Portal),
                    "telefon" => Ok(WpisyKanal::Telefon),
                    "skan_poczty" => Ok(WpisyKanal::SkanPoczty),
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

/// The values of column `wpisy.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyStan {
    Czeka,
    Rozstrzygnieta,
}

impl WpisyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyStan] = &[WpisyStan::Czeka, WpisyStan::Rozstrzygnieta];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyStan::Czeka => "czeka",
            WpisyStan::Rozstrzygnieta => "rozstrzygnieta",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "czeka" => Ok(WpisyStan::Czeka),
                    "rozstrzygnieta" => Ok(WpisyStan::Rozstrzygnieta),
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

/// The values of column `wpisy.rozstrzygniecie`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRozstrzygniecie {
    Przydzielona,
    Odrzucona,
}

impl WpisyRozstrzygniecie {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRozstrzygniecie] = &[
        WpisyRozstrzygniecie::Przydzielona,
        WpisyRozstrzygniecie::Odrzucona,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRozstrzygniecie::Przydzielona => "przydzielona",
            WpisyRozstrzygniecie::Odrzucona => "odrzucona",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRozstrzygniecie {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "przydzielona" => Ok(WpisyRozstrzygniecie::Przydzielona),
                    "odrzucona" => Ok(WpisyRozstrzygniecie::Odrzucona),
                    other => Err(format!("{other} is no value of WpisyRozstrzygniecie")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyRozstrzygniecie {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.dzial`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyDzial {
    Posrednictwo,
    Transakcje,
    Najem,
}

impl WpisyDzial {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyDzial] = &[
        WpisyDzial::Posrednictwo,
        WpisyDzial::Transakcje,
        WpisyDzial::Najem,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyDzial::Posrednictwo => "posrednictwo",
            WpisyDzial::Transakcje => "transakcje",
            WpisyDzial::Najem => "najem",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyDzial {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "posrednictwo" => Ok(WpisyDzial::Posrednictwo),
                    "transakcje" => Ok(WpisyDzial::Transakcje),
                    "najem" => Ok(WpisyDzial::Najem),
                    other => Err(format!("{other} is no value of WpisyDzial")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyDzial {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The parameters of `meldunek_dzienny`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MeldunekDziennyParams {
    pub dzien: String,
}

/// A row of `meldunek_dzienny`.
#[derive(Debug, Clone, PartialEq)]
pub struct MeldunekDziennyRow {
    pub wplynelo: i64,
    pub oddano_agentom: i64,
    pub zapytania_po_godzinie: i64,
    pub wyslano_kart: i64,
    pub karty_czekaja_od_wczesniej: i64,
    pub niejednoznaczne: i64,
}

impl MeldunekDziennyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(MeldunekDziennyRow {
            wplynelo: row.get("wplynelo")?,
            oddano_agentom: row.get("oddano_agentom")?,
            zapytania_po_godzinie: row.get("zapytania_po_godzinie")?,
            wyslano_kart: row.get("wyslano_kart")?,
            karty_czekaja_od_wczesniej: row.get("karty_czekaja_od_wczesniej")?,
            niejednoznaczne: row.get("niejednoznaczne")?,
        })
    }
}

/// Stałe pola meldunku Recepcji z jednego dnia (RRRR-MM-DD): ile wiadomości wpłynęło, ile oddano
/// agentom spraw, ile zapytań z portali i ze strony biura czekało dłużej niż godzinę, ile kart
/// wysłano, ile kart czeka od wcześniejszego dnia i ile wiadomości było niejednoznacznych. Kanały
/// podaje wplywy_wedlug_kanalow, a zakłócenia zdarzenia_zrodel. Meldunek jest wiadomością; pamięć
/// trzyma tylko to, z czego się go liczy.
pub fn meldunek_dzienny(
    room: &impl xmeta_room::MetaRoom,
    params: &MeldunekDziennyParams,
) -> Result<Option<MeldunekDziennyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT
  (
    SELECT count(*)
    FROM recepcja_wplywy_dziennik
    WHERE substr(wplynelo, 1, 10) = :dzien
  ) AS wplynelo,
  (
    SELECT count(*)
    FROM recepcja_wplywy_dziennik
    WHERE substr(wplynelo, 1, 10) = :dzien
      AND oddano = 'agent_sprawy'
  ) AS oddano_agentom,
  (
    SELECT count(*)
    FROM recepcja_wplywy_dziennik
    WHERE substr(wplynelo, 1, 10) = :dzien
      AND kanal IN ('portal', 'formularz')
      AND oddano_o IS NOT NULL
      AND julianday(oddano_o) - julianday(wplynelo) > 1.0 / 24
  ) AS zapytania_po_godzinie,
  (
    SELECT count(*)
    FROM recepcja_karty_wpisy
    WHERE substr(wyslano, 1, 10) = :dzien
  ) AS wyslano_kart,
  (
    SELECT count(*)
    FROM recepcja_karty_wpisy
    WHERE stan = 'czeka'
      AND substr(wyslano, 1, 10) < :dzien
  ) AS karty_czekaja_od_wczesniej,
  (
    SELECT count(*)
    FROM recepcja_karty_wpisy
    WHERE substr(wyslano, 1, 10) = :dzien
      AND rodzaj = 'pismo_bez_sprawy'
  ) AS niejednoznaczne"#,
        parameters: Vec::from([
            ("dzien", xmeta_room::ToSqlValue::to_sql_value(&params.dzien)),
        ]),
    };
    let rows = room.read(&statement)?;
    rows.first().map(MeldunekDziennyRow::from_row).transpose()
}

/// A row of `karty_czekajace`.
#[derive(Debug, Clone, PartialEq)]
pub struct KartyCzekajaceRow {
    pub id: i64,
    pub rodzaj: WpisyRodzaj,
    pub wplyw_id: i64,
    pub wyslano: String,
    pub rodzaj_sprawy: Option<WpisyRodzajSprawy>,
    pub rodzaj_nieruchomosci: Option<String>,
    pub miejscowosc: Option<String>,
    pub kanal: Option<WpisyKanal>,
    pub zglaszajacy: Option<String>,
    pub kontakt: Option<String>,
    pub wiadomosc_odnosnik: String,
    pub stan: WpisyStan,
    pub rozstrzygnieto: Option<String>,
    pub rozstrzygniecie: Option<WpisyRozstrzygniecie>,
    pub dzial: Option<WpisyDzial>,
}

impl KartyCzekajaceRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(KartyCzekajaceRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            wplyw_id: row.get("wplyw_id")?,
            wyslano: row.get("wyslano")?,
            rodzaj_sprawy: row.get("rodzaj_sprawy")?,
            rodzaj_nieruchomosci: row.get("rodzaj_nieruchomosci")?,
            miejscowosc: row.get("miejscowosc")?,
            kanal: row.get("kanal")?,
            zglaszajacy: row.get("zglaszajacy")?,
            kontakt: row.get("kontakt")?,
            wiadomosc_odnosnik: row.get("wiadomosc_odnosnik")?,
            stan: row.get("stan")?,
            rozstrzygnieto: row.get("rozstrzygnieto")?,
            rozstrzygniecie: row.get("rozstrzygniecie")?,
            dzial: row.get("dzial")?,
        })
    }
}

/// Karty, które czekają na Kierownika biura, od najstarszej.
pub fn karty_czekajace(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<KartyCzekajaceRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM recepcja_karty_wpisy
WHERE stan = 'czeka'
ORDER BY wyslano, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(KartyCzekajaceRow::from_row).collect()
}

/// The parameters of `wyslij_karte`.
#[derive(Debug, Clone, PartialEq)]
pub struct WyslijKarteParams {
    pub rodzaj: WpisyRodzaj,
    pub wplyw_id: i64,
    pub wyslano: String,
    pub rodzaj_sprawy: Option<WpisyRodzajSprawy>,
    pub rodzaj_nieruchomosci: Option<String>,
    pub miejscowosc: Option<String>,
    pub kanal: Option<WpisyKanal>,
    pub zglaszajacy: Option<String>,
    pub kontakt: Option<String>,
    pub wiadomosc_odnosnik: String,
}

/// Karta wysłana Kierownikowi biura z wpisu dziennika oddanego na kartę.
pub fn wyslij_karte(
    room: &impl xmeta_room::MetaRoom,
    params: &WyslijKarteParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO recepcja_karty_wpisy (rodzaj, wplyw_id, wyslano, rodzaj_sprawy, rodzaj_nieruchomosci,
  miejscowosc, kanal, zglaszajacy, kontakt, wiadomosc_odnosnik, stan)
VALUES (:rodzaj, :wplyw_id, :wyslano, :rodzaj_sprawy, :rodzaj_nieruchomosci, :miejscowosc, :kanal,
  :zglaszajacy, :kontakt, :wiadomosc_odnosnik, 'czeka')"#,
                parameters: Vec::from([
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("wplyw_id", xmeta_room::ToSqlValue::to_sql_value(&params.wplyw_id)),
                    ("wyslano", xmeta_room::ToSqlValue::to_sql_value(&params.wyslano)),
                    (
                        "rodzaj_sprawy",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj_sprawy),
                    ),
                    (
                        "rodzaj_nieruchomosci",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.rodzaj_nieruchomosci,
                        ),
                    ),
                    (
                        "miejscowosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.miejscowosc),
                    ),
                    ("kanal", xmeta_room::ToSqlValue::to_sql_value(&params.kanal)),
                    (
                        "zglaszajacy",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zglaszajacy),
                    ),
                    ("kontakt", xmeta_room::ToSqlValue::to_sql_value(&params.kontakt)),
                    (
                        "wiadomosc_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wiadomosc_odnosnik),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_rozstrzygniecie_karty`.
#[derive(Debug, Clone, PartialEq)]
pub struct OdnotujRozstrzygniecieKartyParams {
    pub rozstrzygnieto: String,
    pub rozstrzygniecie: WpisyRozstrzygniecie,
    pub dzial: Option<WpisyDzial>,
    pub id: i64,
}

/// Kierownik biura przydzielił kartę działowi albo ją odrzucił; dane zgłaszającego znikają z karty.
pub fn odnotuj_rozstrzygniecie_karty(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujRozstrzygniecieKartyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE recepcja_karty_wpisy
SET stan = 'rozstrzygnieta',
  rozstrzygnieto = :rozstrzygnieto,
  rozstrzygniecie = :rozstrzygniecie,
  dzial = :dzial,
  zglaszajacy = NULL,
  kontakt = NULL
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "rozstrzygnieto",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rozstrzygnieto),
                    ),
                    (
                        "rozstrzygniecie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rozstrzygniecie),
                    ),
                    ("dzial", xmeta_room::ToSqlValue::to_sql_value(&params.dzial)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
